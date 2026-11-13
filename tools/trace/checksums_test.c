/* checksums_test.c — the checksum hook's fold on the host (item 4214).
 *
 * Built and run by `rondata::blind`'s `the_checksum_hook_folds_a_frame_s_calls`.
 * Exits non-zero, naming the case, when `csum_line` keeps a call it should
 * drop, folds a set it should keep, or keeps one it should fold. The calls
 * are `check_all`'s sixteen as the listing makes them: line 0x1a for units,
 * every five to 0x60, then 99 for `total`. */
#include <stdio.h>

typedef unsigned int u32;
typedef int i32;
#include "checksums.h"

static int failed;

#define CHECK(cond, ...)               \
    do {                               \
        if (!(cond)) {                 \
            printf("FAIL " __VA_ARGS__); \
            printf("\n");              \
            failed = 1;                \
        }                              \
    } while (0)

/* `check_all`'s sixteen calls on one state; returns how many came back due
 * (0 or 1), the last call's answer. */
static int one_call(Csum *c, u32 base, i32 frame, u32 caller) {
    int due = 0;
    for (u32 i = 0; i < CSUM_N; i++) {
        u32 line = i == CSUM_N - 1 ? 99 : 0x1a + 5 * i;
        u32 ret = 0x536658u + 0x58u * i; /* inside check_all */
        due = csum_line(c, ret, line, base + i, caller, frame);
        if (i != CSUM_N - 1) CHECK(!due, "frame %d line %u was due before total", frame, i);
    }
    return due;
}

int main(void) {
    Csum c;
    csum_init(&c);

    /* The setup path's call, frame -1, is a frame like any other. */
    CHECK(one_call(&c, 100, -1, 0x93fb68) == 1, "the first set is due");
    CHECK(c.last_frame == -1 && c.ordinal == 0 && c.last[0] == 100 && c.last[15] == 115,
          "the first set is kept whole: frame %d ordinal %u", c.last_frame, c.ordinal);

    /* Frame 90: the read loop's call, then the final one on the same
     * state, then a second pass of the loop: one set, at ordinal 0. */
    CHECK(one_call(&c, 200, 90, 0x93fb48) == 1, "frame 90's first set is due");
    CHECK(c.ordinal == 0 && c.caller == 0x93fb48, "ordinal %u caller %x", c.ordinal, c.caller);
    CHECK(one_call(&c, 200, 90, 0x93fb68) == 0, "the same state again is folded");
    CHECK(one_call(&c, 200, 90, 0x93fb68) == 0, "and again");
    /* A package moved the state: due, at the frame's fourth call. */
    CHECK(one_call(&c, 300, 90, 0x93fb68) == 1, "a moved state is due");
    CHECK(c.ordinal == 3 && c.last[0] == 300, "the moved set is the fourth call: ordinal %u", c.ordinal);

    /* Frame 91 on a state equal to 90's last: due, because the frame moved. */
    CHECK(one_call(&c, 300, 91, 0x93fb68) == 1, "a new frame's first set is due on an equal state");
    CHECK(c.ordinal == 0 && c.last_frame == 91, "frame 91 ordinal %u", c.ordinal);

    /* One word moved is enough: `total` alone. */
    {
        int due = 0;
        for (u32 i = 0; i < CSUM_N; i++) {
            u32 line = i == CSUM_N - 1 ? 99 : 0x1a + 5 * i;
            due = csum_line(&c, 0x536658u, line, 300 + i + (i == CSUM_N - 1), 0x93fb68, 91);
        }
        CHECK(due == 1 && c.last[15] == 316, "a moved total is due");
    }

    /* Another caller of the logger, outside check_all, never counts. */
    CHECK(csum_line(&c, 0x1afac0u, 0x1a, 7, 0, 92) == 0, "a call outside check_all is dropped");
    CHECK(csum_line(&c, CSUM_RVA_HI, 99, 7, 0, 92) == 0, "the range's end is outside it");
    CHECK(csum_line(&c, 0x536658u, 0x1b, 7, 0, 92) == 0, "a line no walk logs is dropped");
    CHECK(c.last_frame == 91, "nothing outside check_all moved the kept frame");

    /* A torn set — total without units — is counted and never due. */
    {
        Csum t;
        csum_init(&t);
        int due = 0;
        for (u32 i = 1; i < CSUM_N; i++) {
            u32 line = i == CSUM_N - 1 ? 99 : 0x1a + 5 * i;
            due = csum_line(&t, 0x536658u, line, i, 0, 5);
        }
        CHECK(due == 0 && t.torn == 1, "a torn set is not due: due %d torn %u", due, t.torn);
        CHECK(one_call(&t, 0, 5, 0) == 1 && t.ordinal == 1, "the torn call still counts its place: %u", t.ordinal);
    }

    /* The sixteen lines map to sixteen slots. */
    for (u32 i = 0; i < 15; i++) CHECK(csum_index(0x1a + 5 * i) == (i32)i, "line %u", i);
    CHECK(csum_index(99) == 15 && csum_index(0x65) == -1 && csum_index(0x15) == -1, "the ends");

    if (!failed) printf("ok\n");
    return failed;
}
