# Item 3874: item 3871's line (run1494: the golden start, map 14, cover=1, `weather` aimed at (41, 42)) with
# `TERRAIN` at detail 40 on 620 only (`docs/MAPGEN.md` §20). Every edge write in `mark_trans_flags@008625a0`,
# `check_for_tendrils@00859eb0` and `set_edge_x/y` is a `Log::say` at `GLOG_TERRAIN` (0x1c) detail 0x28, and a say
# outside the two dumps is read against `details[0]` (`[Misc Logging]`: `begin_frame` and `end_frame` reset the mode
# to 0), which no console `loglevel` reaches (it writes row 3). So `@logdetail 0 28 40` opens that one byte at 620's
# top, before the plant, and `@logdetail 0 28 0` shuts it at 621's.
#
# Predicted (the reading, §20): W flank (41, 41) sums 2 and returns 0, so `Set edge x (42,41) with flags 7 to
# BLOCKED`; S flank (42, 42) has N, E and W one up, sums 3, takes the three-neighbour arm and returns 1: `Set edge y
# (42,43) with flags 5 to BLOCKED` and no `Set edge y (42,42) with flags 7 to BLOCKED`.
#
# The twin (run1130) shares every input before 620: `samegame.py --from 620`.
600 @cursor 70 33
620 @logdetail 0 28 40
620 @aim 41 42
620 !weather
621 @logdetail 0 28 0
621 @dumpall
