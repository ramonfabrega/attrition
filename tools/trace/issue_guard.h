/* The `@` issuers: which `CommandManager::issue_*` each verb calls, the
 * prologue the shipped build has there, and the guard that checks it before
 * the call. Included by `tracer.c`, and by `issue_guard_test.c` on the host
 * (`rondata::blind`'s `the_issuer_guard_reads_through_the_coverage_jmp`), so
 * the guard is tested as the DLL compiles it. Needs `u8` and `u32`.
 *
 * Verbs: 0 `move`, 1 `patrol`, 2 `guard`, 3 `follow`, 4 `garrison`, 5
 * `eject`, 6 `form`, 7 `attack`, 8 `amove`, 9 `explore`, 10 `flee`, 11
 * `flight`, 12 `strike`, 13 `build`, 14 `spell`, 15 `settransport`, 16
 * `repair`, 17 `buildmask`, 18 `queueup`, 19 `unqueue`, 20 `gatherpoint`,
 * 21 `launchpatrol`, 22 `launchpatrolall`, 23 `launchstrike`.
 * 8, 9 and 10 are issue_move_to with another `orders` byte; 12 is
 * issue_flight with another; 16 is issue_swarm_around; 22 is
 * issue_launch_patrol with another queue and shift; 23 is issue_flight on a
 * group of buildings (item 976). `launchpatrolctrl`/`launchpatrolalt` are
 * 21, `launchstrikectrl`/`launchstrikealt` and `launchmove` 23, with ctrl,
 * alt or MOVE_TO in the call and the same issuer (item 1009). 24 `alarm`
 * and 25 `gather` (item 1167). */

#define ISSUE_VERBS 26
#define ISSUE_PROLOGUE 11

#define RVA_ISSUE_MOVE_TO 0x541720u
#define RVA_ISSUE_PATROL 0x541800u
#define RVA_ISSUE_GUARD 0x541ed0u
#define RVA_ISSUE_FOLLOW 0x541e70u
#define RVA_ISSUE_GARRISON 0x541a70u
#define RVA_ISSUE_EJECT_ALL 0x541ca0u
#define RVA_ISSUE_FORM 0x541580u
#define RVA_ISSUE_ATTACK 0x5415e0u
#define RVA_ISSUE_FLIGHT 0x541d40u
#define RVA_ISSUE_BUILD 0x541c30u
#define RVA_ISSUE_SPELL 0x541b80u
#define RVA_ISSUE_SET_TRANSPORT 0x541910u
#define RVA_ISSUE_SWARM_AROUND 0x5416b0u
#define RVA_ISSUE_BUILDMASK 0x541f80u
#define RVA_ISSUE_QUEUE_UP 0x541be0u
#define RVA_ISSUE_UNQUEUE 0x542c40u /* the WallOut overload, VA 0x942c40 */
#define RVA_ISSUE_GATHER_POINT 0x541b20u
#define RVA_ISSUE_LAUNCH_PATROL 0x541860u
#define RVA_ISSUE_ALARM 0x541d00u
#define RVA_ISSUE_GATHER 0x541a20u

static const u32 ISSUER_RVA[ISSUE_VERBS] = {
    RVA_ISSUE_MOVE_TO,   RVA_ISSUE_PATROL,        RVA_ISSUE_GUARD,        RVA_ISSUE_FOLLOW,
    RVA_ISSUE_GARRISON,  RVA_ISSUE_EJECT_ALL,     RVA_ISSUE_FORM,         RVA_ISSUE_ATTACK,
    RVA_ISSUE_MOVE_TO,   RVA_ISSUE_MOVE_TO,       RVA_ISSUE_MOVE_TO,      RVA_ISSUE_FLIGHT,
    RVA_ISSUE_FLIGHT,    RVA_ISSUE_BUILD,         RVA_ISSUE_SPELL,        RVA_ISSUE_SET_TRANSPORT,
    RVA_ISSUE_SWARM_AROUND, RVA_ISSUE_BUILDMASK,  RVA_ISSUE_QUEUE_UP,     RVA_ISSUE_UNQUEUE,
    RVA_ISSUE_GATHER_POINT, RVA_ISSUE_LAUNCH_PATROL, RVA_ISSUE_LAUNCH_PATROL, RVA_ISSUE_FLIGHT,
    RVA_ISSUE_ALARM,     RVA_ISSUE_GATHER};

