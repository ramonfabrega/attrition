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
    /// **Its own height**, `SubObjectData::z_internal`: the ground under
    /// its centre ([`crate::World::object_z`]) as it stands once its own
    /// terraform has run, at placement, and never rewritten after — a
    /// later neighbour's terraform moves the tile and leaves the building
    /// where it was (run1429's Factory `1/2026`: 185 from frame 1583, its
    /// tile 214 from 2186; item 4035). The one rewrite is the start's:
    /// [`Sim::reinit_game_units`] samples every building again once the
    /// whole start stands (item 4397).
    pub z: i32,
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
    pub repeat_air: bool, // twin of WallData::has_repeat_air
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
    /// SubObject flags bit 0x10. A surviving ordinary building is marked
    /// when struck. An active city is marked only at its damage ceiling.
    pub damage_marked: bool,
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
    /// **`build_masks & 0x8000`: jammed** (item 4541, `docs/COMBAT.md`
    /// §8.6). `Build::do_attack@006228f0` is its one writer: past the hits
    /// test it asks `ObjectData::is_jammed@00653660` (vslot `+0xe0`), sets
    /// the bit and returns without a shot when an enemy special forces unit
    /// stands within reach, and clears it otherwise ([`crate::Sim::is_jammed`]).
    /// `Wall::inc_time@0063fb60` reads it beside `near_o` for an anti-air
    /// building's wind-down.
    pub jammed: bool,
    /// **`build_masks & 0x2000`: no builder could be found** (item 4455).
    /// One writer sets it, `Leader::check_orphaned_buildings@006c9f20`'s
    /// tail (`orw %cx, 0x60(%eax)` at `6ca6c6`), on a computer's
    /// unfinished site no builder stands on and none was recruited for. One
    /// clears it, `Group::action_swarm_around@0070fbe0`'s `BUILD_AT`
    /// member arm, after the member's `add_build_order` (`andw` at
    /// `7107bb`). The listing holds no other `or`/`and` of the bit on
    /// `+0x60`/`+0x61`. Its reader, `Object::take_damage@00652020`'s
    /// site-abandon disband, is a SEAM in `damage_building`.
    pub no_builder: bool,
    /// `ObjectData::hold_frames` on a dead building: 30 from its close
    /// (`Object::close@00647160`, reached through `Wall::close:155`), one
    /// off a frame in `Objects::process_all`'s building band, one back for
    /// each squad `process_ejection` lets out. [`Sim::find_free`] reuses
    /// no number it holds (`docs/COMBAT.md` §59.3, item 3335).
    pub hold_frames: i32,
    /// `Build +0x78` on a fort: its record in the leader's fort list,
    /// `Forts::init_fort`'s at activation (`docs/LEADERS.md` §13.2).
    pub fort_slot: Option<usize>,
    /// `Build +0x78` on an oil well or platform: its record in the
    /// leader's well list, `init_oil_well`'s at activation (§13.2).
    pub oil_slot: Option<usize>,
    /// The city it belongs to, as an index into [`Sim::cities`].
    pub city: Option<usize>,
    /// The garrison chain — squad captains, in the order they entered.
    pub garrison: Vec<usize>,
    /// `ObjectData::launch_frames` (`+0x41`): a hangar's frames since its
    /// last launch, saturating at `FRAMES_BETWEEN_LAUNCHES`
    /// (`crate::air`'s `do_launch`, `docs/ORDERS.md` §38).
    pub launch_frames: i32,
    /// `ObjectData::launching` (`+0x44`): the planes `do_launch` has let
    /// go and whose EXIT has not yet killed its order ([`Launching`]).
    pub launching: Launching,
    /// `BuildData::founder`.
    pub founder: Player,
    /// **`BuildData::max_age` (`+0x83`) carried across a capture** (item
    /// 4463). `Build::init@00629740:54` writes the owner's ages and
    /// `Leader::gain_tech@006dcb60:538`–`550` raises it to the owner's on
    /// an age, never lowers it, so on a building that never changed hands
    /// the byte is the owner's ages. `Build::swap_team@00628210:44`–`48`
    /// gives the new building the larger of its own and the old one's, so
    /// a building taken from a player further on keeps that player's age:
    /// this is the old building's byte, 0 on one never captured, and
    /// [`Sim::build_max_age`] is the larger of it and the owner's ages.
    pub max_age_floor: u8,
    /// The four per-call construction-clock clauses, as inputs.
    pub clock: crate::cities::build::ClockMods,
    /// Frame of the last hit by another player, for the repair gate.
    pub hit_frame: Option<i64>,
    /// A fort's entry in [`Sim::sources`], while it projects territory.
    pub fort_source: Option<usize>,
    /// `BuildData::gather_down` (`+0x70`): the head of the gatherer chain,
    /// by unit slot; the rest of the chain is each member's own
    /// [`crate::Unit::gather_down`] (`Sim::gather_chain` walks it).
    /// `docs/ORDERS.md` §6.1.
    pub gather_head: Option<usize>,
    /// `BuildData::gather_max`: the slot count; `None` is uncapped (the
    /// non-flat count is `docs/ECONOMY.md`'s open item). A flat type gets 1.
    pub gather_max: Option<i32>,
    /// `BuildData::gather_from`: a woodcutter's or mine's resource tiles, in
    /// tiles, in the shuffled order the original keeps. An input.
    pub gather_from: Vec<Pos>,
    /// Allocated tile slots in the original MiningList, kept when tiles are removed.
    pub gather_from_capacity: i32,
    /// `MiningList::mtn` (`BuildData +0xb4`): the placed mountain range a
    /// mine's list was filled from — [`crate::Sim::find_nearest_mountain`]'s
    /// placed index — or −1. `BuildTypeData::find_gather_tcoords@0063bdc0`
    /// stores it when the range arm wins inside the reach, before it adds a
    /// tile; `MiningList::MiningList` and `Build::close` write −1, and
    /// nothing else stores it (`docs/ECONOMY.md`, "The mine's range").
    pub mtn: i8,
    /// `MiningList::cliff` (`+0xb5`): the cliff of [`crate::Sim::cliffs`]
    /// the list was filled from, or −1; the cliff arm's store, beside
    /// [`Building::mtn`]'s.
    pub cliff: i8,
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
    /// **`ObjectData::uid` (`+0x30`)** — `Object::init@00647750`'s
    /// per-player counter (`Objects +0x1d4 + 2 who`, post-incremented by
    /// every object the player inits, unit or building; [`Marks::uid`]).
    /// Nothing in the simulation reads it; the builds line folds it
    /// (`docs/CHECKSUM.md` §14, item 4094).
    pub uid: u16,
    /// **`WallData::frame_started` (`+0x5c`)** — the game frame
    /// `Wall::start@0063e810` ran on; `Wall::init@0063e9b0` writes −1.
    /// Folded by the builds line (`docs/CHECKSUM.md` §14).
    pub frame_started: i32,
    /// **`WallData::build_masks & 0x400`** — `Wall::process@00640450`'s
    /// latch of `helpers`: set when the building had a builder or repairer
    /// since the last process, cleared when it had none, and `helpers`
    /// zeroed either way. Folded by the builds line (`docs/CHECKSUM.md`
    /// §14).
    pub had_helpers: bool,
    /// **`ObjectData`'s four chain words** (`up`, `down`, `down_who`,
    /// `up_who`): the building's place on its cell's chain of objects,
    /// threaded where `Object::init@00647750` threads it — its own point's
    /// cell — and held, stale words and all (`docs/COLLISION.md` §3.2,
    /// item 4098). Folded by the builds line (`docs/CHECKSUM.md` §14).
    pub chain: crate::movement::collide::ObjectChain,
    /// `SubObjectData::flags & 0x80` of a building: raised by a round
    /// fired, dropped at the head of the next `do_attack`; it holds
    /// [`Building::visible`] against the 32-frame clear.
    pub attacking: bool,
    /// **`Build +0x7e` once a stance press has written it** (item 4282,
    /// `docs/CLIENT_INPUT.md` §27): `Group::action_stance@0070d440`'s
    /// building arm, [`crate::Sim::action_stance_buildings`]. `None` until
    /// then, and [`crate::Sim::build_stance`] answers `Build::init`'s byte.
    pub stance_set: Option<u8>,
}

