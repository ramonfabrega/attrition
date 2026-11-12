# end-of-game: second (blind) reading

- Slug: `end-of-game`
- Leaves (`LeaderData`, size `0x6ee4`): `defeated_by` (+0x10, int),
  `wonderwin_stamp` (+0x448, int), `wonderwin_timer` (+0x44c, int),
  `victory_type` (+0x7d8, int), `defeat_type` (+0x7dc, int),
  `defeat_stamp` (+0x6924, int).
- Reader: Claude Opus 5.5, blind (has not opened the first reading).
- Sources: decompile export `~/ghidra-projects/decomp`, the full `.text`
  listing (llvm-objdump), `types.txt`.

## Layout note (applies to every leaf)

`Leader` (size `0x6eec`) carries `LeaderData` (size `0x6ee4`) at offset 0:
`this->field_0x6eb8` in `Leader` methods is `LeaderData::data_encrypted`
(138 uses, the encrypted accessors), and `this->field_0x8` is `who`. So a
`Leader*` displacement equals the `LeaderData` offset; the 8 trailing bytes
are `Leader`'s own. The global array is `leaders` at `0xe3a390`, stride
`0x6eec`, ten slots (the loops stop at `0xe71af0`); slot `i`'s field `off`
is `0xe3a390 + off + i*0x6eec`.

**One writer covers every leaf**: `LeaderData::walk_data (006d6750)` walks
the byte block `[who, +0x692a)` for every seated leader (`leader_flags & 1`)
— all six leaves sit inside it. Its callers are `LoadGame::verify_load
(005a39f0)` (a load writes the leaves back), `SaveGame::verify_save
(005a76b0)`, `Leaders::walk_data (006e38e0)` and the checksum walkers
`CheckSums::check_all (00936560)`, `CheckSums::check_one_leader (00937020)`,
`CheckSums::check_leaders (009375a0)`. A second bulk writer:
`Leader::init (006e3930)` with `tribe < 0` (`param_2`) calls
`memset(this, 0, 0x6929)` (listing `6e398c..6e3995`), zeroing every leaf;
then, on every `init`, it stores the per-leaf values given below. Neither is
repeated per leaf.

## Leaf 1: `defeated_by` (+0x10, int)

### Writers

Export, `->defeated_by`: only `LeaderData::log_data (006e5110)` (a read);
the two hits in `Options::describe (00724460)` are a mistyped local, not a
leader. Export, `field_0x10` restricted to `Leader`/`Leaders`/`LeaderData`
`this`: `Leader::defeat_by (006d1c80)` lines 28 (read) and 80 (write), and
`Leader::init (006e3930)` line 151 (write). No `leaders.list[i].field_0x10`
anywhere. Listing: the only absolute reference, `0xe3a3a0` at `95359d` in
`RecordGame::write_random_game_info (009534e0)`, is the *end* pointer of a
byte-range write of `tribe` (`[0xe3a39c, 0xe3a3a0)`), not an access of this
field. A register-relative `0x10(%reg)` cannot be told apart in the listing,
so the export's typed net is the one that counts here.

| writer | value | predicate |
|---|---|---|
| `Leader::init (006e3930)` (listing: `this->field_0x10 = -1` after `reset_score`) | `-1` | every init |
| `Leader::defeat_by (006d1c80)` store at `6d1da6` (`movl %esi, 0x10(%edi)`) | `by` (its first argument) | entry guard `defeated_by < 0` (`6d1c8e: cmpl $0, 0x10(%edi); jge` → return), then `by >= 0` (`6d1d9e: js` skips) |

`defeat_by` has one caller, `Leader::defeat (006ecb00)` at `6ecb1d`, which
passes `(by, type)` from its own `(type, by, flag)`. So `defeated_by` is set
**once** per leader per game: the first `Leader::defeat` that carries a
non-negative `by` while the field is still `-1`. A later call never
overwrites it (the guard returns). A defeat with `by = -1` leaves it `-1`, so
a leader can be defeated and keep `defeated_by == -1`.

### The event, by caller of `Leader::defeat` (listing pushes; `type, by, flag`)

