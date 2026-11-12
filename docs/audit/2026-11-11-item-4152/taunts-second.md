# Item 4152, second reading: `taunts`

- **Slug**: `taunts`
- **Leaves** (`LeaderData`, size `0x6ee4`): `last_taunt` (+0x354, `TauntRequest[8]`),
  `taunt_frame` (+0x374, `int[8]`), `incoming_taunt` (+0x394, `int[8]`),
  `incoming_taunt_who` (+0x3b4, `int[8]`), `incoming_taunt_frame` (+0x3d4, `int[8]`),
  `incoming` (+0x3f4, `int`).
- **Reader**: Claude Opus 5.5 (`claude-opus-5-5`), blind second reading, 2026-11-11.
- **Sources**: the decompile export (`~/ghidra-projects/decomp`), `types.txt`,
  the full `.text` listing at the job's `aa/exe.s`.

## 0. What the nets were, and what is common to all six leaves

**Layout.** `Leader` (`0x6eec`) is `LeaderData` (`0x6ee4`) at offset 0 plus a
vbtable pointer at `+0x6ee4`: `Leader::Leader (006d2bf0)` stores the vbtable
at `this+0x6ee4` and calls `LeaderData::LeaderData(this)` on the same
pointer, and every `Leader` method reads `who` as `this->field_0x8`. So a
`Leader*` and a `LeaderData*` have the **same** offsets for these leaves; no
shift. `leaders.list` is an inline array of `Leader` at `0xe3a390`, stride
`0x6eec`, ten records (`Leaders::init (006ed850)` runs to `0xe7f8c8`); the
per-frame loops run the first eight (to `0xe71af0`).

**`TauntRequest`** (PDB `LF_ENUM 0x4B16`, field list `0x4B15`): `TAUNT_NONE`
0, `FOOD` 1, `TIMBER` 2, `WEALTH` 3, `METAL` 4, `OIL` 5, `NEED` 6,
`BUILD_WONDER` 7, `BUILD_GROUND` 8, `BUILD_SEA` 9, `BUILD_AIR` 10,
`BUILD_INFRA` 11, `RUSH` 12, `BOOM` 13, `ATTACK` 14, `DEFEND` 15, `HELP` 16,
`NUM_TAUNTS` 17. Underlying `int`, so `last_taunt` is eight 4-byte words.
Leader flag bits used below (PDB enumerates): `LEADER_VALID` 1,
`LEADER_ACTIVE` 2, `LEADER_HUMAN` 4. `diplos` value 2 is `DIPLO_ALLY`.

**Export net.** `grep -rn` for the six names over `funcs/` finds nothing (the
decompiler prints every access as `field_0xNNN`). `field_0x354/374/394/3b4/
3d4/3f4` hits inside `Leader`/`Leaders` methods: `Leader::receive_taunt
(006b8bf0)`, `Leader::process_taunt (006b8cc0)`, `Leader::process_taunts
(006b8ba0)`, `Leader::process (006b88b0)`, `Leaders::process_all (006ed2a0)`,
`Leader::init (006e3930)`, `Leader::action_ping (006d18a0)`. All other hits
are other classes (`GameReportWin`, `ConquestInvadeWin`, `RiverSection`,
`RoadsOut`, `TerrainOut`, `CoordInfo`, `TerrainVis`).

