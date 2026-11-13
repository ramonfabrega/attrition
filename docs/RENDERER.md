# The renderer — a proposal

*2026-11-06. A proposal, not a decision: Ramon decides what phase 4's first
items are, and this document is what he decides from. Every choice below is
marked **Choice**, with the alternative beside it. Nothing here was built
when it was written; item 4018 built the camera (§3.4), item 4026 the draw
list (§3.5).
Written by a worker on Opus 5.5 from the tree at `4314ac355`.*

**Why now.** Entry 29's three counters stand (`docs/DECISIONS.md` entry 29;
the seventieth pass ratified them, entry 110 §3): the long captures run
24,000 of 24,000 frames on both maps, the blind list is at open 0, and the
widening ledger closes by construction. By entry 29's letter, phase 4 opens.
Entry 110 §3 put the opening to Ramon, and he said go on 2026-11-06.

**How this was established.** The worker read `CLAUDE.md`,
`docs/DEBUG_VIEWER.md`, `docs/DECISIONS.md` entries 29, 38, 100 and 110,
`docs/FORMATS.md`'s model, animation and install sections, `docs/ORACLE.md`'s
camera and screenshot notes, and the parts of `crates/sim` and
`crates/rondata` a client would read: `Sim`, `Unit`, `Building`, `World`,
`anim::Guy`, `diff::run_traced_observed` and `debug_view::Window::observe`.
The function addresses come from the decompile index. The asset file counts
in §3 come from listing the install's directories, not from their contents.
Nothing has been run, and no claim here is diff-backed yet. Section 7 lists
what is still open.

## 1. Scope

**What the renderer is.** It is a thin client over observed simulation state,
as `CLAUDE.md`'s architecture section says. Each tick it reads a snapshot
from the simulation (§2) and draws it. It never writes anything back. The
simulation does not depend on it: `sim` keeps no graphics, windowing or async
dependency, and must still run inside a `#[test]` with no display attached.
The renderer is the **only** place art lives, and it can be swapped. Phase
6's asset swap replaces the renderer's asset layer and leaves the simulation
untouched.

**The original made the same split, in name.** The executable keeps its
drawing in `*Out` classes beside the simulation classes. The export holds 501
functions across 60 of them: `TerrainOut` 142, `GroupOut` 33, `UnitOut` 21,
`GuyOut` 9, `BuildOut` 7, and so on. `GuyOut::draw@005dc490`,
`UnitOut::draw@00615b80` and `BuildTypeOut::draw@00634060` are the draw
entries for a figure, a unit and a building type.

The split leaks, so a function's class name does not settle which side it is
on:

- `TerrainOut::find_tcoord_z@008544a0` is simulation state. It answers the
  tile heights `sim::world` pins.
- `TerrainOut::terraform_for_building@00875210` rewrites that height grid.
- `RiversOut::calculate_all` is map generation (`docs/MAPGEN.md`).

The rule here goes by data flow instead. Anything whose output changes a
later frame belongs to the simulation, whatever its class. Anything that only
turns state into pixels belongs to the renderer.

**What the renderer does not reproduce.** The original draws with `float`
transforms (`GuyOut::draw` builds `Transform<float>` locals). The renderer may
use floats freely: the no-float rule binds the simulation, not the client.
What the renderer must never do is hand a float back, cache a value the
simulation reads, or decide anything the next tick depends on.

**In scope for the first items:**

- A **replay viewer**: the simulation's own run of a scored capture, drawn
  tick by tick from a start dump and the capture's order stream, with no
  human input.
- A **camera** that reproduces the original's projection (§3, §5).
- **Terrain** from the simulation's height grid and cell land types.
- **Figures and buildings** at their simulated positions, facing and
  animation clocks.
- **Territory borders and fog** for one chosen viewer, read from the
  simulation's own grids.
- **The original's assets**, read from the user's install at runtime.

**Out of scope until something books it** (each is a §7 question):

- **Input that issues orders.** Selecting and panning are client concerns
  and cheap. Turning a click into an order means producing the original's
  command packages. That is the order stream's job (`rondata::input`), and
  today it replays recorded orders rather than minting new ones. A playable
  client is a later item, not a first one.
- **The `iface*` windows**, which are cut from v1 (`CLAUDE.md`, phases). Any
  heads-up display beyond a debug overlay waits for a decision.
- **Sound.** Pixels are the first oracle.
- **Starting a game without the original.** The simulation loads its world
  and starting units from the original's start dump (`docs/MAPGEN.md`,
  introduction; §15 for the map chain built so far). So a first renderer can
  only show games the original has started. That is enough for the oracle
  work in §5, and not enough for a game.
- **Conquer the World, the editor, triggers and multiplayer**, which stay cut
  as `CLAUDE.md` says.

**The lab's viewer stays a diagnostic.** `docs/DEBUG_VIEWER.md` §1 calls it
"not phase 4's game client", and this proposal keeps it that way.

- Its job is to show the original and the port side by side, after the
  harness's corrections, on logged frames only.
- A client's job is to show the port alone, every tick, with no dump.

The two share one thing: the observer seam, which §2 generalises.

**Choice: replay viewer first.** *Alternative:* a playable client against the
AI from the first item. That needs order minting, which the simulation does
not expose, and a game start without a dump, which it cannot do. Both would
come before a single correct pixel.

## 2. The observed-state interface

### 2.1 What exists

**The observer.** `rondata::diff::run_traced_observed`
(`crates/rondata/src/diff/harness.rs`) is the one read-only view of
simulation state today. After each comparison, `Replay::step` calls
`observe(&Built, &Frame, &FrameResult)`, and the callback holds only a shared
reference. Three properties matter here:

- **It fires on logged frames only.** `step` ticks forward to the next
  `FRAME` record and observes once. A client needs every tick.
- **It sees corrected state.** `Built::tick` installs the original's random
  word, and the figures' clocks, on every frame the dump traced. The pairs'
  games are uncorrected from frame 1 by construction (`docs/DECISIONS.md`
  entry 100 §4). Whether the 24,000-frame long captures install anything
  past their start block is not established here (§7).
- **It needs the whole log.** The harness reads every frame's dump to compare
  against. The debug viewer's 30-record export peaked at 1,353 MiB resident
  (`docs/DEBUG_VIEWER.md` §3), and most of that is dump reading, not the
  simulation.

**The stepping itself is small.** `Replay::step`'s loop is
`stream.apply(frame, &mut built)` followed by `built.tick()`, and
`Built::tick` is `Sim::tick` plus the installs. A runner with no dump past
the start block is that loop with the installs and the comparison removed.

**What the debug viewer reads.** `debug_view::Window::observe` pulls these
from `Built`:

- the world's size (`World::width`/`height` × `UNITS_PER_CELL`);
- each unit's `Pos` and path stack;
- `Built::units` (`UnitLink`: the log's `(who, o)` → the simulation's index);
- `Sim::unit_by_o`;
- the comparator's `FrameResult`.

It walks the **dump's** unit list and links each record to a simulation
unit. A client has no dump, so it has to walk the simulation's own list.

### 2.2 What the simulation already carries that a renderer wants

All of it is integer:

