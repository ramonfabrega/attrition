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

/// How often a unit's periodic work runs at all. `Unit::process` gates a whole
/// block of per-unit upkeep on this.
pub const UNIT_UPKEEP_FRAMES: i64 = 16;

/// How often a unit's attrition period is recomputed.
///
/// Not every frame. This is the single most consequential fact about the
/// cadence: a unit that walks out of hostile territory keeps its stale period
/// until the next refresh, and one that walks in takes nothing until then.
pub const ATTRITION_REFRESH_FRAMES: i64 = 32;

/// A unit, as attrition sees one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unit {
    pub owner: Player,
    /// The unit's index in its owner's object list — `SubObjectData::o`.
    ///
    /// Every periodic thing a unit does is phased by this, so that a hundred
    /// units do not all do their upkeep on the same frame. It is part of the
    /// simulation rather than an optimisation: it decides *which* frames a
    /// given unit bleeds on.
    pub index: i16,
    pub pos: Pos,
    pub health: i32,
    /// Figures in the squad, one to four. Damage per tick is set by this and
    /// nothing else.
    pub squad_size: i32,
    pub kind: attrition::UnitKind,
    /// Whether the unit is on the map, rather than garrisoned in a building or
    /// riding in a transport. Off the map, none of this runs.
    pub on_map: bool,
    /// Whether the unit is inside a friendly supply radius. Supply is not
    /// implemented yet; this stands in for `Supplies::find_supply`.
    pub in_supply: bool,
    /// The period the last refresh wrote, in frames. Zero means not bleeding.
    /// Public because it is observable state, not a private counter — the
    /// original keeps it in `UnitData::attrition` and the interface shows it.
    pub attrition: i32,
    /// Whether the current bleed goes through supply.
    pub ignores_supply: bool,
    /// Whether supply sheltered this unit from a tick that was otherwise due.
    /// The original tracks the same thing in a display flag.
    pub sheltered: bool,
}

impl Unit {
    pub fn new(owner: Player, index: i16, pos: Pos, health: i32) -> Unit {
        Unit {
            owner,
            index,
            pos,
            health,
            squad_size: 1,
            kind: attrition::UnitKind::default(),
            on_map: true,
            in_supply: false,
            attrition: 0,
            ignores_supply: false,
            sheltered: false,
        }
    }

    pub fn alive(&self) -> bool {
        self.health > 0
    }

    /// The frame counter this unit's periodic work is phased against.
    pub const fn phase(&self, frame: i64) -> i64 {
        frame + self.index as i64
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
    /// The cadence is the part of this mechanic that would be easiest to get
    /// plausibly wrong, and it is not a countdown. `Unit::process` recomputes
    /// the period every 32 frames and, on **every** frame, applies damage when
    /// `(frame + unit index) % period == 0`. Two things follow that a
    /// countdown would not give:
    ///
    /// - **The damage is phase-locked to the global frame**, not to when the
    ///   unit entered hostile ground. Change a period from 48 to 24 and the
    ///   unit re-locks to the 24-frame grid immediately, mid-interval.
    /// - **Leaving hostile territory does not stop the bleeding at once.** The
    ///   stale period survives until the next refresh, so a unit can take one
    ///   more tick up to 32 frames after walking out — and, symmetrically,
    ///   takes nothing for up to 32 frames after walking in.
    pub fn tick(&mut self) -> Vec<Tick> {
        let frame = self.frame;
        let mut events = Vec::new();
        for i in 0..self.units.len() {
            if !self.units[i].alive() || !self.units[i].on_map {
                continue;
            }
            let phase = self.units[i].phase(frame);

            // The refresh. `process_attrition` clears the period on entry, so
            // an exempt unit comes out of it with nothing pending.
            if phase % ATTRITION_REFRESH_FRAMES == 0 {
                let outcome = self.attrition_for(i);
                let unit = &mut self.units[i];
                unit.sheltered = false;
                match outcome {
                    attrition::Outcome::Exempt(_) => {
                        unit.attrition = 0;
                        unit.ignores_supply = false;
                    }
                    attrition::Outcome::Period {
                        frames,
                        ignores_supply,
                    } => {
                        unit.attrition = frames;
                        unit.ignores_supply = ignores_supply;
                    }
                }
            }

            let unit = &mut self.units[i];
            if unit.attrition == 0 || phase % i64::from(unit.attrition) != 0 {
                continue;
            }
            // Supply gets first refusal. A unit inside a friendly supply
            // radius takes no attrition at all — which is what supply is for,
            // and the whole reason an army can campaign abroad.
            if unit.in_supply && !unit.ignores_supply && unit.kind.shelterable() {
                unit.sheltered = true;
            } else {
                let d = attrition::damage(unit.squad_size);
                unit.health -= d;
                events.push(Tick {
                    unit: i,
                    frame,
                    damage: d,
                    killed: !unit.alive(),
                });
            }
        }
        self.frame += 1;
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
#[path = "harness_tests.rs"]
mod tests;
