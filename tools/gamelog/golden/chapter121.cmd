# Golden record, chapter one hundred and twenty-one -- a formation's move with a shift-attack on a fleeing unit
# behind it (item 3939, GOLDEN 171; parked 3936a, AI 397).
# Chapter one hundred and twenty's standing Envelop, with the shift-attack laid on a who=1 Citizen walked away from
# the Catapults instead of on a Barracks: a `Build` answers `check_target_path:138-141`'s vslot `+0x1c` with a folded
# `return 1`, a `Unit` with 0. The unit block (`:78-97`) returns early unless the target is fleeing: moving (vslot
# `+0xd8`), its heading at least a sixth of the circle off the line back to the shooter (`heading - (bearing -
# 0x80000000) >= 0x2aaaaaaa`, `5e2493`) and `flanking` nonzero (at most `0xd5555555`).
#   686: the Citizen `1/6`, born after the Catapults' birth so its auto-gather does not walk it east first;
#   688: `be 1` and `@move 1 3168 8832 6`, due north along x=16 past who=0's Scout at 19,60 (`@move` refuses another
#        seat, so the seat is switched for one package and back on 689);
#   697: `@form 0 2 0 6 7` and `@attackadd 0 6 1 6 7`, one package, so each Catapult's list is `[GROUP_MOVE
#        (bit 4 clear)] .. [ATTACK on 1/6]` from 699; 714 is the leader `0/6`'s sixteenth frame inside the list
#        (`(714 + 6) & 15 == 0`), the Citizen about 4,300 north of the move's point (2328, 16392), out of a
#        Catapult's 15 tiles, seen by the Scout, and walking away.
# Killer, before the take (the journal's): the leader's list standing on 714, `1/6` moving north and out of range
# of the move's point, and the cover=1 twin entering no `Group::is_moving_to@00713bd0`.
#   run1522 (cover=0), run1522b (cover=1, --twin), both with the golden start's lobby pinned:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch121 \
#       --map 14 --end-frame 1000 --log-window 605 1000 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --ai-tribe 11 --profile DIFFICULTY=0 --profile STARTING_RESOURCES=1 \
#       --profile STARTING_TECHNOLOGY=0 --profile ENDING_TECHNOLOGY=7 \
#       --cmd-file tools/gamelog/golden/chapter121.cmd
0 !ai off
610 add catapult who=0 12,84
612 add catapult who=0 18,84
618 add scout who=0 19,60
686 add citizen who=1 16,66
688 be 1
688 @move 1 3168 8832 6
689 be 0
697 @form 0 2 0 6 7
697 @attackadd 0 6 1 6 7
