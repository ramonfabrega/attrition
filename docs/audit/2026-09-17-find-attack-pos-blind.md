# Blind reading: Unit::find_attack_pos@00601280 (2026-09-17, Opus 5)

Blind second reading. Sources: the Ghidra decompile export
(`~/ghidra-projects/decomp/funcs/Unit/find_attack_pos@00601280.c`,
`types.txt`, `vtables.txt`) and `llvm-objdump -d` over
`riseofnations.exe` (image base 0x400000; every address below is a VA).
I did not open `docs/COMBAT.md`, `docs/ARMY.md`, `docs/QUEUE.md`,
`crates/sim/src/combat*`, `crates/sim/src/army.rs`, or any journal file.

Confidence tags used per claim: **[bytes]** = read off the
disassembly; **[decomp]** = read off Ghidra's C and believed;
**[infer]** = plausible but unconfirmed.

---

## 0. Ground facts the rest of the report leans on

| fact | evidence |
|---|---|
| World position unit: **1 tile = 0xC0 = 192**. Tile index of a position `p` is `T[p >> 6]` where `T[i] = i/3`. | `602045 movl 0xcae5fc,%edx` then `60206b pushl (%ecx,%edx)` with `ecx = (x>>6)*4` **[bytes]** |
| Candidate positions are snapped to a **0x30 = 48** (quarter-tile) grid with centres at `48k + 24`. | `602051 movl (%edx,%esi,4),%eax` / `602054 leal (%eax,%eax,2),%esi` / `60205a shll $0x4,%esi` / `60205d addl $0x18,%esi` → `T[x>>4]*48 + 24` **[bytes]** |
| Every stored coordinate is XOR-masked with **0x63637**. `SubObjectData +0x10 x_internal`, `+0x14 y_internal`. | `types.txt` `struct SubObjectData`; `601517 xorl $0x63637` and ~40 more sites **[bytes]** |
| `ObjectType` (= `this->field_0x18`, `SubObjectData::ptype`) fields used here: `+0x1f8 min_range`, `+0x1fc max_range`, `+0x218 domain`, `+0x234 x_size`, `+0x238 y_size`, `+0x244 big_radius`. | `types.txt` `struct ObjectTypeData` (size 0x2b8) **[decomp]** |
| Object vtable slots used: `+0x18 is_unit` (1 on Unit/Animal, 0 on Build/Wall/Object), `+0x1c is_build` (1 on Build/Wall), `+0x20` (1 on Build only, 0 on Wall), `+0xac get_build` (returns `this` on Build, NULL elsewhere), `+0xbc is_on_map`, `+0x12c min_range`, `+0x130 max_range`, `+0x17c get_speed`, `+0xd8 is_moving`. | `vtables.txt`; the folded bodies are `Window::get_button@0041bff0 { return 0; }`, `Buffer::is_pending_load@0041e0e0 { return 1; }`, `GraphicWinXML::get_graphicwinxml@0041c000 { return this; }` **[bytes/decomp]**, names for 0x18/0x1c/0x20/0xac are **[infer]** from which classes answer 1 |
| The per-player object array is `*(void**)(0xC0AEC0 + who*0x1C)`, indexed `[o]`. `0xC0618C` holds `GameAccess::objects`; `0xC06188` holds `GameAccess::world`; `0xC06184` holds `GameAccess::game_random`. | `6012d8`, `601375`, `6020a9`, `602107` **[bytes]** |

Helper functions, re-derived:

- `vector_dist@0046CFF0(__fastcall ecx=dx, edx=dy)` — integer octagonal
  distance: with `a=|dx|`, `b=|dy|`, `hi=max`, `lo=min`, returns
  `hi + lo²/(2·hi)` when `lo ≤ 59999`, else `(lo + 2·hi)/2`. Returns 0
  when `hi == 0`. No floats. **[decomp]**
- `find_angle@0092D130(__fastcall ecx=dx, edx=dy)` — binary angle,
  2³² = one turn, **0 = −Y ("north"), 0x40000000 = +X ("east")**,
  clockwise. **[decomp]**
- `sin_table@00A46A00(__fastcall ecx=folded angle, edx=radius)` —
  256-entry quarter-turn table (`sine_table`), linear interpolation on
  the low 22 bits, amplitude 0xFFFF, returns `sin·r >> 16`. **[decomp]**
- `flanking@0092CFE0(__fastcall ecx=angle delta)` — returns 0 when the
  unsigned delta is above 0xD5555555 (i.e. within ~60° behind), else 1
  or 2. **[decomp]**

Enum values used below, all read from the PDB type stream
(`llvm-pdbutil dump --types rise.pdb`) rather than guessed:

