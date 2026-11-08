# Golden record, chapter one hundred and thirty-three -- the building defence remainder: a dock astride a border, and a
# Fort's upgraded reach with a General inside (item 3992, GOLDEN 185; parked 3970b; adjudication 4 row 64, adjudication 5
# row 36). Chapter one hundred and twenty-five's lobby (GOLDEN 175: the golden start, map 14, who=1 the British).
#   Arm 1: `Wall::process@00640450:209`-`:219` -- a dock (`is_dock`, the type's vslot `+0x108`) asks
#          `BuildTypeData::check_enemy_adjacent@00638d70` in place of the owner of its own cell: the ring one tile round
#          the footprint, corners and ocean tiles skipped, x outer and y inner; the first land tile that is unowned,
#          the dock's own player's or an ally's both ways ends the walk with no hit, and only a ring of rival land
#          bleeds. who=1's Dock `1/2006` at tile (59, 152) stands on who=0's water with one unowned shore tile in its
#          ring: the own-cell reading bleeds it 8 every 32 frames, the ring reading never. `1/2007` at (57, 154) is the
#          control: its ring is all who=0's land, and both readings bleed it.
#   Arm 2: `BuildTypeData::get_building_range@00639990`'s `FORT_UPGRADE_RANGE` at FORTRANGE2 (Fortification, bonus 40)
#          and `BuildData::max_range@0062e470`'s `GENERAL_BUILDING_RANGE` for a General inside: who=1's Fort `1/2008`
#          reaches 14 tiles, 13 with either term off. A Lookout `1/2009` lights the ground (a building's search takes
#          only what its owner sees), and who=0's Citizen `0/6` stands at `attack_dist` 2520 -- inside 14 tiles' 2688,
#          outside 13's 2496.
#   Arm 3 (not staged, parked 3992a): `in_enemy_territory@00638b40`'s `is_really_seen` arm asks a placer with no city
#          (`+0x3f8 == 0`); a golden lobby has none (`Leader::lost_a_city@006dc6d0` defeats the last city's loser).
# Killers, before the take (a cheat on trace frame F shows on block F+2):
#   (1) `1/2006`'s `damage` rises by 8 on a 32-frame step -- the ring did not replace the own cell (the reading is
#       wrong), or theirs' border differs from ours at the dock;
#   (2) `1/2007`'s `damage` never rises -- the control does not bleed: the dock reaches no enemy-land arm at all;
#   (3) the Fort never names `0/6` in `attack_ox` -- the staging did not reach the reach test;
#   (4) the Fort's first target block is later than the crate's (648) -- the reach is 13 tiles or less in theirs.
#   run1550 (cover=0), with chapter one hundred and twenty-five's lobby:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch133 \
#       --map 14 --end-frame 1000 --log-window 600 1000 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --ai-tribe 11 --profile DIFFICULTY=0 --profile STARTING_RESOURCES=1 \
#       --profile STARTING_TECHNOLOGY=0 --profile ENDING_TECHNOLOGY=7 \
#       --cmd-file tools/gamelog/golden/chapter133.cmd
0 !ai off
604 add dock who=1 59,152
606 add dock who=1 57,154
608 add fort who=1 200,104
610 tech who=1 fortification on
612 add general who=1 203,106
620 be 1
620 @garrison 1 2008 1 6
621 be 0
636 add lookout who=1 182,96
642 add citizen who=0 184,104
