# ORDERS.md versus `crates/sim` — 2026-09-05

Reader: Opus, docs-versus-code pass (not a blind reading: this reader saw the
document and the implementation, and nothing else).

Checked: ~300 stated rules across §1–§15, against `crates/sim/src/orders.rs`,
`movement.rs`, `group.rs`, `gather.rs`, `fish.rs`, `calc_gather.rs`,
`transport.rs`, `collide.rs` and `lib.rs`. **18 rows** below — the ones where
the code and the document disagree. Everything not listed was found
implemented as stated, or is named as a seam by §13/§14 (those are not rows:
a declared omission is not a drift).

Two sections got a lighter pass than the rest and should not be read as
cleared: **§9** (the start of a game) is implemented in
`crates/rondata/src/diff.rs`, not `crates/sim`, and was only spot-checked
against §9.3's `(n, ordered)` rule; **§11** (what the gamelog writes) is
`crates/rondata/src/gamelog.rs`'s and §13 states the sim does not emit the
list at all.

## Rows

### R1 — `work`'s target-liveness gate skips the `ATTACK` action, so a dead target is not killed before the step

| | |
|---|---|
| document | ORDERS.md §2.3 step 8, "if the target's `uid` … differs from the order's … an `is_attack` action that is current clears its target, and then **`repath(); kill_current_order(0)`**" |
| code | `crates/sim/src/orders.rs:1156-1173` (`fn work`) |
| document says | The liveness test in `work` covers **every** targeted action, `ATTACK` included: a dead or slot-reused target means `repath()` then `kill_current_order(0)`, before `do_job` dispatches. |
| code does | The `dead` match arms are `Build`, `Repair`, `Garrison` and `Gather` only; `Body::Attack(_)` falls into `_ => false`. An attack order whose target died is dispatched to `do_attack`, which kills it *and then immediately runs `find_melee_target` and queues a fresh attack order* (`orders.rs:4778-4785`). |
| difference shows | `UNITS=3` order-list dump on the frame after a chased target dies: the original's unit has an empty (or next-order) list, this sim's has a new `ATTACK` (type 10) at the front. run17 (`gamelog-run17-combat.txt`) and run23/run28 (islands war/engagement) are the captures that carry deaths under attack orders. |
| reached | reached (`Unit::work@0060d180`, `Unit::do_attack@005f1b80` are both entered; neither is in the blind list) |

