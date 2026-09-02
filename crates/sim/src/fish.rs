//! `Unit::think_fish@005f4c60` — where an idle fishing boat goes next.
//!
//! The specification is `docs/ORDERS.md` §6.8. In one paragraph: an idle
//! Fisherman scores the 289 cells of the 17 × 17 around its own, keeps the
//! ones that are water in its own region and unblocked, and takes the best
//! of `1_000_000 / manhattan` — a million if the cell holds a good it may
//! gather, nothing at all if it does not — plus a jitter of `rnd(60)` and
//! **plus the ring index itself**. If the winner is the cell it is standing
//! on it unpacks; otherwise it walks there.
//!
//! Three arms, all three of them in run58's capture and all three needed:
//! frame 4462 has no good anywhere in range and the `+ i` bias alone
//! decides it; 4871 has one Fish five cells off and the million carries it;
//! 4948 is the boat standing on that Fish, which is the cast.

use crate::orders::{MoveKind, QueuePos};
use crate::tech::TypeId;
use crate::world::{Cell, Good, MOVE_289, OIL, Pos, UNITS_PER_CELL};
use crate::{Player, Sim};

/// `TypeIndex::FISHERMEN` — the lineage `Unit::think@005f6e40:1421` tests
/// with `is(0x13d, 0)` before it calls this at all.
pub const FISHERMEN: TypeId = 0x13d;

/// The spell `Unit::add_cast_order` rewrites the generic **unpack**
/// (`0x28c`) into for a unit in the `FISHERMEN` lineage, and the value
/// run58's block 4949 prints as `spell 658`.
pub const UNPACK_FISHERMEN: i32 = crate::orders::spell::UNPACK_FISHERMEN;

/// The generic unpack, as `think_fish` asks for it.
pub const UNPACK: i32 = crate::orders::spell::UNPACK;

/// The numerator a cell that holds a gatherable good scores before the
/// distance divides it (`005f4e2e`: `0xf4240`).
const GOOD_SCORE: i32 = 1_000_000;

/// The jitter's span — `rnd(0x3c)`, added to every candidate's score.
const JITTER: i32 = 0x3c;

/// The head's cadence: an **unpacked** fisherman re-examines where it is
/// standing one frame in 1,024, phased by `o` (`005f4c8f`).
const RECHECK_PERIOD: i64 = 0x400;

/// The one draw site, under the original's own offset from
/// `think_fish@005f4c60` — the `rnd(60)` each accepted cell spends. The
/// second draw in the function, at `+0x2c9`, is the fallback for
/// `best > 0x120`, and `best` starts at 0 and is only ever assigned a loop
/// index, so nothing can reach it (§6.8, "The unreachable half").
pub const SITE_JITTER: &str = "Unit::think_fish+0x27a";

/// `Unit::think_fish` up to `Unit::think_merchant`, the next function in
/// the export — what isolates this mechanic's own draws in a trace.
pub const CODE: std::ops::Range<u32> = 0x005f_4c60..0x005f_5170;

impl Sim {
    /// `ObjectsData::find_good_at@0065bec0(x, y, who, 0, 0)` — the fast
    /// path, which is the only one `think_fish` uses.
    ///
    /// The original walks the cell's object chain and asks whether it ends
    /// in a good; here the terminator is a field, because
    /// `Objects::init_good` is the only thing that ever writes one
    /// ([`crate::world::World::good_at`]). Then: the good must be live, it
    /// must not be `OIL`, and — since `who` is a real player here —
    /// `LeaderData::type_avail(t, 1)` must be non-zero.
    ///
    /// Answers the good's **`TypeIndex`**, as the original does; every
    /// caller here only asks whether there is one.
    pub(crate) fn find_good_at(&self, c: Cell, who: Player) -> Option<TypeId> {
        let (_, g): (usize, Good) = self.world.good_at(c)?;
        if !g.alive || g.ty == OIL {
            return None;
        }
        (self.type_avail(who, g.ty) != crate::tech::NOT_AVAILABLE).then_some(g.ty)
    }

