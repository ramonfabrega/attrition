# Item 4152, family 17 (`spells`): adjudication

- Slug: `spells`. Leaf: `LeaderData::tech` (+0x6c0c, `BitMask<806>`),
  bits `0x275..0x2ab`.
- Readings: `spells-reading.md` (first), `spells-second.md` (blind second).
- Adjudicator: Claude Opus 5.5 (claude-opus-5-5), 2026-11-11.
- Method: the export (`~/ghidra-projects/decomp`), the full `.text`
  listing (`exe.s` in the job scratch), `crates/sim` grep, and an
  independent re-read of run 1562's frame-91 packet. Nothing run of the
  original.

## Leaf `tech.ptr`, bits `0x275..0x2ab`

### 1. Claim by claim

| # | claim | first | second | mark |
|---|---|---|---|---|
| 1 | the one setter in a plain game is `Leader::init (006e3930)`'s spell pass, `6e45d9..6e461d`, writing `tech` only | yes | yes | agree |
| 2 | predicate: `type_avail(t,1) != 0` and (`preq[0] != -1` or `spell_flags & 1 == 0`) | yes | yes | agree |
| 3 | `type_avail` of a spell reduces to `has_preq` + `type_eligible`: tribe bit in `tribe_mask`, no preq below -1, preq -1 or held, a tech preq's age within `ending_technology` | yes | yes | agree |
| 4 | the tribe test is `tribe_can_type (006d9380)` (first; `006d9410` "second body, not read") vs `tribe_can_type (006d9410)` (second) | 006d9380 | 006d9410 | disagree |
| 5 | the eight `-2` rows are refused by `type_eligible` (first) / by both `type_eligible` and `has_preq` (second) | `type_eligible` | both | one-sided (the `has_preq` half) |
| 6 | split: 46 set (every `preq[0] == -1` row), 8 clear (`-2`), `RADARSPELL` 0x299 clear unless Industrial Age is held at init | from `craftrules.xml` columns | from the loaded `SPELLTYPE` dump and the packet | agree |
| 7 | no spell row has `spell_flags` bit 0 (letter `a`); every `tribe_mask` is 24 ones | yes | yes | agree |
| 8 | `gain_tech (006dcb60)` sets bit `param_1` unconditionally (line 303); no loop inside it reaches `0x275` | yes | yes | agree |
| 9 | the only plain-game route into `gain_tech` with a spell is `Build::finished (00628490)`'s fall-through when `has_spell` is 0 | yes | yes | agree |
| 10 | that fall-through cannot fire with the shipped data (first) / is reachable by "a queued spell that lost availability" (second) | cannot | can, rarely | disagree |
| 11 | clears: `LeaderData::LeaderData (006d7540)`, `Leaders::clear (006ed5d0)`, `Leader::close (006b7f80)`, `Leader::init`'s `BitMask<806>::init` and its `param_2 < 0` arm | yes | yes | agree |
| 12 | `Leader::lose_tech (006d2850)` cannot clear a spell (unit or tech arms only) | yes | yes | agree |
| 13 | out of plain play: cheats (`action_cheat_give_techs (00593180)`, `action_cheat_zero_techs (00593120)`), console, `disable_type_by_tribe (00a006a0)`, scenario/editor/CtW, `walk_data` | yes | yes (no `zero_techs`) | agree |
| 14 | `Leader::init`'s callers: `Setup::build_game (005ac190)` (first); plus `Setup::build_leader (005abc70)` as a second plain-game caller, and `load_leader_chunk` "three sites" (second) | build_game | build_game + build_leader | one-sided |
| 15 | readers: `has_spell (006e0bc0)` reads the bit only for a letter-`a` row; `has_tech (006e0c80)` reads it raw; `log_data`; no plain-game sim decision reads the bits | yes | yes | agree |
| 16 | crate: `TechTree::start` (`crates/sim/src/tech.rs:2235`) has no spell pass; `Kind` ends at `Gov`; `bits_of`/`tree_type_index` in `checksum/leaders.rs` cannot name `0x275+`; the SEAM at `cities/queue_disband.rs:173` | yes | yes | agree |
| 17 | crate: `orders::SpellType` (loaded at `crates/rondata/src/load.rs:727`) has `flags` but not `PREQ0`/`TRIBE_MASK` | yes | "did not check" | one-sided |
| 18 | plain game reaches it: yes, at setup, every live leader | yes | yes | agree |
| 19 | run 1562 frame 91: 46 of 55 set in slots 0 and 1, none in `tech_at_start`, the nine clear = eight `-2` + `RADARSPELL` | — | yes, from the packet | one-sided |

