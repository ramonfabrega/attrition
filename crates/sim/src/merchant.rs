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
    /// The `ever_seen` bit `compute_reg_territory`'s goods scan set on this
    /// good for `who` ([`Sim::claim_cell_goods`]).
    pub(crate) fn good_seen_bit(&self, gi: usize, who: Player) -> bool {
        who < 8
            && self
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

    /// The craft table as `craftrules.xml` has it, for the one row these
    /// tests need: 55 records, and the Merchant's `0x290` is the
    /// twenty-eighth with a `JOB_TIME` of 148 (`docs/ORDERS.md` §6.9).
    /// Without it the deploy's job time is zero and the first frame casts
    /// outright, which hides everything that happens before the clock.
    fn merchant_job_time(s: &mut Sim) {
        s.spells = vec![crate::orders::SpellType::default(); 55];
        let row =
            usize::try_from(crate::orders::spell::UNPACK_MERCHANT - crate::orders::spell::FIRST)
                .unwrap();
        s.spells[row].job_time = 148;
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

    /// **`do_cast`'s re-seat, on the first frame of the unpack**
    /// (`docs/ORDERS.md` §6.9 step 2, `docs/ANIM.md` §4.10). The merchant
    /// is standing on its own unit-cell centre, so the point does not
    /// move; what the call is for is the crew, which is *put* on its
    /// offset instead of walking four more frames and paying an arrival
    /// stand the original never pays. Great Lakes 6151 → 6463.
    #[test]
    fn the_unpack_s_first_frame_seats_the_merchant_s_crew() {
        let (mut s, u) = merchant_sim();
        // **On its own unit-cell centre**, which is where a merchant that
        // has walked to a tile corner stands and is the whole point: the
        // re-seat's `set_new_location` then has `from == to`, and used to
        // return before it reached the crew. The fixture's world-cell
        // centre is a 48-grid *boundary*, not a centre.
        let at = crate::collide::ucell_centre(crate::collide::ucell(s.units[u].pos));
        assert!(s.set_new_location(u, at, true));
        assert_eq!(crate::collide::ucell_centre(crate::collide::ucell(at)), at);
        // The tile has to answer `good_merchant_spot`, whose last test is
        // `calc_gather` — a good on the tile, and the tile's own
        // `AS_BUILDING` bit.
        s.world.add_good(Good {
            pos: at,
            ty: 26,
            alive: true,
        });
        let t = at.tile();
        let m = s.world.tile_mask(t);
        s.world.set_tile_mask(t, m | tile::AS_BUILDING);

        // A crew figure with a track offset, seated and then dragged
        // behind — the state the Merchant `1/24` is in when it arrives.
        s.units[u].guys = vec![crate::anim::Guy::fresh(1), crate::anim::Guy::fresh(2)];
        s.art.tracks.insert(2, (-48, -192));
        s.seat_guys(u);
        let seated = s.units[u].guys[1].follow.expect("a tracked crew guy").des;
        s.units[u].guys[1].follow.as_mut().unwrap().body.pos =
            Pos::new(seated.x - 300, seated.y - 300);
        s.units[u].guys[1].anim = crate::anim::JOG;

        // `craftrules.xml` gives the Merchant's `0x290` a `JOB_TIME` of
        // 148 (`docs/ORDERS.md` §6.9), so the first frame seats and starts
        // the clock rather than deploying outright.
        merchant_job_time(&mut s);

        s.add_cast_order_at(u, crate::orders::spell::UNPACK, QueuePos::New);
        let Some(Body::Cast(c)) = s.current_order(u).map(|o| o.body) else {
            panic!("a cast order")
        };
        s.do_cast(u, c);

        let f = s.units[u].guys[1].follow.expect("still tracked");
        assert_eq!(
            f.body.pos, seated,
            "the crew figure is teleported, not left to walk"
        );
        assert_eq!(
            f.facing, s.units[u].movement.facing,
            "with its driver's facing, as run75's block 6145 has it"
        );
        assert_eq!(s.units[u].pos, at, "and the merchant itself has not moved");
        assert_eq!(
            s.units[u].guys[0].anim,
            crate::anim::UNPACK,
            "the animation the re-seat comes just ahead of"
        );
        assert_eq!(s.units[u].spell_time, 1, "and the clock has started");
    }

    /// **The spot's third test is the quick collision** (`Unit::find_merchant_spot
    /// @00603ab0`: `detect_unit_collision(x·192, y·192, 1, 1, 0, 0, 0)` at the
    /// tile's corner): a candidate whose corner cell holds another unit is
    /// refused, and the walk goes on to the next tile of the ring. Great
    /// Sahara's Merchant `1/38` stood on the rare `8/6` on 2205 and the
    /// original took `(−1, −1)` where this crate took `(0, 0)` (item 1611).
    #[test]
    fn a_unit_on_a_candidate_s_corner_refuses_the_spot() {
        let (mut s, u) = merchant_sim();
        // `BLOCK_RADIUS 1`, `coll_size 1` — the profile that blocks.
        let rec = s.units[u].ty.expect("a type");
        s.unit_types[rec].combat.block_radius = 48;
        s.unit_types[rec].combat.big_radius = 48;
        s.unit_types[rec].combat.uber_size = 1;
        // The merchant stands inside tile `t`, off its corner's unit cell.
        let t = Pos::new(82, 106);
        let corner = Pos::new(t.x * UNITS_PER_TILE, t.y * UNITS_PER_TILE);
        let at = crate::collide::ucell_centre(Pos::new(
            crate::collide::ucell(corner).x + 2,
            crate::collide::ucell(corner).y + 2,
        ));
        assert!(s.set_new_location(u, at, true));
        assert_eq!(at.tile(), t);
        // A rare on the tile, so `calc_gather` answers at every ring tile
        // the walk reaches first.
        for (dx, dy) in [(0, 0), (-1, -1)] {
            let p = Pos::new((t.x + dx) * UNITS_PER_TILE, (t.y + dy) * UNITS_PER_TILE);
            s.world.add_good(Good {
                pos: p,
                ty: 26,
                alive: true,
            });
            let q = p.tile();
            let m = s.world.tile_mask(q);
            s.world.set_tile_mask(q, m | tile::AS_BUILDING);
        }
        assert_eq!(s.find_merchant_spot(u, 1), Some(t), "nothing in the way");
        // Another unit on the candidate's corner cell.
        let ty = s.units[u].ty;
        let mut b = crate::Unit::new(
            0,
            20,
            crate::collide::ucell_centre(crate::collide::ucell(corner)),
            20,
        );
        b.ty = ty;
        b.type_index = 0x3d;
        s.add_unit(b);
        assert_ne!(
            s.find_merchant_spot(u, 1),
            Some(t),
            "the quick collision refuses the corner's tile"
        );
    }

    /// **`cast_unpack`'s merchant arm** (`docs/MERCHANT.md` §3.2): the
    /// deploy seats the trader on its tile's corner — `(−24, −24)` from the
    /// unit-cell centre it walked to — blocks the two-by-two under it and
    /// marks the economy; `Unit::close` gives the four tiles back. East
    /// Indies' `1/20` on 7663 is the dump this is the shape of (run88).
    #[test]
    fn a_deployed_merchant_sits_on_its_corner_and_blocks_its_square() {
        let (mut s, u) = merchant_sim();
        let at = crate::collide::ucell_centre(crate::collide::ucell(s.units[u].pos));
        assert!(s.set_new_location(u, at, true));
        s.world.add_good(Good {
            pos: at,
            ty: 26,
            alive: true,
        });
        let t = at.tile();
        let m = s.world.tile_mask(t);
        s.world.set_tile_mask(t, m | tile::AS_BUILDING);
        let square =
            [(0, 0), (-1, 0), (0, -1), (-1, -1)].map(|(dx, dy)| Pos::new(t.x + dx, t.y + dy));
        assert!(
            square
                .iter()
                .all(|&p| s.world.tile_mask(p) & tile::BLOCKED == 0),
            "the square is clear before the deploy"
        );
        s.ledgers[1].dirty = false;
        // No craft table: a `JOB_TIME` of 0, so the first frame casts.
        s.add_cast_order_at(u, crate::orders::spell::UNPACK, QueuePos::New);
        let Some(Body::Cast(c)) = s.current_order(u).map(|o| o.body) else {
            panic!("a cast order")
        };
        s.do_cast(u, c);
        assert!(!s.units[u].combat.packed, "deployed");
        let corner = Pos::new(t.x * UNITS_PER_TILE, t.y * UNITS_PER_TILE);
        assert_eq!(s.units[u].pos, corner, "seated on the tile's corner");
        assert_eq!(
            (at.x - corner.x, at.y - corner.y),
            (24, 24),
            "a step of (−24, −24) from where it stood"
        );
        assert!(
            square
                .iter()
                .all(|&p| s.world.tile_mask(p) & tile::BLOCKED != 0),
            "the four tiles under it are blocked"
        );
        assert!(s.ledgers[1].dirty, "and the leader's 0x2000000 is up");
        // Its death gives the square back.
        s.units[u].health = 0;
        s.close_supply(u);
        assert!(
            square
                .iter()
                .all(|&p| s.world.tile_mask(p) & tile::BLOCKED == 0),
            "a dead merchant's square is clear again"
        );
    }

    /// The re-seat's own gate: a tile `good_merchant_spot` refuses kills
    /// the order where it stands, before the clock ever starts.
    #[test]
    fn an_unpack_on_a_bad_spot_dies_on_its_first_frame() {
        let (mut s, u) = merchant_sim();
        merchant_job_time(&mut s);
        s.add_cast_order_at(u, crate::orders::spell::UNPACK, QueuePos::New);
        let Some(Body::Cast(c)) = s.current_order(u).map(|o| o.body) else {
            panic!("a cast order")
        };
        s.do_cast(u, c);
        assert!(s.units[u].orders.is_empty(), "no good under it, no deploy");
        assert_eq!(s.units[u].spell_time, 0, "and the clock never started");
        assert!(s.units[u].combat.packed, "still packed");
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

    /// The second search — and "ordered" is literal twice over. The
    /// candidate must hold a move-family order, **and the distance is
    /// measured to where it is going**: `find_unit_ordered` reads
    /// `UnitData +0x70/+0x74` — `orders_x`/`orders_y` — where its sibling
    /// `find_unit` reads the object's own XOR-ed position (`0065be35`
    /// against `0065cd0f`, the listing; the decompiler prints both as
    /// `vector_dist(unaff_EDI, unaff_ESI)`). So a merchant on the far side
    /// of the map, already walking at a good, keeps every other merchant
    /// off it — which is the whole of item 186 (`docs/JOURNAL.md`,
    /// 2026-09-02).
    #[test]
    fn a_sibling_ordered_at_the_good_refuses_it_from_anywhere() {
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

        // The sibling does not move: it is sent at the good from where it
        // stands, which is nowhere near it. The first search still passes
        // and the second is what refuses.
        let at = s.world.goods()[want].pos;
        s.add_move_order(sib, at, MoveKind::MoveTo, QueuePos::New, false);
        assert_eq!(s.order_type(sib), index::MOVE_TO, "the move family");
        let stands = s.units[sib].pos;
        assert!(
            vector_dist(stands.x - at.x, stands.y - at.y) > FRIENDLY_RANGE,
            "the body is far outside the cell the search asks about"
        );
        let going = s.units[sib].orders_pos;
        assert!(
            vector_dist(going.x - at.x, going.y - at.y) <= FRIENDLY_RANGE,
            "and `orders_x/y` is the good, 48-snapped"
        );
        rearm(&mut s, u);
        assert!(s.think_merchant(u));
        assert_eq!(walking_to(&s, u), Some(other), "refused this time");

        // And it is the **order family** that arms it: with the move
        // killed, `orders_x/y` stops naming the good and the good is
        // taken again.
        s.close_orders(sib);
        s.units[sib].orders_pos = stands;
        rearm(&mut s, u);
        assert!(s.think_merchant(u));
        assert_eq!(
            walking_to(&s, u),
            Some(want),
            "nothing is ordered there now"
        );
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
