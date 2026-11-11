# Family 16, `lobby-words` — first reading

Item 4152, wave 1. Leaves: `cannon_time` (`LeaderData`+0x48, ulong), `handicap` (+0x4c, int), `multi_diff` (+0x50, int). Reader: Claude Opus 5.5, 2026-11-11.

`Leader` is 0x6eec and `LeaderData` 0x6ee4 (`types.txt` lines 42086/42088): a `Leader*` reaches a `LeaderData` leaf at off+8 (vtable and one more word ahead of the data). The leaders array's `LeaderData` for slot i is at 0xe3a390 + i*0x6eec, so the leaves sit at 0xe3a3d8 / 0xe3a3dc / 0xe3a3e0 + i*0x6eec.

(Sections are appended per leaf as each is finished.)

## `cannon_time` (+0x48, ulong)

**What it is.** The number of "cannon time" uses this leader has left: a
multiplayer pause-like allowance that drops the game speed to 0 for 75
frames. Units: a count of uses.

### Writers, both nets

- Export, `->cannon_time`: no hit (the field is only ever reached as
  `leaders.list[i].field_0x48` or `this->field_0x48` on a `Leader *`).
  `Leader` (0x6eec) is `LeaderData` (0x6ee4) at +0 with a vbtable pointer
  at +0x6ee4 (`Leader::Leader (006d2bf0)` stores `_vbtable_` there), so a
  `Leader *`'s `field_0x48` is `LeaderData+0x48` with no prefix; the
  `TerrainOut`-style shift does not apply.
- Listing, `0xe3a3d8(%reg)`: three sites. `CommandManager::issue_cannon_time
  (00941260)` at 009412f1 (a `cmpl`, read), `TurnControl::start_cannon_time
  (00956500)` at 0095658a (load) and 009565eb (store).
- Listing, register-relative `0x48(%reg)` stores in every `Leader`/
  `LeaderData`/`Leaders` method and every function that touches the
  leaders array: `Leader::init (006e3930)` at 006e3af3, base `%ebx` =
  `this` (`movl %ecx,%ebx` at 006e3942). The other `0x48(%reg)` store in
  `Leader::init`, 006e3f6a, is off `%eax = *(this+0x6eb8)`, the
  `LeaderDataEncrypt *`, not this leaf. `Leader::compute_sites`' 006cca21
  is off `*(this+0x6e44)`; `Leader::gain_tech`'s `0x48(%ebp)` stores are
  EH-prolog locals (`%ebx` is `this` there).
- Bulk: `Leader::init`'s `tribe < 0` arm calls `memset(this, 0, 0x6929)`
  (006e398c..006e3995: push 0x6929, 0, `%ebx`), which zeroes all three
  leaves of this family.
- Save-game load: `LeaderData::walk_data (006d6750)` hands
  `[+0x8, +0x692a)` to the `DataWalk`; with `input != 0` (a load) that
  writes the leaf from the file. Callers not followed beyond the checksum
  (read-only) use.

### Event, step, predicate

1. **Set from the lobby.** `Leader::init (006e3930)`, on every call
   whatever the tribe, after the `tribe < 0` clear: `cannon_time =
   cannon_times[game.info.cannon_times].data[0]` — `movzbl 0x33(game)`
   (`Game +0xc` `info`, `+0x18` the options block, `+0xf`
   `cannon_times`), times the 0x58-byte category record, `+0x3c` its
   first `DATA` (006e3ad8..006e3af3). rules.xml's `cannontimes` rows are
   0, 1, 2, 4, 8 for rows 0..4 ("None" .. "8 Cannon Times"). Callers:
   `Leaders::init (006ed850)` (all ten slots, `tribe = -1`, so clear then
   set), `Setup::build_game (005ac190)` per active leader, plus
   `Setup::build_leader`, `ScenarioRead::load_leader_chunk` (6 calls),
   `ScenarioEditor::new_leader`, `MapScenario::make`,
   `ConquestGame::build_conquest_game`, `WalkDataGame::walk_data`.
