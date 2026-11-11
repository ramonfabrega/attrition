# Item 4152, family 10 `end-of-game` — first reading

- Family: 10, `end-of-game`.
- Leaves (`LeaderData` offsets): `defeated_by` (+0x10), `wonderwin_stamp`
  (+0x448), `wonderwin_timer` (+0x44c), `victory_type` (+0x7d8),
  `defeat_type` (+0x7dc), `defeat_stamp` (+0x6924); all int.
- Reader: Claude Opus 5.5 (`claude-opus-5-5`), first reading, wave 1,
  2026-11-11.
- Sources: the decompile export (`~/ghidra-projects/decomp`), the listing
  (`llvm-objdump` of `riseofnations.exe`), `crates/sim/src`.

