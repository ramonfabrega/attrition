//! The simulation: headless, deterministic, and dependent on no engine.
//!
//! This crate knows nothing about pixels, windows, input devices, clocks or
//! threads. Everything in it runs in a `#[test]` with no display attached,
//! which is the property that makes determinism testable at all.
//!
//! Five mechanics run here, and they run together. Borders produce territory,
//! territory produces damage, supply cancels it, units walk in and out of it
//! under orders, and the ground they hold pays its owner. Each has a
//! specification written from the original — `docs/ATTRITION.md`,
//! `docs/SUPPLY.md`, `docs/MOVEMENT.md`, `docs/ECONOMY.md` — and each says how
//! much of itself is established rather than guessed.
//!
//! [`Sim::tick`] is where they meet, and the order it does them in is the
//! original's, twice over. `Game::do_frame` pays every player before it
//! processes any object, so income comes first. And inside a unit, attrition
//! comes before movement, so a unit stepping over a border is not standing
//! there when that frame's attrition looks.
//!
//! # Arithmetic
//!
//! No floating point, ever — and so far, no fixed point either. Every quantity
//! the original computes in these mechanics is an integer with a defined
//! truncation. The one value it keeps in an `f32` is exactly a rational and is
//! carried as one; the one table it builds with doubles is built once at
//! startup, so its integers are pinned rather than recomputed. `Fx` stays
//! unearned until something genuinely needs a fraction; anticipating it would
//! only add rounding the original does not have.

pub mod attrition;
pub mod economy;
pub mod movement;
pub mod supply;
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
    /// Which slot of its owner's supply list this unit registered as, if its
    /// type is a supply source. The original caches the same index in the
    /// unit and gives it back when the unit dies.
    pub supply_slot: Option<usize>,
    /// The period the last refresh wrote, in frames. Zero means not bleeding.
    /// Public because it is observable state, not a private counter — the
    /// original keeps it in `UnitData::attrition` and the interface shows it.
    pub attrition: i32,
    /// Whether the current bleed goes through supply.
    pub ignores_supply: bool,
    /// Whether supply sheltered this unit from a tick that was otherwise due.
    /// The original tracks the same thing in a display flag.
    pub sheltered: bool,
    /// Where it is going and how fast it gets there.
    pub movement: Movement,
}

/// What a unit needs in order to move.
///
/// `speed` and `turn_rate` are **inputs**, not computed here. They are the
/// output of `UnitData::get_speed`'s three-layer pipeline, which reads tech,
/// nation, hero and terrain state the simulation does not model yet. Taking
/// them as inputs is the same choice supply made for a general's aura radius:
/// the mechanic is complete, and the number feeding it arrives when the layer
/// that produces it does. `docs/MOVEMENT.md` documents the pipeline in full.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Movement {
    /// Which way the unit faces. North is zero.
    pub facing: movement::Angle,
    /// Where it is headed. `None` means it is not going anywhere.
    pub dest: Option<Pos>,
    /// Effective speed, in position units per frame.
    pub speed: i32,
    /// How far it can turn in one frame, in binary angle units.
    pub turn_rate: i32,
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
            supply_slot: None,
            attrition: 0,
            ignores_supply: false,
            sheltered: false,
            movement: Movement::default(),
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
    /// One supply network per player. Never shared: an ally's supply wagon
    /// does nothing for your units, which is a rule of the original and not a
    /// simplification here.
    pub supply: Vec<supply::Network>,
    /// Diplomacy, as a full matrix. `at_war[a][b]` is symmetric in practice
    /// but stored both ways, because the original reads it both ways.
    pub at_war: Vec<Vec<bool>>,
    /// One income ledger per player, and what feeds it. Split the way the
    /// original splits them: `holdings` is what a player *has*, and changes
    /// when they build something; `ledgers` is what that is worth and what has
    /// accrued, and changes every frame.
    pub holdings: Vec<economy::Holdings>,
    pub ledgers: Vec<economy::Ledger>,
    pub frame: i64,
}

