# MOVEMENT.md versus `crates/sim` — 2026-09-05

Reader: Opus, docs-versus-code pass (not a blind reading: this reader saw the
document and the implementation, and nothing else).

Checked: 96 stated rules across the document's thirteen rules sections — "The
unit of speed" through "The animal's own `get_speed`". Rows below are the ones
where the code and the document disagree. Where a row turned on what the
original actually does, the decompile export was opened and the verdict is
stated; three rows below are the *document* being wrong and the code right.

## Rows

### R1 — the modern-infantry ×5/4 on the unit's step is stated twice and implemented nowhere

| | |
|---|---|
| document | MOVEMENT.md "The unit step", `:341` — "a step already computed by `do_move` — `get_speed(x, y, 0)`, × `ai_speed` if that is above 1, and × 5/4 for modern infantry"; restated at `:963`–`966` |
| code | `crates/sim/src/orders.rs:1768` (`fn do_move`), `crates/sim/src/orders.rs:1941` |
| document says | `do_move` scales `get_speed`'s answer by 5/4 for a `UnitData::is_modern_infantry` type before handing it to `move_step` |
| code does | `let speed = self.get_speed(u);` and then `self.unit_step(u, mo, speed)` — nothing between them. No caller of `is_modern_infantry` scales a speed |
| difference shows | a modern-infantry unit's per-frame `x_internal`/`y_internal` delta: 4/5 of the original's on every walking frame, so the two sides part on the first frame such a unit walks, and `myspeed` in the same `UNITDATA` record would agree while the step did not — which is the signature of a `do_move` scale rather than a pipeline one. **UNSURE** whether any capture reaches the true arm: the corpus is openings and the predicate wants `unit_flags & 0x100` at age 6 or later |
| reached | reached — `UnitData::is_modern_infantry@00607b40` is cited by address in `docs/audit/2026-08-20-movement.md` and is **not** on the blind list, so traced games do enter it. That is expected: `do_move` asks it on every step. What no capture is known to reach is its **true** arm |

`crates/sim/src/movement.rs:490`–`492` repeats the claim in `move_step`'s own
docstring: "`step` is what `do_move` hands in: `get_speed` for the unit's
square, times the modern-infantry 5/4 where it applies." The predicate itself
exists — `Sim::is_modern_infantry` at `crates/sim/src/form.rs:417`, `unit_flags
& 0x100` and `combat.age > 5` — and its only three callers are formation
geometry (`crates/sim/src/form.rs:369`, `crates/sim/src/group.rs:891`,
`crates/sim/src/grouppath.rs:161`). The speed path never asks it. `ai_speed`,
the other half of the same sentence, is a stated seam in the document's open
questions ("`crates/sim` treats it as 1") and so is not a row; the 5/4 is not.

### R2 — the body step's sea / `SPECIAL_ANIM` arm sets `stopped`, and the document's pseudocode has it clearing it

| | |
|---|---|
| document | MOVEMENT.md "The body step", `:477`–`478` — "`elif domain == sea or order is SPECIAL_ANIM:` / `stopped = 0   # turn, but leave the animation`" |
| code | `crates/sim/src/anim.rs:1273`–`1276` (`fn guy_follow_anim`) |
| document says | a sea unit standing on its destination but still owed a turn comes out of the arm **unstopped** |
| code does | `if … Domain::Sea { self.units[u].guys[g].stopped = true; return; }` — marks it **stopped** |
| difference shows | no sim divergence: the code is right and the document is wrong. Settled against `Guy::move@005d9240` in the decompile — the arm is `if ((*(int *)(… + 0x218) == 1) || (OVar3 = UnitData::order_type(this_00), OVar3 == SPECIAL_ANIM)) { this->field_0x9d = 1; }`, and `field_0x9d` is the same byte the settled arm writes 1 and the walk arm writes 0. Were the document followed, a ship arriving and still turning would re-request `CHAR_DEFAULT` on the frame after (the arrival's `anim == WALK && stopped` gate), spending an idle roll and a `Guy::set_anim` draw the original does not — visible in a `UNITS`/trace window on any docked-ship arrival (run22's dock window) |
| reached | reached (`Guy::move@005d9240` is not on the blind list) |

