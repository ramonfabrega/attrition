//! Territory: which player owns which cell.
//!
//! A weighted-distance Voronoi over cities and forts, per land region. Sea is
//! never owned. This crate recomputes it wholesale; the original does that at
//! setup and then incrementally in play, 256 cells a frame — see
//! [`compute_all_territory`].
//!
//! Two things about it are worth stating before the code, because neither is
//! what a fresh implementation would do and both are visible in the shape of
//! the border on screen.
//!
//! **The distance is a deliberately wrong hypotenuse.** See [`crate::world::vector_dist`].
//!
//! **The near field is contracted three times, compounding.** See
//! [`contract`].
//!
//! Everything is integer arithmetic with defined truncation, so it ports
//! without a rounding decision anywhere.

use crate::tuning::Tuning;
use crate::world::{Cell, Owner, Player, Pos, Terrain, World, vector_dist};

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
    /// The steps [`Self::limit`] is made of, `[civic, city]`: the limit is
    /// `base + civic × civic_step + city × city_step` over a territory-limit
    /// triple, and `compute_reg_territory:114–123` picks the triple per
    /// region — the rules' `TERRITORY_LIMIT_*` unless the style overrides
    /// them ([`crate::world::World::region_territory_limits`], item 3963).
    /// [`Self::limit`] is this over the rules' triple.
    pub steps: [i32; 2],
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
    /// The Civic level plus the flat bonuses' limit steps:
    /// `compute_reg_territory`'s multiplier of the triple's civic step in
    /// [`Self::city_limit`] (`local_18`, lines 198–248).
    pub civic_steps: i32,
    /// The fort border level less one: the multiplier of the triple's city
    /// step in [`Self::fort_limit`] (`aiStack_130 × local_50`, line 259).
    pub fort_steps: i32,
}

impl PlayerBorders {
    /// The borders of a player with no civic, temple or fort tech, no wonders
    /// and no nation bonus — what a city projects on the first frame.
    pub fn plain(t: &Tuning) -> PlayerBorders {
        PlayerBorders::new(
            t,
            0,
            1,
            1,
            &Wonders::default(),
            &NationBonuses::default(),
            0,
        )
    }

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
            flat += t.russian_borders + t.russian_borders_per_age * n.civic;
            steps += i32::from(t.russian_borders != 0) + n.civic;
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
            civic_steps: civic_level as i32 + steps,
            fort_steps,
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
    /// The **Civic library level**, which is what the Russians' per-step
    /// border bonus multiplies — not the age. `compute_reg_territory` reads
    /// `data_encrypted->epoch[1] ^ 0x63187` twice in the same loop body,
    /// once for [`PlayerBorders::civic`] and once here, and they are the
    /// same expression on the same field (`+0xec`, which
    /// `LeaderData::get_city_limit` names `epoch[1]`).
    pub civic: i32,
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
    let mut city_steps = c.level as i32;
    if c.temple {
        bonuses += b.temple;
        limit += b.temple_level * t.territory_limit_city;
        city_steps += b.temple_level;
    }
    Source {
        owner,
        kind: SourceKind::City,
        pos: c.pos,
        bonuses,
        limit,
        steps: [b.civic_steps, city_steps],
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
        steps: [b.civic_steps, b.fort_steps],
        // A fort is only ever found under its own owner.
        owner_agrees: true,
    }
}

