# Item 3484: the console mirror off (`--console-log 0`, rise.ini
# `AllowLogs_ToConsole=0`) beside a twin at the default, on the coverage
# pair's lobby (eval3471.cmd's: map 18, Persian against Inca at All Tech).
# eval3471's lines to 610, so the window holds one `game_random` draw off the
# console (`!eval rand_real()`), and nothing past it; 3484 times the dump.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/run1105 \
#       --map 18 --end-frame 612 --timeout 2400 --ai-tribe 23 \
#       --profile STARTING_TECHNOLOGY=8 --profile STARTING_RESOURCES=7 --profile DIFFICULTY=5 \
#       --log-window 606 611 --detail end:MISC,UNITS=3,LEADERS=9 --detail misc:COMMANDMANAGER=2 \
#       --cursor 1400,1000 --cmd-file tools/gamelog/coverage/console3484-pair.cmd \
#       --cover cover=1 --dumptime 1 --console-log 0
#
# run1106: the same line without `--console-log`.
600 add aircraft_carrier who=0 6,37
602 add hoplite who=0 28,44
610 !eval rand_real()
