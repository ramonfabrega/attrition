# Vision — what a unit reveals as it moves, and what a building reveals
when it is finished

`Object::update_seen@00651b80`, `World::set_seen@006b3c60`,
`Unit::update_los@0060e4d0`, `Wall::update_los@0063eeb0`, and the three
callers that decide when they run.

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

**What this simulation computes** is 1–5b and 12, and of those the type's
own `los`, the citizen terms, the science term, **term 5** and **term 12**
can be nonzero in a capture on disk. ~~Terms 3's three bits and 12 were
claimed here and not in the code~~ until item 1111 built them
(`docs/GOLDEN.md` §48): run422's Citizens see 4 from the block after `tech
who=0 militia on`, and its decoys 1, both diff-backed by the chapter's
widening. No run has furs, dogs, a nomad start or a merchant, but run58 has two packed Fishermen and term 5
is where their `mylos 4` comes from — and `4 → 6` on the frame `1/14`
deploys, which is the clamp lifting (`docs/ORDERS.md` §6.9). It was a
seam here until 2026-09-01, written as the type test alone because this
crate kept no packing state; the state is `combat.packed` now and
`is_packing` is the current order. Terms 6–11 are read and not
implemented; each is an addition to a value whose consumer divides it by
two, so a term is worth a *fog* cell only when it reaches 2.

~~Terms 6–11 are read and not implemented~~ — **term 6 is built** (item
1354): `LeaderData::get_troops_los_upgrade@006e1110` counts every one of
`TROOPS_LOS_1..3` held (bonus rows 61–63, `0x2e9..0x2eb`: Herbal Lore,
Medicine and Pharmaceuticals), and a type whose trainer — `UnitTypeData
+0x40`, this crate's `where_` — is exactly the Barracks, the Stable or the
Auto Plant takes `TROOPS_UPGRADE_LOS` (2) per level, after 5's clamp and
5b's replacement (`Sim::troops_los_level`, `Sim::trainer_ident`). The
listing's `BUY_SELL` arm compares a type the loop never reaches and is
dead. Its archers' sub-arm (objmask `0x4000`/`0x400` and Obsidian, `rare`
bit 26) adds `OBSIDIAN_ARCHERS_RANGE`, which ships as 0, and is not
carried. **Diff-backed** on run529: who=1 takes Herbal Lore on 9782, and
from block 9784 every one of its Barracks and Stable units — the Explorer
`1/0` 12 → 14, Elite Longbowmen 11 → 13, Elite Javelineers and
Cataphracts 8 → 10, Horse Archers 9 → 11 — sees two tiles more, while its
Citizens, Caravans, Merchants, Senator and Trebuchet do not. Terms 7–11
stay read and not implemented.

**`mylos` is a cache** (item 1354). `update_los` *writes*
`ObjectData::mylos` (`+0x3c`), and `UnitData::los@006100c0` — what the fog
sweep and `do_follow`'s spacing read — reads it back. The writers are the
vtable `+0x160` calls on a unit: `Unit::init` (after the packed bit, before
`add_to_world`'s disc), `Unit::set_type@00612fa0:235`,
`Leader::calc_unit_stats@006cf970` over every live unit of the player,
`SpellType::cast_unpack`, `cast_pack` (which nothing here casts) and
`cast_create_decoy` (after `unit_masks |= 1`). This crate keeps the cache as
`Unit::mylos` and writes it with `Sim::update_los` at exactly those sites;
`Sim::unit_los` is the derivation. **`calc_unit_stats` runs at
`Leader::process` on `leader_flags & 0x4000000`**, whose writers are
`Leader::gather` on a change of the rare mask, **`Leader::gain_tech`'s tail
— `|= 0xc000000`, both stats flags, on every tech that reaches it,
Herbal Lore among them** — beside its militia
arm, and the **wonder** arms of `Build::activate@00623e20:1970` (`TypeIndex
0x20e..0x21e`, every other building jumping past to `00626f11`) and
`Build::close@00628980:319`. So a gained tech reaches the fog at the next
leader pass: run529's 29 rows of `mylos` part on 9783 with the term
computed live and agree with the cache.

`epoch[Science]` is `LeaderDataEncrypt::epoch[3]` — the type record makes
`+0xe8 int[4] epoch` and `+0xf4` its fourth entry, and the `^ 0x87` in the
decompile is the low byte of the `^ 0x63187` the sibling `int` reads use.

## 2.1 A building's line of sight — `Wall::update_los@0063eeb0`

`Build` and `Wall` share vtable slot `+0x160`, and the function there is
**not** §2's: it is a shorter one with a tail §2 has no counterpart to.

```
if !is_started                  { mylos = 0; return 0 }
if !is_active && !vfunc(+0x2c)  { mylos = 0; return 0 }
if !is_active                   { mylos = 1; goto tail }
if !(leader_flags & 1) { mylos = 0; c = 0 } else { c = type->los; mylos = c }
mylos = epoch[Science] * type->science_los + c
if is_fort()        mylos += constants.fort_upgrade_range[get_fort_los() + 4]
else if is(0x1b7)   mylos += constants.tower_fort_range[get_tower_fort_los() + 3]
if (fort or tower) and has_wonder(0x212)  mylos += constants.colosseum_fort_range
if leader has furs                        mylos += constants.furs_los
tail: mylos += type->x_size / 2
```

**The tail is the half nobody would guess**, and the dump is what settles
it. run39's AI opens with a Small City at `mylos 15` (`LOS 12`,
`X_SIZE 7`), a Woodcutter's Camp at **7** (`6`, `2`) and four Farms at
**8** (`6`, `4`); every one of them is `LOS + X_SIZE / 2`. The city then
goes to **17** on sim-frame 202 — the frame its Science epoch reaches 1
— which is the science term with the Small City's `SCIENCE_LOS 2`.

Three consequences worth stating:

- An **unfinished** building sees `1 + x_size / 2`, never its type's
  `LOS`; an unstarted one sees nothing. run39's sixth farm is dumped at
  `mylos 0` from the frame it is placed (frame 2, `flags 1`) through the
  frame it is started (70, `flags 3`) — because nothing *calls*
  `update_los` in between — and at **8** from the frame it activates.
- The fort, tower, colosseum and furs terms are read and not carried; none
  can fire in any capture on disk. Every one of them is an addition to a
  value §3 halves, so a term is worth a fog cell only when it reaches 2.
- `type->x_size` is the footprint in **tiles**, the same field the block
  mask is sized by, and the division is C's — toward zero.

The caller is what makes this matter: **`Build::activate@00623e20`'s last
statement is `update_seen(0)`**, the whole disc, at vtable `+0x174`, after
`update_hits` (`+0x15c`) and `update_los` (`+0x160`) a few lines above it.
So a building lights its disc on the frame it is finished, and
`update_all_seen` (§6) throws it again every hundredth frame.

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
arriving on the map gets the **whole disc** — and so does
`SpellType::cast_transport`'s move of a newborn barge onto the water (§11).

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

**Every building that finishes.** `Build::activate@00623e20` ends with
`update_seen(0)` — the whole disc at §2.1's radius, from the building's
own half-cell, since a building is not `is_unit()` and §3's projection is
an `is_unit()` case. This is the third caller, and it is the one this
simulation lacked until 2026-08-31: the fog grew only where units walked,
so a farm finished mid-game revealed nothing at all. What that cost is in
§8.

`Unit::update_local_seen@0060e410` is the unit's half of §6.1's second
reveal. An object with `ObjectData::visible != 0` lights
`circle_radius[type->x_size]` points around its own half-cell into `seen2`
and `seen` with the `visible` byte as the **mask** — the rule that **a
unit which attacks you becomes visible to you**. ~~It is not modelled;
nothing in this simulation sets `visible`~~, and ~~`visible`'s writers in
the original are `Unit::set_attacking@005ff5b0` and
`Unit::do_cast@005ebfe0` … and no building path reaches it~~ — both
**answered 2026-09-21 by item 457, §9**, which models the field and its
clear, and which found five writers rather than two: `Build::do_attack`
and `Build::do_missile_launch` reach it, so a **tower that shoots you
becomes visible to you** as well (§9.1).

## 6.1 The second reveal: a building the enemy has laid eyes on

The building's half of it is a different function and a different trigger,
and it is not gated on `visible` at all. It is
`Wall::update_local_seen@0063ed50`, and what drives it is
`Wall::check_ever_seen@0063ce70`.

**`Wall +0x62 ever_seen` and `+0x63 ever_seen_completed`** are one bit a
player: who has ever had this building's footprint in **current** line of
sight, and who has had it there while the building was finished. The names
are the dump's own — `BUILDDATA` prints both from `BUILDS=1` — so nothing
here is inferred from surrounding code.

**`check_ever_seen(param_1)` grows them.** It scans the footprint, tile by
tile, at `(corner + i) >> 1`, reading `World +0x15c` — `seen`, the
*current* plane, not `seen2` — and ors what it finds in. While the
building is unstarted the owner's own ally mask (`LeaderData +0x6929`)
filters it; once started every bit is taken, and `ever_seen_completed`
takes them too for an active one. The scan is skipped entirely once
`ever_seen` and `ever_seen_completed` already hold every bit the game has
(`Game::everyone_mask` when started, the owner's ally mask when not).

Its tail is the meet loop: for every other active leader whose mask has
**newly** appeared in `ever_seen`, `Leader::meet` fires — the diplomatic
first contact — and then, once, `update_local_seen` at vtable `+0x164`.

**`Wall::update_local_seen` is the write.** Two branches:

| when | mask | `set_seen2`'s `param_4` |
|---|---|---|
| `is_wonder()` **and** not `flags & 0x20` **and** `type+0xfc()==0` **and** `is_started()` | `0xff` — every player at once | 0, so `seen` and `World +0x168` too |
| otherwise | `ever_seen \| visible \| (1 << who)` | 1, so **`seen2` and `WData +0x14` only** |

The rectangle is the footprint **grown by one tile on every side** —
`i` from `−1` to `x_size` inclusive, `j` likewise, `(x_size + 2) ×
(y_size + 2)` tiles — each mapped to its half-cell by `>> 1`.

That `param_4` is load-bearing: the ordinary branch leaves the *current*
line of sight alone, which is what stops one building's reveal from
convincing the building next door that it has been spotted. A model that
folds the two planes into one gets a chain reaction across the whole base.

**The cadence is `Wall::process@00640450`'s first statement**, and it is
the game's own frame rather than the object's phase:

```text
if (frame != 0 && (frame & 7) == who) { targeted = targeted * 3 / 4; check_ever_seen(0); }
```

— every eighth frame, on the one whose low three bits are the owner's
player number. `Wall::start@0063e810` and `Wall::init@0063e9b0` each call
it once more with `param_1 = 0`, and `Wall::process`'s enemy-adjacent arm
twice with `param_1 = 1`, after putting a bit in by hand.

**What it is worth.** This is Great Lakes' 8031. At block 8002 the
original's fog plane carried player 1's bit on fourteen half-cells this
crate had dark — `(10, 73)`, `(11, 73)`, and the block `x` 6–8, `y` 78–82
— and the AI scout therefore priced the step into the human capital's cell
`(3, 39)` at 328 where this crate priced it 9, and walked *through* the
footprint instead of round it (`docs/PATHFINDER.md` §20). The two patches
are the two buildings of player 0's seven whose `ever_seen` reads **3** at
that block: the Small City `0/2000` at half-cell `(8, 80)` and `0/2001` at
`(11, 74)`. For the Small City — 7 × 7, corner tile `(13, 157)` — the
grown rectangle is tiles 12–20 × 156–164, which is exactly half-cells
6–10 × 78–82; the columns at `x` 9 and 10 and the three cells `(8, 78..80)`
were already lit by the scout's own disc, and the twelve that were not are
the twelve that parted.

With it, that plane is **14,400 of 14,400** at block 8002 and `ever_seen`
agrees on all 28 buildings, and this map's word went **8031 → 8186**.

**Seams**, all stated in the code:

- `ObjectData::visible` is still 0 here, so the mask's middle term is
  always zero.
- `Leader::meet` itself is not modelled; only its flag, which is the gate.
- ~~`update_all_seen` still does not clear `seen`, so this crate's `seen` is
  monotone where the original's is rebuilt every hundredth frame. The two
  can only differ over the ≤ 8 frames between a sighting and the owner's
  next check — the bit is taken on the first check either way.~~ **Cleared
  since item 709, §10**: the fog test in `valid_target` reads `seen` too,
  and there the difference is not eight frames but a hundred.
- ~~`Leader::meet` itself is not modelled; only its flag, which is the
  gate.~~ **Closed by item 385** — §6.2 below.
- ~~`Wall::start`'s own direct `seen2` write over its footprint (the owner's
  bit) is not made; it is the owner's own bit over ground the owner's own
  line of sight covers.~~ **Made since item 647** (`Sim::start_building`).
  The premise held only for a building placed beside its builder. The AI
  places a city out of its own sight, and on run157 its scout then priced
  that footprint as unseen ground and cut across it. The write is the
  footprint, `x_size × y_size` and not grown, at `tile >> 1`, and it
  sets `seen2` alone. `docs/SCOUT.md` §14 has the capture.

## 6.2 First contact: the met bit hangs off this mechanic (2026-09-18)

`check_ever_seen`'s tail does two things, and §6.1 above only carried one
of them. The other is **diplomacy's first contact**, and it is the single
live writer of a field four of this map's dumps print.

**`LeaderData::treaties` (`+0x94`), `int[8]`, bit 0 — the met bit.**
`Leader::treaty_on@006e1190` ors a mask into **both** leaders' slot for
the other and is the only writer; `Leader::meet@006e1250` is its only
caller and passes 1; and `meet` has exactly two callers in the whole
executable:

- **`Wall::check_ever_seen@0063ce70`** — the tail loop, for every other
  leader whose ally mask has *newly* appeared in the building's
  `ever_seen`; and
- **`Unit::process_attrition@005e11a0`**, at **three** sites, not two:
  the war arm, the assassin arm, and the generic arm under
  `get_attrition() != 0`. All three carry the same guard,
  `other < 0 || treaties[other] & 1 == 0`.

`Leader::treaty_off@006d0370` clears bit 0, and its only caller is the
debug console's `run_cmd`. **Nothing in a game ever clears the bit.**

### The loop's own gates, exactly

For each `o` in 0..8, with `old` the building's `ever_seen` on entry and
`now` its value after the footprint scan:

```text
o != owner
leaders[o].leader_flags & 1            # LEADER_VALID — a slot in use,
                                       # not the & 2 the census loops take
