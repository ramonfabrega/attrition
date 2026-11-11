# Item 4152, family 9 (`taunts`) — first reading

- Family: 9, slug `taunts`.
- Leaves (`LeaderData` offsets, `types.txt` `struct /rise.pdb/LeaderData`,
  size `0x6ee4`): `last_taunt` (+0x354, `TauntRequest[8]`), `taunt_frame`
  (+0x374, `int[8]`), `incoming_taunt` (+0x394, `int[8]`),
  `incoming_taunt_who` (+0x3b4, `int[8]`), `incoming_taunt_frame` (+0x3d4,
  `int[8]`), `incoming` (+0x3f4, `int`).
- Reader: first reading, wave 1, model Claude Opus 5.5 (`claude-opus-5-5`),
  2026-11-11.
- Sources: the decompile export (`~/ghidra-projects/decomp`), the listing of
  `game/riseofnations.exe` (`llvm-objdump`), `crates/sim/src`.

