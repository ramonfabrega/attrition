# Golden record, chapter forty-six — three arms a unit test alone held
# (DECISIONS 56 §3), two of them staged: a barge landing a squad's three
# riders, and a woodcutter's chain holding a live member that is not
# gathering there when a chopper's wait runs out.
#
# docs/GOLDEN.md §55 (item 1278). Chapter forty-three's cast (§52) on the
# golden start, its 1562 line dropped and its 1900 line naming the barge
# alone (parked 1238), and who=1's City trained a Citizen under a two-point
# gather list (parked 1255):
#
#     0 `!ai off`                                   every chapter's
#   600 `library who=0 1`, 604 a who=0 Dock         chapter forty-three's
#   610 `add chariot who=0 34,177`                  C 0/6
#   612 `add hoplite who=0 34,189`                  H 0/7, 0/8, 0/9: one squad
#   620 `resource who=1 all +500`                   the Citizen's price
#   622 `be 1`, `@gatherpoint 1 44160 12480 0 2000` the City 1/2000's first
#                                                   point, open ground (230,
#                                                   65), action 0: a waypoint
#   624 `@gatherpointadd 1 40512 17856 1 2000`      its second, on the
#                                                   Woodcutter's Camp 1/2001,
#                                                   action 1: the gather
#   626 `@queueup 1 50 1 2000`, 627 `be 0`          one Citizen W, 1/6
#   640 `@move 0 11904 34944 6`                     C to deep water, barge
#                                                   B1 0/10
#   880 `@move 0 14572 31200 10`                    B1 to the waterline
#  1269 `@move 0 15360 31200 10`                    B1 ashore: C out, B1 dead
#  1300 `@move 0 11904 36480 7`                     H to deep water: B2 0/10
#                                                   with the squad aboard
#  1400 `add citizen who=1 82,163`                  X 1/7 beside C
#  1560 `@move 0 14572 31200 10`                    B2 onto B1's point
#  1900 `@move 0 15360 31200 10`                    B2 ashore: dead, and the
#                                                   squad put out as a group
#
#   run496 (item 1278), the take:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch46 \
#       --map 14 --end-frame 2200 --log-window 605 2200 --timeout 5400 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter46.cmd
#
# The window is chapter forty-three's: the last line is 1900, the squad's
# moves end near 1931, and 2200 leaves 250 blocks past it.
#
# ---------------------------------------------------------------------------
# THE READINGS (docs/GOLDEN.md §55 has them whole).
#
# - `Unit::add_gather_order@0061a5c0` joins the building's chain on the
#   call, whatever the queue position: `Build::add_gatherer` pushes the
#   unit at `gather_down`'s head before the order is queued.
# - `Unit::come_out@00617c10`'s routing walks the City's list from its
#   head: the first point, a waypoint, is a `MOVE_TO` `QUEUE_LAST` with
#   the action bit; the last, a gather building of its own with action 1,
#   is `add_gather_order(QUEUE_LAST, 1)`. So W is in 1/2001's chain from
#   the frame it comes out, and its action is the move.
# - `Build::all_gathering@0062f570` calls `check_gatherers@0062f710` first,
#   which unlinks a member that `UnitData::is_gathering_at` refuses — W,
#   whose action is not the gather — and walks what is left.
# - `Object::eject_contents@0064cd20`'s `+0x308 != 1` arm (§52): a squad's
#   passenger comes ashore in a stack group of the boat.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate on run466's start (RON_STAGE,
# RON_STAGE_ALL), trace frames:
# - 725: W 1/6 out at (41640, 16872), `[MOVE_TO (44160, 12480) action,
#   GATHER 1/2001 action]`; 1/2001's chain `[6, 2, 1]`.
# - 1049: the chopper 1/2's wait runs out, with 1/1 out at its tile and W
#   on its first leg: the prune drops W, `all_gathering` answers yes, wait
#   −1 and the chain `[2, 1]`; 1/2 walks home from 1050. Without the prune
#   W's head is a move: no, and a `% 100 + 300` reroll on 1049.
# - 1400: X 1/7 joins the chain's head walking; dead on 1486, as §52's.
# - 1066: W at its waypoint; 1067 its gather, and the chain `[6, 2, 1]`.
# - 1901: B2 dead at (14572, 31200); 0/7, 0/8 and 0/9 ashore at (14712,
#   31224), (14712, 31368) and (14856, 31224), each under the group's move.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire. The walk prints the
# trace frame; its block is the next.
#
# 1. W not out by block 726, or its orders not `[MOVEORDER, GATHERORDER]`.
# 2. The prune: 1/2 not home-bound on block 1050 — its `order:gather.wait`
#    other than −1 on block 1050, or a `Unit::do_non_flat_gather+0xcc3`
#    reroll drawn on 1049.
# 3. The landing: 0/10 alive on block 1902, or 0/7..0/9 `inside` 10 there.
# 4. An `I_ISSUE` refusal on any line (parked 1256).

0 !ai off
600 library who=0 1
604 add dock who=0 53,153
610 add chariot who=0 34,177
612 add hoplite who=0 34,189
620 resource who=1 all +500
622 be 1
622 @gatherpoint 1 44160 12480 0 2000
624 @gatherpointadd 1 40512 17856 1 2000
626 @queueup 1 50 1 2000
627 be 0
640 @move 0 11904 34944 6
880 @move 0 14572 31200 10
1269 @move 0 15360 31200 10
1300 @move 0 11904 36480 7
1400 add citizen who=1 82,163
1560 @move 0 14572 31200 10
1900 @move 0 15360 31200 10
