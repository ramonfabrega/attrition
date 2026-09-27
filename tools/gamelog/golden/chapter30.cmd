# Golden record, chapter thirty — the gather point: a Barracks' rally on
# the ground, on another building and on itself, a City's on a forest, and
# the Clear.
#
# docs/GOLDEN.md §39 (item 928; docs/DECISIONS.md 41 §1 and §5, 49).
# Chapter twenty-eight's cast to 614 (two who=0 Barracks `0/2007` and
# `0/2008`, a Chariot and a Hoplite squad, `!ai off`), then the player's
# rally points and production:
#    616 `@gatherpoint 0 1344 12096 0 2007`   2007's point on open ground
#                                             (tile 7,63, north-west)
#    620 `@queueup 0 132 1 2007`              Hoplites (type 132) at 2007
#    640 `@queueup 0 170 1 2007`              Bowmen (type 170) behind them
#    650 `@gatherpoint 0 3936 28128 0 2000`   the City's point on a forest
#                                             tile (20,146), 560 units from
#                                             its Woodcutter `0/2001`
#    660 `@queueup 0 50 1 2000`               a Citizen (type 50) there
#    700 `@gatherpoint 0 4224 14208 1 2008`   2008's point on itself
#    710 `@queueup 0 132 1 2008`              Hoplites at 2008
#    900 `@gatherpoint 0 4224 14208 1 2007`   2007's point moved onto 2008,
#                                             between the Hoplites' finish
#                                             and the Bowmen's (parked 879)
#   1100 `@gatherpoint 0 -1 -1 0 2007`        the Clear button, after both
# `@gatherpoint <who> <x> <y> <action> <b>…` is the DLL's call of
# `CommandManager::issue_gather_point@00941b20(group, x, y, action,
# add_to_end 0)` (verb 20, new here): action 0 is a right-click on the
# ground, 1 on a friendly object at that object's point, and −1, −1, 0 the
# Clear button's (`Options::do_clear_gather@0071ce70`). `@queueup` is
# chapter twenty-four's verb 18.
#
#   run312 (item 928):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch30 \
#       --map 14 --end-frame 1450 --log-window 605 1450 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter30.cmd
#
# The window is 845 blocks. The last falsifier is the Bowmen's garrison
# into 2008, predicted by 1150; 1450 leaves 300 blocks past it.
#
# ---------------------------------------------------------------------------
# THE COMMAND, under the emulator before the capture (item 928, step 1).
#
# - `issue_gather_point(group, x, y, action, add)` appends 17 bytes,
#   `16 [x][y][action][add]`, each as passed, behind the group (22 with a
#   fresh one-building group); −1, −1 is carried as passed.
# - `Group::action_gather_point@006ff1b0(x, y, action, add)` on a
#   synthesized group of buildings (stub vtables), `Build::add_gather_point@
#   00622e70` and `Build::clear_gather@00623180` run unchanged:
#   a point: one `GatherPoint` {x, y, action}, the list cleared first
#   (QUEUE_NEW); with `add` it is appended (QUEUE_LAST); −1 in either
#   coordinate: `clear_gather` on every member, the list empty; a point
#   past the world's edge is clamped to it; a member that is a University
#   or a Missile Silo is passed over, and so is one that is neither a
#   training building (`build_flags & 0x80000000`) nor has a garrison
#   limit, nor is a Terracotta Army, Kremlin or Senate; a click on a
#   building tile (`& 3 == 3`) the member itself covers stores (−1, −1, 0),
#   the "inside" point, and on a Senate, Kremlin or Terracotta clears it;
#   a group whose members are all City centres (`COUNT_TYPE` VILLAGE)
#   clicking a forest tile (`& 0x30 == 0x30`) or a mountain (`& 3 == 2`)
#   asks `find_building` for its own Woodcutter or Mine within 0x600 and
#   moves the point onto that building's own point, action unchanged.
# What the emulator did not reach: `Unit::come_out@00617c10`'s gather-point
# arms (the exit's bearing and the order after it), which read the world
# and the unit at process time; the chapter measures them.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate through the commands' own entries on
# run308's start (every golden capture is this game to 605), with
# `@gatherpoint` skipped as not modelled. No staging run: run313 was not
# used. Every value is this crate's:
# - 617, 651, 701, 901, 1101: the presses, each skipped.
# - 622: 2007 [132 at 100], food 203, timber 203.
# - 642: 2007 [132, 170], wealth 61.
# - 662: 2000 [Citizen at 100].
# - 712: 2008 [132 at 100], food 130, timber 121.
# - 760: the Citizen `0/10` out at (3192, 31800), no order; a GATHERORDER
#   of its own on 919.
# - 856: the Hoplites `0/11`..`0/13` out on 2007's south ring, (2712,
#   14904), (2712, 15048), (2856, 15000), no order.
# - 953: the Hoplites `0/14`..`0/16` out on 2008's south ring, no order.
# - 1060: the Bowmen `0/17`..`0/19` out at (2424, 14808), (2424, 14952),
#   (2568, 14904), no order.
# The tiles: (7,63) mask 0 (plain land); (20,146) 0xf13c (forest);
# 2008's centre (22,74) 0x6003 (a building).
0 !ai off
606 add barracks who=0 14,74
608 add barracks who=0 22,74
610 add chariot who=0 20,60
614 add hoplite who=0 10,90
616 @gatherpoint 0 1344 12096 0 2007
620 @queueup 0 132 1 2007
640 @queueup 0 170 1 2007
650 @gatherpoint 0 3936 28128 0 2000
660 @queueup 0 50 1 2000
700 @gatherpoint 0 4224 14208 1 2008
710 @queueup 0 132 1 2008
900 @gatherpoint 0 4224 14208 1 2007
1100 @gatherpoint 0 -1 -1 0 2007
