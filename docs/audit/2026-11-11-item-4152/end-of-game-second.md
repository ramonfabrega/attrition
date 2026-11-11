# end-of-game: second (blind) reading

- Slug: `end-of-game`
- Leaves (`LeaderData`, size `0x6ee4`): `defeated_by` (+0x10, int),
  `wonderwin_stamp` (+0x448, int), `wonderwin_timer` (+0x44c, int),
  `victory_type` (+0x7d8, int), `defeat_type` (+0x7dc, int),
  `defeat_stamp` (+0x6924, int).
- Reader: Claude Opus 5.5, blind (has not opened the first reading).
- Sources: decompile export `~/ghidra-projects/decomp`, the full `.text`
  listing (llvm-objdump), `types.txt`.

