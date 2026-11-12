# Item 4152, adjudication: `last-finished`

- Slug: `last-finished`
- Leaves: `last_building_finished` (`LeaderData` +0x6070, `int[129]`), `last_unit_finished` (`LeaderData` +0x6274, `int[352]`)
- Adjudicator: Claude Opus 5.5 (`claude-opus-5-5`), 2026-11-11
- Inputs: `last-finished-reading.md` (first), `last-finished-second.md` (blind second, working-tree copy), the decompile export, the `.text` listing (`exe.s`), `crates/sim`.
- Written per leaf as settled; a leaf without its closing rule below was not finished.

## Common ground (both leaves)

| claim | first | second | mark | verdict, deciding line |
|---|---|---|---|---|
| index bias folded into the displacement: `+0x6070 − 0x19e·4 = +0x59f8`, `+0x6274 − 0x32·4 = +0x61ac`; absolute `0xe3fd88`, `0xe4053c`; slot stride `who·0x1bbb` dwords | yes | yes | agree | **confirmed**: `623f3f imull $0x1bbb` / `623f47 movl %ecx, 0xe3fd88(,%eax,4)`; `62fa10`/`62fa35`. `enums/TypeIndex.txt:65,442` give `BASE_UNITTYPES` 0x32, `BASE_BUILDTYPES` 0x19e |
| init: `Leader::init (006e3930)` fills both with −1, counts 0x81 and 0x160 | yes | yes | agree | **confirmed**: listing `6e4ba8 leal 0x7f(%ecx),%esi` with `ecx` = 2 from `6e4b8a pushl $2; popl %ecx`, so 0x81; `6e4bed`–`6e4bfa` and `6e4bfc`–`6e4c02 movl $0x160`. Decompile `Leader/init.c:725` prints `iVar13 = 0x81` |
| an unseated slot (`param_2 < 0`) takes `memset(this, 0, 0x6929)`, so both arrays are 0 there, and `LeaderData::walk_data` skips them | — | yes | one-sided | **confirmed**: `6e398c pushl $0x6929; xor eax; push ebx; call memset`; `Leader/init.c:322` returns before the fill; `LeaderData/walk_data.c:15–16` walks `[&who, +0x692a)` only under `leader_flags & 1` |
| writer nets: no `->name`; `field_0x6070/6274` only in `Leader::init`; `0x59f8/0x61ac` only in `Build::activate`, `Build::train`, `get_last_type_built`; listing absolutes only at `623f47`, `62fa35`, `9e99fb/9e9a02`, `9e9a21/9e9a28` | yes | yes | agree | **confirmed, complete**, by my own sweeps: (a) every immediate in `[0xe3a390, +9·0x6eec)` whose offset mod 0x6eec falls in `[0x59f8, 0x67f4)` hits only those six sites plus `0xe3fdb2` ×8 (`num_queued` +0x5a22, ushort ×2, whose top index 0x326 ends at +0x606e, short of the arrays); (b) every scaled-index displacement in that range is `0x5a22(…,2)` alone; (c) unscaled displacements in `[0x6070,0x67f4)` belong to `Terrain*`, `GameSpy`, `Conquest*`, `IFaceEditorMain`, `Popup`/`Game::run_solo`/`LoadGame` (all off `game->+0x1328`, e.g. `587ea2`), and `Leader::init`'s two `lea`s; (d) decompile `leaders…field_0x59xx` hits below `0x59f8` are `num_buildings`. No writer either reader missed |
| `walk_data` save/load restore is a writer in kind with no rule | yes | yes (as reader) | agree | confirmed, as above |

## Leaf 1: `last_building_finished` (+0x6070, `int[129]`)

### Claims lined up

