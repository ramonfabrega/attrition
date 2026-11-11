# Item 4152, family 6 (`anti-att`): first reading

- Family: 6, slug `anti-att`
- Leaves: `anti_att` (`LeaderData` +0x7f4, `float`)
- Reader: Claude Opus 5.5 (claude-opus-5-5), wave 1 first reading, 2026-11-11
- Float class: `docs/DECISIONS.md` entry 120 §3 (decided at item 4104) —
  class 1 is "constant on the scored game, emitted as its bits", class 2
  "bits from a software float the crate holds", class 3 "neither: held out
  by name". This reading states what the writers produce.

Layout check: `types.txt:42205` names `LeaderData +0x7f4 float anti_att`.
`Leader` (0x6eec) lists no members of its own and `LeaderData` (0x6ee4) sits
at its offset 0 — `Leader::calc_anti_attrition` reads `this->field_0x8` as
`who` and indexes `leaders.list` (stride 0x6eec, base 0xe3a390) with it, and
its `field_0x7fc` is `take_att_disabled`. So a `Leader*`'s `0x7f4(%reg)` is
`anti_att`, with no prefix to subtract; the leaders-array form is
`0xe3ab84(%reg)` (0xe3a390 + 0x7f4).

## Leaf `anti_att` (+0x7f4, `float`)

### 1. Writers, both nets

**Export net.** `grep -rn 'anti_att'` over `funcs/` hits one reader by name
(`UnitData::get_attrition (00608fd0)`) and nothing else; `field_0x7f4` in a
leader context is in exactly two files: `Leader::calc_anti_attrition
(006cdcc0)` and `Leader::init (006e3930)` line 619. Every other
`field_0x7f4` hit in the export belongs to another class.

**Listing net.** The whole `.text` disassembled once; `0xe3ab84` has **zero**
hits (no inlined store through the leaders array), and the 40 `0x7f4(%reg)`
sites resolve as follows:

| store site | function | record | verdict |
|---|---|---|---|
| 6cdcce, 6cdcda, 6cdd5f, 6cdd8d, 6cdde8, 6cde3c, 6cde66, 6cde8e | `Leader::calc_anti_attrition (006cdcc0)` | `Leader*` in `esi` (= `ecx` on entry) | **writer** |
| 6e4955 (`andl %ecx,0x7f4(%ebx)`, `ecx` zeroed at 6e4949) | `Leader::init (006e3930)` | `Leader*` in `ebx` | **writer** (stores 0, then calls the writer above at 6e49c3) |
| 43619c | `Cliffs::~Cliffs (00436120)` | Cliffs | foreign |
| 56ddfe | `Constants::init (00569a90)` | Constants | foreign |
| 81a89e (`movups`, 16 bytes) | `IFaceResources::setup_rects (00819e70)` | interface | foreign |
| 8ffb60 | `GraphicPieces::close (008feed0)` | renderer | foreign |
| 90ecb5 | `GraphicPieces::~GraphicPieces (0090e720)` | renderer | foreign |
| 93e99b | `ChatSpace::ChatSpace (0093e8d0)` | interface | foreign |
| 5fee0f, 5ff36e, 67cfd5 | indexed `0x7f4(%r,%r,4)` adds/inc | arrays of other records | foreign (read-modify of an int array) |

No `$0x43800000` store to `+0x7f4` exists outside 006cdcc0, and nothing
calls the writer through a vtable: its three direct call sites are
`Leader::calc_unit_stats (006cf970)` at 6cf97b, `Leader::gain_tech
(006dcb60)` at 6dd3cf and `Leader::init (006e3930)` at 6e49c3.

