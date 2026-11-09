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
    pub const CIVILIAN: i32 = 0x293;
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
        ("Civilian", CIVILIAN),
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
