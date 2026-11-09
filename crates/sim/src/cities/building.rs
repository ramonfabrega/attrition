//! [`Building`] and the calls that add one or its type (`docs/CITIES.md` §1;
//! `docs/STRUCTURE.md` §3, `cities/`).
//!
//! Moved out of `lib.rs` unchanged by item 4014 (`docs/STRUCTURE.md` §5, move 6).

use crate::*;

/// A building: a production queue, a combat profile, and — when it has a
/// type — a footprint, a construction clock, a city and a garrison.
/// `docs/CITIES.md` §1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Building {
    pub owner: Player,
    /// Its number in its owner's object array — `SubObjectData::o`, in
    /// `[2000, 3000)`. The scripts and the AI's producers hold buildings by
    /// this number, and a dump's `b.o` is the same figure.
    pub index: i16,
    /// Its snapped centre. Where a finished unit appears: `Build::train`
    /// places it at the building's own position and then puts it inside, so
    /// every unit is born garrisoned and leaves by the ejection path.
    pub pos: Pos,
    pub queue: crate::cities::production::Queue,
    /// Whether this is a library — the original's `is(LIBRARY, 0)`. Two rules
    /// key on it and on nothing else: orders and cancels given to any library
    /// are forwarded to the player's *first* one, and only that first
    /// library's queue fans out across `Muster::library_cities` slots. There
    /// is no building-type system here yet; this flag is the smallest honest
    /// stand-in for the one test production needs.
    pub is_library: bool,
    /// Its combat columns — `None` for a building that neither shoots nor is
    /// shot at. A building with a profile has hit points and can die.
    pub combat: Option<combat::Profile>,
    /// The hit points it was built with and what it has left, whole and in
    /// sixteenths, the way a unit carries them.
    pub hits: i32,
    pub health: i32,
    pub damage_frac: i32,
    /// `WallData::is_active` — finished; an unfinished building takes
    /// quadruple from other buildings and loses progress when hit.
    pub active: bool,
    /// The garrison's contribution to `get_garrison_arrows` — the sum of
    /// `attack / 10` over the foot soldiers inside, an input until there is a
    /// garrison.
    pub garrison_attack: i32,
    /// `BuildData::recharging`, `attack_ox/attack_whom`, the explicit-order
    /// flag.
    pub recharging: i32,
    pub target: Option<combat::Obj>,
    /// `ObjectData::near_o`/`near_who` — see [`Unit::near`].
    pub near: Option<combat::Obj>,
    pub ordered: bool,
    /// `ObjectData::targeted`.
    pub targeted: i32,
    /// Its type, as an index into [`Sim::build_types`]; `None` for the bare
    /// object.
    pub ty: Option<usize>,
    /// The type it was placed as — `BuildData::orig_type`.
    pub orig_ty: Option<usize>,
    /// `flags & 1`: in use. A dead building keeps its slot (and ejects its
    /// garrison) until `hold_frames` runs out.
    /// **`ObjectData::infiltrated` (`+0x3a`)** — the players with an
    /// informer in this building (`cast_double_agent`, `docs/GOLDEN.md`
    /// §27).
    pub infiltrated: u8,
    pub alive: bool,
    /// `flags & 2`: `Wall::start` has run — the footprint is committed.
    pub started: bool,
    /// `build_masks & 0x1000`: has been activated at some point.
    pub activated: bool,
    /// `build_masks & 0x80`: **repeat air** — `WallData::has_repeat_air
    /// @00472410`, the base's vslot `0xf0`. A plane that lands here keeps
    /// its order with the action bit cleared (`Unit::land_plane`), and
    /// `Object::do_launch` launches an unflagged order from it.
    /// `Build::init@00629740` sets it, with `0x8`, on every building that
    /// `can_carry(AIR)`; only the player's repeat-orders toggle
    /// (`Group::action_buildmask(0x80)`) clears it (`docs/ORDERS.md` §40).
    pub repeat_air: bool,
    /// `BuildData::gather`, the gather point list (`crate::cities::rally`): the
    /// player's rally points, in order.
    pub gather: Vec<crate::cities::rally::GatherPoint>,
    /// `build_masks & 0x100`: this building's roads want replanning, and
    /// `Build::process` will replan them on the frame `(frame + o) % 16`
    /// picks out. Set by `City::regen_roads` — `crate::movement::roads` §1.
    pub regen_roads: bool,
    /// `ObjectData::damage`, whole hits taken. [`Building::health`] is kept
    /// equal to `hits_now − damage` and is what combat reads as the share.
    pub damage: i32,
    /// `WallData::job_counter`, `job_counter_2`, `constr_time`,
    /// `construct_hits`; `docs/CITIES.md` §3.
    pub job_counter: i32,
    pub job_counter_2: i32,
    pub constr_time: i32,
    pub construct_hits: i32,
    /// `WallData::helpers`: builders or repairers that contributed this frame.
    pub helpers: i32,
    /// `build_masks & 0x10 | 0x20`: the two-stage under-attack latch.
    pub under_attack: u8,
    /// `build_masks & 0x4000`: eject one squad a frame.
    pub eject_pending: bool,
    /// `ObjectData::hold_frames` on a dead building: 30 from its close
    /// (`Object::close@00647160`, reached through `Wall::close:155`), one
    /// off a frame in `Objects::process_all`'s building band, one back for
    /// each squad `process_ejection` lets out. [`Sim::find_free`] reuses
    /// no number it holds (`docs/COMBAT.md` §59.3, item 3335).
    pub hold_frames: i32,
    /// The city it belongs to, as an index into [`Sim::cities`].
    pub city: Option<usize>,
    /// The garrison chain — squad captains, in the order they entered.
    pub garrison: Vec<usize>,
    /// `ObjectData::launch_frames` (`+0x41`): a hangar's frames since its
    /// last launch, saturating at `FRAMES_BETWEEN_LAUNCHES`
    /// (`crate::air`'s `do_launch`, `docs/ORDERS.md` §38).
    pub launch_frames: i32,
    /// `ObjectData::launching` (`+0x44`): the planes `do_launch` has let
    /// go and whose EXIT has not yet killed its order.
    pub launching: Vec<usize>,
    /// `BuildData::founder`.
    pub founder: Player,
    /// The four per-call construction-clock clauses, as inputs.
    pub clock: crate::cities::build::ClockMods,
    /// Frame of the last hit by another player, for the repair gate.
    pub hit_frame: Option<i64>,
    /// A fort's entry in [`Sim::sources`], while it projects territory.
    pub fort_source: Option<usize>,
    /// `BuildData::gather_down`'s chain: the units registered as gathering
    /// here, newest first. `docs/ORDERS.md` §6.1.
    pub gatherers: Vec<usize>,
    /// `BuildData::gather_max`: the slot count; `None` is uncapped (the
    /// non-flat count is `docs/ECONOMY.md`'s open item). A flat type gets 1.
    pub gather_max: Option<i32>,
    /// `BuildData::gather_from`: a woodcutter's or mine's resource tiles, in
    /// tiles, in the shuffled order the original keeps. An input.
    pub gather_from: Vec<Pos>,
    /// `build_masks & 0x800`: a gatherer bumped `recharging` this frame.
    pub gather_bumped: bool,
    /// The farm's tile states and growth counts (§6.5).
    pub farm: Farm,
    /// `BuildData +0x78`: this dock's slot in its owner's registry
    /// (`docs/TRANSPORT.md` §5.1), while it is active.
    pub dock_slot: Option<usize>,
    /// `WallData::ever_seen` (`Wall +0x62`) and `ever_seen_completed`
    /// (`+0x63`) — the dump prints both under those names. One bit per
    /// player: who has ever had this building's footprint in **current**
    /// line of sight, and who has had it there while it was finished.
    /// `Wall::check_ever_seen@0063ce70` grows them, and a newly arrived
    /// foreign bit is what makes the building light itself into that
    /// player's fog (`docs/VISION.md` §6.1).
    pub ever_seen: u8,
    pub ever_seen_completed: u8,
    /// **`ObjectData::mylos` (`+0x3c`) of a building** — the byte
    /// `Wall::update_los@0063eeb0` last left, a cache as a unit's is
    /// (item 3965, `docs/VISION.md` §2.1). [`Sim::update_build_los`] writes
    /// it where the original calls the vslot `+0x160` on a building;
    /// [`Sim::build_los`] is the derivation.
    pub mylos: i32,
    /// **`ObjectData::healing` (`+0x38`) of a building** — the heal's
    /// display clock, as a unit's is (item 3987, `docs/GOLDEN.md` §180).
    /// Its one writer on a building is `Build::process@0061edf0`'s patriot
    /// repair ([`Sim::patriot_repair`]), `max(rate, healing)` with
    /// `FRENCH_SUPPLY_HEAL_RATE`; `Build::process` takes one off it each
    /// frame of an active building, before the ejection.
    pub healing: i16,
    /// **`ObjectData::visible` (`+0x40`) of a building** — the players it
    /// has shown itself to by shooting them (`Build::do_attack@006228f0`,
    /// `622b5b`..`622bd0`, `docs/GOLDEN.md` §59): the fallback arm of
    /// `BuildData::is_seen`, so a Stockade nobody has seen is still a
    /// legal target for the unit it shot.
    pub visible: u8,
    /// `SubObjectData::flags & 0x80` of a building: raised by a round
    /// fired, dropped at the head of the next `do_attack`; it holds
    /// [`Building::visible`] against the 32-frame clear.
    pub attacking: bool,
}

