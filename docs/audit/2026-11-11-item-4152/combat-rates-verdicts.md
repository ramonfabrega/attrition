# Item 4152, adjudication: `combat-rates`

- Slug: `combat-rates`
- Leaves (`LeaderData`): `frame_battle` (+0xa4c, int); ushort
  `average_death_rate` (+0xa50), `average_kill_rate` (+0xa52),
  `average_damage_rate` (+0xa54), `average_hit_rate` (+0xa56),
  `deaths_current_frame` (+0xa58), `kills_current_frame` (+0xa5a),
  `hits_current_frame` (+0xa5c), `damage_current_frame` (+0xa5e),
  `deaths_fifteen_seconds` (+0xa60), `kills_fifteen_seconds` (+0xa62),
  `hits_fifteen_seconds` (+0xa64), `damage_fifteen_seconds` (+0xa66).
- Adjudicator: Claude Opus 5.5 (`claude-opus-5-5`), 2026-11-11.
- Inputs: `combat-rates-reading.md` (first), `combat-rates-second.md` (blind
  second), the decompile export (`~/ghidra-projects/decomp`), the `.text`
  listing (`exe.s`, llvm-objdump), `crates/sim`.
- Written per claim as settled; a section without its closing rule was not
  finished. The readings share their writers (W1..W4, the fold), so the
  shared claims are settled once in section 0 and each leaf cites them.