**Listing net** (`aa/exe.s`, every `0x354..0x3f7(%reg…)` operand mapped to
its function through `INDEX.tsv`, 1,114 operands): inside
`Leader`/`Leaders`/`LeaderData` the operands that land in the family are only
those of `receive_taunt`, `process_taunt`, `process_taunts`, `process`,
`process_all` and `init`. The other in-range operands in those classes were
each run down to a non-`Leader` base: `Leader::end_process (006b8690)` and
`Leaders::end_process_all (006ed070)` `0x3bc(%edx)` walk a `0x8c`-stride
record (`0x2a4`, `0x330`, `0x3bc`); `Leader::init`'s `0x3f0(%edx)` is
`ebx+0x480+4k` (`0x874..`), its and `Leader::plan_strategy (006b9620)`'s
`0x37a(%edx|%ecx)` are word stores into a per-type array; 
`Leader::production_ai_setup (006c83e0)` `0x3a4(%esi,%ebx)` has `ebx` in
`0xac..0xc0` (`0x450..0x467`); `calc_pop_cap`, `compute_build_score`,
`calc_science_discount`, `get_caravan_limit` read `0xc061f0`/`0xc061e4`
(the constants), not a leader; `LeaderData::log_data`'s are `-0x3xx(%ebp)`
locals. Absolute `0xe3a6e4..0xe3a787` (slot 0 + the family) occurs exactly
twice, both reads in `Leader::action_ping` (`6d1977`, `6d198a`). Out-of-class
in-range operands were sampled by class (`PlayerProfile`, `Camera`,
`Options`, `ScenarioFuncSet`, `ScenarioRead::load_leaders_chunk` = the game
info `player[]` records, `Game::close`/`run_oos_recovery` = a `0x1c`-stride
string table, `IFaceDiplo`, `Unit::plunder`, `Console*`): none has a
`Leader` base.

**Block writers** (no field offset appears; found by the size immediates
`0x6929`, `0x6eec`, `0x6ee4`, `0x1bbb`, `0x1bb9`, `0x1a4a`):

- `memset(this, 0, 0x6929)` — clears `+0x0..+0x6928`, so all six leaves — in
  `Leader::init (006e3930)` on its `param_2 < 0` branch (`6e398c`),
  `Leader::close (006b7f80)` (`6b7f82`) and `Leaders::clear (006ed5d0)`
  (`6ed5e0`, all ten slots).
- `LeaderData::walk_data (006d6750)` walks the byte range `[&who (+0x8),
  +0x692a)` as one block for a `LEADER_VALID` leader, so all six leaves are
  in it. Callers: `CheckSums::check_all (00936560)`,
  `CheckSums::check_one_leader (00937020)`, `CheckSums::check_leaders
  (009375a0)`, `Leaders::walk_data (006e38e0)`, `SaveGame::verify_save
  (005a76b0)`, `LoadGame::verify_load (005a39f0)`. With an input
  `DataWalk` (load) it is a writer; with the checksum walk a reader.

**Callers of the block clears.** `Leader::init(i, -1, i)` is run for every
slot by `Leaders::init (006ed850)`, whose one caller is
`Game::init_rules_and_teams (00589bb0)` — every game start. The later
`Setup::build_game (005ac190)` call passes the slot's tribe (`>= 0`), so it
takes the other branch and does not touch the family.
`WalkDataGame::walk_data (005a2360)` runs `Leader::init(slot 9, 9, -1, 9)`.
`Leaders::clear` is called from `Game::close (00594b80)` and
`Game::run_oos_recovery (005869e0)`. `Leader::close`, `Leader::process` and
`Leader::process_taunts` have **no** direct call, no `jmp`, and their
addresses occur nowhere in the PE (byte search): their bodies live inlined in
`Leaders::clear` and `Leaders::process_all`.

**Frame order.** `Game::loop (00591570)` calls `TurnControl::do_frame
(00957dd0)` (→ `do_frame_solo`/`do_frame_multi` → `CommandManager::
process_turn (0093ef10)` → `CommandPackage::process_all (0094c500)` →
`CommandPackage::process (0094a700)` → `CommandPackage::process_chat
(009454f0)` → `receive_taunt`) **before** `Game::do_frame (00591ef0)`, which
runs `Leaders::process_all` early and increments `game->frame` (`Game+0x550`)
at its end. So a command applied while `game->frame == F` is seen by that
same frame's `Leaders::process_all` with the same `F`.

## 1. `last_taunt` (`+0x354`, `TauntRequest[8]`, indexed by sender slot)

1. **Writers.**
   - `Leader::process_taunt (006b8cc0)` — `6b8d46 movl %ecx,
     0x354(%esi,%ebx,4)`: `last_taunt[sender] = request`. The only
     field-level writer by both nets.
   - Block: `Leader::init (006e3930)` (`param_2 < 0`), `Leader::close
     (006b7f80)`, `Leaders::clear (006ed5d0)` — zero; `LeaderData::walk_data
     (006d6750)` on load.
