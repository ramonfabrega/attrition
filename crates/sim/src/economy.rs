//! Income: how a resource arrives.
//!
//! Rise of Nations has no gatherer carrying a load to a drop-off. A citizen
//! assigned to a farm is a **rate**; the rates are summed per player into six
//! numbers, each is clamped by a commerce cap, and the clamped rate is paid
//! into the stockpile every frame through a remainder accumulator.
//!
//! Specified in `docs/ECONOMY.md`, which says which parts of this are read from
//! the original and which are not. The one that matters here: **how many
//! gatherers a building offers is not established**, so [`Site::gatherers`] is
//! an input rather than something this module computes. That is the same call
//! `movement` made for speed and `supply` made for a general's aura radius.
//!
//! # Two scalings, neither announced
//!
//! A rate is **sixteenths of a resource per `GATHER_RATE` frames**, and
//! `GATHER_RATE` ships as 450 — thirty seconds. So a rate of ten food is held
//! here as 160, and pays out ten food every 450 frames exactly.
//!
//! And three of the constants below arrive at 256× what `rules.xml` writes,
//! because `Constants::init` reads them with `get_fraction(name, 0x100)` while
//! reading their neighbours plain. They are written as ordinary integers, so
//! nothing in the text gives the scale away — only the `>> 8` at the consumer
//! does. See `docs/ECONOMY.md`.
//!
//! # Arithmetic
//!
//! Integers throughout, with the original's truncation. Nothing here needs a
//! fraction: the sixteenths and the 8.8 constants are the original's own way of
//! keeping precision without one.

use crate::tuning::Tuning;
use crate::world::Player;

/// The six basic resources, in the engine's order.
///
/// This is the order of `STARTING_GOODS`, `BASIC_GATHER` and `CITY_GATHER`'s
/// `entry0`..`entry5` slots, and the order the engine's own `TypeIndex`
/// constants sit in. It is *not* the order `resourcerules.xml` lists records
/// in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Resource {
    Food = 0,
    Timber = 1,
    /// `WEALTH` to the engine, `g` in a cost string, "gold" to everybody else.
    Wealth = 2,
    Knowledge = 3,
    Metal = 4,
    Oil = 5,
}

/// How many basic resources there are. Six, everywhere, forever.
pub const RESOURCES: usize = 6;

/// What a bucket holds under the unlimited lobby (`STARTING_RESOURCES == 8`).
/// The original writes it as the masked literal `0x104be`, which is 99,999
/// once the `bucket` XOR comes off.
pub const UNLIMITED_GOODS: i32 = 99_999;

impl Resource {
    pub const ALL: [Resource; RESOURCES] = [
        Resource::Food,
        Resource::Timber,
        Resource::Wealth,
        Resource::Knowledge,
        Resource::Metal,
        Resource::Oil,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Sixteenths per whole resource. Every rate in this module is in these.
///
/// The original multiplies each term by 16 as it goes into the sum and divides
/// by `GATHER_RATE * 16` coming out, so no intermediate term rounds. A
/// market's ten wealth through a twenty percent enhancer is one exact
/// `10 * 16 * 120 / 100`, not two truncations.
pub const RATE_SCALE: i32 = 16;

/// The largest a commerce cap may be, before the `* 16`. A cap of exactly this
/// means uncapped, and the original says so differently in the interface.
pub const CAP_CEILING: i32 = 999;

/// How often the rate may be reassembled when nothing has changed, and the
/// minimum gap that must have passed. Both conditions must hold.
///
/// They do not combine into a 300-frame period, and they do not give 256
/// either: the grid is 256 and the gap is 300, so the first grid point past
/// the gap is **512 frames** — thirty-four seconds.
pub const RATE_REFRESH_GRID: i64 = 256;
/// The minimum frames between two reassemblies. See [`RATE_REFRESH_GRID`].
pub const RATE_REFRESH_MIN_GAP: i64 = 300;
/// The grid used instead while a player's economy is marked dirty.
pub const RATE_REFRESH_DIRTY_GRID: i64 = 8;
/// Frames of stagger per player index, so eight players do not all reassemble
/// their economies on the same tick. Part of the simulation, not an
/// optimisation: it decides *which* frame a change becomes visible on.
pub const PLAYER_STAGGER: i64 = 8;

/// Whether a player reassembles their rates this frame.
///
/// `Leader::calc_gather` returns immediately unless this holds. The dirty flag
/// is `LeaderData`'s `0x2000000`, set by whatever changed the economy — a
/// building finished, a city taken — and cleared by the reassembly.
pub const fn should_recompute(frame: i64, who: Player, stamp: i64, dirty: bool) -> bool {
    let who = who as i64;
    if dirty {
        return frame == 0 || (who + frame) % RATE_REFRESH_DIRTY_GRID == 0;
    }
    frame >= stamp + RATE_REFRESH_MIN_GAP && (frame + who * PLAYER_STAGGER) % RATE_REFRESH_GRID == 0
}

/// Whether a rate was clamped, and how the interface should say so.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OverCap {
    #[default]
    Under,
    /// At the cap, and the cap can still be raised.
    At,
    /// At a cap of [`CAP_CEILING`], which is the original's way of spelling
    /// "no cap" — only knowledge and the Virtual Reality bonus reach it.
    Uncapped,
}

/// The forty-four rare resources, `TypeIndex` `0x06`–`0x31` —
/// `BitMask<44>`'s width, and the bit a good's index takes in
/// `LeaderData::rare` is `good - `[`BASE_RARE`].
pub const RARES: usize = 44;
/// `TypeIndex::BASE_RARE`: the first good that is a rare, and the offset
/// between a good's type index and its bit in the rare masks.
pub const BASE_RARE: usize = 6;
/// `TypeIndex::FISH`.
pub const FISH: usize = 6;
/// `TypeIndex::WHALES` — the rare `Unit::update_speed` reads, and the
/// second of the two `LeaderData::calc_rare` treats as a **fisherman's**
/// rather than a merchant's.
pub const WHALES: usize = 31;
/// `TypeIndex::HORSES` — the rare `TypeData::get_cost@00664090` reads for
/// a Stable or Auto Plant unit: `rare.ptr[1] & 1` (or `rare_conquest`'s),
/// byte 1 bit 0, which is bit `14 - `[`BASE_RARE`]` = 8`;
/// `resourcerules.xml`'s fifteenth `RESOURCE` is Horses (`docs/AI.md` §62).
pub const HORSES: usize = 14;
/// `TypeIndex::RUBBER` — the same arm's second rare, byte 1 bit 1.
pub const RUBBER: usize = 15;
/// `TypeIndex::WINE` — the rare `TypeData::get_cost@00664090`'s **research**
/// arm reads: `testb $0x4, 0x6da4(%ecx)`, else the same at `0x6dcc`
/// (`00664edd`), byte 0 bit 2 of `rare` or of `rare_conquest`, which is bit
/// `8 - `[`BASE_RARE`]` = 2`. Holding it takes `WINE_UNIT_UPGRADES` off a
/// unit's research before the premium (`docs/COSTS.md`, "Researching an
/// upgrade is not building a unit"; `docs/AI.md` §94).
pub const WINE: usize = 8;
/// `TypeIndex::SPICE` — the rare `Caravan::trade_value@0073d9d0` reads:
/// `leaders.list[who] +0x6da4 & 0x40`, else the same at `+0x6dcc`, byte 0
/// bit 6 of `rare` or of `rare_conquest`, which is bit `12 - `[`BASE_RARE`]`
/// = 6`; `resourcerules.xml`'s thirteenth `RESOURCE` is Spice.
pub const SPICE: usize = 12;
/// `TypeIndex::TOBACCO` — the rare `Wall::update_construct_time` reads.
///
/// `0063d560:32` tests `LeaderData +0x6da5 & 0x20` **or** `+0x6dcd &
/// 0x20`, the two rare masks, and divides the construction clock by
/// `(tobacco_building_speed + 100) / 100`. Byte 1 bit 5 of the mask is bit
/// `19 - `[`BASE_RARE`]` = 13`, and the same byte's bit 7 is Furs
/// (`docs/VISION.md` §1's `+0x6da5 & 0x80`), so the two anchors agree; the
/// second field is `rare_conquest`, which this crate keeps unioned into
/// [`Ledger::rare`] already ([`crate::Sim::has_rare`]).
pub const TOBACCO: usize = 19;

/// `TypeIndex::AMBER` — the one rare the **market** reads.
///
/// `LeaderData::calc_market_prices@006dc2a0` tests `rare.ptr[1] & 8`, byte
/// 1 bit 3, which is bit `17 - `[`BASE_RARE`]` = 11`; `resourcerules.xml`'s
/// eighteenth `RESOURCE` is Amber. It widens the gap between the sell and
/// the buy price by `AMBER_MARKET` in the holder's favour, and it is the
/// only rare in that function.
pub const AMBER: usize = 17;

/// `TypeIndex::GEMS` — the one rare the **border** table reads.
///
/// `World::compute_reg_territory@006b0bb0`'s flat-bonus arm is an inlined
/// `LeaderData::has_rare(GEMS)`: `rare.ptr[2] & 0x80 || rare_conquest.ptr[2]
/// & 0x80`, and byte 2 bit 7 is bit `29 - `[`BASE_RARE`]` = 23`. The same
/// bit is the only one `Leader::calc_gather` singles out — when it and it
/// alone changes, the mask's writer calls `Regions::fix_all_borders`
/// (`docs/ATTRITION.md`, "Territory").
pub const GEMS: usize = 29;

/// `TypeIndex::PEACOCKS` — the one rare the **population cap** reads.
///
/// `Leader::calc_pop_cap@006dc490`'s tail, `006dc656`: `testb $0x8,
/// 0x6da6(%esi)`, else the same at `0x6dce` — byte 2 bit 3 of `rare` or of
/// `rare_conquest`, which is bit `25 - `[`BASE_RARE`]` = 19` — and then
/// `cap × (PEACOCKS_POP + 100) / 100`, on every path through the function,
/// the scenario and ignore-cap ones included (`docs/COSTS.md`, the cap;
/// item 1147).
pub const PEACOCKS: usize = 25;

/// What one `resourcerules.xml` record pays whoever stands on it —
/// `GoodTypeData`'s two `(BONUS_TYPE, BONUS_NUM)` pairs, which is the whole
/// of what `LeaderData::calc_rare@006e08d0` reads off the good.
///
/// A pair whose `BONUS_TYPE` is not one of the six resources is skipped;
/// the original's test is `(unsigned)index < 6`, so `none` (`-1`) falls out
/// the same way. The amount is in **whole resources per period**, as the
/// file writes it — the `× 16` into sixteenths is `calc_rare`'s.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GoodType {
    pub bonus: [(Option<Resource>, i32); 2],
}

