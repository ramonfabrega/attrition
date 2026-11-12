# Item 4152, family 17 (`spells`): first reading

- Family: 17, slug `spells`.
- Leaf: `tech.ptr` — `LeaderData::tech` (+0x6c0c, `BitMask<806>`), the bits
  for type indices `0x275..0x2ab` that are set in `tech` and absent from
  `tech_at_start`.
- Reader: Claude Opus 5.5 (first reading, wave 1), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the listing
  (`llvm-objdump`), and a grep of `crates/sim`. Nothing run.


## Where the bits sit

`tech` is a `BitMask<806>` at `LeaderData +0x6c0c` (`types.txt`: `bits`
+0x0, `size` +0x4, `flags` +0x8, `ptr` `uchar[101]` +0xc), so the array is
`+0x6c18`; `tech_at_start` is the same shape at `+0x6c80`. `Leader`
(`0x6eec`) carries `LeaderData` (`0x6ee4`) at offset 0 — `this->field_0x8`
is `who` in every `Leader` method read — so a `Leader*`'s `field_0x6c0c` is
the same field; in the listing the array of slot `i` is `0xe40fa8 + i*0x6eec`
and the mask head `0xe40f9c + i*0x6eec`. The decompiler prints the setter
as `BitMask<256>::set` on every mask (one folded body, `0x450360`).

The leaf is one rule, not 46: **every spell row the shipped
`craftrules.xml` gives `PREQ0 None` gets its bit from `Leader::init`'s spell
pass; nothing else sets one in a plain game.** The 55 rows of
`0x275..0x2ab` (`enums/TypeIndex.txt`, `BASE_SPELLTYPES`..`CANCEL_ALLIANCE7`;
`END_SPELLTYPES` is `0x2ac`) split, by reading the file's columns against
the pass's predicate:

| rows | count | `PREQ0` | bit at init |
|---|---|---|---|
| `0x275, 0x277, 0x27a..0x27d, 0x27f..0x281, 0x283, 0x286, 0x288..0x298, 0x29a..0x2ab` | 46 | `None` | **set** |
| `0x276, 0x278, 0x279, 0x27e, 0x282, 0x284, 0x285, 0x287` | 8 | `Disable` | clear |
| `0x299` (`RADARSPELL`, "Upgrade") | 1 | `INDUSTRIAL_AGE` (`0x224`) | set only when the leader holds Industrial Age at init |

