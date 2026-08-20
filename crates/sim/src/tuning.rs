//! The tuned numbers the simulation depends on.
//!
//! These are inputs, not constants of the universe. The simulation takes a
//! [`Tuning`] and never reaches for a global, so a scenario, a mod, or a test
//! can hand it a different one.
//!
//! [`Tuning::RON`] holds the values Rise of Nations ships. They are here for
//! the same reason they are quoted in `docs/ATTRITION.md`: a formula nobody can
//! evaluate is not a specification. Nothing is copied out of the user's install
//! — `cargo run -p rondata -- <install>` re-reads the install's own
//! `rules.xml` and fails if any of these has drifted from it. See
//! [`Tuning::slots`].

/// One named entry of a [`Tuning`], for cross-checking against a game install.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    /// A `<NAME value="..."/>` constant.
    Value(i32),
    /// A `<NAME entry0="..." entry1="..."/>` array.
    Entries(&'static [i32]),
    /// A constant the engine loads as 8.8 fixed point, held here already
    /// scaled: `3/2` in the file is 384 here, and a plain `10` is 2560.
    ///
    /// The scale is not something the file states. It is fixed by how the
    /// original consumes the value — a multiply followed by a shift right by
    /// eight — so the check has to reconstruct it rather than compare digits.
    /// Nor is it a property of the *syntax*: `PEASANT_RATE` is written as the
    /// plain integer `10 resources` and still arrives at 2560, because
    /// `Constants::init` happens to read that one with `get_fraction(name,
    /// 0x100)` while reading its neighbours plain. Which constants are scaled
    /// is a fact about the loader, one constant at a time.
    Ratio256(i32),
    /// An `entryN` array whose elements are each loaded as 8.8 fixed point.
    Entries256(&'static [i32]),
    /// A constant the engine loads scaled by a *hundred*, held here already
    /// scaled: `6/5` in the file is 120 here and `1/1` is 100.
    ///
    /// The same rule as [`Slot::Ratio256`] with a different denominator, and
    /// the reason that rule is stated as "one constant at a time" rather than
    /// "fixed point or not". `Constants::init` reads the production
    /// accelerators and the unit rate pair with `get_fraction(name, 100)`
    /// while reading `RESEARCH_TICK_PREMIUM`, three lines of the same file
    /// away, with `get_fraction(name, 0x100)`. Nothing in the written value
    /// distinguishes them; only the loader does. See `docs/PRODUCTION.md`.
    Ratio100(i32),
    /// A constant the engine loads through `get_fraction(name, 0xc0)` — a
    /// length in position units, 192 to the tile — held here already scaled:
    /// `1/2 tile` in the file is 96 here. Combat's `TARGET_RADIUS` is the one
    /// so far; see `docs/COMBAT.md`.
    Ratio192(i32),
}

/// Everything the attrition, territory and supply passes read.
///
/// Field names match the shipped constant names, lowercased, so that a reader
/// with `rules.xml` open can follow along. Where the original's own struct
/// layout let a lookup run off the end of one array into the next — the
/// engine's `ATTRITION_UPGRADE[n + 3]` reaching into `ATTRITION_IMPROVED`, and
/// `FORT_UPGRADE_TERR[n + 3]` into `TEMPLE_UPGRADE_TERR` — the arrays are
/// separate here and the indices are written the way the designers meant them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tuning {
    // ---- the rate ----
    /// Baseline tick period in frames, before resistance and strength.
    pub attrition: i32,
    /// Period applied for a border violation while at peace.
    pub peace_attrition: i32,
    /// Period applied by an assassin.
    pub assassin_attrition: i32,
    /// Indexed by how many steps of the attrition tech chain the owner holds.
    pub attrition_improved: [i32; 4],
    /// Percentage resistance from Foraging tiers 1, 2, 3. The fourth is unused.
    pub attrition_upgrade: [i32; 4],
    /// Percentage reduction for siege units.
    pub siege_attrition: i32,
    /// Percentage increase for militia.
    pub militia_attrition: i32,
    /// Percentage strength increase per age the owner is ahead of the victim.
    pub attrition_aged_up: i32,
    /// Percentage strength increase from the Colosseum.
    pub colosseum_attrition: i32,
    /// Percentage strength increase for the Russians.
    pub russian_attrition: i32,
    /// Percentage strength increase from a Conquer the World bonus.
    pub ctw_attrition: i32,
    /// Percentage strength increase from the Kremlin.
    pub kremlin_attrition: i32,
    /// Percentage resistance from the Statue of Liberty. 100 means immune.
    pub liberty_attrition: i32,
    /// Percentage resistance for the Mongols.
    pub mongol_attrition: i32,
    /// Percentage resistance from titanium.
    pub titanium_attrition: i32,

    // ---- the territory ----
    /// Cost cap, in tiles, before the `<< 8` fixed-point scale is applied.
    pub territory_base: i32,
    /// Numerator of the designers' radius expression.
    pub territory_num: i32,
    /// Denominator of the designers' radius expression.
    pub territory_den: i32,
    /// Furthest a city may reach, in tiles, before bonuses.
    pub territory_limit_base: i32,
    /// Added to that limit per civic tech level.
    pub territory_limit_civic: i32,
    /// Added to that limit per city level, temple level, and fort level.
    pub territory_limit_city: i32,
    /// Weight applied to a city's border bonuses.
    pub city_territory_multiplier: i32,
    /// Weight applied to a fort's border bonuses.
    pub fort_territory_multiplier: i32,
    /// Border bonus for a capital, in place of its city-level bonus.
    pub capital_territory_bonus: i32,
    /// Border bonus by city level: city, town, metropolis.
    pub city_upgrade_terr: [i32; 3],
    /// Border bonus by fort border tech level.
    pub fort_upgrade_terr: [i32; 4],
    /// Border bonus by temple border tech level. The fifth is unused.
    pub temple_upgrade_terr: [i32; 5],
    /// Border bonus by civic tech level.
    pub civic_upgrade_terr: [i32; 8],
    /// Flat border bonus from the Colosseum.
    pub colosseum_territory_bonus: i32,
    /// Border bonus the Colosseum adds to forts specifically.
    pub colosseum_fort_borders: i32,
    /// Flat border bonus from the Eiffel Tower.
    pub eiffel_tower_territory_bonus: i32,
    /// Flat border bonus from a gem resource.
    pub gems_territory_bonus: i32,
    /// Border bonus Roman forts carry.
    pub roman_fort_borders: i32,
    /// Flat border bonus for the Russians.
    pub russian_borders: i32,
    /// Added to the Russian bonus per age.
    pub russian_borders_per_age: i32,
    /// Percentage increase to a temple's border bonus from Tikal.
    pub tikal_temple_borders: i32,
    /// Flat border bonus the Red Fort adds to the fort it stands in.
    pub red_fort_borders: i32,

    // ---- supply ----
    /// A supply source's reach, in tiles, before upgrades.
    pub supply_radius: i32,
    /// Added to that reach per step of the supply upgrade chain.
    pub supply_radius_upgrade: i32,
    /// Added to a supply or general radius by the Terra Cotta Army. Ships as
    /// zero: the wonder is wired in and contributes nothing.
    pub terra_cotta_range: i32,
    /// Base of a general's aura radius, in tiles.
    pub general_radius: i32,
    /// Parmenio's scale on that radius, as 8.8 fixed point. The shipped file
    /// writes `3/2`; the original multiplies and shifts right by eight, which
    /// is what fixes the scale.
    pub parmenio_radius_adjust: i32,
    /// Wellington's percentage scale on it.
    pub wellington_radius: i32,
    /// Kutosov's percentage scale on it.
    pub kutosov_radius: i32,
    /// Added to it for a military patriot.
    pub mil_patriot_radius_bonus: i32,
    /// Added to it for an economic patriot.
    pub econ_patriot_radius_bonus: i32,
    /// Period, in frames, at which supply repairs damage. Zero means never,
    /// which is what ships.
    pub supply_heal_rate: i32,
    /// Period granted instead by the supply-heal nation bonus.
    pub french_supply_heal_rate: i32,
    /// Period granted instead by Versailles.
    pub versailles_supply_heal_rate: i32,
    /// Non-zero if a siege unit under attack reloads as though out of supply.
    pub artillery_under_attack_fires_slowly: i32,

    // ---- movement ----
    /// Master scale on every unit's turn rate, as 8.8 fixed point. The file
    /// writes `1/1 rate` and `Constants::init` reads it with
    /// `get_fraction(name, 0x100)`, so it is 256 here — which is what makes
    /// `(type.turn_speed >> 8) * UNIT_TURN_SPEED` the type's angle with its
    /// low byte cleared, and `UNIT_TURN_SPEED * 0xb60b` one degree.
    pub unit_turn_speed: i32,
    /// Multiplier on the turn rate of a packed unit.
    pub unit_pack_turn_bonus: i32,

    // ---- economy ----
    /// Frames a gather rate is quoted over. 450 is thirty seconds.
    pub gather_rate: i32,
    /// What each player starts with.
    pub starting_goods: [i32; 6],
    /// A free trickle per resource, before anything is built. Ships as zero.
    pub basic_gather: [i32; 6],
    /// What a city is worth per period before anybody works in it.
    pub city_gather: [i32; 6],
    /// One gatherer's rate, as 8.8 fixed point. The file writes `10 resources`
    /// and the engine loads 2560; see [`Slot::Ratio256`].
    pub peasant_rate: i32,
    /// One oil well gatherer's rate, as 8.8 fixed point.
    pub oil_rate: i32,
    /// One scholar's rate by university level, as 8.8 fixed point.
    pub scholar_rate: [i32; 6],
    /// Percentage bonus to a city's food by granary level.
    ///
    /// The original reaches this array, and the two below it, by indexing
    /// `level + 4` off the *preceding* array — `FISHERMEN_BONUS` for this one.
    /// Written the way the designers meant it that is `[level - 1]`, and the
    /// levels are one-based. Same shape as `ATTRITION_UPGRADE` reaching into
    /// `ATTRITION_IMPROVED`; see the note on this struct.
    pub granary_bonus: [i32; 5],
    /// Percentage bonus to a city's timber by lumber mill level.
    pub lumbermill_bonus: [i32; 5],
    /// Percentage bonus to a city's metal by smelter level.
    pub smelter_bonus: [i32; 5],
    /// Percentage bonus to oil per refinery, applied at the player level
    /// rather than per city — the only enhancer written that way.
    pub refinery_bonus: i32,
    /// Wealth per city per period, unconditionally. Ships as zero.
    pub village_taxes: i32,
    /// Wealth per building in a city. Ships as zero.
    pub building_taxes: i32,
    /// Wealth for a city with a market. The only non-zero term in the line.
    pub market_taxes: i32,
    /// Wealth for a city with a temple. Ships as zero.
    pub temple_taxes: i32,
    /// Knowledge per city, unconditionally. Ships as zero.
    pub village_literacy: i32,
    /// Knowledge for a city with a university.
    pub university_literacy: i32,
    /// Knowledge for a city with a library. Ships as zero.
    pub library_literacy: i32,
    /// Wealth per period for the whole map, shared out by territory, indexed
    /// by taxation level.
    pub territory_taxes: [i32; 5],
    /// The ceiling on every capped resource's rate, by commerce level.
    pub commerce_cap: [i32; 8],

    // ---- costs ----
    /// What a unit's written `COST` is multiplied by.
    pub unit_cost_factor: i32,
    /// The same, for buildings.
    pub build_cost_factor: i32,
    /// The same, for techs and for anything that is not a unit, building or
    /// spell.
    pub tech_cost_factor: i32,
    /// The same, for spells.
    pub spell_cost_factor: i32,
    /// What a *building's* `SUPPORT` ramp is multiplied by. Ships as one, and
    /// there is no unit equivalent: a unit's ramp is not scaled at all.
    pub build_support_factor: i32,
    /// Ceiling on a scholar's ramp, as a percentage of its base price.
    pub unit_scholar_ramp_max: i32,
    /// Ceiling on a citizen's or merchant's ramp.
    pub unit_worker_ramp_max: i32,
    /// Ceiling on the ramp of a civilian that is neither. Generals, spies,
    /// supply wagons, caravans.
    pub unit_other_civilian_ramp_max: i32,
    /// Ceiling on a fighting unit's ramp. The lowest of the four, which is why
    /// an army's price plateaus.
    pub unit_military_ramp_max: i32,
    /// Percentage the maize rare resource takes off a ramp term.
    pub maize_ramping_bonus: i32,
    /// Largest per-resource price difference a refit may be charged for.
    pub unit_refit_max_cost: i32,
    /// Global multiplier on the price of researching an upgrade, as 8.8 fixed
    /// point. Ships as `1/1` and is the identity; the per-unit
    /// `RESEARCH_PREMIUM_COST` is where the doubling actually lives.
    pub research_premium: i32,
    /// Percentage a final tech's price rises per final tech already held.
    pub ramp_final: i32,
    /// Percentage off a tech per age the player is behind the leader.
    pub tech_age_behind_discount: i32,
    /// The same, for the knowledge component. Twice the size.
    pub tech_age_behind_knowledge_discount: i32,
    /// Percentage off a tech per library colour the player is behind in.
    pub tech_color_behind_discount: i32,
    /// The same, for the knowledge component.
    pub tech_color_behind_knowledge_discount: i32,
    /// Percentage off a unit per Military library level the player holds above
    /// the unit's own `MILITARY_LEVEL` — how obsolete units get cheap. The
    /// player's side of the comparison is `epoch[0]`, the Military tech line,
    /// and not the age.
    pub military_unit_discount: i32,
    /// The same, when researching the upgrade rather than building the unit.
    pub military_upgrade_discount: i32,

    // ---- production ----
    /// Hundredths of a frame a training job advances per call. Ships as `1/1`,
    /// loads as 100, so one call is one frame. A debug knob, and the designers
    /// say so in the file.
    pub accel_train: i32,
    /// The same, for putting a building up.
    pub accel_construct: i32,
    /// The same, for research — and for the first one of a unit type, which is
    /// a research job.
    pub accel_research: i32,
    /// Percentage scale on every unit's base build time, before the ramp.
    /// Ships as `6/5`, so units take twenty percent longer than `JOB_TIME`.
    pub unit_rate_base: i32,
    /// Percentage scale on the per-unit build-time ramp term. Ships as `3/4`.
    pub unit_rate_progression: i32,
    /// Global multiplier on research *time*, as 8.8 fixed point, and the twin
    /// of `research_premium` on the cost side. Ships as `1/1` and is the
    /// identity; the per-unit `RESEARCH_PREMIUM_TIME` is where the doubling
    /// lives.
    pub research_tick_premium: i32,
    /// Percentage a queued item's price falls per science level, refunded in
    /// place while it waits.
    pub tech_science_discount: i32,
    /// Percentage a research job's time falls per science level above the
    /// tech's own.
    pub tech_science_speedup: i32,

    // ---- population ----
    /// The population cap by age, before the lobby's ceiling.
    pub pop_cap: [i32; 8],
    /// What one city adds to the cap. Ships as zero, so cities add nothing.
    pub village_pop: i32,
    /// Flat addition from the Colossus.
    pub colossus_pop_cap: i32,
    /// Percentage the Bantu add to their cap.
    pub bantu_pop_cap: i32,
    /// Percentage the Bantu add to the ceiling their cap is clamped by.
    pub bantu_final_pop_cap: i32,
    /// Percentage the peacock rare resource adds, after the clamp.
    pub peacocks_pop: i32,

    // ---- the tech tree ----
    // The nation and wonder powers the prerequisite predicates and the free-tech
    // cascades are gated on. Each is the `rules.xml` constant as loaded: non-zero
    // is on, and 2 means "and start with one" where the file says so. See
    // `docs/TECH.md`.
    /// Greeks: Knowledge without the Classical age.
    pub greek_knowledge_early: i32,
    /// Greeks: the University without its age (2: and start with one).
    pub greek_university_early: i32,
    /// Romans: towers and forts without their age, and the fort line without its non-age prerequisites.
    pub roman_fort_early: i32,
    /// Egyptians: the Granary early (2: and start with one).
    pub egyptian_granary_early: i32,
    /// French: the Lumber Mill early (2: and start with one).
    pub french_lumbermill_early: i32,
    /// Germans: Metal, the Mine and the Smelter without the Classical age.
    pub german_metal_early: i32,
    /// Germans: the Granary, Lumber Mill and Smelter early.
    pub german_buildings_early: i32,
    /// Koreans: the Temple early, and its techs free (2: and start with one).
    pub korean_temple_upgrades: i32,
    /// Egyptians: a wonder one age before its prerequisite.
    pub egyptian_wonders_early: i32,
    /// Germans: the agriculture, carpentry and metal lines one Science level early.
    pub german_industry_early: i32,
    /// Iroquois: governments one age early. Ships off.
    pub iroquois_govs_early: i32,
    /// Greeks: the starting knowledge stock arrives with the Classical age.
    pub greek_delay_knowledge: i32,
    /// Chinese: Herbal Lore and Medicine free with their prerequisite.
    pub chinese_herbal_lore: i32,
    /// Red Fort: the Fortification line free.
    pub red_fort_fortification: i32,
    /// Red Fort: the Tactics line free.
    pub red_fort_tactics: i32,
    /// Lakota: cavalry and armoured-car upgrades free.
    pub lakota_cav_upgrades: i32,
    /// Iroquois: scout upgrades free.
    pub iroquois_scout_upgrades: i32,
    /// Indians: elephant upgrades free.
    pub indians_elephant_upgrades: i32,
    /// Koreans: militia upgrades free.
    pub korean_militia_upgrades: i32,
    /// Koreans: the Taxation line free. Ships off.
    pub korean_temple_tax_upgrades: i32,
    /// Persians: the Taxation line free.
    pub persians_taxation: i32,
    /// Mongols: the Forage line free.
    pub mongol_free_forage: i32,
    /// Russians: the Allegiance line free.
    pub russian_attrition_upgrades: i32,
    /// Egyptians: the Agriculture line free.
    pub egyptian_granary_upgrades: i32,
    /// Romans: every Fort tech free.
    pub roman_fort_upgrades: i32,
    /// Hanging Gardens: the enhancer lines free. Ships off.
    pub hanging_gardens_upgrades: i32,
    /// French: the Carpentry line free.
    pub french_lumbermill_upgrades: i32,
    /// Chinese: the Literacy line free. Ships off.
    pub chinese_knowledge_upgrades: i32,
    /// Germans: the Metal line free. Ships off.
    pub german_metal_upgrades: i32,
    /// Germans: the industry lines free. Ships off.
    pub german_industry_upgrades: i32,
    /// Germans: heavy infantry upgrades free. Ships off.
    pub german_heavy_infantry: i32,
    /// British: archer upgrades free.
    pub british_archer_upgrades: i32,
    /// Spanish: scout upgrades free.
    pub spanish_scout_upgrades: i32,
    /// Turks: siege upgrades free.
    pub turk_free_siege_upgrades: i32,
    /// Statue of Liberty: unit upgrades free.
    pub liberty_free_upgrades: i32,
    /// Colosseum: the Fortification line free. Ships off.
    pub colosseum_fort_upgrades: i32,
    /// Tikal: the Religion line free. Ships off.
    pub tikal_temple_upgrades: i32,
    /// Dutch: start one Commerce level up.
    pub dutch_free_commerce: i32,
    /// Russians: start one Civic level up.
    pub russian_free_civic: i32,
    /// Aztecs: start one Military level up.
    pub aztec_free_military: i32,
    /// Romans: start one Military level up.
    pub roman_free_military: i32,
    /// Americans: start one Science level up. Ships off.
    pub americans_free_science: i32,
    /// Persians: start with Despotism. Ships off.
    pub persians_despotism: i32,

    // ---- combat (`docs/COMBAT.md`) ----
    /// Percent per level of flank; the rear is one level, the sides two.
    pub flank_bonus: i32,
    /// A mounted attacker's flank bonus as a fraction of the base — consumed
    /// `>> 8`, so the shipped `40` is 40/256 of it.
    pub cavalry_flank_bonus: i32,
    /// The same for a VEHICLE attacker.
    pub vehicle_flank_bonus: i32,
    /// 8.8: damage to light, modern and musket infantry on rocky ground.
    pub rocky_modifier: i32,
    /// The focus-fire window, in frames.
    pub overkill_frames: i32,
    /// 8.8: a second ranged squad's damage inside the window.
    pub overkill_damage: i32,
    /// 8.8: damage to an entrenched unit from its front, or from splash.
    pub entrenchment_modifier: i32,
    /// 8.8: damage to a unit standing in a river.
    pub river_modifier: i32,
    /// 8.8: damage to a city by its original owner.
    pub recapture_city_modifier: i32,
    /// Height units per increment of the height bonus.
    pub height_increment: i32,
    /// Percent per increment.
    pub height_bonus: i32,
    /// The combat table's age bonus, percent, by age difference.
    pub one_age_down: i32,
    pub two_ages_down: i32,
    pub three_ages_down: i32,
    pub four_ages_down: i32,
    pub five_ages_down: i32,
    /// The calibration of target sizes for projectile scatter, position units.
    pub target_radius: i32,
    /// Percent less damage to the Red Fort from aircraft.
    pub red_fort_air_defense: i32,
    /// The Japanese barracks bonus; negative means "per age".
    pub japanese_damage: i32,
    /// Percent bonus for stable units against siege and supply.
    pub russian_cossack_damage: i32,
    /// Added to the damage of units under Wellington against factory units.
    pub wellington_siege_attack: i32,
    /// 8.8: entrenchment under Antipater.
    pub antipater_entrench_bonus: i32,
    /// Whether the Supercollider is immune to aircraft.
    pub super_immune: i32,
    /// How far, in tiles, an idle unit looks for something to attack.
    pub unit_respond_range: i32,
    /// The same in the DEFENSIVE stance.
    pub unit_defensive_respond_range: i32,
    /// The same for a guard.
    pub unit_guard_respond_range: i32,
}

