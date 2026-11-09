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

