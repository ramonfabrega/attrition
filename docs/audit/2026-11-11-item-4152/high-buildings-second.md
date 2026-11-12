# high-buildings: second (blind) reading

Slug `high-buildings`. Leaves: `territory_high` (LeaderData +0x9dc, int), `reg_terr` (+0x145e, ushort[64]), `high_buildings` (+0x5660, ushort[129]).

Reader: Claude Opus 5.5, blind (has not opened the first reading, the README, CHECKSUM §20, PARKED, the item journals or LEADERS.md). Sources: the decompile export at `~/ghidra-projects/decomp`, `types.txt`, and the llvm-objdump listing of `riseofnations.exe`.

Layout note. `Leader` (0x6eec) holds `LeaderData` (0x6ee4) at offset 0 and adds a vbtable pointer at `+0x6ee4` (`Leader::Leader (006d2bf0)` stores `_vbtable_` there). A `Leader*` method's `field_0xNNN` is therefore `LeaderData +0xNNN` with no shift: `Leader::init (006e3930)` clears `field_0x5762` x 0xb0 dwords (`num_units`) and `field_0x6070` x 0x81 to -1 (`last_building_finished`), which settles it. The global `leaders.list[0]` is at `0xe3a390`, stride `0x6eec`.

## Leaf 1: `territory_high` (`+0x9dc`, int)

### 1. Writers, both nets

- Export by name: `->territory_high` has one hit, a reader (`LeaderData::log_data (006e5110)` line 2003). `field_0x9dc` hits `HotKeyGroup`, `HotKeyGroup::walk_data`, `AchieveChartPane::set_chart_data`; only the last is on a leader (a reader).
- Listing, `0x9dc(%reg)` and `0xe3ad6c`: every `0x9dc(` store is on another class (`Game::init` is a `Game` field, `SelectGroup`, `TerrainMap`, `Console`, `ConquestSetupWin`, `TileSet`, `Constants::init`); no absolute `0xe3ad6c` anywhere. **No direct store exists.**
- **The two writers store through a pointer to the neighbouring `territory` (`+0x9d8`), at `0x4(%edx)`**: `mov $0xe3ad68,%edx` (= `0xe3a390 + 0x9d8`) then `cmp %ecx,0x4(%edx); cmovg 0x4(%edx),%ecx; mov %ecx,0x4(%edx)`.
  - `World::compute_all_territory (006b5700)`, store at `006b5880` (pointer loaded `006b5834`).
  - `GameDaemon::check_borders (00732060)`, store at `00732160` (pointer loaded `0073211b`).
  Both export bodies print it as `piVar[1] = max(*piVar, piVar[1])` off `&leaders.list[0].field_0x9d8`.

### 2. Event, step, predicate

Both writers run the same tail: for slots 0..7 only (the loop ends at `0xe724c8 = 0xe3a390 + 8*0x6eec + 0x9d8`; slots 8 and 9 of the 10-slot array are never visited) whose `leader_flags & 1` is set, `territory := sum over regions r in 0..63 with Region.size (+0x14) != 0 of reg_terr[r]`, then `territory_high := max(territory_high, territory)` (signed compare, `cmovg`). Units: tiles (cells of the region coordinate lists). No scale.

- `compute_all_territory` runs it unconditionally, after it has zeroed every active slot's `reg_terr`, run `compute_reg_territory(r, 1)` over every region 0..63 with size != 0 with `GameDaemon::borders = -1` (no budget), and forced the owner bytes of regions 64..127 to 0xff.
- `check_borders` (once a frame from `GameDaemon::process_all (00732700)`, call at `00732824`) runs it only when (a) some `compute_reg_territory(r, 0)` call this frame returned with bit 2 set, i.e. a region's cursor reached its size this frame, and (b) after that, no region 0..63 with size != 0 still has `Region.borders (+0x2c) < size`. So `territory_high` steps on the frame a border pass completes, by `max(0, territory - territory_high)`.

