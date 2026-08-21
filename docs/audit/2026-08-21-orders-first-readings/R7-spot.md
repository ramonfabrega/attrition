# R7 — the spatial queries every order's walk ends in

First reading, 2026-08-20/21, from the decompile export (`~/ghidra-projects/decomp`)
with the disassembly (`llvm-objdump`, image base `0x400000`) wherever the
decompiler dropped an operand — which it did for every angle, every
`find_angle` argument pair and the `project` call. Written from understanding;
nothing is transcribed.

**Headline.** Every walk an order makes ends at a point that
`UnitType::find_nearby_spot@0061de70` returns, and that function is entirely
deterministic and RNG-free: it sweeps concentric rings of radius `min, min+step,
…, max` around a centre, and on each ring tries **31 bearings in a fixed order**
— the base bearing, then ±1, ±2, … ±7 sixteenths of a turn, then the odd
thirty-seconds (one of them twice, one never, and the direct opposite never) —
snapping each projected point to the centre of its 48-unit quarter-tile and
taking **the first candidate** that is on the map, of the right terrain class,
not on a `0x4000` tile, not inside the target building's footprint, and not
colliding. The base bearing at every gather/build/garrison call site is
`find_angle(unit − target)`: the ring is searched starting on the unit's own
side of the target. The collision half uses, for the normal `FILTER_NOT_ME`
case, the quarter-tile occupancy grid (`CollCheck::collide_here`) plus a
9-cell scan of other units' **ordered** positions (`orders_x/y`), both with
`new_block_radius` in quarter-tiles; a building's `new_block_radius` is 0, so
buildings are kept off only by their `0x4000` tiles and the explicit footprint
test. `UnitData::invalid_loc` is a different predicate (terrain + `0x4000`,
five behaviour flags, fog-aware for humans) that `find_nearby_spot` does **not**
call; on an open flat map away from water, forest, mountains and buildings it
returns 0. `Object::adjacent_to` is `attack_dist < 0x60` with one exception for
a sea-domain *caller*. `WData.down/down_who` is the head of a per-cell object
chain, not a territory claim (a correction for `docs/ATTRITION.md`).

Reading conventions: position units are 1/192 tile (`docs/FORMATS.md`); a
**UCoord** is a quarter-tile, 48 units, `div_3_table[v >> 4]`; a tile is
`div_3_table[v >> 6]`; a cell is a 4×4 tile block, `tile >> 2` or
`div_3_table[v >> 8]`. `div_3_table[i] = floor(i / 3)`. Angles are the signed
32-bit binary angles of `docs/MOVEMENT.md` (north 0, east `0x40000000`, a full
turn 2³²; `0x10000000` is a sixteenth of a turn, `0x8000000` a thirty-second).
Object positions are `SubObjectData::x_internal/y_internal` (+0x10/+0x14)
XOR `0x63637`. Struct field names are from `types.txt`; `UnitType`'s are on
`ObjectTypeData` (`types.txt:41157` lists `UnitTypeData` mostly as padding —
the base `ObjectTypeData` is where `attack +0x1e8`, `domain +0x218`,
`guy_spacing +0x224`, `x_size +0x234`, `y_size +0x238`, `guy_radius +0x23c`,
`block_radius +0x240`, `big_radius +0x244`, `new_block_radius +0x248`,
`new_big_radius +0x24c` live; `unit_flags +0x2b4`, `squad_size +0x304`,
`uber_size +0x308` are `UnitTypeData`'s own; `Type +0x4` is `TypeData::type`,
the `TypeIndex`).

---

## 1. `UnitType::find_nearby_spot@0061de70`

### 1.1 Signature and what each argument is

`int UnitType::find_nearby_spot(T, x, y, *out_x, *out_y, min, max, step, angle,
filter, o, who, nocoll, uber, bo, bwho, region)` — `__thiscall`, 16 stack
arguments (`ret 0x40`), `this` = the **unit type** doing the standing.

| # | name here | in/out | meaning, from the body |
| --- | --- | --- | --- |
| this | `T` | in | the `UnitType` whose radii, domain and `TypeIndex` decide the test |
| 1, 2 | `x, y` | in | the centre, position units (a building's centre, a unit's position, a tile centre…) |
| 3, 4 | `out_x, out_y` | **out** | written on every return: the accepted spot on success, **the input `x, y` on failure** (prologue copies `x, y` to the two locals the epilogue stores, `0x61de86/0x61dea4`, `0x61e3e1–0x61e3ef`) |
| 5 | `min` | in | the first ring radius, position units; `0` means "the centre itself is the first candidate" |
| 6 | `max` | in | the last ring radius; **`0` (with `min > 0`) or negative = default**, §1.2 |
| 7 | `step` | in | ring increment; **`≤ 0` = default `(max − min) / 8`**, clamped to ≥ 1 |
| 8 | `angle` | in | the base bearing (binary angle) from the centre at which each ring's sweep starts |
| 9 | `filter` | in | `FilterIndex`: `FILTER_ALL 0`, `FILTER_TYPE 1`, `FILTER_BASE_TYPE 2`, **`FILTER_NOT_ME 3`**, `FILTER_NOT_MY_UBER 4`, **`FILTER_CAN_COLLIDE 5`**, `FILTER_CONSTRUCT 6`, … (PDB `LF_ENUM FilterIndex`, `pdbtypes.txt:18062`) |
| 10, 11 | `o, who` | in | the object the filter is relative to ("me"): `FILTER_NOT_ME` excludes exactly `(o, who)`; `−1` = none |
| 12 | `nocoll` | in | non-zero: **skip the collision test entirely** — any passable candidate is accepted (`0x61e334`) |
| 13 | `uber` | in | non-zero: a whole squad is being placed — widens the default `max` and the collision radius by half a formation (`((uber_size − 1) · guy_spacing) / 2`), and forces the general collision path |
| 14, 15 | `bo, bwho` | in | a building whose **footprint tiles are refused** (except a FARM for a peasant type), §1.4 step 5; `−1` = none |
| 16 | `region` | in | `≥ 0`: the candidate's cell must have this `WData.region` (or `region2` on a coastal cell's ocean tile); `−1` = no constraint |

