## 19. The last building and unit finished

### Leaves

- `last_building_finished`: +0x6070, `int[129]`, indexed by building
  `TypeIndex − 0x19e` (`BASE_BUILDTYPES`).
- `last_unit_finished`: +0x6274, `int[352]`, indexed by unit
  `TypeIndex − 0x32` (`BASE_UNITTYPES`).

The executable folds each index bias into the displacement, so its stores
address +0x59f8 and +0x61ac (absolute `0xe3fd88` and `0xe4053c`, slot
stride 0x1bbb dwords). A search for the leaf offsets alone misses every
writer.

### The rule

**Shared by both leaves.**

- **Init**: `Leader::init (006e3930)` fills both arrays with −1 (0x81 and
  0x160 cells) for a seated leader. An unseated slot takes the bulk memset,
  so both are 0 there, and `LeaderData::walk_data (006d6750)` never walks
  them.
- **Step**: an overwrite with an object number `o`, never an increment.
- **Reset**: `Leader::init`, and the trigger reader's read-and-clear (below).
  Nothing else clears a cell: no death, capture or delete path.
- **Writer nets are complete**: the two stores below, `Leader::init`, the
  trigger reader, and the save/load restore. Every other hit in the range is
  another class or `num_queued` (+0x5a22, which ends at +0x606e).
- **Reader**: `ScenarioFuncSet::get_last_type_built (009e9910)` alone, plus
  the checksum and save/load. No sim decision reads either leaf. The reader
  takes a 1-based player and rejects `p − 1 > 7` as an unsigned test (so it
  rejects player 0 too). It also rejects an unseated leader. It picks the
  leaf by type: buildings are 0x19e..0x21e, units are `TypeData::is_unit_type
  (004707e0)`, `0x31 < t < 0x19e`. It returns the cell and sets it to −1.
  The unit arm takes in the 12 gaia types 0x192..0x19d, which index cells
  352..363, past the array into +0x67f4..+0x6823 (`ages_queued` onward). No
  writer reaches those indices, so the overrun lives in the trigger layer
  only. `get_last_unit_built (009e9a70)` reads a `ScenarioData` scalar, not
  this leaf.

**`last_building_finished`.**

- **Writer**: `Build::activate (00623e20)` with arguments
  `(captured, _, counted)`. If `captured == 0 && counted != 0`, it sets
  `last_building_finished[who][ty − 0x19e] = o`. `o` is the building's
  `SubObjectData::o`, a sign-extended `short` at +0xa. The same `o` also goes
  to `ScenarioData::last_build_built_o[who]`, which is not this leaf.
- **The type is read after the Chinese Large City branch.** The branch needs
  `flags & 0x20`, tribe bonus 0xe, `captured == 0`, `chinese_large_cities`,
  and a type other than 0x213. When it fires it calls `set_type(0x19f)`,
  and the store re-reads the type afterwards, so such a city records at
  index 1.
- **Dispatch**: vtable slot +0x1a8 is `Build::activate` only in
  `Build::vftable (00b42174)`. `Wall`, `BuildOut` and `BuildData` carry
  `Wall::activate (0063e4b0)`, so walls (a separate object kind) never write.
- **Callers that write** all pass `(0, 1, 1)`:
  - `Wall::do_construct (006434d0)`, on completion: accumulated work +0x48
    ≥ `BuildData::construct_time` (vtable slot +0x18c), as an **unsigned**
    compare;
  - `Console::hurry_action (007c3810)`;
  - `ScenarioFuncSet::create_building (009f4d30)`.
- **Callers that do not write**:
  - (0,0,0): `Setup::build_cities (005ab910)`, `ConquestGame::build_cities`,
    `ScenarioEditor::add_build` and `::activate_selected`,
    `ScenarioRead::load_city_chunk` and `::load_build_chunk`.
  - (0,1,0): `Build::check_capture (006276a0)`, `SpellType::cast_bribe
    (00672d00)`, `Leader::defeat_by`, `City::find_buildings`,
    `ConsoleWin::run_cmd`, `UnitBalance::next`.
  - (0, `param_2`, 0): `Leader::produce_building (006e1400)`.
  - (1,1,0): `Cities::capture_city (00733380)`,
    `ScenarioFuncSet::switch_teams`.

