/* The dump clock (item 3483): where a dumped frame's seconds go. Included by
 * `tracer.c` after `emit`, `rd_fs` and `g_base`; `rontrace.cfg`'s
 * `dumptime=1` (every frame timed) or `dumptime=2` (even frames timed, odd
 * frames run the same stubs with the clock off — the probe's own cost, read
 * off the frame clock beside them). Absent or 0, nothing here is installed
 * and the capture runs every byte as before.
 *
 * **The write path it times** (the decompile export, `INDEX.tsv`):
 *
 * - `Game::do_frame@00591ef0` calls `GameLog::begin_frame@00932a70` and
 *   `GameLog::end_frame@009329d0`; each, inside the engine's log window, sets
 *   the mode and calls `GameLog::full_dump@00930380`, whose `details[mode][k]`
 *   tests pick the walkers — `GameLog::dump_units@00930020` (`UNITS`),
 *   `GameLog::dump_leaders@009302d0` (`LEADERS`) and their siblings. Every
 *   walker opens with `NetDaemon::process_all@00951300` and ends with
 *   `fflush(NULL)`; `begin_frame` flushes the gamelog's stream twice first.
 * - A walker reaches each object's `log_data`, which hands every field to the
 *   log's vslot +0x1c (`GameLog::vftable` 0xb39158: `GameLog::say@0043cee0`,
 *   +0x14 and +0x18 its `ulong` and `float` twins), a thunk to `Log::say@00a3ab00`
 *   (`@00a3ae30`, `@00a3b160`): `check_accept`, the indent as `fputwc(L' ')`
 *   one space at a time, then one `fwprintf` of the field's name and value
 *   through `formats[0|1][kind]` — `__stdio_common_vfwprintf`, the CRT's.
 * - The CRT is `api-ms-win-crt-stdio-l1-1-0.dll`'s, so `fflush`, `fputwc`,
 *   `fclose`, `fputws` and `__stdio_common_vfwprintf` are imports: the
 *   executable calls each through its IAT slot (VA 0xac55c8, 0xac55d4,
 *   0xac55cc, 0xac5610, 0xac5634). Under Wine the forwarder lands in
 *   `ucrtbase.dll`, whose own buffered write ends in `kernel32!WriteFile`
 *   through *its* IAT — the syscall, separated from the formatting.
 *
 * **How.** A code site's first instructions are displaced into a stub, as
 * the draw hooks are; an import's IAT slot is swapped for a stub that jumps
 * through the saved pointer. Every stub calls `dt_enter` with the address
 * of the caller's return slot, which, on the main thread with the clock on,
 * pushes `(site, return address, rdtsc)` on a shadow stack and points the
 * return slot at the site's exit thunk; the function runs untouched, and its
 * own `ret` (any `ret N`) lands in the thunk, which stamps, folds the span
 * into the site's inclusive and exclusive sums (a child's span is taken off
 * its parent's exclusive) and returns where the caller called from. No
 * argument is read except `WriteFile`'s byte count, and no convention is
 * assumed: the thunk is reached with the caller's own stack pointer.
 *
 * `rdtsc` is the stamp — no syscall on the game's thread, and it is read
 * twice a call — and `QueryPerformanceCounter` is read once a frame, at
 * `do_frame`'s entry, beside it, so a reader converts ticks to seconds from
 * the run's own ratio. The first frame also times `DT_CAL` loops of a
 * one-line function called directly and through a stub, so the record says
 * what one timed call costs.
 *
 * Records (K_INFO), each once a frame at the next `do_frame`'s entry, about
 * the frame `f` that just ended (block `f + 1`'s end dump is inside it):
 *
 *   I_DUMPCLOCK a = f, b = QPC ticks over f, c..d = TSC ticks lo/hi,
 *               e = clock on | shadow entries discarded << 8 | depth << 24
 *   I_DUMPTIME  a = site | f << 8, b = calls, c..d = inclusive TSC lo/hi,
 *               e = bytes lo (WriteFile's third argument; 0 elsewhere)
 *   I_DUMPEXCL  a = site | f << 8, b..c = exclusive TSC lo/hi, d = bytes hi
 *
 * once at attach per site:
 *
 *   I_DUMPSITE  a = site, b = the VA hooked (code entry or IAT slot),
 *               c = 0 installed, 1 the bytes are not the shipped prologue,
 *               2 no IAT slot found, 3 `RON_STATE_FRAME` holds the entry,
 *               4 the slot points inside the executable (not bound),
 *               5 left out by `dumpsites=`;
 *               d = kind (0 code, 1 the exe's IAT, 2 ucrtbase's IAT),
 *               e = the stub
 *
 * and once, on the first frame:
 *
 *   I_DUMPCAL   a = DT_CAL, b = TSC of the direct loop, c = TSC of the
 *               stubbed loop, d = QPC frequency, e = TSC over QPC's 1/64 s
 *               (a coarse ratio; the frame clocks give the run's own). */

