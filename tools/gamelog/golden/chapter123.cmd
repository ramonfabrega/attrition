# Golden record, chapter one hundred and twenty-three -- a patrol with a Helicopter in the group (item 3967, GOLDEN 173; T71-6).
# `Group::action_patrol@007030c0` hands the call to `action_air_patrol` only when the leader is a plane or
# `GroupData::count(COUNT_DOMAIN, 4)` -- param 4 counts vslot `+0xc0`, `UnitData::is_plane@0046ce40`: domain 2 without
# the helicopter bit `0x20` -- is non-zero. A Helicopter is no plane, so a group holding one and no plane takes the land
# arm, and every member on the map, not a plane and not busy, the Helicopter among them, gets a GroupPatrolOrder.
#   620: the Helicopter `0/6` and the Hoplite squad (captain `0/7`) patrolled together to (3456, 11136).
#   700: a second Helicopter `0/10`, alone, patrolled to (11136, 11904).
# Killers, before the take (the journal's list): (1) the cover=1 twin enters `Group::action_air_patrol` on 620 or 700
# -- the hand-off is not the plane count; (2) `0/6` or `0/10` holds no PATROLORDER on 622 / 702 -- the original orders no
# Helicopter on a patrol either, and T71-6 is wrong; (3) `0/7`..`0/9` hold none on 622 -- the Helicopter's group took the
# air arm in theirs too.
#   run1538 (cover=0), run1538b (cover=1), both with the golden start's lobby pinned:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch123 \
#       --map 14 --end-frame 1000 --log-window 605 1000 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --ai-tribe 11 --profile DIFFICULTY=0 --profile STARTING_RESOURCES=1 \
#       --profile STARTING_TECHNOLOGY=0 --profile ENDING_TECHNOLOGY=7 \
#       --cmd-file tools/gamelog/golden/chapter123.cmd
0 !ai off
610 add helicopter who=0 16,36
612 add hoplite who=0 56,40
616 add helicopter who=0 30,30
620 @patrol 0 3456 11136 6 7
700 @patrol 0 11136 11904 10
