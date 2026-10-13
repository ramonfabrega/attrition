# Golden record, chapter sixty-one — a guarding follower's refused chase and
# its refused target (item 2032, `docs/GOLDEN.md` §74; parked 1887).
#
# `Unit::find_melee_target@005ff9c0`'s head (lines 32–91) answers for every
# caller: a non-captain takes its captain's target when the captain's action
# is an ATTACK, and gets −1 when it is not, before any search. Two of its
# callers a follower reaches go through this crate headless:
#
#   1. `Unit::fight@005fd4d0`'s chase on a building that
#      `Unit::find_attack_pos@00601280` refuses a guard (`chase_refused`):
#      `Unit::find_new_target@005ff6a0` → `find_melee_target(−1, NULL, 0, 1, 0)`.
#   2. `fight`'s guard refusal (`Object::check_target@00649e00`'s leash
#      refuses the target): the attack killed, then the same search.
#
# THE STAGING, walked by this crate on run801's start (RON_STAGE,
# RON_PROBE_2032 on the probe commit; trace frames). Scene one: the squad
# `0/6` (captain), `0/7`, `0/8` guards the Citizen `0/9` beside who=1's
# Stockade `1/2006`; the captain stands in reach of it, `0/8` does not, and
# from 689 every sixteen frames `0/8` takes the captain's Stockade through
# the guard's look and its chase is refused (head: the captain's Stockade;
# this crate's search: the same Stockade). A who=1 Citizen `1/7` added at
# 760 by the squad: from 769 this crate's search for `0/8` finds the Citizen
# where the head answers the Stockade. Scene two, 4,000 units west: the
# squad `0/10` (captain), `0/11`, `0/12` guards the Citizen `0/13`; who=1's
# Citizen `1/6` is walked through it from 700 (`be 1`), the squad chases it
# out of the leash, and the followers' attack is refused on 872 and 906
# (head: nothing, the captain not attacking; this crate's search: nothing).
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch61 \
#       --map 14 --end-frame 1000 --log-window 605 1000 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter61.cmd
0 !ai off
602 add hoplite who=0 225,152
604 add citizen who=0 229,152
606 add stockade who=1 234,152
610 add hoplite who=0 205,160
612 add citizen who=0 209,160
614 add citizen who=1 197,160
620 @guard 0 9 0 6
630 @guard 0 13 0 10
700 be 1
700 @move 1 41472 30720 6
701 be 0
760 add citizen who=1 230,154