| # | claim | first | second | mark | verdict, deciding line |
|---|---|---|---|---|---|
| 1 | writer `Build::activate (00623e20)` stores the building's `o` (`+0xa`, `movswl`) at `[who][type − 0x19e]`; same `o` to `ScenarioData::last_build_built_o[who]` | yes | yes | agree | **confirmed**: `623f31`–`623f56`; `Build/activate.c:154–157` |
| 2 | predicate `captured == 0 && counted != 0` (arg 1, arg 3) | yes | yes | agree | **confirmed** (spot-checked): `623e70 movl 0x8(%ebp),%edi`, `623f26 movl 0x10(%ebp),%ebx`, `623f29 test edi; jne` skip, `623f2d test ebx; je` skip. Both ints, tested for zero only, so sign is moot |
| 3 | the type is read after the Chinese Large City branch, so such a city is recorded under 0x19f (index 1) | yes | — | one-sided | **confirmed**: `623eeb pushl $0x19f; 623ef0 call *0x84` (`vtables.txt` Build slot `+0x84` = `Wall::set_type`), then the store re-reads `0x18(%esi)`→`+0x4` at `623f31`/`623f38`. The branch needs `flags & 0x20`, tribe bonus 0xe, `captured == 0`, `chinese_large_cities`, and the type not 0x213 (`Build/activate.c:133–151`). The crate's head already orders it the same way (`crates/sim/src/cities/activate.rs:65–76`) |
| 4 | the record is an overwrite, not a step | yes | yes | agree | confirmed, `623f47` is a plain `movl` |
| 5 | dispatch: slot `+0x1a8` is `Build::activate` only in `Build::vftable (00b42174)`; `BuildOut`, `BuildData`, `Wall` carry `Wall::activate (0063e4b0)`, so walls never record | yes | yes (`vtables.txt:28415`) | agree | **confirmed**: `vtables.txt` scan of every `+0x1a8` naming an `activate` gives exactly those four. Walls are a different object kind (`ObjectsArray::get_new_data (006450d0)` case 2 constructs `Wall::Wall`, case 1 `Build::Build`; `Objects::init_build (0065d190)` allocates from `[2000,3000)`, walls come from `Objects::init_wall (00658c50)`) |
| 6 | callers that write: `Wall::do_construct (006434d0)` `643663`, `Console::hurry_action (007c3810)` `7c38bd`, `ScenarioFuncSet::create_building (009f4d30)` `9f4e9f`, all `(0,1,1)` | yes | yes | agree | **confirmed**: my own scan of every `call *0x1a8(%r)` and `mov 0x1a8(%r),%r2; call *%r2` with its three pushes |
| 7 | non-writing callers, full list | `ConquestGame::build_cities` ×2 (0,0,0); `ConsoleWin::run_cmd` ×2 (0,1,0); `ScenarioRead::load_build_chunk` ×3 | omits `ConquestGame::build_cities` and `ConsoleWin::run_cmd`; `load_build_chunk` ×2 + `load_city_chunk` ×1 | disagree (bookkeeping) | **corrected, union**: `Setup::build_cities` `5ababf`, `ConquestGame::build_cities` `7962f9`–`796309` and `796d1d`–`796d2d`, `ScenarioEditor::add_build` `99d134`, `::activate_selected` `9a06a2`, `ScenarioRead::load_city_chunk` `9aa409`–`9aa419`, `::load_build_chunk` `9aa8c4`, `9aa9c0` are (0,0,0); `Build::check_capture` `627f82`, `SpellType::cast_bribe` `6734e2`, `Leader::defeat_by` `6d20ec`, `City::find_buildings` `7388a8`, `ConsoleWin::run_cmd` `7dff65`–`7dff73` and `7e055b`–`7e056b`, `UnitBalance::next` `9b9ec6`–`9b9ed6` and `9ba14a`–`9ba15a` are (0,1,0); `Leader::produce_building` `6e2dff`–`6e2e0a` is (0, param_2, 0); `Cities::capture_city` `733687`, `73390a` and `ScenarioFuncSet::switch_teams` `9f4833` are (1,1,0). The first was right on `ConquestGame`/`ConsoleWin`, the second on `load_city_chunk`. None of them writes, so the writer set is unchanged. The other `0x1a8` loads (`ConquestMap::init_mini_map`, `TileSetData::close`, `RTreeData::try_wcoord_entry`) are fields of other classes, not calls |
| 8 | `Wall::do_construct` completes when accumulated work `+0x48` ≥ slot `+0x18c` | yes | yes | agree | **confirmed, with the comparison's sign**: `Wall/do_construct.c:110–112`, an **unsigned** compare (`uVar4 <= *(uint *)&field_0x48`); slot `+0x18c` is `BuildData::construct_time` (`vtables.txt`). The crate's `do_construct_steps` is that site |
| 9 | `Console::hurry_action` reachability | open ("`exec_keymap` not followed") | "debug console's hurry" | one-sided gap | **settled, off the plain path**: `Console/exec_keymap.c:266–269`, key case 0x45, requires `this->cheat_keys != 0` and semaphore bit 4 clear; the other caller is `ConsoleWin::run_cmd` (`7dbd28`) |
| 10 | reset: `Leader::init` −1; the trigger read sets the read cell to −1; nothing else (no death, capture or delete path) | yes | yes (+ "never on death") | agree | **confirmed** by the complete writer nets above |
| 11 | reader `ScenarioFuncSet::get_last_type_built (009e9910)`: 1-based player, rejects `p−1 > 7` and an unseated leader, building range 0x19e..0x21e, read-and-clear | yes | yes | agree | **confirmed**: `ScenarioFuncSet/get_last_type_built.c:27–33, 43–48, 79–80`. The player test is unsigned (`7 < uVar5`), so player 0 is rejected too |
| 12 | no sim decision reads it; checksum and save/load do | yes | yes | agree | confirmed (reader net = writer net's loads: `9e99fb` only) |
| 13 | crate site: `activate_head_steps` (`crates/sim/src/cities/activate.rs:53`) tests the predicate at `:74` and calls the empty `SimHead::record_last_built` (`:139`; `:349` is the test view) | yes | yes | agree | **confirmed** by `grep -n`. Called from `Sim::activate` (`crates/sim/src/cities/city.rs:2420`, head at `:2454`) |
| 14 | the value is the building's `o` | "the original's object-table slot, not the crate's `buildings` index" | "the crate's `Building::index`, `[2000,3000)`" | disagree (wording) | **corrected, reconciled**: both are right about different things. The crate's `buildings` *vector position* `b` is not `o`, but the crate already carries `o` as `Building::index` (`crates/sim/src/cities/building.rs:14–17`, "`SubObjectData::o`, in `[2000, 3000)`"). Store `buildings[b].index` |
| 15 | crate record index = `TypeIndex − 0x19e` | — | yes (`economy/cost.rs:1254`) | one-sided | **confirmed**: `crates/rondata/src/load.rs:220` `build_of_type_index` is `t − BASE_BUILDTYPES`; read `buildings[b].ty` after the head's `set_type` (`activate.rs:118–131` switches it to the Town record) |
| 16 | `Leader::produce_building`'s activate has counted 0; the crate uses that arm at frame 0 | — | yes, "did not read which other callers reach it" | one-sided | **confirmed, and it is a crate finding**: the original pushes `(0, param_2, 0)` at `6e2dff`–`6e2e0a` (`Leader/produce_building.c:1111`), but the crate's frame-0 arm calls `self.activate(o, false, true)` (`crates/sim/src/ai/ai_place.rs:1035`), **counted = true**. With `record_last_built` filled as-is, every building `Setup` produces through it at frame 0 would be recorded, against the 4124 journal's frame-0 columns, which are −1 in every slot (`docs/journal/2026-11-11-item-4124.md:159`). The same flag already gates `claim_building_high`'s `free_units_for` (`crates/sim/src/cities/nations.rs:238`), so the fix wants its own check, not just a flip |
| 17 | plain game reaches it | yes | yes | agree | **confirmed**: every finished construction through `Wall::do_construct`. The start city (`Setup::build_cities`, counted 0), frame-0 produced buildings, captures and bribes do not record |

### The rule, as a builder should port it

- **Storage**: `last_building_finished: [i32; 129]` per seated leader, −1 at
  `Leader::init`, 0 for an unseated slot (never walked).
- **Writer**: in `Build::activate`, when `captured == 0 && counted != 0`, set
  `last_building_finished[who][ty_after_head − 0x19e] = o`, where
  `ty_after_head` is the type after the Chinese Large City `set_type(0x19f)`
  and `o` is the building's `SubObjectData::o` (sign-extended `short`). Walls
  (a separate object kind) never write it.
- **Step**: overwrite. **Reset**: `Leader::init`, and the trigger
  `get_last_type_built`, which reads a cell and sets it to −1. A death never
  clears it.
- **Crate site**: `SimHead::record_last_built`
  (`crates/sim/src/cities/activate.rs:139`): `table[who][buildings[b].ty?] =
  buildings[b].index as i32` (crate record = `TypeIndex − 0x19e`). Before
  that lands, change `crates/sim/src/ai/ai_place.rs:1035` to pass
  `counted = false` as the original does, and re-check what
  `claim_building_high` pays on that path.
- **Plain game**: yes, on every completed construction. Not at game start.
- **Confidence**: high. The store, predicate, index, init and dispatch are
  each settled at a listing address. Diff-backed in part: the 4124 checksum
  values (−1 at the start, building-range `o`s 2007..2015 later). **Open**:
  `ai_place.rs:1035`'s counted flag, which needs a re-gate once the hook is
  filled. Whether the original's building `o` allocation (`Objects::find_free
  (0065ad60)` over `[2000,3000)`) matches the crate's `Building::index` is
  the crate's existing contract, not re-read here.

## Leaf 2: `last_unit_finished` (+0x6274, `int[352]`)

### Claims lined up

| # | claim | first | second | mark | verdict, deciding line |
|---|---|---|---|---|---|
| 1 | writer `Build::train (0062f9b0)` stores `Objects::init_unit`'s return at `[who][type − 0x32]`; same value to `ScenarioData::last_unit_built_o[who]` (`0xcc1930`) | yes | yes (address given) | agree | **confirmed**: `62f9fe call 0x65e0c0`, `62fa16 movl %ecx, 0xcc1930(,%edi,4)`, `62fa35 movl %ecx, 0xe4053c(,%eax,4)` |
| 2 | predicate: the return is ≥ 0 (signed); no exemption inside `train` | yes | yes | agree | **confirmed** (spot-checked): `62fa08 testl %ecx,%ecx; 62fa0a js 0x630431`. Nothing between `62fa0a` and `62fa35` branches |
| 3 | index is `train`'s **argument**, not a type read back from the unit | yes | yes | agree | **confirmed**: `%esi` is pushed as `init_unit`'s type at `62f9f6` and added at `62fa1d addl %esi,%eax` |
| 4 | the value for a squad | not read | not read ("the captain's number, presumably") | open in both | **settled**: `Objects::init_unit (0065e0c0)` makes `UBER_SIZE` figures (`+0x308`) in a loop, and on success returns `get_captain()` of the last figure (`Objects/init_unit.c:223–233`), so the cell holds the **captain's `o`**. If `find_free` fails part-way through, it returns the negative code (`:42–44`) and `train` writes nothing, even though the earlier figures exist |
| 5 | caller set: `Build::finished` 2, `Build::process` 5, `Build::activate` 15, `Leader::gain_tech` 1, `City::assimilate` 1, `Setup::place_unit` 1, `ConsoleWin::run_cmd` 1, `UnitBalance::next` 1 | yes, but totalled "26" | yes | agree (first's total is a slip) | **confirmed: 27 references**, every one a direct `call 0x62f9b0` (my own count over `exe.s`). No vtable holds `Build::train` (`vtables.txt`, 0 hits) |
| 6 | `Setup::place_unit`'s train | "start units placed **at a building** (`param_2 ≥ 0`) are recorded" | "the **fallback** only, after every placement try fails (60 with a starting town, else 500)" | **disagree** | **corrected, the second is right**: `Setup/place_unit.c:36–53`. `param_2 ≥ 0` only moves the search centre onto the building (`:27–34`). Each try that finds a free tile returns through `Objects::init_unit` directly (`:100–102`), which does not write. `Build::train` runs only once the try counter reaches 60 (starting town) or 500 (`local_1c`, nomad), and only with `param_2 ≥ 0`. With `param_2 < 0`, the exhausted budget calls `init_unit` directly instead (`:44–49`). Diff support: the 4124 journal's first sample has unit slot 0 (the citizen, 0x32) at −1 (`docs/journal/2026-11-11-item-4124.md:160`), which is what the second reading predicts at the start |
| 7 | gaia edge: the reader's inlined `is_unit_type` is `0x32..=0x19d`, which takes in the 12 gaia types 0x192..0x19d, indexing 352..363 past the array into `+0x67f4..+0x6823` | yes | gave the same range, missed the overrun | one-sided | **confirmed**: `TypeData::is_unit_type (004707e0)` is `0x31 < type < 0x19e`; `get_last_type_built` takes that arm at `ScenarioFuncSet/get_last_type_built.c:66–71`; `BASE_GAIATYPES` 0x192 and `NUM_UNITTYPES` 0x160 (`enums/TypeIndex.txt:376,428`). `0x6274 + 352·4 = 0x67f4` (`ages_queued`). Trigger layer only; no writer reaches those indices |
| 8 | reset: `Leader::init` −1; the trigger read −1; nothing else, a death included | yes | yes | agree | confirmed by the complete nets |
| 9 | readers: `get_last_type_built` alone, plus checksum and save/load; `get_last_unit_built (009e9a70)` reads the scalar, not this leaf | yes | yes (+ the sibling) | agree | **confirmed**: `9e9a21` is the only load |
| 10 | crate site `Sim::build_train` (`crates/sim/src/cities/queue.rs:1693`) after `init_unit` (`units/store.rs:504`) | yes | yes | agree | **confirmed** by `grep -n`. Non-test callers: `queue.rs:1612`, `:1675`, `cities/activate.rs:391`, `cities/city_views.rs:591`, `cities/nations.rs:432, 524, 583, 655, 686` |
| 11 | how the crate's `ty` maps to the index | "`ty` is a record index; map through `.tree` and subtract 50" | "the crate's unit record is `TypeIndex − 0x32`" | **disagree** | **corrected, the second is right**: `crates/rondata/src/load.rs:1468` loads `type_index: BASE_UNITTYPES + i`, and `:212–216` `unit_of_type_index` is `t − BASE_UNITTYPES`. So the cell is `[ty]` directly (or `unit_types[ty].type_index − 0x32`). `.tree` is a tech-tree `TypeId`, not a `TypeIndex`: subtracting 50 from it would be wrong |
| 12 | the value the crate stores | "the returned unit index must equal the original's `o`" | "`units[unit].index`" | disagree (wording) | **corrected**: the crate's return is a vector position. `o` is `Unit::index` (`crates/sim/src/units/unit.rs:40–47`, "`SubObjectData::o`"). Store `units[unit].index`. The crate's `init_unit` returns the squad head, whose `index` the members take as their captain (`units/store.rs`, the `captain = self.units[h].index` arm), which matches row 4 |
| 13 | crate `Sim::place_unit` stops short of the fallback train | — | yes | one-sided | **confirmed**: `crates/sim/src/units/starting.rs:26–27, 68` returns `Err(PlaceRefused::Train)`. When that SEAM is built, it must go through `build_train` so the write follows |
| 14 | the crate's `init_unit` cannot fail where the original's can | — | — | (adjudicator) | a note for the builder: `Sim::init_unit` returns `usize`, so the original's "negative return → no write" has no crate counterpart until object-table exhaustion is modelled |
| 15 | plain game reaches it | yes | yes | agree | **confirmed**: every queued unit (`Build::finished`) and every free unit from `Build::activate`/`Build::process`. The Americans' bombers (`Leader::gain_tech`) and assimilation's hero (`City::assimilate`) are skirmish-reachable too. The console, `UnitBalance` and the trigger are not |

### The rule, as a builder should port it

- **Storage**: `last_unit_finished: [i32; 352]` per seated leader, −1 at
  `Leader::init`, 0 for an unseated slot (never walked).
- **Writer**: `Build::train(trainer, t)`: once `init_unit` returns `o ≥ 0`,
  set `last_unit_finished[trainer.who][t − 0x32] = o` (for a squad, the
  captain's `o`). Every successful train writes, whatever the kind. A unit
  made by `init_unit` without `Build::train` (normal start placement, gaia,
  triggers) never writes.
- **Step**: overwrite. **Reset**: `Leader::init`, and the trigger's
  read-and-clear. A death never clears it.
- **Crate site**: `Sim::build_train` (`crates/sim/src/cities/queue.rs:1693`),
  right after `init_unit`: `table[who][ty] = units[unit].index as i32`, with
  `ty` the crate's unit record, which already equals `TypeIndex − 0x32`.
  `Sim::place_unit`'s `PlaceRefused::Train` fallback, once built, routes
  through `build_train`.
- **Plain game**: yes, on every trained or free unit. The start units are
  not recorded unless placement falls back.
- **Confidence**: high. The store, predicate, index, init, caller set and
  squad value are settled at listing addresses or decompile lines.
  Diff-backed in part: the 4124 checksum values (−1 at the start, then 21
  and 14). **Open**: nothing in the original. On the crate side, the
  modelled `init_unit` never fails.

## Verdict table

| leaf | claims confirmed / corrected / open | the rule in one line |
|---|---|---|
| common ground | 5 / 0 / 0 | both arrays −1 at `Leader::init` (0 when unseated, unwalked); the bias is folded into `+0x59f8`/`+0x61ac`; the writer nets are complete at six listing sites |
| `last_building_finished` | 14 / 3 / 0 (the 3: caller-list union; value = `Building::index`; crate frame-0 `counted` flag) | `Build::activate` with `captured == 0 && counted != 0` sets `[who][type_after_chinese − 0x19e] = o`; Build objects only; trigger read-and-clear; crate `record_last_built`, after `ai_place.rs:1035` stops passing `counted = true` |
| `last_unit_finished` | 11 / 3 / 0 (the 3: `place_unit` fallback only; crate index = `ty`, not `.tree − 50`; value = `units[unit].index`), plus the open squad value settled as the captain's `o` | every `Build::train` whose `init_unit` returns `o ≥ 0` sets `[who][t − 0x32] = o`; trigger read-and-clear; crate `build_train` after `init_unit` |
