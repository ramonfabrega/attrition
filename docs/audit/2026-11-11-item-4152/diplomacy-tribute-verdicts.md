# Item 4152, family 8 (`diplomacy-tribute`): adjudication

- Slug: `diplomacy-tribute`. Leaves: `tribute_stamp` (+0x174 int[8]),
  `gift_stamp` (+0x194 int[8]), `counteroffer` (+0x314 int[8]),
  `tribute_demanded` (+0x334 int[8]), `tributes` (+0x498 int[6]),
  `tribute_sent` (+0x860), `tribute_received` (+0x864).
- Readings: `diplomacy-tribute-reading.md` (first), `diplomacy-tribute-second.md` (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text` listing
  (`exe.s` in the job scratch), `crates/sim` grep. Nothing run of the original.
- Spawn questions: (A) does an all-computer skirmish at the default team
  style reach any writer; (B) `tributes` not reduced for refunded `dows`;
  (C) the zeroed `dows` slot being `P.dip[P]`. Settled in the shared section
  below, then leaf by leaf (appended as settled).

