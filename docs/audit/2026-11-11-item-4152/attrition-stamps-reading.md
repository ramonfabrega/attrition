# Item 4152, family 5 `attrition-stamps` — first reading

Leaves: `attrition_stamp` (LeaderData +0x1f4), `attrition_stamp2` (+0x1f8), `attrition_stamp3` (+0x1fc), `att` (+0x7f0), `give_att_disabled` (+0x7f8), all `int`.

Reader: Claude Opus 5.5 (claude-opus-5-5), first reading, 2026-11-11. Sources: the decompile export (`~/ghidra-projects/decomp`), the listing (`llvm-objdump` of `riseofnations.exe`), `types.txt`, and `crates/sim`.

## Layout note

`types.txt` lists `Leader` (size 0x6eec) with no fields of its own and `LeaderData` (0x6ee4). The decompile reaches these leaves as `leaders.list[i].field_0x1f4` and as `this->field_0x1f4` in `Leader::init (006e3930)`, which also writes `field_0x7f0`/`field_0x7f8` beside them; `Leader::calc_attrition (006cdea0)` tests `field_0x7f8` and writes `field_0x7f0`. The `Leader` offsets therefore equal the `LeaderData` offsets (no prefix; the 8 extra bytes are after `LeaderData`). The listing check is in each leaf section (absolute stores at `0xe3a390+off`).

Listing check of the base: every absolute store found is `0xe3a390+off(%reg)` with `%reg = slot·0x6eec` (`imull $0x6eec`), and the two loop stores (`Game::do_frame`, `Leader::gain_tech`) run a register from `$0xe3a390` in steps of 0x6eec to `0xe71af0`. So the leaves are the 8-slot `leaders` array's `LeaderData` fields, no prefix.

How the nets were run: `grep -rn -- '->leaf\b'` and `'[.>]leaf\b'` over `decomp/funcs` (hits only in `LeaderData::log_data (006e5110)` and `LeaderData::get_attrition (006e0b60)`); `grep 'field_0x(1f4|1f8|1fc|7f0|7f8)\b'` (Leader offsets = LeaderData offsets); the listing for `0xe3a584`, `0xe3a588`, `0xe3a58c`, `0xe3ab80`, `0xe3ab88` (absolute), and every dword store / unary op to `0x1f4|0x1f8|0x1fc|0x7f0|0x7f8(%reg)` attributed to its function through `INDEX.tsv`, then the base register of each store in a leader-ish function read from the surrounding instructions.

## Leaf 1 — `attrition_stamp` (+0x1f4, int)

### Writers

| writer | site (listing) | store | base proven by |
|---|---|---|---|
| `Unit::process_attrition (005e11a0)` peace arm, shipped path | `5e1880` | `game->frame` | `movzbl 0x9(%ebx)` (unit owner) `· 0x6eec`, absolute `0xe3a584` |
| `Unit::process_attrition (005e11a0)` peace arm, grace sub-arm | `5e1868` | `max(1, ally_to_war_grace + frame − 900)` | same |
| `Unit::process_attrition (005e11a0)` team-style-2 ("assassin") arm | `5e1586` | `game->frame` | same |
| `Game::do_frame (00591ef0)` rush-timer end | `59232b` | `0` | `%ebx` from `$0xe3a390`, `testb $1,(%ebx)`, step 0x6eec |
| `Leader::gain_tech (006dcb60)` rush-age reached | `6e038a` | `0` (`%ebx` was `xorl`ed) | `%esi` from `$0xe3a390`, step 0x6eec |
| `Leader::init (006e3930)` | `6e3dda` | `0` | `%ebx` = `this` |
| `Leaders::clear (006ed5d0)` | memset | `0` | `memset(slot, 0, 0x6929)` per slot covers +0x1f4 |
| `LeaderData::walk_data (006d6750)` via `Leaders::walk_data` / `WalkDataGame::walk_data (005a2360)` | — | the saved value | a loading walker writes every walked leaf; not read store by store |

Stores at `0x1f4(%reg)` rejected after reading the base: `Game::close`/`Game::run_oos_recovery` (`%eax` from global `0xc06214`, a 0xc4-stride clear), `ConquestGame::close` (`%esi` = the `ConquestGame`), `ScenarioEditor::{change_tribe, change_team_color, change_type, change_name_dialog, new_leader, delete_leader, init}` (`%eax`/`%esi` = the editor at global `0xe7f8dc`/`this`; `ScenarioEditor +0x1f4` is `update_leader_list` in `types.txt` line 21289), `ScenarioRead::load_leaders_chunk` (`%edx` is a local record of strings being constructed, not a leader).

