# The scout's explore target — `Unit::think_scout@005f6010`

*First reading, 2026-08-26, in the main thread on Opus, from the Ghidra
export (`~/ghidra-projects/decomp`), the PDB type records (`types.txt`),
the listing (`llvm-objdump`) wherever the decompiler dropped an argument or
inverted a branch, and **three draw-site traces that already contain the
mechanic**: `rontrace-run20.log`, `rontrace-fuzz-424242.log` and
`rontrace-run14.log` (`docs/ORACLE.md`, "The draw-site trace and function
coverage"). The reading was steered by the traces throughout: every claim
below about which branch runs is a branch a capture reached, and §10 is the
diff.*

**What this is.** `docs/QUEUE.md` carried, for four days, "the sim draws 15
fewer than the original at frame 0". Three of those four blocks closed on
2026-08-26 (`docs/SYNC.md` §4.2); the fourth is this one — **ten draws, at
`+0x436` ×4, `+0x458` ×2 and `+0x64c` ×4, on two unrelated maps**. This
document is what those ten are: an idle AI scout picking somewhere to
explore, by walking outward in rings around each city it knows and scoring
the unseen cells it finds.

It is also the answer to `docs/SYNC.md` §5's largest single window gap —
sim-frame 95 of run13, 6 against 23, recorded there as "the AI scout's
`think_scout` re-target — the sim has no seen map". The sim does have a seen
map (`World::seen2`, loaded from a `DUMP_ALL` dump since 2026-08-25); what
it did not have was this.

**Confidence.** The gate (§2), the ring walk (§5–§6) and the cell filter
(§7) are read line by line against the listing and then *checked against
three traces*, which between them pin the exact draw sequence on three maps;
high. **One line of §7 was wrong for four days and a fourth capture found
it** — the surface probe's tile, `(4x, 4y + 2)` where the listing reads the
cell centre; §7's own subsection has the misreading and what settled it, and
run33's frame 361 is now a diff (2026-08-30, item 76). Every frame-0 trace
had agreed with the wrong tile, because no candidate at frame 0 straddles a
shoreline. The score (§8) is read from the listing and is arithmetic no capture
separates yet — the danger term is zero in every run on disk and the goods
term needs a leader with exactly one city; medium. The target's rejection
test (§9, `find_unit_ordered`) is read only as far as this mechanic reaches
into it. The branch this document does **not** establish is the region
fallback's cell walk (§11); the goody-box head it deferred is
`docs/GOODY.md` §7 as of 2026-08-31 (§13 item 2).

**Naming.** Offsets are the PDB's: `struct /rise.pdb/UnitData`,
`LeaderData`, `WorldData`, `WData`, `Region`, `CityData`. A cell is 4 × 4
tiles; a tile is `0xc0` coordinate units, a cell `0x300`.

## 1. Shape of the mechanic

An idle unit that is a scout (or a spy) and is not in an army calls
`Unit::think_scout(0)`. In its main branch it does this:

1. Find the nearest friendly city (`find_city`) — that index becomes a
   **rotation offset** into the city list, so the scout's own nearest city
   is examined first.
2. Walk the eight leaders, starting at itself, and each one's cities.
3. Around each accepted city, walk **rings** of cells outward — ring 1 is
   the eight neighbours, ring 3 the twenty-four cells at distance 3, and so
   on. Two draws are spent at the head of every ring: a rotation and a
   phase.
4. Inside a ring, visit every `stride`th cell. A cell that is in the
   scout's own region, of the right surface, **not really seen**, and a
   valid location for this unit gets **one draw** and a score.
5. The lowest score wins. Stop scanning one ring past the ring the winner
   came from.
6. Put the scout in a one-member group and order it `EXPLORE_TO` the
   winning tile.

Everything else in the function is a fallback for when that produces
nothing good enough.

## 2. The gate — who reaches `think_scout`

`Unit::do_idle@0060dcd0` is `set_anim(CHAR_DEFAULT)`, `collide = 0`,
`check_idle`, `think`. The call this document is about is `Unit::think`'s
**tail**, at `005f7615` — the site the traces name `Unit::think+0x7da`
(that offset is the *return* address, `005f761a`):

```
if (!is_supply(this) && !is_hero(this)) {
    if ((type->role & 0x10) == 0 && !this->is(SPY, 0)) return;   // 0x3a = SPY
    if (Object::get_army(this) < 0) think_scout(this, 0);
    return;
}
add_to_army(this);
```

`role & 0x10` is the scout bit (`crate::ai_units`' `role::SCOUT`); `0x3a` is
`TypeIndex::SPY`. So **scouts and spies, not in an army**.

There is a **second** `think_scout(this, 0)` at `005f7392`-ish, inside the
block `Unit::think` enters when `leader_flags & 4` (the leader is human) or
the console's `ai off` is on; it is reached only when the unit's
`unit_masks & 0x100` is set. That block ends

```
if ((this->unit_masks & 0x40000) == 0) return;
```

and `unit_masks & 0x40000` is set at `Unit::init@00612100:586` exactly when
`(leader_flags & 0xc) != 4` — i.e. **on every unit of a leader that is not
a plain human**. So:

| the unit's leader | what happens |
| --- | --- |
| human, not AI-driven | enters the human block, does the spellcaster turn and the "scout is idle" message, then **returns** — `unit_masks & 0x40000` is clear |

| computer (or an AI-driven human) | skips the block entirely, falls through to the tail, and calls `think_scout` |

That is visible in run20's trace as it happens. Frame 0, in file order
(`report.py … draws 0`, and the interleaved `HIT` records):

```
758 HIT  Unit::do_idle
761 draw Guy::set_anim+0x97a < Unit::set_anim+0x56  < Unit::do_idle+0x7d
763 draw Guy::set_anim+0x97a < Unit::set_anim+0xb6  < Unit::do_idle+0x7d
765 HIT  Unit::think
768 HIT  Unit::think_spellcaster          <- the human scout, `0/0`
...
795 draw Guy::set_anim+0x97a < Unit::set_anim+0x56  < Unit::do_idle+0x7d
796 draw Guy::set_anim+0x97a < Unit::set_anim+0xb6  < Unit::do_idle+0x7d
797 HIT  Object::get_army                 <- the AI scout, `1/6`
798 HIT  Unit::think_scout
799 HIT  Unit::find_goody_box
800 …    the ten draws
```

(The human block's gate is `UnitData::is_special`, which the vtable
resolves to `unit_flags2 & 0x10` — `crate::ai_load`'s `uflags2::SCOUT`,
`is(SCOUT)`. So "special" here *means* scout, which is also why the block
ends with the `S_SCOUT_IDLE` sound.)

Two scouts, two figures each, four stands; the human's `think` stops at the
spellcaster turn and the AI's goes on. (`Unit::think_scout` is entered for
the first time in the frame at 798, which is what rules the human's scout
out: an `int 3` is one-shot per arming, so a *first* entry that late is
proof the earlier unit never called it.)

## 3. `Unit::think_scout(this, param_1)` — the four branches

The call site passes `param_1 = 0`, so the **region-index branch** at the
head (`if (param_1 != 0)`, which picks a random cell out of a named
region's coordinate list) is dead from here. What is left:

```
if (type->domain == 0 && find_goody_box(this)) return 1;         // GOODY §7
if ((leaders[who].leader_flags & 4) == 0)                         // not human
    if (is_spellcaster) and think_spellcaster(this) return 1;
region = get_tregion(unit.tile);                                  // the scout's region
best = 99999999; max_ring = 12; budget = 0;
near = find_city(unit.pos, SEARCH_FRIENDLY, who, 0x200, FILTER_ALL);
if (near < 0) near = 0;
if ((this->unit_masks & 0x100) != 0
    || (type->type_index != PEASANTS && type->type_index != PEASANTSKOREAN
        && !this->is(SPY, 0) && type->domain != 1))
        goto CITY_LOOP;                                           // §5
goto REGION_SCAN;                                                 // §11
```

`UnitTypeData +0x4` is the type's own `TypeIndex`, and `0x32`/`0x33` are
`PEASANTS` and `PEASANTSKOREAN` — so a **citizen** takes the region scan and
a scout takes **CITY_LOOP**, which the three traces confirm: every frame-0
draw is at one of the city loop's three sites. A spy and a naval unit take
the region scan too.

After the city loop:

```
CITY_LOOP_DONE:
if (best > 199) goto REGION_SCAN;         // §11
if (best > 99999998) { mark the region scouted; think_civilian_transport } // §12
goto ISSUE;                               // §9
```

## 4. The circle tables — `circle_init@006817f0`

Three globals, filled once at startup by pure integer arithmetic and never
written again:

| symbol | address | what |
| --- | --- | --- |
| `circle_x` | `00cb7e90` | `char[0x3250]` — the x offset of each point |
| `circle_y` | `00cbb0e0` | `char[0x3250]` — the y offset |
| `circle_radius` | `00cbe330` | `int[0x41]` — cumulative point counts |

`circle_init` sweeps `r = 0 .. 0x40`; for each `r` it scans the square
`[-r, r]²` in `x`-major order and appends every `(x, y)` whose **octagonal
distance** is exactly `r`, then writes `circle_radius[r] = points so far`.
The distance is the engine's own `vector_dist` shape, inlined:

```
hi = max(|x|, |y|); lo = min(|x|, |y|);
d = hi == 0 ? 0 : (lo < 60000 ? lo*lo/(2*hi) + hi : (lo + 2*hi) >> 1)
```

So **ring `r` is the index range `[circle_radius[r-1], circle_radius[r])`**,
and the function reads the start through the base `00cbe32c`, which is
`circle_radius - 1` indexed by `r`. The first rings, re-derived:

| ring | start | end | size |
| --- | --- | --- | --- |
| 1 | 1 | 9 | 8 |
| 2 | 9 | 21 | 12 |
| 3 | 21 | 45 | 24 |
| 4 | 45 | 69 | 24 |
| 5 | 69 | 105 | 36 |
| 6 | 105 | 145 | 40 |
| 7 | 145 | 185 | 40 |

Ring 1 being the eight neighbours is the check that the sweep order is
right. The table stops at 12,873 points (`circle_points`), the cap being
`0x3248`.

## 5. The city loop

```
if (leaders[who].city_num == 0) goto REGION_SCAN;
for (i = 0; i < 8; i++) {
    L = (who + i) & 7;
    if ((leaders[L].leader_flags & 2) == 0) continue;
    if (leaders[L].reg_cities[region] == 0) continue;
    if (L != who && is_ally(leaders[L], who)) continue;
    n = leaders[L].city_mark;
    for (j = 0; j < n; j++) {
        c = cities[L][((L == who ? near : 0) + j) % n];
        if ((c->city_flags & 1) == 0) continue;              // not alive
        if (type->domain == 0 && c->reg != region) continue;
        if (c->scouted & (1 << who)) continue;               // CityData +0x4c
        <the ring walk, §6>
        if (ring == (L == who ? 12 : 8))                     // `ring` after the walk
            c->scouted |= 1 << who;                          // exhausted
        max_ring = 8;
        if (L != who) break;                                 // one foreign city only
    }
}
```

Three things there are worth naming.

- **`leaders[L].reg_cities[region]`** (`LeaderData +0x125e`, `ushort[64]`)
  is the leader's city count *in the scout's own region*. A leader with
  nothing on this landmass is skipped whole.
- **`CityData +0x4c`** is a per-leader "I have finished exploring around
  this city" bitmask, set when the ring counter comes out of §6's loop
  equal to `(L == who) * 4 + 8` — 12 for one's own city, 8 for a foreign
  one. The counter is the ring *after* the last one walked, so it lands on
  12 only from `max_ring = 12` with `step = 1`, and on 8 only from
  `max_ring = 8` with `step = 1`. **An AI scout therefore never sets this
  bit**: `step` is 2 around its own city (final ring 13 or 9) and its
  foreign arm runs `max_ring = 3` (final ring 3). Only a plain human's
  scout can, and only on the first city it examines and on the foreign ones
  after that. That reads as an oversight in the original rather than a
  design, and it is transcribed as it stands. Nothing in the export clears
  the bit.
- **`max_ring` starts at 12 and drops to 8 after the first city examined.**
  It is *not* reset per city. So the first city gets the deep walk and
  every later one gets the shallow one.

`find_city`'s return (`near`) is the index **within its owner's city list**
of the nearest friendly city in the scout's region; used only as the
rotation offset for the scout's own leader.

## 6. The ring walk — the two draws at `+0x436` and `+0x458`

`cx`, `cy` are the city's cell (`CityData +0xc`/`+0x10`, `>> 8` then the
divide-by-three table). `step` is 2 when `unit_masks & 0x40000` **and** the
city is the scout's own leader's — i.e. an AI scout around its own city
skips the even rings; otherwise 1. A unit with `0x40000` looking at a
*foreign* city instead gets `max_ring = 3`, `best_ring = 3`, `step = 1`.

```
best_ring = max_ring;                       // lowered on every hit
for (ring = 1; ring < max_ring; ring += step) {
    phase  = ring / 4 + frame % 8;          // truncating divide
    stride = phase + 1;
    end    = circle_radius[ring];
    rot    = end > 1 ? rand() % end : 0;                        // +0x436
    idx    = phase >= 1 ? rand() % stride : 0;                   // +0x458
    start  = circle_radius[ring - 1];
    for (i = start + idx; i < end; i += stride) {
        k = start + (rot + i) % (end - start);
        wx = cx + circle_x[k];  wy = cy + circle_y[k];
        if (ring > best_ring + 1) goto NEXT_CITY;    // one ring past the hit
        <the cell filter, §7>
    }
}
```

Both draws are `Random::get(game_random, 0, 0xffff)` followed by a signed
`idiv`; the trace's `+0x436` and `+0x458` are the two return addresses.
Note the guards, which are the whole reason the two counts differ: the
rotation draw is skipped only when `circle_radius[ring] <= 1` (never, for
`ring >= 1`), and the phase draw only when `ring / 4 + frame % 8 == 0` —
**at frame 0, only for rings 1, 2 and 3**.

Note also that `rot` is taken modulo the *cumulative* count `end`, and the
cell index modulo the *ring size* `end - start`. That is not a typo in the
reading; it is what the listing does at `005f6424` and `005f6497`.

## 7. The cell filter — the draw at `+0x64c`

```
if (wx < 0 || wy < 0 || wx >= world.xs || wy >= world.ys) continue;
d = type->domain;
if (d != 2) {                                          // an aircraft takes anything
    if (wdata[wy*xs + wx].region != region) continue;
    surf = tdata[(4*wy + 2)*tile_xs + 4*wx + 2] & 0x30;   // the cell centre
    if (d == 0 ? surf == 0x20 : surf != 0x20) continue; // land wants not-ocean
}
if (++budget > 0x600) goto CITY_LOOP_DONE;             // 1536 cells, whole-call
owner = wdata[wy*xs + wx].who;  if (owner < 0) owner = who;
if (was_really_seen(2*wx + 1, 2*wy + 1, who)) {
    if (owner != who) continue;
    if (wdata[wy*xs + wx].<short at +0> >= 0) continue;
}
if (invalid_loc(4*wx + 2, 4*wy + 2, 0,0,0,0,0) != 0) continue;
score = vector_dist(|ux - wx|, |uy - wy|) * 8 + rand() % 8;      // +0x64c
<the score, §8>
```

`WorldData::was_really_seen@006b54f0` is **not** `was_seen`: it is the bare
fog read

```
who < 8 && reveal_map != 3 && !(leader_flags & 0x800) && leader.+0x59e4 == 0
    ? seen2[(2wx + 1) + fog_xs * (2wy + 1)] & leader.ally_mask
    : 1
```

with none of `was_seen`'s ally-territory shortcut. So an ally's *territory*
does not count as seen here, only the fog grid does — which is exactly what
makes this the mechanic that needs the seen map.

The escape when the cell **is** seen (`owner == who` and the cell's first
`short` is negative) lets a scout re-target onto its own unclaimed ground.
`WData +0` is `flags`, read as a *signed* short, so "negative" is bit
`0x8000` — `GOODY`, a cell carrying a goody box. Great Lakes has 22 of
them and none is ever a candidate in this capture, so the escape is a
seam here; it is named rather than unknown.

**The fog read is what this filter actually turns on**, and it is only as
good as `seen2`. Item 79 (2026-08-30) was a wrong *reveal* three hundred
frames upstream showing up here as a refused candidate: see §12's
`run33_s_scout_re_targets_at_482_on_the_original_s_ring`.

`vector_dist` is measured in **cells**, from the scout's cell to the
candidate's, and multiplied by 8; the `rand() % 8` (a signed
`& 0x80000007` fold) is a jitter that breaks ties between equidistant
cells.

