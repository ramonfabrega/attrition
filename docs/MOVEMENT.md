# Movement

How fast a unit moves, which way it faces, and how its position advances by one
frame. Not how it decides where to go — pathfinding, collision and formations
are separate mechanics and are only surveyed at the end of this document.

**How this was established.** Symbol names, struct layouts and field offsets
come from `game/sbl/rise.pdb`. Behaviour comes from reading the original with
Ghidra, with those symbols applied, and from direct disassembly wherever the
decompiler dropped a register argument — which in movement is often, because
this code passes almost everything in registers. Nothing is transcribed; see
`docs/DECISIONS.md` entry 7.

**Confidence.** High throughout. The unit of speed, the three-layer speed
pipeline, the angle representation, `find_angle`, the sine table and its
generator, the per-frame step and the turn gate are all read end to end, and the
one thing that looked wrong turned out to be a misreading — see the note below,
which is kept because the mistake is instructive.

**Where the implementation is.** `crates/sim/src/movement.rs`.

---

## The unit of speed

**A speed is position units per frame**, where a position unit is 1/768 of a
world cell and 1/192 of a tile — the units `docs/FORMATS.md` derives from
`div_3_table`.

A unit type's `MOVES` field *is already that number*. `Unit::update_speed`
starts from the type's move value and multiplies by `UNIT_MOVE_SPEED`, whose
annotation reads `1/192 tile (granularity for unit movement speeds)`. One
1/192 of a tile is exactly one position unit, so that constant is the identity
converter: it says what a point of `MOVES` is worth, and multiplying by it
leaves the number alone.

Two independent checks agree:

- `FORCED_MARCH_SPEED` is written `42m` and `GENERAL_SPEED_BONUS` is `3m`. The
  `m` suffix is `MOVES` units, and 42 sits sensibly inside the shipped `MOVES`
  range, which runs from single digits to 115. Under any other scale it would
  be absurd.
- `UnitData::get_speed` floors its result at **3**. A floor of 3 is meaningful
  against a Citizen's 25 and meaningless against a number hundreds of times
  larger.

So a Citizen at `MOVES` 25 covers 25 position units per frame before the step
multiplier below, which at 15 frames per second is about two tiles a second.

**Nothing here needs fixed point.** Every quantity is an integer with defined
truncation, the same as attrition and supply. `Fx` remains unearned; see the
note in `docs/DECISIONS.md` entry 13.

## Angles are binary, and north is zero

An angle is a **signed 32-bit binary angle**: the full circle is 2³², so

| Direction | Value |
| --- | --- |
| North | `0` |
| East | `0x40000000` |
| South | `0x80000000` (i.e. `i32::MIN`) |
| West | `-0x40000000` |

Wrapping is free, arithmetic is exact, and there is no degree or radian
anywhere. `find_angle` returns exactly these values on the axes, which is how
the convention is pinned rather than guessed.

Note that **y increases southward** — `find_angle` negates its `dy` before
doing anything else, which is the screen convention showing through into the
simulation.

## Finding the angle to a point

`find_angle(dx, dy)` is an integer arctangent with **no table**:

```
if dx == 0:  return dy < 0 ? NORTH : SOUTH
if dy == 0:  return dx > 0 ? EAST  : WEST

a, b = |dx|, |dy|
hi, lo = max(a, b), min(a, b)
t = lo * 0x4000 / hi                                  # 0 .. 0x4000
m = (0x2800 - (|0x1333 - t| * 0xb00 >> 14)) * t  &  ~0x3fff
q = m * 4
```

then `q` is placed into the right octant by sign and by whether `|dx| <= |dy|`.

The polynomial is the whole thing: a straight-line correction term
`0x2800 - |0x1333 - t| * 0xb00 / 2^14` scaling the ratio `t`. It is exact on
the axes by construction and worst near the octant boundary, where it returns
541,917,184 against a true 45° of 536,870,912 — **0.94% high**. That error is
part of the game's behaviour, not a defect to be fixed: a true `atan2` would
change every unit's heading slightly and every position that follows from it.

The mask `& ~0x3fff` throws away the low 14 bits before the multiply by four,
so the result is quantised to 2¹⁶ of a full circle.

## The sine table