`work` at `orders.rs:1156` opens `if let Some(a) = self.action_of(u)` and the
match at `:1157-1163` enumerates `Body::Build(b) | Body::Repair(b) |
Body::Garrison{building: b, ..}` and `Body::Gather(g)`; every other body,
attack included, yields `false`. The comment above it ("a build, repair,
garrison or gather whose target died") shows the narrowing was deliberate, but
§2.3 names the attack arm explicitly and gives it a different tail
(`repath(); kill_current_order`) from `do_attack`'s re-search. The two differ
observably: the original's unit ends the frame order-less, this one ends it
with a replacement attack order.

### R2 — `think`'s first arm, a squad member following its captain's fight, is absent

| | |
|---|---|
| document | ORDERS.md §2.4 step 1, "A non-captain whose captain's action is `ATTACK` with a valid target: `add_attack_order(target, QUEUE_NEW, …)` — squads follow the captain's fight." |
| code | `crates/sim/src/orders.rs:1376-1477` (`fn think`) — no such arm |
| document says | Before the citizen mask-clear and before the auto-attack, an idle non-captain whose captain is attacking a valid target takes that target with `QUEUE_NEW`. |
| code does | `think` runs: the citizen carry-clear, the mod-16 cadence gate, the `idle == 1`/32 auto-attack (`think_attack_join_army` + `find_melee_target`), `think_peasant`, `think_caravan`, the tail gate, `think_fish`, `think_merchant`, `think_scout`/`think_join_army`. A follower never consults its captain's order. |
| difference shows | Order-list dump at `UNITS=3` for a multi-figure squad: in the original every non-captain figure of a squad whose captain is fighting carries its own `ATTACK`; here only the captain does. No capture is known to reach it — the traced games' squads are single-figure or the followers are steered by `do_group_move` — so this is reading-only. |
| reached | blind for the arm's own predicate; `Unit::think@005f6e40` itself is reached |

`think` at `orders.rs:1390` starts with the citizen arm and `orders.rs:1397`
is the cadence gate; the auto-attack at `:1409` is guarded by
`self.attack_of(me) != 0 && (unit.idle == 1 || phase & 0x1f == 0)` and uses
`find_melee_target`, which is the *own* search, not the captain's target.
There is no `squad_captain`-driven read anywhere in `think`
(`squad_captain` at `orders.rs:2193` is only called from the group-move
code).

### R3 — `think`'s remembered `near_o`/`near_who` auto-attack is absent

| | |
|---|---|
| document | ORDERS.md §2.4 step 3, "a captain with a remembered `near_o/near_who` that is alive and in range → `add_attack_order(near, QUEUE_NEW, 0, 0)`; then for fighting kinds `think_attack` … or `think_merchant`" |
| code | `crates/sim/src/orders.rs:1409-1420` |
| document says | The auto-attack arm has **two** stages: first the remembered near object (whatever last brushed the unit), taken directly if alive and in range; only then the `think_attack` search. |
| code does | Only the second stage: `think_attack_join_army(u)` then `find_melee_target(u, -1)`. There is no `near_o`/`near_who` field on `Unit` at all (grep over `crates/sim/src` finds the identifier only in `ai_host.rs:989`, an unrelated script parameter). |
| difference shows | A soldier that has just had a target walk past it and then gone idle: the original re-engages on its first idle frame with no search; here it runs the full `find_melee_target` sweep, which can pick a different unit and takes different RNG. `UNITS=3` order list plus the combat draws on the first idle frame — run28/run29 (islands engagement windows) are the nearest captures. |
| reached | blind for the arm (`Unit::think@005f6e40` is reached, the memory's writer is not separately cited) |

### R4 — the chase move's "target gone" arm has no `0x481` distance gate

| | |
|---|---|
| document | ORDERS.md §4.4, step 4 of "Before planning": "A target that is gone and the unit **within `0x481`** of its point → `repath(); return 0`." |
| code | `crates/sim/src/orders.rs:1636-1650` (`fn do_move`) |
| document says | The repath fires only when the unit is already close to the dead target's point; a unit still far away keeps walking its move. |
| code does | `match self.units[u].combat.target { Some(t) if valid => …, _ => { self.repath(u); return Did::Nothing; } }` — any gone/invalid target repaths at any distance. |
| difference shows | Order-list depth on the frame after a distant chase target dies: the original still shows `MOVE_TO` in front of the `ATTACK`, this sim has popped it. Frame is the death frame + 1 in run23/run28 (`gamelog-run23-islands-war.txt`, `gamelog-run28-islands-engagement.txt`). |
| reached | reached (`Unit::do_move@005f7b30`) |

### R5 — the chase-move kill is not gated on the attacker's type or the collision test

| | |
|---|---|
| document | ORDERS.md §4.4, step 4: "if `ATTACK` and the target alive with the same `uid` — a **ranged** type with a non-building, non-flank, in-range target **and no collision at its own spot** → `kill_current_order` …; a **melee** type every 16 frames may `find_melee_target` a closer in-range unit; a **building** target in range, valid, no collision → kill the move" |
| code | `crates/sim/src/orders.rs:1642-1647` |
| document says | Three distinct arms. A melee attacker's chase move is **not** killed when the target comes into range — it only gets a periodic better-target search; only a ranged attacker (or a building target) kills the move, and only when nothing is standing on its own spot. |
| code does | `if self.is_in_range(me, t) { self.kill_current_order(u); return Did::Something; }` — one arm, for every attacker type, with no `detect_unit_collision` probe and no melee 16-frame search. |
| difference shows | A melee unit closing on a target: the original keeps `MOVE_TO` in front of the `ATTACK` (and keeps stepping) on the frame the target enters range; here the move is popped and the unit stops one step short. `UNITS=3` order list plus the unit's position, on the first in-range frame — run17 and run28 are the melee-engagement captures. |
| reached | reached (`Unit::do_move@005f7b30`) |

### R6 — `AttackOrder::ever_in_range` has no reader: the packer's drop rule is not implemented

| | |
|---|---|
| document | ORDERS.md §7.1, "`in_range`, `ever_in_range` … Read in **one** place: a **packer** type (`unit_flags2 & 4`) that is not packed, out of range now but `ever_in_range` — its target walked out of its reach once it had it — **drops the order** rather than chasing." |
| code | `crates/sim/src/orders.rs:276` (field), `:4818` (the only write) |
| document says | `ever_in_range` is a live predicate: a packer whose target has left its reach abandons the attack instead of chasing. |
| code does | The field is written (`a.ever_in_range = true` at `orders.rs:4818`) and never read anywhere in `crates/sim` — grep finds the field, its `false` initialisers (`orders.rs:1040`, `collide.rs:1866`), the write, and the `rondata` diff plumbing, and nothing else. A packer chases like any other type. |
| difference shows | A packed/unpacked siege unit whose target retreats: the original's order list loses the `ATTACK` and gains nothing; this sim inserts a `MOVE_TO` chase. `UNITS=3` order list on the frame the target leaves range. No capture is known to put a packer type on a retreating target, so this is reading-only. |
| reached | reached for `Unit::fight@005fd4d0`; the packer branch itself is not separately cited |

### R7 — §13's "not implemented" list is stale: `GroupMoveOrder` and the group-move step are implemented

| | |
|---|---|
| document | ORDERS.md §13, "**Not implemented** (documented above, stated here): `ATTACK_TO` as an order kind of its own, `GUARD`, `FOLLOW`, `PATROL`, `ATTACK_GROUND`, **`GroupMoveOrder` (§8.3, §8.4)**, board/await-board, cast, trade, strafe, air, special-anim" |
| code | `crates/sim/src/orders.rs:207` (`struct GroupMove`), `:1956` (`fn do_group_move`), `:1981` (`group_move_leader`), `:2020` (`group_move_follower`); also `Body::Cast` (`:288`, `do_cast`) and `Body::Trade` (`:300`, `do_trade`) |
| document says | The group move order, cast and trade are not modelled. |
| code does | `MoveOrder::group: Option<GroupMove>` carries the whole `GROUPORDER` block and `work` dispatches a grouped move to `do_group_move` (`orders.rs:1197-1200`); `CastOrder` and `TradeOrder` are both `Body` variants with their own steps. §8.6 and §15 of the same document say `GroupMoveOrder` landed 2026-09-04, and §6.9 documents `do_cast`, so §13's list contradicts them. |
| difference shows | No observable — a documentation row. It matters because §13 is what a fresh session reads to decide what is missing, and three of its entries are now wrong. |
| reached | n/a |

### R8 — a step refused by `invalid_loc` does not clear the verified-line bit

| | |
|---|---|
| document | ORDERS.md §4.5, "A step into a tile `invalid_loc` says is blocked: **`unit_masks &= ~8`**; return 0 — next frame `do_move` re-plans." Also §4.2: the bit is "cleared … **when `move_step` enters a tile that turns out invalid**". |
| code | `crates/sim/src/orders.rs:2894-2900` (`fn unit_step`) |
| document says | The refusal clears the "straight line verified" bit, so the *next* frame re-enters `do_move`'s planning half and re-runs `find_path` from the new bearing. |
| code does | `if target.tile() != from.tile() && invalid_loc(...) != 0 { return Did::Nothing; }` with an explicit comment "SEAM: the original also clears `unit_masks & 8` here; the bit has no reader this crate models". The bit **is** modelled — `Unit::line_ok` (`crates/sim/src/lib.rs:192`, doc-commented "`unit_masks & 8`: a straight line to the waypoint has been verified"), and `do_move` at `orders.rs:1782` branches on it. So the next frame skips `find_path` and steps straight at the tile it was just refused. |
| difference shows | A unit walking into a refused tile: the original re-plans on the following frame (and may pay `do_move`'s grid draw, one `Random::get` off the sync stream — `SITE_MOVE_GRID`), this one retries the same blocked step silently. The observable is the unit's position for the frames after the refusal, and the sync-stream draw count. run32/run62 (`gamelog-run32-roadpath.txt`, `gamelog-run62-roadpath.txt`) walk units across terrain edges; run43 (roadterraform) likewise. |
| reached | reached (`Unit::move_step@005faf30`) |

The sibling arm eleven lines down does clear it — `orders.rs:2962-2964`, "A
step the world refuses: re-plan next frame" sets `line_ok = false` — so the
two refusal paths in the same function disagree with each other, and only one
of them matches §4.5. The comment claiming the bit has no modelled reader is
the error: `orders.rs:1782` (`if !self.units[u].line_ok`) is that reader, and
`orders.rs:1689` (a fresh waypoint) and `:720` (`clear_partial_path`) are two
more writers of it.

### R9 — a swarm's approach move is always an `EXPLORE_TO`; the original makes it a `MOVE_TO` for a human builder and for every repairer

| | |
|---|---|
| document | ORDERS.md §5.4, "`add_move_facing_order(spot, angle, mode = BUILD_AT ? 1 \| (human ? 0 : 2) : 1, pathed 0, pos, action…)` — in the dump the order this creates is an **`EXPLORETOORDER`** (type 3)" |
| code | `crates/sim/src/orders.rs:3323-3346` (`fn swarm_around`), the `MoveKind::ExploreTo` at `:3329` |
| document says | The mode is `1` for a `REPAIR` swarm and for a **human**'s `BUILD_AT` swarm, and `1 \| 2 = 3` only for a non-human `BUILD_AT`. §1.2 and §4.3 give the mapping: mode 1 → `MOVE_TO`, 2 → `ATTACK_TO`, 3 → `EXPLORE_TO`. |
| code does | `swarm_around` passes `MoveKind::ExploreTo` unconditionally, for build and repair, human and AI. |
| difference shows | The `UNITS=3` order list: the approach move logs `type 1` (`MOVEORDER`) in the original for a human builder and for any repairer, `type 3` (`EXPLORETOORDER`) here. It also changes behaviour, not only the label: `work` routes a `MoveKind::ExploreTo` through `do_explore_to_tail` (`orders.rs:1204-1206`), so this sim runs `find_goody_box` one frame in fifteen on walks the original never looks around on. run33 (`gamelog-run33-longtrace.txt`) and run69 (`gamelog-run69-greatlakes-3k.txt`) carry human citizens building; run10's AI citizens are the case that *does* match, which is why §5.4's dump note reads type 3. |
| reached | reached (`Group::action_swarm_around@0070fbe0` is entered; it is not in the blind list) |

Settled in the export rather than by reading alone, because the mode is a
computed local. `funcs/Group/action_swarm_around@0070fbe0.c:71` sets
`local_40 = 1` and `:115` overrides it **only** under `if (param_4 ==
BUILD_AT)` with `local_40 = ~(*(uint *)(leaders.list + who) >> 1) & 2 | 1` —
so a `REPAIR` swarm keeps mode 1, and a `BUILD_AT` swarm is 1 or 3 by the
leader's bit `0x2`. `local_40` is then the 5th argument of the
`add_move_facing_order` call at `:367`, and
`funcs/Unit/add_move_facing_order@005e55c0.c:65-79` is the dispatch:
`param_4 == 2 → ATTACK_TO`, `== 3 → EXPLORE_TO`, `== 4 → FLEE_TO`, else
`MOVE_TO`.

### R10 — the garrison approach ring has no `DOCK` case and no relaxed retry

| | |
|---|---|
| document | ORDERS.md §5.7, "**Not adjacent**: `add_move_order(spot, …)` with the spot from `find_nearby_spot` on the ring `min(x_size, y_size) × 0x60 + 0x30` (**a DOCK: `0x1b0..0x330`**), **retried relaxed**; … no spot → kill." |
| code | `crates/sim/src/orders.rs:3829-3845` (`fn do_garrison_order`, the `NotAdjacent` arm) |
| document says | Three things: the footprint ring, a special `0x1b0..0x330` ring for a `DOCK` target, and a second relaxed attempt before giving up. |
| code does | `let r = xs.min(ys) * HALF_TILE + SNAP;` then one `find_nearby_spot(u, bpos, r, -1, 0, angle, None)`; `None` kills the order at once. No dock ring, no retry. |
| difference shows | A unit ordered to garrison a dock it is not next to: the original walks to a spot on the `0x1b0..0x330` annulus, this one to the footprint ring or (more likely, a dock's footprint being large) kills the order. `UNITS=3` order list plus the inserted move's `x/y`, on the frame the garrison order first steps. run22 (`gamelog-run22-islands-dock-window.txt`) is the dock capture; no capture is known to garrison one. |
| reached | blind (`Unit::do_garrison@005e6b80` is in `/Users/rf-studio/ron-audit-scratch/blind-all-68traces.txt`) |

### R11 — the arrived gatherer's AI repair arm is absent

| | |
|---|---|
| document | ORDERS.md §6.3, the "every frame from here" block: "**AI citizen, every 256 frames phased by `(o + frame + who)`, diff > 1, `b` damaged, not under attack, in a city → `add_repair_order(ox, whom, QUEUE_NEW, 0)`; return**" |
| code | `crates/sim/src/orders.rs:4155-4179` (`fn do_gather`, the arrived tail) — no such arm |
| document says | An arrived AI gatherer whose building is damaged, not under attack and inside a city drops the gather and takes a repair order, on a 256-frame phase. |
| code does | The arrived tail goes straight to the oil-well bump / `do_farm`. `add_repair_order` has exactly three callers in the whole crate — `lib.rs:2899` (the player's order), `soak.rs:343` (the fuzzer) and its own definition at `orders.rs:921`; nothing in `do_gather` calls it. |
| difference shows | An AI citizen gathering at a damaged farm or camp: the original's order list shows `REPAIRORDER` (type 13) replacing the `GATHERORDER` on the qualifying frame, and the building's damage falls; here the citizen keeps gathering. `UNITS=3` order list plus `BUILDS` damage, at a `(o + frame + who) % 256 == 0` frame after any damage. run23/run24 (islands war/raid) damage AI buildings; run10 and run33 have long AI gather runs but no damage. |
| reached | reached (`Unit::do_gather@005ef2a0` is entered) |

### R12 — the tile approach ignores `avoid_x/y`

| | |
|---|---|
| document | ORDERS.md §6.4, `goto_build == 0`: "`find_nearby_spot(T, 0xc0, 0x100, 2, angle, NOT_ME, …)` fails, **or the spot is `u.pos` or `avoid_x/y`**: `tx = ty = −1; goto_build = 1; wait = −1; if dist_mod: dist_mod--`" |
| code | `crates/sim/src/orders.rs:4352-4355` (`fn do_non_flat_gather`) |
| document says | Three ways to give the tile up: no spot, the spot is where the unit already stands, or the spot is the point `find_path` last recorded as unreachable (`avoid_x/y`, §4.2). |
| code does | `Some(spot) if spot != here => { … }` — only the first two. `Unit::avoid` is written at `orders.rs:2376` (`find_path`'s "goal walked all the way back") and cleared at `:2752`, and is read nowhere. |
| difference shows | A woodcutter whose approach spot is the point its own `find_path` gave up on: the original abandons the tile (the `GATHERORDER` row's `tx/ty` go to −1 and `goto_build` to 1 on that frame), this one queues the walk again. The whole gather row is diffed (`OrderMismatch::Gather`), so run39 (`gamelog-run39-islands-longtrace.txt`) or run33 would show it on the frame it happens — no capture is known to have hit it. |
| reached | reached (`Unit::do_non_flat_gather@005f0170`) |

### R13 — the tile score's `dist_mod` clamp for a small mountain is missing

| | |
|---|---|
| document | ORDERS.md §6.4, the tile choice: "`dm = dist_mod;` (**a tiny mountain: `dm = min(dm, 3)`**); `best = 9999999` … `score = max(3, vector_dist(…)) * dm + (i >> 2)`" |
| code | `crates/sim/src/orders.rs:4460` (`let dm = g.dist_mod;`) and the score at `:4477-4478` |
| document says | Before the scan, `dist_mod` is clamped to 3 for a small mountain, so a mine on a small deposit weighs distance less. |
| code does | `dm` is `g.dist_mod` verbatim; there is no clamp. For a mine `add_gather_order` sets `dist_mod = 10` (§6.2), so an unclamped score is up to 3⅓× the clamped one and can pick a different tile. |
| difference shows | The `GATHERORDER` row's `tx`/`ty` on the frame a miner first chooses a tile, on a map with a small mountain. It is the same field run10's woodcutter comparison pins for wood. No islands/Great Lakes capture is known to put a miner on a small mountain, so this is reading-only. |
| reached | reached (`Unit::do_non_flat_gather@005f0170`) |

`UNSURE:` the document does not define "a tiny mountain" — the predicate is a
parenthesis, not a cited test. What would settle it is the listing around
`005f04xx` (the `dist_mod` load ahead of the scan) and the mine/mountain type
test it guards. The row is written because the clamp is stated and absent,
not because the predicate is known.

### R14 — `add_gather_order` does not clear the "has been a builder" bit

| | |
|---|---|
| document | ORDERS.md §6.2 step 1, "`QUEUE_NEW` → the clear (§3.1). **`unit_masks &= ~0x400`**" |
| code | `crates/sim/src/orders.rs:968-1005` (`fn add_gather_order`) |
| document says | Taking a gather order clears `unit_masks & 0x400`, the "has been a builder" latch. |
| code does | `add_gather_order` never touches `Unit::was_builder`, which is this crate's `unit_masks & 0x400` (`crates/sim/src/lib.rs:215`, and the two setters are `add_build_order`/`add_repair_order` at `orders.rs:912` and `:922`). The only clear is `think_peasant`'s tail at `orders.rs:1561`. |
| difference shows | A citizen that built something and then took a gather order: the original goes idle with the latch clear, so §5.9's build arm (`orders.rs:1541`, `was_builder \|\| stance ∈ {1,2}`) does **not** ask `find_build_spot`; here it does, and the citizen can be pulled off to a site. The observable is the order list the next time that citizen idles — `UNITS=3`, a `BUILDORDER` where the original has a `GATHERORDER` — and it changes what `find_build_spot` counts. run69 (`gamelog-run69-greatlakes-3k.txt`) and run33 both have citizens that build and then gather; §5.9's note records that this arm's predicate has already moved Great Lakes' word twice. |
| reached | reached (`Unit::add_gather_order@0061a5c0`) |

The latch matters only for a **stance-0** worker, because stance 1 and 2
pass the arm anyway — which is exactly an AI's trained citizen (`Unit::init`
and `Build::train` make it 0, per §5.9's note), so the divergence is on the
AI's own citizens rather than an edge case.

### R15 — `adjacent_to` has no sea-domain arm, so `BOAT_GARRISON_MAX_DISTANCE` is unused

| | |
|---|---|
| document | ORDERS.md §10, "`Object::adjacent_to(o, who)`: both active; `attack_dist(…) < 0x60` … **except a caller that is sea-domain, `is_unit()` and without `unit_flags & 0x10`, which uses `BOAT_GARRISON_MAX_DISTANCE` (`+0x180` if its `new_block_radius < 4`)**"; §12 lists `BOAT_GARRISON_MAX_DISTANCE` as a constant of this mechanic |
| code | `crates/sim/src/orders.rs:3025-3036` (`fn adjacent_to`) |
| document says | Two thresholds. A sea unit is "adjacent" at a much wider distance than a land one, widened again by `0x180` for a small-radius boat. |
| code does | One threshold for everything: `combat::attack_dist(…) < ADJACENT`, where `ADJACENT` is `0x60` (`orders.rs:437`). Grep for `boat_garrison`/`BOAT_GARRISON` over `crates/` returns nothing — the constant is not loaded, let alone read. |
| difference shows | A fishing boat or a barge reaching an oil platform or a dock: the original counts it arrived at `BOAT_GARRISON_MAX_DISTANCE`, this one keeps walking (or keeps re-swarming) until it is within half a tile. The observable is the `GATHERORDER`'s `been_there` and the inserted `MOVEORDER`, on the frame the boat first comes near. run22 (`gamelog-run22-islands-dock-window.txt`) and run58's Fisherman are the nearest captures; run58's boat gathers on open sea rather than at a platform, so no capture on disk is known to reach the sea arm. |
| reached | reached (`Object::adjacent_to` is reached through `do_build`/`do_gather`; the sea arm itself is not separately cited) |

### R16 — §12 calls the group follower's cone 60°; §15 and the code make it 120°

| | |
|---|---|
| document | ORDERS.md §12, "the group follower's `v/3` capped 9, **the 60° cone `0x55555555`**"; §8.3 likewise, "within 60° of the leader's heading". §15 contradicts both: "within **a third of a turn** … (`5e8167`'s `0x55555555`)" |
| code | `crates/sim/src/orders.rs:396-402` (`fn within_third`), used at `orders.rs:2121` and `:2154` |
| document says | §12 and §8.3: a 60° window. §15: 120°. |
| code does | `within_third`: the wrapped difference folded into half a turn and compared against `0x5555_5555`, with the doc comment "**a third of a turn**, 120°, not the quarter `reversing` uses". The code follows §15. |
| difference shows | No observable *between* the sim and the original if §15 is right — the code matches §15. The disagreement is inside the document, and §12 is the stale half: `0x55555555 / 0x100000000` is one third of a turn, and the fold halves the range it covers on each side, so the window is ±120°. A reader taking §12's number would narrow the follower's straight-to-slot arm and change every marching frame of run76 (`gamelog-run76-greatlakes-archermarch.txt`). |
| reached | reached (`Unit::do_group_move@005e79a0`) |

§12's other named constant with no code behind it is `MTN_TINY_SIZE` — "the
miner's `dist_mod` cap, §6.4" — which is R13; grep over `crates/` finds it
only in a test comment in `crates/sim/src/gather.rs:905`, about
`max_gatherers`, not about `dist_mod`.

### R17 — §7.2's 1-in-5 re-search predicate is inverted; the code (and the decompile) trigger on `% 5 != 0`

| | |
|---|---|
| document | ORDERS.md §7.2, "**The 1/5 re-search** (not mandatory, not recharging, captain, unit target): `Random::get % 5 == 0` or flag `0x10` → `find_new_target`." |
| code | `crates/sim/src/orders.rs:4800-4814` (`fn do_attack`) |
| document says | The re-search fires on one draw in five (`% 5 == 0`), and flag `0x10` also fires it. |
| code does | `if !self.profile(Obj::Unit(t)).combat_role && roll % 5 != 0 && let Some(f) = self.find_melee_target(u, -1) && f != target { self.retarget_attack(u, f); return; }` — four draws in five, and only for a target that is **not** a combat unit. Flag `0x10` is handled upstream at `orders.rs:4778` (the "bad target" kill), not here. |
| difference shows | How often a unit swaps targets mid-fight: `UNITS=3`'s `ATTACKORDER` `ox`/`whom` over an engagement, and the combat draw sequence. run28/run29 (`gamelog-run28-islands-engagement.txt`, `…-run29-…-window.txt`) are the engagement windows; the draw itself is a `Unit::fight` trace record on every frame the arm runs. |
| reached | reached (`Unit::fight@005fd4d0`) |

The code is right and §7.2 is the stale half — settled in the export.
`funcs/Unit/fight@005fd4d0.c:403-407`:

```
iVar12 = Random::get(GameAccess::game_random,0,0xffff);
if ((iVar12 % 5 == 0) || (pUVar11 = update_order(this), (pUVar11->flags & 0x10U) != 0)) {
  local_14 = 0;
}
… if ((iVar12 == 0) || (local_14 != 0)) { … find_new_target … }
```

`local_14` is the "look for a better target" flag, set from the target's type
just above; `% 5 == 0` **clears** it, and so does flag `0x10`. So the search
runs on `% 5 != 0` and flag `0x10` *suppresses* it here — the opposite of
§7.2 on both clauses. `docs/COMBAT.md` §8.2 step 0 already states the draw
correctly ("the draw is not `% 5 == 0`"), so this is ORDERS.md disagreeing
with COMBAT.md as well as with the code; COMBAT.md's own "or the order has
flag `0x10`" is the same clause read the same wrong way.

Note also that `fight` takes **two** `Random::get` draws
(`fight@005fd4d0.c:349` and `:403`), not one. The first is the guard arm —
an AI captain whose *activity* is `GUARD` (`get_activity() == 0xc`) rolls
once and kills the order on `draw & 0x80000001` — which is ORDERS §7.2's
"Opportunity check". `GUARD` is a declared seam (§13), so this crate takes
neither that draw nor its branch; it is worth recording because the draw is
on the **sync stream** and a capture with a guarding AI unit would part
here.

### R18 — a re-plan that returns 0 with an empty stack kills the order instead of taking the top

| | |
|---|---|
| document | ORDERS.md §4.4, the re-plan's result handling: "`if r == 0: if path.length == 0: **goto TAKE**` … `elif r != −1: TAKE: last_x/y = −1; dest = 1; top = peek …`" |
| code | `crates/sim/src/orders.rs:1906-1910` (`fn do_move`) |
| document says | `r == 0` with an empty stack falls through to `TAKE`, which peeks — and §4.2 records that `peek` on an empty stack returns `list[0]`, a stale entry, with callers guarding on `length`. The order is **not** killed. |
| code does | `let Some(top) = self.units[u].path.last().copied() else { self.kill_current_order(u); return Did::Something; };` — an empty stack at `TAKE` kills the move. |
| difference shows | A unit whose re-plan empties its stack: the original walks at whatever `list[0]` holds and re-plans next frame; this one drops the order and the action beneath it becomes current. `UNITS=3` order-list depth on that frame. No capture is known to reach it — the planners always leave the goal on the stack when they answer 0. |
| reached | reached (`Unit::do_move@005f7b30`) |

`UNSURE:` this is the one row where the document describes reading
uninitialised memory, so "what the original does" is not obviously worth
copying. It is written because §4.4 states the control flow explicitly and
the code takes a different branch, and because the difference is a killed
order rather than a stale coordinate. What would settle whether it can ever
happen: the three planners' contract on a zero return — whether
`find_wpath`/`find_tpath`/`find_upath` can answer 0 having popped the stack
empty.

---

## Summary

| row | claim | side that is wrong | reached |
|---|---|---|---|
| R1 | `work`'s liveness gate skips the `ATTACK` action | code | reached |
| R2 | `think`'s squad-follows-captain arm absent | code | blind |
| R3 | `think`'s remembered `near_o`/`near_who` auto-attack absent | code | blind |
| R4 | the chase move's "target gone" arm has no `0x481` gate | code | reached |
| R5 | the chase-move kill is not gated on attacker type or collision | code | reached |
| R6 | `ever_in_range` has no reader; the packer drop rule absent | code | reached |
| R7 | §13's "not implemented" list is stale (`GroupMoveOrder`, cast, trade) | document | n/a |
| R8 | a step refused by `invalid_loc` does not clear the verified-line bit | code | reached |
| R9 | a swarm's approach move is always `EXPLORE_TO` | code | reached |
| R10 | the garrison ring has no `DOCK` case and no relaxed retry | code | blind |
| R11 | the arrived gatherer's AI repair arm absent | code | reached |
| R12 | the tile approach ignores `avoid_x/y` | code | reached |
| R13 | the tile score's `MTN_TINY_SIZE` `dist_mod` clamp absent | code | reached |
| R14 | `add_gather_order` does not clear the "was a builder" bit | code | reached |
| R15 | `adjacent_to` has no sea arm; `BOAT_GARRISON_MAX_DISTANCE` unused | code | reached |
| R16 | §12 calls the follower cone 60°; §15 and the code make it 120° | document | reached |
| R17 | §7.2's 1-in-5 re-search predicate is inverted | document | reached |
| R18 | a re-plan returning 0 with an empty stack kills the order | code (`UNSURE:`) | reached |

Fifteen rows are the code lagging the document; three (R7, R16, R17) are the
document lagging the code, and two of those three are ORDERS.md disagreeing
with another document of the set (§13/§15 with each other, §7.2 with
COMBAT.md §8.2 step 0).

**The three rows most worth acting on**, by the size of the observable and
the ease of the fix:

1. **R8** — one line. The comment says the bit has no modelled reader and it
   does (`Unit::line_ok`); the sibling refusal path eleven lines away
   already clears it. It changes when a blocked unit re-plans, and a
   re-plan is a sync-stream draw.
2. **R9** — one argument. A human builder's and every repairer's approach
   move is the wrong `OrderIndex` in the dump *and* takes a
   `find_goody_box` the original never runs.
3. **R14** — one line, and it lands on the AI's own trained citizens, whose
   worker stance is 0 and for whom the latch is therefore the whole
   predicate of §5.9's build arm.

**On method.** Six of the eighteen rows turn on something a code comment
asserts about the original: R8 ("the bit has no reader this crate models" —
it does), R5 and R4 (the chase arm compressed to one test), R9 (the mode
argument read as a constant), R13 and R12 (clauses dropped from a
transcribed pseudocode block). A comment that states what the original does
is doing the same job as the document and is not checked against it by
anything; where the two disagree, nothing fails. That is the same gap this
pass exists to cover, one level down.
