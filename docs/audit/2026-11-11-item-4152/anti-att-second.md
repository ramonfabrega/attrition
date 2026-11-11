# anti-att — second (blind) reading

- Item: 4152, wave 2
- Slug: `anti-att`
- Leaves: `anti_att` (`LeaderData` +0x7f4, float)
- Reader model: Claude Opus 5.5 (claude-opus-5-5)
- Blind: the first reading and the other forbidden files were not opened.

## `anti_att` — `LeaderData` +0x7f4, `float`

Layout check: `Leader` (0x6eec) prints no fields in `types.txt`; `LeaderData`
(0x6ee4) sits at `Leader`+0 — `Leader::calc_anti_attrition (006cdcc0)` reads
`who` as `this+0x8` and indexes `0xe3a390 + who·0x6eec` with it, and
`ScenarioFuncSet::disable_take_attrition (00a028c0)` stores
`take_att_disabled` at `0xe3ab8c` = `0xe3a390+0x7fc`. So a `Leader*` and a
`LeaderData*` share offsets, and the global address of this leaf for slot
`i` is `0xe3ab84 + i·0x6eec`.

### 1. Every writer, by both nets

Export net: `grep -- '->anti_att\b'` finds nothing (no named access).
`grep 'field_0x7f4\b'` finds two functions:
`Leader::calc_anti_attrition (006cdcc0)` (eight stores) and
`Leader::init (006e3930)` (`:619`, store 0). No other `this` class shifts
the offset (prefix 0).

Listing net (`exe.s`, every `, 0x7f4(%reg)` destination plus one-operand
forms; `0xe3ab84` absolute: zero hits):

| site | function | base | verdict |
|---|---|---|---|
| `6cdcce` `movl $0x43800000` | `Leader::calc_anti_attrition (006cdcc0)` | `esi`=`this` (Leader) | **writer** |
| `6cdcda`, `6cdd5f`, `6cde66` `movl $0` | same | same | **writer** |
| `6cdd8d`, `6cdde8`, `6cde3c`, `6cde8e` `movss %xmm1` | same | same | **writer** |
| `6e4955` `andl %ecx(=0)` | `Leader::init (006e3930)` | `ebx`=`this` | **writer** (0) |
| `56ddfe` | `Constants::init (00569a90)` | `Constants*` | not this leaf |
| `5fee0f`, `5ff36e` `addl …,0x7f4(%edx,%ecx,4)` | `Unit::fight (005fd4d0)` | `edx` = `[0xc061ec]`, a per-who int array of another global (leaders are reached as the static `0xe3a390` or `[0xc061e0]`) | not this leaf |
| `67cfd5` `incl 0x7f4(%ecx,%eax,4)` | (same `[0xc061ec]` base, flag `0x821`) | — | not this leaf |
| `43619c`, `81a89e`, `8ffb60`, `90ecb5`, `93e99b` | `Cliffs::~Cliffs`, `IFaceResources::setup_rects`, `GraphicPieces::close`, `GraphicPieces::~GraphicPieces`, `ChatSpace::ChatSpace` | their own `this` | not this leaf |

A third, range writer: `LeaderData::walk_data (006d6750)` walks
`[&who, &field_0x692a)` raw through the `DataWalk`'s first slot, which spans
+0x7f4. On an input walk (save-game load, `WalkDataGame::walk_data
(005a2360)` → `Leaders::walk_data (006e38e0)`) that writes the saved bits
back verbatim. Whether any multiplayer join or resync drives an input walk
was not read.

No inlined copy of `calc_anti_attrition` exists: the immediate
`0x43800000` lands on `+0x7f4` exactly once in `.text`.

Callers of `calc_anti_attrition` (`calll 0x6cdcc0`, three, no `jmp`):
`Leader::calc_unit_stats (006cf970)` at `6cf97b`, `Leader::gain_tech
(006dcb60)` at `6dd3cf`, `Leader::init (006e3930)` at `6e49c3`.

### 2. The event, the step, the predicate

`calc_anti_attrition` is a **pure recompute**, never an increment. It reads
only the leader's state and five integer constants, and stores a single:

1. Store `0x43800000` (256.0).
2. If `take_att_disabled` (+0x7fc) ≠ 0 → store `0x00000000`, return.
3. Foraging tier: the first of `has_preq(FORAGING_3 = 0x300)`,
   `has_preq(FORAGING_2 = 0x2ff)`, `has_preq(FORAGING_1 = 0x2fe)` that holds
   picks index 2, 1, 0 of `Constants::attrition_upgrade` (+0x1c8, `int[4]`).
   None held → skip this step. If the entry ≥ 100 → store 0, return; else
   `x = (x ·ss 100.0) /ss (float)(100 − entry)`.
