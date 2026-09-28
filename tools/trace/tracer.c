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
 *   2. Function coverage. Every function entry in the coverage table
 *      (`rontrace.funcs`, built by `funcs.py` from the Ghidra export and
 *      the executable) gets a 5-byte `jmp` to a stub of its own; the stub
 *      records the entry, runs a **copy** of the instructions the jmp
 *      displaced (branches rewritten, the table carries them) and jumps
 *      back to the rest of the function. The entry is written once, at
 *      attach, with one thread alive, and never touched again: no
 *      exception is raised and no code is restored. The first entry of
 *      each function over the run is recorded, and inside the frame window
 *      of `rontrace.cfg` the first entry per frame, which gives per-frame
 *      sets for those frames; outside the window a recorded function's
 *      stub takes a two-instruction fast path. Until 2026-09-08 this was an
 *      `int 3` per entry, a vectored handler and a restore per hit; under
 *      free Wine on Apple Silicon that died at a 32->64 transition, and
 *      `wow64bop.c` found the reason in the game's own shape: a write to
 *      translated code while another thread is mid-syscall breaks that
 *      thread's mode switch — the exception path was never the cause.
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
 *      nor a coverage stub can give, and the one thing the gamelog has no
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

#if defined(RON_RESTORE_PROBE) && !defined(RON_SEARCH_GRAPH)
#error RON_RESTORE_PROBE requires structural graph capture
#endif

#if defined(RON_SEARCH_GRAPH) && !defined(RON_SEARCH_CENSUS)
#error RON_SEARCH_GRAPH requires the bounded census lane
#endif

#if defined(RON_CONGESTION_PROBE) && (!defined(RON_SEARCH_CENSUS) || defined(RON_COMMAND_PROBE))
#error RON_CONGESTION_PROBE requires census and excludes the single-unit command probe
#endif

#if defined(RON_SEARCH_CENSUS) && (defined(RON_PATH_CAPSULE) || defined(RON_ORDER_CAPSULE) || defined(RON_CAPSULE_PROBE) || defined(RON_HIDE_SCENE))
#error RON_SEARCH_CENSUS excludes capsule and suppression experiments
#endif

#if defined(RON_CAPSULE_PROBE) && (!defined(RON_COMMAND_PROBE) || defined(RON_HIDE_SCENE))
#error RON_CAPSULE_PROBE requires RON_COMMAND_PROBE and excludes RON_HIDE_SCENE
#endif

#if defined(RON_PATH_CAPSULE) && (defined(RON_ORDER_CAPSULE) || defined(RON_CAPSULE_PROBE) || defined(RON_HIDE_SCENE))
#error RON_PATH_CAPSULE excludes other capsule and suppression experiments
#endif

#if defined(RON_ORDER_CAPSULE) && (defined(RON_CAPSULE_PROBE) || defined(RON_HIDE_SCENE))
#error RON_ORDER_CAPSULE excludes RON_CAPSULE_PROBE and RON_HIDE_SCENE
#endif

#if defined(RON_HIDE_SCENE) && !defined(RON_TURN_PROBE)
#error RON_HIDE_SCENE requires RON_TURN_PROBE for the render-call witness
#endif

#if defined(RON_STATE_FRAME) && defined(RON_RESTORE_PROBE)
#error Frame snapshots and restore probes use separate capture lanes
#endif

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
IMPORT(void *, GetModuleHandleA, (const char *));
IMPORT(u32, GetModuleFileNameA, (void *, char *, u32));
IMPORT(i32, FlushInstructionCache, (HANDLE, const void *, u32));
IMPORT(HANDLE, GetCurrentProcess, (void));
IMPORT(u32, GetCurrentThreadId, (void));
IMPORT(void, OutputDebugStringA, (const char *));
IMPORT(HANDLE, CreateThread, (void *, u32, void *, void *, u32, u32 *));
IMPORT(void, Sleep, (u32));

#define GENERIC_READ 0x80000000u
#define GENERIC_WRITE 0x40000000u
#define FILE_SHARE_READ 1u
#define CREATE_ALWAYS 2u
#define OPEN_EXISTING 3u
#define FILE_ATTRIBUTE_NORMAL 0x80u
#define INVALID_HANDLE ((HANDLE)(i32)-1)
#define PAGE_EXECUTE_READWRITE 0x40u
#define PAGE_EXECUTE_READ 0x20u
#define PAGE_READWRITE 0x04u
#define MEM_COMMIT 0x1000u
#define MEM_RESERVE 0x2000u
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
    I_PROXIED = 12, /* a call site is proxied: a = rva, b = stub, c = nargs, d = site id */
    I_COVER = 13, /* the coverage stubs are built: a = region, b = stubs, c = table entries excluded */
    I_DROPPED = 14, /* records lost to a full buffer since the last flush: a = count */
    I_UNITID = 15, /* RON_COLLIDE_PROBE: a = site, b = UnitData*, c = o, d = who */
    I_ISSUE = 17, /* an `@` issuer line: a = frame, b = line index | refusal << 16
                   * (0 = issued; see `issue_line`), c = package size before,
                   * d = after (or the refused object), e = objects named */
    I_ISSUE_UNIT = 18, /* one named object as issued: a = frame, b = o | who << 16,
                        * c = uid, d = x, e = y (decoded internal) */
    I_COLLBLOCK = 16, /* RON_COLLIDE_PROBE: a = cx | cy << 8 | src << 16 | part << 24,
                       * b..e = four dwords of the block's 256 bits (part 0: 0..3,
                       * part 1: 4..7). src 0 = the world's live block, 1 = the
                       * pathfinder's copy (`fill_slots`); src 2 = no live block,
                       * b = the pointer WData holds (0 or ~0), one record only. */
};

typedef struct {
    u32 rva;
    u32 len; /* displaced prologue bytes (>= 5, whole instructions, no rel) */
    u32 kind;
    u8 expect[10];
} HookSite;

static const HookSite HOOKS[] = {
#ifdef RON_STATE_FRAME
    /* GameLog::end_frame: mov ecx,[game_log.log_start_frame]. */
    {0x5329d0, 6, 9, {0x8b,0x0d,0xd4,0x13,0xeb,0x00}},
    /* Normal do_frame continuation after end_frame: cmp byte [ebx+0x8a0],0. */
    {0x192586, 7, 10, {0x80,0xbb,0xa0,0x08,0x00,0x00,0x00}},
#endif
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

#if defined(RON_TARGET_PROBE) && defined(RON_TURN_PROBE)
#error "RON_TARGET_PROBE and RON_TURN_PROBE both claim call-site ids 8 and 9"
#endif
#if defined(RON_LEADER_PROBE) && (defined(RON_TARGET_PROBE) || defined(RON_TURN_PROBE))
#error "RON_LEADER_PROBE claims call-site ids 8, 9 and 10 too"
#endif
#if defined(RON_COLLIDE_PROBE) && \
    (defined(RON_TARGET_PROBE) || defined(RON_TURN_PROBE) || defined(RON_LEADER_PROBE))
#error "RON_COLLIDE_PROBE claims call-site ids 8 through 12 too"
#endif
#if defined(RON_GUARD_PROBE) && (defined(RON_TARGET_PROBE) || defined(RON_TURN_PROBE) || \
                                 defined(RON_LEADER_PROBE) || defined(RON_COLLIDE_PROBE))
#error "RON_GUARD_PROBE claims call-site ids 8 through 13 too"
#endif

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
    /* PathFinder::astar_caravan_road@00685990(Stack<PathData>*, whoA, whoB,
     * p4, p5, caravan, p7) — the road search, so its entry and return
     * **bracket** one plan the way `do_air_physics` brackets a bird's
     * frame. `ret 0x1c`.
     * push ebp; mov ebp,esp; push -1; push 0xa8b81b */
    {0x285990, 10, 7, 0, {0x55, 0x8b, 0xec, 0x6a, 0xff, 0x68, 0x1b, 0xb8, 0xa8, 0x00}},
    /* PathFinderData::valid_roadcoord@00688740(x, y, from.x, from.y, p5,
     * p6, p7, p8) — the gate, and the only record that carries a road
     * candidate's **coordinate**: `calc_road_cost` is handed a pooled
     * `PathNode *`, so the tile it prices is the one the call before it
     * admitted. `ret 0x20`. push ebp; mov ebp,esp; sub esp,0x18 */
    {0x288740, 6, 8, 0, {0x55, 0x8b, 0xec, 0x83, 0xec, 0x18, 0, 0, 0, 0}},
    /* PathFinder::calc_road_cost@00686300(PathNode *, whoA, whoB, dir,
     * this) — `docs/ROADS.md` §5.2's price, and the answer is the whole
     * question: a road search's node sequence is comparable term by term
     * only if the original's own costs are on the record. `ret 0x14`.
     * push ebx; mov ebx,esp; sub esp,8 — not the usual frame, so the
     * displaced prologue is `53 8b dc 83 ec 08`. */
    {0x286300, 6, 5, 0, {0x53, 0x8b, 0xdc, 0x83, 0xec, 0x08, 0, 0, 0, 0}},
#ifdef RON_TARGET_PROBE
    /* The target-selection triple (item 386, `docs/COMBAT.md` 18). One
     * search's candidate list is not in any dump: `near_o` records only the
     * **nearest** candidate, and the order that lands records only the
     * winner, so a tie between equal units is unfalsifiable from outside.
     * `find_nearby_target`'s entry and return **bracket** one search, and the
     * two inside it are the two numbers the ranking is built from.
     *
     * Object::find_nearby_target@00648da0(max_dist, int *who, add_order,
     * cavarch, flags) - `ret 0x14`; the return is the chosen `o` or -1, and
     * `flags` rides the RET record as a4.
     * push ebp; mov ebp,esp; sub esp,0x70 */
    {0x248da0, 6, 5, 0, {0x55, 0x8b, 0xec, 0x83, 0xec, 0x70, 0, 0, 0, 0}},
    /* ObjectData::attack_dist@006488f0(o, who, x, y) - 13.1's edge-to-edge
     * distance, and the numerator of the score's divisor. `ret 0x10`.
     * push ebp; mov ebp,esp; sub esp,8 */
    {0x2488f0, 6, 4, 0, {0x55, 0x8b, 0xec, 0x83, 0xec, 0x08, 0, 0, 0, 0}},
    /* Object::compare_target@0064e5c0(o, who, in_range, ai) - 12.3's value,
     * the other half. `ret 0x10`.
     * push ebp; mov ebp,esp; sub esp,0x2c */
    {0x24e5c0, 6, 4, 0, {0x55, 0x8b, 0xec, 0x83, 0xec, 0x2c, 0, 0, 0, 0}},