/// A building that people gather at, as income sees one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Site {
    pub resource: Resource,
    /// How many are working it.
    ///
    /// **An input.** `BuildTypeData::calc_gather` walks the tiles in a radius
    /// counting resource richness and river tiles to decide how many slots a
    /// site offers; that survey is unread, and guessing at it would be worse
    /// than taking the answer from outside. Everything downstream of this
    /// number is specified.
    pub gatherers: i32,
    /// For knowledge only: the university level that selects `SCHOLAR_RATE`.
    /// One-based, as the original's arrays are.
    pub level: i32,
}

impl Site {
    pub const fn new(resource: Resource, gatherers: i32) -> Site {
        Site {
            resource,
            gatherers,
            level: 1,
        }
    }
}

/// A city, as income sees one.
///
/// The enhancer percentages are stored rather than derived because that is how
/// `CityData` stores them: `City::calc_gather` fills the bytes from
/// `GRANARY_BONUS` and friends before anything reads them, and a city with no
/// granary carries a zero rather than a missing level.
///
/// There are **three** of them, not four. `CityData` has a fourth byte at
/// `+0x59` that `CityData::enhancer_amount` would read for oil, and
/// `City::calc_gather` writes it as an unconditional zero every time it runs —
/// so a city never enhances oil, and refineries act at the player level
/// instead ([`Holdings::refineries`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct City {
    /// Percentage bonus to food from a granary, zero if there is none.
    pub granary: i32,
    /// Percentage bonus to timber from a lumber mill.
    pub lumber_mill: i32,
    /// Percentage bonus to metal from a smelter.
    pub smelter: i32,
    /// Buildings in the city, for `BUILDING_TAXES`.
    pub buildings: i32,
    pub market: bool,
    pub temple: bool,
    pub university: bool,
    pub library: bool,
    /// `CityData::trade_val` (+0x52) — what the trade routes ending here
    /// pay, already in sixteenths, and `calc_city_resources`' very first
    /// line (`docs/CARAVAN.md` §7.2).
    pub trade_val: i32,
    pub sites: Vec<Site>,
}

impl City {
    /// Fills the enhancer byte for a resource from the level of the building
    /// that enhances it — what `City::calc_gather` does every time it runs,
    /// before anything reads the byte.
    ///
    /// Levels are one-based. The original reaches these arrays by indexing
    /// `level + 4` off the array that physically precedes each one, which is
    /// the same off-the-end idiom `docs/ATTRITION.md` records for
    /// `ATTRITION_UPGRADE`; here the arrays are separate and the index is
    /// written the way the designers meant it.
    ///
    /// Wealth, knowledge and **oil** have no enhancer and are left alone. The
    /// original gates the three that exist — granary and lumber mill on city
    /// flags, smelter on the metal enhancer actually standing in the city —
    /// and this call is the caller saying the gate passed.
    pub fn set_enhancer(&mut self, t: &Tuning, r: Resource, level: usize) {
        let pick = |a: &[i32; 5]| a[(level.max(1) - 1).min(a.len() - 1)];
        match r {
            Resource::Food => self.granary = pick(&t.granary_bonus),
            Resource::Timber => self.lumber_mill = pick(&t.lumbermill_bonus),
            Resource::Metal => self.smelter = pick(&t.smelter_bonus),
            Resource::Wealth | Resource::Knowledge | Resource::Oil => {}
        }
    }

    /// The enhancer percentage for a resource — `CityData::enhancer_amount`,
    /// which returns zero for the three resources with no city enhancer.
    ///
    /// Oil is one of the three: its byte exists but is written zero, so
    /// `REFINERY_BONUS` is applied once at the player level in [`assemble`]
    /// rather than here. Scaling oil in both places would count refineries
    /// twice.
    pub const fn enhancer(&self, r: Resource) -> i32 {
        match r {
            Resource::Food => self.granary,
            Resource::Timber => self.lumber_mill,
            Resource::Metal => self.smelter,
            Resource::Wealth | Resource::Knowledge | Resource::Oil => 0,
        }
    }
}

