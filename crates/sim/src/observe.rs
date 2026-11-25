//! **The read door** (`docs/STRUCTURE.md` §5 move 9, `docs/RENDERER.md`
//! §2.3, item 4017): what a client may see of a running game, as an owned
//! value it can keep past the tick.
//!
//! [`Sim::observe`] copies the objects a renderer draws into a [`Scene`];
//! [`Sim::observe_terrain`] and [`Sim::observe_territory`] copy the two grids
//! that change rarely, and the scene's version counters say when to fetch
//! them again. Four rules hold here, and RENDERER §2.3 states them:
//!
//! 1. Nothing in it is a float or a renderer type.
//! 2. An object is keyed by the original's identity, `(who, o)` — never by
//!    its slot in a `Vec`.
//! 3. It is read-only by type: every function takes `&self`, draws nothing
//!    from `rng` and writes nothing. `soak`'s
//!    `observing_every_tick_leaves_the_digest_alone` is the check.
//! 4. It reports and does not interpret: no culling, no draw order, no
//!    "visible on screen".

use crate::combat::Obj;
use crate::movement::Angle;
use crate::territory::attrition::Domain;
use crate::{Owner, Player, Pos, Sim};

/// An object's identity as the original keeps it: its owner and its number
/// in the owner's object array (`SubObjectData::o`), the pair a dump prints
/// and the harness links on. Units are numbered from 0 and buildings from
/// 2000. Whether a number is reused after a death inside one game is not
/// established (`docs/RENDERER.md` §7), so a client should not assume an
/// `ObjId` names one object for the whole game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjId {
    pub who: Player,
    pub o: i16,
}

/// One tick's drawable state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scene {
    /// The frame count, [`Sim::frame`].
    pub frame: i64,
    /// Every living unit on the map, in the simulation's store order. A
    /// unit at no health keeps its record here until it is reused, but the
    /// original has taken it out of its object array — a corpse is drawn
    /// from [`Scene::deaths`] (item 4026).
    pub units: Vec<UnitScene>,
    /// Every building in use, in the store order.
    pub buildings: Vec<BuildingScene>,
    /// **Every unit inside a building**, building by building in the
    /// store order and each building's chain in the order its squads
    /// entered (item 4280, `docs/CLIENT_INPUT.md` §26): one icon each on
    /// the original's building panel, whose press is
    /// `Options::exec@007188c0` case `0xaa`. A read only.
    pub garrisons: Vec<GarrisonScene>,
    /// The projectiles in flight.
    pub projectiles: Vec<ProjectileScene>,
    /// The corpses fading.
    pub deaths: Vec<DeathScene>,
    /// Each player slot's leader record, in slot order: what the HUD's bar
    /// prints (item 4198, `docs/HUD.md` §4).
    pub leaders: Vec<LeaderScene>,
    /// The viewer's fog plane on the half-cell grid (`fog_xs × fog_ys`,
    /// row-major): one of the four levels `WorldMapFog::fill_colors_fog`
    /// writes ([`fog::UNSEEN`], [`fog::REGION`], [`fog::EXPLORED`],
    /// [`fog::VISIBLE`]; item 4624, `docs/RENDERER.md` §3.32.1). `None` for
    /// an observer who sees everything, and on a world with no fog grid
    /// installed.
    pub fog: Option<Vec<u8>>,
    /// Moves when the terrain grids change: fetch
    /// [`Sim::observe_terrain`] again.
    pub terrain_version: u32,
    /// **`GameAccess::ai_off`**, [`crate::ai::AiState::ai_off`]: what the
    /// console's `ai on` and `ai off` read before they issue the toggle
    /// (`ConsoleWin::run_cmd@007d6a70` case `0xc`; item 4283,
    /// `docs/CLIENT_INPUT.md` §28). A read only.
    pub ai_off: bool,
    /// Moves when a cell's claimants change: fetch
    /// [`Sim::observe_territory`] again.
    pub territory_version: u32,
}

/// **One player's leader record, as the original's resource bar and
/// population box read it** (item 4198, `docs/HUD.md` §4). A copy: the
/// door adds no state and no rule. The fields are the ones
/// `IFaceResources::draw_resource@00818840` and
/// `IFaceMainBase::draw_pop_cap@0080ff90` read off `LeaderData`, at the
/// scale the record stores them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderScene {
    /// The player slot, the record's `who`.
    pub who: Player,
    /// `LeaderData::tribe`, an index into the tech tree's tribes.
    pub tribe: usize,
    /// That tribe's `<TRIBE name>` from the install's tables, empty when
    /// unloaded: a copy of the table's entry, so a client needs no tables
    /// of its own to name the nation.
    pub nation: String,
    /// `LeaderDataEncrypt::ages` (`+0xdc`), the dump's `ages_get()`: the
    /// ages gained past the first, 0 in the Ancient age.
    pub ages: i32,
    /// `LeaderDataEncrypt::bucket` (`+0x0`), per good in the record's
    /// order (food, timber, wealth, knowledge, metal, oil): the stockpile,
    /// in whole units, the number `draw_resource` writes.
    pub bucket: [i32; 6],
    /// `LeaderDataEncrypt::income` (`+0x94`), in sixteenths: the rate
    /// `draw_resource` writes beside the stockpile as `(income + 8) / 16`.
    pub income: [i32; 6],
    /// `LeaderDataEncrypt::over_cap` (`+0x4c`) non-zero: the good is at its
    /// commerce cap, which `draw_resource` flashes.
    pub over_cap: [bool; 6],
    /// `LeaderData::control` (`+0x940`): the population occupied, the
    /// first number of `draw_pop_cap`'s `n/cap`.
    pub control: i32,
    /// `LeaderData::pop_cap` (`+0x7e4`): the cap it is measured against.
    pub pop_cap: i32,
    /// What a market's panel offers this leader, per good: the `(buy,
    /// sell)` price of each good it has a button for, `None` for the rest
    /// ([`Sim::market_offer`]; item 4259). A read only.
    pub market: [Option<(i32, i32)>; 6],
    /// The slots it is at war with, either way ([`Sim::is_enemy`],
    /// `LeaderData::is_enemy`), in slot order: what a right press's gather
    /// point on a building asks of the building's owner, action 2 against 1
    /// (item 4301, `docs/CLIENT_INPUT.md` §11.4). A read only.
    pub enemies: Vec<Player>,
}

