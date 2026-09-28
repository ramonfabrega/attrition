# Golden record, chapter thirty-three — an Airbase's launch issuers: a strike
# and two launch patrols from the base, and an action-3 point followed by
# another.
#
# docs/GOLDEN.md §42 (item 976; docs/DECISIONS.md 41 §1 and §5, 49).
# Chapter thirty-two whole — its Clear on 1850 leaves the four planes in
# 0/2007 with no order by 2106, fuelled on 2145 (0/8), 2215 (0/7), 2256
# (0/9) and 2302 (0/6) — then an enemy Barracks by the Airbase and five
# presses on it:
#    2200 `add barracks who=1 72,78`           in the Airbase's sight
#    2260 `@launchstrike 0 2006 1 2007`        verb 23, issue_flight ATTACK
#    2280 `@launchpatrol 0 13440 9600 2007`    verb 21, P1, tile (70, 50)
#    2295 `@launchpatrolall 0 7680 11520 2007` verb 22, P2, tile (40, 60)
#    2305 `@gatherpoint 0 1 0 3 2007`          action 3 on the Citizen 0/1
#    2335 `@gatherpointadd 0 9600 7680 0 2007` P3, tile (50, 40), after it
#
#   run358 (item 976):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch33 \
#       --map 14 --end-frame 2740 --log-window 605 2740 --timeout 6600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter33.cmd
#
# The window is 2,135 blocks: the last staged event is 0/8 over P3, near
# 2486 by this crate's prototype; 2740 leaves 254 blocks past it.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate on run344's start with a prototype of
# the arms (kept off the branch until the floor is pinned):
# - 2201: the Barracks 1/2006 at (72, 78), the dead one's number reused.
# - 2262: 0/8 a STRAFEORDER on 1/2006, mandatory 1, flags 4, and out.
# - 2282: 0/9 an AIRPATROLORDER over P1, flags 4, and out.
# - 2297: 0/7 and 0/6 over P2, flags 4; 0/7 out on 2298, 0/6 inside.
# - 2307: the list [(1, 0, 3)]; all four a STRAFEORDER on 0/1, 0/6 flags
#   4 and inside, the others flags 0, each flying at 0/1.
# - 2337: the list [(1, 0, 3), P3]; all four an AIRPATROLORDER over P3.
# - 2406..2486: all four over P3.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (§42 has the readings).
#
# 1. The issues reach the pump: 2261, 2281, 2296, 2306, 2336.
# 2. The Barracks is 1/2006 on 2201 (else 3 cannot fire).
# 3. The strike (2262): 0/8's (the reading); 0/9's; none.
# 4. The plain patrol (2282): 0/9's (the reading); 0/7's; 0/6's; none.
# 5. The shift-click (2297): 0/7 and 0/6 (the reading); 0/7 alone.
# 6. Action 3 (2307): all four on 0/1, 0/6 flags 4 (the reading); all
#    flags 0; the stacks kept.
# 7. The escort (2308..2336): xx/yy on 0/1; a patrol over it; no turn.
# 8. The append (2337): each a patrol over P3 alone (the reading); the
#    strike with P3 behind it.
# 9. The arrivals over P3 by 2739.
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
2200 add barracks who=1 72,78
2260 @launchstrike 0 2006 1 2007
2280 @launchpatrol 0 13440 9600 2007
2295 @launchpatrolall 0 7680 11520 2007
2305 @gatherpoint 0 1 0 3 2007
2335 @gatherpointadd 0 9600 7680 0 2007
