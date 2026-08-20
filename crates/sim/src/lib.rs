//! The simulation: headless, deterministic, and dependent on no engine.
//!
//! This crate knows nothing about pixels, windows, input devices, clocks or
//! threads. Everything in it runs in a `#[test]` with no display attached,
//! which is the property that makes determinism testable at all.
//!
//! Seven mechanics run here, and they run together. Borders produce territory,
//! territory produces damage, supply cancels it, units walk in and out of it
//! under orders, the ground they hold pays its owner, that income buys the next
//! unit at a price that climbs with every one already built — and the unit
//! takes time to arrive. Each has a specification written from the original —
//! `docs/ATTRITION.md`, `docs/SUPPLY.md`, `docs/MOVEMENT.md`,
//! `docs/ECONOMY.md`, `docs/COSTS.md`, `docs/PRODUCTION.md` — and each says how
//! much of itself is established rather than guessed.
//!
//! [`Sim::tick`] is where they meet, and the order it does them in is the
//! original's, three times over. `Game::do_frame` pays every player before it
//! processes any object, so income comes first. Buildings and units are both
//! objects and interleave by index in `Objects::process_all`; here the
//! buildings go first, which is what the original does whenever the building
//! predates the unit it just made. And inside a unit, attrition comes before
//! movement, so a unit stepping over a border is not standing there when that
//! frame's attrition looks.
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
pub mod cost;
pub mod economy;
pub mod movement;
pub mod production;
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

/// A unit, as attrition sees one — which is **one figure**, not one squad.
///
/// Rise of Nations units are squads of one to four figures, and each figure is
/// its own `UnitData` in the owner's unit list with its own health and its own
/// cadence; the squad is what the player selects, the figure is what bleeds.
/// This struct is the figure. [`Unit::squad_size`] is how many figures its
/// squad currently has, which is what sets its share of the damage.
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
    /// Whole hit points left.
    pub health: i32,
    /// Sixteenths of a hit point taken but not yet carried into
    /// [`Unit::health`] — `ObjectData::damage_frac`. Attrition deals its damage
    /// in sixteenths, and nothing is lost to truncation between ticks.
    pub damage_frac: i32,
    /// Figures currently in this figure's squad, one to four — the original's
    /// `curr_uber_size`. Damage per tick is set by this and nothing else.
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

/// A unit type the simulation knows how to build.
///
/// The price is the data's, the kind is what attrition and supply read, and
/// the group is which production building's output the ramp counts against —
/// `None` for a type whose progression counts by type, which is every civilian
/// in the shipped data.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UnitType {
    pub price: cost::Price,
    pub kind: attrition::UnitKind,
    pub group: Option<usize>,
    /// The `HITS` column: what a unit of this type is built with.
    pub hits: i32,
    /// The time half of the same record — `JOB_TIME`, `JOB_EXTRA_TIME` and
    /// `RESEARCH_PREMIUM_TIME`. See `docs/PRODUCTION.md`.
    pub times: production::Times,
}

/// What a player has built, as the price and the population cap see it.
///
/// The original keeps three per-type arrays on the leader and this holds the
/// two that production reads: `num_units` at `+0x5762`, which is what exists,
/// and `num_queued` at `+0x5a22`, which is what has been ordered. The
/// distinction is not cosmetic — **the price ramp counts both and the time
/// ramp counts only the first**, so ordering five hoplites at once raises the
/// price of each but not the time of any. See `docs/PRODUCTION.md`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Muster {
    /// Units of each type, indexed by [`Sim::unit_types`]. `num_units`.
    pub by_type: Vec<i32>,
    /// Units of each type ordered and not yet delivered. `num_queued`.
    pub queued_by_type: Vec<i32>,
    /// Whether each type has been researched — the availability bit at
    /// `leader + 0x6c18`. It decides two things at once and must decide them
    /// together: the *first* of a unit type is a research job, at research
    /// pace and research time, and every one after is a train job.
    pub researched: Vec<bool>,
    /// How many of the player's cities hold a library —
    /// `LeaderData::get_building_cities`. This is how many slots of the
    /// **first library's** queue advance at once — and of no other queue —
    /// and since every research job is forwarded to that library, it is how
    /// many technologies can progress simultaneously.
    pub library_cities: usize,
    /// The same, by production group.
    pub by_group: Vec<i32>,
    /// Population occupied — `LeaderData::control`.
    pub control: i32,
    /// The cap it is measured against. Recomputed from scratch by
    /// [`Sim::recompute_pop_caps`] rather than maintained incrementally,
    /// because `Leader::calc_pop_cap` recomputes it from scratch too.
    pub cap: i32,
    /// The player's age, indexing `POP_CAP`. An input until there is a tech
    /// layer to produce it.
    pub age: usize,
    /// The lobby's population setting. The shipped choices are 50, 75, 100,
    /// 125, 150 and 200, and the largest is exactly `POP_CAP[7]`.
    pub limit: i32,
    /// The wonders, nations and scenario overrides that move the cap.
    pub bonuses: cost::PopBonuses,
}

