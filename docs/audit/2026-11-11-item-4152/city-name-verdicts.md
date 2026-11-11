# Item 4152, adjudication: `city-name`

- Slug: `city-name`
- Leaf: `city_name` (`LeaderData` +0x40c, int)
- Adjudicator: Claude Opus 5.5 (`claude-opus-5-5`), 2026-11-11
- Inputs: `city-name-reading.md` (first), `city-name-second.md` (blind second), the decompile export (`~/ghidra-projects/decomp`), the `.text` listing (`exe.s`), `crates/sim`.
- Written per claim as settled; a leaf without its closing rule below was not finished.

## Leaf `city_name` (`LeaderData` +0x40c, int)

### Layout

Both readings: `Leader` holds `LeaderData` at offset 0, so `field_0x40c` on a
`Leader*` is `city_name`, and slot `i` is `0xe3a79c + i·0x6eec`. **Agree,
confirmed**: `types.txt` 42088-42092 (`LeaderData` size 0x6ee4, `who` +0x8,
`tribe` +0xc), and `Leaders::clear (006ed5d0)` memsets from
`&leaders.list[0].field_0x6c14 - 0x1b05·4`, which is the record's own byte 0.

### 1. Claims lined up

| claim | first | second | mark | verdict | deciding line |
|---|---|---|---|---|---|
| store `+= 1` on the winner slot | 73614a | 73614a | agree | confirmed | `incl 0xe3a79c(%eax)` at 73614a; `generate_name:229-230` |
| store `= 1` on own when own is 0, first-name branch | 7362d2 | 7362d2 | agree | confirmed | `movl $0x1,0xe3a79c(%eax)` at 7362d2 under `cmpl $0x0` at 7362c9; `generate_name:261-263` |
| `Leader::init` clears to 0 | 6e3b14 | 6e3b14 | agree | confirmed | `Leader::init:139`, listing 6e3b14 |
| `Leader::init` also memsets `[0, 0x6929)` when `param_2 < 0` | not made | made | one-sided | **confirmed** (redundant with the explicit 0) | `Leader::init:70-77`; pushes `0x6929, 0, %ebx` then call at 6e398c-6e3995 |
| `Leaders::clear` memsets the record | not made | made | one-sided | **confirmed**; callers `Game::close (00594b80)` 594faa and `Game::run_oos_recovery (005869e0)` 586dd6 | `Leaders::clear:12` |
| `LeaderData::walk_data` restores it on a save-game load (raw range `[+0x8, +0x692a)` when `leader_flags & 1`) | not made (said a load restarts at 0, for the *scenario* path) | made | one-sided | **confirmed**; the first's claim stands only for `ScenarioRead`, which both agree drops the value | `walk_data:16-17` (`&this->who` to `&this->field_0x692a`) |
| the sync checksum reads it through that same walk | not made | made | one-sided | **confirmed** | `CheckSums::check_one_leader:5` is `walk_data` on `leaders.list + param_2` |
| absolute `0xe3a79c` sites | 8 (7 in `generate_name`, 1 in `save_leaders_chunk`) | "4" in `generate_name` plus the save | disagree (count) | **corrected to the first's 8**: 736015, 73601d, 736144, 73614a, 736150, 7362c9, 7362d2, 9a791e. Both have the same two stores, so no writer moves | grep of `exe.s` |
| `Leader::init` call sites in `ScenarioRead::load_leader_chunk` | 3 | 6 | disagree | **corrected to 6**: 9ac1a9, 9ac24f, 9ac3bc, 9ac4c7, 9ac59c, 9ac69f. Same function either way; no rule moves | `call 0x6e3930` grep |
| `0x40c(%reg)` stores outside the Leader family are other classes | listed | listed | agree | confirmed. My own sweep (two-operand and single-operand stores, x87 and string ops) finds 12 sites; only 6e3b14 is a `Leader`. 9ad098 is the `GameInfo.player[]` clear (stride 0x8c run, `movw` to +0x414 at 9ad0a4) | `exe.s` 1768470-1768505 |
| no whole-record copy | made | made (adds the memsets and walk) | agree + one-sided | confirmed, with the three bulk writers above as the complete bulk set | no `$0x6ee4`/`$0x1bb9` operand; the memsets and the walk are what remain |

**Writer completeness.** I found no writer that neither reader named. The
second reader's list is the complete one: two value stores in
`City::generate_name (00735c90)`, the clear in `Leader::init (006e3930)`, and
the three bulk writers (`Leader::init`'s memset, `Leaders::clear`,
`LeaderData::walk_data` in input mode).
