# lobby-words: second (blind) reading

Slug: `lobby-words`. Leaves: `cannon_time` (LeaderData +0x48, ulong), `handicap` (+0x4c, int), `multi_diff` (+0x50, int).
Reader: Claude Opus 5.5 (claude-opus-5-5), blind to the first reading.

Method: `->name` and `field_0x48/4c/50` over the export, then the plain
listing (`aa/exe.s`) for every absolute `0xe3a3d8`/`0xe3a3dc`/`0xe3a3e0`
displacement (slot-indexed stores into the global `leaders` array) and for
each candidate's own stores. `Leader` (0x6eec) holds `LeaderData` at +0
(the ctor `Leader::Leader (006d2bf0)` calls `LeaderData::LeaderData` on
`this` and writes its vbtable at +0x6ee4), so a `Leader*` offset equals the
`LeaderData` offset; no shift applies.

Shared context, re-derived here and used by all three leaves:

- **Setup order in a random-map game.** `Game::run (00584590)` calls
  `Game::init_rules_and_teams (00589bb0)` (call at 0x584885), which calls
  `Leaders::init (006ed850)` (0x589d04) and, near its end,
  `Game::init_teams` then `Game::init_handicaps (0058abf0)` (0x58a32d).
  Later `Game::init` → `Setup::build_game (005ac190)` (0x58d1b3) calls
  `Leader::init (006e3930)` once more per slot (0x5acef7), passing the
  slot's own `tribe` (`+0xc`).
- **`Leaders::init`** walks all ten slots (`0xe3a390`..`0xe7f8c8`, stride
  0x6eec) calling `Leader::init(slot, tribe = -1, slot)`.
- **`Leader::init`**: if `tribe < 0` (listing 0x6e3948 `test esi,esi; jns`)
  it `memset`s `[+0x0, +0x6929)` to zero (0x6e398c..0x6e3995) — which
  zeroes all three leaves — then, on both paths, stores `cannon_time`
  (0x6e3af3). With `tribe ≥ 0` (the `build_game` call for a seated slot)
  there is no clear, so `handicap` and `multi_diff` survive from
  `init_handicaps`.
- **`GameInfo` in `Game`** is at `Game+0xc`; its lobby bytes
  (`GameInfo_u_24_s_0`, 30 bytes) at `Game+0x24`, so `difficulty` is
  `Game+0x2b` and `cannon_times` `Game+0x33`; `player[8]` (stride 0x8c) at
  `Game+0x44`, so `player[0].flags` is `Game+0x74`. In `Player`: `flags`
  +0x30 (u16; bit 0 seated, bit 2 human — the bit `LeaderData::is_human
  (006ec170)` reads on `leader_flags`, and which `load_leader_chunk`
  mirrors between the two), `who` +0x33, `team` +0x34 (signed char),
  `handicap` +0x35, `diff` +0x38.
- **`Game::semaphore`** is a `BitMask<256>` at `Game+0x814`, bytes at
  `Game+0x820`. Bit 2 (`+0x820 & 4`) is the multiplayer bit (its gate on
  `issue_cannon_time` below; `docs/CENSUS.md` §385 names `run_gamespy` its
  one setter). Bit 26 is `+0x823 & 4`; I did not read what sets it.
