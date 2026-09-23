# Golden record, chapter four — the Temple, the border, and the bleed.
#
# docs/GOLDEN.md §8. The namesake. Three levers, each on its own frame so a
# WORLD dump either side attributes the border move to one of them:
# a Temple at 300, Religion at 400 (the first temple border level), Civic 3
# at 500. Then a hostile squad standing on player 0's own ground with
# Allegiance granted, a scout beside it as the exemption control, and a
# Supply Wagon arriving late to cancel the tick.
#
# TWO windows, because the two questions want different records
# (docs/DECISIONS.md 41 §2 — digest first, detail on demand). The recipe is
# run127's tested command with the chapter file swapped, one line a window:
#
#   run132, the border:
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch4b \
#       --map 14 --end-frame 1500 --log-window 295 545 --timeout 1500 \
#       --detail end:WORLD=6,BUILDS=7,CITIES=5,MISC=1 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter4.cmd
#
#   run133, the bleed: the same line with `~/ron-golden/ch4u`,
#       --log-window 595 1500 and --detail end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2
#
# **The border window is [295, 545), not §8's [295, 345)** (item 552): the
# narrow one sees the Temple and neither of the other two levers, so two of
# §8's five falsifiers could not have fired in it. `WORLD` prints its cells
# at detail 2 and `LeaderData::territory` only at `LEADERS=8`, so the cell
# count is the only cheap reading of a border and the window has to span all
# three levers. Both captures run to 1500 so their traces overlap whole.
#
# The `--detail start:` line is not optional: without it the start block
# carries no leaders and no units, and `walk_chapter` refuses the capture
# (run112's first take). A relative `--cmd-file` is resolved against the
# repo root by `golden_capture.sh` since item 415. `--timeout` bounds the
# game, and the runner's give-up is a truncation (docs/RUNS.md run107);
# read `receipt.json` before the dump. WORLD is ~3 MB a frame.
#
# The channel's rules this file obeys (docs/ORACLE.md, "The cheat channel"):
# a `!` line reaches the console-only half of `run_cmd`'s two switches and a
# bare line only the chat half; lines run in file order and a frame below its
# predecessor is clamped; and `no_mouse = 1`, so an `add` without an `x,y`
# silently reuses a stale cursor — every placement here carries one.
#
# Coordinates are TILES (docs/INPUT.md §11.3). This chapter is the one that
# needs real geography, and it is read off run105's own `[Start Game]` WORLD
# block rather than guessed: player 0's capital Napata stands at world
# (3168, 30816) — cell (4,40), tile (16,160) — and its territory is cells
# x 0-14, y 29-50. Tile 28,160 is cell (7,40), free ground beside the
# capital; tile 24,176 is cell (6,44), well inside the same border.

