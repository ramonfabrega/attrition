# Golden record, chapter ninety -- `check_cast_target_path`'s last two arms
# and range-arrival's unseen kill (item 2949, `docs/GOLDEN.md` §114; §113
# left them: a Bribe's range (6 tiles) is inside a Spy's sight (8), so the
# unseen-arrival kill was never reached by run890).
#
# THE CASTERS (`craftrules.xml` / `unitrules.xml`): the Informer (`fbcml`,
# craft 0x27f = 639) reaches 10 tiles on a unit, 5 on a building; a Spy sees 8.
# So an Informer on a unit arrives at range with the target 8..10 tiles off,
# beyond its sight: a cast on a target the owner does not see.
#
# THE STAGING (walked on run871's start, `RON_STAGE=ch76:chapter90:1200`;
# `add` coordinates snap to land, so the numbers are not tiles):
#   arm 1, a building target lost mid-approach: Spy A `0/6` and Spy B `0/7`
#     are both sent to infiltrate the who=1 Barracks `1/2006`; A arrives
#     (walk: 754), infiltrates it (794), and B, still walking, finds the
#     target invalid at its next review (809): `check_cast_target_path`'s kill.
#   arm 2, an unseen target at range arrival: Spy C `0/8`, 5,500 units off,
#     is sent at the who=1 Hoplite `1/6` (a squad: `1/6`..`1/8`); it
#     arrives 2,120 units from it, sight is 1,536, and the walk kills the
#     cast on 841, the frame after (`do_cast`'s `is_seen`).
#   arm 3, a stealth-hidden target: Spy D `0/9` is sent at the who=1 Spy
#     `1/9`, idle and undetected; the walk kills it on arrival, 664.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch90 \
#       --map 14 --end-frame 1200 --log-window 605 1200 --timeout 3600 \
#       --detail end:UNITS=3,BUILDS=7,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter90.cmd
#
0 !ai off
600 library who=0 2
602 library who=1 2
606 add barracks who=1 80,80
608 add spy who=0 60,82
610 add spy who=0 20,82
612 add spy who=0 100,100
614 add spy who=0 120,130
616 add hoplite who=1 134,100
618 add spy who=1 134,130
620 @spell 0 639 2006 1 15360 15360 6
621 @spell 0 639 2006 1 15360 15360 7
622 @spell 0 639 6 1 25728 19200 8
623 @spell 0 639 9 1 25728 24960 9
