# The merchant's walk

Where an idle merchant goes, and what it does when it gets there.

A rare resource pays nothing until a merchant stands on it
(`docs/ECONOMY.md`, step 6 and "What an owned rare does"), and the AI does
not train one until it has seen a rare (`docs/ECONOMY.md`, "The rares a
leader has seen"). This document is the piece between those two: the order
that sends the merchant to the good, and the search that deploys it there.

Three functions and a call site:

1. **`Unit::think_merchant@005f4740`** (§2) — the whole of the decision. A
   packed merchant scores its leader's `new_rares` list, moves the winner
   to the **back** of that list, and issues a `MOVE_TO` at the good.
2. **`Unit::unpack_merchant@006038e0`** (§3), which sits at
   `think_merchant`'s head and is also its ending: is there a spot near
   here worth deploying on, and if so, walk to it and cast.
3. **`Unit::think@005f6e40`** (§4) — the two arms that reach the above, and
   the third that never does.

Its scoreboard entry is East Indies' word, which stood at **6353** while
no merchant was ever trained, **6356** while one was trained and stood
still, and **6570** with this document implemented.

---

## 1. What a merchant is, to the code that asks

`UnitData::is_merchant@0046d370` is **three exact `TypeIndex` values** and
never a lineage: `MERCHANT` `0x3d`, `MERCHANTDUTCH` `0x3e`, `FURTRAPPER`
`0x190` (400). `UnitData::is_rare_collector@0046fae0` is those three *or*
the `FISHERMEN` (`0x13d`) lineage — the set that earns from a deposit by
standing on it.

All three merchants **pack**: `unit_flags2 & 4`, which
`Unit::init@00612100:376` turns into `unit_masks & 0x80000` at birth. So a
merchant comes out of its Market packed, and stays packed until it deploys.
That one bit is what every arm below reads.

---

## 2. `Unit::think_merchant@005f4740`

Answers `1` when it did something, which ends the caller's `Unit::think`.

### 2.1 The head: a deployed merchant is finished

```
if (unit_masks & 0x80000) == 0:      return 1     # not packed — deployed
if unpack_merchant(this, 3):         return 1     # deployed it, here
```

**The first line is a `return 1`, not a `return 0`.** A merchant that has
already unpacked onto a rare never searches again, and never falls through
to `Unit::think`'s tail either — it does not join an army, it does not
scout, it stands on its good and earns. The second is §3.

### 2.2 The score, over `new_rares`

`local_10` is taken once before the loop:
`WorldData::get_tregion@006b52e0` of the **merchant's own tile**
(`crate::world::World::tregion_alt`, `docs/PATHFINDER.md` §15 — not the
plain `World::tregion`). So is the ocean bit of that tile,
`TData.mask & 0x30 == 0x20`.

Then, for entry `i` of `LeaderData::new_rares` — the goods-list indices
`Leader::new_rare@006d9e70` records, in the order the leader saw them:

```
base = 200 - 10 * i                       # decremented at the loop tail,
                                          # accepted or not
g = goods[new_rares[i]]
if not (g.flags & 1):                     continue     # alive
if not (g.ever_seen & (1 << who)):        continue
c = (g.x / 0x300, g.y / 0x300)                         # the good's world cell
if c.who >= 0 and not is_ally(who, c.who):continue     # -1 and -2 both pass
if ocean_bit(g.tile) != ocean_bit(my tile):            continue
at = (c.x * 0x300 + 0x180, c.y * 0x300 + 0x180)        # the cell's centre
if find_unit(at, SEARCH_FRIENDLY, who, 0x300, FILTER_NOT_ME(o, who),
             FILTER_TYPE(my type)) >= 0:  continue
if find_unit_ordered(at, …, 0x300, same filters) >= 0: continue
if find_unit(at, SEARCH_ENEMY, who, 0xc00, FILTER_COMBAT) >= 0: continue
score = base + (c.region == local_10 ? 100 : 0) - danger[who][c >> 1]
score = max(score, 1)
if score > best: best = new_rares[i]
```

Six things worth saying about that:

- **The base falls whether or not the slot is taken.** `local_30 -= 10` is
  at the loop tail, past every `continue`, so a refused good still costs
  the goods behind it ten points. Position in the list is what the number
  measures, not rank among the acceptable.
- **The two region reads are different functions.** The merchant's is
  `get_tregion` on a *tile*; the good's is the plain `WData +4` of its
  *cell*. Item 142's distinction, and this is a call site where the two
  sit in the same comparison.
- **Everything after the ally test is asked at the centre of the good's
  cell**, `c * 0x300 + 0x180`, not at the good's own point. A good near a
  cell edge is searched around from up to a third of a cell away.
- **The type filter is exact.** `FILTER_TYPE` compares `TypeData.type`, the
  `TypeIndex` the three ids above name, so a Fur Trapper does not keep a
  Merchant off a good and a Dutch Merchant does not keep a Fur Trapper.
- **"Ordered" is literal, twice over** — §2.2.1.
  `ObjectsData::find_unit_ordered@0065bc40` requires the candidate to hold
  a **move-family** order (`{1, 2, 3, 4, 0x12, 0x13, 0x15}`,
  `docs/ORDERS.md` §1.2), *and* it measures the distance to the
  candidate's `orders_x`/`orders_y` rather than to its body. So a sibling
  standing at the good is caught by the first search and a sibling walking
  at it — from anywhere on the map — by the second; a sibling gathering
  somewhere else is caught by neither.
- **The floor is 1, and it is only reachable through danger.** With
  `base ≥ 200 − 10·(list length)` and the bonus non-negative, the `max(·,
  1)` matters exactly where the danger map is large — which is the war
  case, and no capture on disk has one.

Ties keep the **earlier** good: the compare is `best < score`, strict.

### 2.2.1 The two searches measure two different points (2026-09-02)

`find_unit` and `find_unit_ordered` take the same twelve arguments and read
as the same sweep. They are not. Both fold over the eight leaders' unit
arrays, apply the same two `Search::valid_filter` gates, and finish with
`vector_dist(|dx|, |dy|) <= param_5`; what differs is where the delta comes
from:

| function | the point it measures | listing |
| --- | --- | --- |
| `find_unit@0065ca80` | the **object's** own position, `SubObject +0x10/+0x14`, un-XOR-ed with `0x63637` | `0065cd0f`, `0065cf24` |
| `find_unit_ordered@0065bc40` | the **unit's** `orders_x`/`orders_y`, `UnitData +0x70/+0x74` | `0065be35` |

The decompiler prints both as `vector_dist(unaff_EDI, unaff_ESI)` and names
neither, so this is a listing reading — the audit README's `unaff_` rule
again, and the two functions sitting a page apart is what makes it safe: the
same shape with one operand swapped.

So the second search does not ask "is one of mine standing near this good"
— the first already did — it asks **"is one of mine on its way here"**, and
a merchant halfway across the map with a `MOVE_TO` at the good answers yes.
That is what makes §2.3's rotation a *design* rather than a tie-breaker:
the list rotation is the cheap half, and this is the half with teeth.

The one gate `find_unit_ordered` does apply to the body is the region test
behind `param_6 & 0x200`, which reads the object's cell like `find_unit`
does — and `think_merchant` passes `param_6 = 0`, so it never runs here.

### 2.3 The rotation, and the order

```
new_rares.remove(best)          # SimpleArray<int>::remove — by value,
new_rares.add(best)             # shifting the tail down; then appended
add_move_order(good.x, good.y, MOVE_TO, QUEUE_NEW, action = 0)
```

`SimpleArray<int>::remove` finds the **value**, not an index, and shifts
the tail down, so the list keeps its order with the winner moved to the
end. It is a rotation, never a shortening: `num_rare_resources_seen` — the
script function that buys merchants — reads the same length afterwards, and
the *next* merchant to think scores the good this one took last. That is
the whole of the "two merchants do not go to the same rare" design, and it
is cheaper than the object searches beside it.

The tail is `Unit::add_move_order@00616ed0` inlined: `unit_masks &=
~0x4000000`, `path.length = 0`, `close_orders`, `clear_partial_path`,
`update_action`, then the order — which is `QUEUE_NEW` exactly
(`docs/ORDERS.md` §1.5). The destination is the good's point snapped to the
48-unit grid and the bearing is `find_angle` to the **unsnapped** point,
which is `add_move_order`'s own asymmetry (`docs/ORDERS.md` §4.3).

---

## 3. `Unit::unpack_merchant@006038e0` and the spot search

```
unpack_merchant(this, ring):
    t = find_merchant_spot(this, x, y, ring)
    if t is none: return 0
    unit_masks &= ~0x100
    add_cast_order(-1, -1, …, UNPACK 0x28c, QUEUE_NEW, 0)
    a = find_angle(t.x * 0xc0 - x, t.y * 0xc0 - y)          # the raw point
    MOVE_TO at snap(t.x * 0xc0), snap(t.y * 0xc0), angle a  # built inline
    orderlist.add; clear_partial_path; head = head->next
    update_action
    return 1
```

`Unit::add_cast_order@005e4a60`'s head rewrites the generic `0x28c` by the
caster: a machine gun's `0x28e`, the **three merchant ids'** `0x290`, the
`FISHERMEN` lineage's `0x292`.

### 3.1 The walk goes in front of the cast (2026-09-03)

`LinkListBase::add` is the back of the queue, but the line after it is
**`head = head->next`** — the identical statement `add_cast_order`'s
`QUEUE_FIRST` arm runs, and `docs/ORDERS.md` §1.5 already names its effect:
the new order becomes the current one. So the two orders are
**`[MOVE_TO, CAST]`**: the merchant walks to the spot and casts on arrival,
which is the only reading that makes sense of a spot search that can answer
a tile the merchant is not standing on.

This document said "appended behind it" for a day, and run68 is what
settled it. Its block 6714 is `1/19`'s arrival:

| field | dump |
|---|---|
| `x_internal`, `y_internal` | 32076, 37188 — where the walk to the `CITRUS` ended |
| `orders_x`, `orders_y` | **32280, 36888** |
| `dest_angle` | 346619904 |
| order list, as printed | `CASTORDER` (`type 14`, `spell 656` = `0x290`), then `MOVEORDER` (`type 1`, `x 32280 y 36888 angle 346619904`) |

The dump's print order is the **reverse** of the execution order, and both
walks are in the export: `OrderList::log_data@00730070` sets
`node = head->prev` and then advances by `next`, so it prints `head` first,
while `Unit::update_action@0060a870` sets the same `node = head->prev` and
advances by `prev`. `UnitDump::orders_front_first` in `rondata::gamelog`
has always reversed it. So `[CAST, MOVE]` printed is `[MOVE, CAST]` run,
and the three fields agree with that and with nothing else:
`update_action` stops on the first order that is neither a plain move nor a
`CHANGE_FORM` (§3.3 of `docs/ORDERS.md`), so a cast in front would have
left `orders_x/y` at the unit's own position — which is exactly what this
crate had. The unit then turns for three frames and steps at (32280, 36888)
on 6718.

The destination and the angle are two different points. The order is built
inline rather than through `Unit::add_move_order`, but the arithmetic is
that adder's: `x = (t.x · 0xc0 / 0x30) · 0x30 + 0x18`, and the angle is
`find_angle` of the delta to the **unsnapped** `t · 0xc0`. Tile
`(168, 192)` gives `(32280, 36888)` and 346619904 — the dump's two numbers,
and the snapped point is 408616960 away from the angle it would have given.
`add_move_order` already splits them the same way (`docs/ORDERS.md` §4.3),
so `add_move_order(u, t · 0xc0, MoveTo, First, false)` is the whole of it.

`Unit::find_merchant_spot@00603ab0` is a gate and a ring:

```
if not calc_gather(this, …, 1, 1, x, y):  return 0     # nothing near ME
for i in 0 .. radius[ring]:                            # radius[] @00add1e0
    t = (tile.x + move_x[i], tile.y + move_y[i])
    if t off the map:                     continue
    if not good_merchant_spot(this, t):   continue
    if invalid_loc(this, t, 0,0,0,0,0):   continue
    if detect_unit_collision(this, t * 0xc0, 1, 1, 0, 0, 0): continue
    return t
return 0
```

**The gate is `UnitData::calc_gather@00609180` where the unit already
stands**, and it is what makes the head cheap: a merchant walking across
the map asks one gather search per think and never walks a candidate.
`radius[]` at `00add1e0` is `1, 9, 25, 49, 81, …`, the cumulative count of
`move_x`/`move_y` per ring, so ring 3 is the 49-entry `MOVE_49` and ring 4
the 81 around it.

`UnitData::good_merchant_spot@006068a0` is the **two-by-two whose
bottom-right corner is `t`** — `(x, y)`, `(x−1, y)`, `(x, y−1)`,
`(x−1, y−1)` — each on the map (`WorldData::is_valid@0046f760`), not
`BLOCKED` (`0x4000`), not a building footprint (`mask & 3 == 3`) and not
`PLACED` (`0x80`); and then `calc_gather` again, at `t`'s own corner
`(t.x · 0xc0, t.y · 0xc0)`. That is the deployed merchant's footprint: it
becomes a two-tile-square thing, which is why the search asks about four
tiles for a one-tile walker.

---

## 4. Where `Unit::think@005f6e40` calls it

Three arms mention a merchant, and only one of them fires for an AI:

| where | gate | reached |
|---|---|---|
| the auto-attack block, `think:146` | `is(MERCHANTDUTCH 0x3e, 1)` and packed | the Dutch merchant only — it is the merchant with an attack |
| the rare-collector arm, `think:169`–`179` | `is_rare_collector` **and `leader_flags & 4`** and packed, on `idle == 1` or every 32 | **a human's** merchant or fishing boat: `do_gather(search)` and then `unpack_merchant(this, 4)` |
| step 5's own, `think:283` (`LAB_005f7515`) | `is_merchant`, on `idle == 1` or every **128** | every merchant, and this is the AI's |

**`leader_flags & 4` is *human*** — the same bit `Leader::new_rare@006d9e70`
tests to refuse a plain human's reveals (`docs/ECONOMY.md`) and
`crate::scout` reads as `& 2` in play, `& 4` human, `& 8` AI-driving-a-human.
So the middle row is a human player's convenience — walk your merchant onto
a rare and it deploys itself, with a message and `S_INVALID_ORDER` when the
spot search refuses — and an AI never enters it. `docs/ORDERS.md` §6 step 5
said "(AI only)" and had it backwards; the trace agrees with the flag, not
the prose: `Unit::unpack_merchant` is first entered on run54's frame
**6354**, the AI merchant's first idle frame, and never in the four
thousand frames its AI fishing boats spend idle before that.

**The cadence is 128, and it is the third period in this function.** The
mod-16 gate at the head decides whether `think` is entered; the mod-32 gate
above `think_fish` decides the tail; and this one, between `think_fish` and
`think_carry`, is `idle == 1 || ((o + frame) & 127) == 0`. A merchant that
finds nothing therefore re-searches four times more slowly than a fisherman
does.

---

## 5. A trained unit's crew is seated at birth (2026-09-02)

Not merchant-specific, and found here because the merchant is the first
unit this simulation ever *trained* with a **tracked** crew figure.

`Unit::init@00612100:548`–`549` is `update_gpiece(this)` and then
`set_new_location(this, x, y, 1, 1)`. That `param_3 = 1` reaches
`Guy::set_new_location@005d86f0` on every crew figure with its own
`des_x`/`des_y` — the track offset `Guy::update_gpiece@005d8530` read out
of the piece's art — so a crew figure is **placed** on its offset at birth
rather than left to walk there.

This crate seated the guys a dump handed it and the ones a transport put
ashore, and nothing else. The consequence is `Guy::do_turn@005d97a0`:

```
if this.guy_num == 0:
    for g in guys[squad_size .. length]:
        if g.track_dx == 0 and g.track_dy == 0:
            do_turn(g, …)                # and g may spend its own draw
```

A crew figure with a track offset is *not* recursed into; a trackless one
is, and — because a merchant packs, so every one of its figures carries
`guy_flags & 8` (`docs/ANIM.md` §4.8) — it asks for a turn animation the
merchant's art does not have, falls to the idle, and rolls. The Merchant's
piece has `trackdist="10"` in `unit_graphics.xml` on every one of its
`<UNIT>` entries, so the original's crew figure has an offset and stays
silent, and this crate's had none and turned beside it: **two
`Guy::set_anim+0x97a < Guy::do_turn+0x4a < Unit::move_step+0x389` draws a
frame where the original spends one**, for the three frames of the
merchant's first turn.

`Sim::init_guys` now ends with `Sim::seat_guys`, which is where
`Unit::init` puts it.

---

## 6. Coverage

**Diff-backed on run54** — the 24,000-frame East Indies capture, whose word
went from 6356 to 6570 when this landed, to 6571 with the collision probe
behind it (`docs/COLLISION.md` §4.2), to 6715 when §2.2.1 put the second
merchant on its own rare, and to **6739** when §3.1 put the deploy walk in
front of the unpack cast — and **on run66**, 260 blocks of the same
game over `[6340, 6600)`, which holds this merchant's walk from its birth
to its arrival stand: **every position of it is the original's**, frame for
frame, and so is every other unit's as far as the word
(`run66_s_window_is_the_original_s_unit_for_unit`, 13,600 fields). run67
adds sixty blocks of **every figure's whole record**, both merchants
included (`run67_s_window_is_every_figure_s_whole_record`, 28,890 fields,
zero differing). The three together agree draw for draw through the
merchant's think, its first turn, 214 frames of its walk, the collision
that ends it and the recovery after:

- the head's `unpack_merchant(3)` refusing at the Market (the trace enters
  `Unit::unpack_merchant` and `Unit::find_merchant_spot` on frame 6354 and
  the unit does not deploy);
