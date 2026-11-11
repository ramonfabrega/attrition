//! **The leaders line** (`CheckSums::check_leaders@009375a0`,
//! `docs/CHECKSUM.md` §7 and §20): every slot of the `leaders` array, 8,
//! through `LeaderData::walk_data@006d6750`. Each slot hands over its two
//! flag words; a slot with `leader_flags & 1` is walked whole —
//!
//! 1. `[+0x8, +0x692a)`, `who`..`ally_mask`, one range ([`LEADER_FIELDS`],
//!    276 leaves and two uncovered 2-byte runs);
//! 2. `dip[0..8]`, eight `Diplomacy` records ([`DIPLOMACY_FIELDS`]);
//! 3. `pers`, the `Personality`'s 24 ints ([`PERSONALITY_FIELDS`]);
//! 4. six `BitMask`s, each its `bits` and `size` then `size` bytes;
//! 5. `sites` and `make_list` (`Array<T>::walk_data`: the length, and when
//!    it is not 0 the size, `increment`, the flags `& 0xbf` and each
//!    element), then `mil_trainers`, `new_rares` and `oil_patches`
//!    (`SimpleArray<int>::walk_data`: the same header, the elements in one
//!    range);
//! 6. `prod_script` (`String::walk_data@00a1b2d0`): its length, then its
//!    characters, UTF-16;
//! 7. the three rare masks;
//! 8. `LeaderDataEncrypt::walk_data@006d9900`: 62 plain words — per good,
//!    `bucket`, `leftover`, `resource_cap`, `over_cap`, `resources`,
//!    `support`, `income`, `rate` and `bonus`; then `resource_cap[6]`, the
//!    four `epoch` words, `ages`, `epochs` and `discovered`.
//!
//! Each range is built from [`Leaf`]s, and a leaf this crate holds no
//! value for is `None` ([`Sim::leader_image`]): the census of item 4121,
//! and the list a held-out line names.

use super::{Line, Range};
use crate::Player;
use crate::Sim;

