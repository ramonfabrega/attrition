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

