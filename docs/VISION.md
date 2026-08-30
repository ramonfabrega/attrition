# Vision — what a unit reveals as it moves

`Object::update_seen@00651b80`, `World::set_seen@006b3c60`,
`Unit::update_los@0060e4d0`, and the two callers that decide when they run.

**How this was established.** Entirely from the decompile and the listing —
`tools/ghidra/decomp` for the bodies, `llvm-objdump` over
`00651b80`–`00651e00` and `00681920`–`00681b1e` for the two things the
decompiler dropped (the radius division and `project`'s register
arguments), and the PE's own `.rdata` bytes for `ring_init`'s offset
tables. Two field names come from the type record rather than from the
surrounding code: `ObjectTypeData +0x21c los` / `+0x220 science_los`, and
`LeaderData +0x6eb8 data_encrypted` → `LeaderDataEncrypt +0xe8 epoch[4]`,
which is what settles the `^ 0x87` byte read at `+0xf4` as
`epoch[Line::Science]`.

**Confidence.** High on the arithmetic — the radius, the projection, the
index ranges and the two grids are byte-verified. Lower on which of
`update_los`'s dozen additive terms can fire, because only the first few
are reachable in any capture on disk; §2 lists every one with its
constant so a later reader does not have to re-derive them. §7 is what is
not established.

## 1. Shape of the mechanic

Three grids, one bit per player, two fog cells to a world cell each way
(`fog_xs = 2 × xs`), and the numbers are in **half-cells** of `0x180`
position units throughout:

| grid | `World` | dumped as | cleared | what it means |
|---|---|---|---|---|
| `seen` | `+0x15c` | `seen[scan]` | `World::clear_seen`, from `update_all_seen` | currently in someone's line of sight |
| `seen2` | `+0x160` | `seen2[scan]` | `World::clear_seen2`, never in a game | **ever** seen — what `WorldData::was_seen` and `was_really_seen` read |
| `seen3` | `+0x164` | `seen3[scan]` | `memset` in `update_all_seen` | the inner disc, only for an object with `SubObjectData.flags & 0x40` |

The three names are settled by the three `clear_*` functions, each of
which clears exactly one plane.

`seen2` is monotone: nothing in a running game clears it. That is why the
simulation could get as far as it did on a frame-0 snapshot, and why
closing the gap is a matter of adding writes rather than of modelling
decay.

Two more planes are written by the same call and are **not** modelled
here, because nothing in this simulation reads them: `World +0x168`, the
cell-resolution twin of `seen` (cleared with it), and `WData +0x14`, the
per-cell byte the `WORLD` dump prints as `was_seen`.

## 2. The line of sight — `Unit::update_los@0060e4d0`

Writes `ObjectData::mylos` (`+0x3c`) and returns it. `UnitData::los`
(vtable `+0x128`) reads it back and adds two wonder terms; that is the
value `update_seen` starts from.

```
if !(leader_flags & 1)  { mylos = 0; return 0 }     // an inactive leader sees nothing
mylos = type->los                                   // <LOS>, in TCoords (tiles)
if mylos == 0 { return 0 }                          // and a type with no LOS never reveals
```

then, in order:

| # | gate | term |
|---|---|---|
| 1 | leader has furs (`+0x6da5 & 0x80` or `+0x6dcd & 0x80`) | `+ constants.furs_los` |
| 2 | leader has dogs (`+0x6da8 & 1` or `+0x6dd0 & 1`) **and** `is(0x45)` — the Scout lineage | `+ constants.dogs_scout_los` |
| 3 | `TypeIndex` is `0x32`/`0x33` — Citizen or its Korean twin — and `leader +0x6c20 & 4` / `& 8` / `& 0x10`, the Militia, Minuteman and Partisan upgrades (`Object::update_hits@00647010` selects `TypeIndex` `0x42`/`0x43`/`0x44` off the same three bits, which are exactly those three units) | `+2` each |
| 3b | the same two `TypeIndex`es, and `GameInfo.starting_town == 0` — a nomad start | `+2` |
| 4 | always | `+ epoch[Science] × type->science_los` |
| 5 | `unit_flags2 & 4` (packs) and (`unit_masks & 0x80000` or `is_packing()`) | `mylos = min(mylos, 4)` |
| 5b | otherwise, `TypeIndex` `0x3d`/`0x3e`/`0x190` — Merchant, Armed Merchant, Fur Trapper | `mylos = epoch[Science] + 4`, replacing everything above |
| 6 | `type->where_` is `0x1ab`/`0x1ac`/`0x1ad` | `+ get_troops_los_upgrade() × constants.troops_upgrade_los`, and then, for a type with `objmask 0x4000` or `0x400` whose leader has obsidian, `+ constants.obsidian_archers_range` |
| 7 | tribe bonus 9 (Spanish) and `is(0x45)` | `+ constants.spanish_los` |
| 8 | tribe bonus 0x12 (Iroquois) and (`TypeIndex` `0x32`/`0x33`, or not `is(0x45)` and `where_ != 0x1ab`) | `+ constants.iroquois_los` |
| 9 | `is_siege()` (type vtable `+0x10c`) | `+ turk_siege_los` (tribe 8), `+ liberty_siege_range`, `+ eiffel_siege_range` (wonders `0x219`, `0x21c`) |
| 10 | `is(0x3a)` — the Spy lineage | `+ get_spy_upgrade() × constants.spy_upgrade_los` |
| 11 | `is_hero()` | `+ get_general_upgrade() × constants.general_upgrade_los` |
| 12 | `unit_masks & 1` | `mylos = 1`, replacing everything above |

The `TypeIndex`es above are the loader's own (`docs/DATALAYER.md`: unit
record *i* is `TypeIndex 0x32 + i`), so `0x32`/`0x33` are Citizen and
Korean Citizen, `0x3a` Spy, `0x3d`/`0x3e` Merchant and Armed Merchant,
`0x45` Scout, `0x190` Fur Trapper.

**What this simulation computes** is 1–5b and 12, and of those only the
type's own `los`, the citizen terms and the science term can be nonzero in
any capture on disk: no run has furs, dogs, a militia upgrade, a nomad
start, a packed siege engine or a merchant. Terms 6–11 are read and not
implemented; each is an addition to a value whose consumer divides it by
two, so a term is worth a *fog* cell only when it reaches 2.

`epoch[Science]` is `LeaderDataEncrypt::epoch[3]` — the type record makes
`+0xe8 int[4] epoch` and `+0xf4` its fourth entry, and the `^ 0x87` in the
decompile is the low byte of the `^ 0x63187` the sibling `int` reads use.

## 3. The radius, and where it is centred

`Object::update_seen(this, ring)`:

```
los = UnitData::los()               // vtable +0x128 — §2 plus two wonders
if los == 0: return
r = (los * 0xc0) / 0x180            // tiles → half-cells: los / 2, toward zero
if r > 0x40: r = 0x40
```

`0xc0` is a tile and `0x180` a half-cell, so **`LOS` in the rules file is
in tiles and the fog radius is half of it**. The listing at `651c0d`–
`651c2f` is `lea (%eax,%eax,2); shl $6` — `los × 3 × 64` — then the signed
divide-by-384 magic sequence and a `cmovg` against `0x40`. A Citizen's
`<LOS>2</LOS>` is a radius of **1**; a Scout's `4` is **2**; Hoplites and
the Explorer at `6` are **3**; Slingers and the General at `8` are **4**.

The centre is the unit's own half-cell, `div_3_table[p >> 7]` on each
axis — except for a small, ordinary, land unit, which sees from **one
half-cell in front of its own nose**:

```
if r < 4 and is_unit() and type->domain == Land
                      and !(unit_flags2 & 4) and !(unit_masks & 1):
    (px, py) = project(x, y, dist = 0x180, angle = unit->angle)
    centre = (div3[px >> 7], div3[py >> 7])
else:
    centre = (div3[x >> 7], div3[y >> 7])
```

