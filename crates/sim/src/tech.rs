//! The tech tree: what a player may buy, and what owning it changes.
//!
//! Everything purchasable in Rise of Nations is one entry in a single table of
//! types, every entry carries up to three prerequisites that are entries in
//! the same table, and a player's progress is one bit per entry. Ages and
//! library levels are rows of that table with a few extra rules. The whole
//! mechanic is four predicates over the bits — [`TechTree::has_tech`],
//! [`TechTree::has_preq`], [`TechTree::type_eligible`],
//! [`TechTree::type_avail`] — one procedure that sets a bit and cascades
//! ([`TechTree::gain_tech`]), and some counting. `docs/TECH.md` is the
//! specification and says how each of these was established.
//!
//! The table is data, not code: a [`TechTree`] is built from [`TypeDef`]s the
//! way the original builds its `types` array from the rules files, and every
//! rule that names a specific type — the Market, the Senate, the machine-gun
//! line — does so through [`Roles`], so that a tree without that role leaves
//! the rule inert rather than wrong. The indices are the tree's own; the
//! original's `TypeIndex` ranges are represented by [`Kind`], which is what
//! its `is_unit_type`/`is_age_type`/… predicates test.

use crate::Tuning;

/// An entry of the tree. Its index is its identity, as in the original.
pub type TypeId = usize;

/// A prerequisite slot, as `Types::tech_key` resolves the column: a tech, the
/// word `none` (`TYPE_NONE`, −1), or the word `disable` (`TYPE_ANY_BUILDING`,
/// −2). The three behave differently — see [`TechTree::has_tech`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Preq {
    /// `none`: satisfied.
    #[default]
    None,
    /// `disable`: never satisfied, and refused outright by eligibility.
    Disabled,
    /// A type that must be owned.
    Of(TypeId),
}

/// A library line. The discriminant is the original's `cat`, which indexes
/// `epoch[4]`: `epoch[0]` is the Military level, and it is not the age.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Line {
    Military = 0,
    Civic = 1,
    Commerce = 2,
    Science = 3,
}

impl Line {
    pub const ALL: [Line; 4] = [Line::Military, Line::Civic, Line::Commerce, Line::Science];

    pub const fn index(self) -> usize {
        self as usize
    }

    /// The category the console's four epoch verbs number: `military` 0,
    /// `civic` 1, `commerce` 2, `science` 3 (`run_cmd` cases 0x38–0x3b,
    /// `docs/INPUT.md` §11).
    pub const fn of(cat: i32) -> Option<Line> {
        match cat {
            0 => Some(Line::Military),
            1 => Some(Line::Civic),
            2 => Some(Line::Commerce),
            3 => Some(Line::Science),
            _ => None,
        }
    }
}

/// How many levels a library line has, and how many ages there are.
pub const LEVELS: usize = 7;
/// How many library epochs there are: four lines of seven. Owning them all is
/// the Tech Race victory.
pub const EPOCHS: i32 = 28;

/// The unit-type facts the tree reads, from the `FLAGS` and `OBJ_MASK`
/// columns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnitTraits {
    /// Flag `h` (`unit_flags & 0x80`): free with its prerequisite. Citizens,
    /// scholars, caravans, scouts, and the first unit of every military line.
    pub free: bool,
    /// Flag `j` (`unit_flags & 0x200`): a researched upgrade in a jump chain.
    pub jumpable: bool,
    /// Flag `y` (`unit_flags & 0x1000000`): a nation's unique unit.
    pub unique: bool,
    /// `OBJ_MASK` digit `1` (`0x4000000`): a patriot or scenario hero.
    pub hero: bool,
    /// One of the six government patriots (`0x160–0x165`).
    pub patriot: bool,
    /// A non-zero `ATTACK`: a combat unit. `UnitType::init` gives every
    /// combat unit that names only an age an implicit second prerequisite,
    /// the Military epoch of that age — see [`TechTree::finalize`].
    pub combat: bool,
}

/// What class of thing an entry is — the original's `TypeIndex` range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A resource, `0x00–0x31`.
    Good,
    /// A unit, `0x32–0x191`.
    Unit(UnitTraits),
    /// A building, `0x19e–0x21e`. `auto` is `build_flags & 4`: the type
    /// arrives through `gain_tech` when its prerequisites do, like a free unit.
    Building { auto: bool, wonder: bool },
    /// One of the seven ages, `0x220–0x226`; `0` is Classical.
    Age(u8),
    /// One of the 28 library techs, `0x227–0x242`; `level` 0–6.
    Epoch { line: Line, level: u8 },
    /// One of the four finals, `0x243–0x246`.
    Final,
    /// A building tech, `0x247–0x26e`.
    Plain,
    /// A government, `0x26f–0x274`: three tiers of two columns.
    Gov { tier: u8, column: u8 },
}

impl Kind {
    pub const fn is_unit(self) -> bool {
        matches!(self, Kind::Unit(_))
    }
    pub const fn is_building(self) -> bool {
        matches!(self, Kind::Building { .. })
    }
    /// `is_tech_type`: anything in `0x220–0x274`.
    pub const fn is_tech(self) -> bool {
        matches!(
            self,
            Kind::Age(_) | Kind::Epoch { .. } | Kind::Final | Kind::Plain | Kind::Gov { .. }
        )
    }
    /// `is_plain_tech_type`: a tech that is neither an age nor an epoch.
    pub const fn is_plain_tech(self) -> bool {
        matches!(self, Kind::Final | Kind::Plain | Kind::Gov { .. })
    }
    pub const fn unit(self) -> Option<UnitTraits> {
        match self {
            Kind::Unit(u) => Some(u),
            _ => None,
        }
    }
}

/// One entry of the tree: the `TypeData` fields the predicates read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeDef {
    pub name: String,
    pub kind: Kind,
    /// `PREQ0 PREQ1 PREQ2`. A unit's third slot is never read.
    pub preq: [Preq; 3],
    /// `FROM`: what this upgrades from.
    pub from: Option<TypeId>,
    /// `JUMP`: the next upgrade in a unit's line.
    pub jump: Option<TypeId>,
    /// What this type becomes. A building's `JUMP`; otherwise derived by the
    /// loader from the successor's `FROM` — see [`TechTree::finalize`].
    pub upgrade: Option<TypeId>,
    /// A building's successor by `FROM`, derived by the loader
    /// (`BuildTypeData::to`); the Town's count of upgrades reads it.
    pub to: Option<TypeId>,
    /// `OBS`/`OBSOLETE`: the tech that makes this type obsolete.
    pub obs: Preq,
    /// `WHERE`: the building type that makes it.
    pub where_: Option<TypeId>,
    /// `TRIBE_MASK`, one bit per nation.
    pub tribe_mask: u32,
    /// A tech's `AGE` column.
    pub age: i32,
    /// A unit's `GRAFT`: the nation variant this one is a variant of.
    pub graft: Option<TypeId>,
    /// The `is_list`: classes this type counts as — `is(x, 0)` without a
    /// lineage walk.
    pub is_list: Vec<TypeId>,
    /// `leader_off`: players a scenario has disabled this tech for.
    pub leader_off: u8,
    /// A tech's `COST` column, in the file's units (`docs/COSTS.md`: times
    /// `TECH_COST_FACTOR` when charged). Units and buildings carry theirs on
    /// their own records; this is only read for a tech.
    pub cost: [i32; crate::economy::RESOURCES],
    /// A tech's `JOB_TIME`, in frames — `TypeData::research_time` is this
    /// times a hundred, times `RESEARCH_TICK_PREMIUM` (`docs/PRODUCTION.md`).
    pub job_time: i32,
    /// `TechType::ai[11]` (`+0x1cc`): the eleven weights the production AI
    /// multiplies a technology's base score by, derived at load from what the
    /// tech unlocks — [`crate::ai_load::compute_ai_values`]. Zero on
    /// everything that is not a tech, and `ai[7]` is never written at all.
    pub ai: [i16; crate::ai_load::AI_WEIGHTS],
}

impl TypeDef {
    pub fn new(name: &str, kind: Kind) -> TypeDef {
        TypeDef {
            name: name.to_string(),
            kind,
            preq: [Preq::None; 3],
            from: None,
            jump: None,
            upgrade: None,
            to: None,
            obs: Preq::Disabled,
            where_: None,
            tribe_mask: u32::MAX,
            age: -1,
            graft: None,
            is_list: Vec::new(),
            leader_off: 0,
            cost: [0; crate::economy::RESOURCES],
            job_time: 0,
            ai: [0; crate::ai_load::AI_WEIGHTS],
        }
    }

    pub fn good(name: &str) -> TypeDef {
        TypeDef::new(name, Kind::Good)
    }
    pub fn unit(name: &str, traits: UnitTraits) -> TypeDef {
        TypeDef::new(name, Kind::Unit(traits))
    }
    pub fn building(name: &str) -> TypeDef {
        TypeDef::new(
            name,
            Kind::Building {
                auto: false,
                wonder: false,
            },
        )
    }
    pub fn age(name: &str, n: u8) -> TypeDef {
        let mut t = TypeDef::new(name, Kind::Age(n));
        t.age = n as i32;
        t
    }
    pub fn epoch(name: &str, line: Line, level: u8) -> TypeDef {
        let mut t = TypeDef::new(name, Kind::Epoch { line, level });
        t.age = level as i32;
        t
    }
    pub fn plain(name: &str, age: i32) -> TypeDef {
        let mut t = TypeDef::new(name, Kind::Plain);
        t.age = age;
        t
    }
    pub fn gov(name: &str, tier: u8, column: u8) -> TypeDef {
        TypeDef::new(name, Kind::Gov { tier, column })
    }

    pub fn preqs(mut self, p: [Preq; 3]) -> TypeDef {
        self.preq = p;
        self
    }
    pub fn needs(mut self, slot: usize, t: TypeId) -> TypeDef {
        self.preq[slot] = Preq::Of(t);
        self
    }
    pub fn from(mut self, t: TypeId) -> TypeDef {
        self.from = Some(t);
        self
    }
    pub fn jump(mut self, t: TypeId) -> TypeDef {
        self.jump = Some(t);
        self
    }
    pub fn at(mut self, where_: TypeId) -> TypeDef {
        self.where_ = Some(where_);
        self
    }
    pub fn with_age(mut self, age: i32) -> TypeDef {
        self.age = age;
        self
    }
    pub fn tribes(mut self, mask: u32) -> TypeDef {
        self.tribe_mask = mask;
        self
    }
}

/// A nation, as the tree sees one: its unit substitutions and whether it is
/// a barbarian tribe.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tribe {
    /// `Tribe::graft[352]`: per unit type, the replacement this nation
    /// trains instead. `None` is identity.
    pub graft: Vec<Option<TypeId>>,
    pub barbarian: bool,
    /// The nation's display name — the tribe file's `<TRIBE name>`, what
    /// `ScenarioFuncSet::find_nation` returns and the scripts compare
    /// (`"Egyptians"`, `"Lakota"`, …). Empty when unloaded.
    pub name: String,
    /// `Tribe::unit_continent` (`+0x68`), the tribe file's
    /// `<UNIT_CONTINENT>`: which of the six unit art styles this nation's
    /// units are drawn in. `GraphicPieces::get_unit_gpiece` multiplies it
    /// by [`crate::anim::PIECES_PER_STYLE`], so it is a *simulation* input
    /// and not a drawing one — the piece decides which animation packet a
    /// guy plays and therefore how long every animation of its runs
    /// (`docs/ANIM.md` §3.4). Zero — Europe — when unloaded, which is what
    /// `Tribe::Tribe` initialises it to.
    pub unit_continent: i32,
}

