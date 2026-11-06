# Golden record, chapter one hundred and seventeen -- Counterintelligence on an enemy Spy, cast by a Scout (item 3920,
# GOLDEN 167; AI 392.3's third staging, parked 3907a).
# Chapter ninety-one's lobby (GOLDEN 115, run894). There a Spy caster was refused on issue: a Spy is CLOAKED, seen by
# its enemy only through `seen3`, which a DETECT (`Z`) object writes. The Scout is both the craft's FROM2 and a
# DETECT unit (objmask FRPZC), so its own disc lights the who=1 Spy for who=0 and the cast is valid.
#   `SpellType::cast@00676ce0` case 0x277 asks `is_valid_target` again, then `cast_counterintel@00673740(spy, 1)`
#   with ecx = who 0: an enemy target -> the caster's `units_killed` += 1 and `score_combat` += cost / 5 (a
#   single-figure captain), then `Unit::die(2, -1, 0)` through vslot +0x158.
# Falsifiers, before the take (the journal has them): (1) no CASTORDER on the Scout two frames after the press -- the
# DETECT premise is false; (2) no `mana_burn` +500 on it; (3) the who=1 Spy not dead with `dtype` 2; (4) the cover=1
# twin enters no `cast_counterintel`.
#   run1514 (cover=0) and run1514b (cover=1, `--ai-tribe <the twin's who=1 tribe> --cover cover=1 --no-ffwd`):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch117 \
#       --map 14 --end-frame 900 --log-window 605 900 --timeout 3600 \
#       --detail end:UNITS=3,BUILDS=7,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter117.cmd
0 !ai off
600 library who=0 2
602 library who=1 2
606 add scout who=0 132,130
616 add spy who=1 134,130
620 @spell 0 631 6 1 25728 24960 6