### The surface tile is the cell centre — and how it was read wrong

~~`4*wx`, not `4*wx + 2`~~. This document carried the `x` uncentred for
four days, and it was a **decompiler fold read as a field offset**. Ghidra
prints the probe as

```
*(byte *)(world->tdata + 4 + ((wy*4 + 2) * world->tile_xs + wx*4) * 2) & 0x30
```

and the `+ 4` looks exactly like the `+ 4` two lines above it, which really
is a field offset — `wdata[...]` has stride `0x1c` and `WData::region` at
`+0x4`. But `TData` is **`size 0x2` with `mask` at `+0`**, so four bytes is
**two elements**, and the listing says so plainly:

```
005f652e  leal 0x2(,%edi,4), %eax      ; 4·wy + 2
005f6535  imull 0x18(%ebx), %eax       ; × tile_xs
005f6539  leal (%eax,%esi,4), %ecx     ; + 4·wx
005f653c  movl 0x138(%ebx), %eax       ; tdata
005f6542  movb 0x4(%eax,%ecx,2), %al   ; tdata[ecx + 2].mask
```

The canonical accessors through the same array carry no constant at all —
`WorldData::is_cliff_at@0046f8c0` and `is_tocean_slow@006b2400` are both
`tdata[(tile_xs·ty + tx) * 2]` — which is the second thing that settles it.

