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
