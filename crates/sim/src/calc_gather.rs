//! `UnitData::calc_gather@00609180` — "is there a good under me still, and am
//! I alone on it?"
//!
//! The specification is `docs/ORDERS.md` §6.10. It is the head of
//! `Unit::think_fish` (§6.8) and the whole of `Unit::do_gather@005fce20`,
//! and it answers three things at once: the good's `TypeIndex` into
//! `UnitData::rare` (`+0x54`), the ring index it was found at into
//! `UnitData::good_obj` (`+0x94`), and — as its return value — whether the
//! caller may stay where it is.
//!
//! Both of its walks are in **tiles**, not cells, and both take the first
//! hit rather than the best: the octagonal spiral `circle_x`/`circle_y`
//! ([`crate::ai_place::circle`]) out to `circle_radius[r]`, where `r` is
//! `upgrade_level × 4 + 4` for a fisherman or a merchant and
//! `upgrade_level + 2` for anything else. A tile qualifies on its **surface
//! field** — ocean for a fisherman, anything but ocean for a merchant — and
//! on [`crate::world::tile::AS_BUILDING`]; the good itself is then looked
//! up in the tile's **cell**, which is why a boat standing on a fish
//! usually answers ring **1** rather than 0: the fish's object marks its
//! own tiles, and the boat is rarely on one of them.
//!
//! The return value is not "there is a good": it is "there is a good **and
//! nobody else of my kind is sharing it**". After a hit the function walks
//! the object chains of the `circle_radius[2]` cells around the unit,
//! counts the owner's other live, on-map, unpacked units of the same
//! lineage inside `(their radius + mine) × 192`, and divides the six gather
//! rates by `count + 1`. With a count of zero it answers 1 and the caller
//! stays put; with any competition at all it answers **0** and the caller
//! goes looking for somewhere else — which is what sends a crowded fishing
//! boat back through `think_fish`'s 17 × 17.

use crate::tech::TypeId;
use crate::world::{Cell, Pos, TILES_PER_CELL, vector_dist};
use crate::{Sim, orders};

/// `UnitData::is_merchant@0046d370` / `TypeData::is_merchant@0042dbd0`: the
/// **exact ids** `MERCHANT`, `MERCHANTDUTCH` and `FURTRAPPER`, never a
/// lineage. The same three `UnitData::is_rare_collector@0046fae0` tests
/// before falling back to `is(FISHERMEN)`.
const MERCHANTS: [TypeId; 3] = [0x3d, 0x3e, 0x190];

/// `circle_radius[2]` is the ring the crowd count walks — in **cells**,
/// where the two searches above it are in tiles.
const CROWD_RING: usize = 2;

/// Whose record a read of [`GatherView`] is about: the unit calling
/// `calc_gather`, or one of the others on the crowd walk.
#[derive(Clone, Copy)]
pub enum Who<H> {
    Me,
    Other(H),
}

/// `UnitData::calc_gather@00609180`'s ten stack arguments, as the nullness
/// and the numbers the body tests: `param_1`, `param_2` and `param_3` are the
/// out-pointers a caller hands or nulls (`rare`, `flag`, `good`), `param_5`
/// the `BitMask<44>` and `param_6` the per-good tally (`mask`, `tally`);
/// `param_4`, the six rates, is never null. `x` and `y` are `param_9` and
/// `param_10`, and `x == -1` is the original's "the unit's own place".
#[derive(Clone, Copy, Debug)]
pub struct GatherArgs {
    pub rare: bool,
    pub flag: bool,
    pub good: bool,
    pub mask: bool,
    pub tally: bool,
    pub p7: i32,
    pub p8: i32,
    pub x: i32,
    pub y: i32,
}

