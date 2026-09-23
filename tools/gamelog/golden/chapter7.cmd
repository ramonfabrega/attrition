# Golden record, chapter seven — the civilians, and what AI-off takes away.
#
# docs/GOLDEN.md §11. The Civilian line — Citizen, Caravan, Merchant,
# Scholar, Fur Trapper — on their owner's own ground, with the Leader AI off.
# The premise is a PREDICTION rather than a setup: `Unit::think@005f6e40`'s
# tail is gated by `if ((leader_flags & 4) != 0 || ai_off != 0)`, whose one
# unconditional statement returns for a unit without the AI-driven mask
# (docs/INPUT.md §11.4). So with `!ai off` these five should do NOTHING —
# no gather, no trade, no carry — and a GATHERORDER or a TRADEORDER in the
# dump falsifies the reading of that gate.
#
# This chapter is a PAIR, the way chapter one and run104 are: the same file
# run again with the `0 !ai off` line deleted is the control, and there the
# same five units should act.
# TWO captures, one file each (item 578). The recipe is run133's tested
# command with the chapter file swapped, one line a capture:
#
#   run141, the chapter:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch7 \
#       --map 14 --end-frame 1200 --log-window 605 1200 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,BUILDS=7,CITIES=5,GOODS=3,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter7.cmd
#
#   run142, the control: the same line with `~/ron-golden/ch7c` and
#       --cmd-file tools/gamelog/golden/chapter7_control.cmd — this file
#       with the `0 !ai off` line deleted and nothing else changed.
#
# The `--detail start:` line is not optional: without it the start block
# carries no leaders and no units, and `walk_chapter` refuses the capture
# (run112's first take). A relative `--cmd-file` is resolved against the
# repo root by `golden_capture.sh` since item 415. `--timeout` bounds the
# game, and the runner's give-up is a truncation (docs/RUNS.md run107):
# read `receipt.json` before the dump, and wait on the runner's exit.
#
# The names resolve by `ConsoleWin::parse_type`'s order, unit table first,
# and each is its table's first match: `citizen` → Citizen (record 0,
# TypeIndex 50), `caravan` → Caravan (9, 59), `merchant` → Merchant (11,
# 61, ahead of Merchant Fleet at 268), `scholar` → Scholar (2, 52), `fur`
# → Fur Trapper (350, 400). All five are UBER_SIZE 1. No capture on this
# disk has staged any of them (grep of every `rontrace.cmd`, item 578).
#
# The channel's rules this file obeys (docs/ORACLE.md, "The cheat channel"):
# a `!` line reaches the console-only half of `run_cmd`'s two switches and a
# bare line only the chat half; lines run in file order and a frame below its
# predecessor is clamped; and `no_mouse = 1`, so an `add` without an `x,y`
# silently reuses a stale cursor — every placement here carries one.
#
# Coordinates are TILES (docs/INPUT.md §11.3), read off run105's own
# `[Start Game]` WORLD block: player 0's capital Napata is cell (4,40),
# tile (16,160), and cells (6,40) and (6,44) are free owned ground beside
# it — tiles 24,160 and 24,176.

