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
| `.BH3` model (1,524 in `art/`, 323 in `terrain art/`) | `render::bh3`, `rondata::artdata` | **Since item 4019, the encoding**: vertices, normals, texcoords, faces and the node tree, as `BH3Method::load_bh3@009156f0` reads them, self-checking; 1,845 of 1,847 decode, and the two that fail are named (FORMATS, "The model files"). Which file a piece plays: `<UNIT … model="…bh3">` (`artdata::unit_art`). | The normals' fourth word, and the transform fields compared against a loaded `HierObj`. |
| `.tga` texture (2,171 in `art/`, 730 in `terrain art/`) | `rondata::mountains` | Type 10 (run-length), 32-bit, descriptor `0x08`, bottom row first: the mountain templates (FORMATS, "The mountain templates"). | Every other TGA variant the install uses, which textures a model binds, and how terrain textures map to land types. |
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
   client's frame on object footprints, never on pixels.

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
a scaffold piece), the slot's angle, and the sprite and meshes themselves:
the client skips both poses (parked 4061a, 4061b).
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
  reached at `times[k − 1]` (`HierObj::set_to_time@00536e30`). Posed, a
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

**The draw.** Each tree is its model at rest. The window draws it flat green, and `--no-trees` leaves it out. In
candidate C's key buffer a tree is rasterized after the sprites, its fragments cut by the leaves' alpha and cull
above (`Leaves::Alpha`, the default), so it hides what stands behind it, and it is scored by §5.3's tree rule. A tree is not a draw-list piece, so K's M (§5.4) is not widened and K holds at 59 of 59.

**The count** (§5.3): **29 of 33 → 25 of 29** as solid cards (4080), **→ 26 of 29** with the leaves cut (4082):
0/6 0.348 → 0.513. Before the count, on our buffers: of the five figures' pixels a tree's triangle covers, the
leaves' texels cut 12, all 0/6's, and on every covered pixel the tree is the nearer, by 190–1,300 camera units.
So four figures by the woodcutter, which the photograph shows in front of the pine, stay hidden (parked 4082a).
The rejected readings, `TreePixels::Scene` and `Leaves::Solid`, read 26 and 25 of 29.

**What is not established.**
- ~~The leaves' alpha~~: read and applied, above (item 4082).
- Why four figures the photograph shows in front of the pine stand behind it in ours: a figure's depth or
  position against the tree, or the photograph's reading (parked 4082a).
- The blend: the key buffer keeps a fragment whole where its alpha passes; the original blends it, which moves
  colour, not a footprint.
- The depth of a building's sprite against a tree. Ours puts the anchor's flat draw-list depth against the tree's
  per-pixel depth. The original's sprite may carry its `zbuffer` model's depth (parked 4076b).
- The tree's idle animation, which is not posed.
- The section name `#<TILESET>_TREE_MODELS0`. It is a reading, checked by the count of models (24, all of
  which decode), and since by the packet's geometry on 12 cells.
- Whether cutting a tree moves `terrain_version`.

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

**It does not measure** anything until art is drawn. It also needs a capture
for every data point, and it inherits the translation stack's noise (§3.3).
It is the art phase's counter, not the first one.

**The first row, pinned** (item 4038, 2026-11-09, Opus 5.5;
`render::silhouette`, `client::silhouette`). **7 of 23 objects** on the
Aztec Modern pair's frame 3294 at run1559b's camera, against that take's
photograph, drawn as the client draws by default: posed, at the
original's facing, with the tanks read by the hand-traced mask
(`RUN1559B_3294_MASKED`, item 4071: 6 → 7). By colour alone it reads 6
(`RUN1559B_3294_POSED`), and drawn at rest 2 (`RUN1559B_3294`).
**The photograph shows frame 3279** (item 4073): on frame 3279, at the same
camera, readers and constants, the row reads **16 of 24**
(`RUN1559B_3279_MASKED`), 13 by colour (`RUN1559B_3279_POSED`) and 4 at
rest (`RUN1559B_3279`); run1441's camera reads 0 of 24 under both readers.
The 3294 rows are kept as the packet's frame.
- *Ours.* Each drawn object's world-space triangles are rasterized on the
  CPU with the client shader's arithmetic into a buffer of object keys,
  the nearest winning by camera-space depth.
- *The original's.* The photograph has no object ids, so its footprint is
  read by colour. Each object we draw gets a window: its pixels plus 6.
  A 6-pixel ring outside the window, where none of our objects lies, is
  taken as the ground. Its colours are binned at 32 levels a channel and
  dilated by one bin, and every pixel in the window whose colour is not
  the ground's counts as the original's footprint. Both footprints are
  the whole scene's inside the window, since the photograph cannot say
  which figure of a crowd a pixel belongs to.
- *The score.* An object counts when the intersection over union reaches
  0.5. An object is not scored when its ring meets the frame's edge or
  the photograph's interface (eleven rectangles placed by eye, among them
  the city's label and the selected figure's portrait), or when it shows
  fewer than 40 pixels.
- *Made to fail.* The same frame at run1441's camera, which the
  photograph was not taken at, scores 0 of 25 (`RUN1441_CAMERA_3294`).
- *The facing* (item 4069, §3.6): the drawn map was a reflection of the
  original's in world y, a sign flip, not an offset; posed, 1 → 6 of 23
  (the rows: item 4069's journal).
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
  (2) inside a trace of its own: its footprint; (3) inside a trace of
  another object: neither, since the trace says whose the pixel is, and
  counting it under a neighbour scores the traced object a second time
  under the neighbour's name; (4) elsewhere: by colour, the ground being
  the ring's kind-4 pixels where none of ours lies. An object is not
  scored when its ring meets the frame's edge or fewer than 40 of its
  pixels lie outside the interface. Every trace is named by its object,
  tanks and figures included (`Reader::Traced`). The rejected readings,
  reported beside it: kind (3) as footprint, and the interface as before.
- *The misses* (on 3279). ~~The capital is drawn as its `zbuffer` model
  (4036a).~~ The figures by the derrick: 4082a.
- **The row on the default draw**, the buildings as their sprites, by
  item 4077's reader (`RUN1559B_3279_TRACED`): 27 of 33 (4077), 29 of 33
  (4079, tank 1/27 traced), 25 of 29 with the trees (4080, §3.7) and
  **26 of 29** with the leaves cut (4082); treeless 29 of 33; run1441's
  camera 0 of 28. Each step's rows: its journal.
  Out: three riflemen (4079b) and the figures the trees hide (4082a). The
  `RUN1559B_3279_*` rows above are the `zbuffer` draw's, kept by name.
- *The trees* (4080, before its count; §3.7). A tree hides what is
  behind it; its pixel is the ground's; it is never scored. Rejected:
  `TreePixels::Scene`.
- *The leaves* (4082, before its count; §3.7). A tree's fragment is kept
  where its bilinear texel alpha exceeds 0, culled counter-clockwise
  (`Leaves::Alpha`). Rejected: `Leaves::Solid`. The window does not cut.
- *What it does not measure.* Colour: textures and team colours are not
  bound (4038a).
- *Where the inputs are.* The photograph is converted once to `.tga`
  (`sips -s format tga`), so `render::tga` reads it with no new
  dependency. `client --silhouette OUT.bmp` prints every row and writes
  the scoring over the photograph. The reading constants and the
  interface rectangles are this frame's, chosen once and pinned, never
  tuned to a count.

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
  textures' mapping, and the 16 PNGs** (§3.2).
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