    /// `Unit::think_fish@005f4c60` (`docs/ORDERS.md` §6.8). `true` when it
    /// did something — a move order or a cast — which ends the caller's
    /// `think`.
    pub(crate) fn think_fish(&mut self, u: usize, frame: i64) -> bool {
        let packs = self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].combat.packs);
        let packed = self.units[u].combat.packed;
        let who = self.units[u].owner;
        let o = i64::from(self.units[u].index);

        // The head, and it is only for a boat that has already deployed:
        // once in 1,024 frames it asks whether it can still gather where it
        // stands, records the answer in `unit_masks & 0x20`, and gives up
        // for this frame if it can. A **packed** fisherman skips the whole
        // thing and searches every time it is called.
        //
        // SEAM: `UnitData::calc_gather@00609180` is not modelled, so the
        // answer taken here is "no, it cannot gather here", which sends the
        // boat back through the search, and `unit_masks & 0x20` — the bit
        // the original records the answer in — is not kept. run58 reaches
        // this on frame 5106 and spends **no draw** either way, so no
        // capture on disk tells the two apart (§6.8, "What is not
        // established").
        if packs && !packed && (o + frame) % RECHECK_PERIOD != 0 {
            return false;
        }

        let here = self.units[u].pos.cell();
        let Some(region) = self.world.tregion_alt(self.units[u].pos.tile()) else {
            return false;
        };

        // The walk. `best` starts at **0** — the unit's own cell — and the
        // score to beat at 0, so a fisherman that accepts nothing anywhere
        // ends up pointing at itself.
        let mut best = 0usize;
        let mut best_score = 0i32;
        for (i, &(dx, dy)) in MOVE_289.iter().enumerate() {
            let c = Cell::new(here.x + dx, here.y + dy);
            if !self.world.contains(c) {
                continue;
            }
            let d = self.world.cell_data(c);
            // `flags & 0x100` is the coastal half-land bit, and a cell that
            // carries it is skipped outright.
            if d.flags & 0x100 != 0 {
                continue;
            }
            // `land_key[]`: 1 SANDY, 2 OCEAN — the water pair.
            if d.land != 1 && d.land != 2 {
                continue;
            }
            // `WData.region`, not `get_tregion`: the candidate is compared
            // raw against the *unit's* `get_tregion`.
            if self.world.region_of(c) != Some(region) {
                continue;
            }
            let mut score = if self.find_good_at(c, who).is_some() {
                // A good another fisherman has already claimed — the cell's
                // chain head is somebody else's object — is skipped
                // entirely, and skipped **before** the draw.
                if let Some((down, down_who)) = self.claim_of(c)
                    && down_who < 8
                    && down != self.units[u].index
                {
                    continue;
                }
                GOOD_SCORE
            } else {
                0
            };
            let dist = ((here.x - c.x).abs() + (here.y - c.y).abs()).max(1);
            score /= dist;
            self.mark(SITE_JITTER);
            score += self.rng.roll() % JITTER + i32::try_from(i).expect("289 fits");
            if best_score < score {
                best = i;
                best_score = score;
            }
        }

        let (dx, dy) = MOVE_289[best];
        if dx == 0 && dy == 0 {
            // Standing on the winner. A packed boat deploys; an unpacked
            // one has nothing left to do.
            if packs && packed {
                self.add_cast_order(u, UNPACK);
                return true;
            }
            return false;
        }
        let to = Pos::new(
            (here.x + dx) * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            (here.y + dy) * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        );
        self.add_move_order(u, to, MoveKind::MoveTo, QueuePos::New, false);
        true
    }

    /// `WData.down` / `down_who` as `think_fish` reads them: the object at
    /// the head of the cell's chain, when there is one.
    fn claim_of(&self, c: Cell) -> Option<(i16, i8)> {
        let d = self.world.cell_data(c);
        (d.down >= 0).then_some((d.down, d.down_who))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tuning;
    use crate::orders::Body;
    use crate::world::{CellData, Terrain, World};

    /// A 40 × 40 all-ocean world with one AI player, and a Fisherman —
    /// packing, sea domain, in the `FISHERMEN` lineage — standing at cell
    /// `(20, 20)`. `land 2` is `OCEAN`, which is what the search accepts.
    fn fish_sim() -> (Sim, usize) {
        let mut world = World::new(40, 40);
        world.fill_region(Terrain::Sea, Cell::new(0, 0), Cell::new(39, 39));
        for y in 0..40 {
            for x in 0..40 {
                world.set_cell_data(
                    Cell::new(x, y),
                    CellData {
                        land: 2,
                        // `WData.down` is `−1` on an empty cell; the
                        // default `0` would read as "object 0 is standing
                        // here" and take every candidate out of the walk.
                        down: -1,
                        ..CellData::default()
                    },
                );
            }
        }
        let mut s = Sim::new(Tuning::RON, world, 2);
        s.nation[0].human = true;
        s.nation[1].human = false;
        // The `TypeIndex` space up to `FISHERMEN`, so the lineage tests in
        // `think` and `add_cast_order` are the real ones. `FISH` is 6 and
        // `OIL` 5, as `TypeIndex` has them.
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
                t.add(crate::tech::TypeDef::good(n));
            }
            while t.types.len() < FISHERMEN {
                t.add(crate::tech::TypeDef::plain("filler", -1));
            }
            let boat = t.add(crate::tech::TypeDef::unit(
                "Fishing Boat",
                crate::tech::UnitTraits::default(),
            ));
            assert_eq!(boat, FISHERMEN, "the fixture's ids are TypeIndex's");
        }
        s.tech = (0..2)
            .map(|_| crate::tech::PlayerTech::new(&s.tech_tree))
            .collect();
        let t = s.add_unit_type(crate::UnitType {
            hits: 20,
            moves: 40,
            tree: Some(FISHERMEN),
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

    /// `TypeIndex::FISH`.
    const A_GOOD: usize = 6;

    /// `Unit::init@00612100:376` — the bit the whole cadence hangs off.
    #[test]
    fn a_type_that_packs_is_born_packed() {
        let (s, u) = fish_sim();
        assert!(
            s.units[u].combat.packed,
            "a Fisherman leaves the dock packed, which is what lets it search \
             on its first idle frame instead of once in 1,024"
        );
    }

    /// The 17 × 17 walk's shape, and the shipped table's typo inside it.
    #[test]
    fn the_move_table_covers_the_whole_seventeen_by_seventeen() {
        assert_eq!(crate::world::MOVE_289.len(), 0x121);
        assert_eq!(&crate::world::MOVE_289[..49], &crate::world::MOVE_49);
        // Every entry but the last is in the ring its index says, and the
        // ring boundaries are the odd squares.
        assert_eq!(crate::world::MOVE_289[0], (0, 0));
        for (i, &(dx, dy)) in crate::world::MOVE_289.iter().enumerate().take(288).skip(1) {
            let r = dx.abs().max(dy.abs());
            assert!(
                ((2 * r - 1) * (2 * r - 1)..(2 * r + 1) * (2 * r + 1))
                    .contains(&i32::try_from(i).unwrap()),
                "entry {i} {:?} is not in ring {r}",
                (dx, dy)
            );
        }
        assert_eq!(
            crate::world::MOVE_289[288],
            (-8, -16),
            "`move_y[288]` is the shipped table's typo; \
             `rondata::pe` checks it against the executable"
        );
    }

    /// **Nothing worth having anywhere**, which is run54's frame 4462: every
    /// numerator is zero, so the score is `rnd(60) + i` and the *last*
    /// accepted index all but always wins. The last index is 288, and 288 is
    /// the typo — so an empty sea sends a fishing boat sixteen cells north
    /// and eight west, out of the square it just searched.
    #[test]
    fn an_empty_sea_is_decided_by_the_ring_index_and_lands_on_the_typo() {
        let (mut s, u) = fish_sim();
        // Two candidates only — the unit's own cell and the typo's — so the
        // `+ i` term decides it outright rather than on a jitter: index 288
        // beats index 0 by 288 against at most 59. Everything else is
        // `land 0`, `BASELAND`, which the walk refuses.
        for y in 0..40 {
            for x in 0..40 {
                if (x, y) == (20, 20) || (x, y) == (12, 4) {
                    continue;
                }
                let c = Cell::new(x, y);
                let mut d = s.world.cell_data(c);
                d.land = 0;
                s.world.set_cell_data(c, d);
            }
        }
        s.units[u].idle = 1;
        assert!(s.think_fish(u, 0), "it takes a move order");
        let Some(Body::Move(m)) = s.units[u].orders.front().map(|o| o.body) else {
            panic!("a move order")
        };
        assert_eq!(
            m.dest.cell(),
            Cell::new(12, 4),
            "(20, 20) + move_289[288] = (20 − 8, 20 − 16)"
        );
    }

    /// **A good five cells off**, which is run54's frame 4871: a million
    /// over the Manhattan distance dwarfs every jitter, so the fish wins
    /// however late in the ring the empty water is.
    #[test]
    fn a_good_in_range_beats_every_empty_cell() {
        let (mut s, u) = fish_sim();
        s.world.add_good(crate::world::Good {
            pos: Pos::new(17 * 768 + 384, 18 * 768 + 384),
            ty: A_GOOD,
            alive: true,
        });
        s.units[u].idle = 1;
        assert!(s.think_fish(u, 0));
        let Some(Body::Move(m)) = s.units[u].orders.front().map(|o| o.body) else {
            panic!("a move order")
        };
        assert_eq!(m.dest.cell(), Cell::new(17, 18), "the fish, at distance 5");
    }

    /// **Standing on it**, which is run54's frame 4948: the winner is the
    /// unit's own cell, so a packed boat unpacks instead of moving — and
    /// `add_cast_order` re-aims the generic `0x28c` at the Fisherman's own
    /// `0x292`, which is the `spell 658` the dump prints.
    #[test]
    fn a_boat_on_its_fish_unpacks_with_the_fisherman_s_own_spell() {
        let (mut s, u) = fish_sim();
        let at = s.units[u].pos;
        s.world.add_good(crate::world::Good {
            pos: at,
            ty: A_GOOD,
            alive: true,
        });
        s.units[u].idle = 1;
        assert!(s.think_fish(u, 0));
        let Some(Body::Cast(c)) = s.units[u].orders.front().map(|o| o.body) else {
            panic!("a cast order")
        };
        assert_eq!(c.spell, UNPACK_FISHERMEN, "run58 block 4949: `spell 658`");
    }

    /// **And the cast survives to run**, which is item 149 end to end
    /// (`docs/ORDERS.md` §6.9). run58's `1/14` queues `0x292` on frame
    /// 4948, and its dump then prints `spell_time` climbing 1 … 39 on
    /// 4950 … 4988 and `unit_masks 786440 → 262152` on 4989 — forty
    /// `do_cast` steps, because `craftrules.xml` gives `Deploy`
    /// (`Fishermen`) a `JOB_TIME` of 40. Nothing is drawn in any of them,
    /// and the boat is still packed until the last.
    #[test]
    fn the_deploy_waits_out_its_job_time_and_then_clears_the_packed_bit() {
        let (mut s, u) = fish_sim();
        // The craft table as the install has it: 55 rows, and `0x292` is
        // the thirtieth.
        s.spells = vec![crate::orders::SpellType::default(); 55];
        let row = usize::try_from(UNPACK_FISHERMEN - crate::orders::spell::FIRST).unwrap();
        s.spells[row].job_time = 40;
        assert_eq!(s.spell_job_time(UNPACK_FISHERMEN), 40);
        let at = s.units[u].pos;
        s.world.add_good(crate::world::Good {
            pos: at,
            ty: A_GOOD,
            alive: true,
        });
        s.units[u].idle = 1;
        assert!(s.think_fish(u, 0));
        let seed = s.rng.seed;
        for step in 1..40 {
            s.work(u, i64::from(step));
            assert_eq!(s.units[u].spell_time, step, "one step a frame");
            assert!(s.units[u].combat.packed, "still packed on step {step}");
            assert!(
                matches!(
                    s.units[u].orders.front().map(|o| o.body),
                    Some(Body::Cast(_))
                ),
                "and the order is still the cast on step {step}"
            );
        }
        s.work(u, 40);
        assert!(!s.units[u].combat.packed, "the fortieth step deploys it");
        assert_eq!(s.units[u].spell_time, 0, "and the clock is put back");
        assert!(
            s.units[u].orders.is_empty(),
            "`do_cast` kills every craft but the transport once it has cast"
        );
        assert_eq!(s.rng.seed, seed, "and the whole deploy spends no draw");
    }

    /// The other half of the same frame: a `JOB_TIME` the table does not
    /// carry is **0**, and a cast with one lands on its first step. That is
    /// the transport craft's own number, and it is why the fixtures that
    /// board a barge need no table at all.
    #[test]
    fn a_craft_with_no_row_casts_on_its_first_step() {
        let (s, _) = fish_sim();
        assert!(s.spells.is_empty());
        assert_eq!(s.spell_job_time(crate::orders::spell::TRANSPORT), 0);
        assert_eq!(s.spell_job_time(UNPACK_FISHERMEN), 0);
    }

    /// An **oil** patch is in the goods list and in no cell's chain
    /// (`Objects::init_good@00653f30` returns before it writes the
    /// terminator), and `find_good_at` refuses it a second time by type. So
    /// a boat sitting on oil is not sitting on anything.
    #[test]
    fn oil_is_not_a_good_a_fisherman_can_see() {
        let (mut s, u) = fish_sim();
        let at = s.units[u].pos;
        s.world.add_good(crate::world::Good {
            pos: at,
            ty: OIL,
            alive: true,
        });
        assert!(s.find_good_at(at.cell(), 1).is_none());
        s.units[u].idle = 1;
        assert!(s.think_fish(u, 0), "it moves rather than unpacking");
    }

    /// The head's cadence: an **unpacked** boat looks once in 1,024 frames,
    /// phased by `o`, and spends nothing on the other 1,023.
    #[test]
    fn an_unpacked_boat_re_examines_its_water_once_in_1024_frames() {
        let (mut s, u) = fish_sim();
        s.units[u].combat.packed = false;
        s.units[u].idle = 1;
        let before = s.rng.seed;
        assert!(!s.think_fish(u, 5), "14 + 5 is not a multiple of 1,024");
        assert_eq!(s.rng.seed, before, "and it spends no draw finding out");
        // `o` is 14, so 1,010 is the first frame it looks.
        assert!(s.think_fish(u, 1010));
    }

    /// A cell whose good another player's object is standing on is skipped
    /// **before** the jitter, and a cell holding the unit's own object is
    /// not (`005f4e14`: `down >= 0 && down_who < 8 && down != o`).
    #[test]
    fn a_good_another_unit_is_standing_on_is_not_a_candidate() {
        let (mut s, u) = fish_sim();
        let at = Pos::new(17 * 768 + 384, 18 * 768 + 384);
        s.world.add_good(crate::world::Good {
            pos: at,
            ty: A_GOOD,
            alive: true,
        });
        let mut d = s.world.cell_data(at.cell());
        d.down = 3;
        d.down_who = 0;
        s.world.set_cell_data(at.cell(), d);
        s.units[u].idle = 1;
        assert!(s.think_fish(u, 0));
        let Some(Body::Move(m)) = s.units[u].orders.front().map(|o| o.body) else {
            panic!("a move order")
        };
        assert_ne!(
            m.dest.cell(),
            Cell::new(17, 18),
            "somebody else's claim takes the cell out of the walk"
        );
    }
}
