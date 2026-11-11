# Item 4152, family 7 — `diplomacy-relations`: first reading

- **Family**: 7, `diplomacy-relations`.
- **Leaves** (`LeaderData` offsets; `int[8]` unless named): `good_deeds`
  (+0xd4), `blacken` (+0x20c, `int`), `dow` (+0x270), `broke_alliance`
  (+0x2b0), `made_peace` (+0x2d0), `last_spoke` (+0x2f4), `hire_stamp`
  (+0x1b4), `hire_who` (+0x1d4).
- **Reader**: Claude Opus 5.5 (`claude-opus-5-5`), first reading, 2026-11-11.
- **Sources**: the decompile export (`~/ghidra-projects/decomp`), the listing
  of `riseofnations.exe` (`llvm-objdump`), `types.txt`. `Leader` is
  `0x6eec`, `LeaderData` `0x6ee4`.