- `OrderIndex`: `ATTACK = 10`, `GUARD = 12`. (Ghidra's `FILTER_*`
  substitutions in this function are mostly spurious — the values are
  plain integers.)
- `FilterIndex`: `FILTER_NOT_ME = 3`, `FILTER_CAN_COLLIDE = 5`.
- `role` bit `0x400` = **`ROLE_RANGED`**.
- `unit_masks` bit `0x40000` = `UNIT_AI_CONTROL`; bit `0x800000` =
  `UNIT_CAN_TRANSPORT`. `unit_masks2` bit `0x20000` =
  `UNIT2_MARINE_ENTRENCH`.
- `domain`: `0 = LAND, 1 = SEA, 2 = AIR` (from
  `UnitData::is_plane@0046CE40` → `domain == 2`;
  `UnitData::in_supply`/`can_ever_transport` → `domain == 0`).
- `TData::mask` bits: `0x4000 = MASK_BLOCKED`; the two-bit field
  `0x30` takes the value `0x20 = MASK_WATER`.

Virtual slot names come from the PDB field lists' `vftable offset`
(`SubObjectData`: `+0x18 is_unit`, `+0x1c is_wallbuild`, `+0x20 is_build`;
`SubObject`: `+0xa8 get_unit`, `+0xac get_build`; `ObjectData`:
`+0xbc is_on_map`, `+0x12c min_range`, `+0x130 max_range`, `+0xd8 is_moving`;
`ObjectTypeData`: `+0x10c is_siege`), **not** from `vtables.txt`, whose
answers there are COMDAT-folded aliases. **[bytes]**

---

## 1. Signature and inputs

Mangled name, from the SEH handler thunk pushed at `601285`
(`__ehhandler__find_attack_pos_Unit__QAEHHHHPAVCoord__0V2_1H_Z`):

```
?find_attack_pos@Unit@@QAEHHHHPAVCoord@@0V2@1H@Z
  → public: int __thiscall Unit::find_attack_pos(
        int, int, int, Coord *, Coord *, Coord, Coord, int)
```

Eight stack arguments, `retl $0x20` at `6012fb`/`601625`. **[bytes]**

| arg | ebp | meaning | evidence |
|---|---|---|---|
| `this` | ecx | the attacker | |
| 1 `o` | +0x08 | target's **object index** | `6015da movl (%eax,%esi,4),%ecx` with `esi = [ebp+8]` |
| 2 `who` | +0x0C | target's **player** | `6015d6 movl 0x14(%ebx,%eax),%eax`, `ebx = [ebp+0xC]*0x1c` (`601510 leal (,%ebx,8); subl %ebx; shll $2`) |
| 3 `melee_filter` | +0x10 | 0 → `FILTER_NOT_ME`, non-0 → `FILTER_CAN_COLLIDE` | `6015c1..6015ce: lea 0x3(,(p3!=0)*2)` |
| 4 `out_x` | +0x14 | **out** | `6015fa movl %ecx,(%eax)` |
| 5 `out_y` | +0x18 | **out** | `601602` |
| 6 `from_x` | +0x1C | the position the attacker is coming **from** | see below |
| 7 `from_y` | +0x20 | ″ | |
| 8 `pass_through` | +0x24 | forwarded verbatim as `find_nearby_spot`'s `param_12` | `602b10 pushl 0x24(%ebp)` |

`from_x/from_y` are **not** the target's position. The seven-argument
overload `Unit::find_attack_pos@00602E60` supplies them as *the
attacker's own* coordinates (`602e66..602e78`: `this->x ^ 0x63637`,
`this->y ^ 0x63637`), and the scoring at `602107` measures each candidate's
distance *from them*. **[bytes]** Callers may pass something else — a
group rally point — which is exactly why the parameter exists.

Both out-params are written on every non-exceptional return path, including
most `return 0` paths (§6).

State read from `this`: `inside_down`(+0x28), `inside_down_who`(+0x3e),
`who`(+0x9), `o`(+0xa), `x_internal`(+0x10), `y_internal`(+0x14),
`ptype`(+0x18), `angle`(+0x50), `unit_masks`(+0x68), `unit_masks2`(+0x6c),
`inside_up`(+0x82), the order-list cursor (+0xcc/+0xd0/+0xd4/+0xdc), and,
through `ptype`, `min_range`, `max_range`, `domain`, `big_radius`, `role`.

**Side effect on `this`**: if `unit_masks & UNIT_CAN_TRANSPORT` is set on
entry it is **cleared for the duration of the search** (`6014f2..601505`)
and re-set on every exit (`60160a`, `6027a8`, `602aa8`, `602b3f`, `602d65`,
`602d77`). `UnitData::invalid_loc` reads that bit (`invalid_loc@00607C30`
line 138), so the clear changes which candidates are legal. This is real
mutation of simulation state inside what reads like a query. **[bytes]**

