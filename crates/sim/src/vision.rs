//! What a unit reveals as it moves — `docs/VISION.md`.
//!
//! Three things: the line of sight a unit has (`Unit::update_los`), the disc
//! of fog cells that lights up around it (`Object::update_seen`), and the two
//! moments it runs — a step that crosses a half-cell, and the hundredth-frame
//! resync.
//!
//! **Why it is load-bearing rather than cosmetic.** `seen2` is monotone and
//! three things read it: `PathFinder::calc_cost`'s fog branch
//! (`docs/PATHFINDER.md` §5), `Unit::think_scout`'s cell filter
//! (`docs/SCOUT.md` §7) and the AI's site census (`docs/AI.md` §15.8). Until
//! this module existed the grid was the `WORLD` dump's frame-0 snapshot and
//! nothing wrote it, so a path planned at frame 100 was planned against a map
//! the unit had walked off the edge of.
//!
//! Nothing here draws on the sync stream.

use crate::Sim;
use crate::ai_load::uflags2;
use crate::ai_place::circle;
use crate::attrition::Domain;
use crate::movement::{cos_component, sin_component};
use crate::world::{Pos, vector_dist};
use std::sync::OnceLock;

/// A tile in position units — `TCoord`'s scale, and what `<LOS>` is measured
/// in.
const UNITS_PER_TILE: i32 = 0xc0;
/// A fog cell in position units: half a world cell.
pub const UNITS_PER_FOG: i32 = 0x180;
/// `Object::update_seen`'s cap on the radius, in fog cells.
const MAX_RADIUS: i32 = 0x40;
/// `ring_init`'s last ring, and the ring branch's own clamp.
const RING_LAST: i32 = 0x20;
/// The distance a small land unit's vision is thrown forward, from the
/// `mov $0x180, %edx` at `651d00`.
const PROJECT_DIST: i32 = UNITS_PER_FOG;

/// `ring_x`/`ring_y`/`ring_radius` — `circle`'s thickened twin, the offsets
/// an *incremental* reveal walks (`docs/VISION.md` §4).
pub struct Ring {
    pub x: Vec<i32>,
    pub y: Vec<i32>,
    /// `ring_radius[r]`: how many entries lie within ring `r`. Only
    /// `0..=0x20` are filled; `ring_init` stops there.
    pub radius: [usize; RING_LAST as usize + 1],
}

/// The four orthogonal offsets `ring_init` patches a ring's gaps with, read
/// out of the PE at `.rdata` `00add254` (x) and `00add214` (y): north, east,
/// south, west.
const ORTHOG: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// `ring_init@00681920`, rebuilt.
pub fn ring() -> &'static Ring {
    static RING: OnceLock<Ring> = OnceLock::new();
    RING.get_or_init(|| {
        let c = circle();
        let mut r = Ring {
            x: vec![0],
            y: vec![0],
            radius: [1; RING_LAST as usize + 1],
        };
        for k in 1..=RING_LAST as usize {
            // The ring is the circle's own, copied out whole.
            let (cs, ce) = (c.radius[k - 1], c.radius[k]);
            if cs < ce {
                r.x.extend_from_slice(&c.x[cs..ce]);
                r.y.extend_from_slice(&c.y[cs..ce]);
            }
            let start = r.radius[k - 1];
            r.radius[k] = r.x.len();
            // Then patched: a point with fewer than two orthogonal
            // neighbours in the ring so far gains the ones that lie strictly
            // inside it. The count walks the **live** end, so a point added
            // for one `i` counts for the next.
            let end = r.x.len();
            for i in start..end {
                let (x, y) = (r.x[i], r.y[i]);
                let mut cnt = (start..r.x.len())
                    .filter(|&j| (y - r.y[j]).abs() + (x - r.x[j]).abs() == 1)
                    .count();
                if x == 0 || y == 0 {
                    cnt += 1;
                }
                if cnt >= 2 {
                    continue;
                }
                for (ox, oy) in ORTHOG {
                    if vector_dist(x + ox, y + oy) < k as i32 {
                        r.x.push(x + ox);
                        r.y.push(y + oy);
                    }
                }
                r.radius[k] = r.x.len();
            }
        }
        r
    })
}