**`last_unit_finished`.**

- **Writer**: `Build::train (0062f9b0)`, given a trainer and a type `t`,
  calls `Objects::init_unit (0065e0c0)`. If the return is ≥ 0 (signed), it
  sets `last_unit_finished[trainer.who][t − 0x32] = o`, where `o` is that
  return. The same value also goes to `ScenarioData::last_unit_built_o[who]`
  (`0xcc1930`). The index is `train`'s argument `t`, not a type read back
  from the unit. Nothing inside `train` exempts a kind: every successful
  train writes.
- **A squad** records the **captain's `o`**: `init_unit` makes `UBER_SIZE`
  (+0x308) figures and returns the last figure's `get_captain()`. If
  `find_free` fails part-way, `init_unit` returns a negative code, and
  nothing is written although the earlier figures exist.
- **Callers**: 27 direct calls and no vtable entry. `Build::activate` makes
  15, `Build::process (0061edf0)` 5, `Build::finished (00628490)` 2, and
  one each come from `Leader::gain_tech (006dcb60)`, `City::assimilate
  (00738e90)`, `Setup::place_unit (005abca0)`, `ConsoleWin::run_cmd` and
  `UnitBalance::next`.
- **`Setup::place_unit` trains only as a fallback.** Each placement try that
  finds a free tile calls `init_unit` directly, which does not write.
  `Build::train` runs only after the try budget is spent (60 with a starting
  town, 500 for a nomad), and only when `param_2 ≥ 0`. In that case
  `param_2` also moves the search centre onto the building. When the budget
  is spent with `param_2 < 0`, it calls `init_unit` directly instead. So a
  normal start unit is never recorded. Neither are gaia units or units a
  trigger makes.

### In the crate

- **Building hook**: `activate_head_steps`
  (`crates/sim/src/cities/activate.rs:53`) tests the predicate at `:74` and
  calls the empty `SimHead::record_last_built` (`:139`; `:349` is the test
  view). It is reached from `Sim::activate`
  (`crates/sim/src/cities/city.rs:2420`, head at `:2454`). The head already
  orders the Chinese branch before the type read (`activate.rs:65–76`).
  - **Index**: `buildings[b].ty`, read after the head's `set_type`
    (`activate.rs:118–131`). The crate's building record is
    `TypeIndex − 0x19e` (`crates/rondata/src/load.rs:220`).
  - **Value**: `buildings[b].index`, the crate's `SubObjectData::o` in
    `[2000, 3000)` (`crates/sim/src/cities/building.rs:14–17`), **not** the
    vector position `b`.
- **Trap, the frame-0 `counted` flag.** `Leader::produce_building` passes
  counted 0. The crate's frame-0 arm calls `self.activate(o, false, true)`
  (`crates/sim/src/ai/ai_place.rs:1035`), which is counted = true. Fill the
  hook as-is and every building `Setup` produces at frame 0 would be
  recorded, where the witness below shows −1. The same flag already gates
  `claim_building_high`'s `free_units_for`
  (`crates/sim/src/cities/nations.rs:238`). So the fix needs its own check
  and a re-check of what that path pays, not a bare flip.
- **Unit hook**: `Sim::build_train` (`crates/sim/src/cities/queue.rs:1693`),
  right after `init_unit` (`crates/sim/src/units/store.rs:504`), with
  `table[who][ty] = units[unit].index as i32`.
  - **Index**: the crate's unit record `ty` already equals
    `TypeIndex − 0x32` (`load.rs:1468`, `:212–216`). `.tree` is a tech-tree
    id, so subtracting 50 from it would be wrong.
  - **Value**: `units[unit].index` is `SubObjectData::o`
    (`crates/sim/src/units/unit.rs:40–47`). `init_unit` returns the squad
    head, and its `index` is what the members take as their captain, which
    matches the original.
  - **Callers that inherit the write**: `queue.rs:1612`, `:1675`,
    `cities/activate.rs:391`, `cities/city_views.rs:591`, and
    `cities/nations.rs:432, 524, 583, 655, 686`.