impl Sim {
    pub fn new(tuning: Tuning, world: World, players: usize) -> Sim {
        Sim {
            players: vec![attrition::PlayerState::default(); players],
            sources: Vec::new(),
            units: Vec::new(),
            supply: vec![supply::Network::default(); players],
            at_war: vec![vec![false; players]; players],
            holdings: vec![economy::Holdings::new(); players],
            ledgers: vec![economy::Ledger::starting(&tuning); players],
            tuning,
            world,
            frame: 0,
        }
    }

    /// Adds a player and returns their index.
    ///
    /// Five vectors are kept in step by this. Growing one of them by hand
    /// leaves the others short, and the failure shows up as an index panic in
    /// whichever pass reaches the longest one first.
    pub fn add_player(&mut self) -> Player {
        let who = self.players.len();
        self.players.push(attrition::PlayerState::default());
        self.supply.push(supply::Network::default());
        self.holdings.push(economy::Holdings::new());
        self.ledgers.push(economy::Ledger::starting(&self.tuning));
        for row in &mut self.at_war {
            row.push(false);
        }
        self.at_war.push(vec![false; who + 1]);
        u8::try_from(who).expect("too many players")
    }

    /// Marks a player's economy as changed, so the next reassembly happens
    /// within eight frames rather than at the lazy 512-frame cadence.
    ///
    /// The original sets this flag from wherever the change happened — a
    /// building finished, a city captured. Anything that edits
    /// [`Sim::holdings`] should say so here, or the change will not show up
    /// for up to half a minute.
    pub fn economy_changed(&mut self, who: Player) {
        self.ledgers[who as usize].dirty = true;
    }

    /// Adds a unit, registering it as a supply source if its type is one.
    ///
    /// This is `Unit::init`'s half of the supply bookkeeping and the reason to
    /// prefer it over pushing onto `units` directly: a supply wagon that never
    /// registered supplies nobody, silently.
    pub fn add_unit(&mut self, unit: Unit) -> usize {
        let i = self.units.len();
        let owner = unit.owner as usize;
        let source = unit.kind.supply_unit;
        self.units.push(unit);
        if source {
            self.units[i].supply_slot = Some(self.supply[owner].list.register(i));
        }
        i
    }

    /// Gives a dead unit's supply slot back — `Unit::close`.
    fn close_supply(&mut self, unit: usize) {
        let owner = self.units[unit].owner as usize;
        if let Some(slot) = self.units[unit].supply_slot.take() {
            self.supply[owner].list.close(slot);
        }
    }

    /// Whether anything of `owner`'s supplies a unit standing at `at`.
    ///
    /// A source's liveness is resolved here rather than stored, exactly as the
    /// original resolves it: the supply record holds an index and nothing
    /// else, so a wagon that dies or boards a transport stops supplying with
    /// no bookkeeping anywhere.
    pub fn supplied_at(&self, owner: Player, at: Pos) -> bool {
        self.supply[owner as usize].supplies(&self.tuning, at, |u| {
            self.units.get(u).map(|w| supply::Wagon {
                pos: w.pos,
                active: w.alive(),
                on_map: w.on_map,
            })
        })
    }

    /// Sets two players at war with each other.
    pub fn declare_war(&mut self, a: Player, b: Player) {
        self.at_war[a as usize][b as usize] = true;
        self.at_war[b as usize][a as usize] = true;
    }

