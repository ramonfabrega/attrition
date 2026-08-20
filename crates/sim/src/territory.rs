//! Territory: which player owns which cell.
//!
//! A weighted-distance Voronoi over cities and forts, recomputed wholesale
//! rather than incrementally, per land region. Sea is never owned.
//!
//! Two things about it are worth stating before the code, because neither is
//! what a fresh implementation would do and both are visible in the shape of
//! the border on screen.
//!
//! **The distance is a deliberately wrong hypotenuse.** See [`raw_distance`].
//!
//! **The near field is contracted three times, compounding.** See
//! [`contract`].
//!
//! Everything is integer arithmetic with defined truncation, so it ports
//! without a rounding decision anywhere.

use crate::tuning::Tuning;
use crate::world::{Cell, Owner, Player, Pos, Terrain, World};

/// What kind of thing is projecting a border.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceKind {
    City,
    Fort,
}

/// One border projector, with its per-player and per-object bonuses already
/// resolved.
///
/// Resolving them here rather than inside the sweep is a departure from the
/// original, which recomputes a per-player bonus table at the top of every
/// region pass and then reaches into it per cell per city. The numbers are the
/// same; see [`PlayerBorders`] for where they come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    /// The player whose list this source was found in. Ties between equal-cost
    /// claims are broken by scan order, which is keyed off this.
    pub owner: Player,
    pub kind: SourceKind,
    pub pos: Pos,
    /// Border bonuses, weighted by the city or fort multiplier in the cost.
    pub bonuses: i32,
    /// Furthest raw distance in tiles this source may reach, before the near
    /// field contraction. A source beyond its limit does not claim at all,
    /// which is not the same as claiming at a cost above the cap.
    pub limit: i32,
    /// Whether the source's own record of its owner agrees with [`Self::owner`].
    /// When it does not, a winning claim writes [`Owner::Ambiguous`].
    ///
    /// The original resolves the two independently: the *limit* comes from the
    /// player whose list the city was found in, while the *bonuses* come from
    /// the owner the city itself records. A caller reproducing that case
    /// builds [`Self::bonuses`] from the recorded owner's [`PlayerBorders`]
    /// and leaves [`Self::owner`] as the list it came from.
    pub owner_agrees: bool,
}

/// The per-player half of a border bonus: everything that depends on the
/// player rather than on the individual city or fort.
///
/// The original recomputes this for all eight players at the start of every
/// region pass, then indexes it by player inside the per-cell loop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayerBorders {
    /// `CIVIC_UPGRADE_TERR` at the player's civic tech level.
    pub civic: i32,
    /// `TEMPLE_UPGRADE_TERR` at the player's temple border level, after Tikal.
    /// Added only for cities that actually have a temple.
    pub temple: i32,
    /// `FORT_UPGRADE_TERR` at the fort border level, plus the Colosseum's and
    /// the Romans' additions.
    pub fort: i32,
    /// Flat additions that apply to everything: gems, the Colosseum, the
    /// Eiffel Tower, Russian borders, and the handicap allowance.
    pub flat: i32,
    /// Distance limit for a city, before its own level and temple add to it.
    pub city_limit: i32,
    /// Distance limit for a fort.
    pub fort_limit: i32,
    /// Temple border tech level, 1 to 4. Contributes to a city's limit.
    pub temple_level: i32,
}

