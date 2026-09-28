# Golden record, chapter thirty-six — the missile's other arms: a second
# strike pressed while the first counts down, the altitude redraw on a
# missile's one step, and MISSILE_DEFENSE_BONUS at the blast and at the order.
#
# docs/GOLDEN.md §45 (item 1078; docs/DECISIONS.md 41 §1 and §5, 49).
# Chapter thirty-five whole (its V2 0/10 launched on 2701 onto 1/2006, the
# Barracks struck on 2820), then:
#    2705 `resource who=0 all +500`              two V2s' knowledge and oil
#    2710 `add missile_silo who=0 150,98`        0/2010, V2b's
#    2712 `add missile_silo who=0 150,90`        0/2011, V2c's
#    2720 `@queueup 0 313 1 2010`                V2b, born 2925 as 0/14
#    2725 `@queueup 0 313 1 2011`                V2c, born 2930 as 0/15
#    2730 `add barracks who=1 198,98`            T_home 1/2007, cell (49, 24), who=1's land
#    2735 `add barracks who=1 186,98`            T_far 1/2008, cell (46, 24), no one's
#    2740 `add elite_special_forces who=0 190,98` the spotter 0/13, cell (47, 24)
#    3019 `@launchstrike 0 2007 1 2010`          V2b on T_home; launched on 3050,
#                                                (14 + 3050) & 7 == 0: the redraw
#    3029 `@launchstrike 0 2008 1 2010`          the re-press, V2b counting down
#    3070 `tech who=1 missile_shield on`         the shield, V2b in flight
#    3080 `@launchstrike 0 2008 1 2011`          V2c on T_far, who=1 shielded
#    (3169: V2b's round comes down on T_home, in who=1's land)
#
#   run390 (item 1078):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch36 \
#       --map 14 --end-frame 3420 --log-window 605 3420 --timeout 7200 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter36.cmd
#   (cover=0 on the click-free lane, 1011's split; a cover=1 twin would be
#   run391 on the queue lane.)
#
# The window is 2,815 blocks: the last scheduled event is V2b's landing on
# 3169; 3420 leaves 250 blocks past it.
#
# ---------------------------------------------------------------------------
# DROPPED ARMS (§45): the nuke — its type is a research job at the silo
# before it trains, and its blast is Nuke::do_damage over many frames, a
# chapter of its own; unit_masks & ~0x4000000 — its one setter (5e56cc,
# add_move_facing_order's QUEUE_LAST arm) is a move a missile never gets.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate on run371's start (RON_STAGE, stage_walk,
# RON_STAGE_MAP for the territory) for what it carries:
# - 2710, 2712: 0/2010 at (28800, 18816), 0/2011 at (28800, 17280), no one's.
# - 2721, 2726: each queue one V2, charged 120/120 and 140/140.
# - 2925: V2b 0/14 inside 0/2010; 2930: V2c 0/15 inside 0/2011.
# - 2730, 2735: 1/2007 at (38016, 18816), cell owned by who=1;
#   1/2008 at (35712, 18816), cell unowned.
# This crate, unbuilt, then: re-points V2b to T_far on 3030 (no pass-over),
# draws the redraw on 3050 (as the reading), orders V2c on 3081 and
# launches it on 3111 (no shield at the order), and strikes T_far twice.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (§45 has the readings).
#
# 1. The issues reach the pump: 2721, 2726, 3020, 3030, 3081.
# 2. The staging: 0/2010 and 0/2011 on 2711/2713; 1/2007, 1/2008 on
#    2731/2736; V2b 0/14 on 2926, V2c 0/15 on 2931 (else 5 cannot fire).
# 3. The strike (3021): 0/14 one AIRATTACKGROUNDORDER to T_home's point,
#    flags 4; 0/2010's recharging 30 on 3021.
# 4. The re-press (3031): 0/14's order unchanged (the reading); re-pointed
#    to T_far (this crate).
# 5. The launch and its redraw (3050, block 3051): the frame's draws open
#    do_air_physics+0xba, then Ammo::init+0xae8, +0xb25 (the reading, both
#    sides); no redraw draw. Round sz within one step's climb of 466.
# 6. The shield at the order (3082): 0/15 inside with no order (the
#    reading); an order (this crate); 0/2011's recharging 0.
# 7. The shield at the blast (3169, block 3170): 1/2007 at damage 0 and
#    present, the round gone (the reading); struck (a building's wound).
# 8. The tech line (3071): who=1's Missile Shield held; no other field of
#    who=1 moves on the block but its clocks.
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
2215 resource who=0 all +500
2220 add missile_silo who=0 52,64
2225 @queueup 0 313 2 2009
2240 @queueup 0 310 2 2008
2440 @launchpatrol 0 7680 11520 2009
2460 @gatherpoint 0 7680 11520 0 2009
2500 @gatherpoint 0 5760 12288 0 2008
2650 @launchmove 0 2008 0 2009
2670 @launchstrike 0 2006 1 2009
2705 resource who=0 all +500
2710 add missile_silo who=0 150,98
2712 add missile_silo who=0 150,90
2720 @queueup 0 313 1 2010
2725 @queueup 0 313 1 2011
2730 add barracks who=1 198,98
2735 add barracks who=1 186,98
2740 add elite_special_forces who=0 190,98
2800 @gatherpoint 0 -1 -1 0 2008
3000 @launchstrike 0 2006 1 2008
3010 @launchpatrol 0 3840 13440 2008
3019 @launchstrike 0 2007 1 2010
3029 @launchstrike 0 2008 1 2010
3070 tech who=1 missile_shield on
3080 @launchstrike 0 2008 1 2011
