/* The log stride (Loop 3085, item 3117): which frames of the engine's own
 * log window a `rontrace.cfg` `stride=N` keeps. Included by `tracer.c`, and
 * by `stride_test.c` on the host (`rondata::blind`'s
 * `the_log_stride_opens_one_block_in_n`). Needs `i32`.
 *
 * `Game::do_frame@00591ef0` calls `GameLog::begin_frame` with the frame it
 * entered on (`f`), steps `frame`, and calls `GameLog::end_frame` with
 * `f + 1`; both dump when `log_start_frame <= frame < log_end_frame`. So the
 * tracer, at the entry of `do_frame` `f`, writes `[f, f + 2)` for a kept
 * frame — both of that call's dumps, the end block being `f + 1` — and a
 * window no frame reaches otherwise (never a negative start: the engine
 * reads one as "every frame"). `lo`/`hi` are the engine's own window as
 * `rise2.ini` set it, read once; a frame is kept when `lo <= f < hi` and
 * `(f - lo) % n == 0`. */

#define STRIDE_CLOSED 0x7fffffff

static void stride_window(i32 frame, i32 lo, i32 hi, i32 n, i32 *start, i32 *end) {
    if (n >= 1 && frame >= lo && frame < hi && (frame - lo) % n == 0) {
        *start = frame;
        *end = frame + 2;
    } else {
        *start = *end = STRIDE_CLOSED;
    }
}
