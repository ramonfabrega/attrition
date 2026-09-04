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

**And a third pass, 2026-08-27 (queue item 34), which is the one with a check
behind it.** Three things this document had wrong survived both readings and
were caught by a differential: guy 0's body does not chase the unit at all
(the 11/8 belongs to a guy with a track offset), `UnitData::angle` is the
heading rather than the facing, and a unit starts life at `0x55555555` rather
than north. Each is marked where it changed what an earlier draft said, and
"The checks" at the end is what now holds them.

**Confidence.** High throughout, and for the first time partly *measured*
rather than argued: both of a unit's angles are compared against the
original's own dump on every unit-frame where the two sides agree on the
position, which on run20's opening is exact and on run10's 1,772 frames leaves
only the `set_angle` callers this simulation does not make. The unit of speed,
the three-layer speed pipeline, the angle representation, `find_angle`, the
sine table and its generator, both per-frame steps and both turn rules are
read end to end, and the one thing that looked wrong turned out to be a
misreading — see the note below, which is kept because the mistake is
instructive.

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
document.** A walking unit points where it is going: read frame by frame at
`UNITS=3`, a squad stepping south-east logged `angle` 133.90° falling to
133.45° against a per-frame position delta whose `find_angle` is exactly
135.00°, and one stepping north-west logged −43.20° rising to −42.85° against
−45.00°. `tools/gamelog/heading.py` is the reader. `docs/COMBAT.md` §14.1
rested its flank convention on exactly this, and it is what makes level 1 the
rear.

**Which field that is, corrected 2026-08-27 (item 34).** An earlier draft
called `UnitData +0x50 angle` "the direction the unit faces". It is the
direction the unit *wants* — the bearing to where it is going. See "Two
angles" below: `move_step` writes it from `find_angle` on every frame, before
turning, and the facing the step is actually taken along lives in
`GuyData +0x18`. The observation above stands either way, because a unit that
has finished turning has the two equal; what it cannot distinguish is exactly
what the correction is about. A unit the original has just created carries
neither: `Unit::init@00612100` writes **`0x55555555`** — 120° — into `angle`
and `dest_angle`, a literal with nothing behind it, and that is what every
unit faces until its first order (`Angle::INITIAL`).

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

### The mirror is the original's; the decompiled branch is real and unreached (closed 2026-08-27)

The paragraph above says the second quarter is "mirrored with
`0x7fffffff - angle`", and that is what `crates/sim` does. The decompiled
`sin_table@00a46a00` reads as though it did something else — keep the index
it was given and, with bit 30 set, return `0xffff − cur + delta` — and on
2026-08-26 transcribing that made run31's `curr` wrong by hundreds. Read
from the listing (`llvm-objdump` over `a46a00`–`a46b27` and `92d0c0`–`92d129`),
both are true, and there is no contradiction:

- **`sin_table` is exactly what the decompiler says.** `a46a2d`: `test eax`
  on `angle & 0x40000000`; the bit-clear path (`a46a5f`) is `delta + cur`
  and the bit-set path (`a46a4c`–`a46a52`) is `delta − cur + 0xffff`, on the
  unmirrored index. That is not a mirror and not a sine.
- **The mirror lives in the callers.** `sinx@0092d100` and `cosx@0092d0c0`
  negate the distance and clear bit 31 for a negative angle, then
  `mov ecx, 0x7fffffff; sub ecx, angle; and eax, 0x40000000; cmove ecx, angle`
  — mirror when bit 30 is set, keep otherwise — and tail-jump to
  `sin_table` with **bit 30 always clear**. `cosx` is `sinx` after
  `lea esi, [ecx + 0x40000000]`.
- **The compiler inlined that fold at the call sites.** The sim's functions
  — `Unit::move_step` (twice), `Form::compute_dests` (fourteen),
  `Group::update_positions` (four), the three pathfinders, `Army::do_forming`,
  `Guy::set_angle`, `go_around_building`, `check_fuel`, `project` — call
  `sin_table` directly, and every one of them carries the
  `0x7fffffff`/`cmove` sequence in the instructions before the call. Ghidra
  shows `sin_table(unaff_EDI, unaff_ESI)` because it lost the register
  arguments, which is why the fold was invisible from the decompile.
- **63 of the 65 call sites in the executable fold.** The two that do not
  are one loop in `MapGrass::make_continents@00695050` (`6950cb`, `6950e5`),
  which walks `n` equally spaced angles round a full circle and takes the
  cosine as `sin_table(angle + 0x3fffffff)` — the `cos_table@00a469f0`
  convention (`add ecx, 0x3fffffff; jmp sin_table`), which has no other
  caller. So the bit-30 branch is live in **map generation only**, and the
  day that is ported it must be reproduced as written, sawtooth and all.

For everything this document covers, then, `sin_component`'s mirror is the
original's own arithmetic, instruction for instruction. The lesson is the
audit README's: when the decompiler prints an `unaff_` argument, the
listing is the only place the caller's half of the contract can be read.

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
  +158), and **`Guy::move` puts guy 0 straight onto that point** — see "The
  body step". Guy 0 does not chase and never lags.

So the unit is a point that moves at its quoted speed and the body is a
presentation pinned to it. The body's bookkeeping — `last_speed` and
`avg_speed` — feeds the unit step's turn rate, below. That is why `crates/sim`
models the body at all, even though nothing in the simulation reads its
position yet: the unit's turn rate is not honest without those two counters,
and the *value* they hold turns out to depend on the body not chasing.

(An earlier draft described only `Guy::move`, called it "the integration, per
figure", and applied it to the unit's position. Every consequence that
followed — the 11/8 on the unit, the "27% faster than quoted" claim, the
pre-turn heading in the trig, and the turn gate as the unit's rule — was that
one mistake. **A later one, corrected 2026-08-27, kept the 11/8 as the
body's rate.** It is not guy 0's either; see below.)

## Two angles

Also two *angles* per unit, in different fields, and telling them apart is
what item 34 was.

| field | what it is | who writes it |
| --- | --- | --- |
| `GuyData::angle` (`+0x18`) | the **facing** — the direction the step is taken along | `Guy::do_turn` only, and never by more than the turn rate |
| `UnitData::angle` (`+0x50`) | the **heading** — the bearing to the current destination | `Unit::set_angle`, outright, from anywhere |
| `GuyData::des_angle` (`+0x64`) | the same value again | `Unit::set_angle` writes both in one call |
| `UnitData::dest_angle` (`+0x58`) | the **order's** angle: where the order wants the unit to end up pointing | `Unit::update_action` (§3.3 of `docs/ORDERS.md`) |

