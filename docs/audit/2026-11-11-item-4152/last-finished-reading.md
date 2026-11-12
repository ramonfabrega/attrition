# Family 19 — `last-finished`: first reading

Leaves: `last_building_finished` (LeaderData +0x6070, int[129]), `last_unit_finished` (LeaderData +0x6274, int[352]).
Reader model: Opus 5.5 (claude-opus-5-5), 2026-11-11. Reading complete; sections were appended per leaf.

## Common ground

- **Index.** Both arrays are indexed by the `TypeIndex` of a type record
  minus that kind's base: `last_building_finished[type − 414]`
  (`BASE_BUILDTYPES` = 0x19e = 414, `NUM_BUILDTYPES` = 0x81 = 129, so types
  414..542) and `last_unit_finished[type − 50]` (`BASE_UNITTYPES` = 0x32 =
  50, `NUM_UNITTYPES` = 0x160 = 352, so types 50..401). Both bases come
  from `enums/TypeIndex.txt`. The original folds the bias into the
  displacement, which is why the field offsets never show up at a store:
  `+0x6070 − 414·4 = +0x59f8` and `+0x6274 − 50·4 = +0x61ac`. Off the
  leaders array that gives `0xe3a390 + 0x59f8 = 0xe3fd88` and
  `0xe3a390 + 0x61ac = 0xe4053c`, with the slot folded in as
  `who·0x1bbb` dwords (0x1bbb·4 = 0x6eec).
- **Value.** Neither is a frame or a count. Each holds the **object number
  `o`**, the owner's object-table slot (`SubObjectData +0xa`, `short`,
  sign-extended to int), of the **most recent** object of that type the
  leader finished. `-1` means none, or that a trigger has already consumed
  it (below). The item 4124 journal's checksum columns
  (`docs/journal/2026-11-11-item-4124.md:159–160`) match this: building
  slots hold 2007, 2009, 2012 and 2015 (building-range `o`s) and unit slot
  0 holds 21 and then 14.
- **Init.** `Leader::init (006e3930)` sets every dword of both arrays to
  `0xffffffff` (-1), with `rep stosl` at `6e4bef` (count 0x81) and at
  `6e4bfc` (count 0x160) (its export file, lines 744–753). The
  count register for the first loop comes from `lea 0x7f(%ecx)` and the
  decompiler propagates it as `0x81`. 0x81·4 = 0x204 = 0x6274 − 0x6070, so
  that count fills the array exactly. `Leader::init` is called from
  `Setup::build_leader`, `Setup::build_game`, `Leaders::init`,
  `MapScenario::make`, `WalkDataGame::walk_data`,
  `ConquestGame::build_conquest_game`, `ScenarioEditor::new_leader` and
  `ScenarioRead::load_leader_chunk` (×6) (direct calls to `0x6e3930`).
- **Every store, both nets.** The decompile export has **no** hit for
  `->last_building_finished` or `->last_unit_finished`. `field_0x6070` and
  `field_0x6274` hit only in `Leader::init`, and `field_0x59f8` and
  `field_0x61ac` only in `Build::activate` and `Build::train`. The full
  listing (`exe.s`) gives `0x6070`/`0x6274` only in `Leader::init`;
  `0xe40400`/`0xe40604` nowhere; `0xe3fd88` at `623f47` (`Build::activate`,
  store) and `9e99fb`/`9e9a02` (`ScenarioFuncSet::get_last_type_built`, load
  then store of -1); `0xe4053c` at `62fa35` (`Build::train`, store) and
  `9e9a21`/`9e9a28` (the same trigger function). Seven `0x59f8(%reg)`
  byte ops (`testb`/`orb`/`movb` at `7747d0`..`77c49f`) sit in
  `ConquestWin::draw_text`, `ConquestWin::on_key_click` and
  `ConquestWin::ConquestWin`. Their base is the `ConquestWin` `this`, not a
  `Leader`, so they are a different field. `0x61ac(%reg)` appears nowhere.
- **Save/load.** Both arrays sit inside `LeaderData::walk_data (006d6750)`'s
  whole-slot range `[+0x8, +0x692a)` (`docs/CHECKSUM.md` §8). They
  therefore go into the checksum stream for the three whole-walked slots,
  and are restored by `Leaders::walk_data (006e38e0)` through
  `WalkDataGame::walk_data (005a2360)` when a game is loaded. That restore
  is a writer in kind, but it carries no game rule.

