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

### 1b. The listing net, counted

- 39 calls to `BitMask<256>::set (00450360)` in `.text`; the ones whose `ecx`
  is loaded from `0x6c0c(reg)` or `0xe40f9c + who*0x6eec` within eight lines
  before the call: `Leader::init` 4, `Leader::gain_tech` 2 (line 303 at
  `0x6dcfa0`, the unit graft at `0x6dde1f`), `Leader::lose_tech` 2,
  `ConsoleWin::run_cmd` 2. Matches the export.
- Non-`testb` register forms `0x6c0c(`/`0x6c10(`/`0x6c14(`/`0x6c18(`:
  `LeaderData::LeaderData`, `Leader::close (006b7f80)` (memset of the bytes,
  `flags = 1`), `LeaderData::walk_data`, `LeaderData::log_data`, `lose_tech`,
  `gain_tech`, `Leader::init`, and `MapScenario::make (0068c8f0)` at `0x68cff4`
  (an inline `bts` into a unit bit, loop bound `0x160`, scenario only).
- Absolute forms: `0xe40f9c` in `action_cheat_give_techs`, `gain_tech`,
  `ConsoleWin::run_cmd`; `0xe40fa4` in `Leaders::clear (006ed5d0)` (zeroes all
  eight slots' masks, `flags = 1`); `0xe40fa8(idx)` loads only (`testb`), except
  `ScenarioFuncSet::disable_type_by_tribe`. A scan for any absolute address
  inside the eight slots' bytes `+0x4c..+0x57` of `tech.ptr` finds only
  `disable_type_by_tribe`'s slot 1–7 copies (28 lines). No inline fixed-index
  store into a spell bit exists anywhere.

### 2. The event, the step, the predicate

**The event is `Leader::init (006e3930)`, once per leader per game, before
frame 1.** Its plain-game callers are `Setup::build_game (005ac190)` and
`Setup::build_leader (005abc70)`; the others are `Leaders::init (006ed850)`,
`WalkDataGame::walk_data (005a2360)`, `MapScenario::make`,
`ConquestGame::build_conquest_game (00797c40)`, `ScenarioEditor::new_leader
(009a1490)` and `ScenarioRead::load_leader_chunk (009ac080)` (three sites).

Order inside `init`: zero `tech` and `tech_at_start` (`BitMask<806>::init`);
loop 1, `0x220..0x274` (ages and techs below the starting age), sets both
masks; loop 2, buildings `0x19e..0x21e`, `type_avail(t,1)`, `tech` only; loop 3,
units `0x32..0x191`, both masks; **loop 4, the spells, `t = 0x275 .. 0x2ab`**:

> set `tech` bit `t` iff `LeaderData::type_avail(t, 1) != 0` **and**
> (`spelltypes[t]->preq[0] != -1` **or** `spelltypes[t]->spell_flags & 1 == 0`).

`tech_at_start` is not written by loop 4. Then `Leader::reset_obs_flags
(006e32a0)`, then the nation bonuses (Dutch/Russian/Aztec/Roman/American free
epochs, Persian despotism — all `gain_tech` of an epoch or government, after
the spell loop, so they cannot feed it). Listing `0x6e45d9..0x6e461d` confirms
the predicate: `cmpl $-1, 0x30(spell)`, `testb $1, 0x1c8(spell)`.

Field meanings, from the loader `SpellType::init (00674a80)`: `+0x30` is the
`TypeData::preq[0]` slot, filled by `Types::tech_key` from the XML's prereq
text; `+0x1c8` is `spell_flags`, built from a letter string — each letter `a..z`
sets bit `letter - 'a'`, each digit `1..9` sets bit `digit - 0x17` (mod 32) — so
bit 0 is flag letter `a`. `SpellType::log_data (00675450)` prints `+0x1c8` as
`spell_flags`, which is how the dump's values above are this field.

What `type_avail (006e33a0)` reduces to for a spell (not a unit, vtable `+0xc`;
not a building, `+0x14`; not a government, `+0x2c`): `has_preq(t)` must hold,
then it returns `type_eligible(t, 1)` (4 unless refused). Unrolled:

- `LeaderData::type_eligible (006dbd10)`: `tribe_can_type(t) == 4` — the
  leader's tribe bit in `tribe_mask` (`LeaderData::tribe_can_type (006d9410)`);
  then for each preq slot `i` (`TypeData::get_preq (00668700)`): a preq below
  `-1` refuses (returns 0); a preq that is a non-epoch tech whose age
  (`techtypes[q] + 0x1c8`) exceeds the game's `ending_technology` refuses.
  Spells are none of good/unit/building/tech, so otherwise 4.
- `LeaderData::has_preq (006db810)`: each slot's `get_preq`; a negative one
  goes to `LeaderData::has_tech (006e0c80)`, which answers 1 for `-1`
  (`TYPE_NONE`) and 0 for `-2` (`TYPE_ANY_BUILDING`); a non-negative one,
  after `special_preq (006d7030)` (no spell case), and not a bonus type, is
  `has_tech(q)`. No age or government tail applies to a spell.
- `get_preq` for slot 0 returns `preq[0]` raw; slots 1 and 2 are `-1` for
  every spell here, so its age-remapping arms never fire.

So in this ruleset (every `spell_flags` even), the bit for spell `t` is set at
init **iff the leader's tribe is in `tribe_mask` (all are), `preq[0]` is not
`-2`, and `preq[0]` is `-1` or a type the leader already owns after loops 1–3.**
For run 1562's two leaders (starting age 0, no `all_techs`): 46 set; the eight
`-2` spells refused by `type_eligible` and `has_preq` both; `RADARSPELL`
refused because `INDUSTRIAL_AGE` (0x224) is not owned at age 0. That is the
packet, bit for bit, in both slots.

Step: one bit, 0 to 1, no counter. Units: none (a membership bit).

**After init, in a plain game, nothing re-runs this loop.** Reaching the
Industrial Age (`gain_tech(0x224)`) does not set `RADARSPELL`'s bit: no loop in
`gain_tech` reaches `0x275` (every loop bound in its body is an exclusive `< 0x21f`
or `< 0x24a .. < 0x275`, i.e. buildings and techs only), and `Leader::set_age` gains ages only. The only
plain-game setter after init is `Build::finished`'s fall-through (§1): a spell
queued in a building whose `has_spell` is false at the finish is "gained" —
its bit set by `gain_tech(spell, x, y, 1, 1)` — instead of cast. With no
flag-`a` spell in the data, that needs a queued spell that lost availability
between queue and finish.

### 3. What clears or resets it

`LeaderData::LeaderData`, `Leader::close`, `Leaders::clear` (all slots),
`Leader::init`'s `BitMask<806>::init` at the top of its main arm and its
`param_2 < 0` reset arm, and a save load (`walk_data`). In play: **nothing
clears a spell bit** — `Leader::lose_tech` clears only units and techs (a spell
falls out of both arms, exactly as `crates/sim/src/tech.rs`'s `lose` comment
says for "a building, good, spell or bonus"). Outside a plain game:
`disable_type_by_tribe` (trigger) and the console. So in a plain game the
spell bits are a frozen snapshot of `init`'s loop 4, plus the rare
`Build::finished` gain.

### 4. Who reads it

- `LeaderData::has_spell (006e0bc0)`: for a spell with `spell_flags & 1` it
  requires the `tech` bit **and** `type_avail`; for every other spell it is
  `type_avail` alone and never reads the bit. With no flag-`a` spell in this
  data, **the spell bits are read by no game rule through `has_spell`**.
  `has_spell` itself is asked by `BuildOut::make_options` (spell buttons),
  `Build::finished`, `Options::do_spell`, `Unit::do_cast`,
  `Group::action_spell` and the AI.
- `LeaderData::has_tech (006e0c80)` with a spell argument reads the bit raw
  (not a unit, not a building); I found no caller passing a spell.
- `LeaderData::log_data (006e5110)` (the `bits`/`size`/`bit_values` dump
  lines) and the save walker; and the windowed checksum, whose leaders part
  carries `tech.ptr` (that is why this leaf parts at all).

### 5. Where `crates/sim` has this event

`TechTree::start` in `crates/sim/src/tech.rs` (line ~2235) is `Leader::init`'s
tech block, called from `Sim::start_techs` in `crates/sim/src/tech/gain.rs`. It
ports loops 1–3 and then `reset_obs_flags`, and **has no loop 4**. Worse for a
builder, the tree cannot hold the bits: `TechTree::types` holds goods, units,
buildings and techs only (`Kind` has no spell variant, `tech.rs:100`), and
`checksum/leaders.rs` `tree_type_index` maps tree indices onto `0x00..0x274`.
`queue_disband.rs:172` says it outright: "this crate's tree holds no craft",
answering `has_spell(0x29a)` with a SEAM. So today **the crate's `tech` mask
carries none of 0x275..0x2ab** while the original carries 46 at frame 91 —
the whole of this leaf's parting.

A builder adds, between the unit loop and `self.reset_obs_flags(setup, p)` in
`TechTree::start`, the 55 spell bits (either spell entries in the tree, or a
separate `spells: [bool; 55]` on `PlayerTech` that `bits_of` in
`checksum/leaders.rs` appends as `0x275 + i`), each set iff tribe bit in
`tribe_mask` && `preq[0] != -2` (and every preq slot `>= -1`, and any tech
preq's age `<= ending_technology`) && (`preq[0] == -1` || owned) && (`preq[0]
!= -1` || `spell_flags & 1 == 0`). The inputs are the loaded `SpellType`
records (the crate's SEAM names `craftrules.xml` for DISBAND; where the
other 54 load from I did not check): `preq`, `tribe_mask`, `spell_flags`. And `Build::finished`'s spell fall-through
`gain_tech` is a second, rarer site (the crate's `finish_disband` in
`cities/queue_disband.rs` is the only spell-finish arm it has; DISBAND's preq
is `-1`, so it never falls through).

### 6. Reached by a plain game?

Yes, every plain skirmish: `Setup::build_game`/`build_leader` →
`Leader::init` → loop 4, before frame 1. Run 1562 (a playback of a plain
two-player skirmish, no scenario, no CtW) shows the result at frame 91 in
both player slots. The `Build::finished` fall-through is plain-game reachable
but needs an unusual queue. The cheat order (`process_cheat_give_techs`), the
console, the trigger functions, the scenario reader/editor and CtW are the
rest.

### 7. Confidence and what I did not read

- **High** for the rule and its inputs: the decompile, the listing at
  `0x6e45d9`, the original's own loaded `SPELLTYPE` values, and the packet's
  bytes agree on all 55 bits in both slots.
- Read in full: `Leader::init`'s tech block, `type_avail`, `type_eligible`,
  `has_preq`, `has_tech`, `get_preq`, `special_preq`, `tribe_can_type`,
  `has_spell`, `lose_tech`, `fix_tech_flags`, `Build::finished`'s spell arm,
  `action_cheat_give_techs`, the console's tech arm, `SpellType::init`'s
  `preq`/`spell_flags` lines, `gain_tech`'s first 330 lines and its loop bounds.
- Not read: the remaining ~2,100 lines of `gain_tech` body (I relied on the
  absence of any loop reaching `0x275` and on the two `set` calls on `tech`);
  `ScenarioFuncSet::gain_tech`/`gain_upgrade`, `ScenarioRead`,
  `ScenarioEditor`, `ConquestGame` bodies (out of a plain game); every reader
  beyond those named; what XML text loads as `-2` (`Types::tech_key`), and
  whether those eight spells are enabled some other way in other rulesets.
- Not established: whether a starting age past the Industrial Age (or
  `all_techs`) sets `RADARSPELL` at init in a capture — predicted by the rule
  (loop 1 grants the age before loop 4 runs), not observed.

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `tech` bits 0x275..0x2ab (spells) | `Leader::init (006e3930)` loop 4 (the writer); `Build::finished (00628490)` → `Leader::gain_tech (006dcb60)` on a queued spell `has_spell` refuses; out of plain play: `Game::action_cheat_give_techs (00593180)`, `ConsoleWin::run_cmd (007d6a70)`, `ScenarioFuncSet::disable_type_by_tribe (00a006a0)` (clears) | bit set once at init iff `type_avail(t,1)` (tribe in mask, no preq `< -1`, preq `-1` or owned) and (`preq[0] != -1` or `spell_flags & 1 == 0`); never re-evaluated; `tech_at_start` untouched | `LeaderData::LeaderData (006d7540)`, `Leader::close (006b7f80)`, `Leaders::clear (006ed5d0)`, `Leader::init`'s zeroing; never in play (`lose_tech` skips spells) | none: `TechTree::start`, `crates/sim/src/tech.rs`, has no spell loop and the tree holds no spell types (`queue_disband.rs:172` SEAM); add between the unit loop and `reset_obs_flags` | yes, every skirmish, before frame 1; run 1562 frame 91: 46 of 55 set in both slots (8 have `preq -2`, `RADARSPELL` needs `INDUSTRIAL_AGE`) | high (decompile + listing + loaded records + packet agree bit for bit) |
