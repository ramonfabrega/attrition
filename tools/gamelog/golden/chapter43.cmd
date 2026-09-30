# Golden record, chapter forty-three — the landing's arms that no walk holds:
# a barge that dies stepping ashore on its figure's sixty-fourth frame,
# standing, and two Hoplite barges whose passengers are a squad's.
# `Unit::process@00610bc0`'s `Guy::process` over the dead boat's figure (its
# collision repaint, `Guy::process@005e0230`'s tail), and
# `Object::eject_contents@0064cd20`'s `uber_size > 1` arm.
#
# docs/GOLDEN.md §52 (item 1223; docs/DECISIONS.md 41 §1 and §5, 49, 56 §3).
# Chapter twenty's lake (§28) on the golden start.
#
#     0 `!ai off`                                   every chapter's
#   600 `library who=0 1`, 604 a who=0 Dock         the transport level
#                                                   (chapter twenty's)
#   610 `add chariot who=0 34,177`                  C 0/6
#   612 `add hoplite who=0 34,189`                  H 0/7, 0/8, 0/9: one squad
#   640 `@move 0 11904 34944 6`                     C to deep water: its cast
#                                                   on 702, barge B1 0/10
#   880 `@move 0 14572 31200 10`                    B1 to the north-east
#                                                   shore's waterline, tile
#                                                   (75, 162): standing there
#                                                   from 1081
#  1269 `@move 0 15360 31200 10`                    B1's first step is dry
#                                                   land: C ashore and B1 dead
#                                                   on 1270, `avg_speed` 0,
#                                                   and (1270 + 10) % 64 == 0
#  1300 `@move 0 11904 36480 7`                     H to deep water: 0/7's
#                                                   barge B2 0/10 (the number
#                                                   B1 held), the squad aboard
#                                                   it (item 1235); no B3
#  1400 `add citizen who=1 82,163`                  X 1/6 beside C: a ground
#                                                   death, for run467's DEATHS
#  1560 `@move 0 14572 31200 10`                    B2 onto B1's own point
#  1562 `@move 0 14572 31008 11`                    B3 a tile north of it:
#                                                   refused (no B3)
#  1900 `@move 0 15360 31200 10 11`                 B2 and B3 ashore: the
#                                                   DLL refuses the whole
#                                                   line for the absent
#                                                   `0/11` (refusal 3, item
#                                                   1248), so B2 stays at
#                                                   the waterline, the
#                                                   squad aboard, and the
#                                                   squad arm is not staged
#
#   run466 (item 1223), the take:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch43 \
#       --map 14 --end-frame 2200 --log-window 605 2200 --timeout 5400 \
#       --detail end:UNITS=3,GUYS=4,BUILDS=7,CITIES=5,LEADERS=5,GROUPS=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter43.cmd
#   run467, the second stanza (parked 1105): the same line into
#   ~/ron-golden/ch43d with `DEATHS=1` beside `GROUPS=1` at the end.
#
# `GROUPS=1` at `GUYS=4` for the squad's pool slot, as chapter twenty. The
# window is 1,595 blocks: the last scheduled line is 1900, the squad's group
# moves end near 1931, and 2200 leaves 250 blocks past it.
#
# ---------------------------------------------------------------------------
# THE READINGS (docs/GOLDEN.md §52 has them whole).
#
# - `Unit::set_new_location@005f8d20`'s sea arm: `eject_contents(0, -1, 1,
#   1)`, `num_inside`, `Object::die(0)` and return, **before** `+0x10`/`+0x14`
#   are written: the boat dies where it stood, and its figure is on its `des`.
# - `Unit::process@00610bc0` loops `Guy::process` after the think with no
#   test; `Guy::process@005e0230`'s tail, on `(frame + o) & 63 == 0` with
#   the figure's `avg_speed` (`+0x84`) 0, a non-air type and a non-zero
#   `+0x248`, re-marks the disc gated on the figure's own `get_tregion`.
#   A barge on the water half of a coastal cell marks nothing; at tile
#   (75, 162) it marks 28 cells, all water.
# - `eject_contents`' `+0x308 != 1` arm: `Unit::reset_move_orders@005fd080`
#   on the boat (each move's `+0x44`/`+0x48` := `x`/`y`), then — unless the
#   passenger is AI-driven and in an army — a group of the boat,
#   `Group::set_up_insert`, `Group::kill(boat)`, `Group::add(passenger)`,
#   `Groups::push_group(who, g, 1)` and `Group::finish_insert` on the slot.
# - The crew loop (`+0x304 .. +0xe8`) is empty on every boat that carries:
#   the five carriers are `CREW_SIZE 0`, `UBER_SIZE 1`.
#
# ---------------------------------------------------------------------------
# THE STAGING, walked by this crate on run460's start (RON_STAGE,
# RON_STAGE_ALL), trace frames:
# - 702: C casts; 703 B1 0/10 born with C inside; 855 at deep water.
# - 1081: B1 at (14572, 31200); its `avg_speed` 0 from 1091.
# - 1270: C ashore at (14712, 31224), B1 dead, `avg_speed` 0; the repaint
#   marks cells (300..303, 647..653).
# - 1355: 0/7 casts, B2 0/10 on 1356 with the whole squad aboard (item
#   1235; item 1223's walk put 0/7 alone aboard, and 0/9 cast a B3 0/11
#   the original never makes). The lines naming 11 act on nothing.
# - 1402: C takes X; X dies on 1486.
# - 1850: B2 on B1's point; 1841 B3 a tile north.
# - 1901: B2 dead, 0/7 ashore holding a group move to (15384, 31368); 0/8
#   walks from the west shore with its own; 1910: B3 dead, 0/9 ashore.
# - 1921, 1931: 0/7 and 0/9 at their points.
# - Run466 answers otherwise (item 1248): the 1900 line names an `0/11`
#   that was never made, and the DLL refuses the whole line (the trace's
#   `I_ISSUE` on 1900, refusal 3, object 11). B2 stands at the waterline
#   with the squad aboard to the capture's end, and falsifier 3 has
#   nothing to read.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire. The walk prints the
# trace frame; its block is the next.
#
# 1. B1 not dead on block 1271, or C not ashore there.
# 2. The dead boat's repaint read: B2's approach to (14572, 31200) differs
#    from this walk's between 1561 and 1850 (the only reader staged).
# 3. The squad arm: 0/7 on block 1902 with no MOVEORDER, or no GROUPDATA
#    slot holding 0/7's squad from 1902.
# 4. X not dead by block 1487 (run467: no DEATH record for it).

0 !ai off
600 library who=0 1
604 add dock who=0 53,153
610 add chariot who=0 34,177
612 add hoplite who=0 34,189
640 @move 0 11904 34944 6
880 @move 0 14572 31200 10
1269 @move 0 15360 31200 10
1300 @move 0 11904 36480 7
1400 add citizen who=1 82,163
1560 @move 0 14572 31200 10
1562 @move 0 14572 31008 11
1900 @move 0 15360 31200 10 11
