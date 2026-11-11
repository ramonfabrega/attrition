# Item 4152, family 18 (`score`) — first reading

- Family: 18, slug `score`.
- Leaves (all `int`, `LeaderData` offsets): `score` (+0x18), `score_explored` (+0x1c),
  `score_territory` (+0x20), `score_units` (+0x24), `score_units_2` (+0x28),
  `score_buildings` (+0x2c), `score_economy` (+0x30), `score_pop` (+0x34),
  `score_unit_upgrades` (+0x38), `score_research` (+0x3c), `score_wonders` (+0x40),
  `score_combat` (+0x44).
- Reader: Opus 5.5 (claude-opus-5-5), first reading, 2026-11-11.
- Sources: the decompile export (`~/ghidra-projects/decomp`), the listing
  (`llvm-objdump` of `riseofnations.exe`), `types.txt`.

Layout check: `Leader` (0x6eec) lists no fields of its own and `LeaderData`
(0x6ee4) sits at offset 0 of it — `Leader::compute_score (006ec560)` reads
`this->field_0x8` as `who` and `*(uint*)this` as `leader_flags`, and the
listing indexes `leaders` at `0xe3a390 + who·0x6eec`. So a `Leader*`'s
`field_0xNN` is `LeaderData + 0xNN` here, with no prefix to subtract. The
twelve leaves' absolute addresses for slot 0 are `0xe3a3a8`…`0xe3a3d4`.

## 0. The shared machinery (read once, cited by every leaf)

### 0.1 Who runs it, and on which frames

- **`Leaders::strategy_all (006ed430)`** walks the eight slots
  (`0xe3a390`…`0xe71af0`) and, for each with `leader_flags & 3 == 3`, calls
  `check_explore`, `plan_strategy`, **`compute_score(this, 0)`**, `diplomacy`,
  in that order. Its one call is `Game::do_frame (00591ef0):267` (listing
  `592448`), gated on `semaphore[1] & 8 == 0` (the AI-off / editor bit,
  `docs/ARMY.md`), **before** `game->frame` is incremented at `:293`. So the
  `frame` `compute_score` tests is the frame number the `do_frame` began with.
  Humans pass the `& 3 == 3` gate too: the score is computed for every live
  leader, human or computer.
- **`Leader::compute_score (006ec560)`, the gate.** It recomputes only when
  one of: `game->frame == 0`; the argument is non-zero (forced);
  `semaphore[0] & 0x40`; or `(who + frame) % 10 == 0`. Otherwise it returns
  and every leaf keeps its value. So in a running game each slot recomputes
  **every 10th frame, phased by its slot number** (slot `w` on frames
  `≡ −w mod 10`), plus frame 0 for everyone.
  - `semaphore[0] & 0x40` is set by `Game::check_victory (005926b0):114`
    (listing `59286e`, the game-over latch: `check_victory` returns at its
    head once it is set), and by `RecordGame::read_package (00952d90)`
    (`952f75`), `TurnControl::toggle_pause (00957ab0)` (`957b9a`) and
    `MPDropWin::on_button_clicked (0095ab60)` (`95acd6`). After a victory the
    score recomputes on every frame `strategy_all` still runs. The three
    others were not read past the store.
- **`Game::defeat_all (00592aa0)`** calls `compute_score(this, 1)` (forced)
  for every slot with `leader_flags & 1`. Its one caller is
  `GameDaemon::process_victory (00730ef0):75` (listing `730f22`), on the arm
  `get_armageddon() <= game->armageddon`, i.e. after Armageddon; there the
  forced recompute takes the zeroing arm (§0.3) — so in practice this caller
  only ever zeroes.

### 0.2 The body of `compute_score`, in order

On a recompute frame (`this` is the leader):

1. `score_explored = 0` (unconditional).
2. If `leader_flags & 0x1800000` (either dirty bit 23 or 24):
   `compute_unit_score` (writes `score_units`, `score_units_2`), then clear
   bit 23 (`& 0xff7fffff`).
3. `compute_build_score` (writes `score_buildings`, `score_wonders`) —
   unconditional.
