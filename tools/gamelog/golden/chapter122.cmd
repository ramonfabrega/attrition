# Golden record, chapter one hundred and twenty-two -- a queue order on a mixed group (item 3966, GOLDEN 172; T71-14).
# `Group::action_queue_up@006fdbb0`'s train arm offers `Build::queue_up(ty, 1)` to every alive, active member of the
# commanded group, and leaves the type test to `could_queue` -> `BuildData::can_make@0062db10`, whose
# `BuildTypeData::queue_here` gate (`:31`-`:36`) stands ahead of the per-kind tail and binds a unit too.
#   620: Hoplites (132) on the Barracks `0/2007` and the Stable `0/2008`, num 1: both hold nothing, so the sort keeps
#        the list's order; the Barracks lays one, the Stable's `queue_here` refuses.
#   640: Light Horse (209) on the same two, num 2: the sort puts the Stable (0 queued) ahead of the Barracks (1);
#        each pass the Stable lays one and the Barracks refuses, so the Stable holds two.
# Killers, before the take (the journal's list): (1) the cover=1 twin enters no `Group::action_queue_up` on trace
# frame 621 -- the press never reached the group arm; (2) `0/2008` reads `queued 1 [132]` on 622 -- the original
# queues a Hoplite at a Stable and T71-14 is wrong; (3) `0/2007` reads anything but `queued 1 [132]` after 642 --
# the Barracks laid a Light Horse, or refused its own type.
#   run1537 (cover=0), run1537b (cover=1), both with the golden start's lobby pinned:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch122 \
#       --map 14 --end-frame 1000 --log-window 605 1000 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --ai-tribe 11 --profile DIFFICULTY=0 --profile STARTING_RESOURCES=1 \
#       --profile STARTING_TECHNOLOGY=0 --profile ENDING_TECHNOLOGY=7 \
#       --cmd-file tools/gamelog/golden/chapter122.cmd
0 !ai off
601 resource who=0 all +3000
606 add barracks who=0 14,74
608 add stable who=0 24,74
620 @queueup 0 132 1 2007 2008
640 @queueup 0 209 2 2007 2008
