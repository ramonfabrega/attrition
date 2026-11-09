//! Every id family the simulation names, defined once (`docs/STRUCTURE.md`
//! §4, item 4013).
//!
//! An id here is a number the original hardcodes or the shipped data
//! fixes by position — a nation's roster index, a `TypeIndex`, a spell's
//! row — and the modules that read one re-export it rather than declaring
//! their own. Where a family is the user's data, its `NAMED_*` table pairs
//! each constant with the name the install gives it, and
//! `cargo run -p rondata -- <install>` re-derives every pair and exits
//! non-zero on one that has moved.

/// The nation roster's indices — `rules.xml`'s `TRIBES` order, which is
/// what `LeaderData::tribe` holds and what `has_tribe_bonus(n)` compares
/// against (`docs/TECH.md` §"`has_preq`"). The AI hardcodes several, and
/// `Build::activate`'s free-unit block tests the powers by these numbers
/// (`crate::nations::power` is this module under that name).
pub mod tribe {
    pub const AZTECS: usize = 0;
    pub const MAYA: usize = 1;
    /// The Inca's wealth per miner is `BuildTypeData::calc_gather`'s
    /// (`Sim::site_of`, item 3360).
    pub const INCA: usize = 2;
    pub const BANTU: usize = 3;
    pub const NUBIANS: usize = 4;
    /// The Greeks' research arm is `ObjectData::train_time`'s
    /// (`Sim::greek_research`), not `Build::activate`'s.
    pub const GREEKS: usize = 5;
    pub const ROMANS: usize = 6;
    /// The Egyptians' Farm wealth is `BuildTypeData::calc_gather`'s
    /// (`Sim::site_of`, item 3332).
    pub const EGYPTIANS: usize = 7;
    pub const TURKS: usize = 8;
    pub const SPANISH: usize = 9;
    pub const FRENCH: usize = 10;
    pub const BRITISH: usize = 11;
    pub const GERMANS: usize = 12;
    pub const RUSSIANS: usize = 13;
    pub const CHINESE: usize = 14;
    /// The Japanese' Farm arm is `get_cost`'s
    /// (`Sim::japanese_farm_discount`).
    pub const JAPANESE: usize = 15;
    pub const KOREANS: usize = 16;
    pub const MONGOLS: usize = 17;
    pub const IROQUOIS: usize = 18;
    pub const LAKOTA: usize = 19;
    pub const AMERICANS: usize = 20;
    /// The Indians' building ramp is `get_cost`'s (`Sim::indian_ramp`).
    pub const INDIANS: usize = 21;
    pub const DUTCH: usize = 22;
    /// The Persians' caravan is `Build::process`'s
    /// (`Sim::persian_market_caravan`), not `Build::activate`'s.
    pub const PERSIANS: usize = 23;

    /// Each constant beside its nation file's `<TRIBE name>`, which
    /// `rondata` re-derives from `rules.xml` and `tribes/`.
    pub const NAMED_TRIBES: [(&str, usize); 24] = [
        ("Aztecs", AZTECS),
        ("Maya", MAYA),
        ("Inca", INCA),
        ("Bantu", BANTU),
        ("Nubians", NUBIANS),
        ("Greeks", GREEKS),
        ("Romans", ROMANS),
        ("Egyptians", EGYPTIANS),
        ("Turks", TURKS),
        ("Spanish", SPANISH),
        ("French", FRENCH),
        ("British", BRITISH),
        ("Germans", GERMANS),
        ("Russians", RUSSIANS),
        ("Chinese", CHINESE),
        ("Japanese", JAPANESE),
        ("Koreans", KOREANS),
        ("Mongols", MONGOLS),
        ("Iroquois", IROQUOIS),
        ("Lakota", LAKOTA),
        ("Americans", AMERICANS),
        ("Indians", INDIANS),
        ("Dutch", DUTCH),
        ("Persians", PERSIANS),
    ];
}

