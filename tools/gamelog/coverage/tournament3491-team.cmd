# Item 3491: `Game::is_tournament@0043ec90` from a defeated computer's chair. run1086's lobby (Great Lakes, map 14,
# seed 12345, `--profile PLAYERS=2`) with **no** `ally 1` line, so the three leaders are a free-for-all, and `--no-ffwd`
# so every frame is drawn. Each of `is_tournament`'s 50 call sites needs `semaphore & 0x42` clear *and*
# `Player::is_over(console->play)`; the pool's own quit gives the second only on a frame nothing is drawn after.
#   600  the script host's `defeat(3)` (1-based: leader 2) is `ScenarioFuncSet::defeat@009f3f50` -> `Leader::defeat`,
#        with no `EndGameWin::exec`, so the game does not quit (the console's own `defeat` does). `!` because `eval`
#        is in `run_cmd`'s first switch (3471).
#   602  `be 2` (run_cmd case 0x2b, a chat command, so a plain line): the console's chair moves to the defeated leader.
#   604  `reveal 0` (case 0x57) clears semaphore bit 1 (run1109).
# On disk (run1086's first dump): leader_flags 7, 19, 19; PLAYER who 2 `flags 1`. Predicted on 601: who 2's
# leader_flags 19 -> 81, PLAYER flags 1 -> 129.
# Both takes stage `--cursor 1400,1000` (off the map: Loop 3432). The twin is this file at cover=0, every input shared.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/run1113 \
#       --map 14 --end-frame 640 --log-window 598 640 --timeout 3600 --profile PLAYERS=2 --no-ffwd \
#       --detail end:UNITS=3,LEADERS=9 --detail misc:COMMANDMANAGER=1 --cover cover=1 \
#       --cursor 1400,1000 --cmd-file tools/gamelog/coverage/tournament3491-team.cmd
598 @dumpall
600 !eval defeat(3)
602 be 2
604 reveal 0
606 @dumpall
620 @dumpall
