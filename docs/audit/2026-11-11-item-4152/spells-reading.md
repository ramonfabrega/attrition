# Item 4152, family 17 (`spells`): first reading

- Family: 17, slug `spells`.
- Leaf: `tech.ptr` — `LeaderData::tech` (+0x6c0c, `BitMask<806>`), the bits
  for type indices `0x275..0x2ab` that are set in `tech` and absent from
  `tech_at_start`.
- Reader: Claude Opus 5.5 (first reading, wave 1), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the listing
  (`llvm-objdump`), and a grep of `crates/sim`. Nothing run.

(In progress; sections are appended as they are finished.)