typedef unsigned long long u64;
IMPORT(i32, QueryPerformanceCounter, (u64 *));
IMPORT(i32, QueryPerformanceFrequency, (u64 *));

#define I_DUMPCLOCK 25
#define I_DUMPTIME 26
#define I_DUMPEXCL 27
#define I_DUMPSITE 28
#define I_DUMPCAL 29

#define DT_STUB 80 /* bytes a site: the entry stub (38 at most), then the exit thunk (25) at +48 */
#define DT_THUNK 48
#define DT_DEPTH 64
#define DT_CAL 100000

typedef struct {
    u32 va; /* a code site's entry, or the exe IAT slot's VA; 0 for ucrtbase's */
    u8 kind; /* 0 code, 1 the exe's IAT, 2 ucrtbase's IAT (by import name) */
    u8 len; /* displaced prologue bytes (code) */
    u8 expect[10];
} DtSite;

enum {
    DT_BEGIN_FRAME, DT_END_FRAME, DT_FULL_DUMP, DT_DUMP_UNITS, DT_DUMP_LEADERS,
    DT_SAY_INT, DT_SAY_ULONG, DT_SAY_FLOAT, DT_PROCESS_ALL,
    DT_VFWPRINTF, DT_FPUTWC, DT_FFLUSH, DT_FCLOSE, DT_FPUTWS, DT_WRITEFILE,
    DT_CALSITE, DT_NSITES
};

static const DtSite DT_SITES[DT_CALSITE] = {
    /* GameLog::begin_frame@00932a70: mov eax, [0xeb136c] */
    {0x932a70, 0, 5, {0xa1, 0x6c, 0x13, 0xeb, 0x00}},
    /* GameLog::end_frame@009329d0: mov ecx, [0xeb13d4] */
    {0x9329d0, 0, 6, {0x8b, 0x0d, 0xd4, 0x13, 0xeb, 0x00}},
    /* GameLog::full_dump@00930380: push ebp; mov ebp,esp; mov eax, fs:[0] */
    {0x930380, 0, 9, {0x55, 0x8b, 0xec, 0x64, 0xa1, 0x00, 0x00, 0x00, 0x00}},
    /* GameLog::dump_units@00930020: push ebp; mov ebp,esp; sub esp,0xc */
    {0x930020, 0, 6, {0x55, 0x8b, 0xec, 0x83, 0xec, 0x0c}},
    /* GameLog::dump_leaders@009302d0: push ecx; push esi; push edi; mov edi,ecx */
    {0x9302d0, 0, 5, {0x51, 0x56, 0x57, 0x8b, 0xf9}},
    /* Log::say@00a3ab00 (the int fields): push ebp; mov ebp,esp; push -1; push 0xab245c */
    {0xa3ab00, 0, 10, {0x55, 0x8b, 0xec, 0x6a, 0xff, 0x68, 0x5c, 0x24, 0xab, 0x00}},
    /* Log::say@00a3ae30 (ulong): push -1; push 0xab2497 */
    {0xa3ae30, 0, 10, {0x55, 0x8b, 0xec, 0x6a, 0xff, 0x68, 0x97, 0x24, 0xab, 0x00}},
    /* Log::say@00a3b160 (float): push -1; push 0xab24d2 */
    {0xa3b160, 0, 10, {0x55, 0x8b, 0xec, 0x6a, 0xff, 0x68, 0xd2, 0x24, 0xab, 0x00}},
    /* NetDaemon::process_all@00951300: push -1; push 0xaa6157 */
    {0x951300, 0, 10, {0x55, 0x8b, 0xec, 0x6a, 0xff, 0x68, 0x57, 0x61, 0xaa, 0x00}},
    {0xac5634, 1, 0, {0}}, /* __stdio_common_vfwprintf */
    {0xac55d4, 1, 0, {0}}, /* fputwc */
    {0xac55c8, 1, 0, {0}}, /* fflush */
    {0xac55cc, 1, 0, {0}}, /* fclose */
    {0xac5610, 1, 0, {0}}, /* fputws */
    {0, 2, 0, {0}},        /* ucrtbase.dll's WriteFile */
};