The document is the only artefact carrying the error; fix the pseudocode line,
not the code. The same pseudocode also elides `GuyData +0x9e` (the pending
special-animation byte) from both arms of the at-des branch, which the
decompile shows guarding each `set_anim` — that is `docs/ANIM.md`'s to own, and
is noted here only so a later pass does not read the omission as a code gap.

### R3 — a crew guy's step speed skips layer 3 entirely

| | |
|---|---|
| document | MOVEMENT.md `:777`–`780` — "`GuyData::get_speed` is `UnitData::get_speed(body x, body y, 1)` — the body's tile, and **flag 1, so the group cap never applies to the body** — plus nine … `crates/sim` feeds the body the same speed input as the unit; the `+9` and the cap difference are not modelled" |
| code | `crates/sim/src/lib.rs:3184` (`fn process_follower`) → `crates/sim/src/movement.rs:803` |
| document says | the difference between the body's speed and the unit's is exactly two things: the `+9` and the group cap |
| code does | `let speed = self.units[i].movement.speed;` — the **cached** layer-1/2 value (`Sim::type_speed`, `crates/sim/src/rares.rs:172`), not `Sim::get_speed`. So a crew figure's `(speed × 11) / 8` misses the action scale (×9/8 / ×10/8) and the river halving as well |
| difference shows | a tracked crew figure's own `GUY` `x`/`y` in a per-frame `UNITDATA` window, on any frame its leader is under an `ATTACK` or `GUARD` action or standing on a `0x800` tile at `z <= 0`: the sim's figure steps `11/8 × moves` where the original steps `11/8 × (9/8 × moves)` or `11/8 × (moves/2)`. Divergence is immediate — one frame — and then compounds until the Manhattan snap. No capture is known to reach it: run56's scout and run67's merchant are on plain ground under a plain move, which is why `run56_s_figures_stand_where_the_original_s_do` and `run67_s_window_is_every_figure_s_whole_record` both pass |
| reached | reached (`GuyData::get_speed` is not on the blind list — every guy calls it every frame) |

The decompile confirms the shape the document states: `Guy::move@005d9240`'s
tracked branch is `iVar9 = GuyData::get_speed((GuyData *)local_10);` and then
`local_c = (…iVar9 * 0xb…) >> 3`, so the eleven-eighths is applied to a *layer
3* number. Note also that in `UnitData::get_speed@00608720` the tile the
`0x800` mask is read from comes from the **passed** `param_1`/`param_2` while
the `z` gate reads the unit's own `field_0xc` — so for a crew figure standing
on a different tile from its leader the original mixes the crew's tile with the
unit's height. `Sim::on_river` (`crates/sim/src/orders.rs:645`) takes both from
one `Pos` and cannot express that; it is moot only while the crate never asks
for a crew figure's speed at all.

### R4 — "the turn arm … is not modelled yet" is stale

| | |
|---|---|
| document | MOVEMENT.md `:543`–`546` — "The two arms above the turn are `Guy::move`'s own `set_anim` callers, and the second of them — the **turn arm** — is not modelled yet … What it costs to land is in the queue" |
| code | `crates/sim/src/anim.rs:1261`–`1279` (`fn guy_follow_anim`) |
| document says | a body that has arrived but has not come round to the order's angle is not put back on the walk here |
| code does | it is: the arm is implemented, guard and all — the sea/`SPECIAL_ANIM` branch first (R2), then `if anim != TURN_LEFT && anim != TURN_RIGHT && anim != ATTACKWALK { self.guy_set_anim(u, g, WALK, false, true); }` and `stopped` cleared |
| difference shows | no observable — the document lags the code |
| reached | reached (`Guy::move@005d9240`) |

