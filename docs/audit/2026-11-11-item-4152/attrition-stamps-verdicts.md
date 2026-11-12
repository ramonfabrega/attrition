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

## The nets, re-run once for all five leaves

- **Listing, absolute.** `0xe3a584|588|58c|e3ab80|e3ab88` occur twelve times
  in `exe.s`: 5e132c, 5e14f9, 5e1586, 5e1710, 5e1868, 5e1880 (all
  `Unit::process_attrition (005e11a0)`), 5e1a25, 5e1b8a
  (`Unit::suffer_attrition (005e1a10)`), 640b3a, 640bb7 (`Wall::process
  (00640450)`), a02926, a02966 (the two trigger actions). No absolute
  `0xe3ab80` (`att`). The constant-slot forms `0xe3a390 + k·0x6eec + off`,
  k = 1..9, occur nowhere. **agree** with both readers.
- **Listing, register-relative stores.** Every `mov/inc/dec/add/sub/or/and/
  xor/neg/not` whose destination is `0x{1f4,1f8,1fc,7f0,7f8}(%reg)`
  (`esp`/`ebp` excluded): 149 lines, attributed through `INDEX.tsv`. The
  leader stores are exactly `Game::do_frame (00591ef0)` 59232b,
  `Leader::gain_tech (006dcb60)` 6e038a, `Leader::calc_attrition
  (006cdea0)` 6cdfdc, and `Leader::init (006e3930)` 6e3dda/6e3de6/6e3df1/
  6e4984/6e498a; `Leader::init` 6e3d63 / 6e4b06 and `Leader::plan_strategy
  (006b9620)` 6b9bff are other words (below). The rest are non-leader bases
  both readers rejected, plus `Objects::init`/`Objects::close` (65f2aa,
  65e7a6, base `this` = `Objects`), `PlayerProfile`, `SteamLobbyMgr`,
  `BuildType`/`UnitType`/`SpellType` inits and windows, which neither
  named and none of which is a leader. **No writer neither reader found.**
- **Bulk.** `Leaders::clear (006ed5d0)` memsets each slot's first 0x6929
  bytes (covers all five offsets); `LeaderData::walk_data (006d6750)`
  writes them only on a load (the lost-capital verdicts traced its
  callers). One-sided (first reading); **confirmed**, not a plain-game
  writer.

## Leaf 4 — `att` (+0x7f0, int) — settled first

### The floor on a base of 0 (the spawn line's question)