/// **A craft a unit can be handed** (item 4264): its row and the two
/// flag bits the button and the pick read — `spell_flags & 0xe`, a craft
/// that takes a target (`Options::do_spell@0071d7a0`), and `& 8`, one
/// whose target is the ground (`Options::picked_spot@00721c40`'s
/// `OPTION_SPELL` arm).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Craft {
    pub t: i32,
    pub targeted: bool,
    pub ground: bool,
}

/// **What a right press's spy arms ask of an object** (item 4303,
/// `docs/CLIENT_INPUT.md` §37): `Console::execute_at_cursor@007c6630`'s
/// Scout-line and Spy arms, which issue a craft on the object under the
/// cursor. A read only; the default is an object of none of them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpyMarks {
    /// `is(SPY, 0)` (`0x3a`, vslot `+0xb8`): both arms' Counterintelligence
    /// on an enemy, and the Spy arm's `COUNT_TYPE` test on a member.
    pub spy: bool,
    /// `is(SCOUT, 0)` (`0x45`) and `is(COMMANDO, 0)` (`0x49`): the Scout-line
    /// arm's two `COUNT_TYPE` tests on a member.
    pub scout_line: bool,
    pub commando: bool,
    /// `ObjectData::is_peasant`: the Spy arm's Informer on a unit.
    pub peasant: bool,
    /// `has_objmask(0x200000)`, the vehicle mask (vslot `+0x148`): the
    /// Commando's Sabotage on a unit.
    pub vehicle: bool,
    /// `SpellTypeData::is_valid_target` of Bribe (`0x275`) and of the
    /// Sniper (`0x281`) for the viewer ([`Sim::spell_valid_target`]);
    /// `false` with no viewer.
    pub bribe: bool,
    pub sniper: bool,
    /// A building: `WorldData::get_whose` of the cell under it is the
    /// viewer, the Spy arm's Bribe on a building, its Informer otherwise.
    pub viewer_ground: bool,
}

/// One unit on the map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitScene {
    pub id: ObjId,
    pub owner: Player,
    /// **`ObjectData::uid` (`+0x30`)**, [`crate::Unit::uid`]: what
    /// `CommandPackage::add_group@0094bb60` compares beside each object's
    /// number before it writes a group equal to the last one sent as no
    /// objects (item 4267, `docs/CLIENT_INPUT.md` §20). A read only.
    pub uid: u16,
    /// An index into the unit types; `None` for the bare object.
    pub ty: Option<usize>,
    /// The type's `TypeIndex` (`ptype +4`), −1 when the loader named none:
    /// the key `SelectGroup::add@00716c10` sorts a selection by, and what
    /// `ObjectData::is_worker` and `UnitData::is_merchant` test (item
    /// 4180, `docs/CLIENT_INPUT.md` §7).
    pub type_index: i32,
    /// `UnitData +0xaa` and `+0xab`, [`crate::Unit::form`] and
    /// [`crate::Unit::form_width`]: what `GroupData::get_form` and
    /// `get_form_mod_option` read when a right press mints a group's move
    /// (`Console::down@007cd8f0`; item 4180).
    pub form: i8,
    pub form_width: i8,
    /// The unit's position, 1/768 of a cell.
    pub pos: Pos,
    /// The pinned tile height under it, [`crate::World::tile_z`].
    pub z: i32,
    /// `GuyData::angle`, guy 0's facing. North is zero.
    pub facing: Angle,
    pub health: i32,
    pub max_health: i32,
    /// `SubObjectData::is_seen` (vslot `+0x48`, named by the PDB's
    /// `SubObjectData` field list) of the viewer, `UnitData::is_seen@00607a60`
    /// as [`Sim::target_seen_by`] ports it; `true` when there is no viewer.
    /// `Console::find_target_daemon@007c5930` takes no object that answers
    /// no, so a right press on a unit the fog hides finds nothing (item 4202,
    /// `docs/CLIENT_INPUT.md` §10).
    pub seen: bool,
    /// `GroupData::count(COUNT_ATTACK)@00711720`'s member test: its type's
    /// `attack` (`+0x1e8`) is nonzero and it `is_captain` (`ObjectData`
    /// vslot `+0xe8`, named by the PDB's `ObjectData` field list). A
    /// selection none of whose members answers yes draws
    /// `Console::execute_at_cursor`'s follow or guard on a unit (item 4202).
    /// The count's other arm, a `TRANSPORTBARGE` carrying such a unit, is
    /// not carried here.
    pub attacks: bool,
    /// **`UnitData::get_captain` (vslot `+0xe4`)**, [`crate::combat::State::captain`]:
    /// the captain's unit index, its own for a lone figure. A right press
    /// on one's own unit with a building selected names it as a gather
    /// point's `(captain, who)`, action 3 (item 4301, `docs/CLIENT_INPUT.md`
    /// §11.4). A read only.
    pub captain: i16,
    /// **The crafts its panel offers** (item 4264, `docs/CLIENT_INPUT.md`
    /// §18): every craft row (`spell::FIRST..=LAST`) whose `FROM` or
    /// `FROM2` line the unit is and that [`Sim::spell_castable`] passes, in
    /// row order. What `UnitOut::make_options` puts on row 1 is read only
    /// as far as this; the button's own gates are `Options::do_spell`'s.
    pub crafts: Vec<Craft>,
    /// What the right press's spy arms ask of it (item 4303).
    pub spy: SpyMarks,
    /// **The slots it takes in a garrison**, as `GroupData::test_garrison
    /// @0070b320` counts them before it issues one (item 4273,
    /// `docs/CLIENT_INPUT.md` §23): a decoy (`unit_masks & 1`) none,
    /// anything else its type's `control_cost` (`+0x2f0`). A read only.
    pub garrison_cost: i32,
    /// **The buildings of its owner's it may enter**, by `o`: every one
    /// with a garrison limit ([`BuildingScene::garrison_limit`] above 0)
    /// whose type `UnitTypeData::can_garrison@0061d7d0` admits its own
    /// ([`Sim::can_garrison`]), the type half of `UnitData::can_garrison
    /// @006099a0`; the finished half is the building's
    /// [`BuildingScene::active`] (item 4273). A read only.
    pub enters: Vec<i16>,
    /// Its figures, one per guy.
    pub figures: Vec<FigureScene>,
}

