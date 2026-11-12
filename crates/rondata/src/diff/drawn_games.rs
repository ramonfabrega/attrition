//! **The games the renderer's census walks** (item 4166, `docs/RENDERER.md`
//! §3.13): every late-age pair and every battery word, each as the
//! handoff's `Late ages:` and `Battery word:` lines name it, with the
//! captures the free runner stands it up from. The list lives here, inside
//! `diff`, because the pair and word modules are `diff`'s own;
//! `free::ages` reads it. `free::ages::the_census_walks_every_late_age_and_battery_game`
//! holds it to `AI_WORDS`' rows on those two lines, so a new row is
//! walked or the test names it.

/// One game: the name its line carries, the start sibling, the long
/// capture and its trace, and the game's last frame.
pub(crate) struct Game {
    pub name: &'static str,
    pub start: &'static str,
    pub long: (&'static str, &'static str),
    pub length: i64,
    /// The game's module: its own tests are the `testkit::GROUND_INEXACT`
    /// rows whose name holds it, and the census reads the largest of them
    /// (the walk to the closing block is the game's longest) beside its own
    /// count of the game's inexact ground reads.
    pub module: &'static str,
}

macro_rules! game {
    ($m:ident, $c:ident) => {
        Game {
            name: super::$m::$c.name,
            start: super::$m::$c.start,
            long: super::$m::$c.long,
            length: super::$m::$c.length,
            module: stringify!($m),
        }
    };
}

pub(crate) const LATE_AGES: &[Game] = &[
    game!(german_alltech_pair, GERMAN_ALLTECH),
    game!(aztec_alltech_pair, AZTEC_ALLTECH),
    game!(russian_modern_pair, RUSSIAN_MODERN),
    game!(american_modern_pair, AMERICAN_MODERN),
    game!(japanese_industrial_pair, JAPANESE_INDUSTRIAL),
    game!(chinese_industrial_pair, CHINESE_INDUSTRIAL),
    game!(british_enlightenment_pair, BRITISH_ENLIGHTENMENT),
    game!(turkish_enlightenment_pair, TURKISH_ENLIGHTENMENT),
    game!(persian_medieval_pair, PERSIAN_MEDIEVAL),
    game!(greek_classical_pair, GREEK_CLASSICAL),
    game!(korean_ancient_pair, KOREAN_ANCIENT),
    game!(roman_classical_pair, ROMAN_CLASSICAL),
    game!(mongol_classical_pair, MONGOL_CLASSICAL),
    game!(chinese_medieval_pair, CHINESE_MEDIEVAL),
    game!(lakota_medieval_pair, LAKOTA_MEDIEVAL),
    game!(bantu_ancient_pair, BANTU_ANCIENT),
    game!(japanese_gunpowder_pair, JAPANESE_GUNPOWDER),
    game!(iroquois_ancient_pair, IROQUOIS_ANCIENT),
    game!(egyptian_classical_pair, EGYPTIAN_CLASSICAL),
    game!(spanish_classical_pair, SPANISH_CLASSICAL),
    game!(maya_ancient_pair, MAYA_ANCIENT),
    game!(german_ancient_pair, GERMAN_ANCIENT),
    game!(korean_classical_pair, KOREAN_CLASSICAL),
    game!(japanese_ancient_pair, JAPANESE_ANCIENT),
    game!(egyptian_gunpowder_pair, EGYPTIAN_GUNPOWDER),
    game!(aztec_ancient_pair, AZTEC_ANCIENT),
    game!(russian_medieval_pair, RUSSIAN_MEDIEVAL),
    game!(inca_classical_pair, INCA_CLASSICAL),
    game!(french_classical_pair, FRENCH_CLASSICAL),
    game!(dutch_medieval_pair, DUTCH_MEDIEVAL),
    game!(roman_medieval_pair, ROMAN_MEDIEVAL),
    game!(mongol_gunpowder_pair, MONGOL_GUNPOWDER),
    game!(bantu_medieval_pair, BANTU_MEDIEVAL),
    game!(chinese_gunpowder_pair, CHINESE_GUNPOWDER),
    game!(greek_gunpowder_pair, GREEK_GUNPOWDER),
    game!(spanish_gunpowder_pair, SPANISH_GUNPOWDER),
    game!(lakota_gunpowder_pair, LAKOTA_GUNPOWDER),
    game!(iroquois_enlightenment_pair, IROQUOIS_ENLIGHTENMENT),
    game!(aztec_medieval_pair, AZTEC_MEDIEVAL),
    game!(inca_enlightenment_pair, INCA_ENLIGHTENMENT),
    game!(indian_enlightenment_pair, INDIAN_ENLIGHTENMENT),
    game!(american_enlightenment_pair, AMERICAN_ENLIGHTENMENT),
    game!(british_classical_pair, BRITISH_CLASSICAL),
    game!(persian_classical_pair, PERSIAN_CLASSICAL),
    game!(japanese_medieval_pair, JAPANESE_MEDIEVAL),
    game!(korean_medieval_pair, KOREAN_MEDIEVAL),
    game!(german_industrial_pair, GERMAN_INDUSTRIAL),
    game!(roman_gunpowder_pair, ROMAN_GUNPOWDER),
    game!(french_enlightenment_pair, FRENCH_ENLIGHTENMENT),
    game!(mongol_enlightenment_pair, MONGOL_ENLIGHTENMENT),
    game!(british_industrial_pair, BRITISH_INDUSTRIAL),
    game!(american_industrial_pair, AMERICAN_INDUSTRIAL),
    game!(russian_gunpowder_pair, RUSSIAN_GUNPOWDER),
    game!(greek_enlightenment_pair, GREEK_ENLIGHTENMENT),
    game!(lakota_enlightenment_pair, LAKOTA_ENLIGHTENMENT),
    game!(chinese_enlightenment_pair, CHINESE_ENLIGHTENMENT),
    game!(japanese_enlightenment_pair, JAPANESE_ENLIGHTENMENT),
    game!(spanish_industrial_pair, SPANISH_INDUSTRIAL),
    game!(persian_gunpowder_pair, PERSIAN_GUNPOWDER),
    game!(inca_industrial_pair, INCA_INDUSTRIAL),
    game!(egyptian_industrial_pair, EGYPTIAN_INDUSTRIAL),
    game!(british_medieval_pair, BRITISH_MEDIEVAL),
    game!(mongol_medieval_pair, MONGOL_MEDIEVAL),
    game!(greek_medieval_pair, GREEK_MEDIEVAL),
    game!(french_medieval_pair, FRENCH_MEDIEVAL),
    game!(spanish_modern_pair, SPANISH_MODERN),
    game!(german_modern_pair, GERMAN_MODERN),
    game!(egyptian_information_pair, EGYPTIAN_INFORMATION),
    game!(russian_information_pair, RUSSIAN_INFORMATION),
    game!(japanese_modern_pair, JAPANESE_MODERN),
    game!(bantu_modern_pair, BANTU_MODERN),
    game!(egyptian_modern_pair, EGYPTIAN_MODERN),
    game!(american_information_pair, AMERICAN_INFORMATION),
    game!(maya_information_pair, MAYA_INFORMATION),
    game!(aztec_modern_pair, AZTEC_MODERN),
    game!(chinese_modern_pair, CHINESE_MODERN),
    game!(british_modern_pair, BRITISH_MODERN),
    game!(roman_modern_pair, ROMAN_MODERN),
    game!(turkish_information_pair, TURKISH_INFORMATION),
    game!(french_information_pair, FRENCH_INFORMATION),
    game!(spanish_information_pair, SPANISH_INFORMATION),
    game!(german_industrial_moderate_pair, GERMAN_INDUSTRIAL_MODERATE),
];

