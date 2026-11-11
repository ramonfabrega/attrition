# Item 4152, family 11 `lost-capital` — adjudication

- Family 11, slug `lost-capital`. Leaves: `LeaderData::lost_capital_stamp`
  (+0x414, int), `lost_capital_timer` (+0x418, int), `lost_capital_modifier`
  (+0x41c, int).
- Inputs: `lost-capital-reading.md` (first), `lost-capital-second.md` (blind
  second), the README row 11, and `docs/CITIES.md` §8.5.
- Adjudicator: Opus 5.5 (`claude-opus-5-5`), wave 3, 2026-11-11.
- Sources: the export `~/ghidra-projects/decomp/funcs/`, the `.text` listing
  `r4152/aa/exe.s` (llvm-objdump, AT&T syntax), `crates/sim/src`.
- Written leaf by leaf; a leaf without its closing rule was not finished.

