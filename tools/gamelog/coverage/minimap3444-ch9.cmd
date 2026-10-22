# Item 3444: chapter nine's lobby (docs/GOLDEN.md §17, run180's lines) on the click-free pool, its chariot's
# `620 @move` replaced by a plain right-click on the minimap (`tools/trace/click.h`) -> WorldMap::on_right_up.
# The minimap's diamond, read off a paused window capture of this lobby at 621 (game coordinates): bottom
# (483,1066) = world (0,0), left (326,988) = x 46080, right (645,988) = y 46080, top (483,911). The point
# (433,1021) fits world ~(20550, 5950), cell (26, 7). The first unshared input is 616; `samegame.py --from 616`.
0 !ai off
610 add chariot who=0 16,36
612 add hoplite who=0 16,52
616 select 6 who=0
620 @rclick 433 1021
640 @move 0 4992 16512 7