/// **`LeaderData`'s walked `[+0x8, +0x692a)`** (`types.txt`, `LeaderData`,
/// size 0x6ee4): `(name, offset, element width, count)`, 276 leaves. The
/// bytes no leaf covers, `+0x606e..+0x6070` and `+0x6922..+0x6924`, are
/// padding.
pub const LEADER_FIELDS: &[(&str, usize, usize, usize)] = &[
    ("who", 0x8, 4, 1),
    ("tribe", 0xc, 4, 1),
    ("defeated_by", 0x10, 4, 1),
    ("gov", 0x14, 4, 1),
    ("score", 0x18, 4, 1),
    ("score_explored", 0x1c, 4, 1),
    ("score_territory", 0x20, 4, 1),
    ("score_units", 0x24, 4, 1),
    ("score_units_2", 0x28, 4, 1),
    ("score_buildings", 0x2c, 4, 1),
    ("score_economy", 0x30, 4, 1),
    ("score_pop", 0x34, 4, 1),
    ("score_unit_upgrades", 0x38, 4, 1),
    ("score_research", 0x3c, 4, 1),
    ("score_wonders", 0x40, 4, 1),
    ("score_combat", 0x44, 4, 1),
    ("cannon_time", 0x48, 4, 1),
    ("handicap", 0x4c, 4, 1),
    ("multi_diff", 0x50, 4, 1),
    ("chat_status", 0x54, 4, 8),
    ("diplos", 0x74, 4, 8),
    ("treaties", 0x94, 4, 8),
    ("agendas", 0xb4, 4, 8),
    ("good_deeds", 0xd4, 4, 8),
    ("attack_stamp", 0xf4, 4, 8),
    ("raid_stamp", 0x114, 4, 8),
    ("capital_stamp", 0x134, 4, 8),
    ("ally_stamp", 0x154, 4, 8),
    ("tribute_stamp", 0x174, 4, 8),
    ("gift_stamp", 0x194, 4, 8),
    ("hire_stamp", 0x1b4, 4, 8),
    ("hire_who", 0x1d4, 4, 8),
    ("attrition_stamp", 0x1f4, 4, 1),
    ("attrition_stamp2", 0x1f8, 4, 1),
    ("attrition_stamp3", 0x1fc, 4, 1),
    ("ctw_hero_stamp", 0x200, 4, 1),
    ("ctw_hero_retreat_stamp", 0x204, 4, 1),
    ("repair_stamp", 0x208, 4, 1),
    ("blacken", 0x20c, 4, 1),
    ("aggression", 0x210, 4, 8),
    ("strong", 0x230, 4, 8),
    ("weak", 0x250, 4, 8),
    ("dow", 0x270, 4, 8),
    ("invaders", 0x290, 4, 8),
    ("broke_alliance", 0x2b0, 4, 8),
    ("made_peace", 0x2d0, 4, 8),
    ("got_diplo_message", 0x2f0, 4, 1),
    ("last_spoke", 0x2f4, 4, 8),
    ("counteroffer", 0x314, 4, 8),
    ("tribute_demanded", 0x334, 4, 8),
    ("last_taunt", 0x354, 4, 8),
    ("taunt_frame", 0x374, 4, 8),
    ("incoming_taunt", 0x394, 4, 8),
    ("incoming_taunt_who", 0x3b4, 4, 8),
    ("incoming_taunt_frame", 0x3d4, 4, 8),
    ("incoming", 0x3f4, 4, 1),
    ("city_num", 0x3f8, 4, 1),
    ("village_num", 0x3fc, 4, 1),
    ("city_mine", 0x400, 4, 1),
    ("village_mine", 0x404, 4, 1),
    ("city_mark", 0x408, 4, 1),
    ("city_name", 0x40c, 4, 1),
    ("lost_city_stamp", 0x410, 4, 1),
    ("lost_capital_stamp", 0x414, 4, 1),
    ("lost_capital_timer", 0x418, 4, 1),
    ("lost_capital_modifier", 0x41c, 4, 1),
    ("hero_mark", 0x420, 4, 1),
    ("wonder_mark", 0x424, 4, 1),
    ("fort_mark", 0x428, 4, 1),
    ("dock_mark", 0x42c, 4, 1),
    ("oil_well_mark", 0x430, 4, 1),
    ("supply_mark", 0x434, 4, 1),
    ("caravan_mark", 0x438, 4, 1),
    ("special_mark", 0x43c, 4, 1),
    ("popwin_stamp", 0x440, 4, 1),
    ("popwin_timer", 0x444, 4, 1),
    ("wonderwin_stamp", 0x448, 4, 1),
    ("wonderwin_timer", 0x44c, 4, 1),
    ("econ", 0x450, 4, 6),
    ("escrow", 0x468, 4, 6),
    ("escrow_rate", 0x480, 4, 6),
    ("tributes", 0x498, 4, 6),
    ("base_rate", 0x4b0, 4, 6),
    ("worst_good", 0x4c8, 4, 1),
    ("best_good", 0x4cc, 4, 1),
    ("shortages", 0x4d0, 4, 1),
    ("reg_known_rares", 0x4d4, 4, 64),
    ("reg_unpack_merch", 0x5d4, 4, 64),
    ("known_rares", 0x6d4, 4, 1),
    ("rares_collected", 0x6d8, 4, 44),
    ("production_step", 0x788, 4, 1),
    ("prod_script_run", 0x78c, 4, 1),
    ("script_step", 0x790, 4, 1),
    ("wonder_mod", 0x794, 4, 1),
    ("ground_mod", 0x798, 4, 1),
    ("air_mod", 0x79c, 4, 1),
    ("sea_mod", 0x7a0, 4, 1),
    ("infra_mod", 0x7a4, 4, 1),
    ("defense_mod", 0x7a8, 4, 1),
    ("gather_stamp", 0x7ac, 4, 1),
    ("support_stamp", 0x7b0, 4, 1),
    ("nuke_stamp", 0x7b4, 4, 1),
    ("flock_stamp", 0x7b8, 4, 1),
    ("nukes_used", 0x7bc, 4, 1),
    ("missiles_used", 0x7c0, 4, 1),
    ("tech_frame", 0x7c4, 4, 1),
    ("tech_cat_frame", 0x7c8, 4, 4),
    ("victory_type", 0x7d8, 4, 1),
    ("defeat_type", 0x7dc, 4, 1),
    ("site_mark", 0x7e0, 4, 1),
    ("pop_cap", 0x7e4, 4, 1),
    ("pop_issues", 0x7e8, 4, 1),
    ("misery", 0x7ec, 4, 1),
    ("att", 0x7f0, 4, 1),
    ("anti_att", 0x7f4, 4, 1),
    ("give_att_disabled", 0x7f8, 4, 1),
    ("take_att_disabled", 0x7fc, 4, 1),
    ("neutral_attrition", 0x800, 4, 1),
    ("disable_building_attrition", 0x804, 4, 1),
    ("units_built", 0x808, 4, 1),
    ("units_killed", 0x80c, 4, 1),
    ("units_lost", 0x810, 4, 1),
    ("buildings_built", 0x814, 4, 1),
    ("buildings_razed", 0x818, 4, 1),
    ("buildings_lost", 0x81c, 4, 1),
    ("cities_built", 0x820, 4, 1),
    ("cities_captured", 0x824, 4, 1),
    ("cities_lost", 0x828, 4, 1),
    ("army_high", 0x82c, 4, 1),
    ("peasant_high", 0x830, 4, 1),
    ("scholar_high", 0x834, 4, 1),
    ("caravan_high", 0x838, 4, 1),
    ("merchant_high", 0x83c, 4, 1),
    ("city_high", 0x840, 4, 1),
    ("village_high", 0x844, 4, 1),
    ("population_high", 0x848, 4, 1),
    ("city_pop_high", 0x84c, 4, 1),
    ("resources_controlled", 0x850, 4, 1),
    ("wonders_built", 0x854, 4, 1),
    ("wonders_held", 0x858, 4, 1),
    ("senates_built", 0x85c, 4, 1),
    ("tribute_sent", 0x860, 4, 1),
    ("tribute_received", 0x864, 4, 1),
    ("goody_box_techs", 0x868, 4, 1),
    ("goody_box_resources", 0x86c, 4, 1),
    ("goody_box_units", 0x870, 4, 1),
    ("collected", 0x874, 4, 6),
    ("supported", 0x88c, 4, 6),
    ("gather_slots", 0x8a4, 4, 6),
    ("filled_gather_slots", 0x8bc, 4, 6),
    ("gather_slots_high", 0x8d4, 4, 6),
    ("trade_profit", 0x8ec, 4, 1),
    ("forts_built", 0x8f0, 4, 1),
    ("units_bribed", 0x8f4, 4, 1),
    ("age_stamp", 0x8f8, 4, 7),
    ("plunder_scale", 0x914, 4, 1),
    ("bonus_cap", 0x918, 4, 7),
    ("move_orders_issued", 0x934, 4, 1),
    ("column_orders_issued", 0x938, 4, 1),
    ("active", 0x93c, 4, 1),
    ("control", 0x940, 4, 1),
    ("combat", 0x944, 4, 1),
    ("non_siege", 0x948, 4, 1),
    ("sea_combat", 0x94c, 4, 1),
    ("siege", 0x950, 4, 1),
    ("defense", 0x954, 4, 1),
    ("attack", 0x958, 4, 1),
    ("pop", 0x95c, 4, 1),
    ("naval", 0x960, 4, 1),
    ("air", 0x964, 4, 1),
    ("missile", 0x968, 4, 1),
    ("transports", 0x96c, 4, 1),
    ("fishermen", 0x970, 4, 1),
    ("idle_fishermen", 0x974, 4, 1),
    ("peasants", 0x978, 4, 1),
    ("scholars", 0x97c, 4, 1),
    ("caras", 0x980, 4, 1),
    ("merchants", 0x984, 4, 1),
    ("scouts", 0x988, 4, 1),
    ("fighters", 0x98c, 4, 1),
    ("bombers", 0x990, 4, 1),
    ("cruise", 0x994, 4, 1),
    ("nuke", 0x998, 4, 1),
    ("best_pop", 0x99c, 4, 1),
    ("best_attack", 0x9a0, 4, 1),
    ("best_armor", 0x9a4, 4, 1),
    ("best_move", 0x9a8, 4, 1),
    ("wars", 0x9ac, 4, 1),
    ("allies", 0x9b0, 4, 1),
    ("active_wars", 0x9b4, 4, 1),
    ("active_wars_with", 0x9b8, 4, 1),
    ("free_peasants", 0x9bc, 4, 1),
    ("xport_peasants", 0x9c0, 4, 1),
    ("gatherers", 0x9c4, 4, 1),
    ("home_reg", 0x9c8, 4, 1),
    ("attacked", 0x9cc, 4, 1),
    ("full_cities", 0x9d0, 4, 1),
    ("explored", 0x9d4, 4, 1),
    ("territory", 0x9d8, 4, 1),
    ("territory_high", 0x9dc, 4, 1),
    ("effective_pop", 0x9e0, 4, 1),
    ("other_team_terr", 0x9e4, 4, 1),
    ("min_other_team_terr", 0x9e8, 4, 1),
    ("my_team_terr", 0x9ec, 4, 1),
    ("scholar_militia", 0x9f0, 4, 1),
    ("retargets", 0x9f4, 4, 1),
    ("barracks_units", 0x9f8, 4, 1),
    ("stable_units", 0x9fc, 4, 1),
    ("factory_units", 0xa00, 4, 1),
    ("combat_units", 0xa04, 4, 1),
    ("dock_units", 0xa08, 4, 1),
    ("air_units", 0xa0c, 4, 1),
    ("barracks_queued", 0xa10, 4, 1),
    ("stable_queued", 0xa14, 4, 1),
    ("factory_queued", 0xa18, 4, 1),
    ("combat_queued", 0xa1c, 4, 1),
    ("dock_queued", 0xa20, 4, 1),
    ("air_queued", 0xa24, 4, 1),
    ("peasants_garr", 0xa28, 4, 1),
    ("stable_garr", 0xa2c, 4, 1),
    ("scout_garr", 0xa30, 4, 1),
    ("barracks_garr", 0xa34, 4, 1),
    ("nukes_launched", 0xa38, 4, 1),
    ("nukes_in_flight", 0xa3c, 4, 1),
    ("frame_attacked", 0xa40, 4, 1),
    ("attacked_by", 0xa44, 4, 1),
    ("gov_hero_frame", 0xa48, 4, 1),
    ("frame_battle", 0xa4c, 4, 1),
    ("average_death_rate", 0xa50, 2, 1),
    ("average_kill_rate", 0xa52, 2, 1),
    ("average_damage_rate", 0xa54, 2, 1),
    ("average_hit_rate", 0xa56, 2, 1),
    ("deaths_current_frame", 0xa58, 2, 1),
    ("kills_current_frame", 0xa5a, 2, 1),
    ("hits_current_frame", 0xa5c, 2, 1),
    ("damage_current_frame", 0xa5e, 2, 1),
    ("deaths_fifteen_seconds", 0xa60, 2, 1),
    ("kills_fifteen_seconds", 0xa62, 2, 1),
    ("hits_fifteen_seconds", 0xa64, 2, 1),
    ("damage_fifteen_seconds", 0xa66, 2, 1),
    ("strategy", 0xa68, 2, 64),
    ("reg_active", 0xae8, 2, 127),
    ("reg_combat", 0xbe6, 2, 127),
    ("reg_defense", 0xce4, 2, 64),
    ("reg_attack", 0xd64, 2, 127),
    ("reg_pop", 0xe62, 2, 64),
    ("reg_naval", 0xee2, 2, 63),
    ("reg_transports", 0xf60, 2, 63),
    ("reg_peasants", 0xfde, 2, 64),
    ("reg_free_peasants", 0x105e, 2, 64),
    ("reg_xport_peasants", 0x10de, 2, 64),
    ("reg_gatherers", 0x115e, 2, 64),
    ("reg_gather_slots", 0x11de, 2, 64),
    ("reg_cities", 0x125e, 2, 64),
    ("reg_forts", 0x12de, 2, 64),
    ("reg_docks", 0x135e, 2, 64),
    ("reg_land", 0x13de, 2, 64),
    ("reg_terr", 0x145e, 2, 64),
    ("reg_buildings", 0x14de, 2, 8256),
    ("num_buildings", 0x555e, 2, 129),
    ("high_buildings", 0x5660, 2, 129),
    ("num_units", 0x5762, 2, 352),
    ("num_queued", 0x5a22, 2, 806),
    ("last_building_finished", 0x6070, 4, 129),
    ("last_unit_finished", 0x6274, 4, 352),
    ("ages_queued", 0x67f4, 1, 1),
    ("epochs_queued", 0x67f5, 1, 1),
    ("reg_attacked", 0x67f6, 1, 64),
    ("reg_wars", 0x6836, 1, 64),
    ("reg_neutrals", 0x6876, 1, 64),
    ("reg_allies", 0x68b6, 1, 64),
    ("num_bonus_cards", 0x68f6, 1, 38),
    ("num_ctw_rate_bonuses", 0x691c, 1, 6),
    ("defeat_stamp", 0x6924, 4, 1),
    ("team_color", 0x6928, 1, 1),
    ("ally_mask", 0x6929, 1, 1),
];

