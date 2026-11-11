# attrition-stamps — second (blind) reading

- Slug: `attrition-stamps`
- Leaves (`LeaderData`, all `int`): `attrition_stamp` (+0x1f4),
  `attrition_stamp2` (+0x1f8), `attrition_stamp3` (+0x1fc), `att` (+0x7f0),
  `give_att_disabled` (+0x7f8)
- Reader: Claude Opus 5.5 (claude-opus-5-5), blind to the first reading.
- Sources: `~/ghidra-projects/decomp` export, the `.text` listing at
  `tmp/r4152/aa/exe.s`, `types.txt`; `docs/ATTRITION.md`,
  `crates/sim/src/territory/attrition.rs` for the comparison.

