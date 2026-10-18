/* stride_test.c — the log stride's window on the host (Loop 3085).
 *
 * Built and run by `rondata::blind`'s `the_log_stride_opens_one_block_in_n`.
 * Exits non-zero, naming the case, when `stride_window` opens a frame it
 * should leave closed or closes one it should open. The engine's own test
 * (`GameLog::begin_frame@00932a70`, `end_frame@009329d0`) is copied in
 * `logged` so the cases read as the dump the capture would print. */
#include <stdio.h>

typedef int i32;
#include "stride.h"

static int failed;

/* `log_start_frame < 0 || (log_start_frame <= frame && frame < log_end_frame)`. */
static int logged(i32 start, i32 end, i32 frame) {
    return start < 0 || (start <= frame && frame < end);
}

/* One `Game::do_frame` entered at `f`: `begin_frame` reads `f`, the frame
 * counter steps, `end_frame` reads `f + 1` — the block the harness keys. */
static int end_block(i32 f, i32 lo, i32 hi, i32 n) {
    i32 s, e;
    stride_window(f, lo, hi, n, &s, &e);
    return logged(s, e, f + 1);
}

static int begin_block(i32 f, i32 lo, i32 hi, i32 n) {
    i32 s, e;
    stride_window(f, lo, hi, n, &s, &e);
    return logged(s, e, f);
}

int main(void) {
    const i32 lo = 37, hi = 1114, n = 16;
    int blocks = 0;
    for (i32 f = 0; f < 2000; f++) {
        int want = f >= lo && f < hi && (f - lo) % n == 0;
        if (end_block(f, lo, hi, n) != want) {
            printf("FAIL end block %d: %s\n", f + 1, want ? "closed" : "opened");
            failed = 1;
        }
        if (begin_block(f, lo, hi, n) != want) {
            printf("FAIL begin block %d: %s\n", f, want ? "closed" : "opened");
            failed = 1;
        }
        blocks += want;
    }
    if (blocks != (hi - lo + n - 1) / n) {
        printf("FAIL %d blocks\n", blocks);
        failed = 1;
    }
    /* A stride of 1 is the engine's own window, frame for frame. */
    for (i32 f = 0; f < 200; f++) {
        if (end_block(f, 50, 100, 1) != (f >= 50 && f < 100)) {
            printf("FAIL stride 1 at %d\n", f);
            failed = 1;
        }
    }
    if (!failed) printf("ok\n");
    return failed;
}
