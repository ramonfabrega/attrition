# Item 3871: the golden start (map 14, Great Lakes' default lobby) at cover=1 on the click-free pool, run1129's line
# and `--hover` with the console-planted contact added (`docs/MAPGEN.md` §19.6): `@aim` stores land cell (41, 42)'s
# centre in `ConsoleWin` `mouse_coord_x/y` (+0x518/+0x51c), since a `!` line runs `parse_cmd` with `no_mouse` 1 and
# leaves the cursor stale; `!weather` (`run_cmd` case 99) then calls `spot_update_land(41, 42, BASELAND, 3)`, whose
# re-mark runs `create_transition_renderables` in the same call. Predicted: `TerrainOut::check_for_diagonal@008595f0`
# entered on 620. `@dumpall` on 621 prints the block after the write whole, so the cell written can be read.
#
# The twin (run1130) is run1129's file at cover=0: every input before 620 shared, so `samegame.py --from 620`.
600 @cursor 70 33
620 @aim 41 42
620 !weather
621 @dumpall
