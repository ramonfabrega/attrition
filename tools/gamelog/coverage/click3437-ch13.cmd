# Item 3437: run1033's take (item 3427, chapter thirteen's lobby) on the click-free pool, its hand clicks
# as `@click` lines on the frames run1033's trace entered them (`tools/trace/click.h`).
0 !ai off
606 add barracks who=0 14,74
610 add chariot who=0 20,60
614 add hoplite who=0 10,90
620 @garrison 0 2007 0 6
640 @garrison 0 2007 0 7
700 select 2007 who=0
700 camera 14,74
# run1033 `c 34 956`: the Barracks' first train button -> BuildData::can_queue@0062da00 (736)
736 @click 34 956
# `c 158 1056`: Eject -> Options::do_eject_all, GroupOut::issue_eject_all (748)
748 @click 158 1056
