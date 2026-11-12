# Item 4152, family 16 (`lobby-words`): adjudication

- Slug: `lobby-words`. Leaves: `cannon_time` (`LeaderData`+0x48, ulong),
  `handicap` (+0x4c, int), `multi_diff` (+0x50, int).
- Readings: `lobby-words-reading.md` (first), `lobby-words-second.md`
  (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text`
  listing (`exe.s` in the job scratch), `crates/sim` grep, and the kept
  dumps' `GAME INFO` lines. Nothing run of the original.
- The spawn's question: the readings were summarised as differing on
  `multi_diff`'s plain-game value (the lobby difficulty for every seated
  slot, against 0). Settled first, below, then leaf by leaf.

(Appended per leaf as each is settled.)

## Leaf `multi_diff` (+0x50, int), settled first

### The headline question: the plain-game value

**There is no disagreement on the rule, only on the number quoted for
it.** Both readings give the same store: in a solo random-map game
`Game::init_handicaps (0058abf0)` writes the lobby's `difficulty` byte
into `multi_diff` for every seated slot 0..7, human or computer. The
first quotes it as "the lobby difficulty (0 under the crate's default
`Lobby`)"; the second as "the lobby `DIFFICULTY` byte ... 0 (Easiest) by
`default_setting`, and 0 in every kept dump `docs/AI.md` §80.1
surveyed". The README's one-line summary of the second ("**0**") drops
the "lobby byte" half of what the second actually wrote.

- **What `init_handicaps` stores (confirmed, listing).** The store loop's
  tail, per seated slot (`testb $0x1,-0x4c(%esi)` at 0058ad90, `%esi`
  walking `0xe3a3dc` by 0x6eec to `< 0xe71b3c`, so slots 0..7):
  `Game::is_scenario (0043ec70)` at 0058ae09 (it reads only
  `0x821`/`0x822` off `%ecx` and preserves it; `%ecx` was reloaded with
  `this`, the `Game`, from `-0x20(%ebp)` at 0058ae04 — stored there at
  0058abfd); scenario and `0x823 & 4` clear → skip to 0058ae4c (no
  store); else `0x820 & 4` or `0x823 & 4` → 0058ae38: `aidiff[i]`
  (`-0xc4(%ebp,%edi)`) if non-negative (`cmovns`), else 2; else
  0058ae2d..0058ae32: `movzbl 0x2b(GameAccess::game)`, and
  **the deciding line is the store `movl %eax,0x4(%esi)` at 0058ae49**.
  `Game+0x2b` is `info` (+0xc) + the lobby union (+0x18) + `difficulty`
  (+0x7) by `types.txt` (`Game` 44556, `GameInfo` 44636,
  `GameInfo_u_24_s_0` 44655). No test of the human bit sits on that
  path, so the person's own slot gets the same byte.
- **What `get_diff` reads (confirmed).** `LeaderData::get_diff
  (006ec000)`, listing 006ec000..006ec039: bit 2 → `multi_diff` raw;
  else `multi_diff` only under `0x821 & 0x10`, not `0x822 & 2`, and
  `multi_diff >= 0`; else `movzbl 0x2b` (006ec031). In a solo random map
  both arms return the same byte, so the leaf is never the deciding
  input there, but it holds that byte, not 0.
- **The crate's default `Lobby` difficulty (confirmed 0, and
  irrelevant to the port).** `impl Default for Lobby`
  (`crates/sim/src/ai.rs:1113..1119`) sets `difficulty: 0`, documented as
  run6/run7's lobby. But the harness fills the lobby per capture: the
  pinned pairs in `crates/rondata/src/diff/*_pair.rs` set `difficulty: 5`
  (e.g. `persian_classical_pair.rs:24`), 4 (`french_medieval_pair.rs:23`)
  and 2 (`aztec_word.rs:22`). A constant 0 would be wrong for every one
  of those games.
- **The kept dumps (corrected: the second's "0 in every kept dump" is
  stale).** `docs/AI.md` §80.1's "no capture on disk has ever read
  anything else" was true before the §80.4 captures; it is not now. A
  fresh tally of the first `DIFFICULTY` line of each of the 1,194
  `gamelog-*.txt` in the oracle's `Logs` (2026-11-11): 385 at 0, 18 at 1,
  37 at 2, 157 at 3, 135 at 4, 461 at 5, one with none. The first
  reading's own tally (344 / 18 / 35 / 144 / 135 / 448) differs by the
  captures landed since, not in kind.

**Verdict.** Confirmed for both readings' rule; **corrected** for the
number: the plain-game value is `lobby.difficulty` per seated slot, which
is 0 only in a lobby left at the default. A builder must not emit 0.

### 1. Claim by claim

| # | claim | first | second | mark | verdict, deciding line |
|---|---|---|---|---|---|
| 1 | `Game::init_handicaps (0058abf0)` writes it once per game build, slots 0..7 seated | yes | yes | agree | confirmed, 0058ae49 / loop bound 0058ae5a |
| 2 | predicate: scenario and not bit 26 → untouched; bit 2 or bit 26 → `aidiff[i]` else 2; else lobby `difficulty` | yes | yes | agree | confirmed, 0058ae09..0058ae46 |
| 3 | `aidiff[i]` = max `Player.diff` over occupied non-human records with `who == i`, floor −1 | yes | yes | agree | confirmed, 0058ac46..0058ac51 (`testb $4,%dl; jne` skips the diff max for a person), start `orl $-1,%ebx` 0058ac26 |
| 4 | `Leader::Leader (006d2bf0)` stores 2; never survives (memset at `Leaders::init`) | yes ("no direct caller") | yes (reached from `Leaders::Leaders (006ed900)`, push at 006ed91e) | agree; the second's caller is one-sided | confirmed both: 006d2c24; `pushl $0x6d2bf0` at 006ed91e is the only reference (vector-construction helper) |
| 5 | `Leader::set_diff (006d2470)` is dead | no caller found | no call, no push, VA bytes absent from the PE | agree | confirmed: no listing reference; `70 24 6d 00` occurs 0 times in `riseofnations.exe` |
| 6 | `DropControl::process_drop (00959500)` stores 3, reason 3, when no other player record holds the slot | "no other occupied person" | "no other connected player controls it" | disagree in wording | **corrected (first)**: the loop at `process_drop` decompile lines 50..55 tests `flags & 1`, `flags & 0x100 == 0`, `flags & 0xd0 == 0` and `who` equal, and never the human bit 4: any other occupied record, person or computer, keeps the leader human. Store 0095967a. Multiplayer only. |
| 7 | `ScenarioRead::load_leader_chunk (009ac080)` copies chunk `+0xe8` after a `Leader::init` | yes | yes | agree | confirmed, 009ac5a1..009ac5aa |
| 8 | `ScenarioFuncSet::set_leader_difficulty (009e5350)` stores its 1..6 argument unchanged, occupied slot with `leader_flags & 2`; `set_difficulty (009e52f0)` stores arg−1 into the lobby byte; `get_leader_difficulty (009e53b0)` returns raw | yes | yes (6 out of range noted) | agree | confirmed, 009e535e..009e5386, 009e5302..009e530a, 009e53d6 |
| 9 | `IFaceEditorNations::on_combobox_select (009897d0)` 009898bc and `ScenarioEditor::change_difficulty (0099bc00)` 0099bc0d store; editor only | yes | yes, and `change_difficulty` has no caller | agree; caller claim one-sided | stores confirmed; `change_difficulty`'s callers not re-checked (editor, cut from v1) |
| 10 | memsets: `Leader::init (006e3930)` `tribe < 0` arm, `Leaders::clear (006ed5d0)` | both | `Leaders::init` path only | one-sided (`Leaders::clear`) | confirmed by the first's citation; it is the game's end, not a start |
| 11 | `walk_data (006d6750)` restores it on a save load | yes | yes | agree | not re-read (save load is out of a plain game) |
| 12 | readers: `get_diff` and its inlined copies, the selector shape | yes | yes | agree | confirmed for `get_diff` (006ec000..006ec039) and `get_gather_handicap (006d66a0)` (decompile lines 19..27) |
| 13 | crate: `Sim::ai_difficulty` (`crates/sim/src/ai/ai_types.rs:137`) is `get_diff`'s solo arm; `diff_of` (`ai/army/target_gate.rs:133`); `leader_long` (`checksum/leaders.rs:949`) has no arm; held out at `checksum_lines.rs:1815`; `leader.rs:58` lists it unmodelled | yes | yes (fewer sites) | agree | confirmed, all lines as cited |
| 14 | plain-game value | lobby difficulty (0 by default) | lobby byte (0 by default, "0 in every kept dump") | agree on the rule | rule confirmed; the second's dump claim **corrected** (above) |

**Writer completeness.** The listing's absolute net (`0xe3a3e0` stores:
0095967a, 009898bc, 0099bc0d, 009ac5aa, 009e5386) and a sweep of every
`movl $0xe3a3xx,%reg` walking pointer for a store landing on +0x50 (only
0058ae49) match the two readings' union exactly. No new writer.

One finding neither reading made: **what `semaphore` bit 26
(`0x823 & 4`) is.** Its one setter is `SetupWin::setup_game (005b3860)`
(`orb $0x4,0x823(%edx)` at 005b4389), taken when the setup window's combo
`0x36` has entry 10 selected (decompile lines 542..545), the arm that
also sets the scenario path. So bit 26 is "a scenario chosen from the
skirmish setup window", under which `init_handicaps` takes the per-player
`diff` combos even for a scenario. Its clears are 005861f8
(`Game::run_scenario (005860c0)`) and 00586fb5 (`Game::run_playback
(00586ea0)`). A random-map skirmish never selects entry 10, so the bit
is clear in a plain game.

The crate's live `lmulti` (`target_gate.rs:397`) answers −1, which is
not the original's value (the original holds `lobby.difficulty`); it is
unread while `semaphore()` answers `[0; 3]`, so it changes nothing, but a
builder porting this leaf should make `lmulti` return the same value as
the checksum arm so the two cannot drift.

### 2. The rule as a builder should port it

- **Writer (plain game):** at game setup, after the leaders are cleared,
  for each seated slot 0..7: `multi_diff = lobby.difficulty` (0..5, the
  lobby byte as stored). Unseated slots and slots 8, 9: 0.
- **Out of a plain game:** bit 2 (multiplayer) or bit 26 (scenario from
  setup): the slot's largest computer `Player.diff`, or 2 if it has no
  computer record; a scenario without bit 26: whatever the scenario's
  leader chunk carries (`+0xe8`); a dropped network player whose slot has
  no other live record: 3; the trigger function: its argument 1..6 raw.
- **Step / reset:** none per frame. Reset only at the next game build.
- **Crate site:** a `"multi_diff"` arm in `Sim::leader_long`
  (`crates/sim/src/checksum/leaders.rs:949`) returning
  `i64::from(self.ai.lobby.difficulty)` for each seated slot; the walk
  already skips unseated slots. Point `target_gate`'s live `lmulti` at
  the same value.
- **Plain game:** reached by every random-map game; frozen at the lobby
  byte; never the deciding input of any reader.
- **Confidence:** high (listing for writer and predicate, `types.txt` for
  the offset, the harness's own per-capture lobbies, and 949 dumps that
  agree; section 3).
- **Open:** none for a plain game. `change_difficulty`'s callers and the
  13 inlined `get_diff` copies were not re-read here (neither changes the
  plain-game value).

### 3. The dump cross-check (an observation, not a diff)

Re-run here independently of the first reading (2026-11-11): of the
1,194 kept `gamelog-*.txt`, 949 print `multi_diff` within their first
20 MB. In each, the first eight `multi_diff` lines carry only 0 and that
dump's own first `DIFFICULTY` value: 297 at `DIFFICULTY 0` (0 only), 13
at 1 (0, 1), 25 at 2 (0, 2), 121 at 3 (0, 3), 107 at 4 (0, 4), 385 at 5
(0, 5), and the one dump without a `DIFFICULTY` line prints 0 and 5. The
0s fit the unseated slots, which keep the memset's 0. No dump shows a
seated slot at 0 under a non-zero lobby, which is the check that would
have killed "the lobby byte". Nothing in `rondata::diff` compares this
leaf yet; the arm above, once added, turns this observation into a diff.

## Leaf `cannon_time` (+0x48, ulong)

### 1. Claim by claim

| # | claim | first | second | mark | verdict, deciding line |
|---|---|---|---|---|---|
| 1 | `Leader` holds `LeaderData` at +0, so a `Leader*`'s +0x48 is the leaf; slot i's leaf is `0xe3a3d8 + i·0x6eec` | yes (but its file header says "a `Leader*` reaches a leaf at off+8") | yes | agree; the first contradicts itself | confirmed +0: `Leader::init`'s seed stores `0x48(%ebx)` with `%ebx = this` (006e3942, 006e3af3), and the absolute sites use `0xe3a3d8`; the first's header line is a slip its own body overrides |
| 2 | seed: `Leader::init (006e3930)` stores `cannon_times[game.info.cannon_times].data[0]` on every call, both tribe arms | yes | yes | agree | confirmed: `jns 0x6e3a36` at 006e394a skips only the memset; 006e3a36..006e3af3 has no branch; `movzbl 0x33(game)`, `imul $0x58`, `+0x3c`, store 006e3af3 |
| 3 | rows: rules.xml `cannontimes` 0, 1, 2, 4, 8 | yes | yes (count 5 from `init_rules_data`) | agree | not re-read; two independent readings of the file agree |
| 4 | step: `TurnControl::start_cannon_time (00956500)` −1 when `who >= 0`, `cannon_time_who < 0`, `cannon_time != 0`; also stamps who, frame, saved speed, tick | yes | yes | agree | confirmed: `js` 00956528, `cmpl $0,0x24(%esi); jl` 0095652e, `testl; jne` 00956590, `decl` / store 009565ea..009565eb, 009565f1..00956609. The refusals post feedback only when `who` is the console's (`cmpl 0x298`), else return silently. |
| 5 | issuer: `CommandManager::issue_cannon_time (00941260)` refuses without `semaphore[0] & 4`, then sends 0x4d only if the console leader's `cannon_time` is unsigned `> 0` | yes ("!= 0") | yes ("unsigned `> 0`") | agree (same test: unsigned `> 0` is `!= 0`) | confirmed, `cmpl $0x0,0xe3a3d8(%eax); ja` at 009412f1..009412f8 |
| 6 | `process_cannon_time (009464f0)` and `start_cannon_time` themselves test no semaphore | yes | implicit | one-sided | confirmed for `start_cannon_time` (00956500..00956609 reads no `0x820`); a crafted 0x4d in solo would reach it, and with row 0's 0 it is refused at 00956592 |
| 7 | clears: `Leader::init`'s memset (`tribe < 0`, `[+0, +0x6929)`), `Leaders::clear (006ed5d0)` at game end | both | memset only | one-sided (`Leaders::clear`) | confirmed memset 006e398c..006e3995; `Leaders::clear` accepted on the first's citation (game end, not a start) |
| 8 | `walk_data (006d6750)` restores it on a save load | yes | yes | agree | not re-read |
| 9 | `Leader::reset_score (006e37f0)` does not touch it | yes | no claim | one-sided | accepted (the listing's absolute net and the `this`-relative net raise no store there) |
| 10 | readers: the two guards; `IFaceMainBase::update_visibility (00811d40)` greys the button; `log_data` does not print it | yes | guards and `log_data` | one-sided (`update_visibility`) | the absolute net shows only the two guard reads (009412f1, 0095658a), so `update_visibility` reads it through a register; UI only, no bearing on the rule |
| 11 | crate: none; `LEADER_FIELDS` `checksum/leaders.rs:53`; held out `checksum_lines.rs:1771`; `Command::CannonTime` decoded at `rondata/src/commands.rs:996` and unconsumed | yes | yes | agree | confirmed, lines as cited |
| 12 | plain game: seeded, never stepped, 0 under `CANNON_TIMES 0` | yes | yes | agree | confirmed for the step (issuer gated on bit 2, 009412f8's refusal branch above it); the 0 rests on `CANNON_TIMES 0`, which the first finds in 1,193 of 1,194 dumps |

**Writer completeness.** Absolute net: one store, 009565eb. Walking
pointer sweep: no store landing on +0x48. `this`-relative: 006e3af3 (the
006e3f6a store is off the `+0x6eb8` `LeaderDataEncrypt*`, as both say).
No new writer.

### 2. The rule as a builder should port it

- **Writer:** at every leader build, `cannon_time =
  CANNON_TIMES_ROW[lobby.cannon_times]` with rows `[0, 1, 2, 4, 8]`, for
  every slot 0..9 (the seed is not gated on seating).
- **Step:** −1 on an accepted command 0x4d (nobody already in cannon
  time, value non-zero, `who >= 0`); the command's only issuer refuses
  without the multiplayer bit.
- **Reset:** the next game build.
- **Crate site:** add `cannon_times` to `ai::Lobby` (parsed from the
  dump's `CANNON_TIMES`), and a `"cannon_time"` arm in `Sim::leader_long`
  (`crates/sim/src/checksum/leaders.rs:949`) returning the row's count.
- **Plain game:** reached (the seed); frozen at 0 with the default lobby;
  the step never fires solo.
- **Confidence:** high. **Open:** whether the solo setup window offers a
  non-zero `CANNON_TIMES` (it would change the frozen value, not the
  rule); `walk_data`'s save path.

## Leaf `handicap` (+0x4c, int)

### 1. Claim by claim

All offsets below are `Player`'s (`flags` +0x30, `who` +0x33, `team`
+0x34 signed, `handicap` +0x35, `diff` +0x38), the walk in the human pass
being `%esi = game + 0x79` = `player[0].handicap` (0058ac89).

| # | claim | first | second | mark | verdict, deciding line |
|---|---|---|---|---|---|
| 1 | writer `Game::init_handicaps (0058abf0)`, stores 0058ade8 and 0058ae07, slots 0..7 seated | yes | yes | agree | confirmed, `movl %eax,(%esi)` twice, guard 0058ad90 |
| 2 | pass 0: `own[i]` = max `handicap` over occupied records with `who == i`, any kind, floor 0 | yes | yes | agree | confirmed: 0058ac54..0058ac5a run for a person too (the `testb $4; jne` at 0058ac46 jumps to 0058ac54, past only the `diff` max) |
| 3 | `sum[i]` starts at `own[i]` when `Game::is_cooperative (00594810)`, else 0 | yes | yes | agree | confirmed: `neg; sbb; and %edi` 0058ac7c..0058ac87 |
| 4 | human pass over `(flags & 5) == 5`: own slot → `own = max`, `sum += h`; other slot, team differs from `get_team(i)` → `enemy = max`, nothing added; other slot, same team → `mates += 1` unless `team == 8`, `sum += h` | yes | yes | agree | confirmed: 0058ac90..0058ac97, `cmpl %ebx` 0058aca0, `get_team` 0058aca7, signed `movsbl` 0058acac, enemy 0058acb7..0058acbf, mates `cmpb $8; cmove` 0058acce..0058acd5, sum 0058ace5..0058acea |
| 5 | `top` = max over seated slots of `sum`; the maxima of `own` and `enemy` are computed and discarded | yes | yes | agree | confirmed: 0058ad50..0058ad82 builds three maxima from 0; `%edi` cleared at 0058ad84, `%esi` reloaded at 0058ad86, `%ebx` (the `sum` max) survives into 0058ada1 |
| 6 | store: no mates → `clamp(top − sum, 0, 20)`; mates → `((top − sum) / (mates + 1) + max(0, enemy − own)) / 2`, truncating, clamp 0..20; then `/ max(1, num_teams − 1)` (`Game+0x6a4`) | yes | yes | agree | confirmed: 0058ada7..0058add6 (`idivl`, `cmovsl` 0, `cltd; sub; sar` a truncating halving, `js` → 0, `cmovg` 20), 0058add8..0058ade4, 0058adea..0058ae07 (`decl; cmpl $1; cmovl` 1) |
| 7 | "a computer record's handicap byte matters only through `d = enemy − own`" | no claim | yes | one-sided | **corrected**: it also enters `sum[i]`'s start under `is_cooperative` (claim 3 seeds `sum` from pass 0's `own`, which includes computer records). Moot in a plain game (`Player::init` zeroes every byte). |
| 8 | slots 8 and 9 never written | yes | yes | agree | confirmed (loop bound `0xe71b3c` = slot 8's leaf) |
| 9 | `ScenarioRead::load_leader_chunk (009ac080)` stores chunk `+0xe0` at 009ac1fc, 009ac457, 009ac643, each behind `semaphore[3] & 4 == 0` | yes | yes (the gate unstated) | agree; gate one-sided | confirmed: `testb $0x4,0x823(%edx)` at 009ac186, 009ac392, 009ac56f precede the three stores; `movl 0xe0(%edx),%eax` 009ac1f6 |
| 10 | memsets (`Leader::init` `tribe < 0`, `Leaders::clear`) and `walk_data` | yes | memset, `walk_data` | agree (`Leaders::clear` one-sided) | as for `cannon_time` |
| 11 | readers: `get_handicap (006da740)`, and every caller tests `semaphore[0] & 4` set first | yes | yes | agree | confirmed: three `call`s (00650b67, 00651780 in `ObjectData::train_time (006508c0)`, 006b107f in `World::compute_reg_territory (006b0bb0)`) plus `get_gather_handicap (006d66a0)`'s tail; each behind the bit (train_time decompile lines 98, 537; compute_reg_territory line 250; get_gather_handicap line 13) |
| 12 | `get_handicap`'s difficulty shift (+5/+10/+16) runs only with the bit clear, so is unreachable from those callers | yes | yes (and `+16` would overrun the 21 rows) | agree | accepted; follows from claim 11 |
| 13 | `get_handicap_level (006d6740)` has no caller | "callers not followed" | "no caller" (`docs/CENSUS.md` §385) | one-sided | accepted on the second's citation; no bearing on a plain game |
| 14 | crate: none; `Holdings::handicap` is a different quantity (`rondata/src/diff/leader.rs:55`); `LEADER_FIELDS` `checksum/leaders.rs:54`; held out `checksum_lines.rs:1793` | yes | yes | agree | confirmed |
| 15 | plain game: 0 every slot | yes | yes | agree | confirmed: one person at handicap 0 gives every `sum` 0, `top` 0, every `h` 0; the first's scan finds only 0 in 1,194 dumps' leader `handicap` lines |

**Writer completeness.** Absolute net: `0xe3a3dc` is stored only at
009ac1fc, 009ac457, 009ac643, read at 009a7a80, and loaded as the
walking pointer at 0058ad86. Walking-pointer sweep: only 0058ade8 and
0058ae07. No new writer.

### 2. The rule as a builder should port it

- **Writer:** once at game setup, after the leaders are cleared, over
  the eight lobby `Player` records and seated slots 0..7: pass 0
  `own[i]`, `aidiff[i]`; `sum[i] = cooperative ? own[i] : 0`; the human
  pass (claim 4); `top = max(0, max sum)`; the store (claim 6), each
  division truncating toward zero. Slots 8, 9 and unseated: 0.
- **Step / reset:** none per frame; the next game build.
- **Crate site:** a `"handicap"` arm in `Sim::leader_long`
  (`crates/sim/src/checksum/leaders.rs:949`). For a solo game it may
  return 0 for every seated slot only while the lobby carries no
  per-player handicap; the full computation needs the lobby's `Player`
  table (`team`, `handicap`, `flags`) and `num_teams`, which
  `rondata::recgame::Slot` carries and `ai::Lobby` does not.
- **Plain game:** reached; 0 everywhere; no reader consumes it without
  the multiplayer bit.
- **Confidence:** high for the formula (every branch read in the listing,
  agreeing with both readings). **Open:** `is_cooperative`'s full
  predicate and `get_team`'s team-8 branch (both matter only with more
  than one person); whether the solo setup window lets a person set a
  non-zero handicap.

## Verdict table

| leaf | claims confirmed / corrected / open | the rule in one line |
|---|---|---|
| `multi_diff` +0x50 | 12 / 2 / 0 (corrected: the second's "0 in every kept dump", stale since §80.4; the first's "no other occupied **person**" in `process_drop`, which tests no human bit) | at setup, each seated slot 0..7 gets `lobby.difficulty` in a solo random map (the slot's max computer `Player.diff`, else 2, under multiplayer or a setup-chosen scenario; untouched in a plain scenario); frozen; port as `lobby.difficulty`, **not** 0 |
| `cannon_time` +0x48 | 12 / 0 / 0 (the first's header "off+8" is a self-contradiction its body corrects) | every leader build seeds `[0,1,2,4,8][lobby.cannon_times]` for every slot; −1 per accepted command 0x4d, whose only issuer needs the multiplayer bit; plain game 0, frozen |
| `handicap` +0x4c | 14 / 1 / 0 (corrected: a computer record's byte also seeds `sum` under `is_cooperative`) | at setup, the catch-up row: `clamp(top − sum, 0, 20)` (halved with the enemy-over-own gap when the slot has human teammates) `/ max(1, num_teams − 1)`, slots 0..7; plain game 0; read only under the multiplayer bit |

New finding beyond both readings: `semaphore` bit 26 (`Game+0x823 & 4`)
is set only by `SetupWin::setup_game (005b3860)` at 005b4389 when the
setup window's combo 0x36 picks entry 10 (a scenario from the skirmish
setup), and cleared by `Game::run_scenario (005860c0)` and
`Game::run_playback (00586ea0)`; clear in every plain random-map game.
