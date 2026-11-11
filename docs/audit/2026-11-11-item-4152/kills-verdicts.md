# Item 4152, family 2 (`kills`): adjudication

- Slug: `kills`. Leaves (`LeaderData`): `units_killed` (+0x80c, int),
  `units_lost` (+0x810, int), `buildings_razed` (+0x818, int),
  `buildings_lost` (+0x81c, int).
- Readings: `kills-reading.md` (first), `kills-second.md` (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text`
  listing (`exe.s` in the job scratch), `crates/sim` grep. Nothing run of
  the original.

(Written per leaf as settled.)
