use super::*;
use crate::build::{BuildType, flags};
use crate::tech::{TechTree, TypeDef};
use crate::world::{Terrain, UNITS_PER_TILE};
use crate::{Cell, Player, Pos, Tuning, World};

const TILE: i32 = UNITS_PER_TILE;

/// **The only route a tech offer takes to the ceiling** — item 382,
/// `docs/AI.md` §45. Great Lakes' block 9179 measured on both sides:
/// Mercenaries' weights are `[152, 16, 4, 0, 10, 0, 0, 0, 44, 0, 0]`,
/// so [`weight_total`] answers **110** at peace (`ai[5] + ai[1]`) and
/// **144** at war (`ai[0] / 3`), and every other factor on the frame is
/// shared. The pre-income values that come out of those two are what
/// the dump shows either side of the overflow.
#[test]
fn the_tech_ceiling_is_the_income_multiply_wrapping_not_a_saturation() {
    // `income` on the frame, read off this crate's own `check_income`.
    const INCOME: i32 = 0x40;
    // The war weight and the peace weight, from the same `base`.
    const AT_WAR: i32 = 34_560_000;
    const AT_PEACE: i32 = 26_400_000;

    // The original's offer wraps 32 bits and the guard catches the sign.
    assert!(mul(INCOME, AT_WAR) < 0, "the product must overflow");
    assert_eq!(income_scaled(INCOME, AT_WAR), 9_999_999);
    // This crate's does not, and comes out a quarter of itself.
    assert!(mul(INCOME, AT_PEACE) > 0);
    assert_eq!(income_scaled(INCOME, AT_PEACE), 6_600_000);

    // The threshold, exactly: `2^31 / 0x40`.
    assert_eq!(income_scaled(INCOME, 33_554_431), 8_388_607);
    assert_eq!(income_scaled(INCOME, 33_554_432), 9_999_999);

    // And the two weights are the whole of the parting: 55/72.
    let ai = [152, 16, 4, 0, 10, 0, 0, 0, 44, 0, 0];
    let peace = WeightFacts {
        team_style: 0,
        cities: 2,
        full_cities: 0,
        my_team_terr: 568,
        other_team_terr: 266,
        min_other_team_terr: 266,
        active_wars: 0,
        sea_map: 1,
    };
    let war = WeightFacts {
        active_wars: 1,
        ..peace
    };
    assert_eq!(weight_total(&ai, &peace), 110);
    assert_eq!(weight_total(&ai, &war), 144);
    assert_eq!(i64::from(AT_PEACE) * 144, i64::from(AT_WAR) * 110);
}

fn tile_pos(tx: i32, ty: i32) -> Pos {
    Pos::new(tx * TILE + TILE / 2, ty * TILE + TILE / 2)
}

struct Types {
    village: usize,
    library: usize,
    temple: usize,
    university: usize,
    senate: usize,
    tower: usize,
    barracks: usize,
    classical: TypeId,
    science1: TypeId,
    commerce2: TypeId,
    military1: TypeId,
    taxation: TypeId,
    religion: TypeId,
    agriculture: TypeId,
    fortification: TypeId,
    gov_a: TypeId,
    gov_b: TypeId,
    /// A unit type at the Barracks that needs the Classical Age — an
    /// upgrade the leader researches before it trains one.
    phalanx: TypeId,
}

fn bt(ident: Ident, flag: &str) -> BuildType {
    BuildType {
        ident,
        x_size: 4,
        y_size: 4,
        flags: flags::parse(flag),
        job_time: 100,
        hits: 500,
        ..BuildType::default()
    }
}

