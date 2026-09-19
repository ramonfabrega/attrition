# Golden record, chapter eight — the commanders, and a war that is declared.
#
# docs/GOLDEN.md §12. Chapter one's `604 war` is the BARE form: cases 0x2c-
# 0x2e print the diplomacy table and change nothing, and its squads engage
# because a Quick Battle already starts at war (docs/INPUT.md §11.6). This
# chapter is the one that actually moves the diplomacy state, three times,
# each with a fight running across the change: peace, then war, then
# alliance. And it carries the Command line — a General, whose aura is
# passive, and a Spy, whose abilities are the game's own spells.
# Staged with
#   tools/explore/golden_capture.sh OUT --map 14 --end-frame 1200 \
#       --cmd-file tools/gamelog/golden/chapter8.cmd \
#       --log-window 605 1200 \
#       --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1
# through `tools/gamelog/viadriver.sh`, which is what gives Wine a window.
# LEADERS=5 rather than 2: the diplomacy row is what this chapter reads.
#
# The channel's rules this file obeys (docs/ORACLE.md, "The cheat channel"):
# a `!` line reaches the console-only half of `run_cmd`'s two switches and a
# bare line only the chat half; lines run in file order and a frame below its
# predecessor is clamped; and `no_mouse = 1`, so an `add` without an `x,y`
# silently reuses a stale cursor — every placement here carries one.
#
# Coordinates are TILES (docs/INPUT.md §11.3): the unowned arena of chapters
# one to three, world cell (1,10) and its neighbours.

# `ai` is console-only (table index 12). Every chapter's premise.
0 !ai off

# The Medieval age: the General and the Spy both want Mathematics, and
# `library` carries the whole shelf rather than one tech.
600 library who=0 2
602 library who=1 2

# A squad each, so there is a fight for the diplomacy to interrupt, and a
# commander each: the General's bonus is an aura over the squad beside it,
# the Spy's is a spell it can be made to cast.
610 add hoplite who=0 4,40
612 add general who=0 5,40
615 add hoplite who=1 12,40
617 add spy who=1 12,41

# The three diplomacy levels, each with a target token, which is what makes
# them the non-bare form. `parse_who`'s default here is `console->who`, so a
# bare `1` is a player and not a level (docs/INPUT.md §11.2).
700 peace 1
800 war 1
900 ally 1
