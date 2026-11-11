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

## 1. `last_taunt` (+0x354, `TauntRequest[8]`, indexed by the sender's `who`)

**Writers.**
- Export, `field_0x354`: `Leader::process_taunt (006b8cc0)` (store),
  `Leader::action_ping (006d18a0)` (read, `leaders.list[0].field_0x354 +
  (param_3 + who)·4`). The other hits — `GameReportWin::close`,
  `RiverSection::init`, `ConquestInvadeWin::close` — have a `this` of
  their own class, not a leader. `->last_taunt`: no hit (the export never
  prints the name).
- Listing, absolute `0xe3a6e4` (= 0xe3a390 + 0x354): one site, 0x6d1977,
  `cmpl $0x10, 0xe3a6e4(,%ecx,4)` in `action_ping` — a read.
- Listing, register-relative `0x354(%reg…)`: 18 stores in `.text`; the one
  whose base is a `Leader*` is **0x6b8d46** `movl %ecx, 0x354(%esi,%ebx,4)`
  in `process_taunt` (`esi` = `this`, from `ecx` at entry; `ebx` = arg 2).
  The other 17 are in `ConquestMapLoader`, `ConquestLeader`, `PlayerProfile`,
  `Constants::init`, `Camera`, `TileSetData`, `RiverSection`, `IFaceButton`,
  `WorldMapBackground` (float 0x3f800000 = 1.0f) and
  `GameSpyGroup::OnDownloadScores` (indexed, 0x96c1ee) — none a leader base.
- Bulk zeroing: `memset(this, 0, 0x6929)` (listing `pushl $0x6929; xorl
  %eax,%eax; pushl %eax; pushl %ebx; calll 0x55e0b2`) in `Leader::init
  (006e3930)` at 0x6e3995, `Leaders::clear (006ed5d0)` at 0x6ed5ee and
  `Leader::close (006b7f80)` at 0x6b7f8c; 0x6929 bytes from +0 cover
  +0x354..+0x373.
- Bulk restore: a savegame load walks `WalkDataGame::walk_data (005a2360)`
  → `Leaders::walk_data` → `LeaderData::walk_data (006d6750)`, whose range
  [+0x8, +0x692a) (`docs/CHECKSUM.md` §8) holds this leaf. Not read in load
  mode here; stated from the checksum's use of the same walker.
- `ScenarioRead::load_leader_chunk (009ac080)` and
  `ScenarioWrite::save_leaders_chunk (009a77c0)`: no reference to +0x354.

**Event, step, predicate.** `process_taunt(this = receiver R, req, S)` with
S the sender's `who` (from the ring). It stores `last_taunt[S] = req` and
`taunt_frame[S] = game->frame` (§2) **before** acting on `req`, after these
gates, in this order (listing 0x6b8ce0–0x6b8d36):
1. `leaders[R.who].leader_flags & 4` clear (R is not human) and `& 2` set
   (R active) — else return, nothing stored.
2. If S ≠ R.who: `leaders[R].diplos[S] == 2` **and**
   `leaders[S].diplos[R] == 2` (mutual alliance, re-checked at dispatch, not
   only at receipt) — else return.
3. S ≠ R.who — else return (the self case skips gate 2 and dies here).

No bound on `req` here: whatever the ring holds (1 and up; `receive_taunt`
rejects 0 and negatives but has no upper bound) is stored, and values
above 16 fall to the switch's `default` (no action). From the UI,
`check_for_order` yields 1–16. The step is an overwrite, not an increment:
the last dispatched request from S, values 1–16. Units: an enum.

What `process_taunt` does after the store (not this family's leaves, named
for the reader): `req` 1–5 (a good: food, timber, wealth, metal, oil) — if
both R and S have the good available (`type_avail(...) == 4`), and, unless
teams are locked and R is not neutral, R's `gift_stamp[S]` (+0x194) is 0 or
`frame − gift_stamp[S] ≥ 0x1194` (4,500 frames), and R holds more than 0x95
(149) of it (the `LeaderDataEncrypt` word `^ 0x8221`), R stamps
`gift_stamp[S]`, clears its pending actions with S and offers S a third of
the stock (`action_offer(S, good, stock/3)` then `action_respond(S, 1)`);
otherwise R chats a refusal to the local player. `req` 6: picks R's
scarcest good (available to both, not knowledge) and, if S is the local
player, plays a taunt line — no state. `req` 7–15 rescale or set R's
production-AI weights at +0x794..+0x7a8 and +0x6dec, clamped after (a
separate family). `req` 16 (`HELP`): no action and no message.

