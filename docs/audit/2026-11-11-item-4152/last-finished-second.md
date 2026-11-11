# Item 4152, second reading: `last-finished`

- Slug: `last-finished`
- Leaves: `last_building_finished` (`LeaderData` +0x6070, `int[129]`),
  `last_unit_finished` (`LeaderData` +0x6274, `int[352]`)
- Reader: Claude Opus 5.5 (`claude-opus-5-5`), blind second reading, 2026-11-11
- Sources: the decompile export (`~/ghidra-projects/decomp`), `types.txt`,
  the `.text` listing of `riseofnations.exe`, and `crates/sim`.

