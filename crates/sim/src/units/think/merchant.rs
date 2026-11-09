//! Where an idle merchant goes — `Unit::think_merchant@005f4740` and the
//! spot search at its head.
//!
//! The specification is `docs/MERCHANT.md`. In one paragraph: a **packed**
//! merchant first asks whether it can deploy where it already stands
//! (`Unit::unpack_merchant@006038e0`, ring 3); failing that it scores its
//! leader's `new_rares` list — the goods-list indices `Leader::new_rare`
//! records as this player walks the fog (`crate::rares`) — as
//! `200 − 10·i`, `+100` for a good in its own `get_tregion`, minus the
//! danger map, floored at 1, with three object searches that refuse a
//! candidate outright; the winner is taken **out of the list and put back
//! on the end**, and a `MOVE_TO` at the good replaces everything the unit
//! was doing.
//!
//! It is the other half of `crate::rares`: a rare pays nothing until a
//! merchant stands on it (`docs/ECONOMY.md` step 6), and this is how the
//! merchant gets there. East Indies' AI trains its first on frame 6353 and
//! steps on 6356.

use crate::orders::{MoveKind, QueuePos};
use crate::world::{MOVE_289, Pos, UNITS_PER_CELL, UNITS_PER_TILE, tile, vector_dist};
use crate::{Player, Sim};

/// `Unit::think_merchant` up to `Unit::think_fish`, the next function in
/// the export — the range that isolates this mechanic's own draws in a
/// trace. It takes **none**: the whole function is arithmetic, and the
/// draws the original's frame 6356 grows are `Unit::move_step`'s, from the
/// order this issues.
pub const CODE: std::ops::Range<u32> = 0x005f_4740..0x005f_4c60;

/// The score the **first** entry of `new_rares` starts from (`005f47d8`:
/// `0xc8`), and it falls by ten a slot whether or not the slot is
/// accepted.
const FIRST_SCORE: i32 = 200;

/// What a slot's score falls by per list position.
const SCORE_STEP: i32 = 10;

/// The bonus for a good whose cell is in the merchant's own region
/// (`005f4a19`: `+ 0x64`).
const OWN_REGION: i32 = 100;

/// The radius, in world units, of the two friendly searches: one cell.
const FRIENDLY_RANGE: i32 = 0x300;

/// The radius of the enemy-combat search: four cells.
const ENEMY_RANGE: i32 = 0xc00;

/// `radius[3]` — how many entries of [`MOVE_289`] `unpack_merchant(3)`
/// walks. The table at `00add1e0` is `1, 9, 25, 49, 81, …`, the cumulative
/// count per ring, and `find_merchant_spot` indexes it with its own
/// argument.
const RING: [usize; 5] = [1, 9, 25, 49, 81];

impl Sim {
    // ------------------------------------------------------------------
    // §2 — `Unit::think_merchant@005f4740`
    // ------------------------------------------------------------------