#endif
#ifdef RON_LEADER_PROBE
    /* The AI's own decisions (DECISIONS 41 §6, parked 367; the seventh
     * pass built it). The original logs a Leader's *state* and never its
     * reasoning: `LEADERS=9` prints the ranked make list after the fact,
     * and no category prints an offer, so item 432 recovered the four
     * offers of Great Lakes 9380 by replaying the list backwards. These
     * three sites print them forwards.
     *
     * Leader::create_units@006c40a0(void) — the bracket: a `make_me`
     * nested in it is a unit offer and not `create_buildings`'. `ret`.
     * push ebp; mov ebp,esp; sub esp,0xbc — nine bytes, no rel. */
    {0x2c40a0, 9, 0, 0, {0x55, 0x8b, 0xec, 0x81, 0xec, 0xbc, 0x00, 0x00, 0x00, 0}},
    /* MakeList::make_me@006c9be0(t, val, escrow, cat, city, up, p7, num,
     * wx, wy) — the offer itself: ten dwords, `ret 0x28`; the first eight
     * ride the records (`p7` is stored nowhere by the callee) and the two
     * coordinates do not. push ebp; mov ebp,esp; push ebx; mov ebx,ecx */
    {0x2c9be0, 6, 10, 0, {0x55, 0x8b, 0xec, 0x53, 0x8b, 0xd9, 0, 0, 0, 0}},
    /* Leader::make_this@006c94f0(slot) — the purchase, `ret 4`; the
     * answer is whether it bought. push ebp; mov ebp,esp; and esp,-8 */
    {0x2c94f0, 6, 1, 0, {0x55, 0x8b, 0xec, 0x83, 0xe4, 0xf8, 0, 0, 0, 0}},
#endif
#ifdef RON_COLLIDE_PROBE
    /* The collision sweep, read from inside (`docs/COLLISION.md` 9, item
     * 456). The refusal is dumped nowhere: `detect_unit_collision` writes
     * `collide_o`/`collide_who` and 5.4's snap arm clears them two
     * instructions after the probe returns, so no gamelog category at any
     * detail level can name the unit that refused a step. These five print
     * the sweep forwards - who asked, which cell the probe stopped on,
     * which units the 3x3 walk examined, and what the corner rule made of
     * each. Every argument count below is the function's own `ret <imm>`
     * divided by four, read off the image.
     *
     * Unit::detect_unit_collision@00617060(x, y, quick, boats, p5, nocoll,
     * top_only) - the **bracket**: everything below it belongs to one
     * unit's probe of one point. `ret 0x1c`, and `this` is the asking
     * unit. push ebp; mov ebp,esp; sub esp,0x40 */
    {0x217060, 6, 7, 0, {0x55, 0x8b, 0xec, 0x83, 0xec, 0x40, 0, 0, 0, 0}},
    /* CollCheck::collide_here@00682540(o, who, ucx, ucy, coll_size,
     * &hit_x, &hit_y, nocoll) - 4.2's probe, and the only record that says
     * in o/who terms **who is asking**: the first two arguments are the
     * caller's own pair. The answer is whether a cell was found; the cell
     * itself rides the next call, not this one, because the two out
     * pointers carry it. `ret 0x20`. push ebp; mov ebp,esp; sub esp,0x2c */
    {0x282540, 6, 8, 0, {0x55, 0x8b, 0xec, 0x83, 0xec, 0x2c, 0, 0, 0, 0}},
    /* UnitData::will_be_corner@00609fa0(hit_x, hit_y, ucx, ucy) - called
     * once, immediately after a successful probe, so its presence **is**
     * `collide_here != 0` and its first two arguments are **the hit cell
     * as values**. Its answer is the asking unit's half of 4.3's corner
     * rule. `ret 0x10`. push ebp; mov ebp,esp; movzx eax,[ecx+9] */
    {0x209fa0, 7, 4, 0, {0x55, 0x8b, 0xec, 0x0f, 0xb6, 0x41, 0x09, 0, 0, 0}},
    /* UnitData::is_here@0060a0c0(&hit_x, &hit_y) - one call per candidate
     * the 3x3 world-cell walk reaches, `this` the candidate: the census of
     * **who was looked at**, and its answer is who covers the hit cell.
     * The two arguments are pointers to the cell, so the identity record
     * below is what makes the call readable. `ret 8`.
     * push ebp; mov ebp,esp; mov eax,[ecx+0x10] */
    {0x20a0c0, 6, 2, 0, {0x55, 0x8b, 0xec, 0x8b, 0x41, 0x10, 0, 0, 0, 0}},
    /* UnitData::is_corner@0060a040(hit_x, hit_y, self) - the other half of
     * the corner rule, `this` the **blocker**, and it is reached only when
     * `will_be_corner` was non-zero and every soft arm declined. So a call
     * here says the scan got to the last gate, and its absence beside a
     * `will_be_corner 0` says the collision was hard without one. `ret
     * 0xc`. push ebp; mov ebp,esp; sub esp,8 */
    {0x20a040, 6, 3, 0, {0x55, 0x8b, 0xec, 0x83, 0xec, 0x08, 0, 0, 0, 0}},
#endif
#ifdef RON_GUARD_PROBE
    /* A unit's own step, bracketed (item 696, `docs/GOLDEN.md` §19): on
     * run190's tick 721 the guard `0/6`, on its post with its `GUARD`
     * reading on-post, takes one (21, 1) step and names the wagon in
     * `collide_o`, with no order added and no draw. `set_new_location` (site
     * 4) is always proxied; these six say which of the unit's functions it
     * was nested in. Each `this` is named by an INFO 15. Argument counts are
     * each function's own `ret <imm>` divided by four.
     *
     * Unit::do_guard@005e5c70(UnitOrder *) - `ret 4`.
     * push ebp; mov ebp,esp; and esp,-8 */
    {0x1e5c70, 6, 1, 0, {0x55, 0x8b, 0xec, 0x83, 0xe4, 0xf8, 0, 0, 0, 0}},
    /* Unit::do_move@005f7b30(UnitOrder *) - `ret 4`.
     * push ebp; mov ebp,esp; sub esp,0x40 */
    {0x1f7b30, 6, 1, 0, {0x55, 0x8b, 0xec, 0x83, 0xec, 0x40, 0, 0, 0, 0}},
    /* Unit::move_step@005faf30(MoveOrder *, step) - `ret 8`.
     * push ebp; mov ebp,esp; sub esp,0xa2c */
    {0x1faf30, 9, 2, 0, {0x55, 0x8b, 0xec, 0x81, 0xec, 0x2c, 0x0a, 0x00, 0x00, 0}},
    /* Unit::resolve_unit_collision@005f9d30(x, y) - `ret 8`.
     * push ebp; mov ebp,esp; push -1 */
    {0x1f9d30, 5, 2, 0, {0x55, 0x8b, 0xec, 0x6a, 0xff, 0, 0, 0, 0, 0}},
    /* Unit::detect_unit_collision@00617060, as RON_COLLIDE_PROBE's site 8.
     * `ret 0x1c`. push ebp; mov ebp,esp; sub esp,0x40 */
    {0x217060, 6, 7, 0, {0x55, 0x8b, 0xec, 0x83, 0xec, 0x40, 0, 0, 0, 0}},
    /* Unit::detect_boat_collision@005fa8b0(x, y, mates) - the push, whose
     * answer is 1 "handled" or 0 "hand the step to the land scan". `ret
     * 0xc`. push ebp; mov ebp,esp; and esp,-8 */
    {0x1fa8b0, 6, 3, 0, {0x55, 0x8b, 0xec, 0x83, 0xe4, 0xf8, 0, 0, 0, 0}},
#endif
#ifdef RON_TURN_PROBE
    /* GuyData::turn_speed(int), ret 4; opt-in field replay experiment. */
    {0x1de340, 6, 1, 0, {0x55, 0x8b, 0xec, 0x53, 0x8b, 0xd9, 0, 0, 0, 0}},
    /* Scene::render(int,int,int), ret 12: presentation boundary witness. */
    {0x4b3270, 10, 3, 0, {0x55, 0x8b, 0xec, 0x6a, 0xff, 0x68, 0x98, 0xed, 0xa9, 0x00}},
#endif
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

/* The coverage table, one record per listed function (`funcs.py`). */
typedef struct {
    u32 rva;
    u8 orig_len; /* bytes the copy stands in for; 0 = excluded */
    u8 code_len;
    u8 nfix;
    u8 pad;
    u8 code[24]; /* the displaced prologue, branches rewritten to rel32 */
    u8 fix_off[4]; /* rel32 slots in `code` */
    u32 fix_target[4]; /* their absolute targets */
} Entry;
#define MAX_FUNCS 65536
static Entry g_funcs[MAX_FUNCS];
static u32 g_nfuncs;
static u8 g_hit[MAX_FUNCS]; /* entered at least once this run */
static i32 g_hit_frame[MAX_FUNCS]; /* the frame of the last recorded entry */
static u8 g_fast[MAX_FUNCS]; /* the stub skips its call: recorded, and no window open */
static i32 g_in_window; /* the current frame is inside rontrace.cfg's window */
static u8 *g_stubs; /* one STUB_BYTES slot per table entry */
static u32 g_nstubs;
#define STUB_BYTES 80 /* 42 of stub, up to 24 of copy, 5 of jmp */
/* 4 MB: every function in the table can be entered between two flushes (a
 * window frame records each once) and the coverage stubs never flush — see
 * `emit_nosys`. */
