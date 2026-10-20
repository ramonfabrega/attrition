//! **`Unit::do_non_flat_gather@005f0170`'s tile choice** (item 3381): the head
//! (the site's recharge bump, the group back-pointer) and the arm a worker
//! takes when it is at the camp with no wait (`goto_build != 0`, `wait < 0`):
//! pick a resource tile from the site's list, roll the wait, set the carrying
//! walk. A pure walk over a [`GatherTileView`] — the sweep's port is a view
//! over the emulated original's windows, the crate's is [`SimGatherTile`]
//! (`docs/ORDERS.md` §6.4).
//!
//! The steps are the listing's, call for call, in the listing's order:
//!
//! 1. **the head** (`5f019d`..`5f0245`): a `been_there` order bumps the site's
//!    `recharging` once (three virtual calls of slot `+0xac`: test the flag,
//!    add to `+0x7a`, set the flag), and every call drops the unit's group
//!    back-pointer (`+0x80 = -1`).
//! 2. **the stand and the leader's flag** (`5f0253`..`5f027a`).
//! 3. **the held tile** (`5f027f`..`5f02ac`): kept while `has_gather_access`
//!    holds for it.
//! 4. **the choice** (`5f02b2`..`5f05fb`): the Doober hold of a ranged type
//!    dropped, a tiny range's distance weight capped at 3, the list scanned
//!    for the lowest score, the pick moved to the end of the list.
//! 5. **the wait** (`5f06a9`..`5f072c`): `% 200 + 400`, the order's
//!    `goto_build` cleared, and the carrying walk by the camp's resource.
//!
//! Not modelled, and not observable: the three `Error::report` asserts
//! ("GET PANCAKE.", "FIND PANCAKE.", "FIND PANCAKE!") — each is a debug
//! assert on an off-map tile that answers 0 and writes only the caller's own
//! stack; and the hold [`Doober`]s (`+0x86`), cosmetic, which the unit record
//! here does not carry ([`SimGatherTile::remove_doober`] is a no-op).
//!
//! One thing the sweep forces: **a tile that is held but no longer gatherable
//! stays held** when no replacement is found — `tx`/`ty` are written only by
//! the pick, so an empty or walled-in list leaves the stale tile in place (the
//! port used to clear it).

use crate::world::vector_dist;
use crate::{Pos, Sim};

/// The score a tile must beat (`local_2c = 0x98967f`): no tile scoring that
/// or more is ever chosen.
pub const NO_TILE: i32 = 0x98_967f;
/// The tile mask's blocked bit (`mask & 0x4000`).
pub const MASK_BLOCKED: u16 = 0x4000;
/// `UnitType::index` values `0x32..=0x35` hold a hold-Doober for the gather.
pub const DOOBER_TYPES: std::ops::RangeInclusive<i32> = 0x32..=0x35;
/// `unit_masks |= 0x40000000`: the walk to the ore.
pub const FLAG_TO_ORE: u32 = 0x4000_0000;
/// `unit_masks |= 0x10000000`: the walk to the wood.
pub const FLAG_TO_WOOD: u32 = 0x1000_0000;
/// The wait a miner is left with: it never decrements.
pub const MINER_WAIT: i32 = 1_000_000;

/// The fields of the `GatherOrder` the steps read and write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileOrder {
    pub tx: i32,
    pub ty: i32,
    pub wait: i32,
    pub goto_build: bool,
    pub dist_mod: i32,
    pub been_there: bool,
}

/// What the tile choice reads and does. Every method is one call or one
/// inline read of the listing.
pub trait GatherTileView {
    // ---- the head ----
    /// Slot `+0xac` of the building's object, then `(site+0x60) & 0x800`.
    fn site_recharge_flag(&mut self) -> bool;
    /// Slot `+0xac` again, then `site+0x7a += 1`.
    fn site_recharge_bump(&mut self);
    /// Slot `+0xac` a third time, then `site+0x60 |= 0x800`.
    fn site_recharge_flag_set(&mut self);
    /// `this+0x80 = -1`: the unit's group back-pointer.
    fn clear_group(&mut self);