pub(crate) const BATTERY_WORDS: &[Game] = &[
    game!(spanish_lakes_word, SPANISH_LAKES),
    game!(american_lakes_word, AMERICAN_LAKES),
    game!(turkish_word, TURKISH),
    game!(spanish_east_word, SPANISH_EAST),
    game!(inca_word, INCA),
    game!(french_word, FRENCH),
    game!(persian_word, PERSIAN),
    game!(aztec_word, AZTEC),
    game!(american_word, AMERICAN),
    game!(korean_word, KOREAN),
    game!(dutch_sahara_word, DUTCH_SAHARA),
    game!(iroquois_sahara_word, IROQUOIS_SAHARA),
    game!(german_sahara_word, GERMAN_SAHARA),
    game!(inca_sahara_word, INCA_SAHARA),
    game!(indian_sahara_word, INDIAN_SAHARA),
    game!(lakota_word, LAKOTA),
    game!(bantu_word, BANTU),
    game!(russian_word, RUSSIAN),
    game!(indian_word, INDIAN),
    game!(maya_word, MAYA),
    game!(egyptian_word, EGYPTIAN),
    game!(dutch_word, DUTCH),
    game!(japanese_word, JAPANESE),
    game!(greek_word, GREEK),
    game!(spanish_word, SPANISH),
    game!(iroquois_word, IROQUOIS),
    game!(roman_word, ROMAN),
    game!(german_word, GERMAN),
];

/// The held-out battery's lobbies (`diff::battery::BATTERY`, DECISIONS 61
/// §6): never read by a measure that steers, so never walked by the
/// census; `free::ages` holds its lists apart from these.
pub(crate) fn held_out() -> impl Iterator<Item = &'static str> {
    super::battery::BATTERY.iter().map(|l| l.name)
}