#define BUF_RECS 131072
#ifndef FLUSH_RECS
#define FLUSH_RECS 256 /* 8 KB writes; a kill loses at most this many records */
#endif
static u32 g_buf[BUF_RECS * 8];
static u32 g_nbuf;
static u32 g_dropped; /* records lost to a full buffer, reported at the next flush */

static char g_dir[300]; /* the exe's directory, with the trailing backslash */

/* ---- the cheat channel ------------------------------------------------- */

static void emit(u32 k, u32 a, u32 b, u32 c, u32 d, u32 e, u32 f);
static void flush(void);

#define MAX_CMDS 512
#define CMD_CHARS 160
typedef struct {
    i32 frame;
    i32 from_chat; /* 1 = chat half (a `cheat` line), 0 = console half (`!` lines),
                    * 2 = an issuer line (`@`), which never reaches parse_cmd */
    u16 text[CMD_CHARS]; /* UTF-16, NUL-terminated */
} Cmd;
static Cmd g_cmds[MAX_CMDS];
static u32 g_ncmds;
static u32 g_next_cmd; /* lines run in file order; frames must not decrease */

typedef int(__thiscall *parse_cmd_fn)(void *self, void *line, int from_chat, int no_mouse);
typedef void *(__thiscall *string_ctor_fn)(void *self, const u16 *text);
typedef void(__thiscall *string_dtor_fn)(void *self);

