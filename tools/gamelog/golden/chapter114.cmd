# Golden record, chapter one hundred and fourteen -- units the console's `eval` creates, on the lake and on land (item
# 3917, GOLDEN 164; AI 391.2 row 10, 393.1 row 2; parked 3906c, 3909b).
# Chapter twenty's lobby to 604 (GOLDEN 28, run249): `library who=0 1` (Written Word, the transport level) and the Dock
# on lake 70, so who=0's `can_transport` is not none. Then two lines through the console's `eval` (run_cmd case 0x15,
# the `!` half): `ScenarioFuncSet::create_unit@009f4c50` -> `add_unit@009e2220(who - 1, x, y, type, num, 0)`, `x`, `y`
# in quarter cells (`x * 0xc0 + 0x60`), the type matched by its name (`get_type_index`), then `get_graft`.
#   620: one Hoplites squad on tile 62,182 -- cell (15, 45), OCEAN, flags 0 on run249's start: a land type on an
#        `is_ocean` tile -> `Objects::init_unit` at the raw point, `push_group`, `Group::action_transport@00702620(1)`
#        (a barge built at the captain's point, the squad `go_inside` it), then `action_form` and
#        `Group::dbg_jump_to_action@00707a00` on a group whose members are inside.
#   640: two Hoplites squads on tile 34,181 -- cell (8, 45), BASELAND: each at `find_nearby_spot(.., 0xc00, ..)`,
#        `push_group`, `action_form(0, 0, QUEUE_NEW)` (the group moved to `get_loc_to`), then `dbg_jump_to_action`:
#        each active member on the map `Unit::dbg_jump_to_action@00602ea0`, popped to its last move's point.
# Falsifiers, before the take (the journal has them): (1) no `HIT` in the twin on `create_unit`, `add_unit`,
# `action_transport` on 620, or on `Group::dbg_jump_to_action`, `Unit::dbg_jump_to_action` on 640; (2) block 621: no
# who=0 barge holding the Hoplites (a -1 return: the name or the tile refused); (3) block 641: the Hoplites at their
# placement points holding a `CHANGE_FORM` or a move -- not snapped; (4) a `CM` line on 621 or 641 (the console's eval
# issues no command: anything there is a command this reading does not know).
#   run1511 (cover=0) and run1511b (cover=1, `--ai-tribe 11 --cover cover=1 --no-ffwd`):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch114 \
#       --map 14 --end-frame 900 --log-window 605 900 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter114.cmd
0 !ai off
600 library who=0 1
604 add dock who=0 53,153
620 !eval create_unit(1, 62, 182, "Hoplites", 1)
640 !eval create_unit(1, 34, 181, "Hoplites", 2)
