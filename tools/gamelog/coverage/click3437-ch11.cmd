# Item 3437: run1032's take (item 3427, chapter eleven's lobby) on the click-free pool, its hand clicks as
# `@click`/`@rclick` lines on the frames run1032's trace entered them (`tools/trace/click.h`).
0 !ai off
610 add chariot who=0 16,36
612 add supply who=0 16,44
614 add hoplite who=0 36,152
616 select 6 who=0
616 camera 16,44
# run1032 `rc 960 445`: 0/6 right-clicks the wagon 0/7 -> GroupOut::issue_guard (624)
624 @rclick 960 445
# `c 34 935`: the Move button (Options::exec 704), then `c 958 492`, the pick on 0/7 -> issue_follow (708)
704 @click 34 935
708 @click 958 492
