# Golden record, chapter six — the air, and the one command that orders.
#
# docs/GOLDEN.md §10. Three of the six order classes no document cites are
# the air ones, and the channel reaches one of them directly: `bird`, table
# index 82, is the ONLY case in `ConsoleWin::run_cmd@007d6a70` that calls an
# `add_*_order` at all — it drops a gaia Wild Bird and hands it
# `Unit::add_air_patrol_order@005e4350`.
# Staged with
#   tools/explore/golden_capture.sh OUT --map 14 --end-frame 900 \
#       --cmd-file tools/gamelog/golden/chapter6.cmd \
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
# Coordinates are TILES (docs/INPUT.md §11.3): the unowned arena of chapters
# one to three, world cell (1,10) and its neighbours.

# `ai` is console-only (table index 12). Every chapter's premise.
0 !ai off

# The Modern age, which is where the Fighter and the Bomber are. `library`
# rather than `age`, because an aircraft's stats are epoch-fed and `age`
# leaves all four epochs Ancient (docs/RUNS.md run101-run105).
600 library who=0 6
602 library who=1 6

# Fighter reaches 2-7 tiles with sixteen rounds; Bomber 1-3 with ten. Eight
# tiles apart is outside both, so whatever closes the gap is the air line's
# own movement and not a shot taken on the frame of birth.
610 add fighter who=0 4,40
615 add bomber who=1 12,40

# `bird` LAST, and on its own frame, because it is the one line here whose
# placement the channel does not control: case 0x52 reads `mouse_coord_x/y`
# with no `x,y` argument and no `WorldData::restrict`, and with
# `no_mouse = 1` those are whatever `ConsoleWin`'s constructor left. If it
# lands somewhere impossible it will do so after every other record is
# already on disk.
700 bird