### 3. Reset

**Nothing resets it.** `Leader::init (006e3930)` zeroes `+0x9d4`, `+0x9d8`, `+0x9e0`, `+0x9e4`, `+0x9e8`, `+0x9ec` (listing `006e4a78`..`006e4a96`) and skips `+0x9dc`; its four `rep stos`/`stos` runs start at `+0x68f6`, `+0x480` and elsewhere and none spans `+0x9dc`. No function memsets a whole `Leader` (no `0x6ee4`/`0x6eec`/`0x45538`-sized clear or copy in the listing). `LeaderData::walk_data (006d6750)` restores it on a load (the raw block `[+0x8, +0x692a)`). So within one process it is zero from BSS and only ratchets up; **a second game in the same process starts with the previous game's high-water mark** (reading only; untested). A capture that runs one game per process cannot see the difference.

### 4. Readers

`LeaderData::log_data (006e5110)` (the dump) and `AchieveChartPane::set_chart_data (007f2c40)` case 5: `(int)(territory_high * 100.0f / world+0x78)`, the post-game chart's territory percent (`0x42c80000` = 100.0f; float, UI only). It is inside the checksum's walked range for an active slot. No sim reader.

### 5. Crate site

`crates/sim/src/territory/upkeep.rs` `Sim::update_territory_holdings`, which sets `holdings.territory` and is called at exactly the two original moments: `border_pass.rs` `advance_border_pass` (the `!open && finished_one` arm = check_borders' tail) and `settle_borders` / `start_border_pass` (setup's `compute_all_territory`). A builder adds `holdings.territory_high = max(territory_high, territory)` there and never resets it in a per-game init (or, if the crate wants one game per `Sim`, notes the cross-game carry as a deliberate divergence). The checksum census lists it at `crates/sim/src/checksum/leaders.rs:236` with no value today.

### 6. Plain game?

Yes. `compute_all_territory` is called by `Setup::build_cities (005ab910)` and `Setup::build_game (005ac190)` (twice) before the first frame, and `check_borders` every frame. Its other callers (`ConquestGame::build_cities`, `ScenarioEditor::*`, `ScenarioRead::import`, `ScenarioFuncSet::gain_upgrade/lose_upgrade/create_building`) are CtW, editor and trigger paths.

### 7. Confidence

High on the writers, the step and the predicate (export and listing agree, both nets run). Medium on "never reset": I grepped every `0x9dc` displacement, the absolute, whole-struct clears and the init path; a `rep movs` of a whole `Leaders` from a save-game buffer would not show in those greps beyond walk_data. The four functions that load `0x6eec` as an immediate (`Game::retake_capital`, `Leader::action_respond`, `Leader::action_declare`, `ConquestStartWin::exec`) use it as a slot stride (checked in the listing), not a copy size.

## Leaf 2: `reg_terr` (`+0x145e`, ushort[64])

**Index**: the region slot `r`, 0..63, of `GameAccess::regions` (`Region`, 0x88 bytes; the first 64 slots, the ones `compute_all_territory` and `check_borders` iterate as `0..0x2200/0x88`; slots 64..127 are the ones whose cells `compute_all_territory` forces unowned, i.e. not land). **Value**: the number of cells of region `r`'s coordinate list (`Region.coords`, list pointer at `+0x7c`) that the current border pass has visited and assigned to this leader. Units: cells. No bias: every access is `0x145e + 2r` (or the absolute `0xe3b7ee + 2*(slot*0x3776 + r)`, `0x3776 = 0x6eec/2`).

### 1. Writers, both nets