`Unit::move_step` calls `find_angle` on the remainder to its destination and
then, at once and before any turning, `set_angle(this, want, …, 0)`. So the
heading is the bearing every frame, quantised to 2¹⁶ of a circle by
`find_angle`'s mask, while the facing crawls toward it at the turn rate. The
third argument is the snap flag; `move_step` passes zero at all three of its
call sites, so **nothing in the step ever snaps the facing**. Only
`Guy::set_angle` with that flag set does, and the step does not use it.

Two consequences worth stating because they are easy to get backwards:

- The group layer's `update_positions` rotates its slot table by the leader's
  **heading** (`docs/GROUPS.md` §6.6), not its facing — the two differ by up
  to the turn rate on any frame the leader is turning, which is exactly why
  §6.6 had to call `curr` a mid-frame quantity.
- An idle body turns toward the heading (`Guy::move`'s first branch), so a
  unit that arrives and is given the order's angle swings onto it over the
  frames after it stops, rather than snapping on the frame it arrives.

## The docs-versus-code pass, 2026-09-05

`docs/audit/2026-09-05-movement-vs-code.md` (Opus reader, Opus
adjudication). 96 stated rules traced to the code that implements them;
seven disagree, all seven confirmed, none struck.

**Three are this document lagging the code**, and are the lane's to correct:
the body step's sea / `SPECIAL_ANIM` arm sets `stopped` and the pseudocode
clears it (`Guy::move@005d9240` settles it, and the code is right); "the
turn arm … is not modelled yet" is stale (`Sim::guy_follow_anim` implements
it); and "`CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT` … which this crate does not
model at all" is stale (`Sim::guy_do_turn_anim`, `guy_flags & 8` gate and
all).

**Two are stated, unimplemented and unreached.** The modern-infantry × 5/4
on the unit's step is stated twice here and implemented nowhere —
`do_move` hands `get_speed`'s answer straight to `unit_step`, and no caller
of `is_modern_infantry` scales a speed; the corpus is openings, so nothing
reaches the predicate's true arm. And a crew guy's step speed is the cached
layer-1/2 value rather than `get_speed`, so it skips the action scale and
the river halving as well; run56's scout and run67's merchant walk plain
ground under a plain move, which is why both their tests pass.

~~**One is stated, unimplemented, and reached**~~ — the out-of-world refusal
cleared the verified-line bit and returned `Did::Nothing` where the original
leaves the bit and returns 1. Unreached in fact: the corpus has no
edge-of-map march. It mattered because `group_move_leader` ungroups a
formation on `Did::Nothing`, so a marching group whose leader steps out of
the world dissolved here and does not there. **Fixed 2026-09-05**, with the
rest of the bit's lifecycle: "The verified line's lifecycle" below.

**And one turned out to be much larger than the row.** The SEAM at
`unit_step`'s tile-permission refusal said of `unit_masks & 8` that "the bit
has no reader this crate models"; `crate::orders` has exactly one, on the
hot path — `do_move` re-paths when it is clear. The trace confirms the
consequence directly: run65 enters `Unit::find_path@005fb910` on sim-frame
6207 and not on 6206, which is the refused step and the re-verification
after it. Then the widening: the dump has printed `unit_masks` all along and
run65's window compared eleven fields of a unit and not that one. Comparing
bit 3 against `Unit::line_ok` over the window gave **335 disagreements in
450 unit-frames**, and the shape was not the refusal at all — from the first
block the original carries the bit *set* on standing units where this crate
carried it clear. **Settled the same day** (item 230): the cause was
`clear_partial_path`, which clears no mask in the original, and the count is
**0 of 450**. "The verified line's lifecycle" below has the seven writers
and what each of the four corrections was.

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
    if (x + sx, y - cy) is outside the world:  return    # try again next frame
    if it lands on another tile and invalid_loc(that tile) != 0:
        unit_masks &= ~8; return                         # the world refuses the step
    move there
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