Both readings say `att` floors at 1 even when the tech base is 0, against
`docs/ATTRITION.md` §"1. Strength" step 1 ("No steps means no attrition at
all, which is where everybody starts") and against
`crates/sim/src/territory/attrition.rs:43–45` (`if tech_steps == 0 {
return 0; }`). **agree; confirmed, by the listing and by the dumps.**

**The listing** (`Leader::calc_attrition (006cdea0)`, 6cdea0–6cdfe4, read
whole):

- 6cdea4 `xorl %edi,%edi`; 6cdea6 `cmpl %edi,0x7f8(%ebx)`; 6cdeac `jne
  6cdfdc` — `give_att_disabled != 0` stores `edi = 0` and skips everything.
- 6cdeb3–6cdee1: `esi` runs 0x2dd..0x2e0, `edi` counts the leading run
  `has_preq (006db810)` answers for (`je 6cdee3` leaves at the first miss).
  The 0x2ad / `has_tribe_bonus(4)` test at 6cdeb8 compares the loop index,
  which starts at 0x2dd: dead.
- 6cdee3 `testl %edi,%edi; je 6cdef3` — **with `n = 0` the base load at
  6cdeec is skipped and `edi` stays 0**; there is no early exit to the
  store. 6cdeec `movl 0x1d4(%eax,%edi,4),%edi` = `C+0x1d8 + 4(n−1)` =
  `attrition_improved[n−1]` (`types.txt` 44916–44917: `attrition_upgrade`
  `int[4]` at +0x1c8, `attrition_improved` `int[4]` at +0x1d8; the
  decompile's `attrition_upgrade[n+3]` is the same address, misnamed).
- Each of the four arms (6cdf09–6cdf2b Colosseum `C+0x470`, 6cdf3b–6cdf5d
  Russian `C+0x74c`, 6cdf79–6cdf9b CtW `C+0xa0c`, 6cdfb5–6cdfd8 Kremlin
  `C+0x510`; `types.txt` 45019, 45202, 45365, 45059) computes
  `(pct + 100)·edi`, signed ÷100 (`imull 0x51eb851f; sarl 5; shrl 31;
  addl`), and **the deciding instruction is the `cmovel` (6cdf2b, 6cdf5d,
  6cdf9b, 6cdfd8) on the zero flag of that final `addl`**: a quotient of 0
  becomes 1, whatever made it 0. `(pct+100)·0 = 0 → 1`.
- 6cdfdc `movl %edi,0x7f0(%ebx)` — the one store.

So `att = 1` for a leader with no attrition step and at least one applying
multiplier; `att = 0` only with no step and no multiplier, or under
`give_att_disabled`.

**The dumps** (all read by `grep -n` and `sed -n` ranges):

- `~/ron-golden/run981/map-7/gamelog.txt` (Great Sahara, `DUMP_ALL` start,
  lobby `DIFFICULTY=4` only, so the default Ancient start; receipt players
  `(who 0, tribe 4)`, `(who 1, tribe 13)`). Five full dumps
  (`BEGIN FULL DUMP` at 4786, 1200614, 2426127, 3651787, 4877444); the
  first, 4786–1200613, prints `frame 0` at 1032107.
  - **Line 809939 `att 0`** belongs to the record opened by `BEGIN
    LEADERDATA` at **809534**: `who 0` (809535), `tribe 4` (809536, the
    Nubians), `city_num 1` (809755).
  - **Line 822197 `att 1`** belongs to the record opened at **821792**:
    `who 1` (821793), `tribe 13` (821794), `city_num 1` (822013),
    `tech_frame 0`, `wonder_mark 0`, `anti_att 256.000000` (822198).
  - The same pair repeats in all five dumps through frame 37: Russian
    `att 1` at 822197, 2018002, 3243574, 4469225, 5695006; Nubian `att 0`
    at 809939, 2005744, 3231316, 4456967, 5682748.
- **Why this record isolates the floor.** At frame 0 neither leader has
  gained a tech (`tech_frame 0`) or holds a wonder, and the CtW arm needs
  `semaphore[2] & 2` (a campaign). `has_tribe_bonus (006e1370)` answers
  for who 1: `city_num 1`, so the starting-town/city gate passes, and the
  crate's own nation table makes tribe 13 the Russians (`ids.rs:37`
  `RUSSIANS = 13`; `Sim::set_tribe`, `cities/nations.rs`, takes the power
  index from the tribe index). Under the formula, `att 1` for who 1 has
  exactly one reading: `n = 0`, Russian arm applies, `0 → 1`. The
  alternatives fail: `n = 1` with the Russian arm gives `1·200/100 = 2`,
  `n = 0` without it gives 0. who 0 (no multiplier, `n = 0`) gives 0, as
  printed. **The floor is diff-backed by the original's own dump.**
- **Other kept dumps** (same scan: `BEGIN LEADERDATA` → `who`/`tribe`/
  `city_num`/`tech_frame`/`att`, first records only):

  | dump | lobby | Russian record | `att` | reading |
  |---|---|---|---|---|
  | run981/map-7 | Ancient start | who 1, line 822197, tech_frame 0 | **1** | `n = 0`, Russian floor |
  | run1257/map-7 | `STARTING_TECHNOLOGY=2` | who 1, line 825324, tech_frame 0 | 2 | Nubian who 0 prints 0 on the same start (813066), so the Russian holds one step the Nubian does not: `1·200/100`. Consistent with `gain_tech`'s `russian_attrition_upgrades` arm (decompile 1763; `C+0x750`) handing the Russians the attrition chain; that arm was not read to the end |
  | run1148, run1400 (map-18), run1179 | Modern/Information starts | who 1, tech_frame 0 | 2 | same shape; Nubian 0 |
  | run1467/map-7 | `STARTING_TECHNOLOGY=7` | who 2, line 841573 | 2 | French who 1 and Nubian who 0 print 0 |
  | run1161, run1149b, run1468b, run1483, run1492 | held Modern/Information | tech_frame 0 | 2 | later records 4/8/16/32 as steps and the Kremlin arrive |

  Every Russian value on disk fits the listing's formula; only run981 has
  a Russian at `n = 0`, and it prints 1. The Nubian/French zeros on starts
  where the Russian holds a step also show the step is Russian-only there,
  not a start tech.

**Verdict: corrected — `docs/ATTRITION.md` §"1. Strength" step 1 and
`attrition::strength`'s early return are wrong.** The crate holds
`strength 0` where the original holds `att 1` for every Russian leader
with a city and no attrition step (from frame 0 of an Ancient start), and
for a Colosseum or Kremlin holder with no step. The rule: start `x` at 0,
skip the base lookup when `n = 0`, apply each arm, and after each write
`if x == 0 { x = 1 }`. The crate's `.max(1)` differs from `== 0 → 1` only
for a negative quotient (`pct < −100`), unreachable with shipped values;
port the original's form. Consequence (a reading of the consumer, both
readers): `UnitData::get_attrition (00608fd0)` 608fe6 reads the territory
owner's `att`; `0` means no attrition, so Russian land bleeds a hostile
unit from the first frame in the original and not at all in the crate. No
capture with a foreign unit in Russian land before a step was looked up.

