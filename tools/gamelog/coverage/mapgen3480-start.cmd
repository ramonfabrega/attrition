# Item 3480: the golden start (map 14) at cover=1 on the click-free pool, regenerated mid-game in four map styles no
# capture has traced, to enter `TerrainOut::check_for_diagonal@008595f0` and through it the `int` overload of
# `WorldData::is_valid@00471170`. A sweep, not a prediction: which style leaves a listed transition cell with a flags
# word of 0 after `build_trans_flags`/`check_blocks`/`check_connects`/`check_fills` is not read.
#
# `run_cmd` case 0x5a (`mapstyle`, console-only, so `!`): a number falls through `parse_map_style`'s name match to
# `convert_int`, clamped to 0..0x16 into `game->info.map_style` (indices: `mapstyle.py --list`). Case 0x5b (`mapgen`):
# gated on `Game::is_solo`; a bare `mapgen` takes `timeGetTime()` as the seed, so every line here carries an argument;
# then `Setup::build_game` (or `Game::close`/`Game::init` when `map_dirty`), `WorldMap::reinit_all`, and
# `TurnControl::set_pause(1)`. A paused game stops calling `do_frame`, so no frame-keyed line can unpause it: each
# `mapgen` is preceded by an `@after` press of the pause key (`--bindkey OPTION_PAUSE=19`), which runs from
# `Game::loop`'s entry; `issue_toggle_pause` toggles directly when `game->semaphore & 0x10`.
#
#   21 Nile Delta, 13 Australian Outback, 20 Atlantic Sea Power, 11 African Watering Hole
#
# The twin is the frames before the first `mapgen` (0..5) against run1093 (the golden start, `0 !ai off`, the same
# `--cursor`): past frame 6 the take is a different map by construction.
0 !ai off
5 !mapstyle 21
6 @after 2000 key 19
6 !mapgen quick
15 !mapstyle 13
16 @after 2000 key 19
16 !mapgen quick
25 !mapstyle 20
26 @after 2000 key 19
26 !mapgen quick
33 !mapstyle 11
34 @after 2000 key 19
34 !mapgen quick