2. **Event, step, predicate.** `process_taunt(request, sender)` is called
   from the per-frame queue drain (§3 below) on the receiving leader. It
   returns before the store unless all of:
   - the receiver's slot (`leaders[this->who]`) is **not** `LEADER_HUMAN`
     and **is** `LEADER_ACTIVE` (`6b8cef`, `6b8cf8`);
   - if `sender != leaders[who].who`: `leaders[who].diplos[sender] == 2`
     **and** `leadersc[sender].diplos[leaders[who].who] == 2` (mutual
     `DIPLO_ALLY`, re-checked at drain time, so an alliance broken during
     the delay drops the taunt) (`6b8d0e`, `6b8d29`);
   - `this->who != sender` (`6b8d34`).
   Then, **before** the `switch` on the request and for **any** request
   value (the store precedes the `cmp $0xf; ja default` range test at
   `6b8d5b`, so values above 16 are stored too), `last_taunt[sender] =
   request`. Not a counter: an overwrite with the sender's latest taunt.
   No units; the value is a `TauntRequest`.
3. **Reset.** Zero at every game start (`Game::init_rules_and_teams` →
   `Leaders::init` → `Leader::init(i, -1, i)` → `memset 0x6929`), at
   `Game::close` and OOS recovery (`Leaders::clear`). Never cleared after
   being consumed; it persists until the same sender's next processed taunt.
4. **Readers.** `Leader::action_ping (006d18a0)` (`6d1977 cmpl $0x10,
   0xe3a6e4(,%ecx,4)`, `ecx = k·0x1bbb + pinger`): when a `LEADER_HUMAN`
   leader pings for itself (`param_3 == this->who`), for each other
   `LEADER_VALID`, non-human leader `k` that `is_ally(pinger, k)`, it
   requires `leaders[k].last_taunt[pinger] == TAUNT_HELP` and
   `game->frame <= leaders[k].taunt_frame[pinger] + 225` (`0xe1` frames),
   then `Armies::find_useful_army` and `Army::send_here(…, MOVE_TO)`, and
   stamps the army's `+0x28` with `0xa8c`. Also walked by the checksum and
   save (`LeaderData::walk_data`).
5. **Crate.** No such event. `crates/sim/src/checksum/leaders.rs:87` names
   the leaf, but `Sim::leader_long` (same file, ~line 949) has no arm for
   it, so it is emitted unheld (zero bytes). The drain that would write it
   belongs in the per-player loop at the head of `Sim::tick`
   (`crates/sim/src/core/frame.rs`, which already ports `Leader::process`'s
   head); the reader, `action_ping`, is not ported (`Armies::
   find_useful_army` exists at `crates/sim/src/ai/army.rs:2207` with no
   caller; `Command::Ping` is not mapped in `rondata::input`).
6. **Plain game?** Only with a **human** who has a **computer ally** and
   sends a taunt order in chat (§3's producer). No trigger, CtW or
   multiplayer needed, but a skirmish with no human→allied-AI chat never
   writes it; an AI never issues a chat command.
7. **Confidence.** High on the writer, predicate and step (export + listing
   agree). Not read: the inside of `Taunts::check_for_order` beyond its
   shape, the `switch` arms' downstream effects (not this leaf's).

## 2. `taunt_frame` (`+0x374`, `int[8]`, indexed by sender slot)

1. **Writers.** `Leader::process_taunt (006b8cc0)` — `6b8d54 movl %eax,
   0x374(%esi,%ebx,4)` with `eax = game->frame` (`0xc061ec` → `+0x550`).
   Block: the three `memset 0x6929` sites and `walk_data` on load, as §0.
2. **Event, step, predicate.** Same store site and gate as `last_taunt`
   (§1): written in the same breath, `taunt_frame[sender] = game->frame`,
   the frame on which the queued taunt is processed (for a non-help taunt
   that is the receive frame + 60; for `TAUNT_HELP` the receive frame
   itself, §4). Units: sim frames (`Game::frame`). Overwrite, not a counter.
3. **Reset.** As `last_taunt`: zero at game start, `Game::close`, OOS
   recovery; never cleared on use.
4. **Readers.** `Leader::action_ping` (`6d198a movl 0xe3a704(,%ecx,4)`,
   `+0xe1`): the help window, `game->frame <= taunt_frame[pinger] + 225`.
   Checksum and save via `walk_data`.
