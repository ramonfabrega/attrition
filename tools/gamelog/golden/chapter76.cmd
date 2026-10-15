# Golden record, chapter seventy-six — the President's train arm held on a
# human (item 2572, `docs/GOLDEN.md` §95; `train_time@006508c0:103–107`,
# built by item 2491, AI §225, diff-backed before only on the peace pair's
# AI Airbase).
#
# THE STAGING: `age who=0 3`, a Senate `0/2007` and a Barracks
# `0/2008`. A first walk queued Democracy (626) at 610 and the queue refused
# it: a second-tier government needs one of the first tier (`docs/TECH.md`,
# "Governments pair and tier"). So Republic (624) on 610 (400 frames, the
# Senator born at the finish), Democracy on 1030 (700 frames), and
# `Build::finished`'s Senate tail `set_type`s the Senator into The President.
# THE CONTROL IN THE SAME RUN: Hoplites queued at the Barracks on 620, long
# before any President; a second batch on 1750, with the President born.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch76 \
#       --map 14 --end-frame 2100 --log-window 600 2100 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,DEATHS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter76.cmd
#
0 !ai off
600 age who=0 3
601 resource who=0 all +5000
602 add senate who=0 14,74
604 add barracks who=0 22,74
610 @queueup 0 624 1 2007
620 @queueup 0 132 1 2008
1030 @queueup 0 626 1 2007
1750 @queueup 0 132 1 2008