    /// The whole of it. `true` ends the caller's `think`.
    ///
    /// **A deployed merchant returns 1 at once.** The head is
    /// `unit_masks & 0x80000` — *packed* — and a clear bit takes the
    /// early `return 1`, so a merchant that has already unpacked onto a
    /// rare never searches again and never falls through to the tail of
    /// `Unit::think` either (`docs/MERCHANT.md` §2.1).
    pub(crate) fn think_merchant(&mut self, u: usize) -> bool {
        if !self.units[u].combat.packed {
            return true;
        }
        if self.unpack_merchant(u, 3) {
            return true;
        }
        let who = self.units[u].owner;
        let here = self.units[u].pos;
        // `get_tregion` of the **merchant's own tile**, taken once
        // (`005f47b3`), and compared below against each good's *cell*
        // region — the plain `WData +4`, not `get_tregion` again
        // (`docs/PATHFINDER.md` §15 names the pair).
        let mine = self.world.tregion_alt(here.tile());
        let ocean = self.world.tile_mask(here.tile()) & tile::SURFACE == tile::SURFACE_OCEAN;

        let mut best: Option<usize> = None;
        let mut best_score = -1i32;
        let list = self.ai[who as usize].new_rares.clone();
        for (i, &gi) in list.iter().enumerate() {
            // `local_30`, decremented at the loop tail whatever the slot
            // did — so a refused good still costs the ones behind it ten.
            let base = FIRST_SCORE - SCORE_STEP * i32::try_from(i).unwrap_or(0);
            let Some(g) = self.world.goods().get(gi).copied() else {
                continue;
            };
            // `SubObjectData.flags & 1`, then `ever_seen & (1 << who)`.
            if !g.alive || !(self.good_seen_bit(gi, who) || self.good_ever_seen(g.pos, who)) {
                continue;
            }
            let c = g.pos.cell();
            // The good's cell's `WData.who` (`+0xf`): unowned — `-1`, and
            // `-2` with it — passes without asking, and an owner has to be
            // an ally.
            if let Some(owner) = self.world.owner(c).player()
                && !self.is_ally(who, owner)
            {
                continue;
            }
            // The **ocean bit of the two tiles**, compared for equality:
            // a land merchant will not walk at a good in the water and a
            // sea one will not walk at a good ashore. It is a tile read on
            // both sides, not a domain test.
            if (self.world.tile_mask(g.pos.tile()) & tile::SURFACE == tile::SURFACE_OCEAN) != ocean
            {
                continue;
            }
            // Everything below is asked at the **centre of the good's
            // cell**, not at the good.
            let at = Pos::new(
                c.x * UNITS_PER_CELL + UNITS_PER_CELL / 2,
                c.y * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            );
            if self.merchant_refused(u, at) {
                continue;
            }
            let mut score = base;
            if self.world.region_of(c) == mine {
                score += OWN_REGION;
            }
            score -= self.world.danger_half(who, c);
            score = score.max(1);
            // `if (local_24 < iVar5)` — strict, so a tie keeps the good
            // nearer the front of the list.
            if best_score < score {
                best_score = score;
                best = Some(gi);
            }
        }

        let Some(gi) = best else { return false };
        // `SimpleArray<int>::remove` finds the **value**, shifts the tail
        // down, and the good is appended — so the winner goes to the back
        // of the list and is the *last* thing the next merchant scores.
        // The list is a rotation, not a queue: nothing is ever dropped.
        let rares = &mut self.ai[who as usize].new_rares;
        if let Some(at) = rares.iter().position(|&x| x == gi) {
            rares.remove(at);
        }
        rares.push(gi);
        let Some(g) = self.world.goods().get(gi).copied() else {
            return false;
        };
        // The tail is `add_move_order(good, MOVE_TO, QUEUE_NEW, 0)`
        // inlined: `unit_masks &= ~0x4000000`, `path.length = 0`,
        // `close_orders`, `clear_partial_path`, `update_action`, then the
        // order — the 48-unit snap on the destination and the bearing to
        // the **unsnapped** good (`docs/ORDERS.md` §4.3).
        self.add_move_order(u, g.pos, MoveKind::MoveTo, QueuePos::New, false);
        true
    }

