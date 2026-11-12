# score: second (blind) reading

Slug `score`. Leaves: `score` (+0x18), `score_explored` (+0x1c), `score_territory` (+0x20), `score_units` (+0x24), `score_units_2` (+0x28), `score_buildings` (+0x2c), `score_economy` (+0x30), `score_pop` (+0x34), `score_unit_upgrades` (+0x38), `score_research` (+0x3c), `score_wonders` (+0x40), `score_combat` (+0x44), all `int` in `LeaderData`.

Reader: Claude Opus 5.5, blind (has not opened the first reading). Sources: decompile export `~/ghidra-projects/decomp`, the PE listing `exe.s`.

## 0. The family at a glance

Eleven of the twelve leaves are written only by `Leader::compute_score (006ec560)` and the
part functions it calls, all `__thiscall` on a `Leader*` in `esi`/`ebx`/`edi`
(`mov %ecx,%esi` etc. at each prologue). `Leader` (size 0x6eec) lists no fields of its own in
`types.txt`; it is `LeaderData` (0x6ee4) at offset 0 plus 8 trailing bytes. The proof that the
base is `LeaderData+0` and not shifted: the same `this` is read as `leader_flags` at `(%esi)`
(`testl $0x1800000,(%esi)`, 006ec59a) and as `who` at `0x8(%esi)` (`imull $0x6eec,0x8(%esi)`,
006ec5e7, used as the slot index into the global array at `0xe3a390`). So `0x18(%esi)` ..
`0x44(%esi)` are `score` .. `score_combat`. `score_combat` is the exception: the part function
`Leader::compute_combat_score (006bc250)` is an empty body and is never called; the leaf is
stepped event-by-event at kills, always through the absolute form `0xe3a3d4(%reg)` =
`0xe3a390 + 0x44 + slot·0x6eec`.

### 0.1 Cadence: `Leader::compute_score (006ec560)`

Callers (listing `call 0x6ec560`, two sites only, and the export agrees):

- `Leaders::strategy_all (006ed430)` at 006ed45b, `compute_score(this, 0)`, for every slot of
  the 8 (`0xe3a390`..`0xe71af0`, stride 0x6eec) with `leader_flags & 3 == 3`, between
  `plan_strategy` and `diplomacy`. `strategy_all` is called once per sim frame from
  `Game::do_frame (00591ef0)` (call 00592448) unless `semaphore` bit 11 (`semaphore.ptr[1] & 8`,
  Game+0x814 bits at +0x820) is set.
- `Game::defeat_all (00592aa0)` at 00592adb, `compute_score(this, 1)` for every slot with
  `leader_flags & 1`; its one caller is `GameDaemon::process_victory (00730ef0)` (call 00730f22),
  the end-of-game sweep.

The gate at the head (006ec563..006ec594): the body runs when **any** of
`game->frame == 0` (Game+0x550), `param_1 != 0`, `semaphore` bit 6 (`byte [Game+0x820] & 0x40`),
or `(who + frame) % 10 == 0` (signed `idiv` by 10, remainder tested; `who` = LeaderData+0x8).
Otherwise it returns with no store. So in a plain game each live leader recomputes on one frame
in ten, phased by slot: slot `w` on frames with `frame ≡ −w (mod 10)`, plus frame 0.

### 0.2 Body order

1. `score_explored = 0` (store `0x1c(%esi)` at 006ec5a0, unconditional once the gate passes).
2. If `leader_flags & 0x1800000` (bit 23 or bit 24): `compute_unit_score`, then clear bit 23
   only (`andl $0xff7fffff,(%esi)`).
3. `compute_build_score` (always), writes `score_buildings`, `score_wonders`.
4. `compute_economy_score` (always), writes `score_economy`.
5. `score_pop = 0` (006ec5ca, unconditional).
6. If `leader_flags & 0x1000000` (bit 24): `compute_unit_upgrades_score`, `compute_research_score`,
   then clear bit 24 (`andl $0xfeffffff,(%esi)`).
7. `score_territory = leaders[who].territory * 1000 / world->land_size` (signed `imul $0x3e8`,
   `cltd; idivl 0x78(%ecx)`, truncation toward zero). `territory` is LeaderData+0x9d8, read
   absolute at `0xe3ad68(%eax)` (`0xe3a390 + 0x9d8`); `land_size` is World+0x78 through the
   pointer at `GameAccess::world` (0x00c06188).
8. If `game->armageddon (Game+0x6e0) >= Game::get_armageddon (00594020)`: zero `score_economy`,
   `score_buildings`, `score_territory`, `score_units`, `score_explored`, `score_combat`,
   `score_wonders`, `score_research`, `score_unit_upgrades`, `score_pop`, `score` and return.
   **`score_units_2` (+0x28) is not zeroed here.**
9. Else `score = score_combat + score_wonders + score_research + score_unit_upgrades +
   score_economy + score_buildings + score_units + score_territory + score_pop + score_explored`
   (plain `add`s, wrapping i32; 006ec66d..006ec68a). **`score_units_2` is not a term.**

