# Item 4152, family 10 `end-of-game` — first reading

- Family: 10, `end-of-game`.
- Leaves (`LeaderData` offsets): `defeated_by` (+0x10), `wonderwin_stamp`
  (+0x448), `wonderwin_timer` (+0x44c), `victory_type` (+0x7d8),
  `defeat_type` (+0x7dc), `defeat_stamp` (+0x6924); all int.
- Reader: Claude Opus 5.5 (`claude-opus-5-5`), first reading, wave 1,
  2026-11-11.
- Sources: the decompile export (`~/ghidra-projects/decomp`), the listing
  (`llvm-objdump` of `riseofnations.exe`), `crates/sim/src`.

## Common ground

- **`Leader*` is `LeaderData*`, offsets unshifted.** `types.txt` gives
  `Leader` (0x6eec) as `LeaderData` (0x6ee4) at +0 with 8 trailing bytes
  (`+0x6ea4..` are `LeaderData`'s own); `Leaders` is `Leader[10]` from +0.
  Every `Leader::` method below reaches `who` as `this+0x8` and loops
  `0xe3a390..0xe71af0` in steps of 0x6eec, so the leaf offsets are the same
  in `Leader` and `LeaderData` code, and the global form is
  `0xe3a390 + i·0x6eec + off`.
- **Global-form net** (`0xe3a3a0`, `0xe3a7d8`, `0xe3a7dc`, `0xe3ab68`,
  `0xe3ab6c`, `0xe40cb4` over the full listing): **no store at all**; only
  reads (listed per leaf) and one `lea 0xe3a3a0` in
  `RecordGame::write_random_game_info (009534e0)`, which is the *end*
  pointer of a `[tribe, tribe+4)` range handed to `SaveGame`'s writer — not
  a write of `defeated_by`.
- **Bulk writers of all six** (not game events):
  - `Leader::init (006e3930)` with `param_2` (tribe) < 0 opens with
    `memset(this, 0, 0x6929)` (listing `6e398c..6e3995`), which zeroes every
    leaf of this family. `Leaders::init (006ed850)` calls it that way for
    each slot (`push 0; push -1; push who`, `6ed8c5..6ed8cb`), from
    `Game::init_rules_and_teams (00589bb0)`.
  - `LeaderData::walk_data (006d6750)` walks `[who, +0x692a)` for a seated
    slot; under an input walker (`LoadGame::verify_load (005a39f0)`) that
    restores all six from a save. Under the checksum walker it only reads.
- **The victory/defeat type codes**, from the call sites (a code is the
  first argument; values are the original's own):
  - `victory_type`: 0 conquest (`Game::check_victory (005926b0)`,
    `Leader::set_diplo (006ec6a0)`, console, `ScenarioFuncSet::victory
    (009f3fa0)`, `TurnControl::toggle_pause (00957ab0)`); 1 wonders and 2
    territory (`GameDaemon::process_victory (00730ef0)`); 3 the ending
    technology (`Leader::gain_tech (006dcb60)`, VICTORY 9); 4 score goal
    (VICTORY 3); 5 economic (VICTORY 8); 6 time limit (VICTORY 4). Allies
    of the winner inherit the winner's code (below).
  - `defeat_type`: 0 no cities (`Leader::lost_a_city (006dc6d0)`, default
    elimination) or no units and no cities (`Unit::close (0060ee50)`) or
    console/scenario; 1 capital elimination (`Leader::process_elimination
    (006b8a20)`, `lost_a_city`'s `GLOBAL_GOVERNMENT_BONUS` arm,
    `gain_tech`, `Player::leave_game (006ee010)` with a capital timer
    running); 2 and 3 elimination modes 2 and 3 (`lost_a_city`); 4 musical
    chairs (VICTORY 5, `process_victory`); 5 an enemy's victory
    (`Leader::victory (006ec9b0)`'s cascade); 6 resign and 7 drop/disconnect
    (`Player::leave_game` from `Player::resign (006edcb0)` /
    `Player::drop (006ede80)`; 7 also `CommandPackage::end_process
    (0094c800)`); 8 armageddon (`Game::defeat_all (00592aa0)`); 9 scenario
    (`ScenarioFuncSet::defeat (009f3f50)`).

## `defeated_by` (+0x10)

**1. Writers.**
- Export: `->defeated_by` appears only in readers
  (`LeaderData::log_data (006e5110)`, `Options::describe (00724460)`);
  `field_0x10 =` in a `Leader`/`LeaderData` context only in
  `Leader::defeat_by (006d1c80)` and `Leader::init (006e3930)` (the other
  `field_0x10` hits in `LeaderData::LeaderData`/`~LeaderData` are
  `sites.field_0x10` and `make_list.field_0x10`, sub-objects).
- Listing: below +0x200, so register stores were scanned in every function
  that also names `0x6eec`, `0xe3a390` or `0xe71af0` (169 functions). The
  `Leader`/`LeaderData`/`Game`/`Player` hits were each read at the store:
  only `6d1da6` (`defeat_by`, `mov %esi,0x10(%edi)`, `edi` = `this`) and
  `6e3b6b` (`init`, `mov %edi,0x10(%ebx)`, `edi` = −1 from `or $-1,%edi` at
  `6e3aca`) are this leaf. The others are `data_encrypted` (`+0x6eb8`)
  records (`Leader::production_ai_setup`, `Game::action_cheat_increase_
  buckets`), `String`s (`say_pay`, `say_full_name`), site records
  (`compute_sites`), the `Game` object (`Game::run_editor`) and local
  arrays (`calc_rare`, `calc_city_resources`).
- Callers of `defeat_by`: only `Leader::defeat (006ecb00)` (listing, one
  `call 0x6d1c80`), which passes its own `(by, type)` through.

**2. Event and predicate.** `Leader::defeat(type, by, instant)` calls
`defeat_by(by, type)` *first*, before its own already-defeated guard.
`defeat_by`:
- returns at once if `defeated_by >= 0` (`6d1c8e`): **the leaf is
  write-once per game** once a defeater has been recorded;
- (for `type != 5`, the blow-up arm — see `defeat_stamp`);
- stamps `defeat_stamp`;
- returns if `by < 0` (`6d1da0`), leaving `defeated_by` at −1;
- else `defeated_by = by` (`6d1da6`), whatever `elimination` is; then, if
  `elimination != 3`, the captor arm (cities to `by` or its allied founder,
  buildings handed over or killed).
So the leaf is the slot index of the first leader passed as `by` to a
`defeat` of this leader. Which callers pass `by >= 0`: `lost_a_city` (its
`param_3`, the city's captor, which may itself be −1), the
`GLOBAL_GOVERNMENT_BONUS` arms (`lost_a_city`, `gain_tech` → the bonus
holder), `process_elimination` (the holder of the capital's tile, or
self under elimination ≠ 1 — note that arm is entered only under
elimination 1, so the "self" branch is dead unless the option changes
mid-call), and `leave_game` with a capital timer running (capital holder
under elimination 1, else **self**). Every other caller passes −1:
`Unit::close`, the victory cascade (5), musical chairs (4), `defeat_all`
(8), resign/drop without a capital timer (6/7), `CommandPackage` (7),
scenario and console. A `by` equal to the loser's own slot is possible
(`leave_game`) and is stored as such; the captor arm then skips the
`capture_city` for `who == by`.

**3. Reset.** `Leader::init` common path → −1 (`6e3b6b`), after the
memset branch's 0. Nothing else; no per-frame fold.

**4. Readers.** `LeaderData::log_data` (the gamelog dump), `Options::
describe` (UI text), `LeaderData::walk_data` (checksum/save). No sim
decision read was found in the export: no `field_0x10` read in a
`Leader` context outside `defeat_by`'s own guard. The guard itself is the
one decision it changes — a second `defeat` with a `by` never re-runs the
captor arm.

**5. Crate.** `crates/sim/src/cities/city.rs:623` `Sim::defeat(who, by:
Option<Player>)` carries `by` but stores no `defeated_by`; its guard is
`leaders.defeated[who]`, not `defeated_by >= 0`. `crates/sim/src/leaders/
resign.rs` `Sim::resign` is `defeat(6, −1, 0)` and also stores nothing.
`checksum/leaders.rs:39` lists the leaf but no arm fills it (falls through
`_ => None`). A builder adds `defeated_by: [i32; N]` (init −1) to the
leaders state and writes it in `Sim::defeat` when `by.is_some()` and the
slot is still −1; `resign` writes nothing. Note the crate's `resign` does
not model `leave_game`'s capital-timer branch (defeat type 1 with `by` =
capital holder / self).

**6. Plain game?** Yes: a skirmish against the computer reaches
`lost_a_city` (a captured or razed last city) and, under capital
elimination, `process_elimination`. `leave_game`'s branch is reached by a
human resign. The `GLOBAL_GOVERNMENT_BONUS` arms need that bonus's tech.

**7. Confidence.** High on the store, the guard and the `by < 0` skip
(listing). Medium on "no other writer": the register-relative net below
+0x200 was filtered to functions that name the leaders array, so a writer
reaching a `Leader*` through a pointer it was handed (no `0x6eec` in the
body) would be missed; the export net covers that case only as far as the
decompiler typed the base. `lost_a_city`'s callers (`City::close
(00737550)`, `Build::close (00628980)`) were not read for how they
compute the captor.

## `defeat_stamp` (+0x6924)

**1. Writers.** Export: `field_0x6924` stored only in `Leader::defeat_by
(006d1c80)` and `Leader::init (006e3930)`; read in
`AchieveChartPane::set_chart_data (007f2c40)`. Listing: `0x6924(%reg)`
stores at `6d1d98` (`defeat_by`) and `6e3e0f` (`init`, `eax` = 0); global
form `0xe40cb4`: none.

**2. Event and predicate.** In `defeat_by`, under `defeated_by < 0` only,
`defeat_stamp = game.frame` (`6d1d92..6d1d98`, `Game+0x550`), **before**
the `by < 0` test. So:
- the first `defeat` of a leader stamps the frame, in frames;
- **every later `defeat` call while `defeated_by` is still −1 re-stamps
  it**, because `defeat` calls `defeat_by` before its own
  `leader_flags & 0x40` (already defeated) return. A leader defeated with
  `by = −1` (resign, `Unit::close`, cascade, armageddon) and later handed
  to `defeat` again (e.g. `Unit::close` when another of its units closes
  with `num_units == 0` and no city; `lost_a_city` for a city still on its
  books) moves its stamp forward. Once a `by >= 0` has landed the stamp is
  frozen.
- The `type != 5` blow-up arm runs before the stamp: under
  `semaphore[0] & 4` (multiplayer), when among the other seated leaders
  none has already won (`& 0x20` stops the walk) and two undefeated ones
  are not allied, and (some other seated human is alive or no slot has a
  human), it blows up towers/units (or buildings and units
  under VICTORY 5). It does not touch either leaf.

**3. Reset.** `Leader::init` → 0 in both branches (memset, then
`6e3e0f`).

**4. Readers.** `AchieveChartPane::set_chart_data` (post-game chart: a
zero stamp is read as "still alive", replaced by the current frame),
`log_data`, `walk_data`. No sim decision.

**5. Crate.** No such field. `Sim::defeat` (`cities/city.rs:623`) and
`Sim::resign` (`leaders/resign.rs`) are the sites; the builder needs a
`defeated_by` alongside to reproduce the re-stamp rule, and must route the
stamp **ahead of** `Sim::defeat`'s `defeated[who]` early return (the crate
returns before anything for an already-defeated leader; the original
re-stamps first). `checksum/leaders.rs:310` lists the leaf, unfilled.

**6. Plain game?** Yes, the same paths as `defeated_by`.

**7. Confidence.** High (listing for both stores and the order against
`defeat`'s guard). Not read: whether a plain game ever calls `defeat`
twice on a leader with `defeated_by == −1` (needs `Unit::close`'s and
`City::close`'s sequencing after a defeat), so whether the re-stamp is
ever observed in a skirmish is not established.

