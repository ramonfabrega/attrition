# Item 3493 (the sweep of 3480a), as item 3480's fallback: one `--map N --end-frame 36` take a style at cover=1 on the click-free pool, entering
# `TerrainOut::check_for_diagonal@008595f0` (and the `int` overload `WorldData::is_valid@00471170`) on the setup
# frame (-1), when `create_transition_renderables` first builds the map's transitions. The mid-game route
# (`!mapstyle`/`!mapgen`, `mapgen3480-start.cmd`) faulted natively inside `mapgen`'s own `parse_cmd` on run1097.
# The twin is this file at cover=0 on the same style.
0 !ai off