**Bulk writers** (no displacement to find, found by the record's size):

- `Leaders::clear (006ed5d0)`: `memset(leader, 0, 0x6929)` per slot (the
  listing pushes `$0x6929` at 6ed5e0, base `0xe40fa4 − 0x6c14`), which
  covers +0x7f4: the bits become `0x00000000` (+0.0).
- `LoadGame::walk_function (0043d950)`: a loaded game `fread`/`gzread`s raw
  bytes into each walked range; `anti_att` lies in the leaders line's
  walked `[+0x8, +0x692a)`, so a load restores the saved bits verbatim. The
  save track's, not a plain game's. `WalkDataGame::walk_data` also calls
  `Leader::init` (5a30e6); the order of that call against the raw read was
  not read.

**Leader::init's callers** (it is not once-only per process, only per
game): `Setup::build_leader`, `Setup::build_game`, `Leaders::init` (the
plain game's), `MapScenario::make`, `ConquestGame::build_conquest_game`,
`ScenarioEditor::new_leader`, six sites in `ScenarioRead::load_leader_chunk`,
and `WalkDataGame::walk_data`.

### 2. The event, the step, the predicate

`anti_att` is not stepped; it is **recomputed from scratch** each time
`Leader::calc_anti_attrition (006cdcc0)` runs. Its value is a pure function
of five inputs read at that moment plus four constants:

1. Store 256.0 (`0x43800000`, listing 6cdcce — the store precedes the test).
2. If `take_att_disabled` (+0x7fc) ≠ 0: store `0x00000000` and stop.
3. Foraging, **highest tier only**, not cumulative: `has_preq(FORAGING_3)`
   (TypeIndex 0x300) picks `attrition_upgrade[2]`, else `FORAGING_2`
   (0x2ff) picks `[1]`, else `FORAGING_1` (0x2fe) picks `[0]`, else no
   factor. `attrition_upgrade` is `Constants +0x1c8` (int[]); the writer
   indexes only 0..2. If the chosen `pct ≥ 100`: store 0 and stop.
4. Statue of Liberty: `has_wonder(0x219)` (`STATUEOFLIBERTY`), pct =
   `liberty_attrition` (`Constants +0x524`).
5. Mongol: `has_tribe_bonus(0x11)`, pct = `mongol_attrition` (`+0x7f0`).
6. Titanium: bit 30 of `rare.ptr` (`+0x6da7 & 0x40`) **or** of
   `rare_conquest.ptr` (`+0x6dcf & 0x40`) — i.e. an inlined
   `has_rare(TITANIUM = 0x24)` (`has_rare (006e0770)` indexes `id − 6`);
   pct = `titanium_attrition` (`+0x95c`).

For steps 3–6, each held source with `pct ≥ 100` stores `0x00000000` and
returns at once (later sources are not read); otherwise

    anti_att := divss( mulss(anti_att, 100.0f), cvtdq2ps(100 − pct) )

in that order, **single precision, SSE scalar** (listing 6cdd6e–6cdd8d and
its three twins: `movss`, `mulss [0xb696ac]` = 100.0f `0x42c80000`,
`cvtdq2ps`, `divss`, `movss` back). Each step rounds to nearest-even in
`f32`; there is no x87 extended intermediate. The order Foraging →
Liberty → Mongol → Titanium is observable in the bits (see below).

**Predicates worth naming.**
- The predicates read `leaders.list[this->who]`, the store goes to `this`;
  in every caller these are the same record.
- `has_tribe_bonus (006e1370)` answers 0 when the game's `info.flags & 4`
  (the lobby's "No Nation Powers", per `docs/TECH.md:386`), **or when the
  game has no starting town and the leader has `city_num == 0`** (a nomad
  start before the first city), or `tribe < 0`; a CtW
  `conquest_racial_powers` bit answers 1; `leader_flags2 & 0x40` answers 0;
  else the tribe's bonus index must equal 17.
- `has_wonder (006ebc10)` answers nonzero for a CtW `conquest_wonders` bit
  once the civics epoch reaches `ctw_wonder_min_civics`, or for a live
  wonder object of that type whose city index (`short +0x72`) is ≥ 0.
- With shipped `LIBERTY_ATTRITION = 100` (`crates/sim/src/core/tuning.rs:1440`)
  the Statue makes the value 0 outright.

**When it runs** (each a recompute, not a step):
- `Leader::init` — every game start, after zeroing it (6e4955, 6e49c3).
- `Leader::calc_unit_stats (006cf970)` first two lines (`calc_attrition`,
  `calc_anti_attrition`), reached from:
  - the per-frame fold in `Leaders::process_all (006ed2a0)` and
    `Leader::process (006b88b0)`: when `leader_flags & 0x4000000`, clear it
    and run `calc_unit_stats`. That bit is raised by
    `Leader::gain_tech`'s tail (`|= 0xc000000`, unconditional, line 2313;
    and `|= 0x4000000` for a unit-type tech at line 661),
    `Leader::lose_tech` (`|= 0xc000000`), `Leader::gather` /
    `Leader::calc_gather` when the assembled `rare` mask changes
    (`|= 0xc000000`, which is how titanium arrives — `gather` runs before
    the dirty test in the same `process` call), `Build::activate` for a
    **wonder only** (types 0x20e..0x21e, line 1970; other buildings jump
    past to `00626f11`), and `Build::close` for a wonder's close (line 319).
  - `Leader::set_age (006d25a0)` and `Leader::set_epoch (006d26f0)`
    directly (scenario/editor age setting).
  - Scenario editor and trigger functions directly: `ScenarioEditor::
    change_game_rule_combo`, `add_build`, `research_tech`, `new_leader`;
    `ScenarioFuncSet::gain_upgrade`, `lose_upgrade`, `create_building`,
    `set_object_type_{max_health,armor,attack,max_range,min_range}`,
    `set_unit_type_speed`, `set_unit_type_max_craft`,
    `set_type_line_of_sight`.
- `Leader::gain_tech (006dcb60)` line 433 directly, when the tech passes
  a type predicate (vtable `+0x38`), its techtype `+0x14 == 1`, and the
  civics epoch now equals `ctw_wonder_min_civics` — the moment CtW
  conquest wonders switch on. Redundant with the tail's dirty bit in a
  plain game.

**Staleness.** Because the field is cached, it lags any input that moves
without raising `0x4000000`: `take_att_disabled` is toggled by
`ScenarioFuncSet::enable_take_attrition (00a02880)` and its sibling at
00a028c0 (stores 0 / 1 to `0xe3ab8c`) with no recompute; a Mongol nomad's
first city flips `has_tribe_bonus` with no wonder/tech/rare event of its
own; a Liberty captured with its city depends on the capture path raising
the bit (not read).

**The bits, shipped constants** (`attrition_upgrade = 25, 50, 75`,
`LIBERTY 100`, `MONGOL 50`, `TITANIUM 50`; `tuning.rs:1432–1442`). Computed
here by IEEE single rounding of each `mulss`/`divss` in the writer's order:

| Foraging | Mongol | Titanium | bits | value | exact rational |
|---|---|---|---|---|---|
| — | — | — | `0x43800000` | 256.0 | 256 |
| — | one of the two | | `0x44000000` | 512.0 | 512 |
| — | ✓ | ✓ | `0x44800000` | 1024.0 | 1024 |
| 1 | — | — | `0x43aaaaab` | 341.33334350586 | 1024/3 |
| 1 | one of the two | | `0x442aaaac` | 682.66674804688 | 2048/3 |
| 1 | ✓ | ✓ | `0x44aaaaac` | 1365.3334960938 | 4096/3 |
| 2 | — / one / both | | `0x44000000` / `0x44800000` / `0x45000000` | 512 / 1024 / 2048 | same |
| 3 | — / one / both | | `0x44800000` / `0x45000000` / `0x45800000` | 1024 / 2048 / 4096 | same |
| any | Liberty held, or `take_att_disabled` | | `0x00000000` | +0.0 | (immune) |
| — | (after `Leaders::clear`, before `init`) | | `0x00000000` | +0.0 | |

**The load-bearing finding for entry 120 §3**: the Foraging-1 chain is
*not* the nearest single to its exact rational. 1024/3 rounds to
`0x43aaaaab`; ×100 rounds to 34133.3359375 (`0x47055556`); ÷50 gives
`0x442aaaac`, where `f32(2048/3)` is `0x442aaaab`. Likewise 4096/3 lands on
`0x44aaaaac`, not `0x44aaaaab`. So `crates/sim`'s exact rational
(`territory::attrition::Resistance`) gives the right **integer** after
`get_attrition`'s truncation (all 16 truncate equal; the crate's test
`every_reachable_resistance_lands_on_a_known_integer` pins eight) but
**cannot give the bits by rounding once**. The bits follow from integers
the simulation holds only by replaying the writer's chain through the
software float: `Single::from_bits(0x43800000)`, then per held source
`.mulss(Single::from_bits(0x42c80000)).divss(Single::from_i32(100 − pct))`
(`crates/sim/src/num/single.rs:232, 254, 155`). That is **class 2** of
entry 120 §3 (bits from the software float, no allowance added to
`no_float.rs`), not class 3; and on a game where no leader holds Foraging,
Liberty, the Mongol bonus or titanium it is **class 1**, the constant
`0x43800000` — which is what entry 120 §3.4 read on run 1562 frame 91 and
run1100 frame 5408. The rational and the bits agree on every value except
the three Foraging-1 rows, where only the replayed chain is right.