### R5 — "`CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT` … which this crate does not model at all" is stale

| | |
|---|---|
| document | MOVEMENT.md `:547`–`550` — "`Guy::do_turn` overrides the walk with `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT` whenever the guy's piece has a turn animation (`guy_flags & 8`, `Guy::init_real@005db6b0:179`), which this crate does not model at all" |
| code | `crates/sim/src/anim.rs:803`–`819` (`fn guy_do_turn_anim`), reached from `crates/sim/src/lib.rs:3086`, `crates/sim/src/lib.rs:3208` and `crates/sim/src/orders.rs:2808` |
| document says | neither the override nor `guy_flags & 8` is in the crate |
| code does | both are: `if to == was || !self.guy_turns(u, g) { return; }` is the `guy_flags & 8` gate, and the sign of `heading − was` picks `TURN_RIGHT` or `TURN_LEFT` |
| difference shows | no observable — the document lags the code |
| reached | reached (`Guy::do_turn@005d97a0`) |

### R6 — the tile-permission refusal does not clear `unit_masks & 8`, and the document calls the bit unmodelled when the crate models it

| | |
|---|---|
| document | MOVEMENT.md `:457`–`458` — "SEAM: the original also clears `unit_masks & 8` on the refusal. The bit has no reader this crate models." |
| code | `crates/sim/src/orders.rs:2896`–`2899` (`fn unit_step`); the bit is `crates/sim/src/lib.rs:193`, `pub line_ok: bool`, documented there as "`unit_masks & 8`: a straight line to the waypoint has been verified" |
| document says | the crate has no reader for `unit_masks & 8`, so not clearing it costs nothing |
| code does | the crate has exactly one, and it is on the hot path: `crates/sim/src/orders.rs:1782`, `if !self.units[u].line_ok { … find_path … }`. So the original re-verifies its line on the frame after a refused step and the crate steps straight again |
| difference shows | the frame after any tile refusal. run65's caravan is the known one: `move_step` drops the step on sim-frame 6206, and on 6207 the original enters `Unit::find_path@005fb910` (and `go_around_building` behind it, which may push a waypoint) before stepping, where this crate goes straight to `move_step`. Falsifiable two ways in the existing capture — the trace's `Unit::find_path` entry on 6207 in `rontrace-run65*`, and the caravan's `x_internal`/`y_internal` on 6207 onward if the re-plan pushes any detour at all |
| reached | reached (`Unit::move_step@005faf30`) |

`005fb7c1`–`005fb7fd` in the decompile is unambiguous:
`((UnitData *)this)->unit_masks = ((UnitData *)this)->unit_masks & 0xfffffff7;
return 0;` — the clear and the zero return are the same two lines. The crate
takes the `return 0` (`Did::Nothing`) and leaves the bit. **UNSURE** how much
of run65's window this actually moves: if `find_path`'s straight-line test
succeeds on 6207 the two sides re-converge within the frame and only the trace
draw differs. Settled by re-running `run65_s_window_is_the_original_s_unit_for_unit`
with the clear added.

### R7 — the out-of-world refusal clears `unit_masks & 8` and returns "nothing", where the original leaves the bit and returns 1

