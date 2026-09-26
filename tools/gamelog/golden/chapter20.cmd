# Golden record, chapter twenty — the board line: the player's transport
# toggle, and the move it gates across a lake.
#
# docs/GOLDEN.md §28 (item 803; docs/DECISIONS.md 41 §1 and §5, 49). Chapter
# nine's harness, one more verb: `@settransport <who> <flag> <o>…` is a
# call, from rontrace.dll, of the original's own
# `CommandManager::issue_set_transport@00941910(group, flag)` — what
# `Options::do_transport@0071c500` passes through `GroupOut::
# issue_set_transport@0070ad10` for the transport button
# (`!GroupData::can_transport()`), and what `Options::picked_spot@00721c40`'s
# `OPTION_DISEMBARK` passes (1) ahead of an `issue_move_to`.
#
#   run249 (item 803):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch20 \
#       --map 14 --end-frame 1300 --log-window 605 1300 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter20.cmd
#
#   Run 2026-09-26: 836 s, 286 MB, the same game as run250 for all 647
#   frames they share, 355,840 GROUPDATA (512 a block). No falsifier fired:
#   0/7's bit off on 622 and on again on 802, no order laid by either
#   toggle; 0/6's Transport CASTORDER (650, flags 0) on 703 and barge 0/8
#   on 704 with 0/6 inside; 0/7's stack empty at the shore on 719, no
#   barge; its cast on 829 and barge 0/9 on 830; 0/8 at its point on 856
#   and 0/9 on 967; 0/6 ashore on 1160, 0/8 gone. No BOARDORDER or
#   AWAITBOARDORDER on any block. docs/RUNS.md, run249.
#
# `GUYS=4` with `GROUPS=1`: every golden capture's level since run215, the
# line whose pool printed. `LEADERS=2` prints `leader_flags`, whose `0x700`
# is the transport level; `UNITS=3` prints each unit's `unit_masks`, whose
# `0x800000` is the auto-transport bit, its order stack and its path;
# `BUILDS=7` the Dock. The window is 695 blocks: this crate expects the
# first parting between 702 (the embark) and 718 (the unflagged unit at the
# shore), and the chapter's last event, the disembark, near 1160; 250 blocks
# of runway past 718 end at 968, so the window is sized to the disembark
# plus 140.
#
# ---------------------------------------------------------------------------
# THE ISSUER, as the emulator ran it before the pair (item 803, step 1).
#
# Under unicorn on command_oracle.py's fixture, `issue_set_transport(group,
# flag)` appends a 5-byte `set_transport` (type 0x0e, `[flag i32]`, as
# passed, 1 and 0 alike) behind a fresh 5-byte `group` — 10 bytes for one
# object — or the 3-byte reuse (8). `GroupOut::issue_set_transport` appends
# the same bytes, and nothing under `semaphore & 0x10`; `use_mp_playback`
# appends nothing. It writes the package and the selection caches: no order,
# no unit, no draw. Its prologue is `55 8b ec 83 ec 08 b9 60 ff e8 00`.
#
# THE PROCESS SIDE (read, not emulated): `process_set_transport@00948f60`
# logs `process_set_transport <frame>` and calls `Group::action_set_
# transport(flag)@007024b0`: `action_begin` (it zeroes the group's +0x28),
# then — for a group that is not buildings — the flag forced to 0 when the
# leader's level (`leader_flags` 0x100/0x200/0x400) is 0, and over the
# group's `num` members each active, `can_ever_transport` unit's
# `unit_masks` 0x800000 set (flag 1) or cleared (flag 0). No order.
#
# THE STAGING, walked before the capture (run250, the staging to 646, this
# file's first eight lines; docs/RUNS.md run250). `leader_flags` 1799
# (the level bits 0x100, 0x200, 0x400) from 605: `library who=0 1` gains Written Word, TRANSPORT_BONUS's
# prerequisite, and the Dock on 604 is the dock `check_transport` counts.
# Every who=0 unit carries 0x800000 on 605; `0/6` and `0/7` are born with
# it; `process_set_transport 621`, and `0/7`'s `unit_masks` 8388608 → 0 on
# 622 with its stack empty. On 642 `0/6` holds a `MOVEORDER` (flags 5) to
# (11904, 34944) and a six-entry plan whose waypoint (8856, 34200) — cell
# (11, 44), the lake's first water cell — carries flags 4, the embark. On
# 644 `0/7` holds the same order to (11904, 36480) and a six-entry plan
# straight onto the water with **no** flags-4 entry and no pull-back.
#
# ---------------------------------------------------------------------------
# THE READING (docs/GOLDEN.md §28; docs/TRANSPORT.md §3.4, §6).
#
# 1. **The toggle builds no order.** `BoardOrder` and `AwaitBoardOrder` are
#    built by `Unit::add_board_order@005e4d10` and `Unit::add_await_board_
#    order@005e4c80`; the first's one caller is `Group::action_board_ship@
#    00700010`, reached from `process_board_ship@00948e00` (command 0x0f,
#    which nothing issues: docs/COMMANDS.md §6) and from `Group::finish_
#    insert@0070e620`'s case 8, which replays a `BoardOrder` that exists;
#    the second's are `action_board_ship` and `Unit::check_meet_ship@
#    00604550`, under `Unit::do_board@005ed1f0`, a `BoardOrder`'s own step.
#    `copy_order@0072f900` and `get_new_order@00730550` copy or load one.
# 2. **What boards is a cast.** `Unit::set_new_location@005f8d20`: a step
#    of a unit that `can_transport` onto a water tile is `add_cast_order(
#    -1, -1, -1, -1, 0x28a, QUEUE_FIRST, 0)` — the Transport craft (650),
#    flags 0 — and the unit's frame ends; `do_cast` → `SpellType::cast_
#    transport@00670db0` then makes the barge, which takes the unit's orders
#    and path, the unit inside it.
# 3. **Without the bit** the step onto water is refused (`UnitData::
#    invalid_loc@00607c30`'s land arm) and no cast is laid.
#
# ---------------------------------------------------------------------------
# THE CAST, on the west shore of lake 70 (sea region 70; run105's start
# `WORLD`: land to x 10 on rows 44–47, water from x 11).
#
# - 600: `library who=0 1` — Classical, and Written Word.
# - 604: a who=0 **Dock** at tile (53, 153), cell (13, 38), the lake's
#   north-west shore — the level, 3.
# - 610: **Chariot `0/6`** at tile (34, 177), (6648, 34104).
# - 612: **Chariot `0/7`** at tile (34, 189), (6648, 36408).
# - 620: `@settransport` flag 0 on `0/7`.
# - 640: `0/6` to (11904, 34944), cell (15, 45), deep water.
# - 642: `0/7` to (11904, 36480), cell (15, 47), deep water.
# - 800: `@settransport` flag 1 on `0/7`.
# - 820: `0/7` to (11904, 36480) again.
# - 900: `0/8` — the barge this crate expects `0/6`'s cast to make — to
#   (19584, 34944), cell (25, 45), dry ground on the east shore.
#
# A call on trace frame F is processed between F+1 and F+2 and is on block
# F+2 (§17).
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (docs/GOLDEN.md §28).
#
# 1. The issue does not reach the pump: trace frame 620 or 800, an INFO 17
#    with a refusal; or no `process_set_transport` on 621 or 801.
# 2. The toggle is not the bit: block 622, `0/7` still with 0x800000; or
#    block 802 without it.
# 3. The toggle builds an order: `0/7`'s stack not empty on 622 or 802; or
#    a `BOARDORDER` / `AWAITBOARDORDER` on any unit on any block.
# 4. The unflagged unit boards: `0/7` holding a `CASTORDER`, or a who=0
#    unit born, before 800 — first at the block its step reaches the
#    waterline (this crate: 716–718).
# 5. The flagged unit does not board by a cast: `0/6`'s stack not a
#    `CASTORDER` (spell 650, flags 0) ahead of its `MOVEORDER` on the block
#    before its step crosses (this crate: 702); or no `0/8` of the barge
#    type on the next block holding that `MOVEORDER`, with `0/6` off the
#    map.
# 6. The re-flagged unit does not board: after 820, `0/7` walking into the
#    water or stopping as before, with no `CASTORDER` (this crate: 828).
# 7. The disembark: `0/8` does not put `0/6` ashore on the east bank, or
#    survives it (this crate: 1159); or the 900 line is refused (INFO 17,
#    refusal 3: `0/8` is not the barge).
#
# ---------------------------------------------------------------------------

0 !ai off
600 library who=0 1
604 add dock who=0 53,153
610 add chariot who=0 34,177
612 add chariot who=0 34,189
620 @settransport 0 0 7
640 @move 0 11904 34944 6
642 @move 0 11904 36480 7
800 @settransport 0 1 7
820 @move 0 11904 36480 7
900 @move 0 19584 34944 8
