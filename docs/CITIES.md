# Cities and buildings

How a building is placed, built, finished, garrisoned, captured and lost, and
how a city — a building with a record behind it — grows through its three
levels. The tenth mechanic, and the one the previous nine kept handing off to:
production's `construct_time`, combat's tower arrows and city clamp, attrition's
city sources, the tech tree's `City::check_upgrade`.

**Scope.** The building as an object: its footprint, its placement rules, the
construction clock and what drives it, hit points during and after
construction, the activation step and its side effects, destruction and
refunds, repair, and the damage a building takes standing in enemy land. The
city as a record: which buildings belong to it, its radius, its three levels
and the predicate that raises them, the population value, the capital. The
garrison: who may enter what, the capacity, the FIFO chain, ejection one squad
a frame, the garrison heal. Capture: the eligibility, the count inside the
radius, the hand-over, plunder, assimilation, the city heal, and what losing
the last city does.

**Not in it.** The production queue a finished building runs
(`docs/PRODUCTION.md`), the arrows a garrison feeds a tower
(`docs/COMBAT.md` §8.6), the territory a city projects (`docs/ATTRITION.md`),
the goods a city gathers and the enhancers (`docs/ECONOMY.md`), the tech tree's
`type_avail` (`docs/TECH.md`); airbases, carriers, silos and transports, which
share the garrison chain but have their own launch path (surveyed in §6.8);
the AI's site scoring; the scenario editor; Conquer the World (`ConquestCity`
is a CtW map-node record, nothing in `Build`/`City`/`Cities` touches it).

**How this was established.** Symbol names, struct layouts and field offsets
come from `game/sbl/rise.pdb`; behaviour from reading the original with Ghidra,
with those symbols applied, through the export under `~/ghidra-projects/decomp`
(`tools/ghidra/`). Five readers took one sub-area each — placement,
construction, city levels, garrisons, capture — and each read its functions in
full: `BuildTypeData::blocked_site`/`blocked_tcoord`/`blocked_location` and
the chain from `Group::action_build` down; `Build::init`/`start`/`process`/
`activate`/`close`, `Wall::init`/`start`/`do_construct`/`update_hits`/
`update_construct_time`/`process`/`activate`/`close`, `BuildData::
construct_time`, `Unit::do_build`/`do_repair`, `Object::disband`; `City::init`/
`close`/`find_buildings`/`check_upgrade`/`capture`/`assimilate`, every
`CityData::*` accessor, `Cities::init_city`/`capture_city`, `Build::
add_to_city`/`remove_from_city`/`find_city`, `LeaderData::get_radius`/
`get_city_limit`; `Unit::go_inside`/`come_out`/`do_garrison`, `Object::
insert_inside`/`remove_from_inside`/`eject_contents`, `ObjectData::
num_inside`/`count_inside`, `BuildTypeData::get_garrison_limit`, `UnitTypeData::
can_garrison`, `Build::process_ejection`, the garrison branch of `Unit::
process_healing`; `Build::check_capture`, `BuildData::check_capture_eligible`,
`Build::swap_team`, `Build::plunder`, `Leader::lost_a_city`/`lost_capital`/
`recapture_capital`/`defeat_by`. Three claims the decompiler left ambiguous
were settled in the disassembly (`llvm-objdump`): the argument `Unit::do_build`
passes to `do_construct` (§3.3), the value written to `capture_strength`
(§7.2), and the `do_garrison` helicopter/dock literals (§6.4). Nothing is
transcribed; see `docs/DECISIONS.md` entry 7.

**Confidence.** High for the arithmetic throughout — the construction clock,
the per-builder contribution, the site's growing hit points, the repair
period, the capture count, the assimilation tick, the level predicate, the
garrison limit — each a short function read whole. High for the placement
predicates (`BlockIndex` values resolved from the PDB enum) and the lifecycle
order. Medium for the `BUILD_FLAGS` letter ↔ column pairing (inferred from read
order against the DTD, corroborated by the shipped data) and for the type-side
vtable slots `vtables.txt` does not carry (named from their bodies). Medium
for `Unit::come_out`'s placement and the `find_buildings` conversion branch,
long functions read once. The per-wonder effects of `Build::activate` are
listed, not derived. `leader_flags` bit names are inferred from use.

**What it changed elsewhere.** `docs/COMBAT.md` §6 step 28 said the recapture
modifier keys on the city's "original owner"; `ObjectData::get_damage` compares
`CityData::race` — the nation the city is assimilated to — not `founder`
(§7.4). `docs/COMBAT.md` §12.1's "`VEHICLE|WAR_MACHINE`" is a conjunction
(§7.1). `docs/COMBAT.md` §12's unidentified tile class `0x30` is forest (§2.3).
`docs/PRODUCTION.md`'s open `construct_time` item closes here (§3.2).
`docs/ECONOMY.md`'s `num_buildings` includes the city building itself (§5.5).

---

## 1. The object model

### 1.1 Classes and records

A building is a `Build` (0xdc bytes) — `is-a Wall` (0x74) `is-a Object`
(0x50) `is-a SubObject` (0x28); the data halves `BuildData` ← `WallData`
(0x70) ← `ObjectData` (0x50) ← `SubObjectData` (0x1c). Buildings occupy
object indices **2000–2999** of their owner's object array
(`Objects::init_build`: `find_free(who, 2000, 3000, …)`). A city is a
building whose type `is_city()` plus a `CityData` record (0xb8 bytes) in
`cities.lists[who]`, the building's `BuildData::city` (+0x72) being the index.

Fields this mechanic reads and writes, named from `types.txt`:

| field | meaning |
| --- | --- |
| `SubObjectData +0x8 flags` | §1.2 |
| `+0x9 who`, `+0xa o` | owner, object index |
| `+0x18 ptype` | the building's `BuildType` |
| `ObjectData +0x20 myhits`, `+0x24 damage`, `+0x3b damage_frac` | `docs/COMBAT.md` §2.2 |
| `+0x28 inside_down`, `+0x3e inside_down_who` | head of the garrison chain, −1 empty (§6.1) |
| `+0x32 hold_frames`, `+0x38 healing`, `+0x3d targeted`, `+0x40 visible` | |
| `WallData +0x48 job_counter` | **construction progress** — compared with `construct_time` |
| `+0x4c job_counter_2` | the same sum, never reduced by damage; the cancel refund's numerator |
| `+0x50 constr_time` | the stored base construction time, written once at placement (§3.1) |
| `+0x54 construct_hits` | the site's current full-health figure (§3.4) |
| `+0x5c frame_started` | the frame `Wall::start` ran |
| `+0x60 build_masks` | §1.3 |
| `+0x64 helpers` | builders or repairers that contributed **this frame** |
| `BuildData +0x6c orig_type` | the type it was placed as |
| `+0x72 city`, `+0x74 city_down` | the city it belongs to; the next building in that city's chain |
| `+0x76 wonder`, `+0x78` (fort/dock/oil-well/farm slot), `+0x7a recharging`, `+0x7f founder`, `+0x80 gather_max`, `+0x82 queued`, `+0x83 max_age` | |
| `CityData +0x4 city_flags` | §1.4 |
| `+0x6 city`, `+0x8 o` | its own index; the city **building's** `o` — the head of the member chain |
| `+0xa reg`, `+0xc/+0x10 x, y` | the region of its cell; the building's position |
| `+0x14 attack_stamp`, `+0x18 raid_stamp`, `+0x20 capture_stamp`, `+0x24 assimilation_timer` | 0 at founding; the current frame on a transfer |
| `+0x28 capture_strength` | §7.2 |
| `+0x52 trade_val`, `+0x56..+0x59 granary/lumber_mill/smelter/refinery` | `docs/ECONOMY.md` |
| `+0x5d pop` | 1 at init, copied on capture; **not** the pop value (`get_pop_value` is computed from the level) and not touched by a level-up |
| `+0x5e who`, `+0x5f race`, `+0x60 founder` | owner; the nation it is **assimilated to** (−1 at init, `who` on founding); who founded it |
| `+0x68 was_capital_flags` | bit per player: was once that player's capital |
| `+0x74 vans`, `+0x90 name` | caravans; cosmetic |
| `BuildTypeData +0x2c0 build_flags` | §1.5 |
| `+0x2c8 garrison_max`, `+0x2cc base_arrows`, `+0x2d4 plunder_value`, `+0x2d8 plunder_good`, `+0x2e0 to` | per type |
| `+0x2b4 town_hits`, `+0x2b8 min_city_size` | loaded; **read by nothing in the sim** (only `backup`/`restore`) |
| `TypeData +0x8 job_time` | the record's build time; `TypeData::time(who) = job_time × 100` |
| `ObjectTypeData +0x210 hits`, `+0x218 domain`, `+0x234 x_size`, `+0x238 y_size` | |

`LeaderData` counters: `+0x3f8 city_num` (live cities), `+0x400 city_mine`
(moves with it everywhere read), `+0x408 city_mark` (high-water mark of
`cities.lists[who]`), `+0x410 lost_city_stamp`, `+0x414 lost_capital_stamp`,
`+0x418 lost_capital_timer`, `+0x41c lost_capital_modifier`, `+0x7e4 pop_cap`,
`+0x814 buildings_built`, `+0x820 cities_built`, `+0x824 cities_captured`,
`+0x828 cities_lost`, `+0x854 wonders_built`, `+0x95c pop`, `+0xe62
reg_pop[64]`, `+0x125e reg_cities[64]`, `+0x12de reg_forts[64]`, `+0x145e
reg_terr[64]`, `+0x14de reg_buildings[64][129]`, `+0x555e
num_buildings[129]` (indexed `type − BASE_BUILDTYPES`; reached as `+0x5222 +
type·2`), `+0x5660 high_buildings[129]`, `+0x5a22 num_queued[806]`, `+0x6070
last_building_finished[129]`. `Game +0x6cc world_pop`, `+0x6d0 world_cities`.

### 1.2 `SubObjectData::flags`

| bit | meaning | set / cleared |
| --- | --- | --- |
| `0x1` | in use — `SubObjectData::is_active` | `SubObject::init`; `Build::close` ends with `flags = 0` |
| `0x2` | **started** — `WallData::is_started` | `Wall::start`; `Wall::init` clears `0x2|0x4` |
| `0x4` | **active** — construction finished, `WallData::is_active` | `Wall::activate` |
| `0x10` | damaged / at zero | `Object::take_damage` (`docs/COMBAT.md` §7.2) |
| `0x20` | **is a city** | `SubObject::init`, when the type's `is_city` slot answers — the whole `VILLAGE` lineage; `Build::close` re-asserts it after `Wall::close` |

A building has three states: *placed* (`0x1`), *started* (`0x3`), *active*
(`0x7`). Every `(*this + 0x4c)` / `(*this + 0x50)` in the decompile is
`is_active` / `is_started`.

### 1.3 `WallData::build_masks`

| bit | meaning | set | read |
| --- | --- | --- | --- |
| `0x1` | marked blocking by `Unit::resolve_block` | movement | — |
| `0x4` | explicit attack order | `Build::add_attack_order` | `docs/COMBAT.md` §8.6 |
| `0x8` | launch pending (airbase/carrier/silo) | `Build::init` (`|0x88` for `can_carry(AIR)`) | `Build::process` → `Object::do_launch` |
| `0x10`, `0x20` | **under attack**, two-stage | `Object::take_damage` ors `0x30` on a hit by another player | `is_under_attack` = `& 0x20`; `Wall::process` every 32 frames (phased by `o`): `0x10` set → clear it, else clear `0x20` — so "under attack" lasts 32–64 frames after the last hit; `do_build`/`do_repair` quarter their rate on it |
| `0x40` | infinite queue | `docs/PRODUCTION.md` | |
| `0x100` | roads need re-laying | `City::regen_roads` | `Build::process` every 16 frames |
| `0x200` | was neutralized last frame | `Build::process` | toggles the city's temple/lumber/granary/market flags |
| `0x400` | had a builder last frame | `Wall::process` | UI |
| `0x800` | a worker already counted this frame | `Wall::do_construct`, `Unit::do_gather` | `Wall::process` clears every frame |
| `0x1000` | **has been activated** | `Wall::activate` | `Build::plunder` plunders only an activated building |
| `0x2000` | the AI gave up on this site | `Leader::check_orphaned_buildings` | `take_damage` disbands such a site when `damage·2 ≥ hits` |
| `0x4000` | **ejection pending** | `Object::eject_contents`, `Build::check_capture` | `process_ejection`, one squad per frame (§6.6) |
| `0x8000` | jammed this frame | `Build::do_attack` | combat |

### 1.4 `CityData::city_flags`

| bit | meaning | evidence |
| --- | --- | --- |
| `0x1` | slot is a live city | `Cities::init_city` scans for a clear bit; `City::close` clears it |
| `0x2` | "do not heal / auto-repair" | read by `Build::process`; the setter was not found |
| `0x10` | **capital** | `City::init`'s fifth argument; `find_capital` searches it |
| `0x40` | alarm (`CITY_ALARM`, PDB `CityFlag`) | `Group::action_alarm`; cleared by `come_out` when the city empties |
| `0x80` | has an active TEMPLE | `add_to_city` / `remove_from_city` / `activate`; `update_hits` reads it |
| `0x100` | **captured by an enemy, unassimilated** | `City::capture` sets; `City::assimilate` clears; the plunder protection (§8.3) |
| `0x200`, `0x400`, `0x800` | has an active GRANARY / LUMBERMILL / MARKET | same as `0x80` |
| `0x4000` | the founding capital | set with `0x10` in `City::init` |
| `0x8000` | was somebody's founding capital, since captured | `City::capture` |

`City::capture` copies the old flags `& 0xf13f` (the per-building bits are
re-derived by the `find_buildings` that follows) and then drops `0x10|0x4000`.

### 1.5 The type: `BuildType::init`, `build_flags`, lineage

`BuildType::init@00632340` reads `buildingrules.xml` column by column. The
footprint is `X_SIZE × Y_SIZE` in **tiles** (shipped: 2×2 to 8×8; cities 7×7;
Senate 4×7; Airbase 5×9); an odd side is centred on a tile centre, an even one
on a tile corner (`snap_center`). `BUILD_FLAGS` is a lower-cased string:
letter `c` sets bit `c − 'a'`, digit `d` sets bit `d − '0' + 25`. `domain`
(`+0x218`) is **derived** by `BuildType::set_domain`: `flags & 2 ? (flags & 1)
+ 1 : 0` — 0 land, 1 water, 2 both. Dock/Anchorage/Shipyard (`ebn`, `ecbn`)
and the Oil Platform (`igbe`) are **1, water**; no shipped building carries
`a` together with `b`, so domain 2 is unused — and `a` alone, which 114 of
the 130 rows carry, means nothing to this mechanic. (A first reading had the
docks at 2; `set_domain` says otherwise, and the placement code special-cases
docks by `is_dock`, not by domain.) `BuildType::init_final_flags` forces the flat bit (`3`)
on `FARM`, `OILWELL`, `OILPLATFORM`.

| bit | letter | tested where | meaning |
| --- | --- | --- | --- |
| `0x1` | `a` | `set_domain` | with `b`: domain 2 (unused); alone, on almost everything |
| `0x2` | `b` | `set_domain` | has a water domain |
| `0x4` | `c` | `docs/TECH.md` | upgrades in place |
| `0x10` | `e` | `blocked_tcoord`, `blocked_location`, `get_town`, `increment_stats`, `is_unassimilated`, `capture_city` | **does not need a city**: skips the city-radius test and `get_town`; **counted in `num_buildings` even outside one**; not converted on capture. Cities, every military building, docks, towers, forts, airbase, woodcutter, mine, oil well, houses carry it; granary, library, market, temple, senate, university, farm, smelter do not |
| `0x20` | `f` | `blocked_location` | exempt from the friendly-territory test (no shipped building) |
| `0x40` | `g` | `is_gather_type` (type slot `+0x90`) | gatherer: farm, woodcutter, mine, university, oil well/platform |
| `0x200` | `j` | `blocked_location`, `find_city_at` | **one per city** (granary, library, market, temple, senate, university, smelter…) |
| `0x400` | `k` | `blocked_tcoord` | always `BLOCKED_NEED_WALL` (unused) |
| `0x800` | `l` | `get_town`, `blocked_location` | with `e` clear: must be inside a `TOWN`-or-better (unused in the shipped data) |
| `0x2000` | `n` | `is_civilian`, `validate_build`, `capture_city` | not civilian — military, city, dock; **exempt from conversion on capture** |
| `0x4000000` | `1` | — | **derived**: in an upgrade line, both ends (`Types::init`) |
| `0x8000000` | `2` | `Build::init` | **derived**: a technology is researched here (`TechType::set_research`, the whole `WHERE` lineage) — and one of its consequences is the queue depth below |
| `0x10000000` | `3` | `is_flat` (type slot `+0x94`) | flat gatherer — **derived** (`finalize_init_all`) |
| `0x20000000` | `4` | — | **derived**: a craft is cast here (`SpellType::init`); only the Small City |
| `0x40000000` | `5` | `is_military_trainer` | **derived**: `UnitType::init`'s tail, read on the `FROM` root |
| `0x80000000` | `6` | `is_training_building` | **derived**: trains anything, read on the root |

**The last six are derived, not written** — no shipped `BUILD_FLAGS` string
contains a digit at all, so reading the column alone leaves every one of them
clear. The derivations and their oracle are in `docs/DATALAYER.md`, "The
derived words no column carries" (129 of 129 buildings' `build_flags` equal
the program's own).

**The build queue's capacity is three-way, not two** (`Build::init@00629740`,
corrected 2026-08-25): a **military trainer or any training building holds
20**, a **research building 10**, and **everything else 2** — the first
reading had the last as twenty. Both trainer predicates read their flag on
the root of the `FROM` chain, so a Castle is a trainer because the Fort is.

Lineage: `is(t, 0)` is `ObjectTypeData::is`, equality-or-`from`-chain
(`docs/TECH.md`). So `BuildTypeData::is_city` = `is(VILLAGE, 0)` admits
`VILLAGE` (0x19e, "Small City"), `TOWN` (0x19f, "Large City"), `METROPOLIS`
(0x1a0, "Major City") and `FORBIDDENCITY` (0x213, `from` Small City);
`is_fort` = `is(FORTX, 0)` admits Fort/Castle/Fortress/Redoubt and the Red
Fort; `is_tower` = `is(TOWER, 0)` Tower/Keep/Stockade; `is_dock` = `is(DOCK,
0)` Dock/Anchorage/Shipyard. `is_defensive` = tower | fort | `AIRBASE` |
`attack != 0`. `BuildTypeData::get_city_level`: `VILLAGE` → 1, `TOWN` → 2,
`METROPOLIS`/`FORBIDDENCITY` → 3; `CityData::upgrades_to` = −1 at level 3,
else `(t != VILLAGE) + TOWN`. The three city levels are **three building
types**, and a level-up is a `set_type` of the city building (§5.3).