5. **Crate.** As `last_taunt`: listed at `checksum/leaders.rs:88`, no
   `leader_long` arm, unheld; the writer belongs with the drain in
   `Sim::tick` (`core/frame.rs`).
6. **Plain game?** As `last_taunt`.
7. **Confidence.** High.

## 3. `incoming_taunt` (`+0x394`, `int[8]`, a ring indexed by `incoming`)

1. **Writers.**
   - `Leader::receive_taunt (006b8bf0)` — `6b8c4f movl %ebx,
     0x394(%esi,%eax,4)` with `eax = this->incoming`: `incoming_taunt[
     incoming] = request`.
   - `Leader::init (006e3930)`, `param_2 < 0` branch: besides the memset,
     an explicit loop `6e3a01..6e3a1e` zeroes `[k-8]`, `[k]`, `[k+8]` off
     `+0x3b4` for `k = 0..7`, i.e. `incoming_taunt`, `incoming_taunt_who`,
     `incoming_taunt_frame`, and `incoming = 0` (`6e3a09`). (Redundant with
     the memset just before it, but a real store.)
   - Block: `Leader::close`, `Leaders::clear`, `walk_data` on load.
2. **Event, step, predicate.** The producer chain:
   `ChatBox::on_modal_end (0093d560)` / `IFaceDiploBase::on_button_clicked
   (008014e0)` / `IFaceDiploBase::on_key_click (008017f0)` →
   `LeaderOut::issue_chat (006d2c50)`, which sets the command's `taunt` from
   `Taunts::check_for_order (009809d0)` (the chat text read as a taunt
   number whose taunt entry carries an order string, mapped to 1, 2, 3, …;
   −1 otherwise) → `CommandManager::issue_chat (00942b10)` → a
   `ChatCommand` (`bits`, `taunt`, `taunt_num`, `len`, `string`) →
   `CommandPackage::process_chat (009454f0)`, which for each `LEADER_VALID`
   recipient slot `k` (all when `bits == -1`, else bit `k` of `bits`, both
   subject to `chat_status` not 2 and, in the masked case, to the text not
   being a cheat line in a non-network game) calls `receive_chat` and, if
   `taunt >= 0`, `Leader::receive_taunt(leaders[k], taunt, sender_play)`
   (two sites, `94573a`, `945870`). Nothing else calls `receive_taunt`, and
   `CommandManager::issue_chat` has only the two interface callers: the
   computer never sends one.
   `receive_taunt(request, play)`: `sender = game.info.player[play].who`
   (byte `+0x77` of the `0x8c` record). It stores only if
   `request != 0` and `request >= 0` (`6b8c0d`, `6b8c15`), the **sender's**
   slot is `LEADER_ACTIVE` (`6b8c21`), the receiver is not `LEADER_HUMAN`
   and is `LEADER_ACTIVE` (`6b8c30`, `6b8c34`), `LeaderData::is_ally
   (006edb50)(receiver, sender)` (mutual `DIPLO_ALLY`, or the same slot),
   and `receiver.who != sender` (`6b8c44`). No upper bound on `request`.
   Step: overwrite of ring slot `incoming` with the request.
3. **Reset.** Zero at game start (memset and the explicit loop),
   `Game::close`, OOS recovery. **Not** cleared when drained: the drain
   matches on the frame, so a consumed slot keeps its old request until the
   ring comes round to it.
4. **Readers.** The drain, `Leaders::process_all (006ed2a0)` (the inlined
   `Leader::process`/`process_taunts` tail; `6ed3d7 leal 0x3b4(%esi),%edi`):
   for every `LEADER_ACTIVE` slot, if `game->frame != 0`, for `k = 0..7` in
   order, if `incoming_taunt_frame[k] == game->frame` call
   `process_taunt(incoming_taunt[k], incoming_taunt_who[k])`. Also
   checksum/save via `walk_data`.