/// Everything `UnitData::calc_gather` reads or writes outside its own
/// locals, one method a distinct access, so an emulated window and the
/// `Sim` answer the same questions in the same order (item 3330).
pub trait GatherView {
    /// A unit as the chain walk names it: the `Sim`'s index, the sweep's
    /// `(list, index)` pair.
    type H: Copy;
    /// `*(this->type + 4)` — the type's index.
    fn type_index(&self, w: Who<Self::H>) -> i32;
    /// The type's `is(0x13d, 0)` (vtable `+0x60`).
    fn type_is_fisher(&mut self, w: Who<Self::H>) -> bool;
    /// The type's `get_upgrade_level` (vtable `+0x124`).
    fn type_radius_base(&mut self, w: Who<Self::H>) -> i32;
    /// `unit_masks & 0x80000`.
    fn packed(&self, w: Who<Self::H>) -> bool;
    /// `field_0x10 ^ 0x63637` and `field_0x14 ^ 0x63637`.
    fn pos(&self, w: Who<Self::H>) -> (i32, i32);
    /// The byte at `this + 9`.
    fn me_owner(&self) -> i32;
    /// The short at `this + 0x94`.
    fn good_obj(&self) -> i32;
    /// `div_3_table[i]`.
    fn div3(&self, i: i32) -> i32;
    /// The world's `+0x18` and `+0x1c`: the tile grid.
    fn tile_dims(&self) -> (i32, i32);
    /// The world's `+0` and `+4`: the cell grid.
    fn cell_dims(&self) -> (i32, i32);
    /// The tile mask at `tile_width * y + x`.
    fn tile_mask(&self, x: i32, y: i32) -> u16;
    /// The signed byte at `+0xf` of the cell at `(cx, cy)`.
    fn cell_owner(&self, cx: i32, cy: i32) -> i32;
    /// The head of the cell's chain, `None` for a negative index.
    fn cell_head(&self, cx: i32, cy: i32) -> Option<Self::H>;
    /// The object's `+0x2c`/`+0x2e` link, `None` for a negative index.
    fn next(&self, h: Self::H) -> Option<Self::H>;
    /// The list the chain element lives in — its owner.
    fn list_of(&self, h: Self::H) -> i32;
    /// `Object` vtable `+0x8`.
    fn active(&mut self, h: Self::H) -> bool;
    /// `Object` vtable `+0xbc`.
    fn on_map(&mut self, h: Self::H) -> bool;
    /// `Object` vtable `+0xb8`, `is(ty, 0)`.
    fn is_line(&mut self, h: Self::H, ty: i32) -> bool;
    /// `UnitData::is_unpacking@0060a4b0`.
    fn unpacking(&mut self, h: Self::H) -> bool;
    /// The chain index against the short at `this + 0xa`.
    fn is_self(&self, h: Self::H) -> bool;
    /// `vector_dist@0046cff0(|dx|, |dy|)` — `ecx`, `edx`.
    fn vector_dist(&mut self, dx: i32, dy: i32) -> i32;
    /// `ObjectsData::find_good_at@0065bec0(cx, cy, who, 0, 0)`, called with
    /// whatever `ecx` the caller last held — block A's tile `y` origin,
    /// block B's `cx` — which the callee never reads and the sweep records.
    fn find_good(&mut self, cx: i32, cy: i32, who: i32, ecx: i32) -> i32;
    /// `LeaderData::is_ally@006edb50` on `leaders[who]`.
    fn is_ally(&mut self, who: i32, owner: i32) -> bool;
    /// `leaders[who]`'s `+0x8`.
    fn leader_id(&self, who: i32) -> i32;
    /// `leaders[a]`'s `+0x74 + 4 * b`.
    fn relation(&self, a: i32, b: i32) -> i32;
    /// `LeaderData::calc_rare@006e08d0(good, rates, ally)`.
    fn calc_rare(&mut self, good: i32, ally: bool);
    /// The six rates zeroed.
    fn rates_zero(&mut self);
    /// The six rates divided by `n`.
    fn rates_div(&mut self, n: i32);
    /// `BitMask<44>`: the bit `good - 6`, and the flags cleared.
    fn mask_set(&mut self, good: i32);
    /// `tally[good - 6] += 1`.
    fn tally_inc(&mut self, good: i32);
    /// `*param_1`.
    fn out_rare(&mut self, v: i32);
    /// `*param_2`.
    fn out_flag(&mut self, v: i32);
    /// `*param_3`.
    fn out_good(&mut self, v: i32);
}

/// `is_merchant@0046d370`'s three ids, as the body inlines them.
fn merchant_id(t: i32) -> bool {
    matches!(t, 0x3d | 0x3e | 0x190)
}

/// The ring index the searches run to and the radius the crowd test sums:
/// `upgrade_level × 4 + 4` for a merchant id or a fisherman, `+ 2` for
/// anything else. The merchant ids skip the `is(0x13d)` call.
fn gather_ring<V: GatherView>(v: &mut V, w: Who<V::H>) -> i32 {
    if merchant_id(v.type_index(w)) {
        return v.type_radius_base(w).wrapping_mul(4).wrapping_add(4);
    }
    if v.type_is_fisher(w) {
        v.type_radius_base(w).wrapping_mul(4).wrapping_add(4)
    } else {
        v.type_radius_base(w).wrapping_add(2)
    }
}

