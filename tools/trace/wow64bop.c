/*
 * wow64bop.c — the smallest program that decides whether the fault behind
 * `docs/ORACLE.md`'s "226: the fault is the bop, not the handler" belongs to
 * Wine or to the machine's 32-bit x86 emulation.
 *
 * The finding it isolates: on this Mac, once a 32-bit thread has been through
 * the vectored-exception path, its next 32->64 transition does not switch
 * mode — the thread lands on `wow64cpu`'s bop entry still in 32-bit code and
 * dies on the first RIP-relative operand there, which decodes as an absolute
 * read of the displacement (0x00004ECD for the syscall entry, 0x00004DC9 for
 * the unix-call one). Everything about the game is irrelevant to that: it
 * needs one vectored handler, one `int 3`, and one syscall afterwards.
 *
 * So this is the falsifier, and it costs nothing to carry to another host —
 * no install, no graphics, no display, no window. Three phases, each
 * announced through `WriteFile`, which is the syscall under test:
 *
 *   A  before anything is armed          — proves the output channel works
 *   B  plant 0xCC on `target`, call it   — the handler restores and continues
 *   C  say so afterwards                 — THIS is the syscall that faults
 *
 * Reading the result:
 *
 *   A, C hits=1, PASS   the whole mechanism works here; `cover=1` is viable
 *                       on this host and the tracer's int3 forest would run
 *   A only, then
 *   "Unhandled page fault … at address <wow64cpu+0x1139 or +0x123d>"
 *                       reproduced: the bop lost the mode switch
 *   A, C hits=0         a different failure — the breakpoint never reached
 *                       the handler; report the Eip convention, not this
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

static HANDLE g_out;
static u8 *g_target;
static u8 g_orig;
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

/* The function the breakpoint is planted on. It must not be inlined and must
 * not be folded away, so it takes an argument and the caller uses the answer. */
__attribute__((noinline)) static i32 target(i32 x) { return x + 1; }

/* Deliberately makes no Win32 call: a call here would itself be the first
 * transition after the exception path, and would move the fault into the
 * handler rather than after it. */
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

    /* B: the trap. If the handler never runs, the process dies here with an
     * unhandled EXCEPTION_BREAKPOINT rather than a page fault. */
    i32 r = target(41);

    /* C: the first syscall after the exception path. This is the one that
     * faults on a host where the bop loses the mode switch. */
    say("C: back from the breakpoint, hits=");
    say_int(g_hits);
    say(" declined=");
    say_int(g_declined);
    say(" target(41)=");
    say_int(r);
    say("\n");

    if (g_hits == 1 && g_declined == 0 && r == 42) {
        say("PASS: a vectored handler can continue a 32-bit int3 here, and a\n"
            "      syscall afterwards still switches mode. cover=1 is viable.\n");
        ExitProcess(0);
    }
    say("ODD: the handler did not behave as expected — read the counters above\n");
    ExitProcess(1);
}
