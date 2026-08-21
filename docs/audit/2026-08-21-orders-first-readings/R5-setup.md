# R5 — The start of a game: what exists at frame 0, and which orders the starting units carry

*First reading, 2026-08-20, from the decompile export under
`~/ghidra-projects/decomp/` (`funcs/`, `types.txt`, `vtables.txt`,
`enums/TypeIndex.txt`) and, for one jump table the decompiler could not
recover, `llvm-objdump` over the shipped PE. Written from understanding; the
decompile was read, not transcribed.*

The question this answers for the harness (`docs/DATALAYER.md` §3): the
frame-0 dump is taken **after `Setup::build_game` has run and before the first
`Game::do_frame`**, and the starting citizens of a Small/Large Town start are
**created beside their assigned building with a `GATHER` order already in
their order list** — which is why units `1–5` stand next to buildings
`2001–2005` and start walking without anyone issuing anything. The AI's scout
carries no order at creation; its movement is `Leaders::strategy_all`, which
runs on the very first `do_frame` (`Leader::plan_strategy` does not skip
frame 0).

---

## 1. What it is

### 1.1 The actors

| thing | where | what |
| --- | --- | --- |
| `Setup` | `struct Setup // size 0x4` (`types.txt:44777`) — a stateless namespace-as-class; every method is `__thiscall` with an unused `this` | builds the starting world: `build_game@005ac190`, `build_empire@005abb80`, `build_cities@005ab910`, `build_units@005aafc0`, `place_unit@005abca0`, `small_city_buildings@005aae10`, `large_city_buildings@005aac80`, `build_civ_specific@005ab760`, `get_starting_citizens@005aaf50`, `build_leader@005abc70` (`Leader::init` pass-through), `reinit_game_units@005abf90` (re-reads every object's terrain z after terrain is up), `init_wild_life@005aac70` (`return 0`) |
| `Game` | `struct Game // size 0xc70` (`types.txt:44556`): `info` +0xc (`GameInfo`), `frame` +0x550, `tick` +0x560, `starting[6]` +0x600, `start_list[8]` +0x65c, `start_index[8]` +0x67c, `num_nations` +0x6a0, `semaphore` +0x814 | the lobby → `info`; `frame`/`tick` zeroed by `build_game`; `starting[]` is the six stockpile amounts every player begins with |
| `GameInfo_u_24_s_0` (`info` +0x18) | `team_style` +0, `starting_town` +0x8, `starting_resources` +0x9, `starting_resources2` +0xa, `reveal_map` +0xc, `starting_technology` +0x10, … (`types.txt:44636`ff) | the lobby's combo indices, one byte each — `starting_town` and `starting_resources` are the two Setup reads |
| `ObjectsData` | `lists[10]` +0x4 (`ObjectsArray`, 0x1c each — the per-player object table, data pointer at +0x10 so `field_0x14 + who*0x1c`), `unit_mark[10]` +0x15c, `build_mark[10]` +0x184, `wall_mark[10]` +0x1ac (`types.txt:27006`) | the id scheme, §1.2 |
| `WorldData` | `xs` +0, `ys` +4, `tile_xs` +0x18, `start_x`/`start_y` +0x80/+0x9c (`SimpleArray<WCoord>`, data at +0x10 → `field_0x90`/`field_0xac`), `wdata` +0x134 (`WData`, 0x1c per cell: `flags` +0, `land` +2, `region` +4, `down` +8), `tdata` +0x138 (`types.txt:18235`ff) | the map-maker's start cells, one per start slot |
| `LeaderData` | `leader_flags` +0 (bit 0 active, bit 2 human — `Game::solo_init` tests `& 5 == 5`), `leader_flags2` +4 (bit 7 = CtW nomad), `who` +8, `tribe` +0xc, `cities_built` +0x820, `active` +0x93c, `home_reg` +0x9c8 (`types.txt:42088`ff) | |
| `SubObjectData` | `flags` +8 (bit 0 alive), `who` +9, `o` +0xa, `z/x/y_internal` +0xc/+0x10/+0x14 (stored `^ 0x63637`), `ptype` +0x18 (`types.txt:43430`) | every object's header |
| `UnitData` | `unit_masks` +0x68, `o_up`/`o_down` +0x8e/+0x90 (the uber-squad chain), `orderlist` +0xc8 (`OrderList`, 0x1c; its `LinkListBase` at +0xcc), `guys` +0xe4 (`types.txt:43325`) | |
| `GatherOrder` | size 0x34: `TargetOrder` base `ox` +8, `whom` +0xc, `uid` +0x10; `tx`/`ty` +0x14/+0x18, `build_type` +0x1c, `wait` +0x20, `goto_build` +0x24, `non_flat_gather` +0x25, `dist_mod` +0x26, `been_there` +0x27 (`types.txt`) | what a starting citizen carries |

### 1.2 The id scheme — why units are `0..`, buildings `2000..`, walls `3000..`

`Objects::clear@0065d740` sets, for every active player, `unit_mark = 0`,
`build_mark = 2000`, `wall_mark = 3000` (via the `obj_mark[3]` pointer table
at +0x1e8). `Objects::find_free@0065ad60(who, base, limit, &mark, want)` is
the only allocator:

- `want ≥ 0` — take that exact slot (allocating every empty slot from `*mark`
  up to it) and set `*mark = want + 1`. Used by loaders, not by Setup.
- `want < 0` — scan `base .. *mark` for a **dead** slot (`flags & 1 == 0`,
  `hold_frames == 0`, and either not a unit or with `o_up < 0`) and reuse it;
  else if `*mark ≥ limit` → `-1`; else allocate at `*mark`, `mark++`, return
  the old mark. The object constructed is `Unit` (0x158; `Animal` for
  `who ≥ 8`) for `base == 0`, `Build` (0xdc) for `base == 2000`, `Wall` (0x74)
  otherwise.

`Objects::init_unit@0065e0c0` calls `find_free(who, 0, 2000, &unit_mark[who], -1)`;
`Objects::init_build@0065d190` and `Setup::build_cities` call
`find_free(who, 2000, 3000, &build_mark[who], -1)`. So on a fresh
`Objects::clear`, **the first unit a player creates is `o 0`, the first
building `o 2000`**, and the ids are per player. In the default start the
scout is placed before the citizens (§2.4), so scout = `0`, citizens = `1..5`;
the city is `2000` and the five buildings `small_city_buildings` produces are
`2001..2005` in the order produced (§2.3).

`init_unit` creates `uber_size` (`UnitTypeData +0x308`) `Unit` objects per
call, chained by `o_up`/`o_down` (`+0x8e`/`+0x90`); the first is the captain
and is placed exactly at `(x, y)` by `Unit::init@00612100` (vtable `+0x8c`,
`set_new_location` at `init:549`); followers are `find_nearby_spot`-placed
around the captain. Each fresh `Unit::init` does `close_orders(this, 0);
clear_partial_path; update_action` (`init:107`) — **a fresh unit's order list
is empty**, and `o_up = o_down = -1`, collide fields `-1`. For a citizen
`uber_size` is 1.

### 1.3 Coordinates

One tile is `0x300 = 768` position units; the tile centre is `+0x180`
(`Setup::build_units`: `x = start_x * 0x300 + 0x180`). `div_3_table[c >> 8]`
is the tile of a position (`c / 768`); `div_3_table[c >> 6]` is the quarter-
tile (`c / 192`). `BuildTypeData::snap_center` snaps a building's centre for
its footprint (`x_size`/`y_size`, `ObjectTypeData +0x234/+0x238`).

---

## 2. The sequence — from `Game::run` to the first `do_frame`

### 2.1 Before the world: `Game::run@00584590`

In order, each cited by the line of `run` where it sits:

1. `GameLog::init` (`run:300`); `game_random->random_seed = info.seed`
   (`run:302`) — the lobby seed (`SEED` in `-config`, `Seed (0 for random)`
   in `rise.ini`, `docs/ORACLE.md`).
2. `Game::init_rules_and_teams@00589bb0` (`run:312`). Inside, in order:
   `starting_technology == 9` → randomised (`:106ff`); `Map::set_map`
   (`:132`); **`init_starting_resources`** (`:134`, §2.2);
   `Leaders::init` (`:136`); … `num_nations/num_players` (`:348`);
   `everyone_mask`; **`init_teams`** (`:361`, fills `info.player[].team`
   and `Game::on_team[]` — only skimmed); **`init_handicaps`** (`:363`,
   per-player handicap from `info.player[].handicap` and `difficulty` —
   only skimmed); **`init_tribes`** (`:365`, `leaders[].tribe` from the
   lobby, with the "random" choices drawn from `game_random` —
   `init_tribes:52–146`; a random tribe is `rand % 0x18`, 24 tribes).
3. `init_console` (`run:323`), `init_type_backup` (`run:326`).
4. `Game::init@0058c480(0, is_scenario, 0)` (`run:460`) → `init_data`
   (`init:313`), `init_camera`, …, `World::init` (`init:466`),
   **`Setup::build_game`** (`init:544`, §2.2–§2.5), then
   **`Setup::reinit_game_units`** (`init:629`) — every alive object's
   `z_internal` re-read from `TerrainOut::find_tcoord_z` at its tile, units
   `0..unit_mark`, builds `2000..build_mark`, walls `3000..wall_mark`.
5. `clock_init` (`run:480`), `solo_init` (`run:483` — hotkey-group
   housekeeping for human players only; nothing of the sim).
6. `playing = 1; GameLog::begin_game` (`run:703–704`) — **the
   `[Start Game]` dump, `GAMELOGMODE_START_GAME`, at `game->frame == 0`**
   (`GameLog::begin_game@00932b70` ends in `full_dump`). This is the frame-0
   state the harness builds from.
7. `Game::loop@00591570` (`run:1104`) → `TurnControl::do_frame` →
   `Game::do_frame@00591ef0` (`loop:51–58`), §4.

`Setup::build_game` has exactly two other callers: `ConsoleWin::run_cmd@007d6a70:4334`
(a console restart, `build_game(0,0,0)`) and itself (unwind funclets).

### 2.2 `init_starting_resources@0058a500` — the six stockpiles

For `i in 0..6` (the six goods, `Game::starting[i]` at +0x600):

- `starting_resources == 12` (`'\f'`) → first replaced by `rand % 12`
  (`game_random`).
- `base = Constants +0x234 + 4i` = **`starting_goods[i]`**
  (`types.txt` Constants `+0x234 int[6] starting_goods`) — the decompiler's
  `(-0x3cc - this) + piVar6 + constants` is `constants + (0x600 - 0x3cc) +
  4i`; and when `starting_resources == 7` and `i != 3`, `base =
  starting_goods[0]` (the food amount for every good but wealth).
- The lobby option row `starting_resources.list[setting]` (0x58 bytes per
  row; `+0x3c` = `lo`, `+0x40` = `hi` — the same row shape `rush_rules.list`
  and `cannon_times` use) gives `lo` and `span = hi − lo`:
  `lo == 0` → `starting[i] = base / 2`; else
  `starting[i] = lo * base + (span*base > 1 ? rand % (span*base) : 0)` —
  **a random draw from `game_random` per good** whenever the row has a
  spread.
- `starting_resources == 8` → `starting[i] = 99999` regardless.

`Leader::init@006e3930:881–897` then pays it: for every good `type_avail`,
`bucket_add(t, starting[t])` — Barbarians (`game_rules == 8`) defending team
gets `(starting_resources2 + 1) ×`; a CtW nomad gets
`× ctw_starting_res_x`. The lobby's start *age* is a different path
(`docs/TECH.md`, `starting_technology`, `LeaderData::starting_age`).

### 2.3 `Setup::build_game@005ac190` — the world, then one empire per player

In order (`setup.cpp` line numbers are the `say_checksum` markers):

1. `game_random->random_seed = info.seed` again (`:157`); `semaphore` bits;
   **`frame = tick = 0`**, `world_pop/cities/villages/total_units = 0`,
   `market[k] = 0x32, market_flux = next_flux = 0xf, flux_length = 1`
   (`:330–345`); `Objects::clear`, `Groups::clear`, per-leader counters
   zeroed; `world->seed = rand(0..0xffff) % 0xffff + 1` (`:2cc`).
2. Random-map path (`param_2 == 0`): `Map::new_map` → `game_map->make(num_nations, -1, splash)`
   (vtable `+0x64` = `Map::make@0068bc90`) — the map-maker places the start
   cells into `world->start_x/start_y` (`Map::place_start_in_region@0068ac00`
   picks a region cell with `game_random` draws, `:56`, and keeps it at
   least 5, then 3, cells from the edge) — and `randomize_starts`
   (`Map +0x2ec`) is remembered; `Terrain::init` (`:2e9`). The scenario path
   (`param_2 != 0`) instead runs `ScenarioRead::import` and skips all of
   §2.3.4–6 except `Leader::reset_obs_flags`; CtW (`semaphore[2] & 2`)
   runs `ConquestGame::build_conquest_game`. **Out of scope here; note only
   that the random-map path is the one that reaches `build_empire`.**
3. `Terrain::mark_forest` (`:374`).
4. **Start-slot order.** `Game::start_list[k]` (+0x65c) is filled with the
   player indices `0..7`: a random permutation when `randomize_starts`
   (`rand & 7`, re-drawn until a fresh slot — `:5ac…`), else identity.
   Then `Leader::init(who, tribe, who)` for every active `who` (`:3bb`),
   graphic preloads, `Leader::reset_score`.
5. **`build_empire(who, slot)`** for each `k`, `who = start_list[k]`,
   `start_index[who] = k`, `slot` = a running counter when randomised else a
   compacted index of active players (`:47c` region). Under `team_style 7`
   and `map != 0x16` teammates are built consecutively after their first
   member. A player is built once (`local_138[who]`).
6. `World::analyze_map`, **`World::compute_all_territory`** (`:491`),
   `Herd::create_units` for each herd, scenario script compile, CtW setup
   script, `ImageIO::process`. Return 0.

**`build_empire@005abb80(who, slot)`**: lobby flag `info.flags & 4` ("no
nation powers") marks two types; if `semaphore[1] & 2` (unit-balance test
mode) → `UnitBalance::init`; else `city = build_cities(who, slot,
starting_town)` then `build_units(who, slot, city, starting_town)`; Spanish
(`has_tribe_bonus(9)`) → `leader_flags |= 0x1000`.

**`build_cities@005ab910(who, slot, starting_town)`**:

- `home_reg = wdata[start_y*xs + start_x].region` — always, even for a nomad.
- `starting_town == 0` or a CtW nomad (`leader_flags2 & 0x80`) → return `-1`
  (no city; the "Nomad" start).
- Else: `o = find_free(who, 2000, 3000, &build_mark[who], -1)` (= 2000 on a
  fresh table); `snap_center` of `VILLAGE` (`0x19e`, the first city type)
  at `(start_x*0x300, start_y*0x300)`; **`Build::init(who, VILLAGE, o, x, y, 0)`**
  (vtable `+0x1a0`); Chinese (`bonus 0xe`) with `CHINESE_LARGE_CITY` or
  `_CITIES` → `set_type(TOWN 0x19f)`; **`Build::activate(0, 0, 0)`**
  (`+0x1a8`, `captured=0, announce=0, counted=0` — `docs/CITIES.md` §4) —
  so **the starting city is a finished, active city at frame 0**;
  `cities_built++` by hand (since `counted = 0`); `update_seen(0)`,
  `update_seen_ally`; `city = BuildData.city` (`+0x72`); if `city ≥ 0`:
  `World::compute_all_territory`, then
  `starting_town == 3` → `large_city_buildings` + `build_civ_specific`;
  `== 2` → `small_city_buildings` + `build_civ_specific`; `== 1` →
  `build_civ_specific` only. Return `o`.

**The pre-placed buildings — `small_city_buildings@005aae10(who, city)`**,
each a `Leader::produce_building(leader, type, city.o, 0)` (the city record's
`o`, `CityData +0x8`), in this order: `WOODCUTTER` (`0x1a2`); then unless
Lakota (`bonus 0x13` — skipped entirely) — Americans (`bonus 0x14`) with
`AMERICANS_STARTING_FARMS > 0` → that many `FARM` (`0x1a1`), else **three
`FARM`**; then `LIBRARY` (`0x1b3`). So the default is **2001 woodcutter,
2002–2004 farms, 2005 library**. `large_city_buildings@005aac80`:
`WOODCUTTER`, five `FARM` (none for Lakota), four `TOWER` (`0x1b7`),
`LIBRARY`. `build_civ_specific@005ab760` then adds: `MARKET` for Nubians (4)
or Dutch (0x16); `UNIVERSITY` for Greeks (5) if `GREEK_UNIVERSITY_EARLY > 1`;
`TEMPLE` for Koreans (0x10) if `KOREAN_TEMPLE_UPGRADES > 1`; `GRANARY`
Egyptians (7) if `EGYPTIAN_GRANARY_EARLY > 1`; `LUMBERMILL` French (10) if
`FRENCH_LUMBERMILL_EARLY > 1`; `SENATE` Iroquois (0x12) if
`IROQUOIS_SENATE != 0`. (Powers by roster index, `docs/TECH.md` §has_preq.)

**Who chooses the site: `Leader::produce_building@006e1400` — the AI's own
building placer, with `game->frame == 0` special cases.** Not the map-maker,
not a fixed offset table. The shape (read, not transcribed — this is AI code
and belongs to phase 5):

- Refuses unless the anchor is an active city (or the type has
  `build_flags & 0x10`). Anchor tile = the city's; search radius index
  `local_c = (get_radius(city type) + 2) / 4` cells, spiral over
  `circle_x/circle_y[1 .. circle_radius[local_c])` (the same
  `circle_*` tables `docs/CITIES.md` §13 and `docs/COMBAT.md` use).
- Candidate filter per cell: in bounds, `wdata.flags & 0x4000` clear,
  `WorldData::check_building_wcoord` (footprint from `x_size/y_size`), the
  footprint fits the map edge, `buildings_allowed`, `is_ocean` agreeing with
  the type's `domain` (`+0x218 == 1` is sea), then `BuildTypeData::blocked_site`
  at the snapped centre (`docs/CITIES.md` §2.6).
- Score per candidate: **`FARM`/`MINE`: `find_friends` (adjacent same-type
  count `n`) → `(n+2)*1000`, else `4000/dist + rand % 500`** — a
  `game_random` draw per candidate; **`WOODCUTTER` at frame 0: 1000 if
  `dist ≤ 3`, 500 if 4, 250 beyond, times the cube of the forest count that
  `blocked_site` returned**; others `gather_at`-weighted. Best score wins.
  So **the starting sites depend on the RNG stream and the map, not on a
  table**; they are determined by the seed.
- Then a second pass on the winning cell (`:2a78ff`): for every type but
  the woodcutter (and not a `DOCK`), a 2 × 2 box of sub-positions, each
  unblocked one drawing `rand % 100`, the max wins — another draw per
  cell; the woodcutter instead scans the `move_x/move_y` offsets within
  `radius[4 − x_size]` for the most forest (the count `blocked_site` returns
  through its last argument), no draw.
- **Creation at frame 0:** `Objects::init_build(who, type, x, y, 0, -1)`
  then **`Build::activate(0, 1, 0)`** (`announce=1, counted=0`) — **the
  starting buildings are complete and active at frame 0, not construction
  sites**. (`frame != 0` is the AI's live path: pay the cost via the type's
  `+0xd4` slot, `init_build`, pick the nearest idle citizen, and
  `Group::action_swarm_around(... BUILD_AT)`.) `city.buildings_count`
  (`CityData +0x64`) is incremented either way.

### 2.4 `Setup::build_units@005aafc0(who, slot, city, starting_town)` — the units and their orders

Start tile `(sx, sy) = (start_x[slot], start_y[slot])`; centre
`(sx*0x300+0x180, sy*0x300+0x180)`.

**How many citizens.** `get_starting_citizens@005aaf50(&n, &ordered, starting_town)`
is a four-way jump table the decompiler could not recover; the PE bytes at
`0x5aaf50–0x5aafa6` (`llvm-objdump`) settle it: `(n, ordered)` =
**`(3, 0)` for 0, `(4, 0)` for 1, `(5, 2)` for 2, `(10, 5)` for 3**;
`starting_town ≥ 4` → `n = 0, ordered = 0` (the jump table is not consulted).
Then: `starting_resources == 7` → `n += 8`; `== 8` → `n += 12`;
Americans with `AMERICANS_STARTING_FARMS > 3` and `starting_town == 2` →
`n += farms − 3`; Lakota (`0x13`) → `n −= 3` (small) / `−= 5` (large);
Koreans (`0x10`) → `n += KOREAN_CITIZENS[0]`.

**The scout first** (only when `starting_town != 0`): type =
`types[0x45 = SCOUT]` unless the tribe cannot use it, in which case
`TypeData::type_graft(tribe)` (vtable `+0x68`); `current_upgrade` of it;
`place_unit(who, city, type, centre)`. Spanish (`bonus 9`): +`SPANISH_EXTRA_SCOUT`
more, and one more if `reveal_map > 1`. Dutch (`0x16`): two
`MERCHANTDUTCH` by `place_unit`. Greeks (`5`) with `GREEK_START_SCHOLARS`:
that many `SCHOLARS` (`0x34`) by `place_unit`, each `Unit::go_inside` the
nearest friendly `UNIVERSITY` found by `ObjectsData::find_building` (only if
one exists — `build_civ_specific` will have made one when
`GREEK_UNIVERSITY_EARLY > 1`).

**`place_unit@005abca0(who, city, type, x, y)`** — the scattered placement:
`type = current_upgrade(type)`; if `city ≥ 0` the anchor is the city's
`(x, y)`. Up to 60 tries (500 for a Nomad): draw `idx = rand % circle_radius[r]`
with `r = 2` (`r = 8`, or `0x18` if the player has `active` units, for a
Nomad); cell = anchor tile + `(circle_x[idx], circle_y[idx])`; accept if in
bounds, **same `region` as the anchor tile**, `wdata.flags & 0x30 == 0`,
not water (`land` 1 or 2 with `flags & 0x100` clear), `flags & 0x100` clear,
**`wdata.down < 0` (no object on the cell)**, `tdata` word bit 14 clear and
low two bits not 3, `flags` bit 15 clear → `Objects::init_unit(who, type,
cell centre, -1, -1, -1)` and return. After 60 failures: no city →
`init_unit` at the given point + `Unit::come_out(u, 0)`; a city →
**`Build::train(city, type)`** (the unit is queued in the city instead).
One `game_random` draw per try. **No order is given.** The scout, and every
Nomad/City-only citizen, starts idle.

**The citizens**, `n` of them, in a loop with `i` = the citizen index and a
farm cursor `k` (starts 0):

- `starting_town < 2` → `place_unit(who, city, citizen type, centre)`:
  scattered, idle. (citizen type = `types[0x32 = PEASANTS/BASE_UNITTYPES]`
  or the tribe's graft.)
- `starting_town ≥ 2`:
  - `i < ordered` → target `t = 2001` (the first building after the city —
    the **woodcutter**), if `lists[who][2001]` is alive.
  - else scan `t = 2002 + k, 2003 + k, …` up to `build_mark`, stopping at
    the first alive object that `is(FARM, 0)` (or at the first dead slot);
    accept it if `is_active()` (vtable `+0x10`), its type's `+0x90` slot
    (a "gatherable building" predicate on the type) is true and it is not a
    `UNIVERSITY`; the cursor advances past it. If nothing is found, fall
    back to `2001` again if it is active, gatherable, not a university and
    `BuildData::num_gatherers < gather_max` (`+0x80`); else `place_unit` at
    the city (idle).
  - **For a target `t`:** `o = Objects::init_unit(who, citizen, t.x, t.y, -1, -1, -1)`
    — created **at the building's centre**; **`Unit::come_out(u, 0)`**;
    then, unless `starting_resources == 8`, **`Unit::add_gather_order(u, t, QUEUE_NEW, 0)`**.

So for the default Small Town: citizens `1, 2` are created at the woodcutter
`2001` with `GATHER 2001`; `3, 4, 5` at farms `2002, 2003, 2004` with
`GATHER` on each. The library `2005` and the city `2000` get nobody. Large
Town: `1..5` on the woodcutter, `6..10` on farms `2002..2006`.

**What `come_out(u, 0)` does to a unit that is inside nothing**
(`Unit::come_out@00617c10:172–215, 518`): a non-captain defers to its
captain; `get_inside < 0` → `UnitType::find_nearby_spot` around its own
position with `min = block_radius` (`ObjectTypeData +0x240`),
`max = block_radius + UNIT_DISEMBARK_DISTANCE`, `FILTER_NOT_ME`, and
`set_new_location` there — i.e. **the citizen steps off the building's
footprint to the nearest free spot beside it**. For a non-human owner it
also `close_orders` non-civilian types (`:560ff`) — citizens and scholars
are exempt, so it does not matter here. That is why the dump shows each
citizen *beside* its building, not on it.

**What `add_gather_order(u, t, QUEUE_NEW, 0)` leaves** (`Unit::add_gather_order@0061a5c0`):
`QUEUE_NEW` → `unit_masks &= ~0x4000000`, `+0xc0 = 0`, `close_orders(0)`,
`clear_partial_path`, `update_action`; `unit_masks &= ~0x400`; a `GATHER`
order from `OrdersMemManager` with `ox = t`, `whom = who`,
`uid = t.uid` (`ObjectData +0x30`), `build_type = t.ptype->type`,
`tx = ty = -1`, `non_flat_gather = 0`; if the target type's `+0x90` slot is
true: **`Build::add_gatherer(t, o, who)` — the citizen is registered on the
building immediately, at setup**, and if the type's `+0x94` slot is false
and it is not a `UNIVERSITY`: `non_flat_gather = 1`, `dist_mod = 10` for a
`MINE` else `4`; `param_3 == 0` → the order's `flags & ~4`; a cross-region
target on a transportable unit sets `unit_masks |= 0x800000`; then
`orderlist.add(order)`; `update_action`. **That `GATHER` order is the only
thing in the citizen's list at frame 0.** Its first step is the Gather
reader's (seam §3).

### 2.5 What else the start fixes

- `Game::start_index[who]` (for the lobby's "start position" display);
  `LeaderData::home_reg`.
- The market and flux tables, `world_pop = 0` etc. (§2.3.1).
- `World::compute_all_territory` twice (after the city, after all empires) —
  the borders at frame 0 are already the city's full territory
  (`docs/ATTRITION.md`).
- Herd animals (`Herd::create_units`), who `≥ 8` (Animals, `find_free`'s
  `who < 8` test).

---

## 3. The seams

| from | to | what crosses |
| --- | --- | --- |
| `build_cities` | `Build::init(who, VILLAGE, o, x, y, 0)`, `Build::activate(0, 0, 0)` | `docs/CITIES.md` §3/§4: a finished city at frame 0 — `Cities::init_city`, `find_buildings`, `calc_pop_cap`, capital flag; `counted = 0` so `cities_built` is added by hand |
| `produce_building` at frame 0 | `Objects::init_build(who, type, x, y, 0, -1)`, `Build::activate(0, 1, 0)` | complete farms/woodcutter/library at frame 0 — `gather_slots += gather_max`, `LIBRARY → new_library` (`docs/PRODUCTION.md`) per CITIES §4 |
| `build_units` | `Objects::init_unit` → `Unit::init` (`+0x8c`) → `set_new_location` | `docs/MOVEMENT.md`: the unit is placed, `angle` etc. initialised; `Leader::track_unit_type`, `active--`, `units_built--` (the pop accounting inside `Objects::init_unit@0065e0c0`) |
| `build_units` | `Unit::come_out(u, 0)` → `find_nearby_spot`, `set_new_location` | the off-footprint step |
| `build_units` | `Unit::add_gather_order(u, t, QUEUE_NEW, 0)` → `Build::add_gatherer(t, o, who)` | **the Gather order reader**: on frame 0's `Objects::process_all`, the citizen's `GatherOrder` is the action; `Build::add_gatherer` has already counted it on the building (`BuildData::gather`, `num_gatherers`) — the economy (`docs/ECONOMY.md`, `calc_gather`) sees the gatherer from frame 0 |
| `place_unit` | `Build::train(city, type)` (fallback only) | `docs/PRODUCTION.md` |
| `Leaders::strategy_all` (frame 0) | `Leader::plan_strategy` → explore | the AI's scout order — phase 5; for the harness it is an order stream input |

---

## 4. Where it sits in the frame — and the frame-number convention

`Game::do_frame@00591ef0`, in order: `GameLog::begin_frame` (`:108`,
`[Start Frame]` dump at the current `frame`); player speed; scenario script;
`Leaders::process_all` (`:199`, income first — `docs/ECONOMY.md`); rush-rule
message; **`Leaders::strategy_all`** (`:267`, the AI, unless
`semaphore[1] & 8`); `GameDaemon::process_all` (`:270`); `Armies::process_all`;
**`Objects::process_all`** (`:275`, every unit's order step, `move_step`,
`fight`, attrition, the building clocks); `Objects::inc_time`;
`GraphicEvents::process`; `Leaders::end_process_all`; `Achieve::capture_data`;
`Leader::process_event_frame`; **`frame = frame + 1`** (`:293`);
`OrdersMemManager::cycle`; `Roads::scan_and_kill_stray_roads`; `frame % 15 == 0 → tick++`
(`:296`); **`GameLog::end_frame`** (`:315`, the `[End Frame]` dump).

**Frame numbering in `gamelog.txt`.** `GameLog::check_accept@009309a0:41–52`
opens a new `FRAME <n>` block the first time a line is accepted with
`last_frame_logged < game->frame`, using `game->frame` — and `end_frame` runs
**after** the increment. So:

- the `[Start Game]` dump (`GameLog::begin_game`) is the state after
  `build_game`, at `game->frame == 0`, before any `do_frame` — the
  "frame-0 dump";
- **`FRAME n` ([End Frame]) is the state after `n` passes of `do_frame`**,
  the n-th pass having run with `game->frame == n − 1`. `Leader::plan_strategy@…:69–76`
  runs when `frame == 0` **or** on its cadence, and `Leader::check_explore@006bc860:14`
  likewise — so the AI can order its scout during pass 1 (frame 0), and the
  scout's position first differing at `FRAME 2` is one pass for the order
  (and its path request through `GameDaemon`) and one to move. The AI's
  scout therefore moves because of an **AI order issued in `do_frame`**, not
  a scripted start; nothing in `Setup` orders it.
- A human's citizen "moving on frame 4": it was created with its `GATHER`
  order before pass 1, so three passes went by before its position differed
  in the dump. What those passes are is the Gather reader's — the `wait`
  field (`GatherOrder +0x20`), `hold_frames`, or the `GameDaemon` path
  service are the candidates; this reading does not settle it.
- There is no sim pass between `build_game` and `begin_game`:
  `reinit_game_units` only re-reads `z`; `clock_init`/`solo_init` touch no
  object.

---

## 5. What the gamelog writes for the start

`docs/DATALAYER.md` §1–§2 already covers `UnitData::log_data`,
`BuildData::log_data`, `CitiesData::log_data`, `LeaderData::log_data`. What
matters here:

- Level 0 `UNITS`: `(who, o)`, `type`, position, `angle`, `GUY`s. It shows
  **where** each starting unit stands (beside its building) and nothing of
  the order list at level 0 — `UnitData::log_data@0060b070:760` calls
  `OrderList::log_data` only at the deeper detail. So the level-0 dump
  **cannot** show the `GATHER` orders that §2.4 says are there; a `UNITS=3`
  run (the orders sub-block, if that is where `OrderList::log_data` lands —
  not verified here) would.
- Level 0 `BUILDS`: ids `2000..2005`, position, type — enough to see the
  site choice and its order (`2001` woodcutter first, then farms, then the
  library), and that `2000` is the city.
- `CITIES`: the city record, `o = 2000`, `reg`.
- `LEADERDATA`: `tribe`, `score`; `GOODS` per leader is a level-1 field
  (DATALAYER "not established"), so the §2.2 stockpile is **not** in the
  level-0 dump — `DUMP_ALL=1` at `[Start Game]` would show it, but the
  cheaper check is the `resource` console command's echo (`docs/ORACLE.md`).
- The `MAPMAKE` category at detail 6 writes a line per start slot with
  `(slot, start_x, start_y, dist)` (`Map::make@0068bc90:308–320`) — the
  way to read the map-maker's start cells without terrain.

---

## 6. Confidence, what is not established, and the checks

**High** (read directly, several functions agreeing):

- The startup order §2.1; `frame = tick = 0` in `build_game`; the
  `[Start Game]` dump sits after `build_game` and before the first
  `do_frame`; the `FRAME n` = "after n passes" convention (`check_accept`
  + `do_frame:293/315`).
- The id scheme (`Objects::clear` marks, `find_free`, `init_unit`,
  `init_build`): scout `0`, citizens `1..n`, city `2000`, buildings
  `2001..`.
- The citizen counts per `starting_town` (PE jump table), the modifiers, the
  scout/merchant/scholar extras, the citizen→building assignment rule, the
  creation at the building's centre + `come_out` + `add_gather_order(QUEUE_NEW)`,
  `add_gatherer` at setup, and no orders for the scout and the idle-placed
  citizens.
- The buildings per town size and their production order; created complete
  via `init_build` + `activate(0,1,0)`; the city via `Build::init` +
  `activate(0,0,0)`.
- `init_starting_resources`'s arithmetic and the `starting_goods` base (the
  offset derivation `0x600 − 0x3cc = 0x234`).

**Medium:**

- The **site choice** in `produce_building` is described in shape only; its
  scoring was read, not re-derived line by line. What is solid: it is the
  AI placer, it spirals `circle_*`, it draws `game_random` per candidate for
  farms and per jitter cell, and at frame 0 it activates. **Do not
  reimplement it for the harness — take the positions from the dump.**
- The type-vtable slots `+0x90`/`+0x94` on a building type ("gatherable" /
  its companion) are named by role from their use in `add_gather_order` and
  `build_units`; `vtables.txt` has no `BuildType::vftable` entry to name
  them (the base `Type::vftable +0x90` is a folded trivial body).
- The lobby value names: `starting_town` 0 = Nomad, 1 = city only, 2 =
  "Small Town" (the `check.ini` the harness run used says
  `startingtowns=Small Town`), 3 = Large Town; `starting_resources` 7 and 8
  are the two special rows (8: 99999 of everything, +12 citizens, no gather
  orders — the "infinite" row; 7: +8 citizens, food-amount for all goods),
  12 = random. Which labels those are is a lobby-list read not done here.
- `init_teams`/`init_handicaps`: skimmed for what they touch, not read.

**Not established:**

- The gather order's first frames — why the position first changes at
  `FRAME 4` rather than `FRAME 1`. The Gather reader's `process` will say;
  the candidates are named in §4.
- What `Build::activate(0, 1, 0)` vs `(0, 1, 1)` skips for the starting
  buildings (`counted = 0`): CITIES §4 lists the `counted` gates
  (`last_building_finished`, `cities_built`, the free-unit high-water
  marks); whether `gather_slots` is under `counted` was not re-read.
- Whether `OrderList::log_data` reaches the file at `UNITS=3` (it is at
  `UnitData::log_data:760`, past the level-0 fields) — one run settles it.
- `uber_size` for the scout and citizen (assumed 1 — the dump shows one
  `o` per unit).

**Behavioural checks, each cheap** (`docs/ORACLE.md` recipe):

1. **Orders at frame 0**: `[Start Game] UNITS=3` (or whatever detail
   reaches `OrderList::log_data`) with `Seed` fixed — expect units `1..5`
   each with one `GATHER` order on `2001, 2001, 2002, 2003, 2004`, the scout
   `0` with none. Also settles 6.3.
2. **The walk**: `[End Frame] UNITS=3` for frames 1–6 on the same seed —
   the citizen's `path`/`wait` per frame explains the three quiet passes.
3. **The stockpile**: `resource` in the `~` console at frame 0 against
   §2.2 for the seed — checks the `starting_goods` base and the `lo/hi`
   row read.
4. **The AI scout**: `ai off` in the console before frame 0 (the
   `StartConsole=1` path) — the scout should then stand still for the whole
   run, which pins "the AI issues it" against "scripted".
