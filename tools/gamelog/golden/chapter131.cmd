# Golden record, chapter one hundred and thirty-one -- a gather point on a unit that dies, and a Disband queued at a
# second Library then cancelled (item 3989, GOLDEN 182; adjudication 2 rows 32 and 31). Chapter one hundred and
# twenty-five's lobby (GOLDEN 175: the golden start, map 14, the start's Library `0/2005`), its Small City `0/2007` at
# (40, 160) and a second Library `0/2008` in it.
#   Arm 1: `Build::replace_gather@00622d10`, from `HotKeyGroups::kill@00714e30` in `Unit::close@0060ee50` on a unit
#          that is no squad's follower (`+0x8e < 0`). The City `0/2000` takes an action-3 point on the idle Citizen
#          `0/4` (`x` 4, `y` 0: the point names `(o, who)`), and `die 4,0` closes the Citizen. The point's target has
#          the dying unit's captain (vslot `0xe4`, `get_captain`) and no hotkey group names a successor, so the point
#          becomes action 0 at the Citizen's own position (`+0x10`/`+0x14`, decoded). A Citizen then trained at
#          `0/2000` walks to it.
#   Arm 2: `Build::action_unqueue@00620280` and `Build::unqueue@006207c0` on a library: each forwards to
#          `get_first_library` unless the slot it names holds `DISBAND` (0x29a). `0/2008` queues `DISBAND`
#          (`Build::queue_up` never forwards it) and a cancel of its slot 0 stays at `0/2008`: the entry goes, and
#          the Library stands.
#   Arm 3 (closed by the disk, not staged): `Group::kill@00714110`'s `clear(-1)` at `num == 0` and a re-push onto the
#          slot -- ch40's bell re-pushes onto slot 0 emptied on 702, ch16's slot 0 is cleared on 804 and re-pushed on
#          902, and both pools are widened whole on closed words.
# Killers, before the take (a cheat on trace frame F shows on block F+2):
#   (1) `0/2000`'s GATHERPOINT not (4, 0, action 3) on 612 -- the verb missed;
#   (2) `0/2008` reads `queued` other than 1 on 632 -- the original refuses or forwards a Library's DISBAND;
#   (3) `0/2008` reads `queued` 1 on 652, or `0/2005` moves -- the cancel was forwarded (the reading is wrong);
#   (4) `0/2000`'s point still action 3 on 672, or action 0 anywhere but `0/4`'s last position -- replace_gather
#       did not run, or runs otherwise;
#   (5) `0/2008` gone before the end -- the Disband stood.
#   run1547 (cover=0), with chapter one hundred and twenty-five's lobby:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch131 \
#       --map 14 --end-frame 1000 --log-window 605 1000 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=9,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 --ai-tribe 11 --profile DIFFICULTY=0 --profile STARTING_RESOURCES=1 \
#       --profile STARTING_TECHNOLOGY=0 --profile ENDING_TECHNOLOGY=7 \
#       --cmd-file tools/gamelog/golden/chapter131.cmd
0 !ai off
601 resource who=0 all +3000
603 add small_city who=0 40,160
606 add library who=0 34,152
610 @gatherpoint 0 4 0 3 2000
630 @queueup 0 666 1 2008
650 @unqueue 0 0 2008
670 die 4,0
690 @queueup 0 50 1 2000