/// The two uncovered runs inside the long range: `(offset, width)`.
pub const LEADER_PADDING: [(usize, usize); 2] = [(0x606e, 2), (0x6922, 2)];

/// `Diplomacy` (size 0x5c): `(name, count)`, each int.
pub const DIPLOMACY_FIELDS: &[(&str, usize)] = &[
    ("agree", 1),
    ("any_offer", 1),
    ("treaty", 1),
    ("offers", 6),
    ("dows", 6),
    ("attacks", 8),
];

/// `Personality` (size 0x60), 24 ints.
pub const PERSONALITY_FIELDS: [&str; 24] = [
    "rush",
    "cities",
    "upgrades",
    "arms",
    "army",
    "army_size",
    "raid",
    "invade",
    "target",
    "strategy",
    "raze",
    "spells",
    "forts",
    "nukes",
    "air",
    "naval",
    "market",
    "scouts",
    "civilians",
    "early_army",
    "friendly_human",
    "alliance_human",
    "friendly_ai",
    "alliance_ai",
];

/// The `leaders` array's length (`check_leaders` walks to `0xe71af0`).
pub const LEADER_SLOTS: usize = 8;

/// `LeaderDataEncrypt`'s nine per-good arrays in the walk's order.
pub const ENCRYPT_SIX: [&str; 9] = [
    "bucket",
    "leftover",
    "resource_cap",
    "over_cap",
    "resources",
    "support",
    "income",
    "rate",
    "bonus",
];

