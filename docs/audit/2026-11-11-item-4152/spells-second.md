# Item 4152, second (blind) reading: `spells`

- Slug: `spells`
- Leaf: `LeaderData::tech` (+0x6c0c, `BitMask<806>`), restricted to the
  bits for type indices 0x275..0x2ab that run 1562 frame 91 shows set in
  `tech` and absent from `tech_at_start`.
- Reader model: Claude Opus 5.5 (claude-opus-5-5), blind to the first reading.

## Ground: what the bits are and what frame 91 holds

- `BitMask<806>` is `{bits, size, flags, ptr[101]}` (`types.txt`, `struct
  /rise.pdb/BitMask<806>`, size 0x74), so the byte array starts at
  `LeaderData +0x6c18`, and bit `t` is `ptr[t >> 3]` bit `t & 7`. The family's
  bits 0x275..0x2ab are bytes 0x4e (bit 5) through 0x55 (bit 3), i.e.
  `LeaderData +0x6c66..+0x6c6d`; for slot `i` in the global array, absolute
  `0xe40ff6 + i*0x6eec .. 0xe40ffd + i*0x6eec` (`0xe3a390 + 0x6c18 = 0xe40fa8`).
- `Leader` (0x6eec) has no prefix: every `Leader` method uses `this` as the
  `LeaderData` (`*(uint*)this` is `leader_flags`, `this->field_0x8` is `who`), so
  `field_0x6c0c` in a `Leader` method is `tech` and `field_0x6c18` its bytes.
- The range is exactly the spell types: `enums/TypeIndex.txt` puts
  `BASE_SPELLTYPES = END_TECHTYPES = 0x275` (BRIBE) and `END_SPELLTYPES =
  0x2ac`; `NUM_SPELLTYPES = 55`. So this leaf is "which spells the leader's
  `tech` mask carries".