/// A flat world, two leaders — 0 human, 1 the AI — with every research
/// building available and at least one tech at each.
fn sim() -> (Sim, Types) {
    let mut w = World::new(40, 40);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
    let mut sim = Sim::new(Tuning::RON, w, 2);
    // These are the research coin's tests, and the coin's outcome is
    // the *seed's*. A building started here would plan its road
    // (`Wall::start`, `crate::roads` §1) and spend a draw a node before
    // the coin is tossed, which is a different question.
    sim.plan_roads = false;
    for l in &mut sim.ledgers {
        l.bucket = [10_000; RESOURCES];
        l.income = [100 * 16; RESOURCES];
        l.cap = [1000 * 16; RESOURCES];
    }
    let village = sim.add_build_type(bt(Ident::Village, "ean"));
    let library = sim.add_build_type(bt(Ident::Library, "jam"));
    let temple = sim.add_build_type(bt(Ident::Temple, "jam"));
    let university = sim.add_build_type(bt(Ident::University, "jam"));
    let senate = sim.add_build_type(bt(Ident::Senate, "jam"));
    let tower = sim.add_build_type(bt(Ident::Tower, "ean"));
    let barracks = {
        let mut b = bt(Ident::Barracks, "ean");
        b.flags |= MILITARY_TRAINER;
        sim.add_build_type(b)
    };

    let mut tree = TechTree::new().with_tuning(&Tuning::RON);
    for name in ["Food", "Timber", "Wealth", "Knowledge", "Metal", "Oil"] {
        let mut d = TypeDef::good(name);
        d.tribe_mask = u32::MAX;
        tree.add(d);
    }
    let recs = [
        village, library, temple, university, senate, tower, barracks,
    ];
    let names = [
        "Village",
        "Library",
        "Temple",
        "University",
        "Senate",
        "Tower",
        "Barracks",
    ];
    let mut ids = Vec::new();
    for name in names {
        let mut d = TypeDef::building(name);
        d.tribe_mask = u32::MAX;
        ids.push(tree.add(d));
    }
    let (lib_id, temple_id, uni_id, senate_id, tower_id) = (ids[1], ids[2], ids[3], ids[4], ids[5]);
    let barracks_id = ids[6];

    let age_names = [
        "Classical Age",
        "Medieval Age",
        "Gunpowder Age",
        "Enlightenment Age",
        "Industrial Age",
        "Modern Age",
        "Information Age",
    ];
    let mut ages = Vec::new();
    for (n, name) in age_names.iter().enumerate() {
        ages.push(tree.add(TypeDef::age(name, n as u8).at(lib_id)));
    }
    let mut epochs = Vec::new();
    for line in Line::ALL {
        for level in 0..7u8 {
            let name = format!("{line:?} {level}");
            epochs.push(tree.add(TypeDef::epoch(&name, line, level).at(lib_id)));
        }
    }
    let taxation = tree.add(TypeDef::plain("Taxation", 0).at(temple_id));
    let religion = tree.add(TypeDef::plain("Religion", 0).at(temple_id));
    let agriculture = tree.add(TypeDef::plain("Agriculture", 0).at(uni_id));
    let fortification = tree.add(TypeDef::plain("Fortification", 0).at(tower_id));
    let mut govs = Vec::new();
    for (i, name) in [
        "Empire",
        "Republic",
        "Monarchy",
        "Democracy",
        "Socialism",
        "Capitalism",
    ]
    .iter()
    .enumerate()
    {
        govs.push(tree.add(TypeDef::gov(name, (i / 2) as u8, (i % 2) as u8).at(senate_id)));
    }
    let mut phalanx = TypeDef::unit("Phalanx", tech::UnitTraits::default())
        .at(barracks_id)
        .needs(0, ages[0]);
    phalanx.tribe_mask = u32::MAX;
    let phalanx = tree.add(phalanx);

    sim.set_tech_tree(tree);
    for (rec, id) in recs.iter().zip(ids.iter()) {
        sim.tables.build_types[*rec].tree = Some(*id);
    }
    sim.nation[0].human = true;
    let t = Types {
        village,
        library,
        temple,
        university,
        senate,
        tower,
        barracks,
        classical: ages[0],
        science1: epochs[Line::Science.index() * 7],
        commerce2: epochs[Line::Commerce.index() * 7 + 1],
        military1: epochs[Line::Military.index() * 7],
        taxation,
        religion,
        agriculture,
        fortification,
        gov_a: govs[0],
        gov_b: govs[1],
        phalanx,
    };
    (sim, t)
}

fn city(sim: &mut Sim, t: &Types, who: Player, tx: i32, ty: i32) -> usize {
    let b = sim
        .place_building(who, t.village, tile_pos(tx, ty))
        .expect("the city places");
    while !sim.buildings[b].active && sim.buildings[b].alive {
        sim.do_construct(b, 1_000_000);
    }
    let c = sim.buildings[b].city.expect("a finished city has a record");
    let w = who as usize;
    sim.ai[w]
        .census
        .resize(sim.world.region_count(), sim.territory.players.len());
    sim.ai[w]
        .city_ai
        .resize(sim.cities.len(), Default::default());
    sim.ai[w].census.pop = 20;
    c
}

fn building(sim: &mut Sim, who: Player, rec: usize, tx: i32, ty: i32) -> usize {
    let b = sim
        .place_building(who, rec, tile_pos(tx, ty))
        .expect("it places");
    while !sim.buildings[b].active && sim.buildings[b].alive {
        sim.do_construct(b, 1_000_000);
    }
    b
}

/// Keeps the scores small enough to read: one head of population and a
/// shortage, so the deep multiplier chains do not wrap the way the
/// original's do.
fn small(sim: &mut Sim, who: Player) {
    sim.ai[who as usize].census.pop = 1;
    sim.ai[who as usize].shortages = 1;
}

/// The value one tech would be offered at, straight from the scorer.
fn value(sim: &mut Sim, who: Player, t: TypeId) -> Option<(i32, i32)> {
    let where_ = sim.tables.tech_tree.types[t]
        .where_
        .expect("a research building");
    let rec = sim.build_record(where_).expect("a building record");
    let low = sim.lowest_epoch(who);
    let over = sim.goods_over_cap(who);
    sim.tech_value(who, t, rec, low, over)
}

fn offered(sim: &Sim, who: Player) -> Vec<i32> {
    sim.ai[who as usize]
        .make_list
        .list
        .iter()
        .map(|m| m.t)
        .filter(|&x| x >= 0)
        .collect()
}

// ---- the pure arithmetic ----

#[test]
fn the_weight_block_collapses_to_one_with_no_weights() {
    assert_eq!(weight_total(&[0; AI_WEIGHTS], &WeightFacts::default()), 1);
}

#[test]
fn the_weight_block_sums_the_eleven_with_ai_zero_tenthed() {
    let mut ai = [0; AI_WEIGHTS];
    ai[0] = 30;
    ai[3] = 5;
    let f = WeightFacts {
        active_wars: 1,
        ..WeightFacts::default()
    };
    // 1 + 30/10 + 5 = 9, the war term is ai[0]/3 = 10, no sea.
    assert_eq!(weight_total(&ai, &f), 19);
}

#[test]
fn peace_adds_ai_five_and_ai_one_where_war_adds_a_third_of_ai_zero() {
    let mut ai = [0; AI_WEIGHTS];
    ai[0] = 30;
    ai[1] = 2;
    ai[5] = 4;
    let war = WeightFacts {
        active_wars: 1,
        ..WeightFacts::default()
    };
    let peace = WeightFacts {
        active_wars: 0,
        ..WeightFacts::default()
    };
    // The sum is 1 + 3 + 2 + 4 = 10 either way.
    assert_eq!(weight_total(&ai, &war), 10 + 10);
    assert_eq!(weight_total(&ai, &peace), 10 + 4 + 2);
}

