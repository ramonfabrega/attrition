# Golden record, chapter one hundred and twenty-four -- a strike from inside a base on a target unseen (item 3968,
# GOLDEN 174; T71-5). Chapter twenty-two's game whole, and one line: `@strike 0 2006 1 6` on 740, the Fighter `0/6`
# inside `0/2007` since 722, on who=1's Barracks `1/2006`, which who=0 has not seen (`ever_seen` 2 until 762).
# `Group::action_flight@006fb260`'s inside arm: `6fbba8 testl $0x8000000, 0x1e4; je 6fbf6c` -- a type without the
# missile bit jumps past `valid_target`, the shield and the reach to `6fbe61`'s `is_ally || war_allowed`, and is
# `add_strafe_order(2006, 1, 2007, 0, 1, QUEUE_NEW, 1)`. So the original lays the strike on 742; the crate before
# item 3968 refused it on `valid_target` (an unseen building). The predicate is the type's missile bit, so a Fighter
# enters where the booking named a Bomber; no Bomber stands inside before the Barracks is seen in this game.
# On 766 chapter twenty-two's own strike re-points the strafe (the STRAFE arm), so from 768 the game is twenty-two's.
# Killers, before the take (the journal's list): (1) the cover=1 twin enters no `Group::action_flight` on trace
# frame 740 -- the press never reached the group; (2) `0/6` holds no order on block 742 -- the original refuses an
# unseen target and T71-5 is wrong; (3) `1/2006`'s `ever_seen` holds who=0's bit (1) on 742 -- the target was seen,
# and the staging does not test the arm.
#   run1539 (cover=0), run1539b (cover=1):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch124 \
#       --map 14 --end-frame 1000 --log-window 605 1000 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter124.cmd
0 !ai off
600 library who=0 6
606 add airbase who=0 60,72
610 add fighter who=0 60,84
612 add bomber who=0 52,84
614 add bomber who=0 68,84
616 add barracks who=1 110,86
620 @strike 0 2006 1 7 8
640 @flight 0 2007 0 6
660 @flight 0 2007 0 7 8
664 @strike 0 2006 1 7 8
740 @strike 0 2006 1 6
766 @strike 0 2006 1 6
