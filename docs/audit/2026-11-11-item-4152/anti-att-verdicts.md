# Item 4152, family 6 (`anti-att`): adjudication

- Family: 6, slug `anti-att`
- Leaf: `anti_att` (`LeaderData` +0x7f4, `float`)
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), wave 3, 2026-11-11
- Inputs: `anti-att-reading.md` (first), `anti-att-second.md` (blind second),
  `docs/DECISIONS.md` entry 120 §3, the decompile export, the `.text`
  listing (`exe.s`, job tmp), and an IEEE single replay of the writer's chain
  (numpy `float32`, job tmp; a check, never in the crate).

Appended per claim as it is settled.

## Leaf `anti_att` (+0x7f4, `float`)

### A. Layout, writers and resets

| # | claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| A1 | `LeaderData +0x7f4 float anti_att`; `Leader*` and `LeaderData*` share offsets; array form `0xe3ab84 + i·0x6eec` | yes | yes | agree | **confirmed** | `types.txt:42205`; listing 6cdce9 (`imull $0x6eec, 0x8(%esi)` + `0xe3a390`, `who` at +0x8) |
| A2 | `Leader::calc_anti_attrition (006cdcc0)` is the computing writer, 8 stores: 6cdcce (256.0), 6cdcda / 6cdd5f / 6cde66 (0), 6cdd8d / 6cdde8 / 6cde3c / 6cde8e (`movss`) | yes | yes | agree | **confirmed** | listing 6cdcc0–6cde9c, read whole |
| A3 | `Leader::init (006e3930)` stores 0 at 6e4955 (`andl` with `ecx` = 0 from 6e4949), then calls the writer at 6e49c3 | yes | yes | agree | **confirmed**, with an addition: init also zeroes `take_att_disabled` at 6e4990 **before** the recompute, so init's recompute never takes the disabled exit; and init's head is itself a `memset(this, 0, 0x6929)` (6e398c), a second zeroing of this leaf neither reader named, overwritten by the same 6e49c3 | listing 6e398c, 6e4955, 6e4990, 6e49c3 |
| A4 | `Leaders::clear (006ed5d0)` memsets each slot's `[+0, +0x6929)` → `+0.0` | yes | — | one-sided | **confirmed**: base `0xe40fa4 − 0x6c14 = 0xe3a390`, size `0x6929`. Its callers are `Game::close` (594faa) and `Game::run_oos_recovery` (586dd6): between games, not inside a plain game's frames. `Leader::close (006b7f80)` holds the same memset but has **no caller** in `.text` (its body is the one inlined into `Leaders::clear`); neither reader named it, and it writes nothing live | listing 6ed5e0–6ed5ee; 6b7f82; no `0x6b7f80` operand anywhere in `exe.s` |
| A5 | a load restores the saved bits raw: first names `LoadGame::walk_function (0043d950)`, second `LeaderData::walk_data (006d6750)` on an input walk | yes | yes | agree in substance (the walker and the walked range) | **confirmed**: `walk_data` hands the walker `[&who, &field_0x692a)` raw, which spans +0x7f4, and only when `leader_flags & 1` | `LeaderData/walk_data@006d6750.c:16-17` |
| A6 | no other store: `0xe3ab84` absolute has zero hits; the other `0x7f4(%reg)` sites are foreign (Cliffs, Constants, interface, renderer, chat; and three indexed int adds off `[0xc061ec]`) | yes | yes | agree | **confirmed**, my own nets: every `, 0x7f4(` destination (16 lines), one-operand forms (`incl` at 67cfd5 only), and wide stores (`movups`/`movq`/`movsd`/`fstpl`) at displacements `0x7e5..0x7f3` (none). The three indexed sites load `edx`/`ecx` from `0xc061ec` and test `+0x821`, which is not the leaders array | `exe.s` greps; listing 5fedf1–5fee0f, 5ff354–5ff36e, 67cfc5–67cfd5 |
| A7 | three direct callers of the writer and no indirect: `calc_unit_stats` 6cf97b, `gain_tech` 6dd3cf, `init` 6e49c3; one `0x43800000` store to +0x7f4 in `.text` (no inlined copy) | yes | yes | agree | **confirmed**. Neither `vtables.txt` nor the listing has the address as data | `grep 'calll 0x6cdcc0'` → exactly those three |
| A8 | reset: nothing decays it; every run starts again from 256.0 | yes | yes | agree | **confirmed** (6cdcce precedes every test) | listing 6cdcce |