/// The types the rules name by role. Each is optional: a tree without the
/// role leaves the rule inert.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Roles {
    /// `VILLAGE`, the Small City — the root of the city line.
    pub village: Option<TypeId>,
    /// `DOCK`, the root of Anchorage and Shipyard.
    pub dock: Option<TypeId>,
    /// `TRANSPORT_BONUS`'s one prerequisite — the technology whose
    /// ownership is `has_preq(TRANSPORT_BONUS)` (`docs/TRANSPORT.md` §4;
    /// Written Word in the shipped `rules.xml`). `None` leaves the bonus
    /// granted, the tree's rule for a role it does not know.
    pub transport_preq: Option<TypeId>,
    /// `COLONIZE_BONUS`'s one prerequisite — the technology whose ownership
    /// is `has_preq(COLONIZE_BONUS)`, the fourth of `rules.xml`'s
    /// `TECHBONUSES` and **Coinage** in the shipped file
    /// (`docs/CITIES.md` §2.6.1). `None` leaves the bonus granted, the
    /// tree's rule for a role it does not know.
    pub colonize_preq: Option<TypeId>,
    /// `LeaderData::get_gov@006d6a20`'s six tests, in its own order —
    /// `SOCIALISM_1`, `CAPITALISM_1`, `MONARCHY_1`, `DEMOCRACY_1`,
    /// `DESPOTISM_1`, `REPUBLIC_1` — each the bonus's three prerequisites
    /// and the government it answers. `docs/TECH.md` §"The government
    /// patriot".
    pub gov_bonuses: Vec<([Preq; 3], TypeId)>,
    /// `FISHERMEN1`–`FISHERMEN3`' prerequisites, in level order — the
    /// twentieth, twenty-first and twenty-second of `rules.xml`'s
    /// `TECHBONUSES` (`0x2bf`–`0x2c1`; Agriculture, Crop Rotation and Food
    /// Industry as shipped). `LeaderData::get_fishermen@006d6e80` answers
    /// the highest one held, and that indexes `FISHERMEN_BONUS`
    /// (`docs/ECONOMY.md`, step 6). An unknown entry is simply not held.
    pub fishermen_preq: [Option<TypeId>; 3],
    /// `MERCHANTS_1`–`MERCHANTS_4`' prerequisites, in level order —
    /// `0x30f`–`0x312`, Taxation through Income Tax.
    /// `LeaderData::get_merchants_level@006d6dc0` indexes `MERCHANTS_BONUS`
    /// with the highest held, and **level 0 is 100%**, so the whole term is
    /// inert until Taxation.
    pub merchants_preq: [Option<TypeId>; 4],
    /// `REPUBLIC_1`–`REPUBLIC_3`' prerequisites, in level order — bonuses
    /// 113–115 (`0x31d`–`0x31f` off `BASE_BONUSTYPES`), each **Republic**
    /// in the shipped file. `calc_resource_caps@006ce900` asks `has_preq`
    /// of the three from the top down and adds that tier's
    /// `REPUBLIC_COMMERCE_BONUS` to every capped good (`docs/AI.md` §72).
    /// A row the tree does not know is not held.
    pub republic_preq: [Option<TypeId>; 3],
    /// The five *enhancer and taxation* ladders, in level order — the
    /// `TECHBONUSES` rows `docs/ECONOMY.md`'s indexed arithmetic is keyed
    /// by, and the same shape as [`Roles::merchants_preq`]: a row the tree
    /// does not know is simply not held.
    ///
    /// `BASE_BONUSTYPES` is `TypeIndex` `0x2ac`, so these are bonuses 15–18
    /// (`GRANARY2..5`), 22–24 (`LUMBERMILL2..4`), 25–27 (`SMELTER2..4`),
    /// 53–57 (`UNIVERSITY2..6`) and 95–98 (`TAX_1..4`) — read off
    /// `enums/TypeIndex.txt`, and bracketing the two indices `load.rs`
    /// already had right (`FISHERMEN1..3` at 19–21, `MERCHANTS_1..4` at
    /// 99–102).
    ///
    /// `LeaderData::get_granary@006db340`, `CityData::lumber_level@00736820`,
    /// `get_smelter@006db3f0` and `get_university@006db1f0` all answer
    /// **`1`** when none is held; `get_taxation@006d6e20` answers **`0`**.
    /// That is the whole difference between the four enhancers and the tax.
    pub granary_preq: [Option<TypeId>; 4],
    pub lumbermill_preq: [Option<TypeId>; 3],
    pub smelter_preq: [Option<TypeId>; 3],
    pub university_preq: [Option<TypeId>; 5],
    pub taxation_preq: [Option<TypeId>; 4],
    /// `TEMPLEBORDERS2..4` and `FORTBORDERS2..4`, bonuses 28–30 and 37–39
    /// (`0x2c8..0x2ca`, `0x2d1..0x2d3` off `BASE_BONUSTYPES`): Religion,
    /// Monotheism and Existentialism, and Fortification, Bombardment and
    /// Strategic Reserves in the shipped file. `compute_reg_territory@006b0bb0`
    /// asks `has_preq` from the top down, so the level is **`1` plus the
    /// highest held** — `1` meaning none, not "no temple" — and it indexes
    /// `TEMPLE_UPGRADE_TERR` and `FORT_UPGRADE_TERR` and multiplies the
    /// city and fort limit steps (`docs/ATTRITION.md`, "The cost").
    pub temple_borders_preq: [Option<TypeId>; 3],
    pub fort_borders_preq: [Option<TypeId>; 3],
    /// `ATTRITION1..4`, bonuses 49–52 (`0x2dd..0x2e0`): Allegiance, Oath of
    /// Fealty, Patriotism and Nationalism in the shipped file.
    /// `Leader::calc_attrition@006cdea0` counts the **leading run** held —
    /// it breaks at the first `has_preq` that fails — and that count
    /// indexes `ATTRITION_IMPROVED` (`docs/ATTRITION.md`, "Strength").
    pub attrition_preq: [Option<TypeId>; 4],
    pub airbase: Option<TypeId>,
    pub market: Option<TypeId>,
    pub knowledge: Option<TypeId>,
    /// `MATHEMATICS` (`0x228`), which with `type_avail(KNOWLEDGE, 1)` opens
    /// `create_units`' merchant arm (`docs/AI.md` §55).
    pub mathematics: Option<TypeId>,
    pub university: Option<TypeId>,
    pub tower: Option<TypeId>,
    pub fortx: Option<TypeId>,
    pub granary: Option<TypeId>,
    pub lumbermill: Option<TypeId>,
    pub smelter: Option<TypeId>,
    pub metal: Option<TypeId>,
    pub mine: Option<TypeId>,
    pub temple: Option<TypeId>,
    pub refinery: Option<TypeId>,
    pub oilwell: Option<TypeId>,
    pub town: Option<TypeId>,
    pub senate: Option<TypeId>,
    pub machinegun: Option<TypeId>,
    pub rifleman: Option<TypeId>,
    pub infantry: Option<TypeId>,
    pub mechinfantry: Option<TypeId>,
    pub cataphract: Option<TypeId>,
    pub horsearchers: Option<TypeId>,
    pub nuclearmissile: Option<TypeId>,
    pub icbm: Option<TypeId>,
    /// The Tower through Redoubt line (`0x1b7–0x1be`), whose non-age
    /// prerequisites the Romans waive.
    pub fort_line: Vec<TypeId>,
    /// The agriculture, carpentry and metal lines whose Science requirement
    /// the Germans take one level early.
    pub german_industry: Vec<TypeId>,
    /// The five techs every final requires.
    pub final_needs: Vec<TypeId>,
    /// Taxation, Vassalage, Social Contract and Income Tax — `TAXATION`
    /// and the three after it (`0x24d..=0x250`), the window both
    /// `get_cost`'s British discount and `research_techs`' Temple ×100
    /// test as `t − TAXATION < 4`.
    pub taxation_line: Vec<TypeId>,
    /// The seventeen wonders, `PYRAMIDS` through `SPACEPROGRAM`
    /// (`0x20e..=0x21e`), indexed by `TypeIndex − 0x20e` — what
    /// `LeaderData::has_wonder`'s argument names ([`wonder`]).
    pub wonder_line: Vec<TypeId>,
}

/// `LeaderData::has_wonder`'s arguments, as offsets into
/// [`Roles::wonder_line`] (`TypeIndex − BASE_WONDERTYPES`).
pub mod wonder {
    pub const PYRAMIDS: usize = 0x20e - 0x20e;
    pub const COLOSSUS: usize = 0x20f - 0x20e;
    pub const HANGING_GARDENS: usize = 0x210 - 0x20e;
    pub const TIKAL: usize = 0x214 - 0x20e;
    /// The one wonder `has_wonder` holds without a city (`param_1 ==
    /// 0x216`).
    pub const RED_FORT: usize = 0x216 - 0x20e;
    pub const ANGKOR_WAT: usize = 0x217 - 0x20e;
    pub const KREMLIN: usize = 0x21a - 0x20e;
    pub const TAJ_MAHAL: usize = 0x21b - 0x20e;
    pub const EIFFEL_TOWER: usize = 0x21c - 0x20e;
}

/// What gates a free-tech rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    /// A nation power — the tribe's index in the roster.
    Power(usize),
    /// A wonder the player holds.
    Wonder(TypeId),
}

/// How a free-tech block matches a candidate against the tech just gained.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Shape {
    /// The common shape: the candidate's prerequisites are met and one of
    /// them is the tech just gained.
    #[default]
    PreqMatch,
    /// The unit-line blocks (Lakota, Iroquois, Indians, Korean militia): one
    /// of the candidate's two prerequisites is the tech just gained and the
    /// other is owned.
    TwoPreq,
}

/// One of the nation and wonder free-tech blocks at the end of `gain_tech`:
/// for every candidate that matches and is eligible, gain it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreeRule {
    pub gate: Gate,
    /// The `rules.xml` constant behind the block, as loaded: non-zero is on.
    pub enabled: i32,
    pub candidates: Vec<TypeId>,
    pub shape: Shape,
}

/// The lobby's game-rule setting, as far as the tree reads it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Rules {
    #[default]
    Standard,
    /// `game_rules == 4`.
    Deathmatch,
    /// `game_rules == 8`: two starting ages, a defending team.
    BarbariansAtTheGates,
    /// `game_rules == 11`: no finals, no missiles.
    InfoDeathmatch,
}

/// The lobby settings the tree reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Setup {
    /// `starting_technology`: 0 Ancient … 7 Information, 8 All Technologies.
    pub starting_age: i32,
    /// `starting_technology2`, the defenders' extra ages in Barbarians at the
    /// Gates.
    pub starting_age2: i32,
    /// `ending_technology`, **one-based**: 1 Classical … 7 Information.
    pub ending: i32,
    pub rules: Rules,
    /// `info.flags` bit 31, "Early Info Age": no finals.
    pub no_finals: bool,
    /// `info.flags & 4`, "No Nation Powers".
    pub no_nation_powers: bool,
    /// `info.flags & 8`, "No Unique Units".
    pub no_unique_units: bool,
    /// `victory == 9`, the Tech Race: reaching the ending age wins.
    pub tech_race: bool,
}

impl Setup {
    /// A full game from the Ancient age to the Information age.
    pub const STANDARD: Setup = Setup {
        starting_age: 0,
        starting_age2: 0,
        ending: 7,
        rules: Rules::Standard,
        no_finals: false,
        no_nation_powers: false,
        no_unique_units: false,
        tech_race: false,
    };
}

impl Default for Setup {
    fn default() -> Setup {
        Setup::STANDARD
    }
}

/// One player's progress — the bits and counters `LeaderData` keeps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerTech {
    /// `tech`: owned, one bit per type.
    pub tech: Vec<bool>,
    /// `tech_at_start`: the bits `Leader::init` set.
    pub tech_at_start: Vec<bool>,
    /// `obs_flags`: obsolete for this player.
    pub obs: Vec<bool>,
    /// How many of each type this player has queued, for the government
    /// pairing rule — `has_tech_queued`.
    pub queued: Vec<i32>,
    /// `ages`: how many ages are owned.
    pub ages: i32,
    /// `epochs`: how many of the 28 library techs are owned.
    pub epochs: i32,
    /// `discovered`: everything else `gain_tech` counted.
    pub discovered: i32,
    /// `epoch[cat]`: the level on each line, indexed by [`Line`].
    pub epoch: [i32; 4],
    /// `age_stamp`: the frame each age was gained on.
    pub age_stamp: [Option<i64>; LEVELS],
    /// Index into [`TechTree::tribes`].
    pub tribe: usize,
    /// The nation power this player holds, by roster index, if any.
    pub power: Option<usize>,
    /// `get_team()`; in Barbarians at the Gates, team 0 defends.
    pub team: i32,
    /// Whether the player has a city, and started with one — the gate
    /// `has_tribe_bonus` puts on every nation power.
    pub has_city: bool,
    /// Wonders the player holds, for the wonder-gated rules.
    pub wonders: Vec<TypeId>,
    /// `leader_flags2 & 0x1000`: no patriots.
    pub no_patriots: bool,
    /// `leader_flags2 & 0x800`: no governments.
    pub no_governments: bool,
    /// `LeaderData::gov`: the government last gained, if any.
    pub gov: Option<TypeId>,
    /// `LeaderData +0xa48 gov_hero_frame`: −1 until a government patriot
    /// is born (`Unit::init` stamps the frame), 1 for a government held
    /// at frame 0. `docs/TECH.md` §"The government patriot".
    pub gov_hero_frame: i64,
}

impl PlayerTech {
    pub fn new(tree: &TechTree) -> PlayerTech {
        let n = tree.types.len();
        PlayerTech {
            tech: vec![false; n],
            tech_at_start: vec![false; n],
            obs: vec![false; n],
            queued: vec![0; n],
            ages: 0,
            epochs: 0,
            discovered: 0,
            epoch: [0; 4],
            age_stamp: [None; LEVELS],
            tribe: 0,
            power: None,
            team: 0,
            has_city: true,
            wonders: Vec::new(),
            no_patriots: false,
            no_governments: false,
            gov: None,
            gov_hero_frame: -1,
        }
    }

    /// The Military library level, `epoch[0]` — what `POP_CAP` is indexed by.
    pub const fn military_level(&self) -> usize {
        self.epoch[Line::Military.index()] as usize
    }

    /// `has_wonder`.
    pub fn has_wonder(&self, w: TypeId) -> bool {
        self.wonders.contains(&w)
    }
}

/// What `type_avail` answers.
pub const NOT_AVAILABLE: i32 = 0;
/// Prerequisites met and eligible, but the bit is clear: a research job.
pub const RESEARCHABLE: i32 = 2;
/// Owned, or nothing to own: a train job, a build, a purchase.
pub const AVAILABLE: i32 = 4;