    /// The three object searches, in the original's order and each one
    /// only asked when the one before it found nothing (`005f493d`,
    /// `005f497c`, `005f49b0`). `true` refuses the good.
    ///
    /// 1. `find_unit(SEARCH_FRIENDLY, who, 0x300, FILTER_NOT_ME(o, who),
    ///    FILTER_TYPE(type))` — one of my own kind is already there.
    /// 2. `find_unit_ordered(…, 0x300, same filters)` — one of my own kind
    ///    is on its way. "Ordered" is literal twice over: the candidate
    ///    must hold a **move-family** order (`crate::scout`'s
    ///    `MOVE_FAMILY`), **and the distance is measured to where it is
    ///    going, not to where it stands** — `UnitData +0x70/+0x74`, which
    ///    is `orders_x`/`orders_y` (`docs/MERCHANT.md` §2.2.1).
    /// 3. `find_unit(SEARCH_ENEMY, who, 0xc00, FILTER_COMBAT)` — an enemy
    ///    that can shoot is within four cells.
    ///
    /// The type filter is the exact `TypeData.type`, not the lineage: a
    /// Fur Trapper does not keep a Merchant off a good.
    fn merchant_refused(&self, u: usize, at: Pos) -> bool {
        let who = self.units[u].owner;
        let ty = self.units[u].type_index;
        let near = |s: &Self, i: usize, r: i32| {
            let p = s.units[i].pos;
            vector_dist(p.x - at.x, p.y - at.y) <= r
        };
        // `find_unit_ordered`'s own measure: the candidate's
        // `orders_x`/`orders_y`, the point `update_action` last wrote.
        let heading_for = |s: &Self, i: usize, r: i32| {
            let p = s.units[i].orders_pos;
            vector_dist(p.x - at.x, p.y - at.y) <= r
        };
        // `SEARCH_FRIENDLY` (`SearchIndexBH` 1, the PDB's own record) with
        // `FILTER_NOT_ME(o, who)` and `FILTER_TYPE(type)` — and case 1 of
        // `Search::valid_search@0067daa0` is `param_2 != this → 0`: the
        // iterated leader must *be* the asker, so this is **my own**
        // units, never me, of exactly my `TypeData.type` — not my allies'
        // (`docs/MERCHANT.md` §6.1; the docs-versus-code pass of
        // 2026-09-05 found `is_ally` here). The leader loop stops at
        // eight, so gaia is not in the search space.
        let sibling = |s: &Self, i: usize| {
            let x = &s.units[i];
            i != u && x.alive() && x.on_map && !x.is_gaia() && x.owner == who && x.type_index == ty
        };
        if (0..self.units.len()).any(|i| sibling(self, i) && near(self, i, FRIENDLY_RANGE)) {
            return true;
        }
        if (0..self.units.len()).any(|i| {
            sibling(self, i)
                && crate::scout::MOVE_FAMILY.contains(&self.order_type(i))
                && heading_for(self, i, FRIENDLY_RANGE)
        }) {
            return true;
        }
        (0..self.units.len()).any(|i| {
            let x = &self.units[i];
            x.alive()
                && x.on_map
                && !x.is_gaia()
                && self.is_enemy(who, x.owner)
                && self.attack_of(crate::combat::Obj::Unit(i)) != 0
                && near(self, i, ENEMY_RANGE)
        })
    }

    /// `check_ever_seen`'s accumulation, as far as a simulation with no
    /// per-object byte can answer it: the good's own fog cell, read with
    /// `was_really_seen` — the bare line-of-sight accumulation, with none
    /// of `was_seen`'s ally-territory shortcut. [`crate::goody`] reads the
    /// same field of an item the same way and says why.
    /// The `ever_seen` bit `compute_reg_territory`'s goods scan
    /// ([`Sim::claim_cell_goods`]) or `reveal_fog`'s rare arm
    /// ([`Sim::reveal_fog`]) set on this good for `who`.
    pub(crate) fn good_seen_bit(&self, gi: usize, who: Player) -> bool {
        who < 8
            && self
                .territory
                .good_seen_bits
                .get(gi)
                .is_some_and(|b| b >> who & 1 != 0)
    }

    pub fn good_ever_seen(&self, at: Pos, who: Player) -> bool {
        let t = at.tile();
        self.was_really_seen_fog(t.x >> 1, t.y >> 1, who)
    }

    // ------------------------------------------------------------------
    // §3 — `Unit::unpack_merchant@006038e0`
    // ------------------------------------------------------------------

    /// Deploy here if here will do. `true` when a spot was found and the
    /// unpack ordered, which ends every caller.
    ///
    /// `think_merchant` asks with ring 3; `Unit::think`'s own
    /// rare-collector arm — **human-only**, `leader_flags & 4`
    /// (`docs/MERCHANT.md` §4) — asks with ring 4, and run127's human
    /// fishing boat is the capture that reaches it (`docs/ORDERS.md`
    /// §23).
    pub(crate) fn unpack_merchant(&mut self, u: usize, ring: usize) -> bool {
        let Some(t) = self.find_merchant_spot(u, ring) else {
            return false;
        };
        // `add_cast_order(-1, -1, …, 0x28c, QUEUE_NEW, 0)` — the unpack,
        // rewritten to the merchant family's `0x290` — and then the walk
        // to the spot, **in front of it**: the line after the list `add`
        // is `head = head->next`, which is exactly what
        // `add_cast_order`'s own `QUEUE_FIRST` arm runs
        // (`docs/ORDERS.md` §1.5). So the merchant walks to the spot and
        // casts on arrival, and run68's block 6714 is the oracle —
        // `orders_x/y` and `dest_angle` are the move's, and the unit
        // steps at it (`docs/MERCHANT.md` §3.1).
        self.add_cast_order_at(u, crate::orders::spell::UNPACK, QueuePos::New);
        let to = Pos::new(t.x * UNITS_PER_TILE, t.y * UNITS_PER_TILE);
        self.add_move_order(u, to, MoveKind::MoveTo, QueuePos::First, false);
        true
    }

