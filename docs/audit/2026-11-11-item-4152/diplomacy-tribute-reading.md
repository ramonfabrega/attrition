# Item 4152, family 8 (`diplomacy-tribute`) — first reading

- **Family**: 8, `diplomacy-tribute`.
- **Leaves** (`LeaderData` offsets, `types.txt`): `tribute_stamp` (+0x174
  int[8]), `gift_stamp` (+0x194 int[8]), `counteroffer` (+0x314 int[8]),
  `tribute_demanded` (+0x334 int[8]), `tributes` (+0x498 int[6]),
  `tribute_sent` (+0x860 int), `tribute_received` (+0x864 int).
- **Reader**: Claude Opus 5.5 (`claude-opus-5-5`), wave 1 first reading,
  2026-11-11.
- **Base**: `Leader` (0x6eec) and `LeaderData` (0x6ee4) share offset 0 — a
  `Leader` method prints `who` as `this->field_0x8` (for example
  `Leader::action_clear_tributes (006d1690)`), so a `Leader*` displacement is
  the `LeaderData` offset unchanged. The slot-`i` record is at
  `0xe3a390 + i*0x6eec`; the `Diplomacy dip[8]` array begins at +0x692c
  (`0xe40cbc - 0xe3a390`), 0x5c each.

