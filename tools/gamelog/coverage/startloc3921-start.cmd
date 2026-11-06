# Item 3921: the golden start (map 14, run1129's lobby: the Nubian human, tribe 4; computer tribe 11) at cover=1 on
# the click-free pool, one `!eval` of the BHS script function `find_extra_starting_loc` (MAPGEN §22.3, parked 3908a).
# `ScenarioFuncSet::init_funcs` registers it with (num_nation_space, num_land_space, bool_only_check_player);
# the third argument is 1-based, so 1 is the human's seat (leader 0, whose `leaders.list[0] & 1` is set).
# Predicted: `ScenarioFuncSet::find_extra_starting_loc@009e5070` entered on 620, one `game_random` draw when region
# 1's coords hold more than one cell, then `MapConquest::check_starting_loc@0068d7e0` per walked cell.
# Falsifier (written before the take): `009e5070` entered and `0068d7e0` not — region 1's list empty, or the
# argument order wrong.
#
# The twin (run1515b) is this file at cover=0: every input shared, so no `--from`.
620 !eval find_extra_starting_loc(4, 4, 1)
