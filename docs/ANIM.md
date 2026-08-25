# The animation clock

*Established 2026-08-24 from the decompile (`tools/ghidra/`, the export
under `decomp/`), the executable's own `UnitAnimCat` table, the install's
`anim_graphics.xml`, and two `DUMP_ALL` oracles: run12 (frames 0–3) and
run13 (sim-frames 95–103), read with `tools/gamelog/anims.py`. Implemented
in `crates/sim/src/anim.rs`, wired through `lib.rs`'s tick and `orders.rs`'s
stands; the harness side is `crates/rondata/src/{gamelog,diff}.rs`. The
document says how each claim was established and what it has not.*

Every figure (`Guy`) of every unit plays an animation, `cur_time` frames into
one of `end_time`. `Objects::inc_time` steps every clock once a frame, and a
clock that runs out restarts its animation through `Guy::set_anim` — which,
for an idle animation, **draws once from `game_random`** to pick the next
idle variant. That draw is why the clock is simulation state and not
rendering: on run12's frame 0 forty of the 120 draws are the animals' first
idle rolls, and on run13's sim-frame 100 twelve of the 18 are fish whose idle
animations ran out together (`docs/SYNC.md` §4, §4.1). This document is the
mechanism behind those counts.

## 1. The record — `GuyData` (`rise.pdb`, size 0xbc)

| offset | field | what |
|---|---|---|
| `+0x74` | `cur_time` (ulong) | frames into the current animation |
| `+0x78` | `end_time` (ulong) | the animation's length; **0** right after `init_real` |
| `+0x7c` | `last_time` (int) | `cur_time` before this frame's step; **−1** right after any `set_anim` |
| `+0x88` | `gpiece` | the graphic piece — the model whose animation packet the lengths come from |
| `+0x8c` | `o` | the unit's object number |
| `+0x9a` | `guy_flags` | bit 4 an attack-2 double step, bit 0x20 the two-variant idle, 0x40 a plane, 0x80 a boat's crew |
| `+0x9c` | `cur_anim` (char) | the `UnitAnim` slot (§2) |
| `+0x9d` | `stopped` | the body stood on its destination at the last `Guy::move` |
| `+0x9e` | `hold_attack`, `+0xa0` `queued_attack` | combat's deferred animations (§6) |
| `+0xa2` | `guy_num` | the member's index in its unit |

A `DUMP_ALL` dump prints all of it per guy per pass (`GuyData::log_data@
005de6c0`); `rondata::gamelog::Guy` reads `cur_time`, `end_time`, `last_time`,
`cur_anim`, `gpiece`, `guy_flags`, `stopped`, `guy_num`. Below `DUMP_ALL` the
per-frame `GUY` blocks are empty.

## 2. The slots and their categories

`UnitAnim` (`rise.pdb` type 0x46B1), 38 slots:

```
0 DEFAULT  1–3 IDLE1–3  4–6 GROUP_IDLE1–3  7 SLOG  8 WALK  9 JOG  10 ATTACKWALK
11–14 ATTACK1–3, ATTACKSPECIAL  15–20 the six deaths  21–22 TURN_LEFT/RIGHT
23–24 PACK/UNPACK  25 CHOP_WOOD  26 WALK_WITH_WOOD  27 DUMP_WOOD  28 WALK_TO_WOOD
29 MINE_ORE  30 WALK_WITH_ORE  31 DUMP_ORE  32 WALK_TO_ORE  33 BUILD  34 REPAIR
35 SOW  36 REAP  37 FARM
```

`UnitAnimCat` — 38 dwords at `.rdata+0x2f370` (`S_GDATA32` at `0002:193392`
in the PDB; file offset `0x6f2b70`), read straight out of the executable:

```
slot 0–6 → 0     7,8,9 → 8     10 → 10     11–14 → 12     15–20 → 15
21 → 21  22 → 22  23 → 23  24 → 24  25 → 25  26 → 8  27 → 27  28 → 8
29 → 29  30 → 8   31 → 31  32 → 8   33 → 33  34 → 34  35 → 35  36 → 36  37 → 37
```

