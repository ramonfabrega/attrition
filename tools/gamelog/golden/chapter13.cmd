# Golden record, chapter thirteen — the garrison line, an issuer the AI never
# uses from a command.
#
# docs/GOLDEN.md §21 (item 718; docs/DECISIONS.md 41 §1 and §5, 49). Chapter
# nine's harness, two more verbs: each `@garrison` line below is a call, from
# rontrace.dll, of the original's own `CommandManager::issue_garrison@00941a70`
# with an unmodified pick's arguments (`Options::picked_spot@00721c40`
# through `GroupOut::issue_garrison@0070a9b0`, QUEUE_NEW), and the `@eject`
# line one of `CommandManager::issue_eject_all@00941ca0` with the Eject
# button's (`Options::exec@007188c0` option 0x1f -> `Options::do_eject_all@
# 0071c470` -> `GroupOut::issue_eject_all@00708b90`: back_to_work 0 and three
# -1s). The turn pump processes each command it appends as it processes a
# click (tools/trace/tracer.c, `issue_line`). No capture on disk holds a
# player's garrison command or an eject; every garrison the long captures
# hold is the AI's or a trained unit's.
#
#   run208 (item 718):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch13 \
#       --map 14 --end-frame 1000 --log-window 605 1000 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,BUILDS=7,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter13.cmd
#
# ---------------------------------------------------------------------------
# THE ISSUERS, as the emulator ran them before the pair (item 718, step 1).
#
# Under unicorn on command_oracle.py's fixture, widened to who=0's objects
# 6..10 as live units (6 and 8 captains) and a Build at 2003:
#   `issue_garrison(group, ox 2003, whom 0, QUEUE_NEW 2)` appends **18
#   bytes** — the 5-byte `group` (num 1, who 0, the one object) and a
#   13-byte `garrison`, type 0x14, `[ox i32][whom i32][queued i32]` — and
#   writes the package's size and data and the selection caches
#   `CommandPackage::last_who_sent` (00c06354) and `last_num_sent`
#   (00cae5f8), and `last_objects_sent`/`last_uids_sent` through the
#   imported memcpy. Nothing else: no unit, no order, no draw. The same
#   selection again appends the 3-byte reuse (16); two captains make a
#   7-byte group (20); `queued` rides through as passed (0, 1, 3).
#   `issue_eject_all(group [2003], 0, -1, -1, -1)` appends **22 bytes** —
#   the `group` naming the building (`add_group` takes a non-unit object
#   whole: its vslot 0x18 answers 0, so no captain test) and a 17-byte
#   `eject_all`, type 0x1a, `[back_to_work][who][eject_o][eject_who]`, the
#   three -1s as passed. `back_to_work 1` rides through.
#   `use_mp_playback`, `semaphore & 0x10` and `semaphore & 4` each append
#   nothing, for both.
# **What the emulator cannot reach**: `CommandPackage::process_garrison@
# 00948760` -> `Group::action_garrison@00700490` -> `Unit::add_garrison_order@
# 005e4080`; each frame's `Unit::do_garrison@005e6b80` -> `go_inside@0061a2e0`
# and `kill_garrison_order@005e2bd0`; `CommandPackage::process_eject_all@
# 00947fe0` -> `Group::action_eject_all@00710b40` -> `Object::eject_contents@
# 0064cd20`; and `Build::process_ejection@006201e0` -> `Unit::come_out@
# 00617c10`.
#
# ---------------------------------------------------------------------------
# THE READING (docs/GOLDEN.md §21; docs/ORDERS.md §5.7, §5.8; docs/CITIES.md
# §6.1-§6.6; the decompile of each function named).
#
# `process_garrison` hands `ox, whom, queued` to `action_garrison(g, ox, whom,
# queued, search 0)` when the target is dead-or-alive-checked alive. With
# QUEUE_NEW, for an own building that is active, has a garrison limit and is
# not unassimilated, every member that is active, on the map, not air (or a
# helicopter) and not entering or exiting, and whose type `can_garrison` the
# building's, gets `add_garrison_order(b, whom, search 0, QUEUE_NEW, action
# 1)`: a GARRISON (type 26) with the action bit, `ox/whom/uid` the building.
# Each frame, with it at the head, `do_garrison`:
#   not `Object::adjacent_to` the building (vslot 0x170: `attack_dist < 0x60`,
#     edge to edge): `find_nearby_spot` on the ring `min(x_size, y_size) *
#     0x60 + 0x30` = 432 round the building's centre, biased toward the unit,
#     FILTER_NOT_ME (retried relaxed), and `add_move_order(spot, MOVE_TO, 0,
#     QUEUE_FIRST, action 0)` — a plain leg above the GARRISON, which stays;
#   adjacent, and `num_inside(0) + control_cost <= limit`, and the territory
#     at the building's cell nobody's, its owner's or an ally's:
#     `go_inside(get_captain(), b, who, 0)` — the whole squad, appended at the
#     bottom of the chain (FIFO) — then `kill_garrison_order(captain)`, which
#     walks the captain's `o_down` chain and on each unit whose action is a
#     GARRISON `repath`s (pops the leading moves) and kills it.
# `process_eject_all` hands `back_to_work 0, who -1, eject_o -1, eject_who
# -1` to `action_eject_all`: `who < 0`, so every building of the group that
# is alive, holds a squad and is no hangar gets `eject_contents(0, -1, 0,
# 1)`, which for a building on the map outside the editor **defers**:
# `build_masks |= 0x4000`. Each frame `Build::process` (every frame, an
# active building) runs `process_ejection`: `come_out(captain of the head,
# 1)` — **one squad a frame, from the head**. `come_out` puts the captain on
# the ring `(x_size + y_size) * 0x30 + UNIT_TRAIN_DISTANCE` = 672 .. 864
# round the building, swept from due south, and each member round its
# captain.
#
# The cast. A Barracks (4 x 4, `GARRISON_MAX` 10) trains Hoplites (`WHERE
# Barracks`: `can_garrison`'s own-trainer arm) and not Chariots (`WHERE
# Stable`: the sibling arm, `where` 0x1ac admits 0x1ab under
# `Game::get_patch_version > 3`, which this build's own `info.version` takes
# — docs/RECGAME.md §5). Both types carry POP 1, so the two squads fill 2 of
# 10. Neither is a worker nor a packing type, so each takes the plain
# QUEUE_NEW arm.
#
# ---------------------------------------------------------------------------
# THE PREMISE AND ITS KILLER, read before the run (docs/GOLDEN.md §3, point 5).
#
# The premise: **a player's garrison command gives each commanded unit one
# GarrisonOrder on the building; each walks to the building's approach ring
# by a plain MOVE_TO leg above it, and the first of a squad to reach the
# door takes the whole squad inside, off the map, with its orders gone; the
# building's eject puts the squads back on the map one a frame, first in
# first out, on the exit ring south of it.**
#
# 1. `check_accept_issue` and `process_group`'s player test, as in chapter
#    nine, with the same writers.
# 2. **`action_garrison`'s early returns and skips**: the owner test (the
#    building's `whom` is the group's player); the building's vslot 0x10
#    `is_active`; `get_garrison_limit` 0 (a Barracks' is 10); `BuildData::
#    is_unassimilated` (a building outside any city is not a city); per
#    member, vslot 0x8 and `is_on_map` (vslot 0xbc), the air test (`domain
#    == 2` without the helicopter bit), `is_entering_or_exiting`, and
#    **`can_garrison` false, which skips the member with no order**. The
#    killer of the chariot's half is `get_patch_version`; its only input is
#    `GameInfo.version`, which the lobby writes.
# 3. **The editor arm** (`semaphore.ptr[1] & 8`, `Game::semaphore` bit 0xb):
#    QUEUE_NEW with it set is an instant `go_inside` and no order. Its
#    writers are `Game::Game` (zero) and `ConsoleWin::run_cmd`'s editor
#    toggle (set and reset) — no line here reaches it.
# 4. **`do_garrison`'s kills**: the building not active (vslots 0xc and
#    0x4c); not the unit's owner's nor a mutual ally's; limit 0 or
#    `can_garrison` false; no spot on the approach ring, relaxed; full
#    (2 of 10 cannot be); a building on an enemy's territory — the start
#    WORLD has `who -1` on every cell x 0..6, y 14..22, and no line here
#    moves a border; a city's two gates (a Barracks is not one, `flags &
#    0x20` clear).
# 5. **The eject's**: `action_eject_all` runs only with `who < 0`, or
#    `eject_who < 0` and `who` the group's; per building, alive,
#    `num_inside(1) > 0` and not `can_carry(AIR)`. `eject_contents` defers
#    only for a building on the map (vslot 0xbc), with `param_2 < 0`, outside
#    the editor, not a hangar. `come_out` refuses — the unit stays inside —
#    when `find_nearby_spot` finds nothing on the ring even with collision
#    off; the ground south of the Barracks is open BASELAND.
# 6. The loops, and their bounds: `action_garrison`'s member loop over
#    `group.num` (+0xc, below 0x80 in `Group::add`), here 1 and 3;
#    `action_eject_all`'s over its group's `num`, here 1, top down; the
#    scenario sweep ahead of each only under `ScenarioData::ignore_orders`,
#    whose writers are `ScenarioFuncSet`'s. `kill_garrison_order` and
#    `go_inside` walk the captain's `o_down` chain (`uber_size`, 3 for the
#    squad, 1 for the chariot); `process_ejection` has no loop, so one squad
#    a frame; `come_out`'s member recursion is the same chain.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (a call on trace frame F is
# processed before tick F+1, so its order is first on block F+2 — the tick
# that applies a command steps it, docs/ORDERS.md §2.3).
#
# 1. **The issuer does not reach the pump.** Trace frames 620, 640 and 900:
#    an `INFO 17` with a refusal; or no `COMMANDMANAGER` `process_group`
#    and `process_garrison` text between blocks 621/622 (641/642), or no
#    `process_eject_all` between 901/902.
# 2. **Not one GarrisonOrder a unit on the building.** Block 622 for `0/6`,
#    block 642 for `0/7`..`0/9`: no type-26 order at the bottom of the
#    stack; `ox/whom` other than 2007/0; a `uid` other than the Barracks'
#    (13); the action bit clear; `search` other than 0. Or a member of the
#    squad with none.
# 3. **A unit does not walk to the door.** Block 622 (`0/6`), 642 (the
#    squad): no MOVEORDER leg above the GARRISONORDER, without the action
#    bit, whose point is 432 .. 480 from the Barracks' centre (2688, 14208)
#    on the unit's side; or the GARRISON gone before the unit is inside.
# 4. **The door does not take the squad whole.** The chariot is predicted
#    in near block 708, the squad near 766 (its captain's ~2,820 at ~23 a
#    frame): any of `0/7`..`0/9` inside on a block the others are not; a
#    unit inside with an order left; `inside_up` other than the chain
#    2007 <- 6 <- 7 <- 8 <- 9 (the Barracks' `inside_down` 6); or a unit
#    still on the map at block 850.
# 5. **The eject does not put them out one squad a frame, first in first
#    out.** Block 902: `0/6` on the map on the ring 672 .. 864 south of the
#    Barracks (bearing 0 at 672 is (2712, 14904) snapped), the squad still
#    inside and the Barracks' `inside_down` 7. Block 903: the squad on the
#    map, `inside_down` -1. It fires if both come out on one block, the
#    squad first, anyone on 901 or earlier, or anyone later than 903.
# 6. **They come out with orders.** Blocks 902 .. 999: a stack that is not
#    empty on any of the four (back_to_work 0; no rally point on a cheat's
#    Barracks; a human's unit keeps what it had, and it had nothing).
#
# check: `cmdsran.py` shows six `INFO cmd` returning 1 (`0 !ai off`, the
#        three cheats, `37 !ffwd`, `1000 !quit`); the three `@` lines are
#        `INFO 17` records.
# check: `MAP_STYLE 14`, seed 12345, blocks 1 and 605..999 at least.
# ---------------------------------------------------------------------------

# `ai` is console-only (table index 12). Every chapter's premise.
0 !ai off

# The door. A Barracks (`0/2007`, uid 13), finished (`add`'s building arm,
# docs/INPUT.md §11.5), centred on tile (14, 74)'s corner, (2688, 14208): its
# footprint is cell (3, 18), open BASELAND with nobody's border on it.
606 add barracks who=0 14,74

# The cast, Ancient. A Chariot (`0/6`) on cell (5, 15), ~2,880 north-east of
# the door; a Hoplite squad (captain `0/7` with `0/8`, `0/9`) on cell (2,
# 22), ~3,250 south of it. Both walk BASELAND, west of chapter nine's sand
# (x 7..11).
610 add chariot who=0 20,60
614 add hoplite who=0 10,90

# THE LEVERS: two player garrisons through the issuer, `@garrison <who> <ox>
# <whom> <o>`. The chariot into the Barracks.
620 @garrison 0 2007 0 6
# The squad, by its captain, into the same Barracks.
640 @garrison 0 2007 0 7

# And the building puts them out: `@eject <who> <b>`, the Eject button.
900 @eject 0 2007