    /// `Unit::find_merchant_spot@00603ab0`: the gate, then the ring.
    ///
    /// The gate is `calc_gather` **where the unit already stands** — so a
    /// merchant nowhere near a deposit leaves without walking a single
    /// candidate, and that is the whole of the head on East Indies' frame
    /// 6354. Behind it, `move_x`/`move_y` in [`MOVE_289`] order for
    /// `radius[ring]` entries, and the first tile that is a good merchant
    /// spot, a valid location and unoccupied wins.
    ///
    /// The third test is `detect_unit_collision(x·192, y·192, quick 1,
    /// boats 1, 0, 0, 0)` at the tile's **corner** (`00603ab0`): the quick
    /// form, which names nobody and writes nothing on a miss
    /// ([`Sim::detect_quick`]) — so a rare's own square, or a unit
    /// standing on one, refuses the spot (item 1611: Great Sahara's
    /// merchant `1/38` on 2205, the original's `(−1, −1)` against our
    /// `(0, 0)`).
    fn find_merchant_spot(&mut self, u: usize, ring: usize) -> Option<Pos> {
        #[cfg(test)]
        tests::SPOT_SEARCHES.with(|n| n.set(n.get() + 1));
        // Null out-pointers (`00603ab0:20`): the gate writes neither
        // `rare` nor `good_obj` (item 3306).
        if !self.calc_gather_at(u, self.units[u].pos, false) {
            return None;
        }
        let t0 = self.units[u].pos.tile();
        let n = RING.get(ring).copied().unwrap_or(0);
        for &(dx, dy) in MOVE_289.iter().take(n) {
            let t = Pos::new(t0.x + dx, t0.y + dy);
            if !self.world.tile_in_bounds(t) {
                continue;
            }
            if self.good_merchant_spot(u, t)
                && self.invalid_loc(u, t, false, false, false, false, false)
                    == crate::path::loc::VALID
                && !self.detect_quick(
                    u,
                    Pos::new(t.x * UNITS_PER_TILE, t.y * UNITS_PER_TILE),
                    false,
                )
            {
                return Some(t);
            }
        }
        None
    }

    /// `UnitData::good_merchant_spot@006068a0`: the **two-by-two** whose
    /// bottom-right corner is `t` — `(x, y)`, `(x−1, y)`, `(x, y−1)`,
    /// `(x−1, y−1)` — every tile of it on the map, unblocked
    /// (`tile::BLOCKED`), not a building footprint (`tile::OBJECT` all
    /// set) and not [`tile::PLACED`]; and then `calc_gather` at `t`'s own
    /// corner.
    pub(crate) fn good_merchant_spot(&mut self, u: usize, t: Pos) -> bool {
        for (dx, dy) in [(0, 0), (-1, 0), (0, -1), (-1, -1)] {
            let p = Pos::new(t.x + dx, t.y + dy);
            if !self.world.tile_in_bounds(p) {
                return false;
            }
            let m = self.world.tile_mask(p);
            if m & tile::BLOCKED != 0
                || m & tile::OBJECT == tile::OBJECT_BUILDING
                || m & tile::PLACED != 0
            {
                return false;
            }
        }
        self.calc_gather_at(
            u,
            Pos::new(t.x * UNITS_PER_TILE, t.y * UNITS_PER_TILE),
            false,
        )
    }
}

#[cfg(test)]
mod tests;
