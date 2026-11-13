# Client input: a person's events, as the script's orders

*Item 4145, 2026-11-11; §5 item 4149, the same day; §6 item 4179, 2026-11-12; §7 item 4180, the same day; §8 item 4195, the same day; §9 item 4194, the same day; §10 item 4202, 2026-11-13. Not `docs/INPUT.md`, which is the recorded order stream into the harness. The client's `input` module (`crates/client/src/input.rs`); the oracle is the minted script
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
  layer has no garrison arm (parked 4179c).
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
