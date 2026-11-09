# Territory reading — group `rest-1` (2026-11-06)

The seventy-first tranche's reading fan-out: one ruling per function of group
`rest-1` (60 functions over the small stems `ammo`, `animal`, `armies`,
`caravans`, `caster`, `categories`, `circle`, `cities`, `collblock`,
`collcheck`, `coord`, `deathobj`, `dock`, `docks`, `fogmaster`, `fort`,
`forts`, `gameaccess`; `tools/rubric.py --tables 0`'s open territory). Read by
name from the decompile export (`~/ghidra-projects/decomp/funcs/<Class>/<name>@<va>.c`)
against `crates/sim` and the mechanic documents. Reader: Opus 5.5. "entered" is
the group file's `entered_by_lockstep` column (`L` = a run1016–1022 twin set,
the `cover=1` twins; `-` none; other runs named). Nothing below is transcribed;
only the expression a ruling turns on is quoted.

## Part 1

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 0067d380 | Ammo::inc_time | open | L | twin | `Sim::process_projectiles` (`crates/sim/src/combat/fight.rs:6092`), with its arms `guided_closes` (:6251, the `+0x68` spline arm) and `crash_comes_down` (:6321, `+0x50 == 1`). Compared: `cur_time += 1` first; a dead shooter's `+0x32` bump (port `Sim::hold_frames_tick`, and in-walk for a crash); `flags & 2` clear → return until `cur_time < 200`, then `close` (port `p.spent`, `SPENT_LIFE`); `cur_time < total_time` returns; flag 4 without 8 tries `hit_target` then `check_hit(GROUND)`, else sets 8 (port `rolling`/`missed`); `(uint)(total_time*3) < cur_time` → `close` (port identical, unsigned); off-world (`is_valid`) → `close`; `find_data_z < z` → return, else `pos = point, total_time = cur_time` (port `z.gt(ground)`, `Single` arithmetic); then `restrict`, and `do_damage` unless `flags & 0x10` (port `!p.harmless`). Agree. The census misses it because the crate's fn is not `inc_time`; diff-backed (`docs/COMBAT.md` §9.2, §42.2, §46). |
| 00679200 | AmmoOut::set_shooter_vis | layout | - | ui | Stores the shooter's vtable `+0x48` (`is_seen`) answer for `console->who` into the render struct's `shooter_vis`; the only reader in the export is `AmmoOut::draw@0067ac10`. Renderer state. Census: `is_ui_body`'s `cls.endswith("Out")` already catches it; it shows as `layout` only because a sweep backs it, and `territory_kind` tests backing before the excluded marks. |
| 0067ab50 | AmmoData::walk_data | layout | L | saveload | `DataWalk` over the ammo's bytes from `flags` to `rolling` (to `ammo_path` when `flags & 3`), a presence byte for the spline, and `SplineData::walk_data` on it (popping or pushing a `Recycler<Spline>` on input). Serialisation and the sync checksum; produces no state on a playing frame. Census: the proposed `walk` class, short name `^walk_(rules_)?data$` (siblings `game-1`, `object-*` propose the same). |
| 0067d370 | Ammo::process | layout | - | dead | Empty body (`return;`), a vtable override. Nothing to port. Census: no existing mark; proposed **`empty`** — a function whose decompiled body is a bare `return;` (or `return 0;`) under 100 bytes; catches this. |
| 005d72c0 | Animal::process | layout | run53,run54 | twin | `anim::animal_process_steps` (`crates/sim/src/units/anim.rs:339`), called from the gaia arm of the unit loop (`crates/sim/src/lib.rs:7188`). Compared: vslot `+0x188` (`Animal::work`) first, then `Guy::process` on figures `0 .. (char)+0xb5` and `type+0x304 .. +0xe8`, the counts read after the think; the port runs `work`, then `guy_walk_counts` → `process_walk(guy_mark, squad_size, num_guys)` → the body step. Agree (item 2341). |
| 005d7330 | Animal::work | layout | L | twin-differs | `Sim::work` (`crates/sim/src/orders/mod.rs:3430`), which passes gaia over the recharge gate (`work_gate.rs:134`), then one `do_job` on the front order — agrees with the original's `Unit::do_job(order_type, order)` off the `+0xdc` cursor. **Differs**: the original's tail, on `(id + frame) & 0xf == 0` for a herd animal (`+0x86 != -1`) whose herd has a good (`HerdData +0x14 != -1`), sets `GoodData::ever_seen |= 1 << p` for each player `p` with `WorldData::is_seen` at the animal's fog cell. The crate keeps no per-good `ever_seen` (PARKED 3387e, 3220). Its one sim reader in this group is `AnimalData::is_seen`'s first herd arm (next rows). No dump prints `ever_seen`; the difference would show as a hunter's or an attacker's target pick on a herd animal standing outside the viewer's really-seen fog (UNITDATA `whom`/`ox` of a citizen on a hunt order, `UnitsSync`). |
| 005d7d50 | Animal::update_los | layout | L | absorbed | Writes the byte `+0x3c` (`los`) to 0 and returns 0 (CENSUS.md:859): an animal sees nothing. The crate's `Sim::update_los` (`crates/sim/src/vision/mod.rs:251`) has no gaia arm and writes `unit_los` (the type's LOS) into an animal's `mylos`; nothing reveals fog for owner 9 (the planes are per player), so the effect is inert. The dumped `ObjectData::mylos` on a gaia record is where a nonzero type LOS would show (`build.rs:2993`'s whole-run `mylos` diff); see Not established. |
| 005d7ea0 | AnimalData::walk_data | layout | L | saveload | `Unit::walk_data`, then (behind the `must_walk` presence byte) the animal's own `ox`, `whom`, `aid`. Serialisation and sync checksum. Census: proposed `walk` class. |
| 005d8120 | AnimalData::valid_target_const | layout | run1016,run1019,run1021,run1062,run1176,run1179 | absorbed | Two lines: the target slot's `flags & 1` (alive), no leader or diplomacy test — an animal attacker may target anything (`docs/ANIM.md` §1673). No twin: the crate's animal behaviour is `Sim::animal_idle` / `think_bird` / `think_farm_animal`, none of which acquires a target, so the attacker-side dispatch is never reached; ANIM.md records the asymmetry as unreachable. Entered by six runs because `Object::valid_target` dispatches through the attacker's vtable on every target scan touching an animal's slot as attacker; no lockstep field parted. |
| 005d8160 | AnimalData::is_seen | layout | - | twin-differs | `Sim::animal_is_seen` (`crates/sim/src/combat/fight.rs:1411`). Agree: `reveal_map == 3` → 1 (port's `has_fog`); herdless → `was_seen`; tribe bonus 9 or 4 → `was_seen`; else `was_really_seen`, all at `div_3_table[pos >> 7]`. **Differs in three arms**: (1) herdless with `ox >= 0` and `param_1 >= 0` delegates to the object at `(whom, ox)`'s vslot `+0x3c` then `+0x48` (`is_seen`), which the port never takes; (2) the herd's good `ever_seen & (1 << who)` → 1 (SEAM; not carried, PARKED 3220); (3) the tribe-bonus arm is skipped when the good's kind is 6 or `0x1f`, which the port takes as not exempt. Not entered by any lockstep run (RUNS.md:23000: not entered without `--poke e386e0=08`). Would show on UNITDATA `whom`/`ox` of a unit targeting a herd animal outside the viewer's really-seen fog. |
| 006f3700 | Armies::walk_data | open | run1016,run1497,run53,run54 | saveload | 7.5 KB, read by its callees: `Array::walk_data_ptr` over each leader's `PtrArray<Army>`, `Army::Army` on input and each army's `walk_data`, `Error::report` on a count mismatch. The container template every `X::walk_data` of a `PtrArray` stem instantiates (the same shape as `Caravans`, `Cities`, `Docks`, `Forts` below). No state on a playing frame. Census: proposed `walk` class. |
| 006f2f90 | Armies::leader_defeated | layout | L | twin | `Sim::armies_leader_defeated` (`crates/sim/src/army.rs:2563`), called from `city.rs:610` and `resign.rs:48`. Compared: every slot with the army's leading short `!= 0` (valid) gets `Army::stop`; the port halts the army's groups (`group_action_halt(g, 0)`, ARMY.md §14) and leaves the slot valid. Agree. |
| 006f30f0 | Armies::diplo_change | layout | - | twin | `Sim::armies_diplo_change` (`crates/sim/src/army.rs:2548`), from `lib.rs:6494`. Compared: the leader gate (`flags & 1`, `(flags & 0xc) != 4`, `leader_flags2 & 0xa == 0`) then `Army::process(army, 1)` on every valid army; the port skips a human or defeated leader and runs `army_process(who, s, true)`. The `leader_flags2` bits 2 and 8 are scenario switches no skirmish sets (ARMY.md:486, AI.md:41); a defeated leader's `Army::process` returns at its own defeat test after `human_frame -= 1`, which the port's early return skips — not dumped. Agree on every skirmish path. |
| 006f3160 | Armies::find_city | layout | - | dead | The first of 16 slots with a valid army whose `+0x20 == city` and `status & 1` (mustering), else −1. No caller in the export (`grep -rl` finds only itself; ARMY.md §16.1 item 5: "No caller in the game"). Census: no mark catches an uncalled function; proposed **`uncalled`** — a function with no caller in the export and no vtable slot (`vtables.txt`); catches this. |
| 006f3250 | Armies::emergency | layout | - | twin | `Sim::armies_emergency` (`crates/sim/src/army.rs:2533`), from `fight.rs:2950`. Compared: the same leader gate as `diplo_change`; per valid army the target `+0x30`/`+0x32` and `+0x34` set −1, then `Army::process(1)`; the port sets `target = None` and runs `army_process(.., true)`. Agree (ARMY.md §16.1 item 8 settles the caller as the city alarm's). |
| 006f3b00 | Armies::process_all | layout | L | twin | `Sim::armies_process_all` (`crates/sim/src/army.rs:1374`). Compared: per leader with `flags & 1`, `(flags & 0xc) != 4`, `leader_flags2 & 0xa == 0`; per valid army `hurry = status & 0x80`, the bit cleared, `Army::process(hurry)`; the port's `status::HURRY` consumed into `bypass` identically. The port skips only a human leader (the `0xa` bits are unset in a skirmish, as above). Agree; ARMY.md §5 is diff-backed. |
| 0073e3f0 | Caravans::walk_data | open | run1016,run1497,run53,run54 | saveload | 8 KB: the `PtrArray<Caravan>` container walk per leader (`Caravan::Caravan` on input, each `walk_data`). No state. Census: proposed `walk` class. |
| 0073e0c0 | Caravans::new_danger | layout | - | unmodelled | Called only by `Object::take_damage@00652020` on the killing blow of a land caravan on a linked route (CENSUS.md:532, CARAVAN.md §10). For every leader with `flags & 3 == 3` that is the caravan's owner or mutually allied (`+0x74[who] == 2` both ways), every live caravan (`flags & 3 == 3`): if it has no `+0x20` and a road (`+0x18 > 0`), restart its route when any road point lies within `0x600` (`vector_dist < 0x601`) of the blow; else restart unconditionally — `Caravan::restart_trade_route`. The crate's `Unit::close` caravan arm only frees the slot (CARAVAN.md:781; PARKED 993). **Not lockstep-entered**; run353 enters it on 1007 (not a lockstep game). Would show in UNITDATA for each surviving caravan of that owner on the frames after the kill (its order and destination, `UnitsSync`), on run353's frame 1007. |
| 00739ab0 | Caster::walk_data | layout | - | saveload | `Array<ActiveSpell>::walk_data` on the caster's active-spell array. Census: proposed `walk` class. |

## Part 2

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 006f1e30 | Categories::find_data | layout | - | ctw | Linear search of a `Categories` table for the row whose data word (`+0x3c`) equals the argument; `Error` on a miss. Sole caller `ConquestStyle::set_game_rules@007a9440` (Conquer the World). Census: proposed class `^Conquest` callers-only is not a name mark; propose the function by name in a **Conquer the World** replaced/cut group, or `uncalled`-in-skirmish. |
| 006f20e0 | Categories::find_key | layout | L | replaced | The XML loaders (proposed name extension `Categories::find_key`). Linear search of a `Categories` table by `String` key, `Error::key` on a miss. Callers: `UnitType::init@0061ab50:321` and `BuildType::init@00632340` resolving a type's `CAT` text against `rules.xml`'s `unit_cats`, plus `PreSetupWin` (UI) and four `Conquest*` (CtW). Twin in the loader layer: `rondata::load` resolves `CAT` by key against `<CATEGORIES id="unit_cats">` (`crates/rondata/src/load.rs:566`). Entered by the twins at load only. |
| 008c4350 | CircleRender::scale | name | - | ui | 8.5 KB, read by its first screen and callers: rebuilds the selection circle's render vertices and colours, sampling `TerrainOut::find_data_z` for the ground under each vertex. Callers are all draw paths (`UnitOut::draw_underlays`, `ItemOut::draw_underlays`, `ObjectsOut::draw_overlays`, `Scene::draw_overlays`, `MessageWin::on_redraw`). The `name` kind is a collision: `crates/sim` defines an unrelated `fn scale`. Census: proposed class mark `cls.endswith("Render")` (catches `CircleRender::scale`, `::get_name`, `::on_field_change`). |
| 00447880 | CircleRender::get_name | layout | - | ui | Returns the literal `"Circle"` — the property system's class name for the renderer. Census: proposed `Render$` class (above); or a short-name mark `^get_name$` for the reflection interface (also catches `FogMaster::get_name`). |
| 008c51a0 | draw_thick_diamond | layout | - | ui | A line-drawing helper of the renderer. `is_ui_body`'s `^draw` already catches it; it shows as `layout` only because a sweep backs it (backing is tested before the excluded marks). |
| 008c6db0 | CircleRender::on_field_change | layout | - | ui | `init_circle(this); return 0;` — the property system's change hook rebuilding the circle mesh. Census: proposed `Render$` class, or a short-name mark `^on_field_change$`. |
| 00735410 | Cities::walk_data | open | - | saveload | 8 KB: the `PtrArray<City>` container walk per leader (length check with `"Array::walk_data_ptr: write len exceeds size"`, `City::City` on input, each `walk_data`, the city name's `String::walk_data`). No state. Census: proposed `walk` class. |
| 00681e30 | CollBlock::get | layout | L | twin | `CollGrid::get` (`crates/sim/src/movement/collide.rs:582`). The original reads bit `(x mod 16)·16 + (y mod 16)` of the world cell's `CollBlock` bitmask (`+0xc`); the port reads one bit per unit cell of a flat grid. Same predicate (is the unit cell's collision bit set); the block indexing is storage. Agree. |
| 00682070 | CollBlock::set | layout | L | twin | `CollGrid::set` (`crates/sim/src/movement/collide.rs:587`). Set or clear the same bit; the original also keeps the block's empty cache `+0x8` (0 after a set, 2 "unknown" after a clear when it was 0), which `BitMask<768>::empty` recomputes on 2 (COLLISION.md §4.2). The port has no cache: `CollGrid::any_in_cell` (:574) recomputes, so the cache is absorbed and exact either way. Agree. |
| 006820e0 | CollCheck::fill_slots | open | L | twin-differs | `Sim::probe_slots` (`crates/sim/src/movement/collide.rs:2054`) with `ProbeSlots`. Agree: the 2×2 world cells from `((ucx − r) >> 4, (ucy − r) >> 4)`, the first always and the others when the box crosses, none off the map; a slot kept only when the world cell's region equals `get_tregion` of the probe centre's tile (`region2` on a coastal water half), every slot when the centre has none; for `nocoll`, the pathfinder's `+0x4c` copy tree read first and a copy inserted on a miss (PATHFINDER.md §26, COLLISION.md §4.2). **Differs** (the port's own SEAM): a copy outlives the probe that took it, so a later `nocoll` probe from another region reads it ungated in the original; the crate gates every probe on its own centre. Inert on one-region maps (Great Lakes); would show as a path or step disagreement (UNITDATA `x`/`y`, `UnitsSync`) on a multi-region map where a `nocoll` search crosses a region boundary between probes. |
| 00682ad0 | CollCheck::move_unit | open | L | twin | `Sim::coll_move` (`crates/sim/src/movement/collide.rs:1679`), from `set_new_location`. Compared: clear the disc around `from` cells further than `size` from `to`, set those around `to` further than `size` from `from`, each pass gated by its own end's tile region; and the one-block head (both discs in one world cell, `|Δ| + 2·size < 16` on both axes) looked up once by the **old** tile's region, both passes ungated, a mismatched region ending the call. Agree (COLLISION.md §2.4, §16, diff-backed). |
| 004613b0 | WCoord::operator_TCoord | layout | L | twin | `Cell::centre_tile` (`crates/sim/src/world/mod.rs:114`). Both `4·w + 2`. Agree. The census's `operator` mark would exclude it; it shows as `layout` because a sweep backs it. |
| 00461450 | WCoord::WCoord | layout | L | dead | The constructor: `value = param_1`. `is_dead_weight`'s constructor rule catches it; `layout` only because a sweep backs it. |
| 0046f080 | WCoord::get_tcorner | layout | L | absorbed | `w << 2`, the cell's corner tile. The crate writes it inline as `c * TILES_PER_CELL` (`crates/sim/src/world/mod.rs:60`, e.g. `world.rs:702`, `ai_place.rs:304`). No twin needed. |
| 00472b30 | TCoord::operator< | layout | L | absorbed | `this->value < param_1->value`, a signed integer compare on the tile coordinate; the crate compares `i32`s directly. The census's `operator` mark already excludes it bar the sweep backing. |
| 00681b20 | WCoordData::walk_data | layout | - | saveload | One `DataWalk` call over the 4-byte value. Census: proposed `walk` class. |
| 00681c50 | TCoordData::walk_data | layout | - | saveload | As `WCoordData::walk_data` (same 187-byte body shape), over the tile coordinate. Census: proposed `walk` class. |
| 008d5240 | DeathObj::inc_time | open | run1018–1022,run1176 | twin | `Sim::death_objs_inc_time` (`crates/sim/src/combat/fight.rs:4171`) and `Sim::cull_deaths` (:4200), with the life in `combat::Death::life` / `Figures` (`crates/sim/src/combat/mod.rs:579`). Compared: first statement `hold_frames += 1` on the record's object slot, dead or not (port: one bump per live figure record, the slot's latest occupant); counter `+0x40` +1 a call, the record zeroed when it reaches `get_game_frames(piece, cur_anim)` plus `corpse_fade_end` (a corpse, `+0x44 != -1`), else plus `body_fade_end` unless the type's domain is 1 (sea, no fade) — the port's precomputed `life`, cull at `frame − first_frame + 1 >= life`; on the end, `clear_blocking` when vslot `+0x120` (`blocks_while_dead`) answers (port `coll_remove` under `BLOCKS_WHILE_DEAD`). Agree (GOLDEN §127, §149.10, §156.1; COMBAT §59.3). Two arms the port takes differently, both inert on the captures: while a `blocks_while_dead` record lives the original floors the hold at 30, which the port omits — the bump and `process_all`'s decrement already hold it at ≥ 30 from `Object::close`'s `0x1e`; and a record whose data piece is null is zeroed at once, where the port's `life: None` (a piece the install does not describe) is never culled. `DeathObjOut::inc_bleed` is the renderer's. |
| 008d4a90 | DeathObjData::blocking | layout | - | twin | Answers the dead object's type vslot `+0x120` (`blocks_while_dead`) `!= 0`. No direct caller in the export (inlined at its uses); the crate's predicate is `unit_flags & BLOCKS_WHILE_DEAD != 0` (`crates/sim/src/combat/fight.rs:4219`, `:3567` `keeps_disc`, `:3967`). Same predicate. Agree. |
| 008d5520 | DeathObjData::walk_data | layout | - | saveload | `DataWalk` over the record to `first_frame`, and the rest when `valid`. Census: proposed `walk` class. |
| 00740c40 | Dock::walk_data | layout | - | saveload | `DataWalk` over the dock record (skipped on a checksum walk; the slot short reset to −1 on input). Census: proposed `walk` class. |
| 00740c90 | Docks::remask_docks | open | - | twin-differs | Inline in `BuildType::mask_me`'s dock arm (`crates/sim/src/cities/city.rs:2059`), laying `Sim::mask_dock_water` (:2115) again. Agree: on a dock's unmask, every other dock lays its apron — tiles `−3 .. size + 2` about the parity-corrected corner, in bounds and ocean (`mask & 0x30 == 0x20`), `bad += 1` where the tile lacked `0x2000`, then the bit set (port `World::set_bad_path` → `set_bad_path_bit` keeps `bad`, `world.rs:1873`) (TRANSPORT.md §5.4, ORDERS.md §25.2). **Differs in the candidate set**: the original walks every active leader's `Docks` registry (`flags & 1` on the leader, `+8 & 1` on the dock record), excluding the `(who, slot)` pair passed; the port walks every alive, `started` dock building except `b`. The registry entry is made by `Docks::init_dock` when a dock **activates** (`crates/sim/src/transport.rs:390`), so an unfinished started dock is relaid by the port and not by the original (ORDERS.md:6416 records the set as unestablished). Not entered by any lockstep run. Shows in the pathfinder's tile `0x2000` and `WData.bad` (no dump prints them) and so in a ship's path (UNITDATA `x`/`y`) beside an unfinished dock whose apron overlaps a dying dock's. |
| 00741100 | Docks::walk_data | open | run1016,run1497,run53,run54 | saveload | 7.6 KB: the `PtrArray<Dock>` container walk per leader. Census: proposed `walk` class. |
| 0047be70 | FogMaster::get_name | layout | - | ui | Returns the literal `"Fog Master"` — the property system's name for the fog renderer's master object. Census: proposed short-name mark `^get_name$` (with `CircleRender::get_name`). |
| 0073ebc0 | Fort::walk_data | layout | - | saveload | One `DataWalk` call over the fort record. Census: proposed `walk` class. |
| 0073ef30 | Forts::walk_data | open | run1016,run1497,run53,run54 | saveload | 7.3 KB: the `PtrArray<Fort>` container walk per leader. Census: proposed `walk` class. |
| 0073ebe0 | Forts::find_fort | layout | L | twin | `Sim::fort_at` (`crates/sim/src/ai/ai_sites.rs:308`), for `Leader::compute_site_stats@006cd040`, its only caller. Compared: walk `forts.lists[who]` (count `LeaderData +0x428`) for an entry with `+6 & 1` whose building's world cell `div_3_table[pos >> 8]` equals `(wx, wy)`; the port: an alive, `active` fort building of `who` whose `pos.cell()` is the cell (`active` stands for the list membership, `Fort::init` being reached only from `Build::activate`; AI.md §358.3). The original returns the index, the caller tests `>= 0`; the port answers the bool. Agree. |