/* ---- the issuer lines -------------------------------------------------
 *
 * A line whose text starts with `@` is not a cheat: it is an order, put into
 * the local player's `CommandPackage` through the original's own issuer, so
 * the turn pump processes it exactly as it processes a click (item 676,
 * `docs/GOLDEN.md` §17; `docs/DECISIONS.md` 41 §1 and 49). Twenty-four verbs:
 *
 *   `@move <who> <x> <y> <o> [<o> ...]`   internal coordinates, object ids
 *   `@patrol <who> <x> <y> <o> [<o> ...]` the same, through issue_patrol
 *   `@guard <who> <ox> <whom> <o> [<o> ...]` the charge's id and owner in
 *                                         place of the point, issue_guard
 *   `@follow <who> <ox> <whom> <o> [<o> ...]` the leader's, issue_follow
 *   `@garrison <who> <ox> <whom> <o> [<o> ...]` the building's,
 *                                         issue_garrison
 *   `@eject <who> <b> [<b> ...]`          building ids, issue_eject_all
 *   `@form <who> <form> <rotate> <o> [<o> ...]` a formation index and a
 *                                         rotation, issue_form
 *   `@attack <who> <ox> <whom> <o> [<o> ...]` the target's id and owner,
 *                                         issue_attack
 *   `@amove <who> <x> <y> <o> [<o> ...]`  `@move`'s fields, ATTACK_TO
 *   `@explore <who> <x> <y> <o> [<o> ...]` `@move`'s fields, EXPLORE_TO
 *   `@flee <who> <x> <y> <o> [<o> ...]`   `@move`'s fields, FLEE_TO
 *   `@flight <who> <ox> <whom> <o> [<o> ...]` an aircraft's flight to one's
 *                                         own base or carrier, issue_flight
 *                                         with MOVE_TO
 *   `@strike <who> <ox> <whom> <o> [<o> ...]` an aircraft's flight at an
 *                                         enemy, issue_flight with ATTACK
 *   `@build <who> <x> <y> <type> <o> [<o> ...]` a building's `TypeIndex`
 *                                         placed at the point, issue_build
 *   `@spell <who> <type> <ox> <whom> <x> <y> <o> [<o> ...]` a craft's
 *                                         `TypeIndex` cast on the target
 *                                         picked at the point, issue_spell
 *   `@settransport <who> <flag> <o> [<o> ...]` the auto-transport toggle,
 *                                         issue_set_transport
 *   `@repair <who> <ox> <whom> <o> [<o> ...]` the building's id and owner,
 *                                         issue_swarm_around with REPAIR
 *   `@buildmask <who> <mask> <b> [<b> ...]` building ids, issue_buildmask
 *   `@queueup <who> <type> <num> <b> [<b> ...]` building ids, issue_queue_up
 *   `@unqueue <who> <p> <b> [<b> ...]`   building ids, issue_unqueue once
 *                                         per building
 *   `@gatherpoint <who> <x> <y> <action> <b> [<b> ...]` building ids,
 *                                         issue_gather_point
 *   `@gatherpointadd <who> <x> <y> <action> <b> [<b> ...]` the same with
 *                                         add_to_end 1 (verb 20)
 *   `@launchpatrol <who> <x> <y> <b> [<b> ...]` building ids,
 *                                         issue_launch_patrol, QUEUE_NEW
 *   `@launchpatrolall <who> <x> <y> <b> [<b> ...]` the same with
 *                                         QUEUE_LAST and shift 1
 *   `@launchstrike <who> <ox> <whom> <b> [<b> ...]` building ids,
 *                                         issue_flight with ATTACK
 *   `@launchpatrolctrl`, `@launchpatrolalt`, `@launchstrikectrl`,
 *   `@launchstrikealt`                    the same with ctrl or alt 1
 *   `@launchmove <who> <ox> <whom> <b> [<b> ...]` a base of one's own,
 *                                         issue_flight with MOVE_TO
 *
 * calls `CommandManager::issue_move_to@00941720(&command_manager, group, x,
 * y, QUEUE_NEW 2, set_angle 0, angle 0, MOVE_TO 1, form -1, width -1,
 * disembark 0)` — the arguments `WorldMap::on_right_up@008c7050:203` passes
 * for a plain right-click. The lab's probes (live_move_probe.h, L15) passed
 * orders 0, form 0, width 0, which is not a click. `group` is a GroupOut
 * with `num` at +0xc, `who` at +0x4a and the object list at +0x8cc, the
 * three fields `CommandPackage::add_group@0094bb60` reads. The package is
 * `command_manager.local_package` (+0x28, size at +0x10, data at +0x12); the
 * command is processed by the next `process_turn`, before the next frame's
 * `do_frame`, so its order is in the next logger block's dump.
 *
 * `@patrol` calls `CommandManager::issue_patrol@00941800(&command_manager,
 * group, x, y, QUEUE_NEW 2)` — the arguments `WorldMap::on_right_up@008c7050:
 * 206` passes for a patrol click with no modifier — and appends a 10-byte
 * `patrol` (type 0x0a) behind the group (item 693, `docs/GOLDEN.md` §18).
 *
 * `@guard` calls `CommandManager::issue_guard@00941ed0(&command_manager,
 * group, ox, whom, QUEUE_NEW 2)` — what `Options::picked_spot@00721c40`
 * passes through `GroupOut::issue_guard` for an unmodified pick — and
 * appends a 13-byte `guard` (type 0x1f, `[ox][whom][queued]`) behind the
 * group (item 696, `docs/GOLDEN.md` §19). The charge is not checked here:
 * `Group::action_guard` asks it, at process time, what the chapter measures.
 *
 * `@follow` calls `CommandManager::issue_follow@00941e70(&command_manager,
 * group, ox, whom, QUEUE_NEW 2)` — what `Options::picked_spot@00721c40`
 * passes through `GroupOut::issue_follow@00708980` for an unmodified pick —
 * and appends a 13-byte `follow` (type 0x1e, `[ox][whom][queued]`) behind
 * the group (item 714, `docs/GOLDEN.md` §20). As for the guard, the leader
 * is `Group::action_follow`'s to ask at process time.
 *
 * `@garrison` calls `CommandManager::issue_garrison@00941a70(&command_manager,
 * group, ox, whom, QUEUE_NEW 2)` — what `Options::picked_spot@00721c40`
 * passes through `GroupOut::issue_garrison@0070a9b0` for an unmodified pick
 * of a building — and appends a 13-byte `garrison` (type 0x14, `[ox][whom]
 * [queued]`) behind the group (item 718, `docs/GOLDEN.md` §21). The
 * building is `Group::action_garrison`'s to ask at process time.
 *
 * `@eject` calls `CommandManager::issue_eject_all@00941ca0(&command_manager,
 * group, 0, -1, -1, -1)` on a group of the player's own **buildings** —
 * the bytes the Eject button appends (`Options::exec@007188c0` option 0x1f
 * → `Options::do_eject_all@0071c470(console->who)` → `GroupOut::
 * issue_eject_all@00708b90`, whose own-player arm writes `back_to_work 0`
 * and three −1s) — a 17-byte `eject_all` (type 0x1a, `[back_to_work][who]
 * [eject_o][eject_who]`) behind the group. Its objects are checked as
 * buildings: in `objects`, not `units`, and on `Build::vftable`.
 *
 * `@form` calls `CommandManager::issue_form@00941580(&command_manager,
 * group, form, rotate, QUEUE_NEW 2)` — what `Options::do_formation@
 * 007215b0` passes through `GroupOut::issue_form@0070b090` for a formation
 * button with no modifier held (rotate 0) — and appends a 13-byte `form`
 * (type 0x03, `[form][rotate][queued]`) behind the group (item 723,
 * `docs/GOLDEN.md` §22). `Group::action_form` reads it at process time.
 *
 * `@attack` calls `CommandManager::issue_attack@009415e0(&command_manager,
 * group, ox, whom, ignore 0, QUEUE_NEW 2)` — what `Console::
 * execute_at_cursor@007c6630:2741` passes through `GroupOut::issue_attack
 * @0070b060` for an unmodified right-click on an enemy — and appends a
 * 17-byte `attack` (type 0x04, `[ox][whom][ignore][queued]`) behind the
 * group (item 731, `docs/GOLDEN.md` §23). `ignore` is read only against
 * siege, special and spy members, and the chapter's cast has none. The
 * target is not checked here: `CommandPackage::process_attack@00949c30`
 * and `Group::action_attack` ask it at process time.
 *
 * `@amove` is `@move` with `orders` ATTACK_TO (2) for MOVE_TO (1): the
 * bytes `WorldMap::on_right_up@008c7050:199` passes for a ctrl+right-click
 * on the ground, the attack-move (item 731).
 *
 * `@explore` and `@flee` are `@move` with `orders` EXPLORE_TO (3) and
 * FLEE_TO (4) (item 738, `docs/GOLDEN.md` §24). EXPLORE_TO is the bytes
 * `Options::picked_spot@00721c40:905` passes for the Explore button's pick
 * on the ground. FLEE_TO is what its Flee arm (`:946`) passes, to a
 * friendly building's point, before a QUEUE_LAST `issue_garrison` of that
 * building; the verb issues the move alone, to the point it is given.
 *
 * `@flight` and `@strike` call `CommandManager::issue_flight@00941d40(
 * &command_manager, group, ox, whom, orders, shift 0, ctrl 0, alt 0)` and
 * append a 25-byte `flight` (type 0x1c, `[ox][whom][shift][ctrl][alt]
 * [orders]`) behind the group (item 746, `docs/GOLDEN.md` §25). `Console::
 * execute_at_cursor@007c6630:2849` passes MOVE_TO through `GroupOut::
 * issue_flight@00708b10` for aircraft right-clicked on their own base, and
 * `:2835` ATTACK for aircraft right-clicked on an enemy. The target is not
 * checked here: `Group::action_flight@006fb260` asks it at process time.
 *
 * `@build` calls `CommandManager::issue_build@00941c30(&command_manager,
 * group, x, y, x, y, type, QUEUE_NEW 2)` and appends a 25-byte `build`
 * (type 0x19, `[x][y][x2][y2][type][queued]`) behind the group (item 779,
 * `docs/GOLDEN.md` §26). `Options::picked_spot@00721c40:531` passes the
 * drop's point, the drag's start and `QUEUE_NEW` for an unmodified click
 * through `GroupOut::issue_build@00708c60`; the drag's start is the click
 * itself when nothing is dragged, and `Group::action_build@00707510`
 * reads only the first point. The site, the price and the builders are
 * `action_build`'s to decide at process time.
 *
 * `@spell` calls `CommandManager::issue_spell@00941b80(&command_manager,
 * group, type, ox, whom, x, y)` and appends a 21-byte `spell` (type 0x17,
 * `[ox][whom][type][x][y]`) behind the group (item 790, `docs/GOLDEN.md`
 * §27). `Options::picked_spot@00721c40:749` passes a targeted craft's
 * `TypeIndex`, the object `SpellTypeData::find_target` found under the
 * cursor and the cursor's own point through `Options::target_spell@
 * 0071dda0` for an unmodified pick; the DLL skips `GroupOut::
 * validate_spell`, which `Group::action_spell@006fe1a0` runs again at
 * process time, where the caster, the target and the price are decided.
 *
 * `@settransport` calls `CommandManager::issue_set_transport@00941910(
 * &command_manager, group, flag)` and appends a 5-byte `set_transport`
 * (type 0x0e, `[flag i32]`) behind the group (item 803, `docs/GOLDEN.md`
 * §28). `Options::do_transport@0071c500` passes `!GroupData::
 * can_transport()` through `GroupOut::issue_set_transport@0070ad10` for
 * the transport button, and `Options::picked_spot@00721c40`'s
 * `OPTION_DISEMBARK` passes 1 ahead of an `issue_move_to`. The flag is
 * `Group::action_set_transport@007024b0`'s to apply at process time,
 * forced to 0 there when the leader's transport level is 0.
 *
 * `@repair` calls `CommandManager::issue_swarm_around@009416b0(
 * &command_manager, group, ox, whom, QUEUE_NEW 2, REPAIR 13)` and appends
 * a 17-byte `swarm_around` (type 0x06, `[ox][whom][queued][orders]`)
 * behind the group (item 813, `docs/GOLDEN.md` §29). `Console::
 * execute_at_cursor@007c6630` passes those arguments through `GroupOut::
 * issue_swarm_around@0070afc0` for an unmodified right-click on a damaged,
 * finished building of one's own or an ally's; `Options::picked_spot@
 * 00721c40` passes the same for the Repair pick. The DLL skips the
 * cursor's damage test: `Group::action_swarm_around@0070fbe0` takes the
 * members at process time and `Unit::do_repair@005ee420` asks the damage.
 *
 * `@buildmask` calls `CommandManager::issue_buildmask@00941f80(
 * &command_manager, group, mask, 1)` and appends a 9-byte `buildmask` (type
 * 0x21, `[mask i32][set i32]`) behind a group of buildings (item 867,
 * `docs/GOLDEN.md` §32). `Options::set_air_repeat@0071c740` passes 0x80
 * through `GroupOut::issue_buildmask@00708820` for the repeat button on a
 * selection of buildings. The issuer writes `set` 1 whatever its third
 * argument (the listing never reads `[ebp+0x10]`), and
 * `Group::action_buildmask@006fc9a0` reads neither: it toggles the bit off
 * the first member's state, on each member `WallData::valid_buildmask@
 * 0063e2a0` admits, at process time.
 *
 * `@queueup` calls `CommandManager::issue_queue_up@00941be0(
 * &command_manager, group, type, num)` and appends a 9-byte `queue_up`
 * (type 0x18, `[type i32][num i32]`) behind a group of buildings (item
 * 877, `docs/GOLDEN.md` §33). `GroupOut::issue_queue_up@00708c90` passes a
 * selection's click on a unit's button through it; the issuer tests
 * nothing of the type (under the emulator, 14 bytes with the group).
 * `Group::action_queue_up@006fdbb0` calls `Build::queue_up(type, 1)` on
 * each member `num` times at process time, and the price and the room are
 * asked there.
 *
 * `@gatherpoint` calls `CommandManager::issue_gather_point@00941b20(
 * &command_manager, group, x, y, action, add_to_end 0)` and appends a
 * 17-byte `gather_point` (type 0x16, `[x][y][action][add_to_end]`) behind
 * a group of buildings (item 928, `docs/GOLDEN.md` §39). A right-click on
 * the ground with buildings selected passes action 0
 * (`WorldMap::on_right_up@008c7050:228`, `Console::execute_at_cursor@
 * 007c6630:3405`); on a friendly object its point and 1, an enemy's 2
 * (`execute_at_cursor:3497`); the Clear button −1, −1, 0, 0
 * (`Options::do_clear_gather@0071ce70`). `add_to_end` is the shift key's,
 * and the verb passes 0; `@gatherpointadd` passes 1 (item 955). The issuer tests nothing (under the emulator, 17
 * bytes as passed): `Group::action_gather_point@006ff1b0` takes the
 * members at process time.
 *
 * `@launchpatrol` calls `CommandManager::issue_launch_patrol@00941860(
 * &command_manager, group, x, y, QUEUE_NEW 2, shift 0, ctrl 0, alt 0)` and
 * appends a 25-byte `launch_patrol` (type 0x0b, `[x][y][queued][shift]
 * [ctrl][alt]`) behind a group of buildings (item 976, `docs/GOLDEN.md`
 * §42; under the emulator, 30 bytes with the fresh group): a right-click on
 * the ground with an Airbase selected (`WorldMap::on_right_up@008c7050:240`,
 * `Console::execute_at_cursor@007c6630:3230`). `@launchpatrolall` passes
 * the shift-click's QUEUE_LAST 1 and shift 1. `@launchstrike` calls
 * `issue_flight@00941d40(&command_manager, group, ox, whom, ATTACK 10, 0,
 * 0, 0)` on a group of buildings: an Airbase's right-click on an enemy in
 * air range (`execute_at_cursor`), which `Group::action_flight@006fb260`
 * turns into `action_launch_flight`. Neither issuer tests the objects:
 * `Group::action_launch_patrol@00703580` and `action_launch_flight@006fbfb0`
 * pick the planes at process time. `@launchpatrolctrl`/`@launchpatrolalt`
 * pass ctrl or alt 1 on the plain click, `@launchstrikectrl`/
 * `@launchstrikealt` the same on `issue_flight`, and `@launchmove` is
 * `issue_flight(group, ox, whom, MOVE_TO 1, 0, 0, 0)` — the right-click on
 * another base of one's own (`execute_at_cursor`'s `group_air` arm, whose
 * last three are shift, ctrl and alt) — item 1009, `docs/GOLDEN.md` §43.
 *
 * `@unqueue` calls `CommandManager::issue_unqueue@00942c40(&command_manager,
 * b, p)` once per building, as `Options::exec@007188c0`'s option 0xa6
 * loops a selection, and appends a 15-byte `unqueue` (type 0x30,
 * `[who i32][o i32][p i32][uid i16]`) with **no `group`** (item 884,
 * `docs/GOLDEN.md` §34; under the emulator, 15 bytes a call). `p` is a
 * queue slot, or the selector's negatives: −1 the last, −5 five, −10 all
 * (`Build::action_unqueue@00620280`). The issuer reads the building's
 * `who`, `o` and `uid` and nothing else.
 *
 * Refusals, each an I_ISSUE with the refusal in b's high half and nothing
 * issued: 1 no console or
 * `who` is not the console's player (`process_group` hands another player's
 * group to `is_team` and drops it); 2 the issuer's prologue is not the
 * shipped one; 3 an object is out of the registry, not active, not `who`'s,
 * not that id, or not a captain (`add_group` would drop it silently) — for
 * `@eject`, `@buildmask`, `@queueup`, `@unqueue`, `@gatherpoint` and the
 * three `@launch` verbs, not a building; 4 the
 * package cannot hold a fresh group and the move (the issuer returns void and
 * appends nothing); 5 the text did not parse; 6 the package did not grow. */
#define RVA_CONSOLE 0x806210u /* MiscAccess::console, VA 0xc06210 (Console *) */
#define CONSOLE_PLAY_OFF 0x2a0u /* Console::play */
#define RVA_UNITS 0x80aeb0u /* units: per player, stride 0x1c: +4 length, +0x10 slots */
#define RVA_COMMAND_MANAGER 0xa8ff60u /* command_manager, VA 0xe8ff60 */
#include "issue_guard.h"
#define RVA_OBJECTS 0x80618cu /* GameAccess::objects, VA 0xc0618c (ObjectsData *):
                               * lists[who] at +4 + who * 0x1c, length +4, slots +0x10 */
