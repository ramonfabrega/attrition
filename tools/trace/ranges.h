/* ranges.h — step R (item 4248): every range the checksum's fold takes on
 * the open lines, every frame. Included by `tracer.c` under `-DRON_RANGES`
 * (which needs `-DRON_CHECKSUMS`), and by `ranges_test.c` on the host
 * (`rondata::blind`'s `the_ranges_hook_keeps_a_due_set_s_ranges`). Needs
 * `u8`, `u16`, `u32` and `i32`; no CRT.
 *
 * `CheckSum::walk_function@00936ff0` is the fold every walk hands its byte
 * ranges to (`__thiscall`, `(start, end)`, `ret 8`), and `check_groups`
 * also runs zlib's `adler32@005089d0` (cdecl `(seed, buffer, length)`) over
 * eight words it copies to a stack local, from the call that returns to
 * 0x937587. Both are hooked at their entry with the entry's esp, as the
 * checksum hook is, and `check_all@00936560`'s entry opens a set.
 *
 * Which line a range belongs to is the count of `check_all`'s value lines
 * logged since the set opened: a line's walk runs, then its value is logged
 * (`tools/recomp/checksum.py` attributes the same way, and its streams sum
 * to the original's value on every line). The ranges of a line in
 * `RNG_MASK` are copied — site, start, length and bytes, at the fold's
 * entry, so a stack local's bytes are the ones folded — into a buffer the
 * set owns. `checksums.h` decides at `total` whether the set is due; only a
 * due set is written, so a frame's many `check_all` calls on an unmoved
 * state cost a copy and no write.
 *
 * The record (`rontrace.ranges` beside the exe; `tools/trace/ranges.py` and
 * `rondata::trace::ranges` read it), little-endian:
 *   file header, 8 words: RNG_MAGIC, RNG_VERSION, image base, RNG_MASK,
 *     0, 0, 0, 0;
 *   per due set, an `RngSet` (24 words), then `nranges` `RngEntry` (four
 *     words each), then `nbytes` bytes, padded with zeros to a word.
 * A set whose ranges did not fit carries `RNG_OVER` in `flags` and none of
 * its ranges. */

#ifndef RNG_MASK
#define RNG_MASK 0xffu /* lines 0..7: units, builds, walls, ammo, deaths, groups, guys, leaders */
#endif
#define RNG_MAGIC 0x524e4752u /* "RGNR" */
#define RNG_VERSION 1u
#define RNG_SET_MAGIC 0x54455352u /* "RSET" */
#define RNG_OVER 1u
#define RNG_VIA_FOLD 0u /* CheckSum::walk_function */
#define RNG_VIA_ZLIB 1u /* adler32@005089d0, from check_groups */
#define RNG_ZLIB_RET 0x537587u /* the RVA check_groups' zlib call returns to */

typedef struct {
    u8 line;  /* check_all's line index, 0 = units */
    u8 via;   /* RNG_VIA_* */
    u16 pad;
    u32 site; /* the fold's return address (VA) */
    u32 start;
    u32 len;
} RngEntry;

typedef struct {
    u32 magic;   /* RNG_SET_MAGIC */
    i32 frame;   /* the tracer's frame, as a K_CSUM set's */
    u32 ordinal; /* the set's place among the frame's check_all calls */
    u32 caller;  /* check_all's return into process_turn (VA) */
    u32 count;   /* the frame's check_all calls so far */
    u32 flags;   /* RNG_OVER */
    u32 nranges;
    u32 nbytes;
    u32 v[16];   /* the set's fifteen lines and total */
} RngSet;

typedef struct {
    RngEntry *e;
    u32 cap_e, n;
    u8 *b;
    u32 cap_b, nb;
    u32 line; /* the line the next range belongs to */
    u32 open; /* between check_all's entry and its total */
    u32 tid;  /* the thread that opened the set */
    u32 over; /* this set's ranges did not fit */
    u32 sets, over_sets;
} Rng;

static void rng_init(Rng *r, RngEntry *e, u32 cap_e, u8 *b, u32 cap_b) {
    r->e = e;
    r->cap_e = cap_e;
    r->b = b;
    r->cap_b = cap_b;
    r->n = r->nb = r->line = r->open = r->tid = r->over = r->sets = r->over_sets = 0;
}

/* check_all's entry: a fresh set on thread `tid`. */
static void rng_begin(Rng *r, u32 tid) {
    r->n = r->nb = r->line = r->over = 0;
    r->open = 1;
    r->tid = tid;
}

/* Whether a fold on `tid` now belongs to a set's kept line: the hot path,
 * taken before any argument is read (the rules line alone is 250,000
 * ranges a call). */
static int rng_wants(const Rng *r, u32 tid) {
    return r->open && r->line < 32 && ((RNG_MASK >> r->line) & 1u) && r->tid == tid;
}

/* One range, its bytes at `p`. */
static void rng_take(Rng *r, u32 via, u32 site, u32 start, const u8 *p, u32 len) {
    if (r->over) return;
    if (r->n >= r->cap_e || len > r->cap_b - r->nb) {
        r->over = 1;
        return;
    }
    RngEntry *x = &r->e[r->n++];
    x->line = (u8)r->line;
    x->via = (u8)via;
    x->pad = 0;
    x->site = site;
    x->start = start;
    x->len = len;
    for (u32 i = 0; i < len; i++) r->b[r->nb + i] = p[i];
    r->nb += len;
}

/* check_all logged line `index` (`csum_index`'s; 15 is total). The next
 * range belongs to the line after it; total closes the set. */
static void rng_logged(Rng *r, i32 index) {
    if (index < 0) return;
    r->line = (u32)index + 1;
    if (index == 15) r->open = 0;
}

/* The header of a due set, from the checksum hook's fold. */
static void rng_header(Rng *r, RngSet *s, const Csum *c) {
    s->magic = RNG_SET_MAGIC;
    s->frame = c->last_frame;
    s->ordinal = c->ordinal;
    s->caller = c->caller;
    s->count = c->count;
    s->flags = r->over ? RNG_OVER : 0;
    s->nranges = r->over ? 0 : r->n;
    s->nbytes = r->over ? 0 : r->nb;
    for (u32 k = 0; k < 16; k++) s->v[k] = c->last[k];
    r->sets++;
    if (r->over) r->over_sets++;
}
