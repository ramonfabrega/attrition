# Item 3454: chapter seven's lobby (docs/GOLDEN.md §11, run141) with a wonder added and razed by console
# lines alone: `add` without `NEW` completes the Pyramids (type 526, 0x20e) at once, so
# `Build::activate` -> `Wonders::init_wonder@0073c860` raises `LeaderData::wonder_mark` (+0x424); `die`
# then runs `Build::close` -> `Wonders::close_wonder@0073c7e0` on the building's wonder slot (+0x76).
# The second `add` asks whether the console honours one wonder of a type per player; if it stands, the
# first `die` closes the lower entry (the mark should stay) and the second the highest (the mark falls
# past both). 2007 is the object run1060's `add small_city` took on 640 in this lobby.
0 !ai off
600 library who=0 2
610 add citizen who=0 24,160
615 add caravan who=0 28,160
620 add merchant who=0 24,164
625 add scholar who=0 28,164
630 add fur who=0 24,176
640 add pyramids who=0 40,160
645 add pyramids who=0 48,160
660 die 2007,0
670 die 2008,0