# ---------------------------------------------------------------------------
# THE PREDICTION, written before run132 and run133 so the captures can fail
# (item 552). As chapters two and five: a golden chapter takes no
# `captures.txt` stanza, so the stanza's `check:` lines live here. Every one
# is a grep or a count. A dump's block label is the sim frame plus one, so a
# line staged at 300 lands on block 301.
#
# check: ten `INFO cmd` records, every one returning 1 — the eight staged
#        here plus `37 !ffwd 1` and `1500 !quit`, which the capture adds
#        (`python3 tools/gamelog/cmdsran.py <trace>`), in each capture.
# check: `MAP_STYLE 14` and seed 12345 read back from each dump's GAME INFO.
# check: each window is the window asked for — run132 `BEGIN FRAME` 1, then
#        295..544 with no gap, then 1501: 252 blocks; run133 1, 595..1499,
#        1501: 907 blocks.
# check: **the two captures are one game.** Both traces cover all 1500
#        frames whatever the window, and their per-frame draw counts are
#        identical on every frame.
#
# The Temple (run132):
# check: a new `BUILDDATA` with `orig_type 437` and `who 0` on block 301, on
#        cell (7,40): `x_internal/768` 7, `y_internal/768` 40 after
#        `snap_center`. **Finished**, because `run_cmd` without `NEW` calls
#        the building's vslot 0x1a8 straight after `init_build`
#        (docs/ORACLE.md's table): `(int)construct_hits` equal to `myhits`.
# check: Napata's `city_flags` 18449 → 18577 on block 301 — bit 0x80, which
#        `Build::activate` sets on the Temple's city and
#        `compute_reg_territory` reads as "this city has a temple".
#
# The border (run132), counted as cells with `who 0` in each block's WORLD:
# check: 266 on block 300, as the start block has it (and 261 with `who 1`).
# check: **more than 266 by block 320** and flat again before 401; more
#        again by 420 after Religion (temple level 1 → 2: TEMPLE_UPGRADE_TERR
#        4 → 6, limit +4 → +8 tiles); more again by 520 after Civic 3
#        (CIVIC_UPGRADE_TERR 4, limit +12 tiles). The recompute is budgeted
#        at 256 cells a frame, so each step lands over several blocks, not
#        one; no cell goes from 0 to anything else, and the `who 1` count
#        stays 261 — the two capitals are far apart.
#
# The bleed (run133), per `UNITDATA`:
# check: three hoplite figures `who 1` born on block 601, a scout on 606, a
#        Supply Wagon on 1101: the unit count +3, +1, +1, less any DEATH.
# check: each hoplite figure reads `attrition 48` from its first refresh —
#        the first f ≥ 600 with (f + o) % 32 == 0 — while it stands on a
#        `who 0` cell: player 0 holds Allegiance (one step, strength 1),
#        player 1 no resistance, both Ancient. Its `damage_frac` and
#        `damage` then grow by 6/16 on each block f + 1 with
#        (f + o) % 48 == 0, up to 1100.
# check: the scout reads `attrition 0` on every block.
# check: from 1101, with the wagon within 14 tiles of the squad, no tick
#        lands on the 48-grid, and `unit_masks2` carries 0x40000 (262144)
#        on the frames a tick was due; the period stays 48. The wagon reads
#        `attrition 0` throughout (at war, eligibility check 13).
# caveat: the squad is at war beside player 0's capital, and run16 saw such
#        a squad walk off to fight within ~2,000 frames. Combat damage is
#        off the 48-grid, and a figure that leaves `who 0` ground reads 0 at
#        its next refresh; each is read as what it is, not as a falsifier.
#
# What would falsify the chapter (docs/GOLDEN.md §8), read FIRST:
#
# - **The Temple an unstarted site** — `construct_hits` 0, or below `myhits`
#   and not rising. Then the lever is `finish`, which the interpreter lacks.
# - **The `who 0` count unchanged across 300** — the Temple placed and the
#   border indifferent.
# - **Unchanged across 400 or 500** — the tech, or the civic level, is not a
#   border term.
# - **`attrition 0` on the squad after 600** while it stands on `who 0`
#   ground — overturns docs/ATTRITION.md's "one step is 48".
# - **A nonzero `attrition` on the scout** — overturns the `is_special`
#   exemption run16 observed.
# - **`damage` climbing on the 48-grid after 1101** with the wagon within 14
#   tiles — supply is not the counter; or **ticks stopping before 1101**
#   while the squad is still on `who 0` ground — something other than the
#   wagon is sheltering it.
# ---------------------------------------------------------------------------

# `ai` is console-only (table index 12). Every chapter's premise.
0 !ai off

# The building arm of `add`: a leading count would place ONE building, not
# num (docs/INPUT.md §11.5), so none is given. The Temple is 4x4 tiles, which
# is exactly one world cell.
300 add temple who=0 28,160

# The first temple border level. `Religion` is the tech the game's own
# rules.xml hangs "Temples increase city effect on National Borders" on.
400 tech who=0 religion on

# The civic term, the second independent border lever: TERRITORY_LIMIT_CIVIC
# is 4 tiles a level and CIVIC_UPGRADE_TERR cheapens distance with it.
500 civic who=0 3

# Attrition needs a strength: at war with no attrition tech the period is the
# sentinel 0 and nothing bleeds (docs/ATTRITION.md, run16). One step is 48.
550 tech who=0 allegiance on

# A hostile squad on player 0's ground, and a scout beside it. The scout is
# `is_special` and is exempt; it is the control that makes the squad's bleed
# a measurement rather than an observation.
600 add hoplite who=1 24,176
605 add scout who=1 27,176

# Supply cancels the tick. The wagon arrives five hundred frames in, so the
# same squad is its own before-and-after.
1100 add supply who=1 25,178