### B. The predicate and the step

Both readers give the same order, the same constants and the same exits.
Spot-checked against the listing, predicate by predicate:

| # | claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| B1 | 256.0 is stored **before** the `take_att_disabled` test; `≠ 0` → store 0, return | yes | yes | agree | **confirmed** (`cmpl $0, 0x7fc(%esi)` at 6cdcc7 sets the flags, the store at 6cdcce leaves them, `je` at 6cdcd8) | 6cdcc7–6cdcda |
| B2 | the Foraging tier is the **highest** of `has_preq(0x300)`, `has_preq(0x2ff)`, `has_preq(0x2fe)`, indexing `attrition_upgrade[2]`, `[1]`, `[0]`; not cumulative | yes | yes | agree | **confirmed** (`leal -0x62(%edi)` = 2 at 6cdd0a, `movl $1` at 6cdd2a, `xorl` at 6cdd4c; one load at 6cdd53) | 6cdcf1–6cdd53 |
| B3 | `has_preq(FORAGING_n)` means "holds that tech" | assumed, unread | assumed, medium | agree (both flagged) | **corrected**: FORAGING_1..3 are **bonus types** (`0x2fe..0x300`, inside `is_bonus_type`'s `0x2ac..0x325`), not techs, and `has_preq` answers "this type's prerequisites are met". For these three none of `has_preq`'s nation arms, the finals, the age quota or the government tail applies, so it reduces to `has_tech(preq0)`. The shipped `TECHBONUSES` rows 82, 83, 84 give `preq0` = **Forage**, **Supply**, **Logistics**. So the tier is the highest of Logistics / Supply / Forage held. The crate already reads this exact shape: `bonus_at(n)` in `crates/rondata/src/load.rs` (e.g. `:1932` for `ATTRITION1..4`, whose rows 49–52 read Allegiance … Nationalism, the same index base) and `Sim::bonus_level` (`crates/sim/src/economy/rares.rs:84`) | `LeaderData/has_preq@006db810.c:89-140, 199-226`; `game/data/rules.xml` `TECHBONUSES` records 82–84 (read by position in the job's scratch) |
| B4 | the threshold is inclusive: `pct ≥ 100` → store 0 and return at once; later sources unread | yes | yes | agree | **confirmed**; the compare is **signed** (`cmpl %edi,%ecx; jl` at 6cdd5a, `jge` at 6cddbf / 6cde1f, `cmpl $0x64; jl` at 6cde5f), so a negative pct would pass and divide by more than 100. Shipped values are all ≥ 0 | 6cdd5a, 6cddbf, 6cde1f, 6cde5f |
| B5 | Statue of Liberty: `has_wonder(0x219)`, pct `liberty_attrition` (`Constants +0x524`) | yes | yes | agree | **confirmed**. `has_wonder` answers non-zero for a CtW `conquest_wonders` bit once the Civic level reaches `ctw_wonder_min_civics`, or for an entry of **this leader's** wonder list whose live flag (`+0xc & 1`) is set, whose object's type matches, and (for any type but `0x216`) whose object's city short `+0x72` is ≥ 0 | 6cdd9d–6cddbf; `LeaderData/has_wonder@006ebc10.c:30-48` |
| B6 | Mongol: `has_tribe_bonus(0x11)`, pct `mongol_attrition` (`+0x7f0`) | yes, with the arms | yes, without the "no powers"/nomad arms | first fuller | **confirmed as the first states it**: 0 under `info.flags & 4`, or no starting town **and** `city_num == 0`, or `tribe < 0`; then 1 on a CtW `conquest_racial_powers` bit; then 0 if `leader_flags2 & 0x40`; else the tribe's bonus index (`tribe·0x5f0 + 0x54`) must equal 17. The crate's `TechTree::has_tribe_bonus` (`crates/sim/src/tech.rs:1102`) holds the first two arms and the index compare; it has no `leader_flags2 & 0x40` arm (unread what sets it) | `LeaderData/has_tribe_bonus@006e1370.c:8-20` |
| B7 | titanium: bit 30 of `rare.ptr` (+0x6da7 & 0x40) **or** of `rare_conquest.ptr` (+0x6dcf & 0x40), an inlined `has_rare(TITANIUM = 0x24)`, pct `titanium_attrition` (`+0x95c`) | yes | yes | agree | **confirmed**: `BitMask<44>` puts `ptr` at +0xc (`types.txt:42413`), so `rare.ptr` byte 3 is +0x6da7 and bit 6 of it is bit 30; `has_rare` indexes `id − 6` (`BASE_RARE`), and `TypeIndex` 36 = `TITANIUM`. Both tests read `leaders.list[who]` cached at 6cde0b | 6cde44–6cde62; `LeaderData/has_rare@006e0770.c:9-13`; `enums/TypeIndex.txt:11, 50` |
| B8 | the step is `x := divss(mulss(x, 100.0f), cvtdq2ps(100 − pct))`, scalar SSE single, the constant at `0xb696ac` = `0x42c80000` | yes | yes | agree | **confirmed**; I read `0xb696ac` from `.rdata` myself: `42c80000` = 100.0 | 6cdd6e–6cdd8d (and three twins); PE `.rdata` at VA `0xb696ac` |
| B9 | MXCSR is round-to-nearest-even, no FTZ/DAZ | — | assumed, unread | one-sided | **confirmed**: `.text` holds no `ldmxcsr` and no `stmxcsr`. The one FP-control call is `_initialize_default_precision (0055dd7c)` → `_controlfp_s(NULL, 0x10000, 0x30000)`, which sets the **x87** precision control (`_PC_53` under `_MCW_PC`) and leaves SSE alone. No denormal arises on any reachable value, so FTZ/DAZ would not matter either | `exe.s` greps; listing 55dd7c–55dd88; IAT slot `0xac556c` = `_controlfp_s` |
| B10 | the order Foraging → Liberty → Mongol → titanium is observable in the bits | yes | yes (Foraging before the other two; Mongol against titanium not, both 50) | agree | **confirmed** by replay: Mongol before Foraging 1 gives `0x442aaaab`, the writer's order gives `0x442aaaac`. So the crate's doc sentence "With exact rationals it is not [observable]" (`crates/sim/src/territory/attrition.rs:100-101`) holds for the rational and its integer, never for the bits | replay (§D) |
| B11 | the predicates read `leaders.list[this->who]`, the stores go to `this` | yes | implied | one-sided | **confirmed**; the same record in every caller (each passes its own leader) | 6cdce9, 6cdd0f, 6cdd31, 6cdd95, 6cddfa vs `0x7f4(%esi)` |

### C. When it runs, and the readers

| # | claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| C1 | `calc_unit_stats (006cf970)` opens with `calc_attrition` then `calc_anti_attrition`; it runs from the per-frame fold in `Leaders::process_all` / `Leader::process` when `leader_flags & 0x4000000`, which the fold clears | yes | yes | agree | **confirmed** | 6cf974, 6cf97b |
| C2 | the `0x4000000` setters: `gain_tech` (6dd993, 6e0315), `lose_tech` (6d2afd), `gather` (6ce3d0), `calc_gather` (6cf5ea), `Build::activate` (626f0b), `Build::close` (628f3f) | by decompile grep | by listing | agree | **confirmed**: my `orl $0x4000000/$0xc000000, (%reg)` net finds exactly these seven, plus `isValidComponent` (a33bf2), which is not a leader. `Leader::gain_tech`'s two are its tail and its unit-type arm | `exe.s` `orl` net; function bounds from `INDEX.tsv` |
| C3 | `Build::activate` raises the bit **for a wonder only** (types `0x20e..0x21e`); every other building jumps past to 626f11 | yes | "building activation" (§6), unqualified | disagree | **first confirmed, second corrected**: 625ae2–625af4 compare the type to `0x20e`/`0x21f` and send anything outside to 626f11, the instruction after the `orl` at 626f0b | listing 625adf–625aff, 626f0b–626f11 |
| C4 | `gain_tech`'s direct call (6dd3cf) fires on a type that answers vtable `+0x38`, with techtype `+0x14 == 1`, when the decrypted Civic level equals `ctw_wonder_min_civics`; "redundant in a plain game" (first) / "only timing" (second) | yes | yes | agree, slot unnamed by both | **confirmed and named**: slot `+0x38` is `TypeData::is_epoch_type` (`vtables.txt:29606`). `CTW_WONDER_MIN_CIVICS` ships as **4** (`rules.xml:599`), so the arm is **reached in a plain game** at the epoch that takes the Civic line to 4, where it runs `calc_pop_cap`, `calc_anti_attrition`, `calc_attrition`, `Region::fix_borders` at once. The value it stores equals the next fold's (a pure recompute on unchanged inputs), so it moves the leaf early only when a source arrived after this frame's fold | listing 6dd383–6dd3cf; `Leader/gain_tech@006dcb60.c:427-436` |
| C5 | **a builder recomputes `anti_att` in `Sim::apply_gained`, beside `calc_attrition` (`crates/sim/src/tech/gain.rs:608`)** | yes | yes | agree | **corrected**. The original calls `calc_anti_attrition` from `gain_tech` **only** in C4's Civic-4 arm. Every other tech reaches it through the dirty bit, at the gainer's **next** `Leader::process` fold. Research finishes inside `Objects::process_all`, which runs after `Leaders::process_all` (`crates/sim/src/core/frame.rs:181-186`'s own comment), so Forage finishing on frame F leaves the old bits in F's leaders line and the new bits from F+1. An eager recompute in `apply_gained` is one frame early on every such frame. The same holds for `calc_attrition`, whose only `gain_tech` call is 6dd3d6 in the same arm; the crate's eager call at `gain.rs:608` is the attrition-stamps family's to settle, flagged here | callers of 0x6cdcc0: 6cf97b, 6dd3cf, 6e49c3 only; callers of 0x6cdea0: 6cf974, 6dd3d6, 6e49bc only |
| C6 | `set_age (006d25a0)` / `set_epoch (006d26f0)` and the `ScenarioEditor::*` / `ScenarioFuncSet::*` functions call `calc_unit_stats` directly | yes | yes | agree | **confirmed as a list** (spot-read, not each opened); none in a plain game's frame loop except as the lobby's starting age might reach `set_age`, unread by both | — |
| C7 | staleness: `take_att_disabled` toggled by `ScenarioFuncSet::enable_take_attrition (00a02880)` / `disable_take_attrition (00a028c0)` with no recompute; a nomad Mongol's first city flips `has_tribe_bonus` with no dirty bit of its own | yes | yes (the first half) | agree / one-sided | **confirmed** for the trigger pair (both store to `0xe3ab8c` and call nothing). The nomad half is **open**: whether the founding of a first city raises `0x4000000` by another path was not read by either; settled by grepping the city founder (`City::init` / `Build::activate` for a city centre) for a leader-flags write | — |
| C8 | reader `UnitData::get_attrition (00608fd0)`: non-militia `cvttss2si(mulss(mulss(cvtdq2ps(k), anti_att), 2^-8))`, k = 256 or 512 for siege; militia bypass | yes | yes | agree | **confirmed**: 60912c–60914e; `0xb69430` = `0x3b800000` read from `.rdata` (1/256). The leaf read is **the unit's owner's** (`movzbl 0x9(%esi)`, base `[0xc061e0]`). k and 2^-8 are powers of two, so the product is exact and the integer is `trunc(anti_att·k/256)` | 609122–60914e |
| C9 | the Foraging-1 idle immunity after the read | — | yes ("Foraging 1 plus `is_idle` → 0") | one-sided | **confirmed, and sharpened**: the test is `has_preq(0x2fe)` on the same leader (609133, 609152), i.e. "holds **Forage**" (B3), and Forage is under Supply and Logistics, so it holds at tiers 2 and 3 too, not at "tier exactly 1". The crate's `PlayerState.foraging_1` (`crates/sim/src/territory/attrition.rs:261`) must be set from `has_preq(FORAGING_1)`, not from the tier | 609133–60916e |
| C10 | `LeaderData::get_anti_attrition (006e0b70)` is `movss 0x7f4(%ecx)` with no direct caller; `log_data` prints it at 6e8947 | yes | yes | agree | **confirmed** (no `0x6e0b70` operand in `exe.s`) | `exe.s` grep |

### D. The bits, and the float class

**Replay.** I replayed the writer's chain for all 32 combinations of
(no tier / tier 1 / 2 / 3) × Liberty × Mongol × titanium under the shipped
constants (`attrition_upgrade = 25, 50, 75`, Liberty 100, Mongol 50, titanium
50; `crates/sim/src/core/tuning.rs:1432, 1440-1442`). I did it twice,
independently. Once in Python, as an f64 product or quotient of two singles
rounded to single: the product is exact in f64, and f64's 53 bits are at least
2·24 + 2, so the quotient's double rounding is harmless. And once with exact
rationals, rounding each `mulss` and `divss` to the nearest single, ties to
even. The two agree on every state:

| tier | Liberty | Mongol, titanium | writer's bits | nearest single to the exact rational | reader's integer (k = 256 / 512) |
|---|---|---|---|---|---|
| — | no | neither / one / both | `0x43800000` / `0x44000000` / `0x44800000` | same | 256/512, 512/1024, 1024/2048 |
| 1 | no | neither | `0x43aaaaab` | same | 341 / 682 |
| 1 | no | one | **`0x442aaaac`** | `0x442aaaab` | 682 / 1365 |
| 1 | no | both | **`0x44aaaaac`** | `0x44aaaaab` | 1365 / 2730 |
| 2 | no | neither / one / both | `0x44000000` / `0x44800000` / `0x45000000` | same | exact |
| 3 | no | neither / one / both | `0x44800000` / `0x45000000` / `0x45800000` | same | exact |
| any | yes | any | `0x00000000` | — | 0 |
| any (disabled) | — | — | `0x00000000` | — | 0 |

The intermediates on the bold rows: `0x43800000 ×100 → 0x46c80000, /75 →
0x43aaaaab; ×100 → 0x47055556 (34133.3359375), /50 → 0x442aaaac; ×100 →
0x47855556, /50 → 0x44aaaaac`. **Both readers' one-ulp claims are
confirmed**, and so is their claim that every reader integer equals the
floor of the exact rational (all 32 states, both k). Distinct patterns
over the shipped data: **nine** (`0`, `0x43800000`, `0x43aaaaab`,
`0x44000000`, `0x442aaaac`, `0x44800000`, `0x44aaaaac`, `0x45000000`,
`0x45800000`). The second's "eleven distinct patterns" counts table rows;
**corrected** to nine.

**Class (entry 120 §3).** The family table's "class 3" (`README.md` row 6)
is **corrected**. Both readers' class 1 / class 2 is **confirmed**:

- **Class 1** on a frame where no walked leader holds Forage (or above),
  the Statue, the Mongol bonus or titanium: the constant `0x43800000`, the
  store at 6cdcce. That is entry 120 §3.4's reading on run 1562 frame 91
  and run1100 frame 5408.
- **Class 2** in general. Every input is an integer the simulation holds
  (four predicates, four `Tuning` percentages), and the arithmetic is
  `cvtdq2ps`, `mulss`, `divss` with one `.rdata` constant, all of which
  `num::single::Single` already has: `from_bits` (`single.rs:54`),
  `from_i32` (`:155`), `mulss` (`:232`) and `divss` (`:254`), each
  correctly rounded to nearest even. B9 settles MXCSR. Nothing joins
  `no_float.rs`'s allowances.
- A pinned table of the nine patterns would serve the shipped data only.
  The chain serves a modded `rules.xml` too, so it is the port, and the
  table above is its test.

### E. The crate site

| # | claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|---|
| E1 | the rule exists unwired as an exact rational, `territory::attrition::resistance` with `ResistanceMods` | `:102, :132` | `:102, :132` | agree | **confirmed**: its only callers are in `territory/attrition/tests.rs`; `PlayerState.resistance` defaults to `Resistance::BASE` (`:268`) | `crates/sim/src/territory/attrition.rs:102, 132, 248, 268` |
| E2 | the home is `Sim::calc_unit_stats` (`economy/rares.rs:679`), run by the fold at `core/frame.rs:188` on `roster.unit_stats_dirty`; its module doc names `calc_attrition` / `calc_anti_attrition` as calls it does not make | yes | yes | agree | **confirmed** (`rares.rs:47-48`, `:679`; `frame.rs:186-189`) | — |
| E3 | also add it in `Sim::apply_gained` (`tech/gain.rs:601/608`) | yes | yes | agree | **corrected** (C5): only C4's Civic-4 arm, not every gain | — |
| E4 | the checksum leaf is `("anti_att", 0x7f4, 4, 1)` at `checksum/leaders.rs:151` with no value arm; held out as `anti_att (unheld)` | yes | yes (+ `checksum_lines.rs:1756`) | agree | **confirmed**; and the second's "emitted as zeros" holds: `Leaf::bytes` writes 0 for an unheld leaf (`checksum/leaders.rs:401-409`). That is wrong for every walked leader not under Liberty or the disabled flag | `crates/rondata/src/diff/checksum_lines.rs:1756` |
| E5 | the crate's doc "the product … is never an integer" is false (256, 512, … are integers); the conclusion that the integer matches still holds | — | yes | one-sided | **confirmed** | `crates/sim/src/territory/attrition.rs:76-79` |
| E6 | Liberty needs a "wonder a player owns" the crate says it does not hold | — | yes (citing the `upkeep.rs` doc) | one-sided | **partly corrected**: the crate has `PlayerTech::has_wonder` (`tech.rs:814`, a membership test over `wonders`) and `Sim::wonders_held` (`economy/holdings.rs:572`), and `player_borders` already reads Tikal, the Colosseum and the Eiffel Tower through `has_wonder` (item 2106, `territory/upkeep.rs` doc). The `calc_attrition` doc saying the crate "holds no wonder a player owns" is stale on its face. Whether `wonders` carries `has_wonder`'s `+0x72 ≥ 0` (captured-city) predicate is **open** for the builder | `crates/sim/src/territory/upkeep.rs` (`player_borders` doc) |

### F. Reached by a plain game?

Both readers say yes, with the same exclusions. **Confirmed**, with two
additions:

- **A Mongol leader starts at `0x44000000`, not `0x43800000`.**
  `Leader::init`'s recompute (6e49c3) runs after the tribe is known: the
  body calls `has_tribe_bonus` at `Leader/init@006e3930.c:539-574`, before
  `:636`. A skirmish with starting towns passes `has_tribe_bonus`'s nomad
  arm while `city_num` is 0. The plain-game init value is therefore 256.0,
  or 512.0 for the Mongols unless "No Nation Powers" is set or the start
  is nomadic. Medium: the caller's store of `tribe` was not traced. A
  builder can check it against run 1562's nations; entry 120 §3.4's 256.0
  on every walked leader needs none of them to be Mongol.
- C4's arm is plain-game too (Civic 4).

Not in a plain game: `take_att_disabled` (triggers), `rare_conquest`,
`conquest_wonders`, `conquest_racial_powers` (CtW), the editor and
`ScenarioFuncSet` callers, `Leaders::clear` (between games), and the load
walk (save track).

