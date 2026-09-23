/* runtime.c — the guest's memory, and the way out of a lifted function.
 *
 * `rc_reserve` maps the flat 4 GiB guest space; the harness writes the
 * image's sections into it at their preferred base and lays out a stack,
 * as `tools/emu/callfn.py` does under unicorn. `rc_call` enters a lifted
 * function and reports whether it trapped: a trap is a longjmp back here,
 * so a division by zero, an unlifted target or a wrong return address is an
 * answer with a message and not a dead process. The generated `lifted.c`
 * supplies `rc_dispatch`.
 */
#include "recomp.h"

#include <setjmp.h>
#include <stdio.h>
#include <sys/mman.h>

static jmp_buf rc_jb;
static char rc_msg[256];

uint8_t *rc_reserve(void) {
    void *p = mmap(NULL, 1ull << 32, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANON, -1, 0);
    return p == MAP_FAILED ? NULL : (uint8_t *)p;
}

int rc_call(cpu_t *c, uint32_t entry) {
    if (setjmp(rc_jb))
        return 1;
    rc_dispatch(c, entry);
    return 0;
}

const char *rc_trap_message(void) { return rc_msg; }

_Noreturn void rc_trap(cpu_t *c, uint32_t at, const char *why) {
    (void)c;
    snprintf(rc_msg, sizeof rc_msg, "%08x: %s", at, why);
    longjmp(rc_jb, 1);
}

_Noreturn void rc_badret(cpu_t *c, uint32_t at, uint32_t want) {
    snprintf(rc_msg, sizeof rc_msg, "%08x: the callee returned to %08x, not %08x", at, c->eip, want);
    longjmp(rc_jb, 1);
}

void rc_cpuid(cpu_t *c) { rc_trap(c, 0, "cpuid"); }