So the seven idles are one category (`CHAR_DEFAULT`), the seven walks — the
three plain ones and the four carrying ones — are one (`CHAR_WALK`), the
three attacks are one (`CHAR_ATTACK2`), the deaths one, and every work
animation is its own. **The category, not the slot, decides every branch in
`set_anim` and `inc_time`.** `anim::CAT` is the table; `anim::category`.

This corrects a row of `docs/SYNC.md` §3.2: the `param_2 == 0xc` branch of
`set_anim` is not the sow, it is the **attack** category (0xc = 12) — the one
draw that picks `ATTACK1`/`ATTACK2`/`ATTACK3` at 30/40/30 percent when the
request carries `param_3`. The sow (35) is its own category and its
`set_anim` never draws (`do_gather@005ef2a0:407` passes `(CHAR_SOW, 0, 1)`,
not 0 as that table said; it makes no difference).

## 3. The lengths and the pieces are art

`set_anim` ends by writing `end_time = animmgr.frames[packet.action_ids[cur_anim]]`
(`set_anim:725–736`) — the animation file's frame count, per graphic piece
and slot — and **3** for a slot the packet lacks. The piece is
`GraphicPieces::get_unit_gpiece@0090c030(type, who, o, guy_num, packing)`:

- a gaia type (`TypeIndex` `0x192..0x19e`, the birds and the herd animals):
  `first_bird_piece − 0x4b6 + 3·type + (seed + o) % 3` with `seed` the map
  seed — three pieces per type, cycling by object number. Run12 (seed 7236):
  sheep `o` 0, 1, 2, 3 → 60063, 60064, 60065, 60063; fish `o` 4, 5, 6 →
  60073, 60074, 60072;
- a player's unit: by the type, the tribe's art set (`tribes[+0x68]`), the
  leader's age bracket (`(age ^ 0x62766 + …) / 3`, capped at 2), the crew
  index `guy_num`, and — unless the type's `+0x2b8 & 4` — the **gender bit
  `o & 1`** (`get_unit_gpiece:155`), with fall-backs down the age and the
  set. Run12's citizens: player 0's odd `o` → 6688, even → 352; player 1's
  odd → 6336, even → 0. The scouts: 371 + 13043 (the dog) for player 0,
  19 + 12691 for player 1.

The sim reads none of the art. It takes both as an input, [`anim::Art`]:
`lengths: (gpiece, slot) → frames` and `pieces: (owner, type, sub, guy_num)
→ gpiece` with `sub` the gender bit or the gaia variant. `rondata` fills the
lengths from every `GUY` block of every pass of a `DUMP_ALL` dump
(`Log::anim_lengths`; run13 shows 33 pairs, run12 a few more) and the pieces
from the start dump's guys; siblings pool their lengths. A piece and slot the
table lacks gets `end_time = UNKNOWN` (never wraps, never draws — §9) rather
than the original's 3, because an unobserved length and a missing slot
cannot be told apart from a dump.

What the dumps show (`anims.py --lengths`): the citizen's `DEFAULT` is 33
frames and `IDLE1` 232 on every citizen piece; the walks 15 on every citizen
and scout piece, 16 on the sheep; the chop 33, the sow 47; the scout's
`DEFAULT` 61 (piece 371) and `IDLE2` 41; the sheep's idles 90/109/250 by
piece; the fish's 101/116/170 by piece, the same for every variant.

## 4. `Guy::set_anim@005da300` — the draw

Every request goes through the early returns, then the apply. The paths a
unit on open ground reaches, in the order the function tests them:

1. **The early returns** (`:155–224`). An idle request (`DEFAULT`, second
   argument 0) on a guy whose current category is idle returns while
   `cur_time < end_time` — the request is a no-op until the animation runs
   out. On a **walking** guy it compares the guy's destination with its own
   position (`des_x != x − off_x`): a body still on its way returns (and, if
   the walk cycle has run out, rewinds `cur_time` to 0 — no draw); a body
   that has **arrived** falls through to the roll. That is the arrival draw:
   run13's sheep 0 at sim-frame 101, `SLOG 11/16 → DEFAULT 0/90`, one draw.
   Any other request whose category equals the current one and whose clock
   has not run out returns too, unless the second argument forces it.
