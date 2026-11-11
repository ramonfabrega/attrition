# Item 4152, family 17 (`spells`): adjudication

- Slug: `spells`. Leaf: `LeaderData::tech` (+0x6c0c, `BitMask<806>`),
  bits `0x275..0x2ab`.
- Readings: `spells-reading.md` (first), `spells-second.md` (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text`
  listing (`exe.s` in the job scratch), `crates/sim` grep, and an
  independent re-read of run 1562's frame-91 packet. Nothing run of the
  original.

## Leaf `tech.ptr`, bits `0x275..0x2ab`

### 1. Claim by claim

| # | claim | first | second | mark |
|---|---|---|---|---|
| 1 | the one setter in a plain game is `Leader::init (006e3930)`'s spell pass, `6e45d9..6e461d`, writing `tech` only | yes | yes | agree |
| 2 | predicate: `type_avail(t,1) != 0` and (`preq[0] != -1` or `spell_flags & 1 == 0`) | yes | yes | agree |
| 3 | `type_avail` of a spell reduces to `has_preq` + `type_eligible`: tribe bit in `tribe_mask`, no preq below -1, preq -1 or held, a tech preq's age within `ending_technology` | yes | yes | agree |
| 4 | the tribe test is `tribe_can_type (006d9380)` (first; `006d9410` "second body, not read") vs `tribe_can_type (006d9410)` (second) | 006d9380 | 006d9410 | disagree |
| 5 | the eight `-2` rows are refused by `type_eligible` (first) / by both `type_eligible` and `has_preq` (second) | `type_eligible` | both | one-sided (the `has_preq` half) |
| 6 | split: 46 set (every `preq[0] == -1` row), 8 clear (`-2`), `RADARSPELL` 0x299 clear unless Industrial Age is held at init | from `craftrules.xml` columns | from the loaded `SPELLTYPE` dump and the packet | agree |
| 7 | no spell row has `spell_flags` bit 0 (letter `a`); every `tribe_mask` is 24 ones | yes | yes | agree |
| 8 | `gain_tech (006dcb60)` sets bit `param_1` unconditionally (line 303); no loop inside it reaches `0x275` | yes | yes | agree |
| 9 | the only plain-game route into `gain_tech` with a spell is `Build::finished (00628490)`'s fall-through when `has_spell` is 0 | yes | yes | agree |
| 10 | that fall-through cannot fire with the shipped data (first) / is reachable by "a queued spell that lost availability" (second) | cannot | can, rarely | disagree |
| 11 | clears: `LeaderData::LeaderData (006d7540)`, `Leaders::clear (006ed5d0)`, `Leader::close (006b7f80)`, `Leader::init`'s `BitMask<806>::init` and its `param_2 < 0` arm | yes | yes | agree |
| 12 | `Leader::lose_tech (006d2850)` cannot clear a spell (unit or tech arms only) | yes | yes | agree |
| 13 | out of plain play: cheats (`action_cheat_give_techs (00593180)`, `action_cheat_zero_techs (00593120)`), console, `disable_type_by_tribe (00a006a0)`, scenario/editor/CtW, `walk_data` | yes | yes (no `zero_techs`) | agree |
| 14 | `Leader::init`'s callers: `Setup::build_game (005ac190)` (first); plus `Setup::build_leader (005abc70)` as a second plain-game caller, and `load_leader_chunk` "three sites" (second) | build_game | build_game + build_leader | one-sided |
| 15 | readers: `has_spell (006e0bc0)` reads the bit only for a letter-`a` row; `has_tech (006e0c80)` reads it raw; `log_data`; no plain-game sim decision reads the bits | yes | yes | agree |
| 16 | crate: `TechTree::start` (`crates/sim/src/tech.rs:2235`) has no spell pass; `Kind` ends at `Gov`; `bits_of`/`tree_type_index` in `checksum/leaders.rs` cannot name `0x275+`; the SEAM at `cities/queue_disband.rs:173` | yes | yes | agree |
| 17 | crate: `orders::SpellType` (loaded at `crates/rondata/src/load.rs:727`) has `flags` but not `PREQ0`/`TRIBE_MASK` | yes | "did not check" | one-sided |
| 18 | plain game reaches it: yes, at setup, every live leader | yes | yes | agree |
| 19 | run 1562 frame 91: 46 of 55 set in slots 0 and 1, none in `tech_at_start`, the nine clear = eight `-2` + `RADARSPELL` | — | yes, from the packet | one-sided |

### 2. Predicates, spot-checked against the body (claims 1, 2, 3, 6, 7)

- **Confirmed.** Listing `6e45d9..6e461d`: `t` from `0x275`, `call
  0x6e33a0` (`type_avail`) with `(t, 1)` (`6e45de/6e45e0`), `je` skips on
  zero (`6e45ed`); then the type row from `0xc061a8 -> +0x10 [t]`,
  `cmpl $-1, 0x30(row)` (`6e45fa`), `jne` to the set; else `testb $1,
  0x1c8(row)` (`6e45ff`), `jne` skips; set is `leal 0x6c0c(%ebx)` +
  `call 0x450360` with `(t, 1)` (`6e4608..6e4611`); bound `cmpl $0x2ac`
  (`6e4617`, exclusive). No store to `0x6c80` (`tech_at_start`) in the loop.
  The next instruction is `call 0x6e32a0` (`reset_obs_flags`, `6e4621`), so
  the pass sits between the unit pass and `reset_obs_flags`, as both said.
  Deciding line: `6e45fa`/`6e45ff`/`6e460b`.
- `type_avail (006e33a0)` lines 15–21: `has_preq` zero returns 0, then
  `type_eligible` non-4 returns as is; a spell then falls through the unit
  (vslot `0xc`), building (`0x14`) and government (`0x2c`) arms to 4.
- `type_eligible (006dbd10)`: line 21 `tribe_can_type != 4` returns 0; the
  preq loop (lines 39–67) returns 0 for any `get_preq < -1` (line 43) and
  for a non-bonus, non-epoch tech preq whose `techtypes[q] +0x1c8` exceeds
  `ending_technology` (lines 55–57); the tail (lines 199–213) returns 4 for
  a type outside `0x220..0x274` in the default `is_tech_type`. A spell
  answers 4 when its preqs pass. Deciding line: `type_eligible (006dbd10)`
  lines 43, 199–207.
- **The values the predicate meets, confirmed independently.** I parsed
  run 1562's own `SPELLTYPE` records (`gamelog_p.txt`, frame 91 block,
  55 records, `type` exactly `0x275..0x2ab` in order): every `tribe_mask`
  is `16777215`; no `spell_flags` is odd; `preq` is `[-1,-1,-1]` for all but
  `0x276, 0x278, 0x279, 0x27e, 0x282, 0x284, 0x285, 0x287` (`[-2,-1,-1]`)
  and `0x299` (`[548,-1,-1]`, `INDUSTRIAL_AGE` 0x224). The 46/8/1 split
  both readers gave is the loaded data's, not only the file's.

### 3. Claim 4, which `tribe_can_type`

**Corrected (minor), for the second.** `type_eligible`'s call is `6dbd1e:
call 0x6d9410`, the `TypeIndex` overload `tribe_can_type (006d9410)`. The
first reading's `006d9380` is the `Type*` overload, which `type_eligible`
does not call (it is called from `Object::eject_contents (0064cd20)` at
`64d09f`, from `type_eligible`'s graft walk, and elsewhere). The two bodies
compute the same thing for a spell: not a unit (vslot `0xc`), so straight to
`(1 << (tribe & 31)) & type +0x10` (`tribe_mask`), times 4. A slot with
`tribe -1` tests bit 31, which a 24-bit mask never has, so it fails — the
first's remark holds through either body. Deciding line: `6dbd1e`.

### 4. Claim 5, the `-2` rows in `has_preq`

**Confirmed (the second's extra half).** `has_preq (006db810)` lines
136–140: a negative `get_preq` goes to `has_tech`; `has_tech (006e0c80)`
lines 13–16 answer 1 only for `TYPE_NONE` (-1) and 0 for
`TYPE_ANY_BUILDING` (-2, `enums/TypeIndex.txt` line 868). So a `-2` row is
refused twice: `has_preq` first (it runs first in `type_avail`), then
`type_eligible` line 43 would refuse it too. Immaterial to the result;
material only to a port that skips one of the two checks.

### 5. Claims 9 and 10, `Build::finished`'s fall-through

- Claim 9 **confirmed.** `628687: call 0x6e0bc0` (`has_spell`), `62868e: je
  0x6284f3`; from there the building range test (`62850e`/`628515`) fails
  for a spell and `628548: call 0x6dcb60` (`gain_tech`) runs with the queued
  type in `%esi`.
- Claim 10 **corrected, for the second; the first's "cannot fire in a plain
  game with the shipped data" stands.** With no letter-`a` row, `has_spell`
  is `type_avail(t, 1)` (`has_spell (006e0bc0)` lines 21–33). For the
  fall-through, a spell must be queued while available and be unavailable at
  the finish. `BuildData::can_make (0062db10)` lines 27–37 refuse any queue
  whose `type_avail` is 0, so the spell was available at queueing. Nothing in
  a plain game takes availability away afterwards: the tribe is fixed;
  `ending_technology` is fixed; a `-1` preq is always held; `RADARSPELL`'s
  preq is an age, and ages are lost only through `Leader::set_age
  (006d25a0)` (lines 26–70 call `lose_tech`), whose callers are
  `ConsoleWin::run_cmd`, `ScenarioFuncSet::gain_next_age`/`lose_current_age`
  and `UnitBalance::next` — none plain-game. So the path is dead in a plain
  game with this ruleset; the second's "needs a queued spell that lost
  availability" is a true condition that nothing in a plain game meets.
  Deciding lines: `can_make (0062db10)` line 35, and `set_age`'s caller set.
  **Open, small:** a malformed lockstep queue order that bypasses
  `can_make` was not traced.

### 6. Claims 11–13, the writer list, re-netted

**Confirmed complete.** My own nets:

- Export: `grep -rln 'field_0x6c0c|field_0x6c18|field_0x6c1[0-9a-f]|->tech|.tech'`
  over `funcs/` gives 42 files; the writing lines are exactly
  `LeaderData::LeaderData` (lines 23–26), `Leader::close` (17–19),
  `Leader::lose_tech` (19, 107), `Leader::gain_tech` (303, 857),
  `ConsoleWin::run_cmd` (6504), the two cheats, `MapScenario::make` (320),
  `Leaders::clear` (11), `disable_type_by_tribe` (65–108), plus
  `Leader::init`, `walk_data` and `log_data`.
- Listing: every instruction in `exe.s` with a displacement in
  `0x6c0c..0x6c7f` off a register, or an absolute/indexed address inside
  any of the eight slots' `tech` struct (`0xe40f9c + k*0x6eec`, 0x74 bytes),
  attributed to its function by the export's address index: 57 functions;
  all are `testb`/`movzbl`/`movb`-to-register loads except the stores and
  `lea`/`add`s in the functions just listed. The only non-load `movb` into
  the array is `MapScenario::make` at `68cff4` (unit range, scenario). Also
  every immediate `$0x6c0c..$0x6c7f` and `$0xe40f9c..` in `.text`: the
  extras (`SetupWin::setup_buttons`, `ConquestSetupWin::*`,
  `GameSpy::join_staging_room`, `ScriptGameInterface::init`) add those
  constants to string-table bases (`0xc06378`, `0xc8cd00`), not to a leader.
- No writer neither reader named. The second omitted
  `action_cheat_zero_techs (00593120)`; the first had it.

### 7. Claim 14, `Leader::init`'s callers

**Corrected (minor), for the second.** The direct `call 0x6e3930` sites in
`.text`: `WalkDataGame::walk_data` (`5a30e6`), `Setup::build_leader`
(`5abc8d`), `Setup::build_game` (`5acef7`), `MapScenario::make` (`68cf18`),
`Leaders::init` (`6ed8cb`), `ConquestGame::build_conquest_game` (`797e52`),
`ScenarioEditor::new_leader` (`9a1575`) and `ScenarioRead::load_leader_chunk`
at **six** sites (`9ac1a9`..`9ac69f`), not three. `Setup::build_leader
(005abc70)` has **no caller**: no `call`/`jmp` to `0x5abc70` in `.text`, and
its address does not occur as data in the PE (`70bc5a00` absent). So the
plain-game caller is `Setup::build_game (005ac190)` alone, as the first had
it. Deciding line: the `exe.s` grep for `0x6e3930` and `5abc70`.

### 8. Claim 15, the readers: one neither reader found

**Confirmed, with an addition.** `has_spell (006e0bc0)` reads the bit only
under `spell_flags & 1` (lines 21–27), else returns `type_avail`;
`has_tech (006e0c80)` line 37 reads it raw for a non-unit, non-building
type. Of the listing net's variable-index `testb` readers in simulation
code, each was checked for its type gate: `Build::do_queue` `61e5aa` and
`61ec02`, `Build::queue_up` `62165f`/`62182e`, `Group::action_queue_up`
`6fdde5`, `ObjectData::train_time` `650a90`/`650ade` (units) and `65167e`
(buildings), `TypeData::get_cost` lines 159/425 (units) and 705
(buildings), `BuildData::count_queue` (units), `has_tech_queued` (techs),
`reset_obs_flags` (units `< 0x192`): none can index a spell.

**One can: `Build::do_queue (0061e410)` at `61e501`** (decompile lines
93–98). When the building `is(MISSILESILO 0x208, 1)` (`61e4b6..61e4d6`) and
its `inside_down` (`+0x28`) is not negative (a missile already inside), and
the head-of-queue type's bit in `tech` is set, and that type is not
`DISBAND` (0x29a), `do_queue` returns without progressing the queue. This
reads `tech` for whatever type is queued, spell or not. `DISBAND`'s bit is
one of the 46 set at init, and the explicit `!= DISBAND` is what keeps a
loaded silo able to disband. By run 1562's loaded `SPELLTYPE` records no
other spell names the silo (`from`, `from2`, `where` are `-1` only for
`DISBAND`; the rest name other lineages), so in a plain game this reader
meets a spell bit only for the one spell it exempts. Effect today: none.
For a builder: if the spell bits are ported, `do_queue`'s silo test must
keep the `DISBAND` exemption, or a loaded silo stalls a queued disband.
**Open:** `BuildTypeData::queue_here` for the silo was not read, so "no
other spell can be queued at a silo" rests on the loaded `from/where`
columns. Deciding line: `61e501`, `61e4a8` (the earlier `DISBAND` test),
`61e508`.

The other spell-bit readers are interface (`BuildOut::make_options` eight
sites, `IFace*`, `WallOut::draw`, `ConsoleWin::tell_tech`), the scenario
writer (`ScenarioWrite::save_leaders_chunk`), CtW
(`ConquestGame::setup_scenario_bonuses`), `log_data`, and the windowed
checksum. So the readers' "no simulation decision in a plain game reads
these bits" holds in effect, through one exempted reader.

### 9. Claims 16–17, the crate site

**Confirmed.** `TechTree::start` is `crates/sim/src/tech.rs:2235`; its last
pass (the building/unit loop, `:2290–:2320`) is followed by
`self.reset_obs_flags(setup, p)` at `:2321`; `Kind` (`:100`) ends at
`Gov { tier, column }` (`:117`), so `PlayerTech::tech` (`:720`,
`Vec<bool>` over tree entries) has no spell slot. `bits_of` at
`crates/sim/src/checksum/leaders.rs:704–710` maps through
`tree_type_index` (`:524`). The SEAM is `crates/sim/src/cities/
queue_disband.rs:172–174`. `orders::SpellType`
(`crates/sim/src/orders/order.rs:95`) carries `job_time`, `flags`, `range`,
`mana`, `from`, `duration`, `duration_upgrade` — no `preq`, no
`tribe_mask` — filled at `crates/rondata/src/load.rs:725–740` from the
craft records (`flags` by `craft_flags`, `load.rs:2171`). The first's claim
17 is confirmed; the second left it unchecked. The diff side lists
`("tech.ptr", 2)` at `crates/rondata/src/diff/checksum_lines.rs:1839`.
Home of the event: `TechTree::start`, between the unit loop and
`reset_obs_flags` — the original's order (`6e461d` then `6e4621`).

