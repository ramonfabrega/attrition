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

A blind second reading (`docs/audit/2026-08-20-movement.md`) doubly confirmed
the unit of speed, the angle convention, `find_angle`, the sine table and its
lookup, `turn_towards`, the Manhattan snap and the per-axis clamps — and
overturned the frame the first draft hung them on. The original keeps **two
positions per unit**: the unit's own, which `Unit::move_step` advances and
which everything in the simulation reads, and the *body's*, which `Guy::move`
walks toward the unit. The first draft read `Guy::move`, took it for the
integration, and applied its rules — including an 11/8 multiplier that belongs
to the body — to the unit. Those corrections are landed below and marked where
they changed what an earlier draft said.

**Confidence.** High throughout. The unit of speed, the three-layer speed
pipeline, the angle representation, `find_angle`, the sine table and its
generator, both per-frame steps and both turn rules are read end to end, and
the one thing that looked wrong turned out to be a misreading — see the note
below, which is kept because the mistake is instructive.

**Where the implementation is.** `crates/sim/src/movement.rs`, driven from
`Sim::process_movement` in `crates/sim/src/lib.rs`.

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
leaves the number alone. `Constants::init` reads it with `get_fraction(…, 0xc0)`
— `1 × 192 / 192 = 1`.

Two independent checks agree:

- `FORCED_MARCH_SPEED` is written `42m` and `GENERAL_SPEED_BONUS` is `3m`. The
  `m` suffix is `MOVES` units, and 42 sits sensibly inside the shipped `MOVES`
  range, which runs from single digits to 115. Under any other scale it would
  be absurd.
- `UnitData::get_speed` floors its result at **3**. A floor of 3 is meaningful
  against a Citizen's 25 and meaningless against a number hundreds of times
  larger.

So a Citizen at `MOVES` 25 covers **25 position units per frame** — at 15
frames per second, just under two tiles a second. (An earlier draft said the
unit actually covered `25 × 11/8 = 34` and that every quoted speed was "27%
slower than what a unit actually covers". It is not: the 11/8 is the *body's*
catch-up rate, below. The unit moves at its quoted speed.)

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
anywhere in the step. `find_angle` returns exactly these values on the axes,
which is how the convention is pinned rather than guessed.

Note that **y increases southward** — `find_angle` negates its `dy` before
doing anything else, which is the screen convention showing through into the
simulation.

**Confirmed in a logged run (2026-08-20), and it settles a question in another
document.** `UnitData +0x50 angle` is the direction the unit *faces*, and a
walking unit faces where it is going: read frame by frame at `UNITS=3`, a
squad stepping south-east logged `angle` 133.90° falling to 133.45° against a
per-frame position delta whose `find_angle` is exactly 135.00°, and one
stepping north-west logged −43.20° rising to −42.85° against −45.00°. The
heading quantises to the diagonal because the step does; the facing sits about
two degrees off it and drifts smoothly, which is a unit turning as it walks.
`tools/gamelog/heading.py` is the reader. `docs/COMBAT.md` §14.1 rested its
flank convention on exactly this, and it is what makes level 1 the rear.

Degrees do appear in one place: the data. A type's `<TURN_SPEED>` is written
in degrees and converted once at load by `degrees_to_angle`, which is not a
multiply by 2³²/360 but a decomposition into exact pieces — quarter turns of
`0x40000000`, then 45° (`0x20000000`), 30° (`0x15555555`), 15° (`0xaaaaaaa`),
whole degrees at `0xb60b60`, plus **three units for every complete five
degrees in the remainder** to make up what `0xb60b60` loses to truncation.
`degrees_to_angle(45)` is exactly `0x20000000`; `degrees_to_angle(1)` is
`0xb60b60`; `degrees_to_angle(20)` is `0xe38e38d`, one more than the
`0xe38e38c` the unit step compares it against, which is why a 20° type is not
"slow" and a 19° type is. `crates/sim` implements the same decomposition.

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

## Two positions, two steps

A unit in the original is a `Unit` and one or more `Guy`s — the figures drawn
for it, one to four per squad. **The `Unit` has a position and so does each
`Guy`, and they are moved by different code at different rates.**

- The unit's position is `UnitData::x_internal / y_internal` (XOR-masked, see
  `docs/FORMATS.md`). It is what territory, attrition, supply, collision and
  every other mechanic read, and **`Unit::move_step` advances it** once a frame
  from `Unit::work → do_job → do_move`. Its step is the unit's speed, plainly.
