# The pathfinder

**Status: first reading done, 2026-08-23.** Read from the full decompile
export (`~/ghidra-projects/decomp/`, `tools/ghidra/`) by the main thread, per
the model split: `astar_path@00683770` (977 lines), `calc_cost@00684e50`
(509), all three grid wrappers from the inside, the validity predicates, the
lifecycle functions, and the container semantics the open list's ordering
lives in. Two Opus surveys supported it — the container internals
(`~/ghidra-projects/reports/pathfinder/containers.md`) and the logging/oracle
question (`…/logging.md`) — with every load-bearing claim from either
re-verified here against the decompile (the tie-break in
`Tree::ordered_insert@004796f0` was read directly). The direction tables were
dumped from the PE at the linker map's addresses. Confidence: **high** on the
search, the cost function's structure, the wrappers, and the march
correction; each stated exception is under §12.

**And the search is diff-backed since 2026-09-03.** run70's `callwin`
proxies `astar_path` and `calc_cost` over a Great Lakes window and puts the
original's own expansion beside this crate's, cell for cell: twenty-one
expansions in the same order, every priced step's arguments and answer the
same, and the whole of §4's ordering — the wheel, the LIFO on an equal
`value`, the `<=` that lets a tie's first arrival keep it — confirmed by a
run rather than a reading (§17).

This document replaces the brief of 2026-08-22 (git has it). **The blind
second reading ran the same day**: two Opus readers re-derived the search
and the cost/wrappers halves from the export alone, an adjudicator took all
32 points of tension back to the decompile and the PE listing, and the
verdicts were ratified and folded in — `docs/audit/2026-08-23-pathfinder.md`
records them, and every claim below marked "audit V*n*" carries its verdict
number. The blind derivation agreed with the first reading on every
load-bearing structural fact (the LIFO tie-break, the wheel, the budgets,
`anti_unit`'s meaning, the arrival radius); its corrections were at the
edges — branch reachability, guard clauses, and the corner-probe table the
first reading had declined to trust.

---

## 1. Shape of the mechanic

One global `PathFinder` object (`pathfinder`), with its per-search state — a
`PathFinderData`, size 0x88 — embedded at **`PathFinder+0x40`** (proved
twice: every container offset lines up, and `dump_all@0092f2d0` passes
`&pathfinder->field_0x40` to `PathFinderData::log_data`). Three public
entries plan on three grids, all through one `astar_path(stack, step, anti)`:

| entry | grid | step | who calls it |
|---|---|---|---|
| `find_wpath` | world cells | `0x300` | a fresh far move (`do_move`, §4.5 of `docs/ORDERS.md`) |
| `find_tpath` | tiles | `0xc0` | the nearer refinement |
| `find_upath` | 48-unit cells | `0x30` | collision recovery (`resolve_unit_collision`) |

All three: pop the goal off the unit's `Stack<PathData>`, pre-walk it
toward the start, push back what the unit should walk (top first), return the
stack length; `0` = no path, `−1` = goal off the map (stack emptied) — as
`docs/ORDERS.md` §4.6 had it from the caller's side, now confirmed from the
inside. Out of scope, named and skipped: `astar_river` (map generation),
`astar_caravan_road`/`find_road` (the caravan's road layer, with its nine
tunable `road_*` weights at `PFD+0x60..0x80`, set once by `PathFinder::init`
from constants and adjustable only in a debug window), `find_wpath_army` (the
group's shared path — **it belonged with group orders and now lives
there**: `docs/GROUPS.md` §6.7, `crates/sim/src/grouppath.rs`. The named
function at `00683730` is four instructions and has zero callers, because
`Group::action_move_near` inlines it; what it does is set `+0x70` and call
`find_wpath` on the `grouppath` static).

## 2. The state

`PathFinderData` (`PFD`), with both offset forms since the decompile prints
`PathFinder+0xNN`:

| PFD | PathFinder | field | written by | meaning |
|---|---|---|---|---|
| +0x00 | 0x40 | `openlist` | init/suspend | `Tree<PathNode*,int>` — the open list, keyed on `PathNode::value` |
| +0x04 | 0x44 | `openlistrefs` | init/suspend | `BRTree<TreeNode*,ulong>` — open-list nodes by cell `metric` |
| +0x08 | 0x48 | `closedlist` | init/suspend | `BRTree<PathNode*,ulong>` by `metric` |
| +0x0c | 0x4c | `blocklist` | init/suspend | `Tree<CollBlock*,int>`: the copies `fill_slots` takes of world cells' collision blocks; ~~not read by the unit search~~ read by every `valid_ucoord` probe, emptied by `kill_lists`, handed to the unit on a suspend (§26) |
| +0x10 | 0x50 | `validlist` | init/suspend | `BRTree<int,ulong>` — `valid_ucoord`'s memo, `metric → 0/1` |
| +0x14 | 0x54 | `pathing_unit` | every wrapper | the unit |
| +0x18/0x1c | 0x58/0x5c | `sx, sy` | every wrapper | the unit's **tile** (`div3[pos>>6]`) |
| +0x20 | 0x60 | `dbg_collisions` | nothing | dead (printed by `log_data`, never read) |
| +0x24 | 0x64 | `anti_unit` | `find_upath` = 1 | **"this is the unit grid"** — not the `anti` argument. Gates the `limit` budget, the suspend path, +5-per-probe at `0xc0`, and flag 2 on reconstructed waypoints |
| +0x28/0x2c | 0x68/0x6c | `offx, offy` | `find_tpath` | target unit's sub-tile offset, `pos % 0xc0 − 0x60`; **no readers anywhere** (writers survey) — dead. `astar_path` derives its own `toff` instead, and from the **order**, not the target (§4.1) |
| +0x30 | 0x70 | `army` | `find_wpath`; also `Group::action_move_near` (which inlines `find_wpath_army`'s body — the named function has zero callers) | military, not a worker, not attacking, unit's own cell not river-flagged (§3). ~~**The attacking clause is vacuous**~~ — **live for `AttackOrder`, `GroupAttackOrder` and `StrafeOrder`**, whose `is_attack` answers 1 (§29); a `return 0` for every move (§22) |
| +0x34 | 0x74 | `iroquois` | `astar_path` entry | the unit's `unit_masks2 & 0x4000` — forest-walking; zeroed by each wrapper after |
| +0x38 | 0x78 | `worker` | `find_wpath` | `ObjectData::is_worker` |
| +0x3c | 0x7c | `no_danger` | `astar_path` entry | 1 when the action is an attack (vfunc `+0x10` == 10), the order is `ATTACK_TO`/`GROUP_ATTACK_TO`, or `who >= 8` |
| +0x40 | 0x80 | `limit` | `find_upath` wrappers | `500 / repaths[who]²` (halved with `anti`); restore: `300 / repaths²`. Expansion cap, applied only when `anti_unit` |
| +0x44 | 0x84 | `saving` | restore wrapper / `astar_path` | 1 = resume the search stashed on the unit. **Cross-call global state**: `find_tpath` and `find_upath` both skip their whole pre-A\* block when it is set, and `limit` persists across wrappers that never write it — a determinism hazard to respect when collision recovery lands (audit V29) |
| +0x48 | 0x88 | `avoid_land` | `astar_path` entry | §5's terrain-mode derivation |
| +0x4c | 0x8c | `avoid_sea` | `astar_path` entry | 0/1/2; 2 = refuse water outright |
| +0x50 | 0x90 | `valid_hit` | `valid_ucoord` | memo-cache hit counter (saved/restored across suspend) |
| +0x54 | 0x94 | `scouting` | `find_wpath` | auto-explore on `EXPLORE_TO` (§3); inverts the seen/unseen cost |
| +0x58 | 0x98 | `can_transport` | nothing found | not read by the unit search |
| +0x5c | 0x9c | `dbg_view_failures` | nothing | dead |
| +0x60..0x80 | 0xa0..0xc0 | `road_*` (nine) | `PathFinder::init` | caravan-road weights; out of scope |
| +0x84 | 0xc4 | `show_debug` | **nothing in the binary** | gates `dbg_tree_depth`; permanently 0 |

`PathNode`, 0x24 bytes, recycled through a global free list:

```
x, y      the node's position (grid-cell centre, in position units)
length    g — accumulated calc_cost
estimate  h — the heuristic (§6)
value     g + h — the open list's key
timeout   depth from the start node, in steps
metric    the node's grid-cell index: div3[x>>s] + div3[y>>s] * width
          (s = 8/6/4, width = cells/tiles/48-cells per row) — the identity
          key for the open-refs/closed/valid BRTrees
transport calc_cost's "this step embarks" flag → waypoint flag 4
building  set during 0xc0 reconstruction on a passable building tile (§7)
parent    the chain the path is read back off
```

`PathFinder::clear` zeroes every per-search field **except `pathing_unit`**
and the container pointers; each wrapper sets what it needs and re-zeroes its
own modes after (verified: `find_wpath` clears `army`/`worker`/`iroquois`,
`find_upath` clears `anti_unit`/`iroquois`, `find_tpath` clears `iroquois`).

### 2.1 The containers, because they are behaviour

From the Opus survey, spot-verified here; full detail with quotes in
`~/ghidra-projects/reports/pathfinder/containers.md`.

- The open list is a **plain unbalanced BST** keyed on `value`.
  `ordered_insert` descends right only on **strict** `key < new`; equality
  goes left — so a new node with an equal `value` sits **before** every
  existing equal — and `first_open_node` takes the leftmost. **Ties are
  LIFO: the newest equal-f node is expanded first.** A binary heap or a
  `(f, insertion-seq)` map that breaks ties oldest-first diverges on the
  first tie, and with an integer cost function ties are constant.
- The `BRTree`s (open-refs by metric, closed by metric, the validity memo)
  are red-black trees keyed on `metric`, exact-match `seek`, **no remove**:
  deletion is a `removed = 1` tombstone (+ a length decrement), and a later
  insert of the same key **resurrects the tombstone in place** (overwrites
  `data`, clears `removed`). That in-place merge is the engine's
  decrease-key.
- `Tree::length` is the search loop's termination condition ("open list
  empty"). `first_open_node` re-walks to the leftmost each call; on an empty
  tree it raises the engine's own error ("failed finding an open node
  somehow") — unreachable, the loop guards length first.
- Removal from the open list: `find_node_open` re-seats the tree's cursor
  from the refs entry, then `remove_current` unlinks that node (in-order
  predecessor promotion, no rebalance); the refs entry is tombstoned.

## 3. The wrappers, from the inside

`docs/ORDERS.md` §4.6's table is **confirmed** — every claim checked while
reading the bodies. What the inside reading adds:

**Public 4-arg forms** (`@00688e10/…e60/…eb0`) fetch the unit, un-XOR its
position (`pos ^ 0x63637`), and call the big forms. `find_upath`'s sets
`limit = 500 / max(1, repaths[who]²)`, halved if `anti`; `find_upath_restore`
sets `saving = 1` and `limit = 300 / repaths²`; both zero `saving` after.

**`find_wpath@00688fc0`** (world cells):

- Goal off the map → empty the stack, return −1. Same cell as the start, or
  a flying type (`unit_flags & 0x20`) → push the goal back untouched, return
  length.
- The pull-back walk (**AI leaders** — see below; domain < 2, and vfunc
  `+0x8` says no **or** the unit cannot transport): step the goal toward
  the start by `0x180` per iteration while the remaining Manhattan ≥
  `0x300`, else `0x30`. Three exits (audit V15/V17): the goal tile's
  `get_tregion` equals the start tile's (and, for a **sea-domain unit**, an
  `invalid_loc(goal,0,1,1,1,0)` clears) → on to the near test and A\*;
  the remainder is smaller than the current step on **both axes** → give
  up: push the goal where it stands and **return without A\***; the walk
  reaches the start's cell → the same push-and-return. The step's *sign* is
  the sine's own fold and not a second one — §13. **Human** leaders
  (`leaders.list[who] & 4` is literally `LeaderData::is_human` — audit
  V14) take the other variant, itself under the same guard: pop entries
  until one's world cell's centre region matches and its **half-cell**
  (`cell*2+1`) `was_seen`, also breaking on a final-flagged entry or an
  empty stack, re-validating each popped entry's cell (off the map →
  empty, −1); afterwards a gate — goal half-cell seen, or a sea-domain
  unit — rejoins the AI walk (audit V16).
- Push the (possibly pulled-back) goal; **cell-Manhattan < 3 → return
  length** (the near case the stub was exact for).
- Modes: `scouting = 1` iff the type has `+0x2c8 & 0x10`, the order is
  `EXPLORE_TO` with vfunc `+0x10` == 3, and (order `flags & 4` clear or
  `unit_masks & 0x40100`). **`+0x2c8` is `UnitTypeData::role`** — the word
  `UnitType::determine_roles@0061c320` derives — and bit `0x10` is set for a
  land type that `is(SCOUT)` and a sea type that `is(BARK)`, nothing else
  (`crate::ai_load::role::SCOUT`, already loaded on every type). **The type
  half is load-bearing and was missing until 2026-08-31** (queue item 113):
  `scouting` prices seen ground at `0x400` against unseen `8`, so *any* unit
  given an `EXPLORE_TO` walked toward the fog. run10's AI citizen `1/1` is
  sent to its second city's site under an `EXPLORETO` on frame 777 and took a
  ten-cell detour west through unexplored ground where the original walks
  seven cells south-east; no capture had exercised it before, because until
  the AI's borders were right no non-scout in either game was given an
  explore order over any distance. The `flags & 4` clause is a **seam**: it
  can only ever turn scouting *off* for a scout whose explore order carries
  `ACTION`, the two `unit_masks` bits are unmodelled, and every explore order
  in the corpus has `flags 1`. **`army` and `worker` are AI-only** — a human's
  `find_wpath` jumps straight to the search past the whole block (audit
  V14): for AI leaders, `army = 1` iff the type is military (`+0x1e8`
  attack ≠ 0, or its `is_supply` virtual — for the base class,
  `unit_flags2 & 0x40`), **and** not `is_worker`, **and** not
  `is_attacking` (the current order is an `ATTACK`, a group attack or a
  strafe — §29), **and** the **unit's own** cell's flags lack `0x100`
  (river); `worker = 1` iff `is_worker`. ~~The `is_attacking` clause is
  **dead in the shipped executable**~~ **(struck by item 899, §29: dead for
  every move order, live for three classes)** — the virtual it calls is a bare
  `return 0` in all seventeen order vtables — and reading it as a live
  test cost Great Lakes' word 322 frames: **§22**, which has the bytes
  and the value diff.
- Push `{goal-cell centre (cell*0x300+0x180), tol 0x180, flags 0}`, then
  `{start-cell centre, tol 0, flags 0}`; `astar_path(0x300, 0)`. Return 0 →
  pop the pushed goal, return `−(goal.flags & 1)`; else return the stack
  length. Clear `army`/`worker`/`iroquois`; `kill_lists`.

**`find_tpath@006897d0`** (tiles): as ORDERS §4.6, plus: the goal keeps its
own tolerance on the stack; the goal-tile-centre entry's tolerance is
**`0x180` if the goal's tolerance was `0x180`, else `0x60`**; the start-tile
centre gets tol 0. Before A\*, `PFD.offx/offy = mo->off_x/off_y % 0xc0 −
0x60` — the **current move order's** own, behind the same
`is_move()`/`update_move_order()` pair as §4.1's `toff` and a `Unit`
virtual (`+0x18`) that is a folded `return 1`. **R2, 2026-08-26**: the
first reading had this as "the current order's target is a unit,
`target.pos % 0xc0 − 0x60`", where `docs/ORDERS.md` §4.6's table had it
right; the field is dead either way (no readers), but the misreading is
the same one §4.1 carried and it was the live one. On failure pop a
non-final top; no order-killing here.

**`find_upath@00682f30`** (48-cells): as ORDERS §4.6, with the pre-walk's
validity being `valid_ucoord` (which also **seeds the memo**), the same-cell
and Manhattan-<2 pushes forcing tolerance 0, and `anti_unit = 1` around the
`astar_path(0x30, anti)` call. On failure (return ≤ 0, and not suspended):
pop a non-final top, then **kill the unit's current order unless** the
order's target is a unit and the order's `+0x1c` (the pause astar's failure
path just rolled, §4.4) is non-zero. On success with more than three
entries: drop a popped top equal to the unit's exact position (not final);
then, while the top run has `flags & 2`, drop middle waypoints that are
collinear with their neighbours (`ref − mid == mid − next` on both axes)
or whose surrounding pair sit exactly `0x30` apart on **both** axes (a
diagonal corner — audit V22), keeping any with `flags & 4`. The reference
point advances to the examined waypoint on a keep and on a collinear
drop, and stays put on a corner drop. Re-push what survives in order.
`kill_lists` always.

## 4. `astar_path(stack, step, anti)` — the search

### 4.1 Entry

Pops the **start** entry (top; becomes the root `PathNode`, `length 0`,
`timeout 0`, `parent 0`), then the **goal** entry (its `to` is the target
point; its tolerance is *not* the arrival tolerance — see below). After both
pops the stack's new top is the caller's final goal, and **`tol` := that
entry's tolerance** (0 when the stack emptied). Derived once:

- `iroquois = unit_masks2 & 0x4000`; `no_danger` per §2's rule.
- **`toff_x/toff_y`** — `0, 0`, then, if the unit has a current order and
  that order **`is_move()`**, the order's own `+0x4c/+0x4e` shorts. Both
  virtuals are the base `UnitOrder`'s and `docs/ORDERS.md` §4.1 already
  names them: slot `+0x14` is `is_move` and slot `+0x40` is
  `update_move_order`. Neither says anything about a *target*. `is_move` is
  a folded constant — `mov eax,1; ret` at `0041e0e0` for `MoveOrder` and
  its six derivatives, `xor`-to-zero at `0041bff0` for the base — and
  `update_move_order` is `lea eax,[ecx-0x54]; ret` at `00482f60`, the
  order's own `MoveOrder` sub-object. So `toff` is **the current move
  order's `off_x/off_y`**, which
  `Unit::add_move_facing_order@005e55c0` writes as `x mod 0x300` — the
  destination's offset inside its world cell — for every move, targeted or
  not. Used by the world probe (§6) and by every reconstructed waypoint
  (§7). **R2, 2026-08-26**: the first reading read the two folded slots as
  a target test and a target fetch; run20's dump settled it in a grep (the
  logged chain is at `cell*0x300 + off_x`, and `off_x` is on the same
  record).
- Per-step work budget `work_cap = 50` (`0x30`: **500**); stride `su = 1`
  (`0x30`: `(type +0x248 collision + 1) / 2`, min 1 — big units search on a
  coarser lattice); `stride = su * step`; row width `W` = cells (`0x300`),
  tiles (`0xc0`), or cells×16 (`0x30`); direction increment `dinc = 1`
  (`0x30` with `anti ≠ 0`: 2 — **only the four cardinals are expanded**).
- Arrival radius **`arrive = tol/2 + stride`** (`tol` = the *final* goal
  entry's tolerance — 0 for a plain move, so `0x300` on the world grid).
- `avoid_land`/`avoid_sea` from the start's terrain: **same region** for
  start and goal (§16 — the world grid compares the raw `WData.region`, the
  tile and unit grids call `get_tregion`) → on ocean (`0x300`:
  `WorldData::is_ocean` of the *cell*; else tile `& 0x30 == 0x20`) →
  `avoid_land = 1, avoid_sea = 0` (a ship); on land → `avoid_sea =
  1, avoid_land = 0`, and `avoid_sea = 2` when the action is an attack
  (vfunc `+0x10` == 10) — a fighting land unit refuses water. A type with
  `unit_flags & 0x10` whose `unit_masks & 0x40000` is set → both 0.
  Different regions → both 0 (the crossing is the point), except
  `avoid_land = 1` when the order has `flags & 0x20` — **a byte of a
  player's move command and nothing else sets it** (`add_move_facing_order`'s
  last parameter, `0` at every call site but `Group::action_move_near`'s own
  forward; twenty-fourth pass, A4 row 24), so the crate's dormant arm is right
  for every order a computer issues.
- The root's `metric` = its grid-cell index; `estimate = value = h(start)`;
  inserted into open + refs. Start-to-goal Manhattan is kept (the PDB's
  `UnitData::start_dist`), and the **direction preference**: if `|dx| >
  |dy|` then `pref = 7` if `goal.x < start.x` else `3`; else `pref = 5` if
  `start.y <= goal.y` else `1` — always a diagonal index; `pref + 1` is the
  **cardinal facing the goal**, and it is expanded first.

### 4.2 The direction wheel

`move_x/move_y` (compass tables, dumped from the PE at
`00adcaf0`/`00adc400`), indices 1–8:

```
 idx:   1        2       3        4       5        6       7        8
 d:   (-1,-1)  (0,-1)  (+1,-1)  (+1,0)  (+1,+1)  (0,+1)  (-1,+1)  (-1,0)
```

Odd = diagonal (+8 cost, §5), even = cardinal. Expansion order per node:
`pref+1, pref+2, … pref+8` (wrapping 9→1), i.e. the cardinal toward the
goal, then the wheel clockwise. In anti mode: `pref+1, +3, +5, +7` — the
four cardinals only.

### 4.3 The loop

While the open list is non-empty:

1. `n = first_open_node()` — min `value`, LIFO on ties (§2.1).
2. **Stop test**: `manh(n, goal) <= arrive`, **or** the work budget is
   spent (`traversed + probes >= work_cap * 64`, i.e. 3200, unit grid
   32000), **or** (`anti_unit` and `probes > limit`).