/// The `TypeIndex` of a spell, the value a `CastOrder` carries at `+0x20`
/// and `SpellType::cast` switches on. The spell rows are `0x275..=0x2ab`,
/// in `craftrules.xml`'s own order (`docs/DATALAYER.md`);
/// `crate::orders::spell` re-exports them beside `is_pack`/`is_unpack`.
pub mod spell {
    /// The first row of `craftrules.xml`, `Bribe` — the base `SpellType`
    /// index and the offset [`crate::Sim::spell`] subtracts.
    pub const FIRST: i32 = 0x275;
    /// The last, the eighth `Cancel Alliance`. `TypeData::is_spell_type`
    /// is `FIRST <= i <= LAST` and nothing else (`Unit::do_cast`'s head).
    pub const LAST: i32 = 0x2ab;

    /// `TRANSPORT` — the shore conversion (`docs/TRANSPORT.md` §6).
    pub const TRANSPORT: i32 = 0x28a;

    /// The Spy's three crafts (`FROM Spy`), each targeted: `BRIBE` (`fcbhm`,
    /// `MANA 1000`), `COUNTERINTEL` (`febchm`, `MANA 500`; the one a human
    /// Spy's `think_spellcaster` casts by itself) and `INFORMER` (`fbcml`,
    /// `MANA 500`), which `SpellType::cast` hands to `cast_double_agent`
    /// and `GroupOut::validate_spell` names `DOUBLE_AGENT`
    /// (`docs/GOLDEN.md` §27).
    pub const BRIBE: i32 = 0x275;
    pub const COUNTERINTEL: i32 = 0x277;
    pub const INFORMER: i32 = 0x27f;
    /// `Pilfer Resources`, `Sabotage` and `Sniper`, the rows
    /// `SpellType::valid_target` and `think_spellcaster` name beside them.
    pub const PILFER: i32 = 0x276;
    pub const SABOTAGE: i32 = 0x280;
    pub const SNIPER: i32 = 0x281;

    /// The three untargeted crafts chapter thirty-nine casts
    /// (`docs/GOLDEN.md` §48): the General's `CREATE_DECOY` (`lm`,
    /// `MANA 1000`, `JOB_TIME 100`), which `SpellType::cast` hands to
    /// `cast_create_decoy@00674370`; the Militia's `CIVILIAN` and the
    /// Citizen's and Scholar's `TO_ARMS` (no flags, `JOB_TIME 5`), handed
    /// to `cast_civilian@006704a0` and `cast_to_arms@00670880`.
    pub const CREATE_DECOY: i32 = 0x27a;
    /// The General's two crafts behind its moving arm
    /// (`Unit::think_spellcaster`'s hero arm, `docs/AI.md` §99.13):
    /// `AMBUSH` and `FORCED_MARCH`, untargeted. What they do when cast is
    /// not carried.
    pub const AMBUSH: i32 = 0x27b;
    pub const FORCED_MARCH: i32 = 0x27c;
    /// `Entrench`, the General's third untargeted craft
    /// (`SpellType::cast_entrench@00672820`, `docs/GOLDEN.md` §166).
    pub const ENTRENCH: i32 = 0x27d;
    /// `Rally!`, the row before `CREATE_DECOY` — the third craft
    /// `Unit::do_cast` plays a hero's attack slot for.
    pub const RALLY: i32 = 0x279;
    pub const CAST_CIVILIAN: i32 = 0x293;
    pub const TO_ARMS: i32 = 0x294;

    /// The four **pack** rows and the four **unpack** ones, in the pairs
    /// `add_cast_order` rewrites `PACK`/`UNPACK` into: the siege engine's
    /// (the generic pair, and the one the file names `Catapult`), the
    /// machine gun's, the merchants' and the fishing boat's
    /// (`docs/ORDERS.md` §6.9).
    pub const PACK: i32 = 0x28b;
    pub const UNPACK: i32 = 0x28c;
    pub const PACK_MACHINEGUN: i32 = 0x28d;
    pub const UNPACK_MACHINEGUN: i32 = 0x28e;
    pub const PACK_MERCHANT: i32 = 0x28f;
    pub const UNPACK_MERCHANT: i32 = 0x290;
    pub const PACK_FISHERMEN: i32 = 0x291;
    pub const UNPACK_FISHERMEN: i32 = 0x292;