`sinx(angle, distance)` and `cosx(angle, distance)` return the component of a
step of `distance` along an angle. `cosx` is `sinx` a quarter-turn later —
literally: it computes `angle + 0x40000000` and falls into the same code.

Both go through a **256-entry quarter-wave table with linear interpolation**,
built at startup by `trig_init` into `.bss` at `0xe32f40`. The generator, read
from the floating-point constants it uses:

```
table[i] = (int)( sin(i * 1.570796327 / 255.0) * 65535.0 )      for i in 0..256
```

Three things about that line are worth stating.

**The π/2 is typed in, not computed.** `1.570796327` is π/2 rounded to ten
significant figures, and it is 1.3 × 10⁻¹⁰ larger than the nearest double. It
was a literal in the source.

**The divisor is 255, and the index runs 0..255.** The table therefore reaches
sin = 1 at its last entry, while the reader treats that entry as covering only
255/256 of the quarter turn. The table is stretched by 256/255 relative to how
it is indexed, so the engine's sine runs slightly ahead of a true one — at the
midpoint it gives 46482 where a true sine gives 46340.

**The reader interpolates into `table[(i + 1) & 0xff]`.** The index is
incremented as a byte, so the entry after the last one is the first one. In the
top 1/256 of a quarter the interpolation therefore runs from 65535 down toward
0 instead of staying at 65535.

The interpolation itself, from a 30-bit angle within the quarter:

```
i    = (a & 0x3fffffff) >> 22           # 0..255
frac =  a & 0x3fffff                    # 22 bits
s    = table[i] + ((table[(i+1) & 0xff] - table[i]) * frac >> 22)
return (s * distance) >> 16
```

`crates/sim` pins the 256 integers as constants rather than recomputing them.
The original builds them once from doubles; pinning the result is both
closer to what the original actually runs on and free of any dependence on a
platform's `sin`.

### The multiply overflows, and the answer depends on it

This is the one place in movement where reading the arithmetic carefully is not
optional, and an earlier draft of this document got it wrong in a way worth
recording.

`sinx` and `cosx` fold an angle into the first quarter before the lookup: a
negative angle — the far half of the circle — is handled by negating the
*distance*, and the second quarter is mirrored with `0x7fffffff - angle`.
Follow `cos(north)` through that. It becomes `0x40000000`, mirrors to
`0x3fffffff`, and lands on index 255 with a nearly full fraction — exactly the
wrapped step, interpolating from 65535 down toward `table[0]`. Read naively
that gives 1 instead of 65535, and a unit ordered due north does not move.

It does move, because the interpolation multiply is a 32-bit `imul` and
`-65535 × 0x3fffff` does not fit in 32 bits. Truncated, it comes back as
`+4,259,839`, which shifts down to **1**, and `1 + 65535` is **65536** —
exactly one in the 16.16 scale that the final `>> 16` undoes. The wrap cancels
the wrap.

So the axis angles work by arithmetic overflow. Widen that multiply while
"cleaning up" and every unit ordered along an axis stops dead.
`crates/sim/src/movement.rs` uses a `wrapping_mul` and says why at the site.

There are also **three precision paths** on the distance, differing only in
where the 16-bit shift is split so that `s × distance` cannot overflow: below
65535 the shift is all at the end, below 2²⁴ it is split eight and eight, above
that the distance is pre-shifted by sixteen. Which path runs changes where
truncation falls, so it is behaviour rather than optimisation — though a
movement step only ever takes the first.

## The per-frame step

`Guy::move` is the integration, per figure — RoN units are squads of one to
four, and each figure moves itself.

```
if position == destination:
    turn toward the destination facing, and stop

angle     = find_angle(dest.x - x, dest.y - y)
remaining = turn_towards(angle)              # turns, returns the turn still owed
if remaining > 2 * turn_speed:  return       # turning in place costs the frame

step = get_speed() * 11 / 8
if |dx| + |dy| <= step:
    arrive exactly on the destination
else:
    sx = sinx(angle, step)
    cy = cosx(angle, step)
    if |sx| > |dx|:  sx =  dx                # never overshoot on an axis
    if |cy| > |dy|:  cy = -dy
    if the world accepts (x + sx, y - cy):  move there
```

Four things in that are not what a fresh implementation would write.