- Each guy's position is `GuyData::x / y`, and `GuyData::des_x / des_y` is
  where it is trying to be. `Unit::set_new_location` writes the unit's new
  position into guy 0's `des_x / des_y` on every step (`Unit/set_new_location`
  +158), and **`Guy::move` walks the body toward that point** at
  `floor(speed × 11/8)` — fast enough to always catch up, which is the whole
  reason the multiplier exists.

So the unit is a point that moves at its quoted speed and the body is a
presentation that chases it. The two share one thing that matters to the
simulation, guy 0's **facing** (`GuyData::angle`), which both steps turn and
both steps read; and the body's bookkeeping — `last_speed` and `avg_speed` —
feeds the unit step's turn rate, below. That is why `crates/sim` models the
body's position and those two counters even though nothing in the simulation
reads the body's position yet: the unit's turn rate is not honest without them.

(An earlier draft described only `Guy::move`, called it "the integration, per
figure", and applied it to the unit's position. Every consequence that
followed — the 11/8 on the unit, the "27% faster than quoted" claim, the
pre-turn heading in the trig, and the turn gate as the unit's rule — was that
one mistake.)

## The unit step — `Unit::move_step`

Once per frame, given the order's destination and a step already computed by
`do_move` — `get_speed(x, y, 0)`, × `ai_speed` if that is above 1, and × 5/4
for modern infantry:

```
dx, dy    = dest - pos
want      = find_angle(dx, dy)
ts        = turn_speed(guy 0, mode 0)                    # see "Turning"
new, owed = turn_towards(facing, want, ts)               # snap if ts >= owed
manh      = |dx| + |dy|
slow      = type.turn_speed < 20° ? 2 : 1

if manh < slow * 192  or  path.flags & 4:                # short hop, or told to
    if owed != 0:  facing = new; return                  # turn in place
else:
    limit = 45°
    if (non-land type or vehicle) and manh >= slow * 384:  limit = 80°
    if owed >= limit:  facing = new; return              # turn in place

if owed >= 45° / slow:  step /= 2                        # half step while turning
elif unit_masks & 0x100000:  step /= 2, clear the bit    # one-shot half step
facing = new

if manh <= step:
    arrive exactly on the destination                    # Manhattan snap
else:
    sx = sinx(new, step)                                 # post-turn facing
    cy = cosx(new, step)
    if manh < 2 * step:                                  # only then:
        if |sx| > |dx|:  sx =  dx                        # clamp each axis
        if |cy| > |dy|:  cy = -dy
    if (x + sx, y - cy) is inside the world:  move there
    arrived if |dest - pos| <= tolerance                 # 0 unless colliding
```

Things in that a fresh implementation would not write:

**The step is the speed.** No multiplier. (An earlier draft put the body's
11/8 here.)

**Turning in place has two gates, and they depend on distance.** Close to the
destination — under one tile, or two for a type whose turn speed is below 20°
— any turn still owed after this frame's turning costs the whole frame. Further
out, the unit may move while it still owes up to 45°, or up to 80° if it is a
ship, an aircraft or a vehicle at two tiles or more. The "slow" doubling is
what stops a lumbering type from spending its last tile spinning.

**It walks at half speed while still turning.** Owing 45° or more (22.5° for a
slow type) after this frame's turn halves the step. A separate one-shot half
step, `unit_masks & 0x100000`, is set by `detect_unit_collision` and consumed
here; it is collision's, not movement's, and is not modelled.

**The trig uses the facing after this frame's turn**, `new`, not the heading
`want`. A unit that cannot complete its turn this frame walks along where it
is actually pointing. (An earlier draft used `want`; the difference shows
whenever a turn is still owed.)

**The clamps are gated on `manh < 2 × step`.** Only within two steps of the
destination are the components clamped to the remaining `dx` and `dy`. Further
out they are taken as the table gives them — which changes nothing observable,
because a component of a step no longer than half the Manhattan distance cannot
exceed either leg even with the table's and the arctangent's percent of error;
the gate is a shortcut, kept because it is the rule. The clamps only ever
reduce, and when both fire the unit lands exactly on the destination.

**Arrival is the Manhattan snap.** `manh <= step` puts the unit exactly on its
destination, the same test the body uses. After a partial step the unit also
counts as arrived if what remains is within `UnitData::tolerance`, which is
zero except after an unresolved collision, where `move_step` sets it to twice
the remaining Manhattan distance so the unit can give up short.

