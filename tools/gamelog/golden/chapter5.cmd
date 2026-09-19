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