/// Everything the rate assembly reads about one player.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Holdings {
    pub cities: Vec<City>,
    /// Refineries anywhere, which scale oil at the player level rather than
    /// per city. `REFINERY_BONUS` is the only enhancer written "per refinery".
    pub refineries: i32,
    /// Owned land tiles, and the map's total. The territory tax is their ratio.
    pub territory: i32,
    pub land_size: i32,
    /// Level of the taxation tech, indexing `TERRITORY_TAXES`.
    pub taxation: usize,
    /// Level of the commerce tech line, indexing `COMMERCE_CAP`.
    pub commerce: usize,
    /// The highest `REPUBLIC_n` bonus held, 1–3, or 0 for none: which
    /// `REPUBLIC_COMMERCE_BONUS` the cap adds (`docs/AI.md` §72).
    pub republic: usize,
    /// `LeaderData::has_wonder`, a bit per wonder (`1 << tech::wonder::*`):
    /// an activated wonder of the type standing in one of the player's
    /// cities. Refreshed every frame, because `calc_resource_caps` runs
    /// every frame and reads it there (`docs/ECONOMY.md` §15).
    pub wonders: u32,
    /// A flat addition to the commerce cap, `LeaderData + 0x918`.
    ///
    /// **Scenario-script only.** `ScenarioFuncSet::set_bonus_cap` is its one
    /// writer in the whole executable; nothing in a skirmish touches it, so it
    /// is zero unless a scenario says otherwise. It is *not* where
    /// `LeaderData::resource_cap_add` (Angkor Wat) puts its bonus — that adds
    /// straight into the cap itself, which is why [`commerce_cap`] adds it
    /// after the percentages rather than before.
    pub bonus_cap: [i32; RESOURCES],
    /// The three nation powers on the commerce cap that the shipped data has,
    /// plus the British one that applies to every slot. **Inputs**, like the
    /// rest of the nation layer: nothing reads the dump's `tribe` yet, so a
    /// traced game leaves all four false — see `docs/ECONOMY.md`.
    pub british: bool,
    pub egyptians: bool,
    pub french: bool,
    pub inca: bool,
    /// Percentage adjustment to income from the difficulty setting.
    ///
    /// **AI leaders only.** A human earns 100% of their capped rate unless the
    /// multiplayer handicap option is on. See [`gather_handicap`].
    pub handicap: i32,
    /// `LeaderData::escrow_rate` (`+0x480`), the percent of the paid rate
    /// that [`pay`] reserves into [`Ledger::escrow`]. Written every frame
    /// from the census, and zero for a human: `do_gather`'s escrow arm is
    /// guarded by `(leader_flags & 0xc) != 4`, the human bit (`docs/AI.md`
    /// §98).
    pub escrow_rate: [i32; RESOURCES],
    /// Whether each good type is available yet. Oil is not, before the
    /// Industrial age, and an unavailable resource takes no part at all: no
    /// rate, no cap, no accrual.
    pub available: [bool; RESOURCES],
    /// Whether the player holds each good's prerequisite — `has_preq`. Only
    /// the cost path reads it, and only for a resource that is *not*
    /// available: it chooses which of the two redirect tables in
    /// `crates/sim/src/cost.rs` charges the price somewhere else.
    pub discovered: [bool; RESOURCES],
    /// Gather buildings that pay outside any city — every oil well and
    /// platform, and every camp or mine with no city link.
    /// `Leader::calc_gather` steps 4 and 5, which take no city enhancer
    /// and no `CITY_GATHER` (`crates/sim/src/holdings.rs`).
    pub outside: Vec<Site>,
    /// **Step 6**, already summed: what the player's idle fishermen and
    /// merchants pay, in sixteenths, each one's `LeaderData::calc_rare`
    /// divided by the crowd sharing its deposit
    /// (`crates/sim/src/holdings.rs`).
    pub rares: [i32; RESOURCES],
    /// `LeaderData::rare_owned` (`+0x6dac`): a bit per rare a unit of this
    /// player's was standing on when step 6 last ran. Cleared and rebuilt
    /// on every recompute, which is why a rare with nobody on it pays
    /// nothing and grants nothing.
    pub rare_owned: u64,
}

impl Holdings {
    /// Holdings with every resource available and nothing in them.
    pub fn new() -> Holdings {
        Holdings {
            available: [true; RESOURCES],
            land_size: 1,
            ..Holdings::default()
        }
    }
}

/// One player's income state, and the only thing here that persists between
/// frames.
///
/// The field names are the original's, from the `LeaderDataEncrypt` type
/// record. In the original every one of them is stored XOR-masked; that is
/// tamper resistance rather than arithmetic and does not survive into here.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ledger {
    /// The stockpile: what can be spent.
    pub bucket: [i32; RESOURCES],
    /// A reservation held back from ordinary spending, fed at `escrow_rate`
    /// percent of income. `crates/sim/src/cost.rs` is what reads it: a payment
    /// may not dip into it, and a payment that needs to abandons the whole
    /// reservation rather than consuming part of it. [`pay`] feeds it at
    /// [`Holdings::escrow_rate`] (`docs/AI.md` §98).
    pub escrow: [i32; RESOURCES],
    /// Fractional carry between frames, in rate-frames.
    pub leftover: [i32; RESOURCES],
    /// The assembled rate, in sixteenths.
    pub rate: [i32; RESOURCES],
    /// The commerce cap, in sixteenths.
    pub cap: [i32; RESOURCES],
    /// The net rate the interface shows — captured after the cap and *before*
    /// the difficulty handicap, so the number on screen is not what arrives.
    pub income: [i32; RESOURCES],
    pub over_cap: [OverCap; RESOURCES],
    /// Lifetime total, for the score screen.
    pub collected: [i32; RESOURCES],
    /// `LeaderData + 0x86c goody_box_resources`: how much a player's goody
    /// boxes have paid, all goods together. A score counter — `Leader::
    /// reset_score` zeroes it beside `goody_box_techs` and `goody_box_units`
    /// — and `crate::goody` is its only writer, as in the original.
    pub goody_box_resources: i32,
    /// `LeaderData + 0x8a4`, the dump's `gather_slots`: how many gatherers
    /// the player's finished gather buildings have room for, per resource.
    /// `Build::activate` adds a building's `gather_max` as it goes active and
    /// nothing ever takes it back — a razed farm leaves its slot behind.
    pub gather_slots: [i32; RESOURCES],
    /// `LeaderData + 0x8d4`, the dump's `gather_slots_high`: the high-water
    /// mark of [`Ledger::gather_slots`]. It is what decides whether a
    /// completed gather building pays its bonus, and rebuilding a razed farm
    /// therefore pays nothing.
    pub gather_slots_high: [i32; RESOURCES],
    /// `LeaderData::rare` (`+0x6d98`): `rare_owned | rare_conquest`, as
    /// `Leader::gather` recomputes it every frame. This is what the rest of
    /// the game asks — `Unit::update_speed`'s whales arm, the pop cap, the
    /// border colours — and it changes only on a recompute frame, because
    /// that is when `rare_owned` is rebuilt.
    ///
    /// `rare_conquest` is Conquer-the-World's and is always empty here.
    pub rare: u64,
    /// Frame of the last reassembly.
    pub gather_stamp: i64,
    /// Whether something changed since the last reassembly.
    pub dirty: bool,
}

impl Ledger {
    /// A ledger holding the game's starting goods.
    pub fn starting(t: &Tuning) -> Ledger {
        Ledger {
            bucket: t.starting_goods,
            dirty: true,
            ..Ledger::default()
        }
    }

    pub const fn get(&self, r: Resource) -> i32 {
        self.bucket[r as usize]
    }
}

/// The per-gatherer rate for a resource, in sixteenths.
///
/// Three constants and one shift. `PEASANT_RATE` and `OIL_RATE` are loaded at
/// 256× and read back with `>> 8`; `SCHOLAR_RATE` is multiplied by 16 before
/// the same shift, which is how it arrives already scaled.
///
/// **This is exact for a mine, a woodcutter's camp and a university, and it is
/// one particular case of a farm or an oil well.** The original's mine and
/// woodcutter branches really do use a land-independent `(PEASANT_RATE >> 8)
/// << 4` = 160, and a university `(SCHOLAR_RATE[level - 1] * 16) >> 8`. Its
/// *flat* branch — farms, oil wells, oil platforms — instead sums the land's
/// `num_make` richness over every tcoord of the building's footprint,
/// doubling river tcoords by `RIVER_RESOURCE_VALUE` and skipping tcoords owned
/// by a non-ally, and only then multiplies by the rate and shifts. 160 is what
/// that comes to for a farm whose sixteen tcoords each make one food; a farm
/// straddling a border yields less. Reproducing it needs the tile survey that
/// [`Site::gatherers`] is an input to stand in for, so it is deferred with it.
pub fn per_gatherer(t: &Tuning, r: Resource, level: i32) -> i32 {
    match r {
        Resource::Knowledge => {
            let i = (level.max(1) as usize - 1).min(t.scholar_rate.len() - 1);
            (t.scholar_rate[i] * RATE_SCALE) >> 8
        }
        Resource::Oil => (t.oil_rate >> 8) * RATE_SCALE,
        _ => (t.peasant_rate >> 8) * RATE_SCALE,
    }
}

