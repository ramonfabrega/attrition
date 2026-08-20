//! The simulation: headless, deterministic, and dependent on no engine.
//!
//! This crate knows nothing about pixels, windows, input devices, clocks or
//! threads. Everything in it runs in a `#[test]` with no display attached,
//! which is the property that makes determinism testable at all.
//!
//! Phase 1 is one mechanic end to end: **attrition**. Borders produce
//! territory, territory produces damage. See `docs/ATTRITION.md` for the
//! specification this implements and how much of it is established rather than
//! guessed.
//!
//! # Arithmetic
//!
//! No floating point, ever. Attrition turns out to need no fixed point either:
//! every quantity the original computes here is an integer with a defined
//! truncation, and the one value it keeps in an `f32` is exactly a rational,
//! carried as one. `Fx` will be earned when movement arrives; anticipating it
//! would only add rounding the original does not have.

pub mod attrition;
pub mod territory;
pub mod tuning;
pub mod world;

pub use tuning::Tuning;
pub use world::{Cell, Owner, Player, Pos, Terrain, World};

/// Frames per second. The original's whole clock, and the unit every attrition
/// period is quoted in.
pub const FRAMES_PER_SECOND: i32 = 15;

/// A unit, as attrition sees one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unit {
    pub owner: Player,
    pub pos: Pos,
    pub health: i32,
    /// Figures in the squad, one to four. Damage per tick is set by this and
    /// nothing else.
    pub squad_size: i32,
    pub kind: attrition::UnitKind,
    /// Frames until the next attrition tick, or zero for "not currently
    /// bleeding". The original stores this on the unit as an `i16` and
    /// recomputes it every tick, so a unit that walks out of hostile territory
    /// stops immediately rather than finishing its countdown.
    pub countdown: i32,
}

impl Unit {
    pub fn new(owner: Player, pos: Pos, health: i32) -> Unit {
        Unit {
            owner,
            pos,
            health,
            squad_size: 1,
            kind: attrition::UnitKind::default(),
            countdown: 0,
        }
    }

    pub fn alive(&self) -> bool {
        self.health > 0
    }
}

/// A headless world with players and units, enough to run attrition end to
/// end and no more.
///
/// This is a harness, not an architecture. It exists so that the mechanic can
/// be exercised as the original exercises it — per unit, per tick, against a
/// recomputed territory map — rather than only through its parts.
#[derive(Clone, Debug)]
pub struct Sim {
    pub tuning: Tuning,
    pub world: World,
    pub players: Vec<attrition::PlayerState>,
    pub sources: Vec<territory::Source>,
    pub units: Vec<Unit>,
    /// Diplomacy, as a full matrix. `at_war[a][b]` is symmetric in practice
    /// but stored both ways, because the original reads it both ways.
    pub at_war: Vec<Vec<bool>>,
    pub frame: i64,
}

impl Sim {
    pub fn new(tuning: Tuning, world: World, players: usize) -> Sim {
        Sim {
            tuning,
            world,
            players: vec![attrition::PlayerState::default(); players],
            sources: Vec::new(),
            units: Vec::new(),
            at_war: vec![vec![false; players]; players],
            frame: 0,
        }
    }

    /// Sets two players at war with each other.
    pub fn declare_war(&mut self, a: Player, b: Player) {
        self.at_war[a as usize][b as usize] = true;
        self.at_war[b as usize][a as usize] = true;
    }

    /// Recomputes every border from scratch.
    ///
    /// Wholesale, like the original: one captured city can change ownership
    /// arbitrarily far away, so there is no incremental path worth having.
    pub fn recompute_territory(&mut self) {
        let players = u8::try_from(self.players.len()).expect("too many players");
        territory::compute_all_territory(&mut self.world, &self.tuning, &self.sources, players);
    }

