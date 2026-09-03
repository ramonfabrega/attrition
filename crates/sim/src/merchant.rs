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
            if !g.alive || !self.good_ever_seen(g.pos, who) {
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
    ///    is on its way. "Ordered" is literal: the candidate must hold a
    ///    **move-family** order (`crate::scout`'s `MOVE_FAMILY`).
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
        // `SEARCH_FRIENDLY` with `FILTER_NOT_ME(o, who)` and
        // `FILTER_TYPE(type)` — mine and my allies', never me, of exactly
        // my `TypeData.type`. The leader loop stops at eight, so gaia is
        // not in the search space.
        let sibling = |s: &Self, i: usize| {
            let x = &s.units[i];
            i != u
                && x.alive()
                && x.on_map
                && !x.is_gaia()
                && s.is_ally(who, x.owner)
                && x.type_index == ty
        };
        if (0..self.units.len()).any(|i| sibling(self, i) && near(self, i, FRIENDLY_RANGE)) {
            return true;
        }
        if (0..self.units.len()).any(|i| {
            sibling(self, i)
                && crate::scout::MOVE_FAMILY.contains(&self.order_type(i))
                && near(self, i, FRIENDLY_RANGE)
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
    fn good_ever_seen(&self, at: Pos, who: Player) -> bool {
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
    /// (`docs/MERCHANT.md` §4) — asks with ring 4 and no capture reaches
    /// it.
    pub(crate) fn unpack_merchant(&mut self, u: usize, ring: usize) -> bool {
        let Some(t) = self.find_merchant_spot(u, ring) else {
            return false;
        };
        // `add_cast_order(-1, -1, …, 0x28c, QUEUE_NEW, 0)` — the unpack,
        // rewritten to the merchant family's `0x290` — and then the walk
        // to the spot, **appended behind it**: `LinkListBase::add` is the
        // back of the queue (`docs/ORDERS.md` §1.5).
        self.add_cast_order_at(u, crate::orders::spell::UNPACK, QueuePos::New);
        let to = Pos::new(t.x * UNITS_PER_TILE, t.y * UNITS_PER_TILE);
        self.add_move_order(u, to, MoveKind::MoveTo, QueuePos::Last, false);
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
    /// SEAM: `detect_unit_collision(x, y, 1, 1, 0, 0, 0)`, the third test,
    /// is not asked. This crate's is `&mut` — it writes the
    /// `collide_o`/`collide_who` bookkeeping every path out of the move
    /// step depends on — and a read-only form is its own item; nothing on
    /// disk reaches a ring walk at all.
    fn find_merchant_spot(&mut self, u: usize, ring: usize) -> Option<Pos> {
        if !self.calc_gather(u) {
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
    fn good_merchant_spot(&mut self, u: usize, t: Pos) -> bool {
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
        self.calc_gather_at(u, Pos::new(t.x * UNITS_PER_TILE, t.y * UNITS_PER_TILE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tuning;
    use crate::orders::{Body, index};
    use crate::world::{Cell, Good, Terrain, World};

    /// A 40 × 40 land world with a human (0) and a computer (1), and the
    /// computer's Merchant — `TypeIndex` `0x3d`, packing — standing at
    /// cell `(20, 20)`. Two regions: the west half is one, the east half
    /// (the merchant's) another, which is what the `+100` needs.
    fn merchant_sim() -> (Sim, usize) {
        let mut world = World::new(40, 40);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(19, 39));
        world.fill_region(Terrain::Land, Cell::new(20, 0), Cell::new(39, 39));
        let mut s = Sim::new(Tuning::RON, world, 2);
        s.nation[0].human = true;
        s.nation[1].human = false;
        {
            let t = &mut s.tech_tree;
            while t.types.len() < 0x3d {
                t.add(crate::tech::TypeDef::plain("filler", -1));
            }
            let m = t.add(crate::tech::TypeDef::unit(
                "Merchant",
                crate::tech::UnitTraits::default(),
            ));
            assert_eq!(m, 0x3d, "the fixture's ids are TypeIndex's");
        }
        s.tech = (0..2)
            .map(|_| crate::tech::PlayerTech::new(&s.tech_tree))
            .collect();
        let t = s.add_unit_type(crate::UnitType {
            hits: 20,
            moves: 40,
            tree: Some(0x3d),
            type_index: 0x3d,
            ..crate::UnitType::default()
        });
        s.unit_types[t].combat.packs = true;
        let at = Pos::new(20 * UNITS_PER_CELL + 384, 20 * UNITS_PER_CELL + 384);
        let mut u = crate::Unit::new(1, 19, at, 20);
        u.ty = Some(t);
        u.type_index = 0x3d;
        let i = s.add_unit(u);
        (s, i)
    }

    /// Puts a good at a cell's centre and tells the computer it has seen
    /// it. Answers the goods-list index the score reads.
    fn a_good(s: &mut Sim, c: Cell) -> usize {
        let gi = s.world.add_good(Good {
            pos: Pos::new(
                c.x * UNITS_PER_CELL + UNITS_PER_CELL / 2,
                c.y * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            ),
            ty: 26,
            alive: true,
        });
        s.ai[1].new_rares.push(gi);
        gi
    }

    /// Re-arms the merchant for another think: packed again, and nothing
    /// on its order list.
    fn rearm(s: &mut Sim, u: usize) {
        s.units[u].combat.packed = true;
        s.close_orders(u);
    }

    /// Which good the `MOVE_TO` the think just issued points at. The order
    /// carries the 48-snapped point, so the comparison is against the
    /// snap of each good in turn.
    fn walking_to(s: &Sim, u: usize) -> Option<usize> {
        let Some(Body::Move(m)) = s.current_order(u).map(|o| o.body) else {
            return None;
        };
        (0..s.world.goods().len())
            .find(|&i| crate::orders::snapped(s.world.goods()[i].pos) == m.dest)
    }

    /// §2.1 — and it is a `1`, not a `0`: the deployed merchant ends the
    /// think where it stands.
    #[test]
    fn a_deployed_merchant_ends_the_think_and_searches_nothing() {
        let (mut s, u) = merchant_sim();
        assert!(s.units[u].combat.packed, "a merchant is born packed");
        let west = a_good(&mut s, Cell::new(5, 20));
        s.units[u].combat.packed = false;
        assert!(s.think_merchant(u), "a deployed merchant answers 1");
        assert!(s.units[u].orders.is_empty(), "and issues nothing");
        assert_eq!(s.ai[1].new_rares, vec![west], "and rotates nothing");
    }

    /// §2.2's two terms that East Indies cannot separate: the good in the
    /// merchant's **own** region wins from the back of the list, because
    /// `+100` beats the `−10` a position costs.
    #[test]
    fn the_own_region_bonus_outweighs_the_list_position() {
        let (mut s, u) = merchant_sim();
        let west = a_good(&mut s, Cell::new(5, 20));
        let east = a_good(&mut s, Cell::new(30, 20));
        assert_ne!(
            s.world.region_of(Cell::new(5, 20)),
            s.world.region_of(Cell::new(30, 20)),
            "the fixture's two halves are two regions"
        );
        assert!(s.think_merchant(u), "it found one");
        assert_eq!(
            s.ai[1].new_rares,
            vec![west, east],
            "200 against 190 + 100: the winner is taken out and appended"
        );
        let dest = match s.current_order(u).map(|o| o.body) {
            Some(Body::Move(m)) => m.dest,
            other => panic!("a MOVE_TO, not {other:?}"),
        };
        assert_eq!(
            dest,
            crate::orders::snapped(s.world.goods()[east].pos),
            "the east good, 48-snapped"
        );
    }

    /// The same pair with the bonus taken away: position alone decides,
    /// and the **head** of the list wins.
    #[test]
    fn with_no_bonus_the_head_of_the_list_wins() {
        let (mut s, u) = merchant_sim();
        let a = a_good(&mut s, Cell::new(5, 20));
        let b = a_good(&mut s, Cell::new(6, 20));
        assert!(s.think_merchant(u));
        assert_eq!(
            s.ai[1].new_rares,
            vec![b, a],
            "200 beats 190, and neither is in the merchant's region"
        );
    }

    /// And where the two terms **do** separate: eleven goods deep, the
    /// `−10` a position costs has eaten the `+100` an own region pays, and
    /// the head of the list wins anyway. This is the assertion the step
    /// has of its own — with the `−10` dropped, the twelfth good scores
    /// 300 and takes it.
    #[test]
    fn eleven_positions_outweigh_the_own_region_bonus() {
        let (mut s, u) = merchant_sim();
        let head = a_good(&mut s, Cell::new(5, 20));
        for y in 0..10 {
            a_good(&mut s, Cell::new(6, 20 + y));
        }
        // Index 11, in the merchant's own region: 200 − 110 + 100 = 190.
        let mine = a_good(&mut s, Cell::new(30, 20));
        assert_eq!(s.ai[1].new_rares.len(), 12);
        assert!(s.think_merchant(u));
        assert_eq!(
            walking_to(&s, u),
            Some(head),
            "200 for the head against 190 for the good in its own region"
        );
        assert_eq!(
            *s.ai[1].new_rares.last().expect("twelve"),
            head,
            "and the winner is the one appended"
        );
        assert_ne!(mine, head, "the two are different goods");
    }

    /// §2.2's first search: a sibling of my exact type standing within a
    /// cell of the good's cell centre refuses the good outright.
    #[test]
    fn a_sibling_at_the_good_refuses_it() {
        let (mut s, u) = merchant_sim();
        let want = a_good(&mut s, Cell::new(30, 20));
        let other = a_good(&mut s, Cell::new(5, 20));
        let ty = s.units[u].ty;
        let mut v = crate::Unit::new(1, 20, s.world.goods()[want].pos, 20);
        v.ty = ty;
        v.type_index = 0x3d;
        let sib = s.add_unit(v);
        assert!(s.think_merchant(u), "it took the other one instead");
        assert_eq!(walking_to(&s, u), Some(other));
        s.units[sib].pos = Pos::new(0, 0);
        rearm(&mut s, u);
        assert!(s.think_merchant(u));
        assert_eq!(
            walking_to(&s, u),
            Some(want),
            "with the sibling gone the good is taken"
        );
    }

    /// The second search — and the whole of what "ordered" means: the same
    /// sibling, in range but only refusing while it holds a move-family
    /// order.
    #[test]
    fn an_ordered_sibling_refuses_it() {
        let (mut s, u) = merchant_sim();
        let want = a_good(&mut s, Cell::new(30, 20));
        let other = a_good(&mut s, Cell::new(5, 20));
        let ty = s.units[u].ty;
        let mut v = crate::Unit::new(1, 20, Pos::new(0, 0), 20);
        v.ty = ty;
        v.type_index = 0x3d;
        let sib = s.add_unit(v);
        assert!(s.think_merchant(u), "the sibling is nowhere near");
        assert_eq!(walking_to(&s, u), Some(want), "it took the good");

        // In range, with a move order on it.
        s.units[sib].pos = s.world.goods()[want].pos;
        s.add_move_order(sib, Pos::new(0, 0), MoveKind::MoveTo, QueuePos::New, false);
        assert_eq!(s.order_type(sib), index::MOVE_TO, "the move family");
        rearm(&mut s, u);
        assert!(s.think_merchant(u));
        assert_eq!(walking_to(&s, u), Some(other), "refused this time");
    }

    /// The third: an enemy that can shoot, four cells out.
    #[test]
    fn an_enemy_that_can_shoot_refuses_it_at_four_cells() {
        let (mut s, u) = merchant_sim();
        let want = a_good(&mut s, Cell::new(30, 20));
        let other = a_good(&mut s, Cell::new(5, 20));
        let soldier = s.add_unit_type(crate::UnitType {
            hits: 30,
            ..crate::UnitType::default()
        });
        s.unit_types[soldier].combat.attack = 5;
        // `SEARCH_ENEMY` is `LeaderData::is_enemy`, so a player merely not
        // allied is not searched at all.
        s.declare_war(0, 1);
        let mut e = crate::Unit::new(0, 1, s.world.goods()[want].pos, 30);
        e.ty = Some(soldier);
        let them = s.add_unit(e);
        assert!(s.think_merchant(u));
        assert_eq!(walking_to(&s, u), Some(other), "the good it can reach");
        s.units[them].pos = Pos::new(
            30 * UNITS_PER_CELL + UNITS_PER_CELL / 2 + ENEMY_RANGE + 1,
            30 * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        );
        rearm(&mut s, u);
        assert!(s.think_merchant(u));
        assert_eq!(
            walking_to(&s, u),
            Some(want),
            "one unit past `0xc00` and the good is acceptable again"
        );
    }

    /// A merchant with nothing to score answers 0 — which sends the
    /// caller on to `Unit::think`'s tail — and a dead good is not a
    /// destination.
    #[test]
    fn an_empty_list_answers_nothing() {
        let (mut s, u) = merchant_sim();
        assert!(!s.think_merchant(u), "no rares, no order");
        // A good the list names and `SubObjectData.flags & 1` denies.
        let gi = s.world.add_good(Good {
            pos: Pos::new(5 * UNITS_PER_CELL, 20 * UNITS_PER_CELL),
            ty: 26,
            alive: false,
        });
        s.ai[1].new_rares.push(gi);
        assert!(!s.think_merchant(u), "a dead good is not a destination");
        assert_eq!(s.ai[1].new_rares, vec![gi], "and nothing rotated");
    }

    /// §4's cadence, through `Unit::do_idle` — the first idle frame and
    /// then one frame in 128, phased by `o`.
    ///
    /// The probe is `UnitData::calc_gather`'s own first write:
    /// `unpack_merchant`'s spot search clears `rare` to `−1` on every
    /// call, and nothing else in an idle merchant's frame touches it.
    #[test]
    fn a_merchant_thinks_on_its_first_idle_frame_and_then_once_in_128() {
        let (mut s, u) = merchant_sim();
        let o = i64::from(s.units[u].index);
        let mut thought = Vec::new();
        for frame in 0..400 {
            s.units[u].rare = 7;
            s.do_idle(u, frame);
            if s.units[u].rare == -1 {
                thought.push(frame);
            }
        }
        let want: Vec<i64> = std::iter::once(0)
            .chain((1..400).filter(|f| (f + o) % 128 == 0))
            .collect();
        assert_eq!(
            thought, want,
            "`idle == 1 || ((o + frame) & 127) == 0`, and the mod-32 gate \
             above it never keeps a frame this one wants"
        );
    }

    /// §5 — the crew figure a **trained** unit is born with sits on its
    /// track offset, because `Unit::init` ends in
    /// `set_new_location(·, ·, 1, 1)`.
    #[test]
    fn a_trained_unit_s_tracked_crew_is_seated_at_birth() {
        let (mut s, _) = merchant_sim();
        let t = s.unit_types.len() - 1;
        s.unit_types[t].combat.crew_size = 1;
        // `piece_of` falls back to the dump-keyed table when the install's
        // is empty, so the fixture states guy 1's piece the way a dump
        // would, and gives that piece a track.
        s.art.pieces.insert((1, t, 1, 1), 4242);
        s.art.tracks.insert(4242, (-48, -192));
        let mut v = crate::Unit::new(1, 21, Pos::new(9000, 9000), 20);
        v.ty = Some(t);
        v.type_index = 0x3d;
        let i = s.add_unit(v);
        s.init_guys(i, Some(t));
        assert_eq!(s.units[i].guys.len(), 2, "crew_size + squad_size");
        assert_eq!(s.units[i].guys[1].gpiece, 4242);
        assert!(
            s.units[i].guys[1].follow.is_some(),
            "a tracked crew figure is seated at birth — without it \
             `Guy::do_turn` recurses into it and it spends a draw the \
             original does not"
        );
    }
}