3. If stopping because of `limit` while still outside `arrive` and `anti ==
   0`: **suspend** — re-insert `n`, set `saving = 1`, move all five
   containers plus `tol`, `pref`, goal, `start_dist`, `traversed + probes`,
   `valid_hit`, `avoid_land/sea` onto the unit (`UnitData +0x104..0x148`:
   the PDB names them `openlist…blocklist`, `tol`, `offset`, `start_dist`,
   `endx/endy`, `valid_hit`, `traversed`), hand the global fresh containers,
   and return **−1**. `find_upath_restore` later re-enters with `saving =
   1`, which restores all of it and jumps straight into this loop.
4. Otherwise, if stopping: reconstruction (§7) — but first, if the budget
   (not arrival) ended a `0xc0` search with `anti_unit` set, or any `0x30`
   search, **fail instead**: return 0; on the unit grid, when the order's
   target is a unit, roll `pause = Random::get(0, 0xffff) % 3 + 6` into the
   order — **an RNG draw on the shared stream** — and, gated only on the
   grid, add 30 to `UnitData::safe` (`+0xb2`; the PDB's name — audit
   V1/V13). A search called with `anti ≠ 0` never takes this failure path
   at all — it always reconstructs whatever it reached (audit V27). A
   budget-ended search **on every other grid** — the `0x300` world's and
   the `0xc0` tile grid's without `anti_unit` — instead **drains the open
   list keeping the node nearest the goal** (`vector_dist`; `astar_path@00683770
   :568-592`, twenty-fourth pass, group 8), reconstructs the
   partial path from it, and — if the nearest node still needs a transport
   to reach the goal and the unit can take one (`unit_masks & 0x800000 &&
   !(unit_masks2 & 0x2000)`, or `unit_flags & 0x10`) — re-pushes the final
   goal with `flags |= 4` first.
5. Expansion: for each direction `d` in wheel order, the neighbour at `n +
   (move_x[d], move_y[d]) * stride`, `metric += move_x[d] + move_y[d] * W`:
   - **Validity** (§6): `0x300` → `valid_wcoord` (offset by `toff − 0x180`
     while `timeout < 2`, raw after); `0xc0` → `invalid_loc(tile,
     0,1,1,1,0) == 0`; `0x30` → `valid_ucoord`, and for a big unit
     (`su ≥ 2`) on a **diagonal**, every sub-step of the stride checked.
     Invalid → recycle, next direction.
   - `probes += 1` (`+5` on the tile grid with `anti_unit`).
   - `c = calc_cost(n → neighbour, d, step, timeout+1)` (§5);
     `0x7fffffff` → recycle. **If the neighbour is exactly the goal point,
     `c /= 2`.**
   - `g = n.length + c`. In the **closed** list (non-tombstoned) → drop —
     closed is final, a better late `g` does not reopen it. In the **open**
     list (by metric): existing `g` ≤ new → drop; else remove the existing
     (tombstoning its ref) and continue.
   - `h` = §6; `value = g + h`; `timeout = n.timeout + 1`. On the tile grid
     only: **drop the node if `h/32 + timeout > 120`** (round-toward-zero
     division) — the depth-plus-distance cap.
   - Insert into open + refs (the refs insert resurrects a tombstone in
     place).
6. Insert `n` into closed by metric; loop.

Open list exhausted → return 0, with the same unit-grid side effects as
step 4 — except the pause roll additionally needs the order data's
`+0x20 < 13` (the vfunc `+0x40` read; the gate is byte-verified, the
field's meaning is still open — §12). On both failure exits the
`safe += 30` cooldown is gated only on the grid being `0x30`, not on the
target.

### 4.4 Failure, and who consumes it

The RNG draws in step 4 and the exhausted case are the pathfinder's only
touches on the shared stream, and they happen **only on unit-grid
failures**. `find_upath` then reads the pause it wrote: pause set → the
current order survives (retry after the pause); no pause (a plain move to
somewhere unreachable) → `kill_current_order`.

## 5. `calc_cost(from, to, dir, step, depth)` — per-step cost

Returns `(base * 32) / 256 + extra + (dir & 1) * 8` — i.e. a flat step costs
**32**, a diagonal **+8**, `base` scales the step, `extra` adds flat
penalties. `0x7fffffff` = refuse the step. By grid:

**Unit grid (`0x30`)**: `base = 0x100`, `extra = 0` — geometry only — then
the transport tail below.

**World cells (`0x300`)**, unseen (`was_really_seen(to half-cell, who)`
false — the per-player fog byte; always "seen" for `who >= 8`, revealed
maps, and post-defeat leaders). The read is at
`div_3_table[to >> 7]` on both axes, i.e. the **half-cell containing the
step's destination**, `to / 0x180` — *not* the cell's `2c + 1` sample that
`Unit::think_scout` and the site census use, which is a different function
call with one letter between the names (§6, `crate::scout`):
`base = 0x124`; armies (`army`) pay `base = 0x2480` if the cell's flags have
`0x200`; **scouts (`scouting`) pay `base = 8`** — exploration seeks the
unseen. `extra = 8` unless scouting. Terrain, owner, danger are **not
read** — fog hides them.

**World cells, seen** (and tiles, where noted):

| term | amount | condition |
|---|---|---|
| base | `0x100`; **`0x400` if `scouting`** | seen ground is 32× dearer than unseen to a scout |
| danger | `+ danger[who][to >> 9 block] / 8` (arithmetic, rounded toward 0) | unless `no_danger`; **`docs/DANGER.md` is the writer, and it is negative around your own city** — the tile grid takes the same number *unshifted* |
| own territory | `− 4` | cell owner == who; **non-ocean cells only** (audit V18). **The whole additive column is clamped at zero after the army terms** (audit V19 — the conclusion is unchanged since those terms are positive): on clean ground the discount only ever offsets danger, never the base |
| enemy territory | `+ 4` | owner ≥ 0 and `is_enemy`; non-ocean cells only |
| ocean cell | `+ 200` | `is_ocean(to)` and `avoid_sea ≠ 0` |
| land cell | `+ 200` | not ocean and `avoid_land` |
| terrain | `+ 20 × tcost` | `tcost` = cell byte `+0x11` (the PDB's `WData.blocked`), or `+0x13` (`WData.solid`, signed) if `iroquois` — audit V13. **It is a count, and it has a writer**: see below |
| impassable terrain | `+ 100000` | `tcost ≥ 13` |
| army, rough | `+ 10000` | `army` and `tcost ≥ 5` |
| army, flagged cell | `base <<= 5` | `army` and cell flags `& 0x200` |
| corner-cutting | `0x7fffffff` | §5.1 |
| fleeing | `extra ×= 3` | `UnitData::is_fleeing`; reachable from the world-seen and tile branches only — never the fog branch or the unit grid (audit V5) |
| no-rush timer | `+ 500` | `rr = rush_rules age ≠ 0`, current age < `rr`, (`rr < 9` **or** `frame < rush_rules[rr].+0x3c × 900` — the once-garbled clause, settled in the listing, audit V6), owner ≥ 0, not an ally; same two branches as fleeing |
| diplomacy | `+ 5000` | (`army` or `worker`) and team-style rules: peace with the owner (styles 0/8/11), or style 2 and the owner is neither `who` nor `get_target(who)`; `0x300` only, and reachable from the **fog branch too** (audit V5) |
| river cell | `base ×= 3` | `0x300` only, cell flags `& 0x100`, **skipped whenever `needs_transport > 0`** — a shoreline crossed in either direction, transporter or not (audit V20) |

The `is_ocean` of the ocean and land rows — and of the entry block's
water test in §4.1 — is **`WorldData::is_ocean@006b4830`**, the `WData`
test: `flags & 0x100` clear *and* `land` 1 or 2. It is not "the cell's
region is a sea region", and a `HALFLAND` cell is where the two part
(§16).

The fog branch and the terrain row below are **one term, not two**, and
2026-08-26 is where that stopped being a guess. Both had been seams; each
was landed alone against run20's own scout and each made the chain
*worse* or barely better — terrain alone took run20's path-stack
disagreements from 21 to **24**, fog alone to 12 — and the pair took them
to **0**, nine waypoints reproduced entry for entry. The reason is in the
numbers: a scout's base is `8` unseen against `0x400` seen, a factor of
128, so *whether a cell is known* and *what it costs once known* multiply
rather than add. Pricing one without the other prices nothing.

**`tcost` is a running count of the cell's blocked tiles, and one function
keeps it** (2026-09-01, run55; the queue's item 125). `WData.blocked`
(`+0x11`) is not a property the map generator writes once: it is *how many
of the cell's sixteen tiles carry `TData.mask & 0x4000`*, and
**`World::set_blocked_at@006b4900`** is the only writer. Every caller in
the executable goes through it — `BuildType::mask_me@006312a0` for a
building's footprint, `Mountains::add_mountain`, `World::set_cliff_at`,
`Good::init`, `TerrainGroup::drop_tile`, `SpellType::cast_pack` — and it
does three things at once, each guarded so that setting an already-set bit
costs nothing:

- when the tile's `0x4000` actually changes, the cell's `blocked`
  **and** `solid` (`+0x13`) step together — a building stops a
  forest-walker as surely as anyone, so the forest's own
  `blocked > 0, solid == 0` is written elsewhere;
- the tile's own `0x2000` is cleared as it becomes blocked (and its road
  cleared, `set_road_at(t, 0, 0, 0)`), restored on unblocking if
  `WorldData::has_blocked_neighbors@006b2990` still says yes;
- all eight neighbours take `0x2000` — the "next to something blocked"
  bit the tile grid charges `0x400` for, three rows down — and lose it on
  unblocking only when they have no other blocked neighbour left.
  `WData.bad` (`+0x12`) counts that bit per cell the same way.

So a building raises the *pathfinder's terrain cost* of every cell it
stands on, by the number of tiles of that cell it covers. That is the whole
of §12's frame-1476 scout: a city with nine of a cell's sixteen tiles under
it charges `20 × 9 − 4 = 176` on top of a scout's 128, and opens §5.1's
corner-cutting gate, and a simulation that keeps the bit without the count
charges nothing and walks through the city.

**Tiles (`0xc0`)** read the tile mask instead: `base = 0x100`, `0x400` if
the tile has `0x2000` (rough); danger and the ±4 owner terms as above (the
cell containing the tile); `+ 4000` for a tile whose mask has bit 14 set
with `(mask & 3) == 3` — the passable-building (gate) tile, the same
predicate reconstruction marks `building` by. No terrain/army/scout terms.

**The transport tail (all grids)**: `needs_transport(from tile, to tile)`
returns 0 (no shoreline crossed), 1 (water → land), 2 (land → water). If
crossing and the unit can transport (`unit_masks & 0x800000 && !(unit_masks2
& 0x2000)`, or `unit_flags & 0x10`) and `depth ≥ 2`: `avoid_sea == 2` →
refuse (`0x7fffffff`); embarking (2): `extra += 500` if both avoids are 0
else `+= 2000`; **the unit-grid `extra ×= 4` sits outside that guard** — a
disembark skips the 500/2000 and still takes the shift (audit V21); set
the node's `transport` flag. At `depth == 1` the same test runs **from the
wrapper's start tile** (`pathfinder +0x58/+0x5c`, written from `find_wpath`'s,
`find_tpath`'s and `find_upath`'s start argument — a group's plan starts
elsewhere than the unit stands; group 7) with `+250`/`+1000` and no shift. A unit that
cannot transport pays nothing here — the water itself was already priced.

### 5.1 Corner-cutting (**`0x300` only** — the tile branch jumps clean over
it (audit V4); `depth < 10`, and only when `tcost ≠ 0` or `iroquois`)

The step's **world-cell** delta (`div3[(to−from)>>8]`, on every grid) is
matched to its wheel direction, and 2–4 `is_blocked_at` probes run on
tiles around the *from* tile; `is_blocked_at(t, mode)` = tile mask bit
`0x4000` (building), except `mode ≠ 0` (iroquois) exempts full forest
tiles (`mask & 0x30 == 0x30`). **The probe table is settled** — byte-
verified in the listing during the audit (V3; the jump table at
`0x6858dc`, all eight arms), superseding the first reading's distrust.
With `cx, cy` the from tile:

| dir | probes | refuses when |
|---|---|---|
| 1 NW | `(cx−2,cy−2)  (cx−3,cy−3)` | **either** blocked |
| 2 N | `(cx−2,cy−3)  (cx−1,cy−3)  (cx,cy−3)  (cx+1,cy−3)` | **all four** blocked |
| 3 NE | `(cx+1,cy−2)  (cx+2,cy−3)` | either |
| 4 E | `(cx+2,cy−2)  (cx+2,cy−1)  (cx+2,cy)  (cx+2,cy+1)` | all four |
| 5 SE | `(cx+1,cy+1)  (cx+2,cy+2)` | either |
| 6 S | `(cx−2,cy+2)  (cx−1,cy+2)  (cx,cy+2)  (cx+1,cy+2)` | all four |
| 7 SW | `(cx−2,cy+1)  (cx−3,cy+2)` | either |
| 8 W | `(cx−3,cy−2)  (cx−3,cy−1)  (cx−3,cy)  (cx−3,cy+1)` | all four |

A refusal is `0x7fffffff` — you cannot cut a diagonal past a building
corner, and a cardinal step is refused only when the whole destination-
side tile edge is built over. On zero-cost terrain (all of
`gamelog-run6`) the block never runs at all, which is why it sat behind
the terrain-cost seam. **Implemented 2026-08-26** as
`Sim::cuts_a_corner`, from this table, with the direction recovered from
the step's own world-cell delta rather than from the wheel index — a big
unit's two-cell stride matches no `move_x` entry, so the whole block is
skipped for it, which is the original's behaviour and not an accident of
transcription. No capture on disk exercises it (§12).

## 6. Validity, the memo, and the heuristic

