# combat-rates: second (blind) reading

- Slug: `combat-rates`
- Item: 4152, wave 2
- Reader: Claude Opus 5.5 (claude-opus-5-5), blind to the first reading
- Leaves (`LeaderData`, size `0x6ee4`):
  `frame_battle` (+0xa4c, int);
  ushort: `average_death_rate` (+0xa50), `average_kill_rate` (+0xa52),
  `average_damage_rate` (+0xa54), `average_hit_rate` (+0xa56),
  `deaths_current_frame` (+0xa58), `kills_current_frame` (+0xa5a),
  `hits_current_frame` (+0xa5c), `damage_current_frame` (+0xa5e),
  `deaths_fifteen_seconds` (+0xa60), `kills_fifteen_seconds` (+0xa62),
  `hits_fifteen_seconds` (+0xa64), `damage_fifteen_seconds` (+0xa66).

Sources: the decompile export at `~/ghidra-projects/decomp`, the listing at
`tmp/r4152/aa/exe.s` (llvm-objdump of `.text`), `types.txt`.