### When `att` is recomputed (the driver's timing question)

Three direct calls of 0x6cdea0 exist in `.text`: 6cf974
(`Leader::calc_unit_stats (006cf970)`, its first call), 6dd3d6
(`Leader::gain_tech (006dcb60)`), 6e49bc (`Leader::init (006e3930)`).
**agree** with both readers.

- **The direct `gain_tech` arm is narrow.** 6dd393–6dd3c4: the gained
  type's vslot `+0x38` true (`je 6dd3e3`), its techtype record's `+0x14 ==
  1` (6dd3a2 `cmpl %edx,0x14(%eax)` with `edx = 1`), and the leader's
  civic level `0x6eb8→+0xec ^ 0x63187` equal to `C+0xad4`
  (`ctw_wonder_min_civics`, 6dd3be). Only then 6dd3c8 `calc_pop_cap`,
  6dd3cf `calc_anti_attrition`, **6dd3d6 `calc_attrition`**, 6dd3db
  `Region::fix_borders`. An ATTRITION step is not that arm's ordinary
  case. **confirmed** (both readers; the anti-att verdicts' C5/E3 agree).
- **Every other tech reaches `att` through the dirty fold.** `gain_tech`'s
  tail, 6e0315 `orl $0xc000000` (decompile 2313), sets the
  `0x4000000` bit on every tech that is not sent past it by the two
  `goto LAB_006e05d9` exits (decompile 883, 935: a non-tech arm and a type
  whose vslot `+0x28` is 0). `Leaders::process_all (006ed2a0)` decompile
  42–44 tests the bit, clears it, and calls `calc_unit_stats`, whose first
  call is `calc_attrition`. The other setters of the bit (`orl $0x4000000`
  / `$0xc000000`): `Build::activate` 626f0b, `Build::close` 628f3f,
  `Leader::gather` 6ce3d0, `Leader::calc_gather` 6cf5ea,
  `Leader::lose_tech` 6d2afd, `gain_tech` 6dd993 and 6e0315, plus
  5e56cc and 66aee6/66aef6 that neither reader attributed (not followed;
  they only make the refresh more frequent).
- **The order inside a frame.** `Game::do_frame (00591ef0)` calls
  `Leaders::process_all` at decompile line 199, then
  `NetDaemon::process_all` (200), `Leaders::strategy_all` (267) and
  `Objects::process_all` (275). The research path into `gain_tech` is
  `Build::finished` (call at 628548) and `Build::queue_up` (6212bf), both
  reached from building and command processing after line 199; the other
  callers are `Leader::init`, `gain_tech` itself (free techs), CtW setup
  and `ScenarioFuncSet`. So a tech completed on frame `f` by a building
  leaves `att` at its old value for the rest of frame `f` — including
  every unit `Unit::process_attrition` refreshes after that building in
  `Objects::process_all`, and the end-of-frame dump of frame `f` — and the
  new value appears in `Leaders::process_all` of frame `f + 1`, before any
  unit of `f + 1` reads it.
- **The crate.** `Sim::apply_gained` (`crates/sim/src/tech/gain.rs:601`)
  sets both dirty flags and then calls `self.calc_attrition(who)` at
  `:608` eagerly, and `Sim::calc_unit_stats`
  (`crates/sim/src/economy/rares.rs:679`) does not call it. **Verdict:
  corrected — the crate updates `att` one frame early** for every tech a
  building completes, and never for the triggers the original also folds
  (wonder `Build::activate`/`Build::close`, `lose_tech`, the gather
  paths). The port: drop the call from `apply_gained`, call
  `calc_attrition` first in `Sim::calc_unit_stats` (the `0x4000000` fold,
  which the crate already raises for a wonder at
  `crates/sim/src/cities/city.rs:2491`, `:2887`), keep a call at the
  `Leader::init` point, and add the `ctw_wonder_min_civics` direct arm
  only if a reachable game needs it (its predicate equals a CtW constant).
  Dump witness: see "Addendum: the one-frame lag in a dump" after the
  verdict table.

### The rest of the leaf, claim by claim

| claim | first | second | mark | verdict, deciding line |
|---|---|---|---|---|
| writers: `calc_attrition` 6cdfdc, `init` 6e4984 (0, then 6e49bc recompute) | yes | yes | agree | **confirmed**: the register net above |
| `give_att_disabled` short-circuit → 0, no floor | yes | yes | agree | **confirmed**: 6cdea6/6cdeac |
| leading run of 0x2dd..0x2e0, 0x2ad arm dead | yes | yes | agree | **confirmed**: 6cdeb3–6cdee1 |
| base `attrition_improved[n−1]` (decompile name wrong) | yes | yes (names both arrays) | agree | **confirmed**: 6cdeec; `types.txt` 44917 |
| order Colosseum → Russian → CtW → Kremlin, truncating, `== 0 → 1` | yes | yes | agree | **confirmed**: 6cdef3–6cdfd8 |
| CtW predicate: `game+0x822 & 2` and leader byte `+0x6900` (`num_bonus_cards[10]`) | yes | yes (names the byte) | agree / one-sided name | **confirmed**: 6cdf65, 6cdf6e |
| `has_tribe_bonus` is 0 under `info.flags & 4`, or no starting town and `city_num == 0`, or `tribe < 0` | — | yes | one-sided | **confirmed**: `LeaderData/has_tribe_bonus@006e1370.c` lines 8–11; the crate mirrors it (`tech.rs:1102–1106`). So a nomad-start Russian with no city is `att 0` until its first city |
| shipped pcts 50 / 100 / 50 / 100 | — | yes | one-sided | **open** for `ctw_attrition` (unreachable); the other three match `docs/ATTRITION.md` §1 |
| reader: `UnitData::get_attrition (00608fd0)` 608fe6, owner's slot, `att == 0` → none, else divisor after the aged-up scaling | yes | yes | agree | **confirmed**: 608fe6 `movl 0x7f0(%ecx,%eax),%ebx` is the only indexed read in `.text` |
| `LeaderData::get_attrition (006e0b60)` has no caller | yes | yes | agree | not re-checked (accessor) |
| crate gap: `StrengthMods` built with `russian` only; Colosseum/Kremlin false | yes | yes | agree | **confirmed**: `territory/upkeep.rs:43–46`; the doc comment ("this crate holds no wonder a player owns", `:20–22`) is stale: `tech.rs:600` `COLOSSEUM`, `:611` `KREMLIN` and `Holdings::wonders_held` (`economy/holdings.rs:572`) exist |
| crate site `Sim::calc_attrition` (`territory/upkeep.rs:24`) → `attrition::strength` (`territory/attrition.rs:42`) | yes | yes | agree | **confirmed** (grep) |
| plain game reaches it | yes | yes | agree | **confirmed**: run981 |

### The rule as a builder should port it

- Writers: `Leader::calc_attrition (006cdea0)`; `Leader::init` (0, then
  the same recompute); `Leaders::clear` (0); a load.
- Formula, integers: `if give_att_disabled { att = 0; return }`;
  `n` = leading run of ATTRITION1..4 held; `x = if n > 0 {
  attrition_improved[n−1] } else { 0 }`; then for each of Colosseum
  (`colosseum_attrition`), Russian bonus (`russian_attrition`), CtW
  (`ctw_attrition`, campaign only), Kremlin (`kremlin_attrition`), in
  that order and only when it applies: `x = (100 + pct)·x / 100`
  (truncating toward zero), `if x == 0 { x = 1 }`. `att = x`.
- When: in `calc_unit_stats` on the `0x4000000` fold at the next
  `Leaders::process_all` (tech gained or lost, wonder activated or
  closed, gather paths), plus `Leader::init`, plus the narrow
  `ctw_wonder_min_civics` arm of `gain_tech`. Not eagerly in
  `apply_gained`.
- Crate site: `attrition::strength` (`territory/attrition.rs:42`) loses
  its early return; `Sim::calc_attrition` (`territory/upkeep.rs:24`)
  gains the wonder mods and a `give_disabled` short-circuit and moves
  under `Sim::calc_unit_stats` (`economy/rares.rs:679`).
- Plain game: yes, from frame 0 for a Russian with a city.
- Confidence: high (listing read whole; the floor diff-backed by run981's
  start dump). Open: the `russian_attrition_upgrades` arm's exact grant
  (it explains run1257's 2 but was not read to the end); the
  unattributed `0x4000000` setters 5e56cc, 66aee6, 66aef6.

## Leaf 1 — `attrition_stamp` (+0x1f4, int, frames)

| claim | first | second | mark | verdict, deciding line |
|---|---|---|---|---|
| W1 peace arm `= frame` (5e1880), W2 grace sub-arm `= max(1, grace + frame − 900)` (5e1868), W3 assassin arm `= frame` (5e1586), all on the **unit owner's** slot | yes | yes | agree | **confirmed**: the absolute net; decompile `Unit/process_attrition@005e11a0.c` 239–249, 281 |
| re-arm gate `stamp == 0 \|\| frame − stamp > 900`, strict | yes | yes | agree | **confirmed**: 5e150b `cmpl $0x384,%eax`; 5e1510 `jle` (skip at exactly 900) |
| peace arm taken when `!is_enemy(owner, tile)` or (`semaphore[2] & 2` and `!war_allowed`) | yes | yes | agree | **confirmed**: decompile 160–161 |
| assassin arm: `is_enemy`, `team_style == 2`, tile owner ≠ `get_target()` | yes | yes | agree | **confirmed**: decompile 264–268 |
| the store is not gated on the console; only the message and sound are | yes | yes | agree | **confirmed**: decompile 239–249 (store after the console block closes at 238) |
| **grace sub-arm reachability** | dead with shipped data | "reachable by diplomacy in a plain game" | **disagree** | **corrected (second reading wrong)**: the sub-arm needs `bVar4 && bVar2`; `bVar4` is false whenever both `broke_alliance` tests read `delay <= frame − stamp` (decompile 163–169), and `ALLY_TO_WAR_DELAY`/`GRACE` are absent from `rules.xml` and load as −1 (`docs/ATTRITION.md` check 12, line 325–328; `territory/attrition.rs:431–433`), so `−1 <= frame − stamp` always holds and `bVar4` is never set. A broken alliance changes nothing with shipped data; a mod could open it |
| W4 `Game::do_frame (00591ef0)` 59232b `= 0` for every slot with `leader_flags & 1`, when `rush_rules > 8`, `frame == rush_rules[r].minutes·900` (`+0x3c`, 0x58 stride), `team_style ∉ {0, 8, 11}` | yes | yes | agree | **confirmed**: decompile 202–225; it runs after `Leaders::process_all` (199) and before `Objects::process_all` (275) |
| W5 `Leader::gain_tech` 6e038a `= 0` for every live slot, age-based rule reached while war barred, `Game::teams_locked` | yes | yes | agree | **confirmed**: decompile 199 (`bVar41 = war_allowed == 0`), 2314–2328 |
| W6 `Leader::init` 6e3dda, `Leaders::clear`, a load | yes (all) | init only | one-sided (bulk) | **confirmed**: the bulk note above |
| readers: `process_attrition` (5e14f9, 5e1710) and `log_data` | yes | yes | agree | **confirmed**: the absolute net |
| crate: no stamp; `Sim::end_the_peace` (`leaders/diplomacy.rs:74`) misnames the store as `ScenarioEditor`'s `update_leader_list` (comment at `:87–88`) | yes | yes | agree | **confirmed**: the store's base is `$0xe3a390` stepped by 0x6eec; `update_leader_list` is `ScenarioEditor +0x1f4`. Same mislabel at `docs/AI.md` 20956, 20968, 21011 (first reading; not re-read) |
| crate: `Situation.assassin` hard-coded `false` | yes | — | one-sided | **confirmed**: `territory/upkeep.rs:331` |
| crate: no age-rule zeroing | yes | yes | agree | **confirmed**: no rush/war arm in `tech/gain.rs` (grep) |
| plain game: yes (peace pair, run1072 1392 on block 1393); assassin needs `TEAM_STYLE 2` | yes | yes | agree | **confirmed** for the peace arm (both cite the 1392 note); assassin and grace unreached |

**Rule.** On the 32-frame refresh of a unit standing in a non-allied
rival's land, past every exemption that precedes the arm: if the pair is
not at war (or war is not yet allowed in a campaign), or the style is 2
and the land is not the owner's target, and `stamp[owner] == 0 ||
frame − stamp[owner] > 900`, then `stamp[owner] = frame` (the grace form
is dead with shipped constants). Zeroed for every live slot at the timed
rush end (`Game::do_frame`, after `Leaders::process_all`) and at the age
rush end (`gain_tech`). Reset at init. Crate site: beside the `Situation`
in `Sim::process_attrition` (`territory/upkeep.rs:239`, arm outcome
`peace_attrition`/`assassin_attrition`), the timed zeroing in
`Sim::end_the_peace` (fix its comment), the age zeroing new in the
`gain_tech` port. Plain game: yes. Confidence high; open: the assassin
arm and the age zeroing have no capture.

## Leaf 2 — `attrition_stamp2` (+0x1f8, int, frames)

| claim | first | second | mark | verdict, deciding line |
|---|---|---|---|---|
| writer `Unit::suffer_attrition` 5e1b8a `= frame` when 0 (test 5e1a25), every slot; console gates only the messages | yes | yes | agree | **confirmed**: the absolute net |
| once-only: nothing re-arms it but init/clear; the rush zeroings write +0x1f4 only | yes | yes | agree | **confirmed**: 59232b, 6e038a are `0x1f4(%reg)` |
| `is_ally(owner, owner)` (self) gates the message | yes | yes | agree | **confirmed** by both from the listing; message only |
| callers: two, both in `Unit::process (00610bc0)` | yes (decompile 528, 282) | yes (610d35, 6117fe) | agree | **confirmed**: `calll 0x5e1a10` at 610d35 and 6117fe only |
| stored before the damage, so a killing tick stamps | yes | — | one-sided | **confirmed**: the crate's walk mirrors it, `territory/suffer_view.rs:94–117` stamps before `v.hurt` |
| crate site | `Sim::suffer_attrition` top (`upkeep.rs:286`) | `SimSuffer::warned_at`/`stamp_warned` stubs (`suffer_view.rs:152–155`) | disagree (granularity) | **corrected to the second's**: `Sim::suffer_attrition` (`upkeep.rs:286`) builds a `SimSuffer` and calls `suffer_attrition_steps` (`:296–301`), whose `v.warned_at(who) == 0 → … v.stamp_warned(who)` is already in the original's place; the port fills the two stubs (return 0 / no-op today) with a per-leader word, not a new test at the top |
| plain game: yes | yes | yes | agree | **confirmed** |

**Rule.** `stamp2[owner] = frame` on the slot's first attrition tick
(period tick or detected decoy), every slot, before the damage; never
re-armed. Reset at init. Crate site: `SimSuffer::warned_at` /
`stamp_warned` (`territory/suffer_view.rs:152–155`). Plain game: yes.
Confidence high.

## Leaf 3 — `attrition_stamp3` (+0x1fc, int, frames)

| claim | first | second | mark | verdict, deciding line |
|---|---|---|---|---|
| writer `Wall::process` 640bb7 `= frame` | yes | yes | agree | **confirmed**: the absolute net |
| unstarted site: `disband(0)` then straight to the message and the store, **no** `stamp3 == 0` test | yes | yes | agree | **confirmed**: 6409d9 `calll 0x6455c0`; 6409de `jmp 0x640b43`, past the test at 640b3a |
| started: `take_damage(8, …)` via vslot `+0x16c`; nonzero → return, no stamp; else store only if 0 | yes | yes | agree | **confirmed**: 640b22 `calll *0x16c(%eax)`, 640b2a `jne 640bbd`, 640b3a `cmpl $0,0xe3a58c(%eax)`, 640b41 `jne 640bbd` |
| predicate: `(frame + id) & 15 == 0`, owner's `disable_building_attrition` 0, `& 31` too unless rush rules bar war; owner of tile ≥ 0, ≠ own, not allied | yes | yes | agree | **confirmed** for the 32 mask: 6409e3 `andl $0x8000001f` (16 mask not re-traced; both read it in the decompile) |
| rejected 6e3d63 (`+0x2d0 + 4i` diplomacy loop) and 6e4b06 (16-bit, other record) | yes | yes | agree | **confirmed**: the register net |
| crate site `Sim::process_building` (`cities/city.rs:2947`, block ~2997–3010); crate cadence 32 only (`cities/build.rs:1083`) | yes | yes | agree | **confirmed** (grep) |
| plain game: yes | yes | yes | agree | **confirmed** by reading; run551's 682 is a prediction (first reading), not looked up |

**Rule.** In `Wall::process`'s rival-land block: an unstarted site there is
disbanded and `stamp3[owner] = frame` every time; a started building takes
8 and, if alive and `stamp3[owner] == 0`, `stamp3[owner] = frame`. Reset at
init. Crate site: both arms of the enemy-territory block in
`Sim::process_building`. Plain game: yes. Confidence high; open: the
16-frame cadence before war under rush rules (unmodelled in the crate).

## Leaf 5 — `give_att_disabled` (+0x7f8, int)

| claim | first | second | mark | verdict, deciding line |
|---|---|---|---|---|
| writers `ScenarioFuncSet::enable_give_attrition (00a02900)` a02926 (0), `disable_give_attrition (00a02940)` a02966 (1), slot `arg − 1`, live (`flags & 1 && & 2`) | yes | yes | agree | **confirmed**: the absolute net |
| `Leader::plan_strategy (006b9620)` is **not** a writer (README row 5, `docs/CHECKSUM.md:1760`) | yes (6b9bff is a 16-bit store at `this + 0xbe6 + 0x7f8` = `reg_land`) | yes ("a 16-bit store into another record") | agree | **confirmed**: 6b9bff is `movw`, and the register net finds no other leader dword store at +0x7f8. The README row is wrong |
| `Leader::init` 6e498a (0) | yes | yes | agree | **confirmed** |
| readers: `process_attrition` 5e132c (tile owner's slot) and `calc_attrition` 6cdea6 | yes | yes | agree | **confirmed** |
| not printed by `log_data` | yes | — | one-sided | **confirmed** for run981's records (no such key between `misery` and `att`, 822196–822197) |
| **crate site** | `PlayerState::give_disabled`, read at `territory/attrition.rs:498` | "none … `PlayerState` no such field" | **disagree** | **corrected (second reading wrong)**: `territory/attrition.rs:257` declares `give_disabled`, `:272` defaults it false, `:498` `victim.take_disabled \|\| owner.give_disabled → Exempt(Disabled)` reads it. Nothing writes it |
| plain game: no (trigger layer) | yes | yes | agree | **confirmed** |

**Rule.** Set to 1 / 0 by the two trigger actions for a live 1-based slot;
nonzero exempts every unit on that leader's land at once and makes the next
`calc_attrition` store 0. Reset at init. Crate site: the field exists and
is read; the writer belongs to the trigger track, and `Sim::calc_attrition`
gains the short-circuit. Plain game: no. Confidence high.

## Verdict table

| leaf | claims confirmed / corrected / open | the rule in one line |
|---|---|---|
| `attrition_stamp` +0x1f4 | 13 / 1 / 0 (second reading's "grace reachable in a plain game" corrected: dead with shipped −1 constants) | `= frame` on a unit's refresh in a not-at-war (or style-2 non-target) rival's land when `stamp == 0 \|\| frame − stamp > 900`; `= 0` for every live slot at the timed and the age rush end; owner's slot; crate: none yet, `end_the_peace` comment misnames it |
| `attrition_stamp2` +0x1f8 | 6 / 1 / 0 (crate site: the `SimSuffer::warned_at`/`stamp_warned` stubs, not a new test) | `= frame` once, on the slot's first attrition tick, before the damage; never re-armed |
| `attrition_stamp3` +0x1fc | 7 / 0 / 0 | `= frame` on every unstarted site disbanded in rival land, and on a started building's first surviving hit when 0 |
| `att` +0x7f0 | 12 / 2 / 1 (**the zero-base floor: `docs/ATTRITION.md` §1 step 1 and `attrition.rs:43` wrong, diff-backed by run981**; **the crate recomputes a frame early** in `apply_gained`; open: `ctw_attrition`'s shipped value) | `att = 0` if `give_att_disabled`, else `x = n ? attrition_improved[n−1] : 0` then each applying arm (Colosseum, Russian, CtW, Kremlin) `x = (100+pct)·x/100; if x == 0 {x = 1}`; recomputed in `calc_unit_stats` on the `0x4000000` fold at the next `Leaders::process_all` |
| `give_att_disabled` +0x7f8 | 6 / 1 / 0 (second reading's "no crate field" corrected: `PlayerState::give_disabled`, `attrition.rs:257`, read at `:498`); README row's `plan_strategy` writer wrong (both readers) | set 1 / 0 by the two trigger actions for a live slot; exempts the leader's land and zeroes `att`; trigger layer only |

