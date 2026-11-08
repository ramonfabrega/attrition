# Golden record, chapter one hundred and twenty-nine -- the patriot's repair and the garrison heal's level (item 3987,
# GOLDEN 180; parked 3961b, 3959a). Chapter one hundred and twenty-eight's lobby (the golden start, map 14). The
# patriots, the struck buildings and the wounded squad are who=0's (the human); who=1's Hoplites strike them.
#   Arm 1: `Build::process@0061edf0`'s patriot repair, the Senator's branch -- the Barracks `0/2007` on who=0's own
#          ground beside its Senator (0x161) takes one point on FRENCH_SUPPLY_HEAL_RATE's phase (o + frame) % 20 and
#          writes `healing` 20.
#   Arm 2: the same arm, the President's branch -- the Barracks `0/2008` on unowned ground beside its President
#          (0x163): the President repairs where the tile owner is negative, the Senator and the CEO only on allied
#          ground (run1544's Barracks beside a Senator on unowned ground is never repaired).
#   Arm 3: `Unit::process_healing@005e0670`'s inside arm at heal level 1 -- who=0 holds Herbal Lore, so its wounded
#          Hoplites garrisoned in its capital `0/2000` heal on UNIT_HEAL_RATE[1] = 15 frames, not 20.
# Killers, before the take: (1) the Barracks `0/2007` damaged and on the phase while `healing` prints 0 -- the
# Senator's branch does not repair on allied ground; (2) the Barracks `0/2008` damaged with `healing` 0 throughout --
# the President's branch does not repair on unowned ground; (3) a garrisoned Hoplite of arm 3 printing `healing` 20
# (or 16..19 the frame after a heal) -- the level is not read; (4) no blow on any of the three -- the staging missed.
# Walked first (RON_STAGE=ch128:chapter129:900 RON_STAGE_ALL=1): 0/2007 struck from 635, repaired on 653 (healing 20);
# 0/2008 struck on 669, repaired on 672; 0/8 garrisoned on 853, healed on 862 (healing 15) and every 15 after.
#   run1545 (cover=0), run1545b (cover=1, `--cover cover=1 --no-ffwd --twin ~/ron-golden/ch129`):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch129 \
#       --map 14 --end-frame 1100 --log-window 605 1100 --timeout 3600 \
#       --detail end:UNITS=3,DEATHS=1,LEADERS=2,BUILDS=7 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter129.cmd
0 !ai off
600 tech who=0 herbal_lore on
606 add barracks who=0 36,152
608 add the_senator who=0 33,152
610 add barracks who=0 131,120
612 add the_president who=0 128,120
614 add hoplite who=0 30,172
620 add hoplite who=1 39,152
622 add hoplite who=1 140,120
624 add hoplite who=1 38,172
640 be 1
640 @attack 1 2007 0 6
640 @attack 1 2008 0 9
640 @attack 1 8 0 12
641 be 0
700 @garrison 0 2000 0 8