#[test]
fn the_team_style_sign_flips_on_ai_ten() {
    let mut ai = [0; AI_WEIGHTS];
    ai[10] = 7;
    let base = WeightFacts {
        active_wars: 1,
        ..WeightFacts::default()
    };
    for style in [0, 8, 0xb] {
        let f = WeightFacts {
            team_style: style,
            ..base
        };
        assert_eq!(weight_total(&ai, &f), 1 + 7 + 7);
    }
    let f = WeightFacts {
        team_style: 1,
        ..base
    };
    assert_eq!(weight_total(&ai, &f), 1 + 7 - 7);
}

#[test]
fn full_cities_and_territory_ride_on_ai_five() {
    let mut ai = [0; AI_WEIGHTS];
    ai[5] = 10;
    let base = WeightFacts {
        team_style: 1,
        active_wars: 1,
        ..WeightFacts::default()
    };
    let f = WeightFacts {
        cities: 4,
        full_cities: 3,
        ..base
    };
    assert_eq!(weight_total(&ai, &f), 1 + 10 + 20);
    let f = WeightFacts {
        cities: 4,
        full_cities: 1,
        ..base
    };
    assert_eq!(weight_total(&ai, &f), 1 + 10 + 10);
    let f = WeightFacts {
        my_team_terr: 1,
        other_team_terr: 9,
        min_other_team_terr: 5,
        ..base
    };
    assert_eq!(weight_total(&ai, &f), 1 + 10 + 20);
}

#[test]
fn the_sea_term_is_cubed_and_negated_on_a_dry_map() {
    let mut ai = [0; AI_WEIGHTS];
    ai[2] = 2;
    let f = WeightFacts {
        team_style: 1,
        active_wars: 1,
        sea_map: 3,
        ..WeightFacts::default()
    };
    assert_eq!(weight_total(&ai, &f), 1 + 2 + 2 * 27);
    let f = WeightFacts { sea_map: 0, ..f };
    assert_eq!(weight_total(&ai, &f), 1 + 2 - 2);
}

#[test]
fn the_recency_factors_are_four_and_three_in_any_real_game() {
    for frame in [0, 1, 100, 100_000] {
        assert_eq!(recency(frame, 1, 14400, 12, 4), 4);
        assert_eq!(recency(frame, 1, 25600, 16, 3), 3);
    }
    // `f2 × (f1 × val / 2) / 2` is `val × 3`.
    let v = 1000;
    assert_eq!(mul(3, mul(4, v) / 2) / 2, 3 * v);
}

#[test]
fn the_recency_factors_only_fall_below_their_cap_on_a_future_stamp() {
    assert_eq!(recency(-8 * 14400, 1, 14400, 12, 4), 4);
    assert_eq!(recency(-9 * 14400, 1, 14400, 12, 4), 4);
    assert_eq!(recency(-10 * 14400, 1, 14400, 12, 4), 3);
    assert_eq!(recency(-13 * 25600, 1, 25600, 16, 3), 3);
    assert_eq!(recency(-14 * 25600, 1, 25600, 16, 3), 3);
    assert_eq!(recency(-15 * 25600, 1, 25600, 16, 3), 2);
}

/// `get_cost`'s British taxation term (`docs/AI.md` §74). run227's
/// who=1, the British, held 106 food and 76 timber on block 16779 and
/// priced Taxation at 88 each here — `8 × TECH_COST_FACTOR` and the
/// Science surcharge — so `check_income`'s escrow arm read it
/// unaffordable (`0x40`) where the original's, at half the price, read
/// `0x100`: the offer's value 22784 against 91136.
#[test]
fn the_british_pay_half_for_the_taxation_line() {
    let (mut sim, t) = sim();
    for x in [t.taxation, t.religion] {
        sim.tables.tech_tree.types[x].cost = [8, 8, 0, 0, 0, 0];
    }
    sim.tables.tech_tree.roles.taxation_line = vec![t.taxation];
    sim.holdings[1].available = [true, true, true, true, true, false];
    sim.ledgers[1].bucket = [106, 76, 89, 299, 20, 0];
    sim.tech[1].has_city = true;
    let income = |sim: &Sim, x| sim.check_income(1, x, 0x400, None, true, -1, 1, 0);
    assert_eq!(sim.tech_price(1, t.taxation), [88, 88, 0, 0, 0, 0]);
    assert_eq!(
        income(&sim, t.taxation),
        0x40,
        "not British: 88 is out of reach"
    );
    sim.tech[1].power = Some(0xb);
    assert_eq!(sim.tech_price(1, t.taxation), [44, 44, 0, 0, 0, 0]);
    assert_eq!(income(&sim, t.taxation), 0x100, "British: 44 is in reach");
    // Only the four taxation techs, and only for the British.
    assert_eq!(sim.tech_price(1, t.religion), [88, 88, 0, 0, 0, 0]);
    sim.tech[1].power = Some(0xc);
    assert_eq!(sim.tech_price(1, t.taxation), [88, 88, 0, 0, 0, 0]);
}

