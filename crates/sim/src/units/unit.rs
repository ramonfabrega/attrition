//! [`Unit`], [`UnitType`], [`Muster`], the object-number bands and the squad
//! walks (`docs/STRUCTURE.md` §3, `units/`).
//!
//! Moved out of `lib.rs` unchanged by item 4014 (`docs/STRUCTURE.md` §5, move 6).

use crate::*;

/// [`Unit::head_serial`]: the head order's identity, the original's
/// `UnitOrder *`. **Identity, not state**: it compares equal to every other
/// serial, so two units that differ only in which object heads their list
/// are the same unit to every comparison of state, and it prints as nothing
/// in particular (a checkpoint's reseat replaces one order with an equal
/// one, and the run69 interventions compare state by value and by `Debug`).
#[derive(Clone, Copy, Default)]
pub struct HeadSerial(pub u32);

impl std::fmt::Debug for HeadSerial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HeadSerial")
    }
}

impl PartialEq for HeadSerial {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for HeadSerial {}

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
    pub kind: crate::territory::attrition::UnitKind,
    /// Whether the unit is on the map, rather than garrisoned in a building or
    /// riding in a transport. Off the map, none of this runs.
    pub on_map: bool,
    /// Which slot of its owner's supply list this unit registered as, if its
    /// type is a supply source. The original caches the same index in the
    /// unit and gives it back when the unit dies.
    pub supply_slot: Option<usize>,
    /// `UnitData +0x86` for a **caravan**: its slot in the owner's
    /// [`caravan::Caravans`] list. The same field carries a hero's or a
    /// special's slot in the original; this is the caravan half alone.
    pub caravan: Option<usize>,
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
    /// Its type, as an index into [`Sim::unit_types`], if it has one. A unit
    /// without a type has no combat profile and neither attacks nor is worth
    /// anything to a target search; attrition and movement do not need it.
    pub ty: Option<usize>,
    /// What combat keeps on the unit: the reload counter, the target, the
    /// overkill record. See `docs/COMBAT.md`.
    pub combat: combat::State,
    /// The building this unit is garrisoned in — `UnitData::inside_up` when
    /// it points at a building. `Some` implies `on_map == false`. See
    /// `docs/CITIES.md` §6.
    pub inside: Option<usize>,
    /// The **boat** this unit is riding — `UnitData::inside_up` when it
    /// points at a unit rather than a building. `Some` implies
    /// `on_map == false`, and it is the whole reason a passenger's orders
    /// stop being stepped (`docs/TRANSPORT.md` §6).
    pub inside_unit: Option<usize>,
    /// **A unit's own training queue** (item 3981, `docs/AI.md` §413): the
    /// Aircraft Carrier's planes, `UnitData +0xa0 num_queued` (a short)
    /// and `+0x64 queue_time`, the clock (`crate::air::carrier_queue`).
    pub num_queued: i16,
    pub queue_time: i32,
    /// The hit points a whole figure carries, what the garrison heal repairs
    /// up to.
    pub max_health: i32,
    /// The order list — front is the current order. `docs/ORDERS.md` §1.4.
    pub orders: std::collections::VecDeque<orders::Order>,
    /// The path stack; the top is the current waypoint (§4.2).
    pub path: Vec<orders::PathData>,
    /// **A 48-grid search this unit suspended** — `UnitData +0x104..0x148`,
    /// the five containers and the nine scalars `astar_path` hands over
    /// when it runs out of `limit` short of the goal
    /// (`docs/PATHFINDER.md` §4.3 step 3, §18).
    ///
    /// While it is `Some`, `do_move` does not step: its first block resumes
    /// the search instead (`docs/ORDERS.md` §4.4 step 2).
    /// [`Sim::clear_partial_path`] is what drops it.
    pub search: Option<Box<crate::movement::path::Search>>,
    /// **`UnitData::start_dist` (`+0x130`) — the dumped witness that this
    /// unit suspended a 48-grid search** (`docs/PATHFINDER.md` §18.6).
    ///
    /// It is a *stamp on the unit*, not a field of the stash: the original
    /// keeps it on `UnitData`, so it outlives the search that wrote it and
    /// still reads back long after `clear_partial_path` has freed the five
    /// containers. Three writers in the whole executable, and only one
    /// writes a value — `astar_path@00683770:517`, the suspend, which
    /// stores the search's start-to-goal Manhattan; the other two zero it
    /// at a unit's birth (`UnitData::UnitData@00606670:74` and
    /// `Unit::init@00612100:567`), which is why a recycled `o` slot reads
    /// 0 again and a live unit never does.
    pub start_dist: i32,
    /// `UnitData::tolerance`: the current waypoint's arrival radius.
    pub tolerance: i32,
    /// `unit_masks & 8`: a straight line to the waypoint has been verified.
    pub line_ok: bool,
    /// `UnitData::path_recursion`.
    pub path_recursion: u8,
    /// `UnitData::orders_x/orders_y`: the final destination of the leading
    /// run of transit moves.
    pub orders_pos: Pos,
    /// `UnitData::idle` (§2.4).
    pub idle: u8,
    /// `UnitData::stance`, the worker stance: 1 is the normal citizen (builds
    /// and gathers), 0 gathers only, 2 builds only.
    pub stance: u8,
    /// `UnitData::spell_time` (`+0x98`) — a caster's clock, and the field a
    /// **bird** reuses as its patrol counter: `Animal::think_bird` steps it
    /// every frame it runs and again on each eighth, and the number it has
    /// reached is the modulus of the landing roll (`crate::gaia`,
    /// `docs/SYNC.md` §3.9). `Unit::init` clears it with `mana_burn`, so a
    /// bird starts at 0.
    pub spell_time: i16,
    /// **`UnitData::mana_burn` (`+0x96`)** — the craft a caster has spent
    /// and not yet recovered: `mana_left = mana − mana_burn`. `Unit::init`
    /// starts a **spy** at half its `mana` (`is(SPY, 0)`, the only type it
    /// names), `pay_cast_costs` adds a craft's `MANA`, and `Unit::process`
    /// takes one back a frame while `unit_masks & 0x2a000` is clear
    /// (`docs/GOLDEN.md` §27). SEAM: an aircraft's burn — `do_strafe`'s
    /// `bombing_mana_cost` and `process`'s air recharge — is not carried.
    pub mana_burn: i16,
    /// **`UnitData::full`** (`+0xac`, a signed byte): "nobody can get at
    /// this one". `Unit::find_melee_pos@006010b0` adds 5 to the *target's*
    /// when it finds no open slot round it, capped at `0x1e`; `Unit::
    /// process@00610bc0` takes one off a frame at its head; and
    /// `Object::find_nearby_target`'s score of a unit divides by `full + 1`
    /// (`0064f110`). Item 3385: run1011's woodcutter `0/3` reads 5, 9, 8 …
    /// from block 13278 as two Inti Macemen fail to reach it, and the
    /// Heavy Archer `1/99` takes the citizen `0/4` over it on 13282.
    pub full: i8,
    /// **`UnitData::waiting`** (`+0xad`, a signed byte): `Unit::fight@
    /// 005fd4d0`'s search budget. Its invalid-target arm (`LAB_005fdb9e`)
    /// adds 2 and returns without searching while `waiting < 5` and the
    /// leader's [`Sim::retargets`] is past ten; `Unit::process@00610bc0`
    /// takes one off a frame at its head; `Unit::init@00612100` zeroes it.
    /// Item 3598: run1241b's Phalanx `1/116` reads 2 on block 6718 and 1
    /// on 6719, the frame twelve of who=1's units lost the Citizen `0/12`.
    pub waiting: i8,
    /// **`ObjectData::healing`** (`+0x38`, a short): the heal's display
    /// clock. Every heal `Unit::process_healing@005e0670` lands writes
    /// `max(rate, healing)` with that arm's period, `Unit::process@
    /// 00610bc0`'s head takes one off a frame, and its air arm writes 0 on
    /// the map and 1 on a heal inside; `Unit::init@00612100` zeroes it.
    /// Item 3961 (`docs/AI.md` §405): run1151c's Citizens `0/5` and `0/8`
    /// read 45 on their civilian heal and count down, and who=1's planes
    /// in their Airbase read 1 one frame in 11.
    pub healing: i16,
    /// **`UnitData::cavarch_o`/`cavarch_who` (`+0xa2`/`+0xa8`)** and the
    /// uid beside them (`+0xa6`) — a caster's target, written by
    /// `Group::action_spell` and again by each frame of `do_cast`'s
    /// targeted arm (`docs/GOLDEN.md` §27). `None` is `cavarch_o` −1.
    pub cast_target: Option<combat::Obj>,
    /// **`cavarch_who` (`+0xa8`) itself**: 0 from `Unit::init`, the
    /// target's owner once `action_spell` or `do_cast` writes a target, and
    /// −1 after an untargeted press — `action_spell` writes the pair
    /// `(ox, whom)` = `(−1, −1)` whenever it differs from what is held
    /// (`docs/GOLDEN.md` §48).
    pub cavarch_who: i8,
    /// **`cavarch_o` −2** (`0xfffe`): a fire-on-the-move type's search
    /// found nothing twice running, and the next one waits for the
    /// thirty-two-frame phase (`Unit::cavarch_fight@005ff4b0`'s tail,
    /// `Unit::do_move@005f7b30:66`). `cast_target` is `None` with it.
    /// [`crate::combat::cavarch`] writes it; −1 is `cast_target` `None` without.
    pub cavarch_idle: bool,
    /// **The head order's identity** — bumped by every write that puts a
    /// different order object at the head of the list (a kill, a remove at
    /// the head, a push in front, the ungroup's new plain move). It stands
    /// in for the original's pointer compare `pUVar1 == param_1` after
    /// `do_move` (`Unit::do_attack_to@005f2320`, `Unit::do_explore_to@005f24a0`):
    /// a move whose timer ran out is killed and the guard re-issues an
    /// `ATTACK_TO` to the same point, which a kind-and-point test calls the
    /// same order and the original does not (`docs/AI.md` §135).
    pub head_serial: HeadSerial,
    /// **`unit_masks & 0x20000`** — "the cast has started": set on a
    /// targeted cast's first in-range frame, cleared by
    /// `kill_current_order`, and one of the three bits that stop mana
    /// recovering (`docs/GOLDEN.md` §27).
    pub casting: bool,
    /// **`unit_masks & 0x8000`** — a hero on a Forced March, set by
    /// `SpellType::cast_march` and held to the craft's end frame, the
    /// `ActiveSpell` its hero record keeps (`docs/AI.md` §99.13); one of
    /// the three bits that stop mana recovering.
    pub marching: Option<i64>,
    /// **`unit_masks & 0x2000`** — a hero in an Ambush, set by
    /// `SpellType::cast_ambush@00671610` (`|= 0x2800`) and held to the
    /// craft's end frame, the `ActiveSpell` its hero record keeps, which
    /// `Caster::process_spells@00739ad0` expires (`&= 0xffffd7ff`); the
    /// third of the three bits that stop mana recovering (`docs/AI.md`
    /// §261).
    pub ambushing: Option<i64>,
    /// **`ObjectData::infiltrated` (`+0x3a`)** — the players who have an
    /// informer in this object, one bit per `who`: `cast_double_agent`
    /// sets the caster's (`docs/GOLDEN.md` §27).
    pub infiltrated: u8,
    /// **A plane's figure in the air** — guy 0's bank, pitch and altitude
    /// (`GuyData +0x44/+0x48/+0x4c/+0x50/+0x14/+0x70`), which
    /// `Unit::bank_aircraft` and `Unit::pitch_aircraft` step on every frame
    /// of a flight (`docs/ORDERS.md` §33). Zero on the ground and on every
    /// unit that never flies. A wild bird's bank is `crate::air::Flight`
    /// instead, which predates this and carries no altitude.
    pub airframe: air::Airframe,
    /// `unit_masks & 0x400`: has been given a build or repair order.
    pub was_builder: bool,
    /// **`ObjectData::uid` (`+0x30`)** — `Object::init@00647750`'s per-player
    /// counter ([`Marks::uid`]), taken at [`crate::Sim::add_unit`]. Nothing
    /// in the simulation reads it; the units line folds it, and a target
    /// order's `uid` is its target's (`docs/CHECKSUM.md` §17, item 4109).
    pub uid: u16,
    /// **`UnitData::los_x`/`los_y` (`+0x78`/`+0x7c`)** — the centre of the
    /// 48-unit cell `Unit::init@00612100:103` was handed, `div_3(v >> 4) ·
    /// 0x30 + 0x18`, its only writer: the unit's birth cell, taken at
    /// [`crate::Sim::add_unit`]. Read by nothing but the units line
    /// (`docs/CHECKSUM.md` §17, item 4109).
    pub birth_cell: Pos,
    /// `unit_masks & 0x78000000`: the carrying walk a gatherer plays.
    ///
    /// Four bits, and `Guy::set_anim`'s walk arm reads them in its own
    /// order — `0x10000000` `WALK_TO_WOOD`, then `0x8000000`
    /// `WALK_WITH_WOOD`, then `0x40000000` `WALK_TO_ORE`, then
    /// `0x20000000` `WALK_WITH_ORE` — so they are kept as the nibble
    /// rather than as a slot. `Unit::do_non_flat_gather` is the only
    /// writer: it clears them whole on the frame a worker reaches its
    /// tile and sets one on each walk it issues. `Unit::think` clears
    /// them for a citizen, `kill_current_order` for anybody.
    ///
    /// They are why a walk this crate had been playing as the plain
    /// `CHAR_WALK` — the carrying slots' lengths were unknown, so the
    /// packet fallback swallowed them — is a carrying walk, and why an
    /// arrival stand fires for one and not the other (`docs/ANIM.md`
    /// §4.4).
    pub carry: u32,
    /// `SubObjectData::flags & 0x10`: could not reach its target.
    pub cant_reach: bool,
    /// `unit_masks & 1`: a decoy; not counted as a gatherer.
    pub decoy: bool,
    /// `unit_flags & 0x400`: the Kremlin's spy, which `Build::process`
    /// tags as it trains it and which stands outside the owner's `control`
    /// (`docs/AI.md` §188).
    pub kremlin_spy: bool,
    /// `ObjectData::mylos` (`+0x3c`): the line of sight in tiles **as
    /// `Unit::update_los` last wrote it** — a cache, not a derivation. The
    /// fog sweep and the follower's spacing read it; [`Sim::update_los`]
    /// is its one writer, called where the original calls vtable `+0x160`
    /// on a unit (`docs/VISION.md` §2).
    pub mylos: i32,
    /// **`UnitData::unit_masks2` (`+0x6c`)** — carried as the word the dump
    /// prints, and one bit of it is written: [`combat::umask2::NOT_FIRING`]
    /// (`docs/COMBAT.md` §43.2, `docs/ANIM.md` §5).
    ///
    /// Every dump on this disk carries this word as `0`, `4`, `0x10` or
    /// `0x40000` and nothing else, so the one bit is the whole of the
    /// field on both headline windows — which is why
    /// [`crate::Sim::guy_inc_time`]'s zero step could be implemented
    /// without modelling the rest of the word.
    pub unit_masks2: u32,
    /// **`ObjectData::hold_frames`** (`+0x32`) — how many more frames this
    /// slot is held after the figure inside it is gone (`docs/COMBAT.md`
    /// §11, §42.3).
    ///
    /// Three writers, all of them on a **dead** object: `Object::die` sets
    /// it to `max(1, …)` over the object's own ammo still in flight, so
    /// that a dead archer's arrow still lands; `Ammo::inc_time` bumps it
    /// once a frame for every in-flight shot whose **shooter** is no
    /// longer active; and `DeathObj::inc_time` bumps it once a frame while
    /// the death animation plays. Nothing writes it on a living unit,
    /// which is exactly what the dump says — 0 on every unit-frame of
    /// both headline windows — and why §9.2's "a target that is no longer
    /// active has its `hold_frames` bumped" was the wrong way round: the
    /// ammo matches on `+0x3c`/`+0x40`, the pair `Ammo::init` fills from
    /// the shooter.
    ///
    /// It holds nothing here: this crate never recycles a unit slot, so
    /// the field is carried to be **compared** rather than acted on.
    pub hold_frames: i32,
    /// `UnitData::avoid_x/avoid_y`: the point `find_path` recorded as
    /// unreachable, which `valid_wcoord` refuses. Cleared before each step
    /// (`docs/ORDERS.md` §4.5).
    pub avoid: Option<Pos>,
    /// The figures' animation clocks — `UnitData::guys`, one `Guy` per
    /// member (`docs/ANIM.md`). Empty for a unit stood up without art,
    /// which then plays nothing and draws nothing for it.
    pub guys: Vec<crate::units::anim::Guy>,
    /// `UnitData::is_captain` — `o_up < 0`, a unit that heads its own
    /// squad; every standalone unit is one.
    pub captain: bool,
    /// `UnitData +0x8e` — this figure's captain, the object `Group::add`
    /// substitutes for it when `keep_captain` is 0 (`docs/GROUPS.md` §4.1).
    /// `None` for a captain, which is what [`Self::captain`] says: the
    /// original keeps one short and reads its sign bit.
    pub o_up: Option<usize>,
    /// `UnitData +0x90` — the next figure down the squad chain, which
    /// `Group::add` pulls in behind its captain with `keep_captain = 1`.
    /// A three-figure squad is a captain, its `o_down`, and that one's.
    pub o_down: Option<usize>,
    /// `UnitData +0x80` — the pool slot this unit's **group back-pointer**
    /// names, `0..64` within its owner's block, or `None` for −1
    /// (`docs/GROUPS.md` §23). The dump prints it as `group`, `who·64 +
    /// slot`.
    ///
    /// It is its own state, not a view of any list: `Unit::set_group`,
    /// `Groups::push_group` and `get_open_slot`'s eviction write it,
    /// `Group::kill` and `do_group_move` clear it, and `Group::add` does
    /// not touch it at all. So a unit can name a group that does not
    /// list it and be listed by one it does not name, and every reader
    /// the original has goes through the pointer: the cap
    /// (`groups[+0x80].speed`), `do_group_move`'s first line,
    /// `Object::get_army` and `Group::normalize`'s prune.
    pub group_ptr: Option<u8>,
    /// `ObjectData::near_o`/`near_who` (+0x34/+0x36) — the **nearest**
    /// candidate the last search saw, which is not the one it chose
    /// (`docs/COMBAT.md` §37.1).
    ///
    /// `Object::find_nearby_target@00648da0` writes it for every candidate
    /// that clears `check_target` and is nearer than the best so far,
    /// **above and independent of** the `max_dist` gate and of the score,
    /// and clears it to `-1` when the nearest one it saw is beyond
    /// `0xf00`. So it is the search's footprint rather than its answer, it
    /// survives untouched between searches, and two arms read it as a
    /// cached incumbent: `do_move`'s captain retarget (§37.2) and
    /// `Unit::think`'s 31-in-32 fast path (§37.5, item 2049); this crate
    /// has both.
    pub near: Option<combat::Obj>,
    /// `unit_masks & 0x800000`: may auto-transport (`docs/TRANSPORT.md`
    /// §3). Granted at birth under the leader's level and by
    /// `check_transport`; the player's toggle writes it too.
    pub auto_transport: bool,
    /// `unit_masks2 & 0x2000`: a scenario's per-unit veto (§3.2).
    pub never_transport: bool,
    /// `guy_flags & 0x20`, which collapses the idle roll to two variants.
    /// `Unit::set_in_danger` raises it with [`Unit::in_danger`], and
    /// `Guy::process` drops it on a standing guy's sixty-fourth frame
    /// (item 530, `docs/ORDERS.md` §22). **It is guy 0's**:
    /// `set_in_danger` sets it on figures `0 .. guy_mark` alone, so a crew
    /// figure never carries it (`docs/ANIM.md` §13).
    pub guy_flag_0x20: bool,
    /// An animal's herd, an index into [`gaia::Gaia::herds`] — the
    /// `UnitData+0x86` union for an `Animal`. `None` for everything else
    /// and for a herdless animal.
    pub herd: Option<usize>,
    /// A pasture's animal — the farm it belongs to and its place in the
    /// five (`docs/SYNC.md` §3.6). `None` for everything else, which is
    /// what sends a herdless animal down `think_farm_animal`'s dead end.
    pub farm_animal: Option<crate::cities::farms::FarmAnimal>,
    /// The type's `TypeIndex`, when the loader named it; −1 otherwise. The
    /// birds' walk coin reads it.
    pub type_index: i32,
    /// `UnitData +0xaa`: the formation index the last group move applied,
    /// −1 none. `GroupData::get_form` reads it back (`docs/GROUPS.md`
    /// §4.4, §6.6).
    pub form: i8,
    /// `UnitData +0xab`: the twin of [`Self::form`] — the formation
    /// *width* the last group move applied, −1 none.
    /// `GroupData::get_form_mod_option` averages it over the members and
    /// `Form::compute` scales the column count by the average, so the two
    /// bytes together are how a formation remembers its own shape
    /// (`docs/GROUPS.md` §6.4, §6.6).
    pub form_width: i8,
    /// `UnitData::collide` — how many frames running this unit has been
    /// blocked. `Unit::do_idle` zeroes it; `detect_unit_collision` ages it
    /// out five frames after the last one (`docs/COLLISION.md` §4, §6).
    pub collide: i16,
    /// `UnitData::collide_frame`.
    pub collide_frame: i64,
    /// Where this unit's collision disc stands in [`Sim::coll`] — **guy
    /// 0's unit cell as the occupancy last saw it**, not the unit's
    /// (`docs/COLLISION.md` §16). `CollCheck::move_unit` is called only
    /// from `Guy::set_new_location`, so the bits follow the figure: a
    /// unit moved by someone else's turn (a push) keeps its disc on the
    /// old cell until its own `Guy::process` next runs. `None` while it
    /// is not painted.
    pub coll_at: Option<Pos>,
    /// `UnitData::collide_o` / `collide_who` / `collide_guy`: what is in
    /// the way, `-1` for nothing.
    pub collide_o: i16,
    pub collide_who: i8,
    pub collide_guy: i16,
    /// `UnitData::safe` (`+0xb2`): frames left of the collision cooldown a
    /// failed 48-grid search buys. `Unit::work` counts it down and
    /// `detect_unit_collision` refuses to test while it is set.
    pub safe: i32,
    /// `unit_masks & 0x40`: waiting for the unit in front to move.
    pub waiting_on: bool,
    /// `unit_masks & 0x100000`: the one-shot half step a soft collision
    /// asks for (`docs/MOVEMENT.md`, `docs/COLLISION.md` §4.3).
    pub half_step: bool,
    /// **`unit_masks & 0x10`, struck while moving in contact**
    /// (`Unit::target_opportunity@005fffc0`, `6002c8`): a unit hit by a
    /// unit while its front order is a move, with the attacker inside the
    /// two types' `big_radius` (`+0x244`) summed — unless its own action
    /// is an `ATTACK` whose target is inside twice that. `Object::do_damage`
    /// doubles the blow that raised it and every later one before it
    /// clears (`0064a5dd`), `UnitData::get_speed@00608720` halves a land
    /// unit's speed under it, and the end of `Unit::work@0060d180:449`
    /// clears it — on the paths that reach the end. The dump prints it on
    /// a unit struck after its own work ran (`docs/AI.md` §309).
    pub contact: bool,
    /// `ObjectData::down` / `up`: this unit's place in its world cell's
    /// object chain, newest first (`docs/COLLISION.md` §3).
    pub down: Option<usize>,
    pub up: Option<usize>,
    /// **`ObjectData`'s four chain words**, held on the whole chain of
    /// objects — buildings and a blocking wreck on it too — where
    /// [`Unit::down`]/[`Unit::up`] are the units of it every reader walks
    /// (`docs/COLLISION.md` §3.2, item 4098).
    pub chain: crate::movement::collide::ObjectChain,
    /// `UnitData::rare` (`+0x54`, the union with `air_alt` and
    /// `former_type`): the `TypeIndex` of the good `UnitData::calc_gather`
    /// last found for this unit, `-1` when it looked and found none.
    /// `Unit::init@00612100:90` starts it at **0**, so a unit that has
    /// never asked is not the same as one that asked and failed
    /// (`docs/ORDERS.md` §6.10).
    pub rare: i32,
    /// `UnitData::good_obj` (`+0x94`): the `circle_x`/`circle_y` index, in
    /// tiles from the unit's own, that `calc_gather` found that good at —
    /// the one index its next call retries before walking the spiral
    /// again. `Unit::init:281` starts it at `-1`.
    pub good_obj: i16,
    /// `unit_masks & 0x20`: `calc_gather`'s "and I may go on gathering
    /// here", which `Unit::do_gather@005fce20` records and `think_fish`
    /// clears. Both of the original's gameplay callers pass `param_7 = 1`,
    /// which is the arm that never sets it, so nothing here has ever seen
    /// it true — and no `unit_masks` in any capture on disk carries the bit
    /// (`docs/ORDERS.md` §6.10).
    pub gather_here: bool,
    /// `ObjectData + 0x8 & 8`: "this unit has gone idle and nothing has
    /// given it an order since". `Unit::check_idle` sets it and
    /// `Unit::work` clears it, and for a **fisherman or a merchant** each
    /// of those two transitions marks the owner's economy dirty — because
    /// each of them changes who `Leader::calc_gather` step 6 will walk
    /// ([`crate::economy::rares`]). It is the reason a boat that finishes its
    /// journey has its deposit counted within eight frames rather than at
    /// the next 512-frame refresh.
    pub idle_latch: bool,
    /// **`ObjectData::visible` (`+0x40`)** — the byte of players this
    /// object has made itself visible to *by attacking them*, one bit per
    /// `who`. `docs/VISION.md` §7.
    ///
    /// It is the fallback arm of `UnitData::is_seen`: a unit standing in a
    /// player's fog is still a legal target for that player while its bit
    /// is set, which is how a shooter that nobody can see still gets shot
    /// back at. [`Sim::set_attacking`] is the only writer here;
    /// [`Sim::work`] clears it.
    pub visible: u8,
    /// **`UnitData::unit_masks & 4`** — the "in danger" latch
    /// `Unit::set_in_danger@005fcfb0` raises and `Unit::work@0060d180`
    /// drops on the unit's own 32-frame tick (`0060d19d`, above every
    /// gate — the same tick that clears [`Unit::visible`]).
    ///
    /// Its one reader here is `Group::action_move_near`'s §6.6 step 6:
    /// a member carrying it takes a **plain** move where the rest of the
    /// group takes a `GroupMoveOrder`. `Unit::go_to_unit@005f78c0` sets
    /// it over the whole squad it is about to walk, one line before the
    /// walk, which is why no `go_to` group has ever marched in formation.
    ///
    /// The other two writers go through `Unit::set_in_danger`, which
    /// marks the whole squad and its guys ([`Sim::set_in_danger`]):
    /// `Object::take_damage@00652020` marks a unit that is hit, and
    /// `Unit::work` marks an attacker and its target every frame the
    /// action is an `ATTACK` (item 530, `docs/ORDERS.md` §22). Nothing
    /// else in the engine reads the bit.
    pub in_danger: bool,
    /// **`SubObjectData::flags & 0x80`** — "attacked this frame, or is
    /// still on an `ATTACK` order".
    ///
    /// `Unit::set_attacking@005ff5b0` raises it on every strike and
    /// `Unit::work@0060d180` drops it on any frame the front order is not
    /// `ATTACK`. Its only job is to hold [`Unit::visible`] against the
    /// 32-frame clear at the head of [`Sim::work`], which is why the bit
    /// survives about a third of a second past the last arrow and the
    /// visibility survives up to 32 frames past that.
    pub attacking: bool,
    /// `unit_masks & 0x1000`, "blown" (`UnitData::is_blown@0060a6e0`): a
    /// stealth unit that has worked an order is no longer hidden by
    /// `is_seen`'s stealth arm. Set by `Unit::work@0060d180:386` (an order,
    /// a `STEALTH`-not-`CLOAKED` type) and by every `|= 0x11000`; dropped
    /// by `check_idle@006032c0` once the idle counter reaches 10, and only
    /// while [`Unit::fought`] is clear (item 2785, `docs/GOLDEN.md` §103).
    pub blown: bool,
    /// `unit_masks & 0x10000`: `fight@005fd4d0:580`, `do_attack_ground` and
    /// `do_cast` write `0x11000` together; nothing this crate reaches
    /// clears the upper bit, so `check_idle` never drops `blown` after.
    pub fought: bool,
    /// `unit_masks & 0x800`, the cloak `UnitData::is_seen@00607a60` and
    /// `is_cloaked@0060a6a0` read: `Unit::process_cloak@005e1c20`'s
    /// Iroquois forest arm writes it (`docs/AI.md` §402).
    pub cloak: bool,
}

