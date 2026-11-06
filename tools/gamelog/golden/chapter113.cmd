# Golden record, chapter one hundred and thirteen -- a peasant alarm (item 3916, GOLDEN 163; AI 391.2 row 9, parked 3906c).
# Chapter thirty-nine's lobby (GOLDEN 48: `library who=0 1`, the Citizen A `0/6` at 30,150), without its casts or its
# Militia tech. Two presses of the key bound to OPTION_PEASANT_ALARM, each on one selected Citizen of who=0:
# `Options::exec@007188c0`'s 0x1b arm -> `Options::do_alarm@0071c3f0` (a `group` and a one-byte `alarm`, 0x1b); the pump
# runs `CommandPackage::process_alarm@00947ef0` -> `Group::action_alarm@0070ec30`, whose unit-group arm (+0x49 clear) is
# `Group::action_alarm_peasant@006fd980`: each member of type 0x32/0x33 asks
# `ObjectsData::find_building(pos, SEARCH_FRIENDLY, who, -1, 0x200, FILTER_CAN_GARRISON, o, who)` -- range -1, the
# whole map, and 0x200 the flags word (the searcher's region), the listing's pushes at 6fda58..6fda6e -- drops every
# GARRISONORDER (0x1a; GATHER is 7) action with `repath` + `kill_current_order(0)`, and takes a GARRISONORDER at the
# head, search 1, the action bit, as `add_garrison_order(b, who, 1, QUEUE_FIRST, 1)` lays it.
#   622: A `0/6`, idle (run422 on 619: no orders), 15 tiles from the City `0/2000` at (3168, 30816).
#   642: the start's Citizen `0/1`, on its GATHERORDER at the Woodcutter `2001` (run422 on 619): kept under the order.
#   680: A again, mid-walk to the City (run1510: inside on 746), its action the GARRISONORDER of 622: the drop arm.
# Falsifiers, before the take: (1) trace 622 or 642: `INFO 22` with KEYMAP_NONE (the entry refused the selection);
# (2) no `process_alarm 623` / `643` on the CM channel; (3) block 624: `0/6` holding no GARRISONORDER -- AI 391.2's
# reading of 0x200 as a 512-unit radius, which refuses a City 15 tiles off, against this script's reading (range -1);
# (4) block 644: `0/1` still holding its GATHERORDER, or its GARRISONORDER not at the head -- FIRED on run1510's first
# take: the loop's 0x1a is GARRISON, so the gather stays; (4b) block 682: `0/6` holding two GARRISONORDERs; (5) either order naming a
# building other than `0/2000`, or `search` not 1; (6) the cover=1 twin: no HIT on `Group::action_alarm_peasant@006fd980`
# or `Options::do_alarm@0071c3f0`.
#   run1510c (cover=0), run1510d (cover=1, `--cover cover=1 --no-ffwd`), into ch113c and ch113d; run1510/1510b were the
#   first take, without the 680 press:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch113c \
#       --map 14 --end-frame 900 --log-window 605 900 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --bindkey OPTION_PEASANT_ALARM=124 \
#       --cmd-file tools/gamelog/golden/chapter113.cmd
# bindkey OPTION_PEASANT_ALARM=124
0 !ai off
600 library who=0 1
606 add citizen who=0 30,150
620 select 6 who=0
622 @key 124
640 select 1 who=0
642 @key 124
678 select 6 who=0
680 @key 124
