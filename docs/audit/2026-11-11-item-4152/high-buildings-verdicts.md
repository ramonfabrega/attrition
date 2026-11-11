# high-buildings: adjudication

- Item 4152, wave 3. Adjudicator: Claude Opus 5.5, 2026-11-11.
- Inputs: `high-buildings-reading.md` (first), `high-buildings-second.md` (blind second), the decompile export, and the full .text listing of `riseofnations.exe` (llvm-objdump).
- Leaves: `territory_high` (+0x9dc int), `reg_terr` (+0x145e ushort[64]), `high_buildings` (+0x5660 ushort[129]).

## Leaf 1: `territory_high` (+0x9dc, int)

### Claims lined up

| claim | first | second | status |
|---|---|---|---|
| writer: `World::compute_all_territory (006b5700)`, store `006b5880` via `edx = 0xe3ad68` (`&territory`), `0x4(%edx)` | yes | yes | agree, **confirmed** (`006b5834`, `006b5877`..`006b5880`) |
| writer: `GameDaemon::check_borders (00732060)`, same fold | yes (store not given) | yes, store `00732160` | agree, **confirmed** (`0073211b`, `00732157`..`00732160`) |
| no direct store at `0x9dc(` on a leader, no absolute `0xe3ad6c` | yes | yes | agree, **confirmed** (below) |
| predicate: slots 0..7 only (bound `0xe724c8`), `leader_flags & 1` | yes | yes | agree, **confirmed** (`testb $0x1,-0x9d8(%edx)` at `006b5840`/`00732120`; `cmpl $0xe724c8,%edx` at `006b588f`/`0073216f`) |
| step: `territory = Σ reg_terr[r]` over r in 0..63 with `Region.size (+0x14) != 0`, then `high = max(high, territory)`, signed | yes | yes | agree, **confirmed** (`cmpl $0x0,0x14(%ecx,%eax)` at `006b5856`; `cmovgl` at `006b587c`, signed) |
| `check_borders` folds only when a call this frame returned bit 2 and no sized region 0..63 has cursor `+0x2c` < size | yes | yes | agree, **confirmed** (`testb $0x2,%dl` at `007320c8`; the scan `007320f8`..`00732117`, `jl 0x732177` at `00732109`) |
| plain game: setup (`Setup::build_cities`, `Setup::build_game` twice) and every frame via `GameDaemon::process_all` | yes | yes | agree, **confirmed** (both readers; not re-read beyond the callers) |
| readers: `AchieveChartPane::set_chart_data (007f2c40)` (UI float), `LeaderData::log_data`; no sim reader | yes | yes | agree, **confirmed**: every `0x9dc(` access in `Unit::think_spellcaster`, `ObjectsOut::draw_underlays`, `Options::do_center`, `IFaceSelected::update_visibility`, `SelectGroup::get_item`, `ConquestSetupWin::on_combo_fill_items` is a load on a non-leader class |
| **reset** | `Leader::init (006e3930)` tribe<0 arm memsets `[+0, +0x6929)`; ordinary path skips +0x9dc | **"nothing resets it"**; no whole-struct clear; carries across games in one process | **disagree; first confirmed and extended, second corrected** |
| crate site: `Sim::update_territory_holdings` (`territory/upkeep.rs`), the fold at the two `border_pass.rs` sites | yes | yes | agree, checked below |

### The reset, settled from the listing

