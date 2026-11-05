# Item 3881 (docs/AI.md §382.4; parked 3877b): chapter fifteen's lines with
# an unarmed member, a General, in the selection, so that
# `Unit::do_attack@005f1b80:119`'s unarmed, grouped arm calls
# `GroupData::get_max_range@00711080` on 735–736.
#
#   run1498:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/run1498 \
#       --map 14 --end-frame 1000 --log-window 605 1000 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/coverage/maxrange3881-ch15.cmd --cover cover=1
#
# The cast: Hoplites 0/6..0/8 (captain 6), the General expected at 0/9 (the
# next id; read off `UNITS` at 612), Hoplites 0/10..0/12 (captain 10), and
# run215's Chariot 1/6. The selection names 6, 9 and 10, so the General is
# in it whichever squad id it pushes to. The twin is run215 (`cover=0`), and
# the take parts it from 611, the first unshared input, by construction.
#
# Killers (journal 2026-11-05-item-3881): the General's head on 736 is not
# an ATTACKORDER (`Group::action_attack` filters it out), or it is and
# 0x711080 carries no HIT.

0 !ai off

610 add hoplite who=0 12,84
611 add general who=0 14,84
612 add hoplite who=0 18,84
614 add chariot who=1 16,64

620 @move 0 3192 10752 6 9 10
734 @attack 0 6 1 6 9 10
860 @amove 0 3192 7680 6 9 10
