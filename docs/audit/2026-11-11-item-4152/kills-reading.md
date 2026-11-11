# Item 4152, family 2 (`kills`) — first reading

- Leaves: `units_killed` (LeaderData +0x80c), `units_lost` (+0x810),
  `buildings_razed` (+0x818), `buildings_lost` (+0x81c), all `int`.
- Reader: Claude Opus 5.5 (claude-opus-5-5), wave 1 first reading, 2026-11-11.
- Sources: the decompile export `~/ghidra-projects/decomp` and the listing of
  `riseofnations.exe` (`llvm-objdump`). Written per leaf as each finishes.

