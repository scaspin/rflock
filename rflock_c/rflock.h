#ifndef RFLOCK_H
#define RFLOCK_H

#include "mem.h"

// Read Fast Phase-Fair (ticket) Lock
#define WINC 0x100
#define WBITS 0x3
#define PRES 0x2
#define PHID 0x1
#define PRESENT 0x3
#define COMPLETED 0x4

#define CORES_MAX  40
int CORES;

typedef struct rflock_struct {
    volatile unsigned read_status[CORES_MAX * 16];
    
    volatile unsigned win;
    unsigned int _buf1[15];
    
    volatile unsigned int wout;
    unsigned int _buf2[15];
} __attribute ((aligned (16) )) rflock_t;


/*
 *  Fast read Phase-Fair (ticket) Lock: initialize.
 */
static inline void rflock_init(rflock_t *lock, int cores)
{
    CORES = cores;
    for (int i=0; i < CORES; i++)
    {
	lock->read_status[i*16]= COMPLETED;
    }

    lock->win = 0;
    lock->wout = 0;
}

/*
 *  Fast read Phase-Fair (ticket) Lock: read lock.
 */
static inline void rflock_read_lock(rflock_t *lock, int core)
{
    unsigned int w;
    lock->read_status[core*16] = PRESENT;
    w = lock->win & WBITS;
    lock->read_status[core*16] = w & PHID;

    while (((w & PRES) != 0) && (w == (lock->win & WBITS)))
    {
        cpu_relax();
    }
}

/*
 *  Phase-Fair (ticket) Lock: read unlock.
 */
static inline void rflock_read_unlock(rflock_t *lock, int core)
{
    lock->read_status[core*16] = COMPLETED;
}

/*
 *  Phase-Fair (ticket) Lock: write lock.
 */
static inline void rflock_write_lock(rflock_t *lock)
{
    unsigned int w, wticket, read_waiting;

    // Wait until it is my turn to write-lock the resource
    wticket = __sync_fetch_and_add(&lock->win, WINC) & ~WBITS;
    while (wticket != lock->wout)
    {
        cpu_relax();
    }

    __sync_fetch_and_xor(&lock->win, WBITS);
    read_waiting = lock->win & PHID;

    for (int i = 0; i<CORES ; i++)
    {
	while ((lock->read_status[i*16] != read_waiting) && (lock->read_status[i*16] != COMPLETED))
	{
		cpu_relax();
	}
    }
}

/*
 *  Phase-Fair (ticket) Lock: write unlock.
 */
static inline void rflock_write_unlock(rflock_t *lock)
{
    __sync_fetch_and_and(&lock->win, 0xFFFFFF01);
    lock->wout = lock->wout + WINC; // only one writer should ever be here
}

#endif 
