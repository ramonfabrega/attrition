# Golden record, chapter one hundred and five -- a Tower strikes a squad that is walking toward it (item 3292, GOLDEN 142; parked 3291, 3283).
# Chapter 104 (Tower, stationary squad) closed on its first take.  Here both hoplite captains are ordered east toward the Tower at 806, so the
# squad is walking (a move front) when struck: `Unit::target_opportunity`'s move-front proximity arm (parked 3283).
# Chapter 103 had a who=1 bowman strike the six stance-0 hoplites beside a Sabotaged Tank's wreck.  Here the shooter is a who=1 Tower
# (a building, the other arm of the wreck's chain: the building shooter's chain readers and near-face gate) added east of the hoplites at 800.
#   RON_STAGE=ch76:chapter105:1500
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch105 \
#       --map 14 --end-frame 1500 --log-window 605 1500 --timeout 5400 \
#       --detail end:UNITS=3,BUILDS=7,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter105.cmd
0 !ai off
600 library who=0 2
602 library who=1 2
608 add commando who=0 71,83
609 add commando who=0 73,83
610 add commando who=0 72,82
611 add commando who=0 72,84
616 add tank who=1 72,83
620 @spell 0 640 6 1 13824 15936 6
621 @spell 0 640 6 1 13824 15936 7
622 @spell 0 640 6 1 13824 15936 8
623 @spell 0 640 6 1 13824 15936 9
790 add hoplite who=0 74,84
791 add hoplite who=0 75,82
800 add tower who=1 80,83
806 @move 0 14592 16032 10
806 @move 0 14592 16008 13