now & leaders[o].ally_mask != 0
old & leaders[o].ally_mask == 0        # the bit is NEW
  -> reveal = true                     # set here, before the test below
  -> if treaties[owner][o] & 1 == 0:  Leader::meet(owner, o, x, y)
if reveal: update_local_seen()         # vtable +0x164, §6.1
```

The `reveal` flag is raised **before** the already-met test, so a second
sighting still relights the footprint; only `meet` is once-only. That
ordering is carried here.

### Why it is worth a section

`Leader::plan_strategy`'s war and region census loops gate on
`treaties[i] & 1` and on nothing about humans (`docs/AI.md` §43, §45), so
this bit decides whether the AI's `active_wars` is 0 or 1 — and
`ai_research::weight_total` takes `ai[0] / 3` instead of `ai[5] + ai[1]`
when it is 1, which on Great Lakes is **144 against 110** on every tech
the AI ranks. Until item 385 this crate stood in for the bit with "skip
human leaders", which answers *never met* — and the alternative, *met from
frame 1*, cost the long capture's word 9182 → 7182.

**What the fog answers instead is 7944**, and the original's own dumps
bracket it: player 1's `treaties[0]` is 0 on blocks 6950 (run84) and 7514
(run91) and 1 on 8174 (run19) and 9170 (run107), with `diplos[0]` at 0 —
at war — on all four. The contact is in (7616, 8174]; this crate's is
inside it. Great Lakes' word went **9182 → 9415** on it, and all four of
those windows' leader residues fell (110 → 95, 91 → 80, 92 → 91, 82 → 81)
with nothing arriving.

### What carries it

[`Sim::treaty_on`] and [`Sim::meet`] in `lib.rs`, `Sim::treaties` beside
`at_war` and `allied`, the loop in `Sim::check_ever_seen`, and
[`Sim::has_met`] reading the bit for the census and for the record
comparison. `a_building_seen_by_the_enemy_is_first_contact` and
`an_unstarted_building_cannot_introduce_two_enemies` are the unit tests;
the standing oracle is `treaties` in the leader-record comparison across
the four windows above.

**The attrition path is a stated seam.** Its three sites are read and not
wired, because instrumented over run53's 24,000 frames **no unit of
either leader reaches a non-exempt attrition outcome** (item 382), so no
capture on this disk can tell whether it is right. A map where an army
campaigns abroad would; that is when to wire it.

## 6.3 The contact frame itself — 7945, on both sides (2026-09-21)

Item 390, the capture §6.2 named as owed. §6.2 gave the met bit a writer
and Great Lakes' word moved 9182 → 9415 on it, but the *frame* had never
been checked: this crate's first contact is sim-frame 7944 — block **7945**
in the dump's own numbering, which lags the sim by one — and the original's
was bracketed only to **(7616, 8174]**, 558 frames wide, by four windows of
the same game with none of them inside the gap.

**run115 closes it to a block, and the block is the same one.** 130 blocks
of `LEADERS=9` over `[7880, 8010)` — run94's window narrowed, at run94's
detail but for that one category — and the comparison reads the flip off
the record rather than off a grep:

| | first block carrying the bit |
|---|---|
| this crate, `1/treaties[0]` | **7945** |
| the original, `1/treaties[0]` | **7945** |
| the original, `0/treaties[1]` — the mirror | **7945** |

So `Leader::treaty_on@006e1190` ors into both leaders' slots on the same
frame, as §6.2 read it, and the fog path reaches it on the original's own
block. Nothing else moves: `diplos` is 2 on each leader's own slot and 0 on
the cross slot on all 130 blocks of both leaders, so the diplomacy is not
renegotiated anywhere near contact — the two are at war from before the
window and only *contact* happens inside it. The bit does not fall again
through block 8009, which is what `Leader::treaty_off@006d0370` having no
in-game caller predicts.

The four slots above 1 stay 0 on every block of both leaders, and the dump
carries two more `LEADERDATA` records — leader slots 8 and 9 — that are
zero throughout, which is `Sim::check_ever_seen`'s `leader_flags & 1` gate
having nothing to do there.

### What carries it

`run115_s_window_is_the_met_bit_s_own_frame` (`crates/rondata/src/diff/
leader.rs`): 260 blocks, 272,480 field-frames, the flip triple pinned at
`(7945, 7945, 7945)`, `diplos` asserted not to move, and the 89-field
residue pinned by name as `PARTS_ON_RUN115`. Made to fail on purpose —
commenting out the `Sim::meet` call in `Sim::check_ever_seen`'s tail, which
is item 385's whole change, reads `(None, 7945, 7945)`.

`docs/RUNS.md`, run115, is the capture and its five checks.

### What it opened, and it is not this mechanic's

`0/wars`, `0/active_wars` and `0/active_wars_with` agree at nought for 121
blocks and then part on **8001**, where the original writes the *human*
leader a war census — `wars 1`, `active_wars 1`, `active_wars_with 2` —
beside a single `production_step` tick that falls back to 0 on 8002. That
is 56 blocks after first contact, and it is the first direct evidence that
the original runs its census for a human leader at all. This crate leaves
the human's census at zero forever. It belongs to `docs/AI.md` §43's
`human` skip, not here.

### What is not established

- **Whether the two sides agree on contact for the same *reason*.** The
  block matches and the mirror matches; the capture does not print which
  building's `ever_seen` newly acquired which ally mask, so a different
  sighting that happens to land on the same frame is not excluded. The
  `DUMP_ALL` window that would settle it costs ~61 MB a frame and has not
  been booked.
- **Contact on any other map.** 7945 is Great Lakes' and one game's. East
  Indies has no `LEADERS≥3` window near its own contact, and no capture on
  this disk dates it.
- **The attrition path to `meet`**, unchanged from §6.2: three sites in
  `Unit::process_attrition@005e11a0`, read and not wired, because no unit
  of either leader reaches a non-exempt attrition outcome in run53's 24,000
  frames.
- **`treaties`' other bits.** Only bit 0 is modelled and only bit 0 is
  compared; the record's values are 0 or 1 on every block of this window,
  so nothing here says what the higher bits would carry.

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
- ~~**`update_los` terms 6–11.** Read, tabulated in §2, not implemented; no
  run reaches one.~~ Term 6 is built and diff-backed on run529 (§2); terms
  7–11 are read and not implemented, and no run reaches one. Each would
  have to reach 2 to move a fog cell.
- ~~**`Unit::update_local_seen`**, and with it `ObjectData::visible` and
  `type->x_size` as a radius. Not modelled.~~ **Answered 2026-09-21 by item
  457, §9** — all three, with the radius settled as the XML's
  `CIRCLE_RADIUS` (`UnitType::init@0061ab50:655`-`662`).
- **The two per-cell planes** `World +0x168` and `WData +0x14`. Written by
  `set_seen`, read by nothing here. `WData +0x14` is the `WORLD` dump's
  `was_seen`, so a future widening of that record would need it.
- ~~**`Unit::explore_goody@005f9780`'s one draw.**~~ Answered
  2026-08-31 in `docs/GOODY.md`, and it is not one draw but **one per good
  the finder can gather** — three in the Ancient age. It shares this
  mechanic's *caller* and not its trigger: the reveal hangs off the **tile**
  test (`p >> 6`) and the goody off the **cell** test (`p >> 8`), which is
  §6's outer one.
- ~~**`mylos` is a cached value, and this simulation computes it fresh.**~~
  **Modelled by item 1354 (§2)**: the cache, its six writers and the
  flag's. The reading below said `gain_tech` raises `0x4000000` only on
  its militia arm; the tail's `|= 0xc000000` (`orl $0xc000000, (%ebx)` at
  `006e0315`, decompile line 2313) raises it on every tech that
  reaches the tail, and run529's block 9784 is that. Whether any of
  `gain_tech`'s early returns skips it is not established.
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

  ~~**What that does not yet explain**~~ **Answered by item 1354 as to the
  value**: with the cache refreshed on the leader pass after a gain, run10's
  scout, run349's and run382's agree on 202, and so do chapter
  twenty-six's `0/0` on 1023 and every Citizen row of run174's, run99's,
  run416's and run471's windows (§2). What the flag
  reading below says is still unexplained: run10's one disagreement was
  player 1's Scout, `mylos` 4
  through frame 202 and 6 from 203, and the per-frame `LEADERDATA` (at
  `LEADERS=1`, free on every capture) shows **no** `0x4000000` on player 1
  at the end of either frame — where player 0 carries it at 202 and is
  clear at 203, which is the bit behaving exactly as read. So the refresh
  that moved that scout is not one of the sites above, or not on the frame
  they would put it. *Check:* `GUYS=2 LEADERS=9` over frames 195–210 with
  `rontrace` attached — the trace names the function that ran.
- ~~**No dump on disk carries a *second* fog plane to diff against.**~~
  **Three do, and no capture was needed** (item 320, 2026-09-17). The
  `WORLD` scan is written under `[Start Game]` *and* on every block a
  `DUMP_ALL` **window** covers, which is a `window:` line in
  `tools/gamelog/captures.txt` rather than a raised `[End Frame]` category.
  On Great Lakes, run13 carries the plane at blocks **95–104**, run73 at
  **5564–5580** and run93 at **7929–7936**; each block holds 14,400
  `seen2[scan]` values (printed six times over, once per `WORLD` record)
  beside every cell's `WData` and all 57,600 tile masks.
  `run13_s_fog_grid_is_the_original_s_on_every_cell_of_ten_frames`
  had the first; item 320 added
  `run93_s_block_7932_is_this_crate_s_world_cell_for_cell`, and **this
  crate's fog plane is the original's at block 7932 — 14,400 of 14,400
  half-cells, 7,932 frames of every reveal §3–§6 makes.** So §3–§6 are
  diff-backed now, out to seventy frames under this map's own word, and the
  costed capture is not owed. What is still not diffed is any frame past
  7936 and any plane but `seen2`.
- **`Unit::update_local_seen` is not a corner: it is Great Lakes' 8031**
  (item 320, 2026-09-17). The bullet below has said since 2026-08-31 that
  it is "not modelled; nothing in this simulation sets `visible`". run95
  puts a price on that. At block 8002 this crate's plane parts from the
  original's on **fourteen half-cells** — a 3 × 5 block at `x` 6–8,
  `y` 78–82, which is two `circle_radius[1]` discs centred on `(7, 79)` and
  `(7, 81)`, plus the single points `(10, 73)` and `(11, 73)` — and every
  one is **player 1's bit over ground player 0 holds**. Two of them are the
  `2c + 1` half-cells of the cells the AI scout's route reads, so the
  original prices that step **328** and this crate **9**
  (`docs/PATHFINDER.md` §20.2). A mask that is not the owner's bit is
  exactly what §6's reveal has and `Object::update_seen`'s disc does not,
  and no disc of the one AI unit in the west can make the shape. The
  reveal lands between blocks **7937 and 7998**, and
  `run95_s_block_8002_is_where_the_fog_parts_and_the_price_with_it` is the
  standing check.

  **What the successor inherits is an elimination, not a candidate.** Every
  writer of the `seen2` plane the decompile names was checked against that
  window and none of them can have run in it:

  | writer | ruled out by |
  |---|---|
  | `Object::update_seen`'s disc, via `World::set_seen` | the mask is the owner's bit, and no disc of the one AI unit in the west has the shape (§3's radius and centre, measured) |
  | `Object::update_seen`'s tail, `set_seen2` masked by `ObjectData::infiltrated` (`+0x3a`, **not** `visible`) | `infiltrated`'s only writers are `ScenarioFuncSet::add_infiltrated`/`remove_infiltrated` — script functions a normal game never calls |
  | `Unit::update_local_seen` / `Wall::update_local_seen` | reached only from `update_seen(0)` and `update_all_seen`, and **no `update_seen(0)` happens in the window**: `update_all_seen` runs at frame 7933 (block 7934, and run93's 7936 is still dark), and `Build::activate`/`Wall::activate`/`Unit::init` need a roster change that did not occur — `BUILDDATA` 56, `WALLDATA` 56, `UNITDATA` 180 and `ANIMALDATA` 80, identical at 7932 and 7999 |
  | `Ammo::do_damage`'s `set_seen2(…, 0xff, 0)` | it is the **nuke** flash — `ACHIEVEEVENT_NUKE_EXPLOSION`, `Nuke::add_nuke` — in the Ancient age |
  | `ScenarioFuncSet::set_explored` / `set_seen` | script-only |

  So the writer is **unestablished**, and the next move is an *offset*
  search — writes to `World +0x160` and to `ObjectData +0x40` — rather than
  another grep by name, which is what missed `infiltrated` being the tail's
  mask in the first place.

## 8. What the simulation carries, and what checks it

`crates/sim/src/vision.rs`, and the fog planes on `crate::world::World`.

| § | what | where |
|---|---|---|
| 1 | `seen` and `seen2`, and `World::set_seen` answering "newly revealed" | `world.rs` |
| 2 | `Sim::unit_los` — terms 1–5b and 12, of which only the type's own `LOS`, the citizen terms and the science term can be nonzero here | `vision.rs` |
| 2.1 | `Sim::build_los` — the started/active heads, the science term and the `x_size / 2` tail; the fort, tower, colosseum and furs terms are seams | `vision.rs` |
| 2.1, 5 | `Sim::build_sweep` and `Sim::update_seen_build` — the whole disc at the building's own half-cell | `vision.rs` |
| 3, 4 | `Sim::seen_sweep` — the radius, the forward projection **along `UnitData::angle`**, and the four index ranges | `vision.rs` |
| 4 | `vision::ring` — `ring_init` rebuilt in integers | `vision.rs` |
| 5 | `Sim::update_seen` | `vision.rs` |
| 6 | `Sim::moved_to` at the move step, at the gather stand and at a barge's birth (§11), `Sim::update_seen` at ejection, `Sim::update_seen_build` from `Sim::activate`, `Sim::update_all_seen` — now units **and** buildings — from `tick` | `orders.rs`, `garrison.rs`, `transport.rs`, `city.rs`, `lib.rs` |
| 6.1 | `Sim::check_ever_seen` and `Sim::update_local_seen_build` — the footprint scan, the grown rectangle, the `seen2`-only write | `vision.rs` |
| 6.2 | the meet loop, `Sim::treaty_on`, `Sim::meet`, `Sim::has_met` — first contact and the met bit | `vision.rs`, `lib.rs`, `ai_census.rs` |

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

**§6.2's check is the leader record's `treaties`**, and it is four
windows of one game rather than one window: run84's block 6950 and run91's
7514 with the bit clear, run19's 8174 and run107's 9170 with it set, every
block of each. It is the shape a one-window check cannot have — a bit that
is wrong *in time* passes any window taken on one side of the flip, and
these four bracket it from both. `diplos` is compared beside it and parts
nowhere, which is what says the thing that moves is contact and not the
diplomacy.

**The third differential check is §2.1's, and it is what the second one
could not see** (2026-08-31, item 99). run13's ten frames are frames 95 to
104 of a game in which no building finishes, so a grid that grows only
from units passed it exactly — and the defect it missed cost East Indies
a hundred and sixty-three frames of word. run39's AI finishes its sixth
farm on frame **219** at cell `(54, 51)`; at `mylos 8` its disc has radius
4 and reaches the three cells of column 56 beside it, and stops short of
`(56, 54)`, which `Unit::think_scout` still needs dark to pick as its next
target. Nineteen frames later the scout re-targets, and the `EXPLORE_TO`
path it plans is what reads the grid: a scout prices an unseen cell at a
base of **8** against a seen cell's `0x400` (`docs/PATHFINDER.md` §5), so
with those three cells dark it walked *through* them, and with them lit it
takes the seen column 55 — which is the original's own dumped stack, entry
for entry. `diff::tests::run39_s_sixth_farm_lights_the_cells_its_scout_
then_paths_around` is both halves, and
`run39_s_long_trace_says_where_the_second_map_s_word_parts` went
**413 → 576** with it.

The lesson is the twin of item 79's, and it is now in
`docs/audit/README.md`: a monotone grid hides its errors until something
reads a cell, and *"the pass cannot remove a bit, so it need not run"* —
which is what this module's own comment said about buildings — answers a
question nobody asked. Monotonicity says a second sweep cannot **unset**
anything. It says nothing about a sweep that was never thrown.

**Sixteen deliberate breakages, all red.** Ten against the unit tests
(the radius divisor, the science term, the projection and its gate, the
ring table's reach, `ring_init`'s patch condition and its strict
inequality, the half-cell trigger, the resync's frame, `set_seen`'s
return) and three against the diff (a trained unit losing its type again,
the nomad term firing on every map, the merchants' fixed radius applied
to every type), and three for §2.1 (the `x_size / 2` tail dropped, the
unfinished building given its type's `LOS`, and `update_seen_build`
dropped from `Sim::activate` — which leaves the score at 576, because the
hundred-frame pass covers this one case on frame 233, and fails the
frame-220 half of the new diff, which is the whole reason that half is
there). Two of the ten were caught only after the *guard* was
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

## 9. `ObjectData::visible` — a unit that attacks you becomes visible to you (item 457, 2026-09-21)

`ObjectData::visible` (`+0x40`, a `char`; `types.txt`) is the byte of
players an object has made itself visible to **by attacking them**. It is
the fallback arm of `UnitData::is_seen@00607a60`, so it decides who may
legally target a unit standing in their own fog — `docs/COMBAT.md` §31.1
has the chain from `Object::valid_target` down to that test, and §31.7
booked this as the half it left unmodelled.

§6's note has said since 2026-08-31 that nothing here sets it. It does now.

### 9.1 The writers, grepped rather than believed

`CLAUDE.md`'s standing rule is to grep the writers of every field you call
frozen, and §31.7's list of two was short by three. Searching the whole
decompile for writes to `+0x40` on an `Object` lineage — the decompiler
prints it as `field_0x40`, never as `visible`, which is why a search by
name finds nothing — gives **five**:

| writer | what it does |
|---|---|
| `Unit::set_attacking@005ff5b0` | `visible \|= 1 << victim_who`, from `Unit::fight`'s tail |
| `Unit::do_cast@005ebfe0:544` | the same three lines, for an offensive spell |
| `Unit::work@0060d180:63` | **the clear** — §9.3 |
| `Build::do_attack@006228f0:147` | `visible \|= 1 << target_who` after `Object::fire_ammo`, with its own clear at `:47` |
| `Build::do_missile_launch@00622670:70` | `visible = 0xff` — a missile silo is visible to **everybody** the frame it fires |
| `Object::init@00647750:25` | `visible` and `launch_frames` zeroed together as one short, at birth |

So **§6's "no building path reaches it" is withdrawn**: a tower that shoots you
becomes visible to you on exactly the rule a unit does, and a silo that
launches becomes visible to the whole map. Neither is modelled here — this
section is the unit half — and `Wall::update_local_seen`'s mask
(`ever_seen | visible | (1 << owner)`, §6.1) therefore still carries a
stubbed term.

### 9.2 `set_attacking`, exactly

```
set_attacking(this, who):                    # who = the victim's player
    attacked_table[this->who][who] |= 1      # 00e3a424, per-pair
    leaders[who].field_0x94[leaders[this->who].team] |= 1
    this->flags |= 0x80                      # SubObjectData::flags
    if (visible & (1 << who)) == 0:
        if !WorldData::is_seen(pos / 0x180, who):
            visible |= 1 << who
            this->vtable[0x164]()            # Unit::update_local_seen
        else:
            visible |= 1 << who
