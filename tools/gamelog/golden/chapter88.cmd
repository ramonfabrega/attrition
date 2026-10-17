# Golden record, chapter eighty-eight -- a Bribe whose target turns invalid
# on the approach (item 2915, `docs/GOLDEN.md` §112; §110/§111 built
# `check_cast_target_path` and watched its re-target and unseen arms, and
# left the kill arm: a *seen* target no longer valid ends the walk and the
# cast, `repath` + `kill_current_order`, `check_target_path@005e22d0`'s
# action-0xe arm).
#
# THE STAGING (walk on run871's start, `RON_STAGE=ch76:chapter88:1500`):
# two who=0 Spies `0/6` (8 tiles from the target) and `0/7` (28 tiles) are
# both ordered to Bribe one who=1 Hoplite `1/6` at 1112. `0/6` arrives and
# converts it; the Hoplite is then who=0's own unit, `is_valid_target`
# fails for `0/7`'s still-walking cast, and its next review
# (`(frame + 7) & 0xf == 0`) takes the kill arm.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch88 \
#       --map 14 --end-frame 1500 --log-window 1100 1500 --timeout 3600 \
#       --detail end:UNITS=3,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter88.cmd
#
0 !ai off
600 library who=0 2
602 library who=1 2
608 add spy who=0 120,100
609 add spy who=0 100,100
1100 add hoplite who=1 128,100
1112 @spell 0 629 6 1 23600 19200 6
1112 @spell 0 629 6 1 23600 19200 7
