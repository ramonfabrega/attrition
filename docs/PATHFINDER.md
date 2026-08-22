# The pathfinder

**Status: not read. This is the brief, not the mechanic.** Nothing below is
established behaviour; it is the scope, the entry points, the state layouts
and the oracle, gathered so that the first reading starts at the decompile
instead of at a survey. Whoever runs that reading **replaces this document
with the real one**, written the way `docs/ORDERS.md` and `docs/COMBAT.md`
are: how it was established, how confident it is, and what it has *not*
established.

**Who should read it: Fable.** Per the working agreement's model split, a
first decompile reading is the delicate core — the errors it prevents are
predicates a less careful read produces confidently, and a wrong spec is the
expensive failure. The survey below was Opus's job and is done.

---

## 1. Why this one is next

`docs/ORDERS.md` §4.6 names `PathFinder` as the next mechanic and leaves it
as a **stub**: `find_wpath` pushes the goal and returns, so the simulation
walks straight lines. Three things follow from that, and all three are now
visible rather than argued:

- Every path stack in the harness disagrees with the original's beyond the
  first segment — 750 `path-length` and 33 `path-to` disagreements over 432
  frames on `gamelog-run6` (`docs/DATALAYER.md` §3.1).
- The one unit that currently tracks the original end to end
  (`gamelog-run6`'s `0/1`) does so on a walk short enough not to need
  planning. The second woodcutter's citizen, `0/2`, parts from it on frame 4
  by a few position units and never fully rejoins.
- The AI (queue item 8) issues orders that are *executed* through this. Doing
  AI first would stack a second stub on this one.

## 2. The oracle, which is new

**This is the first mechanic to have a per-frame numeric oracle before a line
of it is written.** At `UNITS=3` the original logs each unit's own
`Stack<PathData>` — bottom (the goal, with the final flag) first, then the
waypoints, the top being where the unit is walking now (`docs/ORDERS.md`
§11.1) — and the harness already reads it and compares it segment by segment:

```
rondata <install> --gamelog <Logs/gamelog-run6-…-builds7.txt> --diff
  → by kind: … path-length 750, path-to 33
  → who 1 o 2: first order disagreement at frame 4 — PathLength { ours: 1, theirs: 2 }
```

`OrderMismatch::PathLength` and `PathTo` are deliberately **not scored**
today, precisely because they are measuring the stub (`crates/rondata/src/
diff.rs`). When the pathfinder lands, **making them score is the acceptance
test**, and "how many frames until a path segment disagrees" is the number
the work is judged on. `gamelog-run6` is 432 frames of it; a longer or
busier run is one drive away (`docs/ORACLE.md`, the recipe).

A second oracle exists and is untried: `gamelog.ini` has a **`PATHFINDER`
category** (it is at 1 under `[Misc Logging]` already), and the binary has
`PathFinderOut::registration@00687e40`, `dbg_tree_depth@00688360` and
`dbg_draw_failures@00688730`. What that category actually writes, and at
which detail threshold, is the first cheap thing to find out — it may hand
over the search's own node counts, which no reconstruction can otherwise
check.

## 3. Scope

**In.** `PathFinder::astar_path` and the search it runs: the open list, the
closed list, the node metric, the expansion order, the termination and the
node limit; `calc_cost`, the per-step cost function, and what it reads;
`valid_wcoord`/`valid_tcoord`/`valid_ucoord`, the three grids' passability
predicates; `PathFinderData`'s per-search state and who sets each field;
`PathNode`; `first_open_node`, `add_to_openlist`, `find_node_open`,
`find_node_closed`, `get_estimate` (the heuristic); the suspend/resume path
(`saving`, `find_upath_restore`, `find_road_restore`); `repaths[who]` and
the quadratic node limit; what the three wrappers hand in and take back
(already mapped, §5 below — verify, do not re-derive).

**Out, at least at first.** `astar_river` (748 lines) and
`astar_caravan_road`/`calc_road_cost`/`valid_roadcoord` — the map-maker's and
the caravan's searches, not a unit's; name them and move on.
`Recycler<PathNode>`, `Tree`, `BRTree` are allocator and container
boilerplate: read enough to know the ordering the containers impose on the
open list (that *is* behaviour) and no further. `Group`/`Form` path sharing
(`find_wpath_army`) belongs with the group orders.

## 4. The entry points, with what they cost to read

From the export at `~/ghidra-projects/decomp/funcs/` (`tools/ghidra/`).
`PathFinder` has 34 methods; these are the ones the mechanic is in.

| function | lines | what it is |
|---|---|---|
| `PathFinder::astar_path@00683770` | **977** | the search itself; the mechanic |
| `PathFinder::calc_cost@00684e50` | **509** | the per-step cost; the balance of the mechanic |
| `PathFinder::first_open_node@00687970` | 78 | what "cheapest" means to the open list |
| `PathFinder::kill_lists@00687ae0` | 64 | teardown; what survives a search |
| `PathFinder::init@00689ec0` | 60 | the per-search setup |
| `PathFinder::clear@00688150` | 41 | between searches |
| `PathFinder::valid_ucoord@00687c80` | 35 | the 48-unit grid's passability |
| `PathFinder::valid_wcoord@00687da0` | 27 | the world-cell grid's |
| `PathFinder::empty_lists@00689c70` | 19 | |
| `PathFinder::add_to_openlist@00687aa0` | 14 | thin; the ordering is in the container |
| `PathFinder::valid_tcoord@00687d60` | 14 | the tile grid's |
| `PathFinder::clear_unitvals@00687ad0` | 9 | |
| `PathFinderData::get_estimate@00688310` | 18 | **the heuristic** — small, and it decides everything |
| `PathFinderData::find_node_open@006882b0` | 22 | |
| `PathFinderData::find_node_closed@00688270` | 17 | |
| `PathNode::init@00688a00` | — | |

Named and out of scope: `astar_river@00686690` (748),
`astar_caravan_road@00685990` (388), `calc_road_cost@00686300` (149),
`PathFinderData::valid_roadcoord@00688740` (112),
`calc_river_cost@00687930` (15), `building_danger@00685900` (15).

## 5. The state, from the PDB

`PathFinderData`, size `0x88` — one global, reused per search, which is why
who writes each field matters as much as who reads it:

```
+0x00 openlist      Tree<PathNode*,int>*         +0x40 limit
+0x04 openlistrefs  BRTree<TreeNode<…>*,ulong>*  +0x44 saving
+0x08 closedlist    BRTree<PathNode*,ulong>*     +0x48 avoid_land
+0x0c blocklist     Tree<CollBlock*,int>*        +0x4c avoid_sea
+0x10 validlist     BRTree<int,ulong>*           +0x50 valid_hit
+0x14 pathing_unit  Unit*                        +0x54 scouting
+0x18 sx, +0x1c sy  TCoord (the unit's tile)     +0x58 can_transport
+0x20 dbg_collisions                             +0x5c dbg_view_failures
+0x24 anti_unit                                  +0x60..0x80 road_* (nine)
+0x28 offx, +0x2c offy                           +0x84 show_debug
+0x30 army, +0x34 iroquois, +0x38 worker, +0x3c no_danger
```

`PathNode`, size `0x24`:

```
+0x00 x, +0x04 y (Coord)   +0x10 value      +0x1c z_val (short)
+0x08 length               +0x14 timeout    +0x1e transport (uchar)
+0x0c estimate             +0x18 metric     +0x1f building (uchar)
                           (ulong)          +0x20 parent (PathNode*)
```

`length` + `estimate` + `value` + `metric` is four numbers where A\* needs
two, and **which of them the open list is keyed on is the first question the
reading has to answer** — `Tree<PathNode*,int>` carries a `current_metric`
and `TreeNode` a `metric`, so the ordering may not be the `value` the cost
function computes. `iroquois` next to `army` and `worker` says the nation
layer reaches in here; `no_danger`, `avoid_land`/`avoid_sea`,
`can_transport` and `anti_unit` are per-search modes that a reading which
only follows the common case will miss.

## 6. What is already established — verify, do not re-derive

`docs/ORDERS.md` §4.6 has, from the orders reading and its blind second
reading:

- `Unit::find_path@005fb910`, the straight-line verifier that decides whether
  the pathfinder is called at all — and its "far and reachable" rule
  (cell-Manhattan > 4), which is why a near move never touches `PathFinder`.
  **One open question here is now urgent and is this reading's to answer**
  (`docs/ORDERS.md` §4.6): the march as transcribed has a fixed point, and it
  hung `crates/sim` on a plain move order, because `sinx(ang, spd)` truncates
  a small cross-axis component to zero while that axis' remainder is still
  bigger than one step. Either the original's trig does not truncate that way
  or its exit test is not the one we transcribed. Settling it is a few
  minutes in the listing at `0x5fb910`, and the answer changes the RNG
  stream, so it is worth doing early rather than at the end.
- The **calling convention of all three wrappers**: they take the unit's own
  `Stack<PathData>`, pop the goal off its top, push back what the unit should
  walk (top first), and return the stack length — `0` for no path
  (`−(flags & 1)` in `find_wpath`'s A\* case), `−1` for a goal off the map.
- Each wrapper's pre-A\* work: `find_wpath@00688fc0` on world cells
  (`0x300`), `find_tpath@006897d0` on tiles (`0xc0`),
  `find_upath@00682f30` on 48-unit cells (`0x30`), each with its
  same-cell/flyer short-circuit, its walk-the-goal-back-toward-the-start
  loop, its Manhattan cut-off, and what it pushes before calling
  `astar_path(step, anti)`.
- `find_upath`'s post-processing: on failure pop a non-final top **and kill
  the unit's current order** unless a move with `retry != 0`; on success with
  more than three entries, drop a top equal to the unit's position and
  compact collinear `flags & 2` side-steps.
- `repaths[who]` (`GameDaemon`): halved each frame (to 0 below 3),
  incremented per pathfinder call under collision, shrinking `find_upath`'s
  limit as `500 / repaths²`.
- `go_around_building@005fc350`, the non-pathfinder detour.

Treat all of that as the *interface* the reading has to meet, and re-check it
in passing rather than trusting it: it was read from the caller's side.

## 7. The traps this project keeps re-learning

In the order they have cost something:

1. **Read the loaders.** Three mechanics in a row were wrong about a field
   because it is *derived* at load rather than read from a column — the
   Military-epoch prerequisite (tech), `BuildType::set_domain` (combat), and
   `init_final_flags`' `FLAT` bit (orders, which silently disabled the whole
   farm path). Whatever `PathFinderData`'s per-search fields are set from,
   find the setter.
2. **Grep the writers of every field you call frozen, and the callers of
   every function you call once-only** (the cities audit). One global reused
   by every search is exactly the shape that punishes this.
3. **When the decompiler prints a local that cannot be right, the listing
   settles it in a minute** — `llvm-objdump`, or the PE bytes. Four claims in
   the orders reading were settled that way. A 977-line function will have
   several.
4. **A behavioural check is cheap** (`docs/ORACLE.md`, "Running a check").
   And for this mechanic the check is cheaper than usual, because §2's diff
   is already built and needs no staging.
5. `tools/ghidra/README.md` has the rest.

## 8. When it is written

The definition of done is the working agreement's five, plus one specific to
this mechanic: **turn `OrderMismatch::PathLength` and `PathTo` into scoring
disagreements** in `crates/rondata/src/diff.rs`, and quote the frame count
they survive on `gamelog-run6`. That number is the mechanic's grade, and it
is the first one the project has been able to state in advance.
