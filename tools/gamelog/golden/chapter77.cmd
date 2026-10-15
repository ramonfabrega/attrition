# Golden record, chapter seventy-seven — the Porcelain Tower's ship arm held
# on a human (item 2598, `docs/GOLDEN.md` §96; `train_time@006508c0:316`–`340`,
# built by item 2526, AI §226, diff-backed before only on the peace pair's
# AI Space Program).
#
# THE STAGING is chapter seventy-three's wonder (a Small City, a Citizen, a
# site a probe found legal, `finish`) with the Porcelain Tower `0/533`
# (Gunpowder Age, `age who=0 3`) and a Dock `0/2007` at (53, 153), chapter
# forty-three's lake, in place of its Library. A Galley is queued at the Dock
# on 620, long before any Tower (the control, in the same run), and again on
# 1100, after the first is out, with the Tower finished on 701.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch77 \
#       --map 14 --end-frame 1400 --log-window 600 1400 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter77.cmd
#
0 !ai off
600 library who=0 1
601 age who=0 3
602 resource who=0 all +50000
603 add dock who=0 53,153
604 add small_city who=0 40,160
605 add citizen who=0 38,162
608 @build 0 7776 29472 533 6
620 @queueup 0 341 1 2007
700 select 2009 who=0
701 finish
1100 @queueup 0 341 1 2007