2. **Spent, by 1.** `TurnControl::start_cannon_time (00956500)`, reached
   only from `CommandPackage::process_cannon_time (009464f0)` (command
   0x4d, `CommandPackage::process (0094a700)`), which passes the leader of
   the package's `play`. Predicate, from the listing: `who >= 0`; and
   nobody already in cannon time (`turn_control.cannon_time_who < 0`,
   `cmpl $0,0x24(%esi)` at 0095652e — if someone is, the local console
   gets a feedback string and nothing changes); and `cannon_time != 0`
   (0095658a..00956592 — zero gives the console a refusal and returns).
   Then `cannon_time -= 1` (`decl` then store at 009565eb), and
   `cannon_time_who = who`, `cannon_time_stamp = frame`,
   `cannon_saved_speed = speed`, `cannon_time_start = tick`; if `speed !=
   0` the speed goes to 0 with `set_new_target_time(200)`.
   `TurnControl::check_cannon_time (009579e0)` (from `Game::do_frame`)
   ends it once `frame - cannon_time_stamp > 0x4a` (75 frames); it does
   not touch the leaf.
3. **Issued only in multiplayer.** `CommandManager::issue_cannon_time
   (00941260)`, from `Options::exec (007188c0)` case
   `OPTION_CANNON_TIME`, refuses with a message unless `game.semaphore
   [0] & 4` (`testb $0x4,0x820(game)` at 00941282), and otherwise queues
   command 0x4d only if the console leader's `cannon_time != 0`
   (009412f1).

### Reset

`Leader::init`'s `tribe < 0` memset (via `Leaders::init`, from
`Game::init_rules_and_teams (00589bb0)`), then the same function's lobby
store. `Leaders::clear (006ed5d0)` also memsets every slot's
`[+0, +0x6929)` to 0 (006ed5e0..006ed5ee, base `0xe40fa4 - 0x6c14` =
`0xe3a390`), from `Game::close` and `Game::run_oos_recovery` — the end of
a game, not a start. `Leader::reset_score (006e37f0)` does not touch
+0x48 (its only bulk store is a 7-dword `-1` fill at `this+0x8f8`). No
per-frame fold.

### Readers

`start_cannon_time` (the guard above), `issue_cannon_time` (the guard),
`IFaceMainBase::update_visibility (00811d40)` (greys the cannon button
when 0 or when someone is in cannon time), `LeaderData::walk_data`
(checksum and save), `LeaderData::log_data` (not printed: the dump has no
`cannon_time` row). No reader changes a sim decision except through the
speed change, which is the turn controller's, not the sim's.

### Crate

The crate has no cannon time: no `cannon_times` lobby field
(`crates/sim/src/ai.rs` `Lobby`, line ~1041), no command 0x4d in the sim
(`rondata::commands` decodes `Command::CannonTime` at
`crates/rondata/src/commands.rs:996`, and nothing in `crates/sim` consumes
it). The checksum leaf is listed at `crates/sim/src/checksum/leaders.rs:53`
and held out (`crates/rondata/src/diff/checksum_lines.rs:1771`,
"cannon_time (unheld)"). A builder adds a `"cannon_time"` arm to
`Sim::leader_long` (`crates/sim/src/checksum/leaders.rs:949`) returning
the lobby row's count, which needs `Lobby` to carry `CANNON_TIMES` and
its `DATA` (0 for row 0).

### Plain game?

**Value in a default skirmish: 0** for every active slot — `CANNON_TIMES 0`
in run11 and run1313 (`DIFFICULTY` lines nearby), and row 0's `DATA` is 0.
Even with a non-zero lobby row, the decrement is unreachable solo: the
only issuer refuses without `semaphore[0] & 4` (multiplayer). A recorded
or crafted command 0x4d in a solo game would still reach
`process_cannon_time` (no semaphore test there or in
`start_cannon_time`), but with the lobby's 0 it is refused at the
`cannon_time != 0` guard. So in a plain game the leaf is the lobby
constant, frozen.

### Confidence and not read

High for the writers (both nets agree: three displacement sites, one
`this`-relative store, one memset). Not followed: `WalkDataGame::walk_data`
and the save-load `DataWalk` callers; whether the CtW and scenario
`Leader::init` callers pass a different `cannon_times` (they cannot: the
store reads the game's lobby byte, not an argument).

