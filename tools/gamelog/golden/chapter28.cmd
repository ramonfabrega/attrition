# Golden record, chapter twenty-eight — two buildings under one command:
# `Group::action_queue_up`'s sort and passes across two Barracks, the
# infinite-queue toggle on two, and the command's building group of two
# in the pool.
#
# docs/GOLDEN.md §37 (item 888; docs/DECISIONS.md 41 §1 and §5, 49).
# Chapter thirteen's cast to 614 with a second who=0 Barracks `0/2008`
# eight tiles east of `0/2007`, then:
#    620 `@queueup 0 170 1 2007`        Bowmen at 2007 alone: 2007 queued 1
#    640 `@queueup 0 132 3 2007 2008`   Hoplites, num 3, the busier listed
#                                       first: the sort and the passes
#    700 `@buildmask 0 64 2008`         2008's infinite bit on
#    720 `@buildmask 0 64 2008 2007`    the toggle on [on, off]
#    740 `@queueup 0 132 1 2007 2008`   refused on price; the pool: the
#                                       same two in the other order
#    760 `@queueup 0 132 1 2007 2008`   refused; the pool: the same group
# The squads trained out of both Barracks (825 to 1324) are pushed into
# the pool behind the building groups.
#
#   run304 (item 888):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch28 \
#       --map 14 --end-frame 1580 --log-window 605 1580 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,LEADERS=2,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter28.cmd
#
# The window is 975 blocks. The last falsifier is the fifth squad's seat,
# 1324 by this crate's walk; 1580 leaves 256 blocks past it.
#
# ---------------------------------------------------------------------------
# UNDER THE EMULATOR FIRST (item 888, step 1; a scratch script on
# tools/emu/callfn.py's machine, out of git). Real `Build::vftable` and
# `Group::vftable` objects; `Build::queue_up@00620f40`,
# `LeaderData::researching@006db510` and the type record's `is` stubbed.
# - `Group::action_queue_up@006fdbb0`, a train job (the bit set): the
#   members sorted once by `queued` (`+0x82`, unsigned byte), least first,
#   a swap only on strictly less; then `num` passes, each over every
#   member alive (`+8 & 1`) and finished (`+8 & 4`): `queue_up(type, 1)`,
#   whose answer is not read. Two empty: 2007, 2008 (num 1); num 3:
#   2007, 2008, 2007, 2008, 2007, 2008. [q2, q0]: 2008 first. [q1, q1]:
#   list order. num 0 and -1: nothing. One not finished: skipped, and a
#   member at the sort's slot i that is not finished is not a pivot. The
#   action writes nothing on a member.
# - The research arm (the bit clear): once, on the idle member first, else
#   the least queued; a refusal goes on to the next.
# - `Group::action_buildmask@006fc9a0(0x40)`: [on, off] -> [off, off];
#   [off, on] -> [on, off]; [off, off] -> [on, on]; [on, on] -> [off, off].
# - `Group::add@00714350` twice, then `Groups::push_group@0070f9e0(0, g,
#   1)`: one record, `buildings 1`, num 2, the list in the command's
#   order, stamp the frame, speed 0; nothing written on a building.
#   Pushed again: `equals_group` against `last_group`'s record, equal ->
#   nothing written. [2008, 2007] after [2007, 2008]: not equal (the
#   compare is ordered) -> a new seat. A duplicate is dropped by
#   `GroupData::member`.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate through the commands' entries
# (`input::group_queue_up`, `input::group_buildmask`) on run285's start
# (chapter twenty-four's capture, this game to 607), with a prototype of
# the two-building seat for the pool's values (run305 was not used):
# - 610: `0/2008` at (4224, 14208).
# - 622: `0/2007` [170 at 100] 41 timber 51 wealth; food 254, timber 200,
#   wealth 62.
# - 642: `0/2008` [132 at 100 53/41, 132 60/50]; `0/2007` [170, 132 56/45,
#   132 65/56]; food 22, timber 9 (the fourth entry by 7 timber).
# - 702: `0/2008` 4160. 722: both 4096.
# - pool: 622 slot 1 [2007]; 642 slot 0 [2007, 2008]; 702 slot 1 [2008];
#   722 slot 0 [2008, 2007]; 742 slot 1 [2007, 2008] stamp 741; 762 the
#   same.
# - 825: the Bowmen `0/10`..`0/12` out of 2007, slot 0; 876: Hoplites
#   `0/13`..`0/15` out of 2008, slot 1; 1067 `0/16`.. slot 2; 1126 `0/19`..
#   slot 3; 1324 `0/22`.. slot 4.
0 !ai off
606 add barracks who=0 14,74
608 add barracks who=0 22,74
610 add chariot who=0 20,60
614 add hoplite who=0 10,90
620 @queueup 0 170 1 2007
640 @queueup 0 132 3 2007 2008
700 @buildmask 0 64 2008
720 @buildmask 0 64 2008 2007
740 @queueup 0 132 1 2007 2008
760 @queueup 0 132 1 2007 2008
