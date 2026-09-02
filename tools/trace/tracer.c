/*
 * rontrace.dll — an in-process trace of the original.
 *
 * Loaded into riseofnations.exe by the one import that `patch_exe.py` adds
 * to a copy of it. Two instruments, one log:
 *
 *   1. The draw-site trace. Every function that steps the game's LCG is
 *      trampolined at its entry — `Random::get()`, `Random::get(int,int)`,
 *      `MathUtilFuncSet::rand_real` (the script VM's, inlined on
 *      `game_random`), `Random::reseed` — and each call is logged with the
 *      RNG it hit, the seed before the step, the caller's return address and
 *      two more frames of the ebp chain, and the simulation frame.
 *      `Game::do_frame` is trampolined too, which is where the frame comes
 *      from (`Game+0x550`) and where the per-frame seed is recorded.
 *
 *   2. Function coverage. Every function entry in the Ghidra export
 *      (`rontrace.funcs`, u32 RVAs) gets an `int 3`; a vectored exception
 *      handler logs the first hit, restores the byte and resumes. Armed once
 *      at attach (so the log holds every function the run ever entered, with
 *      the frame it was first entered on) and re-armed at the start of every
 *      frame inside the window in `rontrace.cfg`, which gives per-frame sets
 *      for those frames.
 *
 *   3. The cheat channel. `rontrace.cmd` beside the exe lists `<frame> <line>`
 *      entries; at the entry of `Game::do_frame` for that sim-frame each line
 *      is handed to `ConsoleWin::parse_cmd` — the function the chat box calls
 *      with the `cheat ` prefix stripped — so a scenario is staged from a
 *      file, inside the tick, reproducibly, with no keyboard or mouse. A line
 *      starting with `!` goes in as a console command instead (the
 *      console-only half of the table: `quit`, `ai off`, `pause`). Each
 *      executed line is an INFO record. The line does NOT travel the order
 *      stream, so a recording of the run does not carry it.
 *
 *   4. The call proxies. `rontrace.cfg`'s `callwin=LO-HI` replaces each
 *      listed function with a proxy that logs its arguments, calls the
 *      original, and logs its **answer** — the one thing neither a draw hook
 *      nor an `int 3` can give, and the one thing the gamelog has no
 *      category for. `PathFinder::calc_cost` is why it exists: a per-step
 *      cost dump over one frame's searches, delimited by
 *      `PathFinder::astar_path`'s own entry and return. Without a `callwin`
 *      nothing is patched, so every earlier capture is reproduced exactly.
 *
 * Freestanding: no CRT, kernel32 only, no floating point (the hooked
 * functions' callers may have live x87/SSE state; the stubs save only the
 * integer registers and flags). Built by `build.sh` with clang, llvm-dlltool
 * and rust-lld. Everything address-shaped is for the PDB-matched executable
 * only — the hook table checks the prologue bytes it expects before it
 * patches anything, and refuses (logging an INFO record) on a mismatch.
 *
 * Log format (`rontrace.log` beside the exe): a 32-byte header then 32-byte
 * records of eight u32s; see `report.py` for the reader.
 */

typedef unsigned char u8;
typedef unsigned short u16;
typedef unsigned int u32;
typedef int i32;
typedef void *HANDLE;

#define WINAPI __stdcall
#define IMPORT(ret, name, args) __declspec(dllimport) ret WINAPI name args

IMPORT(HANDLE, CreateFileA, (const char *, u32, u32, void *, u32, u32, HANDLE));
IMPORT(i32, WriteFile, (HANDLE, const void *, u32, u32 *, void *));
IMPORT(i32, ReadFile, (HANDLE, void *, u32, u32 *, void *));
IMPORT(i32, CloseHandle, (HANDLE));
IMPORT(u32, GetFileSize, (HANDLE, u32 *));
IMPORT(i32, VirtualProtect, (void *, u32, u32, u32 *));
IMPORT(void *, VirtualAlloc, (void *, u32, u32, u32));
IMPORT(void *, AddVectoredExceptionHandler, (u32, void *));
IMPORT(void *, GetModuleHandleA, (const char *));
IMPORT(u32, GetModuleFileNameA, (void *, char *, u32));
IMPORT(i32, FlushInstructionCache, (HANDLE, const void *, u32));
IMPORT(HANDLE, GetCurrentProcess, (void));
IMPORT(u32, GetCurrentThreadId, (void));
IMPORT(void, OutputDebugStringA, (const char *));

#define GENERIC_READ 0x80000000u
#define GENERIC_WRITE 0x40000000u
#define FILE_SHARE_READ 1u
#define CREATE_ALWAYS 2u
#define OPEN_EXISTING 3u
#define FILE_ATTRIBUTE_NORMAL 0x80u
#define INVALID_HANDLE ((HANDLE)(i32)-1)
#define PAGE_EXECUTE_READWRITE 0x40u
#define MEM_COMMIT 0x1000u
#define MEM_RESERVE 0x2000u
#define EXCEPTION_BREAKPOINT 0x80000003u

typedef struct {
    u32 code, flags;
    void *record, *address;
    u32 nparams;
    u32 info[15];
} EXCEPTION_RECORD;