    /// Each named row beside its `craftrules.xml` `<NAME>`; `rondata`
    /// reads the record at `id - FIRST`. The four pack rows and the four
    /// unpack ones share a name each, so for those the check is their
    /// pairing, not which engine they pack.
    pub const NAMED_SPELLS: [(&str, i32); 24] = [
        ("Bribe", BRIBE),
        ("Pilfer Resources", PILFER),
        ("Counterintelligence", COUNTERINTEL),
        ("Rally!", RALLY),
        ("Create Decoys", CREATE_DECOY),
        ("Ambush", AMBUSH),
        ("Forced March", FORCED_MARCH),
        ("Entrench", ENTRENCH),
        ("Informer", INFORMER),
        ("Sabotage", SABOTAGE),
        ("Sniper", SNIPER),
        ("Transport", TRANSPORT),
        ("Pack", PACK),
        ("Deploy", UNPACK),
        ("Pack", PACK_MACHINEGUN),
        ("Deploy", UNPACK_MACHINEGUN),
        ("Pack", PACK_MERCHANT),
        ("Deploy", UNPACK_MERCHANT),
        ("Pack", PACK_FISHERMEN),
        ("Deploy", UNPACK_FISHERMEN),
        ("Civilian", CAST_CIVILIAN),
        ("To Arms", TO_ARMS),
        ("Bribe", FIRST),
        ("Cancel Alliance", LAST),
    ];
}

/// `OrderIndex` — the value the gamelog's `type` line carries, stored as
/// the order's byte; `crate::orders::index` re-exports it beside the
/// family tests. A walk over the original that reads `get_type()` as an
/// `i32` widens these (`as i32`) rather than declaring its own: every
/// value is below `0x80`, so the widening is the identity on it.
pub mod order {
    pub const NONE: u8 = 0;
    pub const MOVE_TO: u8 = 1;
    pub const ATTACK_TO: u8 = 2;
    pub const EXPLORE_TO: u8 = 3;
    pub const FLEE_TO: u8 = 4;
    pub const BUILD_AT: u8 = 6;
    pub const GATHER: u8 = 7;
    pub const ATTACK: u8 = 10;
    /// `FollowOrder` — [`crate::orders::Body::Follow`], issued by
    /// `Group::action_follow@006fd510` from a player's follow command
    /// (`docs/ORDERS.md` §28).
    pub const FOLLOW: u8 = 11;
    /// `GuardOrder` — [`crate::orders::Body::Guard`], issued by
    /// `Group::action_guard` (`docs/ORDERS.md` §7.5, §24) and read by
    /// `UnitData::get_speed`'s order scale (`docs/MOVEMENT.md`, "The
    /// effective speed here, now").
    pub const GUARD: u8 = 12;
    pub const REPAIR: u8 = 13;
    pub const CAST_SPELL: u8 = 14;
    pub const TRADE_ROUTE: u8 = 15;
    pub const GARRISON: u8 = 26;
    pub const THINK: u8 = 27;

    /// The three kinds of the **move family** this crate does not spell as
    /// a plain `MoveKind`. `CHANGE_FORM` is `FormOrder`, which this crate
    /// has no order for at all; the other two are what the original calls
    /// a move whose `MoveOrder` carries a [`crate::orders::GroupMove`], and
    /// [`crate::orders::Order::index`] returns them because `get_type()` does
    /// (`GroupMoveOrder::get_type@00485a10` → `GROUP_MOVE`,
    /// `GroupAttackToOrder::get_type@004825b0` → `GROUP_ATTACK_TO`).
    pub const CHANGE_FORM: u8 = 18;
    pub const GROUP_MOVE: u8 = 19;
    pub const GROUP_ATTACK_TO: u8 = 21;
    /// `GroupPatrolOrder` — [`crate::orders::Body::Patrol`]. `PATROL` (5) is a
    /// class that is never constructed alone: `Unit::add_patrol_order@
    /// 005e4560` asks `OrdersMemManager::get_obj` for `GROUP_PATROL` and
    /// nothing else (`docs/ORDERS.md` §7.7, §27).
    pub const GROUP_PATROL: u8 = 22;
    /// `AttackGroundOrder` — [`crate::orders::Body::AttackGround`], which this
    /// crate issues from one place: `Unit::fight`'s siege arm, an
    /// unpacked packer's shot at a unit (`docs/COMBAT.md` §57).
    pub const ATTACK_GROUND: u8 = 23;
    /// `StrafeOrder` — [`crate::orders::Body::Strafe`], the one class
    /// `CommandManager::issue_flight@00941d40` makes, through
    /// `Unit::add_strafe_order@005e48c0` (`docs/ORDERS.md` §32). `AirOrder`
    /// has no index of its own: it is this class's second base.
    pub const STRAFE: u8 = 16;
    /// `AirPatrolOrder` — [`crate::orders::Body::AirPatrol`], which this crate
    /// makes in one place: `Unit::do_strafe`'s arm for a strike whose
    /// target it may not take, `add_air_patrol_order` over the strike's
    /// point (`docs/ORDERS.md` §34).
    pub const AIR_PATROL: u8 = 17;
    /// `AirAttackGroundOrder` — [`crate::orders::Body::AirAttackGround`], a
    /// missile's strike: `Unit::add_strafe_order@005e48c0`'s head turns a
    /// strafe of a missile type on a valid target into
    /// `add_air_attack_ground_order@005e41c0` at the target's point
    /// (`docs/PRODUCTION.md`, "The missile's launch and round").
    pub const AIR_ATTACK_GROUND: u8 = 24;
}