/// What a unit needs in order to move.
///
/// `speed` and `turning` are **inputs**, not computed here. The speed is the
/// output of `UnitData::get_speed`'s three-layer pipeline, which reads tech,
/// nation, hero and terrain state the simulation does not model yet, and the
/// turning facts come from the unit type. Taking them as inputs is the same
/// choice supply made for a general's aura radius: the mechanic is complete,
/// and the number feeding it arrives when the layer that produces it does.
/// `docs/MOVEMENT.md` documents the pipeline in full.
///
/// The rest is state the original keeps too: the unit's position is on
/// [`Unit`], and the body — the figure that chases it, whose `last_speed` and
/// `avg_speed` the unit's turn rate reads — is here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Movement {
    /// Which way the unit faces — **`GuyData::angle` (`+0x18`)**, guy 0's own
    /// facing, which the unit step turns by the rate and steps along. North is
    /// zero.
    pub facing: movement::Angle,
    /// The facing as the body's last `Guy::process` left it — what the
    /// body follow compares against to tell whether anything has turned
    /// guy 0 since (`guy_flags & 2`: set by every `Guy::do_turn` that
    /// moves the angle, a push by another unit included, and cleared only
    /// after `Guy::move`; item 3571).
    pub frame_facing: movement::Angle,
    /// Where it *wants* to face: **`UnitData::angle` (`+0x50`)**, which is
    /// also **`GuyData::des_angle` (`+0x64`)** — `Unit::set_angle` writes the
    /// two together, and `Unit::move_step` calls it with the bearing to the
    /// destination on every frame, before turning. So this is the heading, not
    /// the facing: it is what the dump's `UNITDATA angle` line carries, what
    /// `Group::update_positions` rotates a slot table by, and what an idle
    /// body turns toward.
    pub heading: movement::Angle,
    /// **`UnitData::dest_angle` (`+0x58`)**, which is a different field and a
    /// different thing: the angle the *order* wants the unit to end up facing.
    /// `Unit::update_action` seeds it from [`Movement::heading`] and then
    /// overwrites it with the angle of the last transit move it walks past
    /// (`docs/ORDERS.md` §3.3). Nothing in the step reads it.
    pub des_angle: movement::Angle,
    /// Where it is headed. `None` means it is not going anywhere.
    pub dest: Option<Pos>,
    /// Effective speed, in position units per frame. Used for both the unit
    /// step and the body's chase; the body's own `+9` and ungrouped speed are
    /// not modelled.
    pub speed: i32,
    /// The type-level facts behind the turn rate and turn limits.
    pub turning: movement::Turning,
    /// The body, standing on the unit until the unit moves.
    pub body: movement::Body,
    /// **`UnitData::unit_masks & 2`** — the unit's own mirror, the per-unit
    /// twin of `GroupData::facing` (`docs/GROUPS.md` §25). Two writers, by
    /// the listing: `Unit::set_angle@00605400` flips it whenever the new
    /// heading is `reversing` from the old (`605424`, for every unit, before
    /// the leader test that flips the group), and
    /// `Unit::kill_current_order@005e2cb0` writes a dying move's handed-back
    /// mirror into it (`5e3087`/`5e308d`, again before the leader test).
    /// One reader here: `Unit::do_guard@005e5c70` negates a guard's `dx`
    /// when its **target** carries it.
    pub mirror: bool,
    /// **The trackless crew's `des_angle` (`GuyData +0x64`) stands apart
    /// from its `angle` (`+0x18`)** — set by an order's `Unit::set_angle`
    /// with the snap flag clear ([`Sim::unit_set_angle`]), cleared by the
    /// next `Guy::do_turn`, by guy 0's `Guy::move` and by the snap (item
    /// 3762, `docs/GOLDEN.md` §149.3). `Unit::move_step`'s own `set_angle`
    /// is not counted: its `do_turn` follows it in the same step. It holds
    /// the frame it was written on, because guy 0's `Guy::move` turns the
    /// crew on every frame its heading is not its facing — whatever order
    /// the unit stands on — and this crate runs no body follow for some
    /// of them (a packer's unpack, the Bombard `1/94` of run1254 on 4428). `Guy::set_angle@005d9010`
    /// writes every crew figure's `+0x64` from `+0x304` up and leaves its
    /// `+0x18`; `Guy::do_turn@005d97a0` writes both equal on the figures
    /// it recurses into. Between the two, a trackless crew figure reads
    /// `des_angle != angle` exactly when guy 0's heading is not its facing
    /// — and `Guy::set_anim`'s attack test (`005da38a`–`005da3ba`) defers
    /// its swing. run1413's Bombard `2/23` on 759: `do_attack_ground`
    /// turns it and swings on one frame, and the two trackless crew pay
    /// the swing in `Guy::move+0x166`.
    pub crew_des: Option<i64>,
}