/// `get_cost`'s German industry term (`:1277`–`1281`, item 3378): a
/// German leader pays half for a food, timber or metal gathering-line
/// technology. run1003's who=1, the Germans, listed Carpentry at `val`
/// 120000 on block 13579 where this crate, pricing it whole, listed it
/// at a quarter of that and dropped it under the Crossbowmen.
#[test]
fn the_germans_pay_half_for_the_industry_lines() {
    let (mut sim, t) = sim();
    for x in [t.taxation, t.religion] {
        sim.tables.tech_tree.types[x].cost = [8, 8, 0, 0, 0, 0];
    }
    sim.tables.tech_tree.types[t.taxation].name = "Carpentry".into();
    sim.holdings[1].available = [true, true, true, true, true, false];
    sim.ledgers[1].bucket = [106, 76, 89, 299, 20, 0];
    sim.tech[1].has_city = true;
    let income = |sim: &Sim, x| sim.check_income(1, x, 0x400, None, true, -1, 1, 0);
    assert_eq!(sim.tech_price(1, t.taxation), [88, 88, 0, 0, 0, 0]);
    assert_eq!(
        income(&sim, t.taxation),
        0x40,
        "not German: 88 is out of reach"
    );
    sim.tech[1].power = Some(0xc);
    assert_eq!(sim.tech_price(1, t.taxation), [44, 44, 0, 0, 0, 0]);
    assert_eq!(income(&sim, t.taxation), 0x100, "German: 44 is in reach");
    // Only the industry lines, and only for the Germans.
    assert_eq!(sim.tech_price(1, t.religion), [88, 88, 0, 0, 0, 0]);
    sim.tech[1].power = Some(0xb);
    assert_eq!(sim.tech_price(1, t.taxation), [88, 88, 0, 0, 0, 0]);
}

/// `get_cost:1048`–`1053`: a government costs an American leader
/// nothing, and only a government, and only the Americans (item 3374:
/// run999b's who=1 queues Republic on 10982 and pays only a Dock's
/// timber beside it).
#[test]
fn an_american_government_is_free() {
    let (mut sim, t) = sim();
    for x in [t.gov_b, t.taxation] {
        sim.tables.tech_tree.types[x].cost = [7, 0, 0, 0, 7, 0];
    }
    sim.holdings[1].available = [true, true, true, true, true, false];
    let plain = sim.tech_price(1, t.gov_b);
    assert!(plain[0] > 0 && plain[4] > 0);
    let taxation = sim.tech_price(1, t.taxation);
    sim.tech[1].power = Some(crate::ai::tribe::AMERICANS);
    assert_eq!(sim.tech_price(1, t.gov_b), [0; RESOURCES]);
    assert_eq!(sim.tech_price(1, t.taxation), taxation, "a plain tech pays");
    sim.tech[1].power = Some(crate::ai::tribe::DUTCH);
    assert_eq!(sim.tech_price(1, t.gov_b), plain);
}

/// The price feeds both AI affordability and the purchase: run574's
/// Medicine offer and Monotheism charge (`docs/AI.md` §109).
#[test]
fn democracy_prices_non_library_research_before_british_taxation() {
    use crate::tech::Preq;
    let (mut sim, t) = sim();
    sim.holdings[1].available = [true, true, true, true, true, false];
    sim.tech[1].has_city = true;
    sim.tables.tech_tree.types[t.religion].cost = [15, 0, 15, 0, 0, 0];
    sim.tables.tech_tree.types[t.agriculture].cost = [18, 0, 18, 0, 0, 0];
    sim.ledgers[1].bucket = [193, 189, 242, 619, 218, 0];
    let income = |s: &Sim| s.check_income(1, t.agriculture, 0x400, None, true, -1, 1, 0);
    assert_eq!(sim.tech_price(1, t.religion), [165, 0, 165, 0, 0, 0]);
    assert_eq!(income(&sim), 0x40, "198 food exceeds the 193 held");
    let lower = [Preq::Of(t.gov_a), Preq::None, Preq::None];
    let higher = [Preq::Of(t.gov_b), Preq::Of(t.gov_a), Preq::None];
    sim.tables.tech_tree.roles.democracy_preqs = [Some(lower), Some(higher)];
    sim.tech[1].tech[t.gov_b] = true;
    assert_eq!(
        sim.tech_price(1, t.religion)[0],
        165,
        "all prerequisite slots matter"
    );
    sim.tech[1].tech[t.gov_a] = true;
    assert_eq!(sim.tech_price(1, t.religion), [132, 0, 132, 0, 0, 0]);
    assert_eq!(sim.tech_price(1, t.agriculture)[0], 158);
    assert_eq!(
        income(&sim),
        0x100,
        "Medicine's affordability multiplier quadruples"
    );

    // Separate tiers so the else-chain cannot accidentally compound them
    // or select the lower one: shipped bonuses are both 20.
    sim.tables.tuning.democracy_tech_bonus = [10, 20];
    assert_eq!(sim.tech_price(1, t.religion)[0], 132);
    sim.tech[1].tech[t.gov_b] = false;
    assert_eq!(sim.tech_price(1, t.religion)[0], 148);
    sim.tables.tech_tree.roles.democracy_preqs[0] =
        Some([Preq::Of(t.gov_a), Preq::None, Preq::Disabled]);
    assert_eq!(
        sim.tech_price(1, t.religion)[0],
        165,
        "disabled prerequisite refuses"
    );
    sim.tables.tech_tree.roles.democracy_preqs[0] = Some(lower);
    sim.tables.tuning.democracy_tech_bonus = [20, 20];

    let library = sim.tables.build_types[t.library].tree.unwrap();
    for (id, kind) in [
        (t.classical, "age"),
        (t.science1, "epoch"),
        (t.taxation, "plain library tech"),
    ] {
        sim.tables.tech_tree.types[id].cost = [15, 0, 15, 0, 0, 0];
        sim.tables.tech_tree.types[id].where_ = Some(library);
        sim.tech[1].tech[t.gov_a] = false;
        let price = sim.tech_price(1, id);
        sim.tech[1].tech[t.gov_a] = true;
        assert_eq!(sim.tech_price(1, id), price, "{kind} is excluded");
    }
    // Even a mislocated age/epoch is excluded by its kind first.
    for id in [t.classical, t.science1] {
        sim.tables.tech_tree.types[id].where_ = sim.tables.tech_tree.types[t.religion].where_;
        assert_eq!(sim.democracy_tech_discount(1, id), 0);
    }
    sim.tables.tech_tree.roles.taxation_line = vec![t.religion];
    sim.tech[1].power = Some(0xb);
    // A base that distinguishes the two sequential truncations.
    sim.tables.tech_tree.types[t.religion].cost = [13, 0, 13, 0, 0, 0];
    assert_eq!(
        sim.tech_price(1, t.religion)[0],
        57,
        "143 -> 114 -> 57; reversed gives 56"
    );
}