/// `ArmyData::status` bits (`docs/AI.md`'s army section, §1) — the field
/// is `Army::status: i32`, and `crate::army::status` is this module. A
/// walk that reads the word as `u32` widens these (`as u32`): every bit is
/// below `0x100`, so the cast is the identity on it.
pub mod army_status {
    pub const MUSTERING: i32 = 0x01;
    pub const MARCHING: i32 = 0x02;
    pub const MARCHED: i32 = 0x04;
    pub const NO_TARGET: i32 = 0x08;
    pub const FORMING: i32 = 0x10;
    pub const DEFENDING: i32 = 0x20;
    pub const TRANSPORTING: i32 = 0x40;
    pub const HURRY: i32 = 0x80;
}

/// `CountIndex` (the type record's enum, `rise.pdb`): the first argument
/// of `ObjectData::count_inside` and its kin, which the walks over the
/// original pass as an `i32`.
pub mod count {
    pub const COUNT_WORKERS: i32 = 0;
    pub const COUNT_PEASANTS: i32 = 1;
    pub const COUNT_ATTACK: i32 = 2;
    pub const COUNT_MILITARY: i32 = 3;
    pub const COUNT_DOMAIN: i32 = 9;
    pub const COUNT_ON_MAP: i32 = 10;
    pub const COUNT_GATHER_PEASANTS: i32 = 11;
    pub const COUNT_GATHER_WORKERS: i32 = 12;
    pub const COUNT_GATHER_SCHOLARS: i32 = 13;
    pub const COUNT_BUILD_PEASANTS: i32 = 14;
    pub const COUNT_STRICT_TYPE: i32 = 16;
    pub const COUNT_TYPE: i32 = 17;
    pub const COUNT_TYPE_SAME_WHO: i32 = 18;
    pub const COUNT_NON_DECOY_TYPE: i32 = 19;
    pub const COUNT_CATEGORY: i32 = 20;
    pub const COUNT_REGION: i32 = 21;
    pub const COUNT_TRADE: i32 = 22;
    pub const COUNT_GARRISON_ARROWS: i32 = 26;
    pub const COUNT_WHO_INSIDE: i32 = 30;
}

/// `QueuePos` — where `add_order` puts an order in a unit's queue, in the
/// original's numbering (`QUEUE_FIRST` 0, `QUEUE_LAST` 1, `QUEUE_NEW` 2:
/// the listing's `cmovne` in `Group::action_form`).
pub mod queue {
    pub const QUEUE_FIRST: i32 = 0;
    pub const QUEUE_LAST: i32 = 1;
    pub const QUEUE_NEW: i32 = 2;
}

/// `SearchIndex` — `find`'s search mode, its third argument.
pub mod search {
    /// `SEARCH_INDEX_SEARCH_ALL`.
    pub const SEARCH_ALL: i32 = 0;
    /// `SEARCH_FRIENDLY`.
    pub const SEARCH_FRIENDLY: i32 = 1;
}

/// `CityData`'s flags at `+4`.
pub mod city_flag {
    /// The alarm bell is rung.
    pub const CITY_BELL: i32 = 0x40;
}

/// A type's `domain` (`type+0x218`), as the walks over the original read
/// it.
pub mod domain {
    /// An air type.
    pub const DOMAIN_AIR: i32 = 2;
}

