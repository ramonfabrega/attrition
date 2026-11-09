# Territory adjudication — list 1

The seventy-first tranche's third reading (`docs/audit/README.md`), over the 17 rows of
`adj-1.tsv` on which the first reader (`territory-<group>.md`) and the blind second reader
(`territory-second-<n>.md`) disagreed. Adjudicator: Opus 5.5. Every row settled against the
decompile export (`~/ghidra-projects/decomp/funcs/`), with `llvm-objdump` on the shipped executable
where the decompile could not settle it, and against `crates/sim` at the worktree's tip (41c823576).

**How the two vocabularies are read.** The first reader's `twin-differs` and the second's `differs`
both mean "the crate has the rule with a difference"; the second's `carried` means "the same
predicate". On most rows below the two readers found the *same* facts and differ only on whether a
difference that cannot be reached is a difference. Those rows are ruled on the letter: **A** when
the predicate the crate writes is not the original's, however unreachable, and the row says
`inert` and why; **B** when the difference the first reader names does not exist in the code.
An inert difference's item is `none` unless something cheap would pin it.

## Part 1

| va | name | kind | first | second | verdict | settles it | the item |
|---|---|---|---|---|---|---|---|
| 006f9a90 | Army::charge | DISAGREE | twin-differs | carried | **A, inert** | `charge@006f9a90:13-21`: `field_0x30/0x34 = param_1/param_2` sits inside `if (-1 < *piVar3 && groups[g]+0xc != 0)`, so the target is written only when some group has members; `crates/sim/src/army.rs:1704-1718` writes `target = Some(target)` after the `valid` test alone. Both readers say so; the second calls it unreachable, which it is: both callers reach it through `army_of(u)` for the hit siege unit (`orders.rs:8837`, `fight.rs:2789`), a member, so the one group is never empty. | none |
| 007395c0 | CityData::get_empty_trade_routes | DISAGREE | twin-differs | carried | **A, inert** | `get_empty_trade_routes@007395c0:19-22`: `(flags & 2) && ((s[0]==city && s[1]==owner) \|\| (s[2]==city && s[3]==owner))` over the city's own `vans`, answering `1 − count`; `crates/sim/src/caravan.rs:226-236` needs `v.alive && v.linked`, matches `city_a/city_b` by the global index with no owner, over every leader's slots, and answers a bool. The crate's own sweep asserts both partings still part (`crates/sim/src/sweep/city_get_empty_trade_routes.rs:89-101`) and `docs/PARKED.md` 1628(a) parks them as unreachable — so "same predicate" is refuted by the crate's own test, and "equivalent in play" is the parked ruling, not the second reader's. | none (parked 1628(a) holds it) |
| 006820e0 | CollCheck::fill_slots | DISAGREE | twin-differs | carried | **B** | `fill_slots@006820e0:84-98`: with `param_4` and a tree, a slot the tree holds sets `local_24 = 1` and reads the tree's copy, skipping the region gate at `:104`. The crate does the same: `crates/sim/src/movement/collide.rs:2091-2111` takes `copies.entry((cx,cy)).or_insert_with(..)` from `Sim::coll_copies` (`lib.rs:1607`, a map that outlives the probe, cleared only by the search at `path.rs:605`), and `ProbeSlots::get` (`collide.rs:78-85`) reads a held copy before `readable`. The difference the first reader names is the doc comment's SEAM at `collide.rs:2051-2053`, which the code no longer has: the SEAM is stale. | a citation: strike the stale SEAM sentence at `crates/sim/src/movement/collide.rs:2051-2053` (`tools/seams.py` lists it as live) |
| 0070bab0 | GroupData::get_stance_option | DISAGREE | twin-differs | carried | **A, inert** | `get_stance_option@0070bab0:66-73`: `if (this->buildings == 0) cVar1 = unit+0xb1; else cVar1 = (vslot +0x3c)()+0x7e;` — the crate's `group_stance_option` (`crates/sim/src/group.rs:2038-2063`) has no buildings arm because its `Group` (`group.rs:513-525`) carries no buildings flag. Both readers name the absent arm; the out-parameter (`*param_1 = distinct < 2`) is the interface's (callers `IFaceOptions::setup_button`, `Options::describe`, `Scene::set_help_type`; `action_stance` passes NULL), so it is no difference. The arm is inert: no crate group is a building group (`docs/GROUPS.md:2034`), and the stance callers are `action_stance` and `Army::set_stance`, the latter gated on `STANCE_COMBAT`. | none |
| 0070d370 | GroupData::get_stance_type | DISAGREE | twin-differs | carried | **A, inert** | `get_stance_type@0070d370:13-17`: `buildings == 0 ? find_leader(this, 0) : list[0]`; `crates/sim/src/group.rs:1977-1998` seeds from `group_find_leader` only. The tail agrees: the original returns the last member's `SVar2`, the crate the leader's `s`, both `STANCE_NONE` whenever no caster was seen (a caster leader is itself a member). Same inert buildings arm as the row above. | none |
| 007111c0 | Group::action_scramble | DISAGREE | twin-differs | carried | **A** | The original walks any member's `inside_down` chain, and its second caller `Unit::do_patrol@005f1910:87-101` builds a one-unit group of the patrolling unit when `+0x28 >= 0` and `is(0x15f, 1)` (an Aircraft Carrier) and scrambles it. `crates/sim/src/airbase.rs:277-285` takes `buildings: &[usize]` only (`planes_inside`, `:63-73`), its one live caller is the replayed command (`crates/rondata/src/input.rs:1109`), and `do_patrol`'s tail is a named SEAM (`crates/sim/src/orders/mod.rs:5075`). Plus `is_busy` (SEAM `airbase.rs:275`). The second reader lists all three as residuals and still rules `carried`; they are arms the crate lacks. | a capture: a skirmish with a patrolling Aircraft Carrier holding planes, `UNITS` per frame, read for the planes' `AIRPATROLORDER` over the carrier's point (no lockstep game has a carrier) |
| 0069bc90 | MapHimalayas::make_continents | DISAGREE | unmodelled | carried | **B (twin, unwired)** | The decompile differs from `MapGreatSahara::make_continents@0069bf70` only in the class name (`diff` of the two bodies), and the listing settles it: `llvm-objdump` of `0x69bc90..0x69be00` against `0x69bf70..0x69c0e0` is the same 144 lines, instruction for instruction, the four branch targets each at the same offset (`+0xb1` etc.) from their own entry. So the rule is `crates/sim/src/mapgen/mod.rs:406` `great_sahara_make_continents`, as Mesa's is (`mapgen.rs:427-435`, item 3869). What the crate lacks is only a style-9 route, which the first reader's "body not read" could not see. | a widening on dumps on disk: run the 7807 check (`crates/rondata/src/mapgen.rs:1288`, filtered to `style == 7`) on style 9 (run348's Him column, MAPGEN §2); and a citation: MAPGEN §5's "read the same" is now listing-checked, which reduces item 3946's port to an alias |
| 0069be00 | MapAmazonBasin::make_continents | DISAGREE | unmodelled | carried | **B (twin, unwired)** | Same as Himalayas: `0x69be00..0x69bf70` is Great Sahara's listing instruction for instruction (144 lines, branch offsets equal from entry). The rule is `mapgen.rs:406`; no style-8 route. | a widening on disk if run1099 carries `map.cpp` 7805/7807: the same 7807 check on style 8; the citation reduces item 3945's port to an alias |
| 0069c0e0 | MapOldWorld::make_continents | DISAGREE | unmodelled | carried | **B (twin, unwired)** | Same: `0x69c0e0..0x69c250` is Great Sahara's listing instruction for instruction. The first reader's "2,089 bytes, the size of Great Sahara's 2,104" is the decompile file's size, not the function's (each body is 0x170 bytes). | a widening on disk if run1117 carries `map.cpp` 7805/7807: the 7807 check on style 6; the citation reduces item 3944's port to an alias |

## Part 2

| va | name | kind | first | second | verdict | settles it | the item |
|---|---|---|---|---|---|---|---|
| 00647e90 | Object::insert_inside | DISAGREE | twin-differs | carried | **A** | `insert_inside@00647e90:37`: `if ((*(byte *)(piVar2 + 0x1a) & 1) == 0)` — the unit's `+0x68` bit 0, the decoy bit (`docs/ARMY.md:253`) — gates every counter; `crates/sim/src/lib.rs:2183-2191` `track_garrison` asks `is_captain` only, and says so (SEAM `lib.rs:2176`). The second reader's "a decoy never garrisons, so inert" is not shown: the original keeps a `COUNT_NON_DECOY_TYPE` mode for garrisons, which the crate carries (`orders.rs:8393-8410`), so a decoy inside is a state the original provides for. Field: the first reader's "the counters are not dumped" is wrong — `LEADERDATA` `peasants_garr stable_garr scout_garr barracks_garr` are printed and sit in the uncompared pin (`crates/rondata/src/diff/coverage.rs:275`), as the second reader says. | a widening on dumps on disk: compare the four `LEADERDATA` `*_garr` counters (drop them from the pin at `coverage.rs:275`) on every pinned run; a decoy garrisoning would then part on its frame |
| 0088e100 | Roads::scan_and_kill_bad_tcoord | DISAGREE | twin-differs | carried | **A, narrowed** | `scan_and_kill_bad_tcoord@0088e100:31-32`: `is_terrain_creation != 0 \|\| element_num == *(int *)(this+0xdc)`. The crate's arm is `Some(e) if e.made \|\| e.c4` (`crates/sim/src/movement/mesh.rs:533`), and `e.c4` **is** set — by `set_diags`' corner tail (`mesh.rs:794`, `docs/ROADS.md` §12, item 3287) — so the first reader's "never taken here" is wrong, and rests on `docs/ROADS.md` §10.5's "the C4 exemption is `is_terrain_creation` alone here" (`ROADS.md:1259-1262`), which §12 superseded. What remains is the second reader's residual: an element whose `RoadsPieces::find_map` pick is the C4 piece is exempt in the original and not here (the pick is not modelled, `ROADS.md:1375`). A difference, narrower than the first reader drew it. | a citation: strike `docs/ROADS.md` §10.5's "C4 exemption is `is_terrain_creation` alone here" and point it at §12; `element_num` is printed by no log, so no widening |
| 006680e0 | Type::sum_rules_cost | DISAGREE | twin-differs | carried | **A, inert** | `sum_rules_cost@006680e0:16-27`: `constants->unit_cost_factor * sum` / `build_cost_factor * sum` / … by id range; `crates/rondata/src/load.rs:1433` and `:1654` fill `cost: c.cost.iter().sum::<i32>() * 10`, the literal. Both readers agree; equal on shipped data (every factor ships 10), parts on a mod. | none (a one-line loader change to `Tuning`'s factors would move no score) |
| 005e3530 | Unit::modify_group_order | DISAGREE | twin-differs | carried | **A, inert** | The gate is the caller's: `Group::refresh_group_order@00713a50:51-56` calls `modify_group_order` only for a member with vslot `+0x8` (active), vslot `+0xbc` (on the map) and `+0xdc != 0`; `crates/sim/src/group.rs:4036-4047` `group_rewrite_leader` rewrites every listed member. Both readers name it; the second adds the order against `update_positions`. Inert unless a member is inside something when another takes the formation. | none (a capture with a garrisoned member of a group whose leader is lost would show `UNITDATA … GROUPORDER oxx`; no item books it) |
| 00602ea0 | Unit::dbg_jump_to_action | DISAGREE | twin-differs | not-a-rule | **A** | The second reader's "no counterpart, by design … no skirmish capture can enter it" is refuted twice: the crate has the twin, `crates/sim/src/scenario_add.rs:208` `unit_dbg_jump_to_action`, called through `group_dbg_jump_to_action` (`:194`) from the script `create_unit` path (`:142`, item 3917, run1511's 641); and the export's callers include `ScenarioFuncSet::add_unit@009e2220` and `ScenarioFuncSet::group_jump_move@009fb2d0`, script functions, not only the editor. The twin builds the move arm only; the two head arms are absent — `get_action()` type `0x1a` → `insert_inside(target)`, `repath`, `kill_current_order` (`dbg_jump_to_action@00602ea0:18-29`) and type 7 → `set_new_location(target pos)` (`:31-39`). On the `create_unit` path the fresh units' action is the form move `group_action_form` just gave them (`scenario_add.rs:141`), so the head arms are not entered there. | none on the script path; a citation: the twin's doc (`scenario_add.rs:201`) should name the two absent head arms as a SEAM |
| 0063d440 | Wall::kill_at_tile | DISAGREE | twin-differs | carried | **A, inert as far as read** | The listing settles the decompiler's branch: `63d4b3 call *0x50(%eax); 63d4b8 jne 0x63d54f` — a started first-found ends the tile; and `find_building_placed_at@00658c80:43` keeps only `iVar5 < 8`. `crates/sim/src/city.rs:1906-1932` skips started buildings and keeps looking, and takes any owner. The second reader's equivalence argument holds by induction **if no site can be placed on a started building's footprint**: the first building to start on a tile meets no started rival there (itself excluded) and disbands every unstarted one, so every later started building on that tile was started from an empty tile, and the early return never chooses. That premise is a placement rule (`docs/CITIES.md` §3.1: an unstarted site's `0x80` already blocks; §9.4: two players may still double-place), not something this function shows — which is why §12 leaves it `UNSURE`. | a citation: the induction above, written into `docs/CITIES.md` §9.4 with the placement premise named, closes §12's `UNSURE` or names what would open it |
| 0063eeb0 | Wall::update_los | DISAGREE | twin-differs | carried | **A, inert** | Both readers list the same three omissions in `crates/sim/src/vision/mod.rs:444-472` `build_los` (colosseum and furs terms, constants shipped 0; the `leader_flags & 1` gate on the base `LOS`), which is a difference by the brief's own definition. The field claims part: the second reader's "`mylos` printed and compared over whole runs (`build.rs:2993`)" cites the **unit** comparison (26,701 unit-frames against `Sim::unit_los`, `build.rs:3034`); the building record's `mylos` is in the uncompared pin, `"GAME/FRAME/BUILDDATA/WALLDATA/OBJECT"` at `crates/rondata/src/diff/coverage.rs:204-205`, as the first reader says. | a widening on dumps on disk: compare `BUILDDATA/WALLDATA/OBJECT mylos` against `Sim::build_los` (drop it from the pin at `coverage.rs:205`) |
| 006b3b80 | WorldData::is_city_at | DISAGREE | twin-differs | carried | **A** | `is_city_at@006b3b80:17-21`: `if (-1 < who && who != p && !is_ally(p, who)) return 0; return 1;` — a cell at `who == −2` passes. `crates/sim/src/place.rs:715-719`: `Owner::None => {}`, `Owner::Player(p) if is_ally => {}`, `_ => return None` — `Owner::Ambiguous` (the −2, written by `territory.rs:446`) refuses, against the crate's own invariant that every consumer reads it as unowned (`world.rs:175-180`). The second reader's "passes when `< 0`" is the original's rule and not the crate's; its sweep (`sweep/world_data_is_city_at.rs:48-50`) re-derives the predicate inline and never calls `find_city_at`, so it cannot catch this. | a one-line fix with a test (`Owner::None \| Owner::Ambiguous => {}`); no capture is known to hold a −2 cell under a city radius when something asks `find_city_at` (placement, `alarm`, `castable`) |