1. **`Leader::init (006e3930)`, tribe<0 arm.** `movl 0xc(%ebp),%esi` (`006e393f`) loads `param_2`; `jns 0x6e3a36` at `006e394a` skips the arm when it is >= 0. Inside: `pushl $0x6929; xorl %eax,%eax; pushl %eax; pushl %ebx; calll 0x55e0b2` at `006e398c`..`006e3995`. `0x55e0b2` is a `jmpl *0xac5424` thunk, and `rise_z.map` names it `_memset` (`0055e0b2`, `vcruntime:VCRUNTIME140.dll`; `__imp__memset` at `00ac5424`). cdecl, last push first: `memset(this, 0, 0x6929)`, so `[+0, +0x6929)` is zeroed and +0x9dc lies inside. The arm then falls through into the ordinary path at `006e3a36`. `param_2` is the tribe: the ordinary path stores it with `movl %esi,0xc(%ebx)` at `006e3aff`, and `types.txt` line 42092 names `LeaderData +0xc` `tribe`.
2. **`Leaders::clear (006ed5d0)`**, a writer neither reader named: `movl $0xe40fa4,%esi` (`006ed5d9`, `= 0xe3a390 + 0x6c14`), then `pushl $0x6929; leal -0x6c14(%esi),%eax; pushl $0; pushl %eax; calll 0x55e0b2` at `006ed5e0`..`006ed5ee`, which is `memset(&leaders.list[i], 0, 0x6929)`. The loop steps `0x1bbb` dwords (`0x6eec`) up to `0xe864dc` (decompile of `Leaders::clear (006ed5d0)` line 13 and the loop tail), so it covers **all ten slots**. Its callers are `Game::close (00594b80)` (`00594faa`) and `Game::run_oos_recovery (005869e0)` (`00586dd6`).
3. The ordinary path of `Leader::init` does skip +0x9dc. Its run is `0x9d4`, `0x9d8`, `0x9e0`, ... at `006e4a78`..`006e4a96`, and none of its `rep stos` runs (`+0x68f6` 9 dwords, `+0x480` 6, `+0x6d8` 0x2c, `+0x5762`, `+0x5a22`, `+0x6070`, `+0x6274`) spans it. Both readers had this right. The second reader's net for a whole-struct clear looked for `0x6ee4`/`0x6eec`/`0x45538` sizes, and the clear is `0x6929` bytes, the struct less its non-POD tail. That is why the net missed it.

Who reaches the tribe<0 arm: `Leaders::init (006ed850)` calls `Leader::init(i, -1, i)` with `pushl %edi; pushl $-0x1; pushl %edi` at `006ed8c5`..`006ed8cb` for every slot, and so does `WalkDataGame::walk_data (005a2360)` at `005a30db`..`005a30e6` on the one record at `0xe789dc`. `Setup::build_leader (005abc70)`, `Setup::build_game (005ac190)` (`005acee9`..`005acef7`) and `ScenarioRead::load_leader_chunk (009ac080)` pass **the slot's current tribe** (`pushl 0xe3a39c(%reg)`) as `param_2`, so they also memset any slot whose tribe is still negative. `Leaders::init` is called only from `Game::init_rules_and_teams (00589bb0)` (`00589d04`). In `Game::run (00584590)` that call (decompile line 312) comes before `Game::init` (line 460). `Game::init` reaches `Setup::build_game` (`0058d1b3`), and `Setup::build_game` runs setup's `compute_all_territory`. **So every game starts with `territory_high = 0` before its first fold, and `Game::close` zeroes it again at teardown.** The second reader's "a second game in the same process inherits the previous high-water mark" is wrong.

