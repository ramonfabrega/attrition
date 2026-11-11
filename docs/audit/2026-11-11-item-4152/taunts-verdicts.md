# Item 4152, family 9 (`taunts`): adjudication

- Slug: `taunts`. Leaves (`LeaderData`): `last_taunt` (+0x354,
  `TauntRequest[8]`), `taunt_frame` (+0x374, `int[8]`), `incoming_taunt`
  (+0x394, `int[8]`), `incoming_taunt_who` (+0x3b4, `int[8]`),
  `incoming_taunt_frame` (+0x3d4, `int[8]`), `incoming` (+0x3f4, `int`).
- Readings: `taunts-reading.md` (first), `taunts-second.md` (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text`
  listing (`exe.s` in the job scratch), `crates/sim` and `crates/rondata`
  grep. Nothing run of the original.

(Written per leaf as settled.)