/// Which table an `update_seen` pass walks, and the slice of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sweep {
    /// The fog cell the disc is centred on.
    pub centre: (i32, i32),
    /// The radius in fog cells, after every clamp `update_seen` applies.
    pub radius: i32,
    /// Whether the offsets come from [`ring`] rather than from `circle`.
    pub ring: bool,
    pub start: usize,
    pub end: usize,
}

/// `p >> 7` through `div_3_table` — the fog cell a position lies in,
/// `p / 0x180` floored. The same read `PathFinder::calc_cost` makes
/// (`docs/PATHFINDER.md` §5).
pub const fn fog_of(p: i32) -> i32 {
    p.div_euclid(UNITS_PER_FOG)
}

impl Sim {
    // ------------------------------------------------------------------
    // §2 — the line of sight
    // ------------------------------------------------------------------

    /// `Unit::update_los@0060e4d0` into `UnitData::los@006100c0`, as far as
    /// this simulation models it: the type's own `LOS`, the citizen terms,
    /// the science term, and the two clamps. `docs/VISION.md` §2 tabulates
    /// the seven terms that are read and not implemented; none of them can
    /// fire in any capture on disk.
    ///
    /// Zero for a unit whose type has no `LOS`. The head's other exit —
    /// `leader_flags & 1`, an unused leader slot — has no counterpart here,
    /// where every player in [`Sim::players`] exists.
    pub fn unit_los(&self, u: usize) -> i32 {
        let unit = &self.units[u];
        let who = unit.owner as usize;
        let Some(rec) = unit.ty else { return 0 };
        let ty = &self.unit_types[rec];
        let mut los = ty.los;
        if los == 0 {
            return 0;
        }
        // Term 3: the citizen upgrades and the nomad start. `TypeIndex`
        // `0x32`/`0x33` are the two citizens.
        if matches!(unit.type_index, 0x32 | 0x33) && self.lobby.starting_town == 0 {
            los += 2;
        }
        // Term 4: the science line.
        let epoch = self
            .tech
            .get(who)
            .map_or(0, |t| t.epoch[crate::tech::Line::Science.index()]);
        los += epoch * ty.science_los;
        // Term 5: a packed siege engine sees four tiles at most. This
        // simulation has no packing state, so the branch is the type test
        // alone and it is a seam — `docs/VISION.md` §7.
        if !ty.cols.flag2(uflags2::PACKS) {
            // Term 5b: the two merchants and the fur trapper see a fixed
            // radius that ignores everything above.
            if matches!(unit.type_index, 0x3d | 0x3e | 0x190) {
                los = epoch + 4;
            }
        }
        los
    }

    // ------------------------------------------------------------------
    // §3, §4 — the disc
    // ------------------------------------------------------------------

