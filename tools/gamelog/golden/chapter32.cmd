# Golden record, chapter thirty-two — an Airbase's gather point: the planes
# re-ordered on each press, a second point appended, a plane trained under
# the list, and the Clear.
#
# docs/GOLDEN.md §41 (item 947; docs/DECISIONS.md 41 §1 and §5, 49).
# Chapter twenty-nine whole — the Airbase `0/2007` repeating, `0/6` flying
# from 1585, `0/7` and `0/8` inside to 1789 and 1813, the Biplane queued on
# 1540 and born on 1746 — and three `@gatherpoint` lines on the Airbase:
#    1600 `@gatherpoint 0 11520 7680 0 2007`     P1, tile (60, 40), NEW
#    1700 `@gatherpointadd 0 5760 5760 0 2007`   P2, tile (30, 30), appended
#    1850 `@gatherpoint 0 -1 -1 0 2007`          the Clear
#
#   run344 (item 947):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch32 \
#       --map 14 --end-frame 2360 --log-window 605 2360 --timeout 5400 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter32.cmd
#
# The window is 1,755 blocks: the last staged event is the last plane's
# landing after the Clear, near 2106 by this crate's prototype; 2360 leaves
# 254 blocks past it.
#
# ---------------------------------------------------------------------------
# UNDER THE EMULATOR (item 947, step 1; the table is in §41).
#
# Build::add_gather_point@00622e70 at an Airbase (build_masks & 8,
# is(0x1bf)) re-orders every live air unit homed there from the whole list
# on every press: the first point an add_air_patrol_order(point, base, 1),
# the rest appended to its arrays, the old air order's cruising_alt and
# sharp_turn carried when either is non-zero. QUEUE_NEW's clear_gather
# first sends a flying plane home and empties an inside one's orders.
# Build::train@0062f9b0's CARRY_AIR arm gives the trained plane the same
# patrol and leaves it inside.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate through input::group_gather_point on
# run308's start; a prototype of the arms (reverted before the capture)
# gives the reading's values. run345 was not used.
# - 1602: 0/6, 0/7, 0/8 each an AIRPATROLORDER over P1, flags 4.
# - 1702: each [P1, P2]; 0/6's cruising_alt its own of 1701.
# - 1741: 0/6 at P1, waypoint 1.
# - 1746: the Biplane 0/9 born inside on [P1, P2], flags 4; out on 1747.
# - 1789, 1813: 0/7, 0/8 launched.
# - 1852: all four a STRAFEORDER home, returning 1, flags 0.
# - 2034..2106: all four inside with no order.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (§41 has the readings).
#
# 1. The issues reach the pump: process_gather_point on 1601, 1701, 1851.
# 2. The lists: [P1] on 1602, [P1, P2] on 1702, empty on 1852.
# 3. The NEW press (1602): each plane a patrol over P1, flags 4 (the
#    reading); its old patrol (the list alone); a strafe home or nothing
#    (the clear half alone).
# 4. The LAST press (1702): each patrol [P1, P2]; 0/6's height copied.
# 5. The trained plane (1746): 0/9 on [P1, P2], flags 4, or no order; out
#    on 1746 or 1747.
# 6. The walk: 0/6's waypoint 1 near P1, then its heading on P2.
# 7. The Clear (1852): each plane a strafe home, or its patrol.
# 8. The landings: each inside with no order by 2359.
#
# A call on trace frame F is on block F+2 (§17).
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
766 @strike 0 2006 1 6
1540 @queueup 0 287 1 2007
1600 @gatherpoint 0 11520 7680 0 2007
1700 @gatherpointadd 0 5760 5760 0 2007
1850 @gatherpoint 0 -1 -1 0 2007