**`path.flags & 4` is read, since 2026-09-01.** The gate above is the
original's `(manh < slow × 0xc0) || (local_18 & 4)` at `005fb1a5`, where
`local_18` is the fourth word of the **top** of the path stack
(`UnitData +0xb8`, length `+0xc0`, entry `len − 1`) — the waypoint the
step is being taken toward. The bit is `docs/PATHFINDER.md` §7's transport
marker and `Sim::shore_flagged`'s waterline flag, one bit with two
producers (`docs/ORDERS.md` §1.4's `path_flag`), and until this item
`crates/sim` wrote it and nothing read it. So **a unit turns in place
before it crosses the waterline, however far away the waypoint is**, and
a citizen walking to its embark point reached it a frame early without
that. `docs/SYNC.md` §3.25; the guard is
`close_to_the_destination_any_turn_owed_costs_the_frame`'s last two
cases. With an empty stack the original reads slot 0 of it regardless —
whatever the last path left there — and `crates/sim` passes `false`.

**A step that changes tile is asked permission, and a refusal costs the
whole frame** (2026-09-02, run65). `005fb7c1`–`005fb7fd`: the proposed
point's tile is compared against the one the unit is standing on — both
through `div_3_table[v >> 6]`, the same conversion `do_trade` uses — and
where they differ `UnitData::invalid_loc` is called with **all five flags
clear**, the same call `find_path` and `go_around_building` make. A
non-zero answer drops the step entire: no `set_anim`, so the walk is not
even requested; no `set_new_location`, so no move and no reveal; the move
order keeps its waypoint, and `move_step` returns 0 where every other
refusal returns 1. The unit therefore stands where it is, turns another
frame's worth on the next one, and tries the step again from a bearing it
has turned further round.

The consequence is a **turn that costs more frames than the turn rate
says**, and it is what East Indies' word parted on for one item. run65's
caravan owes 38.9° on sim-frame 6206 — inside the 45° gate — so both the
original and this crate compute the same full step to `(38750, 40508)`,
which is one tile north of the tile the caravan is standing on and inside
its own city's footprint. The original drops it, turns 24° more, and
walks on 6207 through a tile it may have. Nothing about the *turn* was
ever wrong: the two sides' bearings, rates and `avg_speed` decay agree
frame for frame either side of it (`docs/CARAVAN.md` §8,
`run65_s_window_is_the_original_s_unit_for_unit`).

### The verified line's lifecycle (2026-09-05, item 230)

`unit_masks & 8` is **the straight line to the current waypoint has been
verified**, and `do_move` reads it at the top of every move frame: with the
bit set it skips `find_path` entirely and goes to the step. It has exactly
seven writers in the whole executable, and the list is the mechanic:

| site | what |
|---|---|
| `Unit::init@00612100:111` | `unit_masks = 0` — the bit is clear at birth |
| `Unit::do_move@005f7b30:428` | **clear**, in the `MoveOrder::dest == 0` arm: the frame a new waypoint is lifted off the path stack |
| `Unit::do_move@005f7b30:580` | **clear**, when `find_path` returned 0 and the stack top is the unit's own position |
| `Unit::do_move@005f7b30:583` | **set**, when `find_path` returned 0 and the top is somewhere else — and the order's `dest_x/dest_y` become that top |
| `Unit::do_move@005f7b30:682` | **set**, the same test after the re-plan's `TAKE` |
| `Unit::find_path@005fb910:366` | **set**, when a `go_around_building` detour verifies and goes back on the stack |
| `Unit::go_around_building@005fc350:352` | **clear**, when the detour gives up |
| `Unit::move_step@005faf30:301` | **clear**, on the tile-permission refusal above — the `& 0xfffffff7` and the `return 0` are the same two lines |

And that is all of them. Grep the export for `0xfffffff7` and for
`unit_masks | 8`: nothing else on a `UnitData` writes bit 3.

**Nothing tears it down.** `Unit::clear_partial_path@005e3920` — the
function every order teardown, `close_orders`, and
`resolve_unit_collision`'s animal give-up calls — frees the suspended
pathfinder search (`UnitData +0x104`, `+0x10c`) and a `Tree<CollBlock *>`
and touches no mask at all. `kill_current_order`, `add_move_facing_order`
and arrival likewise. So a unit that finishes a walk **keeps the bit set**
and carries it, standing, until its next order's first `do_move` clears it
at `:428`.

That is what this crate had wrong, and the size of it was not visible from
the refusal the row named: `Sim::clear_partial_path` cleared
`crate::Unit::line_ok`, so every standing unit here carried the bit clear
where the original carries it set. Comparing bit 3 against `Unit::line_ok`
over run65's window — the widening the docs-versus-code pass asked for —
gave **335 disagreements in 450 unit-frames**; with the four corrections
(the no-op `clear_partial_path`, the clear on the tile refusal, the
out-of-world arm that must leave the bit and answer 1, and the arrival that
leaves the clear to the next frame's `do_move`) it is **0 of 450**, and the
row now rides the labelled comparison in
`run65_s_window_is_the_original_s_unit_for_unit` rather than a pinned
count. It cost neither long word a frame: East Indies stayed at 7448 and
Great Lakes at 6848.

SEAM: `Group::action_unitmask@006fcb90`, reached from
`CommandPackage::process_unitmask`, sets or clears an arbitrary mask over
a group's members from the order stream. Nothing in the corpus sends bit 3
through it, and this crate has no counterpart.

Not modelled, and listed at the end: the flyer branch (`unit_flags & 0x20`,
which turns by the body's rule instead), `detect_unit_collision` /
`resolve_unit_collision`, the rest of the path stack's flags, and the
order angle the unit snaps to on the final waypoint.

## The body step — `Guy::move`

Once per frame for every guy, from `Guy::process`, whether or not the unit is
moving. **Corrected 2026-08-27 (item 34); what an earlier draft had here was
the wrong branch.**

```
if pos == des:                                   # the unit did not move
    last_speed = 0                               # FIRST, and the turn reads it
    if des_angle == angle:                       # settled: the arrival stand
        if cur_anim == CHAR_WALK and stopped:  set_anim(DEFAULT, 0, 1)
        stopped = 1
    elif domain == sea or order is SPECIAL_ANIM:
        stopped = 0                              # turn, but leave the animation
    else:                                        # still coming round: the arm
        if cur_anim not in (TURN_LEFT, TURN_RIGHT, ATTACKWALK):
            set_anim(CHAR_WALK, 0, 1)
        stopped = 0
    turn toward des_angle at turn_speed(mode 1)  # only if not turned this frame
elif guy_num == 0 or (track_dx == 0 and track_dy == 0):
    last_speed = vector_dist(des - pos)          # the jump's own length
    pos = des                                    # written straight onto it
else:                                            # a guy with a track offset
    heading = find_angle(des - pos)
    owed    = turn_towards(facing, heading, turn_speed(mode 1))
    if 2 * turn_speed(mode 1) < owed:  return    # turning costs the frame
    step       = floor(get_speed() * 11 / 8)     # × ai_speed if > 1
    last_speed = step
    if |dx| + |dy| <= step:
        last_speed = vector_dist(dx, dy)
        pos = des                                # Manhattan snap
    else:
        sx = sinx(facing, step); cy = cosx(facing, step)
        if |sx| > |dx|:  sx =  dx                # never overshoot on an axis
        if |cy| > |dy|:  cy = -dy
        if the world accepts (x + sx, y - cy):  move there
avg_speed = (avg_speed * 3 + last_speed) / 4     # skipped on the turn-gate return
```

**Guy 0 never chases.** The `guy_num == 0` test is the first thing after the
walk animation, and it goes straight to the write: no turn, no trig, no
clamps, no 11/8. Whatever the unit's step was, the body is put on top of it,
and `last_speed` is `vector_dist` of that step. An earlier draft read the
third branch, took it for guy 0's, and put the 11/8 here — which is the same
mistake as the draft before it, one field further down.

**So `last_speed` is the unit's own Euclidean speed**, and this is the whole
of why the body is modelled. The number that comes out is not the same one a
chase would give. run10's AI scout walks a diagonal at `MOVES` 34, stepping
`(24, 24)`: `vector_dist(24, 24)` is 36 — the octagonal measure, `hi + lo²/2hi`
— and `(3a + 36) / 4` truncates to a fixpoint of **33**, so the turn rate is
divided by `33/4 + 1 = 9`. An 11/8 chase would have given 46, an average of
43 and a divisor of 11. The two differ by 22% on every turning frame, and the
9 is the one that reproduces the original's own `0x222221c` on frame 62.

**And a unit that did not move has `last_speed` zero the same frame.** Turn in
place, arrive, be refused a step by the world — the body is already on the
unit, the first branch runs, and the *next* frame's unit step finds
`last_speed == 0`. For a foot or mounted type that is the instant turn
(below), which is why a unit that stops to turn pays exactly one frame for it
and not seven.

**And so does the turn at the foot of the same branch** — that is the order
above, and it is load-bearing. `last_speed = 0` is written at `:53`, ahead of
everything else in the at-des branch, and the `turn_towards` at `:99` reads
the zero it just wrote. **A standing body therefore swallows whatever turn it
is owed in one frame**, however slowly its type turns, because
`GuyData::turn_speed@005de340:29` answers a zero `last_speed` on a foot or
mounted guy with `0x80000000`. Not the next frame: this one. This crate read
`last_speed` as it stood before the frame until 2026-08-30, so a slow turner
came round at its rate — five degrees a frame for a chicken — and run10's AI
scout was a frame behind the original on the three frames after an arrival
(96, 362, 721), which was queue item 37. Diff-backed: those three rows are
gone and the compared population grew from 33,992 to 35,868
(`rondata::diff`'s `run10_s_opening_trains_the_original_s_citizens_on_its_frames`).

**The two arms above the turn are `Guy::move`'s own `set_anim` callers**, and
the second of them — the **turn arm** — is not modelled yet. A body that has
arrived but has not yet come round to the order's angle is put *back on the
walk* and marked unstopped, every frame it is still turning, so the next
frame's idle request sees the walk category and rolls again. That is the
second draw of `docs/SYNC.md` §3.11's arrival pair. What it costs to land is
in the queue: the arm reads `des_angle != angle` on every standing unit, and
`Guy::do_turn` overrides the walk with `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT`
whenever the guy's piece has a turn animation (`guy_flags & 8`,
`Guy::init_real@005db6b0:179`), which this crate does not model at all.

**The 11/8 is the tracked guy's**, and nothing else's. A literal, with no
constant behind it. `track_dx`/`track_dy` are what the branch is gated on;
what sets them is unread, and the simulation has only guy 0, so the branch is
read and not modelled.

**And that unmodelled branch is now the headline's own divergence** — East
Indies' word, 2665 of run54/run56 (2026-09-01). A scout is **two guys**: the
man and his dog, `Unit::set_anim`'s two loops (`0..guy_mark` at `+0x56`,
`squad_size..num_guys` at `+0xb6`), and the trace tells them apart by that
offset. run56's per-frame `UNITDATA` prints each `GUY`'s own `x`, `y` and
`angle`, and around the scout's arrival they are **not the unit's**:

| frame | guy 0 | dog | `+0x56` draw | `+0xb6` draw |
| --- | --- | --- | --- | --- |
| 2663 | (40431, 34293), turning | (40382, 34388), turning | — | — |
| 2664 | (40416, 34272) — **arrived** | (40365, 34367), still walking | yes | yes |
| 2665 | angle == `des_angle` at last | (40352, 34323) | yes | — |
| 2666 | standing | (40339, 34279) | — | — |
| 2667 | standing | (40326, 34235) | — | — |
| 2668 | standing | (40322, 34217) — **arrived** | — | yes |
| 2669 | standing | angle == `des_angle` at last | — | yes |

So the dog walks four frames past the unit's own arrival, on its own body,
at its own speed, and comes round to its own angle four frames after guy 0
does. Every one of those frames is a `Guy::set_anim` decision the unit's body
cannot make: `set_anim`'s early return for a walking guy tests **that guy's**
`des` against **that guy's** position, and the turn arm above tests that
guy's `angle` against that guy's `des_angle`.

**Landed 2026-09-01, and the word moved 2665 → 3021.** `crates/sim` gives a
tracked crew guy a body of its own — `anim::Follow`, `movement::follower_des`
and `movement::follower_step` — and the section below is what it is built
from. Every figure of every player's unit in run56 now stands where the
original's does on all 3,000 frames, guy 0 and dog alike
(`rondata::diff`'s `run56_s_figures_stand_where_the_original_s_do`,
1,062,354 fields).

## The follower's destination

A crew guy does not chase its leader: it is **told** a point, and the point is
rewritten by the leader. Three things about it were unread until now, and each
was load-bearing.

### Where the track offset comes from

`GuyData::track_dx / track_dy` are `+0x54` and `+0x58` — not `+0x92`/`+0x94`,
which are `off_x`/`off_y` and are a different field entirely (below). They are
**art data**, and the only writer is `Guy::update_gpiece@005d8530`:

```
if guy_num != 0 and piece.unit_data != null:
    track_dx = (int)(unit_data.track_offsetx * guy_scale * piece.scale)
    track_dy = (int)(guy_scale * unit_data.track_offsety * piece.scale)
else:
    track_dx = track_dy = 0
```

`UnitRDataStruct::track_offsetx / track_offsety` (`+0xc` / `+0x10`) and
`RData::scale` (`+0x88`) are `unit_graphics.xml`'s own `trackoffsetx`,
`trackoffsety` and `scale`, and `guy_scale` is the executable's `float` at
`00c06244` — `Guy.obj`'s only exported datum in `rise_z.map`, **4.8** in
`.data`, written nowhere but three `ConsoleWin::run_cmd` arms. Every scout's
dog in the shipped data writes `trackoffsetx="-20" trackoffsety="10"
scale="1"`, so its pair is `(-96, 48)`.

**Guy 0 never has one**, because the function's first test is `guy_num != 0`.
Neither does a crew guy whose entry names no offset: `ADVMACHINEGUN`'s loader
carries `trackoffsetx="0"` in its unpacked entry and `-25`/`-2` in its packed
one, which is the same pair of branches by another name. 203 of the install's
pieces name one at all.

The multiply is floating point in the original and floating point at **load**
here — `rondata::artdata::piece_tracks`, which hands the simulation integers
(`CLAUDE.md`, "no floating point in the sim"; `DECISIONS.md` entry 16's
"pinned table" arm).

### The rotation, and its axes

Two functions write a crew guy's `des_x`/`des_y` (`+0x5c`/`+0x60`), and both
end in the same arithmetic. Given a leader position `(lx, ly)` and an angle
`a`:

```
des_y = ly + sinx(a, track_dx) + sinx(a + 0x40000000, track_dy)
des_x = lx + sinx(a + 0x40000000, track_dx) + sinx(a + 0x80000000, track_dy)
then clamp each axis into 0 ..= world_dimension * 0x300 - 1
```

with `sinx(a, d)` the folded quarter-wave of "The sine table" above. **The
axes are not the step's.** A movement step adds `sinx` to `x` and subtracts
`cosx` from `y`; this adds the first component to `y` and the second to `x`,
which is the same vector turned a quarter further round. So `track_dx` is a
*lateral* offset and `track_dy` a *longitudinal* one — the field names are the
ground-track art's, and the follow reuses them.

The decompiler prints both call pairs as `sin_table(unaff_ESI, unaff_EDI)`,
so this was read from the listing (`005d88f3–005d894d` and
`005d90bc–005d9192`) — the audit README's rule about `unaff_`, again.

The clamp runs only when at least one component is non-zero, which is the only
case either writer is reached for.

### Who writes it, and when

| writer | reached from | leader position | angle |
| --- | --- | --- | --- |
| `Guy::set_new_location@005d86f0` | `Guy::move`'s guy-0 snap | guy 0's **new** position | guy 0's `angle` (the facing) |
| `Guy::set_angle@005d9010` | `Guy::do_turn`, and so from every `turn_towards` | guy 0's current position | the angle just turned to |
| `Guy::set_angle@005d9010` | `Unit::set_angle@00605400`, at the **top** of every `move_step` | guy 0's current position | the **heading** |
| `Guy::set_new_location@005d86f0`, with the **snap flag** | `Unit::set_new_location(·, ·, 1, ·)` — `Unit::init`'s seating, `resolve_unit_collision`'s cell-centre snap, and `Unit::do_cast`'s re-seat of a rare collector (`docs/ORDERS.md` §6.9 step 2) | the point being snapped to | the facing — and the crew is *put* there |

The loop is the same eight lines in both functions, and the `track != 0` test
inside it gates only the **rotation**: `des_angle` and the base `des` are
written for every figure past `squad_size`, tracked or not. So a trackless
crew figure carries guy 0's own point and guy 0's facing, and guy 0 itself
carries the heading — which is what `docs/MOVEMENT.md`'s "Two angles" calls
"the same value again".

The last writer before the crew's own `Guy::move` is the one that counts, and
`Unit::process` runs `Guy::process` for the squad first and the crew after
(`00610bc0:540–551`), so:

- **guy 0 moved**: `Guy::set_new_location`, from the new position and the
  facing.
- **guy 0 stood and was still owed a turn**: `Guy::do_turn`'s, from the same
  position and the facing it has just reached. `turn_towards` calls `do_turn`
  whether or not the angle actually moved, so this fires every such frame.
- **guy 0 stood and was settled**: *nothing*. `Guy::move:55`'s `des_angle ==
  angle` arm jumps straight to the average, past the turn — so the crew keeps
  the point it was last given and walks on toward it.

That third row is the whole of item 128. Guy 0's position is the unit's own in
both live cases, so `crates/sim` writes it as one expression gated on
`!was_at_des || !facing_settled`.

~~`Unit::set_angle`'s row is the residue: it rewrites the crew's point with
the *heading* rather than the facing, and on the frames that matter one of
the other two overwrites it. Not modelled.~~ **Modelled 2026-09-02, and it
was not a residue** — it is the row a collision reads. `move_step` calls
`Unit::set_angle` at its **top**, before the collision block, so on every
frame the bearing moves at all, a crew figure that walked exactly onto its
destination last frame is off it again by the time the blocked stand asks it
to idle. `Guy::set_anim`'s walking-guy early return (`des != pos`,
`docs/ANIM.md` §4 step 1) then takes it and it does not roll. East Indies
6571 is the measurement: 720,896 of a turn — one frame's worth of bearing —
moves a `(-48, -192)` track by one unit on each axis, and the merchant's
crew figure spent a draw here that the original does not.

**And the snap teleports the crew.** `Unit::set_new_location`'s `param_3`
does not stop at guy 0: it is handed on as `Guy::set_new_location(guy 0, pos,
1)`, whose crew loop finishes each figure with `set_angle(crew, des_angle,
1)` and `set_new_location(crew, des, 1)` — the facing written outright and
the body placed on the offset, rather than left to walk after the leader.
run67's block 6572 is the record: the merchant is snapped to its cell centre
and its figure is on (34464, 37595), the rotation of its track about that
centre, with `angle` equal to the driver's to the digit.

Both are `Sim::crew_des`, whose `snap` argument is the callers' `param_3`,
and both are pinned by `run67_s_window_is_every_figure_s_whole_record` —
now **32,742** fields, every `GuyData` the dump prints, over sixty frames.

**And a snap onto the point the unit already holds still teleports the
crew.** `Unit::set_new_location@005f8d20` has no early return for an
unchanged position: the same point makes both of its cell tests false, so
it falls through to `LAB_005f9033`, writes the coordinates back, and runs
the guy half anyway — the crew loop included. `crates/sim` had exactly such
an early return, and it cost Great Lakes' word 312 frames (item 210). The
capture is unambiguous: run75's block 6145 has the Merchant `1/24`'s crew
figure at `(40706, 14716)` with `angle 1605566464`, its driver's to the
digit, on the frame its driver — standing on its cell centre already, so
`from == to` — begins its unpack; this crate had it four frames behind and
still walking.

### A crew guy's turn rate is a quarter turn, flat

`GuyData::turn_speed@005de340` is not one formula but two, and the whole of
"Turning" below is the **first** half:

```
rate = 0x40000000
if guy_num < type.squad_size:
    rate = (type.turn_speed >> 8) * UNIT_TURN_SPEED
    if packed:  rate *= UNIT_PACK_TURN_BONUS
else:
    if track_dx != 0:  return 0x40000000
    if track_dy != 0:  return 0x40000000
if last_speed == 0 and (guy_flags & 0x10):  return 0x80000000
...
```

So a tracked crew guy returns **ninety degrees** before the instant-from-a-stop
test and before either mode. Two consequences, and run54's frame 412 shows
both: the dog comes round from `1681129472` to `-1540096000` in one frame —
exactly a quarter turn — where the man it follows manages the scout type's
twenty-seven degrees; and it **never gives a frame up to turning**, because
`Guy::move` abandons the step when `2 × rate < owed` and `owed` is
`|delta| − rate`, which wants a shortest turn past 270°.

### `off_x` and `off_y` are always zero

`Guy::set_anim`'s walking-guy early return tests `des_x != x - off_x || des_y
!= y - off_y` (`005da300:111`, `:163`), and `Guy::move`'s tracked branch
subtracts the same pair from the position before adding the step
(`005d9680–5d9691`). Both are `GuyData +0x92` / `+0x94`, and
**`Guy::clear@005db590:49` is the only writer in the executable** — one
`undefined4` of zero, at construction. Grepped, per the audit README's "grep
the writers of every field you call frozen". So the test is `des == pos` and
the subtraction is a no-op, which is what `crates/sim` implements.

### And the crew that has no track

`Guy::do_turn@005d97a0:37` recurses into the crew **only** where both track
components are zero, and such a guy's destination is its leader's position
exactly. So a trackless crew guy has guy 0's position, guy 0's angle and guy
0's arrival test on every frame: it *is* guy 0's body, and `crates/sim` leaves
it sharing one rather than modelling a second that could only ever agree.

**The idle body's turn.** Standing on its destination and not turned this
frame by the unit step (`guy_flags & 2`, which `do_turn` sets), guy 0 turns
its own facing toward `des_angle` — the heading, which `Unit::set_angle` last
wrote. `turn_towards` uses `turn_speed(mode 1)`, and the instant-turn test
sits *before* the mode test, so a stopped foot unit snaps its body onto the
heading in one frame. That is how a unit that has arrived comes around to its
order's angle.

`GuyData::get_speed` is `UnitData::get_speed(body x, body y, 1)` — the body's
tile, and **flag 1, so the group cap never applies to the body** — plus nine
when the current order's vslot `0x2c` is set. `crates/sim` feeds the body the
same speed input as the unit; the `+9` and the cap difference are not modelled.

**And the crew jogs.** `Guy::move`'s tracked branch pays a figure
`(get_speed * 11) / 8` a frame — the sign-corrected `>> 3` at `005d9600` —
so a figure keeping station behind a leader that walks at its base speed
averages eleven eighths of it. `Guy::set_anim`'s walk arm divides **the
asked guy's own** `avg_speed` (`this->field_0x84` at `005db438`, not guy
0's) by `moves * UNIT_MOVE_SPEED` and jogs above eleven tenths, so a tracked
crew figure plays `CHAR_JOG` where its own leader plays `CHAR_WALK`.
run67's merchant is `cur_anim 9` beside its driver's 8 on every block of the
window. It is not cosmetic: `Guy::move`'s arrival arm tests the **slot**
(`cur_anim == CHAR_WALK`, `docs/ANIM.md` §4.6), so a figure read as walking
spends an idle roll on every arrival that a jogging one does not.

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

Objmask `0x20` and `0x1000` are bits 5 and 12, `FOOT` and `MOUNTED`, so the
predicate is the data file's own note with two riders. `unit_flags & 0x10`
(`FLAGS e`, "this flag is set in the program" — a sea transport) lets a type
in without either mask; `unit_flags & 2` keeps one out, and its legend is
**"Unit is a horse-drawn cart type thing"**. A cart does not pivot. And a
packing type (`unit_flags2 & 4`) takes the `0x8` branch instead and never gets
`0x10` at all.

**Nothing populated that flag until 2026-08-27**, which is the other half of
item 34: `crates/sim` had the rule and the rate, and every unit built from the
data came out with the flag clear, so no unit in the simulation had ever
turned instantly. `sim::turning_of` derives the whole `Turning` from the type
now — the rate, this flag and the 80° `wide_limit` — and the training site and
both of `rondata::diff`'s scene builders go through it.

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

Runs when something changes — `Leader::calc_unit_stats@006cf970`, off
`LeaderData`'s `0x4000000`, which `Leader::process` clears in the same frame it
is raised — and writes a cached value on the unit. One arm of it is landed
(2026-09-02): the **whales** bonus, `+WHALES_SHIPS_MOVE%` for an objmask
`0x2000` (`NAVAL`) type whose owner holds rare bit 25, which run63 measures as
`myspeed` 38 → 45 on three Fishermen and 25 → 30 on a Transport Barge in one
frame (`docs/ECONOMY.md`, "What an owned rare does"; `crates/sim/src/rares.rs`).
The rest of the pipeline below is still an input. From the type's move value: the transport bonus (`MILITARY_TRANSPORT_BONUS`, 0) and the
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
  earlier draft said damaged). Standing on a **tile** carrying flag `0x800`
  — `crate::world::tile::RIVER` — halves it again, **but only with
  `z_internal <= 0`**: the river bed, not the bank (see "The river halves a
  land unit's step" below);
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

### The river halves a land unit's step (2026-09-04, item 208)

The tile half of layer 3 is `0060879b`–`006087d8`:

```
if ((z_internal ^ 0x63637) > 0) goto skip;              // 0060879b
tile = div_3_table[y >> 6] * world.tile_xs + div_3_table[x >> 6];
if ((world.tdata[tile].mask & 0x800) == 0) goto skip;   // 006087d3
speed /= 2;
```

Three things in that are worth naming, and one of them nearly cost a wrong
reading.

- **The `^ 0x63637` is not a predicate: the coordinates are stored
  obfuscated.** The same constant appears on `x_internal` and `y_internal`
  in the `+0x2f` wrapper `UnitData::get_speed@006086f0`, which only forwards
  them to the virtual, and on all three in
  `SubObjectData::log_data@00661f80`, which only prints them. The listing at
  `00608705` is `xorl $0x63637, %eax` **before the push**, so every reader
  decodes and `SubObject::set_new_location@00662680` re-encodes on the way
  in. Taken as a guard, `(z ^ 0x63637) > 0` skipping the halving would read
  as `z < 0` and the mechanic would never fire; decoded, the predicate is
  plainly **`z > 0` skips it**, which is what the dump shows.
- **`z_internal` is the tile's own height, not a separate field to carry.**
  `set_new_location` writes it as `TerrainOut::find_tcoord_z` of the tile it
  has just moved to, which is `World::tile_z` here, so the stored `z` and
  the tile the mask is read from are always the same tile
  (`Sim::on_river`). A world with no height grid reads zero everywhere and
  passes the `z` test, which is the flat harness world.
- **It is the `TData` mask, `tdata` at `world +0x138` with stride 2** — the
  same array and the same `div_3_table` shape `docs/SCOUT.md` §7 reads the
  surface out of.

**The diff that says so.** run75 is run53's game with the cheap per-frame
window on `[5845, 6160)`, taken for Great Lakes' word at 6080. The AI scout
`1/0` walks the river south of its second city: its dumped `z_internal` is
14 on 5948, **0** on every frame of 5949–6089 and 17 on 6090, and its
per-frame step is 34 outside that span and **17** inside it on a `myspeed`
of 34 throughout. Both halves are pinned as a `#[test]` that was made to
fail twice —
`movement::tests::a_land_unit_on_a_river_tile_at_or_below_the_waterline_walks_at_half`
— and the window as a whole in
`rondata::diff`'s `run75_s_window_is_the_scout_s_walk_down_the_river`: 315
frames, every unit, nobody off the original's point. Great Lakes' word
**6080 → 6151**.

**What of layer 3 is still not here.** `crate::Sim::get_speed` now carries
the order scale and this halving; three arms remain seams, each for want of
an input rather than a reading. `unit_masks & 0x10` is set and cleared
inside one frame so no dump can print it, and this crate's
`target_opportunity` is the retaliation alone. `has_general(0, 0x162)`'s
siege doubling has no general in any capture. The group cap has its
arithmetic (`movement::group_capped`, made to fail once) and no group speed
to feed it.

## The animal's own `get_speed` (2026-08-30, item 95)

**Everything above is `UnitData::get_speed`, and an animal never runs a line
of it.** `get_speed(x, y, flag)` is a virtual — slot `+0x17c`, and both
`Unit::do_move@005f7b30` and `Unit::find_path@005fb910` take the step length
from it. `Animal`'s and `AnimalData`'s vtables carry
**`AnimalData::get_speed@005d8380`** in that slot, and it calls
`UnitData::speed` (layer 2, the auras) and nothing else from the pipeline:

```
speed = UnitData::speed()
if type.domain == 2:  return speed             # air: no hurry, no floor
order = get_current_order()
if order and order->is_move():                 # slot +0x14
    m = order->get_move_order()                # slot +0xb8
    if vector_dist(|x − m->x|, |y − m->y|) > 0x180:
        speed = speed * 3 / 2                  # truncated toward zero
return max(speed, 3)
```

So for a herd animal or a pasture's, none of layer 3 applies: not the order
scale, not the `unit_masks & 0x10` halving, not the `0x800` tile, not the
group cap. What `do_move` applies *after* the virtual — `ai_speed`, and the
modern-infantry `5/4` — still does.

**The three things a fresh reading gets wrong here.**

- **It is the order's goal, not the waypoint.** `get_move_order` then
  `MoveOrder +0x4/+0x8`, the point the order was given at — not `+0x2c/+0x30`
  `dest_x/dest_y`, which the pathfinder overwrites with the current leg. An
  animal on the last leg of a long walk is still hurrying.
- **It is not the herd centre.** `0x180` is the same number
  `Animal::do_idle` measures its wander with — the near branch is
  `vector_dist(centre − here) < 0x181` — but that one is measured from the
  herd's centre and this one from the order. They agree on the constant and
  on nothing else.
- **The two vtable slots are COMDAT-folded in the map**, so the export names
  `+0x14` and `+0xb8` after whatever trivial stub they were merged with
  (`Window::get_button`, `StrafeOrder::is_air`). The PDB's `LF_ONEMETHOD`
  list carries the real `vftable offset` for each: **20 is `is_move`, 184 is
  `get_move_order`.**

**Which walks it fires on.** `Animal::do_idle`'s **near** branch offsets the
animal by at most `4 × 1 × 0x30 = 192` on each axis, so its `vector_dist` is
at most 288 and a near wander never hurries. The **far** branch —
`UnitType::find_nearby_spot` around the herd centre at `0xc0`, swept from
the literal bearing `0x55555555` rather than the animal's facing
(`docs/SYNC.md` §3.19) — routinely
lands beyond `0x180`, so the hurry is the far branch's, and it lasts until
the animal has closed to `0x180`. That is why an animal's walk *decelerates*
partway through, which is what it looks like in a dump and is not what it is.

**How it was established, and how confident this is.** Diff-backed on both
captures, and the arithmetic has no free parameter.
`rondata::diff::an_animal_more_than_0x180_from_its_order_hurries_by_three_halves`
re-derives `move_step`'s whole step from `myspeed`, `orders_x/orders_y` and
the position the dump prints, for **every gaia unit-frame on which an animal
moved** in run39 and run33: **264 steps, 39 of them beyond `0x180`, and 264
predicted exactly**. Drop the `3/2` and exactly those 39 break; move the
threshold to `0x200` and 27 break; make the ratio `4/3` and all 39 break.

The base is the dump's own: run39's `8/2` prints `myspeed 19` on every frame
of the walk, including the two whose step is 28 long, so the `3/2` is applied
strictly after `UnitData::speed` and is not a different cached value. Great
Lakes' herd prints `myspeed 11`, so the ratio is exercised against two bases.

**Not established.** Whether `is_move` is true for any order this crate does
not model as a `Body::Move` — `AttackToOrder`, `ExploreToOrder` and
`FleeToOrder` all embed a `MoveOrder` and would answer the `+0xb8` — since no
capture has an animal carrying one. And `AnimalData::get_speed`'s
domain-2 arm returns **before** the floor of 3, so a bird with a cached speed
below 3 would keep it; no bird in any capture is that slow, and this crate has
no `AirOrder` to walk one with (`docs/SYNC.md` §3.9).

---


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

~~**And it got mixed up again, in the other direction** (2026-08-26).~~
Closed 2026-08-27 from the listing: the decompiled branch is real, the
mirror is inlined in every sim caller, and the branch is reached only from
`MapGrass::make_continents`. See "The mirror is the original's" under "The
sine table".

~~**What the type's own stored turn speed means.**~~ Degrees in the file,
`degrees_to_angle` at load, a binary angle in memory; and `UNIT_TURN_SPEED` is
256. See "Turning".

~~**Whether `ai_speed` belongs to the simulation.**~~ `Game::init_data` sets it
to 1 and only a cheat changes it. It multiplies both steps and is part of the
synchronised state by construction; `crates/sim` treats it as 1.

~~**Which field is the facing and which the heading.**~~ Settled 2026-08-27,
item 34, and settled by a *check* rather than a reading: `UnitData::angle`
against the heading and guy 0's `angle` against the facing, on every
unit-frame where the two sides agree on the position. See "Two angles" and
the checks below.

**Still open:**

- ~~**The arrival frame's facing, and it is worth one grep.**~~ **Closed
  2026-08-30**, and it was not `guy_flags & 2` at all: `Guy::move` writes
  `last_speed = 0` at the head of the at-des branch, ahead of the
  `turn_towards` at its foot, and a zero `last_speed` is the instant turn —
  so the body comes round in one frame however slowly its type turns. "The
  body step" above has it; run10's three scout rows are gone.
- ~~**What world tile flag `0x800` is.**~~ **Closed 2026-09-04**, by a named
  accessor rather than by inference: `WorldData::is_river@0046d390` is
  `tdata[tile_xs·y + x].mask & 0x800` and nothing else. It is a **river**,
  and run75 is the run that watches a scout wade one at half speed — "The
  river halves a land unit's step" above.
- **The flyer branch of `move_step`.** A type with `unit_flags & 0x20` owing
  less than a quarter turn turns by `do_turn` and then again by the body's
  rate, with no turn-in-place and no half step. Read, not modelled: the
  simulation has no aircraft.
- **Pathfinding.** `PathFinder`, `PathNode`, a `BRTree` and a `Stack<PathData>`
  per unit. `move_step` takes the top of the stack as its waypoint and pops it
  on arrival; `PathData.flags & 4` forces turn-in-place and `& 2` allows a
  final snap through a collision; `& 1` marks the last waypoint. Entirely
  unread beyond that.
- ~~**Collision and pushing.** … Unread.~~ **Read and modelled**:
  `docs/COLLISION.md` and `crates/sim/src/collide.rs`. `detect_unit_collision`
  runs on every proposed step, `resolve_unit_collision` when it fails,
  `UnitData::tolerance` is set to `2 × manh` when the unit gives up, and the
  `0x100000` one-shot half step is collision's — the last of those is the only
  part still unmodelled (§7 there). `PUSH_SIZE` and `PUSH_CIRCLES` in
  `unitrules.xml` remain unread; the occupancy profile the mechanic actually
  uses is `BLOCK_RADIUS`.
- **Formations and groups.** `Group::compute_speed`, `report_speed` and
  `leader_report_speed` fill the group speed the cap reads. Unread.
- **The order layer.** `MoveOrder`, `PatrolOrder`, `AttackToOrder` and the rest
  sit above all of this and decide what the destination is — and the angle the
  unit and its body snap to on the last waypoint.
- **The body's `+9`.** `GuyData::get_speed` adds nine when the current order's
  vslot `0x2c` is non-zero. Which orders, and why nine, is unread. It is a
  *crew* guy's speed now that one takes steps of its own, so this is no
  longer inert — run56 does not reach it, and the day a capture does the
  crew's step will be nine short.
- ~~**`track_dx` / `track_dy`.**~~ **Closed 2026-09-01.** They are `+0x54` and
  `+0x58`, they are art (`Guy::update_gpiece`), and the follow they drive is
  "The follower's destination" above.
- **`Unit::set_angle`'s own crew write.** `Guy::set_angle` is reached from
  `Unit::set_angle@00605400` as well as from `do_turn`, and rewrites the crew's
  destination with the **heading** rather than the facing. On every frame run56
  reaches, one of the other two writers overwrites it, so it is not modelled;
  a capture where a standing settled unit has its heading set by an order
  alone would tell the two apart.
- **The crew's own `stopped` and animation, past the walk.** `Guy::move`'s
  moving arm asks for `UVar4` — the carrying walks, `CHAR_ATTACKWALK` for a
  guy whose unit has a `cavarch_o` — under a guard on `guy_flags & 0x40` and
  `type+0x2b8 & 4`. `crates/sim` applies the unit's walk to the crew
  unconditionally. Nothing in the corpus separates them yet.
- **How many figures a unit is made of.** `Sim::init_guys` still makes one,
  so a unit this simulation *trains* has no crew and spends one
  `Guy::init_real` draw where the original spends `num_guys`. Every crew guy
  in the corpus comes from a dump. `UnitType +0x304` is `squad_size` and
  `UnitData +0xe8` the total; what fills the gap between them is unread.

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

**D2 was half wrong and both readings missed it (2026-08-27, item 34).** The
11/8 is not the unit's — that much the audit fixed — but it is not guy 0's
either: `Guy::move` tests `guy_num == 0` before it reaches the turn, and takes
a branch that writes the body onto its destination and records `vector_dist`.
Two independent readers went past that test on the way to the arithmetic
underneath it, which is the recurring shape in `docs/audit/README.md`: the
predicates are where the errors are, not the arithmetic. What found it was
not a third reading. It was the first frame of a differential check on a
field the dump had been printing all along.

## The checks (2026-08-27, item 34)

Three, all differential, all in `rondata::diff`:

- **`UnitData::angle` and guy 0's `angle`, on every unit-frame where the two
  sides agree on the position.** The gate matters: a unit that walked
  somewhere else points somewhere else as a consequence, and counting that
  measures the position gap twice. run20's opening is **72 comparisons, zero
  disagreements**; run10's 1,772 frames are 13,542 comparisons and 5,435
  disagreements, and none of the residue is the step's. ~~It is
  `Unit::set_angle`'s other seventeen callers, which the simulation does not
  make.~~ **It was not** (2026-08-30, item 36): it was two predicates in code
  this crate already had — `add_move_order` taking the angle to the *snapped*
  destination rather than to the point it was handed, and `move_step`'s
  gather clause applied to both arrival arms rather than to the Manhattan
  snap alone. `docs/SYNC.md` §3.12; **8,969 of 33,992 → 1,227 of 35,868**,
  and every farmer left the residue.
- **The AI scout in run10, `1/0`**: two rows in the whole run, both after an
  arrival — ~~see the open question~~ **none since 2026-08-30**, the standing
  body's instant turn. Frames 57 to 91 — the case item 34 was opened on — are
  exact on position and on both angles.
- **run13's window, frame 102**: the `think_scout` ring draws, `22 / 6`
  before and `6 / 6` after.

Five deliberate breakages, all red: the instant-turn flag forced off (run13's
102 goes to `27 / 8`), `last_speed` set to the 11/8 step (run10's comparison
count falls, and the unit test on the fixpoint fails outright), the initial
angle put back to north (run20's zero becomes 64), the heading compared
against the facing, and the step's `set_angle` given the facing instead of the
heading (both raise run10's count).

## The fourth check (2026-09-01, item 128) — every figure, every frame

`run56_s_figures_stand_where_the_original_s_do` compares the whole of the
per-frame `GUY` record — each figure's own `x`, `y` and `angle` — over East
Indies' 3,000-frame capture, for every unit of every owner and every figure of
every unit. **1,062,354 fields.** Nothing in it is installed: guy 0's body is
the unit's own, and a crew guy's is derived from its piece's track offset and
from the two writers above, so every row is a prediction the dump can refuse.

Three tallies come out, and only one is a residue of this mechanic:

- **Every player figure agrees, on every frame.** Man and dog alike.
- 301,810 rows are gaia's, and all of them are the **angle alone** — an
  animal's spawn bearing, which this simulation does not derive.
  `Sim::reseat_animal` puts the position back every traced frame, so the place
  agrees and the angle never does. Asserted at its number.
- One row is on the capture's own last frame, which every widening here
  exempts: the dump is written before the rest of that frame runs.

It was made to fail on purpose first, by handing the crew guy no track: two
figures part on **frame 0**, before a single tick, because a dog seated on its
man is already 109 units from where the original's stands.

The score it moved is East Indies' long-capture word, **2665 → 3021**
(`rondata::diff::tests::LONG_WORD_EAST_INDIES`).