impl PlayerBorders {
    /// Builds the per-player half from tech levels and the handful of wonders
    /// and nation bonuses that matter.
    ///
    /// `civic_level` indexes `CIVIC_UPGRADE_TERR`; `temple_level` and
    /// `fort_level` are 1-based tech steps, as the original computes them from
    /// its `TEMPLEBORDERS2..4` and `FORTBORDERS2..4` prerequisites — level 1
    /// means "none of those researched", not "no temple".
    ///
    /// `handicap_bonus` is the flat allowance an AI handicap adds; the
    /// original derives it as `(handicap + 15) / 25` and, unlike every other
    /// flat bonus, gives no matching extension to the distance limit.
    pub fn new(
        t: &Tuning,
        civic_level: usize,
        temple_level: i32,
        fort_level: i32,
        w: &Wonders,
        n: &NationBonuses,
        handicap_bonus: i32,
    ) -> PlayerBorders {
        let civic = t.civic_upgrade_terr[civic_level.min(t.civic_upgrade_terr.len() - 1)];

        let mut temple =
            t.temple_upgrade_terr[clamp_level(temple_level, t.temple_upgrade_terr.len())];
        if w.tikal {
            temple = (t.tikal_temple_borders + 100) * temple / 100;
        }

        let mut fort = t.fort_upgrade_terr[clamp_level(fort_level, t.fort_upgrade_terr.len())];
        let mut fort_steps = fort_level - 1;
        if w.colosseum {
            fort += t.colosseum_fort_borders;
        }
        if n.roman {
            fort += t.roman_fort_borders;
            // Half the Roman bonus, rounded up, also extends the fort's reach.
            fort_steps += (t.roman_fort_borders + 1) / 2;
        }

        // Every flat bonus buys reach as well as cheapness: a matching number
        // of `TERRITORY_LIMIT_CIVIC` steps goes onto the distance limit, so a
        // bonus never pushes the cost curve past a reach the limit then clips.
        // The Russians take half of each flat bonus and one step for it,
        // because their own per-age bonus is meant to be the one that counts.
        let mut flat = 0;
        let mut steps = 0;
        if n.gems {
            flat += halve_if(t.gems_territory_bonus, n.russian);
            // Gems are the one bonus worth a fixed single step either way.
            steps += 1;
        }
        if w.colosseum {
            flat += halve_if(t.colosseum_territory_bonus, n.russian);
            steps += russian_steps(t.colosseum_territory_bonus, n.russian);
        }
        if w.eiffel_tower {
            flat += halve_if(t.eiffel_tower_territory_bonus, n.russian);
            steps += russian_steps(t.eiffel_tower_territory_bonus, n.russian);
        }
        if n.russian {
            flat += t.russian_borders + t.russian_borders_per_age * n.age;
            steps += i32::from(t.russian_borders != 0) + n.age;
        }
        flat += handicap_bonus;

        let city_limit =
            t.territory_limit_base + (civic_level as i32 + steps) * t.territory_limit_civic;
        PlayerBorders {
            civic,
            temple,
            fort,
            flat,
            city_limit,
            fort_limit: fort_steps * t.territory_limit_city + city_limit,
            temple_level,
        }
    }
}

/// The wonders that touch borders.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Wonders {
    pub colosseum: bool,
    pub tikal: bool,
    pub eiffel_tower: bool,
}

/// The nation bonuses and map resources that touch borders.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NationBonuses {
    pub roman: bool,
    pub russian: bool,
    /// Whether the player holds a gem resource.
    pub gems: bool,
    /// Current age, for the Russian per-age border bonus.
    pub age: i32,
}

/// A city's own contribution, on top of its player's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct City {
    pub pos: Pos,
    /// 0 for a city, 1 for a town, 2 for a metropolis. The Forbidden City
    /// counts as a metropolis.
    pub level: usize,
    /// Whether this city projects the capital bonus in place of its level's.
    /// The Forbidden City confers this.
    pub capital: bool,
    /// Whether the city has a temple, which is what unlocks the temple bonus.
    pub temple: bool,
    /// Whether the city's own record of its owner matches the player whose
    /// list it is in.
    pub owner_agrees: bool,
}

/// A fort's own contribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fort {
    pub pos: Pos,
    /// Whether this is the Red Fort, which adds a flat four to its own bonus.
    /// There is no constant behind that four in the original.
    pub red_fort: bool,
}

