# Item 4152, family 10 `end-of-game` — first reading

- Family: 10, `end-of-game`.
- Leaves (`LeaderData` offsets): `defeated_by` (+0x10), `wonderwin_stamp`
  (+0x448), `wonderwin_timer` (+0x44c), `victory_type` (+0x7d8),
  `defeat_type` (+0x7dc), `defeat_stamp` (+0x6924); all int.
- Reader: Claude Opus 5.5 (`claude-opus-5-5`), first reading, wave 1,
  2026-11-11.
- Sources: the decompile export (`~/ghidra-projects/decomp`), the listing
  (`llvm-objdump` of `riseofnations.exe`), `crates/sim/src`.

## Common ground

- **`Leader*` is `LeaderData*`, offsets unshifted.** `types.txt` gives
  `Leader` (0x6eec) as `LeaderData` (0x6ee4) at +0 with 8 trailing bytes
  (`+0x6ea4..` are `LeaderData`'s own); `Leaders` is `Leader[10]` from +0.
  Every `Leader::` method below reaches `who` as `this+0x8` and loops
  `0xe3a390..0xe71af0` in steps of 0x6eec, so the leaf offsets are the same
  in `Leader` and `LeaderData` code, and the global form is
  `0xe3a390 + i·0x6eec + off`.
- **Global-form net** (`0xe3a3a0`, `0xe3a7d8`, `0xe3a7dc`, `0xe3ab68`,
  `0xe3ab6c`, `0xe40cb4` over the full listing): **no store at all**; only
  reads (listed per leaf) and one `lea 0xe3a3a0` in
  `RecordGame::write_random_game_info (009534e0)`, which is the *end*
  pointer of a `[tribe, tribe+4)` range handed to `SaveGame`'s writer — not
  a write of `defeated_by`.
- **Bulk writers of all six** (not game events):
  - `Leader::init (006e3930)` with `param_2` (tribe) < 0 opens with
    `memset(this, 0, 0x6929)` (listing `6e398c..6e3995`), which zeroes every
    leaf of this family. `Leaders::init (006ed850)` calls it that way for
    each slot (`push 0; push -1; push who`, `6ed8c5..6ed8cb`), from
    `Game::init_rules_and_teams (00589bb0)`.
  - `LeaderData::walk_data (006d6750)` walks `[who, +0x692a)` for a seated
    slot; under an input walker (`LoadGame::verify_load (005a39f0)`) that
    restores all six from a save. Under the checksum walker it only reads.
- **The victory/defeat type codes**, from the call sites (a code is the
  first argument; values are the original's own):
  - `victory_type`: 0 conquest (`Game::check_victory (005926b0)`,
    `Leader::set_diplo (006ec6a0)`, console, `ScenarioFuncSet::victory
    (009f3fa0)`, `TurnControl::toggle_pause (00957ab0)`); 1 wonders and 2
    territory (`GameDaemon::process_victory (00730ef0)`); 3 the ending
    technology (`Leader::gain_tech (006dcb60)`, VICTORY 9); 4 score goal
    (VICTORY 3); 5 economic (VICTORY 8); 6 time limit (VICTORY 4). Allies
    of the winner inherit the winner's code (below).
  - `defeat_type`: 0 no cities (`Leader::lost_a_city (006dc6d0)`, default
    elimination) or no units and no cities (`Unit::close (0060ee50)`) or
    console/scenario; 1 capital elimination (`Leader::process_elimination
    (006b8a20)`, `lost_a_city`'s `GLOBAL_GOVERNMENT_BONUS` arm,
    `gain_tech`, `Player::leave_game (006ee010)` with a capital timer
    running); 2 and 3 elimination modes 2 and 3 (`lost_a_city`); 4 musical
    chairs (VICTORY 5, `process_victory`); 5 an enemy's victory
    (`Leader::victory (006ec9b0)`'s cascade); 6 resign and 7 drop/disconnect
    (`Player::leave_game` from `Player::resign (006edcb0)` /
    `Player::drop (006ede80)`; 7 also `CommandPackage::end_process
    (0094c800)`); 8 armageddon (`Game::defeat_all (00592aa0)`); 9 scenario
    (`ScenarioFuncSet::defeat (009f3f50)`).

