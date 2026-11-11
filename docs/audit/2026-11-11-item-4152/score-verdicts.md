# Item 4152, family 18 (`score`) — adjudication

- Family 18, slug `score`; the twelve leaves `score` (+0x18) … `score_combat` (+0x44).
- Inputs: `score-reading.md` (first), `score-second.md` (blind second), README table "The families".
- Adjudicator: Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Sources: the listing (`/Users/rf-studio/.claude/jobs/c88e6820/tmp/r4152/aa/exe.s`, `llvm-objdump` of
  `riseofnations.exe`), the decompile export (`~/ghidra-projects/decomp`), `crates/sim/src`.
- Method: every part's integer formula checked instruction by instruction in the listing (operand
  order of each multiply, each divide's idiom and sign fix, each clamp's compare and branch sense,
  which counts sum into `n`). Settled per leaf below, appended as each closed.

