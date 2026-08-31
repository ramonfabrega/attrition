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

**What it is not.** Two neighbouring functions carry "goody" in their names
and are different mechanics:

- `Unit::find_goody_box@005f2540`, a scout's *search* for a box to walk to —
  `docs/SCOUT.md` §12, already read, never yet reached by a capture.
- `Unit::get_goody_box@005f7690`, reached from `World::reveal_fog@006b3d30`
  when a unit carrying `unit_masks & 0x100` reveals a box: it pushes a
  one-member group and gives it an `EXPLORE_TO` order to the box's cell
  centre. Unmodelled; nothing here sets `unit_masks & 0x100`.

This document is only what happens on arrival.

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
| `score = draw % 25 + bucket`, lowest wins, strict `<` | reading (listing at `5f99f7`–`5f9a12`) |
| The fallback to `WEALTH` | reading (`cmovnsl` at `5f9a7c`) |
| `epoch[3] × GOODY_BOX_AGE + GOODY_BOX`, and the Spanish pair | reading (listing in §3). **Not diff-backed** — see §6 |
| Frame 0 consumes without paying | reading |
| The four guards | reading; only the animal guard is exercised by a capture, and only negatively |
| The cell's object chain and the goody *item* | read, **not modelled** — queue item 48 |

## 6. What is not established

- **The pile is unchecked.** No dump on disk prints a leader's `bucket`
  after a box is opened: the `LEADERS` detail that writes `bucket`,
  `ages_get()` and `epoch_get(scan)` is emitted once, in the start block,
  where every value is still its opening one. *Capture, owed:* East Indies,
  the run39 lobby and seed under `samegame.py`, `LEADERS` enabled per frame
  in `gamelog.ini`, ~900 frames. It would pin `bucket[2]` stepping by 50 on
  frame 867 and `epoch_get(scan)` reading `0 0 0 1` — which is the *only*
  thing that separates the Science reading from an `ages` reading, since in
  an Ancient-age game `ages` is 0 and would pay 25.
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
- **Whether a box's *good* is right.** The lottery's winner depends on the
  finder's buckets, which no capture prints at the moment a box opens; the
  frame's draw count would be identical whichever good won.