Verdict: **corrected (second), confirmed and extended (first)**. Deciding lines: `006e398c`..`006e3995` (`Leader::init`'s memset), `006ed5e0`..`006ed5ee` (`Leaders::clear`), and `0055e0b2` = `_memset` in `rise_z.map`.

### Writer completeness (own nets)

- Export: `->territory_high` is read only in `LeaderData::log_data`. `field_0x9dc` hits no leader writer.
- Listing, displacement `0x9dc(`: 60 hits. I mapped each non-`%ebp` store and load to its function: `HotKeyGroup*`, `SelectGroup*`, `Constants::init`, `Game::init` (`0058c50c`, a `Game` field), `Console::*`, `TileSet::init`, `TerrainMap*`, `ConquestSetupWin*`, `CommandPackage::process_hotkey`. Every store is on a non-leader class.
- Listing, absolutes `0xe3ad4x`..`0xe3ad7x`: `$0xe3ad68` appears only as the two folds' base. `0xe3ad68(%eax)` at `006ec5f4` and `0xe3ad68(%edx)` at `009e9006` are `territory` reads (`imull`). There is no `0xe3ad6c` absolute.
- The writers are the two folds plus the two memsets. No writer is missing.

### Crate site

`Sim::update_territory_holdings` is at `crates/sim/src/territory/upkeep.rs:129` (`grep -n`; the reader cited the doc line 128), called from `territory/border_pass.rs:56` (`start_border_pass`, pre-play), `:170` (`settle_borders`) and `:248` (`advance_border_pass` tail). Both readers put the fold there, and that is right: it is called at the setup site and at the pass-complete tail of `advance_border_pass`. **Reset**: the first reader's "only at sim creation" is right for the crate, because one `Sim` is one game, so the second reader's "deliberate cross-game divergence" note is moot. A crate that ever reuses a `Sim` across games must zero the field at game start.

### Rule as a builder should port it

- Field: `territory_high: i32` per player, in tiles (the unit of `territory`).
- Writers: (a) setup's wholesale territory recompute (`compute_all_territory`), and (b) the border pass's tail on a frame where some region finished this frame and no sized land region still has an open cursor (`check_borders`).
- Predicate: slots 0..7 with `leader_flags & 1`.
- Step: after `territory` is re-summed from `reg_terr` over land regions 0..63 with size != 0, set `territory_high = max(territory_high, territory)` (signed).
- Reset: zero at game creation (`Leaders::init` gives every slot `Leader::init(i, -1, i)`, whose tribe<0 arm memsets `[+0, +0x6929)`), and zero at game teardown and at OOS recovery (`Leaders::clear`). It is never lowered in between.
- No sim reader. It is checksummed: the row at `crates/sim/src/checksum/leaders.rs:236`.
- Plain game: yes.
- Confidence: high on all of it, all listing-confirmed. Open: nothing on the rule. Whether `territory`'s crate units equal the original's tiles decides whether the checksum value agrees. That is a property of `territory`, not of this leaf.

## Leaf 2: `reg_terr` (+0x145e, ushort[64])

### Claims lined up

| claim | first | second | status |
|---|---|---|---|
| index: region slot r in 0..63 (records `0x88` apart, land half), no bias | yes | yes | agree, **confirmed** (every access is `0xe3b7ee + 2·(slot·0x3776 + r)`) |
| writer: `World::compute_reg_territory (006b0bb0)` zeros at `006b0c2c` (`movw %cx,(%eax)`, `eax = 0xe3b7ee + 2r` from `006b0c17`) | yes | yes | agree, **confirmed** |
| writer: same function, `incw 0xe3b7ee(,%eax,2)` at `006b185b`, `eax = owner·0x3776 + r` | yes | yes | agree, **confirmed** (`006b184d`..`006b185b`) |
| predicate for `+= 1`: winner `local_14 >= 0` (−1 and −2 count for nobody) | yes | yes (only names −1) | agree, **confirmed**: `movl -0x4(%ebp),%edx` at `006b1755`, then `js 0x6b19e2` at `006b175a` skips the whole tail for a negative owner. `-0x4(%ebp)` is unchanged up to `006b1834`, where it feeds the `imull` |
| −2 = the winning city's +0x5f byte differs from the slot whose city list held it | one-sided (first): "the city's own owner byte" | — | **corrected (name)**: `types.txt` line 44374 ff. names `CityData +0x5f` **`race`** (a `char`), not an owner byte. The arm is decompile of `World::compute_reg_territory (006b0bb0)` lines 467–484: `local_14 = -2`, and it becomes the slot only if `city->race == slot`. The rule (−2 counts for nobody) stands. What `race` holds for a captured city is not established here. The crate's `Owner::Ambiguous` doc (`crates/sim/src/world.rs:172`) says "the winning city's own record of its owner", which is the same naming gap |
| zero predicate: on a call where region r's cursor `+0x2c` is 0 (after `reset` zeroes it), for slots 0..7 (bound `0xc09a50`) with `leader_flags & 1`; also zeroes `reg_known_rares[r]` (+0x4d4) and live cities' `bordering` (+0x65); **before** the budget test | yes | yes | agree, **confirmed**: `cmpl $0x0,0x2c(%ecx,%eax)` / `jne` at `006b0bfc`; `testb $0x1,-0x408(%edx)` at `006b0c21` with `edx = 0xe3a798 = 0xe3a390 + 0x408`; budget `cmpl $0x100,0x24(%eax)` at `006b0ca2`, after the block. `types.txt` names `+0x65` `bordering` |
| one-sided, neither reader: the zero block's city loop runs over `leaders.list[s] +0x408` entries (`cmpl %ecx,(%edx)` at `006b0c31`) and tests `city +0x4 & 1` (`city_flags`); then the region's `+0x8` bit `0x80` is cleared (`006b0c76`). The `incw` tail sets it again (`orl $0x80,0x8(%eax,%ecx)` at `006b186b`) | — | — | finding, outside this leaf |
| early return: cursor >= size, or `GameDaemon::borders (+0x24) > 0xff` | yes | yes | agree, **confirmed** (`006b0c88`..`006b0c8e`; `006b0ca2`, signed `jl`, so −1 = unlimited) |
| writer: `World::compute_all_territory (006b5700)` whole-array clear per slot | "32-dword clear per slot" | "32 dword zero stores" | agree on the effect, **confirmed with a correction to the shape**: eight `movups %xmm0` 16-byte stores (`006b5721`..`006b573c`) = 128 bytes = 64 ushorts, **only for slots with `leader_flags & 1`** (`testb $0x1,-0x145e(%eax)` at `006b5715`), slots 0..7 (bound `0xe72f4e = 0xe3b7ee + 8·0x6eec`, `006b5745`). Then `borders = -1` (`006b575b`), and `compute_reg_territory(r, 1)` for every r with size != 0 |
| writer: `Leader::init (006e3930)` ordinary path, loop based at `this + 0xd64`, `+0x6fa`, i < 0x40 | yes | yes | agree, **confirmed**: `leal 0xd64(%ebx),%edx` at `006e4ae1`; `cmpl $0x40,%esi` / `jge` at `006e4b0d`; `movw %cx,0x6fa(%edx)` at `006e4b1f`; edx steps 2 per i (`006e4b9a`) |
| reset: tribe<0 memset | first yes, second no | — | **confirmed** (Leaf 1: `006e398c`); the same memset covers `+0x145e`. **Extended**: `Leaders::clear (006ed5d0)` also zeroes it (Leaf 1) |
| reader: `BuildTypeData::blocked_location (006375b0)` at `00637911`: `CITY_SPACING − RELAX_CITY_SPACING` when `Region.size·9/10 <= reg_terr[who][r]` | yes | yes | agree; crate reads the same rule at `crates/sim/src/cities/location.rs:305` (`v.region_size(reg) * 9 / 10 <= v.reg_terr(who, reg)`) |
| readers: the `territory` sums in both folds (`006b5860`, `00732140`) | yes | yes | agree, **confirmed** (Leaf 1 listing) |
| mid-pass partial count; the crate's on-demand `Sim::reg_terr` (`cities/place.rs:344`) diverges mid-pass | yes | yes | agree; checked the crate: `zero_region_rares` (`territory/border_pass.rs:125`) says the `+0x145e` write "has no field in this crate", and `advance_border_pass` (`:184`) has the `*at == 0` arm (`:205`) and the per-cell `who.player()` (`:222`) |
| crate sites for a stored field: `advance_border_pass`'s cursor-0 arm and cell loop; `wholesale_bordering` (`:136`) / `settle_borders` (`:166`) / `start_border_pass` (`:48`) for the wholesale recount | yes | yes | agree, **confirmed** that every named function exists |

### Writer completeness (own nets)

Both readers' listing nets (the `0x145e(` displacement, the absolute `0xe3b7ee`, scaled displacements around it) are reproduced by the hits above. The memsets (`Leader::init`'s tribe<0 arm and `Leaders::clear`) are the only writers neither net could see, because they are a size, not a displacement. The second reader lists `Wall::*_stats`, `Fort::*` and `Dock::*` as landing on neighbouring per-region arrays. I did not re-run that sweep. No new writer.