```

Both arms set the bit; only the first lights anything. The reveal is
`Unit::update_local_seen@0060e410` — `circle_radius[type->x_size]` points
of the `circle_x`/`circle_y` spiral around the unit's own half-cell,
written with `visible` **as the mask**, through
`World::set_seen2(…, param_4 = 0)`, which is **both** planes: `seen2` and
the current line of sight. That is unlike the building's second reveal
(§6.1), which is `seen2` only.

`Unit::fight@005fd4d0` has **two** call sites. The one at `005fe4f7` is
gated on the target's type vtable `+0x18` and on the attacker's
`ptype->vtable[0x10c]`; the main strike path at `LAB_005feec6`, reached
after `set_anim`, is ungated and is the one that runs here.

**`+0x10c` is `ObjectTypeData::is_siege`**, and §31.7's reading of it as a
gate *on the write* was wrong — it gates one call site. Settled three ways,
none of them the decompiler's guess: the PDB type record for
`ObjectTypeData` lists `is_siege` at `vftable offset = 268` between
`is_dock` (264) and `is_tank` (272); `ObjectData::is_dock@004711e0` is a
one-line thunk whose body is `ptype->vtable[0x108]()`, which pins the
origin; and `ObjectData::is_siege@0046ef90`'s body is `ptype->
vtable[0x10c]()`. `UnitTypeData::is_siege@00470460` is `unit_flags &
0x20000`. Chapter two has no siege, so only the ungated site fires here.

`do_cast`'s own `+0x10c` call is at `005ecd31`, four hundred lines above
the `visible` write and in an unrelated general/pack branch; nothing gates
the cast's write but the spell type's `+0x1c8 & 0xe`, a different-player
test, and `target->vtable[0x48]` — `is_seen` on the target.

### 9.3 The clear, and it is a 32-frame slot with a latch

`Unit::work@0060d180`, above every gate in the function — a unit that
returns early still runs it:

```
if ((frame + o) & 0x8000001f) == 0:      # (frame + o) % 32 == 0
    if (flags & 0x80) == 0:
        visible = 0
    unit_masks &= ~4
