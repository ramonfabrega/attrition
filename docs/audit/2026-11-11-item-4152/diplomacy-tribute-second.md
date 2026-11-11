# diplomacy-tribute — blind second reading (item 4152, wave 2)

- **Slug:** `diplomacy-tribute`
- **Leaves (`LeaderData`, size `0x6ee4`):** `tribute_stamp` (+0x174, int[8]),
  `gift_stamp` (+0x194, int[8]), `counteroffer` (+0x314, int[8]),
  `tribute_demanded` (+0x334, int[8]), `tributes` (+0x498, int[6]),
  `tribute_sent` (+0x860, int), `tribute_received` (+0x864, int).
- **Reader model:** Claude Opus 5.5 (`claude-opus-5-5`).
- **Sources:** the decompile export `~/ghidra-projects/decomp/` and the
  `.text` listing (`llvm-objdump`, pre-dumped). Written blind: no reading of
  the first write-up, CHECKSUM §20, PARKED, LEADERS or the item's journals.

