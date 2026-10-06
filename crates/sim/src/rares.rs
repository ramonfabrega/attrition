//! `Leader::calc_gather`'s **step 6** — every idle fisherman and merchant —
//! and the one thing owning a rare does to a unit.
//!
//! The specification is `docs/ECONOMY.md`, step 6 and "What an owned rare
//! does". Three pieces, and they are one mechanic because the middle one is
//! the only reason the first has a visible effect this early in a game:
//!
//! 1. **The walk.** `Leader::calc_gather@006ceee0` clears `rare_owned`, then
//!    walks the player's objects in `o` order and hands every `is(FISHERMEN)`
//!    or `UnitData::is_merchant` whose `order_type` is `NONE` to
//!    `Unit::do_gather@005fce20`. Each answers the good it stands on; the
//!    good's payout ([`economy::calc_rare`]) is divided by the crowd sharing
//!    it and added to the player's rate, and the good's bit goes into
//!    `rare_owned`.
//! 2. **The mask.** `Leader::gather@006ce280` then sets `rare =
//!    rare_owned | rare_conquest` and, when that differs from last frame's,
//!    raises `LeaderData`'s `0x4000000`. One bit of the forty-four is
//!    singled out there: when **Gems** ([`economy::GEMS`], bit 23) moves,
//!    and only then, the writer invalidates every region's border stamp —
//!    so [`Sim::tick`] re-runs `sync_territory` on that change alone
//!    (`docs/ATTRITION.md`, "Territory").
//! 3. **`Leader::calc_unit_stats@006cf970`**, which `Leader::process` runs on
//!    the same frame the flag goes up, calls `Unit::update_speed@006055c0` on
//!    every one of that player's units. Its one rare arm is **Whales**:
//!    a naval type's cached speed becomes `(WHALES_SHIPS_MOVE + 100) / 100`
//!    of itself.
//!
//! That third step is what makes this measurable rather than merely correct.
//! East Indies' AI puts a second Fisherman on a whale on frame 5551 and every
//! ship it owns goes from 38 to 45 in the same frame — the Fisherman walking
//! to its own deposit then arrives twenty-three frames earlier, and the word
//! is where that arrival is.
//!
//! # What is not modelled
//!
//! - **The Porcelain Tower's pass**, which sets `rare_owned` bits for every
//!   rare inside the player's own territory whether anyone stands on it or
//!   not. It is the one exception to "a rare pays only through a merchant"
//!   and it is the wonder layer's.
//! - **`rare_conquest`**, Conquer-the-World's own mask; always empty here.
//! - **Everything else `update_speed` does** — the transport and marine
//!   bonuses, Bantu, French siege,
//!   Versailles, aluminium, the spy/general upgrade counts and the two
//!   Aztec types (`docs/MOVEMENT.md`, "The speed pipeline"). None of them is
//!   modelled anywhere in this crate, so recomputing a unit's cached speed
//!   from its type's `MOVES` loses nothing that was ever there.
//! - **`calc_unit_stats`' other three calls** — `calc_attrition`,
//!   `calc_anti_attrition` and `Unit::update_armor`.

use crate::economy::{self, RESOURCES};

/// The gunpowder foot line's `TypeIndex`es, the four `update_speed` tests
/// by identity (`enums/TypeIndex.txt`: 98, 100, 102, 104).
const ARQUEBUSIERS: i32 = 0x62;
const RIFLEMAN: i32 = 0x64;
const INFANTRY: i32 = 0x66;
const MECH_INFANTRY: i32 = 0x68;
use crate::tech::TypeId;
use crate::world::Player;
use crate::{Sim, orders};

impl Sim {
    /// `LeaderData::get_fishermen@006d6e80` — the highest `FISHERMEN`
    /// bonus row whose prerequisite the player holds, 0 for none.
    pub(crate) fn fishermen_level(&self, who: Player) -> usize {
        self.bonus_level(who, &self.tech_tree.roles.fishermen_preq)
    }

    /// `LeaderData::get_merchants_level@006d6dc0`.
    pub(crate) fn merchants_level(&self, who: Player) -> usize {
        self.bonus_level(who, &self.tech_tree.roles.merchants_preq)
    }

    /// The shared shape of the two: the rows are tested most advanced
    /// first and the answer is that row's one-based position. A row the
    /// tree does not know is not held — the opposite of the *bonus* rule
    /// ([`crate::tech::Roles::colonize_preq`]), because here a missing
    /// name must not invent an upgrade the player never bought.
    pub(crate) fn bonus_level(&self, who: Player, rows: &[Option<TypeId>]) -> usize {
        for (i, row) in rows.iter().enumerate().rev() {
            let Some(t) = *row else { continue };
            if self
                .tech_tree
                .has_tech(&self.setup, &self.tech[who as usize], t)
            {
                return i + 1;
            }
        }
        0
    }

    /// The tier of a **government's bonus ladder** held — `DESPOTISM_1..3` or
    /// `REPUBLIC_1..3`, tested most advanced first like [`Sim::bonus_level`]
    /// but with `has_preq`'s own gate on a tier above the first: its `preq0`,
    /// two governments taken (three for the third tier), and every tier under
    /// it held the same way (twenty-fourth pass, group 21; A1 rows 5 and 6).
    /// Every shipped tier carries one constant, so no price moves today.
    pub(crate) fn government_bonus_level(&self, who: Player, rows: &[Option<TypeId>]) -> usize {
        let p = &self.tech[who as usize];
        let held = |tier: usize| -> bool {
            (0..=tier).all(|k| {
                let Some(t) = rows.get(k).copied().flatten() else {
                    return false;
                };
                let below = k.checked_sub(1).and_then(|b| rows[b]);
                self.tech_tree.has_tech(&self.setup, p, t)
                    && self.tech_tree.bonus_tier_extra(&self.setup, p, k, below)
            })
        };
        (0..rows.len())
            .rev()
            .find(|&i| held(i))
            .map_or(0, |i| i + 1)
    }

