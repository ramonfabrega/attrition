# Item 4152, family 9 (`taunts`) — first reading

- Family: 9, slug `taunts`.
- Leaves (`LeaderData` offsets, `types.txt` `struct /rise.pdb/LeaderData`,
  size `0x6ee4`): `last_taunt` (+0x354, `TauntRequest[8]`), `taunt_frame`
  (+0x374, `int[8]`), `incoming_taunt` (+0x394, `int[8]`),
  `incoming_taunt_who` (+0x3b4, `int[8]`), `incoming_taunt_frame` (+0x3d4,
  `int[8]`), `incoming` (+0x3f4, `int`).
- Reader: first reading, wave 1, model Claude Opus 5.5 (`claude-opus-5-5`),
  2026-11-11.
- Sources: the decompile export (`~/ghidra-projects/decomp`), the listing of
  `game/riseofnations.exe` (`llvm-objdump`), `crates/sim/src`.

## 0. The base, and the mechanism the six leaves share

**Base.** `Leader` is 0x6eec bytes and `LeaderData` 0x6ee4, but `LeaderData`
sits at **offset 0** inside `Leader`, not behind a prefix: in every `Leader`
method read here `*(uint*)this` is `leader_flags` and `this->field_0x8` is
`who`, and `Leader::process_taunt (006b8cc0)` reaches
`iVar7·0x6eec + 0xe3a404` for `leaders.list[who].field_0x74` (0xe3a404 −
0xe3a390 = 0x74, `diplos`). So a `this->field_0xNNN` in a `Leader` method is
`LeaderData + 0xNNN`; no offset shift applies to this family.

**`TauntRequest`** (PDB, `LF_ENUM` 0x4B16, field list 0x4B15, underlying
`int`): `TAUNT_NONE` 0, `FOOD` 1, `TIMBER` 2, `WEALTH` 3, `METAL` 4, `OIL` 5,
`NEED` 6, `BUILD_WONDER` 7, `BUILD_GROUND` 8, `BUILD_SEA` 9, `BUILD_AIR` 10,
`BUILD_INFRA` 11, `RUSH` 12, `BOOM` 13, `ATTACK` 14, `DEFEND` 15, `HELP` 16,
`NUM_TAUNTS` 17. `types.txt` names the element type of `last_taunt` but has no
enum body; the PDB type stream does.

**`leader_flags` bits used below** (`docs/AI.md` §1, not re-derived here):
1 = valid (seated), 2 = active, 4 = human.

**The pipeline** (one chain; every writer of the six leaves is on it):

1. *Sender, UI only.* `LeaderOut::issue_chat (006d2c50)` takes the typed
   line, strips a `;` / `:` / `@` prefix (recipient mode 2 / 3 / 5), and calls
   `Taunts::check_for_order (009809d0)` on the rest. That parses the text as
   an integer, finds the taunt with that id, and maps its order string to a
   `TauntRequest` (1–5, 7–16; 6 by its last compare). It returns −1 when no
   taunt has the id or the taunt has no order string, and 0 when the order
   string matches none of the names. The result rides as the `taunt` word of
   `CommandManager::issue_chat (00942b10)` → `CommandPackage::add_chat
   (0094c080)`, opcode `'D'` (0x44), layout `ChatCommand` (`bits` +1,
   `taunt` +5, `taunt_num` +9, `len` +0xd, text +0x11). An empty line (after
   trimming spaces) is dropped by both `issue_chat` and `add_chat`, so a
   request needs text; `issue_chat` replaces a short numeric line (< 4
   characters) by the taunt's own text, so the text is never empty for a
   valid id.
   Callers of `LeaderOut::issue_chat` (listing `calll 0x6d2c50`):
   `IFaceDiploBase::on_button_clicked (008014e0)` (0x801722),
   `IFaceDiploBase::on_key_click (008017f0)` (0x80197c), `ChatBox::on_modal_end
   (0093d560)` (0x93d6f6). `ChatBox::on_modal_end`'s other path, the cheat
   line at 0x93d666, calls `CommandManager::issue_chat` with `taunt` = 0
   (`TAUNT_NONE`, pushed `$0x0` at 0x93d65f) — never a request.
   `CommandPackage::add_chat`'s only caller is `CommandManager::issue_chat`
   (0x942c10). **No AI function issues a chat command**: every producer is a
   window of the local human. A computer's own words go through
   `Leader::chat_to_local (006ec520)`, which is display only.
2. *Lockstep delivery.* `CommandPackage::process_chat (009454f0)` (dispatched
   from `CommandPackage::process (0094a700)` case `'D'`) loops the eight
   slots. For each slot with `leader_flags & 1`, subject to the chat-status
   gates (`chat_status` +0x54: the sender's entry for the receiver 0 and the
   receiver's entry for the sender not 2, or `semaphore[0] & 0x40`; with a
   recipient mask, the receiver's bit set), it calls
   `Leader::receive_chat (006b8ab0)`, then `Leader::receive_taunt (006b8bf0)`
   **if `taunt >= 0`** (0x94573a and 0x945870, one per mask arm).
3. *Queue.* `receive_taunt` writes one slot of an 8-entry ring (§§3–6
   below).
4. *Dispatch.* Every frame, `Leaders::process_all (006ed2a0)` — the live
   copy; `Leader::process (006b88b0)` and `Leader::process_taunts (006b8ba0)`
   hold the same loop but have **no caller** in the listing (no `call`/`jmp`
   to either address; their only `calll 0x6b8cc0` sites are inside
   themselves, 0x6b89f4 and 0x6b8bd2) — scans the ring of every slot with
   `leader_flags & 2`, **when `game->frame != 0`**, and for each of the 8
   entries whose `incoming_taunt_frame == game->frame` calls
   `Leader::process_taunt(this, incoming_taunt[i], incoming_taunt_who[i])`.
   The entry is **not cleared** after dispatch; equality fires it once.
5. *Act.* `process_taunt` records `last_taunt` / `taunt_frame` and acts
   (§§1–2).
6. *Consume.* `Leader::action_ping (006d18a0)` reads `last_taunt` /
   `taunt_frame` (§1).

**Order within a frame.** `Game::loop (00591570)` runs
`TurnControl::do_frame (00957dd0)` — whose solo and multi arms call
`CommandManager::process_turn` — before `Game::do_frame (00591ef0)`, and
`Game::do_frame` increments `frame` at its line 293, after
`Leaders::process_all` at line 199. So a request received while
`game->frame` = F is in the ring before that frame's `Leaders::process_all`
runs with the same F. That is what makes `TAUNT_HELP`'s zero delay (§5) fire
on the receiving frame. *Reading only*; the solo path calls `process_turn`
also on loop passes that do not step a frame, which does not change F.