/// One squad captain inside a building ([`Scene::garrisons`]): the
/// building's `o`, and the unit's `who`, `o` and `uid` (`+0x30`), which
/// `UnitOut::issue_come_out@00607be0` writes into command `0x31`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GarrisonScene {
    pub building: i16,
    pub owner: Player,
    pub o: i16,
    pub uid: u16,
    pub type_index: i32,
}

/// One figure of a unit: where it stands and which piece, animation and
/// frame the original would draw (`docs/ANIM.md`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FigureScene {
    /// The figure's own position — a crew body's, where it has one; guy
    /// 0's own body (`Movement::body`), which a push that does not move
    /// the guys leaves behind its unit's `pos` (item 4027); and otherwise
    /// its unit's.
    pub pos: Pos,
    /// The figure's own height, the `GUY` record's `z` (`GuyData +0x14`),
    /// as `Guy::update_z@005d9950` keeps it: a sea unit's figure at 0, the
    /// water's surface; an aircraft's at its own altitude (`Airframe::z`);
    /// every other on the ground under its point (`World::data_z`), which
    /// may be below zero. Not the unit's [`UnitScene::z`]: a figure stands
    /// where the renderer draws it (`docs/RENDERER.md` §2.3, item 4026).
    pub z: i32,
    /// The figure's own facing where it has one, otherwise its unit's.
    pub facing: Angle,
    /// The graphic piece.
    pub gpiece: i32,
    /// The `UnitAnim` slot.
    pub anim: i8,
    pub cur_time: u32,
    pub end_time: u32,
}

