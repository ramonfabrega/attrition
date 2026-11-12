## 16. `cannon_time`, `handicap` and `multi_diff`

Three words a game build writes from the lobby and that nothing moves in a
plain game. None of them is 0 by rule: `multi_diff` holds the lobby's
difficulty byte.

### 16.1 Leaves

- `cannon_time`, +0x48, `ulong`, a count of uses. Slot `i` at
  `0xe3a3d8 + i·0x6eec`.
- `handicap`, +0x4c, `int`, a row 0..20. Slot `i` at `0xe3a3dc + i·0x6eec`.
- `multi_diff`, +0x50, `int`, a difficulty level. Slot `i` at
  `0xe3a3e0 + i·0x6eec`.

### 16.2 The rule

**`multi_diff`.**

- **Writer in play: `Game::init_handicaps (0058abf0)`**, once per game
  build, after the leaders are cleared, for each seated slot 0..7
  (`leader_flags & 1`). Human and computer slots alike: no test of the
  human bit sits on the path. Slots 8, 9 and unseated slots keep the
  memset's 0. The predicate, in order:
  1. a scenario (`Game::is_scenario (0043ec70)`) with `semaphore` bit 26
     clear: no store;
  2. bit 2 (multiplayer) or bit 26: `aidiff[i]` if non-negative, else 2.
     `aidiff[i]` is the largest `Player.diff` over occupied computer
     `Player` records with `who == i`, starting from −1;
  3. otherwise: the lobby's `difficulty` byte (`Game+0x2b`, that is `info`
     +0xc, the lobby union +0x18, `difficulty` +0x7).
- **Bit 26** (`Game+0x823 & 4`) means "a scenario chosen from the skirmish
  setup window". Its one setter is `SetupWin::setup_game (005b3860)`, when
  the setup combo 0x36 picks entry 10. `Game::run_scenario (005860c0)` and
  `Game::run_playback (00586ea0)` clear it. A random-map skirmish never sets
  it.
- **Writers off the plain path:** `DropControl::process_drop (00959500)`
  stores 3 when a dropped network player's slot has no other occupied
  `Player` record (any kind, person or computer, keeps it). Multiplayer
  only. `ScenarioRead::load_leader_chunk (009ac080)` copies the chunk's
  +0xe8 after a `Leader::init`. `ScenarioFuncSet::set_leader_difficulty
  (009e5350)` stores its 1..6 argument raw, on an occupied slot with
  `leader_flags & 2`. The editor stores in `IFaceEditorNations::
  on_combobox_select (009897d0)` and `ScenarioEditor::change_difficulty
  (0099bc00)` are editor only.
- **Dead or overwritten:** `Leader::set_diff (006d2470)` has no reference
  at all. `Leader::Leader (006d2bf0)` stores 2, which the memset at
  `Leaders::init` erases.
- **Step and reset:** none per frame. Reset at the next game build. The
  bulk writers apply as in the header.
- **Reader:** `LeaderData::get_diff (006ec000)` and its inlined copies, and
  `get_gather_handicap (006d66a0)`. Under bit 2, `get_diff` returns the
  leaf raw. Otherwise it returns the leaf only under `0x821 & 0x10`,
  without `0x822 & 2`, with the leaf `>= 0`, and else the lobby byte. In a
  solo random map both arms give the same byte, so the leaf never decides
  anything there. It still holds that byte.

**Port:** each seated slot 0..7 gets `multi_diff = lobby.difficulty` (0..5,
as stored). A builder must not emit 0.

**`cannon_time`.**

- **Seed: `Leader::init (006e3930)`**, on every call and both tribe arms.
  The negative-tribe test skips only the memset. The value is
  `CANNON_TIMES_ROW[lobby.cannon_times]` (`Game+0x33`, rows of stride
  0x58, field +0x3c), with the shipped `rules.xml` `cannontimes` rows
  `[0, 1, 2, 4, 8]`. The seed is not gated on seating, so it covers slots
  0..9.
- **Step: `TurnControl::start_cannon_time (00956500)`**, on an applied
  command 0x4d. Exemptions, all must pass: the sender `who >= 0`; nobody is
  already in cannon time (`cannon_time_who < 0`); `cannon_time != 0`. Then
  `cannon_time -= 1`. It also stamps the cannon-time `who`, frame, saved
  speed and tick. A refusal posts feedback only when `who` is the console's.
  `start_cannon_time` and `process_cannon_time (009464f0)` test no
  semaphore.