/// What one city contributes, in sixteenths — `LeaderData::calc_city_resources`.
///
/// The very first line is the city's `trade_val` (`CityData + 0x52`), which
/// is where **caravan income arrives**: `City::compute_trade` recomputes it
/// when a route starts, ends or first delivers rather than on every
/// recompute, and it is already in sixteenths (`docs/CARAVAN.md` §7.2). The
/// Forbidden City's percentage and the CEO hero's are still missing, for the
/// same reason the rest of the wonder layer is.
pub fn city_rates(t: &Tuning, city: &City) -> [i32; RESOURCES] {
    let mut out = [0; RESOURCES];
    out[Resource::Wealth.index()] += city.trade_val;

    // The gathering buildings. The enhancer scales the *per-gatherer* rate and
    // truncates there, before the multiply by how many are working — so a
    // hundred citizens under a twenty percent granary is a hundred times a
    // truncated twelve, not a truncated twelve hundred.
    for site in &city.sites {
        let i = site.resource.index();
        let mut rate = per_gatherer(t, site.resource, site.level);
        let enhancer = city.enhancer(site.resource) + 100;
        if enhancer != 100 {
            rate = rate * enhancer / 100;
        }
        out[i] += rate * site.gatherers;
    }

    // The city itself. Ships as ten food and ten timber: a city is worth that
    // much per thirty seconds before anybody works in it.
    for r in Resource::ALL {
        out[r.index()] += t.city_gather[r.index()] * RATE_SCALE;
    }

    out[Resource::Wealth.index()] += taxes(t, city) * RATE_SCALE;
    out[Resource::Knowledge.index()] += literacy(t, city) * RATE_SCALE;
    out
}

/// A city's wealth from taxes — `CityData::get_taxes`.
///
/// Three of the four constants ship as zero. As shipped this whole function is
/// "a market is worth ten wealth per thirty seconds"; the other three are read
/// by the original, so they are tuning rather than dead weight, but they do
/// nothing in a stock game.
pub fn taxes(t: &Tuning, city: &City) -> i32 {
    let mut n = t.village_taxes + city.buildings * t.building_taxes;
    if city.market {
        n += t.market_taxes;
    }
    if city.temple {
        n += t.temple_taxes;
    }
    n
}

/// A city's knowledge from literacy — `CityData::get_literacy`.
///
/// A university is worth ten knowledge per thirty seconds. A library on its own
/// is worth nothing: its value is that scholars can live in it.
pub fn literacy(t: &Tuning, city: &City) -> i32 {
    let mut n = t.village_literacy;
    if city.university {
        n += t.university_literacy;
    }
    if city.library {
        n += t.library_literacy;
    }
    n
}

/// Assembles the six rates — `Leader::calc_gather`, in its order.
///
/// The terms this leaves out are the ones `docs/ECONOMY.md` lists as unread or
/// out of scope: the nation and wonder multipliers, and the idle-unit loop —
/// which is where **fishermen and merchants** pay, and with them every owned
/// rare resource, since a rare pays only while a merchant stands on it.
/// Caravans are not in this function at all; they arrive per city, through
/// [`city_rates`].
pub fn assemble(t: &Tuning, h: &Holdings) -> [i32; RESOURCES] {
    let mut out = [0; RESOURCES];

    // A free trickle per resource, which ships as zero for all six.
    for r in Resource::ALL {
        out[r.index()] = t.basic_gather[r.index()] * RATE_SCALE;
    }

    for city in &h.cities {
        let c = city_rates(t, city);
        for r in Resource::ALL {
            out[r.index()] += c[r.index()];
        }
    }
    // Steps 4 and 5: the sites outside every city, at the bare rate.
    for site in &h.outside {
        let i = site.resource.index();
        out[i] += per_gatherer(t, site.resource, site.level) * site.gatherers;
    }

    // Step 6: every idle fisherman and merchant, and with them every rare
    // anyone is standing on. `crates/sim/src/holdings.rs` is the walk; the
    // sum arrives here already in sixteenths and already divided by each
    // deposit's crowd.
    for r in Resource::ALL {
        out[r.index()] += h.rares[r.index()];
    }

    // Refineries scale oil at the player level rather than per city.
    let oil = Resource::Oil.index();
    out[oil] = (h.refineries * t.refinery_bonus + 100) * out[oil] / 100;

    out[Resource::Wealth.index()] += territory_tax(t, h);
    resource_bonuses(t, h.wonders, &mut out);
    out
}

/// `LeaderData::calc_resource_bonuses@006db030`, the last thing
/// `Leader::calc_gather` does to the rates before it clears the dirty
/// flag — its wonder terms, in the listing's order, each truncating
/// (`docs/ECONOMY.md` §15). The Pyramids' `PYRAMIDS_FOOD` is the one a
/// capture has reached: Great Lakes' who=1 reads food 1920 on 17087's
/// reassembly against 1600 without it.
///
/// **Seams**, stated rather than built: the Russian oil term
/// (`has_tribe_bonus(0xd)`, first in the listing), Virtual Reality's
/// `GLOBAL_PROSPERITY` and the Conquer-the-World rate bonuses. No capture
/// holds a Russian player or reaches either of the others.
pub fn resource_bonuses(t: &Tuning, wonders: u32, out: &mut [i32; RESOURCES]) {
    use crate::tech::wonder;
    let has = |w: usize| wonders & (1 << w) != 0;
    let pct = |v: &mut i32, p: i32| *v = (p + 100) * *v / 100;
    if has(wonder::PYRAMIDS) {
        pct(&mut out[Resource::Food.index()], t.pyramids_food);
    }
    if has(wonder::COLOSSUS) {
        pct(&mut out[Resource::Wealth.index()], t.colossus_wealth);
    }
    if has(wonder::HANGING_GARDENS) {
        out[Resource::Knowledge.index()] += t.hanging_gardens_knowledge * RATE_SCALE;
    }
    if has(wonder::ANGKOR_WAT) {
        pct(&mut out[Resource::Metal.index()], t.angkor_metal);
    }
    if has(wonder::TAJ_MAHAL) {
        pct(&mut out[Resource::Wealth.index()], t.taj_wealth);
    }
    if has(wonder::EIFFEL_TOWER) {
        pct(&mut out[Resource::Oil.index()], t.eiffel_oil);
    }
    if has(wonder::TIKAL) {
        pct(&mut out[Resource::Timber.index()], t.tikal_timber);
    }
}

/// `LeaderData::calc_rare@006e08d0` — what one deposit pays the player
/// standing on it, in sixteenths.
///
/// The good gives at most two `(resource, amount)` pairs
/// ([`GoodType`]), each `× 16`. Two percentages then sit on them, and
/// which one reaches which half is the part a reader gets wrong:
///
/// - **`MERCHANTS_BONUS` replaces the 100**, and it reaches a resource
///   only when the unit stands in friendly ground (`ally`) *or* when the
///   good is [`FISH`] or [`WHALES`] **and the resource is not food**. So a
///   fish's wealth half is a merchant's business and its food half is not.
/// - **`FISHERMEN_BONUS` is added**, to the **food slot only**, and only
///   for those same two water goods.
///
/// Both are inert as shipped at level zero — `MERCHANTS_BONUS` entry 0 is
/// 100% and `FISHERMEN_BONUS` entry 0 is 0% — which is why a level-0
/// fisherman on a fish pays exactly `BONUS_NUM × 16` and nothing else.
///
/// **Seam.** `extra` — the Japanese fishing-boat bonus on the two water
/// goods, and the Porcelain Tower and Nubian terms on everything else —
/// is the nation and wonder layer and is always zero here. It is the only
/// term that would scale slots 1–5 without a merchant level.
pub fn calc_rare(
    t: &Tuning,
    good_index: usize,
    good: &GoodType,
    fishermen: usize,
    merchants: usize,
    ally: bool,
) -> [i32; RESOURCES] {
    let mut out = [0; RESOURCES];
    let mut scale = [100; RESOURCES];
    let water = good_index == FISH || good_index == WHALES;
    let merchant_pct = t.merchants_bonus[merchants.min(t.merchants_bonus.len() - 1)];
    for (res, num) in good.bonus {
        let Some(res) = res else { continue };
        let i = res.index();
        out[i] += num * RATE_SCALE;
        if ally || (water && res != Resource::Food) {
            scale[i] = merchant_pct;
        }
    }
    // The fishermen level is read for the two water goods and nowhere
    // else; the `else` arm is where Porcelain and Nubian would go.
    let fish_pct = if water {
        t.fishermen_bonus[fishermen.min(t.fishermen_bonus.len() - 1)]
    } else {
        0
    };
    let extra = 0;
    if fish_pct + extra != 0 || scale[0] > 100 {
        out[0] = (fish_pct + extra + scale[0]) * out[0] / 100;
    }
    for i in 1..RESOURCES {
        if extra != 0 || scale[i] > 100 {
            out[i] = (scale[i] + extra) * out[i] / 100;
        }
    }
    out
}