/* x86 CONTEXT: Eip is at +0xb8 — ContextFlags, six debug registers, the
 * 112-byte FLOATING_SAVE_AREA, four segment registers, then the integer
 * registers in this order. */
typedef struct {
    u32 ContextFlags;
    u32 Dr[6];
    u8 fsave[112];
    u32 SegGs, SegFs, SegEs, SegDs;
    u32 Edi, Esi, Ebx, Edx, Ecx, Eax, Ebp, Eip, SegCs, EFlags, Esp, SegSs;
    u8 ext[512];
} CONTEXT;

typedef struct {
    EXCEPTION_RECORD *rec;
    CONTEXT *ctx;
} EXCEPTION_POINTERS;

/* ---- the executable ---------------------------------------------------- */

#define TEXT_RVA 0x1000u
#define TEXT_SIZE 0x6C4000u /* .text VirtualSize 0x6C3230, rounded up */
/* `game_random` (Random, seed at +0): PDB 0003:2300556, section 3 = .data at
 * RVA 0x806000, so RVA 0xA37A8C, VA 0xE37A8C — `GameAccess::game_random` is
 * the pointer to it at VA 0xC06184 that rand_real loads. */
#define RVA_GAME_RANDOM 0xA37A8Cu
#define GAME_FRAME_OFF 0x550u /* Game::frame */

/* The cheat channel's three addresses, all PDB-named:
 *   `MiscAccess::console_win` (ConsoleWin *) — PDB 0003:2595460, .data RVA
 *     0x806000 + 0x279A84 = RVA 0xA7FA84 (VA 0xE7FA84);
 *   `ConsoleWin::parse_cmd@007d6470(this, String *line, int from_chat, int no_mouse)`
 *     — the chat box calls it (line, 1, 0) after stripping `cheat `; 0 for
 *     from_chat reaches the console-only commands; no_mouse = 1 skips
 *     `TerrainOut::get_mouse_coords`;
 *   `String::String(wchar_t *)@00a1edd0` — the const-string constructor
 *     (flags = 1, no heap; the object is 0x14 bytes) and
 *   `String::~String@00a1ee20`.
 * All three are __thiscall. */
#define RVA_CONSOLE_WIN 0xA7FA84u
#define RVA_PARSE_CMD 0x3D6470u
#define RVA_STRING_CTOR 0x61EDD0u
#define RVA_STRING_DTOR 0x61EE20u

enum {
    K_HIT = 0,
    K_GETF = 1, /* Random::get()          float, this = ecx */
    K_FRAME = 2, /* Game::do_frame entry */
    K_GETI = 3, /* Random::get(int,int)   this = ecx, arg0 = lo */
    K_REAL = 4, /* MathUtilFuncSet::rand_real — game_random, inlined */
    K_INFO = 5,
    K_RESEED = 6, /* Random::reseed(seed)   this = ecx, arg0 = new seed */
    K_CALL = 7, /* a proxied call's entry:  a = site, b = this, c..f = args 0..3 */
    K_RET = 8, /* and its return:          a = site, b = eax,  c..e = args 4..6,
                * f = the byte behind arg7 where the site names one, else ~0 */
};

enum {
    I_ATTACH = 1, /* a = base, b = nfuncs, c = window lo, d = window hi */
    I_HOOK_MISMATCH = 2, /* a = rva, b..e = the first 16 bytes found */
    I_HOOKED = 3, /* a = rva, b = stub */
    I_NOFUNCS = 4,
    I_PROTECT_FAIL = 5,
    I_ARMED = 6, /* a = frame, b = count */
    I_DETACH = 7,
    I_DECLINED = 8, /* a breakpoint that was not ours: a = address, b = Eip, c = armed, d = saved, e = orig */
    I_CMD = 9, /* a cheat line ran: a = frame, b = line index, c = from_chat, d = parse_cmd's return */
    I_CMD_NOCONSOLE = 10, /* a line was due but MiscAccess::console_win is null: a = frame, b = index */
    I_CMDS = 11, /* attach: a = lines parsed from rontrace.cmd */
    I_PROXIED = 12, /* a call site is proxied: a = rva, b = stub, c = nargs */
};

typedef struct {
    u32 rva;
    u32 len; /* displaced prologue bytes (>= 5, whole instructions, no rel) */
    u32 kind;
    u8 expect[10];
} HookSite;

static const HookSite HOOKS[] = {
    /* Game::do_frame@00591ef0: push ebp; mov ebp,esp; push -1; push 0xa83e41 */
    {0x191ef0, 10, K_FRAME, {0x55, 0x8b, 0xec, 0x6a, 0xff, 0x68, 0x41, 0x3e, 0xa8, 0x00}},
    /* Random::get@00a39cf0: push ebp; mov ebp,esp; push ecx; imul eax,[ecx],0x19660d */
    {0x639cf0, 10, K_GETF, {0x55, 0x8b, 0xec, 0x51, 0x69, 0x01, 0x0d, 0x66, 0x19, 0x00}},
    /* Random::get@00a39d70: push ebp; mov ebp,esp; push -1; push 0xab22d0 */
    {0x639d70, 10, K_GETI, {0x55, 0x8b, 0xec, 0x6a, 0xff, 0x68, 0xd0, 0x22, 0xab, 0x00}},
    /* MathUtilFuncSet::rand_real@009e18b0: push ebp; mov ebp,esp; push ecx; mov ecx,[0xc06184] */
    {0x5e18b0, 10, K_REAL, {0x55, 0x8b, 0xec, 0x51, 0x8b, 0x0d, 0x84, 0x61, 0xc0, 0x00}},
    /* Random::reseed@00a39d30: push ebp; mov ebp,esp; mov eax,[ebp+8]; xor [ecx],eax */
    {0x639d30, 8, K_RESEED, {0x55, 0x8b, 0xec, 0x8b, 0x45, 0x08, 0x31, 0x01, 0, 0}},
};
#define NHOOKS (sizeof(HOOKS) / sizeof(HOOKS[0]))

