# Item 4152, family 10 (`end-of-game`): adjudication

- Slug: `end-of-game`. Leaves (`LeaderData`): `defeated_by` (+0x10),
  `wonderwin_stamp` (+0x448), `wonderwin_timer` (+0x44c), `victory_type`
  (+0x7d8), `defeat_type` (+0x7dc), `defeat_stamp` (+0x6924); all int.
- Readings: `end-of-game-reading.md` (first), `end-of-game-second.md`
  (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text`
  listing (`exe.s` in the job scratch), `crates/sim` grep. Nothing run of
  the original.

(Written per leaf as settled.)
