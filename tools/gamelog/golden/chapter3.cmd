# Golden record, chapter three — the mounted and siege lines.
#
# docs/GOLDEN.md §7. The premise: three chariots and one catapult against a
# hoplite squad. The chariots move at a mounted speed and the catapult has a
# MINIMUM range of three tiles, so the chapter separates two things chapter
# two cannot — a per-line speed, and a shot a unit refuses to take because
# the target is too close.
# Staged with
#   tools/explore/golden_capture.sh OUT --map 14 --end-frame 900 \
#       --cmd-file tools/gamelog/golden/chapter3.cmd \
#       --log-window 605 900 \
#       --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2 \
#       --detail misc:COMMANDMANAGER=1
# through `tools/gamelog/viadriver.sh`, which is what gives Wine a window.
#
# The channel's rules this file obeys (docs/ORACLE.md, "The cheat channel"):
# a `!` line reaches the console-only half of `run_cmd`'s two switches and a
# bare line only the chat half; lines run in file order and a frame below its
# predecessor is clamped; and `no_mouse = 1`, so an `add` without an `x,y`
# silently reuses a stale cursor — every placement here carries one.
#
# Coordinates are TILES (docs/INPUT.md §11.3): the same unowned arena as
# chapters one and two, world cell (1,10) and its neighbours.

# `ai` is console-only (table index 12). Every chapter's premise.
0 !ai off

# The Classical age, where the chariot and the catapult are first available.
# `age` alone, not `library` — chapter two carries the `library` arm, and one
# chapter changing one lever is what makes a parting attributable.
600 age who=0 4
602 age who=1 4

# The leading count is a count for a unit type: three chariots, not one
# squad of three (docs/INPUT.md §11.5). Chariots reach eight tiles.
610 add 3 chariot who=0 4,40
615 add hoplite who=1 12,40

# The catapult reaches 3-15 tiles. Eight tiles from the hoplites it can fire;
# if the hoplites close inside three it must stop, which is the thing here
# that no other chapter can show.
620 add catapult who=0 4,41
