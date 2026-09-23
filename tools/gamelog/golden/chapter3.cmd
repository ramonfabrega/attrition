# Golden record, chapter three — the mounted and siege lines.
#
# docs/GOLDEN.md §7. The premise: three chariots and one catapult against a
# hoplite squad. The chariots move at a mounted speed and the catapult has a
# MINIMUM range of three tiles, so the chapter separates two things chapter
# two cannot — a per-line speed, and a shot a unit refuses to take because
# the target is too close.
#
# run145 (item 587). The recipe is run141's tested command with the chapter
# swapped and chapter two's dump set, one line:
#
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch3 \
#       --map 14 --end-frame 900 --log-window 605 900 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter3.cmd
#
# The `--detail start:` line is not optional: without it the start block
# carries no leaders and no units, and `walk_chapter` refuses the capture
# (run112's first take). A relative `--cmd-file` is resolved against the
# repo root by `golden_capture.sh` since item 415. `--timeout` bounds the
# game, and the runner's give-up is a truncation (docs/RUNS.md run107):
# read `receipt.json` before the dump.
#
# The channel's rules this file obeys (docs/ORACLE.md, "The cheat channel"):
# a `!` line reaches the console-only half of `run_cmd`'s two switches and a
# bare line only the chat half; lines run in file order and a frame below its
# predecessor is clamped; and `no_mouse = 1`, so an `add` without an `x,y`
# silently reuses a stale cursor — every placement here carries one.
#
# Coordinates are TILES (docs/INPUT.md §11.3): the same unowned arena as
# chapters one and two, world cell (1,10) and its neighbours.
#
# The names resolve by `ConsoleWin::parse_type`'s order, unit table first,
# each an exact match: `chariot` → Chariot (record 145, TypeIndex 195),
# `catapult` → Catapult (215, 265), `hoplite` → Hoplites (82, 132) as in
# chapters one and two. The Chariot is a nation's unique unit (TRIBE_MASK
# bit 7 only) and who=0 is Nubia, but `run_cmd`'s `add` arm calls no
# `get_graft` and neither does `Objects::init_unit@0065e0c0`, so the type
# placed is the type named (read 2026-09-23, item 587). Both are UBER_SIZE
# 1; the Hoplites are 3.
#
# **What the disk already had** (item 587, `rg` of every dump under the
# `Logs` archive and `~/ron-golden` for a `GUY` of type 195 or 265): no
# Chariot anywhere, and Catapults only in run44 — Islands, 700 frames,
# `GUYS=4` and no `AMMO`, the AI's two ordered unaided at 247 and 248. No
# capture has ever dumped a catapult's shot.
#
# ---------------------------------------------------------------------------
# THE PREDICTION, written before run145 so the capture can fail (item 587).
# A golden chapter takes no `captures.txt` stanza, so the stanza's `check:`
# lines live here. Every one is a grep or a count. A dump's block label is
# the sim frame plus one, so a line staged at 610 lands on block 611.
#
# check: every staged line runs — `python3 tools/gamelog/cmdsran.py <trace>`
#        shows eight `INFO cmd` records each returning 1: the six here plus
#        `37 !ffwd N` and `900 !quit`, which the capture adds.
# check: `MAP_STYLE 14` and seed 12345 read back from the dump's GAME INFO.
# check: the window is the window asked for — `BEGIN FRAME` 1, then
#        605..899 with no gap, then 901: 297 blocks, as run112's.
# check: `age who=N 2` lands `ages_get() 1` in both players' LEADERDATA
#        from 605 — the Classical age, `age`'s level counting from one
#        (run105: `age 8` → `ages_get() 7`) — with `epochs_get() 0`.
# check: births — three separate player-0 units `0/6`, `0/7`, `0/8` on
#        block 611, each `up -1 down -1` (not a squad), each one `GUY type
#        195`; the hoplite squad `1/6..1/8` on 616, `GUY type 132`, threaded
#        6 → 7 → 8; the catapult `0/9` on 621, `GUY type 265`.
# check: `myspeed` reads 30 on the chariots, 25 on the hoplites and 19 on
#        the catapult — MOVES in unitrules.xml, read through at the
#        Classical age with no military epoch.
# check: **the catapult fires.** An `AMMO` record with `who 0 o 9` and
#        `cur_time 1`, its target one of `1/6..1/8`, while that target
#        stands between 3 and 15 tiles of it; by chapter two's analogy
#        (the slingers, born on 621 and ordered on 621) its first
#        `ATTACKORDER` is on or near its birth block. Lower confidence on
#        the frame than on the fact.
# check: **the chariots fire**: `AMMO` with `who 0 o 6..8` (range 0–8,
#        AMMO_PER_ATT 1), and the hoplites close on player 0 — by chapter
#        two's analogy the first chariot and hoplite orders near 635.
#
# What would falsify the chapter (docs/GOLDEN.md §7), read FIRST, and this
# file predicts that NONE of the three fires:
#
# - **The catapult firing at a target inside three tiles.** An `AMMO` with
#   `who 0 o 9 cur_time 1` whose target's centre is under 3 × 192 − 6 =
#   570 units from the catapult's on the launch block, less the two units'
#   big radii (`ObjectData::is_in_range`'s rescue, docs/COMBAT.md §13.2):
#   a launch under 2.5 tiles (480 units) fires it outright, and one in the
#   band 480..570 is read against `attack_dist` before it is called either
#   way. If the hoplites never close inside three tiles of the catapult,
#   this falsifier CANNOT fire, and the report says so rather than
#   calling it held.
# - **Three chariots whose per-frame displacement equals the hoplites'.**
#   Measured on blocks where both walk: the largest one-block step of a
#   chariot against a hoplite's. Predicted ~30:25. If the chariots never
#   walk, this one cannot fire either, and is reported as untested.
# - **A leading count of 3 producing nine units or one** — or none:
#   `add 3 chariot` should be exactly three type-195 units on block 611.
# ---------------------------------------------------------------------------

# `ai` is console-only (table index 12). Every chapter's premise.
0 !ai off

# The Classical age, where the chariot and the catapult are first available.
# `age` alone, not `library` — chapter two carries the `library` arm, and one
# chapter changing one lever is what makes a parting attributable. `age`'s
# level counts from one, so Classical is 2 (item 587: this file said 4, which
# is Gunpowder).
600 age who=0 2
602 age who=1 2

# The leading count is a count for a unit type: three chariots, not one
# squad of three (docs/INPUT.md §11.5). Chariots reach eight tiles.
610 add 3 chariot who=0 4,40
615 add hoplite who=1 12,40

# The catapult reaches 3-15 tiles. Eight tiles from the hoplites it can fire;
# if the hoplites close inside three it must stop, which is the thing here
# that no other chapter can show.
620 add catapult who=0 4,41