**`UnitData::invalid_loc(tx, ty, p3..p7)`** (`@00607c30`; audit V11
verified every clause): the returns are `0` valid, `1` off the map, `2`
terrain/domain refusal, `3` a sea unit over shallows/river (`tile &
0x2400`) — the code `valid_wcoord` forgives in the goal cell — and `4`
blocked by a building. The flags: `p3` skips the building check; `p4`
enables the fog shortcut — a flag-4 (**human**) leader's probe returns
valid when **all four** fog half-cells of the tile's cell are unseen; `p5`
lets an armed unit pass ~~its own side's buildings~~ **a building that is
not its own**, and a built tile where none is found (`find_any_building_at`
owner test, `607fb6`..`60800a`: `find_who == who` → 4, else 0; item 1209,
`docs/GOLDEN.md` §51) — the tile grid passes it, and `astar_path` flags
such a node `0x10` for `Unit::resolve_block`; `p6`/`p7` relax the water refusal for a transport-forced unit
(`unit_masks & 0x800000`), and `p6` is also forced on when the unit's path
top carries flag 4. Dispatch is on the type's domain (`+0x218`: 0 land, 1
sea, 2 air — air is always valid); the land arm refuses forest (except
forest-walkers, `unit_masks2 & 0x4000`), mountains, cliffs and water, and
**first**, when `p3` and `p6` are both set, a tile whose **cell** carries
`WData.flags & 0x70` (mountain, forest, `0x40`): that is `valid_wcoord`'s
probe, so the world search refuses a forest *cell* (§27; ~~the tile
alone~~ was this crate's reading until item 776); the
shared tail refuses a `0x4000`-blocked tile unless the unit itself stands
on one. An eighth argument exists at every call site and is never read —
a stale register.

- **`valid_wcoord(p, timeout, goal)`**: refuse the unit's own
  `avoid_x/avoid_y` (the point `find_path` recorded as unreachable —
  `docs/ORDERS.md` §4.5); then `invalid_loc(tile of p, 1, timeout > 1, 0,
  1, 0)`; a verdict of exactly **3** is forgiven when `p` is in the goal's
  cell. Note the first two steps of a world search are probed at `p + toff
  − 0x180` — the cell corner when there is no move order current, the
  order's own sub-cell point when there is (§4.1) — later steps at `p`
  itself. Unlike §7's, this one is not gated: `toff` is simply zero.
- **`valid_tcoord(p)`** (pre-walk only; the search inlines it):
  `invalid_loc(tile, 0,1,1,1,0) == 0`.
- **`valid_ucoord(p, metric)`**: bounds (`0 ≤ p < tiles × 0xc0`), then the
  **memo**: `validlist.seek(metric)` hit → `valid_hit += 1`, return the
  cached verdict; miss → `invalid_loc(tile, 0,1,0,1,0) == 0 &&
  detect_unit_collision(p, 1, 1, …, 1, 0) == 0`, cached under `metric`.
  The unit grid is the only one that sees other units, and it sees them
  through a per-search cache.
- **Heuristic**: `h = vector_dist(node → goal) * 10` on the unit grid,
  `* 60 / step` otherwise (`get_estimate@00688310`). Per step of the grid
  that is **60** (world/tile) or **480** (unit) against a `g` of 32–40 —
  the heuristic overweights distance ~2× (unit grid ~12×), so the search
  is deliberately greedy/beeline-biased, not admissible-A\*. Replicate,
  do not "fix".

## 7. Reconstruction

From the stop node `n` (arrival or budget): the walk-back root is `n`
itself, **except** a `0x300` arrival without transport flags uses
`n.parent` — the last cell centre before the goal is redundant since the
final goal is already on the stack below. On `0xc0` first: every chain node
whose tile is a gate (`mask & 3 == 3` and bit 14) gets `building = 1`, and
the root moves to the **deepest** such node — which means the emitted path
is **truncated** there: the unit is routed only as far as the gate nearest
it (audit V24). Then up the parent chain, for
each node push one `PathData`:

- position: the node's, plus — when the current order **`is_move()`**
  (§4.1's `toff`, re-read here rather than carried) — `toff − 0x180`
  (world) or `toff % 0xc0 − 0x60` (tile). The unit grid never offsets, and
  neither does any grid when the order is not a move: the whole expression,
  the `− 0x180` included, is inside the `is_move` arm;
- tolerance: `0x180` (world) / `0` (unit grid) / tile: `0` if the unit can
  transport and `anti_unit == 0`, else `0x60`; forced 0 on a
  transport-flagged node. The tile arm's own test is at `00684bfc` and it
  is the unit's, not the type's alone: `unit_masks & 0x800000` without
  `unit_masks2 & 0x2000`, **or** `unit_flags & 0x10` (`+0x2b4` bit `e`,
  the sea transport). So the same type walks with two different
  tolerances over one game — a citizen is granted `0x800000` the frame
  after its side's Dock finishes — and the difference is worth three
  frames a leg at speed 25, because the caller's arrival test is
  `manhattan(waypoint − step) <= tolerance` (`docs/ORDERS.md` §4.5);
- flags: `2` if `anti_unit` or unit grid; `| 0x10` if **this node**
  carries `building`; `| 4` if **this node** carries `transport` — both
  per emitted node, not from the walk-back root (audit V23);
- skip a push equal to the stack's current top; the start node itself is
  pushed **only on the unit grid**.

Return 1. The wrapper then reports the stack's length, and `kill_lists`
frees every node (closed, open, tombstones) back to the recyclers.

## 8. `repaths` — the pressure valve

`GameDaemon::repaths[who]` (`GameDaemon +0x0`, 8 slots): halved every frame
and snapped to zero below 3 (`GameDaemon::process_all`), and squared into
`find_upath`'s `limit`. The writers survey refines `docs/ORDERS.md` §4.6's
sketch: it is incremented at `Unit::do_move` (one frame in four per unit)
and in `resolve_unit_collision` (saturating at 16, throttled to 1-in-16
above 8), and read back outside the pathfinder by `do_move` (non-zero
forces a gatherer to re-target) and `resolve_unit_collision` (above 3 the
collide threshold rises `0x20 → 0x80`). Zeroed by `GameDaemon::init/close`
and `World::close`; an OOS recovery calls `PathFinder::close`, wiping every
mode field.

**The decay is the load-bearing half, and it went unmodelled** (item 80,
2026-08-30). Without it the counter is a lifetime tally: this crate's
`repaths[1]` reached 5 by run33's frame 500 and never came down, which put
`resolve_unit_collision`'s throttle permanently into its `≥ 4` arm and
threw away three collisions in four — including the one at frame 571 the
original repaths and staggers (`docs/COLLISION.md` §6 step 6, §8).
`Sim::tick` now runs the halving where `Game::do_frame` runs it: first
thing in `GameDaemon::process_all`, before the vision sweep and the market,
and before any unit steps. The `500 / repaths²` limit is downstream of the
same fix — a decayed counter is 0 or 3 on almost every frame, so the
squared divisor is the one the original uses.

## 9. `Unit::find_path`'s march — the §4.6 question, settled

The fixed point that hung `crates/sim` (2026-08-22) came from a
mistranscription, and the listing question ORDERS §4.6 left open is now
answered from `find_path@005fb910`'s own body:

1. **The angle is recomputed from the current remainder every iteration** —
   `find_angle(dx, dy)` is inside the loop. The transcription hoisted it.
2. **Each component is clamped to its axis' remainder**: `|sinx(ang, spd)| >
   |dx| → step_x = dx` (and the y mirror). The march can never overshoot an
   axis, and once an axis closes, the recomputed angle points wholly along
   the other.
3. The exits: enter/continue the loop only while `spd < manh`; inside,
   return 0 if `manh` **grew** since the previous iteration, or — after
   stepping and recomputing — if `|dx| ≤ |step_x|` **and** `|dy| ≤
   |step_y|` with the *actual clamped components* just used (`<=`, not
   `<`). All three exits return 0 (walk).

With (1) and (2) the no-progress state is unreachable for `spd ≥ 3` (the
dominant component of `sinx/cosx` at `spd ≥ 3` is ≥ 2 after truncation), so
the original needs no progress guard, and the citizen case that hung us —
`(dx, dy) = (60, 1600)`, `spd = 25` — walks: y closes exactly, the angle
re-aims due east, x closes, return 0, **no `find_wpath` draw**. The sim's
"return 1 on no progress" guard is retired with the march rewritten this
way. (`docs/ORDERS.md` §4.6 amended in place, pointing here.)

## 10. The oracle

- **The per-frame path-stack diff is the oracle**, as the brief planned:
  `UNITS=3` logs each unit's stack bottom-first, the harness reads it, and
  `OrderMismatch::PathLength`/`PathTo` **score** as of this landing
  (`crates/rondata/src/diff.rs`, `order_ticks_before_divergence`).
- **The grade, on `gamelog-run6` (432 frames):** the first woodcutter's
  citizen (`0/1`) matches position, orders and path stack for **all 432
  frames**; the second (`0/2`) — the stub's visible gap, `PathLength` from
  frame 4 — now matches the original's **path stack for 427 straight
  frames**, its first disagreement of any kind being an order-list `Length`
  at frame 428. The global tallies did not move (750 `path-length`, 33
  `path-to`) because every remaining path disagreement is on **player 1's
  mirror units, whose straight lines cross forest**: the level-0 dump
  carries no terrain, the harness's world is flat grass, so its marches
  verify lines the original's `find_path` refuses at the tree line and
  detours or plans around. The next path-diff improvement is therefore a
  **world-data item** — get the map's forest tiles into the harness — not a
  search item. Both facts are pinned in
  `the_original_s_own_run_is_still_matched_frame_for_frame`.
  **The forest tiles are in the harness as of 2026-08-24** — a `WORLD=6`
  start dump carries every tile's `TData` mask (`docs/ORACLE.md`, "The map
  is a dump too"; run9 for this lobby), and `build_sim` loads them. On
  run9's 36 frames the path-stack disagreements fell 84 → 68 and the order
  disagreements 60 → 43, but `0/2`'s frame-4 divergence stands with the
  real forest under it — so what remains there is the pathfinder's (the
  open items in §11), not the map's. Run10 (run7's length, the map) is the
  next measurement.
- ~~**There is no numeric per-search oracle to switch on**~~ — **there is
  one now, and it is not a logger** (2026-09-01, run55). The survey below
  stands as a statement about the *game*: the `PATHFINDER` gamelog category
  (index 29, threshold ≥ 10) emits exactly one line — `astar_river`'s seed
  at map generation; `dbg_tree_depth` computes node counts but prints to an
  on-screen window gated on `show_debug`, which nothing writes;
  `dbg_draw_failures` is an empty function; `PathFinderData::log_data` (17
  mode flags) is reachable only via `DUMP_ALL=1`, which hangs the game
  (`docs/ORACLE.md`). What changed is the instrument: `tools/trace` now
  **proxies** a chosen function — logs its arguments, calls the original
  through the displaced-prologue trampoline, and logs `eax` — so
  `calc_cost`'s answer is readable even though nothing prints it.
  `rontrace.cfg`'s `callwin=LO-HI` switches it on;
  `PathFinder::astar_path`'s own entry and return delimit one search;
  `report.py … calls` and `rondata::trace::Call` read it.
  **run55 is the first**: run39's lobby and seed, `callwin=1460-1490`,
  and `rngcmp.py` says its word is run39's on all 1,501 overlapping frames
  with zero differing — so the proxies cost the simulation nothing.
  In that whole thirty-one-frame window the game ran **one** search, the
  scout's, 110 `calc_cost` calls on sim-frame 1476. 103 of them already
  agreed with this crate; the seven that did not were all steps into the
  four cells under a city, and are §5's `tcost` note above. With that
  landed the two searches are **identical call for call** — same
  arguments, same answers, nothing extra on either side
  (`diff::tests::run55_s_frame_1477_prices_are_the_originals`, which
  compares by argument list rather than by position, so a disagreement
  there is the formula and never the search order).
- **One free number**: `UNITS=3` already prints `start_dist` — §4.1's
  start-to-goal Manhattan, stashed on the unit — so every logged search
  hands over one checkable value with no new capture.
- **The record is diffed whole as of 2026-08-31.** `PATHDATA` prints four
  numbers and the comparison read two of them: `OrderMismatch::PathField`
  now scores `tolerance` and `flags` beside `PathTo`'s point, and
  `diff::tests::a_path_stack_s_rows_are_compared_whole` is the guard. No
  floor moved when it landed — of run39's 24 disagreeing rows the first is
  frame **1518**, past that capture's score — but the census it prints is
  itself an oracle. Over run39's 1,724 multi-entry stacks the bottom is
  `(tolerance 0, FINAL)` **every time** (`Unit::do_move` pushes
  `{mo->x, mo->y, 0, 1}` before it plans, `docs/ORDERS.md` §4.4; the
  group's own push is `docs/GROUPS.md` §6.7), and above it there are
  exactly three shapes and no others: `(384, 0)` 1,408 tops and 2,325
  middles, §7's world reconstruction; `(0, 2)` 73 and 54, the unit grid's
  `SIDESTEP`; and `(0, 0)` 243 tops, §4.4's collision rewrite —
  `tolerance = collider.big_radius × 3`, which is zero for every
  `BLOCK_RADIUS 1` type in the corpus — which this crate already models
  (`orders.rs`, the waypoint block). The counts are the dump's own, so
  they are pinned exactly rather than as a ceiling.
- **And the row that census could not hold was the tile grid's own**
  (2026-09-01, item 141). run39 is 1,850 frames and East Indies' AI Dock
  finishes on 3579, so no unit in that capture ever satisfies §7's
  transport test and every tile waypoint it plans is `0x60` on both
  sides — the check was right and the capture was short. On run58 the
  same comparison opens **1,750 rows**, the first at frame 3644,
  `tolerance ours 96 theirs 0` on the AI's citizen `1/13`; it is now
  asserted **empty before the word** in
  `diff::tests::run58_s_five_thousand_frames_stand_where_the_original_s_do`.
  The lesson is the capture, not the comparison: a widening's census is
  only as wide as the frames it ran over, and a rule that switches on a
  mid-game grant needs a capture that reaches the grant.

## 11. What the sim implements

`crates/sim/src/path.rs`: the search of §4–§7 on the three grids, the
containers replaced by deterministic equivalents that preserve **min-value
with LIFO ties** and metric-keyed dedup; the world read through the layers
the sim has (tile mask bits via `place.rs`, cell owner via `territory`,
`tregion`, the `WData` records and the `seen2` fog plane via the `WORLD`
dump), with named seams returning the open-ground answer for the layers it
does not (rush rules: off) — each seam marked in the code with the §5 term
it stubs. **Two of those seams closed 2026-09-02**: `danger` has a writer
(`crate::danger`, `docs/DANGER.md`) and `diplos` is installed from the dump,
so the danger term and the enemy-territory `+4` are both live.
**The cliff seam closed 2026-08-26** with `docs/SCOUT.md`:
`WorldData::is_cliff_at@0046f8c0` is one line, `(TData.mask & 3) == 1`, so
the two-bit terrain-object field's third value is named
(`world::tile::OBJECT_CLIFF`) and `invalid_loc`'s land arm refuses a cliff
as the original does.
`find_path`'s march rewritten per §9, and `do_move`'s two planner call
sites wired per `docs/ORDERS.md` §4.4 (the fresh-move `find_wpath` with
tile fallback; the RNG-thresholded re-plan). `invalid_loc` is implemented
with the original's five flags and return codes over the sim's tile masks
— which surfaced a placement bug: `mask_building` was marking **flat**
gatherers' footprints `BLOCKED`, where the original's `mask_me` only
blocks tiles the type's mask template marks, and citizens stand on farms.
`toff` is read from the current order (§4.1) rather than stubbed —
`Sim::toff`, which is `None` when no move order is current and the offset
is then not applied at all.
`find_wpath` is split in two: `Sim::find_wpath(u)` is the four-argument
overload at `00688e10` — the object's own position — and
`Sim::find_wpath_from(u, here, army_hint)` is the six-argument one at
`00688fc0`, which takes the stack and the start point. Only
`Group::action_move_near` calls the second (`docs/GROUPS.md` §6.7): it
plans on a stack that is not a unit's, from a start that is the leader's
top-of-stack, and it forces the `army` mode on through `pathfinder +0x70`
for a group that belongs to an army — the same word §3's mode block sets
for an AI's own units, and the only way a **human**'s army reaches it.
**The fog and terrain seams closed 2026-08-26** (the queue's item 32), and
with them everything that hung off a cell record: `+20 × WData.blocked`
(`WData.solid`, signed, for a forest-walker), the `+100000` at 13, the
army's `+10000` at 5 and its `base << 5` on a `NEARBLOCK` cell, the fog
branch's `0x124` / `0x2480` / scout's `8`, the halfland `base ×= 3`, and
§5.1's corner-cutting probes — which were only ever unreachable because
`tcost` was pinned to zero. What holds them is twelve deliberate
breakages, listed in §12; the last of them, the half-cell fog convention,
is held by a unit test alone because **no capture on disk separates it**
from the `2c + 1` read: every world-grid node in every dump sits past the
half-cell line, so `to / 0x180 == 2c + 1` throughout.
The transport tail's shoreline result is now threaded to the halfland
multiplier the way `00685773`–`006858b9` threads it — the `depth == 1`
probe overwrites the `from → to` one, and the multiplier is gated on
whichever ran last.
~~Big-unit strides were dormant~~ — implemented and diff-backed by item 1431 (§30); the transport tail remains as documented;
suspend returns −1 without stashing (its restorer has no caller until
collision recovery exists); `find_upath` is complete and tested, and
**`resolve_unit_collision` now calls it** (`docs/COLLISION.md` §6 step 6),
which is also where `valid_ucoord`'s `detect_unit_collision` half came
alive: the 48-grid plan goes round the units in the way.

## 12. What is not established

Four of the first reading's open items were **settled by the audit**
(`docs/audit/2026-08-23-pathfinder.md`): the corner-cutting probe offsets
(§5.1's table, byte-verified, V3), the no-rush guard's third clause
(`rr < 9 ||`, V6), the region-crossing `avoid_land` store (V25) and the
`vector_dist` operands (V26). Still open:

- ~~**`toff` is not only a unit target's, and the simulation's zero is
  wrong.**~~ **Settled and implemented 2026-08-26** — §4.1 has the two
  vtable slots and their folded bodies, §7 the gate. It was never a
  target's: `toff` is the *current move order's* `off_x/off_y`. Run20's
  unit `1/0` now walks the original's own chain, `cell*0x300 + 504` on
  both axes, and three of its five world nodes are the original's entry for
  entry (`diff::tests::run20_s_world_chain_sits_on_the_move_orders_own_
  offset`, made to fail on purpose twice — once with `toff` pinned to zero,
  once with the offset removed altogether).
- ~~**The goal at the bottom of that stack is `0x18` short of the
  order's own.**~~ **Not the pre-walk** (2026-08-26, settled by reading;
  the implementation is the queue's item 31). `find_wpath`'s pre-walk
  cannot produce it in either variant: the AI walk steps by `0x180`/`0x30`
  through `sin_table`, never `−0x18` on both axes at once, and the human
  variant only pops. Nor can `Unit::do_move`, which pushes
  `{mo->x, mo->y}` verbatim — and `add_move_facing_order` writes
  `x = u*0x30 + 0x18`, so an `mo->x` is *always* `≡ 0x18 (mod 0x30)` and
  `41952` is `≡ 0`. What actually pushes it is
  **`Group::action_move_near@00704990`**: the group plans one path of its
  own on the global `grouppath`, and the goal it pushes is the **raw slot
  destination** `{slot_x[leader], slot_y[leader]}` read out of the form
  table at `form+0x514`/`form+0x714` — un-snapped — with the members'
  stacks filled from it. The member's own order carries the same point
  snapped (`x = 41976`) and the group's destination in `orig_x/orig_y`
  (`41952`), which is why all three numbers line up on a group of one.
  ~~Two consequences the simulation does not have yet, both item 31's: the
  goal, and the fact that the path exists **at order time**.~~
  **Both landed 2026-08-26** — `crates/sim/src/grouppath.rs`,
  `docs/GROUPS.md` §6.7 and §12.4. Run20's `1/0` now carries
  `(41952, 36576, 0, 1)` at the bottom of a stack it already has at frame
  1, with `flags 1`; `rondata --diff` scores run20 at **0** order
  disagreements where it scored 1.
- ~~**The middle of run20's `1/0` chain still parts.**~~ **Closed
  2026-08-26** (the queue's item 32), and it was neither the direction
  wheel nor a tie: it was `calc_cost` not reading the two things the data
  layer had been able to answer for weeks. The sim's first step was
  diagonal to `(50,51)` because nothing charged it the `+180` that cell's
  `WData.blocked = 9` costs; the row it then walked was row 51 because
  nothing told a scout that row 50 was dark and therefore nearly free.
  With the fog read and the terrain cost both live, **all nine entries
  agree, in order** — position, tolerance and flag — and `rondata --diff`
  scores run20 at **0** path-stack disagreements against 21, with the
  whole order stream matched over the five frames it steps. The chain is
  the assertion now
  (`diff::tests::run20_s_group_member_is_pathed_at_order_time_off_the_
  leaders_slot` compares it whole). See §5's note on why the two terms
  had to land together.
- ~~**The fog the sim reads is the frame-0 snapshot.**~~ **Closed
  2026-08-27** — `docs/VISION.md` and `crates/sim/src/vision.rs`. A unit
  now lights a disc of `LOS / 2` fog cells every time it crosses a
  half-cell, centred (below radius four) a half-cell ahead of its own
  facing, and `Object::update_seen`'s hundredth-frame resync runs from
  `tick`. `seen2` grows as the game runs, so a path planned at frame 100
  is planned against what its planner has actually seen.

  **And the run10 row this entry blamed on the fog was not the fog's.**
  Frame 102 went 24 → **22** with the reveal live and stopped there. The
  trace says both sides give AI scout `1/0` the same `EXPLORE_TO` and walk
  it to the same point; the original turns in one frame on frame 62 and
  this simulation stands for seven and then eases for eight, arriving six
  frames late and spending its `think_scout` ring draws a frame after the
  original spent them. It is `docs/MOVEMENT.md`'s stopped-unit instant
  turn, and it is the queue's now. `docs/VISION.md` §8.
- **The half-cell fog convention is unfalsifiable by any run on disk.**
  `calc_cost` reads `div_3_table[to >> 7]`; `Unit::think_scout` reads
  `2c + 1`. Every world-grid search node in every capture sits at a
  sub-cell offset past `0x180` — run20's whole chain is at `+504`, its
  start at `+408` — so the two readings answer identically on all of them,
  and swapping one for the other moves not a single number in run20 or in
  the 301-frame fuzz map. Only
  `path::tests::the_unseen_is_cheap_to_a_scout_and_the_read_is_the_half_
  cell` separates them. A capture that would: any world path whose unit
  stands at a sub-cell offset under `0x180` on either axis.
- **The corner-cutting probes and the halfland multiplier are implemented
  and unexercised by any capture.** Removing either moves nothing in run20
  or the fuzz map; both are held by unit tests written from §5.1's table
  and from `006858b9`. §5.1's own open question — what the gate bits mean
  on the ground — is unchanged below.
- The exhausted-open-list pause gate `order data +0x20 < 13` is
  byte-verified (V30); **which field that is** (order age? range band?)
  remains unread and unnamed in the PDB.
- The large-unit diagonal sub-probes memoise `validlist` under keys in
  **world units** — a different key space from the lattice metrics the
  rest of the search uses, so they can collide (V8; listing-confirmed,
  and almost certainly an original bug). Dormant at stride 1; whichever
  way it is replicated must be a deliberate decision when big-unit
  strides go live.
- What the gate bits (`mask & 3 == 3` + bit 14) mean on the ground — a
  behavioural check with a wall-and-gate scenario would settle §5.1's and
  §7's interpretation (blind-a's request, V24).
- `PFD.can_transport` (+0x58) and the three `dbg_*` fields have no writer
  or no reader in scope; treated as dead.
- Region identity (`get_tregion`) is taken as `docs/ORDERS.md` §4.6 had
  it; the region map's own construction (`WorldData`) has not been read.
- `find_wpath_army` (zero callers — `Group::action_move_near` inlines its
  body), `find_road`/`calc_road_cost`/`valid_roadcoord`, `astar_river`:
  named, out of scope, unread beyond signatures.
- The Nubian/attrition and garrison items in `docs/CITIES.md`/`ATTRITION.md`
  are untouched by this reading.
- `0/2`'s **position** still parts from the original at frame 4 by `(2, 8)`
  units even though its path stack now matches — the waypoint is the same
  and the step toward it differs, which points at the movement layer
  (`docs/MOVEMENT.md`'s step/turn interplay), not the planner. Pre-existing,
  unchanged by this landing.
- ~~**The AI scout's `EXPLORE_TO` world path on run39's frame 1477**~~
  **Closed 2026-09-01** (the queue's item 125), and it was neither the
  heuristic nor a tie: it was **`WData.blocked` — a count of the cell's
  blocked tiles — that nothing in this crate was keeping**. §5's note on
  `tcost` has the mechanic and `World::set_blocked_at@006b4900`;
  `crates/sim/src/world.rs` has the implementation and
  `Sim::mask_building` is its only caller so far.

  The reading that had been carried into this entry — that the original's
  route was the *cheaper* one under this crate's own costs, 661 against
  672, so two sides could agree on every step's price and still return
  different routes — was **wrong in its premise**. They did not agree on
  every step's price. run55's per-step dump (§10) put the two side by
  side on the frame itself, and of the original's 110 `calc_cost` calls
  103 already matched; every one of the seven that did not was a step
  into one of the four cells under player 1's second city at tile
  (180, 188) — `(44,46) (45,46) (44,47) (45,47)`, nine of each cell's
  sixteen tiles built over. The original charges `128 + 20×9 − 4 = 304`
  to enter one and refuses `(45,46) → (44,47)` outright, §5.1's
  corner-cutting having a non-zero `tcost` to open on at last; this crate
  charged 128 and refused nothing, so its search went **through** the
  city. The six things §12 had ruled out were all correctly ruled out —
  fog, danger, the buildings' positions, the cell records, the estimate,
  the stop test — and the seventh, the one nobody had thought to name,
  was that a cell record can *change*.

  With the count kept, the frame-1476 search is the original's **call for
  call**, and run39 as a whole is matched end to end: 1,851 ticks of
  1,851, 1,850 order-frames of 1,850, **neither player diverging
  anywhere**, and the word running to the end of the capture. The
  successor is the long capture: run54 puts East Indies' word at
  **2176**, on a gathering building's own survey (the queue's item 85).

  **The lesson, which is the audit README's own and cost a month here:
  grep the writers of every field you call frozen.** The terrain cost had
  been read, implemented, audited and diffed, and every one of those
  passes treated `WData.blocked` as a property of the map because the
  frame-0 dump it was loaded from is a map. One `grep` for the field's
  writers names `set_blocked_at` in a second.

## 13. The pull-back's step, and the fold that must not be applied twice (2026-09-01)

All three pull-back walks — `find_wpath@00688fc0`, `find_tpath@006897d0`,
`find_upath@00682f30` — decompile to the same four lines:

```
angle = find_angle(dx, dy);
if (angle < 0) step = -step;
x += sin_table(…);
y -= sin_table(…);
```

**That `if` is not the caller's.** It is the compiler's inline of `sinx`'s
own sign fold — the one `movement::sin_component` already performs — and
the decompiler prints it at the call site because `sin_table` takes the
*folded* angle and a signed distance. The listing says so plainly. At
`0x6894c3`, immediately after `find_angle` returns:

```
6894c3  mov  edi, [ebp-0x20]      ; edi = the step, unsigned
6894c6  mov  ecx, eax             ; the angle
6894cb  mov  eax, edi
6894d2  test ecx, ecx
6894d4  jns  6894e1
6894d6    neg  eax                ; ← the SINE call's distance only
6894d8    and  edx, 0x7fffffff
6894f5  call sin_table            ; x += sin_table(folded, ±step)
6894fa  mov  edx, [ebp-0x34]      ; the angle again, unrotated
6894ff  add  edx, 0x40000000      ; a quarter turn
68950b  jns  689515
68950d    neg  edi                ; ← the COSINE call's own fold, on `edi`,
68950f    and  edx, 0x7fffffff    ;   which was never negated
689528  call sin_table            ; y -= sin_table(folded2, ±step)
```

The cosine's distance is reloaded from the **un-negated** step and negated
again only on the sign of `angle + 0x40000000`. That is exactly
`sin_component(angle, step)` and `cos_component(angle, step)` — two
independent folds, one per call — and there is no caller-level flip at all.

**Doing it twice cancels it.** This crate flipped the step *and* handed the
flipped step to `sin_component`, which flipped it back, so for every angle
in the western half the step ran the wrong way: the goal walked *away* from
the start instead of toward it, the region test never matched, and the loop
did not terminate. It surfaced as an `i32` overflow in `find_angle` after
~200 iterations, once the goal was far enough out that `lo * 0x4000` no
longer fitted.

**Why no capture caught it.** The walk only runs when the goal's tile
region differs from the start's, and every such call on every capture on
disk matched on its **first** test and broke out before the body ran once.
The body first executed on 2026-09-01, when the AI's Dock moved two cells
south onto a coastal cell whose tile region is not its builder's
(`docs/AI.md` §21) — and then it ran forever. With the fold removed it
converges in one step: `(43896, 41400)` → `(43593, 41163)`, region 12,
which is the start's.

**And the step constants, from the same two listings.** The step is the
same number as the walk's own give-up threshold, and it is not `0x30`
everywhere:

| walk | give-up test | step | where |
|---|---|---|---|
| `find_wpath` | `0x180` when Manhattan ≥ `0x300`, else `0x30` | the same | `0x689487`, `0x6894a8` |
| `find_tpath` | `0x60` on both axes | **`0x60`** | `mov ebx, 0x60` at `0x6899da` |
| `find_upath` | `0x18` on both axes | **`0x18`** | `mov edi, 0x18` at `0x68318b` |

This crate had `0x30` in the last two. Neither moves any number on any
capture on disk — for the same reason the fold did not — but both are now
what the listing says.

**Coverage.** Reading-only, and by the listing rather than the decompile:
every claim here is a byte at a named address. No capture exercises the
body more than once, so none of it is diff-backed; what a run *does* pin
is the consequence — with the fold removed, run56's 3,000 frames still
stand at zero position disagreements for every unit but the dock's builder
(`run56_s_collision_block_agrees_past_the_scored_length`).

## 14. The pull-back's gate — who is allowed to keep the goal (2026-09-01)

§13 settled how the pull-back *steps*. This is the test in front of it,
and it is the whole of transport pathing.

`PathFinder::find_wpath@00688fc0` reaches the walk at `00689375` only for

```
type->domain < 2  and  (!is_on_map()  or  !UnitData::can_transport(this))
```

so **a unit that can board skips the pull-back entirely** and hands
`astar_path` the goal it was given. The human branch above it (`leaders &
4`) guards its own, different walk with the same `is_on_map &&
can_transport` pair.

Why it is load-bearing: the walk steps the goal toward the unit until
`get_tregion(goal) == get_tregion(here)`, and for an island target the
first region it matches is the unit's **own island**. A land unit that
cannot board is then asked for a route to a point on its own coast, which
is right — there is nowhere else it can go. A unit that *can* board and is
put through the same walk is asked for the same thing, plans a route to
its own shore, and never crosses. That is exactly what this crate did on
the day the AI first pointed a scout at another island: `find_wpath`
returned a one-entry partial at the near shore, `do_move` re-planned on
the tile grid the next frame and spent the grid draw
(`Unit::do_move+0xe84`) the original does not, and East Indies' word fell
3608 → 3585. With the gate the eleven-waypoint route this crate plans is
the original's, entry for entry (`docs/TRANSPORT.md` §7).

**Not modelled**, and stated here rather than in the code: the walk's own
break test has a second clause for a **sea** unit — the matched region
must also pass `invalid_loc(t, 0, 1, 1, 1, 0)` — which this crate does
not make. No boat in any capture has re-planned from inside the pull-back.

## 15. The pull-back asks `get_tregion`, and the sim was asking the other one (2026-09-01)

**Amended by §16 the same day.** The fourth site is *half* a site: line
395 is `astar_path`'s `param_2 ≠ 0x300` arm alone, and the world grid
takes an inlined arm that reads `WData.region` and calls nothing. The
three pull-back walks below are unaffected.

All four region reads in the pathfinder — the three pull-back walks
(`find_wpath@00688fc0:104`, `find_tpath@006897d0:90`,
`find_upath@00682f30:116`) and `astar_path@00683770:395`'s
`avoid_land`/`avoid_sea` derivation — call **`WorldData::get_tregion`**,
which is `crate::world::World::tregion_alt`: a cell with `flags & 0x100`
whose *tile* is ocean answers its `region2`, the sea region, and only
otherwise its `region`. `crate::world::World::tregion` is the plain
`region_of(cell_of_tile)` and is a different function.

This crate asked the plain one at all four sites until 2026-09-01, and it
cost the score the day a boat first pathed. The AI's Fisherman `1/14`
stands on cell (57, 55) of East Indies — `SANDY`, `flags 0x104`,
`region 11` (land), `region2 65` (sea) — and is sent to (49, 39), which is
region 65. With `tregion` the start answered **11**, the pull-back's break
test never matched, and the walk dragged the goal three quarters of the way
back to the boat before the give-up exit fired; the whole route was
planned to a point the boat had not been asked to go to. With
`tregion_alt` the loop exits on its first test, as it does for every unit
that stands where it looks.

The remaining `World::tregion` callers — `army`, `group`, `orders`,
`roads`, `place`, `scout`, `transport` — are still unchecked; the queue
carries them.

## 16. The same-region test is two functions, and the world grid's is the raw field (2026-09-01)

§15 read `astar_path@00683770:395`'s `get_tregion` pair and took it for
the whole `avoid_land`/`avoid_sea` derivation. It is one of two arms. The
prologue branches on the grid before it asks anything:

- **`param_2 == 0x300`** (line 378): no call at all. Two `short`s are read
  straight out of the `WData` array — `world+0x134 + (width × cy + cx) ×
  0x1c + 4`, which `types.txt` names **`WData.region`** — one for the start
  cell and one for the goal cell, and compared. No coastal refinement, no
  `region2`, no tile.
- **otherwise** (lines 391–398): `div_3_table[x >> 6]` on both axes — the
  *tile* — and two **`WorldData::get_tregion`** calls, §15's pair. The tile
  and unit grids take the refinement; the world grid never sees it.

The water test the `same` branch then makes is
**`WorldData::is_ocean@006b4830`** of the start **cell** on `0x300`
(`flags & 0x100` clear and `land` 1 or 2), and the tile-mask surface test
elsewhere — not "the cell's region is a sea region", which is what this
crate had been asking in both `astar_path` and `calc_cost`'s ocean row.
The two answers part on exactly the cells `HALFLAND` marks.

**Why it is load-bearing, and what it moved.** The AI's Fisherman `1/14`
stands on East Indies' cell (57, 55) — `flags 0x104`, `region 11` (land),
`region2 65` (sea) — and is sent to the fish at (49, 39), region 65. §15's
`tregion_alt` answers 65 for the start, so this crate said *same region*,
read the start as water, and set `avoid_land = 1`; the original compares
`region` 11 against 65, says **different**, and leaves both avoids at 0.
The boat's whole sea route is downstream of that one bit: every coastal
`HALFLAND` cell along the channel was costing this crate an extra 200 it
costs the original nothing, and the 3,200-probe budget ran out somewhere
else, with a different node nearest the goal to reconstruct from.

The two pull-back sites and the tile grids are unaffected — §15's fix
stands where §15 made it. The world grid was simply never one of the four.

**What a diff backs.** run58's `PATHDATA` rows: both AI Fishermen's sea
stacks — `1/14` planned on frame 4464 and `1/16` on 4870, sixteen rows
each from (57, 55) to (49, 39), and `1/16`'s two rows that step around the
boat already sitting on `1/14`'s cells — agree with the original **row for
row, point, tolerance and flag**, where before every slot from 1 up was
two cells north. The boats had a pin of their own in the run58 test;
they no longer need one, and no unit leaves the original's point before
the word. East Indies' long word: **4871 → 4945**.

**What this has not established.** Whether the two tests ever disagree on
a cell that is *not* `HALFLAND` — nothing in either capture forces it, and
the reading says they cannot (`is_ocean` refuses `HALFLAND` outright,
`region2` is only consulted for it). And the world grid's raw read is off
the array with no bounds test: a start or goal cell off the map would read
whatever lies there, which no capture has reached and this crate answers
`None` for.

## 17. The middle nodes were one 48-grid step short, and the search was right (2026-09-03)

Two captures, two maps, two units, one signature: a route whose **ends,
length, flags and switch frames are the original's** and whose **middle
waypoints sit one 48-grid step away**.

- **East Indies, run68, `1/13`.** From the `[6660, 6740)` window's block
  6686 the unit's `PATHDATA` stack read `path[2].y` **38712** here against
  38760 and `path[3].x` **40584** against 40536, and its own position went
  with them on 6718.
- **Great Lakes, run69, `1/9`.** The AI's woodcutter walks from its camp
  to the tree it chose on frame 1959, and the dump prints the *current*
  step rather than the stack — `MOVEORDER`'s `dest_x`/`dest_y`, live while
  `dest` is 1 — so the route read off as a sequence whose slots 2 and 3
  were `(40728, 17544)`/`(40824, 17640)` here against `(40776, 17544)`/
  `(40872, 17640)`. Both sides took slot 2 on the **same frame**, 1993, and
  the two routes were the same total length: ours 144 + diagonal + 144, the
  original's 192 + diagonal + 96. It cost a frame of arrival — 2015 against
  **2016** — and that frame was the whole of Great Lakes' word, because the
  woodcutter's 404-frame chop clock starts when it reaches the tile.

**The search was right and the grid was wrong.** run70 is a `callwin` over
`PathFinder::astar_path` and `PathFinder::calc_cost` on Great Lakes frames
1955–1985, run53's game to the frame (`rngcmp.py`: 2,001 frames, zero
differing). `calc_cost` is called **only for a neighbour that passed
`valid_ucoord`**, so the proxy's argument list is the validity filter's own
answer, one row a cell — and the two searches' expansions can be laid side
by side. They agree on **every cell either probed but one**:

    ours valid, theirs invalid : [(849, 366)]
    ours invalid, theirs valid : []

`(849, 366)` is `(40776, 17592)`, the node this crate turned south onto.
The original refuses it from all three of its neighbours that reach it —
`(848, 365)` dir 5, `(849, 365)` dir 6, `(850, 366)` dir 8 — and never
prices it at all; with it gone the wheel and the heuristic put the route
exactly where the original's is. Twenty-one expansions here, twenty-one
there, in the same order.

So nothing in §4 was wrong: the wheel, `first_open_node`'s LIFO on an equal
`value`, the `<=` that lets the first arrival keep a tie, the flat 32/40 of
the unit grid's `calc_cost`, and `get_estimate`'s `vector_dist × 10` were
all confirmed by a run rather than a reading. What was missing was one bit
of the **collision index**: `(848, 367)`, a corner of the stationary
gatherer `1/10`'s block, which `1/9`'s own diagonal step had cleared
eighty-one frames earlier and which the original's sixty-fourth-frame
repaint had put back (`docs/COLLISION.md` §2.2).

**What a diff backs.** All of it, on both maps.
`run69_s_three_thousand_frames_stand_where_the_original_s_do` asserts that
no unit leaves the original's point before the word and no move order's
waypoint parts before it; the word moved **2419 → 2808** with the fix.
`run68_s_window_is_every_unit_s_whole_record_to_the_word` compares `1/13`'s
stack whole — the exception is deleted rather than kept — and its window now
runs to 6730 with no field of any unit parting but `stance`.

## 18. The pre-walk's gate, and the suspend, which is landed unwired (2026-09-17)

Item 301 was booked as "East Indies 7811/7812 is `find_upath`'s suspend".
The suspend is real, is reached, and is now implemented — and it is **not**
what 7811 was. Two separate things came out of the window, and they are
written down separately because only one of them is a defect this crate
could see before the dump was widened.

### 18.1 The pre-walk is gated, and the gate is not `find_wpath`'s

`find_upath@00682f30`'s goal pull-back (§3, "the pre-walk") runs inside

```c
if ((*(int *)(type + 0x218) < 2) && (can_transport(unit) == 0)) { do { … } while (…); }
```

at `00683095` — the type's **domain** `+0x218 < 2` (not air) **and**
`UnitData::can_transport@0046f960` answering 0. It is a **conjunction with
no vfunc `+0x8` disjunct**, where `find_wpath@00688fc0:91` has the
disjunctive form §3 already records: `domain < 2 && (vfunc+8 == 0 ||
!can_transport)`. The two wrappers' gates are different, and this crate had
`find_upath` running the walk unconditionally.

`can_transport` is `(unit_masks & 0x800000 && !(unit_masks2 & 0x2000)) ||
(type unit_flags & 0x10)` — the same predicate §7's tile tolerance reads.
run90's `1/7` prints `unit_masks 8651786` = `0x84040A`, so `0x800000` is
set (its side has a Dock) and `unit_masks2` is 0: **the original never pulls
that citizen's 48-grid goal back at all.**

This crate did, and the pull-back is what block 7811 was. On frame 7810
`resolve_unit_collision` step 6 hands `find_upath` the move order's own goal
`(39624, 38760)`; `valid_ucoord` refuses it, because the other citizen
`1/6`'s 3×3 occupancy block covers that cell, so the walk stepped the goal
`0x18` at a time along the bearing — `(39634, 38739)`, `(39644, 38718)`,
`(39655, 38697)` — until it landed on the unit's **own** 48-cell, and
returned it as a one-entry final leg. `1/7` then walked six units and
arrived on 7812, killing its `EXPLORE_TO` twenty-one frames early. The
original's stack on 7811 is the untouched goal, at `(39624, 38760)`.

### 18.2 The suspend is reached, and `start_dist` is the proof

With the gate applied the search runs for real and **suspends**: `r = −1`,
with the stack left holding exactly one entry, the caller's final goal, tol
0 flags 1 — because `astar_path` pops its own start and goal at entry and
pushes nothing. That is the original's block 7811, field for field.

The corroboration is a field nobody had looked at. **`UnitData::start_dist`
(`+0x130`) has exactly one writer in the whole executable** — the suspend
block at `astar_path@00683770:517` — and it stores the search's
start-to-goal Manhattan. Over run90's 111 blocks and every unit of each it
is zero everywhere except `1/7`, where it reads **144** from block 7811 to
the end of the capture; 144 is `|39672 − 39624| + |38664 − 38760|`, the
Manhattan from `1/7`'s snapped cell centre to its move order's goal. Nothing
clears it, so it is a permanent stamp like `collide_frame` and is asserted
as a **change** (`run90_s_window_is_east_indies_shuffle`).

A `grep` of the disk puts the mechanic's reach beyond this one window:
twenty-three captures carry a non-zero `start_dist`, run16 with 8,752 rows.
The suspend is not a corner of the pathfinder. **§19 is that sweep, done**
— and it amends this paragraph twice: ~~the suspend block is the field's
only writer~~ it is the only writer of a *value*, two initialisers zero it,
and the reach is asserted per capture rather than grepped.

### 18.3 What is implemented, and what is not wired

**Implemented and landed** (`crates/sim/src/path.rs`):

- [`Search`] — the stash, one field per `UnitData +0x104..0x148` name:
  the open list, its metric refs, the closed list and the validity memo
  (`blocklist` has no counterpart, the unit search never fills it), plus
  `tol`, `offset` (which is the direction **preference**, not an offset),
  `start_dist`, `avoid_land`/`avoid_sea`, `endx`/`endy` and `traversed`.
  `valid_hit` is a counter nothing reads and is not kept.
- `astar_path`'s suspend: re-insert the stop node into the open list, move
  the state onto the unit, return −1. The re-insert matters — without it
  the resumed search finds that node again through its neighbours and comes
  out on a different chain.
- `astar_path(…, resume)` — the original's `PathFinder::saving` — which
  skips the whole prologue, **does not pop the stack**, and jumps into the
  loop with the restored state; `arrive` is rebuilt from the restored `tol`.
- `Sim::find_upath_restore` (`00688f40`): `saving = 1`, `anti = 0`, limit
  `300 / max(1, repaths²)`, and `find_upath`'s failure teardown suppressed —
  `00683380`'s pop-and-kill sits inside the same `saving == 0` guard as the
  prologue, so a resumed search that gives up leaves the order alone.
- `Sim::clear_partial_path` stopped being a no-op: it is what frees the
  stash, which is the original's whole body for `005e3920`.

~~**Read, built and not wired**~~ — **wired on 2026-09-17**, all three
together because they are one change (`docs/DECISIONS.md` entry 30), and
§19's comparison runs against the wired tree:

- the §18.1 gate;
- `find_upath`'s limit `500 / max(1, repaths²)`, halved with `anti`
  (`00688eb0`) — a seam since item 80 even though `repaths` has been
  modelled since;
- `do_move`'s suspended-search block, `docs/ORDERS.md` §4.4 step 2: while
  the stash is `Some`, **no step happens**, the "has the blocker gone" probe
  fires on the 5th, 7th, 9th … frame after `collide_frame`, `repaths` ticks
  on every fourth `o + frame`, `collide` counts up **every** frame, and
  `find_upath_restore` resumes. `UnitData +0x48` in that block is
  `collide_frame` by the type record, so its clock is `frame −
  collide_frame`; that is what run90's `1/7` counts 1 → 9 over 7812-7820.

Wired, it is right on the mechanic and wrong on the score, which is entry
30's case exactly. Every row below measured on **one tree**, `1397b05`,
the subsets by taking `4521ccc`'s two files onto the landed tip one at a
time — `crates/sim/src/path.rs` is the gate and the limit, `orders.rs` is
`do_move`'s block:

| | East Indies (run54) | Great Lakes (run53) |
|---|---|---|
| base (`75e13a4`) | 7812 | 7679 |
| landed here (none of the three) | 7812 | 7679 |
| the gate alone | 7812 | 7679 |
| the gate and the limit (`path.rs`) | 7812 | 7679 |
| `do_move`'s block alone (`orders.rs`) | 7812 | **6862** |
| **all three** | **8193** | **6862** |

**Every proper subset is neutral or worse; only the whole gains.** The gate
is a *prerequisite* for reaching the suspend and buys not one frame by
itself — without it `1/7`'s 7810 plan never gets past the pull-back — and
`do_move`'s block without the gate pays the entire Great Lakes cost for
none of the East Indies gain, because `1/28`'s 6860 search suspends whether
or not the gate is there and `1/7`'s does not. So "all three together
because they are one change" is a **measured** causal claim rather than a
convenience of presentation, and the reason `worktree-loop-301-suspend`
cannot be split into a safe half and a risky half is a fact rather than an
assertion: the halves are separately worthless, and one of them is
separately harmful.

**And the unbundled row was taken twice, against two different trees, and
only the second is true.** Measured on the pre-295 tree the gate and the
limit alone read East Indies **8193**, and that figure was reported as
theirs. It is not: 295's census fix changed *which* divergence binds East
Indies' long capture, so a measurement of any change against a pre-295 tree
was measuring against a different binding constraint, and the +381 belongs
entirely to the suspend once 295 is in. The counterfactual is the point.
On the old tree this would have landed `path.rs` **wired**, as a clean +381
with no cost on either map, and the attribution would have been wrong and
would have stayed wrong — a wired change that moves a word is exactly the
thing nobody goes back and re-derives. The rule it leaves is not about how
an item is written: **an attribution measured against a moving base is
unreliable in both directions**, and the only fix is to re-measure every
row on the tip before attributing any of them.

run90's `1/6` and `1/7` stop parting **anywhere** in the capture's 111
blocks with it wired — the whole two-citizen shuffle, positions, collision
fields, order records and path stacks, agrees — which is the measurement
that will justify wiring it. The Great Lakes fall is upstream and named:
run76's `1/28` still holds a `GROUP_ATTACK_TO` at 6860 and the original's
order becomes `ATTACK_TO` during that frame, whose enqueue frees the
suspended search through `clear_partial_path`; ~~this crate's `1/28` is
already on a plain `ATTACK_TO` before 6858, so its formation ended early and
no order change is left to free the stash~~ — **that half was wrong, and
§18.5 has the measurement: the formation ends on the original's own frame,
and what this crate was missing is `kill_current_path`'s own
`clear_partial_path` call.** Both sides are running the *same*
search there — the original's `start_dist` is 7680 against this crate's
7669 start. That row was the queue's **item 304**, closed on 2026-09-17
by §18.5.

The wired change is one commit on the branch **`worktree-loop-301-suspend`**.

### 18.4 What is not established

- **Why `1/7`'s 7810 search costs more than 500 probes** for a goal three
  48-cells away is not explained here. It does — measured, both sides —
  and the nine resumes over 7811-7819 never finish it either; the original
  abandons it on frame 7819 through the blocker-gone arm and walks the
  remaining stack. A cell-by-cell expansion diff (the §17 proxy, pointed at
  the 48 grid) would settle it and no run needs it to.
- **`valid_hit`** is saved and restored by the original and is not kept
  here; nothing reads it in any path either reading has found.
- **The pre-walk's give-up exit targets the wrong label here.** `00683082`
  is push-and-return-length; this crate's `break` falls through to the near
  test and the search. With the gate unwired the arm is only reachable for a
  unit that cannot transport, and no capture on disk reaches it, so it is
  recorded rather than changed.

### 18.5 The Great Lakes cost was `kill_current_path`, not the formation (2026-09-17)

Item 304 was booked as "run76's `1/28` ends its formation early", off §18.3's
reading of the fall. **It does not.** With `4521ccc`'s three lines applied,
`run76_s_window_is_the_ai_squad_s_march` reports **zero** scoring kind or
`GROUPORDER` rows over all 630 marching unit-frames: this crate's squad holds
its `GROUP_ATTACK_TO` to 6860 and degrades on 6860 exactly as the original's
does. The formation's end was never the disagreement — and the record said so
on the first run, which is the widening rule working. What the run does report
is three rows on **6862 and 6863**, `1/28` alone:

| field | ours | theirs |
|---|---|---|
| order flags | 4 | 5 |
| `MOVEORDER::dest` | 0 | 1 |
| path length (6862) | 0 | 10 |
| path length (6863) | 0 | 13 |
| `coll_x`/`coll_y` (6863) | (42770, 24618) | (42754, 24612) |

The original re-plans on 6861 and this crate does not: its `1/28` is still
holding the search that suspended on 6860, and `do_move`'s §4.4 step 2 returns
before every arm while one is pending. It stands there for the rest of the
capture. The frames either side of the ungroup, out of the dump itself:

```
 block 6860  GROUPATTACKTOORDER  collide_frame -1    collide 0  start_dist 0     stack 2
 block 6861  ATTACKTOORDER       collide_frame 6860  collide 1  start_dist 7680  stack 0
 block 6862  ATTACKTOORDER       …                              start_dist 7680  stack 10
```

**`Unit::kill_current_path@005e31d0` is what frees it**, and this crate's had
only half the body:

```c
if (0 < this->path_length) {
  do { … } while (((stack[len].flags & 1) == 0) && (len != 0));
  clear_partial_path(this);          // ← this crate did not do this
}
```

The pop back through the segment's final waypoint was modelled; the
`clear_partial_path` under it was not. `Unit::ungroup_move_order@005fd140`
calls it on every member that is **not** the leader, so the frame a formation
degrades is the frame each follower's stash goes — which is precisely the
"the order change frees the search" §18.3 named without naming the function.
It is not an order-adder's doing at all: the `ATTACK_TO` that replaces the
`GROUP_ATTACK_TO` is built by `MoveOrder::operator=` and spliced into the
list, and no `add_*_order` runs.

Four functions call it in the executable — `kill_current_order`,
`kill_group_move`, `kill_group_order`, `ungroup_move_order` — and this crate
calls it from the same four, so the one line covers all of them. Note the
guard: the clear sits **inside** the `0 < length` test, so a unit with an
empty stack keeps its search. That is the whole difference between this and
`kill_current_order@005e2cb0`, which clears unconditionally at its tail, and
both halves are asserted
(`killing_the_current_path_frees_a_suspended_search`, made to fail both ways).

**The score, measured on one tree.** Base is `5430558` plus this section's
one-line fix; "wired" adds `4521ccc`'s two files on top of that.

| | East Indies (run54) | Great Lakes (run53) |
|---|---|---|
| base, suspend unwired | 7812 | 7679 |
| this fix alone, unwired | 7812 | 7679 |
| `4521ccc` wired, without this fix | 8193 | 6862 |
| **`4521ccc` wired, with it** | **8193** | **7679** |

So the fix is neutral until the suspend is wired — a stash nothing reads is a
stash nothing misses — and with it wired the Great Lakes cost is gone and the
East Indies **+381** stands. The value diff beside the words is the table
above: `1/28`'s own `dest`, path length and `coll` on 6862-6863, which agree
after it; `run76_s_window_is_the_ai_squad_s_march` drops from four units ever
off position to **two** (`1/5` at 6866 and `1/28` at 6862 both stop parting),
and those two are run76's standing `1/24` and `1/25`.

**What this does not establish.** Whether any *other* caller of
`clear_partial_path` is missing here — the executable has 63, and only the
`kill_current_path` one was checked against a capture. The
`Group::action_*` family and the `think_carry*` family are the two blocks
with no counterpart call in this crate at all, and no run on disk reaches
either with a stash pending.

## 19. `start_dist` over every capture on disk, and what it is made of (2026-09-17)

§18.2 established `UnitData::start_dist` from **one** window: run90's `1/7`
reads 144, and 144 is the Manhattan from its snapped cell centre to its move
order's goal. One unit, one value, one capture — and a single value can agree
with a wrong reading for a long time. Item 308 asked the same question of
every capture already on disk. **Twenty-three carry a non-zero value**, over
**1,184,141 dumped rows** and **51 stamped units**; run16 alone holds 8,752
stamped rows. Nothing was captured for this: the rows were already there.

The assertable count is **16,617 stamped rows in 23 captures**, and it is
what the `FRAME` records carry — the only thing a frame diff can ever reach.
A raw `grep` of the same files finds 16,643 and 53 units; the extra 26 rows
and 2 units sit in root-level `FULL DUMP` blocks written *outside* any frame,
which the four islands window captures each end with. That difference is
worth naming rather than rounding away: a sweep that counts the file and a
diff that walks the frames are not measuring the same thing, and the smaller
number is the honest one.

**The writers, all three.** §18.2 said "exactly one writer in the whole
executable", which is right about *values* and incomplete about the field:

| site | what it writes |
|---|---|
| `PathFinder::astar_path@00683770:517` | the value — the suspend block, and the only place a non-zero ever comes from |
| `UnitData::UnitData@00606670:74` | 0, at construction |
| `Unit::init@00612100:567` | 0, beside the five container pointers |

The third matters, and the corpus proves it acts. The value itself is built at
`:453`, `local_6c = |local_50| + |local_4c|` — the sign-mask idiom for
`abs(dx) + abs(dy)` — from the start-to-goal delta of the search about to run,
and it is read back at `:297` on a resume, so a resumed search keeps the
distance the *first* attempt started from and does not recompute it.

**Three invariants, each of which the decompile predicts and none of which a
mis-read field would satisfy.** They are asserted per capture in
`the_suspend_s_stamp_is_a_48_grid_manhattan_on_every_capture`.

- **Every non-zero value is a positive multiple of 48.** Both ends of the
  delta are 48-cell *centres*, so the difference is a whole number of cells
  however the goal was arrived at. 16,617 rows, 29 distinct values, from 144
  (3 cells) to 42,960 (895), and not one off the grid. This is the check that
  would have caught a field read at the wrong offset or the wrong width —
  item 269's third axis — because an `int` misread would land off the grid
  almost surely.
- **Every stamped unit-frame has `collide_frame >= 0`.** The 48-grid search a
  unit can suspend is the one `resolve_unit_collision` step 6 starts, so a
  stamp with no collision behind it would say the suspend is reached some
  other way. There is none in 16,617 rows.
- **Nothing clears it within a unit's life.** Keyed on `uid` there are **zero**
  non-zero → zero transitions in 1,184,141 rows.

**The one apparent clear is the proof, not the exception.** Keyed on the
per-player `o` alone there is exactly one in the whole corpus: run16's `1/9`
reads 768 at block 3016 and 0 at block 3960. The unit is absent from the dump
for the 944 blocks between, and the record either side names two different
objects —

```
 block 3016   who 1  o 9   uid 17   myhits 40   start_dist 768
 block 3960   who 1  o 9   uid 25   myhits 70   start_dist 0
```

— so `o` was handed back out and a new unit was born into it. That is
`Unit::init@00612100:567` zeroing the stamp, observed acting once. It is also
why `uid` is now parsed (`UnitDump::uid`): `o` is a slot and cannot carry a
claim about permanence.

**And the stamp is not a high-water mark.** The frames hold 22 transitions —
17 first stamps and **5 re-stamps**, non-zero → non-zero, in both directions
(run50's `0/21` goes 3120 → 816, run16's `1/12` goes 720 → 1440). A later
suspend simply overwrites it, which is what a single unconditional assignment
at `:517` says and what a "furthest search so far" reading would have got
wrong.

**The transitions are what is asserted, not the values.** A field nothing
clears is checked on the block it *changes*, never on the block it reads —
`collide_frame`'s rule, and the shape run85's first teeth check fell into.
Once run90's `1/7` reads 144 it reads 144 on all 89 of its later blocks, so a
value assertion there passes 89 times for the price of one; the suspend is
dated by 7811, the block the value appears on, and that is the row. The pinned
table is every transition in every capture — `(block, who, o, from, to)` — and
it holds the two the earlier items proved as ordinary rows of a larger set:
run90's `(7811, 1, 7, 0, 144)` and run76's `(6861, 1, 28, 0, 7680)`. Thirteen
of the twenty-three have none to pin, because a window capture that opens
after its unit suspended carries the stamp already standing; for those the row
and unit counts are what says the reader could see anything at all.

**The field is now compared, not only asserted.** `UnitData::log_data` writes
`start_dist` at *every* detail level — it is there in the smallest capture on
disk — and nothing compared it until this item: `FrameResult::search_compared`
/ `search_diverged` check it on every unit-frame whose position the two sides
agree on, in both directions from zero. A stamp this crate does **not** have
is the interesting half, because it says the original gave up on a search
where this crate did not. That required the stamp to move out of `path::Search`
and onto `sim::Unit`: in the original it lives on `UnitData` and outlives the
containers handed over beside it, so it still reads back long after
`clear_partial_path` has freed them, and a copy that dies with the stash could
never match a dump taken after the search ended.

**Where the reach is.** Sixteen of the twenty-three captures are Great Lakes or
East Indies — the two maps the finish line names — and the suspend is dense in
exactly the windows the word sits in: run76 (9 rows, `1/28`, §18.5), run83,
run84, run87, run89, run91 all carry the same two Great Lakes units at 6336 and
7680. The mechanism §18.3 wires is not a corner of one window.

**And on the wired tree it agrees.** The comparison was run on the base's
own tip — `7cfd663`, which carries the suspend wired (§18.3) — against the
two windows the suspend was found in:

| | unit-frames compared | disagreements |
|---|---|---|
| run76 (Great Lakes) | 8,032 | **0** |
| run90 (East Indies) | 2,860 | **0** |

10,892 comparisons, nothing apart. That is a *value* diff, not a draw
stream: it says this crate's `1/7` stamps 144 on 7811 and `1/28` stamps
7680 on 6861, the same blocks and the same numbers the original does, and
keeps them for the rest of both windows. The column would have been all
disagreement a day ago — an unwired suspend stamps nothing — so the new
counter is also a standing check that the wiring stays wired.

**What this does not establish.** The three invariants are properties of the
original's own dump. They say the field is what §18 claims; they do not say
this crate reproduces a row on any capture the comparison has not run, and
the two windows above are the only ones it has. The other twenty-one carry
stamped rows that no test yet replays — that is reach for the taking, and
each one is a capture already on disk.

## 20. The scout's route through a city footprint — what block 8002 is not (2026-09-17)

Great Lakes' word stands at **8031** and its first divergence is one
`find_wpath` call: the AI scout `1/0` takes an `EXPLORETOORDER` to
`(4344, 32760)` on block 8002 — the destination §8.2 of `docs/SCOUT.md`
bought — and plans a **five**-entry stack where the original plans **six**.

| slot | this crate | the original | cell |
|---|---|---|---|
| 0 | `(4320, 32736)` tol 0 flag 1 | same | the order's own goal |
| 1 | `(2808, 31992)` | same | `(3, 41)` |
| 2 | `(2808, 31224)` | `(2040, 31224)` | `(3, 40)` v `(2, 40)` |
| 3 | `(2808, 30456)` | `(2040, 30456)` | `(3, 39)` v `(2, 39)` |
| 4 | `(3576, 29688)` | `(2808, 29688)` | `(4, 38)` v `(3, 38)` |
| 5 | — | `(3576, 29688)` | `(4, 38)` |

Read top-down, the original walks `(5,38) → (4,38) → (3,38) → (2,39) →
(2,40) → (3,41)` and this crate `(5,38) → (4,38) → (3,39) → (3,40) →
(3,41)`. Cells `(3,39)` and `(3,40)` are two of the four the **human
capital** sits on — a Small City, 7 × 7 at corner tile `(13, 157)`, whose
blocked template paints 6 × 6 at tiles 13–18 × 157–162, nine blocked tiles
a cell (§20.2); the original goes round them to the west and this crate
through them. Its **position** then parts on block 8014, the frame the
routes' headings separate — `(3912, 29680)` against `(3912, 29677)`.

**Two forcings reproduce the original's stack entry for entry**, and no
third was found:

- half-cells `(7, 79)` and `(7, 81)` read as *seen* by player 1 — those are
  `div_3_table[to >> 7]` of the two nodes, the exact points `calc_cost`'s
  fog test reads — which moves both cells out of the fog branch and prices
  them at `0x400 / 8 + 20 × 9 + …`;
- cells `(3, 39)` and `(3, 40)` refused by `valid_wcoord`.

Either alone is not enough: with `(7, 79)` only, slot 2 still parts; with
`(7, 81)` only, three rows do.

### 20.1 What block 8002 is **not**, and the diff that says so

Everything the search reads was eliminated against the original's own
`WORLD` scan rather than by reading, and that was possible because
`docs/VISION.md` §7's "no dump on disk carries a *second* fog plane to
diff against" is **wrong**: a `DUMP_ALL` *window* prints the whole scan on
every block it covers, and three Great Lakes archives carry one mid-game —
run13 at 95–104, run73 at 5564–5580 and **run93 at 7929–7936**, seventy
frames under the word. `run93_s_block_7932_is_this_crate_s_world_cell_for_cell`
is the diff:

| plane | compared | parting |
|---|---|---|
| fog (`seen2`) | 14,400 half-cells | **0** |
| cells (`flags`, `who`, `blocked`, `solid`, `bad`) | 3,600 | **0** |
| tile masks | 57,600 | 171, one cluster |

So at block 8002 the search reads the original's own world. Ruled out with
it:

- **the danger grid** — diff-backed against run64, and unread in the fog
  branch anyway;
- **`think_scout`** — the destination and the frame's 38 draws are the
  original's (`docs/SCOUT.md` §8.2);
- **the cost arithmetic** — every priced step of this crate's search
  reproduces its exact number by hand, seen and unseen: `(4,38)` costs
  `128 + 8 + 4 = 140` (base `0x400` scouting, `danger 65 >> 3`, the enemy
  `+4`), `(4,39)` `328` (the same plus `20 × 9`), `(3,38)` and `(3,39)`
  `1` and `9` (base `8`, no extra, the diagonal's `8`);
- **the search's mode flags** — `scouting`, `army`, `worker`, `no_danger`
  and `iroquois` were each forced on and off; none produces the original's
  stack, and three of them move the *destination*, which is already pinned.

### 20.2 It is the fog, and the writer is the building itself (run95)

~~Why the fog is the weaker of the two, and what is still open.~~ **The
capture overturned this section's conclusion the same day it was written,
and the elimination below was right in its premise and wrong in its
verdict.** run95's `callwin` proxy over `calc_cost` on sim-frame 8001 —
**83** records, the whole of `1/0`'s one search and the only search the
window holds — prices the step from cell
`(4, 38)` into `(3, 39)` at **328**, and the neighbouring step into
`(3, 38)` at **1**:

| step | the original | this crate | what 328 is made of |
|---|---|---|---|
| `(4,38) → (3,38)` W | **1** | 1 | base `8`, unseen, scouting |
| `(4,38) → (3,39)` SW | **328** | 9 | `128` seen base + `8` danger + `4` enemy + `20 × 9` terrain + `8` diagonal |
| `(3,38) → (3,39)` S | **320** | — | the same without the diagonal |
| `(3,38) → (2,39)` SW | **9** | 9 | unseen |

The step is **priced, not refused** — and a `valid_wcoord` refusal happens
upstream of `calc_cost`, so the record's presence is itself the verdict
that the predicate passed. So `invalid_loc` is not the answer (§20.3's two
gaps stand on their own), and the fog forcing was the right one all along.

run95's own `DUMP_ALL` window then says why, at the block the search runs
on. **Fourteen half-cells of 14,400 part at block 8002**, in two patches,
and every one is player 1's bit over ground player 0 holds:

```
 y=78  (6,78) (7,78)        the 3 x 5 block x 6-8, y 78-82, which is
 y=79  (6,79) (7,79)        two radius-1 discs — circle_radius[1], the
 y=80  (6,80) (7,80)        3 x 3 — centred on (7,79) and (7,81): the
 y=81  (6,81) (7,81) (8,81) 2c+1 half-cells of cells (3,39) and (3,40)
 y=82  (6,82) (7,82) (8,82)
 y=73  (10,73) (11,73)      two single points, beside the human's 0/1 and 0/2
```

`(7, 79)` and `(7, 81)` are exactly the two the route reads, which is why
forcing them seen reproduced the original's stack entry for entry.

**And §20.2's premise survives: no `update_seen` disc of the scout can make
this.** `1/0` is the only owner-1 unit ever west of cell 37, its `mylos`
is 6 on every compared frame so its radius is 3, and its projected sweep
centre runs `(15,81) → (14,81) → (13,81) → (12,81) → (12,80) → (12,79) →
(11,78) → (11,77) → (10,76) → (10,77) → (9,77)` — never within five
half-cells of `(7, 81)`. Nor is the shape a disc of any radius: a
radius-1 pair is, and a radius-1 disc centred on a **cell's own centre
half-cell** is not something `Object::update_seen` produces for a unit at
all.

**So it is a second reveal this simulation does not make**, and item 322
found the writer — not by a name grep but by the offset search item 320
booked, and then not by reading at all. ~~`Unit::update_local_seen`, with
`ObjectData::visible` as the mask.~~ **`Wall::update_local_seen@0063ed50`,
driven by `Wall::check_ever_seen@0063ce70`**, whose mask is the building's
own `ever_seen` byte and whose rectangle is its footprint grown by one
tile each way. `docs/VISION.md` §6.1 is the whole mechanic.

**The dump had the answer printed on every record.** `BUILDDATA` writes
`ever_seen` and `ever_seen_completed` from `BUILDS=1` and nothing here had
ever parsed them. At block 8002 exactly **two** of player 0's seven
buildings read `3` — the Small City `0/2000` at half-cell `(8, 80)` and
`0/2001` at `(11, 74)` — where the other five read `1`, and the two are
the centres of the two patches. One `grep` of a block already on disk,
against the session item 320 spent eliminating the writers a name grep can
find.

**And the geometry is the type's size, not the blocked mask.** A Small
City is **7 × 7** — `X_SIZE 7`, and the dump's own `mylos 15` is
`LOS 12 + x_size / 2` (`docs/VISION.md` §2.1) — while the *blocked*
region `BuildType::mask_me`'s template paints inside it is **6 × 6** at
tiles 13–18 × 157–162, which is the `blocked 9` run95's `WORLD` scan
carries on each of the four cells `(3, 39)`, `(3, 40)`, `(4, 39)`,
`(4, 40)`. `update_local_seen` walks the *type's* rectangle: corner tile
`(13, 157)`, `i` and `j` from `−1` to 7, tiles 12–20 × 156–164, half-cells
**6–10 × 78–82**. The columns at `x` 9 and 10 and the three cells
`(8, 78..80)` were already lit by the scout's own disc; the twelve that
were not are twelve of the fourteen, and `0/2001`'s own rectangle is the
other two.

The reveal lands between blocks **7937 and 7998**: run93's 7936 still has
all fourteen dark and run95's 7999 already has them lit, and the planes are
identical at 7932 (§20.1). `check_ever_seen` runs on `frame & 7 == who`,
so player 0's buildings check on 7944, 7952, … 7992 — every one of them
inside that window.

~~And inside that window nothing happened that could have called it.~~
**That elimination was sound and its premise was too narrow.** It
enumerated the callers of `update_seen(0)`; `update_local_seen` is vtable
slot `+0x164` and has **ten** callers, of which `Wall::check_ever_seen`,
`Build::process`, `Build::do_attack`, `Build::check_capture`,
`Build::do_missile_launch` and `Build::start` are none of them. A grep of
the *slot* rather than the name is what finds them, which is the rule
`CLAUDE.md` already carried and item 320 had already been bitten by once
the same day. (The roster counts it leaned on were also wrong in the other
direction: the block carries **28** buildings, 7 of player 0's and 21 of
player 1's, and 56 was a count of `BEGIN` lines over a nested record.)

The assertion is
`run95_s_block_8002_is_where_the_fog_parts_and_the_price_with_it`, which
now pins the plane **exact** — 14,400 of 14,400 — and `ever_seen` record
for record on all 28 buildings. With it Great Lakes' word went
**8031 → 8186**.

**And the word at 8031 is this route's own**, which was worth measuring
rather than assuming. `RON_DEBUG_SITES=8025-8035` with
`RON_DEBUG_UNIT=1/0@8025-8035` on run53's 24,000-frame trace attributes the
parting draw: `f8031 unit 1/0: Unit::do_move+0xe84` — the same unit and the
same call site as 8030, which item 319 moved. What it is doing there is the
route: by 8031 this crate's scout has consumed its column-3 waypoints, sits
at `(3427, 29988)` with `[goal, (2808, 31992)]` left on the stack, and
spends a **tile-grid** repath — thirteen entries at tolerance `0x60` —
going round the capital's footprint it should never have been beside. The
original spends no `Unit::do_move` draw between 7676 and 9443. So the fog
fix and the word are the same item rather than two: with `(7, 79)` and
`(7, 81)` lit the scout takes the original's western route, never marches
into those tiles, and never spends the draw. **By how much the word then
moves is still unmeasured** — the next parting could be anywhere.

### 20.3 Two gaps `invalid_loc` has here either way

Neither moves this map's word; both are real and cited so a successor does
not re-derive them.

- **The land arm's cell test.** `invalid_loc@00607c30`'s `param_4 == 0`
  branch opens with `if ((cellflags & 0x70) && ((cellflags & 0x20) == 0 ||
  !(unit_masks2 & 0x4000)) && param_3 && param_6) return 2` — mountain,
  forest or the unnamed `0x40` refuses a **land** unit at the *cell*, and
  it is the twin of the `0x70` test this crate already makes on the sea
  arm (`SEA_REFUSES`). `p3` and `p6` are both 1 only from `valid_wcoord`,
  so it is a **world-grid-only** refusal: `valid_tcoord` and
  `valid_ucoord` pass `p3 = 0`. Unimplemented.
- **The fog shortcut.** `param_4` (`1 < timeout`) plus `leaders.list[who]
  & 4` returns **valid** early when all four of the tile's cell's fog
  half-cells are unseen, skipping every terrain test. `_fog_relax` is an
  unused parameter here. It is a relaxation, so it can only ever make this
  crate refuse where the original allows — and the two are a pair in the
  same sense §5's fog branch and terrain row are: landing the first
  without the second would refuse unseen rough ground the original walks.

## 21. The retry a failed unit-grid search buys, and the kill it spares (2026-09-17)

The 48-grid search does not only answer "no path". When it fails it
**buys the mover a delay** and, on the strength of that delay,
`find_upath` leaves the order alone instead of killing it. Item 329 found
this by widening Great Lakes 8187, where run19's `1/28` stands still for
eight frames with a full path stack and `crates/sim` threw its chase away
and went back to `Unit::fight`.

### 21.1 The two fields

`MoveOrder` (the type record, `rise.pdb`) carries four counters in a row
and three of them had been read as one another at some point:

| offset | name | what |
|---|---|---|
| `+0x18` | `pause` | the head-on stagger, `% 9 + 1` (`docs/COLLISION.md` §6) |
| `+0x1c` | `retry` | **this section** — frames the move sits out after a failed unit-grid search |
| `+0x20` | `attempts` | the ceiling counter that decides whether another `retry` may be bought |
| `+0x24` | `timer` | the move's self-destruct |

The dump prints all four on every `MOVEORDER` block, so each is checkable
rather than argued.

### 21.2 The two failure tails, and the gate that is not shared

`PathFinder::astar_path@00683770` leaves a unit-grid search (`param_2 ==
0x30`) by two doors, and **they do not have the same gate**:

- **Work cap** — `traversed + probes >= work_cap && anti == 0`,
  `00683770:552`-`564`. Gate: the current order is a ~~**transit**~~
  **move**, vslot `+0x14` (§21.6). Roll:
  `Random::get(game_random, 0, 0xffff) % 3 + 6` into that order's
  `retry`. The draw's return address is `006848c9`
  (`astar_path+0x1159`).
- **Open list exhausted** — `00683770:919`-`971`. Gate: the current order
  is a ~~transit~~ move (§21.6) **and** its `attempts` is under `0xd`. Same roll, same
  field. The draw's return address is `00684e07`
  (`astar_path+0x1697`), and that is the one run53's trace records on
  Great Lakes 8187.

Both tails then add **30** to `UnitData::safe` (`+0xb2`), outside the
move test, which is the cooldown `detect_unit_collision` reads
(`docs/COLLISION.md` §4.1).

The `attempts` ceiling on one door and not the other is the kind of
difference a single reading folds away; it is here because the listing
was read, and because the two draw sites are separate entries in
`rondata::trace`'s table for the same reason.

### 21.3 What spares the order

`PathFinder::find_upath@00682f30:174`-`196`, on `astar_path < 1` and no
suspend: pop the top entry unless it carries `FINAL`, then

```
order = update_order(unit)
if is_move(order) && order->move_data->retry != 0:     leave it alone   # §21.6
else:                                                  kill_current_order(unit)
```

So the kill is the **default** and the roll is the reprieve. A crate that
implements the kill without the roll — which this one did until item 329
— destroys every chase that meets a blocked neighbour, and the divergence
shows up not as a missing draw but as a unit re-entering `fight` a frame
later with an empty stack.

### 21.4 What consumes it

`Unit::do_move@005f7b30:348`-`369`, between the action tests and the
planner:

```
if retry != 0:  retry -= 1; if retry == 0: attempts += 3; return
<the modern-infantry unpack arm>
if attempts != 0: attempts -= 1
<push the goal, find_wpath, …>
```

A standing `retry` skips the whole planner: no plan, no waypoint, no
step, no draw. `attempts` decays by one on every frame that reaches the
planner and gains three each time a delay runs out, so a unit wedged
against a neighbour climbs to the `0xd` ceiling and stops being able to
buy quiet.

~~**SEAM**: the unpack arm between the two — `is_modern_infantry &&
!has_general(0x8000)` on the `(o * 0x11 + frame) & 0x7f == 0` phase,
which sets `retry` from the type's `+0x78` and `attempts` to `−3` — is
not modelled. No capture on disk has a packable type in it.~~ Built by
item 1113 (`docs/GROUPS.md` §34.3): the arm is a **pack**, `CHAR_PACK`,
and `+0x78` is guy 0's `end_time` read after it; run404 packs twenty
times.

### 21.5 The value diff

run19's block window, `1/28` of Great Lakes, sim-frames 8187 onward — the
dump's own numbers against the crate's, after item 329:

| field | 8187 | 8188 | 8189 |
|---|---|---|---|
| `retry` | 8 | 7 | 6 |
| `safe` | 30 | 29 | 28 |
| position | `(36456, 23592)` | `(36456, 23592)` | `(36456, 23592)` |
| `collide` / `collide_o` | 1 / 36 | 1 / 36 | 1 / 36 |

Every row is exact on both sides, the rolled 8 included — which is what
tells a right roll from a draw spent in the right place. The three
chasers beside it (`1/27`, `1/29`) walk to the dump's own coordinates on
each of the three frames.

**Not established**: whether any capture reaches the *work-cap* tail —
`006848c9` appears in no trace on disk, so its missing `attempts` gate
rests on the listing alone. The residue beside it: `1/28`'s path stack is
**41** entries where the dump says 42, an off-by-one already present at
8186 and belonging to the world-grid plan (`find_wpath` drops one
waypoint near the goal), not to this mechanic.

### 21.6 The gate is `is_move`, and the action bit is not read (2026-09-23, item 673)

§21.2 and §21.3 wrote the gate as "a transit", and this crate built it
as `Order::is_transit`: a move **without** the action bit. The listing
has one virtual call in each of the three places, and it is the same
slot:

- the work-cap tail: `call *0x14` on the current order before the roll;
- the open-list tail: `call *0x14` at `00684d5b`, then `+0x40` and
  `cmpl $0xc, 0x20(%eax)` for the ceiling;
- `find_upath`'s reprieve: `call *0x14` at `006833c9`, then `+0x40` and
  `cmpl %edi, 0x1c(%eax)` for `retry`.

`+0x14` is `is_move` (`docs/ORDERS.md` §1.1). On `MoveOrder`,
`AttackToOrder`, `GroupMoveOrder` and `GroupAttackToOrder` the slot is
`StrafeOrder::is_air` in `vtables.txt`, the COMDAT fold of `return 1`.
No flag is read. So an attack-move with the action bit set buys the
retry and keeps its order exactly as a transit leg does.

**Where it showed.** Great Lakes 12536: group 66's thirteen take a
`GROUP_ATTACK_TO` (flags 5) on both sides. The captain `1/27` steps
into the standing `1/64`, and its 48-grid search exhausts. The
original rolls `retry` 6 (the `astar_path+0x1697` draw), spares the
order, and `do_group_move`'s step-10 tail ungroups the squad: `1/27` to
a plain `ATTACK_TO` with `dest` 0, and `1/28` and `1/29` to plain
orders that re-plan ten world entries at tolerance 384. This crate
killed `1/27`'s order, so no ungroup came and `1/28`–`1/29` marched on
in formation. The value diff and what moved are in `docs/AI.md` §65.

**Pinned**: `a_walled_in_attack_move_buys_the_retry_and_keeps_its_order`
(`sim`), made to fail against `is_transit`; the dump's own `1/27` on
block 12537, `retry` 6 and the rest, in
`run174_s_word_frame_is_widened_whole`.

**Not established**: whether any other `is_transit` reader in this crate
stands for a `+0x14` call. `get_action`'s walk (`docs/ORDERS.md` §1.2)
is `is_move && !action` in the listing, and was not re-read here.

## 22. `is_attacking` is a `return 0`, and the `army` mode it was switching off (2026-09-18)

> **Amended by item 899 (§29).** The census below read the seventeen
> *primary* order vftables and is right about each of them: every move,
> `ATTACK_TO` among them, answers 0. It missed the three classes whose
> `UnitOrder` vftable is a secondary one — `AttackOrder`,
> `GroupAttackOrder`, `StrafeOrder` — and those answer **1**. So the
> clause is live, and what was wrong in the first reading was its
> meaning ("has a combat target"), not its existence.

`find_wpath`'s mode block (§3) sets `army = 1` for an AI unit that is
military, is not a worker, is not `is_attacking`, and does not stand on a
cell flagged `0x100`. This crate read the third clause as "the unit has a
combat target". **It is not that, and in the shipped executable it is not
anything**: the clause never fires, and reading it as a live test turned
`army` off for exactly the units the mode exists for — an AI army walking
to an `ATTACK_TO`.

### 22.1 The chain, and where it ends

`UnitData::is_attacking@0060a5b0` refreshes `orderlist.field_0x4` from
`field_0x14` and calls the **current order's** virtual `+0x18`:

```
0060a5f0:  call   *0x18(%eax)     # the order's vtable, not the unit's
```

Slot `+0x18` is `is_attack`: the only class in the export that declares an
override there is `AttackGroundOrder`, and `rise.pdb` names its thunk
`AttackGroundOrder::is_attack'vtordisp{-4,0}'@0048384b` — which is how the
slot is identified at all, because every other order class folds onto a
stub.

Read straight out of `riseofnations.exe`, slot `+0x18` of each of the
seventeen order vtables:

| vtable | `+0x18` |
|---|---|
| `UnitOrder` `00b474f0`, `MoveOrder` `00b4a12c`, `AttackToOrder` `00b48850`, `GroupAttackToOrder` `00b47e34`, `GroupMoveOrder` `00b494b4`, `GroupOrder` `00b49330`, `ExploreToOrder` `00b48714`, `FleeToOrder` `00b485d8`, `PatrolOrder` `00b48aec`, `GroupPatrolOrder` `00b48498`, `AirOrder` `00b4788c`, `AirPatrolOrder` `00b48c50`, `FormOrder` `00b49608`, `ThinkOrder` `00b48d88`, `SpecialAnimOrder` `00b49078` | `0041bff0` |
| `AttackGroundOrder` `00b49f1c`, `AirAttackGroundOrder` `00b49d90` | `0048384b` |

`0041bff0` is the COMDAT-folded `{ return 0; }` the PDB happens to name
`Window::get_button`. `0048384b` is a `vtordisp` thunk that adjusts `this`
and **tail-calls `0041bff0`**. So `is_attacking` returns 0 for every order
the game can hold, and `find_wpath` is its only caller in the whole
executable.

That is the correction: `army = military && !is_worker && !river`, with no
attacking term at all. The bytes are the evidence a decompile listing
cannot give on its own — a folded body reads as a foreign function name,
and here the foreign name was the whole of the mistake.

### 22.2 What the mode was worth — Great Lakes 8186

The mode is not a detail. `army` is what makes §5's two army terms live:
`base <<= 5` on a cell flagged `NEARBLOCK` (`0x200`) — 32 against 1024 for
the step — and `+10000` on `tcost >= 5`. With it off, an army prices
rough and built-up ground exactly like a citizen.

run97 block 8187 is the frame Great Lakes' AI sends six units from its
base to the far south-west under an `ATTACK_TO`. `Group::action_move_near`
plans **one** world path on the leader and hands each member the same
chain offset by its formation slot (`docs/GROUPS.md` §6.7), so one wrong
mode costs six units their whole route. The original's chain and this
crate's agree entry for entry from the start until the fourteenth
waypoint, `(26712, 26808)`, and then part for five steps before rejoining
at `(22872, 27576)`:

| step | the original | this crate, before |
|---|---|---|
| 1 | `(25944, 26040)` — cell (33, 33) | `(25944, 27576)` — cell (33, 35) |
| 2 | `(25176, 26040)` — (32, 33) | `(25176, 26808)` — (32, 34) |
| 3 | `(24408, 26040)` — (31, 33) | `(24408, 27576)` — (31, 35) |
| 4 | `(23640, 26808)` — (30, 34) | `(23640, 27576)` — (30, 35) |
| 5 | `(22872, 27576)` — (29, 35) | `(22872, 27576)` — (29, 35) |

Both routes are five steps with three diagonals, so on **geometry alone
they tie at 184** — which is why the wrong one was reachable at all. The
cells are what separates them. This crate's own world at 8186, on the
cells the two routes differ over:

| cell | flags | blocked | what it costs |
|---|---|---|---|
| (33, 33), (32, 33), (31, 33), (30, 34), (29, 35) | `0x0` | 0 | 32 either way |
| (33, 35), (32, 34), (31, 35) | `0x200` | 0 | 32 without the mode, **1024 with it** |
| (32, 35) | `0x220` | 12 | `+240`; what keeps *both* routes off row 35 |

So the original's route costs 184 and this crate's 3160 once the mode is
right, and the tie is broken the original's way. The danger map is zero on
every one of these cells for both players, and run93's block 7932 puts the
whole western half of the map cell for cell on the original's own scan
(`crates/rondata/src/diff/world.rs`) — so neither danger nor terrain was
ever a candidate, and the mode was the only term left.

### 22.3 The value diff

`crates/sim/src/path.rs`'s `Sim::army_mode`, with `!attacking` removed:

| | before | after |
|---|---|---|
| Great Lakes word (run53) | 8663 | **8985** |
| East Indies word (run54) | 9711 | 9711 |
| run97 point-and-goal fields wrong below the word | 601 | **6** |
| — per frame | 2.8 over `[8029, 8663)` | **0.006** over `[8029, 8985)` |
| — units ever off position | 8 | **4** (three of them the standing trio) |

The residue's whole live half goes: `1/27`, `1/28`, `1/29` and `1/41`, the
four that first parted at 8579–8602, are exact for the length of the
window. What is left is `1/33`'s six fields from 8584 and the standing
trio's own `(24, 24)`, which is older than every capture on this map.

The make-it-fail is the same edit reverted: with `attacking =
combat.target.is_some()` back in the conjunction the word returns to 8663
and the residue to 601, both measured.

### 22.4 What this does not establish

- **The river clause has no capture.** `armed && !worker && !river` is
  implemented whole now — the flag is `cell::HALFLAND` on the unit's
  **own** cell, read off the obfuscated position fields at `006896ae`,
  not on the search's `here` — but adding it moved neither word and
  neither residue by a single field: no unit in the corpus plans a
  `find_wpath` while standing on a `0x100` cell.
  `a_unit_on_a_river_cell_is_not_an_army` carries it from the listing
  alone. **The capture that would refuse it**: a `WORLD`-plus-`UNITDATA`
  window on a map where the AI's route starts on a shoreline cell, with
  the path stack either side of the plan; East Indies is the candidate and
  none of its blocks on disk sits on such a frame.
- ~~**The `is_supply` arm of the first term.** `type.attack == 0` still
  takes the mode when the unit's `is_supply` virtual answers yes
  (`unit_flags2 & 0x40` for the base class). Unimplemented, and named in
  `army_mode`'s own comment; no supply unit in the corpus plans a
  `find_wpath`.~~ Built and diff-backed by item 569: golden chapter
  four's Supply Wagon plans one on 1101 (§25).
- **Whether `is_attacking` was ever live.** The claim here is about the
  shipped binary's bytes and nothing else. An earlier build, or an order
  class cut before release, may well have returned 1 from `+0x18`; the
  vtables that ship do not.

**Three residues the widening left on the same record**, each printed by
`great_lakes_8186_plans_the_probe_s_route_the_original_s_way` and none of
them the route:

- **The bottom entry, the formation slot.** Under the destination sits the
  raw slot `Group::action_move_near` pushes (§12's second item). At block
  8187 this crate gives `1/27`, `1/28` and `1/29` the same `y` — 21275,
  x 144 apart — while the original gives `(38993, 21170)`, `(39031,
  21309)` and `(38954, 21030)`: the slot table **rotated** by the unit's
  angle, which is `Group::update_positions@00713810` (`docs/GROUPS.md`
  §6.6). `1/40` agrees on it and `1/41`/`1/42` do not, which is what a
  rotation about the leader looks like. It costs no draw in this window.
- **A whole second leg under `1/40`'s.** The original's stack is 94
  entries where this crate's is 48, and the 46 extra sit *beneath* the
  current leg — a queued move this crate does not hold. The 46 that
  overlap agree entry for entry, so the plan is right and the queue is
  short.
- **The top entry's `tolerance`/`flags`.** The dump prints the current
  waypoint as `t0 f1` on every frame of the march; this crate keeps the
  planner's `t384 f0` on it. Unmoved by this item, before and after.

**The successor the word names** is neither: Great Lakes 8985 is a
`Leader::use_market+0x1ed` this crate spends where the original spends a
third `Leader::make_stuff+0x221`, with the frame's count equal at eight
either side — and at 9182 the exchange runs the other way, the original
taking the market draw and this crate a `make_stuff`. The market schedule
below the word is the original's own on all four frames it holds
(`8582, 8585, 8782, 8982`), so it is the *order within the leaders' tick*
that has moved, not the gate.

## 23. `1/62`'s detour round `1/27`, one cell's verdict — Great Lakes 11903, widened (item 563, 2026-09-23)

Item 560 moved Great Lakes' long word to 11903: ours spends 5 draws
against the original's 4, parting at index 1, with `1/64`'s
`Guy::set_anim+0x97a < Unit::move_step+0x823` against the original's
`Guy::set_anim+0x97a < Guy::inc_time+0x271`. No dump reached it (run135
ends on block 11859), and no mechanism was named.

### 23.1 run136, and the readings

`docs/RUNS.md`, run136: run135's line over `[11840, 11959]`, 264,051,273
bytes, all six checks green. The stanza (`tools/gamelog/captures.txt`)
wrote four readings first. R1: the original's `1/64` does not stop on
11903. R2: a lag. R3: another collider. R4: another clock.

`run136_s_word_frame_is_widened_whole` walks run123 → run125 → run130 →
run135 → run136 from 11400: 560 blocks, 1,905,261 record rows, 1,179,360
leader rows and 645 pool lists.

- **R2 is dead.** The 284 keys first parting under the word are run135's
  standing floor exactly. Nothing new parts on 11860..11901, and on block
  11902 no standing position lies within a thousand units of `1/62`.
- **R4 is dead.** `1/64`'s wait on 11898..11902 agrees block for block.
- **R1 holds, and R3 names why.** The word's first row is block 11902's
  `1/62` `path:length`, ours 6 against the original's 8.

### 23.2 What the dump says

Sim-frame 11901 writes block 11902. Both sides' `1/62` stand at (38904,
21144), unit cell (810, 440), stopped on `1/27` (`collide_o 27` on both,
`coll` (38862, 21181) on both). Both re-plan the flag-2 detour to their
waypoint (38088, 21912), and the two plans go round `1/27` on opposite
sides:

| | top entries of the stack, next first |
| --- | --- |
| original | (38904, 21096) → (38856, 21048) → (38664, 21048) → (38088, 21624) → (38088, 21672) |
| this crate | (38904, 21192) → (38904, 21288) → (38280, 21912) |

On block 11903 the original's `1/62` has walked north to (38904, 21119),
into `1/64`'s way, and `1/64` waits on it hard (`collide 5`, `collide_o
62`). This crate's has walked south to (38904, 21169), so `1/64` steps to
(39004, 21112). On frame 11903 it plans its own detour and stops, which is
the extra draw. `1/27` and `1/28` agree on both sides throughout, in
position and in cell.

### 23.3 This crate's search, measured

A scratch print in `valid_ucoord` (§6), not landed, records every probe
of `1/62`'s search on sim-frame 11901. From (810, 440) toward (793, 456),
§4.1's preference is `pref = 7`, so the wheel starts at W:

| dir | cell | verdict | hit cell |
| --- | --- | --- | --- |
| W | (809, 440) | refused | (808, 441), `1/27` |
| NW | (809, 439) | refused | (808, 440), `1/27` |
| N | (810, 439) | valid | — |
| NE | (811, 439) | refused | (812, 438), `1/64` |
| E | (811, 440) | valid | — |
| SE | (811, 441) | refused | (812, 440), `1/64` |
| **S** | **(810, 441)** | **valid** | — |
| SW | (809, 441) | refused | (808, 440), `1/27` |

S is nearer the goal than N, and the unit grid's heuristic is ten times
`vector_dist` (§6), so the search takes S. For the original to go north,
its `valid_ucoord` must refuse S. The disc's two cells outside `1/62`'s
own block, for S, are (809, 442) and (811, 442). The collision index here
holds a bit on neither. (Beside them it has two holes that §2.2's repaint
has not yet filled: (812, 439) and (814, 442), inside `1/64`'s and
`1/28`'s blocks.)

**The probe's form is settled from the listing.** `valid_ucoord@00687c80`
pushes `detect_unit_collision(x, y, 1, 1, ecx, 1, 0)` (`00687d22`–
`00687d34`), and `detect_unit_collision@00617060` hands its sixth argument
to `collide_here@00682540` as `nocoll` (`00617186 pushl 0x1c(%ebp)`). So
the search's probe is the disc, as COLLISION §4.2 says, and not the
leading edge. **The instrument disagrees:** run116's probe log shows
every `collide_here` inside a `nocoll=1` search probe with `nocoll=0`.
Either the proxy logs that argument wrong or it passes it wrong, and the
second would perturb the searches it records. That is the probe build's
question, not this section's. For S from (810, 440) the two forms test
the same two cells anyway.

### 23.4 The payoff probe

`Sim::probe_refuse` (`path::RefuseProbe`, test-only) refuses one unit
cell to one unit's searches on one sim-frame, and changes nothing else.
Refuse (810, 441) to `1/62` on 11901, and its plan becomes the original's
entry for entry, eight entries north by (38904, 21096). `1/64` then waits
as the original's does, and **no record of any unit first parts on blocks
11897..11905**. Landed as `run136_s_word_is_one_cell_of_1_62_s_search`,
made to fail on purpose by moving the refused cell off the search's
ground; it then fails on this crate's own southern plan. With the probe
in, run53's word moves **11903 → 12038**. That was measured once on the
run53 test and is not landed, because the probe is a hand-placed verdict
and not a mechanism.

So the word is **one cell's verdict in `valid_ucoord`**, and the search
around it (§4–§7) is right on this frame.

### 23.5 What this has *not* established

- ~~**Why the original refuses (810, 441).**~~ It never probes it: the
  verdict is `1/27`'s, left in the pathfinder's memo by a `find_upath`
  that returned before `kill_lists` (§24.4, item 566). Three candidates
  were named, and none was the answer: a bit at (809, 442) or (811, 442) that the original's index
  holds and this crate's does not (a mark §2.2's repaint or §2.3's
  crossing puts there, or a clear this crate makes and the original does
  not); `1/62`'s own block computed on a point other than its unit cell
  (dead for this frame: its figure stands on its unit point on every
  block 11898..11903); or a refusal outside the index, from
  `invalid_loc`, which this crate's answers 0 (valid) for the cell. `1/62`'s `safe` and its path top's
  `flags & 8` gate the probe off entirely (COLLISION §4.1), so they cannot
  refuse.
- **The instrument that would settle it.** run137 was booked for it
  (`tools/gamelog/captures.txt`): a `RON_COLLIDE_PROBE` build with
  `callwin` over 11900..11902, which would record each of the original's
  search probes with its hit cell. The build faulted at load under free
  Wine: `7BF21139`, read of `0x00004ECD`, `docs/ORACLE.md`'s wow64 mode-
  switch fault, which is layout-sensitive. run116's probe build did not
  hit it on 2026-09-21. The stanza is retired, not taken.

### 23.6 Coverage

**Diff-backed**: §23.1 and §23.2, from `run136_s_word_frame_is_widened_whole`,
which pins the word's 36 rows on blocks 11902..11904, the 284-key floor
under it, `1/62`'s and `1/64`'s rows under the word, and `1/64`'s stop as
the one animation change on either side alone. The word itself is
`run53_s_24000_frames_put_the_ceiling_where_run33_did`. **Counterfactual,
landed**: §23.4, `run136_s_word_is_one_cell_of_1_62_s_search`.
**Listing-backed**: §23.3's argument chain, `llvm-objdump` of
`0x687cec..0x687d40` and `0x617141..0x6171b0`. **Measured once, not
landed**: §23.3's table (a scratch print) and §23.4's 12038.

## 24. Why S is refused: the pathfinder's memo outlives the search that filled it (item 566, 2026-09-23)

§23.5 left one question: why the original's `valid_ucoord` refuses unit
cell (810, 441), S, to `1/62` on sim-frame 11901. **It does not refuse
it: it never asks.** The verdict is already in the pathfinder's validity
memo, put there by another unit's `find_upath` earlier on the same frame,
and nothing cleared it. With the memo carried, Great Lakes' word moves
**11903 → 12038**, and the pieces are measured by run138.

### 24.1 The capture-free route, and why it was not the answer

The brief's first route was to call `PathFinder::valid_ucoord@00687c80`
under `tools/emu/callfn.py` on synthesized state. That answers what the
original's *function* says given state we build. So I read every function
on the path against this crate's port first:

- `valid_ucoord`, with the push order re-read off the listing
  (`00687cec..00687d34`: `nocoll` 1, `top_only` 0);
- `Unit::detect_unit_collision@00617060`;
- `CollCheck::collide_here@00682540` and `CollCheck::fill_slots@006820e0`;
- `CollCheck::move_unit@00682ad0` and `WorldData::get_coll_block@006b5350`;
- `UnitData::invalid_loc@00607c30` and `WorldData::is_cliff_at@0046f8c0`;
- the unit-grid arm of `PathFinder::calc_cost@00684e50`.

All of them agree. Emulating them would have returned this crate's own
verdict. What was different was never inside a function. It was *which
state one call inherits from another*, and only a capture sees that.

### 24.2 The readings, and what killed each

Five readings, each with what killed it:

- **A resumed search**, carrying an old memo. Dead: `1/62`'s `start_dist`
  is 0 on every block up to 11905 (§18.2).
- **A bit the index holds and this crate does not.** Dead twice over.
  From the dump: on block 11901 only five objects stand near S, all
  units, and no disc reaches (809, 442) or (811, 442). No unit leaves the
  dump in 11760..11905. From run138: its live block (50, 27) at every
  probe of 11901 holds neither bit (§24.4).
- **A stale copy of the block.** `fill_slots` answers a `nocoll` probe
  from the pathfinder's copy tree (`pathfinder +0x4c`), which also
  outlives the probe that filled it. It is real (§24.5), but run138's copy
  of (50, 27) is the live block, bit for bit.
- **Another cell of the search.** Refusing (810, 442) or (810, 443) by
  hand makes the search give up. Refusing any other cell changes nothing.
  Only S reproduces the original's plan.
- **The instrument perturbs what it records** (§23.3). Dead, and it was
  the reader's fault. `report.py calls` printed a literal `0` for every
  record's eighth argument, which the proxy never logs. The proxy pushes
  all eight to the original from its own frame. run116's collide evidence
  stands; `report.py` now prints `nocoll=?`.

### 24.3 The instrument: INFO 16

`RON_COLLIDE_PROBE` gains one record (`tools/trace/tracer.c`,
`I_COLLBLOCK`). At every `collide_here` call inside the `callwin`, it
prints the 256 bits of the probe centre's world cell twice:

- the **live** block (`World +0x134`, stride 0x1c, `+0x18`);
- the pathfinder's **copy** of that cell, when its tree holds one.

It is read-only. The two globals are the PDB's `GameAccess::world`
(`0xc06188`) and `pathfinder` (`0xe85e40`), both confirmed in
`fill_slots`' own listing (`006821c3`, `00682275`). The build has no
`popad` and no `popfd`, and its `.funcs` and patched exe are
byte-identical to the plain build's. run137's fault was not this build's:
run138 launched clean on the first try, with the same source (§24.7).

### 24.4 run138: S is never probed

`docs/RUNS.md`, run138. `1/62`'s search on 11901 expands its root at
(810, 440) with the wheel from W, and the record has W, NW, N, NE and E:
five `detect_unit_collision` brackets with their hit cells. It has **no
record at all for SE, S or SW**. Its second node, E (811, 440), does the
same: W, NW, then (memo) N, NE and E, and nothing for SE, S or SW. Its
third, N (810, 439), gets all eight. The six silent cells are exactly row
441 from x 809 to 812. A silent refusal in this loop can come from only
one place: `valid_ucoord`'s memo hit, which logs nothing.
`BRTree<int,unsigned long>::seek@00479b90` is an exact match, so those
entries are real entries for those cells.

**Who wrote them** is in the same frame's record, before `1/62` moves.
`1/27`, stopped on `1/62`, re-plans:

1. It pops its path and snaps to (38760, 21192).
2. Its `find_upath` pre-walks its goal (39000, 21192) back toward itself,
   `0x18` a step along row 441.
3. That makes five `valid_ucoord`-shaped probes (`quick 1`, `nocoll 1`) of
   (812, 441), (811, 441), (810, 441), (809, 441) and (808, 441). Each is
   refused, the hit cells in `1/62`'s own block.
4. The pre-walk reaches `1/27`'s own cell and returns
   (`find_upath@00682f30:103`).

That return is **before** `astar_path`, so it never reaches the
`kill_lists` at `:330`. The five verdicts stay in `pathfinder +0x50`, and
`1/62`'s search, three units later, starts on that memo.

### 24.5 The rule, and what this crate carries

`PathFinder +0x50` is the pathfinder's memo, not a search's:

- `valid_ucoord` caches each 48-cell's verdict there under its metric
  (`gx + gy · 16 · xs`, the same key in the pre-walk and in the search).
- Only `PathFinder::kill_lists@00687ae0` empties it.
- Every finder calls that right after its `astar_*` returns, and on none
  of its early returns: `find_upath:330`, `find_wpath@00688fc0:261`,
  `find_tpath@006897d0:182`, `find_road@00688a40:46`.
- A fresh `astar_path@00683770` resets `valid_hit` and nothing else.
- A suspend hands the memo to the unit and pops an empty one; a resume
  closes the current one and takes the unit's.

`crates/sim`: `Sim::path_memo`, used by the pre-walk and a fresh search,
and emptied by `Sim::kill_lists` after each of the four finders' searches.
The suspend and resume arms were already there (`Search::valid_memo`).

**The copy tree has the same lifetime and is ~~still not modelled~~
modelled since item 678, §26** (COLLISION §4.2's SEAM). `fill_slots` inserts copies on a miss, and
`resolve_unit_collision`'s unwind probe makes them outside any search, so
a copy can be frames old. run136's coverage has `kill_lists` only on 11896
and 11901 across 11896..11910. It is not this word: run138's copy equals
the live block.

### 24.6 What it moved

With no probe, run53's Great Lakes word moves **11903 → 12038**
(`LONG_WORD_GREAT_LAKES`). The delta at 12038: ours 4 draws, the
original's 5, parting at index 4, where the original has `Guy::set_anim
+0x97a < Unit::move_step+0x823`. That is past run136's last block, so its
widening is owed (item 571).

The value diff is nearer than the draw word.
`run136_s_word_frame_is_widened_whole` now finds **every record agreeing
from 11902 through 11921**. The 36 rows that parted on 11902..11904 are
gone, and `1/64`'s stop with them. On **11922** army 1's squad (`1/37`..`1/42`,
`1/62`..`1/64`) stands `stopped` with `last_speed 0` and `avg_speed 0` in
the original, and walks on here. Those 29 rows are pinned. Positions
agree on that block, and a stop spends no draw.
`run136_s_word_is_one_cell_of_1_62_s_search` asserts the original's plan
with nothing placed by hand.

### 24.7 What is not established

- **Why the squad stops on 11922.** That is item 571's.
- ~~**The copy tree's persistence** (§24.5), and whether any word turns on it.~~ Great Lakes 12897 did, through a suspended search's resume: §26.
- **run137's fault.** run138 ran the same source clean, which makes the
  lab's intermittent startup fault (2 of 19 plain launches,
  `docs/lab/2026-09-09-startup-cohort.md`) the likely cause, but one
  clean launch does not prove it.
- `astar_river`'s `kill_lists` has no runtime caller in any trace. It is
  not modelled.

### 24.8 Coverage

**Diff-backed**:
- §24.4's silent cells and `1/27`'s pre-walk, from run138's trace
  (`report.py calls 11901`);
- §24.6, from `run136_s_word_frame_is_widened_whole` and
  `run136_s_word_is_one_cell_of_1_62_s_search`;
- the word, from `run53_s_24000_frames_put_the_ceiling_where_run33_did`.

**Listing-backed**: §24.1's push order and §24.3's globals.

**Reading only**: §24.5's list of `kill_lists` sites and the resume and
suspend swaps. `find_upath`'s early returns are the ones this word
measured.

## 25. A Supply Wagon plans as an army: `find_wpath`'s `is_supply` arm (item 569, 2026-09-23)

Golden chapter four's Supply Wagon `1/10` is born on 1101 and sent after
its army's first member (`docs/ARMY.md` §4.3). Its `find_wpath` from
world cell (6, 44) to (13, 37) is the first one a supply unit plans in
any capture. This crate's route was one leg short, and the wagon walked
its own line from 1102. That was 552's parked "birth path", and 567's
700 units on 1277.

### 25.1 What the dump said

run133's path stack on 1101, in world cells, top first:

| | route |
|---|---|
| the original | (7, 45) (8, 44) (9, 43) (10, 42) (11, 41) (12, 40) (12, 39) (12, 38) (13, 37) |
| this crate, before | (7, 43) (8, 42) (9, 41) (10, 41) (11, 40) (12, 39) (12, 38) (13, 37) |

The original's first step is **south-east**, away from a goal to the
north-east. It goes round (7, 43), (7, 44) and (6, 43), which carry
`0x200` in `WData.flags`. This crate's cost trace priced those cells 36
and 44, the plain step plus the enemy-territory 4. That is the price
without the army mode. With it, `base <<= 5` makes each 1024 (§5). The
cause was named before any reading: the two routes differ exactly on the
`0x200` cells.

### 25.2 The rule

`find_wpath@00688fc0:218`–`241`: for an AI leader, the mode's first term
is `type +0x1e8 ≠ 0`, the attack. When the attack is 0, it asks the
unit's vslot `+0xcc`, `is_supply`, and takes the mode if that answers
yes. When the slot is the base `UnitData::is_supply@0046ce80` the call is
inlined as `type +0x2b8 & 0x40`, `unit_flags2`'s supply bit. The six unit
vtables that carry the slot all hold the base function
(`vtables.txt`), so the arm is the raw bit. It has no hero exclusion,
unlike `Sim::is_supply_unit`. No hero reaches it, because heroes are
armed. `not a worker` and `not on a river cell` follow as before (§3,
§22).

`Sim::army_mode` carries it now. With it, the wagon's route on 1101 is
the original's cell for cell. The walk from 1102 agrees, and so does
every escort post from 1277 (`docs/ORDERS.md` §24.6). The post is the
wagon's position plus the slot's offset.

### 25.3 What it moved

With `docs/ORDERS.md` §24.9, golden chapter four **1416 → 1500**, its
trace end. This section removed 89 of the widening's under-the-word rows:
20 on the wagon from 1101 and 69 on the escort from 1277.

### 25.4 What is not established

- **A hero's arm**: `unit_flags2 & 0x40` on a hero type. The listing
  takes it, and no armed hero reaches the second term. Not run.
- **A human's supply unit** jumps the whole mode block (§3). No capture
  has a human wagon planning.

### 25.5 Coverage

**Diff-backed**: §25.1 and §25.2, by
`chapter_four_s_word_frame_is_widened_whole`. Its 1101 block asserts both
routes cell for cell, and it fails on the old route with the arm
reverted. **Listing-backed**: the vtable census of `+0xcc`. **Unit
test**: `a_supply_wagon_plans_as_an_army_and_an_unarmed_plain_unit_does_not`.

## 26. A resumed search reads the blocks it copied: the pathfinder's `blocklist` (item 678, 2026-09-24)

§24.5 found that `PathFinder +0x50`, the validity memo, outlives the
search that filled it, and left the copy tree beside it unmodelled
because run138's copy equalled the live block. Great Lakes 12897 is the
first word that turns on the copy tree, and the turn is a **resume**.

### 26.1 The frame, read from the dump

`run174_s_word_frame_is_widened_whole`, rows on `1/40`–`1/42` and `1/15`,
every block 12538..12899:

- **12626 is the first parting on the squad.** `1/41`, blocked by `1/34`
  since 12623 (`collide_o 34`, both sides), holds 23 path entries here and
  20 in the original. The eight world entries agree. Every difference is
  in the flag-2, tolerance-0 entries of the 48-grid sidestep. Both leave
  (38040, 21624) west and north to (37464, 21384). From there the
  original walks north up x 37464 and east along row 21144 to 38136. This
  crate jogs east to x 37512 and up to row 21096, which routes round
  (37560..37752, 21144), where `1/66` stands on this frame.
- **The chain follows from it.** `1/41` trails the original's by three
  frames from 12662 to 12824. On 12825 the original's `1/41` stands one
  frame with no collider and `retry` 0, and the squad ungroups to kind 2
  (`do_group_move`'s follower arm). This crate's is 46 units further back,
  and it does not ungroup. After the ungroup the two sides walk different
  plans, and on 12897 the original's `1/41` meets `1/15`.

### 26.2 The readings, and what killed each

Written from the dump, before the copy tree was read or built:

- **R1 — the extra entries come from a different world plan.** Killed if
  the world entries agree on 12626. **Killed**: slots 0..8 agree, and only
  the 48-grid entries part.
- **R2 — the ungroup is 12537's mechanism by another caller** (§21.6).
  Killed if no `astar_path` failure is on 12825's path. **Killed**: the
  original's `1/41` holds `retry` 0 and `collide` 0 on 12824..12826, so no
  failure tail rolled.
- **R3 — the stand on 12898 is a collision predicate, not the plan.**
  Killed if the stand goes when the plan agrees. **Killed**: with the copy
  tree, `1/41` never meets `1/15` out of step.
- **R4 — the 48-grid search reads the collision blocks differently.** Every
  unit near `1/41` agrees on position through 12625, so the difference
  had to be in what the search reads rather than in who stands where.
  §24.5 names one input the crate did not carry. Killed if modelling the
  copy tree leaves 12626 as it was. **Held.**

No loop bound carries a premise here.

### 26.3 The rule

`CollCheck::fill_slots@006820e0(x, y, size, nocoll)`, called at the top of
`collide_here@00682540`:

1. The slots are the world cells the probe box `(x ± size, y ± size)`
   touches, in unit cells `>> 4`: the first always, the other three when
   the box crosses into them, and none off the map.
2. With `nocoll` set and the tree allocated, each slot the tree holds
   (`Tree<CollBlock*,int>::seek` on `cy · xs + cx`) is read from its copy
   and nothing else.
3. Every other slot is read live, through §4.2's region gate
   (`docs/COLLISION.md`).
4. With `nocoll` set, each slot that was read live is copied, `0x300` bits,
   empty when the gate refused it, and `ordered_insert`ed into the tree.

`nocoll` is set by exactly two callers. One is `valid_ucoord@00687c80`,
which calls `detect_unit_collision(x, y, quick 1, …, nocoll 1, 0)`, so
every probe of a 48-grid search and of `find_upath`'s pre-walk reads the
tree. The other is `resolve_unit_collision@005f9d30`'s stack unwind.

**The tree's lifetime is the memo's** (§24.5), from the same functions:

- `PathFinder::init@00689ec0` allocates it once;
- `kill_lists@00687ae0` deletes every copy (its `+0x4c` loop);
- `astar_path@00683770`'s suspend (`:506`–`508`) hands it to the unit at
  `+0x114` and pops an empty one;
- its resume (`:253`–`288`) deletes the current tree and takes the unit's.

So a copy dies at the next `kill_lists`, which every finder calls after
its search. A copy made where no search follows — an unwind probe, a
pre-walk that returns early — is what the next search reads. **And a
suspended search keeps the copies it made**, so its resume reads the
blocks as they stood when it suspended.

### 26.4 What happened on 12623..12625

The trace below is from a debug print in this crate, now removed.

- **Sim-frame 12623**: `1/41`'s `find_upath` copies five world cells,
  (48..50, 27..28), runs over its `limit` and suspends. The tree goes to
  the unit.
- **Sim-frame 12624**: `do_move`'s suspended-search block resumes it. The
  resume takes the unit's tree, and every `valid_ucoord` probe of a cell
  not yet in the memo reads those copies. Cell (48, 27)'s copy holds `1/66`
  on (782, 442..444) and (783, 444). The live block holds it on (783, 441):
  it walks north-east about 30 units a frame. Cell (49, 28) differs in 19
  unit cells the same way.
- This crate read the live blocks, found `1/66` across row 440, and routed
  round it. The original found row 440 open.

The memo was already carried across the suspend (§24.5). The copies were
not, and that was the whole difference.

### 26.5 What this crate carries

- `Sim::coll_copies`, keyed on the world cell. Each value is the gated
  16 × 16 unit cells, one row per word.
- `collide.rs`'s `ProbeSlots::copied`, filled by `probe_slots` for a
  `nocoll` probe as steps 1–4 above describe.
- `Sim::kill_lists` empties the tree.
- `Search::block_copies` carries it across a suspend and back.

**Pinned capture-free**: `collide::tests::a_nocoll_probe_reads_the_block_the_last_one_copied`.
Against live-only reads it fails on "the copy still has it where it
stood".

### 26.6 What is not established

- **The bits past 256.** A `CollBlock` is `0x300` bits (`fill_slots`' own
  constructor), and this crate's block is 16 × 16. What the other 512 hold
  is not read. No probe here reads past the first 256.
- **A copy taken from another region.** The copy is the gated slot, and a
  later probe from another region reads it ungated (COLLISION §4.2).
  Great Lakes is one region, and no capture tests it.
- **The unwind probe's copies.** `resolve_unit_collision`'s unwind fills
  the tree outside any search. This crate does the same, but no diff has
  yet isolated a word that turns on it.

### 26.7 Coverage

**Diff-backed**:

- `run174_s_word_frame_is_widened_whole`: the move's value diff. 74 keys
  on `1/40`–`1/42` from 12626 are gone, and nothing on the squad parts
  through 12899.
- The long word, by `run53_s_24000_frames_put_the_ceiling_where_run33_did`.

**Listing-backed**: `fill_slots`' four steps; `valid_ucoord`'s
`nocoll 1`; the suspend and resume swaps.

**Reading only**: none that the diff does not reach. The unwind probe's
copies are carried, and no diff has isolated them.

## 27. The world probe reads the cell: `invalid_loc`'s cell arm (item 776, 2026-09-25)

**Established** by a diff against the original's own priced steps, and
the listing. **High** confidence for the clause on forest cells. Its
`0x40` bit and the forest-walker exemption rest on the listing alone
(§27.4).

### 27.1 The word, and what the disk said

Great Lakes' word 17128 was `1/9`'s re-plan. Under it, `1/9` and `1/72`
took other world paths to `1/2022` from tick 17087. The original's `1/72`
went right round the Pyramids (`1/2026`) in nine entries, and ours cut
north-west through cell (56, 18) in three. The original's `1/9` went by
(55, 20), where ours went by (55, 19).

**run240** (`docs/RUNS.md`) printed the world on block 17087, the state
the tick plans over, and its trace proxied every `calc_cost` of the
game. `run240_s_world_at_17087_is_the_original_s` reads both.

- **The world is the original's where the searches look.**
  - One cell of 3,600 parts: (2, 40).
  - The tile masks part only on `0x4`, the residue run72's and run189's
    pins carry.
  - who=1's danger map agrees whole.
  - The fog parts only on other players' bits, round the Pyramids.
- **The first priced step to part is `1/9`'s first expansion.** The 82
  steps before it agree.
  - From its root, cell (56, 19), ours priced N into (56, 18) at 200 and
    W into (55, 19) at 198.
  - The original priced neither: its validity probe refused both.

### 27.2 The clause

`valid_wcoord` calls `invalid_loc(tile, 1, timeout > 1, 0, 1, 0)` (§6).
Both probes land on plain tiles (city radius and `BAD_PATH`, one of
them road), which every tile test passes. The refusal is the land arm's **first** test, in the listing
at `00607e6f`–`00607ead`:

```
flags = cells[(ty >> 2) * xs + (tx >> 2)].flags      ; 0x1c stride, +0x134
if (flags & 0x70)                                     ; mountain | forest | 0x40
   && !((flags & 0x20) && (unit_masks2 & 0x4000))     ; a walker in a forest cell
   && param_3 && param_6:
    return 2
```

`param_6` is forced to 1 when the path's top entry carries `flags & 4`
(the function's first statement). Cells (56, 18) and (55, 19) carry
forest, road and `NEARBLOCK`. Cell (56, 19) carries road and `NEARBLOCK`
without forest, so it passes.

So the world search refuses a forest *cell*, where the tile test would
only refuse its forest *tiles*. This crate's land arm began at the tile;
`Sim::invalid_loc` (`crates/sim/src/path.rs`) now reads the cell first.

The only caller that passes both flags is `valid_wcoord`. The others pass
`param_6` only through a transport-flagged path top, as the original does.

**Pinned capture-free**:
`path::tests::a_world_probe_refuses_a_forest_cell_on_plain_ground`.
Under the tile-only rule it fails on its first assertion.

### 27.3 What moved

- **All 303 priced steps of tick 17087** agree with the original's, key
  and price: six world searches, among them `1/9`'s 42 and `1/72`'s 169.
- **The value diff** (`run226_s_word_frame_is_widened_whole`): every row
  of `1/9` and `1/72` from 17088 goes.
  - **Nothing parts on 17088..17181.**
  - The floor goes 398/38/1256 → 398/0/876.
- **Great Lakes 17128 → 17181**, inside run226.
  - On 17181 ours spends 11 draws against 5, parting at index 0: ours
    spends three `Leader::create_buildings+0xffb`/`+0x1017` pairs the
    original does not.
  - Block 17182's first rows are who=1's `MAKE` slots 0, 1, 2 and 8.
    Here they hold three wonders (types 526, 528 and 527, category 8);
    there they hold nothing.
  - That is a row, not a cause.

### 27.4 What is not established

- **The `0x40` bit.** Nothing names it (`crate::world::cell`). No cell a
  run226 search reaches carries it without forest or mountain beside it.
- **The forest-walker exemption.** It is read from the listing.
  `forest_walker` is still this crate's seam (`unit_masks2 & 0x4000`,
  the Iroquois), so the exemption never fires here.
- **The other callers under a transport-flagged top.** Such a caller
  passes `ignore_buildings` and a forced `param_6`. No capture on disk
  plans a transport leg across a forest cell.

### 27.5 Coverage

**Diff-backed**:

- the clause on forest cells, by
  `run240_s_world_at_17087_is_the_original_s`: every priced step of tick
  17087, and the world beside it;
- the move, by `run226_s_word_frame_is_widened_whole`;
- the long word, by `run53_s_24000_frames_put_the_ceiling_where_run33_did`.

**Listing-backed**: the clause's operands (`00607e8f`–`00607ead`).

**Reading only**: the `0x40` bit and the walker exemption (§27.4).

## 28. An army under an attack-to plans without the danger map: `no_danger`'s order arm (item 773, 2026-09-25)

**Established** by a diff against the original's own priced steps, and
the listing. **High** confidence for the `ATTACK_TO` arm, which run248
exercises. The `GROUP_ATTACK_TO` and gaia arms rest on the listing alone
(§28.4).

### 28.1 The word, and what the disk said

East Indies' word 17189 was `1/55`'s blocked stand by `1/60`, a frame
before the original's. Under it, army 0's column (`1/55`, `1/57` and
`1/58`) takes an `ATTACKTOORDER` on tick 17146 to three formation slots
eight cells north, and each plans a world path. `1/55`'s and `1/57`'s
agree. **`1/58`'s parts on block 17147**: the original's goes round the
west in 18 entries, in the column; ours went straight north up
x = 35592 in 10. The original's `1/58` then walks 104..290 from `1/57`
and `1/55`, and it is the nearest unit on `1/55`'s half steps of 17183
and 17185 (247, 275). Ours' was a thousand away, so ours' `1/55` took
whole steps, came 30 ahead, and stood on `1/60` a frame early.

**run248** (`docs/RUNS.md`) printed the world on block 17146 and proxied
every `calc_cost` of the game. `run248_s_world_at_17146_is_the_original_s`
reads both.

- **The world is the original's.** No cell, no danger value and no tile
  mask beyond run240's `0x4` residue parts. The fog parts on 16
  half-cells, far from these searches.
- **The first priced step to part is the 32nd of tick 17146**, in `1/48`'s
  search: into (34176, 38016), 60 here against 116 there. Every
  difference in that search is who=1's danger / 8 at the step's region:
  −450 / 8 = −56 at (22, 24), and −312 / 8 = −39 at (22, 23). The
  original priced the step without the danger map, and ours with it.

### 28.2 The clause

`astar_path`'s prologue writes `no_danger` (`+0x7c`, §2's table) from
four terms, in this order (listing `00683866`–`006838a3`):

```
a = UnitData::get_action(unit)                    ; 00608450
if a && a->get_type() == ATTACK (0xa)  -> 1      ; 683873 / 683876
if who >= 8 (unsigned)                 -> 1      ; 68387b
if order_type(unit) == ATTACK_TO (2)   -> 1      ; 683888
if order_type(unit) == GROUP_ATTACK_TO (0x15) -> 1 ; 683895
else                                   -> 0      ; 68389a
```

The decompile says the same (`astar_path@00683770:119-127`), and so does
§2's table row. This crate carried the first term only
(`Sim::no_danger_mode`, `crates/sim/src/path.rs`). An army under an
attack-to therefore priced its own city's negative danger. Near who=1's
city that makes a step up to 56 cheaper, which is enough to open the
straight road north for `1/58`.

**Pinned capture-free**:
`path::tests::an_attack_to_order_plans_without_the_danger_map`. Under the
one-term rule it fails on its first case.

### 28.3 What moved

- **All 28,828 priced steps of tick 17146** agree with the original's,
  key and price: every world search of the tick, `1/58`'s among them.
  The first 31 agreed before the arm.
- **The value diff** (`run233_s_word_frame_is_widened_whole`): every row
  of `1/55`, `1/57` and `1/58` goes (22, 13 and 25), and `1/55`'s stand
  by `1/60` falls on 17190 on both sides. `1/63` loses two rows. The
  floor goes 293/363/1,139 → 293/312/1,036.
- **East Indies 17189 → 17403**, inside run233. On 17403 ours spends 7
  draws against 6, parting at index 0: ours spends `Unit::do_move+0xe84`,
  and the original a `Guy::inc_time` wrap. Under it on 17403 the
  original's `1/60` walks under an `ATTACK_TO` and ours' under a
  `GROUP_ATTACK_TO`, to another spot. From 17363, `1/67`..`1/69` part on
  `group` (−1 here, 69 there). Those are rows, not a cause.
- Great Lakes holds at 20568. The East Indies endpoint moves 3 → 1
  unlinked, 2 → 1 and 3 → 6 on the buildings, with `off` holding at 47.

### 28.4 What is not established

- **The `GROUP_ATTACK_TO` arm.** It rests on the listing (`683895`). The
  column's order on 17146 is an ungrouped `ATTACK_TO`, and no search in
  run248 is known to run under the grouped one.
- **The gaia arm.** It also rests on the listing (`68387b`). An animal's
  world search never priced danger in a diff before, and none is on
  run248's tick.
- **The 16 fog half-cells** where the original's who=1 bit is set and
  ours is not. No search of tick 17146 prices one, so they changed no
  step here. Their writer is not read.

### 28.5 Coverage

**Diff-backed**:

- the `ATTACK_TO` arm and the world on block 17146, by
  `run248_s_world_at_17146_is_the_original_s`;
- the move, by `run233_s_word_frame_is_widened_whole`;
- the long word, by
  `run54_s_24000_frames_are_where_the_second_map_s_word_now_parts`.

**Listing-backed**: the four terms and their order
(`00683866`–`006838a3`).

**Reading only**: the grouped and gaia arms (§28.4).

**Unit-tested, made to fail on purpose**:
`path::tests::an_attack_to_order_plans_without_the_danger_map`.

## 29. `is_attacking` is live for an attack order: a group's walk home plans as a citizen (item 899, 2026-09-26)

**Established** by a diff against the original's own priced steps (run240's
proxies, every `calc_cost` of Great Lakes from frame 0 to 17093), by the
PE's vftables and by the linker map. **High** confidence for the
`AttackOrder` arm, which Great Lakes exercises twice; the `GroupAttackOrder`
and `StrafeOrder` arms rest on the map and the bytes alone (§29.5).

### 29.1 The frame, and what the disk said

Great Lakes' word was **20800**: on block 20801 ours' `1/60` stood blocked
by `1/64` (`collide 1`, `collide_who 1`, `collide_o 64`) and the original's
walked. `1/60`'s world route already parted on run294's first block, 19840:
22 path slots, slot 15 (32640, 24960) here against (32640, 24192) there,
the same `x` and a `y` one cell south, in stretches that re-join. The route
is the bottom half of a 108-entry stack planned on **17656**, when the AI
sends the four walkers out to the far point (2856, 31656) and queues their
walk home: the home leg is a group plan from the far point, on
`grouppath`, with `1/60` as its leader.

No dump compares a value on 17351..19839 (parked 796), and the plan's frame
is inside that gap. But the shape was on disk already: `PROBE_PLAN_PARTED`
(`docs/ORDERS.md` §17.6) is **the same shape one probe earlier**, the walk
home of 8186's six, 22 entries a cell or two south of the original's.
**run240's trace proxied every `calc_cost` of the game to 17093**, and until
this item only tick 17087's 303 had been read.

### 29.2 The measurement

`run240_s_every_priced_step_is_the_original_s` walks all 522 frames that
price a step, 105,493 steps. Before this item **two** parted, and one was
8186: the group's search from the far point (the first of the tick's seven
`astar_path` calls, the only one on `grouppath`, `00ee1538`) prices its
first step **10216 here against 216 there**, and a `0x200` cell **1028
against 36**. Those are exactly `calc_cost`'s two army terms (§5: `+10000`
on `tcost >= 5`, `base << 5` on `NEARBLOCK`): the original ran the search
with `army` (`pathfinder +0x70`) off, and every other search of the tick,
and all 98 other group plans the trace holds, with it as this crate had it.

What set 8186 apart was the leader's **current order**. Printed at the plan,
`1/40`'s list is `[Attack, Move home]`: the out leg's `ATTACK_TO` has
already been replaced by the attack it arrives to make, and the walk home is
queued behind it. Every other group plan's leader holds a move.

### 29.3 The clause: `AttackOrder::is_attack` answers 1

`find_wpath`'s mode block calls `UnitData::is_attacking@0060a5b0` at
`006896b9` with `ecx` still the unit (`ObjectData::is_worker` leaves it).
`is_attacking` loads the current order from the list's head node and
tail-calls its vftable slot `+0x18`, `is_attack` (`0060a5f0`). §22 read
that slot in seventeen vftables. The order list holds `UnitOrder *`, so the
vftable is the one at the order's `UnitOrder` subobject, and for three
classes that is a **secondary** vftable the census did not read. Read out
of `riseofnations.exe` (slot `+0x18` is the seventh dword):

| vftable | `+0x18` |
|---|---|
| `??_7UnitOrder@@6B@` `00b474f0`, `??_7MoveOrder@@6B@` `00b4a12c`, `??_7TargetOrder@@6BUnitOrder@@@` `00b47760` | `0041bff0` (`return 0`) |
| `??_7AttackGroundOrder@@6B@` `00b49f1c` | `0048384b` (thunk to `0041bff0`) |
| `??_7AttackOrder@@6BUnitOrder@@@` `00b47628`, `??_7GroupAttackOrder@@6BUnitOrder@@@` `00b491fc` | **`0047ef8e`** |

`0047ef8e` is `sub ecx,[ecx-4]; jmp 0041e0e0`, and `0041e0e0` is `mov eax,1;
ret`. The linker map names both ends: `?is_attack@AttackOrder@@UBEHXZ`,
`?is_attack@GroupAttackOrder@@UBEHXZ` and `?is_attack@StrafeOrder@@UBEHXZ`
at `0041e0e0`; `@UnitOrder@@`, `@AttackGroundOrder@@` and
`@AirAttackGroundOrder@@` at `0041bff0`.

So **`army = military && !worker && !is_attacking && !river`**, with
`is_attacking` meaning the current order is an `ATTACK` (a group attack or a
strafe, too). A unit marching under an `ATTACK_TO` is still an army (§22's
correction stands); a unit standing on its attack is not. The group plan's
forced arm (`706190`, a group with an army) sets the flag before the call
and is untouched. Built in `Sim::army_mode` (`crates/sim/src/path.rs`);
`GroupAttackOrder` is not modelled in this crate, so the arm reads
`Body::Attack` and `Body::Strafe`.

### 29.4 What moved

- **All 522 frames of run240's priced steps but one agree**, 8186's 2,319
  among them; the one standing is 15986 (below). 
- **17656**: `1/60`'s walk home is the original's. On run294 its 22 path
  rows of 19840 and 14 later keys close; on run243 its 23 rows of 20500,
  its 13 to the word and its runway close, and none opens. **The value diff
  on the old word's block, 20801**: `1/60` at (35873, 23502), `collide 0`,
  `collide_who −1`, `collide_o −1`, `stopped 0`, path length 11 to
  (44851, 22480), on both sides; `1/64` standing at (36600, 23304),
  `stopped 1`, `collide_frame 11905`, on both sides. Ours' `1/60` was
  blocked by `1/64` (`collide 1`, `collide_who 1`, `collide_o 64`).
- **Great Lakes 20800 → 24000, the trace's own end**: no frame of run53's
  trace parts, 0..23999. The endpoint at 24001 goes **14 → 0 off**.
- East Indies holds at 23182.

### 29.5 What this has *not* established

- **The `GroupAttackOrder` and `StrafeOrder` arms** rest on the map and the
  bytes. This crate models neither a group attack order nor a planning
  strafer.
- **17351..19839 and 20819..23959 are compared by the draw stream alone.**
  No dump holds them; the plan on 17656 is shown right by its product
  (run294's path rows), not by a priced step.
- **15986**, the one frame of run240 whose steps still part: `1/73`'s
  fourth step into (45696, 13440) at depth 2 prices 44 here against 32
  there, on a nine-step search that is not a group's. It parts no draw and
  names no score.
- `no_danger` (§28) is a separate prologue term and is not touched.

### 29.6 Coverage

**Diff-backed**: every priced step of Great Lakes to 17090
(`run240_s_every_priced_step_is_the_original_s`); tick 8186 whole, the
group's 360 steps priced as a citizen's
(`run240_s_tick_8186_prices_the_walk_home_as_a_citizen_s`); 17656's route
through run294 and run243; the end through run80 and the endpoint.
**Bytes- and map-backed**: the vftable table above.
**Unit-tested, made to fail on purpose**:
`path::tests::an_army_is_armed_unworked_and_off_the_river`. With the term
out, it fails, and so do both run240 tests (item 899's journal).



## 30. Large units recover on their own stride (item 1431, 2026-10-01)

East Indies run583 widens the frame-16762 word, every record and unit.
Before the word, block 16761's Transport Galleon 1/189 had nine path entries
against sixteen and `start_dist` 1056 against zero. Its recovery suspended
here while the original completed. The next block's x was 38904 against
38934. The collision itself agreed after item 1427.

The implementation still fixed the unit-grid stride at one. The existing
§4 specification already states the missing rule: `astar_path@00683770`
derives `max(1, (collision + 1) / 2)` for the 48-unit grid, and one for
the other grids. This Transport Galleon's collision radius is three, so it searches
in 96-unit steps. An in-decision probe on frame 16760 measured the old
search suspending after 503 probes with limit 500; changing the stride
lets it finish. Raising the budget is not the correction.

Large diagonal expansions test every intermediate 48-unit position,
short-circuiting on refusal. Cardinal expansions test their endpoint.
The diagonal validity memo uses position-unit offsets from the current
node's metric, while endpoint node metrics use direction-grid offsets.
This surprising distinction is confirmed in the executable listing
`00684350`–`00684371`, not inferred from decompiler local names. The Rust
implementation expresses the stride and probe sequence directly; no
original function body is transcribed.

With both stride and diagonal probes, the word moves **16762 → 16878**
(+116). Whole-window differing field keys drop **1104 → 878**. On block
16761 the path length is sixteen and start_dist zero; on block 16762 x
is 38934. Across all 257 blocks, every compared field of 1/189 agrees
except the standing `form` field (-1 against 0). The run583 widening
asserts that stronger per-unit result in addition to its aggregate pin.
The synthetic large-recovery test rejects jumping a refused intermediate
point and checks both small and large strides. The coverage driver moves
to the new word's window. Other open-map words are unchanged.

The next word is still inside run583. Multiple army members' stance
first differs on block 16879; its cause is not established by this item.
The captured Transport Galleon path backs the reached large-unit behavior. Other
collision radii and the unusual memo-key generalization retain reading-only
review debt; no independent reading was performed in this single-agent run.


## 31. AI transport types clear the terrain preference (item 1433, 2026-10-01)

run583 places Transport Galleon 1/172's first disagreement at block
16933: path length 26 against 33, start_dist 2064 against zero. The next
block has collide 2 against 1 and position (12744,24744) against
(12744,24714). On 16940 the original's passenger 1/91 has disembarked at
(12792,24264); ours remains inside 172 at its old (29281,38031).
The random word parts at 16940, four draws against five. These whole-record
rows, not the first idle label, identify the recovery as the upstream cause.

A probe inside the 16932 decision confirms collision radius three and
stride two after item 1431. The search still suspends at 506 valid probes
against limit 500. Its north step is valid, but `avoid_land=1` prices an
embark later in the original's detour at 2000 before the unit-grid shift.
The existing §4.1 exception was still a seam: for a same-region search,
a type with `unit_flags & 0x10` and a unit with `unit_masks & 0x40000`
sets both avoidance modes to zero. `astar_path@00683770` tests both bits
before choosing the ordinary water/land modes. Here the latter bit means
AI-controlled; it is not a terrain or collision-size flag.

The implementation now checks the transport type flag and `ai_driven`
inside the same-region branch, for every grid. Human transports, ordinary
AI units, and the separate cross-region `0x20` rule (not "fleeing") retain their own
modes. The synthetic matrix covers both predicates independently, land
and water, and all three grids. The simulator's existing `ai_driven`
helper is not a human slot's takeover: leader bit 8 is set at lobby time on
a human seat and cleared per unit by a human command, never at run time
(twenty-fourth pass, A4 rows 17–18); a defeat clears the unit bit
(`Leader::defeat@006ecb00`, group 6: `unit_ai_bit`).

The measured word advances **16940 → 17507** (+567); the other open-map
words are unchanged. run583 differing field keys drop **699 → 286**.
Every compared field for passenger 91 and transport 172 now agrees,
except the transport's standing form field. The named widening asserts
that whole-record result: the 33-entry path, zero start_dist, correct
movement and disembarkation are included. The new random word lies past
run583's last block, so the floor stays 16940 and item 1434 owes run585
before any mechanism is booked on 17507.

**Identity correction.** The original GUY records for 172, 188 and 189
name type 321, TRANSPORTGALLEON in the PDB enum. Internal `Unit.ty=271`
is an index into this crate's compact type table, not original type 271
(HOWITZER). Earlier item 1427/1431 write-ups used the latter name in
error. COLLISION §21 and §30 above are corrected; the historical journals
remain intact and this is their correction record. No simulation identity
or comparison changed as part of that terminology correction.

The reached path and passenger state are diff-backed. Generalization of
the exception outside these captures retains independent-reading debt.
No original body was transcribed and no blind review was simulated.