- **Lobby defaults.** `GameInfo::default_setting (005d7200)` is a jump
  table over the 30 lobby bytes in struct order (table at 0x5d723c, read
  from the PE): index 7 `difficulty` → 0, index 15 `cannon_times` → 0.
  `PlayerProfile::default_profile (005ceca0)` fills the profile's
  `game_params` from it. A kept dump confirms the whole default row:
  `gamelog-run1-fulldump.txt`'s `GAME INFO` prints `DIFFICULTY 0`,
  `CANNON_TIMES 0` and every other byte equal to the table (TEAM_STYLE 1,
  MAP_STYLE 14, MAP_SIZE 2, …), the human's `PLAYER` block `flags 7 … team 8
  handicap 0 … diff 2`, the computer's `flags 1 … team 8 handicap 0 … diff 2`.

## Leaf 1: `cannon_time` (+0x48, ulong)

### 1. Writers

Export, `->cannon_time`: **no hit at all** (no reader prints the name).
`field_0x48` on a `Leader`/`leaders.list[]` base: `Leader::init (006e3930)`
and `TurnControl::start_cannon_time (00956500)`. Every other `field_0x48`
hit is on a different class (`Wall`, `City`, `Army`, `Group`, `Unit`,
`Build`, `Ammo`, `World`, `UnitBalance`); the `Leader::init` hit at
0x6e3f6a is `LeaderDataEncrypt+0x48` through the `+0x6eb8` pointer, not
ours.

Listing, absolute `0xe3a3d8(%reg)`: three sites —
0x9412f1 (`cmpl $0, …` — read, `CommandManager::issue_cannon_time
(00941260)`), 0x95658a (read) and **0x9565eb (store)**, both in
`start_cannon_time`, base `edi = who·0x6eec` (0x956584 `imul $0x6eec,%edx,%edi`).
Register-relative: **0x6e3af3** `mov %eax,0x48(%ebx)` in `Leader::init`,
`ebx = this` (`Leader*`, 0x6e3942 `mov %ecx,%ebx`).
Bulk: `LeaderData::walk_data (006d6750)` walks `[+0x8, +0x692a)` for a
seated slot; with `DataWalk::input != 0` (saved-game load) that restores
the leaf. `Leaders::init`'s path through `Leader::init` memsets it to 0
before the store.

| writer | store | value |
|---|---|---|
| `Leader::init (006e3930)` | 0x6e3af3 | `cannon_times.list[game.info.cannon_times].data[0]` |
| `TurnControl::start_cannon_time (00956500)` | 0x9565eb | `cannon_time − 1` |
| `LeaderData::walk_data (006d6750)` | bulk | the saved value (load only) |
| `Leader::init`, `tribe < 0` arm | memset | 0, immediately overwritten by the row above |

`Leader::Leader (006d2bf0)` does not touch +0x48.

### 2. Event, step, predicate

- **Seed.** `Leader::init` reads the lobby byte `Game+0x33`
  (`movzbl 0x33(%eax)`; `eax = GameAccess::game`, `0xc061ec`), multiplies
  by 0x58 (`sizeof(Category)`), adds it to `cannon_times.list.list`
  (`0xe800e0`) and loads `+0x3c`, `Category::data[0]`. Unconditional: both
  the `tribe < 0` and `tribe ≥ 0` paths reach it. `Categories::init
  (006f21d0)` stores each `<DATA>` through `String::convert_int` into
  `data[k]` (plain integer, no scaling). `Game::init_rules_data (0058ec60)`
  loads `cannon_times` with count 5; `data/rules.xml`'s `cannontimes`
  category gives the five rows `0, 1, 2, 4, 8` (None, 1, 2, 4, 8 Cannon
  Times). **Units: a count of uses.**
- **Step: −1 per use.** `start_cannon_time(who)` (from
  `CommandPackage::process_cannon_time (009464f0)`, from
  `CommandPackage::process`, i.e. command 0x4d off the order stream) steps
  it when **all** of: `who ≥ 0`; `turn_control.cannon_time_who < 0` (no
  cannon time already running; `cmpl $0,0x24(%esi)`); `cannon_time != 0`.
  On step it also sets `cannon_time_who = who`, `cannon_time_stamp =
  game.frame`, `cannon_saved_speed = speed`, `cannon_time_start =
  game.tick`, and if the speed is not 0 drops it to 0 with
  `set_new_target_time(200)`. With `cannon_time == 0` it only posts an
  "invalid order" feedback to the issuing console's player. No slot or kind
  is exempt beyond those three tests.
- **Issuer.** The one issuer, `CommandManager::issue_cannon_time
  (00941260)` (from `Options::exec`), builds command byte 0x4d only when
  `semaphore` bit 2 is set (`testb $4,0x820(%eax); jne`); otherwise it posts
  a message and returns. With bit 2 it also checks the local player's
  `cannon_time` as an **unsigned** `> 0` (`cmpl $0; ja`) before sending.
  No AI path calls it (no other caller of `issue_cannon_time` or
  `start_cannon_time` in the listing).

### 3. Reset

Every `Leader::init` re-seeds it from the lobby (`Leaders::init` and
`Setup::build_game` both, at game start; also each scenario/editor/CtW
leader build). Nothing in the frame loop resets it. `end_cannon_time` (from
`TurnControl::check_cannon_time`) clears `cannon_time_who`, not this leaf.

### 4. Readers

`start_cannon_time` (the zero test) and `issue_cannon_time` (the unsigned
local test). `LeaderData::log_data (006e5110)` does **not** print it. The
checksum walk reads it as part of `[+0x8, +0x692a)`.

### 5. Crate

No event: `crates/sim` has no cannon-time command, no `cannon_times`
lobby byte (`ai::Lobby` in `crates/sim/src/ai.rs` carries `difficulty` but
not `cannon_times`), and the leaf is listed only in
`crates/sim/src/checksum/leaders.rs` `LEADER_FIELDS`; `Sim::leader_long`
falls through to `None` for it. A builder would add a `cannon_times` byte
to `ai::Lobby` (parsed from the dump's `CANNON_TIMES`, rows `0,1,2,4,8`) and
return `cannon_times_row[lobby.cannon_times]` from `leader_long`'s match;
the decrement belongs with command 0x4d, which the crate does not port
(`crates/sim/src/sweep/` has `command_package_process_cannon_time` and
`turn_control_end_cannon_time` sweeps only).

### 6. Plain game

The seed is reached by every game. The step is **not**: its only issuer is
gated on `semaphore` bit 2 (multiplayer), and the AI never issues 0x4d. In
a solo skirmish the leaf is constant for the whole game at
`rows[CANNON_TIMES]`; with the default lobby byte 0 it is **0** for every
slot (seated or not). A solo player who picks "4 Cannon Times" in the lobby
(if the combo is offered in solo — not read) would hold 4, still never
stepped.

### 7. Confidence

High for the writers and the seed (listing-confirmed stores, rows from the
loader and the XML, default byte from the jump table and a dump). Not read:
whether the solo setup window shows the cannon-times combo; what else
might write `+0x48` through a `Leader*` held in a register I did not trace
(the register-relative net below +0x200 is noisy — I covered every
`Leader`/`LeaderData` method and every `leaders.list[...]`
`field_0x48` in the export, not every `0x48(%reg)` in `.text`).

## Leaf 2: `handicap` (+0x4c, int)

### 1. Writers

Export, `->handicap`: the `LeaderData` hits are all reads
(`get_handicap (006da740)`, `get_handicap_level (006d6740)`, `log_data
(006e5110)`); the `'\0'` stores in `Player`, `PlayerConnectionData`,
`ScenarioEditor::new_leader` are the **`Player`/`PlayerConnectionData`
`uchar handicap`** (a different struct, +0x35 / +0x38), not this leaf.
`field_0x4c` on a leaders base: `Game::init_handicaps (0058abf0)`
(`piVar10 = &leaders.list[0].field_0x4c`) and `ScenarioRead::load_leader_chunk
(009ac080)` (three sites).

Listing, absolute `0xe3a3dc`: `0x58ad86 mov $0xe3a3dc,%esi` (the walking
pointer of `init_handicaps`), stores `0x9ac1fc`, `0x9ac457`, `0x9ac643`
(`load_leader_chunk`, `mov %eax,0xe3a3dc(%edi)`, `edi = slot·0x6eec`), and
one read `0x9a7a80` (`ScenarioWrite::save_leaders_chunk`). The walk's
stores are **`0x58ade8 mov %eax,(%esi)`** and **`0x58ae07 mov
%eax,(%esi)`**, `esi` stepping 0x6eec from `0xe3a3dc` while `< 0xe71b3c`
(slots 0..7), guarded by `testb $1,-0x4c(%esi)` (`leader_flags & 1`).
A sweep of every function that loads a `$0xe3a3xx` immediate for a store to
`0x48/0x4c/0x50(%reg)`: the hits in `Leader::gain_tech` are `ebp`-based
locals (`ebx` is `this`), in `ScenarioRead::load_leaders_chunk` the base is
`Game` (`testb $4,0x823(%edx)` just above), in `UnitBalance::next` and the
`Unit`/`Wall`/`City`/`Army`/`Ammo`/`Object` methods another record. None is
this leaf.

| writer | store | value |
|---|---|---|
| `Game::init_handicaps (0058abf0)` | 0x58ade8, 0x58ae07 | the catch-up index below, then `/ max(num_teams − 1, 1)` |
| `ScenarioRead::load_leader_chunk (009ac080)` | 0x9ac1fc, 0x9ac457, 0x9ac643 | the scenario file's leader record `+0xe0` (after a `Leader::init`) |
| `Leader::init (006e3930)`, `tribe < 0` | memset | 0 |
| `LeaderData::walk_data (006d6750)` | bulk | saved value (load) |

### 2. Event, step, predicate

Set once, in `Game::init_handicaps`, called once per game build from
`Game::init_rules_and_teams` after `Game::init_teams`. Per seated slot `i`
(`leader_flags & 1`; slots 0..7), over the eight lobby `Player` records `p`:

1. **Own and AI pass**, every `p` with `flags & 1` and `p.who == i`:
   `own_i = max(own_i, p.handicap)` (start 0); and if `p` is not human
   (`flags & 4` clear) `aidiff_i = max(aidiff_i, p.diff)` (start −1; used
   by `multi_diff`, leaf 3).
2. **Sum start:** `sum_i = own_i` if `Game::is_cooperative (00594810)`,
   else 0.
3. **Human pass**, every `p` with `(flags & 5) == 5` (seated **and**
   human): if `p.who == i`: `own_i = max(own_i, p.handicap)`, `sum_i +=
   p.handicap`. Else if `p.team != LeaderData::get_team(i)` (the signed
   byte against `get_team (006ec040)`, which for a team-8 "no team" leader
   returns the lowest seated slot it is teamed with, normally itself):
   `enemy_i = max(enemy_i, p.handicap)` (start 0). Else (a teammate on
   another slot): `allies_i += 1` unless `p.team == 8`, and `sum_i +=
   p.handicap`.
4. `M = max over seated slots of sum_i` (the maxima of `own` and `enemy`
   are computed and discarded: `edi`/`esi` are reused at 0x58ad84/0x58ad86).
5. **Store**, per seated slot:
   - `allies_i == 0`: `h = clamp(M − sum_i, 0, 20)`;
   - else: `t = (M − sum_i) / (allies_i + 1)` (C division, numerator ≥ 0),
     `d = max(enemy_i − own_i, 0)`, `h = (t + d) / 2` (truncating; the
     `cltd; sub; sar` idiom), then `clamp(h, 0, 20)`.
   - `handicap = h / max(num_teams − 1, 1)` (`Game+0x6a4`, C division).

Units: an **index into the `handicaps` category**, 0..20. `Game::init_rules_data`
loads it with count 0x15 = 21; `data/rules.xml` rows are `0, 5, 10, …,
100`, i.e. row `k` holds `5k` (a percent). Exempt: unseated slots (left at
`Leaders::init`'s 0) and computer players' own `Player.handicap` bytes (only
human records enter step 3; step 1's `own` does take a computer record's
byte, but its `who == i` branch in step 3 is the only place `own` is
compared, so an AI byte matters only through `d`). Slots 8 and 9 are never
touched.

The effect is a catch-up: the slot (team) whose humans carry the largest
total handicap gets 0, everyone else gets the shortfall, capped at 20.

### 3. Reset

`Leaders::init` (memset to 0) at every game build, then `init_handicaps`
recomputes it. Never touched in the frame loop.

### 4. Readers

`LeaderData::get_handicap (006da740)`: `handicaps.rows[handicap]`, with a
difficulty-dependent index shift (+5/+10/+16) **only when `semaphore` bit 2
is clear**; but all three of its callers (`ObjectData::train_time
(006508c0)` twice, `World::compute_reg_territory (006b0bb0)`,
`LeaderData::get_gather_handicap (006d66a0)`) call it **only with bit 2
set**, so the shift arm is unreachable from them and the read is the plain
`5 × handicap` percent. (Were the `+16` arm reached, index up to 36 would
run past the 21 rows.) `compute_reg_territory` also requires the leader
human (`leader_flags & 4`). `get_handicap_level (006d6740)` has no caller
(`docs/CENSUS.md` §385 says the same). `log_data` prints it as `handicap`;
`save_leaders_chunk` writes it to a scenario.

### 5. Crate

No event. The crate has no lobby `Player` table with per-player handicap
bytes, no `num_teams`/cooperative flag feeding it, and no `init_handicaps`;
`crates/sim/src/economy/holdings.rs` (the `get_gather_handicap` port)
states that `semaphore[0] & 4` is not modelled and zeroes a human, and
`crates/sim/src/territory.rs` takes a `handicap_bonus` argument. The leaf
is listed only in `crates/sim/src/checksum/leaders.rs` `LEADER_FIELDS`
(`leader_long` → `None`). A builder would add the value at the game-setup
site that builds leaders (the same place `ai::Lobby` is filled), and return
it from `Sim::leader_long` in `checksum/leaders.rs`.

### 6. Plain game

`init_handicaps` runs in every random-map game, so the writer is reached.
**Value in a plain default skirmish: 0 for every slot.** The only human's
`Player.handicap` is 0 (`Player::init` zeroes it; the dump prints
`handicap 0` for both players), so every `sum_i` is 0, `M = 0`, every `h =
0`. Even a nonzero human handicap in solo would only seat a value here (the
computers would get `min(H, 20) / max(num_teams − 1, 1)`, the human 0) —
no consumer reads it without `semaphore` bit 2, so a solo game never feels
it. The scenario writer sits behind the scenario loader.

### 7. Confidence

High for the formula (read from the listing, every branch), the writers
(both nets plus the immediate sweep) and the plain-game value (arithmetic
on dump-confirmed inputs). Not diff-backed: no kept dump I checked prints
`LeaderData::log_data`'s `handicap` line (`gamelog-run1-fulldump.txt` has
only the `Player` block's), so the leader-side 0 is derived. Not read:
`Game::is_cooperative`'s full predicate (only its opening `team_style`
compares), what sets `Player.flags` bit 2 in the lobby (I took it from the
`load_leader_chunk` mirror and the dump's `flags 7`/`flags 1`), whether
the solo lobby offers a per-player handicap.

## Leaf 3: `multi_diff` (+0x50, int)

### 1. Writers

Export, `->multi_diff`: reads only (`get_diff (006ec000)`,
`get_gather_handicap (006d66a0)`, `get_handicap (006da740)`, `log_data
(006e5110)`, `Army::find_target (006f69b0)`). `field_0x50` on a leaders
base: `Leader::Leader (006d2bf0)`, `Leader::set_diff (006d2470)`,
`DropControl::process_drop (00959500)`, `IFaceEditorNations::on_combobox_select
(009897d0)`, `ScenarioRead::load_leader_chunk (009ac080)`,
`ScenarioEditor::change_difficulty (0099bc00)`; and `init_handicaps`'s
`piVar10[1]`. (`Leader::diplomacy`'s `local_28[-1].list[9].field_0x50` is a
pointer-arithmetic artefact on another array, and `ConsoleWin::run_cmd`'s
is `roads->field_0x50`.)

Listing, absolute `0xe3a3e0` stores: 0x95967a (`movl $3`, `process_drop`),
0x9898bc (`on_combobox_select`, `ecx = console_who·0x6eec`), 0x99bc0d
(`change_difficulty`), 0x9ac5aa (`load_leader_chunk`, from the record's
`+0xe8`, after `Leader::init`), 0x9e5386 (`ScenarioFuncSet::set_leader_difficulty
(009e5350)`). Register-relative: 0x6d2c24 `movl $2,0x50(%esi)` in
`Leader::Leader` (`esi = this`), 0x6d2474 `mov %eax,0x50(%ecx)` in
`set_diff` (`ecx = this`), **0x58ae49 `mov %eax,0x4(%esi)`** in
`init_handicaps` (`esi` = the `+0x4c` walking pointer, so `+0x50`).

| writer | store | value | reached from |
|---|---|---|---|
| `Leader::Leader (006d2bf0)` | 0x6d2c24 | 2 | `Leaders::Leaders (006ed900)` (the array's static construction; `push $0x6d2bf0` at 0x6ed91e) |
| `Leaders::init` → `Leader::init`, `tribe < 0` | memset | 0 | every game build |
| `Game::init_handicaps (0058abf0)` | 0x58ae49 | lobby `difficulty`, or the slot's max AI `Player.diff` (default 2) | every game build, see predicate |
| `DropControl::process_drop (00959500)` | 0x95967a | 3 | `CommandPackage::process_ungraceful_player_drop`, `DropControl::process_drops` |
| `ScenarioRead::load_leader_chunk (009ac080)` | 0x9ac5aa | scenario record `+0xe8` | scenario load |
| `ScenarioFuncSet::set_leader_difficulty (009e5350)` | 0x9e5386 | the script's argument, 1..6, stored unchanged | the trigger VM (`init_funcs` registers it) |
| `IFaceEditorNations::on_combobox_select (009897d0)` | 0x9898bc | combo index | scenario editor |
| `ScenarioEditor::change_difficulty (0099bc00)` | 0x99bc0d | argument | **no caller** in the listing |
| `Leader::set_diff (006d2470)` | 0x6d2474 | argument | **no caller, no pointer**: no `call`/`push` of 0x6d2470 and its VA bytes `70 24 6d 00` occur nowhere in the PE; `rise_z.map` gives the address one name (not folded) |
| `LeaderData::walk_data (006d6750)` | bulk | saved value | saved-game load |

### 2. Event, step, predicate

Set once per game build by `init_handicaps`, after `Leaders::init` zeroed
it (the static ctor's 2 never survives to a game). Per seated slot
(`leader_flags & 1`, slots 0..7), the tail of the store loop
(0x58ae09..0x58ae49):

- **Skipped** (left 0) when `Game::is_scenario (0043ec70)` — `semaphore`
  bit 12 (`+0x821 & 0x10`) or bit 17 (`+0x822 & 2`) — **and** `semaphore`
  bit 26 (`+0x823 & 4`) is clear.
- Otherwise, if `semaphore` bit 2 **or** bit 26 is set:
  `multi_diff = aidiff_i` (the max `Player.diff` over non-human lobby
  records with `who == i`, from leaf 2's step 1), or **2** when there is
  none (`cmovns`: −1 → 2).
- Otherwise (the solo random-map case): `multi_diff = game.info.difficulty`
  (`movzbl 0x2b(%eax)`), the lobby byte, for **every seated slot, human or
  computer**.

Units: a difficulty level, 0 Easiest … 5 Toughest (`data/rules.xml`
`difficulties`, six rows, no `<DATA>`; the index is the value). Exempt:
slots 8, 9 and unseated slots. Later steps, none per frame:
`process_drop` sets 3 (Tough) on the leader of a dropped network player
when no other connected player controls it, while clearing the human bit
(`leader_flags & ~4`, `Player.flags & 0xffeb`); the trigger function
stores its 1..6 argument as given (so a script's "1" lands as Easy, and 6
is out of the 0..5 range — no clamp; I did not read whether the trigger
layer documents 1-based input); the scenario/editor writers store what the
file or combo carries.

### 3. Reset

`Leaders::init`'s memset at every game build, then `init_handicaps` (or a
scenario's `load_leader_chunk`). Nothing in the frame loop.

### 4. Readers

`LeaderData::get_diff (006ec000)` — 56 direct `call`s (AI production,
diplomacy, armies, `Object::take_damage`, `Unit::do_gather`, …) plus
inlined copies read through `0xe3a3e0(%reg)` (`Game::determine_end_of_game_stats_and_achievements`,
`Unit::think_spellcaster`, `Unit::find_repair_spot`, `Army::find_target`)
and `0x50(this)` (`Leader::create_units`, `production_ai_setup`,
`research_techs`), and the copies inside `get_handicap` and
`get_gather_handicap`. The selector, in every copy: **`semaphore` bit 2 →
`multi_diff`; else `multi_diff` only if bit 12 is set, bit 17 clear and
`multi_diff ≥ 0`; else the lobby `difficulty`.** So in a solo random-map
game (bits 2, 12, 17 clear) no AI decision reads this leaf. Also
`log_data` (`multi_diff`), `save_leaders_chunk`,
`ScenarioFuncSet::get_leader_difficulty`.

### 5. Crate

The value exists: `ai::Lobby::difficulty` (`crates/sim/src/ai.rs`),
read by `Sim::ai_difficulty` (`crates/sim/src/ai/ai_types.rs`), which is
the crate's `get_diff` for the solo arm. The leaf itself is only listed in
`crates/sim/src/checksum/leaders.rs` `LEADER_FIELDS`; `Sim::leader_long`
returns `None`. A builder adds `"multi_diff" => one(i64::from(lobby.difficulty))`
to `leader_long`'s match (the walk already skips unseated slots via
`leader_seats()`, matching `walk_data`'s `leader_flags & 1` gate). No
scenario, drop, editor or trigger path exists in the crate; the
`semaphore` bit-2/26 arm (per-slot AI `diff`) would need the lobby
`Player.diff` bytes, which the crate does not carry.

### 6. Plain game

Reached: `init_handicaps`'s solo arm runs in every random-map skirmish.
**Value in a plain default skirmish: the lobby `DIFFICULTY` byte for every
seated slot — 0 (Easiest) by `default_setting`, and 0 in every kept dump
`docs/AI.md` §80.1 surveyed (`DIFFICULTY 0`).** It is never read for a
decision in that game. The drop writer is multiplayer; the scenario,
editor and trigger writers sit behind the scenario/trigger layer;
`set_diff` and `change_difficulty` are dead.

### 7. Confidence

High: every store confirmed in the listing with its base register typed;
the predicate read from both `init_handicaps`'s listing and `get_diff`.
Not diff-backed on the leader side (no kept dump I opened prints
`log_data`'s `multi_diff`; the inputs are dump-confirmed). Not read: what
sets `semaphore` bits 12, 17 and 26 (I know only that `is_scenario` is
12-or-17 and that `load_leaders_chunk` tests 26 beside bit 11); whether a
lobby choice other than Easiest changes `GameInfo::load_from_config`'s byte
for a plain launch.

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `cannon_time` +0x48 | `Leader::init (006e3930)` seed; `TurnControl::start_cannon_time (00956500)` −1; `LeaderData::walk_data (006d6750)` load | seed = `cannontimes` row `[0,1,2,4,8][lobby CANNON_TIMES]`; −1 per command 0x4d when no cannon time is running and the count ≠ 0 | every `Leader::init` (game build, scenario, editor, CtW) | none; would be a `cannon_times` byte on `ai::Lobby` and a `leader_long` arm in `checksum/leaders.rs` | seed yes, value **0** at default; the step is multiplayer-only (`issue_cannon_time (00941260)` gated on `semaphore` bit 2), never in solo | high |
| `handicap` +0x4c | `Game::init_handicaps (0058abf0)`; `ScenarioRead::load_leader_chunk (009ac080)`; memset in `Leader::init`; `walk_data` load | once per build: catch-up index `clamp(M − sum_i, 0, 20)` (or the ally-split form) `/ max(num_teams−1, 1)` over human lobby handicaps | `Leaders::init` memset each build | none; `leader_long` arm; consumers ported without it (`economy/holdings.rs`, `territory.rs`) | yes, value **0** at default; consumers (`get_handicap`) only under `semaphore` bit 2 | high (derived, not diffed) |
| `multi_diff` +0x50 | `Leader::Leader (006d2bf0)` =2 (static, overwritten); `init_handicaps` (lobby difficulty / AI `diff`); `DropControl::process_drop (00959500)` =3; `load_leader_chunk`; `ScenarioFuncSet::set_leader_difficulty (009e5350)`; `IFaceEditorNations::on_combobox_select (009897d0)`; dead: `Leader::set_diff (006d2470)`, `ScenarioEditor::change_difficulty (0099bc00)` | once per build = lobby `DIFFICULTY` (solo); later only by drop/scenario/trigger/editor | `Leaders::init` memset each build | `ai::Lobby::difficulty` exists; add `leader_long` arm in `checksum/leaders.rs` | yes, value **lobby DIFFICULTY = 0** at default for seated slots; unread by `get_diff` in solo | high (derived, not diffed) |
