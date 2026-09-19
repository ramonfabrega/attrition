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
# Staged with
#   tools/explore/golden_capture.sh OUT --map 14 --end-frame 1200 \
#       --cmd-file tools/gamelog/golden/chapter7.cmd \
#       --log-window 605 1200 \
#       --detail end:UNITS=3,GUYS=2,BUILDS=7,CITIES=5,GOODS=3,LEADERS=2 \
#       --detail misc:COMMANDMANAGER=1
# through `tools/gamelog/viadriver.sh`, which is what gives Wine a window.
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

# `ai` is console-only (table index 12). DELETE THIS LINE for the control.
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