### 3. What clears or resets it

- `Leaders::clear (006ed5d0)`: memset → `0x00000000`.
- `Leader::init (006e3930)`: stores 0, then `calc_anti_attrition` → 256.0
  or the modified value (a CtW/scenario leader can start with a tier).
- No per-frame fold writes it unconditionally; the `0x4000000` fold
  recomputes only when the bit is set. `reset_score` does not touch it
  (no `0x7f4` in its listing range).

### 4. Readers

- `UnitData::get_attrition (00608fd0)` — the one decision: on the
  non-militia branch, `cvttss2si( mulss( mulss(cvtdq2ps(scale), anti_att),
  0x3b800000 = 1/256 ) )` (listing 60912c–60914e), `scale` = 256, or
  `25600 / (100 − siege_attrition)` = 512 for siege. Both scales and 1/256
  are powers of two, so the product is exact and the result is
  `trunc(anti_att · scale / 256)` of the stored single. 0.0 makes the
  unit immune. Militia bypass `anti_att`.
- `LeaderData::log_data (006e5110)` prints it (6e8947), the
  `anti_att 256.000000` `docs/ATTRITION.md:839` quotes.
- `LeaderData::walk_data (006d6750)` folds its 4 bytes in the leaders
  line; `SaveGame`/`LoadGame::walk_function` write and restore them.