if order != ATTACK:
    flags &= 0x7f
```

So the latch is read **before** it is dropped: the frame a unit stops
attacking still counts as attacking, and the visibility outlives the last
arrow by up to 32 frames. `Build::do_attack:47` is the same pair for a
building, with `flags &= 0x7f` unconditional.

**This is diff-backed, five for five.** run112 prints `visible` and
`flags` on every unit record, and the five clears it carries each land on
the first frame `(f + o) ≡ 1 (mod 32)` — the dump's label is the sim frame
plus one — after `flags & 0x80` drops:

| unit | `0x80` drops | `visible` clears | its slot |
|---|---|---|---|
| `0/6` | 819 | **827** | `f ≡ 27 (mod 32)`: 795, 827 |
| `0/7` | 848 | **858** | `f ≡ 26`: 826, 858 |
| `0/8` | 848 | **857** | `f ≡ 25`: 825, 857 |
| `0/9` | 846 | **856** | `f ≡ 24`: 824, 856 |
| `0/10` | 830 | **855** | `f ≡ 23`: 823, 855 |

`1/7` is the control: its latch drops at 764 and comes back at 792, its
slot is 794, and the byte never clears. `0/11` dies at 729 with the latch
up and never clears either.

### 9.4 `WorldData::is_seen` has a fourth arm, and §31.2 omitted it

```
is_seen(fx, fy, who):
    if who > 7 or reveal_map == 3:                        return 1
    L = leaders[who]
    if (L.leader_flags & 0x800) or L.num_units[0x141]:    return 1
    if (L.leader_flags & 0x2000) and wdata[cell].who >= 0
            and L.is_ally(wdata[cell].who):               return 1
    return seen[fy * fog_xs + fx] & L.ally_mask
