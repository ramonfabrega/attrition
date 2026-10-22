# Item 3483: where a dumped frame's seconds go. The golden start (Great Lakes,
# map 14, seed 12345) with run1084's first line and none of its window's
# console lines, so the window's frames are the dump and nothing else; run1084
# (`console3464-start.cmd`) is the `rngcmp` twin to 606, its 600/602 console
# lines drawing nothing (run1084 against run1031b: 701 identical).
#
# The clock is `rontrace.cfg`'s `dumptime=` (tools/trace/dumptime.h), carried
# on the `--cover` line:
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/run1101 \
#       --map 14 --end-frame 606 --log-window 600 605 --timeout 3600 \
#       --detail end:UNITS=3,LEADERS=9 --cursor 1400,1000 \
#       --cmd-file tools/gamelog/coverage/dumptime3483.cmd --cover $'cover=1\ndumptime=2'
#
# run1111: `--detail end:LEADERS=9`, `dumptime=1`; run1112: run1101's line with
# `--stride 2`, `dumptime=1`.
0 !ai off
606 !quit
