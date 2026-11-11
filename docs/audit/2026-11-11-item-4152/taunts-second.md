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

