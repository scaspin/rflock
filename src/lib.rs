#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::sync::atomic::{spin_loop_hint, AtomicUsize, Ordering};

pub struct RFLock {
    pub read_status: [AtomicUsize; 100],
    pub win: AtomicUsize,
    pub wout: AtomicUsize,
}

const CORES: usize = 100;

const WINC: usize = 0x100; // writer increment
const WBITS: usize = 0x3; // writer bits in rin
const PRES: usize = 0x2; // writer present bit
const PHID: usize = 0x1; // phase ID bit
const PRESENT: usize = 0x3; // reader present indicator
const COMPLETED: usize = 0x4; // reader completed indicator

const ZERO_MASK: usize = !255usize;

impl RFLock {
    pub const fn new() -> RFLock {
        RFLock {
            read_status: [AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0),AtomicUsize::new(0)],
            win: AtomicUsize::new(0),
            wout: AtomicUsize::new(0),
        }
    }

    pub fn read_lock(&self, c: usize) {
        self.read_status[c].store(PRESENT, Ordering::Relaxed);
        let w = self.win.load(Ordering::Relaxed) & PHID;
        while ((w & PRES) != 0) && (w == (self.win.load(Ordering::Relaxed) & WBITS)) {
            spin_loop_hint();
        }

    }

    pub fn read_unlock(&self, c: usize) {
        self.read_status[c].store(COMPLETED, Ordering::Relaxed);
    }

    pub fn write_lock(&self) {
        let wticket = self.win.fetch_add(WINC, Ordering::Relaxed) & !WBITS;
        while wticket != self.wout.load(Ordering::Relaxed) {
            spin_loop_hint();
        }
        let w = PRES | (wticket & PHID);
        self.win.fetch_xor(PRESENT, Ordering::Relaxed);
        let read_waiting = w & PHID;
        for c in 0..CORES {
            while (self.read_status[c].load(Ordering::Relaxed) != read_waiting) && (self.read_status[c].load(Ordering::Relaxed) != COMPLETED){
                spin_loop_hint();
            }
        }
    }

    pub fn write_unlock(&self) {
        self.win.fetch_add(0xFFFFFF01, Ordering::Relaxed);
        self.wout.fetch_add(WINC, Ordering::Relaxed);
    }
}

pub struct RFLock_C(pft_lock_struct);

impl RFLock_C {
    pub fn new() -> RFLock_C {
        let mut lock = pft_lock_struct {
            rin: 0,
            rout: 0,
            win: 0,
            wout: 0,
        };
        unsafe {
            pft_lock_init(&mut lock);
        }
        RFLock_C(lock)
    }

    pub fn read_lock(&self) {
        unsafe {
            let const_ptr = self as *const RFLock_C;
            let mut_ptr = const_ptr as *mut RFLock_C;
            pft_read_lock(&mut (*mut_ptr).0);
        }
    }

    pub fn read_unlock(&self) {
        unsafe {
            let const_ptr = self as *const RFLock_C;
            let mut_ptr = const_ptr as *mut RFLock_C;
            pft_read_unlock(&mut (*mut_ptr).0);
        }
    }

    pub fn write_lock(&self) {
        unsafe {
            let const_ptr = self as *const RFLock_C;
            let mut_ptr = const_ptr as *mut RFLock_C;
            pft_write_lock(&mut (*mut_ptr).0);
        }
    }

    pub fn write_unlock(&self) {
        unsafe {
            let const_ptr = self as *const RFLock_C;
            let mut_ptr = const_ptr as *mut RFLock_C;
            pft_write_unlock(&mut (*mut_ptr).0);
        }
    }
}