/// `get_cost`'s Silver arm (`docs/AI.md` §99.16). run470's who=1 held
/// Silver on 9955 and paid 382 food and 382 knowledge for the
/// Gunpowder Age, `(100 − 15) × 450 / 100`; this crate charged 450.
#[test]
fn silver_takes_its_percent_off_an_age_and_nothing_else() {
    let (mut sim, t) = sim();
    for x in [t.classical, t.science1, t.taxation] {
        sim.tables.tech_tree.types[x].cost = [45, 0, 0, 45, 0, 0];
    }
    sim.holdings[1].available = [true, true, true, true, true, false];
    let (age, epoch, plain) = (
        sim.tech_price(1, t.classical),
        sim.tech_price(1, t.science1),
        sim.tech_price(1, t.taxation),
    );
    sim.ledgers[1].rare = 1 << (crate::economy::SILVER - crate::economy::BASE_RARE);
    let silvered = sim.tech_price(1, t.classical);
    assert_eq!(silvered[0], age[0] * 85 / 100, "Silver, an age's food");
    assert_eq!(silvered[3], age[3] * 85 / 100, "and its knowledge");
    assert_eq!(sim.tech_price(1, t.science1), epoch, "an epoch is no age");
    assert_eq!(sim.tech_price(1, t.taxation), plain, "nor a plain tech");
}

/// `plan_strategy`'s cheap research tick asks `Leader::can_pay(0)`, the
/// head slot's own test with its escrow flag (`docs/AI.md` §2.3,
/// §99.16): run470's who=1 held 402 food with 93 of it escrowed for
/// the Gunpowder Age at 382, and the original's tick of 9955 bought it.
#[test]
fn the_cheap_tick_prices_an_escrowed_head_against_the_whole_bucket() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.library, 12, 5);
    sim.tables.tech_tree.types[t.science1].cost = [10, 0, 0, 0, 0, 0];
    sim.holdings[1].available = [true, true, true, true, true, false];
    let price = sim.tech_price(1, t.science1)[0];
    let head = |escrow| crate::ai::MakeObject {
        t: t.science1 as i32,
        val: 1000,
        escrow,
        num: 1,
        cat: 8,
        ..crate::ai::MakeObject::EMPTY
    };
    let ready = |sim: &mut Sim, escrow| {
        sim.ai[1].make_list.list[0] = head(escrow);
        sim.ledgers[1].bucket = [price + 1; RESOURCES];
        sim.ledgers[1].escrow = [0; RESOURCES];
        sim.ledgers[1].escrow[0] = 10;
    };
    // An unescrowed head is priced against the bucket less the
    // reservation, and the tick passes it by.
    ready(&mut sim, 0);
    sim.research_tick(1);
    assert!(
        !sim.researching(1, t.science1),
        "price {price} > bucket − escrow"
    );
    // The escrowed head is the reservation's, and the tick buys it.
    ready(&mut sim, 1);
    sim.research_tick(1);
    assert!(
        sim.researching(1, t.science1),
        "the escrowed head is bought"
    );
}

/// `get_cost`'s library-line tail (`docs/AI.md` §99.8). run482's
/// who=1, the British, held Dye on 4576 and researched Empire, the
/// civic epoch at level 1: this crate charged 144 food and the original
/// 108, and the 36 stood in who=1's food bucket to the word 5782.
#[test]
fn a_line_s_rare_and_nations_take_their_percent_off_its_epochs() {
    let (mut sim, t) = sim();
    let civic1 = sim.tables.tech_tree.epochs[Line::Civic.index()][1].unwrap();
    for x in [civic1, t.commerce2, t.military1, t.taxation] {
        sim.tables.tech_tree.types[x].cost = [160, 0, 0, 0, 0, 0];
    }
    sim.holdings[1].available = [true, true, true, true, true, false];
    let price = |sim: &Sim, x| sim.tech_price(1, x)[0];
    let (civic, commerce, military, plain) = (
        price(&sim, civic1),
        price(&sim, t.commerce2),
        price(&sim, t.military1),
        price(&sim, t.taxation),
    );
    // Dye: a quarter off the civic line and nothing else.
    let rare = |sim: &mut Sim, good: usize| {
        sim.ledgers[1].rare = 1 << (good - crate::economy::BASE_RARE);
    };
    rare(&mut sim, 10);
    assert_eq!(price(&sim, civic1), civic * 75 / 100, "Dye, civic");
    assert_eq!(price(&sim, t.commerce2), commerce, "Dye is not commerce's");
    assert_eq!(price(&sim, t.military1), military);
    assert_eq!(price(&sim, t.taxation), plain, "a plain tech takes no line");
    // Silk the commerce line's, Furs the military's.
    rare(&mut sim, 11);
    assert_eq!(price(&sim, t.commerce2), commerce * 75 / 100, "Silk");
    assert_eq!(price(&sim, civic1), civic);
    rare(&mut sim, 0x15);
    assert_eq!(price(&sim, t.military1), military * 75 / 100, "Furs");
    // A nation's step is its own truncation after the rare's: the
    // Turks' third off what Furs left.
    sim.tech[1].power = Some(8);
    assert_eq!(
        price(&sim, t.military1),
        (100 - 33) * (military * 75 / 100) / 100,
        "Furs, then the Turks"
    );
    assert_eq!(
        price(&sim, civic1),
        civic,
        "the Turks are the military line's"
    );
    // Despotism, the military line's second step (run492, `docs/
    // GOLDEN.md` §54): 15 % off what the rare left, and nothing off
    // another line.
    sim.tech[1].power = None;
    sim.ledgers[1].rare = 0;
    sim.tables.tech_tree.roles.despotism_preq = [Some(t.taxation); 3];
    sim.tech[1].tech[t.taxation] = true;
    assert_eq!(
        price(&sim, t.military1),
        military * 85 / 100,
        "Despotism, military"
    );
    assert_eq!(price(&sim, civic1), civic, "Despotism is not civic's");
    assert_eq!(
        price(&sim, t.commerce2),
        commerce,
        "Despotism is not commerce's"
    );
}

