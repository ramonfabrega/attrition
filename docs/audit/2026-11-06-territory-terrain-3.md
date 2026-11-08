# Territory reading: group `terrain-3` (the `terrainvis` stem)

The seventy-first tranche's reading fan-out (READER_BRIEF, 2026-11-06), read on **Opus 5.5**.
Seventeen functions of `TerrainVis`, the terrain renderer's vertex-stream and visibility cache.
`entered` counts the lockstep runs the group file lists (`-` none; the file lists at most six).
The decompile is `~/ghidra-projects/decomp`; crate citations are at `41c823576`.

**The class is the renderer's, settled by its type record.** `types.txt` gives `TerrainVis`
(size 0x228) as a member of `TerrainOut` (`+0x52fc terrain_vis`), and every one of its fields is
a draw resource or a draw-loop stamp: `world_streams` (`VertStream *`), `world_ibs_9a/9b/25a/25b`
and `deco_ibs` (`IndexBuffer`), `source_pools` (`RenderHelper *[7]`), `source_fvf` (vertex
formats), `verts`/`colors`/`faces`/`tex0`/`tex1`/`norms`/`tangents`, `vis_list` and
`last_moved_frame` (stamped with `Game::graphic_loop`, the render counter, never the sim frame),
`cur_zoom`/`zoom_changed` (the camera's zoom), and `fog_transition_bucket`. Nothing in the class
is read by a simulation function: every caller in the export is a renderer (`TerrainOut::render`,
`TerrainOut::render_fow`, `TerrainOut::inc_fog_time`, `RiverSectionOut::render`,
`TerrainBorder::render`, `CoordInfo::add_geometry`) or another `TerrainVis` method, with one
exception, `Terrain::object_placed`, which only invalidates the cache. `docs/MAPGEN.md` §12
already calls the stem "the renderer's vertex-stream and visibility cache", and `docs/ROADS.md`
§7.4 calls `object_placed`'s `invalidate_wcoord` arm "visual and unmodelled by design".

**The fog here is the drawn fog, not vision.** `fill_fog_texcoords` reads the global `fog`'s
`fog_lookup` with stride `fog_xs + 2` — the shape `WorldMapFog::get_fog_lookup@008d1830` answers
(`(y+1)*(xs+2) + x + 1`, swept in `crates/sim/tests/sweep/world_map_fog_get_fog_lookup.rs`), the
renderer's padded copy — and writes only `f32` texture coordinates into a `FogTransition` and the
terrain's vertex buffer. The simulation's vision is `World::seen`/`seen2` (`docs/VISION.md`), which
no function of this group reads or writes.

**Why the lockstep runs enter them.** The traced original renders while it plays; a lockstep
trace's function coverage records every function the process executed, so these are entered on
every captured game by the draw loop, not by a rule.

**Proposed census mark for every row: class `Vis$`** (or the literal class `TerrainVis`). The
INDEX has exactly one class ending in `Vis` — `TerrainVis`, 22 functions: the 17 below plus
`clear`, `close`, `init`, the constructor and destructor, which `is_dead_weight` already catches.
An alternative with wider reach: extend `is_ui_body`'s `cls.endswith("Out")` to "a class that is
a member of an `*Out` class in `types.txt`", which catches `TerrainVis` and would catch any other
helper a renderer embeds; the literal regex is the cheap one.

## Part 1

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 008b0080 | TerrainVis::move_stream_bucket | open | 6 | ui | Moves the vertex bucket at the land type's `highwater_mark` into a freed slot (`VertStream` vtable +0x14, a copy of `verts_per_bucket` vertices), stamps `vis_list[slot] = graphic_loop`, swaps `bucket_owners`, and walks the high-water mark down to the last bucket stamped this draw. Sole caller `squeeze_cache`. Mark: class `Vis$`. |
| 008b0190 | TerrainVis::squeeze_cache | open | 6 | ui | Compacts one land type's vertex stream: every bucket whose `vis_list` stamp is not this draw (`graphic_loop`, or `last_moved_frame` on a re-draw) is overwritten by `move_stream_bucket`; the fog-transition arm moves `fog_transition_bucket` alongside; ends with `world_render_len = (highwater+1) * verts_per_bucket` and the stream's lock/unlock (vtable +0x28). Sole caller `build_index_buffer`. Mark: class `Vis$`. |
| 008b0300 | TerrainVis::build_index_buffer | open | 6 | ui | For the base and last land types calls `squeeze_cache`; for the five decal types appends each visible bucket's faces (`SimpleArray<ushort>::append_with_offset`, offset `slot*0x19`) into `faces` and uploads them to the type's `IndexBuffer` (created through `graphsys` vtable +0xac when too small). Sole caller `set_pending_wcoords`. Mark: class `Vis$`. |
| 008b26f0 | TerrainVis::set_pending_wcoords | open | 6 | ui | The per-draw flush (bracketed by `profile` zone 0x47): per land type, trims `highwater_mark` to this draw's stamps, gives every pending `CoordInfo` a block (`get_free_block`), stamps `stream_block_id`/`last_vis_frame`/`vis_list` with `graphic_loop`, copies its vertices (`move_data_to_stream`), clears the pending list and rebuilds the index buffer; then `last_moved_frame = graphic_loop`. Sole caller `TerrainOut::render@008558b0`. Mark: class `Vis$`. |
| 008b2960 | TerrainVis::get_ib | open | 6 | ui | Flips the double-buffered index buffer (`world_ibs_9a/9b` or `_25a/_25b` by `verts_per_bucket`, 0x18 or 0x60 indices a bucket), sizes it to the high-water mark, creates or locks the land type's `VertStream` with its `source_fvf`, and returns `cur_ib`. Callers `TerrainOut::render`, `TerrainOut::render_fow`, `RiverSectionOut::render`, `TerrainBorder::render`. Mark: class `Vis$`. |
| 008b3190 | TerrainVis::set_zoom | open | 6 | ui | Camera zoom change: resets every land type's `last_scan_start` and `highwater_mark` (0xffff) and `last_moved_frame` (0x7fffffff), keeps `cur_zoom` unless the argument is 4 or 5, picks the 9- or 25-vertex index buffer pair, and on 5 calls `clear`. Sole caller `TerrainOut::render`. Mark: class `Vis$`. |
| 008b2eb0 | TerrainVis::add_visible_wcoord | open | 6 | ui | Marks a terrain cell visible this draw: if already cached for the current `last_moved_frame` it only restamps `graphic_loop`; else, for the fog land type, it fills a `FogTransition` (`fill_fog_texcoords`, `FILL_CURRENT`) and writes its 25 `f32` texcoords into the terrain's vertex buffer (`Terrain +0x1a80[+0x1ad0]`), then appends the cell to `pending_list`. Sole caller `CoordInfo::add_geometry@0084bef0` (ruled `ui` by group `terrain-2`). Mark: class `Vis$`. |
| 008b19f0 | TerrainVis::fill_fog_texcoords | open | 6 | ui | Reads a 4x4 window of the renderer's fog table (`fog.fog_lookup`, stride `fog_xs + 2`) and maps it through `fog_tcoord_lookup` to 25 `f32` texture coordinates; on `FILL_ALL` also reads the current ones back from the vertex buffer and stores per-coordinate deltas scaled by the scene's frame rate (`MiscAccess::scene`) and clamped, for the fog fade. Pure float, no sim write. Callers `TerrainOut::inc_fog_time@0086d8e0`, `add_visible_wcoord`. Not vision: `docs/VISION.md`'s `seen`/`seen2` are untouched. Mark: class `Vis$`. |
| 008b0460 | TerrainVis::move_data_to_stream | open | 6 | ui | 11,770 bytes, read by its calls and fields: copies one `CoordInfo`'s vertices, colours, normals, tangents and two texture-coordinate sets (`TexArray::add`, `source_pools`, `cur_pass`, `deco_faces`) into the land type's `world_streams` block; `float` locals throughout. Sole caller `set_pending_wcoords`. Mark: class `Vis$`. |
| 008b0ce0 | TerrainVis::get_free_block | open | 6 | ui | 9,695 bytes, read by its calls and fields: finds a free bucket in a land type's stream by scanning `vis_list` from `last_scan_start` for a stamp older than this draw, else grows `highwater_mark`; on first use builds the static 9- and 25-vertex face templates (`FaceArray`, the `Ftemp9`/`Ftemp25` statics its unwind funclets name) and the index buffers through `graphsys`; `Error::report` on overflow. Sole caller `set_pending_wcoords`. Mark: class `Vis$`. |

## Part 2 — the `layout` rows (an emulator sweep, no consumer)

Each of these seven has a `crates/sim/tests/sweep/terrain_vis_*.rs` test that runs the original
under unicorn against an inline restatement of its byte writes; nothing in `sim` outside `sweep/`
consumes them, which is what `layout` means. The sweeps are correct as regressions of the
emulator lane and say nothing about the game; a census mark of `ui` on the class would leave
them standing and take the rows off the open list.

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 008b0060 | TerrainVis::get_verts_per_bucket | layout | - | ui | `sys_grade > 2` (the graphics-quality grade) → 9, else `verts_per_bucket[cur_zoom]`: vertices per terrain bucket by zoom. No caller in the export; the same expression is inlined in `move_stream_bucket`, `squeeze_cache`, `get_ib`. Sweep `crates/sim/tests/sweep/terrain_vis_get_verts_per_bucket.rs` (item 1942). Mark: class `Vis$`. |
| 008b13b0 | TerrainVis::wcoord_was_cached | layout | - | ui | `CoordInfo.last_vis_frame[i] == last_moved_frame`, a draw-stamp compare. No caller in the export (inlined into `add_visible_wcoord`'s first test). Sweep `terrain_vis_wcoord_was_cached.rs` (item 1932). Mark: class `Vis$`. |
| 008b13d0 | TerrainVis::invalidate_wcoord | layout | 6 | ui | The `CoordInfo *` overload: for the cell and its eight neighbours (`move_x`/`move_y`, bounded by the world's `xs`/`ys`) sets the cached draw stamps `CoordInfo +0x6c` and `+0x70` to -1 (the latter conditional on the fog bucket), forcing a vertex rebuild. Caller `Terrain::object_placed@00850c40` (with a `CoordInfo *` from `Terrain +0x6ab0`) — the arm `docs/ROADS.md` §7.4 rules visual. Sweep `terrain_vis_invalidate_wcoord_1.rs` (item 2927). Mark: class `Vis$`. |
| 008b2e80 | TerrainVis::invalidate_wcoord | layout | 6 | ui | The `(WCoord *, WCoord *)` overload: looks up the cell's `CoordInfo` in `Terrain +0x6ab0` and tail-calls the overload above. No direct caller in the export besides the inlined path; entered by the lockstep runs through the draw loop. Sweep `terrain_vis_invalidate_wcoord.rs` (item 2655). Mark: class `Vis$`. |
| 008b1e60 | TerrainVis::invalidate_fog_wcoord | layout | - | ui | Sets the cell's `CoordInfo +0x70` (the fog bucket's draw stamp) to -2 when it equals `last_moved_frame` or is already -2, else to -1. No caller in the export; the fog-cache invalidation path that does run is `WorldMapFog::fill_colors_fog@008d1420` → `TerrainOut::invalidate_fog_cache@00866980` → `TerrainOut::invalidate_fog_cache_wcoord@00866800`, all renderer (it sets `scene->fog_changed`). Sweep `terrain_vis_invalidate_fog_wcoord.rs` (item 2655). Mark: class `Vis$`. |
| 008b2a80 | TerrainVis::set_base_source_pool | layout | - | ui | `source_pools[0] = RenderHelper *`, `source_fvf[0] = vertex format`. No caller in the export (inlined into `TerrainOut`'s set-up). Sweep `terrain_vis_set_base_source_pool.rs` (item 1942). Mark: class `Vis$`. |
| 008b2aa0 | TerrainVis::set_extra_source_pool | layout | - | ui | `source_pools[i+1]`, `source_fvf[i+1]` for decal pool `i`. No caller in the export. Sweep `terrain_vis_set_extra_source_pool.rs` (item 1966). Mark: class `Vis$`. |

## Summary

- **Counts by ruling** (17): `ui` 17. No `twin`, `absorbed`, `replaced` or `unmodelled` row.
- **Proposed census class**: class regex **`Vis$`** (equivalently the literal `TerrainVis`) as a
  `ui` mark beside `is_ui_body`'s `*Out`. It catches all 22 `TerrainVis` functions and nothing
  else in the INDEX: the 17 rows here (`move_stream_bucket`, `squeeze_cache`,
  `build_index_buffer`, `move_data_to_stream`, `get_free_block`, `fill_fog_texcoords`,
  `set_pending_wcoords`, `get_ib`, `add_visible_wcoord`, `set_zoom`, `get_verts_per_bucket`,
  `wcoord_was_cached`, both `invalidate_wcoord`, `invalidate_fog_wcoord`,
  `set_base_source_pool`, `set_extra_source_pool`) and the five `is_dead_weight` already has
  (`clear`, `close`, `init`, constructor, destructor). Evidence: the class's type record holds
  only draw resources (`VertStream`, `IndexBuffer`, `RenderHelper`, vertex arrays) and render
  stamps, and it lives inside `TerrainOut` at `+0x52fc`. A wider variant — "a class embedded in
  an `*Out` class's type record" — would generalise the `*Out` rule; not measured here.
- **Proposed replaced groups**: none.
- **`unmodelled` rows**: none. The six-run lockstep entries are the original's draw loop
  (`TerrainOut::render` → `set_pending_wcoords`, `set_zoom`, `get_ib`; `CoordInfo::add_geometry`
  → `add_visible_wcoord`), not a rule; every write lands in `TerrainVis`, a `CoordInfo` draw
  stamp (`+0x6c`, `+0x70`, `last_vis_frame`, `stream_block_id`) or a GPU buffer, none of which a
  simulation function reads.

## Not established

- The renderer-only claim for `CoordInfo +0x6c`/`+0x70`/`+0x78`/`+0x7a`/`last_vis_frame` rests on
  this group's writers and readers plus group `terrain-2`'s ruling of `CoordInfo::add_geometry`;
  a full grep of every reader of those `CoordInfo` offsets across the export was not run.
- `Terrain::object_placed` addresses the cache as `Terrain +0x533c` while `types.txt` places
  `terrain_vis` at `TerrainOut +0x52fc`; the 0x40 gap (a base-class offset between `Terrain` and
  `TerrainOut`, presumably) was not reconciled, and the decompile's single stack argument there
  does not by itself say which `invalidate_wcoord` overload is called. Neither changes the ruling.
- That the global `fog` read by `fill_fog_texcoords` is the `WorldMapFog` instance rests on the
  matching stride (`fog_xs + 2`, padded) with `WorldMapFog::get_fog_lookup`; the global's type
  record was not opened.