/*
 * The call proxies — the third instrument, and the only one that reads a
 * function's *answer*.
 *
 * A draw hook logs and falls through; that cannot give a return value. A
 * proxy instead **replaces** the function: it logs the arguments, calls the
 * original through the displaced-prologue trampoline, logs `eax`, and
 * returns to the caller cleaning the same bytes the original would. The
 * arguments are read out of the proxy's own frame, so recursion and
 * re-entrancy cost nothing — the shadow stack is the real one.
 *
 * Every proxied site is `__thiscall` and callee-clean (`ret 4*nargs`), which
 * is checked against the listing before it is entered here. `out7` says the
 * eighth argument is a `uchar *` the callee writes; its byte is logged on the
 * return record, so the whole record is compared rather than the number the
 * question happens to want.
 *
 * They are installed **only when `rontrace.cfg` carries a `callwin`**, so an
 * unset configuration is the instrument every capture up to run54 ran.
 */
typedef struct {
    u32 rva;
    u32 len; /* displaced prologue bytes (>= 5, whole instructions, no rel) */
    u32 nargs; /* stack dwords; the callee cleans 4*nargs */
    u32 out7; /* the eighth argument is a uchar* the callee writes */
    u8 expect[10];
} CallSite;

static const CallSite CALLS[] = {
    /* PathFinder::astar_path@00683770(Stack<PathData>*, step, anti) — the
     * search itself, so its entry and return delimit one plan.
     * push ebp; mov ebp,esp; push -1; push 0xa8b801 */
    {0x283770, 10, 3, 0, {0x55, 0x8b, 0xec, 0x6a, 0xff, 0x68, 0x01, 0xb8, 0xa8, 0x00}},
    /* PathFinder::calc_cost@00684e50(from.x, from.y, to.x, to.y, dir, step,
     * depth, uchar *transport) — §5's per-step price, `ret 0x20`.
     * push ebp; mov ebp,esp; sub esp,0x50 */
    {0x284e50, 6, 8, 1, {0x55, 0x8b, 0xec, 0x83, 0xec, 0x50, 0, 0, 0, 0}},
    /* Unit::do_air_physics@005e86d0(UnitOrder *, Coord to.x, Coord to.y) —
     * the bird's frame, and its entry and return **bracket** the two below,
     * so a nested record is a bird's and no other unit's. The two coords
     * are the patrol point already clamped by `WorldData::restrict`, which
     * is the goal `docs/SYNC.md` 3.9 measures the heading against.
     * `ret 0xc`. push ebp; mov ebp,esp; sub esp,0x28 */
    {0x1e86d0, 6, 3, 0, {0x55, 0x8b, 0xec, 0x83, 0xec, 0x28, 0, 0, 0, 0}},
    /* Unit::air_turn_speed@005ea390(sign, 0) — how far the heading may move
     * this frame. `bank_aircraft` is its only caller, and the answer is the
     * **bank angle** scaled onto the type's rate, so this is the one way to
     * read a bird's hidden accumulator from outside. `ret 8`.
     * push ebp; mov ebp,esp; mov eax,[0xc061f0] (absolute, so it moves) */
    {0x1ea390, 8, 2, 0, {0x55, 0x8b, 0xec, 0xa1, 0xf0, 0x61, 0xc0, 0x00, 0, 0}},
    /* Unit::set_new_location@005f8d20(Coord x, Coord y, int, int) — where
     * the step landed. Every moving unit calls it; a bird's are the ones
     * nested inside a `do_air_physics` bracket, with `0, 1` for the last
     * two. `ret 0x10`. push ebp; mov ebp,esp; sub esp,0x20 */
    {0x1f8d20, 6, 4, 0, {0x55, 0x8b, 0xec, 0x83, 0xec, 0x20, 0, 0, 0, 0}},
};
#define NCALLS (sizeof(CALLS) / sizeof(CALLS[0]))

/* ---- state ------------------------------------------------------------- */

static u32 g_base;
static HANDLE g_log = INVALID_HANDLE;
static HANDLE g_proc;
static volatile i32 g_lock;
static i32 g_frame = -1; /* -1 until the first do_frame */
static i32 g_win_lo = -1, g_win_hi = -2; /* re-arm when lo <= frame <= hi */
static i32 g_cover = 1;
static i32 g_cw_lo = -1, g_cw_hi = -2; /* the call proxies log when lo <= frame <= hi */
static i32 g_calls = 0; /* proxies installed (only when a callwin was given) */

