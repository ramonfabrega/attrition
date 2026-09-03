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
- **"Ordered" is literal.** `ObjectsData::find_unit_ordered@0065bc40`
  requires the candidate to hold a **move-family** order (`{1, 2, 3, 4,
  0x12, 0x13, 0x15}`, `docs/ORDERS.md` §1.2), so a sibling standing at the
  good is caught by the first search and a sibling walking to it by the
  second; a sibling gathering somewhere else is caught by neither.
- **The floor is 1, and it is only reachable through danger.** With
  `base ≥ 200 − 10·(list length)` and the bonus non-negative, the `max(·,
  1)` matters exactly where the danger map is large — which is the war
  case, and no capture on disk has one.

Ties keep the **earlier** good: the compare is `best < score`, strict.

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
    add_move_order-shaped MOVE_TO at (t.x * 0xc0, t.y * 0xc0), appended
    clear_partial_path; update_action
    return 1
```

`Unit::add_cast_order@005e4a60`'s head rewrites the generic `0x28c` by the
caster: a machine gun's `0x28e`, the **three merchant ids'** `0x290`, the
`FISHERMEN` lineage's `0x292`. And the `MOVE_TO` goes on with
`LinkListBase::add`, which is the **back** of the queue
(`docs/ORDERS.md` §1.5) — so the cast is the current order and the walk is
behind it.

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
went from 6356 to 6570 when this landed, and which now agrees draw for draw
through the merchant's think, its first turn and 214 frames of its walk:

- the head's `unpack_merchant(3)` refusing at the Market (the trace enters
  `Unit::unpack_merchant` and `Unit::find_merchant_spot` on frame 6354 and
  the unit does not deploy);
- the cadence — the merchant thinks on its first idle frame, 6354, and the
  original's `think_merchant` is entered there and nowhere earlier;
- the order, and that it is a `QUEUE_NEW` `MOVE_TO`: the turn begins on
  6356, three frames after birth, at `Unit::move_step+0x389`;
- §5's crew seat, which is the difference between one turn draw a frame
  and two.

**Reading only** — no run on disk separates these:

- the scoring arithmetic itself. East Indies' AI has exactly two goods in
  `new_rares` and one of them is in its own region, so the pick is
  `300 > 190` and any of the three terms could be wrong without moving it.
  The `−10` step needs a third good; the danger term needs a war; the
  floor needs both.
- the three object searches. All three answer "nothing" on run54's frame
  6354, so the reading is what says they are searches at the cell centre
  rather than at the good, and what says the type filter is exact.
- the ally test on the good's cell, and the ocean-bit equality. East
  Indies' two rares are unowned land.
- the rotation. It is observable only through a **second** merchant
  choosing differently, and the original trains one on 6571 — past the
  frame this crate now parts on.
- the whole of §3 past its gate: `good_merchant_spot`'s four tiles, the
  ring order, and the cast-then-walk order pair. Nothing on disk reaches a
  ring walk, because a merchant that reaches its good is a merchant that
  has walked further than any capture follows one.

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
- **What happens at the good.** The arrival, the cast, the deployed
  merchant's two-by-two footprint and the `rare`/`good_obj` pair
  `Unit::do_gather@005fce20` writes for it are `docs/ECONOMY.md` step 6's,
  and the frame East Indies' merchant reaches its `CITRUS` is past the
  current word.
