/* runtime.c — the guest's memory, and the way out of a lifted function.
 *
 * `rc_reserve` maps the flat 4 GiB guest space with no access at all;
 * `rc_map` opens the ranges the harness fills — the image's sections, a
 * stack, a snapshot's ranges — as `tools/emu/callfn.py` maps them under
 * unicorn. A read or write anywhere else is a fault, and the fault names
 * the guest address, which is what unicorn's unmapped-access hook reports
 * and the price list for the next thing to lay out. `rc_call` enters a
 * lifted function and reports whether it trapped: a trap, a fault, an
 * unlifted target or a wrong return address is a longjmp back here with a
 * message, not a dead process. The generated `lifted.c` supplies
 * `rc_dispatch`.
 */
#include "recomp.h"

#include <setjmp.h>
#include <signal.h>
#include <stdio.h>
#include <sys/mman.h>
#include <unistd.h>

#define SPACE (1ull << 32)

static sigjmp_buf rc_jb;
static char rc_msg[256];
static uint8_t *rc_mem;          /* the reservation the fault handler translates against */
static volatile int rc_live;     /* inside rc_call: a fault is ours to catch */

uint8_t *rc_reserve(void) {
    void *p = mmap(NULL, SPACE, PROT_NONE, MAP_PRIVATE | MAP_ANON, -1, 0);
    if (p == MAP_FAILED)
        return NULL;
    rc_mem = (uint8_t *)p;
    return rc_mem;
}

/* Rounded out to the host's page — 16 KiB on Apple Silicon, against the
 * guest's 4 KiB — so a fault's granularity is the host page: a read within
 * the same 16 KiB as a mapped range does not fault. */
int rc_map(uint8_t *mem, uint32_t base, uint32_t size) {
    uint64_t pg = (uint64_t)getpagesize();
    uint64_t lo = base & ~(pg - 1), hi = ((uint64_t)base + size + pg - 1) & ~(pg - 1);
    return mprotect(mem + lo, hi - lo, PROT_READ | PROT_WRITE);
}

static void on_fault(int sig, siginfo_t *si, void *ctx) {
    (void)ctx;
    uint8_t *at = (uint8_t *)si->si_addr;
    if (rc_live && at >= rc_mem && at < rc_mem + SPACE) {
        snprintf(rc_msg, sizeof rc_msg, "fault: %08llx is not mapped", (unsigned long long)(at - rc_mem));
        siglongjmp(rc_jb, 1);
    }
    signal(sig, SIG_DFL);  /* not ours: die the ordinary way */
    raise(sig);
}

int rc_call(cpu_t *c, uint32_t entry) {
    struct sigaction sa = {0}, old_segv, old_bus;
    sa.sa_sigaction = on_fault;
    sa.sa_flags = SA_SIGINFO;
    sigaction(SIGSEGV, &sa, &old_segv);
    sigaction(SIGBUS, &sa, &old_bus);
    int rc = 0;
    if (sigsetjmp(rc_jb, 1)) {
        rc = 1;
    } else {
        rc_live = 1;
        rc_dispatch(c, entry);
    }
    rc_live = 0;
    sigaction(SIGSEGV, &old_segv, NULL);
    sigaction(SIGBUS, &old_bus, NULL);
    return rc;
}

const char *rc_trap_message(void) { return rc_msg; }

_Noreturn void rc_trap(cpu_t *c, uint32_t at, const char *why) {
    (void)c;
    snprintf(rc_msg, sizeof rc_msg, "%08x: %s", at, why);
    siglongjmp(rc_jb, 1);
}

_Noreturn void rc_badret(cpu_t *c, uint32_t at, uint32_t want) {
    snprintf(rc_msg, sizeof rc_msg, "%08x: the callee returned to %08x, not %08x", at, c->eip, want);
    siglongjmp(rc_jb, 1);
}

void rc_cpuid(cpu_t *c) { rc_trap(c, 0, "cpuid"); }