/// One building in use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildingScene {
    pub id: ObjId,
    pub owner: Player,
    /// An index into the building types; `None` for the bare object.
    pub ty: Option<usize>,
    pub pos: Pos,
    /// Its own height, `SubObjectData::z_internal` ([`crate::cities::building::Building::z`]):
    /// the ground under it once its own terraform has run, which a later
    /// terraform does not move (item 4035).
    pub z: i32,
    /// Hit points left and the hit points it was built with.
    pub health: i32,
    pub hits: i32,
    /// `WallData::is_active`: finished.
    pub active: bool,
    /// `WallData::is_started` (`flags & 2`, [`crate::cities::building::Building::started`]):
    /// `Wall::start` has run. `WallOut::draw_overlays@00641600` draws a
    /// building's scaffold only while it is started and not active
    /// (`docs/RENDERER.md` §3.5; item 4061).
    pub started: bool,
    /// Construction progress as the simulation keeps it: `constr_time`
    /// against `construct_hits` (`docs/CITIES.md` §3).
    pub constr_time: i32,
    pub construct_hits: i32,
    /// The type it was placed as, `BuildData::orig_type` — the one type a
    /// dump prints, and the type the renderer's K of M counts
    /// (`docs/RENDERER.md` §5.4; item 4035).
    pub orig_ty: Option<usize>,
    /// `WallData::job_counter` and `BuildData::construct_time(0)`: the
    /// share built, which `WallOut::draw@00641930` reads to choose whether
    /// an unfinished building is drawn (`docs/RENDERER.md` §3.5).
    pub job_counter: i32,
    pub construct_time: i32,
    /// The art coordinates `GraphicPieces::get_build_gpiece` walks from
    /// (item 4035), as `Wall::update_gpiece@00643070` passes them: the
    /// nation of its **founder** (`BuildData::founder`, `+0x7f`), an index
    /// into the tech tree's tribes — a captured city keeps the art of the
    /// nation that built it — and its age, `BuildData::max_age`
    /// ([`Sim::build_max_age`]): the owner's ages owned, the value
    /// `Leader::gain_tech` writes to every building of the player, or the
    /// age a captured building kept from its old owner (item 4463).
    pub tribe: usize,
    pub age: i32,
    /// A farm's record (`BuildData +0x78 >= 0`), which
    /// `FarmsOut::draw@008d75f0` draws as a wheat field; `None` for every
    /// other building (item 4048).
    pub field: Option<FieldScene>,
    /// What a right press on it asks of a gather building (item 4179,
    /// `docs/CLIENT_INPUT.md` §6): for a gather type (`build_flags & 0x40`,
    /// `BuildTypeData::is_gather_type`), the gatherers its chain holds
    /// (`BuildData::gather_down`) and its slot count (`gather_max`, `None`
    /// uncapped); `None` for every other building.
    pub gather: Option<(i32, Option<i32>)>,
    /// `BuildData::is_seen@0062e1a0` of the viewer, as
    /// [`Sim::build_is_seen`] ports it; `true` when there is no viewer
    /// (item 4202, [`UnitScene::seen`]).
    pub seen: bool,
    /// `BuildTypeData::is_training_building@00639de0` of its type, as
    /// [`crate::cities::build::is_training_building`] ports it: the arm of
    /// `GroupData::count`'s `COUNT_GATHER_POINT` the client's gather point
    /// reads (item 4249, `docs/CLIENT_INPUT.md` §11). A read only.
    pub trains: bool,
    /// `ObjectData::is(MARKET)` of its type: the building whose panel
    /// carries the sell and buy buttons (`BuildOut::make_options@00629ed0`;
    /// item 4259, `docs/CLIENT_INPUT.md` §16). A read only.
    pub market: bool,
    /// A city's centre (`SubObjectData::flags & 0x20`): the building whose
    /// panel carries the city's four gather buttons
    /// (`BuildOut::make_options@00629ed0`'s `OPTION_CITY_*_GATHER`), read
    /// as [`Sim::group_action_city_gather`] reads it, the building its city
    /// names (item 4261, `docs/CLIENT_INPUT.md` §17). A read only.
    pub city: bool,
    /// `ObjectData::is(VILLAGE)` of its type, the `VILLAGE` lineage
    /// (`GroupData::count`'s `COUNT_TYPE` 0x19e, which
    /// `Options::do_city_gather@0071cdb0` asks of the selection; item
    /// 4261). A read only.
    pub village: bool,
    /// `ObjectData::is(UNIVERSITY)` of its type (`0x1a4`): the one building
    /// a city's gather point may not name (`Console::execute_at_cursor`'s
    /// gather arm, item 4301, `docs/CLIENT_INPUT.md` §11.4). A read only.
    pub university: bool,
    /// **`ObjectData::uid` (`+0x30`)**, which `BuildOut::issue_unqueue
    /// @00629e70` writes into the cancel beside the building's `o` (item
    /// 4265, `docs/CLIENT_INPUT.md` §19). A read only.
    pub uid: u16,
    /// **`BuildData::queued` (`+0x82`)**: the live entries of its
    /// production queue, one queue button each on the original's panel
    /// (`IFaceSelected::on_button_clicked@00826e00`'s `OPTION_QUEUE`; item
    /// 4265). A read only.
    pub queued: usize,
    /// **`BuildData +0x72 >= 0`**: it belongs to a city, the test
    /// `Options::exec@007188c0`'s rename arm (option `0x117`) makes before
    /// it opens the city's name for editing (item 4268,
    /// `docs/CLIENT_INPUT.md` §21). A read only.
    pub in_city: bool,
    /// **`BuildTypeData::get_garrison_limit`** of it ([`Sim::garrison_limit`])
    /// and **`ObjectData::num_inside(0)`** ([`Sim::num_inside`]): the room
    /// `GroupData::test_garrison@0070b320` compares a unit's slots against,
    /// and the limit `Options::picked_spot@00721c40`'s `OPTION_GARRISON`
    /// arm asks to be nonzero (item 4273, `docs/CLIENT_INPUT.md` §23). A
    /// read only.
    pub garrison_limit: i32,
    pub inside: i32,
    /// What the right press's spy arms ask of it (item 4303): a building
    /// is never a Spy, a Scout, a citizen or a vehicle.
    pub spy: SpyMarks,
}

/// A farm's `FarmStruct`, as `FarmsOut::draw@008d75f0` reads it
/// (`docs/RENDERER.md` §3.5; item 4048).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldScene {
    /// The footprint of its type, `BuildTypeData +0x234` and `+0x238`,
    /// which `FarmsOut::draw` snaps the field's corner by.
    pub x_size: i32,
    pub y_size: i32,
    /// `farm_type`: [`crate::cities::farms::ANIMAL_FARM`] is the pasture,
    /// which the field pass skips.
    pub farm_type: u8,
    /// `terrain_height[col][row]`, `heights[col * 5 + row]`
    /// ([`crate::cities::farms::Farm::heights`]).
    pub heights: [i32; 25],
    /// `percent[col][row]` as its count of `0.005f` adds
    /// ([`crate::cities::farms::Farm::adds`]), `col * 4 + row`: the
    /// height and colour `FarmsOut::draw` gives each plot's box (item
    /// 4650, `docs/RENDERER.md` §6.1). A read only, outside the checksum
    /// lines' walk.
    pub adds: [i32; 16],
    /// `status[col][row]` ([`crate::cities::farms::Farm::state`]): which
    /// of the two `0.005f` sequences `adds` counts, the cut cell's
    /// decaying from `1.0f` (item 4650).
    pub status: [u8; 16],
}

/// One projectile in flight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectileScene {
    pub owner: Player,
    pub launch: Pos,
    pub landing: Pos,
    pub angle: Angle,
    /// `cur_time` of `total_time`.
    pub cur_time: i32,
    pub total_time: i32,
    pub air: bool,
    /// The round's graphic piece, `AmmoData::gpiece`, which
    /// `AmmoOut::draw@0067ac10` draws (`docs/RENDERER.md` §3.13, item
    /// 4175): an `<AMMO>` of `effects_graphics.xml` at
    /// [`crate::combat::FIRST_AMMO_PIECE`] plus its index, or a falling
    /// aircraft's own piece; `-1` where the shot's piece is not read.
    pub gpiece: i32,
}

