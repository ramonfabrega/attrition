//! `cities/` — the files `docs/STRUCTURE.md` §3 maps here (item 4012).

pub mod activate;
pub mod build;
pub mod build_unqueue_view;
pub mod building;
pub mod capture;
pub mod capture_view;
pub mod city;
pub(crate) mod construct;
pub mod farms;
pub mod find_builds_view;
pub mod friends;
pub mod location;
pub mod nations;
pub mod place;
pub mod production;
pub mod queue;
pub mod queue_disband;
pub mod rally;
pub mod site_recruit;
pub mod tcoord;
pub mod unit_unqueue_view;

/// The `cities/` subsystem's fields of [`crate::Sim`] (`docs/STRUCTURE.md`
/// §3, item 4016): moved whole, each with its doc, in `Sim`'s order.
#[derive(Clone, Debug)]
pub struct CitiesState {
    /// `Farms`' list: every farm in the order it was activated
    /// (`Farms::add` runs from `Build::activate`), which is the order
    /// `Farms::inc_time` walks and so the order the farm draws are spent in
    /// (`docs/SYNC.md` §3.3, §4.1). It is not the buildings' order: the
    /// starting positions are built from the higher start slot down, so
    /// the AI's farms come before the human's in every `DUMP_ALL` dump.
    pub farm_order: Vec<usize>,
    /// The city records — `docs/CITIES.md` §5.
    pub cities: Vec<crate::cities::city::City>,
    /// One per player: the nation, wonder and tech inputs this mechanic reads.
    pub nation: Vec<crate::cities::city::Nation>,
    /// `LeaderData::lost_city_stamp`: the frame each player last lost a city.
    pub lost_city_stamp: Vec<Option<i64>>,
    /// `LeaderData::cities_built`, `cities_captured`, `cities_lost`.
    pub city_tally: Vec<crate::cities::city::Tally>,
    /// `LeaderData::high_buildings` (`+0x5660`): per player, the most
    /// buildings of a lineage this player has ever held finished at once,
    /// indexed by the lineage's **root** record. Only `Build::activate`
    /// raises it, and what it gates is the nation's free units
    /// (`crate::cities::nations`). Grown on demand, because the build types are
    /// loaded after the simulation is stood up.
    pub building_high: Vec<Vec<i32>>,
    /// `leader_flags & 0x8000000` per player: the wall stats are stale.
    /// `Leader::process` answers it with `calc_wall_stats` — every unfinished
    /// building's clock re-baked and every building's hit points refreshed —
    /// before any object is processed. `docs/CITIES.md` §3.2.
    pub wall_stats_dirty: Vec<bool>,
    /// `Game::wonders` (`Game +0x618`, `int[17]`), a bit per
    /// [`tech::wonder`] offset: a wonder type some player has activated.
    /// `Wonders::init_wonder@0073c860` sets it (`Wonder::init@0073c5e0` has no caller),
    /// nothing clears it, and `BuildTypeData::already_built@0063ce10` reads
    /// it for [`Sim::type_avail`] (`docs/TECH.md`, "A built wonder is
    /// built for everyone").
    pub wonders_built: u32,
}