/// `UnitData::calc_gather@00609180`, step for step over a [`GatherView`]:
/// the head, the remembered ring index (block A), the spiral (block B), and
/// the crowd count behind both. Answers the executable's `eax`.
pub fn unit_calc_gather_steps<V: GatherView>(v: &mut V, a: &GatherArgs) -> i32 {
    // `*param_1 = -1`, `*param_2 = 0`, the six rates to zero — before every
    // other test.
    if a.rare {
        v.out_rare(-1);
    }
    if a.flag {
        v.out_flag(0);
    }
    v.rates_zero();
    // The head: a packed fisherman or merchant answers nothing, but only
    // through the `param_7 == 0` form.
    if (merchant_id(v.type_index(Who::Me)) || v.type_is_fisher(Who::Me))
        && v.packed(Who::Me)
        && a.p7 == 0
    {
        return 0;
    }
    let (px, py) = if a.x == -1 {
        v.pos(Who::Me)
    } else {
        (a.x, a.y)
    };
    let mine = gather_ring(v, Who::Me);
    let who = v.me_owner();
    let circle = crate::ai_place::circle();
    let ring = usize::try_from(mine)
        .ok()
        .and_then(|r| circle.radius.get(r).copied())
        .unwrap_or(0);
    let (tx, ty) = (v.div3(px >> 6), v.div3(py >> 6));

    // **Block A**: the remembered index, tried first and on its own.
    let cached = v.good_obj();
    if cached >= 0 && (cached as usize) < ring {
        let i = cached as usize;
        let (x, y) = (tx.wrapping_add(circle.x[i]), ty.wrapping_add(circle.y[i]));
        let (tw, th) = v.tile_dims();
        if x >= 0 && y >= 0 && x < tw && y < th {
            let merchant = merchant_id(v.type_index(Who::Me));
            let m = v.tile_mask(x, y);
            let ocean = m & 0x30 == 0x20;
            if merchant != ocean {
                if m & 0x200 != 0 {
                    let g = v.find_good(x >> 2, y >> 2, who, ty);
                    if g >= 0 {
                        let o = v.cell_owner(x >> 2, y >> 2);
                        let ally = o >= 0 && v.is_ally(who, o);
                        return gather_tail(v, a, g, ally, (px, py), mine);
                    }
                }
                if a.good {
                    v.out_good(-1);
                }
            }
        }
    }

    // **Block B**: the spiral, first tile that answers.
    for i in 0..ring {
        let (x, y) = (tx.wrapping_add(circle.x[i]), ty.wrapping_add(circle.y[i]));
        let (tw, th) = v.tile_dims();
        if x < 0 || y < 0 || x >= tw || y >= th {
            continue;
        }
        let merchant = merchant_id(v.type_index(Who::Me));
        let m = v.tile_mask(x, y);
        let ocean = m & 0x30 == 0x20;
        if merchant == ocean || m & 0x200 == 0 {
            continue;
        }
        let g = v.find_good(x >> 2, y >> 2, who, x >> 2);
        if g < 0 {
            continue;
        }
        if a.good {
            v.out_good(i as i32);
        }
        let o = v.cell_owner(x >> 2, y >> 2);
        let mut ally = false;
        if o >= 0 {
            let id = v.leader_id(who);
            ally = o == id || (v.relation(who, o) == 2 && v.relation(o, id) == 2);
        }
        return gather_tail(v, a, g, ally, (px, py), mine);
    }
    0
}

/// `LAB_00609573`: the writes a hit makes, `calc_rare`, and the crowd count
/// that decides the answer.
fn gather_tail<V: GatherView>(
    v: &mut V,
    a: &GatherArgs,
    g: i32,
    ally: bool,
    (px, py): (i32, i32),
    mine: i32,
) -> i32 {
    if a.rare {
        v.out_rare(g);
    }
    if a.mask {
        v.mask_set(g);
    }
    if a.tally {
        v.tally_inc(g);
    }
    v.calc_rare(g, ally);
    let circle = crate::ai_place::circle();
    let (cx0, cy0) = (v.div3(px >> 8), v.div3(py >> 8));
    let mut count = 0i32;
    for i in 0..circle.radius[CROWD_RING] {
        let (cx, cy) = (cx0.wrapping_add(circle.x[i]), cy0.wrapping_add(circle.y[i]));
        if cx < 0 || cy < 0 {
            continue;
        }
        let (cw, ch) = v.cell_dims();
        if cx >= cw || cy >= ch {
            continue;
        }
        let mut cur = v.cell_head(cx, cy);
        while let Some(h) = cur {
            cur = v.next(h);
            if v.list_of(h) != v.me_owner() || !v.active(h) || !v.on_map(h) {
                continue;
            }
            let ty = v.type_index(Who::Me);
            if !v.is_line(h, ty) {
                continue;
            }
            // `param_8 == 0`: any packed unit is skipped. Otherwise a packed
            // unit is skipped unless it is in the middle of unpacking.
            if v.packed(Who::Other(h)) && (a.p8 == 0 || !v.unpacking(h)) {
                continue;
            }
            if v.is_self(h) {
                continue;
            }
            let (mx, my) = v.pos(Who::Me);
            let (ox, oy) = v.pos(Who::Other(h));
            let d = v.vector_dist(
                ox.wrapping_sub(mx).wrapping_abs(),
                oy.wrapping_sub(my).wrapping_abs(),
            );
            let theirs = gather_ring(v, Who::Other(h));
            if d < theirs.wrapping_add(mine).wrapping_mul(192) {
                count += 1;
            }
        }
    }
    if count == 0 {
        return 1;
    }
    v.rates_div(count + 1);
    if a.p7 != 0 {
        return 0;
    }
    if a.flag {
        v.out_flag(1);
    }
    1
}