### 2. Predicates, spot-checked against the body (claims 1, 2, 3, 6, 7)

- **Confirmed.** Listing `6e45d9..6e461d`: `t` from `0x275`, `call
  0x6e33a0` (`type_avail`) with `(t, 1)` (`6e45de/6e45e0`), `je` skips on
  zero (`6e45ed`); then the type row from `0xc061a8 -> +0x10 [t]`,
  `cmpl $-1, 0x30(row)` (`6e45fa`), `jne` to the set; else `testb $1,
  0x1c8(row)` (`6e45ff`), `jne` skips; set is `leal 0x6c0c(%ebx)` +
  `call 0x450360` with `(t, 1)` (`6e4608..6e4611`); bound `cmpl $0x2ac`
  (`6e4617`, exclusive). No store to `0x6c80` (`tech_at_start`) in the loop.
  The next instruction is `call 0x6e32a0` (`reset_obs_flags`, `6e4621`), so
  the pass sits between the unit pass and `reset_obs_flags`, as both said.
  Deciding line: `6e45fa`/`6e45ff`/`6e460b`.
- `type_avail (006e33a0)` lines 15–21: `has_preq` zero returns 0, then
  `type_eligible` non-4 returns as is; a spell then falls through the unit
  (vslot `0xc`), building (`0x14`) and government (`0x2c`) arms to 4.
- `type_eligible (006dbd10)`: line 21 `tribe_can_type != 4` returns 0; the
  preq loop (lines 39–67) returns 0 for any `get_preq < -1` (line 43) and
  for a non-bonus, non-epoch tech preq whose `techtypes[q] +0x1c8` exceeds
  `ending_technology` (lines 55–57); the tail (lines 199–213) returns 4 for
  a type outside `0x220..0x274` in the default `is_tech_type`. A spell
  answers 4 when its preqs pass. Deciding line: `type_eligible (006dbd10)`
  lines 43, 199–207.
- **The values the predicate meets, confirmed independently.** I parsed
  run 1562's own `SPELLTYPE` records (`gamelog_p.txt`, frame 91 block,
  55 records, `type` exactly `0x275..0x2ab` in order): every `tribe_mask`
  is `16777215`; no `spell_flags` is odd; `preq` is `[-1,-1,-1]` for all but
  `0x276, 0x278, 0x279, 0x27e, 0x282, 0x284, 0x285, 0x287` (`[-2,-1,-1]`)
  and `0x299` (`[548,-1,-1]`, `INDUSTRIAL_AGE` 0x224). The 46/8/1 split
  both readers gave is the loaded data's, not only the file's.

### 3. Claim 4, which `tribe_can_type`

**Corrected (minor), for the second.** `type_eligible`'s call is `6dbd1e:
call 0x6d9410`, the `TypeIndex` overload `tribe_can_type (006d9410)`. The
first reading's `006d9380` is the `Type*` overload, which `type_eligible`
does not call (it is called from `Object::eject_contents (0064cd20)` at
`64d09f`, from `type_eligible`'s graft walk, and elsewhere). The two bodies
compute the same thing for a spell: not a unit (vslot `0xc`), so straight to
`(1 << (tribe & 31)) & type +0x10` (`tribe_mask`), times 4. A slot with
`tribe -1` tests bit 31, which a 24-bit mask never has, so it fails — the
first's remark holds through either body. Deciding line: `6dbd1e`.

### 4. Claim 5, the `-2` rows in `has_preq`

**Confirmed (the second's extra half).** `has_preq (006db810)` lines
136–140: a negative `get_preq` goes to `has_tech`; `has_tech (006e0c80)`
lines 13–16 answer 1 only for `TYPE_NONE` (-1) and 0 for
`TYPE_ANY_BUILDING` (-2, `enums/TypeIndex.txt` line 868). So a `-2` row is
refused twice: `has_preq` first (it runs first in `type_avail`), then
`type_eligible` line 43 would refuse it too. Immaterial to the result;
material only to a port that skips one of the two checks.