4. `compute_economy_score` (writes `score_economy`) — unconditional.
5. `score_pop = 0` (unconditional).
6. If `leader_flags & 0x1000000` (bit 24): `compute_unit_upgrades_score`
   (writes `score_unit_upgrades`), `compute_research_score` (writes
   `score_research`), then clear bit 24 (`& 0xfeffffff`).
7. `score_territory = leaders[who].territory × 1000 / world->land_size`
   (`LeaderData+0x9d8` × 1000 / `WorldData+0x78`, signed integer division,
   truncating toward zero) — unconditional.
8. If `game->armageddon >= Game::get_armageddon()`: zero `score_economy`,
   `score_buildings`, `score_territory`, `score_units`, `score_explored`,
   `score_combat`, `score_wonders`, `score_research`, `score_unit_upgrades`,
   `score_pop` and `score`, and **return**. **`score_units_2` (+0x28) is not
   in this list.**
9. Else `score = score_combat + score_wonders + score_research +
   score_unit_upgrades + score_economy + score_buildings + score_units +
   score_territory + score_pop + score_explored`. **`score_units_2` is not
   summed** (it is a subset of `score_units`, §4). Plain 32-bit wrapping
   adds.

`compute_pop_score (006bc190)` and `compute_explore_score (006bc5e0)` each
store 0 to their leaf and have **no caller** in the listing (`calll 0x6bc190`
and `calll 0x6bc5e0`: zero hits); `compute_combat_score (006bc250)` and
`compute_wonder_score (006bc260)` are empty `ret`s with no caller either.

### 0.3 Armageddon gate

`Game::get_armageddon (00594020)` = `ARMAGEDDON + ARMAGEDDON_PER_TEAM ×
game->num_sides + ARMAGEDDON_PER_NATION × game->num_nations` (Constants
fields), ×3 when `starting_resources == 7`, ×2 when it is 5, 6, 11 or 12,
floored at 100 when it is 8. `game->armageddon` (`Game+0x6e0`) is the nuke
counter. Each `compute_*_score` also opens with "zero my leaf(s); if
`game->armageddon >= get_armageddon()` return". In a skirmish without nuclear
weapons fired the gate is open. The crate has no `get_armageddon`
(`crates/sim/src/combat/nuke.rs:199`, `crates/sim/src/air.rs:1666` SEAMs).

### 0.4 The two dirty bits