#[test]
fn the_slot_is_the_category() {
    assert_eq!(slot_for(0), 10);
    assert_eq!(slot_for(1), 9);
    assert_eq!(slot_for(2), 4);
    assert_eq!(slot_for(3), 8);
}

#[test]
fn the_epoch_quota_loosens_with_difficulty() {
    assert!(epoch_gate(0, 3, 4, 2));
    assert!(!epoch_gate(0, 3, 5, 2));
    assert!(!epoch_gate(1, 3, 4, 3));
    assert!(epoch_gate(2, 3, 8, 3));
    assert!(!epoch_gate(2, 3, 9, 3));
    assert!(!epoch_gate(2, 3, 8, 4));
    for d in [3, 4, 5] {
        assert!(epoch_gate(d, 0, 99, 99));
    }
}

#[test]
fn the_age_pacing_holds_the_ai_behind_the_humans() {
    assert_eq!(age_pace(0, 2, 3, 1), AgePace::Delay((1 + 16) * 225));
    assert_eq!(age_pace(1, 2, 3, 1), AgePace::Delay((1 + 8) * 225));
    assert_eq!(age_pace(0, 3, 3, 1), AgePace::Skip);
    assert_eq!(age_pace(2, 3, 3, 1), AgePace::Delay((1 + 20) * 225));
    assert_eq!(age_pace(2, 4, 3, 1), AgePace::Skip);
    assert_eq!(age_pace(3, 9, 0, 1), AgePace::Unpaced);
}

// ---- research_techs ----

#[test]
fn a_leader_with_no_research_building_offers_nothing() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    sim.research_techs(1);
    assert_eq!(sim.ai[1].make_list.head().t, -1);
}

#[test]
fn a_library_offers_the_library_lines_in_their_category_slots() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.library, 12, 5);
    sim.research_techs(1);
    let list = &sim.ai[1].make_list;
    assert!(list.head().t >= 0, "something was offered");
    assert_eq!(list.list[8].t, t.science1 as i32, "Science is cat 3");
    assert_eq!(list.list[10].t, t.military1 as i32, "Military is cat 0");
}

#[test]
fn the_capital_countdown_lobby_stops_a_leader_that_lost_its_capital() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.library, 12, 5);
    sim.lobby.elimination = 1;
    sim.cities[c].capital = false;
    sim.research_techs(1);
    assert_eq!(sim.ai[1].make_list.head().t, -1);
    sim.cities[c].capital = true;
    sim.research_techs(1);
    assert!(sim.ai[1].make_list.head().t >= 0);
}

#[test]
fn a_tower_tech_is_refused_below_difficulty_two_and_takes_slot_six() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.tower, 12, 5);
    small(&mut sim, 1);
    sim.lobby.difficulty = 1;
    assert_eq!(value(&mut sim, 1, t.fortification), None);
    sim.lobby.difficulty = 2;
    let (hard, slot) = value(&mut sim, 1, t.fortification).expect("offered at 2");
    assert_eq!(slot, 6, "the Tower overrides the category's slot");
    // The ×100 is the Tower's: the same tech at a Temple takes ×4.
    let temple_id = sim.tables.build_types[t.temple].tree.expect("a tree entry");
    sim.tables.tech_tree.types[t.fortification].where_ = Some(temple_id);
    building(&mut sim, 1, t.temple, 16, 5);
    let (soft, slot) = value(&mut sim, 1, t.fortification).expect("offered");
    assert_eq!(slot, 8);
    assert_eq!(hard / 100, soft / 4);
}

#[test]
fn the_taxation_line_takes_a_further_hundred_at_the_temple() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.temple, 12, 5);
    small(&mut sim, 1);
    let (tax, _) = value(&mut sim, 1, t.taxation).expect("Taxation is valued");
    let (rel, _) = value(&mut sim, 1, t.religion).expect("Religion is valued");
    assert!(rel > 0);
    assert_eq!(tax, mul(rel, 100));
}

#[test]
fn the_university_multiplies_by_a_thousand() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.university, 12, 5);
    small(&mut sim, 1);
    let (uni, _) = value(&mut sim, 1, t.agriculture).expect("valued");
    let temple_id = sim.tables.build_types[t.temple].tree.expect("a tree entry");
    sim.tables.tech_tree.types[t.agriculture].where_ = Some(temple_id);
    building(&mut sim, 1, t.temple, 16, 5);
    let (tem, _) = value(&mut sim, 1, t.agriculture).expect("valued");
    assert_eq!(uni / 1000, tem / 4);
}

#[test]
fn the_government_coin_is_one_draw_and_the_dropped_column_is_not_offered() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.senate, 12, 5);
    // `pers.raid < 0` short-circuits the coin: no draw for it, and
    // column 0 is thrown away.
    sim.ai[1].pers.raid = -1;
    let before = sim.rng;
    sim.research_techs(1);
    // Only the first tier's pair is reachable in this tree: one of it
    // is dropped without a draw, the other takes exactly one.
    let mut check = before;
    check.roll();
    assert_eq!(sim.rng, check, "one draw for the survivor, none for a coin");
    let list = offered(&sim, 1);
    assert!(!list.contains(&(t.gov_a as i32)), "column 0 is dropped");
    assert!(list.contains(&(t.gov_b as i32)));
}