/// Turns a city into a [`Source`].
pub fn city_source(t: &Tuning, owner: Player, b: &PlayerBorders, c: &City) -> Source {
    let level_bonus = if c.capital {
        t.capital_territory_bonus
    } else {
        t.city_upgrade_terr[c.level.min(t.city_upgrade_terr.len() - 1)]
    };
    let mut bonuses = b.civic + level_bonus + b.flat;
    let mut limit = b.city_limit + c.level as i32 * t.territory_limit_city;
    if c.temple {
        bonuses += b.temple;
        limit += b.temple_level * t.territory_limit_city;
    }
    Source {
        owner,
        kind: SourceKind::City,
        pos: c.pos,
        bonuses,
        limit,
        owner_agrees: c.owner_agrees,
    }
}

/// Turns a fort into a [`Source`].
pub fn fort_source(t: &Tuning, owner: Player, b: &PlayerBorders, f: &Fort) -> Source {
    let mut bonuses = b.fort + b.civic;
    if f.red_fort {
        bonuses += t.red_fort_borders;
    }
    Source {
        owner,
        kind: SourceKind::Fort,
        pos: f.pos,
        bonuses: bonuses + b.flat,
        limit: b.fort_limit,
        // A fort is only ever found under its own owner.
        owner_agrees: true,
    }
}

fn clamp_level(level: i32, len: usize) -> usize {
    (level.max(1) as usize - 1).min(len - 1)
}

const fn halve_if(bonus: i32, yes: bool) -> i32 {
    if yes { bonus / 2 } else { bonus }
}

/// Limit steps a flat bonus buys: half the bonus rounded up, or one step flat
/// for the Russians.
const fn russian_steps(bonus: i32, russian: bool) -> i32 {
    if russian { 1 } else { (bonus + 1) / 2 }
}

/// The original's integer hypotenuse, in tiles, before the near field
/// contraction.
///
/// With `hi` the larger of `|dx|` and `|dy|` and `lo` the smaller, this is
/// `hi + lo² / (2·hi)` — a first-order approximation that is exact on the
/// axes, worst on the diagonal, and never a square root. It is what makes Rise
/// of Nations' borders read as faintly octagonal rather than circular;
/// substituting a true hypotenuse would visibly change every border.
///
/// The original has a second branch, `hi + lo / 2`, guarded by `lo < 60000`.
/// That is an overflow guard on `lo * lo`, not a shape decision: no map is
/// sixty thousand tiles across, so the branch is unreachable in play. It is
/// kept because it costs nothing and because leaving it out would quietly
/// change behaviour at a size the original defined.
pub const fn raw_distance(dx: i32, dy: i32) -> i32 {
    let (dx, dy) = (dx.abs(), dy.abs());
    let (hi, lo) = if dx >= dy { (dx, dy) } else { (dy, dx) };
    if hi == 0 {
        return 0;
    }
    if lo < 60_000 {
        hi + (lo * lo) / (hi * 2)
    } else {
        hi + lo / 2
    }
}

/// The near field contraction: three tests applied in order to the running
/// distance.
///
/// Each test reads the value the previous one wrote, so they stack. Four tiles
/// becomes two, then one, then zero — a city's immediate surroundings cost
/// nothing at all, which is why borders bulge close in and taper further out.
/// Written as three separate `if`s rather than a match precisely because the
/// stacking is the point.
pub const fn contract(mut d: i32) -> i32 {
    if d < 13 {
        d = d * 2 / 3;
    }
    if d < 9 {
        d = d * 2 / 3;
    }
    if d < 5 {
        d /= 2;
    }
    d
}

/// The cost one source charges to reach a contracted distance.
///
/// ```text
/// cost = TERRITORY_DEN * distance * 256 / (TERRITORY_NUM + multiplier * bonuses)
/// ```
///
/// The 256 is an 8.8 fixed-point scale, matched by the `TERRITORY_BASE << 8`
/// cap. This is the reciprocal of the designers' own annotation on
/// `TERRITORY_NUM`, which describes a radius; the code works in cost per unit
/// distance, so a larger bonus makes distance cheaper and the border reach
/// further.
///
/// Returns `None` if the bonuses drive the denominator to zero or below, which
/// the shipped data never does — a negative bonus pool would otherwise invert
/// the whole border.
pub const fn cost(t: &Tuning, kind: SourceKind, distance: i32, bonuses: i32) -> Option<i32> {
    let multiplier = match kind {
        SourceKind::City => t.city_territory_multiplier,
        SourceKind::Fort => t.fort_territory_multiplier,
    };
    let den = bonuses * multiplier + t.territory_num;
    if den <= 0 {
        return None;
    }
    Some(t.territory_den * distance * 256 / den)
}