- **The only issuer**, `CommandManager::issue_cannon_time (00941260)`,
  refuses without `semaphore[0] & 4` (multiplayer). It then sends 0x4d only
  if the console leader's `cannon_time` is non-zero (unsigned `> 0`).
- **Not writers:** `Leader::reset_score (006e37f0)`. The store at 006e3f6a
  in `Leader::init` goes through the +0x6eb8 `LeaderDataEncrypt*`, not this
  leaf.
- **Reset:** the next game build. The bulk writers apply as in the header.
- **Readers:** the two guards above, plus `IFaceMainBase::update_visibility
  (00811d40)`, which greys the button (UI only). `log_data` does not print
  it.

**`handicap`.** Offsets in this rule are `Player`'s: `flags` +0x30, `who`
+0x33, `team` +0x34 (signed), `handicap` +0x35, `diff` +0x38.

- **Writer in play: `Game::init_handicaps (0058abf0)`**, the same call and
  seating loop as `multi_diff`, over the eight lobby `Player` records.
  1. **Pass 0:** `own[i]` is the largest `handicap` over occupied records
     with `who == i`, of any kind, starting from 0.
  2. `sum[i] = own[i]` if `Game::is_cooperative (00594810)`, else 0.
  3. **Human pass**, over records with `(flags & 5) == 5`, sender `h` its
     handicap, slot `w` its `who`, for each seated `i`:
     - `w == i`: `own[i] = max(own[i], h)`, `sum[i] += h`;
     - `w != i` and `team != get_team(i)`: `enemy[i] = max(enemy[i], h)`;
     - `w != i`, same team: `mates[i] += 1` unless `team == 8`, and
       `sum[i] += h`.
  4. `top = max(0, max over seated i of sum[i])`. The maxima of `own` and
     `enemy` are also computed and discarded.
  5. The store, every division signed and truncating toward zero:

         if mates[i] == 0: h = clamp(top − sum[i], 0, 20)
         else:             h = clamp(((top − sum[i]) / (mates[i] + 1)
                                      + max(0, enemy[i] − own[i])) / 2, 0, 20)
         handicap[i] = h / max(1, num_teams − 1)      // num_teams: Game+0x6a4

  Slots 8, 9 and unseated slots: 0. A computer record's byte enters
  through `own` (and so `enemy − own`) and, under `is_cooperative`, the
  seed of `sum`. `Player::init` zeroes every byte, so this is moot in a
  plain game.
- **Off the plain path:** `ScenarioRead::load_leader_chunk (009ac080)`
  stores the chunk's +0xe0 at three sites, each behind `semaphore[3] & 4`
  clear (bit 26).
- **Step and reset:** none per frame. Reset at the next game build.
- **Readers:** `LeaderData::get_handicap (006da740)`, called from
  `ObjectData::train_time (006508c0)` (two sites), `World::
  compute_reg_territory (006b0bb0)` and `get_gather_handicap`'s tail, every
  caller behind `semaphore[0] & 4`. `get_handicap`'s difficulty shift
  (+5/+10/+16) runs only with that bit clear, so no caller reaches it.
  `get_handicap_level (006d6740)` has no caller.

### 16.3 In the crate

- **No leaf is held.** All three are in `LEADER_FIELDS`
  (`crates/sim/src/checksum/leaders.rs:53-55`), and `Sim::leader_long`
  (`:949`) has no arm for them. Each is a `(unheld)` entry of
  `LEADERS_HELD_OUT` with count 2: `cannon_time` at
  `crates/rondata/src/diff/checksum_lines.rs:1771`, `handicap` at `:1793`,
  `multi_diff` at `:1815`. Each arm below removes its entry.
- **`multi_diff`:** add an arm returning `i64::from(self.ai.lobby.difficulty)`
  for each seated slot. The walk already skips unseated slots.
  `Sim::ai_difficulty` (`crates/sim/src/ai/ai_types.rs:137`) is `get_diff`'s
  solo arm, and `diff_of` (`crates/sim/src/ai/army/target_gate.rs:133`)
  uses it.
- **Trap: the live `lmulti` answers −1** (`target_gate.rs:397`). The
  original holds the lobby byte. Nothing reads it while `semaphore()`
  answers `[0; 3]`, but the port points `lmulti` at the same value as the
  checksum arm so the two cannot drift.