- the cadence — the merchant thinks on its first idle frame, 6354, and the
  original's `think_merchant` is entered there and nowhere earlier;
- the order, and that it is a `QUEUE_NEW` `MOVE_TO`: the turn begins on
  6356, three frames after birth, at `Unit::move_step+0x389`;
- §5's crew seat, which is the difference between one turn draw a frame
  and two;
- **the second merchant's whole choice** — §2.2.1's ordered search, §2.3's
  rotation and the destination they produce. `1/20` is born on 6571 and
  the two rares it scores are the AI's only ones, so the pick is binary
  and the dump names it: `orders_x/y 28728/24120`, the *other* good, with
  a 22-entry path where `1/19`'s rare is seven away. `do_move`'s
  `length > 10` arm then holds its first step back to 6575, which is how
  the divergence showed — as a frame, not a destination.
- **the arrival, and the whole of §3** — run68's 123 blocks over
  `[6595, 6718)`, every unit's whole record, 122,752 fields
  (`run68_s_window_is_every_unit_s_whole_record_to_the_word`). §3.1 is
  the entry above; the ring's answer, the destination's snap and the
  angle's unsnapped point are all in the same block.

**Reading only** — no run on disk separates these:

- the scoring arithmetic itself. East Indies' AI has exactly two goods in
  `new_rares` and one of them is in its own region, so the pick is
  `300 > 190` and any of the three terms could be wrong without moving it.
  The `−10` step needs a third good; the danger term needs a war; the
  floor needs both.
