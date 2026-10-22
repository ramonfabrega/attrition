# Item 3461: the coverage pair's lobby (3446's carrier lobby, run651/run652's game) at cover=1 on the click-free pool,
# AI on: two bound keys and a carrier's right-click. The keys are bound in the lane's staged profile by `--bindkey`
# (F13, F14 = VK 124, 125, keys no default binding uses): OPTION_HALT=124  OPTION_CYCLE_GENERAL=125.
# Cells are run651's WORLD order (row-major in y): cells x=0..2 are OCEAN for y=0..19, the human's coast.
# `add` takes quarter cells (192 internal units).
#   600       an Aircraft Carrier 0/14 on the sea (3446's cell; run1066 and run1076 both drew 0/14 for the first add)
#   602, 604  a who=0 Hoplite squad 0/15..17 and a General 0/18 inland of the capital
#   606       the squad walked east: a group with MOVEORDERs
#   612, 620  the squad's captain selected; F13 -> KEYMAP_OPTION_HALT -> Options::do_halt -> GroupOut::issue_halt ->
#             CommandPackage::process_halt@00949140 (`CM` `process_halt`, COMMANDMANAGER detail 1) -> action_halt
#   640       F14 -> OPTION_CYCLE_GENERAL (option 0x62, shift up: param_1 0) -> Options::do_cycle_general's second
#             pass finds the General (not a gov hero) -> GroupsData::find_group_const@007084c0
#   650, 660  the carrier alone selected, the camera on open sea (cell 1,13), the right-click on the camera's centre
#             tile (960, 445; 3443's mapping): Console::execute_at_cursor's all-type-0x15f arm ->
#             WorldData::is_tocean@0046fb10, then the move (`U3` `orders_x/y`)
# The first unshared input is 600; `samegame.py --from 600` against run672 (blocks 577..833).
600 add aircraft_carrier who=0 6,37
602 add hoplite who=0 28,44
604 add general who=0 28,52
606 @move 0 7680 8448 15 16 17
612 select 15 who=0
620 @key 124
640 @key 125
650 select 14 who=0
650 camera 6,52
660 @rclick 960 445