So the surface tile is `(4x + 2, 4y + 2)`, **the same tile `invalid_loc` is
handed four lines later**, and the asymmetry the note was written to explain
never existed.

**What it cost, and what the diff says.** On run33's frame 361 the AI scout
`1/0` re-targets. Cell `(48, 23)` carries ocean at tile `(192, 94)` and land
at `(194, 94)`; with the uncentred probe the candidate was refused, the call
spent **twenty-six** draws where the original spends twenty-seven, and the
scout went to `(48, 20)` instead. With the centre tile the two agree draw
for draw and the scout takes the original's cell —
`rondata::diff`'s `run33_s_scout_re_targets_at_361_on_the_original_s_cell`
is that frame as a sequence and as a destination, and the headline went
**362 → 436** (2026-08-30, item 76).

The lesson generalises and is now in `docs/audit/README.md`: **a constant
byte offset inside a decompiled array index is a field offset only if the
element is wide enough to hold one.** Check the record's size before
reading it as one.

## 8. The score

```
if (L != who && (L < 0 || (leaders[who].treaties[L] & 3) == 0)) {
    score *= 2;
    if ((leaders[L].leader_flags & 4) == 0) score *= 2;   // a computer's city
}
score += danger[who][(wy >> 1) * reg_xs + (wx >> 1)];
score += (owner != who) ? 4 : 0;
if      (leaders[who].city_num == 0 && (goods(wx, wy) & 0x02)) score /= 2;
else if (leaders[who].city_num == 1 && (goods(wx, wy) & 0x10)) score /= 2;
if (score >= best) continue;
if (find_unit_ordered(<same basic type, mine, not me, within 0x600>) >= 0) continue;
best = score; target = (4*wx + 2, 4*wy + 2); best_ring = min(best_ring, ring);
```

