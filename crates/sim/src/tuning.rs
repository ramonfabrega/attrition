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
    /// What each player starts with, per good — before the lobby's
    /// `STARTING_RESOURCES` row scales it. See `docs/COSTS.md`, "The
    /// starting grant arrives with the good".
    pub starting_goods: [i32; 6],
    /// Persians: percent added to the starting **food** grant. The file
    /// writes `50%` and the engine reads the face value, so the grant is
    /// `(50 + 100) x food / 100`.
    pub persians_bonus_food: i32,
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
    /// Percentage a **fisherman** adds to the food half of the rare it
    /// stands on, by Fishermen upgrade level — `LeaderData::calc_rare`'s
    /// one addition, and the reason a level-0 fisherman on a fish pays the
    /// raw `BONUS_NUM0` (`docs/ECONOMY.md`, step 6). Level 0 ships as 0%.
    pub fishermen_bonus: [i32; 5],
    /// Percentage a **merchant** scales its rare's payout by, by Merchants
    /// upgrade level. It *replaces* the 100 rather than adding to it, and
    /// it reaches the non-food half of a fish or a whale as well as
    /// everything a merchant stands on in friendly ground. Level 0 ships as
    /// 100%, so it is inert until Taxation.
    pub merchants_bonus: [i32; 5],
    /// Percentage the **Whales** rare adds to every naval type's cached
    /// speed — `Unit::update_speed@006055c0`'s one rare arm
    /// (`docs/ECONOMY.md`, "What an owned rare does").
    pub whales_ships_move: i32,
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
    /// The market's price cycle (`GameDaemon::calc_markets`, `docs/SYNC.md`
    /// §3.1): the floor a price steps up past two at a time, the price
    /// every good drifts toward, the least half-variance of a flux, the
    /// least and the spread of a trend's length in ticks, and how many
    /// frames make a tick.
    pub market_basement: i32,
    pub market_equilibrium: i32,
    /// What one trade moves a good's own price by — `do_sell` walks it
    /// down, `do_buy` up (`docs/ECONOMY.md` §12). The file's value carries
    /// its own gloss, "3 +/- to sell price (double to buy price)", which is
    /// the `price * 2` in `calc_market_prices` rather than a second
    /// constant.
    pub market_supply_demand: i32,
    /// The three constants `LeaderData::calc_market_prices` reads beyond
    /// the price itself: the Nubians' spread bonus, Amber's, and the
    /// Supercollider's buy ceiling and sell floor.
    pub nubian_market_prices: i32,
    pub amber_market: i32,
    pub super_buy: i32,
    pub super_sell: i32,
    /// Ships as **0**, which switches off the arm that would pin both
    /// market prices at a flat 100 for a Russian past the fifth age.
    pub russian_communism: i32,
    pub market_min_variance: i32,
    pub market_min_trend: i32,
    pub market_trend_range: i32,
    pub market_cycle_rate: i32,
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
    /// Percentage the British add to every commerce cap.
    pub british_commerce: i32,
    /// The same, for the Egyptians on food.
    pub egyptian_food_commerce: i32,
    /// The same, for the French on timber.
    pub french_timber_commerce: i32,
    /// The same, for the Inca on wealth.
    pub inca_wealth_cap: i32,
    /// What finishing a farm pays, once per gather slot the player has never
    /// held before — `Build::activate`'s tail through `Build::do_bonus`.
    /// A flat amount, not per slot.
    pub food_bonus_for_farm: i32,
    /// The same for a woodcutter's camp, and this one *is* per new slot.
    pub timber_bonus_per_wood_slot: i32,
    /// The same for a university. Flat.
    pub knowledge_bonus_for_university: i32,
    /// The same for a mine, per new slot.
    pub metal_bonus_per_mine_slot: i32,
    /// The same for an oil well or platform. Flat.
    pub oil_bonus_for_well: i32,
    /// Percentage the Germans add to a completion bonus.
    pub german_completion_bonus: i32,

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
    /// `GOODY_BOX`: the flat half of a goody box's pile (`docs/GOODY.md`
    /// §3). Written `25 resources` and read by `Constants::get_item`, so it
    /// arrives plain — not one of the `get_fraction` neighbours.
    pub goody_box: i32,
    /// `GOODY_BOX_AGE`: the per-level half, multiplied by the finder's
    /// **Science** library level and not by their age.
    pub goody_box_age: i32,
    /// `SPANISH_RUINS_BASE`: [`Tuning::goody_box`]'s replacement for a
    /// player with `has_tribe_bonus(9)`.
    pub spanish_ruins_base: i32,
    /// `SPANISH_RUINS`: [`Tuning::goody_box_age`]'s replacement, likewise.
    pub spanish_ruins: i32,
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
    /// How far, in tiles, an idle citizen looks for a site to build (`find_build_spot`).
    pub unit_build_respond_range: i32,
    /// How far, in tiles, an idle citizen looks for a building to gather at (`find_gather_spot`).
    pub unit_gather_respond_range: i32,
    /// Percent of the base construction time for a player with no city yet — a nomad's first city. Ships as 300.
    pub capital_build_time: i32,
    /// Percent more hit points per `BUILDINGS_HP_n` tech held.
    pub building_hp_upgrade: i32,
    /// Percent more hit points for a non-defensive building per city level above the first.
    pub senate_hp_bonus: i32,
    /// Percent more hit points for a city with a temple, by temple level.
    pub temple_upgrade_hp: [i32; 5],
    /// Percent by which Tikal raises the temple hit-point bonus.
    pub tikal_temple_hp: i32,
    /// A city's radius in tiles at level one.
    pub city_center_radius: i32,
    /// Added to the radius per city level above one.
    pub city_center_pop_radius: i32,
    /// Added to the radius for the Indians.
    pub indians_city_radius: i32,
    /// Distinct completed building kinds a city needs, besides itself, to become a Large City.
    pub city_buildings: i32,
    /// The same for a Major City.
    pub metro_buildings: i32,
    /// Minimum distance between cities, in tiles; a city at exactly this distance is refused.
    pub city_spacing: i32,
    /// Taken off the spacing once the placer holds ninety percent of the region.
    pub relax_city_spacing: i32,
    /// Tiles (read in cells, divided by four) within which a colonising city or fort must find open sea; zero disables the rule.
    pub first_city_near_coast: i32,
    /// Minimum distance from a fort to a friendly fort or city, in tiles.
    pub fort_spacing: i32,
    /// Minimum distance from a fort to an enemy city, in tiles.
    pub fort_to_enemy_city_spacing: i32,
    /// Farms a city may hold.
    pub farms_per_city_base: i32,
    /// Added per city level above one. Ships as zero.
    pub farms_per_city_level: i32,
    /// The Egyptian base instead.
    pub egyptian_farms_per_city_base: i32,
    /// Added to the Bantu city limit when the Civic level is non-zero.
    pub bantu_city_limit: i32,
    /// Added to the city limit by the Pyramids.
    pub pyramids_city_limit: i32,
    /// Whether a Dutch fort may stand on unowned ground anywhere. Ships off.
    pub dutch_fort_placement: i32,
    /// Radius in tiles within which units and buildings are counted for a city capture.
    pub city_capture_radius: i32,
    /// Plunder on capture per city level above one.
    pub city_plunder_per_level: i32,
    /// Plunder floor for a first capital capture, paid in every usable good.
    pub capital_plunder: i32,
    /// Percent of a building's plunder value an enemy collects on its kill.
    pub plunder: i32,
    /// Percent bonus on plunder for an Aztec attacker.
    pub aztec_plunder: i32,
    /// Percent of plunder the Despot (or Spitamenes) adds as the hero cut.
    pub thedespot_plunder: i32,
    /// Whether a Russian victim receives the plunder of its own building.
    pub russian_plunder_steal: i32,
    /// Percent off a German mine's plunder value.
    pub german_mine_cost: i32,
    /// Percent off a Tikal owner's own temple's plunder value.
    pub tikal_temple_cost: i32,
    /// Frames a captured city takes to assimilate.
    pub assimilation_timer: i32,
    /// Percent bonus on a Turkish owner's elapsed assimilation time.
    pub turk_assimilate: i32,
    /// Percent bonus on the elapsed time when the founder retakes their own city.
    pub reassimilation: i32,
    /// How many frames of assimilation pass per frame with the Citizen in the city.
    pub thecitizen_assimilation_speed: i32,
    /// Frames between a city's self-repair steps.
    pub city_heal_rate: i32,
    /// Whether Chinese cities are founded as Large Cities.
    pub chinese_large_cities: i32,
    /// Percent faster Mayan construction.
    pub maya_building_speed: i32,
    /// Percent faster construction with Versailles. Ships as zero.
    pub versailles_building_speed: i32,
    /// Percent faster construction with the Tobacco rare.
    pub tobacco_building_speed: i32,
    /// Percent faster British ship creation — `ObjectData::train_time`'s
    /// British arm, on any type whose domain is the sea
    /// (`docs/PRODUCTION.md`, "The tail").
    pub british_ship_speed: i32,
    /// Percent faster British creation of the Archers line. Ships as zero,
    /// so the arm is live and inert at once.
    pub british_archer_speed: i32,
    /// Percent faster British anti-air — the *unit* in
    /// `ObjectData::train_time` and the tower in `Wall::update_construct_time`
    /// read the same constant.
    pub british_aa_speed: i32,
    /// Percent faster Dutch fort construction. Ships as zero.
    pub dutch_fort_speed: i32,
    /// Percent faster Roman fort construction.
    pub roman_fort_speed: i32,
    /// Percent off construction time in the Hanging Gardens' city. Ships as zero.
    pub hanging_gardens_build_time: i32,
    /// Percent faster construction with the President nearby.
    pub thepresident_building_speed: i32,
    /// Whether the Iroquois' first senate is instant.
    pub iroquois_quick_senate: i32,

    // ---- `Build::activate`'s free units (`crate::nations`) ----
    /// Whether a Nubian Market delivers a free caravan. Ships as zero, so
    /// the arm is live and inert at once.
    pub nubian_free_caravan: i32,
    /// Scouts an Iroquois Barracks delivers.
    pub iroquois_free_scout: i32,
    /// Legions a Roman Barracks delivers per tier of the age ladder, and the
    /// cap on their product.
    pub roman_barracks_legion: i32,
    pub roman_max_legion: i32,
    /// The ladder the legion count is scaled by: the ages at which it
    /// reaches one, two and three.
    pub roman_age_for_1_legion: i32,
    pub roman_age_for_2_legions: i32,
    pub roman_age_for_3_legions: i32,
    /// The same ladder for a British Barracks' archers — but the tier **is**
    /// the count here, with neither multiplier nor cap.
    pub british_age_for_1_archer: i32,
    pub british_age_for_2_archers: i32,
    pub british_age_for_3_archers: i32,
    /// Light infantry an Aztec Barracks delivers per tier, and their cap.
    pub aztec_barracks_light: i32,
    pub aztec_max_light: i32,
    /// Siege engines a Turkish Siege Factory delivers.
    pub turk_free_siege: i32,
    /// Whether a French Siege Factory delivers a supply wagon, and a French
    /// fort a general.
    pub french_free_supply: i32,
    pub french_free_general: i32,
    /// Triremes a Spanish Dock delivers, before the Gunpowder age.
    pub spanish_free_trireme: i32,
    /// Light ships a Dutch Dock delivers.
    pub dutch_free_light_ship: i32,
    /// Fishermen a British Dock delivers. Ships as zero.
    pub british_free_fishermen: i32,
    /// Scholars an American University delivers, and bombers an American
    /// Airbase delivers once the Modern age is owned.
    pub americans_free_scholar: i32,
    pub americans_free_bomber: i32,
    /// Fighters a German Airbase delivers.
    pub german_free_fighter: i32,
    /// Cavalry a Mongol Stable delivers: `start` below Military level two,
    /// `free` at or above it, and `three_mil` as a floor past level two.
    pub mongol_start_cavalry: i32,
    pub mongol_free_cavalry: i32,
    pub mongol_three_mil_cavalry: i32,
    /// Citizens a Korean city delivers, indexed by the city count less one.
    pub korean_citizens: [i32; 9],
    /// Whether Korean builders and repairers ignore the under-attack penalty.
    pub korean_build_under_fire: i32,
    /// Percent off the Korean repair period.
    pub korean_repair: i32,
    /// Percent more hit points on every Mayan building.
    pub maya_building_hp: i32,
    /// Percent more on Roman forts and towers. Ships as zero.
    pub roman_fort_hp: i32,
    /// Percent more on every building with the Taj Mahal.
    pub taj_building_hp: i32,
    /// Percent more on the other forts with the Red Fort.
    pub red_fort_fort_hps: i32,
    /// Percent more on a Nubian market.
    pub nubian_hit_points: i32,
    /// Garrison slots a tower gains per fort-garrison tech level.
    pub tower_garrison_upgrade: i32,
    /// The same for a fort.
    pub fort_garrison_upgrade: i32,
    /// Frames between garrison heal steps, by heal tech level.
    pub unit_heal_rate: [i32; 4],
    /// Percent faster the garrison heal runs in, or with, the Red Fort.
    pub red_fort_heal: i32,
    /// Position units: the near edge of the exit ring a unit leaving a building lands on, beyond the footprint. `3/2 tile`.
    pub unit_train_distance: i32,
    /// Position units: the far edge of that ring. `5/2 tile`.
    pub unit_train_max_distance: i32,
    /// Position units: the same pair for a unit whose **domain is the
    /// sea** — `Unit::come_out@00617c10`'s other arm (`6183dd`), which
    /// also adds the type's own `big_radius`. `3/2 tile`.
    pub boat_train_distance: i32,
    /// Position units: the far edge of a boat's exit ring. `8 tiles`, more
    /// than three times a land unit's, because a dock's water may be.
    pub boat_train_max_distance: i32,
    /// Position units: how far from a boarding unit `cast_transport` looks
    /// for the water its transport is born on (`docs/TRANSPORT.md` §6).
    /// `3/1 tile`.
    pub unit_board_distance: i32,
    /// Position units: how far past its own `block_radius` a passenger put
    /// ashore looks for its spot, swept from the boat's own heading
    /// (`docs/TRANSPORT.md` §6.4). `3/1 tile`, the same as the boarding
    /// distance and a different constant.
    pub unit_disembark_distance: i32,
    /// Whether Lakota razing is charged — when non-zero, a Lakota disband refunds in full and a Lakota kill of its own building plunders nothing.
    pub lakota_raze_price: i32,
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
        persians_bonus_food: 50,
        basic_gather: [0, 0, 0, 0, 0, 0],
        city_gather: [10, 10, 0, 0, 0, 0],
        peasant_rate: 2560,
        oil_rate: 8960,
        scholar_rate: [1280, 1792, 2560, 3840, 5120, 6400],
        fishermen_bonus: [0, 50, 100, 200, 200],
        merchants_bonus: [100, 120, 150, 200, 300],
        whales_ships_move: 20,
        granary_bonus: [20, 50, 100, 200, 250],
        lumbermill_bonus: [20, 50, 100, 200, 250],
        smelter_bonus: [50, 100, 150, 200, 250],
        refinery_bonus: 33,
        village_taxes: 0,
        building_taxes: 0,
        market_taxes: 10,
        market_basement: 10,
        market_equilibrium: 65,
        market_supply_demand: 3,
        nubian_market_prices: 20,
        amber_market: 10,
        super_buy: 125,
        super_sell: 50,
        russian_communism: 0,
        market_min_variance: 2,
        market_min_trend: 8,
        market_trend_range: 16,
        market_cycle_rate: 1,
        temple_taxes: 0,
        village_literacy: 0,
        university_literacy: 10,
        library_literacy: 0,
        territory_taxes: [0, 50, 100, 200, 300],
        commerce_cap: [70, 100, 150, 200, 260, 320, 400, 500],
        british_commerce: 25,
        egyptian_food_commerce: 10,
        french_timber_commerce: 10,
        inca_wealth_cap: 33,
        food_bonus_for_farm: 20,
        timber_bonus_per_wood_slot: 5,
        knowledge_bonus_for_university: 25,
        metal_bonus_per_mine_slot: 5,
        oil_bonus_for_well: 50,
        german_completion_bonus: 50,

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
        goody_box: 25,
        goody_box_age: 25,
        spanish_ruins_base: 30,
        spanish_ruins: 26,
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
        unit_build_respond_range: 12,
        unit_gather_respond_range: 32,
        capital_build_time: 300,
        building_hp_upgrade: 10,
        senate_hp_bonus: 35,
        temple_upgrade_hp: [25, 50, 100, 150, 200],
        tikal_temple_hp: 50,
        city_center_radius: 20,
        city_center_pop_radius: 4,
        indians_city_radius: 4,
        city_buildings: 5,
        metro_buildings: 9,
        city_spacing: 24,
        relax_city_spacing: 6,
        first_city_near_coast: 16,
        fort_spacing: 12,
        fort_to_enemy_city_spacing: 32,
        farms_per_city_base: 5,
        farms_per_city_level: 0,
        egyptian_farms_per_city_base: 7,
        bantu_city_limit: 1,
        pyramids_city_limit: 1,
        dutch_fort_placement: 0,
        city_capture_radius: 10,
        city_plunder_per_level: 100,
        capital_plunder: 500,
        plunder: 100,
        aztec_plunder: 100,
        thedespot_plunder: 100,
        russian_plunder_steal: 1,
        german_mine_cost: 0,
        tikal_temple_cost: 0,
        assimilation_timer: 2000,
        turk_assimilate: 200,
        reassimilation: 300,
        thecitizen_assimilation_speed: 4,
        city_heal_rate: 4,
        chinese_large_cities: 1,
        maya_building_speed: 20,
        versailles_building_speed: 0,
        tobacco_building_speed: 10,
        british_ship_speed: 33,
        british_archer_speed: 0,
        british_aa_speed: 33,
        dutch_fort_speed: 0,
        roman_fort_speed: 50,
        hanging_gardens_build_time: 0,
        thepresident_building_speed: 33,
        iroquois_quick_senate: 1,
        nubian_free_caravan: 0,
        iroquois_free_scout: 1,
        roman_barracks_legion: 1,
        roman_max_legion: 3,
        roman_age_for_1_legion: 1,
        roman_age_for_2_legions: 3,
        roman_age_for_3_legions: 5,
        british_age_for_1_archer: 0,
        british_age_for_2_archers: 2,
        british_age_for_3_archers: 3,
        aztec_barracks_light: 1,
        aztec_max_light: 3,
        turk_free_siege: 2,
        french_free_supply: 1,
        french_free_general: 1,
        spanish_free_trireme: 1,
        dutch_free_light_ship: 1,
        british_free_fishermen: 0,
        americans_free_scholar: 1,
        americans_free_bomber: 1,
        german_free_fighter: 2,
        mongol_start_cavalry: 1,
        mongol_free_cavalry: 2,
        mongol_three_mil_cavalry: 3,
        korean_citizens: [1, 3, 5, 5, 5, 5, 5, 5, 5],
        korean_build_under_fire: 1,
        korean_repair: 50,
        maya_building_hp: 25,
        roman_fort_hp: 0,
        taj_building_hp: 100,
        red_fort_fort_hps: 33,
        nubian_hit_points: 50,
        tower_garrison_upgrade: 2,
        fort_garrison_upgrade: 5,
        unit_heal_rate: [20, 15, 10, 5],
        red_fort_heal: 500,
        unit_train_distance: 288,
        unit_train_max_distance: 480,
        boat_train_distance: 288,
        boat_train_max_distance: 1536,
        unit_board_distance: 576,
        unit_disembark_distance: 576,
        lakota_raze_price: 0,
    };

    /// Every value in [`Tuning::RON`] that comes from a named constant in
    /// `rules.xml`, paired with the name the shipped file gives it.
    ///
    /// This is what lets a tool re-derive [`Tuning::RON`] from a real install
    /// and report a drift, rather than us asserting numbers into the void. The
    /// two entries with no constant behind them are absent by design.
    pub const fn ron_slots() -> [(&'static str, Slot); 295] {
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
            ("BRITISH_COMMERCE", Slot::Value(T.british_commerce)),
            (
                "EGYPTIAN_FOOD_COMMERCE",
                Slot::Value(T.egyptian_food_commerce),
            ),
            (
                "FRENCH_TIMBER_COMMERCE",
                Slot::Value(T.french_timber_commerce),
            ),
            ("INCA_WEALTH_CAP", Slot::Value(T.inca_wealth_cap)),
            ("FOOD_BONUS_FOR_FARM", Slot::Value(T.food_bonus_for_farm)),
            (
                "TIMBER_BONUS_PER_WOOD_SLOT",
                Slot::Value(T.timber_bonus_per_wood_slot),
            ),
            (
                "KNOWLEDGE_BONUS_FOR_UNIVERSITY",
                Slot::Value(T.knowledge_bonus_for_university),
            ),
            (
                "METAL_BONUS_PER_MINE_SLOT",
                Slot::Value(T.metal_bonus_per_mine_slot),
            ),
            ("OIL_BONUS_FOR_WELL", Slot::Value(T.oil_bonus_for_well)),
            (
                "GERMAN_COMPLETION_BONUS",
                Slot::Value(T.german_completion_bonus),
            ),
            ("REFINERY_BONUS", Slot::Value(T.refinery_bonus)),
            ("VILLAGE_TAXES", Slot::Value(T.village_taxes)),
            ("BUILDING_TAXES", Slot::Value(T.building_taxes)),
            ("MARKET_TAXES", Slot::Value(T.market_taxes)),
            ("MARKET_BASEMENT", Slot::Value(T.market_basement)),
            ("MARKET_EQUILIBRIUM", Slot::Value(T.market_equilibrium)),
            ("MARKET_SUPPLY_DEMAND", Slot::Value(T.market_supply_demand)),
            ("NUBIAN_MARKET_PRICES", Slot::Value(T.nubian_market_prices)),
            ("AMBER_MARKET", Slot::Value(T.amber_market)),
            ("SUPER_BUY", Slot::Value(T.super_buy)),
            ("SUPER_SELL", Slot::Value(T.super_sell)),
            ("RUSSIAN_COMMUNISM", Slot::Value(T.russian_communism)),
            ("MARKET_MIN_VARIANCE", Slot::Value(T.market_min_variance)),
            ("MARKET_MIN_TREND", Slot::Value(T.market_min_trend)),
            ("MARKET_TREND_RANGE", Slot::Value(T.market_trend_range)),
            ("MARKET_CYCLE_RATE", Slot::Value(T.market_cycle_rate)),
            ("TEMPLE_TAXES", Slot::Value(T.temple_taxes)),
            ("VILLAGE_LITERACY", Slot::Value(T.village_literacy)),
            ("UNIVERSITY_LITERACY", Slot::Value(T.university_literacy)),
            ("LIBRARY_LITERACY", Slot::Value(T.library_literacy)),
            ("STARTING_GOODS", Slot::Entries(&T.starting_goods)),
            ("PERSIANS_BONUS_FOOD", Slot::Value(T.persians_bonus_food)),
            ("BASIC_GATHER", Slot::Entries(&T.basic_gather)),
            ("CITY_GATHER", Slot::Entries(&T.city_gather)),
            ("FISHERMEN_BONUS", Slot::Entries(&T.fishermen_bonus)),
            ("MERCHANTS_BONUS", Slot::Entries(&T.merchants_bonus)),
            ("WHALES_SHIPS_MOVE", Slot::Value(T.whales_ships_move)),
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
            ("GOODY_BOX", Slot::Value(T.goody_box)),
            ("GOODY_BOX_AGE", Slot::Value(T.goody_box_age)),
            ("SPANISH_RUINS_BASE", Slot::Value(T.spanish_ruins_base)),
            ("SPANISH_RUINS", Slot::Value(T.spanish_ruins)),
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
            (
                "UNIT_BUILD_RESPOND_RANGE",
                Slot::Value(T.unit_build_respond_range),
            ),
            (
                "UNIT_GATHER_RESPOND_RANGE",
                Slot::Value(T.unit_gather_respond_range),
            ),
            ("CAPITAL_BUILD_TIME", Slot::Value(T.capital_build_time)),
            ("BUILDING_HP_UPGRADE", Slot::Value(T.building_hp_upgrade)),
            ("SENATE_HP_BONUS", Slot::Value(T.senate_hp_bonus)),
            ("TEMPLE_UPGRADE_HP", Slot::Entries(&T.temple_upgrade_hp)),
            ("TIKAL_TEMPLE_HP", Slot::Value(T.tikal_temple_hp)),
            ("CITY_CENTER_RADIUS", Slot::Value(T.city_center_radius)),
            (
                "CITY_CENTER_POP_RADIUS",
                Slot::Value(T.city_center_pop_radius),
            ),
            ("INDIANS_CITY_RADIUS", Slot::Value(T.indians_city_radius)),
            ("CITY_BUILDINGS", Slot::Value(T.city_buildings)),
            ("METRO_BUILDINGS", Slot::Value(T.metro_buildings)),
            ("CITY_SPACING", Slot::Value(T.city_spacing)),
            ("RELAX_CITY_SPACING", Slot::Value(T.relax_city_spacing)),
            (
                "FIRST_CITY_NEAR_COAST",
                Slot::Value(T.first_city_near_coast),
            ),
            ("FORT_SPACING", Slot::Value(T.fort_spacing)),
            (
                "FORT_TO_ENEMY_CITY_SPACING",
                Slot::Value(T.fort_to_enemy_city_spacing),
            ),
            ("FARMS_PER_CITY_BASE", Slot::Value(T.farms_per_city_base)),
            ("FARMS_PER_CITY_LEVEL", Slot::Value(T.farms_per_city_level)),
            (
                "EGYPTIAN_FARMS_PER_CITY_BASE",
                Slot::Value(T.egyptian_farms_per_city_base),
            ),
            ("BANTU_CITY_LIMIT", Slot::Value(T.bantu_city_limit)),
            ("PYRAMIDS_CITY_LIMIT", Slot::Value(T.pyramids_city_limit)),
            ("DUTCH_FORT_PLACEMENT", Slot::Value(T.dutch_fort_placement)),
            ("CITY_CAPTURE_RADIUS", Slot::Value(T.city_capture_radius)),
            (
                "CITY_PLUNDER_PER_LEVEL",
                Slot::Value(T.city_plunder_per_level),
            ),
            ("CAPITAL_PLUNDER", Slot::Value(T.capital_plunder)),
            ("PLUNDER", Slot::Value(T.plunder)),
            ("AZTEC_PLUNDER", Slot::Value(T.aztec_plunder)),
            ("THEDESPOT_PLUNDER", Slot::Value(T.thedespot_plunder)),
            (
                "RUSSIAN_PLUNDER_STEAL",
                Slot::Value(T.russian_plunder_steal),
            ),
            ("GERMAN_MINE_COST", Slot::Value(T.german_mine_cost)),
            ("TIKAL_TEMPLE_COST", Slot::Value(T.tikal_temple_cost)),
            ("ASSIMILATION_TIMER", Slot::Value(T.assimilation_timer)),
            ("TURK_ASSIMILATE", Slot::Value(T.turk_assimilate)),
            ("REASSIMILATION", Slot::Value(T.reassimilation)),
            (
                "THECITIZEN_ASSIMILATION_SPEED",
                Slot::Value(T.thecitizen_assimilation_speed),
            ),
            ("CITY_HEAL_RATE", Slot::Value(T.city_heal_rate)),
            ("CHINESE_LARGE_CITIES", Slot::Value(T.chinese_large_cities)),
            ("MAYA_BUILDING_SPEED", Slot::Value(T.maya_building_speed)),
            (
                "VERSAILLES_BUILDING_SPEED",
                Slot::Value(T.versailles_building_speed),
            ),
            (
                "TOBACCO_BUILDING_SPEED",
                Slot::Value(T.tobacco_building_speed),
            ),
            ("BRITISH_SHIP_SPEED", Slot::Value(T.british_ship_speed)),
            ("BRITISH_ARCHER_SPEED", Slot::Value(T.british_archer_speed)),
            ("BRITISH_AA_SPEED", Slot::Value(T.british_aa_speed)),
            ("DUTCH_FORT_SPEED", Slot::Value(T.dutch_fort_speed)),
            ("ROMAN_FORT_SPEED", Slot::Value(T.roman_fort_speed)),
            (
                "HANGING_GARDENS_BUILD_TIME",
                Slot::Value(T.hanging_gardens_build_time),
            ),
            (
                "THEPRESIDENT_BUILDING_SPEED",
                Slot::Value(T.thepresident_building_speed),
            ),
            (
                "IROQUOIS_QUICK_SENATE",
                Slot::Value(T.iroquois_quick_senate),
            ),
            ("NUBIAN_FREE_CARAVAN", Slot::Value(T.nubian_free_caravan)),
            ("IROQUOIS_FREE_SCOUT", Slot::Value(T.iroquois_free_scout)),
            (
                "ROMAN_BARRACKS_LEGION",
                Slot::Value(T.roman_barracks_legion),
            ),
            ("ROMAN_MAX_LEGION", Slot::Value(T.roman_max_legion)),
            (
                "ROMAN_AGE_FOR_1_LEGION",
                Slot::Value(T.roman_age_for_1_legion),
            ),
            (
                "ROMAN_AGE_FOR_2_LEGIONS",
                Slot::Value(T.roman_age_for_2_legions),
            ),
            (
                "ROMAN_AGE_FOR_3_LEGIONS",
                Slot::Value(T.roman_age_for_3_legions),
            ),
            (
                "BRITISH_AGE_FOR_1_ARCHER",
                Slot::Value(T.british_age_for_1_archer),
            ),
            (
                "BRITISH_AGE_FOR_2_ARCHERS",
                Slot::Value(T.british_age_for_2_archers),
            ),
            (
                "BRITISH_AGE_FOR_3_ARCHERS",
                Slot::Value(T.british_age_for_3_archers),
            ),
            ("AZTEC_BARRACKS_LIGHT", Slot::Value(T.aztec_barracks_light)),
            ("AZTEC_MAX_LIGHT", Slot::Value(T.aztec_max_light)),
            ("TURK_FREE_SIEGE", Slot::Value(T.turk_free_siege)),
            ("FRENCH_FREE_SUPPLY", Slot::Value(T.french_free_supply)),
            ("FRENCH_FREE_GENERAL", Slot::Value(T.french_free_general)),
            ("SPANISH_FREE_TRIREME", Slot::Value(T.spanish_free_trireme)),
            (
                "DUTCH_FREE_LIGHT_SHIP",
                Slot::Value(T.dutch_free_light_ship),
            ),
            (
                "BRITISH_FREE_FISHERMEN",
                Slot::Value(T.british_free_fishermen),
            ),
            (
                "AMERICANS_FREE_SCHOLAR",
                Slot::Value(T.americans_free_scholar),
            ),
            (
                "AMERICANS_FREE_BOMBER",
                Slot::Value(T.americans_free_bomber),
            ),
            ("GERMAN_FREE_FIGHTER", Slot::Value(T.german_free_fighter)),
            ("MONGOL_START_CAVALRY", Slot::Value(T.mongol_start_cavalry)),
            ("MONGOL_FREE_CAVALRY", Slot::Value(T.mongol_free_cavalry)),
            (
                "MONGOL_THREE_MIL_CAVALRY",
                Slot::Value(T.mongol_three_mil_cavalry),
            ),
            ("KOREAN_CITIZENS", Slot::Entries(&T.korean_citizens)),
            (
                "KOREAN_BUILD_UNDER_FIRE",
                Slot::Value(T.korean_build_under_fire),
            ),
            ("KOREAN_REPAIR", Slot::Value(T.korean_repair)),
            ("MAYA_BUILDING_HP", Slot::Value(T.maya_building_hp)),
            ("ROMAN_FORT_HP", Slot::Value(T.roman_fort_hp)),
            ("TAJ_BUILDING_HP", Slot::Value(T.taj_building_hp)),
            ("RED_FORT_FORT_HPS", Slot::Value(T.red_fort_fort_hps)),
            ("NUBIAN_HIT_POINTS", Slot::Value(T.nubian_hit_points)),
            (
                "TOWER_GARRISON_UPGRADE",
                Slot::Value(T.tower_garrison_upgrade),
            ),
            (
                "FORT_GARRISON_UPGRADE",
                Slot::Value(T.fort_garrison_upgrade),
            ),
            ("UNIT_HEAL_RATE", Slot::Entries(&T.unit_heal_rate)),
            ("RED_FORT_HEAL", Slot::Value(T.red_fort_heal)),
            ("UNIT_TRAIN_DISTANCE", Slot::Ratio192(T.unit_train_distance)),
            (
                "UNIT_TRAIN_MAX_DISTANCE",
                Slot::Ratio192(T.unit_train_max_distance),
            ),
            ("BOAT_TRAIN_DISTANCE", Slot::Ratio192(T.boat_train_distance)),
            (
                "BOAT_TRAIN_MAX_DISTANCE",
                Slot::Ratio192(T.boat_train_max_distance),
            ),
            ("UNIT_BOARD_DISTANCE", Slot::Ratio192(T.unit_board_distance)),
            (
                "UNIT_DISEMBARK_DISTANCE",
                Slot::Ratio192(T.unit_disembark_distance),
            ),
            ("LAKOTA_RAZE_PRICE", Slot::Value(T.lakota_raze_price)),
        ]
    }
}
