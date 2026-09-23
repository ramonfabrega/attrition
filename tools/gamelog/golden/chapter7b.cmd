# Golden record, chapter seven-b: the computer's civilians under the cheat.
#
# docs/GOLDEN.md §11's restage (the twelfth pass, item 628). Chapter seven
# staged the Civilian line for who=0, and its first falsifier fired: the
# human carries `leader_flags & 4`, so `Unit::think@005f6e40:205`'s block is
# entered for who=0 with or without `ai off` (docs/INPUT.md §11.9). This
# chapter stages the same five, Citizen, Caravan, Merchant, Scholar and Fur
# Trapper, for **who=1, the computer**, whose bit 4 is clear. Its lines are
# otherwise chapter seven's, and the one lever it moves is the owner.
#
# It is a PAIR. `chapter7b_control.cmd` is this file less the `0 !ai off`
# line and nothing else.
#
#   run156, the chapter:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch7b \
#       --map 14 --end-frame 1200 --log-window 605 1200 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,BUILDS=7,CITIES=5,GOODS=3,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter7b.cmd
#
#   run157, the control: the same line with `~/ron-golden/ch7bc` and
#       --cmd-file tools/gamelog/golden/chapter7b_control.cmd
#
# Chapter seven's header has the channel's rules and why the `--detail
# start:` line is not optional. The names resolve as they did there:
# Citizen 50, Caravan 59, Merchant 61, Scholar 52, Fur Trapper 400.
#
# ---------------------------------------------------------------------------
# THE STAGING, checked before the run (docs/GOLDEN.md §3, point 5).
#
# Coordinates are TILES (docs/INPUT.md §11.3). London, who=1's capital, is
# cell (55,21) and tile (220,84). Its start buildings stand at cells (52,22),
# (52,23), (53,21), (53,22), (54,22) and (56,21) (run142's BUILDDATA on 606),
# and the WORLD block's terrain blocks nearest the south-east are (56,22),
# (53..54,23), (52..54,24) and (57,25..26). The five go on the clear
# south-east side, on cells that are owned by 1 and unblocked in run141's
# third WORLD block:
#
#   citizen  (57,23) tile 228,92      caravan  (58,23) tile 232,92
#   merchant (57,24) tile 228,96      scholar  (58,24) tile 232,96
#   fur      (57,27) tile 228,108
#
# That is chapter seven's shape: two cells off the capital, the fur trapper
# four cells behind the citizen. The only building run142's AI raised by
# 1199 is at (54,32), well south. No capture on this disk has staged a
# who=1 unit on London's ground (grep of every golden `.cmd`).
#
# ---------------------------------------------------------------------------
# THE READING, done before the run. It says the premise falls before the
# first frame.
#
# 1. **Every unit of who=1 carries `unit_masks & 0x40000`.** `Unit::init
#    @00612100:585` sets it when `(leader_flags & 0xc) != 4`, which is true
#    for who=1's `0x03000013`. The dump agrees. Every who=1 unit in run141
#    prints 262152 or 262154 on 700 under `!ai off`, and run146's
#    cheat-`add`ed who=1 hoplites print 331790. All of them have `0x40000`
#    set.
# 2. **So the block's exit cannot fire on them.** For who=1, bit 4 is clear.
#    That means `:206` is entered only with `ai_off`. Its arms are guarded by
#    `uVar4 != 0` (bit 4), so they do not run. Its one unconditional
#    statement is `:264`, `if ((unit_masks & 0x40000) == 0) return`, and that
#    is false for all five. **The tail runs under `!ai off` exactly as
#    without it.**
# 3. **The citizen's arm is above the block anyway.** `think_peasant` and
#    `think_caravan` run at `:154`–`:165`, gated only by `leader_flags & 2`,
#    which both leaders carry. For an AI-driven unit, `think_peasant@005f5760`
#    waits for `idle >= 1`, not the human's 12 (`:17`–`:40`). It searches
#    with an unlimited range (`:95`), and a citizen tries
#    `think_civilian_transport` first (`:50`–`:57`).
#
# Taken together: **`ai off` reaches a computer's civilians only through
# `Leader::production_ai`** (docs/INPUT.md §11.4), the one other reader of
# the flag in the simulation's territory. That reader bails with the
# cheat on and plans with it off. Neither half of the block's predicate can
# be the thing that decides, on this lobby, for a unit born to its current
# owner.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS (docs/GOLDEN.md §11), and where each could first fire.
#
# 1. **A GATHERORDER, TRADEORDER or carry on any of the five in run156 at or
#    after the citizen's think_peasant wait: the block does not close the
#    tail.** It first fires on who=1's citizen `UNITDATA`. The citizen is
#    the first of the five born, on block 611, and its wait is `idle 1`, so
#    the fire comes on its first think frame, block 611 or 612, not ~763.
#    The record is its order stack, then `carry` once it cuts, then
#    LEADERDATA 1's gather slots. **PREDICTED TO FIRE**, by (1)–(3) above.
# 2. **No order on the citizen in run157 by the same block: the pair is
#    vacuous.** It could first fire on the same record and the same block.
#    **Predicted NOT to fire.** The citizen's order comes from
#    `think_peasant`, which the AI's economy does not gate.
# 3. **A draw-stream parting under the word in this crate's walk of either
#    capture: the seam is real.** It would first show on the citizen's order
#    frame (611 or 612), where this crate's `ai_off && !ai_driven(owner)`
#    and the original's block are both open for who=1 (orders.rs). They
#    agree there, so no parting is predicted from the seam. A parting
#    elsewhere is a residue booked by its frame (DECISIONS 42).
#
# check: every staged line runs. `cmdsran.py` shows nine `INFO cmd`
#        returning 1 in run156 (the seven here, `37 !ffwd`, `1200 !quit`)
#        and eight in run157.
# check: `MAP_STYLE 14`, seed 12345, and 597 blocks in run156 (596 in
#        run157): 1 (run156 only), 605..1199, 1201.
# check: frame 0 is one game in both, and they part on frame 1, as run141
#        against run142 did (12 draws against 54, the production AI).
# check: five new who=1 `UNITDATA` on 611, 616, 621, 626, 631 with GUY types
#        50, 59, 61, 52, 400, each with `unit_masks & 0x40000`. **Their `o`
#        differs between the pair**, because the AI trains in run157 and not
#        in run156 (run142 has who=1 o 0..8 on 606, run141 o 0..5). Identify
#        them by type and birth block, never by slot.
# check: the scholar takes no order in either run: `find_gather_spot` has no
#        university, and a 0x34 skips the repair and scout arms. The caravan
#        takes no TRADEORDER: London is who=1's only city, and chapter seven's
#        caravan found nothing. Lower confidence: the merchant and the fur
#        trapper reach `think_merchant` in the tail, which the human's never
#        did, and either may take an order in BOTH runs.
# check (the pair's own): whatever differs between run156 and run157 on the
#        five is `production_ai`'s. Candidate: the citizen's
#        `find_build_spot` (`:59`–`:65`, worker stance 1 or 2) finds a
#        planned site only when the AI plans, so a BUILDORDER in run157 and a
#        GATHERORDER in run156 is `ai off` reaching the civilians from
#        above, not through the block.
# ---------------------------------------------------------------------------

# `ai` is console-only (table index 12). DELETE THIS LINE for the control
# (`chapter7b_control.cmd` is that file, and differs from this one only here).
0 !ai off

# The Medieval age for the civilians' owner, as chapter seven's for who=0.
600 library who=1 2

# The same five, for the computer, on its own ground beside London.
610 add citizen who=1 228,92
615 add caravan who=1 232,92
620 add merchant who=1 228,96
625 add scholar who=1 232,96
630 add fur who=1 228,108