/// The `Sim` answering [`GatherView`] for unit `u`. `found` and `divisor`
/// are what the callers that want the rates read back: the good
/// `calc_rare` was asked about, and the `count + 1` the rates were divided
/// by (1 when they were not).
pub(crate) struct SimGather<'a> {
    s: &'a mut Sim,
    u: usize,
    pub(crate) found: Option<TypeId>,
    pub(crate) divisor: i32,
}

impl<'a> SimGather<'a> {
    pub(crate) fn new(s: &'a mut Sim, u: usize) -> Self {
        SimGather {
            s,
            u,
            found: None,
            divisor: 1,
        }
    }

    fn unit(&self, w: Who<usize>) -> usize {
        match w {
            Who::Me => self.u,
            Who::Other(o) => o,
        }
    }
}

impl GatherView for SimGather<'_> {
    type H = usize;
    fn type_index(&self, w: Who<usize>) -> i32 {
        self.s
            .unit_tree(self.unit(w))
            .map_or(-1, |t| i32::try_from(t).unwrap_or(-1))
    }
    fn type_is_fisher(&mut self, w: Who<usize>) -> bool {
        self.s.unit_line_is(self.unit(w), crate::fish::FISHERMEN)
    }
    fn type_radius_base(&mut self, w: Who<usize>) -> i32 {
        self.s
            .unit_tree(self.unit(w))
            .map_or(0, |t| self.s.upgrade_level(t))
    }
    fn packed(&self, w: Who<usize>) -> bool {
        self.s.units[self.unit(w)].combat.packed
    }
    fn pos(&self, w: Who<usize>) -> (i32, i32) {
        let p = self.s.units[self.unit(w)].pos;
        (p.x, p.y)
    }
    fn me_owner(&self) -> i32 {
        i32::from(self.s.units[self.u].owner)
    }
    fn good_obj(&self) -> i32 {
        i32::from(self.s.units[self.u].good_obj)
    }
    fn div3(&self, i: i32) -> i32 {
        i.div_euclid(3)
    }
    fn tile_dims(&self) -> (i32, i32) {
        (
            self.s.world.width() * TILES_PER_CELL,
            self.s.world.height() * TILES_PER_CELL,
        )
    }
    fn cell_dims(&self) -> (i32, i32) {
        (self.s.world.width(), self.s.world.height())
    }
    fn tile_mask(&self, x: i32, y: i32) -> u16 {
        self.s.world.tile_mask(Pos::new(x, y))
    }
    fn cell_owner(&self, cx: i32, cy: i32) -> i32 {
        self.s
            .world
            .owner(Cell::new(cx, cy))
            .player()
            .map_or(-1, i32::from)
    }
    fn cell_head(&self, cx: i32, cy: i32) -> Option<usize> {
        let slot = (cy as usize) * (self.s.world.width() as usize) + (cx as usize);
        self.s.chain_heads.get(slot).copied().flatten()
    }
    fn next(&self, h: usize) -> Option<usize> {
        self.s.units[h].down
    }
    fn list_of(&self, h: usize) -> i32 {
        i32::from(self.s.units[h].owner)
    }
    fn active(&mut self, h: usize) -> bool {
        self.s.units[h].alive()
    }
    fn on_map(&mut self, h: usize) -> bool {
        self.s.units[h].on_map
    }
    fn is_line(&mut self, h: usize, ty: i32) -> bool {
        usize::try_from(ty).is_ok_and(|t| self.s.unit_line_is(h, t))
    }
    fn unpacking(&mut self, h: usize) -> bool {
        self.s.is_unpacking(h)
    }
    fn is_self(&self, h: usize) -> bool {
        h == self.u
    }
    fn vector_dist(&mut self, dx: i32, dy: i32) -> i32 {
        vector_dist(dx, dy)
    }
    fn find_good(&mut self, cx: i32, cy: i32, who: i32, _ecx: i32) -> i32 {
        self.s
            .find_good_at(Cell::new(cx, cy), who as crate::world::Player)
            .map_or(-1, |g| i32::try_from(g).unwrap_or(-1))
    }
    fn is_ally(&mut self, who: i32, owner: i32) -> bool {
        self.s
            .is_ally(who as crate::world::Player, owner as crate::world::Player)
    }
    fn leader_id(&self, who: i32) -> i32 {
        who
    }
    fn relation(&self, a: i32, b: i32) -> i32 {
        if self
            .s
            .allied_with(a as crate::world::Player, b as crate::world::Player)
        {
            2
        } else {
            0
        }
    }
    fn calc_rare(&mut self, good: i32, _ally: bool) {
        self.found = usize::try_from(good).ok();
    }
    fn rates_zero(&mut self) {}
    fn rates_div(&mut self, n: i32) {
        self.divisor = n;
    }
    fn mask_set(&mut self, _good: i32) {}
    fn tally_inc(&mut self, _good: i32) {}
    fn out_rare(&mut self, v: i32) {
        self.s.units[self.u].rare = v;
    }
    fn out_flag(&mut self, v: i32) {
        self.s.units[self.u].gather_here = v != 0;
    }
    fn out_good(&mut self, v: i32) {
        self.s.units[self.u].good_obj = v as i16;
    }
}

