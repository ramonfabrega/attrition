# Client input: a person's events, as the script's orders

*Item 4145, 2026-11-11; §5 item 4149, the same day. Not `docs/INPUT.md`, which is the recorded order stream into the harness. The client's `input` module (`crates/client/src/input.rs`); the oracle is the minted script
(`docs/RECGAME.md` §9), whose file the original played for 24,000 frames (run1561).*

An order minted from input events that equals the script's order, field for field, is an order the original has
already played. That is the whole method here: the script is the oracle, and the input layer is counted against it.

## 1. What an event becomes

The layer takes events, the session's state and a view, and mints orders in the script's own vocabulary,
`recgame::Order`: a recording frame, the issuing slot, then a `group` and the one command behind it.

- **The events** are a press at a pixel, a right press at a pixel, and a key.
- **The session's state** (`input::Input`) is the issuing slot, the selection, whether the selection is a building,
  and the building chosen to place.
- **The view** (`input::View`) is the camera's matrix and screen, the size the draw takes, the frame's `Scene` and the
  ground's `TerrainScene`.

Each event does one thing:

- **A press** selects the issuing slot's drawn object whose anchor is nearest the pixel, within `PICK_RADIUS` (24 px).
  The anchor is the object's point at its own `z`, through `render::camera::world_to_screen`. A press on nothing clears
  the selection. If a building has been chosen, the press places it instead: a `build` at the ground under the pixel,
  with `x2`/`y2` equal to `x`/`y`, and `queued` 2.
- **A right press**, with units selected, moves them: a `move_to` at the ground under the pixel. Its fields are the
  script's right-click fields: `set_angle` 0, `angle` 0, `orders` 1, `queued` 2, `form`/`width` −1, `disembark` 0.
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
a right press, and an unbound character key into a key. Each event runs under the camera and window size the frame is
drawn with. A minted order is issued for the frame about to be stepped, and printed. `--lobby` opens the script's
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
- that a right press always moves.

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
- **Everything the item ruled out**: a right press on an object (gather, attack, garrison), drag and box selection,
  control groups, queued (shift) orders, more than one object selected, a second slot, and every command beyond the
  script's three.
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