- **Bit 23 (`0x800000`)**, set by `Unit::init (00612100):578` (listing
  `612d83`, `orl $0x800000,(%ebx)` on the owner's `leader_flags`, when the
  owner `< 8`) and by `Unit::close (0060ee50):79` (listing `60eedb`, when the
  unit's owner `< 8`). A unit created or closed.
- **Bit 24 (`0x1000000`)**, set by `Leader::gain_tech (006dcb60):158`
  (listing `6dcb99`), unconditionally at entry. A tech (including a unit or
  building type's availability, an age) gained.
- Listing net: every `orl $0x800000|$0x1000000|$0x1800000` and byte-form
  `orb $0x80,2(...)` / `orb $0x1,3(...)` in `.text`. The other hits are unit
  masks (`0x68(%reg)`), window/road/profile flags, and one `orb $0x1,0x3(%edi)`
  in library code (`CheckConversionForWarnings`). Not netted: a set by
  whole-word assignment of `leader_flags` (e.g. a load), which this reading
  did not hunt.
- Consequence: **queueing a unit or starting a research does not set either
  bit**, though `num_queued` and `researching` are inputs (§4, §9, §10). The
  unit, upgrade and research parts go stale until the next unit is created or
  closed or the next tech gained. Building and economy parts are refreshed on
  every recompute frame regardless.

### 0.5 `TypeData::get_score_value (006673b0)` — vslot `+0x7c`, `value(t, p)`

The base value every part but territory/economy uses. No `Type` vtable
overrides `+0x7c` (`vtables.txt`: every `+0x7c` is `TypeData::get_score_value`).

- `c = Σ costs[0..6]` of type `t` (`TypeData+0x18`, as written in the XML,
  no factor). If `p == 0` and `t` is MILITIA (0x42), MINUTEMAN (0x43) or
  PARTISAN (0x44), the costs are taken from type 0x32 (PEASANTS, the
  Citizen) instead.
- Unit type (`0x32 ≤ t < 0x19e`): `v = UNIT_COST_FACTOR × c`. If `p == 0`,
  return `v`. If `p != 0`: `v = trunc256(RESEARCH_PREMIUM × v)`, then
  `trunc256(v × research_premium_cost(t))`, where `trunc256(x)` is signed
  division by 256 rounding toward zero (`(x + (x>>31 & 0xff)) >> 8`),
  `RESEARCH_PREMIUM` is 8.8 (ships 256 = 1/1, `crates/sim/src/core/tuning.rs:1625`)
  and `research_premium_cost` is `UnitTypeData+0x2e0`, 8.8. The premium
  arm uses the type's own costs (the militia substitution is `p == 0` only).
- Build type (`0x19e ≤ t < 0x21f`): `BUILD_COST_FACTOR × c`.
- Tech type (`0x220 ≤ t < 0x275`): `TECH_COST_FACTOR × c`.
- Spell type (`0x275 ≤ t < 0x2ac`): `SPELL_COST_FACTOR × c`.
- Anything else: `TECH_COST_FACTOR × c`.
- Each range test is the inlined `is_*_type` with a guard: if the vslot is
  overridden it calls it instead. Factors ship 10, 10, 10, 10
  (`tuning.rs:1614`–`1617`).

### 0.6 `get_kill_value` — vslot `+0x80`, `kill(t, owner)`

- `TypeData::get_kill_value (00663a00)`: `value(t, 0)`.
- `UnitTypeData::get_kill_value (0061d3d0)` (decompile garbled; the listing
  `61d3d0`–`61d41e` settles it): `value(t, 0)`, and if `owner >= 0` plus
  `(support_cost[0] + support_cost[1]) × (num_units_owner[t] +
  num_queued_owner[t])` (`ObjectTypeData+0x270`, `+0x274`; the owner's
  `LeaderData+0x5762` and `+0x5a22` arrays, through `leadersc`).
- `BuildTypeData::get_kill_value (00639790)`: `value(t, 0)`, and if
  `owner >= 0` plus `(support_cost[0] + support_cost[1]) ×
  BUILD_SUPPORT_FACTOR × (num_buildings_owner[t] + num_queued_owner[t])`.
- Which vtables carry the two overrides was not settled: `vtables.txt` has
  no `UnitType`/`BuildType` `Type` vtable (`tools/ghidra/README.md`'s
  `vftable_for_Type_` trap), so the assignment is by the class names.

### 0.7 Where the crate stands (applies to every leaf)

`compute_score` is not modelled. The step site is `Sim::strategy_all` in
`crates/sim/src/ai/ai_drive.rs:73`, whose comment at `:87` reads
"`compute_score(0)` — not modelled". **A builder must call it before the
human branch's `continue` at `:82`–`:85`**: the original calls
`compute_score` for every leader passing `leader_flags & 3 == 3`, humans
included (§0.1), but the crate's human arm leaves the loop before the
comment's position. The sweep order inside the original's loop is
`check_explore`, `plan_strategy`, `compute_score`, `diplomacy`, so
`diplomacy` on the same frame sees this frame's score. The crate also skips
`self.leaders.defeated[w]` where the original tests `leader_flags & 3`;
whether those agree for a defeated slot was not checked. The leaves already
have checksum slots in `crates/sim/src/checksum/leaders.rs:41`–`52`
(`LEADER_FIELDS`). The two dirty bits have no crate twin; a builder needs a
per-player flag pair set at unit creation, unit close and tech gain
(`Leader::gain_tech`'s crate port is `crates/sim/src/tech/gain.rs`).
`Leader::reset_score` zeroes are the crate's default (0), so no reset code
is needed for a fresh game.

### 0.8 What clears them, for all twelve

- **`Leader::reset_score (006e37f0)`** stores 0 to all twelve (and to
  `+0x808`…`+0x938`, other families). Callers (listing): `Leader::init
  (006e3930):140` (`6e3b1a`), right after `who` is written; and
  `Setup::build_game (005ac190):731` (`5ad1e7`), once per slot with
  `leader_flags & 1`, late in game setup (after the `setup.cpp` checksum
  `0x408`). Neither clears the dirty bits, so frame 0 recomputes everything
  with whatever bits creation and the starting techs set.
- `compute_score`'s own Armageddon arm (§0.2 step 8).
- A saved game's load (`Leaders::walk_data (006e38e0)`) presumably restores
  the record; not read.

---

## 1. `score` (+0x18)

1. **Writers.** Export: `->score` is never an lvalue (readers only:
   `get_team_score`, `get_mvp_score`, `log_data`, `AchieveChartPane::on_redraw`,
   `StatWin*`, `Options::describe`); `field_0x18` lvalues on a `Leader*`:
   `compute_score` (`:44`, `:50`) and `reset_score`. Listing:
   `0xe3a3a8(%reg)` appears only as loads (`Leader::diplomacy`,
   `Army::find_target`, `GameDaemon::process_victory`, `ScenarioFuncSet::score`,
   `at_least_points`, `less_than_points`); register-relative stores
   `0x18(%reg)` in `Leader`/`LeaderData`/`Leaders` methods: only
   `compute_score` (`6ec5..`) and `reset_score` (`6e37f3`); the rest are
   `%esp`/`%ebp` locals, the `sites` block (`compute_sites`, base
   `0x6e44(%ebx)`), the `data_encrypted` block (`Leader::init`'s `6e3e8b`,
   base `0x6eb8`) and arrays inside `Leader::init` (`6e3a18`, base
   `0x3b4(%ebx)`).
2. **Step.** `score = score_combat + score_wonders + score_research +
   score_unit_upgrades + score_economy + score_buildings + score_units +
   score_territory + score_pop + score_explored` on a recompute frame
   (§0.1) when Armageddon has not come; `0` (and return) when it has.
   `score_units_2` is not a term. Since `score_pop` and `score_explored` are
   always 0 at the add (§8, §2), in this build `score = combat + wonders +
   research + upgrades + economy + buildings + units + territory`.
3. **Reset.** §0.8; the Armageddon arm.
4. **Readers that decide.**
   - `Leader::diplomacy (006bc950):118`–`140`: over live leaders, counts
     those whose `score` is above mine (`local_64`) and finds the top scorer
     (`local_70`); `:2198` on is another walk. `get_team_score` (`006d6520`;
     the plain `score` unless `semaphore[0] & 0x80`, then a team sum/average
     — not read further) at `:566`, `:606`, `:608`, `:665`. The crate SEAMs
     these at `crates/sim/src/leaders/diplomacy.rs:155`, `:385`.
   - `Army::find_target (006f69b0):372`: with more than 19 members and more
     than 2 siege, a target leader whose `score` exceeds `4·mine/3`
     (`(score_mine << 2) / 3 < score_target`) changes the branch.
   - `GameDaemon::process_victory (00730ef0)`: `:443`–`449` and `:554`–`562`
     take the top `get_team_score`; `:625`–`660`, under `victory == 5` (the
     musical-chairs victory, every `chairs[...]·900` frames), the live
     leader with the strictly lowest `score` is defeated (a tie defeats
     nobody).
   - `Leader::process_event_frame (006ec180):90`, `:99` (`get_team_score`;
     not read for what it decides).
   - `LeaderData::log_data (006e5110):209` prints `score` in the `LEADERDATA`
     dump — the per-frame oracle carries it.
   - Display: `get_mvp_score (006d5de0)` (`score == 0 → ...`, else
     `3·combat − economy/2 − buildings/2 + territory + score`),
     `StatWin`, `AchieveChartPane`. Scenario triggers: `ScenarioFuncSet::score`,
     `at_least_points`, `less_than_points`, `is_victory_score`.
5. **Crate.** §0.7. No crate code computes it; its readers are SEAMed.
6. **Plain game?** Yes: `strategy_all` every frame for every live leader.
   The `defeat_all` caller needs Armageddon (nukes); `victory == 5` is a
   skirmish setting.
7. **Confidence.** High for the formula and the gate (decompile and listing
   agree, `6ec560`–`6ec67x`). Not read: `get_team_score` past its head,
   `process_event_frame`'s use, the three non-victory setters of
   `semaphore[0] & 0x40`.

## 2. `score_explored` (+0x1c)

1. **Writers.** `compute_score` (`6ec5a0`, `movl $0x0,0x1c(%esi)`, step 1,
   unconditional); `compute_explore_score (006bc5e0)` (stores 0, **no
   caller**); `reset_score`; the Armageddon arm. No absolute `0xe3a3ac`
   reference anywhere in `.text`. No other register-relative store in a
   `Leader` method (§1's net).
2. **Step.** **None: it is 0 on every recompute and never anything else.**
   In terms of `explored` (`LeaderData+0x9d4`, written by
   `Leader::check_explore (006bc860):20`, lane 4153's): `score_explored = 0`,
   independent of `explored`. Nothing in this build reads `explored` into
   the score. (The brief's premise that `score_explored` reads `explored` is
   not borne out by this executable; an older build may have had
   `compute_explore_score` do so, which is a guess.)
3. **Reset.** §0.8.
4. **Readers.** The `score` sum (a zero term). `StatWin` may show it; not
   checked.
5. **Crate.** Default 0 already matches; nothing to build beyond the checksum
   slot.
6. **Plain game?** Yes (the zero store runs every recompute).
7. **Confidence.** High: two stores of a constant 0, no other writer by
   either net.

## 3. `score_territory` (+0x20)

1. **Writers.** `compute_score` (`6ec604`, `movl %edi,0x20(%esi)`, and the
   Armageddon zero at `6ec628`); `reset_score`.
2. **Step.** On every recompute frame (§0.1): `territory × 1000 /
   land_size`, signed 32-bit, truncating toward zero. `territory` is
   `LeaderData+0x9d8` of `leaders[who]` (the same record); `land_size` is
   `WorldData+0x78` (World has no prefix before `WorldData`: `check_explore`
   reads `+0x24`/`+0x28` as `reg_xs`/`reg_ys`). Units: per-mille of the
   map's land tiles. Not gated by a dirty bit. Computed before the
   Armageddon test, then zeroed by it.
3. **Reset.** §0.8; Armageddon.
4. **Readers.** The `score` sum; `get_mvp_score`.
5. **Crate.** Inputs exist: `Holdings::territory` and `Holdings::land_size`
   (`crates/sim/src/economy.rs:554`–`555`), the same ratio the territory tax
   uses (`economy/holdings/tests.rs:823`). Whether `Holdings::territory` is
   refreshed with the same timing as `LeaderData::territory` was not
   checked.
6. **Plain game?** Yes.
7. **Confidence.** High. `land_size == 0` would fault in the original; not
   considered.

## 4. `score_units` (+0x24)

1. **Writers.** `Leader::compute_unit_score (006bc500)` (`6bc50c` zero,
   `6bc5b4` add; its one call `6ec5ab` in `compute_score`);
   `compute_score`'s Armageddon zero; `reset_score`.
2. **Step.** Recomputed from scratch only on a recompute frame (§0.1) **with
   `leader_flags & 0x1800000` set** (§0.4); otherwise it keeps its last
   value. For each unit type `t` in `[0x32, 0x192)` (all 352 unit types,
   gaia animals included):
   - `n = num_queued[t] + num_units[t − 0x32]` (`LeaderData+0x5a22 + 2t`,
     `+0x56fe + 2t` = `+0x5762` indexed from 0x32; both `ushort`;
     listing `0xe3fdb2`, `0xe3fa8e`).
   - If `n != 0`: `u = ( ((n − 1) × (support_cost[0] + support_cost[1]) × n)
     / 2 + value(t, 0) × n ) / 5`, each `/` signed and truncating toward
     zero (`cltd; sub; sar` for `/2`, `0x66666667` for `/5`). `support_cost`
     is `ObjectTypeData+0x270`/`+0x274` of `unittypes[t]`; `value` is §0.5.
     No `BUILD_SUPPORT_FACTOR` here (the building part has it).
   - `score_units += u`.
   - Gate inside: if `game->armageddon >= get_armageddon()` the leaf is left
     at 0.
   In words: what the army would cost to rebuy at the current ramp —
   base cost times count plus the triangular support ramp — fifthed.
   Counts include units queued in production, though queueing does not set
   the dirty bit.
3. **Reset.** §0.8; zeroed at the head of each `compute_unit_score`.
4. **Readers.** The `score` sum; `StatWin` (not read).
5. **Crate.** Inputs: `Muster::by_type` and `Muster::queued_by_type`
   (`crates/sim/src/units/unit.rs:838`–`840`, `num_units`/`num_queued`),
   unit costs and support (`crates/sim/src/economy/cost.rs:144`), the cost
   factors (`crates/sim/src/core/tuning.rs:580`–`608`). Dirty bit 23's sites:
   the crate's unit creation (`Unit::init`'s twin) and `close_unit`
   (`crates/sim/src/combat/fight/delivery.rs:825`, `die_with`, and every
   other close path). The crate's `Muster` indexes by record, not by
   TypeIndex; the `0x32` bias is the original's.
6. **Plain game?** Yes.
7. **Confidence.** High (listing `6bc500`–`6bc5dc` read instruction by
   instruction). Not read: whether every unit-removal path goes through
   `Unit::close` (e.g. conversion, upgrade-in-place); a removal that does not
   would not set bit 23.

## 5. `score_units_2` (+0x28)

1. **Writers.** `compute_unit_score` (`6bc513` zero, `6bc5c6` add);
   `reset_score`. **Not** in `compute_score`'s Armageddon zero list.
2. **Step.** Same loop and same `u` as `score_units` (§4), added only when
   the type's `attack` (`ObjectTypeData+0x1e8`) is non-zero: the
   fighting-unit subset of `score_units`. Same dirty-bit gate. Armageddon:
   `compute_unit_score` zeroes it at its head and then returns before the
   loop, so once Armageddon has come it is 0 after the next recompute that
   has a dirty bit; `compute_score`'s own zeroing arm leaves it alone. A
   recompute after Armageddon with neither bit set leaves the old value.
3. **Reset.** §0.8.
4. **Readers.** **Not a term of `score`.** Display only (`StatWin`,
   `AchieveChartPane` — not read for it); no decision reader found
   (`->score_units_2` has no hit in the export).
5. **Crate.** As §4; the `attack` column is `ObjectTypeData+0x1e8`
   (`docs/AI.md:231` cites the same field).
6. **Plain game?** Yes.
7. **Confidence.** High for the formula; the claim "no decision reads it"
   rests on the name grep and the absence of `0xe3a3b8` loads in `.text`.

## 6. `score_buildings` (+0x2c)

1. **Writers.** `Leader::compute_build_score (006bc3f0)` (`6bc3f7` zero,
   `6bc4c7` add; one call `6ec5b8`); Armageddon zero (`6ec621`);
   `reset_score`.
2. **Step.** On **every** recompute frame (no dirty bit). For each build
   type `t` in `[0x19e, 0x21f)` (cities, walls and wonders included):
   - `n = num_queued[t] + num_buildings[t − 0x19e]` (`+0x5a22 + 2t`,
     `+0x5222 + 2t` = `+0x555e` from 0x19e; listing `0xe3fdb2`,
     `0xe3f5b2`).
   - If `n != 0`: `b = value(t, 0) × n + ((n − 1) × ((support_cost[0] +
     support_cost[1]) × BUILD_SUPPORT_FACTOR) × n) / 2` (the `/2` truncating;
     `BUILD_SUPPORT_FACTOR` is `Constants+0x37c`, ships 1).
   - If `t` is a wonder (inlined `is_wonder_type`, address compare
     `0x470780`; `0x20e ≤ t < 0x21f`, PYRAMIDS … SPACEPROGRAM; an overridden
     vslot is called instead): `score_wonders += b / 3` (§11). Else
     `score_buildings += b / 5`.
   - Armageddon gate as everywhere.
3. **Reset.** §0.8; zeroed at the head of each call.
4. **Readers.** The `score` sum; `get_mvp_score` (`− buildings/2`).
5. **Crate.** Inputs: `num_buildings_of` / `num_sites_of`
   (`crates/sim/src/ai/ai_types.rs:51`, `:59`; the crate's reading of
   `num_queued` for a building is "placed sites not yet active"),
   `build_support_factor` (`tuning.rs:590`).
6. **Plain game?** Yes.
7. **Confidence.** High (listing `6bc3f0`–`6bc4f7`).

## 7. `score_economy` (+0x30)

1. **Writers.** `Leader::compute_economy_score (006bc270)` (`6bc277` zero;
   adds at `6bc2f0`, `6bc324`, `6bc34d`; one call `6ec5bf`); Armageddon zero
   (`6ec61a`); `reset_score`. (`calc_resource_caps`'s `0x30(%edi)` and
   `resource_cap_add`'s `0x30(%ecx)` are `data_encrypted->resource_cap`, base
   `+0x6eb8`.)
2. **Step.** On every recompute frame (no dirty bit). For each good
   `g = 0..5` (TypeIndex 0..5: food, timber, metal, oil, wealth, knowledge)
   with `LeaderData::type_avail(g, 1) != 0`:
   - `a = (bucket[g] − game->starting[g]) / 20`, truncating; clamped to
     `[0, 100]`. `bucket` is `data_encrypted->bucket[g] ^ 0x8221` (the
     stockpile, whole units); `starting[g]` is `Game+0x600 + 4g` (the
     starting grant; the decompile prints it as `info.player + 0x5bc + 4g`,
     the listing `(%edi,%eax)` with `edi = 0x600 + 4g` on `game` settles it).
   - `b = (data_encrypted->income[g] ^ 0x90236) / 80`, truncating; capped
     at 200, **no floor** (a negative income would subtract). `income` is
     the net rate in sixteenths (`docs/AI.md:375`), so `b` is income in
     whole units per the ledger's time base, fifthed.
   - `score_economy += a + b`.
   Then `score_economy += 25 × popcount(rare)` (`BitMask<44>::count` of
   `LeaderData+0x6d98`, the rares held).
3. **Reset.** §0.8; zeroed at the head.
4. **Readers.** The `score` sum; `get_mvp_score` (`− economy/2`).
5. **Crate.** Inputs: `Ledger::bucket`, `Ledger::income`
   (`crates/sim/src/economy.rs:681`, `:696`), `Holdings::starting`
   (`:615`), `rare` (`:725`), `TechTree::type_avail`
   (`crates/sim/src/tech.rs:1714`). Whether the crate's `income` is the
   same quantity at the same moment as the original's `income` at
   `strategy_all` time was not checked.
6. **Plain game?** Yes.
7. **Confidence.** High for the arithmetic (listing `6bc270`–`6bc354`).
   Not read: `type_avail(g, 1)` for goods past `has_preq`/`type_eligible`
   (which goods it refuses, e.g. oil before its tech).

## 8. `score_pop` (+0x34)

1. **Writers.** `compute_score` (`6ec5ca`, step 5, unconditional 0; and the
   Armageddon zero); `compute_pop_score (006bc190)` (stores 0, **no
   caller**); `reset_score`. (`calc_resource_caps`'s `0x34(%edi)` is
   `data_encrypted`.)
2. **Step.** **None: always 0.** Its term in `score` is a zero.
3. **Reset.** §0.8.
4. **Readers.** The `score` sum.
5. **Crate.** Default 0 matches.
6. **Plain game?** Yes (the zero store).
7. **Confidence.** High.

## 9. `score_unit_upgrades` (+0x38)

1. **Writers.** `Leader::compute_unit_upgrades_score (006bc1a0)` (`6bc1a3`
   zero, `6bc230` add; one call `6ec5d5`); Armageddon zero; `reset_score`.
   (`calc_resource_caps`'s `0x38(%edi)` and `gain_tech`'s `0x38(%ebp)` are
   `data_encrypted` and a frame local.)
2. **Step.** Recomputed only on a recompute frame **with `leader_flags &
   0x1000000`** (bit 24, a tech gained). For each unit type `t` in
   `[0x32, 0x192)`, add `value(t, 1) / 5` (truncating; §0.5's premium arm)
   when all of:
   - `has_tech(t)` (`LeaderData::has_tech (006e0c80)`) **or**
     `researching(t, −1, 0, 1)` (`LeaderData::researching (006db510)`: some
     building of mine, object ids from 2000, active per its vslot `+0x4c`,
     has `t` in a queue slot; with the last argument 1 only an exact match
     counts);
   - **not** `tech_at_start[t]` (`LeaderData+0x6c80` BitMask, bits at
     `+0x6c8c`, byte `t >> 3`, bit `t & 7`; the listing's `rolb` walks it);
   - **not** `unit_flags & 0x80` (`UnitTypeData+0x2b4`, flag `h`: free with
     its prerequisite, `crates/sim/src/tech.rs:81`).
   In words: the research price of every unit line researched (or queued)
   since the start, excluding the free ones, fifthed.
3. **Reset.** §0.8; zeroed at the head of each call.
4. **Readers.** The `score` sum.
5. **Crate.** Inputs: `PlayerTech::tech_at_start` (`crates/sim/src/tech.rs:722`),
   the unit-research bit (`Muster::researched`, `units/unit.rs:841`, or the
   tech bitmask), `research_premium_cost` (`economy/pricing.rs:733` reads
   it), `uflags` 0x80. No crate `LeaderData::researching` over building
   queues was found (only a trait method in `groups/group_queue_up.rs:76`).
   Dirty bit 24's site: `crates/sim/src/tech/gain.rs`.
6. **Plain game?** Yes.
7. **Confidence.** High for the loop and its three predicates (listing
   `6bc1a0`–`6bc241`). Not read: `has_tech` for a unit type past its head
   (it consults `has_preq`; what it returns for an unresearched but
   available-at-start type is not settled), `researching`'s vslot `+0x4c`
   meaning.

## 10. `score_research` (+0x3c)

1. **Writers.** `Leader::compute_research_score (006bc360)` (`6bc363` zero,
   `6bc3db` add; one call `6ec5dc`); Armageddon zero; `reset_score`.
2. **Step.** Same gate as §9 (bit 24). For each type `t` in
   `[0x220, 0x275)` (BASE_TECHTYPES = BASE_AGETYPES = 0x220, up to the
   first spell; ages included), add `value(t, 0) / 5` (i.e.
   `TECH_COST_FACTOR × Σcosts / 5`, truncating) when `has_tech(t)` or
   `researching(t, −1, 0, 0)` (a tech is never a unit type, so the fourth
   argument's upgrade arm never fires: exact queue match only), and not
   `tech_at_start[t]`. No `unit_flags` test.
3. **Reset.** §0.8.
4. **Readers.** The `score` sum.
5. **Crate.** As §9.
6. **Plain game?** Yes.
7. **Confidence.** High (listing `6bc360`–`6bc3e7`). The base value is the
   type's written costs × factor, not the ramped price a player pays.

## 11. `score_wonders` (+0x40)

1. **Writers.** `compute_build_score` (`6bc3fe` zero, `6bc4ec` add);
   Armageddon zero; `reset_score`. `compute_wonder_score (006bc260)` is an
   empty `ret` with no caller. (`Leader::init`'s `6e3c15` `0x40(%esi)` is an
   array walk off `%esi`, not the leaf; `gain_tech`'s `0x40(%ebp)` a local.)
2. **Step.** §6's loop: for a wonder type (`0x20e ≤ t < 0x21f`) with `n =
   num_queued[t] + num_buildings[t − 0x19e] != 0`, `score_wonders += b / 3`
   with `b` as in §6 (truncating `0x55555556` multiply). Every recompute
   frame. Wonders under construction count through `num_queued`.
3. **Reset.** §0.8; zeroed at the head of `compute_build_score`.
4. **Readers.** The `score` sum.
5. **Crate.** As §6.
6. **Plain game?** Yes.
7. **Confidence.** High.