| | |
|---|---|
| document | MOVEMENT.md `:413`–`416` — "**A step outside the world is refused, not clamped**, and the order continues; the unit tries again next frame. (`crates/sim` asks `World::accepts`, which is the same bounds test.)"; and `:441`–`444`, "`move_step` returns 0 where every other refusal returns 1" |
| code | `crates/sim/src/orders.rs:2961`–`2965` (`fn unit_step`) |
| document says | the order continues unchanged and the unit re-tries the same step next frame; this refusal is one of the ones that return 1 |
| code does | `self.units[u].line_ok = false; self.units[u].movement.dest = None; return Did::Nothing;` — it clears the verified-line bit (forcing a re-path next frame) and returns the value the original reserves for the two refusals that return 0 |
| difference shows | two ways, both at the map edge. (a) the re-plan: a `Unit::find_path` entry in the trace on the frame after, and any detour it pushes moves the unit's position. (b) the return value: `do_move`'s answer is discarded for a plain move (`crates/sim/src/orders.rs:1193`–`1206`, which drops the `Did`) but not for a **group** leader — `group_move_leader` (`crates/sim/src/orders.rs:2014`–`2016`) ungroups the whole formation when `do_move` answers `Did::Nothing`, so a marching group whose leader's step leaves the world dissolves here and does not in the original. No capture is known to reach either; the corpus has no edge-of-map march |
| reached | reached (`Unit::move_step@005faf30`) |

The four bounds tests in the decompile each `return 1` on their own
(`005faf30`, the block ending `if (*(int *)&GameAccess::world->field_0x1c *
0xc0 <= CVar13.value) { return 1; }`), and none of them touches `unit_masks`.
The crate's own comment on the branch — "A step the world refuses: re-plan next
frame" — is the divergence written down as if it were the rule.

## What was checked and found faithful

A later pass need not re-derive these. Each was read in the document and then
found, by name or inlined, in the code cited beside it.

**The unit of speed.** `MOVES` is already position units per frame and no
converter is applied (`crates/sim/src/lib.rs:2038`, `crates/sim/src/rares.rs:172`
— `type_speed` starts from `t.moves`); there is no `unit_move_speed` slot in
`Tuning` at all, which is the identity converter written down as an absence.
The floor of 3 is `movement::SPEED_FLOOR` (`crates/sim/src/movement.rs:416`)
and is applied last, after the cap, in both `Sim::get_speed`
(`crates/sim/src/orders.rs:624`) and `movement::group_capped`
(`crates/sim/src/movement.rs:424`) — which is the original's order at
`00608720`.

**Angles.** The four cardinals and `Angle::INITIAL = 0x55555555`
(`crates/sim/src/movement.rs:59`–`67`), y increasing southward (`find_angle`
negates `dy` first, `crates/sim/src/movement.rs:131`), and `Angle::INITIAL`
actually installed on a fresh unit (`crates/sim/src/lib.rs:398`–`401`) rather
than only defined. `degrees_to_angle` (`crates/sim/src/movement.rs:102`) is the
decomposition, three units per complete five degrees included, and
`degrees_to_angle(20)` is `SLOW_TURN_BELOW + 1` so a 20° type is not slow.

**`find_angle`.** Both axis early-outs, `t = lo * 0x4000 / hi`, the
straight-line correction `0x2800 − |0x1333 − t| × 0xb00 >> 14`, the `& ~0x3fff`
mask, the `× 4`, and all eight octant placements
(`crates/sim/src/movement.rs:128`–`165`). The 0.94% overshoot at 45° is pinned
as a test rather than corrected.

**The sine table.** 256 pinned integers with `[255] == 65535` and `[128] ==
46482` (`crates/sim/src/movement.rs:181`); the byte-wide `(idx + 1) & 0xff`
wrap and the `wrapping_mul` the axis angles depend on
(`crates/sim/src/movement.rs:229`–`235`); the mirror `0x7fffffff − a` in the
*caller* and the bit-30 branch deliberately not modelled
(`crates/sim/src/movement.rs:243`–`256`); negative angles handled by negating
the distance; `cos` as `sin` a quarter turn later
(`crates/sim/src/movement.rs:276`); and all three precision paths with the
shift split at 65535 and 2^24 (`crates/sim/src/movement.rs:262`–`268`).