/// The bits of `WData.flags`, as `WData::log_data@006af7e0` spells them —
/// each word below is printed exactly when its bit is set, solved from
/// run20's 3,600 cells (`docs/ORACLE.md`, "The map is a dump too";
/// 2026-08-25). `0x1`, `0x400`, `0x800`, `0x1000` and `0x2000` have no word
/// in that dump: the writer tests `0x1000` and `0x2000` and run20 never
/// sets them; `0x400` has no name here. `0x800` is **`OIL`**, settled by
/// `WorldData::is_oil_at@00472af0`, which is that bit and nothing else.
pub mod cell {
    /// The bit `Build::close@00628980` sets on the cell under a building's
    /// position when it closes for any reason but 5, a city's capture
    /// (`:186`, `or $2`): a building lost — destroyed, disbanded, or the old
    /// half of a conversion. `Leader::produce_building@006e1400` is its
    /// only reader in the simulation, and its only clearer: the oil arm
    /// skips a flagged patch for that pass, the spiral a flagged cell with
    /// an enemy object within `0xf00` (`docs/AI.md` §168). The PDB names no
    /// `WData` flag.
    pub const CLOSED: u16 = 0x2;
    /// `COAST` — a land cell of the shore; the muster search's class 3.
    pub const COAST: u16 = 0x4;
    /// `ROCK` — `WorldData::is_rocks@006b4380`; the land class 6, or 7
    /// with [`OIL`].
    pub const ROCK: u16 = 0x8;
    /// `MOUNTAIN` — with the unnamed `0x40`, the muster search's class 5.
    pub const MOUNTAIN: u16 = 0x10;
    /// `FOREST` — the muster search's class 4.
    pub const FOREST: u16 = 0x20;
    /// `ROAD`.
    pub const ROAD: u16 = 0x80;
    /// `HALFLAND` — a coastal cell whose `region2` is the sea region; what
    /// `WorldData::is_ocean` tests first.
    pub const HALFLAND: u16 = 0x100;
    /// `NEARBLOCK`.
    pub const NEARBLOCK: u16 = 0x200;
    /// `WorldData::is_oil_at@00472af0` is `flags & 0x800`, and nothing
    /// else — so the bit the map dump prints no word for is oil. It is
    /// what turns a rock cell into land class 7 and a water cell into an
    /// offshore oil patch ([`crate::world::World::land_class`]).
    pub const OIL: u16 = 0x800;
    /// `BUILDING` — a cell holding a building's centre.
    pub const BUILDING: u16 = 0x4000;
    /// `GOODY`.
    pub const GOODY: u16 = 0x8000;
    /// A cliff's footprint, `Cliffs::add_cliff@008a6690`'s mark — the
    /// bit the map dump prints no word for, which with [`MOUNTAIN`] is the
    /// muster search's class 5.
    pub const CLIFF: u16 = 0x40;
}

/// `CoordInfo.flags` — the bits `crate::terrain_init::coord_flags` sets, and
/// the vertex codes `TerrainOut::get_vert_codes@0086bf70` ORs
/// (`crate::map_heights::code`).
pub mod coord {
    /// A `NONE` cell, or one whose world flags carry 0x4.
    pub const NONE_CELL: u16 = 0x8000;
    /// `NONE`, HALFLAND (0x100) or 0x400: the half-land gate's first bit.
    pub const GATE_HALF: u16 = 0x4;
    /// `BASELAND` with a `NONE` or 0x4 cell among its eight neighbours.
    pub const COASTAL: u16 = 0x20;
    /// `OCEAN`.
    pub const OCEAN: u16 = 0x1000;
    /// `OCEAN` with a `NONE` or 0x4 cell within the 5×5 square: the gate's
    /// second bit. `check_and_set_halfland`'s tail sets it too.
    pub const SHALLOW: u16 = 0x80;
    /// What `check_and_set_halfland_wcoord` tests: `flags & 0x84`.
    pub const GATE_MASK: u16 = GATE_HALF | SHALLOW;
    /// Cleared by `convert_beach_to_sandy` on a cell it converts.
    pub const BEACH: u16 = 0x2000;
    /// A mountain vertex, which `adjust_for_mountains` marks: 100 (the
    /// `CoordInfo` code `0x2`; not `cell::MOUNTAIN`, the `WData` bit).
    pub const MOUNTAIN_VERT: u16 = 0x2;
}

