# Item 3477: the golden start (map 14, the Nubian human, tribe 4, with a Market; computer tribe 11) at cover=1 on the
# click-free pool: the seventeen `ScenarioFuncSet` rows through the console's `eval`, and `Tribes::find` behind
# `disable_type_by_tribe`. 3471's route: a plain line is the chat door, which reaches `run_cmd`'s second switch only;
# `eval` (case 0x15) is in the first, so its lines take `!`. The tracer keeps every byte to the end of the line.
# `who` is 1-based in every body (who 1 = leader 0, the human; who 2 = leader 1, the computer).
#
# 0..660 are `console3464-start.cmd`'s lines (run1084's), so the market is paid for (605) and exercised before 670.
# Then one `!eval` a frame pair, read-only calls first and the leader-flag writes last, so nothing earlier rides a
# disabled AI. The moved fields are `LEADERS=2`'s (run1084's frame-600 block, on disk: who 0 `leader_flags` 7,
# who 1 19, both `leader_flags2` 0, every `base_rate[scan]` 0; Napata `city_flags` 18449):
#   678/680  buy_good / sell_good: who 0's food bucket up, wealth down; then timber down, wealth up (Leader::do_buy)
#   682      set_base_rate: who 0's base_rate[0] 0 -> 80 (num << 4)
#   684      set_capital: Napata re-asserted, city_flags stays 18449 (trace only)
#   692      set_auto_peasant_level(2): the Citizens' stance 0 -> 2 if the option's level is not already 2
#   694      enable_production_ai clears bit 4 of who 0's leader_flags2 (stays 0)
#   696/698  disable_combat_ai / disable_all_unit_ai: who 0's leader_flags2 0 -> 8 -> 10
#   700      disable_unit_ai(1, 0): unit 0's +0x68 |= 0x1000000 (which dumped field is +0x68 is not read)
#   702      disable_city_ai(2): who 1's leader_flags2 0 -> 16
#   704      disable_type_by_tribe on two names that match nothing: Tribes::find@006ef2d0 returns -1, a no-op
# Both takes stage `--cursor 1400,1000` (the empty right half of the bottom panel, off the map: Loop 3432).
# The twin is this file at cover=0: every input shared, so there is no `--from`.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/run1092 \
#       --map 14 --end-frame 720 --log-window 600 720 --timeout 3600 \
#       --detail end:LEADERS=2,BUILDS=7,CITIES=5,UNITS=3 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/coverage/eval3477.cmd --cover cover=1 \
#       --cursor 1400,1000
0 !ai off
600 !cursor tinfo
602 !startareas
605 resource who=0 all +1000
620 @sell 0 1 0
640 @buy 0 1 0
660 @sell 0 1 1
670 !eval get_difficulty()
672 !eval can_buy_sell(1)
674 !eval find_buy_price(1, "Food")
676 !eval find_sell_price(1, "Food")
678 !eval buy_good(1, "Food")
680 !eval sell_good(1, "Timber")
682 !eval set_base_rate(1, "Food", 5)
684 !eval set_capital(1, "Napata")
686 !eval toggle_infinite_queue(1, 2000, 1)
688 !eval repeat_orders_enable(1, 0)
690 !eval repeat_orders_disable(1, 0)
692 !eval set_auto_peasant_level(2)
694 !eval enable_production_ai(1)
696 !eval disable_combat_ai(1)
698 !eval disable_all_unit_ai(1)
700 !eval disable_unit_ai(1, 0)
702 !eval disable_city_ai(2)
704 !eval disable_type_by_tribe("NoSuchType", "NoSuchTribe")