/// Wealth from owned ground.
///
/// A player holding a fraction *f* of the map's land earns
/// `f * TERRITORY_TAXES[level]` wealth per thirty seconds. This is the one
/// place in the game where territory pays rather than merely hurting whoever
/// stands in it — the economic mirror of `docs/ATTRITION.md`.
///
/// **The British take the rate `(BRITISH_TAXATION + 100) / 100` times**,
/// truncating on the rate before the territory multiplies it
/// (`Leader::calc_gather@006ceee0`, listing `0x6cf64a..0x6cf6b7`; as
/// shipped, doubled). Without it East Indies' British AI earned one tax
/// where the original earns two from its Taxation on tick 17183, 1234
/// wealth against 1380 (`docs/ECONOMY.md` §16). The Conquer-the-World
/// arm beside it (`CTW_MISSIONARIES_BONUS`) is cut with CtW.
pub fn territory_tax(t: &Tuning, h: &Holdings) -> i32 {
    if h.land_size <= 0 {
        return 0;
    }
    let level = h.taxation.min(t.territory_taxes.len() - 1);
    let mut rate = t.territory_taxes[level];
    if h.british {
        rate = (t.british_taxation + 100) * rate / 100;
    }
    h.territory * rate * RATE_SCALE / h.land_size
}

/// The commerce cap for one resource, in sixteenths — `Leader::calc_resource_caps`.
///
/// Knowledge is exempt: its cap is [`CAP_CEILING`] regardless of anything. That
/// single asymmetry shapes the whole game, because it is why scholars scale and
/// farmers do not.
///
/// The **nation** terms are here, in the original's order: the British on
/// every slot first, then the one per-resource power — Egyptian food, French
/// timber, Inca wealth. Each is a percentage of the running value and each
/// truncates on its own, so a British Egyptian's food cap is
/// `70 * 125 / 100 * 110 / 100` and not `70 * 137 / 100`. The wonder and
/// rare terms are additions on the same value and are not here; they belong
/// with the rest of the wonder layer.
///
/// The **republic** term is: the highest `REPUBLIC_n` held adds its
/// `REPUBLIC_COMMERCE_BONUS`, once and not summed, after the wonders and
/// before `bonus_cap` (`calc_resource_caps@006ce900`). run221 measures it:
/// East Indies' British AI took Republic on 15782, and its cap is 3792
/// against 2992, fifty more on every capped good (`docs/AI.md` §72).
///
/// The British 25% is what run40 measures: the AI's `resource_cap` is 1392
/// on every frame of the window against the human's 1120, and
/// `70 * 125 / 100` is 87 — the half is lost here, before the `* 16`.
pub fn commerce_cap(t: &Tuning, h: &Holdings, r: Resource) -> i32 {
    if r == Resource::Knowledge {
        return CAP_CEILING * RATE_SCALE;
    }
    let level = h.commerce.min(t.commerce_cap.len() - 1);
    let mut cap = t.commerce_cap[level];
    if h.british {
        cap = cap * (100 + t.british_commerce) / 100;
    }
    let nation = match r {
        Resource::Food if h.egyptians => t.egyptian_food_commerce,
        Resource::Timber if h.french => t.french_timber_commerce,
        Resource::Wealth if h.inca => t.inca_wealth_cap,
        _ => 0,
    };
    cap = cap * (100 + nation) / 100;
    // The wonders' flat additions (`calc_resource_caps@006ce900`'s wonder
    // arm, `docs/ECONOMY.md` §15). Diamonds, which comes before them, is
    // not built.
    cap += wonder_commerce(t, h.wonders, r);
    if let Some(&bonus) = h
        .republic
        .checked_sub(1)
        .and_then(|i| t.republic_commerce_bonus.get(i))
    {
        cap += bonus;
    }
    let cap = cap + h.bonus_cap[r.index()];
    cap.clamp(0, CAP_CEILING) * RATE_SCALE
}

/// What the wonders add to one good's commerce cap, in whole units —
/// `calc_resource_caps@006ce900` between Diamonds and the republic term.
///
/// The slots each wonder reaches are the listing's branches: the Pyramids
/// on food and wealth (`iVar5 == 0 || iVar5 == 2`), the Colossus on timber
/// and wealth, the Taj Mahal on wealth; then oil takes the Eiffel Tower
/// and the Kremlin, and every other slot below six except wealth takes the
/// Kremlin, with Tikal on timber and Angkor Wat on metal
/// (`resource_cap_add`, the same addition). Knowledge never reaches the
/// arm. All are flat, so their order does not truncate.
pub fn wonder_commerce(t: &Tuning, wonders: u32, r: Resource) -> i32 {
    use crate::tech::wonder;
    let has = |w: usize| wonders & (1 << w) != 0;
    let mut add = 0;
    if matches!(r, Resource::Food | Resource::Wealth) && has(wonder::PYRAMIDS) {
        add += t.pyramids_commerce;
    }
    if matches!(r, Resource::Timber | Resource::Wealth) && has(wonder::COLOSSUS) {
        add += t.colossus_commerce;
    }
    match r {
        Resource::Wealth => {
            if has(wonder::TAJ_MAHAL) {
                add += t.taj_wealth_commerce;
            }
        }
        Resource::Oil => {
            if has(wonder::EIFFEL_TOWER) {
                add += t.eiffel_oil_commerce;
            }
            if has(wonder::KREMLIN) {
                add += t.kremlin_commerce;
            }
        }
        Resource::Food | Resource::Timber | Resource::Metal => {
            if has(wonder::KREMLIN) {
                add += t.kremlin_commerce;
            }
            if r == Resource::Timber && has(wonder::TIKAL) {
                add += t.tikal_timber_commerce;
            }
            if r == Resource::Metal && has(wonder::ANGKOR_WAT) {
                add += t.angkor_metal_commerce;
            }
        }
        Resource::Knowledge => {}
    }
    add
}

/// Every commerce cap.
pub fn caps(t: &Tuning, h: &Holdings) -> [i32; RESOURCES] {
    let mut out = [0; RESOURCES];
    for r in Resource::ALL {
        out[r.index()] = commerce_cap(t, h, r);
    }
    out
}

/// What finishing a gather building pays its owner, per resource —
/// `Build::activate`'s tail, through `Build::do_bonus`.
///
/// The tail runs for a gather building (`build_flags & 0x40`) whatever else
/// it is: the building's `gather_max` goes into [`Ledger::gather_slots`] for
/// the resource it gathers, and **only the part of that which is past the
/// high-water mark is paid for**. So the first farm of a game pays, a farm
/// rebuilt where one was razed does not, and a woodcutter's camp on a
/// richer patch pays only for the slots the last one did not have.
///
/// Two of the five are flat and three are per new slot, which is not a
/// symmetry the constants' names give away: `FOOD_BONUS_FOR_FARM` and
/// `KNOWLEDGE_BONUS_FOR_UNIVERSITY` and `OIL_BONUS_FOR_WELL` are paid once
/// however many slots arrived, and `TIMBER_BONUS_PER_WOOD_SLOT` and
/// `METAL_BONUS_PER_MINE_SLOT` multiply. Wealth has no case at all: the
/// original's switch falls through for it, as it does for a resource no
/// gather building gathers.
pub fn completion_bonus(t: &Tuning, r: Resource, new_slots: i32) -> i32 {
    match r {
        Resource::Food => t.food_bonus_for_farm,
        Resource::Timber => t.timber_bonus_per_wood_slot * new_slots,
        Resource::Knowledge => t.knowledge_bonus_for_university,
        Resource::Metal => t.metal_bonus_per_mine_slot * new_slots,
        Resource::Oil => t.oil_bonus_for_well,
        Resource::Wealth => 0,
    }
}

