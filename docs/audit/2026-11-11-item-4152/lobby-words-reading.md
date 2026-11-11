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

## `handicap` (+0x4c, int)

**What it is.** A row index into rules.xml's `handicaps` category
("Standard" 0, "Skill +1" 5, "Skill +2" 10, … as `DATA` at `+0x3c` of
each 0x58-byte record), computed once at game start to even out the
human players' lobby handicaps across sides. Units: a table row, 0..20.

### Writers, both nets

- Export, `->handicap` on a `LeaderData`: only the reader
  `LeaderData::log_data`; every other `->handicap` hit is `Player`'s or
  `PlayerConnectionData`'s byte (+0x35 / +0x38 of those records).
  `leaders.list[i].field_0x4c`: `Game::init_handicaps (0058abf0)` (as a
  walking pointer) and `ScenarioRead::load_leader_chunk (009ac080)` (three
  stores).
- Listing, `0xe3a3dc`: `movl $0xe3a3dc,%esi` at 0058ad86 in
  `Game::init_handicaps`, then stores `movl %eax,(%esi)` at 0058ade8 and
  0058ae07, stepping `%esi += 0x6eec` to `< 0xe71b3c` (slots 0..7);
  `ScenarioRead::load_leader_chunk` at 009ac1fc, 009ac457, 009ac643
  (`movl %eax,0xe3a3dc(%edi)`, `%edi = slot*0x6eec`); `ScenarioWrite::
  save_leaders_chunk (009a77c0)` reads it at 009a7a80.
- Register-relative `0x4c(%reg)` stores in the leader methods: none on a
  `LeaderData` base. The candidates the scan raised are other records:
  `Leader::compute_sites` 006cca1a (off `*(this+0x6e44)`),
  `Leader::gain_tech` (EH-prolog locals off `%ebp`),
  `ScenarioRead::load_leaders_chunk` 009ace3f (`%edx = game`, `0xc061ec`).