Second side effect: when the current order is `ATTACK`, the branch at
`6015f4`/`601937` can zero the order-data byte at `+0x1c`
(`Unit::update_order(this)->vfn(0x50)()[0x1c] = 0`). **[decomp]**

**Falsifier for §1**: `UnitsSync` on any frame where a unit issues an
attack. `UnitData::log_data@00646xxx` prints `orders_x`, `orders_y`,
`angle` and `tolerance`; `ObjectData::log_data@00645FD0` prints
`inside_down` and `inside_down_who`. If `out_x/out_y` do not become the
unit's `orders_x/orders_y` on that frame, the "these are the movement
destination" reading is wrong.

---

## 2. Early control flow, in order

### 2.1 Transport delegation — `6012A1`

```
if ((short)this->inside_down >= 0  &&  this->ptype->domain == SEA)
    return find_attack_pos(objects[this->inside_down_who][this->inside_down],
                           o, who, p3, out_x, out_y, from_x, from_y, p8);
```
`6012a1 movzwl 0x28(%edi),%ecx` / `6012a8 js` / `6012ad cmpl $1,0x218(%eax)`;
the tail call at `6012e8` passes all eight arguments unchanged. **[bytes]**
The `domain == SEA` guard on *`this`* rather than on the container is
surprising and I flag it: I read the bytes correctly, but I cannot say
what game situation puts a sea unit inside another object. **[bytes for the
test, infer for the meaning]**

### 2.2 The "already in range" early-out — `6013A1`…`6013F9`

```
siege_flag = this->ptype->is_siege()                       (vfn +0x10c, 60135f)
          && (this->unit_masks & UNIT_AI_CONTROL)          (60136c)
          && target->is_build()                            (vfn +0x20,  60138f)

if (this->is_on_map() && !siege_flag
    && ObjectData::is_in_range(this, o, who, from_x, from_y, _, 0, NULL))  (6013db)
        { *out_x = from_x; *out_y = from_y; return 1; }    (6013e4..6013f9)
```
`is_on_map` is speculatively devirtualised: `6013aa cmpl $0x46CE30,%eax`
against `UnitData::is_on_map`, whose body is `inside_up >> 15`. **[bytes]**

### 2.3 Constants latched before the search

- `601405`: `long_range_cut = max(0x600, (this->max_range() + 4) * 0xC0)`
  — 8 tiles or `max_range+4` tiles, whichever is larger. Stored at
  `ebp-0x44`. **[bytes]**
- `60141e`…`6014E6`: if the order cursor is live and the current order's
  type is `ATTACK (10)` (`60145e cmpl $0xa`), latch
  `order_flag = order_data[0x1c]` (a byte, `601475`) and — only when
  `this->ptype->domain != AIR` and `this->ptype->domain !=
  target->ptype->domain` — `tregion = WorldData::get_tregion(world,
  &my_tile_x, &my_tile_y)` (`6014e1`). Otherwise `tregion = -1`
  (`60130d`). **[bytes]**

### 2.4 The standoff radius `R` — the heart of the function

`R` lives at `ebp-0x14`. Seeded at `60155d` with
`R = ObjectData::attack_dist(this, o, who, this->x, this->y)` — the
distance at which `this` can attack that target from where it stands.
The bearing `approach_angle = find_angle(from_x − target->x,
from_y − target->y)` is latched at `60153b` into `ebp-0x20`. **[bytes]**

Then, in this order:

1. `601579` `if (target->is_unit() && R > (this->max_range() + 8)*0xC0)`
   → `R = (this->max_range() + 2) * 0xC0` and set `moving_target = 1`
   (`6015ac`). *"A mobile target far outside my reach: aim at
   max_range+2 tiles."* **[bytes]**
2. else `60162b` `if ((this->ptype->role & ROLE_RANGED) != 0)`:
   - `60165c` `if (R < this->min_range()*0xC0 − 6 && !siege_flag)` —
     too close. `project@0092CF40(ecx = this->angle, edx =
     (min_range+1)*0xC0 − R)` **rewrites `from_x`/`from_y` in place**
     (`60168f..6016ab` pass `&[ebp+0x1c]`, `&[ebp+0x20]`), backing the
     reference point off along the unit's own facing; then
     `R = min_range()*0xC0 + 0x90`. **[bytes]**
   - `6016f1` else `if (R > this->max_range()*0xC0 − 6 || siege_flag)`:
     ```
     t  = this->max_range()*0xC0 − 0x60                    (601726)
     if (this->max_range() < 10) t += this->ptype->big_radius − 0x30   (601732..601743)
     if (target->is_unit())      t -= target->ptype->big_radius       (601759..601775)
     R  = max(0xC0, t)                                     (601778..601780)
     ```
   - `60170c` else: `R` is already inside `[min,max]` — leave it.