/// Something `gain_tech` did that the rest of the simulation acts on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gained {
    /// The bit was set (first time or not). Every gain, cascaded or not,
    /// reports one of these, in the order the original set them.
    Type(TypeId),
    /// A Science epoch arrived: the first library re-prices its queue.
    ScienceEpoch,
    /// A Military epoch arrived: the population cap is recomputed.
    MilitaryEpoch,
    /// A unit type arrived: standing units of `from` (and of every type in
    /// the jump chain below it) convert to it, and so do their queue entries.
    UnitUpgrade { to: TypeId },
    /// Under the Tech Race victory, the age just gained is the ending age.
    TechRaceWon,
}

/// The tree itself: the table of types and the facts the rules hang on it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TechTree {
    pub types: Vec<TypeDef>,
    pub tribes: Vec<Tribe>,
    pub roles: Roles,
    pub free_rules: Vec<FreeRule>,
    /// Per line, the seven epochs from level 0 up — `get_epoch_base` and the
    /// walk `compute_epoch` does.
    pub epochs: [[Option<TypeId>; LEVELS]; 4],
    /// The seven ages, Classical first.
    pub ages: [Option<TypeId>; LEVELS],
    /// The governments by tier and column.
    pub govs: [[Option<TypeId>; 2]; 3],

    // The `rules.xml` constants `has_preq` and `special_preq` read, as loaded
    // (non-zero is on). Copied from a `Tuning` by [`TechTree::with_tuning`];
    // zero — off — by default.
    pub greek_knowledge_early: i32,
    pub greek_university_early: i32,
    pub roman_fort_early: i32,
    pub egyptian_granary_early: i32,
    pub french_lumbermill_early: i32,
    pub german_metal_early: i32,
    pub german_buildings_early: i32,
    pub korean_temple_upgrades: i32,
    pub egyptian_wonders_early: i32,
    pub german_industry_early: i32,
    pub iroquois_govs_early: i32,
}

impl TechTree {
    pub fn new() -> TechTree {
        TechTree {
            tribes: vec![Tribe::default()],
            ..TechTree::default()
        }
    }

    /// Adds a type and returns its id. Ages, epochs and governments are also
    /// filed by position so the line arithmetic can find them.
    pub fn add(&mut self, t: TypeDef) -> TypeId {
        let id = self.types.len();
        match t.kind {
            Kind::Age(n) => self.ages[n as usize] = Some(id),
            Kind::Epoch { line, level } => self.epochs[line.index()][level as usize] = Some(id),
            Kind::Gov { tier, column } => self.govs[tier as usize][column as usize] = Some(id),
            _ => {}
        }
        self.types.push(t);
        id
    }

    pub fn add_tribe(&mut self, tribe: Tribe) -> usize {
        self.tribes.push(tribe);
        self.tribes.len() - 1
    }

    /// What the original's loaders derive after reading the columns, done
    /// once the whole table is in. Call it after the last [`TechTree::add`].
    ///
    /// - `UnitType::init`: a **combat** unit whose `PREQ0` is an age and
    ///   whose `PREQ1` is `none` gets the Military epoch of that age as its
    ///   second prerequisite (`preq[1] = preq[0] + 0x1c`).
    /// - `UnitType::init`: a unit with a `FROM`, unless it is a hero, writes
    ///   itself into its predecessor's `upgrade` — when it has no graft, or
    ///   the predecessor has one, or it is free (`h`); a grafted unit
    ///   inherits its graft's `upgrade` if it has none.
    /// - `BuildType::init`: a building with a `FROM`, unless a hero, is its
    ///   predecessor's `to`, and its `upgrade` too when it carries
    ///   `build_flags & 4`; and a building's `where` is its `FROM`, unless
    ///   it is the base of its line (`basic_type() == self`), when it is
    ///   nothing.
    pub fn finalize(&mut self) {
        let n = self.types.len();
        for u in 0..n {
            let Kind::Unit(traits) = self.types[u].kind else {
                continue;
            };
            if traits.combat
                && let Preq::Of(a) = self.types[u].preq[0]
                && let Some(level) = self.age_of(a)
                && self.types[u].preq[1] == Preq::None
                && let Some(m) = self.epochs[Line::Military.index()][level as usize]
            {
                self.types[u].preq[1] = Preq::Of(m);
            }
            if let Some(f) = self.types[u].from
                && !traits.hero
                && (self.types[u].graft.is_none() || self.types[f].graft.is_some() || traits.free)
            {
                self.types[f].upgrade = Some(u);
            }
            if let Some(g) = self.types[u].graft
                && self.types[u].upgrade.is_none()
            {
                self.types[u].upgrade = self.types[g].upgrade;
            }
        }
        for b in 0..n {
            let Kind::Building { auto, .. } = self.types[b].kind else {
                continue;
            };
            if let Some(f) = self.types[b].from {
                self.types[f].to = Some(b);
                if auto {
                    self.types[f].upgrade = Some(b);
                }
            }
            // `basic_type()`: the root of the `from` chain.
            let mut root = b;
            let mut guard = 0;
            while let Some(f) = self.types[root].from {
                root = f;
                guard += 1;
                if guard > 1024 {
                    break;
                }
            }
            self.types[b].where_ = if root == b { None } else { self.types[b].from };
        }
    }

    pub fn kind(&self, t: TypeId) -> Kind {
        self.types[t].kind
    }

    /// `num_preq()`: two for a unit or a good, three for a building or a
    /// tech.
    pub fn num_preq(&self, t: TypeId) -> usize {
        match self.kind(t) {
            Kind::Unit(_) | Kind::Good => 2,
            _ => 3,
        }
    }

    fn epoch_of(&self, t: TypeId) -> Option<(Line, u8)> {
        match self.kind(t) {
            Kind::Epoch { line, level } => Some((line, level)),
            _ => None,
        }
    }

    fn age_of(&self, t: TypeId) -> Option<u8> {
        match self.kind(t) {
            Kind::Age(n) => Some(n),
            _ => None,
        }
    }

    /// `get_epoch_base(cat)`: the bottom of a line.
    pub fn epoch_base(&self, line: Line) -> Option<TypeId> {
        self.epochs[line.index()][0]
    }

    /// `UnitTypeData::get_military_level_slow@0061d4d0` — the Military
    /// library level a unit type stands at. `Types::finalize_grafting@00669840`
    /// caches it into `military_level` (`+0x2dc`) for every unit type at
    /// load, so the `< 0` arms of `get_military_level` and `get_cost` never
    /// run and this is the whole of it.
    ///
    /// `preq[0]` when it is a Military epoch (`is_epoch_type`, tech `cat`
    /// 0), else `get_preq(1, −1)` — which is `preq[1]` in a game that starts
    /// in the Ancient age and ends in the Information age (the scaled arms
    /// for any other span are not read here: `docs/AI.md` §56.5). The level
    /// is that tech's `TypeIndex − 0x23b` (`leal -0x23b(%esi)` at
    /// `0061d55b`); the line's first tech is `0x23c`, so level **1** is this
    /// crate's zero-based `level` plus one. Anything else is level 0.
    pub fn military_level_of(&self, t: TypeId) -> i32 {
        let military = |p: Preq| match p {
            Preq::Of(x) => match self.kind(x) {
                Kind::Epoch {
                    line: Line::Military,
                    level,
                } => Some(i32::from(level) + 1),
                _ => None,
            },
            _ => None,
        };
        let d = &self.types[t];
        military(d.preq[0])
            .or_else(|| military(d.preq[1]))
            .unwrap_or(0)
    }

    /// A tech's `age` field.
    fn tech_age(&self, t: TypeId) -> i32 {
        self.types[t].age
    }

    // ---- nations ----

    fn tribe(&self, p: &PlayerTech) -> &Tribe {
        &self.tribes[p.tribe]
    }

    /// `LeaderData::has_tribe_bonus(n)`.
    pub fn has_tribe_bonus(&self, setup: &Setup, p: &PlayerTech, n: usize) -> bool {
        if setup.no_nation_powers || !p.has_city {
            return false;
        }
        p.power == Some(n)
    }

    /// `LeaderData::tribe_can_type`: 4 or 0.
    pub fn tribe_can_type(&self, setup: &Setup, p: &PlayerTech, t: TypeId) -> i32 {
        if let Some(u) = self.kind(t).unit()
            && u.unique
            && (setup.no_unique_units || self.tribe(p).barbarian)
        {
            return 0;
        }
        if self.types[t].tribe_mask & (1u32 << (p.tribe as u32 & 31)) != 0 {
            4
        } else {
            0
        }
    }

    /// `LeaderData::get_graft`: the type, or this nation's variant of it.
    pub fn get_graft(&self, setup: &Setup, p: &PlayerTech, t: Option<TypeId>) -> Option<TypeId> {
        let t = t?;
        if self.tribe_can_type(setup, p, t) != 0 {
            return Some(t);
        }
        // `UnitTypeData::type_graft`: the tribe table, for units only.
        if self.kind(t).is_unit()
            && let Some(g) = self.tribe(p).graft.get(t).copied().flatten()
        {
            return Some(g);
        }
        Some(t)
    }

    /// `ObjectTypeData::is(x, strict)`: lineage, not equality.
    pub fn is(&self, t: TypeId, x: TypeId, strict: bool) -> bool {
        self.is_at(t, x, strict, 0)
    }

    fn is_at(&self, t: TypeId, x: TypeId, strict: bool, depth: usize) -> bool {
        if t == x {
            return true;
        }
        // A type the tree does not carry — every hand-built fixture, and
        // any install whose tables this crate has not loaded — can only
        // answer the equality above. The original indexes a real table and
        // has no such case; here the alternative is a panic in a caller
        // that has no business knowing whether a tree was loaded.
        if self.types.get(t).is_none() {
            return false;
        }
        if strict {
            // Strict: units only, the graft, and not a unique target.
            let Some(u) = self.kind(x).unit() else {
                return false;
            };
            return self.kind(t).is_unit() && self.types[t].graft == Some(x) && !u.unique;
        }
        if self.types[t].is_list.contains(&x) {
            return true;
        }
        // `is_slow`: the graft, then the `from` chain.
        if self.types[t].graft == Some(x) {
            return true;
        }
        match self.types[t].from {
            Some(f) if depth < 1024 => self.is_at(f, x, false, depth + 1),
            _ => false,
        }
    }

    // ---- has_tech ----

    /// `LeaderData::has_tech` for a prerequisite slot: `none` is satisfied,
    /// `disable` never is.
    pub fn has_tech_p(&self, setup: &Setup, p: &PlayerTech, q: Preq) -> bool {
        match q {
            Preq::None => true,
            Preq::Disabled => false,
            Preq::Of(t) => self.has_tech(setup, p, t),
        }
    }

    /// `LeaderData::has_tech`: a good is always had; a unit needs its
    /// prerequisites **and** the bit; a building needs only its
    /// prerequisites; a tech, the bit.
    pub fn has_tech(&self, setup: &Setup, p: &PlayerTech, t: TypeId) -> bool {
        match self.kind(t) {
            Kind::Good => true,
            Kind::Unit(_) => self.has_preq(setup, p, t) && p.tech[t],
            Kind::Building { .. } => self.has_preq(setup, p, t),
            _ => p.tech[t],
        }
    }

    /// `get_govs_taken`.
    pub fn govs_taken(&self, setup: &Setup, p: &PlayerTech) -> i32 {
        self.govs
            .iter()
            .flatten()
            .flatten()
            .filter(|&&g| self.has_tech(setup, p, g))
            .count() as i32
    }

    // ---- get_preq ----

    /// `starting_technology`, or in Barbarians at the Gates the smaller of
    /// the two — the `start` the slot-1 remap uses.
    fn remap_start(setup: &Setup) -> i32 {
        if setup.rules == Rules::BarbariansAtTheGates {
            setup.starting_age.min(setup.starting_age2)
        } else {
            setup.starting_age
        }
    }

    /// `TypeData::get_preq(i, who)`: the slot, with the lobby's and one
    /// nation's substitutions. `who` is `None` for the original's `−1`.
    pub fn get_preq(&self, setup: &Setup, who: Option<&PlayerTech>, t: TypeId, i: usize) -> Preq {
        let def = &self.types[t];
        match i {
            0 => def.preq[0],
            1 => self.get_preq_1(setup, who, t),
            2 => {
                let q = def.preq[2];
                let Preq::Of(p) = q else {
                    return q;
                };
                if let Some((line, level)) = self.epoch_of(p)
                    && let Preq::Of(r) = self.get_preq_1(setup, None, t)
                    && let Some((rl, rlevel)) = self.epoch_of(r)
                    && line == rl
                    && level <= rlevel
                {
                    return Preq::None;
                }
                if self.age_of(p).is_some()
                    && def.where_.is_some()
                    && def.where_ == self.roles.university
                    && setup.ending < self.tech_age(p) + 1
                {
                    return Preq::None;
                }
                q
            }
            _ => Preq::None,
        }
    }

