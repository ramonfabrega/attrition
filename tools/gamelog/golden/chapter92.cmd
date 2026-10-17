# Golden record, chapter ninety-two -- Sabotage's building arm and its
# refused unit arm (item 2994, `docs/GOLDEN.md` §116; `SpellType::
# cast_sabotage@006720e0`'s building arm and `SpellTypeData::
# is_valid_target@006763c0`'s 0x280 arm, refused by this crate until this
# chapter).
#
# THE CASTER (`craftrules.xml`): Sabotage (`fcm`, craft 0x280 = 640) is
# cast by a Commando on an enemy building at a one-tile range; on a unit
# `is_valid_target` asks for objmask 0x200000 (a vehicle), so a foot soldier
# is refused. A building takes half its `hits_left`, floored at
# COMMANDO_MIN_DAMAGE (200), as an attrition-flagged `take_damage` of dtype 3.
# Pilfer (0x276) has `FROM None`: no unit can be given the cast, so its arm
# is a reading and is not staged.
#
# THE STAGING (walked on run871's start, `RON_STAGE=ch76:chapter92:1200`;
# `add` coordinates snap to land):
#   arm 1, a building razed: Commandos `0/6`..`0/9` are all sent at a
#     who=1 Barracks `1/2006` (1,200 hits; a Farm cannot be `add`ed): each
#     blow is half of what is left, floored at 200 -- 600, 300, then 200
#     twice, the fourth taking the last 100.
#   arm 2, a unit refused: Commando `0/10` at the who=1 Hoplite `1/6`.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch92 \
#       --map 14 --end-frame 1200 --log-window 605 1200 --timeout 3600 \
#       --detail end:UNITS=3,BUILDS=7,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter92.cmd
#
0 !ai off
600 library who=0 2
602 library who=1 2
606 add barracks who=1 80,80
608 add commando who=0 70,82
609 add commando who=0 72,84
610 add commando who=0 70,86
611 add commando who=0 72,88
612 add commando who=0 100,130
616 add hoplite who=1 134,100
620 @spell 0 640 2006 1 15360 15360 6
621 @spell 0 640 2006 1 15360 15360 7
622 @spell 0 640 2006 1 15360 15360 8
623 @spell 0 640 2006 1 15360 15360 9
624 @spell 0 640 6 1 25728 19200 10