    /// The centre, radius and index range `Object::update_seen` would walk
    /// for this unit, or `None` when it reveals nothing.
    ///
    /// Split out from [`Sim::update_seen`] so the arithmetic can be tested
    /// without a fog grid.
    pub fn seen_sweep(&self, u: usize, ring_pass: bool) -> Option<Sweep> {
        let los = self.unit_los(u);
        if los == 0 {
            return None;
        }
        let mut r = (los * UNITS_PER_TILE) / UNITS_PER_FOG;
        if r > MAX_RADIUS {
            r = MAX_RADIUS;
        }
        let c = circle();
        let unit = &self.units[u];
        let pos = unit.pos;
        if r < 4 {
            // A small, ordinary, land unit sees from a half-cell in front of
            // its own nose.
            let projects = unit.ty.is_some_and(|rec| {
                self.unit_types[rec].kind.domain == Domain::Land
                    && !self.unit_types[rec].cols.flag2(uflags2::PACKS)
            });
            let (centre, start) = if projects {
                let facing = unit.movement.facing;
                let p = Pos::new(
                    pos.x + sin_component(facing, PROJECT_DIST),
                    pos.y - cos_component(facing, PROJECT_DIST),
                );
                // `r - 5 < 1` for every `r < 4`, so this is always
                // `circle_radius[0]` — the whole disc bar its centre.
                let start = if ring_pass { c.radius[0] } else { 0 };
                ((fog_of(p.x), fog_of(p.y)), start)
            } else {
                // The original's `iVar5 = r - 1; if (iVar5 < 1) …` — a
                // radius of one takes the whole disc rather than a ring of
                // eight, because there is no ring below it.
                let start = if ring_pass && r > 1 {
                    c.radius[(r - 1) as usize]
                } else {
                    0
                };
                ((fog_of(pos.x), fog_of(pos.y)), start)
            };
            return Some(Sweep {
                centre,
                radius: r,
                ring: false,
                start,
                end: c.radius[r as usize],
            });
        }
        let centre = (fog_of(pos.x), fog_of(pos.y));
        if !ring_pass {
            return Some(Sweep {
                centre,
                radius: r,
                ring: false,
                start: 0,
                end: c.radius[r as usize],
            });
        }
        if r > RING_LAST {
            r = RING_LAST;
        }
        let g = ring();
        let inner = if r - 1 < 1 { 0 } else { r - 1 } as usize;
        Some(Sweep {
            centre,
            radius: r,
            ring: true,
            start: g.radius[inner],
            end: g.radius[r as usize],
        })
    }

    // ------------------------------------------------------------------
    // §5 — the write
    // ------------------------------------------------------------------

    /// `Object::update_seen@00651b80` for a unit: light its disc into the
    /// fog. `ring_pass` is the original's `param_1` — set by a walking
    /// unit, clear by one arriving on the map.
    ///
    /// Returns how many fog cells this player saw for the first time, which
    /// is the count of `reveal_fog` calls the original would make; nothing
    /// reads it but the tests.
    pub fn update_seen(&mut self, u: usize, ring_pass: bool) -> usize {
        if !self.world.has_fog() || !self.units[u].on_map || !self.units[u].alive() {
            return 0;
        }
        let who = self.units[u].owner;
        if who >= 8 {
            return 0;
        }
        let Some(sw) = self.seen_sweep(u, ring_pass) else {
            return 0;
        };
        let mask = 1u8 << who;
        let (cx, cy) = sw.centre;
        let (fw, fh) = (self.world.fog_xs(), self.world.fog_ys());
        // The hoisted test: the whole disc on the grid, so the per-point
        // bounds check can be skipped.
        let whole = cx - sw.radius >= 0
            && cy - sw.radius >= 0
            && cx + sw.radius < fw
            && cy + sw.radius < fh;
        let (xs, ys): (&[i32], &[i32]) = if sw.ring {
            let g = ring();
            (&g.x, &g.y)
        } else {
            let c = circle();
            (&c.x, &c.y)
        };
        let mut revealed = 0;
        for i in sw.start..sw.end {
            let (x, y) = (xs[i] + cx, ys[i] + cy);
            if (whole || (x >= 0 && y >= 0 && x < fw && y < fh)) && self.world.set_seen(x, y, mask)
            {
                revealed += 1;
            }
        }
        revealed
    }

    // ------------------------------------------------------------------
    // §6 — the cadence
    // ------------------------------------------------------------------

    /// The half-cell test `Unit::set_new_location@005f8d20` puts in front of
    /// `update_seen`, and the call. `from` is where the unit was before the
    /// step; the unit has already been moved.
    /// Answers `None` when the step stayed inside one fog cell and the
    /// reveal was therefore skipped, and `Some(newly revealed)` when it ran
    /// — the distinction a fog-cell count cannot make, since a second
    /// sweep from the same centre reveals nothing either way.
    pub(crate) fn moved_to(&mut self, u: usize, from: Pos, ring_pass: bool) -> Option<usize> {
        let to = self.units[u].pos;
        if fog_of(from.x) == fog_of(to.x) && fog_of(from.y) == fog_of(to.y) {
            return None;
        }
        Some(self.update_seen(u, ring_pass))
    }

