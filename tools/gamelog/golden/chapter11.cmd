# Golden record, chapter eleven — the guard line, an issuer the AI never uses.
#
# docs/GOLDEN.md §19 (item 696; docs/DECISIONS.md 41 §1 and §5, 49). Chapter
# nine's harness, a third verb: each `@guard` line below is a call, from
# rontrace.dll, of the original's own `CommandManager::issue_guard@00941ed0`
# with an unmodified guard pick's arguments (`Options::picked_spot@00721c40`
# through `GroupOut::issue_guard@007088e0`, QUEUE_NEW), and the turn pump
# processes the command it appends as it processes a click (tools/trace/
# tracer.c, `issue_line`). The AI reaches `Group::action_guard` only for an
# army's wagon escort (docs/ORDERS.md §24), never from a command, and no dump
# on disk prints `process_guard`.
#
#   run190 (item 696):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch11 \
#       --map 14 --end-frame 1250 --log-window 605 1250 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter11.cmd
#
# ---------------------------------------------------------------------------
# THE ISSUER, as the emulator ran it before the pair (item 696, step 1).
#
# `issue_guard` under unicorn on command_oracle.py's fixture, widened to
# who=0's objects 6..10 as live captains, with these calls' own arguments
# `(group, ox, whom, QUEUE_NEW 2)`: each appends **18 bytes** — the 5-byte
# `group` (num 1, who 0, the one object) and a 13-byte `guard`, type 0x1f,
# `[ox i32][whom i32][queued i32]` — and writes the package's size and data
# and the selection caches `CommandPackage::last_who_sent` (00c06354),
# `last_num_sent` (00cae5fb), `last_objects_sent` (00cbfeb0) and
# `last_uids_sent` (00cbffb0). Nothing else: no unit, no order, no draw. The
# same selection again appends the 3-byte reuse and the guard (16); `queued`
# rides through as passed. `use_mp_playback`, `semaphore & 0x10` and
# `semaphore & 4` each append nothing, as for the move and the patrol.
#
# `Group::action_guard@006fcd30` entered under the same machine on the group
# [0/8] and a charge whose vtable is `Build::vftable@00b42174` (0/2001):
# `Group::clear`, `Group::action_begin`, one folded `return 0`, and it
# returns, having written only the group's `disband`. On `Unit::vftable` it
# goes on to `is_on_map`. **What the emulator cannot reach** is the unit
# half: `CommandPackage::process_guard@009478a0` -> `action_guard(g, ox,
# whom, queued, 0)` -> `Unit::add_guard_order@005e3e40`, and each frame's
# `Unit::do_guard@005e5c70`.
#
# ---------------------------------------------------------------------------
# THE READING (docs/GOLDEN.md §19; docs/ORDERS.md §7.5, §24.3-24.5).
#
# The charge's first test in `action_guard` is object vslot +0x8, then +0xbc
# (`006fce46`, `006fce56`). The PDB's `SubObjectData` method list names
# +0x8 `is_valid_unit` (the export's `SubObjectData::is_active` is the
# folded body's name); on `Build::vftable` the slot is `Window::get_button`,
# a folded `return 0`. **So a guard on a building gives no order at all.**
# The booking's second half — "a squad guarding a building should enter
# GuardOrder" — is kept as staged and predicted to enter nothing.
#
# For a unit charge: the charge becomes its captain; the escort (members
# active, on the map, not planes, of its domain; siege kept, the command's
# filter is 0) is laid out by `Group::compute_form@00707c80` at the charge
# with the guard flag and a human's formation 1; each member gets one
# GUARD (type 12) with its slot's (dx, dy) and the action bit. Each frame
# `do_guard` puts the post at the charge's position plus (dx, dy) rotated by
# its heading, snapped to the 48-unit cell; off it, a transit ATTACKTOORDER
# at QUEUE_FIRST, `do_move` the same frame, and `retry = rand % 3 + 6` when
# that leg ends at once — the one draw. While the charge moves both periodic
# arms are skipped; while it stands, on `(o + frame + 8) % 16 == 0`,
# `Unit::find_melee_target@005ff9c0(-1, 0, 0, 1, 0)` searches the ordinary
# respond radius and stacks the attack at QUEUE_FIRST above the GUARD.
#
# ---------------------------------------------------------------------------
# THE PREMISE AND ITS KILLER, read before the run (docs/GOLDEN.md §3, point 5).
#
# The premise: **a player's guard command on a unit gives the guard one
# GuardOrder on its charge; the guard takes its post, keeps it while the
# charge walks, and engages an enemy in its respond radius while the charge
# stands, the attack stacked above the guard.**
#
# 1. `check_accept_issue` and `process_group`'s player test, as in chapter
#    nine, with the same writers; none runs in a Quick Battle.
# 2. **`action_guard`'s early returns**: `GroupData::buildings` (+0x49),
#    written only by `Group::add@00714350` (the DLL names unit captains);
#    the charge not `is_valid_unit` or not `is_on_map` (the building half,
#    by construction; a cheat's wagon is on the map); `LeaderData::is_ally@
#    006edb50(0, 0)`; "Invalid order" for `ox < 2000` past whom's unit
#    count, an Error::report (0/7 is inside it); an empty escort.
# 3. The human sweep and the cycle break reach nothing: at 620 no unit
#    guards the wagon, and the wagon has no order.
# 4. The member and slot loops are bounded by `group.num` (+0xc, below 0x80
#    in `Group::add`), here 1; the sweep by who=0's unit count.
# 5. The unit half stays on BASELAND of region 1, column x 3..5, y 9..18,
#    west of chapter nine's sand (region 65, x 7..11); cell (9, 38) is
#    baseland; no goody box or animal within five cells.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (a call on trace frame F
# is processed before tick F+1, so its order is first on block F+2).
#
# 1. **The issuer does not reach the pump.** Trace frames 620, 640, 720: an
#    `INFO 17` with a refusal; or no `COMMANDMANAGER` group and guard text
#    between blocks 621/622 (641/642), or no move text between 721/722.
# 2. **The guard is not one GuardOrder on its charge.** Block 622, `0/6`: no
#    type-12 order, `ox/whom` other than 7/0, or the action bit clear. The
#    offset is a value row: this crate's `compute_form` says (0, 372).
# 3. **The guard does not keep its post on a moving charge.** On the block
#    the wagon's stack empties (this crate: ~915) and on block 1000: `0/6`
#    without its GUARD on `0/7`, or `guard_x/y` more than one 48-unit cell
#    from the wagon plus the rotated offset; on 1000, `0/6` off that cell.
# 4. **The guard does not engage.** `1/6` is first on block 1001: no
#    ATTACKORDER on `1/6` above `0/6`'s GUARDORDER by block 1027 (phase ticks
#    1010 and 1026), or the GUARD gone from under it.
# 5. **The building half gives an order.** Block 642 to 1249: any order on
#    `0/8`..`0/10`.
#
# This crate, walked before the run from run184's frame 0: the GUARD on 622
# over a leg, on post (3528, 8760) by ~660; the wagon's MOVE_TO on 722,
# arriving (3456, 11904) ~915; the chariot on (3480, 12264) from ~955; `1/6`
# firing from 1004; the guard's ATTACK above its GUARD from 1011; the squad
# with no order.
#
# RUN 2026-09-24 as run190 (item 696; docs/RUNS.md): 254 s, 85 MB. NO
# FALSIFIER FIRED. All three `INFO 17` records issued (package 10 -> 28, 28,
# 37); the dump logs `process_group` and `process_guard` between blocks
# 621/622 and 641/642. Block 622: `0/6` a GUARDORDER on 0/7, flags 4,
# dx 0 dy 372, post (3528, 8760), under an ATTACKTOORDER leg (timer 59). The
# wagon's stack empties on 857; the post re-read to (3480, 12264) by 890.
# Block 1011: an ATTACKORDER on 1/6 above the GUARD. The squad: no order on
# any block. Then 1/6 walks off on an army ATTACKTO (1021), the guard's
# attack goes (1037), 1/6 returns and shoots it, the guard never re-engages
# and is gone from 1141.
#
# check: `cmdsran.py` shows seven `INFO cmd` returning 1 (`0 !ai off`, the
#        four cheats, `37 !ffwd`, `1250 !quit`); the three `@` lines are
#        `INFO 17` records.
# check: `MAP_STYLE 14`, seed 12345, blocks 1 and 605..1249 at least.
# ---------------------------------------------------------------------------

# `ai` is console-only (table index 12). Every chapter's premise.
0 !ai off

# The cast, Ancient, one lever each: the guard, a Chariot (`UBER_SIZE` 1,
# `0/6`) at chapter nine's seat, cell (4, 9); its charge, a Supply Wagon
# (`UBER_SIZE` 1, no attack, `0/7`), two cells south on cell (4, 11); a
# Hoplite squad (captain `0/8` with `0/9`, `0/10`) on cell (9, 38), four
# cells east of who=0's `0/2001`.
610 add chariot who=0 16,36
612 add supply who=0 16,44
614 add hoplite who=0 36,152

# THE LEVERS: two player guards through the issuer, `@guard <who> <ox>
# <whom> <o>`. The chariot guards the wagon.
620 @guard 0 7 0 6
# The squad, by its captain, guards the building: predicted to give nothing.
640 @guard 0 2001 0 8

# The charge walks, through chapter nine's issuer: to cell (4, 15)'s centre.
720 @move 0 3456 11904 7

# An enemy in range while the charge stands: who=1's next unit, `1/6`, ~1,300
# units south of the guard's post, inside both chariots' sight.
1000 add chariot who=1 17,70