`project@0092cf40` is `out_x = x + sin(angle)·dist`, `out_y = y −
cos(angle)·dist` — the same pair `Unit::move_step` walks with. The
decompiler dropped both register arguments; the listing at `651cf1`
(`mov 0x50(%ecx), %ecx`, the unit's `angle`) and `651d00`
(`mov $0x180, %edx`) is what fixes them. So a citizen with radius 1 is
lighting a disc centred a half-cell ahead of itself, which is most of what
a citizen's vision *is*.

**`unit->angle` is `UnitData +0x50`, and it is not the guy's facing.**
The distinction is the whole of item 79 (2026-08-30), and this section
carried it correctly for two days while `vision.rs` projected along the
other one. `Unit::set_angle` writes `UnitData::angle` (`+0x50`) *and*
`GuyData::des_angle` (`+0x64`) together, with the bearing to the
destination, on every frame of a move; the body's own `GuyData::angle`
(`+0x18`) then turns toward it at the type's rate. So while a unit turns
the two differ — on run33's frame 168 the AI scout's dumped `UNITDATA
angle` is −51.6° and its guy's is −83.0°, thirty-one degrees apart — and
the projected half-cell differs with them. The dump prints both: a
`UNITDATA` record's `angle` is the heading, and each `GUYS` sub-record's
`angle` is the facing.

What it cost: the disc thrown along the facing lit fog cell `(113, 57)`,
which the original's never reached, and three hundred frames later
`Unit::think_scout`'s cell filter (`docs/SCOUT.md` §7) refused cell
`(56, 28)` as already-seen and spent thirty draws where the original
spends thirty-one. run33's word went 482 → **571** when it was fixed.

## 4. The two index ranges — full disc and thickened ring

The offsets come from `circle_x`/`circle_y`/`circle_radius`
(`docs/SCOUT.md` §4) or from `ring_x`/`ring_y`/`ring_radius`, and which
one, and which slice, is settled by the `ring` argument:

| case | table | `[start, end)` |
|---|---|---|
| `ring == 0` (any `r`) | circle | `[0, circle_radius[r])` — the whole disc |
| `ring != 0`, `r < 4`, projected centre | circle | `[circle_radius[max(r−5, 0)], circle_radius[r])` — and `r ≤ 3`, so the start is `circle_radius[0] = 1`: **the whole disc bar its centre** |
| `ring != 0`, `r < 4`, own centre | circle | `[circle_radius[r−1], circle_radius[r])` when `r ≥ 2`, else `[0, …)` |
| `ring != 0`, `r ≥ 4` | ring | `r` is first clamped to `0x20`; `[ring_radius[max(r−1, 0)], ring_radius[r])` |

Two consequences worth stating, because they are what make a partial
implementation defensible: for every unit whose `<LOS>` is 7 tiles or
less — which in the Ancient age is *every* unit a game starts with — the
moving case is the full disc bar one point, and the ring table is never
touched.

The `seen3` cutoff is a third index into the same range: `0` unless
`SubObjectData.flags & 0x40`, and otherwise `ring_radius[min(r, 0x1f)]`
or `circle_radius[r]` — every point below it also sets `seen3`. Note the
`0x1f` clamp writes back into `r` and therefore into the range's own end;
it is unreachable, since `r ≥ 0x20` needs `<LOS>` of 64 tiles and the
largest in the file is far below that.

**`ring_init@00681920` builds the thickened ring**, and the thickening is
the point: an octagonal ring `r` has diagonal gaps a unit stepping one
half-cell would fall through, so each ring is copied out of the circle
table and then patched.

```
ring_x[0] = ring_y[0] = 0; ring_radius[0] = 1; n = 1
for r in 1..=0x20:
    copy circle[circle_radius[r-1] .. circle_radius[r]) to ring[n ..]; n += that many
    start = ring_radius[r-1]; ring_radius[r] = n; end = n
    for i in start..end:
        (x, y) = ring[i]
        cnt = #{ j in start..n : |y − ring_y[j]| + |x − ring_x[j]| == 1 }
        if x == 0 or y == 0: cnt += 1
        if cnt >= 2: continue
        for (ox, oy) in [(0,−1), (1,0), (0,1), (−1,0)]:
            if vector_dist(|x + ox|, |y + oy|) < r:
                ring[n] = (x + ox, y + oy); n += 1
        ring_radius[r] = n
```

The inner count walks `start..n` with the **live** `n`, so a point added
this pass is counted for the next `i`; the outer bound is the `n` captured
before the pass (`681ae8`/`681b02` in the listing). The four offsets are
the PE's own bytes at `.rdata` `00add254` (x: `0, 1, 0, −1`) and
`00add214` (y: `−1, 0, 1, 0`) — north, east, south, west. `vector_dist` is
`docs/SCOUT.md` §4's octagonal distance, inlined again at
`681a33`–`681aac`.

## 5. The write — `World::set_seen@006b3c60`

For each offset `i` in the range, at `(X, Y) = (tab_x[i] + cx, tab_y[i] +
cy)`:

```
if whole_disc_on_grid or (0 <= X < fog_xs and 0 <= Y < fog_ys):
    if set_seen(X, Y, who, i < seen3_cutoff):
        reveal_fog(X, Y, who, o)
    if this->infiltrated != 0:
        set_seen2(X, Y, infiltrated, 0)
```

`whole_disc_on_grid` is one test the loop hoists: `(cx ± r, cy ± r)` all
inside, computed before the loop and used to skip the per-point bounds
check.

`set_seen(fx, fy, who, also_seen3)` ors `1 << who` (or `0xff` for
`who < 0`) into `seen`, `seen2`, `World +0x168[cell]` and `WData[cell]
+0x14`, plus `seen3` when asked, and **returns whether `seen2` changed** —
that is, whether this is the first time that player has seen the cell.
`set_seen2@006b4bb0` is the infiltration twin: it takes a mask rather than
a player, sets `seen2` always and `seen` only when its last argument is
zero.

`World::reveal_fog@006b3d30` is the newly-revealed hook: rare resources
(`Leader::new_rare`, the "found a rare resource" message), the goody-box
claim, and `Unit::get_goody_box` for a unit carrying `unit_masks & 0x100`.
None of it draws on the sync stream. `Unit::explore_goody`, reached from
`set_new_location` on a cell whose `WData` first `short` is negative,
**does** — one draw — and is a separate mechanic (§7).

## 6. When it runs

Two callers, and between them they are the whole cadence.

**Every step that crosses a half-cell.** `Unit::set_new_location@005f8d20`
compares the old and new **tile** (`div_3_table[p >> 6]`), and inside that
the old and new **half-cell** (`div_3_table[p >> 7]`); a change in the
latter calls `update_seen(param_3 == 0)`. Since a half-cell is two tiles,
the outer test is subsumed by the inner one. `Unit::move_step` passes
`param_3 = 0`, so ordinary movement is always the **ring** case;
`Unit::init`, `Unit::work` and `find_path`'s pull-back pass `1`, so a unit
arriving on the map gets the **whole disc**.

**Every hundredth frame.** `GameDaemon::process_all@00732700` calls
`update_all_seen` when `frame % 100 == 0x21` — frames 33, 133, 233 — and
it is the fourth thing that function does, after `process_victory` and
`calc_danger` and **before** `calc_markets`. It clears `seen` and `seen3`
whole, then walks every active leader's objects: `update_seen(0)` for
anything that passes the object's own two vtable predicates, and
`update_local_seen` for a started building. Because `seen2` is monotone
this pass adds nothing to it that the incremental writes have not already
added; what it rebuilds is `seen`, and the consequence worth writing down
is that **current visibility is only fully recomputed every hundred
frames** — between resyncs `seen` only ever grows.

`Unit::update_local_seen@0060e410` is the second, smaller reveal: an
object with `ObjectData::visible != 0` lights `circle_radius[type->x_size]`
points around its own half-cell into `seen2` and `seen` with the
`visible` byte as the **mask**. It is not modelled; nothing in this
simulation sets `visible`.

## 7. What is not established

- **`unit_masks & 1`.** Term 12 clamps `mylos` to 1 and the projection
  branch refuses it. Its writers are not in the decompile's `Unit` bodies
  under a form `grep` finds; its readers (`do_build` returning early,
  `close`, `set_new_location` skipping `explore_goody`, `plunder`,
  `do_repair`) read as "this object is a placement ghost, not a real
  unit". No capture sets it. The simulation has no such flag and treats
  it as clear.
- **`SubObjectData.flags & 0x40`**, the whole of `seen3`. Unnamed here,
  unset in the simulation, and the plane it gates is not modelled.
- **`update_los` terms 6–11.** Read, tabulated in §2, not implemented; no
  run reaches one. Each would have to reach 2 to move a fog cell.
- **`Unit::update_local_seen`**, and with it `ObjectData::visible` and
  `type->x_size` as a radius. Not modelled.
- **The two per-cell planes** `World +0x168` and `WData +0x14`. Written by
  `set_seen`, read by nothing here. `WData +0x14` is the `WORLD` dump's
  `was_seen`, so a future widening of that record would need it.
- **`Unit::explore_goody@005f9780`'s one draw.** `set_new_location` calls
  it whenever a unit newly enters a cell whose `WData` first `short` is
  negative — a goody. That is one sync draw this simulation does not
  spend, on any map with goodies. It belongs to a goodies mechanic, not
  to this one, but it shares this mechanic's trigger.
- **`mylos` is a cached value, and this simulation computes it fresh.**
  `Leader::calc_unit_stats@006cf970` walks a player's units calling
  `update_los`, and `Leader::process` calls *it* only when `leader_flags &
  0x4000000` is set — the twin of the `0x8000000` this simulation already
  models as `wall_stats_dirty`. So the original's `mylos` is refreshed on
  the frame **after** its inputs change. §8's diff sees exactly that, once,
  and nothing else.

  The sites, corrected 2026-08-28 by a `grep` of the vtable call rather
  than of the name. `Unit::update_los` is slot `+0x160`, and the whole set
  of callers is `Unit::init`, `Unit::set_type`, `Leader::calc_unit_stats`,
  `Leader::calc_wall_stats`, `Build::activate`, `Cities::capture_city`,
  `Wall::swap_team` and three `SpellType::cast_*`. ~~the AI's
  `Leader::check_explore` and `plan_strategy`~~ — **neither calls it**;
  both only read `World +0x160`, a field at the same offset, which is what
  a text search finds. The dirty bit's writers are `Build::activate` and
  `Build::close`, which raise it on the building's owner, and
  `Leader::gain_tech`, which raises it only for a gained type that
  `is_unit_type` **and** `is(MILITIA, 0)` — `TypeIndex 0x42`, pushed at
  `6dd98a` in the listing, the militia line term 3 adds `+2` for.
  `set_age` and `set_epoch` call `calc_unit_stats` outright rather than
  through the bit, and in a normal game neither runs: `ConsoleWin::run_cmd`
  and the scenario functions are their only callers.

  **What that does not yet explain**, and it is the open question this
  document owes: run10's one disagreement is player 1's Scout, `mylos` 4
  through frame 202 and 6 from 203, and the per-frame `LEADERDATA` (at
  `LEADERS=1`, free on every capture) shows **no** `0x4000000` on player 1
  at the end of either frame — where player 0 carries it at 202 and is
  clear at 203, which is the bit behaving exactly as read. So the refresh
  that moved that scout is not one of the sites above, or not on the frame
  they would put it. *Check:* `GUYS=2 LEADERS=9` over frames 195–210 with
  `rontrace` attached — the trace names the function that ran.
- **No dump on disk carries a *second* fog plane to diff against.** The
  `WORLD` scan — `seen[scan]`/`seen2[scan]`/`seen3[scan]`, 14,400 triples
  on a 60×60 map — is written only under `[Start Game]`, so every capture
  has the frame-0 snapshot and nothing after it. The check that would turn
  §3–§6 into a diff is `[End Frame] WORLD=6` over a short window: about
  180k lines a frame, which is affordable for ten frames the way
  `tools/gamelog/window.py` already cuts one. Until then §2 is diffed and
  §3–§6 are not.

## 8. What the simulation carries, and what checks it

`crates/sim/src/vision.rs`, and the fog planes on `crate::world::World`.

| § | what | where |
|---|---|---|
| 1 | `seen` and `seen2`, and `World::set_seen` answering "newly revealed" | `world.rs` |
| 2 | `Sim::unit_los` — terms 1–5b and 12, of which only the type's own `LOS`, the citizen terms and the science term can be nonzero here | `vision.rs` |
| 3, 4 | `Sim::seen_sweep` — the radius, the forward projection **along `UnitData::angle`**, and the four index ranges | `vision.rs` |
| 4 | `vision::ring` — `ring_init` rebuilt in integers | `vision.rs` |
| 5 | `Sim::update_seen` | `vision.rs` |
| 6 | `Sim::moved_to` at the move step and at the gather stand, `Sim::update_seen` at ejection, `Sim::update_all_seen` from `tick` | `orders.rs`, `garrison.rs`, `lib.rs` |

**The differential check is §2's, and it is the whole of run10.** Every
object record carries `ObjectData::mylos` at every detail level — which is
a thing to remember about this dump generally, and it cost one `grep`
rather than a reading. `rondata::diff` compares it on every capture and
`--diff` prints the tally;
`diff::tests::run10_s_opening_trains_the_original_s_citizens_on_its_frames`
asserts **26,433 unit-frames and exactly one disagreement**, which is §7's
cache: player 1's first science level lands and this simulation reports
`4 + 1 × 2 = 6` for its Scout on the frame the original still reports 4.
That pair is what proves `epoch[3]` is the Science line and `science_los`
its multiplier — the type record named the field, the run confirmed it.

**What the check found on its first run was not a vision defect.** 5,170
of those 26,433 unit-frames disagreed with `ours 0`, because a unit the
simulation *trained* had never been given its type: `Sim::advance_job`
set `kind` and left `Unit::ty` unset, so it had no `LOS`, no combat
profile, no speed of its own and no worker role. `Unit::init` sets all of
them and the harness's own loader always had; the production path now
does too. 5,170 → 1. The cost was that the AI's trained citizens
**started gathering**, so `the_original_s_own_run_is_still_matched_frame_
for_frame`'s path ceiling rose from 892 to 1,602 — a unit that idles
disagrees once a frame and a unit that works disagrees in detail — while
every traced check held to the number.

**The second differential check is §5's, and it is the grid itself**
(2026-08-30, item 79). `WData::log_data`'s `WORLD` block prints the whole
of `seen2` — 14,400 bytes on Great Lakes, one bit a player — and for a
month nothing compared it: the grid was installed from a frame-0 dump and
then grown by this module with no oracle at all. run13 is run10's own game
with `DUMP_ALL` over frames 95–104, so it prints the grid **ten times**,
and `diff::tests::run13_s_fog_grid_is_the_original_s_on_every_cell_of_
ten_frames` walks the simulation forward with nothing installed and
compares all 144,000 cells. It is exact.

It is also **not** what caught item 79 — the projection is wrong only
while a unit turns, and in that window nothing turned far enough to move a
fog cell. The check that caught it was the scout's own ring walk three
hundred frames later, and the lesson is in `docs/audit/README.md`: a
monotone grid hides its own errors until something downstream reads a
cell, and the reading arrives wearing the downstream mechanic's name. A
radius one fog cell too large fails the new check on its first frame.

**Thirteen deliberate breakages, all red.** Ten against the unit tests
(the radius divisor, the science term, the projection and its gate, the
ring table's reach, `ring_init`'s patch condition and its strict
inequality, the half-cell trigger, the resync's frame, `set_seen`'s
return) and three against the diff (a trained unit losing its type again,
the nomad term firing on every map, the merchants' fixed radius applied
to every type). Two of the ten were caught only after the *guard* was
strengthened: `moved_to` now answers `None` when the half-cell test
skipped the reveal, because a fog-cell count cannot tell a skipped sweep
from a repeated one, and the ring case asserts the ring table's own
cumulative indices rather than only the flag.

**What it did not fix, and this is the useful part.** `docs/PATHFINDER.md`
§12 booked run10's frame-102 draw gap on stale fog. It is not: with the
fog live the row goes 24 → 22 and stays. Both sides give the AI scout
`1/0` the same `EXPLORE_TO` and walk it to the same place; on frame 62 the
original stands still for one frame and turns, stepping a constant
`(−19, +29)` from 63 onward, while this simulation takes one more step,
**stands for seven frames**, and eases into the heading over eight more.
Ten frames behind, it arrives at 101 where the original arrived at 95 and
spends its `think_scout` ring draws a frame later. That is
`docs/MOVEMENT.md`'s stopped-unit instant turn, and the queue books it
there.
