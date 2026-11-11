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