| caller | type | by |
|---|---|---|
| `Leader::lost_a_city (006dc6d0)` `6dc7bf` | 1 | captor (`param_3`) — elimination mode 1, the lost city's flag `param_1 != 0`, captor `>= 0`, loser lacks wonder `0x213`, and the captor or an ally of it holds tech preq `0x2b9` (`GLOBAL_GOVERNMENT_BONUS`) |
| `Leader::lost_a_city` `6dc81b` (shared tail) | 0, 2 or 3 | captor: 3 under elimination 3 when `param_2 != 0`; 2 under elimination 2 when `param_1 != 0` unless the loser has tribe bonus `0x17` and still holds its own capital; 0 otherwise when `leader_flags2 & 1 == 0` and `city_num (+0x3f8) == 0` |
| `Leader::process_elimination (006b8a20)` `6b8a9d` | 1 | the holder of the loser's capital (`LeaderData::find_capital`'s second out), when elimination is 1, `lost_capital_timer (+0x418) != 0` and `Game::retake_capital() <= frame − lost_capital_stamp`; the `6b8a75` arm (by = own `who`) needs elimination `!= 1` after testing `== 1`, so it is dead unless `gamec` differs from `game` |
| `Leader::gain_tech (006dcb60)` `6dd599` | 1 | the gainer, for every other leader with no capital whose capital is held by the gainer or an ally, when the tech gained is bonus `0xae4/4`'s preq |
| `Player::leave_game (006ee010)` `6ee0ef` / `6ee0c6` | 1 | capital holder (elimination 1) or **the leaver's own `who`** (other modes), when `lost_capital_timer != 0` |
| `Player::leave_game` `6ee104` | 6 (from `Player::resign (006edcb0)`) or 7 (from `Player::drop (006ede80)`) | -1 |
| `Unit::close (0060ee50)` `60f474` | 0 | -1 |
| `Game::defeat_all (00592aa0)` `592ad2` | 8 | -1 |
| `Leader::victory (006ec9b0)` `6ecadd` | 5 | -1 |
| `GameDaemon::process_victory (00730ef0)` `731e80`, `731f02` | 4 | -1 |
| `CommandPackage::end_process (0094c800)` `94cf4e` | 7 | -1 |
| `ConsoleWin::run_cmd (007d6a70)` `7dd55a` | 0 | -1 |
| `ScenarioFuncSet::defeat_attacker_and_allies (009e5f60)` | 0 | -1 |
| `ScenarioFuncSet::defeat (009f3f50)` | 9 | -1 |

So the only callers that can set `defeated_by` are the four `type 0..3`
paths with a captor: `lost_a_city` (all four types), `process_elimination`,
`gain_tech` and `leave_game`'s capital arm. Value: a slot index `0..9`
(the captor), or the defeated leader's own index in the `leave_game` edge.

### Reset

`Leader::init (006e3930)` to `-1`; the `tribe < 0` memset to 0 just before
it. Nothing else.

### Readers

`Leader::defeat_by (006d1c80)` (its own once-only guard),
`LeaderData::log_data (006e5110)` (the dump), and the checksum walk. No
gameplay reader found in the typed export.

### Crate site

The crate has the event: `Sim::defeat` in `crates/sim/src/cities/city.rs`
(`pub fn defeat(&mut self, who, by: Option<Player>)`), reached from
`Sim::lost_a_city` (same file) and documented as `Leader::defeat →
defeat_by`. It keeps only `self.leaders.defeated[who]`
(`crates/sim/src/leaders.rs:35`). A builder adds `defeated_by` beside
`defeated` in `leaders.rs` and sets it in `Sim::defeat` when `by` is `Some`
and the field is still `-1`. `Sim::resign` (`crates/sim/src/leaders/resign.rs`)
is the `by = -1` path and leaves it. `process_elimination`'s capital timer
and `gain_tech`'s arm are not ported as defeat paths (the crate's
`lost_a_city` models only the default mode's "no city left").

### Plain game?

Yes. A skirmish against the computer reaches `lost_a_city` (the last city
captured) and, in a capital-elimination lobby, `process_elimination`.
`gain_tech`'s arm needs a game where someone gains the bonus preq while
others have lost their capitals — also a plain game. None needs triggers,
CtW or multiplayer, except `leave_game`'s capital arm (a human quitting
under a running capital timer).

### Confidence

High on the writer set, the once-only guard and the `by >= 0` predicate
(listing read). Medium on the elimination-mode semantics of
`lost_a_city`'s branches (`param_1` / `param_2` meanings were read from
which branch tests which, not from the callers of `lost_a_city`). Not read:
`Leader::lost_a_city`'s callers, `Game::retake_capital`.

## Leaf 6: `defeat_stamp` (+0x6924, int)

### Writers

