# Item 4152, second reading: `last-finished`

- Slug: `last-finished`
- Leaves: `last_building_finished` (`LeaderData` +0x6070, `int[129]`),
  `last_unit_finished` (`LeaderData` +0x6274, `int[352]`)
- Reader: Claude Opus 5.5 (`claude-opus-5-5`), blind second reading, 2026-11-11
- Sources: the decompile export (`~/ghidra-projects/decomp`), `types.txt`,
  the `.text` listing of `riseofnations.exe`, and `crates/sim`.

## Common ground: layout, base, init

- `types.txt:42354–42355`: `+0x6070 int[129] last_building_finished`,
  `+0x6274 int[352] last_unit_finished`. 129 = the building `TypeIndex`
  range `0x19e..=0x21e`; 352 = the unit range `0x32..=0x19d`. The two
  arrays are back to back (`0x6070 + 129*4 = 0x6274`, `0x6274 + 352*4 =
  0x67f4 = ages_queued`).
- `Leader` (`types.txt:42086`, size `0x6eec`) lists no members of its own;
  `Leader::init (006e3930)` addresses `num_units` as `field_0x5762` and
  `high_buildings` as `field_0x5660`, the `LeaderData` offsets, so
  `LeaderData` sits at `Leader +0` and a `Leader*` displacement equals the
  `LeaderData` offset (the extra 8 bytes are at the end; `Leader::Leader
  (006d2bf0)` puts a vbtable pointer at `+0x6ee4`).
- **Both writers index with the type bias folded into the base.** The
  global array is `0xe3a390 + who*0x6eec`, written as `who*0x1bbb +
  TypeIndex` scaled by 4:
  - buildings: `0x6070 − 0x19e*4 = 0x59f8`, absolute `0xe3fd88`;
  - units: `0x6274 − 0x32*4 = 0x61ac`, absolute `0xe4053c`.
  So index `i` of `last_building_finished` is building `TypeIndex 0x19e+i`,
  and index `i` of `last_unit_finished` is unit `TypeIndex 0x32+i` — the
  same scheme as `num_buildings`/`num_units` (`0x5762 − 0x32*2 = 0x56fe`).
- **Init.** `Leader::init (006e3930)`, decompile lines 744–753, listing
  `6e4bef` (`leal 0x6070(%ebx),%edi`) and `6e4bfc` (`leal 0x6274(%ebx),%edi`),
  each followed by a `rep stosd` of `0xffffffff`: counts `0x81` and `0x160`.
  Every element is **−1**. This is on the seated path only (`param_2 >= 0`).
  An unseated slot (`param_2 < 0`, lines 70–105 and the early `return 0`
  at 323) takes `memset(this, 0, 0x6929)` instead (listing `6e398c`
  `pushl $0x6929; xor eax; push ebx; call memset`), so there both arrays
  are **0** — but `LeaderData::walk_data` only walks past the two flag
  words when `leader_flags & 1`, so an unseated slot's zeros are never
  checksummed.
- **Search nets run** (both leaves together):
  - export by name: `grep -rn -- 'last_building_finished\b\|last_unit_finished\b'
    funcs/` — no hits (the decompiler never names either field);
  - export by offset: every `field_0xNNNN` in `[0x59f8,0x5a22)` ∪
    `[0x6070,0x67f4)` outside `Conquest*`/`Terrain*` classes, and every
    leader-context line with a raw `0xNNNN` in that range: four hits —
    `Leader::init` ×2, `Build::activate:155`, `Build::train:81` — plus the
    reader `ScenarioFuncSet::get_last_type_built:69–80`;
  - listing, absolute: every immediate in `[0xe3fd88, 0xe40b84)`: only
    `0xe3fd88` (×3: `623f47` store, `9e99fb` load, `9e9a02` store −1),
    `0xe4053c` (×3: `62fa35` store, `9e9a21` load, `9e9a28` store −1), and
    `0xe3fdb2` (×8, which is `num_queued` at `+0x5a22`, not these leaves);
  - listing, register-relative: every displacement in `[0x59f8,0x67f4)`.
    No scaled-index (`(,%reg,4)`) access off a register at all; the
    unscaled ones are `Leader::init`'s two `lea`s and otherwise
    `ConquestWin`/`ConquestGame`/`ConquestNode`/`TerrainData`/`Game::run_solo`/
    `LoadGame`/`Popup` fields of their own structs (checked by `INDEX.tsv`
    owner of each site); `Achieve::record_data`'s hit is `-0x6238(%edi)`
    off a mid-record anchor, i.e. `+0x9d8`, not these leaves;
  - whole-record copies: no `rep movs` of `0x1bb9`/`0x1bbb` dwords and no
    push of `0x6ee4`/`0x6eec`/`0x37760` as a size; the only bulk access is
    `LeaderData::walk_data (006d6750)`'s raw range `[&who, +0x692a)`, which
    contains both arrays.

