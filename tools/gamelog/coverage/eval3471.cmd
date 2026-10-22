# Item 3471: the coverage pair's lobby (run1080's: 3446's carrier lobby, run651/run652's game) at cover=1 on the
# click-free pool, AI on: the console's `eval` and `netcmd`. `--detail misc:COMMANDMANAGER=2` on both takes, because
# `process_console_cmd`'s `process_command_cmd` line is a `Log::say` at detail 2.
# A plain line is the chat door (`parse_cmd(.., from_chat 1, ..)`), which reaches `run_cmd`'s second switch only;
# `eval` (case 0x15) is in the first, so its lines take `!`. The tracer keeps every byte to the end of the line.
#   600, 602  run1080's first two `add`s, so the Hoplite captain is 0/15 as there (the carrier is 0/14)
#   610       `!eval rand_real()` -> Compiler::eval_command@009befe0 -> MathUtilFuncSet::rand_real@009e18b0
#             (one game_random draw)
#   620       `!eval set_bonus_cap(1, "Wealth", 200)`: who is 1-based; "Wealth" is types.list[2], so leader 0's
#             `bonus_cap[2]` 0 -> 200 (ScenarioFuncSet::set_bonus_cap@009fbd60; `LEADERS=9` `bonus_cap`)
#   630       `!eval is_on_transport(1, 15)` on the Hoplite: not a plane -> ObjectData::in_a_ship@006440c0
#   640       `netcmd resource who=0 food +1000`: run_cmd case 0xf issues `local_48`, the args' FIRST token
#             (`TokenString::tokenize` on " ,\t"), so the replicated command is the bare `resource` ->
#             CommandPackage::process_console_cmd@00943f30 (`CM process_command_cmd resource`), which prints the
#             buckets and writes none
# The twin is this file at cover=0: every input shared.
600 add aircraft_carrier who=0 6,37
602 add hoplite who=0 28,44
610 !eval rand_real()
620 !eval set_bonus_cap(1, "Wealth", 200)
630 !eval is_on_transport(1, 15)
640 netcmd resource who=0 food +1000