/// The difficulty setting's percentage adjustment to income —
/// `LeaderData::get_gather_handicap`.
///
/// No constant stands behind these six numbers; they are literals in the
/// function. They are also the only place the difficulty setting touches the
/// economy.
///
/// **This table is for AI leaders.** `get_gather_handicap` returns zero for a
/// human before it reaches the table, unless the multiplayer handicap option
/// is on, in which case a human takes their own per-player handicap from a
/// different table. So "hard" does not mean the player earns less; it means
/// every AI earns 25% or 50% more.
pub const fn gather_handicap(difficulty: u8) -> i32 {
    match difficulty {
        0 => -35,
        1 => -15,
        2 => -7,
        4 => 25,
        5 => 50,
        _ => 0,
    }
}

/// Pays one frame of income into the stockpile — `Leader::do_gather`.
///
/// `ledger.rate` and `ledger.cap` must already hold this frame's values.
pub fn pay(t: &Tuning, ledger: &mut Ledger, h: &Holdings, frame: i64) {
    let denom = t.gather_rate * RATE_SCALE;
    for r in Resource::ALL {
        let i = r.index();
        if !h.available[i] {
            continue;
        }

        // `support` would be subtracted here. It is always zero: the original's
        // `Leader::calc_support` writes six zeros and returns, and nothing else
        // fills the array. There is no unit upkeep in Rise of Nations.
        let mut rate = ledger.rate[i];

        // A negative rate is displayed and then abandoned — it never takes
        // anything out of the pile.
        if rate < 0 {
            ledger.income[i] = rate;
            ledger.over_cap[i] = OverCap::Under;
            continue;
        }

        let cap = ledger.cap[i];
        if rate > cap {
            ledger.over_cap[i] = if cap > CAP_CEILING * RATE_SCALE - RATE_SCALE {
                OverCap::Uncapped
            } else {
                OverCap::At
            };
            rate = cap;
        } else {
            ledger.over_cap[i] = OverCap::Under;
        }

        // Captured before the handicap: the number on screen is the capped
        // rate, not the rate that arrives. On the hardest setting the two
        // differ by a quarter.
        ledger.income[i] = rate;

        if h.handicap != 0 {
            rate = (h.handicap + 100) * rate / 100;
        }

        // The accumulator. Written the original's way — a divide, a remainder,
        // and a `while` that can only run once — rather than as the equivalent
        // `leftover += rate; whole = leftover / denom`, because the two agree
        // only while `rate` is non-negative and the guard above is the only
        // thing that makes it so.
        let mut whole = rate / denom;
        ledger.leftover[i] += rate % denom;
        while ledger.leftover[i] >= denom {
            whole += 1;
            ledger.leftover[i] -= denom;
        }

        ledger.bucket[i] += whole;
        ledger.collected[i] += whole;

        // The escrow, `do_gather`'s tail (`006ce450`, `docs/ECONOMY.md`,
        // "Escrow"): `escrow_rate` percent of the same paid rate, over
        // `GATHER_RATE × 1600`. It is not an accumulator: the remainder is
        // thrown away and bought back one frame in `n`.
        let e = h.escrow_rate[i];
        if e != 0 {
            let d = t.gather_rate * 1600;
            let v = e.wrapping_mul(rate);
            let mut q = v / d;
            let r = v % d;
            if r != 0 {
                let n = ((d + r / 2) / r).max(2);
                if frame % i64::from(n) == 0 {
                    q += 1;
                }
            }
            ledger.escrow[i] += q;
        }
    }
}

