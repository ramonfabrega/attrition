# Item 3443: chapter seven's lobby (docs/GOLDEN.md §11, run141) with a second city, its Caravan
# right-clicked onto it: `Console::execute_at_cursor` -> `verify_trade` -> `GroupOut::issue_trade` ->
# `CommandManager::issue_trade@00941960`. run141's Caravan `0/7` took no order: Napata was who=0's
# only city. The Small City at (40, 160) is chapter seventy-three's (a pool take placed it there);
# `camera 40,160` draws its tile at (960, 445), as run1032's `camera 16,44` drew the wagon's.
0 !ai off
600 library who=0 2
610 add citizen who=0 24,160
615 add caravan who=0 28,160
620 add merchant who=0 24,164
625 add scholar who=0 28,164
630 add fur who=0 24,176
640 add small_city who=0 40,160
642 select 7 who=0
642 camera 40,160
650 @rclick 960 445