/// One dead unit's fading record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeathScene {
    pub who: i32,
    pub o: i32,
    pub first_frame: i64,
    /// The `UnitAnim` slot each record plays, `DeathObj +0x8`.
    pub cur_anim: i32,
    /// **One piece per figure whose record still stands this frame**
    /// (`docs/RENDERER.md` §3.13, item 4175): `DeathObj +0x20`, each guy's
    /// own `gpiece` at the death (`DeathObj::init@008d4c60`'s vslot
    /// `+0x178`), which `DeathObjOut::draw@008d4090` sends through
    /// `GraphicPieces::draw` on slot [`Self::cur_anim`]. A record whose
    /// life has run out, or that the pool evicted, is gone from the list.
    pub pieces: Vec<i32>,
    /// **Where each of those records lies** (item 4191, `docs/RENDERER.md`
    /// §3.13), one per entry of [`Self::pieces`]: `DeathObjData` `x`, `y`,
    /// `z` and `new_angle`, the guy's own point and facing at the kill's
    /// making ([`crate::combat::Lie`]), which `DeathObjOut::draw@008d4090`
    /// places and turns the body by.
    pub places: Vec<crate::combat::Lie>,
    /// **Each of those records' action, in game frames**, one per entry
    /// of [`Self::pieces`]: `AnimationPacket::get_game_frames` of its
    /// piece on [`Self::cur_anim`], which `DeathObjOut::draw@008d4090`
    /// reads every draw — the body plays to `(frames × 200 − 200) / 3` ms
    /// and holds there, and its fade and the skeleton's count from it
    /// (item 4191). `None` where the install does not describe the slot.
    pub frames: Vec<Option<u32>>,
}

/// The terrain grids, fetched when [`Scene::terrain_version`] moves.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TerrainScene {
    /// The map's size in cells.
    pub width: i32,
    pub height: i32,
    /// The tile heights, row-major over `width × 4` by `height × 4` tiles;
    /// empty on a world with no height table.
    pub tile_z: Vec<i32>,
    /// The corner grid the tile heights are taken from
    /// (`TerrainData::master_land_heights`), row-major over `width × 4 + 1`
    /// by `height × 4 + 1` corners, each single as its 32 bits — the
    /// heightfield a client draws (item 4036); empty on a world with no
    /// height table.
    pub corners: Vec<u32>,
    /// Each cell's `WData.land` and `land_sub`, row-major.
    pub land: Vec<i8>,
    pub land_sub: Vec<u8>,
    /// Each cell's `WData.flags`, row-major — the bits [`crate::world::cell`]
    /// names. A client reads [`crate::world::cell::FOREST`] off it: the
    /// cells `ObjectsOut::sort_trees@00656c70` stands trees on (item 4080).
    pub flags: Vec<u16>,
    /// Each cell's `WData.region` and `WData.region2` (`+0x4`, `+0x6`) as
    /// the original numbers them — land `1..0x40`, sea above — row-major,
    /// `-1` for none. The surf's shore reads them
    /// (`TerrainOut::tcoord_is_surfline@00867470`; item 4239).
    pub region: Vec<i16>,
    pub region2: Vec<i16>,
    /// Each tile's `TData.mask` (`World +0x138`), row-major over `width ×
    /// 4` by `height × 4` tiles — the bits [`crate::world::tile`] names.
    pub tiles: Vec<u16>,
    /// The placed mountain ranges: each range's tiles.
    pub mountains: Vec<Vec<Pos>>,
    pub version: u32,
}

/// Who holds each cell, fetched when [`Scene::territory_version`] moves.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TerritoryScene {
    pub width: i32,
    pub height: i32,
    /// Each cell's owner, row-major.
    pub owner: Vec<Owner>,
    pub version: u32,
}

/// **The roads as they stand** (item 4199, `docs/RENDERER.md` §3.19): one
/// row per element of the road mesh (`RoadsPieces::candidates`, the
/// original's `CoordInfo::roads_in_wcoord`), in the mesh's index order. A
/// copy of what the simulation keeps: the eight connections, the two
/// reference counts, and each tile's compass of roads and of footprints —
/// the two caches `RoadsOut::fill_cache@00893b30` fills before the texture
/// passes read them. The passes themselves, which choose a piece and a
/// rotation, are a renderer's (`client::roads`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoadsScene {
    /// The tile grid's size, four tiles a cell.
    pub tiles_wide: i32,
    pub tiles_high: i32,
    pub roads: Vec<RoadTile>,
}

/// One element of the road mesh.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoadTile {
    /// The tile.
    pub x: i32,
    pub y: i32,
    /// `RoadElementCandidate::flags`' top byte: the eight connections,
    /// clockwise from NW at bit 7 (`crate::movement::mesh::dir`, shifted
    /// down 24).
    pub dirs: u8,
    /// `ref_count` (`+0xa`) and `pending_camel_steps` (`+0xb`).
    pub refs: u8,
    pub refs2: u8,
    /// `is_terrain_creation` (`+0xf`): the mesh laid this tile itself.
    pub made: bool,
    /// The element is `set_diags`' C4 corner piece (`docs/ROADS.md` §12).
    pub c4: bool,
    /// `flags`' low byte, the intersections the texture marks left on the
    /// tile, history and all (item 4677; `crate::movement::mesh::Elem::inter`).
    pub inter: u8,
    /// The intersections as the tile's piece was last picked (item 4682;
    /// `crate::movement::mesh::Elem::picked`).
    pub picked: u8,
    /// `fill_cache`'s `road_cache`: bit `k` set when compass neighbour `k`
    /// (`MOVE_X`/`MOVE_Y` order, 0 the tile itself, 1 NW, 2 N … 8 W) is a
    /// road tile (`tile::SURFACE == SURFACE_ROAD`). Off the map is clear.
    pub road: u16,
    /// `fill_cache`'s `neighbor_cache`: bit `k` set when compass neighbour
    /// `k` is a footprint (`tile::OBJECT == OBJECT_BUILDING`) that is not a
    /// road.
    pub footprint: u16,
}

