# Item 3464: two console reports on the golden start (CENSUS row 840), riding
# run1031's own lines (`market3426.cmd`) so its `cover=0` twin run1031b is the
# twin for the whole window: the two `!` lines below are the only inputs
# run1031b does not share, and both come before its first (605).
#
# `!cursor tinfo` (`run_cmd` case 0x3d): the `tinfo` keyword (`int_str_array`
# +0x8ebc, entry 1827) is the arm that, past `WorldData::valid`, calls
# `World::get_wdata@0046d220` on `ConsoleWin::mouse_coord_x/y`. No `@cursor`:
# the tracer calls `parse_cmd` with `no_mouse = 1`, which never refreshes
# `mouse_coord_x/y` from the cursor (docs/ORACLE.md, "The channel's
# vocabulary"), so the cell is the field's stale value whatever is pinned.
# `!startareas` (case 0x60): every leader whose flag bit 1 is set walks the
# start circle (`circle_radius[0..5]`) through `WorldData::is_forest@006b5560`.
# Both only add text to the console window's list.
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/run1084 \
#       --map 14 --end-frame 700 --log-window 600 700 --timeout 3600 \
#       --detail end:LEADERS=2,BUILDS=7,CITIES=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cmd-file tools/gamelog/coverage/console3464-start.cmd --cover cover=1
0 !ai off
600 !cursor tinfo
602 !startareas
605 resource who=0 all +1000
620 @sell 0 1 0
640 @buy 0 1 0
660 @sell 0 1 1
700 !quit