#define RVA_BUILD_VFTABLE 0x742174u /* Build::vftable, VA 0xb42174 */
#define ISSUE_MAX 32
static u8 g_groupout[0x9d0];

/* The coverage table's record for `rva` when `arm_all` planted a stub over
 * it, else 0. The table is sorted by RVA (`funcs.py`). */
static const Entry *cover_entry(u32 rva) {
    u32 lo = 0, hi = g_nfuncs;
    while (lo < hi) {
        u32 mid = lo + (hi - lo) / 2;
        if (g_funcs[mid].rva < rva) lo = mid + 1;
        else hi = mid;
    }
    return lo < g_nfuncs && g_funcs[lo].rva == rva && g_funcs[lo].orig_len ? &g_funcs[lo] : 0;
}

static int issue_int(const u16 **t, i32 *out) {
    const u16 *p = *t;
    while (*p == ' ' || *p == '\t') p++;
    i32 sign = 1, v = 0, any = 0;
    if (*p == '-') { sign = -1; p++; }
    while (*p >= '0' && *p <= '9') { v = v * 10 + (i32)(*p - '0'); p++; any = 1; }
    *t = p;
    *out = sign * v;
    return any;
}

static int issue_verb(const u16 **t, const char *verb) {
    const u16 *p = *t;
    for (; *verb; verb++, p++)
        if (*p != (u16)*verb) return 0;
    *t = p;
    return 1;
}

static void issue_line(i32 frame, u32 idx, const u16 *text) {
    u16 *pkg_size = (u16 *)(g_base + RVA_COMMAND_MANAGER + 0x28 + 0x10);
    u32 before = *pkg_size;
    const u16 *t = text;
    /* 0 `move`, 1 `patrol`, 2 `guard`, 3 `follow`, 4 `garrison`, 5
     * `eject`, 6 `form`, 7 `attack`, 8 `amove`, 9 `explore`, 10 `flee`,
     * 11 `flight`, 12 `strike`, 13 `build`, 14 `spell`, 15
     * `settransport`, 16 `repair`, 17 `buildmask`, 18 `queueup`, 19
     * `unqueue`, 20 `gatherpoint`, 21 `launchpatrol`, 22 `launchpatrolall`,
     * 23 `launchstrike`:
     * the issuer and its prologue (`issue_guard.h`)
     * and its command's size. A guard's two numbers are the charge's `ox`
     * and `whom`, a follow's the leader's, a garrison's the building's, an
     * attack's and a repair's the target's, a form's the formation and
     * the rotation; an eject has none. `@gatherpointadd` is verb 20 with
     * `add_to_end` 1 (item 955), the shift-click that appends a point. */
    i32 gather_add = 0, launch_ctrl = 0, launch_alt = 0, launch_move = 0;
    i32 verb = issue_verb(&t, "move ")       ? 0
               : issue_verb(&t, "patrol ")   ? 1
               : issue_verb(&t, "guard ")    ? 2
               : issue_verb(&t, "follow ")   ? 3
               : issue_verb(&t, "garrison ") ? 4
               : issue_verb(&t, "eject ")    ? 5
               : issue_verb(&t, "form ")     ? 6
               : issue_verb(&t, "attack ")   ? 7
               : issue_verb(&t, "amove ")    ? 8
               : issue_verb(&t, "explore ")  ? 9
               : issue_verb(&t, "flee ")     ? 10
               : issue_verb(&t, "flight ")   ? 11
               : issue_verb(&t, "strike ")   ? 12
               : issue_verb(&t, "build ")    ? 13
               : issue_verb(&t, "spell ")    ? 14
               : issue_verb(&t, "settransport ") ? 15
               : issue_verb(&t, "repair ")   ? 16
               : issue_verb(&t, "buildmask ") ? 17
               : issue_verb(&t, "queueup ")  ? 18
               : issue_verb(&t, "unqueue ")  ? 19
               : issue_verb(&t, "gatherpointadd ") ? (gather_add = 1, 20)
               : issue_verb(&t, "gatherpoint ") ? 20
               : issue_verb(&t, "launchpatrolall ") ? 22
               : issue_verb(&t, "launchpatrolctrl ") ? (launch_ctrl = 1, 21)
               : issue_verb(&t, "launchpatrolalt ") ? (launch_alt = 1, 21)
               : issue_verb(&t, "launchpatrol ") ? 21
               : issue_verb(&t, "launchstrikectrl ") ? (launch_ctrl = 1, 23)
               : issue_verb(&t, "launchstrikealt ") ? (launch_alt = 1, 23)
               : issue_verb(&t, "launchstrike ") ? 23
               : issue_verb(&t, "launchmove ") ? (launch_move = 1, 23)
                                             : -1;
    if (verb < 0) { emit(K_INFO, I_ISSUE, (u32)frame, idx | ((u32)(5) << 16), before, before, 0); return; }
    i32 who, x = 0, y = 0, type = 0, ox = 0, whom = 0, ids[ISSUE_MAX];
    u32 n = 0;
    /* `@spell`'s craft and target come before its point;
     * `@settransport` has one number, the flag, which rides in `x`, and
     * `@buildmask` one, the mask, likewise, and `@unqueue` one, the
     * selector; `@queueup`'s type and count ride in `x` and `y`;
     * `@gatherpoint`'s action rides in `type`, after its point. */
    if (!issue_int(&t, &who) ||
        (verb == 14 && (!issue_int(&t, &type) || !issue_int(&t, &ox) || !issue_int(&t, &whom))) ||
        ((verb == 15 || verb == 17 || verb == 19) && !issue_int(&t, &x)) ||
        (verb != 5 && verb != 15 && verb != 17 && verb != 19 &&
         (!issue_int(&t, &x) || !issue_int(&t, &y))) ||
        ((verb == 13 || verb == 20) && !issue_int(&t, &type))) {
        emit(K_INFO, I_ISSUE, (u32)frame, idx | ((u32)(5) << 16), before, before, 0);
        return;
    }
    while (n < ISSUE_MAX && issue_int(&t, &ids[n])) n++;
    if (!n || who < 0 || who > 7) { emit(K_INFO, I_ISSUE, (u32)frame, idx | ((u32)(5) << 16), before, before, n); return; }
    u8 *console = *(u8 **)(g_base + RVA_CONSOLE);
    if (!console || *(i32 *)(console + CONSOLE_PLAY_OFF) != who) {
        emit(K_INFO, I_ISSUE, (u32)frame, idx | ((u32)(1) << 16), before, before, n);
        return;
    }
    /* 8, 9 and 10 are issue_move_to with another `orders` byte. */
    i32 move_kind = verb == 8 ? 2 : verb == 9 ? 3 : verb == 10 ? 4 : 1;
    u32 rva = ISSUER_RVA[verb];
    u32 size = verb >= 21  ? 0x19
               : verb == 20  ? 0x11
               : verb == 19  ? 0x0f
               : verb >= 17  ? 0x09
               : verb == 16  ? 0x11
               : verb == 15  ? 0x05
               : verb == 14  ? 0x15
               : verb >= 11 ? 0x19
               : verb >= 8 ? 0x16
               : verb == 7 ? 0x11
               : verb == 5 ? 0x11
               : verb >= 2 ? 0x0d
               : verb      ? 0x0a
                           : 0x16; /* 6 is 0x0d */
    /* Under `cover=1` the issuer's entry is `arm_all`'s jmp to its stub,
     * and the guard reads the displaced bytes from the table's copy
     * (`issue_guard.h`). */
    const Entry *cov = g_stubs ? cover_entry(rva) : 0;
    u32 stub = cov ? (u32)(g_stubs + STUB_BYTES * (u32)(cov - g_funcs)) : 0;
    if (!issuer_is_shipped((const u8 *)(g_base + rva), g_base + rva, ISSUER_PROLOGUE_BYTES[verb], ISSUE_PROLOGUE,
                           stub, cov ? cov->code : 0, cov ? cov->orig_len : 0, cov ? cov->code_len : 0,
                           cov ? cov->nfix : 0)) {
        emit(K_INFO, I_ISSUE, (u32)frame, idx | ((u32)(2) << 16), before, before, n);
        return;
    }
    /* A unit verb names live captains, from `units`; `@eject`,
     * `@buildmask`, `@queueup`, `@unqueue` and `@gatherpoint` name live
     * buildings, from `objects`, on `Build::vftable`. */
    i32 on_buildings = verb == 5 || verb >= 17;
    u8 *objects = on_buildings ? *(u8 **)(g_base + RVA_OBJECTS) : 0;
    u8 *band = on_buildings ? (objects ? objects + 4 + (u32)who * 0x1c : 0)
                            : (u8 *)(g_base + RVA_UNITS + (u32)who * 0x1c);
    u32 length = band ? *(u32 *)(band + 4) : 0;
    u8 **slots = band ? *(u8 ***)(band + 0x10) : 0;
    for (u32 j = 0; j < n; j++) {
        u8 *unit = (ids[j] >= 0 && (u32)ids[j] < length && slots) ? slots[ids[j]] : 0;
        if (!unit || !(unit[8] & 1) || unit[9] != (u8)who || *(u16 *)(unit + 0xa) != (u16)ids[j] ||
            (on_buildings ? *(u32 *)unit != g_base + RVA_BUILD_VFTABLE : !(*(u16 *)(unit + 0x8e) & 0x8000))) {
            emit(K_INFO, I_ISSUE, (u32)frame, idx | ((u32)(3) << 16), before, (u32)ids[j], n);
            return;
        }
    }
    /* A fresh group (3 + 2n), the command, and two bytes of slack each;
     * `@unqueue` is a command a building and no group. */
    if ((verb == 19 ? before + size * n + 4 : before + 3 + 2 * n + size + 4) > 0x200) {
        emit(K_INFO, I_ISSUE, (u32)frame, idx | ((u32)(4) << 16), before, before, n);
        return;
    }
    for (u32 i = 0; i < sizeof g_groupout; i++) g_groupout[i] = 0;
    *(i32 *)(g_groupout + 0xc) = (i32)n;
    g_groupout[0x4a] = (u8)who;
    for (u32 j = 0; j < n; j++) {
        *(u16 *)(g_groupout + 0x8cc + 2 * j) = (u16)ids[j];
        u8 *unit = slots[ids[j]];
        emit(K_INFO, I_ISSUE_UNIT, (u32)frame, (u32)ids[j] | (u32)who << 16, *(u16 *)(unit + 0x30),
             *(u32 *)(unit + 0x10) ^ 0x63637u, *(u32 *)(unit + 0x14) ^ 0x63637u);
    }
    if (verb >= 21) {
        /* issue_launch_patrol(group, x, y, queue, shift, ctrl, alt) —
         * QUEUE_NEW for `@launchpatrol`, QUEUE_LAST and shift for
         * `@launchpatrolall` — or issue_flight(group, ox, whom, ATTACK, 0,
         * ctrl, alt) for `@launchstrike`, on a group of buildings (item
         * 976); ctrl or alt 1 for the `ctrl`/`alt` spellings, and MOVE_TO
         * for `@launchmove` (item 1009). */
        typedef void(__thiscall *launch_fn)(void *, void *, i32, i32, i32, i32, i32, i32);
        ((launch_fn)(g_base + rva))((void *)(g_base + RVA_COMMAND_MANAGER), g_groupout, x, y,
                                    verb == 23 ? (launch_move ? 1 : 10) : verb == 22 ? 1 : 2, verb == 22,
                                    launch_ctrl, launch_alt);
    } else if (verb == 20) {
        /* issue_gather_point(group, x, y, action, add_to_end 0): a
         * right-click with a selection of buildings, or the Clear
         * button's −1, −1, 0, 0. */
        typedef void(__thiscall *gather_point_fn)(void *, void *, i32, i32, i32, i32);
        ((gather_point_fn)(g_base + rva))((void *)(g_base + RVA_COMMAND_MANAGER), g_groupout, x, y,
                                          type, gather_add);
    } else if (verb == 19) {
        /* issue_unqueue(b, p) once per building: the cancel on a
         * selection of buildings, no group. */
        typedef void(__thiscall *unqueue_fn)(void *, void *, i32);
        for (u32 j = 0; j < n; j++)
            ((unqueue_fn)(g_base + rva))((void *)(g_base + RVA_COMMAND_MANAGER), slots[ids[j]], x);
    } else if (verb == 18) {
        /* issue_queue_up(group, type, num): a unit's button on a selection
         * of buildings. */
        typedef void(__thiscall *queue_up_fn)(void *, void *, i32, i32);
        ((queue_up_fn)(g_base + rva))((void *)(g_base + RVA_COMMAND_MANAGER), g_groupout, x, y);
    } else if (verb == 17) {
        /* issue_buildmask(group, mask, set 1): the repeat button's 0x80 on
         * a selection of buildings. */
        typedef void(__thiscall *buildmask_fn)(void *, void *, i32, i32);
        ((buildmask_fn)(g_base + rva))((void *)(g_base + RVA_COMMAND_MANAGER), g_groupout, x, 1);
    } else if (verb == 16) {
        /* issue_swarm_around(group, ox, whom, QUEUE_NEW 2, REPAIR 13): a
         * right-click on a damaged friendly building. */
        typedef void(__thiscall *swarm_fn)(void *, void *, i32, i32, i32, i32);
        ((swarm_fn)(g_base + rva))((void *)(g_base + RVA_COMMAND_MANAGER), g_groupout, x, y, 2, 13);
    } else if (verb == 15) {
        /* issue_set_transport(group, flag): the transport button's
         * toggle, or OPTION_DISEMBARK's 1. */
        typedef void(__thiscall *set_transport_fn)(void *, void *, i32);
        ((set_transport_fn)(g_base + rva))((void *)(g_base + RVA_COMMAND_MANAGER), g_groupout, x);
    } else if (verb == 14) {
        /* issue_spell(group, type, ox, whom, x, y): a targeted craft's
         * unmodified pick. */
        typedef void(__thiscall *spell_fn)(void *, void *, i32, i32, i32, i32, i32);
        ((spell_fn)(g_base + rva))((void *)(g_base + RVA_COMMAND_MANAGER), g_groupout, type, ox, whom,
                                   x, y);
    } else if (verb == 13) {
        /* issue_build(group, x, y, x2 = x, y2 = y, type, QUEUE_NEW 2):
         * an unmodified drop with no drag. */
        typedef void(__thiscall *build_fn)(void *, void *, i32, i32, i32, i32, i32, i32);
        ((build_fn)(g_base + rva))((void *)(g_base + RVA_COMMAND_MANAGER), g_groupout, x, y, x, y,
                                   type, 2);
    } else if (verb >= 11) {
        /* issue_flight(group, ox, whom, orders, shift 0, ctrl 0, alt 0):
         * MOVE_TO (1) for `@flight`, ATTACK (10) for `@strike`. */
        typedef void(__thiscall *flight_fn)(void *, void *, i32, i32, i32, i32, i32, i32);
        ((flight_fn)(g_base + rva))((void *)(g_base + RVA_COMMAND_MANAGER), g_groupout, x, y,
                                    verb == 12 ? 10 : 1, 0, 0, 0);
    } else if (verb == 5) {
        /* issue_eject_all(group, back_to_work 0, who -1, eject_o -1,
         * eject_who -1): the Eject button's bytes. */
        typedef void(__thiscall *eject_fn)(void *, void *, i32, i32, i32, i32);
        ((eject_fn)(g_base + rva))((void *)(g_base + RVA_COMMAND_MANAGER), g_groupout, 0, -1, -1, -1);
    } else if (verb == 7) {
        /* issue_attack(group, ox, whom, ignore 0, QUEUE_NEW 2): a
         * right-click on an enemy. */
        typedef void(__thiscall *attack_fn)(void *, void *, i32, i32, i32, i32);
        ((attack_fn)(g_base + rva))((void *)(g_base + RVA_COMMAND_MANAGER), g_groupout, x, y, 0, 2);
    } else if (verb && verb < 8) {
        /* issue_patrol(group, x, y, queue), issue_guard(group, ox, whom,
         * queue), issue_follow(group, ox, whom, queue) and
         * issue_garrison(group, ox, whom, queue) share one shape: two ints
         * and QUEUE_NEW. So does issue_form(group, form, rotate, queue). */
        typedef void(__thiscall *patrol_fn)(void *, void *, i32, i32, i32);
        ((patrol_fn)(g_base + rva))((void *)(g_base + RVA_COMMAND_MANAGER), g_groupout, x, y, 2);
    } else {
        typedef void(__thiscall *issue_fn)(void *, void *, i32, i32, i32, i32, i32, i32, i32, i32, i32);
        /* `@move` passes MOVE_TO (1), `@amove` ATTACK_TO (2), `@explore`
         * EXPLORE_TO (3), `@flee` FLEE_TO (4). */
        ((issue_fn)(g_base + rva))((void *)(g_base + RVA_COMMAND_MANAGER), g_groupout, x, y, 2, 0, 0,
                                   move_kind, -1, -1, 0);
    }
    u32 after = *pkg_size;
    emit(K_INFO, I_ISSUE, (u32)frame, idx | ((u32)(after > before ? 0 : 6) << 16), before, after, n);
}

