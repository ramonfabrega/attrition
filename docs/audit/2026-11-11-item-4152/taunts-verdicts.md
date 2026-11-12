# Item 4152, family 9 (`taunts`): adjudication

- Slug: `taunts`. Leaves (`LeaderData`): `last_taunt` (+0x354,
  `TauntRequest[8]`), `taunt_frame` (+0x374, `int[8]`), `incoming_taunt`
  (+0x394, `int[8]`), `incoming_taunt_who` (+0x3b4, `int[8]`),
  `incoming_taunt_frame` (+0x3d4, `int[8]`), `incoming` (+0x3f4, `int`).
- Readings: `taunts-reading.md` (first), `taunts-second.md` (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text`
  listing (`exe.s` in the job scratch), `crates/sim` and `crates/rondata`
  grep. Nothing run of the original.

(Written per leaf as settled.)

## 0. The shared pipeline (claims common to all six leaves)

The two readings describe one chain: a human's chat line, a lockstep
`'D'` command, `receive_taunt` filling an 8-slot ring, a per-frame drain in
`Leaders::process_all`, `process_taunt` stamping `last_taunt` /
`taunt_frame`, and `action_ping` consuming them. The claims below are
settled once here and cited by the leaves.

| # | claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| 0.1 | `LeaderData` at offset 0 of `Leader`; no offset shift | yes | yes | agree | **confirmed** | `Leader::receive_taunt (006b8bf0)` 0x6b8c44 `cmpl %edi, 0x8(%esi)` reads `who` at +0x8 off `this`; `process_taunt` 0x6b8ce0 likewise |
| 0.2 | `TauntRequest` 0..17, `HELP` = 16 | yes | yes | agree | **confirmed** | 0x6b8c68 `cmpl $0x10, %ebx` (the HELP arm); 0x6d1977 `cmpl $0x10` (the ping test) |
| 0.3 | `receive_taunt`'s only callers are `CommandPackage::process_chat (009454f0)`, one per arm, when `taunt >= 0` | yes | yes | agree | **confirmed** | `calll 0x6b8bf0` occurs at 0x94573a and 0x945870 only; the export's `funcs/CommandPackage/process_chat` file (009454f0) `if (-1 < *local_28)` before each call |
| 0.4 | `process_chat`'s delivery gate | broadcast: sender's `chat_status[recv]` = 0 and receiver's `chat_status[sender]` != 2, each forgiven by `semaphore & 0x40`; masked: receiver's bit set | `chat_status` not 2; masked: not a cheat line in a non-network game | disagree in detail | **corrected** (both partial). For each valid slot k: **broadcast** (`bits == -1`): (`leaders[S].chat_status[k] == 0` or `sem & 0x40`) and (`leaders[k].chat_status[S] != 2` or `sem & 0x40`). **Masked**: (`bits` bit k or `sem & 0x40`); then, outside multiplayer (`sem & 4` clear), a line starting with the cheat string goes to `ConsoleWin::parse_cmd` and nothing is delivered; else (`leaders[k].chat_status[S] != 2` or `sem & 0x40`). The masked arm has **no** sender-side test. A further gate neither reader named: when the sending player has a `net_player` that `Crossplay::Service()` reports blocked, nothing is delivered to anyone. | the export's `funcs/CommandPackage/process_chat` file (009454f0) lines read by `grep -n 'local_2c == 0xffffffff'` and the `LAB_00945817` arm; the crossplay `goto LAB_0094563c` with `bVar2 = true` |
| 0.5 | No computer issues a chat command; every producer is a window of the local human | yes | yes | agree | **confirmed**. `CommandManager::issue_chat (00942b10)` has two direct callers, 0x6d3205 (`LeaderOut::issue_chat`) and 0x93d666 (`ChatBox::on_modal_end`'s cheat path); `LeaderOut::issue_chat` three, 0x801722, 0x80197c, 0x93d6f6; `CommandPackage::add_chat` one, 0x942c10; none of the four addresses occurs as a 32-bit word anywhere in the PE (no vtable or pointer). The second's "`CommandManager::issue_chat` has only the two interface callers" is right in count; the two are `LeaderOut::issue_chat` and `ChatBox`. | the `calll` census above |
| 0.6 | The cheat path pushes `taunt = 0` | yes | (silent) | one-sided | **confirmed**: `add_chat`'s `param_2` is the `taunt` word (+5), and `issue_chat`'s third argument feeds it; 0x93d65f `pushl $0x0` is that argument. `receive_taunt` is then called (0 >= 0) and rejects it at 0x6b8c0f. | 0x93d65d..0x93d666; the export's `funcs/CommandPackage/add_chat` file (0094c080) `*(int *)(this->data + iVar4 + 5) = param_2` |
| 0.7 | `Taunts::check_for_order (009809d0)`: −1 when no taunt has the id or its order string is empty, 0 when the order string matches no name, else 1..16 | yes | "−1 otherwise" | disagree (minor) | **corrected toward the first**: −1 for no id / empty order string, **0** for an unmatched string, else the request. Neither reader noted that **two** names map to `HELP` (the compares at string offsets `+0x2620` and `+0x1f1f8`). Harmless to the leaves: 0 and −1 are both refused or never delivered. | the export's `funcs/Taunts/check_for_order` file (009809d0), the `return -1` and the `iVar2 = 0x10` after the two-name test |
| 0.8 | The drain: `Leaders::process_all (006ed2a0)`, for every slot with `flags & 2`, when `game->frame != 0`, for k = 0..7 in order, if `incoming_taunt_frame[k] == frame` call `process_taunt(incoming_taunt[k], incoming_taunt_who[k])`; slot not cleared | yes | yes | agree | **confirmed** | 0x6ed2b0 `testb $0x2, (%esi)`; 0x6ed3ce `cmpl $0x0, 0x550(%ecx)`; 0x6ed3e2..0x6ed3f4 |
| 0.9 | `Leader::process (006b88b0)`, `Leader::process_taunts (006b8ba0)`, `Leader::close (006b7f80)` are dead (no caller) | yes (close: "no direct caller found") | yes (and no PE word) | agree | **confirmed**: no `call`/`jmp` to any of the three in `.text`, and none occurs as a 32-bit word in the PE | listing grep; PE byte count 0 for each |
| 0.10 | Frame order: commands are applied in `TurnControl::do_frame` before `Game::do_frame`'s `Leaders::process_all`, on the same `game->frame` | yes (medium) | yes (medium) | agree | **confirmed, by a dump, not only a reading**: run1077's help taunt fired at all, and `taunt_frame[0]` reads 5411 on its 5412 dump (`docs/RUNS.md`, run1077 entry); a `HELP` received after that frame's drain could never fire, since the due test is equality. run1086's food taunt: `taunt_frame[0]` 661 with `receive_taunt` entered one block earlier, so receipt frame + 60 = 661. Both are dump readings in `docs/RUNS.md`, **not yet assertions**. | `docs/RUNS.md` run1077 ("On 5412 leader 1 has `last_taunt[0]` 16 and `taunt_frame[0]` 5411") and run1086 ("On 661 who 1 holds `last_taunt[0]` 1, `taunt_frame[0]` 661") |
| 0.11 | Sender range | 0–7 | 0..9 | disagree (immaterial) | **open, immaterial**: `S` is the byte `player[play].who` (0x6b8c08); the ring and `last_taunt` are `[8]`, `leaders.list` has ten records. A live player's `who` is 0..7 in every capture; nothing read here bounds the byte. | 0x6b8c08 `movzbl 0x77(%eax,%ecx), %edi` |

### 0.R Reset writers, common to all six

- `memset(this, 0, 0x6929)` covers +0x0..+0x6928: **confirmed** at three
  sites. The readings cite different instructions of the same sequences
  (the first the `calll`, the second the `pushl $0x6929`): `Leader::init`
  0x6e398c/0x6e3995 on the `param_2 < 0` branch (0x6e394a `jns 0x6e3a36`
  skips both the memset and the explicit loop), `Leader::close`
  0x6b7f82/0x6b7f8c (dead, 0.9), `Leaders::clear` 0x6ed5e0/0x6ed5ee.
- `Leader::init`'s explicit loop 0x6e3a01..0x6e3a1e (the three ring arrays
  and `incoming`), same branch: **confirmed**.
- **Completeness, a finding**: `Leader::init (006e3930)` has ten callers,
  and the readings named three (`Leaders::init` 0x6ed8cb, `Setup::build_game`
  0x5acef7, and the second's `WalkDataGame::walk_data` 0x5a30e6, which
  passes `(9, -1, 9)` on slot 9 at 0xe789dc and so clears slot 9). The
  other seven pass a tribe word as `param_2`: `Setup::build_leader`
  (0x5abc8d, `leaders[i]+0xc`), `MapScenario::make` (0x68cf18, a byte),
  `ConquestGame::build_conquest_game` (0x797e52), `ScenarioEditor::new_leader`
  (0x9a1575, a `% 0x18`), `ScenarioRead::load_leader_chunk` (0x9ac1a9,
  0x9ac24f, 0x9ac3bc). Any of them clears the family only if that word is
  negative, and all run at setup, when the family is already zero. Not
  material to a plain game; **open** whether `Setup::build_leader` can run
  mid-match (it would re-zero only on a negative tribe).
- `LeaderData::walk_data (006d6750)` over [+0x8, +0x692a): agreed by both as
  the load-mode writer and checksum reader; **confirmed** as to range by
  `docs/CHECKSUM.md` §8, load mode not re-read (as both said).

### 0.N The writer nets, re-run

- Absolute: every leader slot 0..9's family range (`0xe3a390 + i·0x6eec +
  0x354..0x3f7`) grepped over `exe.s`: two hits, both reads in
  `action_ping` (0x6d1977, 0x6d198a). `0xe3a788` (+0x3f8, one past the
  family) is used 16 times and as an immediate at 0x62964d; it is the next
  leaf, not this family.
- Register-relative: every operand with displacement 0x354..0x3f7 mapped to
  its function through `INDEX.tsv` (2,526 operands). Indexed stores of the
  family's exact shape (`0x354/374/394/3b4/3d4(%r,%r,4)`): six in
  `receive_taunt`/`process_taunt` and one in `GameSpyGroup::OnDownloadScores`
  (0x96c1ee, not a leader). Stores in `Leader`/`Leaders`/`LeaderData` in the
  range besides those: `Leader::end_process` / `Leaders::end_process_all`
  `0x3bc` (base walks 0x330, 0x3bc, 0x448: the `0x8c`-stride `GameInfo`
  player record), `Leader::plan_strategy` 0x6b9be3 and `Leader::init`
  0x6e4b34 `0x37a` (base `ebx+0xbe6+…`, a ushort array far past the family),
  `Leader::init` 0x6e3f20 `0x3f0` (a walking pointer in the encrypted
  block), `Leader::production_ai_setup` 0x6c8721 `0x3a4(%esi,%ebx)` (byte
  offset `ebx`, the encrypted-int walk). None is a family store. `Game::close`
  / `Game::run_oos_recovery` `0x3b4` and `0x3d0` are off the pointer at
  `0xc06214` (a 0xc4 × 0x1c table; the first said 0xc4-stride, the second
  0x1c-stride, both are strides of the same 2-D clear), not a leader.
- **No writer beyond the two functions and the reset writers above.**

## 1. `last_taunt` (+0x354, `TauntRequest[8]`, indexed by sender `who`)

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| writer | `process_taunt` 0x6b8d46 | same | agree | **confirmed** | 0x6b8d46 `movl %ecx, 0x354(%esi,%ebx,4)`, `ecx` = arg 1 (req), `ebx` = arg 2 (S) |
| predicate, receiver | `leaders[R.who]` not human (bit 4) and active (bit 2) | same | agree | **confirmed** | 0x6b8cef `testb $0x4, %cl`, 0x6b8cf8 `testb $0x2, %cl` on `0xe3a390 + who·0x6eec` |
| predicate, alliance | if S ≠ R: `leaders[R].diplos[S] == 2` and `leaders[S].diplos[R] == 2`, re-tested at drain | same | agree | **confirmed** | 0x6b8d0e `cmpl $0x2, 0xe3a404(%eax,%ebx,4)`; 0x6b8d29 `cmpl $0x2, 0x74(%eax,%ecx,4)` with `ecx = S·0x1bbb + R` |
| predicate, self | S ≠ R | same | agree | **confirmed** | 0x6b8d34 `cmpl %ebx, %edx; je` |
| predicate, request bound | none at store; > 16 stored and falls to default | same | agree | **confirmed**: the store precedes the range test | 0x6b8d4d `decl %ecx`, 0x6b8d5b `cmpl $0xf, %ecx; ja 0x6b9264` |
| step | overwrite `[S] = req` | same | agree | **confirmed** | 0x6b8d46 |
| reset | `Leader::init(…, <0, …)`, `Leaders::clear`, (`Leader::close`); never in a match | same | agree | **confirmed**, with §0.R's added callers | §0.R |
| reader | `action_ping`: human pinger P pinging for itself; each other valid non-human `is_ally` k with `last_taunt[P] == 16` and `frame <= taunt_frame[P] + 225` sends `find_useful_army` via `send_here(MOVE_TO)`, army +0x28 = 0xa8c | same | agree | **confirmed**; loop runs 8 slots (0xc09710 to 0xc097f0, stride 0x1c); a failed `is_ally` continues the loop (0x6d196f → 0x6d1adf → 0x6d1aaf) | 0x6d190e `testb $0x4, (%edi)`, 0x6d1917, 0x6d1938, 0x6d194d, 0x6d1964, 0x6d1977, 0x6d1991..0x6d199d, 0x6d19ae, 0x6d19d0, 0x6d19de |
| crate | none; leaf listed `checksum/leaders.rs:87`, `leader_long` (949) falls to `_ => None` (1305); chat skipped `rondata/src/input.rs:1611`; drain home `core/frame.rs:94` | same (cites ~949, `Stream::one`) | agree | **confirmed**: `leader_long` at `crates/sim/src/checksum/leaders.rs:949`, its catch-all at :1305; `Command::Chat` skipped at `crates/rondata/src/input.rs:1611` inside `Stream::one` (:1503); the per-player loop at `crates/sim/src/core/frame.rs:94` is `Leaders::process_all`'s analogue; `Armies::find_useful_army` exists (`crates/sim/src/ai/army.rs:2207`) with no caller; `Command::Ping` is not mapped | the cited lines |
| plain game | only a human's taunt-order chat to a mutually allied computer | same | agree | **confirmed**; run1077 and run1086 are such games | §0.5; `docs/RUNS.md` run1077, run1086 |
| dump corroboration | (none) | `blind.rs` comments, run1077 | one-sided | **confirmed** and stronger than the second said: `docs/RUNS.md` run1077 reads leader 1's `last_taunt[0]` 16 on 5412; run1086 reads `last_taunt[0]` 1 on 661 and 2 on 681. Dump readings, not assertions. | `docs/RUNS.md` run1077, run1086 entries |

**The rule as a builder should port it.**

- State: per leader, `last_taunt: [i32; 8]`, zero at game start.
- Writer: `process_taunt(R, req, S)`, reached only from the drain (§5). If
  R is active and not human, and S ≠ R, and R and S are mutually allied
  (`diplos == 2` both ways, tested now), set `last_taunt[S] = req` (any
  value the ring holds, 1 and up), then act on `req` (1..16; above 16 does
  nothing).
- Reset: game start only.
- Reader: `action_ping` when a human P pings for itself: for each other
  valid, non-human leader k with `is_ally(P, k)`, `last_taunt[P] == 16` and
  `frame <= taunt_frame[P] + 225`, send `find_useful_army(k, …)` to the spot
  and set that army's +0x28 to 2,700.
- Crate site: a new `process_taunt` called from the drain in `Sim::tick`'s
  per-player loop (`crates/sim/src/core/frame.rs:94`); a `leader_long` arm
  (`crates/sim/src/checksum/leaders.rs`) to hold the leaf.
- Plain game: only with a human who has a computer ally and sends a
  taunt-order chat; otherwise constant 0.
- **Confidence: high**; the store and gates are read in the listing by two
  readers and the adjudicator, and two dumps carry values the rule predicts.
  Open: `LeaderData::walk_data` in load mode (not re-read).

## 2. `taunt_frame` (+0x374, `int[8]`, indexed by sender `who`)

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| writer | `process_taunt` 0x6b8d54, `eax = game->frame` | same | agree | **confirmed** | 0x6b8d41 `movl 0xc061ec, %eax`, 0x6b8d4e `movl 0x550(%eax), %eax`, 0x6b8d54 `movl %eax, 0x374(%esi,%ebx,4)` |
| predicate | §1's gates, same breath | same | agree | **confirmed** | §1 |
| step | overwrite `[S] = frame` at dispatch; = receipt for HELP, receipt + 60 otherwise | same | agree | **confirmed**, by dump (§0.10): 5411 for run1077's HELP, 661 and 681 for run1086's food and timber | 0x6b8d54; `docs/RUNS.md` run1077, run1086 |
| reset | as §1 | same | agree | **confirmed** | §0.R |
| reader | `action_ping`, `frame <= taunt_frame[P] + 225` | same | agree | **confirmed**: `jl` skips when `taunt_frame + 0xe1 < frame`, so frame = tf + 225 still answers | 0x6d198a, 0x6d1991 `addl $0xe1, %ecx`, 0x6d1997 `cmpl 0x550(%eax), %ecx`, 0x6d199d `jl` |
| crate | none; `checksum/leaders.rs:88` | same | agree | **confirmed** | `crates/sim/src/checksum/leaders.rs:88` |
| plain game | as §1 | same | agree | **confirmed** | §1 |

**The rule as a builder should port it.** `taunt_frame: [i32; 8]`, zero at
game start; set `taunt_frame[S] = frame` (sim frames, absolute) in the same
`process_taunt` store as `last_taunt[S]`, under the same gates. Read only by
`action_ping`'s window `frame <= taunt_frame[P] + 225`. Crate site as §1.
Plain game as §1. **Confidence: high.** Open: nothing beyond §1's.

## 3. `incoming_taunt` (+0x394, `int[8]`, the ring's request word)

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| writer | `receive_taunt` 0x6b8c4f, slot `incoming` | same | agree | **confirmed** | 0x6b8c49 `movl 0x3f4(%esi), %eax`, 0x6b8c4f `movl %ebx, 0x394(%esi,%eax,4)` |
| sender S | `player[play].who`, byte +0x77 of the 0x8c record | same | agree | **confirmed** | 0x6b8bf3 `imull $0x8c, 0xc(%ebp)`, 0x6b8c08 `movzbl 0x77(%eax,%ecx), %edi` |
| predicate: `req != 0`, `req >= 0`, no upper bound | yes | yes | agree | **confirmed** (signed: `js` rejects negatives) | 0x6b8c0d `testl %ebx,%ebx; je`, 0x6b8c15 `js` |
| predicate: sender active | `leaders[S].flags & 2` | same | agree | **confirmed** (active, not merely valid) | 0x6b8c21 `testb $0x2, 0xe3a390(%eax)` |
| predicate: receiver not human, active | yes | yes | agree | **confirmed** | 0x6b8c30 `testb $0x4, %al; jne`, 0x6b8c34 `testb $0x2, %al; je` (`this`'s own flags here, where `process_taunt` reads `leaders[this->who]`'s: the same word for a seated leader) |
| predicate: `is_ally(R, S)` | S = R or both `diplos` 2 | same | agree | **confirmed** | `LeaderData::is_ally (006edb50)` 0x6edb59 (self → 1), 0x6edb66 and 0x6edb7a (both `== 2`) |
| predicate: R ≠ S | yes | yes | agree | **confirmed** | 0x6b8c44 `cmpl %edi, 0x8(%esi); je` |
| step | overwrite of one ring slot | same | agree | **confirmed** | 0x6b8c4f |
| reset | init memset + explicit loop (0x6e3a10), `Leaders::clear`, (`Leader::close`); not cleared on dispatch | same | agree | **confirmed** | 0x6e3a10 `movl %edx, -0x20(%eax)`, `eax = this+0x3b4+4k`; the drain 0x6ed3e2..0x6ed405 stores nothing |
| reader | the drain, `pushl -0x20(%edi)` 0x6ed3f1 | same | agree | **confirmed** | 0x6ed3f1 |
| delivery gate upstream | §0.4 (first's version) | §0.4 (second's version) | disagree | **corrected**, see §0.4 | §0.4 |
| crate | none; `receive_taunt` from a chat arm replacing `input.rs:1611`; ring on the leaders record; drain in `core/frame.rs:94` | same, ring in `crates/sim/src/leaders.rs`, `checksum/leaders.rs:89` | agree | **confirmed**: `Command::Chat` is decoded with `taunt` and `taunt_num` (`crates/rondata/src/commands.rs:921`) and dropped at `input.rs:1611`; `crates/sim/src/leaders.rs` exists | cited lines |
| plain game | human → computer ally | same | agree | **confirmed** | §0.5 |

**The rule as a builder should port it.**

- State, per leader: `incoming_taunt: [i32; 8]`, `incoming_taunt_who:
  [i32; 8]`, `incoming_taunt_frame: [i32; 8]`, `incoming: i32`, all zero at
  game start.
- Producer: a `Chat` command from player `play` with `taunt >= 0`. For each
  valid slot k (0..7), apply §0.4's delivery gate, then call
  `receive_taunt(k, taunt, play)`.
- `receive_taunt(R, req, play)`: `S = player[play].who`. Accept only if
  `req > 0` (signed), S is active, R is active and not human,
  `is_ally(R, S)`, and R ≠ S. Then `incoming_taunt[incoming] = req`.
- Reset: game start only; dispatch leaves the slot as it was.
- Crate site: a chat arm in `Stream::one` (`crates/rondata/src/input.rs`,
  replacing the skip at :1611) calling a new `receive_taunt` on the sim;
  the ring on the leaders record (`crates/sim/src/leaders.rs`); a
  `leader_long` arm for each leaf.
- Plain game: as §1.
- **Confidence: high** on the store and gates (listing, three readers);
  **medium** on the delivery gate's `semaphore & 0x40` and crossplay arms
  (decompile only, and neither bit's meaning is established here).

## 4. `incoming_taunt_who` (+0x3b4, `int[8]`, the ring's sender word)

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| writer | `receive_taunt` 0x6b8c5c, `edi` = S | same | agree | **confirmed**; `incoming` re-read (0x6b8c56) but unchanged, so the same slot as §3 | 0x6b8c5c `movl %edi, 0x3b4(%esi,%eax,4)` |
| value | S, the leader `who`, not the play index | same | agree | **confirmed** | 0x6b8c08 |
| value range | 0–7 | 0..9 | disagree | **open, immaterial** (§0.11) | 0x6b8c08 |
| predicate, step | as §3 | same | agree | **confirmed** | §3 |
| reset | init loop 0x6e3a13, memsets | same | agree | **confirmed** | 0x6e3a13 `movl %edx, (%eax)` |
| non-leader `0x3b4` stores | `Game::close`/`run_oos_recovery` 0xc4-stride; `ScenarioRead::load_leaders_chunk` 0x8c record | 0x1c-stride table; `load_leaders_chunk` game-info records | disagree (immaterial) | **confirmed not leaders**: base `[0xc06214]`, a 2-D clear of strides 0xc4 and 0x1c (§0.N); `load_leaders_chunk`'s `edx` is a non-leader record | 0x586af8 `movl 0xc06214, %eax` |
| reader | the drain, `pushl (%edi)` 0x6ed3ed, as `process_taunt`'s S | same | agree | **confirmed** | 0x6ed3ed |
| crate | as §3 (`checksum/leaders.rs:90`) | same | agree | **confirmed** | `crates/sim/src/checksum/leaders.rs:90` |
| plain game | as §1 | same | agree | **confirmed** | §1 |

**The rule as a builder should port it.** In `receive_taunt`, after the
§3 gate, `incoming_taunt_who[incoming] = S` (the sender's leader `who`).
The drain passes it to `process_taunt` as S, which indexes `last_taunt`,
`taunt_frame` and the alliance test by it. Reset at game start only; not
cleared on dispatch. Crate site and plain game as §3. **Confidence: high.**
Open: whether a `who` byte above 7 can reach it (§0.11).

## 5. `incoming_taunt_frame` (+0x3d4, `int[8]`, the ring's due frame)

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| writers | `receive_taunt` 0x6b8c79 (HELP) and 0x6b8c91 (else) | same | agree | **confirmed** | 0x6b8c68 `cmpl $0x10, %ebx; jne 0x6b8c82` |
| value, HELP | `game->frame` | same | agree | **confirmed** | 0x6b8c73 `movl 0x550(%eax), %eax`, 0x6b8c79 |
| value, other | `game->frame + 60` | same | agree | **confirmed**, and dump-backed (run1086: due 661 for a request received on the block before) | 0x6b8c8e `addl $0x3c, %ecx`, 0x6b8c91 |
| HELP fires on its receiving frame | medium | medium | agree | **confirmed by dump** (§0.10) | `docs/RUNS.md` run1077 |
| drain skips frame 0, so a HELP received on frame 0 never fires and the zeroed ring never fires | yes (both halves) | the zeroed-ring half | agree / one-sided | **confirmed** | 0x6ed3ce `cmpl $0x0, 0x550(%ecx); je 0x6ed407` |
| a ninth request inside one delay overwrites an unfired slot; no occupancy check | yes | yes | agree | **confirmed**: `receive_taunt` reads no slot state before writing | 0x6b8c49..0x6b8cab |
| several slots due on one frame | (implicit) | k = 0..7 in order | one-sided | **confirmed**: the drain walks k = 0..7 and calls once per match, in slot order, not arrival order (after the ring wraps the two differ) | 0x6ed3dd `movl $0x8, %ebx`, 0x6ed3ff `addl $0x4, %edi` |
| reset | init loop 0x6e3a18, memsets | same | agree | **confirmed** | 0x6e3a18 `movl %edx, 0x1c(%eax)` after `leal 0x4(%eax)`, so +0x3d4 + 4k |
| reader | the drain's due test 0x6ed3e2 | same | agree | **confirmed** | 0x6ed3e2 `movl 0x20(%edi), %eax`, 0x6ed3e5 `cmpl 0x550(%ecx), %eax` |
| crate | as §3, compare with the sim's `frame` (`checksum/leaders.rs:91`) | same | agree | **confirmed** | `crates/sim/src/checksum/leaders.rs:91` |
| plain game | as §1 | same | agree | **confirmed** | §1 |

**The rule as a builder should port it.** In `receive_taunt`, after the §3
gate: `incoming_taunt_frame[incoming] = frame` if `req == 16`, else
`frame + 60`, where `frame` is the sim frame at which the command is
applied (before that frame's leader pass). The drain, at the tail of each
active leader's per-frame step and only when `frame != 0`, calls
`process_taunt(incoming_taunt[k], incoming_taunt_who[k])` for each k in
0..7 whose due frame equals `frame`. Nothing is cleared. Reset at game start
only. Crate site and plain game as §3. **Confidence: high** (both constants
in the listing; the zero delay and the 60 both read back from dumps).
Open: nothing material.

## 6. `incoming` (+0x3f4, `int`, the ring's write cursor)

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| writer | `receive_taunt` 0x6b8cab | same | agree | **confirmed** | 0x6b8cab `movl %eax, 0x3f4(%esi)` |
| step | `(x + 1)` signed `% 8`; `& 7` for x in 0..7 | same | agree | **confirmed**: `incl; andl $0x80000007; jns; decl; orl $-8; incl` is C's signed remainder; the cursor is only ever 0 or the result, so it stays in 0..7 | 0x6b8c9e..0x6b8cab |
| predicate | advances on every accepted request, HELP included | same | agree | **confirmed**: both arms join at 0x6b8c98 | 0x6b8c80 `jmp 0x6b8c98` |
| reset | `Leader::init` 0x6e3a09 store and memset; `Leaders::clear`; (`Leader::close`) | same | agree | **confirmed** | 0x6e3a09 `movl %edx, 0x3f4(%ebx)`, `edx = 0` (0x6e39ff) |
| readers | `receive_taunt` only; the drain ignores it | same | agree | **confirmed**: no read of +0x3f4 off a leader outside `receive_taunt` in the register net | §0.N |
| non-leader `0x3f4` stores | `Constants::init`, `ConquestLeader`, `RoadsOut`, `RTreeData`, … | (`field_0x3f4` other classes) | agree | **confirmed not leaders**; `RTreeData::try_wcoord_entry` 0x9234ca and `Constants::init` 0x56c1eb are their own records | §0.N |
| crate | as §3 (`checksum/leaders.rs:92`) | same | agree | **confirmed** | `crates/sim/src/checksum/leaders.rs:92` |
| plain game | as §1 | same | agree | **confirmed** | §1 |

**The rule as a builder should port it.** `incoming: i32`, zero at game
start; after the three slot stores of an accepted request (§3's gate),
`incoming = (incoming + 1) & 7`. Read only by `receive_taunt` as the slot
index. Never reset in a match. Crate site and plain game as §3.
**Confidence: high.** Open: nothing.

## 7. Against the README row

The row names `receive_taunt`, `process_taunt`, `end_process`. Both
readings drop `end_process`, and **that is confirmed**: `Leader::end_process
(006b8690)` and `Leaders::end_process_all (006ed070)` store at `0x3bc` off
the `0x8c`-stride `GameInfo` player record (0x6b87b4, 0x6ed1a3; the same
base takes 0x330 and 0x448), never off a leader. The row's missing reset
writers are §0.R.

## Verdict table

| leaf | confirmed / corrected / open | the rule in one line |
|---|---|---|
| `last_taunt` +0x354 | 11 / 0 / 0 | `process_taunt`: if R active, not human, S ≠ R, mutual ally now, `last_taunt[S] = req` (before the switch, any value); read by `action_ping`'s `== 16` test |
| `taunt_frame` +0x374 | 7 / 0 / 0 | same store, `taunt_frame[S] = frame`; `action_ping` answers while `frame <= taunt_frame[P] + 225` |
| `incoming_taunt` +0x394 | 12 / 1 / 0 | `receive_taunt`: if `req > 0`, S active, R active non-human, `is_ally(R,S)`, R ≠ S, `incoming_taunt[incoming] = req`; delivery gate corrected (§0.4) |
| `incoming_taunt_who` +0x3b4 | 8 / 0 / 1 | same gate, `incoming_taunt_who[incoming] = S` (leader `who`); range of S open, immaterial |
| `incoming_taunt_frame` +0x3d4 | 11 / 0 / 0 | same gate, `= frame` for HELP, `frame + 60` otherwise; drained by equality, slot order, never on frame 0 |
| `incoming` +0x3f4 | 8 / 0 / 0 | `(incoming + 1) & 7` per accepted request |
| shared (§0) | 8 / 2 / 2 | delivery gate (0.4) and `check_for_order`'s 0 return (0.7) corrected; sender range (0.11) and `Setup::build_leader` mid-match (0.R) open |

Counts are per claim row above. The corrections are upstream of the leaves
(the delivery gate and the return code) and change no leaf's store; the one
that changes what a builder writes is §0.4: the masked chat arm has no
sender-side `chat_status` test, a masked cheat line outside multiplayer
delivers nothing, and a crossplay-blocked sender delivers nothing.
