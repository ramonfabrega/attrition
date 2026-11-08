# Golden record, chapter one hundred and twenty-eight -- the patriots' armour beside a fight (item 3986, GOLDEN 179;
# the territory adjudication 4's rows 61 and 63). Chapter one hundred and twenty-seven's lobby (the golden start, map 14).
# The patriots and the struck units are who=0's (the human: no per-unit think marches them off at birth, as it did
# who=1's in the first walk); who=1's Hoplites strike them on explicit orders.
#   Arm 1: `UnitData::armor@00610160` -- the Hoplites `0/7..0/9` beside who=0's THECITIZEN (0x164, shipped as The
#          Comrade, Socialism's patriot; the booking's "Democracy Citizen" is the enum's name, not the unit's) add
#          THECITIZEN_ARMOR_BONUS (2) after the rally term.
#   Arm 2: the Heavy Knight `0/11` (Stable-trained, type +0x40 == 0x1ac) beside who=0's Monarch (0x162) adds
#          THEMONARCH_CAV_ARMOR (2) in the same arm.
#   Arm 3: `WallData::armor@0063fa60` -- the Barracks `0/2007`, within its Senator's (0x161) radius less the footprint's
#          (x_size + y_size) x 0x60, adds THESENATOR_BUILD_ARMOR (5) before the unfinished halving.
# Killers, before the take: (1) arm 1's first blow on a Hoplite equal to the crate's before this item (rally alone) --
# the Comrade's term does not enter; (2) arm 2's first blow on the knight equal to the rally-only blow; (3) the
# Barracks' first blow equal to the 5-less armour's; (4) no blow on any of the three -- the staging missed;
# (5) the cover=1 twin entering neither `UnitData::armor` nor `WallData::armor`.
#   run1544 (cover=0), run1544b (cover=1, `--cover cover=1 --no-ffwd --twin ~/ron-golden/ch128`):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch128 \
#       --map 14 --end-frame 1100 --log-window 605 1100 --timeout 3600 \
#       --detail end:UNITS=3,DEATHS=1,LEADERS=2,BUILDS=7 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter128.cmd
0 !ai off
606 add the_comrade who=0 100,100
608 add hoplite who=0 101,100
610 add the_monarch who=0 100,140
612 add heavy_knight who=0 102,140
614 add the_senator who=0 128,120
616 add barracks who=0 131,120
620 add hoplite who=1 110,100
622 add hoplite who=1 110,140
624 add hoplite who=1 140,120
640 be 1
640 @attack 1 7 0 6
640 @attack 1 11 0 9
640 @attack 1 2007 0 12
641 be 0
