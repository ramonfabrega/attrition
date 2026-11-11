# Item 4152, second (blind) reading: `kills`

- Slug: `kills`
- Leaves: `units_killed` (LeaderData +0x80c, int), `units_lost` (+0x810, int),
  `buildings_razed` (+0x818, int), `buildings_lost` (+0x81c, int)
- Reader: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11
- Sources: the decompile export (`~/ghidra-projects/decomp`), the PE listing
  (`aa/exe.s`, llvm-objdump of `.text`), `crates/sim`. Not opened: the first
  reading, the README, CHECKSUM §20, PARKED, LEADERS, the 41xx journals.