## Part 3

The `gameaccess` stem: `GameAccessConst::get_scary_*` are one-line accessors that hand out a
mutable pointer to a singleton from const code (`return &cliffs;`). Their callers were counted
two ways: `grep -rl` over the export, and a scan of the shipped executable's `.text` for every
`E8`/`E9` rel32 whose target is the function, plus a search of the whole file for the address
as an absolute pointer (a function table, a VM registration). Eleven of the fourteen have **no
caller and no pointer anywhere in the file**: nothing reaches them, the scripting VM included,
so they are `dead`, not `replaced`. Proposed census mark for the stem:
`^GameAccessConst::get_scary_` (class-and-prefix; catches all fourteen here plus
`get_scary_console_who`, `get_scary_graphic_pieces`, `get_scary_message_win`), or the general
**`uncalled`** test proposed under `Armies::find_city`.

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 00583370 | GameAccessConst::get_scary_cliffs | layout | L | absorbed | `return &cliffs;`. Five call sites, all in `BuildTypeData::calc_gather@00639e40` (`0x63a6a8`, `0x63aaf5`, `0x63ab1f`, `0x63ab49`, `0x63acf7`), before `CliffsData::find_nearest`. The crate owns its cliffs on `Sim` and needs no accessor; the call site is named in `crates/sim/src/economy/calc_gather_type_view.rs:181` (whose cliff arm is itself a seam, `gather.rs:460` — that is `find_nearest`'s ruling, not this one's). |
| 00583380 | GameAccessConst::get_scary_scenario_editor | layout | - | dead | Returns the scenario editor singleton. No call site in `.text`, no pointer in the file. Census: `^GameAccessConst::get_scary_` or `uncalled`. |
| 00583390 | GameAccessConst::get_scary_iface_main | layout | - | dead | Returns the main interface window. No caller, no pointer. Same mark. |
| 005833a0 | GameAccessConst::get_scary_camera | layout | - | dead | Returns the camera. No caller, no pointer. Same mark. |
| 005833b0 | GameAccessConst::get_scary_fog | layout | - | dead | Returns the fog renderer singleton. No caller, no pointer. Same mark. |
| 005833c0 | GameAccessConst::get_scary_console | layout | - | dead | Returns `MiscAccess::console`. No caller, no pointer. Same mark. |
| 005833d0 | GameAccessConst::get_scary_iface_current | layout | - | dead | Returns the current interface window. No caller, no pointer. Same mark. |
| 005833e0 | GameAccessConst::get_scary_console_leader | layout | L | ui | `leaders.list + console->who`, the local viewer's leader. Ten call sites: `Leader::gain_tech@006dcb60` (the age/tech announcement's treaty and ally tests choosing the message and `S_AGE_ADVANCE_OPPONENT` sound), `Leader::set_diplo@006ec6a0:53` (the treaty message), and `Object::do_damage@0064a480:1394–1429` (`0x64c6b2`–`0x64c752`: the text bubble on a city's capture, worded by whether the console's leader is the attacker or its ally and sees the object). Every use selects a message or sound relative to the console viewer; no simulation state is written from it. Census: `^GameAccessConst::get_scary_`; the callers' arms are the UI's. |
| 00583400 | GameAccessConst::get_scary_terrain | layout | - | dead | Returns the terrain renderer. No caller, no pointer. Same mark. |
| 00583410 | GameAccessConst::get_scary_options | layout | - | dead | Returns `MiscAccess::options`. No caller, no pointer. Same mark. |
| 00583420 | GameAccessConst::get_scary_doobers | layout | - | dead | Returns the doober (floating-text) list. No caller, no pointer. Same mark. |
| 00583430 | GameAccessConst::get_scary_my_player | layout | run1017,run53,run54 | ui | `game.info.player + console->play`, the local viewer's lobby record. Three call sites: `Leader::defeat@006ecb00:178` (for the console's own leader, its `flags & 0x40` picks the post-game screen) and `Leader::gain_tech` (`0x6de44d`, `0x6de6da`; the announcement). Console-relative UI. Same mark. |
| 00583450 | GameAccessConst::get_scary_select_group | layout | - | dead | Returns the console player's selection group (`play × 0x9f0` into `select_groups`). No caller, no pointer. Same mark. |
| 00583470 | GameAccessConst::get_scary_scene | layout | - | dead | Returns the scene. No caller, no pointer. Same mark. |

