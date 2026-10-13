# Golden record, chapter fifty-nine — a squad that guards, between two
# Stockades (item 1960, `docs/GOLDEN.md` §71; parked 1887).
#
# `Unit::find_melee_target@005ff9c0`'s head (lines 32–91) answers for every
# caller: a non-captain takes its captain's target when the captain's action
# is an ATTACK, and finds *nothing* when it is not. No golden chapter has a
# squad member reach a search outside the group's retarget and the add path,
# because `Unit::think@005f6e40`'s first statement keeps a non-captain out of
# the idle search (a captain mirror). A **Guard** is the order a member holds
# that runs its own search: `do_guard`'s idle look, one frame in sixteen.
#
# THE STAGING, walked by this crate on run786's start (RON_STAGE,
# RON_PROBE_1887 on the probe commit), trace frames: `602 add hoplite` is a
# squad `0/6` (captain) with `0/7`, `0/8`; the Citizen `0/9` is the charge; the
# squad guards it from 620 (`@guard 0 9 0 6`, the captain only: a member named
# beside it takes no order); two Stockades of who=1 east and west, 13 and 10
# cells from the squad, whose fire the squad answers. From ~672 the members
# search every sixteen frames: `0/8` and `0/7` with the captain not attacking
# (the head's answer is nothing; this crate searches), and `0/7` again the
# frame after, the captain's target a Stockade, a valid one (the head's answer is
# that target; this crate searches).
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch59 \
#       --map 14 --end-frame 760 --log-window 605 760 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/golden/chapter59.cmd
0 !ai off
602 add hoplite who=0 225,152
604 add citizen who=0 228,152
606 add stockade who=1 238,152
608 add stockade who=1 215,152
620 @guard 0 9 0 6
