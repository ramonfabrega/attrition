# Golden record, chapter one hundred and twenty-six -- the building family's five rows in one staging (item 3970, GOLDEN 176; T71-8).
# who=1 is the Maya (`--ai-tribe 1`). Its City `1/2000` takes a Temple, a Tower `1/2007` takes the Militia bit (byte 8 of the
# tech mask, `get_tower_fort_los` 2) and a General `1/6` garrisoned inside, and two of who=0's Chariots attack them:
# `0/6` the Tower, `0/7` the City. The terms each needs, and the crate's answer before this item:
#   `BuildData::attack@0062e610`: a General inside a Tower-line building adds GENERAL_BUILDING_ATTACK x 10 (120 -> 130);
#   `BuildData::get_base_garrison_arrows@0062e340`: a Mayan building adds MAYA_GARRISON_ARROWS to BASE_ARROWS (the Tower
#     1 -> 2; the Small City 0 -> 1, so it shoots ungarrisoned, its race being its owner);
#   `BuildTypeData::get_building_range@00639990`: TOWER_FORT_RANGE[1] = 1 on the Militia bit;
#   `BuildData::max_range@0062e470`: GENERAL_BUILDING_RANGE (1) for the General inside; TEMPLE_UPGRADE_RANGE[0] = 1 for a
#     templed city centre. So the Tower reaches 12 tiles, not 10, and the City 11.
# The Tower stands off the forest (`is_in_range@006486b0` refuses from a forest tile, the staging walk's first draft).
# Killers, before the take (the journal's list): (1) the Tower never names `0/6` in `attack_ox` -- the staging did not
# reach the family; (2) its first target block is the crate's 745, the 10-tile reach -- neither range term entered;
# (3) the City never names `0/7` -- the Maya arrow is not the city's, or `race` differs; (4) a Tower round's blow is
# the 120 attack's -- the General's term did not enter; (5) the cover=1 twin enters none of `BuildData::attack`,
# `get_base_garrison_arrows`, `get_building_range`, `get_temple_range`.
#   run1541 (cover=0), run1541b (cover=1, `--cover cover=1 --no-ffwd --twin ~/ron-golden/ch126`):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch126 \
#       --map 14 --end-frame 1100 --log-window 605 1100 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --ai-tribe 1 --profile DIFFICULTY=0 --profile STARTING_RESOURCES=1 \
#       --profile STARTING_TECHNOLOGY=0 --profile ENDING_TECHNOLOGY=7 \
#       --cmd-file tools/gamelog/golden/chapter126.cmd
0 !ai off
606 add temple who=1 226,90
608 add tower who=1 200,104
610 tech who=1 militia on
612 add general who=1 203,106
620 be 1
620 @garrison 1 2007 1 6
621 be 0
640 add chariot who=0 190,120
642 add chariot who=0 236,100
700 @attack 0 2007 1 6
702 @attack 0 2000 1 7