## Summary

**60 rulings**: twin 13 · twin-differs 4 · absorbed 5 · unmodelled 1 · saveload 13 · dead 14 ·
ui 8 · ctw 1 · replaced 1.

- **twin (13)**: `Ammo::inc_time`, `Animal::process`, `Armies::leader_defeated`,
  `::diplo_change`, `::emergency`, `::process_all`, `CollBlock::get`, `::set`,
  `CollCheck::move_unit`, `WCoord::operator_TCoord`, `DeathObj::inc_time`,
  `DeathObjData::blocking`, `Forts::find_fort`. Each is `open`/`layout` only because the
  crate's fn does not carry the original's snake-case name (`process_projectiles`,
  `animal_process_steps`, `armies_*`, `CollGrid::get/set`, `coll_move`, `centre_tile`,
  `death_objs_inc_time`, `fort_at`): a doc-comment spelling of `Class::method` on each would
  move all thirteen to `port` under `is_port_backed`.
- **twin-differs (4)**: `Animal::work` (the goods `ever_seen` write), `AnimalData::is_seen`
  (three arms), `CollCheck::fill_slots` (the copy tree's lifetime, a SEAM), `Docks::remask_docks`
  (the candidate set).
- **absorbed (5)**: `Animal::update_los`, `AnimalData::valid_target_const`,
  `WCoord::get_tcorner`, `TCoord::operator<`, `GameAccessConst::get_scary_cliffs`.

### Proposed census classes

- **`walk`** — short name `^walk_(rules_)?data$` outside `orders` (the same pattern
  `game-1` and `object-*` propose). Catches here: `AmmoData::walk_data`,
  `AnimalData::walk_data`, `Armies::walk_data`, `Caravans::walk_data`, `Caster::walk_data`,
  `Cities::walk_data`, `WCoordData::walk_data`, `TCoordData::walk_data`,
  `DeathObjData::walk_data`, `Dock::walk_data`, `Docks::walk_data`, `Fort::walk_data`,
  `Forts::walk_data` (13; five of them, the `PtrArray` container walks, are 7–8 KB `open`
  rows — 38 KB of the group's open bytes).
- **`uncalled`** — no `E8`/`E9` rel32 call site in `.text`, no absolute pointer in the file,
  no `vtables.txt` slot. Catches here: `Armies::find_city` and the eleven dead
  `GameAccessConst::get_scary_*`. A cheap scan (this report did it in one Python pass over
  the exe) and it needs no name.
- **`^GameAccessConst::get_scary_`** — the accessor class whole (17 functions in the
  export; 14 here). The two that are called (`console_leader`, `my_player`) are
  console-relative UI and `cliffs` a pointer return, so the class mark loses nothing.
- **`Render$` on the class** — `CircleRender::scale`, `::get_name`, `::on_field_change`
  (the selection-circle mesh; `is_ui_body` catches `*Out` but not `*Render`).
- **`^(get_name|on_field_change)$` on the short name** — the property system's reflection
  hooks: `CircleRender::get_name`, `FogMaster::get_name`, `CircleRender::on_field_change`.
- **`empty`** — a decompiled body that is a bare `return;`: `Ammo::process`.
- **Ordering, not a class**: `AmmoOut::set_shooter_vis`, `draw_thick_diamond`,
  `WCoord::WCoord`, `WCoord::operator_TCoord` and `TCoord::operator<` already match
  `is_ui_body`, the constructor rule or `operator`, and are listed `layout` only because
  `territory_kind` tests sweep backing first. Testing the excluded marks before backing
  (or letting a mark override a layout-only backing) clears five rows with no new pattern.

### Proposed replaced groups

- **The XML loaders**, extended by name with `Categories::find_key` (resolves a type's
  `CAT` text at `UnitType::init`/`BuildType::init`; `rondata::load` resolves the same key,
  `crates/rondata/src/load.rs:566`).
- **Conquer the World** (cut from v1 by CLAUDE.md): `Categories::find_data`, whose only
  caller is `ConquestStyle::set_game_rules`.

### The unmodelled, ranked

No row in this group is both **entered by a lockstep run and wholly unmodelled**. The
nearest thing is an unmodelled *arm* of an entered function:

1. **`Animal::work`'s `ever_seen` write** (entered by run1016–1021) — every 16 frames a herd
   animal marks its herd's good seen by each player whose fog sees it. Inert on every
   capture because its one simulation reader, `AnimalData::is_seen`'s herd arm, was never
   entered by a lockstep run (RUNS.md:23000). Field: none dumped (`GoodData::ever_seen`);
   it would show on UNITDATA `whom`/`ox` of a unit choosing a herd animal outside the
   viewer's really-seen fog (`UnitsSync`).
2. **`AnimalData::is_seen`'s three arms** (not entered) — the carried-animal delegation,
   the good's `ever_seen`, the fish/`0x1f` exemption. Same field as 1.
3. **`Caravans::new_danger`** (not entered by lockstep; run353 enters it on 1007) — on a
   caravan's kill, every caravan of the owner and its mutual allies whose road passes
   within `0x600` (or has no road) restarts its route. Field: UNITDATA of the surviving
   caravans (order, destination) on the frames after the kill; run353 frame 1007.
4. **`CollCheck::fill_slots`'s copy lifetime** (entered) — inert on one-region maps; UNITDATA
   `x`/`y` on a multi-region map where a `nocoll` search crosses regions.
5. **`Docks::remask_docks`'s candidate set** (not entered) — an unfinished started dock
   relaid by the port; tile `0x2000`/`WData.bad` are not dumped, so a ship's UNITDATA
   `x`/`y` beside overlapping docks.

## Not established

- **`Animal::work`'s leader test.** The decompile indexes `leaders.list` by the animal's
  own owner byte (`+0x9`) inside the per-player loop, where the player `p` would be
  expected; the listing would settle which. It does not change the ruling.
- **`Animal::update_los`.** Whether the crate gives an animal a non-zero `mylos` (its
  `Sim::update_los` has no gaia arm, and the animal types' LOS was not traced to their
  rows; they are not in `unitrules.xml`). If non-zero, the gaia records' dumped `mylos`
  would part unless the whole-run `mylos` diff (`crates/rondata/src/diff/build.rs:2993`)
  leaves gaia out — not checked.
- **`Docks::remask_docks`.** That `Docks::init_dock` runs on activation rests on
  `crates/sim/src/transport.rs:390`'s doc comment, not on a reading of its callers here.
- **`DeathObj::inc_time`'s null-piece arm** against the port's `life: None`: whether any
  shipped death piece is null in the original (so the arms ever meet) was not checked.
- **`AmmoOut::set_shooter_vis`** has no caller in the export; its call is presumably
  inlined into `Ammo::init`/`Object::fire_ammo` — not checked, and immaterial to a `ui` ruling.
- **The eleven dead accessors**: the scan covered direct `E8`/`E9` rel32 and absolute
  32-bit pointers; an indirect call through a computed address (none is plausible for a
  `__cdecl` accessor) would escape it.
