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

`Sim::process_attrition` (`crates/sim/src/territory/upkeep.rs:239`) on its 32-frame refresh, where `attrition_for` (`:316`) builds the `Situation` and `territory::attrition::process` (`crates/sim/src/territory/attrition.rs` ~`:518–526`) assigns `peace_attrition` (`!at_war`) or `assassin_attrition` (`situation.assassin`). The stamp belongs where that outcome is a `Period` with `ignores_supply == true` (true exactly for those two arms), with the exempt checks before it already ported in order; the `WarGrace` exemption is dead with shipped data but would stamp first in the original. Note `situation.assassin` is hard-coded `false` (`upkeep.rs:330`), so the assassin arm has no crate path yet. The zeroings: `Sim::end_the_peace` (`crates/sim/src/leaders/diplomacy.rs:74`) is `Game::do_frame`'s arm — **its comment at `:87` calls the `+0x1f4` store `update_leader_list` "the interface's"; that name is `ScenarioEditor +0x1f4`, and the store's base is the leader slot (`0xe3a390 + i·0x6eec`), so it is this leaf.** The same misreading stands in `docs/AI.md:20956`, `:20968`, `:21011`. The age-based arm in `gain_tech` has no crate counterpart (grep of `crates/sim/src/tech/gain.rs` for rush/war finds none).

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