- Export by name: `->reg_terr` has no hit at all (the export never names it). `field_0x145e`: `World::compute_all_territory`, `World::compute_reg_territory`, `BuildTypeData::blocked_location` (reader), `GameDaemon::check_borders` (reader); plus `Leader::init`'s biased pointer (below).
- Listing (`0x145e(`, `0xe3b7ee`, every scaled `0x1000..0x14ff(...,2)` displacement and absolute `0xe3b3xx..0xe3b8xx`): the only stores are
  - `World::compute_reg_territory (006b0bb0)`: `lea 0xe3b7ee(,%eax,2)` at `006b0c17` with `movw %cx,(%eax)` (zero) at `006b0c2c`; and `incw 0xe3b7ee(,%eax,2)` at `006b185b`.
  - `World::compute_all_territory (006b5700)`: `mov $0xe3b7ee,%eax` at `006b570a`, then 32 dword zero stores off `%eax` (the whole 64-ushort array).
  - `Leader::init (006e3930)`: inside its `0..0x7f` clear loop the pointer is `field_0xd64 + 2i` and the store `puVar11[0x37d]` lands at `0xd64 + 0x6fa = 0x145e` (+2i), for `i < 0x40`. A biased pointer: the export never prints `0x145e` for it.
  - No other: `Wall::*_stats` (`0xe3b86e` = `reg_buildings`, `0xe3b532`), `Fort::init/close` (`0xe3b66e` = `reg_forts`), `Dock::init/close` (`0xe3b6ee` = `reg_docks`), `plan_strategy` (`0x10de`..`0x13de`) all land on other per-region arrays; none reaches `0x145e..0x14dd`.

### 2. Event, step, predicate

`compute_reg_territory(r, reset)` is the per-region border pass. Per cell visited (cursor `Region.borders` `+0x2c`, advanced by 1 per cell, across all regions at most 256 cells per frame: `GameDaemon::borders` counts them, `0xff < borders` ends the call; `compute_all_territory` sets it to -1 = unlimited) it computes the cell's winning owner `local_14` (the competition loop runs over slots 0..7, starting at slot `x & 7` of the cell, among slots with `leader_flags & 1`) and writes the map owner byte (`WorldData +0x134` cell `+0xf`) and runner-up (`+0x10`). **If the winner is >= 0, `reg_terr[winner][r] += 1`** (16-bit `incw`, `006b185b`); an unowned cell (-1) steps nothing. Only slots 0..7 can win.

Clear at the pass's start: on any call where region `r`'s cursor is 0 (after `reset` has zeroed it, when the caller passed 1), for slots 0..7 (loop bound `0xc09a50` over the 0x1c-byte `cities` lists, 8 entries) with `leader_flags & 1`: `reg_terr[r] := 0`, `reg_known_rares[r] := 0` (`+0x4d4`), and every one of the slot's cities' byte `+0x65` (bordering mask) := 0. This happens **before** the budget test, so a region whose cursor is 0 but whose turn the budget denies is re-zeroed on every frame until its first cell is processed. So mid-pass `reg_terr[p][r]` is a partial count (cells 0..cursor-1 only); it equals "cells of r owned by p" only after r's pass completes, and holds that value until the next pass of r restarts at cursor 0.

A pass is started by setting every region's cursor to 0: `Regions::fix_all_borders (0067f7d0)` (direct callers `Leader::calc_gather`, `Leader::fix_tech_flags`, `Leader::gain_tech`; I did not chase inlined copies), or `compute_all_territory` (`reset = 1` per region).

### 3. Reset

The pass-start clear above (per region, per active slot 0..7); `compute_all_territory`'s whole-array clear for every active slot 0..7 (`006b570a`, before any region runs); `Leader::init`'s clear of the slot. Restored by `LeaderData::walk_data` on a load.

### 4. Readers

`compute_all_territory` and `check_borders` (sum into `territory`, Leaf 1); `BuildTypeData::blocked_location (006375b0)` (`00637911`): a slot holding `reg_terr[r] >= Region.size * 9 / 10` (integer division) of the target region gets `city_spacing - relax_city_spacing` instead of `city_spacing`; `LeaderData::log_data` (`reg_terr[scan]`). It is in the checksum's walked range.