/* Run every line due at `frame`. Called at the top of Game::do_frame, before
 * the frame's phases — so a line sees the state at the end of the previous
 * frame, and its effects are in this frame's dump. */
static void run_cmds(i32 frame) {
    while (g_next_cmd < g_ncmds && g_cmds[g_next_cmd].frame <= frame) {
        Cmd *c = &g_cmds[g_next_cmd];
        u32 idx = g_next_cmd++;
        if (c->from_chat == 2) {
            issue_line(frame, idx, c->text);
            flush();
            continue;
        }
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
    if (g_dropped && g_nbuf < BUF_RECS) { /* I_DROPPED: a = records lost */
        u32 *r = g_buf + g_nbuf * 8;
        r[0] = K_INFO;
        r[1] = I_DROPPED;
        r[2] = g_dropped;
        r[3] = r[4] = r[5] = r[6] = 0;
        r[7] = (u32)g_frame;
        g_nbuf++;
        g_dropped = 0;
    }
    if (g_log != INVALID_HANDLE && g_nbuf)
        WriteFile(g_log, g_buf, g_nbuf * 32, &written, 0);
    g_nbuf = 0;
}

static void emit(u32 k, u32 a, u32 b, u32 c, u32 d, u32 e, u32 f) {
    lock();
    if (g_nbuf >= BUF_RECS) {
        g_dropped++;
    } else {
        u32 *r = g_buf + g_nbuf * 8;
        r[0] = k;
        r[1] = a;
        r[2] = b;
        r[3] = c;
        r[4] = d;
        r[5] = e;
        r[6] = f;
        r[7] = (u32)g_frame;
        g_nbuf++;
    }
    /* `>=`, not `==`: the coverage stubs add records without flushing, so
     * the count can pass FLUSH_RECS between two emits from here. */
    if (g_nbuf >= FLUSH_RECS) flush_locked();
    unlock();
}

static void flush(void) {
    lock();
    flush_locked();
    unlock();
}

/* A record from a coverage stub: no flush, ever. A stub runs inside the
 * game's own call, on every thread the game has, and a syscall from there
 * was one of the shapes that died in the probes behind "Coverage is back"
 * (`docs/ORACLE.md`); the flusher and the frame hook write the log. A full
 * buffer drops the record and counts it. */
static void emit_nosys(u32 k, u32 a, u32 b) {
    lock();
    if (g_nbuf < BUF_RECS) {
        u32 *r = g_buf + g_nbuf * 8;
        r[0] = k;
        r[1] = a;
        r[2] = b;
        r[3] = r[4] = r[5] = r[6] = 0;
        r[7] = (u32)g_frame;
        g_nbuf++;
    } else {
        g_dropped++;
    }
    unlock();
}

/* The flusher: a thread of the instrument's own that writes the buffer out
 * every 20 ms. The coverage stubs never flush (`emit_nosys`) and the frame
 * hook flushes once a frame, so without this a startup's records would sit
 * in memory until the first frame, and a death before it would lose them
 * all — which is how the eleven probes behind "Coverage is back" were read. */
static u32 WINAPI flusher(void *arg) {
    (void)arg;
    for (;;) {
        Sleep(20);
        flush();
    }
}

/* ---- coverage ---------------------------------------------------------- */

/* A hooked or proxied entry carries its own `jmp` already, and the copy a
 * coverage stub runs would be that jmp. A proxied function therefore has no
 * HIT record — its CALL/RET records are the stronger evidence anyway. */
static int is_hook_site(u32 rva) {
    for (u32 i = 0; i < NHOOKS; i++)
        if (rva >= HOOKS[i].rva && rva < HOOKS[i].rva + HOOKS[i].len) return 1;
    if (g_calls)
        for (u32 i = 0; i < NCALLS; i++)
            if (rva >= CALLS[i].rva && rva < CALLS[i].rva + CALLS[i].len) return 1;
    return 0;
}

/* `lock cmpxchg8b` n bytes at offset `at` of the 8-byte word `w`. */
static void cas_word(u8 *w, u32 at, const u8 *bytes, u32 n) {
    for (;;) {
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

/* Write five bytes over code with locked writes: one `cmpxchg8b` where
 * [p, p+5) sits inside an 8-byte word, two where it straddles. Only ever
 * called from `arm_all` at attach, when the process has one thread. */
static void patch5(u8 *p, const u8 *five) {
    u32 at = (u32)p & 7;
    u8 *w = p - at;
    u32 first = 8 - at;
    if (first > 5) first = 5;
    cas_word(w, at, five, first);
    if (first < 5) cas_word(w + 8, 0, five + first, 5 - first);
}

/* A coverage stub's slow path: the first entry of the function this run,
 * and inside the window the first entry per frame. Two threads can arrive
 * together; the worst case is one duplicate record, which the reader folds.
 * No syscall is made here — none is needed, and none could be: the log is
 * written by the flusher and the frame hook. */
static void __cdecl on_cover(u32 i) {
    if (!g_hit[i]) {
        g_hit[i] = 1;
        g_hit_frame[i] = g_frame;
        emit_nosys(K_HIT, g_base + g_funcs[i].rva, rd_fs(0x24));
    } else if (g_in_window && g_hit_frame[i] != g_frame) {
        g_hit_frame[i] = g_frame;
        emit_nosys(K_HIT, g_base + g_funcs[i].rva, rd_fs(0x24));
    }
    if (!g_in_window) g_fast[i] = 1;
}

/*
 * The stub, one per table entry, in a region written as data and then made
 * executable. **No `pushad`/`popad` and no `pushfd`/`popfd`**: a thread
 * running `popad` — reliably — or `popfd` — now and then — while another
 * thread is mid-syscall breaks that thread's 32->64 switch on this machine
 * (`wow64bop.c`'s one-instruction loops, `docs/ORACLE.md`, "Coverage is
 * back"); plain push/pop, lahf/sahf and calls are clean. So the flags go
 * through `lahf`/`seto` and come back through `add al,0x7f`/`sahf`, and
 * only the registers the cdecl callee may clobber are saved:
 *
 *   50                     push eax
 *   9F                     lahf                   ; SF ZF AF PF CF -> ah
 *   0F 90 C0               seto al                ; OF -> al
 *   80 3D ff ff ff ff 00   cmp byte ptr [g_fast+i], 0
 *   75 rr                  jne done               ; recorded, no window open
 *   51 52                  push ecx; push edx
 *   50                     push eax               ; the saved flags, across the call
 *   68 ii ii ii ii         push i
 *   B8 hh hh hh hh         mov eax, on_cover
 *   FF D0                  call eax
 *   83 C4 04               add esp, 4
 *   58                     pop eax                ; the saved flags
 *   5A 59                  pop edx; pop ecx
 *   done:
 *   04 7F                  add al, 0x7f           ; OF <- (al == 1)
 *   9E                     sahf                   ; the rest <- ah
 *   58                     pop eax
 *   <code_len bytes>       the displaced prologue, rel32 slots fixed up
 *   E9 bb bb bb bb         jmp rva + orig_len
 *
 * The registers, the flags and the stack are exactly the caller's when the
 * copy runs — on both paths, since a label reached by a jump can be
 * carrying live flags — and the copy runs the original's own instructions.
 */
static u32 build_cover_stub(u8 *st, u32 i) {
    const Entry *e = &g_funcs[i];
    u32 n = 0;
    static const u8 head[] = {0x50, 0x9F, 0x0F, 0x90, 0xC0, 0x80, 0x3D};
    memcpy(st + n, head, sizeof head);
    n += sizeof head;
    *(u32 *)(st + n) = (u32)&g_fast[i];
    n += 4;
    st[n++] = 0x00;
    st[n++] = 0x75; /* jne done */
    u32 jne_at = n++;
    static const u8 save[] = {0x51, 0x52, 0x50, 0x68};
    memcpy(st + n, save, sizeof save);
    n += sizeof save;
    *(u32 *)(st + n) = i;
    n += 4;
    st[n++] = 0xB8; /* mov eax, on_cover */
    *(u32 *)(st + n) = (u32)(void *)on_cover;
    n += 4;
    static const u8 call[] = {0xFF, 0xD0, 0x83, 0xC4, 0x04, 0x58, 0x5A, 0x59};
    memcpy(st + n, call, sizeof call);
    n += sizeof call;
    st[jne_at] = (u8)(n - (jne_at + 1));
    static const u8 done[] = {0x04, 0x7F, 0x9E, 0x58};
    memcpy(st + n, done, sizeof done);
    n += sizeof done;
    u32 copy_at = n;
    memcpy(st + n, e->code, e->code_len);
    n += e->code_len;
    for (u32 k = 0; k < e->nfix; k++) {
        u8 *slot = st + copy_at + e->fix_off[k];
        *(u32 *)slot = e->fix_target[k] - ((u32)slot + 4);
    }
    st[n++] = 0xE9;
    u32 back = g_base + e->rva + e->orig_len;
    *(u32 *)(st + n) = back - ((u32)(st + n) + 4);
    n += 4;
    return n;
}

/* Build every stub, then plant the jmps — all of it at attach, before the
 * executable's entry point, when this is the only thread. */
static u32 arm_all(void) {
    u32 excluded = 0;
    for (u32 i = 0; i < g_nfuncs; i++) {
        Entry *e = &g_funcs[i];
        u32 off = e->rva - TEXT_RVA;
        if (e->orig_len && (off >= TEXT_SIZE || off + e->orig_len > TEXT_SIZE || is_hook_site(e->rva)))
            e->orig_len = 0;
        if (!e->orig_len) excluded++;
    }
    u32 size = (STUB_BYTES * g_nfuncs + 4095) & ~4095u;
    g_stubs = (u8 *)VirtualAlloc(0, size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    if (!g_stubs) return 0;
    for (u32 i = 0; i < g_nfuncs; i++)
        if (g_funcs[i].orig_len) build_cover_stub(g_stubs + STUB_BYTES * i, i);
    u32 old;
    if (!VirtualProtect(g_stubs, size, PAGE_EXECUTE_READ, &old)) {
        emit(K_INFO, I_PROTECT_FAIL, 1, 0, 0, 0, 0);
        g_stubs = 0;
        return 0;
    }
    /* The copies are taken from the table, not the live bytes, so the
     * order of the writes does not matter: an entry inside another's
     * displaced range still reads as the original did. */
    u32 n = 0;
    for (u32 i = 0; i < g_nfuncs; i++) {
        if (!g_funcs[i].orig_len) continue;
        u8 *p = (u8 *)(g_base + g_funcs[i].rva);
        u8 j[5];
        j[0] = 0xE9;
        u8 *st = g_stubs + STUB_BYTES * i;
        *(u32 *)(j + 1) = (u32)st - ((u32)p + 5);
        patch5(p, j);
        n++;
    }
    g_nstubs = n;
    FlushInstructionCache(g_proc, (void *)(g_base + TEXT_RVA), TEXT_SIZE);
    emit(K_INFO, I_COVER, (u32)g_stubs, n, excluded, 0, 0);
    return n;
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

#ifdef RON_COMMAND_PROBE
#ifdef RON_CAPSULE_PROBE
static void capsule_request_copy(u8 *package);
#endif
#include "../explore/live_move_probe.h"
#endif

#ifdef RON_CONGESTION_PROBE
#include "../explore/live_congestion_probe.h"
#endif

static void __cdecl on_hook(u32 kind, u32 ecx, u32 ebp, u32 caller, u32 arg0) {
    if (kind == K_FRAME) {
        i32 frame = *(i32 *)(ecx + GAME_FRAME_OFF);
        g_frame = frame;
        u32 seed = *(u32 *)(g_base + RVA_GAME_RANDOM);
        u32 armed = 0;
        if (g_cover && g_stubs) {
            i32 in = frame >= g_win_lo && frame <= g_win_hi;
            if (in && !g_in_window) memset(g_fast, 0, g_nfuncs); /* every stub records again */
            g_in_window = in;
            if (in) armed = g_nstubs; /* the stubs recording this frame */
        }
        emit(K_FRAME, (u32)frame, seed, armed, caller, 0, 0);
        flush();
#ifdef RON_HIDE_SCENE
        /* Experimental unsupported display mode skips both Scene::render
         * branches in Game::loop_render while retaining its service work.
         * Restore before the scheduled quit; this is not startup-headless. */
        static u8 saved_display;
        u8 *scene = *(u8 **)(g_base + 0x80620cu);
        if (frame == 18 && scene) {
            saved_display = scene[0x218];
            scene[0x218] = 3;
            emit(K_INFO, 120, (u32)frame, saved_display, 3, 0, 0);
        }
        if (frame == 35 && scene) {
            scene[0x218] = saved_display;
            emit(K_INFO, 120, (u32)frame, 3, saved_display, 0, 0);
        }
#endif
        run_cmds(frame);
#ifdef RON_CONGESTION_PROBE
        probe_congestion(frame);
#endif
#ifdef RON_COMMAND_PROBE
        probe_move(frame);
#endif
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

#include "hook_stub.h"
#ifdef RON_STATE_FRAME
#include "../explore/register_image_stub.h"
#include "../explore/live_frame_snapshot.h"
#include "../explore/frame_snapshot_stub.h"
#endif

static u32 build_stub(u8 *s, const HookSite *h) {
#ifdef RON_STATE_FRAME
    if (h->kind == 9 || h->kind == 10) {
        return build_frame_snapshot_stub(s,(u32)s,g_base+h->rva,
            (u32)(void *)(h->kind==9?frame_snapshot_enter:frame_snapshot_after),(const u8 *)(g_base+h->rva),h->len);
    }
#endif
    return build_hook_stub(s, (u32)s, g_base+h->rva, h->kind, (u32)(void *)on_hook,
                           (const u8 *)(g_base+h->rva), h->len);
}

/* ---- the call proxies -------------------------------------------------- */

#ifdef RON_TURN_PROBE
#include "../explore/live_turn_probe.h"
#endif

#ifdef RON_SEARCH_CENSUS
#include "../explore/live_search_census.h"
#endif
#ifdef RON_RESTORE_PROBE
#include "../explore/live_restore_probe.h"
#endif

#if defined(RON_COLLIDE_PROBE) || defined(RON_GUARD_PROBE)
/*
 * Name the object behind a `this`. Three of the collision sites are
 * `__thiscall` on a `UnitData *` whose own pair is `+0xa` (o, a short) and
 * `+0x9` (who, a byte) - the two `will_be_corner@00609fa0` itself indexes
 * `units[who][o]` with, so they are the record's own definition of the
 * fields and not a guess. One INFO record per call turns a log full of
 * heap addresses into one that reads in o/who, which is what the dump
 * beside it is keyed on.
 */
static void collide_name(u32 site, u32 self) {
    if (self < 0x10000u) return;
    emit(K_INFO, I_UNITID, site, self, (u32) * (u16 *)(self + 0xa), (u32) * (u8 *)(self + 9), 0);
}
#endif

#ifdef RON_COLLIDE_PROBE
/*
 * The occupancy block a probe reads, as the probe is entered (item 566).
 * `collide_here(o, who, ucx, ucy, …)` reads the bitmask of the world cells
 * its disc reaches; this prints the **centre's** world cell, twice where
 * it can: the live `WData::block` (`World +0x134`, stride 0x1c, `+0x18`),
 * and the copy a `nocoll` probe reads instead when the pathfinder's tree
 * (`pathfinder +0x4c`, keyed `cy * xs + cx`) holds one
 * (`CollCheck::fill_slots@006820e0`). The copy outlives the probe that
 * took it until `PathFinder::kill_lists` runs, so the two can differ, and
 * which one a refusal's bit is in is the question these records answer.
 * Read-only: two pointer chains and a tree walk, the same one
 * `Tree<CollBlock*,int>::seek@00479220` takes. The globals are the PDB's
 * `GameAccess::world` (0xc06188) and `pathfinder` (0xe85e40), both
 * confirmed by `fill_slots`' own listing (`006821c3`, `00682275`).
 */
static void collide_block_emit(u32 hdr, const u8 *bits) {
    const u32 *w = (const u32 *)bits;
    emit(K_INFO, I_COLLBLOCK, hdr, w[0], w[1], w[2], w[3]);
    emit(K_INFO, I_COLLBLOCK, hdr | (1u << 24), w[4], w[5], w[6], w[7]);
}

static void collide_blocks(u32 ucx, u32 ucy) {
    const u8 *world = *(const u8 *const *)(g_base + (0xc06188u - 0x400000u));
    if ((u32)world < 0x10000u) return;
    i32 xs = *(const i32 *)world, ys = *(const i32 *)(world + 4);
    i32 cx = (i32)ucx >> 4, cy = (i32)ucy >> 4;
    if (cx < 0 || cy < 0 || cx >= xs || cy >= ys || cx > 255 || cy > 255) return;
    u32 hdr = (u32)cx | ((u32)cy << 8);
    const u8 *wdata = *(const u8 *const *)(world + 0x134);
    u32 block = *(const u32 *)(wdata + (u32)(cy * xs + cx) * 0x1c + 0x18);
    if (block < 0x10000u || block == 0xffffffffu)
        emit(K_INFO, I_COLLBLOCK, hdr | (2u << 16), block, 0, 0, 0);
    else
        collide_block_emit(hdr, (const u8 *)block + 0xc);
    const u8 *tree = *(const u8 *const *)(g_base + (0xe85e8cu - 0x400000u));
    if ((u32)tree < 0x10000u) return;
    const u8 *node = *(const u8 *const *)(tree + 0xc);
    i32 key = cy * xs + cx;
    while ((u32)node >= 0x10000u) {
        i32 m = *(const i32 *)(node + 0x10);
        if (key < m) {
            node = *(const u8 *const *)node;
        } else if (key <= m) {
            u32 copy = *(const u32 *)(node + 0xc);
            if (copy >= 0x10000u) collide_block_emit(hdr | (1u << 16), (const u8 *)copy + 0xc);
            return;
        } else {
            node = *(const u8 *const *)(node + 4);
        }
    }
}
#endif

static void __cdecl on_call(u32 site, u32 self, u32 a0, u32 a1, u32 a2, u32 a3) {
    if (g_frame < g_cw_lo || g_frame > g_cw_hi) return;
    emit(K_CALL, site, self, a0, a1, a2, a3);
#ifdef RON_COLLIDE_PROBE
    /* 8 detect_unit_collision (the asker), 11 is_here and 12 is_corner
     * (the candidates). 9 and 10 are the asker again and carry no new
     * pointer. */
    if (site == 8 || site == 11 || site == 12) collide_name(site, self);
    /* 9 collide_here(o, who, ucx, ucy, ...): the block it is about to read. */
    if (site == 9) collide_blocks(a2, a3);
#endif
#ifdef RON_GUARD_PROBE
    /* Every guard-probe site is `__thiscall` on a unit. */
    if (site >= 8 && site <= 13) collide_name(site, self);
#endif
#ifdef RON_TURN_PROBE
    if (site == 8) probe_turn(self);
#endif
}

static void __cdecl on_ret(u32 site, u32 ret, u32 a4, u32 a5, u32 a6, u32 a7) {
    if (g_frame < g_cw_lo || g_frame > g_cw_hi) return;
    u32 out = 0xffffffffu;
    if (site < NCALLS && CALLS[site].out7 && a7 > 0x10000u) out = *(u8 *)a7;
    emit(K_RET, site, ret, a4, a5, a6, out);
#ifdef RON_SEARCH_CENSUS
    if (site == 0 && ret == 0xffffffffu) census_suspension();
#endif
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
#ifdef RON_SEARCH_CENSUS
    if (g_base != 0x400000u || g_cover) {
        emit(K_INFO, 136, 1, g_base, (u32)g_cover, 0, 0);
        return;
    }
    emit(K_INFO, 136, 0, 1, 64, 0, 0);
#ifdef RON_RESTORE_PROBE
    install_restore_probe();
#endif
#endif
    u8 *page = (u8 *)VirtualAlloc(0, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    if (!page) return;
    u32 used = 0;
    for (u32 i = 0; i < NCALLS; i++) {
#ifdef RON_SEARCH_CENSUS
        if (i != 0) continue; /* No per-node cost proxies in the census lane. */
#endif
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
        emit(K_INFO, I_PROXIED, h->rva, (u32)stub, h->nargs, i, 0);
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
        } else if (*p == '@') {
            c->from_chat = 2;
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

#ifdef RON_CAPSULE_PROBE
#include "../explore/live_capsule_probe.h"
#endif

#ifdef RON_ORDER_CAPSULE
#include "../explore/live_order_capsule.h"
#endif

#ifdef RON_PATH_CAPSULE
#include "../explore/live_path_capsule.h"
#endif

#ifdef RON_AUTOSTART
#include "../explore/live_autostart.h"
#endif

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
        g_nfuncs = read_file("rontrace.funcs", g_funcs, sizeof g_funcs) / sizeof(Entry);

        /* header: magic, version (2 = the displaced-prologue table), base,
         * .text, nfuncs, window */
        emit(0x544E4F52u, 2, g_base, TEXT_RVA, TEXT_SIZE, g_nfuncs, (u32)g_win_lo);
        g_buf[7] = (u32)g_win_hi;

        u32 old;
        if (!VirtualProtect((void *)(g_base + TEXT_RVA), TEXT_SIZE, PAGE_EXECUTE_READWRITE, &old)) {
            emit(K_INFO, I_PROTECT_FAIL, 0, 0, 0, 0, 0);
            flush();
            return 1;
        }
        install_hooks();
#ifdef RON_AUTOSTART
        install_autostart();
#endif
        if (g_cw_hi >= g_cw_lo) {
            g_calls = 1;
            install_calls();
        }
#ifdef RON_CAPSULE_PROBE
        install_capsule();
#endif
#ifdef RON_ORDER_CAPSULE
        install_order_capsule();
#endif
#ifdef RON_PATH_CAPSULE
        install_path_capsule();
#endif
        if (g_cover) {
            if (!g_nfuncs) emit(K_INFO, I_NOFUNCS, 0, 0, 0, 0, 0);
            u32 armed = arm_all();
            emit(K_INFO, I_ARMED, (u32)-1, armed, 0, 0, 0);
            CreateThread(0, 0, (void *)flusher, 0, 0, 0);
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