4. `has_wonder(0x219 = STATUEOFLIBERTY)`: `liberty_attrition` (+0x524), same
   rule.
5. `has_tribe_bonus(0x11)` (tribe's bonus id is 0x11, or bit 0x11 of
   `conquest_racial_powers` — the latter is CtW): `mongol_attrition`
   (+0x7f0), same rule.
6. Bit 6 of byte 3 of `rare.ptr` (+0x6da7) or of `rare_conquest.ptr`
   (+0x6dcf): `BitMask<44>` puts `ptr` at +0xc, so this is rare bit 30 =
   `TITANIUM` (0x24) − `BASE_RARE` (6). `titanium_attrition` (+0x95c), same
   rule. (`rare_conquest` is CtW.)

Each step is, in the listing, `movss x; mulss [0xb696ac]; cvtdq2ps
(100−p); divss; movss x` — one single-precision multiply and one divide,
each rounded (scalar SSE, `-fp:precise`). `0xb696ac` holds `0x42c80000`
(100.0; read from `.rdata` in the PE). "≥ 100" is `cmp p,100; jl` /
`99 < p` — the threshold is inclusive at 100.

Units: the leaf is a resistance scale whose 256.0 is "1×" in 8.8 terms
(the reader multiplies by 2^-8); the constants are integer percentages.
`Constants::init (00569a90)` loads `liberty_attrition` (`:1434`),
`mongol_attrition` (`:1774`), `titanium_attrition` (`:1941`) by `get_item`
(plain) and `attrition_upgrade[0..3]` (`:526`–`:549`) by `convert_int` of
`entryN`; the shipped `rules.xml` values are 25, 50, 75, (100); 100; 50; 50
— matching `crates/sim/src/core/tuning.rs:1432`–`1442`.

**When it runs** (all three callers):

- `Leader::init (006e3930)` at slot setup: zeroes the leaf (`:619`), zeroes
  `take_att_disabled` (`:628`), then `calc_attrition` and
  `calc_anti_attrition` (`:635`–`636`), so init's 0 is overwritten at once.
  Callers: `Setup::build_leader (005abc70)`, `Setup::build_game (005ac190)`,
  `Leaders::init (006ed850)`, `WalkDataGame::walk_data (005a2360)` (slot 9
  only), `MapScenario::make (0068c8f0)`, `ConquestGame::build_conquest_game
  (00797c40)`, `ScenarioEditor::new_leader (009a1490)`,
  `ScenarioRead::load_leader_chunk (009ac080)` ×6.
- `Leader::calc_unit_stats (006cf970)`, whose first two calls are
  `calc_attrition` then `calc_anti_attrition`. Reached each frame from
  `Leader::process (006b88b0)` / `Leaders::process_all (006ed2a0)` when
  `leader_flags & 0x4000000`, which is cleared there. That flag is set
  (listing, `orl $0x4000000|$0xc000000` on a leader base) by
  `Leader::gain_tech (006dcb60)` (`6dd993`, `6e0315`), `Leader::lose_tech`
  (`6d2afd`), `Leader::gather` (`6ce3d0`), `Leader::calc_gather` (`6cf5ea`,
  after the rare mask at +0x6d98 is rebuilt), `Build::activate (00623e20)`
  (`626f0b`) and `Build::close (00628980)` (`628f3f`, wonder lost). Direct
  callers also: `Leader::set_age (006d25a0)` / `set_epoch (006d26f0)`
  (reached from `ScenarioFuncSet::gain_next_age`, `lose_current_age`,
  `ConsoleWin::run_cmd`, `UnitBalance::next`) and many `ScenarioFuncSet::*`
  and `ScenarioEditor::*` functions.
- `Leader::gain_tech (006dcb60)` direct (`:428`–`433`, listing
  `6dd383`–`6dd3cf`): when the tech's type answers non-zero at vtable
  `+0x38`, its techtype `+0x14` is 1, and the decrypted
  `data_encrypted->epoch[1]` (`LeaderDataEncrypt` +0xec, xor `0x63187`)
  equals `Constants::ctw_wonder_min_civics` (+0xad4) — then `calc_pop_cap`,
  `calc_anti_attrition`, `calc_attrition`, `Region::fix_borders`. The
  slot-`+0x38` predicate was not named; the effect on this leaf is only
  timing, since the dirty flag covers the same tech a frame later.