impl Movement {
    /// A unit standing at `pos`, with its body on it and stopped — which, for
    /// a foot or mounted type, is what lets the first order turn it
    /// instantly. All three angles are `Unit::init`'s own
    /// [`movement::Angle::INITIAL`], not north.
    pub const fn at(pos: Pos) -> Movement {
        Movement {
            facing: movement::Angle::INITIAL,
            frame_facing: movement::Angle::INITIAL,
            heading: movement::Angle::INITIAL,
            des_angle: movement::Angle::INITIAL,
            dest: None,
            speed: 0,
            turning: movement::Turning {
                type_turn_speed: 0,
                packed: false,
                instant_from_stop: false,
                wide_limit: false,
            },
            body: movement::Body::at(pos),
            mirror: false,
            crew_des: None,
        }
    }

    /// `Unit::set_angle`'s mirror flip (`605410`–`605424`): the new
    /// heading against the old `+0x50`, by `reversing`'s inclusive window.
    const fn flip_mirror(&mut self, to: movement::Angle) {
        let d = to.0.wrapping_sub(self.heading.0) as u32;
        if d >= 0x4000_0000 && d <= 0xc000_0000 {
            self.mirror = !self.mirror;
        }
    }

    /// Points the unit and its body a given way at once — `Guy::set_angle`
    /// with the snap flag, the one caller that moves the facing and the
    /// heading together. `Unit::move_step` never uses it: its `set_angle`
    /// passes zero for that flag, so a step moves the heading and leaves the
    /// facing to the turn rate.
    ///
    /// `Guy::set_angle` is not `Guy::do_turn`, so it leaves `guy_flags & 2`
    /// as it found it: the snap is not a turn the body's own `Guy::move`
    /// skips for, and [`Movement::frame_facing`] moves with the facing.
    pub const fn set_facing(&mut self, facing: movement::Angle) {
        self.flip_mirror(facing);
        self.facing = facing;
        self.frame_facing = facing;
        self.heading = facing;
        self.des_angle = facing;
        self.crew_des = None;
    }