Export `field_0x6924`: `Leader::defeat_by (006d1c80)` line 78,
`Leader::init (006e3930)` line 270, and a reader
`AchieveChartPane::set_chart_data (007f2c40)`. Listing: stores at
`6d1d98` (`movl %eax, 0x6924(%edi)` in `defeat_by`) and `6e3e0f`
(`Leader::init`); no absolute `0xe40cb4` reference.

### The event

`defeat_by` stores `game->frame` (`game + 0x550`, a sim frame) whenever it
is entered with `defeated_by < 0`, **before** it tests `by`. Every
`Leader::defeat` call enters `defeat_by` first (`6ecb1d`), so the stamp is
written on every defeat call made while `defeated_by` is still `-1` —
including calls on a leader who is already defeated: `Leader::defeat`
tests `leader_flags & 0x40` only *after* `defeat_by` returns. So:

- a defeat with a captor (`by >= 0`) stamps once and freezes it (the guard
  closes);
- a defeat with `by = -1` stamps and leaves the guard open, so any later
  `Leader::defeat` on that leader re-stamps. `Unit::close (0060ee50)`'s
  call has no defeated-flag test (listing `60f42f..60f474`: owner `< 8`,
  `active (+0x93c) == 0`, `city_num (+0x3f8) == 0`, `semaphore[1] & 8 == 0`,
  `leader_flags2 & 1 == 0`), so a resigned or otherwise captor-less loser
  whose leftover units later close is re-stamped at each such close.

Under `type == 5` the blow-up loop is skipped, but the stamp is not.

### Reset

`Leader::init (006e3930)` to 0 (and the `tribe < 0` memset).

### Readers

`AchieveChartPane::set_chart_data (007f2c40)` (UI chart),
`LeaderData::log_data (006e5110)`, the checksum walk.

### Crate site

`Sim::defeat` in `crates/sim/src/cities/city.rs` and `Sim::resign` in
`crates/sim/src/leaders/resign.rs` are the events; both return early when
`self.leaders.defeated[who]` is set, which is **not** the original's
predicate for this field. A builder keeps `defeat_stamp` beside `defeated`
in `crates/sim/src/leaders.rs`, writes `self.frame` at entry of both while
`defeated_by == -1` and before the early return, and needs `Unit::close`'s
re-entry — the crate's unit close does not call `defeat` at all (grep finds
no `defeat` call from `units/`), so that re-stamp is a missing event.

### Plain game?

