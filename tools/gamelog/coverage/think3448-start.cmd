# Item 3448: ORDERS §42.3 rows 2 and 3 on the golden start (map 14; run215's lobby to 600).
# Row 2: eight Citizens `0/6`..`0/13` `@gather`ed onto the Woodcutter's Camp `0/2001`, which already holds `0/2`:
# past the camp's `gather_max`, a Citizen's first `do_gather` (`been_there` 0) meets the cap and takes
# `kill_current_order; add_think_order` (`do_gather@005ef2a0:89`-`:113`), and `do_think_order` the next frame.
# Row 3: one Citizen `0/14`, a Lookout (521) site and then, shift-queued (`@buildadd`, QUEUE_LAST), a Barracks
# (427) site: the Lookout finished, `do_build`'s tail sees an order behind it and calls `check_build_order`
# (ORDERS §5.2 step 6, §5.3). The first unshared input is 600; `samegame.py --from 600` against run215.
0 !ai off
600 resource who=0 all +2000
602 add citizen who=0 26,144
604 add citizen who=0 28,144
606 add citizen who=0 30,144
608 add citizen who=0 26,148
610 add citizen who=0 28,148
612 add citizen who=0 30,148
614 add citizen who=0 26,152
616 add citizen who=0 28,152
618 add citizen who=0 36,172
620 @gather 0 2001 6 7 8 9 10 11 12 13
630 @build 0 7296 34176 521 14
632 @buildadd 0 7296 36864 427 14
