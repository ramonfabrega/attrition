# Item 4152, family 3 (`combat-rates`): first reading

- **Family**: 3, slug `combat-rates`.
- **Leaves** (`LeaderData` offsets; `Leader` has `LeaderData` at +0, so a
  `Leader*` `field_0xNNN` is the same offset): `frame_battle` (+0xa4c, int);
  ushort `average_death_rate` (+0xa50), `average_kill_rate` (+0xa52),
  `average_damage_rate` (+0xa54), `average_hit_rate` (+0xa56),
  `deaths_current_frame` (+0xa58), `kills_current_frame` (+0xa5a),
  `hits_current_frame` (+0xa5c), `damage_current_frame` (+0xa5e),
  `deaths_fifteen_seconds` (+0xa60), `kills_fifteen_seconds` (+0xa62),
  `hits_fifteen_seconds` (+0xa64), `damage_fifteen_seconds` (+0xa66).
- **Reader**: Claude Opus 5.5 (`claude-opus-5-5`), 2026-11-11.
- **Sources**: the decompile export under `~/ghidra-projects/decomp`
  (`funcs/`, `types.txt`, `INDEX.tsv`), and the listing of
  `game/riseofnations.exe` by `llvm-objdump`. Slot `i`'s `LeaderData` is at
  `0xe3a390 + i*0x6eec`.