impl Tuning {
    /// The values Rise of Nations ships.
    ///
    /// Two of these are not read straight off a constant. `red_fort_borders`
    /// is a literal `4` in the original's territory pass with no constant
    /// behind it. `tikal_temple_borders` is the field the symbols resolve as
    /// `tikal_temple_hp`; by position in the shipped file the designers' name
    /// for that slot is `TIKAL_TEMPLE_BORDERS`, and since both are 50 the
    /// shipped behaviour is the same under either reading.
    pub const RON: Tuning = Tuning {
        attrition: 48,
        peace_attrition: 8,
        assassin_attrition: 8,
        attrition_improved: [1, 2, 4, 8],
        attrition_upgrade: [25, 50, 75, 100],
        siege_attrition: 50,
        militia_attrition: 300,
        attrition_aged_up: 25,
        colosseum_attrition: 50,
        russian_attrition: 100,
        ctw_attrition: 50,
        kremlin_attrition: 100,
        liberty_attrition: 100,
        mongol_attrition: 50,
        titanium_attrition: 50,

        territory_base: 24,
        territory_num: 11,
        territory_den: 5,
        territory_limit_base: 44,
        territory_limit_civic: 4,
        territory_limit_city: 4,
        city_territory_multiplier: 4,
        fort_territory_multiplier: 4,
        capital_territory_bonus: 6,
        city_upgrade_terr: [0, 3, 6],
        fort_upgrade_terr: [2, 4, 6, 9],
        temple_upgrade_terr: [2, 4, 6, 9, 12],
        civic_upgrade_terr: [0, 1, 2, 4, 6, 8, 11, 14],
        colosseum_territory_bonus: 3,
        colosseum_fort_borders: 0,
        eiffel_tower_territory_bonus: 6,
        gems_territory_bonus: 2,
        roman_fort_borders: 3,
        russian_borders: 0,
        russian_borders_per_age: 1,
        tikal_temple_borders: 50,
        red_fort_borders: 4,

        supply_radius: 14,
        supply_radius_upgrade: 2,
        terra_cotta_range: 0,
        general_radius: 6,
        parmenio_radius_adjust: 384,
        wellington_radius: 200,
        kutosov_radius: 300,
        mil_patriot_radius_bonus: 3,
        econ_patriot_radius_bonus: 1,
        supply_heal_rate: 0,
        french_supply_heal_rate: 20,
        versailles_supply_heal_rate: 20,
        artillery_under_attack_fires_slowly: 1,

        unit_turn_speed: 256,
        unit_pack_turn_bonus: 2,

        gather_rate: 450,
        starting_goods: [200, 200, 100, 100, 100, 100],
        basic_gather: [0, 0, 0, 0, 0, 0],
        city_gather: [10, 10, 0, 0, 0, 0],
        peasant_rate: 2560,
        oil_rate: 8960,
        scholar_rate: [1280, 1792, 2560, 3840, 5120, 6400],
        granary_bonus: [20, 50, 100, 200, 250],
        lumbermill_bonus: [20, 50, 100, 200, 250],
        smelter_bonus: [50, 100, 150, 200, 250],
        refinery_bonus: 33,
        village_taxes: 0,
        building_taxes: 0,
        market_taxes: 10,
        temple_taxes: 0,
        village_literacy: 0,
        university_literacy: 10,
        library_literacy: 0,
        territory_taxes: [0, 50, 100, 200, 300],
        commerce_cap: [70, 100, 150, 200, 260, 320, 400, 500],

        unit_cost_factor: 10,
        build_cost_factor: 10,
        tech_cost_factor: 10,
        spell_cost_factor: 10,
        build_support_factor: 1,
        unit_scholar_ramp_max: 2000,
        unit_worker_ramp_max: 500,
        unit_other_civilian_ramp_max: 200,
        unit_military_ramp_max: 125,
        maize_ramping_bonus: 50,
        unit_refit_max_cost: 40,
        research_premium: 256,
        ramp_final: 50,
        tech_age_behind_discount: 10,
        tech_age_behind_knowledge_discount: 20,
        tech_color_behind_discount: 10,
        tech_color_behind_knowledge_discount: 20,
        military_unit_discount: 5,
        military_upgrade_discount: 10,

        accel_train: 100,
        accel_construct: 100,
        accel_research: 100,
        unit_rate_base: 120,
        unit_rate_progression: 75,
        research_tick_premium: 256,
        tech_science_discount: 10,
        tech_science_speedup: 10,

        pop_cap: [25, 50, 75, 100, 125, 150, 175, 200],
        village_pop: 0,
        colossus_pop_cap: 50,
        bantu_pop_cap: 100,
        bantu_final_pop_cap: 25,
        peacocks_pop: 10,

        greek_knowledge_early: 1,
        greek_university_early: 2,
        roman_fort_early: 1,
        egyptian_granary_early: 2,
        french_lumbermill_early: 2,
        german_metal_early: 0,
        german_buildings_early: 1,
        korean_temple_upgrades: 2,
        egyptian_wonders_early: 1,
        german_industry_early: 1,
        iroquois_govs_early: 0,
        greek_delay_knowledge: 1,
        chinese_herbal_lore: 1,
        red_fort_fortification: 1,
        red_fort_tactics: 1,
        lakota_cav_upgrades: 1,
        iroquois_scout_upgrades: 1,
        indians_elephant_upgrades: 1,
        korean_militia_upgrades: 1,
        korean_temple_tax_upgrades: 0,
        persians_taxation: 1,
        mongol_free_forage: 1,
        russian_attrition_upgrades: 1,
        egyptian_granary_upgrades: 1,
        roman_fort_upgrades: 1,
        hanging_gardens_upgrades: 0,
        french_lumbermill_upgrades: 1,
        chinese_knowledge_upgrades: 0,
        german_metal_upgrades: 0,
        german_industry_upgrades: 0,
        german_heavy_infantry: 0,
        british_archer_upgrades: 1,
        spanish_scout_upgrades: 1,
        turk_free_siege_upgrades: 1,
        liberty_free_upgrades: 1,
        colosseum_fort_upgrades: 0,
        tikal_temple_upgrades: 0,
        dutch_free_commerce: 1,
        russian_free_civic: 1,
        aztec_free_military: 1,
        roman_free_military: 1,
        americans_free_science: 0,
        persians_despotism: 0,
        flank_bonus: 50,
        cavalry_flank_bonus: 40,
        vehicle_flank_bonus: 33,
        rocky_modifier: 170,
        overkill_frames: 30,
        overkill_damage: 85,
        entrenchment_modifier: 170,
        river_modifier: 512,
        recapture_city_modifier: 512,
        height_increment: 200,
        height_bonus: 10,
        one_age_down: 15,
        two_ages_down: 20,
        three_ages_down: 50,
        four_ages_down: 60,
        five_ages_down: 70,
        target_radius: 96,
        red_fort_air_defense: 33,
        japanese_damage: -5,
        russian_cossack_damage: 25,
        wellington_siege_attack: 1,
        antipater_entrench_bonus: 204,
        super_immune: 0,
        unit_respond_range: 12,
        unit_defensive_respond_range: 4,
        unit_guard_respond_range: 8,
    };

