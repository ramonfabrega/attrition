# Item 4152, family 15 (`odd-counters`): adjudication

- Slug: `odd-counters`. Leaves (all `int`, `LeaderData`): `repair_stamp`
  (+0x208), `missiles_used` (+0x7c0), `units_bribed` (+0x8f4),
  `nukes_launched` (+0xa38).
- Readings: `odd-counters-reading.md` (first), `odd-counters-second.md`
  (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text`
  listing (`exe.s` in the job scratch), `crates/sim` grep. Nothing run of
  the original. Leaves are appended as each is settled.