    // ---- the arm ----
    /// `leaders[this+9].flags |= 0x2000000`.
    fn mark_leader(&mut self);
    /// `Unit::set_anim(CHAR_DEFAULT, 0, 1)`.
    fn stand(&mut self);
    /// `WorldData::has_gather_access(&order.tx, &order.ty, who, 1, 0)`, asked
    /// of the order's own tile.
    fn order_tile_gatherable(&mut self, tx: i32, ty: i32) -> bool;
    /// The same call, asked of a list tile (a stack copy).
    fn list_tile_gatherable(&mut self, x: i32, y: i32) -> bool;
    /// Slot `+0xac` of the building's object: the record the list lives in.
    fn site_enter(&mut self);
    /// The unit type's `+4`.
    fn unit_type_index(&mut self) -> i32;
    /// `this+0x86` as a short: the unit's hold-Doober id, negative for none.
    fn doober(&mut self) -> i32;
    /// `Doober::remove_hold_doobers(id)`, then `this+0x86 = -1`.
    fn remove_doober(&mut self, id: i32);
    /// The list's count (`site+0x9c`).
    fn tiles_len(&mut self) -> i32;
    /// `MiningList::mtn`, the signed byte at `site+0xb4`: negative for none.
    fn site_mtn(&mut self) -> i32;
    /// `constants.mtn_tiny_size`.
    fn mtn_tiny(&mut self) -> i32;
    /// The list's `i`th tile.
    fn tile_at(&mut self, i: i32) -> (i32, i32);
    /// The tile word at `(x, y)` (`world+0x138`, stride `world+0x18`).
    fn tile_mask(&mut self, x: i32, y: i32) -> u16;
    /// The building's tile: `div_3_table` of its position, both axes.
    fn building_tile(&mut self) -> (i32, i32);
    /// `Array<TCoordData>::remove((x, y))`.
    fn remove_tile(&mut self, x: i32, y: i32);
    /// The list's append: the pick goes to the end.
    fn push_tile(&mut self, x: i32, y: i32);
    /// `Random::get(game_random, 0, 0xffff)`.
    fn rnd(&mut self) -> i32;
    /// The camp is a Woodcutter (slot `+0xac`, then `is(0x1a2, 0)`).
    fn site_is_woodcutter(&mut self) -> bool;
    /// `this+0x68 |= bits`.
    fn or_unit_flags(&mut self, bits: u32);
}

/// The head: the recharge bump (a `been_there` order only) and the group.
pub fn gather_head_steps<V: GatherTileView>(v: &mut V, been_there: bool) {
    if been_there && !v.site_recharge_flag() {
        v.site_recharge_bump();
        v.site_recharge_flag_set();
    }
    v.clear_group();
}

/// The arm for `goto_build != 0 && wait < 0`: choose a tile, roll the wait.
pub fn gather_tile_steps<V: GatherTileView>(v: &mut V, o: &mut TileOrder) {
    if !o.been_there {
        o.been_there = true;
        v.mark_leader();
    }
    v.stand();
    let held = o.tx >= 0 && o.ty >= 0 && v.order_tile_gatherable(o.tx, o.ty);
    if !held {
        v.site_enter();
        let ty = v.unit_type_index();
        if DOOBER_TYPES.contains(&ty) {
            let d = v.doober();
            if d >= 0 {
                v.remove_doober(d);
            }
        }
        let mut weight = o.dist_mod;
        let count = v.tiles_len();
        if v.site_mtn() >= 0 && count < v.mtn_tiny() && weight > 3 {
            weight = 3;
        }
        if count < 1 {
            return;
        }
        let (bx, by) = v.building_tile();
        let mut best = NO_TILE;
        let mut pick = (0, 0);
        for i in 0..count {
            let (x, y) = v.tile_at(i);
            if v.tile_mask(x, y) & MASK_BLOCKED == 0 || !v.list_tile_gatherable(x, y) {
                continue;
            }
            let dist = vector_dist((x - bx).wrapping_abs(), (y - by).wrapping_abs()).max(3);
            let score = dist.wrapping_mul(weight).wrapping_add(i >> 2);
            if score < best {
                best = score;
                pick = (x, y);
            }
        }
        if best == NO_TILE {
            return;
        }
        v.remove_tile(pick.0, pick.1);
        v.push_tile(pick.0, pick.1);
        o.tx = pick.0;
        o.ty = pick.1;
    }
    o.wait = 400 + v.rnd() % 200;
    o.goto_build = false;
    if v.site_is_woodcutter() {
        v.or_unit_flags(FLAG_TO_WOOD);
    } else {
        v.or_unit_flags(FLAG_TO_ORE);
        o.wait = MINER_WAIT;
    }
}