- **Frame 91, read from the packet itself** (`~/ron-golden/run1562/map-14/
  frame-snapshot.bin`, logger frame 91; the text dump's `bit_values` is the
  bytes printed as unpadded decimals and does not parse uniquely — 156 million
  101-byte parses — so I read the memory). Slots 0 and 1 (the two players),
  bytes `0x4c..0x57` of `tech.ptr`: `00 00 a0 bc 4b ff ff fd ff 0f 00 00` in
  both; `tech_at_start` has none of these bits; slots 2, 3 are empty. That is
  **46 of the 55 spells set in each slot**; the nine clear are 0x276 PILFER,
  0x278 ASSIMILATE, 0x279 RALLY, 0x27e RESTOCK_SUPPLIES, 0x282 BLOW_TREAD,
  0x284 PARADROP, 0x285 RESEARCH_PROJECT, 0x287 MOVE_CAPITAL_HERE, 0x299
  RADARSPELL. (Probe: a 30-line reader of the packet's range records, kept in
  the job's scratch, not in git.)
- The same run's dump prints every `SPELLTYPE` as the original loaded it
  (gamelog line 185432 on): all 55 have `tribe_mask 16777215` (all 24
  tribes); `preq[0]` is `-2` for exactly 630, 632, 633, 638, 642, 644, 645,
  647, is `548` (0x224, `INDUSTRIAL_AGE`) for 665, and `-1` for the other 46;
  `preq[1]`, `preq[2]` are `-1` for all; and every `spell_flags` is even (bit 0
  clear). The nine clear bits are exactly the eight `-2` spells plus the one
  that needs the Industrial Age. The rule below predicts the packet bit for bit.

## Leaf `tech` (+0x6c0c), bits 0x275..0x2ab (the spells)

### 1. Writers, by both nets

Export net: `grep -rn -- '->tech\b'` finds only readers plus the constructor
and `walk_data`; the writes go through `BitMask<256>::set(&this->field_0x6c0c,
t, v)` (the 256 is LTCG folding — one body for every `BitMask<N>::set`), and
`grep -rn 'field_0x6c0c\|field_0x6c18'` plus the callers of `gain_tech` give
the rest. Every writer, and whether it can touch a spell bit:

| writer | what it does to `tech` | spell bits? |
|---|---|---|
| `LeaderData::LeaderData (006d7540)` | `bits=0x326, size=0x65, memset 0, flags=1` | clears (construction) |
| `Leader::init (006e3930)`, `param_2 < 0` arm | `BitMask::clear` on `tech`, `tech_at_start`, `obs_flags`, the conquest masks | clears (leader reset) |
| `Leader::init (006e3930)`, main arm | `BitMask<806>::init` on `tech` and `tech_at_start` (zero), then four set loops; **the fourth is the spell loop**, `0x275..0x2ab`, `tech` only | **sets — the writer of this leaf** |
| `Leader::gain_tech (006dcb60)` line 303 | sets bit `param_1` unconditionally, whatever its kind | sets, if a caller passes a spell (see below) |
| `Leader::gain_tech (006dcb60)` line 857 | sets unit bits `0x32..0x191` in `tech` and `obs_flags` (graft chain) | no |
| `Leader::lose_tech (006d2850)` | clears only when the type is a unit (vtable `+0xc`) or a tech (`+0x28`, 0x220..0x274) | **no** — a spell is neither, so it falls through untouched |
| `Leader::gain_tech (00471110)`, `Leader::lose_tech (0047df20)` | forwarders to the two above | as above |
| `Game::action_cheat_give_techs (00593180)` | raw byte ORs at `0xe40f9c + who*0x6eec + 0xc + (t>>3)` for every `t < 0x326`, zeroes `tech.flags` (`+0x6c14`) | sets all; reached only from `CommandPackage::process_cheat_give_techs (00945070)` — the cheat order |
| `ConsoleWin::run_cmd (007d6a70)` ~line 6460–6510 | dev-console tech command: a non-tech type's `tech` bit set or cleared with `BitMask::set` directly, techs through `gain_tech`/`lose_tech` | sets/clears; console only |
| `ScenarioFuncSet::disable_type_by_tribe (00a006a0)` | clears bit `t` in all eight slots' `tech` bytes inline | clears; trigger layer |
| `ScenarioFuncSet::gain_tech (009ee1a0)`, `gain_upgrade (009eeb90)`, `ScenarioRead::load_leader_chunk (009ac080)`, `ScenarioEditor::research_tech (0099e9d0)`, `change_game_rule_combo (0099b260)`, `MapScenario::make (0068c8f0)`, `ConquestGame::setup_scenario_bonuses (0078be90)`, `build_cities (00796140)` | `gain_tech` callers | scenario / editor / CtW only |
| `LeaderData::walk_data (006d6750)` | the save walker streams `ptr[0..size)` and sets `flags = 2` on read | restores (load) |

`gain_tech`'s plain-game callers and their argument ranges, read at each call:
`Build::activate (00623e20)` passes only types in 0x261..0x263, 0x26c..0x26e,
0x264..0x266, `FORTIFICATION..0x253`, units `0x32..0x191`, `END_FINALTYPES
..0x249`, `RELIGION..0x24c` (all below 0x275); `Leader::set_age (006d25a0)` and
`MapScenario::make` ages; `Leader::set_epoch (006d26f0)` epochs; `Build::queue_up
(00620f40)` only an age (vtable `+0x34`) under victory 9; `Leader::init`'s nation
bonuses an epoch or a government; `TerrainOil::gain_tech (008a2e40)` writes no
leader mask at all (it clears a terrain patch byte). **One plain-game path can
pass a spell: `Build::finished (00628490)`**. For a spell (vtable `+0x48`,
0x275..0x2ab) it asks `LeaderData::has_spell`; if true it calls
`SpellType::cast` and returns 1; **if false it falls out of the spell arm**, the
type is not a building (vtable `+0x14`), and control reaches the common tail
`LAB_00628520`, `gain_tech(leader, spell, x, y, 1, 1)`, which sets the spell's
bit. That is the "research it first" path for a spell whose `spell_flags` bit 0
demands an explicit grant (`has_spell` below); in this ruleset no spell has that
bit, so the path is reached only by a queued spell that has stopped being
available by the time the building finishes it.

Listing net (`exe.s`, the full `.text` objdump): `grep` for absolute
displacements into the eight slots' family bytes (`0xe40ff6..0xe40ffd` plus
`k*0x6eec`) and for the base forms `0xe40fa8(`/`0xe40f9c(`/`0x6c18(`/`0x6c0c(`
— see §1b below for the counts. The writers above are the only stores found.