5. **Crate.** No event. `Command::Chat` (`rondata::commands`, which already
   decodes `taunt` and `taunt_num`) is skipped in `Stream::one`
   (`crates/rondata/src/input.rs:1611`, "a cheat line, not an order"). A
   builder adds `receive_taunt` there (per recipient slot) with the ring
   state on the leaders record (`crates/sim/src/leaders.rs`), the drain in
   `Sim::tick`'s per-player loop (`crates/sim/src/core/frame.rs`), and the
   checksum arm in `Sim::leader_long` (`checksum/leaders.rs:89`, unheld
   today).
6. **Plain game?** Only with a human sending a taunt order to a computer
   ally (team lobby or an alliance made in game). Not trigger, CtW or
   multiplayer; but an all-computer game, or a human with no computer ally,
   never writes it.
7. **Confidence.** High on `receive_taunt` (export + listing agree). Medium
   on the recipient gating inside `process_chat` (read in the decompile
   only, not the listing) and on `check_for_order`'s mapping (shape read,
   string table not resolved).

## 4. `incoming_taunt_who` (`+0x3b4`, `int[8]`, ring)

1. **Writers.** `Leader::receive_taunt (006b8bf0)` — `6b8c5c movl %edi,
   0x3b4(%esi,%eax,4)`, `edi` = the sender's leader slot (from
   `game.info.player[play].who`, not the play index). `Leader::init`'s
   explicit loop (`6e3a13 movl %edx,(%eax)` at `+0x3b4+4k`) and the
   memsets; `walk_data` on load.
2. **Event, step, predicate.** Same call and gate as `incoming_taunt` (§3),
   same ring slot (`incoming` is re-read between the stores but not changed
   until the end). Overwrite with the sender slot, `0..9`.
3. **Reset.** As §3.
4. **Readers.** The drain in `Leaders::process_all` passes it as
   `process_taunt`'s sender, which then indexes `last_taunt`/`taunt_frame`
   and `diplos` by it. Checksum/save.
5. **Crate.** As §3 (`checksum/leaders.rs:90`, unheld).
6. **Plain game?** As §3.
7. **Confidence.** High.

## 5. `incoming_taunt_frame` (`+0x3d4`, `int[8]`, ring)

1. **Writers.** `Leader::receive_taunt (006b8bf0)`, two stores on the two
   arms of one branch: `6b8c79` (request `== 0x10`, `TAUNT_HELP`) writes
   `game->frame`; `6b8c91` (any other request) writes `game->frame + 0x3c`.
   `Leader::init`'s explicit loop (`6e3a18 movl %edx,0x1c(%eax)` after the
   `+4`, i.e. `+0x3d4+4k`) and the memsets; `walk_data` on load.
2. **Event, step, predicate.** Same call and gate as §3. The value is the
   frame on which the drain will process the slot: **60 frames** after
   receipt (`0x3c`, sim frames) for every request but help, **0 frames**
   for `TAUNT_HELP`. The zero delay works because commands are applied in
   `TurnControl::do_frame` before `Game::do_frame`'s `Leaders::process_all`
   on the same `game->frame` (§0). The drain's `frame != 0` guard keeps the
   zeroed ring from firing on frame 0. More than eight taunts to one
   receiver inside one delay overwrite queued slots before they fire.
3. **Reset.** As §3. Not cleared on drain; a stale value never re-fires
   because `game->frame` only rises.
4. **Readers.** The drain (`piVar[8] == game->frame`). Checksum/save.
5. **Crate.** As §3 (`checksum/leaders.rs:91`, unheld).
6. **Plain game?** As §3.
7. **Confidence.** High on the two constants and the branch (listing).
   Medium on the same-frame claim for help: the call chain and the order
   in `Game::loop` were read, the package stamp test inside
   `process_turn` was not.

## 6. `incoming` (`+0x3f4`, `int`, the ring's next write slot)

1. **Writers.** `Leader::receive_taunt (006b8bf0)` — `6b8cab movl %eax,
   0x3f4(%esi)`. `Leader::init (006e3930)` — `6e3a09 movl %edx,0x3f4(%ebx)`
   with `edx = 0`, `param_2 < 0` branch, plus the memset. `Leader::close`,
   `Leaders::clear` (memset); `walk_data` on load. (`RoadsOut::
   one_time_init`, `GameReportWin::close`, `ConquestInvadeWin::close` and
   the `TerrainOut` family hit `field_0x3f4` on their own classes.)
