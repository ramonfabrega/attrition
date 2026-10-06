//! `Wall::process@00640450`'s **site recruiter**: an unfinished wonder or
//! fort of a computer leader calls a citizen in (`docs/AI.md` §69).
//!
//! Every 32 frames phased by `o`, inside the same `(frame + o) & 31` block
//! as the under-attack decay and **ahead of the `helpers` reset**, a site
//! that is not active (`WallData::is_active`, `+8 & 4`) and whose owner is
//! not human (`leader_flags & 4`) takes this arm when it is a wonder
//! (`BuildData::is_wonder`, Build vslot `+0x2c`) or its type is a fort
//! (`ObjectTypeData::is_fort`, vslot `+0xfc`, by the PDB's method record)
//! with no damage (`+0x24`). It wants `max(4, helpers)` builders, one more
//! for each rival's unbuilt wonder of the same type that is ahead of it but
//! not far ahead, and while `helpers` is short of that it asks
//! `ObjectsData::find_unit@0065ca80(site, SEARCH_FRIENDLY, who, 0xf00,
//! 0x200, FILTER_TYPE 0x32, FILTER_NOT_BUSY)` for one citizen and hands it
//! `add_build_order(site, QUEUE_NEW, 0)` — no action bit, which is how the
//! dump tells this order from `produce_building`'s swarm (`UNITORDER flags
//! 0` on run202's block 15383).
//!
//! `helpers` is read before this frame's reset, so it is the number of
//! builders that worked the site since the last one. A wonder `do_move`
//! has just stripped of its builder is exactly the case: Great Lakes'
//! Pyramids `1/2026`, placed on 15382 with `1/70` swarmed onto it, the
//! swarm's approach refused the same frame and both orders gone — and
//! then, on the site's own phase (15382 + 2026 = 544 × 32), `1/70` is the
//! nearest citizen that is not busy and is sent straight back.

use crate::Sim;
use crate::orders::{QueuePos, index};
use crate::world::{Cell, Pos, vector_dist};

/// `max(4, helpers)`: the floor of builders a recruiting site wants.
const MIN_BUILDERS: i32 = 4;

/// The recruit's reach, `0xf00` — twenty tiles, in world units.
const RECRUIT_RANGE: i32 = 0xf00;

/// `__real_3eb33333`, the `0.35f` a rival's progress is compared against,
/// as the exact binary fraction the float holds: `0x333333 | 0x800000`
/// over `2^25`.
const RIVAL_EARLY: (i64, i64) = (11_744_051, 1 << 25);

impl Sim {
    /// The recruit arm, for building `b` on a frame where its `(frame + o)
    /// & 31` is zero. The caller has already taken the arm's two outer
    /// gates — not active, not a human's — and has **not** yet reset
    /// `helpers`.
    ///
    /// SEAM: the oil platform's arm beside it — every 128 frames, an
    /// unfinished `0x1a6` with no friendly unit targeting it
    /// (`find_unit(…, FILTER_TARGET, o, who)`) is disbanded — is not
    /// modelled; no capture on file has an oil platform site.
    pub(crate) fn site_recruit(&mut self, b: usize) {
        let bd = &self.buildings[b];
        // `BuildData::is_wonder@00472320`, the range `0x20d < type < 0x21f`
        // — the Forbidden City's `Ident` is not `Wonder`, its flag is (group
        // 5; A8 row 29).
        let wonder = self.is_wonder_site(b);
        let fort = bd
            .ty
            .is_some_and(|t| crate::build::is_fort(&self.build_types, t))
            && bd.damage == 0;
        if !wonder && !fort {
            return;
        }
        let helpers = bd.helpers;
        let mut want = helpers.max(MIN_BUILDERS);
        if wonder {
            want += self.rival_wonders(b);
        }
        if helpers >= want {
            return;
        }
        let (who, at) = (bd.owner, bd.pos);
        let Some(u) = self.find_idle_citizen(who, at, RECRUIT_RANGE) else {
            return;
        };
        self.add_build_order(u, b, QueuePos::New, false);
    }