Return: **0 = found** (`out` = the spot), **non-zero (1) = none** — the
epilogue is `sete al` on a "found" local that starts 0 (`0x61e3f1–0x61e3f8`);
the decompile's `return (uint)!bVar6`. R4's reading of the convention is
correct.

There is a `Unit::find_nearby_spot@00617010` wrapper, 14 arguments
(`ret 0x38`): it passes `this->ptype`, its own `(o, who)` (`SubObjectData
+0xa/+0x9`) as the filter's "me", then its arguments 10 and 11 as `nocoll` and
`uber`, **constants `−1, 0` as `bo, bwho`**, and its argument 14 as `region`;
**its arguments 12 and 13 are not forwarded at all** (disassembly
`0x617013–0x617048`: the pushes are `[ebp+0x3c]`, `0`, `−1`, `[ebp+0x30]`,
`[ebp+0x2c]`, …). `Unit::fight@005fd4d0:273` and `Unit::come_out:957/1210`
pass real values in those two slots; they are dead.

No `Random::get` anywhere on the path — not in `find_nearby_spot`, `project`,
`sin_table`, `Objects::find_collision`, `find_ordered_collision`,
`ObjectsData::find_unit_with_radius`, `find_unit_ordered_with_radius`,
`CollCheck::collide_here` (grep of the six export files for `Random`: none).
The only state it writes besides `*out` is the search-result scratch
`ObjectsData::find_dist (+0x1fc)` / `find_who (+0x200)` inside the helper
searches.

### 1.2 The defaults (prologue, `0x61dea8–0x61df5d`)

```
if (min > 0 && max == 0) || max < 0:
    if uber: max = min + 0xc0 + 4 · ( ((uber_size − 1) · guy_spacing) / 2 + big_radius )
    else:    max = min + 4 · big_radius
             if big_radius == 0 && !(unit_flags & 0x10): max = min + 0x240      # three tiles
if max < min: max = min
if step <= 0: step = (max − min) / 8            # C division, truncating (cdq/and 7/add/sar 3)
step = max(step, 1)
r_coll = block_radius                           # +0x240, position units
if r_coll == 0 && filter == FILTER_ALL: r_coll = 0x180
if uber: r_coll += ((uber_size − 1) · guy_spacing) / 2 + 0x30
pairwise = !uber && (filter == FILTER_NOT_ME || filter == FILTER_CAN_COLLIDE) && o >= 0 && who >= 0
warship  = domain == 1 && (attack != 0 || T.is(AIRCRAFTCARRIER 0x15f, 1))        # Type vslot +0x60
farm_ok  = bo >= 0 && T.type ∈ {PEASANTS 0x32, PEASANTSKOREAN 0x33} && objects[bwho][bo].is(FARM 0x1a1, 0)
```