| record | where | what a client draws from it |
|---|---|---|
| unit | `Sim::units: Vec<Unit>` | `owner`, `index` (the original's `o`), `pos: Pos` (1/768 cell), `ty`, `health`/`max_health`, `on_map`, `inside`/`inside_unit`, `squad_size`, `attrition`, `sheltered`, `orders`, `path` |
| figure | `Unit::guys: Vec<anim::Guy>` | `gpiece` (the graphic piece), `anim` (the `UnitAnim` slot), `cur_time`/`end_time`, `follow` (a crew body's own `pos` and `facing: Angle`), `turret` angles |
| building | `Sim::buildings: Vec<Building>` | `owner`, `index`, `pos`, `ty`/`orig_ty`, `health`/`hits`, `active`/`started`, `constr_time`, `garrison` |
| terrain | `World` (private fields, accessors), `Sim::mountains` | `tile_z` (a pinned integer height per tile), `corner_z` (the corner grid as integer-mantissa singles), `cells` (`land`, `land_sub`, `flags`), regions, `coast`; the placed mountain ranges |
| territory | `World::owner`, `owner_at`, `Sim::borders` | the owner of each cell, and border lines |
| fog | `World::fog`, `fog_now`, `fog_detect` | per-player visibility planes on the half-cell grid |
| effects | `Sim::projectiles`, `Sim::deaths`, `Sim::nukes` | missiles in flight, and corpses fading |
| cities | `Sim::cities` | names, levels, centres, for labels |

The graphic piece is simulation state already, because an animation's frame
count decides when a random draw is spent (`docs/FORMATS.md`, "The animation
file"). `docs/ANIM.md` §3.2 derives a player's piece from the original's own
piece ranges (`GraphicPieces::init_piece_ranges@008f70e0`,
`GraphicPieces::get_unit_gpiece@0090c030`). The animation clocks are compared
against the dump's `GUY` records wherever a `DUMP_ALL` dump prints them
(below `DUMP_ALL` the per-frame `GUY` blocks are empty, ANIM §1), so a
renderer that reads `gpiece`, `anim`
and `cur_time` gets the original's model, animation and frame without
computing any of them.

### 2.3 The proposed snapshot

**Choice: a new module `sim::observe`, with `Sim::observe(&self, viewer:
Option<Player>) -> Scene`.** It returns an owned value, called every tick.

*Alternative 1:* the client reads `Sim`'s public fields directly, as the debug
viewer does. That needs no simulation change and couples the client to every
internal layout. `Sim`'s fields are public for the test harness's sake, not
as an interface. The first refactor of `Unit` would break the renderer.

*Alternative 2:* a borrowed view (`Scene<'a>` holding slices into `Sim`).
That is zero-copy, but it pins the client to the simulation's thread and
lifetime. At this game's scale (hundreds of units, not tens of thousands) a
copy each tick costs little.

*The name is a choice too:* the crate already uses `*_view` for the sweeps'
walks (57 modules), so `view` is taken.

**What `Scene` holds:**

- `frame: i64`.
- `units`: one `UnitScene` per unit **on the map**, carrying:
  - `id: ObjId`;
  - `owner`, `ty`;
  - `pos: Pos`, and `z: i32` (the pinned tile height under it);
  - `facing: Angle`;
  - `health`, `max_health`;
  - `figures: Vec<FigureScene { pos, facing, gpiece, anim, cur_time, end_time }>`.
- `buildings`: `id`, `owner`, `ty`, `pos`, `z`, `health`, `hits`, and a
  construction-progress pair, integers as the simulation keeps them.
- `projectiles` and `deaths`: position, kind and clock.
- `fog: Option<Vec<u8>>`: the chosen viewer's plane, `None` for an observer
  who sees everything.
- `terrain_version: u32` and `territory_version: u32`. Each counter moves
  when its grid changes. A terraform rewrites corners
  (`TerrainOut::terraform_for_building`), and territory moves with cities and
  forts. The client fetches the grid itself only when a version moves:
  `Sim::observe_terrain() -> TerrainScene { width, height, tile_z, land,
  land_sub, mountains }`, and `Sim::observe_territory()`.

**The rules the module holds:**

1. **Nothing in it is a float or a renderer type.** No `f32`, no colour, no
   matrix, no texture handle. `no_float.rs` already scans every file in
   `sim`, so a new module is guarded by construction.
2. **It is keyed by the original's identity, not by `Vec` index.** `ObjId` is
   `(who, o)`, the identity the dump prints and the harness links on. A
   dumped slot index is not an identity (`CLAUDE.md`), and nor is the
   simulation's `Vec` position. Whether an `o` is reused after a death within
   one game decides whether `ObjId` needs a generation counter. That is not
   established here (§7).
3. **It is read-only by type.** It takes `&self`. A test asserts that calling
   `observe` every tick leaves the soak digest (`soak.rs`) bit-identical to
   not calling it. That holds trivially today, and the test is there for the
   day someone adds a cache.
4. **It reports, it does not interpret.** It carries no "is this unit
   visible on screen" and no draw order. Projection and culling are the
   client's.

**As built (item 4017, `crates/sim/src/observe.rs`).** The shape above
holds, with five readings the text left open:
- `ObjId` is `{ who: Player, o: i16 }`.
- `buildings` holds the store's buildings with `alive` set. A building's
  progress pair is `constr_time` against `construct_hits`.
- `fog` is one byte per half-cell: `fog::UNSEEN` 0, `EXPLORED` 1 (the
  viewer's `seen2` bit), `VISIBLE` 2 (its `seen` bit). It is `None` for
  `viewer: None` and on a world with no fog grid.
- `TerrainScene.mountains` is each placed range's tile list.
- The counters are `World::terrain_version` and `territory_version`. The
  terrain counter moves on a changed tile height, corner or cell record,
  not on the movement bits in a tile's mask. The territory counter moves
  on a changed `(who, who2)`.

**Widened by item 4026** (§3.5), the door's one change for the draw list:
- `FigureScene.z`: the figure's own height, as `Guy::update_z@005d9950`
  keeps it. A sea unit's figure is at 0. An aircraft's is at its
  `Airframe::z`. Every other figure is on `World::data_z` of its own point,
  which may be below zero. The draw list anchors there, and so does
  `GuyOut::draw`. `follow.body` was already in: `FigureScene::pos` is a
  crew body's own point.
- `units` holds the **living** units on the map. A unit at no health keeps
  its record, but the original has taken it out of its object array.
- `o` **is** reused within one game: on the pair, `1/68`'s number belongs to
  a new plane by 2701. A client keys a figure on `(who, o)` per frame,
  never across a death.

**Amended by item 4027**: guy 0's `FigureScene::pos` is its own point,
`Movement::body` (`GuyData +0xc`), not the unit's `pos`. The two part when
another unit's collision pushes this one through `set_new_location` with
`move_guys` false: the unit moves and the figure stays until the guy next
moves (§3.5). No field was added; `observe` reads a different one.

The read-only test is `observe::tests::observing_every_tick_leaves_the_digest_alone`,
which still holds.
The orders a client issues go through `sim::command`, the other door.

### 2.4 What changes, and what does not

**In `sim`:** one module (`observe.rs`), three methods, the version counters
written where the grids change, and the read-only test. `Sim::tick`, the
mechanics and the harness do not change.

**In `rondata`:** a **free runner** that builds a `Sim` from a start dump
with `diff::build_sim`, and then each tick applies the order stream
(`input::Stream::apply`), calls `Sim::tick` and hands `Sim::observe` to a
callback. It installs nothing and compares nothing.

- The per-frame dump is not read past the start block, so memory is the
  simulation's, not the log's.
- `run_traced_observed` stays as it is, for the diagnostic viewer and the
  tests.

**Choice: the free runner lives in `rondata`.** That crate already owns the
start dump and the order stream, and tools take an install path. *The
alternative* is a new crate that depends on both. That is only worth it if
`rondata`'s compile time becomes the client's bottleneck.

**What need not change:** `Built`, `Replay` and `FrameResult`. The debug
viewer keeps its seam.

**As built (item 4020, `crates/rondata/src/free.rs`).** `FreeRunner` holds
a `Built` from `build_sim` and its own frame count. Each `step` applies the
stream, when there is one, and calls `sim.command().tick()`. `scene` and
`terrain` are `Sim::observe` and `observe_terrain`. It reads no `Sim`
field. Two of the readings above did not hold as written:
- **The start sibling's frames are read, the long capture's are not.**
  `FreeRunner::from_captures` reads the long capture's setup block, and
  each sibling through `IndexedCapture::with_replay_initial`. A setup-only
  sibling stood the units up with **no figures**.
  `diff::borrow_from_siblings` lends a start dump's guys only when the
  sibling carries frame seeds, and only its frames carry those. For a
  pair's `DUMP_ALL` start that is three frame states. The seeds and clocks
  that come with them are never installed.
- **The harness's corrections are the start block's, so the scenes
  agree.** `tests::the_free_run_s_scenes_are_the_harness_s` covers 3,390
  of 3,390 ticks of `GreatLakesAztecModern`, 35 of 35 logged frames
  among them, with zero differences (`SCENE_TICKS_AGREEING`). The harness
  side is `Built::tick` from the whole long capture's initial.

The clock's first readings sit beside the test (`RUNNER_CLOCK`,
`HARNESS_CLOCK`, measured alone under `/usr/bin/time -l`, warm caches):
- runner: 431 ms to frame zero, 952 ms to frame 3390, 325 MiB peak;
- harness: 540 ms, 1,088 ms, 458 MiB.

## 3. The visual oracle

### 3.1 Nothing from the install enters the repository

The renderer follows the rule the data layer already follows (`CLAUDE.md`,
hard constraints): **it takes an install path at runtime** and generates what
it needs on demand.

- **Decoders live in `rondata`**, beside `artdata` (the `.BHa` key-time
  reader) and `mountains::Tga`.
- Each decoder is **self-checking**: a chunk must end where its header says,
  as `.BHa`'s does today. `cargo run -p rondata -- <install>` re-derives the
  structural claims and exits non-zero when one stops holding.
- **Decoded caches** go under `~/Library/Caches/attrition/`, outside every
  worktree.
- **The original's screenshots** come from `tools/gamelog/shot.sh` on the
  capture lane, and are kept in the run archive like a dump, never in git.
  `docs/ORACLE.md`, "Traps that cost a run each", says to capture the game's
  rectangle and never the desktop.
- **Reference projection tables** (§3.3) are generated from packets, outside
  the repository, and regenerated by a tool.

**Choice: decoders in `rondata`.** *Alternative:* a separate `ronart` crate,
so that the format readers do not lengthen the release suite's build. Take it
if the decoders grow large. *Amended by item 4019:* the decoders read bytes
in `render` (`bh3`, `bha`), which depends on nothing. `rondata`, which
depends on `sim`, opens the install, resolves the files and surveys them.
So the client can decode without reaching `sim`.

### 3.2 What `docs/FORMATS.md` establishes, format by format

The file counts below come from listing the install, 2026-11-06.

| format | where | established | not established |
|---|---|---|---|
| BIG archives | — | **Absent**: the Extended Edition ships unpacked (FORMATS, "Verified: the shipped install") | nothing is needed |
| `.BHa` animation (2,062 in `art/`) | `render::bha`, `rondata::artdata` | The chunk container (`{u32 size; u16 id; u16 num_chunks}`); the root node's durations, frames = `round(times·3/200)`, as in `AnimMgr::force_load@0053ade0` and `AnimObj::load_hier@0054b700`, diff-backed against `DUMP_ALL` `end_time`. **Since item 4019, the whole file**: the node tree, every node's 36-byte keys (duration, quaternion, translation, one unread word), the skipped roots and `id 10` tracks; 2,062 of 2,062 decode (FORMATS, "The model files"). | The key's ninth word, the `id 10` element, and how a tree binds to a model it does not match. |
| `.BH3` model (1,524 in `art/`, 323 in `terrain art/`) | `render::bh3`, `rondata::artdata` | **Since item 4019, the encoding**: vertices, normals, texcoords, faces and the node tree, as `BH3Method::load_bh3@009156f0` reads them, self-checking; 1,845 of 1,847 decode, and the two that fail are named (FORMATS, "The model files"). Which file a piece plays: `<UNIT … model="…bh3">` (`artdata::unit_art`). | ~~The normals' fourth word~~ (item 4122: none; packed twelve bytes apart, FORMATS §3), and the transform fields compared against a loaded `HierObj`. |
| `.tga` texture (2,171 in `art/`, 730 in `terrain art/`) | `rondata::mountains`, `render::tga`, `client::ground` | Type 10 (run-length), 32-bit, descriptor `0x08`, bottom row first: the mountain templates (FORMATS, "The mountain templates"). Every kind the install ships (item 4037). **A `BASELAND` cell's base texture and its place**, by reading (item 4127, §3.6, "The ground's base layer"). | Which textures a model binds beyond those read; the blends, the coast, the cliffs, the water and the mountains' own textures. |
| `.png` (16 in `terrain art/`) | — | The extension only. | Their role. |
| `unit_graphics.xml`, `anim_graphics.xml` | `rondata::artdata` | Piece → animation slot → `.bha` file, looping or not. | Material, scale and attachment attributes beyond those read. |
| `effects_graphics.xml` `<MOUNTAINS>` | `rondata::mountains` | The template order, and `TEMPLATE_TEX` read as solid cells. | `MAIN_ALPHA_TEX` and `RING_ALPHA_TEX`, which are loaded for the renderer. |
| `mapstyles/*.xml` (22), terrain art's `bridgelabelsandratings` | — | — | Unexamined for drawing. |
| `.cur` (114 in `art/`) | — | — | Cursors. They wait on input. |

**The asset layer needs one more reading the simulation never needed:** what
a piece's `<UNIT>` binds besides its animations, meaning textures, team-colour
masks and scale. That reading belongs to item 4 in §6, and its claims go into
`docs/FORMATS.md` with their evidence, as every format claim does.

### 3.3 A frame of the original becomes a check

**What makes the oracle exact rather than visual.** The port's positions
already equal the original's on every compared frame of every scored capture.
That is what entry 29's first counter says. So comparing pixels is not the
first check. The first check compares **where the original would draw a
thing** with **where the renderer draws it**, both computed from the same
integer position.

**The original's projection is a function we can call.**
`MiscAccess::world_to_screen@00583810` adds the console's offset to
`MiscAccess::world_to_console@00583840`. That reads the camera, which
`BaseCamera::update_transforms@00537cc0` and
`Camera::update_transforms@008430a0` build. Tilt comes from
`Camera::adjust_tilt@00843970` and zoom from its sibling
`Camera::zoom_to_level`, and the
starting camera from `Game::init_camera@00590bc0`.

**Two rungs reach it** (`docs/EMULATOR.md` §8):

- **A packet** (`tools/explore/frame_snapshot.py`, the `RON_STATE_FRAME`
  tracer build) copies the process's private data, camera included, at one
  frame boundary. `tools/recomp/step4.py` then runs `world_to_screen` on it
  for every unit's dumped position. The result is the original's own screen
  coordinates for that frame and camera.
- **`tools/emu/callfn.py`** with a synthesized camera sweeps zooms, tilts and
  centres without a capture, at the "hour of synthesized state" the emulator
  document prices for a function that reads a singleton.

**Before booking a packet, grep the disk.** Packets already exist (run144's
is cited in FORMATS). The camera fields come from `types.txt`.

**The approximate transform already measured is not the oracle.**
`docs/ORACLE.md`, "Aiming a right-click", fits screen position as linear in
`u = wx − wy` and `v = wx + wy`. It gets within a tile or two, which is good
enough to aim a click and not good enough to score. It also ignores height.

**What can be compared, in order of cost:**

1. **Projected anchor points.** For each object, the screen point the
   original's function gives for its position and height, against the
   client's, at the same camera. The comparison is exact up to the client's
   rounding to whole pixels, and headless on both sides.
2. **The drawn set.** Which objects a viewer sees on a frame: on the map,
   alive, and inside the viewer's fog. The fog planes are simulation state,
   and the dump prints `mylos` (`docs/VISION.md` §2).
3. **The pose index.** Piece, animation slot and frame. The simulation
   already agrees with the dump on these, so the client only has to resolve
   them to the right files.
4. **Silhouettes against a screenshot.** The original's frame, taken at a
   chosen frame (`pause` and `break #` from the console, `docs/ORACLE.md`,
   "The `~` console") and camera (`cheat camera`), is compared with the
   client's frame on object footprints, never on pixels. **A second
   photograph is the first one's take with one staged line** (item 4160,
   run1564): `3200 camera 192,200` before `3294 pause 1`. The lane's
   `Console Coord Mode=2` reads it as a tile, and the packet's `Camera`
   holds `loc` at that tile's centre, (36960, 38496), with zoom, tilt,
   direction and screen unchanged. The lane profile's `StartReveal=1`
   already holds the display reveal (`semaphore` bit 1, `run_cmd` case
   0x57), so no fog darkens a far camera. The window's capture carries a
   32-row title bar that run1559b's lacks. Cropped by 32 rows, its
   bottom-left interface matches run1559b's photograph with a mean
   difference of 8.8, summed over the three channels. The next best
   offset, 31 rows, gives 25. The camera is `RUN1564_LOC` and the
   photograph is `ground::tests::run1564`.

**Nothing is pixel-exact at first, and maybe never.** The original draws
through Direct3D 11, then DXVK, then MoltenVK, then Metal, with its own float
transforms and filtering. A pixel difference against that measures the
translation stack as much as the port. Rung 4 is for the art phase's eyes and
for the silhouette metric in §5's third candidate. It is not a gate.

## 3.4 The camera, read and in register (item 4018)

*2026-11-08, Opus 5.5. Built in `crates/render/src/camera.rs`; checked by
`crates/render/tests/camera_sweep.rs` and `camera_packet.rs`.*

**The score.** Camera states in register: **240 of 240** on the sweep,
9,600 of 9,600 points within one pixel and with the same off-screen flag,
and every `to_screen_matrix` element bit-identical to the original's. On
run1484's packet: **394 of 394** points (197 dumped positions, each at
two screen sizes), the matrix bit-identical. Both are pinned
(`IN_REGISTER`). The sweep was made to fail once on purpose: with the
negative-angle reduction read the obvious way it read 160 of 240.

**How it was established.** Every claim below was read off the decompile
export and the listing, and then checked by running the original's own
functions on the same inputs. Nothing was transcribed (`CLAUDE.md`).

- **The view.** `Camera::update_transforms@008430a0` builds the eye from
  two transforms.
  - The local transform is a translation of `(0, 0, −zoom_distance)`. For
    the game's own camera (`MiscAccess::camera`) the distance is first
    multiplied by the global `camera_scale`.
  - `axis_transform` is reset, then rotated by `rotate_x((int)(tilt_angle
    + 180))` and `rotate_z((int)dir_angle)`. Both angles are **truncated
    to whole degrees** (`cvttss2si`). It is then moved to `loc + adjust`,
    at height `loc_z_adjust`.
  - `BaseCamera::update_transforms@00537cc0` composes the two, `axis ∗
    local`. `Transform<float>::operator*@004217b0` applies the right-hand
    operand first. It writes `render_matrix` as the rigid inverse of the
    result: the rotation transposed, the translation `−t·Rᵀ`, inline.
  - The camera's parent is a `HierObj` whose world transform has type 0,
    so it composes as the identity. On the packet its bytes are stale.
- **The rotations.** `Quat<float>::set@00420c70` takes a whole-degree
  angle. It sets `w = cos(a/2)` and the axis times `sin(a/2)`, both from
  the half-degree tables (`fast_half_degree_to_cosine@00a29010` and its
  sine twin). Each table entry is `cosf(i · ½ · DEG)` with `DEG` =
  `0x3c8efa35`. A later rotation composes as `existing ⊗ new`.
  `Matrix<float>::fill@00420df0` writes the standard matrix of `q`,
  row-major, for row vectors.
  - **The quirk**: a negative angle reduces to `359 − (−a mod 360)`, not
    `360 − …`. So `−1°` turns as `−2°`, and `−360°` as `−1°`. The sweep's
    negative `dir_angle` and `tilt + 180` states depend on it.
- **The projection lives in the graphics DLL.** `graphsys` is not built by
  the executable. `GraphSys::init_system@0054abf0` loads a DLL and calls
  its export `get_graphsy_object_ptr`. The install's `d3dgl.dll` is that
  DLL, and it ships its own `d3dgl.pdb` and `d3dgl.map`. Vtable slots
  `0x198` and `0x19c` are `D3D11GLSys::set_projection` (DLL `0x10020700`)
  and `set_projection_parallel` (`0x10020830`). Slot `0x194`
  (`set_camera_transform`) only uploads the matrix.
  - Both take the tangent of the half angle the same way: the half angle in
    single precision (`angle · π/180 · ½`), the tangent in double
    (`__libm_sse2_tan_precise`), converted back to single. Both write
    `m0 = 1/t` and `m5 = 1/(t · view_ratio)`.
  - Perspective sets `m10 = f/(f−n)`, `m11 = 1`, `m14 = −m10 · n` and
    `m15 = 0`.
  - Parallel sets `m10 = pd/(f−n)`, `m11 = 0`, `m14 = −(n · pd)/(f−n)` and
    `m15 = pd`, where `pd` is `parallel_distance`.
  - Either one leaves the identity unless `angle > 0` and `far > near`.
- **The game's camera is the parallel one.** run1484's packet holds
  `parallel = 1`, with `parallel_distance` equal to `zoom_distance / 2`
  (10,429.4 against 20,858.8). Its `fixed_zooms` are the seven levels the
  sweep draws from, and every `fixed_angles` entry is 60. So the zoom
  scales the picture only through `parallel_distance`. The eye's distance
  moves depth and nothing on screen.
- **`to_screen_matrix` is `render · projection`.** `Matrix<float>::operator*@00420f30`
  called as `projection.operator*(render)` returns `render · projection`,
  each element summed left to right.
- **The screen point.** `MiscAccess::world_to_console@00583840` is
  `__cdecl (x, y, z, w, h)`, with the outputs through `ecx` and `edx`
  (the listing, not the decompile, says so).
  - Height enters as the third integer, directly. This settles §7.1's
    question.
  - The point goes through `to_screen`, in the order `m4·y + m0·x + m8·z +
    m12`. It is divided by `w` unless `w` is 0 or 1.
  - It is mapped as `(x′ + 1) · w · ½` and `(1 − y′) · h · ½`. Each result
    is truncated, then stepped up when the dropped fraction is at least
    ½, which a negative coordinate's fraction never is.
  - A `w` or `h` below 1 takes `graphsys`'s `scaled_screen_xs` and
    `(int)(scaled_screen_ys · natural_ratio)`.
  - The return is 1 only in that case, and only when the point falls
    outside [−1, 1].
  - `world_to_screen@00583810` adds the console's short at `+0x12a` to y.
- **Units.** The dump's `x_internal`, `y_internal` and `z_internal` are the
  world integers `world_to_screen` takes. The camera's `loc_x` and `loc_y`
  are in the same units.

**The two checks.**

- **The sweep** (`tools/render/camera_oracle.py`, seed 4018). It runs 240
  synthesized states over every field `update_transforms` reads:
  - the seven `fixed_zooms` and random zooms, and `camera_scale` off 1.0;
  - tilts from −260 to 120 and directions from −720 to 720, the negative
    ones included;
  - adjusts, both projections, four screen sizes and console offsets.

  Each state carries 40 integer points, heights up to 6,000. Half are
  projected at an explicit size, half at `graphsys`'s. The oracle lays
  out a `Camera` at the PDB's offsets and maps `d3dgl.dll` beside the
  executable. It enters the original's `update_transforms`, then its
  `world_to_screen`, under `tools/emu`'s harness, which it imports and
  never edits.
  - It answers four imports with Python's double libm: the CRT's `cos` and
    `sin`, the DLL's tangent, and `memset`.
  - It stubs three callees that write no matrix the projection reads:
    `D3D11Context::set_projection`, slot `0x194`, and
    `Camera::compute_clip_planes@00842950`.
  - A perspective point with the original's own `w < 1` is not drawn as
    an input, since it has no pixel. No state is dropped.
- **The packet** (`tools/render/packet_points.py`). It maps run1484's
  packet, logger frame 676, lazily, a page at a time. It calls the
  original's `world_to_screen` on the game's own camera, `graphsys` and
  console, for every `SUBOBJECT` on dump block 676. No packet was booked.
  run1484's was already on disk, and its camera is the in-game one.

**Diff-backed**: every bullet above. The sweep and the packet run the
original's code on the inputs, so a reading error in any of them fails a
pinned test.

**What is not established.**

- **The two imports' last bits.** The bottle resolves `cos`, `sin` and
  `tan` to Wine's `ucrtbase`. The oracle and this crate both use the host's
  libm. A one-ulp disagreement there could move a table entry. The packet
  check, whose matrix the bottle built, is bit-identical. That is one
  camera, not a proof.
- **The camera's animation.** `Camera::adjust_tilt@00843970` interpolates
  `tilt_angle` between `fixed_angles`, and `Camera::zoom_to_level` only
  arms a zoom. The per-frame step that moves `zoom_distance` and
  `parallel_distance` is not read. This crate takes the fields as state.
  Every `fixed_angles` entry on the packet is 60, so the interpolation is
  unexercised. Its decompile reads `(zoom − angle)` where `(zoom − zoom₀)`
  would be expected. The listing settles which, when an item needs it.
- **`Game::init_camera@00590bc0`**, the starting state, is not read. A
  replay viewer will take its camera from the packet's values, or from a
  user, until something books it.
- **Culling.** `compute_clip_planes` was stubbed, not read.
- **Points behind a perspective eye.** They are outside the sweep's domain.
  The game's camera is parallel, where they do not arise.

## 3.5 The draw list, and frames in register (item 4026)

*2026-11-09, Opus 5.5. Built in `crates/render/src/draw.rs`, fed by
`rondata::free::placed`; checked by `rondata::free::tests::the_draw_list_s_frames_are_in_register`.*

**The score.** On `GreatLakesAztecModern` (4020's pair), run1429's logged
frames: **35 of 35 in register** (`FRAMES_IN_REGISTER`; 33 at 4026, 35 at
4027), 3,842 of 3,842 figures within one pixel of the original's own
projection, and the drawn sets equal on all 35. **K 31 of M 59** (§5.4).

**What the list is.** `render::draw::draw_list` takes one `Placed` per
figure (object id, figure index, the figure's own point and height, piece,
slot, frame, facing), a `Camera`, a `Screen` and an optional `Sight`, and
returns one `Drawn` per figure drawn: the same identity and pose, the
anchor from `world_to_screen`, and a depth key. The depth is the point's
camera-space z (`render_matrix`'s third column), furthest first, ties by
identity. `render` still depends on nothing in `sim`. `rondata`, which holds
both, copies a `Scene` into `Placed` (`free::placed`, `free::sight`), so the
edge stays `rondata` → `render`, as item 4019 set it.

- *Drawn:* a figure with a piece (`-1` draws nothing). For a viewer, also
  only its own figures and those on a half-cell its plane holds `VISIBLE`.
- *Not culled:* the clip planes (`Camera::compute_clip_planes@00842950`)
  are unread, so an off-screen figure is listed at its off-screen pixel.

**The reference.** `GuyOut::draw@005dc490` calls `world_to_console` on the
guy's own `+0xc`, `+0x10`, `+0x14` with `w = h = −1`, then adds the
console's offset (the listing, `005dc4d8`–`005dc52a`). So a figure's anchor
is the `GUY` record's `x y z`, not its unit's `x_internal`. The point is not
interpolated there. The bank at `+0x44`/`+0x48` is blended by
`graphic_percent`. For each logged frame, every figure the dump prints for
a unit with `inside_up` −1 goes through the original's `update_transforms`
and `world_to_screen` under unicorn (`tools/render/camera_oracle.py
points`, 4018's harness, `tools/emu` untouched). The camera is pinned:
run1484's game camera moved to the map's centre, at 1920×1080. A frame is
in register when the two drawn sets are equal and every anchor is within
one pixel on both axes.

**The door, widened twice** (§2.3's "As built"):

- **A figure's height.** `FigureScene` had no `z`, and `UnitScene::z` is
  the tile under the unit. A figure stands on `find_data_z` of its own
  point, or on its own altitude. Before the change, 3,240 of 3,842 figures
  agreed and 0 of 35 frames.
- **A dead unit.** `observe` filtered `on_map` only. A unit at no health
  keeps its record until the slot is reused, and the original has taken it
  out of its object array (run1429 stops printing `0/3`, `0/4`, `0/12` and
  `0/13` by 2601). `1/68`'s number belongs to a new plane by 2701. Before
  the change, 24 of 35 frames were in register.

**The two frames 4026 left out**, and why they came in (item 4027). Each
was one guy 0 standing apart from its unit's point:

| frame | figure | dump `GUY` | original's pixel | ours at 4026 (the unit's) | 4026 pixel | ours at 4027 | 4027 pixel |
|---|---|---|---|---|---|---|---|
| 2501 | `1/9` guy 0, piece 11264 | (39869, 23358, 305) | (3554, 1827) | (39879, 23369, 305) | (3554, 1829) | (39869, 23358, 305) | (3554, 1827) |
| 3001 | `1/67` guy 0, piece 232 | (37576, 19846, −16) | (3746, 1433) | (37591, 19892, −14) | (3741, 1438) | (37576, 19846, −16) | (3746, 1433) |

The input that parted was the figure's position. Height, camera and
projection all agreed. 4026's reading said a figure needs its own point
apart from its unit's. That was right, and the sim already carries that
point: `Movement::body`, guy 0's `+0xc`/`+0x10`. `observe` had read the
unit's `pos` instead.

- **Who moved the unit.** Both units' `collide_frame` is the frame before
  the logged one (2500, 3000). Their `collide_o` names a later unit (75
  and 90). That unit's collision resolution moved the pushed unit through
  `Unit::set_new_location@005f8d20` with `move_guys` 0, after the pushed
  unit's own guys had already moved this frame (`Guy::move@005d9240` walks
  a guy to its destination; it does not read the unit's point).
- **Checked on every frame.** Over all 35 logged frames, these are the only
  single-guy units on the map whose figure stands apart from its unit. A
  third, `1/29` on 2501 (pushed by 75, 1 unit off), was already within a
  pixel. Every other unit with `collide_frame` on the frame before has its
  figure on its point, so the push alone does not part them.

**Made to fail once.** Reading a figure's height as its unit's tile
(`UnitScene::z`) gives 0 of 35 frames and 3,240 of 3,842 figures. Reading
guy 0's point as its unit's (4026's door) gives 33 of 35, and
`observe::tests::a_scene_agrees_with_the_sim_it_copies` fails on its push
case.

**The clock** (`DRAW_CLOCK`, measured as §2.4's): 434 ms to frame zero,
994 ms to frame 3390, 325 MiB peak. The runner alone, the same hour, read
428 ms, 974 ms and 325 MiB. The list costs about 20 ms over 3,390 ticks
(6 µs a tick for 109 figures), and no memory. The slowest single list took
18 µs.

**Diff-backed:** the anchors, the height rule, the dead-unit rule and the
drawn sets on the observer's view, all against run1429's dump and the
original's own projection. **Listing-backed:** the reference's choice of
point (`GuyOut::draw`).

**What is not established.**

- **The viewer's drawn set.** run1429 dumps `UNITS` and `GUYS` only, with
  no fog plane on its frames. So the reference here is the observer's:
  every figure on the map. The `Sight` rule is unit-tested, not diffed.
  The original's own draw predicate (`UnitOut::draw@00615b80`'s gate) is
  unread. run1428's `DUMP_ALL` frames print `seen` planes, which is
  where a viewer's check would start.
- **The depth key's order** against the original's. No dump prints a
  draw order.
- ~~**Buildings.** They are not in the list. `BuildingScene` carries no
  piece, and run1429's frames print no building.~~ In the list since item
  4035, below.
- **A building's drawn set is a reading.** Both sides run the same
  `WallOut::draw` predicate, each on its own state, so the comparison
  checks the inputs (finished, `job_counter`, the construction time) and
  not the predicate. `is_force_construction`, and the caller's own gate
  on `WallOut::draw`, are unread. So is whether a viewer sees a fogged
  building it has seen before (`ever_seen`); the `Sight` rule for
  figures is applied to buildings unchanged.
- **The piece drawn is `render_gpiece`, not `gpiece`.** `WallOut::draw`
  draws `+0x68`, which `Wall::update_gpiece@00643070` may set an age
  lower, or to the console player's age. The list carries `gpiece`, the
  field the dump prints. The anchor does not depend on it: `halfoffset`
  is the same for every `<BUILD>` of a type.

**Buildings in the list** (item 4035). `render::draw::PlacedBuilding`, one
per building in use, copied by `rondata::free::placed_buildings`, leaves
the list as a `Drawn` whose `Pose` is `Building`. The reference is
`WallOut::draw@00641930`:

- *The anchor.* `world_to_screen(x − 0x60·h, y − 0x60·h, z, −1, −1)` on
  the building's own `x_internal`, `y_internal`, `z_internal`
  (`SubObjectData +0x10`, `+0x14`, `+0xc`), where `h` is
  `BuildRDataStruct::build_half_tcoord`, the drawn piece's `<BUILD>`'s
  `halfoffset` (0 or 1; the listing at `00642033`–`0064207d`). A building
  is a sprite: the half tile back is where the sprite's own origin sits.
  `AIRDEFENSE`, `RADAR` and `SAM` take a model at the point, no offset.
- *Drawn.* The alpha is opaque past 0.88 of the share built
  (`job_counter / BuildData::construct_time(0)`, 1 when finished), fades
  in from 0.84, and is inverted for every type but the farm. So an
  unfinished building is drawn past 0.84, a farm only before 0.88, and
  type `0x1a6` only finished. A farm is drawn by a second pass too,
  `FarmsOut::draw@008d75f0`'s wheat field, finished or not (item 4048,
  below).
- *The piece.* `GraphicPieces::get_build_gpiece@0090bce0` from the
  current type, the **founder's** nation (`BuildData::founder`; a captured
  city keeps the art of the nation that built it) and the building's age:
  down the ages in the nation's style, then its backup style, then the
  default, the first `<BUILD>` that loads (`artdata::BuildLook`).

**The score.** On the lobby's every frame that prints buildings — run1428's
two, run1429's start and closing block, run1429b's 95 — **99 of 99 frames
in register, 4,564 of 4,564 records** (3,285 drawn, the rest drawn by
neither side), each anchor within one pixel of the original's projection
of the dumped point, and **4,564 of 4,564 pieces the dump's `gpiece`**
(`BUILDING_FRAMES_IN_REGISTER`, `BUILDINGS_IN_REGISTER`,
`BUILDING_PIECES_AGREEING`). Building K **20 of 21** (§5.4).

**The door, widened.** `BuildingScene` gained `orig_ty`, `job_counter`,
`construct_time`, and the founder's tribe and the owner's age. Its `z` is
`Building::z` now, not the live tile:

| record | dump `z_internal` | ours, wrong | pixel, theirs | pixel, ours wrong | ours now |
|---|---|---|---|---|---|
| run1429b 3295 `1/2026` (Factory) | 185 | 214, the live tile | (3766, 1725) | (3766, 1720) | 185, (3766, 1725) |
| run1428 0 `0/2000` (capital) | 536 | 546, after its own terraform only | (−3384, −527) | (−3384, −529) | 536, (−3384, −527) |

- A building's height is the ground after **its own** terraform, at
  placement, and nothing rewrites it. The Factory's tile went 176 → 185
  when it was placed (1583), and 185 → 214 when a later terraform reached
  it (2186). The original kept 185.
- The start's buildings take the setup block's `z_internal`
  (`diff::build_sim`). Standing them up one at a time leaves each capital
  on the ground its own terraform left, before its neighbours' moved it.

**Made to fail once.** No `halfoffset`: 3,397 of 4,564 records, building K
13, every half-offset type 15 pixels low. The owner's style for the
founder's: 4,563, the captured capital `1/2047` on the Aztecs' piece 57015
against the dump's 57144. The live tile for `Building::z`: 4,468, the
Factory 5 pixels off.

**Diff-backed:** the anchors, the heights, the pieces. **Listing-backed:**
the half-tile rule. **Reading only:** the drawn predicate.

**The farm's wheat field in the list** (item 4048). A building with a farm
record (`BuildData +0x78 ≥ 0`, from `Farms::add@008d8a40` at placement)
carries a `render::draw::Field`, and leaves the list a second time as
`Pose::Field`. The reference is `FarmsOut::draw@008d75f0`:

- *The corner.* `WallData::tile_corner@00643440` of the building's point by
  its type's footprint (`BuildTypeData +0x234`, `+0x238`): the point's tile
  (`div_3_table[(x ^ 0x63637) >> 6]`, the position unmasked), snapped to its
  centre on an odd size, read back as a tile, less half the size, times
  `0xc0`. The field is 4×4 stalk boxes a tile apart, the first half a tile
  in from the corner. The entry's anchor is the corner itself on
  `terrain_height[0][0]`: the point the 25 heights are sampled from.
- *The heights.* `FarmStruct::terrain_height[5][5]` (`+0x48`, `x` major):
  `TerrainOut::find_data_z` at the footprint's 25 tile corners, sampled by
  `Farms::add` for a crop farm (a pasture jumps past them and keeps zeros),
  and again by `Farms::recalc_heights@008d88a0`, which `Wall::init@0063e9b0`
  calls after a non-farm building's terraform, for **its owner's** farms
  whose corner is within eight tiles on both axes. `sim::Farm::heights`
  keeps them; a starting farm takes the dump's.
- *Drawn.* Every record but the pasture's (`farm_type != 1`). The viewer's
  gate (vtable `+0x17c`) is read as the buildings' `Sight` rule.

**The score.** On the lobby's 99 building frames, **402 of 402 farm records
in register**, all 402 drawn by both sides, each corner within one pixel of
the original's projection, and **402 of 402 records whole** (`farm_type` and
all 25 heights the original's): `FIELDS_IN_REGISTER`, `FIELD_RECORDS_WHOLE`.
The original's record is the dump's own `Farms::log_data` list where it
prints one (run1428), the start's on run1429's frame 0, and run1559b's
packet at logger frame 3294 for run1429b's window and run1429's 3390 (the
list at `0x00c0a914`, read once and held in the test). The reference corner
is `sim::cities::place::tile_corner_of`, which item 2122's sweep holds
against the original. Building K **21 of 21**, **K 59 of M 59**.

**Made to fail once.** The anchor half a tile in (the first stalk box): 0
of 402, every corner 15 pixels low. No `recalc_heights`: 18 of 402 whole
(`1/2002` on 3390 reads 196 where theirs reads 221); the corner's own
height holds, so in register stays 402.

**Not established.** `Farms::add`'s own sample: with it disabled the 402
stay whole, because every farm placed in this game is resampled by a later
recalc before a logged frame. The field's mesh is not built: the client
skips `Pose::Field` (parked 4048a).

**The foundation and the scaffold** (item 4061). Two passes over a
building beside its sprite, each leaving the list as its own pose:

- *The foundation*, `WallOut::draw_underlays@006410a0` (`Pose::Foundation`):
  every type but `0x1a1` exactly, at alpha 0xff fading over 0.84..0.88 of
  the share (1.0 once finished), 0xff for `0x1a6` while unfinished — the
  sprite's complement. Point `(x − 0x60·b, y − 0x60·b, z)`: `b` the piece's
  `halfoffset`, or the footprint's `x` less its `constructsize` (per piece,
  `BuildRData +0x28`) where that is not 0. Slot
  (`get_construct_gpiece@00907840`): 9 for the water domain, 10 for the
  Senate's lineage, else `constructsize`, or the footprint's `y`, less 1 —
  the eleven `<CONSTRUCTION>` entries in order.
- *The scaffold*, `GraphicPieces::draw_scaffold@00907420` from
  `WallOut::draw_overlays@00641600` (`Pose::Scaffold`): started (`flags & 2`)
  and not finished; not the farm, not the water domain. Table
  `scaffold_timing[y − 1]`, `[9]` for the Senate's lineage
  (`init_scaffold_timing@008ed6c0`, `effects_graphics.xml`; `x y z` ×3; a
  slot whose name `ammo_names` lacks is dropped, matched without case). Each
  slot with `start ≤ share ≤ end`, at `(x − h + dx, y − h + dy, z + dz)`,
  `h` the **raw** `halfoffset` (one unit). Piece: the name's first
  `<SCAFFOLD>` plus 4 × the owner's age bracket (0–2, 3–4, 5+; `ages`).

**The score.** On the 99 building frames, **4,564 of 4,564 records** in
register on both halves: 877 foundations drawn by each side, 2,781 scaffold
pieces, each within one pixel (`FOUNDATIONS_IN_REGISTER`,
`SCAFFOLDS_IN_REGISTER`). The reference is the dumped record through the
listing's arithmetic, written in the test. The four types Loop 4049 named
are drawn — Angkor Wat's 8 placed, never started, by the foundation
alone — so building K **21 of 21**, **K 59 of M 59**.

**Made to fail once.** The scaffold moved back `0x60·h` for one unit:
4,301 of 4,564 scaffolds, building K 17. No `constructsize`: 4,071 of
4,564 foundations (the cities, 435–437), K 15. The `started` gate dropped
survives here — every placed record has share 0, below every window (the
least `start_percent` is 0.05) — and is held by `render`'s unit test.

**Not established.** The scaffold's element and age bracket (no dump prints
a scaffold piece), the slot's angle (parked 4061b), its `Scaffold Blend`,
the crane's pose and `draw_ammo`'s `draw_flags` gate (parked 4163a), and
the pixels. ~~The sprite and meshes themselves: the client skips both poses
(parked 4061a).~~ Item 4163 draws each with its own decoded file: a
foundation as its `<CONSTRUCTION>` sprite (type `0xc`, `draw_build`), a
scaffold piece as its `<SCAFFOLD>` model, a guy's mirror at scale 3
(`draw_scaffold`'s chad): 877 of 877 and 2,781 of 2,781 on the 99 frames
(`client::world::tests`).
- ~~**A figure's own point apart from its unit's.**~~ Settled by item
  4027: guy 0's is `Movement::body`, and the door reads it now.

## 3.6 First pixels: the terrain's condition, the client and its session trace (item 4036)

*2026-11-09, Opus 5.5. The door's `TerrainScene::corners`; `render::terrain`
and `render::figure`; the `client` crate. Checked by
`rondata::free::tests::the_terrain_s_corners_are_in_register` and the
client's own tests.*

**The terrain's binary condition (§5.4), pinned.** On the lobby's every
frame that prints a height table — run1428's `DUMP_ALL` start (frame 0),
1 and 37 (`Log::dump_heights`) — **3 of 3 frames in register, 174,243 of
174,243 corners**, each within one pixel of the original's own projection
of the dumped corner, and **174,243 heights** printing as the dump's
millionths (`TERRAIN_FRAMES_IN_REGISTER`, `TERRAIN_CORNERS_IN_REGISTER`,
`TERRAIN_HEIGHTS_AGREEING`).
- *The point.* Corner `(cx, cy)` at `(192·cx, 192·cy)`, the height the
  corner grid's single (`TerrainData::master_land_heights`) truncated to
  the integer `world_to_screen` takes; the original's side runs its own
  `world_to_screen` under unicorn on the dumped value, at the draw list's
  pinned camera and 1920×1080.
- *The door, widened.* `TerrainScene` carried the tile heights only, which
  are the mean of two corners (`find_tcoord_z`). It now carries `corners`,
  each single as its 32 bits — no float enters `sim`.
- *Not the start's copy.* Frame 0's grid is the dump's own (`build_sim`
  installs it), but frame 37's is not: the AI's first terraform moved 248
  corners round (216..221, 68..69), so 58,329 distinct points project.
  **Made to fail once**: drawing every frame from the frame-0 grid gives
  2 of 3 frames and 174,077 corners; corner (220, 68) on frame 37 sits at
  z 202, pixel (5546, 1225), against the dump's 175 at (5546, 1230).

**What the client draws** (`client`, `wgpu` 30 and `winit` 0.30, earned
here, §4.2). It opens the install and the pair's captures, stands the free
runner up and steps it to the frame, then draws:
- *the terrain*: two triangles a tile from the corners, flat in the land
  kind's debug colour (`land_key[]`: `BASELAND`, `SANDY`, `OCEAN`, `NONE`),
  a placed mountain's tiles grey, shaded by slope;
- *each figure*: its piece's `.BH3` from `unit_art`, carried out of its
  nodes into model space (`render::figure::rest_pose`: a vertex sits in its
  node's space, `vertex · node · parent · … · root`), scaled by the
  `<UNIT>`'s `scale=` times `guy_scale`, its y negated and turned by
  `d = fast_angle_to_degrees(angle) + 180` with x towards y
  (`render::figure::guy_placement`, item 4069), and moved to the draw
  list's point. `GuyOut::draw@005dc490` adds the 180
  (`__real_43340000`) and multiplies by `guy_scale`, which reads **4.8** on
  run1441's and run1559b's packets (`?guy_scale@@3MA`, `0x00c06244` by the
  linker map). Each figure is posed by default (item 4069; `--rest` for
  the rest pose): item 4067's pose
  sets each figure at its `anim` slot and `cur_time` through the `.BHa`
  its `<UNIT>` names (`render::figure::posed`). The key is the node's
  `local` transform *inside* its `.BH3` rest transform, `parent.world ·
  parent2local · local` (`HierObj::update_transforms@00536c60`). The two
  trees are paired by place, and a node the animation lacks stands at rest.
  The clock is `cur_time · 200 / 3` ms clamped to the clip, with key `k`
  reached at `times[k − 1]` (`HierObj::set_to_time@00536e30`). **The
  pose is the original's on the packet** (item 4092): `draw_unit@008e7da0`
  writes the slot and that clock to the piece's `AnimationPacket`
  (`8e8be6`), and the transform method hands `set_to_time` the clip
  `action_ids[slot]` names (`9247a4`). Run there on run1559b's packet,
  every joint of five riflemen at 3279 and 3294 lies within 0.44 model
  units of ours (`tree_points.py --pose`,
  `the_riflemen_s_pose_against_the_packet`). `draw_unit`'s remap of a slot
  to a shared clip on `Game::frame`'s clock (`8e7e0b`) is off there
  (`mem_grade` 0, `section_render[11]` 1); the clamp is `animmgr.times`
  (parked 4092b). Posed, a
  citizen's body turns with its `Bip01` key, and every figure then reads a
  quarter turn off the photograph (§5.3, parked 4067a);
- *each building*: its `<BUILD>`'s `texture=` sprite (item 4076), a quad in
  screen pixels at `render::draw::sprite_at` of the draw list's anchor,
  alpha-tested at half. It is blitted over the ground and writes the
  anchor's depth, so a nearer figure draws over it. `--zbuffer` draws the
  `zbuffer` model at the anchor instead, unscaled and unturned, as do the
  three types drawn as a model at the point;
- *the overlay* (§7.2 item 8): the frame, the last frame's milliseconds and
  the camera, from a 5×7 font in the crate, with no font dependency.
The view is the observer's (§7.2 item 7), and the tint is by owner, a
debug colour, not a team colour (§6's "After these").

**The ground's base layer (item 4127).** *A reading and one photograph;
committed before its count.* What a `BASELAND` tile draws, read from:
- *The choice.* `TerrainOut::generate_land_lists@0085f6a0` hands each
  cell's `land` (`WData +2`) and `land_sub` (`+3`) to
  `TerrainList::add_new_coord_info@0084ba10`, whose bucket is
  `TileSet::convert_land_tex_to_linear@0087ace0`: on land 0 the textures
  of the `<BASELAND>` bases before the `land_sub`th, plus
  `(cx mod 3) + 3·(cy mod 3)` over the cell's own coordinates (the
  `0x55555555` divide, checked numerically for every `x` below 300). The
  land pass binds `TileSet::get_land_textures_by_index@0087a9a0` of the
  bucket (`TerrainOut::render@008558b0`). So a cell takes the `land_sub`th
  base's file — the tileset's `<BASE>`s in file order, `land_sub` the
  crate's own (`sim::observe::TerrainScene::land_sub`, the generator's
  `fill_fertile`) — and of it the sub-tile its coordinates name.
- *The sub-tile.* `load_as_supertile@00542640` cuts a `supertile="1"`
  file 3 × 3: sub-tile `k` is the `w/3` square at column `k mod 3`, row
  `k / 3` of the loaded image, top row first as `render::tga` gives it
  (the convention the figures' textures already sample by).
- *How it lies.* `TerrainOut::generate_land@0085f8d0` gives corner
  `(x, y)` the texcoord `((x & 3)/4 + (x >> 2), (y & 3)/4 + (y >> 2))`
  (`85fb9a`–`85fc7e`, `TexArray::add` taking `u` in `xmm1`, `v` in
  `xmm2`): one sub-tile a cell, `u` along world x, `v` along world y,
  origin the cell's corner. The `_mtrl` set (`+0x448`) has the same
  coordinates; the third (`+0x1a80`) is the map's, `x/xs`.
- *What multiplies it.* The vertex colour
  `determine_land_height_color@0086c1e0` writes is `0xffffffff` on every
  path. The light (the tileset's `<LIGHTING>`, the normals) and the
  `_mtrl` stage are not drawn: the texel alone, as the colour reader's.
- *Drawn:* every `BASELAND` tile not under a placed mountain, bilinear
  over the sub-tile's mips (`render::silhouette::Paint::Textures`). A
  mountain's tiles take their own draw (§3.10),
  `SANDY` and `OCEAN` theirs and the water (§3.12); the debug draw stays
  behind a flag as the floor's.
- *Not drawn, and named at the count:* the blends between bases and
  kinds (`generate_transitions`, `create_transition_renderables`), the
  cliffs, the decorations, the weathering, the fog, the light.

The row (`RUN1559B_3279_GROUND`): of the scored tiles — at least 40
ground pixels, a ground pixel one where none of our objects draws,
outside every object's window (its pixels plus 6), every trace and every
`INTERFACE` rectangle, the terrain the nearest surface — those whose two
means over the tile's ground pixels lie within 0.05 in (r, g)/(r + g + b).
On this tree before the rule: 898,590 ground pixels, 1,109 scored tiles,
all `BASELAND` (106 under a mountain); **the debug colour's floor 92**.
**The count, on the CPU painter: 949 of 1,109** (sub 1 157 of 158, sub 2
688 of 697, under a mountain 59 of 106, sub 3 45 of 148); the mapping's
check, luminance r 0.499 over 787,770 pixels against the half-period
control's 0.083. Sub 3's dirt reads brown where the photograph does not:
the residue, parked 4127a–c. **The client draws it** (`World::
drawn_terrain`, `gpu`'s `fs_ground`): the heightfield less the textured
tiles (`TerrainMesh::build_except`), and the tiles one mesh per sub-tile;
`--debug-ground` draws the floor. `World::terrain`, which the readers
take, stays whole.

**Which base a tile draws (item 4129).** *The packet backs the choice;
the old row did not.* That row cannot tell the bases apart: every tile
drawn in base 2 scores 1,053 and in base 0 1,052. On run1559b's packet
(`tools/render/land_buckets.py`) the `pCoords` list each land-0 cell sits in
is the rule on the packet's own `land_sub` on 2,810 of 2,810 cells, and is
our bucket on 2,779. The 31 between are cells the original's
`spot_update_land` re-listed and our terrain has not (parked 4129b).
`trans_bucket` keeps the first bucket, so it is stale on 157. The row
for the choice, `RUN1559B_3279_GROUND_BASE`: a plain tile counts when its
drawn base has the best per-tile luminance r of the four bases (each laid
by the read mapping) and beats its own control, the cell index one on.
**758 of 1,003**, against 2, 274, 697 and 26 for every tile drawn in
one base. On 999 tiles the best base beats its own control. **The
losses are the blend:** 236 of 244 sit on a cell whose
`create_transition_renderables` patch lays `landindex`, the bucket of
`land_sub − 1`, over the base, and on 230 of them the photograph reads
that base. The alpha and its bitcode are unread (parked 4129a).

**The transition patch (item 4132).** Drawn from the packet's own patch
under the read alpha (`docs/FORMATS.md`), it wins 395 of 503 patched
tiles (`RUN1559B_3279_GROUND_BLEND_PACKET`) against controls at most 204.
The rule is §3.8 (item 4134), the draw §3.9.

**The projection is the camera's own.** The shader multiplies by
`to_screen_matrix` uploaded row-major (WGSL reads it transposed, so `M · v`
is the original's `v · M`). The divide by `w` and the viewport are
`world_to_console`'s. `the_gpu_s_projection_lands_on_the_draw_list_s_anchor`
holds the arithmetic to the anchor within a pixel. Depth is the client's:
camera-space z over a fixed range, since the original's near and far
planes would clip the map's far corners.

**The pinned camera is run1559b's.** That take photographed frame 3294 and
packed its state on the same frame: loc (2104.28, 30816), the default zoom,
tilt 60, dir 45, parallel, 1920×1080. run1441's packet of the same frame of
the same game held loc (3168, 29744.68), and run1559's photograph (no
packet) matches run1559b's, not run1441's. So the camera is not a function
of the frame. The first stills, drawn at run1441's camera, put every object
exactly on run1441's packet pixels (tanks (1191, 770) and (1065, 917)
against ours (1190, 765) and (1065, 908)), and ~335 px left of the
photograph.

**Compared by eye** with run1559b's photograph (kept outside the
repository; the journal names the paths):
- *Agrees:* the map's west edge, (0, 595)→(1220, 0) against the original's
  (0, 600)→(1220, 0); the mountain's extent, x 1220–1920 and y 25–385
  against 1240–1920 and 25–370; the figures' places — the lone figure at
  (1125, 300), the group by the derrick, the tanks and infantry south of
  the capital; the figures' size and uprightness.
- *Does not:* the capital is drawn at Napata's place but far smaller, since
  a `zbuffer` model is the depth proxy, not the sprite; trees are not
  drawn (forests are not in the scene); the original darkens the map's
  edge; there are no textures, no poses and no team colours.

**The session trace** (DECISIONS 114 §4; the profiler chosen here). Every
session writes the Trace Event Format JSON that Perfetto and
`about:tracing` open: a span per phase and per drawn frame, the marks
`frame zero`, `first pixels` and `quit`, and an `rss_kib` counter sampled
by `ps` every 100 ms (`getrusage` needs `unsafe`, which the workspace
forbids, so the sampled peak is a floor). `client --read-trace` reads the
clock from the file. Median of three windowed sessions (300 frames presented, the game
playing from 2994, on 2026-11-09): **485 ms to frame zero, 1,166 ms to
first pixels, p99 frame 21.3 ms (vsync), slowest 89 ms (the first, with
the terrain's upload), peak RSS 437 MiB sampled**; `/usr/bin/time -l`
read 450 MiB on the same runs. Beside it, the harness reads 540 ms to
frame zero (`HARNESS_CLOCK`) and the runner 431 ms (`RUNNER_CLOCK`).

**Diff-backed:** the terrain's corners and heights; the figures' anchors
(§3.5, unchanged). **Listing- and packet-backed:** the 180°, `guy_scale`, and the scale's
source (item 4065): the chad starts at `(1, −1, 1)`
(`GraphicChad::GraphicChad@008e22f0`) and is multiplied by `RData.scale`
(`+0x88`) × `guy_scale` unless `RData.flags` (`+0x92`) has bit 2, which
`init_unit_data@008f4290` sets only on a piece it turns into building data;
`scale` is the `<UNIT>`'s `scale=` (below 0.1 read as 1). On run1559b's own
packet piece 196 reads 0.85 and `0x1`, pieces 60 and 10912 1.0 and `0x1`, so
the client's `scale=` × 4.8 is the original's (parked 4065a). **The facing
and its sense** (item 4069). The angle is `GuyData::angle` (`+0x18`),
blended from `last_angle` (`+0x1c`) by `graphic_percent`, in the listing at
`5dcac4`–`5dcb3b` (`edi` is `this` throughout, set at `5dc4b0`); the
decompile garbles it as `_padding_`. Then `fast_angle_to_degrees`, whose
table is `angle_to_degrees` of each top byte, monotone with no sign of its
own (`a28f70`, `a28e00`), and `+ 180` into `GraphicChad::angle` (`+0x3c`),
which `draw_unit` truncates into `rotate_z` (`8e8887`). `draw_unit` adds a
per-slot offset (`0xc8c774`) only for a slot in `0..7` (`8e80dc`–`8e8119`).
The sense and the mirror come from the original's own model-to-world map,
`get_position@0090b750`, whose port (`sim::data::pivot::release_offset_at`)
is packet-backed at all 256 headings: `v = s·R(d)·P` with y negated, `d`
the same degree. The client turned the figure by `placement`'s quaternion
with no mirror. That is the same map reflected in world y through the
anchor: a sign flip of every heading. It read as a quarter turn on the
figures facing near −45° (parked 4067a), because the error is twice the
heading's angle from the x axis. `client::world::tests::
a_figure_is_turned_as_get_position_turns_a_model_point` holds the drawn map
against the port at every top-byte facing, and the unmirrored turn misses.
**Reading only, and by eye:** the rest pose, the building's model. The
pose's composition
is pinned by `figure::tests::the_key_is_applied_inside_the_rest_transform`,
which turns red with the order swapped, and by eye a posed rifleman aims
with both hands at the shoulder. The `graphic_dt` inside a frame (the client
draws at 0) and `load_hier`'s key flags (a blend skipped within 0.01 and 0.1)
are not reproduced. Not established: why our `fill` of a `rotate_z`
quaternion turns opposite to `get_position`'s `R(d)`, when the same `fill`
of the nodes' rotations poses a figure that looks right by eye. The
figure-level map is pinned to the port, and the node-level convention
has not been read again.

**What is not established.**
- **Why the camera moved between run1441 and run1559.** Both takes stage
  the same cursor pin and inputs. The camera probably moves in real time
  rather than in sim time, but nothing has been read to show it.
- **A mid-game height table.** Only the start's `DUMP_ALL` blocks print
  one, so the condition is not checked past frame 37.
- ~~**What a building draws.** The sprite and its anchor are the original's.
  The `zbuffer` model's own placement is unread.~~ The sprite and its place
  are read and packet-backed (item 4076, `docs/FORMATS.md`, "What a
  building draws"). The `zbuffer` model is the depth. Its own placement,
  and whether a figure behind the sprite is hidden by it or by the
  anchor's depth, are unread (parked 4076b).

## 3.7 The map's trees, where the original stands them (item 4080)

**How it was established.** The placement is ported from `ObjectsOut::sort_trees`' path and checked as a value
diff against the original's own functions on run1559b's packet (`tools/render/tree_points.py`, which calls
`make_tree_mask`, `place_tree` and `world_to_screen` under unicorn). A forest cell is `WData.flags & 0x20`, carried
through the door as `TerrainScene::flags`. Each cell draws its own local `Random` seeded `(ulong)(cx·7 + cy)` at its
centre, so a tree's place is a function of the map, not of the session. Slot 0 falls into slot 1 (the jump table
sends it to code with no break between them). The model is `(y + x) % 24`. The transform is `(s, −s, s)` then the
place, with no turn, and `s` = `treescale` × 3. z is `find_data_z` over the cell's corners. `rondata::artdata::
tree_art` reads the tileset's `treescale` and `graphics.txt`'s `#<TILESET>_TREE_MODELS0` list.

**Diff-backed:** on frame 3279 the runner's forest cells are the packet's 189, and all 60 trees on the 12 visible
forest cells agree on cell, mask, slot, place, z, model and pixel
(`client::world::tests::the_trees_stand_where_the_packet_stands_them`; shifted one cell, 0). 320 sweep rows (16
masks × 5 slots at four cells) agree too (`render::trees::tests::every_place_the_packet_prints_is_ours`).

**Diff-backed, the geometry:** the packet holds each visible forest cell's trees as the original assembled
them, in world space (`RTreeData::tree_cache`, `TreeCacheEntry::cache_verts[7]`, printed by
`tree_points.py`'s `cache` rows). On all 12 cached cells, our five models placed by `tree_placement` span the
same extent to within 0.001 on all six bounds. At 1.05 × `tree_scale` no cell agrees
(`client::world::tests::the_trees_are_the_packets_own_geometry`). The scale, the `(s, −s, s)` transform and the
models are therefore the original's.

**The leaves** (item 4082; each claim read on the listing, and the bold ones checked on run1559b's packet with
`tree_points.py`'s `rstate` and `section` rows).
- *The state.* `GraphicPieces::init_tree_map@008f7c20` gives the tree flush `render_states[9]`.
  `Scene::init_render_states@008ba450` sets `enables |= 0x20084`, `alpha_func` Greater, src/inv-src alpha blend
  and stage 0 `alphaop MODULATE`; it leaves `alpha_ref` at `RenderState::init@00535ba0`'s 0. **Packet: enables
  0x20087, zfunc 3 (LessEqual), alpha_func 4 (Greater), alpha_ref 0, cull_mode 2.**
- *The bits* (d3dgl's listing). `set_depth_stencil_state` (DLL `0x1002b5f0`): depth test `enables & 1`, depth
  write `(enables & 3) == 3`. `set_shader_options` (`0x1002db30`): `enables & 0x80` is the alpha test at
  `alpha_ref / 255`, `TEST_GREATER` being `clip(a > ref)`. So the trees depth-test, depth-write and alpha-test at
  0. **Land units' `render_states[3]` reads 0x20287**: they depth-test and write too.
- *The cull.* `set_rasterizer_state` (`0x1002b410`) sets `FrontCounterClockwise` and maps cull_mode 0/1/2
  through `0x100dd094` to NONE/BACK/FRONT, so mode 2 culls a triangle wound counter-clockwise on the target
  (`render::silhouette::Cull`).
- *The textures.* `#<TILESET>_TREE_TEXTURES0`'s first seven lines are the bark and six leaf layers
  (`TreeArt::textures`). **Each of `RTreeData::tree_textures[7]` holds one mip level.** `ImageIO::decode_tga`
  writes a bottom-origin file from its last row up, so v = 0 is the image's top row.
- *The sections.* `TreeTransformMethod::make_index_table@00920df0` names a model's nodes into six slots: `root`
  and `Trunk` the bark, `Bottom_1_*` texture 1 or 2 by A/B, `Middle_1_*` and `Middle_2_*` 3, 4 or 5 by A/B/C,
  `Top_1_*` 6 (`render::trees::slots`). **Diff-backed: compared without case, all 12 cached cells agree with the
  packet's per-section vertex and index counts; with case, none; each texture shifted one on, none**
  (`client::world::tests::the_leaves_are_the_packets_own_sections`).
- *The order.* `Scene::render@008b3270` runs `pop_all_zoomed_in@00655e70`, which draws and flushes the figures
  and then calls `flush_trees` (when `draw_flags[1] & 4`). The trees follow the figures under the same LessEqual
  test, so a figure in front of a tree is one nearer than it.

**The draw.** Each tree is its model at rest. The window draws it textured (item 4173, below; flat green before), and `--no-trees` leaves it out. In
candidate C's key buffer a tree is rasterized after the sprites, its fragments cut by the leaves' alpha and cull
above (`Leaves::Alpha`, the default), so it hides what stands behind it, and it is scored by §5.3's tree rule. A tree is not a draw-list piece, so K's M (§5.4) is not widened and K holds at 59 of 59.

**The count** (§5.3): **29 of 33 → 25 of 29** as solid cards (4080), **→ 26 of 29** with the leaves cut (4082):
0/6 0.348 → 0.513. Before the count, on our buffers: of the five figures' pixels a tree's triangle covers, the
leaves' texels cut 12, all 0/6's, and on every covered pixel the tree is the nearer, by 190–1,300 camera units.
So four figures by the woodcutter, which the photograph shows in front of the pine, stay hidden (parked 4082a).
The rejected readings, `TreePixels::Scene` and `Leaves::Solid`, read 26 and 25 of 29. **→ 30 of 33** with the
figures' silhouettes (4085, §5.3 "The silhouettes"): on the pixels our pine covers, the photograph holds neither the
figure's own colours nor the leaf's but `render_states[1]`'s `tfactor`, the owner's unit colour at 128/255 (4103), over the leaf —
red on 94–97 % of each of the four figures' covered pixels and 4–22 % ten pixels over, the colour's share by least
squares at a mode of 0.5–0.6 — and the pine stands where ours does. The figure stands behind the tree in the
original too; it is drawn over it. The rejected readings with the silhouettes: `Silhouettes::None` 26 of 29,
`TreePixels::Scene` 29 of 33, `Leaves::Solid` 29 of 33.

**The trees' own colour** (item 4173; `client::trees`). *The draw, read off run1566b's card trace* (the pass after
3279, §3.14): the trees are one draw a texture, the bark and the six leaf layers of `#EARLY_AUTUMN_TREE_TEXTURES0`
(`tree_Tea_*`), each stage 0 `ADDSIGNED(TEXTURE, CURRENT)`, stage 1 `MODULATE(CURRENT, TFACTOR)` at a white factor,
alpha `MODULATE(TEXTURE, CURRENT)`, enables 0x20087, `alpha_func` Greater at 0, options `ALPHA_TEST VERTEX_COLOR_DIFFUSE
TL_FAKE_LIGHT` and no `LIGHTING`. *The vertices, off run1559b's packet:* every `tree_cache` entry's vertex is eight
dwords, xyz, the `D3DCOLOR` `0xff808080`, two texcoords (`TREE_DIFFUSE`). *The shader* (`ps_global.hlsl`, read):
`TL_FAKE_LIGHT` without `LIGHTING` raises the stage colour to 1.9 and multiplies by the HDR 5; the tone map follows
(§3.14). d3dgl's argument table (`0x100fe0c8`, built at `0x10002730`) writes `TA_CURRENT` as `COLOR_CURRENT` on
either stage, and the shader starts `outColor` at 1; Direct3D's `CURRENT` on stage 0 is the diffuse.

*The rule, committed before the count.* A tree is scored when its own pixels in the key buffer — `TreePixels::Scene`,
the leaves' alpha test and cull, the silhouettes taking a figure's pixels — number 40 off the photograph's
`INTERFACE`. No footprint is read off the photograph: the trees stand in register (4080, 4085), so each own pixel is
read against the photograph's pixel under it, by the colour reader (`HUE`) and the luminance reader (`LUMA_READER`),
on `paint_cut`'s texel coloured four ways (`TreeColour`): **Flat**, the client's green as its sRGB target stores it
(the count as drawn, pinned first; expected 0); **Texel**, `ADDSIGNED` with the vertex diffuse, `t + 0x80 − 127.5`;
**FakeLight**, Texel through `TL_FAKE_LIGHT` and the tone map at the clamp (`fake_light`); **Literal**, `CURRENT`
the initial 1, `t + 127.5` clamped, through the same. `RUN1559B_3279_TREES` is the colour reader's count; the drawn
candidate is the one most trees agree with by colour, a tie broken by the luminance count, a second tie keeping
Texel; `RUN1559B_3279_TREES_LUMA` is the luminance count beside it. Made to fail at run1441's camera. The painter
and `client::gpu` bind the winner's texel alike.

*The count* (`client::trees::tests`; **diff-backed**: the card's tree state, `the_trees_draw_on_the_card`). Of 56
trees scored: Flat **0** (luminance 1), pinned first; Texel 21 (56); **FakeLight 37** (54); Literal **0** (0). So
FakeLight is drawn, `RUN1559B_3279_TREES` **0 → 37 of 56**, `_LUMA` **1 → 54**; run1441's camera reads 0 of 57. The
photograph kills `CURRENT` = 1 on stage 0: whatever d3dgl's table writes, the tree takes the vertex diffuse there. Of
FakeLight's lit pixels 93,177 of 162,307 agree; the opaque texels' 37,408 of 50,170, the translucent ones' (alpha under
255, 69 % of them) 55,769 of 112,137. **On the client's still** (§3.15's harness) the row is the painter's, 37 and 54,
every tree within 5 % of its painted agreement (`RUN1559B_3279_TREES_STILL`). The client draws the trees after the
figures' soup and before the guys pass, LessEqual with depth written, no cull (mode 2 culls nothing on this frame).

*What it moved.* A silhouette over a tree now blends over the tree as drawn: `RUN1559B_3279_COLOUR` **18 → 17**, a
named regression. 0/1's 159 behind pixels agree 81 → 66 (80 needed); composed before the tone map, 11. On the still
`_COLOUR_STILL` holds 14 (0/1 30 → 64, the painter's 66), `_TRACED_STILL` **31 → 32**: 0/1's ring no longer holds a
silhouette's red over flat green (4171a), 0.159 → 0.850.

**The shadows (item 4182; read on the listing, the rule committed before its count, `53f6cfdfd`).** A shadow is
a decal. `DecalTransformMethod::execute@00923a70` lays a flat world-space quad from a `DecalRDataStruct`, and
`DecalFlushMethod::execute@00923fd0` draws it under an identity transform. Both axes are turned by the angle
(truncated, mod 360). Then `x = loc.x + (±hw·r1.x ± hh·r2.x − xoffset)·xstretch`, `y = loc.y ± hw·r1.y ±
hh·r2.y`, `z = loc.z`. The texcoords cover the whole texture, or a quarter for corners 0–3 (`render::decal`).
- `draw_tree_shadow@00905910` runs at every placed tree (`sort_trees`): 192 × 192, angle 0, stretch 1.5, offset
  38.4, corner `(int)(x + y) % 4` of the tileset's eighth `TREE_TEXTURES0` line.
- `draw_unit_shadow@008e8fa0` runs from `GuyOut::draw_shadows@005dc070`: `shadowwidth`/`shadowheight` × 3, the
  `<SHADOW>` that `shadowtype` names, angle `180 − fast_angle_to_degrees`, stretch 1.25, offset 0.25, at the point
  plus `shadow_offset`. On run1559b's packet 31 of 32 offsets lie within 11 units (`tree_points.py --guytree`).
- On run1566b's pass after 3279 they are draws 116–118, 124 and 119. Each is `BLACK` (alpha 255), alpha
  `MODULATE` texel and vertex, enables 0x20084 (no depth test). They come after the land, roads, borders and
  tracks, and before the mountains (121).

*The rule.* A photograph pixel is under a shadow when three things hold. It lies in a projected decal of the frame
(every tree `tree_list` stands, every drawn guy whose piece names a `<SHADOW>`). The decals' `1 − Π(1 − α)` there
(level 0, nearest) is at least `1 − 1/1.2`. And `ground_why` calls it ground or window. A tile scores on 40 such
pixels. Both sides' means over those pixels are read by `HUE` and `LUMA_READER`, the still against the
photograph. *Counted* (`client::shadows`), luma, hue at 92 throughout:
- **The before draw, `World::shadows` off: 41 of 105.** Tree tiles 13 of 68, figure tiles 28 of 37.
- **Blended after the tone map, as a plain alpha blend: 24.** Ours is 0.79 of theirs.
- **Taken before it, `T(T⁻¹(g)·keep)` (`client::gpu`'s keep map and shade pass, §3.14's curve): 89.** Trees 61 of
  68, 1.29 → 0.93. This is drawn.
- **The guys' decals as read: 66.** Figure tiles 5 of 37: not drawn (4182c).
- **Mirrored, the made-to-fail: 59 of 108.**

**What is not established.**
- ~~The leaves' alpha~~: read and applied, above (item 4082).
- ~~Why four figures the photograph shows in front of the pine stand behind it in ours~~: they stand behind it in
  the original too, and are drawn over it as silhouettes, `render_states[1]` (item 4085, §5.3 "The silhouettes").
- Which land units take method list 1 rather than 0, 2 or 4 of `init_land_unit_map`: every figure on frame 3279's
  photograph that stands behind a tree or the woodcutter is drawn so, and the selector is not read.
- ~~The order of figures among themselves in the guys pass~~: read (item 4087). `UnitOut::draw@00615b80` keys
  `Tree<Guy*,float>` by `1e7·type + 1e4·cur_anim + 100·cur_time − 1e-5·(x·m8 + y·m9)` (a guy on a tile with
  flags `0xc`: the last term alone), and the pass walks it in order: a batch by unit type, not a depth sort. On
  run1559b's packet the riflemen (type 110) precede the tanks (246). Every list writes depth with `zbias` 0, so
  the order decides no opaque pixel; whether a silhouette (list 1, not written) lies over a nearer figure turns
  on the type ids.
- ~~A vehicle's tilt to the slope (item 4087; parked 4087a)~~: read and drawn (item 4089, §5.3 "The tilt";
  `render::slope`). Not established: the draw clock the ease runs on, which the packet puts at about one draw
  per 11–16 sim frames (parked 4089a); the client draws the ease's fixed point.
- The blend: the key buffer keeps a fragment whole where its alpha passes; the original blends it, which moves
  colour, not a footprint. Item 4173 measures what it costs: the translucent texels part 50 %, the opaque 25 %
  (parked 4173a).
- ~~The trees' shadows: draw 119 of the pass, `tree_t_shadows.tga`, `SELECTARG2(TEXTURE)` at enables 0x20084, the
  dark patch under each tree on the photograph; not drawn (parked 4173c)~~: read and drawn, the figures' with them
  (item 4182, "The shadows" above).
- What the guys' decals miss. As read they darken the row's figure tiles past the photograph (28 → 5 of 37,
  0.99 → 0.82), and they are not drawn (parked 4182c). Also not established: the city's paved base, which is
  `RoadsOut`'s (4182a), and the `shadow_offset` (4182d).
- Why 0/1 behind the pine agrees with the raw texel under its silhouette (81 of 159) and not with the drawn leaf
  (66), nor with the blend before the tone map (11) (parked 4173b).
- Why tree models 9–15 agree at 0.71–0.81 and the rest at 0.34–0.63, the photograph greener on the latter's opaque
  texels (parked 4173d).
- The depth of a building's sprite against a tree. Ours puts the anchor's flat draw-list depth against the tree's
  per-pixel depth. The original's sprite may carry its `zbuffer` model's depth (parked 4076b).
- The tree's idle animation, which is not posed.
- The section name `#<TILESET>_TREE_MODELS0`. It is a reading, checked by the count of models (24, all of
  which decode), and since by the packet's geometry on 12 cells.
- Whether cutting a tree moves `terrain_version`.

## 3.8 The transition patch (item 4134)

**What it is.** A land-0 cell next to a cell one `land_sub` below it lays a second
texture over its base: the base of `land_sub − 1`, through one of 14 alphas, at texcoords that
pick a quadrant of the alpha and turn it. The port is `client::patch`. From the cells'
`(land, land_sub)`, the tileset's `COAST_BASE` and `BASELAND` count and the texture fractal, it makes
the edge masks and then the choice. It draws nothing. The oracle is run1559b's packet, printed whole by
`tools/render/patches.py`: every patch, every cell, both mask arrays, and the tables.

**The rule, as ported.**

1. *The queue.* `mark_nodes` runs `mark_trans_flags` on every cell, row-major, on masks that start
   at zero. A cell is queued when one of its eight neighbours lies one step below it under
   `TileSet::is_combo_acceptable`. Within a land that means the same land, sub one less.
   `create_transition_renderables` visits only the queued cells.
2. *The masks.* Each cell edge holds a 9-bit word with three 3-bit fields: block (shift 6), connect
   (shift 3) and fill (shift 0). `edge_mask_x` holds the vertical edges and `edge_mask_y` the
   horizontal ones (type record `+0x495c` and `+0x4960`; shipped at `+0x40` more, like
   `transitions`). `create_nodes` allocates `(w + 1)·h` of the one and `(h + 1)·w` of the other,
   zeroed; a write lands only where its index is a cell, so the edges at `x = w` and `y = h` stay
   zero. Setting a value in the block field also clears the same bits of the connect field
   (`set_edge_x`, `set_edge_y`).
   - *`mark_trans_flags`, per cell.* The first orthogonal neighbour (N, E, S, W order) one step
     below queues the cell. `add_unique` refuses a cell already queued, so the later sides do
     nothing. It connects all four edges (connect 7), runs `check_for_tendrils`, and if that
     blocked nothing, blocks the edge facing that neighbour (block 7). A cell with no lower
     orthogonal neighbour but a lower diagonal is queued too: it ORs connect 7 into four
     `edge_mask_x` words (its two sides, the left side again and the left side of the cell below,
     exactly as the decompile prints and as the packet agrees), then runs `check_for_tendrils`.
   - *`check_for_tendrils`.* With the eight lower-neighbour flags it does three things.
     - Each corner whose two sides are not lower takes a fill bit on its two edges, or the block
       bit if its diagonal is lower.
     - Two opposite lower sides with the other two not lower block both other edges with 5.
     - Three lower sides block the fourth with 5 and its fill bit 2, and connect the two edges
       beside it on the neighbour across it.

     `neighbor_count` is called and its result dropped.
3. *The codes.* `build_trans_flags` reads one 12-bit code per field. The top edge goes in bits 9–11,
   the right in 6–8, the bottom in 3–5 and the left in 0–2. The bottom and left groups are
   bit-reversed through `bit_reversal`.
4. *The checks.*
   - `check_blocks` opens a full group whose neighbour lies one step below. It closes an open
     group whose neighbour lies one step above. Then each corner bit drags its neighbour across the
     corner.
   - `check_connects` gives an empty group facing a lower neighbour the *complement* of the block
     code's group, `connect |= ~(block & g)`. The listing's `and; not; or` confirms the whole-word
     complement. It clears a set group that faces any other cell.
   - `check_fills` clears a group facing another cell, and clears whatever the block code holds of
     it.
   - When nothing blocks, `check_for_diagonal` runs. Two orthogonal neighbours equal to each other
     and not to the cell set a corner shape. Off the map counts as land 0, sub 0. With no such pair,
     a lower diagonal neighbour sets one.
   - Finally an empty side takes its middle fill bit, and two empty sides that meet take their
     corner.
5. *The pick.* `Transitions::init` names each alpha's eight orientations from its file name, a
   12-digit binary code: the code, its three rotations by group, and the four bit-reversed mirrors.
   `Transitions::find_map` admits an orientation when all of these hold:
   - it shares no bit with the block code;
   - it holds every fill bit;
   - it has a bit in each group the connect code touches;
   - it is zero when all three codes are zero.

   The admitting alphas are the candidates, in table order. The fractal picks one at
   `⌊frac · (n / 100)⌋`, held below `n`, and the same rule then picks an orientation among the
   admitted ones. All of this is single-precision arithmetic, done here in `sim::num::Single`.
6. *The texcoords.* Orientation 0–3 turns the cell's own `trans_texcoords0` point as
   `(1 − u, 1 − v)`; orientation 4–7 turns it as `(u, 1 − v)`. The switch's jump table shows which
   case gets which input; the decompiled bodies hide it. The quarter turns per orientation are
   2, 3, 0, 1, 2, 1, 0, 3, turning `(u, v) → (v, 1 − u)` about the centre. The result is halved and
   moved into quadrant `⌊frac · 0.04⌋`. Quadrants 1 and 2 add ½ to u; quadrants 2 and 3 add ½ to v.

**What the packet's diff backs** (`client::patch::tests`):

- *Stage 0, the fractal.* `Fractal::init(61, 61, 0, 2, World.seed)` gives the packet's `frac` on all
  2,064 patched cells, on the start dump's `World.seed`.
- *The orientations.* From the 14 file names, the eight codes are the packet's on every alpha.
- *Stages 2 and 3, on the packet's own masks* (`RUN1559B_3279_PATCH_ON_ITS_MASKS`). Of the 2,810
  land-0 cells:
  - 1,509 of the 1,512 drawn patches take our alpha, and all 1,509 take our texcoords at all 25
    vertices;
  - all 113 emptied patches are "none" under our rule;
  - all 1,185 unpatched cells are "none" too.

  Each stage was perturbed once:

  | perturbation | drawn alphas | texcoords |
  |---|---|---|
  | the seed one on | 1,506 | 584 |
  | one patched cell's sub set to 0 | 1,508 | 1,508 |
  | the orientation mirrored | 1,509 | 0 |

  The texcoord rule was found against the packet: the flip first (799), then the turn's sign
  (1,509).
- *The counts.* The packet holds **1,512** drawn land-0 patches and 113 without geometry, 1,625 patch
  records in all. Item 4132's "1,399 drawn" took the 113 off twice.
- *Stage 1, the whole chain from the cells alone* (`RUN1559B_3279_PACKET_PATCH`). It marks the masks
  from the packet's `(land, land_sub)`, then runs the checks, the pick and the texcoords. The result:
  - **all 1,512** drawn patches take our alpha and our 25 texcoords, the three cells above
    included;
  - all 113 emptied patches are "none";
  - all 1,185 unpatched cells are "none".

  The queue is exactly the cells step 1 names. Clearing any one field of our masks (connect, block
  or fill) drops the drawn count.
- *Our masks against the packet's* (`RUN1559B_3279_PACKET_MASKS`): 3,402 and 3,375 of 3,660 edges.
  This is not whole, and is not meant to be: the packet's masks carry the spot updates that came
  after the patches were laid.
- *On our own cells* (`RUN1559B_3279_PATCH_ON_OUR_CELLS`): our mapgen's `land_sub` lags the packet's
  on 31 cells (4129b). Here 1,471 of 1,512 drawn patches take our alpha and texcoords, 79 of 113
  emptied patches are "none", and 1,185 of 1,185 unpatched cells are "none". All 75 cells that part
  are one of the 31 (27) or next to one (48).

**Where the 113 come from.** `Terrain::spot_update_land` handles a land change mid-game. It resets the
four edge masks of each cell in its list to `0x38`, empties that cell's patch (counts, bucket and
texture zeroed), re-marks the cell, and reruns the pass. A cell it empties that is not re-queued
keeps an empty patch. Our rule draws nothing there, and neither does the original.

**What parts on the packet's own masks, and why.** Three cells part there: (41, 15), (56, 25) and
(45, 28), each carrying alpha 13. On those masks the candidates are alphas 1, 7 and 13, and the
fractal picks index 1, 0 and 0. On masks marked fresh from the same cells, all three take 13. So
the masks went stale, not the patches. `spot_update_land` sets an updated cell's edges to `0x38`
and re-marks it. Because the queue is never cleared, a cell already queued skips its sides, and an
edge it shares with a neighbour changes after that neighbour's patch was laid. The parting edges
cluster on the 31 cells 4129b found moved. Further out they are whole blocks of bare connect
words, which are updated regions whose change left no net `land_sub`.

**What rests on the reading alone, and what is not established.**

- The history: which cells `spot_update_land` updated, and in what order. It is not on the packet,
  so `RUN1559B_3279_PACKET_MASKS` stays short. Nothing the pick reads depends on it.
- ~~Two inputs come from the packet rather than our data~~ — **answered by item 4139** (§3.9):
  `COAST_BASE` 3 and `BASELAND`'s four bases (`TileSetData +0x62c`, `+0x208`) are read through
  `rondata::mapgen::tileset_groups` and asserted equal to the packet's. The cross-land branches
  that use them are reached on the land-1 and land-2 borders and held only in aggregate by the
  count above.
- `Transform::rotate_z`'s direction; the packet fixes it.
- Land 1 and land 2 patches (319 and 120). ~~The draw (parked 4132a, 4132b)~~ — **answered by item 4139** (§3.9).

## 3.9 The transition patch, drawn (item 4139)

**What it is.** The patch §3.8 chooses, laid by the CPU painter and the client.
`client::ground::ground_patch` gives each patched `BASELAND` tile not under a mountain its
two triangles. Each corner carries a texcoord into the patch's base, which is the base of
`land_sub − 1` at the cell's own sub-tile, and a texcoord into the alpha, which is the patch's
grid point in eighths. `our_patches` runs the choice on our own cells. It takes the tileset's
`COAST_BASE` and `BASELAND` count from `rondata::mapgen::tileset_groups`, which equal the
packet's 3 and 4 (parked 4134b, answered), and the texture fractal on the start dump's
`World.seed`. `World::drawn_terrain` groups the triangles into one `PatchMesh` per (patch's
base, alpha). `gpu`'s `fs_patch` draws them after the base.

**The pass, as read.** `TerrainOut::render`'s `showtrans` block draws per (alpha, `landindex`)
batch. Its state is the `RenderState` at `TerrainOut +0x442c`, filled in `one_time_init`.
The fields are named by the type record and the values by the PDB's enums.

| field | stage 0 (the patch's base) | stage 1 (the alpha) |
|---|---|---|
| colorop | MODULATE (TEXTURE, DIFFUSE) | SELECTARG2 (CURRENT) |
| alphaop | SELECTARG2 (DIFFUSE) | MODULATE (TEXTURE, CURRENT) |
| address u, v | CLAMP | CLAMP |
| mag, min, mip filter | POINT | POINT |
| mipmap_bias | — | −1.5 |

The blend is SRCALPHA over INVSRCALPHA, after the base. So the output is the patch's base × A
plus the frame × (1 − A). 4132's two assumptions are kept: A weights the patch's base, and the
level is 0. λ, from each scored tile's own footprint, runs from −1.27 to −0.47; less 1.5 and
rounded, it gives level 0 on all 503. One thing is corrected: the alpha is read with point
sampling, clamped, not bilinear. `RUN1559B_3279_PATCH_READS` counts each read with our choice
on the packet's cells:

| read | wins of 503 |
|---|---|
| A, bilinear wrapped (4132's) | 395 |
| A, nearest texel, clamped (the pass) | **402** |
| 1 − A, either | 120, 122 |

**What the diffs back** (`client::ground::tests`).

- `RUN1559B_3279_GROUND_BLEND`: our choice wins **402 of 503** on the packet's cells and 402 on
  our own. The controls reach 15, 32, 5 and 44. Against 4132's 395 it gains 24 tiles and loses 17.
  127 of the 503 carry an emptied patch, and every read draws the base alone there; all 127 are
  asserted to be ties. On the 376 drawn tiles alone (`RUN1559B_3279_GROUND_BLEND_DRAWN`) it wins
  **282**, against 275 for the packet's own patch and a best control of 147. The losses lean to
  the cell's sides and corners, where A is mixed.
- `RUN1559B_3279_GROUND` is re-pinned with the patch laid: **949 → 1,051 of 1,109**. 103 tiles
  are gained and one is lost, (11, 189). Sub 3 goes from 45 of 148 to 148 of 148. Of the 58 left,
  47 are a mountain's own draw, 8 sit inside one base and 3 on a blend. Drawing with 1 − A drops it
  to 886. The fractal's seed one on leaves it at 1,051. *This row sees that the patch's base is
  laid, not which alpha.* Every tile drawn in base 2 still scores 1,053. The base alone stays
  pinned as `RUN1559B_3279_GROUND_BARE`.
- `RUN1559B_3279_GROUND_BASE` is re-pinned with the rule's draw carrying the patch: **758 → 951
  of 1,003**. 193 are gained and none lost, against the one-base controls' best of 697. The seed one
  on drops it to 923, and the luminance r from 0.618 to 0.595. 4129's base-only 758 stays pinned
  as `RUN1559B_3279_GROUND_BASE_BARE`.
- On the card, frame 3280's still (`client --still`) differs on 323,043 of 2,073,600 pixels from
  the same frame with the patch's uploads emptied. That shows the pass draws. It is not a score.

**What rests on the reading alone, and what is not established.**

- Stage 0's own filter and addressing: the state reads POINT and CLAMP. Both the painter and the
  client sample the patch's base as they sample the base, bilinear and wrapped. This is unmeasured
  (parked 4139a).
- The client's blend runs in the sRGB target's linear space. The original's card blends the
  stored values. The GPU frame is not scored against the painter (parked 4139b).
- The patch on land 1 and land 2 (319 and 120 on the packet), and every pass after it.

## 3.10 A mountain's own draw (item 4141)

**What it is.** A placed range drawn as `TerrainOut::render_mountains`
draws it, over the ground §3.9 lays beneath it, by the CPU painter
(`client::ground::MountainArt::over`) and the client (`gpu`'s
`fs_mountain`). Before it, the painter and the client laid the debug grey
on every tile under a mountain.

**The art, as read.** `render_mountains` walks the sixteen template slots
and draws each range's mesh in two passes:

| pass | stage 0 | stage 1 |
|---|---|---|
| ring, first | `TileSetData::mountain_alpha`: the tileset's `<MOUNTAINS>` `ALPHATEX` | `MountainRangeOut::ring_alpha`: the template's `RING_ALPHA_TEX` |
| main, second | `TileSetData::mountain_base`: `BASETEX` | `MountainRangeOut::mountain_alpha`: `MAIN_ALPHA_TEX` |

The two `MountainRangeOut` fields are named by the type record (+0x1d0,
+0x22c, behind the virtual base's 4). `TileSet::init` names the files.
`rondata::artdata::mountain_textures` and `rondata::mountains::alpha_files`
read them. On early autumn, stage 0 is `t_mountain_plain.tga` and
`t_mountain_blend.tga`, both 256 × 256. Stage 1 is 128 × 128 per template.

**The texcoords.** `MountainRange::init` lays a vertex at every fourth
pixel of the template's image where the alpha is lit. That is the tile
corner `4 · (tile − 4 · loc) + 128`. It gives the vertex two texcoords:

- stage 0: the pixel times 0.0625, so one repeat per cell, from the
  template's origin;
- stage 1: the pixel less the lit bounding box's near corner, over its
  extent, so the alpha is stretched over the box.

The listing settles both. The decompiler mangles the arguments here. The
constant 0.0625 is read from the PE. `TerrainOut::fill_mountain_data`
copies both sets into the mesh and gives every vertex the colour white.

**The pass, as read.** The state is `MountainsOut::mountain_blend`,
filled in `Mountains::init` over `RenderState::init`'s defaults. Its
fields are named by the type record and its values by the PDB's enums.

| field | stage 0 | stage 1 |
|---|---|---|
| colorop | MODULATE (TEXTURE, DIFFUSE) | ADDSIGNED (TEXTURE, CURRENT) |
| alphaop | SELECTARG1 (TEXTURE) | MODULATE (TEXTURE, CURRENT) |
| address u, v | WRAP | CLAMP |
| mag, min, mip filter | POINT, POINT, POINT | LINEAR, LINEAR, POINT |

The blend is SRCALPHA over INVSRCALPHA, and the alpha test is GREATER
against 0. So each layer is `t0 + t1 − ½` per channel, at A = `t0.a ·
t1.a`, laid over what is already drawn.

**What the diffs back** (`client::ground::tests`, frame 3279's
photograph, the 106 scored tiles under a mountain).

- `RUN1559B_3279_MOUNTAIN_VALUES`, before any reading: all 106 are on
  one range, template 14 at cell (4, 36). Under the row's bar the grey
  agrees on 59 and the ground as if no mountain stood there on **76**.
  `BASETEX`'s mean agrees on 60 and `ALPHATEX`'s on 0. Nearest by tile:
  41, 59, 6 and 0, with no tie.
- `RUN1559B_3279_MOUNTAIN_READS`, each read beside the ground, the best
  control:

  | read | agree of 106 | nearer than the ground |
  |---|---|---|
  | **the pass as read** | **105** | 85 |
  | the main pass alone | 105 | 91 |
  | 1 − A | 97 | 90 |
  | stage 1's colour ignored (no ADDSIGNED) | 62 | 35 |
  | the alpha unstretched (over 256) | 88 | 62 |
  | over the grey, not the ground | 98 | 77 |

  The read is kept: ADDSIGNED, the stretch, A as weight and the ground
  drawn beneath. *The count does not tell the ring pass apart*: the main
  pass alone ties it.
- `RUN1559B_3279_GROUND`, re-pinned with the draw: **1,051 → 1,097 of
  1,109**. The 106 go from 59 to 105, with 46 tiles gained and none lost.
  (12, 141), on the range's edge, is the one left. The grey's row stays
  pinned as `RUN1559B_3279_GROUND_GREY`.
- `the_client_draws_every_tile_once`: the client lays the ground under
  every mountain tile and each range's tiles twice, ring then main,
  after the patch.

**What rests on the reading alone, and what is not established.**

- Stage 0's filter: the state reads POINT. The painter and the client
  sample it bilinear and on level 0. This is unmeasured, as parked
  4139a is for the patch.
- The diffuse. The vertex is white and the state's `tfactor` is set to
  `WHITE`. `TerrainOut::render` generates normals for the mesh, and
  whether lighting reaches the diffuse is unread.
- `Mountains::init` adds a pixel-shader situation. Stage 2 binds
  `mountain_mtrl` (`BASETEX`'s `_mtrl` sibling) where the card answers.
  Which path ran under DXVK is not established. Neither is drawn.
- `TileSet::init` sets the tileset's tint on both stage 0 textures, and
  the tint is not applied.
- The range's own vertex heights, the mid-zoom mesh and level of
  detail, and the snowcap underlay (`CoordInfo::add_geometry`). The draw
  takes our terrain mesh's tiles.
- How the original draws the ground under a mountain cell. The count
  backs only that a ground is there (105 against 98 over the grey).
- The client blends in the sRGB target's linear space (parked 4139b). The
  GPU frame is not scored. Item 4172 moved `fs_mountain`'s ADDSIGNED onto
  the stored bytes: run in linear space it drew the body at Y 52 on the
  still against the photograph's 142, where the painter reads 0.83–0.92;
  after, 127. The layers' blend over the ground is still linear.

## 3.11 The clock, under its ceilings (item 4161)

**The session.** `client --lobby --headless --frame 0 --play --frames
600`: Great Lakes from our own start (`World::open_game(.., Game::Lobby)`,
which builds `FreeRunner::from_lobby` from run53's lobby choices), drawn
off-screen at 1920×1080 at the pinned camera. Each frame steps one tick,
draws, and waits for the GPU (`Gpu::wait`). The wait stands in for the
window's present, so the frame times count work and no vsync. The pinned
camera sees the human's capital, its citizens and the trees at frame 0.
A frame's phases are spans nested inside its `frame` span: `step`, `scene`
(the draw list, sprites, figure meshes and trees), `ground` (the heightfield
and its textured meshes), `gpu` (uploads, passes, submit) and `wait`.
`render::clock::Clock::read` names the slowest frame and the longest phase
inside it.

**The readings** (eight sessions on 2026-11-12, release, with three other
lanes gating beside them, load average 14 to 20):

| reading | range | median | ceiling |
|---|---|---|---|
| launch to frame zero | 635 to 1,220 ms | 705 ms | 1.5 s |
| p99 frame | 4.4 to 11.8 ms | 5.2 ms | 20 ms |
| slowest frame | 87 to 219 ms, frame 1 in all eight | 100 ms | 400 ms |
| peak resident set (sampled) | 239 to 256 MiB | 242 MiB | 320 MiB |

**The worst is the first frame, and its cause is first sight.** Frame 1
decodes the model, textures and sprite of every piece it shows. That is
`scene`, 36 to 57 ms, the longest phase in six of the eight sessions. It
also builds the ground's meshes (`ground`, 14 to 27 ms) and uploads both
(`gpu` and `wait`, 36 to 144 ms). Frame 2 still decodes some (`scene` 11
ms), and from frame 3 a frame takes about 4 ms. The phase that leads frame 1
swings between `scene` and `wait` under load. So the pin holds the frame
(`Ceilings::slowest_at`) and names the cause in words, never by a phase
name.

**What this item changed.** Before it, `--lobby` loaded the install's tables
twice and built the pair's runner from run1429's dumps, only to drop it. The
still read 1,474 ms to frame zero, 792 ms of it opening the pair, and 427
MiB. `World::open_game` loads once and builds only the game it names: 705
ms and 242 MiB. The pair's runner, holding its dumps, was most of the
memory.

**The pins.** `render::clock::CLOCK` holds the four ceilings and the
slowest frame. `crates/client/tests/clock.rs` runs the client binary as a
process of its own, so the clock is the client's and not the test
harness's. It reads the session's trace and fails on any ceiling the
session breaks (`Ceilings::broken_by`). The queue's `Clock:` line is
`rondata::diff::floors::clock_line` of the same constant. **Made to fail
once:** p99 at 1 ms and the peak at 200 MiB read "p99 frame 5.41 ms over its
ceiling 1; peak RSS 238.78 MiB over its ceiling 200". `render::clock::tests::
a_ceiling_under_the_reading_breaks` holds each ceiling one unit under a
fixed trace's reading. A ceiling falls when the client gets faster and rises
only on a named regression.

**Confidence.** These are measured times on one machine under a shared
load, not a diff: nothing of the original is compared. The resident set is
`ps` sampled every 100 ms, a floor on the true peak (§3.6).

**What is not established.**
- **Seconds to lobby.** DECISIONS 114 §4 names it, and the client has no
  lobby screen to time. The lobby's choices are read from run53's dump.
- **The windowed session.** Its frames wait on vsync (§3.6's 21.3 ms
  p99), so it is not the ceilings' session. A window over the lobby is
  unmeasured here because a window opened in a gate could take a capture
  lane's clicks.
- **The ground's art on the lobby** is the pair's start's: run1428's
  tileset and seed, run53's `game_random seed 12345`. That the two maps'
  tilesets agree is read off the picture, not checked.
- **What broke the ceilings in a gate on 2026-11-12** (gate-4171: p99
  75.10 ms, frame 420's `gpu` phase 129 ms). Not this repository's own
  draws: beside one `--still` loop, the client's test binary on four
  threads, or four still loops, the session's p99 read 4.75 to 5.89 ms
  with a host-wide lock on off-screen devices and without it (the pass by
  supply; the lock is unmerged). The test runs a second session when the
  first breaks and fails only if both do.
- **Whether first sight belongs before frame zero.** Decoding every piece
  the lobby could show before the first frame would trade frame 1's ~100 ms
  for a longer frame zero. Nothing has measured that trade.

## 3.12 The water and the shore (item 4164)

*2026-11-12, Opus 5.5. `client::ground`: `GroundTextures::index_land`,
`water_fade`, `holds_water`, `water_alpha` and `water_ramp`;
`client::world::World::drawn_terrain`'s water layer;
`rondata::artdata::GroundArt`'s `coastal` and `ocean`. Scored on
run1564's photograph (§3.3, rung 4). Item 4178 (below, "The water
drawn"): `client::ground`'s `WaterGrid`, `water_grid`, `water_mesh`,
`water_colour_ramp`, `floor_texture` and `floor_shade`;
`World::set_screen`; `gpu::Frame::patch_version`.*

**The land pass, read.** `TileSet::convert_land_tex_to_linear@0087ace0`
buckets every cell, not only `BASELAND`'s. For land 1 (`eTILE_COASTAL`,
our `SANDY`) and 2 (`eTILE_OCEAN`) it returns the count of every texture
of the types before, and nothing more. Neither `land_sub` nor the cell's
coordinates enter (`iVar5` is the sub only when the land is 0, and the
`0x55555555` term is behind `param_1 == eTILE_FERTILE`). So a `SANDY`
cell draws `<COASTAL>`'s first texture and an `OCEAN` cell `<OCEAN>`'s,
laid one repeat a cell by `generate_land` as every cell is. The type
names are the PDB's `TileSetLandTypes`. On `early_autumn` the two are
`sand.tga` and `ocean_floor_5.tga` under `BASEDIR`.

**The water, read.** `TerrainOut::render_water@00852cd0` draws one mesh
that `create_water_verts@0086ceb0` lays over every cell
`coord_contains_water@0086a080` passes. That is `COAST` or `HALFLAND` in the
cell's `WData` flags, land `OCEAN`, or a `CoordInfo` arm, one of which
asks whether any of the cell's 5 × 5 corners is below zero. A vertex
falls every half cell, two tiles. Its colour is `watercolor_solid`, white,
and its alpha is `get_water_fade@00869fd0` of the corner height there:

- 245 (`graphics.txt` `WaterColor Solid`) at or below −55
  (`__real_c25c0000`);
- none above `TerrainOut +0x4d80`;
- between them, `(h − top) / −55 · 245`, truncated.

`+0x4d80` is a registered variable, with range −91 to 0
(`TerrainOut::registration@00865930`). The decompile holds no value for it.
**run1564's packet holds −25.0** at `0x00c0fe70`: `terrain`, `0x00c0b0f0`,
is the object `MiscAccess::terrain` points at. So the fade does not meet
245 at −55, where it reads 133. The same packet holds the reading's other
values: `+0x5614` −40 (`OceanFadeHeight`), `+0x5618` 3 (`OceanFloorScale`)
and `+0x564c` `0xf5ffffff` (white at 245).

**The surface, read off the card (item 4168).** The technique is
"Normal Water" (`TerrainOut::one_time_init`). run1567's card trace
(run1564's take, the probe of §3.14) holds the water as the pass's one
draw with `SCREEN_REFLECTIONS`, after 3279
(`the_water_and_the_floor_at_the_card`):

- stage 0 `SELECTARG2(TEXTURE, DIFFUSE)` for colour and alpha: the
  vertex's white and its fade; the bound `newwater` frame does not enter;
- stage 1 `SELECTARG2(DIFFUSE, CURRENT)`, and its alpha
  `SELECTARG2(TEXTURE | COMPLEMENT, CURRENT)`: the current. **The fog
  fade the reading put there does not enter either**;
- options `LIGHTING SPEC_MODEL_TORRANCE SCREEN_REFLECTIONS`, with **no**
  `ANIMATE_WATER_NORMALS` and no normal map;
- blended source alpha over, on the HDR target, before the tone map;
- `PerMaterial`: ambient 0.2, diffuse (1, 1, 1.5), specular 40, emissive
  0, `gPower` (the roughness) 0.3; the light white, its direction
  (−0.707, 0, −0.707); the scene's ambient the units'.

`create_water_verts` writes positions, two texcoords and the colour, and
`render_water` adds nothing: ~~**the water's vertices carry no normal**, so
its N·L, specular and reflection ray are whatever the vertex shader reads
for a missing one (parked 4168a)~~. The mesh drawn is `modify_water`'s,
which carries the normal (0, 0, 1) at every vertex (item 4174, below).

**The floor is lit too.** The `ocean_floor_5.tga` draw is opaque,
`LIGHTING`, stage 0 `MODULATE2X(TEXTURE, DIFFUSE)` and stage 1
`MODULATE(TEXTURE, CURRENT)` on the same file by the second texcoord.
Its material is white. The ground's light is diffuse 1.36 with ambient
(0.02, 0.06, 0.36), and the lit branch reads only the diffuse. The sand
draws with normal, spec and gloss maps, blended at its vertex alpha.
**The photograph backs neither the lit water nor the lit floor (below).
It backs the ocean's base as the water's colour.**

`RUN1564_3279_WATER` gives the `SANDY` and `OCEAN` tiles agreeing under
`RUN1564_3279_GROUND`'s rule:

| the water's surface | `SANDY`, of 772 | `OCEAN`, of 437 |
|---|---|---|
| none: the land pass alone | 55 | 233 |
| **the ocean's base, the draw** | **294** | **233** |
| `water0000.tga` | 112 | 0 |

The lit path is not drawn (parked 4164a).

**What the row says.** `RUN1564_3279_GROUND` goes **709 → 1,236 of
1,920**:

- `SANDY` 0 → 294;
- `OCEAN` 0 → 233;
- the land 709 of 711, unchanged;
- on the base alone, 584 → 872.

The residue is one tint. The photograph's water is bluer and darker than
the floor's texel. On `OCEAN` tile (161, 191), at pixel (296, 1), the
photograph reads share (0.139, 0.362) and we read (0.118, 0.405), which
agrees by 0.048. The 563 `SANDY` tiles the photograph shows blue average
(62, 114, 147) against our (78, 203, 231). The 209 it shows dry average
(161, 157, 108) against our (173, 200, 166), so the water lies on beach
the photograph shows dry (parked 4164b).

**The client draws it.** The land pass goes into `ground_base`'s meshes.
The water is one `PatchMesh` over every tile of a wet cell, between the
patch and the mountains. Its alpha rides a 256-texel linear ramp, so
`fs_patch` takes it unchanged. `the_client_draws_every_tile_once` holds it
once over each wet tile. The drawn-terrain cache had used an empty debug
mesh as its "not built" mark. With every tile of the lobby map textured,
it rebuilt every frame, and the clock's peak went 242 → 765 MiB. The mark
is now all three lists empty, and the clock reads 247 MiB.

**Diff-backed:** the fade's top and the water constants (the packet). The
rest rests on a reading and one photograph: the bucket, the predicate and
the fade's shape. `water_fade` is pinned by
`the_water_fades_as_get_water_fade_fades`.

**The meshes the original drew, off the packet (item 4174).** run1564's
packet holds both meshes the card draws: the floor in `scene_ocean_floor`
(`TerrainData +0x35a0`; the decompile's `TerrainOut` offsets are the type
record's plus 0x40), and the water in `TerrainOut::modify_water@0086e700`'s
function statics (`verts` `0x00ee179c`, `norms` `0x00ee17bc`, `colors`
`0x00ee183c`, `sbl/rise_z.map`). `tools/render/water_packet.py` dumps them.
The counts are the card's: 624 floor vertices on draw 36 and 1,755 water
vertices on draw 162, and `args[1]` is the vertex count.

- **The land pass draws no `OCEAN` tile.** Draws 0–34 bind grass and dirt
  only. `ocean_floor_5.tga` is drawn once, by `render_ocean_floor` (the
  call at `0x857e6b` is the return both draw 35 and 36 carry).
- **The floor** is `build_ocean_floor@0086c7f0`'s plane: one vertex a cell
  corner, every one at **z −283**. Its texcoords are exactly `xy / 6144`
  plus one offset on the first layer and `−xy / 6144` plus another on the
  second: **one repeat every eight cells, the second layer mirrored**. The
  offsets drift with `modify_ocean_floor_texture`'s clock.
- **The floor's grey** is `modify_ocean_floor@00873960`'s: `OceanVars`'
  `floor*_top` (255) to `floor*_bot` (64) along the screen's ramp, times
  0.6, alpha `0x99`. The ramp is `determine_frustum_intersect@00854770`'s:
  the ground points under the screen's top-left and bottom-left pixels,
  and a point's fraction along `(1, 1)/√2` between them over their
  distance. The lateral axis does not enter.
- **The water's colour** is `modify_water`'s: the white `create_water_verts`
  lays is replaced each frame by `OceanVars`' `top*_top` (0, 159, 255) to
  `top*_bot` (0, 111, 218) on the same ramp. Its alpha peaks at 189, not
  245 (`generate_water`'s smoothing, ~~parked 4174b~~ settled by item
  4178, "The water drawn", below).
- **The water's normal is (0, 0, 1)** on all 1,755 vertices, its tangent
  (1, 0, 0). 4168a is settled: the lit branch sees N·L 0.707.

`client::ground::ScreenRamp` unprojects our own camera. At run1564's it
gives d0 48,490.9 and length 9,729.0, against a fit to the packet's water
of 48,408 and 9,683. Through `floor_grey` and `water_rgb` it reproduces
**every one of the 624 floor greys and 1,755 water colours within a byte**
(`the_floor_and_the_water_off_the_packet`; with `FLOOR_BOT` set to 70, 85
greys part and the test fails).

**The card's model, counted, and its kill** (`RUN1564_3279_CARD`). The
floor goes on `OCEAN` tiles as the card's stages leave it: the first
layer doubled under the grey, clamped, times the second. Every wet tile
takes the water at our alpha. Counted unlit, and through the lit branch
of the install's pixel shader, read: the stage colour to the 1.45, times
the HDR multiplier on the material diffuse × the light × N·L / π plus the
material ambient × the scene's ambient. That is blended in HDR and toned
at the clamp, with no specular and no screen reflection:

| model | `RUN1564_3279_GROUND` | `OCEAN` | `_LUMA` | `OCEAN`, luma |
|---|---|---|---|---|
| as drawn | **1,236** | 233 | 791 | 2 |
| unlit | 793 | 0 | **1,426** | 261 |
| lit | 814 | 0 | 1,072 | 75 |

Both card models put the lake's brightness in register and lose its hue.
On lake tile (182, 191) the photograph reads (38, 97, 146), the lit model
(1, 91, 180) and the drawn (53, 223, 220). The photograph's lake holds
red, 41 to 46 in four boxes, correlated with blue (0.5 to 0.9). No input
on the packet carries any: the water's red byte is 0, and the floor's red
survives the product of two cyan texels only as 1 to 20. **The scored row
falls under either model, so neither is drawn** (parked 4174a).

**The water drawn (item 4178).** The client now draws the water's own
mesh, the floor under it, and the water's colour. Every input is the
packet's, diffed vertex by vertex against run1564's drawn water
(`the_water_s_lattice_off_the_packet`, `RUN1564_WATER_LATTICE`).

- **The lattice.** `create_water_verts` lays each wet cell's 3 × 3
  points, 384 units apart, and `modify_water` draws each 384-unit quad as
  four triangles about its centre. The centre takes its four corners' mean
  alpha, truncated: 540 of 540 drawn centres. **All 1,755 drawn vertices
  stand at z 0**, 80 distinct corners of them over ground above 0. So the
  water is flat at 0 and not at `max(0, ground)`: the odd branch's
  `find_data_z` does not reach the drawn z. `client::ground::water_mesh`
  lays it.
- **Which cells are wet.** `coord_contains_water`'s corner arm is taken on
  every land, not only `SANDY`. With the `SANDY`-only arm, 64 drawn
  corners fall on `BASELAND` cells (flag `0x400`) that we leave dry. On
  every land none do (`holds_water`).
- **The alpha** (4174b settled). `create_water_verts`' odd branch reads
  `get_water_fade` at `4 · (row >> 1)` tiles. So an odd row's vertex takes
  the fade of the corner 384 units north of it. `generate_water` then
  smooths in place, row by row, at the solid 245: each laid point takes
  the mean of itself and its laid 3 × 3 neighbours, the smaller of that
  and its own, squared and truncated. **597 of 597 drawn corners agree
  exactly** (`WATER_READ`). The deep interior's fixed point under in-place
  smoothing is the 189 the packet peaks at. Each other reading agrees on
  fewer: odd rows at their own height 404; from a copy 142; at solid 220
  from a copy 346, in place 141. The `0x800` quarter does not enter
  run1564's view.
- **The colour.** `modify_water`'s ramp of `OceanVars`' top bytes rides a
  256-texel texture, `water_colour_ramp`, at each vertex's ramp fraction.
  `World::set_screen` gives the camera's `ScreenRamp`, and only the
  water's and the shade's texcoords are laid again when it moves. The
  gpu uploads the patches again on `Frame::patch_version`, the terrain
  standing.
- **The floor.** The land pass draws no `OCEAN` tile. They take the
  floor's two layers: `<OCEAN>`'s texture at `xy / 6144`, and again
  mirrored about the packet's summed offsets (`FLOOR_MIRROR`), baked into
  one texture by `floor_stage` at the brightest grey, 153. A black layer
  over it at alpha `1 − grey / 153` takes each vertex to
  `modify_ocean_floor`'s ramp grey. That is the stage's product wherever
  `MODULATE2X` does not clamp. The floor is drawn on the `OCEAN` tiles'
  own triangles, not the plane at −283.
- **No shader changes.** The water and the shade ride `fs_patch` as an
  alpha patch does.

The rows, on the client's still, from the base:

| row | before | after | `SANDY` | `OCEAN` |
|---|---|---|---|---|
| `RUN1564_3279_GROUND_STILL` | 1,237 | **1,373** | 295 → 664 | 233 → 0 |
| `GROUND_LUMA_STILL`, run1564 | 791 | **1,001** | 83 → 80 | 2 → 215 |

The land is unchanged on both. run1559b's still holds 1,097 and 1,076.

- **`OCEAN` loses its hue**: 4174a's red, met on the still. **`SANDY`
  gains it**: the water's own blue over the sand is the photograph's
  shallows. The value diffs are the item's journal.
- **The ramp is drawn because it is the packet's**, vertex by vertex; held
  at its middle the rows read 1,352 and 1,179 (the journal).
- **The painter keeps item 4164's water** on the seabed, under the wider
  corner arm. `RUN1564_3279_GROUND` goes 1,236 → 1,233: three plain tiles
  of `BASELAND` shore cells. `RUN1564_3279_CARD` goes to (1,233, 784,
  813), its luminance unchanged (parked 4178a).

**Diff-backed** (run1564's packet): the lattice, that every drawn corner
lies on a cell we lay wet (that every cell we lay wet in view is drawn is
not checked: the view's cull is not read), the alpha, the centres and z
0; 4174's floor greys and water colours. Resting on a
reading: the floor's two layers as one texture, and the shade. Resting on
one photograph: the rows.

**What is not established.**

- **The water's red** (4174a). The photograph's lake reads r ≈ 40, and
  every packet input to the floor and the water gives 0 to 20. The
  specular, by the reading's terms at roughness 0.3 and the light
  (0.707, 0, 0.707), is about 0.05 in HDR, a few bytes. The coral layer
  (draws 130–131, `coralblend.tga`), the surf (163–169) and the frame's
  mean luminance are not drawn. Item 4178 reads the screen reflection in
  the install's `ps_global.hlsl`. It marches ten 10-unit steps along the
  reflected ray, against the depth of what was drawn before the water, and
  on a hit within 0.00075 it adds `N·V` times the scene below the water
  (`BackTex`). On the pinned camera (orthographic, near 6,589, far 36,262)
  that threshold is about 22 view units. So it can hit in the shallows,
  adding the sand's own colour, and not over the floor 283 units down.
  That is a reading, not counted.
- ~~**Where the water lies.**~~ Settled by item 4178 ("The water drawn"):
  flat at z 0 on its own lattice, the corner arm on every land; no longer
  on the seabed's triangles.
- ~~**The water's alpha** (4174b)~~: settled by item 4178, 597 of 597. The
  quarter on a tile's `0x800` flag is not read; it does not enter run1564's
  view.
- **The shore's blend** (parked 4178b). On frame 3279 the card draws
  `sand.tga` as its own layer, ~~4,928 vertices blended at their alpha
  (draw 35, flushed at `render_ocean_floor`'s entry)~~ at the land's depth
  fade (§3.16), and four
  transition-patch draws on `sand.tga` (`110000000011` to
  `110011111111`). Ours draws `SANDY` cells whole in the land pass, so the
  beach meets the grass on a sawtooth.
- **The floor's place and light.** It is drawn on the `OCEAN` tiles' own
  triangles, not the plane at −283, so its edge against the underwater sand
  is a sawtooth too. It is unlit; the card lights it (its light diffuse
  1.36).
- **The floor's texcoord offsets** drift with
  `modify_ocean_floor_texture`'s clock; the client lays run1564's. The surf
  (`<COASTAL>`'s `COASTALPHA*`, `SANDDRY`, `SANDWET`) and the coral are not
  drawn.

## 3.13 The draw list across the ages (item 4166)

*2026-11-12, Opus 5.5. Built in `crates/rondata/src/free/ages.rs`, over
the games `rondata::diff::drawn_games` lists. Checked by
`free::ages::the_draw_list_across_the_ages_resolves`. Amended by items
4169 and 4175, the same day.*

**The score.** Over the 82 late-age pairs and the 28 battery words, on the
4,608 frames their long captures dump: **resolved 1,171,784 of 1,171,799
objects over 110 lobbies** (`AGES_RESOLVED`; 1,166,600 at item 4166,
1,167,081 at 4169). The closing blocks' pieces are ours on **34,394 of
34,399** records (`AGES_CLOSING`), and their corpses' on **196 of 196**
(`AGES_CORPSES`, item 4175). K is M on **98 of 110** lobbies
(`AGES_LOBBIES_WHOLE`; 83 at item 4166; item 4175 left it there). The run takes about 100 s on four walkers.

**What is walked.** Each game is the free runner stood up from its start
sibling, its long capture's setup block and its trace's pasture
(`FreeRunner::from_captures`), stepped to the closing block. The frames are
frame 0 and every `FRAME` the long prints: every hundredth frame on a pair,
and only the setup and closing blocks on a battery word. The lists are the
handoff's `Late ages:` and `Battery word:` rows of `AI_WORDS`, held to them by
`the_census_walks_every_late_age_and_battery_game`. The same test asserts
that no game is in the held-out battery (`diff::battery::BATTERY`, DECISIONS
61 §6). The battery words are the lobbies promoted out of it.

**What an object is.** Each entry of the client's draw list on the frame
(`render::draw::draw_list` of `free::placed` and `free::placed_buildings`, as
the observer sees it), plus each projectile and each corpse in the scene. An
entry resolves when its piece names install files that decode under
`render`'s readers:

- a figure: its `<UNIT>` model and the animation of the slot it plays
  (`PieceDecoding::decodes_on`, §5.4's condition 1 on that slot);
- a building: §5.4's building condition 1 for the piece it shows
  (`artdata::building_decoding`), on the tileset its map draws;
- a field: the farm's `<FARM>` and wheat image on that tileset;
- a foundation: its `<CONSTRUCTION>` sprite (and the dock's model);
- a scaffold piece: its `<SCAFFOLD>` model, texture and animation;
- a round: its `<AMMO>`'s model and texture, or a falling aircraft's
  `<UNIT>` model (`ProjectileScene::gpiece`, `AGES_ROUNDS` by piece);
- a corpse: each standing record's piece on the slot it plays
  (`DeathScene::pieces` on `cur_anim`), which joins §5.4's played slots.

**The unresolved, cheapest first** (`AGES_UNRESOLVED`, objects summed):

| type | objects | what fails | what would settle it |
|---|---|---|---|
| ~~Sawmill (424), piece 55219~~ | ~~22~~ | ~~an `anim` `anim_graphics.xml` does not name~~ | resolved by item 4169, below |
| ~~Smelter (425), pieces 50963, 51092~~ | ~~180~~ | ~~the same~~ | resolved by item 4169 |
| ~~Observation Post line, 522–525 (pieces 50803–50805)~~ | ~~279~~ | ~~the `<BUILD>` names no file~~ | resolved by item 4169 |
| Citizen and CEO crews (38322, 38323, 39378, 39379) | 15 | slot 2's `ceo_crew3_idle3.bha` is not in the install | what the figure's draw makes of a null action |
| ~~projectiles~~ | ~~2,133~~ | ~~the scene carries no piece~~ | resolved by item 4175, below |
| ~~corpses~~ | ~~2,570~~ | ~~the scene carries no piece~~ | resolved by item 4175 |

The 522 rows are Observation Posts already upgraded: the building keeps
`orig_type` 522 and shows the Air Defense Gun's piece. No type is bent to a
missing file. Every file a `<UNIT>` or `<BUILD>` names on these frames is in
the install, except the crews' one `.bha`.

**What the loaders do with what is not there** (item 4169):

- **A name no `<ANIM>` row carries names no file.**
  `GraphicPieces::decipher_animation@008face0` answers `-1`.
  `init_build_data@008f2f20:295` reads a `<BUILD>`'s own `anim` only for
  `build_anim_time`: on `-1` it skips the load and leaves the time 0. The
  sawmill's `sawmill_3t Default` is that case (`LUMBERMILL-ARAB-AGE3` and
  `-EINDIAN-AGE3`; no `sawmill_3t_addon.bha` ships). `init_build_events
  @008ea3c0:252` stores the `-1` in an `EXISTEVENT`'s anim word and goes on;
  that is the smelter's `BUILD_BUILD` event, `smelter_0a_addon` without
  `Default`. Neither reports. So the piece loads, and `artdata::
  building_decoding` no longer counts the name as a failing file.
- **The Air Defense line loads as a `<UNIT>`.** `GraphicPieces::verify_load
  @00906550:162`–`173` sends a piece of type `0x20b`–`0x20d` among the first
  `0x81` build pieces to `init_unit_data`. The `<UNIT>` of the piece's own
  name, `GRAPH-DEFAULT-AGE0`, is its art, as `artdata::wall_packets` reads
  its slots (COMBAT §84.2). `artdata::loads_as_unit` names the pieces, and
  `building_decoding` holds them to the `<UNIT>`'s model and every slot's
  file. All decode.
- **A named file the install lacks is a null action.**
  `AnimMgr::force_load@0053ade0` logs "animation was not found" through
  `OutputDebugString` only. It stores no `AnimObj`, 200 ms and so 3 frames
  (`sim::units::anim::MISSING`). `AnimationPacket::get_animobj@00917170`
  hands the draw that null. The crews stay unresolved: what the hierarchy's
  pose call makes of null is unread.

**The round's piece and the corpse's** (item 4175). The door widened in
`sim::observe` alone; the simulation now carries both.

- **A round's piece is the package's** (`Ammo::init@0067bbf0:264` copies
  it to `AmmoData +0x8`). `GraphicEvents::execute_game_events@008e48e0`
  puts the release event's own piece there; `init_unit_events@008e2520:284`
  made it `first_ammo_piece` plus the first `ammo_names` entry
  `String::operator==` matches (without case), and `ammo_names` opens with
  the `<AMMO>` names in file order (`init_ammo_piece_ranges@008f6140`).
  `Object::fire_ammo@0064c8b0` puts a building's `first_ammo_piece +
  get_shot` (0 to 5) and a unit's `fire_proj` (`UnitType +0x2cc`):
  `first_ammo_piece`, or an `<INSTANTHIT ammo>`'s (`:447`; the four
  missiles). `Ammo::init_crash@0067b800:44` puts the falling plane's own
  piece. All 35 release types the install names are `<AMMO>` names.
- **`first_ammo_piece` is `0xeaf5`, from the dumps**
  (`sim::combat::FIRST_AMMO_PIECE`): run109's arrows print 60157
  (`ArcherArrowWOtip`, 8), run1403b's Infantry 60173 and 60174 (24, 25),
  its Chi-Ha 60185 (`Tankshell`, 36), run1405b's nuke 60249 (`Nuke`, 100).
- **A corpse's piece is each guy's own.** `DeathObj::init@008d4c60` takes
  vslot `+0x178`, `UnitData::get_gpiece@0060dd80`: `guys[k] +0x88`. Its
  `cur_anim < 0xf` swap to `RData::unit_data +0x24` is never taken, since
  `dtype >= 1`. `DeathScene::pieces` lists the records whose life stands.

**The value diff, the rounds and the corpses.** run109's 183 `AMMO`
blocks now compare `gpiece`, all 60157 both sides
(`run109_says_every_ammo_field_this_crate_models_is_the_original_s`), and
every `AMMO=5` walk compares it (`golden::round_rows`). The 110 closing
blocks' `DEATH_OBJS`: 196 records, each `gpiece` one of its corpse's
standing pieces (`AGES_CORPSES`); the harness compares `DEATH_OBJS`
`gpiece` the same way on every capture. No building's round is on a walked
frame, and no packet was read: the dumps print both pieces.

**The value diff for these: the original printed the pieces.**
`get_build_gpiece@0090bce0` takes the first piece whose `data_pieces` entry
is set. A piece that failed to load would have been walked past, and the
dump would print another. On the 110 closing blocks (`AGES_CLOSING_WATCHED`,
ours of printed): sawmill 55219 2 of 2; smelter 50963 5 of 5 and 51092 5 of
5; the Air Defense line 50803 10 of 10, 50804 11 of 11 and 50805 19 of 19;
the crews 38322 10 of 10, 38323 9 of 9, 39378 1 of 1 and 39379 3 of 3.

**The value diff: the closing block.** Every long capture prints its units'
guys and its buildings with `gpiece` on its closing block, frame
`length + 1`. On each lobby, every printed record whose object the runner
holds is compared with the runner's piece on the same frame. Of 34,399
records, 34,394 agree. The five that part are all
`GreatLakesAmericanInformation` on frame 4321, guy 0 of the Americans'
Assault Marines (TypeIndex 122, land, foot):

| record | dump `gpiece` | ours | our slot |
|---|---|---|---|
| `1/55` | 72, `ASSAULTMARINES-DEFAULT-AGE0` | 6408, `…-AGE0-FEMALE` | 23 |
| `1/67` | 72 | 6408 | 8 |
| `1/69` | 72 | 6408 | 9 |
| `1/71` | 72 | 6408 | 7 |
| `1/73` | 72 | 6408 | 9 |

The dump prints `cur_anim` 9 on each. The variant a figure takes, male or
female, is chosen at the guy's making. Ours took the female row of the
same entry, and both rows decode. No pixel is compared (parked 4166a).

**K of M on each lobby.** This is §5.4's counter read on the lobby's own
dumped frames. M is the figure pieces played (with the slots played) plus
the building types drawn (with the pieces drawn). A member counts when its
files decode and the closing block's piece is ours on every record of it.
On 98 of 110 lobbies K is M (83 at item 4166). The other 12 fall short
only through the crews' row above, or through the Assault Marines' piece. **Condition 2 is read on
pieces, not on pixels.** No lobby but `GreatLakesAztecModern` has its anchors
put through the original's projection (§3.5).

**Inexact ground.** `Built`'s guard counts height reads on corners this crate
cannot pin to the original's single (`docs/COMBAT.md` §46.3). The runner
hands the census its count (`FreeRunner::take_ground_inexact`), and each game
is held to the largest `testkit::GROUND_INEXACT` row of its own module. No
game exceeds its pin. Maya Ancient's 52 is its closing state's pin.

**Made to fail once.** A field resolved on a tileset with no wheat image
turns every field red (below, in the journal). The held-out guard is red
with a battery lobby added to the list.

**Diff-backed:** the closing block's pieces, on 110 lobbies. Item 4169's
three loader readings are backed by the same block: the original printed
every piece they resolve. Item 4175's corpse piece, on the same blocks, and
its round piece on run109 and the `AMMO=5` walks. **Reading only:** every resolution beyond "the
file decodes", and what an unnamed anim's `-1` draws. A piece whose file
decodes but is the wrong file is caught only where a dump prints the piece:
the closing block, and the Aztec pair's starts.

**What is not established.**

- **Any pixel off the Aztec pair.** Anchors are not projected on these
  lobbies.
- **The viewer's drawn set.** The list is the observer's, every object on
  the map.
- **The pieces between the start and the closing block.** The pairs' strides
  print guys without `gpiece`.
- **4166a**, the Assault Marines' variant. Item 4169's reading is in its
  journal: the original's piece is guy state written at seventeen call
  sites, and ours is derived from the object number every frame.
- **What the addon draws under `BUILD_BUILD`** with the smelter's `-1`
  anim, and what the crews' figure draws from a null action.
- **What a round or a corpse draws**: a round resolves on its files, not
  on a pixel; what slot `AmmoOut::draw@0067ac10` plays on a falling
  plane's unit piece is unread. The client draws neither (parked 4175a,
  4183a): the door lacks the corpse's place and the round's height, below.

**The photograph of a corpse and a round** (item 4183, run1570; the rule
before any count, so no count yet). run1570 is run1559b's paused, traced
take with the pause on 3331, 19 frames past the record `who 0 o 7
first_frame 3312 cur_anim 17 gpiece 10912`. Its packet's camera is not
the client's pinned one: `loc` is (3168, 30816), the capital's centre,
against run1559b's (2104.28, 30816). Through the packet's own camera the
dying unit's last place (4006, 28612) projects to (1438, 337), among the
oil derrick's logs at the tree line. A second corpse, `0/11` from 2703,
stands on the same frame. The rounds aimed at `0/7` and at the city all
land on screen: pieces 60164 and 60174 on 3309–3318, and the
`Tankshell` 60185 on 3309–3310 only. The dump prints no position for a
`DEATH_OBJS` record, so the packet's pool is the only oracle for the
corpse's place.

**The rule, `RUN1570_DEAD`.** It is drawn on the photographed frame F,
which is settled first by the movers, as `client::silhouette::PHOTO_FRAME`
settled run1559b's 3279 (4073a). Every corpse record standing on F, and
every round in flight on F, is one window. It is placed by the original's
geometry at the original's points:
- a corpse at its `DeathObj`'s x, y, `center_z` and angle, read off the
  packet's pool, posed on `cur_anim` at `DeathObjOut::draw@008d4090`'s
  time (`age × 200 / 3` ms, held at the action's end);
- a round at `AmmoOut::draw`'s point, read off the dump's `AMMO` on F:
  x and y from `sx`, `sy` to `ex`, `ey` at `cur_time / total_time`, and z
  by `traj` (linear; `sz + v1z·t + GRAV_Z·t²/2`; the path table).

Each window is projected through the packet's camera. Ours is the client's
still (§3.15). The window's colour is read by `_TRACED`'s reader, and its
ring is the still's pixels where no key lies. A window counts in register
when the overlap is 0.5 or more and the chromaticity is within 0.05: the
rows' own tolerances. An occluded window (`0/7`'s, if the trees cover it)
stays in the selection and is named, never dropped. The count as drawn
(none) is pinned first.
- **A building's round piece and the Merchant arm** of `fire_ammo` are
  read, not diffed; no walked frame or `AMMO=5` walk holds one.
- **The doobers' 16** before `first_ammo_piece`: the constant is the
  dumps', not the loader's sum.

## 3.14 The device's state at a figure's draw (item 4165)

**The trace.** run1566b's card trace (`tools/trace/card.h`, read by `rondata::trace::card`) copies, at each
`D3D11GLSys::render` (DLL `0x10023ac0`, `0x10023d30`), the `RenderState`, the four constant buffers, `mShaderOptions`
and both stages' texture names. The pass after 3279 is the photograph's: under the take's fast-forward the original
draws one frame in sixteen, after 3263 and 3279 (4073a).

**The match** (the rule before its count). A figure of §5.3's 33 matches the lit draw at its own point, `gWorld`'s
translation, within `CARD_MATCH`; a building its `_NONQUAD` sprite draw. **33 of 33, every figure at 0.0 units**
(`RUN1559B_3279_DEVICE_STATE`); on the 3263 pass, 22 of 33.

**Diff-backed there, on every lit figure draw:**
- stage 0 `BLENDTEXTUREALPHA(TEXTURE, TFACTOR)`, the tfactor the owner's `TEAM_UNIT` (4102);
- stage 1 `MODULATE(CURRENT, DIFFUSE)`, the diffuse 1 with no vertex colour;
- options `ALPHA_TEST LIGHTING`: no vertex colour, normal map, fake light or `NO_HDR`;
- the material, the white light and the scene's ambient `lit_pipeline` reads (4114).

On every silhouette draw: `SELECTARG2(TFACTOR)` at 128, source alpha over the rest (4103). The tanks' streamed
normals are ours within 2.4–3.5° (median).

**So** the colour with no fitted term is 4114's candidate, 17 of 33. The figure that parts, 1/27, parts past the
draw (4165a). Stage 1's alpha, `MODULATE4X(TEXTURE, TFACTOR)` with blending on, makes a texel under alpha 64
translucent over what is behind it: 319 of the figures' 23,316 own pixels, drawn by no draw of ours (4165b).

**Past the draw, read off the packets (item 4168).** The packet keeps the heap, so the executable's `graphsys`
(`0x00e335a0`) reaches the `D3D11GLSys` and its `mToneMapper` (+4976) and `mAntiAlias` (+4980)
(`tools/render/tonemap_packet.py`; `d3dgl.pdb`'s `GSToneMapper`, `D3D11ToneMapper`, `ToneMapData`, `D3D11AntiAlias`).
Each object is typed by its vtable at its own `d3dgl.map` address, both under one relocation, `0x68550000`. On
run1559b's and run1564's packets, the members and the constant buffer's shadow (`mData`) agree bit for bit
(`the_tone_mapper_on_both_packets`):

| field | live | `lit_pipeline` |
|---|---|---|
| exposure, adjustment | 1.0, 0 | `EXPOSURE` |
| white point | 3.5 | `WHITE_POINT` |
| minimum luminance | 1.4 | `MIN_LUMINANCE` |
| saturation | 1.0 | `SATURATION`, the identity |
| tint | (238, 193, 14, **0**) | `TINT_ALPHA`, the identity |
| bloom threshold, tightness, strength | 0.6, 12.5, 6.0 | `bloom_bright` |
| FXAA | on | not drawn |

**So the tone map is the candidate's, every input live**: 4114's "no caller sets them" is now a diff, except the
tint's rgb, which is set but unused at alpha 0. The colour row is 18, unmoved, since a chromaticity reader sees the
white scale only through clipping. On tank 1/27, the photograph's own bright pass, blurred by the bloom's
taps at either scale, bounds the bloom from above. Over its 60 × 40 window it averages 0.84/255, at most 10.2/255.
**So bloom does not part 1/27.** It may part 0/0, whose bound averages 38.8/255. What is left past the draw for
1/27 is the frame mean through clipping, and FXAA (4168b).

**On brightness (item 4172; `Luma`, the rule committed before its count).** On the colour reader's pixels, dark
ones kept (an object's: its key, the traced footprint; a tile's: the ground row's), each side's mean of 0.3 r +
0.59 g + 0.11 b (`LUMA`), floored at 8. A unit agrees when the larger is at most 1.2 times the smaller; an object
is scored on 40 held pixels, a tile as the ground row scores it. The figures are counted twice: as drawn, and under
4114's `lit_pipeline` at the clamp after the team blend, by the face's N·L against −`cur_dir`, on every mesh pixel
not behind. The one more agree with is drawn, if its colour row is not under 18; a tie keeps the drawn. Pins:
`RUN1559B_3279_LUMA`, `_GROUND_LUMA`, `RUN1564_3279_GROUND_LUMA`. **Counted** on the painter: the figures **20 of
33** as drawn, **23** lit, whose colour row is 17, so the drawn stays; the ground **1,075 of 1,109** (the
mountain's body, eight tiles at 0.83–0.92, in) and run1564's **791 of 1,920** (`OCEAN` 2 of 437, `SANDY` 83 of
772); run1441's camera 1 of 32. A light 1.5 times as strong takes them to 4 and 1.

## 3.15 The client's still under the photograph's readers (item 4171)

**The rule** (4171, before its count, `0c4d44851`; DECISIONS 127 §1). The rows
counted on the client's own pixels: `Gpu::still` at the photograph's
size, the frame the client draws (`build`, `drawn_terrain`) from the
world and camera each row's painter takes, without the debug overlay.
Each row keeps its rule, its constants and its selection; only *ours*
is read off the still. `_TRACED`: our side of a window is the
photograph's colour reader run on the still, its ring the still's
pixels where no key lies, outside the interface; a pixel the
photograph's side leaves out is out of both. `_COLOUR`: [`HUE`] on the
same pixels, ours the still's pixel for the painter's texel (§5.3). Both
`_GROUND`s: ours the mean of the still over every ground pixel of the
tile. **The tolerance is the rows' own** (0.5 overlap, 0.05 in
chromaticity): the still is *equal* to the painter on a row when the
count and the members agree; otherwise each crossing is named with
both sides' values. Per pixel, still against painter, a channel
within 8 of 255 agrees: a diagnostic, never a row. Expected: `_COLOUR`
red first, the figures washed.

**The count.** On the draw as it stood: `_TRACED` 32,
`_COLOUR` **2**, `_GROUND` 1,003 (94 mountain tiles down, the body
(22, 18, 6)), run1564 1,286. Two causes, each a diff: the silhouette
after the lit pass, where a figure's far side passes Greater on its own
near side; and an sRGB target, so every blend, `ADDSIGNED` and vertex
colour is taken in linear light where the original's stages and the
painter take bytes. **The client now draws the silhouette first, once a
pixel (a stencil), on plain `Unorm` targets**: `_TRACED_STILL` 31 of
33, `_COLOUR_STILL` 14 of 33, `_GROUND_STILL` 1,097 of 1,109 tile for
tile, run1564 1,237 of 1,920 (one tile at the bar). The partings, named
in the pins: 0/1 behind the pine on our flat green (~~4173's~~: 30 → 64 with the trees textured, the painter's 66; §3.7), three by
the card's mip level per pixel against the painter's per triangle
(4171b), and two footprints the still's colour reader cannot separate:
0/1, its ring holding a silhouette the painter keys nothing at (4171a; ~~gone with the trees textured~~, `_TRACED_STILL` 32, §3.7),
and 1/40, a tank the photograph's side reads by hand. Not established:
the original's cull mode 2 on both states, which this draw does not
take (4171c).

## 3.16 The shore's sand (item 4192)

*2026-11-12, Opus 5.5. `client::ground`: `land_alpha`, `land_alpha_of`,
`LAND_FADE_TOP`, `COAST_DEPTH` and `LAND_HEIGHT`;
`client::world::World::drawn_terrain`'s faded land;
`tools/render/sand_packet.py`. Scored on run1564's photograph (§3.12, split
from it at the section ceiling).*

**The sand layer, off the packet.** The land pass keeps one `RenderHelper`
per linear land texture at terrain `+0x4b0`, 38 of them, which
`CoordInfo::add_geometry@0084bef0` fills four corners a tile. On run1564's
packet only index 36 is filled: `sand.tga`, 4,928 vertices and 7,392
indices, 77 cells of 16 quads. That is the card's draw 35, and
`scene_coral` (draw 131) holds the same 4,928 positions. Each vertex's
colour is `master_land`'s at its corner and its z the corner's height.

**The fade, read and diffed.** `master_land`'s alpha is
`TerrainOut::light_land_vert@00866e50`'s depth fade, on **58,081 of 58,081
corners**:

- 255 at or above −52 (`__real_c2500000`), and 0 from −271;
- between them `(1 − m / span) · 255`, truncated, with `m` the depth past
  −52 and `span = |coast_depth + land_height + 2 + 52|`, 219;
- `coast_depth` is terrain `+0x5630` (−303) and `land_height` the global
  `0x00cbe54c` (30.0), both off the packet. `build_ocean_floor`'s plane is
  `coast_depth + land_height − 10`, the floor's −283 (§3.12).

A scan over the span puts 219 alone at 58,081. At `land_height` 0 the
formula agrees on 47,874 corners and on 1,077 of the sand's 4,928
vertices (`ZERO_SAND`, the made-to-fail side). Value diffs, packet against
`land_alpha`: the sand's first vertex, (148, 192) at −190.71, 93 and 93; a
corner at −59.01, 246 and 246 (247 at `land_height` 0); at −74.98, 228 and
228 (231); at −273, 0 and 0. Pinned by `the_land_fade_off_the_packet`,
`RUN1564_LAND_FADE`; seen red with the top at −51.

**What it blends over.** The packet's render states (`CompareFunc` from
the PDB, 3 is `CMP_LESSEQUAL`; the words are the item's journal):
`base_terrain_rs` tests and writes depth and blends source alpha over;
`ocean_floor_rs` tests depth, writes none, and is opaque. So the land writes depth wherever it is drawn,
and the floor plane at −283 shows only on the 42 of its 104 cells in view
that no land covers, the `OCEAN`-only cells, where the client draws it.
Under the land there is the clear colour, `BLACK`: `Game::init` calls
`graphsys` slot 0x140, `set_clear_color` (vftable offset 320 in the PDB's
`LF_ONEMETHOD`). **The deep sand fades to black under the water.**

**The client draws it.** In `drawn_terrain`, a land-pass triangle with any
corner below −52 leaves the opaque pass for its texture's faded
`PatchMesh`, alpha `land_alpha` on the 256-texel ramp, drawn before the
floor's shade and every transition patch. No shader changes.
`the_client_draws_every_tile_once` counts each faded triangle once,
against its texture's base tiles.

The rows, on the client's still, from the base (`SANDY` of 772):

| row | before | after |
|---|---|---|
| `RUN1564_3279_GROUND_STILL` | 1,373 (`SANDY` 664) | **1,024** (315) |
| `GROUND_LUMA_STILL`, run1564 | 1,001 (`SANDY` 80) | **1,318** (397) |
| `GROUND_STILL_SRGB`, run1564 | 834 | 927 |

run1559b's three are unchanged, as are the painter's `RUN1564_3279_GROUND`
1,233 and `_LUMA` 791 (it draws no fade). **The hue row falls because
the luma rises**: the deep sand had agreed on hue only lying opaque under
the water. By class on the still (photograph, then ours): the deep
`SANDY`, alpha 0, 251 tiles, (49, 111, 155) Y 101 against Y 145 →
(1, 104, 175) Y 87; the mid `SANDY`, 394 tiles, (89, 124, 135) Y 117
against Y 163 → (81, 131, 151) Y 122. What is lost is 4174a's red, as on
`OCEAN`. The fall was taken on the commander's word (2026-11-12).

**Diff-backed** (run1564's packet): the fade on every corner, its
constants, the sand layer's vertices and alphas. **Read off the packet,
not compared per pixel**: the render states and the clear colour. **Resting
on one photograph**: the rows.

**What is not established.**

- **The floor's brightness** (4178c). Drawn without the floor, `OCEAN`
  reads Y 92 against the deep sand's 87, and the photograph's 102 and 101,
  so the original's floor shows about as dark as black under the water;
  the card lights it (`ps_global.hlsl`'s HDR multiplier, the 1.45 power),
  and ours is unlit. Luma `OCEAN` would go 215 → 276; not drawn.
- **The water's tone** (4174a). Over every water class the photograph is
  close to `0.6 · ours + 45`: the lit water and the screen reflection,
  read, not counted.
- **The beach's edge against the grass** (4178b's remainder). The card
  binds `sand.tga` in four transition-patch draws; whether our coast arm
  of `patch::Ground::combo` lays them on run1564 is not checked. The
  sawtooth stands.

## 4. The stack

The rule is `CLAUDE.md`'s **earn every dependency**: a crate enters when it
has real code, not in anticipation. The `sim` crate has no dependencies at
all. `rondata` has four: `flate2`, `lz4_flex`, `roxmltree` and `fixed`.

### 4.1 The options

**A. A browser client fed by a replay export.** This is what the lab viewer
already is: an HTML file with embedded JSON, drawn on a canvas, with no
network requests (`docs/DEBUG_VIEWER.md` §1–§2).

- *For:* no new Rust dependency, and it already exists.
- *Against:* it is a file, not a client. Exports are bounded at 200 records
  and 64 MiB. A three-dimensional model would mean writing WebGL in
  hand-rolled JavaScript with no libraries, because the page loads nothing
  external.
- *A variant:* compile `sim` to WebAssembly and run it live in the page. The
  crate has no dependencies, and its only operating-system calls are debug
  environment reads (`RON_DEBUG_*`, `RON_TURN_TABLE`). So
  `wasm32-unknown-unknown` is plausible. **It has not been tried.**

**B. A native two-dimensional isometric first pass** (sprites or flat shapes
through a small framebuffer crate).

- *For:* few dependencies, quick to a first image.
- *Against:* the original is three-dimensional. Its figures are `.BH3`
  meshes posed by `.BHa` animations, and its camera tilts and zooms
  (`Camera::adjust_tilt`, `Camera::zoom_to_level`). A sprite pass would be
  thrown away the day the first mesh draws, and it cannot be put in register
  with a tilting camera without reproducing the camera anyway.

**C. A native window with `wgpu`** (with `winit` for the window): Metal on
this machine, and Vulkan or Direct3D elsewhere.

- *For:* it is where three-dimensional meshes, the original's camera and
  phase 6's new art all end up.
- *Against:* it is the largest dependency tree the workspace would take on,
  around a hundred transitive crates. It buys nothing until there is a mesh
  to draw.

### 4.2 The recommendation

**Choice: no graphics dependency for the first three items, then C.**

- The counter (§5) is **headless**. The camera, projection, drawn set and
  draw list are pure functions in a new `render` crate that depends on `sim`
  and `rondata` only, and they run in `cargo test` with no display.
- The lab viewer stays the place to *look* at a disagreement while that is
  built. Teaching it to draw the client's draw list beside the original's
  reference is a small change to an existing diagnostic, not a new stack.
- **`wgpu` and `winit` are earned by the first decoded mesh** (§6, item 5),
  in a separate `client` binary crate. *Earned by item 4036 (§3.6): `wgpu`
  30 and `winit` 0.30; the lockfile grew by about 230 crates, all of them
  under `client`.* That keeps the testable core free of
  graphics, which is the same split as `sim` and `rondata` one level up.

*Alternative 1:* grow option A into the client. That is cheapest now, but it
ends in a JavaScript renderer the Rust core cannot test.

*Alternative 2:* B as a stepping stone. It costs an item and leaves nothing
behind.

*Alternative 3:* C from the first item. Then the dependency arrives an item
or two before anything uses it, which is what "earn every dependency" rules
out.

### 4.3 What the first dependency buys

`wgpu` buys: drawing the decoded `.BH3` meshes at their simulated positions
under a camera already proven in register (§5); terrain as a lit heightfield
from `tile_z` and the corner grid; and the eventual art swap, at no cost to
the core. `winit` buys a window and a camera the user can pan. Neither enters
`sim`, and neither enters `render`.

## 5. The counter

The loop books only what names a score (`CLAUDE.md`, working agreement). The
renderer needs a number a test can pin, a floor that can only rise, and a
denominator that comes from outside the work. Here are three candidates and a
recommendation.

### 5.1 Candidate A: frames in register

**Definition.** On a pinned capture, the client is driven by the free runner
(§2.4) from the start dump and the order stream. Score the consecutive
logged frames from frame 0 on which two things hold:

- every object the client draws has its integer screen anchor within **1
  pixel** of the original's projection (§3.3, rung 1) of the dumped position;
- the drawn set equals the original's (§3.3, rung 2).

Both are taken at a pinned camera and a pinned resolution.

**It measures:** that the snapshot is complete and faithful, and that the
camera, the heights and the visibility set are right.

**It does not measure:** anything about appearance. And it **saturates**:
once the projection matches, every frame of a lockstep capture agrees,
because the positions already did. It is a step function that moves from 0
to the capture's length in one or two items, and then has nothing left to
steer.

**Choice: a 1-pixel tolerance**, because both sides project the same
integers and only rounding separates them. *Alternative:* a tolerance in
world units, which is independent of the camera but hides a wrong zoom.

### 5.2 Candidate B: asset files decoded

**Definition.** The count of the install's files of each kind that decode
under a self-checking reader. The denominators are the counts in §3.2:
2,062 `.bha`, 1,847 `.bh3` and 2,901 `.tga`.

**It measures:** format coverage, with a denominator nobody chose.

**It does not measure:** whether anything is drawn, or drawn in the right
place. It also counts files the game never loads, such as editor leftovers,
the same as the citizen's walk.

### 5.3 Candidate C: scene silhouettes against the original's frame

**Definition.** For each pinned (capture, frame, camera), the overlap of
per-object footprints between the client's frame and the original's
screenshot, above a threshold.

**It measures:** what a player would see.

**It does not measure** anything until art is drawn; each point is a
capture, with §3.3's noise.

**The first row** (4038, `render::silhouette`, `client::silhouette`): 7 of
23 on the Aztec Modern pair's frame 3294 at run1559b's camera
(`RUN1559B_3294_MASKED`). **The photograph shows frame 3279** (4073): 16 of
24 there (`RUN1559B_3279_MASKED`); run1441's camera 0 of 24. The other
rows are their pins'.
- *Ours.* Each drawn object's world-space triangles are rasterized on the
  CPU with the client shader's arithmetic into a buffer of object keys,
  the nearest winning by camera-space depth.
- *The original's.* The photograph has no object ids, so its footprint is
  read by colour. Each object we draw gets a window: its pixels plus 6.
  A 6-pixel ring outside the window, where none of our objects lies, is
  taken as the ground. Its colours are binned at 32 levels a channel and
  dilated by one bin; a window pixel not of the ground's colours is the
  original's footprint. Both footprints are the whole scene's in the
  window: the photograph cannot say whose a crowd's pixel is.
- *The score.* An object counts when the intersection over union reaches
  0.5. An object is not scored when its ring meets the frame's edge or
  the interface (eleven rectangles by eye), or it shows under 40 pixels.
- *Made to fail.* run1441's camera, 0 of 25 (`RUN1441_CAMERA_3294`).
- *The second reader* (item 4071): a mask traced by hand on the photograph
  (`TRACED`, `Reader::Mask`), which reads neither our draw nor a colour,
  pinned before its first count. It reads both bare windows (`BARE`) empty
  and agrees with colour where colour passes. A robust ring is refused
  (`ROBUST_BARE`). The trace holds for this photograph alone (4071a).
- *The photograph's frame* (item 4073): the movers project nearest the
  photograph at **3279**, 3.0, 5.1 and 3.2 px (`PHOTO_FRAME`,
  `PHOTO_ANCHORS`, asserted), not the paused 3294 (4073a).
- *A traced region* (item 4077; the rule written before its count). An
  object's window and ring hold four kinds of pixel: (1) under an
  `INTERFACE` rectangle, neither ground nor footprint: left out of ours,
  theirs and the ground, since the photograph cannot know what it covers;
  (2) inside a trace of its own: its footprint; (3) inside another's
  trace: neither, else the traced object scores twice; (4) elsewhere: by
  colour, the ground the ring's kind-4 pixels where none of ours lies.
  Not scored: a ring at the frame's edge, or under 40 pixels outside the
  interface. Every trace is named by its object (`Reader::Traced`).
  Rejected: kind (3) as footprint, and the interface as before.
- **The row on the default draw**, the buildings as their sprites, by
  item 4077's reader (`RUN1559B_3279_TRACED`): **33 of 33** (4095;
  3279 whole 31, `_CURRENT`); its steps (4077–4095) are the
  pin's comment and the journals; treeless 30 of 33; run1441's camera
  0 of 32. The
  `RUN1559B_3279_*` rows above are the `zbuffer` draw's, kept by name.
- *The trees* (4080, before its count; §3.7). A tree hides what is
  behind it; its pixel is the ground's; it is never scored. Rejected:
  `TreePixels::Scene`.
- *The leaves* (4082, before its count; §3.7). A tree's fragment is kept
  where its bilinear texel alpha exceeds 0, culled counter-clockwise
  (`Leaves::Alpha`). Rejected: `Leaves::Solid`. The window does not cut.
- *The silhouettes* (4085, before its count; §3.7). Where a tree or a
  sprite is nearer than a land figure, the pixel is the figure's and the
  nearer depth stays; figures keep depth order among themselves
  (`Silhouettes::Drawn`; `render_states[1]`, 4085's journal).
  Rejected: `Silhouettes::None`.
- *The tilt* (4089, before its count; §3.7). A figure of `CAT` Mech or
  Artillery (`+0x14` 2, 3) turns by `misc_rot`, before its facing, to the
  negated `WorldData::find_slope_normal` at its point, taken whole, when
  that z is under 0.999 (`Tilt::Target`). Beside it: the ease at a draw a
  frame from 3200 (`Tilt::Eased`); kept, `Tilt::Level`.
- *The point* (4095, before its count). 3280's slots, clocks and
  facings at the 3279 points (`Blend::Last`).
- *The colour reader* (4099, before its count; `Hue`). On the row's
  draw, each object's pixels its own key holds that the traced reader
  calls the original's (kinds 2, 4); dark ones (sum under 60) left out.
  Each side as (r, g) / (r + g + b), which a white light's strength does
  not move; ours the texel, unlit. A pixel agrees within 0.05; an object
  passes on 40 lit pixels, half agreeing. **11 of 33** (`_TEXEL`), the
  tint's 2; run1441's camera 0. Texcoords as stored.
- *The team colour* (4102, before its count). The texel is
  `t·α + c·(1 − α)`, `c` the owner's `TEAM_UNIT` (`render_states[0]`'s
  `TOP_BLENDTEXTUREALPHA`); a reading, one photograph: **15 of 33**.
- *The silhouette's colour* (4103, before its count). A silhouette's
  pixel is `c·a + n·(1 − a)`, `c` the owner's `TEAM_UNIT`, `a` 128/255,
  `n` the nearer tree's texel or sprite's pixel (`render_states[1]`'s
  `SELECTARG2(TFACTOR)`); a reading, one photograph: **18 of 33**.
- *The constants* and interface are this frame's, pinned, never tuned
  to a count (the inputs: 4089's journal).

### 5.4 Recommended: pieces in register, K of M

**Definition.**

- **M** is the set of distinct graphic pieces (`gpiece`) the simulation's
  own run of the pinned captures plays, plus the building types those
  captures' dumps hold. The pieces are the simulation's, because only a
  `DUMP_ALL` dump prints `gpiece` (ANIM §1). They are checked against every
  `DUMP_ALL` dump of the same lobby on disk, where the two must agree. So M
  is counted from what the games show, not chosen.
- A piece **counts toward K** when two things hold:
  1. its `.BH3` model, and every `.BHa` its captured animation slots name,
     decode under the self-checking readers;
  2. on every logged frame of every pinned capture where an object plays it,
     that object's draw-list entry is in register in candidate A's sense.

*Ruled by item 4028; confirmed by the seventy-fifth steering pass (2026-11-09, `docs/DECISIONS.md` entry 115): condition 1 is the definition's own words read on what the run shows, and a building that is decoded but not drawn is not in register — nothing is stubbed true.*
- **Condition 1 is read on the slots the run plays**, as written. A slot
  the run never shows the piece in is not drawn, so its file is not asked
  (`PieceDecoding::decodes_on`, with slots from the runner's scenes, which
  equal the harness's). `artdata::PIECES_DECODING` stays the stricter
  count over every slot of every entry.
- **A building type's condition 1** holds when every piece the dumps print
  for it names a `<BUILD>` and decodes:
  - its `zbuffer` `.bh3`;
  - its own `anim`;
  - each `EXISTEVENT` addon's model and `anim` (`docs/FORMATS.md`, "The
    gaia pieces and the building pieces").

  An entry that names no model does not meet it: an empty list is not a
  decode.
- **The farm (type 417) is held to the files its own pass draws it
  from** (item 4037). Its `<BUILD>` names nothing. `FarmsOut::draw
  @008d75f0` builds a wheat field instead, from the `<FARM>` element and
  the tileset's wheat image (`docs/FORMATS.md`, the building pieces).
  So its condition 1 is that element read whole and that image decoding
  under `render::tga` (`artdata::farm_decoding`), on the tileset the
  lobby drew. The `<FARM>`'s own `texture=` is read and dropped by the
  loader, so it is not asked.
- **A building type's condition 2 is unmet, not vacuous**:
  - the dumps print every building's position on the logged frames, so
    there is a dumped frame to compare against;
  - ~~but the draw list carries no building entry to put in register.~~
    Since item 4035 it does (§3.5): a type is in register when every
    record of it the dumps print is drawn by both sides or neither and,
    drawn, sits within one pixel with the dump's piece.
  - **And only on records some side draws** (Loop 4049): a record drawn
    by neither must still agree, but it is no evidence of register. A
    type every record of which is drawn by neither is not in register,
    and the test names it (`types_in_register`, with each type's drawn
    count printed beside its records).

  A decoded building earns K only then. Nothing is stubbed true.

**The shape.** It reads like the rubric's "unit types seen 296 of 302". It
has a denominator from the oracle's own files. It rises one piece at a time,
so it steers for many items. And it embeds candidate A as its per-piece
condition, so frames in register is checked on every gate without being the
headline.

**Pinned** in a `render` test, with the scoreboard line parsed the way
`rondata::diff` parses the queue's. The queue's line is
`Renderer: K <k> of M <m> · frames <a> of <b> · pieces <c> of <d>`, built
from the pins by `rondata::diff::floors::renderer_line` and held by
`the_handoff_s_renderer_line_is_the_pinned_counter`, a commander's line.

**It measures:** that the client can resolve, decode and place everything the
scored games show.

**It does not measure:** textures, lighting, the correctness of an animation
pose beyond its frame index, the terrain's look, or any interface.

**The terrain is outside M** and has its own binary condition: the
heightfield's projected corner points are in register at the pinned camera.
*Pinned since item 4036 (§3.6): 3 of 3 frames, 174,243 of 174,243
corners.*

**Counted (item 4020, `rondata::free::tests`; re-counted by 4026).** On
`GreatLakesAztecModern`:
- **M = 59** (`RENDER_M`): 38 pieces the runner's 3,391 scenes play, with
  `-1` (no piece) left out, plus the 21 building types (`orig_type`) the
  lobby's three captures hold. 4020 counted 39 and M = 60. Piece 265 was
  played only by a unit at no health, which the door no longer reports
  (§2.3).
- **K = 58 of M's 59** (`RENDER_K`, the sum of `RENDER_K_PIECES` and
  `RENDER_K_BUILDINGS`, §3.5; 29 at item 4026, 31 at 4027, 38 at 4028, 58
  at 4035), with both conditions live:
  - 38 of the 38 pieces decode (condition 1, on the slots played);
  - all 38 are in register on every logged frame they are played on
    (condition 2). Item 4027 brought in pieces 232 and 11264.
  - ~~The 21 building types have neither a decoder nor a dumped frame, so
    they count toward no K.~~ Since item 4028, 20 of the 21 building types
    decode (`M_BUILDING_TYPES_DECODING`: 25 pieces, 80 files). ~~They count
    toward no K, because the draw list does not carry them (the ruling
    above).~~ Since item 4035 ~~all 21~~ are in register (§3.5), and the 20
    that decode count: `RENDER_K_BUILDINGS` 20. ~~The farm is short on
    condition 1 alone: its `<BUILD>` names no model (item 4037's).~~
  - Since item 4037, **21 of 21 decode** (81 files; the farm from
    `farm_wheat_1.tga`, `early_autumn`'s), and **20 of 21 are in
    register**. The farm's 402 dumped records were "drawn by neither",
    but only because `WallOut::draw` skips a finished farm. The original
    draws every one of them through `FarmsOut::draw`, and the draw list
    carries no wheat field. Counting it would stub condition 2 true, so
    it is out: `RENDER_K_BUILDINGS` (20, 21, 20, 21), K unchanged at 58.
    ~~The field's entry is parked (4037a).~~
  - Since item 4048, **21 of 21 in register**: the draw list carries the
    wheat field (§3.5), and the 402 farm records are drawn by both sides,
    all in register — `RENDER_K_BUILDINGS` (21, 21, 21, 21), **K 59 of M
    59**. ~~Every record counted is drawn (Loop 4049's rule).~~
  - Loop 4049 measured that rule, and it does not hold for four types:
    the Redoubt (446, 192 records), Hanging Gardens (528, 96), Terra
    Cotta Army (529, 96) and Angkor Wat (535, 8) are under construction
    on every dumped frame, below the share `WallOut::draw` draws a piece
    at, so neither side draws any record of them. The original draws
    their scaffold in `WallOut::draw_overlays@00641600`
    (`GraphicPieces::draw_scaffold@00907420`), which the draw list does
    not carry: the farm's shape before 4048. Under the rule the test
    reads `RENDER_K_BUILDINGS` (17, 21, 17, 21), K 55. **The seventy-sixth
    pass re-pinned it: K 55 of M 59** (`docs/DECISIONS.md` entry 116), a
    named fall of the counter and not of the picture, and the scaffold in
    the draw list is item 4061.
  - Since item 4061, **21 of 21 in register** again: the draw list carries
    the foundation and the scaffold (§3.5), every record of the four is
    drawn by both sides, and `RENDER_K_BUILDINGS` reads (21, 21, 21, 21),
    **K 59 of M 59**, under the rule that can fail.
- **The dumps check every figure they print a `gpiece` for.** Only the
  `DUMP_ALL` states print one: run1428's two, and run1429's start block.
  That is 306 figures on the map, all 306 agreeing, and on each dump's
  frames the two sides hold the same set of pieces (12 and 32 of the 39).
  run1429's stride and run1429b's window print guys without `gpiece`. The
  other pieces rest on the scene's equality with the harness.
- ~~**31 of the 38 pieces decode** under item 4019's readers
  (`M_PIECES_DECODING`; 32 of 39 before piece 265 left). The seven short are six gaia pieces, which no
  `<UNIT>` entry names, and one citizen crew row with a failing file.~~
  **38 of 38 since item 4028.**
  - The six gaia pieces resolve through the loader's own name,
    `{GRAPH}-TYPE{v}` (`unit_node_from_gpiece`), and decode.
  - 38322's missing `ceo_crew3_idle3.bha` belongs to `CHAR_IDLE2`, which
    the run never plays.

*What M is not yet:*
- one pair;
- figures of units on the map only. Five garrisoned figures fall outside it;
- ~~no building's own `gpiece` in the scene.~~ Since item 4035 the draw
  list carries one, resolved in `rondata` from what the door carries, and
  it is the dump's on all 4,564 records;
- no corpse or projectile art;
- the building half counts types made, not types drawn. A capital's
  upgrade is outside it: the runner's scenes hold 23 types, the dumps 21.

**When it changes.** Candidate C replaces it once K reaches M. Phase 6's
asset swap retires condition (1) and keeps condition (2).

**Choice: this counter.** *The alternative most worth weighing* is candidate
A alone, as a gate rather than a counter, with the order of items set by hand
instead of by a number. That is simpler, and honest for a phase that may
prove short. But it gives the loop nothing to book after the second item.

## 6. The first five items

In dependency order. Each item names the score it moves and what it needs
from `sim`. None is minted: item numbers and the queue are the commander's.

**1. The scene and the free runner.**

- *Builds:* `sim::observe` (§2.3) and the `rondata` free runner (§2.4).
- *Tests:*
  - on a pair (uncorrected from frame 1, entry 100 §4), every logged frame's
    `Scene` positions and `gpiece`/`anim`/`cur_time` equal the harness's on
    the same frame, with zero differences;
  - the soak digest is unchanged when `observe` runs;
  - the runner's peak memory is measured against the harness's.
- *Score:* the scene agrees with the harness on N of N logged frames of the
  pair: the precondition sub-score, pinned. The **M** of §5.4 is counted in
  the same item, from the runner's own scenes, and checked against the
  lobby's `DUMP_ALL` dumps.
- *Needs from `sim`:* the module, the version counters, and nothing in
  `tick`.
- *Built by items 4017 (`sim::observe`) and 4020* (the runner, §2.4's "As
  built"). The scene agrees with the harness on 3,390 of 3,390 ticks,
  35 of 35 logged frames. M = 60 (§5.4). The clock's first readings are
  beside the test.

**2. The camera, proven against the original's function.**

- *Builds:* a pure camera and projection in a new `render` crate. It is
  written from reading `BaseCamera::update_transforms` and
  `Camera::update_transforms`, never transcribed.
- *Tests:* it is checked against `MiscAccess::world_to_screen` run under
  unicorn on synthesized camera states (a sweep, as `crates/sim` runs its
  sweeps) and on a packet on disk.
- *Score:* camera states in register, K of the sweep, with every point within
  1 pixel.
- *Needs from `sim`:* nothing.
- *Built by item 4018* (§3.4): 240 of 240, and 394 of 394 points on
  run1484's packet.

**3. The draw list, and frames in register.**

- *Builds:* the headless draw list (object id, screen anchor, depth key,
  piece, slot, frame, facing) from a `Scene` and a camera.
- *Tests:* checked frame by frame against the reference built from the
  original's projection of the dumped positions and the drawn set from the
  dump's visibility.
- *Score:* frames in register on the pair (candidate A, kept as a floor), and
  the K of §5.4 counted with the decode condition stubbed true. The decode
  half arrives in item 4.
- *Needs from `sim`:* the viewer's fog plane in `Scene`, and the figures'
  `follow.body` positions, so a crew piece draws where its body is.
- *Built by item 4026* (§3.5): **33 of 35 frames in register** on the pair,
  and K 29 of M 59, both conditions live. The door gained `FigureScene.z`
  and dropped dead units (§2.3). Item 4027 took both to **35 of 35** and
  **K 31**: guy 0's figure reads its own body.

**4. The asset resolution, and the first decoder.**

- *Builds:* piece → `<UNIT>` → `.bh3` and `.bha` files (extending
  `rondata::artdata`); the `.BH3` container decoded to vertices, faces,
  nodes and bounds, self-checking; the remaining `.BHa` key fields.
- *Evidence:* each claim lands in `docs/FORMATS.md` with what proves it: the
  PDB's types, the chunk arithmetic, and a round trip against ptasev's
  converter's reading of the same file, which is a reading aid and not a
  source.
- *Score:* the §5.4 K, now with both conditions, rising per piece decoded.
- *Needs from `sim`:* nothing new.
- *Built by item 4019*, the decode half: `render::bh3`/`bha` and
  `artdata::unit_art`/`piece_decoding`. **Pieces decoding 1,439 of 1,452**
  (`artdata::PIECES_DECODING`), the 13 short named by file. The decoders
  sit in `render` rather than `rondata` (§3.1's choice is amended):
  `rondata` depends on `sim`, `render` must not, and `rondata` now
  depends on `render`.
- *Item 4028, the rest of the resolution:*
  - the gaia pieces (`artdata::bird_pieces`): pieces decoding 1,465 of
    1,478;
  - the building pieces, through the loader's name
    (`artdata::build_piece_name`, `build_art`, `building_decoding`): 20 of
    M's 21 building types decode;
  - condition 1 is read on the played slots: K 38 of M 59.

**5. First pixels.**

- *Builds:* the `client` crate (`winit`, `wgpu`). It draws the terrain
  heightfield in flat land-type colours, and each figure's decoded mesh at
  its draw-list anchor and facing, untextured, at the pinned camera.
- *Compared with:* a screenshot of the original at the same frame and camera,
  taken on the capture lane and kept outside the repository. The comparison
  is by eye at this stage. Candidate C's silhouette metric is the item after.
- *Score:* the terrain's binary condition (§5.4), and K unchanged. This item
  makes the counted pieces visible, and that is the dependency's earning.
- *Needs from `sim`:* `observe_terrain` (heights, land types and mountain
  placements).
- *Built by item 4036* (§3.6): the `client` crate, the door's corner grid,
  the terrain condition pinned at 3 of 3 frames, K unchanged at 58 of 59,
  and the session trace with the clock read from it.

**After these**, in the order the counter's misses name them: textures and
team colours, animation poses, buildings' construction stages, territory
lines and fog shading, projectiles and corpses, then candidate C.

## 7. What is not established, and what needs Ramon's word

### 7.1 Not established

- What parts tank 1/27 from the photograph once its draw's whole state and normals are the original's, and the
  tone mapper's every input is live (§3.14, 4168): the texture as sampled, the frame's mean through clipping, or
  FXAA. Bloom is bounded out. And stage 1's `MODULATE4X` alpha, blended over what is behind, which no draw of
  ours makes (4165b). Settled by the luminance target's mip 10 read off the card on that pass, FXAA drawn, and
  the blend drawn and counted (4168b).
- ~~The water's normal, which its vertices do not carry (§3.12, 4168a). Settled by the water VB's input layout
  in `d3dgl.dll` and the vertex shader's default for a missing normal.~~ Settled by item 4174: `modify_water`'s
  mesh, the one drawn, carries (0, 0, 1) at all 1,755 vertices on run1564's packet (§3.12).
- The water's red: the photograph's lake reads r ≈ 40, and no packet input to the floor or the water carries
  any (§3.12, 4174a); the screen reflection's reach is read, not counted (item 4178). ~~Its alpha past
  `create_water_verts` (4174b).~~ Settled by item 4178: 597 of 597 drawn corners (§3.12, "The water drawn").
  The shore's blend, `sand.tga`'s own layer and its patches, is not drawn (§3.12, 4178b).

- ~~**What the client's own pixels show**~~ (the pass of 2026-11-12, DECISIONS 127): counted on the still by
  item 4171 (§3.15); the wash was the silhouette's place and the sRGB target. ~~The trees
  stay flat green (no texture bound, parked 4038a)~~: textured by item 4173 (§3.7, "The trees' own colour").
- ~~**How bright anything is.**~~ The readers compare (r, g) / (r + g + b). By box on those stills the mountain's
  body sums 130 against the photograph's 392, and the lake 506 against 316; a mountain's body and a tree are in
  no row. ~~Settled by item 4172's luminance reader~~: on the painter (§5.3, "The luminance reader") the figures
  read 20 of 33, the ground 1,075 of 1,109 with the body in, run1564's 791 of 1,920; the body's 130 was the
  client's ADDSIGNED in linear space (§3.10), 357 after. ~~The trees are item 4173's~~: 54 of 56 (§3.7).
- **Whether the client's other blends part from the painter's on brightness** (4172a): the textured pass's
  team mix and the hardware blends of the silhouette, the patch and the mountain's layer run in the sRGB
  target's linear space, the painter's on the stored bytes. Settled by item 4171's rows on the still.
- **Why the medium tanks read 0.70–0.73 of the photograph's brightness** as drawn and darker lit (4172b), where
  the riflemen read 1.1–1.5 drawn and agree lit. The texture as sampled is 4168b's; the lit candidate's diffuse
  by the stored normals (4122) is uncounted on brightness.

Each line names what would settle it.

- **Whether the long captures run uncorrected past their start block.** The
  pairs do (entry 100 §4), and on `GreatLakesAztecModern` the free runner
  agrees with the harness on every tick (item 4020). For the long ones, run
  the same comparison on one.
- ~~**Whether `o` is reused within one game.**~~ It is (item 4026, §2.3):
  `1/68` on the pair. A client keys on `(who, o)` within a frame.
- ~~**How many pieces M holds.**~~ 38 on the first pair, and M = 59 with
  the building types (items 4020 and 4026, §5.4). Whether every lobby's pinned capture
  has a `DUMP_ALL` sibling to check its count against is still a grep of the
  archive. On this lobby, only the start states print a unit's `gpiece`.
  ~~Still a grep~~: every long capture's closing block prints it (item
  4166, §3.13), and on 110 lobbies the runner's pieces are the dump's on
  34,394 of 34,399 records. The five are the Assault Marines' variant
  (parked 4166a).
- **Whether the original interpolates figures between simulation frames**
  when it draws. In `GuyOut::draw`, ~~the place to read~~ the anchor's point
  is not interpolated and the bank is (§3.5). Whether the mesh's own
  transform is interpolated is still unread past that function's head.
  It does not affect the counter, which compares at frame boundaries. It
  does decide whether the client's motion looks like the original's.
- ~~**`world_to_screen`'s inputs beyond the camera.** It takes two `Coord`s and
  two more integers. Whether height enters through those, or through a
  lookup inside, is a reading item 2 owes.~~ Settled by item 4018 (§3.4):
  `(x, y, z, w, h)`, the height the third integer, the outputs in `ecx`
  and `edx`.
- **The `.BH3` encoding, every TGA variant past type 10, the terrain
  textures' mapping, and the 16 PNGs** (§3.2). ~~The terrain textures'
  mapping~~: a `BASELAND` cell's base layer is read (item 4127, §3.6,
  "The ground's base layer") and a mountain's (§3.10); the blends, the
  coast, the cliffs and the water are not, and frame 3279 shows no tile of
  the last three (item 4143, `RUN1559B_3279_UNSCORED`; parked 4143a).
  **run1564's photograph shows two of them** (item 4160,
  `RUN1564_3279_CENSUS`): in view, 488 `OCEAN` tiles (437 scoreable),
  901 `SANDY` (772) and 32 `ROCK` (32). No cell of Great Lakes carries
  `CLIFF`, so no photograph of this map holds a cliff. The ground's row
  on it, its rule unchanged, is 709 of 1,920 (`RUN1564_3279_GROUND`).
  The land agrees on 709 of 711. The water and the shore agreed on 0 of
  1,209 in the debug colours; drawn as §3.12 reads them, 527 (item
  4164, 1,236 of 1,920). The
  shoreline is in register. No plain tile is photographed blue, and no
  `OCEAN` tile is photographed otherwise. With the camera moved two tiles
  there are 0 to 3 such tiles, at four up to 40, and at eight up to 142
  (`RUN1564_3279_SHORE`). This places the camera to about two tiles, the
  width of the beach, and no closer. Not
  established by it: the water is drawn at its surface and our tile is
  the seabed, so a water pixel's tile is the seabed's projection
  (parked 4160a); and the photograph's frame is taken as 3279 without a
  mover to clock it (parked 4160b).
- **The scaffold's own draw** (§3.5, parked 4163a): `draw_ammo`'s
  `Scaffold Blend`, the crane's `CraneWork` pose from the building's clock,
  and `GraphicPieces::draw`'s gate on `draw_flags` for the ammo range; the
  client draws it opaque, at rest, always. The dock's foundation model
  beside its sprite (parked 4163b): the client draws the sprite alone.
- **Whether `sim` builds for `wasm32-unknown-unknown`.** It is untried, and
  matters only if option A's live variant is wanted.
- **The licence of ptasev/Rise-of-Nations.** Reading it is fine under the
  project's rules. Anything beyond reading would need its licence checked.

### 7.2 Ramon's word

1. ~~**The counter.** Pieces in register K of M (recommended), frames in
   register alone, or another.~~ K of M, in use since item 4020; the queue's
   `Renderer:` line carries it from the seventy-fifth pass (entry 115).
2. ~~**The stack.** Headless first and `wgpu` at the first mesh (recommended),
   or the browser client grown, or `wgpu` from the start.~~ As recommended
   (entry 114 §4); `wgpu` and `winit` are earned by §6 item 5, booked at the
   seventy-fifth pass.
3. ~~**Replay viewer first, or playable first.** Playable needs order minting
   and a game start without the original's dump. Both are simulation work,
   and the first is the larger one.~~ Playable, by the finish line (entry 114
   §1); the viewer is the road there, and the two simulation halves are the
   `start` and `orders` tracks (`docs/TRACKS.md`).
4. ~~**Starting a game without the dump.** Is the rest of map generation's
   chain (`docs/MAPGEN.md` §15) and the start block booked alongside phase 4,
   or after it?~~ Alongside: the `start` track opens on the sim lane at the
   seventy-fifth pass, once the structure's last two items land (entry 115).
5. ~~**The simulation track beside the renderer.** Entry 110 §3's
   recommendation runs the held-out style beside phase 4. Does it run, and on
   how many lanes?~~ Two lanes, one of them the sim's (entry 114 §3); the
   held-out style landed (MAPGEN §24) and the sim lane's word is the `start`
   track.
6. ~~**The crate layout.** A headless `render` crate plus a `client` binary,
   and decoders in `rondata` (recommended), or another split.~~ As
   recommended, with the decoders in `render` (item 4019's amendment, §6
   item 4): `rondata` depends on `render`, never the reverse.
7. **Which viewer the client shows by default:** one player under fog, or an
   omniscient observer. The counter needs one pinned, and the default
   proposed is the human's seat, whose fog the dump prints. *The seventy-fifth
   pass's ruling, Ramon's to overturn:* first pixels (§6 item 5) show the
   observer, because K's register half is counted on the observer's view and
   no logged frame of the pair prints a fog plane (parked 4026a); the human's
   seat becomes the default when 4026a checks the viewer's drawn set against
   the original.
8. **How much interface comes in with the client** beyond a debug overlay:
   selection outlines, health bars and a resource line. `UnitOut::draw`'s
   neighbours (`draw_health_bar`, `draw_unit_circles`) are drawing, not
   `iface*` windows. Still, every one is scope. *The seventy-fifth pass's
   ruling, Ramon's to overturn:* a debug overlay only — the frame number, the
   clock and the camera — until candidate C is counted; every drawn element
   that is not a piece or the terrain is scope the counter cannot see.