/// **`ObjectTypeData::upgrade_level@00661090`**'s walk: up `from` (the
/// type record's `+0x3c`, `None` once negative) from `t`, counting each
/// ancestor `is` answers for and stopping at the first it does not. The
/// original's `is` is its vtable `+0x60` — inlined as `t == next`, then
/// the `is_list`, then `is_slow` (`+0xf4`) when the list is empty — which
/// the caller supplies. The original's `FROM` chain is acyclic by
/// construction; a hand-built fixture's need not be, so the walk stops
/// past 64. Item 2276's sweep holds it.
pub(crate) fn upgrade_steps(
    t: TypeId,
    from: impl Fn(TypeId) -> Option<TypeId>,
    mut is: impl FnMut(TypeId) -> bool,
) -> i32 {
    let mut level = 0;
    let mut cur = t;
    while let Some(next) = from(cur) {
        if !is(next) {
            break;
        }
        level += 1;
        cur = next;
        if level > 64 {
            break;
        }
    }
    level
}

impl Sim {
    /// `ObjectTypeData::upgrade_level@00661090`: how many steps up the
    /// `FROM` chain are still the same thing as this type.
    ///
    /// The original walks `from` from the type itself and stops at the
    /// first ancestor that is not in the starting type's own lineage —
    /// `is(ancestor, 0)`, inlined as equality, then the `is_list`, then
    /// `is_slow` (vtable `+0xf4`), which is exactly
    /// [`crate::tech::TechTree::is`]. A type with `<FROM>none</FROM>` — the
    /// shipped Fishermen — answers 0.
    pub(crate) fn upgrade_level(&self, t: TypeId) -> i32 {
        let types = &self.tech_tree.types;
        upgrade_steps(
            t,
            |c| types.get(c).and_then(|d| d.from),
            |next| self.tech_tree.is(t, next, false),
        )
    }

    /// The ring index `calc_gather` searches to, and the radius its crowd
    /// test measures in tiles: `upgrade_level × 4 + 4` for a merchant by id
    /// or anything in the `FISHERMEN` lineage, `upgrade_level + 2`
    /// otherwise (`00609243`..`0060927f`, and the same three arms again at
    /// `006097c4`..`00609816` for the other unit).
    #[cfg(test)]
    pub(crate) fn gather_radius(&self, u: usize) -> i32 {
        let Some(t) = self.unit_tree(u) else { return 2 };
        let level = self.upgrade_level(t);
        if MERCHANTS.contains(&t) || self.tech_tree.is(t, crate::fish::FISHERMEN, false) {
            level * 4 + 4
        } else {
            level + 2
        }
    }

    /// `UnitData::is_merchant@0046d370`.
    pub(crate) fn is_merchant(&self, u: usize) -> bool {
        self.unit_tree(u).is_some_and(|t| MERCHANTS.contains(&t))
    }

    /// `UnitData::is_rare_collector@0046fae0`: the three merchant **ids**,
    /// or anything in the `FISHERMEN` lineage — `is_merchant`'s own three
    /// tests with `is(0x13d)` behind them instead of nothing.
    ///
    /// Two callers here: `Unit::do_cast`'s unpack arm, where it gates the
    /// re-seat (`crate::Sim::do_cast`, step 2), and `Unit::think`'s human
    /// rare-collector arm (`docs/ORDERS.md` §23).
    pub(crate) fn is_rare_collector(&self, u: usize) -> bool {
        self.unit_tree(u).is_some_and(|t| {
            MERCHANTS.contains(&t) || self.tech_tree.is(t, crate::fish::FISHERMEN, false)
        })
    }

