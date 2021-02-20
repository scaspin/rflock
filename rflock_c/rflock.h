#ifndef RFLOCK_H
#define RFLOCK_H

#include "mem.h"

// Read Fast Phase-Fair (ticket) Lock
#define RF_WINC 0x100
#define RF_WBITS 0x3
#define RF_PRES 0x2
#define RF_PHID 0x1
#define RF_PRESENT 0x3
#define RF_COMPLETED 0x4

#define CORES_MAX  40
int RF_CORES;

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
void rflock_init(rflock_t *lock, int cores)
{
    RF_CORES = cores;
    for (int i=0; i < RF_CORES; i++)
    {
	lock->read_status[i*16]= RF_COMPLETED;
    }

    lock->win = 0;
    lock->wout = 0;
}

/*
 *  Fast read Phase-Fair (ticket) Lock: read lock.
 */
void rflock_read_lock(rflock_t *lock, int core)
{
    unsigned int w;
    lock->read_status[core*16] = RF_PRESENT;
    w = lock->win & RF_WBITS;
    lock->read_status[core*16] = w & RF_PHID;

    while (((w & RF_PRES) != 0) && (w == (lock->win & RF_WBITS)))
    {
        cpu_relax();
    }
}

/*
 *  Phase-Fair (ticket) Lock: read unlock.
 */
void rflock_read_unlock(rflock_t *lock, int core)
{
    lock->read_status[core*16] = RF_COMPLETED;
}

/*
 *  Phase-Fair (ticket) Lock: write lock.
 */
void rflock_write_lock(rflock_t *lock)
{
    unsigned int wticket, read_waiting;

    // Wait until it is my turn to write-lock the resource
    wticket = __sync_fetch_and_add(&lock->win, RF_WINC) & ~RF_WBITS;
    while (wticket != lock->wout)
    {
        cpu_relax();
    }

    __sync_fetch_and_xor(&lock->win, RF_WBITS);
    read_waiting = lock->win & RF_PHID;

    for (int i = 0; i<RF_CORES ; i++)
    {
	while ((lock->read_status[i*16] != read_waiting) && (lock->read_status[i*16] != RF_COMPLETED))
	{
		cpu_relax();
	}
    }
}

/*
 *  Phase-Fair (ticket) Lock: write unlock.
 */
void rflock_write_unlock(rflock_t *lock)
{
    __sync_fetch_and_and(&lock->win, 0xFFFFFF01);
    lock->wout = lock->wout + RF_WINC; // only one writer should ever be here
}

#endif 
