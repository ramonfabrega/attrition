/* issue_guard_test.c [rontrace.funcs] — the `@` issuer guard on the host.
 *
 * Built and run by `rondata::blind`'s `the_issuer_guard_reads_through_the_
 * coverage_jmp`. Exits non-zero, naming the case, when the guard refuses an
 * issuer it should call or calls one it should refuse. With the coverage
 * table `build.sh` wrote beside the DLL, it also plants each verb's issuer
 * the way `arm_all` does from that table's own record and asks the guard. */
#include <stdio.h>
#include <string.h>

typedef unsigned char u8;
typedef unsigned int u32;
#include "issue_guard.h"

static int failed;

static void expect(const char *what, int got, int want) {
    if (got != want) {
        printf("FAIL %s: the guard %s\n", what, got ? "called it" : "refused it (code 2)");
        failed = 1;
    }
}

/* `arm_all`'s write: `E9 rel32` to the stub over the first five bytes. */
static void plant(u8 *live, u32 va, u32 stub) {
    u32 rel = stub - (va + 5);
    live[0] = 0xE9;
    live[1] = (u8)rel, live[2] = (u8)(rel >> 8), live[3] = (u8)(rel >> 16), live[4] = (u8)(rel >> 24);
}

int main(int argc, char **argv) {
    const u32 va = 0x400000u + RVA_ISSUE_FLIGHT, stub = 0x0a310000u;
    const u8 *want = ISSUER_PROLOGUE_BYTES[11];
    u8 live[ISSUE_PROLOGUE], copy[24];

    /* cover=0: the shipped bytes, no stub. */
    memcpy(live, want, ISSUE_PROLOGUE);
    expect("cover=0, the shipped prologue", issuer_is_shipped(live, va, want, ISSUE_PROLOGUE, 0, 0, 0, 0, 0), 1);
    /* cover=1: the jmp over `push ebp; mov ebp, esp; sub esp, 0x1c` (six
     * bytes), the table's copy of them, the rest as shipped. run314. */
    memcpy(copy, want, 6);
    plant(live, va, stub);
    expect("cover=1, the jmp to its own stub", issuer_is_shipped(live, va, want, ISSUE_PROLOGUE, stub, copy, 6, 6, 0), 1);
    /* A jmp to anything but this entry's stub is not the coverage's. */
    plant(live, va, stub + 80);
    expect("cover=1, a jmp to another stub", issuer_is_shipped(live, va, want, ISSUE_PROLOGUE, stub, copy, 6, 6, 0), 0);
    /* A jmp with no stub planted is someone else's patch. */
    plant(live, va, stub);
    expect("cover=0, a jmp over the prologue", issuer_is_shipped(live, va, want, ISSUE_PROLOGUE, 0, 0, 0, 0, 0), 0);
    /* Another build: the displaced bytes differ. */
    copy[5] = 0x18;
    expect("cover=1, another build's displaced bytes", issuer_is_shipped(live, va, want, ISSUE_PROLOGUE, stub, copy, 6, 6, 0), 0);
    copy[5] = want[5];
    /* Another build: the bytes past the jmp differ. */
    live[7] = 0x61;
    expect("cover=1, another build's later bytes", issuer_is_shipped(live, va, want, ISSUE_PROLOGUE, stub, copy, 6, 6, 0), 0);
    live[7] = want[7];
    /* A copy with a rewritten branch is not the original's bytes. */
    expect("cover=1, a rewritten copy", issuer_is_shipped(live, va, want, ISSUE_PROLOGUE, stub, copy, 6, 10, 1), 0);
    /* A displaced range shorter than the jmp cannot be one of arm_all's. */
    expect("cover=1, a four-byte displaced range", issuer_is_shipped(live, va, want, ISSUE_PROLOGUE, stub, copy, 4, 4, 0), 0);

    if (argc > 1) {
        /* `funcs.py`'s record: rva, orig_len, code_len, nfix, pad, code[24],
         * fix_off[4], fix_target[4] — 52 bytes, sorted by rva. */
        FILE *f = fopen(argv[1], "rb");
        if (!f) {
            printf("FAIL cannot open %s\n", argv[1]);
            return 1;
        }
        static u8 table[65536 * 52];
        size_t len = fread(table, 1, sizeof table, f), rows = len / 52;
        fclose(f);
        for (u32 verb = 0; verb < ISSUE_VERBS; verb++) {
            const u8 *rec = 0;
            for (size_t i = 0; i < rows; i++) {
                u32 rva;
                memcpy(&rva, table + 52 * i, 4);
                if (rva == ISSUER_RVA[verb]) rec = table + 52 * i;
            }
            char what[96];
            snprintf(what, sizeof what, "verb %u (rva %06x) as the table arms it", verb, ISSUER_RVA[verb]);
            if (!rec) {
                printf("FAIL %s: not in the table\n", what);
                failed = 1;
                continue;
            }
            u32 orig_len = rec[4], code_len = rec[5], nfix = rec[6];
            const u8 *w = ISSUER_PROLOGUE_BYTES[verb];
            u32 vva = 0x400000u + ISSUER_RVA[verb];
            memcpy(live, w, ISSUE_PROLOGUE);
            if (orig_len) plant(live, vva, stub);
            expect(what, issuer_is_shipped(live, vva, w, ISSUE_PROLOGUE, orig_len ? stub : 0, rec + 8, orig_len, code_len, nfix), 1);
            printf("verb %2u rva %06x orig_len %u code_len %u nfix %u\n", verb, ISSUER_RVA[verb], orig_len, code_len, nfix);
        }
    }
    if (!failed) printf("ok\n");
    return failed;
}