    fn get_preq_1(&self, setup: &Setup, who: Option<&PlayerTech>, t: TypeId) -> Preq {
        let def = &self.types[t];
        let mut q = def.preq[1];
        // Iroquois governments one age early — off in the shipped data.
        if let Preq::Of(p) = q
            && let Some(n) = self.age_of(p)
            && matches!(def.kind, Kind::Gov { .. })
            && let Some(w) = who
            && self.has_tribe_bonus(setup, w, 18)
            && self.iroquois_govs_early != 0
        {
            q = match n.checked_sub(1).and_then(|m| self.ages[m as usize]) {
                Some(prev) => Preq::Of(prev),
                None => Preq::None,
            };
        }
        let start = Self::remap_start(setup);
        let Preq::Of(p) = q else {
            return q;
        };
        if start == 0 && setup.ending >= 7 {
            return q;
        }
        if let Some((line, _)) = self.epoch_of(p)
            && def.kind.is_unit()
            && line == Line::Military
        {
            let age = self.tech_age(p);
            if age < start {
                return Preq::None;
            }
            let span = setup.ending - start + 1;
            let n = (28 / span) * (age - start + 1) + 3;
            // `0x23b + n/4`, clamped into the Military line: 0x23b is one
            // below the line's base, so the level is `n/4 - 1`.
            let level = (n.div_euclid(4) - 1).clamp(0, LEVELS as i32 - 1);
            return self.epochs[Line::Military.index()][level as usize]
                .map_or(Preq::None, Preq::Of);
        }
        if start != 0
            && let Some((line, _)) = self.epoch_of(p)
        {
            let age = self.tech_age(p);
            if start <= age && 7 - start > 0 {
                let idx = ((age - start + 1) * 7) / (7 - start) - 1;
                let idx = idx.clamp(0, LEVELS as i32 - 1);
                let remapped = self.epochs[line.index()][idx as usize];
                // `min(idx, preq[1])`: the lower of the two in the line.
                let chosen = match remapped {
                    Some(r) if self.epoch_of(r).map(|e| e.1) < self.epoch_of(p).map(|e| e.1) => r,
                    _ => p,
                };
                if let Preq::Of(p2) = def.preq[2]
                    && let Some((l2, lv2)) = self.epoch_of(p2)
                    && Some(l2) == self.epoch_of(chosen).map(|e| e.0)
                    && lv2 > self.epoch_of(chosen).map_or(0, |e| e.1)
                {
                    return Preq::None;
                }
                return Preq::Of(chosen);
            }
            return Preq::None;
        }
        q
    }

    // ---- has_preq ----

    /// `LeaderData::techs_per_age(age)`: how many library techs an age asks
    /// for, in a full game 2, 6, 10, 14, 18, 22, 26.
    pub fn techs_per_age(&self, setup: &Setup, p: &PlayerTech, age: TypeId) -> i32 {
        let start = if setup.rules == Rules::BarbariansAtTheGates {
            setup.starting_age
        } else {
            self.starting_age(setup, p)
        };
        let a = self.tech_age(age);
        if start == 0 && setup.ending > 6 {
            return a * 4 + 2;
        }
        let span = setup.ending - start + 1;
        if span == 0 {
            return 0;
        }
        let n = (a - start + 1) * (28 / span) - 2;
        if start == 0 && a == 0 && n == 1 { 2 } else { n }
    }

    /// `LeaderData::starting_age`.
    pub fn starting_age(&self, setup: &Setup, p: &PlayerTech) -> i32 {
        if setup.rules == Rules::BarbariansAtTheGates && p.team == 0 {
            let a = setup.starting_age.min(7);
            let b = setup.starting_age2.min(7);
            return (a + b).min(setup.ending);
        }
        setup.starting_age.min(7)
    }

    /// `LeaderData::all_techs`: the "All Technologies" starting setting.
    pub fn all_techs(&self, setup: &Setup, p: &PlayerTech) -> bool {
        if setup.rules == Rules::BarbariansAtTheGates && p.team == 0 {
            return setup.starting_age == 8 || setup.starting_age2 == 8;
        }
        setup.starting_age == 8
    }

    /// `LeaderData::special_preq`: `Some(true)` when the prerequisite is
    /// waived outright; otherwise the (possibly rewritten) prerequisite.
    fn special_preq(
        &self,
        setup: &Setup,
        p: &PlayerTech,
        t: TypeId,
        q: TypeId,
    ) -> Result<Preq, ()> {
        let r = &self.roles;
        // Roman forts: every non-age prerequisite of the fort line.
        if r.fort_line.contains(&t)
            && self.has_tribe_bonus(setup, p, 6)
            && self.roman_fort_early != 0
            && self.age_of(q).is_none()
        {
            return Err(());
        }
        // Bantu: unit upgrades do not require Military research.
        if self.kind(t).is_unit()
            && self.has_tribe_bonus(setup, p, 3)
            && !r.cataphract.is_some_and(|c| self.is(t, c, true))
            && !r.horsearchers.is_some_and(|h| self.is(t, h, true))
            && matches!(self.epoch_of(q), Some((Line::Military, _)))
        {
            return Err(());
        }
        let mut q = Preq::Of(q);
        // Egyptian wonders one age early.
        if let Kind::Building { wonder: true, .. } = self.kind(t)
            && let Preq::Of(x) = q
            && let Some(n) = self.age_of(x)
            && self.has_tribe_bonus(setup, p, 7)
            && self.egyptian_wonders_early != 0
        {
            q = match n.checked_sub(1).and_then(|m| self.ages[m as usize]) {
                Some(prev) => Preq::Of(prev),
                None => Preq::None,
            };
        }
        // German industry one Science level early.
        if r.german_industry.contains(&t)
            && self.german_industry_early != 0
            && self.has_tribe_bonus(setup, p, 12)
            && let Preq::Of(x) = q
            && let Some((Line::Science, level)) = self.epoch_of(x)
        {
            q = match level
                .checked_sub(1)
                .and_then(|m| self.epochs[Line::Science.index()][m as usize])
            {
                Some(prev) => Preq::Of(prev),
                None => Preq::None,
            };
        }
        Ok(q)
    }

    /// `LeaderData::has_preq`: the prerequisites are met.
    pub fn has_preq(&self, setup: &Setup, p: &PlayerTech, t: TypeId) -> bool {
        let r = &self.roles;
        let some = |o: Option<TypeId>| o == Some(t);
        // Nation exceptions that waive the whole check.
        if some(r.market) && self.has_tribe_bonus(setup, p, 4) {
            return true;
        }
        if some(r.knowledge) && self.has_tribe_bonus(setup, p, 5) && self.greek_knowledge_early != 0
        {
            return true;
        }
        if some(r.university)
            && self.has_tribe_bonus(setup, p, 5)
            && self.greek_university_early != 0
        {
            return true;
        }
        if (some(r.tower) || some(r.fortx))
            && self.has_tribe_bonus(setup, p, 6)
            && self.roman_fort_early != 0
        {
            return true;
        }
        if some(r.granary) && self.has_tribe_bonus(setup, p, 7) && self.egyptian_granary_early != 0
        {
            return true;
        }
        if some(r.lumbermill)
            && self.has_tribe_bonus(setup, p, 10)
            && self.french_lumbermill_early != 0
        {
            return true;
        }
        if (some(r.granary) || some(r.lumbermill) || some(r.smelter))
            && self.has_tribe_bonus(setup, p, 12)
            && self.german_buildings_early != 0
        {
            return true;
        }
        if (some(r.metal) || some(r.mine) || some(r.smelter))
            && self.has_tribe_bonus(setup, p, 12)
            && self.german_metal_early != 0
        {
            return true;
        }
        if some(r.temple) && self.has_tribe_bonus(setup, p, 16) && self.korean_temple_upgrades != 0
        {
            return true;
        }
        if some(r.refinery)
            && self.has_tribe_bonus(setup, p, 13)
            && r.oilwell.is_some_and(|o| self.has_preq(setup, p, o))
        {
            return true;
        }
        // The finals ignore their data and require five named techs.
        if self.kind(t) == Kind::Final {
            return r.final_needs.iter().all(|&n| self.has_tech(setup, p, n));
        }

        for i in 0..self.num_preq(t) {
            let q = self.get_preq(setup, Some(p), t, i);
            let q = match q {
                Preq::Of(x) => match self.special_preq(setup, p, t, x) {
                    Err(()) => continue,
                    Ok(q) => q,
                },
                other => other,
            };
            if !self.has_tech_p(setup, p, q) {
                return false;
            }
        }

        // The age quota.
        if let Kind::Age(_) = self.kind(t)
            && p.epochs < self.techs_per_age(setup, p, t)
        {
            return false;
        }

        // Governments tier: either government of the tier below.
        if let Kind::Gov { tier, column } = self.kind(t)
            && tier > 0
        {
            let below = self.govs[tier as usize - 1];
            let same = below[column as usize].is_some_and(|g| self.has_tech(setup, p, g));
            let other = below[1 - column as usize].is_some_and(|g| self.has_tech(setup, p, g));
            return same || other;
        }
        true
    }

    // ---- type_eligible / type_avail ----

    /// `LeaderData::check_predecessor`: the predecessor must be available,
    /// or not be for this nation.
    fn check_predecessor(&self, setup: &Setup, p: &PlayerTech, mut pred: Option<TypeId>) -> bool {
        let mut guard = 0;
        while let Some(x) = pred {
            if self.type_avail(setup, p, x, false) >= 3 {
                break;
            }
            if self.types[x].tribe_mask & (1u32 << (p.tribe as u32 & 31)) != 0 {
                return false;
            }
            pred = self.get_graft(setup, p, self.types[x].from);
            guard += 1;
            if guard > 1024 {
                break;
            }
        }
        true
    }

    /// `LeaderData::type_eligible(t, strict)`: 0 or 4.
    pub fn type_eligible(&self, setup: &Setup, p: &PlayerTech, t: TypeId, strict: bool) -> i32 {
        if self.tribe_can_type(setup, p, t) != 4 {
            return 0;
        }
        // The first city is always eligible; the sim has no cities, so the
        // Town's count is taken as zero and the rule as always true.
        if self.roles.town == Some(t) {
            return 4;
        }
        for i in 0..self.num_preq(t) {
            match self.get_preq(setup, Some(p), t, i) {
                Preq::Disabled => return 0,
                Preq::Of(q) => {
                    if let Some(_) = self.age_of(q)
                        && setup.ending < self.tech_age(q)
                    {
                        return 0;
                    }
                }
                Preq::None => {}
            }
        }
        let def = &self.types[t];
        match def.kind {
            Kind::Good => {
                if strict && self.has_tech_p(setup, p, def.obs) {
                    return 0;
                }
                4
            }
            Kind::Unit(traits) => {
                if (self.roles.nuclearmissile == Some(t) || self.roles.icbm == Some(t))
                    && setup.rules == Rules::InfoDeathmatch
                {
                    return 0;
                }
                if strict {
                    if self.has_tech_p(setup, p, def.obs) || p.obs[t] {
                        return 0;
                    }
                    // The jump rule.
                    let mut u = self.get_graft(setup, p, def.jump);
                    let mut guard = 0;
                    while let Some(x) = u {
                        if self.has_preq(setup, p, x) {
                            if p.tech[x] {
                                return 0;
                            }
                            if self.kind(x).unit().is_some_and(|j| j.jumpable)
                                && !p.tech[t]
                                && !traits.free
                            {
                                return 0;
                            }
                        }
                        u = self.get_graft(setup, p, self.types[x].jump);
                        guard += 1;
                        if guard > 1024 {
                            break;
                        }
                    }
                }
                // The raw slots: an age past the ending age.
                for q in def.preq {
                    if let Preq::Of(x) = q
                        && let Some(n) = self.age_of(x)
                        && setup.ending < n as i32 + 1
                    {
                        return 0;
                    }
                }
                if traits.jumpable {
                    return 4;
                }
                let pred = self.get_graft(setup, p, def.from);
                if self.check_predecessor(setup, p, pred) {
                    4
                } else {
                    0
                }
            }
            Kind::Building { .. } => {
                if strict && (self.has_tech_p(setup, p, def.obs) || p.obs[t]) {
                    return 0;
                }
                4
            }
            Kind::Age(n) => {
                if setup.ending < n as i32 + 1 {
                    return 0;
                }
                4
            }
            Kind::Final => {
                if setup.rules == Rules::InfoDeathmatch || setup.no_finals {
                    return 0;
                }
                4
            }
            Kind::Gov { tier, column } => {
                if let Some(partner) = self.govs[tier as usize][1 - column as usize]
                    && (self.has_tech(setup, p, partner) || p.queued[partner] > 0)
                {
                    return 0;
                }
                4
            }
            Kind::Epoch { .. } | Kind::Plain => 4,
        }
    }