- the three object searches' *shape*. The second's `orders_x/y` measure is
  diff-backed (§2.2.1) — it is the whole of the second merchant's pick —
  but all three answer "nothing" on run54's frame 6354, so the reading is
  still what says they are asked at the cell centre rather than at the
  good, and what says the type filter is exact.
- the ally test on the good's cell, and the ocean-bit equality. East
  Indies' two rares are unowned land.
- ~~the rotation~~ — **diff-backed since 2026-09-02**, through the second
  merchant's destination (above). What is still reading-only is the
  rotation's *arithmetic*: with two goods and one of them already ordered
  at, the ordered search alone decides, and a list that was never rotated
  would pick the same rare.
- ~~the whole of §3 past its gate … nothing on disk reaches a ring walk~~
  — **run68 reaches it, and §3 is diff-backed** (2026-09-03). `1/19`
  arrives at its `CITRUS` on block 6714, runs the ring, and both sides
  take tile **(168, 192)** — `MOVE_49`'s fourth entry, with entries 0, 1
  and 2 refused. So the gate, `good_merchant_spot`'s two-by-two, the
  `MOVE_289` walk order and `radius[3]` all have an oracle, and so do the
  order the two orders run in (§3.1), the 48-snap of the destination and
  the angle to the unsnapped point. What parted was §3.1 alone, and the
  window ~~holds to 6718 — a unit that is item 191's, not this
  mechanic's~~ **now holds to 6730, the last block run68 carries**: item
  191 closed on 2026-09-03 (`docs/COLLISION.md` §2.2) and took `1/13`'s
  position with it, so no field of any unit parts inside the window. **The ring is still one candidate deep**: the winner was the
  fourth entry of forty-nine, so nothing on disk exercises the walk past
  ring 1, and `detect_unit_collision` (§7) still refused nothing.