2. **The idle roll** (`:254–320`), when the requested category is 0. First
   the group-idle gate: a captain (`o_up < 0`, every standalone unit) whose
   piece has a `GROUP_IDLE2` animation skips the roll one frame in sixteen,
   `(frame + 0x2e + o) & 15 == 0`, and takes `DEFAULT`. Otherwise, **if the
   unit has no suspended search (`openlist == 0`), one draw**, `p = rand %
   100`. The variant is chosen only when the guy was already idle and the
   request carries its third argument (`local_20 == DEFAULT && param_3`):
   `p ≤ 69 → DEFAULT`, `≤ 82 → IDLE1`, `≤ 95 → IDLE2`, else `IDLE3` — with
   `guy_flags & 0x20` collapsing everything above 69 to `IDLE1`, and a
   peasant standing on a tile whose mask has `& 3` taking `IDLE1` for
   anything above 82. A guy arriving from a walk or a work animation draws
   but takes `DEFAULT` regardless. A variant the packet lacks falls back to
   `DEFAULT` (`:546–554`).
3. **The attack roll** (`:575–587`), category 12 with the third argument:
   one draw, `p < 30 → ATTACK1`, `p > 70 → ATTACK3`, else `ATTACK2`.
4. **The walk** (`:614–706`), category 8: the plain walk becomes `SLOG` /
   `WALK` / `JOG` by the body's average speed against the type's base
   (`avg_speed / (moves · UNIT_MOVE_SPEED)` below `0.6f` slogs, above
   `1.1f` jogs — a float in the original, cross-multiplied here; the
   lengths are equal on every piece observed so the boundary is
   unobservable); a bird's walk is a coin (`% 100 > 49 → JOG`); the carrying
   walks come from `unit_masks & 0x78000000`. A walk-to-walk change mid-walk
   rescales `cur_time · len_new / len_old` (integer), a walk already
   playing keeps its time.
5. **The apply** (`:559–573`, `:707–717`): a new slot starts at `cur_time =
   0`; the same slot keeps what ran past its end, `cur_time −= min(cur_time,
   end_time)`. Then `last_time = −1` (set at `:253`) and `end_time` from the
   table.

`Unit::set_anim@00616f40` runs `Guy::set_anim` for guys `0..guy_mark` and
`squad_size..num_guys` — every member, so a scout's dog rolls its own
variant on the unit's idle request (§5 says why that never shows).

**The callers on open ground**, and which of them draw:

| caller | request | draws when |
|---|---|---|
| `Unit::do_idle@0060dcd0` (no order) | `(DEFAULT, 0, 1)` | the guy's idle ran out (never — phase 7 caught it first), or the body just arrived from a walk |
| `Animal::do_idle@005d7460`, every idle frame | `(DEFAULT, 0, 1)` | as above — the sheep's arrival |
| `Unit::do_non_flat_gather@005f0170:512`, the tile choice; `:135`, the return to camp; the camp stand | `(DEFAULT, 0, 1)` | the woodcutters' four draws at frame 0 (§8) |
| `do_non_flat_gather:130`, `:267`, `:271`, `:412`, `:416` | `CHOP_WOOD`, `MINE_ORE`, `DUMP_WOOD`, `DUMP_ORE` | never — their own categories |
| `do_gather:407`, `:473`, `:479`, `:486` | `SOW`, `REAP` | never |
| `Guy::move@005d9240:86`, `Unit::move_step@005faf30:304` | the walk | never (a bird's coin aside) |
| `Guy::move:59`, the frame after a walking guy stops with the plain `WALK` slot and nothing else changed it | `(DEFAULT, 0, 1)` | the arrival, when no order made the request first |
| `Guy::inc_time` (§5) | the wrap | an idle running out |

## 5. `Guy::inc_time@005d9e10` — the step and the wrap

Phase 7 of the frame (`docs/SYNC.md` §2): `Objects::inc_time` walks leaders
0–9 **in order, no rotation**, each leader's live units in object order (then
its buildings), calling `Unit::inc_time@00610b40` → `Guy::inc_time` for guys
`0..guy_mark` and `squad_size..num_guys`. `Unit::inc_time` runs only while
`inside_up < 0` (a garrisoned unit's clocks stop) or for the two boat
categories. Per guy:

```
step = 1  (2 under guy_flags & 4 with an ATTACK2 playing; 0 while unit_masks2 & 0x10)
if guy_num < squad_size or cat(cur_anim) == WALK:
    last_time = cur_time; cur_time += step
    while cur_time >= end_time:
        if loopings[action(cur_anim)]:  set_anim(cur_anim, 0, 1)        # restart
        elif cat != ATTACK2:            set_anim(cur_anim == 10 ? WALK : DEFAULT, 0, 1)
        else:                           set_anim(DEFAULT, 0, 0) + the queued attack
else:
    cur_anim = guys[0].cur_anim; cur_time = guys[0].cur_time            # the mirror
```

Three things follow.

- **A category-0 animation running out always draws**, looping or not: both
  restarts are idle requests whose clock has run out, and both take §4's
  roll. `anim_graphics.xml` lists `Peasant Default`, `Peasant Idle1–3`,
  `Scout0 Default/Idle1–3` and the dog's under `<LOOPING>`; it changes
  nothing. Run13's sim-frame 100: twelve fish on piece 60073 (length 101)
  reach 101 together in `o` order (4, 7, …, 37) and roll `2,0,0,1,0,0,0,0,0,
  0,0,0` from `0x60032f25`, which are the dump's new `cur_anim`s, each with
  `cur_time 0` and `last_time −1`. Pinned: `anim::tests::run13_s_fish_wrap_
  together`.
- **A walk or a work animation running out is silent.** `Man Walk`, `Farmer
  Sow`, `Farmer Reap`, `Lumberjack Chop3/Carry`, `Miner Dig3/Walk` are
  looping and restart through the walk branch or the same-category apply,
  neither of which draws; a non-looping one (`Lumberjack Dump`, `Miner
  Dump`, the attacks, the deaths, pack/unpack) falls to `DEFAULT` and draws
  — `anim::non_looping` is the rule by slot, from the XML's names (§9).
- **The mirror.** A member past the squad's size that is not walking copies
  guy 0's slot and time and never steps or draws itself: the scout's dog.
  Run13's sim-frame 101: the human scout's two guys go `60/61 → 0/61` on
  **one** draw, the dog's `last_time` stays −1 throughout. Pinned:
  `anim::tests::the_mirror`. (The dog's `end_time` is its own, from its own
  `set_anim`s; it only matters if it is shorter than guy 0's running
  animation, which §9 leaves open.)

The four woodcutters' frame 0 on run12 — their draws at 36, 37, 46, 47 are
unit-phase stands (§4's camp stand; a same-slot `set_anim` leaves `cur_time
0`, which a phase-7 wrap could not), yet their clocks show `0/232, last −1`
at the frame's end while the farmers beside them show `1/47, last 0`: phase
7 skipped them that one frame. Neither gate fits: their `inside_up` is −1
and `unit_masks2` 0 at every pass, their object flags 1. Open (§9); it
moves those four wraps by one frame.

## 6. What the sim does with it

`crates/sim/src/anim.rs`, in the order the frame reaches it:

- **`Sim::guy_set_anim`** — §4 whole, minus the boat-crew offsets and the
  squad's group-idle synchronisation (a one-guy unit has no partner). The
  draw is `rng.roll() % 100`; the `openlist` gate is always open (the sim
  keeps no suspended search, `docs/PATHFINDER.md`).
- **`Sim::set_default_anim`** at the callers in §4's table: `do_idle`
  (`orders.rs`), the wood machine's camp stand, tile choice and return, and
  the frame-after-arrival in `guys_follow`. The work animations at the
  chop, the dump, the sow and the reap. The walk in **`guys_follow`**, run
  from `process_movement` with the body as it stood before the follow: a
  body away from the unit starts the walk (`Guy::move:86`; `move_step:304`
  is the same frame), one standing on it since last frame goes idle if it is
  still on the plain `WALK` slot (`Guy::move:59`) — the carrying walk for a
  gather's transit legs from `gather_walk`.
- **`Sim::guys_inc_time`** in the tick between `process_gaia` and the ammo,
  §5 whole: leader order, the mirror, the bounded wrap loop. A unit created
  this frame is skipped (`born == frame`): the original spends a trained
  unit's first frame inside its building (`inside_up ≥ 0` — run13's `1/6`
  ends sim-frame 99 at `0/232, last −1` after its two draws).
- **`Sim::init_guys`** at every spawn (`Trained`, `produce`):
  `Guy::init_real@005db6b0` — `cur_time 0, end_time 0, last_time −1`, one
  draw `% 100` → `DEFAULT` below 70, then `IDLE1`, `IDLE2`, `IDLE3` by tens
  (a variant the packet lacks → `DEFAULT`). The first idle request then
  finds `end_time 0` and rolls again: the trained citizen's two draws at
  sim-frame 99 (`docs/SYNC.md` §4.1). One guy per unit until the loader
  carries `num_guys`.
- **Gaia's units.** Owners 8 and 9 are units of the sim now (`marks` has
  ten slots; the tick's loop gives them the order step and the body follow
  and nothing a player owns), and **`Sim::animal_idle`** is `Animal::do_idle`:
  the idle request, then — for a land animal with a herd — when its idle is
  on its last frame (`cur_time == end_time − 1`), one draw `% 10 < 3` to
  wander: within `0x181` of the herd centre (§7) three more draws pick a
  compass step and two magnitudes (`(k + 1) · move · 0x30` on each axis)
  and issue a move; further out `find_nearby_spot(centre, 0xc0)` issues one
  without a draw. The `detect_unit_collision` before the near move is a
  seam. A herdless animal's `think_farm_animal` (every 128 frames, one
  draw `& 7` on a farm) is unread. The fish never reach any of it: their
  domain is not land.

`docs/SYNC.md`'s per-frame counts move with it (its §5). Run10 with run11,
run3 and run13 as siblings (the pinned set, `diff::tests::run13_s_window_
counts_…`): **sim-frames 98, 99, 100, 102 and 103 match exactly** — 99 gains
the trained citizen's two creation draws (8/8), 100 the twelve fish wraps
(18/18) — 101 is 20/21 with the scout's wrap in and the sheep's arrival
out (the sim has no wandering sheep to arrive), 97 is 6/7 and 96 26/28 (the
AI scout's re-target draws, `think_scout`), 95 6/23 (its scan). With run12
as a sibling too, **frame 0 goes 48 → 96 of 120** (the forty animals, the
two scouts' four and the four woodcutters; the 24 left are the human scout's
scan, the AI scout's explore path and the 4-draw tail — none of them
clocks), frame 1 holds at 54/54, frame 2 at 6/6 — and every window frame
reads one higher than above, because run12's frame-1 clocks put the AI
scout's dog under a shorter idle than guy 0's, which the sim's standing
scout then re-rolls every frame (§9, the dog). The forty first idles of
run12's frame 0 and their forty variants are pinned in `anim::tests::run12_
s_forty_first_idles`, against the LCG from draw 48 and the dump's
`cur_anim`s; the human scout's own first roll is the original's draw 20,
`p = 91 → IDLE2`.

## 7. The animals' herd centre

`Animal::do_idle:37–39`: with the herd record `(cx, cy, wx, wy)`, the
centre it measures from is `((wx + 2·cx) · 0x300 + 0x480) / 3` on each axis
— a third of the way from the home cell's centre to the wander centre's, in
position units (`anim::herd_centre`). The herd's own walk every 64 frames is
`gaia.rs`. Which herd an animal belongs to is `UnitData+0x86` for an
`Animal`, which the dump does not print; the harness assigns the herd of the
animal's own type — one sheep herd on this lobby — and the fish, which are
`HERDFISH` in twelve schools, never wander.

## 8. The harness

`rondata --diff`:

- reads every guy's clock out of a `DUMP_ALL` dump's `GUY` blocks, at the
  start (`Initial::units[].guys`) and at the end of every traced frame
  (`Initial::frame_guys`, engine frame `n − 1` under `FRAME n`, beside the
  word `frame_seeds` reads from the same block), and the lengths from all
  of it (`Initial::anim_lengths`). `ANIMALDATA` → `UNITDATA` is read
  through, so gaia's animals are in `units` now;
- **borrows** what a dump lacks from its siblings (`borrow_from_siblings`):
  the start units' clocks by `(who, o)` from any sibling on the same setup
  stream, the animals wholesale when a `UNITS`-level dump has none, the
  traced frames' clocks with their words, and every sibling's lengths;
- stands the sim up with the art (`Art::lengths`, `Art::pieces` from the
  start guys, `Art::gaia_types` from the loader, the map seed) and each
  unit's guys (`set_guy`), gaia's units included with their herd;
- and at every traced frame's end, after installing the word, **installs the
  clocks** — except on a unit that is walking on one side only (the AI
  scout: the sim has no `think_scout`), which keeps its own; the note names
  the count. A walking clock on a standing unit would read every idle
  request as an arrival, one draw a frame.

`tools/gamelog/anims.py` is the instrument: the per-pass table for named
units, `--lengths` for the table, `--wraps` for every clock reset between
two passes.

## 9. What is not established

- **The woodcutters' un-stepped frame 0** (§5): which gate skipped their
  `Guy::inc_time` on run12's frame 0 and not the farmers'. `inside_up`,
  `unit_masks2 & 0x10` and the object flags are ruled out by the dump. The
  sim steps them; their first wrap (232 frames) lands one frame early.
- **Whether a scout's dog draws on the unit's idle request.** `Unit::set_anim`
  loops every member and `Guy::set_anim:268` enters the roll for any
  `guy_num != 0`, so by the code it does; run12's frame 0 cannot tell (the
  scan's 15 and the explore's 8 were placed by elimination, and a dog draw
  at 21 or 38 just shortens one of them). The sim draws for it. If it does
  not, frame 0's 96 is 94, and the dog whose `end_time` is shorter than guy
  0's running animation — which would draw every idle frame in the sim's
  reading — never does. The draw-site trace (`docs/SYNC.md` §6) settles it
  in one run.
- **Lengths the dumps have not shown**: `DUMP_WOOD`, `REAP`, `FARM`, the
  scout's `IDLE1/3`, most citizen variants on most pieces. A missing entry
  never wraps here; the original's is 3 for a slot the packet lacks. A
  `DUMP_ALL` window over frames 108–125 of this lobby shows the chop (110,
  122), the sow (112, 141) and the walks wrapping, and a longer one the
  dumps; a BHA reader would settle all of them from the art.
- **The loop flags by slot** (`anim::non_looping`): from the XML's names,
  not from the packets' slot-to-file mapping. Only the dumps and the
  attacks depend on it.
- **The walk's speed ratio** is `f32` in the original; cross-multiplied
  here. Unobservable while every piece's three walks share a length.
- **The group-idle gate**: `Art::group_idle` is empty until a dump shows a
  captain skipping a roll; the animals' forty first idles at frame 0
  include `o` 2, 18 and 34, whose gate frame it was, so their pieces have
  no `GROUP_IDLE2` — consistent with the roll for every one of them.
- **The attack animations, the deaths, pack/unpack, the boat crews'
  offsets, the turn animations, the squad's synchronised group idle** —
  read as far as the table in §4 and not modelled: the harness's runs have
  no fights.
- **`num_guys` per type**: one guy per spawned unit; the start dump's units
  carry as many as it prints.
- **`think_farm_animal`**, and the birds after creation (`think_bird`,
  `do_air_physics`) — unread past their draw sites (`docs/SYNC.md` §3.2).
- **The 4-draw tail of frame 0** is still not a wrap (`docs/SYNC.md` §6).