    /// `LeaderData::type_avail(t, strict)`: [`NOT_AVAILABLE`],
    /// [`RESEARCHABLE`] or [`AVAILABLE`].
    pub fn type_avail(&self, setup: &Setup, p: &PlayerTech, t: TypeId, strict: bool) -> i32 {
        if !self.has_preq(setup, p, t) {
            return NOT_AVAILABLE;
        }
        let e = self.type_eligible(setup, p, t, strict);
        if e != 4 {
            return e;
        }
        match self.kind(t) {
            Kind::Unit(traits) => {
                if traits.patriot && p.no_patriots {
                    return NOT_AVAILABLE;
                }
                if !p.tech[t] {
                    return RESEARCHABLE;
                }
                if let Some(mg) = self.roles.machinegun
                    && self.is(t, mg, false)
                {
                    let owns =
                        |r: Option<TypeId>| self.get_graft(setup, p, r).is_some_and(|x| p.tech[x]);
                    if owns(self.roles.rifleman)
                        || owns(self.roles.infantry)
                        || owns(self.roles.mechinfantry)
                    {
                        return AVAILABLE;
                    }
                    return NOT_AVAILABLE;
                }
                AVAILABLE
            }
            Kind::Building { auto, .. } => {
                let def = &self.types[t];
                if def.from.is_some() && auto && !p.tech[t] {
                    return RESEARCHABLE;
                }
                // The Senate's capital walk needs cities; not modelled.
                AVAILABLE
            }
            Kind::Gov { .. } if p.no_governments => NOT_AVAILABLE,
            _ => AVAILABLE,
        }
    }

    /// `BuildTypeData::queue_here`: may a building of type `b` queue `t`.
    pub fn queue_here(&self, b: TypeId, t: TypeId) -> bool {
        let def = &self.types[t];
        if def.preq[0] == Preq::Disabled {
            return false;
        }
        let Some(w) = def.where_ else {
            return false;
        };
        if def.kind.is_unit() {
            self.is(b, w, false) || self.types[w].upgrade.is_some_and(|u| self.is(b, u, false))
        } else {
            self.is(b, w, def.kind.is_building())
        }
    }

    /// `LeaderData::current_upgrade`: the newest available type in `t`'s
    /// line for this player.
    pub fn current_upgrade(&self, setup: &Setup, p: &PlayerTech, t: TypeId) -> TypeId {
        let mut cur = t;
        let mut guard = 0;
        loop {
            let def = &self.types[cur];
            let next = if def.kind.is_unit() {
                let j = self.get_graft(setup, p, def.jump);
                match j {
                    Some(x) if self.has_preq(setup, p, x) && p.tech[x] => Some(x),
                    _ => self.upgrade_step(setup, p, cur),
                }
            } else {
                self.upgrade_step(setup, p, cur)
            };
            match next {
                Some(x) if x != cur => cur = x,
                _ => return cur,
            }
            guard += 1;
            if guard > 1024 {
                return cur;
            }
        }
    }

    fn upgrade_step(&self, setup: &Setup, p: &PlayerTech, cur: TypeId) -> Option<TypeId> {
        let u = self.get_graft(setup, p, self.types[cur].upgrade)?;
        if self.type_avail(setup, p, u, true) < 3 {
            return None;
        }
        Some(u)
    }

    // ---- counting ----

    /// `LeaderData::compute_epoch(cat)`: the contiguous level from the bottom.
    pub fn compute_epoch(&self, setup: &Setup, p: &PlayerTech, line: Line) -> i32 {
        let mut n = 0;
        for e in self.epochs[line.index()].iter().flatten() {
            if !self.has_tech(setup, p, *e) {
                break;
            }
            n += 1;
        }
        n
    }

    /// `Leader::reset_obs_flags`: the obsolete set, from scratch.
    pub fn reset_obs_flags(&self, setup: &Setup, p: &mut PlayerTech) {
        for b in p.obs.iter_mut() {
            *b = false;
        }
        for (b, def) in self.types.iter().enumerate() {
            if let Kind::Building { auto: true, .. } = def.kind
                && let Some(from) = def.from
                && self.has_preq(setup, p, b)
            {
                p.obs[from] = true;
            }
        }
        let units: Vec<TypeId> = (0..self.types.len())
            .filter(|&u| self.kind(u).is_unit())
            .collect();
        for &u in &units {
            if !p.tech[u] {
                continue;
            }
            for &v in &units {
                if self.get_graft(setup, p, self.types[v].jump) == Some(u) {
                    p.obs[v] = true;
                }
            }
            if let Some(f) = self.get_graft(setup, p, self.types[u].from) {
                p.obs[f] = true;
            }
        }
    }

    /// Does `u`'s jump chain (grafted) reach `t`?
    pub fn jumps_to(&self, setup: &Setup, p: &PlayerTech, u: TypeId, t: TypeId) -> bool {
        let mut j = self.get_graft(setup, p, self.types[u].jump);
        let mut guard = 0;
        while let Some(x) = j {
            if x == t {
                return true;
            }
            j = self.get_graft(setup, p, self.types[x].jump);
            guard += 1;
            if guard > 1024 {
                break;
            }
        }
        false
    }

    // ---- gain_tech ----

    /// `Leader::gain_tech(t, …, announce, 1)`: the bit, the counters, and the
    /// cascades. Everything the rest of the simulation must act on comes
    /// back as [`Gained`] events, in order.
    pub fn gain_tech(
        &self,
        setup: &Setup,
        p: &mut PlayerTech,
        t: TypeId,
        frame: i64,
    ) -> Vec<Gained> {
        // `Leader::gain_tech@006dcb60:161`, ahead of everything: a
        // government held at frame 0 counts as its patriot already born.
        if matches!(self.kind(t), Kind::Gov { .. }) && frame == 0 {
            p.gov_hero_frame = 1;
        }
        let mut out = Vec::new();
        self.gain(setup, p, t, frame, &mut out, 0);
        out
    }

    /// `LeaderData::get_gov@006d6a20`: the first of the six government
    /// bonuses whose prerequisites the player holds, as its government —
    /// not `LeaderData::gov`, which is the last one gained.
    pub fn get_gov(&self, setup: &Setup, p: &PlayerTech) -> Option<TypeId> {
        self.roles
            .gov_bonuses
            .iter()
            .find(|(preqs, _)| preqs.iter().all(|&q| self.has_tech_p(setup, p, q)))
            .map(|&(_, g)| g)
    }

    /// `LeaderData::get_gov_hero@006e0600`: the first unit type, in index
    /// order below `0x192`, whose `unit_flags` carry the patriot bit
    /// (`0x4000000`, `FLAGS` digit `1`) and one of whose three
    /// prerequisites is [`Self::get_gov`]. None with `leader_flags2 &
    /// 0x1000` (no patriots) or no government.
    pub fn get_gov_hero(&self, setup: &Setup, p: &PlayerTech) -> Option<TypeId> {
        if p.no_patriots {
            return None;
        }
        let gov = self.get_gov(setup, p)?;
        (0..self.types.len().min(0x192)).find(|&u| {
            self.kind(u).unit().is_some_and(|x| x.patriot)
                && (0..3).any(|i| self.get_preq(setup, None, u, i) == Preq::Of(gov))
        })
    }

    fn gain(
        &self,
        setup: &Setup,
        p: &mut PlayerTech,
        t: TypeId,
        frame: i64,
        out: &mut Vec<Gained>,
        depth: usize,
    ) {
        if depth > 64 {
            return;
        }
        let kind = self.kind(t);
        // 1. A Science epoch re-prices the first library's queue.
        if let Kind::Epoch {
            line: Line::Science,
            ..
        } = kind
            && frame != 0
        {
            out.push(Gained::ScienceEpoch);
        }
        // 2. Counters, only on a first gain.
        if !self.has_tech(setup, p, t) {
            match kind {
                Kind::Age(n) => {
                    p.ages += 1;
                    p.age_stamp[n as usize] = Some(frame);
                }
                Kind::Epoch { line, .. } => {
                    p.epochs += 1;
                    p.epoch[line.index()] += 1;
                }
                _ => p.discovered += 1,
            }
        }
        // 3. The bit.
        p.tech[t] = true;
        out.push(Gained::Type(t));

        match kind {
            Kind::Unit(traits) => {
                // 7. Predecessors are owned and obsolete at once, unless `t`
                // is a hero; standing units convert.
                out.push(Gained::UnitUpgrade { to: t });
                if !traits.hero {
                    let from = self.get_graft(setup, p, self.types[t].from);
                    for u in 0..self.types.len() {
                        if !self.kind(u).is_unit() {
                            continue;
                        }
                        if Some(u) == from || self.jumps_to(setup, p, u, t) {
                            p.tech[u] = true;
                            p.obs[u] = true;
                        }
                    }
                }
                return;
            }
            Kind::Building { .. } => {
                // 8. The type it upgrades from is obsolete.
                if let Some(from) = self.types[t].from {
                    p.obs[from] = true;
                }
                return;
            }
            _ => {}
        }

        // 9. An epoch.
        if let Kind::Epoch {
            line: Line::Military,
            ..
        } = kind
        {
            out.push(Gained::MilitaryEpoch);
        }
        if let Kind::Gov { .. } = kind {
            p.gov = Some(t);
        }
        if matches!(kind, Kind::Age(_)) && setup.tech_race && p.ages == setup.ending {
            out.push(Gained::TechRaceWon);
        }

        // 11. Buildings cascade.
        for b in 0..self.types.len() {
            let Kind::Building { auto, .. } = self.kind(b) else {
                continue;
            };
            if !self.has_preq(setup, p, b) {
                continue;
            }
            let hit =
                (0..self.num_preq(b)).any(|i| self.get_preq(setup, Some(p), b, i) == Preq::Of(t));
            if hit && auto && self.type_eligible(setup, p, b, true) != 0 {
                self.gain(setup, p, b, frame, out, depth + 1);
            }
        }
        // 12. Units cascade: the free unit of every line.
        for u in 0..self.types.len() {
            let Kind::Unit(traits) = self.kind(u) else {
                continue;
            };
            let hit = self.get_preq(setup, Some(p), u, 0) == Preq::Of(t)
                || self.get_preq(setup, Some(p), u, 1) == Preq::Of(t);
            if hit
                && self.has_preq(setup, p, u)
                && !self.has_tech(setup, p, u)
                && traits.free
                && !traits.hero
                && self.type_eligible(setup, p, u, true) != 0
            {
                self.gain(setup, p, u, frame, out, depth + 1);
            }
        }
        // 13. Nation and wonder free techs.
        for rule in &self.free_rules {
            if rule.enabled == 0 {
                continue;
            }
            let open = match rule.gate {
                Gate::Power(n) => self.has_tribe_bonus(setup, p, n),
                Gate::Wonder(w) => p.has_wonder(w),
            };
            if !open {
                continue;
            }
            for &c in &rule.candidates {
                let hit = match rule.shape {
                    Shape::PreqMatch => {
                        self.has_preq(setup, p, c)
                            && (0..self.num_preq(c))
                                .any(|i| self.get_preq(setup, Some(p), c, i) == Preq::Of(t))
                    }
                    Shape::TwoPreq => {
                        let q0 = self.get_preq(setup, Some(p), c, 0);
                        let q1 = self.get_preq(setup, Some(p), c, 1);
                        (q0 == Preq::Of(t) && self.has_tech_p(setup, p, q1))
                            || (q1 == Preq::Of(t) && self.has_tech_p(setup, p, q0))
                    }
                };
                if hit && self.type_eligible(setup, p, c, true) != 0 {
                    self.gain(setup, p, c, frame, out, depth + 1);
                }
            }
        }
    }

    // ---- lose_tech ----

    /// `TechType::is_ultimate_preq(t, y)`: does `t` appear in `y`'s
    /// prerequisite closure.
    fn is_ultimate_preq(&self, setup: &Setup, t: TypeId, y: TypeId, depth: usize) -> bool {
        if depth > 64 {
            return false;
        }
        for i in 0..self.num_preq(y) {
            if let Preq::Of(q) = self.get_preq(setup, None, y, i)
                && (q == t || self.is_ultimate_preq(setup, t, q, depth + 1))
            {
                return true;
            }
        }
        false
    }

    /// `Leader::lose_tech`.
    pub fn lose_tech(&self, setup: &Setup, p: &mut PlayerTech, t: TypeId) {
        self.lose(setup, p, t, 0);
        self.reset_obs_flags(setup, p);
    }

    fn lose(&self, setup: &Setup, p: &mut PlayerTech, t: TypeId, depth: usize) {
        if depth > 64 {
            return;
        }
        let kind = self.kind(t);
        if kind.is_unit() {
            if self.has_tech(setup, p, t) {
                p.discovered -= 1;
            }
            p.tech[t] = false;
            return;
        }
        if !kind.is_tech() {
            // A building, good, spell or bonus: the bit is not touched.
            return;
        }
        p.tech[t] = false;
        for y in 0..self.types.len() {
            if !self.kind(y).is_tech() || !p.tech[y] {
                continue;
            }
            let dependent = match (self.age_of(t), self.age_of(y)) {
                (Some(a), Some(b)) => a <= b,
                _ => self.is_ultimate_preq(setup, t, y, 0),
            };
            if dependent {
                self.lose(setup, p, y, depth + 1);
            }
        }
        match kind {
            Kind::Age(_) => {
                p.ages = self
                    .ages
                    .iter()
                    .flatten()
                    .filter(|&&a| self.has_tech(setup, p, a))
                    .count() as i32;
            }
            Kind::Epoch { line, .. } => {
                p.epochs -= 1;
                p.epoch[line.index()] = self.compute_epoch(setup, p, line);
            }
            _ => p.discovered -= 1,
        }
    }

