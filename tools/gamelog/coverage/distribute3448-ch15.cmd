# Item 3448: ORDERS §42.3 row 5, `Group::distribute_attack@00713390`, on chapter fifteen's lobby (GOLDEN §23,
# run215's lines to 614). A player's attack makes no group move (`Group::action_attack` adds per-member
# orders), and every player move carries the action bit, so the leader arm's predicate (an `ATTACK` action
# beneath a current group move, `do_group_move@005e79a0:108`-`:137`) needs a group move without it: the one
# `Group::action_gather@00700b90:462` gives soldiers sent to gather (`action_move_near(…, MOVE_TO, action 0)`).
# So the two Hoplite squads `0/6`, `0/9` are `@gather`ed onto the Woodcutter's Camp `0/2001`, far south, and
# then shift-queued (`@attackadd`, QUEUE_LAST) onto the Chariot `1/6` north of them; four who=1 Supply Wagons
# `1/7`..`1/10` stand across the walk for the leader to collide with. The first unshared input is 616;
# `samegame.py --from 616` against run215.
0 !ai off
610 add hoplite who=0 12,84
612 add hoplite who=0 18,84
614 add chariot who=1 16,64
616 add supply_wagon who=1 13,100
618 add supply_wagon who=1 15,100
620 add supply_wagon who=1 17,100
622 add supply_wagon who=1 19,100
630 @gather 0 2001 6 9
632 @attackadd 0 6 1 6 9
