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
| +0x0c | 0x4c | `blocklist` | init/suspend | `Tree<CollBlock*,int>`; freed each teardown (not read by the unit search) |
| +0x10 | 0x50 | `validlist` | init/suspend | `BRTree<int,ulong>` — `valid_ucoord`'s memo, `metric → 0/1` |
| +0x14 | 0x54 | `pathing_unit` | every wrapper | the unit |
| +0x18/0x1c | 0x58/0x5c | `sx, sy` | every wrapper | the unit's **tile** (`div3[pos>>6]`) |
| +0x20 | 0x60 | `dbg_collisions` | nothing | dead (printed by `log_data`, never read) |
| +0x24 | 0x64 | `anti_unit` | `find_upath` = 1 | **"this is the unit grid"** — not the `anti` argument. Gates the `limit` budget, the suspend path, +5-per-probe at `0xc0`, and flag 2 on reconstructed waypoints |
| +0x28/0x2c | 0x68/0x6c | `offx, offy` | `find_tpath` | target unit's sub-tile offset, `pos % 0xc0 − 0x60`; **no readers anywhere** (writers survey) — dead. `astar_path` derives its own `toff` instead, and from the **order**, not the target (§4.1) |
| +0x30 | 0x70 | `army` | `find_wpath`; also `Group::action_move_near` (which inlines `find_wpath_army`'s body — the named function has zero callers) | military, not a worker, not attacking, start cell not river-flagged (§3) |
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
  `is_attacking`, **and** the start cell's flags lack `0x100` (river);
  `worker = 1` iff `is_worker`.
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
  `avoid_land = 1` when the order has `flags & 0x20`.
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
   budget-ended `0x300` search instead **drains the open list
   keeping the node nearest the goal** (`vector_dist`), reconstructs the
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
unit's own tile** (`sx/sy`) with `+250`/`+1000` and no shift. A unit that
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
lets an armed unit pass its own side's buildings (`find_any_building_at`
owner test); `p6`/`p7` relax the water refusal for a transport-forced unit
(`unit_masks & 0x800000`), and `p6` is also forced on when the unit's path
top carries flag 4. Dispatch is on the type's domain (`+0x218`: 0 land, 1
sea, 2 air — air is always valid); the land arm refuses forest (except
forest-walkers, `unit_masks2 & 0x4000`), mountains, cliffs and water; the
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
Big-unit strides and the transport tail are implemented as dormant seams;
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
