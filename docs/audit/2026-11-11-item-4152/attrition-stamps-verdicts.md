# Item 4152, family 5 `attrition-stamps` — adjudication

- Family 5, slug `attrition-stamps`. Leaves (`LeaderData`, all `int`):
  `attrition_stamp` (+0x1f4), `attrition_stamp2` (+0x1f8),
  `attrition_stamp3` (+0x1fc), `att` (+0x7f0), `give_att_disabled` (+0x7f8).
- Inputs: `attrition-stamps-reading.md` (first), `attrition-stamps-second.md`
  (blind second), the README row 5, `docs/ATTRITION.md` §"Strength",
  `crates/sim/src/territory/attrition.rs`.
- Adjudicator: Opus 5.5 (`claude-opus-5-5`), wave 3, 2026-11-11.
- Sources: the export `~/ghidra-projects/decomp/funcs/`, the `.text` listing
  `r4152/aa/exe.s` (llvm-objdump, AT&T syntax), `crates/sim/src`, and the
  kept dumps under `~/ron-golden/` (read by `grep -n` and `sed -n` ranges).
- Written leaf by leaf; the `att` floor is settled first because the spawn
  line asks for it.

