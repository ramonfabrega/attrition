# Territory reading — group `borders-2` (2026-11-06)

A territory reader's rulings on the 19 `layout` functions of the `wall` stem
(`/Users/rf-studio/.claude/jobs/d388900a/tmp/t71/groups/group-borders-2.tsv`), one row per
function, read by name from the decompile export. Reader: Opus 5.5, the seventy-first tranche's
reading fan-out. `entered` is the TSV's `entered_by_lockstep` column (`y` = some lockstep run
entered it, `-` = none). Nothing here is transcribed; only the expression a ruling turns on is quoted.
Consistent with the sibling report `docs/audit/2026-11-06-territory-borders-1.md`.

**Which slot a live building dispatches.** `Wall` is the base of `Build`; every live building
carries `Build::vftable@00b42174`. On it `WallData::{is_trade, is_sea_trade, is_active,
is_started, armor, los, support_cost, get_garrison_limit, is_neutralized, in_unfriendly_territory}`,
`WallOut::get_gpiece` and `Wall::{die, update_los, inc_time}` are live; `WallData::{is_seen +0x48,
walk_data +0x7c, hits +0x11c, is_now_seen +0x17c, construct_time +0x18c}` and `WallOut::make_options
+0x54` are **overridden** by `BuildData::*` / `BuildOut::make_options`, so they run only when the
override calls them by name (read per row). Slot `+0x20` on `Build::vftable` is
`Buffer::is_pending_load@0041e0e0`, `return 1`.