    /// Advances one frame.
    ///
    /// Every unit's period is recomputed each tick and its countdown is
    /// clamped to it, so leaving hostile ground stops the bleeding at once and
    /// walking into worse territory speeds it up immediately. The countdown
    /// itself is where this implementation goes beyond what has been read out
    /// of the original: `process_attrition` sets the period and
    /// `suffer_attrition` applies the damage, but the code that counts between
    /// them has not been found. This is the straightforward reading, and
    /// `docs/ATTRITION.md` records it as an assumption rather than a finding.
    pub fn tick(&mut self) -> Vec<Tick> {
        self.frame += 1;
        let mut events = Vec::new();
        for i in 0..self.units.len() {
            if !self.units[i].alive() {
                continue;
            }
            let outcome = self.attrition_for(i);
            let unit = &mut self.units[i];
            match outcome {
                attrition::Outcome::Exempt(_) => unit.countdown = 0,
                attrition::Outcome::Period(p) => {
                    // Clamp rather than reset, so that a unit already part way
                    // through a long period is not made to start again when it
                    // steps into faster attrition.
                    unit.countdown = if unit.countdown == 0 {
                        p
                    } else {
                        unit.countdown.min(p)
                    };
                    unit.countdown -= 1;
                    if unit.countdown == 0 {
                        let d = attrition::damage(unit.squad_size);
                        unit.health -= d;
                        events.push(Tick {
                            unit: i,
                            frame: self.frame,
                            damage: d,
                            killed: !unit.alive(),
                        });
                    }
                }
            }
        }
        events
    }

    fn attrition_for(&self, i: usize) -> attrition::Outcome {
        let unit = &self.units[i];
        let ground = self.world.owner_at(unit.pos);
        let victim = &self.players[unit.owner as usize];
        let at_war = ground
            .player()
            .is_none_or(|o| self.at_war[unit.owner as usize][o as usize]);
        let situation = attrition::Situation {
            ground,
            in_free_zone: false,
            mutual_treaty: false,
            at_war,
            in_war_grace: false,
            assassin: false,
            conquest_exempt: false,
        };
        attrition::process(
            &self.tuning,
            &unit.kind,
            unit.owner,
            victim,
            &situation,
            |p| self.players[p as usize],
        )
    }
}

