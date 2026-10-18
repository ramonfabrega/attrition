# Golden record, chapter ninety-four -- Sabotage on an armed vehicle (item 3067,
# `docs/GOLDEN.md` section 118; `SpellType::cast_sabotage@006720e0`'s unit
# arm with `dtype` 3, the boom, and `SpellTypeData::is_valid_target@006763c0`'s
# 0x280 arm: a unit target needs objmask 0x200000 = V).
#
# THE STAGING (chapter 91's start, `RON_STAGE=ch76:chapter94:1200`):
#   arm 1, a vehicle blown up (Artillery, armed with a minimum range; a Tank shot the Commando dead (parked 3065)): Commando `0/6` (70,82) is sent at the who=1
#     Artillery `1/6` (72,83), beside a Barracks at (80,80) whose border the
#     chapter-92 Commandos survived, Sabotage 0x280.
#   arm 2, a second vehicle: Commando `0/7` (72,88) at the Artillery `1/7`
#     (74,89).
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch94 \
#       --map 14 --end-frame 1200 --log-window 605 1200 --timeout 3600 \
#       --detail end:UNITS=3,BUILDS=7,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter94.cmd
#
0 !ai off
600 library who=0 2
602 library who=1 2
606 add barracks who=1 80,80
608 add commando who=0 70,82
609 add commando who=0 72,88
616 add artillery who=1 72,83
617 add artillery who=1 74,89
620 @spell 0 640 6 1 13824 15936 6
621 @spell 0 640 7 1 14208 17088 7