3. else (melee, `601789`): `R = 0x30` (`601791`). If
   `target->is_unit()` (`6017a1`) **and** `!target->is_moving()`
   (`6017ba`), hand the whole problem to
   `Unit::find_melee_pos@006010B0` (`6017eb`) and return its outcome
   (§6). **[bytes]**

`R` is a plain world-unit distance, not fixed point. Every `*0xC0`
converts a tile-scale range field into world units. **[bytes]**

**Falsifier for §2.4**: a `UnitsSync` frame in which a ranged unit
inside its own minimum range receives a move order — `orders_x/orders_y`
must land at `min_range*192 + 144` from the target along the unit's
`angle`. A capture with a bow-armed unit walked to melee contact would
print it. A `find_melee_pos` claim cannot be falsified from this
function's dump alone; it needs that function read separately.

### 2.5 The fork — `6015DF`

```
if (target->is_wallbuild() == 0)   → §5, the find_nearby_spot path
else                                → §3/§4, the perimeter walk
```
`6015df calll *0x1c(%eax)`, `6015e4 je 0x6027d4`. **[bytes]**

Immediately inside the perimeter branch, `6015ea`: if the unit's
**activity** type (latched at `60134c` from
`UnitData::get_activity(this)->vfn(0x10)()`) is `GUARD (12)`, write
`*out = (from_x, from_y)` and **return 0** (`6015f4..601611`). **[bytes]**

**A hedge I cannot resolve.** `is_wallbuild()` is 1 for both `Build`
*and* `Wall`, but the perimeter walk immediately calls
`target->get_build()` (`+0xac`) and dereferences the result at `+0x18`
— and `WallData::vftable+0xac` is the folded `return 0`. A `Wall` target
therefore reads `*(int*)0x18`. Either walls never reach here (excluded by
an upstream caller) or I have mis-read the slot. I could not settle it
without reading the callers' target selection, which was outside my
brief. **[bytes for the slot values, unresolved for the consequence]**

---

## 3. Candidate generation — the perimeter walk (building targets)

This is the only candidate enumeration this function performs itself.
It walks the **rounded rectangle at distance `R` outside the building's
bounding box**, with **two walkers going opposite ways from the same
seed point**, alternating one candidate each.

### 3.1 Geometry

`bx = target->get_build()->ptype->x_size`, `by = … y_size`, both in
**tiles**. The building's half-extent in world units is `bx*0x60` and
`by*0x60` (`0x60 = 96 = 192/2`). **[bytes, e.g. `601f30 movl 0x238(%eax)`
then `601f39 leal (%eax,%eax,2)` / `601f48 shll $5` = ×96]**

Eight **octants**, numbered 1..8, **even = a straight side, odd = a
corner arc**:

| oct | anchor | evidence |
|---|---|---|
| 1 | NW corner, base angle `0xC0000000` | `601c22` case 1 |
| 2 | N side: `(tx, ty − by*0x60 − R)` | `601e45`-ish case 2 |
| 3 | NE corner, base angle `0x00000000` | case 3 |
| 4 | E side: `(tx + bx*0x60 + R, ty)` | case 4 |
| 5 | SE corner, base angle `0x40000000` | case 5 |
| 6 | S side: `(tx, ty + by*0x60 + R)` | case 6 |
| 7 | SW corner, base angle `0x80000000` | case 7 |
| 8 | W side: `(tx − bx*0x60 − R, ty)` | case 8 |

Corner anchors are the *bare* corner (`tx ± bx*0x60`, `ty ± by*0x60`);
the `R` is added by the sine pair at run time (§3.4). Angles are the
`find_angle` convention: 0 = north, 0x40000000 = east, clockwise. **[bytes]**

### 3.2 Where the walk starts — `601650`…`601BB8`

```
dxo = |from_x − target->x| − bx*0x60      (60196x, decomp param_8)
dyo = |from_y − target->y| − by*0x60
corner = (dxo > 0) && (dyo > 0)
if (dyo < dxo)                                   // X-dominant
    if (!corner) oct = (from_x <= target->x) ? 8 : 4
    else if (target->x < from_x) oct = (target->y < from_y) ? 5 : 3
    else                        oct = (target->y < from_y) ? 7 : 1
else                                             // Y-dominant
    if (!corner) oct = (target->y < from_y) ? 6 : 2
    else if (target->y < from_y) oct = (from_x <= target->x) ? 7 : 5
    else                        oct = (target->x < from_x) ? 3 : 1
```
Both walkers start on the **same** octant and the **same** point
(`601e04/601e0e` write `cx[1]`/`cx[0]`, `601e17/601e1a` write
`cy[1]`/`cy[0]`, all from the same registers). **[bytes]**