## Summary

**Counts** (17 rows, all `DISAGREE`, no `CLAIM` row on this list): **A 13** — of them **A, inert 8**
(`Army::charge`, `get_empty_trade_routes`, `get_stance_option`, `get_stance_type`, `sum_rules_cost`,
`modify_group_order`, `Wall::update_los`, `Wall::kill_at_tile` as far as read), **A, narrowed 1**
(`scan_and_kill_bad_tcoord`), **A 4** (`action_scramble`, `insert_inside`, `dbg_jump_to_action`,
`is_city_at`); **B 4** (`CollCheck::fill_slots`, and the three `make_continents` as Great Sahara's
twin, unwired); both wrong 0; ambiguous 0.

The pattern: on 11 of the 13 A rows the blind reader found the same difference and called it
`carried` because it judged it unreachable; only `is_city_at` and `dbg_jump_to_action` are A because
the second reader misread the crate. On the B rows the first reader was the one misled — by a stale
SEAM comment (`fill_slots`) and by not reading the bodies (`make_continents`). Two field claims were
wrong and are corrected above: building `mylos` is **uncompared** (second reader said compared), and
the garrison counters are **printed, uncompared** (first reader said not dumped).

**The rows that name an item, cheapest first:**

1. **Widening, `BUILDDATA/WALLDATA/OBJECT mylos`** — remove it from the uncompared pin
   (`crates/rondata/src/diff/coverage.rs:205`) and compare it against `Sim::build_los` on every
   pinned run; a dump on disk, a parser-side line (`Wall::update_los`).