All `/ 2` are the signed truncating kind (`cdq; sub eax, edx; sar eax`). The
radii come from `UnitType::init@0061ab50:750–758`: `new_block_radius` is the
XML `BLOCK_RADIUS` (in UCoords), `guy_radius = block_radius =
UNIT_BLOCK_RADIUS (48) · new_block_radius`, `big_radius = ((squad_size − 1) ·
guy_spacing) / 2 + guy_radius`, `new_big_radius = big_radius / 48` (see
`docs/COMBAT.md` §? "`block_radius` is the type's `BLOCK_RADIUS ×
UNIT_BLOCK_RADIUS`", `docs/DATALAYER.md`). Note that `uber` uses `uber_size`
(+0x308) where `big_radius` was built from `squad_size` (+0x304).

Note `min == 0 && max == 0` is *not* the default case: `max` stays 0, one ring
of radius 0, one candidate — "is this exact point free", which is how
`action_swarm_around`'s `+0x30` nudge is re-validated.

### 1.3 The candidate sequence (`0x61e008–0x61e0a0`, tail `0x61e3ab–0x61e3dc`)

```
r = min;  k = 0
loop:
    if r != 0:
        a  = angle + k · 0x10000000 + (k < −7 || k > 7 ? 0x8000000 : 0)     # wraps mod 2^32
        (px, py) = project(x, y, r, a)         # = (x + sinx(a, r), y − cosx(a, r))
    else:
        (px, py) = (x, y)
    cx = div_3_table[px >> 4] · 0x30 + 0x18    # centre of the 48-unit quarter-tile containing p
    cy = div_3_table[py >> 4] · 0x30 + 0x18
    k' = (r == 0) ? 16 : (k <= 0 ? −k : −(k + 1))
    if candidate (cx, cy) passes §1.4:  *out = (cx, cy); return 0
    k = k' + 1
    if k > 15:                                 # ring exhausted
        if max <= r:  *out = (x, y); return 1
        r = min(r + step, max);  k = 0
```

The `k` bookkeeping (decompile `iVar8 = param_13 + 1; if (param_13 < 1)
iVar8 = param_13; param_13 = −iVar8;` then `param_13 + 1`, exit on `> 0xf`)
yields, per ring with `r > 0`, exactly **31 candidates**, in this order of `k`:

```
0, 1, −1, 2, −2, 3, −3, 4, −4, 5, −5, 6, −6, 7, −7, 8, −8, 9, −9, 10, −10, 11, −11, 12, −12, 13, −13, 14, −14, 15, −15
```

and the bearings they stand for (relative to `angle`, in turns): the fifteen
sixteenths `0, ±1/16, … ±7/16`, then `17/32` (k = 8), **`17/32` again** (k =
−8: `−8/16 + 1/32 ≡ 17/32`), `19/32, 15/32, 21/32, 13/32, 23/32, 11/32, 25/32,
9/32, 27/32, 7/32, 29/32, 5/32, 31/32, 3/32`. So a ring covers all sixteen
sixteenths **except the direct opposite (8/16)** and all odd thirty-seconds
**except 1/32**, with 17/32 tried twice. The half-step term is the disassembly
at `0x61e017–0x61e01e` (`lea eax,[ecx+7]; cmp edx(=14), eax; sbb eax,eax; and
eax, 0x8000000` — set exactly when `k + 7` is outside `0..14` unsigned) and
`0x61e02a–0x61e033` (`shl ecx, 0x1c; add ecx, eax; add ecx, [ebp+0x24]`). A
ring with `r == 0` has one candidate (k is set to 16). The ring at `r == max`
is searched (r is clamped to `max`, searched, then the `max <= r` test ends
the loop). The decompile's loop counter is the re-used `uber` stack slot, which
is why it prints as `param_13`.

`project@0092cf40` (`__fastcall`-style: `ecx` = angle, `edx` = distance, four
stack args `x, y, &ox, &oy`): `ox = x + sinx(a, d)`, `oy = y − cosx(a, d)`,
where the fold is byte-for-byte the one in `sinx@0092d100` (negative angle →
negate the distance and clear the sign; `& 0x40000000` set → `0x7fffffff − a`;
then `sin_table@00a46a00`), and the cosine leg is the same code on `a +
0x40000000` (`lea ecx,[edi+0x40000000]`, `0x92cf84`). These are the `sinx` /
`cosx` of `docs/MOVEMENT.md` ("The sine table"), so the sim's existing
implementation is the one to call; `+sin` on x and `−cos` on y is the
north-is-zero, y-grows-south convention of that document.

### 1.4 The validity test, per candidate (`0x61e0a0–0x61e3ab`)

In this order; the first failing step skips to the next candidate.

1. **On the map**: `0 ≤ cx < tile_xs · 0xc0` and `0 ≤ cy < tile_ys · 0xc0`
   (`WorldData::tile_xs/tile_ys` +0x18/+0x1c).
2. Tile `(tx, ty) = (div_3_table[cx >> 6], div_3_table[cy >> 6])`; cell
   `(tx >> 2, ty >> 2)`; `T = tdata[ty · tile_xs + tx].mask` (`World +0x138`,
   `ushort`), `W = wdata[cy_cell · xs + cx_cell]` (`World +0x134`, 28 bytes).
3. **Region** (`region ≥ 0` only): `v = (W.flags & 0x100) == 0 ? W.region :
   ((T & 0x30) == 0x20 ? W.region2 : W.region)` — on a coastal cell an ocean
   tile answers with the water region (`WData +0x4 region`, `+0x6 region2`);
   `v != region` → skip.
4. **Blocked tile**: unless `domain == 2` (air), `T & 0x4000` → skip. (`0x4000`
   is what a building's `mask_me` sets on its tiles, `docs/CITIES.md` §2.3/§3.6.)
5. **The target's footprint** (`bo ≥ 0` and `objects[bwho][bo]`'s vslot `+0x20`
   is non-zero — the slot that is `Window::get_button` = 0 on `Unit` and
   `Buffer::is_pending_load` = non-zero on `Build`/`BuildData`, i.e. "is a
   building"): take the building `B` from vslot `+0xac` (the same
   `get_button`/`get_graphicwinxml` pair: `Build` returns itself), its corner
   exactly as `WallData::tile_corner` (§3): `cxB = div_3_table[B.x >> 6] −
   (B.x_size >> 1)`, same for y; if `cxB ≤ tx < cxB + x_size && cyB ≤ ty < cyB
   + y_size` and `!farm_ok` → skip. So a builder never stands on its site, a
   gatherer never on its building, a garrisoner never on the fort — but a
   peasant may stand anywhere on a farm. (`T.type` is compared with `0x32/0x33`
   at `0x61dfb8`; the `is(FARM)` goes through the object's `ObjectData::is`
   devirtualised onto its type's `Type` vslot `+0x60`.)
6. **Terrain class**, by `T.domain` (+0x218): air (2) → pass; sea (1) → need
   `(T & 0x30) == 0x20` (ocean), and for a `warship` also `!(T & 0x2400)`
   (`0x2000` "bad path" and `0x400` — unnamed in `docs/CITIES.md` §2.3; river/
   shallows by position); land (0) → need `(T & 0x30) != 0x20`. **Nothing
   else**: a land unit's spot may be a forest (`0x30`), road, mountain or
   cliff tile; only ocean and `0x4000` are refused here. (`invalid_loc` is
   stricter, §2, and `find_path` later teleports a unit off a tile
   `invalid_loc` rejects — that is the `find_path@005fb910:64–69` call in the
   caller table.)
7. **Collision**: `nocoll != 0` → accept. Else
   - **pairwise path** (`pairwise`): accept iff
     `Objects::find_collision(cx, cy, o, who, 0) == 0` **and**
     `Objects::find_ordered_collision(cx, cy, o, who) == 0` (§1.5);
   - **general path** (any other filter, `o`/`who < 0`, or `uber`): accept iff
     `ObjectsData::find_unit_with_radius(cx, cy, …, −1, r_coll, …, filter, o,
     who) < 0` **and** (`who < 0` or
     `ObjectsData::find_unit_ordered_with_radius(cx, cy, who, r_coll, …,
     filter, o, who) < 0`) (§1.6).

### 1.5 The pairwise collision tests (`FILTER_NOT_ME`/`CAN_COLLIDE`, not `uber`)

Both take the candidate in position units and "me" as `(o, who)`; both read
**`new_block_radius`** (+0x248, UCoords) and work in UCoords (`div_3_table[v >>
4]`); both are Chebyshev (each axis separately `≤ r_me + r_other`).

**`Objects::find_collision@0065b1b0(x, y, o, who, 0)`** — if my type's
`domain == 0` (land): `return CollCheck::collide_here(o, who, ux = div3(x>>4),
uy = div3(y>>4), new_block_radius, 0, 0, 0)` — the quarter-tile occupancy grid
(`CollCheck`, `BitMask<768>` planes filled by `fill_slots`), the same primitive
`detect_unit_collision` and the pathfinder use (§4). Otherwise (sea/air, or the
last flag non-zero), and only if my `new_block_radius != 0`: the 9 cells
(`move_x/move_y[0..8]`) around `div3(x>>8), div3(y>>8)`; each cell's object
chain — head `WData +0x8 down / +0xa down_who`, links `ObjectData +0x2c down /
+0x2e down_who` — every object that is not me, owner `< 8`, `is_active`
(vslot +0x8) and `is_on_map` (+0xbc), with `units[who][o]->ptype->new_block_radius
!= 0`: collide iff `|ux − div3(obj.x >> 4)| ≤ r_me + r_obj` and the same on y
(the object's **current** position, masked).

**`Objects::find_ordered_collision@0065b440(x, y, o, who)`** — `0` if my
`new_block_radius == 0`; else the same 9-cell chain walk, but against each
other unit's **ordered position** `UnitData::orders_x/orders_y` (+0x70/+0x74,
not masked) — the spot it has been told to stand on; then, if I am in a
group (`UnitData::group +0x80 ≥ 0`, the group owned by `who`, the group's
`+0x49` byte clear), the same ordered-position test against every other
member (`groups[g] + 0x8cc` shorts, count `+0xc`) that is active, has
`inside_up (+0x82) < 0` and a non-zero `new_block_radius`. So a spot another
own unit is already walking to is taken.

A building's `new_block_radius` is `0` (`ObjectType::ObjectType@0065f880:31`
zeroes +0x248; no `BuildType` file writes it) so buildings never trip the
chain walk; what keeps units off them is step 4/5 above. (`units[who][o]` and
`objects[who][o]` are parallel `PtrArray<Unit>[10]` / `PtrArray<Object>[10]`
tables — `UnitsData::lists +0x0`, `ObjectsData::lists +0x4`, each `list` at
`+0x10` of the array — indexed by the same `(o, who)`; what a building's slot
holds in `units` is not established, see §6.)

### 1.6 The general collision tests (`uber`, `FILTER_ALL`, no "me")

**`ObjectsData::find_unit_with_radius@00659890(x, y, search, −1, r, ?, filter,
o, who)`** returns the index of a unit (≥ 0) or −1. With `r ≥ 0` and a small
enough circle (`circle_radius[(r + 0x2ff) / 0x300] ≤ Game::total_units`) it
walks the cells of a precomputed circle (`circle_x/circle_y`) outward from
the candidate's cell and each cell's object chain; otherwise every player's
whole unit list. For each object with owner `< 8`, `Search::valid_search`,
`is_active`, `is_on_map` and passing the filter (`Search::valid_filter@0067dbb0`
dispatches on `filter − 1`: **`FILTER_NOT_ME` = "not `(o, who)`"**
(`0x67dc27`), **`FILTER_CAN_COLLIDE`** = not me, both `ObjectData::is_map_unit`,
not the same group (`+0x80`, unless −1), **same `domain`** (`0x67dc9c–0x67dd51`);
`FILTER_ALL` passes everything): `d = vector_dist(candidate − object position)`
(the two operands are register-passed and lost in the decompile; by the
sibling functions they are the dx, dy to the object's masked position); if
`d ≤ obj.big_radius` (+0x244) → return it at once; else if `d − obj.big_radius
≤ r` (and ≤ the best so far) → remember it. So **the candidate is rejected iff
some other unit lies within `obj.big_radius + r_coll` of it** (vector distance,
`docs/COMBAT.md`'s `vector_dist`). The iteration order only decides *which*
index comes back, which `find_nearby_spot` ignores.

**`ObjectsData::find_unit_ordered_with_radius@00658ef0(x, y, who, r, ?, filter,
o, who, …)`** — the same distance test, `big_radius + r`, over **my own
player's** unit list only (`who < 8`, else an `Error::report "Not passing valid
who."`), restricted to units whose current order is `MOVE_TO, ATTACK_TO,
EXPLORE_TO, FLEE_TO, CHANGE_FORM, GROUP_MOVE, GROUP_ATTACK_TO`
(`UnitData::order_type`), against — by its name and its sibling — their
ordered position (`orders_x/y`; the `vector_dist` operands are again lost).

### 1.7 Who calls it, with what (grep of the export; the Unit wrapper inlined)

Argument columns are `(min, max, step, angle, filter, o/who, nocoll, uber,
bo/bwho, region)`; "me" = the moving unit; "site" = the target object.

| caller | centre | min | max | step | angle | notes |
| --- | --- | --- | --- | --- | --- | --- |
| `Group::action_swarm_around@0070fbe0:312` (build/repair/gather approach, R3 §2.3) | site | `R = min(x_size, y_size) · 0x60 + 0x30` (halved for a FARM under BUILD_AT; a dead site uses the unit type's `big_radius`) | `0` → default `R + 4·big_radius` | `−1` → `/8` | **`find_angle(me.x − site.x, me.y − site.y)`** (asm `0x7101e8–0x71021c`: the member at `[ebp−0x1c]`, minus the site's `ebx/esi`) | `NOT_ME` me; `nocoll 0, uber 0`; **`bo/bwho = the site`** (footprint refused); region −1 |
| same, `:334` — the `+0x30` nudge | spot ± 0x30 per axis away from the site | 0 | 0 | 0 | same | one candidate: "is the nudged point free"; same `bo/bwho` |
| `Unit::do_garrison@005e6b80:184/192` (§CITIES 6.4) | target | `size · 0x60 + 0x30`, or `size · 0x60 + 0x1b0` when the target `is(…)` some `TypeIndex` (the usual `ObjectData::is` devirtualisation, vslot `+0xb8` / `Type +0x60`; the index is register-passed and lost — not read) | `−1` → default, or `size · 0x60 + 0x330` in that case | 0 | `find_angle(me − target)` (asm `0x5e6f15–0x5e6f35`) | first try `nocoll 0`, then **`nocoll 1`** (`:192`); `bo/bwho −1` |
| `Unit::do_gather@005ef2a0:217` (R4 §3.2) | building | `(flat ? x_size : y_size) · 0x60 + 0x30` | `−1` | 0 | `find_angle(me − b)` (asm `0x5ef80c–0x5ef845`) | `NOT_ME` me; all trailing −1/0 |
| `Unit::do_non_flat_gather@005f0170:123/465/485` (R4 §3.3) | building | `local_50`/`local_3c` (R4), `0x600` | `−1` | 0 | `find_angle(…)` | as above |
| same `:277` — the woodcutter's tile | tile centre | `0xc0` | `0x100` | `2` | `find_angle(…)` | rings 0xc0, 0xc2, … 0x100 — 33 rings × 31 |
| `Unit::come_out@00617c10:195/202` (R3 §2.10, R5) | own position | `block_radius` | `block_radius + UNIT_DISEMBARK_DISTANCE` | 0 | **`0x80000000`** (south) | `nocoll 0` then `1`; further variants at `:372–:478` with `FILTER_ALL`, `uber 1`, `×2` ranges |
| `Unit::find_path@005fb910:69` — teleport off a tile `invalid_loc(…,1,0,0,0,0)` rejects (R2 §3.1) | own position | 0 | `−1` → `4·big_radius` | `/8` | `0x55555555` | `NOT_ME` me |
| `Unit::go_to@005f7a50:15` | given | given | given | 0 | `0x55555555` | **`uber 1`** |
| `Objects::init_unit@0065e0c0:120/198` (spawn) | given / a unit's position | `r · 0x30` | `r · 0x60 + 0xc0` | −1 | **the unit's own `UnitData::angle` (+0x50)** | `NOT_ME` |
| `Unit::fight@005fd4d0:273` | midpoint of me and target | 0 | `0x180` | 0 | `0x55555555` | via the wrapper: `nocoll 0, uber 0` |
| `Unit::do_attack_ground:126` | projected | given | −1 | 0 | `angle − 0x10000000` | |
| `Unit::find_attack_pos:362/904`, `target_opportunity`, `do_guard`, `do_follow`, `do_trade`, `do_cast`, `Group::action_move_near`, `action_transport`, `action_spell`, spells, `Herd`, cheats, console, editor | various | | | | mostly `0x55555555`; `do_follow:175` and `init_unit` use a unit's `angle`; `cast_create_decoy` uses `0` | `find_attack_pos:362` is the one caller passing `region` and a non-`NOT_ME` filter |

`0x55555555` is a third of a turn (120°) — an arbitrary fixed base, not a
sentinel: nothing in the body tests for it (no `55555555` in the
disassembly).

---

## 2. `UnitData::invalid_loc@00607c30(tx, ty, a, b, c, d, e)`

A **tile** predicate (`TCoord` arguments — callers pass `div_3_table[pos >> 6]`),
`this` = the unit asking. Seven stack arguments plus a junk eighth the
decompiler invents from a register (`ret` size not checked; every caller
passes seven meaningful values). Return **0 = valid**, non-zero = invalid,
with a code: **1** off the map, **2** terrain, **3** a warship on a
`0x2400` tile, **4** a `0x4000` (structure-blocked) tile. `find_nearby_spot`
never calls it; `find_path`, `move_step`, `do_move`, `go_around_building`,
`resolve_unit_collision`, the pathfinder's `valid_tcoord/ucoord/wcoord`,
`astar_path`, `find_wpath`, `get_final_loc`, `find_attack_pos`,
`find_open_slots`, `do_guard`, `check_meet_ship`, `think_scout`,
`Group::action_move_near` and the editor do (full list: the `r7_calls2.py`
grep, 60 sites).

What it reads: `UnitData::path` (the waypoint stack: if the top node's `flags
& 4` is set, `d` is forced to 1 — R2 §1.4 names that flag), the unit's
`ptype` (`domain`, `attack`, `Type::is(AIRCRAFTCARRIER)` through the
`ObjectData::is` devirtualisation), `unit_masks` / `unit_masks2`
(`UnitData`), its own position (masked), the owner's `LeaderData::leader_flags
& 4` (human, `docs/CITIES.md` §2.4), `WorldData::was_seen` on half-cells,
`tdata[].mask`, `wdata[].flags`, `WorldData::is_cliff_at`, `is_built_at`,
`ObjectsData::find_any_building_at` (+ its `find_who` result), and
`UnitData::can_transport`.

The decision, in order (`0x607c30–0x608030`):

```
if path.length && path.top.flags & 4: d = 1
if tx < 0 || ty < 0 || tx >= tile_xs || ty >= tile_ys: return 1
cell = (tx >> 2, ty >> 2); T = tdata[ty·tile_xs + tx].mask; W = wdata[cell].flags
if b && leaders[who].leader_flags & 4:                        # a human's unit respects the fog
    if none of the four half-cells (2cx, 2cy), (2cx+1, 2cy), (2cx, 2cy+1), (2cx+1, 2cy+1) was_seen by who:
        return 0                                               # unseen is presumed passable
match domain:
  sea (1):
    if (T & 0x30) == 0x20:                                     # ocean tile
        if attack != 0 || is(AIRCRAFTCARRIER):                 # a warship
            if T & 0x2400: return 3
        # (an unarmed, non-carrier ship skips straight to the 0x4000 test)
    else:                                                      # a land tile
        if d == 0 && e == 0: return 2
        if !can_transport(this): return 2                      # only a transport may touch land
        if W & 0x70: return 2                                  # rock / mountain cells
  land (0):
    if (W & 0x70) && !((W & 0x20) && unit_masks2 & 0x4000) && a && d: return 2
    cls = T & 0x30
    if cls == 0x30 (forest) || (T & 3) == 2 (mountain) || is_cliff_at(tx, ty):
        if cls != 0x30: return 2                               # mountain, cliff: never
        if !(unit_masks2 & 0x4000): return 2                   # forest: only for that mask
    if ((d == 0 && e == 0) || !(unit_masks & 0x800000)) && cls == 0x20: return 2   # ocean
  air (2): return 0
# LAB_00607f50 — the structure test
if a == 0 && (T & 0x4000) && !(cls == 0x30 && unit_masks2 & 0x4000)
   && !(tdata[tile of my own position].mask & 0x4000):         # already inside one: may step to another
    if attack != 0 && c && is_built_at(tx, ty):
        find_any_building_at(tx, ty, who, FILTER_ALL, 0)
        return (objects.find_who == who) ? 4 : 0               # an enemy structure: attackable, "valid"
    return 4
return 0
```

The five flags, by effect (the original's names are not in the symbols):
`a` — skip the `0x4000` structure test (and, with `d`, make rock/mountain
cells invalid): the "my own tile" checks pass it (`find_path:64 (1,0,0,0,0)`,
`do_move:595 (1,0,0,0,1)`, `get_final_loc`, `valid_wcoord`);
`b` — fog-respecting for humans (the pathfinder: `astar_path`, `find_wpath`,
`valid_tcoord`, `valid_ucoord` pass `(0,1,1,1,0)` / `(0,1,0,1,0)`);
`c` — an enemy structure's tile counts as valid for an armed unit (pathfinder);
`d` — the pathfinder's mode: lets a transport's path cross land tiles, lets a
`unit_masks & 0x800000` land unit cross ocean, forced by the path top's flag 4;
`e` — the same two gates as `d` without the rock/mountain cell rule
(`do_move:595`, `find_attack_pos:620`, `find_open_slots:51`).

**On the harness's open flat map** a land unit with all flags zero gets: `1`
off the map; `2` on an ocean tile, a forest tile (unless `unit_masks2 &
0x4000`), a mountain tile or a cliff; `4` on a structure's `0x4000` tile
(unless it is already standing on one); **0 everywhere else** — plain land,
road, territory of any owner. Rock/mountain `WData.flags` cells only matter
with `a && d`. So "always 0 away from water, forest, mountains and buildings"
is right.

---

## 3. `Object::adjacent_to@00651f40(o, who)`, `WallData::covers_tile@006439b0`, `tile_corner@00643440`

**`adjacent_to`** is the virtual `Object` slot `+0x170` that `do_build:102`,
`do_garrison:150`, `do_gather:194`, `do_non_flat_gather` call **on the moving
unit** with the target's `(o, who)`; `Object::adjacent_to` is its only
implementation (no other file mentions the name). Read in full:

```
if !(this.flags & 1): return 0                     # SubObjectData +0x8 bit 0: I am active
if !(objects[who][o].flags & 1): return 0          # the target is active
if this.ptype.domain == 1 && this.is_unit():       # vslot +0x18: Unit non-zero, Build zero
    me = units[this.who][this.o].ptype             # my own UnitType again, via the unit table
    if !(me.unit_flags & 0x10):
        limit = constants.boat_garrison_max_distance
        if me.new_block_radius < 4: limit += 0x180
        return attack_dist(o, who) < limit
return attack_dist(o, who, my x, my y) < 0x60
```

So R3's `attack_dist < 0x60` (`docs/COMBAT.md` §13.1: the footprint-/radius-
aware distance — a point on the target's extent is at 0) is the rule for
every land and air unit and for a sea unit flagged `unit_flags & 0x10`; a
**sea-domain unit** (a ship adjacent to a dock, a transport to its passenger,
a ship to whatever it is garrisoning/gathering at) uses
`BOAT_GARRISON_MAX_DISTANCE`, plus one and a half tiles when its own
`new_block_radius < 4`. Which `attack_dist` overload is called differs between
the two branches (two- and four-argument); both measure from the caller's own
position. There is no transport-*passenger* exception here: the exception
keys on the caller's domain, not the target's.

**`covers_tile(tx, ty)`**: `tile_corner(&cx, &cy)`; `cx ≤ tx < cx + x_size &&
cy ≤ ty < cy + y_size` → 1, else 0 (`BuildTypeData +0x234/+0x238` through
`ptype`). Exactly `docs/CITIES.md` §2.2's footprint, which `place.rs::footprint`
already enumerates.

**`tile_corner(&cx, &cy)`**: `cx = div_3_table[(x ^ 0x63637) >> 6] − (x_size >>
1)`, same for y (the `+0x60` for an odd size is applied and then divided back
out — `div_3_table[(tile·0xc0 + 0x60) >> 6] == tile` — so it is a no-op for a
tile-aligned centre; `docs/CITIES.md` §2.2 and `place.rs::tile_corner` have
it). `find_nearby_spot` §1.4 step 5 inlines the same arithmetic.

---

## 4. `Unit::detect_unit_collision@00617060(x, y, a, b, c, d, e)` — the interface

`__thiscall` on the moving unit, two `Coord` (position units) and five `int`
flags; returns **0 (no collision) or 1** (`return 1` at `:96, :381, :397, :427`;
`return 0` otherwise). R2 §3.3 has the callers (`do_move` every other frame on
`coll_x/y`, `move_step` per step, `resolve_unit_collision`, `Animal::do_idle`,
`find_merchant_spot`, `PathFinder::valid_ucoord`); the argument patterns seen
are `(1,1,0,0,0)` (the probes), `(0,1,0,0,0)` and `(0,0,0,0,0)` (`move_step`,
`do_move:507`), `(1,1,0,0,1)` (`do_move:145`), `(1,1,?,1,0)` (`valid_ucoord`).

What it reads and tests, in order (internals only as far as the brief asks):

- `ptype.domain` (air: the answer depends only on `a`), the type's `Type`
  vslot `+0x10c` (a type predicate, unread), `UnitData::is_hero` (`unit_flags2
  & 0x20`) / `is_supply` (`& 0x40`): for sea units, heroes, supply wagons and
  that predicate's types with `d == 0` there are early `return 0`s gated on
  `a`, `b` and `detect_boat_collision(x, y, 1)` (`:56–78`).
- `UnitData +0xc0` and `safe (+0xb2)` (`== 0` required to test at all).
- The target quarter-tile `(div3(x>>4), div3(y>>4))`; if it differs from the
  unit's current one: **`CollCheck::collide_here(my o, who, ux, uy,
  new_block_radius, &hit_ux, &hit_uy, d)`** — the occupancy-bitmask test in
  UCoords, radius **`new_block_radius` (+0x248)**. No hit → continue to the
  `0` return. A hit with `a != 0` → **return 1** (`:96`). A hit with `a == 0`
  → a 9-cell scan (`move_x/move_y` around `div3(x>>8), div3(y>>8)`,
  `WorldData::get_down` per cell) for the other **unit** (`is_unit`, domain ≠
  air, `is_on_map`, `UnitData::is_here(hit)`), then order-aware exceptions:
  a `TRADE_ROUTE` caravan vs. an order-`0xf` unit, an order-`0xc` unit whose
  target is me, an `ATTACK` (my target in range — `max_range` vslot `+0x130`,
  `attack_dist`), `is_moving` on either side, same-owner `new_block_radius == 1`
  pairs — deciding `local_8` (1 = collision).

For the harness: the only geometry is **my `new_block_radius`** (in
quarter-tiles; `BLOCK_RADIUS` from the XML, typically 1) against the
`CollCheck` planes that `fill_slots` populates from the units' occupancy —
whether buildings are in those planes is not established (§6); buildings are
otherwise kept off by `invalid_loc`'s `0x4000`. Two units register a
collision only when their quarter-tile discs overlap, i.e. centres within
`r_me + r_other` UCoords (≈ 96 units for two radius-1 types). A handful of
well-spaced units on open ground never trip it; the only way they do is by
walking to the same spot, which `find_ordered_collision` (§1.5) is there to
prevent.

---

## 5. The seam into `crates/sim`

What exists (this worktree):

- `world.rs`: `Pos` (position units, `tile()`, `cell()`), `World::tile_mask`
  with the `tile::` bit names (`SURFACE 0x30`, `SURFACE_OCEAN 0x20`,
  `SURFACE_FOREST 0x30`, `BLOCKED 0x4000`, `BAD_PATH 0x2000`, `OBJECT_MOUNTAIN
  2`, …), regions per cell (`region_of`), `vector_dist`, `tile_in_bounds`,
  `UNITS_PER_TILE 192`. Missing: `WData.flags` bits (`0x100` coastal, `0x70`
  rock/mountain), `region2`, the per-cell object chain.
- `place.rs`: `snap_center`, `tile_corner`, `footprint`, `blocked_tcoord` and
  the `Blocked` verdicts; `fight.rs`: `attack_dist_from`/`attack_dist`
  (`combat::attack_dist`, `combat::snap(v) = div3(v>>4)·0x30`, `extent`);
  `combat.rs` `Profile { block_radius, big_radius, … }` (position units);
  `movement.rs`: `find_angle`, `sin_component`/`cos_component` (the `sinx`/
  `cosx` of MOVEMENT.md), `Angle`; `lib.rs` `Movement::orders_pos`
  (`orders_x/y`), `Unit`, `Job`.
- The tuning constants `UNIT_BLOCK_RADIUS` (48), `UNIT_DISEMBARK_DISTANCE`,
  `BOAT_GARRISON_MAX_DISTANCE` are `Tuning::RON` slots (`docs/DATALAYER.md`).

What the implementation needs, and the shape that follows from the reading:

1. **`find_nearby_spot`** as a free function over the sim's `World` plus a
   unit/building index: `fn find_nearby_spot(ty: &UnitType, centre: Pos, min,
   max, step, angle: Angle, filter, me: Option<Obj>, nocoll, uber, footprint_of:
   Option<Obj>, region: Option<u16>) -> Result<Pos, Pos>` with **exactly** §1.2's
   defaults, §1.3's ring/bearing sequence (31 per ring, `k` order, the half-step
   term, `project = (x + sin_component(a, r), y − cos_component(a, r))`,
   quarter-tile snap `+0x18`), §1.4's tests in order, and the two collision
   paths. The per-type inputs are `domain`, `attack`, `is(AIRCRAFTCARRIER)`,
   `TypeIndex ∈ {PEASANTS, PEASANTSKOREAN}`, `unit_flags & 0x10`,
   `block_radius`, `big_radius`, `new_block_radius`, `guy_spacing`,
   `uber_size`. The type-level `Kind`/`Profile` in `combat.rs` already carries
   the two radii; `new_block_radius = block_radius / 48` and `guy_spacing`,
   `uber_size` need adding (all in `rondata`'s unit loader).
2. **The collision half** needs (a) a per-quarter-tile occupancy of units
   (`CollCheck`) with `new_block_radius` discs — the same structure
   `detect_unit_collision` and movement need, so it belongs to `movement.rs`'s
   future collision work (R2 §3.3), and (b) an "ordered position" per unit —
   `Movement::orders_pos` exists — with the 9-cell chain walk replaced by any
   spatial query that returns the same set (the set, not the order, decides
   accept/reject). For the **general path** the test is `vector_dist(candidate −
   unit) ≤ unit.big_radius + r_coll` over all units (filtered) and the ordered
   variant over own units with a transit order. Until collision exists,
   `nocoll`-style acceptance is the stated assumption; on the harness's map
   (no blockers) it gives the identical spot, because the first candidate
   (`k = 0` on ring `min`, on the unit's side of the target) is free.
3. **`invalid_loc`** (§2) as `fn invalid_loc(u: &Unit, t: Pos, a, b, c, d, e)
   -> u8` on `World::tile_mask` + a `WData.flags` rock/mountain layer + cliffs;
   the fog branch is a no-op without fog (as `fight.rs::is_in_range` already
   states).
4. **`adjacent_to`** is `fight.rs::attack_dist(me, target) < 0x60` with the
   sea-caller exception (needs `BOAT_GARRISON_MAX_DISTANCE`, `unit_flags & 0x10`,
   `new_block_radius`); **`covers_tile`** is `place.rs::footprint(...).contains`
   (or a corner-and-size test). Both exist in substance.
5. The angle every order feeds it: `find_angle(me.pos − target.pos)` — the sim's
   `movement::find_angle(dx, dy)` with `dx = me.x − target.x`, `dy = me.y −
   target.y` (asm-confirmed at the swarm, garrison and gather sites).

Nothing here is logged by name: the gamelog shows the *result* — `orders_x/y`
and the unit's position under `UNITS=3` (R2 §5) — which is what the harness
diffs; a wrong candidate order shows up as a unit standing on the wrong side
of its site.

---

## 6. Confidence, what is not established, behavioural checks

**High** (decompile + disassembly agree): the 16-argument signature and the
`Unit` wrapper's dropped arguments; the defaults of §1.2; the ring sequence and
the 31-bearing order of §1.3 including the half-step term, the duplicate
17/32 and the two never-tried bearings; `project`'s fold and sign convention;
the quarter-tile snap `+0x18`; the order and content of the per-candidate
tests (bounds, region/region2, `0x4000`, footprint-except-farm-for-peasants,
terrain class by domain with the warship `0x2400` rule); the two collision
paths' selection (`pairwise` vs general); the return convention and the
out-params on failure; no RNG; `find_angle(me − target)` as the base bearing
at the swarm, garrison and gather call sites; `adjacent_to` and `covers_tile`
in full; `invalid_loc`'s gates and return codes; `FILTER_NOT_ME` and
`FILTER_CAN_COLLIDE` semantics (jump-table targets read from the PE).

**Medium**: `find_unit_with_radius` / `find_unit_ordered_with_radius`'s
distance operands (`vector_dist`'s register arguments are dropped by the
decompiler; "candidate minus the unit's current / ordered position" is from
the siblings and the names); `detect_unit_collision`'s early returns for sea
units/heroes/supply (read once, not re-derived); what `WData.flags & 0x70`
bits other than `0x8` rock and `0x10` mountain are; the meaning of
`unit_masks2 & 0x4000` ("may enter forest", by effect) and `unit_masks &
0x800000` ("may cross ocean when pathing", by effect).

**Not established**: the original's names for `invalid_loc`'s five flags and
for `find_nearby_spot`'s arguments 12–16 (named here by effect); `Object`
vslot `+0x20` (read as "is a building" from the two vtables' folded bodies) and
`+0xac` ("the building itself"); `UnitType`'s `Type` vslot `+0x10c` and the
`TypeIndex` `do_garrison` tests with `is(…)` to pick the wider ring; whether a building's
id slot in `units[who][o]` is its own pointer (the `find_collision` chain walk
reads `new_block_radius` through it; a building's is 0 either way);
`CollCheck::fill_slots` — whether buildings occupy the occupancy planes;
`WorldData::is_cliff_at`, `can_transport`, `was_seen`, `Search::valid_search`,
the `circle_x/y/radius` tables (order-only, irrelevant to accept/reject);
`TData 0x400`. And a correction for `docs/ATTRITION.md` (its table calls
`WData down/down_who` "claim strength and its claimant" and its open questions
say they "may belong to a different system"): they are the **head of the
per-cell object chain** — `find_collision`, `find_ordered_collision`,
`find_unit_with_radius` and `detect_unit_collision` all walk `WData +0x8/+0xa`
→ `ObjectData +0x2c/+0x2e` — a spatial index, not territory.

**Behavioural checks** (each one logged run; the recipe is `docs/ORACLE.md`):

1. *The candidate order.* A citizen ordered to build a 2×2 site from due
   east with the east-side ring spot blocked by another unit standing on it
   (`cheat select` + right-click): with `UNITS=3` the builder's `orders_x/y`
   should be the `k = 1` or `k = −1` candidate (north-east or south-east of the
   site on the ring `R = 2·0x60 + 0x30`), not the west side. Distinguishes the
   ±1/16 alternation from a plain clockwise sweep.
2. *The missing opposite and the duplicate 17/32.* Only reachable by blocking
   15 of a ring's bearings — an editor scenario, low value; the reading is
   already settled in the disassembly.
3. *`find_unit_with_radius`'s distance.* Two own units, one standing; send
   the other with `go_to` (`uber 1`, the general path) to a point `d` from the
   first: the spot is refused until `d > standing.big_radius + mover.block_radius
   + (uber_size − 1)·guy_spacing/2 + 0x30`; a single-type test pins the operand
   question.
4. *`invalid_loc` on forest.* A citizen told to walk onto a forest tile: with
   `unit_masks2 & 0x4000` clear the walk ends on the tile's edge
   (`find_path`'s pull-back, R2 §3.1) rather than on it.

All four are 20–40-minute drives; none blocks the implementation, whose
harness case (open ground, first candidate free) is decided by §1.2–§1.4 alone.

