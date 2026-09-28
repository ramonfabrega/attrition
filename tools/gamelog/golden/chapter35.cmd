# Golden record, chapter thirty-five — the Helicopter's and missiles' launch
# arms: the resources staged, a Missile Silo, two V2 Rockets and two
# Helicopters trained with and without a gather point, and both launch
# commands pressed on each base.
#
# docs/GOLDEN.md §44 (item 1019; docs/DECISIONS.md 41 §1 and §5, 49).
# Chapter thirty-four's cast to its second Airbase (2210) — the four planes
# in 0/2007 with no order, the enemy Barracks 1/2006 at (72, 78), 0/2008
# empty at (44, 76) — then:
#    2215 `resource who=0 all +500`              bucket_set on six goods
#    2220 `add missile_silo who=0 52,64`         0/2009
#    2225 `@queueup 0 313 2 2009`                two V2 Rockets
#    2240 `@queueup 0 310 2 2008`                two Helicopters
#    2440 `@launchpatrol 0 7680 11520 2009`      verb 21 at the silo, V2 #1 inside
#    2460 `@gatherpoint 0 7680 11520 0 2009`     P_s, tile (40, 60), the silo's
#    2500 `@gatherpoint 0 5760 12288 0 2008`     P_h, tile (30, 64), 0/2008's
#    2650 `@launchmove 0 2008 0 2009`            verb 23, MOVE_TO from the silo
#    2670 `@launchstrike 0 2006 1 2009`          verb 23, ATTACK from the silo
#    2800 `@gatherpoint 0 -1 -1 0 2008`          the Clear at 0/2008
#    3000 `@launchstrike 0 2006 1 2008`          verb 23 at 0/2008, a Helicopter inside
#    3010 `@launchpatrol 0 3840 13440 2008`      verb 21 at 0/2008, P2 tile (20, 70)
#
#   run371 (item 1019):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch35 \
#       --map 14 --end-frame 3260 --log-window 605 3260 --timeout 10800 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --cover cover=1 \
#       --cmd-file tools/gamelog/golden/chapter35.cmd
#
# The window is 2,655 blocks: the last staged event is the press at 3010
# (processed 3011, read 3012); 3260 leaves 250 blocks past it.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate on run362's start (RON_STAGE, the golden
# harness's stage_walk) for what it carries — the resources, the silo, the
# queues and the births; the arms after each birth are the reading's
# (docs/PRODUCTION.md, "The Helicopter and the missile under a point"):
# - 2215: who=0's buckets +500 each; 2220: the silo 0/2009 at (9984, 12288).
# - 2226: 0/2009's queue [V2, V2]; 2241: 0/2008's [Helicopter, Helicopter].
# - 2430: V2 0/10 in 0/2009, no order.
# - 2445: Helicopter 0/11 born and out at once, two draws, no order.
# - 2442: nothing (a missile takes no launch patrol).
# - 2462: 0/2009's list [P_s]; no hangar walk at a silo.
# - 2502: 0/2008's list [P_h]; 0/11 outside with no order is homed nowhere.
# - 2635: V2 0/12 in 0/2009 under [P_s], no order (a missile on the ground).
# - 2652: nothing (MOVE_TO refuses a missile).
# - 2658: Helicopter 0/13 in 0/2008 under [P_h]: a patrol over P_h, flags 4;
#   launched by 0/2008 and flying to P_h.
# - 2672: 0/10 a STRAFEORDER on 1/2006, mandatory 1, flags 4; 0/2009's
#   recharging 30 at its launch, and 0/10 out thirty frames later.
# - 2802: 0/13 a STRAFEORDER home to 0/2008, mandatory 0; lands with none.
# - 3002: nothing (a Helicopter's tank is 0, so its reach is 0).
# - 3012: 0/13 a MOVE_TO to P2's cell centre (3864, 13464), flags 4; out.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (§44 has the readings).
#
# 1. The issues reach the pump: 2226, 2241, 2441, 2461, 2501, 2651, 2671,
#    2801, 3001, 3011.
# 2. The resource line: who=0's six buckets +500 on 2215 (LEADERDATA).
# 3. The silo is 0/2009 on 2221 (else 4 onward cannot fire).
# 4. Helicopter 0/11's birth (2445): out, no order (the reading); inside.
# 5. The silo's launch patrol (2442): nothing (the reading); 0/10 a patrol.
# 6. 0/2008's point (2502): 0/11 untouched (the reading); 0/11 a patrol.
# 7. V2 0/12 under [P_s] (2635): no order (the reading); a patrol over P_s.
# 8. MOVE_TO from the silo (2652): nothing (the reading); a flight home.
# 9. Helicopter 0/13 under [P_h] (2658): a patrol, flags 4 (the reading);
#    no order; a patrol flags 0.
# 10. The silo's strike (2672): 0/10's strike, flags 4 (the reading); 0/12's;
#     none. Then recharging 30 and 0/10 out on the thirtieth frame.
# 11. The Clear (2802): 0/13 a strafe home, mandatory 0 (the reading); 0/11's.
# 12. The strike at 0/2008 (3002): nothing (the reading); 0/13 a strike.
# 13. The patrol at 0/2008 (3012): 0/13 a MOVE_TO, flags 4 (the reading); an
#     AIRPATROLORDER; nothing.
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
2800 @gatherpoint 0 -1 -1 0 2008
3000 @launchstrike 0 2006 1 2008
3010 @launchpatrol 0 3840 13440 2008
