# Golden record, chapter one — two armies, a late age, and no Leader AI.
#
# DECISIONS 41 §1: the rules track runs on a staged game with the AI silenced,
# in chapters, driven from this file for the spawns and the cheats. Staged with
#   tools/explore/golden_capture.sh OUT --map 14 --end-frame 900 \
#       --cmd-file tools/gamelog/golden/chapter1.cmd \
#       --log-window LO HI --detail end:...
# through `tools/gamelog/viadriver.sh`, which is what gives Wine a window.
#
# The channel's rules this file obeys (docs/ORACLE.md, "The cheat channel"):
# a `!` line reaches the console-only half of `run_cmd`'s two switches and a
# bare line only the chat half; lines run in file order and a frame below its
# predecessor is clamped; and `no_mouse = 1`, so an `add` without an `x,y`
# silently reuses a stale cursor — every placement here carries one.

# `ai` is console-only (table index 12). This is the chapter's premise.
0 !ai off

# A late age for both players, so the dump carries the fields only a late age
# writes. `cheat age 3` would read 3 as the age, hence `who=`.
600 age who=0 8
602 age who=1 8

# War before the spawns, so the squads are hostile from birth and nothing has
# to be ordered — and no console command this chapter uses issues an order.
# (Exactly one in the whole vocabulary does: `bird`, table case 82, calls
# `Unit::add_air_patrol_order@005e4350`. docs/GOLDEN.md §13.)
604 war

# `add hoplite` places three squads, not one; the leading count is not a count.
# On map 14 at seed 12345 these land ~2.5 tiles apart. NOT near player 0's
# capital, which the first draft of this line said: `4,40` is read by the
# TILE arm of `parse_coord`, so it is world cell (1,10) — unowned BASELAND,
# thirty cells from Napata at cell (4,40). docs/GOLDEN.md §4 and §5.
610 add hoplite who=0 4,40
615 add hoplite who=1 5,40
