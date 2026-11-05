# Item 3879: item 3874's printed line (run1495) moved to item 3876's passing cell (`docs/MAPGEN.md` §20, "The
# offline search"): East Indies, `--map 18`, map size 2, two players, seed 12345, run1156's lobby, at cover=1 on the
# click-free pool. `weather` aimed at P = (2, 17) plants `BASELAND` 3 on an `OCEAN` cell whose north-east diagonal
# C = (3, 16) is `SANDY` with both flanks F1 (2, 16) and F2 (3, 17) `OCEAN`. `TERRAIN` at detail 40 on 620 alone,
# by `@logdetail`, prints every edge write of the re-mark.
#
# Predicted (`plant.py cell gamelog-run1157-… 2,17`): 80 `Set edge` writes; write 19 `x (3,16) 7` (F1's (A) batch,
# `check_for_tendrils` returning 0 at flank sum 2), write 38 `y (3,17) 7` (F2's, the same); C's word after
# `check_blocks` empty, so `TerrainOut::check_for_diagonal@008595f0` is entered at (3, 16). Killer (3876a): the
# `@dumpall` on 621 shows (2, 17) `OCEAN` 0 -> `BASELAND` 3 alone and the trace has no `HIT` for 008595f0.
#
# No `@cursor` and no `--hover` (run1494's carried them from run1129): the twin is run1157, run1156's lobby at
# cover=0 with neither, so every input before 620 is shared and `samegame.py --from 620` holds by construction.
620 @logdetail 0 28 40
620 @aim 2 17
620 !weather
621 @logdetail 0 28 0
621 @dumpall