### 5. Crate site

No stored field. `crates/sim/src/cities/place.rs:344` `Sim::reg_terr` recounts "cells of the region the world's owner map gives to `who`" on demand, read by `cities/location.rs:305` (the blocked_location rule). `crates/sim/src/territory/border_pass.rs` already runs the original's pass (`advance_border_pass`, 256-cell budget, `CELLS_PER_FRAME`) and says in `zero_region_rares` that the block's `+0x145e` write "has no field in this crate". A builder stores `reg_terr[player][region]` in the pass: zero it in `zero_region_rares` (the cursor-0 block; it already runs before the budget test there), `+= 1` per cell in `advance_border_pass`'s cell loop when `who.player()` is `Some`, and in `wholesale_bordering`/`settle_borders` (the `compute_all_territory` arm) clear all then count. **The on-demand count diverges from the original mid-pass**: it counts unvisited cells at their old owner where the original has not counted them yet. Whether that already moves the blocked_location rule in a capture was not checked.

### 6. Plain game?

Yes: `Setup::build_cities`/`build_game` (via `compute_all_territory`) and `check_borders` every frame once any fix has reset the cursors (tech gain and gather recalculation call `fix_all_borders`).

### 7. Confidence

High on writers, index, step and clear (export and listing agree; the biased `Leader::init` pointer checked by arithmetic). Medium on the owner competition itself (I read only its shape: rotation start `x & 7`, 8 slots, winner >= 0); its scoring is the territory mechanic the crate already ports and is outside this leaf. Not read: whether `fix_all_borders` is inlined elsewhere.

## Leaf 3: `high_buildings` (`+0x5660`, ushort[129])

**Index**: a building `TypeIndex` minus `0x19e`, so slots 0..0x80 are building types `0x19e..0x21e`. The bias is explicit in `Wall::decrement_stats (00642da0)` (`sub $0x19e,%esi` at `00642e0c` before indexing the sibling `num_buildings` at `0xe3f8ee` = `0xe3a390 + 0x555e`), and it is why every access to this array is spelled at the **biased base `0x5324` = `0x5660 - 2*0x19e`** (absolute `0xe3f6b4`), never at `0x5660`. **The slot written is the lineage root**: `basic_type(t)`, i.e. follow `TypeData.from` (`+0x3c`) until it is negative and take that record's `type` (`+0x4`). **Value**: the most buildings of the root's line (root and every type above it, counted from `t` upward) the player has held at once, as sampled at each activation. Units: buildings. Monotone within a game.

### 1. Writers, both nets

- Export by name: `->high_buildings` hits `LeaderData::log_data` (reader, `high_buildings[scan]` beside `num_buildings[scan]`, `puVar5[-0x81]`), and four `high_buildings[x + 0x4f]` in `LeaderData::get_support_count (006da110)`, `LeaderData::get_units (006e07d0)`, `LeaderData::has_general (006e1050)`, `IFaceOptions::setup_selected (008165b0)`. **Those four are not this field**: `0x5660 + 2(x + 0x4f) = 0x56fe + 2x` = `num_units[x - 0x32]` (unit type ids start at `0x32`), the README's own example. Likewise every absolute `0xe3fa8e`/`0xe3faf2` in the listing (`ScenarioFuncSet::num_type*`, `Leader::compute_unit_score`, `Leader::unit_prod_value`) is `num_units`, not `high_buildings`, though the address falls inside `[0xe3f9f0, 0xe3faf2)`.
- Export, `field_0x5660`: `Leader::init` (the clear) and `TerrainOut::one_time_init` (another class). `field_0x5324`: `Build::activate (00623e20)` lines 614/616 (the writer), `BuildData::construct_time (0062d5c0)` line 126, `TypeData::get_cost (00664090)` line 700.
- Listing (`0x5660(`, `0xe3f9f0`, `0x5324(`, `0xe3f6b4`, every scaled `0x52xx..0x57xx` displacement, every `0x102(` relative to `num_buildings`): **one store**, `Build::activate (00623e20)` at `00624c16`, `movw %bx,0xe3f6b4(,%ecx,2)` with `ecx = slot*0x3776 + basic_type`. Plus `Leader::init`'s clear (`lea 0x5660(%ebx)` at `006e4bad`, zeroing `[i]` and `[i-0x81]`, i.e. `num_buildings`, for 0x81 entries).

