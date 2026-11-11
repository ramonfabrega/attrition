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

## `victory_type` (+0x7d8)

**1. Writers.** Export: `field_0x7d8` stored only in `Leader::victory
(006ec9b0)`; read in `EndGameWin::exec (0083bcc0)` and
`Game::determine_end_of_game_stats_and_achievements (00583b30)`
(`EndGameWin::get_type_string (0083a420)` reads `EndGameWin`'s own copy).
Listing: `0x7d8(%reg)` stores in `Constants::init`, `Constants::log_data`
(negative `ebp` offset), `GraphicPieces::close`, `ParamRegisterWindow::
registration` (other types) and `6ec9ec` in `Leader::victory` (`esi` =
`this`, `eax` = first argument). Global form `0xe3ab68`: one read
(`58403c`, `determine_end_of_game_stats_and_achievements`), no store.
Listing callers of `0x6ec9b0`: `Game::check_victory`, `GameDaemon::
process_victory` (6 sites), `Leader::gain_tech` (2), `Leader::set_diplo`,
`Leader::victory` (its own cascade), `ConsoleWin::run_cmd`,
`ScenarioFuncSet::victory`, `TurnControl::toggle_pause`.

**2. Event and predicate.** `Leader::victory(type, instant)`:
- does nothing if `leader_flags & 0x60` (already won, 0x20, or already
  defeated, 0x40) (`6ec9be`); **so the leaf is write-once** and a defeated
  leader never gets one;
- sets `leader_flags |= 0x20`, `leader_flags2` bit 0x100 = `instant != 0`,
  `victory_type = type` (`6ec9ec`); achievement event; cleans the queues
  of its buildings from object 2000 up; `Player::win` for each seated
  player of this `who`; `scene.recalc_borders = 1`;
- **cascade**: for every other slot with `leader_flags & 3 == 3` (seated,
  bit 1 still set), sets that slot's `0x80`, then calls `victory(type,
  instant)` on it if `LeaderData::is_ally(that, who)` (one-way, the other
  slot's view), else `defeat(5, −1, 0)`. Allies thus get the same code,
  and an ally already won keeps its first.
- No `semaphore[1] & 2` (balance-test) guard here, unlike `defeat`.
The type codes are in "Common ground". The VICTORY options that reach
each: wonder arm under VICTORY 0/1/6, territory 0/1/7, score 3, time 4,
economic 8, ending tech 9; conquest (`check_victory`, `set_diplo`) under
any.

**3. Reset.** Only `Leader::init`'s memset branch (tribe < 0), i.e.
`Leaders::init` at game setup. The tribe ≥ 0 path (`Setup::build_leader
(005abc70)`, `Setup::build_game (005ac190)`, scenario loads) does not
touch +0x7d8 (its `stosl` run from +0x7c8 stops at +0x7d4, `6e3bee..
6e3bf8`).

**4. Readers.** End-of-game UI and statistics/achievements only
(`EndGameWin::exec`, `determine_end_of_game_stats_and_achievements`), plus
`log_data`/`walk_data`. No sim decision reads the value; the sim reads the
flag bit 0x20 instead.

**5. Crate.** No victory event at all. `crates/sim/src/territory/
victory.rs` is the territory clock only; its `seams` table names the
expiry (`Leader::victory(2)`) as not built ("the game never ends by
territory"). There is no `check_victory`, no wonder arm, no cascade.
`checksum/leaders.rs:144` lists the leaf, unfilled. A builder would add a
`Sim::victory(who, type, instant)` (beside `Sim::defeat` in
`cities/city.rs`, or in `territory/victory.rs`) and call it from the
expiry seam and from a `check_victory` after `Sim::defeat`.

**6. Plain game?** Yes: a skirmish against the computer ends by conquest
(`check_victory` from the last enemy's `defeat`, code 0) or, under the
default VICTORY 0, by the wonder or territory timer (codes 1, 2).
`toggle_pause`, console and scenario callers are outside it.

**7. Confidence.** High on the guard, the store and the cascade's shape
(listing `6ec9b0..6ec9ec`; cascade from the export). Not read in the
listing: the cascade's argument order (export only). `check_victory`'s
gating (the armageddon comparison before its `victory(0,0)`, and the
"no living human" exit) was read from the export only and is muddled
there; whether a skirmish's conquest always reaches `victory(0, 0)` is not
established.

## `defeat_type` (+0x7dc)

**1. Writers.** Export: `field_0x7dc` stored only in `Leader::defeat
(006ecb00)`; read in `EndGameWin::exec`, `StatWin::draw_normal_stats
(007ebd20)`, `StatWin::on_redraw (007f0e70)`, `GameReportWin::
on_button_clicked (007b4f50)`, `determine_end_of_game_stats_and_
achievements`, `AchieveChartPane::on_redraw (007f6ff0)`, `::draw_game_info
(007f3ce0)`, `::on_spot_move (007f7e40)`, `IFaceDiplo::draw_player_area
(007ff420)`. Listing: `0x7dc(%reg)` stores in `IFaceRenderManager::
clear_all`, `Constants::init`, `Constants::log_data`, `GraphicPieces::
close` (other types) and `6ecb7b` in `Leader::defeat` (`ebx` = `this`,
`esi` = first argument). Global form `0xe3ab6c`: a `mov $imm` in
`determine_end_of_game_stats_and_achievements` (`584425`) and a read in
`ConquestGame::process_results (00798940)` (`799194`, compares slot 0's
code with 10 — CtW). Callers of `0x6ecb00` (listing): `Unit::close`, `Leader::
lost_a_city` (2), `Leader::process_elimination` (2), `Leader::gain_tech`,
`Leader::victory`, `Game::defeat_all`, `GameDaemon::process_victory` (2),
`Player::leave_game` (3), `CommandPackage::end_process`, `ConsoleWin::
run_cmd`, `ScenarioFuncSet::defeat`, `::defeat_attacker_and_allies` (2).

**2. Event and predicate.** `Leader::defeat(type, by, instant)`:
1. `defeat_by(by, type)` (always; see the two leaves above);
2. returns if `leader_flags & 0x40` (already defeated) (`6ecb24`) or
   `semaphore[1] & 2` (`6ecb32`, the unit-balance test mode);
3. `leader_flags2` bit 0x200 = `instant != 0`; `leader_flags = (flags &
   ~2) | 0x40`; `Armies::leader_defeated(who)`; `leader_flags2 |= 2`;
   `game.musical_chairs = game.frame`; **`defeat_type = type`**
   (`6ecb7b`);
4. then the achievement event (none for type 5), building queues cleaned,
   units' orders cleared (planes die), `Player::lose`, messages,
   `Game::check_victory` unless `semaphore[0] & 0x40`, and in multiplayer
   (`semaphore & 0x44 == 4`) the loser's treasury split among its seated
   allies.
So the leaf is **write-once per defeat**, written only on the first
`defeat` that passes the 0x40 guard. A leader that has *won* (0x20) is not
protected: `defeat` does not test 0x20, and the victory cascade's
`flags & 3 == 3` admits a prior winner, so a winner can later receive a
`defeat_type` while keeping its `victory_type`.
Per-caller predicates for a plain game:
- `Leader::lost_a_city(is_capital, p2, captor)` (args read off the
  listing, `0x14/0x18/0x1c(%esp)`): returns under `semaphore[1] & 2`, or
  `semaphore[1] & 8` with `turn_control` flag 1. Under elimination 1, a
  lost capital with `captor >= 0`, and the loser lacking wonder 0x213
  (`has_wonder`), defeats with type 1 and `instant = 1` if the captor or
  any ally of the captor (seated, alive) has `GLOBAL_GOVERNMENT_BONUS`
  (TypeIndex 697). Then by elimination mode: 2 → type 2 for a lost
  capital, unless the loser has tribe bonus 0x17 and still holds a capital
  of its own; 3 → type 3 when `p2 != 0`; otherwise (0, 1, others) →
  type 0 when `leader_flags2 & 1` is clear and `+0x3f8` (city count) is 0.
- `Leader::process_elimination`: elimination 1 and the capital timer
  (`+0x418`) running, when `Game::retake_capital() − (frame − +0x414) <= 0`
  → type 1, `by` = the capital tile's holder.
- `Unit::close`: the owner (< 8) has `num_units` (+0x93c) 0 and city count
  0, `semaphore[1] & 8` clear and `leader_flags2 & 1` clear → type 0,
  `by = −1`.
- `Leader::victory`'s cascade → type 5; `Game::defeat_all` (armageddon,
  `flags & 0x42 == 2`) → 8; `process_victory`'s musical chairs (VICTORY 5,
  every `chairs·900` frames, lowest `score`, no tie) → 4.
- `Player::leave_game(t)`: if the leader is seated and alive and its
  capital timer is running → type 1 (by capital holder under elimination
  1, else self); else type `t` (6 from `resign`, 7 from `drop`), `by = −1`.

**3. Reset.** Only `Leader::init`'s memset branch, as for
`victory_type`.

**4. Readers.** UI, statistics and CtW (`ConquestGame::process_results`
tests 10, a code no non-CtW caller passes — `ConquestLeader::defeat` is
the CtW path). No skirmish sim decision; the sim reads bit 0x40.

**5. Crate.** `Sim::defeat` (`crates/sim/src/cities/city.rs:623`) has no
type argument; its one caller is `lost_a_city` (`cities/city.rs:613`,
default elimination only, i.e. type 0, and it ignores `_was_capital`).
`Sim::resign` (`crates/sim/src/leaders/resign.rs`) is type 6. No
`process_elimination`, no `Unit::close` last-unit defeat (grep finds no
`num_units == 0` defeat), no cascade, no armageddon. A builder adds a
`defeat_type` array and a `type` parameter to `Sim::defeat`, writes it
after the `defeated[who]` guard (which matches the 0x40 guard), and
passes 6 from `resign`. `checksum/leaders.rs:145` lists the leaf,
unfilled.

**6. Plain game?** Types 0, 5, 6 are reached by a skirmish against the
computer with default options (last city lost; an enemy's win; a human's
resign); type 1/2/3 need a non-default elimination option or the
`GLOBAL_GOVERNMENT_BONUS` tech; 4 and 8 need VICTORY 5 or armageddon
(armageddon is reachable in a plain game by nuclear use); 7 is
multiplayer; 9 is the trigger layer.

**7. Confidence.** High on the store, both guards and their order
(listing `6ecb00..6ecb7b`), and on `lost_a_city`'s arguments and mode
switch (listing `6dc6d0..6dc85e`). From the export only: the per-caller
predicates of `Unit::close`, `process_elimination`, `leave_game`,
`defeat_all` and the musical-chairs arm. Not followed: what wonder 0x213
and tribe bonus 0x17 are by name; `City::close`/`Build::close`'s
computation of `is_capital`, `p2` and `captor`.

## `wonderwin_timer` (+0x44c)

**1. Writers.** Export: `field_0x44c` stored in `Leader::init
(006e3930)` and `GameDaemon::process_victory (00730ef0)`; the other
export hits (`Wall::process`, `Group::walk_data`, `Group::add`,
`Army::find_target`, `Game::is_victory_timer`, `GameReportWin::close`,
`ConquestInvadeWin::close`, `Leaders::process_all`, `Leader::process`) are
reads or other types. Listing `0x44c(%reg)` stores: `process_victory`
`731070` (`mov $1`) and `731357` (`mov $0`), `ebx` = the walked
`Leader*` (`0xe3a390 + i·0x6eec`, `730f60`/`7313a2`); `Leader::init`
`6e4166` (`eax` = 0). The rest are other types: `Constants::init`/
`log_data`, `Game::init_teams` (byte, `GameInfo.player[]`, stride 0x8c),
`ConquestLeader` ctor/dtor (a vtable pointer), `RTreeData::clear`,
`DropControl::process_drop` (bytes), `ScenarioRead::load_leaders_chunk
(009acde0)` (`9ad0ed`; its neighbours store a byte at `+0x3c4`/`+0x450`
and a word at `+0x414`, so the base is not a `LeaderData`). Global form
`0xe3a7dc`: reads only (`Leader::diplomacy (006bc950)` `6be27f`,
`Army::find_target (006f69b0)` `6f6c38` and `6f72e0`) and a `mov $imm` in
`Game::is_victory_timer (00594950)`.

**2. Event and predicate.** It is a **0/1 flag, "the wonder clock runs"**,
not a count. `GameDaemon::process_victory`, every sim frame (called from
`GameDaemon::process_all`), wonder arm:
- Gate: `VICTORY` ∈ {0, 1, 6} (`730f2d..730f3a`). `W = Game::wonder_winning
  (005948a0)` — the seated, undefeated slot whose `get_wonder_net` reaches
  the `wonderwins` row's threshold and leads (ties: allied → higher
  `get_wonder_value`, unallied → none), or −1.
- Walk slots `i` = 0..7 with `leader_flags & 3 == 3`, in slot order:
  - **Off side** — `W < 0`, or `i ≠ W` and `who_i ≠ W` and not mutually
    allied with `W` (`diplo_i[W] == 2` and `diplo_W[who_i] == 2`): if the
    flag is set, **flag = 0** (`731357`), the stamp is debited (see
    `wonderwin_stamp`), event `ACHIEVEEVENT_TIMER_STOP`, and the first stop
    of the frame plays `S_WONDER_VICTORY_STOP`.
  - **On side** — compute `gg` = some alive (`flags & 1`) slot `j` that is
    `i`, or has `who_j == i`, or is mutually allied with `i`, holds
    `has_preq(GLOBAL_GOVERNMENT_BONUS)` (TypeIndex 697, `push $0x2b9`,
    `730ffa`). Then:
    - if `VICTORY == 0` and (`semaphore[0] & 4` or `num_nations > 1`) and
      `!gg`: if the flag is clear, **flag = 1** (`731070`) and the stamp is
      credited, event `ACHIEVEEVENT_TIMER_START`, and — only for the first
      on-side slot of the frame — the start message and
      `S_WONDER_VICTORY_START`. If the flag is already set and this is the
      first on-side slot of the frame: `remaining = Game::wonder_timer() −
      (frame − stamp)`; **`remaining == 0` exactly** (the `sub` sets ZF at
      `7311dd`, `jne` at `7311e4`) → `Leader::victory(i, 1, 0)`; at
      `remaining` ∈ {4500, 1800, 900} frames (`0x1194`, `0x708`, `0x384`)
      and the console player not an ally of `i`, a warning.
    - otherwise (VICTORY 1 or 6, a one-nation solo game, or `gg`): the
      first on-side slot of the frame wins at once, `victory(i, 1, gg)`.
    - the "first on-side slot" latch (`-0x1c(%ebp)`) is then set.
  - **Unlike the territory arm the walk does not stop**: every on-side
    slot gets its flag set, but only the first on-side slot (slot order)
    has its own clock tested for expiry.
- `Game::wonder_timer (005944f0)`: 0 when `constants.wonder_timer < 1`,
  else `max(1, (world[0]·WONDER_TIMER + base/2) / base)` with `base` a
  `map_sizes` row's `+0x144`: the rules' frames scaled by map size,
  rounded half up. (Not followed: which `map_sizes` row, and what
  `world[0]` is.)

**3. Reset.** `Leader::init` → 0 in both branches (`6e4166` and the
memset). No per-frame fold besides the arm itself.

**4. Readers (decisions).** `Army::find_target` — an enemy's city scores
×10 while that enemy's flag is set, it has wonders, and it is `W`
(`crates/sim/src/ai/army/target_score.rs:148`); and the army's
`about_to_win` (`local_34`) when a leader's flag or `popwin_timer` is set
(`find_target` export line ~911). `Leader::diplomacy (006bc950)` tests
the flag (with `popwin_timer`) of another leader before a diplomacy
branch (`6be27f`). The function the export files as `Wall::process
(00640450)` skips a seated unallied leader's wonder count when its flag is
set (crate note: `crates/sim/src/cities/site_recruit.rs:157`; the name may
be a `-GL` fold and was not checked against the map).
`Game::is_victory_timer` (flag of any seated slot under VICTORY 0, or a
`popwin_timer`) is read by `Leader::diplomacy`, `Options::describe` and
the diplomacy windows.

**5. Crate.** Not carried. `crates/sim/src/territory/victory.rs` holds
the POPWIN arm only (`process_victory_territory`, called from
`core/frame.rs:222`); the wonder arm, which in the original runs
**before** the POPWIN arm inside the same `process_victory`, is absent.
Readers stub it: `ai/army/target_gate.rs:428` `w44c` returns 0;
`cities/site_recruit.rs:157` SEAM; `ai/army.rs:38` lists it among fields
not kept. A builder adds `wonderwin_timer`/`wonderwin_stamp` beside
`popwin_timer`/`popwin_stamp` (`self.ai.ai[i]`), a
`process_victory_wonder` ahead of `process_victory_territory` in
`core/frame.rs`, reusing `Sim::wonder_winning` (`ai/ai_build.rs:1439`).
`checksum/leaders.rs:114` lists the leaf, unfilled.

**6. Plain game?** Yes: the default `VICTORY` 0 in a skirmish of two or
more nations runs the timer branch; the immediate branch needs VICTORY 1/6,
a solo game, or `GLOBAL_GOVERNMENT_BONUS`.

**7. Confidence.** High on both stores, the gate, the side test, the
`gg` loop's indices, the exact-zero expiry and the no-break walk
(listing `730ef0..7313b9`). The message/sound details are from the export.
Not read: `Game::wonder_winning`'s tie rule in the listing (export only;
the crate already ports it under a sweep), and the readers' full
predicates.

## `wonderwin_stamp` (+0x448)

**1. Writers.** Export: `field_0x448` stored in `Leader::init`,
`Leader::process (006b88b0)` and `Leaders::process_all (006ed2a0)` (the
same refresh; `Leader::process` has no caller in the export or the listing
— its body is inlined into `process_all`), and `GameDaemon::
process_victory`. Listing `0x448(%reg)` stores: `process_victory`
`731080` (`add %eax` = `game.frame`) and `731367` (`sub %eax` =
`game.frame`); `Leaders::process_all` `6ed3c8` (`inc`); `Leader::process`
`6b89c8` (the uncalled copy); `Leader::init` `6e4160` (0). Other types:
`GraphicWinXML::close` (byte), `Constants::init`/`log_data`, `Leader::
end_process (006b8690)` and `Leaders::end_process_all (006ed070)` (word
stores into `GameInfo.player[]` flags, base `+0x3bc` + 0x8c, not the
leader), `ScenarioRead::load_leaders_chunk` (not a `LeaderData`, as
above). Global form `0xe3a7d8`: nothing.

**2. Event and predicate.** A banked clock in frames:
- **start** (flag 0 → 1, `731080`): `stamp += frame`;
- **stop** (flag 1 → 0, `731367`): `stamp −= frame`;
- while running, elapsed = `frame − stamp`; while stopped, `−stamp` is the
  elapsed time banked (≤ 0 stamp);
- **pay-back** in `Leaders::process_all` (from `Game::do_frame (00591ef0)`),
  for each slot with `leader_flags & 2` (alive), in slot order: when
  `constants.timer_refresh_ratio` (`Constants+0xd00`, read at `6ed36a`)
  is non-zero and `frame % ratio == 0`, and `stamp < 0` and the flag is
  0, `stamp += 1` (`6ed3b4..6ed3c8`). So banked progress is lost at one
  frame per `TIMER_REFRESH_RATIO` frames (5 in the crate's tuning,
  `core/tuning.rs:1923`) while the clock is stopped, and never goes
  positive by this path. The same block does the same for `+0x414/+0x418`
  and `popwin_stamp/popwin_timer`.
- Expiry reads it (`remaining` above).
No slot is exempt beyond the arm's `flags & 3 == 3` (start/stop) and
`flags & 2` (pay-back).

**3. Reset.** `Leader::init` → 0 (`6e4160`, and the memset).

**4. Readers.** `process_victory`'s expiry (decision: the wonder win);
`IFaceMainBase::draw_timers (0080b060)` (the on-screen countdown);
`log_data`, `walk_data`.

**5. Crate.** Not carried; same site as `wonderwin_timer`. The pay-back
already exists for the POPWIN pair in `Sim::refresh_victory_timers`
(`territory/victory.rs:88`, from `core/frame.rs:194`); the builder adds
the wonder pair to that function. Note it checks `timer_refresh_ratio`
and `frame % r` as the original does; whether it is gated on `flags & 2`
like `Leaders::process_all` depends on its caller's loop
(`core/frame.rs:~185`), not read. `checksum/leaders.rs:113` lists the
leaf, unfilled.

**6. Plain game?** Yes, with `wonderwin_timer`.

**7. Confidence.** High (listing for all four stores and the pay-back
predicate). Not read: `IFaceMainBase::draw_timers`' arithmetic.