**The step is `speed × 11/8`.** A literal, with no constant behind it —
1.375× whatever the speed pipeline produced. Every quoted speed in the game's
data is therefore 27% slower than what a unit actually covers.

**Arrival is a Manhattan test.** `|dx| + |dy| <= step`, not the octagonal
`vector_dist` that territory and supply use, and not a true distance. So a unit
approaching diagonally snaps to its destination from further away than one
approaching along an axis.

**Turning gates movement entirely.** A figure owing more than twice its turn
speed spends the whole frame turning and covers no ground. This is why heavy
units feel sticky when given a reversing order.

**The per-axis clamps only ever reduce.** They stop the trig from overshooting
the destination on either axis; they never extend a step.

## Where movement sits in the frame

`Guy::move` is reached from `Guy::process`, and `Guy::process` is called from
the **bottom** of `Unit::process` — after `process_healing`, after
`process_cloak`, after `process_attrition`, and after the supply check and the
bleed. Movement is the last thing a unit does each frame.

That ordering is observable, so it is part of the specification rather than an
implementation detail. Attrition looks at the position the unit had at the
*start* of the frame, which means:

- a unit stepping over a border does not bleed for the crossing on the frame it
  crosses;
- and symmetrically, a unit leaving hostile ground is charged for the frame it
  leaves on, if a tick was due.

Combined with the 32-frame refresh in `docs/ATTRITION.md`, this is why a raid
that is in and out quickly can cost nothing at all, and why a unit can take one
last tick standing on safe ground. `crates/sim/src/lib.rs` runs the two in this
order and `harness_tests.rs` pins both consequences.

## Turning

Turn speed comes from the type's own value scaled by two constants:

```
turn_speed = (type.turn_speed >> 8) * UNIT_TURN_SPEED
if packed:  turn_speed *= UNIT_PACK_TURN_BONUS
```

`UNIT_TURN_SPEED` is written `1/1 rate` and `UNIT_PACK_TURN_BONUS` `2x`, and
both are used as plain integer multipliers — 1 and 2.

**What the type's own stored turn speed means is not established.** The `>> 8`
suggests 8.8 fixed point, but a `<TURN_SPEED>` of 16 taken that way yields a
rate of 16 against angle differences in the tens of millions, which would make
a unit take minutes to turn round. Either the field is stored in binary angle
units already, or the shift is doing something else. Nothing in this document
depends on the answer, and `crates/sim` takes the type's value as an input
rather than asserting its scale.

`Guy::turn_towards` turns by at most `turn_speed` toward the target angle, the
short way round, and returns how much turn is still owed — which is what the
movement gate above tests. A difference below `0x2222220` — about 1/120 of a
circle — counts as already facing.

## The speed pipeline

Three layers, in this order. `crates/sim` implements the parts that do not
depend on tech, nation and hero state it does not yet model, and takes the rest
as an input.

### 1. `Unit::update_speed` — the cached base

Runs when something changes, and writes a cached value on the unit. From the
type's move value: transport and marine bonuses scaled by the player's age,
then a multiply by `UNIT_MOVE_SPEED`, then a whales bonus, then a correction for
the gunpowder foot line (below), then Bantu, French siege, Versailles,
aluminium, and finally `+ n/4` for each of the spy, general and supply upgrade
counts. The result is propagated down a linked chain of units.

**The gunpowder foot line carries a hardcoded speed correction**, tested most
advanced first because each type also *is* its predecessor:

| Type | Scale |
| --- | --- |
| Mechanized Infantry | ×9/8 |
| Infantry | ×17/16 |
| Rifleman | ×32/27 |
| Arquebusiers | ×5/4 |

No constant stands behind any of them. The correction shrinks as the line
advances, which reads like the stored `MOVES` values for these four were chosen
for something other than speed — animation cadence is the obvious candidate —
and then corrected back here.

### 2. `UnitData::speed` — auras and nation bonuses

On top of the cached value: the Iroquois spear bonus in allied territory, then
forced march from a general's aura, then Spitamenes and Blucher for cavalry,
Porus for elephants, Charles for everything, Napoleon for siege.

Forced march is the interesting one: it *replaces* the speed with
`FORCED_MARCH_SPEED × UNIT_MOVE_SPEED` rather than scaling it, and then takes
whichever of that and the unit's own speed is larger. A fast unit is not slowed
by joining a forced march.

