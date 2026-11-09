# Golden record, chapter one hundred and thirty-five -- the Helicopter's stand-off and Delete on a building (item 3994,
# GOLDEN 187; parked 3967a; territory adjudication 2 row 48).
# Run1498's lines (AI 382.4: chapter fifteen's, with a General in the selection) and a Helicopter `0/13` (TI 310, armed,
# RANGE 1-7) in every selection, and the attack moved from 734 to 700. On 701 the General, unarmed and grouped, steps the 700 attack through
# `Unit::do_attack:119`'s grouped arm: `GroupData::get_max_range@00711080` over the members alive, on the map, not a
# plane (vslot +0xc0: a Helicopter is no plane), armed and not siege -- so the Helicopter's 7, not the Hoplites' 0 -- and
# the General's MOVEORDER lands (7 + 2) x 0xc0 off the Chariot, not run1498's (0 + 1) x 0xc0 + 0xc0.
# Then Delete (OPTION_DISBAND, `Group::action_disband@0070e260`) on a building group of one:
#   900: a finished Barracks added; 904 selected, 906 Delete: active, `count_queue` 0 -> `Build::queue_up(DISBAND, 0)`;
#        the entry runs 21000 at 100 a frame and the Barracks goes at its finish (GOLDEN 165's chain).
#   916: The Art of War (`library who=0 2`, chapter eighteen's: the Barracks' prerequisite); 920: a Citizen added; 922 a Barracks site dropped by `@build`; 926 the site selected, 928 Delete: not active ->
#        `Object::disband(o, 0)` at once, the refund the fraction not built.
# Killers, before the take (the journal's list): (1) the General's MOVEORDER on 702 lies (0 + 1) x 0xc0 + 0xc0 = 384
# off the Chariot before the snap -- the Helicopter's range does not count; (2) the finished Barracks gone on 908 (disbanded, not queued), or no
# BUILDQUEUE type 666 entry on it by 908; (3) the site alive on 930.
#   run1552 (cover=0), run1552b (cover=1 twin):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch135 \
#       --map 14 --end-frame 1250 --log-window 605 1250 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --bindkey OPTION_DISBAND=124 \
#       --cmd-file tools/gamelog/golden/chapter135.cmd
# bindkey OPTION_DISBAND=124
0 !ai off

610 add hoplite who=0 12,84
611 add general who=0 14,84
612 add hoplite who=0 18,84
613 add helicopter who=0 16,86
614 add chariot who=1 16,64

620 @move 0 3192 10752 6 9 10 13
700 @attack 0 6 1 6 9 10 13
860 @amove 0 3192 7680 6 9 10 13

900 add barracks who=0 60,72
904 select 2007 who=0
906 @key 124

916 library who=0 2
920 add citizen who=0 24,184
922 @build 0 7296 34176 427 14
926 select 2008 who=0
928 @key 124