typedef struct {
    u32 site, ret;
    u64 t0, child;
} DtFrame;

static i32 g_dt_mode; /* rontrace.cfg `dumptime=` */
static u32 g_dt_mask = ~0u; /* rontrace.cfg `dumpsites=` (hex): the sites installed, bit = site */
static i32 g_dt_on; /* the clock runs this frame */
static u32 g_dt_tid; /* do_frame's thread, read at the first frame */
static u8 *g_dt_page;
static u32 g_dt_orig[DT_NSITES]; /* an import's target, or a code site's trampoline */
static DtFrame g_dt_stack[DT_DEPTH];
static u32 g_dt_depth, g_dt_leaks;
static u32 g_dt_calls[DT_NSITES];
static u64 g_dt_incl[DT_NSITES], g_dt_excl[DT_NSITES], g_dt_bytes[DT_NSITES];
static i32 g_dt_frame = -1;
static u64 g_dt_q, g_dt_t;

static u64 dt_tsc(void) { return __builtin_ia32_rdtsc(); }

static u64 dt_qpc(void) {
    u64 q = 0;
    QueryPerformanceCounter(&q);
    return q;
}

static void __cdecl dt_enter(u32 site, u32 *slot) {
    if (!g_dt_on || g_dt_depth >= DT_DEPTH || rd_fs(0x24) != g_dt_tid) return;
    DtFrame *f = &g_dt_stack[g_dt_depth++];
    f->site = site;
    f->ret = *slot;
    f->child = 0;
    *slot = (u32)(g_dt_page + DT_STUB * site + DT_THUNK);
    g_dt_calls[site]++;
    if (site == DT_WRITEFILE) g_dt_bytes[site] += slot[3];
    f->t0 = dt_tsc();
}

/* Reached only through a thunk whose return slot `dt_enter` rewrote, so the
 * site's entry is on the stack; anything above it was left by an unwind
 * that skipped its thunk, and is dropped and counted. */
static u32 __cdecl dt_exit(u32 site) {
    u64 t1 = dt_tsc();
    while (g_dt_depth > 1 && g_dt_stack[g_dt_depth - 1].site != site) {
        g_dt_depth--;
        g_dt_leaks++;
    }
    DtFrame *f = &g_dt_stack[--g_dt_depth];
    u64 e = t1 - f->t0;
    g_dt_incl[site] += e;
    g_dt_excl[site] += e - f->child;
    if (g_dt_depth) g_dt_stack[g_dt_depth - 1].child += e;
    return f->ret;
}

/* The entry stub, then the exit thunk at +DT_THUNK:
 *
 *   50 51 52           push eax; push ecx; push edx
 *   8D 44 24 0C        lea eax, [esp+0xc]        ; the caller's return slot
 *   50                 push eax
 *   6A ss              push site
 *   B8 xx xx xx xx     mov eax, dt_enter
 *   FF D0              call eax
 *   83 C4 08           add esp, 8
 *   5A 59 58           pop edx; pop ecx; pop eax
 *   <len bytes>        the displaced prologue     (code)
 *   E9 rr rr rr rr     jmp entry + len
 *   | FF 25 pp pp pp pp  jmp [g_dt_orig + 4*site]  (an import)
 *
 *   6A 00              push 0                     ; the return address, below
 *   50 52 51           push eax; push edx; push ecx
 *   6A ss              push site
 *   B8 xx xx xx xx     mov eax, dt_exit
 *   FF D0              call eax
 *   83 C4 04           add esp, 4
 *   89 44 24 0C        mov [esp+0xc], eax
 *   59 5A 58           pop ecx; pop edx; pop eax
 *   C3                 ret
 *
 * Every register the function was handed and every register it returns
 * (eax, edx) passes through; the flags do not, and no caller of these
 * reads them across a call. */
