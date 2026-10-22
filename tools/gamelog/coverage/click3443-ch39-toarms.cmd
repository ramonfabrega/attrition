# Item 3443: chapter thirty-nine's lobby (docs/GOLDEN.md §48, run422) on the click-free pool, its first
# To Arms pressed by the Citizen's own button: `Options::do_spell@0071d7a0` through `Options::exec`.
# A spell's cell is its GRID_X on row 1 (`UnitOut::make_options@00615fc0`: `Options::add(OPTION_SPELL,
# grid_x, 1, ...)`; craftrules' GRID_Y is not read there), so To Arms (GRID_X 4) is the second row's last
# cell, (286, 976) on the 1920x1080 card (run1055's `sel2.png`: Barracks (0,0) is (34, 935), Military
# Buildings (1,1) is (96, 976)). The first take clicked (286, 935), the Citizen's Alarm bell
# (`OPTION_ALARM_PEASANT` at (4,0)): `Options::do_alarm` on 620, no `do_spell`. The twin's
# `620 @spell 0 660 -1 -1 0 0 6` is dropped for the clicks. The second take clicked (286, 976) on 620
# alone and hit an empty cell: `UnitOut::make_options`, which adds the spells, first ran on 621. So 620's
# click stays and 624's presses To Arms.
0 !ai off
600 library who=0 1
604 tech who=0 militia on
606 add citizen who=0 30,150
608 add citizen who=0 34,150
610 add militia who=0 38,150
612 add general who=0 40,180
614 add hoplites who=0 44,180
616 add slingers who=0 40,184
618 select 6 who=0
620 @click 286 976
624 @click 286 976
640 @spell 0 660 -1 -1 0 0 7
660 @spell 0 659 -1 -1 0 0 8
700 @spell 0 659 -1 -1 0 0 6
720 @spell 0 634 -1 -1 0 0 9