Type-side vtable slots (`BuildType::vftable{for Type}`, which `vtables.txt`
predates; named from their bodies and, for `+0x64`/`+0xfc`/`+0x90`/`+0x94`,
read out of the PE at the address `rise_z.map` gives): `+0x1c is_wonder_type`,
`+0x60 is(t, strict)`, `+0x64 is_city`, `+0x6c time`, `+0x78 get_cost`,
`+0x84 can_pay_cost`, `+0x90 is_gather_type`, `+0x94 is_flat`, `+0xd4
pay_cost`, `+0xe4 basic_type`, `+0xfc is_fort`, `+0x108 is_dock`. Object-side
(`Build::vftable`, `vtables.txt` 28308): `+0x8 is_valid_unit` (0 on a Build),
`+0x18 is_unit` (0), `+0x1c is_wallbuild` (1), `+0x20 is_build` (1), `+0x2c
is_wonder`, `+0x3c`/`+0xac` accessors returning the data view, `+0x4c
is_active`, `+0x50 is_started`, `+0x94 swap_team`, `+0xb8 is`, `+0xbc
is_on_map` (always 1 for a building), `+0xec get_capture_value` (folded
`return 1`), `+0x11c hits`, `+0x150 close`, `+0x158 die`, `+0x15c
update_hits`, `+0x168 repair_damage`, `+0x16c take_damage`, `+0x178
get_garrison_limit`, `+0x188 is_under_attack`, `+0x18c construct_time`,
`+0x1a0 init`, `+0x1a4 start`, `+0x1a8 activate`, `+0x1b0 finished`.

---

## 2. Placement

### 2.1 The chain

```
CommandPackage::process_build → Group::action_build            (the player's order)
   → GroupData::validate_build → snap_center → blocked_site      (refuse + feedback)
   → city-count limit (cities only, §2.7)
   → type_avail == 4, snap_center, blocked_site again (per stacked site)
   → can_pay_cost / pay_cost (the price is charged here, docs/COSTS.md)
   → Objects::init_build → find_free(2000..3000) → Build::init    (§3.1)
   → action_swarm_around(…, BUILD_AT)                             (the peasants go)
peasant arrives → Unit::do_build → Wall::do_construct
   → not yet started: blocked_site once more; ok → Build::start   (§3.3)
   → progress; done → Build::activate                             (§4)
```

The AI (`Leader::found_cities`, `produce_city`, `produce_building`) reaches
`blocked_site` and `init_build` by its own road with the same rules.
`blocked_location` and `blocked_tcoord` are called only from `blocked_site`;
`blocked_town(wx, wy, who)` is `blocked_site(wx·0x300, wy·0x300, who, −1, 0)`
for a cell coordinate. Both sim calls pass `exclude_o = −1`.

### 2.2 Geometry — `snap_center`, `tile_corner`

`snap_center(x, y, who)`: an oil well/platform snaps to the oil cell under or
adjacent (the eight `move_x/move_y` neighbours, seen by `who`); a dock, or a
mine/woodcutter with a player, tries the 80 `move_x/move_y` tile offsets for
the first with `blocked_site == 0` when the clicked tile fails; then `x' =
tile(x)·192 + (x_size odd ? 96 : 0)`, same for `y`. `tile_corner`: `tx =
tile(x) − x_size/2`, so the footprint is tiles `[tx, tx+x_size) × [ty,
ty+y_size)`. `corner_tile` inverts it: `x = (x_size + 2·tx)·96`.

### 2.3 The tile layer — `TData.mask` and `WData.flags`

One `ushort` per tile (`World +0x138`) and one `WData` per cell (`+0x134`,
28 bytes). Bits this mechanic reads, as the placement code names them:

`TData.mask`: `(& 0x3) == 3` a building footprint, `== 2` mountain; `(& 0x30)
== 0x10` road, `== 0x20` ocean, `== 0x30` forest, `== 0` plain land; `0x40`
a second building placed on the tile; `0x80` a building placed (not yet
started) here; `0x100` **inside some city's radius** (`Wall::mask_city`);
`0x200` treated as a building (unnamed); `0x800` river; `0x2000` bad path;
`0x4000` blocked. `docs/COMBAT.md` §12's "`0x30` unknown" is forest.

`WData.flags`: `0x8` rock, `0x10` mountain, `0x100` coastal / not-ocean,
`0x800` oil, `0x4000` a building's centre cell (`mask_me`). `WData +0xf who`
is the territory owner (`docs/ATTRITION.md`); `+0x4 region`; `+0x14 was_seen`.

### 2.4 `blocked_site` — the per-site driver

`blocked_site(x, y, who, exclude_o, *gather_out) → BlockIndex`:

```
(tx, ty) = tile_corner(x, y)
if is_city and who ≥ 0: for each active leader i == who or allied:
    if leaders[who].leader_flags & 4 (human) and frame − 75 < leaders[i].lost_city_stamp and not editor:
        return BLOCKED_LOST_CITY (0x2d)                     # a 75-frame grace after you or an ally lost a city
first = 0; unseen = 0
for each footprint tile: r = blocked_tcoord(u, v, who, exclude_o)
    OFF_MAP (0x22) → return it
    r != 0 and (first == 0 or r == TERRITORY (0x17)) → first = r     # TERRITORY overrides an earlier reason
    who ≥ 0 and the tile is not was_seen by who → unseen++
first == BLOCKED_SEEN (0x24) → return it; unseen > area/2 and not a dock → 0x24
first == WATER (0xe) → return it
r = blocked_location(x, y, tx, ty, who, exclude_o, gather_out)
return r != 0 ? r : first
```

So the site-level verdict wins; the tile-level one is used only when the site
is otherwise clean. (The scenario editor relaxes a fixed set of reasons; not
modelled.) `gather_out` is written by `blocked_location` alone and only on
its last path — §2.6.7; `Sim::blocked_site_slots` is the form that hands it
back, and `Sim::blocked_site` is that form's first half.

### 2.5 `blocked_tcoord` — one tile

Every reason needs the half-cell `was_seen` by `who` (else `BLOCKED_SEEN`,
0x24); ruins, building and rare additionally need `was_really_seen` (else
`BLOCKED_UNSEEN`, 0x23). The enum names read inverted against their use; the
numbers are what is returned.

```
off the tile map → OFF_MAP 0x22
W = cell(u,v); T = tdata[u,v]
W.region < 0 → RUINS 4
(T & 3) == 3 → BUILDING 1
not a dock:
    domain == 1: owner cannot transport and not editor → CANT_TRANSPORT 0x2e;  (T & 0x30) != 0x20 → LAND 0xf
    domain == 0 and (T & 0x30) == 0x20 → WATER 0xe
T & 0x4000: (T & 0x30) == 0x30 → FOREST 6;  (T & 3) == 2 or W.flags & 0x10 → MOUNTAIN 2;  else RARE 7
T & 0x200 → BUILDING 1
not an oil type: W.flags & 8 → ROCK 3;   oil type: not W.flags & 0x800 → NO_OIL 5
T & 0x80 and find_building_placed_at(u, v, who, exclude_o) ≥ 0 → BUILDING 1     # a placed, unstarted building already blocks
not (build_flags & 0x10) and not (T & 0x100) → OUTSIDE_RADIUS 0x1f              # must be inside a city mask
(is_city or is_fort) and who ≥ 0 and not Lakota and not (Dutch fort with DUTCH_FORT_PLACEMENT) and W.who < 0:   # unowned ground
    reg = tregion(u, v)
    if reg_cities[reg] or reg_forts[reg] → TERRITORY (fort: NEUTRAL_TERRITORY 0x1a, city: 0x17)
    any unbuilt city/fort of who (not exclude_o) in reg → the same
    # else: allowed — the colonisation foothold
build_flags & 0x400 → NEED_WALL 0x2b
T & 0x800 → RIVER 0x2c
flat gather type and LandData::get_amount(tile, good) == 0 → NO_RESOURCES 9
0
```

**A city or fort may stand on unowned ground only as the player's first city
or fort (built or placed) in that region** — the foothold. Every later one
must be inside friendly territory (§2.6.1). The Lakota skip the test;
`DUTCH_FORT_PLACEMENT` ships 0.

### 2.6 `blocked_location` — the site

`cx, cy` = the cell of `(x, y)`; `reg = tregion(tile(x), tile(y))`. The whole
first half (§2.6.1–2.6.4) runs only with a player.

**2.6.1 Territory.**

```
not a city or fort:
    not (build_flags & 0x20) and not a dock and not editor:
        r = non_friendly_territory(footprint)           # land tiles only
        r ≥ 3 → ENEMY_TERRITORY 0x18;  r == 2 → NEUTRAL_TERRITORY 0x1a;  r == 1 and not Lakota → 0x1a
city or fort:
    in_enemy_territory(footprint) → city_num != 0 ? ENEMY_TERRITORY 0x18 : ENEMY_TERRITORY2 0x19
    reg_forts[reg] == 0 and reg_cities[reg] == 0 and city_num != 0 and FIRST_CITY_NEAR_COAST != 0:
        not has_preq(COLONIZE_BONUS 0x2af) → COLONIZE 0x1c
        no ocean cell of a real sea (region size > 24) within circle_radius[FIRST_CITY_NEAR_COAST / 4] cells → PORT_CITY 0x1d / PORT_FORT 0x1e
```

`non_friendly_territory`: per land tile, owner friendly → skip; `< −1` → 2;
unowned → ≥ 1; foreign → **3** if at war or the same player or both-way
allied (the odd cases), else 2. So every land tile under a non-city building
must be friendly territory, the Lakota may use unowned ground, docks are
exempt. `in_enemy_territory`: any tile owned by a non-friendly player where
(the placer already has a city anywhere) or (the placer has a city or fort in
that region) or (that owner really sees the tile) → 1. `check_enemy_adjacent`
is dead to placement (it is the dock's territory probe for the building-side
attrition, §9.5).

**`COLONIZE_BONUS` is a technology's, not a nation's** (2026-09-02, run63).
`has_preq(0x2af)` reads the **fourth** of `rules.xml`'s 122 `TECHBONUSES`
(`0x2ac` is the first, `docs/TECH.md`'s type map), whose one `PREQ` is
`preq0="Coinage"` — "Can colonize new continents" in its own `DESC`. So a
leader may not put its first city or fort in a region until it has Coinage,
and until then every cell of an unsettled region scores **zero** in
`compute_site_stats` (`docs/AI.md` §2.13 step 2 zeroes the base on a refused
`blocked_site(TOWN, …)`). `crates/sim` carried it as a `Nation` flag nothing
ever set, which made the refusal permanent; it is `Roles::colonize_preq` now,
loaded the way `TRANSPORT_BONUS`'s prerequisite already was
(`docs/TRANSPORT.md` §4). East Indies' AI takes its Coinage job on frame
5177, so no capture before run63's window reached the difference — and the
frame it moved is East Indies' word, 5592 → 5669 (`docs/SCOUT.md` §11.1).

**2.6.2 City spacing** (`is_city`): for every active leader `i`:

```
spacing = CITY_SPACING (24)
regions[reg].size × 9/10 ≤ leaders[who].reg_terr[reg] → spacing −= RELAX_CITY_SPACING (6)   # the placer holds ≥ 90 % of the region
unbuilt cities of i: others' (or the excluded own) must be started; same region and vector_dist(tiles) ≤ spacing → CITY_DISTANCE 0x14
i == who or leaders[i].reg_cities[reg]: built cities of i, same region, vector_dist ≤ spacing → 0x14
```

A city needs a *greater* distance than the spacing (`≤` blocks). The
distance is `vector_dist` over **tile** differences (explicit in the fort loop;
register-hidden in the city loop). Own unstarted cities count; others' only
once started.

**Confirmed in a logged run (2026-08-20)**, the check §15 named, and on two
bearings. The reference was the capital, a built Small City on tile
`(32, 156)`; the placer had `CITY_SPACING` at its base 24 (one city, nowhere
near 90 % of the region). Each test tile was probed **twice**: first with a
tower site, then — the tower removed — with a city site on the same tile, one
builder ordered onto each (`docs/ORACLE.md`, "A scripted placement test").

| tile | `vector_dist` | tower | city |
| --- | --- | --- | --- |
| `(56, 156)` | 24 | starts, `flags 1 → 3` | **disbanded** |
| `(57, 156)` | 25 | — | starts, `flags 33 → 35` |
| `(32, 180)` | 24 | starts, `flags 1 → 3` | **disbanded** |
| `(32, 181)` | 25 | — | starts, `flags 33 → 35` |

The tower is the control that makes this decisive. A tower is not a city, so
it is subject to §2.6.1's *stricter* territory test — every land tile under it
must be friendly — and to the same terrain verdicts. A tower that starts on a
tile therefore proves that tile is the placer's own territory, is buildable
ground, and is reachable by the builder, which removes every candidate reason
for the city's refusal except the spacing rule. (It removes them in the right
order, too: §2.4 returns `blocked_location`'s verdict *over* the tile-level
one, so a city that is both outside friendly territory and too close reports
`CITY_DISTANCE` — meaning the ladder's boundary is the spacing boundary and
not a territory edge.)

So the boundary sits exactly between 24 and 25: `CITY_SPACING` is 24 and the
compare is `≤`. The reading is confirmed, including that the blocked distance
is the constant itself rather than one less.

