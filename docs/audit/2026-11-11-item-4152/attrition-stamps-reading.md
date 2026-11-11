# Item 4152, family 5 `attrition-stamps` — first reading

Leaves: `attrition_stamp` (LeaderData +0x1f4), `attrition_stamp2` (+0x1f8), `attrition_stamp3` (+0x1fc), `att` (+0x7f0), `give_att_disabled` (+0x7f8), all `int`.

Reader: Claude Opus 5.5 (claude-opus-5-5), first reading, 2026-11-11. Sources: the decompile export (`~/ghidra-projects/decomp`), the listing (`llvm-objdump` of `riseofnations.exe`), `types.txt`, and `crates/sim`.

## Layout note

`types.txt` lists `Leader` (size 0x6eec) with no fields of its own and `LeaderData` (0x6ee4). The decompile reaches these leaves as `leaders.list[i].field_0x1f4` and as `this->field_0x1f4` in `Leader::init (006e3930)`, which also writes `field_0x7f0`/`field_0x7f8` beside them; `Leader::calc_attrition (006cdea0)` tests `field_0x7f8` and writes `field_0x7f0`. The `Leader` offsets therefore equal the `LeaderData` offsets (no prefix; the 8 extra bytes are after `LeaderData`). The listing check is in each leaf section (absolute stores at `0xe3a390+off`).