/// The values of [`Scene::fog`]: the level `WorldMapFog::fill_colors_fog@008d1420`
/// gives a fog cell and writes to `fog_lookup` (item 4624; `docs/RENDERER.md`
/// §3.32.1). The first predicate that holds sets it, in this order:
/// `WorldData::is_seen@006b55c0`, `was_really_seen@006b54f0`,
/// `was_seen@006b53f0`.
pub mod fog {
    /// Level 0: none of the three holds (the minimap's grey 0).
    pub const UNSEEN: u8 = 0;
    /// Level 1: `was_seen` and not really seen. Past the `seen2` bits, the
    /// region rule: an ally owns the cell and holds a city or a fort in its
    /// region (the minimap's grey `0x50`).
    pub const REGION: u8 = 1;
    /// Level 2: `was_really_seen`, the `seen2` bits under the ally mask,
    /// not in sight now (grey `0x80`).
    pub const EXPLORED: u8 = 2;
    /// Level 3: `is_seen`, in sight now under the ally mask (grey 255).
    pub const VISIBLE: u8 = 3;
}

impl Sim {
    /// This tick's drawable state as `viewer` sees it, or everything when
    /// `viewer` is `None`. Reads only; see the module note.
    pub fn observe(&self, viewer: Option<Player>) -> Scene {
        let units = self
            .units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.on_map && u.alive())
            .map(|(i, u)| {
                let facing = u.movement.facing;
                let domain = self.unit_domain_of(i);
                UnitScene {
                    id: ObjId {
                        who: u.owner,
                        o: u.index,
                    },
                    owner: u.owner,
                    uid: u.uid,
                    ty: u.ty,
                    type_index: u.type_index,
                    form: u.form,
                    form_width: u.form_width,
                    pos: u.pos,
                    z: self.world.tile_z(u.pos.tile()),
                    facing,
                    health: u.health,
                    max_health: u.max_health,
                    seen: viewer.is_none_or(|p| self.target_seen_by(p, Obj::Unit(i))),
                    captain: u.combat.captain as i16,
                    attacks: u.captain
                        && u.ty
                            .is_some_and(|t| self.tables.unit_types[t].combat.attack != 0),
                    crafts: (crate::ids::spell::FIRST..=crate::ids::spell::LAST)
                        .filter(|&t| self.spell_castable(t, i))
                        .filter_map(|t| {
                            let d = self.spell(t)?;
                            Some(Craft {
                                t,
                                targeted: d.targeted(),
                                ground: d.flags & 8 != 0,
                            })
                        })
                        .collect(),
                    spy: SpyMarks {
                        spy: self.unit_is(i, crate::units::think::scout::SPY),
                        scout_line: self.unit_is(i, crate::ids::ty::SCOUT),
                        commando: self.unit_is(i, crate::ids::ty::COMMANDO),
                        peasant: self.is_peasant(i),
                        vehicle: self.profile(Obj::Unit(i)).obj_masks
                            & crate::combat::mask::VEHICLE
                            != 0,
                        bribe: viewer.is_some_and(|p| {
                            self.spell_valid_target(crate::ids::spell::BRIBE, p, Obj::Unit(i))
                        }),
                        sniper: viewer.is_some_and(|p| {
                            self.spell_valid_target(crate::ids::spell::SNIPER, p, Obj::Unit(i))
                        }),
                        viewer_ground: false,
                    },
                    garrison_cost: if u.decoy {
                        0
                    } else {
                        u.ty.map_or(1, |t| self.tables.unit_types[t].price.pop)
                    },
                    enters: u.ty.map_or_else(Vec::new, |ut| {
                        self.buildings
                            .iter()
                            .enumerate()
                            .filter(|(b, bd)| {
                                bd.alive
                                    && bd.owner == u.owner
                                    && bd.ty.is_some_and(|bt| self.can_garrison(ut, bt))
                                    && self.garrison_limit(*b) > 0
                            })
                            .map(|(_, bd)| bd.index)
                            .collect()
                    }),
                    figures: u
                        .guys
                        .iter()
                        .enumerate()
                        .map(|(k, g)| {
                            let (pos, facing) = self.figure_point(i, k);
                            FigureScene {
                                pos,
                                z: match domain {
                                    Domain::Sea => 0,
                                    Domain::Air => u.airframe.z,
                                    Domain::Land => self.world.data_z(pos.x, pos.y).0,
                                },
                                facing,
                                gpiece: g.gpiece,
                                anim: g.anim,
                                cur_time: g.cur_time,
                                end_time: g.end_time,
                            }
                        })
                        .collect(),
                }
            })
            .collect();
        let buildings = self
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.alive)
            .map(|(i, b)| BuildingScene {
                id: ObjId {
                    who: b.owner,
                    o: b.index,
                },
                owner: b.owner,
                ty: b.ty,
                pos: b.pos,
                z: b.z,
                health: b.health,
                hits: b.hits,
                active: b.active,
                started: b.started,
                constr_time: b.constr_time,
                construct_hits: b.construct_hits,
                orig_ty: b.orig_ty,
                job_counter: b.job_counter,
                construct_time: self.construct_time_of(i),
                tribe: self.tech.get(b.founder as usize).map_or(0, |p| p.tribe),
                age: self.build_max_age(i) as i32,
                // A record in the list — `farm.valid` is `Farms::add`'s,
                // and a farm the harness stood up from a dump joins the
                // list without it.
                field: self.cities.farm_order.contains(&i).then(|| FieldScene {
                    x_size: b.ty.map_or(0, |t| self.tables.build_types[t].x_size),
                    y_size: b.ty.map_or(0, |t| self.tables.build_types[t].y_size),
                    farm_type: b.farm.farm_type,
                    heights: b.farm.heights,
                    adds: b.farm.adds,
                    status: b.farm.state,
                }),
                gather: b
                    .ty
                    .filter(|&t| {
                        self.tables.build_types[t].flags & crate::cities::build::flags::GATHERER
                            != 0
                    })
                    .map(|_| (self.gather_chain(i).len() as i32, b.gather_max)),
                seen: viewer.is_none_or(|p| self.build_is_seen(i, p)),
                trains: b.ty.is_some_and(|t| {
                    crate::cities::build::is_training_building(&self.tables.build_types, t)
                }),
                market: b.ty.is_some_and(|t| {
                    crate::cities::build::is(
                        &self.tables.build_types,
                        t,
                        crate::cities::build::Ident::Market,
                    )
                }),
                city: b
                    .city
                    .is_some_and(|c| self.cities.cities.get(c).is_some_and(|c| c.building == i)),
                village: self.building_is_city(i),
                university: self.building_is(i, crate::cities::build::Ident::University),
                uid: b.uid,
                queued: b.queue.items.len(),
                in_city: b.city.is_some(),
                garrison_limit: self.garrison_limit(i),
                inside: self.num_inside(i),
                spy: SpyMarks {
                    viewer_ground: viewer.is_some_and(|p| {
                        self.world.owner(crate::World::cell_of_tile(b.pos.tile()))
                            == crate::world::Owner::Player(p)
                    }),
                    ..SpyMarks::default()
                },
            })
            .collect();
        let projectiles = self
            .combat
            .projectiles
            .iter()
            .map(|p| ProjectileScene {
                owner: p.owner,
                launch: p.launch,
                landing: p.landing,
                angle: p.angle,
                cur_time: p.cur_time,
                total_time: p.total_time,
                air: p.air,
                gpiece: p.gpiece,
            })
            .collect();
        let deaths = self
            .combat
            .deaths
            .iter()
            .map(|d| {
                let age = self.frame - d.first_frame;
                let n = usize::from(d.figures.n).min(crate::combat::Figures::MAX);
                DeathScene {
                    who: d.who,
                    o: d.o,
                    first_frame: d.first_frame,
                    cur_anim: d.cur_anim,
                    pieces: (0..n)
                        .filter(|&k| age < d.figures.life[k])
                        .map(|k| d.figures.gpiece[k])
                        .collect(),
                    places: (0..n)
                        .filter(|&k| age < d.figures.life[k])
                        .map(|k| d.figures.place[k])
                        .collect(),
                    frames: (0..n)
                        .filter(|&k| age < d.figures.life[k])
                        .map(|k| {
                            let (gp, anim) = (d.figures.gpiece[k], i8::try_from(d.cur_anim).ok()?);
                            match self.roster.art.piece_lengths.get(&gp) {
                                Some(slots) => Some(
                                    slots
                                        .get(&anim)
                                        .copied()
                                        .unwrap_or(crate::units::anim::MISSING),
                                ),
                                None => self.roster.art.length(gp, anim),
                            }
                        })
                        .collect(),
                }
            })
            .collect();
        Scene {
            frame: self.frame,
            units,
            buildings,
            garrisons: self.observe_garrisons(),
            projectiles,
            deaths,
            leaders: self.observe_leaders(),
            fog: viewer.and_then(|p| self.observe_fog(p)),
            terrain_version: self.world.terrain_version(),
            ai_off: self.ai.ai_off,
            territory_version: self.world.territory_version(),
        }
    }

    /// [`Scene::garrisons`].
    fn observe_garrisons(&self) -> Vec<GarrisonScene> {
        self.buildings
            .iter()
            .filter(|b| b.alive)
            .flat_map(|b| {
                b.garrison.iter().filter_map(move |&c| {
                    let u = self.units.get(c).filter(|u| u.alive())?;
                    Some(GarrisonScene {
                        building: b.index,
                        owner: u.owner,
                        o: u.index,
                        uid: u.uid,
                        type_index: u.type_index,
                    })
                })
            })
            .collect()
    }

    /// Every player slot's [`LeaderScene`]: a slot is one the tech, the
    /// ledgers and the muster all hold.
    /// **Guy `k`'s own point and facing** (`GuyData` `x`, `y`, `angle`):
    /// a crew body's where it follows one; guy 0's own body
    /// (`Movement::body`, `GuyData +0xc`), which a push that does not move
    /// the guys leaves behind its unit's `pos` (item 4027); otherwise the
    /// unit's. The door draws a figure here, and a death record lies here
    /// (item 4191, [`crate::combat::Lie`]).
    pub(crate) fn figure_point(&self, i: usize, k: usize) -> (Pos, Angle) {
        let u = &self.units[i];
        let own = if k == 0 { u.movement.body.pos } else { u.pos };
        match u.guys.get(k).and_then(|g| g.follow) {
            Some(f) => (f.body.pos, f.facing),
            None => (own, u.movement.facing),
        }
    }

    fn observe_leaders(&self) -> Vec<LeaderScene> {
        let slots = self
            .tech
            .len()
            .min(self.economy.ledgers.len())
            .min(self.roster.muster.len());
        (0..slots)
            .map(|who| {
                let (t, l, m) = (
                    &self.tech[who],
                    &self.economy.ledgers[who],
                    &self.roster.muster[who],
                );
                LeaderScene {
                    who: who as Player,
                    tribe: t.tribe,
                    nation: self
                        .tables
                        .tech_tree
                        .tribes
                        .get(t.tribe)
                        .map_or_else(String::new, |n| n.name.clone()),
                    ages: t.ages,
                    bucket: l.bucket,
                    income: l.income,
                    over_cap: l.over_cap.map(|o| o != crate::economy::OverCap::Under),
                    control: m.control,
                    pop_cap: m.cap,
                    market: self.market_offer(who as Player),
                    enemies: (0..slots)
                        .map(|b| b as Player)
                        .filter(|&b| self.is_enemy(who as Player, b))
                        .collect(),
                }
            })
            .collect()
    }

    /// **The map's goods as they stand** (item 4312; `docs/RENDERER.md`
    /// §3.18): `World::goods`, the list `Objects::init_good@00653f30` lays
    /// before the first frame, which `ObjectsOut::draw_good_underlays`
    /// walks to draw each good's underlay (the `Oil` billboard).
    pub fn observe_goods(&self) -> Vec<crate::world::Good> {
        self.world.goods().to_vec()
    }

    /// The terrain grids as they stand.
    pub fn observe_terrain(&self) -> TerrainScene {
        let w = &self.world;
        let (width, height) = (w.width(), w.height());
        let cells = (0..height).flat_map(|y| (0..width).map(move |x| crate::Cell::new(x, y)));
        let (land, land_sub): (Vec<i8>, Vec<u8>) = cells
            .clone()
            .map(|c| {
                let d = w.cell_data(c);
                (d.land, d.land_sub)
            })
            .unzip();
        let flags = cells.clone().map(|c| w.cell_data(c).flags).collect();
        let number = |r: Option<u16>| r.map_or(-1, |r| w.region_number(r) as i16);
        let region = cells.clone().map(|c| number(w.region_of(c))).collect();
        let region2 = cells.map(|c| number(w.cell_data(c).region2)).collect();
        let tiles = (0..height * 4)
            .flat_map(|y| (0..width * 4).map(move |x| Pos::new(x, y)))
            .map(|t| w.tile_mask(t))
            .collect();
        TerrainScene {
            width,
            height,
            tile_z: w.tile_z_grid().to_vec(),
            corners: w.corner_singles().iter().map(|c| c.bits()).collect(),
            land,
            land_sub,
            flags,
            region,
            region2,
            tiles,
            mountains: self
                .terrain
                .mountains
                .iter()
                .map(|m| m.tiles.clone())
                .collect(),
            version: w.terrain_version(),
        }
    }

    /// The road mesh as it stands ([`RoadsScene`], item 4199).
    pub fn observe_roads(&self) -> RoadsScene {
        use crate::ai::ai_place::{MOVE_X, MOVE_Y};
        use crate::world::tile;
        let w = &self.world;
        let (tw, th) = (w.width() * 4, w.height() * 4);
        let compass = |x: i32, y: i32| {
            let (mut road, mut footprint) = (0u16, 0u16);
            for k in 0..9 {
                let (nx, ny) = (x + MOVE_X[k], y + MOVE_Y[k]);
                if nx < 0 || ny < 0 || nx >= tw || ny >= th {
                    continue;
                }
                let m = w.tile_mask(Pos::new(nx, ny));
                let is_road = m & tile::SURFACE == tile::SURFACE_ROAD;
                road |= u16::from(is_road) << k;
                footprint |= u16::from(m & tile::OBJECT == tile::OBJECT_BUILDING && !is_road) << k;
            }
            (road, footprint)
        };
        let roads = self
            .movement
            .mesh
            .elems()
            .map(|(i, e)| {
                let (x, y) = (i.rem_euclid(tw), i.div_euclid(tw));
                let (road, footprint) = compass(x, y);
                RoadTile {
                    x,
                    y,
                    dirs: (e.flags >> 24) as u8,
                    refs: e.refs,
                    refs2: e.refs2,
                    made: e.made,
                    c4: e.c4,
                    inter: e.inter,
                    picked: e.picked,
                    road,
                    footprint,
                }
            })
            .collect();
        RoadsScene {
            tiles_wide: tw,
            tiles_high: th,
            roads,
        }
    }

    /// Who holds each cell as it stands.
    pub fn observe_territory(&self) -> TerritoryScene {
        let w = &self.world;
        let (width, height) = (w.width(), w.height());
        TerritoryScene {
            width,
            height,
            owner: (0..height)
                .flat_map(|y| (0..width).map(move |x| w.owner(crate::Cell::new(x, y))))
                .collect(),
            version: w.territory_version(),
        }
    }

    /// **One player's fog plane** ([`Scene::fog`]), `None` on a world with
    /// no fog grid: each fog cell's level as `fill_colors_fog@008d1420`
    /// sets it, through the three predicates the sim already ports
    /// ([`fog`]). Item 4624: run1814b's packet holds `fog_lookup`
    /// 13,336 / 727 / 127 / 210 on frame 3294, and this plane is it cell
    /// for cell (`client::minimap::fog`'s test). It is a read: the twelve
    /// checksum lines' walk never calls it.
    pub fn observe_fog(&self, p: Player) -> Option<Vec<u8>> {
        let w = &self.world;
        if !w.has_fog() {
            return None;
        }
        let (fw, fh) = (w.fog_xs(), w.fog_ys());
        Some(
            (0..fh)
                .flat_map(|fy| (0..fw).map(move |fx| (fx, fy)))
                .map(|(fx, fy)| {
                    if self.world_is_seen_fog(fx, fy, p) {
                        fog::VISIBLE
                    } else if self.was_really_seen_fog(fx, fy, p) {
                        fog::EXPLORED
                    } else if self.was_seen_fog(fx, fy, p) {
                        fog::REGION
                    } else {
                        fog::UNSEEN
                    }
                })
                .collect(),
        )
    }
}

pub mod alerts;

#[cfg(test)]
mod tests;