- **SEAM**: `Sim::place_unit` stops at `Err(PlaceRefused::Train)`
  (`crates/sim/src/units/starting.rs:26–27, 68`). When the fallback is
  built, it must go through `build_train` so the write follows.
- **No failure path**: the crate's `init_unit` returns `usize` and cannot
  fail, so "negative return → no write" has no counterpart until
  object-table exhaustion is modelled.
- **Checksum**: both leaves are `(unheld)` in `LEADERS_HELD_OUT`, count 2
  each (`crates/rondata/src/diff/checksum_lines.rs:1805, 1808`; pile
  `out:finish` in the 4124 journal). Each port removes its entry.

### Reached by a plain game

- **`last_building_finished`**: every construction completed through
  `Wall::do_construct`. These do not record: the start city
  (`Setup::build_cities`, counted 0), frame-0 produced buildings, captures
  and bribes. Off the plain path:
  - `Console::hurry_action` needs `cheat_keys != 0` and semaphore bit 4
    clear (key case 0x45 of `Console::exec_keymap`), or comes from
    `ConsoleWin::run_cmd`;
  - `create_building` is a trigger;
  - `ConquestGame::build_cities` is Conquer the World (and does not write).
- **`last_unit_finished`**: every queued unit (`Build::finished`) and every
  free unit (`Build::activate`, `Build::process`). The Americans' bombers
  (`Leader::gain_tech`) and assimilation's hero (`City::assimilate`) are
  skirmish-reachable too. Start units record only on the placement
  fallback. The console, `UnitBalance` and triggers are off the plain path.
- **The only reader** is a trigger, so in a plain game the leaves matter to
  the checksum and the save alone.

### How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/last-finished-*.md`); no diff backs it
yet.

The verdicts were 30 confirmed, 6 corrected and 0 open (common ground 5/0/0,
buildings 14/3/0, units 11/3/0). A squad's value, which both readings had
left open, was settled as the captain's `o`. The store, predicate, index,
init, dispatch and caller sets are each settled at a listing address.
Confidence is high.

The corrections that changed a rule:
- `Setup::place_unit` records only on its fallback train, not for every
  start unit placed at a building;
- the crate's unit cell is `[ty]`, not `.tree − 50`;
- the stored values are `Building::index` and `Unit::index`, not vector
  positions;
- the crate's frame-0 activate passes counted = true where the original
  passes 0.

The building caller list was corrected to the union of both readings; no
caller in it writes.

**Witness, not a pinned diff**: the 4124 journal's checksum columns
(`docs/journal/2026-11-11-item-4124.md:159–160`). Every cell of both leaves
is −1 at the start, including the citizen's unit cell 0, as the
fallback-only rule predicts. Later, building cells hold `o`s 2007..2015 and
the unit cell 0 holds 21, then 14.

### Not established

- **The frame-0 `counted` flag** (`ai_place.rs:1035`) needs a re-gate once
  `record_last_built` is filled, together with what `claim_building_high`
  pays on that path. A port settles it against the witness (−1 at the start).
- **Whether the crate's `Building::index` matches the original's building
  `o` allocation** (`Objects::find_free (0065ad60)` over `[2000, 3000)`) is
  the crate's existing contract and was not re-read here. A pinned diff of
  `last_building_finished` would settle it.
- **The no-write-on-failure branch of `Build::train`** cannot be exercised
  until the crate models object-table exhaustion.
- **How a port treats the trigger reader's gaia overrun** into
  +0x67f4..+0x6823 is a design choice for when triggers are built (they are
  cut from v1). Nothing in the original is open.
- **Both leaves stay unpinned** until the crate holds them and the
  `out:finish` rows on a kept capture diff clean.
