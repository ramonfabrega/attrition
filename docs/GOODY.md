# The goody box

**What this establishes.** What a unit gets for walking onto ruins:
the trigger (`Unit::set_new_location@005f8d20+0x3cc`), the four guards in
front of it, the lottery that picks which good the box holds
(`Unit::explore_goody@005f9780`, one sync draw per candidate), the pile's
arithmetic, and what happens to the cell afterwards. It closes
`docs/VISION.md` §7's "`Unit::explore_goody@005f9780`'s one draw", which had
been parked as "belongs to a goodies mechanic, not to this one".

**Confidence.** High for the trigger, the guards, the candidate rule and the
draw count: three draws on frame 867 of run39 and three again on frames 898
and 1659 of run33, and East Indies' word runs through 867 only with the count
right (`rondata::diff::a_goody_box_draws_once_for_each_good_its_finder_can_gather`).
High for the arithmetic as *read* — the listing is quoted below — but **no
capture on disk checks the pile**, because leader detail is written once, at
start, before any box is opened. §6 names the capture that would.

**And the approach.** §7 is the other half, added 2026-08-31 with item 106:
`Unit::find_goody_box@005f2540`, the 49-cell sweep that sends a land unit
*to* a box it has seen, and `Unit::get_goody_box@005f7690`, the walk it
issues. Neither spends a draw, and both are diff-backed — East Indies'
scout re-aims on frame 825 and arrives on 879, which is what took the word
879 → 1256.

`World::reveal_fog@006b3d30` reaches `get_goody_box` by a third path, for a
unit carrying `unit_masks & 0x100`. Nothing here sets that bit, and it is
still a seam.

---

## 1. Where a box lives

One bit on the cell. `WData`'s first `short` is read **signed**, and bit
`0x8000` — `GOODY` in `docs/ORACLE.md`'s cell-flag table — is its sign bit,
which is why the original's trigger reads "the `short` is negative" rather
than a mask test. The `WORLD` record's `goodies` scalar is the map
generator's count and never changes in play: run39 prints `goodies 7` on
every one of its 3,708 frames, and its seven cells are

```
(52, 5)  (22, 22)  (53, 26)  (8, 28)  (36, 31)  (38, 34)  (45, 49)
```

which is the whole of East Indies' supply. Great Lakes has 22
(`docs/SCOUT.md` §7).

Behind the bit there is also an **object**: the cell's `down`/`down_who`
chain ends with `down == -3` and `down_who` holding an index into `items`
rather than a unit. `ObjectsData::find_goody_at@0065c040` walks that chain
and returns the item when the terminal is `-3` and the item's `flags & 1` is
set. The simulation does not model the item — `crates/sim/src/collide.rs`
chains units only, which is the queue's item 48 — so here the bit is the
whole of a box's existence. §5 says what that costs.

## 2. The trigger, and its four guards

`Unit::set_new_location@005f8d20` computes two "did it move" predicates: a
**cell** change (`div_3_table[p >> 8]`, the same coarse index the `WData`
array is keyed by) and a **tile** change (`p >> 6`). The tile one is what the
fog reveal hangs off (`docs/VISION.md` §6). The goody hangs off the *cell*
one, inside the branch that calls `Object::add_to_world`, and takes four
further tests in this order:

```c
if (local_10 != 0) {                        // the cell changed, and on map
  add_to_world(this);
  if (   (*(code**)(*(int*)this + 0x30))() == 0   // not an animal
      && (this->field_0x68 & 1) == 0              // unit_masks & 1
      && this->type->field_0x218 == 0             // type->domain == 0, land
      && wdata[cell].down_first_short < 0 )       // GOODY
    explore_goody(this);
}
```

- **Slot `+0x30` is `SubObjectData::is_animal`**, vftable offset 48. The map
  cannot name it — both overrides are trivial and COMDAT-folded, so the
  export prints `Buffer::is_pending_load` (`return 1`) in `Animal::vftable`
  and `Window::get_button` (`return 0`) in `Unit::vftable` — and the name
  comes from the PDB's `LF_ONEMETHOD` list. It is the same slot step 0 of
  `docs/COLLISION.md` §6 reads. **Gaia takes nothing**, which matters on a
  map whose herds wander: run39's animals cross open ground for 1,850 frames
  and never open a box.