static u32 g_funcs[65536];
static u32 g_nfuncs;
static u8 g_orig[TEXT_SIZE]; /* the byte under the int3 */
static u8 g_armed[TEXT_SIZE]; /* 1 while an int3 is planted */
static u8 g_saved[TEXT_SIZE]; /* g_orig valid */

#define BUF_RECS 32768
#define FLUSH_RECS 256 /* 8 KB writes; a kill loses at most this many records */
static u32 g_buf[BUF_RECS * 8];
static u32 g_nbuf;

static char g_dir[300]; /* the exe's directory, with the trailing backslash */

/* ---- the cheat channel ------------------------------------------------- */

static void emit(u32 k, u32 a, u32 b, u32 c, u32 d, u32 e, u32 f);
static void flush(void);

#define MAX_CMDS 512
#define CMD_CHARS 160
typedef struct {
    i32 frame;
    i32 from_chat; /* 1 = chat half (a `cheat` line), 0 = console half (`!` lines) */
    u16 text[CMD_CHARS]; /* UTF-16, NUL-terminated */
} Cmd;
static Cmd g_cmds[MAX_CMDS];
static u32 g_ncmds;
static u32 g_next_cmd; /* lines run in file order; frames must not decrease */

typedef int(__thiscall *parse_cmd_fn)(void *self, void *line, int from_chat, int no_mouse);
typedef void *(__thiscall *string_ctor_fn)(void *self, const u16 *text);
typedef void(__thiscall *string_dtor_fn)(void *self);

/* Run every line due at `frame`. Called at the top of Game::do_frame, before
 * the frame's phases — so a line sees the state at the end of the previous
 * frame, and its effects are in this frame's dump. */
static void run_cmds(i32 frame) {
    while (g_next_cmd < g_ncmds && g_cmds[g_next_cmd].frame <= frame) {
        Cmd *c = &g_cmds[g_next_cmd];
        u32 idx = g_next_cmd++;
        void *console = *(void **)(g_base + RVA_CONSOLE_WIN);
        if (!console) {
            emit(K_INFO, I_CMD_NOCONSOLE, (u32)frame, idx, 0, 0, 0);
            continue;
        }
        u32 str[6]; /* String is 0x14 bytes; one spare */
        ((string_ctor_fn)(g_base + RVA_STRING_CTOR))(str, c->text);
        int r = ((parse_cmd_fn)(g_base + RVA_PARSE_CMD))(console, str, c->from_chat, 1);
        ((string_dtor_fn)(g_base + RVA_STRING_DTOR))(str);
        emit(K_INFO, I_CMD, (u32)frame, idx, (u32)c->from_chat, (u32)r, 0);
        flush();
    }
}

/* ---- freestanding helpers --------------------------------------------- */

void *memset(void *d, int c, unsigned n) {
    u8 *p = (u8 *)d;
    while (n--) *p++ = (u8)c;
    return d;
}

void *memcpy(void *d, const void *s, unsigned n) {
    u8 *p = (u8 *)d;
    const u8 *q = (const u8 *)s;
    while (n--) *p++ = *q++;
    return d;
}

static u32 rd_fs(u32 off) {
    u32 v;
    __asm__ volatile("movl %%fs:(%1), %0" : "=r"(v) : "r"(off));
    return v;
}

static void lock(void) {
    while (__atomic_exchange_n(&g_lock, 1, __ATOMIC_ACQUIRE)) {
    }
}

static void unlock(void) { __atomic_store_n(&g_lock, 0, __ATOMIC_RELEASE); }

static void path_join(char *out, const char *name) {
    u32 i = 0;
    while (g_dir[i]) {
        out[i] = g_dir[i];
        i++;
    }
    while (*name) out[i++] = *name++;
    out[i] = 0;
}

/* ---- the log ----------------------------------------------------------- */

static void flush_locked(void) {
    u32 written;
    if (g_log != INVALID_HANDLE && g_nbuf)
        WriteFile(g_log, g_buf, g_nbuf * 32, &written, 0);
    g_nbuf = 0;
}

static void emit(u32 k, u32 a, u32 b, u32 c, u32 d, u32 e, u32 f) {
    lock();
    u32 *r = g_buf + g_nbuf * 8;
    r[0] = k;
    r[1] = a;
    r[2] = b;
    r[3] = c;
    r[4] = d;
    r[5] = e;
    r[6] = f;
    r[7] = (u32)g_frame;
    if (++g_nbuf == FLUSH_RECS) flush_locked();
    unlock();
}

static void flush(void) {
    lock();
    flush_locked();
    unlock();
}

/* ---- coverage ---------------------------------------------------------- */

/* A patched entry must never also carry an int3: the coverage byte would land
 * on the `jmp` and the handler would restore it, unhooking the site. A
 * proxied function therefore has no HIT record — its CALL/RET records are
 * the stronger evidence anyway. */
static int is_hook_site(u32 rva) {
    for (u32 i = 0; i < NHOOKS; i++)
        if (rva >= HOOKS[i].rva && rva < HOOKS[i].rva + HOOKS[i].len) return 1;
    if (g_calls)
        for (u32 i = 0; i < NCALLS; i++)
            if (rva >= CALLS[i].rva && rva < CALLS[i].rva + CALLS[i].len) return 1;
    return 0;
}