## Part 1 — the `wall` stem's layout rows, first half (10 rows)

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 0063e2a0 | WallData::valid_buildmask | layout | - | twin | `crates/sim/src/air.rs:1297`, inside `Sim::action_buildmask` (`air.rs:1291`): `admitted = (queue && can_infinite(b)) \|\| (repeat && is_hangar(t))`. Original: `0x40` arm needs vslot `+0x20` (on `Build::vftable` a constant 1) then `BuildData::can_infinite`; `0x80` arm `ObjectData::can_carry(AIR)`; the crate's `can_infinite` (`production.rs:731`) and `is_hangar` (`garrison.rs:812`, `can_carry(AIR)`) — agree. Emulator sweep `crates/sim/tests/sweep/wall_data_valid_buildmask.rs` holds `eax` and every stub call to the port. Its only caller is `Group::action_buildmask` (the repeat / infinite-queue toggle, `docs/GOLDEN.md` §32–§33, diff-backed on run281/run285); never entered by a lockstep run because no traced lockstep game toggled it. |
| 0063ed30 | WallData::is_neutralized | layout | y | twin | `return 0`. `crates/sim/src/orders.rs:13766` `GatherSpotSim::neutralized` (the `GatherSpotView` impl) answers `false` (the data vslot `+0x180`, read by `find_gather_spot` at `orders.rs:13659`) — agrees. Live on `Build::vftable +0x180`. |
| 0063ed40 | WallData::get_garrison_limit | layout | - | twin | Forwards `BuildTypeData::get_garrison_limit(type +0x18, who +0x9)`; live on `Build::vftable +0x178`, called by name from `BuildOut::make_options` (UI). Twin `Sim::garrison_limit` (`crates/sim/src/units/garrison.rs:65`): the type's `garrison_max` plus `(level−1)·{TOWER,FORT}_GARRISON_UPGRADE` at the owner's level (`docs/CITIES.md` §6.3) — the same forward of `(type, owner)`; agrees. Not entered by a lockstep run: most simulation readers (`Unit::do_garrison`, `find_garrison_build`, `Group::action_garrison`, `Build::train`) call `BuildTypeData::get_garrison_limit` directly; the slot's simulation dispatcher is `Unit::come_out@00617c10` (lines ~871/886), whose arm no traced game reached. The crate routes every reader (`alarm.rs:114`, `rally.rs:666`, `lib.rs:4954`) through the one function. |
| 0063f6c0 | Wall::die | layout | y | twin | Pure forward `Object::die(this, p0, p1, p2)` (`docs/journal/2026-10-13-item-1911.md`, emulator: `ret 0xc`, agree). Twin `Sim::die_building` (`crates/sim/src/cities/city.rs:2567`): `close_building(b, false)` then `hold_frames = max(hold_frames, 1)` — the `Object::die` tail on a building (`fight.rs:4073`'s `Object::die@00647080` reading); agrees for the forward. Lockstep-entered by every building death. |
| 0063fa30 | WallData::support_cost | layout | - | dead | `*(type +0x270 + i·4) << 8`. Slot `+0x140` is never dispatched on an `Object`-derived receiver (`docs/AI.md` §383.1: the six `call [reg+0x140]` sites go to `UnitType`/graphics tables) and no function calls it by name (export grep). The field stays live through `BuildTypeData::get_kill_value`, ported as `cost.rs` `kill_value_by` (`cost.rs:1253`). Census: already a §383.1 `RESIDUE` candidate; proposed mark **`SLOT_0x140`** — the three bodies `WallData::support_cost@0063fa30`, `ObjectData::support_cost@006452f0`, `UnitData::support_cost@0060fde0`, short name `^support_cost$` on an `*Data` class. |
| 0063eeb0 | Wall::update_los | layout | y | twin-differs | Live on `Build::vftable +0x160`; writes the byte `+0x3c`. Twins: the wired `Sim::build_los` (`crates/sim/src/vision.rs:444`) and the whole-function sweep port `wall_update_los` (`crates/sim/src/vision/wall_los_view.rs:60`, held to the emulated original by `sweep/wall_update_los.rs`). Agree on the gates (`is_started`; unfinished and not a wonder → 0; a wonder site → 1), the base `LOS + epoch[Science]·SCIENCE_LOS`, the fort term `fort_upgrade_range[get_fort_los+4]` and the tower term `tower_fort_range[get_tower_fort_los+3]`, and the tail `+ x_size/2` (`docs/VISION.md` §2.1, run39/run1186/run1277). **`build_los` differs, inertly, in three places**: (1) it omits the colosseum term (fort or tower, `has_wonder(0x212)` → `+COLOSSEUM_FORT_RANGE`) and the furs term (`+0x6da5`/`+0x6dcd` bit 7 → `+FURS_LOS`), parked 3631a — both constants ship as 0; (2) it omits the `leader_flags & 1` (`LEADER_VALID`) gate that zeroes the base — a defeat does not clear that bit (`docs/VISION.md` line ~699), so it is set for every building owner; (3) it sums in `i32` where the original wraps the byte at 8 bits — unreachable at shipped magnitudes. Only a mod's constants would move it. Field: `BUILDDATA/WALLDATA/OBJECT` `mylos` — printed and pinned **uncompared** (`crates/rondata/src/diff/coverage.rs:205`); comparing it is a widening, not a capture. |
| 0063fa50 | WallData::los | layout | y | absorbed | `(int)(char)+0x3c`, the byte `update_los` stored; live on `Build::vftable +0x128`, read by `Object::update_seen` and the fog walks. The crate stores no byte: `Sim::build_los` (`vision.rs:444`) recomputes it at each read, gated on `started` as the store is (a placed-unstarted site reads 0 in both, `docs/VISION.md` §2.1). The cache's lag (a value read between a change and the next `update_los` call) is the only possible difference; run1293's packet holds who=1's fog to the byte with the computed value (§2.1). Emulator sweep: `docs/CENSUS.md` item 1724 (signed char `+0x3c`). |
| 0063fa60 | WallData::armor | layout | y | twin-differs | Live on `Build::vftable +0x124` (`get_damage`'s target armour). Twin `Sim::armor_of` (`crates/sim/src/combat/fight.rs:478`), `Obj::Building` arms: `ObjectData::armor` (type armour + the Dutch term) then the halving of a building with `flags & 4` clear, a signed `/ 2` toward zero — agree (`docs/COMBAT.md` §4.2, run514's Keep site, diff-backed). **Differs on the Senator arm** (`63fab0`–`63fb1f`): when `leaders[who] +0x59c0` (a short) is non-zero, `HeroesData::find_hero(pos, …, 0x161, (type x_size + y_size)·0x60)` — a `THESENATOR` within reach — adds `thesenator_build_armor` before the halving; the shipped `rules.xml` sets it to **5**. The crate's `SEAM` (`fight.rs:474`), parked 1359. The first arm (the receiver itself `is(0x161)`) needs vslot `+0x18`, which on `Build::vftable` is `Window::get_button`, so it never fires for a building. Lockstep-entered on every building strike; the Senator arm is inert on every capture so far because no strike landed within a Senator's reach (East Indies `1/60` and run470 `1/80` have Senators, parked 1359). Field: the struck building's `hits` (`BUILDDATA`) and the `GUY` block's `damage`/`damage_frac` on a capture where a Republic-government player's Senator stands within `(x+y)·96` of a building being hit. |
| 00642470 | WallOut::make_options | layout | - | ui | Adds `OPTION_DISBAND` / `OPTION_BACK` / `OPTION_CANCEL` buttons to `MiscAccess::options` by the options mode — the selection panel. On every live building slot `+0x54` is `BuildOut::make_options@00629ed0`, and nothing calls this body by name (export grep), so it never runs. Census: `is_ui_body` already matches it (`cls.endswith("Out")`), but `census.territory` returns `layout` for a swept function (`tools/census.py:1228`) **before** it reaches the `is_ui_body` test (`:1233`), so a swept `*Out` body stays open. Proposed fix rather than a new class: test the excluding marks ahead of the sweep's `layout` — catches this row and `WallOut::get_gpiece@006424d0`. |
| 006424d0 | WallOut::get_gpiece | layout | y | ui | `return render_gpiece` (int `+0x68`, `docs/CENSUS.md` item 1724). Live on `Build::vftable +0x34`; its callers by name are `Wall::inc_time@0063fb60` (the `GraphicPieces::emit_*_particles` arm, which `borders-1` ruled render) and `GraphicPieces::emit_build_particle@00906d10`. The graphic piece handle; no simulation value depends on it. Census: `is_ui_body` (`WallOut`), masked by the `layout` ordering above. |

## Part 2 — the `wall` stem's layout rows, second half (9 rows)

| va | name | kind | entered | ruling | evidence |
|---|---|---|---|---|---|
| 006424e0 | WallData::is_trade | layout | y | absorbed | If the receiver's `+0xb8` is `ObjectData::is`, the type's slot `+0x60`, else `is(0x19e, 0)` — the city line (`docs/journal/2026-10-13-item-1840.md`, emulator: agree). Live on `Build::vftable +0x24`. Simulation callers: `Unit::do_trade@005ed270` (~line 599, the route's end re-read through its object, then the arrival box `max(x_size, y_size)·0x60 + 0x306`) and `Group::action_trade@00701cc0` (the player's trade command; `Console::find_target_daemon` is UI). The crate holds the route's ends as **city indices** and asks `cities[i].alive` (`crates/sim/src/economy/caravan.rs:422`), with the box at `caravan.rs:433` and `city_span` (`:557`) — the `SEAM` at `caravan.rs:418` names the difference: a route survives its city being replaced in the object slot there, not here. `Group::action_trade` is swept whole in `trade_view.rs` (`data_is_trade`, `obj_is_trade`) and not wired (`trade_view.rs:15`). |
| 00642510 | WallData::walk_data | layout | y | saveload | The `DataWalk` visitor's `WallData` layer: `Object::walk_data`, a section label from `int_str_array`, then `job_counter..+0x66` when `must_walk` (or slot `+0x78`) says so. Reached only through `BuildData::walk_data@0062f270` (which overrides slot `+0x7c` on every live building) by name. `DataWalk`'s subclasses are `SaveGame`/`LoadGame` and `CheckSum` (`docs/ORACLE.md` "write_header", `docs/FORMATS.md` §CommandPackage point 2): save, load and the desync checksum — no simulation value is computed. The crate carries none of the three; its determinism check is the soak's digest (`crates/sim/src/soak.rs`) and its oracle reader is `crates/rondata/src/gamelog.rs`. Emulator sweep: `docs/journal/2026-10-17-item-2900.md` (both arms of the `must_walk` compare). Proposed census mark **`DATA_WALK`**: short name `^walk(_rules)?_data$` — catches this row, `BuildData::walk_data@0062f270`, `AnimalData::walk_data@005d7ea0` and every class's sibling (`Object::walk_data`, `BuildQueue::walk_data`, `Array<…>::walk_data`). Which walk the lockstep runs entered (the checksum each `CheckDesyncsEveryXFrames`, or an autosave) is not established. |
| 00642bb0 | WallData::hits | layout | - | dead | `p ? +0x20 : construct_hits (+0x54)`. Slot `+0x11c` is overridden by `BuildData::hits` on every live building and nothing calls this body by name (export grep), so only a most-derived `Wall` reaches it, and none exists (`docs/AI.md` §383.2, `RESIDUE` candidate on `Wall::Wall`'s predicate; GOLDEN §144.4). Census: §383.2's three Wall overloads — proposed mark **`WALL_OVERLOAD`**, exact names `WallData::hits@00642bb0`, `WallData::construct_time@00642d90`, `WallData::can_make@00472cb0` (`borders-1` ruled it `dead`), and `ObjectData::max_range` through the Wall triple. |
| 00642bd0 | WallData::is_seen | layout | y | twin-differs | Reached by name from `BuildData::is_seen@0062e1a0` (slot `+0x48`, `valid_target_const`'s test 5 on a building target). Twin `Sim::build_is_seen` (`crates/sim/src/combat/fight.rs:1493`): owner → seen; then `(started \|\| ally_mask[who] & 1<<owner) && ever_seen & ally_mask[who]` — agree, diff-backed on Great Lakes' word holding at 14982 (`docs/VISION.md` §10.4–§10.5). **Differs, one-directionally** (the crate can only refuse further), on the arms its `SEAM` (`fight.rs:1490`) leaves out: `reveal_map == 3` (the crate honours it in `world_is_seen_fog`, `fight.rs:1538`, but not here), `leader_flags[who] & 0x800`, `leader[who] +0x59e4` (a short) non-zero, and the city-centre arm (`flags & 0x20`, semaphore bit 4 clear, both leaders' first byte `& 4`). Field: a unit's `order:target` (`UNITDATA`) on a building it should be able to target — a capture in a `reveal_map 3` lobby is the cheapest, since that arm needs no tech. |
| 00642ca0 | WallData::is_now_seen | layout | y | ui | Reached by name from `BuildData::is_now_seen@0062e130` (slot `+0x17c`) and `ObjectsOut::sort_all@00657320`. Every caller of slot `+0x17c` on a building passes the **console's** player: `Wall::inc_time@0063fb60:80` gates the `GraphicPieces` particle emission on it (the lockstep entry), and the rest are `WallOut::draw_health_bar`, `draw_overlays`, `is_force_construction`, `FarmsOut::draw`, `GraphicPieces::draw_build`. The calls on slot `+0x17c` in `Object::poor_target`, `Unit::find_attack_pos`, `Unit::find_path` take three arguments on **unit** receivers, a different method. A render visibility test: the same predicate as `is_seen`'s head, then `WorldData::is_seen` of the building's half-cell. Proposed census pattern **`NOW_SEEN`**: short name `^is_now_seen$` on a `*Data` class whose callers are `*Out`/`GraphicPieces`/`Wall::inc_time` — catches this row and `BuildData::is_now_seen@0062e130`. |
| 00642d90 | WallData::construct_time | layout | - | dead | `return constr_time`. Slot `+0x18c` is overridden by `BuildData::construct_time` on every live building; no caller by name (export grep). `docs/AI.md` §383.2: only a `Wall` instance reaches it, and none is built. `WALL_OVERLOAD` (above). The live construction clock is `Wall::update_construct_time`, which `borders-1` ruled `twin` (`build.rs:518`). |
| 0063e4b0 | Wall::activate | layout | y | twin-differs | Live on `Build::vftable +0x1a8`, called by `Build::activate@00623e20`. Twin: the `Wall::activate` part of `Sim::activate` (`crates/sim/src/cities/city.rs:2277`; `docs/CITIES.md` §4 "The common part"). Agree: `ever_seen_completed (+0x63) \|= ally_mask[who]` (`city.rs:2283`, item 1452, run622); not started → vslot `+0x1a4` `Wall::start` (`start_building`); `flags \|= 4` and `build_masks \|= 0x1000` (`active`, `activated`, `lib.rs:1729`); `job_counter = job_counter_2 = 0`; the `increment_stats` guard (`build_flags +0x2c0 & 0x10`, or vslot `+0x20` and the building's city `+0x72 >= 0`) → `mil_trainer_open`; `leader_flags \|= 0x2000000 \| 0x8000000` → `wall_stats_dirty`. The `num_queued[type]` decrement (`leader +0xe3fdb2` short array) and the unit-counter tail (`+0xa10..0xa24`, needs `is_unit_type`, inert for a building) are `borders-1`'s `increment_stats` ruling (recounted live here). Vslot `+0x20` is 1 on a `Build`, so the `update_seen(0)` arm is `Build::activate`'s own tail. UI/sound/render: `options->rebuild`, `scene->recalc_builds`, the console's `S_CREATED` and `MessageWin::add_event`, `Doober::remove_cover_doober`. **Differs** in one write: an `is(0x20b)` building (the Air Defense line) starts with `job_counter = 0x40000000`, the crate with 0. On a finished building with objmask `0x80000000` `job_counter` is the turret's angle, which `Build::process@0061edf0:116–124` turns toward the target by at most `0x38e38e3` a frame and only `WallOut::draw*` reads — no simulation reader found (export grep of `field_0x48`). Field: `BUILDDATA/WALLDATA` `job_counter` on a finished Air Defense, which the harness compares only on an unfinished site (`crates/rondata/src/diff/harness.rs:687`); widening it would part on any capture holding one, and score nothing. |
| 0063e9b0 | Wall::init | layout | y | twin | Live on `Build::vftable +0x1a0`, called from `Build::init`. Twin `Sim::init_build` (`crates/sim/src/cities/city.rs:1473`), step for step: `flags &= 0xf9` and `build_masks`, `job_counter{,_2}`, `helpers` zeroed (the fresh `Building`; `demolition` and `frame_started = -1` are not carried, both printed and pinned uncompared at `crates/rondata/src/diff/coverage.rs:192`); `start_me(1)` (the `PLACED`/`PLACED_TWICE` loop, `city.rs:1543`, which `borders-1` ruled agreeing); `update_construct_time` (`construct_base`), `update_hits(0)`, `update_los` (an unstarted site's byte is 0, which `build_los` answers), `check_ever_seen(0)` (`city.rs:1561`, item 1446, run610); under `param_6 == 0` (`!restore`) and `type != FARM` (an identity, not a lineage) `terraform_for_building` (`city.rs:1584`, run64/run72, `docs/ROADS.md` §7.6) — agree. Absorbed elsewhere: `leader_flags \|= 0x2000000` (the economy's dirty bit, `docs/AI.md` §13430 item 2) is raised by the callers, `place_building` (`city.rs:1461`) and the AI's `ai_sites.rs:1244`, before `init_build`; the closing 5×5 (9×9 for a city, `is(0x19e)`) cell walk calling `Unit::update_z` on every ground unit is covered by the crate reading a unit's height from its tile (`World::tile_z`, `orders.rs:2210`) rather than storing `z_internal`. Render/sound: the console group's `S_CREATED`, `scene->recalc_builds`, `Terrain::refresh_good_z`, `Farms::recalc_heights` (float mesh heights), `set_new_location`. The `is(0x19e)` call under `param_6 == 0` discards its result. |
| 00643070 | Wall::update_gpiece | layout | y | ui | Chooses the building's art: `gpiece (+0x58)` and `render_gpiece (+0x68)` from `GraphicPieces::get_build_gpiece(type, who or city race, …, age offset from leader +0x6eb8 +0xdc)`, the memory grade, the console's `original_who`, and `GraphicEvents::verify_load`. Callers: `SubObject::set_type`, `Build::swap_team`, `Leader::gain_tech` (an age's art), `Objects::reload_data`, the scenario editor. No simulation reader of a building's `+0x58` was found (export grep of `field_0x58` over `Wall`/`Build`/`Object`; the `Unit` hits are a different layout), and `+0x68` is read only by `WallOut::get_gpiece`'s render callers (row above). `docs/PARKED.md` 3395d: "a wall's piece choice is art state". Field: `BUILDDATA/WALLDATA` `gpiece`, pinned uncompared (`crates/rondata/src/diff/coverage.rs:192`). Proposed census pattern **`GPIECE`**: short name `^(update_gpiece\|get_gpiece)$` on `Wall`/`WallOut` — catches this row and `WallOut::get_gpiece@006424d0` (`Unit::update_gpiece@005e2920` and `Guy::update_gpiece@005d8530` are excluded: the crate ports them, `docs/PARKED.md` 3164/3229b). |

## Summary

**Counts (19):** `twin` 5 · `twin-differs` 4 · `ui` 4 · `dead` 3 · `absorbed` 2 · `saveload` 1 ·
`unmodelled` 0. Twins: `valid_buildmask`, `is_neutralized`, `get_garrison_limit`, `Wall::die`,
`Wall::init`. Every live-slot simulation body of this half of the stem has a crate counterpart;
the four `twin-differs` are each a stated `SEAM` or an inert constant, and none is a rule the
crate lacks outright.

**Proposed census classes** (Ramon's to accept):

- **Mark ahead of `layout`** — not a new class but an ordering: `census.territory` returns
  `layout` for a swept function (`tools/census.py:1228`) before it tests `is_ui_body` (`:1233`),
  so a swept `*Out` body stays open though the existing mark names it. Catches
  `WallOut::make_options@00642470`, `WallOut::get_gpiece@006424d0` here, and every swept `*Out`
  body elsewhere.
- **`WALL_OVERLOAD`** — exact names, `docs/AI.md` §383.2's predicate (no live object is a most-derived
  `Wall`; `Build` overrides the slot; no caller by name): `WallData::hits@00642bb0`,
  `WallData::construct_time@00642d90`, `WallData::can_make@00472cb0` (`borders-1`).
- **`SLOT_0x140`** — short name `^support_cost$` on an `*Data` class, §383.1's predicate (slot
  `+0x140` never dispatched on an object): `WallData::support_cost@0063fa30`,
  `ObjectData::support_cost@006452f0`, `UnitData::support_cost@0060fde0`.
- **`DATA_WALK`** — short name `^walk(_rules)?_data$`: `WallData::walk_data@00642510`,
  `BuildData::walk_data@0062f270`, `AnimalData::walk_data@005d7ea0`, and every `Object`/`BuildQueue`/
  `Array<…>`/`PtrLinkList…` sibling. The `DataWalk` visitor serves only `SaveGame`, `LoadGame` and
  `CheckSum`.
- **`NOW_SEEN`** — short name `^is_now_seen$` on `WallData`/`BuildData`: `WallData::is_now_seen@00642ca0`,
  `BuildData::is_now_seen@0062e130` — the console player's render visibility (every building
  dispatcher of slot `+0x17c` is a renderer or `Wall::inc_time`'s particle arm).
- **`GPIECE`** — short name `^(update_gpiece|get_gpiece)$` on `Wall`/`WallOut`: `Wall::update_gpiece@00643070`,
  `WallOut::get_gpiece@006424d0` (not `Unit::`/`Guy::update_gpiece`, which the crate ports).

**Proposed replaced group:** **"save, load and the desync checksum"** — the `DataWalk` family
(`docs/ORACLE.md` "write_header": `Game::walk_data(SaveGame/LoadGame)`; `docs/FORMATS.md`:
`CheckSum : DataWalk`). The crate has no save format and its determinism check is the soak's
digest, so the family is replaced by design rather than unported; evidence is
`WallData::walk_data@00642510` here and the `DATA_WALK` names above.

**`unmodelled`:** none. In their place the four `twin-differs`, ranked; all four are lockstep-entered:

1. **`WallData::armor@0063fa60`: the Senator arm.** A `THESENATOR` (`0x161`) within
   `(x_size + y_size)·96` of a struck building, when its owner's `leader +0x59c0` is set, adds
   `THESENATOR_BUILD_ARMOR` (**5** in the shipped `rules.xml`) before the unfinished halving. It is
   a live rule, `SEAM` at `fight.rs:474`, parked 1359. Field: the building's `hits` (`BUILDDATA`)
   and the `GUY` block's `damage`/`damage_frac` on a capture where a Senator stands near a
   building under attack.
2. **`WallData::is_seen@00642bd0`: four arms not carried.** `reveal_map == 3`,
   `leader_flags & 0x800`, `leader +0x59e4`, and the city-centre arm under the semaphore. The
   crate can only refuse further. Field: `UNITDATA` `order:target` on a building target, cheapest
   in a `reveal_map 3` lobby.
3. **`Wall::update_los@0063eeb0`: three omissions in `build_los`.** The colosseum and furs terms
   (both ship 0, parked 3631a), the `LEADER_VALID` gate (always set for an owner) and the 8-bit
   wrap. Inert on shipped rules. Field: `BUILDDATA/WALLDATA/OBJECT` `mylos`, pinned uncompared
   (`coverage.rs:205`); comparing it is a widening.
4. **`Wall::activate@0063e4b0`: the Air Defense's start angle.** `job_counter = 0x40000000` for
   `is(0x20b)`, 0 here. On a finished building this is a turret angle with no simulation reader.
   Field: `BUILDDATA/WALLDATA` `job_counter` on a finished Air Defense, compared only on unfinished
   sites (`harness.rs:687`).

## Not established

- **Which `DataWalk` the lockstep runs entered** `WallData::walk_data` through: the
  `CheckSum` (`CheckDesyncsEveryXFrames`), an autosave, or the logger. Grep the trace for
  `CheckSum::`/`SaveGame::` entries on run1016–run1021 to settle it.
- **`leader +0x59c0`, `+0x59e4` and `leader_flags & 0x800`** were not named from `types.txt`.
  `+0x59c0` gates `armor`'s hero search, and the other two are `is_seen` arms. Item 1350 / §4.2
  names `+0x59c0` only by its offset.
- **The first byte's bit `4` of both leaders** in `is_seen`'s city-centre arm, and semaphore bit
  `4`, are unnamed. The arm is listed, not interpreted.
- **A doc inconsistency, not settled here:** `docs/CITIES.md` (line ~1536) calls
  `leaders.list[who] |= 0x2000000` "a repaint flag … nothing this simulation reads". The crate
  and `docs/AI.md` §13430 treat it as the economy's dirty bit (`economy_changed`). The `Wall::init`
  row follows the crate. The CITIES paragraph looks stale.
- **`Wall::init`'s `Unit::update_z` walk** was called absorbed on the crate's own `tile_z`
  reading (`orders.rs:2210`). Whether any unit `z` stored elsewhere in the crate (a crew figure,
  an aircraft) needs the refresh was not checked. The walk skips types with `+0x218 != 0`, which
  was not named.
- **No simulation reader of a finished building's `job_counter` (`+0x48`) or `gpiece` (`+0x58`)**
  rests on a grep of the decompiler's `field_0x48`/`field_0x58` spellings over `Wall`, `Build`,
  `Object`, `Unit` and `Objects`. A reader that spells the field by name, or reaches it through
  another class's view, would be missed.