impl Building {
    /// `hits(0)`: the site's growing figure while not active, the full one
    /// after.
    pub const fn hits_now(&self) -> i32 {
        if self.active {
            self.hits
        } else {
            self.construct_hits
        }
    }

    /// Keeps `health` — combat's share — equal to `hits_now − damage`.
    pub(crate) const fn sync_health(&mut self) {
        self.health = self.hits_now() - self.damage;
    }

    /// `WallData::is_under_attack` = `build_masks & 0x20`.
    pub const fn is_under_attack(&self) -> bool {
        self.under_attack & 0x2 != 0
    }

    /// The frame counter this building's periodic work is phased against —
    /// **its object number, not its handle** ([`Unit::phase`]'s twin).
    ///
    /// `Build::process@0061edf0:728` reads `(frame + o)`, and every period
    /// the building keeps hangs off it: the under-attack decay and the
    /// building's own attrition every 32 (`docs/CITIES.md` §1.3, §9.5), the
    /// road replan every 16 (`docs/ROADS.md` §1), the capture re-test every
    /// 64 (§7.1), the city heal every `CITY_HEAL_RATE` (§8.2) and the
    /// tower's target search (`docs/COMBAT.md` §8.6).
    ///
    /// [`Building::index`] is that `o`; the handle is this crate's `Vec`
    /// slot, which agrees with it in no game at all — `o` is per player and
    /// starts at [`BUILD_BASE`] (2000, which is 16 mod 32), the handle is
    /// global and starts at 0, and `o` is recycled by [`Sim::find_free`]
    /// while the `Vec` only grows.
    pub const fn phase(&self, frame: i64) -> i64 {
        frame + self.index as i64
    }
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

impl Sim {
    /// Registers a building type and returns its id.
    pub fn add_build_type(&mut self, ty: crate::cities::build::BuildType) -> usize {
        self.tables.build_types.push(ty);
        if self.combat.table.builds() < self.tables.build_types.len() {
            self.combat.table = self
                .combat
                .table
                .grown(self.tables.unit_types.len(), self.tables.build_types.len());
        }
        self.tables.build_types.len() - 1
    }