impl Building {
    /// MiningList starts with five slots and doubles when a new tile would exceed the block.
    pub fn update_gather_capacity(&mut self) {
        self.gather_from_capacity = self.gather_from_capacity.max(5);
        while self.gather_from.len() > self.gather_from_capacity as usize {
            self.gather_from_capacity *= 2;
        }
    }

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

/// **`ObjectData::launching` (`+0x44`), a `SimpleArray<int> *`** (item
/// 4465, `docs/CHECKSUM.md` §18): the planes, or the missiles, a hangar has
/// let go, in the order they joined, by [`Sim::units`] index.
///
/// The pointer is null until the first launch: `Object::do_launch@0064f3b0:171`
/// mallocs the array then, empty (`SimpleArray<int>::SimpleArray<int>@00433c00`:
/// size 0, `increment` −1, flags 0), and adds the unit. **Nothing frees
/// it**: every other writer (`do_launch`'s kill arm, `Unit::kill_current_order@005e2cb0:148`,
/// `Build::clear_gather@00623180:70`, `Build::do_missile_launch@00622670`)
/// calls `SimpleArray<int>::remove@00462e70`, which shifts the rest down
/// in order and keeps the block. So `Object::walk_data@00647830` folds a
/// 1 for the pointer from the first launch on, empty or not, and then
/// the array's own walk.
///
/// `size` is the block's capacity: `ArrayBase<int>::add@0042daf0` grows a
/// full array through `SimpleArray<int>::increase_size@004220e0`, whose
/// `increment` −1 adds 4 to an empty block and the capacity to any other
/// (`4220ee`..`4220fc`: `edi = size ? size : 4`), so 4, 8, 16. run1721's
/// silo `1/2022` on 2588: length 1, size 4, `[84]`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Launching {
    list: Vec<usize>,
    size: u32,
}