/// The largest population the lobby offers, and this simulation's default.
pub const DEFAULT_POP_LIMIT: i32 = 200;

impl Muster {
    fn new(t: &Tuning) -> Muster {
        let mut m = Muster {
            limit: DEFAULT_POP_LIMIT,
            ..Muster::default()
        };
        m.cap = cost::pop_cap(t, m.age, m.limit, &m.bonuses);
        m
    }
}

/// Why a unit was not built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The population cap would be exceeded — `check_population`.
    Population,
    /// Something in the price could not be paid — `Leader::can_pay`.
    Cost,
}

impl Unit {
    pub fn new(owner: Player, index: i16, pos: Pos, health: i32) -> Unit {
        Unit {
            owner,
            index,
            pos,
            health,
            damage_frac: 0,
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
    /// The unit types this simulation can build. A type's index is its id, the
    /// way the original's tables are index-keyed; see `docs/DECISIONS.md`
    /// entry 9.
    pub unit_types: Vec<UnitType>,
    /// One per player: what they have built, and the population it occupies.
    pub muster: Vec<Muster>,
    /// Where an unavailable resource's price is charged instead.
    pub redirects: cost::Redirects,
    /// The production buildings, each with its own queue.
    pub buildings: Vec<Building>,
    pub frame: i64,
}

/// A building with a production queue.
///
/// The simulation does not model buildings as objects yet — they take no
/// damage, hold no garrison and occupy no ground. This is the production half
/// of one, which is what the queue needs and all it needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Building {
    pub owner: Player,
    /// Where a finished unit appears. `Build::train` places it at the
    /// building's own position and then puts it inside, so every unit is born
    /// garrisoned and leaves by the ejection path — which this does not model.
    pub pos: Pos,
    pub queue: production::Queue,
    /// Whether this is a library — the original's `is(LIBRARY, 0)`. Two rules
    /// key on it and on nothing else: orders and cancels given to any library
    /// are forwarded to the player's *first* one, and only that first
    /// library's queue fans out across `Muster::library_cities` slots. There
    /// is no building-type system here yet; this flag is the smallest honest
    /// stand-in for the one test production needs.
    pub is_library: bool,
}

/// A unit that came out of a queue this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Produced {
    /// Index into [`Sim::units`].
    pub unit: usize,
    pub ty: usize,
    /// Index into [`Sim::buildings`].
    pub at: usize,
}