### The event, the step, the predicate

`Unit::process_attrition` runs per unit from `Unit::process (00610bc0)` (its only caller, `:454`) on the unit's 32-frame refresh phase (`(frame + id_phase) & 31 == 0` inside the `& 15` gate). Before the stamp is reached the unit has passed every exemption that precedes the branch: not inside a scenario `attrition_free_points` radius; tile owner ≥ 0 (a negative tile owner returns either at `neutral_attrition == 0` or at the later `neutral_attrition != 0` return, so never stamps); tile owner ≠ unit owner; tile owner's `leader_flags & 1` and `& 2`; tile owner ≠ `leaders[owner].who`; not mutual `diplos == 2` (alliance); `leaders[owner].take_att_disabled == 0`; `leaders[tile].give_att_disabled == 0`; the unit-kind chain (worker gathering on a flagged tile, merchant, hero, supply, `type+0x1e8 == 0`, special, `is(0x3a)`, caravan); type domain `+0x218 != 1` (ships); `leaders[owner].neutral_attrition == 0`; not (CtW `semaphore[2] & 2` and `team_style != 2` and conquest bonus 9).

Then two arms write the stamp:

1. **Peace arm** — taken when `is_enemy(owner, tile) == 0` (both sides' `diplos` nonzero: peace, or alliance one way only), or when CtW is on and `Game::war_allowed` is 0. The stamp is re-written when `stamp == 0` or `frame − stamp > 900` (literal `0x384`, frames; `jle` at `5e1510` skips at exactly 900). The value is `game->frame`, except when both the "alliance broken within `ally_to_war_delay`" test (`+0x2b0` stamps, either side) and "still inside `ally_to_war_grace`" hold, in which case it is `ally_to_war_grace + frame − 900`, floored at 1 (`cmovl` at `5e185b`) — so the next re-stamp waits out the grace. With shipped data `ALLY_TO_WAR_DELAY` and `ALLY_TO_WAR_GRACE` load as `-1` (`docs/ATTRITION.md` check 12; `Constants::init (00569a90):2399–2401`), every window test passes, and the stamp is always `frame`. The write is **not** gated on the console: the message/sound (`Game::say_no_war` when `rush_rules != 0`, `!war_allowed` and `stamp == 0`; then one of six strings) is for the console's leader only, the store is for every slot.
2. **Assassin arm** — taken when `is_enemy` is 1, `team_style == 2`, and `LeaderData::get_target(owner) (006da000)` is not the tile owner. Same `stamp == 0 || frame − stamp > 900` gate, value `frame`.

At war under any other team style, or at war with the target in style 2, the function jumps straight to the damage (`LAB_005e18f1`) and the stamp is untouched. So: the stamp is the frame of the slot's latest peace/assassin border violation, re-armed at most once per 901 frames, across all of that slot's units.

`Game::do_frame (00591ef0):219–226`: on the frame where `rush_rules > 8` (the time-based rules) and `frame == rush_rules[r].minutes·900` (the table word at `+0x3c` of the 0x58-byte row), and `team_style ∉ {0, 8, 11}` (`Game::teams_locked` inlined), each slot with `leader_flags & 1` gets `attrition_stamp = 0`, and then (unless `LeaderData::is_neutral`) declares war on every live slot it neither allies nor is at war with (`Leader::set_diplo(…, 0)`). It runs after `Leaders::process_all` and before `Objects::process_all`, so a unit refreshed that same frame sees 0 — but by then the leaders are at war, so the peace arm no longer stamps.

`Leader::gain_tech (006dcb60):195–200, 2314–2330`: when the gained tech is an age (type vtable `+0x34`), `rush_rules != 0`, `tech − BASE_ITEMTYPES == rush_rules` (the age-based rules, 1..8), and `Game::war_allowed` was 0 when tested (before the age is applied), then if `Game::teams_locked`, every slot with `leader_flags & 1` gets `attrition_stamp = 0` and the same war declaration (`set_diplo(…, 0)`, message `0xb414`).

### Reset

`Leader::init` (0, `:264`), `Leaders::clear` (memset), the two rush-end zeroings above. No per-frame fold. `Leader::reset_score (006e37f0)` does not touch it (grepped).

### Readers

`Unit::process_attrition` only (`5e14f9` assassin arm, `5e1710` peace arm): the 900-frame re-arm gate and `say_no_war`'s `stamp == 0` test. `LeaderData::log_data` prints it. It changes no damage and no decision outside the warning; it is checksummed state.

### Crate site

`Sim::process_attrition` (`crates/sim/src/territory/upkeep.rs:239`) on its 32-frame refresh, where `attrition_for` (`:318`) builds the `Situation` and `territory::attrition::process` (`crates/sim/src/territory/attrition.rs:521–526`) assigns `peace_attrition` (`!at_war`) or `assassin_attrition` (`situation.assassin`). The stamp belongs where that outcome is a `Period` with `ignores_supply == true` (true exactly for those two arms), with the exempt checks before it already ported in order; the `WarGrace` exemption is dead with shipped data but would stamp first in the original. Note `situation.assassin` is hard-coded `false` (`upkeep.rs:331`), so the assassin arm has no crate path yet. The zeroings: `Sim::end_the_peace` (`crates/sim/src/leaders/diplomacy.rs:74`) is `Game::do_frame`'s arm — **its comment at `:87` calls the `+0x1f4` store `update_leader_list` "the interface's"; that name is `ScenarioEditor +0x1f4`, and the store's base is the leader slot (`0xe3a390 + i·0x6eec`), so it is this leaf.** The same misreading stands in `docs/AI.md:20956`, `:20968`, `:21011`. The age-based arm in `gain_tech` has no crate counterpart (grep of `crates/sim/src/tech/gain.rs` for rush/war finds none).

### Plain game?

Yes. The peace arm needs two leaders at peace (`diplos 1`) and a unit of one in the other's land — the "peace pair" lobby (`RUSH_RULES 10`, `TEAM_STYLE 1`) does it: `docs/RUNS.md` run1072 has leader 0's `attrition_stamp` = **1392** on block 1393 (twin run848: 0), written on frame 1392 after `say_no_war`. The `Game::do_frame` zeroing is reached in the same lobby at frame 4500 (`set_diplo` on 4500 in run1072; outside its dump window 1378–1635, so no dump witnesses the 0 yet). The age-rush zeroing needs `RUSH_RULES` 1..8, a lobby setting. The assassin arm needs `TEAM_STYLE 2`, a lobby setting no capture has (`docs/ARMY.md:1051`). The grace sub-arm needs a mod (shipped constants `-1`).

### Confidence

High for the writers and the 900-frame gate (listing read at every store). The `team_style == 2` name "assassin" rests on the constant it loads (`assassin_attrition`), not on an enum. Not followed: `LeaderData::walk_data`'s loader path store by store; the callers of `Game::do_frame`'s rush table layout (`+0x3c`, 0x58 stride) beyond the decompile; whether `type+0x218 == 1` is exactly "ship".

## Leaf 2 — `attrition_stamp2` (+0x1f8, int)

### Writers

| writer | site | store | base |
|---|---|---|---|
| `Unit::suffer_attrition (005e1a10)` | `5e1b8a` | `game->frame` | unit owner `· 0x6eec`, absolute `0xe3a588` |
| `Leader::init (006e3930)` | `6e3de6` | `0` | `%ebx` = `this` |
| `Leaders::clear (006ed5d0)` | memset | `0` | slot memset of 0x6929 bytes |
| `LeaderData::walk_data (006d6750)` (load) | — | saved value | not read store by store |

Rejected `0x1f8(%reg)` stores: `ConquestGame::close` (`this`), `ScenarioEditor::init` (`this`), `ScenarioFuncSet::set_object_type_min_range` (an `ObjectType`'s `min_range`), `ScenarioRead::load_leaders_chunk` (local record), `Objects::add_ammo`/`Objects::kill_guy` (`Objects`).

### The event, the step, the predicate

`suffer_attrition(unit, graphic)` tests `leaders[owner].attrition_stamp2 == 0` at entry (`5e1a25`). If 0: when the owner is the console's leader, the tile under the unit has an owner ≥ 0, and `LeaderData::is_ally(owner, owner)` is 0 (the slot against itself: the listing pushes `%ebx` = the unit owner with `%ecx = 0xe3a390 + owner·0x6eec` before `call 0x6edb50` at `5e1a8e`–`5e1a95`, so the argument is the owner, not the tile owner — whether `is_ally(self)` can be 0 was not read; it gates only the message), it posts the "your units are suffering attrition" messages (string `0x120c0` when the tile owner's `tribe == 13`, else `0x120d4`; then `0x120fc`/`0x120e8` on `type_avail(FACTORY)`) and `S_ATTRITION_WARNING`. Then, **for every slot, console or not, message or not**, `attrition_stamp2 = game->frame`. Nothing re-arms it but a reset, so it is the frame of the slot's first attrition tick of the game. It is written before the damage call (`vtable +0x16c`), so a tick that kills still stamps.

Callers (listing, two `call 0x5e1a10`, both in `Unit::process (00610bc0)`): `:528` the period tick — `field_0x9e != 0`, `(frame + id) % field_0x9e == 0`, and `process_supply` refused shelter — with `graphic = 1`; and `:282` the decoy arm — a decoy on the map on a cell the owner's `field_0x6929` mask does not cover — with `graphic = (frame % 7 == 0)`.

### Reset

`Leader::init`, `Leaders::clear`. The two rush-end zeroings write `+0x1f4` only (read at `59232b`, `6e038a`). No per-frame fold.

### Readers

`Unit::suffer_attrition` itself (the once-only gate); `LeaderData::log_data`. No decision in the sim reads it.

### Crate site

`Sim::suffer_attrition` (`crates/sim/src/territory/upkeep.rs:286`) is the one entry both callers reach (the period tick from `Sim::process_attrition`, and the detected decoy from `Sim::recover_mana`, per its doc comment). A builder adds `if stamp2[owner] == 0 { stamp2[owner] = frame }` at its top, before the swept walk.

### Plain game?

Yes: the first attrition tick of any unit of a slot, in any lobby where a unit bleeds — at war in hostile land with the owner's attrition tech held, or at peace. ATTRITION.md's behavioural runs (run16) have such ticks. No dump value was looked up for this reading.

### Confidence

High for the writer and the once-only predicate (writers grepped in both nets; the two zeroing loops store at `+0x1f4`, not `+0x1f8`). The `is_ally(owner, owner)` argument is confirmed in the listing; what `is_ally` answers for its own slot was not read (message only).

## Leaf 3 — `attrition_stamp3` (+0x1fc, int)

### Writers

| writer | site | store | base |
|---|---|---|---|
| `Wall::process (00640450)` | `640bb7` | `game->frame` | building owner (`movzbl 0x9(%ebx)`) `· 0x6eec`, absolute `0xe3a58c` |
| `Leader::init (006e3930)` | `6e3df1` | `0` | `%ebx` = `this` |
| `Leaders::clear (006ed5d0)` | memset | `0` | slot memset |
| `LeaderData::walk_data (006d6750)` (load) | — | saved value | not read store by store |

Rejected `0x1fc(%reg)` stores: `Leader::init 6e3d63` (`%esi = this + 0xd4 + 4i` in the 8-iteration diplomacy loop, so `+0x2d0 + 4i`), `Leader::init 6e4b06` (a `movw` off `this + 0xd64`), `Unit::{do_attack, think_attack, cavarch_fight}` (a target object), `ScenarioEditor::{init, delete_leader}` (the editor), `ScenarioFuncSet::set_object_type_max_range` (an `ObjectType`), `ScenarioRead::load_leaders_chunk` (local record), the `Objects::find*` family (`GameAccess::objects`). `Wall` is the base of every building: `Build::process (0061edf0)` is the one direct caller of `Wall::process`.

### The event, the step, the predicate

`Wall::process` per building per frame. The enemy-land arm (`docs/CITIES.md` §9.5, which this reading agrees with): `(frame + id) & 15 == 0`; owner's `disable_building_attrition (+0x804) == 0`; then if `rush_rules == 0 || Game::war_allowed`, additionally `(frame + id) & 31 == 0`; the tile owner `t` (or, when the type's vtable `+0x108` answers nonzero, `BuildTypeData::check_enemy_adjacent`'s ring owner); `t ≥ 0`, `t ≠ owner`, `!LeaderData::is_ally(owner, t)`. Then:

- **not started** (`WallData::is_started`, `byte 8 & 2`, inlined at `640b01`) → `Object::disband(0)` (`6409d9`), then `jmp 640b43` — straight to the console message and **the stamp, with no `stamp3 == 0` test**. So every ghost site removed from rival land re-stamps.
- **started** → `take_damage(8, 0, 1, −1, 1, 0x55555555, −1, −1, 0)` through vtable `+0x16c` (`640b22`); a nonzero answer (the building died) returns with no stamp (`640b2a`); otherwise, only when `stamp3 == 0` (`640b3a`), the console message (string `0x123cc`, `S_ATTRITION_WARNING`, console's leader only) and the stamp = `game->frame`, for every slot.

So `attrition_stamp3` = the frame of the slot's first surviving building-attrition hit of the game, overwritten by the frame of any later ghost-site disband in rival land. Units: `frame` is `Game +0x550` (`movl 0x550(%eax)` off `0xc061ec`).

### Reset

`Leader::init`, `Leaders::clear`. No other store (both nets).

### Readers

`Wall::process` (the once-only gate on the started arm); `LeaderData::log_data`. No decision reads it.

### Crate site

`Sim::process_building` (`crates/sim/src/cities/city.rs:2997–3010`): the `!started → disband_building(b, false)` arm (stamp unconditionally) and the `building_attrition(b, frame)` arm followed by `if !alive return` (stamp when alive and `stamp3 == 0`). The crate gates on `phase % ENEMY_TERRITORY_PERIOD (32)` only; the 16-frame cadence under rush rules before war is not modelled (`crates/sim/src/cities/build.rs:1083–1088` says so), so a rush lobby would stamp on frames the crate does not visit.

### Plain game?

Yes. A started building on ground a rival's border took — run551 (`docs/CITIES.md` §9.5, `docs/GOLDEN.md` §57): a who=1 Tower site hit on 682 and every 32 frames after, `damage` 0 → 8 on block 683. By this reading who=1's `attrition_stamp3` reads 682 from block 683 on (a prediction, not looked up in the dump). A ghost site in rival land needs a border to move over a placed, unstarted foundation.

### Confidence

High: both arms' control flow read in the listing (`6409cd`–`6409de`, `640b0d`–`640bb7`). Not followed: what vtable `+0x108` on a build type is (dock ring vs plain tile, per CITIES.md); `take_damage`'s return convention beyond "nonzero returns".

## Leaf 4 — `att` (+0x7f0, int)

### Writers

| writer | site | store | base |
|---|---|---|---|
| `Leader::calc_attrition (006cdea0)` | `0x7f0(%ebx)`, `%ebx` = `this` (`movl %ecx,%ebx` at `6cdea1`) | the strength, below | `this` |
| `Leader::init (006e3930)` | `6e4984` (`:626`) | `0`, then `calc_attrition(this)` at `:635` | `%ebx` = `this` |
| `Leaders::clear (006ed5d0)` | memset | `0` | slot memset |
| `LeaderData::walk_data (006d6750)` (load) | — | saved value | not read store by store |

No absolute store to `0xe3ab80` exists. Rejected `0x7f0(%reg)` stores: `Constants::init` (`Constants +0x7f0` is `mongol_attrition`, which `Leader::calc_anti_attrition`'s `0x7f0(%ebx)` load at `6cde17` also reads — `%ebx` = `GameAccess::constants`), `ChatSpace::ChatSpace`, `IFaceResources::IFaceResources`, `TileSetData::{close, flush_all}`; `ScenarioFuncSet::ctw_set_sub_goals`'s is a `leal`.

### The event, the step, the predicate

`calc_attrition`, from the listing (`6cdea0`–`6cdfdc`), integers throughout:

1. If `give_att_disabled (+0x7f8) != 0` → `att = 0` (`jne 6cdfdc` past everything).
2. `n` = the length of the leading run of `ATTRITION1..ATTRITION4` (`TypeIndex` 0x2dd..0x2e0 = 733..736) that `LeaderData::has_preq (006db810)` answers for. The `BUY_SELL` (0x2ad = 685) / tribe-bonus-4 test inside the loop can never match an index in 733..736 (dead, as `docs/ATTRITION.md` settled).
3. `x = 0`; if `n > 0`, `x = constants[+0x1d4 + 4n]` — the decompile's `attrition_upgrade[n + 3]`, which runs past the 4-entry `ATTRITION_UPGRADE` into `ATTRITION_IMPROVED[n − 1]` = 1, 2, 4, 8 with shipped data (the crate's `Tuning` note at `crates/sim/src/core/tuning.rs:63`).
4. Then, in this order, each arm `x = (100 + pct)·x / 100` (signed, truncating; `imul 0x51eb851f; sar 5` is ÷100) **and then `x = 1` if the result is 0** (`cmovel` on the zero flag of the quotient, `6cdf2b`):
   - Colosseum (`has_wonder(0x212)`), `colosseum_attrition` (`constants +0x470`, shipped 50);
   - tribe bonus 13 (`has_tribe_bonus(0xd)`, the Russians), `russian_attrition` (`+0x74c`, shipped 100);
   - CtW (`semaphore[2] & 2`) and the leader byte `+0x6900` nonzero, `ctw_attrition`;
   - Kremlin (`has_wonder(0x21a)`), `kremlin_attrition`.
5. `att = x`.

**The floor fires on a base of 0.** With `n = 0`, `x` enters step 4 as 0, and any arm that applies gives `(100 + pct)·0 / 100 = 0`, then 1. So a leader with no attrition tech but with the Colosseum, the Kremlin, the CtW byte, or the Russian bonus has `att = 1`, not 0. **Diff-backed by the dump on disk**: run981 (`~/ron-golden/run981/map-7/gamelog.txt`, the `DUMP_ALL` start, Great Sahara, players `(0, tribe 4)` and `(1, tribe 13)`) prints `att 0` for the Nubian leader (line 809939, record from `tribe 4` at 809536) and **`att 1` for the Russian leader** (line 822197, record from `tribe 13` at 821794), both with no attrition tech at frame 0–36. `docs/ATTRITION.md` §"Strength" step 1 says "No steps means no attrition at all, which is where everybody starts"; that is false for the Russians, and `crate::territory::attrition::strength` (`crates/sim/src/territory/attrition.rs:42–45`, the `if tech_steps == 0 { return 0; }` at `:43`) returns 0 early on `tech_steps == 0`, so the crate holds 0 where the original holds 1 for every Russian leader from frame 0. The consequence is not cosmetic: `UnitData::get_attrition (00608fd0)` returns 0 when the territory owner's `att == 0` and otherwise `resistance / att` (aged-up scaled), so for a base victim (`anti_att` 256.0, non-siege) Russian land answers `256 / att'` (`att'` = 1 before the aged-up term) from the first frame, where the crate answers no attrition at all (a reading of the consumer; no capture has a foreign unit in Russian land that I looked up).

Event: `calc_attrition` is called from `Leader::calc_unit_stats (006cf970):10` (first statement), `Leader::gain_tech (006dcb60):434` (only inside the arm where the gained tech is a civic (`vtable +0x38`), its tech-type word `+0x14 == 1`, and the civics level equals `ctw_wonder_min_civics` — beside `calc_pop_cap`, `calc_anti_attrition`, `Region::fix_borders`), and `Leader::init (006e3930):635` (listing: three `call 0x6cdea0`). `calc_unit_stats` is called from `Leader::process (006b88b0)` and `Leaders::process_all (006ed2a0)` (the `0x4000000` dirty flag, which `gain_tech` raises on every tech via `leader_flags |= 0xc000000`, and `Build::activate`/`Build::close` raise for a wonder), from `Leader::set_age`/`set_epoch`, and from the scenario editor and `ScenarioFuncSet` (`gain_upgrade`, `lose_upgrade`, `set_object_type_*`, `create_building`, …).

### Reset

`Leader::init` (0, then recomputed — so a Russian slot leaves `init` at 1), `Leaders::clear` (0). Recomputed, not folded, on every `calc_unit_stats`.

### Readers

`UnitData::get_attrition (00608fd0)` (`608fe6`, `0x7f0(%ecx,%eax)`, the territory owner's slot): `att == 0` or `constants.attrition == 0` → no attrition; otherwise the divisor, after `att = ((attrition_aged_up·age_diff + 100)·att + 99) / 100` when the owner is at or above the victim's age. `LeaderData::get_attrition (006e0b60)` returns it and has no direct caller in the listing (inlined). `LeaderData::log_data` prints it.

### Crate site

`Sim::calc_attrition` (`crates/sim/src/territory/upkeep.rs:24`) writes `territory.players[w].strength`, which is `att`. Three gaps against the original:

1. `attrition::strength` returns 0 when `tech_steps == 0` before the arms (above) — wrong for the Russians from frame 0, and for any Colosseum/Kremlin holder without the tech.
2. `StrengthMods` is built with `russian` only (`upkeep.rs:43–46`); `colosseum` and `kremlin` stay false, though the crate now has owned wonders (`crate::tech::wonder::COLOSSEUM`/`KREMLIN`, `wonders_held`). Its doc comment ("this crate holds no wonder a player owns") is stale.
3. It runs only from `Sim::apply_gained` (`crates/sim/src/tech/gain.rs:608`), immediately; the original's main path is `calc_unit_stats` on the next `Leader::process`. `Sim::calc_unit_stats` (`crates/sim/src/economy/rares.rs:679`) does not call it, so a wonder built (which raises the crate's `unit_stats_dirty`, `crates/sim/src/cities/city.rs:2491`, `:2887`) never recomputes `att`; and a tech gained during `Objects::process_all` changes `att` for the rest of that frame's units in the crate, the next frame in the original. A builder moves the call into `calc_unit_stats` (and keeps the `Leader::init` one) and adds the `give_att_disabled` short-circuit.

### Plain game?

Yes: the tech arm in any game reaching Allegiance; the Russian floor in any lobby with a Russian player, from frame 0 (run981, run644/647, run976 are Russian lobbies on disk); Colosseum/Kremlin in any game that builds them. The CtW arm and the `give_att_disabled` short-circuit are not reached.

### Confidence

High: the arithmetic and the zero-base floor read in the listing, and the floor diff-backed by run981's start dump. Not followed: the `ctw_wonder_min_civics` arm's predicate in `gain_tech` beyond the three conditions named; `Leader::set_age`/`set_epoch`'s own conditions.

## Leaf 5 — `give_att_disabled` (+0x7f8, int)

### Writers

| writer | site | store | base |
|---|---|---|---|
| `ScenarioFuncSet::enable_give_attrition (00a02900)` | `a02926` | `0` | `(arg − 1)·0x6eec`, absolute `0xe3ab88` |
| `ScenarioFuncSet::disable_give_attrition (00a02940)` | `a02966` | `1` | same |
| `Leader::init (006e3930)` | `6e498a` (`:627`) | `0` | `%ebx` = `this` |
| `Leaders::clear (006ed5d0)` | memset | `0` | slot memset |
| `LeaderData::walk_data (006d6750)` (load) | — | saved value | not read store by store |

**`Leader::plan_strategy (006b9620)` is not a writer.** Its one `0x7f8(%reg)` store, `6b9bff`, is `movw %ax, 0x7f8(%ecx)` with `%ecx = %ebx + 0xbe6` (`6b9bbe`), i.e. a 16-bit store to `this + 0x13de` = `reg_land[i]` (`ushort[64]` at `+0x13de` in `types.txt`), in the per-region clear loop beside `0x67f6(%edx,%ebx)`. The other `0x7f8(%reg)` dword stores are `Constants::init`, `Cliffs::~Cliffs`, `GraphicPieces::{close, ~GraphicPieces}`, `IFaceRenderManager::clear_all` — none a leader. `Leader::gain_tech 6df4fe` is a `cmpl` on `GameAccess::constants +0x7f8` (`%eax` from `0xc061f0`). No unary op on `0x7f8(%reg)` exists.

### The event, the step, the predicate

The two trigger actions take a 1-based player number; for `arg − 1 < 8` (unsigned) with the slot's `leader_flags & 1` and `& 2` both set, they store 0 (enable) or 1 (disable) and return 1; otherwise −1 and no store. Both are registered only in `ScenarioFuncSet::init_funcs (009c7570)` (the immediate `$0xa02900` at `9d3fd7`, `$0xa02940` at `9d4005`, each pushed to `call 0x9d4da0` with a name string; no direct call anywhere in `.text`). Neither calls `calc_attrition`, so `att` keeps its last value until the next `calc_unit_stats`; the gate below reads the flag directly, so the effect is immediate anyway.

### Reset

`Leader::init` (0), `Leaders::clear` (0). No fold.

### Readers

- `Unit::process_attrition (005e11a0)` (`5e132c`, `cmpl $0, 0xe3ab88(%edx)` with the **tile owner's** slot): nonzero → the unit takes no attrition in that leader's land (before the kind tests and before the stamp).
- `Leader::calc_attrition (006cdea0)` (`6cdea6`, `this`): nonzero → `att = 0`.
- Not printed by `LeaderData::log_data` (no `give_att` string in it; run981's leader records have `att`/`anti_att` and no such line), so the checksum stream is its only witness.

### Crate site

The gate exists as `PlayerState::give_disabled` read in `territory::attrition::process` (`crates/sim/src/territory/attrition.rs:498`, `victim.take_disabled || owner.give_disabled → Exempt(Disabled)`), in the original's order. Nothing writes it but a default; there is no trigger layer. `Sim::calc_attrition` does not short-circuit on it (the effect is already covered by the exemption, but the stored `att` would differ). A builder would add a `give_disabled` arm to `calc_attrition` and leave the writer to the trigger track.

### Plain game?

No. Both writers are trigger-layer actions (`ScenarioFuncSet`), cut from v1 with the trigger system; a plain skirmish holds the `Leader::init` 0 for the whole game. `crates/sim/src/checksum/leaders.rs:1253–1271` already answers its sibling `take_att_disabled` as a constant 0 on that ground; `give_att_disabled` belongs in that same list.

### Confidence

High: both nets, every store's base named, `plan_strategy`'s store disassembled. Not read: the script-side names of the two actions (strings at `0xb121ac`, `0xb12254`), and which shipped scenarios call them.

## Against the README row and the documents

- **`give_att_disabled`'s writer is not `Leader::plan_strategy`.** The row's last entry (and `docs/CHECKSUM.md:1760`) came from `plan_strategy`'s `movw %ax, 0x7f8(%ecx)` at `6b9bff`, whose base is `this + 0xbe6`: the store is `reg_land[i]` at `+0x13de`. The writers are `ScenarioFuncSet::enable_give_attrition (00a02900)` / `disable_give_attrition (00a02940)`, trigger layer only.
- **`attrition_stamp` has a third non-init writer the row omits: `Game::do_frame (00591ef0)`**, the time-based rush rules' end (zero for every live slot), beside `gain_tech`'s age-based one. `Sim::end_the_peace` (`crates/sim/src/leaders/diplomacy.rs:87`) and `docs/AI.md:20956`, `:20968`, `:21011` call that store `update_leader_list`, "the interface's"; `update_leader_list` is `ScenarioEditor +0x1f4`, and this store's base is the leader slot.
- **`att` is 1, not 0, for a leader with no attrition tech and any of the four multipliers** (the floor runs on a zero base). run981's start dump: Russian `att 1`, Nubian `att 0`. `docs/ATTRITION.md` §"Strength" step 1 and `attrition::strength`'s early return say 0.
- `attrition_stamp3` is the building-attrition warning stamp (`Wall::process`, the base of every building), which `docs/ATTRITION.md:817` lists as "still unseen".
- The row's other writers check out: `Unit::process_attrition` and `gain_tech` for `attrition_stamp`; `Unit::suffer_attrition` for `attrition_stamp2`; `Wall::process` for `attrition_stamp3`; `Leader::calc_attrition` for `att`.

## Summary

| leaf | writers (Name (addr)) | step | reset | crate site | plain game? | confidence |
|---|---|---|---|---|---|---|
| `attrition_stamp` +0x1f4 | Unit::process_attrition (005e11a0) ×3 stores; Game::do_frame (00591ef0); Leader::gain_tech (006dcb60); Leader::init (006e3930) | `= frame` on a peace-arm (not `is_enemy`, or CtW pre-war) or team-style-2 non-target border violation, when `stamp == 0` or `frame − stamp > 900`; grace sub-arm `max(1, grace + frame − 900)` (dead with shipped −1); `= 0` for every live slot at the rush rules' end (time: `frame == minutes·900`, rules > 8; age: rules 1..8 reached while war barred), team style ∉ {0, 8, 11} | Leader::init, Leaders::clear, the two rush-end zeroings | `Sim::process_attrition` refresh (`territory/upkeep.rs:239`), the `ignores_supply` Period from `territory::attrition::process`; zeroing in `Sim::end_the_peace` (`leaders/diplomacy.rs:74`, misnamed at `:87`); no age-rush arm in the crate | yes (peace pair; run1072 = 1392 on block 1393); assassin arm needs TEAM_STYLE 2 | high |
| `attrition_stamp2` +0x1f8 | Unit::suffer_attrition (005e1a10); Leader::init | `= frame` once, on the slot's first attrition tick (period tick or detected decoy), all slots | Leader::init, Leaders::clear | `Sim::suffer_attrition` (`territory/upkeep.rs:286`), top | yes | high |
| `attrition_stamp3` +0x1fc | Wall::process (00640450); Leader::init | `= frame` on a building's first surviving enemy-land hit (`stamp3 == 0`), and unconditionally on every unstarted site disbanded in rival land | Leader::init, Leaders::clear | `Sim::process_building` (`cities/city.rs:2997–3010`), both arms | yes (run551 Tower site, hit on 682: predicted 682) | high |
| `att` +0x7f0 | Leader::calc_attrition (006cdea0); Leader::init | recomputed: 0 if `give_att_disabled`; `ATTRITION_IMPROVED[n−1]` over the leading run of ATTRITION1..4; ×(100+pct)/100 for Colosseum, Russians, CtW, Kremlin, each floored at 1 **even on a zero base**; via calc_unit_stats (every `0x4000000`), init, one gain_tech arm | Leader::init (then recomputed), Leaders::clear | `Sim::calc_attrition` (`territory/upkeep.rs:24`) → `attrition::strength`: zero-base early return, no Colosseum/Kremlin, not called from `calc_unit_stats` | yes (run981: Russian `att 1` at start, diff-backed) | high |
| `give_att_disabled` +0x7f8 | ScenarioFuncSet::enable_give_attrition (00a02900); ScenarioFuncSet::disable_give_attrition (00a02940); Leader::init — **not** Leader::plan_strategy | trigger action: 0 / 1 for a live slot (1-based arg) | Leader::init, Leaders::clear | read as `PlayerState::give_disabled` (`territory/attrition.rs:498`), never written; no trigger layer | no (trigger layer) | high |
