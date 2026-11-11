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