/// One player's whole economy for one frame — `Leader::gather`.
///
/// The order is the original's: reassemble the rates if the cadence says so,
/// recompute the caps unconditionally, then pay.
pub fn process(t: &Tuning, ledger: &mut Ledger, h: &Holdings, who: Player, frame: i64) {
    if should_recompute(frame, who, ledger.gather_stamp, ledger.dirty) {
        ledger.rate = assemble(t, h);
        ledger.gather_stamp = frame;
        ledger.dirty = false;
    }
    ledger.cap = caps(t, h);
    pay(t, ledger, h, frame);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn farm_city(gatherers: i32) -> City {
        City {
            sites: vec![Site::new(Resource::Food, gatherers)],
            ..City::default()
        }
    }

    #[test]
    fn a_rate_pays_out_once_per_gather_rate_frames() {
        // The whole point of the representation: a rate of R whole resources
        // delivers exactly R of them every GATHER_RATE frames, with no drift.
        let t = Tuning::RON;
        let mut h = Holdings::new();
        h.cities.push(farm_city(1));

        let mut l = Ledger {
            dirty: true,
            ..Ledger::default()
        };
        for frame in 0..t.gather_rate as i64 {
            process(&t, &mut l, &h, 0, frame);
        }

        // One citizen on a farm is PEASANT_RATE, and the city itself is
        // CITY_GATHER.
        let expected = (t.peasant_rate >> 8) + t.city_gather[0];
        assert_eq!(l.bucket[Resource::Food.index()], expected);
        assert_eq!(expected, 20);
    }

    #[test]
    fn peasant_rate_is_ten_despite_being_written_as_ten() {
        // The trap: PEASANT_RATE reads "10 resources" and is loaded as 2560.
        // Reading the digits and skipping the shift would make a farmer worth
        // 2560 food per thirty seconds — a hundred and fifty times too much.
        let t = Tuning::RON;
        assert_eq!(t.peasant_rate, 2560);
        assert_eq!(per_gatherer(&t, Resource::Food, 1), 10 * RATE_SCALE);
        assert_eq!(per_gatherer(&t, Resource::Oil, 1), 35 * RATE_SCALE);
        assert_eq!(per_gatherer(&t, Resource::Knowledge, 1), 5 * RATE_SCALE);
        assert_eq!(per_gatherer(&t, Resource::Knowledge, 6), 25 * RATE_SCALE);
    }

    #[test]
    fn the_enhancer_truncates_per_gatherer_not_per_city() {
        // GRANARY_BONUS level 1 is 20%. A per-gatherer rate of 160 sixteenths
        // scaled by 120/100 is 192 exactly, so pick a rate where it does not
        // divide: a scholar at university level 1 is 80, and 80*120/100 = 96.
        let t = Tuning::RON;
        let city = City {
            granary: 25,
            sites: vec![Site::new(Resource::Food, 3)],
            ..City::default()
        };
        let rates = city_rates(&t, &city);
        // 160 * 125 / 100 = 200, times three gatherers.
        assert_eq!(
            rates[Resource::Food.index()],
            200 * 3 + t.city_gather[0] * RATE_SCALE
        );
    }

    #[test]
    fn a_market_is_ten_wealth_and_a_university_ten_knowledge() {
        let t = Tuning::RON;
        let city = City {
            buildings: 9,
            market: true,
            temple: true,
            university: true,
            library: true,
            ..City::default()
        };
        // Every other constant in both lines ships as zero.
        assert_eq!(taxes(&t, &city), 10);
        assert_eq!(literacy(&t, &city), 10);
    }

    #[test]
    fn the_lazy_refresh_period_is_512_frames_not_256() {
        // The grid is 256 and the minimum gap is 300, so the first grid point
        // past the gap is 512. This is the kind of thing that reads as a bug
        // in a diff five thousand frames later.
        let stamp = 0;
        let hits: Vec<i64> = (1..=1200)
            .filter(|&f| should_recompute(f, 0, stamp, false))
            .collect();
        assert_eq!(hits, vec![512, 768, 1024]);
        // ...and only the first is real, because each reassembly moves the
        // stamp forward.
        let mut stamp = 0;
        let mut real = Vec::new();
        for f in 1..=1200 {
            if should_recompute(f, 0, stamp, false) {
                real.push(f);
                stamp = f;
            }
        }
        assert_eq!(real, vec![512, 1024]);
    }

    #[test]
    fn the_refresh_is_staggered_by_player() {
        // Eight players do not all reassemble on the same tick.
        let at = |who: Player| {
            (1..=2000)
                .find(|&f| should_recompute(f, who, 0, false))
                .unwrap()
        };
        assert_eq!(at(0), 512);
        assert_eq!(at(1), 504);
        assert_eq!(at(2), 496);
        assert_eq!(at(7), 456);
    }

    #[test]
    fn a_dirty_economy_refreshes_within_eight_frames() {
        // Which is why building a farm moves the income display at once.
        for who in 0..8u8 {
            let first = (1..=16)
                .find(|&f| should_recompute(f, who, 0, true))
                .unwrap();
            assert!(first <= 8, "player {who} waited {first} frames");
        }
        assert!(
            should_recompute(0, 3, 0, true),
            "frame zero is unconditional"
        );
    }

    #[test]
    fn the_commerce_cap_bites_and_knowledge_escapes_it() {
        let t = Tuning::RON;
        let mut h = Holdings::new();
        // Enough farmers to blow past a level-zero cap of seventy, and enough
        // scholars to blow past it too.
        h.cities.push(farm_city(50));
        h.cities.push(City {
            sites: vec![Site {
                resource: Resource::Knowledge,
                gatherers: 20,
                level: 6,
            }],
            ..City::default()
        });

        let mut l = Ledger {
            dirty: true,
            ..Ledger::default()
        };
        process(&t, &mut l, &h, 0, 0);

        let food = Resource::Food.index();
        assert!(l.rate[food] > l.cap[food]);
        assert_eq!(l.over_cap[food], OverCap::At);
        assert_eq!(l.income[food], 70 * RATE_SCALE);

        let know = Resource::Knowledge.index();
        assert_eq!(l.over_cap[know], OverCap::Under);
        assert_eq!(l.income[know], 25 * 20 * RATE_SCALE);

        // And a capped economy really does earn the cap and no more.
        for frame in 1..t.gather_rate as i64 {
            process(&t, &mut l, &h, 0, frame);
        }
        assert_eq!(l.bucket[food], 70);
        assert_eq!(l.bucket[know], 25 * 20);
    }

    #[test]
    fn the_knowledge_exemption_is_a_ceiling_of_999_not_infinity() {
        // "Exempt from the commerce cap" is exactly what the original means and
        // no more: knowledge skips COMMERCE_CAP and takes 999, which is still a
        // number. Fifty scholars at the last university level would earn 1250,
        // and earn 999.
        let t = Tuning::RON;
        let mut h = Holdings::new();
        h.cities.push(City {
            sites: vec![Site {
                resource: Resource::Knowledge,
                gatherers: 50,
                level: 6,
            }],
            ..City::default()
        });

        let mut l = Ledger {
            dirty: true,
            ..Ledger::default()
        };
        for frame in 0..t.gather_rate as i64 {
            process(&t, &mut l, &h, 0, frame);
        }
        let know = Resource::Knowledge.index();
        assert_eq!(l.rate[know], 1250 * RATE_SCALE);
        assert_eq!(l.over_cap[know], OverCap::Uncapped);
        assert_eq!(l.bucket[know], CAP_CEILING);
    }

    /// run226's value diff (`docs/ECONOMY.md` §15): Great Lakes' British
    /// AI at commerce level 3 finishes the Pyramids on sim-frame 17084, and
    /// its food and wealth caps read 4800 on 17085 — `200 * 125 / 100` is
    /// 250, plus fifty, times sixteen — where timber, metal and oil stay at
    /// 4000 and knowledge at 999.
    #[test]
    fn the_pyramids_add_fifty_to_food_and_wealth_only() {
        let t = Tuning::RON;
        let mut h = Holdings::new();
        h.british = true;
        h.commerce = 3;
        assert_eq!(commerce_cap(&t, &h, Resource::Food), 4000);
        h.wonders = 1 << crate::tech::wonder::PYRAMIDS;
        assert_eq!(commerce_cap(&t, &h, Resource::Food), 4800);
        assert_eq!(commerce_cap(&t, &h, Resource::Wealth), 4800);
        for r in [Resource::Timber, Resource::Metal, Resource::Oil] {
            assert_eq!(commerce_cap(&t, &h, r), 4000, "{r:?}");
        }
        assert_eq!(
            commerce_cap(&t, &h, Resource::Knowledge),
            CAP_CEILING * RATE_SCALE
        );
        // The Kremlin reaches every capped good but wealth.
        h.wonders = 1 << crate::tech::wonder::KREMLIN;
        assert_eq!(commerce_cap(&t, &h, Resource::Wealth), 4000);
        assert_eq!(commerce_cap(&t, &h, Resource::Oil), (250 + 200) * 16);
    }

    /// run226's second value diff (`docs/ECONOMY.md` §15): who=1's first
    /// reassembly after the Pyramids, sim-frame 17087, reads food income
    /// 1920 where it read 1600 — the twenty percent, on food alone.
    #[test]
    fn the_pyramids_pay_a_fifth_more_food() {
        let t = Tuning::RON;
        let mut out = [1600, 2560, 1024, 1840, 1120, 0];
        resource_bonuses(&t, 0, &mut out);
        assert_eq!(out, [1600, 2560, 1024, 1840, 1120, 0]);
        resource_bonuses(&t, 1 << crate::tech::wonder::PYRAMIDS, &mut out);
        assert_eq!(out, [1920, 2560, 1024, 1840, 1120, 0]);
    }

    /// run221's value diff (`docs/AI.md` §72): the British AI at commerce
    /// level 2 holds Republic, and its cap is 3792 — `150 * 125 / 100` is
    /// 187, plus fifty, times sixteen — where the dump read 2992 before it.
    /// Knowledge takes no term.
    #[test]
    fn a_republic_adds_fifty_to_every_capped_good() {
        let t = Tuning::RON;
        let mut h = Holdings::new();
        h.british = true;
        h.commerce = 2;
        assert_eq!(commerce_cap(&t, &h, Resource::Food), 2992);
        for tier in 1..=3 {
            h.republic = tier;
            assert_eq!(commerce_cap(&t, &h, Resource::Food), 3792);
            assert_eq!(commerce_cap(&t, &h, Resource::Metal), 3792);
            assert_eq!(
                commerce_cap(&t, &h, Resource::Knowledge),
                CAP_CEILING * RATE_SCALE
            );
        }
    }

    #[test]
    fn the_last_commerce_tech_is_worth_seven_times_the_first() {
        let t = Tuning::RON;
        let mut h = Holdings::new();
        h.cities.push(farm_city(100));
        let at = |level: usize| {
            let mut h = h.clone();
            h.commerce = level;
            commerce_cap(&t, &h, Resource::Food) / RATE_SCALE
        };
        assert_eq!(at(0), 70);
        assert_eq!(at(7), 500);
        // Past the end of the array clamps rather than reading off it.
        assert_eq!(at(99), 500);
    }

    #[test]
    fn an_unavailable_resource_takes_no_part() {
        // Oil before the Industrial age: the rate is assembled and the cap is
        // computed, but nothing accrues.
        let t = Tuning::RON;
        let mut h = Holdings::new();
        h.available[Resource::Oil.index()] = false;
        h.cities.push(City {
            sites: vec![Site::new(Resource::Oil, 4)],
            ..City::default()
        });

        let mut l = Ledger {
            dirty: true,
            ..Ledger::default()
        };
        for frame in 0..600 {
            process(&t, &mut l, &h, 0, frame);
        }
        assert!(l.rate[Resource::Oil.index()] > 0);
        assert_eq!(l.bucket[Resource::Oil.index()], 0);
        assert_eq!(l.leftover[Resource::Oil.index()], 0);
    }

    #[test]
    fn territory_pays_a_share_of_the_taxation_rate() {
        let t = Tuning::RON;
        let mut h = Holdings::new();
        h.land_size = 1000;
        h.territory = 200;
        h.taxation = 2; // 100%

        // A fifth of the map at full taxation is twenty wealth per thirty
        // seconds — two markets' worth, from borders alone.
        assert_eq!(territory_tax(&t, &h), 20 * RATE_SCALE);

        h.taxation = 4; // 300%
        assert_eq!(territory_tax(&t, &h), 60 * RATE_SCALE);
        h.taxation = 0;
        assert_eq!(territory_tax(&t, &h), 0);
    }

    /// **The British double it** (`docs/ECONOMY.md` §16): East Indies'
    /// British AI on tick 17183, Taxation held, 305 of 1669 tiles — one
    /// tax is 146 sixteenths and the original's rate rose by 292.
    #[test]
    fn the_british_take_the_territory_tax_twice() {
        let t = Tuning::RON;
        let mut h = Holdings::new();
        h.land_size = 1669;
        h.territory = 305;
        h.taxation = 1; // 50%
        assert_eq!(territory_tax(&t, &h), 146);
        h.british = true;
        assert_eq!(territory_tax(&t, &h), 292);
    }

    #[test]
    fn refineries_scale_oil_at_the_player_level() {
        let t = Tuning::RON;
        let mut h = Holdings::new();
        h.cities.push(City {
            sites: vec![Site::new(Resource::Oil, 2)],
            ..City::default()
        });
        let bare = assemble(&t, &h)[Resource::Oil.index()];
        h.refineries = 3;
        let with = assemble(&t, &h)[Resource::Oil.index()];
        assert_eq!(with, (3 * 33 + 100) * bare / 100);

        // And only at the player level. `City::calc_gather` writes the oil
        // enhancer byte as an unconditional zero, so a city can never scale
        // oil as well — which would count the same refineries twice.
        let mut city = City::default();
        for r in Resource::ALL {
            city.set_enhancer(&t, r, 5);
        }
        assert_eq!(city.enhancer(Resource::Oil), 0);
        assert_eq!(city.enhancer(Resource::Food), 250);
    }

    #[test]
    fn the_handicap_changes_what_arrives_but_not_what_is_shown() {
        let t = Tuning::RON;
        let mut h = Holdings::new();
        h.cities.push(farm_city(2));
        // An AI on the hardest setting. A human would carry a handicap of
        // zero here whatever the difficulty — see `gather_handicap`.
        h.handicap = gather_handicap(5); // +50%

        let mut l = Ledger {
            dirty: true,
            ..Ledger::default()
        };
        for frame in 0..t.gather_rate as i64 {
            process(&t, &mut l, &h, 0, frame);
        }
        let shown = l.income[Resource::Food.index()] / RATE_SCALE;
        assert_eq!(shown, 30);
        assert_eq!(l.bucket[Resource::Food.index()], 45);
    }

    #[test]
    fn the_accumulator_does_not_drift_over_a_long_game() {
        // Fifteen minutes at an awkward rate. The remainder carry is the only
        // thing standing between this and a slow, invisible divergence.
        let t = Tuning::RON;
        let mut h = Holdings::new();
        h.cities.push(farm_city(4));
        h.cities[0].set_enhancer(&t, Resource::Food, 1);

        let mut l = Ledger {
            dirty: true,
            ..Ledger::default()
        };
        let frames = 15 * 60 * crate::FRAMES_PER_SECOND as i64;
        for frame in 0..frames {
            process(&t, &mut l, &h, 0, frame);
        }
        // Four farmers under a level-one granary is 928 sixteenths, which
        // divides neither the period nor a frame — so every frame carries a
        // remainder and the carry is the only thing holding the total exact.
        let rate = l.rate[Resource::Food.index()];
        assert_eq!(rate, 928);
        assert!(rate <= l.cap[Resource::Food.index()], "not the cap's test");
        let expected = (rate as i64 * frames) / (t.gather_rate as i64 * RATE_SCALE as i64);
        assert_eq!(l.bucket[Resource::Food.index()] as i64, expected);
    }

    /// `do_gather`'s escrow (`docs/AI.md` §98), on run357's who=1: food
    /// income 2240 at the Toughest handicap pays 3360 a frame, so escrow
    /// takes `40 × 3360 / 720000` = 0 remainder 134400 and rounds up one
    /// frame in `(720000 + 67200) / 134400` = 5; timber's 1280 pays 1920
    /// and rounds up one in 9. The dump's escrow climbs on exactly those
    /// frames (blocks 5606, 5611, … and 5608, 5617, …), and it is **not**
    /// an accumulator: nothing carries between frames. A human escrows
    /// nothing.
    #[test]
    fn escrow_takes_its_rate_of_the_paid_income_one_frame_in_n() {
        let t = Tuning::RON;
        let mut h = Holdings::new();
        h.available = [true; RESOURCES];
        h.handicap = gather_handicap(5);
        h.escrow_rate = [40; RESOURCES];
        let mut l = Ledger {
            cap: [999 * RATE_SCALE; RESOURCES],
            rate: [2240, 1280, 0, 0, 0, 0],
            ..Ledger::default()
        };
        let mut food = Vec::new();
        let mut timber = Vec::new();
        for frame in 5600..5620 {
            let before = l.escrow;
            pay(&t, &mut l, &h, frame);
            if l.escrow[0] != before[0] {
                food.push(frame);
            }
            if l.escrow[1] != before[1] {
                timber.push(frame);
            }
        }
        assert_eq!(food, [5600, 5605, 5610, 5615]);
        assert_eq!(timber, [5607, 5616]);
        h.escrow_rate = [0; RESOURCES];
        let kept = l.escrow;
        pay(&t, &mut l, &h, 5620);
        assert_eq!(l.escrow, kept, "a zero rate, a human's, escrows nothing");
    }

    #[test]
    fn a_negative_rate_is_shown_and_never_charged() {
        let t = Tuning::RON;
        let h = Holdings::new();
        let mut l = Ledger {
            bucket: [100; RESOURCES],
            cap: [999 * RATE_SCALE; RESOURCES],
            rate: [-160; RESOURCES],
            ..Ledger::default()
        };
        for _ in 0..1000 {
            pay(&t, &mut l, &h, 0);
        }
        assert_eq!(l.bucket, [100; RESOURCES]);
        assert_eq!(l.income, [-160; RESOURCES]);
    }
    #[test]
    fn a_finished_gather_building_pays_only_for_slots_nobody_has_held() {
        // `Build::do_bonus`'s table, and the split the constants' names hide:
        // a farm, a university and an oil well pay flat, a camp and a mine
        // pay per new slot.
        let t = Tuning::RON;
        assert_eq!(completion_bonus(&t, Resource::Food, 1), 20);
        assert_eq!(completion_bonus(&t, Resource::Food, 4), 20, "flat");
        assert_eq!(completion_bonus(&t, Resource::Knowledge, 7), 25, "flat");
        assert_eq!(completion_bonus(&t, Resource::Oil, 1), 50, "flat");
        assert_eq!(completion_bonus(&t, Resource::Timber, 5), 25, "per slot");
        assert_eq!(completion_bonus(&t, Resource::Metal, 3), 15, "per slot");
        // Wealth has no case at all: no gather building gathers it, and the
        // original's switch falls through.
        assert_eq!(completion_bonus(&t, Resource::Wealth, 9), 0);
    }

    #[test]
    fn the_british_commerce_cap_is_the_one_run40_measures() {
        // `COMMERCE_CAP[0]` is seventy; the British take `70 × 125 / 100`,
        // and the truncation is *before* the `<< 4`, which is why the dump
        // prints 1392 and not 1400.
        let t = Tuning::RON;
        let mut h = Holdings::new();
        assert_eq!(commerce_cap(&t, &h, Resource::Food), 70 * RATE_SCALE);
        h.british = true;
        assert_eq!(commerce_cap(&t, &h, Resource::Food), 1392);
        assert_eq!(commerce_cap(&t, &h, Resource::Wealth), 1392);
        // Knowledge skips the body entirely, British or not.
        assert_eq!(
            commerce_cap(&t, &h, Resource::Knowledge),
            CAP_CEILING * RATE_SCALE
        );
        // The per-resource powers stack on the British one, each truncating
        // on its own: `70 × 125 / 100 = 87`, then `87 × 110 / 100 = 95`.
        h.egyptians = true;
        assert_eq!(commerce_cap(&t, &h, Resource::Food), 95 * RATE_SCALE);
        assert_eq!(
            commerce_cap(&t, &h, Resource::Timber),
            87 * RATE_SCALE,
            "food only"
        );
    }
}
