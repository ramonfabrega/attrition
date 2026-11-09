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

/// Whose record a read of [`CalcGatherView`] is about: the unit calling
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
pub trait CalcGatherView {
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
fn gather_ring<V: CalcGatherView>(v: &mut V, w: Who<V::H>) -> i32 {
    if merchant_id(v.type_index(w)) {
        return v.type_radius_base(w).wrapping_mul(4).wrapping_add(4);
    }
    if v.type_is_fisher(w) {
        v.type_radius_base(w).wrapping_mul(4).wrapping_add(4)
    } else {
        v.type_radius_base(w).wrapping_add(2)
    }
}

/// `UnitData::calc_gather@00609180`, step for step over a [`CalcGatherView`]:
/// the head, the remembered ring index (block A), the spiral (block B), and
/// the crowd count behind both. Answers the executable's `eax`.
pub fn unit_calc_gather_steps<V: CalcGatherView>(v: &mut V, a: &GatherArgs) -> i32 {
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
fn gather_tail<V: CalcGatherView>(
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

/// The `Sim` answering [`CalcGatherView`] for unit `u`. `found` and `divisor`
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

impl CalcGatherView for SimGather<'_> {
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
        self.s.terrain.chain_heads.get(slot).copied().flatten()
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
        let types = &self.tables.tech_tree.types;
        upgrade_steps(
            t,
            |c| types.get(c).and_then(|d| d.from),
            |next| self.tables.tech_tree.is(t, next, false),
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
        if MERCHANTS.contains(&t) || self.tables.tech_tree.is(t, crate::fish::FISHERMEN, false) {
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
            MERCHANTS.contains(&t) || self.tables.tech_tree.is(t, crate::fish::FISHERMEN, false)
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
mod tests;
