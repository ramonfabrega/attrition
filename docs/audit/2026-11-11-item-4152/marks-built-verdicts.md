# Item 4152, family 13 (`marks-built`): adjudication

- Slug: `marks-built`. Leaves (`LeaderData`, all `int`): `fort_mark`
  (+0x428), `oil_well_mark` (+0x430), `supply_mark` (+0x434),
  `wonders_built` (+0x854), `wonders_held` (+0x858), `senates_built`
  (+0x85c), `forts_built` (+0x8f0).
- Readings: `marks-built-reading.md` (first), `marks-built-second.md`
  (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text`
  listing (`exe.s` in the job scratch), `crates/sim` grep. Nothing run of
  the original. The spawn line names one question to settle above the
  rest: whether a captured wonder credits the captor `wonders_built += 1`.

