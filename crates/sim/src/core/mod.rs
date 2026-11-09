//! `core/` — the files `docs/STRUCTURE.md` §3 maps here (item 4011).

pub mod flag_reads;
pub mod frame;
pub mod tuning;
pub mod type_index;

/// The load-time tables of [`crate::Sim`] (`docs/STRUCTURE.md` §3's shared
/// state, item 4016): five fields read across the crate and written only
/// while a game is set up — `Sim::new`, `add_unit_type`, `add_build_type`,
/// `set_tech_tree` and the lobby pass (`sync_setup_from_lobby`,
/// `finalize_type_ages`). §3's sixth, `nation`, is rewritten whenever a
/// player gains or loses a city (`refresh_nation_powers`), so it is
/// `cities/`'s state. Moved whole, each with its doc, in `Sim`'s order.
#[derive(Clone, Debug)]
pub struct Tables {
    pub tuning: crate::Tuning,
    /// The unit types this simulation can build. A type's index is its id, the
    /// way the original's tables are index-keyed; see `docs/DECISIONS.md`
    /// entry 9.
    pub unit_types: Vec<crate::UnitType>,
    /// The tech tree — what may be bought and what owning it changes; see
    /// `docs/TECH.md`. Empty until [`Sim::set_tech_tree`], and a unit type
    /// joins it through [`UnitType::tree`].
    pub tech_tree: crate::tech::TechTree,
    /// The lobby settings the tree reads: starting and ending ages, the
    /// game rules, the no-powers flags.
    pub setup: crate::tech::Setup,
    /// The building types — `docs/CITIES.md` §1.5. A building with
    /// [`Building::ty`] set obeys the placement, construction, city and
    /// garrison rules; one without is the bare production-and-combat object
    /// the earlier mechanics used.
    pub build_types: Vec<crate::cities::build::BuildType>,
}
