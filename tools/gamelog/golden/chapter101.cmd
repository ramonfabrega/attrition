# Golden record, chapter one hundred and one -- an idle squad beside a Tank wreck, no Barracks to draw it away (item 3253, GOLDEN 136; parked 3235).
# The original leaves a `blocks_while_dead` wreck on the object chain until the death object's cull, so a reader that
# walks the chain without testing `alive` (`find_nearby_target`, `find_units`, `alarm`) sees the Tank's wreck.  This crate
# keeps the wreck off the chain.  Chapter 99's Sabotage kills who=1's Tank at ~800; two who=0 hoplite squads are added
# beside the wreck and left idle (no orders), to search.
#   RON_STAGE=ch76:chapter101:1500
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch101 \
#       --map 14 --end-frame 1500 --log-window 605 1500 --timeout 5400 \
#       --detail end:UNITS=3,BUILDS=7,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter101.cmd
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