- Bulk: `Leader::init (006e3930)`'s `tribe < 0` memset and `Leaders::clear
  (006ed5d0)`'s memset, both `[+0, +0x6929)` to 0; the save-load
  `LeaderData::walk_data (006d6750)` with `input != 0`.

### Event, step, predicate — `Game::init_handicaps (0058abf0)`

Called once, from `Game::init_rules_and_teams (00589bb0)` after
`Leaders::init` and `init_teams` and before `init_tribes`. Read from the
listing (the decompile prints "type propagation not settling"). `Player`
records are the eight of `game.info.player`, 0x8c bytes each; offsets
below are `Player`'s (`flags` +0x30, `who` +0x33, `team` +0x34 signed,
`handicap` +0x35, `diff` +0x38). `flags` bit 0 is an occupied slot, bit 2
a person (a recorded human slot holds 0x0007, an AI slot 0x2001 —
`crates/rondata/src/recgame.rs:33`).

Pass 1, for each leader slot `i` in 0..7 with `leader_flags & 1`:

- `own[i]` = the largest `handicap` of the occupied players with `who ==
  i` (any kind; floor 0). `aidiff[i]` = the largest `diff` of the
  occupied players with `who == i` and **not** a person (floor −1).
- `sum[i]` starts at `own[i]` if `Game::is_cooperative (00594810)` is
  true, else 0. Then over the players with `flags & 5 == 5` (occupied
  people):
  - `who == i`: `own[i] = max(own[i], handicap)`, `sum[i] += handicap`;
  - a different `who` whose `team` differs from `LeaderData::get_team
    (006ec040)` of `i`: `enemy[i] = max(enemy[i], handicap)` (floor 0),
    nothing added;
  - a different `who` on the same team: `mates[i] += 1` unless that
    player's `team == 8`, and `sum[i] += handicap` either way.

Pass 2: `top = max over active i of sum[i]` (the other two maxima the
listing computes, of `own` and `enemy`, are discarded — their registers
are overwritten before use at 0058ad84).

Pass 3, for each active slot `i` (0..7):

- `mates[i] == 0`: `h = top − sum[i]`, then 0 if negative, at most 20.
- else: `h = ((top − sum[i]) / (mates[i] + 1) + max(0, enemy[i] −
  own[i])) / 2`, each division truncating toward zero, then 0 if
  negative, at most 20.
- `handicap = h / max(1, game.num_teams − 1)` (`Game +0x6a4`,
  truncating).

So the leader with the largest summed human handicap gets 0 and the rest
get the shortfall, capped at row 20 and shared across the other teams.

Slots 8 and 9 (the two extra slots `Leaders::init` walks to `0xe7f8c8`)
are never written here; they keep the memset's 0 (run1313's record:
`who 8` and `who 9` print `handicap 0`).

### Other events

- `ScenarioRead::load_leader_chunk (009ac080)`: a scenario's leader chunk
  carries `handicap` at chunk `+0xe0`, stored after `Leader::init` in three
  of the chunk-version arms, each behind `(game.semaphore[3] & 4) == 0`.
  Scenario files only.
- Save-game load through `walk_data`.

### Reset

`Leader::init`'s and `Leaders::clear`'s memsets. No per-frame fold;
`Leader::reset_score` does not touch it.

### Readers

- `LeaderData::get_handicap (006da740)`: `handicaps[row].DATA` where
  `row = handicap`, shifted by the difficulty for a solo game (difficulty
  0 or 1 and a person: `+10` / `+5`; difficulty 4 or 5 and a computer:
  `+10` / `+16`) — but that shift only runs when `semaphore[0] & 4` is
  clear, and **every caller of `get_handicap` tests `semaphore[0] & 4`
  set first**: `ObjectData::train_time (006508c0)` twice (`time × (200 −
  h) / 200`), `World::compute_reg_territory (006b0bb0)` (a person only,
  `+ (h + 15) / 25` to the territory allowance), and
  `LeaderData::get_gather_handicap (006d66a0)` (a person only). The
  handicaps table `0xe80138` is referenced only by `get_handicap` and
  `GameInfo::load_from_config` in the whole `.text`, so there is no
  inlined copy.
- `LeaderData::get_handicap_level (006d6740)` returns it raw (callers not
  followed).
- `ScenarioWrite::save_leaders_chunk`, `LeaderData::log_data` (prints
  `handicap`), `LeaderData::walk_data` (checksum, save).

### Crate

No `init_handicaps`. `Holdings::handicap` (`crates/sim/src/economy.rs:621`)
is a different quantity — the income percentage `get_gather_handicap`
returns — and `crates/rondata/src/diff/leader.rs:55` says so. The
checksum leaf is listed at `crates/sim/src/checksum/leaders.rs:54`. A
builder would put the start-of-game computation beside
`Sim::sync_setup_from_lobby` (`crates/sim/src/setup.rs:209`), and the
checksum arm in `Sim::leader_long`
(`crates/sim/src/checksum/leaders.rs:949`); the lobby (`ai::Lobby`) holds
no per-player `handicap`, `team` or `diff` today, so the inputs would come
from the recorded-game slots (`rondata::recgame::Slot`).

### Plain game?

**Value in a default skirmish: 0** for every slot. With one person, `sum`
is that person's lobby handicap on their own slot and 0 elsewhere; the
person's slot gets `top − sum = 0`, and each computer slot gets
`min(20, person's handicap) / max(1, num_teams − 1)` — 0 when the person's
handicap byte is 0, which `Player::init (006ee180)` sets and every kept
dump scanned shows (see the scan note at the end). Whatever the value, no
solo reader consumes it: all three `get_handicap` callers are behind
`semaphore[0] & 4`. Frozen after `init_handicaps`.

### Confidence and not read

High for the writer list (displacement net and the leader-method store
scan agree; `->name` net only finds `log_data`). Medium-high for pass 1's
team arm: `get_team`'s own `team == 8` branch (it may answer another
slot's index when the player's team is 8 and `semaphore[0]` bit 7 is
clear) was read but not traced into `is_team`, and `is_cooperative` was
read only to its first tests (`team_style` 8, 9, 10). Not followed:
`get_handicap_level`'s callers; whether a solo lobby lets a person set a
non-zero handicap byte.

## `multi_diff` (+0x50, int)