/// One unit taking one tick of attrition damage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tick {
    pub unit: usize,
    pub frame: i64,
    pub damage: i32,
    pub killed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attrition::{PlayerState, Resistance, StrengthMods, strength};
    use crate::territory::{City, NationBonuses, PlayerBorders, Wonders, city_source};
    use crate::world::UNITS_PER_CELL;

    fn centre_of(c: Cell) -> Pos {
        Pos::new(
            c.x * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            c.y * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        )
    }

    /// Two players on a strip of land. Player 1 holds a city at one end and
    /// enough attrition tech to make its border bite; player 0 has a unit.
    fn skirmish(tech_steps: usize) -> Sim {
        let t = Tuning::RON;
        let mut world = World::new(24, 1);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(23, 0));

        let mut sim = Sim::new(t, world, 2);
        let borders = PlayerBorders::new(
            &t,
            0,
            1,
            1,
            &Wonders::default(),
            &NationBonuses::default(),
            0,
        );
        sim.sources.push(city_source(
            &t,
            1,
            &borders,
            &City {
                pos: centre_of(Cell::new(20, 0)),
                level: 0,
                capital: false,
                temple: false,
                owner_agrees: true,
            },
        ));
        sim.players[1] = PlayerState {
            strength: strength(&t, tech_steps, &StrengthMods::default()),
            team: 1,
            ..PlayerState::default()
        };
        sim.players[0].team = 0;
        sim.declare_war(0, 1);
        sim.recompute_territory();
        sim
    }

    #[test]
    fn a_unit_outside_the_border_never_bleeds() {
        let mut sim = skirmish(1);
        // Cell 8 is 48 tiles from the city, past its 44-tile limit.
        sim.units
            .push(Unit::new(0, centre_of(Cell::new(8, 0)), 100));
        assert_eq!(sim.world.owner_at(sim.units[0].pos), Owner::None);
        for _ in 0..600 {
            assert!(sim.tick().is_empty());
        }
        assert_eq!(sim.units[0].health, 100);
    }

    #[test]
    fn a_unit_inside_a_hostile_border_bleeds_on_schedule() {
        let mut sim = skirmish(1);
        sim.units
            .push(Unit::new(0, centre_of(Cell::new(15, 0)), 100));
        assert_eq!(sim.world.owner_at(sim.units[0].pos), Owner::Player(1));

        // One attrition tech, a plain unit: 48 frames a tick, 16 damage a
        // tick, so 3.2 seconds and a sixth of a full-health squaddie's life.
        let mut ticks = Vec::new();
        for _ in 0..(48 * 3) {
            ticks.extend(sim.tick());
        }
        assert_eq!(ticks.len(), 3);
        assert_eq!(
            ticks.iter().map(|t| t.frame).collect::<Vec<_>>(),
            vec![48, 96, 144]
        );
        assert!(ticks.iter().all(|t| t.damage == 16));
        assert_eq!(sim.units[0].health, 100 - 48);
        assert_eq!(48 / FRAMES_PER_SECOND, 3);
    }

    #[test]
    fn attrition_kills() {
        let mut sim = skirmish(4);
        sim.units
            .push(Unit::new(0, centre_of(Cell::new(15, 0)), 100));
        // Four tech steps is strength 8, so six frames a tick: a lone figure
        // dies in seven ticks, inside three seconds.
        let mut killed_at = None;
        for _ in 0..600 {
            for t in sim.tick() {
                if t.killed {
                    killed_at = Some(t.frame);
                }
            }
        }
        assert_eq!(killed_at, Some(42));
        assert!(!sim.units[0].alive());
    }

    #[test]
    fn a_bigger_squad_bleeds_slower_per_figure() {
        let mut sim = skirmish(1);
        let mut four = Unit::new(0, centre_of(Cell::new(15, 0)), 100);
        four.squad_size = 4;
        sim.units.push(four);
        let mut total = 0;
        for _ in 0..(48 * 4) {
            total += sim.tick().iter().map(|t| t.damage).sum::<i32>();
        }
        // Four ticks at 4 damage rather than four at 16.
        assert_eq!(total, 16);
    }

    #[test]
    fn walking_out_of_the_border_stops_the_bleeding_at_once() {
        let mut sim = skirmish(1);
        sim.units
            .push(Unit::new(0, centre_of(Cell::new(15, 0)), 100));
        for _ in 0..40 {
            sim.tick();
        }
        assert!(sim.units[0].countdown > 0);
        // Step outside. The period is recomputed every tick, so the countdown
        // is discarded rather than run down.
        sim.units[0].pos = centre_of(Cell::new(8, 0));
        for _ in 0..600 {
            assert!(sim.tick().is_empty());
        }
        assert_eq!(sim.units[0].health, 100);
        assert_eq!(sim.units[0].countdown, 0);
    }

    #[test]
    fn a_players_own_territory_is_safe() {
        let mut sim = skirmish(1);
        // Give the unit to the player who owns the border.
        let mut u = Unit::new(1, centre_of(Cell::new(15, 0)), 100);
        u.kind = attrition::UnitKind::default();
        sim.units.push(u);
        for _ in 0..600 {
            assert!(sim.tick().is_empty());
        }
    }

    #[test]
    fn the_statue_of_liberty_walks_through_anything() {
        let mut sim = skirmish(4);
        sim.players[0].resistance = Resistance::Immune;
        sim.units
            .push(Unit::new(0, centre_of(Cell::new(15, 0)), 100));
        for _ in 0..600 {
            assert!(sim.tick().is_empty());
        }
        assert_eq!(sim.units[0].health, 100);
    }

    #[test]
    fn the_whole_run_is_deterministic() {
        // The property the entire project rests on. Two independent runs of
        // the same setup must agree frame for frame, not merely in the end
        // state.
        let run = || {
            let mut sim = skirmish(3);
            sim.units
                .push(Unit::new(0, centre_of(Cell::new(15, 0)), 100));
            sim.units
                .push(Unit::new(0, centre_of(Cell::new(21, 0)), 100));
            let mut log = Vec::new();
            for _ in 0..500 {
                log.extend(sim.tick());
            }
            (log, sim.units.clone(), sim.world.clone())
        };
        let (log_a, units_a, _) = run();
        let (log_b, units_b, _) = run();
        assert_eq!(log_a, log_b);
        assert_eq!(units_a, units_b);
        assert!(!log_a.is_empty());
    }
}