#[test]
fn a_raid_zero_leader_spends_one_extra_draw_on_the_coin() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.senate, 12, 5);
    sim.ai[1].pers.raid = 0;
    let before = sim.rng;
    sim.research_techs(1);
    // The coin is inside the per-tech loop: each of the two reachable
    // governments tosses its own, and the survivor takes a second
    // draw. This seed keeps one of the pair, so three draws.
    let mut check = before;
    for _ in 0..3 {
        check.roll();
    }
    assert_eq!(sim.rng, check, "a coin each, then the survivor's draw");
    let list = offered(&sim, 1);
    let a = list.contains(&(t.gov_a as i32));
    let b = list.contains(&(t.gov_b as i32));
    assert!(a ^ b, "this seed's two coins agreed");
}

#[test]
fn a_raiding_leader_drops_the_second_column_without_a_coin() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.senate, 12, 5);
    sim.ai[1].pers.raid = 1;
    let before = sim.rng;
    sim.research_techs(1);
    let mut check = before;
    check.roll();
    assert_eq!(sim.rng, check);
    let list = offered(&sim, 1);
    assert!(list.contains(&(t.gov_a as i32)));
    assert!(!list.contains(&(t.gov_b as i32)));
}

#[test]
fn an_epoch_line_more_than_one_ahead_of_the_lowest_is_dropped() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.library, 12, 5);
    sim.tech[1].epoch[Line::Commerce.index()] = 2;
    assert_eq!(value(&mut sim, 1, t.commerce2), None);
    sim.tech[1].epoch[Line::Commerce.index()] = 1;
    assert!(value(&mut sim, 1, t.commerce2).is_some());
}

#[test]
fn the_line_at_the_lowest_epoch_is_worth_ten_times_as_much() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    // Read the ×10 away from the Library, whose own branch divides the
    // score by the line's level and would confound it.
    let temple_id = sim.tables.build_types[t.temple].tree.expect("a tree entry");
    sim.tables.tech_tree.types[t.science1].where_ = Some(temple_id);
    building(&mut sim, 1, t.temple, 12, 5);
    small(&mut sim, 1);
    sim.lobby.difficulty = 3;
    let (level, _) = value(&mut sim, 1, t.science1).expect("valued");
    sim.tech[1].epoch[Line::Science.index()] = 1;
    let (ahead, _) = value(&mut sim, 1, t.science1).expect("valued");
    assert_eq!(level, mul(ahead, 10));
}

#[test]
fn the_transport_bonus_s_prerequisite_is_worth_thirty_times_on_a_sea_map() {
    // `TRANSPORT_BONUS`'s `find_preq` (`6c71a2`–`6c71c9`): run1161's
    // packet prices Written Word at base 600000 where this crate read
    // 20000 (item 3515, `docs/AI.md` §304).
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    let temple_id = sim.tables.build_types[t.temple].tree.expect("a tree entry");
    sim.tables.tech_tree.types[t.science1].where_ = Some(temple_id);
    building(&mut sim, 1, t.temple, 12, 5);
    small(&mut sim, 1);
    sim.lobby.difficulty = 3;
    sim.world.set_sea_map(2);
    let (plain, _) = value(&mut sim, 1, t.science1).expect("valued");
    sim.tables.tech_tree.roles.transport_preq = Some(t.science1);
    let (needed, _) = value(&mut sim, 1, t.science1).expect("valued");
    assert_eq!(needed, mul(plain, 30));
    // One landmass class or fewer: no multiplier.
    sim.world.set_sea_map(1);
    let (dry, _) = value(&mut sim, 1, t.science1).expect("valued");
    assert_eq!(dry, plain);
}

#[test]
fn the_age_gate_holds_the_ai_a_step_behind_the_human() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.library, 12, 5);
    sim.lobby.difficulty = 0;
    assert_eq!(
        value(&mut sim, 1, t.classical),
        None,
        "the human is still in the Ancient age"
    );
    sim.tech[0].ages = 1;
    assert!(
        value(&mut sim, 1, t.classical).is_some(),
        "an unstamped age passes once the human is there"
    );
}

#[test]
fn a_stamped_age_waits_out_the_human_delay() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.library, 12, 5);
    sim.lobby.difficulty = 0;
    sim.tech[0].ages = 1;
    sim.tech[0].age_stamp[0] = Some(0);
    // `(who + 16) × 225` = 3825 frames for leader 1.
    sim.frame = 3824;
    assert_eq!(value(&mut sim, 1, t.classical), None);
    sim.frame = 3825;
    assert!(value(&mut sim, 1, t.classical).is_some());
}

/// Difficulty 2 waits on the **previous** age's stamp — the original
/// passes `t − 1` where 0 and 1 pass `t` (`0x6c6e94`, called at
/// `0x6c6eac`). The first age has no predecessor, so it is never
/// delayed however early the human reached it, even though the stamp
/// difficulty 0 would read is set and recent.
/// (`docs/audit/2026-08-25-ai.md`, B3-d.)
#[test]
fn difficulty_two_waits_on_the_previous_ages_stamp() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.library, 12, 5);
    sim.tech[0].ages = 1;
    sim.tech[0].age_stamp[0] = Some(0);
    sim.frame = 0;

    // Difficulty 0 reads `age_stamp[0]` and holds for `(1 + 16) × 225`.
    sim.lobby.difficulty = 0;
    assert_eq!(value(&mut sim, 1, t.classical), None);

    // Difficulty 2 reads the age before it — there is none, so the
    // original's call returns −1 and the gate opens at once.
    sim.lobby.difficulty = 2;
    assert!(
        value(&mut sim, 1, t.classical).is_some(),
        "the first age has no previous stamp to wait on"
    );
}