    /// Points the unit a given way and **leaves the body to turn** —
    /// `Unit::set_angle(a, a, 0)`, which is what every order in the
    /// ordinary path calls. `Unit::set_angle@00605400` writes the unit's
    /// own `+0x50` and then `Guy::set_angle@005d9010`, whose snap flag is
    /// that third argument: with it zero the guy takes `des_angle`
    /// (`+0x64`) alone and its `angle` (`+0x18`) stands.
    ///
    /// One frame, and it is visible: a body owed a turn is a body
    /// `Guy::move` finds at its destination with `des_angle != angle`,
    /// which is the **turn arm** — `set_anim(CHAR_WALK, 0, 1)`, on top of
    /// whatever the order just asked for. So the work animation an order
    /// sets on the frame it also turns the unit is overwritten by the walk
    /// for that frame and takes hold on the next. Run44's citizen `0/2` is
    /// the record: at the camp on frame 542 its `wait` steps 32 → 31 and
    /// its guy plays `26 1/15`, the carrying walk restarted; `27 1/32`,
    /// the wood dump, is frame 543 (`docs/ANIM.md` §4.7).
    ///
    /// The snapping [`Movement::set_facing`] is the other arm — the flag
    /// set — and its callers are `do_move`'s re-face, `go_inside`,
    /// `do_spec_anim` and the scenario loader, none of which this crate
    /// reaches yet.
    pub const fn set_heading(&mut self, heading: movement::Angle) {
        self.flip_mirror(heading);
        self.heading = heading;
    }
}

