/* ranges_test.c — step R's range buffer on the host (item 4248).
 *
 * Built and run by `rondata::blind`'s `the_ranges_hook_keeps_a_due_set_s_ranges`.
 * Drives `ranges.h` the way `tracer.c` does: `check_all`'s entry opens a set,
 * each fold offers a range, each logged line moves the line, and the
 * checksum hook's fold (`checksums.h`) says at `total` whether the set is
 * due. Exits non-zero, naming the case, when a range lands on the wrong
 * line, a line outside the mask or another thread's fold is kept, a fold
 * after `total` is kept, the bytes are not the ones offered at the fold, or
 * a set that does not fit is not flagged. */
#include <stdio.h>

typedef unsigned char u8;
typedef unsigned short u16;
typedef unsigned int u32;
typedef int i32;
#include "checksums.h"
#include "ranges.h"

static int failed;

#define CHECK(cond, ...)                 \
    do {                                 \
        if (!(cond)) {                   \
            printf("FAIL " __VA_ARGS__); \
            printf("\n");                \
            failed = 1;                  \
        }                                \
    } while (0)

static RngEntry entries[64];
static u8 bytes[256];

/* One `check_all` call on `tid`: each of the fifteen lines folds `per`
 * ranges of `len` bytes (the byte is the line's index), from thread `tid`,
 * then its value is logged; `total` last. A fold from `other` is offered
 * before each line's. Returns the checksum hook's answer at `total`. */
static int one_call(Rng *r, Csum *c, u32 tid, u32 other, i32 frame, u32 base, u32 per, u32 len) {
    u8 buf[16];
    rng_begin(r, tid);
    int due = 0;
    for (u32 i = 0; i < CSUM_N; i++) {
        for (u32 k = 0; k < len; k++) buf[k] = (u8)i;
        if (i != CSUM_N - 1) {
            if (rng_wants(r, other)) rng_take(r, RNG_VIA_FOLD, 0xdead, 0, buf, len);
            for (u32 j = 0; j < per; j++)
                if (rng_wants(r, tid)) rng_take(r, RNG_VIA_FOLD, 0x700000u + i, 0x1000u * i + j, buf, len);
            for (u32 k = 0; k < len; k++) buf[k] = 0xee; /* the stack local is gone */
        }
        u32 line = i == CSUM_N - 1 ? 99 : 0x1a + 5 * i;
        rng_logged(r, csum_index(line));
        due = csum_line(c, 0x536658u + 0x58u * i, line, base + i, 0x93fb68u, frame);
    }
    if (rng_wants(r, tid)) rng_take(r, RNG_VIA_FOLD, 0xbeef, 0, buf, len); /* past total */
    return due;
}

int main(void) {
    Rng r;
    Csum c;
    RngSet s;
    csum_init(&c);
    rng_init(&r, entries, 64, bytes, 256);

    /* frame 5: two ranges a line, three bytes each; lines 0..7 kept */
    CHECK(one_call(&r, &c, 7, 9, 5, 100, 2, 3), "frame 5's first set is due");
    rng_header(&r, &s, &c);
    CHECK(s.magic == RNG_SET_MAGIC && s.frame == 5 && s.ordinal == 0 && s.flags == 0, "frame 5's header");
    CHECK(s.nranges == 16 && s.nbytes == 48, "eight lines of two ranges: %u ranges, %u bytes", s.nranges, s.nbytes);
    for (u32 k = 0; k < s.nranges && k < 64; k++) {
        CHECK(entries[k].line == k / 2, "range %u is on line %u, not %u", k, k / 2, entries[k].line);
        CHECK(entries[k].site == 0x700000u + k / 2, "range %u's site", k);
        CHECK(entries[k].start == 0x1000u * (k / 2) + k % 2, "range %u's start", k);
        CHECK(entries[k].len == 3, "range %u's length", k);
    }
    for (u32 k = 0; k < s.nbytes; k++)
        CHECK(bytes[k] == k / 6, "byte %u is the fold's (%u), not %u", k, k / 6, bytes[k]);
    for (u32 k = 0; k < 16; k++) CHECK(s.v[k] == 100 + k, "frame 5's word %u", k);

    /* the same state again on frame 5: not due, and the next set starts empty */
    CHECK(!one_call(&r, &c, 7, 9, 5, 100, 1, 1), "an unmoved set on frame 5 is not due");
    CHECK(r.n == 8 && r.nb == 8, "a new call starts its own set: %u ranges", r.n);

    /* a set that does not fit is flagged and carries no range */
    CHECK(one_call(&r, &c, 7, 9, 6, 200, 9, 4), "frame 6's set is due");
    rng_header(&r, &s, &c);
    CHECK(s.flags == RNG_OVER && s.nranges == 0 && s.nbytes == 0, "an overfull set is flagged, empty");
    CHECK(r.sets == 2 && r.over_sets == 1, "two sets written, one over: %u, %u", r.sets, r.over_sets);

    /* the bytes alone overflowing is flagged too */
    CHECK(one_call(&r, &c, 7, 9, 7, 300, 3, 16), "frame 7's set is due");
    rng_header(&r, &s, &c);
    CHECK(s.flags == RNG_OVER, "384 bytes in a 256-byte buffer is over");

    /* a fold with no set open is not kept */
    rng_init(&r, entries, 64, bytes, 256);
    CHECK(!rng_wants(&r, 7), "no set open, nothing wanted");

    if (!failed) printf("ok\n");
    return failed;
}