## 7. What is not established

- **`detect_unit_collision`, the third test of `find_merchant_spot`, is not
  asked.** This crate's is `&mut` — it writes the `collide_o`/`collide_who`
  bookkeeping every path out of the move step depends on — and a read-only
  form of it is its own item. A merchant may therefore deploy on a tile
  another unit is standing in.
- **The `-6 … -1` "over time" pieces.** `docs/ANIM.md` §3.4's merchant seam
  is still a seam, but it is now known to be **unreachable for a packed
  merchant**: `GraphicPieces::get_unit_gpiece@0090c030` takes the over-time
  arm only when its `param_5` — the packed flag — is zero. So the art a
  walking merchant uses is the ordinary arithmetic's `-PACKED` slot, which
  this crate already computes, and the seam bites only once a merchant has
  deployed.
- **The Dutch merchant's arm.** `think:146` calls `think_merchant` from
  inside the auto-attack block, ahead of everything else, for a packed
  `MERCHANTDUTCH`. No capture has one, and this crate does not model the
  arm.
- **`unit_masks &= ~0x100`**, which `unpack_merchant` clears before the
  cast. The bit is `Unit::work` step 3's ("this unit was ordered
  recently"), and nothing here keeps it.
- ~~**The walk's own end.**~~ — run68 has it: `1/19` stops at
  (32076, 37188) on block 6713, is idle on 6714, and turns for three
  frames before stepping at its deploy spot on 6718 (§3.1).
- **What happens at the good.** The **cast** — the deployed merchant's
  two-by-two footprint and the `rare`/`good_obj` pair
  `Unit::do_gather@005fce20` writes for it (`docs/ECONOMY.md` step 6) —
  is still past every capture. run68's window ends at 6730 with the
  merchant eleven frames into a walk it has not finished, so nothing on
  disk has ever seen a `MERCHANT` unpack: `unit_masks & 0x80000` clearing,
  the two-by-two `PLACED`, and whatever the cast order does when the walk
  in front of it dies. The capture's *closing* block, 6746, does carry the
  merchant standing on (32280, 36888) with its `SubObjectData.flags` 9 → 1
  — but at the `MISC` detail the quit writes, which is the object base and
  nothing else, and a closing block is not a frame state
  (`docs/ORACLE.md`, run68). The next window is `[6730, 6800)`.
