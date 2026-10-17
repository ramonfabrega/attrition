# Golden record, chapter eighty-six — the cloaked Spy's Bribe, the control
# that reaches range (item 2876, `docs/GOLDEN.md` §109; chapter eighty-five,
# §108, closed the conversion and left the control short: its target walked
# off after the blown first Spy, parked 2868, 2870).
#
# THE STAGING (walked on run871's start, `RON_STAGE=ch76:chapter86:1300`):
# the control alone. No Scout, no sighted pair, no Spy blown -- so nothing
# of who=1's is ordered to attack toward a cast, and the Hoplite added at
# 1100 stays where it was put. One who=0 Spy `0/6` (cast waits for
# `mana_burn` 0, §108), 2 tiles from one who=1 Hoplite `1/6`; Bribe
# (`fcbhm`, craft 0x275 = 629) is cast on it at 1112.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch86 \
#       --map 14 --end-frame 1300 --log-window 1100 1300 --timeout 3600 \
#       --detail end:UNITS=3,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter86.cmd
#
0 !ai off
600 library who=0 2
602 library who=1 2
608 add spy who=0 120,100
1100 add hoplite who=1 122,102
1112 @spell 0 629 6 1 23600 19200 6