`Game::get_armageddon (00594020)`: `armageddon + armageddon_per_nation·num_nations +
armageddon_per_team·num_sides` (all `get_item`, plain integers, `Constants::init (00569a90)`
:2413-2417), ×3 when `info.starting_resources == 7`, ×2 when it is 5, 6, 11 or 12, floored at 100
when it is 8. Every part function repeats this test at its own head and leaves its part at 0
when armageddon has been reached, so the zeroing in step 8 is belt and braces except for
`score_territory`, `score_explored`, `score_pop`, `score_combat` and `score`.

### 0.3 The dirty bits (who sets `leader_flags` bits 23 and 24)

Listing net: every `orl $0x800000|$0x1000000` whose destination is a leader record (`(%reg)`
where the reg was built as `0xe3a390 + slot·0x6eec`, or `0xe3a390(%reg)`):

- bit 23 (`0x800000`): `Unit::close (0060ee50)` at 0060eedb (unit removed, owner byte
  `+0x9 < 8`) and `Unit::init (00612100)` at 00612d83 (unit created, `param_1 < 8`). The many
  other `orl $0x800000, 0x68(%reg)` hits are `UnitData::unit_masks`, not this field.
- bit 24 (`0x1000000`): `Leader::gain_tech (006dcb60)` at 006dcb99, unconditionally at entry.
  (The `orl $0x1000000, 0x4(%eax)` hits at 0051a33f, 00525128, 00891040.. are another
  record's `+0x4`; not a `Leader*`.)
- The only clears are the two `andl` in `compute_score` (listing grep for `andl
  $0xff7fffff|$0xfeffffff` on a leader record finds those two and nothing else).

Consequence a builder must keep: `score_units`/`score_units_2` are recomputed only on a gated
frame following a unit creation, a unit removal or a tech gain; `score_unit_upgrades` and
`score_research` only on a gated frame following a tech gain. Both include *queued/researching*
items in their inputs, but queuing alone does not dirty them, so the parts can sit stale.
A forced call (`param_1 = 1`) bypasses the cadence gate but **not** the dirty bits.

### 0.4 `TypeData::get_score_value (006673b0)` — the common value

Vtable slot `+0x7c` of every `Type` vtable. PE bytes: `UnitType`'s `Type` vtable 0x00b41fd4,
`BuildType`'s 0x00b42b94, `ObjectType`'s 0x00b43aac and `TypeData`'s 0x00b43e48 all hold
0x006673b0 at `+0x7c`; the map has one `get_score_value` symbol. With `S = Σ costs[0..6]`
(TypeData+0x18, six ints) of type `t'`:

- `t' = t`, except when `param_1 == 0` and `t ∈ {MILITIA 0x42, MINUTEMAN 0x43, PARTISAN 0x44}`,
  where `t' = BASE_UNITTYPES 0x32` (the costs of type 0x32 stand in).
- unit type (`0x31 < t < 0x19e`, or a non-inlined `is_unit_type` override true):
  `param_1 == 0`: `unit_cost_factor · S`. `param_1 != 0`:
  `x = research_premium · (unit_cost_factor · S)`; `x = trunc(x / 256) ·
  UnitTypeData::research_premium_cost(t')`; return `trunc(x / 256)` (the `(v + (v>>31 & 0xff)) >> 8`
  idiom is division toward zero). `research_premium` is 8.8 (`get_fraction(…,0x100)`,
  `Constants::init`:1161); `research_premium_cost` (UnitTypeData+0x2e0) is 8.8
  (`String::fraction(…,0x100)`, `UnitType::init (0061ab50)`:612).
- build type (`0x19d < t < 0x21f`): `build_cost_factor · S`.
- tech type (`0x21f < t < 0x275`) and the fallthrough: `tech_cost_factor · S`.
- spell type (`0x274 < t < 0x2ac`): `spell_cost_factor · S`.
- `unit_cost_factor`, `build_cost_factor`, `tech_cost_factor`, `spell_cost_factor` are plain
  `get_item` integers (`Constants::init`:1121-1128).

### 0.5 Writers common to every leaf

- `Leader::reset_score (006e37f0)` zeroes all twelve (and the `units_built`.. counters at
  `+0x808`..). Callers: `Leader::init (006e3930)`:140 and `Setup::build_game (005ac190)`:731 (every
  slot with `leader_flags & 1`, after the `setup.cpp` checksum line). Both setup-time.
- `LeaderData::walk_data (006d6750)` copies `[who, +0x692a)` as one range, so on a load
  (`DataWalk::input != 0`) every leaf is restored from the save. Not on a plain game's frame path;
  the same walk feeds the per-frame leaders checksum (crate `checksum/leaders.rs` lists all twelve
  as leaves at `LEADER_FIELDS` lines 41-52), so **every leaf is checksum-visible**.
- Export net: `grep -- '->score…'` finds only readers (§ readers below); `field_0x18`..`field_0x44`
  on a `Leader`/`LeaderData`/`LeaderOut`-typed pointer (a script over every function that
  declares one) finds only `compute_*`, `reset_score` and the unrelated `LeaderOption` class.
  Listing net for the absolute form (`0xe3a3a8`..`0xe3a3d7`): stores only to `0xe3a3d4`
  (`score_combat`, § 12); every `0xe3a3a8` hit is a read of `score`.

---

## 1. `score` (+0x18)

1. **Writers.** `Leader::compute_score (006ec560)`: store `0x18(%esi)` at 006ec68a (the sum) and at
   006ec660 (armageddon zero). `reset_score`, `walk_data` load (§0.5). Base `esi` = `Leader*`
   `this` (§0). No other store found by either net.
2. **Event / step.** Recomputed (not stepped) on a gated frame (§0.1): `score = score_explored +
   score_territory + score_units + score_buildings + score_economy + score_pop +
   score_unit_upgrades + score_research + score_wonders + score_combat`, i32 adds. Not
   `score_units_2`. Where armageddon is reached (`game->armageddon >= get_armageddon()`), 0.
   On a non-gated frame the value stands even if a part (e.g. `score_combat`) moved since.
3. **Reset.** `reset_score` at setup; the armageddon branch.
4. **Readers.** `LeaderData::get_team_score (006d6520)` (own score, or Σ over team when
   `semaphore` bit 7 is set (`(char)ptr[0] < 0`)); `LeaderData::get_mvp_score (006d5de0)`
   (`score == 0 → …`, else `3·combat − economy/2 − buildings/2 + territory + score`);
   `Leader::diplomacy (006bc950)` (counts leaders with a higher score, finds the top scorer,
   compares to `get_team_score` — an AI decision input); `Army::find_target (006f69b0)`:372
   (`target_owner.score·4/3 < my score` gates a siege-army arm — AI decision input);
   `GameDaemon::process_victory (00730ef0)`:636 (lowest/tied score among live leaders);
   `ScenarioFuncSet::score (009e8fa0)`, `at_least_points (009ed450)`, `less_than_points (009edc20)`
   (triggers); `StatWin::setup_normal_entries (007ecc40)`, `AchieveChartPane::on_redraw
   (007f6ff0)`, `LeaderData::log_data (006e5110)` (the `LEADERDATA` logger), `Options::describe`.
5. **Crate.** No `compute_score`: `crates/sim/src/ai/ai_drive.rs` `Sim::strategy_all` (line 73)
   has the comment "`compute_score(0)` — not modelled" between `plan_strategy` and `diplomacy`
   (line 87); that is where the call goes, with the `(who + frame) % 10` gate. Readers already
   carry seams: `leaders/diplomacy.rs` `Sim::diplomacy` line 155 (the score count / top scorer),
   `ai/army/target_score.rs` line 457 (the `score × 4/3` clause). Note the crate's
   `strategy_all` skips humans via `continue` before the `compute_score` slot — the original
   calls `compute_score` for every `leader_flags & 3 == 3` slot, human included, so the call must
   sit before that `continue`. The end-of-game `defeat_all` forced call has no crate site.
6. **Plain game?** Yes: `strategy_all` runs every frame of a skirmish (semaphore bit 11 clear).
   `defeat_all` at game end is also plain-game. The `ScenarioFuncSet` readers are the trigger
   layer.
7. **Confidence.** High: listing read whole for `compute_score`; both callers read. Not read:
   what sets `semaphore` bit 6 (the "always recompute" bit) and bit 7; whether a parts change
   between gated frames is observable to a reader that matters (the readers above read only
   `score`, which is stale by design between gated frames).

## 2. `score_explored` (+0x1c)

1. **Writers.** `compute_score`: `movl $0,0x1c(%esi)` at 006ec5a0 (every gated frame) and 006ec636
   (armageddon). `Leader::compute_explore_score (006bc5e0)` also stores 0 but is dead: no call in
   the listing, no pointer to it in the PE bytes. `reset_score`, `walk_data` load.
2. **Step.** None. It is 0 on every gated frame; the part is stubbed out in this build.
3. **Reset.** As above.
4. **Readers.** Only as a term of `score`.
5. **Crate.** Nothing to add beyond writing 0; `checksum/leaders.rs` line 42 holds the leaf.
6. **Plain game?** Yes (constant 0).
7. **Confidence.** High.

## 3. `score_territory` (+0x20)

1. **Writers.** `compute_score`: `movl %edi,0x20(%esi)` at 006ec604, and 0 at 006ec628
   (armageddon). `reset_score`, `walk_data` load.
2. **Step.** On each gated frame, `territory · 1000 / land_size`, i32 signed, truncating
   (`imul $0x3e8`; `cltd; idivl`). `territory` = `LeaderData+0x9d8` of slot `who` (read through the
   global array, `0xe3ad68 + who·0x6eec`); `land_size` = `World+0x78`. Units: territory in the
   map's own tiles over the land tile count, ×1000 — permille of the land. Computed every gated
   frame regardless of dirty bits; this part has **no** armageddon test of its own, only the
   zeroing in step 8.
3. **Reset.** As above.
4. **Readers.** `score` (term); `get_mvp_score`, `AchieveChartPane::on_redraw` (UI).
5. **Crate.** Inputs exist: `economy.rs` `Holdings::territory` and `Holdings::land_size`
   (line 555; `economy.rs:1142` already divides `territory · rate · RATE_SCALE / land_size`). Add in
   the new `compute_score` beside `ai_drive.rs:87`.
6. **Plain game?** Yes.
7. **Confidence.** High on the formula. Not read: when `land_size` is set, and whether it can be 0
   (an `idiv` by 0 would fault in the original; the crate guards `<= 0` elsewhere).

## 4. `score_units` (+0x24) and 5. `score_units_2` (+0x28)

One function writes both, so one section.

1. **Writers.** `Leader::compute_unit_score (006bc500)`, `this` in `edi` (`mov %ecx,%edi`):
   `movl $0,0x24(%edi)` 006bc50c, `movl $0,0x28(%edi)` 006bc513, `addl %ecx,0x24(%edi)` 006bc5b4,
   `addl %ecx,0x28(%edi)` 006bc5c6. Its one caller is `compute_score` (call 006ec5ab), so the base
   is the `Leader*`. `compute_score` also zeroes `score_units` (not `score_units_2`) at 006ec62f
   under armageddon. `reset_score`, `walk_data` load.
2. **Step.** Runs on a gated frame (§0.1) **only if** `leader_flags & 0x1800000` (§0.3: a unit
   created or closed by this leader, or a tech gained, since the last run). Both start at 0; if
   armageddon is reached they stay 0. Else for every unit type `t` in `[0x32, 0x192)`:
   - `n = num_units[t − 0x32] + num_queued[t]` (u16 each; LeaderData+0x5762 and +0x5a22, read
     absolute `0xe3fa8e`/`0xe3fdb2 + 2·(who·0x3776 + t)`), skip if 0;
   - `sv = get_score_value(t, 0)` (§0.4, `unit_cost_factor · Σcosts`, militia family priced as
     type 0x32);
   - `sup = support_cost[0] + support_cost[1]` (ObjectTypeData+0x270/+0x274 of
     `unittypes[t]`, `GameAccess::unittypes` at 0x00c061fc);
   - `v = trunc( (sv·n + trunc((n−1)·sup·n / 2)) / 5 )` — the price of `n` units under the
     per-unit support ramp, a fifth of it;
   - `score_units += v`; and if `attack` (ObjectTypeData+0x1e8 of `unittypes[t]`) `!= 0`,
     `score_units_2 += v`. So `score_units_2` is the armed subset.
3. **Reset.** `reset_score`; `score_units` also by the armageddon branch; `score_units_2` keeps its
   last value through armageddon (the part function would zero it on its next dirty run, since
   it stores 0 before the armageddon test).
4. **Readers.** `score_units` is a term of `score`. `score_units_2`: no reader found in the export
   by name (`grep -- '->score_units_2'` hits nothing) and it is not a term of `score`; a
   register-relative `0x28(%reg)` read cannot be excluded by grep.
5. **Crate.** Inputs exist: per-type unit counts (`checksum/leaders.rs:1178`, `m.by_type`) and
   queued counts (`m.queued_by_type`, :1183); `tables.unit_types` carries `attack` and the support
   costs; `core/tuning.rs:580` `unit_cost_factor`. The dirty bit needs a hook at unit creation
   (`units/store.rs:48` `add_unit`, `setup.rs:59`) and removal (`combat/fight/delivery.rs:1245`
   `close_unit`), and at tech gain (`tech/gain.rs:189` `gain_tech`). No crate code computes this
   part today.
6. **Plain game?** Yes.
7. **Confidence.** High on the arithmetic (listing read whole). Not read: whether `Unit::init`'s
   leader pointer (`-0x14(%ebp)` at 00612d83) is always `leaders[owner]` (assumed from the `< 8`
   test beside it); whether `num_queued` for a unit type counts only queue entries.

## 6. `score_buildings` (+0x2c) and 11. `score_wonders` (+0x40)

One function writes both.

1. **Writers.** `Leader::compute_build_score (006bc3f0)`, `this` in `esi`: `movl $0,0x2c(%esi)`
   006bc3f7, `movl $0,0x40(%esi)` 006bc3fe, `addl %eax,0x2c(%esi)` 006bc4c7, `addl %eax,0x40(%esi)`
   006bc4ec. One caller, `compute_score` (006ec5b8). Armageddon zero in `compute_score` at
   006ec621 (+0x2c) and 006ec644 (+0x40). `reset_score`, `walk_data` load.
2. **Step.** Every gated frame (no dirty bit). Both start at 0; 0 under armageddon. Else for every
   building type `t` in `[0x19e, 0x21f)`:
   - `n = num_buildings[t − 0x19e] + num_queued[t]` (LeaderData+0x555e, +0x5a22), skip if 0;
   - `sv = get_score_value(t, 0)` = `build_cost_factor · Σcosts`;
   - `sup = support_cost[0] + support_cost[1]` of `buildtypes[t]` (`GameAccess::buildtypes`
     0x00c061f4);
   - `v = sv·n + trunc((n−1)·(sup·build_support_factor)·n / 2)` (`build_support_factor` =
     Constants+0x37c, plain `get_item`);
   - wonder test: vtable slot `+0x1c` compared with `TypeData::is_wonder_type` (0x00470780; the
     PE shows every `Type` vtable holds that address, so the inline arm always applies):
     `0x20e ≤ type < 0x21f` → `score_wonders += trunc(v / 3)`; else `score_buildings +=
     trunc(v / 5)`.
3. **Reset.** `reset_score`; armageddon branch.
4. **Readers.** Terms of `score`; `score_buildings` also in `get_mvp_score` and
   `AchieveChartPane` (`− buildings/2`). `score_wonders`: `score` only.
5. **Crate.** `num_buildings_of` / `num_sites_of` (the crate fills `num_queued[0x19e+rec]` with
   placed sites, `checksum/leaders.rs:1188`); `core/tuning.rs:590` `build_support_factor`. No
   crate code computes it.
6. **Plain game?** Yes (wonders are buildable in a skirmish).
7. **Confidence.** High (listing read whole).

## 7. `score_economy` (+0x30)

1. **Writers.** `Leader::compute_economy_score (006bc270)`, `this` in `ebx`: `movl $0,0x30(%ebx)`
   006bc277, `addl %eax,0x30(%ebx)` 006bc2f0, 006bc324, 006bc34d. One caller, `compute_score`
   (006ec5bf). Armageddon zero 006ec61a. `reset_score`, `walk_data` load.
2. **Step.** Every gated frame. 0, then (unless armageddon) for each good `r` in 0..6 with
   `LeaderData::type_avail(r, 1) != 0` (006e33a0; the good's prerequisites and eligibility):
   - `a = trunc( ((bucket[r] ^ 0x8221) − game->starting[r]) / 20 )`, clamped to `[0, 100]`.
     `bucket` is `LeaderDataEncrypt+0x0` through `data_encrypted` (LeaderData+0x6eb8), XOR-masked
     with 0x8221; `starting` is `Game+0x600` (int[6]). So: stock above the starting grant, a point
     per 20, at most 100.
   - `b = min( trunc((income[r] ^ 0x90236) / 80), 200 )` — `income` at `LeaderDataEncrypt+0x94`,
     masked with 0x90236. **No lower clamp**: a negative income gives a negative `b`.
   - `score_economy += a + b`.
   Then `score_economy += 25 · BitMask<44>::count(rare)` (`rare` LeaderData+0x6d98; `count` at
   0x0047cd40, the number of rare resources held).
   Divisions are `imul 0x66666667; sar 3|5; add sign` = truncation toward zero.
3. **Reset.** `reset_score`; armageddon branch.
4. **Readers.** `score` term; `get_mvp_score`/`AchieveChartPane` (`− economy/2`).
5. **Crate.** All inputs exist unmasked: `economy.rs` `Ledger::bucket` (681), `Ledger::income`
   (696), `Holdings::starting` (616), `Holdings::rare` (725, a `u64` popcount). Note the crate's
   `income` is documented as "the net rate the interface shows"; whether its units match the
   original's `income` word is for the builder to check against a dump (the leaders checksum walks
   `income` in `LeaderDataEncrypt::walk_data`). `type_avail` for a good is in the crate's AI
   (`type_avail` is used by `queued_units`, `docs/AI.md` §2.16).
6. **Plain game?** Yes.
7. **Confidence.** High on arithmetic. Not read: `type_eligible` for goods in full.

## 8. `score_pop` (+0x34)

1. **Writers.** `compute_score`: `movl $0,0x34(%esi)` at 006ec5ca (every gated frame) and
   006ec659 (armageddon). `Leader::compute_pop_score (006bc190)` stores 0 too but is dead (no call,
   no PE pointer). `reset_score`, `walk_data` load.
2. **Step.** None; always 0 in this build.
3–7. As `score_explored`: term of `score` only; nothing to build but the 0; plain game; high.

## 9. `score_unit_upgrades` (+0x38)

1. **Writers.** `Leader::compute_unit_upgrades_score (006bc1a0)`, `this` in `edi`:
   `movl $0,0x38(%edi)` 006bc1a3, `addl %eax,0x38(%edi)` 006bc230. One caller (`compute_score`,
   006ec5d5). Armageddon zero 006ec652. `reset_score`, `walk_data` load.
2. **Step.** On a gated frame **only if** `leader_flags & 0x1000000` (a tech gained since the
   last run, §0.3). 0, then unless armageddon, for each unit type `t` in `[0x32, 0x192)`:
   counted if (`LeaderData::has_tech(t)` (006e0c80) or `LeaderData::researching(t, −1, 0, 1)`
   (006db510: `t` sits in a queue of one of my buildings, exact match)) and `tech_at_start` bit
   `t` is clear (bits at LeaderData+0x6c8c = `tech_at_start`+0xc past the BitMask header; the
   listing's `testb %bl,0x6c8c(%eax,%edi)` with `bl` rotating from `1 rol (t&7)`) and
   `unit_flags & 0x80` (UnitTypeData+0x2b4) clear. Each adds `trunc(get_score_value(t, 1) / 5)`;
   `get_score_value(t,1)` = `trunc(trunc(research_premium·unit_cost_factor·Σcosts / 256) ·
   research_premium_cost / 256)` (§0.4; no militia substitution with `param_1 = 1`).
3. **Reset.** `reset_score`; armageddon branch.
4. **Readers.** `score` term only.
5. **Crate.** `tech.rs:722` `tech_at_start: Vec<bool>`; researched unit lines in the crate's
   tech state; queued techs `t.queued` (`checksum/leaders.rs:1193`); `core/tuning.rs:608`
   `research_premium`; `research_premium_cost` from the unit tables. Hook the dirty bit in
   `tech/gain.rs:189` `gain_tech` (also `setup.rs:238`, `tech.rs:1896`).
6. **Plain game?** Yes.
7. **Confidence.** High on arithmetic; medium on `researching`'s full predicate (read once, the
   `param_4 = 1` arm is an exact-type match). Not read: `has_tech` body.

## 10. `score_research` (+0x3c)

1. **Writers.** `Leader::compute_research_score (006bc360)`, `this` in `edi`: `movl $0,0x3c(%edi)`
   006bc363 and the `addl` into `0x3c(%edi)` after 006bc3d6. One caller (`compute_score`,
   006ec5dc). Armageddon zero 006ec64b. `reset_score`, `walk_data` load.
2. **Step.** Same gate as § 9 (bit 24). 0, then unless armageddon, for each `t` in
   `[0x220, 0x275)` (`BASE_AGETYPES` = `BASE_TECHTYPES` = 0x220 up to `BASE_SPELLTYPES`; ages are
   in the range): counted if (`has_tech(t)` or `researching(t, −1, 0, 0)`) and `tech_at_start`
   bit `t` clear. Each adds `trunc(get_score_value(t, 0) / 5)` = `trunc(tech_cost_factor ·
   Σcosts / 5)`. No `unit_flags` test here.
3–4. As § 9; term of `score` only.
5. **Crate.** As § 9; the tech tree and `tuning.tech_cost_factor`.
6. **Plain game?** Yes.
7. **Confidence.** High on arithmetic. `researching(…, 0)` adds an upgrade-line equivalence arm
   that needs both types to be unit types, so it is inert for techs (read, not tested).

## 12. `score_combat` (+0x44)

Not computed by `compute_score` (`Leader::compute_combat_score (006bc250)` is an empty `ret`,
never called, no PE pointer to it). Stepped at kills, in the killer's slot, always through the
absolute form `0xe3a3d4(%reg)` where `%reg = slot·0x6eec` — the base is unambiguous.

### 12.1 The value: `get_kill_value` (vtable slot `+0x80`)

PE bytes of the `Type` vtables: `UnitType` (0x00b41fd4+0x80) → `UnitTypeData::get_kill_value
(0061d3d0)`; `BuildType` (0x00b42b94+0x80) → `BuildTypeData::get_kill_value (00639790)`;
`ObjectType`/`TypeData` → `TypeData::get_kill_value (00663a00)`. With `p` the victim's player
(the argument):

- unit: `get_score_value(t, 0) + (support_cost[0] + support_cost[1]) · (num_units[t−0x32] +
  num_queued[t])` of player `p` (listing 0061d3d0..: the export's body for this one is garbled;
  read from the listing — `imul $0x3776,p; add t; movzwl 0x56fe/0x5a22(leaders,idx,2)`); if
  `p < 0`, `get_score_value(t, 0)` alone. That is the victim's *current marginal* unit price.
- building: `get_score_value(t, 0) + (support_cost[0]+support_cost[1]) · build_support_factor ·
  (num_buildings[t−0x19e] + num_queued[t])` of player `p`; `p < 0` → the score value alone.
- anything else: `get_score_value(t, 0)`.

Every site adds or subtracts `trunc(value / 5)` (`imul 0x66666667; sar 1; add sign`).

### 12.2 Writers

1. **`Object::take_damage (00652020)`** — a unit killed by a known killer. Stores
   `subl %eax,0xe3a3d4(%ecx)` 0065313e, `movl $0,0xe3a3d4(%ecx)` 00653146,
   `addl %eax,0xe3a3d4(%ecx)` 0065316d; `%ecx = killer_slot·0x6eec` (`-0x4(%ebp)` set at 00653043
   from `ebx`, the killer's player argument). Predicate, in order (listing 00652f46..00653173):
   - the victim is not a building (vslot `+0x1c` false; a building takes `buildings_lost`
     `+0x81c += 1` and leaves);
   - victim `is_captain` (vslot `+0xe8`; inline arm `UnitData::is_captain (0046ceb0)`:
     `o_up` (UnitData+0x8e) bit 15 set, i.e. no `o_up` link — the head of its squad);
   - `UnitData::curr_uber_size (0060a760)(0) == 1` (the last figure);
   - `unit_masks & 1` (UnitData+0x68) clear (the crate calls this bit `decoy`);
   - then `units_lost` (`+0x810`) `+= 1` on the victim; then the killer's object id argument
     (`0x20(%ebp)`) must be `>= 0`, else leave;
   - victim `+0xa58 += 1` (u16), killer `+0xa5a += 1` (u16) and killer `units_killed`
     (`+0x80c`) `+= 1` (00653026..00653046);
   - the victim object passes vslot `+0x18`; if its unit type has `unit_flags & 0x10`, the victim
     is redirected to the object named by its `+0x3e` (player) / `+0x28` (id), which must also
     pass vslot `+0x18`;
   - `value = get_kill_value(type of victim, victim player)`; if
     `LeaderData::is_ally (006edb50)(killer, victim player)`: `score_combat −= trunc(value/5)`,
     floored at 0 (`jns` after the `sub`, else store 0); else `score_combat += trunc(value/5)`.
     (`is_ally` of oneself is true, so killing one's own unit costs points.)
2. **`Object::do_damage (0064a480)`** — a building destroyed by an attacking object. Stores
   `subl` 0064bb02, `movl $0` 0064bb1b (after `cmpl $0` / `jge`), `addl` 0064bb50, base
   `attacker_owner·0x6eec`. Predicate: the target's `take_damage` (vslot `+0x16c`, which is
   `Object::take_damage` in every object vtable) returned non-zero (destroyed) and the target's
   vslot `+0x1c` is true (a building); then attacker `buildings_razed` (`+0x818`) `+= 1`; then
   `value = get_kill_value(target type, param_2 = target player)`, and the same ally rule:
   ally → `−= trunc(value/5)` floored at 0, else `+= trunc(value/5)`. The value is computed after
   `take_damage` returned.
3. **`SpellType::cast_sabotage (006720e0)`**, two arms, caster `param_2`, victim `param_4`, no
   ally test and no floor:
   - unit arm, `addl %eax,0xe3a3d4(%esi)` 00672252: target not a building (vslot `+0x1c`), a unit
     (vslot `+0x18`), `is_captain`, `curr_uber_size(0) == 1`, `unit_masks & 1` clear → victim
     `+0xa58 += 1`, caster `+0xa5a += 1`, caster `units_killed += 1`, `score_combat +=
     trunc(get_kill_value(t, victim)/5)`.
   - building arm, `addl %eax,0xe3a3d4(%esi)` 006727a0: the building's `take_damage(max(hits/2,
     floor), …, attacker −1, …)` returned non-zero → caster `buildings_razed += 1`, `score_combat
     += trunc(get_kill_value(t, victim)/5)`. (The `take_damage` there passes no attacker, so §1's
     arm does not also credit.)
4. **`SpellType::cast_counterintel (00673740)`**, `addl %eax,0xe3a3d4(%edi)` 0067388e, caster
   slot in `edi`: target (climbed to its captain via vslot `+0xe4` if not a captain) owned by a
   player other than the caster and not mutually allied (`diplos[...] == 2` both ways skips),
   a unit (vslot `+0x18`), `is_captain`, `curr_uber_size(0) == 1`, `unit_masks & 1` clear →
   caster `units_killed += 1`, `score_combat += trunc(get_kill_value(t, target player)/5)`, then
   the target dies (vslot `+0x158`).
5. `compute_score`'s armageddon zero (006ec63d); `reset_score`; `walk_data` load.

### 12.3 The rest

3. **Reset.** Only `reset_score` (setup) and the armageddon zeroing; it otherwise accumulates for
   the whole game (it is the only part that is a running total rather than a recount).
4. **Readers.** `score` (term, picked up on the next gated frame); `get_mvp_score` and
   `AchieveChartPane::on_redraw` (`3 · score_combat`).
5. **Crate.** The kill arms exist: `combat/fight/delivery.rs` — `take_damage_typed` (line 469;
   the unit death arm at ~537-550 calls `unit_plunder` and `inca_refund`, whose gates are the same
   captain / `curr_uber_size == 1` / not-decoy tests; the credit goes there, *before*
   `close_count`, since in the original the credit precedes the death); `do_damage` (line 49; the
   building-killed arm at ~213, beside `plunder_kill`); `cast_sniper_unit` (line ~807) and
   `cast_sabotage_building` (line 884) carry `SEAM: … score +0x44 += cost / 5`;
   `spells/cast.rs:1022-1028` `cast_counterintel` carries the same SEAM. `get_kill_value` itself
   has no crate function (grep finds none); `get_score_value` has a likely relative in
   `economy/pricing.rs` but not the score form.
6. **Plain game?** Yes for all four writers: combat kills of units and buildings, and the spy
   abilities (sabotage, sniper, counter-intelligence) are skirmish features.
7. **Confidence.** High on the stores, the base, the `/5`, the ally sign and the floor (listing).
   Medium on the take_damage predicate's meaning (the `+0x3e`/`+0x28` redirect for
   `unit_flags & 0x10` was read in the listing, not interpreted). Not established: whether the
   victim's own count is still included in `num_units`/`num_buildings` when `get_kill_value` runs
   (take_damage's credit precedes `die`, so likely yes for units; do_damage's runs after the
   target's `take_damage`, so likely no for buildings — not verified); the exact meaning of
   `+0xa58`/`+0xa5a`.

---

## What I did not read, overall

- What sets `semaphore` bits 6 (force recompute every frame) and 7 (team scores); both are
  probably lobby/scenario settings.
- `LeaderData::has_tech`, `type_eligible`, and `BitMask<44>::count` bodies (names taken as
  given; `count` is a popcount by its name and use).
- Whether `leader_flags` bit 24 is set before frame 0 (if setup grants techs through
  `gain_tech`, it is; frame 0 always passes the cadence gate).
- No diff backs any of this: the crate holds no score leaf value, and I did not look for a dump.
  A `LEADERDATA` logger line carries `score` (`LeaderData::log_data (006e5110)`:209), and the
  leaders checksum walks all twelve, so a value diff is available once built.

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `score` +0x18 | `Leader::compute_score (006ec560)` | recount every 10th frame by slot (`(who+frame)%10==0`, frame 0, forced, sem bit 6): Σ of the ten parts except `score_units_2`; 0 at armageddon | `Leader::reset_score (006e37f0)`, armageddon; `walk_data` load | none; `ai/ai_drive.rs` `strategy_all` :87 "not modelled" (call must precede the human `continue`) | yes | high |
| `score_explored` +0x1c | `compute_score` (store 0); `compute_explore_score (006bc5e0)` dead | always 0 | as above | write 0 | yes | high |
| `score_territory` +0x20 | `compute_score` | `territory·1000/land_size`, trunc, every gated frame | as above | inputs in `economy.rs` `Holdings` | yes | high |
| `score_units` +0x24 | `Leader::compute_unit_score (006bc500)`; `compute_score` (armageddon 0) | gated + dirty (bit 23/24): Σ_t trunc((sv·n + trunc((n−1)·sup·n/2))/5), n = units + queued | as above | none; hooks `units/store.rs:48`, `delivery.rs:1245`, `tech/gain.rs:189` | yes | high |
| `score_units_2` +0x28 | `compute_unit_score` only | same terms, only types with `attack != 0`; **not in `score`, not zeroed at armageddon** | `reset_score`, part's own 0 | none | yes | high |
| `score_buildings` +0x2c | `Leader::compute_build_score (006bc3f0)`; `compute_score` | gated: Σ non-wonder trunc((sv·n + trunc((n−1)·sup·bsf·n/2))/5), n = buildings + sites | as above | none; `num_buildings_of`/`num_sites_of` | yes | high |
| `score_economy` +0x30 | `Leader::compute_economy_score (006bc270)`; `compute_score` | gated: Σ_avail goods clamp((bucket−starting)/20,0,100) + min(income/80,200); + 25·#rares | as above | none; `economy.rs` `Ledger`/`Holdings` | yes | high |
| `score_pop` +0x34 | `compute_score` (store 0); `compute_pop_score (006bc190)` dead | always 0 | as above | write 0 | yes | high |
| `score_unit_upgrades` +0x38 | `Leader::compute_unit_upgrades_score (006bc1a0)`; `compute_score` | gated + bit 24: Σ unit types held or queued, not at start, not `unit_flags&0x80`: trunc(get_score_value(t,1)/5) | as above | none; `tech.rs` `tech_at_start`, `gain_tech` hook | yes | high (medium on `researching`) |
| `score_research` +0x3c | `Leader::compute_research_score (006bc360)`; `compute_score` | gated + bit 24: Σ t∈[0x220,0x275) held or queued, not at start: trunc(tech_cost_factor·Σcosts/5) | as above | none | yes | high |
| `score_wonders` +0x40 | `compute_build_score`; `compute_score` | gated: Σ wonders (0x20e..0x21e) trunc(v/3) | as above | none | yes | high |
| `score_combat` +0x44 | `Object::take_damage (00652020)`, `Object::do_damage (0064a480)`, `SpellType::cast_sabotage (006720e0)` ×2, `SpellType::cast_counterintel (00673740)`; `compute_score` (armageddon 0) | per kill: ±trunc(get_kill_value(victim type, victim)/5), minus and floored at 0 when the victim is an ally (combat arms only; spells always +) | `reset_score`, armageddon | SEAMs in `combat/fight/delivery.rs` (sniper, sabotage building) and `spells/cast.rs` counterintel; unit/building kill arms in `delivery.rs` `take_damage_typed`/`do_damage` | yes | high on stores; medium on predicate meanings |
