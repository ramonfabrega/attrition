# Golden record, chapter sixteen — explore and flee: the move issuer's
# trailing selector.
#
# docs/GOLDEN.md §24 (item 738; docs/DECISIONS.md 41 §1 and §5, 49). Chapter
# nine's harness, two more verbs: `@explore` and `@flee` are calls, from
# rontrace.dll, of the original's own `CommandManager::issue_move_to@
# 00941720` with a right-click's arguments and its `orders` byte — the
# trailing selector — set to EXPLORE_TO (3) and FLEE_TO (4) for MOVE_TO. The
# Explore button's pick on the ground passes exactly that
# (`Options::picked_spot@00721c40:905`); the Flee button's (`:946`) passes
# FLEE_TO to a friendly building's point and then a QUEUE_LAST garrison of
# it, and the verb issues the move alone, on the ground.
#
#   run219 (item 738):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch16 \
#       --map 14 --end-frame 1250 --log-window 605 1250 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter16.cmd
#
# `GUYS=4` with `GROUPS=1` and neither `DEATHS` nor `AMMO`: run215's levels,
# the line whose pool printed (parked 733). The goody-box leg is a pushed
# group, so the pool is worth having.
#
# ---------------------------------------------------------------------------
# THE ISSUER, as the emulator ran it before the pair (item 738, step 1).
#
# Under unicorn on command_oracle.py's fixture, widened to who=0's objects
# 6..14 with 6, 9 and 12 as live captains, `issue_move_to(group, 3192, 10752,
# QUEUE_NEW, 0, 0, orders, -1, -1, 0)` for `orders` 0 to 5 appends the same
# 27 bytes (a 5-byte `group` of one and the 22-byte `move_to`) for every
# value, differing in the `orders` byte alone: the issuer copies it and
# tests nothing. The same selection again appends the 3-byte reuse; a
# non-captain is dropped; `queued` 0, 1 and 2 ride through;
# `use_mp_playback`, `semaphore & 0x10` and `semaphore & 4` each append
# nothing. **No order, no unit, no draw.** The class is chosen at process
# time, which the emulator cannot reach: `CommandPackage::process_move_to@
# 009497c0` -> `Group::action_move_to@0070fba0` -> `Group::action_move_near@
# 00704990` -> `Unit::add_move_facing_order@005e55c0`.
#
# ---------------------------------------------------------------------------
# THE READING (docs/GOLDEN.md §24; docs/ORDERS.md §1.2, §4.1, §4.3, §8.1;
# docs/GROUPS.md §4.1, §6.6, §17; docs/GOODY.md §2, §7).
#
# `add_move_facing_order` switches on its kind: 2 -> get_obj(ATTACK_TO),
# 3 -> get_obj(EXPLORE_TO), 4 -> get_obj(FLEE_TO), anything else MOVE_TO.
# `action_move_near:758` hands a member to `add_group_move_order` only for
# MOVE_TO or ATTACK_TO, so **a squad told to explore or flee gets one plain
# order a member, at its formation slot, and no group order**.
# EXPLORE_TO is stepped by `Unit::do_explore_to@005f24a0`: `do_move`, and
# every fifteenth frame (`(o + frame) % 15 == 0`), on a captain whose head
# is still this order, `find_goody_box@005f2540`'s 49-cell sweep. A seen box
# in the unit's region sends `get_goody_box@005f7690`: a one-member group of
# the captain (which drags its figures, GROUPS §4.1), pushed, and
# `action_move_to(box cell centre, QUEUE_FIRST, EXPLORE_TO, action 0)`. The
# group's QUEUE_FIRST copies the leader's action-flagged orders aside, halts,
# issues the box leg QUEUE_NEW and re-issues each copy through
# `Group::finish_insert@0070e620` case 3: `action_move_near(orig_x, orig_y,
# QUEUE_LAST, set_angle 1, the copy's angle, EXPLORE_TO, action 1)` — to the
# copy's **click**, not its slot. Stepping into the box's cell opens it
# (`Unit::set_new_location` -> `explore_goody`, one draw a candidate good).
# FLEE_TO is stepped by `Unit::do_flee_to@005f2480`, which is `do_move`.
#
# ---------------------------------------------------------------------------
# THE CAST, on chapter nine's and ten's ground (BASELAND, no border). Two
# goody boxes are in reach, read off run215's start WORLD: (1, 19) and
# (16, 21).
#   0/6        a Chariot, seated at tile (12, 60), cell (3, 15)
#   0/7..0/9   Hoplites, captain 0/7, seated at tile (52, 60), cell (13, 15)
#
# A call on trace frame F is processed between blocks F+1 and F+2, and its
# orders are in block F+2 (docs/GOLDEN.md §17's convention).
#
#   620: the chariot explores south, past the box at (1, 19).  Orders in 622.
#   640: the squad explores south-east, to two cells short of the box at
#        (16, 21), which its captain sees on the way.           Orders in 642.
#   900: the chariot flees back north.                          Orders in 902.
#  1000: the squad flees back north-west.                       Orders in 1002.

0 !ai off

610 add chariot who=0 12,60
612 add hoplite who=0 52,60

620 @explore 0 2400 17280 6
640 @explore 0 12672 14976 7
900 @flee 0 2400 11520 6
1000 @flee 0 10752 11520 7