Reading: *start where the attacker is, and open outward in both
directions.*

### 3.3 Step sizes — `601B4A`…`601BF0`

Linear step along a side (`ebp+0xC`, the reused `who` slot):

```
if (this->ptype->max_range == 0)                    step = 0x20; ang_cap = 0x20000000
else if (this->ptype->big_radius >= 0x31 && bx >= 3 && by >= 3)
                                                    step = 0xC0
else if (this->ptype->big_radius >  0x18 && bx >  1 && by >  1)
                                                    step = 0x40
else                                                step = 0x20
```
and, in the `max_range != 0` branch (`601bb8`):
```
ang_cap = min( (0x40000000 / this->max_range()) / (0xC0 / step), 0x20000000 )
nsteps  = 0x40000000 / ang_cap          (>= 2, because of the cap)
max_i   = nsteps − 1                    (ebp-0x28)
ang_step= 0x40000000 / nsteps           (ebp-0x4c)
```
So a corner arc (one quarter turn) is cut into `nsteps` angular
increments, and only indices `1 .. nsteps−1` are visited — the two
endpoints are the adjoining sides. **[bytes/decomp]**

A corner start seeds the step index at the **middle of the arc**:
`step[0] = step[1] = (max_i / 2) + 1`, with round-toward-zero
(`601df0 jns; addl $1; 601df5 sarl; 601df7 incl`). A side start leaves
`step[]` uninitialised, which is safe only because it is read solely
while the octant is odd, and every even→odd transition writes it
(`6025cd` tail). **[bytes]** — a latent uninitialised read that never
fires; worth reproducing as "set it when you enter the arc", not as
"initialise it to garbage".

### 3.4 One candidate — `601FB0`

```
w   = walker index (0 or 1), from ebp+0x10
x, y = cx[w], cy[w]
if (oct[w] & 1) {                                   // on a corner arc
    ang = step[w] * ang_step + base_angle[w]        (601fc6..601fd4)
    x += sin_table(fold(ang),               (ang<0 ? −R : R))   // = R·sin θ
    y -= sin_table(fold(ang + 0x40000000),  (…))                // = R·cos θ
}                                                   // R == 0 ⇒ both are 0
X = T[x >> 4]*0x30 + 0x18                           (602045..60205d)
Y = T[y >> 4]*0x30 + 0x18
tx = T[X >> 6];  ty = T[Y >> 6]
```
`fold` is the caller-side quarter-wrap at `601fe2..602001`
(`r = −r` when the angle is negative, then `0x7FFFFFFF − a` when bit 30
is set). **[bytes]**

So **the candidate is always a quarter-tile centre**, never an arbitrary
point: `48k + 24`. That is the grain of the whole search.

### 3.5 Advancing — `602183`…`602790`

```
w ^= 1                                               (602186)
if (oct[w] & 1) {                                    // arc
    step[w] += (w ? +1 : −1)                         (60219d / 6021a3)
    if (step[w] > max_i || step[w] < 1) {            (6021aa / 6021af)
        oct[w] += (w ? +1 : −1);  if (oct[w] < 1) oct[w] = 8
        re-anchor cx/cy to the new SIDE's near end   (switch @ 6021e6)
    }
} else {                                             // side
    advance cx or cy by ±step, direction by (w, oct) (switch @ 602441)
    if (|target_centre − cur| > size*0x60) {         (602590-ish)
        oct[w] += (w ? +1 : −1);  if (oct[w] > 8) oct[w] = 1
        re-anchor cx/cy to the new CORNER             (switch @ 6025cd)
        step[w] = (w ? 1 : max_i)                    (6025cd tail)
    }
}
```

**Walker 0 runs counter-clockwise (octant index decreasing), walker 1
clockwise.** The side-direction table, read out of the four cases at
`602441`:

| oct | walker 0 | walker 1 |
|---|---|---|
| 2 (N) | `cx -= step` (west) | `cx += step` (east) |
| 4 (E) | `cy -= step` (north) | `cy += step` (south) |
| 6 (S) | `cx += step` (east) | `cx -= step` (west) |
| 8 (W) | `cy += step` (south) | `cy -= step` (north) |

and the bound test uses `bx` on octants 2/6 and `by` on 4/8, against the
target *centre*, so a side spans the building's full width. **[decomp,
cross-checked against the four `movl 0x234/0x238` loads in the listing]**

The two wrap tests are one-sided each (`< 1 → 8` on the arc path,
`> 8 → 1` on the side path) and that is correct, because walker 0 only
ever leaves octant 1 downward and walker 1 only ever leaves octant 8
upward. **[decomp]**