/* Plant an int3 on every listed function that is not currently armed. The
 * first pass records the original byte; later passes only re-plant. */
static u32 arm_all(void) {
    u32 n = 0;
    for (u32 i = 0; i < g_nfuncs; i++) {
        u32 rva = g_funcs[i];
        u32 off = rva - TEXT_RVA;
        if (off >= TEXT_SIZE || is_hook_site(rva)) continue;
        if (g_armed[off]) continue;
        u8 *p = (u8 *)(g_base + rva);
        if (!g_saved[off]) {
            g_orig[off] = *p;
            g_saved[off] = 1;
        }
        *p = 0xCC;
        g_armed[off] = 1;
        n++;
    }
    FlushInstructionCache(g_proc, (void *)(g_base + TEXT_RVA), TEXT_SIZE);
    return n;
}

/* Claim every breakpoint at an address we ever planted on, armed or not:
 * two threads can trap on the same int3 before either handler runs, and the
 * second must not be handed to the game as an unhandled exception. Restoring
 * the byte twice is harmless. */
static i32 WINAPI veh(EXCEPTION_POINTERS *ep) {
    if (ep->rec->code != EXCEPTION_BREAKPOINT) return 0;
    u32 a = (u32)ep->rec->address;
    u32 off = a - g_base - TEXT_RVA;
    if (off >= TEXT_SIZE || !g_saved[off]) {
        /* the other convention: Eip already past the int3 */
        a = ep->ctx->Eip - 1;
        off = a - g_base - TEXT_RVA;
        if (off >= TEXT_SIZE || !g_saved[off]) {
            u32 o2 = (u32)ep->rec->address - g_base - TEXT_RVA;
            emit(K_INFO, I_DECLINED, (u32)ep->rec->address, ep->ctx->Eip,
                 o2 < TEXT_SIZE ? g_armed[o2] : 0xffffffffu,
                 o2 < TEXT_SIZE ? g_saved[o2] : 0xffffffffu, o2 < TEXT_SIZE ? g_orig[o2] : 0);
            flush();
            return 0;
        }
    }
    *(u8 *)a = g_orig[off];
    g_armed[off] = 0;
    FlushInstructionCache(g_proc, (void *)a, 1);
    emit(K_HIT, a, GetCurrentThreadId(), 0, 0, 0, 0);
    ep->ctx->Eip = a;
    return -1; /* EXCEPTION_CONTINUE_EXECUTION */
}

/* ---- the hooks --------------------------------------------------------- */

static u32 stack_ret(u32 ebp, u32 lo, u32 hi, u32 *next) {
    if (ebp < lo || ebp + 8 > hi || (ebp & 3)) {
        *next = 0;
        return 0;
    }
    *next = *(u32 *)ebp;
    return *(u32 *)(ebp + 4);
}

static void __cdecl on_hook(u32 kind, u32 ecx, u32 ebp, u32 caller, u32 arg0) {
    if (kind == K_FRAME) {
        i32 frame = *(i32 *)(ecx + GAME_FRAME_OFF);
        g_frame = frame;
        u32 seed = *(u32 *)(g_base + RVA_GAME_RANDOM);
        u32 armed = 0;
        if (g_cover && frame >= g_win_lo && frame <= g_win_hi) armed = arm_all();
        emit(K_FRAME, (u32)frame, seed, armed, caller, 0, 0);
        flush();
        run_cmds(frame);
        return;
    }
    u32 self = (kind == K_REAL) ? g_base + RVA_GAME_RANDOM : ecx;
    u32 seed = *(u32 *)self;
    u32 hi = rd_fs(4), lo = rd_fs(8); /* TEB StackBase / StackLimit */
    u32 ebp2, ebp3;
    u32 ret2 = stack_ret(ebp, lo, hi, &ebp2);
    u32 ret3 = ret2 ? stack_ret(ebp2, lo, hi, &ebp3) : 0;
    emit(kind, caller, self, seed, ret2, ret3, arg0);
}

/*
 * The stub, per hook (built at attach in an RWX page):
 *
 *   9C                  pushfd
 *   60                  pushad             ; [esp+0x24] = return address, +0x28 = arg0
 *   8B 44 24 28         mov eax,[esp+0x28]
 *   50                  push eax           ; arg0
 *   8B 44 24 28         mov eax,[esp+0x28] ; return address (shifted by the push)
 *   50                  push eax           ; caller
 *   8B 44 24 10         mov eax,[esp+0x10] ; pushad's ebp (shifted by two pushes)
 *   50                  push eax           ; ebp
 *   51                  push ecx           ; this
 *   68 kk kk kk kk      push kind
 *   B8 hh hh hh hh      mov eax, on_hook
 *   FF D0               call eax
 *   83 C4 14            add esp, 20
 *   61                  popad
 *   9D                  popfd
 *   <displaced prologue bytes>
 *   E9 rr rr rr rr      jmp target+len
 */
