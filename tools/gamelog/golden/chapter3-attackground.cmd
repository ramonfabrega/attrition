# Chapter three's lobby, plus one `@attackground` line (item 3425,
# docs/GOLDEN.md §145). A coverage staging, not a chapter: no word is pinned
# on it, and the walk skips the line by name (parked 3425a).
#
# `chapter3.cmd`'s six lines verbatim, then `630 @attackground 0 2328 9480 9`:
# `CommandManager::issue_attack_ground@009417a0(group 0/9, 2328, 9480,
# QUEUE_NEW)`, the manager method nothing in the image calls (§144.2: the
# button's `GroupOut::issue_attack_ground@0070ae60` builds the bytes inline).
# `0/9` is the Catapult, born on 621 at (792, 8328) (run145's dump on 625):
# SIEGE, so `UnitTypeData::can_attack_ground` answers, and the lobby carries
# no rush rules. The point is ten tiles off it, inside its 3–15, south-east,
# clear of the Hoplites at (2424, 7800), on the unowned arena (no peace tile).
#
# run1029, at `cover=1` on the click-free pool, chapter three's own recipe:
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch3ag \
#       --map 14 --end-frame 900 --log-window 605 900 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter3-attackground.cmd --cover cover=1
#
# WHAT WOULD FALSIFY IT, read before the run. (1) The `I_ISSUE` on 630 is
# refused, or the package grows by anything but 15 (a 5-byte `group` and the
# 10-byte command). (2) The dump's COMMANDMANAGER line on block 631 is not
# `process_attack_ground 2328 9480 2`. (3) The trace does not enter
# `process_attack_ground@009494a0` and `Group::action_attack_ground@00704520`
# on 630. (4) `rngcmp.py` against run145 differs on a frame before 630.

0 !ai off
600 age who=0 2
602 age who=1 2
610 add 3 chariot who=0 4,40
615 add hoplite who=1 12,40
620 add catapult who=0 4,41

630 @attackground 0 2328 9480 9