/// The crate's view: the unit `u`'s order at the site `b`.
pub struct SimGatherTile<'a> {
    pub sim: &'a mut Sim,
    pub u: usize,
    pub b: usize,
}

impl SimGatherTile<'_> {
    fn mask_of(&self, x: i32, y: i32) -> u16 {
        self.sim.world.tile_mask(Pos::new(x, y))
    }
}

impl GatherTileView for SimGatherTile<'_> {
    fn site_recharge_flag(&mut self) -> bool {
        self.sim.buildings[self.b].gather_bumped
    }
    fn site_recharge_bump(&mut self) {
        self.sim.buildings[self.b].recharging += 1;
    }
    fn site_recharge_flag_set(&mut self) {
        self.sim.buildings[self.b].gather_bumped = true;
    }
    fn clear_group(&mut self) {
        self.sim.units[self.u].group_ptr = None;
    }
    fn mark_leader(&mut self) {
        let who = self.sim.units[self.u].owner;
        self.sim.ledgers[who as usize].dirty = true;
    }
    fn stand(&mut self) {
        self.sim.mark(crate::anim::SITE_STAND_GATHER);
        self.sim.set_default_anim(self.u);
    }
    fn order_tile_gatherable(&mut self, tx: i32, ty: i32) -> bool {
        let who = self.sim.units[self.u].owner;
        self.sim.has_gather_access(Pos::new(tx, ty), who)
    }
    fn list_tile_gatherable(&mut self, x: i32, y: i32) -> bool {
        let who = self.sim.units[self.u].owner;
        self.sim.has_gather_access(Pos::new(x, y), who)
    }
    fn site_enter(&mut self) {}
    fn unit_type_index(&mut self) -> i32 {
        // The Doober types are cosmetic here; none is a type this crate asks.
        -1
    }
    fn doober(&mut self) -> i32 {
        -1
    }
    fn remove_doober(&mut self, _id: i32) {}
    fn tiles_len(&mut self) -> i32 {
        self.sim.buildings[self.b].gather_from.len() as i32
    }
    fn site_mtn(&mut self) -> i32 {
        // `MiningList::mtn >= 0` is a Mine's range; the crate has no field.
        if self.sim.building_ident(self.b) == crate::build::Ident::Mine {
            0
        } else {
            -1
        }
    }
    fn mtn_tiny(&mut self) -> i32 {
        crate::gather::MOUNTAIN_GATHER[0].0
    }
    fn tile_at(&mut self, i: i32) -> (i32, i32) {
        let t = self.sim.buildings[self.b].gather_from[i as usize];
        (t.x, t.y)
    }
    fn tile_mask(&mut self, x: i32, y: i32) -> u16 {
        self.mask_of(x, y)
    }
    fn building_tile(&mut self) -> (i32, i32) {
        let t = self.sim.buildings[self.b].pos.tile();
        (t.x, t.y)
    }
    fn remove_tile(&mut self, x: i32, y: i32) {
        let list = &mut self.sim.buildings[self.b].gather_from;
        if let Some(at) = list.iter().position(|t| t.x == x && t.y == y) {
            list.remove(at);
        }
    }
    fn push_tile(&mut self, x: i32, y: i32) {
        self.sim.buildings[self.b].gather_from.push(Pos::new(x, y));
    }
    fn rnd(&mut self) -> i32 {
        self.sim.mark(crate::orders::SITE_TILE_WAIT);
        self.sim.rng.roll()
    }
    fn site_is_woodcutter(&mut self) -> bool {
        self.sim.building_ident(self.b) == crate::build::Ident::Woodcutter
    }
    fn or_unit_flags(&mut self, bits: u32) {
        self.sim.units[self.u].carry |= bits;
    }
}
