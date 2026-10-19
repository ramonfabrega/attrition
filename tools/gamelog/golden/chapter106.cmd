# Golden record, chapter one hundred and six -- a who=1 Fort strikes a stationary squad beside a Tank wreck (item 3294, GOLDEN 143; parked 3293, 3283).
# Chapters 104 and 105 had a Tower as the building shooter; here a Fort stands in its place, so the Fort line's near-face gate (never diffed) is read.
# Chapter 105's squad walked and its own attack order took over before a round landed, so the squad stays still here (chapter 104's recipe).
#   RON_STAGE=ch76:chapter106:1500
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch106 \
#       --map 14 --end-frame 1500 --log-window 605 1500 --timeout 5400 \
#       --detail end:UNITS=3,BUILDS=7,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter106.cmd
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
800 add fort who=1 81,83
