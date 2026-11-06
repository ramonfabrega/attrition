# Chapter seventeen's lobby and three air group actions (item 3911,
# docs/GOLDEN.md §158; AI §391.2 rows 1, 11 and 3). Run 1505.
#
# `chapter17.cmd`'s eleven lines verbatim, then three arms, each on a plane
# already flying, at least 50 frames apart:
#
#   680  `@patrol` on the Fighter `0/6`, flying home since 640:
#        `Group::action_patrol@007030c0:54-75` hands a plane leader to
#        `Group::action_air_patrol@007029d0` (row 1).
#   780..792  the camera on the Barracks `1/2006`'s tile, the Bomber `0/7`
#        selected, and a right-click at the view's centre (run1032's
#        mapping, as dock3455-rclick.cmd): `Console::execute_at_cursor
#        @007c6630:3116` asks `GroupData::is_in_air_range@0070b6b0` for an
#        air selection pointed at an enemy (row 11). `0/7` holds its strafe
#        on `1/2006` from 778 (run223), so the Barracks is seen and near.
#   850..852  the Bomber `0/8` selected and F13, bound to OPTION_RETURN:
#        `Options::recall_aircraft@0071c670` writes byte 0x23,
#        `process_recall@00947bd0` → `action_recall` → `Group::action_return
#        @006fad40`'s domain-2 arm, a plane in flight (row 3).
#
# The take, at `cover=1` on the click-free pool, and its `cover=0` twin
# (the same line without `--cover`), for the word:
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch17air \
#       --map 14 --end-frame 1100 --log-window 605 1100 --timeout 5400 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --no-ffwd --bindkey OPTION_RETURN=124 \
#       --cmd-file tools/gamelog/golden/chapter17-air.cmd [--cover cover=1]
#
# The trailing words of the `@rclick` and `@key` lines are the harness's
# (the target under the cursor, the bound action); the DLL ignores them.
#
# WHAT WOULD FALSIFY IT, read before the run (the journal has it whole).
# (1) No `process_patrol` on 681, or `0/6` holding a `PATROLORDER` rather
# than an `AIRPATROLORDER` on 682. (2) No `HIT` on `is_in_air_range` on 792;
# then no `process_flight 2006 1` on 793 means the cursor refused it.
# (3) `INFO 22` on 852 looking up `KEYMAP_NONE`, no `process_recall` on
# 853, or `0/8`'s order on 854 still the strike on `1/2006`.
0 !ai off
600 library who=0 6
606 add airbase who=0 60,72
610 add fighter who=0 60,84
612 add bomber who=0 52,84
614 add bomber who=0 68,84
616 add barracks who=1 110,86
620 @strike 0 2006 1 7 8
640 @flight 0 2007 0 6
660 @flight 0 2007 0 7 8
664 @strike 0 2006 1 7 8
680 @patrol 0 7680 11520 6
780 camera 110,86
786 select 7 who=0
792 @rclick 960 445 2006 1
850 select 8 who=0
852 @key 124 OPTION_RETURN
