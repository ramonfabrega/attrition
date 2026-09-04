# CITIES.md versus `crates/sim` — 2026-09-05

Reader: Opus, docs-versus-code pass (not a blind reading: this reader saw the
document and the implementation, and nothing else).

Checked: roughly 210 stated rules across §1.2–§9.5 — every predicate, formula,
constant and step order in the rules sections. §10's constant table is
re-derived by `cargo run -p rondata` and was not re-checked here; §11 and §12
are read as context (and twice below, §11's *omissions* are the finding: a rule
the code drops that §11 does not record). The overwhelming majority of rules —
the construction clock's ten steps, the four per-call clauses, the harmonic
share, the site's growing hit points, the refund, the repair period and its
price, the level predicate, `find_city_at`'s penalty, the garrison limit and the
`can_garrison` matrix, the exit ring's two arms, the assimilation arithmetic,
the city heal's amount, `Build::plunder`'s ladder, `blocked_site`'s verdict
precedence, city and fort spacing, the colonise rule, the even circle — are
implemented faithfully and are not written down. Twenty rows follow.

One thing worth saying that is not a row: of the 776 functions cited by address
under `docs/`, exactly **one** of CITIES.md's is on the blind list —
`Build::finished@00470e50` (§4.1's thunk). Every other function this document
cites has been entered by some trace, so every row below is `reached`. That does
*not* mean a capture on disk exercises the rule — several rows turn on a city
capture or an elimination, and no capture in
`/Users/rf-studio/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs/`
contains either. Where that is so, the row's "difference shows" says it.

## Rows

### R1 — `Wall::do_construct` does not bump `recharging`, so a building under construction never counts its build frames

| | |
|---|---|
| document | CITIES.md §3.3, "`share = amount / (helpers + 1);  if build_masks & 0x800 == 0: recharging++, build_masks \|= 0x800`" — and the confirmed-run paragraph, "**`recharging` counts up one per frame while building** (0, 1, 2, …), which is the `build_masks & 0x800` step" |
| code | `crates/sim/src/city.rs:1332-1343` (`fn do_construct`) |
| document says | Every frame a builder contributes, the site's `recharging` (`BuildData +0x7a`) is incremented once — once per *frame*, not once per builder — and `build_masks & 0x800` is the once-a-frame latch that enforces it. |
| code does | `do_construct` computes the share, bumps `helpers`, adds to `job_counter`/`job_counter_2` and returns. It touches neither `recharging` nor `gather_bumped` (this crate's name for `build_masks & 0x800`, `crates/sim/src/lib.rs:1026`). The only writers of a building's `recharging` in the crate are the two gather paths — `crates/sim/src/orders.rs:4166` and `crates/sim/src/orders.rs:4221` — and the tower reload, `crates/sim/src/fight.rs:1112`/`1034`. |
| difference shows | `BUILDDATA.recharging` for any site under construction, from the first frame a builder is adjacent. Printed from `BUILDS=6`; the 2026-08-20 tower/barracks run the document cites (§3.3) is exactly the capture that read it. Not currently asserted — `crates/rondata/src/diff.rs:2440-2460` compares only `x_internal`, `y_internal`, the queue and the gather list out of a `BUILDDATA` row, so this diverges silently today. |
| reached | reached (`Wall::do_construct@006434d0` is not in `/Users/rf-studio/ron-audit-scratch/blind-all-68traces.txt`) |

`crates/sim/src/city.rs:1332-1343` is the whole of the progress arm:

```rust
let ct = self.construct_time_of(b);
let bd = &mut self.buildings[b];
let share = build::contribution(amount, bd.helpers);
bd.helpers += 1;
bd.job_counter_2 += share;
bd.job_counter += share;
```

`build::contribution` (`crates/sim/src/build.rs:540-546`) is `max(1, amount /
(helpers + 1))`, which is the document's share exactly; the `recharging` line
that sits between the share and the counters in the document has no
counterpart. The gather side does model the latch — `if
!self.buildings[b].gather_bumped { self.buildings[b].recharging += 1;
self.buildings[b].gather_bumped = true; }` at `crates/sim/src/orders.rs:4165-4168`
— so the omission is specific to the construction path, not a missing field.

### R2 — `Build::activate` does not clear `recharging`

| | |
|---|---|
| document | CITIES.md §4, "**`Build::activate`**, before it: `LIBRARY` → `new_library` (`docs/PRODUCTION.md`); **`recharging = 0`**" |
| code | `crates/sim/src/city.rs:1346-1460` (`fn activate`) |
| document says | Finishing a building resets `recharging` to zero. |
| code does | Nothing in `activate` writes `recharging`; the field keeps whatever it held. |
| difference shows | Compounded with R1 this is currently unobservable (the sim never raises `recharging` during construction). Fix R1 alone and it becomes observable at once as a **tower or city that will not shoot for its first `recharging` frames**: `process_building_combat` decrements `recharging` and returns without firing while it is positive (`crates/sim/src/fight.rs:1032-1035`), so a tower that took 1000 frames to build would sit silent for 1000 frames. Capture: any `BUILDS=6` window over a freshly finished tower, first shot frame. |
| reached | reached (`Build::activate@00623e20` is entered in the traces) |

The two rows are one bug with two halves, and they must land together: R1 alone
regresses combat.

### R3 — every building's periodic phase is the vector handle, not the object number `o`

| | |
|---|---|
| document | CITIES.md §3.7 "`Wall::process`, phased by `o`"; §7.1 "**`Build::process`** every 64 frames phased by `o`"; §8.2 "every `CITY_HEAL_RATE` (4) frames, phased by `o`"; §9.5 "on the 16-frame phase by `o` … `(frame + o) & 0x1f == 0`" |
| code | `crates/sim/src/city.rs:1716` (`fn process_building`), `crates/sim/src/fight.rs:1028` and `:1056` (`fn process_building_combat`) |
| document says | The phase counter is `frame + o`, `o` being the building's object index — 2000–2999 of its owner's array (§1.1). |
| code does | `let phase = frame + b as i64;` where `b` is the position in `Sim::buildings`, a dense vector across all players. The object number lives beside it in `Building::index` (written by `find_free(who, BUILD_BASE=2000, WALL_BASE=3000)` at `crates/sim/src/city.rs:922-924`; the constants are `crates/sim/src/lib.rs:924` and `:926`) and is never used for a phase. Units do it the other way: `Unit::phase` is `frame + self.index as i64` (`crates/sim/src/lib.rs:692-694`). |
| difference shows | Any of the four periodic effects, off by a fixed offset per building. Cheapest: the **building attrition** (§9.5) — a site of player 0's first building has `o = 2000`, `2000 & 31 == 16`, so the original hits it on frames `≡ 16 (mod 32)` and this crate on frames `≡ 0`. `BUILDDATA.damage` on a building standing in enemy land, 16 frames early or late; also the 64-frame capture re-test, which decides *which frame* a zero-hits city changes hands. The city heal (§8.2) survives by luck: `2000 % 4 == 0`. No capture asserts it today — `BUILDDATA.damage` is parsed (`crates/rondata/src/diff.rs:20943`) but not compared. |
| reached | reached (`Wall::process`, `Build::process` both entered) |

`Building::index` is the object number and `b` is not: `init_build` pushes at
`self.buildings.len()` and separately calls `find_free(who, BUILD_BASE,
WALL_BASE)` for `index` (`crates/sim/src/city.rs:921-924`), so the two coincide
only for a single-player game whose first building is at handle 0 — and not even
then, since `BUILD_BASE` is 2000. In the diff harness, where buildings are stood
up from a dump in whatever order the record lists them, the handle bears no
relation to `o` at all. `crates/sim/src/harness_tests.rs:1817` already writes the
rule down correctly — "The building thinks on `(frame + index) & 0x1f == 0`" —
against code that does not implement it. So does one periodic *inside
`process_building` itself*: `regen_roads_due` phases on `frame +
self.buildings[b].index` (`crates/sim/src/roads.rs:171`), one line below the
four that phase on `b`. And the decompile settles which is right — the city
heal at `Build::process@0061edf0:728` reads `(GameAccess::game->frame + (int)*(short
*)&this->field_0xa) % city_heal_rate`, and `+0xa` is `o` (§1.1's field table).

### R4 — `blocked_tcoord` never returns `ROCK` or `NO_OIL`: a building may stand on bare rock, and an oil well off the oil

| | |
|---|---|
| document | CITIES.md §2.5, "`not an oil type: W.flags & 8 → ROCK 3;   oil type: not W.flags & 0x800 → NO_OIL 5`" |
| code | `crates/sim/src/place.rs:313-405` (`fn blocked_tcoord`) |
| document says | Between the `T & 0x200` building test and the `T & 0x80` placed test, one tile is refused with `ROCK` (3) when its cell carries `WData.flags & 8` and the type is not an oil type; an oil type is refused with `NO_OIL` (5) when the cell does **not** carry `WData.flags & 0x800`. |
| code does | Neither test exists. `Blocked::Rock` (`crates/sim/src/place.rs:25`) and `Blocked::NoOil` (`:27`) are declared in the enum and returned from nowhere in the crate — the only two `Blocked` values in that stretch that are returned at all are `Ruins` at `:327` and `Rare` at `:355`. The data is present and named: `crates/sim/src/world.rs:350` is `cell::ROCK = 0x8` (`WorldData::is_rocks@006b4380`) and `:366` is `cell::OIL = 0x800` (`WorldData::is_oil_at@00472af0`), and `World::land_class` already reads both (`crates/sim/src/world.rs:1009-1012`). |
| difference shows | The AI's site picture first: `docs/AI.md` §2.13 step 2 zeroes a cell's base score on a refused `blocked_site(TOWN, …)`, so every rock cell of a map scores as buildable here and does not in the original — a `CITY` record's `ter[6]`/`space[3]` and, downstream, *where* the AI puts its second city. Great Lakes (run53) and East Indies (run63) both have rock. The oil half is only reachable once an oil well is placed, which no capture on disk does. |
| reached | reached (`BuildTypeData::blocked_tcoord` is entered) |

The function walks straight from the `T & 0x4000` block (`:348-356`) to the
`AS_BUILDING` test (`:357`) and then to `PLACED` (`:360`); nothing between them
looks at the cell's flags. The same omission takes out §2.5's last line —
"flat gather type and `LandData::get_amount(tile, good) == 0` → `NO_RESOURCES
9`" — because `gather_verdict` returns `Clear` for a flat type outright
(`crates/sim/src/place.rs:476-478`), so the flat gatherers (farm, oil well, oil
platform) have no resource test at all, at tile level or site level.

### R5 — `snap_center`'s oil and dock searches are unimplemented, and §11 does not record it

| | |
|---|---|
| document | CITIES.md §2.2, "an oil well/platform snaps to the oil cell under or adjacent (the eight `move_x/move_y` neighbours, seen by `who`); a dock, or a mine/woodcutter with a player, tries the 80 `move_x/move_y` tile offsets for the first with `blocked_site == 0` when the clicked tile fails" |
| code | `crates/sim/src/place.rs:84-94` (`fn snap_center`) |
| document says | `snap_center` is three steps: the oil snap, the 80-offset relocation search for a dock/mine/camp, and only then the odd/even centring. |
| code does | Only the third step. The doc-comment concedes it in one line — "The oil and dock searches are not modelled" (`crates/sim/src/place.rs:85-86`) — but §11's list of stated simplifications does not carry it, so a reader of the document alone believes the search is there. |
| difference shows | A dock or camp ordered at a tile that fails `blocked_site`: the original silently relocates it up to 80 offsets away and builds, this crate refuses. The AI places camps and docks through the same `snap_center` (`Leader::produce_building`), so it shows as a missing `BUILDDATA` row or a differing `x_internal`. No capture pins it today. |
| reached | reached (`BuildTypeData::snap_center` entered) |

Only the bookkeeping half of this row is certain: the code knows it is a
simplification and §11 does not list it. Whether the search moves a capture is
untested.

### R6 — `Wall::swap_team` does not run the new owner's `current_upgrade`

| | |
|---|---|
| document | CITIES.md §7.5, "`Wall::swap_team`: `t = current_upgrade(A, B.type)` — **the building becomes A's current upgrade of that type**; `n = init_build(A, t, x, y, restore = 1)`" |
| code | `crates/sim/src/city.rs:2191` (`fn swap_team`): `let n = self.init_build(a, ty, pos, true);` |
| document says | A captured or converted building is re-created as the **captor's** newest researched type in the line — a captured Tower becomes a Keep for an owner who has the Keep. |
| code does | Re-creates it as `ty`, the type it already was. `TechTree::current_upgrade` exists and is used for exactly this shape elsewhere — `crates/sim/src/nations.rs:423`, `crates/sim/src/transport.rs:882`, `crates/sim/src/ai_host.rs:548` — and is not called here. |
| difference shows | `BUILDDATA.orig_type` and the building's `myhits` on the frame after any capture or any `find_buildings` conversion (§5.4), which fires on founding, level-up and every Civic tech, not only on capture. No capture on disk contains a building conversion; the falsifying capture is a `frame_window` with `BUILDS=6` over a `find_buildings` sweep that catches a foreign building — cheapest as a scripted `add NEW` of an enemy tower inside a city radius. |
| reached | reached (`Build::swap_team` / `Wall::swap_team` are not in `/Users/rf-studio/ron-audit-scratch/blind-all-68traces.txt`; some trace enters them, though no capture *on disk* carries a city capture) |

`init_build`'s own `restore` handling is faithful (the farm and terraform arms
are gated on it, `crates/sim/src/city.rs:1002` and `:1013`); it is the type
argument that is wrong.

### R7 — `num_buildings` counts a finished building that belongs to no city

| | |
|---|---|
| document | CITIES.md §4.2, "Callers: `Wall::activate` **when `build_flags & 0x10` or in a city** … **A finished building outside any city without `build_flags & 0x10` is not counted** — the tech tree's 'has a temple' predicates follow the same rule." |
| code | `crates/sim/src/ai_types.rs:45-50` (`fn num_buildings_of`) |
| document says | `LeaderData::num_buildings[type]` moves only through `increment_stats`/`decrement_stats`, and `Wall::activate` calls it only for a type carrying the `e` flag (`NO_CITY`) or a building that is in a city. A temple, granary, market, library, senate, university, farm or smelter standing outside every city is invisible to the counter. |
| code does | `num_buildings_of` counts every `alive && active` building of the type and owner, with no city or flag test. Every reader goes through it: `buildings_of_line` (`crates/sim/src/ai_types.rs:62-75`), which is what `claim_building_high` measures the free-unit high-water mark with (`crates/sim/src/nations.rs:232`), and the AI's and tech tree's "how many of these do I have". |
| difference shows | Reachable only after a city dies: §8.4's `City::close` re-homes members and `decrement_stats` runs "unless `e`", so a temple whose city was destroyed and that finds no new city is counted here and not there. The tech-tree consequence is the visible one — a prerequisite of the form "has a temple" stays satisfied in this crate after the city is razed, so `type_avail` and the AI's build order diverge from that frame on. Capture: a `CITIES=5`/`BUILDS=6` window over a razed city with a surviving non-`e` member; none on disk. |
| reached | reached (`Wall::increment_stats` entered) |

### R8 — `check_upgrade` has no `gain_tech` trigger, and `find_buildings` has no Civic-tech trigger

| | |
|---|---|
| document | CITIES.md §5.3, "Triggers: the tail of `Build::activate` for any non-city member; `find_buildings` (founding, capture, **Civic tech**, the level-up itself); **`Leader::gain_tech` (`docs/TECH.md` step 11) when the gained tech is a prerequisite of `TOWN`**"; §5.4, "Called on founding, after a level-up, on capture, **after every Civic tech** (`docs/TECH.md` step 10: `mask_city` then this)" |
| code | `crates/sim/src/lib.rs:2261-2282` (`fn gain_tech`) and `crates/sim/src/lib.rs:2338-2356` (`fn apply_gained`) |
| document says | Four triggers for `check_upgrade` and four for `find_buildings`; two of them fire from the tech path, so a city with enough kinds levels up on the frame the Civic tech lands. |
| code does | `gain_tech` runs `reprice_library`, the tree, `upgrade_units_to`, `apply_gained` and `check_transport`. `apply_gained` sets `wall_stats_dirty`, syncs the researched bits and goods, recomputes pop caps, and calls `sync_territory` only when the border table actually moved — and never touches `check_upgrade`, `find_buildings` or `mask_city`. The complete call sites are `activate` (founding, `crates/sim/src/city.rs:1411` and `:1430`), `capture_city` (`:2092`) and `check_upgrade` itself (`:610`, `:682`). So the level-up waits for the next building to activate inside the city — or never happens, if none does. |
| difference shows | The whole `CITY` record from the level-up frame onward: `get_pop_value` (1 → 3) into `LeaderData::pop`, the radius (20 → 24) into membership, and thence `ter[6]`, `land`, `filled`, `bordering`, `gatherers`; plus the city building's `myhits` through the Senate bonus, and the territory the city projects. The `CITY` record is diff-backed on run58's 5,201 frames (`diff::tests::run58_s_five_thousand_frames_stand_where_the_original_s_do`, the loop at `crates/rondata/src/diff.rs:2545-2620`), so the assertion exists — that window just never reaches a Civic-gated level-up. |
| reached | reached (`City::check_upgrade@00738b20` and `Leader::gain_tech@006dcb60` both entered) |

### R9 — `capture_city` does not swap the loser's units in the radius on an allied hand-over

| | |
|---|---|
| document | CITIES.md §7.3 step 4, "and, **only for a self/ally hand-over** (the defeat path, §8.5), `O`'s units within the radius whose nearest friendly city is the new one are `swap_team`ed too"; step 2 supplies the radius, "`r = (min(get_radius(O, type), 64) + 3) / 4` cells" |
| code | `crates/sim/src/city.rs:2041-2129` (`fn capture_city`) |
| document says | When a defeated player's city is handed to an ally of theirs, the units standing in that city's radius change hands with it. |
| code does | Nothing: the member loop ends at `find_buildings(newcity)` (`crates/sim/src/city.rs:2092`) and no unit is touched. `Sim::defeat` (`crates/sim/src/city.rs:459-527`) clears the loser's orders and kills or swaps buildings, and never swaps a unit either. Step 2's cell radius is computed nowhere in the crate. |
| difference shows | `UNITDATA.who` for every unit of a defeated player standing near a handed-over city, on the defeat frame. Not reachable in the capture corpus (no elimination on disk) and only reachable at all in a game with allies — so it may be correctly deferred, but §11 does not list it among the stated simplifications and §8.5's own summary repeats the rule. |
| reached | reached (`Cities::capture_city` and `Leader::defeat_by` are not in the blind list) |

### R10 — the plunder gate carries a `capture_stamp != 0` term the document's arithmetic does not have

| | |
|---|---|
| document | CITIES.md §8.3, "**A city that was itself captured and not yet assimilated (`0x100`), or captured less than 4501 frames ago, yields nothing** — no ping-pong plunder"; §5.1, "stamps (attack, raid, capture, assimilation, reduce) = `transfer ? frame : 0`" |
| code | `crates/sim/src/city.rs:2262-2266` (`fn plunder_on_capture`) |
| document says | The gate is `frame − capture_stamp < 4501`, and a **founded** city's `capture_stamp` is `0` — so on the document's own arithmetic every city captured before frame 4501 yields no plunder at all, founded or not. |
| code does | `if old.unassimilated \|\| (old.capture_stamp != 0 && self.frame - old.capture_stamp < 0x1195)`. The extra `capture_stamp != 0` exempts a never-captured city, so a first capture before frame 4501 pays plunder here and, on the document as written, does not in the original. |
| difference shows | `LEADERDATA`'s resource buckets on the frame of an early capture — 100 per city level plus 25 per converted member, and `CAPITAL_PLUNDER` (500) on a first-capital loss. No capture on disk contains a city capture; the falsifying run is a scripted early capture with `LEADERS=1`. |
| reached | reached (`Cities::capture_city` is not in the blind list) |

`UNSURE:` this is a disagreement between the code and the document, not
necessarily between the code and the original — the code's guard may be the
right reading and the document's line the loose one. What settles it: the
compare at the head of `Cities::capture_city`'s plunder block in
`~/ghidra-projects/decomp` — whether it is a bare `frame - capture_stamp <
0x1195` or is itself guarded on a nonzero stamp. The `0x1195` literal is right
either way.

### R11 — `plunder_on_capture` applies the Despot cut to the first-capital payment; the document puts it only on the ordinary one

| | |
|---|---|
| document | CITIES.md §8.3, "Otherwise: **not the first capital loss** → `take = hero ? THEDESPOT_PLUNDER × base / 100 : base` … **The first capital loss**: `O.leader_flags \|= 0x400000`; `base = max(base, CAPITAL_PLUNDER (500))` …, paid into **every** usable good." |
| code | `crates/sim/src/city.rs:2302-2321` (the `else` arm of `fn plunder_on_capture`) |
| document says | The hero cut appears only in the non-first-capital branch; the first-capital branch is described as paying `base` into every usable good, with no hero term. |
| code does | Recomputes `(take, steal)` with the same `hero ? thedespot_plunder * base / 100 : base` shape inside the first-capital arm. |
| difference shows | The captor's buckets on a first-capital capture with a Despot or Spitamenes patriot near the city. `THEDESPOT_PLUNDER` ships 100, so the two agree in the shipped data and this is inert until a mod changes it. |
| reached | reached (`Cities::capture_city` is not in the blind list) |

`UNSURE:` the document's own layout is ambiguous — the hero line may sit above
the branch in the original and apply to both arms. Settles it: the order of the
`THEDESPOT_PLUNDER` multiply against the `leader_flags & 0x400000` test in
`Cities::capture_city`'s plunder block.

### R12 — `update_hits`'s tower exclusions are a type identity here and a lineage in the original, so a Keep gets the Senate bonus

| | |
|---|---|
| document | CITIES.md §3.4, "`Romans, (is_fort or TOWER), not a wonder: h = (ROMAN_FORT_HP + 100) × h / 100`" and "`not a city: active, in a city, not is_fort/TOWER/LOOKOUT: h += SENATE_HP_BONUS × (city level − 1) × h / 100`"; §1.5, "`is_tower` = `is(TOWER, 0)` Tower/Keep/Stockade" |
| code | `crates/sim/src/build.rs:583-584` and `:596-603` (`fn full_hits`) |
| document says | Both exclusions are the `TOWER` **lineage**, which admits Tower, Keep and Stockade. |
| code does | `b.ident == Ident::Tower` in both places, an identity on the base type only — `if m.romans && (is_fort(types, ty) \|\| b.ident == Ident::Tower)` at `:583`, and `&& b.ident != Ident::Tower && b.ident != Ident::Lookout` at `:600-601`. `build::is_tower` exists and is the right predicate (`crates/sim/src/build.rs:406-407`); `place.rs` and `garrison.rs` both use it. The Nubian `MARKET` clause at `:593` is the same shape. |
| document says (consequence) | A Keep or Stockade in a Large City gets **no** Senate bonus, and a Roman's Keep gets `ROMAN_FORT_HP`. |
| code does (consequence) | A Keep or Stockade in a Large City gets +35 % hits it should not have (+70 % in a Major City), and a Roman's Keep loses the +0 % Roman bonus (inert in the shipped data, where `ROMAN_FORT_HP` is 0). |
| difference shows | `BUILDDATA.myhits` for any Keep or Stockade inside a Large or Major City, from the frame the city levels or the Keep upgrades. `BUILDS=6` prints it every frame; not compared today (`crates/rondata/src/diff.rs:2440-2460`). Needs a game that reaches the Keep upgrade *and* a Large City — none of the 68 traces does. |
| reached | reached (`Wall::update_hits@0063f0d0` entered) |

Settled against the export rather than inferred: `Wall::update_hits@0063f0d0`
lines 85-100 are `is_fort` (`+0xfc`), then `is(0x1b7, 0)`, then `is(0x209, 0)` —
three lineage calls, all `is`, and the third is `LOOKOUT`. The Roman clause at
`:50` and the Nubian `MARKET` clause at `:59-63` are the same shape (`is(0x1b4,
0)`). This crate spells all three as `ident ==`.

### R13 — `update_hits`'s garrison eject has no hangar exemption

| | |
|---|---|
| document | CITIES.md §3.4, "`h ≤ damage and a garrison and not can_carry(AIR) → eject_contents(1, −1, 0, 1)`" |
| code | `crates/sim/src/city.rs:1100-1102` (`fn update_hits`) |
| document says | An airbase, carrier or missile silo whose `construct_hits` has fallen to its damage does **not** eject — its contents are killed elsewhere (§6.6, `kill_contents`). |
| code does | `if bd.construct_hits <= bd.damage && !bd.garrison.is_empty() { bd.eject_pending = true; }` — no `can_carry(AIR)` test, though `Sim::is_hangar` exists at `crates/sim/src/garrison.rs:558-563`. |
| difference shows | A damaged airbase would push its planes onto the map rather than keeping or killing them: `UNITDATA` rows appearing beside the building. Airbases are past every capture on disk (no traced game reaches the Industrial age), so this is unreachable now. |
| reached | reached (`Wall::update_hits@0063f0d0`) |

`Wall::update_hits@0063f0d0:198-202` is the guard, verbatim.

### R14 — three gameplay effects of `Build::activate` are absent, and §11 does not record them

| | |
|---|---|
| document | CITIES.md §4's effect table: **wonder** — "**every other player's unfinished copy of the same wonder is `disband`ed with a full refund**"; **SENATE** — "in a city, not captured: `senates_built++`; **if the city is the capital's race and the capital has no senate, the capital flag moves here** (`0x10` cleared elsewhere, set here, `fix_borders`)"; and, on the same rows, `wonders_built++` and the gov-hero `train(get_gov_hero())` |
| code | `crates/sim/src/city.rs:1346-1460` (`fn activate`) |
| document says | Three things happen that are not messages, sounds or AI: a finished wonder kills every rival's half-built copy and refunds it; a finished senate can move its owner's capital, which recomputes borders; and `wonders_built` is what the free-first-wonder clause of `construct_time` (§3.2 (a)) reads. |
| code does | None of the three. Grepping the crate finds no `wonders_built`, no `unbuilt_wonders` list, no `senates_built` and no `get_gov_hero`. §11 lists "The per-wonder one-offs of `activate` are not modelled" — but the rival-copy disband is not a per-wonder one-off, it is the wonder branch's common code, and the senate capital move is not a wonder at all. |
| difference shows | The wonder half: a rival's `BUILDDATA` row vanishing plus a full refund into their buckets, on the frame any wonder finishes. The senate half: `CITY.city_flags & 0x10` moving between two of one player's cities, and the border recompute behind it — directly in the run58-style `CITY` diff (`crates/rondata/src/diff.rs:2545-2620`), which already compares `city_flags`. No capture on disk finishes a wonder or a senate. |
| reached | reached (`Build::activate@00623e20`) |

Also missing and stated in §3.2 (a): `wonders_built == 0` is one of the free
first wonder's conditions, and `ClockMods::free_first_wonder`
(`crates/sim/src/build.rs:497-501`) is an input this crate never computes from
game state.

### R15 — `remove_from_city` does not regenerate the city's roads

| | |
|---|---|
| document | CITIES.md §5.2, "`remove_from_city` unlinks, **`regen_roads`**, clears each flag whose `count_buildings(kind, 0, 1)` dropped to zero, `city = −1`, `decrement_stats`." |
| code | `crates/sim/src/city.rs:566-573` (`fn remove_from_city`) |
| document says | A building leaving a city flags that city's whole membership for a road replan. |
| code does | Unlinks and, for a temple, re-syncs territory. `Sim::city_regen_roads` (`crates/sim/src/roads.rs:152-162`) is called from `activate` (`crates/sim/src/city.rs:1443`) and from nowhere else. |
| difference shows | `TERRAINSYNC`/road tiles in the sixteen frames after a member is destroyed or re-homed; `docs/ROADS.md` §1's own diff. Reachable in any capture where a building dies inside a city — run53 and run63 both have building deaths. |
| reached | reached (`Build::remove_from_city` entered) |

### R16 — §2.6.4's `RAZING_TOWN` verdict is missing

| | |
|---|---|
| document | CITIES.md §2.6.4, "`that city's queue holds DISBAND (0x29a) or RAZE (0x29b) → RAZING_TOWN 0x21`" |
| code | `crates/sim/src/place.rs:696-733` (`fn blocked_location_verdict`, the "must belong to a city" block) |
| document says | Between the `get_town` failure and the one-per-city test, a site is refused when the city it would join has a `DISBAND` or `RAZE` item queued. |
| code does | Goes straight from `get_town`'s `None` arm to the `ONE_PER_CITY` test; `Blocked::RazingTown` is declared in the enum and returned nowhere. |
| difference shows | A player or AI placing into a city that is being razed. `DEPOPULATE`/`DISBAND` queue items are not modelled at all in this crate, so the branch has no inputs; it costs nothing until they are. |
| reached | reached (`blocked_location@006375b0`) |

### R17 — `check_capture`'s building tally admits inactive and water-domain buildings

| | |
|---|---|
| document | CITIES.md §7.2, "for every object X in the cells within `circle_radius[cells]` of B's cell, same land id: skip B, skip U, **dead, sea or air domain, decoys, `!valid_filter(8)`**, `vector_dist(X, B) > dist_max`"; §11, "**`valid_filter(8)`** in the capture count is taken as 'alive and on the map'" |
| code | `crates/sim/src/city.rs:1976-1993` (the building half of `fn check_capture`) |
| document says | The domain filter applies to every object in the sweep, buildings included, so a Dock (domain 1, water — §1.5) never joins either side's tally. |
| code does | The unit loop filters on `Domain::Land` (`crates/sim/src/city.rs:1963-1965`); the building loop filters only on `i == b`, `alive`, the land id and the distance. A Dock, Anchorage, Shipyard or Oil Platform of the defender therefore contributes `7 + garrison` to the defence, and any third party's contributes 1. The loop also admits a building that is not `active` — an unfinished site of the defender counts a full 7. |
| difference shows | Which side wins a coastal capture, and therefore whether a city changes hands at all — a binary outcome on one frame. `CITY.who` and the whole `BUILDDATA` list on the capture frame. No capture on disk contains a city capture. |
| reached | reached (`Build::check_capture` is not in the blind list — it runs every 64 frames on any eligible building) |

`UNSURE:` a second, smaller thing on the same function — §7.2's radius is `R =
city ? CITY_CAPTURE_RADIUS (10) : UNIT_RESPOND_RANGE (12)`, and
`crates/sim/src/city.rs:1931` always takes `city_capture_radius`. The village
arm of §7.1/§7.2 (`health_level > 4`, `swap_team` instead of `capture_city`) is
not modelled either, and §11 does not say so. Whether the village arm is
reachable at all turns on what distinguishes "`flags & 0x20`" from
"`can_capture`" in §7.1, which the document does not resolve.

### R18 — `defeat_by`'s tail is missing, and its trainer predicate is a hand-written type list

| | |
|---|---|
| document | CITIES.md §8.5, "an **active military trainer** (not fort/tower/lookout/city) inside one of `by`'s cities → `swap_team(by)`; the rest `die(3)`. Then the flags, `clean_queue` everywhere, orders cleared, planes die, **the stockpile tributed to allies**." |
| code | `crates/sim/src/city.rs:459-527` (`fn defeat`) |
| document says | The predicate is `is_military_trainer` (the derived `5` flag, read on the `FROM` root — §1.5) with four exclusions, and the function ends by tributing the loser's stockpile to their allies, killing their aircraft and cleaning every queue. |
| code does | `matches!(ident, Barracks \| Stable \| AutoPlant \| SiegeFactory \| Factory \| Dock \| Airbase)` — a hard-coded list, where `build::is_military_trainer(&self.build_types, t)` exists and is already used by `queue_capacity` (`crates/sim/src/build.rs:386`). And the tail is one line, `self.clear_orders(u)` per unit: no stockpile tribute, no aircraft kill, no `clean_queue` (though `close_building` does clean the queue of each building it closes). |
| difference shows | `LEADERDATA`'s buckets for every ally of the loser on the defeat frame — the tribute is the whole stockpile, so it is the loudest number in the record. Unreachable in the corpus (no elimination in any of the 68 traces). |
| reached | reached (`Leader::defeat_by` is not in the blind list) |

### R19 — `City::close` runs `lost_a_city` after the members re-home; the document has it before

| | |
|---|---|
| document | CITIES.md §8.4, "`City::close(old, captor)`: `city_num--`, … armies released; `city_flags &= ~1`; **`Leader::lost_a_city(O, was_capital, race == who, captor)`**; every member: `city = −1`, … and a surviving non-city member **re-finds its city**" |
| code | `crates/sim/src/city.rs:427-445` (`fn close_city`) |
| document says | `lost_a_city` — which in the default elimination mode calls `defeat`, which captures every remaining city and kills or swaps every building — runs **before** the dead city's members are re-homed. |
| code does | Clears each member's `city` and calls `find_city(m)` for the survivors (`:435-441`), and only then `self.lost_a_city(owner, was_capital, captor)` (`:442`). |
| difference shows | Only on the frame a player loses their last city: `defeat` walks `self.buildings` and asks `find_city_at` per trainer, so it sees a different membership picture depending on the order. `BUILDDATA.who` for the loser's trainers inside the captor's cities. Unreachable in the corpus. |
| reached | reached (`City::close` / `Leader::lost_a_city` are not in the blind list) |

`UNSURE:` the ordering matters only where `defeat` reads a member's city, which
in this crate is `find_city_at(by, …)` on the *captor's* cities — probably
insensitive to the loser's re-homing. Filed because the order is stated and the
code inverts it, not because a divergence is demonstrated.

### R20 — `kill_competing_buildings` skips a started rival instead of stopping at it

| | |
|---|---|
| document | CITIES.md §3.5, "**`kill_competing_buildings`** — every *other* not-started building placed on a double-placed footprint tile is `Object::disband(…, 1)` (full refund), **a started one stops the sweep**"; repeated in §9.4, "the first `Wall::start` disbands every other not-started building on a double-placed tile with a full refund. A started one stops the sweep." |
| code | `crates/sim/src/city.rs:1155-1182` (inside `fn start_building`) |
| document says | The sweep over a doubly-placed tile terminates when it meets a building that has already started — whatever candidates lie behind it survive. |
| code does | `if o == b \|\| !self.buildings[o].alive \|\| self.buildings[o].started { continue; }` — a started rival is **skipped**, and the loop carries on collecting every later unstarted one into `victims`. The two behaviours differ whenever a doubly-placed tile carries a started building *and* an unstarted one after it in object order: the original leaves the unstarted one alive, this crate disbands and refunds it. |
| difference shows | A `BUILDDATA` row vanishing plus a full refund into its owner's buckets, on the frame a third building on the same tiles starts. Needs three overlapping placements on one tile, which nothing on disk has; the falsifying capture is a scripted triple `add NEW` on one tile with `BUILDS=1` and `LEADERS=1`. |
| reached | reached (`Wall::start@0063e810`) |

`UNSURE:` the document says "stops the sweep" twice but does not say what the
sweep iterates — per tile, or per object over the whole footprint. If the
original's sweep is per *tile* and each tile's chain is at most two deep, the
two spellings can never differ. Settles it: the loop shape at
`Wall::kill_competing_buildings` in the export, and whether its `break` is
inside the tile loop or the object loop.


## Adjudication — 2026-09-05, Opus

**Twenty confirmed, none struck.** Every row's code citation re-checked
against the source; R1, R3, R7, R13 and R15 read at the site.

**R3 is the one to take first, and it is not a rule the code is missing —
it is a rule the code gets wrong every frame.** `process_building`'s
periodic phase is `let phase = frame + b as i64`, where `b` is the
building's index in this crate's `Vec`. The original's is
`Build::process@0061edf0:728`, the object number `o`. The two agree only
while every building's handle equals its object number, which is true at
the start of a game and stops being true the first time a building dies
and its slot is reused, or the first time a building is created in a
different order than the original's `o` allocation. Every 32-frame phase
downstream of it — the `under_attack` decay here, and whatever else shares
the counter — then fires on the wrong frame for that building, for the rest
of the game. It is a divergence *generator* rather than a divergence, which
is why nothing has attributed a frame to it.

**R15 is cross-confirmed.** `remove_from_city` not regenerating the city's
roads is the same gap the ROADS pass reached independently as its R4 — two
readers, two documents, one missing call. That is the first time this wave
produced the same finding twice from different directions, and it raises
the confidence on both.

**R7** — `num_buildings_of` filters `alive && active && owner && ty` with
no city test, so a finished building belonging to no city is counted —
and **R13** — `update_hits`' eject is `construct_hits <= damage &&
!garrison.is_empty()` with no `can_carry(AIR)` exemption, though
`Sim::is_hangar` exists and is the right predicate — are confirmed as
written.

**R12 is a predicate error of the kind the record says recurs**:
`update_hits`' tower exclusions are a type identity (`b.ident ==
Ident::Tower`) where the original asks `is(TOWER, 0)`, so a Keep takes the
Senate bonus it should not. `build::is_tower` exists and is what
`place.rs` and `garrison.rs` already use. The Nubian `MARKET` clause beside
it has the same shape.

The four `UNSURE:` rows (R10, R11, R17, R19, R20) are left marked as the
reader left them; none is a citation error, and each names what would
settle it.