**A step outside the world is refused, not clamped**, and the order continues;
the unit tries again next frame. (`crates/sim` asks `World::accepts`, which
is the same bounds test.)

Not modelled, and listed at the end: the flyer branch (`unit_flags & 0x20`,
which turns by the body's rule instead), `detect_unit_collision` /
`resolve_unit_collision`, `UnitData::invalid_loc` on a tile change, the path
stack and its flags, and the order angle the unit snaps to on the final
waypoint.

## The body step — `Guy::move`

Once per frame for every guy, from `Guy::process`, whether or not the unit is
moving. For guy 0 (the others are offset copies, or snap to their own
destinations):

```
if pos == des:                                   # caught up
    last_speed = 0
    turn toward des_angle at turn_speed(mode 1)  # only if not turned this frame
else:
    heading = find_angle(des - pos)
    owed    = turn_towards(facing, heading, turn_speed(mode 1))
    if 2 * turn_speed(mode 1) < owed:  return    # turning costs the frame
    step       = floor(get_speed() * 11 / 8)     # × ai_speed if > 1
    last_speed = step
    if |dx| + |dy| <= step:
        last_speed = vector_dist(dx, dy)
        pos = des                                # Manhattan snap
    else:
        sx = sinx(facing, step); cy = cosx(facing, step)    # post-turn facing
        if |sx| > |dx|:  sx =  dx                # never overshoot on an axis
        if |cy| > |dy|:  cy = -dy
        if the world accepts (x + sx, y - cy):  move there
avg_speed = (avg_speed * 3 + last_speed) / 4     # skipped on the turn-gate return
```

**The 11/8 is the body's.** A literal, with no constant behind it, and its job
is to make the body faster than the unit so the body is always on the unit's
heels — an 8/8 body would fall behind on every diagonal, where the Manhattan
distance of the unit's step is longer than the body's snap. (An earlier draft
put this on the unit.)

**The body's turn shares the unit's facing, and for a moving unit never moves
it.** Guy 0's facing is the same field the unit step just turned, and the body
turns toward where the unit actually went — which is along the facing the unit
just stepped on, so the two agree to within the snap tolerance. `crates/sim`
models the turn anyway, on the shared facing, because it is three lines and it
is what turns an idle body: standing on its destination, not turned this frame
by the unit step, it turns toward `des_angle` — the heading the last unit step
recorded through `Unit::set_angle`, or the order angle on arrival, which the
simulation does not have yet.

**`last_speed` and `avg_speed` are the body's, and the unit reads them.** The
unit step's turn rate divides by `avg_speed / 4 + 1` and goes instant when
`last_speed == 0`, so a unit that stops turns on a sixpence and a unit at full
stride turns slowly. `avg_speed` decays by a quarter a frame once the body has
caught up and stopped.

`GuyData::get_speed` is `UnitData::get_speed(body x, body y, 1)` — the body's
tile, and **flag 1, so the group cap never applies to the body** — plus nine
when the current order's vslot `0x2c` is set. `crates/sim` feeds the body the
same speed input as the unit; the `+9` and the cap difference are not modelled.

## Where movement sits in the frame

Both steps run from the **bottom** of `Unit::process` — after
`process_healing`, after `process_cloak`, after `process_attrition`, and after
the supply check and the bleed. `Unit::work` (vslot `0x188`) is called first
and reaches `move_step` through `do_job → do_move`; then `Guy::process` is
called for each guy and reaches `Guy::move`. Movement is the last thing a unit
does each frame, and the unit moves before its body does.

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

`GuyData::turn_speed(mode)` is the one turn rate, and both steps call it:

```
base = (type.turn_speed >> 8) * UNIT_TURN_SPEED
if packed:  base *= UNIT_PACK_TURN_BONUS
if last_speed == 0 and the type turns instantly from a stop:  return 0x80000000
if mode == 1:  return base                                   # the body
return max(UNIT_TURN_SPEED * 0xb60b,  base / (avg_speed / 4 + 1))   # the unit
```

**`type.turn_speed` is a binary angle, and `UNIT_TURN_SPEED` is 256.** The
type's `<TURN_SPEED>` is degrees in the file — `unitrules.xml`'s own comment
says so: "Turn rate of unit, measured in degrees. Modified by `UNIT_TURN_SPEED`
in `rules.xml`. … Foot & Mounted units turn instantly from a stopped position:
so this applies mostly 'while moving'." — and `UnitType::init` passes it
through `degrees_to_angle` before storing it. `UNIT_TURN_SPEED` is written
`1/1 rate` and `Constants::init` reads it with `get_fraction(…, 0x100)`, which
makes it **256**, not 1. So `(turn_speed >> 8) * 256` is the type's angle with
its low eight bits cleared, and `UNIT_TURN_SPEED * 0xb60b` is `0xb60b00`, one
degree to the same precision. `Tuning::RON.unit_turn_speed` carries 256 and
`rondata` checks it as an 8.8 slot, the same way it checks `PEASANT_RATE`. (An
earlier draft had 1, took the type's value as an input of unknown scale, and
said nothing depended on the answer. The floor of one degree a frame depends on
it, and so does the instant-turn rule's existence.)

`UNIT_PACK_TURN_BONUS` is `2x`, read with `get_item`, and applies when
`unit_masks & 0x80000` — the packed state.

**Foot and mounted units turn instantly from a standstill.** The flag is
`guy_flags & 0x10`, set in `Guy::init_real` for a type that is not
`unit_flags2 & 4`, has objmask bit `0x20` or `0x1000` or `unit_flags & 0x10`,
and does not have `unit_flags & 2`. With `last_speed == 0` the rate is
`0x80000000`, which read unsigned is larger than any turn that can be owed, so
`turn_towards` snaps. A Citizen ordered the other way turns and walks on the
same frame. (An earlier draft said "heavy units feel sticky when reversed" and
pinned a unit standing fourteen frames to turn; the original has that only for
types without the flag — siege, ships — or for a unit already moving.)

**While moving, the unit turns slower the faster it goes.** Mode 0 divides the
base by `avg_speed / 4 + 1` and floors the result at one degree a frame. A
Citizen (45°, `MOVES` 25) at full stride has `avg_speed` 25 and turns at
`45° / 7` ≈ 6.4° a frame; a 5° type — the slowest shipped — at the same speed
is on the floor. The body, in mode 1, always gets the base.

`turn_towards` turns by at most the rate toward the target angle, the short way
round, and returns how much turn is still owed. A difference below `0x2222220`
— three degrees — counts as already facing and snaps to the target. The
magnitude of an anticlockwise turn is taken with a bitwise NOT, so it comes out
one short; reproduced because it is free and the value is one the interface
shows.

## The speed pipeline

Three layers, in this order. `crates/sim` implements the parts that do not
depend on tech, nation and hero state it does not yet model, and takes the rest
as an input.

### 1. `Unit::update_speed` — the cached base

Runs when something changes, and writes a cached value on the unit. From the
type's move value: the transport bonus (`MILITARY_TRANSPORT_BONUS`, 0) and the
American marine bonus (`AMERICANS_MARINE_SPEED_BONUS`, 2) scaled by
`LeaderDataEncrypt::epoch[0]` — the owner's **Military library level**, not
the age (an earlier draft said age); then a multiply by `UNIT_MOVE_SPEED`,
then a whales bonus, then a correction for the gunpowder foot line (below),
then Bantu, French siege, Versailles, aluminium, `+ n/4` for each of the spy,
general and supply upgrade counts, and last `+AZTEC_MOVE_SPEED%` for two
Aztec types. The result is propagated down a linked chain of units.

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
forced march from a general's aura, then the heroes — Spitamenes and Blucher
for cavalry, Porus for elephants, Charles for everything, Napoleon for siege.
Spitamenes and Porus are `× 307 >> 8` (a `12/10` loaded as 8.8), Blucher and
Charles `× 120 / 100`, Napoleon `× 150 / 100`.

Forced march is the interesting one: it *replaces* the speed with
`FORCED_MARCH_SPEED × UNIT_MOVE_SPEED` (Alexander's variant is
`42 × 384 >> 8 = 63`) rather than scaling it, and then takes whichever of that
and the unit's own cached speed is larger — the comparison is against the
cached value, before the Iroquois bonus. A fast unit is not slowed by joining a
forced march. A non-land type returns the cached value before any of this.

### 3. `UnitData::get_speed(x, y, flag)` — the effective speed here, now

The layer that actually feeds the step:

- the current order scales it — ×9/8 for two order types, ×10/8 for one of them
  under a flag;
- for a **land** type (domain `+0x218 == 0`): **`unit_masks & 0x10` halves
  it** — the bit `Unit::target_opportunity` sets when a moving unit is within
  the two units' radii of a target object, and `Unit::work` clears at the end
  of every frame. It is "moving in contact with a target", not "damaged" (an
  earlier draft said damaged). Standing on a world tile carrying flag `0x800`
  halves it again, **but only with `z_internal <= 0`** — a unit in the air
  over the tile is not slowed by it (the earlier draft omitted the `z` test);
- a general doubles siege speed;
- **the group cap, with flag 0 only**: a unit in a group with no overriding
  order is limited to the group's speed, which is how a mixed army moves at
  its slowest member's pace. `do_move` passes 0 for the unit step;
  `GuyData::get_speed` passes 1 for the body, which is therefore never capped;
- and the result is floored at **3**.

The floor is not decorative. Halving twice from a slow unit reaches it easily,
and it is what stops a unit in mud from stopping altogether.

`do_move` then scales that once more for the unit step: × `ai_speed` when it
is above 1, and × 5/4 for **modern infantry** — `UnitData::is_modern_infantry`,
a `unit_flags & 0x100` type at age six or later, or any such type under tribe
bonus `0x12`.

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

~~**What the type's own stored turn speed means.**~~ Degrees in the file,
`degrees_to_angle` at load, a binary angle in memory; and `UNIT_TURN_SPEED` is
256. See "Turning".

~~**Whether `ai_speed` belongs to the simulation.**~~ `Game::init_data` sets it
to 1 and only a cheat changes it. It multiplies both steps and is part of the
synchronised state by construction; `crates/sim` treats it as 1.

**Still open:**

- **What world tile flag `0x800` is.** It halves land speed at ground level, so
  it is terrain of some kind — forest, swamp or shallow water are the obvious
  candidates.
- **The flyer branch of `move_step`.** A type with `unit_flags & 0x20` owing
  less than a quarter turn turns by `do_turn` and then again by the body's
  rate, with no turn-in-place and no half step. Read, not modelled: the
  simulation has no aircraft.
- **Pathfinding.** `PathFinder`, `PathNode`, a `BRTree` and a `Stack<PathData>`
  per unit. `move_step` takes the top of the stack as its waypoint and pops it
  on arrival; `PathData.flags & 4` forces turn-in-place and `& 2` allows a
  final snap through a collision; `& 1` marks the last waypoint. Entirely
  unread beyond that.
- **Collision and pushing.** `detect_unit_collision` runs on every proposed
  step, `resolve_unit_collision` when it fails, `UnitData::tolerance` is set to
  `2 × manh` when the unit gives up, and the `0x100000` one-shot half step is
  collision's. `PUSH_SIZE` and `PUSH_CIRCLES` in `unitrules.xml` describe a
  unit's collision profile as a row of circles. Unread.
- **Formations and groups.** `Group::compute_speed`, `report_speed` and
  `leader_report_speed` fill the group speed the cap reads. Unread.
- **The order layer.** `MoveOrder`, `PatrolOrder`, `AttackToOrder` and the rest
  sit above all of this and decide what the destination is — and the angle the
  unit and its body snap to on the last waypoint.
- **The body's `+9`.** `GuyData::get_speed` adds nine when the current order's
  vslot `0x2c` is non-zero. Which orders, and why nine, is unread.

---

## Second reading (2026-08-20) — landed

Blind second derivation and adjudication: `docs/audit/2026-08-20-movement.md`.
All nine disagreements are landed above and in `crates/sim/src/movement.rs` /
`lib.rs`: the two-position model (D1), the 11/8 as the body's rate (D2), the
post-turn facing in the trig (D3), `UNIT_TURN_SPEED = 256` and
`degrees_to_angle` (D4), the instant and moving-mode turn rules (D5), the
`unit_masks & 0x10` meaning (D6), the `z <= 0` gate (D7), the group cap on the
unit step only (D8) and `epoch[0]` (D9). The audit's section 2 — the full
`move_step`, modern infantry, Alexander, the hero scales, `ai_speed` — is
folded in where it belongs. The decompile agreed with the audit at every point
re-read; what this reading adds beyond the audit is that the body's turn shares
the unit's facing field and, for a moving unit, never moves it — which is what
lets the simulation model the body's counters without a second facing.