    /// Step 6 whole: the rates it adds and the `rare_owned` it rebuilds.
    ///
    /// Called from [`Sim::assemble_holdings`], which runs inside the same
    /// cadence gate the original's recompute does — so a rare appears on
    /// exactly the frame it appears there.
    pub(crate) fn gather_rares(&mut self, who: Player) -> ([i32; RESOURCES], u64) {
        let mut out = [0; RESOURCES];
        let mut owned = 0u64;
        let candidates: Vec<usize> = (0..self.units.len())
            .filter(|&u| {
                let unit = &self.units[u];
                unit.owner == who && unit.alive() && unit.on_map
            })
            .collect();
        for u in candidates {
            // `is(0x13d, 0) || is_merchant`, then `order_type() == NONE`.
            // A merchant walking somewhere earns nothing, which is also
            // why a boat only ever pays between two `think_fish` searches.
            if !(self.unit_line_is(u, crate::fish::FISHERMEN) || self.is_merchant(u)) {
                continue;
            }
            if self.order_type(u) != orders::index::NONE {
                continue;
            }
            let Some((good, crowd)) = self.do_gather_rates(u) else {
                continue;
            };
            let Some(g) = self.good_types.get(good).copied() else {
                continue;
            };
            // The ally test the good's own cell answers — `is_ally` in the
            // cached arm and the two-sided diplomacy compare in the
            // spiral's, which are the same predicate.
            let ally = self.gather_cell_friendly(u, who);
            let rate = economy::calc_rare(
                &self.tuning,
                good,
                &g,
                self.fishermen_level(who),
                self.merchants_level(who),
                ally,
                self.tech_tree.has_tribe_bonus(
                    &self.setup,
                    &self.tech[who as usize],
                    crate::ai::tribe::NUBIANS,
                ),
            );
            let share = crowd + 1;
            for (i, r) in rate.iter().enumerate() {
                out[i] += r / share;
            }
            if good >= economy::BASE_RARE && good - economy::BASE_RARE < economy::RARES {
                owned |= 1 << (good - economy::BASE_RARE);
            }
        }
        (out, owned)
    }

    /// Whether the tile the unit found its good on is the finder's own
    /// ground or an ally's — `local_20` in `UnitData::calc_gather`.
    ///
    /// The original reads the **good's** cell rather than the unit's; both
    /// walks record the index they accepted, so the cell is the one
    /// `good_obj` names.
    fn gather_cell_friendly(&self, u: usize, who: Player) -> bool {
        let i = self.units[u].good_obj;
        let Ok(i) = usize::try_from(i) else {
            return false;
        };
        let circle = crate::ai_place::circle();
        let (Some(&dx), Some(&dy)) = (circle.x.get(i), circle.y.get(i)) else {
            return false;
        };
        let t = self.units[u].pos.tile();
        let c = crate::world::Cell::new(
            (t.x + dx).div_euclid(crate::world::TILES_PER_CELL),
            (t.y + dy).div_euclid(crate::world::TILES_PER_CELL),
        );
        let Some(owner) = self.world.owner(c).player() else {
            return false;
        };
        owner == who
            || (self.allied_both_ways(who as usize, owner as usize)
                && self.allied_both_ways(owner as usize, who as usize))
    }

    fn allied_both_ways(&self, a: usize, b: usize) -> bool {
        self.allied
            .get(a)
            .and_then(|r| r.get(b))
            .copied()
            .unwrap_or(false)
    }

    /// `Unit::update_speed@006055c0`, as much of it as this crate models:
    /// the type's `MOVES`, the **Whales** arm, and the **gunpowder foot
    /// line's** hardcoded correction after it (`docs/MOVEMENT.md`, "The speed
    /// pipeline" §1).
    ///
    /// The foot line is four **lineage** tests, `is(t, 0)` through the type's
    /// vslot `0x60` — `ObjectTypeData::is@0065f7d0`, read off the PE at
    /// `0xb41d24` and `0xb41fd4` (twenty-fourth pass, group 22): the type
    /// itself, then its `is_list`, then the graft and `from` chain — and one
    /// scale each, in the order Mechanized Infantry, Infantry, Rifleman,
    /// Arquebusiers, every division truncating toward zero (the listing,
    /// `605700`..`6057d6`). So every national replacement of a line takes its
    /// scale (thirty-one shipped types, the adjudicator's count; an earlier
    /// reading here called it identity and moved four exact indices only):
    /// Mechanized Infantry `×36/32`, Infantry `×34/32`, Rifleman `×32/27`,
    /// Arquebusiers `×30/24`. run404 prints Infantry at **34** on a `MOVES`
    /// of 32 from its birth on 621 (item 1109).
    /// The supply-bit upgrade adds `level * speed / 4` after those terms
    /// (SUPPLY, "Upgrade speed", item 1441).
    pub(crate) fn type_speed(&self, who: Player, ty: usize) -> i32 {
        let mut speed = self.unit_types[ty].moves;
        let naval = self.unit_types[ty].combat.obj_masks & crate::combat::mask::NAVAL != 0;
        if naval && self.has_rare(who, economy::WHALES) {
            speed = (self.tuning.whales_ships_move + 100) * speed / 100;
        }
        let lineage = |x: i32| {
            usize::try_from(self.unit_types[ty].type_index)
                .is_ok_and(|t| self.tech_tree.is(t, x as usize, false))
        };
        let (num, den) = if lineage(MECH_INFANTRY) {
            (36, 32)
        } else if lineage(INFANTRY) {
            (34, 32)
        } else if lineage(RIFLEMAN) {
            (32, 27)
        } else if lineage(ARQUEBUSIERS) {
            (30, 24)
        } else {
            (1, 1)
        };
        speed = speed * num / den;
        // `Unit::update_speed@006055c0`'s trainer arm: a unit whose
        // `WHERE` (`UnitTypeData +0x40`) is `0x1ae` or `0x1af` — the Siege
        // Factory line — moves `FRENCH_SIEGE_MOVE` faster for a French
        // leader, then `VERSAILLES_UNITS_MOVE` under Versailles, each its
        // own `(x + 100) × speed / 100` (`docs/MOVEMENT.md`, "The French
        // siege move"). French East Indies' Supply Wagon `1/72` is born at
        // 30 there, 25 here before item 1455.
        if matches!(self.trainer_where(ty), Some(0x1ae | 0x1af)) {
            if self.nation[who as usize].french {
                speed = (self.tuning.french_siege_move + 100) * speed / 100;
            }
            if self.wonders_held(who) & (1 << crate::tech::wonder::VERSAILLES) != 0 {
                speed = (self.tuning.versailles_units_move + 100) * speed / 100;
            }
        }
        if self.unit_types[ty]
            .cols
            .flag2(crate::ai_load::uflags2::SUPPLY_OR_HERO)
        {
            speed += self.supply_upgrade_level(who) * speed / 4;
        }
        speed
    }

