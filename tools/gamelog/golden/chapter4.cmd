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
# (docs/DECISIONS.md 41 §2 — digest first, detail on demand):
#   the border    --log-window 295 345 \
#                 --detail end:WORLD=6,BUILDS=7,CITIES=5,MISC=1
#   the bleed     --log-window 595 1500 \
#                 --detail end:UNITS=3,GUYS=2,DEATHS=1,LEADERS=2
# and BOTH also want the `[Start Game]` set, which is not optional:
#   --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1
# Without it the dump's start block holds no leaders and no units, the
# harness stands up nothing, and the walk reports a word that is the
# setup's rather than the simulation's — run112's first attempt
# (docs/RUNS.md run112). `chapter_two_holds_to_the_golden_word`'s guard
# refuses such a capture by name now.
# both with `--map 14 --end-frame 1500 --cmd-file <this file>` through
# `tools/gamelog/viadriver.sh`. WORLD is ~3600 cells a frame; do not window
# it wide.
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