### 2. Event, step, predicate

`Build::activate(this, p1, p2, p3)` (vtable slot `0x1a8`, PDB `LF_ONEMETHOD activate`, vftable offset 424, on the building base class), listing `00624ba4`..`00624c16`:

1. `t` = the building's own type (`this->type->type`), slot `s = this+0x9`.
2. `n = num_buildings[t] + get_buildings(to(t))`, where `to` is `BuildTypeData +0x2e0` (`to`) and `LeaderData::get_buildings (006e0680)` adds `num_buildings` of `to(t)`, `to(to(t))`, ... recursively until `to < 0`. So `n` counts `t` and every type it upgrades into.
3. `r = basic_type(t)` (vtable `+0xe4`: PDB names vftable offset 228 on `ObjectTypeData` `basic_type`; `BuildTypeData::basic_type (00639970)` overrides it: `from < 0` ? `type` : `buildtypes[from]->basic_type()`).
4. **If `high_buildings[r - 0x19e] < n` (signed int compare of the zero-extended ushort) and `p1 == 0`, store `high_buildings[r - 0x19e] := n` (truncated to 16 bits).** On a raise, `t == 0x21a` also sets `leader_flags |= 0x10000000`, and `p3 != 0` (with the game/scenario gates) pays the type-specific free units (not this leaf).

`num_buildings[t]` already includes this building when the counting predicate holds: `Build::activate` calls the base activate `Wall::activate (0063e4b0)` at `00623f65`, early, which calls the stat increment `Wall::increment_stats (00643270)` (`incw 0xe3f5b2(,slot*0x3776+t,2)` at `006432e2` = `num_buildings[t-0x19e]++`) when `build_flags (+0x2c0) & 0x10` or the building has a city. (`Wall::*` names are LTCG-folded bodies shared with `Build`; I read only the call order.)

Call sites of slot `0x1a8` and their `(p1, p2, p3)` (pushes read in the listing where the export dropped arguments): `Wall::do_construct (006434d0)` (0,1,1) = construction finished; `Setup::build_cities (005ab910)` (0,0,0) = starting buildings; `Leader::produce_building (006e1400)` (0,p2,0); `City::find_buildings (007384c0)` (0,1,0), `Build::check_capture (006276a0)` (0,1,0), `Leader::defeat_by (006d1c80)` (0,1,0) = buildings changing hands, **which can raise the mark (p1 = 0) but pay nothing (p3 = 0)**; `Cities::capture_city (00733380)` (1,1,0) and `ScenarioFuncSet::switch_teams` (1,1,0) cannot raise it; `ScenarioEditor::*`, `ScenarioRead::load_*_chunk`, `ConquestGame::build_cities` (0,0,0), `ScenarioFuncSet::create_building` (0,1,1).

### 3. Reset

Only `Leader::init (006e3930)` (zero, all 0x81). Never decreased: `Wall::decrement_stats (00642da0)`, the only decrementer of the sibling counts, touches `num_buildings` and `reg_buildings` (saturating at 0) and not this array. Restored by `LeaderData::walk_data` on a load.

### 4. Readers

`Build::activate` itself (the compare); `BuildData::construct_time (0062d5c0)` and `TypeData::get_cost (00664090)`: with tribe bonus `0x12`, type `0x1b6` and `iroquois_quick_senate != 0`, `high_buildings[t - 0x19e] == 0` (indexed by the type itself, not its root) makes the construct time 0 (clamped to 1 after) and the cost 0; `LeaderData::log_data`. In the checksum's walked range.

