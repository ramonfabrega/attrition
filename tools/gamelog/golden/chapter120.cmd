# Golden record, chapter one hundred and twenty -- a formation's move with a shift-attack behind it (item 3936,
# GOLDEN 170; parked 3935a, AI 397).
# Two Catapults of the human's (ranged, so `check_target_path`'s melee one-node refusal is not asked; a Slinger
# cannot attack a building and its attack degrades to a move) and a Scout beside an enemy Barracks, so the
# Barracks' `ever_seen` holds who=0's bit (`BuildData::is_seen`, every eighth frame) and `Group::action_attack`
# lays an ATTACK rather than a move (its `goes` arm):
#   697: `@form 0 2 0 6 7`, Envelop on the standing pair: `Group::action_form@00707220` halts, finds nothing to
#        copy, and lays a GROUP_MOVE through `action_move_to(.., QUEUE_LAST, .., action 0, ..)`, so
#        `add_group_move_order` clears the order's bit 4 (value 4); then `@attackadd 0 2006 1 6 7` in the same
#        package, `process_attack` -> `action_attack(.., QUEUE_LAST)`, so each Catapult's list is
#        `[GROUP_MOVE][ATTACK]` and `Unit::update_action` skips the move to the attack.
#   698: the pair's processing frame, `(698 + 6) & 15 == 0`: the leader `0/6`'s sixteenth frame (`Unit::work:187-197`,
#        the group order's `oxx`), so `check_target_path@005e22d0` reviews the attack; the Barracks is not in range
#        of the move's point, is not a wall, and the head is a group order, so `Group::is_moving_to@00713bd0` is
#        asked from `005e25ee`. Our walk keeps the group move to 716, so 714 is the leader's second such frame.
# Killer, before the take (the journal's): the leader's list `[GROUP_MOVE][ATTACK]`, bit 4 clear and the Barracks
# out of range of the move's point, yet the cover=1 twin enters no `00713bd0` on any of the leader's sixteenth
# frames while the list stands.
#   run1520 (cover=0), run1520b (cover=1, --twin), both with the golden start's lobby pinned:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch120 \
#       --map 14 --end-frame 1000 --log-window 605 1000 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --ai-tribe 11 --profile DIFFICULTY=0 --profile STARTING_RESOURCES=1 \
#       --profile STARTING_TECHNOLOGY=0 --profile ENDING_TECHNOLOGY=7 \
#       --cmd-file tools/gamelog/golden/chapter120.cmd
0 !ai off
610 add catapult who=0 12,84
612 add catapult who=0 18,84
616 add barracks who=1 16,56
618 add scout who=0 17,60
697 @form 0 2 0 6 7
697 @attackadd 0 2006 1 6 7