**Falsifier for §3**: no `gamelog.ini` category prints the candidate
stream — only the winner reaches state. The nearest reachable check is
`UnitsSync` `orders_x/orders_y` for a unit ordered to attack a
**large** building (`x_size ≥ 3`): the destination must be a
quarter-tile centre at `R` outside the footprint, and for a wide
building the chosen point must lie on the side nearest the attacker.
An `orders_x` that is not `≡ 24 (mod 48)` falsifies §3.4 outright, and
that is the cheapest single check in this report.

---

## 4. Rejection, scoring and selection

### 4.1 Rejections (each **skips one candidate**, never the call)

In the order applied, all jumping to `602183`:

| # | address | test |
|---|---|---|
| 1 | `602093` | `UnitData::invalid_loc@00607C30(this, tx, ty, 1, 0, 0, 0, 1, junk) != 0` |
| 2 | `6020C9` | `world->tdata[ty*world->tile_xs + tx].mask & MASK_BLOCKED (0x4000)` |
| 3 | `6020E1` | `Objects::find_collision@0065B1B0(X, Y, this->o, this->who, 0) != 0` |
| 4 | `6020FA` | `Objects::find_ordered_collision@0065B440(X, Y, this->o, this->who) != 0` |

Notes, all **[bytes]**:

- `invalid_loc`'s ninth argument is `(X>>6)*4` — a live register the
  compiler pushed to fill the slot. `invalid_loc` never reads `param_8`
  (grep of its decompilation), so it is dead. Do not reproduce it.
- `find_collision` and `find_ordered_collision` are declared
  `__thiscall` but their bodies reach the singleton through
  `GameAccess::objects` and never touch `this`; the ECX at the call site
  still holds the tile index. Also dead.
- Nothing here tests ownership, line of sight, or whether the candidate
  is actually in weapon range of the target. Range is only re-checked
  *after* the walk, and only on the other branch (§5).

There is **no early-out for the whole call** inside the walk.

### 4.2 Scoring — `602107`…`602149`

```
score = vector_dist(X − from_x, Y − from_y)
      + Random::get(GameAccess::game_random, 0, 0xFFFF) % 0xC0
```
`602124 calll 0xA39D70` (Random::get), `60212A/60212F movl $0xC0; idivl`
(the remainder is taken in EDX), `602139 calll 0x46CFF0` (vector_dist,
`ecx = Δx`, `edx = Δy`), `602141 addl %ebx,%eax`. **[bytes]**

Three consequences worth stating plainly:

1. The metric is **distance the attacker must travel**, not distance to
   the target. Pure integers, no fixed point: `vector_dist` is the
   octagonal approximation `hi + lo²/(2·hi)`.
2. The RNG is drawn for **every candidate that clears §4.1**, whether or
   not it wins. Any port that draws lazily, or draws once, desynchronises
   `game_random` — this is the single most determinism-relevant line in
   the function.
3. The jitter is `rand % 192` = up to one tile, added to the score only.
   It never moves the candidate.

**Selection**: minimum. `602143 testl %edx,%edx; js accept` (best
initialised to −1 at `601E07`) and `602147 cmpl %edx,%eax; jge reject`.
**Strictly less than, so ties go to the candidate examined first** —
and because walker 0 goes first, a tie resolves counter-clockwise.
**[bytes]**

On a win: `bestX = X` (`602152`), `bestY = Y` (`602155`),
`best = score` (`602158`), and the budget is tightened (§4.3).

### 4.3 Budget — the exact rule

Initialised at `601E20`/`601E2A`: `budget = 100 (0x64)`, `counter = 4`.
Loop tail at `602782`:
```
counter += 1
continue while (counter − 4 < budget)
```
so **at most 100 candidates** are ever examined (counter 4…103), fifty
per walker. **[bytes]**

On each new best, at `60214B`:
```
if (R > 0x300)  budget = min(budget, counter + 11)      (60215d..60216a)
else            budget = min(budget, counter)           (60216f..602179)
```
With the tail's `counter − 4 < budget`, that means: after a candidate is
accepted at counter `c`,

- `R ≤ 0x300` (≤ 4 tiles): **exactly 3 more candidates** are examined,
- `R > 0x300` (> 4 tiles): **exactly 14 more**,

and each *further* improvement restarts that countdown from its own
`counter`. The selector between the two is `R` — the standoff radius
from §2.4 — against `0x300 = 768 = 4 tiles`. **[bytes]** I derived 3 and
14 from `c+1−4 < c` and `c+1−4 < c+11`; they are not round numbers and
that is the point.

