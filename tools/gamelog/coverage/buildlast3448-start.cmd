# Item 3448: ORDERS §42.3 row 3, `Unit::check_build_order@00603470`, on the golden start (map 14; run215's lobby
# to 600). run1069's `@buildadd` of a Barracks behind the same one-Citizen selection was processed
# (`process_build … 427 1 633`, after `process_group, repeat 0 0`) and placed no site. Here the second site is
# chapter eighteen's Siege Factory point, sent for a new selection: the Citizen `0/6` lays a Lookout (521)
# and then, shift-queued with `0/7`, the Siege Factory (430). The Lookout finished, `do_build`'s tail sees
# `0/6`'s order behind it and calls `check_build_order` (ORDERS §5.2 step 6, §5.3). The first unshared input
# is 600; `samegame.py --from 600` against run215. The first take (run1071-take1, without the `library`
# line) placed no Siege Factory either: chapter eighteen's `600 library who=0 2` is what makes it available
# (`LeaderData::type_avail(type, 1) == 4`, `action_build`'s gate, which refuses in silence).
0 !ai off
600 library who=0 2
602 resource who=0 all +2000
604 add citizen who=0 36,172
606 add citizen who=0 40,172
630 @build 0 7296 34176 521 6
632 @buildadd 0 7296 36864 430 6 7
