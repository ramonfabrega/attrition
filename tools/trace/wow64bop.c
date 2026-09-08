/*
 * wow64bop.c — the smallest program that reproduces the fault behind
 * `docs/ORACLE.md`'s item 226 (`cover=1` dying under free Wine on Apple
 * Silicon), and the matrix of shapes that found what it actually is.
 *
 * The finding, as of 2026-09-08 (`docs/ORACLE.md`, "Coverage is back"):
 * **a thread running `popad` — reliably — or `popfd` — now and then — or
 * writing to translated code, while another thread is mid-syscall, breaks
 * that other thread's 32->64 mode switch.** The victim lands on `wow64cpu`'s
 * bop entry still in 32-bit mode and dies on the first RIP-relative operand
 * there, decoded as an absolute read of the displacement (0x00004ECD for the
 * syscall entry, 0x00004DC9 for the unix-call one) — or, from the other
 * side, runs 32-bit ntdll as 64-bit code. Single-threaded, every one of
 * those shapes passes. The vectored-exception path this file was written
 * to indict is innocent: the int3 forest died because its handler wrote
 * code on every hit and the game has threads.
 *
 * Three phases, each announced through `WriteFile`, the syscall under test:
 *
 *   A  before anything is armed          — proves the output channel works
 *   B  the instrumented rounds           — the shape the flags select
 *   C  say so afterwards                 — the syscall that dies, when one does
 *
 * Reading the result:
 *
 *   A, C, PASS         the mechanism works here
 *   A only, then
 *   "Unhandled page fault … at address <wow64cpu+0x1139 or +0x123d>",
 *   or A only and a silent exit
 *                      reproduced: a thread lost its mode switch
 *   LOOP: …            the stub re-entered itself, or a cmpxchg8b never took
 *
 * THE SHAPES, one flag each in `wow64bop.sh` (one flag per expansion — zsh
 * does not word-split `${X:+-DA -DB}`, and the first page-count matrix ran
 * with its flag never defined because of that):
 *
 *   (none)      the int3 shape: plant 0xCC, a vectored handler restores and
 *               continues, ROUNDS times, then the syscall. **Its loop is
 *               not what its source says**: at -O1 clang hoists the pure
 *               `target(41)` out of the loop, so the binary does ROUNDS
 *               plain stores and syscalls and then ONE breakpoint. It
 *               faults 12/12 on this machine and passes 5/5 at ROUNDS=1,
 *               and that difference — plain stores interleaved with
 *               syscalls — is the single-threaded form of the finding.
 *   STUB        the jump-stub shape: the entry becomes `jmp stub`; the stub
 *               records the hit, writes the five original bytes back with
 *               `lock cmpxchg8b` (`patch5`; NOCAS: plain stores) and `ret`s
 *               to the restored entry. Calls go through a volatile pointer
 *               so the loop is real. Passes here in every single-threaded
 *               variant: BATCH (writes with no execution between), NOSAY
 *               (no syscall between writes), PAGES=n (n sites on n pages of
 *               a `.text` pad, up to 1024 = 4 MB, ARMONE for one write into
 *               it), STRADDLE (the write crosses a page), ODS (an
 *               OutputDebugString exception first), IDFLAG (EFLAGS.ID set,
 *               as the game's threads carry it), SAYIN (the syscall from
 *               inside the stub), DIAG (per-round hit counts).
 *   THREADS     a second thread that only makes syscalls while the main
 *               thread runs the rounds. With STUB (writes + popad/popfd):
 *               dies. NORESTORE (arm once, every call through a stub that
 *               runs a copy of the prologue; no writes after the arm):
 *               dies. NORESTORE NOFLAGS (no pushfd/popfd): intermittent.
 *               NOARM (the control: no stub at all, both threads storming
 *               syscalls): survives 4/4. LOOPOP=n / POPFONLY: a million of
 *               one instruction class on the main thread and nothing else
 *               — popad (2) dies, popfd (POPFONLY) dies now and then, an
 *               indirect call into the RWX page (3), lahf/seto+sahf (4),
 *               push/pop (5) and a direct call (6) all survive 3/3.
 *
 * That last matrix is what the tracer's coverage stub is built from: no
 * popad, no popfd, no write after attach.
 *
 * Build: tools/trace/wow64bop.sh (no arguments, nothing to install).
 * Run:   wine wow64bop.exe        — the verdict is on stdout.
 *
 * Note for anyone extending it: putting a Win32 call *inside* `veh` — as
 * `tracer.c` did with `FlushInstructionCache` until 2026-09-04 — moves the
 * death from phase C into the handler itself. That is the same fault one
 * step earlier, not a second one.
 */

