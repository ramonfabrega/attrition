# Golden record, chapter one hundred -- a Tank bled to death by attrition releases its disc (item 3236, GOLDEN 132; parked 3190).
# `Unit::close@0060ee50` passes the death type 0 for an attrition death, so `Object::close@00647160` takes the wreck's
# `blocks_while_dead` disc out of the occupancy index at once (chapter 99's Sabotage death, dtype 3, keeps it).
#   RON_STAGE=ch76:chapter100:2600
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch100 \
#       --map 14 --end-frame 2600 --log-window 605 2600 --timeout 5400 \
#       --detail end:UNITS=3,BUILDS=7,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter100.cmd
#
# Staging (walked on ch76's start): who=1 holds Allegiance..Nationalism; who=0's Tank 0/6 stands at (41400,19320) in
# who=1's land and bleeds to 0 by 1890 in this crate; two hoplite squads (captains 0/7, 0/10) added at 1900/1901 are
# sent west across its cell on 1903.
0 !ai off
600 library who=0 2
602 library who=1 2
604 tech who=1 allegiance on
605 tech who=1 oath_of_fealty on
605 tech who=1 patriotism on
605 tech who=1 nationalism on
616 add tank who=0 215,100
1900 add hoplite who=0 222,100
1901 add hoplite who=0 222,102
1903 @move 0 40104 19176 7
1903 @move 0 40104 19200 10
