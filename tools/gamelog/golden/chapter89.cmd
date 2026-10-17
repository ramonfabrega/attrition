# Golden record, chapter eighty-nine -- a Bribe that completes on a target
# outside the caster's owner's sight (item 2930, `docs/GOLDEN.md` §113;
# §111 left (3): run885's Spy, 22 tiles off, never reached range in 400
# frames, so whether range-arrival itself asks `is_seen` was untested).
#
# THE STAGING (walk on run871's start, `RON_STAGE=ch76:chapter89:1500`):
# chapter eighty-seven's staging with the Hoplite 14 tiles off: one who=0
# Spy `0/6`, one who=1 Hoplite `1/6`, AI off; Bribe is cast on it at 1112.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch89 \
#       --map 14 --end-frame 1500 --log-window 1100 1500 --timeout 3600 \
#       --detail end:UNITS=3,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter89.cmd
#
0 !ai off
600 library who=0 2
602 library who=1 2
608 add spy who=0 120,100
1100 add hoplite who=1 134,100
1112 @spell 0 629 6 1 23600 19200 6