**The unit step.** Every line of the pseudocode at MOVEMENT.md `:344`–`376` is
in `movement::move_step` (`crates/sim/src/movement.rs:524`–`620`) in the
original's order: heading before the turn; `slow` from `type_turn_speed <
SLOW_TURN_BELOW`; the near gate `manh < slow × 192` **or** the waypoint's
`TURN_FIRST` (read off the top of the path stack at
`crates/sim/src/orders.rs:2776`–`2779`, `false` for an empty stack); the far
gate at 45°, or 80° for a `wide_limit` type at `slow × 384`; the half step at
`45° / slow`; the Manhattan snap; the trig along the **post-turn** facing; the
clamps gated on `manh < 2 × step`; `y` subtracted. The two turn-in-place arms
are told apart as `TurnArm::Near`/`Far` and marked as two draw sites. Arrival
by the waypoint's `tolerance` is Manhattan, as at `005fb2f1`
(`crates/sim/src/orders.rs:2952`–`2959`). The tile-permission refusal —
`div_3_table` tile compare, `invalid_loc` with all five flags clear, `return 0`
— is at `crates/sim/src/orders.rs:2896`–`2899` (but see R6 for the bit it does
not clear). The `unit_masks & 0x100000` one-shot half step is a stated seam in
both.

**The body step.** `movement::body_follow`
(`crates/sim/src/movement.rs:699`–`721`) writes `last_speed = 0` **first** in
the at-des branch and the caller feeds that zero to the rate
(`crates/sim/src/lib.rs:3066`–`3072`, `if was_at_des { 0 } else { … }`), which
is the instant standing turn; guy 0 snaps with `last_speed =
vector_dist(dx, dy)` and never chases; the idle turn is gated on `guy_flags &
2` (`turned`) and on `facing != heading`; the average is `(3a + s) / 4`
truncating toward zero, skipped on the tracked branch's turn-gate return
(`crates/sim/src/movement.rs:796`–`797`). The tracked branch's whole
arithmetic — `find_angle`, the mode-1 turn, `2 × rate < owed` giving the frame
up, `(speed × 11) / 8`, `last_speed` overwritten by `vector_dist` on the snap,
the per-axis clamps, the world check leaving everything else written — is
`movement::follower_step` (`crates/sim/src/movement.rs:786`–`821`) and
`crates/sim/src/lib.rs:3211`–`3220`.

**The follower's destination.** The rotation's swapped axes — `track.0` into
`y` then `x` a quarter on, `track.1` a quarter and a half on — and the
`0 ..= bound − 1` clamp (`crates/sim/src/movement.rs:742`–`755`); the crew
loop's three writers and, crucially, the *third* case where nothing is written
(`!was_at_des || !facing_settled` at `crates/sim/src/lib.rs:3114`–`3117`);
`Unit::set_angle`'s own crew write at the top of the step, from guy 0's
position and the **heading** (`crates/sim/src/group.rs:1361`–`1368`); the snap
flag teleporting the crew, and `Unit::set_new_location` having **no** early
return for an unchanged position (`crates/sim/src/collide.rs:354`–`416`, item
210). `off_x`/`off_y` are treated as the constant zero they are. A tracked crew
guy's rate is the flat `CREW_TURN_SPEED` ahead of every other test
(`crates/sim/src/lib.rs:3191`), and a trackless one shares guy 0's body
rather than getting one of its own. The `+9` on `GuyData::get_speed` is a
stated seam in both (and see R3 for what else is missing there).

**Where movement sits in the frame.** Attrition before the order step before
the body follow, in `Sim::process_unit` (`crates/sim/src/lib.rs:2939`–`2958`),
with the comment naming the same observable the document does.

**Turning.** `(type_turn_speed >> 8) × 256`, the pack doubling by `×2`, the
instant `0x80000000` ahead of the mode test, mode 1 returning the base, mode 0
dividing by `avg_speed / 4 + 1` and flooring at `256 × 0xb60b`
(`crates/sim/src/movement.rs:340`–`364`); `Tuning::RON` carrying 256 and 2
(`crates/sim/src/tuning.rs:787`–`788`), and `UNIT_TURN_SPEED` checked as an 8.8
slot (`crates/sim/src/tuning.rs:1136`). `turn_towards`'s three-degree
tolerance, its snap when the rate covers the debt, the short way round, and the
bitwise-NOT magnitude that comes out one short
(`crates/sim/src/movement.rs:395`–`410`). `instant_from_stop` derived from the
type — not a packing type, `FOOT`/`MOUNTED`/`TRANSPORT`, and not `CART` — in
`sim::turning_of` (`crates/sim/src/lib.rs:468`–`479`), and `packed` read off
the unit rather than the type at both call sites
(`crates/sim/src/orders.rs:2760`–`2763`, `crates/sim/src/lib.rs:3062`–`3065`).

**The speed pipeline.** Layer 1 is the `MOVES` value plus the whales arm, which
is what the document says is landed (`crates/sim/src/rares.rs:170`–`177`);
layers 1 and 2 are otherwise stated inputs in both. Layer 3's action scale is
`×9/8` for `ATTACK` (index 10) and, for `GUARD` (index 12), `×10/8` under the
AI bit and `×9/8` otherwise — which is `00608720`'s `local_8 != 10 / != 0xc /
unit_masks & 0x40000` exactly, with `ai_driven` as this project's established
stand-in for `0x40000` (`crates/sim/src/orders.rs:603`–`608`). The land gate,
the river halving with `z <= 0`, and the floor are
`crates/sim/src/orders.rs:609`–`624` and `Sim::on_river`
(`crates/sim/src/orders.rs:645`–`648`), with `unit_masks & 0x10`, the general's
siege doubling and the group cap as stated seams in both.

**The animal's own `get_speed`.** The whole replacement — layer 2 only, the
air return before both the hurry and the floor, `is_move` on the current order,
the distance to the order's **goal** (`Order::move_dest` is `MoveOrder`'s
`+0x4/+0x8` `dest`, not the waypoint, `crates/sim/src/orders.rs:363`), the
strict `> 0x180`, `× 3 / 2` truncating, and `max(3)` — is
`crates/sim/src/orders.rs:584`–`593`, with `is_gaia` as the stated stand-in for
the class split.


## Adjudication — 2026-09-05, Opus

Every row re-checked against the source. **Seven confirmed, none struck.**
Three (R2, R4, R5) are the document lagging the code and are the main lane's
to correct. R1 and R3 are unreached: the corpus is openings, so no capture
has a modern-infantry unit past age 6, and run56's scout and run67's merchant
walk plain ground under a plain move. R7 is unreached — the corpus has no
edge-of-map march.

### R6 is confirmed, reached, and much larger than the row said

The trace settles the row outright. `tools/trace/report.py rontrace-run65.log
coverage 6206` has **no** `Unit::find_path` entry and `coverage 6207` has one
— exactly the row's prediction: the original clears `unit_masks & 8` on the
refused step and re-verifies its line on the frame after, where this crate
leaves the bit and steps straight.

And the code claim under it is worse than stated. `orders.rs:2894`'s SEAM
comment says "the bit has no reader this crate models"; the crate has
exactly one and it is on the hot path — `orders.rs:1782`, `if
!self.units[u].line_ok { … find_path … }`. The comment is false where it
stands.

**Then the widening.** The dump has printed `unit_masks` all along and
`run65_s_window_is_the_original_s_unit_for_unit` compared eleven fields of a
unit and not that one. Comparing bit 3 against `sim::Unit::line_ok` over the
window:

> `unit_masks & 8`: 450 rows compared, **335 disagree**

Three quarters, and the shape is not the refusal path at all: from the
window's first block the original carries the bit **set** on standing units
where this crate carries it clear. So `line_ok`'s whole lifecycle is wrong
here, not merely its clear — and each of those frames is a `find_path` this
crate may run and the original does not. Pinned in the test at 335 as a
count that may only fall (made to fail at 334 before landing). The fix is
`crate::orders`', not this lane's.
