# Item 3484: the console mirror off beside a twin at the default, on the
# three-player team lobby (herd3472-team.cmd's: Great Lakes, map 14,
# `--profile PLAYERS=2`, `ally 1`, `--ffwd-minute 6`). herd3472's console
# lines without its click (the cursor is pinned off the map) or its `@dumpall`s.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/run1107 \
#       --map 14 --end-frame 5412 --log-window 5405 5411 --timeout 3600 --ffwd-minute 6 \
#       --profile PLAYERS=2 --detail end:UNITS=3,LEADERS=9 --detail misc:COMMANDMANAGER=1 \
#       --cursor 1400,1000 --cmd-file tools/gamelog/coverage/console3484-team.cmd \
#       --cover cover=1 --dumptime 1 --console-log 0
#
# run1108: the same line without `--console-log`.
100 ally 1
5404 explore all
5406 camera 186,190