Read down it: **near beats far** (the `dist * 8`), **one's own ground beats
a neighbour's** (the `+4` and the `× 2`/`× 4` around a rival's city), **a
computer rival is worth twice a human one**, and **a cell with the right
good on it is worth double** while the leader is still small. `score /= 2`
is the arithmetic halving (`cltd; sub; sar`), rounding toward zero.

`danger` is `WorldData +0x13c`, an `int *[8]` at half the cell resolution
(`reg_xs = world +0x24`). It is zero in every capture on disk at the frames
this mechanic has been observed at.

`find_unit_ordered` is what stops two scouts converging: a unit of the same
`basic_type`, of the scout's own leader, that is not the scout, within
`0x600` (two cells) of the candidate's centre, rejects the candidate
outright.

## 9. The order

```
Group g = Group::clear(-1);
Group::add(&g, this->o, who, 0, 0);
slot = Groups::push_group(who, &g, 1);          // force = 1: a one-member group installs
Group::action_move_to(&groups[slot],
                      target.x * 0xc0 + 0x60, target.y * 0xc0 + 0x60,
                      QUEUE_NEW, 0, 0, EXPLORE_TO, 0, -1, -1, 0);
```

The target is in tiles and is converted to the **tile centre**, which is
half a tile past the cell centre the scan actually scored. `EXPLORE_TO` is
the move kind the pathfinder reads for its unseen-cell preference
(`docs/PATHFINDER.md`, `crate::path`'s `scouting`).

`push_group` is called with `force = 1`, which is the one case
`docs/GROUPS.md` §3.2 names where a group of fewer than two installs.

## 10. The frame-0 fold — what the traces say

The whole mechanic is ten draws at frame 0 on run20 and on the fuzzer's
control map, and twenty-four on run14's Great Lakes. Folded by site and
then read in file order:

| capture | `+0x436` | `+0x458` | `+0x64c` | the sequence |
| --- | --- | --- | --- | --- |
| run20 (East Indies) | 4 | 2 | 4 | `436 436 436 458 64c×4 436 458` |
| fuzz 424242 | 4 | 2 | 4 | the same |
| run14 (Great Lakes) | 6 | 2 | 16 | `436 436 436 458 64c 436 458 436 64c×5 436 64c×10` |

Read run20's against §6 with `frame % 8 == 0`, `step = 2`, `max_ring = 12`:

| ring | `phase` | `+0x436` | `+0x458` | cells accepted |
| --- | --- | --- | --- | --- |
| 1 | 0 | 1 | — | 0 — the city's own neighbours are all seen |
| 3 | 0 | 1 | — | 0 |
| 5 | 1 | 1 | 1 | **4**, `stride = 2` so 18 of the ring's 36 are visited |
| 7 | 1 | 1 | 1 | 0 — `best_ring = 5`, so `7 > 5 + 1` breaks at the first cell |

The step of 2 is what puts both `+0x458` draws on rings 5 and 7 rather than
one on ring 4; a step of 1 gives `436 436 436 458`, which is not what any
of the three traces shows. That is the observation that identified
`unit_masks & 0x40000` as the AI-unit bit rather than the amphibious one
`crate::path` had guessed.

Run14 splits into **two cities**, which is why its shape is different: the
first (the scout's own, `max_ring = 12`, `step = 2`) gives rings 1, 3, 5, 7
with one hit on ring 5 and the ring-7 break; the second is a **foreign**
city, so `unit_masks & 0x40000` sends it down the `max_ring = 3`,
`step = 1` arm — rings 1 and 2, no phase draw at either, five hits and then
ten. Nothing else in the function produces two consecutive `+0x436`s with
no `+0x458` between them.

### The simulation against those three, seed for seed

The count is not the check. Every draw record carries the **seed it was
taken on**, so the whole sequence can be replayed: install the seed of the
trace's first `think_scout` draw in the harness's own scout and run
`Sim::think_scout`. On all three captures it comes out exact — the same
number of draws, at the same sites, in the same order, and the seed left
standing is the one the trace's last draw carries.

| capture | the trace's first draw | ours | the sites |
| --- | --- | --- | --- |
| run20 | `0x9c59_1b2b` | **10** | 4 / 2 / 4, ring 7's rotation on `0xa358_e033` — the trace's draw 32 |
| fuzz 424242 | `0x242c_b7ed` | **10** | 4 / 2 / 4, ring 7's rotation on `0x5d72_1535` — its draw 32 |
| run10/run14 | `0x15fe_bc41` | **24** | **6 / 2 / 16**, and the split is the two cities of the paragraph above: 4 / 2 / 1 then 2 / 0 / 15 |

The third row is the one worth reading twice. The Great Lakes map exercises
the *foreign*-city arm — `max_ring = 3`, `step = 1`, rings 1 and 2 with no
phase draw at either — and the harness reproduces its five hits and its ten
without being told anything about them.

### And the seed was not the frame's own — which is how the swap was priced

Run on the frame's own stream rather than the trace's, the harness reached
`think_scout` **two draws early**. It spent a stand in the unit loop for
each gathering citizen where the original spends none and wraps the same
figures in phase 7 instead (`docs/SYNC.md` §4.2, §6). That defect had been
recorded as **zero-sum** — "no count will ever catch it". It stopped being
zero-sum the moment this mechanic landed, because `think_scout`'s own draw
count depends on the stream it runs on: the rotation decides which cells of
a ring are visited and the fog decides how many of those are taken.

| | run20 | the fuzzed map |
| --- | --- | --- |
| before this mechanic | 165 / 175 | 185 / 195 |
| after, on the frame's own stream | **175 / 175** | **196** / 195 |
| after, on the trace's seed | 175 / 175 | **195 / 195** |
| **with the swap closed** (2026-08-26) | **175 / 175** | **195 / 195** |

Run20 landed on 175 either way — its ring 5 gives four cells on both
streams. The fuzzed map gave five on the harness's stream and four on the
original's, and that one cell was the visible price of the stand/wrap
swap. The swap is closed: the sim's camp-arrival stand was an invention of
its own, the original's branch there is a two-way `CHAR_DUMP_*`, and
removing it put the wraps back as well (`docs/SYNC.md` §6). ~~A `GUYS=4`
frame-0 capture is the check that settles it.~~ It was settled by the
whole-frame sequence check instead (`docs/SYNC.md` §5.1), and the capture
was never booked — the scout's own seed-anchored check is what made that
possible, because it held while the stream reaching it was still wrong.

## 11. The region fallback — read, not implemented

When the city loop leaves `best > 199` — no city in the scout's region, or
nothing near enough — the function falls into a second scan over the
scout's **whole region**, `005f68e3`:

```
n = regions[region].size;
stride = max(1, (n + 99) / 100) + frame % 8;
i = stride > 1 ? rand() % stride : 0;                     // one draw
for (; i < n; i += stride) {
    (wx, wy) = regions[region].coords[i];
    if (was_really_seen(2wx+1, 2wy+1, who)) continue;      // inlined, not called
    if (invalid_loc(4wx+2, 4wy+2, ...) != 0) continue;
    if (domain == 2) accept;
    else if (domain == 0 || get_inside(this) >= 0) { if (surface != 0x20) accept; }
    else if (is_ocean(wx, wy)) accept;
    accept:
        score = vector_dist(...) * 16 + rand() % 8;        // one draw per accepted cell
        ...
}
```

with `local_74` — set just above it, and true for an AI unit that is not a
spy, not "special" and not naval — switching the score's second half from
"danger plus a territory penalty" to "quarter it if the owner is not an
ally, halve it again on a `0x100` tile".

**The cell walk is not reproducible here**, because it iterates
`Region.coords` (`Region +0x6c`, a `WCoordList`) in the order the map
generator's flood fill built it, and no dump carries that list. The
`stride` draw is: `Region.size` is `World::region_size`. So the simulation
takes the first draw and stops, and a run that reaches this branch will be
short by one draw per accepted cell. No capture on disk reaches it — the
AI scout's own city is always in its region at the frames observed — and
`docs/SYNC.md` §6 carries it as an open item.

## 12. What the simulation carries, and what checks it

`crates/sim/src/scout.rs`:

- `scout::circle::table()` — `circle_init` re-derived, built once behind a
  `OnceLock`. The tables live in `.bss` and are filled at startup, so there
  is nothing in the executable to transcribe: this is the loader, and ring
  1 coming out as the eight neighbours is what says the sweep order is
  right.
- `Sim::think_scout` — §3's head, §5's city loop, §6's ring walk, §7's
  filter, §8's score, §9's order. Public, so the harness can drive one
  call on a chosen seed.
- `Sim::was_really_seen` — §7's fog read; the sibling of
  `crate::ai_sites`' `site_was_seen`, which is `was_seen` and is a
  different function with one word between their names. **The grid it
  reads is no longer frozen** (2026-08-27): `crate::vision` writes `seen2`
  as units move, so a scout that has walked for a hundred frames filters
  its candidate cells against what it has actually seen. `docs/VISION.md`.
- `Sim::scout_thinks` and `Sim::unit_is_scout` — §2's gate, wired into
  `Sim::think`'s tail in `crates/sim/src/orders.rs`, where it is exclusive
  with `think_join_army` as it is in the original. **The other arm caught
  up 2026-08-29** (item 68): `think_join_army` was joining any attacker,
  where §2's listing joins a supply wagon or a hero and nothing else, so
  the AI's woodcutters were being conscripted and marched off. This
  section had the listing right; `docs/ARMY.md` §4 did not.

§7's `invalid_loc` is not a new entry point: `crate::path`'s
`Sim::invalid_loc` already carries `UnitData::invalid_loc` and this call
site is its all-flags-zero case. **One seam of it closed with this
mechanic** — `WorldData::is_cliff_at@0046f8c0` is exactly
`(TData.mask & 3) == 1`, so `crate::world::tile::OBJECT_CLIFF` is named and
the land arm refuses a cliff as the original does (`docs/PATHFINDER.md`
§11).

§5's `CityData +0x4c` is **not** carried, and it is a deliberate omission:
the only writer is the mark at the end of §5, which an AI scout can never
reach (its ring counter comes out at 13 or 9, never 12 or 8), and no
human's scout reaches this function at all. The simulation reads it as
always clear and `debug_assert!`s that the mark is unreachable, so a future
change that makes it reachable fails loudly rather than quietly.

The checks:

- `sim::scout`'s own tests: `circle_init`'s first rings and its eight
  neighbours; the ring walk alone, on a world where everything is seen, at
  its six rotations and four phases; a world where everything is unseen,
  where ring 1 is taken whole and ring 3 breaks at its first cell; §2's
  gate from both sides; and the `city_num == 0` fall-through to §11's
  stride draw. Four deliberate breakages, each made to fail before it
  landed: `step` 1 instead of 2, the early exit removed, the phase guard
  relaxed to `>= 0`, and the fog gate short-circuited.
- `rondata::diff`'s
  `run20_s_ai_scout_draws_ten_at_frame_0_in_four_rings` — the whole ten as
  a **sequence**, not a count. It reads `rontrace-run20.log` through
  `rondata::trace`, filters it to the draws `think_scout` took itself
  (`scout::CODE`, the function's address range), and compares site for
  site against the harness's own marks (`scout::SITE_ROTATION`,
  `SITE_PHASE`, `SITE_CELL`, expanded by `diff::mark_sites`). Plus the
  frame-0 count (175 against 175) and §9's `EXPLORE_TO` at ring 5 of the
  AI's city. Made to fail by transposing two of the three marks, which
  leaves the count at ten and the sequence wrong — the case a total cannot
  see.
- `rondata::diff`'s
  `run33_s_scout_re_targets_at_361_on_the_original_s_cell` (2026-08-30) —
  the one call in the corpus where the **cell filter** decides something.
  Three hundred frames into run33 the AI scout `1/0` re-targets, and the
  original spends twenty-seven draws there over seven rings; this compares
  the whole sequence and the destination the dump prints for the order the
  call issues, on a stream the simulation **reaches** rather than has
  installed. Made to fail by putting §7's surface probe back on `4*wx`,
  which is the defect it was written for: twenty-six draws against
  twenty-seven, and the wrong cell.
- `rondata::diff`'s
  `run33_s_scout_re_targets_at_482_on_the_original_s_ring` (2026-08-30) —
  the same scout's *next* re-target, a hundred and twenty frames later, and
  what it pins is §7's **fog** read rather than its surface read. The
  original scores three cells in the first city's ring 7 where this
  simulation scored two: cell `(56, 28)` was seen here and not there,
  because `Object::update_seen` had been throwing the scout's vision disc
  along the guy's eased facing instead of `UnitData::angle`, the unit's own
  heading (`docs/VISION.md` §3), and on frame 168 the scout was mid-turn
  with thirty-one degrees between them. The trace cannot say *which* cell,
  since every accepted cell draws at the same site — §8's arithmetic can:
  of the five cells this refuses in that ring, `(50, 26)`, `(48, 20)` and
  `(48, 24)` score 48, 0 and 32 against the winner's 76 and would have
  taken the frame, so only `(56, 28)` and `(58, 28)` are consistent with
  the destination the dump prints, and only `(56, 28)` was revealed on a
  frame the scout was turning. The word went 482 → **571**.
- `run20_s_pasture_grows_nothing_and_its_five_animals_draw_six`, whose
  frame-0 row moves from 165/175 to **175/175** with this.
- `rondata::diff`'s
  `run39_s_sixth_farm_lights_the_cells_its_scout_then_paths_around`
  (2026-08-31, item 99) — which is **not** this mechanic and is here
  because the frame it pins looked exactly like it. East Indies parts at
  413 with two `do_idle` anims and this call's nine draws missing, and the
  cause is three cells of fog three hundred frames upstream: a building
  lights its own disc when it finishes (`docs/VISION.md` §2.1), nothing
  here did, and the scout's `EXPLORE_TO` path therefore ran through
  ground the original could see and priced accordingly. The scan itself
  was never wrong; the scout was twelve frames late to run it. The word
  went 413 → **576**.
- `crate::no_float` and `crate::soak` as everywhere else.

## 13. What is not established

1. **The region fallback's cell walk** (§11). It needs `Region.coords` in
   the generator's order and no dump carries it. *Capture:* none would
   help; a `REGIONS` dump detail that printed the coordinate list would.
2. ~~**`Unit::find_goody_box`** — the very first thing `think_scout` does
   for a land unit, and it returns 1 (and skips everything here) when it
   finds a goody box to walk to. Not read; the simulation treats it as
   always failing.~~ **Read and implemented 2026-08-31** (item 106):
   `docs/GOODY.md` §7 is the 49-cell sweep, its two fog gates and the
   `EXPLORE_TO` it issues. It is still true that no capture *accepts* at
   this call site — the one that fires on East Indies is
   `Unit::do_explore_to`'s fifteen-frame look, and by the time a scout is
   idle beside a box the box is taken.
3. **The score's danger term** (§8). Zero in every capture. *Capture:* a
   window at a frame where `GameDaemon::calc_danger` has written a non-zero
   figure near a scout, with the scout's chosen target in the same window.
4. **The goods halving** (§8) needs `city_num` to be 0 or 1; every capture
   has the AI at exactly one city at frame 0, so the `& 0x10` arm is live
   and the `& 0x02` arm is not. Which good bits those are is not read.
5. **`find_unit_ordered`'s filter chain** (§8/§9). The simulation
   implements the predicate the call site asks for — same basic type, mine,
   not me, within `0x600` — and not `Search::valid_search`'s
   leader-visibility layer, which for a search of one's *own* units is
   believed to be a no-op. *Capture:* two scouts of one AI leader within
   two cells of the same candidate.
6. **`CityData +0x4c` is never written by an AI scout, and never
   cleared.** The mark's constant is `(L == who) * 4 + 8` and the ring
   counter that meets it can only be 12 or 8, which `step = 2` and the
   foreign arm's `max_ring = 3` both miss — so the bit is unreachable from
   here for a computer's scout, and nothing in the export clears it. Read
   as an oversight in the original and transcribed as it stands; the
   simulation does not carry the field and asserts the mark unreachable
   (§12). *Capture:* a long window with a **human's** scout on the
   `unit_masks & 0x100` call site, which is the only path that can set it.
7. **The `short` at `WData +0`** (§7), the second arm of the seen escape.
   Unnamed in the type record; the simulation reads it as the cell's
   first field and the harness's worlds leave it 0, so the escape never
   fires there.
8. **The second `think_scout` call site** (§2), the one inside the human
   block gated on `unit_masks & 0x100`. Not modelled — no capture reaches
   it, since `0x100` is clear on every scout in every run on disk.
8b. **The order's destination is refined a frame later, and by what is
   not read.** §9 issues the move at the **tile centre**, `4·cell + 2`
   scaled to `tile·0xc0 + 0x60`. Run33's dump prints `1/0`'s new order
   first on frame 362 with `dest 37368, 18168` — the centre plus 24 on
   both axes, which the simulation reproduces — and on frame 363 with
   `dest 37344, 18144`, the centre exactly. So something between the two
   frames takes the offset back off and this crate does not do it; the
   harness's order diff holds for `1/0` to 483 regardless, so no score
   sees it yet. *Capture:* a `UNITS=3` window over the two frames after
   any `EXPLORE_TO`, with `GROUPS=1` to say whether the group's own slot
   layout is what writes it.
9. ~~**The upstream stream, not this mechanic.** On the frame's own stream
   the harness reaches `think_scout` two draws early (§10), so the target
   it picks is not the original's on any map where the rotation matters.
   Everything in §5–§9 is checked on the trace's seed and nothing here is
   checked on the frame's; closing the stand/wrap swap is what would join
   the two. *Capture:* a frame-0 `GUYS=4` window (`docs/SYNC.md` §6).~~
   **Closed 2026-08-26** (`docs/SYNC.md` §5.1, §6): the swap was one line
   of `do_non_flat_gather`, the two streams are now the same one on both
   traced maps, and this mechanic's ten draws are reached at the
   original's own word without installing it. The capture was not needed.
10. **`Unit::think_spellcaster`** at the head of §3, which an AI scout
    calls on every one of these frames and which draws nothing in any
    capture. Unread; the simulation skips it. *Capture:* a window with a
    caster whose spell is off cooldown.