The outer `do … while` at `601960`/`602795` decrements a counter
(`ebp-0x2c`) that is only ever written `1` — at `601306` and again at
`60217c` on every accepted candidate. It therefore **always executes
exactly once**. It is vestigial: a retry loop whose retry was disabled.
Reproduce the body, not the loop. **[bytes: the only three references to
`-0x2c(%ebp)` in the whole function are those two stores and one `subl`]**

**Falsifier for §4**: the RNG claim is the testable one. `LeadersSync`
or any category that prints the random cursor, on a frame where a unit
starts attacking a large building, will disagree with a port that does
not draw once per surviving candidate. The cap of 100 and the 3/14
countdown have **no capture that reaches them** — nothing logs the
candidate count — so those rest on the bytes alone.

---

## 5. The non-building path — `6027D4`

When `target->is_wallbuild()` is false, no enumeration happens here; the
work is delegated. In order:

1. **A flanking/hold decision** (`602800`…`602A1A`), entered only when
   `!(this->unit_masks & UNIT_AI_CONTROL)`, `this->ptype->domain` is
   `AIR` or equal to the target's, and `Unit::update_order(target)` is
   non-null and reports (vfn `+0x14`) that the target is itself busy,
   and `this->ptype->max_range != 0`. It compares the target's facing
   against `approach_angle`; a delta inside ±`0x2AAAAAAA` (~60°) means
   "I am in front of it", otherwise `flanking@0092CFE0` decides whether
   to slide sideways via `project` and `WorldData::is_valid`. **[decomp]**
2. `LAB_00602A1A`: if `target->max_range() < this->max_range()` or
   `this->unit_masks2 & UNIT2_MARINE_ENTRENCH`, and
   `vector_dist(…) < (this->max_range()+4)*0xC0`, then
   `*out = (this->x, this->y)` and **return 1** — stand still and shoot.
   **[decomp]**
3. Otherwise, one call (`602B31`):
```
d = target->ptype->big_radius + this->ptype->big_radius + R      (602ad7..602ae9)
if (d > 0x240) { lo = d − 0xC0; hi = d;  p7 = 0x60; }            (602aec..602afd)
else           { lo = d;        hi = 0;  p7 = 0; }
UnitType::find_nearby_spot@0061DE70(this->ptype,
    target->x, target->y, &tmp_x, &tmp_y,
    lo, hi, p7, approach_angle, filter(3 or 5),
    this->o, this->who, pass_through, 0, -1, 0, tregion)
```
`find_nearby_spot` returns **`!found`** — `return (uint)!bVar6` at its
`0061E3E7`, so **0 means a spot was found**. **[decomp]**

   - found and (`flank_used` or `moving_target`) → `*out = tmp; return 1`
     (`602B85`).
   - found otherwise → `ObjectData::is_in_range(this, o, who, tmp_x,
     tmp_y, …)`; true → `*out = tmp; return 1` (`602B71`), false → fall
     through to §6.
   - not found → fall through to §6.

---

## 6. Failure

There are three distinct failure shapes, and none of them is a sentinel
coordinate.

**a. The perimeter walk found nothing** (`6027AF`): `best < 0`, so it
falls into the common tail at `602BA2` with `out_x/out_y` untouched by
the walk.

**b. The common tail** (`602BA2`…`602DC2`):
```
*out_x = target->x;  *out_y = target->y;                 (602BC8/602BDA)
if (order_flag == 0) return 0;                           (602BDF → 601604)
if (vector_dist(|target->x − this->x|, |target->y − this->y|)
        < long_range_cut) return 0;                      (602C1B → 601604)

ok = find_nearby_spot(ptype, target->x, target->y, out_x, out_y,
                      bx*0xC0, bx*0x300, 0, 0x55555555, FILTER_NOT_ME,
                      this->o, this->who, 0, 0, -1, 0, tregion)   (602C82)
if (ok != 0) {          // NOT found — head for the neighbourhood instead
    *out_x = T[*out_x >> 8]*0x300 + 0x180;               (602C8F..602CAA)
    *out_y = T[*out_y >> 8]*0x300 + 0x180;
    *out_x += (this->x mod 0x300) − 0x180;               (602CC1..602CEE)
    *out_y += (this->y mod 0x300) − 0x180;
    WorldData::restrict(world, out_x, out_y);            (602D1B)
    if (this->ptype->domain == LAND
        && (world->tdata[…].mask & 0x30) == MASK_WATER) return 0;   (602D23..602D5F)
}
if (this->ptype->domain == SEA) return 0;                (602D87)
if (this->ptype->domain == LAND
    && target->ptype->domain == SEA) return 0;           (602DB0)
return 1;                                                (602DBD)
```
The snap is to the centre of a **4×4-tile block** (`0x300 = 768 = 4
tiles`, centre `0x180`), plus the attacker's own offset within its own
block — so a group converging on a distant building spreads out instead
of stacking. The `mod` is a true signed remainder
(`imull $0x2AAAAAAB; sarl $7` = ÷768, then subtract). **[bytes]**