2. **Widening, `LEADERDATA peasants_garr stable_garr scout_garr barracks_garr`** — out of the pin at
   `coverage.rs:275` and compared on every pinned run (`Object::insert_inside`; also pins
   `remove_from_inside`'s −1).
3. **Widening, the 7807 check on styles 6, 8, 9** — `great_sahara_s_mask_is_the_traced_7807`
   (`crates/rondata/src/mapgen.rs:1288`) filters `style == 7`; Himalayas' run348 is MAPGEN §2's Him
   column, and Amazon (run1099) and Old World (run1117) if their traces carry 7805/7807. With the
   listing compare above, items 3944–3946's "port from the reading" is an alias like
   `southwest_mesa_make_continents`.
4. **Citations, no score** — strike the stale SEAM at `crates/sim/src/movement/collide.rs:2051-2053`
   (`fill_slots`); strike `docs/ROADS.md` §10.5's "C4 exemption is `is_terrain_creation` alone here"
   for §12 (`scan_and_kill_bad_tcoord`); name `dbg_jump_to_action`'s two head arms as a SEAM at
   `scenario_add.rs:201`; write the `kill_at_tile` induction into `docs/CITIES.md` §9.4.
5. **A one-line fix, `find_city_at` on `Owner::Ambiguous`** (`crates/sim/src/place.rs:718`), with a
   unit test; no capture known to reach it.
6. **A capture** — a patrolling Aircraft Carrier with planes aboard, `UNITS` per frame, for
   `do_patrol`'s scramble tail (`Group::action_scramble`; `orders.rs:5075` SEAM).

## Not established

- `get_empty_trade_routes`: whether a captured city keeps its `vans` with the old `(city, owner)`
  pair, which is the only way the dropped owner half could part; parked 1628(a) rules it
  unreachable and this reading did not re-derive `close_caravan` or the capture path.
- `Wall::kill_at_tile`: whether any placement path (the AI's, a script's, a fogged enemy's) can lay a
  site on a started building's footprint. The induction makes the early return inert only if none
  can.
- `Wall::update_los`: whether `leader_flags & 1` is ever clear for a building's owner in a skirmish
  (the first reader cites `docs/VISION.md` for "set for every building owner"; not re-read here).
- The three `make_continents` rows: the listing compare shows the instructions, not that runs 1099
  and 1117 carry the 7805/7807 checkpoints; the widening in item 3 is sized on run348 alone until
  someone greps the archive.
- `get_stance_option`/`get_stance_type`: whether a human's stance button can reach a building
  group (the `IFaceOptions::setup_button` path was not read).
- `dbg_jump_to_action`: whether `ScenarioFuncSet::group_jump_move@009fb2d0` has a crate twin, and
  whether any shipped AI script calls it; a script that did would enter the move arm on units
  carrying arbitrary actions, including the two absent head arms.