impl Launching {
    /// `ArrayBase<int>::add`: the block allocated on the first, grown when full.
    pub fn push(&mut self, u: usize) {
        if self.list.len() as u32 == self.size {
            self.size += if self.size == 0 { 4 } else { self.size };
        }
        self.list.push(u);
    }

    /// `SimpleArray<int>::remove` of every entry `keep` refuses (none is
    /// held twice: `do_launch` adds only what `find` misses).
    pub fn retain(&mut self, keep: impl FnMut(&usize) -> bool) {
        self.list.retain(keep);
    }

    /// The entry at `k`, taken out, the rest shifted down.
    pub fn remove(&mut self, k: usize) -> usize {
        self.list.remove(k)
    }

    /// The pointer is not null: something has launched from here.
    pub const fn allocated(&self) -> bool {
        self.size != 0
    }

    /// The block's capacity, `SimpleArray +0x8`.
    pub const fn size(&self) -> u32 {
        self.size
    }
}

impl std::ops::Deref for Launching {
    type Target = [usize];
    fn deref(&self) -> &[usize] {
        &self.list
    }
}

impl PartialEq<Vec<usize>> for Launching {
    fn eq(&self, other: &Vec<usize>) -> bool {
        self.list == *other
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
    /// **`Setup::reinit_game_units@005abf90`** (item 4397; `docs/START.md`
    /// §2 step 8), `Game::init@0058c480:629`'s call after `build_game`:
    /// for each seated leader, every live object (`ObjectData +0x8 & 1`)
    /// in the unit band, then `2000…` and `3000…`, has its `z` (`+0xc`)
    /// sampled again as `TerrainOut::find_tcoord_z(x, y, 1)` of its tile
    /// (`pushl $0x1` at `005ac000`), which is [`crate::World::object_z`].
    /// A unit's height is read off its tile wherever this crate needs it,
    /// so only the buildings carry the write: a start building placed
    /// before a later neighbour's terraform moved its tile takes the
    /// ground as the whole start leaves it (the sample's Farms `0/2003`
    /// under `0/2006`'s terraform, 142 → 145, and `3/2002` under
    /// `3/2005`'s, 316 → 330). Every building at this point is a seated
    /// leader's.
    pub fn reinit_game_units(&mut self) {
        for b in 0..self.buildings.len() {
            if self.buildings[b].alive {
                self.buildings[b].z = self.world.object_z(self.buildings[b].pos.tile());
            }
        }
    }

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
        let z = self.world.object_z(pos.tile());
        let uid = self.next_uid(owner);
        let (chain, gather_from_capacity) = self.building_slot_state(owner, index);
        self.buildings.push(Building {
            owner,
            index,
            pos,
            z,
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
            damage_marked: false,
            job_counter: 0,
            job_counter_2: 0,
            constr_time: 0,
            construct_hits: 0,
            helpers: 0,
            under_attack: 0,
            eject_pending: false,
            jammed: false,
            no_builder: false,
            hold_frames: 0,
            fort_slot: None,
            oil_slot: None,
            stance_set: None,
            city: None,
            garrison: Vec::new(),
            launch_frames: 0,
            launching: Launching::default(),
            founder: owner,
            uid,
            frame_started: 0,
            had_helpers: false,
            chain,
            clock: crate::cities::build::ClockMods::default(),
            hit_frame: None,
            fort_source: None,
            gather_head: None,
            gather_max: None,
            gather_from: Vec::new(),
            gather_from_capacity,
            mtn: -1,
            cliff: -1,
            gather_bumped: false,
            dock_slot: None,
            farm: Farm::default(),
            ever_seen: 0,
            ever_seen_completed: 0,
            max_age_floor: 0,
            mylos: 0,
            healing: 0,
            visible: 0,
            attacking: false,
        });
        let b = self.buildings.len() - 1;
        self.object_chain_add(crate::combat::Obj::Building(b));
        b
    }

    /// The chain and allocated gathering list survive slot reuse.
    /// **The chain words a building's `Object::init@00647750` starts
    /// from**: its number's last occupant's `down_who` and `up_who`, or
    /// `Object::Object@00646e80`'s −1s on a number never used
    /// ([`crate::movement::collide::ObjectChain::reinit`]).
    pub(crate) fn building_slot_state(
        &self,
        owner: Player,
        index: i16,
    ) -> (crate::movement::collide::ObjectChain, i32) {
        let prev = self
            .buildings
            .iter()
            .rev()
            .find(|b| b.owner == owner && b.index == index);
        (
            crate::movement::collide::ObjectChain::reinit(prev.map(|b| &b.chain)),
            prev.map_or(5, |b| b.gather_from_capacity),
        )
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
