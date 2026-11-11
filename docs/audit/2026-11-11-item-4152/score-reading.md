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

