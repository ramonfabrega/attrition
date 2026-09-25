# Golden record, chapter fifteen — the group attack, an issuer the AI
# rarely takes whole.
#
# docs/GOLDEN.md §23 (item 731; docs/DECISIONS.md 41 §1 and §5, 49). Chapter
# nine's harness, two more verbs: `@attack` is a call, from rontrace.dll, of
# the original's own `CommandManager::issue_attack@009415e0` with an
# unmodified right-click's arguments on an enemy (`Console::
# execute_at_cursor@007c6630:2741` through `GroupOut::issue_attack@0070b060`:
# the target's `ox` and `whom`, `ignore` 0, QUEUE_NEW); `@amove` is chapter
# nine's `issue_move_to@00941720` with ATTACK_TO (2) for MOVE_TO, a
# ctrl+right-click on the ground (`WorldMap::on_right_up@008c7050:199`).
# The `@move` before them is chapter nine's right-click. No capture on disk
# holds a player's attack command: the console has no attack verb, and every
# attack on disk is the AI's `engagement` (mandatory 0) or an auto-engage.
#
#   run215 (item 731):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch15 \
#       --map 14 --end-frame 1250 --log-window 605 1250 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter15.cmd
#
#   Run 2026-09-25: ~13 min, 269 MB, same game as run210 to frame 614. All
#   three commands processed on the next frame (`process_attack 6 1 0 2
#   735`). No falsifier fired: six ATTACKORDERs, mandatory 1, flags 20, on
#   736 and no GroupAttackOrder anywhere; 1/6 last prints on 808; six
#   GROUPATTACKTOORDERs, id 861102, on 862, ungrouped on 1060 and standing
#   from 1080 on the points this crate predicted. The pool printed:
#   330,240 GROUPDATA at GUYS=4 (docs/RUNS.md, run215).
#
# `GROUPS=1` for the pool, with `GUYS=4` and neither `DEATHS` nor `AMMO`
# under `[End Frame]`: run178's levels, the one golden-lane-shaped line
# whose pool printed. `GroupData::log_data@0045e1d0` sets no logger type of
# its own, so its lines pass `GameLog::check_accept@009309a0` only when the
# type and detail the previous dumper left are accepted, and
# `GameLog::full_dump@00930380` runs `dump_units`, `dump_walls`,
# `dump_ammo`, `dump_deaths` and then `dump_groups`. run210 asked for the
# pool at `GUYS=2` and got none (parked 733). The target dies inside the
# window, and without `DEATHS` its death is read off `UNITS`, where it stops
# printing.
#
# ---------------------------------------------------------------------------
# THE ISSUER, as the emulator ran it before the pair (item 731, step 1).
#
# Under unicorn on command_oracle.py's fixture, widened to who=0's objects
# 6..14 with 6, 9 and 12 as live captains:
#   `issue_attack(group [6 9 12], ox 2, whom 1, ignore 0, QUEUE_NEW 2)`
#   appends **26 bytes** — the 9-byte `group` (num 3, who 0, the three
#   objects) and a 17-byte `attack`, type 0x04, `[ox i32][whom i32][ignore
#   i32][queued i32]` — and writes the package's size and data and the
#   selection caches, and nothing else: no unit, no order, no draw. The same
#   selection again appends the 3-byte reuse (20); `ignore 7, QUEUE_FIRST`
#   and `ox 0x7fffffff, whom 7, QUEUE_LAST` ride through as passed; a
#   non-captain in the list is dropped. **A negative `ox` or `whom` appends
#   nothing and writes nothing**: the issuer tests both before it asks
#   `check_accept_issue`. `use_mp_playback`, `semaphore & 0x10` and
#   `semaphore & 4` each append nothing.
#   `issue_move_to(group, 4000, 13000, QUEUE_NEW, 0, 0, ATTACK_TO, -1, -1,
#   0)` appends the 9-byte group and the 22-byte `move_to` with its
#   `orders` byte 2.
# **What the emulator cannot reach**: `CommandPackage::process_attack@
# 00949c30` -> `Group::action_attack@00712490` -> `Unit::add_attack_order@
# 005e5410`, and `process_move_to@009497c0` -> `Group::action_move_to` ->
# `action_move_near@00704990` -> `Unit::add_group_move_order@005e4710`.
#
# ---------------------------------------------------------------------------
# THE READING (docs/GOLDEN.md §23; docs/ORDERS.md §7.9, §8.1, §8.3, §8.5;
# docs/GROUPS.md §6.6 and §10).
#
# **No `GroupAttackOrder` is made.** Nothing in the export asks
# `OrdersMemManager::get_obj` for GROUP_ATTACK (20); `get_new_order@
# 00730550:193` builds one only for the save loader and for `copy_order@
# 0072f900`, which copies an order that exists. A player's attack on a
# unit is **N `AttackOrder`s**: `process_attack` hands `ox, whom, ignore,
# queued` to `action_attack(g, ox, whom, mandatory 1, queued, ignore)`,
# and for a unit group on the map, a live unit target and QUEUE_NEW:
#   the leader asks `Unit::find_attack_pos` once when it is out of range
#   (`action_attack+0x41a`, the draw family); then three passes over the
#   members, one a domain, each member (figures too) getting
#   `add_attack_order(unit, ox, whom, QUEUE_NEW, mandatory 1, action 1)`:
#   QUEUE_NEW closes the member's orders, the order is an ATTACK (10) on
#   the target with its `uid`, `mandatory` 1, `new_ord` 1, the action bit.
#   `mandatory 1` is why no member retargets: the `find_melee_target`
#   retarget is the `mandatory == 0` arm, the AI's `engagement`.
# **The ground point is a `GroupAttackToOrder`**, and it is not
# `issue_attack`'s: the click on ground is `issue_move_to` with ATTACK_TO,
# and `action_move_near`'s step 6 gives each member of a land group of two
# or more a group move, which `add_group_move_order` makes a
# `GroupAttackToOrder` (21) when its kind is 2. Its step is
# `Unit::do_group_attack_to@005e74e0`: `do_group_move`, and every fifteenth
# frame (`(o + frame) % 15 == 0`) a `find_melee_target(-1, …)` look.
#
# ---------------------------------------------------------------------------
# THE CAST, on chapter thirteen's and fourteen's ground and the column
# north of it (cells x 3..5, y 9..22: BASELAND, no border; chapter eleven's
# column and chapter thirteen's ground). Two squads and one target:
#   0/6..0/8   Hoplites, captain 0/6   (the leader)
#   0/9..0/11  Hoplites, captain 0/9
#   1/6        a Chariot of who=1, at (3192, 12408), ~3,840 north
# A Hoplite sees ~1,150 and searches 2,304 (`unit_respond_range` 12 ×
# 0xc0); a Chariot sees ~1,730 and outranges it. **There is no ground where
# who=0 sees the Chariot and nobody engages**, and a click can name only a
# seen target (`execute_at_cursor`'s FILTER_SEEN; `fight`'s `valid_target`
# drops an unseen one). So the group walks in under a right-click, whose
# action bit keeps it from answering fire, through the Chariot's spot, and
# the attack comes once it is seen.
#
# A call on trace frame F is processed between blocks F+1 and F+2, and its
# orders are in block F+2 (docs/GOLDEN.md §17's convention).
#
#   620: the right-click north, through the target's spot. Orders in 622.
#   734: the attack on 1/6, seen from ~719.                Orders in 736.
#   860: the attack-move north, on the ground.             Orders in 862.

0 !ai off

610 add hoplite who=0 12,84
612 add hoplite who=0 18,84
614 add chariot who=1 16,64

620 @move 0 3192 10752 6 9
734 @attack 0 6 1 6 9
860 @amove 0 3192 7680 6 9
