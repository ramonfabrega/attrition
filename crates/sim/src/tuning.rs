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
    /// A constant the file writes as a rational and the engine loads as 8.8
    /// fixed point, held here already scaled: `3/2` in the file is 384 here.
    ///
    /// The scale is not something the file states. It is fixed by how the
    /// original consumes the value — a multiply followed by a shift right by
    /// eight — so the check has to reconstruct it rather than compare digits.
    Ratio256(i32),
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
    };

    /// Every value in [`Tuning::RON`] that comes from a named constant in
    /// `rules.xml`, paired with the name the shipped file gives it.
    ///
    /// This is what lets a tool re-derive [`Tuning::RON`] from a real install
    /// and report a drift, rather than us asserting numbers into the void. The
    /// two entries with no constant behind them are absent by design.
    pub const fn ron_slots() -> [(&'static str, Slot); 49] {
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
        ]
    }
}
