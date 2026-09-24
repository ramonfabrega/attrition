# Golden record, chapter six — the air, and the one command that orders.
#
# docs/GOLDEN.md §10. Three of the six order classes no document cites are
# the air ones, and the channel reaches one of them directly: `bird`, table
# index 82, is the ONLY case in `ConsoleWin::run_cmd@007d6a70` that calls an
# `add_*_order` at all — it drops a Wild Bird for gaia's owner 9 and hands it
# `Unit::add_air_patrol_order@005e4350`.
#
#   run168 (item 648):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch6 \
#       --map 14 --end-frame 900 --log-window 605 900 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter6.cmd
#
# The channel's rules this file obeys (docs/ORACLE.md, "The cheat channel"):
# a `!` line reaches the console-only half of `run_cmd`'s two switches and a
# bare line only the chat half; lines run in file order and a frame below its
# predecessor is clamped; and `no_mouse = 1`, so an `add` without an `x,y`
# silently reuses a stale cursor — every placement here carries one.
#
# Coordinates are TILES (docs/INPUT.md §11.3): the unowned arena of chapters
# one to three, world cell (1,10) and its neighbours.
#
# ---------------------------------------------------------------------------
# THE STAGING, read before the run (docs/GOLDEN.md §3, point 5; item 648).
#
# 1. **`bird` is `init_unit(objects, 9, 0x192, cursor)`, then the air patrol
#    order on the same point.** The listing at 0x7df197..0x7df1dd pushes
#    `9` (the OWNER) and `0x192` (`TypeIndex::BIRD`, which the enum also
#    names `BASE_GAIATYPES`), so §10's "gaia type 9" had the two arguments
#    swapped. The point is `console_win +0x518/+0x51c` raw: no
#    `find_nearby_spot`, no `WorldData::restrict`. Then
#    `add_air_patrol_order(cursor, -1, -1, 0, ·)` on the new unit. That is
#    the same pair of calls `Objects::process_all`'s bird sampling makes on
#    a cell centre (crates/sim/src/gaia.rs, `spawn_bird`).
# 2. **Where the cursor is.** `mouse_coord_x/y` have three writers in the
#    export: `parse_cmd` when `no_mouse == 0`
#    (`TerrainOut::get_mouse_coords`, reached from the chat box and
#    `process_chat`); `CommandPackage::process_console_cmd`, which copies a
#    replicated command's own `mouse_x/y`; and nothing else. The tracer calls
#    `parse_cmd(console_win, line, from_chat, 1)` (tools/trace/tracer.c), and
#    no chat or replicated console command runs in this lobby. The object is
#    `malloc(0x558)` in `System::init@00599700`, and neither
#    `ConsoleWin::ConsoleWin` nor `ConsoleWin::init` writes +0x518. **So the
#    cursor is whatever the heap left there.** Predicted: **zero**, a heap
#    block not reused since its pages were committed. The bird is then seated
#    by `Unit::init`'s snap at **(24, 24)**, the world's corner, with its
#    patrol point at (0, 0), cell (0, 0). Lower confidence: a non-zero
#    remnant. A large one indexes `div_3_table[p >> 4]` past its end in
#    `Unit::init`, and the game may die on 700 with every earlier block
#    already on disk (the reason `bird` is last).
#    MEASURED (item 652, run169's packet at logger frame 701): (0, 6). The
#    seat is (24, 24) as predicted; the patrol point is six units off it.
# 3. **No dump prints owner 9** (gaia.rs, `BIRD_OWNER`; run127's dump has
#    UNITDATA for owners 0, 1 and 8 and none for 9, with birds alive).
#    So §10's first falsifier as written, "no AIRPATROLORDER block after 700",
#    fires by construction and cannot tell a bird from no bird. **Its
#    reachable form is the draw stream** (falsifier 1 below).
# 4. **`add` accepts an aircraft anywhere.** Case 0x4d/0x4e is
#    `UnitType::find_nearby_spot` then `init_unit`. Neither tests `WHERE`
#    (the Fighter's and the Bomber's is `Airbase`) or an age. And
#    `find_nearby_spot@0061de70:143` skips the terrain-block test for domain
#    2, so only a collision can push the spot. `fighter` resolves to TYPENAME
#    `Fighter` (#239, Modern) and `bomber` to `Bomber` (#254, Modern), exact
#    matches both. Predicted seats, as chapter one's captain's:
#    Fighter **(888, 7800)**, Bomber **(2424, 7800)**.
# 5. **An aircraft outside a container burns fuel and nothing else stops
#    it.** `Unit::process@00610bc0`: domain 2 with `inside_up < 0` raises
#    `mana_burn` by one a frame up to `mana()` (MANA 400 Fighter, 600
#    Bomber). `check_fuel` and `land_plane` are reached only from the air
#    orders (`do_air_patrol`, `do_air_attack_ground`, `do_strafe`). Neither
#    aircraft gets an order from `Unit::init`. So with no order the fuel
#    counter climbs and never bites inside this window (born 611 and 616;
#    400 frames out is 1011).
# 6. **`!ai off` does not stop who=1's Bomber** (docs/INPUT.md §11.10:
#    `Unit::init:585` marks every who=1 unit `0x40000`). With no order it is
#    `do_idle → think`, as the Fighter is. But `think_attack@005f5a80` forks on
#    that mask. For the human's Fighter (`0x40000` clear) its search runs
#    with the army and the city arms off. For the Bomber it may `add_to_army`
#    and, when the search finds nothing, `go_to_city`.
#    **What that means for "the air line's own movement"**: every order on
#    either aircraft is still the original's own automatic play and none is
#    the channel's, which is all §10's line needs. It no longer says which
#    arm moved the Bomber, so the reading of a move is its order stack's: an
#    `ATTACKORDER`/`STRAFEORDER` is the engagement, and a `MOVEORDER`, or a
#    `group` on the Bomber with no target, is the AI's unit-level arm. The
#    lines are not restaged: the Fighter's side is the clean test, and it is
#    8 tiles (1536) from the Bomber, inside its LOS 16 and one tile outside
#    its reach of 7.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS (docs/GOLDEN.md §10), and where each could first fire.
#
# 1. **The bird: no birth on 700.** First fires on the trace's frame 700.
#    Predicted: one `Guy::init_real` draw under `Unit::init <
#    Objects::init_unit < ConsoleWin::run_cmd`, before phase 1, and
#    `INFO cmd` for `bird` returning 1. Then from 704 (`frame & 7 == 0`) the
#    bird's own `Animal::think_bird+0x82`/`+0xa6` pair every eighth frame,
#    and 704's sampling (`frame & 0x1f == 0`) one `Objects::process_all`
#    pair short of what it would be without the bird. If the cursor is the
#    corner, `Unit::do_air_physics+0x639` edge coins come within its first
#    frames. None of this is in the dump: owner 9 is never printed.
# 2. **An aircraft whose position never changes: an air unit staged
#    outside an airbase is inert.** First fires on the Fighter's UNITDATA
#    (born block 611, who=0) and the Bomber's (616, who=1): `x_internal`/
#    `y_internal` unchanged through 899. It is an absence, so it can only
#    close on the last block. Predicted NOT to fire: the Fighter's search
#    reaches the Bomber, and the Bomber's arms move it either way.
#    `mana_burn` climbs one a frame on both from birth (point 5).
# 3. **`add` refuses the Fighter: the air line needs a base.** First fires
#    on block 611, no new who=0 UNITDATA of the Fighter's type; and 616 for
#    the Bomber. Predicted NOT to fire (point 4).
#
# Lower confidence, and a first for any capture: `library N 6` passes the
# Industrial age, whose `gain_tech` tail carries the oil grant, which no
# capture has reached (docs/INPUT.md §11.11). It would show on 605's
# LEADERDATA goods for both players.
#
# check: every staged line runs. `cmdsran.py` shows eight `INFO cmd`
#        returning 1 (the six here, `37 !ffwd`, `900 !quit`).
# check: `MAP_STYLE 14`, seed 12345, blocks 1 and 605..899 at least.
# check: the Fighter and the Bomber are identified by type and birth block,
#        never by slot.
# ---------------------------------------------------------------------------

# `ai` is console-only (table index 12). Every chapter's premise.
0 !ai off

# The Modern age, which is where the Fighter and the Bomber are. `library`
# rather than `age`, because an aircraft's stats are epoch-fed and `age`
# leaves all four epochs Ancient (docs/RUNS.md run101-run105). Six age techs
# held, Classical to Modern (`tech::LEVELS` is seven).
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
# `no_mouse = 1` those are whatever the heap left in `console_win`. If it
# lands somewhere impossible it will do so after every other record is
# already on disk.
700 bird
