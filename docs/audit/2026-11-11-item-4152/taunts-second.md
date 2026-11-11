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