**Reset.** `Leader::init` (when its second argument is negative —
`Leaders::init (006ed8cb site)` calls `Leader::init(i, -1, i)` for all eight
from `Game::init_rules_and_teams (00589bb0)`; `Setup::build_game`'s call at
0x5acef7 passes the slot's `tribe`, ≥ 0, and skips the clear),
`Leaders::clear` (called from `Game::close` and `Game::run_oos_recovery`),
`Leader::close` (no direct caller found). No per-frame fold; never reset
during a match.

**Readers.** `Leader::action_ping (006d18a0)` only: when a **human** leader
P pings a point (the `param_3 == P.who` arm, reached from
`CommandPackage::process_ping`, call 0x9454bb), every other valid,
non-human leader A that is mutually allied with P (`is_ally`) and has
`last_taunt[P] == 16` (`HELP`) **and** `game->frame ≤ taunt_frame[P] + 225`
(listing 0x6d1991–0x6d199d: `jl` skips when `taunt_frame + 0xe1 < frame`)
sends a useful army (`Armies::find_useful_army`) to the ping with
`Army::send_here(..., MOVE_TO)` and stamps that army's +0x28 = 0xa8c
(2,700). **This changes a decision**: it is the only consumer, and it moves
an AI army. The checksum walker (`LeaderData::walk_data`) reads the leaf
whole.

**Crate.** No event. The sim has no chat command, no ring, no
`process_taunt`: `crates/rondata/src/input.rs:1611` skips `Command::Chat`
as "a cheat line, not an order", and `crates/sim/src/checksum/leaders.rs:87`
lists the leaf, which `leader_long` answers `None`
(`crates/sim/src/checksum/leaders.rs:1305`). A builder adds a per-leader
`last_taunt: [i32; 8]` stored by a new `process_taunt` called from the
per-player loop of `Sim::tick` (`crates/sim/src/core/frame.rs:94`, the
`Leaders::process_all` analogue), and feeds it from a chat arm in
`rondata::input`'s `one` (`crates/rondata/src/input.rs:1503`) that calls a
new `receive_taunt`. There is no `action_ping` either (`Command::Ping` is
decoded in `crates/rondata/src/commands.rs:857` and falls to `"not mapped"`).

**Plain game?** Reachable without a scenario or CtW, but only when a human
has a **computer ally** (mutual `diplos == 2`, so team play) and types a
taunt number whose `taunts.xml` entry carries an order string, or uses the
diplomacy window's chat box the same way. A 1-v-1 against the computer, or
any game where no human sends such a line, never writes it: the value stays
0 from `Leader::init`. No computer sends a request.

**Confidence.** High for the store, its gates and the reset (export and
listing agree; every register-relative store attributed). Medium for "only
from the UI" — it rests on `CommandPackage::add_chat` having one caller and
on the three `LeaderOut::issue_chat` callers all being windows; a replayed
command stream (a recorded game) carries whatever its `taunt` word says.
Not read: `Taunts::init (00980df0)` (where the order strings come from),
`IFaceDiploBase::on_key_click` body, `LeaderData::walk_data` in load mode,
`Leader::close`'s callers through pointers.

## 2. `taunt_frame` (+0x374, `int[8]`, indexed by the sender's `who`)

**Writers.**
- Export, `field_0x374`: `Leader::process_taunt (006b8cc0)` (store),
  `Leader::action_ping (006d18a0)` (read). Other hits (`GameReportWin::close`,
  `ConquestInvadeWin::close`, `RiverSection::init`) are other classes.
- Listing, absolute `0xe3a704`: one site, 0x6d198a `movl
  0xe3a704(,%ecx,4), %ecx` in `action_ping` — a read.
- Listing, `0x374(%reg…)`: 12 stores; the leader one is **0x6b8d54**
  `movl %eax, 0x374(%esi,%ebx,4)` in `process_taunt`, `eax` =
  `[0xc061ec]+0x550` = `game->frame`. The rest: `ConquestMapLoader`,
  `GraphicWinXML`, `PlayerProfile`, `ConquestLeader` (a `movw`), `TileSet`,
  `ChatSpace`, `JukeBox` (a code pointer), `IFaceDiplo` (`movups`, a window).
- Bulk: the same three `memset(…, 0x6929)` as §1; the savegame walker.

**Event, step, predicate.** The same store as §1, one instruction later,
same gates: `taunt_frame[S] = game->frame` (sim frames, absolute) at the
moment of dispatch. For `HELP` that is the frame the chat command was
processed; for every other request it is that frame + 60 (§5). Overwrite,
not increment.

**Reset.** As §1: `Leader::init(…, -1, …)`, `Leaders::clear`,
`Leader::close`; never during a match.

**Readers.** `action_ping` only (§1): the window `frame ≤ taunt_frame[P] +
225` for an AI ally's army to answer a human's ping after `HELP`. 225
frames, sim frames. The checksum walker.

**Crate.** None; as §1. A builder stores it beside `last_taunt` in the same
new `process_taunt`.

**Plain game?** As §1.

**Confidence.** High; one store, attributed by both nets. Not read: as §1.

## 3. `incoming_taunt` (+0x394, `int[8]`, the ring's request word)

**Writers.**
- Export, `field_0x394`: `Leader::receive_taunt (006b8bf0)` (store).
  `Leader::init` clears it as `puVar23[-8]` off a pointer to +0x3b4, which
  the export prints without the field name. Other hits are other classes.
- Listing, absolute `0xe3a724`: no site.
- Listing, `0x394(%reg…)`: 16 stores; the leader one is **0x6b8c4f** `movl
  %ebx, 0x394(%esi,%eax,4)` in `receive_taunt` (`esi` = `this`, `eax` =
  `incoming`, `ebx` = arg 1). The rest: `ConquestMapLoader`,
  `GraphicWinXML`, `RadioButton` (a pointer constant 0xed88d0), `EditGroup`,
  `Constants::init`, `PlayerProfile` (a `movb`), `ConquestLeader`,
  `ScenarioRead::load_leaders_chunk` (0x9ad0b7, base `edx` is a 0x8c-stride
  per-player record, not a leader).
- Pointer-based, invisible to the displacement net: `Leader::init`'s loop at
  0x6e3a10 `movl %edx, -0x20(%eax)` with `eax` = `this + 0x3b4` (+4 per
  pass, 8 passes) — zero.
- Bulk: the three `memset`s, the savegame walker.

**Event, step, predicate.** `receive_taunt(this = R, req, play)` from
`CommandPackage::process_chat` when the chat command's `taunt ≥ 0`. S =
`game->info.player[play].who` (a byte at player record +0x77, stride 0x8c,
listing 0x6b8bf3–0x6b8c08). Stored only when all hold (listing
0x6b8c0d–0x6b8c47):
1. `req ≠ 0` and `req ≥ 0` (`TAUNT_NONE` and negatives rejected; no upper
   bound);
2. `leaders[S].leader_flags & 2` (sender active);
3. `R.leader_flags & 4` clear (receiver **not human**) and `& 2` set
   (receiver active);
4. `LeaderData::is_ally (006edb50)(R, S)` — true when S = R or both
   `diplos` entries are 2;
5. `R.who ≠ S`.

Then `incoming_taunt[incoming] = req`. Overwrite of one ring slot; units
an enum.

**Reset.** `Leader::init(…, -1, …)` twice over (the `memset`, then the
explicit loop), `Leaders::clear`, `Leader::close`. **Not cleared on
dispatch**: after `process_taunt` fires the slot keeps its old `req` until
the ring comes round to it again, so it is part of the checksummed state for
the rest of the match.

**Readers.** `Leaders::process_all (006ed2a0)`: `pushl -0x20(%edi)` at
0x6ed3f1 with `edi` = `this + 0x3b4 + 4i` — passes it as `process_taunt`'s
`req`. (The dead `Leader::process` and `Leader::process_taunts` read it the
same way.) The checksum walker.

**Crate.** None. A builder adds the ring (`incoming_taunt`,
`incoming_taunt_who`, `incoming_taunt_frame`, `incoming`) to the per-leader
state and a `receive_taunt` reached from a chat arm replacing the skip at
`crates/rondata/src/input.rs:1611`; the dispatch scan goes in `Sim::tick`'s
per-player loop (`crates/sim/src/core/frame.rs:94`), after the
timer-refresh work, matching its place at the tail of `Leaders::process_all`.

**Plain game?** As §1 (human sender, computer receiver, mutual alliance).

**Confidence.** High for the store and gates. Not read: whether a command
stream can carry a `taunt` above 16 in practice (only a crafted or foreign
packet could).

## 4. `incoming_taunt_who` (+0x3b4, `int[8]`, the ring's sender word)

**Writers.**
- Export, `field_0x3b4`: `Leader::receive_taunt` (store), `Leader::init`
  (the loop's `*puVar23 = 0`), and the readers `Leaders::process_all`,
  `Leader::process`, `Leader::process_taunts`. `RoadsOut::one_time_init` is
  another class.
- Listing, absolute `0xe3a744`: no site.
- Listing, `0x3b4(%reg…)`: 14 stores; the leader one is **0x6b8c5c** `movl
  %edi, 0x3b4(%esi,%eax,4)` in `receive_taunt` (`edi` = S). The rest:
  `ConquestMapLoader`, `IFaceRenderManager`, `EditGroup`, `Constants::init`,
  `Game::run_oos_recovery` and `Game::close` (0x586be3, 0x594d34 — base
  `eax` in a 0xc4-stride clear of another array, not a leader),
  `ConquestLeader`, `IFaceButton`, `RoadsOut`,
  `ScenarioRead::load_leaders_chunk` (0x9ad092, the 0x8c-stride record).
- Pointer-based: `Leader::init` 0x6e3a13 `movl %edx, (%eax)` — zero.
- Bulk: the three `memset`s, the savegame walker.

**Event, step, predicate.** Same call and gates as §3; stores S, the
sender's `who` (0–7), into the same slot. Overwrite.

**Reset.** As §3; not cleared on dispatch.

**Readers.** `Leaders::process_all`: `pushl (%edi)` at 0x6ed3ed, passed as
`process_taunt`'s S — which then indexes `last_taunt` and `taunt_frame` and
picks the ally the action is aimed at. The checksum walker.

**Crate.** None; as §3.

**Plain game?** As §1.

**Confidence.** High. Not read: as §3.

## 5. `incoming_taunt_frame` (+0x3d4, `int[8]`, the ring's due frame)

**Writers.**
- Export, `field_0x3d4`: `Leader::receive_taunt` (two stores, one per
  branch); `Leader::init` clears it as `puVar23[8]`. Other hits are other
  classes.
- Listing, absolute `0xe3a764`: no site.
- Listing, `0x3d4(%reg…)`: 6 stores; the leader ones are **0x6b8c79** and
  **0x6b8c91** in `receive_taunt`. The rest: `GraphicWinXML`, `SetupWin`
  (a `movb`), `RiverSectionOut`, `RoadsOut`.
- Pointer-based: `Leader::init` 0x6e3a18 `movl %edx, 0x1c(%eax)` after the
  `+4` (so `+0x3b4 + 4 + 0x1c` = `+0x3d4 + 4i`) — zero.
- Bulk: the three `memset`s, the savegame walker.

**Event, step, predicate.** Same call and gates as §3. The value:
- `req == 16` (`TAUNT_HELP`, listing 0x6b8c68 `cmpl $0x10, %ebx`):
  `game->frame` — due the same frame;
- any other `req`: `game->frame + 60` (`addl $0x3c`, 0x6b8c8e) — due 60 sim
  frames later.

Because dispatch tests `incoming_taunt_frame == game->frame` and skips
**frame 0** entirely (`cmpl $0x0, 0x550(%ecx)` at 0x6ed3ce), a `HELP`
received on frame 0 is never dispatched. A slot overwritten by a ninth
request before its due frame is lost: the ring has 8 entries and no
occupancy check. Overwrite.

**Reset.** As §3; not cleared on dispatch (the equality makes a second
firing impossible, the value stays).

**Readers.** `Leaders::process_all`: `movl 0x20(%edi), %eax; cmpl
0x550(%ecx), %eax` at 0x6ed3e2 — the due test. The checksum walker.

**Crate.** None; as §3. The sim's `frame` counter (`Sim::tick`'s `let
frame = self.frame`) is the value to compare.

**Plain game?** As §1.

**Confidence.** High for both constants and the branch (listing and export
agree). Medium for the same-frame claim for `HELP` (the frame order in §0,
read from `Game::loop` and `Game::do_frame`, not traced in a run).

## 6. `incoming` (+0x3f4, `int`, the ring's write cursor)

**Writers.**
- Export, `field_0x3f4`: `Leader::receive_taunt` (read ×5, store ×1),
  `Leader::init` (store 0). The other 14 hits (`TerrainOut`, `RoadsOut`,
  `TerrainVis`, `CoordInfo`, `GameReportWin`, `ConquestInvadeWin`) are other
  classes.
- Listing, absolute `0xe3a784`: no site.
- Listing, `0x3f4(%reg…)`: 8 stores; the leader ones are **0x6b8cab** `movl
  %eax, 0x3f4(%esi)` in `receive_taunt` and **0x6e3a09** `movl %edx,
  0x3f4(%ebx)` (`edx` = 0) in `Leader::init`. The rest: `GraphicWinXML`
  (a `movb`), `Constants::init`, `ConquestLeader`, `RiverSectionOut`,
  `RoadsOut` (float bits 0xbf800000 = −1.0f), `RTreeData`.
- Bulk: the three `memset`s, the savegame walker.

**Event, step, predicate.** Same call and gates as §3; after the three
slot stores, `incoming = (incoming + 1) mod 8` (`incl; andl $0x80000007;
jns …; decl; orl $-8; incl` — C's signed `% 8`; with the cursor always in
0–7 it is `+1 & 7`). Step +1, wrapping at 8. Advances on every accepted
request, `HELP` included.

**Reset.** `Leader::init(…, -1, …)` (the `memset` and the explicit store at
0x6e3a09), `Leaders::clear`, `Leader::close`. Not reset by dispatch.

**Readers.** `receive_taunt` only (the slot index). Dispatch scans all 8
slots and does not read the cursor. The checksum walker.

**Crate.** None; as §3.

**Plain game?** As §1.

**Confidence.** High.

