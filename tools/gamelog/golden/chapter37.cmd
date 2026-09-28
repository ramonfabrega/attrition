# Golden record, chapter thirty-seven — the nuke: its research at the silo,
# its training, its strike and launch, the shield given after the press, the
# round with no scatter, and the blast's forty frames of Nuke::do_damage.
#
# docs/GOLDEN.md §46 (item 1091; docs/DECISIONS.md 41 §1 and §5, 49).
# A cast of its own on the golden start (chapter six's first two lines), NOT
# chapter thirty-six appended: the research is ~1,600 frames that place
# nothing, and it starts at 615 so the window stays near run390's.
#
#     0 `!ai off`                                  every chapter's
#   600 `library who=0 6`                          Modern Age and
#                                                  Nation-in-Arms: the
#                                                  nuke's two PREQs
#   606 `resource who=0 all +2000`                 the research (1000k/1200o)
#                                                  and the train (500k/600o)
#   610 `add missile_silo who=0 100,180`           0/2007
#   615 `@queueup 0 315 1 2007`                    the nuke's RESEARCH: its
#                                                  FLAGS carry no `h`, so the
#                                                  bit is clear; done 2236
#  2300 `@queueup 0 315 1 2007`                    the TRAIN; born 3021 as 0/10
#  2940 `add barracks who=1 120,180`               T 1/2006, ground zero
#                                                  (23040, 34560), no one's land
#  2942 `add barracks who=1 131,185`               F 1/2007, building distance
#                                                  1824: struck on t=39, 13/256
#  2950 `add elite_special_forces who=0 122,180`   0/6, d 518: t=17, 187/256
#  2952 `add elite_special_forces who=0 120,174`   0/7, d 1039: t=26, 118/256
#  2954 `add elite_special_forces who=0 110,180`   0/8, d 1804: t=39, 16/256
#  2956 `add elite_special_forces who=0 110,182`   0/9, d 1870: never (r stops
#                                                  at 1859, under 1920)
#  3050 `@launchstrike 0 2006 1 2007`              the nuke on T
#  3060 `tech who=1 missile_shield on`             the shield after the press,
#                                                  the nuke counting down
#  (3081: the launch; 3200: the landing, t=0; 3239: t=39, the last frame)
#
#   run397 (item 1091):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch37 \
#       --map 14 --end-frame 3490 --log-window 605 3490 --timeout 9000 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1,AMMO=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter37.cmd
#   (cover=0 on the click-free lane, 1011's split; `Nuke::add_nuke@0092ba30`
#   is a NEVER row, so run398 is the same script at cover=1 on the queue lane.)
#
# The window is 2,885 blocks: the last scheduled event is t=39, trace frame
# 3239, block 3240; 3490 leaves 250 blocks past it. One nuke: Armageddon's
# limit is 4 + 1·nations + 2·sides (Game::get_armageddon@00594020), 10 here.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate on run390's start (RON_STAGE, stage_walk,
# RON_STAGE_ALL for the ground units), trace frames:
# - 616: the silo's queue [(nuke, 1000k/1200o)]; 2236: empty, the bit set,
#   nothing placed.
# - 2301: [(nuke, 500k/600o)]; 3021: 0/10 inside 0/2007.
# - 2940, 2942: T at (23040, 34560), F at (25152, 35520).
# - 2950..2956: 0/6 (23544, 34680), 0/7 (23160, 33528), 0/8 (21240, 34680),
#   0/9 (21240, 35064).
# - 3051: 0/10 an AIRATTACKGROUNDORDER to (23040, 34560), flags 4.
# - 3081: the launch; unbuilt, this crate draws the V2's doubled scatter.
# - 3200: the landing; unbuilt, this crate takes the V2's general arm and
#   destroys T and F both, and strikes no probe.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire (§46 has the readings).
# The walk prints the trace frame; its block is the next (§17, §45).
#
# 1. The issues reach the pump: process_queue_up on 616 and 2301,
#    process_flight on 3051.
# 2. The research (block 2237): the entry gone, nothing placed, no unit of
#    type 315; its price on 617, 1000k/1200o.
# 3. The train (block 3022): 0/10 inside 0/2007; 500k/600o on 2302.
# 4. The strike (block 3052): 0/10 one AIRATTACKGROUNDORDER to (23040,
#    34560), flags 4, oxx 2007; recharging 30.
# 5. The shield after the press (blocks 3061..3081): 0/10's order kept, the
#    launch on trace 3081.
# 6. The launch (block 3082): 0/2007's `visible` -1 (0xff); the round `ex ey`
#    exactly (23040, 34560) and no scatter draw on trace 3081; total_time 120.
# 7. The landing (block 3201): T gone; who=0's nuke_stamp 3200 and nukes_used
#    1; no probe struck on 3200.
# 8. The ring (blocks 3218, 3227, 3240): 0/6 struck on trace 3217, 0/7 on
#    3226, 0/8 on 3239; each once.
# 9. The edge (block 3240 on): 0/9 untouched at d 1870.
# 10. F (block 3240): wounded on 3239, not destroyed, and not before.
#
0 !ai off
600 library who=0 6
606 resource who=0 all +2000
610 add missile_silo who=0 100,180
615 @queueup 0 315 1 2007
2300 @queueup 0 315 1 2007
2940 add barracks who=1 120,180
2942 add barracks who=1 131,185
2950 add elite_special_forces who=0 122,180
2952 add elite_special_forces who=0 120,174
2954 add elite_special_forces who=0 110,180
2956 add elite_special_forces who=0 110,182
3050 @launchstrike 0 2006 1 2007
3060 tech who=1 missile_shield on
