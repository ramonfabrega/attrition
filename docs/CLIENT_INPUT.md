# Client input: a person's events, as the script's orders

*Item 4145, 2026-11-11; §5 item 4149, the same day; §6 item 4179, 2026-11-12; §7 item 4180, the same day; §8 item 4195, the same day; §9 item 4194, the same day; §10 item 4202, 2026-11-13; §11 item 4249, 2026-11-14; §12 item 4250, the same day; §13 item 4252, the same day; §14 item 4254, the same day; §15 item 4256, the same day; §16 item 4259, the same day; §17 item 4261, the same day; §18 item 4264, 2026-11-15; §19 item 4265, the same day; §20 item 4267, the same day; §21 item 4268, the same day; §22 item 4270, the same day; §23 item 4273, the same day; §24 item 4276, the same day; §25 item 4277, the same day; §26 item 4280, the same day; §27 item 4282, the same day. Not `docs/INPUT.md`, which is the recorded order stream into the harness. The client's `input` module (`crates/client/src/input.rs`); the oracle is the minted script
(`docs/RECGAME.md` §9), whose file the original played for 24,000 frames (run1561).*

An order minted from input events that equals the script's order, field for field, is an order the original has
already played. That is the whole method here: the script is the oracle, and the input layer is counted against it.

## 1. What an event becomes

The layer takes events, the session's state and a view, and mints orders in the script's own vocabulary,
`recgame::Order`: a recording frame, the issuing slot, then a `group` and the one command behind it.

- **The events** are a press at a pixel, a drag from one pixel to another, a right press at a pixel, and a key.
- **The session's state** (`input::Input`) is the issuing slot, the selection, whether the selection is a building,
  and the building chosen to place.
- **The view** (`input::View`) is the camera's matrix and screen, the size the draw takes, the frame's `Scene` and the
  ground's `TerrainScene`.

Each event does one thing:

- **A press** selects the issuing slot's drawn object whose anchor is nearest the pixel, within `PICK_RADIUS` (24 px).
  The anchor is the object's point at its own `z`, through `render::camera::world_to_screen`. A press on nothing clears
  the selection. If a building has been chosen, the press places it instead: a `build` at the ground under the pixel,
  with `x2`/`y2` equal to `x`/`y`, and `queued` 2.
- **A drag** selects the slot's units in the box, in the order the original lists them, filtered by its civilian
  rule (§7).
- **A right press**, with units selected, on an object gives the command the original's dispatch issues for it
  (§6). On nothing it moves them: a `move_to` at the ground under the pixel. Its fields are the
  script's right-click fields: `set_angle` 0, `angle` 0, `orders` 1, `queued` 2, `disembark` 0, and `form`/`width`
  the formation the original reads off the selection (§7.1), for one unit as for many (§9).
- **A right press on the minimap** (`Event::MapRightPress`) moves the selection to the point it names, with
  `form`/`width` −1/−1, the minimap's own fields (§9). Ours: the minimap is not drawn, so the event carries the
  world point rather than a pixel.
- **A key** trains or chooses:
  - With a building selected, a key in `TRAIN_KEYS` trains: `c` gives a `queue_up` of the Citizen (50), `num` 1.
  - With units selected, a key in `BUILD_KEYS` chooses the building the next press places: `f` chooses the Farm (417).

**The orders reach the walk through the script's door.** `FreeRunner::issue` hands the order to the turn pump. It
goes through `Stream::push`, which encodes and decodes the commands as a recording stores them, and then
`Stream::pump`. They reach a file through `recgame::minted_packages`. Nothing new is minted beside it.

**The ground under a pixel** (`input::ground_at`) is the camera's inverse onto the heightfield, taken to integers:

1. The camera's two equations are solved for `(x, y)` at a height, in `f64`.
2. The height is re-read at the solution until it holds. It is the tile's height (`TerrainScene::tile_z`, 192 units a
   tile, never below 0), which is `World::object_z`'s rule but for its ocean arm.
3. The solution is rounded with `f64::round`.
4. The rounded point is kept if the forward arithmetic takes it back to the same pixel. Otherwise the nearest integer
   point that does is taken. If no integer point does, the rounded point stands; that happens when a pixel is finer
   than the integer grid.

So no float reaches an order's bytes except through that rounding.

**The window** feeds the same layer. `main.rs` turns the cursor's pixel and a left or right mouse press into a press or
a right press, and an unbound character key into a key. A left press becomes a press or a drag when it is released
(§7.1). Each event runs under the camera and window size the frame is
drawn with. The HUD is asked first (`docs/HUD.md` §2, item 4197): a press on one of its buttons is that button's key.
A minted order is issued for the frame about to be stepped, and printed. `--lobby` opens the script's
lobby, built from its choices by `FreeRunner::from_lobby` (`docs/START.md` §7), in place of the pair's game.

## 2. The values (stage 0)

The script is walked on the built start, and each order's object and target are read on the order's own frame. The
height rule holds there: all 52 units of the scene stand at `ground_z` on frames 30, 60 and 90. Every pixel below
comes from `world_to_screen` with `graphsys`'s size, and all of them are on the 1920×1080 screen.

**The camera the client opens with** is `pinned_camera()`:

| frame | object | point, `z` | pixel | target | ground `z` | pixel | the inverse | footprint |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 30 | Scout `0/0` | (3480, 31896), 545 | (1006, 629) | (5880, 31896) | 313 | (1384, 862) | (5883, 31896) | 39 points, 9 × 9 |
| 60 | Village `0/2000` | (3168, 30816), 536 | (1127, 521) | — | | | | |
| 90 | Citizen `0/1` | (4008, 28296), 511 | (1655, 394) | (3552, 29856) | 526 | (1338, 478) | (3554, 29860) | 46 points, 10 × 10 |

At this camera, one pixel covers 39 to 46 integer points of ground. So a click names the script's coordinate's pixel,
and the inverse comes back one rounding away: (+3, 0) and (+2, +4). The item's kill does not hold: each target is
inside its own pixel's footprint. The nearest other anchor of slot 0 is 157, 173 and 46 px away (the Woodcutter
`0/2001` beside the Citizen), so each press selects its own object.

**The closer camera** is the same camera looked at the target at its ground height (`loc`, `loc_z_adjust`) and zoomed
×8 (`parallel_distance / 8`). There the target is at (960, 540) and (960, 541). The footprint is one point, and the
inverse returns the script's coordinate exactly. At this zoom many pixels hold no integer point of their own, and the
rounded point stands (`minted_a_fine_pixel_still_names_the_ground`).

Pinned by `minted_stage_zero_pixels_and_round_trips` (`OBJECTS`, `TARGETS`). Seen red: the Scout's pixel pinned at
(1006, 628).

## 3. The count

`INPUT_MINTED`: **3 of 3**. It was 0, when there was no input layer.

Seven synthetic events go through the layer, each at a stage 0 pixel on its order's frame:

1. Frame 30: a press on the Scout, then a right press at its target under the closer camera.
2. Frame 60: a press on the Village, then `c`.
3. Frame 90: a press on the Citizen, then `f`, then a press at the Farm's site under the closer camera.

They mint three orders, and every field equals the script's. The file written from them is the file the original
played, byte for byte: `~/ron-golden/recordings/item4059-minted-greatlakes-24000.rcx`, 59,280 bytes. The walk that
took the orders stands where the script's effects put it on frame 299 (`docs/RECGAME.md` §9.3): the Scout on
(5880, 31896), and the Farm's site `0/2007` at (3456, 29760). The test is `minted_from_events_the_script_s_orders`.
Seen red: the opening camera's orders, written in the file's place, part from its bytes.

`INPUT_MINTED_OPENING`: **1 of 3**. These are the same events with the targets clicked under the opening camera. The
`queue_up` comes out whole. The move parts by (+3, 0) on frame 30, and the build by (+2, +4) on frame 90. The Farm's
site still snaps to (3456, 29760). The test is `minted_from_events_under_the_opening_camera`.

## 4. Confidence, and what is not established

**Backed by the clicked file's playback** (§5). The clicks at the opening camera mint a file the original plays in
lockstep for 24,000 frames. Over its first 300 frames the records agree whole, the clicked move's three units among
them.

**Backed by the script's count.** On these three orders, the press, right press and key arms mint the bytes of a file
the original played in lockstep. The stage 0 projections are the original's own forward arithmetic
(`docs/RENDERER.md` §3.4).

**Ours, not the original's.** These rest on no reading:

- the pick: the nearest anchor within 24 px;
- the keys `c` and `f`;
- the inverse's rounding;
- ~~that a right press always moves~~ a right press on an object mints what `Console::execute_at_cursor` issues
  (§6); on the ground it moves, and the move's fields are the script's;
- the right press's look (§6): the nearest anchor within 24 px, where the original searches a ±5 px box by bounding
  volume;
- the box's anchor test and its drag threshold (§7.1), ~~and one unit's (−1, −1)~~ the main view's formation for
  one unit (§9);
- the minimap's event carrying a world point (§9).

**Not established:**

- **How the original turns a click into a coordinate.** Its inverse and its rounding were not read. At the opening
  camera, no rule could reach an arbitrary script coordinate, because one pixel covers 39 to 46 points. The clicked
  file (§5) is the layer's coordinate played by the original, not the original's own click.
- **The clicked build's coordinate on the original's side.** The dump prints the snapped site, not the drop point, so
  the build's (+2, +4) is compared by no field (parked 4149b).
- **The clicked file past frame 300.** The long take's draws and closing record cannot tell it from the first
  script's file (§5).
- **The original's hit test**, which uses a piece's drawn pixels.
- **The interface**: hotkeys, the build menu and the cursor.
- **Ground height**: the ocean arm of `object_z`, and any point off the map.
- **Everything the item ruled out**: ~~a right press on an object (gather, attack, garrison)~~ the gather and the help
  to build are played (§6), and the attack, repair and garrison are §6's "not established"; ~~drag and box selection~~ a box of three Citizens is played (§7),
  control groups, queued (shift) orders, ~~more than one object selected~~ a box of more than one type (§7.4), a
  second slot, and every command beyond the ~~script's three~~ five §6 plays and §7's group move.
- **The window under a real mouse.** It is checked by eye only; no capture ran.
- **The window size.** The window draws with its own size, where these tests use `graphsys`'s.
- **The zoom.** The closer camera's ×8 is beyond the original's nearest fixed zoom: `fixed_zooms[6]` is about 1.5×
  closer than the default.

## 5. The clicked script and its playback (item 4149)

§2's table cannot reach the script at the opening camera. So this turns the oracle round: the orders a person can
click there become a second script, and the original's own playback of their file is the oracle.

**The clicks** are §3's seven events, each target clicked under the opening camera:

| frame | event | pixel | the ground the layer mints | why this pixel |
| --- | --- | --- | --- | --- |
| 30 | press, then right press | (1006, 629), then (1384, 862) | `move_to` (5883, 31896) | the script's target's own pixel; its footprint holds the script's point |
| 60 | press, then `c` | (1127, 521) | — (`queue_up` 50) | the Village's anchor |
| 90 | press, `f`, then press | (1655, 394), then (1338, 478) | `build` (3554, 29860) ×2, 417 | the script's drop point's own pixel |

Our walk accepts each order there. The Scout's destination is on open grass, and our walk's Scout stands on it by
frame 299. The Farm's drop point is `Clear` on `blocked_site`, and `snap_center` puts its site `0/2007` at
(3456, 29760), where the first script's stands.

**The second script** is `rondata::script::clicked()`, the first script with `CLICKED_SCOUT_TO` (5883, 31896) and
`CLICKED_FARM_AT` (3554, 29860). It stands beside `script()` and does not replace it. Its constants are typed in
`rondata`, which cannot call the client; the client's `minted_clicked_at_the_opening_camera` mints the orders from the
clicks and holds them equal to `clicked()` field for field.

`INPUT_CLICKED`: **3 of 3**. It was 0, when there was no clicked script. Seen red: `CLICKED_FARM_AT` set to the first
script's (3552, 29856) gave 2 of 3.

**The file** is `~/ron-golden/recordings/item4149-clicked-greatlakes-24000.rcx`, 59,280 bytes, SHA-256
`9636d81e…ca14`. It is the first script's file with the move's and the build's packages changed and nothing else
(`the_clicked_minted_packages_round_trip_and_stand_on_our_placement_check`, `CLICKED_RCX_BYTES`). The client writes
the same bytes from the orders it minted.

**The playback** (`docs/RUNS.md` run1563a, run1563): the click-free lane, `docs/RECGAME.md` §9.3's line with this file.
`the_original_s_playback_of_the_clicked_minted_file_walks_in_lockstep` walks the built start under the pump against
each take (`CLICKED_WALKED`):

- run1563a, `end:MISC,UNITS=3,BUILDS=7`, 300 frames: draws agree on **300 of 300**, count and sequence; **299 records
  compared whole, none parted**. Seen red first: the row pinned `(0, 0, 0, 0)` printed `(300, 300, 299, 0)`.
- run1563, `end:MISC`, the game's whole length: draws agree on **24,000 of 24,000**, count and sequence; one record,
  none parted. Seen red first: the row pinned `(0, 0, 0, 0)` printed `(24000, 24000, 1, 0)`. run1563b, the 2,400-frame
  trace-only take, was not run: the whole length covers its frames.

**`INPUT_PLAYED`**: **24,000 of 24,000** frames in lockstep, 299 records whole, and each order's effect equal on
both sides. It was 0, when no clicked file had been played.

**What the long take cannot see.** The original's word is identical between the two playbacks, the first script's
(run1561) and the clicked one (run1563), on all 24,001 frames (`rngcmp.py`). Our walk under the *first* script's file
also agrees with run1563 on all 24,000 frames' draws and on its closing record. So the 24,000 frames are lockstep, but
they do not tell the clicked file from the first one. The click is seen by run1563a's per-frame records, over 300
frames, and nowhere past them.

**Each order's effect, both sides** (`clicked_effects`, on 31, 61, 91 and 299), equal:

- the Scout at (3514, 31896), (4534, 31896) and (5554, 31896), and on 299 on the clicked (5883, 31896);
- the Village's live queue `[50]` on 61 and 91, and empty by 299;
- the site `0/2007`, type 417, at (3456, 29760) from 91;
- the builder's `move`, `build` on 91, and `gather` on 299.

**The value diff** against the first script's playback (run1561a), on the original's two dumps of 300 frames: only
the Scout's `x` fields part. Its move order holds `x` 5880 on both. Its `orig_x` and its path's `to_x` hold the
clicked 5883 in run1563a where run1561a holds 5880, and its `dest_x`, `orders_x` and `x_internal` part on the frames
after. On 299 it
stands on (5883, 31896) against (5880, 31896). Our walk carries the same split: its move order's target reads
(5880, 31896) on 31, 61 and 91 for the click at 5883 as for the script's 5880, and it stands the Scout on the clicked
point (parked 4149a).

**The build's (+2, +4) is printed nowhere.** Neither the script's drop point nor the clicked one appears in either
dump. The site both snap to, (3456, 29760), is the same for both scripts. So the build's clicked coordinate is equal on
both sides only through the file's bytes and the snapped site; no field compares it.

**Made to fail** (`the_first_minted_script_against_the_clicked_playback_parts`, `FIRST_AGAINST_CLICKED`): our walk
under the first script's file, against run1563a, parts first on frame 31. The parting is the Scout's path target,
ours (5880, 31896) against the original's (5883, 31896), and 269 of 299 records part. The draws do not part inside
300 frames. So the comparison sees three units of a click. Against run1563 the same walk parts nowhere, as above. Seen
red first: pinned `(0, "", "", None)`, and the long take pinned to part.

## 6. A right press on an object (item 4179)

A right press on a thing mints the command the original's own dispatch issues for it. The dispatch is
`Console::execute_at_cursor@007c6630`. `Console::on_right_up@007cd3c0` calls `Console::right_click@007c4e70`, which
calls it with `(0, 0)`: execute, no pick mode. `Console::on_mouse_move@007cef20` calls it with `(1, 0)`, which only
sets the cursor. The arms were read off the decompile and the listing by one reader and checked against the
decompile's call sites (lines 885–897, 931–945 and 1225–1240).

### 6.1 What the dispatch issues

The case read is one selected Citizen of the human's, no modifier held, and the default profile's prefs
(`PlayerProfile::default_mouse@005d01f0` clears `0x20`, `0x200`, `0x400` and `0x800`). Every arm's queue is `QUEUE_NEW`
(2). With shift it is `QUEUE_LAST`, and with shift and alt `QUEUE_FIRST`.

| under the cursor | the command | its fields |
| --- | --- | --- |
| one's own unfinished building | `swarm_around` (`GroupOut::issue_swarm_around@0070afc0`) | `ox`, `whom`, `QUEUE_NEW`, `BUILD_AT` (6) |
| one's own finished gather building with room | `gather` (`GroupOut::issue_gather@0070aa20`) | `ox`, `QUEUE_NEW` |
| one's own finished building, damaged, and full or not a gather type | `swarm_around` | `ox`, `whom`, `QUEUE_NEW`, `REPAIR` (13) |
| one's own finished building, undamaged, not a gather building with room (the city, a full camp) | nothing | |
| an enemy unit, seen | `attack` (`GroupOut::issue_attack@0070b060`) | `ox`, `whom`, `ignore` 0, `QUEUE_NEW` |
| the ground | `move_to` | §1's fields |

The predicates, in the dispatch's order:

1. **An enemy unit under the cursor is looked for first**, then a friendly building.
2. **Built** is the object's `is_active` slot (`flags & 4`).
3. **A gather type** is `build_flags & 0x40`, the letter `g` of `BUILD_FLAGS`: the Woodcutter's camp, the Farm and the
   Mine.
4. **Room** is `BuildData::num_gatherers@00630450` below `gather_max`.
5. **Damaged** is `ObjectData +0x24` nonzero. A repair also asks that the cell's owner be nobody, the player or an
   ally.

**Ours.** The client mints these arms from the scene (`input::View::target`, `Input::on_object`). `BuildingScene::gather`
carries the gather test's inputs: the length of the building's `gather_down` chain, and `gather_max`. The look takes
the nearest anchor within 24 px. It looks for another slot's unit, then the slot's own building, then another slot's
building. Another slot's building draws the attack, as the dispatch's attack search does, with units and buildings in
its mask.

### 6.2 The third script

`rondata::script::right_pressed()` is the clicked script (§5) with two right presses on the opening frames' own
objects, and nothing else:

| frame | press, then right press | pixels at the opening camera | what the layer reads | the order |
| --- | --- | --- | --- | --- |
| 120 | the woodcutter `0/2` (TI 50, Citizen, land), then the Farm's site `0/2007` (TI 417) | (1678, 468), (1338, 463) | own, not built | `swarm_around` 2007, 0, 2, `BUILD_AT` |
| 200 | the trained Citizen `0/6` (TI 50, land), idle by the Village, then the Woodcutter's camp `0/2001` (TI 418) | (976, 602), (1640, 438) | own, built, room | `gather` 2001, 2 |

`INPUT_RIGHT_PRESSED`: **5 of 5**. The seven clicked events and these four mint the script field for field
(`minted_right_pressed_at_the_opening_camera`, `RIGHT`). Before this item it was 0: no right press on an object.
Seen red: the script's `BUILD_AT` set to `REPAIR` gave 4 of 5. `INPUT_CLICKED` stays 3 of 3.

The turn pump takes `swarm_around`, `gather`, `garrison` and `attack` through their `group_*` entries, through
`Sim::command` (`rondata::input::Stream`).

**The file** is `~/ron-golden/recordings/item4179-right-pressed-greatlakes-24000.rcx`, 59,312 bytes, SHA-256
`020ed105…a334c` (`RIGHT_PRESSED_RCX_BYTES`). It is the clicked file with two packages added; the closing package's
`stamp` goes from 5 to 7 with them (`the_right_pressed_minted_packages_round_trip`). The client writes the same bytes
from the orders it minted.

### 6.3 The playback, and each order's effect

**run1568**: the click-free lane, `end:MISC,UNITS=3,BUILDS=7`, 300 frames (`docs/RUNS.md`).
`the_original_s_playback_of_the_right_pressed_file_walks_in_lockstep` gives `RIGHT_PRESSED_WALKED`: draws agree on
**300 of 300** frames, count and sequence; **299 records compared whole, none parted**. Seen red first: pinned
`(0, 0, 0, 0)`, it printed `(300, 300, 299, 0)`.

**The effects, both sides** (`right_pressed_effects`, on 121, 201 and 299), equal:

- the helper `0/2`: `move`, `build` on 121; `build` on 201; `gather` on 299;
- the trained `0/6`: no order on 121; `move`, `gather` on 201 and 299;
- the site `0/2007` finished first on **254**.

The clicked orders' own effects hold, but for one: on 299 the first builder `0/1` holds `move`, `gather`
(`right_pressed_base_effects`).

**The value diff** against the clicked file's playback, run1563a. Our clicked walk agrees with run1563a on all 299
records, and the control walk (`clicked_right_effects`) reads it:

| record | field | run1563a | run1568 |
| --- | --- | --- | --- |
| `0/2` on 121 | position | (4536, 28680) | (4519, 28700) |
| `0/2` on 121 | orders | 1 | 2 |
| `0/2001` on 121 | `gather_down` | 2 | −1 |
| `0/2007` | `flags & 4` set from | 297 | 254 |
| `0/6` on 201 | orders | 0 | 2 |

**Made to fail** (`the_clicked_script_against_the_right_pressed_playback_parts`, `CLICKED_AGAINST_RIGHT_PRESSED`):
our walk under the clicked file, against run1568, parts first on 121, on the helper's position. 179 of 299 records
part, and the draws part from 244. Seen red first: pinned `(0, "", "", None, 0)`.

### 6.4 Confidence, and what is not established

**Backed by the playback.** The arms for one's own unfinished building and for a gather building with room are played
by run1568, with their effects equal on both sides.

**Reading only.** The table's other rows rest on one reading with its call sites checked. No blind second reading was
made. Those rows are the repair, the nothing, the attack and the predicates' order.

**Not established:**

- ~~**The attack.** It is minted, but no enemy stands in view on the opening frames, so no file carries it. It waits for
  a lobby with one (parked 4179a).~~ Played on the harness lobby on the frame our walk first shows an enemy (§8).
- ~~**A selection that cannot attack.** A Scout's `ATTACK` is 0. On an enemy unit it reads as `follow`
  (`GroupOut::issue_follow@00708980`), and on an enemy Spy as a counter-intelligence spell. The layer mints neither:
  it would mint an attack. The scene does not carry a unit's attack (parked 4179b).~~ The follow is minted and played
  (§10). The counter-intelligence arm is not minted (§10.4).
- **The garrison.** It is issued only with the profile's `0x200` set, the advanced option. Then it is issued ahead of
  the build, gather and repair arms, for any building with a garrison limit. The default profile clears it, so the
  layer has no garrison arm on the right press (parked 4179c). ~~No garrison is minted.~~ The unit's garrison button
  and its pick write the same bytes, and are minted and played (§23).
- **Trees, mountains and oil.** The dispatch sends a Citizen to the nearest own Woodcutter's camp or Mine within
  `0x600` units, and gathers there. The scene does not carry the terrain object, so the layer does not mint this (parked 4179d).
- ~~**Is seen.** The attack asks `is_seen`; an unseen enemy is moved to instead. The layer does not test the fog.~~
  The layer tests it, and the fog's press is played (§10).
- **The repair's territory test**, and **the University's** Citizen arm.
- **The room test's count.** `num_gatherers` counts the chain's units that `is_gathering_at` the building, plus a few
  types' insiders. The layer takes the chain's length.
- **Allies.** The scene names an owner, not an alliance. The layer treats every other slot as an enemy.
- ~~**A right press on one of the slot's own units**: in the original, follow or guard. The layer moves to the ground
  under it.~~ It is a `guard` (`is_ally` of the slot itself), minted and not played (§10).

## 7. A dragged box, and a group that moves (item 4180)