```

The third arm — **a player with `leader_flags & 0x2000` sees everything
standing on ground it or an ally owns** — is not in §31.2's transcription
and is not carried here. Like the two above it, it can only ever *refuse*
further, so the seam is one-directional. No capture on disk raises any of
the three.

### 9.5 What it moved

| counter | before | after |
|---|---|---|
| `chapter_two_s_first_attack_orders_are_the_dump_s` | **six** of nine rows | **nine of nine, the dump's own** |
| `GOLDEN_WORD_CHAPTER_TWO` | 624 | 624 — **unmoved** |
| chapter two, first value disagreement | 625 | 625 |
| `LONG_WORD_GREAT_LAKES`, `LONG_WORD_EAST_INDIES`, `GOLDEN_WORD_CHAPTER_ONE` | — | unmoved |
| both endpoints and both ladder rungs | — | unmoved |

The sub-score the item was booked on closed and the headline did not move,
and the two facts belong side by side. `1/6`–`1/8` now take their attack
orders at **635**, the frame the dump has, for the reason the dump has it:
`0/10` set its own bit for player 1 at 631 by shooting a hoplite, and
player 1's line of sight never reaches that cell. What stands at 624 is a
single extra draw on the original's side — `Guy::set_anim+0x97a <
Unit::move_step+0x823`, a unit stepping where this crate's does not — which
is the chase-*destination* residue `docs/COMBAT.md` §31.6 measured at 622
and named parked 400's shape. It is upstream of anything this field can
reach.

### 9.6 The value diff beside it

`chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame` reads
run112's own byte on every unit-frame of the chapter, both directions. The
**shape is exact** — the same nine units gain a bit, each the right
player's — and the frames are pinned in no direction beside the dump's:

| unit | dump | ours |
|---|---|---|
| `0/6`, `0/7`, `0/8` | 636 | **636** |
| `0/9` | 646 | **646** |
| `0/10` | 631 | 629 |
| `0/11` | 640 | 633 |
| `1/6` | 672 | 677 |
| `1/7` | 698 | 694 |
| `1/8` | 665 | 679 |

Four of nine land on the dump's own frame. The five that do not are the
*strike's* timing and not the field's: `set_attacking` fires from
`Unit::fight`'s tail, so a bit that arrives two frames early arrives two
frames early because the arrow did. Pinning the shape exactly and the
frames loosely is what keeps the two separable — a regression in the rule
fails on the shape whatever the engagement does.

Past 683 the clears diverge for the same kind of reason and it is worth
naming: the dump drops `0/6`'s byte at 827 and this crate does not, because
this crate's bowman is still carrying an `ATTACK` order there. The slot
arithmetic is exact on both sides — `1/7` clears here at 794, its own
`f ≡ 26 (mod 32)` — and the input to the latch is what differs.

### 9.7 What checks it

| § | what | where |
|---|---|---|
| 9.2 | `Sim::set_attacking`, from `Sim::fight`'s tail | `fight.rs` |
| 9.2 | `Sim::update_local_seen_unit` — the disc, `visible` as the mask, both planes | `vision.rs` |
| 9.2, 9.4 | `Sim::world_sees` and `Sim::target_is_seen`'s fallback | `fight.rs` |
| 9.3 | the 32-frame clear and the latch, at the head of `Sim::work` | `orders.rs` |
| 9.1 | `Unit::visible`, `Unit::attacking`, `Profile::circle_radius` | `lib.rs`, `combat.rs`, `load.rs` |

`ObjectData::visible` is now part of `crate::diff::harness::compare`, so it
is checked on **every** capture the crate walks rather than on the one the
mechanic was written for — `visible_compared`, beside `mylos` and the
packed bit. That is 37,138 further unit-frames on run79, run87 and run89
with no disagreement; those windows carry no non-zero byte on either side,
so what they check there is that this crate does not invent one.

### 9.8 Coverage

**Diff-backed**: §9.3's whole table and §9.6's, each a test
(`chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame`,
`chapter_two_s_first_attack_orders_are_the_dump_s`), and §9.5's rows, each
a pinned counter.

**Listing- and export-backed**: §9.1's writer table, §9.2's transcription
and its `+0x10c` identification (the PDB type record, the `is_dock` thunk,
and `ObjectData::is_siege`'s own body), §9.4's `WorldData::is_seen`.

**Reading-only, and owed a blind second reading**: the two building writers
of §9.1, `do_cast`'s arm, and §9.4's three always-true arms. No run on disk
executes any of them.

**What this has not established.**

- **The building half is read and not built.** `Build::do_attack` and
  `Build::do_missile_launch` write `visible`, and nothing here does;
  `Wall::update_local_seen`'s third mask term is still a stub (§6.1). *A
  capture would settle it:* a tower firing through fog, `BUILDS=7` with
  `UNITS=3`, and the `BUILDDATA` `visible` byte over the frames it shoots.
- **`Unit::do_cast`'s write is not modelled**, and no capture on disk casts
  an offensive spell.
- **`Profile::x_size` is 0 for a unit type here** where the original's
  `ObjectTypeData::x_size` is the XML's `CIRCLE_RADIUS`
  (`UnitType::init@0061ab50:655`-`662`, `min(col, 10)` into `+0x234` and
  `+0x238`). This section carries the real value as
  `Profile::circle_radius` and leaves `x_size` alone, because several unit
  paths read a *target's* `x_size` and would move under it. ~~**One of
  them is a named hypothesis for chapter two's own residue**~~ —
  **falsified 2026-09-21 by item 462, and the reading was wrong about
  which crate it described.** The falsifier the row wrote was run exactly
  as written: `x_size` and `y_size` set to `circle_radius` for every unit
  type, `chapter_two_s_word_frame_is_widened_whole` re-run over
  `[620, 628)`. **Not one row moved** — the divergence list came back
  byte-identical to the baseline — and the reason is structural rather
  than numerical. `attack_pos.rs:293` sits inside `ring_walk`, which this
  crate reaches only past `let Obj::Building(_) = target else { … }`, and
  `combat::extent` takes its `x_size` arm only when `building` is true.
  **No unit-target path in this crate reads `Profile::x_size` at all**, so
  the field could not have been the residue: the row described the
  original's `00601280` correctly and this crate's `find_attack_pos`
  incorrectly. What the residue actually was is `docs/COMBAT.md` §32 —
  the cell chain and the unit half of `find_attack_pos` — and the three
  destinations the row named are now this crate's own, exactly. The
  divergence the row could not see is that `x_size` is *still* 0 for a
  unit type, and it stays that way until something reads it.
- **§9.6's five late and early arrivals are measured and not diagnosed.**
  They are the engagement's timing, and the frames are in the table.

## 10. The resync forgets, and relights an attacker for its victims (item 709, 2026-09-24)

Golden chapter eleven (run190, `docs/GOLDEN.md` §19) stood at **1133**.
On block 1134 the original's `1/6`, who=1's chariot, has dropped its
`ATTACK` on who=0's guard `0/6` with no draw, `recharging 0`; this crate's
fired. The kill conditions for three readings are in
`docs/journal/2026-09-24-item-709.md`, written before the build.

### 10.1 The pass clears `seen`, and the whole disc relights `visible`

`GameDaemon::update_all_seen@00732840` (§6) opens, when `reveal_map !=
3`, with `World::clear_seen@006b2250`: a `memset` of `seen` (`+0x15c`)
and of its cell twin `+0x168`, each lit cell queued for the fog's
redraw first. Then it relights every active leader's objects. The unit
arm is bounded by leaders to `0xe71af0` (eight) and by each leader's
unit count (`objects +0x15c + 4·who`), and calls `update_seen(0)`
(vslot `+0x174`) on what passes vslots `+0x8` and `+0xbc`.

`Object::update_seen@00651b80`, once the line of sight is not 0:

```text
if param_1 == 0 && (visible != 0 || started_building):   ; local_18
    this->vtable[0x164]()                                  ; update_local_seen