static void dt_build(u32 site, const u8 *code, u32 len, u32 back) {
    u8 *s = g_dt_page + DT_STUB * site;
    u32 n = 0;
    static const u8 head[] = {0x50, 0x51, 0x52, 0x8d, 0x44, 0x24, 0x0c, 0x50, 0x6a};
    memcpy(s, head, sizeof head);
    n = sizeof head;
    s[n++] = (u8)site;
    s[n++] = 0xb8;
    *(u32 *)(s + n) = (u32)(void *)dt_enter;
    n += 4;
    static const u8 mid[] = {0xff, 0xd0, 0x83, 0xc4, 0x08, 0x5a, 0x59, 0x58};
    memcpy(s + n, mid, sizeof mid);
    n += sizeof mid;
    if (code) {
        memcpy(s + n, code, len);
        n += len;
        s[n++] = 0xe9;
        *(u32 *)(s + n) = back - ((u32)(s + n) + 4);
    } else {
        s[n++] = 0xff;
        s[n++] = 0x25;
        *(u32 *)(s + n) = (u32)&g_dt_orig[site];
    }
    u8 *t = s + DT_THUNK;
    n = 0;
    static const u8 thead[] = {0x6a, 0x00, 0x50, 0x52, 0x51, 0x6a};
    memcpy(t, thead, sizeof thead);
    n = sizeof thead;
    t[n++] = (u8)site;
    t[n++] = 0xb8;
    *(u32 *)(t + n) = (u32)(void *)dt_exit;
    n += 4;
    static const u8 ttail[] = {0xff, 0xd0, 0x83, 0xc4, 0x04, 0x89, 0x44, 0x24, 0x0c, 0x59, 0x5a, 0x58, 0xc3};
    memcpy(t + n, ttail, sizeof ttail);
}

static int dt_lower_eq(const char *a, const char *b) {
    for (; *a && *b; a++, b++) {
        char x = *a, y = *b;
        if (x >= 'A' && x <= 'Z') x += 32;
        if (y >= 'A' && y <= 'Z') y += 32;
        if (x != y) return 0;
    }
    return *a == *b;
}

/* The slot in `mod`'s IAT that imports `name` from any DLL, or 0. */
static u32 *dt_iat_slot(u8 *mod, const char *name) {
    if (!mod) return 0;
    u8 *nt = mod + *(u32 *)(mod + 0x3c);
    u32 dir = *(u32 *)(nt + 0x80), size = *(u32 *)(nt + 0x84);
    if (!dir || !size) return 0;
    for (u32 *d = (u32 *)(mod + dir); d[3]; d += 5) {
        u32 *ilt = (u32 *)(mod + (d[0] ? d[0] : d[4])), *iat = (u32 *)(mod + d[4]);
        for (u32 i = 0; ilt[i]; i++) {
            if (ilt[i] & 0x80000000u) continue;
            if (dt_lower_eq((const char *)(mod + ilt[i] + 2), name)) return &iat[i];
        }
    }
    return 0;
}

static int dt_in_exe(u32 p) {
    u8 *nt = (u8 *)g_base + *(u32 *)(g_base + 0x3c);
    return p >= g_base && p < g_base + *(u32 *)(nt + 0x50);
}

/* Is `rva` inside a code site's displaced bytes (`arm_all` leaves it). */
static int dt_is_site(u32 rva) {
    if (!g_dt_mode) return 0;
    for (u32 i = 0; i < DT_CALSITE; i++)
        if (DT_SITES[i].kind == 0 && (g_dt_mask >> i & 1) && rva + 0x400000u >= DT_SITES[i].va && rva + 0x400000u < DT_SITES[i].va + DT_SITES[i].len)
            return 1;
    return 0;
}

static void dt_swap(u32 *slot, u32 stub) {
    u32 old;
    VirtualProtect(slot, 4, PAGE_READWRITE, &old);
    *slot = stub;
    VirtualProtect(slot, 4, old, &old);
}