/// The type-level facts the turn rate and the turn-in-place limits read —
/// `movement::Turning` as `Guy::init_real` and `Unit::move_step` derive them.
///
/// **`instant_from_stop` is `guy_flags & 0x10`**, and `Guy::init_real` builds
/// it out of four type tests: not a packing type (`unit_flags2 & 4`), carrying
/// objmask `FOOT` or `MOUNTED` or the program-set `unit_flags & 0x10`, and
/// **not** `unit_flags & 2` — the `FLAGS b` whose legend is "Unit is a
/// horse-drawn cart type thing". A cart does not pivot. That is the whole of
/// `unitrules.xml`'s own note, "Foot & Mounted units turn instantly from a
/// stopped position", and with a stopped body — which is every frame the unit
/// did not move — it makes the next step's turn free.
///
/// `wide_limit` is `move_step`'s 80° branch: a non-land type, or a `VEHICLE`,
/// keeps walking while it still owes up to 80° — but only two tiles out or
/// more.
///
/// `packed` is not here: it is `unit_masks & 0x80000`, a runtime state.
pub fn turning_of(t: &UnitType) -> movement::Turning {
    let objmask = t.combat.obj_masks;
    let foot_or_mounted = objmask & (combat::mask::FOOT | combat::mask::MOUNTED) != 0;
    movement::Turning {
        type_turn_speed: t.turn_speed,
        packed: false,
        instant_from_stop: !t.cols.flag2(crate::ai::ai_load::uflags2::PACKS)
            && (foot_or_mounted || t.cols.flag(crate::ai::ai_load::uflags::FLAG_TRANSPORT))
            && !t.cols.flag(crate::ai::ai_load::uflags::CART),
        wide_limit: t.combat.domain != crate::territory::attrition::Domain::Land
            || objmask & combat::mask::VEHICLE != 0,
    }
}

