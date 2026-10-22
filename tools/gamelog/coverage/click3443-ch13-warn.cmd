# Item 3443: run1051's stanza (item 3437, chapter thirteen's lobby) with 1,000 taken from each of who=0's
# buckets on 730 (the first take's `resource who=0 all 0` changed nothing: a zero amount is a no-op), so the Barracks' first train button on 736 cannot be afforded: `Options::exec` ->
# `GroupOut::issue_queue_up@00708c90`, whose first test is the type's cost (vslot +0x84), and on a
# refusal `LeaderOut::warn_resources@006d3650` (the message line; no sim field). The one input
# run1051 does not share is the `resource` line.
0 !ai off
606 add barracks who=0 14,74
610 add chariot who=0 20,60
614 add hoplite who=0 10,90
620 @garrison 0 2007 0 6
640 @garrison 0 2007 0 7
700 select 2007 who=0
700 camera 14,74
730 resource who=0 all -1000
# run1033 `c 34 956`: the Barracks' first train button, now unaffordable
736 @click 34 956
# `c 158 1056`: Eject, kept so the take is run1051's but for 730
748 @click 158 1056