- **`unit_masks & 1`** is `docs/VISION.md` §7's placement ghost, clear
  everywhere in this simulation and in every capture.
- **`type->domain == 0`** is the land test `docs/SCOUT.md` §12 uses. A ship
  passing a coastal ruin leaves it.

The caller that matters is `Unit::move_step+0x8f4` — an ordinary walking
step. `Unit::init`, `Unit::work` and `find_path`'s pull-back also reach
`set_new_location`, so a unit *born* on a box opens it; §4 is what it gets.

## 3. The lottery, and the pile

Everything below is behind `game->frame != 0`.

```
best = 99,999,999;  pick = -1
for good in 0..6:
    if type_avail(who, good, 1) == 0: continue        // availability first
    if good == 3: continue                            // KNOWLEDGE, always
    score = Random::get(game_random, 0, 0xffff) % 25 + bucket[good]
    if score < best: best = score; pick = good        // strict <
if pick < 0: pick = 2                                 // WEALTH
```

**One draw per candidate, and the draw count is the candidate count.** That
is what makes the frame a test of `LeaderData::type_avail@006e33a0` over the
six good types rather than of the lottery: every capture that reaches a box
spends **three** draws, because in the Ancient age food, timber and wealth
are available and knowledge, metal and oil are not. The site is
`Unit::explore_goody+0x27c`.

The score is `draw % 25 + bucket[good]`, lowest wins, ties to the earlier
good. So it is neither "the good you have least of" nor a uniform choice: a
good more than 24 behind the field wins outright, and the jitter only decides
between goods within 24 of each other. The 25 is a literal `idivl $0x19`, not
a constant.

**The pile, and the name that lies:**

```
5f9a31  movl 0xe41248(%esi), %eax     ; leaders[who].data_encrypted
5f9a37  movl 0xf4(%eax), %ebx         ; +0xf4 = epoch[3]
5f9a42  je   5f9a59                   ; has_tribe_bonus(9)?
5f9a44  xorl $0x63187, %ebx
5f9a4a  imull 0x68c(%eax), %ebx       ; constants->spanish_ruins      = 26
5f9a51  addl  0x688(%eax), %ebx       ; constants->spanish_ruins_base = 30
        jmp
5f9a59  xorl $0x63187, %ebx
5f9a5f  imull 0xc28(%eax), %ebx       ; constants->goody_box_age      = 25
5f9a66  addl  0xc24(%eax), %ebx       ; constants->goody_box          = 25
```

`LeaderDataEncrypt +0xf4` is **`epoch[3]`**, not `ages`. The struct is
`bucket`…`bonus`, then `+0xdc ages`, `+0xe0 epochs`, `+0xe4 discovered`,
`+0xe8 epoch[4]`; `ages` is masked `0x62766` and `epoch` `0x63187`, and this
site uses `0x63187`. `LeaderData::get_epoch_base@006d6ff0` maps cat 3 to
`BASE_EPOCHTYPES`, which the `TypeIndex` enum gives as `0x227` — the same
value as `BASE_SCIENCETYPES`. **So `GOODY_BOX_AGE` is per Science library
level, and the age has nothing to do with it.** `SPANISH_RUINS` and
`SPANISH_RUINS_BASE` replace *both* halves, not just the base.

All four constants are read by `Constants::get_item` — the plain integer
reader — so `25 resources` and `25 resources / age` arrive as 25, not scaled
to 8.8 the way `PEASANT_RATE`'s `10 resources` is (`docs/PRODUCTION.md`,
`sim::tuning::Slot`).

The pile goes to `bucket[pick]` and to `LeaderData +0x86c
goody_box_resources`, a score counter `Leader::reset_score@006e37f0` zeroes
beside `+0x868 goody_box_techs` and `+0x870 goody_box_units` — the two other
things the RoN designers evidently once meant a box to hold. Neither is
written anywhere in this executable.

## 4. What happens to the cell

Three writes, and the order is the surprise.

1. If `find_goody_at` found the item, the item's vtable `+0x90` is called
   and the box's own coordinates are kept for the message bubble.
2. The cell's object chain is repaired: where `down` is `-3` it becomes
   `-1`. **Except in the head case**: when the cell's *own* `down` is `-3`
   the original writes `-3` straight back and `down_who` unchanged —
   `movl $0xfffffffd, %edx; movw %dx, 0x8(%eax,%ecx,4)` at `5f98ff`. A dead
   store, verified in the listing rather than inferred from the decompiler.