    /// Adds a production building and returns its index.
    pub fn add_building(&mut self, owner: Player, pos: Pos, capacity: usize) -> usize {
        let index = self
            .find_free(owner, BUILD_BASE, WALL_BASE)
            .expect("a player's building band is full");
        self.buildings.push(Building {
            owner,
            index,
            pos,
            queue: crate::cities::production::Queue::new(capacity),
            is_library: false,
            combat: None,
            hits: 0,
            health: 0,
            damage_frac: 0,
            active: true,
            repeat_air: false,
            gather: Vec::new(),
            regen_roads: false,
            garrison_attack: 0,
            recharging: 0,
            target: None,
            ordered: false,
            near: None,
            targeted: 0,
            ty: None,
            orig_ty: None,
            infiltrated: 0,
            alive: true,
            started: true,
            activated: true,
            damage: 0,
            job_counter: 0,
            job_counter_2: 0,
            constr_time: 0,
            construct_hits: 0,
            helpers: 0,
            under_attack: 0,
            eject_pending: false,
            hold_frames: 0,
            city: None,
            garrison: Vec::new(),
            launch_frames: 0,
            launching: Vec::new(),
            founder: owner,
            clock: crate::cities::build::ClockMods::default(),
            hit_frame: None,
            fort_source: None,
            gatherers: Vec::new(),
            gather_max: None,
            gather_from: Vec::new(),
            gather_bumped: false,
            dock_slot: None,
            farm: Farm::default(),
            ever_seen: 0,
            ever_seen_completed: 0,
            mylos: 0,
            healing: 0,
            visible: 0,
            attacking: false,
        });
        self.buildings.len() - 1
    }

    /// Adds a library and returns its index. See [`Building::is_library`].
    pub fn add_library(&mut self, owner: Player, pos: Pos, capacity: usize) -> usize {
        let at = self.add_building(owner, pos, capacity);
        self.buildings[at].is_library = true;
        at
    }

    /// The player's first library — `LeaderData::get_first_library@006db6c0`,
    /// which walks the owner's buildings from the lowest index and returns
    /// the first that is alive (`flags & 1`), active (`WallData::is_active`,
    /// vslot `+0x4c`), in a city (`+0x72 >= 0`), not unassimilated and a
    /// Library by lineage (item 3969: a closed library was still the first
    /// here, and every research order after it was refused).
    pub fn first_library(&self, who: Player) -> Option<usize> {
        (0..self.buildings.len()).find(|&b| {
            let bd = &self.buildings[b];
            bd.owner == who
                && bd.is_library
                && bd.alive
                && bd.active
                && bd.city.is_some()
                && !self.building_unassimilated(b)
        })
    }
}