static u32 build_stub(u8 *s, const HookSite *h) {
    u32 n = 0;
    static const u8 head[] = {0x9C, 0x60, 0x8B, 0x44, 0x24, 0x28, 0x50, 0x8B, 0x44, 0x24,
                              0x28, 0x50, 0x8B, 0x44, 0x24, 0x10, 0x50, 0x51, 0x68};
    memcpy(s, head, sizeof head);
    n = sizeof head;
    *(u32 *)(s + n) = h->kind;
    n += 4;
    s[n++] = 0xB8;
    *(u32 *)(s + n) = (u32)(void *)on_hook;
    n += 4;
    s[n++] = 0xFF;
    s[n++] = 0xD0;
    s[n++] = 0x83;
    s[n++] = 0xC4;
    s[n++] = 0x14;
    s[n++] = 0x61;
    s[n++] = 0x9D;
    memcpy(s + n, (void *)(g_base + h->rva), h->len);
    n += h->len;
    s[n++] = 0xE9;
    u32 back = g_base + h->rva + h->len;
    *(u32 *)(s + n) = back - ((u32)(s + n) + 4);
    n += 4;
    return n;
}

/* ---- the call proxies -------------------------------------------------- */

static void __cdecl on_call(u32 site, u32 self, u32 a0, u32 a1, u32 a2, u32 a3) {
    if (g_frame < g_cw_lo || g_frame > g_cw_hi) return;
    emit(K_CALL, site, self, a0, a1, a2, a3);
}

static void __cdecl on_ret(u32 site, u32 ret, u32 a4, u32 a5, u32 a6, u32 a7) {
    if (g_frame < g_cw_lo || g_frame > g_cw_hi) return;
    u32 out = 0xffffffffu;
    if (site < NCALLS && CALLS[site].out7 && a7 > 0x10000u) out = *(u8 *)a7;
    emit(K_RET, site, ret, a4, a5, a6, out);
}

/*
 * The proxy, per call site (built at attach in an RWX page). It is a whole
 * function with the site's own signature, not a hook that falls through:
 *
 *   55                  push ebp
 *   8B EC               mov ebp,esp        ; [ebp+8 + 4i] = arg i
 *   53                  push ebx
 *   8B D9               mov ebx,ecx        ; this, across both calls
 *   <push arg3..arg0, or 0>                ; cdecl, right to left
 *   53                  push ebx
 *   68 ss ss ss ss      push site
 *   B8 hh hh hh hh      mov eax, on_call
 *   FF D0               call eax
 *   83 C4 18            add esp, 24
 *   <push arg n-1 .. arg 0>                ; the original's own arguments
 *   8B CB               mov ecx,ebx
 *   B8 tt tt tt tt      mov eax, trampoline
 *   FF D0               call eax           ; callee-clean: esp is restored
 *   50                  push eax           ; the answer, saved
 *   <push arg7..arg4, or 0>
 *   50                  push eax
 *   68 ss ss ss ss      push site
 *   B8 hh hh hh hh      mov eax, on_ret
 *   FF D0               call eax
 *   83 C4 18            add esp, 24
 *   58                  pop eax
 *   8D 65 FC            lea esp,[ebp-4]
 *   5B                  pop ebx
 *   5D                  pop ebp
 *   C2 nn 00            ret 4*nargs
 *
 * followed by the trampoline: the displaced prologue and a jump back to
 * `rva + len`, which is a callable copy of the original.
 */
static u32 emit_arg(u8 *s, u32 n, const CallSite *h, u32 i) {
    if (i >= h->nargs) { /* 6A 00  push 0 */
        s[n++] = 0x6A;
        s[n++] = 0x00;
        return n;
    }
    s[n++] = 0xFF; /* FF 75 dd  push [ebp+dd] */
    s[n++] = 0x75;
    s[n++] = (u8)(8 + 4 * i);
    return n;
}

static u32 emit_logcall(u8 *s, u32 n, u32 site, void *fn) {
    s[n++] = 0x68; /* push site */
    *(u32 *)(s + n) = site;
    n += 4;
    s[n++] = 0xB8; /* mov eax, fn */
    *(u32 *)(s + n) = (u32)fn;
    n += 4;
    s[n++] = 0xFF; /* call eax */
    s[n++] = 0xD0;
    s[n++] = 0x83; /* add esp, 24 */
    s[n++] = 0xC4;
    s[n++] = 0x18;
    return n;
}

static u32 build_proxy(u8 *s, const CallSite *h, u32 site) {
    u32 n = 0;
    static const u8 head[] = {0x55, 0x8B, 0xEC, 0x53, 0x8B, 0xD9};
    memcpy(s, head, sizeof head);
    n = sizeof head;
    for (i32 i = 3; i >= 0; i--) n = emit_arg(s, n, h, (u32)i);
    s[n++] = 0x53; /* push ebx (this) */
    n = emit_logcall(s, n, site, (void *)on_call);

    for (i32 i = (i32)h->nargs - 1; i >= 0; i--) n = emit_arg(s, n, h, (u32)i);
    s[n++] = 0x8B; /* mov ecx, ebx */
    s[n++] = 0xCB;
    s[n++] = 0xB8; /* mov eax, trampoline — patched below */
    u32 tramp_imm = n;
    n += 4;
    s[n++] = 0xFF; /* call eax */
    s[n++] = 0xD0;

    s[n++] = 0x50; /* push eax — the answer, saved under our arguments */
    for (i32 i = 7; i >= 4; i--) n = emit_arg(s, n, h, (u32)i);
    s[n++] = 0x50; /* push eax — the answer, as an argument */
    n = emit_logcall(s, n, site, (void *)on_ret);
    s[n++] = 0x58; /* pop eax */
    s[n++] = 0x8D; /* lea esp, [ebp-4] */
    s[n++] = 0x65;
    s[n++] = 0xFC;
    s[n++] = 0x5B; /* pop ebx */
    s[n++] = 0x5D; /* pop ebp */
    s[n++] = 0xC2; /* ret 4*nargs */
    *(u16 *)(s + n) = (u16)(4 * h->nargs);
    n += 2;

    n = (n + 15) & ~15u;
    *(u32 *)(s + tramp_imm) = (u32)(s + n);
    memcpy(s + n, (void *)(g_base + h->rva), h->len);
    n += h->len;
    s[n++] = 0xE9; /* jmp rva + len */
    u32 back = g_base + h->rva + h->len;
    *(u32 *)(s + n) = back - ((u32)(s + n) + 4);
    n += 4;
    return n;
}