    /// `LeaderData::has_rare@006e0770` — a good's bit in the player's
    /// `rare` mask, by **type index** rather than by bit. A good below
    /// [`economy::BASE_RARE`] is not a rare and is never in it, which is
    /// what the original's own `param_1 + -6 >> 3` would index off the
    /// front of.
    ///
    /// The original reads two masks and takes their union,
    /// `rare | rare_conquest`; this crate keeps the union itself in
    /// [`crate::economy::Ledger::rare`], which `Sim::tick` rebuilds every
    /// frame as `Leader::gather` does, so the one field answers both.
    pub fn has_rare(&self, who: Player, good: usize) -> bool {
        let Some(l) = self.ledgers.get(who as usize) else {
            return false;
        };
        good >= economy::BASE_RARE
            && good - economy::BASE_RARE < economy::RARES
            && l.rare >> (good - economy::BASE_RARE) & 1 != 0
    }

    /// `World::reveal_fog@006b3d30`'s rare arm — the only part of that
    /// function this simulation has (`docs/ECONOMY.md`, "The rares a leader
    /// has seen").
    ///
    /// `(fx, fy)` is a **fog** cell, and the call happens exactly where
    /// `World::set_seen` answered that `seen2` changed, so a cell is offered
    /// once per player for the life of a game. The original's own gate is a
    /// tile-mask read at `(2fx + 1, 2fy + 1)` — [`crate::world::tile`]'s
    /// `0x200`, which `Objects::init_good` sets over a good's footprint —
    /// and only then the cell's `find_good_at`.
    pub(crate) fn reveal_fog(&mut self, fx: i32, fy: i32, who: Player) {
        let t = crate::Pos::new(2 * fx + 1, 2 * fy + 1);
        if self.world.tile_mask(t) & crate::world::tile::AS_BUILDING == 0 {
            return;
        }
        // `find_good_at(x, y, who, 1, 0)` — the **index** form, which
        // answers before `type_avail` and so hands `new_rare` goods the
        // caller could not yet build with. The alive and non-`OIL` tests
        // are the same two [`Sim::find_good_at`] makes.
        let c = crate::world::Cell::new(fx >> 1, fy >> 1);
        let Some((gi, g)) = self.world.good_at(c) else {
            return;
        };
        if !g.alive || g.ty == crate::world::OIL {
            return;
        }
        self.new_rare(who, gi);
    }

    /// **Replay the start-of-game rare reveals over an installed fog
    /// grid** — the reconstruction a harness owes a simulation it starts
    /// from a snapshot rather than from `Game::init`.
    ///
    /// [`Sim::reveal_fog`] is reached from one place, `World::set_seen`
    /// answering that `seen2` **changed**, so a good is offered to a
    /// leader once for the life of a game. A harness that *installs* the
    /// dump's `seen2` instead of walking the sweeps that produced it
    /// therefore skips every offer the original had already made when the
    /// block was written, and `new_rares` starts empty where the
    /// original's already holds the rares the leader had seen. Nothing
    /// downstream can recover them: the cells are seen, so `set_seen` will
    /// never answer true for them again.
    ///
    /// That is what sent Great Lakes' AI Merchant to the wrong rare.
    /// `1/24` is born on 5753 and `think_merchant` scores `new_rares`; the
    /// original's list holds the `SILK` at `(40320, 14208)` — seen at
    /// game start, so first in the list and worth the full 200 — and this
    /// crate's held only the two goods a unit walked past *during* the
    /// run, so it picked the one at `(31800, 21816)` and set off
    /// south-west across the map (`docs/MERCHANT.md` §2.2, item 207).
    ///
    /// **The order is the grid's, not the sweep's**, and that is the one
    /// thing this cannot reconstruct: `new_rare` appends, and the score's
    /// base falls ten a slot, so two goods first seen in the same
    /// start-of-game reveal would be ranked by fog-cell order here and by
    /// the reveal's own order there. Captures so far carry at most one
    /// such good a leader, so the difference has never been observable;
    /// `docs/MERCHANT.md` §7 keeps it as an open question rather than a
    /// claim.
    pub fn seed_new_rares_from_fog(&mut self) {
        if !self.world.has_fog() {
            return;
        }
        let (fw, fh) = (self.world.fog_xs(), self.world.fog_ys());
        for fy in 0..fh {
            for fx in 0..fw {
                let Some(bits) = self.world.seen2(fx, fy) else {
                    continue;
                };
                if bits == 0 {
                    continue;
                }
                for who in 0..self.players.len().min(8) {
                    if bits >> who & 1 != 0 {
                        self.reveal_fog(fx, fy, who as Player);
                    }
                }
            }
        }
    }

    /// `Leader::new_rare@006d9e70`: record a seen good on this leader and
    /// on every ally that can still be told about it.
    ///
    /// The four gates, in the original's order:
    ///
    /// * the **caller** is not a plain human — `leader_flags & 0xc != 4`,
    ///   so a human's own reveals record nothing unless the AI is driving
    ///   it (`& 8`, never set here);
    /// * the recipient is in the game, is this leader or a mutual ally, and
    ///   is not a human (`leader_flags & 4`) unless the caller is that
    ///   AI-driven human;
    /// * the good is not already in the list — a linear scan, which is why
    ///   the list stays short and ordered;
    /// * `type_avail(good, strict)` is non-zero, and the good is neither
    ///   `FISH` nor `WHALES`. Those two are the **fisherman's** rares
    ///   (`LeaderData::calc_rare` pays them to a boat, not to a merchant),
    ///   so a coast full of fish never buys a merchant.
    pub(crate) fn new_rare(&mut self, who: Player, gi: usize) {
        if self.nation[who as usize].human {
            return;
        }
        let Some(ty) = self.world.goods().get(gi).map(|g| g.ty) else {
            return;
        };
        if self.tech_tree.is(ty, economy::FISH, false)
            || self.tech_tree.is(ty, economy::WHALES, false)
        {
            return;
        }
        for w in 0..self.players.len() {
            if self.defeated[w] || self.nation[w].human {
                continue;
            }
            if w != who as usize && !(self.allied[who as usize][w] && self.allied[w][who as usize])
            {
                continue;
            }
            if self.ai[w].new_rares.contains(&gi) {
                continue;
            }
            if self.type_avail(w as Player, ty) == crate::tech::NOT_AVAILABLE {
                continue;
            }
            self.ai[w].new_rares.push(gi);
        }
    }