### 5. Crate site

Already ported: `crates/sim/src/cities/nations.rs` `Sim::claim_building_high`, called from `crates/sim/src/cities/city.rs:2578` in `Sim::activate`; the field is `cities.building_high` (`crates/sim/src/cities.rs:45`), indexed by the crate's build-record index of the root (`base_build_type` follows `from`; `ai/ai_types.rs` `buildings_of_line` follows `to`), not by `TypeIndex - 0x19e`. A checksum builder needs only the map from crate record to `TypeIndex - 0x19e`; `crates/sim/src/checksum/leaders.rs:297` lists the leaf with no value fed. Two things for whoever audits it: the crate's `captured` flag must mean `p1 != 0` (only `capture_city` and `switch_teams`), because the `check_capture`/`find_buildings`/`defeat_by` paths pass `p1 = 0` and do raise the mark; and the crate's `counted` (p3 / the increment predicate) is a separate gate on the free units only.

### 6. Plain game?

Yes: every finished construction (`Wall::do_construct`), the starting buildings (`Setup::build_cities`), and captures. CtW, editor and trigger callers exist but are not needed to reach it.

### 7. Confidence

High on the single writer, the biased index, the root slot and the predicate `n > mark && p1 == 0` (export and listing agree; the `0x19e` bias is an explicit `sub` in the listing; `basic_type` is named by the PDB's vftable offset and its body read). Medium on the call-site argument table (read from pushes; I did not read each caller's branch conditions), and on the increment predicate inside `increment_stats` (read from the export of a folded body). Not read: what `Leader::produce_building` is for, and whether the `0x1a8` slot has callers through other spellings (`+ 0x1a8))` in the export, `*0x1a8(` in the listing were the nets).

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `territory_high` +0x9dc | `World::compute_all_territory (006b5700)` store `006b5880`; `GameDaemon::check_borders (00732060)` store `00732160`; both at `0x4(&territory)` | `max(territory_high, territory)` after `territory` is re-summed: setup, and the frame a border pass's last region finishes; slots 0..7 with `flags & 1` | none (not in `Leader::init`); carries across games in one process; walk_data on load | add to `territory/upkeep.rs` `update_territory_holdings` | yes | high (writers, step); medium (no reset) |
| `reg_terr` +0x145e [region 0..63] | `World::compute_reg_territory (006b0bb0)` zero `006b0c2c`, `incw` `006b185b`; `World::compute_all_territory (006b5700)` whole-array zero from `006b570a`; `Leader::init (006e3930)` via `field_0xd64` pointer | `+1` per cell the pass assigns to the slot (winner >= 0, slots 0..7); 256 cells/frame across regions | per region at cursor 0 for active slots, before the budget test (re-zeroed each frame until the first cell); whole array in `compute_all_territory`; `Leader::init` | no field; `cities/place.rs:344` recounts the owner map (diverges mid-pass); store it in `territory/border_pass.rs` (`zero_region_rares`, `advance_border_pass`, `wholesale_bordering`) | yes | high |
| `high_buildings` +0x5660 [TypeIndex - 0x19e, at the lineage root] | `Build::activate (00623e20)` store `00624c16` (biased base `0xe3f6b4`); `Leader::init (006e3930)` clear | `:= n` when `n = sum num_buildings over t, to(t), ... > mark` and `p1 == 0` | `Leader::init` only; never decreases | ported: `cities/nations.rs` `claim_building_high`, `cities.building_high`; checksum leaf unfed | yes | high (writer, index, predicate); medium (call-site args) |

Time and cost: about 75 tool calls, roughly 70 minutes; the expensive parts were the biased bases (`0x5324`, `0xd64`, `0x4(&territory)`) that neither name grep nor offset grep finds, and naming vtable `+0xe4` from the PDB.