/// The highest cost a claim may have and still own the cell.
pub const fn cost_cap(t: &Tuning) -> i32 {
    t.territory_base << 8
}

/// Recomputes ownership for every land region, and clears every sea region.
///
/// Wholesale, like the original: there is no incremental path, because a
/// single captured city changes ownership arbitrarily far away.
pub fn compute_all_territory(world: &mut World, t: &Tuning, sources: &[Source], players: u8) {
    let regions: Vec<(u16, Terrain)> = world.regions().collect();
    for (region, terrain) in regions {
        match terrain {
            Terrain::Land => compute_region_territory(world, t, sources, players, region),
            // Sea is never owned. The original writes 0xFF into both claimant
            // bytes of every sea cell after the land pass; the effect is that
            // a coastal border stops at the water rather than reaching over it.
            Terrain::Sea => {
                let cells: Vec<Cell> = world.cells_in(region).collect();
                for c in cells {
                    world.set_owner(c, Owner::None, Owner::None);
                }
            }
        }
    }
}

/// Recomputes ownership for one land region.
///
/// Equal-cost claims go to whichever source is reached first, so the order of
/// `sources` within a player is observable. List a player's cities before its
/// forts to match the original.
pub fn compute_region_territory(
    world: &mut World,
    t: &Tuning,
    sources: &[Source],
    players: u8,
    region: u16,
) {
    let cap = cost_cap(t);
    let cells: Vec<Cell> = world.cells_in(region).collect();
    for cell in cells {
        let here = cell.centre_tile();
        let mut best: Option<(i32, Owner)> = None;
        let mut second: Option<(i32, Owner)> = None;

        // The player scan order is rotated by the cell's x coordinate. Ties go
        // to whoever is reached first, so without the rotation a contested
        // frontier would belong entirely to the lower-numbered player; with
        // it, equal claims interleave along x. The original writes this as
        // `(i + x) & 7` against its fixed eight-player array. Generalising the
        // modulus is the only change, and it preserves the intent exactly.
        for step in 0..players {
            let p = ((step as i32 + cell.x).rem_euclid(players as i32)) as Player;
            for s in sources.iter().filter(|s| s.owner == p) {
                let d = raw_distance(here.x - s.pos.tile().x, here.y - s.pos.tile().y);
                // The limit is tested against the raw distance, before the
                // contraction. Testing the contracted one would let a source
                // reach measurably further.
                if d > s.limit {
                    continue;
                }
                let Some(c) = cost(t, s.kind, contract(d), s.bonuses) else {
                    continue;
                };
                let claimant = if s.owner_agrees {
                    Owner::Player(s.owner)
                } else {
                    Owner::Ambiguous
                };

                let beaten = best.is_some_and(|(bc, _)| c >= bc);
                if c > cap || beaten {
                    // Too expensive to own the cell, or already beaten — but
                    // still eligible to be recorded as the runner-up. A cell
                    // can therefore name a second claimant it would never
                    // grant to anyone.
                    if second.is_none_or(|(sc, _)| c < sc) {
                        second = Some((c, claimant));
                    }
                } else {
                    if best.is_some() {
                        second = best;
                    }
                    best = Some((c, claimant));
                }
            }
        }

        world.set_owner(
            cell,
            best.map_or(Owner::None, |(_, o)| o),
            second.map_or(Owner::None, |(_, o)| o),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::UNITS_PER_CELL;

    const T: Tuning = Tuning::RON;

    fn plain_player() -> PlayerBorders {
        PlayerBorders::new(
            &T,
            0,
            1,
            1,
            &Wonders::default(),
            &NationBonuses::default(),
            0,
        )
    }

    fn plain_city(pos: Pos) -> City {
        City {
            pos,
            level: 0,
            capital: false,
            temple: false,
            owner_agrees: true,
        }
    }

    #[test]
    fn the_hypotenuse_is_exact_on_the_axes() {
        for n in 0..200 {
            assert_eq!(raw_distance(n, 0), n);
            assert_eq!(raw_distance(0, -n), n);
        }
    }

    #[test]
    fn the_hypotenuse_overshoots_on_the_diagonal() {
        // 3-4-5 comes out exact by luck: 4 + 9/8 = 5.
        assert_eq!(raw_distance(3, 4), 5);
        // The pure diagonal is where the approximation is worst: it returns
        // 1.5x the leg where the true answer is 1.414x, so borders bulge
        // *inward* at the corners and the outline reads as an octagon.
        assert_eq!(raw_distance(40, 40), 60);
        assert_eq!(raw_distance(100, 100), 150);
    }

    #[test]
    fn the_hypotenuse_is_symmetric_and_off_by_at_most_a_truncation() {
        for dx in -60..60 {
            for dy in -60..60 {
                let d = raw_distance(dx, dy);
                assert_eq!(d, raw_distance(dy, dx), "{dx},{dy}");
                assert_eq!(d, raw_distance(-dx, dy), "{dx},{dy}");
                // Never shorter than the longer leg.
                assert!(d >= dx.abs().max(dy.abs()), "{dx},{dy} gave {d}");
                // The exact first-order form is an upper bound on the true
                // distance, but the division truncates, so the result can land
                // one below — 60,30 is the first case, giving 67 where the
                // true distance is 67.08. It is never worse than that.
                let (d, true_sq) = (
                    i64::from(d),
                    i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy),
                );
                assert!((d + 1) * (d + 1) >= true_sq, "{dx},{dy} gave {d}");
            }
        }
    }

    #[test]
    fn the_near_field_contraction_stacks() {
        // The headline case: three tests, each reading the last one's output.
        // 4 -> 2 -> 1 -> 0. The near field around a city costs nothing at all.
        assert_eq!(contract(4), 0);
        // 12 -> 8 -> 5, and the third test does not fire because the second
        // landed exactly on its boundary.
        assert_eq!(contract(12), 5);
        // 8 -> 5 -> 3 -> 1. Eight tiles out is charged as one, which is why a
        // border bulges so hard close in.
        assert_eq!(contract(8), 1);
        // At and above thirteen, nothing happens at all, and the step from
        // twelve to thirteen is therefore a jump from 5 to 13 — the border
        // gradient is discontinuous, by construction.
        assert_eq!(contract(13), 13);
        assert_eq!(contract(44), 44);
    }

    #[test]
    fn contraction_is_monotonic() {
        // Not obvious given three compounding truncating divisions, and
        // load-bearing: a non-monotonic metric would make a border with holes.
        let mut prev = contract(0);
        for d in 1..100 {
            let c = contract(d);
            assert!(
                c >= prev,
                "contract({d}) = {c} < contract({}) = {prev}",
                d - 1
            );
            prev = c;
        }
    }

    #[test]
    fn a_plain_city_reaches_the_distance_the_constants_predict() {
        // Cost cap is TERRITORY_BASE << 8 = 6144. With no bonuses the
        // denominator is TERRITORY_NUM = 11, so the furthest affordable
        // contracted distance solves 5 * d * 256 / 11 <= 6144, giving d <= 52.
        let b = plain_player();
        let s = city_source(&T, 0, &b, &plain_city(Pos::new(0, 0)));
        assert_eq!(s.bonuses, 0);
        assert!(cost(&T, SourceKind::City, 52, 0).unwrap() <= cost_cap(&T));
        assert!(cost(&T, SourceKind::City, 53, 0).unwrap() > cost_cap(&T));
        // But the limit clips it long before that: 44 tiles, which is 11
        // cells. The cost curve is not what bounds a plain city's border.
        assert_eq!(s.limit, T.territory_limit_base);
    }

    #[test]
    fn bonuses_buy_reach() {
        let b = plain_player();
        let plain = city_source(&T, 0, &b, &plain_city(Pos::new(0, 0)));
        let metropolis = city_source(
            &T,
            0,
            &b,
            &City {
                level: 2,
                ..plain_city(Pos::new(0, 0))
            },
        );
        assert!(metropolis.bonuses > plain.bonuses);
        assert!(metropolis.limit > plain.limit);
        // A cheaper cost per tile at the same distance is the whole mechanism.
        let d = 30;
        assert!(
            cost(&T, SourceKind::City, d, metropolis.bonuses)
                < cost(&T, SourceKind::City, d, plain.bonuses)
        );
    }

    /// A one-region world with a single city at the centre of cell `(cx, cy)`.
    fn one_city_world(n: i32, at: Cell) -> (World, Vec<Source>) {
        let mut w = World::new(n, n);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(n - 1, n - 1));
        let b = plain_player();
        let pos = Pos::new(
            at.x * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            at.y * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        );
        let sources = vec![city_source(&T, 0, &b, &plain_city(pos))];
        (w, sources)
    }

    #[test]
    fn one_city_owns_a_blob_around_itself() {
        let (mut w, s) = one_city_world(24, Cell::new(12, 12));
        compute_all_territory(&mut w, &T, &s, 1);
        assert_eq!(w.owner(Cell::new(12, 12)), Owner::Player(0));
        // 44 tiles of limit is 11 cells, and the limit binds well before the
        // cost cap does, so the border stops exactly there and not a cell on.
        assert_eq!(w.owner(Cell::new(1, 12)), Owner::Player(0));
        assert_eq!(w.owner(Cell::new(0, 12)), Owner::None);
        // The claim is contiguous along the ray out of the city.
        for x in 1..=23 {
            assert!(
                w.owner(Cell::new(x, 12)).is_claimed(),
                "cell {x},12 unclaimed"
            );
        }
        // And it is a blob, not a square: the diagonal falls short of the
        // axis, because the metric overshoots there.
        assert_eq!(w.owner(Cell::new(1, 1)), Owner::None);
    }

    #[test]
    fn the_border_is_not_a_circle() {
        // The metric overshoots on the diagonal, so the corner of the world is
        // further away than the axis at the same Chebyshev distance. Measure
        // it directly rather than trusting the outline by eye.
        let axis = raw_distance(40, 0);
        let diagonal = raw_distance(40, 40);
        assert_eq!(axis, 40);
        assert_eq!(diagonal, 60);
        // A true hypotenuse would be 56, so the original's border is pulled in
        // by four tiles at 45 degrees relative to a circle.
        assert!(diagonal > 56);
    }

    #[test]
    fn sea_is_never_owned() {
        let mut w = World::new(8, 4);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(3, 3));
        w.fill_region(Terrain::Sea, Cell::new(4, 0), Cell::new(7, 3));
        let b = plain_player();
        let pos = Pos::new(UNITS_PER_CELL / 2, UNITS_PER_CELL / 2);
        let s = vec![city_source(&T, 0, &b, &plain_city(pos))];
        compute_all_territory(&mut w, &T, &s, 1);
        assert_eq!(w.owner(Cell::new(3, 3)), Owner::Player(0));
        for y in 0..4 {
            for x in 4..8 {
                assert_eq!(w.owner(Cell::new(x, y)), Owner::None, "sea at {x},{y}");
            }
        }
    }

    #[test]
    fn the_nearer_city_wins_and_the_other_is_recorded_second() {
        let mut w = World::new(20, 1);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(19, 0));
        let b = plain_player();
        let at = |cx: i32| Pos::new(cx * UNITS_PER_CELL + UNITS_PER_CELL / 2, UNITS_PER_CELL / 2);
        let s = vec![
            city_source(&T, 0, &b, &plain_city(at(2))),
            city_source(&T, 1, &b, &plain_city(at(17))),
        ];
        compute_all_territory(&mut w, &T, &s, 2);
        assert_eq!(w.owner(Cell::new(2, 0)), Owner::Player(0));
        assert_eq!(w.owner(Cell::new(17, 0)), Owner::Player(1));
        assert_eq!(w.owner(Cell::new(0, 0)), Owner::Player(0));
        assert_eq!(w.owner(Cell::new(19, 0)), Owner::Player(1));
        // Where both reach, the loser is the runner-up rather than nothing.
        assert_eq!(w.owner(Cell::new(9, 0)), Owner::Player(0));
        assert_eq!(w.second(Cell::new(9, 0)), Owner::Player(1));
        // Where only one reaches, there is no runner-up at all.
        assert_eq!(w.second(Cell::new(2, 0)), Owner::None);
    }

    #[test]
    fn equal_claims_interleave_along_x() {
        // Two identical cities the same distance from a column of cells. The
        // scan-order rotation is what stops player 0 taking the lot.
        let mut w = World::new(9, 3);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(8, 2));
        let b = plain_player();
        let at = |cx: i32, cy: i32| {
            Pos::new(
                cx * UNITS_PER_CELL + UNITS_PER_CELL / 2,
                cy * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            )
        };
        let s = vec![
            city_source(&T, 0, &b, &plain_city(at(4, 0))),
            city_source(&T, 1, &b, &plain_city(at(4, 2))),
        ];
        compute_all_territory(&mut w, &T, &s, 2);
        // Along the equidistant middle row every claim ties, so ownership
        // alternates with x rather than going wholly to player 0.
        let row: Vec<Owner> = (0..9).map(|x| w.owner(Cell::new(x, 1))).collect();
        assert!(row.contains(&Owner::Player(0)), "{row:?}");
        assert!(row.contains(&Owner::Player(1)), "{row:?}");
        for (x, o) in row.iter().enumerate() {
            // At even x the scan starts at player 0, so player 0 takes the
            // tie; at odd x it starts at player 1.
            assert_eq!(*o, Owner::Player(u8::from(x % 2 != 0)), "cell {x},1");
        }
    }

    #[test]
    fn a_source_beyond_its_limit_does_not_claim_at_all() {
        // Put a city far enough away that the limit clips it, and check the
        // cell is unowned rather than owned at a high cost.
        let mut w = World::new(40, 1);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 0));
        let b = plain_player();
        let s = vec![city_source(
            &T,
            0,
            &b,
            &plain_city(Pos::new(UNITS_PER_CELL / 2, UNITS_PER_CELL / 2)),
        )];
        compute_all_territory(&mut w, &T, &s, 1);
        // The limit is 44 tiles; cell 11 is 44 tiles away and cell 12 is 48.
        assert_eq!(w.owner(Cell::new(11, 0)), Owner::Player(0));
        assert_eq!(w.owner(Cell::new(12, 0)), Owner::None);
        assert_eq!(w.second(Cell::new(12, 0)), Owner::None);
    }

    #[test]
    fn a_disowned_city_claims_ambiguously() {
        let mut w = World::new(4, 1);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(3, 0));
        let b = plain_player();
        let s = vec![city_source(
            &T,
            0,
            &b,
            &City {
                owner_agrees: false,
                ..plain_city(Pos::new(UNITS_PER_CELL / 2, UNITS_PER_CELL / 2))
            },
        )];
        compute_all_territory(&mut w, &T, &s, 1);
        assert_eq!(w.owner(Cell::new(0, 0)), Owner::Ambiguous);
        assert!(w.owner(Cell::new(0, 0)).is_claimed());
        assert_eq!(w.owner(Cell::new(0, 0)).player(), None);
    }
}