/// A newborn's formation byte (`UnitData +0xaa`, the dump's `form`) —
/// `Unit::init@00612100:81`–`86`: **9** for the four civilian ids,
/// `PEASANTS` to `SCHOLARSKOREAN` (`0x32`–`0x35`, the type's `+4`), and
/// **0** for every other type. Its width twin (`+0xab`) is born −1, which
/// [`Unit::new`] already holds.
///
/// `Sim::group_action_move_to` reads it through `get_form`, and form 9
/// is one of the arms that keep a formation's member off the
/// `GroupMoveOrder`: two Citizens sent together walk two plain moves in the
/// original. Born −1 here, a group of Citizens took form 0 and one group
/// move (item 1330, chapter forty-seven's 838; parked 646).
pub fn init_form(type_index: i32) -> i8 {
    if (0x32..=0x35).contains(&type_index) {
        9
    } else {
        0
    }
}

/// A unit type the simulation knows how to build.
///
/// The price is the data's, the kind is what attrition and supply read, and
/// the group is which production building's output counts it — `None` for a
/// type without attack, which is every civilian in the shipped data. A type
/// whose progression counts by type is still counted in its group; only a
/// by-group progression reads the group back (`cost::Counts::of_group`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UnitType {
    pub price: crate::economy::cost::Price,
    pub kind: crate::territory::attrition::UnitKind,
    pub group: Option<usize>,
    /// The `HITS` column: what a unit of this type is built with.
    pub hits: i32,
    /// The time half of the same record — `JOB_TIME`, `JOB_EXTRA_TIME` and
    /// `RESEARCH_PREMIUM_TIME`. See `docs/PRODUCTION.md`.
    pub times: crate::cities::production::Times,
    /// The combat columns and derived bits — `docs/COMBAT.md` §2.1. A
    /// default profile has zero attack and is neither target nor threat.
    pub combat: combat::Profile,
    /// This type's entry in [`Sim::tech_tree`], if it has one. With it,
    /// queueing is gated by `type_avail` and the first completion goes
    /// through `gain_tech` — predecessors become owned and obsolete, free
    /// units and buildings cascade — instead of only setting the bit. Without
    /// it the type is outside the tree, as every type was before the tree
    /// existed, and [`Muster::researched`] is the whole story.
    pub tree: Option<tech::TypeId>,
    /// What the type needs in order to garrison — `docs/CITIES.md` §6.4.
    pub garrison: crate::units::garrison::UnitTraits,
    /// `MOVES`: the type's base speed in position units per frame — the
    /// number `UnitData::get_speed` starts from, before the nation, tech and
    /// terrain layers, and already in the unit movement uses
    /// (`docs/MOVEMENT.md` §1: `UNIT_MOVE_SPEED` is the identity converter).
    pub moves: i32,
    /// `TURN_SPEED`, through `degrees_to_angle` as `UnitType::init` stores
    /// it — the `type_turn_speed` of [`movement::Turning`].
    pub turn_speed: i32,
    /// `LOS`, `ObjectTypeData +0x21c` — line of sight **in tiles**, which
    /// `Object::update_seen` halves to get a fog radius (`docs/VISION.md`
    /// §3). Zero means the type reveals nothing at all.
    pub los: i32,
    /// `SCIENCE_LOS`, `+0x220` — added once per science level.
    pub science_los: i32,
    /// The type's own `TypeIndex` — `0x32 + record`, the number the dump's
    /// `GUY.type` carries and several predicates test by identity.
    pub type_index: i32,
    /// Which worker kind this is — a citizen or scholar may gather and is
    /// what `think_peasant` runs for (`docs/ORDERS.md` §6.1).
    pub worker: orders::Worker,
    /// The words no column carries — `unit_flags`, `unit_flags2`, `CAT`,
    /// `CARRY` and the `role` the loader derives from them. Every producer
    /// reads them; `docs/DATALAYER.md`, "The derived words no column
    /// carries", is the derivation and [`ai_load`] the code.
    /// The two lineage roots `ObjectData::train_time`'s **British** arm
    /// tests, precomputed the way [`ai_load::RoleFacts`]' are: `is(0xaa)`
    /// — the Bowmen root, which is the Archers line — and `is(0x119)`, the
    /// Anti-Aircraft Gun. `docs/PRODUCTION.md`, "The tail".
    pub archer: bool,
    pub anti_air: bool,
    pub cols: crate::ai::ai_load::UnitCols,
    /// `MANA` (`UnitTypeData +0x2ec`), the pool `UnitData::mana@00609a50`
    /// reads for a caster. SEAM: its supply-upgrade, French and space arms.
    pub mana: i32,
    /// One of the twelve gaia types (`BASE_GAIATYPES..END_GAIATYPES`, record
    /// `352..`). The animals are unit types with a `WHERE` of Large City, and
    /// several of the original's loops stop before them.
    pub gaia: bool,
    /// What [`relative_value::relative_value`] reads of the type, fixed at
    /// load as `UnitType::fill_relative_values@0061be50`'s table is.
    pub rel: crate::ai::relative_value::Facts,
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
    /// And the queued half of it — `LeaderData`'s `barracks_queued`,
    /// `stable_queued`, `factory_queued`, `dock_queued`, `air_queued`.
    ///
    /// `LeaderData::get_support_count@006da110` returns
    /// `<group>_queued + <group>_units` for a type whose `PROGRESSION` is
    /// by-group, exactly as it returns `num_queued + num_units` for one
    /// that is by-type. Without this half the ramp does not advance
    /// *within* a batch: `Leader::produce_unit` queues its units one at a
    /// time and each is priced from the count, so two Longbowmen ordered
    /// together both paid the first one's price (`docs/COSTS.md`, "The
    /// count").
    pub queued_by_group: Vec<i32>,
    /// Population occupied — `LeaderData::control`.
    pub control: i32,
    /// The cap it is measured against. Recomputed from scratch by
    /// [`Sim::recompute_pop_caps`] rather than maintained incrementally,
    /// because `Leader::calc_pop_cap` recomputes it from scratch too.
    pub cap: i32,
    /// The player's Military library level, `epoch[0]` — how many of the seven
    /// Military techs they hold, from The Art of War up. It indexes `POP_CAP`,
    /// and it is not the age. An input until there is a tech layer to produce
    /// it.
    pub military_level: usize,
    /// The lobby's population setting. The shipped choices are 50, 75, 100,
    /// 125, 150 and 200, and the largest is exactly `POP_CAP[7]`.
    pub limit: i32,
    /// The wonders, nations and scenario overrides that move the cap.
    pub bonuses: crate::economy::cost::PopBonuses,
}

