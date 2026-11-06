# Item 3924: Southwest Mesa (map 10, run1493's default lobby: the human tribe 4, computer tribe 11) at cover=1 on
# the click-free pool, one console `add` of a Mine beside a cliff (AI §393.1 row 1, parked 3909a, AI §395).
# The site is read off run1493's start `WORLD` records (60 x 60 cells of 768 units): who=0's cells hold a cliff at
# (11..13, 41..45), and Mesa's start prints `mountain_size 0`, so `MountainsData::find_nearest` finds no range and
# `CliffsData::find_nearest`'s cliff wins. `add`'s bare `x,y` is a 768-unit cell (ORACLE "add … x,y"), so the mine's
# centre is near (10·768 + half its footprint, 43·768 + …), about 960 units from cell (11, 43)'s centre: inside
# `gather_radius × 0xc0` = 1152.
# Predicted: `BuildTypeData::calc_gather`'s mine arm reaches `CliffsData::gather_size@008a8f80` on the `add`'s frame
# or the first `Leader::calc_gather` after it, and the mine's BUILDDATA `cliff` reads >= 0.
# Falsifier (written before the take): no `gather_size` hit on 100..199, or a `NO_MOUNTAIN 0xc` refusal — the cliff
# past the reach, or not in `scary_cliffs`' mining list.
#
# The twin is this file at cover=0: every input shared, so no `--from`.
100 add mine who=0 10,43