```

So the whole-disc call relights, before its own disc, the cells a unit
has made visible to others (§9.2's `Unit::update_local_seen`, `visible`
as the mask, both planes). `seen` is `valid_target`'s fog test
(`docs/COMBAT.md` §31.2, `Sim::world_sees`). **What a resync decides**:
a target's cell stays lit for an attacker's side only if some disc of
that side covers it, or the target's own `visible` byte still carries
the side's bit. Between resyncs `seen` only grows.

This crate skipped the clear, on the ground that `seen` had no reader
(§6.1's seam). Item 447 gave it one. It also never relit `visible`
cells from the whole disc.

### 10.2 Chapter eleven, 1033 and 1133

- `1/6` has `mylos 9`, fog radius 4, and 4 is not below 4, so it sees
  from its own half-cell (§3). It was born on (3384, 13560), half-cell
  (8, 35), and the guard's post (3480, 12264) is (9, 31): dy 4, dx 1,
  `vector_dist` 4, lit.
- By 1033's resync `1/6` has walked to (3359, 13950), half-cell
  (8, 36): dy 5, not in its disc. But the guard shot `1/6` on 1011, so
  its `visible` carries who=1's bit (2) from 1012 to 1050. The resync
  relights the guard's cell for who=1 through it.
- The byte clears on 1051 (§9.3's slot). `seen` does not, until the next
  resync, so `1/6` fires through the fog on ticks 1058, 1083 and 1108.
- On 1133's resync the byte is 0 and `1/6` stands on (3384, 13896), half-
  cell (8, 36). Nothing of who=1's covers (9, 31): its other units stand
  past x 40,000. `valid_target` fails on tick 1133, the tick the reload
  opens, and the invalid-target arm drops the attack without a draw.

### 10.3 The build, and what moved

`World::clear_seen`, called first in `Sim::update_all_seen`; in
`Sim::update_seen`, `update_local_seen_unit` before the disc when the
call is the whole disc and `visible` is not 0; and `Sim::build_is_seen`
for a building target (§10.4).

| build | chapter eleven | Great Lakes |
| --- | --- | --- |
| the clear alone | falls to ~1050: `1/6` walks off on 1051 where the original's fires on 1058 | — |
| the clear and the relight | **1133 → 1139** | **falls 14982 → 9401** |
| and a building target through `ever_seen` | 1139 | **14982**, exactly |

The value diff beside it, from the widening (every record, both
directions):

| block | field | before | after |
| --- | --- | --- | --- |
| 1134 | `1/6` orders | `ATTACK`, `ATTACKTO` against `ATTACKTO` | `ATTACKTO`, both |
| 1134 | `1/6` `recharging` | 25 against 0 | 0, both |
| 1134 | `1/6` `orders_x/orders_y`, `dest_angle`, `order:length` | parted | agree |

Every row from 736 to 1141 agrees. On **1139** the original's guard rolls
its idle stand on the post (`Guy::set_anim+0x97a < Unit::do_guard+0x7f4`)
and this crate's does not. `GUYS=2` prints no animation state, so the
roll's input is not on disk. The first value part past the word is
`1/4`'s move on 1156, downstream of the draw.

### 10.4 A building target is seen through its `ever_seen` byte

`valid_target_const`'s test 5 is the **target's** vslot `+0x48`. For a
unit that is `UnitData::is_seen` (`docs/COMBAT.md` §31.2). For a
building (`Build::vftable@00b42174 +0x48`) it is
`BuildData::is_seen@0062e1a0`: the `visible` bit, then infiltration,
then `WallData::is_seen@00642bd0`. The owner sees it. Otherwise, if the
building is started or the viewer's ally mask (`LeaderData +0x6929`)
holds the owner, it is seen when `ever_seen & ally_mask` is not 0, or
under `reveal_map == 3` or two leader arms. **No fog read.** This crate
asked the fog plane for every target; while `seen` never forgot, the two
answered alike.

**The probe that found it** (scratch, not committed): the old monotone
plane kept as a shadow, and every `world_sees` and `check_ever_seen`
answer that differed from it printed on Great Lakes' 24,000 frames. One
cell differed, from 8233's resync on: who=1 asking about (5, 81), lit in
the shadow and dark after the clear. `check_ever_seen` never differed.
With `Sim::build_is_seen` Great Lakes holds at 14982.

### 10.5 What is not established, and coverage

**Diff-backed**: the clear and the relight together, on chapter eleven's
1134 rows and its every row to 1141; the building arm, on Great Lakes'
word holding at 14982 under the clear. **Listing- and export-backed**: the
unit arm's two vslot tests, taken here as `on_map` and alive; the
buildings' arm, which this crate carried before and does not change.

- `seen` at 1133 is inferred, not read: run190's `[End Frame]` set has no
  `WORLD`. A packet at logger frame 1134 would print who=1's bit on
  (9, 31); the build moving the word is the falsifier that ran instead.
- `seen3` and the cell twin `+0x168` are still not kept. The scenario
  reveal points and the frame-0 arm of `update_all_seen` are not carried;
  no capture here has either.
- A building's `visible` byte is still never set here (§9.1), so a
  building's relight through it cannot fire, and nor can
  `BuildData::is_seen`'s first arm. Infiltration, the two leader arms and
  `WallData::is_seen`'s `flags & 0x20` arm are not carried; each can only
  refuse further here.
- Which who=0 building stands on (5, 81) is not named; the probe printed
  the cell, and the diff is the word holding.

## 11. A barge born on the shore lights its disc on the water (item 1120, 2026-09-28)

East Indies' second word stood at **5975** (`docs/AI.md` §85): the AI sea
scout `1/35`'s region scan (`docs/SCOUT.md` §11) accepted 46 cells here
against 45 there. run415, a packet at logger 5975, held two half-cells seen
for who=1 there and not here: **(90, 80) and (91, 81)**, the second the
probe point of the extra cell (45, 40). run413 held both unseen at 5776.

### 11.1 The instances on the disk first

- **Who could have lit them.** Every who=1 unit within ten half-cells of
  (91, 81) on run414's 5970..5976: the Explorer `1/18` at (93, 75), whose
  disc (`mylos 6`, radius 3) stops six rows short; the citizen `1/22`,
  cargo; and the barge `1/36`, `mylos 6`, sailing west from (83, 79).
  run357 ends at 5857, before the barge; nothing prints 5858..5969.
- **The barge's birth, from its own record.** `los_x/los_y` has one writer,
  `Unit::init` (`docs/ORACLE.md`), so `1/36`'s (33912, 32280) is the point
  it was born on: half-cell (88, 84), `1/22`'s own. Ours puts the barge on
  the water at (34076, 32173), half-cell **(88, 83)**, on tick 5878.
- **The arithmetic.** §3's circle table is octagonal: `vector_dist(3, 2)`
  is `2·2 / (2·3) + 3 = 3`. Both half-cells are at distance 3 from (88, 83)
  — inside a radius-3 disc — and at 4 from (88, 84). So a disc at the spot
  lights them and the birth disc does not.

### 11.2 The rule, read off the listing

`SpellType::cast_transport@00670db0` moves the newborn boat with
`Unit::set_new_location` at `671030`; the pushes at `671027`..`67102f` are
`1, 1, y, x`, so `param_3 = 1` and `param_4 = 1` (the decompiler prints the
coordinates as `CVar6, CStack_3c`). `005f8d20`'s tile arm, when the unit is
on the map and its tile changes, compares the old half-cell (`tile >> 1`)
with the new (`div_3_table[p >> 7]`), and on a difference calls vslot
`+0x174` with `param_3 == 0` — **`update_seen(0)`, the whole disc at the
spot** — after the one `add_to_world` threw at the caster's point from
`Unit::init`.

This crate's `Sim::set_new_location` carries no reveal; its move step does
it through `Sim::moved_to`. `Sim::cast_transport` now calls
`moved_to(boat, birth point, false)` after the move.

### 11.3 What moved, and the value diff

| | before | after |
| --- | --- | --- |
| run415 (after tick 5974), `seen2` | 3 half-cells apart: (90, 80), (91, 81), (45, 73), ours 0 against 2 | **0 of 14,400** |
| run413 (after tick 5775), `seen2` | (45, 73), ours 0 against 2 | **0 of 14,400** |
| frame 5975's draws | 63 against 61, at index 49 | agree |
| East Indies' second word | 5975 | **6151** |
| run414's parted keys | 675 | 327 |

(45, 73) is the same rule, earlier: the barge `1/21`, born on tick 4444 at
(16128, 28960), lights it here only with the fix. Block 5976's three
`1/40` `g.cur_time` rows (ours 1 against 0), which read the draw after the
scan, closed. **Packet-backed** in
`diff::second::run413_s_and_run415_s_fog_grids_are_ours`, every half-cell
and every player's bit.

### 11.4 The killer

On committed `1c24946e`, the one `moved_to` line removed (`git diff --stat`
naming `transport.rs`), restored from git and `touch`ed after:

| pin | result |
| --- | --- |
| `transport::a_boat_born_on_the_shore_lights_its_disc_on_the_water` | fails |
| run346's walk | falls to 5975 |
| `run413_s_and_run415_s_fog_grids_are_ours` | four half-cells: (45, 73) on 5775 and 5974, (90, 80), (91, 81) |
| `run414_s_word_frame_is_widened_whole` | `1/40`'s `g.cur_time[0]` returns on 5976 |
| the compared pin | its window parts |

The unit test first passed without the fix. Its barge carried
`combat.domain` Sea and not `kind.domain`, so vision projected its birth
disc a half-cell ahead (§3), onto the spot. With the kind a sea unit's it
fails without the line.

### 11.5 What is not established

- **The other callers.** This crate's `set_new_location` has eight
  callers that move a unit and throw no disc: `rally.rs` (the captain's
  spot), `gaia.rs`, `lib.rs`'s placement, `collide.rs`'s push and snap,
  `orders.rs`' snap, and `transport.rs`' and `garrison.rs`' seats. In the original each one
  lights a disc when it crosses a half-cell with the unit on the map.
  Not built: none of them sits on the word's frame. A capture that holds
  one crossing a half-cell, on a frame whose fog a later read reaches,
  would settle each of them.
- `update_ceo_position`, the same arm's tail on `UnitData +0x6c &
  0x10000`, is not carried.

**Coverage.** Packet-backed: the two fog grids. Diff-backed: the word's
move and run414's block. Listing-backed: the pushes at `671027`..`67102f`.