/// What one call of `do_queue` on one slot did — the sign `Build::finished`
/// hands back, plus "not done yet".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Advanced {
    /// The counter moved and the entry is not done.
    Pending,
    /// A research entry completed: the bit is set, the entry is gone, no unit.
    Researched,
    /// A train entry completed and a unit was placed.
    Trained(Produced),
    /// Done, offered, refused — zero or negative from `finished`. The entry
    /// stays at full progress. At slot 0 this is what triggers the redirect.
    Blocked,
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
            unit_types: Vec::new(),
            muster: vec![Muster::new(&tuning); players],
            redirects: cost::Redirects::RON,
            buildings: Vec::new(),
            tuning,
            world,
            frame: 0,
        }
    }

    /// Adds a player and returns their index.
    ///
    /// Six vectors are kept in step by this. Growing one of them by hand
    /// leaves the others short, and the failure shows up as an index panic in
    /// whichever pass reaches the longest one first.
    pub fn add_player(&mut self) -> Player {
        let who = self.players.len();
        self.players.push(attrition::PlayerState::default());
        self.supply.push(supply::Network::default());
        self.holdings.push(economy::Holdings::new());
        self.ledgers.push(economy::Ledger::starting(&self.tuning));
        self.muster.push(Muster::new(&self.tuning));
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

    /// Registers a unit type and returns its id.
    ///
    /// Every player's muster grows with it, because a type nobody has built is
    /// still a type whose count the ramp reads — as zero, which is what makes
    /// the first one cheap.
    pub fn add_unit_type(&mut self, ty: UnitType) -> usize {
        let id = self.unit_types.len();
        let groups = ty.group.map_or(0, |g| g + 1);
        self.unit_types.push(ty);
        for m in &mut self.muster {
            m.by_type.push(0);
            m.queued_by_type.push(0);
            m.researched.push(false);
            if m.by_group.len() < groups {
                m.by_group.resize(groups, 0);
            }
        }
        id
    }

    /// Recomputes every player's population cap from scratch —
    /// `Leader::calc_pop_cap`, which the original also calls wholesale
    /// whenever anything that feeds it changes.
    pub fn recompute_pop_caps(&mut self) {
        for m in &mut self.muster {
            m.cap = cost::pop_cap(&self.tuning, m.age, m.limit, &m.bonuses);
        }
    }

    /// What a player would be charged for one of a type, per resource.
    ///
    /// The discount tail is `docs/COSTS.md`'s forty predicates and arrives as
    /// [`cost::Modifiers`] when the nation, wonder and government layers exist
    /// to produce it. Until then it is the undiscounted price, which is
    /// exactly what a stock game with no bonuses charges.
    pub fn price_of(&self, who: Player, ty: usize) -> [i32; economy::RESOURCES] {
        self.price_with(who, ty, &cost::Modifiers::default())
    }

    /// [`Sim::price_of`], with the discount tail supplied.
    pub fn price_with(
        &self,
        who: Player,
        ty: usize,
        m: &cost::Modifiers,
    ) -> [i32; economy::RESOURCES] {
        let muster = &self.muster[who as usize];
        let holdings = &self.holdings[who as usize];
        let unit = &self.unit_types[ty];
        // Built *and* ordered. `LeaderData::get_support_count` sums
        // `num_units` and `num_queued`, so five hoplites ordered at once each
        // pay a ramp step for the ones ahead of them in the queue. The time
        // ramp reads only the first of the two arrays; see
        // `docs/PRODUCTION.md`.
        let counts = cost::Counts {
            of_type: muster.by_type[ty] + muster.queued_by_type[ty],
            of_group: unit.group.map_or(0, |g| muster.by_group[g]),
        };
        cost::charges(
            &self.tuning,
            &unit.price,
            counts,
            m,
            &holdings.available,
            &holdings.discovered,
            &self.redirects,
        )
    }

    /// Adds a production building and returns its index.
    pub fn add_building(&mut self, owner: Player, pos: Pos, capacity: usize) -> usize {
        self.buildings.push(Building {
            owner,
            pos,
            queue: production::Queue::new(capacity),
            is_library: false,
        });
        self.buildings.len() - 1
    }

    /// Adds a library and returns its index. See [`Building::is_library`].
    pub fn add_library(&mut self, owner: Player, pos: Pos, capacity: usize) -> usize {
        let at = self.add_building(owner, pos, capacity);
        self.buildings[at].is_library = true;
        at
    }

    /// The player's first library — `LeaderData::get_first_library`, which
    /// walks the owner's buildings from the lowest index and returns the first
    /// library. (The original also asks that it be active, in a city and not
    /// unassimilated; none of those states exist here.)
    pub fn first_library(&self, who: Player) -> Option<usize> {
        self.buildings
            .iter()
            .position(|b| b.owner == who && b.is_library)
    }

    /// Where an order given at `at` actually lands. A library that is not the
    /// player's first forwards to the first — `queue_up`, `unqueue` and
    /// `do_queue` all perform this redirection, which is what puts every
    /// research job in the game on one queue.
    fn queue_home(&self, at: usize) -> usize {
        if !self.buildings[at].is_library {
            return at;
        }
        self.first_library(self.buildings[at].owner).unwrap_or(at)
    }

    /// Orders one unit of a type at a building — `Build::queue_up`.
    ///
    /// **The price is charged here**, and what was charged is written into the
    /// queue entry, which is what makes a later cancellation exact. Note what
    /// is *not* checked: the population cap. A player at the cap may queue
    /// freely, and finds out at the far end; see [`Sim::process_queues`].
    ///
    /// The two gates are the original's and in its order — affordability
    /// first, then room in the queue.
    pub fn queue_up(&mut self, at: usize, ty: usize) -> Result<usize, production::QueueFail> {
        let at = self.queue_home(at);
        let who = self.buildings[at].owner;
        let charges = self.price_of(who, ty);
        let available = self.holdings[who as usize].available;
        if !cost::can_pay(&charges, &self.ledgers[who as usize], &available, 1) {
            return Err(production::QueueFail::Cost);
        }
        if !self.buildings[at].queue.has_room() {
            return Err(production::QueueFail::Full);
        }
        cost::pay(&charges, &mut self.ledgers[who as usize], &available, false);
        let slot = self.buildings[at].queue.push(ty, &charges);
        self.muster[who as usize].queued_by_type[ty] += 1;
        self.economy_changed(who);
        Ok(slot)
    }

    /// Cancels a queued order — `Build::unqueue` with a refund.
    ///
    /// The slot removed is not necessarily the one named: a cancel walks
    /// forward past entries of the same type, so cancelling one of a run
    /// removes the last of it and the one in progress keeps its progress.
    pub fn cancel(&mut self, at: usize, slot: usize) -> Option<production::Item> {
        let at = self.queue_home(at);
        let who = self.buildings[at].owner;
        let item = {
            let ledger = &mut self.ledgers[who as usize];
            self.buildings[at].queue.unqueue(slot, true, ledger)?
        };
        self.muster[who as usize].queued_by_type[item.ty] -= 1;
        self.economy_changed(who);
        Some(item)
    }

    /// How long the item in `slot` at building `at` takes, right now.
    ///
    /// Recomputed every frame rather than stored, because the original
    /// recomputes it every frame: a unit of the same type completing
    /// elsewhere lengthens this one mid-build.
    pub fn queue_target(&self, at: usize, slot: usize) -> i32 {
        let b = &self.buildings[at];
        let item = &b.queue.items[slot];
        let muster = &self.muster[b.owner as usize];
        production::train_time(
            &self.tuning,
            &self.unit_types[item.ty].times,
            muster.researched[item.ty],
            muster.by_type[item.ty],
            &[],
        )
    }

    /// Advances every building's queue by one frame — `Build::do_queue`.
    ///
    /// Three things about the cadence are worth stating because they would be
    /// easy to get plausibly wrong, and one of them was. It runs on **every**
    /// frame, with no phase offset by building index — unlike the per-unit
    /// upkeep in `docs/ATTRITION.md`, which is phased. **Only the first
    /// library fans out**: its queue advances `Muster::library_cities` slots at
    /// once (`LeaderData::get_building_cities`, inside `do_queue`'s library
    /// branch), a non-first library's queue never advances at all, and every
    /// other building advances slot 0 and slot 0 only. (An earlier draft
    /// applied the fan-out to every building.) And **a stuck head is not a
    /// stuck queue**: when slot 0 is done and refused, the first research
    /// entry behind it advances in its place — see [`Sim::advance_slot`].
    fn process_queues(&mut self) -> Vec<Produced> {
        let mut out = Vec::new();
        for at in 0..self.buildings.len() {
            let who = self.buildings[at].owner;
            let queued = self.buildings[at].queue.items.len();
            if queued == 0 {
                continue;
            }
            if self.buildings[at].is_library {
                if self.first_library(who) != Some(at) {
                    // `do_queue`'s first gate: a non-first library returns.
                    continue;
                }
                // The library branch recurses into `i + 1` before handling
                // its own slot, so deeper slots complete first. Walking the
                // slots in reverse is that order, and it also keeps the lower
                // indices stable when a deeper entry is removed.
                let slots =
                    production::parallel_slots(self.muster[who as usize].library_cities, queued)
                        .max(1);
                for slot in (0..slots).rev() {
                    if let Advanced::Trained(p) = self.advance_slot(at, slot) {
                        out.push(p);
                    }
                }
                continue;
            }
            match self.advance_slot(at, 0) {
                Advanced::Trained(p) => out.push(p),
                Advanced::Blocked => {
                    // The head is done and refused. `do_queue` then asks
                    // `get_next_non_unit` for the first research entry
                    // behind it and advances that one this frame instead. A
                    // train entry behind the head stays put: the cap blocks
                    // every train job, and only train jobs.
                    //
                    // The original's second fallback — on a *negative*
                    // answer only, a non-caravan or helicopter entry — is not
                    // modelled, because nothing here can be refused with a
                    // negative answer yet (no caravans, no aircraft).
                    let researched = &self.muster[who as usize].researched;
                    let next = self.buildings[at].queue.next_research(|ty| !researched[ty]);
                    if let Some(slot) = next
                        && let Advanced::Trained(p) = self.advance_slot(at, slot)
                    {
                        out.push(p);
                    }
                }
                Advanced::Pending | Advanced::Researched => {}
            }
        }
        out
    }

    /// One queue slot, for one frame.
    ///
    /// A slot that is done is offered to [`production::finished`]. A research
    /// entry — a unit type whose availability bit is clear — completes through
    /// `Leader::gain_tech`: the bit is set, the entry is removed with no
    /// refund, and **no unit is placed**; the player queues again for the
    /// first trained one. (An earlier draft spawned a unit here too.) A train
    /// entry that is refused keeps its full progress and is retried next
    /// frame, which is where a population cap actually bites.
    fn advance_slot(&mut self, at: usize, slot: usize) -> Advanced {
        let who = self.buildings[at].owner;
        let ty = self.buildings[at].queue.items[slot].ty;
        let researched = self.muster[who as usize].researched[ty];
        let target = self.queue_target(at, slot);
        let accel = production::accel(&self.tuning, production::Job::for_unit(researched), 1);

        let done = production::advance(&mut self.buildings[at].queue.items[slot], target, accel);
        if !done {
            return Advanced::Pending;
        }

        let muster = &self.muster[who as usize];
        let pop = self.unit_types[ty].price.pop;
        match production::finished(researched, muster.cap, muster.control, pop, false) {
            production::Handover::Population | production::Handover::Limit => Advanced::Blocked,
            production::Handover::Researched => {
                // `gain_tech`: the bit, and nothing else. No refund, no
                // skip-forward, no unit, no population.
                let mut ledger = economy::Ledger::default();
                self.buildings[at].queue.unqueue(slot, false, &mut ledger);
                let muster = &mut self.muster[who as usize];
                muster.queued_by_type[ty] -= 1;
                muster.researched[ty] = true;
                self.economy_changed(who);
                Advanced::Researched
            }
            production::Handover::Trained => {
                let pos = self.buildings[at].pos;
                // No refund on completion, and no skip-forward: the original
                // passes false for both, and they are the same argument.
                let mut ledger = economy::Ledger::default();
                self.buildings[at].queue.unqueue(slot, false, &mut ledger);

                let muster = &mut self.muster[who as usize];
                muster.queued_by_type[ty] -= 1;
                muster.by_type[ty] += 1;
                muster.control += pop;
                if let Some(g) = self.unit_types[ty].group {
                    muster.by_group[g] += 1;
                }

                let index = i16::try_from(self.units.len()).unwrap_or(i16::MAX);
                let mut unit = Unit::new(who, index, pos, self.unit_types[ty].hits);
                unit.kind = self.unit_types[ty].kind;
                let unit = self.add_unit(unit);
                self.economy_changed(who);
                Advanced::Trained(Produced { unit, ty, at })
            }
        }
    }

    /// Builds one unit of a type, if the player can pay for it and has room.
    ///
    /// This is the *instant* path — the price and the unit in one call, with
    /// no queue and no time. [`Sim::queue_up`] is the ordinary one.
    ///
    /// The order is the original's and it is observable: **the population is
    /// checked before the price**, so a player at the cap is refused without
    /// being charged. Everything after that is bookkeeping the ramp depends
    /// on — a count that is not incremented leaves the next one just as cheap.
    pub fn produce(&mut self, who: Player, ty: usize, pos: Pos) -> Result<usize, Refused> {
        let charges = self.price_of(who, ty);
        let pop = self.unit_types[ty].price.pop;
        let muster = &self.muster[who as usize];
        if cost::exceeds_population(muster.cap, muster.control, pop) {
            return Err(Refused::Population);
        }

        let available = self.holdings[who as usize].available;
        if !cost::can_pay(&charges, &self.ledgers[who as usize], &available, 1) {
            return Err(Refused::Cost);
        }
        cost::pay(&charges, &mut self.ledgers[who as usize], &available, false);

        let muster = &mut self.muster[who as usize];
        muster.by_type[ty] += 1;
        if let Some(g) = self.unit_types[ty].group {
            muster.by_group[g] += 1;
        }
        muster.control += pop;

        let index = i16::try_from(self.units.len()).unwrap_or(i16::MAX);
        let mut unit = Unit::new(who, index, pos, self.unit_types[ty].hits);
        unit.kind = self.unit_types[ty].kind;
        let at = self.add_unit(unit);
        self.economy_changed(who);
        Ok(at)
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
    /// Wholesale, which is what the original does at game setup and **not**
    /// what it does in play: there, `GameDaemon::check_borders` recomputes
    /// invalidated regions on a shared budget of 256 cells a frame, so a cell
    /// can carry stale ownership for up to `land cells / 256` frames after a
    /// city changes hands. Steady-state ownership is identical; the transient
    /// is a deliberate simplification until a recorded-game diff says it
    /// matters. See `docs/ATTRITION.md`, "Territory".
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

        // Then the buildings. `Build::process` and `Unit::process` are both
        // reached from `Objects::process_all`, so in the original they
        // interleave by object index rather than running in two passes. Doing
        // buildings first is the choice that keeps a unit handed over this
        // frame visible to this frame's unit loop, which is what the original
        // does whenever the building's index is the lower of the two — and it
        // is, for a building that existed before the unit it just made.
        self.process_queues();

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
        // The damage is sixteenths of a hit point per figure, carried through
        // a fractional accumulator the way `Object::take_damage` carries it,
        // so a lone figure loses one whole point a tick and a figure in a
        // squad of four loses one every fourth tick.
        let sixteenths = attrition::damage(unit.squad_size);
        let (lost, frac) = attrition::take_damage(unit.damage_frac, sixteenths);
        unit.damage_frac = frac;
        unit.health -= lost;
        let killed = !unit.alive();
        if killed {
            self.close_supply(i);
        }
        Some(Tick {
            unit: i,
            frame,
            sixteenths,
            lost,
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

/// One figure taking one tick of attrition damage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tick {
    pub unit: usize,
    pub frame: i64,
    /// Sixteenths of a hit point dealt — [`attrition::damage`] for the
    /// figure's squad size.
    pub sixteenths: i32,
    /// Whole hit points actually deducted this tick, after the fractional
    /// accumulator. Zero on most ticks for a figure in a squad of four.
    pub lost: i32,
    pub killed: bool,
}

#[cfg(test)]
#[path = "harness_tests.rs"]
mod tests;
