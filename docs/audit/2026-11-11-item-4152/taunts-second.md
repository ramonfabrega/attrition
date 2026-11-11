# Item 4152, second reading: `taunts`

- **Slug**: `taunts`
- **Leaves** (`LeaderData`, size `0x6ee4`): `last_taunt` (+0x354, `TauntRequest[8]`),
  `taunt_frame` (+0x374, `int[8]`), `incoming_taunt` (+0x394, `int[8]`),
  `incoming_taunt_who` (+0x3b4, `int[8]`), `incoming_taunt_frame` (+0x3d4, `int[8]`),
  `incoming` (+0x3f4, `int`).
- **Reader**: Claude Opus 5.5 (`claude-opus-5-5`), blind second reading, 2026-11-11.
- **Sources**: the decompile export (`~/ghidra-projects/decomp`), `types.txt`,
  the full `.text` listing at the job's `aa/exe.s`.