## Leaf 1 — `last_building_finished` (+0x6070, int[129])

**1. Writers.**

- `Leader::init (006e3930)`: fills the array with -1 (above).
- `Build::activate (00623e20)`, store at `623f47`
  (its export file, lines 154–158):
  `last_building_finished[who][ptype->type − 414] = (int)this->o`. The same
  guarded block also writes `ScenarioData::last_build_built_o[who] = o`
  (`0xcc1950`, at `623f56`). The type is read fresh from `ptype` (`+0x18`,
  then `TypeData +0x4`) at the store, *after* the Chinese large-city branch
  above it has called slot `+0x84(0x19f)`. A Chinese city founded as a
  Large City is therefore recorded under type 0x19f (index 1), not 0x19e.
- `ScenarioFuncSet::get_last_type_built (009e9910)`, store at `9e9a02`:
  writes -1 into the slot it has just read (a destructive read; see
  readers).
- Save/load restore through `Leaders::walk_data` (above).

**2. Event, step, predicate.** The record is a plain assignment of the
building's `o`, not an increment. It happens in `Build::activate(captured,
announce, counted)` exactly when **`captured == 0 && counted != 0`**
(listing `623f29`–`623f2f`: `edi` = arg 1, `ebx` = arg 3). It comes before
the call to `Wall::activate`, and no kind of building is exempt beyond
that. Every caller of vtable slot `+0x1a8`, by the push order in the
listing (16 `call *0x1a8(…)` sites and 7 register-indirect ones, all
read):

| caller | (captured, announce, counted) | writes? |
|---|---|---|
| `Wall::do_construct (006434d0)` `643663`, when accumulated work `+0x48` ≥ slot `+0x18c` (construct time) | (0,1,1) | **yes**: construction completes |
| `Console::hurry_action (007c3810)` `7c38bd` (keymap/debug "hurry" on the selection) | (0,1,1) | yes |
| `ScenarioFuncSet::create_building (009f4d30)` `9f4e9f` (trigger) | (0,1,1) | yes |
| `Setup::build_cities`, `ConquestGame::build_cities` (×2), `ScenarioEditor::add_build`, `ScenarioEditor::activate_selected`, `ScenarioRead::load_build_chunk` (×3) | (0,0,0) | no |
| `Build::check_capture`, `SpellType::cast_bribe`, `Leader::defeat_by`, `City::find_buildings`, `ConsoleWin::run_cmd` (×2), `UnitBalance::next` (×2) | (0,1,0) | no |
| `Leader::produce_building (006e1400)` `6e2e0a` | (0,arg,0) | no |
| `Cities::capture_city` (×2), `ScenarioFuncSet::switch_teams` | (1,1,0) | no |

The decompile prints some of these with fewer arguments: `City::find_buildings`
as `(0,1)` and `Leader::defeat_by` as `(0)`. The listing shows three pushes
at each. There is no direct call to `0x623e20`.

**Dispatch predicate.** `Build::activate` is slot `+0x1a8` only of
`Build::vftable (00b42174)`, which `Build::Build (00629e00)` installs last.
Its base `0xb42354` (labelled `BuildOut::vftable`, installed first by
`Build::Build` and by `BuildOut::BuildOut (0062d180)`), `BuildData`'s
`0xb426dc` and `Wall`'s `0xb42cf8` all carry `Wall::activate (0063e4b0)` at
`+0x1a8` (PE bytes read). No other vtable overrides the slot. So walls,
which are `Wall` objects, finish construction through the same
`Wall::do_construct` but **never** write this array. Every `Build`
object, cities and wonders included, does.

**3. Reset.** `Leader::init` sets it to -1. A trigger read sets one slot to
-1. Nothing clears it per frame, `reset_score` does not touch it, and
nothing else writes it. A later finished building of the same type
overwrites the slot.

**4. Readers.** Only `ScenarioFuncSet::get_last_type_built (009e9910)`
(a trigger function, `(player 1-based, type name)`). It rejects
`player−1 > 7` and a leader whose `leader_flags & 1` is clear. For a
building type in 0x19e..0x21e it returns
`last_building_finished[p][type − 414]` and **sets it to -1**. Beyond that
the checksum walk and save/load read it. No sim decision reads it.

**5. Crate site.** The event is already ported. `Sim::activate`
(`crates/sim/src/cities/city.rs:2420`) runs
`crate::cities::activate::activate_head_steps`
(`crates/sim/src/cities/activate.rs:53–80`), which already tests
`captured == 0 && counted != 0` and calls `ActivateHeadView::record_last_built`.
Both implementations of that hook are empty (`activate.rs:139`, `:349`).
A builder fills `record_last_built` with `leader[who].last_building_finished[ty − 414] = o`,
using the type *after* the Chinese `set_type(0x19f)`. The crate also
needs the building's original `o`, which the item 4124 values (≈2007+)
show is the original's object-table slot, not the crate's `buildings`
index. `crates/sim/src/checksum/leaders.rs:300` already lays out the field
and `crates/rondata/src/diff/checksum_lines.rs:1805` marks it unheld.

**6. Plain game?** Yes. `Wall::do_construct` reaches it whenever a
building finishes construction in a skirmish. The game-start cities
(`Setup::build_cities`, counted = 0) are **not** recorded. Captures
(`Cities::capture_city`, captured = 1) are not recorded. The trigger
writer and reader, `Console::hurry_action` and the scenario/CtW paths sit
behind the trigger, debug-console or CtW layers.

**7. Confidence and gaps.** High for the stores, predicate, index and
init, because both nets agree and the listing confirms each push.
Not read: whether `Console::hurry_action` is reachable without a cheat
keymap (`Console::exec_keymap` was not followed), and how the original
assigns building `o`s (`Objects`' allocator), which a builder needs in
order to produce the same value. The diff-backed part is the
checksum-stream values in the item 4124 journal. The predicate itself
rests on this reading alone.

## Leaf 2 — `last_unit_finished` (+0x6274, int[352])

**1. Writers.**

- `Leader::init (006e3930)`: fills the array with -1 (above).
- `Build::train (0062f9b0)`, store at `62fa35`
  (its export file, lines 72–81). `Objects::init_unit(who, type,
  x, y, -1, -1, -1)` returns the new unit's `o`. If it is negative,
  `train` returns it and **nothing is written**. Otherwise `train` writes
  `ScenarioData::last_unit_built_o[who] = o` and
  `last_unit_finished[who][type − 50] = o`. Both happen unconditionally,
  before the stance, `go_inside` and garrison logic. The index is
  `train`'s **argument** `type`, not a type read back off the created
  unit.
- `ScenarioFuncSet::get_last_type_built (009e9910)`, store at `9e9a28`:
  writes -1 into the slot it has just read.
- Save/load restore through `Leaders::walk_data`.

**2. Event, step, predicate.** The record is a plain assignment of the new
unit's `o` (the head figure's object slot: `init_unit`'s return), and it
happens **every time `Build::train` creates a unit**. So the predicate is
the set of `Build::train`'s callers. A unit made by `Objects::init_unit`
directly, without `Build::train`, is never recorded. All 26 direct calls to
`0x62f9b0`:

| caller | count | what |
|---|---|---|
| `Build::finished (00628490)` | 2 | a queued unit completes (`train(this, type)`, line 69, after the caravan and aircraft limits pass); the government-hero branch (line 167) |
| `Build::process (0061edf0)` | 5 | per-frame wonder or nation free units: the Kremlin spy (type 0x21a, `(o + frame) % 15 == 0`, `kremlin_spy` countdown), the government hero at a type-0x1b6 building or a city on the frame after the hero stamp `+0xa48`, the tribe-bonus-0x17 caravan at a 0x1b4, and the Terra Cotta free unit at a 0x211 (lines 315–413) |
| `Build::activate (00623e20)` | 15 | free units when a building activates: the government hero, first-of-type nation bonuses (for example the Nubian caravan at a 0x1b4). Most sites are gated on `captured == 0` and `counted != 0`, plus the semaphore check or `ScenarioData::building_unit_bonus` (lines 224–1128) |
| `Leader::gain_tech (006dcb60)` | 1 | `americans_free_bomber` bombers at an airbase on a tech (line 640) |
| `City::assimilate (00738e90)` | 1 | the government hero at an assimilated city (line 139) |
| `Setup::place_unit (005abca0)` | 1 | game-start units placed **at a building** (`param_2 ≥ 0`). Start units with no building go through `Objects::init_unit` and are not recorded (lines 44–54) |
| `ConsoleWin::run_cmd (007d6a70)` | 1 | console command |
| `UnitBalance::next (009b8ac0)` | 1 | development balance harness |

No kind is exempt inside `train` itself. Every successful creation
records, aircraft, boats and heroes included.

**Edge.** In the reader, `TypeData::is_unit_type` is inlined as
`0x32 ≤ type ≤ 0x19d` (50..413). That range takes in the 12 gaia types
402..413 (`BASE_GAIATYPES` = 0x192), which index 352..363, past the end
of the 352-entry array. A trigger naming a gaia type would read, and set
to -1, the bytes at `+0x67f4..+0x6823` (`ages_queued`, `epochs_queued`,
`reg_attacked[..]`). No writer reaches those indices unless `Build::train`
were called with a gaia type, and no caller listed does that.

**3. Reset.** `Leader::init` sets it to -1. A trigger read sets one slot to
-1. There is no per-frame fold and no `reset_score`. The next unit of the
same type trained through `Build::train` overwrites the slot.

**4. Readers.** Only `ScenarioFuncSet::get_last_type_built (009e9910)`.
For a unit type it returns `last_unit_finished[p][type − 50]` and sets it
to -1 (`9e9a21`/`9e9a28`, listing). Beyond that the checksum walk and
save/load read it. No sim decision reads it.

**5. Crate site.** `Sim::build_train`
(`crates/sim/src/cities/queue.rs:1693`) is `Build::train`. It calls
`self.init_unit(who, ty, pos)` (`crates/sim/src/units/store.rs:504`), and
that is where the counter goes:
`leader[who].last_unit_finished[tree(ty) − 50] = o`. The crate's `ty` is
a record index into `tables.unit_types`, not a `TypeIndex`
(`ai_types.rs:34`, `unit_record` maps a tree id to the record). The
builder must map it through `.tree` and subtract 50. The callers already
funnel through it: `queue.rs:1612`, `:1675`, `cities/nations.rs:432–686`,
`cities/city_views.rs:591` and `cities/activate.rs:391`. The returned
unit index must equal the original's `o`. `crates/sim/src/checksum/leaders.rs:301`
lays out the field and `crates/rondata/src/diff/checksum_lines.rs:1808`
marks it unheld.

**6. Plain game?** Yes. `Build::finished` reaches it every time a queued
unit completes in a skirmish, and so do the wonder, nation, gov-hero and
start-unit paths. The trigger reader/clearer, `ConsoleWin::run_cmd` and
`UnitBalance::next` sit behind the trigger, console or development
layers.

**7. Confidence and gaps.** High for the store, index, value and init,
because both nets agree. I did not follow the predicates of the 15
`Build::activate` and 5 `Build::process` train sites one by one. They
decide *whether* a unit is made, not how the record is written, which
is unconditional once `init_unit` succeeds. I also did not read
`Objects::init_unit`'s failure cases or its `o` allocation. The
checksum values in the item 4124 journal (unit slot 0 = 21, then 14) are
the diff-backed evidence that the stored value is a small object number.

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `last_building_finished` [type−414] | `Build::activate (00623e20)` (`623f47`); `ScenarioFuncSet::get_last_type_built (009e9910)` (−1, `9e9a02`); `Leader::init (006e3930)` (−1 fill) | `= o` of the building, when `captured == 0 && counted != 0`, i.e. `Wall::do_construct` completion, `Console::hurry_action` and the trigger `create_building`; `Build` objects only, walls never | `Leader::init` → −1; trigger read → −1; load restores | `activate_head_steps` → `record_last_built` (empty at `cities/activate.rs:139`, `:349`) | yes (construction complete); start cities and captures excluded | high |
| `last_unit_finished` [type−50] | `Build::train (0062f9b0)` (`62fa35`); `ScenarioFuncSet::get_last_type_built (009e9910)` (−1, `9e9a28`); `Leader::init (006e3930)` (−1 fill) | `= o` of the new unit on every successful `Build::train` (queue completion, free units, gov hero, start units at a building) | `Leader::init` → −1; trigger read → −1; load restores | `Sim::build_train` (`cities/queue.rs:1693`), after `init_unit` | yes (every trained unit) | high |