typedef unsigned char u8;
typedef unsigned int u32;
typedef int i32;
typedef void *HANDLE;

#define WINAPI __stdcall
#define IMPORT(ret, name, args) __declspec(dllimport) ret WINAPI name args

IMPORT(i32, WriteFile, (HANDLE, const void *, u32, u32 *, void *));
IMPORT(i32, VirtualProtect, (void *, u32, u32, u32 *));
IMPORT(void *, AddVectoredExceptionHandler, (u32, void *));
IMPORT(HANDLE, GetStdHandle, (u32));
IMPORT(void, ExitProcess, (u32));
#ifdef STUB
IMPORT(void *, VirtualAlloc, (void *, u32, u32, u32));
IMPORT(void, OutputDebugStringA, (const char *));
IMPORT(HANDLE, CreateThread, (void *, u32, void *, void *, u32, u32 *));
IMPORT(void, Sleep, (u32));
#define MEM_COMMIT 0x1000u
#define MEM_RESERVE 0x2000u
#endif

typedef struct {
    u32 code, flags;
    void *record, *address;
    u32 nparams;
    u32 info[15];
} EXCEPTION_RECORD;

/* x86 CONTEXT: Eip is at +0xb8 — the same layout tracer.c carries. */
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

#define EXCEPTION_BREAKPOINT 0x80000003u
#define PAGE_EXECUTE_READWRITE 0x40u
#define STD_OUTPUT_HANDLE ((u32)-11)
#ifndef ROUNDS
#define ROUNDS 8 /* -DROUNDS=n to bisect: one round alone can pass here */
#endif

static HANDLE g_out;
static u8 *g_target;
#ifndef STUB
static u8 g_orig;
#endif
static i32 g_hits;
static i32 g_declined;

/* The syscall under test. Every `say` is a 32->64 transition. */
static void say(const char *s) {
    u32 n = 0, w;
    while (s[n]) n++;
    WriteFile(g_out, s, n, &w, 0);
}

static void say_int(i32 v) {
    char buf[12];
    u32 i = sizeof buf;
    buf[--i] = 0;
    if (!v) buf[--i] = '0';
    while (v > 0) {
        buf[--i] = (char)('0' + v % 10);
        v /= 10;
    }
    say(buf + i);
}

#ifdef STUB
static void say_hex(u32 v) {
    char buf[9];
    for (i32 i = 7; i >= 0; i--) {
        u32 d = v & 15;
        buf[i] = (char)(d < 10 ? '0' + d : 'a' + d - 10);
        v >>= 4;
    }
    buf[8] = 0;
    say(buf);
}
#endif

/* The function the breakpoint is planted on. It must not be inlined and must
 * not be folded away, so it takes an argument and the caller uses the answer. */
__attribute__((noinline)) static i32 target(i32 x) { return x + 1; }

/* Deliberately makes no Win32 call: a call here would itself be the first
 * transition after the exception path, and would move the fault into the
 * handler rather than after it. */
#ifndef STUB
static i32 WINAPI veh(EXCEPTION_POINTERS *ep) {
    if (ep->rec->code != EXCEPTION_BREAKPOINT) return 0;
    u8 *a = (u8 *)ep->rec->address;
    if (a != g_target) {
        /* the other convention: Eip already past the int3 */
        a = (u8 *)(ep->ctx->Eip - 1);
        if (a != g_target) {
            g_declined++;
            return 0;
        }
    }
    *a = g_orig;
    g_hits++;
    ep->ctx->Eip = (u32)a;
    return -1; /* EXCEPTION_CONTINUE_EXECUTION */
}
#endif

#ifdef STUB
/*
 * The stub shape. No exception is ever raised: the entry's first five bytes
 * become `jmp stub`, and the stub — the same code `tracer.c` builds per
 * function — pushes an index, saves flags and registers, calls `on_stub`,
 * which writes the five original bytes back and returns the entry address,
 * then restores everything and `ret`s to the freshly restored entry.
 *
 *   per entry:   68 ii ii ii ii   push index
 *                E9 rr rr rr rr   jmp body
 *   body:        9C 60            pushfd; pushad
 *                8B 44 24 24      mov eax,[esp+0x24]   ; the index
 *                50               push eax
 *                B8 hh hh hh hh   mov eax, on_stub
 *                FF D0            call eax             ; eax = entry
 *                83 C4 04         add esp,4
 *                89 44 24 24      mov [esp+0x24],eax   ; index slot := entry
 *                61 9D            popad; popfd
 *                C3               ret                  ; to the entry
 */