Yes: any defeat (city loss, last unit, a human quitting through the menu,
the conquest victory's type-5 sweep). `rondata/src/diff/third_pair.rs`
already names a capture where it reads `17380`.

### Confidence

High (both stores and the entry order read in the listing). Medium on how
often the `Unit::close` re-stamp fires in practice (depends on whether a
loser keeps units that later close; not measured).

## Leaf 2: `wonderwin_stamp` (+0x448, int) and Leaf 3: `wonderwin_timer` (+0x44c, int)

The two are one clock: a running flag and a frame stamp, stepped together.
They are written up together; each has its own writer table.

### Writers

Export `field_0x448` / `field_0x44c` on a leader: `Leader::init
(006e3930)` lines 379–380, `Leader::process (006b88b0)` lines 52–53,
`Leaders::process_all (006ed2a0)` lines 58–60, `GameDaemon::process_victory
(00730ef0)` lines 100–103 and 142–145. Every other hit (`ConquestInvadeWin`,
`GameReportWin`, `TerrainOut`, `Group`, `CoordInfo`) is a different `this`.
Listing, register-relative stores at `0x448(` / `0x44c(` (all functions,
then each base register typed by hand):

| store | function | base is a leader? |
|---|---|---|
| `6e4160` / `6e4166` `movl %eax, 0x448/0x44c(%ebx)` | `Leader::init (006e3930)` | yes (`this`) |
| `731080` `addl %eax, 0x448(%ebx)`; `731070` `movl $1, 0x44c(%ebx)` | `GameDaemon::process_victory (00730ef0)` | yes (`ebx` walks `0xe3a390` by `0x6eec`) |
| `731367` `subl %eax, 0x448(%ebx)`; `731357` `movl $0, 0x44c(%ebx)` | same | yes |
| `6ed3c8` `movl %eax, 0x448(%esi)` | `Leaders::process_all (006ed2a0)` | yes |
| `6b89c8` `movl %eax, 0x448(%esi)` | `Leader::process (006b88b0)` | yes, but **no reference to `0x6b88b0` exists in `.text`** (map: `?process@Leader@@QAEXXZ`, `leaders.obj`); LTCG inlined it into `Leaders::process_all`, so this body is dead |
| `9ad0e7` / `9ad0ed` | `ScenarioRead::load_leaders_chunk (009acde0)` | no: `edx` is the game-info player table (stores at `0x28c..0x450` with the `0x8c` player stride) |
| `6b87e1`, `6ed1cf` (`movw`) | `Leader::end_process (006b8690)`, `Leaders::end_process_all (006ed070)` | no: game-info `player[7].flags` |
| `56c628`/`56c643`, `516e7a`, `58ba04`, `7144d2`, `923608`, `959725`, `95980e` | `Constants::init`, `GraphicWinXML::close`, `Game::init_teams`, `Group::add`, `RTreeData::clear`, `DropControl::process_drop` | no |

No absolute `0xe3a7d8` (stamp) store; the four absolute `0xe3a7dc`
(timer) references are all reads.

### The event: `GameDaemon::process_victory (00730ef0)`, the wonder arm

Called every frame from `GameDaemon::process_all (00732700)` (its only
caller), which `Game::do_frame (00591ef0)` calls unconditionally. The arm
runs only when the lobby's `victory` byte (`game + 0x38`) is 0, 1 or 6
(`730f2d..730f3a`). It reads `w = Game::wonder_winning (005948a0)` once
(slot index of the leader whose `get_wonder_net` is highest and at least
the lobby `wonderwin` row's `DATA`, ties going to the higher
`get_wonder_value` among allies and to nobody between non-allies; `-1` if
none). Then it walks slots `i = 0..7` (`-0x14` runs `0x74..0x94` by 4 — the
**eight** player slots, not ten), skipping a slot unless
`leader_flags & 3 == 3`. A per-pass flag `handled` (`-0x1c`) starts 0.

For each seated, undefeated slot `i`:

1. **Not on the winning team** — `w < 0`, or (`i != w` and `who_i != w`
   and not (`diplos_i[w] == 2` and `leaders[w].diplos[who_i] == 2`))
   (`730f84..730fb4`; `diplos` is `+0x74 int[8]`, 2 is an alliance, checked
   both ways): if `timer != 0`, then `timer = 0` and
   `stamp -= game->frame` (`731348..731367`), achievement event 10
   (`TIMER_STOP`), and the stop sound once per pass (`-0x28`).
2. **On the winning team**: first `ggb` = 1 if any seated leader `j`
   (`leader_flags & 1`) that is `i`, or has `who_j == i`, or is a mutual
   ally of `i` (`diplos_j[i] == 2` and `diplos_i[who_j] == 2`), holds tech
   preq `0x2b9` (`730fba..731023`). Then:
   - if lobby `victory != 0`, or the game is neither multiplayer
     (`semaphore[0] & 4`) nor has `num_nations (+0x6a0) > 1`, or `ggb`: if
     `handled == 0`, `Leader::victory(i, 1, ggb)` (`731320..73132e`) — an
     **immediate** wonder victory, no clock;
   - else if `timer == 0`: `timer = 1`, `stamp += game->frame`
     (`731070..731080`), event 9 (`TIMER_START`); the messages and start
     sound only when `handled == 0`;
   - else (`timer != 0`), only when `handled == 0`:
     `remaining = Game::wonder_timer() − (frame − stamp)` (`7311cc..7311dd`).
     If `remaining == 0` (an **exact** equality, `subl; jne`), then
     `Leader::victory(i, 1, 0)`. Otherwise, for a viewer who is not `i`'s
     ally, warnings at `remaining` = 4500, 1800 or 900 frames
     (`0x1194`, `0x708`, `0x384`).
   - `handled = 1` after any of these (`73133c`, `7311b6`). The walk does
     **not** stop: later slots on the same team still start their own
     clocks, but do not count down or win.

`Game::wonder_timer (005944f0)`: 0 if `constants.wonder_timer < 1`, else
`max(1, (M/2 + W·T) / M)` in integers, with `T = constants.wonder_timer`
(`Constants::init` loads it with plain `get_item`, so it is the file's
integer), `W` the world's first int (`*GameAccess::world`) and `M` the int at
`map_sizes` entry `+0x144`. So `T` is frames on a reference map, scaled by
the map's ratio and rounded half up.

**So the stamp encodes accumulated time.** Start at frame `F`:
`stamp += F`; elapsed while running is `frame − stamp`. Stop at `G`:
`stamp −= G`, leaving `stamp = −(time run so far)` when it started at 0.
A restart adds the new frame back, so elapsed resumes where it stopped.

### The pay-back: `Leaders::process_all (006ed2a0)`

Every frame `Game::do_frame` calls `Leaders::process_all`, which for each
of the ten slots with `leader_flags & 2` (alive), after `gather`,
`calc_wall_stats`/`calc_unit_stats` and `process_elimination`, runs: when
`constants.timer_refresh_ratio != 0` and
`game->frame % timer_refresh_ratio == 0`, if `wonderwin_stamp < 0` and
`wonderwin_timer == 0`, `wonderwin_stamp += 1` (`6ed3c8`). It does the same
for `lost_capital_stamp/timer (+0x414/+0x418)` and `popwin_stamp/timer
(+0x440/+0x444)` just before. So a stopped clock's debt shrinks by one frame
every `timer_refresh_ratio` frames (the crate's `tuning.rs` pins 5).

### Step summary

- `wonderwin_timer`: 0 → 1 at the start, 1 → 0 at a stop. Never another
  value. Only `process_victory` writes it in play.
- `wonderwin_stamp`: `+frame` at a start, `−frame` at a stop, `+1` per
  `timer_refresh_ratio` frames while stopped and negative.

### Reset

`Leader::init (006e3930)` stores 0 to both on every init (and the
`tribe < 0` memset). A victory does not reset them; a defeat does not
either (but a defeated slot fails `leader_flags & 3 == 3` and is no longer
walked, so its clock freezes in whatever state it was; and `leader_flags & 2`
is cleared by `Leader::defeat`, so its pay-back stops too).

### Readers

- `wonderwin_timer`: `Game::is_victory_timer (00594950)` (any seated,
  undefeated leader's timer or popwin timer running, under lobby
  `victory == 0`), itself read by `Leader::diplomacy (006bc950)`,
  `IFaceDiplo::update_visibility (008004f0)`, `Options::describe
  (00724460)`; `Leader::diplomacy` also reads `0xe3a7dc(%eax)` directly
  (`6be27f`); `Army::find_target (006f69b0)` (`6f6c38`, `6f72e0`, the
  "about to win" gate and score); `Wall::process (00640450)` (an unallied
  leader's running timer changes the wall walk); `LeaderData::log_data`.
- `wonderwin_stamp`: `IFaceMainBase::draw_timers (0080b060)` (the
  on-screen countdown), the countdown in `process_victory` itself,
  `LeaderData::log_data`.

### Crate site

The crate has the twin arm for the territory clock:
`Sim::process_victory_territory` in `crates/sim/src/territory/victory.rs`,
called from `crates/sim/src/core/frame.rs:222` in `GameDaemon::process_all`
order, with its stamp/timer on `crate::ai` (`popwin_stamp`/`popwin_timer`,
`crates/sim/src/ai.rs:711`), and the pay-back `Sim::refresh_victory_timers`
(same file, called from `core/frame.rs:194`). A builder adds
`wonderwin_stamp`/`wonderwin_timer` beside the popwin pair in `ai.rs`, a
`process_victory_wonder` arm run **before** the territory arm in
`frame.rs` (the original runs the wonder arm first, `730f2d` before
`7313b9`), and one more `stamp += 1` line in `refresh_victory_timers`.
`Sim::wonder_winning` already exists (`crates/sim/src/ai/ai_build.rs:1439`,
swept against the emulator in `sweep/game_wonder_winning.rs`), and
`lobby.wonder_win_points` exists (`ai.rs:1097`). Missing: the
`WONDER_TIMER` constant (not in `core/tuning.rs`), `Game::wonder_timer`, the
`victory` event itself. Readers already stubbed: `ai/army/target_gate.rs:428`
`fn w44c` returns 0, and `cities/site_recruit.rs:157` carries a SEAM for
the running timer. Note the arm differs from the territory arm in two
ways a copy would miss: the wonder walk does not return on the first
holder (it sets `handled` and keeps walking), and its expiry is `== 0`
where the crate's territory seam table says `< 1`.

### Plain game?

Yes for the clock: a skirmish under the default lobby (`VICTORY 0`) with
a wonder-victory row reaches the start, stop, countdown and pay-back once a
leader's wonder points reach the `wonderwins` `DATA`. The immediate arm
(`VICTORY` 1 or 6, a single-nation game, or a `GLOBAL_GOVERNMENT_BONUS`
holder on the team) is also a plain game, with a different lobby setting.
No trigger, CtW or multiplayer path writes either field.

### Confidence

High on the writer set (both nets agree, every listing store typed), the
step amounts, the predicate order and the pay-back. Medium on
`Game::wonder_winning`'s tie rule (read in the decompile only; the crate's
sweep covers it). Not read: `Wall::process`'s and `Army::find_target`'s use
beyond "nonzero"; the meaning of `semaphore[0] & 4` beyond multiplayer;
`map_sizes +0x144` and `*world`'s names.

## Leaf 4: `victory_type` (+0x7d8, int)

### Writers

Export `field_0x7d8` on a leader: one store, `Leader::victory (006ec9b0)`
line 23; the rest are reads (`EndGameWin::exec`,
`Game::determine_end_of_game_stats_and_achievements`). `->victory_type`
also matches `EndGameWin`'s own member (`end_game_win.victory_type`), a
copy, not this field. Listing: the only leader store is `6ec9ec`
(`movl %eax, 0x7d8(%esi)` in `Leader::victory`); the other `0x7d8(` stores
(`Constants::init`, `GraphicPieces::close`, `ParamRegisterWindow::registration`)
are other records; no absolute `0xe3ab68` store.

### The event

`Leader::victory(type, instant)`: if `leader_flags & 0x60 == 0` (neither
won, `0x20`, nor defeated, `0x40`; `6ec9be`), it sets `0x20`, sets or clears
`leader_flags2 & 0x100` from `instant`, stores `victory_type = type`, posts
`ACHIEVEEVENT_WON`, cleans every building queue, calls `Player::win` for
the leader's players, then for every other slot with `leader_flags & 3 == 3`
sets `0x80` and calls `victory(type, instant)` on it if it counts this
leader as an ally (`LeaderData::is_ally`, one direction) or `defeat(5, -1, 0)`
otherwise. So the value is set once (the guard) and copied unchanged to the
winner's allies.

Values, by caller (listing pushes `instant, type`):

| type | meaning | caller and predicate |
|---|---|---|
| 0 | last team standing | `Game::check_victory (005926b0)` `592984` (all remaining undefeated seated leaders mutually allied; called from `Leader::defeat`, `Leaders::strategy_all` under `semaphore[1] & 2`, `ScenarioFuncSet::victory`); `Leader::set_diplo (006ec6a0)` `6ec96e` (an alliance leaves no unallied rival); `TurnControl::toggle_pause (00957ab0)` `957bbf`; `ConsoleWin::run_cmd (007d6a70)` `7dd482`; `ScenarioFuncSet::victory (009f3fa0)` |
| 1 | wonder | `GameDaemon::process_victory` `73132e` (see leaves 2–3) |
| 2 | territory | `process_victory` `731942` (the POPWIN arm, lobby `victory` 0, 1 or 7) |
| 3 | technology / ending age | `Leader::gain_tech (006dcb60)` `6de884`, `6de992` (lobby `victory == 9`) |
| 4 | score | `process_victory` `7319b5` (lobby `victory == 3`, team score at the `scores` row) |
| 5 | economic | `process_victory` `731f77` (lobby `victory == 8`, team economy at the `econwins` row) |
| 6 | time limit | `process_victory` `731a69`, `731ac7` (lobby `victory == 4`) |

### Reset

None in play. `Leader::init (006e3930)` does **not** store it; only the
`tribe < 0` memset zeroes it, and a load (`walk_data`) restores it. So
after an `init` with `tribe >= 0` the field keeps whatever it held. 0 is
also a real value (type 0, conquest), so a reader must pair it with
`leader_flags & 0x20`.

### Readers

`EndGameWin::exec (0083bcc0)` (console leader's, or slot 0's, into the
end-of-game window), `EndGameWin::get_type_string (0083a420)` through that
copy, `Game::determine_end_of_game_stats_and_achievements (00583b30)`
(`58403c`, achievements), `LeaderData::log_data`.

### Crate site

No victory event in the crate: `grep` finds no `victory(` function, no
`check_victory`, no `0x20` won flag; `territory/victory.rs`'s `seams` table
names the territory expiry as the missing `Leader::victory(2)`. A builder
adds `victory_type` (and the won flag it pairs with) to
`crates/sim/src/leaders.rs` beside `defeated`, and a `Sim::victory(who,
type, instant)` in `crates/sim/src/leaders/` (beside `resign.rs`) with the
ally sweep that calls `Sim::defeat` with type 5; the first caller would be
the territory expiry seam in `territory/victory.rs`, then a
`check_victory` port called from `Sim::defeat`.

### Plain game?

Yes: a skirmish against the computer ends by conquest (`check_victory`,
type 0), or by wonders / territory under the default lobby. Types 3–6 need
non-default lobby victory settings but no triggers, CtW or multiplayer.
Only the console and scenario callers are outside a plain game.

### Confidence

High on the single writer, its guard and the propagation. High on the type
values per caller (pushes read in the listing). The lobby-mode names (score,
economic, time limit, ending age) are read from the arms' tables
(`scores`, `econwins`, `time_limits`, `ending_technology`), not from the
lobby UI.

## Leaf 5: `defeat_type` (+0x7dc, int)

### Writers

Export `field_0x7dc` on a leader: one store, `Leader::defeat (006ecb00)`
line 69; every other hit is a read. Listing: the only leader store is
`6ecb7b` (`movl %esi, 0x7dc(%ebx)`); other `0x7dc(` stores
(`IFaceRenderManager::clear_all`, `Constants::init`,
`GraphicPieces::close`) are other records; no absolute `0xe3ab6c` store.

### The event

`Leader::defeat(type, by, instant)`: calls `defeat_by(by, type)` first
(leaves 1 and 6), then returns without writing if `leader_flags & 0x40`
(already defeated, `6ecb24`) or `semaphore[1] & 2` (`game + 0x821` bit 1, the
unit-balance test mode, `6ecb32`). Otherwise it sets or clears
`leader_flags2 & 0x200` from `instant`, clears `leader_flags & 2` and sets
`0x40`, calls `Armies::leader_defeated`, sets `leader_flags2 & 2`, stores
`game->musical_chairs = frame`, then `defeat_type = type` (`6ecb7b`), and
posts `DISCONNECTED` (7), `RESIGNED` (6), nothing (5) or `ELIMINATED`
(others). So `defeat_type` is written **once**, at the first effective
defeat — unlike `defeat_stamp`, which `defeat_by` writes before this guard.

Values: the `type` column of the `Leader::defeat` caller table under leaf 1.
In words: 0 no cities and no units left (`lost_a_city` default mode,
`Unit::close`), console, scenario attacker sweep; 1 capital elimination
(`lost_a_city` with `GLOBAL_GOVERNMENT_BONUS`, `process_elimination`,
`gain_tech`, `leave_game` under a capital timer); 2 and 3 the elimination
modes 2 and 3 of `lost_a_city`; 4 musical chairs (`process_victory`, lobby
`victory == 5`, the lowest score every `chairs`·900 frames); 5 someone else
won (`Leader::victory`); 6 resigned; 7 dropped or disconnected
(`Player::drop`, `CommandPackage::end_process`); 8 armageddon
(`Game::defeat_all` from `process_victory` when
`Game::get_armageddon() <= game->armageddon`); 9 scenario defeat. The
readers also test **10** (`EndGameWin::get_type_string`,
`ConquestGame::process_results (00798940)` `799194: cmpl $0xa, 0xe3ab6c`),
but no caller of `Leader::defeat` pushes 10 and no other store exists, so
10 reaches this field only through a load (`walk_data`) or not at all; it
looks like an `EndGameWin`-side value (that class has its own
`defeat_type`).

### Reset

None in play. `Leader::init` does not store it; only the `tribe < 0`
memset and a load.

### Readers

`EndGameWin::exec (0083bcc0)`, `EndGameWin::get_type_string`,
`Game::determine_end_of_game_stats_and_achievements (00583b30)`,
`ConquestGame::process_results (00798940)` (CtW), and the UI:
`StatWin::draw_normal_stats (007ebd20)`, `StatWin::on_redraw (007f0e70)`,
`AchieveChartPane::on_redraw (007f6ff0)`, `AchieveChartPane::draw_game_info
(007f3ce0)`, `AchieveGraphPane::on_spot_move (007f7e40)`,
`GameReportWin::on_button_clicked (007b4f50)` (the `== 8` tests),
`IFaceDiplo::draw_player_area (007ff420)`,
`IFaceDiplo::update_visibility (008004f0)`; `LeaderData::log_data`. No sim
reader found.

### Crate site

`Sim::defeat` (`crates/sim/src/cities/city.rs`, called only by
`Sim::lost_a_city`, so always the default mode's type 0 with a captor)
and `Sim::resign` (`crates/sim/src/leaders/resign.rs`, type 6). A builder
adds a `type` argument to `Sim::defeat` and stores `defeat_type` in
`crates/sim/src/leaders.rs` after the existing `defeated` early return
(that return matches the `0x40` guard here, unlike leaf 6). `Unit::close`'s
type 0, the type-5 sweep (no `Sim::victory`), musical chairs, armageddon
and `process_elimination` are not ported events.

### Plain game?

Yes for 0, 1, 5 and 6 (city loss, capital elimination lobby, conquest
victory sweep, a human quitting through the menu); 2, 3, 4, 8 with
non-default lobby settings but still a plain skirmish; 7 is multiplayer;
9 and the console 0 are triggers/cheats.

### Confidence

High on the single writer and its guard (listing), and on the per-caller
values. Medium on the words for elimination modes 2 and 3 (read from
`lost_a_city`'s branches only) and on armageddon's predicate (not read
past `get_armageddon`'s call).

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `defeated_by` +0x10 | `Leader::defeat_by (006d1c80)` (sole caller `Leader::defeat (006ecb00)`); `Leader::init (006e3930)`; bulk: `LeaderData::walk_data (006d6750)` on load | `= by` once, when still `< 0` and `by >= 0`; captor-less defeats leave `-1` | `-1` in `Leader::init` | `Sim::defeat`, `crates/sim/src/cities/city.rs`; field beside `defeated` in `leaders.rs` | yes (last city captured; capital elimination) | high (writer/guard), medium (elimination-mode words) |
| `wonderwin_stamp` +0x448 | `GameDaemon::process_victory (00730ef0)` (`+frame` start, `−frame` stop); `Leaders::process_all (006ed2a0)` (`+1` pay-back); `Leader::init (006e3930)`; dead copy `Leader::process (006b88b0)` | start `+frame`, stop `−frame`, `+1` every `timer_refresh_ratio` frames while `< 0` and stopped | 0 in `Leader::init` | twin of popwin in `crates/sim/src/territory/victory.rs` (`process_victory_territory`, `refresh_victory_timers`); field beside `popwin_stamp` in `ai.rs` | yes (default lobby, wonder points at the `wonderwins` row) | high |
| `wonderwin_timer` +0x44c | `GameDaemon::process_victory (00730ef0)`; `Leader::init (006e3930)` | 0→1 at start, 1→0 at stop | 0 in `Leader::init` | same; reader stub `ai/army/target_gate.rs:428` `w44c` returns 0 | yes | high |
| `victory_type` +0x7d8 | `Leader::victory (006ec9b0)` only | `= type` once (guard `leader_flags & 0x60 == 0`), copied to allies | none in play (only the `tribe < 0` memset, and a load) | none: no victory event in the crate; add `Sim::victory` in `crates/sim/src/leaders/` | yes (conquest type 0; wonder/territory 1/2; 3–6 by lobby) | high |
| `defeat_type` +0x7dc | `Leader::defeat (006ecb00)` only | `= type` once (after `defeat_by`; guard `leader_flags & 0x40 == 0` and not `semaphore[1] & 2`) | none in play (memset, load) | `Sim::defeat` (`cities/city.rs`), `Sim::resign` (`leaders/resign.rs`); add a type argument | yes for 0, 1, 5, 6; 2–4, 8 by lobby; 7 MP; 9 scenario | high (writer/guard), medium (mode words) |
| `defeat_stamp` +0x6924 | `Leader::defeat_by (006d1c80)`; `Leader::init (006e3930)` | `= frame` on **every** `Leader::defeat` call while `defeated_by < 0` (before the defeated-flag guard), so captor-less losers are re-stamped by later calls, e.g. `Unit::close (0060ee50)` | 0 in `Leader::init` | `Sim::defeat` / `Sim::resign` entry, before their `defeated` early return; `Unit::close`'s defeat call is not ported | yes | high (stores, order), medium (how often the re-stamp fires) |

### What this reading did not read

`Leader::lost_a_city`'s callers (so `param_1`/`param_2` are named by what
they gate, not by what passes them); `Game::retake_capital`;
`Game::get_armageddon`; the wonder clock's consumers in `Wall::process` and
`Army::find_target` beyond "nonzero"; `map_sizes +0x144` and `*world`; the
`walk_data` callers' conditions. No capture or emulator run was made; every
claim here rests on the export and the listing.
