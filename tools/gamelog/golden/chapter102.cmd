# Golden record, chapter one hundred and two -- an aggressive squad searches beside a Tank wreck (item 3263, GOLDEN 137; parked 3262, 3235).
# Chapter 101's idle hoplites (stance 1 in the dump) never searched, so the wreck on the object chain stayed unread.  Here the
# same staging sets both hoplite squads to option 0 (aggressive) at 800, after the wreck is made: a standing search then walks
# the chain with the wreck (a `blocks_while_dead` Tank, who=1) the only object in reach.
#   RON_STAGE=ch76:chapter102:1500
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch102 \
#       --map 14 --end-frame 1500 --log-window 605 1500 --timeout 5400 \
#       --detail end:UNITS=3,BUILDS=7,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter102.cmd
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
800 @stance 0 0 6 7 9