**2.6.3 Fort spacing** (`is_fort`): for every other active leader `i`: `s =
FORT_SPACING` (12) for self/ally, `FORT_TO_ENEMY_CITY_SPACING` (32) otherwise;
their started unbuilt cities and (if `reg_cities[reg]`) built cities in the
region within `s` → `FORT_CITY_DISTANCE 0x16`. Then for every active leader
including `who`: unbuilt forts (others' started; own all but `exclude_o`) and
built forts within `FORT_SPACING` → `FORT_DISTANCE 0x15` (own) / `0x16` — the built forts
**without a region test** (the cities above have one). **Own cities are not
checked against a new fort.**

**2.6.4 Must belong to a city** (`build_flags & 0x10` clear):

```
town = get_town(tile(x), tile(y), who)           # §5.2; −1 if any footprint tile lacks 0x100 or no city covers the centre
town < 0 → (build_flags & 0x800) ? OUTSIDE_TOWN 0x20 : OUTSIDE_RADIUS 0x1f
that city's queue holds DISBAND (0x29a) or RAZE (0x29b) → RAZING_TOWN 0x21
build_flags & 0x200 and count_buildings(city, type) != 0:            # one per city
    another active city of who within min(get_radius(C), 64) with none → ONE_OTHER 0x28;  else ONE 0x27
FARM and count_buildings(city, FARM) ≥ get_farm_limit(city) → FARM 0x29
a wonder (0x20e..0x21e) not the Red Fort: num_wonders(city, 1) > (Egyptians ? 1 : 0) → WONDER 0x2a
```

`num_wonders(city, 1)` counts finished wonders on the chain, excluding the
Red Fort and the city building itself (the Forbidden City). One wonder per
city, two for Egyptians. `get_farm_limit` = `LeaderData::get_farm_limit +
(level − 1) × FARMS_PER_CITY_LEVEL`, the leader part `FARMS_PER_CITY_BASE` (5)
or `EGYPTIAN_FARMS_PER_CITY_BASE` (7) + `TAJ_FARMS` + `KREMLIN_FARMS` +
`OLIVE_FARMS`.

**2.6.5 Water and land** (with or without a player):

```
not a dock: domain == 1 and count_water < area → LAND 0xf;   domain == 0 and count_water != 0 → WATER 0xe
dock: w = count_water
    w < 3·area/4: w != 0 and check_land_adjacent != 0 and not editor → DOCKTERR 0x12; else DOCKWATER 0x11
    check_land_adjacent == 1 and not editor → DOCKTERR 0x12;  == 2 → DOCKLAND 0x10
```

`count_water` counts footprint tiles with `(T & 0x30) == 0x20`.
`check_land_adjacent` scans the one-tile ring (minus corners) for a tile that
is not ocean and not blocked: none → 2; one whose cell owner is friendly (or
`who < 0`, or unowned and Lakota) → 0; land found but all foreign/unowned →
1. **A dock needs three quarters of its footprint on water and an adjacent
land tile inside friendly territory** — the only territory rule a dock obeys.

**2.6.6 Clear ground around cities and forts**: `ring = coastal ? 1 : 2`;
any mountain (`(T & 3) == 2`) in the ring → `MOUNTAIN 2`; any forest (`(T &
0x30) == 0x30`) → `FOREST 6`. Corners excluded.

**2.6.7 Oil and gather slots** (`blocked_location@006375b0:679–716`, the last
thing the function does): an oil type must be `was_seen`; a non-flat gather
type (woodcutter, mine, university) surveys what its site would gather and is
refused when there is nothing there — `calc_gather == 0` → `NO_MOUNTAIN 0xc`
for a mine, `NO_FOREST 0xa` for a camp, `NO_RESOURCES 9` for anything else;
`< 0` → `FOREST_TAKEN 0xb` for a camp, `MOUNTAIN_TAKEN 0xd` otherwise. The
`was_seen` half is visibility and stays unmodelled (§11); the gather half is
`Sim::gather_verdict`.

**The count is `blocked_site`'s out-parameter, and it is the whole reason
the parameter exists.** `*param_7 = max(count, 0)` is written here and
nowhere else in the function, so a non-gather type leaves the caller's own
zero standing. The count is `calc_gather`'s, the same number
`max_gatherers` reads (`docs/ECONOMY.md`, "How many citizens"), taken from
the *survey* when `exclude_o` or `who` is negative and from the named
object's own `MiningList` otherwise. The reader that matters is
`Leader::produce_building`, which scores a woodcutter's camp site by the
**cube** of it and refuses a site under three (`docs/AI.md` §4.4); a
one-tile ring of forest tiles, which is what this crate counted until
2026-09-01, is zero at every site a camp can actually stand on, so the AI
built no camp at all after frame 0.

The player-less form (`who < 0`) runs the survey in the original and skips
it here — the walk's cell-owner test wants a player and there is no oracle
for `who = −1`. No caller in this crate passes it.

Never returned by anything read: `BLOCKED_NEARBY 8`, `CITY_RADIUS 0x13`,
`PEACEFUL_TERRITORY 0x1b`, `ROAD 0x25`, `NEED_ROAD 0x26`.

### 2.7 How many cities — `LeaderData::get_city_limit`

```
civic = epoch[1]                                     # the Civic level, docs/TECH.md
Bantu and civic != 0 → civic += BANTU_CITY_LIMIT (1)
limit = civic + 1 (+ PYRAMIDS_CITY_LIMIT (1) with the Pyramids)
total = num_queued[VILLAGE] (+ queued types whose from-line is VILLAGE) + city_mine − (Forbidden City ? 1 : 0)
```

`Group::action_build` refuses a city (not the Forbidden City) when `limit ≤
total` with `S_CITY_LIMIT_REACHED`. Nothing in `blocked_*` tests it.

---

## 3. Construction

### 3.1 Placement: `Objects::init_build`, `Build::init`, `Wall::init`

`Objects::init_build(who, type, x, y, restore)`: a free slot in 2000–2999,
`snap_center`, then `Build::init(who, type, o, x, y, restore)`; `restore` is
"re-creating, not building" (set by `Wall::swap_team`).

`Build::init`, in order: `founder = who`, `wonder = −1`, `max_age` (the
owner's age), `orig_type = type`; `Wall::init`; `clear_gather`; not restoring
→ `buildings_built++`; `attack != 0` → `defense++` (AI); `city = −1`,
`city_down = −1`, stance defaults; **city** (`flags & 0x20`) → push `(o, who)`
on `unbuilt_cities[who]` (and an AI's whole starting army walks to its first
site); **non-city** → `find_city` (§5.2) — **membership is decided at
placement**; fort → `unbuilt_forts`; wonder (0x20e–0x21e) → `unbuilt_wonders`;
`BuildQueue::init` (20 / 10 / 2, `docs/PRODUCTION.md`); FARM → `Farms::add`;
gather type → `find_gather_tiles` or `gather_max = max_gatherers`;
`can_carry(AIR)` → `build_masks |= 0x88`. **Nothing is paid here** — the
price went at queue time; placement is free and the refund is §9.2.

`Wall::init`: `Object::init`; `flags &= ~6`; `build_masks = 0`; `job_counter
= job_counter_2 = 0`; `helpers = 0`; `frame_started = −1`; `start_me(1)` —
for each footprint tile, `T |= 0x80` if clear else `0x40` ("one placed here /
two"); **`update_construct_time`** (§3.2); **`update_hits(0)`** (§3.4);
`update_los`; terraform (renderer); `Unit::update_z` on land units within
`0x51` tiles (city) or `0x19` (other) — renderer.

**A placed-but-unstarted building already blocks other placements on its
tiles** (`blocked_tcoord`'s `0x80` test) but not movement.

### 3.2 The construction clock

**The stored base — `Wall::update_construct_time@0063d560`**, called once
from `Wall::init`:

```
t = job_time × 100                                   # TypeData::time(who)
Maya and not a wonder:                 t = t × 100 / (MAYA_BUILDING_SPEED + 100)
has_preq(BUILDINGS_CREATED_FASTER 0x313): t = t × 3 >> 2
has_wonder(VERSAILLES):                t = t × 100 / (VERSAILLES_BUILDING_SPEED + 100)
rare Tobacco (bit 13):                 t = t × 100 / (TOBACCO_BUILDING_SPEED + 100)
British and is(AIRDEFENSE 0x20b):      t = t × 100 / (BRITISH_AA_SPEED + 100)
Dutch, is_fort, not a wonder:          t = t × 100 / (DUTCH_FORT_SPEED + 100)
Romans, is_fort, not a wonder:         t = t × 100 / (ROMAN_FORT_SPEED + 100)
city_num == 0:                         t = CAPITAL_BUILD_TIME × t / 100       # 300 %: a nomad's first city
n = get_building_speed_upgrade()       # count of has_preq(BUILDINGS_FASTER_1..3), 0..3
t = (10 − n) × t / 10
constr_time = t
```

**This is evaluated at placement — and again on every `Leader::calc_wall_stats`.**
*(Second reading; the first said "once, at placement". `Leader::process`
answers the `leader_flags & 0x8000000` dirty flag with
`calc_wall_stats@006cf7c0`, which calls `update_construct_time` on every
alive building of the player that is **not yet active**, then
`update_hits` on all. The flag is set by `Wall::activate`,
`Build::close`, `check_upgrade`, `gain_tech` and more — and
`Leader::gather@006ce280`, which raises `0xc000000` (the unit-stats flag
*and* this one) whenever the rare mask moves.)* So a speed tech, a
wonder, a **rare** or the nomad's first city **does** reach every site
still under construction on the next frame; only the value between two
dirty flags is frozen. `crates/sim` keeps the flag per player
(`Sim::wall_stats_dirty`) and re-bakes in `tick` before any object runs.

**The Tobacco row is diff-backed** (2026-09-07, item 261): Great Lakes'
Tower `1/2017` reads `constr_time` **100000** through 6751 and **90909**
from **6752**, the frame the AI's `rare_owned` gains bit
`19 − BASE_RARE = 13` — `× 100 / 110` to the digit, so arithmetic, bit
and re-bake are confirmed at once
(`run76_and_run79_date_the_tobacco_rare_on_the_tower_s_clock`), and the
harness compares `constr_time` and `job_counter` on every capture now.
The mask's bytes begin at `LeaderData +0x6da4`, so `+0x6da5` bit 5 is
Tobacco and bit 7 of it Furs (`docs/VISION.md` §1).

**Confirmed in a logged run (2026-08-20)**, and the second reading was
right. A tower site with no techs held `constr_time = 100000`; under
`cheat tech all on` (`docs/ORACLE.md`) a fresh one read **70000**, exactly
`(10 − 3) × 100000 / 10`, so `get_building_speed_upgrade()` counted all
three `BUILDINGS_FASTER`; and `cheat tech all off` took that untouched,
unstarted site **70000 → 100000** within a few frames, which a value
frozen at placement cannot do. The run pins the re-bake's *scope* too —
the active tower kept 100000 across both and the finished barracks 42000
— so `calc_wall_stats` touches only not-yet-active buildings, a site
**nobody has started** included.

**The per-call modifiers — `BuildData::construct_time(flag)@0062d5c0`**,
vtable `+0x18c`, what `job_counter` is compared against. `flag != 0` returns
the base. Else, in order:

```
t = constr_time
(a) a wonder, Americans, get_wonders() == 0 and wonders_built == 0, not SUPERCOLLIDER/SPACEPROGRAM:
        if no other alive player has an alive, not-yet-active building of this type: t = 1     # the free first wonder
(b) not a wonder, has_wonder(HANGINGGARDENS) with bit 1 clear, city ≥ 0, and the owner's Gardens stand in this city:
        t = (100 − HANGING_GARDENS_BUILD_TIME) × t / 100
(c) has_general(THEPRESIDENT) ≥ 0:     t = t × 100 / (THEPRESIDENT_BUILDING_SPEED + 100)
(d) Iroquois, is(SENATE), IROQUOIS_QUICK_SENATE != 0, high_buildings[SENATE] == 0:  t = 0      # the first senate is instant
return max(t, 1)
```

`docs/PRODUCTION.md`'s open item on this function closes here: it is the
foundation clock, not the queue path; the two share only `TypeData::time`.

### 3.3 Progress — `Unit::do_build` → `Wall::do_construct`

The builder's per-frame step (`Unit::do_build@005eebf0`): target dead → kill
the order; target already active → kill the order, and a builder with no
further orders at a same-owner gather building (not oil platform/university)
gets a gather order there, else `build_done` looks for the next site within
`UNIT_GATHER_RESPOND_RANGE × 192`; not adjacent (`adjacent_to`; a unit standing
on the footprint of a non-FARM site is *not* adjacent) → move and return;
decoy → nothing; **`amount = ACCEL_CONSTRUCT`**, quartered when the building
`is_under_attack` (`build_masks & 0x20`) unless Koreans with
`KOREAN_BUILD_UNDER_FIRE`; `do_construct(amount)`. (That the argument is
`ACCEL_CONSTRUCT` and not `who`, as the decompiler printed it, is settled by
the listing: `push [ebp−8]` — the amount after the `/4` — precedes the call.)

The building's side, `Wall::do_construct(amount)@006434d0`:

```
ai_speed > 1 → amount *= ai_speed                               # a debug global, 1 in play
not started:
    r = blocked_site(own centre, who, own o)
    ok = r ∈ {0, ONE 0x27, ONE_OTHER 0x28, FARM 0x29, NEED_WALL 0x2b}
         or (r == WONDER 0x2a and city ≥ 0 and num_wonders(city, 1) ≤ 1 + Egyptians)
    ok → Build::start (§3.5);  else Object::disband(1) and S_CONSTRUCTION_BLOCKED; return 0
active → return 0
share = amount / (helpers + 1);  if build_masks & 0x800 == 0: recharging++, build_masks |= 0x800
helpers++;  share = max(share, 1)
job_counter_2 += share;  job_counter += share
job_counter ≥ construct_time(0) → activate(0, 1, 1) (§4); return 1
return 0
```

Three consequences. **The first builder to arrive starts the site**, and the
legality is re-checked then — the one-per-city, farm, wall and wonder
verdicts are tolerated (the site itself is now counted), everything else
sends the site back with a full refund. **Extra builders are harmonic**:
`helpers` is the number that already contributed this frame, reset by the
building's own `Wall::process`, and `Objects::process_all` runs every unit
before any building, so `n` builders advance the site `accel·(1 + 1/2 + … +
1/n)` a frame, each term floored, never below 1 — two are 1.5×, four ~2.08×,
eight ~2.72×. **Frames to finish with one builder** = `⌈construct_time /
ACCEL_CONSTRUCT⌉`; `ACCEL_CONSTRUCT` loads `get_fraction(…, 100)`, ships
`1/1` = 100, so an unmodified type takes exactly `job_time` frames.

**Confirmed in a logged run (2026-08-20)**, the check §15 named. A barracks
site (`constr_time = 42000`) and a tower site (`100000`) were placed with
`cheat add NEW`, and `BUILDS=6` under `[End Frame]` logged `job_counter`
every frame (`docs/ORACLE.md`). One builder advanced the site by **exactly
100 a frame**, unbroken for the whole run — so `ACCEL_CONSTRUCT` is 100 and
an unmodified type does take `job_time` frames. A second builder joining the
tower changed the increment to **exactly 150**, and it stayed there: 53
frames at 100, then 194 at 150.

That 150 is worth more than the rate. It settles the harmonic rule *and* the
process order together, because the two readings differ: if `Objects::
process_all` ran buildings before units, `helpers` would be reset between the
two builders' `do_build` calls and each would contribute a full `amount`, for
200 a frame. 150 is only reachable if both builders run against the same
un-reset `helpers` — **every unit does process before any building**, and the
second builder's share is `100 / 2` floored to 50.

Three smaller things the same run pinned:

- **`helpers` always reads 0 in the dump.** `Wall::process` resets it at the
  end of the frame and the log is written after that, so the field can never
  show the count. The *increment* is the only way to read the builder number
  out of a log — worth knowing before designing a diff around it.
- **`recharging` counts up one per frame while building** (0, 1, 2, …),
  which is the `build_masks & 0x800` step, and confirms it is set once per
  frame rather than once per builder.
- **The object's `flags` are a state machine**: `1` placed but not started,
  `3` started and under construction, `7` active. On completion
  `job_counter` resets to 0 and `flags` goes 3 → 7 in one frame — the
  `activate(0, 1, 1)` of §4.

### 3.4 Hit points during construction — `Wall::update_hits@0063f0d0`

Called from `Wall::init`, from `activate`, and **every frame while not
active** from `Wall::inc_time` — which `Objects::inc_time` drives from
`Game::do_frame` right after `process_all`, so the order within a frame is:
all units' `do_build`, all buildings' `Wall::process`, then every site's hit
points refreshed from the new `job_counter`. The sim reads the result through
`BuildData::hits(0)` = `construct_hits`, `hits(1)` = `myhits`.

```
h = type.hits
Maya:                                              h = (MAYA_BUILDING_HP + 100) × h / 100
Romans, (is_fort or TOWER), not a wonder:          h = (ROMAN_FORT_HP + 100) × h / 100
n = get_building_hp_upgrade() (0..3):              h = (BUILDING_HP_UPGRADE × n + 100) × h / 100
has_wonder(TAJMAHAL):                              h = (TAJ_BUILDING_HP + 100) × h / 100
has_wonder(REDFORT), is_fort, not a wonder:        h = (RED_FORT_FORT_HPS + 100) × h / 100
MARKET, Nubians, NUBIAN_HIT_POINTS != 0:           h = (NUBIAN_HIT_POINTS + 100) × h / 100
not a city: active, in a city, not is_fort/TOWER/LOOKOUT:
                                                   h += SENATE_HP_BONUS × (city level − 1) × h / 100
a city: active, in a city, city_flags & 0x80 (temple):
        b = TEMPLE_UPGRADE_HP[temple level − 1] (Tikal: (TIKAL_TEMPLE_HP + 100) × b + 99) / 100)
                                                   h = (b + 100) × h / 100
myhits = h
not active and job_counter < construct_time(0):
    a = max(1, job_counter >> 5);  b = max(1, construct_time(0) >> 5)
    not a wonder:  h = max(1, a × h / b)
    a wonder:      h = (h + 1) / 2 + max(1, (h / 2) × a / b)
construct_hits = h
h ≤ damage and a garrison and not can_carry(AIR) → eject_contents(1, −1, 0, 1)
```

An ordinary site's health grows **linearly from ~0 to full**; a **wonder site
starts at half**. `Object::take_damage` (`docs/COMBAT.md` §7.2) takes `whole ×
50` off `job_counter` per hit — a site loses progress — quadruples a
building's damage to a site, and kills the site when `damage ≥ hits(0)`. The
Senate bonus: **every non-defensive member of a Large City has 35 % more hit
points, of a Major City 70 %** (`SENATE_HP_BONUS` 35); `BUILDING_HP_UPGRADE`
is 10 % per level. `town_hits` in the data is dead.

### 3.5 Start — `Wall::start`, `Build::start`

`flags |= 2`; `frame_started = frame`; **`kill_competing_buildings`** — every
*other* not-started building placed on a double-placed footprint tile is
`Object::disband(…, 1)` (full refund), a started one stops the sweep; **`mask_me
(1, REGEN_FORCE)`** (§3.6); each footprint cell's `was_seen |= 1 << who`;
`mark_behind_tiles`. `Build::start` adds only the wonder-started notice.
**Starting is the moment the site is committed**: the footprint is reserved,
the terrain marked, and for a city the radius mask is laid. Before that the
building is a ghost a rival's start can delete.

### 3.6 What a building marks — `Wall::mask_me` → `BuildType::mask_me`

With `radius = CityData::get_radius(city)` for a city that has its record,
`LeaderData::get_radius(who, type)` for a city building without one, 0 for
anything else: `W.flags |= 0x4000` on the centre cell; every footprint tile
clears `0x40|0x80` and sets `T |= 3`; tiles whose per-type collision mask
(`masks.txt`, named by the graphic — `docs/DATALAYER.md`) is 1 get
`set_blocked_at(1)` (`T |= 0x4000`, neighbours `0x2000`), the others
`set_blocked_at(0)` and, for a city or a `connects_to_roads` type,
`set_road_at(1)` — **and for a type that is neither, every footprint tile
takes `set_road_at(0)` first**, so a farm going up on somebody else's road
takes it away (`docs/ROADS.md` §9.5); a dock's three-tile sea ring
`set_bad_path(1)`; **a city:
`Wall::mask_city(tile, on, radius)` sets `T |= 0x100` on every tile of the
pre-tabulated even circle of `radius` tiles** around the tile under the
building's centre — the `even_circle_x/y` offsets up to
`even_circle_radius[radius]`, which `even_circle_init@006816d0` builds once:
for every `(u, v)` with neither zero, `round(√(u² + v²)) == r` (a `sqrtf`,
rounded half-up — **the one gameplay table the original builds with a
float**) is recorded as tile offset `(u − 1, v − 1)` for positive `u`, `v`
and `(u, v)` otherwise. It is a circle centred on the **corner** between the
centre tile and the one before it, so at radius 20 the mask spans `dx ∈
[−20, 19]` along the axis: one tile shy on the positive side. `crates/sim`
pins it as `4(u² + v²) ≤ (2r + 1)²` (`place.rs`, `city_mask_tiles`). *(Second
reading; a first draft had the disc as `vector_dist ≤ radius`.)* Then
`place_roads`. Nothing in
`init`/`start` touches the border sources — **placement adds no territory**;
`Build::activate` → `Cities::init_city` is where a city becomes a source
(`docs/ATTRITION.md`).

**Partly checked (2026-08-20).** A **library** placed at `(32, 136)` against
a city on `(32, 156)` was **disbanded**; a **tower** on the same tile
**started**, which rules out terrain and territory and leaves
`OUTSIDE_RADIUS`. That is what the even circle predicts at radius 20 — the
library's northmost footprint tile would have to be `135`, and the mask
reaches `136`.

**The asymmetry itself is still open**: it needs the library to *start* at
`(32, 137)`, and on the far side start at `(32, 174)` and be refused at
`(32, 175)`. `docs/ORACLE.md`'s console section has the setup that makes the
rest cheap — `Console Coord Mode=2`, `add NEW library 32,137`, `cheat add 1
citizen 32,141`, `cheat camera 32,137` and a right-click at the viewport
centre.

### 3.7 Every frame — `Wall::process`

`Build::process` begins with `Wall::process` and **returns if not active**.
`Wall::process`, phased by `o`: frame `& 7 == who` → `targeted >>= 2`; every
32 frames → the under-attack decay (§1.3) and, for an AI's site, the
builder-wanting logic (AI); `helpers == 0 → build_masks &= ~0x400` else
`helpers = 0, |= 0x400`; clear `0x800`; **every 16 frames, the building-side
attrition** (§9.5).

---

## 4. Finishing — `Wall::activate`, `Build::activate`

`activate(captured, announce, counted)`: `do_construct` calls `(0, 1, 1)`;
`capture_city` `(1, 1, 0)`; `check_capture`'s village path `(0, 1, 0)`. The
first gates everything a "built it" should do and nothing a transfer should.

**The common part, `Wall::activate`**: `ever_seen_completed |= ally_mask`;
**not started → `start`**; **`flags |= 4`**; `build_masks |= 0x1000`;
`job_counter = job_counter_2 = 0`; `build_flags & 0x10` or (a Build in a
city) → `increment_stats` (§4.2); `num_queued[type]--`; dirty flags.

**`Build::activate`**, before it: `LIBRARY` → `new_library`
(`docs/PRODUCTION.md`); `recharging = 0`; **a city, Chinese with
`CHINESE_LARGE_CITIES`, built not captured, not the Forbidden City →
`set_type(TOWN)`** — Chinese cities are founded as Large Cities; `counted` →
`last_building_finished[type] = o`. After it, the side effects that are
gameplay ("G") rather than message/sound/AI:

| branch | effect |
| --- | --- |
| **city founded** (`flags & 0x20`) | `remove_unbuilt_city`; the capital decision — outside a scenario, **the first city** (`city_num == 0`, `leader_flags2 & 1` clear) is the capital, a Lakota variant re-seats; `city = Cities::init_city(who, o, captured, capital)` (§5.1); **`City::find_buildings`** (§5.4); `counted` → `cities_built++`; not captured → `race = founder = who`; `city_num++`, `world_cities++`, `city_mine++`; **`calc_pop_cap`** |
| GRANARY / LUMBERMILL in a city | `city_flags |= 0x200` / `0x400`; the enhancer percentage from its level table (`docs/ECONOMY.md`) |
| TEMPLE in a city | `city_flags |= 0x80`; **`Region::fix_borders`** — the temple extends territory |
| MARKET in a city | `city_flags |= 0x800` |
| SENATE | not captured and no living government patriot → `train(get_gov_hero())`; in a city, not captured: `senates_built++`; **if the city is the capital's race and the capital has no senate, the capital flag moves here** (`0x10` cleared elsewhere, set here, `fix_borders`) |
| any, in a city | `City::regen_roads` |
| fort / dock / oil | `Forts::init_fort` (**a territory source**, `docs/ATTRITION.md`), `Docks::init_dock`, `OilWells::init_oil_well`; the unbuilt lists popped |
| dock, MARKET, TEMPLE | a count and its high-water mark; a new high (frame > 0, built, counted) → `do_bonus(WEALTH, 30)` |
| high-water mark | the nation's free units — **§4.3**, whole |
| gather type | `gather_slots[k] += gather_max`; a new high → the first-slot resource bonus (`FOOD_BONUS_FOR_FARM`, `TIMBER_BONUS_PER_WOOD_SLOT`, …) |
| **wonder** | `remove_unbuilt_wonder`; `wonders_built++`; `Wonders::init_wonder`; **every other player's unfinished copy of the same wonder is `disband`ed with a full refund**; the per-wonder one-offs — Terra Cotta's free-unit timer, the Kremlin spy, the Forbidden City ending a lost-capital countdown, the Hanging Gardens / Colosseum / Statue of Liberty / Red Fort / Tikal free techs via `gain_tech`, Colossus `calc_pop_cap`, Colosseum/Eiffel/Tikal a border recompute, Space Program the map reveal |
| tail | **`update_hits(0)`** — full health now; `update_los`; **a non-city in a city → `City::check_upgrade`** (§5.3); `update_seen` |

So activation touches the economy only through the listed bonuses, recomputes
territory only for a city, a temple, a senate-moved capital and three
wonders, and moves the population cap only for a city and the Colossus.

### 4.1 The two `finished`

`Build::finished@00470e50` is a thunk to `Build::finished@00628490` — the
**queue-side** completion `docs/PRODUCTION.md` specifies (a unit → `train`; a
building type without `build_flags & 4` → `set_type` in place with the pop
delta, `calc_pop_cap` and `fix_borders`; else `gain_tech`). **There is no
construction-side `finished`**; construction completes in `activate`.

### 4.2 The counters — `Wall::increment_stats` / `decrement_stats`

**Active**: `num_buildings[type]++` and, in a region `< 64`,
`reg_buildings[reg][type]++`; a military trainer onto `mil_trainers`. **Not
active** (through `set_type` only): `num_queued[type]++`. Callers:
`Wall::activate` when `build_flags & 0x10` or in a city; `add_to_city` /
`remove_from_city` for an already-active building changing city;
`Build::close`; `City::close`; `Wall::set_type` (decrement the old, increment
the new — the level-up moves the count from `VILLAGE` to `TOWN`). **A
finished building outside any city without `build_flags & 0x10` is not
counted** — the tech tree's "has a temple" predicates follow the same rule.

### 4.3 The high-water mark, and the nation's free units (2026-09-04)

The row above called this "first-of-its-kind". It is a **high-water mark**,
and the difference is the whole rule: the British get archers with *every*
Barracks, and what the mark buys is that a **replacement** for one that died
gets none.

`Build::activate@00623e20:603` computes

    n = num_buildings[type] + get_buildings(to)          // the upgrade chain above it
    if high_buildings[get_base_type(type)] < n and not captured:
        high_buildings[get_base_type(type)] = n
        if counted:  <the nation's free units>

`get_buildings(who, t, n)@006e0680` walks `BuildTypeData +0x2e0` — the type's
`TO` — adding each `num_buildings`, so a Tower's count includes its Keeps.
The **mark**, though, is indexed by `get_base_type`, the type vtable's
`+0xe4`: the same slot `Unit::think_scout@005f6010:286` hands
`FILTER_BASE_TYPE`. So the mark is per lineage root and the count is per
lineage — a Keep does not re-earn what its Tower claimed. `num_buildings`
(`LeaderData +0x555e`) and `high_buildings` (`+0x5660`) are `ushort[129]`
addressed by raw `TypeIndex` with the `BASE_BUILDTYPES` bias folded into the
base pointer, which is why `BuildData::construct_time@0062d5c0:126` indexes
the same array with the type index unadjusted.

The arms are an else-if chain on the building's own `TypeIndex` — not on its
lineage, so `ANCHORAGE` and `SHIPYARD` are named beside `DOCK` — and each is
four steps: the power, a count, the type, and `check_population` before
**every single unit**.

| building | power | count | unit |
| --- | --- | --- | --- |
| `MARKET` | Nubians (4) | `NUBIAN_FREE_CARAVAN != 0` (ships **0**) | `CARA` |
| `BARRACKS` | Iroquois (18) | `IROQUOIS_FREE_SCOUT` | `SCOUT` |
| `BARRACKS` | Romans (6) | `ROMAN_BARRACKS_LEGION × tier`, capped `ROMAN_MAX_LEGION` | `HOPLITES` |
| `BARRACKS` | British (11) | **the tier itself** — no multiplier, no cap | `BOWMEN` |
| `BARRACKS` | Aztecs (0) | `AZTEC_BARRACKS_LIGHT × (1 / 2 / 3)`, capped `AZTEC_MAX_LIGHT` | `SLINGERS` |
| `SIEGEFACTORY`, `FACTORY` | Turks (8) | `TURK_FREE_SIEGE` | `CATAPULT` |
| `SIEGEFACTORY`, `FACTORY` | French (10) | `FRENCH_FREE_SUPPLY != 0` | `SUPPLYWAGON` |
| `DOCK`, `ANCHORAGE`, `SHIPYARD` | Spanish (9) | `SPANISH_FREE_TRIREME`, and `ages < 5` | `TRIREME` |
| `DOCK`, `ANCHORAGE`, `SHIPYARD` | Dutch (22) | `DUTCH_FREE_LIGHT_SHIP` | `BARK` |
| `DOCK`, `ANCHORAGE`, `SHIPYARD` | British (11) | `BRITISH_FREE_FISHERMEN` (ships **0**) | `FISHERMEN` |
| `UNIVERSITY` | Americans (20) | `AMERICANS_FREE_SCHOLAR` | `SCHOLARS` |
| a fort (type vtable `+0xfc`) | French (10) | `FRENCH_FREE_GENERAL != 0` | `GENERAL` |
| `AIRBASE` | Germans (12) | `GERMAN_FREE_FIGHTER` | `BIPLANE` |
| `AIRBASE` | Americans (20) | `AMERICANS_FREE_BOMBER`, and `has_tech(MODERN_AGE)` | `BOMBER` |
| `STABLE`, `AUTOPLANT`, not a city | Mongols (17) | `MONGOL_START_CAVALRY` below Military 2, else `MONGOL_FREE_CAVALRY`, floored at `MONGOL_THREE_MIL_CAVALRY` past Military 2 | `HORSEARCHERS` |
| a city (`flags & 0x20`) | Koreans (16) | `KOREAN_CITIZENS[city_num − 1]`, clamped to `[0, 7]` | `PEASANTS` |

**The tier is `min(epoch[0], ages)`** — the Military library level and the
age are different fields (`LeaderDataEncrypt +0xe8` and `+0xdc`, XOR
`0x63187` and `0x62766`), and the *smaller* is what the ladder reads. The
Roman and British ladders are `age >= AGE_FOR_3 ? 3 : age >= AGE_FOR_2 ? 2 :
age >= AGE_FOR_1 ? 1 : 0`; the Aztec one is its own shape, `1` at zero and
then `2`, `3` past two. **A zero cap skips the scaling entirely** and pays
the plain per-building figure: the whole ladder sits inside `if (max != 0)`.

Every arm but three runs its base type through `get_graft` and then
`LeaderData::current_upgrade`, so what arrives is the newest type in the
line. The three that skip the upgrade are the American scholar, the French
pair and the Korean citizen.

**And a squad is `uber_size` units, not one unit with three figures.**
`Objects::init_unit@0065e0c0:34` reads `UnitTypeData::uber_size` (`+0x308`)
and loops that many times, each pass a whole `Unit::init` with its own
`Guy::init_real` draw; `Unit::init@00612100:508` sizes the guy stack to
`crew_size + squad_size`, and `squad_size` (`+0x304`) is **written 1 by
`UnitType::init@0061ab50:723` and never written again**. So a Bowmen —
`UBER_SIZE 3`, `CREW_SIZE 0` — is three one-figure objects threaded
`o_up`/`o_down` as a **list**: the head's `o_up` is −1, a member's `o_up` is
the member before it, and run17's frame 1301 has `6 → 7 → 8` exactly so.
Only the head is counted: `init_unit` hands every unit that has an `o_up`
straight back to `track_unit_type(·, −1, ·)` and undoes `control`,
`num_units` and the two running totals, so a squad is one unit and one
population everywhere else. The members are seated by `find_nearby_spot`
around the captain, which takes no draw.

**Coverage.** Diff-backed: the **British Barracks arm**, the gate, and the
`uber_size` loop. Great Lakes' word runs 6612 → **6650** on them together —
run53's AI is tribe 11, its Barracks `1/2016` activates on the exact frame
the original spends three `Guy::init_real` and three `Unit::do_idle` idle
rolls, and the fifteen functions the original enters for the *first time in
24,000 frames* on 6612 are `Army::add_unit`, `Unit::think_attack` and their
neighbours. `a_british_barracks_pays_one_bowmen_as_three_chained_units`
pins the chain, the single count, the second Barracks paying again and the
rebuild paying nothing. All twenty-five constants are re-derived from
`rules.xml` by `cargo run -p rondata`.

**Not established.** Every other arm rests on the reading alone — no capture
on disk has a Roman, Aztec, Turkish, French, Spanish, Dutch, American,
German, Mongol or Korean leader. Nor is the `param_3` gate's second clause
modelled: the original also wants `(semaphore[1] & 0x10) == 0 &&
(semaphore[2] & 2) == 0`, or `ScenarioData::building_unit_bonus`, and what
those two bits are is unread. The Aztec ladder's **pre-patch-4** arm (which
reads `ages` where this reads the min) is not modelled either.

---

## 5. Cities

### 5.1 The record — `Cities::init_city`, `City::init`

`init_city(who, o, transfer, capital)`: the first slot below `city_mark` with
`city_flags & 1` clear, else a new `City`; `city_mark = max(city_mark, slot +
1)`; `City::init`. `City::init`: `city = slot`, `o`, `who`, `pop = 1`, `reg`
from the building's cell, `x, y`; `city_flags = 1`; `race = −1`; stamps
(attack, raid, capture, assimilation, reduce) = `transfer ? frame : 0`; `pv
= get_pop_value()`; `leader.pop += pv`, `world_pop += pv`, `reg_pop[reg] +=
pv`, `reg_cities[reg]++`; `calc_pop_cap`; every region's `borders` (+0x2c)
zeroed (a border recompute trigger); capital → `city_flags |= 0x10 | 0x4000`;
name; `fix_world_vals` (AI site values). `City::close` (§8.4) undoes it.

### 5.2 Membership — `find_city`, `get_town`, `find_city_at`, `add_to_city`

`Build::init` runs `find_city` for every non-city building **at placement**.

`Build::find_city`: a city returns its own `o`; `city ≥ 0` →
`remove_from_city`; `c = get_town(tile(x), tile(y), who)`; `< 0` → none; else
`add_to_city(builds[who][c].city)`.

`BuildTypeData::get_town`: **every footprint tile must carry `T & 0x100`**
(inside some city mask) else −1; then `find_town_at` for an `l`-type, else
`find_city_at(centre, who, type, 0)`.

`ObjectsData::find_city_at(x, y, who, type, skip)`: the tile must be
`WorldData::is_city_at` — `T & 0x100` **and** the cell's territory owner is
nobody, `who` or an ally; then over `who`'s live cities: `d =
vector_dist(tile deltas)` against `CityData::get_radius(C)`; a `build_flags &
0x200` type at a city that already has one gets `d += 100`; keep the
nearest. So a building belongs to **its owner's nearest city whose radius
covers it**, with the one-per-city types pushed toward a city that lacks one.
`find_town_at` is the same over `TOWN`-lineage cities without the penalty.

`Build::add_to_city(c)`: `city = c`; a city is its own head; else append `o`
to the chain (`city_down` links, tail-appended); **only if already active**:
`increment_stats` (unless `build_flags & 0x10`) and the TEMPLE `0x80` /
LUMBERMILL `0x400` / GRANARY `0x200` / MARKET `0x800` flag.
`remove_from_city` unlinks, `regen_roads`, clears each flag whose
`count_buildings(kind, 0, 1)` dropped to zero, `city = −1`, `decrement_stats`.

### 5.3 The radius and the levels

`LeaderData::get_radius(who, t)` = `min(64, CITY_CENTER_RADIUS (20) +
(level(t) − 1) × CITY_CENTER_POP_RADIUS (4) + (Indians ? INDIANS_CITY_RADIUS
(4) : 0))` tiles; `CityData::get_radius` is that for the city building's type.
It bounds `find_city_at`, `find_buildings`, `mask_city` and the capture
sweep. **20, 24, 28 tiles for the three levels.**

**The level-up is automatic, not queued.** `TOWN` and `METROPOLIS` are never
put in a queue (`get_total_cities` counts only queued `VILLAGE`s; `could_queue`
asks only about `DEPOPULATE`/`DISBAND`). `City::check_upgrade` runs:

```
ready_to_upgrade:
    t = city building type;  t == METROPOLIS or FORBIDDENCITY → 0
    next = (t != VILLAGE) + TOWN
    !type_avail(who, next, 1) → 0                     # docs/TECH.md: the Civic line's gate
    can_upgrade(next): VILLAGE → 1; TOWN → enough_kinds(CITY_BUILDINGS + 1); else enough_kinds(METRO_BUILDINGS + 1)
enough_kinds(n): n = min(n, 24); walk the chain from o (the city itself first):
    alive, a Build, active, type not yet seen → collect; count ≥ n → 1
check_upgrade:
    !ready → return
    old = get_pop_value();  set_type(upgrades_to())           # decrement_stats, ptype, increment_stats
    mask_me(1, REGEN_NONE)                                    # the larger radius mask
    leader_flags |= 0x8000000                                 # → update_hits on every building (the Senate bonus, the new hits)
    find_buildings()                                          # pulls in what the larger radius now covers; ends in check_upgrade again
    pop += new − old (leader, world, region);  calc_pop_cap;  Region::fix_borders
```

So `CITY_BUILDINGS` (5) is **the number of distinct completed building types
besides the city itself** a city needs to become a Large City, `METRO_BUILDINGS`
(9) a Major City — the `+1` pays for the city building at the head of the
list. Nothing else: not age, population, wonders or territory; the tech gate
is entirely `type_avail(next)`. The city keeps its `damage` and gains the new
type's hits. Triggers: the tail of `Build::activate` for any non-city member;
`find_buildings` (founding, capture, Civic tech, the level-up itself);
`Leader::gain_tech` (`docs/TECH.md` step 11) when the gained tech is a
prerequisite of `TOWN`.

### 5.4 `City::find_buildings` — the radius sweep

Called on founding, after a level-up, on capture, after every Civic tech
(`docs/TECH.md` step 10: `mask_city` then this), and by the editor:

```
r = min(64, get_radius)
for every active player p, every building B of p:
    alive; not a city; not already in a city; (p != who and B.build_flags & 0x10) → skip
    vector_dist(tile deltas) > r → skip
    p == who:  LIBRARY → new_library;  add_to_city(B)
    else:      n = B.swap_team(who);  n < 0 → B dies and **the sweep returns here** (the rest waits for the next trigger)
               n.activate(0, 1); B.close(0); mask_me(n, 1, REGEN_SIMPLE)
check_upgrade()
```

**Foreign city-bound buildings caught inside a city's radius are converted**
— on founding, level-up, capture and every Civic tech, not only on capture.

### 5.5 What the level feeds

| consumer | rule |
| --- | --- |
| population value | `get_pop_value` = 1 / 3 / 5 → `leader.pop`, `world_pop`, `reg_pop` |
| population cap | `CityData::pop_cap` = `VILLAGE_POP × level` per live city into `calc_pop_cap` (`docs/COSTS.md`; ships 0) |
| radius | 20 / 24 / 28 tiles (+4 Indians) |
| farm limit | `+ (level − 1) × FARMS_PER_CITY_LEVEL` (ships 0) |
| trade value | `num_buildings() + {0, 2, 4}` |
| territory | `CITY_UPGRADE_TERR[level]`, `TERRITORY_LIMIT_CITY` per level (`docs/ATTRITION.md`) |
| member hits | `+ SENATE_HP_BONUS × (level − 1) %` on non-defensive members (§3.4) |
| capture plunder | `CITY_PLUNDER_PER_LEVEL × (level − 1)` (§8.3) |
| garrison, arrows | **nothing by level** — `garrison_max` and `base_arrows` are the three types' own columns (10 / 15 / 20) |

`num_buildings()` — alive, Build, active members **from `o`, so the city
building itself counts**; it feeds `get_taxes` and `get_trade_value`
(`docs/ECONOMY.md`). `count_buildings(t, exact, active_only)` on the chain;
`get_building(t)` the first active `is(t, 0)`; `num_wonders(exclude_city)`
excludes the Red Fort.

### 5.6 The capital

`0x10` on the record (`0x4000` marks the founding one). The first city a
player founds outside a scenario is the capital; a senate in the capital's
race city can move it (§4). `LeaderData::find_capital(L, skip)`: the first own
live city with `0x10`; else another player's live city whose
`was_capital_flags` has `L`'s bit; else none. `holds_capital` = the first
case. `has_capital` is true outright unless `elimination == 1`. What losing
it does is §8.5.

### 5.7 The `CITY` record, whole — `CityData::log_data@004895c0` (2026-09-02)

The dump writes one `CITY` block per **live** city — the whole function is
behind `if ((this->city_flags & 1) == 0) return` — at every detail level, on
every frame a block is written at all. Forty fields, in this order:

`x`, `y`, `pop`, `who`; the two bare strings `name` and `id` (`id` only when
non-empty, and neither carries a key, so neither is parsed); `race`,
`city_flags` (§1.4), `city`, `attack_stamp`, `raid_stamp`, `reduce_stamp`,
`capture_stamp`, `assimilation_timer`, `capture_strength`; then
`Array<CaravanLink>::log_data(&vans)` — its own `length`/`size`/`increment`/
`flags` header and one nested block per link, each `cara` and `who`; then
`o`, `reg`, `scouted`, `in_port`, `peasant_dist`, `trade_val`, `free`,
`busy`, `gatherers`, `ocean`, `land`, `filled`, `bordering`, `ocean_filled`,
`dock_tile`, `was_capital_flags`, `space[3]`, `ter[6]`.

**The four container fields between `capture_strength` and `o` are the
caravan array's, not the city's** — `Array<CaravanLink>::log_data@00489390`
writes them before its entries, and the enclosing `CITIES` array writes its
own three a level up. Reading `length` here as a field of `CityData` gives
the caravan count.

`city` is the slot **within the owner's own city array** — what an
`ARMYDATA`'s `city` indexes (`docs/ARMY.md` §13), not the global `CITIES`
position; the two starting cities of a two-player game are both `city 0`.
`o` is the centre building's object number, which is the identity
`rondata::diff` links a city on, the same one a `BUILDDATA` row carries.

**Coverage.** Diff-backed on run58's 5,201 frames, every field of every live
city, 606,540 comparisons —
`diff::tests::run58_s_five_thousand_frames_stand_where_the_original_s_do`.
Everything agrees except three seams, each pinned there as it stands —
`ter[6]` was a fourth until `World::gather_at` landed, and now agrees on
both of the AI's cities on all 5,201 frames:

| open | why | where |
| --- | --- | --- |
| the human's whole site picture, 13 fields (`ter` among them), every frame | `Sim::strategy_all` skips a human leader; the original does not | `docs/AI.md` §23.1 |
| `1/2007`'s `land`, `filled`, `space[3]`, one apart | the second city's circle sweep | open |
| a gatherer and a free citizen filed under the wrong city | the totals agree, the attribution does not | open |

Before this the record was read four fields deep (`x`, `y`, `pop`, `who`)
and compared only at frame 1 of one capture.

### 5.8 `free` is a byte, and it wraps (2026-09-07)

`CityData +0x5a free` is a **`uchar`** in the type record, and both of its
writers are a bare byte `±1` with no clamp: `Leader::plan_strategy@006b9620`
`:839` increments it for every citizen of the city the sweep finds idle
(`docs/AI.md` §2.3 step 10), and `Leader::produce_building@006e1400:1145`
decrements it when the builder it chose came off the free peasants rather
than the gatherers (§2.20). So a decrement at zero lands on **255**.

Three things say that is the original's shape rather than the decompiler's.
The `space[]` loop six lines above the second writer *does* clamp — `iVar13
= *(byte *)(iVar23 + 0x67 + iVar12) - 1; if (iVar13 < 0) iVar13 = 0;` —
so the absence next to it is deliberate. `log_data` prints **255** and not
−1, which is a zero extension and so a `uchar`. And the one consumer that
reads the counter back rather than testing it against zero,
`Unit::find_gather_spot@005f5170:116`, widens it the same way:
`(uint)*(byte *)(iVar7 + 0x5a)`.

**Diff-backed.** run79's `1/2007` wraps on frame **7183** — the same frame
this crate's own decrement fires, so this was a width and never a timing
difference — and read `-1` here against `255` there on all 67 frames to the
end of the window, until `ai::CityAi::free` became a `u8`. The assertion is
in `run79_s_window_is_every_unit_s_whole_record`, which excludes the rest of
the `CITY` record; it was made to fail on those 67 rows first.

The consumers this moves are the ones that add the counter rather than test
it: `find_gather_spot`'s "fewer than two citizens" gate (`free + gatherers`),
and `create_units`/`create_buildings`' `free + busy` against the city's
slots. Every one of them saw 9 where the original saw 265.

**The rest of the record is the same width and is not yet.** `busy`,
`gatherers`, `ocean`, `land`, `filled`, `bordering`, `ocean_filled`,
`dock_tile`, `space[3]` and `ter[6]` are all `uchar` at `+0x5b`–`+0x71` and
all `i32` in `ai::CityAi`, and `pop` at `+0x5d` is one on `sim::City`;
`gatherers` has a bare byte decrement of its own at
`produce_building@006e1400:1135`. No capture on disk shows one of them
wrapping — `docs/DATALAYER.md` §4.2 carries the row.

The two `free + busy` consumers above are also the fourth reading of the
width, taken from the other end: `create_buildings@006c1be0:1606` widens
`+0x5a` with `(uint)` before it adds `+0x5b` to it, and
`create_units@006c40a0:1209` does the same. Four sites, one answer.

---

## 6. Garrisons

### 6.1 The chain

One doubly-linked chain per container: `ObjectData::inside_down` (+0x28) /
`inside_down_who` (+0x3e) on the building point at the first garrisoned unit;
`UnitData::inside_up` (+0x82) / `inside_up_who` (+0xb4) on a unit point at
what is above it — the building for the first, the previous unit for the
rest. **`UnitData::is_on_map` = `inside_up < 0`** — the whole test the engine
uses for "not garrisoned and not in a transport" (`docs/SUPPLY.md`,
`docs/ATTRITION.md`). `Object::insert_inside` appends at the **bottom**
(`inside_bottom` walks to the end) — FIFO — and takes the unit off the tile
grid; `remove_from_inside` splices it out and puts it back at whatever
position it has; `come_out` sets the position first. `get_inside` returns the
outermost container (a unit inside a transport inside nothing).
`in_a_building` = the outermost is a Build.

### 6.2 Counting — `num_inside`, `count_inside`

`num_inside(mode)`: 0 unless on the map (a building always is); over the
chain's captains — `mode != 0`: one per squad; **`mode == 0`: a government
patriot counts 1, a decoy 0, anything else its type's `control_cost`** — the
slot count the limit is measured against. `count_inside(selector, arg)` skips
every entry that is not `is_valid_unit`; the selectors (PDB `CountIndex`):
`WORKERS`, `PEASANTS`, `ATTACK` (captain with attack), `MILITARY` (obj_masks
`C`), `DOMAIN == arg`, `ON_MAP`, `GATHER_PEASANTS/WORKERS/SCHOLARS`,
`BUILD_PEASANTS`, `STRICT_TYPE`/`TYPE`/`TYPE_SAME_WHO`/`NON_DECOY_TYPE`,
`CATEGORY`, `REGION`, `TRADE`, `GARRISON_ARROWS` (26 — `docs/COMBAT.md` §8.6's
sum), `WHO_INSIDE == arg`.

### 6.3 Capacity — `BuildTypeData::get_garrison_limit(who)`

```
limit = garrison_max                                  # GARRISON_MAX
is(TOWER, 0): level = has_preq(FORTGARRISON4) ? 4 : …3 ? 3 : …2 ? 2 : 1;  limit += (level − 1) × TOWER_GARRISON_UPGRADE (2)
is_fort:      same;                                                       limit += (level − 1) × FORT_GARRISON_UPGRADE (5)
```

No nation, wonder, age or city-level term. Shipped: Small/Large/Major City
10/15/20; Barracks, Stable, Auto Plant, Market 10; Siege Factory, Factory,
Dock, Anchorage, Shipyard 20; the tower line 5 (+2 per level), the fort line
10 (+5); Forbidden City, Red Fort 20; **University, Temple, Library, Senate,
Airbase, Missile Silo, farms and the economic buildings 0** — and a 0 limit
refuses the garrison *order* outright, though scholars still enter a
university by gathering and planes an airbase by training.

### 6.4 The order — `Unit::do_garrison`

`UnitTypeData::can_garrison(unit type, building type)`:

```
domain == SEA and (unit_flags & 0x10 [transport/merchant] or AIRCRAFTCARRIER) → 0
unit_flags & 0x1800 (l "town/tower" or m "fort" — the code does not tell them apart):
    building is_fort or is(TOWER, 0) or is_city → 1
domain == SEA → building is_dock
building == where (the type's trainer) and not UNIVERSITY → 1
where == BARRACKS:            building ∈ {BARRACKS, STABLE, AUTOPLANT}                                   (patch ≥ 4; else BARRACKS only)
where ∈ {STABLE, AUTOPLANT}:  building ∈ {BARRACKS, STABLE, AUTOPLANT, SIEGEFACTORY, FACTORY}            (patch ≥ 4)
where ∈ {SIEGEFACTORY, FACTORY}: those five, or is_fort, or is_city                                      (patch ≥ 4)
0
```

Citizens (`where` = the city) may enter their city; scholars' `where` is the
University and is excluded (they enter by gathering); military with `l`/`m`
may enter city, tower and fort; any unit may enter what trains it and its
sibling military buildings; ships only docks; helicopters reach airbases by a
strafe order; other aircraft never through this path.

`Unit::do_garrison`, per tick, with the target `(o, who)`:

1. A helicopter (`unit_flags & 0x20`) at an `AIRBASE` → the strafe/land path.
2. Target must be a wall/building and **active**.
3. **Same owner, or mutual allies** (`diplos` both 2) — an ally's building
   accepts your garrison.
4. `get_garrison_limit == 0` → kill; `!can_garrison` → kill.
5. Not adjacent → a move order to the approach ring (`min(x_size, y_size) ×
   0x60 + 0x30`; a dock keeps ships off at `+0x1b0..+0x330`) and return.
6. A city: **`CityData::race == owner` (assimilated)** else refused; **`hits/10
   ≤ hits_left`** — a city at under a tenth of its hit points admits nobody.
7. `n = num_inside(0)`, `cost = decoy ? 0 : control_cost`: `n + cost ≤ limit`
   → **the territory owner of the building's tile must be nobody, the owner or
   the owner's ally** (a building standing in enemy territory cannot be
   garrisoned); `go_inside(captain)`; kill the garrison orders. Else (full):
   with the `search` flag and a city, `find_garrison_build(city)` — the
   nearest sibling on the chain with room that `can_garrison` — and re-order
   there; else "building full".

`Group::action_garrison` (the UI/AI group action) applies the same gates
before issuing the order, refuses an `is_unassimilated` target, and `Group::
action_alarm` (the town bell) garrisons every citizen into its city and sets
`CITY_ALARM`.

## In and out — `go_inside`, `come_out` (§6.5)

A subsection of §6 by number, and its own `##` section by size: §6.5
and its two findings are twelve of §6's twenty-one thousand bytes, over
the guard's ceiling, and `CLAUDE.md` splits rather than shaves. The
numbering is unchanged — §6.5, §6.5.1 and §6.5.2 are what the code
cites and what it still cites.

### 6.5 `Unit::go_inside`, `Unit::come_out`

`Unit::go_inside(o, who)`: climb to the captain; `insert_inside`; a squad
(`uber_size > 1`) entering a building clears its orders; the followers follow
down the `o_down` chain; **a scholar is seated on the container, faced to
angle 0 and given a forced `CHAR_DEFAULT` — one draw, §6.5.2**.
Callers: `do_garrison`, `action_garrison`, **`Build::train` (every trained
unit is born inside)**, the flamethrower citizens, `do_gather`/`do_board`,
transports, the editor.

`Unit::come_out(captain)` → 0 on success, 1 on failure: the exit ring around a
building is `d = (x_size + y_size) × 0x30` (+ `big_radius` for a ship); land
`[d + UNIT_TRAIN_DISTANCE, d + UNIT_TRAIN_MAX_DISTANCE]` (3/2 and 5/2 tiles ×
192), sea the `BOAT_*` pair (3/2, 8 tiles); **a dead building (`flags & 1`
clear) has `min = 0`**; a set gather point turns the exit toward it.
`find_nearby_spot` with `FILTER_ALL` for a type with no `block_radius` (retry
at double the ring, then the container position), `FILTER_NOT_ME` otherwise
(retry relaxed, then **fail — the unit stays inside**). Then
`remove_from_inside`, `set_new_location`, the followers, an AI unit leaves
with no orders, the city-alarm cleanup when the city empties, the rally-point
logic (`docs/PRODUCTION.md`). A land citizen leaving an oil platform is
auto-loaded into a fresh transport barge. The **Eject** button is
`Unit::action_come_out` → `come_out(0)`.

**The sea arm is built and diff-backed (2026-09-01).** It is chosen on the
*unit's* own `domain == 1` (`6183b6`), not the building's, and the
`big_radius` term is `ObjectType +0x244` — which
`UnitType::init@0061ab50` computes as
`(num_guys − 1) × guy_spacing / 2 + block_radius` at `61ba93`, thirty
instructions after storing `num_guys = 1` (`61b9de`) and with nothing
between, so in the shipped build **`big_radius` is `block_radius` for every
unit type** and nothing else in the export writes the field. run58's
Fisherman `1/14` is born 192 units further from its Dock than the land ring
would put it, which is that term exactly.

#### 6.5.1 A squad member's host is its captain, not the building

**Diff-backed and closed** (2026-09-04, run76, item 227; opened by 223). A
squad is `uber_size` **objects**, not one object with three figures (§4.3),
and the original does not place them together: `come_out` re-enters on
`o_down` with the "already the captain" flag set (`00617c10:535`; `param_1`
at `:172` stops the member bouncing back up), so each member repeats the
whole search with its siblings already on the map.

**And not around the same thing.** The host `get_inside` returned is thrown
away for a unit that is not its squad's captain: at `618022`–`618044` a clear
`is_captain` bit — `(o_up >> 15) == 0` — calls `get_captain()` (vslot `0xe4`)
and writes it, with the unit's own `who`, into the slots every later term
reads (`[esp+0x14]`, `[esp+0x50]`, folded to `who × 0x1c` at `61804d`). The
host chooses the arm: vslot `0x1c` is the folded `return 1` on
`Build::vftable` and `return 0` on `Unit::vftable`, so a **member takes the
unit-host arm** — a transport passenger's own (`docs/TRANSPORT.md` §6.4) —
with every term the captain's (`61845c`–`618483`):

| term | a captain | a member |
|---|---|---|
| centre | the trainer's position | the **captain's**, as just placed |
| bearing | south, `0x80000000` | the captain's `angle` (`+0x50`) |
| min | the training ring, 0 while the trainer dies | the captain's `block_radius` (`+0x240`) |
| max | `+ (UNIT_TRAIN_MAX − UNIT_TRAIN)` | `min + UNIT_DISEMBARK_DISTANCE` |
| step | 0 → `(max − min) / 8` | the same |

Only the fallback arm is the leaving unit's own: `618490` reads `0x240` off
`this`, so `block_radius == 0` sweeps `FILTER_ALL`, the doubled ring, then
the host's own point, and non-zero sweeps `FILTER_NOT_ME`, the same ring with
collision off, and then **refuses** — the unit stays inside.

**The south bearing is a read, not only a diff.** The constant is at
`617c33`, `movl $0x80000000, 0x30(%esp)` — `[esp+0x2c]` in the body's frame,
`esp` being four lower there for `Group::clear`'s argument push — and that
is the slot `6184cc` hands over as `bias_angle`. Its only other writers are
the gather-point block (`6182c8`, `618355`: `find_angle` toward the rally
spot, also stored as the unit's `angle`) and the unit-host arm (`618462`).

**run76 pins every term at once.** Barracks `1/2016` at `(45120, 25728)`,
`x_size = y_size = 4`: the captain's ring is `8 × 0x30 + 288 = 672` out to
`864`, step `24`, swept from south, and bearing 0 at `r = 672` snaps to
`(45144, 26424)` — `1/27`. From **there** the members' ring is
`[48, 48 + 576]`, step `(624 − 48) / 8 = 72`, swept from the captain's
`angle`, still `Unit::init`'s `0x55555555` on all three. Snapped, in the
sweep's own order: `1/28` is candidate **36** (`r = 120`, `k = 3`) →
`(45144, 26568)`, the first at `144` units where 0–35 are `48`–`135.8` and
its block refuses them; `1/29` is candidate **62** (`r = 192`, `k = 0`, the
bias itself) → `(45288, 26520)`, with 36 and 59 now `1/28`'s point and 37–61
inside the captain again.

Both are exact, and neither is a candidate of the *trainer's* ring at all:
`1/29` sits at `11.25°` from the Barracks, the one thirty-second the
31-bearing counter never produces (`docs/ORDERS.md` §10) — which is what
named this item.

~~**The dump cannot show the chain.**~~ **It does, and a `grep` would have
said so before two captures were booked** (2026-09-06). `o_up`/`o_down` are
`+0x8e`/`+0x90` and `is_captain` is the sign bit of `o_up`
(`00617c10:172`); the `ObjectData` `up`/`down` printed early in a record is
the per-cell list, but at `UNITS=3` the tail prints `o_up`/`o_down` under
those names — run79 reads `-1/32, 31/33, 32/-1`, run76 `-1/28, 27/29,
28/-1`. The captain is the head of the list, in the dump, all along.

**Before the garrison.** `Objects::init_unit@0065e0c0` gives each member a
spot around the captain — `[f × 0x30, f × 0x60 + 0xc0]`, `f` the captain's
raw `BLOCK_RADIUS` (`+0x248`), bias its `angle` — then `set_new_location`.
`Build::train` calls it at the trainer's centre and `go_inside` swallows
all three at once, so a trained squad never keeps it and one born on open
ground does. Not modelled, and no capture reaches it.

**What is not established.** Whether a member is turned to its captain's
`angle` on the way out. `come_out`'s tail splits on the same `is_captain`
bit (`006191a5`) and the not-a-captain arm calls
`set_angle(this, host->angle)` for a unit host (vslot `0x8`);
`docs/TRANSPORT.md` §6.4 has it diff-backed for a passenger. run76 cannot
separate it — both angles are `Unit::init`'s initial value — so it is not
implemented here, and the falsifier is a squad ejected from a building whose
captain has turned.

#### 6.5.2 A scholar is seated on its host, and a trained one never leaves

**Diff-backed and closed** (2026-09-18, run53/run54/run80, item 338). The
one clause §6.5 already carried — "scholars are moved to the container
position" — is three statements, and none of them was implemented. They are
the last block but one of `Unit::go_inside@0061a2e0`, under a gate that is
verbatim `ObjectData::is_scholar@0046d330`:

```
iVar3 = *(int *)(*(int *)&this->field_0x18 + 4);      /* UnitTypeData +0x4 */
if ((iVar3 == 0x34) || (iVar3 == 0x35)) {             /* == is_scholar     */
  leaders.list[who] |= 0x2000000;
  host = objects[param_2][param_1];
  set_new_location(this, host->x ^ 0x63637, host->y ^ 0x63637, 1, 1);
  set_angle(this, 0, ..., 1);
  set_anim(this, CHAR_DEFAULT, 1, 1);
}
```

`UnitTypeData +0x4 in {0x34, 0x35}` is not read from the surrounding code:
`ObjectData::is_peasant@0046d310`, `is_scholar@0046d330` and
`is_worker@0046fa10` are that comparison and nothing else, so the predicate
has a name in the executable's own symbols. It is this crate's
`Worker::Scholar`. The block is inside the per-unit recursion, after
`Object::insert_inside`, so each member of a squad takes it in turn — which
for a scholar (`uber_size` 1) is once.

**`set_anim(CHAR_DEFAULT, 1, 1)` is the draw, and its `force` is the point.**
The second argument skips `Guy::set_anim`'s "already playing" early returns,
so the idle roll is unconditional: one `game_random` draw a figure, at
`Guy::set_anim+0x97a` through `Unit::set_anim+0x56` — the **fourteenth**
caller of that address (`docs/ANIM.md` §4, `docs/SYNC.md` §5). Nothing on
either map reached it before Great Lakes 8272, which is why the site table
had thirteen. Across run53's 24,000 frames it fires on exactly fourteen
frames (8272, 8680, 9087, 9201, 9322, 9510, 9717, 9861, 10012, 10140, 10306,
13555, 13736, 17570) and run54's on fourteen more, the first of them **8466**
— which was East Indies' own word.

The draw is also what *stops* one. A unit created this frame is not skipped
by `Objects::inc_time` (`docs/ANIM.md`, the `guys_inc_time` note), and a
scholar is not skipped for being inside either, so with `Guy::init_real`'s
zero `end_time` the new guy wrapped at once and spent `SITE_WRAP` in phase 7.
The seating gives it a real length before that phase runs. One draw replaces
one draw, in a different phase — so a frame's **count** is unchanged and only
its **order** moves, and Great Lakes 8272 parted on the sequence while the
word ran on to 8374.

**And a trained scholar does not come out.** `Build::train@0062f9b0`'s exit
block splits on the *trainer*: when the building answers `ObjectData::is`
`0x1a4` — the University, the same test `num_scholars@0062d430`,
`calc_gather@0062d360`, `num_gatherers@00630450` and `could_queue@0062da50`
use — it re-reads the **trained type's** `is_scholar` and then compares
`BuildData::gather_max` (`+0x80`, a `char`; the type record names it) against
`ObjectData::num_inside(1)` *after* the unit is in. Over the limit it ejects
like anything else; at or under it calls `check_gatherers` and the scholar
stays. §6.5's caller list said "**`Build::train` (every trained unit is born
inside)**" and this crate let every one of them straight out again, so
Great Lakes' `1/44` was born on 8272, ejected, walked back to its own
university and seated itself a second time on 8285.

**The value diff.** run80 dumps Great Lakes 23960–24001 at full detail,
15,700 frames past the word. Its block 24001 has fourteen of player 1's
units standing on exactly two points — seven at `(40416, 25248)` and seven
at `(41184, 15264)` — which are buildings `1/2019`'s and `1/2020`'s own
positions. `1/44` was `(24, 552)` off, the exit ring, and is now exact;
`1/45`, `1/48`, `1/49`, `1/50` and `1/56` with it, six of the eleven the
endpoint compares where **none** agreed before. The endpoint moves
64 → 61 off and 17 → 15 unlinked on this map, 66 → 62, 13 → 10 and 30 → 29
on the other.

**What this does not establish.** Three things.

- **Which university a scholar walks to.** `1/51`, `1/52`, `1/53` and `1/55`
  are still off at 24001, and by exactly `±(768, 9984)` — which is
  `1/2020 − 1/2019`. So they are seated correctly, on the wrong host. That
  is a `do_gather` target choice and not this section; run80's own block
  refuses any answer that does not put seven in each.
- **The two gates above the arm are read, not diffed.** `Build::train`'s
  outer `(BuildTypeData +0x1e4 & 0x200) == 0` — `obj_masks`, by the type
  record — skips the whole exit block when set, and this crate does not
  model it; no capture on disk has a trainer that takes it, and every
  building any capture trains from ejects. The `gather_inside` arm on the
  non-university side is likewise unmodelled. The falsifier for both is a
  `BUILDS`+`UNITS=3` window over a **full** university: a fifteenth scholar
  trained at one already holding `gather_max` must appear on the exit ring
  within a block, and must not if the outer gate is what actually fires.
- **`leaders.list[who] |= 0x2000000`** is not modelled. It is set by
  `Unit::work`, `do_gather`, `check_idle`, `close`, `come_out`,
  `kill_current_order`, `Wall::init`, `City::assimilate` and a dozen more —
  a repaint flag on every state change, and nothing this simulation reads.

**What the forced `CHAR_DEFAULT` actually resolves to** (2026-09-18, item
340). ~~The seating leaves the scholar on an idle variant~~ — wrong, and it
was this map's next word. `Guy::init_real@005db6b0` sets `guy_flags & 0x80`
for a `TypeIndex` `0x34`/`0x35` figure and for no other, and
`Guy::set_anim@005da300`'s arm on that bit turns the variant the roll just
chose into an **offset**: the head of the host's inside chain plays
`variant + 0x19` — `SCHOLAR`'s four `Scholar Teach` slots — and everyone
under it `variant + 0x1d`. So the draw this section landed is spent on
choosing *which teach animation*, not on an idle, and the clock it starts
runs 30 to 105 frames rather than an idle's 33 to 232. `docs/ANIM.md` §4.11
has the arm, the `is_peasant` correction that goes with it, and run98's
value diff — East Indies' own first scholar, `cur_anim 25, end_time 30` on
the frame it sits down. Great Lakes **8374 → 8382**.

**And the chain is joined before the seating reads it.**
`Object::insert_inside@00647e90` is `go_inside`'s first statement, so by the
time the block above runs the unit is already in the host's `inside_down`
list and can ask whether it heads it. This crate pushed the captain onto
`buildings[b].garrison` *after* the seating, which made the game's first
scholar a student of nobody; the push now happens first.

## Garrisons, continued (§6.6 – §6.8)

### 6.6 Ejecting a building — `eject_contents`, `process_ejection`

`Object::eject_contents(kill_if_stuck, only_type, keep_orders, clear_first)`:
for a building that is not a hangar (`!can_carry(AIR)`), outside the editor,
with no type filter, it **defers**: `build_masks |= 0x4000` and returns. The
immediate path (hangars, transports, a filtered eject) walks the chain,
`come_out`s each, and kills the ones that find no exit when
`kill_if_stuck` (a dying building), or stops at the first otherwise; peasants
leaving a dying city become militia if the owner has the tech, else walk to
the nearest friendly city.

`Build::process_ejection`, from `Build::process` for an active building with
`0x4000`, and from `Objects::process_all` for a **dead** building slot whose
`hold_frames` is still counting: `inside_down < 0 → clear 0x4000`; else
`come_out(captain of the head)` — **one squad per frame, from the head,
FIFO** — and a dead building bumps `hold_frames` so it stays on the list until
empty. There is no failure handling here: a stuck head is retried every frame.

When a building dies (`Object::close`): a non-hangar with a garrison →
`eject_contents(1, −1, 0, 1)` — deferred, one squad a frame, each that finds no
spot dies; a hangar → `kill_contents`; `hold_frames = 0x1e`. `Build::swap_team`
(capture) → `eject_contents(1, −1, 0, 1)` **before** the swap, so the garrison
leaves under the old owner; a missile silo's missile dies. A `FLAMETHROWER`
hit on a building → `eject_contents(1, …)` (deferred), and against a city
`x_size/2` citizens are spawned to die (`docs/COMBAT.md` §7.1 step 9). A dock
hit by a fire raft or an air attacker ejects its ships. The eject-all order
(`Group::action_eject_all`) is the same, peasants-only or everything.

### 6.7 Inside: what a garrisoned unit does, and the heal

`Unit::process`: `process_healing` runs for every unit; then `inside_up < 0`
takes the on-map branch (the targeted decay, attrition and supply, `work`,
the figures' physics); **else**: the cover/cloak bits cleared, and an AI's
unit inside a **calm** city (neither `CITY_UNDER_ATTACK` nor `CITY_ALARM`)
comes out by itself every 32 frames. So inside there is no movement, attack,
gather, supply, attrition, LOS or physics; combat never targets it
(`docs/COMBAT.md` §12.1); it is off the selection and the grid.

**The garrison heal** (the `inside_up ≥ 0` branch of `Unit::process_healing`;
`docs/SUPPLY.md` reads the supply branch): a captain, not air, damaged,
inside a **building** (not a transport):

```
period = UNIT_HEAL_RATE[get_heal_level()]                      # 20 / 15 / 10 / 5 by HEAL_FASTER_1..3
the building is REDFORT, or the owner has the Red Fort active:  period = period × 100 / (RED_FORT_HEAL + 100)    # 500 → 20 → 3
a ship in a dock:                                              period = (2 × period + 2) / 3
period = max(period, 1)
(o + frame) % period != 0 → return
eligible = the building is(type.where, 0)   or   a city   or   is_fort or is(TOWER, 0)
eligible → repair_damage(1, 1, 1); healing = max(healing, period)
```

A city heals anyone inside; a barracks only what a barracks trains; a market,
senate, university or wonder heals nobody. A unit in the barracks that
trained it, no techs, heals one step every 20 frames.

### 6.8 Training, hangars, transports — surveyed

`Build::train` puts every trained unit inside and then, for a non-hangar,
leaves it in if the rally point is the building itself and `num_inside(0) ≤
(limit or 10 if 0)`, else `come_out`, and **a unit that cannot get out of a
full building dies** (`docs/PRODUCTION.md` owns the rest). `can_carry(AIR)`
= `AIRBASE | AIRCRAFTCARRIER | MISSILESILO` (strict) — the hangars, which use
`kill_contents` instead of ejection and `Object::do_launch` (`build_masks &
8`) with `FRAMES_BETWEEN_LAUNCHES`; transports (`carry`, `carry_size`) share
the chain through `do_board`. Their own document.

---

## 7. Capture

### 7.1 Who is capturable, and who tries

`BuildData::check_capture_eligible(B)`: the type `can_capture` (`is_city`,
i.e. the `VILLAGE` lineage), the building is **active**, and `health_level`
— `left = clamp(hits − damage, 0, hits)`; 0 at `≥ 90 %`, 1 `≥ 75 %`, 2 `≥
50 %`, 3 `≥ 25 %`, 4 `≥ 10 %`, 5 `≥ 1`, 6 `< 1` — is **6 for a city building**
(`flags & 0x20`: zero hits left) or **> 4 for a village** (under a tenth).
`Object::take_damage` clamps a city at `damage = hits` and flags `0x10`
(`docs/COMBAT.md` §7.2 step 7); a city never dies, it sits at zero until
captured. `check_capture_health` is the last line alone, called by nothing in
the sim.

`Build::check_capture(B, o, who)` is called with an enemy **unit** as the
would-be captor from four places: **`Object::do_damage`** after every hit on
a capturable building by a non-owner (`docs/COMBAT.md` §7.1 step 9 — every hit
is a capture attempt); **`Build::process`** every 64 frames phased by `o`,
when eligible, with the nearest enemy unit within 16 tiles (`find_unit(…,
SEARCH_ENEMY, 0xc00, FILTER_DOMAIN 0)`) — a zero-hits city is re-tested about
twice a second unprompted; **`Group::action_attack`** on a capture-eligible
target — the attack order becomes a move-to-capture; and **`Object::valid_target`**
(`docs/COMBAT.md` §12.1), which calls it during target validation unless the
attacker is a missile, and then admits the target only to a `VEHICLE` **and**
`WAR_MACHINE` unit under a mandatory attack order on exactly it.

### 7.2 The test — `Build::check_capture`

`O` the owner, `A` the captor, `U` the unit. Returns 1 iff ownership changed.

```
!check_capture_eligible → 0
inside_down ≥ 0 → build_masks |= 0x4000; return 0          # a garrisoned city is never captured: it is emptied first, one squad a frame
A is O or O's ally (unless O is dead) → 0;  A dead → 0
U: land domain, a unit, not a decoy, attack() != 0, not a missile — else 0
frame − city.capture_stamp ≤ 74 → 0                        # a 75-frame grace after a capture
R = city ? CITY_CAPTURE_RADIUS (10) : UNIT_RESPOND_RANGE (12) tiles;  dist_max = R × 192;  cells = ⌈R / 4⌉
land = B's cell's land id
def_base = (frame − capture_stamp < 900) ? city.capture_strength : 2
mine = U.get_capture_value()                                # the triggering unit, counted once, never distance-tested
per_player[A] = mine;  per_player[O] = def_base;  attackers = mine;  defenders = def_base
for every object X in the cells within circle_radius[cells] of B's cell, same land id:
    skip B, skip U, dead, sea or air domain, decoys, !valid_filter(8), vector_dist(X, B) > dist_max
    v = X.get_capture_value()                                # a unit: max(1, min(uber_size, living figures)), 0 for a decoy or siege; a building: 1
    v != 0 and X is O's building: v += num_inside(1) + (is_fort ? 12 : 6); revealed to A
    per_player[X.who] += v
    X is A or (A's ally and O's enemy) → attackers += v;   X is O or (O's ally and A's enemy) → defenders += v
defenders ≥ attackers → 0
```

**If the attackers win**, a city building: the captor is re-chosen as the
allied player with the **most capture value in the radius** (strict `>`, so
lowest index on ties; `A` if all zero); then an allied founder, or failing
that an allied race-owner, who is alive takes it instead (`taker = founder`
if `founder ∉ {O, W}`, allied with `W`, alive; else `race` likewise; else
`W`); `newcity = Cities::capture_city(taker, B.city, O)` (§7.3);
**`cities[taker][newcity].capture_strength = mine`** — the triggering unit's
own capture value (settled in the listing: `[ebp−0x44]` is written once, with
`get_capture_value()`, and stored after the call), which is what defends the
city for the next 900 frames against a recapture. A village: `swap_team(A)`,
`activate(0, 1, 0)`, the old one closed; a failed swap destroys it instead.

The defending side's **buildings weigh more than units**: each of the owner's
buildings in the radius counts `7 + garrison`, a fort `13 + garrison`; a
third party's counts 1 and on neither side. Siege counts nothing.

**Gaia in the tally — a guard, not a reading.** The loop above walks the
cells' object chains, which carry gaia's animals as well, and
`per_player[X.who]` is a **local `int` array** in the original: an animal
that reached it would write past the end. Nothing in the loop bounds the
leader (unlike `find_unit` and `valid_target_const` — `docs/ANIM.md` §6.1),
so either `valid_filter(8)` rejects animals for some other reason or the
overrun is real. The filter is the block at `0067de47` in
`Search::valid_filter`'s jump table; it turns on a type field `+0x1e8` that
is unnamed in the export. The sim skips gaia here on the leader bound, which
cannot change the outcome but is not derived. *Capture:* a contested capture
with an animal inside the radius, `CITIES=5` and `UNITS=3` over the window.

### 7.3 The hand-over — `Cities::capture_city(A, c, O)`

1. `O.cities_lost++`, `A.cities_captured++`; `was_capital = old.city_flags &
   0x10`; a Persian second capital check.
2. `r = (min(get_radius(O, type), 64) + 3) / 4` cells.
3. `n = citybuild.swap_team(A)` (§7.5); `n ≥ 0` → `activate(1, 1, 0)` (a
   transfer: `race`/`founder` untouched, the stamps set to now), `newcity =
   new.city`, `base = CITY_PLUNDER_PER_LEVEL × (level − 1)`.
4. `newcity == −1` (the swap failed): every non-`e` member goes on the close
   list, `base += 25` each. Else **`City::capture`** (§7.4); then for each
   member whose type lacks `build_flags & 0x2000` (civilian): a FARM or
   GRANARY taken by the Lakota is closed; an active one `swap_team(A)`,
   `activate(1, 1, 0)`, `base += 25`; inactive ones are disbanded;
   **`City::find_buildings(newcity)`**; and, **only for a self/ally hand-over**
   (the defeat path, §8.5), `O`'s units within the radius whose nearest
   friendly city is the new one are `swap_team`ed too.
5. **Plunder** (§8.3).
6. The close list: inactive → `disband(0)`; active → `close(5)` (silent
   transfer); the new ones `mask_me(1, REGEN_SIMPLE)`.
7. **Capitals**: `was_capital`, `O` lacks the Forbidden City, and
   `find_capital(O, skip c)` finds no capital held by `O` → `Leader::
   lost_capital(O, A)` (§8.5); the new city has `0x10` and `A` held none →
   `recapture_capital(A)`.
8. **`City::close(old, A)`** (§8.4) — `lost_a_city`; the old building `close(5)`.
9. The new city building: `update_hits(0)`; **`damage = hits(0) − 10`** —
   **a captured city is handed over with ten hit points**; `update_los`;
   `calc_pop_cap(A)`.

### 7.4 `City::capture(new, old, c, A, o)` — the record

```
new.who = A; new.o = o; x, y
A.pop and world_pop: − get_pop_value(new) before, + after; new.pop = old.pop
attack_stamp = raid_stamp = frame; reg, scouted, founder copied
new.race = (O == A or allies(A, O) or A.leader_flags & 0x200000 or A has GLOBAL_GOVERNMENT_BONUS) ? A : old.race   # else foreign: unassimilated
plundered, was_capital_flags, traded_with, conquest_node copied
f = old.city_flags & 0xf13f;  not (O == A or allied) → f |= 0x100 (captured, unassimilated);  name kept
f & 0x10 (a capital): was_capital_flags |= 1 << O;  f & 0x4000 → f |= 0x8000
    unless O is Persian with a founding capital: every unfinished SENATE of O is disbanded(1)
f &= ~(0x10 | 0x4000)
was_capital_flags & (1 << A): f |= 0x10; clear the bit; (f & 0x8000 and A == founder → (f & 0x3fff) | 0x4010: the original capital restored whole); race = A
new.city_flags = f;  building.damage = hits(0) − 10
```

**The recapture modifier** (`docs/COMBAT.md` §6 step 28): `ObjectData::
get_damage` compares **`cities[T.who][T.city].race == A.who`** — the nation
the city is assimilated to, not `founder`. While a stolen city is
unassimilated its previous owner does ×2 to it; once it assimilates the bonus
is gone; a founder attacking an assimilated ex-city of theirs gets nothing.

### 7.5 `Build::swap_team(B, A)`, `Wall::swap_team`

`Build::swap_team`: a MISSILESILO → `kill_contents`; **`eject_contents(1,
−1, 0, 1)`** (the garrison out, under the old owner); not a LIBRARY →
`clean_queue(1)` (**the queue is cleared**; a library's research is kept,
`docs/PRODUCTION.md`); `n = Wall::swap_team(A)`; `n ≥ 0` → `recharging`,
`founder`, `max_age = max(both)`, the gather list and a farm's slot carried
over; a SPACEPROGRAM reveals the map to `A`. `Wall::swap_team`: `t =
current_upgrade(A, B.type)` — **the building becomes A's current upgrade of
that type**; `n = init_build(A, t, x, y, restore = 1)`; `build_masks`,
`ever_seen*`, the `flags` (`0x40` excepted), `job_counter`, `job_counter_2`,
`constr_time`, `construct_hits`, **`damage`**, `visible` copied;
`update_hits(0)`; `update_los`. The `damage` copy is why `capture_city` resets
the city to `hits − 10` afterwards; other captured buildings keep their damage.

---

## 8. Assimilation, the city heal, plunder, loss

### 8.1 Assimilation

A city is unassimilated while `CityData.race != who`. `BuildData::
is_unassimilated(B)` = `city ≥ 0 && cities[B.who][B.city].race != B.who &&
(B is the city building || !(build_flags & 0x10))` — the city, or any member
that is not buildable outside one. Readers: `BuildData::can_make` (nothing
can be queued, `docs/PRODUCTION.md`), `TypeData::get_cost`, `get_garrison_arrows`'s
Maya exemption (`docs/COMBAT.md` §8.6), `do_garrison` (§6.4), the plunder
protection (§8.3), `get_first_library`, `get_building_cities`, the AI and the
interface.

**The timer** is a frame stamp, `assimilation_timer`, set to the capture frame.
`Build::process`, every frame for an unassimilated city building:

```
has_general(THECITIZEN) ≥ 0:  assimilation_timer += 1 − THECITIZEN_ASSIMILATION_SPEED (4)   # the stamp walks back three a frame
e = frame − assimilation_timer
Turks:              e = (TURK_ASSIMILATE (200) + 100) × e / 100
owner == founder:   e = (REASSIMILATION (300) + 100) × e / 100
e ≥ ASSIMILATION_TIMER (2000) → City::assimilate
```

So 2000 frames (133 s), 667 for the Turks, 500 for a founder retaking their
own, 500 with the Citizen; the Turks and the founder compound. No other
nation bonus exists. `City::assimilate`: `city_flags &= ~0x100`; `race = who`;
**`Region::fix_borders`** (the territory changes hands); a Chinese owner with
`CHINESE_LARGE_CITIES` → `set_type(TOWN)` — it becomes a Large City outright,
**without a `mask_me`**, so its radius mask stays at the old size until the
next remask;
a library's queue re-registered; a senate with no living patriot re-trains
one.

### 8.2 The city heal

`Build::process`, a city building with `damage != 0`, **`race == who`** and
`city_flags & 2` clear: every `CITY_HEAL_RATE` (4) frames, phased by `o`,
`repair_damage(get_level(), 0, 1)` — **a city heals its level in hit points
every four frames, and an unassimilated city does not heal**. An AI owner
also orders a nearby idle citizen to repair a city past half damage. There is
no other passive building regeneration.

**Confirmed in a logged run (2026-08-20), both terms.** `cheat damage`
(`docs/ORACLE.md`) put 599 damage on a fresh Small City, and `BUILDS=6`
logged the object's `damage` every frame: it fell by **one every four
frames**, on frames 181, 185, 189, 193 … Five buildings of distinct types
were then added with `cheat add` until the city levelled — `myhits` grew
1200 → 3125 — and the same damage cheat gave **two every four frames**
(1039, 1037, 1035, 1033 …). So the period is `CITY_HEAL_RATE` = 4 exactly and
the amount is `get_level()` exactly, one hit point per level. The phase, and
the claim that an unassimilated city does not heal at all, were not
exercised.

### 8.3 Plunder

**On capture** (`capture_city` step 5): `base == 0` and not a capital →
nothing. **A city that was itself captured and not yet assimilated
(`0x100`), or captured less than 4501 frames ago, yields nothing** — no
ping-pong plunder. Otherwise: not the first capital loss → `take = hero ?
THEDESPOT_PLUNDER × base / 100 : base` (a Despot or Spitamenes patriot of `A`
near the city), into `A`'s lowest bucket among the goods FOOD..WEALTH both
players can use; **a Russian victim with `RUSSIAN_PLUNDER_STEAL` receives the
plunder of its own city** and the captor only the hero cut. **The first
capital loss**: `O.leader_flags |= 0x400000`; `base = max(base,
CAPITAL_PLUNDER (500))` (team styles 1, 2, 9: `CAPITAL_PLUNDER_ASSASSIN ×
max(1, nations − eliminated)`), paid into **every** usable good.

**On a kill** (`Build::plunder(B, o, who)`, from `do_damage`'s building kill
by a non-air, non-splash, non-allied attacker — `docs/COMBAT.md` §7.1 step 8):

```
who == owner, Lakota, LAKOTA_RAZE_PRICE != 0 → nothing
v = plunder_value (PLUNDER); g = plunder_good; v == 0 or g < 0 or !type_avail(who, g) → nothing
German owner, MINE: v = (100 − GERMAN_MINE_COST) × v / 100
own building: not activated (0x1000) → nothing;  TEMPLE with Tikal: v = (100 − TIKAL_TEMPLE_COST) × v / 100
not activated: v = v × min(1, job_counter_2 / construct_time(0))       # FLOAT in the original
own, finished: standing in a city of another race → v = PLUNDER (100 %) × v / 100
an enemy's finished: v = PLUNDER × v / 100
Aztec attacker: v = (AZTEC_PLUNDER + 100) × v / 100
hero cut: a Despot/Spitamenes near → hero = THEDESPOT_PLUNDER × v / 100
Russian owner with RUSSIAN_PLUNDER_STEAL: bucket_add(owner, g, v), bucket_add(who, g, hero);  else bucket_add(who, g, v + hero)
```

`bucket_add` is the per-frame income bucket (`docs/ECONOMY.md`), not the
stockpile. The construction-progress fraction is one of two floats in this
mechanic (the other is the cancel refund, §9.2); an integer `v × min(jc2, ct)
/ ct` agrees except within rounding of an integer boundary.

### 8.4 `City::close` and `Leader::lost_a_city`

`City::close(old, captor)`: `city_num--`, `world_cities--`, `city_mine--`,
`lost_city_stamp = frame`; `pop −= pv`, world and region too, `reg_cities--`;
armies released; `city_flags &= ~1`; **`Leader::lost_a_city(O, was_capital,
race == who, captor)`**; every member: `city = −1`, a library's queue moved to
the next library or cleared, `decrement_stats` unless `e`, `city_down = −1`,
and a surviving non-city member **re-finds its city**; `calc_pop_cap`;
caravans re-path. `Cities::close_city` trims `city_mark`.

`Leader::lost_a_city`, keyed on the lobby's `elimination` byte (the mapping to
the lobby's strings is not established; read as numbers): `1` with a captor
— a `GLOBAL_GOVERNMENT_BONUS` holder among the captor and its allies →
`defeat(1, captor)`; `2` — `defeat(2, captor)` unless a Persian still holds a
capital; `3` — `defeat(3, captor)` under a condition the decompiler lost;
**default (`0`): survive iff `leader_flags2 & 1` or `city_num != 0`, else
`defeat(0, captor)` — losing the last city is elimination.**

### 8.5 Capitals and defeat

`Leader::lost_capital(O, A)`, once: `lost_capital_timer = 1`; `m = ((O's team
territory + w) × 256) / (A's + w)` with `w = land_cells / num_nations`,
clamped `[0x80, 0x200]`; if both have cities and `A` has fewer, `m = m × (O.
city_num + 2) / (A.city_num + 2)` clamped `[0x80, 0x300]`; `lost_capital_
modifier = m`; `lost_capital_stamp = frame + (lost_capital_stamp << 8) / m`.
`recapture_capital(A)`: `timer = 0`, `stamp = ((stamp − frame) × m) >> 8` —
the time spent capital-less carried as a debt that decays one frame per frame
while the capital is held. `Leader::process_elimination`: under `elimination
== 1`, a player whose `lost_capital_timer` is set and `frame − stamp ≥ Game::
retake_capital(p)` (`RETAKE_CAPITAL`, map-size scaled, `× modifier >> 8`) is
`defeat(1, holder)`. How the base countdown is first seeded is not
established.

`Leader::defeat(type, by)` → **`defeat_by`**: unless `elimination == 3`,
**every live city of the loser is `capture_city`'d** — to its founder or race
owner if that player is `by`'s living ally, else to `by` — and an undamaged
one assimilated at once (never, after the `hits − 10`); every other building:
inactive → `die(3)`; an active military trainer (not fort/tower/lookout/city)
inside one of `by`'s cities → `swap_team(by)`; the rest `die(3)`. Then the
flags, `clean_queue` everywhere, orders cleared, planes die, the stockpile
tributed to allies.

---

## 9. Destruction, refunds, repair, the building's own attrition

### 9.1 `Build::close`, `Wall::close`

`Build::close(reason)` (`5` = silent transfer): `attack != 0 → defense--`; if
alive: a LIBRARY remembers whether it was first, else `clean_queue(refund)`
(`docs/PRODUCTION.md`); **active** → `decrement_stats` (when `e` or in a
city), a gather type's `gather_slots −= gather_max`, the dock/market/temple
count down; a rubble tile bit; **`Wall::close`** — `mark_behind_tiles(0)`,
`start_me(0)` (release the footprint reservation), started → **`mask_me(0,
REGEN_SIMPLE)`** (unmask the footprint and, for a city, its radius), not
active → `num_queued[type]--`, `Object::close` (the garrison, §6.6); then a
non-city → `remove_from_city`; **a city → `remove_unbuilt_city` if never
started, `Cities::close_city`, `Region::fix_borders`, `mask_me(1,
REGEN_NONE)` on every other alive active city of every player (the territory
recompute), `calc_pop_cap`**; a fort → `Forts::close_fort`; a dock →
`close_dock`; TEMPLE → a border recompute; an oil well → `close_oil_well`; a
**wonder** → `Wonders::close_wonder` and its one-offs undone (Colossus
`calc_pop_cap`, Colosseum/Eiffel/Tikal `fix_borders`, the Space Program's
reveal lost, the Forbidden City → `lost_capital` if the capital is elsewhere);
a LIBRARY's research moved to the next library; `BuildQueue::close`; `flags
= 0`. `Object::die` → `close`, then `hold_frames = max(1, longest in-flight
projectile + 1)`.

### 9.2 Refunds — `Object::disband(full)`

Every citizen of the owner whose current BUILD order targets this building
loses its orders. **Lakota** with `LAKOTA_RAZE_PRICE != 0` → `full = 1`. `full
== 0` and the building is not active and `job_counter < type.time(who)`:
per good `T`, `refund_T = (int)((float)cost_T × (float)(ct − jc2) /
(float)ct)` with `ct = construct_time(0)` and `cost_T = get_cost(T, …,
orig_type)` — **the fraction of the price not yet built, in float**; `full`
→ the whole price. **`job_counter_2`, never reduced by damage, is the
numerator — a damaged site refunds what was spent, not what is left.** Then
never started → `close(0)` (no death, no rubble); else `die(1)`. The
queue-item refunds (`refund_cost`, `unpay_cost`) are `docs/PRODUCTION.md`'s.

### 9.3 Repair — `Unit::do_repair` → `Build::repair_damage`

`Build::repair_damage(amount)`: `damage = clamp(damage, 0, hits(0))`,
`damage_frac = 0`, `damage = max(0, damage − amount)`. `Unit::do_repair`, the
REPAIR order's step: target alive, same owner or mutual allies, a building,
**active**, **not under attack** unless forced, the tile's territory owner not
an enemy; adjacent; then

```
period (×256) = (helpers + 1) × ((construct_time(0) << 9) / (hits(0) × ACCEL_CONSTRUCT))
a city: period ×= 2;  a captured city (race != who): ×4 instead
Koreans with KOREAN_REPAIR: period = (100 − KOREAN_REPAIR) × period / 100
not Koreans-under-fire: is_under_attack → period <<= 2;  build_masks & 0x10 → <<= 2 again
helpers++
amount = frame × 256 / period − (frame × 256 − 256) / period            # one hit point every period/256 frames
each good T: cost_T = get_cost(T); one unit of T charged whenever hits_left crosses a multiple of hits/cost_T (float) — a full repair costs the full price; cannot pay → the order dies
repair_damage(amount)
```

A full repair at one repairer takes **twice the build time**, ×2 for a city,
×4 for a captured one; extra repairers are harmonic like builders (the shared
`helpers`). No building regenerates on its own except a city (§8.2).

### 9.4 Cancelling a start — `kill_competing_buildings`

Two players (or one player twice) may place on the same tiles; the first
`Wall::start` disbands every other not-started building on a double-placed
tile with a full refund. A started one stops the sweep.

### 9.5 A building in enemy territory bleeds

`Wall::process`, on the 16-frame phase by `o`, owner's
`disable_building_attrition == 0`, and then — **unless the lobby's
`rush_rules` are on and `Game::war_allowed` is still false — only on the
32-frame phase** (the decompile reads `if (rush_rules == 0 || war_allowed)
require (frame + o) & 0x1f == 0`; a first draft had the two cases inverted,
the second reading caught it): the territory owner `t` of the building's
tile (a dock: `check_enemy_adjacent`); `t ≥ 0`, `t != who`, not allied —
**not started → `Object::disband(0)`** (a ghost in enemy land is removed,
full refund); **started → `take_damage(8, 0, 1, …, attrition = 1)`** — eight
hit points **every thirty-two frames** in a normal game (sixteen under rush
rules before war is allowed), through `docs/COMBAT.md` §7.2 with
the attrition flag (so `docs/ATTRITION.md`'s unit rules do not apply; the
call is unconditional on supply). The building half of the mechanic
`docs/ATTRITION.md` specifies for units.

---

## 10. Constants

All loaded by `Constants::init@00569a90` with `get_item` (plain integer)
unless noted. Shipped values from the user's `rules.xml`; `rondata` checks them.

| `rules.xml` | scale | shipped | read by |
| --- | --- | --- | --- |
| `ACCEL_CONSTRUCT` | `get_fraction(…, 100)` — hundredths | `1/1` → 100 | `do_build`, `do_repair` |
| `CAPITAL_BUILD_TIME` | percent | 300 | `update_construct_time` (a player with no city) |
| `BUILDING_HP_UPGRADE` | percent per level | 10 | `update_hits` |
| `SENATE_HP_BONUS` | percent per city level | 35 | `update_hits` |
| `TEMPLE_UPGRADE_HP[5]` | percent | 25 50 100 150 200 | `update_hits` (a city with a temple) |
| `CITY_CENTER_RADIUS`, `CITY_CENTER_POP_RADIUS` | tiles | 20, 4 | `get_radius` |
| `INDIANS_CITY_RADIUS` | tiles | 4 | `get_radius` |
| `CITY_BUILDINGS`, `METRO_BUILDINGS` | kinds | 5, 9 | `can_upgrade` |
| `VILLAGE_POP` | pop | 0 | `CityData::pop_cap` |
| `CITY_SPACING`, `RELAX_CITY_SPACING` | tiles | 24, 6 | `blocked_location` |
| `FIRST_CITY_NEAR_COAST` | tiles (÷4 → cells) | 16 | `blocked_location`; 0 disables the colonise rule |
| `FORT_SPACING`, `FORT_TO_ENEMY_CITY_SPACING` | tiles | 12, 32 | `blocked_location` |
| `FARMS_PER_CITY_BASE`, `FARMS_PER_CITY_LEVEL`, `EGYPTIAN_FARMS_PER_CITY_BASE` | farms | 5, 0, 7 | `get_farm_limit` |
| `BANTU_CITY_LIMIT`, `PYRAMIDS_CITY_LIMIT` | cities | 1, 1 | `get_city_limit` |
| `DUTCH_FORT_PLACEMENT` | flag | 0 | `blocked_tcoord` |
| `CITY_CAPTURE_RADIUS` | tiles | 10 | `check_capture` |
| `UNIT_RESPOND_RANGE` | tiles | 12 | `check_capture` (a village) |
| `RECAPTURE_CITY_MODIFIER` | 8.8 | `2/1` → 512 | `get_damage` step 28 |
| `CITY_PLUNDER_PER_LEVEL`, `CAPITAL_PLUNDER`, `CAPITAL_PLUNDER_ASSASSIN` | goods | 100, 500, 500 | `capture_city` |
| `PLUNDER`, `AZTEC_PLUNDER`, `THEDESPOT_PLUNDER`, `RUSSIAN_PLUNDER_STEAL`, `GERMAN_MINE_COST`, `TIKAL_TEMPLE_COST`, `LAKOTA_RAZE_PRICE` | percent / flags | 100, 100, 100, 1, –, –, 0 | `Build::plunder` |
| `ASSIMILATION_TIMER` | frames | 2000 | `Build::process` |
| `TURK_ASSIMILATE`, `REASSIMILATION` | percent bonus | 200, 300 | `Build::process` |
| `THECITIZEN_ASSIMILATION_SPEED` | frames per frame | 4 | `Build::process` |
| `CITY_HEAL_RATE` | frames | 4 | `Build::process` |
| `RETAKE_CAPITAL` | map-scaled frames | – | `Game::retake_capital` |
| `CHINESE_LARGE_CITIES` | flag | 1 | `activate`, `assimilate` |
| `MAYA_BUILDING_SPEED`, `VERSAILLES_BUILDING_SPEED`, `TOBACCO_BUILDING_SPEED`, `BRITISH_AA_SPEED`, `DUTCH_FORT_SPEED`, `ROMAN_FORT_SPEED` | percent | 20, 0, 10, 33, 0, 50 | `update_construct_time` |
| `HANGING_GARDENS_BUILD_TIME`, `THEPRESIDENT_BUILDING_SPEED`, `IROQUOIS_QUICK_SENATE` | percent / flag | 0, 33, 1 | `construct_time` |
| `KOREAN_BUILD_UNDER_FIRE`, `KOREAN_REPAIR` | flag / percent | 1, 50 | `do_build`, `do_repair` |
| `MAYA_BUILDING_HP`, `ROMAN_FORT_HP`, `TAJ_BUILDING_HP`, `RED_FORT_FORT_HPS`, `NUBIAN_HIT_POINTS`, `TIKAL_TEMPLE_HP` | percent | 25, 0, 100, 33, 50, 50 | `update_hits` |
| `TOWER_GARRISON_UPGRADE`, `FORT_GARRISON_UPGRADE` | slots per level | 2, 5 | `get_garrison_limit` |
| `UNIT_HEAL_RATE[4]` | frames | 20 15 10 5 | the garrison heal |
| `RED_FORT_HEAL` | percent | 500 | the garrison heal |
| `UNIT_TRAIN_DISTANCE`, `UNIT_TRAIN_MAX_DISTANCE`, `BOAT_TRAIN_DISTANCE`, `BOAT_TRAIN_MAX_DISTANCE`, `UNIT_DISEMBARK_DISTANCE` | `get_fraction(…, 0xc0)` — position units | 3/2, 5/2, 3/2, 8, 3 tiles | `come_out`'s exit ring |
| `UNIT_GATHER_RESPOND_RANGE` | tiles | – | `do_build`'s next-site search |
| the free-unit and first-slot bonus family (`NUBIAN_FREE_CARAVAN`, `ROMAN_BARRACKS_LEGION`, …, `FOOD_BONUS_FOR_FARM`, …, `KOREAN_CITIZENS[8]`) | plain | – | `Build::activate` (§4) |

---

## 11. What `crates/sim` models, and how

`crates/sim/src/build.rs` — the building type (`BuildType`: footprint, flags,
domain, lineage, `job_time`, `hits`, `garrison_max`, plunder, the tree id),
the construction clock (`construct_base` with its nation/tech factors as a
`BuildMods` input, `construct_time` with the four per-call clauses as
inputs), `do_construct` with the harmonic `helpers`, `construct_hits`,
`update_hits` with the multipliers as a `HitsMods` input, the repair period
and its price, the cancel refund, the building's own attrition.
`crates/sim/src/place.rs` — the tile layer (`TileMask`, four by four per
cell, on `World`), `blocked_site`/`blocked_tcoord`/`blocked_location` with
the `Blocked` verdicts, city and fort spacing, the foothold, the colonise
rule, `get_town`, the city limit. `crates/sim/src/city.rs` — the record,
membership, the radius, `find_buildings`, `check_upgrade`, the pop value and
cap, the capital, `check_capture`/`capture_city`/`City::capture`, the
assimilation tick, the city heal, plunder on capture, `lost_a_city` in the
default mode. `crates/sim/src/garrison.rs` — the chain as a FIFO list on the
building, `can_garrison`, `garrison_limit`, the `do_garrison` gates,
`go_inside`/`come_out`, deferred ejection one squad a frame, the garrison
heal. `lib.rs` wires them into `Sim::tick` in the original's order: units
(`do_build`, `do_repair`, the heal) before buildings (`Wall::process`, the
construction attrition, the capture re-test, the assimilation tick, the city
heal, ejection), then the sites' `construct_hits` refresh.

**Stated simplifications, each recorded here rather than silently made:**

- **Visibility is not modelled.** `was_seen`/`was_really_seen` are taken as
  true everywhere; `BLOCKED_SEEN`/`BLOCKED_UNSEEN` are never returned.
  **The rule that omission drops** (2026-08-31, read for item 113):
  `blocked_site@00636a50` counts, inside the same footprint walk as
  `blocked_tcoord`, the footprint **tiles** whose fog half-cell (`t >> 1` on
  both axes) `was_seen(·, who)` denies, and after the walk returns `0x24`
  when `x_size × y_size / 2 < that count` — with only a Dock (`is(0x1b0)`)
  exempt, and the whole tally skipped for `who < 0`. Landing it moves no
  measured number on any capture, because `was_seen`'s first arm is
  territorial and every footprint an AI considers is inside its own border,
  so it is booked rather than landed blind (`docs/AI.md` §18, queue).
- **The editor relaxations are not modelled.** `semaphore[1] & 8` is always
  clear.
- **The cancel refund and the unfinished-building plunder are pinned
  rationals** — `cost × (ct − jc2) / ct` and `v × min(jc2, ct) / ct` in
  integers — where the original computes them in `float`. They agree except
  within rounding of an integer boundary; `docs/DECISIONS.md` entry 16.
- **Adjacency** for a builder or repairer is "the unit's tile is within one
  tile of the footprint" (`adjacent_to` was not read; the original's
  `swarm_around` walks the unit there). Movement to the site is the caller's.
- **`come_out` searches §6.5's land ring** through `find_nearby_spot`, and a
  trained unit reaches it: `Build::train@0062f9b0` builds the unit at its
  trainer, `go_inside`s it and lets it out (2026-08-27). **Both of §6.5's
  arms are modelled since item 66**, and the branch between them is the
  type's `block_radius` (`+0x240`), not its `big_radius` as this bullet
  first said: a Citizen's `BLOCK_RADIUS` is 1, so every unit any capture
  trains takes the `FILTER_NOT_ME` arm — the ring free of other units,
  then the *same* ring with `nocoll`, then a **refusal** that keeps the
  unit inside. The `block_radius == 0` arm is the one that doubles the
  ring and falls back to the building's own position; its `FILTER_ALL`
  filter goes through the general test `docs/COLLISION.md` §9 does not
  model, so both of its passes accept. Still not modelled: the `BOAT_*`
  and disembark rings, and the gather point's turn. **The default bearing is
  due south**, `617c33`'s own constant (§6.5.1) and, before it was found,
  diff-backed by run10's five AI-trained citizens at `(42360, 17208)`, on the
  inner radius directly south of London. `train`'s own arms before the exit —
  a gather-inside building that keeps its worker, the dock's boat count, the
  owner's text bubble — are not modelled either; every trainer here lets its
  unit out. The one-squad-a-frame cadence and the FIFO order are kept. **A
  squad's members each run the search for themselves, around the captain**
  (§6.5.1): the `o_down` recursion at `00617c10:535`, and then the unit-host
  arm, which `Sim::come_out_unit_host_spot` shares with the transport's own
  disembark. run76's three **Bowmen** — guy type 177 is unit record 120,
  Bowmen at 70 hits, not record 121's Archers; the `type 21` that named
  them is the first `type` line of a *marching* unit's record and belongs
  to its order (2026-09-06) — are all three exact, and so are run79's two
  Longbowmen squads, born from the same Barracks 601 frames apart on the
  **same three points**. Two unit types, three squads, nine units: the
  placement depends on `(trainer, captain)` and on nothing else — not the
  frame, not the draw, not the member's own type, not what else stands on
  the map. Not modelled: the
  member's turn to its captain's angle (§6.5.1's open question), and
  `init_unit`'s pre-garrison placement of a squad born on open ground.
- **`valid_filter(8)`** in the capture count is taken as "alive and on the
  map" — what every other filter the combat document read reduces to.
- **The capture attempt inside `Object::valid_target`** (§7.1's fourth caller)
  is not modelled; the attempts on every hit and every 64 frames are. A unit
  that would have captured through target validation captures on its next
  hit instead.
- **The AI branches** (the builder-wanting logic, the auto-repair, the
  calm-city auto-exit, the site values) are not modelled; `leader_flags & 4`
  is true for every player.
- **The per-wonder one-offs of `activate`** are not modelled; the hooks
  are: the free first wonder, the Hanging Gardens, the President and the
  Iroquois senate enter `construct_time` as `BuildMods` booleans.
- **Elimination** is the default mode only (`elimination == 0`): a player
  with no city left is defeated, their cities — none — and buildings handled
  by `defeat_by`'s rules; the capital countdown (`mode 1`) is recorded, not
  run.
- **`find_city_at`'s territory test** (`WorldData::is_city_at`'s "the cell
  owner is nobody, `who` or an ally") reads the world's cell owner the way
  `docs/ATTRITION.md` computes it.

---

## 12. What is not established

- **`BUILDINGS_CREATED_FASTER` (`0x313`) never fires.** §3.2's
  `t × 3 >> 2` step: with every tech granted the 2026-08-20 run's tower
  still landed on exactly `100000 × 7/10`. Either the preq is not one
  `cheat tech all` reaches or it is gated on something else; the other
  steps of the clock are unaffected, and the Tobacco row beside it is
  now diff-backed.

1. **`BUILD_FLAGS`'s column** — the letter ↔ bit rule is read; which column
   feeds `+0x2c0` is inferred from the read order and the DTD, corroborated
   by the shipped footprints. Check: a `RULES=1` log dumps the types by name.
2. **`leader_flags & 4`** (human), **`leader_flags2 & 1`** (no starting
   city?), **tribe bonus 0x17** (the Lakota variant of the capital decision),
   **`city_flags & 2`** (the heal/repair veto — no setter found),
   **`city_mine` vs `city_num`** (always moved together here; a third writer
   must exist). All inferred from use.
3. **The editor-only verdicts and `TData 0x200`**, **`WData 0x100`/`region2`**,
   **`WData.who == −2`**: read `World::set_territory` and the terrain writers.
4. ~~**`COLONIZE_BONUS` (0x2af)** — which tech grants it (`docs/TECH.md`'s
   loaders).~~ **Coinage**, and the answer was in the data: it is the fourth
   `TECHBONUS` of `rules.xml` and its `preq0` names the tech (§2.6.1's own
   paragraph, 2026-09-02, run63).
5. **`has_wonder`'s bit 1** in the Hanging Gardens clause.
6. **The `elimination` byte's values → lobby strings**, and the `mode 3`
   condition the decompiler lost; **the seeding of `lost_capital_stamp`**
   before `lost_capital` scales it (`Leader::init`); **the map-size field
   `retake_capital` divides by**.
7. **`Search::valid_filter(8)`** — what the capture count's filter excludes
   (a jump-table dispatch).
8. **`Build::close`'s reason codes** `0`/`3`/`5`, **`Object::disband(0)` vs
   `(1)`** beyond the refund.
9. **Whether a `come_out` that fails in `process_ejection` can wedge an
   ejection forever** (the head never advances, no kill) — **live since
   item 66**, which modelled the refusing arm: before it the search fell
   back to the building's own position and could not fail. It still takes
   a ring with no passable quarter-tile at all, since the second pass
   ignores collision. Check: destroy a tower whose exits are fully
   blocked, `UnitsSync`/`BuildsSync` per frame.
10. **`Unit::come_out`'s rally-point half** and the two `is()` tests that set
    `LEADER_RECOMPUTE` — `docs/PRODUCTION.md`'s and economy's.
11. **The granary/lumber bonus-table swap** in `activate` (the GRANARY branch
    reads `fishermen_bonus`, the LUMBERMILL branch `granary_bonus`) — recorded
    as the decompile shows it; a retype may be hiding an array alias.
12. **Whether the two floats ever differ from the pinned rationals** (§9.2,
    §8.3). Check: a logged run cancelling a half-built barracks and reading
    the refund.
13. **`CAPITAL_PLUNDER_ASSASSIN` is loaded from the same string as
    `CAPITAL_PLUNDER`** (the second reading's capture reader: a loader bug or a
    shared key) — which is why the two ship equal. Not checked further; the
    team-style branch that reads it is not modelled.
14. **`BuildTypeData::to` is last-writer-wins** (2026-08-20, the loader,
    `docs/DATALAYER.md`): `BuildType::init` writes `B[from].to = this` for
    every non-hero record with a `FROM`, in record order, so the Forbidden
    City (record 117, `FROM Small City`) overwrites the Large City's link and
    the program's Small City `to` is the Forbidden City. Whatever reads `to`
    for the city line reads that. Reproduced as read by the loader; the
    sim's lineage tests walk `from`, which is unaffected.
15. **Behavioural checks worth running**, all cheap under `BUILDS=1` /
    `CITIES=1` per frame: two builders on one site (expect `accel +
    accel/2` a frame — settles the harmonic rule and the unit-before-building
    order together); a second city at exactly `CITY_SPACING` tiles (expect
    blocked, `≤`); a dock with exactly ¾ water; a city on an island with and
    without the colonise tech; a placed-but-unstarted enemy city blocking
    your own (it should not); the garrison heal period in a barracks (20) and
    the Red Fort (3); a captured city's hit points (10) and its heal (level
    per 4 frames) once assimilated; the capture count with one citizen versus
    one tower (tower: 7 — the citizen loses).

    **Status.** The first is **run and confirmed** (§3.2, §3.3): two
    builders give `accel + accel/2` exactly, and the construction clock
    re-bakes on a tech change. The city-spacing one is **run and confirmed**
    (§2.6.2): a city is refused at exactly 24 tiles and starts at 25, on two
    bearings. `cheat add` force-places without calling `blocked_site`, so a
    placement cheat can never test a placement rule on its own; the method
    that works — for this and every other `blocked_site` verdict in the list
    — is `add NEW` at the distance under test, one builder, and watch
    whether the site starts (`flags` 1 → 3) or is disbanded when
    `do_construct` runs the check (§3.3), with a tower site on the same tile
    first as the control. `docs/ORACLE.md`, "A scripted placement test".

    **The garrison-heal one is still open**: no way has been found to put a
    unit inside a building from a script — the cheat vocabulary has no
    garrison verb, and neither a right-click nor the panel's garrison button
    moves `inside_up` off −1 (the attempt is in `docs/JOURNAL.md`,
    2026-08-20; it did establish in passing that a damaged unit merely
    standing near a friendly city heals, one of `Unit::process_healing`'s
    six underived branches — `docs/SUPPLY.md`). The remaining route is
    `Build::train` with the rally point on the building itself (§6.8),
    reached by `cheat finish` on a queued unit.

---

## 12.1 The docs-versus-code pass, 2026-09-05

`docs/audit/2026-09-05-cities-vs-code.md` (Opus reader, Opus adjudication).
About 210 stated rules traced to the code that implements them; twenty
disagree, all twenty confirmed, none struck. The construction clock's ten
steps, the harmonic share, the site's growing hit points, the refund, the
repair period and its price, the level predicate, `find_city_at`'s penalty,
the garrison limit and the `can_garrison` matrix, **§6.5's exit ring, both
arms**, the assimilation arithmetic, the city heal, `Build::plunder`'s
ladder, `blocked_site`'s verdict precedence, city and fort spacing, the
colonise rule and the even circle were all found faithful.

**One row was not a missing rule but a wrong one, and it fired every
frame — fixed, and measured to change nothing (item 233, 2026-09-07).**
The periodic phase is the object number `o` (`Build::process@0061edf0:728`);
`Sim::process_building` and `Sim::process_building_combat` used `frame + b`,
where `b` is the building's index in this crate's `Vec`. The two never
agree: `o` is per player from `BUILD_BASE` (2000, which is **16** mod 32)
and is recycled by `Objects::find_free`, the handle is global from 0 and the
`Vec` only grows. Both now read [`Building::phase`], the twin of
`Unit::phase`, and `a_building_s_periodic_phase_is_its_object_number` pins
it in both directions — on the old code it fired on frames 0 and 32 where
the original fires on 16 and 48.

**What the fix moved: nothing, and that is the finding.** The endpoint at
24,001 (`rondata::diff::endpoint`) reads 80 off / 0 unlinked on East Indies
and 73 off / 7 unlinked on Great Lakes both before and after, and a per-frame
digest of every building's `under_attack`, `damage`, `health`, `recharging`,
`target`, `alive` and `active` and every unit's position and health is
**byte-identical for all 24,000 frames of both scored games**. The reason is
that every branch the phase gates is a no-op in these two games — measured
over the same two walks:

| | East Indies (run54) | Great Lakes (run53) |
| --- | --- | --- |
| object numbers a second handle later reused | **10** | **7** |
| building-frames with `under_attack` set (§1.3) | 0 | 0 |
| building-frames on enemy territory (§9.5) | 0 | 0 |
| building-frames live and damaged (§8.2's heal) | 0 | 0 |
| building-frames holding a target (COMBAT §8.6) | 0 | 0 |
| buildings that changed owner (§7.1) | 0 | 0 |

So the slot reuse the row predicted **does happen** — seventeen times across
the two games — and every periodic it would have mis-phased is one no game on
disk ever arms: nothing is ever shot at, nothing stands in enemy land, no
live building is ever damaged, and no tower ever finds a target. The rule is
right and the correction is unfalsifiable here; the capture that would
falsify it is a **war**, with `BUILDS=6` over a building taking damage, and
`BUILDDATA.damage` on the frames either side of a 32-frame boundary.

**One is cross-confirmed by another document's pass.** `Build::remove_from_city`
does not regenerate the city's roads — reached independently as ROADS' own
R4, from `docs/ROADS.md` §1's caller list. Two readers, two documents, one
missing call.

**The rest are stated, unimplemented or implemented differently, and
unreached** — nothing on disk captures a city or eliminates a player.
Grouped: `Wall::do_construct` does not bump `recharging` and `Build::activate`
does not clear it; `blocked_tcoord` never returns `ROCK` or `NO_OIL` and has
lost the flat-gather `NO_RESOURCES` test; `snap_center`'s oil snap and its
80-offset dock/mine search are absent, and §11 does not record that;
`Wall::swap_team` does not run the new owner's `current_upgrade`;
`num_buildings` counts a finished building that belongs to no city;
`check_upgrade` has no `gain_tech` trigger ~~and `find_buildings` no Civic-tech
trigger~~ (the Civic arm landed with item 661, `docs/AI.md` §63); `capture_city` does not swap the loser's units in the radius on an
allied hand-over; `update_hits`' tower exclusions are a **type identity**
where the original asks `is(TOWER, 0)`, so a Keep takes the Senate bonus
(`build::is_tower` exists and is what `place.rs` and `garrison.rs` use, and
the Nubian `MARKET` clause has the same shape); `update_hits`' garrison eject
has no `can_carry(AIR)` exemption though `Sim::is_hangar` exists;
`Build::activate` lacks the rival-wonder disband, the senate capital move and
`wonders_built`, and §11 does not record them; §2.6.4's `RAZING_TOWN` verdict
is missing; `check_capture`'s building tally admits inactive and
water-domain buildings; and `defeat_by`'s tail is missing — stockpile
tribute, planes, `clean_queue` — with its trainer predicate a hand-written
type list.

Five rows the reader marked `UNSURE:` are left marked, each naming what
would settle it: the plunder gate's `capture_stamp != 0` term, the Despot
cut on the first-capital payment, `check_capture`'s village-radius arm,
`City::close`'s ordering of `lost_a_city` against the members re-homing, and
`kill_competing_buildings` skipping a started rival where the document stops
at it.

## 13. Second reading — landed

Five blind readers re-derived the five sub-areas from the same export;
the verdicts are `docs/audit/2026-08-20-cities.md` and the story — what
was doubly confirmed, what was overturned into the sections above, and
what the reading settled for the first — is in `docs/JOURNAL.md`
(2026-08-20). Every correction it produced is landed in `crates/sim`
with a test.

## 14. A mine is refused by a tile distance, not a cell survey (2026-09-18)

§2.6.7's `NO_MOUNTAIN 0xc` is `calc_gather == 0` for a mine, and a mine's
`calc_gather` is not the survey the camp runs: `00639e40`'s head sends
`0x1a3` (and `0x1a4`, the university) past the circle walk, and the mine's
arm asks `MountainsData::find_nearest` whether a **mountain tile** stands
within `MINE_RADIUS × 0xc0` world units of the footprint's centre. The cell
form — a mountain-centred cell inside `MINE_RADIUS` tiles of the anchor —
passes twice as many sites, which is Great Lakes' frame 8382: eighteen of
`Leader::produce_building`'s spiral draws against the original's nine.

`docs/ECONOMY.md`, "The mine's range", has the predicate, the mining list
that follows it, the reconstruction of a mountain range and what none of it
establishes. `docs/AI.md` §39 has what it moved (item 344).