**Exact values.** The value is a function of (tier, Liberty, Mongol bonus,
titanium) and the five constants alone; computed step by step in single
precision with the shipped constants (exact rationals, then RNE to 24
bits per operation):

| Foraging | Mongol | titanium | value | bits | exact rational | f32(exact) |
|---|---|---|---|---|---|---|
| none | – | – | 256.0 | `0x43800000` | 256 | same |
| none | one of the two | | 512.0 | `0x44000000` | 512 | same |
| none | both | | 1024.0 | `0x44800000` | 1024 | same |
| 1 (25) | – | – | 341.33334… | `0x43aaaaab` | 1024/3 | same |
| 1 | one of the two | | 682.66675 | **`0x442aaaac`** | 2048/3 | `0x442aaaab` |
| 1 | both | | 1365.3335 | **`0x44aaaaac`** | 4096/3 | `0x44aaaaab` |
| 2 (50) | – / one / both | | 512 / 1024 / 2048 | `0x44000000` / `0x44800000` / `0x45000000` | same | same |
| 3 (75) | – / one / both | | 1024 / 2048 / 4096 | `0x44800000` / `0x45000000` / `0x45800000` | same | same |
| any | Liberty held (100 ≥ 100) | | 0.0 | `0x00000000` | — | — |
| any | `take_att_disabled` ≠ 0 | | 0.0 | `0x00000000` | — | — |

The two bold rows are where the rounding chain is observable: Foraging 1
then ×100/50 rounds `341.33334·100` up to 34133.3359375 (`0x47055556`) and
`/50` lands on `0x442aaaac`, one ulp above the correctly rounded 2048/3.
A port that forms the exact rational and rounds once is wrong by one ulp
in exactly these two states. Mongol and titanium are both 50, so their
order between themselves is not observable with the shipped data;
Foraging-before-them is.

**Float class (DECISIONS 120 §3).** The arithmetic is `cvtdq2ps`, `mulss`,
`divss` on integers and two constants, with no input that is itself a
float other than the leaf's own prior step — so it never needs class 3.
On a game where no leader holds Foraging, Liberty, the Mongol bonus or
titanium (any opening frame), it is **class 1**: the constant
`0x43800000`, store `6cdcce`, which is what entry 120 §3.4 read on both
frames. In general it is **class 2**: `num::single::Single`
(`crates/sim/src/num/single.rs`) already has `from_i32` (cvtdq2ps),
`mulss` (`:232`) and `divss` (`:254`), so the leaf is
`from_bits(0x43800000)` stepped by `.mulss(from_bits(0x42c80000)).divss(from_i32(100 − p))`
per held source, with 0 for the ≥ 100 and disabled exits. Equivalently a
pinned table of the eleven distinct patterns above keyed by the four
predicates. Assumption: MXCSR is round-to-nearest-even with no FTZ/DAZ in
the game (no denormals arise here either way); not read.

### 3. What clears or resets it

Nothing decays it. Every call recomputes from 256.0. `Leader::init`
zeroes it and immediately recomputes. An input `DataWalk`
(`LeaderData::walk_data (006d6750)`) overwrites it with saved bits.
`take_att_disabled` forces 0 at the next recompute; that field's writers
are `Leader::init` (0) and `ScenarioFuncSet::enable_take_attrition
(00a02880)` / `disable_take_attrition (00a028c0)` (triggers only), and
neither of those calls a recompute itself.

### 4. Who reads it

- `UnitData::get_attrition (00608fd0)`, listing `609122`–`60914e`: for a
  non-militia unit, `cvttss2si(((float)k ·ss anti_att) ·ss 2^-8)`
  (`0xb69430` = `0x3b800000`), where `k` is 256, or
  `25600/(100 − siege_attrition)` = 512 for a siege unit. With the shipped
  data `k` is a power of two, so both multiplies are exact and the integer
  is `trunc(anti_att)` or `trunc(2·anti_att)`; on every row above it equals
  the floor of the exact rational (the one-ulp rows give 682 and 1365
  either way). Then Foraging 1 plus `is_idle` → 0.
- `LeaderData::get_anti_attrition (006e0b70)` (`movss 0x7f4(%ecx)`; no
  direct calls — inlined, and its only inlined load is the one above).
