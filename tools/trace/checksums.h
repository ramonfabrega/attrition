/* checksums.h — step H (item 4214): the original's fifteen checksums, every
 * frame. Included by `tracer.c` under `-DRON_CHECKSUMS`, and by
 * `checksums_test.c` on the host (`rondata::blind`'s
 * `the_checksum_hook_folds_a_frame_s_calls`). Needs `u32` and `i32`.
 *
 * `CheckSums::check_all@00936560` runs on every turn
 * (`CommandManager::process_turn@0093ef10`: once before each read
 * package's `process_all` in playback, and once after the mode arms), and
 * hands each of its fifteen lines and their wrapping `total` to
 * `SyncLogger::logToMemory<unsigned_long_&>@00463930` as
 * `(file, line, cChecksumSync, format, &value)`. The value is computed
 * before the call, so the hook reads it at that function's entry. The
 * logger itself discards it in a game with no desync; this is the only
 * place a frame's values survive the call.
 *
 * Which line a call carries is its `line` argument — the source line in
 * `checksums.cpp`, 0x1a for units and then every five to 0x60 for
 * `script_run_time`, and 99 for `total` (the listing: `push 0x1a` before
 * the call at 0x936653). The logger has 41 callers in the executable; a
 * call counts only when its return address is inside `check_all`.
 *
 * A turn's sets are folded: `process_turn` runs on every pass of
 * `Game::loop`, and most passes call `check_all` on a state no frame has
 * moved. A complete set is due when its frame differs from the last due
 * set's or any of its sixteen words does. `ordinal` is the set's place
 * among the frame's `check_all` calls (0 = the first after that frame's
 * `Game::do_frame`), and `caller` is `check_all`'s return address into
 * `process_turn`, so a reader can name which of the two calls made it. */

#define CSUM_N 16           /* fifteen lines and `total` */
#define CSUM_RVA_LO 0x536560u /* CheckSums::check_all@00936560 */
#define CSUM_RVA_HI 0x536ff0u /* CheckSum::walk_function@00936ff0, the next function */

typedef struct {
    u32 v[CSUM_N];    /* the set being read */
    u32 have;         /* which of its lines arrived, a bit each */
    u32 caller;       /* check_all's return address, read at its first line */
    u32 last[CSUM_N]; /* the last due set */
    i32 last_frame;   /* and its frame; -2 before any */
    i32 cur_frame;    /* the frame `count` counts on */
    u32 count;        /* check_all calls completed on `cur_frame` */
    u32 ordinal;      /* the due set's place among them */
    u32 torn;         /* sets whose sixteen lines did not all arrive */
} Csum;

static void csum_init(Csum *c) {
    for (u32 i = 0; i < CSUM_N; i++) c->v[i] = c->last[i] = 0;
    c->have = c->caller = c->count = c->ordinal = c->torn = 0;
    c->last_frame = -2;
    c->cur_frame = -2;
}

/* The line index of `check_all`'s `line` argument, or -1. */
static i32 csum_index(u32 line) {
    if (line == 99) return CSUM_N - 1;
    if (line >= 0x1a && line <= 0x60 && (line - 0x1a) % 5 == 0) return (i32)((line - 0x1a) / 5);
    return -1;
}

/* One logger call: `ret_rva` its return address less the image base. Returns
 * 1 when it completes a set that is due; the set is then `c->last`, its
 * frame `c->last_frame`, its place `c->ordinal`, its caller `c->caller`. */
static int csum_line(Csum *c, u32 ret_rva, u32 line, u32 value, u32 caller, i32 frame) {
    if (ret_rva < CSUM_RVA_LO || ret_rva >= CSUM_RVA_HI) return 0;
    i32 i = csum_index(line);
    if (i < 0) return 0;
    if (i == 0) {
        c->have = 0;
        c->caller = caller;
    }
    c->v[i] = value;
    c->have |= 1u << i;
    if (i != CSUM_N - 1) return 0;
    u32 whole = c->have == (1u << CSUM_N) - 1;
    c->have = 0;
    if (frame != c->cur_frame) {
        c->cur_frame = frame;
        c->count = 0;
    }
    u32 place = c->count++;
    if (!whole) {
        c->torn++;
        return 0;
    }
    int due = frame != c->last_frame;
    for (u32 k = 0; k < CSUM_N && !due; k++) due = c->v[k] != c->last[k];
    if (!due) return 0;
    for (u32 k = 0; k < CSUM_N; k++) c->last[k] = c->v[k];
    c->last_frame = frame;
    c->ordinal = place;
    return 1;
}