static void install_calls(void) {
    u8 *page = (u8 *)VirtualAlloc(0, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    if (!page) return;
    u32 used = 0;
    for (u32 i = 0; i < NCALLS; i++) {
        const CallSite *h = &CALLS[i];
        u8 *t = (u8 *)(g_base + h->rva);
        int ok = 1;
        for (u32 j = 0; j < h->len; j++)
            if (t[j] != h->expect[j]) ok = 0;
        if (!ok) {
            emit(K_INFO, I_HOOK_MISMATCH, h->rva, *(u32 *)t, *(u32 *)(t + 4), *(u32 *)(t + 8),
                 *(u32 *)(t + 12));
            continue;
        }
        u8 *stub = page + used;
        used += (build_proxy(stub, h, i) + 15) & ~15u;
        t[0] = 0xE9;
        *(u32 *)(t + 1) = (u32)stub - ((u32)t + 5);
        for (u32 j = 5; j < h->len; j++) t[j] = 0xCC; /* never executed */
        emit(K_INFO, I_PROXIED, h->rva, (u32)stub, h->nargs, 0, 0);
    }
    FlushInstructionCache(g_proc, (void *)(g_base + TEXT_RVA), TEXT_SIZE);
}

static void install_hooks(void) {
    u8 *page = (u8 *)VirtualAlloc(0, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    if (!page) return;
    u32 used = 0;
    for (u32 i = 0; i < NHOOKS; i++) {
        const HookSite *h = &HOOKS[i];
        u8 *t = (u8 *)(g_base + h->rva);
        int ok = 1;
        for (u32 j = 0; j < h->len; j++)
            if (t[j] != h->expect[j]) ok = 0;
        if (!ok) {
            emit(K_INFO, I_HOOK_MISMATCH, h->rva, *(u32 *)t, *(u32 *)(t + 4), *(u32 *)(t + 8),
                 *(u32 *)(t + 12));
            continue;
        }
        u8 *stub = page + used;
        used += (build_stub(stub, h) + 15) & ~15u;
        t[0] = 0xE9;
        *(u32 *)(t + 1) = (u32)stub - ((u32)t + 5);
        for (u32 j = 5; j < h->len; j++) t[j] = 0xCC; /* never executed */
        emit(K_INFO, I_HOOKED, h->rva, (u32)stub, 0, 0, 0);
    }
    FlushInstructionCache(g_proc, (void *)(g_base + TEXT_RVA), TEXT_SIZE);
}

/* ---- configuration ----------------------------------------------------- */

static u32 read_file(const char *name, void *buf, u32 cap) {
    char path[320];
    path_join(path, name);
    HANDLE h = CreateFileA(path, GENERIC_READ, FILE_SHARE_READ, 0, OPEN_EXISTING,
                           FILE_ATTRIBUTE_NORMAL, 0);
    if (h == INVALID_HANDLE) return 0;
    u32 size = GetFileSize(h, 0);
    if (size > cap) size = cap;
    u32 got = 0;
    ReadFile(h, buf, size, &got, 0);
    CloseHandle(h);
    return got;
}

static i32 parse_int(const char **p) {
    i32 neg = 0, v = 0;
    while (**p == ' ' || **p == '\t') (*p)++;
    if (**p == '-') {
        neg = 1;
        (*p)++;
    }
    while (**p >= '0' && **p <= '9') v = v * 10 + (*(*p)++ - '0');
    return neg ? -v : v;
}

static int key_is(const char *p, const char *k) {
    while (*k)
        if (*p++ != *k++) return 0;
    return *p == '=';
}

/* rontrace.cfg: lines `window=LO-HI` (frames to re-arm at, inclusive; default
 * none), `cover=0|1` (default 1) and `callwin=LO-HI` (frames over which the
 * proxied call sites log their arguments and answers; **absent means the
 * proxies are not installed at all**, which is what every capture before
 * run55 ran). Anything else is ignored. */
static void read_cfg(void) {
    static char cfg[1024];
    u32 n = read_file("rontrace.cfg", cfg, sizeof cfg - 1);
    cfg[n] = 0;
    const char *p = cfg;
    while (*p) {
        if (key_is(p, "window")) {
            p += 7;
            g_win_lo = parse_int(&p);
            if (*p == '-') p++;
            g_win_hi = parse_int(&p);
        } else if (key_is(p, "callwin")) {
            p += 8;
            g_cw_lo = parse_int(&p);
            if (*p == '-') p++;
            g_cw_hi = parse_int(&p);
        } else if (key_is(p, "cover")) {
            p += 6;
            g_cover = parse_int(&p);
        }
        while (*p && *p != '\n') p++;
        while (*p == '\n' || *p == '\r') p++;
    }
}

/* rontrace.cmd: one entry per line, `<frame> <text>`; `<text>` is what would
 * follow `cheat ` in the chat box (`add hoplite who=0 206,78`), or `!` plus a
 * console command (`!quit`). Lines are run in file order at the top of the
 * named sim-frame; a frame lower than the previous line's is clamped to it.
 * `#` starts a comment; blank lines are skipped. ASCII only (the text is
 * widened byte by byte). */
static void read_cmds(void) {
    static char buf[65536];
    u32 n = read_file("rontrace.cmd", buf, sizeof buf - 1);
    buf[n] = 0;
    const char *p = buf;
    i32 last = -1;
    while (*p && g_ncmds < MAX_CMDS) {
        while (*p == ' ' || *p == '\t') p++;
        if (*p == '#' || *p == '\n' || *p == '\r' || !*p) {
            while (*p && *p != '\n') p++;
            while (*p == '\n' || *p == '\r') p++;
            continue;
        }
        if (!(*p >= '0' && *p <= '9')) { /* not a frame: skip the line */
            while (*p && *p != '\n') p++;
            continue;
        }
        Cmd *c = &g_cmds[g_ncmds];
        c->frame = parse_int(&p);
        if (c->frame < last) c->frame = last;
        last = c->frame;
        while (*p == ' ' || *p == '\t') p++;
        c->from_chat = 1;
        if (*p == '!') {
            c->from_chat = 0;
            p++;
        }
        u32 i = 0;
        while (*p && *p != '\n' && *p != '\r' && i < CMD_CHARS - 1) c->text[i++] = (u8)*p++;
        while (i > 0 && (c->text[i - 1] == ' ' || c->text[i - 1] == '\t')) i--;
        c->text[i] = 0;
        if (i) g_ncmds++;
        while (*p && *p != '\n') p++;
        while (*p == '\n' || *p == '\r') p++;
    }
}

/* ---- entry ------------------------------------------------------------- */

/* cdecl, so the export table names it `Hook` — what patch_exe.py imports */
__declspec(dllexport) void Hook(void) {}

i32 WINAPI DllMain(void *inst, u32 reason, void *reserved) {
    (void)inst;
    (void)reserved;
    if (reason == 1) { /* DLL_PROCESS_ATTACH */
        g_proc = GetCurrentProcess();
        g_base = (u32)GetModuleHandleA(0);
        char exe[300];
        u32 n = GetModuleFileNameA(0, exe, sizeof exe);
        u32 cut = 0;
        for (u32 i = 0; i < n; i++)
            if (exe[i] == '\\') cut = i + 1;
        memcpy(g_dir, exe, cut);
        g_dir[cut] = 0;

        char path[320];
        path_join(path, "rontrace.log");
        g_log = CreateFileA(path, GENERIC_WRITE, FILE_SHARE_READ, 0, CREATE_ALWAYS,
                            FILE_ATTRIBUTE_NORMAL, 0);
        read_cfg();
        read_cmds();
        g_nfuncs = read_file("rontrace.funcs", g_funcs, sizeof g_funcs) / 4;

        /* header: magic, version, base, .text, nfuncs, window */
        emit(0x544E4F52u, 1, g_base, TEXT_RVA, TEXT_SIZE, g_nfuncs, (u32)g_win_lo);
        g_buf[7] = (u32)g_win_hi;

        u32 old;
        if (!VirtualProtect((void *)(g_base + TEXT_RVA), TEXT_SIZE, PAGE_EXECUTE_READWRITE, &old)) {
            emit(K_INFO, I_PROTECT_FAIL, 0, 0, 0, 0, 0);
            flush();
            return 1;
        }
        install_hooks();
        if (g_cw_hi >= g_cw_lo) {
            g_calls = 1;
            install_calls();
        }
        if (g_cover) {
            if (!g_nfuncs) emit(K_INFO, I_NOFUNCS, 0, 0, 0, 0, 0);
            AddVectoredExceptionHandler(1, (void *)veh);
            u32 armed = arm_all();
            emit(K_INFO, I_ARMED, (u32)-1, armed, 0, 0, 0);
        }
        emit(K_INFO, I_ATTACH, g_base, g_nfuncs, (u32)g_win_lo, (u32)g_win_hi, g_cover);
        emit(K_INFO, I_CMDS, g_ncmds, 0, 0, 0, 0);
        flush();
        OutputDebugStringA("rontrace: attached\n");
    } else if (reason == 0) { /* DLL_PROCESS_DETACH */
        emit(K_INFO, I_DETACH, 0, 0, 0, 0, 0);
        flush();
        if (g_log != INVALID_HANDLE) CloseHandle(g_log);
        g_log = INVALID_HANDLE;
    }
    return 1;
}