A press, a drag and a release select a box of the slot's units, and a right press on the ground moves them as one
`group`. The ids and the move's formation fields are the ones the original mints for that selection, read off
`Console::on_left_up@007cde90`, `Console::down@007cd8f0` and the functions they call, and checked against the
recordings on disk where the recordings can say.

### 7.1 The box, and the move's fields

**The box** is `Console::on_left_up`'s arm for a release that is no longer `clicking`:

1. The rectangle is `select_x`..`select_x2` and `select_y`..`select_y2`, sorted (`RectTemplateBase<short>::sort`).
   Without shift the selection is cleared first.
2. The units are walked before the buildings. The buildings are walked only when no unit was taken. The slots are
   walked from the console's own, and the first slot with a unit in the box fixes the selection's owner. Within a
   slot the objects are walked in index order, from `obj_base[kind]`.
3. A unit is in the box when it is alive, on the map, and seen by the console's player, and when
   `UnitOut::around_bounding_volume@00608a30` holds: one of its figures' bounding boxes has its centre, projected,
   within 30 px of the rectangle on every side (`BoundingBox::around_bounding_volume@00544860`, `0x41f00000` and
   `0xc1f00000`).
4. Each unit is added by `SelectGroup::add@00716c10`. It refuses past `0x80` members, and it inserts the unit before
   the first member whose `TypeIndex` (`ptype +4`) is greater. So the list is sorted by type, and within one type it
   is in index order.
5. **The civilian rule.** The default profile sets pref `0x200000` (`PlayerProfile::default_mouse@005d01f0` and
   `default_profile@005ceca0`), so a box with no modifier is filtered after the adds:
   - When every member but the Merchants (61), fishermen (317) and trade units is a worker
     (`ObjectData::is_worker@0046fa10`: TI 50 to 53), and there is a worker, only the workers stay (mode 5).
   - Otherwise the civilians go (mode 1): caravans, workers, merchants (`UnitData::is_merchant@0046d370`: 61, 62,
     400), and anything that `is` the merchant fleet (318) or the fishermen (317).
   - So a box of Citizens and a Scout takes the Scout alone.

**The move's fields.** A right button going down with a selection writes `drag_form = max(GroupData::get_form, 0)`
and `drag_form_mod = GroupData::get_form_mod_option` (`Console::down`, its `param_3 == 1` arm). A plain right press on
the ground then reaches `Console::execute_at_cursor@007c6630`, which calls `GroupOut::issue_move_to(drag_x2,
drag_y2, queue, 0, 0, MOVE_TO, drag_form, drag_form_mod, 0)`. `CommandManager::issue_move_to@00941720` writes the
arguments into the command's bytes in that order: `set_angle` 0, `angle` 0, `orders` 1, `queued`, `form`, `width`,
`disembark` 0.

- `get_form@0070b9f0` is the `UnitData +0xaa` that every active member on the map shares, and −1 as soon as two
  differ.
- `get_form_mod_option@0070bd00` is, for more than one member, the mean of their `+0xab` bytes that are not −1, and 50
  when none is. For one member it is the leader's own byte, or 50.

The minimap's right press is another arm: `WorldMap::on_right_up@008c7050` issues `MOVE_TO` with `-1, -1`.

**The recordings agree.** In the heavengames sample (Seifer008's 2017 match), the `move_to` commands with
`orders` 1 and `set_angle` 0 carry `form`/`width` (0, 50) or (9, 50), at every group size from one to nine and up.
They also carry (−1, −1) 114 times for one unit, which is the minimap's arm. Its 2-to-9-unit groups are ascending in
object number 66 times and not 81 times, as a list sorted by type first would be. The 49 recordings this machine's
captures wrote hold `move_to`s with (0, 50) and (9, 50) only. This was a one-off probe over
`recgame::tests::recordings_on_disk`, not kept as a test.

**The bytes on the box's frame.** run1568's `UNITDATA` prints `form 9` and `form_mod -1` for `0/3`, `0/4` and `0/5`
on frames 230 and 239. So the minted move carries `form` 9 and `width` 50.

**Ours.**

- The box tests each unit's anchor (§1) within `BOX_MARGIN`, 30 px, of the rectangle. The original projects each
  figure's bounding box centre instead.
- Only the issuing slot's units are taken. A box with none clears the selection, where the original walks the
  buildings next.
- The window sends a drag when the release is `DRAG_SLOP` (4 px) or more from the press. The original's `clicking`
  threshold is not read.
- The window now selects on the left button's release, as `on_left_up` does.
- ~~**One unit keeps the scripts' (−1, −1).**~~ The main view mints `max(form, 0)` and the leader's width for one
  unit too, (0, 50) for the Scout on frame 30; the scripts' −1/−1 is the minimap's arm. Played by run1572 (§9).

The scene carries what the box and the formation read (`observe::UnitScene`): `type_index`, `form` and
`form_width`. In the layer they are `input::View::boxed` and `View::formation`, with `Event::Drag`.

### 7.2 The fourth script

`rondata::script::boxed()` is the right-pressed script (§6) with one more package:

| frame | events | pixels at the opening camera | what the layer reads | the order |
| --- | --- | --- | --- | --- |
| 240 | a drag from (740, 450) to (910, 610), then a right press | the three anchors at (886, 587), (765, 522), (795, 473); the right press at (700, 600) | `0/3`, `0/4`, `0/5` (TI 50, land, farming at `0/2002`..`0/2004`), `form` 9 and `form_width` −1 each; the ground (2344, 32710) | `group` [3, 4, 5]; `move_to` (2344, 32710), `set_angle` 0, `angle` 0, `orders` 1, `queued` 2, `form` 9, `width` 50, `disembark` 0 |

The nearest other unit of slot 0 on 240 is `0/6`, at (1185, 616), outside the grown rectangle. The box's three are
one type, so the type sort leaves them in index order. They are all workers, so the civilian rule keeps all three.

`INPUT_BOXED`: **6 of 6** (`minted_boxed_at_the_opening_camera`). The right-pressed events, the drag and the right
press mint the boxed script field for field. Before this item it was 0: no box and no group of more than one unit.
`INPUT_RIGHT_PRESSED` stays 5 of 5 and `INPUT_CLICKED` 3 of 3. The rule without a walk is
`minted_a_box_lists_filters_and_forms_as_the_original`: the type sort, the civilian rule both ways, another slot
left out, and the formation's clamp and mean.

**The file** is `~/ron-golden/recordings/item4180-boxed-greatlakes-24000.rcx`, 59,338 bytes, SHA-256
`5d797ddd…3c838` (`BOXED_RCX_BYTES`). It is the right-pressed file with one package added, 31 bytes of payload. The
closing `stamp` goes from 7 to 8 (`the_boxed_minted_packages_round_trip`). The client writes the same bytes.

### 7.3 The playback, and each unit's path target

**run1569**: the click-free lane, `end:MISC,UNITS=3,BUILDS=7`, 300 frames (`docs/RUNS.md`).
`the_original_s_playback_of_the_boxed_file_walks_in_lockstep` gives `BOXED_WALKED`: draws agree on **300 of 300**
frames, count and sequence; **299 records compared whole, none parted**. Seen red first: pinned `(0, 0, 0, 0)`, it
printed `(300, 300, 299, 0)`. §5's and §6's effects hold on it unchanged.

**Each unit's path target, both sides** (`boxed_effects`, on 241, 270 and 299), equal:

| unit | 241 and 270: the move's target | the path's goal | `form`, `form_mod` | 299 |
| --- | --- | --- | --- | --- |
| `0/3` | (2328, 32712) | (2344, 32710), the click | 9, 50 | stands on its goal, no order |
| `0/4` | (2232, 32760) | (2214, 32775) | 9, 50 | stands on its goal, no order |
| `0/5` | (2424, 32568) | (2408, 32580) | 9, 50 | stands on its goal, no order |

Each Citizen goes to its own slot of the mob formation round the click. The move's target is the slot's 48-unit cell
centre, and the path's goal is the slot.

**The value diff** against the right-pressed file's playback, run1568, on the same frames:

| record | field | run1568 | run1569 |
| --- | --- | --- | --- |
| `0/3` on 241 | position | (2808, 31992) | (2794, 32014) |
| `0/3` on 241 | orders | `gather` behind a move to (2424, 32376) | one move to (2328, 32712) |
| `0/3`, `0/4`, `0/5` on 241 | `form_mod` | −1 | 50 |
| `0/4` on 241 | `orders_x`, `orders_y` | (2040, 31992) | (2232, 32760) |
| `0/5` on 241 | `orders_x`, `orders_y` | (1848, 31608) | (2424, 32568) |
| `0/3`, `0/4`, `0/5` on 299 | position | (2424, 32376), (2040, 31992), (1848, 31608), farming | (2344, 32710), (2214, 32775), (2408, 32580) |

Our right-pressed walk is the control (`right_box_effects`): it reads run1568's column.

**Made to fail** (`the_right_pressed_script_against_the_boxed_playback_parts`, `RIGHT_PRESSED_AGAINST_BOXED`): our walk
under the right-pressed file, against run1569, parts first on 241, on `0/3`'s position, (2808, 31992) against
(2794, 32014). 59 of 299 records part, all of them from 241, and the draws part from 240. Seen red first: pinned
`(0, "", "", None, 0)`.

### 7.4 Confidence, and what is not established

**Backed by the playback.** A box of three Citizens moved with `form` 9 and `width` 50 is played by run1569, with
each unit's slot, path goal and formation bytes equal on both sides.

**Backed by the recordings.** The main view's formation fields, and the minimap's (−1, −1), as §7.1's histogram.

**Reading only**, one reader, with the call sites checked; no blind second reading:

- the box's predicates: the 30 px margin, units before buildings, the slot order, the seen test;
- the type sort, and the cap;
- the civilian rule and its pref;
- `get_form_mod_option`'s mean.

**Not established:**

- **A box of more than one type.** The type sort is read, and the recordings' order fits it. No playback carries it.
- **The civilian rule in a playback.** No box with a Scout, a Merchant or a soldier among the workers has been played.
- **A formation other than the mob's.** `form` 0 with soldiers, a width other than 50, and the right drag that
  sets an angle (`Console::on_right_up`'s `set_angle` 1 arm, and the wheel's `drag_form_mod` ±5).
- **The pump drops `form` and `width`.** `rondata::input::group_move_to` passes neither, and `Sim::group_action_move_to`
  takes `max(get_form, 0)` and `get_form_mod_option` as the original does for (−1, −1). These are the values the right
  press mints, so the effect is the same here. A drag-set formation would part (§7.1; parked 4180b).
- **The figure's bounding box.** The anchor stands in for its centre. A unit near the margin can fall either way.
- **Another slot's units in a box, buildings in a box, shift, ctrl and alt,** and the planes' arm of the width's
  mean.
- ~~**The single unit's main-view fields** (parked 4180a).~~ Played by run1572 (§9).

## 8. An attack on an enemy unit (item 4195)

A soldier is trained and right-pressed on the first enemy our walk shows slot 0. The original plays the file in
lockstep up to the soldier's first blow, and the target's hits are equal on both sides. §6.1's enemy arm is the one
played here: `attack(ox, whom, ignore 0, QUEUE_NEW)`.

### 8.1 The fifth script and its stage 0

**Where an enemy is first seen.** The harness lobby has two players. Slot 1 starts about 37,000 units from slot 0.
Under the boxed script our walk shows slot 1's units to slot 0 on no frame before 6187 (`Sim::world_sees`, the
live sight at the unit's half-cell). On 6187 slot 1's Scout `1/0` (TI 69, land) wanders within 1,465 units of
slot 0's objects. Slot 1's Citizens never leave their base. Slots 8 and 9 are nature, not enemies.

**What a soldier needs.** Slot 0 starts with no Barracks. The Barracks' `PREQ0` is The Art of War, researched at the
Library. Until it is researched, `group_action_build` places nothing.

`rondata::script::attacked()` is the boxed script (§7) with four packages more:

| frame | events, at the opening camera | what the layer reads | the order |
| --- | --- | --- | --- |
| 310 | a press on the Library `0/2005` (1006, 397), then `a` | a building selected | `group` [2005]; `queue_up` 572 (The Art of War), `num` 1 |
| 521 | a press on `0/3` (TI 50) (735, 523), then `b`, then a press at (614, 475) | the ground (1249, 32163) | `group` [3]; `build` 427 at (1249, 32163), `x2`/`y2` the same, `queued` 2 |
| 965 | a press on the Barracks `0/2008` (614, 458), then `s` | a building selected | `group` [2008]; `queue_up` 82 (the Slingers), `num` 1 |
| 6703 | a press on the Slinger `0/7` (TI 82, land) (509, 513), then a right press on `1/0` (1645, 884) | another slot's unit | `group` [7]; `attack` `ox` 0, `whom` 1, `ignore` 0, `queued` 2 |

- The build and the train come ten frames after our walk first allows each. The Barracks is available from 511 and
  finished on 955.
- The drop pixel is the pixel of (1248, 32160), the clear site on a 192-unit grid nearest the builder. Its ground
  under the layer's inverse is (1249, 32163), and `snap_center` puts the site at (1152, 32064).
- 6703 is the first frame our walk shows `1/0` under this script. The first-seen frame moves with the orders' timing:
  with the build and the train issued as soon as allowed it was 5474, and under the boxed script 6187.
- Every anchor is on the 1920×1080 screen at the opening camera, so the script's camera is the client's own.
- Each press lands on its own object. The nearest other anchors are 142, 4, 74 and 17 px away; the 4 px one is the
  Farm `0/2003` beside `0/3`.
- Under the right press, `View::target` finds `1/0` and no other slot's unit within the pick.

The keys are ours: `a` and `s` in `TRAIN_KEYS`, `b` in `BUILD_KEYS`. The HUD's menu grows to match
(`docs/HUD.md` §2), and its labels name the technology through `BASE_TECHTYPES`.

`INPUT_ATTACKED`: **10 of 10** (`minted_attacked_at_the_opening_camera`, `ATTACK_ANCHORS`, `BARRACKS_DROP`). The boxed
events and these nine mint the attacked script field for field. Before this item it was 0: no file carried an attack.
Seen red: the script's `ignore` set to 1 gave 9 of 10. `INPUT_BOXED` stays 6 of 6.

**The file** is `~/ron-golden/recordings/item4195-attacked-greatlakes-24000.rcx`, 59,404 bytes, SHA-256
`619c27db…bcb2e6` (`ATTACKED_RCX_BYTES`). It is the boxed file with four packages added, and the closing `stamp` goes
from 8 to 12 (`the_attacked_minted_packages_round_trip`). The client writes the same bytes from the orders it minted.

### 8.2 The playback, and the blow

**run1571**: the click-free lane, `end:MISC,UNITS=3,BUILDS=7` with records in the window 6690..6900, to frame 6900
(`docs/RUNS.md`). `the_original_s_playback_of_the_attacked_file_walks_in_lockstep` gives `ATTACKED_WALKED`:

- draws agree on **6,900 of 6,900** frames, count and sequence;
- **210 records compared whole, none parted**, 6690 to 6899 (the window prints no record of 6900).

Seen red first: pinned `(0, 0, 0, 0)`, it printed `(6900, 6900, 210, 0)`.

**The attack's effect, both sides** (`attacked_effects`, on 6703, 6704, 6804, 6805 and 6899), equal:

| frame | `1/0`: position, hits left | `0/7`: position, orders |
| --- | --- | --- |
| 6703 | (6762, 31117), 50 | (1176, 32760), none |
| 6704 | (6731, 31133), 50 | (1202, 32744), `move`, `attack` |
| 6804 | (4320, 32084), 50 | (3104, 31512), `move`, `attack` |
| 6805 | (4320, 32118), **47** | (3135, 31512), `move`, `attack` |
| 6899 | (2130, 33412), 31 | (3463, 31825), `move`, `attack` |

The hits left are `myhits − damage` in the dump and `Unit::health` in ours. Our walk's values were pinned before
the playback was read.

**The value diff** against the boxed script's walk, the control. No playback of the boxed file reaches these
frames: run1569 stops at 300. On 6805 the boxed walk has no `0/7`, and `1/0` stands at (8472, 33213) with 50 hits.

**Made to fail** (`the_boxed_script_against_the_attacked_playback_parts`, `BOXED_AGAINST_ATTACKED`): our walk under the
boxed file, against run1571, parts on every one of the 210 records. The first parting, on 6690, is the Slinger
`0/7`, which that walk never trains. The draws part from 535, while the Barracks' site is under construction. Seen
red first: pinned `(0, "", "", None, 0)`. **Our side perturbed**: with the attack consumed one frame late, on 6704,
the lockstep test turns red. What it printed is in the journal.

### 8.3 Confidence, and what is not established

**Backed by the playback.** These are played by run1571, with the blow's hits equal on both sides:
- §6.1's enemy arm, an `attack` with `ignore` 0 and `QUEUE_NEW` on another slot's unit;
- a `queue_up` of a technology at the Library;
- a `build` of the Barracks;
- a `queue_up` of a soldier.

**Not established:**

- ~~**The right press's fog test.** The dispatch asks `is_seen`, and an unseen enemy is moved to instead. The layer still
  does not test the fog. The script presses only on the frame our walk first shows the Scout.~~ Tested and played
  (§10).
- ~~**A Scout or another unit with no attack, selected** (parked 4179b, then item 4194's). Its right press would read as
  `follow` (§9.4).~~ Its follow is played (§10).
- **An enemy building, an ally, and a target the soldier cannot reach.**
- **The record of 6900.** The window ends one record short of the take's last frame.
- **The original's hotkeys and menu.** The keys `a`, `s` and `b` are ours.

## 9. One unit's press, as the main view mints it (item 4194)

A right press on the ground mints the selection's formation for one unit as it does for many. The −1/−1 that every
script before this one carries for its single move is the minimap's arm, not the main view's.

### 9.1 The two arms

- **The main view.** A right button going down with a selection writes `drag_form = max(GroupData::get_form, 0)` and
  `drag_form_mod = get_form_mod_option` (`Console::down@007cd8f0`, §7.1). `Console::execute_at_cursor@007c6630`'s
  ground arm hands both to `issue_move_to` as `form` and `width`. Nothing in that path tests the selection's size.
  For one member, `get_form` is its own `UnitData +0xaa`, and `get_form_mod_option` is its own `+0xab`, or 50 when
  that is −1.
- **The minimap.** `WorldMap::on_right_up@008c7050` takes the world point the minimap names. It holds that point
  inside the map (`0 ≤ x < worldc +0x18 · 0xc0`, the same in `y`) and asks `Group::can_activate`. When
  `count(COUNT_DOMAIN, 4)` is 0 (a domain this item did not name) it issues `issue_move_to(…, MOVE_TO, -1, -1, 0)`, or `ATTACK_TO` with
  ctrl held.

**The bytes on the press's frame.** run1563a's `UNITDATA` prints `form 0` and `form_mod -1` for the Scout `0/0` on 29.
So the main view's move carries `form` 0 and `width` 50 (`script::SCOUT_FORM`, `SCOUT_WIDTH`).

**Ours.** The layer's `Event::RightPress` on the ground always mints `View::formation`. `Event::MapRightPress`
carries a world point and mints −1/−1, bounded by `View::map_extent`. The minimap is not drawn and its pixel
transform is not read, so the event carries the point rather than a pixel. The window sends no minimap event.

### 9.2 The scripts, and the counts

**The main view's clicked script** is `rondata::script::clicked_main_view()`: `clicked()` (§5) with the move's
`form`/`width` (0, 50). It stands beside `clicked()` and changes nothing else.

| frame | events | the order |
| --- | --- | --- |
| 30 | a press on the Scout at (1006, 629), then a right press at (1384, 862) on the main view | `group` [0]; `move_to` (5883, 31896), `orders` 1, `queued` 2, **`form` 0, `width` 50** |
| 60, 90 | §5's | §5's |

`INPUT_CLICKED`: **3 of 3**, now counted against this script (`minted_clicked_at_the_opening_camera`). The old
clicked script counts **2 of 3** against the same events (`INPUT_CLICKED_MINIMAP_ON_MAIN`): the move parts on
`form`/`width` alone. That was "fails today": the layer minted −1/−1 for one unit. Seen red: `SCOUT_WIDTH` set to 49
gave 2 of 3.

**The played files keep their counts.** The test lists for the earlier scripts send the Scout's move as the
minimap's press, at the ground stage 0's inverse names under the event's camera: (5880, 31896) under the closer
camera, and (5883, 31896) under the opening one. `INPUT_MINTED` 3 of 3, `INPUT_MINTED_OPENING` 1 of 3 with
`(30, 3, 0)`, `(90, 2, 4)`, `INPUT_RIGHT_PRESSED` 5 of 5, `INPUT_BOXED` 6 of 6 and §8's `INPUT_ATTACKED` 10 of 10 are unchanged. The HUD's
`HUD_BUTTONS` 2 of 2 is unchanged too, and every file those tests write is still byte for byte the file the original
played. The same test asserts that the minimap's list mints `clicked()` whole and writes the played clicked file.

**The file** is `~/ron-golden/recordings/item4194-clicked-main-view-greatlakes-24000.rcx`, 59,278 bytes, SHA-256
`7a8bffce…5f864d` (`CLICKED_MAIN_RCX_BYTES`). It parts from the clicked file in the move's package alone
(`the_clicked_main_view_minted_packages_round_trip`). The client writes the same bytes from the orders it minted.

### 9.3 The playback, and each order's effect

**run1572**: the click-free lane, `end:MISC,UNITS=3,BUILDS=7`, 300 frames (`docs/RUNS.md`).
`the_original_s_playback_of_the_main_view_clicked_file_walks_in_lockstep` gives `CLICKED_MAIN_WALKED`. The draws
agree on **300 of 300** frames, count and sequence, and **299 records are compared whole, with none parted**. Seen
red: pinned `(0, 0, 0, 0)`. Each order's effect is equal on both sides on 31, 61, 91 and 299, and it is §5's
(`clicked_effects`):

- the Scout at (3514, 31896), (4534, 31896) and (5554, 31896), and on (5883, 31896) on 299;
- the Village's queue `[50]` on 61 and 91;
- the site `0/2007` at (3456, 29760) from 91;
- the builder's `move`, `build`, then `gather`.

**The Scout's formation bytes, both sides** (`the_main_view_s_scout_holds_the_move_s_formation_on_both_sides`,
`SCOUT_FORM`):

| frame | run1572 `form`, `form_mod` | our walk `form`, `form_width` |
| --- | --- | --- |
| 29 | 0, −1 | 0, −1 |
| 31 | 0, 50 | 0, 50 |
| 101 | 0, 50 | 0, 50 |
| 299 | 0, 50 | 0, 50 |

Seen red: pinned −1 on every frame.

**The value diff against run1563a**, the minimap's −1/−1, is **none**. The two dumps differ only in three wall-clock
lines before frame 0: two `SYNCPOINT` timings and `Sys Memory`. On frame 31 run1563a's Scout prints `form 0`,
`form_mod 50`, and so does run1572's. Its `MOVEORDER` `x` reads 5880 in both. So the original fills an unset
`width` with what the main view would have sent, and our walk's `Sim::group_action_move_to` does the same (§7.4).

**Made to fail** (`the_minimap_s_clicked_file_against_the_main_view_playback`). The played clicked file against
run1572 parts **nowhere**: 299 records, none parted, and no draw parts. Seen red: pinned to part on 31. The first
script's file, three units off on the move, parts against run1572 exactly as it parts against run1563a: frame 31, the
Scout's path target (5880, 31896) against (5883, 31896) (`FIRST_AGAINST_CLICKED`). So this take can fail. It does
not tell the two clicked files apart because their state is the same.

### 9.4 Confidence, and what is not established

**Backed by the playback.** One unit moved with the main view's (0, 50) is played by run1572, in lockstep for 300
frames, with each effect and the formation bytes equal on both sides.

**Reading only**, one reader, with the call sites checked:

- the minimap's bound and its `can_activate` test;
- its other arm, for `count(COUNT_DOMAIN, 4)` nonzero, which this item did not read;
- that nothing on the main view's path tests the selection's size.

**Not established:**

- ~~**A Scout on an enemy draws `follow`, and an ally `guard`** (parked, second).~~ Read to the end, the member's
  test named by the PDB, and the follow played: §10. The dispatch's arm for a selection
  none of whose members can attack is read as far as `GroupData::count(COUNT_ATTACK)`. A seen unit under the cursor
  draws `GroupOut::issue_guard` when it is an ally's, and `issue_follow@00708980` otherwise. `COUNT_ATTACK` counts the
  members whose type's `+0x1e8` is nonzero and whose vtable slot `+0xe8` answers nonzero. The export names neither:
  the type's field prints as padding, and COMDAT folding names the slot `is_captain`. The layer still mints an attack.
  No take can play it here: the harness lobby shows slot 0 no enemy until frame 6703 (item 4195).
- **The minimap's pixel.** Its transform from a minimap pixel to the world is not read, and the client draws no
  minimap.
- **A one-unit `form` other than 0**, and a member whose `form_mod` is set. Neither is played.

## 10. A press that follows, and the fog's press (item 4202)

A Scout cannot attack. Its right press on an enemy unit mints `follow`, not `attack`. A right press on an enemy the
fog hides names no object at all: it is a move to the ground under the cursor. Both are minted on the attacked script
(§8), one frame either side of the Slinger's attack.

### 10.1 What the dispatch reads

`Console::execute_at_cursor@007c6630`, the same no-modifier path as §6.1. The member names come from the PDB's type
stream (`llvm-pdbutil dump --types`), not from the code around them:

- **`COUNT_ATTACK`** (`GroupData::count@00711720`) counts a member whose type's `+0x1e8` is nonzero and whose
  `ObjectData` vslot `+0xe8` answers yes. The type record names `+0x1e8` `attack` (`docs/ORDERS.md`). `ObjectData`'s
  field list names vftable offset 232 `is_captain`, `public intro virtual`. The export calls the slot `is_captain`
  too, and the record settles it. The count's other arm is a `TRANSPORTBARGE` (0x140) on the map whose `+0x28`
  object is such a member.
- **The arm for a selection with `COUNT_ATTACK` 0.** The dispatch reaches `LAB_007c98d3` past the gather,
  air-garrison and air arms, which a Scout does not enter. Before that, the Scout's own arm (`COUNT_TYPE` 0x45,
  `SCOUT`) looks under the cursor for one of the player's own objects that is infiltrated (vslot `+0xdc`) and casts
  `COUNTERINTEL` on it. An enemy unit is not the player's own, so it falls through. Then the spy arm (0x3a) and the
  trade arms (0x3b, 0x13e) are skipped. Last comes `find_target(SEARCH_ALL, units, FILTER_ALL)`: a unit found there
  draws `GroupOut::issue_guard(ox, whom, QUEUE_NEW)` if `LeaderData::is_ally` answers yes, and
  `issue_follow(ox, whom, QUEUE_NEW)` otherwise. `is_ally` answers yes for the player itself (`param == who`).