#[test]
fn a_difficulty_three_leader_is_unpaced() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.library, 12, 5);
    sim.lobby.difficulty = 3;
    sim.tech[0].ages = 0;
    assert!(value(&mut sim, 1, t.classical).is_some());
}

#[test]
fn the_knowledge_stockpile_gates_an_epoch() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.library, 12, 5);
    sim.lobby.difficulty = 3;
    sim.tables.tech_tree.types[t.science1].cost[KNOWLEDGE] = 1_000_000;
    sim.ledgers[1].bucket[KNOWLEDGE] = 10;
    assert_eq!(value(&mut sim, 1, t.science1), None);
    sim.ledgers[1].bucket[KNOWLEDGE] = 10_000_000;
    assert!(value(&mut sim, 1, t.science1).is_some());
}

#[test]
fn no_shortage_doubles_the_value() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    building(&mut sim, 1, t.library, 12, 5);
    sim.ai[1].shortages = 0;
    let (easy, _) = value(&mut sim, 1, t.science1).expect("valued");
    sim.ai[1].shortages = 1;
    let (tight, _) = value(&mut sim, 1, t.science1).expect("valued");
    assert_eq!(easy, mul(tight, 2));
}

// ---- produce_tech ----

#[test]
fn produce_tech_queues_at_the_research_building() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    let lib = building(&mut sim, 1, t.library, 12, 5);
    sim.frame = 4;
    assert!(sim.produce_tech(1, t.science1, 1));
    assert_eq!(sim.buildings[lib].queue.items.len(), 1);
    // `Build::queue_up` stamps the leader's `tech_frame` and its
    // category's `tech_cat_frame` (item 1496: run656, block 5).
    assert_eq!(sim.ai[1].tech_frame, 4);
    assert_eq!(sim.ai[1].tech_cat_frame[sim.tech_cat(t.science1)], 4);
    // Already being researched: the original returns "queued" and
    // touches nothing.
    assert!(sim.produce_tech(1, t.science1, 1));
    assert_eq!(sim.buildings[lib].queue.items.len(), 1);
}

#[test]
fn produce_tech_prefers_the_undamaged_and_the_shorter_queue() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    let a = building(&mut sim, 1, t.barracks, 12, 5);
    let b = building(&mut sim, 1, t.barracks, 12, 12);
    sim.ai[1].mil_trainers = vec![a, b];
    let bt_id = sim.tables.build_types[t.barracks]
        .tree
        .expect("the tree entry");
    sim.tables.tech_tree.types[t.military1].where_ = Some(bt_id);
    sim.tables.tech_tree.types[t.science1].where_ = Some(bt_id);
    // `Build+0x24` is `ObjectData::damage`: a damaged trainer scores
    // lower, so the second one wins even though it is second in the
    // list.
    sim.buildings[a].damage = 9;
    assert!(sim.produce_tech(1, t.military1, 1));
    assert_eq!(sim.buildings[b].queue.items.len(), 1);
    assert_eq!(sim.buildings[a].queue.items.len(), 0);
    // With the damage gone the queue length decides, and now `a` is
    // the empty one.
    sim.buildings[a].damage = 0;
    assert!(sim.produce_tech(1, t.science1, 1));
    assert_eq!(
        sim.buildings[a].queue.items.len(),
        1,
        "the empty queue wins"
    );
    assert_eq!(sim.buildings[b].queue.items.len(), 1);
}

#[test]
fn produce_tech_refuses_when_no_building_stands() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    assert!(!sim.produce_tech(1, t.science1, 1));
}

#[test]
fn a_military_trainer_is_searched_in_the_leaders_own_list() {
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    let bar = building(&mut sim, 1, t.barracks, 12, 5);
    let bt_id = sim.tables.build_types[t.barracks]
        .tree
        .expect("the tree entry");
    sim.tables.tech_tree.types[t.military1].where_ = Some(bt_id);
    // Activating it filed it — `Wall::increment_stats`, `docs/AI.md` §29.
    assert_eq!(
        sim.ai[1].mil_trainers,
        vec![bar],
        "an active trainer joins the list when it activates"
    );
    sim.ai[1].mil_trainers.clear();
    assert!(
        !sim.produce_tech(1, t.military1, 1),
        "a trainer not in `mil_trainers` is invisible"
    );
    sim.ai[1].mil_trainers.push(bar);
    assert!(sim.produce_tech(1, t.military1, 1));
    assert_eq!(sim.buildings[bar].queue.items.len(), 1);
}

#[test]
fn produce_tech_queues_a_unit_s_research_on_the_unit_queue() {
    // `produce_tech:262` ends in `Build::queue_up(t, escrow)`, the one
    // queue for every kind, so a unit type the leader does not own is
    // researched as the unit queue's research job (`docs/AI.md` §56).
    // `queue_tech` takes only a technology: this refused until item 545.
    let (mut sim, t) = sim();
    city(&mut sim, &t, 1, 5, 5);
    let bar = building(&mut sim, 1, t.barracks, 12, 5);
    let rec = sim.add_unit_type(crate::UnitType {
        tree: Some(t.phalanx),
        ..crate::UnitType::default()
    });
    assert_eq!(sim.type_avail(1, t.phalanx), tech::NOT_AVAILABLE);
    sim.gain_tech(1, t.classical);
    assert_eq!(sim.type_avail(1, t.phalanx), tech::RESEARCHABLE);
    assert!(sim.produce_tech(1, t.phalanx, 1));
    assert_eq!(sim.buildings[bar].queue.items.len(), 1);
    assert_eq!(sim.muster[1].queued_by_type[rec], 1);
    // Being researched: "queued", and nothing more.
    assert!(sim.produce_tech(1, t.phalanx, 1));
    assert_eq!(sim.buildings[bar].queue.items.len(), 1);
}