### Port note on the zero's predicate

The original zeroes `reg_terr[s][r]` only for slots with `leader_flags & 1`, both at the cursor-0 block (`006b0c21`) and in `compute_all_territory` (`006b5715`). A slot that has lost bit 0 keeps its last counts, and the checksum walks them. The crate's `zero_region_rares` zeroes for every `ai` leader without a liveness test. If the builder puts `reg_terr`'s zero there, it must gate it on the live bit, or a dead slot's row will part. The same question applies to `reg_known_rares` (not this leaf).

### Rule as a builder should port it

- Field: `reg_terr: [u16; 64]` per player, indexed by the original's region number, in tiles.
- Writers, all inside the border pass:
  - `compute_reg_territory(r, reset)`. If `reset`, set the cursor to 0. If the cursor is 0, then for every slot 0..7 with live bit 0 zero `reg_terr[s][r]` (with `reg_known_rares[s][r]` and the slot's live cities' `bordering`); this runs before the 256-tile budget test, so a region the budget denies is re-zeroed every frame until its first tile. Then, for each tile the pass assigns, `reg_terr[winner][r] += 1` (wrapping u16) when the winner is >= 0. −1 (no claim) and −2 (the winning city's `race` is not the slot whose list held it) count for nobody.
  - `compute_all_territory`: zero all 64 for every live slot 0..7, then run every sized land region with an unlimited budget.
- Reset: the above, plus `Leader::init` (ordinary path loop, and the tribe<0 memset) and `Leaders::clear`.
- Reader that decides: city placement's spacing relaxation (`cities/location.rs:305`).
- Crate site: store it in `territory/border_pass.rs` (cursor-0 arm, gated on live; per-cell loop; wholesale recount), and have `Sim::reg_terr` read the store instead of recounting.
- Plain game: yes.
- Confidence: high on the writers, the predicates and the step, all listing-confirmed. Open: what `CityData::race` holds after a capture, which decides when a tile is −2; and whether the crate's recount already parts in a capture (neither reader checked; `docs/PARKED.md` 568 is the existing booking).

## Leaf 3: `high_buildings` (+0x5660, ushort[129])

### Claims lined up

| claim | first | second | status |
|---|---|---|---|
| index `basic_type(T) − 0x19e`; every access spelled at the biased base `0x5324` / `0xe3f6b4` | yes | yes | agree, **confirmed** (`movzwl 0xe3f6b4(,%ecx,2)` at `00624bfc`, `ecx = owner·0x3776 + basic_type` from `00624bea`..`00624bfa`) |
| `basic_type` = vtable `+0xe4` (PDB `LF_ONEMETHOD`, vftable offset 228); body follows `from` (+0x3c) to the root | yes | yes | agree, **confirmed** (`calll *0xe4(%eax)` at `00624bea`; both cite the same PDB record; not re-derived) |
| the `high_buildings[x + 0x4f]` export hits in `get_support_count`, `get_units`, `has_general`, `IFaceOptions::setup_selected` are `num_units` | yes | yes | agree, **confirmed** by arithmetic: `0x5660 + 2(x+0x4f) = 0x5762 + 2(x−0x32)`, and `0x5762` is `num_units` (the `rep stos` at `006e4bd8`, 0xb0 dwords) |
| writer: `Build::activate (00623e20)`, `movw %bx,0xe3f6b4(,%ecx,2)` at `00624c16`, the only store | yes | yes | agree, **confirmed** |
| `n = num_buildings[T] + get_buildings(to(T))`, recursive over the `to` (+0x2e0) chain, i.e. T and every record T upgrades into | yes ("downstream") | yes ("every type it upgrades into") | agree, **confirmed**: `movzwl 0x5222(%ecx,%edi,2),%ebx` at `00624bc8` (`0x5222 = 0x555e − 2·0x19e`, biased `num_buildings`); `js` on `to` at `00624bd2`; `LeaderData::get_buildings (006e0680)` decompile lines 8–12 recurse while `to >= 0`. Its `reg_buildings[0x3c][to + 0x66]` is the same biased `num_buildings` (`0x14de + 0x3c·0x102 + 2·0x66 = 0x5222`) |
| **predicate**: `mark < n` (signed compare of the zero-extended ushort) **and** `p1 == 0` | yes | yes | agree, **confirmed**: `cmpl %eax,%ebx; jle` at `00624c04`..`00624c06` (signed, both operands non-negative); `cmpl $0x0,0x8(%ebp); jne` at `00624c0c`..`00624c10`. Store truncates to 16 bits |
| in the raise branch: `T == 0x21a` sets `leader_flags |= 0x10000000`; `p3 != 0` with the semaphore/scenario gates pays free units | yes | yes | agree, **confirmed** (`00624c1e`..`00624c35`; `cmpl $0x0,0x10(%ebp)` at `00624c3b`; gates `0x821 & 0x10`, `0x822 & 2`, byte `0xcb195b` at `00624c4a`..`00624c63`) |
| `num_buildings[T]` already counts this building (the base `Wall::activate` runs `Wall::increment_stats` first) | yes | yes | agree; not re-read (both cite `006432e2`). Medium, as both said |
| caller table (p1, p2, p3) | 11 rows | 10 rows | agree on every row both made, **confirmed from the listing** for all 16 `calll *0x1a8(%reg)` sites: `Setup::build_cities` `005ababf` (0,0,0); `Build::check_capture` `00627f82` (0,1,0); `Wall::do_construct` `00643663` (0,1,1); `SpellType::cast_bribe` `006734e2` (0,1,0); `Leader::defeat_by` `006d20ec` (0,1,0); `Leader::produce_building` `006e2e0a` (0,p2,0); `Cities::capture_city` `00733687`, `0073390a` (1,1,0); `City::find_buildings` `007388a8` (0,1,0); `Console::hurry_action` `007c38bd` (0,1,1); `ScenarioEditor::add_build` `0099d134` and `activate_selected` `009a06a2` (0,0,0); `ScenarioRead::load_build_chunk` `009aa8c4`, `009aa9c0` (0,0,0); `ScenarioFuncSet::switch_teams` `009f4833` (1,1,0); `ScenarioFuncSet::create_building` `009f4e9f` (0,1,1). The export net (`+ 0x1a8))(`) adds `ConquestGame::build_cities (00796140)` lines 145 and 498 (0,0,0), `ScenarioRead::load_city_chunk`, `UnitBalance::next`, `ConsoleWin::run_cmd`: all CtW, editor or console paths |
| reset: `Leader::init` only (ordinary path loop and tribe<0 memset); never lowered (`Wall::decrement_stats` leaves it) | yes (both paths) | yes ("only `Leader::init`") | agree, **confirmed**: the loop at `006e4bad`..`006e4bc7` zeroes `[i]` and `[i − 0x81]` (`num_buildings`) for 0x81 entries (`ecx = 2 + 0x7f`); the memset at `006e398c` covers `+0x5660`. **Extended**: `Leaders::clear (006ed5d0)` also zeroes it (Leaf 1) |
| readers: `TypeData::get_cost (00664090)` `00665762` and `BuildData::construct_time (0062d5c0)` `0062d8d8`, Iroquois quick Senate, `high_buildings[T − 0x19e] == 0` indexed by T itself | yes | yes | agree; not re-read (both readers cite the same lines) |
| crate: `Cities::building_high` (`crates/sim/src/cities.rs:51`), `Sim::claim_building_high` (`cities/nations.rs:219`) from `Sim::activate` (`cities/city.rs:2420`, call `:2578`); readers `iroquois_free_senate`, `construct_time_of`; checksum row at `checksum/leaders.rs:297`, no emitter | yes | yes | agree, **confirmed**: the guard `if building_high[w][root] >= n \|\| captured { return }` (`nations.rs:234`) is the original's predicate, with `captured` = p1 |
| crate callers must pass `captured = (p1 != 0)` | yes (as a check) | yes (as a check) | **checked**, below |

### Crate callers against the original's p1/p3

- `Cities::capture_city`: `cities/capture_view.rs:236` passes `activate(n, true, false)`, which matches (1, ·, 0).
- `City::find_buildings`: `cities/city_views.rs:276` passes `(false, false)`, which matches (0, ·, 0).
- `Leader::defeat_by`: `cities/city.rs:680` passes `(false, false)`, which matches (0, ·, 0).
- `Wall::do_construct`: the crate routes this through `cities/construct.rs:49`/`:108`, documented as `activate(0, 1, 1)`. Not traced further.
- **`Build::check_capture`'s own swap branch is unported.** `SimCapture` (`cities/capture.rs:427` ff.) answers `swap_team` with `-1` and has an empty `activate_new` (`capture.rs:661`). So the (0,1,0) raise on a non-city building changing hands through `check_capture` never happens in the crate. Only its `capture_city` arm is live. Whether a plain game reaches that branch was not checked. It is a crate gap, not a disagreement between the readings.
- `SpellType::cast_bribe`: `spells/bribe.rs` ports the unit arm. I found no building-arm activation. Open, as above.
- **`Leader::produce_building`: `ai/ai_place.rs:1035` passes `activate(o, false, true)`, but the original pushes p3 = 0** (`pushl $0x0` at `006e2dff`; decompile of `Leader::produce_building (006e1400)` line 1111, `(0, param_2, 0)`). The mark is unaffected because p1 = 0 on both sides. But `claim_building_high` pays `free_units_for` when `counted`, and the crate's `counted` also gates `claim_commerce_slot`/`claim_gather_slots` (`city.rs:2574`/`2581`, both also gated on `frame != 0`) and the line at `city.rs:2517`. So at this call the crate can pay a nation's free units on a building placed at frame 0, where the original pays none. This is a finding outside the leaf's value. A builder should book it against the free-unit or units score, not this leaf's.

### Rule as a builder should port it

- Field: `high_buildings: [u16; 129]` per player, indexed by `basic_type(T) − 0x19e` (the root of T's `from` chain).
- Writer: `Build::activate(p1, p2, p3)` only. Let `n = Σ num_buildings` over T and every record on T's `to` chain, after this building has been counted. If `n > high_buildings[root]` (signed) and `p1 == 0`, set `high_buildings[root] = n as u16`. Inside that raise, `T == 0x21a` sets `leader_flags` bit `0x10000000`, and `p3 != 0` (with the game/scenario gates) pays the nation's free units.
- p1 is 1 only for `Cities::capture_city` and `ScenarioFuncSet::switch_teams`. Every other caller can raise the mark.
- Reset: zero for all 0x81 entries in `Leader::init` (both paths) and in `Leaders::clear`. Never lowered.
- Readers: the Iroquois quick-Senate cost and time tests, indexed by T itself.
- Crate site: already held. Only the checksum emitter is missing: `building_high[who][rec]` maps crate record `rec` to the original's `type − 0x19e`, zero-padded to 129 (the readers' mapping; the `"num_buildings"` arm is the model).
- Plain game: yes (starting buildings, every finished construction, and buildings changing hands).
- Confidence: high on the writer, the index and the predicate, and on the caller table (all 16 listing sites read). Open: the `increment_stats` predicate (both readers read it from a folded body), and whether a plain game reaches `check_capture`'s non-city swap branch or a building bribe, both unported in the crate.

## Verdict table

| leaf | confirmed / corrected / open | the rule in one line |
|---|---|---|
| `territory_high` | 9 / 1 / 0. Corrected: the second reading's "nothing resets it; carries across games". `Leader::init`'s tribe<0 memset `[+0,+0x6929)` (`006e398c`) and `Leaders::clear` (`006ed5e0`, all ten slots, from `Game::close`) both zero it | on setup's wholesale recount and on the frame the last open border region finishes, for live slots 0..7: `territory = Σ reg_terr[r]` (sized land regions), `territory_high = max(territory_high, territory)`; zero at game creation (`Leaders::init`) and teardown (`Leaders::clear`) |
| `reg_terr` | 12 / 2 / 1. Corrected: −2 is the winning city's `race` (+0x5f), not an "owner byte"; the `compute_all_territory` clear is eight 16-byte stores, live slots only. Open: what `race` holds after a capture | per region r: zero for live slots 0..7 on any call where r's cursor is 0 (before the budget test); `+= 1` per tile the pass gives to a slot ≥ 0; `compute_all_territory` zeroes all 64 for live slots, then recounts |
| `high_buildings` | 12 / 0 / 2. Extended reset (`Leaders::clear`). Crate findings: `check_capture`'s swap branch is unported; `produce_building`'s crate call passes `counted = true` where the original's p3 is 0. Open: the plain-game reach of those branches, and the `increment_stats` predicate | `Build::activate`: at `basic_type(T) − 0x19e`, raise to `n = Σ num_buildings over T's to-chain` when `n > mark` and `p1 == 0` (p1 = 1 only for `capture_city`/`switch_teams`); zero in `Leader::init`/`Leaders::clear`; never lowered |