impl Source {
    /// [`Self::limit`] over another territory-limit triple `[base, civic,
    /// city]` — the style's, for the region `compute_reg_territory` is on.
    pub const fn limit_in(&self, l: [i32; 3]) -> i32 {
        l[0] + self.steps[0] * l[1] + self.steps[1] * l[2]
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
/// Wholesale. That is what the original's `World::compute_all_territory` does
/// at game setup, with its cell budget set to unlimited — and **not** what it
/// does in play, where `GameDaemon::check_borders` re-runs only invalidated
/// regions on a shared budget of 256 cells a frame, resuming each region where
/// it left off, so a cell can carry stale ownership for up to
/// `land cells / 256` frames after a city changes hands. Steady-state
/// ownership is identical either way; the transient is a deliberate
/// simplification here until a recorded-game diff says it matters.
pub fn compute_all_territory(world: &mut World, t: &Tuning, sources: &[Source], players: u8) {
    compute_all_territory_won(world, t, sources, players);
}

/// [`compute_all_territory`], and the index into `sources` of the **city**
/// that won each cell (row-major, `−1` for a cell no city won — a fort's,
/// an unowned one's). `compute_reg_territory` keeps it as `local_1c`: the
/// city whose `bordering` the cell's runner-up marks
/// (`docs/AI.md` §167).
pub fn compute_all_territory_won(
    world: &mut World,
    t: &Tuning,
    sources: &[Source],
    players: u8,
) -> Vec<i32> {
    let mut won = vec![-1; (world.width() * world.height()) as usize];
    let regions: Vec<(u16, Terrain)> = world.regions().collect();
    for (region, terrain) in regions {
        match terrain {
            Terrain::Land => {
                compute_region_territory_won(world, t, sources, players, region, &mut won)
            }
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
    won
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
    let mut won = vec![-1; (world.width() * world.height()) as usize];
    compute_region_territory_won(world, t, sources, players, region, &mut won);
}

fn compute_region_territory_won(
    world: &mut World,
    t: &Tuning,
    sources: &[Source],
    players: u8,
    region: u16,
    won: &mut [i32],
) {
    let cap = cost_cap(t);
    // `compute_reg_territory:114–123`: the region's triple, the style's
    // player limits on a player region and its colonized ones elsewhere.
    let limits = world.region_territory_limits(region);
    let cells: Vec<Cell> = world.cells_in(region).collect();
    for cell in cells {
        let here = cell.centre_tile();
        let mut best: Option<(i32, Owner)> = None;
        let mut best_city: i32 = -1;
        let mut second: Option<(i32, Owner)> = None;

        // The player scan order is rotated by the cell's x coordinate. Ties go
        // to whoever is reached first, so without the rotation a contested
        // frontier would belong entirely to the lower-numbered player; with
        // it, equal claims interleave along x. **The rotation is over the
        // eight leader slots, not the players in the game**:
        // `compute_reg_territory@006b0bb0:338` is `(local_20 + x) &
        // 0x80000007` for `local_20` 0..7, and a slot whose `leaders[p] & 1`
        // is clear claims nothing (`:344`). With two players a tie at x = 13
        // starts the scan at slot 5 and reaches player 0 before player 1;
        // the generalised `% players` started it at player 1 (item 3595,
        // `docs/AI.md` §318.2: East Indies' cells (13, 15) and (15, 13)).
        for step in 0..8 {
            let p = ((step + cell.x).rem_euclid(8)) as Player;
            if p >= players {
                continue;
            }
            for (si, s) in sources.iter().enumerate().filter(|(_, s)| s.owner == p) {
                let d = vector_dist(here.x - s.pos.tile().x, here.y - s.pos.tile().y);
                // The limit is tested against the raw distance, before the
                // contraction. Testing the contracted one would let a source
                // reach measurably further.
                if d > limits.map_or(s.limit, |l| s.limit_in(l)) {
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
                    // The two source paths demote the old best differently,
                    // and the difference looks like an oversight rather than
                    // a rule: the city path keeps an existing runner-up when
                    // there was no best to demote, the fort path overwrites
                    // it with "nobody". So a fort that is the first in-cap
                    // claim after an over-cap city was recorded as runner-up
                    // erases that runner-up. It touches `who2` only, never
                    // `who`, and it is one line to match.
                    match s.kind {
                        SourceKind::City => {
                            if best.is_some() {
                                second = best;
                            }
                        }
                        SourceKind::Fort => second = best,
                    }
                    best = Some((c, claimant));
                    // `local_1c`: a city's win names it, a fort's clears it.
                    best_city = if s.kind == SourceKind::City {
                        si as i32
                    } else {
                        -1
                    };
                }
            }
        }

        world.set_owner(
            cell,
            best.map_or(Owner::None, |(_, o)| o),
            second.map_or(Owner::None, |(_, o)| o),
        );
        if let Some(w) = won.get_mut((cell.y * world.width() + cell.x) as usize) {
            *w = best_city;
        }
    }
}

/// `WorldData::is_enemy_territory@006b2490`: the cell's owner (the signed byte `+0xf` of its record) is a player,
/// is not `who`, and is not an ally of `who`'s leader — `LeaderData::is_ally` inlined on `leaders[who]`, whose own
/// `who` field is `me`: the owner is `me`, or both `diplos` entries are 2 (`diplo(of, toward)`, `who`'s toward the
/// owner first, the owner's toward `me` only when that one is 2). Item 2112's sweep holds the order of the reads.
pub fn enemy_territory_by(
    owner: i32,
    who: i32,
    me: i32,
    mut diplo: impl FnMut(i32, i32) -> i32,
) -> bool {
    if owner < 0 || owner == who || owner == me {
        return false;
    }
    !(diplo(who, owner) == 2 && diplo(owner, me) == 2)
}

#[cfg(test)]
mod tests;

// The files of `territory/` (item 4011, `docs/STRUCTURE.md` §3).
pub mod attrition;
pub mod border_pass;
pub mod suffer_view;
pub mod supply;
pub mod upkeep;
pub mod victory;