/// `TypeIndex` values the simulation names — the tree index of a type,
/// which on the shipped data **is** the original's `TypeIndex` (goods
/// `0..`, units from `0x32`, buildings from `0x19e`, techs from `0x220`;
/// `rondata::load` builds the tree in that order). `crate::nations::ty`,
/// `crate::transport::ty` and `crate::ai_census::ty` are this module.
///
/// The unit values `Build::activate`'s free-unit block names are each the
/// *base* of a line: the block hands one to `get_graft` and then — all but
/// three arms — to `current_upgrade`, so what is trained is the newest type
/// the player owns in that line, not this one.
pub mod ty {
    use crate::tech::TypeId;
    /// `CARAVAN` (the free-unit block's `CARA`), the Nubians' free caravan
    /// from a Market, and the lineage `is(0x3b)` the transport and census
    /// tests ask.
    pub const CARAVAN: TypeId = 0x3b;
    /// `SCOUT`, the Iroquois' from a Barracks.
    pub const SCOUT: TypeId = 0x45;
    /// `PEASANTS`, the Koreans' from a city.
    pub const PEASANTS: TypeId = 0x32;
    /// `SCHOLARS`, the Americans' from a University.
    pub const SCHOLARS: TypeId = 0x34;
    /// `GENERAL`, the French one from a fort.
    pub const GENERAL: TypeId = 0x36;
    /// `SUPPLYWAGON`, the French one from a Siege Factory.
    pub const SUPPLYWAGON: TypeId = 0x3f;
    /// `SLINGERS`, the Aztecs' from a Barracks.
    pub const SLINGERS: TypeId = 0x52;
    /// `HOPLITES`, the Roman legion.
    pub const HOPLITES: TypeId = 0x84;
    /// `BOWMEN`, the British archers.
    pub const BOWMEN: TypeId = 0xaa;
    /// `HORSEARCHERS`, the Mongols' from a Stable.
    pub const HORSEARCHERS: TypeId = 0xbb;
    /// `CATAPULT`, the Turks' from a Siege Factory.
    pub const CATAPULT: TypeId = 0x109;
    /// `BIPLANE`, the Germans' from an Airbase — the fighter line's head,
    /// which the census's `air` splits on beside [`BOMBER`].
    pub const BIPLANE: TypeId = 0x11f;
    /// `BOMBER`, the Americans' from an Airbase, in the Modern age.
    pub const BOMBER: TypeId = 0x130;
    /// `FISHERMEN`, the British ones from a Dock.
    pub const FISHERMEN: TypeId = 0x13d;
    /// `BARK`, the Dutch light ship.
    pub const BARK: TypeId = 0x143;
    /// `TRIREME`, the Spanish one from a Dock.
    pub const TRIREME: TypeId = 0x154;
    /// `FIRERAFT`, the head of the line the Germans' submarine arms test
    /// by lineage (`is(0x14e, 0)`): the Fire Raft through the Submarine.
    pub const FIRERAFT: TypeId = 0x14e;

