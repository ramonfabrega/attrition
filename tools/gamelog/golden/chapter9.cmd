# Golden record, chapter nine — the move line, the first issuer chapter.
#
# docs/GOLDEN.md §17 (item 676; docs/DECISIONS.md 41 §1 and §5, 49). Every
# order in chapters one to eight was one the original's own automatic play
# issued. This chapter's two orders are a player's: each `@` line below is
# not a cheat but a call, from rontrace.dll, of the original's own
# `CommandManager::issue_move_to@00941720`, and the turn pump processes the
# command it appends exactly as it processes a right-click
# (tools/trace/tracer.c, `issue_line`). The rest is chapter one's lobby,
# Ancient, with `!ai off`.
#
#   run180 (item 676):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch9 \
#       --map 14 --end-frame 1100 --log-window 605 1100 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter9.cmd
#
# ---------------------------------------------------------------------------
# THE ISSUER, as the emulator ran it before the pair (item 676, step 1).
#
# `issue_move_to` under unicorn with these two calls' own arguments —
# `(group, x, y, QUEUE_NEW 2, set_angle 0, angle 0, MOVE_TO 1, form -1,
# width -1, disembark 0)`, the plain right-click `WorldMap::on_right_up@
# 008c7050:203` passes — writes the local `CommandPackage` (+0x10 size,
# +0x12 data) and the four selection caches `CommandPackage::last_who_sent`,
# `last_num_sent`, `last_objects_sent` and `last_uids_sent`, and nothing
# else: no unit, no order, no draw (SP: `add_group`'s padding roll is under
# `semaphore & 4`). The chariot's call appends 27 bytes, `group` num 1 who 0
# [6] and a 22-byte `move_to`; the squad's 27 as well, `group` num 1 [7] —
# ONE object, the captain — and the same `move_to` to its point. Everything the
# chapter measures happens at process time, which the emulator cannot reach
# without the game: `process_group` building the group from the captain's
# `o_down` chain, `Groups::push_group`, `Group::action_move_to` and the
# path. The lab's L15 calls passed `orders 0, form 0, width 0`, which is not
# a click; on the wire the two differ in exactly those three bytes.
#
# ---------------------------------------------------------------------------
# THE PREMISE AND ITS KILLER, read before the run (docs/GOLDEN.md §3, point 5).
#
# The premise: **a player's move command, issued through the original's
# issuer, reaches the order family and is walked** — a lone unit gets a
# plain `MoveOrder` with the action bit and a world plan, and a squad of
# three gets three `GroupMoveOrder`s under one id and one leader, and both
# arrive.
#
# 1. **What would kill the whole premise is `CommandManager::
#    check_accept_issue@00940a70` answering 0**, so that nothing is appended.
#    It answers 1 in a solo game unless `use_mp_playback` is set or the
#    semaphore has `0x10` (playback) or `4` (network). Their writers:
#    `use_mp_playback` — `CommandManager::CommandManager@00943440` (0) and
#    `CommandManager::set_mp_playback@0093ee70`, a replay's; `semaphore &
#    0x10` — the playback path; `semaphore & 4` — `Game::run_gamespy@
#    00587060` alone (docs/COMMANDS.md §2). None runs in a Quick Battle.
# 2. **The second killer is `CommandPackage::process_group@0094a0c0`'s player
#    test**: a group whose `who` is not `info.player[package.play].who` goes
#    to `LeaderData::is_team` and is dropped, `group = -1`, and the move acts
#    on nothing. The DLL issues only for `who == console->play` (its refusal
#    1), and the console's play is 0 on every capture on disk.
# 3. **The third is `add_group`'s captain filter**: a listed unit whose
#    `is_captain` is 0 is dropped silently. The DLL refuses (refusal 3) on
#    any object that is not a live captain of `who` with that id, so an
#    append always names what the line names.
# 4. **What decides plain move against `GroupMoveOrder`** is
#    `Group::action_move_near@00704990`'s per-member test (docs/ORDERS.md
#    §8.2; `705f00`–`705f61`): a group of two or more, `MOVE_TO`, not
#    modern infantry, not a human's scout, not `unit_masks & 4`, not sea,
#    `form != 9`. The chariot's group is one (`group.num < 2`), so it takes
#    `add_move_facing_order`; the hoplites' is three, since `process_group`
#    walks the captain's `o_down` chain (`+0x90`) and `Group::add`s each
#    figure. Its loop is bounded by `group.num`, at most 128 (`GroupData`'s
#    `list`), here 3.
# 5. **Neither order is issued while its unit is on its birth frame, or in
#    reach of anything.** The chariot is born on 610 and ordered on 620;
#    the squad born on 612 and ordered on 640. Both seats and both walks are
#    on BASELAND of region 1, and the two walks never come within six cells
#    of each other. No walk enters a goody box's cell — the only way a
#    walker opens one (`Unit::set_new_location@005f8d20`, docs/GOODY.md §2;
#    the nearest, (1, 19), is three cells off the squad's line) — no animal
#    is within eight cells, and the enemy's nearest unit is ~35 cells off;
#    who=1 is `!ai off`.
# 6. **The world plan, and the lake.** `PathFinder::find_wpath@00688fc0`
#    returns the goal alone when start and goal cells are under three cells
#    apart by Manhattan; the chariot's move is 12, the squad's 10. Between
#    the chariot and its point is a SANDY lake, region 65, cells x 7–11,
#    y 7–17 (run175's start WORLD): its plan must bend round the north end.
#    The squad's line runs down the west shore, x 4–6, y 13–21, all
#    region 1. For a human leader (`leaders & 4`) of a land unit the goal
#    cell's `was_seen` is read, but start and goal share region 1, so the
#    search runs either way.
# 7. **What this crate predicts, walked before the run** (item 676's
#    scratch walk, stood up from run175, which shares frames 0..599): the
#    chariot a plain `MOVE_TO` with an 11-entry plan round the lake's north
#    end, arriving exactly on (12672, 7296) on block 990; the squad three
#    `GroupMoveOrder`s under one id, leader `0/7`, a 7-entry plan, the
#    figures on their slots on blocks 938–941.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (the logger block N is the
# state after trace tick N−1; the call on trace frame F is processed after
# block F+1 is written and before tick F+1, so its order is first on block
# F+2).
#
# 1. **The issuer does not reach the pump.** Fires on the trace: an
#    `INFO` 17 on frame 620 or 640 with a refusal in its high half, or no
#    `COMMANDMANAGER` group and move text between blocks 621 and 622 (641 and
#    642). Predicted not to (points 1–3).
# 2. **The chariot's order is not a plain move with a plan.** Fires on block
#    622, `0/6`'s order stack: anything but one `MOVEORDER` of type 1 with
#    the action bit, or a path stack of fewer than three entries. Predicted
#    not to (points 4, 6).
# 3. **The squad's orders are not one formation.** Fires on block 642,
#    `0/7`–`0/9`: anything but three orders of type 19 (`GROUP_MOVE`) with
#    one id and one leader. Predicted not to (point 4).
# 4. **A walk does not arrive.** Fires on the block where a unit's stack
#    empties short of its destination — the chariot more than a tile from
#    (12672, 7296), a figure more than two cells from (4992, 16512) — or
#    on 1099 if either is still walking. Predicted not to (points 5–7).
#
# check: every staged line runs. `cmdsran.py` shows five `INFO cmd`
#        returning 1 (the three cheats here, `37 !ffwd`, `1100 !quit`) —
#        the two `@` lines are `INFO 17` records, not `INFO cmd`.
# check: `MAP_STYLE 14`, seed 12345, blocks 1 and 605..1099 at least.
# check: the units are identified by type and birth block, never by slot.
# ---------------------------------------------------------------------------

# `ai` is console-only (table index 12). Every chapter's premise.
0 !ai off

# The cast, Ancient, one lever each. A Chariot is `UBER_SIZE` 1: one figure,
# the lone unit (`0/6`, the first object after who=0's six). Hoplites are
# `UBER_SIZE` 3: `init_unit` threads a captain and two figures on `o_up` /
# `o_down` (`0/7` the captain, `0/8`, `0/9`), as chapter one's run105 shows.
610 add chariot who=0 16,36
612 add hoplite who=0 16,52

# THE LEVER: two player orders through the issuer, internal coordinates.
# The chariot, from tile (16, 36), cell (4, 9), round the lake to cell
# (16, 9)'s centre.
620 @move 0 12672 7296 6
# The squad, by its captain alone, from tile (16, 52), cell (4, 13), to
# cell (6, 21)'s centre.
640 @move 0 4992 16512 7
