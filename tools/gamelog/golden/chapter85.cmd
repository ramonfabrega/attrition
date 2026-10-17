# Golden record, chapter eighty-five — the cloaked Spy's Bribe (item 2832,
# `docs/GOLDEN.md` §108; `SpellType::cast_bribe@00672d00`'s unit arm and
# `Unit::swap_team@006118d0`, built by item 2823, §107, held by no walk).
#
# THE STAGING (walked on run871's start, `RON_STAGE=ch76:chapter85:1300`):
# both sides in the Medieval age. A who=0 Spy `0/6` and a who=1 Hoplite
# `1/6` beside it, with a who=1 Scout (`DETECT`) in whose sight both stand;
# and, 60 tiles off, a second Spy and Hoplite nothing of who=1's sees — the
# control. Bribe (`fcbhm`, craft 0x275 = 629) is cast on each Hoplite.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch85 \
#       --map 14 --end-frame 1260 --log-window 1100 1260 --timeout 3600 \
#       --detail end:UNITS=3,LEADERS=2,DEATHS=1,BUILDS=7 --detail start:MISC,UNITS=3,BUILDS=7,LEADERS=9 --cmd-file tools/gamelog/golden/chapter85.cmd
#
0 !ai off
600 library who=0 2
602 library who=1 2
606 add spy who=0 190,60
608 add spy who=0 120,100
1100 add hoplite who=1 196,57
1101 add scout who=1 198,56
1102 add hoplite who=1 122,102
1112 @spell 0 629 6 1 37600 11300 6
1112 @spell 0 629 10 1 23600 19200 7