    /// `UnitData::calc_gather@00609180` as **`Unit::think_fish`** reaches
    /// it — `param_7 = 1`, `param_8 = 1`, and the position defaulted to the
    /// unit's own.
    ///
    /// Answers the original's return value: `true` is its `1`, "there is a
    /// good here and it is mine alone".
    ///
    /// The six gather rates (`param_4`), the `BitMask<44>` and the per-good
    /// tally (`param_5`, `param_6`) are null at this site; the caller that
    /// wants them is [`Sim::do_gather_rates`], below.
    pub(crate) fn calc_gather(&mut self, u: usize) -> bool {
        self.calc_gather_at(u, self.units[u].pos, true)
    }

    /// The same with the position the original's `param_9`/`param_10`
    /// carry, which `UnitData::good_merchant_spot@006068a0` uses to ask
    /// the question at a **candidate tile** rather than at the unit
    /// ([`crate::merchant`] §3).
    ///
    /// `live` is whether the caller hands the out-pointers. `Unit::do_gather`
    /// and `Unit::think_fish` pass `&rare` (`+0x54`) and `&good_obj`
    /// (`+0x94`); `Unit::find_merchant_spot@00603ab0:20` and
    /// `UnitData::good_merchant_spot@006068a0:50` pass null for `param_1`,
    /// `param_2` and `param_3`, and every write of the three is behind its
    /// null test — so a merchant's spot search reads the remembered index
    /// and writes nothing (item 3306: run680's `1/21`, `rare` 25 and
    /// `good_obj` 0 in ours against the original's 0 and −1 on block 1191).
    ///
    /// The body is [`unit_calc_gather_steps`], which item 3330's sweep
    /// holds row for row against the executable.
    pub(crate) fn calc_gather_at(&mut self, u: usize, at: Pos, live: bool) -> bool {
        let args = GatherArgs {
            rare: live,
            flag: live,
            good: live,
            mask: false,
            tally: false,
            p7: 1,
            p8: 1,
            x: at.x,
            y: at.y,
        };
        unit_calc_gather_steps(&mut SimGather::new(self, u), &args) != 0
    }

    /// `Unit::do_gather@005fce20` — `UnitData::calc_gather` as
    /// **`Leader::calc_gather` step 6** reaches it: `param_7 = 0`,
    /// `param_8 = 0`, the rate array and the two rare outputs live.
    ///
    /// Answers the good the unit is standing on and how many others of its
    /// own lineage share it, so the caller can pay
    /// [`crate::economy::calc_rare`] divided by `crowd + 1` and light the
    /// good's bit in `rare_owned`. `None` is the original's "no good here",
    /// which includes its **first** test: a packed fisherman or merchant
    /// answers nothing at all, and only this caller has that test.
    ///
    /// `Unit::do_gather`'s own tail is the `unit_masks & 0x20` write, and
    /// through this path the bit means "**sharing** a deposit": the
    /// original sets it only where the crowd count came out non-zero.
    pub(crate) fn do_gather_rates(&mut self, u: usize) -> Option<(TypeId, i32)> {
        let at = self.units[u].pos;
        let args = GatherArgs {
            rare: true,
            flag: true,
            good: true,
            mask: false,
            tally: false,
            p7: 0,
            p8: 0,
            x: at.x,
            y: at.y,
        };
        let mut v = SimGather::new(self, u);
        unit_calc_gather_steps(&mut v, &args);
        let (found, divisor) = (v.found, v.divisor);
        found.map(|g| (g, divisor - 1))
    }