- `LeaderData::log_data (006e5110)` at `6e8947`, printed to the game log.
- The sync checksum's walk of the leaders record (the range above).

### 5. Where `crates/sim` has the event

The rule exists as an exact rational, unwired:
`crate::territory::attrition::resistance` (`crates/sim/src/territory/attrition.rs:102`)
with `ResistanceMods` (`:132`), read by the attrition period as
`scale·num/(den·256)` (`:315`). Nothing in the live sim calls
`resistance()` (only `territory/attrition/tests.rs`), so
`territory.players[w].resistance` stays `Resistance::BASE` and
`foraging_1` stays false for every player. The sibling `calc_attrition` is
wired: `Sim::calc_attrition` (`crates/sim/src/territory/upkeep.rs:24`),
called from `Sim::apply_gained` (`crates/sim/src/tech/gain.rs:601`, at
`:608`). A builder adds `calc_anti_attrition` beside it there and at the
head of `Sim::calc_unit_stats` (`crates/sim/src/economy/rares.rs:679`,
which today lacks the original's `calc_attrition`/`calc_anti_attrition`
prologue), consumed from the dirty flag in `core/frame.rs:188`, plus at
leader setup. Liberty needs a "wonder a player owns" the crate says it
does not hold (`upkeep.rs` doc comment); Mongol needs the tribe bonus;
titanium reads `economy.ledgers[w].rare` bit 30.

The checksum emits the leaf as zeros today: `crates/sim/src/checksum/leaders.rs:151`
lists it, and `crates/rondata/src/diff/checksum_lines.rs:1756` holds it out
as `anti_att (unheld)`. A zero is wrong on every frame of every game
(the floor is 256.0, `0x43800000`, from `Leader::init`).

The crate's doc comment at `attrition.rs:76`–`79` says the factor product
"is never an integer" over the shipped data; it is (256, 512, …), but the
f32 is exact there, so the conclusion — the reader's integer equals the
rational's floor — still holds. It is the leaf's **bits** that differ from
the rational, on the two rows marked above.

### 6. Reached by a plain game?

Yes. `Setup::build_game`/`build_leader` → `Leader::init` → the recompute,
on every leader at the start of a skirmish; `Leader::process`'s dirty-flag
path on every tech, building activation and rare change. All four
modifiers are reachable in a skirmish against the computer: the Foraging
techs, the Statue of Liberty, the Mongol nation, titanium. Behind other
layers: `take_att_disabled` (triggers), the `conquest_racial_powers` and
`rare_conquest` arms (CtW), the scenario-editor and `ScenarioFuncSet`
callers, and the input `DataWalk` (save-game load; multiplayer resync not
read).

### 7. Confidence, and what was not read

High on the writer set (both nets agree; listing read end to end for the
writer), the step order, the ≥ 100 rule, the constant 100.0 (PE bytes), and
the bit table (exact-rational simulation of each `mulss`/`divss`; not run
through the crate's `Single`). Medium on `has_preq(FORAGING_n)` meaning
"holds that tech": its generic tail was skimmed, not read. Not read: what
vtable slot `+0x38` in `gain_tech`'s direct branch names; whether any
non-load `DataWalk` writes on input (multiplayer join/resync); MXCSR state;
the remaining `Leader::init` callers' predicates; whether `Build::activate`'s
flag store is on every activation path.

## Summary

| leaf | writers | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `anti_att` (+0x7f4, f32) | `Leader::calc_anti_attrition (006cdcc0)` (via `Leader::calc_unit_stats (006cf970)`, `Leader::gain_tech (006dcb60)`, `Leader::init (006e3930)`); `Leader::init (006e3930)` (0, overwritten at once); `LeaderData::walk_data (006d6750)` (input walk, raw range) | recompute: 256.0, then ×100/(100−p) in single precision for Foraging tier (25/50/75), Liberty (100 → 0), Mongol (50), titanium (50); `take_att_disabled` → 0. Bits: `0x43800000` base; full table above, incl. `0x442aaaac`/`0x44aaaaac` | none decays it; init zero+recompute; save load | unwired `territory::attrition::resistance` (`attrition.rs:102`); add to `Sim::apply_gained` (`tech/gain.rs:601`) and `Sim::calc_unit_stats` (`economy/rares.rs:679`); checksum holds it as zeros | yes (init, tech, rare, building); triggers/CtW only for disable and conquest arms | high (writers, arithmetic, bits); medium (`has_preq`) |
