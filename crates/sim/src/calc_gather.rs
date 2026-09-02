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
use crate::world::{Cell, Pos, TILES_PER_CELL, UNITS_PER_TILE, tile, vector_dist};
use crate::{Sim, orders};

/// `UnitData::is_merchant@0046d370` / `TypeData::is_merchant@0042dbd0`: the
/// **exact ids** `MERCHANT`, `MERCHANTDUTCH` and `FURTRAPPER`, never a
/// lineage. The same three `UnitData::is_rare_collector@0046fae0` tests
/// before falling back to `is(FISHERMEN)`.
const MERCHANTS: [TypeId; 3] = [0x3d, 0x3e, 0x190];

/// `circle_radius[2]` is the ring the crowd count walks — in **cells**,
/// where the two searches above it are in tiles.
const CROWD_RING: usize = 2;

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
        let mut level = 0;
        let mut cur = t;
        loop {
            let Some(next) = self.tech_tree.types.get(cur).and_then(|d| d.from) else {
                return level;
            };
            if next != t && !self.tech_tree.is(t, next, false) {
                return level;
            }
            level += 1;
            cur = next;
            // The original's `FROM` chain is acyclic by construction; a
            // hand-built fixture's need not be, and `circle_radius` has 65
            // entries anyway.
            if level > 64 {
                return level;
            }
        }
    }

    /// The ring index `calc_gather` searches to, and the radius its crowd
    /// test measures in tiles: `upgrade_level × 4 + 4` for a merchant by id
    /// or anything in the `FISHERMEN` lineage, `upgrade_level + 2`
    /// otherwise (`00609243`..`0060927f`, and the same three arms again at
    /// `006097c4`..`00609816` for the other unit).
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

    /// The tile predicate both walks share: a merchant wants **land** and
    /// everyone else wants **ocean**, and either way the tile must carry
    /// [`tile::AS_BUILDING`] before the cell is asked for a good.
    fn gather_tile_ok(&self, merchant: bool, t: Pos) -> bool {
        let m = self.world.tile_mask(t);
        let ocean = m & tile::SURFACE == tile::SURFACE_OCEAN;
        if merchant == ocean {
            return false;
        }
        m & tile::AS_BUILDING != 0
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
        // `*param_1 = -1` and `*param_2 = 0`, unconditionally and before
        // every other test — so a caller that asks and finds nothing is
        // left with `rare == -1`, and `unit_masks & 0x20` is **cleared** on
        // every call with `param_7` set, which is this one.
        self.units[u].rare = -1;
        self.units[u].gather_here = false;
        if self.gather_search(u).is_none() {
            return false;
        }
        // `if (count == 0) return 1;` — and otherwise the rates are divided
        // by `count + 1` and, with `param_7` set, the answer is 0.
        self.gather_crowd(u, true) == 0
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
        self.units[u].rare = -1;
        self.units[u].gather_here = false;
        // `00609200`: the head, and it is only in the `param_7 == 0` form.
        if self.units[u].combat.packed
            && (self.is_merchant(u) || self.unit_line_is(u, crate::fish::FISHERMEN))
        {
            return None;
        }
        let good = self.gather_search(u)?;
        let crowd = self.gather_crowd(u, false);
        self.units[u].gather_here = crowd > 0;
        Some((good, crowd))
    }

    /// Blocks A and B — the search, and the two fields it writes.
    fn gather_search(&mut self, u: usize) -> Option<TypeId> {
        let who = self.units[u].owner;
        let merchant = self.is_merchant(u);
        let radius = self.gather_radius(u);
        let circle = crate::ai_place::circle();
        let ring = usize::try_from(radius)
            .ok()
            .and_then(|r| circle.radius.get(r).copied())
            .unwrap_or(0);
        let t0 = self.units[u].pos.tile();

        // **Block A** (`00609289`..`0060934e`): the remembered index, tried
        // first and on its own. A hit keeps `good_obj` where it is; a miss
        // on an otherwise acceptable tile clears it to `-1` before the
        // spiral runs, and a tile of the wrong surface leaves it alone.
        let cached = self.units[u].good_obj;
        if cached >= 0 && usize::try_from(cached).is_ok_and(|i| i < ring) {
            let i = cached as usize;
            let t = Pos::new(t0.x + circle.x[i], t0.y + circle.y[i]);
            if self.world.tile_in_bounds(t) && self.gather_tile_ok(merchant, t) {
                match self.find_good_at(tile_cell(t), who) {
                    Some(g) => {
                        self.units[u].rare = i32::try_from(g).unwrap_or(-1);
                        return Some(g);
                    }
                    None => self.units[u].good_obj = -1,
                }
            }
        }

        // **Block B** (`0060937d`..`00609585`): the spiral, in tile order,
        // taking the **first** tile that answers rather than the nearest
        // good — which is the same thing, since `circle_init` orders its
        // entries by ring.
        for i in 0..ring {
            let t = Pos::new(t0.x + circle.x[i], t0.y + circle.y[i]);
            if !self.world.tile_in_bounds(t) || !self.gather_tile_ok(merchant, t) {
                continue;
            }
            if let Some(g) = self.find_good_at(tile_cell(t), who) {
                self.units[u].good_obj = i16::try_from(i).unwrap_or(-1);
                self.units[u].rare = i32::try_from(g).unwrap_or(-1);
                return Some(g);
            }
        }
        None
    }

    /// `LAB_00609573`'s tail: how many other units of the finder's own
    /// lineage are close enough to be sharing the deposit. `packed_exempt`
    /// is `param_8`.
    fn gather_crowd(&self, u: usize, packed_exempt: bool) -> i32 {
        let Some(mine_ty) = self.unit_tree(u) else {
            return 0;
        };
        let who = self.units[u].owner;
        let mine = self.gather_radius(u);
        let at = self.units[u].pos;
        let c0 = at.cell();
        let circle = crate::ai_place::circle();
        let mut count = 0;
        for i in 0..circle.radius[CROWD_RING] {
            let c = Cell::new(c0.x + circle.x[i], c0.y + circle.y[i]);
            if !self.world.contains(c) {
                continue;
            }
            let slot = (c.y as usize) * (self.world.width() as usize) + (c.x as usize);
            let mut next = self.chain_heads[slot];
            while let Some(o) = next {
                next = self.units[o].down;
                // `local_20 == this->who`, then `is_active` (`+0x8`) and
                // `is_on_map` (`+0xbc`), then the lineage — the object's
                // `is` is `ObjectData::is`, so it falls through to the
                // type's own `is(this->type, 0)`.
                if o == u
                    || self.units[o].owner != who
                    || !self.units[o].alive()
                    || !self.units[o].on_map
                    || !self.unit_line_is(o, mine_ty)
                {
                    continue;
                }
                // `param_8 == 1`: a packed unit is skipped **unless it is
                // in the middle of unpacking**, which is the arm the zero
                // form has not got.
                if self.units[o].combat.packed && !(packed_exempt && self.is_unpacking(o)) {
                    continue;
                }
                let d = vector_dist(self.units[o].pos.x - at.x, self.units[o].pos.y - at.y);
                if d < (self.gather_radius(o) + mine) * UNITS_PER_TILE {
                    count += 1;
                }
            }
        }
        count
    }

    /// `UnitData::is_unpacking@0060a4b0`: the order at the head of the list
    /// is a craft whose spell `TypeData::is_unpack` (`+0x54`) accepts.
    fn is_unpacking(&self, u: usize) -> bool {
        matches!(
            self.units[u].orders.front().map(|o| o.body),
            Some(orders::Body::Cast(c)) if orders::spell::is_unpack(c.spell)
        )
    }
}

/// `iVar12 >> 2` — the tile's cell, floored, as the original's arithmetic
/// shift gives it off the left or top edge.
fn tile_cell(t: Pos) -> Cell {
    Cell::new(
        t.x.div_euclid(TILES_PER_CELL),
        t.y.div_euclid(TILES_PER_CELL),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{CellData, Good, Terrain, World};
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