No row carries `FLAGS` letter `a` (the file's own legend: "Craft/Ability must
be researched"), and every `TRIBE_MASK` is 24 ones. The 46 is the README
row's count exactly.

## Leaf `tech.ptr`, bits `0x275..0x2ab`

### 1. Writers, by both nets

Export net (`grep -rn 'field_0x6c0c|field_0x6c18|->tech\.|.tech.'`
over `funcs/`) and listing net (every non-test store or `lea` of
`0x6c0c(%r)`, `0x6c18(%r)`, `0xe40f9c`, `0xe40fa8` in an `llvm-objdump` of
all `.text`) give the same set. Each, with whether it can touch a spell bit:

- **`Leader::init (006e3930)`, the spell pass, `6e45d9`..`6e461d`.** The one
  setter of these bits in a plain game. Loop `t = 0x275 .. 0x2ab`:
  `type_avail(t, 1)` (`6e45e3`, `LeaderData::type_avail (006e33a0)`) must be
  non-zero, and then either `SpellTypeData +0x30 != -1` (`TypeData::preq[0]`,
  `6e45fa`) or `SpellTypeData +0x1c8 & 1` clear (`6e45ff`, the `FLAGS`
  letter `a`; `SpellType::init (00674a80)` zeroes `+0x1c8` and ors in
  `1 << (letter - 'a')`, a digit `1`..`9` as `1 << (digit - 0x17)`). Then
  `set(tech, t, 1)` (`6e460b`, `lea 0x6c0c(%ebx)`). **`tech_at_start` is not
  written in this pass** — that is why the bits are in one mask and not the
  other (the age/tech pass, `6e4365`/`6e4373`, and the unit pass,
  `6e45b3`/`6e45c1`, write both; the building pass, `6e4480`, bound
  `0x21f`, writes `tech` only, like this one).
- **`Leader::gain_tech (006dcb60)`**, `set(tech, param_1, 1)` at `6dcfa0`
  (decompile line 303), unconditional on the type. It sets a spell bit only
  when called with a spell type. Its 70 call sites (direct, and through the
  wrapper `Leader::gain_tech (00471110)`) were each checked for the type
  passed: `Build::activate (00623e20)` walks plain techs (`0x261..0x263`,
  `0x26c..0x26e`, `0x264..0x266`, then loops to `0x254`, `0x192`, `0x24a`,
  `0x24d`); `Build::queue_up (00620f40)` an age; `Leader::set_age
  (006d25a0)`/`set_epoch (006d26f0)` ages and epochs; `Leader::init`'s six
  the nation epochs and `GOV_EMPIRE`; the 37 recursive calls inside
  `gain_tech` stop below `0x275` (the last loop's bound, line 1842). The only
  plain-game caller that can pass a spell is **`Build::finished
  (00628490)`**: a queue entry of spell type whose `has_spell` is 0 at the
  finish falls past the spell arm (`62868e je 0x6284f3`), fails the building
  test, and reaches `gain_tech(entry)` at `628548` — "research the spell".
  With the shipped file that cannot fire: `has_spell` of a row without letter
  `a` is `type_avail`, and a `PREQ0 None` row's `type_avail` is never 0, while
  a `Disable` row cannot be queued. The rest (`ConsoleWin::run_cmd
  (007d6a70)`, `ScenarioFuncSet::gain_tech (009ee1a0)`, `gain_upgrade
  (009eeb90)`, `ScenarioEditor::research_tech (0099e9d0)`,
  `change_game_rule_combo (0099b260)`, `ScenarioRead::load_leader_chunk
  (009ac080)`, `ConquestGame::build_cities (00796140)`,
  `setup_scenario_bonuses (0078be90)`, `MapScenario::make (0068c8f0)`) are
  the console, triggers, editor, scenario files and CtW.
- **Clears**: `LeaderData::LeaderData (006d7540)` (`bits 0x326`, `size
  0x65`, `memset(ptr, 0, 0x65)`, `flags 1`); `Leaders::clear (006ed5d0)`
  (memset of each slot's array); `Leader::close (006b7f80)` (memset,
  `6b7fc0`); `Leader::init`'s `param_2 < 0` arm (`BitMask<17>::clear` at
  `6e39bd`, the same folded clear) and its unconditional
  `BitMask<806>::init` (`0x47d860`) of `tech` and `tech_at_start`
  (`6e41e7`, `6e41f3`; decompile lines 393–394) before any pass.
- **Not spell writers, checked**: `Leader::lose_tech (006d2850)` clears only
  for a unit type (vslot `0xc`) or a tech type (vslot `0x28`); a spell is
  neither and falls through to `reset_obs_flags` untouched.
  `MapScenario::make`'s direct store (`68cff4`) walks units `0x32..0x15f`.
  `ScenarioFuncSet::disable_type_by_tribe (00a006a0)` (`a00795`) clears a
  bit per slot — trigger layer. `Game::action_cheat_give_techs (00593180)`
  and `action_cheat_zero_techs (00593120)` set / clear all 806 — cheats.
  `LeaderData::walk_data (006d6750)` restores the array from a save.
  `LeaderData::log_data (006e5110)` (`6eb877`) is the dump's reader.

### 2. The event, the step, the predicate

The event is **`Leader::init` at game setup**, once per live leader:
`Setup::build_game (005ac190)` calls it for each slot whose `leader_flags &
1` is set (decompile line 618), before frame 0. Each bit steps 0 → 1 once.
The predicate, in the original's integers:

`bit t set` ⇔ `type_avail(t, 1) != 0` ∧ (`preq[0] != -1` ∨ `(spell_flags &
1) == 0`).

`type_avail` of a spell reduces, through the functions as read, to:
- `LeaderData::has_preq (006db810)`: each `TypeData::get_preq (00668700)`
  entry must be held — `-1` (`TYPE_NONE`) answers yes through `has_tech
  (006e0c80)`; a real type must be in `tech` (a spell's preq is never a
  bonus type in the shipped file);
- `LeaderData::type_eligible (006dbd10)`: `tribe_can_type (006d9380)` must
  be 4 (`tribe_mask` bit `tribe` set; a slot with `tribe -1` tests bit 31
  and fails); no `get_preq` entry below `-1` (the file's `Disable`); a
  non-epoch tech preq's `TechTypeData +0x1c8` age must not exceed
  `ending_technology`. A spell is not a good, unit, building or tech, so it
  then answers 4;
- `type_avail`'s tail: not a unit, not a building, vslot `0x2c`
  (`is_gov_type`) false → 4.

So the 46 `None` rows pass on every live leader with a nation; the 8
`Disable` rows fail `type_eligible`; `0x299` passes only if Industrial Age
was granted by the age pass above it (starting age past Industrial, or
`all_techs`), and under a non-Disabled preq the `a` letter is moot.
**Nothing re-runs the rule later**: no loop over `0x275..0x2ab` exists in
`gain_tech`, `set_age`, `set_epoch` or anywhere else that writes `tech`
(the listing's only `0x2ac` loop bound that writes is `6e4617`), so
`0x299` stays clear in a game that reaches Industrial Age by research.

### 3. What clears or resets it

`LeaderData::LeaderData`, `Leaders::clear`, `Leader::close`, and
`Leader::init` itself (its `BitMask<806>::init` precedes the pass). No
per-frame fold, no `reset_score`. Within a plain game nothing clears a
spell bit (`lose_tech` cannot).

### 4. Who reads it

- `LeaderData::has_spell (006e0bc0)`: reads the bit **only for a row with
  letter `a`**; otherwise it answers `type_avail`. The listing has three
  `testb $0x1, 0x1c8(%r)` sites in all `.text` — `has_spell` (`6e0c07`),
  `Leader::init` (`6e45ff`) and `BuildOut::make_options` (`62ade3`, the
  interface) — so `has_spell` is not inlined elsewhere. With no `a` row
  shipped, **no simulation decision in a plain game reads these bits**.
- `has_tech (006e0c80)` of a spell type would return the bit; no call with
  a spell constant was found in the export.
- `LeaderData::log_data (006e5110)` prints the mask (`BitMask::log_data`
  on `tech` and `tech_at_start`, decompile lines 2556–2557) — which is why
  the leaders line sees the bits at all.

### 5. Where `crates/sim` has the event

- The event is ported: **`TechTree::start`** (`crates/sim/src/tech.rs:2235`,
  "`Leader::init`'s tech block") runs the age/tech pass, the building pass
  (`tech` only) and the unit pass, then `reset_obs_flags` (`:2321`). It
  **stops before the spell pass**: the tree's `Kind`
  (`crates/sim/src/tech.rs:100`) ends at `Gov` (`0x26f–0x274`), so
  `PlayerTech::tech` (`:720`) has no slot for `0x275..0x2ab`.
- The craft rows are loaded elsewhere: `OrdersState::spells`
  (`crates/sim/src/orders.rs:217`, `orders::SpellType`,
  `crates/sim/src/orders/order.rs:95`), filled at
  `crates/rondata/src/load.rs:727`, carries `flags` (letter `a` is bit 0;
  `craft_flags`, `load.rs:2171`) but **not `PREQ0` nor `TRIBE_MASK`**.
- The emitter is `crates/sim/src/checksum/leaders.rs:711`,
  `bits_of(&t.tech)` mapped through `tree_type_index`, which can only name
  tree rows.
- `has_spell` is a SEAM: `crates/sim/src/cities/queue_disband.rs:173`
  ("this crate's tree holds no craft").

A builder adds the pass at the end of `TechTree::start`'s unit loop, before
`reset_obs_flags`, as the original orders it: for each craft row, set the
bit when its preqs are held (none, or a held type), no preq is `Disable`,
the tribe-mask bit is set, and (`preq[0]` is a type or letter `a` is
clear). That needs `PREQ0` and `TRIBE_MASK` loaded for the 55 rows and
either spell rows in the tree or a 55-bit side mask that the emitter adds
at `0x275 + i`. Only `tech` takes the bits, never `tech_at_start`.

### 6. Reached by a plain game?

Yes, at setup, on every live leader with a nation: the 46 bits are set
before frame 0 and stand unchanged at frame ~91 (and for the whole game).
`0x299` is reached only by a start at or past Industrial Age or an
all-techs game. The `gain_tech` route sits behind the console, triggers,
editor, scenario files and CtW, and behind a shipped row with letter `a`
(none).

### 7. Confidence and what is not read

High for the rule and the 46: the predicate is read in the listing
(`6e45d9`..`6e461d`), and the split of the rows follows from the shipped
columns. Not read: `TypeData::get_preq`'s arms past `param_1 == 2` (a
spell's preq 1 and 2 were taken as `None`, which the file's single `PREQ0`
column implies but the loader `SpellType::init` was not read for); the
digit arm of the `FLAGS` parse and the XML tag the parse reads (a
string-table index, `+0x1e424`, not resolved); `type_eligible`'s vslot
`0x40` test on a preq (moot for `None`); `tribe_can_type`'s second body
(`006d9410`); which slots `Setup::build_game` marks live (slot 0's tribe
and flags not checked); `ScenarioRead::load_leader_chunk`'s three
`gain_tech` calls' types; `BuildOut::make_options` beyond the flag test.
Every store site the two nets found is attributed above.

## Summary

| leaf | writers | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `tech.ptr`, bits `0x275..0x2ab` (46 set: every `PREQ0 None` row) | `Leader::init (006e3930)` spell pass (`6e45d9`..`6e461d`) — the one plain-game setter; `Leader::gain_tech (006dcb60)` via `Build::finished (00628490)` for a letter-`a` row (none shipped) and via console/trigger/editor/scenario/CtW; clears in `LeaderData::LeaderData (006d7540)`, `Leaders::clear (006ed5d0)`, `Leader::close (006b7f80)`, `Leader::init`; cheats and `disable_type_by_tribe (00a006a0)` | 0 → 1 once, at setup, per live leader: `type_avail(t,1) && (preq[0] != -1 \|\| !(flags & 1))`; `tech` only, not `tech_at_start` | `Leader::init` before the pass; no per-frame fold; `lose_tech` cannot clear a spell | `TechTree::start` (`crates/sim/src/tech.rs:2235`, before `reset_obs_flags` at `:2321`); needs `PREQ0`/`TRIBE_MASK` at `crates/rondata/src/load.rs:727` and an emitter path at `crates/sim/src/checksum/leaders.rs:711` | yes, before frame 0; `0x299` only at an Industrial-or-later start | high |