#ifndef PAGES
#define PAGES 1
#endif
#if PAGES > 1
/* -DPAGES=n: n arming sites, one per page of the executable's own .text —
 * a pad of `ret` bytes, so every site is a callable function of one
 * instruction and the write pattern is the tracer's: n pages touched. */
static const u8 g_pad[PAGES * 4096] __attribute__((section(".text"), aligned(4096))) = {
    [0 ...(PAGES * 4096 - 1)] = 0xC3};
#endif
static u8 *g_sites[PAGES];
static u8 g_orig5s[PAGES][5];
static i32 g_saved5;
/* The call goes through a volatile pointer: `target` is pure, and at -O1
 * clang hoists a direct call out of the ROUNDS loop, so the rounds would
 * never enter it (found by disassembly, 2026-09-08). */
static i32(__cdecl *volatile g_tgt)(i32);

/* Write five bytes over live code. Where [p, p+5) sits inside one 8-byte
 * word — three quarters of the executable's entries — the write is one
 * `lock cmpxchg8b`, so a thread arriving at the entry sees the old five or
 * the new five and never a mix. Where it straddles a word the write is two
 * plain stores and the window between them is real; `docs/ORACLE.md`
 * ("Coverage is back") sizes it. */
static i32 g_cas_spins;
static i32 g_round_hits;

/* `lock cmpxchg8b` n bytes at offset `at` of the 8-byte word `w`. */
static void cas_word(u8 *w, u32 at, const u8 *bytes, u32 n) {
    g_cas_spins = 0;
    for (;;) {
        if (++g_cas_spins > 1000) {
            say("LOOP: cmpxchg8b never succeeds on the code page\n");
            ExitProcess(4);
        }
        u32 lo = *(volatile u32 *)w, hi = *(volatile u32 *)(w + 4);
        u8 b[8];
        *(u32 *)b = lo;
        *(u32 *)(b + 4) = hi;
        for (u32 i = 0; i < n; i++) b[at + i] = bytes[i];
        u32 nlo = *(u32 *)b, nhi = *(u32 *)(b + 4);
        u8 ok;
        __asm__ volatile("lock cmpxchg8b %0\n\tsete %1"
                         : "+m"(*(u32 *)w), "=q"(ok), "+a"(lo), "+d"(hi)
                         : "b"(nlo), "c"(nhi)
                         : "cc", "memory");
        if (ok) return;
    }
}

/* Write five bytes over live code with locked writes only: one
 * `cmpxchg8b` where [p, p+5) sits inside an 8-byte word, two — one per
 * word — where it straddles. -DNOCAS uses plain byte stores instead, which
 * is the shape that dies on the bop (see the header). */
static void patch5(u8 *p, const u8 *five) {
#ifdef NOCAS
    for (u32 i = 0; i < 5; i++) p[i] = five[i];
#else
    u32 at = (u32)p & 7;
    u8 *w = p - at;
    u32 first = 8 - at;
    if (first > 5) first = 5;
    cas_word(w, at, five, first);
    if (first < 5) cas_word(w + 8, 0, five + first, 5 - first);
#endif
}

static i32 g_round;
static u32 g_trace[16][3];
static void dump_trace(void) {
    for (u32 i = 0; i < 16 && g_trace[i][1]; i++) {
        say("  hit: round ");
        say_int((i32)g_trace[i][0]);
        say(" caller+0x");
        say_hex(g_trace[i][1]);
        say(" byte0=0x");
        say_hex(g_trace[i][2]);
        say("\n");
    }
}
static u32 __cdecl on_stub(u32 index, u32 caller) {
    if (index >= PAGES) {
        say("LOOP: a stub with an index out of range\n");
        ExitProcess(5);
    }
    g_target = g_sites[index];
    if (g_hits < 16) {
        g_trace[g_hits][0] = (u32)g_round;
        g_trace[g_hits][1] = caller;
        g_trace[g_hits][2] = g_target[0];
    }
#ifndef NORESTORE
    patch5(g_target, g_orig5s[index]);
#endif
#ifdef SAYIN
    /* -DSAYIN: the syscall from inside the stub, after the restore — the
     * tracer's shape when a HIT record flushes from the stub. */
    say("i");
#endif
    g_hits++;
    if (++g_round_hits > 64) {
        /* the entry was restored and still jumped here: the translator ran
         * a stale translation of the entry after the write */
        say("LOOP: the stub is re-entered after the entry was restored\n");
        dump_trace();
        ExitProcess(3);
    }
    return (u32)g_target;
}