    /// `UnitData::is_unpacking@0060a4b0`: the order at the head of the list
    /// is a craft whose spell `TypeData::is_unpack` (`+0x54`) accepts.
    pub(crate) fn is_unpacking(&self, u: usize) -> bool {
        matches!(
            self.units[u].orders.front().map(|o| o.body),
            Some(orders::Body::Cast(c)) if orders::spell::is_unpack(c.spell)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{CellData, Good, Terrain, UNITS_PER_TILE, World, tile};
    use crate::{Tuning, tech};

    /// `TypeIndex::FISH`.
    const A_GOOD: TypeId = 6;

    /// A 40 × 40 all-ocean world with one AI player and a Fisherman at the
    /// centre of cell `(20, 20)` — `fish.rs`'s fixture with the **tile**
    /// grid filled in, which is the half this walk reads and that one does
    /// not.
    fn sea_sim() -> (Sim, usize) {
        let mut world = World::new(40, 40);
        world.fill_region(Terrain::Sea, Cell::new(0, 0), Cell::new(39, 39));
        for y in 0..40 {
            for x in 0..40 {
                world.set_cell_data(
                    Cell::new(x, y),
                    CellData {
                        land: 2,
                        down: -1,
                        ..CellData::default()
                    },
                );
                for ty in 0..TILES_PER_CELL {
                    for tx in 0..TILES_PER_CELL {
                        world.set_tile_mask(
                            Pos::new(x * TILES_PER_CELL + tx, y * TILES_PER_CELL + ty),
                            tile::SURFACE_OCEAN,
                        );
                    }
                }
            }
        }
        let mut s = Sim::new(Tuning::RON, world, 2);
        s.nation[0].human = true;
        s.nation[1].human = false;
        {
            let t = &mut s.tech_tree;
            for n in [
                "Food",
                "Timber",
                "Wealth",
                "Knowledge",
                "Metal",
                "Oil",
                "Fish",
            ] {
                t.add(tech::TypeDef::good(n));
            }
            while t.types.len() < crate::fish::FISHERMEN {
                t.add(tech::TypeDef::plain("filler", -1));
            }
            let boat = t.add(tech::TypeDef::unit(
                "Fishing Boat",
                tech::UnitTraits::default(),
            ));
            assert_eq!(
                boat,
                crate::fish::FISHERMEN,
                "the fixture's ids are TypeIndex's"
            );
        }
        s.tech = (0..2)
            .map(|_| tech::PlayerTech::new(&s.tech_tree))
            .collect();
        let t = s.add_unit_type(crate::UnitType {
            hits: 20,
            moves: 40,
            tree: Some(crate::fish::FISHERMEN),
            ..crate::UnitType::default()
        });
        s.unit_types[t].combat.packs = true;
        s.unit_types[t].combat.domain = crate::attrition::Domain::Sea;
        let at = Pos::new(20 * 768 + 384, 20 * 768 + 384);
        let mut u = crate::Unit::new(1, 14, at, 20);
        u.ty = Some(t);
        let i = s.add_unit(u);
        (s, i)
    }

    /// Puts a good in a cell the way the map loader does, and marks the
    /// tile it sits on — `Objects::init_good` writes the cell's chain
    /// terminator and `Object::add_to_world` the tile's own bit, and the
    /// **two are not the same tile as the boat's**.
    fn put_good(s: &mut Sim, at: Pos, ty: TypeId) {
        s.world.add_good(Good {
            pos: at,
            ty,
            alive: true,
        });
        let t = at.tile();
        let m = s.world.tile_mask(t);
        s.world.set_tile_mask(t, m | tile::AS_BUILDING);
    }

    /// `<FROM>none</FROM>` on the shipped Fishermen, so the radius is the
    /// bottom of the ladder: four tiles, one cell.
    #[test]
    fn a_type_with_no_from_searches_four_tiles() {
        let (s, u) = sea_sim();
        assert_eq!(s.upgrade_level(s.unit_tree(u).unwrap()), 0);
        assert_eq!(s.gather_radius(u), 4);
    }

    /// Nothing anywhere: `rare` goes to `-1`, `good_obj` stands, and the
    /// answer is "no, move".
    #[test]
    fn an_empty_sea_answers_no_and_leaves_rare_at_minus_one() {
        let (mut s, u) = sea_sim();
        assert!(!s.calc_gather(u));
        assert_eq!(s.units[u].rare, -1);
        assert_eq!(s.units[u].good_obj, -1);
        assert!(!s.units[u].gather_here);
    }

    /// **The 5106 shape.** A fish in the boat's own cell but not under its
    /// own tile: ring 0 fails the `AS_BUILDING` test and ring **1** — the
    /// spiral's first neighbour — carries it, which is the `good_obj 1`
    /// run58's dump prints from frame 4992 on.
    #[test]
    fn a_fish_beside_the_boat_s_tile_answers_yes_at_ring_one() {
        let (mut s, u) = sea_sim();
        let t0 = s.units[u].pos.tile();
        let circle = crate::ai_place::circle();
        let one = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
        put_good(
            &mut s,
            Pos::new(one.x * UNITS_PER_TILE + 96, one.y * UNITS_PER_TILE + 96),
            A_GOOD,
        );
        let seed = s.rng.seed;
        assert!(s.calc_gather(u), "alone on it");
        assert_eq!(s.units[u].rare, i32::try_from(A_GOOD).unwrap());
        assert_eq!(s.units[u].good_obj, 1);
        assert_eq!(s.rng.seed, seed, "and the whole walk spends no draw");
    }

    /// The remembered index is retried on its own and keeps its value —
    /// block A, which is what makes the field state rather than an output.
    #[test]
    fn the_remembered_index_is_retried_first_and_kept() {
        let (mut s, u) = sea_sim();
        let t0 = s.units[u].pos.tile();
        let circle = crate::ai_place::circle();
        let one = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
        put_good(
            &mut s,
            Pos::new(one.x * UNITS_PER_TILE + 96, one.y * UNITS_PER_TILE + 96),
            A_GOOD,
        );
        assert!(s.calc_gather(u));
        assert_eq!(s.units[u].good_obj, 1);
        assert!(s.calc_gather(u), "the second call takes block A");
        assert_eq!(s.units[u].good_obj, 1);
    }

    /// A remembered index whose tile is still water but has lost its good
    /// is cleared to `-1` before the spiral runs.
    #[test]
    fn a_remembered_index_that_has_lost_its_good_is_cleared() {
        let (mut s, u) = sea_sim();
        let t0 = s.units[u].pos.tile();
        let circle = crate::ai_place::circle();
        let one = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
        // The tile keeps its `AS_BUILDING` bit and the cell loses its good.
        let m = s.world.tile_mask(one);
        s.world.set_tile_mask(one, m | tile::AS_BUILDING);
        s.units[u].good_obj = 1;
        assert!(!s.calc_gather(u));
        assert_eq!(s.units[u].good_obj, -1);
    }

    /// **The crowd test is the return value.** A second boat of the same
    /// lineage, unpacked, inside `(4 + 4) × 192`, turns the same fish from
    /// "stay" into "go" — and it is `rare` that says the good was found
    /// either way.
    #[test]
    fn a_second_boat_on_the_same_fish_turns_the_answer_round() {
        let (mut s, u) = sea_sim();
        let t0 = s.units[u].pos.tile();
        let circle = crate::ai_place::circle();
        let one = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
        put_good(
            &mut s,
            Pos::new(one.x * UNITS_PER_TILE + 96, one.y * UNITS_PER_TILE + 96),
            A_GOOD,
        );
        let ty = s.units[u].ty;
        let mut other = crate::Unit::new(1, 15, s.units[u].pos, 20);
        other.ty = ty;
        // `add_unit` is `Object::add_to_world`: it puts the boat in the
        // cell's chain itself.
        let o = s.add_unit(other);
        s.units[o].combat.packed = false;
        assert!(!s.calc_gather(u), "somebody else is sharing it");
        assert_eq!(
            s.units[u].rare,
            i32::try_from(A_GOOD).unwrap(),
            "the good was found; the count is what refuses it"
        );
        // A **packed** neighbour is not competition at all.
        s.units[o].combat.packed = true;
        assert!(s.calc_gather(u));
    }

    /// Another player's boat on the same fish is nobody's competition:
    /// the chain walk is fenced on `who` before anything else.
    #[test]
    fn another_player_s_boat_is_not_counted() {
        let (mut s, u) = sea_sim();
        let t0 = s.units[u].pos.tile();
        let circle = crate::ai_place::circle();
        let one = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
        put_good(
            &mut s,
            Pos::new(one.x * UNITS_PER_TILE + 96, one.y * UNITS_PER_TILE + 96),
            A_GOOD,
        );
        let ty = s.units[u].ty;
        let mut other = crate::Unit::new(0, 15, s.units[u].pos, 20);
        other.ty = ty;
        // `add_unit` is `Object::add_to_world`: it puts the boat in the
        // cell's chain itself.
        let o = s.add_unit(other);
        s.units[o].combat.packed = false;
        assert!(s.calc_gather(u));
    }

    /// Oil is in no cell's chain and is refused by type besides, so a boat
    /// sitting on an oil patch is sitting on nothing.
    #[test]
    fn oil_is_not_a_good_calc_gather_can_find() {
        let (mut s, u) = sea_sim();
        let t0 = s.units[u].pos.tile();
        let circle = crate::ai_place::circle();
        let one = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
        put_good(
            &mut s,
            Pos::new(one.x * UNITS_PER_TILE + 96, one.y * UNITS_PER_TILE + 96),
            crate::world::OIL,
        );
        assert!(!s.calc_gather(u));
        assert_eq!(s.units[u].rare, -1);
    }

    /// A **land** tile is refused for anything that is not a merchant, and
    /// the fish under it with it.
    #[test]
    fn a_fisherman_refuses_a_land_tile() {
        let (mut s, u) = sea_sim();
        let t0 = s.units[u].pos.tile();
        let circle = crate::ai_place::circle();
        let one = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
        put_good(
            &mut s,
            Pos::new(one.x * UNITS_PER_TILE + 96, one.y * UNITS_PER_TILE + 96),
            A_GOOD,
        );
        let m = s.world.tile_mask(one);
        s.world
            .set_tile_mask(one, (m & !tile::SURFACE) | tile::AS_BUILDING);
        assert!(!s.calc_gather(u), "the surface field is the first test");
    }
}
