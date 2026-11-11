# Item 4152, family 18 (`score`) — first reading

- Family: 18, slug `score`.
- Leaves (all `int`, `LeaderData` offsets): `score` (+0x18), `score_explored` (+0x1c),
  `score_territory` (+0x20), `score_units` (+0x24), `score_units_2` (+0x28),
  `score_buildings` (+0x2c), `score_economy` (+0x30), `score_pop` (+0x34),
  `score_unit_upgrades` (+0x38), `score_research` (+0x3c), `score_wonders` (+0x40),
  `score_combat` (+0x44).
- Reader: Opus 5.5 (claude-opus-5-5), first reading, 2026-11-11.
- Sources: the decompile export (`~/ghidra-projects/decomp`), the listing
  (`llvm-objdump` of `riseofnations.exe`), `types.txt`.