    /// Every value in [`Tuning::RON`] that comes from a named constant in
    /// `rules.xml`, paired with the name the shipped file gives it.
    ///
    /// This is what lets a tool re-derive [`Tuning::RON`] from a real install
    /// and report a drift, rather than us asserting numbers into the void. The
    /// two entries with no constant behind them are absent by design.
    pub const fn ron_slots() -> [(&'static str, Slot); 173] {
        const T: Tuning = Tuning::RON;
        [
            ("ATTRITION", Slot::Value(T.attrition)),
            ("PEACE_ATTRITION", Slot::Value(T.peace_attrition)),
            ("ASSASSIN_ATTRITION", Slot::Value(T.assassin_attrition)),
            ("SIEGE_ATTRITION", Slot::Value(T.siege_attrition)),
            ("MILITIA_ATTRITION", Slot::Value(T.militia_attrition)),
            ("ATTRITION_AGED_UP", Slot::Value(T.attrition_aged_up)),
            ("COLOSSEUM_ATTRITION", Slot::Value(T.colosseum_attrition)),
            ("RUSSIAN_ATTRITION", Slot::Value(T.russian_attrition)),
            ("CTW_ATTRITION", Slot::Value(T.ctw_attrition)),
            ("KREMLIN_ATTRITION", Slot::Value(T.kremlin_attrition)),
            ("LIBERTY_ATTRITION", Slot::Value(T.liberty_attrition)),
            ("MONGOL_ATTRITION", Slot::Value(T.mongol_attrition)),
            ("TITANIUM_ATTRITION", Slot::Value(T.titanium_attrition)),
            ("TERRITORY_BASE", Slot::Value(T.territory_base)),
            ("TERRITORY_NUM", Slot::Value(T.territory_num)),
            ("TERRITORY_DEN", Slot::Value(T.territory_den)),
            ("TERRITORY_LIMIT_BASE", Slot::Value(T.territory_limit_base)),
            (
                "TERRITORY_LIMIT_CIVIC",
                Slot::Value(T.territory_limit_civic),
            ),
            ("TERRITORY_LIMIT_CITY", Slot::Value(T.territory_limit_city)),
            (
                "CITY_TERRITORY_MULTIPLIER",
                Slot::Value(T.city_territory_multiplier),
            ),
            (
                "FORT_TERRITORY_MULTIPLIER",
                Slot::Value(T.fort_territory_multiplier),
            ),
            (
                "CAPITAL_TERRITORY_BONUS",
                Slot::Value(T.capital_territory_bonus),
            ),
            (
                "COLOSSEUM_TERRITORY_BONUS",
                Slot::Value(T.colosseum_territory_bonus),
            ),
            (
                "COLOSSEUM_FORT_BORDERS",
                Slot::Value(T.colosseum_fort_borders),
            ),
            (
                "EIFFEL_TOWER_TERRITORY_BONUS",
                Slot::Value(T.eiffel_tower_territory_bonus),
            ),
            ("GEMS_TERRITORY_BONUS", Slot::Value(T.gems_territory_bonus)),
            ("ROMAN_FORT_BORDERS", Slot::Value(T.roman_fort_borders)),
            ("RUSSIAN_BORDERS", Slot::Value(T.russian_borders)),
            (
                "RUSSIAN_BORDERS_PER_AGE",
                Slot::Value(T.russian_borders_per_age),
            ),
            ("TIKAL_TEMPLE_BORDERS", Slot::Value(T.tikal_temple_borders)),
            ("ATTRITION_IMPROVED", Slot::Entries(&T.attrition_improved)),
            ("ATTRITION_UPGRADE", Slot::Entries(&T.attrition_upgrade)),
            ("CITY_UPGRADE_TERR", Slot::Entries(&T.city_upgrade_terr)),
            ("FORT_UPGRADE_TERR", Slot::Entries(&T.fort_upgrade_terr)),
            ("TEMPLE_UPGRADE_TERR", Slot::Entries(&T.temple_upgrade_terr)),
            ("CIVIC_UPGRADE_TERR", Slot::Entries(&T.civic_upgrade_terr)),
            ("SUPPLY_RADIUS", Slot::Value(T.supply_radius)),
            (
                "SUPPLY_RADIUS_UPGRADE",
                Slot::Value(T.supply_radius_upgrade),
            ),
            ("TERRA_COTTA_RANGE", Slot::Value(T.terra_cotta_range)),
            ("GENERAL_RADIUS", Slot::Value(T.general_radius)),
            (
                "PARMENIO_RADIUS_ADJUST",
                Slot::Ratio256(T.parmenio_radius_adjust),
            ),
            ("WELLINGTON_RADIUS", Slot::Value(T.wellington_radius)),
            ("KUTOSOV_RADIUS", Slot::Value(T.kutosov_radius)),
            (
                "MIL_PATRIOT_RADIUS_BONUS",
                Slot::Value(T.mil_patriot_radius_bonus),
            ),
            (
                "ECON_PATRIOT_RADIUS_BONUS",
                Slot::Value(T.econ_patriot_radius_bonus),
            ),
            ("SUPPLY_HEAL_RATE", Slot::Value(T.supply_heal_rate)),
            (
                "FRENCH_SUPPLY_HEAL_RATE",
                Slot::Value(T.french_supply_heal_rate),
            ),
            (
                "VERSAILLES_SUPPLY_HEAL_RATE",
                Slot::Value(T.versailles_supply_heal_rate),
            ),
            (
                "ARTILLERY_UNDER_ATTACK_FIRES_SLOWLY",
                Slot::Value(T.artillery_under_attack_fires_slowly),
            ),
            ("UNIT_TURN_SPEED", Slot::Ratio256(T.unit_turn_speed)),
            ("UNIT_PACK_TURN_BONUS", Slot::Value(T.unit_pack_turn_bonus)),
            ("GATHER_RATE", Slot::Value(T.gather_rate)),
            ("REFINERY_BONUS", Slot::Value(T.refinery_bonus)),
            ("VILLAGE_TAXES", Slot::Value(T.village_taxes)),
            ("BUILDING_TAXES", Slot::Value(T.building_taxes)),
            ("MARKET_TAXES", Slot::Value(T.market_taxes)),
            ("TEMPLE_TAXES", Slot::Value(T.temple_taxes)),
            ("VILLAGE_LITERACY", Slot::Value(T.village_literacy)),
            ("UNIVERSITY_LITERACY", Slot::Value(T.university_literacy)),
            ("LIBRARY_LITERACY", Slot::Value(T.library_literacy)),
            ("STARTING_GOODS", Slot::Entries(&T.starting_goods)),
            ("BASIC_GATHER", Slot::Entries(&T.basic_gather)),
            ("CITY_GATHER", Slot::Entries(&T.city_gather)),
            ("GRANARY_BONUS", Slot::Entries(&T.granary_bonus)),
            ("LUMBERMILL_BONUS", Slot::Entries(&T.lumbermill_bonus)),
            ("SMELTER_BONUS", Slot::Entries(&T.smelter_bonus)),
            ("TERRITORY_TAXES", Slot::Entries(&T.territory_taxes)),
            ("COMMERCE_CAP", Slot::Entries(&T.commerce_cap)),
            ("PEASANT_RATE", Slot::Ratio256(T.peasant_rate)),
            ("OIL_RATE", Slot::Ratio256(T.oil_rate)),
            ("SCHOLAR_RATE", Slot::Entries256(&T.scholar_rate)),
            ("UNIT_COST_FACTOR", Slot::Value(T.unit_cost_factor)),
            ("BUILD_COST_FACTOR", Slot::Value(T.build_cost_factor)),
            ("TECH_COST_FACTOR", Slot::Value(T.tech_cost_factor)),
            ("SPELL_COST_FACTOR", Slot::Value(T.spell_cost_factor)),
            ("BUILD_SUPPORT_FACTOR", Slot::Value(T.build_support_factor)),
            (
                "UNIT_SCHOLAR_RAMP_MAX",
                Slot::Value(T.unit_scholar_ramp_max),
            ),
            ("UNIT_WORKER_RAMP_MAX", Slot::Value(T.unit_worker_ramp_max)),
            (
                "UNIT_OTHER_CIVILIAN_RAMP_MAX",
                Slot::Value(T.unit_other_civilian_ramp_max),
            ),
            (
                "UNIT_MILITARY_RAMP_MAX",
                Slot::Value(T.unit_military_ramp_max),
            ),
            ("MAIZE_RAMPING_BONUS", Slot::Value(T.maize_ramping_bonus)),
            ("UNIT_REFIT_MAX_COST", Slot::Value(T.unit_refit_max_cost)),
            ("RESEARCH_PREMIUM", Slot::Ratio256(T.research_premium)),
            ("ACCEL_TRAIN", Slot::Ratio100(T.accel_train)),
            ("ACCEL_CONSTRUCT", Slot::Ratio100(T.accel_construct)),
            ("ACCEL_RESEARCH", Slot::Ratio100(T.accel_research)),
            ("UNIT_RATE_BASE", Slot::Ratio100(T.unit_rate_base)),
            (
                "UNIT_RATE_PROGRESSION",
                Slot::Ratio100(T.unit_rate_progression),
            ),
            (
                "RESEARCH_TICK_PREMIUM",
                Slot::Ratio256(T.research_tick_premium),
            ),
            (
                "TECH_SCIENCE_DISCOUNT",
                Slot::Value(T.tech_science_discount),
            ),
            ("TECH_SCIENCE_SPEEDUP", Slot::Value(T.tech_science_speedup)),
            ("RAMP_FINAL", Slot::Value(T.ramp_final)),
            (
                "TECH_AGE_BEHIND_DISCOUNT",
                Slot::Value(T.tech_age_behind_discount),
            ),
            (
                "TECH_AGE_BEHIND_KNOWLEDGE_DISCOUNT",
                Slot::Value(T.tech_age_behind_knowledge_discount),
            ),
            (
                "TECH_COLOR_BEHIND_DISCOUNT",
                Slot::Value(T.tech_color_behind_discount),
            ),
            (
                "TECH_COLOR_BEHIND_KNOWLEDGE_DISCOUNT",
                Slot::Value(T.tech_color_behind_knowledge_discount),
            ),
            (
                "MILITARY_UNIT_DISCOUNT",
                Slot::Value(T.military_unit_discount),
            ),
            (
                "MILITARY_UPGRADE_DISCOUNT",
                Slot::Value(T.military_upgrade_discount),
            ),
            ("POP_CAP", Slot::Entries(&T.pop_cap)),
            ("VILLAGE_POP", Slot::Value(T.village_pop)),
            ("COLOSSUS_POP_CAP", Slot::Value(T.colossus_pop_cap)),
            ("BANTU_POP_CAP", Slot::Value(T.bantu_pop_cap)),
            ("BANTU_FINAL_POP_CAP", Slot::Value(T.bantu_final_pop_cap)),
            ("PEACOCKS_POP", Slot::Value(T.peacocks_pop)),
            (
                "GREEK_KNOWLEDGE_EARLY",
                Slot::Value(T.greek_knowledge_early),
            ),
            (
                "GREEK_UNIVERSITY_EARLY",
                Slot::Value(T.greek_university_early),
            ),
            ("ROMAN_FORT_EARLY", Slot::Value(T.roman_fort_early)),
            (
                "EGYPTIAN_GRANARY_EARLY",
                Slot::Value(T.egyptian_granary_early),
            ),
            (
                "FRENCH_LUMBERMILL_EARLY",
                Slot::Value(T.french_lumbermill_early),
            ),
            ("GERMAN_METAL_EARLY", Slot::Value(T.german_metal_early)),
            (
                "GERMAN_BUILDINGS_EARLY",
                Slot::Value(T.german_buildings_early),
            ),
            (
                "KOREAN_TEMPLE_UPGRADES",
                Slot::Value(T.korean_temple_upgrades),
            ),
            (
                "EGYPTIAN_WONDERS_EARLY",
                Slot::Value(T.egyptian_wonders_early),
            ),
            (
                "GERMAN_INDUSTRY_EARLY",
                Slot::Value(T.german_industry_early),
            ),
            ("IROQUOIS_GOVS_EARLY", Slot::Value(T.iroquois_govs_early)),
            (
                "GREEK_DELAY_KNOWLEDGE",
                Slot::Value(T.greek_delay_knowledge),
            ),
            ("CHINESE_HERBAL_LORE", Slot::Value(T.chinese_herbal_lore)),
            (
                "RED_FORT_FORTIFICATION",
                Slot::Value(T.red_fort_fortification),
            ),
            ("RED_FORT_TACTICS", Slot::Value(T.red_fort_tactics)),
            ("LAKOTA_CAV_UPGRADES", Slot::Value(T.lakota_cav_upgrades)),
            (
                "IROQUOIS_SCOUT_UPGRADES",
                Slot::Value(T.iroquois_scout_upgrades),
            ),
            (
                "INDIANS_ELEPHANT_UPGRADES",
                Slot::Value(T.indians_elephant_upgrades),
            ),
            (
                "KOREAN_MILITIA_UPGRADES",
                Slot::Value(T.korean_militia_upgrades),
            ),
            (
                "KOREAN_TEMPLE_TAX_UPGRADES",
                Slot::Value(T.korean_temple_tax_upgrades),
            ),
            ("PERSIANS_TAXATION", Slot::Value(T.persians_taxation)),
            ("MONGOL_FREE_FORAGE", Slot::Value(T.mongol_free_forage)),
            (
                "RUSSIAN_ATTRITION_UPGRADES",
                Slot::Value(T.russian_attrition_upgrades),
            ),
            (
                "EGYPTIAN_GRANARY_UPGRADES",
                Slot::Value(T.egyptian_granary_upgrades),
            ),
            ("ROMAN_FORT_UPGRADES", Slot::Value(T.roman_fort_upgrades)),
            (
                "HANGING_GARDENS_UPGRADES",
                Slot::Value(T.hanging_gardens_upgrades),
            ),
            (
                "FRENCH_LUMBERMILL_UPGRADES",
                Slot::Value(T.french_lumbermill_upgrades),
            ),
            (
                "CHINESE_KNOWLEDGE_UPGRADES",
                Slot::Value(T.chinese_knowledge_upgrades),
            ),
            (
                "GERMAN_METAL_UPGRADES",
                Slot::Value(T.german_metal_upgrades),
            ),
            (
                "GERMAN_INDUSTRY_UPGRADES",
                Slot::Value(T.german_industry_upgrades),
            ),
            (
                "GERMAN_HEAVY_INFANTRY",
                Slot::Value(T.german_heavy_infantry),
            ),
            (
                "BRITISH_ARCHER_UPGRADES",
                Slot::Value(T.british_archer_upgrades),
            ),
            (
                "SPANISH_SCOUT_UPGRADES",
                Slot::Value(T.spanish_scout_upgrades),
            ),
            (
                "TURK_FREE_SIEGE_UPGRADES",
                Slot::Value(T.turk_free_siege_upgrades),
            ),
            (
                "LIBERTY_FREE_UPGRADES",
                Slot::Value(T.liberty_free_upgrades),
            ),
            (
                "COLOSSEUM_FORT_UPGRADES",
                Slot::Value(T.colosseum_fort_upgrades),
            ),
            (
                "TIKAL_TEMPLE_UPGRADES",
                Slot::Value(T.tikal_temple_upgrades),
            ),
            ("DUTCH_FREE_COMMERCE", Slot::Value(T.dutch_free_commerce)),
            ("RUSSIAN_FREE_CIVIC", Slot::Value(T.russian_free_civic)),
            ("AZTEC_FREE_MILITARY", Slot::Value(T.aztec_free_military)),
            ("ROMAN_FREE_MILITARY", Slot::Value(T.roman_free_military)),
            (
                "AMERICANS_FREE_SCIENCE",
                Slot::Value(T.americans_free_science),
            ),
            ("PERSIANS_DESPOTISM", Slot::Value(T.persians_despotism)),
            ("FLANK_BONUS", Slot::Value(T.flank_bonus)),
            ("CAVALRY_FLANK_BONUS", Slot::Value(T.cavalry_flank_bonus)),
            ("VEHICLE_FLANK_BONUS", Slot::Value(T.vehicle_flank_bonus)),
            ("ROCKY_MODIFIER", Slot::Ratio256(T.rocky_modifier)),
            ("OVERKILL_FRAMES", Slot::Value(T.overkill_frames)),
            ("OVERKILL_DAMAGE", Slot::Ratio256(T.overkill_damage)),
            (
                "ENTRENCHMENT_MODIFIER",
                Slot::Ratio256(T.entrenchment_modifier),
            ),
            ("RIVER_MODIFIER", Slot::Ratio256(T.river_modifier)),
            (
                "RECAPTURE_CITY_MODIFIER",
                Slot::Ratio256(T.recapture_city_modifier),
            ),
            ("HEIGHT_INCREMENT", Slot::Value(T.height_increment)),
            ("HEIGHT_BONUS", Slot::Value(T.height_bonus)),
            ("ONE_AGE_DOWN", Slot::Value(T.one_age_down)),
            ("TWO_AGES_DOWN", Slot::Value(T.two_ages_down)),
            ("THREE_AGES_DOWN", Slot::Value(T.three_ages_down)),
            ("FOUR_AGES_DOWN", Slot::Value(T.four_ages_down)),
            ("FIVE_AGES_DOWN", Slot::Value(T.five_ages_down)),
            ("TARGET_RADIUS", Slot::Ratio192(T.target_radius)),
            ("RED_FORT_AIR_DEFENSE", Slot::Value(T.red_fort_air_defense)),
            ("JAPANESE_DAMAGE", Slot::Value(T.japanese_damage)),
            (
                "RUSSIAN_COSSACK_DAMAGE",
                Slot::Value(T.russian_cossack_damage),
            ),
            (
                "WELLINGTON_SIEGE_ATTACK",
                Slot::Value(T.wellington_siege_attack),
            ),
            (
                "ANTIPATER_ENTRENCH_BONUS",
                Slot::Ratio256(T.antipater_entrench_bonus),
            ),
            ("SUPER_IMMUNE", Slot::Value(T.super_immune)),
            ("UNIT_RESPOND_RANGE", Slot::Value(T.unit_respond_range)),
            (
                "UNIT_DEFENSIVE_RESPOND_RANGE",
                Slot::Value(T.unit_defensive_respond_range),
            ),
            (
                "UNIT_GUARD_RESPOND_RANGE",
                Slot::Value(T.unit_guard_respond_range),
            ),
        ]
    }
}