- `LeaderData::get_anti_attrition (006e0b70)` is `movss 0x7f4(%ecx),%xmm0;
  ret` (the decompile's `in_ST0` is the xmm0-return trap); the listing has
  no direct call to it.

### 5. Where `crates/sim` has the event

- The formula exists: `territory::attrition::resistance` and
  `ResistanceMods` (`crates/sim/src/territory/attrition.rs:102, 132`),
  as an exact rational, stored in `PlayerState.resistance`
  (`attrition.rs:268`, default `Resistance::BASE`). **Nothing outside tests
  calls `resistance()`**; `docs/ATTRITION.md:989` says it is unwired, and
  every leader's resistance is the base 256.
- The trigger already exists: `Sim::calc_unit_stats`
  (`crates/sim/src/economy/rares.rs:679`) is run by the per-frame fold at
  `crates/sim/src/core/frame.rs:188` on `roster.unit_stats_dirty`, which is
  raised by `Sim::apply_gained` (`crates/sim/src/tech/gain.rs:607`), the
  rare-mask change (`core/frame.rs:165`) and wonder activate/close
  (`crates/sim/src/cities/city.rs:2491, 2887`). Its doc comment
  (`economy/rares.rs:47`) names `calc_attrition` and `calc_anti_attrition`
  as calls it does not make. A builder adds the recompute **there**, and
  at game start beside the crate's `Leader::init` analogue, and in
  `apply_gained` next to `self.calc_attrition(who)`
  (`tech/gain.rs:608`), which currently runs `calc_attrition` eagerly.
- The inputs it needs exist: tech held (`tech_tree.has_tech`), Liberty
  (`PlayerTech::has_wonder`, `crates/sim/src/tech.rs:814`), the Mongol bonus
  (`TechTree::has_tribe_bonus`, `tech.rs:1102`, whose "No Nation Powers" /
  nomad-no-city arms should be checked against 006e1370 when wired),
  titanium (`economy.ledgers[w].rare`, bit `TITANIUM − BASE_RARE`), and
  `PlayerState.take_disabled`.
- The checksum leaf is `("anti_att", 0x7f4, 4, 1)` at
  `crates/sim/src/checksum/leaders.rs:151`, with no value arm (unheld,
  like `plunder_scale` was before its `PLUNDER_SCALE_BITS` arm at line 1238).
  Emitting it means holding the `Single` bits beside the rational in
  `PlayerState`, computed by the chain in §2.

### 6. Reached by a plain game?

- `Leader::init` → 256.0: every game. **Yes.**
- `calc_unit_stats` via the dirty fold: every tech, rare change and wonder
  in a skirmish. **Yes** — but the value moves only if a leader holds one
  of the four sources:
  - Foraging 1–3: researchable techs in a skirmish (whether the AI's
    scripts research them was not read).
  - Statue of Liberty: a skirmish wonder → `0x00000000`.
  - Mongols: a skirmish nation (unless "No Nation Powers", or a nomad
    start before the first city).
  - Titanium: a skirmish rare resource.
- `take_att_disabled`: trigger layer only (`ScenarioFuncSet`). **No.**
- `rare_conquest`, `conquest_wonders`, `conquest_racial_powers`, the
  civics-threshold call in `gain_tech`: CtW. **No.**
- `set_age`, `set_epoch`, `ScenarioEditor::*`, `ScenarioFuncSet::*`:
  scenario/editor. **No** (whether `set_age` is reached by a lobby's
  starting age was not read).
- `LoadGame::walk_function`: the save track.

### 7. Confidence and what is not read

- **High** on the writer set: both nets agree (two named writers, one
  memset, one raw load), the arithmetic is read from the listing, the
  bit table is computed from it. **High** that the bits differ from the
  rational on the Foraging-1 rows (arithmetic, not a reading).
- **Medium** on the trigger list: the `0x4000000`/`0xc000000` setters were
  found by decompile grep for `| 0x4000000;` and `| 0xc000000;`; a set
  spelled with another combined constant, or through a register in the
  listing, would be missed (the listing has no `orl $0x4000000` to a
  leader at all, so every set goes through a register or a combined word).
- Not read: `has_preq (006db810)`'s arm for `FORAGING_n` (assumed plain
  has-tech); the type predicate at `gain_tech`'s vtable `+0x38`; the order
  of `WalkDataGame::walk_data`'s `Leader::init` against the raw read; the
  capture path's raising of the bit for a captured Liberty; whether the AI
  researches Foraging; `Build::activate`'s full path to line 1970 beyond
  the wonder guard at line 1250.