/* `sub esp, 0x18` for issue_move_to's 0x1c-byte command, `0x10` for
 * issue_patrol's, issue_guard's, issue_follow's, issue_garrison's and
 * issue_form's, `0x14` for issue_eject_all's; each then loads
 * `&command_manager` into ecx. issue_attack's `sub esp, 0x14` is
 * followed by `push esi; mov esi, [ebp+0xc]`: it tests `ox` and
 * `whom` before it asks `check_accept_issue`. `@amove`, `@explore` and
 * `@flee` are issue_move_to. issue_flight's is `sub esp, 0x1c`, for its
 * 0x19-byte command; `@flight` and `@strike` are issue_flight.
 * issue_build's is the same `sub esp, 0x1c`, for its 0x19-byte
 * command. issue_spell's is `sub esp, 0x18`, for its 0x15-byte one.
 * issue_set_transport's is `sub esp, 8`, for its 5-byte one.
 * issue_swarm_around's is issue_attack's to the byte: it tests `ox`
 * and `whom` first too, for its 0x11-byte command. issue_buildmask's
 * is `sub esp, 0xc`, for its 9-byte one, and so is issue_queue_up's.
 * issue_unqueue's is `sub esp, 0x10`, for its 15-byte one.
 * issue_gather_point's is `sub esp, 0x14`, for its 17-byte one.
 * issue_launch_patrol's is issue_flight's `sub esp, 0x1c`, for its
 * 0x19-byte command (item 976). issue_alarm's is `push ecx` for its
 * one-byte command (type 0x1b), then the `&command_manager` load and
 * the type's `movb`; issue_gather's is `sub esp, 0xc`, for its 9-byte
 * one (type 0x13, `[ox i32][queued i32]`) — item 1167, off the listing
 * (`941d00`, `941a20`). */
static const u8 ISSUER_PROLOGUE_BYTES[ISSUE_VERBS][ISSUE_PROLOGUE] = {
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x18, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x10, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x10, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x10, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x10, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x14, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x10, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x14, 0x56, 0x8b, 0x75, 0x0c, 0xc6},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x18, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x18, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x18, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x1c, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x1c, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x1c, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x18, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x08, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x14, 0x56, 0x8b, 0x75, 0x0c, 0xc6},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x0c, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x0c, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x10, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x14, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x1c, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x1c, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x1c, 0xb9, 0x60, 0xff, 0xe8, 0x00},
    {0x55, 0x8b, 0xec, 0x51, 0xb9, 0x60, 0xff, 0xe8, 0x00, 0xc6, 0x45},
    {0x55, 0x8b, 0xec, 0x83, 0xec, 0x0c, 0xb9, 0x60, 0xff, 0xe8, 0x00}};

/* Is the issuer at `va`, whose first bytes are `live`, the shipped build's
 * `want`? `stub` is the coverage stub `arm_all` planted over the entry, 0
 * when none was; `copy`, `orig_len`, `code_len` and `nfix` are that entry's
 * record in the coverage table.
 *
 * Under `cover=1` the first five live bytes are not the game's: `arm_all`
 * wrote `E9 rel32` to the entry's stub over them, and the stub runs a copy of
 * the `orig_len` bytes of whole instructions the jmp displaced (for every
 * issuer, `push ebp; mov ebp, esp; sub esp, N`: six). So the shipped bytes
 * are the table's copy below `orig_len` and the live bytes from there on
 * (item 934; run314 refused every issuer with code 2 by comparing the jmp).
 * The copy stands in only when the jmp lands on this entry's own stub and
 * the copy is the original's bytes verbatim — no branch rewritten — so a
 * foreign patch or another build is still refused. */
static int issuer_is_shipped(const u8 *live, u32 va, const u8 *want, u32 n, u32 stub, const u8 *copy,
                             u32 orig_len, u32 code_len, u32 nfix) {
    u32 from = 0;
    if (stub) {
        u32 rel = (u32)live[1] | (u32)live[2] << 8 | (u32)live[3] << 16 | (u32)live[4] << 24;
        if (orig_len < 5 || code_len != orig_len || nfix || live[0] != 0xE9 || rel != stub - (va + 5))
            return 0;
        from = orig_len < n ? orig_len : n;
        for (u32 i = 0; i < from; i++)
            if (copy[i] != want[i]) return 0;
    }
    for (u32 i = from; i < n; i++)
        if (live[i] != want[i]) return 0;
    return 1;
}
