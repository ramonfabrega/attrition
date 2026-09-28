# Golden record, chapter thirty-four — the launch commands' other arms:
# ctrl and alt on both launch commands, MOVE_TO onto a second base, and
# a point list of [P1, A3, P2].
#
# docs/GOLDEN.md §43 (item 1009; docs/DECISIONS.md 41 §1 and §5, 49).
# Chapter thirty-three's cast to its enemy Barracks (2200) — the four
# planes in 0/2007 with no order, fuelled on 2145 (0/8), 2215 (0/7), 2256
# (0/9) and 2302 (0/6) — then a second Airbase and nine presses:
#    2210 `add airbase who=0 44,76`             0/2008
#    2260 `@launchpatrolctrl 0 13440 9600 2007`  verb 21, ctrl 1
#    2275 `@launchstrikealt 0 2006 1 2007`       verb 23, ATTACK, alt 1
#    2305 `@launchmove 0 2008 0 2007`            verb 23, MOVE_TO
#    2312 `@launchstrikectrl 0 2006 1 2007`      verb 23, ATTACK, ctrl 1
#    2320 `@gatherpoint 0 11520 7680 0 2007`     P1, tile (60, 40)
#    2335 `@gatherpointadd 0 1 0 3 2007`         A3, the Citizen 0/1
#    2350 `@gatherpointadd 0 7680 11520 0 2007`  P2, tile (40, 60)
#    2400 `@gatherpoint 0 -1 -1 0 2007`          the Clear
#    2600 `@launchpatrolalt 0 5760 13440 2008`   verb 21, alt 1, at 0/2008
#
#   run362 (item 1009):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch34 \
#       --map 14 --end-frame 2850 --log-window 605 2850 --timeout 6600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter34.cmd
#
# The window is 2,245 blocks: the last staged event is the press at 2600
# (processed 2601, read 2602); 2850 leaves 250 blocks past it.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate on run358's start through the
# commands' own entries, with a prototype of the MOVE_TO arm (kept off the
# branch until the floor is pinned):
# - 2211: the Airbase 0/2008 at (8544, 14688).
# - 2262: 0/8 an AIRPATROLORDER over (13440, 9600), flags 4, and out.
# - 2277: 0/9 a STRAFEORDER on 1/2006, mandatory 1, flags 4; out 2278.
# - 2307: 0/7 a STRAFEORDER home to 0/2008 (returning 1), flags 4, and
#   out; 0/6 fuelled beside it and untouched. In 0/2008 on 2474.
# - 2314: nothing; 0/6 inside with no order.
# - 2322: the list [P1]; 0/6, 0/8, 0/9 over P1, flags 4; 0/7 untouched.
# - 2337: [P1, A3]; the three a STRAFEORDER on 0/1, mandatory 1, flags 0.
# - 2352: [P1, A3, P2]; every stack as on 2351.
# - 2402: the list empty; the three a strafe home, mandatory 0.
# - 2504, 2528, 2594: 0/6, 0/9, 0/8 in 0/2007; 0/7 fuelled on 2558.
# - 2602: nothing; 0/7 in 0/2008 with no order.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (§43 has the readings).
#
# 1. The issues reach the pump: 2261, 2276, 2306, 2313, 2321, 2336, 2351,
#    2401, 2601.
# 2. The second base is 0/2008 on 2211 (else 5 and 11 cannot fire).
# 3. ctrl on the launch patrol (2262): 0/8 (the reading); 0/9; none.
# 4. alt on the launch strike (2277): 0/9 (the reading); 0/7; none.
# 5. MOVE_TO onto 0/2008 (2307): 0/7 (the reading); 0/6; both; none.
# 6. ctrl on the launch strike (2314): none (the reading); 0/6.
# 7. The ground point (2322): three re-ordered, 0/7 not (the reading);
#    all four.
# 8. A3 behind it (2337): three strikes on 0/1, flags 0 (the reading).
# 9. P2 behind A3 (2352): the stacks of 2351 (the reading); a patrol over
#    P2 alone; [STRAFE, AIRPATROL].
# 10. The Clear (2402): three strafes home; 0/7 untouched.
# 11. alt at 0/2008 (2602): none (the reading); 0/7 over the point.
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
2210 add airbase who=0 44,76
2260 @launchpatrolctrl 0 13440 9600 2007
2275 @launchstrikealt 0 2006 1 2007
2305 @launchmove 0 2008 0 2007
2312 @launchstrikectrl 0 2006 1 2007
2320 @gatherpoint 0 11520 7680 0 2007
2335 @gatherpointadd 0 1 0 3 2007
2350 @gatherpointadd 0 7680 11520 0 2007
2400 @gatherpoint 0 -1 -1 0 2007
2600 @launchpatrolalt 0 5760 13440 2008