# ---------------------------------------------------------------------------
# THE PREDICTION, written before run141 and run142 so the captures can fail
# (item 578). As chapters two, four and five: a golden chapter takes no
# `captures.txt` stanza, so the stanza's `check:` lines live here. Every one
# is a grep or a count. A dump's block label is the sim frame plus one, so a
# line staged at 610 lands on block 611.
#
# check: every staged line runs — `python3 tools/gamelog/cmdsran.py <trace>`
#        shows each `INFO cmd` returning 1: the seven here plus `37 !ffwd N`
#        and `1200 !quit`, which the capture adds (nine in run141, eight in
#        run142, which has no `!ai off`).
# check: `MAP_STYLE 14` and seed 12345 read back from each dump's GAME INFO.
# check: each window is the window asked for — `BEGIN FRAME` 1, then
#        605..1199 with no gap, then 1201: 597 blocks, in each capture.
# check: **the two captures are the same game until the gate can matter.**
#        The flag is read first by `Leader::production_ai` on frame 1
#        (docs/INPUT.md §11.4: frame 0 is unchanged by the line and frame 1
#        is not), so the two traces' per-frame draw counts are identical on
#        frame 0 and part on frame 1 — as chapter one against run104 did,
#        54 draws against 12.
# check: five new player-0 `UNITDATA` in run141 on blocks 611, 616, 621,
#        626 and 631, one each, the next five `o` after the start block's
#        0/0..0/5, each with a `GUY type` of 50, 59, 61, 52 and 400.
#
# **What the decompile says, read before the run (item 578).** §11's premise
# was written on the reading of `leader_flags & 4` that item 437 retired: the
# bit is SET on who=0, the human (docs/COMBAT.md §28.1). So for the human's
# units `Unit::think@005f6e40:205`'s block is entered whether `ai_off` is set
# or not, and its exit — `unit_masks & 0x40000` clear → return — fires in
# both captures. `ai off` cannot tell the human's five apart; it changes the
# computer's units and the production AI. And the two arms that would issue
# the chapter's orders sit ABOVE the block (`think:154`–`165`, gated only by
# `leader_flags & 2`): `think_peasant` for a worker (TypeIndex 0x32–0x35)
# and `think_caravan` for a caravan, each after the human's idle wait (12,
# item 494). This crate already models both there.
#
# check: **the citizen `0/6` takes a `GATHERORDER` in BOTH captures**, from
#        `think_peasant` on the frame its `idle` reaches 12, and within
#        ~40 blocks of 611. GOODS=3's stock for player 0 then rises on a
#        delivery; a quiet stock is read against the reader, not called
#        agreement.
# check: **the caravan `0/7` takes a `TRADEORDER` in BOTH captures**, from
#        `think_caravan` (`find_city(SEARCH_ALLIED, FILTER_CAN_TRADE)`) —
#        Napata, the only allied city — on the same idle wait.
# check: the scholar, merchant and fur trapper take NO order in either
#        capture: the scholar's `find_gather_spot` has no university to
#        find, and the other two reach only the rare collector's deploy
#        (which wants `unit_masks & 0x80000`) and the tail (which the human
#        block's exit closes). Lower confidence than the two above.
#
# What would falsify the chapter (docs/GOLDEN.md §11), read FIRST, and this
# file predicts that the FIRST ONE FIRES on `0/6` and `0/7`:
#
# - **A `GATHERORDER` or a `TRADEORDER` on any of the five in run141**, as
#   §11 states it: "the gate does not do what §11.4 reads it as doing". The
#   prediction above is that it fires because the orders come from above the
#   gate, not through it — which says §11's premise is wrong, not §11.4.
# - **No order on any of the five in run142**, the control: the units are
#   inert for a reason that has nothing to do with the gate, and the
#   comparison is vacuous. Predicted NOT to fire (the citizen and caravan).
# - **And the pair's own test, which §11 does not name:** if the five's
#   order kinds and first-order frames DIFFER between run141 and run142,
#   `ai off` reaches the human's civilians after all and the reading above
#   is wrong.
# ---------------------------------------------------------------------------

# `ai` is console-only (table index 12). DELETE THIS LINE for the control
# (`chapter7_control.cmd` is that file, and differs from this one only there).
0 !ai off

# The Medieval age, which is where the Supply Wagon and the upgraded
# Merchant live, and late enough that a Caravan has somewhere to go.
600 library who=0 2

# Five civilians, each a different `think_*` arm, on their owner's ground
# and within reach of his capital.
610 add citizen who=0 24,160
615 add caravan who=0 28,160
620 add merchant who=0 24,164
625 add scholar who=0 28,164
630 add fur who=0 24,176