    /// `Leader::calc_unit_stats@006cf970`, its line of sight and speed: every
    /// one of the player's live units runs `update_los` (vtable `+0x160`,
    /// before the listing's `+0xe8` test) and takes its cached speed again.
    pub(crate) fn calc_unit_stats(&mut self, who: Player) {
        for u in 0..self.units.len() {
            let unit = &self.units[u];
            if unit.owner != who || !unit.alive() {
                continue;
            }
            let Some(ty) = unit.ty else { continue };
            self.update_los(u);
            self.units[u].movement.speed = self.type_speed(who, ty);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::{RATE_SCALE, Resource};
    use crate::world::{
        Cell, CellData, Good, Pos, TILES_PER_CELL, Terrain, UNITS_PER_TILE, World, tile,
    };
    use crate::{Tuning, tech};

    /// The fifty `resourcerules.xml` goods as this install writes the two
    /// this mechanic reads: Fish pays Food 10 and Wealth 10, Whales pays
    /// Food 10 and Metal 10.
    fn good_table() -> Vec<economy::GoodType> {
        let mut v = vec![economy::GoodType::default(); 50];
        v[economy::FISH].bonus = [(Some(Resource::Food), 10), (Some(Resource::Wealth), 10)];
        v[economy::WHALES].bonus = [(Some(Resource::Food), 10), (Some(Resource::Metal), 10)];
        v
    }

    /// A 40 × 40 all-ocean world with a human and an AI, the fifty goods in
    /// the tree, and one deployed AI Fisherman at the centre of cell
    /// `(20, 20)` — `calc_gather.rs`'s fixture with the good table and a
    /// naval `obj_masks` on the boat.
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
        s.good_types = good_table();
        {
            let t = &mut s.tech_tree;
            for i in 0..50 {
                t.add(tech::TypeDef::good(&format!("good{i}")));
            }
            while t.types.len() < crate::fish::FISHERMEN {
                t.add(tech::TypeDef::plain("filler", -1));
            }
            let boat = t.add(tech::TypeDef::unit(
                "Fishing Boat",
                tech::UnitTraits::default(),
            ));
            assert_eq!(boat, crate::fish::FISHERMEN);
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
        s.unit_types[t].combat.obj_masks |= crate::combat::mask::NAVAL;
        s.unit_types[t].combat.domain = crate::attrition::Domain::Sea;
        let at = Pos::new(20 * 768 + 384, 20 * 768 + 384);
        let mut u = crate::Unit::new(1, 14, at, 20);
        u.ty = Some(t);
        let i = s.add_unit(u);
        // A packing type is born packed, and a packed boat is step 6's
        // very first refusal — this one has deployed.
        s.units[i].combat.packed = false;
        (s, i)
    }

    /// Ring 1 of the boat's own spiral, where a good the boat is "standing
    /// on" actually sits (`docs/ORDERS.md` §6.10).
    fn put_good_beside(s: &mut Sim, u: usize, ty: crate::tech::TypeId) {
        let t0 = s.units[u].pos.tile();
        let circle = crate::ai_place::circle();
        let t = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
        s.world.add_good(Good {
            pos: Pos::new(t.x * UNITS_PER_TILE + 96, t.y * UNITS_PER_TILE + 96),
            ty,
            alive: true,
        });
        let m = s.world.tile_mask(t);
        s.world.set_tile_mask(t, m | tile::AS_BUILDING);
    }

    /// Puts a good on the boat's own tile and lights the tile's `0x200`,
    /// which is what [`Sim::reveal_fog`] reads before it looks for one.
    fn good_under(s: &mut Sim, u: usize, ty: crate::tech::TypeId) -> usize {
        let t = s.units[u].pos.tile();
        s.world.add_good(Good {
            pos: Pos::new(t.x * UNITS_PER_TILE + 96, t.y * UNITS_PER_TILE + 96),
            ty,
            alive: true,
        });
        // The footprint, not the tile: the original's gate reads the
        // **odd** tile `(2fx + 1, 2fy + 1)` of a fog cell, so a mark on one
        // even tile is a mark no fog cell can see.
        for dy in 0..2 {
            for dx in 0..2 {
                let at = Pos::new(t.x + dx, t.y + dy);
                let m = s.world.tile_mask(at);
                s.world.set_tile_mask(at, m | tile::AS_BUILDING);
            }
        }
        s.world.goods().len() - 1
    }

    /// The fog cell whose `(2fx + 1, 2fy + 1)` tile is one [`good_under`]
    /// marked.
    fn fog_of_unit(s: &Sim, u: usize) -> (i32, i32) {
        let t = s.units[u].pos.tile();
        (t.x / 2, t.y / 2)
    }

    /// **`FISH` and `WHALES` are not merchant rares**, and
    /// `Leader::new_rare` is where they are dropped: they pay a fishing
    /// boat, so a coast full of them must not buy the AI a Merchant.
    /// Anything else the leader can build with is kept, once.
    #[test]
    fn a_fish_and_a_whale_are_never_a_seen_rare() {
        let (mut s, u) = sea_sim();
        let fish = good_under(&mut s, u, economy::FISH);
        let whale = good_under(&mut s, u, economy::WHALES);
        let citrus = good_under(&mut s, u, 26);
        s.new_rare(1, fish);
        s.new_rare(1, whale);
        assert!(s.ai[1].new_rares.is_empty(), "the fisherman's two rares");
        s.new_rare(1, citrus);
        s.new_rare(1, citrus);
        assert_eq!(
            s.ai[1].new_rares,
            vec![citrus],
            "recorded once, and once only"
        );
    }

    /// **A plain human records nothing** — `leader_flags & 0xc == 4` is the
    /// caller's own gate and `& 4` the recipient's, so neither the human's
    /// own reveals nor an ally's on its behalf reach its list.
    #[test]
    fn a_human_s_reveal_records_nothing_on_either_side() {
        let (mut s, u) = sea_sim();
        let citrus = good_under(&mut s, u, 26);
        s.new_rare(0, citrus);
        assert!(s.ai[0].new_rares.is_empty(), "the human's own reveal");
        assert!(s.ai[1].new_rares.is_empty(), "and it told nobody");
        s.allied[0][1] = true;
        s.allied[1][0] = true;
        s.new_rare(1, citrus);
        assert_eq!(s.ai[1].new_rares, vec![citrus]);
        assert!(
            s.ai[0].new_rares.is_empty(),
            "an ally that is a human is skipped"
        );
    }

    /// **An oil patch is never seen**, because `Objects::init_good` never
    /// puts one in a cell's chain — which is why East Indies' unreplayed
    /// start fog costs that capture nothing: the one good under its AI at
    /// frame 0 is oil.
    #[test]
    fn an_oil_patch_is_not_in_the_chain_to_be_seen() {
        let (mut s, u) = sea_sim();
        assert_eq!(good_under(&mut s, u, crate::world::OIL), 0);
        let (fx, fy) = fog_of_unit(&s, u);
        assert_eq!(Cell::new(fx >> 1, fy >> 1), s.units[u].pos.cell());
        s.reveal_fog(fx, fy, 1);
        assert!(s.ai[1].new_rares.is_empty(), "an oil patch is not a rare");
    }

    /// **The tile mark is the gate.** `reveal_fog` looks for a good only
    /// where `Objects::init_good` marked the ground `0x200`; with the mark
    /// off, the cell's own good is never reached.
    #[test]
    fn reveal_fog_looks_only_where_the_ground_is_marked() {
        let (mut s, u) = sea_sim();
        assert_eq!(good_under(&mut s, u, 26), 0);
        let (fx, fy) = fog_of_unit(&s, u);
        s.reveal_fog(fx, fy, 1);
        assert_eq!(s.ai[1].new_rares, vec![0]);

        let (mut s, u) = sea_sim();
        assert_eq!(good_under(&mut s, u, 26), 0);
        let mark = Pos::new(2 * fx + 1, 2 * fy + 1);
        let was = s.world.tile_mask(mark);
        s.world.set_tile_mask(mark, was & !tile::AS_BUILDING);
        s.reveal_fog(fx, fy, 1);
        assert!(s.ai[1].new_rares.is_empty(), "no mark, no look");
    }

    /// **The two numbers run59's census measured.** A level-0 fisherman
    /// standing on a fish in nobody's territory pays exactly `10 × 16` food
    /// and `10 × 16` wealth — `FISHERMEN_BONUS` entry 0 is 0% and
    /// `MERCHANTS_BONUS` entry 0 is 100%, so neither percentage moves
    /// anything. The AI's income was 1440 against 1600 and its wealth 0
    /// against 160 for exactly this reason.
    #[test]
    fn a_fish_pays_ten_of_each_of_its_two_goods() {
        let t = Tuning::RON;
        let g = good_table();
        let out = economy::calc_rare(&t, economy::FISH, &g[economy::FISH], 0, 0, false, false);
        assert_eq!(out[Resource::Food.index()], 10 * RATE_SCALE);
        assert_eq!(out[Resource::Wealth.index()], 10 * RATE_SCALE);
        assert_eq!(out[Resource::Timber.index()], 0);
    }

    /// **The fishermen bonus is added, and only to food.** At level 2 it is
    /// 100%, so the food half doubles and the wealth half stands.
    #[test]
    fn the_fishermen_bonus_reaches_the_food_half_alone() {
        let t = Tuning::RON;
        let g = good_table();
        let out = economy::calc_rare(&t, economy::FISH, &g[economy::FISH], 2, 0, false, false);
        assert_eq!(t.fishermen_bonus[2], 100);
        assert_eq!(out[Resource::Food.index()], 20 * RATE_SCALE);
        assert_eq!(out[Resource::Wealth.index()], 10 * RATE_SCALE);
    }

    /// **The merchants bonus replaces the 100, and on a water good it
    /// reaches everything but food.** At level 1 it is 120%.
    #[test]
    fn the_merchants_bonus_reaches_a_fish_s_other_half() {
        let t = Tuning::RON;
        let g = good_table();
        let out = economy::calc_rare(&t, economy::FISH, &g[economy::FISH], 0, 1, false, false);
        assert_eq!(t.merchants_bonus[1], 120);
        assert_eq!(out[Resource::Food.index()], 10 * RATE_SCALE);
        assert_eq!(out[Resource::Wealth.index()], 10 * RATE_SCALE * 120 / 100);
    }

    /// In **friendly ground** the same bonus reaches the food half too —
    /// the `local_20` flag, which is the difference between "a merchant in
    /// your own borders" and one abroad.
    #[test]
    fn friendly_ground_lets_the_merchants_bonus_reach_food() {
        let t = Tuning::RON;
        let g = good_table();
        let out = economy::calc_rare(&t, economy::FISH, &g[economy::FISH], 0, 1, true, false);
        assert_eq!(out[Resource::Food.index()], 10 * RATE_SCALE * 120 / 100);
        assert_eq!(out[Resource::Wealth.index()], 10 * RATE_SCALE * 120 / 100);
    }

    /// A good that is **not** one of the two water ones takes no fishermen
    /// bonus at all, however many upgrades the player holds.
    #[test]
    fn a_land_rare_never_takes_the_fishermen_bonus() {
        let t = Tuning::RON;
        let mut g = good_table();
        g[20].bonus = [(Some(Resource::Metal), 10), (None, 0)];
        let out = economy::calc_rare(&t, 20, &g[20], 3, 0, false, false);
        assert_eq!(out[Resource::Metal.index()], 10 * RATE_SCALE);
    }

    /// **The Nubians take half again, on a land rare in friendly ground**
    /// (`NUBIAN_RARE`, `LeaderData::calc_rare`'s `else if` arm): Dye's
    /// knowledge 160 is 240 (run492, `docs/GOLDEN.md` §54). Not abroad,
    /// and not on the two water goods, whose arm is the fishermen's.
    #[test]
    fn a_nubian_merchant_s_land_rare_pays_half_again_at_home() {
        let t = Tuning::RON;
        let mut g = good_table();
        g[10].bonus = [
            (Some(Resource::Wealth), 10),
            (Some(Resource::Knowledge), 10),
        ];
        let home = economy::calc_rare(&t, 10, &g[10], 0, 0, true, true);
        assert_eq!(home[Resource::Knowledge.index()], 240);
        assert_eq!(home[Resource::Wealth.index()], 240);
        let abroad = economy::calc_rare(&t, 10, &g[10], 0, 0, false, true);
        assert_eq!(abroad[Resource::Knowledge.index()], 160, "abroad");
        let other = economy::calc_rare(&t, 10, &g[10], 0, 0, true, false);
        assert_eq!(other[Resource::Knowledge.index()], 160, "not Nubian");
        let fish = economy::calc_rare(&t, economy::FISH, &g[economy::FISH], 0, 0, true, true);
        assert_eq!(fish[Resource::Wealth.index()], 10 * RATE_SCALE, "a fish");
    }

    /// **A Nubian Merchant is born with half again** (`Unit::update_hits`'
    /// `NUBIAN_HIT_POINTS` arm): run492's `0/6`, 135 on a type of 90
    /// (`docs/GOLDEN.md` §54). Another type of the same leader, and the
    /// same Merchant of another nation, take the type's.
    #[test]
    fn a_nubian_merchant_is_born_with_half_again() {
        let (mut s, _) = sea_sim();
        let merchant = s.add_unit_type(crate::UnitType {
            hits: 90,
            type_index: 0x3d,
            ..crate::UnitType::default()
        });
        let other = s.add_unit_type(crate::UnitType {
            hits: 90,
            type_index: 0x40,
            ..crate::UnitType::default()
        });
        for p in &mut s.tech {
            p.has_city = true;
        }
        s.tech[0].power = Some(crate::ai::tribe::NUBIANS);
        s.tech[1].power = Some(crate::ai::tribe::NUBIANS + 1);
        assert_eq!(s.unit_hits(0, merchant), 135);
        assert_eq!(s.unit_hits(0, other), 90, "not a trader");
        assert_eq!(s.unit_hits(1, merchant), 90, "not Nubian");
    }

    /// **The walk, end to end.** One idle Fisherman on a fish: the rate is
    /// the good's, the mask carries the fish's bit, and nothing else does.
    #[test]
    fn an_idle_fisherman_on_a_fish_pays_and_claims_it() {
        let (mut s, u) = sea_sim();
        put_good_beside(&mut s, u, economy::FISH);
        let (rate, owned) = s.gather_rares(1);
        assert_eq!(rate[Resource::Food.index()], 10 * RATE_SCALE);
        assert_eq!(rate[Resource::Wealth.index()], 10 * RATE_SCALE);
        assert_eq!(owned, 1 << (economy::FISH - economy::BASE_RARE));
        assert_eq!(s.units[u].rare, i32::try_from(economy::FISH).unwrap());
    }

    /// **A packed boat is not in the walk at all** — `calc_gather`'s very
    /// first test, and the only caller that has it. It is why the third
    /// Fisherman of run63, still packed and still sailing, claims nothing
    /// while the two deployed ones do.
    #[test]
    fn a_packed_fisherman_claims_nothing() {
        let (mut s, u) = sea_sim();
        put_good_beside(&mut s, u, economy::FISH);
        s.units[u].combat.packed = true;
        let (rate, owned) = s.gather_rares(1);
        assert_eq!(rate, [0; RESOURCES]);
        assert_eq!(owned, 0);
        assert_eq!(s.units[u].rare, -1, "and the field is cleared on the way");
    }

    /// **A boat with an order earns nothing.** `order_type() == NONE` is
    /// step 6's whole membership test, so a fisherman between two
    /// `think_fish` searches pays and one walking does not.
    #[test]
    fn a_fisherman_under_orders_is_not_walked() {
        let (mut s, u) = sea_sim();
        put_good_beside(&mut s, u, economy::FISH);
        let dest = Pos::new(30 * 768, 30 * 768);
        s.add_move_order(
            u,
            dest,
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::New,
            false,
        );
        let (rate, owned) = s.gather_rares(1);
        assert_eq!(rate, [0; RESOURCES]);
        assert_eq!(owned, 0);
    }

    /// **The crowd divides the payout, and it divides per unit.** Three
    /// boats of the same lineage close enough to share one deposit each pay
    /// `160 / 3` — and the truncation is each unit's own, so the deposit
    /// pays **159** rather than 160. Without the division it would pay 480,
    /// which is why a second fisherman on the same fish is worth nothing.
    #[test]
    fn a_shared_deposit_is_divided_per_boat_and_truncated() {
        let (mut s, u) = sea_sim();
        put_good_beside(&mut s, u, economy::FISH);
        let ty = s.units[u].ty;
        let at = s.units[u].pos;
        for o in 15..17 {
            let mut other = crate::Unit::new(1, o, at, 20);
            other.ty = ty;
            let i = s.add_unit(other);
            s.units[i].combat.packed = false;
        }
        let (rate, _) = s.gather_rares(1);
        assert_eq!(rate[Resource::Food.index()], 10 * RATE_SCALE / 3 * 3);
        assert_eq!(rate[Resource::Food.index()], 159);
        assert!(s.units[u].gather_here, "and each knows it is sharing");
    }

    /// **The whale, and the whole of what owning one does to a unit.**
    /// A naval type's cached speed takes `WHALES_SHIPS_MOVE`; a land type's
    /// does not; and neither moves until the mask actually carries the bit.
    #[test]
    fn a_whale_speeds_the_ships_and_nothing_else() {
        let (mut s, u) = sea_sim();
        let boat_ty = s.units[u].ty.unwrap();
        let foot_ty = s.add_unit_type(crate::UnitType {
            hits: 20,
            moves: 25,
            ..crate::UnitType::default()
        });
        assert_eq!(s.type_speed(1, boat_ty), 40, "no whale, no bonus");
        s.ledgers[1].rare = 1 << (economy::WHALES - economy::BASE_RARE);
        assert!(s.has_rare(1, economy::WHALES));
        assert_eq!(s.type_speed(1, boat_ty), 40 * 120 / 100);
        assert_eq!(s.type_speed(1, foot_ty), 25, "a foot type is not naval");
        assert_eq!(s.type_speed(0, boat_ty), 40, "and it is the owner's rare");
    }

    /// **The gunpowder foot line, by identity and truncating** (item 1109):
    /// Infantry's `MOVES` 32 is 34, as run404 prints from 621; each sibling
    /// takes its own scale and no other; a whale does not reach a foot type.
    #[test]
    fn the_gunpowder_foot_line_takes_its_own_scale() {
        let (mut s, _) = sea_sim();
        let foot = |s: &mut Sim, type_index: i32, moves: i32| {
            s.add_unit_type(crate::UnitType {
                hits: 20,
                moves,
                type_index,
                ..crate::UnitType::default()
            })
        };
        let inf = foot(&mut s, INFANTRY, 32);
        let mech = foot(&mut s, MECH_INFANTRY, 32);
        let rifle = foot(&mut s, RIFLEMAN, 32);
        let arq = foot(&mut s, ARQUEBUSIERS, 30);
        let other = foot(&mut s, INFANTRY + 1, 32);
        assert_eq!(s.type_speed(1, inf), 34, "run404's 621: 32 x 34 / 32");
        assert_eq!(s.type_speed(1, mech), 36);
        assert_eq!(s.type_speed(1, rifle), 37, "32 x 32 / 27 truncates");
        assert_eq!(s.type_speed(1, arq), 37, "30 x 30 / 24 truncates");
        assert_eq!(s.type_speed(1, other), 32, "identity, not a lineage");
        s.ledgers[1].rare = 1 << (economy::WHALES - economy::BASE_RARE);
        assert_eq!(s.type_speed(1, inf), 34, "a whale is the navy's");
    }

    /// **The gunpowder foot line is a lineage, not an identity** (twenty-fourth
    /// pass, group 22): `Unit::update_speed`'s four tests ask the type's vslot
    /// `0x60`, which is `ObjectTypeData::is`, so a national replacement that
    /// the tree puts under a line — by `FROM`, by `GRAFT`, or by the
    /// `is_list` — takes the line's scale, the first of the four that
    /// answers wins, and a type outside every line takes none.
    #[test]
    fn a_national_replacement_takes_its_line_s_gunpowder_scale() {
        let (mut s, _) = sea_sim();
        let at = |s: &mut Sim, idx: usize, def: tech::TypeDef| {
            while s.tech_tree.types.len() <= idx {
                s.tech_tree.add(tech::TypeDef::plain("filler", -1));
            }
            s.tech_tree.types[idx] = def;
        };
        let traits = tech::UnitTraits::default;
        // The four lines themselves, then one replacement under each by a
        // different arm of `is`, then one under two lines at once.
        for (idx, name) in [
            (ARQUEBUSIERS, "Arquebusier"),
            (RIFLEMAN, "Rifleman"),
            (INFANTRY, "Infantry"),
            (MECH_INFANTRY, "Mechanized Infantry"),
        ] {
            at(&mut s, idx as usize, tech::TypeDef::unit(name, traits()));
        }
        let by_from = 0x70;
        let by_graft = 0x71;
        let by_list = 0x72;
        let both = 0x73;
        let stranger = 0x74;
        at(
            &mut s,
            by_from,
            tech::TypeDef::unit("Redcoat", traits()).from(INFANTRY as usize),
        );
        let mut graft = tech::TypeDef::unit("Sepoy", traits());
        graft.graft = Some(ARQUEBUSIERS as usize);
        at(&mut s, by_graft, graft);
        let mut list = tech::TypeDef::unit("Jaeger", traits());
        list.is_list.push(RIFLEMAN as usize);
        at(&mut s, by_list, list);
        let mut two = tech::TypeDef::unit("Grenzer", traits());
        two.is_list = vec![ARQUEBUSIERS as usize, RIFLEMAN as usize];
        at(&mut s, both, two);
        at(&mut s, stranger, tech::TypeDef::unit("Cow", traits()));
        let line = |s: &mut Sim, idx: usize, moves: i32| {
            s.add_unit_type(crate::UnitType {
                hits: 20,
                moves,
                type_index: idx as i32,
                ..crate::UnitType::default()
            })
        };
        let a = line(&mut s, by_from, 32);
        let b = line(&mut s, by_graft, 30);
        let c = line(&mut s, by_list, 32);
        let d = line(&mut s, both, 32);
        let e = line(&mut s, stranger, 32);
        assert_eq!(s.type_speed(1, a), 34, "FROM Infantry: 32 x 34 / 32");
        assert_eq!(s.type_speed(1, b), 37, "GRAFT Arquebusiers: 30 x 30 / 24");
        assert_eq!(s.type_speed(1, c), 37, "is_list Rifleman: 32 x 32 / 27");
        assert_eq!(
            s.type_speed(1, d),
            37,
            "Rifleman is asked before Arquebusiers"
        );
        assert_eq!(s.type_speed(1, e), 32, "no line, no scale");
    }

    /// **Peacocks, the one rare the population cap reads** (item 1147):
    /// `calc_pop_cap` tests the mask on every call, so a recompute for any
    /// cause sees it, and it is the owner's alone.
    #[test]
    fn peacocks_raise_the_owner_s_pop_cap_by_a_tenth() {
        let (mut s, _) = sea_sim();
        for m in &mut s.muster {
            m.military_level = 1;
        }
        s.recompute_pop_caps();
        assert_eq!((s.muster[0].cap, s.muster[1].cap), (50, 50));
        s.ledgers[1].rare = 1 << (economy::PEACOCKS - economy::BASE_RARE);
        s.recompute_pop_caps();
        assert_eq!(
            (s.muster[0].cap, s.muster[1].cap),
            (50, 55),
            "Great Sahara's 55: 50 x 110 / 100"
        );
        s.ledgers[1].rare = 1 << (economy::WHALES - economy::BASE_RARE);
        s.recompute_pop_caps();
        assert_eq!(s.muster[1].cap, 50, "no other rare moves it");
    }

    /// **And the mask's own change recomputes it** — `calc_gather`'s
    /// `operator!=` arm calls `calc_pop_cap` after the assignment
    /// (`006ceee0:353`), so the cap follows the Peacocks bit on the frame
    /// the bit moves, with nothing else changing.
    #[test]
    fn the_rare_mask_s_change_recomputes_the_pop_cap() {
        let (mut s, _) = sea_sim();
        for m in &mut s.muster {
            m.military_level = 1;
        }
        s.recompute_pop_caps();
        let bit = 1u64 << (economy::PEACOCKS - economy::BASE_RARE);
        s.holdings[1].rare_owned = bit;
        s.ledgers[1].dirty = false;
        s.ledgers[1].gather_stamp = s.frame;
        assert!(!s.holdings_due(1, s.frame), "the walk does not rebuild it");
        s.tick();
        assert_eq!(s.ledgers[1].rare, bit);
        assert_eq!(s.muster[1].cap, 55, "the bit arrived and the cap moved");
        s.holdings[1].rare_owned = 0;
        s.ledgers[1].dirty = false;
        s.ledgers[1].gather_stamp = s.frame;
        s.tick();
        assert_eq!(s.muster[1].cap, 50, "and it goes when the bit goes");
    }

    /// The truncation is the original's: `38 × 120 / 100` is **45**, which
    /// is what run63's dump prints for all three of the AI's Fishermen from
    /// frame 5552 — and 25 becomes 30 for its Transport Barge.
    #[test]
    fn the_whales_bonus_truncates_where_the_dump_does() {
        let (mut s, u) = sea_sim();
        let boat_ty = s.units[u].ty.unwrap();
        s.unit_types[boat_ty].moves = 38;
        s.ledgers[1].rare = 1 << (economy::WHALES - economy::BASE_RARE);
        assert_eq!(s.type_speed(1, boat_ty), 45);
        s.unit_types[boat_ty].moves = 25;
        assert_eq!(s.type_speed(1, boat_ty), 30);
    }

    /// **`calc_unit_stats` reaches every live unit of the player**, which
    /// is how a boat already on the water gets the new speed on the frame
    /// the whale is claimed rather than the next time it is built.
    #[test]
    fn calc_unit_stats_re_caches_every_unit_s_speed() {
        let (mut s, u) = sea_sim();
        assert_eq!(s.units[u].movement.speed, 0, "the fixture never set one");
        s.ledgers[1].rare = 1 << (economy::WHALES - economy::BASE_RARE);
        s.calc_unit_stats(1);
        assert_eq!(s.units[u].movement.speed, 40 * 120 / 100);
    }

    #[test]
    fn supply_upgrade_speed_counts_sparse_steps_and_refreshes_existing_units() {
        use crate::ai_load::uflags2;
        let mut s = crate::Sim::new(Tuning::RON, World::new(20, 20), 2);
        let mut tree = tech::TechTree::new();
        let steps = [
            tree.add(tech::TypeDef::plain("step one", 0)),
            tree.add(tech::TypeDef::plain("step two", 0)),
            tree.add(tech::TypeDef::plain("step three", 0)),
        ];
        tree.roles.supply_upgrade_preq = steps.map(Some);
        s.set_tech_tree(tree);
        let wagon = s.add_unit_type(crate::UnitType {
            moves: 25,
            hits: 90,
            cols: crate::ai_load::UnitCols {
                unit_flags2: uflags2::SUPPLY_OR_HERO,
                ..crate::ai_load::UnitCols::default()
            },
            ..crate::UnitType::default()
        });
        let hero = s.add_unit_type(crate::UnitType {
            moves: 25,
            hits: 90,
            cols: crate::ai_load::UnitCols {
                unit_flags2: uflags2::GENERAL,
                ..crate::ai_load::UnitCols::default()
            },
            ..crate::UnitType::default()
        });
        let mut unit = crate::Unit::new(0, 0, Pos::new(792, 792), 90);
        unit.ty = Some(wagon);
        let u = s.add_unit(unit);
        s.calc_unit_stats(0);
        assert_eq!(s.units[u].movement.speed, 25);
        // Grant the last step first: this is a count, not a prefix.
        for (step, expected) in [(steps[2], 31), (steps[0], 37), (steps[1], 43)] {
            let old = s.units[u].movement.speed;
            s.gain_tech(0, step);
            assert_eq!(s.type_speed(0, wagon), expected);
            assert_eq!(s.type_speed(1, wagon), 25, "other owner");
            assert_eq!(s.type_speed(0, hero), 25, "general-only bit");
            assert_eq!(
                s.units[u].movement.speed, old,
                "cached until the leader pass"
            );
            s.tick();
            assert_eq!(s.units[u].movement.speed, expected);
        }
    }

    /// **A fisherman going idle drops the economy's period to eight
    /// frames.** `Unit::check_idle`'s latch is what puts a boat that has
    /// just arrived into step 6's walk within a handful of frames instead
    /// of at the next 512-frame refresh — and without it run63's whale
    /// would have been claimed 130 frames late.
    #[test]
    fn a_fisherman_going_idle_marks_the_economy_dirty() {
        let (mut s, u) = sea_sim();
        s.ledgers[1].dirty = false;
        s.units[u].idle_latch = false;
        s.tick();
        assert!(
            s.units[u].idle_latch,
            "the latch is taken on the idle frame"
        );
        assert!(s.ledgers[1].dirty, "and it is what marks the economy");
        // And it is a **latch**: standing there does not mark it again.
        s.ledgers[1].dirty = false;
        s.tick();
        assert!(!s.ledgers[1].dirty, "one transition, one mark");
    }

    /// **A unit's cached speed comes from [`Sim::type_speed`], and the
    /// rule is a guard rather than a sentence.**
    ///
    /// `Unit::update_speed`'s Whales arm has been landed since run63 and
    /// was applied at every spawn site but one: `cast_transport` set the
    /// boat's speed from the type's raw `MOVES`, and that is the only
    /// place in this crate where a **naval** unit is born. East Indies'
    /// long word sat at 7448 for three sessions on that one line, and it
    /// took a 272 MB capture to name it (run86, `docs/ORACLE.md`). Every
    /// new spawn site is another chance to forget, and the failure is
    /// silent everywhere the owner holds no rare — so the source itself
    /// is read here. Made to fail on the three gaia sites it found
    /// (`farms.rs`, `gaia.rs`, the dock's gull), each of which was
    /// behaviour-identical only because owner 9 holds nothing.
    #[test]
    fn a_unit_s_speed_is_never_set_from_the_raw_moves() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
        let mut bad: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(dir).expect("the crate's own src/") {
            let path = entry.expect("a readable entry").path();
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            // `rares.rs` is where `type_speed` reads it, which is the
            // point of the rule and not a breach of it.
            if name == "rares.rs" {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("a readable source file");
            for (i, line) in text.lines().enumerate() {
                let code = line.split("//").next().unwrap_or("");
                if code.contains("movement.speed =") && code.contains(".moves") {
                    bad.push(format!("{name}:{}: {}", i + 1, line.trim()));
                }
            }
        }
        assert!(
            bad.is_empty(),
            "a spawn site sets `movement.speed` from the type's raw `MOVES`;              use `Sim::type_speed`, which is `Unit::update_speed`: {bad:#?}"
        );
    }
}