3. `wdata[cell].flags &= 0x7fff` — the `GOODY` bit goes.

All three run **before** the `frame != 0` gate, so a unit placed on a box
during setup consumes it and is paid nothing. The bit is what stops a second
unit taking the same ruins, since step 2 leaves the item linked.

## 5. Coverage

| Claim | Backed by |
| --- | --- |
| The trigger is a **cell** change, on `move_step`'s step | diff — run39 frame 867, run33 898 and 1659, all `Unit::explore_goody+0x27c < Unit::set_new_location+0x3cc < Unit::move_step+0x8f4` |
| One draw per candidate good | diff — three draws on each of those three frames, and East Indies' word runs to 879 only with three |
| Knowledge is never a candidate; metal and oil are not available in the Ancient age | diff — the same three, indirectly: any other reading of the loop gives a different count |
| The seven `GOODY` cells of run39, and `(45, 49)` taken on 867 | diff — the `WORLD` record's cells and the dumped position of `1/0` on 868 |
| `score = draw % 25 + bucket`, lowest wins, strict `<` | reading (listing at `5f99f7`–`5f9a12`), **and diff** — run60's sim-frame 4988, where the winner is timber by thirty-one and this crate's tie went the other way (`docs/ECONOMY.md`, "run60") |
| The fallback to `WEALTH` | reading (`cmovnsl` at `5f9a7c`) |
| `epoch[3] × GOODY_BOX_AGE + GOODY_BOX` | diff — run42 frame 867 (`docs/ORACLE.md`): `bucket[2]` 50 → 100, `epoch_get(scan)` `0 1 0 1`, and `1 × 25 + 25` is the observed pay; an `ages` reading would pay 25 |
| The Spanish pair replaces both halves | reading (listing in §3); the branch has never run |
| Frame 0 consumes without paying | reading |
| The four guards | reading; only the animal guard is exercised by a capture, and only negatively |
| The cell's object chain and the goody *item* | read, **not modelled** — queue item 48 |

## 6. What is not established