- **Trap: the default lobby is not the answer.** `impl Default for Lobby`
  (`crates/sim/src/ai.rs:1113-1119`) sets `difficulty: 0`, but the harness
  fills the lobby per capture. The pinned pairs carry 5
  (`crates/rondata/src/diff/persian_classical_pair.rs:24`), 4
  (`french_medieval_pair.rs:23`) and 2 (`aztec_word.rs:22`). A constant 0
  is wrong on all of them. `crates/rondata/src/diff/leader.rs:58` lists the
  leaf as unmodelled ("an input, not state").
- **`cannon_time`:** add `cannon_times` to `ai::Lobby`, parsed from the
  dump's `CANNON_TIMES`, and an arm returning the row's count.
  `Command::CannonTime` is decoded (`crates/rondata/src/commands.rs:996`)
  and consumed by nothing. The step needs a 0x4d arm only for a
  multiplayer stream.
- **`handicap`:** an arm returning 0 for every seated slot is right only
  while the lobby carries no per-player handicap. The full computation needs
  each `Player`'s `team`, `handicap` and `flags`, and `num_teams`, which
  `rondata::recgame::Slot` carries and `ai::Lobby` does not.
  `Holdings::handicap` (`crates/rondata/src/diff/leader.rs:55`) is a
  different quantity.

### 16.4 Reached by a plain game

- `multi_diff`: every random-map game reaches the store. It is frozen at
  the lobby byte and decides nothing.
- `cannon_time`: the seed is reached, and it is 0 under the default
  `CANNON_TIMES 0`. The step never fires solo: the issuer needs the
  multiplayer bit.
- `handicap`: reached. With one person at handicap 0, every `sum` and `top`
  is 0, so every slot is 0. No reader consumes it without the multiplayer
  bit.

Behind multiplayer: the `aidiff` arm, `process_drop`, the cannon-time step,
and every `handicap` reader. Behind triggers: `set_leader_difficulty`.
Behind scenarios and the editor: `load_leader_chunk`, bit 26, and the
editor stores. `Leaders::clear` runs at game end.

### 16.5 How established, and how confident

Two readings and an adjudication
(`docs/audit/2026-11-11-item-4152/lobby-words-*.md`); no diff backs it yet.

Verdicts (confirmed / corrected / open): `multi_diff` 12 / 2 / 0,
`cannon_time` 12 / 0 / 0, `handicap` 14 / 1 / 0. The corrections that
changed a rule:

- the plain-game `multi_diff` is `lobby.difficulty`, not 0;
- `process_drop` keeps a leader human while any other occupied record
  holds the slot, person or computer (it tests no human bit);
- a computer record's `handicap` byte also seeds `sum` under
  `is_cooperative`.

The bit-26 setter and clears are new, beyond both readings.

The writer sets rest on an absolute-address net and a walking-pointer
sweep of the listing for each leaf; both match the readings' union.
Confidence is high on all three: every store, predicate and branch of the
handicap formula was read in the listing, and the offsets come from
`types.txt`.

**Witness, not a pinned diff.** Of 1,194 kept `gamelog-*.txt`, 949 print
`multi_diff` within their first 20 MB. In each, the first eight
`multi_diff` lines carry only 0 and that dump's own first `DIFFICULTY`
value: 297 / 13 / 25 / 121 / 107 / 385 at `DIFFICULTY` 0..5. One dump has
no `DIFFICULTY` line and prints 0 and 5. No dump shows a seated slot at 0
under a non-zero lobby. `CANNON_TIMES 0` appears in 1,193 of the 1,194
dumps, and every leader `handicap` line reads 0.

### 16.6 Not established

None of these changes a plain game's value.

- **Whether the solo setup window offers a non-zero `CANNON_TIMES` or a
  per-player handicap.** Either would change the frozen value, not the
  rule. A capture of a skirmish set that way settles it.
- **`is_cooperative`'s full predicate and `get_team`'s team-8 branch.**
  Both matter only with more than one person. A reading of the two
  functions settles them.
- **`ScenarioEditor::change_difficulty`'s callers** and **the 13 inlined
  `get_diff` copies** were not re-read. Neither changes the plain-game
  value.
- **`LeaderData::walk_data` on a save load** was not re-read for these
  leaves. It belongs to the save track.
- **The dump observation becomes a diff** once the three arms land: the
  `rondata::diff` leaders line then compares them on every capture.
