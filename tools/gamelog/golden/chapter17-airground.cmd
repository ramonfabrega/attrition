# Chapter seventeen's lobby and an air attack-ground (item 3911,
# docs/GOLDEN.md §158; AI §391.2 row 2). Run 1505b.
#
# `chapter17.cmd`'s eleven lines verbatim, then one arm: `@attackground` on
# the Bomber `0/7`, on its patrol over the Barracks' point since 665
# (run223), at that point. `Group::action_attack_ground@00704520:119-168`
# hands a group whose leader is a plane to
# `Group::action_air_attack_ground@00703d80`, which lays
# `Unit::add_air_attack_ground_order@005e41c0`.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch17ag \
#       --map 14 --end-frame 930 --log-window 605 930 --timeout 5400 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --no-ffwd \
#       --cmd-file tools/gamelog/golden/chapter17-airground.cmd [--cover cover=1]
#
# WHAT WOULD FALSIFY IT, read before the run: no `process_attack_ground` on
# 681, or `0/7` with no `AIRATTACKGROUNDORDER` on 682; no `HIT` on
# `action_air_attack_ground` on 680's processed frame.
0 !ai off
600 library who=0 6
606 add airbase who=0 60,72
610 add fighter who=0 60,84
612 add bomber who=0 52,84
614 add bomber who=0 68,84
616 add barracks who=1 110,86
620 @strike 0 2006 1 7 8
640 @flight 0 2007 0 6
660 @flight 0 2007 0 7 8
664 @strike 0 2006 1 7 8
680 @attackground 0 21120 16512 7
