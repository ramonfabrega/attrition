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
  the offset, the harness's own per-capture lobbies). The dump
  cross-check is below.
- **Open:** none for a plain game. `change_difficulty`'s callers and the
  13 inlined `get_diff` copies were not re-read here (neither changes the
  plain-game value).

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
