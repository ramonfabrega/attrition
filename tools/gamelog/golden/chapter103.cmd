# Golden record, chapter one hundred and three -- a shooter strikes a squad beside a Tank wreck, so `alarm` runs (item 3267, GOLDEN 138; parked 3266, 3235).
# Chapter 102 showed six stance-0 hoplites take no order from a Sabotaged Tank's wreck.  Here a who=1 bowman is added east of the
# hoplites at 800: it shoots one, and the struck unit's `alarm` (and the squad's response) walks the object chain beside the wreck.
#   RON_STAGE=ch76:chapter103:1500
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch103 \
#       --map 14 --end-frame 1500 --log-window 605 1500 --timeout 5400 \
#       --detail end:UNITS=3,BUILDS=7,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter103.cmd
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
800 add bowmen who=1 82,83
