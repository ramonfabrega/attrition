# Golden record, chapter sixty-two — a follower's guard refusal under its
# squad head, the leash refusals that search and find, and a guard re-issued
# (item 2066, `docs/GOLDEN.md` §75; parked 2033, 2034).
#
# `Unit::fight@005fd4d0`'s guard refusal (`Object::check_target@00649e00`'s
# leash) kills the attack and runs `find_melee_target(−1, NULL, 0, 1, 0)`,
# whose squad head answers a follower before any search: its captain's
# target when the captain's action is an ATTACK, −1 when it is not. Chapter
# sixty-one reached the follower's refusal past its word with the head and
# the search both answering nothing, so nothing held the call.
#
# THE STAGING, walked by this crate on run814's start (RON_STAGE, and
# RON_PROBE_2066 on the probe commit printing the head beside the headless
# search at every guard refusal). Two Hoplite squads, `0/6` (captain), `0/7`,
# `0/8` and `0/10` (captain), `0/11`, `0/12`, guard the Citizen `0/9`; the
# guard is issued on 620 and issued again on 640 (`update_guard_order` finds
# each member's GUARD the second time). who=1's Hoplite squads `1/6` and
# `1/9` are walked past them from 702 (`be 1`); the captains' leash refuses
# their targets on 778, 800, 804, 851 and 884, and the search finds the
# other squad's men inside the leash, or nothing. A third who=1 squad `1/12`
# is added inside the leash on 900, and on 905 the follower `0/7`'s target
# `1/6` is refused while its captain stands on its GUARD: the head answers
# nothing where the headless search takes `1/12`.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch62 \
#       --map 14 --end-frame 1000 --log-window 605 1000 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter62.cmd
0 !ai off
602 add hoplite who=0 225,166
604 add citizen who=0 228,166
606 add hoplite who=0 225,169
620 @guard 0 9 0 6 10
640 @guard 0 9 0 6 10
700 add hoplite who=1 240,166
701 add hoplite who=1 231,163
702 be 1
702 @move 1 38400 31968 6
702 @move 1 44000 26000 9
703 be 0
900 add hoplite who=1 229,171