    /// `Leader::set_age(n)`: own exactly the ages below `n`, and drop what no
    /// longer follows.
    pub fn set_age(&self, setup: &Setup, p: &mut PlayerTech, n: i32, frame: i64) -> Vec<Gained> {
        let n = n.clamp(0, LEVELS as i32) as usize;
        let mut out = Vec::new();
        for &a in self.ages[n..].iter().rev().flatten() {
            if self.has_tech(setup, p, a) {
                self.lose(setup, p, a, 0);
            }
        }
        for &a in self.ages[..n].iter().flatten() {
            if !self.has_tech(setup, p, a) {
                self.gain(setup, p, a, frame, &mut out, 0);
            }
        }
        self.drop_unfollowed(setup, p);
        out
    }

    /// `Leader::set_epoch(line, level)`.
    pub fn set_epoch(
        &self,
        setup: &Setup,
        p: &mut PlayerTech,
        line: Line,
        level: i32,
        frame: i64,
    ) -> Vec<Gained> {
        let level = level.clamp(0, LEVELS as i32) as usize;
        let mut out = Vec::new();
        let row = self.epochs[line.index()];
        for &e in row[level..].iter().rev().flatten() {
            if self.has_tech(setup, p, e) {
                self.lose(setup, p, e, 0);
            }
        }
        for &e in row[..level].iter().flatten() {
            if !self.has_tech(setup, p, e) {
                self.gain(setup, p, e, frame, &mut out, 0);
            }
        }
        self.drop_unfollowed(setup, p);
        out
    }

    /// The tail of `set_age`/`set_epoch`: every owned non-age tech, unit and
    /// building whose prerequisites no longer hold is lost.
    pub(crate) fn drop_unfollowed(&self, setup: &Setup, p: &mut PlayerTech) {
        for t in 0..self.types.len() {
            let k = self.kind(t);
            if k.is_tech() && !matches!(k, Kind::Age(_)) {
                if self.has_tech(setup, p, t) && !self.has_preq(setup, p, t) {
                    self.lose(setup, p, t, 0);
                }
            } else if (k.is_unit() || k.is_building()) && p.tech[t] && !self.has_preq(setup, p, t) {
                self.lose(setup, p, t, 0);
            }
        }
        self.reset_obs_flags(setup, p);
    }

    // ---- the starting position ----

    /// `Leader::init`'s tech block: the bits a player starts with.
    pub fn start(&self, setup: &Setup, tuning: &Tuning, p: &mut PlayerTech) -> Vec<Gained> {
        let start_age = self.starting_age(setup, p);
        p.ages = start_age;
        p.discovered = 0;
        p.epochs = 0;
        for b in p.tech.iter_mut() {
            *b = false;
        }
        for b in p.tech_at_start.iter_mut() {
            *b = false;
        }
        let all = self.all_techs(setup, p);
        for t in 0..self.types.len() {
            let kind = self.kind(t);
            if !kind.is_tech() || matches!(kind, Kind::Final | Kind::Gov { .. }) {
                continue;
            }
            if !(all || self.tech_age(t) < start_age) {
                continue;
            }
            if !all {
                match kind {
                    Kind::Epoch { .. } => {
                        if setup.rules != Rules::BarbariansAtTheGates
                            || p.team != 0
                            || start_age < self.tech_age(t)
                        {
                            continue;
                        }
                    }
                    Kind::Age(_) => {}
                    _ => {
                        let blocked = (0..self.num_preq(t)).any(|i| {
                            match self.get_preq(setup, Some(p), t, i) {
                                Preq::Disabled => true,
                                Preq::Of(q) => self.epoch_of(q).is_some(),
                                Preq::None => false,
                            }
                        });
                        if blocked {
                            continue;
                        }
                    }
                }
            }
            p.tech[t] = true;
            p.tech_at_start[t] = true;
            match kind {
                Kind::Age(_) => {}
                Kind::Epoch { .. } => p.epochs += 1,
                _ => p.discovered += 1,
            }
        }
        for line in Line::ALL {
            p.epoch[line.index()] = self.compute_epoch(setup, p, line);
        }
        for t in 0..self.types.len() {
            match self.kind(t) {
                Kind::Building { .. } => {
                    if self.type_avail(setup, p, t, true) != 0 {
                        p.tech[t] = true;
                    }
                }
                Kind::Unit(_) => {
                    if !self.has_preq(setup, p, t) || self.tribe_can_type(setup, p, t) == 0 {
                        continue;
                    }
                    if !all && let Some(from) = self.get_graft(setup, p, self.types[t].from) {
                        let wanted = match setup.rules {
                            Rules::InfoDeathmatch | Rules::Deathmatch => start_age,
                            _ => start_age - 1,
                        };
                        let behind = (0..self.num_preq(from)).any(|i| {
                            matches!(self.get_preq(setup, Some(p), from, i), Preq::Of(q)
                                if self.age_of(q).is_some() && self.tech_age(q) == wanted)
                        });
                        if behind {
                            continue;
                        }
                    }
                    p.tech[t] = true;
                    p.tech_at_start[t] = true;
                }
                _ => {}
            }
        }
        self.reset_obs_flags(setup, p);
        // The nation starting techs.
        let mut out = Vec::new();
        let mut free_epoch = |this: &TechTree, p: &mut PlayerTech, power, on: i32, line: Line| {
            if this.has_tribe_bonus(setup, p, power) && on != 0 {
                let level = p.epoch[line.index()];
                if level < LEVELS as i32
                    && let Some(e) = this.epochs[line.index()][level as usize]
                {
                    this.gain(setup, p, e, 0, &mut out, 0);
                }
            }
        };
        free_epoch(self, p, 22, tuning.dutch_free_commerce, Line::Commerce);
        free_epoch(self, p, 13, tuning.russian_free_civic, Line::Civic);
        free_epoch(self, p, 0, tuning.aztec_free_military, Line::Military);
        free_epoch(self, p, 6, tuning.roman_free_military, Line::Military);
        free_epoch(self, p, 20, tuning.americans_free_science, Line::Science);
        if self.has_tribe_bonus(setup, p, 23)
            && tuning.persians_despotism != 0
            && let Some(g) = self.govs[0][0]
        {
            self.gain(setup, p, g, 0, &mut out, 0);
        }
        out
    }
}