- **The same arm for a selection that can attack.** It is reached when the attack's search finds nothing, or finds
  an ally: `is_over_enemy`, then `find_target(SEARCH_NON_FRIENDLY)`, and an object that is neither enemy nor ally
  sets `local_54`, which skips it. So for any selection, a press on one's own unit is a `guard`.
- **The fog.** `Console::find_target_daemon@007c5930` takes an object only if it answers `is_seen(console->who, 0)`
  (`SubObjectData` vslot `+0x48`). Its field list names vftable offset 72 `is_seen`, `public intro virtual`. The
  only exceptions are an observer's flags and the semaphore's `0x42`. So every `find_target` above passes over what
  the fog hides. The press then reaches the dispatch's ground arm, `issue_move_to(drag_x2, drag_y2, QUEUE_NEW, 0, 0,
  MOVE_TO, drag_form, drag_form_mod, 0)`. The attack arm also asks `is_seen` (`+0x48`) of what it found, and sends a
  move with the same fields when the object is unseen. Either way the order is the ground's.

**Ours.** The scene carries both answers, as reads that change no state (`sim::observe`):
- `UnitScene::seen` is `Sim::target_seen_by`, the port of `UnitData::is_seen`.
- `BuildingScene::seen` is `Sim::build_is_seen`.
- `UnitScene::attacks` is `captain && unit_types[ty].combat.attack != 0`.

`View::target` takes only seen objects. After another slot's buildings it looks at the slot's own units
(`Hit::Friend`). `View::attacks` is `COUNT_ATTACK` as far as nonzero. `Input::on_object`:
- an enemy unit or building, with a member that attacks: `attack`, as before;
- an enemy unit, with none: `follow`;
- one's own unit: `guard`;
- an enemy building, with none: falls to the ground move. **Ours**: the arms past the follow are not read.

The client and the tests build the layer's scene for the issuing slot (`scene(Some(player))`).

### 10.2 The followed script, and the count

`rondata::script::followed()` is `attacked()` with two packages more (a file holds one package a frame):

| frame | events, at the opening camera | what the layer reads | the order |
| --- | --- | --- | --- |
| 6702 | a press on the Scout `0/0` (1384, 862), then a right press on `1/0` (1652, 882) | `1/0` unseen: nothing; the ground (6792, 31100) | `group` [0]; `move_to` (6792, 31100), `orders` 1, `queued` 2, `form` 0, `width` 50 |
| 6703 | §8's | §8's | §8's attack |
| 6704 | a press on the Scout (1399, 860), then a right press on `1/0` (1637, 883) | another slot's unit, seen; the Scout (TI 69) has no attack | `group` [0]; `follow` `ox` 0, `whom` 1, `queued` 2 |

- 6702 is the last frame our walk hides `1/0` from slot 0. It stands at (6792, 31099), and nothing else is within
  120 px. With the fog press in, `1/0` still stands at (6762, 31117) on 6703 and is first seen there, so §8's frames
  hold (parked 4195a's chaos did not bite).
- The stage 0 test lights the same scene (every `seen` set) and checks that the same pixel then draws the follow. So
  on 6702 the fog test, and nothing else, makes the move.

`INPUT_FOLLOWED`: **12 of 12** (`minted_followed_at_the_opening_camera`, `FOLLOW_ANCHORS`). Before this item it was
**10**: seen red with the fog filter and `COUNT_ATTACK` both taken out, the layer minted `attack` on 6702 and on 6704.
`INPUT_ATTACKED` stays 10 of 10, and every earlier count is unchanged.

**The file** is `~/ron-golden/recordings/item4202-followed-greatlakes-24000.rcx`, 59,435 bytes, SHA-256
`7d0f8302…2048b4` (`FOLLOWED_RCX_BYTES`). It is the attacked file with two packages added. The attack's package moves
up one stamp, and the closing `stamp` goes from 12 to 14 (`the_followed_minted_packages_round_trip`). The client
writes the same bytes from the orders it minted.

**The pump** takes `follow` and `guard` through `rondata::input::group_follow` and `group_guard`, the golden
harness's entries (`docs/GOLDEN.md` §19, §20). Before this item it dropped them as unmapped.

### 10.3 The playback, and each order's effect

**run1573**: the click-free lane. It is run1571's take with the followed file: `end:MISC,UNITS=3,BUILDS=7`, records
in the window 6690..6900, to frame 6900 (`docs/RUNS.md`).
`the_original_s_playback_of_the_followed_file_walks_in_lockstep` gives `FOLLOWED_WALKED`:

- draws agree on **6,900 of 6,900** frames, count and sequence;
- **210 records compared whole, none parted**.

Seen red first: pinned `(0, 0, 0, 0)`, it printed `(6900, 6900, 210, 0)`.

**Each order's effect, both sides** (`followed_effects`), equal on every frame. Our walk's values were pinned before
run1573 was read.

| frame | `1/0`: position, hits | `0/7`: position, orders | `0/0` the Scout: position, orders |
| --- | --- | --- | --- |
| 6702 | (6792, 31099), 50 | (1176, 32760), none | (5883, 31896), none |
| 6703 | (6762, 31117), 50 | (1176, 32760), none | (5908, 31874), `move` |
| 6704 | (6731, 31133), 50 | (1202, 32744), `move`, `attack` | (5933, 31852), `move` |
| 6705 | (6700, 31148), 50 | (1228, 32729), `move`, `attack` | (5959, 31831), `move`, `follow` |
| 6805 | (4320, 32118), **47** | (3135, 31512), `move`, `attack` | (4506, 31571), `move`, `follow` |
| 6899 | (2122, 32090), 31 | (3240, 31944), `attack` | (2641, 32500), `move`, `follow` |

**The value diff against run1571**, the attacked file's playback:
- The fog press moves the Scout from 6703. Under the attacked file it stands idle at (5883, 31896).
- The blow is the same, on 6805, 50 to 47.
- On 6899 the two walks part. `1/0` is at (2122, 32090) on run1573 and at (2130, 33412) on run1571. The Slinger holds
  `attack` alone at (3240, 31944) on run1573, and `move`, `attack` at (3463, 31825) on run1571.

**Made to fail** (`the_attacked_script_against_the_followed_playback_parts`, `ATTACKED_AGAINST_FOLLOWED`): our walk
under the attacked file, against run1573.
- It parts first on 6703, the frame after the fog press, at the Scout: ours idle at (5883, 31896), theirs at
  (5908, 31874) with one order.
- 197 of 210 records part, and the draws part from 6725.

Seen red first: pinned `(0, "", "", None, 0)`.

### 10.4 Confidence, and what is not established

**Backed by the playback.** run1573 plays these, each with its effect equal on both sides:
- the follow arm of a selection with `COUNT_ATTACK` 0, on another slot's seen unit;
- the fog's press: a right press on an enemy the fog hides, minted as the ground's `move_to` in the Scout's own
  formation.

The count's red shows that both turn on the new tests. It is (12, 10) with the fog filter and `COUNT_ATTACK` taken
out.

**Reading only**, one reader, with the call sites and the PDB's records checked:
- that the playback could not tell the fog press's `move_to` from any other: the file is the layer's order, played.
  That the original's own dispatch would have issued it rests on `find_target_daemon`'s `is_seen` admission.
- the guard arm, on one's own unit or an ally's;
- the Scout's counter-intelligence arm on an infiltrated object of one's own;
- that the gather, air-garrison and air arms do not take a Scout's press.

No blind second reading was made of these.

**Ours, not the original's:**
- an enemy building under a press by a selection with no attack falls to the ground move. The arms past the follow
  (`local_38`'s allied search, and on) are not read.

**Not established:**

- **The `TRANSPORTBARGE` arm of `COUNT_ATTACK`.** The scene does not carry what a barge holds.
- **The counter-intelligence spell.** It is not minted: the layer has no spell order.
- **A guard played.** No take carries one. The layer mints it for a press on one's own unit (`Hit::Friend`).
- **Allies.** The layer still names an owner, not an alliance: every other slot is an enemy, so it never draws the
  follow arm's `guard` on another slot.
- **A target the fog hides behind an observer's flags** (`find_target_daemon`'s `0x42` and player-flag arms). The
  layer takes the issuing slot's own sight.
- **`UnitData::is_seen`'s two `unit_masks` bits.** These are `0x800` and `0x1000`, which `Sim::target_seen_by` does
  not carry (`docs/VISION.md`).

## 11. The kinds a game carries, and the layer's share of them (item 4249)

*Item 4249, 2026-11-14. The score is the input track's census: of the order kinds the recordings on disk carry, the
kinds this layer mints, with each kind's share of the orders. A game a person can play is one whose orders the layer
can make; this is how far it is from one.*

### 11.1 The census

The corpus is `recgame`'s (`docs/RECGAME.md` §8): the 49 recordings in the bottle's `Logs\` and the 2017 sample, 50
files. Every package of every file decodes. An order is a command `rondata::input::is_input` keeps: everything but
`camera`, `player_speed` and `check_sums`. They are counted by `Command::name` (`rondata::input::kinds`), and pinned
as `rondata::input::KINDS_ON_DISK`, checked against the disk by `the_order_kinds_on_disk`.

**`INPUT_KINDS`: 20 of 26 kinds, 2,365 of 2,429 orders (97.4 %).** It was 8 of 26 and 1,879 (77.3 %) when the census
was first taken, 9 and 2,056 (84.6 %) once the rally point (§11.2) was minted, 10 and 2,216 (91.2 %) once the chat
(§12) was, 11 and 2,240 (92.2 %) once the pause (§13) was, 13 and 2,286 (94.1 %) once the market's sale and purchase (§16) were, 14 and 2,306 (94.9 %) once the city's gather (§17) was, 15 and 2,323 (95.6 %) once the craft button's spell (§18) was, which no playback has carried, 16 and 2,336 (96.1 %) once the queue button's cancel (§19) was, 17 and 2,346 (96.5 %) once the city's name (§21) was, 18 and 2,354 (96.9 %) once the hot key (§22) was, 19 and 2,360 (97.1 %) once the garrison (§23) was, and is 20 since the buildmask (§24). The kinds the layer mints are `input::MINTS`, held to the
`Command::` values the layer's own source builds by `the_layer_mints_what_it_says`. The count is
`input_kinds_of_the_recordings_on_disk`.

| kind | orders | share | minted |
| --- | ---: | ---: | --- |
| `group` | 1,059 | 43.5 % | yes |
| `move_to` | 396 | 16.3 % | yes |
| `queue_up` | 264 | 10.8 % | yes |
| `gather_point` | 177 | 7.2 % | yes (§11.2) |
| `chat` | 160 | 6.5 % | yes (§12) |
| `attack` | 80 | 3.2 % | yes |
| `leader_options` | 50 | 2.0 % | |
| `build` | 31 | 1.2 % | yes |
| `sell` | 28 | 1.1 % | yes (§16) |
| `swarm_around` | 28 | 1.1 % | yes |
| `pause` | 24 | 0.9 % | yes (§13) |
| `city_gather` | 20 | 0.8 % | yes (§17) |
| `buy` | 18 | 0.7 % | yes (§16) |
| `spell` | 17 | 0.6 % | §18 (the button's 6; the right press's 11 are not minted) |
| `unqueue` | 13 | 0.5 % | yes (§19) |
| `gather` | 11 | 0.4 % | yes |
| `guard` | 10 | 0.4 % | yes |
| `rename_city` | 10 | 0.4 % | yes (§21) |
| `hotkey` | 8 | 0.3 % | yes (§22) |
| `garrison` | 6 | 0.2 % | yes (§23) |
| `buildmask` | 5 | 0.2 % | yes (§24) |
| `cheat_ai_toggle` | 4 | 0.1 % | |
| `come_out` | 4 | 0.1 % | yes (§26) |
| `stance` | 4 | 0.1 % | yes (§27) |
| `disband` | 1 | 0.0 % | |
| `unitmask` | 1 | 0.0 % | |

The shares are cut to the tenth, not rounded.

**Where the orders come from.** Twelve of the 50 files carry anything past `leader_options`. The 2017 sample
(`seifer008 - Toughest AI 4 v 1`, 21,044 frames) carries 2,104 of the 2,429 orders and 22 of the 26 kinds. The other
eleven are the bottle's captures of 2026-08-20 to 08-26, months before any file of ours was minted
(item 4058's, in November), so no order in the census is one this layer made. In lockstep only real input travels, so the AI
players' orders are in no file.

**What the census counts and what it does not.**
- `follow` is minted (§10) and carried by no file, so it is in `MINTS` but not in the 8.
- `guard` is counted as minted, though no take has played one (§10.4).
- `leader_options` is the lobby's, one a file: every file of ours carries it through `recgame::minted_packages`, and
  the layer does not mint it.
- `chat` in the bottle's captures is the cheat console's line (`rondata::input`'s `Chat` arm), all 160 of them;
  `cheat_ai_toggle` is a capture driver's.

**The first kind taken by share was `gather_point`**, 177 orders: a building's rally point. Of them 153 carry `action`
0 and `add_to_end` 0, 12 carry `action` 3, 9 `action` 1 and 3 `action` 2. The layer mints the first form (§11.2).
The next kind by share was `chat`, 160 orders (§12); after it come `leader_options` (the lobby's), `sell` 28,
`pause` 24, `city_gather` 20 and `buy` 18.

### 11.2 A building's rally point (item 4249)

**What the dispatch issues.** With a building selected, `Console::execute_at_cursor@007c6630` reaches its gather arm
when `GroupData::count(COUNT_GATHER_POINT)` is nonzero. It then looks twice: `find_target(SEARCH_ALL, …,
FILTER_ALL)`, then `find_target(SEARCH_FRIENDLY, …)`. When neither finds an object, it calls
`GroupOut::issue_gather_point(drag_x2, drag_y2, 0, shift)`, and `CommandManager::issue_gather_point@00941b20` adds the
group and command `0x16`: `gather_point(x, y, action 0, add_to_end)`. A plain press has no modifier held, so
`add_to_end` is 0. That is the form of 153 of the 177 recorded rally points (§11.1).

`COUNT_GATHER_POINT` (`GroupData::count@00711720`, arm 23) admits a member that is a building with any of:
- a garrison limit;
- `BuildTypeData::is_training_building`;
- one of a chain of type abilities.

**What the layer does.**
- `View::takes_gather_point` reads the training arm alone. It uses `BuildingScene::trains`, a read the scene door
  gained (`crates/sim/src/observe.rs`, `cities::build::is_training_building`); nothing the simulation computes
  changes.
- `View::names_object` stands in for the two looks: any seen object of any slot within `PICK_RADIUS`.
- A right press with such a building selected, and nothing under the cursor, mints `gather_point` at
  `ground_at`'s point, `action` 0 and `add_to_end` 0.
- On an object it mints nothing.

**The script.** `rondata::script::gathered` is the followed script (§10) and two packages more:
- on `GATHER_FRAME` 6706, the capital `0/2000` given a rally point at `GATHER_AT` (4306, 31001);
- on `GATHER_TRAIN_FRAME` 6708, the capital's Citizen.

**Stage 0.** On 6706 at the opening camera, the capital stands at (3168, 30816), `z` 536, pixel (1127, 521); it
trains. The ground pixel (1277, 611) names no object, and the layer's inverse takes it to (4306, 31001). A first point
chosen nearer, (3600, 31400), fell on the capital's own footprint. There `action_gather_point` stores (−1, −1), the
"inside" arm, and the Citizen stays in the building. The point was moved out.

**The count.** `INPUT_GATHERED`: **14 of 14**. The followed events and three more (a press on the capital, the right
press, the key `c`) mint the gathered script field for field (`minted_gathered_at_the_opening_camera`). The file
written from them is the gathered file. Seen red with the building arm minting nothing, as the layer did before: 13
orders, 12 whole in place.

**The file.** `~/ron-golden/recordings/item4249-gathered-greatlakes-24000.rcx`, 59,459 bytes, SHA-256
`d419163f…179ae01`. It is the followed file plus the two packages, and the closing stamp goes from 14 to 16
(`the_gathered_minted_packages_round_trip`).

**The playback** (run1598, `docs/RUNS.md`): draws agree on 7,000 of 7,000 frames, count and sequence. 300 records
(6700..6999) are compared whole, none parted (`GATHERED_WALKED`).

**Each order's effect, equal on both sides** (`gathered_effects`, pinned from our walk before run1598 was read):

| frame | the capital's gather list | the Citizen `0/10` |
| --- | --- | --- |
| 6706 | empty | not on the map |
| 6707 | (4306, 31001, 0) | not on the map |
| 6709 | (4306, 31001, 0) | queued; not on the map |
| 6814 | (4306, 31001, 0) | (4008, 31320), `move` |
| 6832 | (4306, 31001, 0) | (4296, 30984), idle at the point |
| 6999 | (4306, 31001, 0) | (4271, 30624), `move`, `gather` |

**The value diff against run1573**, the followed file's playback: there the capital's gather list is empty through
6899 and no `0/10` stands on the map. **Made to fail:** the followed file walked against run1598 parts first on 6707
at the capital's `gather_len`, ours 0 and theirs 1. 293 of 300 records part, and the draws part from 6813, the
Citizen's coming out (`FOLLOWED_AGAINST_GATHERED`).

### 11.3 Confidence, and what is not established

**Backed by the playback.** run1598 plays a rally point minted from a right press on bare ground, and a Citizen trained
after it walks to it, with every effect equal on both sides.

**Reading only**, one reader, no blind second reading:
- the gather arm's gate and its two looks;
- `add_to_end` as the shift's;
- `COUNT_GATHER_POINT`'s arms.

**Ours, not the original's:**
- `takes_gather_point` reads the training arm alone;
- `names_object` is the nearest-anchor look, not `find_target`'s.

**Not established:**
- **A rally point on an object**: `action` 1 and 2 (the arm by `LeaderData::is_enemy`) and 3 (a friendly target, the
  second look). The layer mints nothing there. Recorded: 9, 3 and 12 orders.
- **The shift-held rally point** (`add_to_end` 1). No recording carries one.
- **A garrisonable building that does not train**, and the type-ability arms of the count.
- **The capital's snap** to a Woodcutter or Mine under the point (`action_gather_point`'s Village arm). The point here
  is bare ground.


## 12. A chat, from the entry line (item 4250)

*Item 4250, 2026-11-14. The next kind by share after the rally point (§11.1) is `chat`, 160 orders, and it needs a part
of the HUD that did not exist: a line to type into. The score is `INPUT_KINDS`.*

### 12.1 What the chat box issues

The reading is one reader's, from the decompile; nothing here has had a blind second reading.

- **The box.** `Options::exec@007188c0` opens it with `ChatBox::exec(special_flags)`: 0 for the plain chat key, 1, 2 and
  3 for chat to all, to allies and to enemies. `ChatBox::exec@0093cf80` limits the edit box to `0x80` characters.
  `docs/ORACLE.md` already drives it with Return.
- **The send.** `ChatBox::on_modal_end@0093d560` takes the line when the box closes with `result` 0:
  - A line that begins with the cheat string (`get_cheat_string@00497c00`; the install's `CHEAT `, compared without case
    over its length by `String::ignore`) goes, in a solo game, to `CommandManager::issue_chat(who, line, 0, 0, 1 << who)`.
  - A line that begins with `@` takes `special_flags` 5.
  - Any other line goes to `LeaderOut::issue_chat@006d2c50` with the box's `special_flags`.
- **The plain box** (`LeaderOut::issue_chat`, `special_flags` 0, the options not in diplomacy's negotiate mode):
  - A first character `*` or `!`, `;`, `:` or `@` picks the targeted arms 1, 2, 3 and 5. They prefix localised strings.
  - `taunt` is `Taunts::check_for_order@009809d0`: the line's leading integer, matched against the taunts' indices, and
    −1 when none matches or the match has no order. The install's taunts are numbered from 1, so a line that does not
    begin with a digit converts to 0 and gives −1.
  - `taunt_num` is the line's number when it is shorter than four characters and names a taunt, else 0.
  - **The recipients** (`bits`): the slot itself, and every live leader (`leader_flags & 1`) whose `chat_status` toward
    the slot is 0. `Game::init_teams` leaves 0 on every seated slot of a game that is not cooperative (`docs/CHECKSUM.md`).
  - With fewer than three nations the text is sent unchanged; with more, a localised prefix goes before it.
- **The command.** `CommandManager::issue_chat@00942b10`, in a game that is not networked, trims the spaces at both ends
  and adds `chat(bits, taunt, taunt_num, text)` to the local package, or nothing when the line is left empty. Every
  recorded chat's package holds the chat alone, beside the frame's `camera` and `player_speed`.
- **Its effect.** `CommandPackage::process_chat@009454f0` hands the line to each recipient: `Leader::receive_chat`, which
  displays it (`LeaderOut::display_chat`, for the console's slot alone), and `Leader::receive_taunt` when `taunt` is not
  −1. A cheat line goes to `ConsoleWin::parse_cmd`. **A plain line writes nothing the simulation keeps.**

### 12.2 What the layer does

- **The entry line** is `Input::line`: Return opens it (`Event::Enter`). While it is open every typed key goes to it,
  Backspace takes a character, and Escape closes it and sends nothing. Return sends it (`Input::chat`). The HUD draws it
  above the menu's row as `SAY: ` and the text (`hud::line_text`, `hud::line_rect`).
- **The cheat arm**: `bits` the slot's own, `taunt` 0, `taunt_num` 0, the line trimmed.
- **The plain arm**: `bits` the slot and every leader the scene carries, `taunt` −1, `taunt_num` 0, the line trimmed.
- **What it mints nothing for**: a targeting first character, a game of three nations or more, a line whose trimmed form
  begins with a digit or a sign, and an empty line.

**`INPUT_CHEATED`: 160 of 160.** Every chat the recordings on disk carry is a cheat line, the original's own mint. Each
one's text, typed into the line by its own slot on its own frame, mints its command field for field
(`minted_cheat_lines_equal_the_recordings`). Seen red with the cheat arm taken out: 0 of 160, each parting at `taunt`,
ours −1 and theirs 0.

### 12.3 The chatted script, the count and the playback

**The script.** `rondata::script::chatted` is the gathered script (§11.2) and one package more: on `CHAT_FRAME` 6710,
`chat(CHAT_BITS 3, −1, 0, CHAT_TEXT "good luck")`. The lobby seats two nations, so the plain box names both.

**The count.** `INPUT_CHATTED`: **15 of 15.** The gathered events and the line typed on 6710 (Return, the nine
characters, Return) mint the chatted script field for field (`minted_chatted_at_the_opening_camera`). On 6710 the scene
carries leaders 0 and 1. Seen red with the line's send minting nothing: 14 orders, 14 whole in place.

**The file.** `~/ron-golden/recordings/item4250-chatted-greatlakes-24000.rcx`, 59,491 bytes, SHA-256
`95817167…c36e37`. It is the gathered file plus the one package, and the closing stamp goes from 16 to 17
(`the_chatted_minted_packages_round_trip`). Seen red first: pinned 0, it printed 59,491.

**Our side, before the playback was read.** Our walk applies nothing for a chat (`rondata::input`'s `Chat` arm). Under
the chatted file against run1598, the gathered file's playback, it is as whole as the gathered walk: draws agree on
7,000 of 7,000 frames, 300 records compared, none parted (`the_chatted_script_against_the_gathered_playback_is_whole`).

**The playback** (run1599, `docs/RUNS.md`): draws agree on 7,000 of 7,000 frames, count and sequence. 300 records
(6700..6999) are compared whole, none parted (`CHATTED_WALKED`). The dump is the same size as run1598's, 64,827,222
bytes, and our walk under either file is whole against either take: the chat moves no record on either side.

**The chat's effect on the original's side** (`CHAT_ENTRIES`, pinned before run1599 was read; seen red first with
run1599 pinned as run1598's). The trace's coverage dates each function's first entry, in sim-frames:

| function | run1598 | run1599 |
| --- | --- | --- |
| `CommandPackage::process_chat@009454f0` | never | 6709 |
| `Leader::receive_chat@006b8ab0` | never | 6709 |
| `LeaderOut::display_chat@006d3320` | never | 6709 |
| `Leader::receive_taunt@006b8bf0` | never | never |
| `Taunts::play@00980cf0` | never | never |

6709 is the tick that consumes 6710's package: run1598's rally point, on 6706, is first entered on 6705
(`CommandPackage::process_gather_point`).

### 12.4 Confidence, and what is not established

**Backed by a diff.**
- The cheat arm's fields, against the 160 chats the original minted.
- The playback: run1599 plays the typed chat in lockstep, and its trace shows the original running it as a plain
  chat, displayed and not a taunt (`CHAT_ENTRIES`).

**Reading only**, one reader:
- the plain box's `bits`, `taunt` −1 and `taunt_num` 0; the playback holds lockstep whatever they hold, and no dump
  prints them;
- the targeted arms, the prefixes and the `@` arm, which the layer refuses.

**Ours, not the original's:**
- Return opens and sends the line, Escape closes it, and `SAY:` is ours;
- every leader the scene carries is taken as live and seated;
- the limit counts UTF-16 units as `text_limit` does, but the box's own handling of a key past it is not read;
- the cheat prefix compares ASCII without case, where `_wcsnicmp` folds by the locale.

**Not established:**
- **4250a. A taunt typed into the line** (a line that begins with a number). It reaches `Leader::receive_taunt` and the
  AI's taunt fields (run1575 printed them: `docs/LEADERS.md`); our walk applies no chat. The layer mints none.
- **A game of three nations or more**, whose plain line takes a localised prefix not read. The layer mints none.
- **The targeted chats** (`*`, `!`, `;`, `:`, `@`, and the box's own 1 to 3).
- **The chat flood guard** (`Console::chat_flood_protection`), local to the box.
- **The original pausing a solo game while its box is open** (`docs/ORACLE.md`). Ours does not pause.


## 13. A pause, from the key (item 4252)

*Item 4252, 2026-11-14. The next kind by share after the chat is `leader_options`, the lobby's (§11.1); after it come
`sell` 28 and `pause` 24. Pause needs no panel, only a key, so it was taken first. The score is `INPUT_KINDS`.*

### 13.1 What the key issues, and what a pause does

The reading is one reader's, from the decompile; the recordings back its shape, and the playback (§13.3) its effect.

- **The key.** `Options::exec@007188c0`'s `OPTION_PAUSE` arm, unless the scenario disables pausing, calls
  `TurnControl::issue_toggle_pause@00956b40`. In a playback (`game.semaphore & 0x10`) that toggles locally and issues
  nothing. Otherwise it adds `pause(state)` to the local package, `state` the complement of the turn control's pause
  bit: 1 to pause, 0 to lift it. `CommandManager::issue_pause@00940ba0` builds the same two bytes for the console's
  `pause 1|0`.
- **Its processing.** `CommandPackage::process_pause@00944160` logs it to the sync log and calls
  `TurnControl::process_pause@00956990`. When `state` equals the bit, it only logs. Otherwise it calls
  `toggle_pause@00957ab0`, and in a solo game it resets the frame clock and puts up the notice
  (`IFaceMainBase::do_pause_notice`).
- **The toggle, in a solo game** (`game.semaphore & 4` clear): it sets or clears the bit and pauses or resumes the
  sound. A networked game's arm counts each slot's `pauses` (ten allowed) and announces them. The recordings' slots all
  carry `pauses` 0 (`docs/RECGAME.md` §3.2).
- **What a paused game does.** `do_frame_solo@009556a0` runs `CommandManager::process_turn` on every call, paused or
  not. While the bit is set, `check_new_frame_solo@00956320` (and `check_new_frame_playback@00956010`) answers 0, so no
  frame is stepped. So the turns go on, and every package issued while paused carries the frame the game stopped on.
  **A pause writes nothing the simulation keeps.**

**Every recorded pause holds its frame** (`every_recorded_pause_holds_its_frame`, `rondata::input::PAUSES_HELD`).
Twelve pairs, all slot 0, eleven in the 2017 sample and one in a bottle capture. Each `pause(1)` is followed, on the
same frame, by the packages issued while paused, one a turn, then by its `pause(0)`. The next package is on a later
frame. 70 packages stand between the twelve pairs: moves, builds, gathers, rally points, `city_gather`, `buy` and
`sell`. Seen red pinned `(12, 0)`.

### 13.2 What the layer does, and the file

- **The key** is `Event::Pause`. It mints `pause(!paused)` in a package of its own and turns `Input::paused`. With
  nothing selected it still mints: a pause names no object.
- **The window** binds Pause and F3 to it, which is ours, and steps no frame while the pause stands. Space stays the
  viewer's own stop, which mints nothing. The HUD draws `PAUSED` (`docs/HUD.md` §1).
- **The file.** `recgame::minted_packages` refused two packages on one frame. It now admits a package on the frame
  before it while the script stands paused, toggling as `process_pause` does. It refuses a script that ends paused,
  because the playback would stand before the closing package. `a_paused_script_holds_its_frame` holds the shape and
  four refusals.
- **Our walk.** `Stream`'s pause arm applies nothing, the turn control's. The packages on a held frame are applied in
  order before that frame's tick.

**The script.** `rondata::script::paused` is the chatted script (§12) and three packages more, all on `PAUSE_FRAME`
6712, the recorded shape:
1. `pause(1)`;
2. the capital `0/2000` given a rally point at `PAUSE_GATHER_AT` (4655, 31075) while paused;
3. `pause(0)`.

**Stage 0.** On 6712 at the opening camera the capital is still the selection and still trains. The ground pixel
(1320, 650) names no object, and the layer's inverse takes it to (4655, 31075) (`PAUSE_ANCHOR`).

**The count.** `INPUT_PAUSED`: **18 of 18.** The chatted events and three more on 6712 (the pause key, the right press,
the pause key) mint the paused script field for field (`minted_paused_at_the_opening_camera`). The runner steps no
frame between them: each event is on 6712. Seen red with the pause arm minting nothing: 16 orders, 15 whole in place.

**The file.** `~/ron-golden/recordings/item4252-paused-greatlakes-24000.rcx`, 59,515 bytes, SHA-256
`495f2016…5f8ab5`. It is the chatted file plus three packages on 6712, stamped 17, 18 and 19, and the closing stamp
goes from 17 to 20 (`the_paused_minted_packages_round_trip`). Seen red first: pinned 0, it printed 59,515.

### 13.3 The playback, and each order's effect

**Our side, pinned before the playback was read** (`paused_effects`):

| frame | the capital's gather list | the Citizen `0/10` |
| --- | --- | --- |
| 6712 | (4306, 31001, 0) | not on the map |
| 6713 | (4655, 31075, 0) | not on the map |
| 6814 | (4655, 31075, 0) | (3960, 31320), `move` |
| 6844 | (4655, 31075, 0) | (4632, 31080), idle at the point |
| 6999 | (4655, 31075, 0) | (4632, 31080), `gather` |

**The value diff against run1599**, the chatted file's playback (`PAUSED_AGAINST_CHATTED`). Our walk under the paused
file parts first on 6713 at the capital's `gather[0].x`, ours 4655 and theirs 4306. 287 of 300 records part, and the
draws part from 6832: there run1599's Citizen stands at the old point, and ours walks on toward the new one.

**The playback** (run1601, `docs/RUNS.md`): draws agree on 7,000 of 7,000 frames, count and sequence. The paused
file plays to its end: the original's playback did not stand at the pause. Of the 300 records (6700..6999), 260 are
whole and **40 part, the first on 6866** (`PAUSED_WALKED`, pinned at none parted before run1601 was read and seen red).

**Each order's effect, equal on both sides.** run1601's values on `PAUSE_READ_AT` are the table above, row for row.

**The pause's effect on the original's side** (`PAUSE_ENTRIES`, pinned before run1601 was read). The trace dates
each function's first entry, in sim-frames:

| function | run1599 | run1601 |
| --- | --- | --- |
| `CommandPackage::process_pause@00944160` | never | 6711 |
| `TurnControl::process_pause@00956990` | never | 6711 |
| `TurnControl::toggle_pause@00957ab0` | never | 6711 |
| `IFaceMainBase::do_pause_notice@0080d9e0` | never | 6711 |

The whole first-entry diff of the two traces adds only these, the rally point's `remove_current` and the notice's
`AnimNotice` chain, all on 6711.

### 13.4 Confidence, and what is not established

**Backed by a diff.**
- The pause's shape, against the twelve recorded pairs (`PAUSES_HELD`).
- The playback: run1601 plays a pause, an order given while paused and the unpause, all on one frame, with every
  draw in lockstep. The original runs the pause as a pause (`PAUSE_ENTRIES`). The rally point given while paused is
  the capital's from the next frame on both sides, and the Citizen walks to it on both sides.

**Reading only**, one reader: the key's arm and the scenario's `pause_disabled`; the networked arm and its ten
`pauses`; the solo frame clock's reset.

**Ours, not the original's:**
- Pause and F3 for the key (the install's default binding is not read);
- the `PAUSED` notice's text and place;
- the layer turns `Input::paused` when it mints, where the original turns its bit when the package is processed. In a
  solo game that is the same turn; two presses in one turn would mint `pause(1)` twice on the original.

**Not established:**
- **4252a. Forty records part on run1601 from 6866.** Player 1's `1/0`, at (2926, 32736) on both takes, takes a
  blow on 6866 that leaves its `damage_frame` at 6836: `damage` 13 → 16, `damage_frac` 4 → 9. On run1599 the same
  blow lands on 6872, and our walk under the paused file lands it where run1599 does. From 6872 the unit's flee target
  differs too (`orders_x` 1272 on run1601, 504 on run1599). Before 6866 our walk follows run1601 on every record,
  including player 1's units that move differently from run1599 from about 6830 (`1/3`'s target, a new move order).
  So the moved rally point's consequences are modelled up to there. ~~Whether the pause's three turns or the new
  situation sets off the early blow is not separated.~~ **Separated by run1602 (§14.1): the pause moves nothing; the
  moved rally point sets off the early blow, and our walk misses it.**
- **The pause notice**, `do_pause_notice`'s text and sound, and the game standing with nothing issued.
- **A paused game's own clock**: the original keeps processing turns while paused; ours steps no frame and issues
  orders on the held frame, which is the recorded shape, but nothing reads what else a paused turn runs.

## 14. The paused file without its pause (item 4254)

*Item 4254, 2026-11-14. Item 4252 left one question open (§13.4, 4252a): on run1601, forty records part from 6866,
and either the pause's three turns or the moved rally point could be the cause. One take answers it. The pause moves
nothing; the parting is in our walk. No score moves: `INPUT_KINDS` stays 11 of 26.*

### 14.1 The separation

**The rallied script** (`rondata::script::rallied`) is the paused script without `pause(1)` and `pause(0)`: the
chatted script plus one package on 6712, the capital's rally point to (4655, 31075). Its file is 59,504 bytes
(`RALLIED_RCX_BYTES`, seen red at 0) and round-trips (`the_rallied_minted_packages_round_trip`). Our walk under it is
our walk under the paused script, because `Stream`'s pause arm applies nothing.

**run1602** plays that file on run1601's line (`docs/RUNS.md` run1602).
- **Its window is run1601's, byte for byte**: 64,832,328 bytes from the first `BEGIN FRAME` on. run1599's window
  differs. So a pause given, an order given while paused and the unpause leave no dumped trace beyond the order's own.
- **Our walk against it** (`RALLIED_WALKED`): draws agree on 7,000 of 7,000 frames. 40 of 300 records part, the first
  on 6866 at player 1's `1/0`, `damage` ours 13 and theirs 16. Pinned at 0 before the take was read and seen red.
- The rally point's effects are the paused file's on both sides, and none of the pause's four functions is entered.
- `the_original_s_playback_of_the_rallied_file_separates_the_pause` holds all of this. It also asserts that run1602's
  window is not run1599's, which is the side that can fail.

**The blow itself** (a dump reading, run1599 against run1601):
- `1/0` flees west at 34 a frame on both takes, with identical records to 6865. It takes blows of 3 and 5/16 each:
  on 6837, 6838 and 6858 on both takes. Then on 6866 and 6870 on run1601 and run1602, and on 6870 and 6872 on run1599
  and on our walk. The total by 6872 is 19 and 14/16 on every side.
- The 6866 blow leaves `damage_frame` at 6836: it lands 29 frames after the last refresh, inside `overkill_frames`.
- `damage_o` 7, `damage_who` 0: the captain of player 0's group 1, `0/7`, `0/8` and `0/9`. Their records are
  identical on both takes through 6875, recharges included: `0/7` fires on 6830, `0/8` on 6844, `0/9` on 6846.
- At 6865 the two takes differ only in the Citizen `0/10`'s place, the capital's gather list, and two far units
  (`1/3` and player 8's `8/3`). So what lands the 6866 blow is state the dump does not print. A missile in
  flight is the candidate. Its flight would then depend on something the Citizen's walk changes.
- From 6872, `1/0`'s flee target differs (`orders_x` 1272 against 504). That follows the blow.

### 14.2 Confidence, and what is not established

**Backed by a diff:** the separation. run1602's window equals run1601's, and our walk parts from both on the same
record.

**Not established:**
- ~~**4254a. What lands `1/0`'s blow on 6866.** The candidate is a missile from `0/9`'s shot on 6846, or `0/8`'s on
  6844. The dump prints no missile. The next instrument is a packet at `RON_STATE_FRAME` 6860 on run1602's file,
  reading the missiles in flight and their targets, then the same on run1599's. The question to answer: why the
  Citizen walking to (4632, 31080) instead of standing at (4296, 30984) changes when a blow lands about 2,400 units from it.
  The fix follows from that.~~ **Answered by item 4256** (§15): `0/8`'s round born on 6859, released by the
  Slinger's `CHAR_ATTACK3` from a node our table lacked. The Citizen's walk moves the draws that pick the swing.

## 15. The rally point's blow: the Slinger's third swing (item 4256)

*Item 4256, 2026-11-14. Item 4254 placed the parting in our walk (§14). One packet names the round, and one row of
the release table closes it. The score: run1601's records whole, **260 → 300 of 300**, and run1602's the same.*

### 15.1 The round, and the row

**The packet** (run1603, `docs/RUNS.md`): the rallied file played to 6863 with `RON_STATE_FRAME=6860`. Its ammo pool
(`tools/recomp/ammo.py`) holds one live round: `0/8`'s on `1/0`, born on 6859, from (3762, 31769, 710) to
(2915, 32738, 632), 1 of 7 frames. Our walk held the same round, born on the same frame, from (3720, 31656, 533) to
(2884, 32738, 632), 1 of 8. So the shot is right and its launch point is wrong. Flying one frame longer, ours lands on
6867, where the dump's blow is on 6866.

**Why the rally point moves it.** `0/8` is a Slinger (piece 384). On the rallied script its figure swings
`CHAR_ATTACK3` (animation 13) from 6844; on the chatted script, `CHAR_ATTACK1` (11). The swing is a draw, and the
Citizen's walk to the moved point changes the draws before it. The draws agree with the original's on every frame of
both takes, so both sides swing the same. `CHAR_ATTACK3` releases at its frame 16, on 6859; `CHAR_ATTACK1` at 21, on
6864. That is the six frames between the takes' blows (§14.1).

**The missing row.** `sim::data::launch::MEASURED` had piece 384's `CHAR_ATTACK1` and `CHAR_ATTACK2` rows (chapter two,
`docs/COMBAT.md` §22) and no `CHAR_ATTACK3`. A release with no row leaves the unit's own square at ground height. The
row is solved from the packet's one round as §22 solves a one-shot row. The unit stands at (3720, 31656) facing
−2140667904, over ground 533. Radius 120 is the only radius that fits (3762, 31769). The bearing is the middle of its
exact fits, −256,363,008 to −250,333,696 stepped by 65,536: **−253,348,352**. The height is 710 − 533 = **177**.

**The value diff** (our walk on the rallied script, against run1602):
- On 6860, `0/8`'s round: launch (3720, 31656, 533) → (3762, 31769, 710); landing (2884, 32738) → (2915, 32738);
  `total_time` 8 → 7. The packet's, field for field (`the_rallied_game_s_round_in_flight_is_the_packet_s`, seen red
  on the old row's values).
- On 6866, `1/0`'s `damage`: ours 13 → 16, theirs 16.
- `RALLIED_WALKED` 40 → 0 of 300 records parted; `PAUSED_WALKED` (run1601) 40 → 0 of 300. Draws on 7,000 of 7,000
  frames on both.

### 15.2 Confidence, and what is not established

**Backed by a diff:** the round on 6860, against the packet; run1601's and run1602's 300 records each, whole.

**Not established:**
- The row is one shot at one facing. A bearing anywhere in the 6,029,312-wide range of exact fits reproduces this
  shot; another facing would narrow it. The same holds for the Crossbowmen's one-shot rows (`docs/COMBAT.md` §22).
- The minted game's other Slinger swings move with the row wherever a `CHAR_ATTACK3` releases. The booking gate's
  corpus says whether any floor moved.

## 16. The market's sale and purchase, from its panel (item 4259)

*Item 4259, 2026-11-14. The score is the census (§11.1): `INPUT_KINDS` 11 → 13 of 26 kinds, 2,240 → 2,286 of 2,429
orders. The HUD gains the market's row; the layer mints `sell` and `buy` as the original's market panel does; our walk
carries both out; the original plays the file (run1604) with every draw and every record in lockstep and the
leader's buckets equal on both sides.*

### 16.1 What the panel issues, and what the orders do

**The panel.** `BuildOut::make_options@00629ed0` fills a finished Market's panel (`is_active`, `is(0x1b4)`). With
`has_tribe_bonus(4)` (the Nubians' power) or `has_preq(BUY_SELL)` (Coinage), for each good but wealth and knowledge
that `type_avail(g, 1)` admits, it adds `OPTION_SELL` and `OPTION_BUY` with the good as the option's object. The sale
is added disabled under a hundred of the good, the purchase under the wealth its buy price
(`LeaderData::calc_market_prices`) asks.

**The issue.** `Options::exec@007188c0`'s `OPTION_SELL`, `_500` and `_ALL` arms call
`LeaderOut::issue_sell@006d3eb0(good, 0, 1 or 2)` on the console's leader; `OPTION_BUY` and its two the same through
`issue_buy@006d3ef0`. `CommandManager::issue_sell@00942a80` writes command `0x2f`, `sell(who, good, flags)`, `who`
the leader's own number, and adds no group; `issue_buy` writes `0x2e`, `buy`, the same way. The resource bar
(`IFaceResources`, not entered by any trace) reaches the same options, with the shift for `_500` and the alt for
`_ALL`.

**The recordings agree.** All 46 sales and purchases on disk, 28 and 18, carry `flags` 0 and stand in a package of
their own with no `group`: the 2017 sample's 44, slot 3's, and one of each in the bottle's capture of 2026-08-24
10'15'53, slot 0's, a sale of food on 1276 and a purchase of timber on 1280.

**What they do.** `CommandPackage::process_sell@009468a0`: with the ability (the bonus or the prerequisite), a market
(`LeaderData::has_market@006d5410`) and no nuclear embargo, `Leader::do_sell@006cfc60` until it refuses, once, five
times with bit 0, every hundred with bit 1. `process_buy@00946a20` is `Leader::action_buy@006cfa20` whole: the gate
`can_buy_sell@006d53e0` (`has_bonus(BUY_SELL)`, which admits the Nubians, and a market) and the embargo, then
`do_buy@006cfbd0` once, five times with bit 0, ten (or a hundred beside bit 0) with bit 2, every hundred with bit 1.
`do_sell` and `do_buy` were already ported for the AI (`docs/AI.md` §40), and so was the gate
(`Sim::market_gate`); `Sim::process_sell` and `Sim::process_buy` are the counts around them, behind the order door.

### 16.2 What the layer does

- **The scene** (`crates/sim/src/observe.rs`, reads only): `BuildingScene::market`, the type's `is(MARKET)`; and
  `LeaderScene::market`, per good the `(buy, sell)` price where the panel has a button, `None` elsewhere
  (`Sim::market_offer`, the panel's gate and its goods without the building's).
- **The HUD** (`hud::market_menu`, `docs/HUD.md` §1): with the slot's own finished Market the selection, a sell and a
  buy button for each good offered, in the goods' order, each with its price. A press on one is `Event::Sell(good)` or
  `Event::Buy(good)`.
- **The layer**: `sell(who, good, 0)` or `buy(who, good, 0)`, `who` the slot, in a package of its own, when the
  button is live (`input::trades`); otherwise nothing.

**The script.** `rondata::script::marketed` is the paused script (§13) and two packages more:
- on `SELL_FRAME` 6714, `sell(0, food, 0)` from slot 0's Market `0/2006`;
- on `BUY_FRAME` 6716, `buy(0, timber, 0)`.

Slot 0 is the Nubians, so the market trades in the Ancient age without Coinage.

**Stage 0.** On 6714 at the opening camera, the Market stands at (2688, 29568), `z` 515, its anchor at pixel
(1248, 389). Its row offers food and timber: SELL FOOD 81, BUY FOOD 106, SELL TIMBER 82, BUY TIMBER 107, all live, at
centres (1258, 1054), (1446, 1054), (1634, 1054), (1822, 1054) at 1920×1080. Slot 0 holds food 721, timber 425,
wealth 249.

**The counts.**
- `INPUT_MARKETED`: **20 of 20**. The paused events and three more, through the HUD and the layer (a press on the
  Market, its SELL FOOD button, then its BUY TIMBER button), mint the marketed script field for field
  (`minted_marketed_at_the_opening_camera`). The file written from them is the marketed file. Seen red with the arm
  adding the selection's `group` before the order: 20 orders, 18 whole.
- `INPUT_TRADED`: **46 of 46**. Every recorded sale and purchase, pressed from its leader's market panel, mints the
  recorded order (`minted_trades_equal_the_recordings`). Seen red the same way: 0 of 46.
- `minted_market_button_edges`: no row for another slot's market or an unfinished one, no button for a good not
  offered, a dead button at a hundred less one or a price less one mints nothing.

**The file.** `~/ron-golden/recordings/item4259-marketed-greatlakes-24000.rcx`, 59,535 bytes, SHA-256
`6affa99f9f45bd2b6b8263fb5f8c19af3a2a187323c7aea7eef809c117e6e3de`. It is the paused file plus the two packages,
stamped 20 and 21; the closing stamp goes from 20 to 22 (`the_marketed_minted_packages_round_trip`; seen red pinned 0).

### 16.3 The playback, and each order's effect

**Our walk under the marketed file against run1601** (`MARKETED_AGAINST_PAUSED`, pinned before run1604 was read):
whole, draws on 7,000 of 7,000 frames and 300 of 300 records. A sale and a purchase write the leader's buckets and
the market's prices; run1601 prints neither, and player 1's Market reads the prices only from its first market draw,
on 8582 of this game.

**run1604** (`docs/RUNS.md`): run1601's line with this file and `LEADERS=2` added, so the take prints the leaders'
buckets. Draws agree on 7,000 of 7,000 frames, count and sequence; 300 records compared whole, none parted
(`MARKETED_WALKED`).

**Each order's effect, equal on both sides** (`marketed_effects`, pinned from our walk before run1604 was read; the
take's leader 0 `bucket` lines read as text, `market_effects_theirs`):

| frame | food | timber | wealth | without the orders (our walk on the paused file) |
| --- | ---: | ---: | ---: | --- |
| 6714 | 721 | 425 | 249 | 721, 425, 249 |
| 6715 | 622 | 425 | 330 | 722, 425, 249 |
| 6716 | 622 | 425 | 330 | 722, 425, 249 |
| 6717 | 622 | 526 | 223 | 722, 426, 249 |
| 6999 | 653 | 544 | 229 | 753, 444, 255 |

The sale takes a hundred food and pays the sell price, 81; the purchase pays the buy price, 107, for a hundred timber.
Both prices are the ones the HUD shows on 6714 and 6716. Knowledge, metal and oil stay 0. **Made to fail:** the take's
buckets from 6715 differ from the walk without the orders. The reader reads run1577's `@dumpall` of frame 91 as
printed, (182, 153, 102, 0, 0, 0), where our walk on the minted script agrees, and reads nothing from run1601
(`the_bucket_reader_reads_a_known_dump`).

**The functions** (`MARKET_ENTRIES`, pinned before the read): `process_sell` and `do_sell` are first entered on 6713,
the tick that consumes 6714's package, and `process_buy`, `action_buy` and `do_buy` on 6715. run1601 enters none of
the five.

### 16.4 Confidence, and what is not established

**Backed by the playback.** run1604 plays a sale and a purchase minted from the market's buttons, and slot 0's six
buckets are ours on the five frames read. The sale's and the purchase's counts at `flags` 0, the gate's Nubian arm,
and the sell and buy prices on those frames are all inside that.

**Reading only**, one reader, no blind second reading:
- the panel's gate and its goods (`make_options`), and the disabled states;
- the counts for `flags` 1, 2 and 4; the emulator sweep of `process_sell` (item 3014) agrees on its counts;
- the embargo's arm.

**Ours, not the original's:**
- the row's place, size and labels; the original's market interface (`IFaceMarket`) is not read;
- a dead button mints nothing; whether the original's disabled option can be pressed is not read;
- the shift and the alt are not read, so the layer mints `flags` 0 alone;
- the market's buttons are no key's event.

**Not established:**
- **The market's price**: `Game`'s `market[6]` is not printed by the take, so the price's fall by
  `MARKET_SUPPLY_DEMAND` after a sale and its rise after a purchase are checked only through the next order's price,
  inside the buckets.
- **`flags` 1 and 2** (the shift's five hundred, the alt's all). No recording carries one.
- **The resource bar's buttons** (`IFaceResources`), the other door to the same options.
- **A market of the Ancient age offers no metal or oil**: `type_avail` is read through `Holdings::available`, and the
  take checks food and timber alone.

## 17. The city's gather, from its panel (item 4261)

*Item 4261, 2026-11-14. The score is the census (§11.1): `INPUT_KINDS` 13 → 14 of 26 kinds, 2,286 → 2,306 of 2,429
orders. The HUD gains the city's row; the layer mints `city_gather` as the city panel's buttons do; our walk carries
it out; the original plays the file (run1605) with every draw and every record in lockstep and the capital's gather
list equal on both sides.*

### 17.1 What the panel issues, and what the order does

**The panel.** `BuildOut::make_options@00629ed0` gives a city centre's panel (`SubObjectData::flags & 0x20`) four
buttons, `OPTION_CITY_FARM_GATHER`, `_WOODCUTTER_`, `_MINE_` and `_ALL_`, when the player profile's preference bit
(`prefs & 2`) is set.

**The issue.** `Options::exec@007188c0` maps the four to `Options::do_city_gather@0071cdb0` with 0x1a1 (Farm), 0x1a2
(Woodcutter), 0x1a3 (Mine) and −1. `do_city_gather` issues when the console's selection can act
(`Group::can_activate`), is of buildings (`GroupData::buildings`), counts one that takes a gather point
(`COUNT_GATHER_POINT`) and one of type 0x19e, the `VILLAGE` lineage (`COUNT_TYPE`). It flashes the selection and calls
`GroupOut::issue_city_gather@0070ac70`: the rally sound, then the selection's `group` and command `0x12`,
`city_gather(t, queued 2)`, whatever the keys.

**The recordings agree, in 18 of 20.** All 20 on disk are the 2017 sample's, slot 3's, `queued` 2; 19 carry `t` 417
and one 419. Eighteen carry a `group` of one city building (`3/2000`, `2010`, `2019` or `2037`) before the order. Two,
on 7205 and on 8179, carry a `group` of no objects; `process_city_gather` then acts on an empty group.

**What it does.** `CommandPackage::process_city_gather@00948a10` is `Group::action_city_gather(t, …)` on the
package's group; `queued` is logged and not read. The action was already ported (`Sim::group_action_city_gather`,
item 3915, `docs/GOLDEN.md` §162): each finished centre in the group has its gather list cleared, then one point per
building of its city whose type is `t` (all three for −1), at that building's own position, action 1. The golden
harness reached it (`input::group_city_gather`); our walk's `Stream` did not, and now does.

### 17.2 What the layer does

- **The scene** (`crates/sim/src/observe.rs`, reads only): `BuildingScene::city`, the building its city names (the
  centre, as `group_action_city_gather` tests it), and `BuildingScene::village`, `Sim::building_is_city`.
- **The HUD** (`hud::city_menu`, `docs/HUD.md` §1): with the slot's own city centre the selection
  (`input::city_panel`), the four buttons in `make_options`' order, in the market's place. A press on one is
  `Event::CityGather(t)`.
- **The layer**: with a building selection that `input::city_gathers` passes (every member a building of the scene,
  one that trains, one a `VILLAGE`), the selection's `group` and `city_gather(t, 2)`; otherwise nothing.

**The script.** `rondata::script::city_gathered` is the marketed script (§16) and one package more: on
`CITY_GATHER_FRAME` 6718, the capital `0/2000`'s FARM button, `group(0, [2000])` and `city_gather(417, 2)`.

**Stage 0.** On 6718 at the opening camera, the capital stands at (3168, 30816), `z` 536, its anchor at pixel
(1127, 521). It is a centre, trains and is a `VILLAGE`, and it is finished. Its row holds GATHER FARM, WOOD, MINE and
ALL at centres (1258, 1054), (1446, 1054), (1634, 1054), (1822, 1054) at 1920×1080.

**The counts.**
- `INPUT_CITY_GATHERED`: **21 of 21**. The marketed events and two more, through the HUD and the layer (a press on
  the capital, then its GATHER FARM button), mint the city-gathered script field for field
  (`minted_city_gathered_at_the_opening_camera`). The file written from them is the city-gathered file. Seen red
  with the layer's `queued` 0: 21 orders, 20 whole.
- `INPUT_CITY_GATHERS`: **18 of 20** (20 of 20 since item 4267, §20.3). Every recorded city gather whose `group` names its building, that building
  pressed and the button of its type, mints the package's `group` and `city_gather` whole
  (`minted_city_gathers_equal_the_recordings`). The two with an empty group part: the panel's gate passes no empty
  selection, so the layer mints nothing. Seen red with `queued` 0: 0 of 20.
- `minted_city_gather_button_edges`: no row for another slot's building or one that is not a centre; each of the four
  types minted; nothing without a trainer, without a `VILLAGE`, with an empty scene, or with a unit selection.

**The file.** `~/ron-golden/recordings/item4261-city-gathered-greatlakes-24000.rcx`, 59,546 bytes, SHA-256
`a823ac470a26a14acb34a2b1e90497929deac3687a3419bccc2812b58c37763c`. It is the marketed file plus the one package,
stamped 22; the closing stamp goes from 22 to 23 (`the_city_gathered_minted_packages_round_trip`; seen red pinned 0).

### 17.3 The playback, and the order's effect

**Our walk under the city-gathered file against run1604** (`CITY_GATHERED_AGAINST_MARKETED`, pinned before run1605
was read): it parts first on 6719, the frame after the package, at the capital's `gather_len`, ours 4 and theirs 1.
281 of 300 records part, and the draws part from 6822.

**run1605** (`docs/RUNS.md`): run1604's line with this file. Draws agree on 7,000 of 7,000 frames, count and sequence;
300 records compared whole, none parted (`CITY_GATHERED_WALKED`).

**The order's effect, equal on both sides** (`city_gathered_effects`, pinned from our walk before run1605 was read;
the take's records read by `gather_effects_theirs_at`):

| frame | the capital's gather list | the Citizen `0/10` | without the order (our walk on the marketed file) |
| --- | --- | --- | --- |
| 6718 | (4655, 31075, 0) | not on the map | the same |
| 6719 | (2688, 32064, 1), (1920, 32064, 1), (2112, 31296, 1), (3456, 29760, 1) | not on the map | (4655, 31075, 0) |
| 6999 | the same four | (5572, 29304), `move`, `gather` | (4655, 31075, 0); the Citizen at (4632, 31080), `gather` |

The paused script's rally point (§13) is cleared, and the capital gains one point per Farm of its city, at the Farm's
position, action 1. **Made to fail:** the take's lists from 6719 differ from the walk without the order.

**The functions** (`CITY_GATHER_ENTRIES`, pinned before the read): `CommandPackage::process_city_gather` and
`Group::action_city_gather` are first entered on 6717, the tick that consumes 6718's package. run1604 enters
neither.

### 17.4 Confidence, and what is not established

**Backed by the playback.** run1605 plays a city gather minted from the capital's FARM button, and the capital's
gather list and the Citizen it sends are ours on the three frames read. The Farm arm of `action_city_gather`, the
clearing of the rally point and the points' order are all inside that.

**Reading only**, one reader, no blind second reading:
- the panel's gate (`flags & 0x20` and the preference bit) and `do_city_gather`'s gate;
- the Woodcutter, Mine and all arms (`t` 418, 419, −1); the golden harness's §162 carries −1.

**Ours, not the original's:**
- the row's place, size and labels; the city panel's interface is not read;
- the profile's preference bit is taken as set;
- `COUNT_GATHER_POINT` is read by its training arm alone, as §11.2;
- `Group::can_activate` is not read; the layer asks only that every member be a building of the scene;
- the buttons are no key's event.

**Not established:**
- ~~**The two recorded gathers with an empty group** (7205, 8179): how the original wrote a `group` of no objects
  behind a gate that counts the selection is not read. Parked 4261a.~~ Read in §18.1: `CommandPackage::add_group`
  writes no objects for a group equal to the last one sent. ~~The layer does not write that yet (4264a).~~ It
  does since item 4267 (§20), and both mint whole.
- **A selection of several buildings**: the panel is a single centre's here, and the layer mints the whole
  selection's group; no recording carries more than one.

## 18. A unit's craft button, and its pick (item 4264)

*Item 4264, 2026-11-15. The score is the census (§11.1): `INPUT_KINDS` 14 → 15 of 26 kinds, 2,306 → 2,323 of 2,429
orders. The HUD gains a unit's craft row; the layer mints `spell` as `Options::do_spell` and `picked_spot` issue it.
**No playback carries it**: slot 0 of the minted game holds no unit with a cast the original would make (§18.3). The
count is against the recordings, the original's own orders.*

### 18.1 What the button and the pick issue

**The button.** `Options::exec@007188c0`'s `OPTION_SPELL` is `Options::do_spell@0071d7a0(t, …)`. It returns unless the
selection can act (`Group::can_activate`) and holds a caster (`GroupData::get_spellcaster`); To Arms also needs the
leader's `has_tech(MILITIA)`, and every craft but Disband and Depopulate a valid caster
(`get_valid_spellcaster`). A craft whose `spell_flags & 0xe` is clear is issued at once:
`target_spell(t, −1, −1, 0, 0)`. A targeted one sets `selecting_spot` and the cursor.

**The pick.** `Options::picked_spot@00721c40`'s `OPTION_SPELL` arm calls `reset`, then `target_spell(t, −1, −1, x, y)`
for a ground craft (`& 8`), or `SpellTypeData::find_target@00674920`'s object, as `target_spell(t, ox, whom, x, y)`.
In both cases `x`, `y` is the pressed point. `Options::target_spell@0071dda0` packs a pack or unpack craft straight
away (`dbg_pack_or_unpack`). Otherwise it calls `GroupOut::validate_spell`, then
`CommandManager::issue_spell@00941b80`: the selection's `group` and command `0x17`, `spell(ox, whom, t, x, y)`.

**The right press.** `Console::execute_at_cursor@007c6630` calls `GroupOut::issue_spell` with the point
(0, 0) on a target: a Scout-line selection's Counterintelligence on an enemy Spy or its own infiltrated object, and
the other crafts beside it. It is not minted here.

**The recordings.** All 17 on disk are the 2017 sample's, slot 3's:
- six Forced March (`0x27c`), `ox`/`whom` −1 at (0, 0): the button's;
- eleven targeted ones (Bribe ×5, Informer on a building, Sabotage ×2, Sniper ×3), each at (0, 0). The pick writes
  the point, so these are the right press's.

Thirteen of the 17 carry a `group` of no objects. **`CommandPackage::add_group@0094bb60` writes that**: when the
group's `who`, its objects and their uids (`+0x30`) all equal the last group sent (`last_who_sent`,
`last_objects_sent`, `last_uids_sent`), it writes a count of zero. Otherwise it writes the list and keeps it. This
answers 4261a's question (§17.4).

### 18.2 What the layer does

- **The scene** (`crates/sim/src/observe.rs`, reads only): `UnitScene::crafts`, each craft row whose `FROM` line the
  unit is and that `Sim::spell_castable` passes, with `targeted` (`& 0xe`) and `ground` (`& 8`).
- **The HUD** (`hud::spell_menu`, `docs/HUD.md` §1): with a unit selection, one button per craft a member offers, in
  row order. A press on one is `Event::Spell(t)`.
- **The layer**: `Event::Spell(t)` with a unit selection one of whose members offers `t` (`input::selection_craft`).
  An untargeted craft gives the selection's `group` and `spell(−1, −1, t, 0, 0)`. A targeted one sets
  `Input::casting`; the next left press gives the `group` and `spell(ox, whom, t, x, y)`, where the object is the
  one a right press would name (`View::target`) and `x`, `y` is the ground under the press. A ground craft gives
  `spell(−1, −1, t, x, y)`. A pick on no object mints nothing and spends the pick; a right press drops it.

**The counts.**
- `INPUT_SPELLS`: **1 of 6** (6 of 6 since item 4267, §20.3). Every recorded button spell, minted from its group's units offering the craft and a
  press on its button (`minted_spells_equal_the_recordings`). The five with an empty group part, because the layer
  does not write `add_group`'s repeat (parked 4264a). Seen red with the arm's point (1, 1): 0 of 6.
- `minted_spell_button_edges`: the row, each craft once; the untargeted spell, also through the HUD; the targeted
  craft's pick on another slot's unit; a ground craft's point; nothing for a craft no member offers, for a building
  selection, for another slot's unit, or for a pick on no object; a right press dropping the pick.

### 18.3 Why no playback, and what is not established

**Slot 0 of the minted game cannot cast anything.** On our walk of the city-gathered file, slot 0 holds six Citizens,
a Scout and three Slingers from 6720 to 12,000, and the same 18 techs. The Citizens' To Arms needs `has_tech(MILITIA)`,
which slot 0 lacks: no slot-0 building researches it, and `do_spell` refuses the button. The Scout's
Counterintelligence needs an enemy Spy, or an infiltrated object of its own or an ally's
(`Sim::counterintel_valid_target`), and the game has neither. So the original's button or pick would issue nothing
the original then casts. A file carrying such an order would test nothing. The probe was a scratch test on this
branch and was not kept.

**Reading only**, one reader, no blind second reading:
- `do_spell`'s gates, `picked_spot`'s arm and `target_spell`;
- `add_group`'s repeat.

**Ours, not the original's:**
- the row's place, size and labels; `UnitOut::make_options` is not read beyond the craft's `FROM` line;
- `get_spellcaster`, `get_valid_spellcaster` and `validate_spell` are stood in for by `Sim::spell_castable` on one
  member;
- the pick's object is `View::target`'s, not `SpellTypeData::find_target`'s;
- the pack and unpack crafts' `dbg_pack_or_unpack` is not taken: the layer would mint a spell for them.

**Not established:**
- **A spell the original plays from our file.** The minted game's slot 0 needs a caster first (parked 4264b).
- **The right press's crafts** (11 recordings), `execute_at_cursor`'s Scout-line and Commando arms (parked 4264c).
- ~~**`add_group`'s repeat in the layer** (parked 4264a): the five button spells and 4261a's two city gathers would mint
  whole with it. Every script whose groups repeat back to back would change bytes, so it moves the layer's pins.~~
  Built in item 4267 (§20): the five mint whole, and the scripts are re-pinned.


## 19. A building's queue button: the cancel (item 4265)

*Item 4265, 2026-11-15. The score is the census (§11.1): `INPUT_KINDS` 15 → 16 of 26 kinds, 2,323 → 2,336 of 2,429
orders. The HUD gains a building's queue row. The layer mints `unqueue` as the queue button issues it. The original
plays the file (run1606) with every draw and every record in lockstep, and the capital's queue is equal on both sides.*

### 19.1 What the button issues, and what the order does

**The button.** `IFaceSelected::on_button_clicked@00826e00` turns a press on a queue entry's button into
`Options::exec(OPTION_QUEUE, slot)`. `OPTION_QUEUE` is 0xa6 (`docs/AI.md`'s row). In `Options::exec@007188c0`, the
0xa6 arm asks `Group::can_activate` of the selection, plays `S_UNQUEUE_BUILDING`, then walks the selection. For each
finished building (`ObjectData::is_active_build`) it calls `BuildOut::issue_unqueue@00629e70(slot)`: command `0x30`,
`[who][o][slot][uid]`, the building's `+0x9`, `+0xa` and `+0x30`, 15 bytes. **No `group` is written.** A unit member
takes `UnitOut::issue_unqueue`, or `GroupOut::issue_stop_spell` with nothing queued. That arm is not minted here. With
no object (`slot` < 0), Shift or Ctrl turn the slot into the selector's negatives. Those are not minted either.

**The recordings agree, 13 of 13.** All 13 are the 2017 sample's, slot 3's. None carries a `group`. Twelve name the
capital `3/2000` with `uid` 0: slot 2 once, slot 0 eleven times. One names `3/2019`, `uid` 58, slot 1.

**What it does.** `CommandPackage::process_unqueue@009466f0` calls `Build::action_unqueue@00620280(slot)` on the
named building. That was already ported (`Sim::action_unqueue`, item 884, `docs/GOLDEN.md` §34), and our walk's
`Stream` already carried it (`rondata::input::unqueue`).

### 19.2 What the layer does

- **The scene** (`crates/sim/src/observe.rs`, reads only): `BuildingScene::uid` and `BuildingScene::queued`.
- **The HUD** (`hud::queue_menu`, `docs/HUD.md` §1): with one finished building of the slot's own selected
  (`input::queue_panel`), one CANCEL button per live queue entry. A press on one is `Event::Unqueue(slot)`.
- **The layer** (`input::unqueues`): with a building selection whose every member is the slot's own building in the
  scene, one `unqueue(who, o, slot, uid)` per finished member, in a package with no group. It mints nothing when no
  member is finished.

**The script.** `rondata::script::unqueued` is the city-gathered script (§17) and one package more. On `UNQUEUE_FRAME`
6720 the capital, still selected from the gather and still training the gathered script's Citizen, has its first
queue button pressed: `unqueue(0, 2000, 0, 0)`.

**Stage 0.** On 6720 the capital is finished, `queued` 1, `uid` 0. Its queue row holds CANCEL 1 centred at
(1822, 1010) at 1920×1080.

**The counts.**
- `INPUT_UNQUEUED`: **22 of 22**. The city-gathered events and one press more, through the HUD and the layer, mint
  the unqueued script field for field (`minted_unqueued_at_the_opening_camera`). The file written from them is the
  unqueued file. Stage 0's button was seen red pinned (0, 0).
- `INPUT_UNQUEUES`: **13 of 13**. Each recording's building is selected with its `uid` and a queue long enough for
  its slot, then its button is pressed. That mints the package whole (`minted_unqueues_equal_the_recordings`). Seen
  red pinned (0, 13).
- `minted_unqueue_button_edges`: a row per live entry, and none for an empty queue, an unfinished building or a
  selection of two buildings. A press through the HUD gives its slot. Two finished buildings give one cancel each,
  in the selection's order, and an unfinished one gives none. Another slot's building, a unit selection and an empty
  scene give nothing.

**The file.** `~/ron-golden/recordings/item4265-unqueued-greatlakes-24000.rcx`, 59,559 bytes, SHA-256
`0301987ecc3b5fe1d941a4cbedf7cf3b89598475f6a9b445b2b0e8ab9553e27b`. It is the city-gathered file plus the one package,
stamped 23; the closing stamp goes from 23 to 24 (`the_unqueued_minted_packages_round_trip`; seen red pinned 0).

### 19.3 The playback, and the order's effect

**Our walk under the unqueued file against run1605** (`UNQUEUED_AGAINST_CITY_GATHERED`, pinned before run1606 was
read): it parts first on 6721, the frame after the package, at the capital's `queued`, ours 0 and theirs 1. 279 of
300 records part, and the draws part from 6813, the frame run1605's Citizen leaves the capital.

**run1606** (`docs/RUNS.md`): run1605's line with this file. Draws agree on 7,000 of 7,000 frames, count and sequence;
300 records compared whole, none parted (`UNQUEUED_WALKED`).

**The order's effect, equal on both sides** (`unqueued_effects`, pinned from our walk before run1606 was read; the
take's records read by `queue_effects_theirs`):

| frame | the capital's live queue (`job_counter`) | the Citizen `0/10` | without the order (our walk on the city-gathered file) |
| --- | --- | --- | --- |
| 6720 | 1200 | not on the map | the same |
| 6721 | empty | not on the map | 1300 |
| 6999 | empty | not on the map | empty; the Citizen at (5572, 29304) |

**Made to fail:** the take's values from 6721 differ from the walk without the order. The effect test was seen red
pinned empty.

**The functions** (`UNQUEUE_ENTRIES`, pinned before the read): `CommandPackage::process_unqueue` and
`Build::action_unqueue` are first entered on 6719, the tick that consumes 6720's package. run1605 enters neither.

### 19.4 Confidence, and what is not established

**Backed by the playback.** run1606 plays a cancel minted from the capital's first queue button, on the entry in
training. The capital's queue and the Citizen it no longer trains are ours on the three frames read.

**Reading only**, one reader, no blind second reading:
- the 0xa6 arm's walk of the selection and its finished-building test;
- that `IFaceSelected`'s queue button passes its slot as the option's object.

**Ours, not the original's:**
- the row's place, size and labels; `IFaceSelected`'s panel is not read beyond the click;
- the row is drawn for one building only, though the arm walks a selection of several;
- `Group::can_activate` is not read; the layer asks that every member be the slot's own building in the scene;
- the buttons are no key's event.

**Not established:**
- **A cancel of an entry waiting behind the one in training**: slot 1 or higher, which one recording carries. The
  layer mints it, as the edges show, but no playback has carried one.
- **The selector's negatives** (Shift, Ctrl, no object) and **a unit's cancel** (`UnitOut::issue_unqueue`, a
  carrier's plane): not minted.
- **A refund** is inside the playback's records (300 whole) but is not read as a value of its own.

## 20. The repeated group: a group equal to the last one sent (item 4267)

*Item 4267, 2026-11-15. The score: `INPUT_SPELLS` 1 → 6 of 6 and `INPUT_CITY_GATHERS` 18 → 20 of 20. The census
(§11.1) does not move: no new kind is minted. The layer now writes a `group` the way `CommandPackage::add_group`
does. A group equal to the last one sent goes as no objects. The scripts from the gathered script on are re-pinned
toward the original: each carries the original's empty group where it selects the capital again. The original plays
the re-pinned unqueued file (run1607).*

### 20.1 What `add_group` writes

`CommandPackage::add_group@0094bb60` builds the group's list from the `GroupOut`'s objects. Each live object
(`flags & 1`) goes in with its uid (`ObjectData +0x30`). It then compares `who`, the count, the objects and the uids
against three statics: `last_who_sent`, `last_num_sent`, `last_objects_sent` and `last_uids_sent`. **When all are
equal it writes `[0][0][who]`, a group of no objects, and leaves the statics alone.** Otherwise it keeps the new list
in the statics and writes `[0][count][who]` and the objects. `CommandPackage::init_class@0094c440` sets `last_who_sent`
to −1, the count to 0 and both lists to −1 when a game starts. Nothing else writes them. The state belongs to the
sending machine, not to a package: a package with no group (a pause, a sale, a cancel) leaves it standing.

The receiver is already ported. `CommandPackage::process_group@0094a0c0`'s `num = 0` arm re-adds the last list it
recorded for that player, each object still alive with the uid it had (`Sim::process_group_play_repeat`, item
4113). The walk's `Stream` keeps the previous selection for an empty group.

### 20.2 What the layer does

- **The scene** (`crates/sim/src/observe.rs`, reads only): `UnitScene::uid`, `Unit::uid`. `BuildingScene::uid` was
  item 4265's.
- **The layer** (`crates/client/src/input.rs`): `input::Sent` holds the statics, and its `Default` is `init_class`'s.
  `Sent::add_group` makes the write. `Input::sent` is the session's state, and `Input::group` now takes the scene: each
  selected object goes with the uid the scene holds for it. Every `group` the layer mints goes through it.
- **The scripts** (`crates/rondata/src/script.rs`): `repeated()`, slot 0's group of no objects, stands in three places.
  These are the gathered script's Citizen queue on 6708, after the rally point's `group(0, [2000])` on 6706; the
  paused and rallied scripts' rally point on 6712; and the city-gathered script's gather on 6718. Between each and the
  capital's last group lie only packages with no group (the chat, the pauses, the sale and purchase). The followed
  script's Scout groups on 6702 and 6704 do not repeat, because the Slinger's attack on 6703 lies between them.
  `script::full_groups` puts each empty group back in full: the last list of the same `who`, which `process_group`'s
  `num = 0` arm repeats. It is the form the files before this item carried.

**Stage 0.** The scripts assume the capital's uid does not change between its groups; the layer reads it off the
walk's scene. The opening-camera tests (`minted_gathered_at_the_opening_camera` to
`minted_unqueued_at_the_opening_camera`) mint the re-pinned scripts field for field with the layer's own uids:
`INPUT_GATHERED` 14, `INPUT_CHATTED` 15, `INPUT_PAUSED` 18, `INPUT_MARKETED` 20, `INPUT_CITY_GATHERED` 21 and
`INPUT_UNQUEUED` 22, all unchanged. On the first run after the layer changed, they parted only at the bytes of the
played files, which `full_groups` now writes.

### 20.3 The counts

- `INPUT_SPELLS`: **6 of 6** (from 1). The test now walks each recording's packages in order and keeps, for each
  slot, the last `group` it carried in full. A recorded group of no objects is minted with the selection that list
  names, and with the layer's `Sent` set to it. The five that parted mint whole. Red at the old pin: (6, 6) against
  (1, 6).
- `INPUT_CITY_GATHERS`: **20 of 20** (from 18), the same way: 7205's and 8179's empty groups follow the same city
  building's full group. Red at the old pin: (20, 20) against (18, 20).
- `minted_spell_button_edges`: the craft pressed twice on one selection, through the layer and then through the HUD,
  now writes the second `group` as no objects.
- **The files the original played** (run1598 to run1606) stay pinned. The round trips and the opening-camera byte
  tests write them from `full_groups` of the re-pinned scripts. They are byte for byte the gathered, chatted, paused,
  rallied, marketed, city-gathered and unqueued files.

**The file.** `~/ron-golden/recordings/item4267-repeated-greatlakes-24000.rcx`, 59,564 bytes, SHA-256
`1efd3a672cccf3957d4a34c7daaed9196e1eae7fae709407562ae5f830d14b3c`
(`the_repeated_minted_packages_round_trip`; seen red pinned 0). It holds the unqueued file's packages, each with its
frame, slot and stamp. Its groups on 6708, 6712 and 6718 carry no objects, and on no other frame
(`REPEATED_FRAMES`). Each repeat drops one object's two bytes. The gzip grows by five bytes, from 59,559. With the
groups put back in full, it is the file run1606 played. The client's events through the HUD write it byte for byte
(`minted_unqueued_at_the_opening_camera`).

### 20.4 The playback

**run1607** (`docs/RUNS.md`) is run1606's line with the repeated file. The receipt names the file's SHA-256. Draws
agree on 7,000 of 7,000 frames, count and sequence, and 300 records are compared whole, with none parted
(`REPEATED_WALKED`, pinned before the take was read; `the_original_s_playback_of_the_repeated_file_walks_in_lockstep`).
The window, 6700 to 6999, holds all three repeats.

**Each order's effect, against the previous playback.** The capital's queue and the Citizen `0/10`, read off
run1607's records, are run1606's under the full groups and our walk's (`unqueued_effects`):

| frame | the capital's queue | the Citizen `0/10` | run1606 (full groups) |
| --- | --- | --- | --- |
| 6720 | [1200] | not on the map | the same |
| 6721 | [] | not on the map | the same |
| 6999 | [] | not on the map | the same |

Each repeat's own effect is inside the 300 records. The gathered Citizen is queued on 6708, the rally point moves on
6712 and the capital's gather list is rebuilt on 6718, each on the capital. Our walk is whole against run1606 and
against run1607, so the two takes' windows are equal record for record. The gamelogs are the same length,
67,852,152 bytes. **Made to fail:** the walked table and the effects were pinned before the read. A receiver that
read an empty group as no selection would leave the Citizen unqueued and the rally point unmoved, which the capital's
records in the window would show. No such take was made.

### 20.5 Confidence, and what is not established

**Backed by the recordings and the playback.** The recordings' 7 empty groups mint whole once the layer keeps the
sender's last group. run1607 plays the three repeats from our file in lockstep, with the window record for record run1606's.

**Reading only**, one reader, no blind second reading:
- that `init_class` is the statics' only other writer (one grep over the export: `add_group` and `init_class` alone
  name `last_objects_sent`);
- the live test (`flags & 1`) and the captain branch of the list's build. The layer writes every selected object;
  §20.2's scripts select nothing dead.

**Ours, not the original's:**
- an object the scene does not hold goes with uid `u16::MAX`, where `add_group` drops a dead object from the list;
- the recording tests keep the statics per package slot. The original keeps one set per machine, and an AI's
  packages, if a host sent them, would share it. No recording on disk is read for that;
- the recordings carry no uids, so the tests give each object the scene's uid. They take an empty group to mean the
  sender's last full group with the uids unchanged, which is what `add_group`'s equality says.

**Not established:**
- **A group that repeats across a selection made again**: a press that selects the same object a second time sends
  no group until an order. The layer keeps `Sent` across a selection change, as `add_group` does, but only the
  capital's case is played.
- **The `Random::get` padding** that `add_group` adds when the game's semaphore bit 4 is set is not written. The
  layer's files never carry it, and no file the original played needed it.

## 21. A city's name, from its panel (item 4268)

*Item 4268, 2026-11-15. The score: `INPUT_KINDS` 16 → 17 of 26, and 2,336 → 2,346 of 2,429 orders (96.1 % → 96.5 %).
The census's next kind by share whose actor slot 0 of the minted game holds is `rename_city` (10): its actor is a
city, and slot 0's capital `0/2000` is one. `leader_options` (50) is the lobby's. The layer mints `rename_city` as
the city's name popup sends it. The renamed script mints 23 of 23 and the recorded renames 10 of 10. The original
plays the file (run1609).*

### 21.1 What the panel issues, and what the order does

- **The option**: `Options::exec@007188c0`'s option `0x117` takes the selection's first object
  (`GroupData::get_o`). It returns unless that object is a building (`ObjectData +0x8 & 0x20`), passes an
  `ObjectData::is` test, is finished (the type's `is_active`) and belongs to a city (`BuildData +0x72 >= 0`). Then it
  opens a `Popup` with `popup_id` 6 and `extra_data` the object, whose edit box holds the city's name (`City +0x90`)
  with a limit of `0x15` units.
- **The popup**: `Popup::on_modal_end@007c13f0`'s case 6, on OK, trims the spaces at both ends of the box's text. A
  line left empty sends nothing. A multiplayer game (semaphore bit 4) first shows "Renaming city…". Then it calls
  `CommandManager::issue_rename_city@009411d0(console who, o, text)`.
- **The command**: byte `0x4b`, `[who][o][22 units]`, `0x35` bytes, in a package of its own with no group. A name
  of more than `0x13` units is cut to `0x14`, and a NUL is written after it. The issuer copies its whole 22-unit
  stack buffer, so past the NUL ride whatever bytes the stack held (`Command::RenameCity::after_nul`; `RECGAME.md`).
- **The receiver**: `CommandPackage::process_rename_city@00944090` takes `objects[who][o]`. When its flag `0x20`
  holds, it assigns the name to the `City +0x90` of the city its `BuildData +0x72` names. It then asks
  `HotKeyGroups::find_group` for a group holding the object, to redraw its name. The receiver reads the name up to
  its NUL and never reads the stack bytes after it.
- **The dump**: `CityData::log_data@004895c0` writes the name bare, after `who`, in the `CITY` record (`CITIES=5`).
  The block parser keeps it as a key and the rest of the line as the value. `CityDump` does not carry it (§21.4).

### 21.2 What the layer does

- **The sim** (`crates/sim/src/cities/rename.rs`): `Sim::process_rename_city`, through the command door
  (`Command::process_rename_city`), writes `CitiesState::names` for the building's city. `Sim::city_name_written`
  reads it. The founding name (`City::generate_name`, the tribe's list) is not carried, so a city reads `None` until
  a rename writes it. **This touches `crates/sim`.** `BuildingScene::in_city` is a read.
- **The walk** (`crates/rondata/src/input.rs`): `RenameCity` is `process_rename_city` on the order's `who` and `o`.
  It was "not mapped" before.
- **The layer** (`crates/client/src/input.rs`): `Event::Rename`, the HUD's press. `input::renames` is the option's
  test. `Input::renaming` makes the entry line the popup's edit box, with `RENAME_LIMIT` (`0x15`) units. Return sends
  `input::rename`: trimmed, empty sends nothing, cut to `RENAME_SENT` (`0x14`) units, `after_nul` empty. Escape
  closes it with nothing sent. `rename_city` is added to `MINTS`.
- **The HUD** (`crates/client/src/hud.rs`): `rename_button`, one RENAME button three rows above the bottom edge at
  the right; `hud::event` routes a press on it; `paint` draws it.
- **The script** (`crates/rondata/src/script.rs`): `renamed()`, the unqueued script and `rename_city(0, 2000,
  "Attrition")` on `RENAME_FRAME` 6730 with no group.

**Stage 0.** On 6730 the capital is finished and in a city. Its RENAME button stands at (1822, 966) under the HUD's
1920 × 1080 (`RENAME_AT`; seen red pinned (0, 0)).

### 21.3 The counts and the file

- `INPUT_RENAMED`: **23 of 23**. The unqueued events, then a press on RENAME, `Attrition` typed and Return, all on
  6730, through the HUD and the layer under the opening camera, mint `script::renamed()`
  (`minted_renamed_at_the_opening_camera`).
- `INPUT_RENAMES`: **10 of 10** (seen red pinned (0, 0)). All ten are in two recordings: `17'27'47` renames `0/2010`
  Norwich and `0/2000` London on 8516 to 8528, and `20'17'33` renames `0/2000` Napata on 14401 to 15153. None carries
  a group. Each is minted from its building, RENAME, its name and Return. **All ten carry stack bytes past the
  terminator.** The count compares every field but those, and strictly it is 0 of 10. The receiver never reads them.
- `minted_rename_edges`: the spaces trimmed, 22 typed units kept as 21 and sent as 20, a blank line sends nothing,
  Escape. No box opens for another slot's building, an unfinished one, or one in no city.
- `INPUT_KINDS` (17, 26), `INPUT_KINDS_ORDERS` (2,346, 2,429); `NOT_MINTED` drops `rename_city`.

**The file.** `~/ron-golden/recordings/item4268-renamed-greatlakes-24000.rcx`, 59,591 bytes, SHA-256
`3bea7e5cd23969ae34ba40d28b428abeb22384574c96aed7b70ecc40269067e4` (`the_renamed_minted_packages_round_trip`;
seen red pinned 0). It is the repeated file's packages, each with its frame, slot and stamp, then the rename on 6730,
stamped 24. The closing stamp goes from 24 to 25. The client's events through the HUD write it byte for byte.

### 21.4 The playback, and the order's effect

**run1609** (`docs/RUNS.md`) is run1607's line with the renamed file and `CITIES=5` added to the window's detail, so
the `CITY` record prints the name. The receipt names the file's SHA-256. Draws agree on 7,000 of 7,000 frames, count
and sequence, and 300 records are compared whole, with none parted (`RENAMED_WALKED`, pinned before the take was
read; `the_original_s_playback_of_the_renamed_file_walks_in_lockstep`). run1607's take could not have told the files
apart: it prints no `CITY` record, and the order moves nothing the record diff reads.

**The order's effect.** The capital's name, slot 0's `CITY` record with `o` 2000, its bare line after `who`
(`name_effects_theirs`), against our walk (`name_effects_ours`). Both were pinned before the take was read. Ours was
seen red pinned empty:

| frame | run1609 | ours | ours without the order (the repeated file) |
| --- | --- | --- | --- |
| 6730 | Napata | none (the founding name is not carried) | none |
| 6731 | Attrition | Attrition | none |
| 6999 | Attrition | Attrition | none |

`Napata` is the same lobby's capital in run1579's `CITY` record. From 6731 on the two sides are equal, and the order
is what moves ours (`the_renamed_minted_script_s_effects_on_our_walk`).

### 21.5 Confidence, and what is not established

The option's tests, the popup's trim and the issuer's cut are read from the decompile, one reading each. The playback
backs the command's bytes and the receiver's write (§21.4).

- **The founding name.** The sim does not carry `City::generate_name`'s list, so our side has no name until a rename.
  The effect agrees from the frame after the package.
- **`CityDump` does not carry the name.** The effect test reads the bare line itself. Adding it to the reader
  would change the oracle cache's generation for every lane, and only this test reads it.
- **The arm's `ObjectData::is` test** (`vtable +0xb8`) is not read by the layer. Neither is `Group::can_activate`;
  the layer asks for the slot's own building instead.
- **A rename from a building that is not the centre.** Any finished building in a city opens the popup. Only the
  capital is played.
- **The box's starting text.** The original's box opens on the city's current name; ours opens empty (below).

**Ours, not the original's:**

- The RENAME button and where it stands. The original opens the popup from the city's name on the selected panel.
- The box opens empty, because the scene does not carry the founding name.
- A name cut inside a surrogate pair keeps the pair whole or drops it.

## 22. A hot key: ctrl and a key, over a selection (item 4270)

*Item 4270, 2026-11-15. The score: `INPUT_KINDS` 17 → 18 of 26, and 2,346 → 2,354 of 2,429 orders (96.5 % → 96.9 %).
The census's next kind by share is `hotkey` (8). What it names is one of a player's hot-key groups, `who × 18` plus
the key, and its contents, the package's selection. Slot 0 of the minted game holds both: its eighteen hot-key groups
are 0 to 17 of the original's 162 (run1562's whole dump on frame 91 prints them, group 10, F1, already holding the
capital `0/2000`), and the capital it selects. The layer mints the selection form, the only one the recordings
carry. The hot-keyed script mints 24 of 24 and the recorded hot keys 8 of 8. The original plays the file (run1610).*

### 22.1 What the key issues, and what the order does

- **The key**: `Console::on_key_down@007cc210` takes `0`..`9` as 0 to 9 and F1..F8 (`0x70`..`0x77`) as 10 to 17, after
  the key map and the nation hot keys (alt) have declined it. A computer leader's console (`leader_flags & 4` clear)
  selects an army instead. With ctrl (`0x20000`) and no shift (`0x10000`), and a selection, it calls
  `GroupOut::issue_hotkey@00708800(who × 0x12 + key)`; with an empty selection it adds a screen group itself (`clear`
  1, `valid` 1, the camera's point and zoom). With ctrl and shift it is `Group::action_hotkey`, which issues nothing.
- **The issuer**: `GroupOut::issue_hotkey` issues nothing in a playback (`semaphore & 0x10`). Otherwise
  `CommandManager::issue_hotkey@00941fd0(group, key, 0)` writes the package's group (`add_group`) and the command:
  byte `0x22`, `[group][clear][valid][x f32][y f32][zoom]`, `0x19` bytes. With a group, `clear` is 0; with the camera
  flag clear, `valid` is 0, `x` and `y` are −1.0 and `zoom` 0.
- **The receiver**: `CommandPackage::process_hotkey@009474d0`, with `clear` 0 and a package group, runs
  `HotKeyGroups::copy_group@00715120`: the package's pool group (the slot `process_group@0094a0c0`'s
  `Groups::push_group(…, 1)` handed back) is copied into the hot-key group. The fields copied are `who`, `num`, `ox`,
  `oy`, `o_dist`, `o_angle`, `buildings`, `speed` (`+0x40`) and the six per-member arrays, and `stamp` becomes the
  frame. Then `valid` (`+0x9d8`) is 0 and the slot's name is redrawn for the console's own. With `clear` 1 it empties
  the slot (`num` 0, `valid` 0) and, with `valid`, writes the camera's point and zoom.
- **The array**: `HotKeyGroups`, `0xa2` groups, apart from the pool. `HotKeyGroups::clear@00715230` gives each its
  index as `id`, `priority` 1, `army` and `form` −1. A whole dump (`@dumpall`) prints it under `HOTKEYGROUPS`,
  one `HOTKEYGROUPDATA` each with its `GROUPDATA` nested, then `loc_x`, `loc_y` and `valid` (run1562, run1610). Which
  writer prints it there is not read: run1610's trace does not enter `GameLog::dump_hotkey_groups`. No checksum line
  reads the array: `check_groups` walks the pool.

### 22.2 What the layer does

- **The sim** (`crates/sim/src/groups/hotkey.rs`): `Sim::process_hotkey`, through the command door
  (`Command::process_hotkey`), copies the pool slot the player pushed last (`Groups::last_group`) into
  `OrdersState::hotkeys` at the command's index, as `copy_group` does, with `stamp` the frame and `valid` 0; `clear` 1
  empties it. `Sim::hotkey_slot` reads it. **This touches `crates/sim`.**
- **The walk** (`crates/rondata/src/input.rs`): `Hotkey` pushes the package's group of buildings
  (`push_command_buildings`, as `process_group` does) and runs `process_hotkey`. A selection of units is skipped,
  named. It was "not mapped" before.
- **The layer** (`crates/client/src/input.rs`): `Event::Hotkey(key)`. With a selection and no entry line open, it
  sends the selection's `group` and `hotkey(who × HOTKEY_SLOTS + key, 0, 0, HOTKEY_NO_CAMERA, HOTKEY_NO_CAMERA, 0)`.
  `HOTKEY_SLOTS` is `0x12`; `HOTKEY_NO_CAMERA` is −1.0f's bits, a constant, so no float is computed. `hotkey` is added
  to `MINTS`.
- **The window** (`crates/client/src/main.rs`): ctrl and `0`..`9`, or F1..F8, send `Event::Hotkey`.
- **The script** (`crates/rondata/src/script.rs`): `hotkeyed()`, the renamed script and, on `HOTKEY_FRAME` 6740, the
  capital's group (repeating the last one sent, so no objects) and `hotkey(1, 0, 0, −1.0, −1.0, 0)`.

**Stage 0.** A key has no pixel. On 6740 the capital, the selection the key copies, is slot 0's and standing
(`minted_hotkeyed_at_the_opening_camera`), and hot-key group 1 is empty on our walk (§22.4).

### 22.3 The counts and the file

- `INPUT_HOTKEYED`: **24 of 24**. The renamed events, then ctrl and `1` on 6740, through the HUD and the layer under the
  opening camera, mint `script::hotkeyed()`.
- `INPUT_HOTKEYS`: **8 of 8** (seen red pinned (0, 0)), on every field, the `group` before each included. All eight
  are in the 2017 sample, slot 3's groups 55, 57, 58 and 59 (keys 1, 3, 4 and 5) on 973 to 12303; each carries a
  group in full, of buildings or of units, `clear` 0, `valid` 0, −1.0 and −1.0, `zoom` 0. Each is minted from a fresh
  layer whose selection is the package's group, and ctrl with `group − 3 × 18`.
- `minted_hotkey_edges`: no selection sends nothing; a key past F8 sends nothing; slot 1's F8 is group 35; a key
  while the entry line is open sends nothing.
- `INPUT_KINDS` (18, 26), `INPUT_KINDS_ORDERS` (2,354, 2,429); `NOT_MINTED` drops `hotkey`.

**The file.** `~/ron-golden/recordings/item4270-hotkeyed-greatlakes-24000.rcx`, 59,605 bytes, SHA-256
`69906463b386719fc9ff0144d75b5c589d4b45c79a90ab8c657f952b3fa0d65c` (`the_hotkeyed_minted_packages_round_trip`; seen
red pinned 0). It is the renamed file's packages, each with its frame, slot and stamp, then the hot key's package on
6740, stamped 25. The closing stamp goes from 25 to 26. The client's events through the HUD write it byte for byte.

### 22.4 The playback, and the order's effect

**run1610** (`docs/RUNS.md`) is run1609's line with the hot-keyed file, `CITIES=5` taken out, and whole dumps
(`@dumpall`) on 6740 and 6741. The receipt names the file's SHA-256. Draws agree on 7,000 of 7,000 frames, count and
sequence, and 300 records are compared whole, with none parted (`HOTKEYED_WALKED`, pinned before the take was read;
`the_original_s_playback_of_the_hotkeyed_file_walks_in_lockstep`).

**Where a whole dump falls.** A `@dumpall` on frame N is written inside the take's `FRAME N` block after that frame's
packages ran: block 6740 already holds the slot the hot key wrote, stamped 6740. So it is our walk's state at N + 1
(`DUMPALL_LAGS`), where the end-frame records of the same block are its state at N (§21.4's `CITY` on 6730 still
prints the founding name). run1610 therefore held no dump from before the package, and **run1611** plays the same file
to 6745 with one whole dump in block 6739.

**The order's effect.** Slot 0's hot-key group 1, the `HOTKEYGROUPDATA` record's every field (`hotkey_effects_theirs`),
against our walk's slot (`hotkey_effects_ours`). Both were pinned before either take was read. Ours was seen red
pinned empty, and the reader's first form red on the wrapper's fields read twice:

| our frame | the take's block | the original | ours | ours without the order (the renamed file) |
| --- | --- | --- | --- | --- |
| 6740 | run1611 6739 | `num 0`, `stamp 0`, no member | the same | the same |
| 6741 | run1610 6740 | `who 0`, `num 1`, `buildings 1`, `stamp 6740`, `list[0] 2000`, offsets, positions and angle 0 | the same | `num 0`, `stamp 0` |
| 6742 | run1610 6741 | the same as 6741 | the same | `num 0`, `stamp 0` |

On every row the fields `copy_group` does not write read as `HotKeyGroups::clear` left them (`army` and `form` −1,
`priority` 1, the rest 0), with `loc_x`/`loc_y` 0.0 and `valid` 0. The order is what moves ours
(`the_hotkeyed_minted_script_s_effects_on_our_walk`).

### 22.5 Confidence, and what is not established

The key's arms, the issuer and the receiver are read from the decompile, one reading each; `copy_group`'s field list
is read off its offsets against `GroupData`'s type record. The playback backs the command's bytes and the copy into
the slot the command names (§22.4).

- **A selection of units.** The walk pushes a group of buildings alone. A unit group's pool record (its formation
  offsets) is what `copy_group` copies; no hot key on units is played.
- **The slot's other writers** are not carried: `HotKeyGroups::kill`, `copy` (a unit coming out), `find_group` (a
  rename, `docs/GROUPS.md`), and whatever puts the capital in F1 before the first frame. A slot no hot key wrote reads
  as `HotKeyGroups::clear` left it.
- **The screen group** (ctrl and a key with nothing selected) is not minted: its `x` and `y` are the camera's floats.
- **The ctrl-and-shift arm** (`Group::action_hotkey`) and the plain key's recall of a group are the interface's and
  issue nothing; they are not built.

**Ours, not the original's:**

- The layer reads ctrl from the window's modifiers; the key map's own bindings are not consulted.
- Ctrl and F3 is a hot key; F3 alone is the pause (§13).

## 23. A garrison: the unit's button, and its pick (item 4273)

*Item 4273, 2026-11-15. The score: `INPUT_KINDS` 18 → 19 of 26, and 2,354 → 2,360 of 2,429 orders (96.9 % → 97.1 %).
The census's next kind by share is `garrison` (6). All six recorded orders have the same form: the selection's
`group`, then `garrison(ox, whom, 2)` into one of the slot's own buildings. Five are the 2017 sample's (slot 3, into
2037, 2045 and 2047, two of them with an empty group) and one is in `20'17'33` (slot 0, a Citizen into `0/2000`).
Slot 0 of the minted game holds both the actor and the target. On 6750 its six Citizens each pass the type test into
the capital `0/2000`, which is finished, has limit 10 and holds nobody. The layer mints the button-and-pick form. The
garrisoned script mints 25 of 25, and the recorded garrisons mint 6 of 6. The original plays the file (run1613).*

### 23.1 What the button and the pick issue

- **Two issuers write the same bytes.** `GroupOut::issue_garrison@0070a9b0(ox, whom, queue)` writes the package's
  group (`add_group`), then command `0x14`, `[ox][whom][queue]`, `0xd` bytes. In a playback it issues nothing
  (`semaphore & 0x10`). `CommandManager::issue_garrison@00941a70` is the same without that test.
- **The right press.** `Console::execute_at_cursor@007c6630` issues a garrison with `QUEUE_NEW` on one of the slot's
  own buildings, when the building's type has a garrison limit and `GroupData::test_garrison` passes. It does so only
  with the profile's `0x200` set, the advanced option (§6), so the layer has no right-press arm.
- **The button.** `UnitOut::make_options@00615fc0` adds `OPTION_GARRISON` to a unit's panel by its type
  (`ptype +0x218`, `unit_flags & 0x1800` and three type tests). `Options::exec@007188c0` calls
  `Options::do_garrison@0071d6b0`. With a selection that can act (`Group::can_activate`) and is not of buildings, it
  sets `selecting_spot` and the cursor. With its argument set, which is the key path, it picks at the cursor at once.
- **The pick.** `Options::picked_spot@00721c40`'s `OPTION_GARRISON` arm looks for an allied building under the
  pick, as `find_target(SEARCH_ALLIED, 2, FILTER_ALL)`. If it finds none it returns before `reset`, so the pick
  stands. Otherwise it calls `reset`, asks the building for a nonzero garrison limit
  (`BuildTypeData::get_garrison_limit`), and on `GroupData::test_garrison@0070b320` issues
  `issue_garrison(ox, whom, QUEUE_NEW)`. Failing the limit, it moves on to a second arm, for a selection holding type
  `0x136`.
- **`test_garrison`** refuses a group of buildings, an empty group and a target below 0. Otherwise it walks the
  members. The first member alive that `UnitData::can_garrison@006099a0` admits decides the answer. That test needs
  a building (`+8 & 1`), valid (`+0x20`), finished (`+0x4c`), whose type `UnitTypeData::can_garrison` admits the
  member's type. The member fits if `num_inside(0)` plus its slots is at most the limit. A decoy (`unit_masks & 1`)
  takes no slots; any other unit takes its type's `control_cost` (`+0x2f0`, named by the type record).

### 23.2 What the layer does

- **The scene** (`crates/sim/src/observe.rs`, reads only): `BuildingScene::garrison_limit` and `inside`
  (`Sim::garrison_limit`, `Sim::num_inside`). `UnitScene::garrison_cost` holds the slots, and `UnitScene::enters` the
  owner's buildings with a limit whose type admits the unit's (`Sim::can_garrison`). **This touches `crates/sim`.**
- **The HUD** (`hud::garrison_button`): with a unit selection one of whose members `enters` somewhere, one button,
  GARRISON, two rows above the bottom edge at the right. A press on it is `Event::Garrison`.
- **The layer** (`crates/client/src/input.rs`): `Event::Garrison` with a unit selection sets `Input::garrisoning`. The
  next left press is the pick. `View::own_building` takes the nearest of the slot's buildings within `PICK_RADIUS`,
  and none keeps the pick. On one, `input::garrisons` is `picked_spot`'s arm and `test_garrison` read as above. It
  mints the selection's `group` and `garrison(o, who, 2)`, or nothing, and the pick is spent either way. A right
  press drops the pick. `garrison` is added to `MINTS`.
- **The walk** (`crates/rondata/src/input.rs`): unchanged. The pump already takes `garrison` through
  `group_garrison`.
- **The script** (`crates/rondata/src/script.rs`): `garrisoned()` is the hot-keyed script and, on `GARRISON_FRAME`
  6750, the Citizen `GARRISON_CITIZEN` `0/2`'s group in full and `garrison(2000, 0, 2)`.

**Stage 0** (`minted_garrisoned_at_the_opening_camera`), on 6750 at the opening camera:
- the Citizen `0/2` (`TypeIndex` 50) stands at pixel (1369, 468), gathering at the farm `0/2007`; it takes 1 slot
  and `enters` `[2000]`;
- the capital stands at pixel (1127, 521), finished, with limit 10 and nobody inside;
- the GARRISON button's centre is (1822, 1010).

### 23.3 The counts and the file

- `INPUT_GARRISONED`: **25 of 25**. The events are the hot-keyed ones, then three presses on 6750: the Citizen, the
  button and the capital. Through the HUD and the layer they mint `script::garrisoned()`. Seen red pinned 24.
- `INPUT_GARRISONS`: **6 of 6** (seen red pinned (0, 0)), the package's orders whole without its camera. Each is
  minted from a fresh layer. Its selection is the package's group, or for an empty group the slot's last full one
  (`add_group`'s repeat, §20). Its units are admitted, and the building is finished with room. Then the button, and
  a pick on the building's anchor.
- `minted_garrison_edges` checks these cases:
  - the button only for a unit selection with a member admitted somewhere;
  - no pick for a building selection or no selection;
  - a pick on no building keeps the pick;
  - each of these spends the pick and mints nothing: a limit of 0, an unfinished building, a full building, a
    building that admits no member;
  - the first admitted member's slots decide;
  - a right press drops the pick.
- `INPUT_KINDS` (19, 26), `INPUT_KINDS_ORDERS` (2,360, 2,429); `NOT_MINTED` drops `garrison`.

**The file.** `~/ron-golden/recordings/item4273-garrisoned-greatlakes-24000.rcx`, 59,620 bytes, SHA-256
`913bfd801ff3c529af07420bed4c80997ef656d0f7aafedb0a3976f2cd4d38d2` (`the_garrisoned_minted_packages_round_trip`;
seen red pinned 0). It holds the hot-keyed file's packages, then the garrison's on 6750, stamped 26. The closing stamp
goes from 26 to 27. The client's events through the HUD write it byte for byte.

### 23.4 The playback, and the order's effect

**run1613** (`docs/RUNS.md`) is run1610's line with the garrisoned file and no whole dump. The receipt names the
file's SHA-256. **Draws agree on 7,000 of 7,000 frames**, count and sequence. Of the window's 300 records, **267 are
whole and 33 part**, from 6871 on (`GARRISONED_WALKED`, pinned before the read with none parted). The first parting
is the Scout `1/0`'s `damage`, ours 13 and theirs 16, and its `damage_frame`, ours 6837 and theirs 6870. Theirs is
`damage_o 7`, a blow of the Slinger `0/7` (the attacked script's) that lands in the original and not on our walk.
The order itself is whole. The parting is downstream of it, on a unit the garrison does not touch (parked 4273a, the
journal). run1610, the same window without the garrison, parts on no record.

**The order's effect.** The Citizen `0/2`'s record, read on both sides: position, `inside_up` and its orders by class,
front first (`garrison_effects_theirs`, `garrison_effects_ours`). Ours were pinned before the take was read and seen
red pinned empty. Ours without the order is the hot-keyed file's walk.

| frame | the original | ours | ours without the order |
| --- | --- | --- | --- |
| 6750 | (3576, 29688), outside, `gather` | the same | the same |
| 6751 | (3576, 29688), outside, `move`, `garrison`, `gather` | the same | `gather` |
| 6766 | (3456, 30048), outside, `move`, `garrison`, `gather` | the same | (3576, 29688), `gather` |
| 6767 | (3456, 30048), `inside_up 2000`, `gather` | the same | outside |
| 6999 | (3456, 30048), `inside_up 2000`, `gather` | the same | outside |

The walk compares `inside_up` on every unit of every record (`diff::harness`'s inside chain), so the Citizen inside
the capital is compared on all 233 records from 6767 to 6999.

### 23.5 Confidence, and what is not established

The issuers, the button, the pick and `test_garrison` are read from the decompile, one reading each. Their call
sites were checked. The playback backs the command's bytes and the Citizen's entry (§23.4).

- **The button's predicate.** When `make_options` offers GARRISON is read only as far as its tests. The HUD shows it
  for a member that `enters` somewhere.
- **The pick's second arm**, for a selection holding type `0x136`, is not minted.
- **An ally's building.** The scene names no alliance, so the pick names the slot's own buildings only.
- **The capital's `inside_down`** is written on its `BUILDDATA` record and not parsed (`gamelog::BuildDump`). The
  walk compares the Citizen's `inside_up` on every record.
- **The right press** with the profile's `0x200` set (§6) is not minted.
- **The Slinger's blow on 6870** (§23.4): run1613 parts 33 of 300 records from 6871 on, on the Scout `1/0`'s damage.
  ~~Where it begins is not read (4273a).~~ ~~The stone leaves on 6863 on both sides and lands a frame later on ours
  (§24.5, item 4276). Its flight time is not read.~~ Settled by §25 (item 4277): the stone is `0/9`'s of 6868, loosed
  from piece 384's planar row two units off the original's bay; run1613 is whole, 300 of 300.

**Ours, not the original's:**

- Where the GARRISON button stands, its label, and when it shows.
- A right press drops the pick.

## 24. A building's infinite queue: the buildmask (item 4276)

*Item 4276, 2026-11-15. The score: `INPUT_KINDS` 19 → 20 of 26, and 2,360 → 2,365 of 2,429 orders (97.1 % → 97.4 %).
The census's next kind by share after `leader_options` is `buildmask` (5). All five recorded
orders are slot 3's in the 2017 sample, each a `group` of `2019` (empty when it repeats the last one sent) and
`buildmask(64, 1)`. Slot 0 of the minted game holds the actor: its capital `0/2000` is a training building, finished,
whose queue is empty on 6760, so the script queues a Citizen there first and then presses the infinite-queue button.
The buildmasked script mints 27 of 27, and the recorded buildmasks mint 5 of 5. The original plays the file
(run1614).*

### 24.1 What the button issues, and what the order does

- **The button.** `IFaceSelected::on_button_clicked@00826e00`, the selected panel's buttons, sends
  `Options::exec(OPTION_INFINITE, o)` for four of its buttons, told apart by name. The keymap also binds
  `OPTION_INFINITE` (`keymap_strings[0xc1]`).
- **The arm.** `Options::exec@007188c0`'s case `0x40` (`OPTION_INFINITE`) tests only that the console's selection is of
  buildings (the select group's `buildings` byte). On that it flashes the group and calls
  `GroupOut::issue_buildmask@00708820(0x40)`. Any other selection falls through to the case below it, `do_formation`.
  `Options::set_air_repeat@0071c740` is the same issuer's other caller, with `0x80` for an airbase's repeat.
- **The issuer.** `issue_buildmask` writes nothing in a playback (`semaphore & 0x10`). Otherwise it writes the
  package's group (`add_group`, with its repeat, §20), then command `0x21`, `[mask][1]`, 9 bytes. `set` is always 1.
  `CommandManager::issue_buildmask@00941f80` is the same without that test.
- **What it does.** `CommandPackage::process_buildmask@00947680` calls `Group::action_buildmask@006fc9a0(mask, set)`
  on the package's group. It toggles and never reads `set`. Each member that is active and that
  `WallData::valid_buildmask@0063e2a0` admits gets the bit: set if it lacks it and every admitted member before it was
  set, cleared otherwise. For `0x40`, `valid_buildmask` admits a member whose `BuildData::can_infinite@0062d4d0`
  passes: a training building with a unit's train job queued (types `0x32` to `0x19d`) that its owner may build. The
  port has had this since item 877 (`sim::Sim::action_buildmask`, `docs/GOLDEN.md` §33), with the re-queue at a train
  job's finish (`Sim::requeue_infinite`).

### 24.2 What the layer does

- **The HUD** (`hud::infinite_button`): with a building selection one of whose members is a training building of the
  slot's own (`BuildingScene::trains`), one button, INFINITE, three rows above the bottom edge, left of RENAME. A
  press on it is `Event::Infinite`.
- **The layer** (`crates/client/src/input.rs`): `Event::Infinite` with a building selection mints the selection's
  `group` and `buildmask(INFINITE_MASK, 1)`, `INFINITE_MASK` being `0x40`. Any other selection, or an open entry
  line, mints nothing. Whether a member takes the bit is left to the walk, as the original leaves it to
  `process_buildmask`. `buildmask` is added to `MINTS`.
- **The walk** (`crates/rondata/src/input.rs`): the pump takes `buildmask` through `group_buildmask`, the golden
  harness's entry since item 877.
- **The script** (`crates/rondata/src/script.rs`): `buildmasked()` is the garrisoned script and two packages more:
  - on `BUILDMASK_TRAIN_FRAME` 6760, the capital's `group` in full and `queue_up(CITIZEN, 1)`;
  - on `BUILDMASK_FRAME` 6762, the `group` as no objects and `buildmask(BUILDMASK_INFINITE, 1)`.

**Stage 0** (`minted_buildmasked_at_the_opening_camera`), on 6760 at the opening camera:
- the capital stands at pixel (1127, 521). It is finished, a training building, and its queue is empty;
- with the capital selected, the Citizen's button centre is (128, 1054), and the INFINITE button's is (1634, 966).

### 24.3 The counts and the file

- `INPUT_BUILDMASKED`: **27 of 27**. The events are the garrisoned ones, then three presses: the capital and the
  Citizen's button on 6760, and the INFINITE button on 6762. Through the HUD and the layer they mint
  `script::buildmasked()`. Seen red pinned 0.
- `INPUT_BUILDMASKS`: **5 of 5** (seen red pinned (0, 0)), the package's orders whole. Each is minted from a fresh
  layer. Its selection is the package's group, or for an empty group the slot's last full one, and the press is the
  INFINITE button.
- `minted_buildmask_edges` checks these cases:
  - the button only for a building selection with a training building of the slot's own;
  - nothing minted for a unit selection, for no selection, or with the entry line open;
  - a building that takes no bit still mints;
  - the same selection again sends its `group` as no objects.
- `INPUT_KINDS` (20, 26), `INPUT_KINDS_ORDERS` (2,365, 2,429); `NOT_MINTED` drops `buildmask`.

**The file.** `~/ron-golden/recordings/item4276-buildmasked-greatlakes-24000.rcx`, 59,637 bytes, SHA-256
`5e5166341d5c4d503d0ff18cee38e123a95a40a4d35b9fe838abc8c7ceea1ecb` (`the_buildmasked_minted_packages_round_trip`;
seen red pinned 0). It holds the garrisoned file's packages, then the two on 6760 and 6762, stamped 27 and 28. The
closing stamp goes from 27 to 29. The client's events through the HUD write it byte for byte.

### 24.4 The playback, and the order's effect

**run1614** (`docs/RUNS.md`) is run1613's line with the buildmasked file. The receipt names the file's SHA-256.
**Draws agree on 7,000 of 7,000 frames**, count and sequence. Of the window's 300 records, **173 are whole and 127
part**, from 6871 on (`BUILDMASKED_WALKED`, pinned before the read with the garrisoned file's 33). Every parting is the
Scout `1/0`'s `damage` and `damage_frame`, and nothing else. The first is the garrisoned file's (4273a, §24.5): ours
13 and theirs 16 on 6871, `damage_frame` ours 6837 and theirs 6870. The blows after it drift: theirs land on 6900 and
6930, ours on 6871 and 6920. So more records part than in run1613, where the two sides met again from 6904.

**The order's effect.** The capital's `build_masks & 0x40` and its queue's job counters, front first, read on both
sides (`buildmask_effects_theirs`, `buildmask_effects_ours`). Ours were pinned before the take was read and seen red
pinned empty. Ours without the order is the same file without the button's package.

| frame | the original | ours | ours without the order |
| --- | --- | --- | --- |
| 6760 | bit clear, queue empty | the same | the same |
| 6762 | bit clear, `[200]`, the Citizen queued | the same | the same |
| 6763 | **bit set**, `[300]` | the same | bit clear, `[300]` |
| 6999 | bit set, `[1900]`, the Citizen re-queued at the first one's finish | the same | bit clear, queue empty |

The walk compares `build_masks & 0x40` against the queue's `infinite` on every building of every record
(`diff::harness`, item 877), so the bit is compared on all 237 records from 6763 to 6999.

### 24.5 Confidence, and what is not established

The button's arm, the issuer and `can_infinite` are read from the decompile, one reading each.
`process_buildmask`, `action_buildmask` and `valid_buildmask` were read and run under the emulator against the port by
items 877, 2443 and 3200. The playback backs the command's bytes, the bit and the re-queue (§24.4).

- **Which buttons send `OPTION_INFINITE`.** `on_button_clicked` tells four of its buttons apart by name, through
  `int_str_array`; which panel draws them, and when, is not read. The HUD shows INFINITE for a training building.
- **The key** bound to `OPTION_INFINITE` (`keymap_strings[0xc1]`) is not minted.
- **The airbase's repeat**, `Options::set_air_repeat`'s `buildmask(0x80, 1)`, is not minted. No recording carries
  it.
- ~~**The Slinger's stone on 6863** (4273a): the garrisoned and buildmasked playbacks part on it. The stone leaves on
  6863 on both sides: the Slinger `0/7`'s `recharging` reaches 0 there, and every compared field agrees. It hits the
  Scout `1/0` on 6870 in the original and on 6871 on ours, a flight of 7 frames against 8.~~ **Settled by §25** (item
  4277): `damage_o 7` names the group's captain, not the shooter. The stone is `0/9`'s, loosed on 6868, and it flies
  4 frames in the original and 5 here, because piece 384's planar row put its launch two units off. Both playbacks
  are whole, 300 of 300. The sub-bullets below are this line's old reading, kept as written:
  - Ruled out: the units' cell-chain words (`up`, `down`, `down_who`, `up_who`), which the walk does not compare.
    Ours equal the take's on every unit of every record from 6700 to 6872.
  - Why the hot-keyed game does not part: its Scout takes other blows. There the original's Slinger reads `up -1`
    on 6786 to 6795, not `up 4`, and the Scout's damage parts from the garrisoned game's from 6837.
  - What is left: the round's flight time (`combat::flight_time`, `(int)(sqrtf(n) / d)`) at its launch. No dump
    prints it without `AMMO`.

**Ours, not the original's:**

- Where the INFINITE button stands, its label, and when it shows.
- The script queues a Citizen before the button, so that a member takes the bit.

## 25. The garrisoned file's blow: the Slingers' bays (item 4277)

*Item 4277, 2026-11-15. The score: `GARRISONED_WALKED` (run1613) **33 → 0** of 300 records parted, and
`BUILDMASKED_WALKED` (run1614) **127 → 0**. Both playbacks are now whole: draws on 7,000 of 7,000 frames and 300 of
300 records. Two packets named the round (run1615, run1615b). The original's own `get_position`, swept on the second
packet, replaced piece 384's three planar release rows with bays. **This item touches `crates/sim`**
(`data::launch`).*

### 25.1 Which stone it was

4273a read the parting as the Slinger `0/7`'s stone, from `damage_o 7` on the Scout `1/0`. But `damage_o` is the
captain of the striking group (`do_damage` step 2, `combat::fight::delivery`), not the shooter. Our walk's ammo pool,
printed frame by frame over 6840..6935 on the buildmasked script, holds no `0/7` round between 6852 and 6922. Two
rounds are aimed at `1/0` around the parting:

- **`0/9`'s**, born on 6868 from `CHAR_ATTACK2` frame 22, 5 frames. It lands first: `damage` 13 → 16 on 6872,
  `damage_frame` 6871.
- **`0/8`'s**, born on 6866 from `CHAR_ATTACK2` frame 22, 8 frames. It lands next: 16 → 19 on 6873.

In run1614's dump, `1/0` reads 16 from 6871 (`damage_frame` 6870) and 19 from 6873. So only the first stone, `0/9`'s,
lands a frame early in the original.

### 25.2 The packets

Both takes are run1614's line with a packet (`docs/RUNS.md`, run1615 and run1615b).

- **Take 1**, at 6865: the original's pool is empty, and so is ours. Ours is empty from 6852 to 6865. The brief named
  6862, which is earlier still. A round is in memory only while it flies, so the packet has to be taken during the
  flight.
- **Take 2**, at 6869 (`tools/recomp/ammo.py`): two live slots.

| round | the original | ours, before |
| --- | --- | --- |
| slot 0, `0/8` → `1/0` | (3679, **31773**, 684) → (2676, 32736, 623), 4 of 8 | (3679, **31775**, 684) → the same, 4 of 8 |
| slot 1, `0/9` → `1/0` | (**3389, 32212**, 686) → (**2733**, 32740, 633), 2 of **4** | (3391, 32213, 686) → (2702, 32740, 633), 2 of 5 |

**Why two units cost a frame.** `Ammo::init` times the flight on the straight line to the unled landing, then pushes
the landing along the moving target's heading by its `avg_speed` times that time (`lead_landing`). The unled landing
is the same on both sides, x 2857:

- ours is 2702 + 5 × 31;
- theirs is 2733 + 4 × 31.

From (3391, 32213) to (2857, 32740) is 750.26; from (3389, 32212), 749.54. Ours truncates to 5 and theirs to 4, and
`proj_speed` 150 is the only integer `d` that does both (749.54 / 5 < d ≤ 750.26 / 5). The lead then scales with the
time: 155 against 124.

### 25.3 The bays

`get_position.py … entries 384` reads three `AttachPos` rows off run1615b: node 0 at frame 21 of `CHAR_ATTACK1`, 22
of `CHAR_ATTACK2` and 16 of `CHAR_ATTACK3`, `RData +0x88` 1.0. A `sweep` of each at every whole degree, then
`fit_bay.py`, admits one `launch::Bay` lattice region per row, matching all 361 headings:

| row | lattice points | bay (centroid): right, fwd, dz |
| --- | --- | --- |
| `CHAR_ATTACK1` 21 | 1,696 | 69,764, 67,823, 178 |
| `CHAR_ATTACK2` 22 | 273 | 41,238, 117,982, 151 |
| `CHAR_ATTACK3` 16 | 592 | −42,142, 113,256, 177 |

Each bay's height is the planar row's. The planar rows came from chapter two's stones (`docs/COMBAT.md` §22) and
run1603's one stone (§15.1). They leave `launch::MEASURED`, and the bays enter `BAYS`. The sweeps sit outside git,
under `~/ron-data/lab-experiments/2026-11-15-item-4277/sweeps/`.

**The value diff** (our walk on the buildmasked script, against run1615b and run1614):

- On 6869, `0/9`'s round: launch (3391, 32213) → (3389, 32212); landing x 2702 → 2733; `total_time` 5 → 4.
  `0/8`'s launch y 31775 → 31773. The packet's, field for field (`the_buildmasked_game_s_round_in_flight_is_the_packet_s`,
  seen red on the planar rows).
- On 6871, `1/0`'s `damage`: ours 13 → 16, theirs 16; `damage_frame` ours 6837 → 6870, theirs 6870.
- `BUILDMASKED_WALKED` 127 → 0; `GARRISONED_WALKED` 33 → 0. Draws on 7,000 of 7,000 frames on both.
- run1603's round (§15.1) is unchanged, field for field (`the_rallied_game_s_round_in_flight_is_the_packet_s`).

### 25.4 Confidence, and what is not established

**Backed by a diff:** run1615b's two rounds; run1613's and run1614's 300 records each, whole. Every heading of
piece 384's three rows, against the original's `get_position` (`the_original_s_release_agrees_at_every_heading_for_piece_384`,
with `RON_BAY_SWEEPS_384`; seen red with `fwd` 117,982 → 117,000: 324 headings disagree).

**Not established:**

- The bay is the centroid of its lattice region. Every point of the region reproduces all 361 whole-degree headings.
  Between them the original's float path and the bay's truncation are not compared, and neither is a heading off a
  whole degree, because `whole_degrees` rounds the facing first.
- Other games' Slinger stones move with the bays wherever the planar rows stood off them. The booking gate's corpus
  says whether any floor moved. This lane ran the five widening families and `minted::` (§25's journal).


## 26. A garrisoned unit's icon: the come-out (item 4280)

*Item 4280, 2026-11-15. The score: `INPUT_KINDS` 20 → 21 of 26, and 2,365 → 2,369 of 2,429 orders (97.4 % → 97.5 %).
The six kinds left (§26.1) were listed first. `leader_options` cannot be minted from our lobby page. Of the three that
tie at 4 orders, `come_out` is the one whose actor slot 0 of the minted game holds: the Citizen `0/2`, inside the
capital `0/2000` from 6767 on the garrisoned and buildmasked walks (§23.4). The layer mints it from the building
panel's icon for that unit. The came-out script mints 28 of 28, and the recorded come-outs 4 of 4. The walk gains
the order's processor, which is a `command` door in `crates/sim`. The original plays the file (run1616, §26.4).*

### 26.1 The six kinds left, and their issuers

Every order of the six on disk, read off the recordings (a scratch test, not kept):

| kind | orders | where | issuer | slot 0's actor |
| --- | ---: | --- | --- | --- |
| `leader_options` | 50 | every file's frame-0 package, one each | `Game::run@00584590` → `LeaderOptionOut::update@006f1a60` → `CommandManager::issue_leader_option`, from `MiscAccess::my_leader_option` | the seat itself; no unit |
| `cheat_ai_toggle` | 4 | the bottle's, slot 0, on frames 6, 6, 101, 1507 | `ConsoleWin::run_cmd@007d6a70` case `0xc`, a cheat word | the console; no unit |
| `come_out` | 4 | the 2017 sample's, slot 3, `o` 59, 114, 92, 110 | `Options::exec@007188c0` case `0xaa` → `UnitOut::issue_come_out@00607be0` | **yes**: the Citizen `0/2` inside `0/2000` |
| `stance` | 4 | the 2017 sample's, slot 3, each `stance(−1)` on `2048` or a repeat of it | `GroupOut::issue_stance@0070b100`; no direct caller in the export | not read (§27: **yes**, the capital `0/2000`) |
| `disband` | 1 | the 2017 sample's, slot 3, `disband(0)` on `2008` | `IFaceSelected::on_button_clicked@00826e00`, `Options::do_disband@0071df20` → `GroupOut::issue_disband@0070a470` | not read |
| `unitmask` | 1 | the 2017 sample's, slot 3, `unitmask(0x100, −1)` on `0` | `Options::do_auto_explore@0071cbf0`, `Options::exec`, `Console::execute_at_cursor` → `GroupOut::issue_unitmask@00708880` | the Scout `0/0` (not taken) |

**`leader_options` is the profile's, not the lobby's.** `Game::init_rules_and_teams@00589bb0` resets
`my_leader_option` (`who` −1, the staging sentinel; `peasants` 0, `buildings` 0, `peasants_wait` 2; mask bits 1 and
3) and calls `LeaderOption::synch_with_prefs@006f1b50`. That copies `player_profile.prefs`' `peasants` and
`buildings`. It then turns five prefs bits into mask bits: `0x4000` → bit 1, `0x80000` → bit 3, `0x100000` → bit 4,
`0x8000` → bit 0, `0x800000` → bit 5. A set bit zeroes the word at `+0x18`, and a clear one sets it to 2 if it is
zero. `peasants_wait` becomes 2 when the prefs' own is nonzero, and 0 otherwise. The recordings fit this:
- The bottle's 49 carry `who` 0, mask `0x0a`, `+0x18` 2: prefs `0x4000` and `0x80000` set.
- The 2017 sample's carries `who` 3, mask `0x2a`, `+0x18` 0: `0x800000` set too.

Our lobby page holds what the `GAMEINFO` block prints, and nothing of the profile's. So it cannot mint the order as
it stands. Every file of ours carries a recording's value, `lo[0]`. That value changes nothing in the original:
`process_leader_options@009441d0` pushes onto units and buildings only the fields that differ from `init`'s. A minted
value other than that one would need the sim to carry `process_leader_options`, the SEAM `Sim::build_stance` names.

### 26.2 What the icon issues, and what the order does

- **The icon.** `Options::exec@007188c0` case `0xaa` names a unit, from the button. `ObjectData::get_inside` gives its
  container. If the container's vslot `0x18` answers 0, the container is not a unit but a building, and the arm calls
  `UnitOut::issue_come_out(unit)` and plays `S_EJECT_ACK`. Otherwise it calls `do_eject_all(−1)`, which is not minted.
- **The issuer.** `UnitOut::issue_come_out@00607be0`, under `check_accept_issue`, writes command `0x31` with the
  unit's own `who` (`+9`), `o` (`+0xa`) and `uid` (`+0x30`): `0xb` bytes, **with no group**.
- **What it does.** `CommandPackage::process_come_out@009465d0` takes `units[who][o]` when it is active and its `uid`
  is the command's, and calls `Unit::action_come_out@005e20b0`. That closes the unit's orders as
  `Group::action_return` does: `unit_masks &= ~0x4000000`, the path stack's length (`+0xc0`) to 0, `close_orders`,
  `clear_partial_path`, `update_action`. It then calls `come_out(this, 0)`, which this crate has carried since
  `docs/CITIES.md` §11 (`Sim::come_out`). A merchant arm comes in between: a unit of type index `0x34` or `0x35`
  inside a plain building of its own owner's that `is(0x1a4)` sets the leader's `0x2000000` bit and passes its trade
  route down its chain. That arm is not ported.

### 26.3 What the layer does, and the counts

- **The sim** (`crates/sim/src/units/come_out_order.rs`, **a `command` door in `crates/sim`**):
  `Sim::process_come_out(who, o, uid)` and `Sim::action_come_out(u)`, read as above, the merchant arm a SEAM. The
  order door forwards `process_come_out`. The scene gains `Scene::garrisons`, one `GarrisonScene` per squad captain
  inside a living building: the building's `o`, and the unit's `who`, `o`, `uid` and type index, in store and chain
  order. It is a read only.
- **The walk** (`crates/rondata/src/input.rs`): `come_out` goes to `process_come_out` on the order's own `who` and `o`.
  An order that names no live unit with that `uid` is a skip.
- **The HUD** (`hud::come_out_menu`): with a building selection, one icon, `OUT <o>`, per unit of the slot's own inside
  a selected building of its own (`input::garrisoned`), right to left, four rows above the bottom. A press on the
  `k`th is `Event::ComeOut(k)`.
- **The layer**: `Event::ComeOut(k)` with a building selection and the entry line closed mints `come_out(who, o,
  uid)` of the `k`th unit. It mints nothing for a unit selection, for an icon past the last, or for another slot's
  building. `come_out` is added to `MINTS`.
- **The script** (`crates/rondata/src/script.rs`): `came_out()` is the buildmasked script and one package more, on
  `COME_OUT_FRAME` 6780: `come_out(0, 2, COME_OUT_UID)`, the uid being 9.

**Stage 0** (`minted_came_out_at_the_opening_camera`), on 6780 at the opening camera:
- the capital stands at pixel (1127, 521);
- inside it, one unit: `0/2`, uid 9, type index 50;
- its icon's centre is (1822, 922).

**The counts.**
- `INPUT_CAME_OUT`: **28 of 28**. The events are the buildmasked ones, then two presses on 6780: the capital and the
  Citizen's icon. Through the HUD and the layer they mint `script::came_out()`. Seen red pinned 27.
- `INPUT_COME_OUTS`: **4 of 4** (seen red pinned (0, 0)), the package's orders whole without its camera. The command
  names no building, so each is minted from a selection of one building of the slot's own holding the unit (ours),
  and a press on its icon.
- `minted_come_out_edges` checks these cases:
  - one icon per unit of the slot's own inside;
  - none for another slot's building or a unit selection;
  - the second icon's order, carrying no group;
  - nothing minted for an icon past the last, another slot's building, a unit selection, or with the entry line
    open.
- `INPUT_KINDS` (21, 26), `INPUT_KINDS_ORDERS` (2,369, 2,429); `NOT_MINTED` drops `come_out`.

**The file.** `~/ron-golden/recordings/item4280-came-out-greatlakes-24000.rcx`, 59,649 bytes, SHA-256
`563fc717098372a61d7add8b1db4c42d84cc3d945fefd100e15e7620b3e57fc0` (`the_came_out_minted_packages_round_trip`;
seen red pinned 0). It holds the buildmasked file's packages, then the come-out's on 6780, stamped 29. The closing
stamp goes from 29 to 30. The client's events through the HUD write it byte for byte.

### 26.4 The playback, and the order's effect

**run1616** (`docs/RUNS.md`) is run1614's line with the came-out file. The receipt names the file's SHA-256. **Draws
agree on 7,000 of 7,000 frames**, count and sequence, and **all 300 of the window's records are whole**
(`CAME_OUT_WALKED`, pinned before the read with none parted). The walk compares `inside_up` on every unit of every
record, so the Citizen is compared out of the capital on all 219 records from 6781 to 6999.

**The order's effect.** The Citizen `0/2`'s record, read on both sides: position, `inside_up` and its orders by class,
front first (`garrison_effects_theirs`, `came_out_effects`). Ours was pinned before the take was read and seen red
pinned empty. Ours without the order is the buildmasked file's walk.

| frame | the original | ours | ours without the order |
| --- | --- | --- | --- |
| 6780 | (3456, 30048), `inside_up 2000`, `gather` | the same | the same |
| 6781 | (3432, 29880), outside, `gather` | the same | (3456, 30048), `inside_up 2000` |
| 6782 | (3432, 29880), outside, `gather` | the same | (3456, 30048), `inside_up 2000` |
| 6999 | (3576, 29688), outside, `gather` | the same | (3456, 30048), `inside_up 2000` |

The Citizen comes out on the frame after the order, south-west of the capital's centre. It keeps its gather order:
`come_out` closes what `close_orders` closes, and the gather survives it on both sides. By 6999 it is back at the
farm `0/2007`, at the point it stood on before the garrison (§23.4's 6750 row).

### 26.5 Confidence, and what is not established

- **One reading each**: the icon's arm, the issuer, `process_come_out` and `action_come_out`. The playback backs the
  command's bytes, `process_come_out`'s `uid` test on a live unit, and the Citizen's exit, its spot, its kept gather
  order and its walk back (§26.4). The `uid` test's refusal is not reached by any take.
- **The merchant arm** of `action_come_out` is not ported (parked 4280a, the journal).
- **The panel's icons.** Which garrisoned units the original's building panel lays out, in what order, and whether a
  selection of several buildings shows them all, is read only as far as `Options::exec`'s arm.
- **`do_eject_all`**, the arm for a unit inside a unit (a transport), is not minted.
- **`leader_options`** (§26.1): our lobby page holds no profile prefs, and the sim does not carry
  `process_leader_options`. A page row for the auto-manage prefs would mint the 50 through `synch_with_prefs`, but a
  file carrying any other value than the recordings' needs that processor first (parked 4280b).
- ~~**`stance`'s issuer**: `GroupOut::issue_stance@0070b100` has no direct caller in the export. Its caller, and what
  `stance(−1)` on a building does, are not read.~~ Read in §27.1: `Options::do_stance@00721480` writes the order
  inline, and the building arm writes `Build +0x7e`.

**Ours, not the original's:**

- where the icons stand, their label and their order;
- the recorded come-outs' building, which their command does not name.

## 27. A building's stance: the STANCE button (item 4282)

*Item 4282, 2026-11-15. The score: `INPUT_KINDS` 21 → 22 of 26, and 2,369 → 2,373 of 2,429 orders (97.5 % → 97.6 %).
The four recorded stances are one person cycling a building's stance: `stance(−1)` on a full group `[2048]` (the
2017 sample, slot 3, frame 9189), then three repeats sent as no objects (9191, 9192, 15059). Slot 0 of the minted
game holds the actor: the capital `0/2000`, a finished training building whose stance kind is worker. The layer mints
the press from a STANCE button. The walk gains the building arm of `Group::action_stance`, and the sim stores
`Build +0x7e`, which it had computed. Both are in `crates/sim`. The original plays the file (run1617, §27.4).*

### 27.1 What the button issues, and what the order does

- **The button.** `Options::exec@007188c0` case `0x11` calls `Options::do_stance@00721480` with the button's own
  option, and case `0x12` calls it with −2. `do_stance` writes the order inline, under `check_accept_issue`: the
  selection's `group` (`add_group`, with its repeat, §20) and command `0x02`, `[stance]`, 5 bytes. That is why
  `GroupOut::issue_stance@0070b100`, the same bytes, has no direct caller in the export. The gate is
  `Group::can_activate(selection, 1, −1)` and then a negative option, or a selection whose stance kind is combat. It
  plays `S_CHANGE_STANCE`.
- **What it does.** `CommandPackage::process_stance@00949ed0` calls `Group::action_stance@0070d440` on the package's
  group.
- **The group's kind** is `GroupData::get_stance_type@0070d370`'s. For a group of buildings the leader is `list[0]`.
  When the leader's kind is none or caster, the first member of another kind wins. If none does, the result is caster
  when a caster was seen, and the leader's own kind otherwise.
- **The option.** Six options for combat, four for worker, two for caster or packer. A negative option steps from
  `GroupData::get_stance_option@0070bab0`: the modal byte over the live members of that kind, a tie going to the
  lowest option. −1 steps forward, `(cur + 1) % n`; −2 steps back, wrapping to `n − 1`.
- **The building arm** (`0070d440:79`–`:87`). Each live member of a building group whose kind matches, and for which
  `is_build` holds (never a wall), takes the option in `Build +0x7e`. The unit arm (`+0xb1`, the combat arms) has
  been ported since item 2165 (`docs/GOLDEN.md` §79).
- **What reads the byte.** `Build::train@0062f9b0` copies it into each unit it trains when the unit's stance kind is
  the building's own (`Sim::trained_stance`). So the press sets the stance of every Citizen a city trains from then
  on, and a worker's stance is what `orders/jobs.rs` reads.

The sim's `Sim::build_stance` comment named two writers of `+0x7e`, `process_leader_options` and
`set_auto_peasant_level`. It missed this third one, `action_stance`. It now names all three.

### 27.2 What the layer does

- **The sim** (**`crates/sim`**). `Building::stance_set` holds `+0x7e` once a press writes it.
  `Sim::build_stance` answers that byte, and `Build::init`'s computed one before it, so the builds checksum line, the
  dump diff and `train` all read the press. `Sim::action_stance_buildings(list, s)` is the building arm, read as
  above. The `command` door forwards it.
- **The walk** (`crates/rondata/src/input.rs`, `group_stance_any`). `stance` on the package's group: a group of
  buildings (every object 2000 and up) is pushed as `process_group` pushes one and takes the building arm; any other
  group takes `group_stance`, the unit arm. Before this item the walk skipped `stance` as "not mapped".
- **The HUD** (`hud::stance_button`). One button, STANCE, three rows above the bottom edge and third from the right.
  It shows for a selection of the slot's own units, or of buildings one of which is a training building of its own.
  A press on it is `Event::Stance`.
- **The layer.** `Event::Stance` with a selection and the entry line closed mints the selection's `group` and
  `stance(STANCE_CYCLE)`, −1. Whether a member takes the option is left to the walk, as the original leaves it to
  `action_stance`. `stance` is added to `MINTS`.
- **The script** (`crates/rondata/src/script.rs`). `stanced()` is the came-out script and one package more, on
  `STANCE_FRAME` 6790: the `group` as no objects (the capital is still the last group sent) and `stance(−1)`.

**Stage 0** (`minted_stanced_at_the_opening_camera`), on 6790 at the opening camera:
- the capital stands at pixel (1127, 521), finished and a training building;
- with it selected, the STANCE button's centre is (1446, 966).

### 27.3 The counts and the file

- `INPUT_STANCED`: **29 of 29**. The events are the came-out ones, then one press on the STANCE button on 6790, the
  capital still selected from the come-out's press. Through the HUD and the layer they mint `script::stanced()`.
  Seen red pinned 28.
- `INPUT_STANCES`: **4 of 4** (seen red pinned (0, 0)), the package's orders whole. Each is minted from a fresh layer.
  Its selection is the package's group, or for an empty group the slot's last full one, and the press is on the
  STANCE button.
- `minted_stance_edges` checks these cases:
  - the button for the capital, for a mixed pick holding it, and for a unit of the slot's own;
  - none for a farm (no training building), another slot's building or unit, no selection, or the entry line open;
  - the order is `group [2000]` and `stance(−1)`, and the same selection again sends its `group` as no objects.
- `a_stance_press_on_buildings_writes_their_byte_and_their_trainees_take_it` (sim): −1 and −2 with the wrap, a member
  of no stance kind left alone, and a unit trained afterwards taking the written byte.
- `INPUT_KINDS` (22, 26), `INPUT_KINDS_ORDERS` (2,373, 2,429); `NOT_MINTED` drops `stance`.

**The file.** `~/ron-golden/recordings/item4282-stanced-greatlakes-24000.rcx`, 59,661 bytes, SHA-256
`56c9882fe065b883c6b76931b5502c0c0db513df83dd9d8050fc2ebe4db879e4` (`the_stanced_minted_packages_round_trip`;
seen red pinned 0). It holds the came-out file's packages, then the press's on 6790, stamped 30. The closing stamp
goes from 30 to 31. The client's events through the HUD write it byte for byte.

### 27.4 The playback, and the order's effect

**run1617** (`docs/RUNS.md`) is run1616's line with the stanced file. The receipt names the file's SHA-256. **Draws
agree on 7,000 of 7,000 frames**, count and sequence, and **all 300 of the window's records are whole**
(`STANCED_WALKED`, pinned before the read with none parted). The walk compares the buildings' `stance` and the units'
on every record.

**The comparison can fail.** The came-out script, which has no press, walked against run1617 parts 209 of the 300
records. The first parting is on 6791: `0/2000` `stance`, ours 0 against theirs 1 (a scratch test, not kept).

**The order's effect.** The capital's `+0x7e`, and each unit of slot 0 from `0/10` on with its `+0xb1`, were read on
both sides (`stance_effects_theirs`, `stance_effects_ours`). Ours were pinned before the take was read and seen red
pinned empty. Ours without the order is the came-out file's walk.

| frame | the original | ours | ours without the order |
| --- | --- | --- | --- |
| 6790 | capital 0 | the same | the same |
| 6791 | **capital 1** | the same | capital 0 |
| 6866 | capital 1; `0/10` trained, stance 1 | the same | capital 0; `0/10` stance 0 |
| 6999 | capital 1; `0/10` 1, `0/11` 1 | the same | capital 0; `0/10` 0, `0/11` 0 |

The press steps the capital's worker byte from 0 to 1 on the frame after the package. Both Citizens its infinite
queue trains after the press (§24.4) take 1 from it. Their walk on stance 1 agrees with the original's to 6999.

### 27.5 Confidence, and what is not established

- **One reading each**: `do_stance`, `get_stance_type`'s building leader, `get_stance_option` and `action_stance`'s
  building arm. The playback backs the command's bytes, the step from the modal 0 to 1 on one worker building, and
  `train`'s copy into two Citizens (§27.4). Not reached by any take: the −2 arm, a group of several buildings, the
  none/caster leader scan, and the combat, caster and packer kinds on a building.
- **Which button carries −1.** Case `0x11` passes the button's own option. That the cycling button's option is −1
  is inferred from the recordings, which carry only −1. The panel's per-option buttons (an option ≥ 0) are not minted.
- **A unit selection** mints the same order and the walk takes it through `group_stance` (item 2165). No take of
  ours has played one minted from the button.
- **`process_leader_options` and `set_auto_peasant_level`**, the other writers of `+0x7e`, are not ported
  (§26.1; parked 4280b).

**Ours, not the original's:**

- where the STANCE button stands, its label, and when it shows (the original's gate is `can_activate`; the stance
  kind of the selection is not read by our HUD);
- the recorded stances' selection, which a repeated `group` does not carry.