**c. The melee shortcut** (`6017EB`…`601934`): when `find_melee_pos`
fails, `*out = (target->x, target->y)` and the same long-range snap runs
inline (`601828`…); if the order type is `ATTACK` the order byte at
`+0x1c` is cleared, and the function returns `find_melee_pos`'s own
return value — which was 0. **[decomp]**

**So**: the caller never gets a sentinel. On `return 0` the out-params
hold *the target's own position* (or, on §2.5's `GUARD` path, the
unchanged `from` position). A port that leaves the out-params alone on
failure will diverge from the original the first time a caller uses them
anyway. **[bytes]**

`UNIT_CAN_TRANSPORT` is restored on all of these.

---

## 7. Callers

Direct callers of `00601280`, found by scanning the whole `.text` for
`calll 0x601280`:

| site | function | note |
|---|---|---|
| `6012E8` | itself | the transport delegation of §2.1 |
| `602E88` | `Unit::find_attack_pos@00602E60` | the 7-arg overload; supplies `from = this->position` |
| `5F1D21`, `5F1D5A` | `Unit::do_attack@005F1B80` | two call sites |
| `7128A5` | `Group::action_attack@00712490` | group attack order |

Through the 7-arg wrapper (`calll 0x602e60` at `5E2710`, `5FE17F`,
`5FE629`, `600979`): `Unit::check_target_path@005E22D0`,
`Unit::fight@005FD4D0` (two sites) and
`Unit::target_opportunity@005FFFC0`. **[bytes]**

I read only the call sites, not what each caller does with the result —
that was outside the brief and I did not want to bleed into the
mechanic's own specification. The shape of the parameters
(`out_x`/`out_y`, plus the `UnitData::log_data` record printing
`orders_x`/`orders_y`) says the result becomes a movement destination,
but I mark that **[infer]**.

---

## 8. Confidence summary and the three claims I trust least

**Derived from the bytes** (disassembly read directly): §1 signature and
argument roles; §2.1–2.5 control flow and every constant in them; §3.1
geometry, §3.2 start selection, §3.3 step arithmetic, §3.4 candidate
quantisation; §4.1 rejection order and addresses; §4.2 scoring formula
and tie-break; §4.3 budget, including the 3/14 countdown and the
vestigial outer loop; §6b.

**Read from the decompiler and believed** (I did not re-check every
branch in the listing): §3.5's side-direction table and the two
re-anchor switches; §5's flanking decision; §6c.

**Plausible but unconfirmed**: the names `is_unit`/`is_wallbuild`/
`is_build`/`get_build` are from the PDB field lists and are solid, but
which concrete classes reach each branch at run time is inference; the
meaning of `order_data[0x1c]` (`order_flag`); the claim that the result
becomes `orders_x/orders_y`.

### The three I trust least

1. **Walls.** §2.5's hedge: `is_wallbuild()` admits `Wall`, but
   `Wall::get_build()` is the folded `return 0` and the perimeter walk
   dereferences it at `+0x18`. Either something upstream excludes walls
   or one of my slot readings is wrong. I did not resolve it, and I would
   resolve it by reading `Unit::fight`'s target selection rather than by
   guessing. **No capture reaches this** — a crash is not a log line.
2. **The `domain == SEA` guard on the transport delegation** (§2.1). The
   test is unambiguous in the bytes; what it *means* is not, and my
   reading of `inside_down` as "the object I am inside" rests only on
   `is_on_map` using `inside_up`, which is the sibling field, not this
   one. Falsifiable by `ObjectData::log_data`'s `inside_down` /
   `inside_down_who` columns on a frame with a loaded transport.
3. **The `flanking` branch of §5.** I read it from the decompiler only.
   Ghidra lost the `__fastcall` register arguments for `flanking`,
   `project` and `vector_dist` throughout that block (`unaff_ESI`,
   `unaff_EBX` everywhere), so the *operands* of those calls are the
   least-supported thing in this report. I verified the register
   conventions at the `602107`/`60153B` call sites but **not** inside the
   `602800`–`602A1A` block. Anyone implementing §5 should disassemble
   that range before trusting my prose.

One further note that is not a hedge but a warning: **this function
mutates `this->unit_masks` and, on the `ATTACK` path, the order data,
while nominally computing a position.** A port that models it as pure
will not diverge in the first frame, but will in the first frame where
`invalid_loc`'s `UNIT_CAN_TRANSPORT` test matters.