// The nation-power constants `has_preq` and `special_preq` read. They sit on
// the tree rather than being passed on every call, set from a `Tuning` by
// [`TechTree::with_tuning`].
impl TechTree {
    /// Copies the prerequisite-waiver constants out of a [`Tuning`].
    pub fn with_tuning(mut self, t: &Tuning) -> TechTree {
        self.greek_knowledge_early = t.greek_knowledge_early;
        self.greek_university_early = t.greek_university_early;
        self.roman_fort_early = t.roman_fort_early;
        self.egyptian_granary_early = t.egyptian_granary_early;
        self.french_lumbermill_early = t.french_lumbermill_early;
        self.german_metal_early = t.german_metal_early;
        self.german_buildings_early = t.german_buildings_early;
        self.korean_temple_upgrades = t.korean_temple_upgrades;
        self.egyptian_wonders_early = t.egyptian_wonders_early;
        self.german_industry_early = t.german_industry_early;
        self.iroquois_govs_early = t.iroquois_govs_early;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tree with the shape of the shipped one and a handful of its rows:
    /// the seven ages, four lines of seven epochs, the four finals, a few
    /// building techs, the governments, a Barracks and a Library and a Town,
    /// the Hoplites → Phalanx → Pikemen line, a free Citizen, the Catapult →
    /// Trebuchet line, and a machine gun.
    struct Fixture {
        tree: TechTree,
        knowledge: TypeId,
        ages: [TypeId; LEVELS],
        epochs: [[TypeId; LEVELS]; 4],
        finals: [TypeId; 4],
        fortification: TypeId,
        herbal_lore: TypeId,
        medicine: TypeId,
        govs: [[TypeId; 2]; 3],
        barracks: TypeId,
        library: TypeId,
        town: TypeId,
        hoplites: TypeId,
        phalanx: TypeId,
        pikemen: TypeId,
        citizen: TypeId,
        catapult: TypeId,
        trebuchet: TypeId,
        machinegun: TypeId,
        rifleman: TypeId,
    }

    const H: UnitTraits = UnitTraits {
        free: true,
        jumpable: false,
        unique: false,
        hero: false,
        patriot: false,
        combat: true,
    };
    const J: UnitTraits = UnitTraits {
        free: false,
        jumpable: true,
        unique: false,
        hero: false,
        patriot: false,
        combat: true,
    };
    const PLAIN: UnitTraits = UnitTraits {
        free: false,
        jumpable: false,
        unique: false,
        hero: false,
        patriot: false,
        combat: true,
    };

    fn fixture() -> Fixture {
        let mut t = TechTree::new();
        let _food = t.add(TypeDef::good("Food"));
        let mut ages = [0; LEVELS];
        let names = [
            "Classical Age",
            "Medieval Age",
            "Gunpowder Age",
            "Enlightenment Age",
            "Industrial Age",
            "Modern Age",
            "Information Age",
        ];
        for (i, n) in names.iter().enumerate() {
            let mut d = TypeDef::age(n, i as u8);
            if i > 0 {
                d = d.needs(0, ages[i - 1]);
            }
            ages[i] = t.add(d);
        }
        let knowledge = t.add(TypeDef::good("Knowledge").needs(0, ages[0]));
        let mut epochs = [[0; LEVELS]; 4];
        for line in Line::ALL {
            for level in 0..LEVELS {
                let mut d = TypeDef::epoch(&format!("{line:?} {level}"), line, level as u8);
                if level > 0 {
                    d = d.needs(0, epochs[line.index()][level - 1]);
                }
                epochs[line.index()][level] = t.add(d);
            }
        }
        let mil = epochs[Line::Military.index()];
        let sci = epochs[Line::Science.index()];
        let com = epochs[Line::Commerce.index()];
        let civ = epochs[Line::Civic.index()];
        let mut finals = [0; 4];
        for (i, n) in [
            "Missile Shield",
            "Global Government",
            "Virtual Reality",
            "Nanotechnology",
        ]
        .iter()
        .enumerate()
        {
            finals[i] = t.add(
                TypeDef::new(n, Kind::Final)
                    .with_age(7)
                    .needs(0, mil[6])
                    .needs(1, ages[6]),
            );
        }
        t.roles.final_needs = vec![ages[6], sci[6], com[6], civ[6], mil[6]];
        let fortification = t.add(TypeDef::plain("Fortification", 1).needs(0, mil[1]));
        let herbal_lore = t.add(TypeDef::plain("Herbal Lore", 0).needs(0, sci[0]));
        let medicine = t.add(
            TypeDef::plain("Medicine", 2)
                .needs(0, herbal_lore)
                .needs(1, sci[2]),
        );
        let mut govs = [[0; 2]; 3];
        let gov_names = [
            ["Despotism", "Republic"],
            ["Monarchy", "Democracy"],
            ["Socialism", "Capitalism"],
        ];
        for tier in 0..3 {
            for col in 0..2 {
                let age = [0, 1, 3][tier];
                govs[tier][col] = t.add(
                    TypeDef::gov(gov_names[tier][col], tier as u8, col as u8)
                        .with_age(age as i32)
                        .needs(1, ages[age]),
                );
            }
        }
        let town = t.add(TypeDef::building("Town"));
        let barracks = t.add(TypeDef::building("Barracks"));
        let library = t.add(TypeDef::building("Library"));
        let citizen = t.add(TypeDef::unit("Citizen", H).at(town));
        let hoplites = t.add(TypeDef::unit("Hoplites", H).at(barracks));
        let phalanx = t.add(
            TypeDef::unit("Phalanx", J)
                .at(barracks)
                .from(hoplites)
                .needs(0, ages[0]),
        );
        let pikemen = t.add(
            TypeDef::unit("Pikemen", J)
                .at(barracks)
                .from(phalanx)
                .needs(0, ages[1]),
        );
        t.types[hoplites].jump = Some(phalanx);
        t.types[phalanx].jump = Some(pikemen);
        let catapult = t.add(
            TypeDef::unit("Catapult", H)
                .at(barracks)
                .needs(0, ages[0])
                .needs(1, mil[0]),
        );
        let trebuchet = t.add(
            TypeDef::unit("Trebuchet", PLAIN)
                .at(barracks)
                .from(catapult)
                .needs(0, ages[1])
                .needs(1, mil[1]),
        );
        t.types[catapult].jump = Some(trebuchet);
        let rifleman = t.add(TypeDef::unit("Rifleman", J).at(barracks).needs(0, ages[4]));
        let machinegun = t.add(
            TypeDef::unit("Machine Gun", H)
                .at(barracks)
                .needs(0, ages[4]),
        );
        t.roles.machinegun = Some(machinegun);
        t.roles.rifleman = Some(rifleman);
        t.roles.town = Some(town);
        t.roles.knowledge = Some(knowledge);
        Fixture {
            tree: t,
            knowledge,
            ages,
            epochs,
            finals,
            fortification,
            herbal_lore,
            medicine,
            govs,
            barracks,
            library,
            town,
            hoplites,
            phalanx,
            pikemen,
            citizen,
            catapult,
            trebuchet,
            machinegun,
            rifleman,
        }
    }

    fn fresh(f: &Fixture) -> PlayerTech {
        let mut p = PlayerTech::new(&f.tree);
        f.tree.start(&Setup::STANDARD, &Tuning::RON, &mut p);
        p
    }

    /// Gains the epochs an age asks for, then the age, up to `to`.
    fn age_up(f: &Fixture, p: &mut PlayerTech, to: usize) {
        let s = Setup::STANDARD;
        for a in 0..=to {
            let need = f.tree.techs_per_age(&s, p, f.ages[a]);
            let mut level = 0;
            while p.epochs < need {
                for line in Line::ALL {
                    if p.epochs < need {
                        f.tree.gain_tech(&s, p, f.epochs[line.index()][level], 1);
                    }
                }
                level += 1;
            }
            assert!(f.tree.has_preq(&s, p, f.ages[a]), "age {a} preqs");
            f.tree.gain_tech(&s, p, f.ages[a], 1);
        }
    }

    #[test]
    fn the_full_game_asks_for_two_then_four_more_library_techs_per_age() {
        let f = fixture();
        let p = fresh(&f);
        let s = Setup::STANDARD;
        let quotas: Vec<i32> = f
            .ages
            .iter()
            .map(|&a| f.tree.techs_per_age(&s, &p, a))
            .collect();
        assert_eq!(quotas, [2, 6, 10, 14, 18, 22, 26]);
        // And the quota is what gates the age, not its data prerequisites:
        // Classical has none, and is still refused until two epochs are owned.
        assert!(!f.tree.has_preq(&s, &p, f.ages[0]));
        let mut p = p;
        f.tree
            .gain_tech(&s, &mut p, f.epochs[Line::Science.index()][0], 1);
        assert!(!f.tree.has_preq(&s, &p, f.ages[0]));
        f.tree
            .gain_tech(&s, &mut p, f.epochs[Line::Military.index()][0], 1);
        assert!(f.tree.has_preq(&s, &p, f.ages[0]));
        assert_eq!(p.epochs, 2);
        assert_eq!(p.epoch, [1, 0, 0, 1]);
    }

    #[test]
    fn a_shortened_game_rescales_the_quota() {
        let f = fixture();
        let p = fresh(&f);
        // Start at Medieval, end at Industrial: span 4, 28/4 = 7 per age.
        let s = Setup {
            starting_age: 2,
            ending: 5,
            ..Setup::STANDARD
        };
        let quotas: Vec<i32> = f
            .ages
            .iter()
            .map(|&a| f.tree.techs_per_age(&s, &p, a))
            .collect();
        // (age - 2 + 1) * 7 - 2: the owned ages come out negative, which the
        // original never asks about.
        assert_eq!(quotas, [-9, -2, 5, 12, 19, 26, 33]);
    }

    #[test]
    fn has_tech_reads_none_disabled_goods_units_and_buildings_differently() {
        let f = fixture();
        let s = Setup::STANDARD;
        let mut p = fresh(&f);
        assert!(f.tree.has_tech_p(&s, &p, Preq::None));
        assert!(!f.tree.has_tech_p(&s, &p, Preq::Disabled));
        // A good is always had, even Knowledge before Classical.
        assert!(f.tree.has_tech(&s, &p, f.knowledge));
        // A building is had on its prerequisites alone; the bit is not read.
        p.tech[f.barracks] = false;
        assert!(f.tree.has_tech(&s, &p, f.barracks));
        // A unit needs its prerequisites and the bit. Phalanx: bit clear.
        assert!(!f.tree.has_tech(&s, &p, f.phalanx));
        p.tech[f.phalanx] = true;
        assert!(!f.tree.has_tech(&s, &p, f.phalanx), "Classical not owned");
        age_up(&f, &mut p, 0);
        assert!(f.tree.has_tech(&s, &p, f.phalanx));
    }

    #[test]
    fn the_starting_position_owns_the_free_units_and_nothing_researched() {
        let f = fixture();
        let p = fresh(&f);
        // Hoplites, Citizen: no prerequisites, free. Phalanx needs Classical.
        assert!(p.tech[f.hoplites]);
        assert!(p.tech[f.citizen]);
        assert!(!p.tech[f.phalanx]);
        assert!(p.tech_at_start[f.hoplites]);
        // Buildings with their prerequisites met get the bit.
        assert!(p.tech[f.barracks]);
        assert!(p.tech[f.library]);
        assert!(p.tech[f.town]);
        assert_eq!((p.ages, p.epochs, p.discovered), (0, 0, 0));
        // Hoplites owned marks nothing obsolete yet: Phalanx jumps from it,
        // but Hoplites' own `from` is nothing.
        assert!(!p.obs[f.hoplites]);
    }

    /// **The opening unit set is a function of the nation.** `Leader::init`
    /// gives a unit type its bit on `has_preq` *and* `tribe_can_type`, so
    /// two leaders of the same tree and the same age start owning different
    /// units — the generic line for a nation with no variant, the variant
    /// for the one that has it, and never both. `rondata::diff::build_sim`
    /// laid the position down before it had read the dump's `tribe` until
    /// 2026-09-04 (`docs/TECH.md`, "The starting position is a function of
    /// the nation"), which is what this pins.
    #[test]
    fn the_starting_units_follow_the_leader_s_nation() {
        let f = fixture();
        let mut tree = f.tree.clone();
        // A generic Ancient unit whose mask clears tribe 0, and tribe 0's
        // own variant of it — `Slingers` and `Atl-Atls`, in miniature.
        let generic = tree.add(TypeDef::unit("Slingers", H).at(f.barracks).tribes(!1u32));
        let variant = tree.add(TypeDef::unit("Atl-Atls", H).at(f.barracks).tribes(1));
        let s = Setup::STANDARD;

        let lay = |tribe: usize| {
            let mut p = PlayerTech::new(&tree);
            p.tribe = tribe;
            p.power = Some(tribe);
            tree.start(&s, &Tuning::RON, &mut p);
            p
        };
        // Tribe 11 owns the generic one and not the variant…
        let british = lay(11);
        assert!(british.tech[generic] && british.tech_at_start[generic]);
        assert!(!british.tech[variant]);
        // …and tribe 0 the other way round. Both own Hoplites, which no
        // nation substitutes — so the difference is the mask and not the
        // seeding.
        let aztec = lay(0);
        assert!(!aztec.tech[generic]);
        assert!(aztec.tech[variant] && aztec.tech_at_start[variant]);
        assert!(british.tech[f.hoplites] && aztec.tech[f.hoplites]);

        // And the consequence the AI reads: what a leader does not own is
        // `RESEARCHABLE` rather than `AVAILABLE`, which is what put two
        // Barracks types in front of `Leader::upgrade_units` on run53's
        // frame 6779.
        assert_eq!(tree.type_avail(&s, &british, generic, true), AVAILABLE);
        assert_eq!(tree.type_avail(&s, &aztec, generic, true), NOT_AVAILABLE);
        assert_eq!(tree.type_avail(&s, &british, variant, true), NOT_AVAILABLE);
    }

    #[test]
    fn a_medieval_start_owns_two_ages_and_the_age_only_techs() {
        let f = fixture();
        let s = Setup {
            starting_age: 2,
            ..Setup::STANDARD
        };
        let mut p = PlayerTech::new(&f.tree);
        f.tree.start(&s, &Tuning::RON, &mut p);
        assert!(p.tech[f.ages[0]] && p.tech[f.ages[1]] && !p.tech[f.ages[2]]);
        assert_eq!(p.ages, 2);
        // No library epoch, and nothing that needs one.
        assert_eq!(p.epochs, 0);
        assert!(!p.tech[f.herbal_lore]);
        assert!(!p.tech[f.fortification]);
        // Governments are skipped at start even when their age is owned.
        assert!(!p.tech[f.govs[0][0]]);
        // Units whose prerequisites are the owned ages are owned outright.
        assert!(p.tech[f.phalanx]);
        assert!(p.tech[f.pikemen]);
        // And the predecessors are obsolete.
        assert!(p.obs[f.hoplites] && p.obs[f.phalanx]);
    }

    #[test]
    fn classical_age_cascades_the_free_catapult_and_leaves_phalanx_to_research() {
        let f = fixture();
        let s = Setup::STANDARD;
        let mut p = fresh(&f);
        age_up(&f, &mut p, 0);
        // Catapult needs Classical and Military 1 (gained as part of the
        // quota, so already owned); it is free: the age's cascade took it.
        assert!(p.tech[f.catapult]);
        // Phalanx is not free: it became a research job.
        assert!(!p.tech[f.phalanx]);
        assert_eq!(f.tree.type_avail(&s, &p, f.phalanx, true), RESEARCHABLE);
        assert_eq!(f.tree.type_avail(&s, &p, f.hoplites, true), AVAILABLE);
        // Pikemen: Medieval not owned.
        assert_eq!(f.tree.type_avail(&s, &p, f.pikemen, true), NOT_AVAILABLE);
    }

    #[test]
    fn the_jump_rule_skips_the_intermediate_upgrade_and_obsoletes_it() {
        let f = fixture();
        let s = Setup::STANDARD;
        let mut p = fresh(&f);
        age_up(&f, &mut p, 1);
        // Medieval owned: Pikemen's prerequisites are met, so Phalanx can no
        // longer be researched — the player jumps straight to Pikemen.
        assert_eq!(f.tree.type_eligible(&s, &p, f.phalanx, true), 0);
        assert_eq!(f.tree.type_avail(&s, &p, f.pikemen, true), RESEARCHABLE);
        // Gaining Pikemen owns and obsoletes everything below it.
        let events = f.tree.gain_tech(&s, &mut p, f.pikemen, 100);
        assert!(events.contains(&Gained::UnitUpgrade { to: f.pikemen }));
        assert!(p.tech[f.phalanx] && p.obs[f.phalanx]);
        assert!(p.tech[f.hoplites] && p.obs[f.hoplites]);
        assert_eq!(f.tree.type_avail(&s, &p, f.phalanx, true), NOT_AVAILABLE);
        assert_eq!(f.tree.current_upgrade(&s, &p, f.hoplites), f.pikemen);
        // Pikemen itself is owned and not obsolete.
        assert_eq!(f.tree.type_avail(&s, &p, f.pikemen, true), AVAILABLE);
    }

    #[test]
    fn a_non_jumpable_upgrade_needs_its_predecessor_available() {
        let f = fixture();
        let s = Setup::STANDARD;
        let mut p = fresh(&f);
        age_up(&f, &mut p, 1);
        // Trebuchet (no `j`) checks its predecessor through type_avail(…, 0):
        // the Catapult is owned (free with Classical), so it passes.
        assert!(p.tech[f.catapult]);
        assert_eq!(f.tree.type_avail(&s, &p, f.trebuchet, true), RESEARCHABLE);
        // Take the Catapult away: the predecessor is merely researchable
        // (2 < 3), and it is for this nation, so the Trebuchet is refused.
        p.tech[f.catapult] = false;
        assert_eq!(f.tree.type_avail(&s, &p, f.trebuchet, true), NOT_AVAILABLE);
    }

    #[test]
    fn governments_pair_and_tier() {
        let f = fixture();
        let s = Setup::STANDARD;
        let mut p = fresh(&f);
        age_up(&f, &mut p, 1);
        let [despotism, republic] = f.govs[0];
        let [monarchy, democracy] = f.govs[1];
        assert_eq!(f.tree.type_avail(&s, &p, despotism, true), AVAILABLE);
        assert_eq!(f.tree.type_avail(&s, &p, republic, true), AVAILABLE);
        // Tier two needs a tier-one government.
        assert_eq!(f.tree.type_avail(&s, &p, monarchy, true), NOT_AVAILABLE);
        // Queueing Republic shuts Despotism.
        p.queued[republic] = 1;
        assert_eq!(f.tree.type_avail(&s, &p, despotism, true), NOT_AVAILABLE);
        p.queued[republic] = 0;
        f.tree.gain_tech(&s, &mut p, republic, 5);
        assert_eq!(f.tree.type_avail(&s, &p, despotism, true), NOT_AVAILABLE);
        // Either column of the tier below opens both of the tier above.
        assert_eq!(f.tree.type_avail(&s, &p, monarchy, true), AVAILABLE);
        assert_eq!(f.tree.type_avail(&s, &p, democracy, true), AVAILABLE);
        assert_eq!(f.tree.govs_taken(&s, &p), 1);
        // `discovered` counts every non-age, non-epoch gain: Republic, and
        // the free Catapult that Classical cascaded.
        assert_eq!(p.discovered, 2);
    }

    #[test]
    fn the_finals_need_five_named_techs_and_respect_early_info_age() {
        let f = fixture();
        let s = Setup::STANDARD;
        let mut p = fresh(&f);
        age_up(&f, &mut p, 6);
        // 26 epochs are owned; the finals need the top of every line.
        assert!(!f.tree.has_preq(&s, &p, f.finals[0]));
        for line in Line::ALL {
            for level in 0..LEVELS {
                f.tree
                    .gain_tech(&s, &mut p, f.epochs[line.index()][level], 1);
            }
        }
        assert_eq!(p.epochs, EPOCHS);
        assert!(f.tree.has_preq(&s, &p, f.finals[0]));
        assert_eq!(f.tree.type_avail(&s, &p, f.finals[0], true), AVAILABLE);
        let early = Setup {
            no_finals: true,
            ..Setup::STANDARD
        };
        assert_eq!(
            f.tree.type_avail(&early, &p, f.finals[0], true),
            NOT_AVAILABLE
        );
        let info_dm = Setup {
            rules: Rules::InfoDeathmatch,
            ..Setup::STANDARD
        };
        assert_eq!(
            f.tree.type_avail(&info_dm, &p, f.finals[0], true),
            NOT_AVAILABLE
        );
    }

    #[test]
    fn the_tech_race_is_won_on_the_ending_age() {
        let f = fixture();
        let race = Setup {
            tech_race: true,
            ..Setup::STANDARD
        };
        let mut p = fresh(&f);
        age_up(&f, &mut p, 5);
        for line in Line::ALL {
            for level in 0..LEVELS {
                f.tree
                    .gain_tech(&race, &mut p, f.epochs[line.index()][level], 1);
            }
        }
        // Every library tech is owned and nothing is won yet: the win is the
        // ending age, under that victory setting only.
        assert_eq!(p.epochs, EPOCHS);
        assert!(f.tree.has_preq(&race, &p, f.ages[6]));
        let plain = f
            .tree
            .gain_tech(&Setup::STANDARD, &mut p.clone(), f.ages[6], 9);
        assert!(!plain.contains(&Gained::TechRaceWon));
        let won = f.tree.gain_tech(&race, &mut p, f.ages[6], 9);
        assert!(won.contains(&Gained::TechRaceWon));
        assert_eq!(p.ages, 7);
        assert_eq!(p.epoch, [7, 7, 7, 7]);
    }

    #[test]
    fn finalize_derives_what_the_loaders_derive() {
        let f = fixture();
        let mut t = f.tree.clone();
        let mil = f.epochs[Line::Military.index()];
        // A combat unit naming only Medieval gets Military 2 as well; a
        // civilian (Citizen: not combat) does not.
        let knight = t.add(
            TypeDef::unit("Knight", J)
                .at(f.barracks)
                .from(f.hoplites)
                .needs(0, f.ages[1]),
        );
        let stable = t.add(TypeDef::building("Stable"));
        let keep = t.add(TypeDef::building("Keep").from(stable));
        t.types[keep].kind = Kind::Building {
            auto: true,
            wonder: false,
        };
        t.finalize();
        assert_eq!(
            t.types[knight].preq,
            [Preq::Of(f.ages[1]), Preq::Of(mil[1]), Preq::None]
        );
        assert_eq!(t.types[f.citizen].preq[1], Preq::None);
        assert_eq!(t.types[f.phalanx].preq[1], Preq::Of(mil[0]));
        // The predecessor's `upgrade` is the successor (the last one written).
        assert_eq!(t.types[f.hoplites].upgrade, Some(knight));
        assert_eq!(t.types[f.phalanx].upgrade, Some(f.pikemen));
        // Buildings: `to` and, for an auto-upgrade, `upgrade`; `where` is the
        // predecessor unless the building is the base of its line.
        assert_eq!(t.types[stable].to, Some(keep));
        assert_eq!(t.types[stable].upgrade, Some(keep));
        assert_eq!(t.types[keep].where_, Some(stable));
        assert_eq!(t.types[stable].where_, None);
        // And so a Keep queues at a Stable.
        assert!(t.queue_here(stable, keep));
        // Which means the implicit requirement bites: with Medieval owned but
        // Military 2 not, the Knight is not researchable.
        let s = Setup::STANDARD;
        let mut p = PlayerTech::new(&t);
        t.start(&s, &Tuning::RON, &mut p);
        let fx = Fixture {
            tree: t.clone(),
            ..f
        };
        age_up(&fx, &mut p, 1);
        assert!(p.tech[mil[0]] && p.tech[mil[1]], "the quota owns both");
        t.lose_tech(&s, &mut p, mil[1]);
        assert!(!t.has_preq(&s, &p, knight));
        t.gain_tech(&s, &mut p, mil[1], 3);
        assert!(t.has_preq(&s, &p, knight));
    }

    #[test]
    fn losing_touches_only_unit_and_tech_bits_and_gaining_a_government_sets_it() {
        let f = fixture();
        let s = Setup::STANDARD;
        let mut p = fresh(&f);
        age_up(&f, &mut p, 0);
        let [despotism, _] = f.govs[0];
        assert_eq!(p.gov, None);
        f.tree.gain_tech(&s, &mut p, despotism, 2);
        assert_eq!(p.gov, Some(despotism));
        // A building's bit survives lose_tech; a unit's does not.
        assert!(p.tech[f.barracks]);
        f.tree.lose_tech(&s, &mut p, f.barracks);
        assert!(p.tech[f.barracks]);
        assert!(p.tech[f.catapult]);
        f.tree.lose_tech(&s, &mut p, f.catapult);
        assert!(!p.tech[f.catapult]);
    }

    #[test]
    fn the_ending_age_caps_ages_units_and_techs() {
        let f = fixture();
        let mut p = fresh(&f);
        age_up(&f, &mut p, 2);
        // Ending at Gunpowder (3): Enlightenment is refused, and so is a unit
        // whose prerequisite is it; Gunpowder itself was allowed.
        let s = Setup {
            ending: 3,
            ..Setup::STANDARD
        };
        assert_eq!(f.tree.type_eligible(&s, &p, f.ages[3], true), 0);
        assert_eq!(f.tree.type_eligible(&s, &p, f.ages[2], true), 4);
        assert_eq!(f.tree.type_eligible(&s, &p, f.rifleman, true), 0);
        // A building tech within the end is fine.
        assert_eq!(f.tree.type_eligible(&s, &p, f.medicine, true), 4);
    }

    #[test]
    fn machine_guns_need_a_rifle_line_unit() {
        let f = fixture();
        let s = Setup::STANDARD;
        let mut p = fresh(&f);
        age_up(&f, &mut p, 4);
        // The Machine Gun is free with Industrial, so the cascade owned it —
        // but type_avail still asks for a Rifleman.
        assert!(p.tech[f.machinegun]);
        assert_eq!(f.tree.type_avail(&s, &p, f.machinegun, true), NOT_AVAILABLE);
        f.tree.gain_tech(&s, &mut p, f.rifleman, 9);
        assert_eq!(f.tree.type_avail(&s, &p, f.machinegun, true), AVAILABLE);
    }

    #[test]
    fn queue_here_reads_lineage_and_the_where_column() {
        let f = fixture();
        assert!(f.tree.queue_here(f.barracks, f.phalanx));
        assert!(!f.tree.queue_here(f.library, f.phalanx));
        assert!(f.tree.queue_here(f.town, f.citizen));
        // A unit with a `disable` first prerequisite can be queued nowhere.
        let mut t = f.tree.clone();
        t.types[f.phalanx].preq[0] = Preq::Disabled;
        assert!(!t.queue_here(f.barracks, f.phalanx));
    }

    #[test]
    fn a_disabled_prerequisite_is_refused_and_a_missing_one_is_not() {
        let f = fixture();
        let s = Setup::STANDARD;
        let mut p = fresh(&f);
        age_up(&f, &mut p, 0);
        let mut t = f.tree.clone();
        t.types[f.phalanx].preq[1] = Preq::Disabled;
        assert_eq!(t.type_eligible(&s, &p, f.phalanx, true), 0);
        assert!(!t.has_preq(&s, &p, f.phalanx));
        t.types[f.phalanx].preq[1] = Preq::None;
        assert!(t.has_preq(&s, &p, f.phalanx));
    }

    #[test]
    fn losing_an_age_takes_its_dependents_and_recounts() {
        let f = fixture();
        let s = Setup::STANDARD;
        let mut p = fresh(&f);
        age_up(&f, &mut p, 2);
        f.tree.gain_tech(&s, &mut p, f.fortification, 3);
        f.tree.gain_tech(&s, &mut p, f.pikemen, 3);
        assert_eq!(p.ages, 3);
        // Losing Medieval loses Gunpowder (an age at or above it) and
        // everything whose closure names it; the epochs do not name ages.
        f.tree.lose_tech(&s, &mut p, f.ages[1]);
        assert!(p.tech[f.ages[0]] && !p.tech[f.ages[1]] && !p.tech[f.ages[2]]);
        assert_eq!(p.ages, 1);
        assert!(p.tech[f.fortification], "needs Military 2 only");
        // set_age drops what no longer follows: Pikemen needs Medieval.
        f.tree.set_age(&s, &mut p, 1, 4);
        assert!(!p.tech[f.pikemen]);
        assert!(p.tech[f.phalanx], "Classical still owned");
        assert_eq!(p.ages, 1);
    }

    #[test]
    fn set_epoch_walks_a_line_and_compute_epoch_agrees() {
        let f = fixture();
        let s = Setup::STANDARD;
        let mut p = fresh(&f);
        f.tree.set_epoch(&s, &mut p, Line::Commerce, 4, 2);
        assert_eq!(p.epoch[Line::Commerce.index()], 4);
        assert_eq!(f.tree.compute_epoch(&s, &p, Line::Commerce), 4);
        assert_eq!(p.epochs, 4);
        f.tree.set_epoch(&s, &mut p, Line::Commerce, 2, 3);
        assert_eq!(p.epoch[Line::Commerce.index()], 2);
        assert_eq!(p.epochs, 2);
    }

    #[test]
    fn nation_free_rules_cascade_when_gated_on() {
        let f = fixture();
        let s = Setup::STANDARD;
        let mut t = f.tree.clone();
        t.free_rules.push(FreeRule {
            gate: Gate::Power(14),
            enabled: 1,
            candidates: vec![f.herbal_lore, f.medicine],
            shape: Shape::PreqMatch,
        });
        let mut p = PlayerTech::new(&t);
        p.power = Some(14);
        t.start(&s, &Tuning::RON, &mut p);
        // Science 1 is Herbal Lore's prerequisite: gaining it hands Herbal
        // Lore over; Medicine waits for Science 3.
        t.gain_tech(&s, &mut p, f.epochs[Line::Science.index()][0], 1);
        assert!(p.tech[f.herbal_lore]);
        assert!(!p.tech[f.medicine]);
        // Without the power, nothing.
        let mut q = PlayerTech::new(&t);
        t.start(&s, &Tuning::RON, &mut q);
        t.gain_tech(&s, &mut q, f.epochs[Line::Science.index()][0], 1);
        assert!(!q.tech[f.herbal_lore]);
        // "No Nation Powers" shuts it too.
        let off = Setup {
            no_nation_powers: true,
            ..Setup::STANDARD
        };
        let mut r = PlayerTech::new(&t);
        r.power = Some(14);
        t.start(&off, &Tuning::RON, &mut r);
        t.gain_tech(&off, &mut r, f.epochs[Line::Science.index()][0], 1);
        assert!(!r.tech[f.herbal_lore]);
    }

    #[test]
    fn a_free_starting_epoch_is_owned_before_frame_zero() {
        let f = fixture();
        let s = Setup::STANDARD;
        let mut p = PlayerTech::new(&f.tree);
        p.power = Some(6); // Romans: ROMAN_FREE_MILITARY
        f.tree.start(&s, &Tuning::RON, &mut p);
        assert_eq!(p.epoch[Line::Military.index()], 1);
        assert_eq!(p.epochs, 1);
        // The Catapult needs Classical too, so it is not free yet.
        assert!(!p.tech[f.catapult]);
    }

    #[test]
    fn the_lobby_remap_rescales_a_units_military_requirement() {
        let f = fixture();
        let mut t = f.tree.clone();
        // A unit asking for Military 4 in slot 1, in a game that starts at
        // Gunpowder and ends at Industrial: span 3 ages, 28/3 = 9 per age.
        let mil = f.epochs[Line::Military.index()];
        let u = t.add(
            TypeDef::unit("Dragoons", J)
                .needs(0, f.ages[2])
                .needs(1, mil[3]),
        );
        let s = Setup {
            starting_age: 3,
            ending: 5,
            ..Setup::STANDARD
        };
        let p = PlayerTech::new(&t);
        // age 3: n = 9 * (3 - 3 + 1) + 3 = 12, 12/4 = 3, level 2.
        assert_eq!(t.get_preq(&s, Some(&p), u, 1), Preq::Of(mil[2]));
        // A requirement from before the start vanishes.
        let v = t.add(TypeDef::unit("Old", J).needs(1, mil[1]));
        assert_eq!(t.get_preq(&s, Some(&p), v, 1), Preq::None);
        // The full game leaves the column alone.
        assert_eq!(
            t.get_preq(&Setup::STANDARD, Some(&p), u, 1),
            Preq::Of(mil[3])
        );
    }

    #[test]
    fn barbarians_defenders_start_with_both_settings() {
        let f = fixture();
        let s = Setup {
            starting_age: 2,
            starting_age2: 2,
            rules: Rules::BarbariansAtTheGates,
            ..Setup::STANDARD
        };
        let mut d = PlayerTech::new(&f.tree);
        d.team = 0;
        assert_eq!(f.tree.starting_age(&s, &d), 4);
        let mut a = PlayerTech::new(&f.tree);
        a.team = 1;
        assert_eq!(f.tree.starting_age(&s, &a), 2);
        // The defenders' epochs are pre-owned in this mode alone.
        f.tree.start(&s, &Tuning::RON, &mut d);
        assert!(d.epochs > 0);
        f.tree.start(&s, &Tuning::RON, &mut a);
        assert_eq!(a.epochs, 0);
    }
}