    /// `FARM`, which the Japanese Farm arm tests by equality
    /// (`get_cost:1003`–`1008`).
    pub const FARM: TypeId = 0x1a1;
    /// `UNIVERSITY`.
    pub const UNIVERSITY: TypeId = 0x1a4;
    /// `LIBRARY`, which the Greek Library arm tests by equality beside the
    /// University (`get_cost:966`–`972`).
    pub const LIBRARY: TypeId = 0x1b3;
    /// `BARRACKS`.
    pub const BARRACKS: TypeId = 0x1ab;
    /// `STABLE` and `AUTOPLANT`, the Mongol arm's pair.
    pub const STABLES: [TypeId; 2] = [0x1ac, 0x1ad];
    /// `SIEGEFACTORY` and `FACTORY`, the Turkish and French arm's pair.
    pub const SIEGE_FACTORIES: [TypeId; 2] = [0x1ae, 0x1af];
    /// `DOCK`, `ANCHORAGE`, `SHIPYARD` — the naval arm tests all three by
    /// name rather than through the lineage.
    pub const DOCKS: [TypeId; 3] = [0x1b0, 0x1b1, 0x1b2];
    /// `MARKET`.
    pub const MARKET: TypeId = 0x1b4;
    /// `AIRBASE`.
    pub const AIRBASE: TypeId = 0x1bf;
    /// The buildings the Maya arm leaves at full timber, each by type
    /// equality (`get_cost:973`–`987`): `BARRACKS`, `STABLE`, `AUTOPLANT`,
    /// `FACTORY`, `SIEGEFACTORY`, `AIRBASE`, `DOCK`, `ANCHORAGE`,
    /// `SHIPYARD`, `FORTX`, `CASTLE`, `FORTRESS`, `REDOUBT`, `MISSILESILO`,
    /// `TOWER`, `KEEP`, `STOCKADE`, `BUNKER`, `LOOKOUT`, `OBSERVATIONPOST`,
    /// `AIRDEFENSE`, `RADAR`, `SAM`. Wonders are refused by their vslot.
    pub const MAYA_FULL_TIMBER: [TypeId; 23] = [
        0x1ab, 0x1ac, 0x1ad, 0x1af, 0x1ae, 0x1bf, 0x1b0, 0x1b1, 0x1b2, 0x1bb, 0x1bc, 0x1bd, 0x1be,
        0x208, 0x1b7, 0x1b8, 0x1b9, 0x1ba, 0x209, 0x20a, 0x20b, 0x20c, 0x20d,
    ];

    /// `MERCHANT`, `ECHINESEMERCHANT`, `MERCHANT2` — the three base types
    /// the merchant tests compare against exactly (the transport rules and
    /// the census).
    pub const MERCHANTS: [TypeId; 3] = [0x3d, 0x3e, 0x190];
    /// `AIRCRAFTCARRIER` — the one carrier that can never transport.
    pub const AIRCRAFTCARRIER: TypeId = 0x15f;
    /// `TRANSPORTBARGE` — the boat a land unit becomes at the shore (§6).
    pub const TRANSPORTBARGE: TypeId = 0x140;
    /// `MERCHANTFLEET` — a caravan's own boat.
    pub const MERCHANTFLEET: TypeId = 0x13e;
    /// `GULLBIRD` — what a finished dock spawns.
    pub const GULLBIRD: TypeId = 0x194;

    /// `TypeIndex` values the census sweep hardcodes; citizens (`0x32`/
    /// `0x33`) and scholars (`0x34`/`0x35`) are `crate::ai_census::Worker`
    /// instead, which is how `rondata` loads them.
    /// `missile` splits on these two exact types.
    pub const CRUISE: TypeId = 0x139;
    pub const NUKE: TypeId = 0x13b;

    /// Each single-type constant beside its `unitrules.xml` or
    /// `buildingrules.xml` `<NAME>`; `rondata` reads the record at
    /// `id - 0x32` or `id - 0x19e`.
    pub const NAMED_TYPES: [(&str, TypeId); 30] = [
        ("Caravan", CARAVAN),
        ("Scout", SCOUT),
        ("Citizen", PEASANTS),
        ("Scholar", SCHOLARS),
        ("General", GENERAL),
        ("Supply Wagon", SUPPLYWAGON),
        ("Slingers", SLINGERS),
        ("Hoplites", HOPLITES),
        ("Bowmen", BOWMEN),
        ("Horse Archer", HORSEARCHERS),
        ("Catapult", CATAPULT),
        ("Biplane", BIPLANE),
        ("Bomber", BOMBER),
        ("Fishermen", FISHERMEN),
        ("Bark", BARK),
        ("Trireme", TRIREME),
        ("Fire Raft", FIRERAFT),
        ("Aircraft Carrier", AIRCRAFTCARRIER),
        ("Transport Barge", TRANSPORTBARGE),
        ("Merchant Fleet", MERCHANTFLEET),
        ("Gull Bird", GULLBIRD),
        ("V2 Rocket", CRUISE),
        ("Nuclear Missile", NUKE),
        ("Farm", FARM),
        ("University", UNIVERSITY),
        ("Library", LIBRARY),
        ("Barracks", BARRACKS),
        ("Market", MARKET),
        ("Airbase", AIRBASE),
        ("Merchant", MERCHANTS[0]),
    ];
}
