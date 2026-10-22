# Item 3445: chapter seventy-four's lobby (run864's lines to 640) at cover=1 on the click-free pool, Options ->
# save mid-game with the Auto Citizen option changed, for `OptionsWinGame::do_save@0082c520` (ORDERS §42.1's
# falsifier: `process_leader_options` after the save, the leader-option record agreeing with the window).
# The Game Menu pauses the game: in a solo game `Game::loop@00591570` skips `do_frame` while a modal window is
# up, so every click after the menu's is an `@after MS` line, run from `Game::loop`'s entry on the wall clock
# (the windows animate in, `GravWindow::animate`).
# Points: the menu button (1899, 11) and the menu's Options (959, 455) and Return to Game (959, 414) are
# run1032's driven screenshot (`/tmp/ron-runs/hand-1032/menus.png`, x 1.3714). The Options window is read off
# `Data/optionswin.xml`, not a screenshot: OPTIONSWIN 880x600 centred (`OptionsWin::exec@0082a6e0`,
# `Window::center`) at (520, 240); pane 0 (`current_pane` 0, the Game pane) at (42, 42); the Auto Citizen
# group at (9, 10) + (24, 20), rows 19 apart: Idle (id 3) the first row, (602, 318); OK at (25, 497) +
# (17, 16), 258x30: (691, 768). The start record holds `peasants 0` (Gather), so Idle changes it to 3.
# `@dumpall` makes 700 (before) and 712 (after) whole dumps: the leader-option record prints only there.
# The first unshared input is 700 (run864's `select 2005`); `samegame.py --from 700`.
0 !ai off
600 age who=0 8
601 resource who=0 all +50000
640 @queueup 0 572 1 2005
700 @dumpall
702 @click 1899 11
702 @after 3000 click 959 455
702 @after 9000 click 602 318
702 @after 11000 click 691 768
702 @after 14000 click 959 414
# If a point missed and a window is still up: Escape twice, then Return to Game again.
702 @after 20000 key 27
702 @after 22000 key 27
702 @after 24000 click 959 414
712 @dumpall
