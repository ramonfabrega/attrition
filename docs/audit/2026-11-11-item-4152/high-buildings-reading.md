# Family 20 — `high-buildings`: first reading

- Item 4152, wave 1 (first reading). Reader: Claude Opus 5.5, 2026-11-11.
- Leaves (offsets are `LeaderData`'s, `types.txt` line 42088 ff.):
  `territory_high` (+0x9dc, int), `reg_terr` (+0x145e, ushort[64]),
  `high_buildings` (+0x5660, ushort[129]).
- Layout fact used throughout: `leaders` is at 0x00e3a390 (`rise_z.map`,
  `?leaders@@3VLeaders@@A`), `Leaders::list` is `Leader[10]` at +0, and
  `Leader` (0x6eec) carries `LeaderData` (0x6ee4) at its offset 0 — the 8
  extra bytes are at the tail. So a `Leader*`'s `field_0xNNN` is
  `LeaderData + 0xNNN` with no shift, and slot `i`'s leaf is at
  `0xe3a390 + i·0x6eec + off`: `territory_high` 0xe3ad6c, `reg_terr`
  0xe3b7ee, `high_buildings` 0xe3f9f0 for slot 0.