static void dt_install(void) {
    if (!g_dt_mode) return;
    g_dt_page = (u8 *)VirtualAlloc(0, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    if (!g_dt_page) return;
    /* The executable is at its preferred base (`g_base`); a VA here is
     * rebased by the difference, as every RVA in this file is. */
    u32 rebase = g_base - 0x400000u;
    for (u32 i = 0; i < DT_CALSITE; i++) {
        const DtSite *s = &DT_SITES[i];
        u32 why = 0, va = s->va ? s->va + rebase : 0;
        u8 *stub = g_dt_page + DT_STUB * i;
        if (!(g_dt_mask >> i & 1)) {
            why = 5;
        } else if (s->kind == 0) {
            u8 *t = (u8 *)va;
#ifdef RON_STATE_FRAME
            if (i == DT_END_FRAME) why = 3;
#endif
            for (u32 j = 0; !why && j < s->len; j++)
                if (t[j] != s->expect[j]) why = 1;
            if (!why) {
                dt_build(i, t, s->len, va + s->len);
                t[0] = 0xe9;
                *(u32 *)(t + 1) = (u32)stub - ((u32)t + 5);
                for (u32 j = 5; j < s->len; j++) t[j] = 0xcc; /* never executed */
            }
        } else {
            u32 *slot = s->kind == 1 ? (u32 *)va : dt_iat_slot((u8 *)GetModuleHandleA("ucrtbase.dll"), "WriteFile");
            if (!slot) why = 2;
            else if (dt_in_exe(*slot)) why = 4;
            if (!why) {
                g_dt_orig[i] = *slot;
                dt_build(i, 0, 0, 0);
                dt_swap(slot, (u32)stub);
                va = (u32)slot;
            }
        }
        emit(K_INFO, I_DUMPSITE, i, va, why, s->kind, (u32)stub);
    }
    FlushInstructionCache(g_proc, (void *)(g_base + TEXT_RVA), TEXT_SIZE);
}

__attribute__((noinline)) static i32 __cdecl dt_one(i32 x) {
    __asm__ volatile("" ::: "memory");
    return x + 1;
}

/* The probe's own price: DT_CAL calls of `dt_one` direct, then through a
 * stub of its own with the clock on (its sums are dropped after). */
static void dt_calibrate(void) {
    typedef i32(__cdecl * one_fn)(i32);
    g_dt_orig[DT_CALSITE] = (u32)(void *)dt_one;
    dt_build(DT_CALSITE, 0, 0, 0);
    one_fn direct = dt_one, stubbed = (one_fn)(void *)(g_dt_page + DT_STUB * DT_CALSITE);
    __asm__ volatile("" : "+r"(direct), "+r"(stubbed));
    volatile i32 acc = 0;
    i32 was = g_dt_on;
    g_dt_on = 1;
    u64 a = dt_tsc();
    for (i32 i = 0; i < DT_CAL; i++) acc += direct(i);
    u64 b = dt_tsc();
    for (i32 i = 0; i < DT_CAL; i++) acc += stubbed(i);
    u64 c = dt_tsc();
    g_dt_on = was;
    g_dt_calls[DT_CALSITE] = 0;
    g_dt_incl[DT_CALSITE] = g_dt_excl[DT_CALSITE] = 0;
    u64 freq = 0;
    QueryPerformanceFrequency(&freq);
    u64 q0 = dt_qpc(), t0 = dt_tsc(), q1;
    do q1 = dt_qpc();
    while (q1 - q0 < freq / 64);
    u64 t1 = dt_tsc();
    emit(K_INFO, I_DUMPCAL, DT_CAL, (u32)(b - a), (u32)(c - b), (u32)freq, (u32)(t1 - t0));
}

/* At `do_frame`'s entry: the frame that ended, then the clock for this one. */
static void dt_frame(i32 frame) {
    if (!g_dt_mode || !g_dt_page) return;
    u64 q = dt_qpc(), t = dt_tsc();
    if (!g_dt_tid) {
        g_dt_tid = rd_fs(0x24);
        dt_calibrate();
    } else {
        u64 dt = t - g_dt_t;
        emit(K_INFO, I_DUMPCLOCK, (u32)g_dt_frame, (u32)(q - g_dt_q), (u32)dt, (u32)(dt >> 32),
             (u32)g_dt_on | (g_dt_leaks & 0xffff) << 8 | g_dt_depth << 24);
        for (u32 i = 0; i < DT_CALSITE; i++) {
            if (!g_dt_calls[i]) continue;
            u32 a = i | (u32)g_dt_frame << 8;
            emit(K_INFO, I_DUMPTIME, a, g_dt_calls[i], (u32)g_dt_incl[i], (u32)(g_dt_incl[i] >> 32),
                 (u32)g_dt_bytes[i]);
            emit(K_INFO, I_DUMPEXCL, a, (u32)g_dt_excl[i], (u32)(g_dt_excl[i] >> 32),
                 (u32)(g_dt_bytes[i] >> 32), 0);
            g_dt_calls[i] = 0;
            g_dt_incl[i] = g_dt_excl[i] = g_dt_bytes[i] = 0;
        }
        g_dt_leaks = 0;
    }
    g_dt_frame = frame;
    g_dt_on = g_dt_mode == 1 || (g_dt_mode == 2 && !(frame & 1));
    g_dt_q = q; /* the records' own cost lands in this frame: it is the probe's */
    g_dt_t = t;
}