- No store left unattributed among the 40 `0x7f4(%reg)` sites.

## Summary

| leaf | writers | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `anti_att` (+0x7f4, f32) | `Leader::calc_anti_attrition (006cdcc0)` (8 stores); `Leader::init (006e3930)` (0, then the former); `Leaders::clear (006ed5d0)` (memset); `LoadGame::walk_function (0043d950)` (raw restore) | recompute, not a step: 256.0 then per held source (Foraging top tier → Liberty → Mongol → titanium) `divss(mulss(x,100.0f), 100−pct)` in f32; any pct ≥ 100 or `take_att_disabled` → +0.0. Bits for shipped data in §2; Foraging-1 rows are `0x43aaaaab`/`0x442aaaac`/`0x44aaaaac`, not the rational's nearest | `Leaders::clear` → 0; `Leader::init` → 0 → recompute | formula `territory/attrition.rs:102` (unwired); add to `Sim::calc_unit_stats` `economy/rares.rs:679` (fold `core/frame.rs:188`), init, `tech/gain.rs:608`; leaf `checksum/leaders.rs:151` | yes (init; Foraging, Liberty, Mongols, titanium); `take_att_disabled` trigger-only; `rare_conquest` etc. CtW | high (writers, arithmetic); medium (trigger list) |

Entry 120 §3 class: **2** in general (the writer's single chain is replayed
exactly by `num::single::Single` from integers the crate holds), **1** on a
frame where no leader holds a source (`0x43800000`). Not class 3 on the
evidence of the writer: the "out by name" in `docs/CHECKSUM.md:543, 597` is
a choice, not forced.