2. **Event, step, predicate.** After the three ring stores of an accepted
   taunt (§3's gate), `incoming = (incoming + 1) mod 8`, computed as
   `(x+1) & 0x80000007` with the signed-remainder fix-up (`6b8c9e..6b8ca7`);
   since `incoming` is never negative this is `(x+1) & 7`. Step: +1 per
   accepted taunt to this receiver, wrapping 7 → 0.
3. **Reset.** Zero at game start, `Game::close`, OOS recovery.
4. **Readers.** Only `receive_taunt` itself (five loads, `6b8c49..6b8c98`);
   the drain scans all eight slots and ignores it. Checksum/save.
5. **Crate.** As §3 (`checksum/leaders.rs:92`, unheld).
6. **Plain game?** As §3.
7. **Confidence.** High.

## 7. Corroboration on disk, and what was not read

- `crates/rondata/src/blind.rs` (comments near lines 320–330 and 380–390)
  records two captures in a team lobby with a human and a computer ally: a
  help taunt left `last_taunt[0] = 16` on leader 1, and food and timber
  taunts were queued for 60 frames and processed through `process_taunt`'s
  case 1. That agrees with §1, §3 and §5 (its `taunt_frame` is one frame
  after the frame it calls the send frame, which is a frame-numbering
  convention I did not chase, not a contradiction of a zero help delay).
- Not read: `Taunts::check_for_order`'s string table (which taunt numbers
  map to which request), `CommandManager::process_turn`'s package stamp
  test, `LeaderData::log_data` beyond the listing net (no operand in the
  family; I did not check whether it walks the block another way),
  `CheckSums::check_leaders`' body (only that it calls `walk_data`), and
  `process_taunt`'s case arms, which write other leaves (`gift_stamp`,
  `wonder_mod`, `ground_mod`, `air_mod`, `sea_mod`, `infra_mod`,
  `defense_mod`, `+0x6dec`), not these.

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `last_taunt` +0x354 `TauntRequest[8]` | `Leader::process_taunt (006b8cc0)`; block: `Leader::init (006e3930)` (`param_2<0`), `Leader::close (006b7f80)`, `Leaders::clear (006ed5d0)`, `LeaderData::walk_data (006d6750)` on load | `[sender] = request` when a queued taunt is drained by a non-human active receiver, sender ≠ receiver, mutual ally at drain time; any request value | zero at game start (`Leaders::init` → `init(i,-1,i)`), `Game::close`, OOS recovery; never on use | none; unheld in `Sim::leader_long` (`checksum/leaders.rs`); writer would sit in `Sim::tick` (`core/frame.rs`) | only via a human's taunt-order chat to a computer ally | high |
| `taunt_frame` +0x374 `int[8]` | as `last_taunt` | `[sender] = game->frame` at the drain | as above | as above | as above | high |
| `incoming_taunt` +0x394 `int[8]` | `Leader::receive_taunt (006b8bf0)`; `Leader::init` explicit loop + memset; `Leader::close`, `Leaders::clear`; `walk_data` on load | ring `[incoming] = request` on a chat command with `taunt > 0`, sender active, receiver non-human active, mutual ally, sender ≠ receiver | as above; not cleared on drain | none; `Command::Chat` skipped in `Stream::one` (`rondata/src/input.rs:1611`); unheld checksum leaf | as above | high (gate), medium (`process_chat` recipient mask) |
| `incoming_taunt_who` +0x3b4 `int[8]` | as `incoming_taunt` | ring `[incoming] = sender slot` | as above | as above | as above | high |
| `incoming_taunt_frame` +0x3d4 `int[8]` | as `incoming_taunt` (two stores, `6b8c79`/`6b8c91`) | ring `[incoming] = frame` for `TAUNT_HELP`, `frame + 60` otherwise | as above | as above | as above | high (constants), medium (same-frame help) |
| `incoming` +0x3f4 `int` | `Leader::receive_taunt`; `Leader::init` (`6e3a09`) + memset; `Leader::close`, `Leaders::clear`; `walk_data` on load | `(x+1) & 7` per accepted taunt | as above | as above | as above | high |
