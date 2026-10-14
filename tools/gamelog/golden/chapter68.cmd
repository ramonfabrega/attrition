# Golden record, chapter sixty-eight — Socialism's price and a live
# military discount: the arms the AI track built that no walk holds (item
# 2357, `docs/GOLDEN.md` §85; parked 2333, 2334, DECISIONS 56 §3).
#
# Three arms of one staging:
#
# 1. `TypeData::get_cost@00664090:666`-`672`, the train arm after
#    Monarchy's: a unit whose trainer (`UnitTypeData +0x40`) is the Siege
#    Factory (`0x1ae`), the Airbase or the Dock, by equality, of a player
#    holding `SOCIALISM_1` takes `(100 - SOCIALISM_UNIT_BONUS) x cost /
#    100`. Built by this item; `train_time`'s twin (2293) is diff-backed on
#    the Airbase only, and here it runs on a Siege Factory.
# 2. `get_cost:632`-`665`: `MILITARY_UNIT_DISCOUNT` (5% a Military level
#    the player stands past the unit's), then Monarchy's quarter on a
#    Stable unit, each its own truncation. run845's discount was 0; here
#    `military who=0 3` stands two past the Light Horse's level 1
#    (`UnitTypeData::get_military_level_slow@0061d4d0`, never backed).
# 3. The order: Light Horse A's ramped food is 70, and 70 -> 63 -> 47 in
#    the listing's order, 70 -> 52 -> 46 the other way round.
#
# Catapult (TypeIndex 265, unit 215, 7m/7t, `JOB_TIME` 250) needs the
# Classical Age and The Art of War; Light Horse (209, unit 159, 4t/6f,
# 175). Four Light Horse are added so A's ramp lands on the parting price.
#
# THE STAGING, walked by this crate on run492's start (RON_STAGE=ch45:
# chapter68:1300, RON_STAGE_ALL), trace frames:
# - 602: the Stable `0/2007` on (14, 74); 604 the Siege Factory `0/2008`
#   on (24, 74). 612: Light Horse `0/6`-`0/9`.
# - 614: Light Horse A queued at **47 food 33 timber** (70/50 ramped, 10%
#   then 25%). 616: Catapult A at 63 timber 63 metal (70, 10%).
# - 800: Socialism gained. 802: Catapult B at **60/60** (90 ramped, 10%,
#   then 25%). 804: Light Horse B at 50/36 (75/55; Monarchy still held,
#   Socialism takes nothing off a Stable unit).
# - 807: A out, `0/10`. 857: Catapult A out, `0/11` (Socialism's time arm
#   on a build in progress). 1006: B out, `0/12`. 1104: Catapult B out,
#   `0/13`.
#
# CENSUS ROWS the staging reaches, never backed (`census.py --unbacked`):
# `UnitTypeData::get_military_level_slow@0061d4d0`; and the arms of
# `get_cost` and `train_time` no walk has held.
#
# WHAT WOULD FALSIFY IT, read before the run. A call on trace frame F is on
# block F+2.
# 1. Socialism's price: Catapult B's entry at 81/81 on block 804 (no arm),
#    or anything but 60/60.
# 2. The discount and its order: A's entry at 46 food (the other order),
#    45 (one fold of 35%), 52/37 (no military discount), on block 616.
# 3. Socialism's time on a Siege Factory: Catapult A out off 857, or B
#    off 1104.
# 4. Equality: Light Horse B at anything but 50/36 on block 806.
#
#   run852 (item 2357), the click-free lane:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch68 \
#       --map 14 --end-frame 1360 --log-window 600 1360 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter68.cmd
#
# The window: the last falsifier is Catapult B's birth, 1104; 1360 leaves
# 256 past it.
0 !ai off
600 library who=0 1
601 resource who=0 all +3000
602 add stable who=0 14,74
604 add siege_factory who=0 24,74
606 tech who=0 the_art_of_war on
608 military who=0 3
610 tech who=0 monarchy on
612 add 4 light_horse who=0 14,80
614 @queueup 0 209 1 2007
616 @queueup 0 265 1 2008
800 tech who=0 socialism on
802 @queueup 0 265 1 2008
804 @queueup 0 209 1 2007