    /// `GameDaemon::update_all_seen@00732840`, on the plane this simulation
    /// keeps: every unit's whole disc, `frame % 100 == 0x21`.
    ///
    /// The original clears `seen` and `seen3` first and rebuilds `seen` from
    /// every object, buildings included. Here the clear is skipped and only
    /// units are walked, because `seen2` is monotone — the pass cannot
    /// *remove* a bit from it — and `seen` has no reader. What it is for is
    /// the units the incremental path misses: one that never crosses a
    /// half-cell still gets its disc every hundred frames.
    pub(crate) fn update_all_seen(&mut self) {
        if !self.world.has_fog() {
            return;
        }
        for u in 0..self.units.len() {
            self.update_seen(u, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tuning;
    use crate::ai_place::circle;
    use crate::movement::Angle;
    use crate::world::{Cell, Terrain, World};

    /// A 40 × 40 land world under an all-dark fog grid, one player, one
    /// unit of a type whose `LOS` the caller picks.
    fn fog_sim(los: i32, science_los: i32) -> (crate::Sim, usize) {
        let mut world = World::new(40, 40);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
        assert!(world.set_fog(vec![0; 80 * 80]));
        let mut s = crate::Sim::new(Tuning::RON, world, 2);
        let t = s.add_unit_type(crate::UnitType {
            hits: 20,
            moves: 40,
            los,
            science_los,
            ..crate::UnitType::default()
        });
        let at = Pos::new(20 * 0x300 + 0x180, 20 * 0x300 + 0x180);
        let mut u = crate::Unit::new(0, 0, at, 20);
        u.ty = Some(t);
        let u = s.add_unit(u);
        (s, u)
    }

    /// How many fog cells this player has ever seen.
    fn seen(s: &crate::Sim, who: u8) -> usize {
        let (fw, fh) = (s.world.fog_xs(), s.world.fog_ys());
        (0..fh)
            .flat_map(|y| (0..fw).map(move |x| (x, y)))
            .filter(|&(x, y)| s.world.seen2(x, y).is_some_and(|v| v & (1 << who) != 0))
            .count()
    }

    /// §3: `LOS` is in tiles and the fog radius is half of it, truncating.
    /// The five the Ancient age actually fields, and the cap.
    #[test]
    fn the_radius_is_half_the_los_in_tiles() {
        for (los, want) in [
            (0, None),
            (2, Some(1)),
            (4, Some(2)),
            (6, Some(3)),
            (8, Some(4)),
            (11, Some(5)),
            (200, Some(0x40)),
        ] {
            let (s, u) = fog_sim(los, 0);
            assert_eq!(
                s.seen_sweep(u, false).map(|w| w.radius),
                want,
                "LOS {los} tiles"
            );
        }
    }

    /// §2 term 4: the science line multiplies `SCIENCE_LOS`, and it is
    /// `epoch[3]` — a Scout's `4 + 1 × 2 = 6`, which run10's frame 202
    /// confirms against the original's own `mylos`.
    #[test]
    fn a_science_level_adds_science_los_and_the_radius_follows() {
        let (mut s, u) = fog_sim(4, 2);
        assert_eq!(s.unit_los(u), 4);
        assert_eq!(s.seen_sweep(u, false).map(|w| w.radius), Some(2));
        s.tech[0].epoch[crate::tech::Line::Science.index()] = 1;
        assert_eq!(s.unit_los(u), 6);
        assert_eq!(s.seen_sweep(u, false).map(|w| w.radius), Some(3));
    }

    /// §3: a small land unit sees from a half-cell in front of its nose,
    /// not from where it stands. Facing east and facing west put the centre
    /// on opposite sides of the unit's own fog cell.
    #[test]
    fn a_small_land_unit_sees_from_a_half_cell_ahead_of_its_facing() {
        let (mut s, u) = fog_sim(4, 0);
        let own = (fog_of(s.units[u].pos.x), fog_of(s.units[u].pos.y));
        // `Angle` runs clockwise from north through the full `u32`; a
        // quarter turn is east, three quarters west (`docs/MOVEMENT.md`).
        s.units[u].movement.set_facing(Angle(0x4000_0000));
        let east = s.seen_sweep(u, false).unwrap().centre;
        s.units[u].movement.set_facing(Angle(-0x4000_0000));
        let west = s.seen_sweep(u, false).unwrap().centre;
        assert_eq!(east, (own.0 + 1, own.1), "facing east, one fog cell east");
        assert_eq!(west, (own.0 - 1, own.1), "facing west, one fog cell west");
    }

    /// §3: and a unit with radius four or more does not project — its
    /// centre is its own fog cell whichever way it faces.
    #[test]
    fn a_unit_with_radius_four_or_more_sees_from_where_it_stands() {
        let (mut s, u) = fog_sim(8, 0);
        let own = (fog_of(s.units[u].pos.x), fog_of(s.units[u].pos.y));
        s.units[u].movement.set_facing(Angle(0x4000_0000));
        assert_eq!(s.seen_sweep(u, false).unwrap().centre, own);
        s.units[u].movement.set_facing(Angle(-0x4000_0000));
        assert_eq!(s.seen_sweep(u, false).unwrap().centre, own);
    }

    /// §4: the moving pass walks the **ring** table only from radius four
    /// up. Below it — every unit an Ancient game starts with — it is the
    /// whole disc bar its centre, which is why a wrong ring table would not
    /// be caught by any capture on disk.
    #[test]
    fn the_ring_table_is_reached_only_from_radius_four() {
        for (los, want_ring) in [(2, false), (4, false), (6, false), (8, true), (11, true)] {
            let (s, u) = fog_sim(los, 0);
            let sw = s.seen_sweep(u, true).unwrap();
            assert_eq!(sw.ring, want_ring, "LOS {los}");
            if want_ring {
                // The thickened annulus, out of the ring table's own
                // cumulative counts — not the circle's, which are smaller.
                let g = ring();
                let r = sw.radius as usize;
                assert_eq!((sw.start, sw.end), (g.radius[r - 1], g.radius[r]));
                assert_ne!(
                    (sw.start, sw.end),
                    (circle().radius[r - 1], circle().radius[r]),
                    "LOS {los}: the ring table's slice is the circle's"
                );
            } else {
                assert_eq!(sw.start, 1, "LOS {los}: the disc bar its centre");
                assert_eq!(sw.end, circle().radius[sw.radius as usize]);
            }
        }
        // And the standing pass is the whole disc, centre included.
        let (s, u) = fog_sim(8, 0);
        let sw = s.seen_sweep(u, false).unwrap();
        assert!(!sw.ring);
        assert_eq!((sw.start, sw.end), (0, circle().radius[4]));
    }

    /// §5: the disc is written into `seen2`, and `update_seen` answers with
    /// the count of cells newly revealed — the original's `reveal_fog`
    /// calls. A second pass from the same spot reveals nothing.
    #[test]
    fn the_disc_lands_in_the_fog_and_only_new_cells_count() {
        let (mut s, u) = fog_sim(4, 0);
        assert_eq!(seen(&s, 0), 0, "an all-dark grid");
        let first = s.update_seen(u, false);
        assert_eq!(first, circle().radius[2], "the whole disc of radius two");
        assert_eq!(seen(&s, 0), first);
        assert_eq!(s.update_seen(u, false), 0, "nothing new the second time");
    }

    /// §6: the trigger is a **half-cell** crossing, not any movement. A
    /// step inside the unit's own fog cell writes nothing.
    #[test]
    fn only_a_step_that_crosses_a_half_cell_reveals() {
        let (mut s, u) = fog_sim(4, 0);
        s.update_seen(u, false);
        let base = seen(&s, 0);
        let from = s.units[u].pos;
        // Inside the same fog cell: it is `0x180` wide and the unit stands
        // at its middle.
        s.units[u].pos = Pos::new(from.x + 0x20, from.y);
        assert_eq!(
            s.moved_to(u, from, true),
            None,
            "a step inside the half-cell must not reach `update_seen` at all"
        );
        assert_eq!(seen(&s, 0), base);
        let from = s.units[u].pos;
        s.units[u].pos = Pos::new(from.x + 0x180, from.y);
        assert!(
            s.moved_to(u, from, true).is_some_and(|n| n > 0),
            "a step across a half-cell reveals"
        );
        assert!(seen(&s, 0) > base);
    }

    /// §6: and the hundredth-frame resync, which is what covers a unit that
    /// never crosses one. Frame 33, not frame 0 and not frame 100.
    #[test]
    fn the_resync_runs_on_frame_thirty_three_of_each_hundred() {
        for (frame, reveals) in [
            (0, false),
            (32, false),
            (33, true),
            (100, false),
            (133, true),
        ] {
            let (mut s, _u) = fog_sim(4, 0);
            s.frame = frame;
            s.tick();
            assert_eq!(
                seen(&s, 0) > 0,
                reveals,
                "frame {frame}: the resync {}",
                if reveals { "should run" } else { "should not" }
            );
        }
    }

    /// A type with no `LOS` reveals nothing at all — the `mylos == 0` head,
    /// which is what keeps a wall or a projectile out of the fog.
    #[test]
    fn a_type_with_no_los_reveals_nothing() {
        let (mut s, u) = fog_sim(0, 0);
        assert_eq!(s.seen_sweep(u, false), None);
        assert_eq!(s.update_seen(u, false), 0);
        assert_eq!(seen(&s, 0), 0);
    }

    /// §4: `ring_init`'s first rings against `circle_init`'s. Ring `r` is at
    /// least the circle's own ring — it is copied out of it — and the patch
    /// only ever adds.
    #[test]
    fn the_ring_table_contains_the_circles_own_ring_and_thickens_it() {
        let c = circle();
        let g = ring();
        assert_eq!((g.x[0], g.y[0]), (0, 0), "the centre is entry zero");
        assert_eq!(g.radius[0], 1);
        for r in 1..=RING_LAST as usize {
            let circle_ring = c.radius[r] - c.radius[r - 1];
            let ring_ring = g.radius[r] - g.radius[r - 1];
            assert!(
                ring_ring >= circle_ring,
                "ring {r}: {ring_ring} entries against the circle's {circle_ring}"
            );
            // Every entry of the circle's ring is at the front of the ring's.
            for k in 0..circle_ring {
                assert_eq!(
                    (g.x[g.radius[r - 1] + k], g.y[g.radius[r - 1] + k]),
                    (c.x[c.radius[r - 1] + k], c.y[c.radius[r - 1] + k]),
                    "ring {r} entry {k} is the circle's"
                );
            }
            // And the patch lands strictly inside ring `r`.
            for k in circle_ring..ring_ring {
                let i = g.radius[r - 1] + k;
                assert!(
                    vector_dist(g.x[i], g.y[i]) < r as i32,
                    "ring {r}'s patch at {:?} is not inside it",
                    (g.x[i], g.y[i])
                );
            }
        }
    }

    /// The thickening is not decorative: ring 1 is the eight neighbours and
    /// gains nothing, and the first ring that gains is the first with a
    /// diagonal gap.
    #[test]
    fn ring_one_is_the_eight_neighbours_and_the_thickening_starts_later() {
        let c = circle();
        let g = ring();
        assert_eq!(g.radius[1] - g.radius[0], 8, "ring 1 is the neighbourhood");
        assert_eq!(c.radius[1] - c.radius[0], 8);
        let grown: Vec<usize> = (1..=RING_LAST as usize)
            .filter(|&r| (g.radius[r] - g.radius[r - 1]) > (c.radius[r] - c.radius[r - 1]))
            .collect();
        assert!(
            !grown.is_empty(),
            "no ring is thickened, so `ring_init`'s patch never fires"
        );
    }
}