/// The largest population the lobby offers, and this simulation's default.
pub const DEFAULT_POP_LIMIT: i32 = 200;

impl Muster {
    pub(crate) fn new(t: &Tuning) -> Muster {
        let mut m = Muster {
            limit: DEFAULT_POP_LIMIT,
            ..Muster::default()
        };
        m.cap = crate::economy::cost::pop_cap(t, m.military_level, m.limit, &m.bonuses);
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
            hold_frames: 0,
            search: None,
            // `Unit::init@00612100:567` zeroes the stamp beside the five
            // container pointers: a unit's search state starts empty.
            start_dist: 0,
            damage_frac: 0,
            squad_size: 1,
            kind: crate::territory::attrition::UnitKind::default(),
            on_map: true,
            supply_slot: None,
            caravan: None,
            attrition: 0,
            num_queued: 0,
            queue_time: 0,
            ignores_supply: false,
            sheltered: false,
            movement: Movement::at(pos),
            ty: None,
            combat: combat::State {
                captain: i32::from(index),
                ..combat::State::default()
            },
            inside: None,
            inside_unit: None,
            max_health: health,
            orders: std::collections::VecDeque::new(),
            path: Vec::new(),
            tolerance: 0,
            line_ok: false,
            path_recursion: 0,
            orders_pos: pos,
            idle: 0,
            stance: 1,
            spell_time: 0,
            mana_burn: 0,
            full: 0,
            waiting: 0,
            healing: 0,
            cast_target: None,
            cavarch_who: 0,
            cavarch_idle: false,
            head_serial: HeadSerial(0),
            casting: false,
            marching: None,
            ambushing: None,
            infiltrated: 0,
            airframe: air::Airframe::default(),
            was_builder: false,
            uid: 0,
            birth_cell: Pos::new(
                pos.x.div_euclid(48) * 48 + 24,
                pos.y.div_euclid(48) * 48 + 24,
            ),
            carry: 0,
            cant_reach: false,
            decoy: false,
            kremlin_spy: false,
            mylos: 0,
            unit_masks2: 0,
            avoid: None,
            guys: Vec::new(),
            captain: true,
            o_up: None,
            o_down: None,
            group_ptr: None,
            near: None,
            auto_transport: false,
            never_transport: false,
            guy_flag_0x20: false,
            herd: None,
            farm_animal: None,
            type_index: -1,
            form: -1,
            form_width: -1,
            collide: 0,
            collide_frame: 0,
            coll_at: None,
            collide_o: -1,
            collide_who: -1,
            collide_guy: -1,
            safe: 0,
            waiting_on: false,
            half_step: false,
            contact: false,
            down: None,
            up: None,
            chain: crate::movement::collide::ObjectChain::default(),
            // `Unit::init@00612100` lines 90 and 281: `rare` is **0** and
            // `good_obj` is `-1`, and the two starts are not the same
            // sentinel.
            rare: 0,
            good_obj: -1,
            gather_here: false,
            idle_latch: false,
            // `Object::init@00647750:25` zeroes `visible` and
            // `launch_frames` together as one short.
            visible: 0,
            in_danger: false,
            attacking: false,
            blown: false,
            fought: false,
            cloak: false,
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

/// `Objects::find_free@0065ad60`'s reuse test on one number (`docs/COMBAT.md` §59.2): the
/// object is dead (`flags & 1` clear), its `hold_frames` (`+0x32`) is zero, and it is either no
/// unit (vslot `+0x18` answers 0) or a captain (`UnitData +0x8e` negative).
pub(crate) fn find_free_reusable(alive: bool, hold: i16, is_unit: bool, captain: bool) -> bool {
    !alive && hold == 0 && (!is_unit || captain)
}

/// `Objects::find_free@0065ad60` with `param_5 < 0`: the first number of `[base, *mark)` that
/// `reusable` accepts, tested in order; failing that `*mark` itself, which moves up one, unless
/// it has reached `limit` (then -1).
pub(crate) fn find_free_steps(
    base: i32,
    limit: i32,
    mark: &mut i32,
    mut reusable: impl FnMut(i32) -> bool,
) -> i32 {
    for i in base..*mark {
        if reusable(i) {
            return i;
        }
    }
    if limit <= *mark {
        return -1;
    }
    *mark += 1;
    *mark - 1
}

/// The high-water marks of a player's object array — `Objects::mark[who]`
/// for each of the three bands `find_free` allocates from. Object numbers
/// (`SubObjectData::o`) are per player: units from 0, buildings from 2000,
/// walls from 3000 (`docs/CITIES.md` §1, `docs/INPUT.md` §2). A mark only
/// ever grows; a freed slot below it is reused by [`Sim::find_free`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Marks {
    pub unit: i16,
    pub build: i16,
    /// `Objects +0x1d4 + 2 who`: the uid the player's next object takes —
    /// `Object::init@00647750` writes it to `ObjectData::uid` and adds one.
    /// [`crate::Sim::next_uid`] is the one writer (item 4094).
    pub uid: u16,
}

impl Default for Marks {
    fn default() -> Marks {
        Marks {
            unit: UNIT_BASE,
            build: BUILD_BASE,
            uid: 0,
        }
    }
}

/// `find_free(who, 0, 2000, …)`: the unit band.
pub const UNIT_BASE: i16 = 0;

/// `find_free(who, 2000, 3000, …)`: the building band.
pub const BUILD_BASE: i16 = 2000;

/// Where the building band ends and the wall band begins.
pub const WALL_BASE: i16 = 3000;

/// `UnitData::curr_uber_size@0060a760`'s two walks: up the `o_up` links to
/// the captain (`up` answers `None` there — the original's `is_captain`,
/// `o_up`'s sign bit), then the captain and each `o_down` link whose figure
/// is alive (`down` answers `None` past the last), counted. Both walks stop
/// after sixteen links, a guard the original does not have. Item 2247's
/// sweep holds it.
pub(crate) fn squad_size(
    mut i: usize,
    up: impl Fn(usize) -> Option<usize>,
    down: impl Fn(usize) -> Option<usize>,
) -> i32 {
    let mut guard = 0;
    while let Some(u) = up(i) {
        i = u;
        guard += 1;
        if guard > 16 {
            break;
        }
    }
    let (mut n, mut cur, mut guard) = (1, down(i), 0);
    while let Some(d) = cur {
        n += 1;
        cur = down(d);
        guard += 1;
        if guard > 16 {
            break;
        }
    }
    n
}

/// The figures a squad walk reaches, in its order — **`Unit::set_in_danger@005fcfb0`**
/// and **`Unit::set_group@00605220`** walk alike: the climb up `up` (the
/// original's `o_up`, `None` at the captain — its `is_captain`, the link's
/// sign bit) from `start`, then the captain and each `down` link (the
/// original's `o_down` whose slot is active, `flags & 1`; `None` past the
/// last). Both walks stop after `limit` links, a guard the original does
/// not have: a cycle would hang it. Item 2276's sweeps hold it.
pub(crate) fn squad_walk(
    start: usize,
    limit: usize,
    up: impl Fn(usize) -> Option<usize>,
    down: impl Fn(usize) -> Option<usize>,
) -> Vec<usize> {
    let mut at = start;
    for _ in 0..limit {
        let Some(u) = up(at) else { break };
        at = u;
    }
    let mut out = vec![at];
    for _ in 0..limit {
        let Some(d) = down(at) else { break };
        out.push(d);
        at = d;
    }
    out
}