#ifdef THREADS
static volatile i32 g_sc_go, g_sc_done, g_sc_n;
static u32 WINAPI syscaller(void *arg) {
    (void)arg;
    while (g_sc_go) {
        u32 w;
        WriteFile(g_out, "", 0, &w, 0); /* a syscall that prints nothing */
        g_sc_n++;
    }
    g_sc_done = 1;
    return 0;
}
#endif

static void arm(u8 *stub, u32 index) {
#ifdef NOARM /* the control: no jmp is ever planted, the stub never runs */
    (void)stub;
    (void)index;
    return;
#endif
    u8 j[5];
    u8 *t = g_sites[index];
    if (!g_saved5) {
        for (u32 k = 0; k < PAGES; k++)
            for (u32 i = 0; i < 5; i++) g_orig5s[k][i] = g_sites[k][i];
        g_saved5 = 1;
    }
    j[0] = 0xE9;
    *(u32 *)(j + 1) = (u32)stub - ((u32)t + 5);
    patch5(t, j);
}

void WINAPI start(void) {
    u32 old;
    g_out = GetStdHandle(STD_OUTPUT_HANDLE);
    say("A: output channel up, nothing armed yet\n");

    g_tgt = target;
#ifdef STRADDLE
    /* A position-independent copy of `mov eax,[esp+4]; inc eax; ret`, two
     * bytes before a page boundary, so the five-byte patch crosses it. */
    u8 *pages = (u8 *)VirtualAlloc(0, 8192, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    if (!pages) {
        say("FAIL: VirtualAlloc refused the straddle pages\n");
        ExitProcess(2);
    }
    static const u8 body[] = {0x8B, 0x44, 0x24, 0x04, 0x40, 0xC3};
    g_target = pages + 4096 - 2;
    for (u32 i = 0; i < sizeof body; i++) g_target[i] = body[i];
    g_tgt = (i32(__cdecl *)(i32))(void *)g_target;
    say("straddle: target two bytes before a page boundary\n");
#elif PAGES > 1
    if (!VirtualProtect((void *)g_pad, sizeof g_pad, PAGE_EXECUTE_READWRITE, &old)) {
        say("FAIL: VirtualProtect refused the pad\n");
        ExitProcess(2);
    }
    for (u32 k = 0; k < PAGES; k++) g_sites[k] = (u8 *)g_pad + 4096 * k;
    g_target = g_sites[0];
    g_tgt = (i32(__cdecl *)(i32))(void *)g_target;
    say("pages: sites on every page of a .text pad\n");
#else
    g_target = (u8 *)(void *)target;
    if (!VirtualProtect(g_target, 16, PAGE_EXECUTE_READWRITE, &old)) {
        say("FAIL: VirtualProtect refused the target\n");
        ExitProcess(2);
    }
#endif
    g_sites[0] = g_target;
    (void)old;

    u8 *page = (u8 *)VirtualAlloc(0, 65536, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    if (!page) {
        say("FAIL: VirtualAlloc refused the stub page\n");
        ExitProcess(2);
    }
    /* pushfd; pushad; mov eax,[esp+0x28] (caller ret); push eax;
     * mov eax,[esp+0x28] (index, shifted by the push); push eax; mov eax,on_stub */
#ifdef NOFLAGS
    static const u8 bodyc[] = {0x60, 0x8B, 0x44, 0x24, 0x24, 0x50,
                               0x8B, 0x44, 0x24, 0x24, 0x50, 0xB8};
#else
    static const u8 bodyc[] = {0x9C, 0x60, 0x8B, 0x44, 0x24, 0x28, 0x50,
                               0x8B, 0x44, 0x24, 0x28, 0x50, 0xB8};
#endif
    u8 *b = page + 16384;
    u32 n = 0;
    for (; n < sizeof bodyc; n++) b[n] = bodyc[n];
    *(u32 *)(b + n) = (u32)(void *)on_stub;
    n += 4;
#ifdef JMPREG
    /* -DJMPREG: the `ret` becomes `add esp,4; jmp dword ptr [esp-4]` — an
     * indirect jump rather than a return, to see whether the translator's
     * return path is what re-enters the stale entry. */
    static const u8 tail[] = {0xFF, 0xD0, 0x83, 0xC4, 0x08, 0x89, 0x44, 0x24, 0x24, 0x61, 0x9D,
                              0x83, 0xC4, 0x04, 0xFF, 0x64, 0x24, 0xFC};
#elif defined(NORESTORE) && defined(NOFLAGS)
    /* call; add esp,8; popad; add esp,4 (drop the index); copy; jmp target+5 */
    static const u8 tail[] = {0xFF, 0xD0, 0x83, 0xC4, 0x08, 0x61, 0x83, 0xC4, 0x04,
                              0x8B, 0x44, 0x24, 0x04, 0x40, 0xE9, 0, 0, 0, 0};
#elif defined(NORESTORE)
    /* call; add esp,8; popad; popfd; add esp,4 (drop the index); copy; jmp target+5 */
    static const u8 tail[] = {0xFF, 0xD0, 0x83, 0xC4, 0x08, 0x61, 0x9D, 0x83, 0xC4, 0x04,
                              0x8B, 0x44, 0x24, 0x04, 0x40, 0xE9, 0, 0, 0, 0};
#elif defined(NOFLAGS)
    static const u8 tail[] = {0xFF, 0xD0, 0x83, 0xC4, 0x08, 0x89, 0x44, 0x24, 0x20, 0x61, 0xC3};
#else
    static const u8 tail[] = {0xFF, 0xD0, 0x83, 0xC4, 0x08, 0x89, 0x44, 0x24, 0x24, 0x61, 0x9D, 0xC3};
#endif
    for (u32 i = 0; i < sizeof tail; i++) b[n++] = tail[i];
#ifdef NORESTORE
    *(u32 *)(b + n - 4) = ((u32)(void *)target + 5) - (u32)(b + n);
#endif
    if (PAGES * 10 > 16384) {
        say("FAIL: too many sites for one stub page\n");
        ExitProcess(2);
    }
    for (u32 k = 0; k < PAGES; k++) {
        u8 *stub = page + 10 * k;
        stub[0] = 0x68;
        *(u32 *)(stub + 1) = k;
        stub[5] = 0xE9;
        *(u32 *)(stub + 6) = (u32)b - ((u32)stub + 10);
    }
    u8 *stub = page;

#ifdef THREADS
    /* -DTHREADS: a second thread that only makes syscalls, while this one
     * arms and restores the entry ROUNDS times — the game's shape: the
     * thread that dies at the bop is never the one writing code. */
    g_sc_go = 1;
    CreateThread(0, 0, (void *)syscaller, 0, 0, 0);
    Sleep(50);
#endif
#ifdef IDFLAG
    /* -DIDFLAG: set EFLAGS.ID (bit 21) first — the game's threads carry it
     * and the stub's popfd writes it back; the falsifier's do not. */
    __asm__ volatile("pushfd\n\torl $0x200000, (%%esp)\n\tpopfd" : : : "memory", "cc");
    say("idflag set\n");
#endif
    /* B: arm, call, ROUNDS times — the int3 shape's loop with the stub in
     * the breakpoint's place, and the same `say` (a syscall) each round. */
    i32 r = 0;
#ifdef BATCH
    /* -DBATCH: the arming shape of `arm_all` — ROUNDS writes to the entry
     * with no execution of it in between (each one re-arms the same five
     * bytes), then one call, then the syscall. -DNOSAY drops the syscall
     * between the writes. This is the shape the shipped int3 falsifier
     * turned out to have (its loop's call was hoisted, see the header). */
#if PAGES > 1
#ifdef ARMONE
    /* -DARMONE: one write into a pad that is RWX end to end — the range of
     * the protection change alone, without the writes. */
    g_round = 1;
    arm(page, 0);
#else
    for (u32 k = 0; k < PAGES; k++) {
        g_round = (i32)k + 1;
        arm(page + 10 * k, k);
#ifndef NOSAY
        say(".");
#endif
    }
#endif
#else
    for (i32 i = 0; i < ROUNDS; i++) {
        g_round = i + 1;
        arm(stub, 0);
#ifndef NOSAY
        say(".");
#endif
    }
#endif
#ifdef ODS
    /* the exception dispatch the tracer's own attach raises */
    OutputDebugStringA("wow64bop: armed\n");
    say(" ods");
#endif
    g_round_hits = 0;
    r = g_tgt(41);
#if PAGES > 1
    r = 42; /* a `ret` site answers nothing; the syscall after it is the test */
#endif
    g_hits = ROUNDS; /* one hit is the whole batch's; the counter below wants ROUNDS */
    say(" batch hits=");
    say_int(g_round_hits);
#else
#ifdef NORESTORE
    arm(stub, 0);
#endif
#ifdef POPFONLY
    /* -DPOPFONLY: no stub at all — a million pushfd/popfd pairs, with the
     * other thread making syscalls. */
    for (i32 i = 0; i < 1000000; i++) __asm__ volatile("pushfd\n\tpopfd" : : : "memory", "cc");
#endif
#ifdef LOOPOP
    /* -DLOOPOP=n: a million iterations of one instruction class on this
     * thread while the other makes syscalls — which class breaks the bop.
     * 2 pushad/popad; 3 an indirect call into the RWX page (a `ret` there);
     * 4 the lahf/seto flag save and its sahf restore; 5 push/pop eax;
     * 6 a direct call into the RWX page. */
    page[4000] = 0xC3;
    void *retsite = page + 4000;
    for (i32 i = 0; i < 1000000; i++) {
#if LOOPOP == 2
        __asm__ volatile("pushad\n\tpopad" : : : "memory");
#elif LOOPOP == 3
        __asm__ volatile("call *%0" : : "r"(retsite) : "memory", "cc");
#elif LOOPOP == 4
        __asm__ volatile("pushl %%eax\n\tlahf\n\tseto %%al\n\taddb $0x7f, %%al\n\tsahf\n\tpopl %%eax" : : : "memory", "cc");
#elif LOOPOP == 5
        __asm__ volatile("pushl %%eax\n\tpopl %%eax" : : : "memory");
#elif LOOPOP == 6
        ((void (*)(void))retsite)();
#endif
    }
    say("loop done ");
#endif
    for (i32 i = 0; i < ROUNDS; i++) {
        g_round_hits = 0;
        g_round = i + 1;
#ifndef NORESTORE
        arm(stub, 0);
#endif
        r = g_tgt(41);
#ifdef DIAG
        say(" ");
        say_int(g_round_hits);
#else
        say(".");
#endif
    }
#endif

    g_round = 99;
#ifdef THREADS
    g_sc_go = 0;
    Sleep(100);
    say("\nsyscaller: ");
    say_int(g_sc_n);
    say(" syscalls, done=");
    say_int(g_sc_done);
#endif
    say("\nC: back from the stubs, hits=");
    say_int(g_hits);
    say("\n");
    dump_trace();
    say(" declined=");
    say_int(g_declined);
    say(" target(41)=");
    say_int(r);
    say("\n");

    if (g_hits == ROUNDS && g_declined == 0 && r == 42) {
        say("PASS: a jump stub can record an entry, restore it and return to it\n"
            "      here, and a syscall afterwards still switches mode.\n");
        ExitProcess(0);
    }
    say("ODD: the stub did not behave as expected — read the counters above\n");
    ExitProcess(1);
}
#else
void WINAPI start(void) {
    u32 old;
    g_out = GetStdHandle(STD_OUTPUT_HANDLE);
    say("A: output channel up, nothing armed yet\n");

    g_target = (u8 *)(void *)target;
    if (!VirtualProtect(g_target, 16, PAGE_EXECUTE_READWRITE, &old)) {
        say("FAIL: VirtualProtect refused the target\n");
        ExitProcess(2);
    }
    AddVectoredExceptionHandler(1, (void *)veh);
    g_orig = *g_target;
    *g_target = 0xCC;

    /* B: the trap, ROUNDS times. One continue is not enough: the game gets
     * two clean ones and dies on the transition after them, at the same
     * instruction with the same registers whichever functions are armed
     * (probes 3 and 4, `docs/ORACLE.md` under 226), so the count is part of
     * the shape and a single-shot probe would call a broken host healthy. */
    i32 r = 0;
    for (i32 i = 0; i < ROUNDS; i++) {
        g_orig = *g_target;
        *g_target = 0xCC;
        r = target(41);
        say(".");
    }

    /* C: the syscalls above and this one are the transitions under test. */
    say("\nC: back from the breakpoints, hits=");
    say_int(g_hits);
    say(" declined=");
    say_int(g_declined);
    say(" target(41)=");
    say_int(r);
    say("\n");

    if (g_hits == ROUNDS && g_declined == 0 && r == 42) {
        say("PASS: a vectored handler can continue a 32-bit int3 here, and a\n"
            "      syscall afterwards still switches mode. cover=1 is viable.\n");
        ExitProcess(0);
    }
    say("ODD: the handler did not behave as expected — read the counters above\n");
    ExitProcess(1);
}
#endif
