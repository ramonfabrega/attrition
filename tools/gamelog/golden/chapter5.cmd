# Golden record, chapter five — the water.
#
# docs/GOLDEN.md §9. The Sail and Naval lines, which no capture on disk has
# ever carried: run16's coverage note lists `ObjectData::in_a_ship` and
# `num_aircraft_here` as never entered because "no ship, aircraft, or
# hero-general was in the game".
# Staged with
#   tools/explore/golden_capture.sh OUT --map 14 --end-frame 900 \
#       --cmd-file tools/gamelog/golden/chapter5.cmd \
#       --log-window 605 900 \
#       --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1
# through `tools/gamelog/viadriver.sh`, which is what gives Wine a window.
#
# The channel's rules this file obeys (docs/ORACLE.md, "The cheat channel"):
# a `!` line reaches the console-only half of `run_cmd`'s two switches and a
# bare line only the chat half; lines run in file order and a frame below its
# predecessor is clamped; and `no_mouse = 1`, so an `add` without an `x,y`
# silently reuses a stale cursor — every placement here carries one.
#
# Coordinates are TILES (docs/INPUT.md §11.3), and this chapter's are read
# off run105's own `[Start Game]` WORLD block. Map 14 is Great Lakes and has
# four of them: sea regions 66 (x 23-31, y 11-17), 67 (43-46, 13-23),
# 69 (38-48, 37-49, the largest at 69 cells) and 70 (13-20, 40-49). Region
# 70 is the one beside player 0's territory; its deepest cells — the ones
# with water on every side within two — are (15,45), (15,46) and (16,46),
# which are tiles x 60-67, y 180-187.
#
# `find_nearby_spot` is what decides where a spawn actually lands, and
# whether it refuses dry ground for a ship is the chapter's first question,
# not an assumption: see docs/GOLDEN.md §9's falsifier.
#
# The recipe is run112's tested command with the chapter file swapped, and
# the `--detail start:` line is not optional: without it the start block
# carries no leaders and no units, and `walk_chapter` refuses the capture
# (run112's first take). A relative `--cmd-file` is resolved against the
# repo root by `golden_capture.sh` since item 415.
#
# ---------------------------------------------------------------------------
# THE PREDICTION, written before run127 so the capture can fail (item 535).
# As chapter two's: a golden chapter takes no `captures.txt` stanza, so the
# stanza's `check:` lines live here. Every one is a grep or a count.
#
# check: six `INFO cmd` records, every one returning 1 — the four staged
#        here plus `37 !ffwd 1` and `900 !quit`, which the capture adds.
#        `python3 tools/gamelog/cmdsran.py <trace>`. A refused `add` may
#        still return 1 — `run_cmd` does not report the spot search — so
#        this check cannot stand in for the next one.
# check: `MAP_STYLE 14` and seed 12345 read back from the dump's GAME INFO.
# check: the window is the window asked for — `BEGIN FRAME` 1, then 605..899
#        with no gap, then 901, the `!quit` block: 297 frame blocks, as
#        run112 has.
# check: the spawns land on their frames. Trireme and Fishermen are both
#        UBER_SIZE 1, so one `UNITDATA` per `add`: 52 at 605, 53 from 611,
#        54 from 616, 55 from 621.
#
# What would falsify the chapter (docs/GOLDEN.md §9), read FIRST:
#
# - **No new `UNITDATA` at 611 or 616** — the `add` refused, and the spot
#   search's filter is what decides a hull's ground: the channel cannot
#   place a ship without a Dock. The unit count stays 52 across 611.
# - **A hull whose `x_internal/768, y_internal/768` is a land cell** of the
#   start block's `WORLD` — region 70 is x 13-20, y 40-49, and the asked
#   cells are (15,45), (16,46) and (15,46). The search does not filter by
#   domain.
# - **Two hulls in water and no `BEGIN AMMO` after 616** — a ship's fight is
#   not the land fight's shooting arm. The triremes stand 7.2 tiles apart
#   (dx 4, dy 6; §9's "about five" is the straight-line misreading), inside
#   RANGE 0-9, and AMMO_PER_ATT is 3, so the prediction is AMMO blocks from
#   the first reload after 616, three rounds a volley.
# ---------------------------------------------------------------------------

# `ai` is console-only (table index 12). Every chapter's premise.
0 !ai off

# Ancient: the Trireme and the Fishermen are both available with no tech at
# all, so this chapter changes no age lever and its parting cannot be an age
# field's.

# Two triremes, five tiles apart in open water. Trireme reaches nine tiles
# and carries three rounds, so they shoot from where they stand.
610 add trireme who=0 60,180
615 add trireme who=1 64,186

# A fishing boat, which fights nothing: the passive Sail record, and the
# control for anything the warships' own record shows.
620 add fisher who=0 61,184
