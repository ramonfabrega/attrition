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

enum {
    K_HIT = 0,
    K_GETF = 1, /* Random::get()          float, this = ecx */
    K_FRAME = 2, /* Game::do_frame entry */
    K_GETI = 3, /* Random::get(int,int)   this = ecx, arg0 = lo */
    K_REAL = 4, /* MathUtilFuncSet::rand_real — game_random, inlined */
    K_INFO = 5,
    K_RESEED = 6, /* Random::reseed(seed)   this = ecx, arg0 = new seed */
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

/* ---- state ------------------------------------------------------------- */

static u32 g_base;
static HANDLE g_log = INVALID_HANDLE;
static HANDLE g_proc;
static volatile i32 g_lock;
static i32 g_frame = -1; /* -1 until the first do_frame */
static i32 g_win_lo = -1, g_win_hi = -2; /* re-arm when lo <= frame <= hi */
static i32 g_cover = 1;

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

static int is_hook_site(u32 rva) {
    for (u32 i = 0; i < NHOOKS; i++)
        if (rva >= HOOKS[i].rva && rva < HOOKS[i].rva + HOOKS[i].len) return 1;
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
 * none) and `cover=0|1` (default 1). Anything else is ignored. */
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
        } else if (key_is(p, "cover")) {
            p += 6;
            g_cover = parse_int(&p);
        }
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
        if (g_cover) {
            if (!g_nfuncs) emit(K_INFO, I_NOFUNCS, 0, 0, 0, 0, 0);
            AddVectoredExceptionHandler(1, (void *)veh);
            u32 armed = arm_all();
            emit(K_INFO, I_ARMED, (u32)-1, armed, 0, 0, 0);
        }
        emit(K_INFO, I_ATTACH, g_base, g_nfuncs, (u32)g_win_lo, (u32)g_win_hi, g_cover);
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
