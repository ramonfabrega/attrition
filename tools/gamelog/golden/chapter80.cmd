# Golden record, chapter eighty — the stealth arm of `UnitData::is_seen`
# on a human's units under a computer's `calc_danger` (item 2755,
# `docs/GOLDEN.md` §102; `is_seen@00607a60`, `is_detected@0060a630`,
# `World::seen3`, built by item 2725, §101, and held by no walk until now).
#
# THE STAGING (three sites, twelve tiles apart on run871's start,
# `RON_STAGE=ch76:chapter80:1210`): each is a who=0 unit with a who=1
# hoplite two tiles off, whose sight lights the cell for leader 1 and,
# being no `DETECT` object, writes no `seen3`.
#   site 1 (176,60): a Commando (`FLAGS s`, idle) — lit, never detected:
#     the arm hides it, so `calc_danger`'s unit pass halves its weight.
#   site 2 (190,60): a Commando, and a Scout (`DETECT`) added for who=1
#     on 1100 — hidden at the 1000 pass, detected by the 1200 one.
#   site 3 (204,60): a Hoplite, the plain control — weighed whole.
# `calc_danger` runs on every 200th frame (`danger::PERIOD`); the window
# prints `WORLD=6` (the danger plane and `seen3`) about 1200.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch80 \
#       --map 14 --end-frame 1204 --log-window 1198 1204 --timeout 3600 \
#       --detail end:WORLD=6,UNITS=3,BUILDS=7,CITIES=5,LEADERS=2,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter80.cmd
#
0 !ai off
606 add airbase who=1 190,60
608 add partisan who=0 190,74
610 add partisan who=0 204,60
612 add hoplite who=0 183,64
1197 add scout who=1 206,62
