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
| 0.3 | `receive_taunt`'s only callers are `CommandPackage::process_chat (009454f0)`, one per arm, when `taunt >= 0` | yes | yes | agree | **confirmed** | `calll 0x6b8bf0` occurs at 0x94573a and 0x945870 only; `funcs/CommandPackage/process_chat@009454f0.c` `if (-1 < *local_28)` before each call |
| 0.4 | `process_chat`'s delivery gate | broadcast: sender's `chat_status[recv]` = 0 and receiver's `chat_status[sender]` != 2, each forgiven by `semaphore & 0x40`; masked: receiver's bit set | `chat_status` not 2; masked: not a cheat line in a non-network game | disagree in detail | **corrected** (both partial). For each valid slot k: **broadcast** (`bits == -1`): (`leaders[S].chat_status[k] == 0` or `sem & 0x40`) and (`leaders[k].chat_status[S] != 2` or `sem & 0x40`). **Masked**: (`bits` bit k or `sem & 0x40`); then, outside multiplayer (`sem & 4` clear), a line starting with the cheat string goes to `ConsoleWin::parse_cmd` and nothing is delivered; else (`leaders[k].chat_status[S] != 2` or `sem & 0x40`). The masked arm has **no** sender-side test. A further gate neither reader named: when the sending player has a `net_player` that `Crossplay::Service()` reports blocked, nothing is delivered to anyone. | `funcs/CommandPackage/process_chat@009454f0.c` lines read by `grep -n 'local_2c == 0xffffffff'` and the `LAB_00945817` arm; the crossplay `goto LAB_0094563c` with `bVar2 = true` |
| 0.5 | No computer issues a chat command; every producer is a window of the local human | yes | yes | agree | **confirmed**. `CommandManager::issue_chat (00942b10)` has two direct callers, 0x6d3205 (`LeaderOut::issue_chat`) and 0x93d666 (`ChatBox::on_modal_end`'s cheat path); `LeaderOut::issue_chat` three, 0x801722, 0x80197c, 0x93d6f6; `CommandPackage::add_chat` one, 0x942c10; none of the four addresses occurs as a 32-bit word anywhere in the PE (no vtable or pointer). The second's "`CommandManager::issue_chat` has only the two interface callers" is right in count; the two are `LeaderOut::issue_chat` and `ChatBox`. | the `calll` census above |
| 0.6 | The cheat path pushes `taunt = 0` | yes | (silent) | one-sided | **confirmed**: `add_chat`'s `param_2` is the `taunt` word (+5), and `issue_chat`'s third argument feeds it; 0x93d65f `pushl $0x0` is that argument. `receive_taunt` is then called (0 >= 0) and rejects it at 0x6b8c0f. | 0x93d65d..0x93d666; `funcs/CommandPackage/add_chat@0094c080.c` `*(int *)(this->data + iVar4 + 5) = param_2` |
| 0.7 | `Taunts::check_for_order (009809d0)`: −1 when no taunt has the id or its order string is empty, 0 when the order string matches no name, else 1..16 | yes | "−1 otherwise" | disagree (minor) | **corrected toward the first**: −1 for no id / empty order string, **0** for an unmatched string, else the request. Neither reader noted that **two** names map to `HELP` (the compares at string offsets `+0x2620` and `+0x1f1f8`). Harmless to the leaves: 0 and −1 are both refused or never delivered. | `funcs/Taunts/check_for_order@009809d0.c`, the `return -1` and the `iVar2 = 0x10` after the two-name test |
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

