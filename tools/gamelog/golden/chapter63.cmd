# Golden record, chapter sixty-three — a shift-queued move laid out from a
# FollowOrder's leader (item 2131, `docs/GOLDEN.md` §78; parked 2058).
#
# `Group::action_move_near@00704990:361` asks `GroupData::get_loc_to@
# 0070c5d0` for a QUEUE_LAST move, and it answers `UnitData::get_final_loc@
# 00608040` of the group's leader: the order list walked from its bottom,
# the first order that names a point. An order whose vslot +0x20 answers 1
# is a target order, and its +0xb4 `TargetOrder`'s `ox/whom` object's
# position is the point. `FollowOrder` is one: its block carries
# `TargetOrder::is_targeted`'s and `get_target_order`'s `vtordisp{-4,12}`
# thunks (00486c2e, 00486c54), and `is_targeted` is a folded `return 1`
# (`Buffer::is_pending_load@0041e0e0`). This crate walks past a Follow, so
# its formation angle is read from the leader's own point, or from a leg's
# end above the Follow. No AI class issues a follow, and the DLL had no
# shift verb: `@moveadd` is `@move` at QUEUE_LAST, what `WorldMap::
# on_right_up@008c7050:88` takes with shift held.
#
# THE STAGING, walked by this crate on run204's start (RON_STAGE). A Hoplite
# squad `0/6`-`0/8` on (60, 56), a Chariot `0/9` on (60, 60), 768 south, and
# a second squad `0/10`-`0/12` on (64, 56). Both squads follow the Chariot
# (620, 622) and stand. 660: `0/6`'s squad is shift-moved to (12408, 12408):
# its list is the Follow alone, the original reads the Chariot's point
# (11640, 11640), 45 degrees to the point, where this crate reads `0/6`'s own.
# 700: the Chariot walks south; `0/10` takes a leg above its Follow. 710:
# `0/10`'s squad is shift-moved to (13560, 13560): bottom first, the
# original reads the Follow, the Chariot's point; this crate the leg's end.
#
# CENSUS ROWS the lobby reaches, never backed (`census.py --unbacked`):
# `GroupData::get_loc_to@0070c5d0`, `UnitData::get_final_loc@00608040`,
# `Group::action_move_near@00704990` (its QUEUE_LAST arm),
# `Group::action_move_to@0070fba0`, `CommandPackage::process_move_to@
# 009497c0`, `CommandPackage::process_follow@009479c0`,
# `Group::action_follow@006fd510`, `Unit::add_follow_order@005e3f60`,
# `Unit::do_follow@005e65d0`.
#
# WHAT WOULD FALSIFY IT. (1) An `I_ISSUE` refused on 620, 622, 660, 700 or
# 710 (`issuesmatch.py --none-refused`). (2) On block 661 `0/6`'s list is not
# [FOLLOW, MOVE]: the shift-move not at the bottom. (3) Block 661: the queued
# MOVE's `angle` is the bearing from `0/6`'s own point, not the Chariot's
# (`find_angle` of (768, 768)): the Follow is not a target order. (4) Block 711: `0/10`'s queued MOVE laid out
# from the leg's end, not the Chariot.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch63 \
#       --map 14 --end-frame 760 --log-window 605 760 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter63.cmd
0 !ai off

610 add hoplite who=0 60,56
612 add chariot who=0 60,60
614 add hoplite who=0 64,56

620 @follow 0 9 0 6
622 @follow 0 9 0 10

660 @moveadd 0 12384 12384 6

700 @move 0 11616 15072 9
710 @moveadd 0 13536 13536 10