**What it is.** A per-leader AI difficulty (0 Easiest … 5 Toughest, the
lobby's `DIFFICULTY` scale) for the games where difficulty can differ by
leader: multiplayer, and a scenario. Units: the difficulty index.

### Writers, both nets

- Export, `->multi_diff`: no writer (readers only: `Army::find_target`,
  `LeaderData::log_data`, and the three accessors). `field_0x50` on a
  leader: `Leader::Leader (006d2bf0)`, `Leader::set_diff (006d2470)`,
  `DropControl::process_drop (00959500)`, `ScenarioEditor::
  change_difficulty (0099bc00)`, `IFaceEditorNations::on_combobox_select
  (009897d0)`, `ScenarioRead::load_leader_chunk (009ac080)`, and
  `Game::init_handicaps (0058abf0)` (as `piVar10[1]`).
- Listing, `0xe3a3e0(%reg)` stores: `DropControl::process_drop` 0095967a
  (`movl $0x3`), `IFaceEditorNations::on_combobox_select` 009898bc,
  `ScenarioEditor::change_difficulty` 0099bc0d, `ScenarioRead::
  load_leader_chunk` 009ac5aa, `ScenarioFuncSet::set_leader_difficulty
  (009e5350)` 009e5386. `Game::init_handicaps` stores through `%esi`
  (`movl %eax,0x4(%esi)` at 0058ae49, `%esi` walking from `0xe3a3dc`).
- Register-relative `0x50(%reg)` stores on a leader base: `Leader::Leader`
  006d2c24 (`movl $0x2,0x50(%esi)`, `%esi` = `this`) and `Leader::set_diff`
  006d2474 (`%ecx` = `this`). The other scan hits are other records:
  `Leader::plan_strategy`'s 16-bit stores (`%si`/`%ax`, not an int),
  `Leader::compute_sites` 006cca28 (off `*(this+0x6e44)`),
  `Leader::gain_tech` (EH-prolog locals), `ScenarioRead::
  load_leaders_chunk` 009ace42 (`%edx = game`), `Army::find_target`
  006f8728 (`%ebx` = the `Army`).
- Bulk: `Leader::init`'s `tribe < 0` memset, `Leaders::clear`'s memset,
  save-load `walk_data`.

### Events, step, predicate

1. **Construction: 2.** `Leader::Leader (006d2bf0)` stores 2 after
   `LeaderData::LeaderData`. It has no direct caller in the listing (the
   static `leaders` array's construction); `Leaders::init`'s memset then
   zeroes it before any game, so the 2 never survives to a frame.
2. **Game start — `Game::init_handicaps (0058abf0)`, pass 3**, for each
   slot `i` in 0..7 with `leader_flags & 1` (listing 0058ae09..0058ae49):
   - if `Game::is_scenario (0043ec70)` (`semaphore[1] & 0x10` or
     `semaphore[2] & 2`) and **not** `semaphore[3] & 4`: unchanged;
   - else if `semaphore[0] & 4` (multiplayer) or `semaphore[3] & 4`:
     `aidiff[i]` (pass 1: the largest `Player.diff` among the slot's
     occupied non-person players) if `≥ 0`, else **2**;
   - else (solo random map): the lobby's `difficulty` (`movzbl
     0x2b(game)`, `GameInfo_u_24.difficulty`).
   `is_scenario` keeps `%ecx` (it is a leaf reading `0x821`/`0x822` off
   `%ecx`), so the following `0x823(%ecx)` / `0x820(%ecx)` tests are on the
   `Game`.
3. **A dropped player: 3.** `DropControl::process_drop (00959500)`, reason
   3, when the dropped player is not flagged `0x80` and no other occupied
   person (`flags & 0x100 == 0`, `flags & 0xd0 == 0`) shares the same
   `who`: the leader loses `leader_flags` bit 2 (becomes a computer) and
   `multi_diff = 3`. Callers `CommandPackage::process_ungraceful_player_drop`
   and `DropControl::process_drops` — multiplayer.
4. **Scenario and editor.** `ScenarioRead::load_leader_chunk` copies chunk
   `+0xe8` (one chunk-version arm, behind `semaphore[3] & 4 == 0`);
   `ScenarioEditor::change_difficulty` and `IFaceEditorNations::
   on_combobox_select` set the editor's choice; the trigger function
   `set_leader_difficulty` (`ScenarioFuncSet::init_funcs`, 2 args) stores
   its argument **as given, 1..6**, for an occupied slot with
   `leader_flags & 2` — unlike `set_difficulty (009e52f0)`, which stores
   its 1..6 minus 1 into the lobby byte, and like `get_leader_difficulty
   (009e53b0)`, which returns the word raw. (So a script's
   `set_leader_difficulty(p, 1)` reads back through `get_diff` as Easy,
   not Easiest; recorded, not judged.)
5. `Leader::set_diff (006d2470)`: `multi_diff = arg`. No direct call or
   jump to 006d2470 in the listing, no `vtables.txt` entry, and
   `rise_z.map` gives it its own address (not folded). Uncalled as far as
   these nets see.

### Reset

`Leader::init`'s and `Leaders::clear`'s memsets. No per-frame fold.

### Readers

The choice "which difficulty" — `LeaderData::get_diff (006ec000)`, read
from the listing: `multi_diff` (raw, even negative) when `semaphore[0] &
4`; else `multi_diff` only when `semaphore[1] & 0x10` (a scenario) and
not `semaphore[2] & 2` (not CtW) and `multi_diff ≥ 0`; else the lobby
`difficulty`. Called from 20 functions (`Leader::diplomacy` 13 times,
`Leader::create_units` 9, `Leader::create_buildings` 5, `research_techs`
and `upgrade_units` 4 each, `Object::take_damage` 2, …). The same test is
inlined (found by a `testb $0x2,0x822(..)` within six instructions of a
`0x50(%reg)` or `0xe3a3e0` load) in `Army::find_target` (12 sites),
`Leader::create_units` (4), `Leader::production_ai_setup` (3),
`Game::determine_end_of_game_stats_and_achievements` (2),
`Armies::find_local_army`, `Leader::research_techs`,
`Unit::find_repair_spot`, `Unit::think_spellcaster`, and inside
`get_gather_handicap` and `get_handicap`. Two inlined copies were re-read
(`create_units`, `determine_end_of_game_stats`); the rest are assumed the
same shape. `get_gather_handicap` alone takes `multi_diff` with no
fallback under `semaphore[0] & 4`. Also `ScenarioFuncSet::
get_leader_difficulty`, `ScenarioWrite::save_leaders_chunk`,
`IFaceEditorNations::update_visibility`, `LeaderData::log_data` (prints
`multi_diff`), `walk_data`.

### Crate

`crate::Sim::ai_difficulty` (`crates/sim/src/ai/ai_types.rs:137`) returns
`ai.lobby.difficulty`, which is `get_diff` for a solo random map — the
only arm a plain game reaches. `ai::army::target_gate::diff_of`
(`crates/sim/src/ai/army/target_gate.rs:132`) models the three-way choice,
with the live view answering `semaphore() = [0; 3]` and `lmulti() = -1`
(`target_gate.rs:397`, `:455`). No `multi_diff` is held: the checksum leaf
(`crates/sim/src/checksum/leaders.rs:55`) is held out
(`crates/rondata/src/diff/checksum_lines.rs:1815`) and the record diff
lists it as unmodelled (`crates/rondata/src/diff/leader.rs:58`). A builder
adds a `"multi_diff"` arm to `Sim::leader_long`
(`crates/sim/src/checksum/leaders.rs:949`): `lobby.difficulty` for each
seated slot 0..7.

### Plain game?

**Value in a default skirmish: the lobby `difficulty`** for every
occupied slot 0..7, the person's included — `init_handicaps`' solo arm
writes it without testing who is human; the crate's default `Lobby` has
`difficulty: 0` (`crates/sim/src/ai.rs`, `impl Default for Lobby`), so 0.
Slots 8 and 9 stay 0. The kept dumps agree: of 1,141 dumps that print a
leader record, every one prints `multi_diff` as only 0 and its own
`DIFFICULTY` (448 at 5, 135 at 4, 144 at 3, 35 at 2, 18 at 1, 344 at 0),
and run1313 (`DIFFICULTY 5`) prints 5 for `who 0` (the person) and `who 1`,
0 for `who 8` and `who 9`. No solo reader consumes it: with
`semaphore[0] & 4` and `semaphore[1] & 0x10` both clear, every `get_diff`
copy takes the lobby byte. Frozen after `init_handicaps`.

### Confidence and not read

High for the writer list (both nets, plus the `->name` net's
`field_0x50` hits, agree; `set_diff` and `Leader::Leader` have no direct
caller). High for the solo value (a dump-backed observation across 1,141
captures, though no diff in the crate pins it). Not read: what
`semaphore[3] & 4` (bit 26) is — it lets a scenario take the
per-player `diff` and lets `load_leader_chunk` and `load_leaders_chunk`
skip their stores; the meaning of `Player.flags` bits 4, 6, 7, 8 in
`process_drop`; and the 13 inlined `get_diff` copies not re-read.