    /// `BuildData::is_wonder@00472320`: the type's range flag, which the
    /// Forbidden City carries and its `Ident` does not.
    fn is_wonder_site(&self, b: usize) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|t| self.build_types[t].wonder)
    }

    /// The wonder's extra builders: one for every unbuilt wonder of the
    /// same type, held by a playing leader that is not this one's ally,
    /// whose progress is ahead of this one's and either under twice it or
    /// under `0.35`.
    ///
    /// Progress is `job_counter / construct_time(0)` in the original, two
    /// `float` quotients. Here each comparison is the exact rational one,
    /// cross-multiplied in `i64`. SEAM: the two part only where a
    /// quotient's rounding to `float` lands it on the other side of a
    /// bound, and `construct_time(0)` (Build vslot `+0x18c`) is read as the
    /// stored `constr_time`; no capture on file has two leaders building
    /// the same wonder. SEAM: a leader whose `wonderwin_timer` (`+0x44c`)
    /// is running counts **one** instead of its wonders; the timer is not
    /// carried, so every rival is walked.
    fn rival_wonders(&self, b: usize) -> i32 {
        let me = &self.buildings[b];
        let (j0, c0) = (i64::from(me.job_counter), i64::from(me.constr_time));
        if c0 <= 0 {
            return 0;
        }
        let who = me.owner;
        let mut extra = 0;
        for (o, bd) in self.buildings.iter().enumerate() {
            if o == b || !bd.alive || bd.active || bd.ty != me.ty || bd.owner == who {
                continue;
            }
            let w = bd.owner as usize;
            if self.defeated.get(w).copied().unwrap_or(true) || self.is_ally(bd.owner, who) {
                continue;
            }
            let (j, c) = (i64::from(bd.job_counter), i64::from(bd.constr_time));
            if c <= 0 {
                continue;
            }
            let ahead = j0 * c < j * c0;
            let close = j * c0 < 2 * j0 * c;
            let early = j * RIVAL_EARLY.1 < RIVAL_EARLY.0 * c;
            if ahead && (close || early) {
                extra += 1;
            }
        }
        extra
    }

    /// `ObjectsData::find_unit@0065ca80` as the recruiter calls it: the
    /// **nearest** of `who`'s own citizens (`SEARCH_FRIENDLY` is case 1 of
    /// `Search::valid_search@0067daa0`, the searcher's own leader) that is
    /// on the map, not busy, and in the tile region of `at`'s cell (the
    /// `0x200` flag), within `range` world units by `vector_dist`.
    ///
    /// - `FILTER_TYPE 0x32` is arm 0 of `Search::valid_filter`'s table
    ///   (`0067dbc3`): `ObjectData::is(PEASANTS, 1)`, the strict lineage.
    /// - `FILTER_NOT_BUSY` (12) is arm 11 (`0067e019`, read off the
    ///   listing): `SubObjectData::is_unit` (vslot `+0x18`, by the PDB),
    ///   then `UnitData::action_type` ∈ {NONE, GATHER, MOVE_TO}. A
    ///   gatherer is not busy.
    ///
    /// Both walks keep `<=` against the running best, so a tie goes to the
    /// **last** candidate in walk order. The list walk (while `total_units`
    /// is under `circle_radius[5]`) is the owner's units in `o` order; the
    /// circle walk is the cells of the ring in circle order, each cell's
    /// object chain from its head.
    pub(crate) fn find_idle_citizen(
        &self,
        who: crate::Player,
        at: Pos,
        range: i32,
    ) -> Option<usize> {
        let region = self.world.region_of(at.cell());
        let root = self.tech_tree.types.iter().position(|d| d.kind.is_unit())?;
        let fits = |u: usize| -> Option<i32> {
            let unit = &self.units[u];
            if !unit.alive() || !unit.on_map || unit.owner != who {
                return None;
            }
            let citizen = unit
                .ty
                .and_then(|r| self.unit_types[r].tree)
                .is_some_and(|t| self.tech_tree.is(t, root, true));
            if !citizen {
                return None;
            }
            let action = self
                .action_of(u)
                .map_or(index::NONE, |i| unit.orders[i].index());
            if !matches!(action, index::NONE | index::GATHER | index::MOVE_TO) {
                return None;
            }
            if self.world.region_of(unit.pos.cell()) != region {
                return None;
            }
            let d = vector_dist((unit.pos.x - at.x).abs(), (unit.pos.y - at.y).abs());
            (d <= range).then_some(d)
        };
        let mut best: Option<(i32, usize)> = None;
        let take = |u: usize, best: &mut Option<(i32, usize)>| {
            if let Some(d) = fits(u)
                && best.is_none_or(|(bd, _)| d <= bd)
            {
                *best = Some((d, u));
            }
        };
        let circle = crate::ai_place::circle();
        let ring = ((range + 0x2ff) / 0x300) as usize;
        let live = self.units.iter().filter(|x| x.alive()).count();
        if live < circle.radius[ring] {
            let mut mine: Vec<usize> = (0..self.units.len())
                .filter(|&u| self.units[u].owner == who)
                .collect();
            mine.sort_by_key(|&u| self.units[u].index);
            for u in mine {
                take(u, &mut best);
            }
        } else {
            let c0 = at.cell();
            let width = self.world.width() as usize;
            for i in 0..circle.radius[ring] {
                let c = Cell::new(c0.x + circle.x[i], c0.y + circle.y[i]);
                if !self.world.contains(c) || self.world.region_of(c) != region {
                    continue;
                }
                let mut next = self.chain_heads[(c.y as usize) * width + (c.x as usize)];
                while let Some(u) = next {
                    next = self.units[u].down;
                    take(u, &mut best);
                }
            }
        }
        best.map(|(_, u)| u)
    }
}

#[cfg(test)]
mod tests {
    use crate::build::{BuildType, Ident};

    /// **The recruiter's wonder test is the range flag** (twenty-fourth
    /// pass, group 5; A8 row 29): a Forbidden City site recruits like any
    /// other wonder, though its `Ident` is `ForbiddenCity`.
    #[test]
    fn a_forbidden_city_site_is_a_wonder_to_the_recruiter() {
        let mut s = crate::Sim::new(crate::Tuning::RON, crate::world::World::new(8, 8), 2);
        let t = s.add_build_type(BuildType {
            ident: Ident::ForbiddenCity,
            wonder: true,
            ..BuildType::default()
        });
        let b = s.add_building(1, crate::Pos::new(0x600, 0x600), 1);
        s.buildings[b].ty = Some(t);
        assert!(s.is_wonder_site(b));
        s.build_types[t].wonder = false;
        assert!(!s.is_wonder_site(b));
    }
}