    /// Copies the territory the border pass produced into the holdings the
    /// territory tax reads, and tells the world how much land there is.
    ///
    /// The original keeps both numbers itself — `LeaderData::territory` and
    /// `WorldData::land_size` — and the tax is their ratio. **The unit
    /// cancels**, so counting cells here where the original counts tiles gives
    /// the same wealth; what would not survive is counting one of them in one
    /// unit and the other in the other.
    ///
    /// Marks every player's economy dirty, since this is exactly the kind of
    /// change the original's flag exists for.
    pub fn update_territory_holdings(&mut self) {
        let mut land = 0;
        let mut owned = vec![0; self.players.len()];
        for (region, terrain) in self.world.regions().collect::<Vec<_>>() {
            if terrain != Terrain::Land {
                continue;
            }
            for cell in self.world.cells_in(region) {
                land += 1;
                if let Some(p) = self.world.owner(cell).player() {
                    owned[p as usize] += 1;
                }
            }
        }
        for (who, holdings) in self.holdings.iter_mut().enumerate() {
            holdings.territory = owned[who];
            holdings.land_size = land;
        }
        for l in &mut self.ledgers {
            l.dirty = true;
        }
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

        // Income first. `Game::do_frame` runs `Leaders::process_all` before
        // `Objects::process_all`, so every player is paid for the frame before
        // any unit in it moves, fights or bleeds. That ordering is observable:
        // a citizen that dies this frame was already paid for it.
        for who in 0..self.players.len() {
            let player = u8::try_from(who).expect("too many players");
            economy::process(
                &self.tuning,
                &mut self.ledgers[who],
                &self.holdings[who],
                player,
                frame,
            );
        }

        for i in 0..self.units.len() {
            if !self.units[i].alive() || !self.units[i].on_map {
                continue;
            }
            // Attrition first, movement second. That is the order inside
            // `Unit::process`, and it is observable: a unit that steps over a
            // border this frame is not standing there when this frame's
            // attrition looks, so it cannot bleed for the crossing until the
            // next one — and, because the period is only refreshed every 32
            // frames, usually not for a good while after that.
            if let Some(tick) = self.process_attrition(i, frame) {
                events.push(tick);
            }
            self.process_movement(i);
        }
        self.frame += 1;
        events
    }

    /// One unit's attrition for one frame — the refresh, the supply veto, and
    /// the damage.
    fn process_attrition(&mut self, i: usize, frame: i64) -> Option<Tick> {
        let phase = self.units[i].phase(frame);

        // The refresh. `process_attrition` clears the period on entry, so an
        // exempt unit comes out of it with nothing pending.
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

        let unit = &self.units[i];
        if unit.attrition == 0 || phase % i64::from(unit.attrition) != 0 {
            return None;
        }
        // Supply gets first refusal — `Unit::process_supply`. A unit inside a
        // friendly supply radius takes no attrition at all, which is the whole
        // reason an army can campaign abroad. The two refusals before the
        // search are the interesting ones: a peace or assassin bleed gives up
        // immediately, and militia and supply units are never sheltered.
        let sheltered = !unit.ignores_supply
            && unit.kind.shelterable()
            && self.supplied_at(unit.owner, unit.pos);

        let unit = &mut self.units[i];
        if sheltered {
            unit.sheltered = true;
            return None;
        }
        let d = attrition::damage(unit.squad_size);
        unit.health -= d;
        let killed = !unit.alive();
        if killed {
            self.close_supply(i);
        }
        Some(Tick {
            unit: i,
            frame,
            damage: d,
            killed,
        })
    }

    /// One unit's movement for one frame — the original's `Guy::move`, which
    /// `Unit::process` reaches last.
    ///
    /// A step the world refuses is not an error and does not cancel the order:
    /// the unit stays put and tries again next frame. It still turns, because
    /// the original turns before it asks.
    fn process_movement(&mut self, i: usize) {
        let unit = &self.units[i];
        let Some(dest) = unit.movement.dest else {
            return;
        };
        let m = unit.movement;
        let step = movement::advance(unit.pos, m.facing, dest, m.speed, m.turn_rate);
        let accepted = self.world.accepts(step.pos);

        let unit = &mut self.units[i];
        unit.movement.facing = step.facing;
        if !accepted {
            return;
        }
        unit.pos = step.pos;
        if step.arrived {
            unit.movement.dest = None;
        }
    }

    /// Sends a unit somewhere. It faces whatever way it already faces and turns
    /// as it goes.
    pub fn order_move(&mut self, unit: usize, dest: Pos) {
        self.units[unit].movement.dest = Some(dest);
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