- ~~**The pile is unchecked.**~~ **Checked 2026-08-31** (`docs/ORACLE.md`,
  run42 — run39's game with `LEADERS=2` per frame, `samegame.py --exclude
  LEADERDATA` giving 900 common frames and none differing): leader 1's
  `bucket[2]` steps 50 → 100 on sim-frame 867, the trace puts
  `find_goody_at` and `explore_goody` on 867 and on no neighbouring frame,
  and the pay of 50 is the Science reading's — `ages` would pay 25. One
  detail of the prediction was wrong: `epoch_get(scan)` reads `0 1 0 1`,
  not `0 0 0 1` — Civic is 1 as well as Science. Only `epoch[3]` enters
  the formula, so §3's arithmetic stands as written.
- **`unit_masks & 1`.** Inherited unread from `docs/VISION.md` §7 and
  treated as clear, as everywhere else.
- **The item, and the message.** `find_goody_at`, the vtable `+0x90` close,
  the sound and the `TextBubble` are all unmodelled. None of them draws.
- **`goody_box_techs` and `goody_box_units`.** Both are on `LeaderData` and
  both are reset by `Leader::reset_score`; neither has a writer in this
  executable, so a box in RoN:EE holds resources and nothing else. Not
  re-checked against the 2003 build.
- **The Spanish branch has never run.** `has_tribe_bonus(9)` is false in
  every capture on disk.
- ~~**Whether a box's *good* is right.**~~ **Answered 2026-09-02, and it
  was wrong** (`docs/ECONOMY.md`, "run60"). run60 is run58's game with
  `[End Frame]` cut to `MISC,LEADERS=2`, so both leaders' six buckets are
  printed on all 5,400 frames. The AI opens a **second** box on sim-frame
  **4988** — the trace cannot show it, because its coverage records fire on
  a function's *first* entry and `explore_goody` had already fired on 867 —
  and the fifty goes to **timber** there. It went to wealth here.

  The lottery is the reason and the arithmetic is §3's, unchanged: the
  AI's buckets going in were food 208, timber **99**, wealth 130 in the
  original and food 208, timber **100**, wealth **100** here. Thirty-one
  clear of the field, timber wins outright against a jitter of at most 24;
  tied at 100, the jitter picks. The thirty wealth this crate was missing —
  a dock's, `Build::activate` line 590, 1,409 frames earlier — is what made
  it a tie. **So §3's formula was right and the input was wrong**, which is
  the only shape of error a draw count can never see: the frame spends
  three draws whichever good wins.

## 7. The approach — `Unit::find_goody_box@005f2540`

*Added 2026-08-31 (item 106), read from the export and then diffed against
run39's own `UNITDATA` frame by frame. High confidence for the sweep, its
two gates and the order it issues; the seams are named in §7.2 and §7.4.*

A box is not only something a unit trips over. A land unit that can see one
**walks to it**, and that walk is why East Indies' scout is standing still
on frame 879 twelve frames after taking the box on 867: the explore order it
finished was not the one `think_scout` gave it on 796.

### 7.1 Where it runs from

Two call sites, and neither spends a draw:

```c
// Unit::think_scout@005f6010, the first thing it does
if (type->domain == 0 && find_goody_box(this)) return 1;

// Unit::do_explore_to@005f24a0, after do_move
if ((o + frame) % 15 == 0
    && orderlist.head == this_order      // still the same order
    && is_captain())                     // o_up < 0
    find_goody_box(this);
```

The first makes an idle scout prefer a box to a ring walk, and it returns
before any of `think_scout`'s ten draws. The second is the one the capture
exercises: **one frame in fifteen, phased by `o`**, a unit still walking the
`EXPLORE_TO` this call was dispatched for looks around again. `is_captain`
is `UnitData::is_captain@0046ceb0`, `o_up < 0` — a figure marching inside
someone else's formation does not go off on its own.

A third caller, `World::reveal_fog@006b3d30`, reaches `get_goody_box`
directly for a unit carrying `unit_masks & 0x100`; that bit is set nowhere
here and it is a seam.

### 7.2 The sweep, and its two gates

49 cells — `move_x`/`move_y` walked to `0xc4 / 4`, which is the 7 × 7
neighbourhood `crate::world::MOVE_49` already holds — around the unit's own
cell, in the table's order. The first cell that passes wins; there is no
scoring.

```c
if (type->domain != 0) return 0;                 // land only
region = wdata[here].region;                     // read raw, -1 included
for (k = 0; k < 49; k++) {
    (wx, wy) = (cx, cy) + (move_x[k], move_y[k]);
    if (out of bounds) continue;
    if (wdata[wx, wy].region != region) continue;
    if ((short)wdata[wx, wy].flags >= 0) continue;        // GOODY is the sign bit
    if (!was_seen(2wx+1, 2wy+1) && !was_seen(2wx, 2wy+1)
     && !was_seen(2wx+1, 2wy) && !was_seen(2wx, 2wy)) continue;
    if (!is_ocean(wx, wy)) {
        i = find_goody_at(wx, wy);
        if (i >= 0 && !items[i]->is_seen(who)) continue;
    }
    if ((wx, wy) == cell(orders_x, orders_y)) return 0;   // already going there
    get_goody_box(this, wx, wy);
    return 1;
}
return 0;
```

Three things there are worth naming.

- **The region is read raw.** `local_20` is the unit's own cell's `region`
  short whatever it is, so a unit standing in no region at all looks for
  boxes in no region at all.
- **The cell's fog gate is four samples, and one is enough.** The listing
  tests `(2x+1, 2y+1)`, `(2x, 2y+1)`, `(2x+1, 2y)`, `(2x, 2y)` in that
  order and falls through to the next cell only when all four are clear.
  This is `WorldData::was_seen@006b53f0`, the one **with** the
  ally-territory shortcut.
- **…and the *item's* gate is the one that bites.** The inlined `is_ocean`
  at the top of `ObjectsData::find_goody_at@0065b7c0` is the same test that
  short-circuits its chain walk, so what the code below it asks is: *if
  there is a goody item on this cell, has this leader seen it?* —
  `ItemData::is_seen@00677850`, vtable slot `+0x48`. That reads
  `ever_seen & ally_mask`, a per-item byte `check_ever_seen` accumulates
  from the **current** line-of-sight grid, and it has **no** territory
  shortcut.

  On East Indies that distinction is the whole mechanic. `(45, 49)` is
  inside player 1's own borders, so `was_seen` answers yes there from frame
  0 and the cell gate alone would fire the sweep on **796** — the frame
  `think_scout` runs on, which parts the word at 796. The item gate holds it
  until the scout's own line of sight reaches the box, which happens between
  811 and 825, and 825 is the next fifteenth frame.

  **SEAM.** This crate chains no items (`docs/QUEUE.md` item 48), so it
  cannot walk `find_goody_at` and cannot hold a per-item `ever_seen`. It
  models `is_seen` as `was_really_seen` — the bare fog, `World::seen2`
  without the shortcut — over the same four half-cells, which is the same
  accumulation under a different name and at cell rather than item
  resolution. Unmodelled with it: the Spanish `has_tribe_bonus(9)` arm and
  the fall-through to `WorldData::is_seen`, the *current* grid rather than
  the accumulated one.

### 7.3 The order — `Unit::get_goody_box@005f7690`

```c
Group g; g.clear(-1); g.add(this->o, this->who);
i = groups.push_group(who, &g, 1);
groups[i].action_move_to(wx * 0x300 + 0x180, wy * 0x300 + 0x180,
                         QUEUE_FIRST, 0, 0, EXPLORE_TO, 0, -1, -1, 0);
```

A one-member group, forced in, and an `EXPLORE_TO` to the box's **cell
centre** — `c × 0x300 + 0x180`, which is not the `4c + 2` tile centre
`think_scout` aims at (`docs/SCOUT.md` §8). `Unit::add_move_facing_order`
then applies its 48-unit snap, so the order's own `x`/`y` land 24 units past
the centre: run39 writes `orders_x 34968` for a box whose cell centre is
34944.

**`QUEUE_FIRST` here is the *group's*, and it is not the unit's.**
`Group::action_move_near@00704990`'s arm at `00704bfe` copies the leader's
**action-flagged** orders aside (`set_up_insert`, `flags & 4`), calls
`action_halt`, recurses as `QUEUE_NEW`, and re-issues the copies as group
actions at `QUEUE_LAST` (`finish_insert`). A plain transit move carries no
action bit, so it is not copied and not restored: the walk the unit was on
is **dropped**. run39's `FRAME 826` is the proof — one order in the list,
where a unit-level `QUEUE_FIRST` would leave two. `docs/GROUPS.md` §17.

### 7.4 What run39 shows, field for field

Player 1's scout `1/0`, from its own `UNITDATA` record:

| frame | `orders_x/y` | path stack | orders |
| --- | --- | --- | --- |
| 797 | `35064 / 37368` | `(35040, 37344)`, `(35064, 38904)`, `(35064, 39672)` | 1 |
| 826 | `34968 / 38040` | `(34944, 38016)` | 1 |
| 879 | `34944 / 38016` | empty | 0 |
| 880 | `31224 / 39672` | `(31200, 39648)`, `(32760, 40440)`, `(33528, 39672)`, `(34296, 38904)` | 1 |

796 is `think_scout`'s ten draws and the target it picks; 825 is the
re-aim, and it changes the order, the whole path stack and nothing else;
879 is the arrival, and the frame the scout is idle for is the sixteen
draws — two `Unit::set_anim` stands and six `think_scout` ring pairs with
two cell draws — the word had been short of. 880 is the next explore
target, over four legs, and it agrees too.

### 7.5 Coverage

| Claim | Backed by |
| --- | --- |
| The look is `(o + frame) % 15`, from `do_explore_to` | diff — run39 re-aims on **825** and on no other frame between 796 and 879 |
| The sweep walks `move_49` and takes the first pass | diff — the cell it takes, and reading for the order within a ring |
| The order is the box's **cell centre**, `QUEUE_FIRST` at the group | diff — `FRAME 826`'s `orders_x/y` and its one path leg |
| A group's `QUEUE_FIRST` halts and re-issues as `QUEUE_NEW` | diff — `FRAME 826` holds **one** order, not two |
| The item's `is_seen` gate, not the cell's `was_seen` | diff — the cell gate alone fires on 796 and parts the word there |
| `is_captain` (`o_up < 0`) | reading; no capture has a figure under a captain walking an explore |
| `think_scout`'s head call | reading — every capture reaches it and none accepts there; by the time a scout is idle beside a box the box is taken |
| The region compare, read raw | reading |
| `find_goody_at`'s chain walk and `ever_seen` itself | read, **not modelled** — §7.2's seam |
| `reveal_fog`'s third call site | read, **not modelled** — nothing sets `unit_masks & 0x100` |
