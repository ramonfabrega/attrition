# Golden record, chapter one hundred and thirty-four -- land-based air, the second three (item 3991, GOLDEN 184;
# adjudication 3 row 36, adjudication 4 row 47, adjudication 2 row 38). Chapter seventeen's lobby and lines (GOLDEN 25,
# run223: the Airbase `0/2007`, the Fighter `0/6` inside it, the Bombers `0/7`, `0/8` out on the Barracks `1/2006`),
# and eleven more.
#   Arm 3 (first): `Group::action_flight@006fb260`'s split (`6fb3f9`..`6fb53f`). A Hoplite squad of who=0's (captain
#          `0/9`) on 780; `@strike` on 790 names the Bomber `0/7` and the squad: the planes go back through
#          `action_flight` alone, and the squad takes `action_attack(2006, 1, mandatory 1, QUEUE_NEW)`.
#   Arm 2: `Unit::pitch_aircraft@005e8de0`'s flier-target arm (`0x5e9114`..`0x5e9206`). who=1's Airbase on 800 and a
#          who=1 Bomber `1/6` standing on our side on 804 (domain 2, the arm's whole predicate on the target); a
#          Fighter `0/12` on 806, flown into `0/2007` on 810, struck at `1/6` on 902: on its strafe it wants
#          `1/6`'s figure `z` plus `min(300, max(0, (0x600 - dist)*5))` (6 < 12), capped at cruise over the ground.
#   Arm 1: `ObjectData::can_carry@006483c0:137`'s `num_aircraft_here(0)`, from `action_flight`'s `MOVE_TO` gate. An
#          Airbase `0/2008` on 820, ten Fighters `0/13`..`0/22` on 830 flown into it on 840; four struck out at
#          who=1's Airbase `1/2007` on 1000; on 1010 `@flight` home to `0/2008` names `0/6` (homed on `0/2007`) and
#          `0/13` (homed on `0/2008`, out): `0/13` turns home (its own home), and `0/6` is refused -- ten planes are
#          homed there though six stand inside.
# The staging was walked on run223's start (`RON_STAGE=ch17:chapter134:1400`): the squad's move on 791; `0/12`'s
# strafe on `1/6` from 932, `1/6` dead on 1194; `0/13`'s strike on 1001, home on 1011; `0/6` with no order to the end.
# Killers, before the take (a cheat on trace frame F shows on block F+2):
#   (1) any of `0/9`..`0/11` with no order on 793, or one that is not an attack/approach on `1/2006` -- no split;
#   (2) `0/12` with no `STRAFEORDER` on `1/6` by 940 -- the flier arm is never entered;
#   (3) `0/12`'s figure `z` above 1000 between 960 and 1180, while it strafes a target on the ground -- the arm is
#       not the reading's (the whole cruise is flown);
#   (4) `0/6` with any order, or outside `0/2007`, after 1012 -- the room counts the garrison alone;
#   (5) `0/13` with no `STRAFEORDER` home to `0/2008` (no target, `returning 1`) on 1012 -- the own-home arm.
#   run1549 (cover=0), chapter seventeen's lobby:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch134 \
#       --map 14 --end-frame 1400 --log-window 605 1400 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter134.cmd
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
780 add hoplite who=0 90,86
790 @strike 0 2006 1 7 9
800 add airbase who=1 110,60
804 add bomber who=1 40,60
806 add fighter who=0 60,80
810 @flight 0 2007 0 12
820 add airbase who=0 60,100
830 add 10 fighter who=0 60,94
840 @flight 0 2008 0 13 14 15 16 17 18 19 20 21 22
902 @strike 0 6 1 12
1000 @strike 0 2007 1 13 14 15 16
1010 @flight 0 2008 0 6 13
