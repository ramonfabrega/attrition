# Item 3889: the golden start (map 14, Great Lakes' default lobby) at cover=1 on the click-free pool, run1494's line
# with the console's `river` arm in place of `weather` (`docs/MAPGEN.md` §21). `ConsoleWin::run_cmd@007d6a70` case
# 0x65 compares its first argument token whole (`String::ignore`, `_wcsicmp`) against `translated_strings.xml`
# entries 868 `start`, 895 `grow`, 896 `calc`, 897 `path`, 898 `flow`: one word a line. `grow` starts a river when
# the global `cur_river` is null (`RiversOut::start_river`) and adds a control point at the cell `@aim` stored
# (`div_3_table[coord >> 8]`); `calc` calls `River::calculate@008858e0` on a non-null `cur_river`.
#
# Three points on one row of inland `BASELAND` ((32..34, 28): its 11x11 is land on the start grid), so
# `RiverOut::ensure_ocean_end` (which runs only past two points) finds no `OCEAN` and no off-map cell in its 7x7, and
# `Spline::calc_spline` keeps an order above zero. One `@aim` a frame: each stores at the top of its frame's do_frame.
# `@dumpall` on 624 prints the block after `calculate`'s `spot_update_land_batch`.
#
# Predicted: `River::calculate@008858e0` entered on 623. Killer: no `HIT` for 008858e0 while the `calc` line's
# `INFO cmd` shows `parse_cmd` returning 1. The twin (run1130) is run1129's file at cover=0: every input before 620
# is shared, so `samegame.py --from 620`.
600 @cursor 70 33
620 @aim 32 28
620 !river grow
621 @aim 33 28
621 !river grow
622 @aim 34 28
622 !river grow
623 !river calc
624 @dumpall