### 3. `UnitData::get_speed(x, y)` — the effective speed here, now

The layer that actually feeds the step:

- the current order scales it — ×9/8 for two order types, ×10/8 for one of them
  under a flag;
- for an ordinary unit type, **being damaged halves it**, and standing on a
  world tile carrying flag `0x800` halves it again;
- a general doubles siege speed;
- **the group cap**: a unit in a group with no overriding order is limited to
  the group's speed, which is how a mixed army moves at its slowest member's
  pace;
- and the result is floored at **3**.

The floor is not decorative. Halving twice from a slow unit reaches it easily,
and it is what stops a damaged unit in mud from stopping altogether.

---

## Open questions

**Closed since the first draft, and kept because the mistake is the useful
part.** The quadrant fold in `sinx`/`cosx` looked wrong and was not. It sends
`cos(north)` through the table's wrapped last step, which read as plain
arithmetic gives 1 instead of 65535 and would freeze a unit walking north. The
32-bit multiply overflows, and the wrap cancels; see "The multiply overflows"
above. The lesson is general: in this codebase an expression that looks broken
should be checked for overflow before it is called broken. The first reading
also came from disassembling `sin_table` in two halves, which is how the
branch structure got mixed up.

**Still open:**

- **What world tile flag `0x800` is.** It halves speed, so it is terrain of
  some kind — forest, swamp or shallow water are the obvious candidates.
- **Whether `ai_speed` belongs to the simulation.** It multiplies the step
  directly inside `Guy::move`, and it is a global set by a cheat and by the
  game-speed control. If it is not identical on every client it is a desync,
  which suggests it is either fixed in multiplayer or part of the synchronised
  state.
- **Pathfinding.** `PathFinder`, `PathNode`, a `BRTree` and a `Stack<PathData>`
  per unit. Entirely unread.
- **Collision and pushing.** `PUSH_SIZE` and `PUSH_CIRCLES` in `unitrules.xml`
  describe a unit's collision profile as a row of circles. Unread.
- **Formations and groups.** `Group::compute_speed` is what fills the group
  speed the cap above reads. Unread.
- **The order layer.** `MoveOrder`, `PatrolOrder`, `AttackToOrder` and the rest
  sit above all of this and decide what the destination is.

---

## Second reading (2026-08-20) — corrections owed

Blind second derivation and adjudication: `docs/audit/2026-08-20-movement.md`.
Position units (192 per tile), the angle convention, `find_angle`, the sine
table's generator and lookup, `sinx`/`cosx`, `turn_towards`, the Manhattan
snap, the per-axis clamps, the speed floor of 3 and the order multipliers are
**doubly confirmed**. Nine disagreements, all resolved against this document;
until they land here and in `crates/sim/src/movement.rs` / `lib.rs`, this
document is wrong on:

- **Which step moves the unit.** `Unit::move_step` (reached through
  `Unit::work` → `do_move`) moves the unit's position; `Guy::move` only walks
  the *body* toward the point the unit already reached, at `speed·11/8`. The
  implementation applies the body's rule and its 11/8 to the unit — the "27%
  faster than quoted" claim is that mistake.
- **The step uses the post-turn facing**, not the pre-turn heading.
- **`UNIT_TURN_SPEED` loads as 256** (`get_fraction(…,0x100)`), and
  `TURN_SPEED` goes through `degrees_to_angle`; `Tuning` carries 1.
- `GuyData::turn_speed` is instant from a stop for foot and mounted units and,
  while moving, divides by `avg_speed/4 + 1` with a ~1° floor — the "sticky
  heavy units" behaviour pinned by a test is not the original's.
- `unit_masks & 0x10` is a moving-target-in-contact flag cleared each `work`,
  not "damaged"; the 0x800 tile halving needs `z_internal <= 0`; the group cap
  applies only with flag 0 (never to the body); the transport/marine scale is
  `epoch[0]`, not age.

The second reading also derives what this document took as input or left
open: the full `move_step` (turn-in-place limits of 45°/80°, half step while
turning, the `manh < 2·step` clamp gate, tolerance arrival, collision), modern
infantry ×5/4, Alexander's forced march, the hero multiplier scales,
`ai_speed = 1`.