/// One leaf of a walked range: its name, its element width, its element
/// count, and this crate's values — `None` where the crate holds none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Leaf {
    pub name: String,
    pub width: usize,
    pub count: usize,
    pub values: Option<Vec<i64>>,
}

impl Leaf {
    fn new(name: impl Into<String>, width: usize, count: usize, values: Option<Vec<i64>>) -> Leaf {
        let values = values.map(|mut v| {
            v.resize(count, 0);
            v
        });
        Leaf {
            name: name.into(),
            width,
            count,
            values,
        }
    }

    /// The leaf's bytes, little endian at its width; an unheld leaf's are
    /// zero.
    pub fn bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.width * self.count);
        for k in 0..self.count {
            let v = self.values.as_ref().map_or(0, |v| v[k]);
            out.extend_from_slice(&v.to_le_bytes()[..self.width]);
        }
        out
    }
}

/// One range of the line as leaves: the range's name and its leaves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderRange {
    pub record: i16,
    pub field: &'static str,
    pub leaves: Vec<Leaf>,
}

/// A walked `BitMask<N>`'s two ranges: the header (`bits`, `size`) and the
/// inline array.
fn mask(
    out: &mut Vec<LeaderRange>,
    record: i16,
    field: [&'static str; 2],
    bits: usize,
    set: Option<Vec<usize>>,
) {
    let size = bits.div_ceil(8);
    out.push(LeaderRange {
        record,
        field: field[0],
        leaves: vec![
            Leaf::new("bits", 4, 1, Some(vec![bits as i64])),
            Leaf::new("size", 4, 1, Some(vec![size as i64])),
        ],
    });
    let bytes = set.map(|set| {
        let mut b = vec![0i64; size];
        for i in set.into_iter().filter(|&i| i < bits) {
            b[i / 8] |= 1 << (i % 8);
        }
        b
    });
    out.push(LeaderRange {
        record,
        field: field[1],
        leaves: vec![Leaf::new("ptr", 1, size, bytes)],
    });
}

/// An `Array<T>`'s or a `SimpleArray<int>`'s walk: the length, and with
/// elements the size, `increment` and flags local, then the elements.
fn array_head(
    out: &mut Vec<LeaderRange>,
    record: i16,
    name: &'static str,
    n: usize,
    size: usize,
    flags: u8,
) {
    out.push(LeaderRange {
        record,
        field: name,
        leaves: vec![Leaf::new("length", 4, 1, Some(vec![n as i64]))],
    });
    if n == 0 {
        return;
    }
    out.push(LeaderRange {
        record,
        field: name,
        leaves: vec![Leaf::new("size", 4, 1, Some(vec![size as i64]))],
    });
    out.push(LeaderRange {
        record,
        field: name,
        leaves: vec![Leaf::new("increment", 2, 1, Some(vec![-1]))],
    });
    out.push(LeaderRange {
        record,
        field: name,
        leaves: vec![Leaf::new("flags", 1, 1, Some(vec![i64::from(flags)]))],
    });
}

/// `Leader::init`'s `plunder_scale`, −1.0 as its IEEE-754 single's bits.
const PLUNDER_SCALE_BITS: i32 = 0xbf80_0000_u32 as i32;

/// `LeaderData`'s `0x2000000`, the economy's dirty bit ([`crate::economy::Ledger::dirty`]).
const DIRTY: i64 = 0x200_0000;

/// The `size` a `SimpleArray<int>` with elements reports: 10 on every
/// non-empty one of both frames (`docs/CHECKSUM.md` §20).
const SIMPLE_ARRAY_SIZE: usize = 10;

impl Sim {
    /// **The leaders line** (`docs/CHECKSUM.md` §20).
    pub fn checksum_leaders(&self) -> Line {
        let ranges = (0..LEADER_SLOTS)
            .flat_map(|slot| self.leader_image(slot as Player))
            .map(|r| Range {
                record: r.record,
                field: r.field,
                bytes: r.leaves.iter().flat_map(Leaf::bytes).collect(),
            })
            .collect();
        Line {
            name: "leaders",
            ranges,
        }
    }

    /// The number of leader slots this game seats (`leader_flags & 1`).
    fn leader_seats(&self) -> usize {
        self.economy.ledgers.len().min(LEADER_SLOTS)
    }

    /// A tree id's `TypeIndex`: goods from 0, units from `BASE_UNITTYPES`
    /// (0x32), buildings from `BASE_BUILDTYPES` (0x19e), techs from
    /// `BASE_TECHTYPES` (0x220), each in tree order (`docs/TECH.md`, "The
    /// type space").
    pub(super) fn tree_type_index(&self, t: usize) -> i64 {
        let types = &self.tables.tech_tree.types;
        let g = types
            .iter()
            .filter(|d| matches!(d.kind, crate::tech::Kind::Good))
            .count();
        let u = g + types.iter().filter(|d| d.kind.is_unit()).count();
        let b = u + types.iter().filter(|d| d.kind.is_building()).count();
        let t32 = t as i64;
        if t < g {
            t32
        } else if t < u {
            0x32 + t32 - g as i64
        } else if t < b {
            0x19e + t32 - u as i64
        } else {
            0x220 + t32 - b as i64
        }
    }

    /// A per-region census array as the original indexes it: by its own
    /// region number less `base`, `width` wide.
    fn leader_region_array(&self, v: &[i32], width: usize, base: usize) -> Vec<i64> {
        let mut out = vec![0i64; width];
        for r in 0..self.world.region_count().min(v.len()) {
            let d = self.world.region_number(r as u16) as usize;
            if d >= base && d - base < width {
                out[d - base] = i64::from(crate::ai::Census::reg(v, r as u16));
            }
        }
        out
    }

    /// `(leader_flags, leader_flags2)` of slot `who`, from the bits this
    /// crate keeps (`docs/CHECKSUM.md` §20).
    pub fn leader_flag_words(&self, who: Player) -> (Option<i64>, Option<i64>) {
        let w = who as usize;
        if w >= self.leader_seats() {
            // An unseated slot: `Leaders::init` runs `Leader::init(i, -1, i)`
            // on all eight, whose `leader_flags |= 0x2000000` is the one bit
            // a slot no player takes holds; `leader_flags2` stays 0.
            return (Some(DIRTY), Some(0));
        }
        let mut f = 1i64;
        if !self.leaders.defeated.get(w).copied().unwrap_or(false) {
            f |= 2;
        }
        if self.cities.nation.get(w).is_some_and(|n| n.human) {
            f |= 4;
        }
        if self.economy.ledgers[w].dirty {
            f |= DIRTY;
        }
        // `leader_flags2`: the two bits this crate keeps, no governments
        // (`0x800`) and no patriots (`0x1000`).
        let t = &self.tech[w];
        let f2 = if t.no_governments { 0x800 } else { 0 } | if t.no_patriots { 0x1000 } else { 0 };
        (Some(f), Some(f2))
    }

    /// One slot's ranges as leaves, in walk order.
    pub fn leader_image(&self, who: Player) -> Vec<LeaderRange> {
        let record = i16::from(who);
        let (f1, f2) = self.leader_flag_words(who);
        let mut out = vec![LeaderRange {
            record,
            field: "flags",
            leaves: vec![
                Leaf::new("leader_flags", 4, 1, f1.map(|v| vec![v])),
                Leaf::new("leader_flags2", 4, 1, f2.map(|v| vec![v])),
            ],
        }];
        let w = who as usize;
        if w >= self.leader_seats() {
            return out;
        }
        // The long range.
        let mut leaves = Vec::new();
        let mut at = 0x8;
        for &(name, off, width, count) in LEADER_FIELDS {
            for &(p, pw) in &LEADER_PADDING {
                if at == p && off == p + pw {
                    leaves.push(Leaf::new(format!("pad@{p:#x}"), 1, pw, Some(vec![0; pw])));
                }
            }
            leaves.push(Leaf::new(
                name,
                width,
                count,
                self.leader_long(who, name, count),
            ));
            at = off + width * count;
        }
        out.push(LeaderRange {
            record,
            field: "who..ally_mask",
            leaves,
        });
        for d in 0..8 {
            let _ = d;
            out.push(LeaderRange {
                record,
                field: "dip",
                // `Diplomacy::clear_all`'s image, `Leader::init`'s: `treaty`
                // −1, every other word 0 (§20).
                leaves: DIPLOMACY_FIELDS
                    .iter()
                    .map(|&(n, c)| {
                        Leaf::new(n, 4, c, Some(vec![if n == "treaty" { -1 } else { 0 }]))
                    })
                    .collect(),
            });
        }
        let a = &self.ai.ai[w];
        let p = &a.pers;
        let pers = [
            p.rush,
            p.cities,
            p.upgrades,
            p.arms,
            p.army,
            p.army_size,
            p.raid,
            p.invade,
            p.target,
            p.strategy,
            p.raze,
            p.spells,
            p.forts,
            p.nukes,
            p.air,
            p.naval,
            p.market,
            p.scouts,
            p.civilians,
            p.early_army,
            p.friendly_human,
            p.alliance_human,
            p.friendly_ai,
            p.alliance_ai,
        ];
        out.push(LeaderRange {
            record,
            field: "pers",
            leaves: PERSONALITY_FIELDS
                .iter()
                .zip(pers)
                .map(|(&n, v)| Leaf::new(n, 4, 1, Some(vec![i64::from(v)])))
                .collect(),
        });
        let t = &self.tech[w];
        let bits_of = |v: &[bool]| -> Vec<usize> {
            v.iter()
                .enumerate()
                .filter(|(_, on)| **on)
                .map(|(i, _)| self.tree_type_index(i) as usize)
                .collect()
        };
        mask(
            &mut out,
            record,
            ["tech.head", "tech.ptr"],
            806,
            Some(bits_of(&t.tech)),
        );
        mask(
            &mut out,
            record,
            ["tech_at_start.head", "tech_at_start.ptr"],
            806,
            Some(bits_of(&t.tech_at_start)),
        );
        mask(
            &mut out,
            record,
            ["obs_flags.head", "obs_flags.ptr"],
            806,
            Some(bits_of(&t.obs)),
        );
        mask(
            &mut out,
            record,
            ["conquest_wonders.head", "conquest_wonders.ptr"],
            17,
            Some(Vec::new()),
        );
        mask(
            &mut out,
            record,
            [
                "conquest_wonders_in_game.head",
                "conquest_wonders_in_game.ptr",
            ],
            17,
            Some(Vec::new()),
        );
        mask(
            &mut out,
            record,
            ["conquest_racial_powers.head", "conquest_racial_powers.ptr"],
            24,
            Some(Vec::new()),
        );
        // `sites`: ten slots, an `Array<Site>`.
        array_head(
            &mut out,
            record,
            "sites",
            a.sites.len(),
            a.sites.len(),
            0x80,
        );
        for s in &a.sites {
            // A cleared site (`compute_sites`' `Site::default()`) holds the
            // clear's 0, not a region; a kept one holds ours, renumbered.
            let cleared = s.wx == 0 && s.wy == 0 && s.val == 0 && s.dist == 0 && s.reg == 0;
            let reg = if cleared {
                0
            } else if s.reg >= 0 && (s.reg as usize) < self.world.region_count() {
                i64::from(self.world.region_number(s.reg as u16))
            } else {
                i64::from(s.reg)
            };
            out.push(LeaderRange {
                record,
                field: "sites[k]",
                leaves: [
                    ("wx", i64::from(s.wx)),
                    ("wy", i64::from(s.wy)),
                    ("val", i64::from(s.val)),
                    ("reg", reg),
                    ("dist", i64::from(s.dist)),
                    ("rank", i64::from(s.rank)),
                ]
                .into_iter()
                .map(|(n, v)| Leaf::new(n, 4, 1, Some(vec![v])))
                .collect(),
            });
        }
        // `make_list`: an `Array<MakeObject>`.
        let ml = &a.make_list.list;
        array_head(&mut out, record, "make_list", ml.len(), ml.len(), 0x80);
        let own: Vec<usize> = self
            .cities
            .cities
            .iter()
            .enumerate()
            .filter(|(_, x)| x.alive && usize::from(x.owner) == w)
            .map(|(i, _)| i)
            .collect();
        for m in ml {
            let ti = if m.t < 0 {
                i64::from(m.t)
            } else {
                self.tree_type_index(m.t as usize)
            };
            let city = usize::try_from(m.city)
                .ok()
                .and_then(|c| own.iter().position(|&i| i == c))
                .map_or(i64::from(m.city), |k| k as i64);
            out.push(LeaderRange {
                record,
                field: "make_list[k]",
                leaves: [
                    ("t", ti),
                    ("val", i64::from(m.val)),
                    ("escrow", i64::from(m.escrow)),
                    ("city", city),
                    ("up", i64::from(m.up)),
                    ("o", i64::from(m.o)),
                    ("num", i64::from(m.num)),
                    ("cat", i64::from(m.cat)),
                    ("wx", i64::from(m.wx)),
                    ("wy", i64::from(m.wy)),
                ]
                .into_iter()
                .map(|(n, v)| Leaf::new(n, 4, 1, Some(vec![v])))
                .collect(),
            });
        }
        for (name, v) in [
            ("mil_trainers", &a.mil_trainers),
            ("new_rares", &a.new_rares),
            ("oil_patches", &a.oil_patches),
        ] {
            array_head(
                &mut out,
                record,
                name,
                v.len(),
                SIMPLE_ARRAY_SIZE.max(v.len()),
                0,
            );
            if !v.is_empty() {
                out.push(LeaderRange {
                    record,
                    field: name,
                    leaves: vec![Leaf::new(
                        "elements",
                        4,
                        v.len(),
                        Some(v.iter().map(|&x| x as i64).collect()),
                    )],
                });
            }
        }
        // `prod_script`: the length, then UTF-16 characters.
        let script: Vec<u16> = a.script.as_deref().unwrap_or("").encode_utf16().collect();
        out.push(LeaderRange {
            record,
            field: "prod_script",
            leaves: vec![Leaf::new("length", 4, 1, Some(vec![script.len() as i64]))],
        });
        if !script.is_empty() {
            out.push(LeaderRange {
                record,
                field: "prod_script",
                leaves: vec![Leaf::new(
                    "chars",
                    2,
                    script.len(),
                    Some(script.iter().map(|&c| i64::from(c)).collect()),
                )],
            });
        }
        let l = &self.economy.ledgers[w];
        let h = &self.economy.holdings[w];
        let set44 = |m: u64| (0..44).filter(|i| m >> i & 1 == 1).collect::<Vec<usize>>();
        mask(
            &mut out,
            record,
            ["rare.head", "rare.ptr"],
            44,
            Some(set44(l.rare)),
        );
        mask(
            &mut out,
            record,
            ["rare_owned.head", "rare_owned.ptr"],
            44,
            Some(set44(h.rare_owned)),
        );
        mask(
            &mut out,
            record,
            ["rare_conquest.head", "rare_conquest.ptr"],
            44,
            Some(Vec::new()),
        );
        // The 62 decrypted words.
        let mut word = |name: String, v: Option<i64>| {
            out.push(LeaderRange {
                record,
                field: "encrypt",
                leaves: vec![Leaf::new(name, 4, 1, v.map(|v| vec![v]))],
            })
        };
        for g in 0..6 {
            for f in ENCRYPT_SIX {
                let v = match f {
                    "bucket" => Some(l.bucket[g]),
                    "leftover" => Some(l.leftover[g]),
                    "resource_cap" => Some(l.cap[g]),
                    "over_cap" => Some(match l.over_cap[g] {
                        crate::economy::OverCap::Under => 0,
                        crate::economy::OverCap::At => 1,
                        crate::economy::OverCap::Uncapped => 2,
                    }),
                    "resources" => Some(l.rate[g]),
                    "income" => Some(l.income[g]),
                    "rate" => Some(a.rate[g]),
                    // `support` and `bonus`: every writer stores 0
                    // (`Leader::init`, `Leader::gather`'s `support`,
                    // `Leader::calc_gather`'s `bonus`).
                    "support" | "bonus" => Some(0),
                    _ => None,
                };
                word(format!("{f}[{g}]"), v.map(i64::from));
            }
        }
        word(
            "resource_cap[6]".into(),
            Some(i64::from(crate::economy::commerce_cap_seventh(
                &self.tables.tuning,
                h,
            ))),
        );
        for e in 0..4 {
            word(format!("epoch[{e}]"), Some(i64::from(t.epoch[e])));
        }
        word("ages".into(), Some(i64::from(t.ages)));
        word("epochs".into(), Some(i64::from(t.epochs)));
        word("discovered".into(), Some(i64::from(t.discovered)));
        out
    }

    /// One leaf of the long range as this crate holds it, or `None`.
    fn leader_long(&self, who: Player, name: &str, count: usize) -> Option<Vec<i64>> {
        let w = who as usize;
        let l = &self.economy.ledgers[w];
        let h = &self.economy.holdings[w];
        let a = &self.ai.ai[w];
        let c = &a.census;
        let t = &self.tech[w];
        let m = &self.roster.muster[w];
        let one = |v: i64| Some(vec![v]);
        let six = |v: &[i32]| Some(v.iter().map(|&x| i64::from(x)).collect::<Vec<_>>());
        let players = self.leader_seats();
        let per_player = |f: &dyn Fn(usize) -> i64| {
            Some(
                (0..count)
                    .map(|i| if i < players { f(i) } else { 0 })
                    .collect::<Vec<_>>(),
            )
        };
        let reg = |v: &[i32]| Some(self.leader_region_array(v, count, 0));
        match name {
            "who" => one(w as i64),
            "tribe" => one(t.tribe as i64),
            "gov" => one(t.gov.map_or(-1, |g| self.tree_type_index(g))),
            "diplos" => per_player(&|i| {
                if i == w {
                    2
                } else if self.leaders.at_war[w][i] {
                    0
                } else if self.leaders.allied[w][i] {
                    2
                } else {
                    1
                }
            }),
            "treaties" => per_player(&|i| {
                if i == w {
                    1
                } else {
                    i64::from(self.leaders.treaties[w][i])
                }
            }),
            "agendas" => per_player(&|i| {
                self.leaders
                    .agendas
                    .get(w)
                    .and_then(|r| r.get(i))
                    .copied()
                    .map_or(0, i64::from)
            }),
            "capital_stamp" => Some(a.capital_stamp.to_vec()),
            "aggression" => six(&a.aggression),
            "strong" => six(&a.strong),
            "weak" => six(&a.weak),
            "invaders" => six(&c.invaders),
            "got_diplo_message" => one(a.got_diplo_message),
            "city_num" => one(self
                .cities
                .cities
                .iter()
                .filter(|x| x.alive && usize::from(x.owner) == w)
                .count() as i64),
            "village_num" => one(i64::from(c.village_num)),
            // `lost_city_stamp` (`+0x410`): `City::close` stamps the frame a
            // city is lost; before any, `Leader::init`'s −90 (`0xffffffa6`).
            "lost_city_stamp" => one(self
                .cities
                .lost_city_stamp
                .get(w)
                .copied()
                .flatten()
                .unwrap_or(-90)),
            "wonder_mark" => one(i64::from(c.wonder_mark)),
            "popwin_stamp" => one(a.popwin_stamp),
            "popwin_timer" => one(i64::from(a.popwin_timer)),
            "econ" => six(&a.econ),
            "escrow" => six(&l.escrow),
            "escrow_rate" => six(&c.escrow_rate),
            "worst_good" => one(a.worst_good as i64),
            "best_good" => one(a.best_good as i64),
            "shortages" => one(i64::from(a.shortages)),
            "reg_known_rares" => reg(&c.reg_known_rares),
            "reg_unpack_merch" => reg(&c.reg_unpack_merch),
            "known_rares" => one(i64::from(a.known_rares)),
            "rares_collected" => six(&h.rares_collected),
            "production_step" => one(i64::from(a.step.number())),
            "prod_script_run" => one(i64::from(a.script_live)),
            "script_step" => one(i64::from(a.script_step)),
            "wonder_mod" => one(i64::from(a.wonder_mod)),
            "ground_mod" => one(i64::from(a.ground_mod)),
            "air_mod" => one(i64::from(a.air_mod)),
            "sea_mod" => one(i64::from(a.sea_mod)),
            "infra_mod" => one(i64::from(a.infra_mod)),
            "defense_mod" => one(i64::from(a.defense_mod)),
            "gather_stamp" => one(l.gather_stamp),
            "flock_stamp" => one(self.gaia.flock_stamp.get(w).copied().unwrap_or(0)),
            "tech_frame" => one(a.tech_frame),
            "tech_cat_frame" => Some(a.tech_cat_frame.to_vec()),
            "site_mark" => one(i64::from(a.site_mark)),
            "pop_cap" => one(i64::from(m.cap)),
            "army_high" => one(i64::from(c.army_high)),
            "peasant_high" => one(i64::from(c.peasant_high)),
            "scholar_high" => one(i64::from(c.scholar_high)),
            "caravan_high" => one(i64::from(c.caravan_high)),
            "merchant_high" => one(i64::from(c.merchant_high)),
            "city_high" => one(i64::from(c.city_high)),
            "village_high" => one(i64::from(c.village_high)),
            "population_high" => one(i64::from(c.population_high)),
            "city_pop_high" => one(i64::from(c.city_pop_high)),
            "resources_controlled" => one(i64::from(c.resources_controlled)),
            "goody_box_resources" => one(i64::from(l.goody_box_resources)),
            "collected" => six(&l.collected),
            "gather_slots" => six(&l.gather_slots),
            "filled_gather_slots" => six(&c.filled_gather_slots),
            "gather_slots_high" => six(&l.gather_slots_high),
            "age_stamp" => Some(t.age_stamp.iter().map(|s| s.unwrap_or(-1)).collect()),
            "bonus_cap" => six(&h.bonus_cap),
            "active" => one(i64::from(c.active)),
            "control" => one(i64::from(m.control)),
            "combat" => one(i64::from(c.combat)),
            "non_siege" => one(i64::from(c.non_siege)),
            "sea_combat" => one(i64::from(c.sea_combat)),
            "siege" => one(i64::from(c.siege)),
            "defense" => one(i64::from(c.defense)),
            "attack" => one(i64::from(c.attack)),
            "pop" => one(i64::from(c.pop)),
            "naval" => one(i64::from(c.naval)),
            "air" => one(i64::from(c.air)),
            "missile" => one(i64::from(c.missile)),
            "transports" => one(i64::from(c.transports)),
            "fishermen" => one(i64::from(c.fishermen)),
            "idle_fishermen" => one(i64::from(c.idle_fishermen)),
            "peasants" => one(i64::from(c.peasants)),
            "scholars" => one(i64::from(c.scholars)),
            "caras" => one(i64::from(c.caras)),
            "merchants" => one(i64::from(c.merchants)),
            "scouts" => one(i64::from(c.scouts)),
            "fighters" => one(i64::from(c.fighters)),
            "bombers" => one(i64::from(c.bombers)),
            "cruise" => one(i64::from(c.cruise)),
            "nuke" => one(i64::from(c.nuke)),
            "wars" => one(i64::from(c.wars)),
            "allies" => one(i64::from(c.allies)),
            "active_wars" => one(i64::from(c.active_wars)),
            "active_wars_with" => one(i64::from(c.active_wars_with)),
            "free_peasants" => one(i64::from(c.free_peasants)),
            "xport_peasants" => one(i64::from(c.xport_peasants)),
            "gatherers" => one(i64::from(c.gatherers)),
            "home_reg" => one(
                if c.home_reg >= 0 && (c.home_reg as usize) < self.world.region_count() {
                    i64::from(self.world.region_number(c.home_reg as u16))
                } else {
                    i64::from(c.home_reg)
                },
            ),
            "attacked" => one(i64::from(c.attacked)),
            "full_cities" => one(i64::from(c.full_cities)),
            "explored" => one(i64::from(c.explored)),
            "territory" => one(i64::from(h.territory)),
            "effective_pop" => one(i64::from(a.effective_pop)),
            "other_team_terr" => one(i64::from(c.other_team_terr)),
            "min_other_team_terr" => one(i64::from(c.min_other_team_terr)),
            "my_team_terr" => one(i64::from(c.my_team_terr)),
            "retargets" => one(i64::from(
                self.combat.retargets.get(w).copied().unwrap_or(0),
            )),
            "peasants_garr" => one(i64::from(c.peasants_garr)),
            "stable_garr" => one(i64::from(c.stable_garr)),
            "scout_garr" => one(i64::from(c.scout_garr)),
            "barracks_garr" => one(i64::from(c.barracks_garr)),
            "frame_attacked" => one(a.frame_attacked),
            "attacked_by" => one(i64::from(a.attacked_by)),
            "gov_hero_frame" => one(t.gov_hero_frame),
            "strategy" => reg(&c.strategy),
            "reg_active" => reg(&c.reg_active),
            "reg_combat" => reg(&c.reg_combat),
            "reg_defense" => reg(&c.reg_defense),
            "reg_attack" => reg(&c.reg_attack),
            "reg_pop" => reg(&c.reg_pop),
            "reg_naval" => Some(self.leader_region_array(&c.reg_naval, count, 0x3f)),
            "reg_transports" => Some(self.leader_region_array(&c.reg_transports, count, 0x3f)),
            "reg_peasants" => reg(&c.reg_peasants),
            "reg_free_peasants" => {
                reg(&c.reg_free_peasants).map(|v| v.into_iter().map(|x| x & 0xffff).collect())
            }
            "reg_xport_peasants" => reg(&c.reg_xport_peasants),
            "reg_gatherers" => reg(&c.reg_gatherers),
            "reg_gather_slots" => reg(&c.reg_gather_slots),
            "reg_cities" => reg(&c.reg_cities),
            "reg_docks" => reg(&c.reg_docks),
            "reg_land" => reg(&c.reg_land),
            "num_units" => six(&m.by_type),
            "num_queued" => {
                // Keyed by `TypeIndex`: a unit's queue count, a building's
                // placed sites, a technology's queue count.
                let mut v = vec![0i64; count];
                for (r, n) in m.queued_by_type.iter().enumerate() {
                    if 0x32 + r < count {
                        v[0x32 + r] = i64::from(*n);
                    }
                }
                for rec in 0..self.tables.build_types.len() {
                    if 0x19e + rec < count {
                        v[0x19e + rec] = i64::from(self.num_sites_of(who, rec));
                    }
                }
                for (id, n) in t.queued.iter().enumerate() {
                    let ti = self.tree_type_index(id);
                    if ti >= 0x220 && (ti as usize) < count {
                        v[ti as usize] = i64::from(*n);
                    }
                }
                Some(v)
            }
            "num_buildings" => Some(
                (0..count)
                    .map(|rec| {
                        if rec < self.tables.build_types.len() {
                            i64::from(self.num_buildings_of(who, rec))
                        } else {
                            0
                        }
                    })
                    .collect(),
            ),
            "reg_buildings" => {
                // `[64][129]`, the land region by its own number, then the
                // building record: finished buildings standing there.
                let mut v = vec![0i64; count];
                for b in self
                    .buildings
                    .iter()
                    .filter(|b| b.alive && b.active && b.owner == who)
                {
                    let (Some(rec), Some(r)) = (b.ty, self.world.region_of(b.pos.cell())) else {
                        continue;
                    };
                    let d = self.world.region_number(r) as usize;
                    if d < 64 && rec < 129 {
                        v[d * 129 + rec] += 1;
                    }
                }
                Some(v)
            }
            "hero_mark" => one(self.roster.heroes.get(w).map_or(0, |x| x.mark as i64)),
            "special_mark" => one(self.roster.specials.get(w).map_or(0, |x| x.mark as i64)),
            "caravan_mark" => one(self.economy.caravans.get(w).map_or(0, |x| x.mark as i64)),
            // `plunder_scale` (`+0x914`), a float: `Leader::init` stores
            // `0xbf800000` (−1.0) and is its one writer (`Unit::plunder` and
            // `Build::plunder` read it), so it is emitted as those bits
            // (`docs/DECISIONS.md` entry 120 §3, class 1; §20).
            "plunder_scale" => one(i64::from(PLUNDER_SCALE_BITS)),
            "ages_queued" => one(i64::from(t.ages_queued)),
            "epochs_queued" => one(i64::from(t.epochs_queued)),
            "reg_attacked" => reg(&c.reg_attacked),
            "reg_wars" => reg(&c.reg_wars),
            "reg_neutrals" => reg(&c.reg_neutrals),
            "reg_allies" => reg(&c.reg_allies),
            "ally_mask" => one(i64::from(c.ally_mask)),
            _ => None,
        }
    }
}
