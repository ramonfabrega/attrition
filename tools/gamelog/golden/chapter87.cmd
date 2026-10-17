# Golden record, chapter eighty-seven's second arm -- a Bribe on a target
# outside the caster's owner's sight (item 2895, `docs/GOLDEN.md` §111;
# §110 built the chase's re-target and left parked 2879: the review's
# `is_seen` gate has never been watched on a target who=0 does not see).
#
# THE STAGING (walk on run871's start, `RON_STAGE=ch76:chapter87:1500`):
# chapter eighty-six's control with the Hoplite put far from the Spy. One
# who=0 Spy `0/6`, one who=1 Hoplite `1/6` 22 tiles off, AI off so it
# stays; Bribe (`fcbhm`, craft 0x275 = 629) is cast on it at 1112, so the
# Spy walks to range and the review (`(frame + o) & 0xf == 0`) asks
# `is_seen` while the target is still in who=0's fog.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch87 \
#       --map 14 --end-frame 1500 --log-window 1100 1500 --timeout 3600 \
#       --detail end:UNITS=3,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter87.cmd
#
0 !ai off
600 library who=0 2
602 library who=1 2
608 add spy who=0 120,100
1100 add hoplite who=1 142,100
1112 @spell 0 629 6 1 23600 19200 6
