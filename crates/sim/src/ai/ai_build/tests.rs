use super::*;
use crate::build::BuildType;
use crate::tech::{TechTree, TypeDef};
use crate::world::{Terrain, UNITS_PER_TILE};
use crate::{Cell, Pos, Tuning, World};

const TILE: i32 = UNITS_PER_TILE;

fn tile_pos(tx: i32, ty: i32) -> Pos {
    Pos::new(tx * TILE + TILE / 2, ty * TILE + TILE / 2)
}

struct Types {
    village: usize,
    farm: usize,
    barracks: usize,
    library: usize,
    market: usize,
    temple: usize,
    tower: usize,
    granary: usize,
    silo: usize,
    wonder: usize,
}

fn bt(ident: Ident, flag: &str, xs: i32, ys: i32) -> BuildType {
    BuildType {
        ident,
        x_size: xs,
        y_size: ys,
        flags: flags::parse(flag),
        job_time: 100,
        hits: 500,
        ..BuildType::default()
    }
}

/// A flat land world, two leaders, every type in the tree and available.
/// The leader's nation is the Maya — nation 0 is the Aztecs, whose ×4 on
/// a Barracks overflows the listing arithmetic (see
/// `the_original_s_own_thirty_two_bit_product_wraps`).
fn sim() -> (Sim, Types) {
    let mut w = World::new(24, 24);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(23, 23));
    let mut sim = Sim::new(Tuning::RON, w, 2);
    for l in &mut sim.economy.ledgers {
        l.bucket = [10_000; RESOURCES];
        l.income = [100 * 16; RESOURCES];
        l.cap = [1000 * 16; RESOURCES];
    }
    let village = sim.add_build_type(bt(Ident::Village, "ean", 7, 7));
    let mut farm = bt(Ident::Farm, "gda", 4, 4);
    farm.flags |= flags::FLAT;
    let farm = sim.add_build_type(farm);
    // The derived flag `UnitType::init` sets on every building that
    // trains a military unit (`build::init_derived_flags`): the valuation
    // reads it, not the ident (item 1496).
    let mut barracks = bt(Ident::Barracks, "ean", 4, 4);
    barracks.flags |= flags::MILITARY_TRAINER;
    let barracks = sim.add_build_type(barracks);
    let library = sim.add_build_type(bt(Ident::Library, "jam", 5, 5));
    let market = sim.add_build_type(bt(Ident::Market, "jam", 4, 4));
    let temple = sim.add_build_type(bt(Ident::Temple, "jam", 4, 4));
    let mut tower = bt(Ident::Tower, "ean", 2, 2);
    tower.attack = 12;
    let tower = sim.add_build_type(tower);
    let granary = sim.add_build_type(bt(Ident::Granary, "ja", 5, 5));
    let silo = sim.add_build_type(bt(Ident::MissileSilo, "ean", 4, 4));
    let mut wonder = bt(Ident::Wonder, "ean", 5, 5);
    wonder.wonder = true;
    wonder.wonder_val = 1;
    let wonder = sim.add_build_type(wonder);

    // The tree: six goods in `goodrules.xml` order, then one entry per
    // building record.
    let mut tree = TechTree::new().with_tuning(&Tuning::RON);
    for name in ["Food", "Timber", "Wealth", "Knowledge", "Metal", "Oil"] {
        let mut d = TypeDef::good(name);
        d.tribe_mask = u32::MAX;
        tree.add(d);
    }
    let recs = [
        village, farm, barracks, library, market, temple, tower, granary, silo, wonder,
    ];
    let names = [
        "Village", "Farm", "Barracks", "Library", "Market", "Temple", "Tower", "Granary", "Silo",
        "Wonder",
    ];
    let ids: Vec<_> = names
        .iter()
        .map(|name| {
            let mut d = TypeDef::building(name);
            d.tribe_mask = u32::MAX;
            tree.add(d)
        })
        .collect();
    sim.set_tech_tree(tree);
    for (rec, id) in recs.iter().zip(ids.iter()) {
        sim.tables.build_types[*rec].tree = Some(*id);
    }
    for t in &mut sim.tech {
        t.tribe = crate::ai::tribe::MAYA;
    }
    (
        sim,
        Types {
            village,
            farm,
            barracks,
            library,
            market,
            temple,
            tower,
            granary,
            silo,
            wonder,
        },
    )
}

fn finish(sim: &mut Sim, b: usize) {
    let mut guard = 0;
    while !sim.buildings[b].active && sim.buildings[b].alive {
        sim.do_construct(b, 1_000_000);
        guard += 1;
        assert!(guard < 10, "a million a frame finishes anything");
    }
}

/// Founds a finished city of `who` and gives its AI record a generous
/// picture: room for anything, four free citizens, terrain everywhere.
fn city(sim: &mut Sim, t: &Types, who: Player, tx: i32, ty: i32) -> usize {
    // `init_build` rather than `place_building`: the city-count limit is
    // a tech rule and these tests are about the listing, not about it.
    let b = sim.init_build(who, t.village, tile_pos(tx, ty), false);
    finish(sim, b);
    let c = sim.buildings[b].city.expect("a finished city has a record");
    let w = who as usize;
    let regions = sim.world.region_count();
    let players = sim.territory.players.len();
    sim.ai.ai[w].census.resize(regions, players);
    sim.ai.ai[w]
        .city_ai
        .resize(sim.cities.cities.len(), crate::ai::CityAi::default());
    sim.ai.ai[w].city_ai[c] = crate::ai::CityAi {
        free: 4,
        busy: 0,
        gatherers: 0,
        peasant_dist: 1,
        in_port: 0,
        ocean: 0,
        land: 40,
        filled: 1,
        dock_tile: 0,
        space: [8, 8, 8],
        ter: [2; RESOURCES],
        ..Default::default()
    };
    sim.ai.ai[w].census.pop = 10;
    if let Some(r) = sim.cities.cities[c].reg {
        let i = r as usize;
        sim.ai.ai[w].census.reg_free_peasants[i] = 4;
        sim.ai.ai[w].census.reg_cities[i] = 1;
    }
    c
}

/// One type's value in one city, straight out of the pipeline — before
/// `check_income` and `make_me`.
fn value(sim: &mut Sim, who: Player, c: usize, rec: usize) -> Option<Listing> {
    let f = sim.city_facts(who, c, 0)?;
    let pass = u32::from(sim.tables.build_types[rec].has(flags::GATHERER));
    sim.building_value(who, &f, rec, pass)
}

fn listed(sim: &Sim, who: Player, rec: usize) -> Option<MakeObject> {
    let id = sim.tables.build_types[rec].tree.expect("in the tree") as i32;
    sim.ai.ai[who as usize]
        .make_list
        .list
        .iter()
        .find(|m| m.t == id)
        .copied()
}

// ------------------------------------------------------------------
// The sweep end to end
// ------------------------------------------------------------------

#[test]
fn the_sweep_lists_what_a_first_city_can_build_and_nothing_else() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    assert_eq!(sim.cities.cities[c].race, Some(0));
    sim.create_buildings(0);
    // The city type itself is never listed: `is_city` is the first gate.
    assert!(listed(&sim, 0, t.village).is_none());
    // A barracks is, in the military category.
    let b = listed(&sim, 0, t.barracks).expect("a barracks is listed");
    assert_eq!(b.cat, 7);
    assert_eq!(b.escrow, 1, "the first trainer is worth saving for");
    // A library is a civic entry in the default category.
    let l = listed(&sim, 0, t.library).expect("a library is listed");
    assert_eq!(l.cat, 8);
    // A farm is a gather entry in the economy category, and carries its
    // city.
    let f = listed(&sim, 0, t.farm).expect("a farm is listed");
    assert_eq!(f.cat, 4);
    assert_eq!(f.city, c as i32);
    assert_eq!(f.num, 1);
    assert_eq!(f.o, -1, "the site is chosen at purchase time");
    // The farm out-values everything here, so it heads the list too.
    assert_eq!(sim.ai.ai[0].make_list.list[0].t, f.t);
}

#[test]
fn nothing_is_listed_for_a_city_still_assimilating() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    sim.cities.cities[c].race = Some(1);
    sim.create_buildings(0);
    assert_eq!(sim.ai.ai[0].make_list, crate::ai::MakeList::new());
}

#[test]
fn the_original_s_own_thirty_two_bit_product_wraps() {
    // `imull %edi, %eax; cltd; and edx,0xff; add; sar $8` at 0x6c3f99:
    // the listing's value is a **signed** 32-bit product. An Aztec's
    // first Barracks is 1000 × 3 × 400 × 100 × 4 = 480,000,000, and
    // 480,000,000 × 0x100 does not fit, so the entry that should be the
    // best in the game comes out negative and loses its own slot.
    let (mut sim, t) = sim();
    for x in &mut sim.tech {
        x.tribe = crate::ai::tribe::AZTECS;
    }
    let c = city(&mut sim, &t, 0, 40, 40);
    let l = value(&mut sim, 0, c, t.barracks).expect("valued");
    assert_eq!(l.val, 480_000_000);
    sim.create_buildings(0);
    assert!(
        listed(&sim, 0, t.barracks).is_none(),
        "the product wraps negative and `make_me` refuses it"
    );
    // Without the nation's ×4 the same entry survives, wrapped but
    // positive: 120,000,000 × 256 mod 2^32 = 655,228,928, over 256.
    for x in &mut sim.tech {
        x.tribe = crate::ai::tribe::MAYA;
    }
    sim.ai.ai[0].make_list = crate::ai::MakeList::new();
    sim.create_buildings(0);
    let b = listed(&sim, 0, t.barracks).expect("listed");
    assert_eq!(b.val, 2_559_488);
}

// ------------------------------------------------------------------
// The gates
// ------------------------------------------------------------------

#[test]
fn a_university_and_a_second_library_need_three_buildings_in_the_city() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    assert_eq!(sim.num_buildings(c), 1);
    assert!(
        value(&mut sim, 0, c, t.library).is_some(),
        "a first library is exempt"
    );
    let b = sim
        .place_building(0, t.library, tile_pos(48, 40))
        .expect("a library places");
    finish(&mut sim, b);
    assert_eq!(sim.num_buildings(c), 2);
    assert!(
        value(&mut sim, 0, c, t.library).is_none(),
        "a second library waits for a third building"
    );
}

#[test]
fn a_temple_is_the_capital_s_alone_on_an_easy_difficulty() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    sim.ai.lobby.difficulty = 0;
    sim.cities.cities[c].capital = false;
    assert!(value(&mut sim, 0, c, t.temple).is_none());
    sim.cities.cities[c].capital = true;
    let l = value(&mut sim, 0, c, t.temple).expect("the capital wants a temple");
    assert_eq!(l.escrow, 1, "a temple is always worth saving for");
    // 1000 × 10000 for a first temple, then the open-ground ×4/3.
    assert_eq!(l.val, 1000 * 10000 * 4 / 3);
    // On a hard difficulty it is wanted anywhere.
    sim.cities.cities[c].capital = false;
    sim.ai.lobby.difficulty = 3;
    assert!(value(&mut sim, 0, c, t.temple).is_some());
}

#[test]
fn the_capital_countdown_leaves_only_military_trainers() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    sim.ai.lobby.elimination = 1;
    sim.cities.cities[c].capital = false;
    // With no capital of my own and nobody else holding one, the
    // original's `find_capital` still answers "me", so nothing changes.
    assert!(value(&mut sim, 0, c, t.market).is_some());
    // Player 1 holds a city that was mine.
    let c2 = city(&mut sim, &t, 1, 80, 40);
    sim.cities.cities[c2].was_capital |= 1;
    assert!(value(&mut sim, 0, c, t.barracks).is_some());
    assert!(value(&mut sim, 0, c, t.market).is_none());
    assert!(value(&mut sim, 0, c, t.farm).is_none());
    // And a capital of my own puts everything back.
    sim.cities.cities[c].capital = true;
    assert!(value(&mut sim, 0, c, t.market).is_some());
}

#[test]
fn a_one_per_city_type_is_not_offered_twice() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    assert!(sim.tables.build_types[t.market].has(flags::ONE_PER_CITY));
    assert!(value(&mut sim, 0, c, t.market).is_some());
    let b = sim
        .place_building(0, t.market, tile_pos(48, 40))
        .expect("a market places");
    assert_eq!(sim.buildings[b].city, Some(c));
    assert!(!sim.buildings[b].active, "an unfinished site still counts");
    assert!(value(&mut sim, 0, c, t.market).is_none());
}

#[test]
fn at_most_three_of_a_plain_kind() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    // The Silo is not a military trainer, not a gather building, not an
    // enhancer, has no attack and is not the Temple: the leader may hold
    // three of it and no more.
    assert!(value(&mut sim, 0, c, t.silo).is_some());
    for i in 0..2 {
        let b = sim
            .place_building(0, t.silo, tile_pos(48, 32 + i * 6))
            .expect("a silo places");
        finish(&mut sim, b);
    }
    assert_eq!(sim.buildings_of_line(0, t.silo), 2);
    assert!(value(&mut sim, 0, c, t.silo).is_some(), "two is still fine");
    let b = sim
        .place_building(0, t.silo, tile_pos(48, 44))
        .expect("a third silo places");
    finish(&mut sim, b);
    assert_eq!(sim.buildings_of_line(0, t.silo), 3);
    assert!(value(&mut sim, 0, c, t.silo).is_none());
}

#[test]
fn a_farm_stops_at_the_city_s_farm_limit() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    let limit = sim.farm_limit(c);
    assert!(limit > 0);
    for i in 0..limit {
        let b = sim
            .place_building(0, t.farm, tile_pos(48, 30 + i * 5))
            .expect("a farm places");
        finish(&mut sim, b);
    }
    assert_eq!(sim.city_count_exact(c, t.farm, false), limit);
    assert!(value(&mut sim, 0, c, t.farm).is_none());
}

// ------------------------------------------------------------------
// The families
// ------------------------------------------------------------------

#[test]
fn an_enhancer_needs_three_filled_slots_of_its_own_good() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    assert!(
        value(&mut sim, 0, c, t.granary).is_none(),
        "no farms, no granary"
    );
    // **A filled slot is a real gatherer since item 442.** This used
    // to push a bare `0` onto `gatherers` and lean on
    // `City::count_gather_slots` reading the chain's raw length; the
    // original subtracts `BuildData::num_gatherers`, which asks each
    // chain member whether it is actually gathering here
    // (`docs/AI.md` §53, §51.1). So the farms get citizens with
    // gather orders, which is what the arithmetic below was always
    // describing.
    let citizen = sim.add_unit_type(crate::UnitType {
        worker: crate::orders::Worker::Citizen,
        hits: 40,
        ..crate::UnitType::default()
    });
    for i in 0..3 {
        let b = sim
            .place_building(0, t.farm, tile_pos(48 + i * 5, 40))
            .expect("a farm places");
        finish(&mut sim, b);
        sim.buildings[b].gather_max = Some(1);
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut u = crate::Unit::new(0, index, tile_pos(48 + i * 5, 40), 40);
        u.ty = Some(citizen);
        u.orders.push_back(crate::orders::Order {
            flags: crate::orders::flag::ACTION,
            body: crate::orders::Body::Gather(crate::orders::GatherOrder {
                building: b,
                tile: None,
                wait: 0,
                goto_build: false,
                dist_mod: 0,
                been_there: true,
            }),
        });
        let u = sim.add_unit(u);
        sim.buildings[b].gatherers.push(u);
    }
    assert_eq!(sim.cities.cities[c].members.len(), 3);
    let g = value(&mut sim, 0, c, t.granary).expect("three filled slots want a granary");
    assert_eq!(g.cat, 4, "an enhancer is an economy entry");
    // v = 1000 (level) × 4/3 (open ground) × n² with n = 3 filled slots;
    // the divisor for a plain type is one.
    assert_eq!(g.val, 1000 * 4 / 3 * 9);
}

#[test]
fn a_market_takes_the_civic_multiplier_and_the_infrastructure_mod() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    // A sixteenth of the usual infrastructure weight, so the ×10000
    // stays inside a signed thirty-two-bit product.
    sim.ai.ai[0].infra_mod = 0x10;
    let m = value(&mut sim, 0, c, t.market).expect("a market is listed");
    // m = 10 + 1, doubled for the first of its kind, then >>2 for a
    // one-city empire = 5; v = 1000 × 5, then × infra × 10000 / 256, then
    // the open-ground ×4/3.
    assert_eq!(m.val, 1000 * 4 / 3 * 5 * 0x10 * 10000 / 256);
    assert_eq!(m.cat, 8);
}

#[test]
fn a_tower_is_dropped_on_the_easiest_difficulty_and_wanted_on_the_hardest() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    sim.ai.lobby.difficulty = 0;
    assert!(value(&mut sim, 0, c, t.tower).is_none());
    sim.ai.lobby.difficulty = 4;
    let tw = value(&mut sim, 0, c, t.tower).expect("a hard AI wants a tower");
    assert_eq!(tw.escrow, 1, "the first tower is worth saving for");
    // A raised defence mod lifts the easy-difficulty drop entirely.
    sim.ai.lobby.difficulty = 0;
    sim.ai.ai[0].defense_mod = 0x200;
    assert!(value(&mut sim, 0, c, t.tower).is_some());
}

/// **A Redoubt is a training building through the Fort it is `FROM`**
/// (`BuildTypeData::is_training_building@00639de0` reads the bit on the
/// root; item 1738, run781's packet: Great Sahara's Persian Redoubt in
/// `1/2018` is divided by `(reg + (queued + 2·have)·2)·2 + 1` = 7, not
/// by the fort's `(queued + 2·have)·4 + 1`). The derived bit is on the
/// Fort alone, so this crate's old scan of the record's own `WHERE` read
/// false. Made to fail once with that scan put back.
#[test]
fn a_redoubt_is_a_training_building_through_the_fort_it_comes_from() {
    let (mut sim, t) = sim();
    let mut root = bt(Ident::Fort, "ean", 4, 4);
    root.flags |= flags::TRAINS;
    let root = sim.add_build_type(root);
    let mut redoubt = bt(Ident::Other, "ean", 4, 4);
    redoubt.from = Some(root);
    let redoubt = sim.add_build_type(redoubt);
    assert!(
        !sim.tables.build_types[redoubt].has(flags::TRAINS),
        "the bit is on the root alone"
    );
    assert!(sim.is_training_building(redoubt));
    assert!(sim.is_training_building(root));
    assert!(!sim.is_training_building(t.tower), "a tower trains nothing");
}

/// **A city a rival's border touches is worth ten times its tower**
/// (`create_buildings:821`–`841`, `CityData.bordering`, `docs/AI.md`
/// §167): Great Sahara's Persian city `1/2018`, `bordering` 3, is
/// offered a Bunker at 80000 against 8000 in a city no border touches.
/// Made to fail once with the multiplier removed.
#[test]
fn a_tower_city_a_rival_s_border_touches_is_worth_ten_times() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    sim.ai.lobby.difficulty = 4;
    let plain = value(&mut sim, 0, c, t.tower).expect("a hard AI wants a tower");
    sim.ai.ai[0].city_ai[c].bordering = 3;
    let touched = value(&mut sim, 0, c, t.tower).expect("still wanted");
    assert!(
        (touched.val - plain.val * 10).abs() <= 10,
        "{} against {}",
        touched.val,
        plain.val
    );
}

/// **A bordered city wants its temple a hundred times more**
/// (`create_buildings:324`–`326`, `docs/AI.md` §167). No capture holds
/// the arm — its mutation failed nothing on the walk — so this does.
/// Made to fail once with the multiplier removed.
#[test]
fn a_temple_in_a_bordered_city_is_worth_a_hundred_times() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    sim.ai.lobby.difficulty = 4;
    // A wanted wonder takes a hundredth first, so the ×10000 stays
    // inside a signed thirty-two-bit product.
    sim.ai.ai[0].wonder_mod = 1;
    let plain = value(&mut sim, 0, c, t.temple).expect("a temple is listed");
    sim.ai.ai[0].city_ai[c].bordering = 3;
    let touched = value(&mut sim, 0, c, t.temple).expect("still listed");
    assert!(
        (touched.val - plain.val * 100).abs() <= 100,
        "{} against {}",
        touched.val,
        plain.val
    );
}

/// **A city a marked cell lies within four cells of** (`city_flags &
/// 0x1000`, `compute_reg_territory@006b0bb0:612`) wants its temple ten
/// times more, twice while `no_heal` (`create_buildings:328`).
#[test]
fn a_temple_near_a_marked_border_cell_is_worth_ten_times() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    sim.ai.lobby.difficulty = 4;
    sim.ai.ai[0].wonder_mod = 1;
    let plain = value(&mut sim, 0, c, t.temple).expect("a temple is listed");
    sim.ai.ai[0].city_ai[c].near_border = true;
    let near = value(&mut sim, 0, c, t.temple).expect("still listed");
    assert!(
        (near.val - plain.val * 10).abs() <= 10,
        "{} / {}",
        near.val,
        plain.val
    );
    sim.cities.cities[c].no_heal = true;
    let hurt = value(&mut sim, 0, c, t.temple).expect("still listed");
    // A `no_heal` city is also worth a hundredth outside the arm.
    assert!(
        (hurt.val * 100 - plain.val * 2).abs() <= 200,
        "{} / {}",
        hurt.val,
        plain.val
    );
}

#[test]
fn the_gather_multiplier_saturates_at_one() {
    // `cmpl %esi, -0x14(%ebp); cmovll` at 0x6c2762 is a minimum against
    // 0x100, so the farm's value never exceeds its own base however many
    // doublings apply.
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    sim.ai.ai[0].econ = [1; RESOURCES];
    sim.ai.ai[0].worst_good = 0;
    sim.ai.ai[0].best_good = 5;
    let f = value(&mut sim, 0, c, t.farm).expect("a farm is listed");
    assert_eq!(f.mult, 2, "a gather type discounts at half the rate");
    // base = ((pop·7000)/cities + level·3000 + open·4000) × infra/256,
    // ×5 for the first farm and ×10 for a city with no members yet, then
    // the tail's ×4/3 and a divisor of one.
    assert!(
        sim.cities.cities[c].capital,
        "the first city is the capital"
    );
    let open = 40 - 1;
    let base = (10 * 7000 + 3000 + open * 4000) * 5 * 10 / 2;
    assert_eq!(f.val, base * 4 / 3);
    // Better terrain raises the multiplier and changes nothing at all:
    // it was already over the cap.
    sim.ai.ai[0].city_ai[c].ter = [5; RESOURCES];
    let g = value(&mut sim, 0, c, t.farm).expect("still listed");
    assert_eq!(g.val, f.val);
}

/// **A gather offer is always escrowed** (item 890, listing
/// `006c3cbb`..`006c3db0`): every exit of the head test writes 1 to the
/// offer's escrow — the head out-valued or empty, the head affordable,
/// the flag, the goods walk. East Indies' 23181 listed a Farm as the
/// new head at `escrow 1` in the original and 0 here, because this
/// crate escrowed only when the test passed. The worst good's rate is
/// raised first, so `gather_value`'s own escrow arm stays shut.
#[test]
fn a_gather_offer_is_escrowed_on_every_exit_of_the_head_test() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    sim.ai.ai[0].rate = [1_000_000; RESOURCES];
    sim.ai.ai[0].make_list.clear();
    let f = value(&mut sim, 0, c, t.farm).expect("a farm is listed");
    assert_eq!(f.escrow, 1, "an empty head: the first compare fails");
    let mut head = MakeObject::EMPTY;
    head.t = sim.tables.build_types[t.barracks]
        .tree
        .expect("in the tree") as i32;
    head.val = i32::MAX;
    sim.ai.ai[0].make_list.list[0] = head;
    let g = value(&mut sim, 0, c, t.farm).expect("still listed");
    assert_eq!(g.escrow, 1, "a richer, affordable head");
}

#[test]
fn a_gather_type_waits_until_its_slots_are_three_quarters_full() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    sim.economy.ledgers[0].gather_slots[0] = 8;
    sim.ai.ai[0].census.filled_gather_slots[0] = 5;
    assert!(value(&mut sim, 0, c, t.farm).is_none(), "5 < 8·3/4");
    sim.ai.ai[0].census.filled_gather_slots[0] = 6;
    assert!(value(&mut sim, 0, c, t.farm).is_some(), "6 == 8·3/4");
}

#[test]
fn a_gather_type_needs_terrain_or_it_is_not_listed() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    sim.ai.ai[0].city_ai[c].ter = [0; RESOURCES];
    assert!(value(&mut sim, 0, c, t.farm).is_none());
}

// ------------------------------------------------------------------
// The wonder branch and its draws
// ------------------------------------------------------------------

#[test]
fn the_wonder_branch_draws_twice_per_city_and_wonder() {
    let (mut sim, t) = sim();
    let _c = city(&mut sim, &t, 0, 40, 40);
    sim.ai.lobby.difficulty = 3;
    let before = sim.rng;
    sim.create_buildings(0);
    let mut probe = before;
    let mut n = 0;
    while probe != sim.rng {
        probe.roll();
        n += 1;
        assert!(n < 20, "the wonder branch draws more than it should");
    }
    assert_eq!(n, 2, "one wonder type, one city, one pass");
    assert!(listed(&sim, 0, t.wonder).is_some());
}

#[test]
fn two_cities_draw_twice_each() {
    let (mut sim, t) = sim();
    let _a = city(&mut sim, &t, 0, 40, 40);
    let _b = city(&mut sim, &t, 0, 80, 40);
    sim.ai.lobby.difficulty = 3;
    let before = sim.rng;
    sim.create_buildings(0);
    let mut probe = before;
    let mut n = 0;
    while probe != sim.rng {
        probe.roll();
        n += 1;
        assert!(n < 20, "too many draws");
    }
    assert_eq!(n, 4);
}

#[test]
fn a_wonder_already_standing_in_the_city_blocks_the_next() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    sim.ai.lobby.difficulty = 3;
    let b = sim
        .place_building(0, t.wonder, tile_pos(48, 40))
        .expect("a wonder places");
    finish(&mut sim, b);
    assert_eq!(sim.buildings[b].city, Some(c));
    assert_eq!(sim.num_wonders(c, true), 1);
    let before = sim.rng;
    sim.create_buildings(0);
    assert_eq!(sim.rng, before, "the branch is refused before it draws");
    assert!(listed(&sim, 0, t.wonder).is_none());
}

/// `CityData::num_wonders` counts a wonder **site** (`docs/AI.md` §70):
/// an unfinished wonder of one type shuts its city out of every other
/// wonder, while a second city still values them. Great Lakes 15384:
/// who=1's Pyramids site took the Hanging Gardens and the Colossus out
/// of its city, two (city, wonder) pairs and four draws. The Silo stands
/// in for the second wonder type, so its own site drops it everywhere
/// through the ally walk and the count is the only thing left to test.
#[test]
fn an_unfinished_wonder_site_shuts_its_city_out_of_the_other_wonders() {
    let (mut sim, t) = sim();
    sim.tables.build_types[t.silo].wonder = true;
    let a = city(&mut sim, &t, 0, 40, 40);
    let _b = city(&mut sim, &t, 0, 80, 40);
    sim.ai.lobby.difficulty = 3;
    let site = sim
        .place_building(0, t.silo, tile_pos(48, 40))
        .expect("the second wonder's site places");
    assert!(!sim.buildings[site].active, "a site, never finished");
    assert_eq!(sim.buildings[site].city, Some(a));
    assert_eq!(sim.num_wonders(a, true), 1, "the site counts");
    let before = sim.rng;
    sim.create_buildings(0);
    let mut probe = before;
    let mut n = 0;
    while probe != sim.rng {
        probe.roll();
        n += 1;
        assert!(n < 20, "too many draws");
    }
    assert_eq!(n, 2, "the other city's wonder alone");
}

/// **A site of one wonder prices every other wonder a step up**
/// (`docs/COSTS.md`, "A wonder is ramped by every wonder", item 890). `get_cost`'s wonder arm counts every
/// wonder the leader holds or has a site of, so on East Indies' frame
/// 20781 who=1's Pyramids site put the Mausoleum and the Colossus at
/// 260 wealth against a purse of 208: `check_income` answered 0, and
/// both wonders went on the make list at `val 0` in the original where
/// this crate, counting only a wonder's own type, listed them at 486 and
/// 398 and went to the market for them. A Barracks is not a wonder and
/// does not move.
#[test]
fn another_wonder_s_site_prices_a_wonder_out_of_the_purse() {
    use crate::cost::{Kind, Price, RampClass};
    use crate::economy::Resource;
    let (mut sim, t) = sim();
    let wealth = Resource::Wealth as usize;
    sim.tables.build_types[t.silo].wonder = true;
    sim.tables.build_types[t.wonder].price = Price {
        kind: Kind::Building,
        class: RampClass::Building,
        ..Price::free()
            .with_base(Resource::Wealth, 20)
            .with_support(Resource::Wealth, 60)
    };
    sim.tables.build_types[t.barracks].price = Price {
        kind: Kind::Building,
        class: RampClass::Building,
        ..Price::free()
            .with_base(Resource::Wealth, 20)
            .with_support(Resource::Wealth, 60)
    };
    let a = city(&mut sim, &t, 0, 40, 40);
    let _b = city(&mut sim, &t, 0, 80, 40);
    sim.ai.lobby.difficulty = 3;
    sim.economy.ledgers[0].bucket[wealth] = 208;
    assert_eq!(sim.building_price(0, t.wonder)[wealth], 200);
    pass_draws(&mut sim, 0);
    let before = listed(&sim, 0, t.wonder).expect("listed with no site");
    assert!(before.val > 0, "affordable: {before:?}");
    let tree = sim.tables.build_types[t.wonder].tree.expect("in the tree");
    assert!(sim.check_income(0, tree, 4 << 8, Some(a), false, -1, 1, 0) > 0);
    let site = sim
        .place_building(0, t.silo, tile_pos(48, 40))
        .expect("the other wonder's site places");
    assert_eq!(sim.buildings[site].city, Some(a));
    sim.economy.ledgers[0].bucket[wealth] = 208;
    assert_eq!(
        sim.building_price(0, t.wonder)[wealth],
        260,
        "one wonder held: 60 x 2 / 2 on top"
    );
    assert_eq!(sim.building_price(0, t.barracks)[wealth], 200);
    sim.ai.ai[0].make_list.clear();
    pass_draws(&mut sim, 0);
    assert_eq!(
        sim.check_income(0, tree, 4 << 8, Some(a), false, -1, 1, 0),
        0,
        "unaffordable with no escrow: check_income answers 0"
    );
    assert!(
        listed(&sim, 0, t.wonder).is_none(),
        "a val-0 offer loses every slot the fuller list holds"
    );
}

/// **A city is priced by every city of the line** (item 1286):
/// `get_cost`'s `is_city` arm sums `num_buildings` and `num_queued` over
/// the Small, Large and Major City. On Great Sahara at Toughest's frame
/// 8377 who=1 held one Small City and two Large ones, and the original
/// priced its fourth city as the fourth: the site was quartered as
/// unaffordable and the market bought food for it, where this crate,
/// counting Small Cities alone, bought it on the spot. A Barracks still
/// counts its own type.
#[test]
fn a_city_is_priced_by_every_city_of_the_line() {
    use crate::cost::{Kind, Price, RampClass};
    use crate::economy::Resource;
    let (mut sim, t) = sim();
    let food = Resource::Food as usize;
    let mut town = bt(Ident::Town, "ean", 7, 7);
    town.from = Some(t.village);
    let town = sim.add_build_type(town);
    let mut metropolis = bt(Ident::Metropolis, "ean", 7, 7);
    metropolis.from = Some(town);
    let metropolis = sim.add_build_type(metropolis);
    let priced = Price {
        kind: Kind::Building,
        class: RampClass::Building,
        ..Price::free()
            .with_base(Resource::Food, 1)
            .with_support(Resource::Food, 50)
    };
    sim.tables.build_types[t.village].price = priced.clone();
    sim.tables.build_types[t.barracks].price = priced;
    let _a = city(&mut sim, &t, 0, 40, 40);
    let b = city(&mut sim, &t, 0, 80, 40);
    let two_small = sim.building_price(0, t.village)[food];
    let barracks = sim.building_price(0, t.barracks)[food];
    sim.buildings[b].ty = Some(town);
    assert_eq!(
        sim.building_price(0, t.village)[food],
        two_small,
        "a Large City counts as a city"
    );
    let one_small = {
        sim.buildings[b].alive = false;
        let p = sim.building_price(0, t.village)[food];
        sim.buildings[b].alive = true;
        p
    };
    assert!(one_small < two_small, "{one_small} against {two_small}");
    // The Major City is the line's third pair (`+0x5562`, `+0x5d62`); no
    // capture's leader holds one when it prices a city.
    sim.buildings[b].ty = Some(metropolis);
    assert_eq!(
        sim.building_price(0, t.village)[food],
        two_small,
        "a Major City counts as a city"
    );
    assert_eq!(sim.building_price(0, t.barracks)[food], barracks);
}

/// Counts the sync-stream draws one `create_buildings` pass takes.
fn pass_draws(sim: &mut Sim, who: Player) -> usize {
    let before = sim.rng;
    sim.create_buildings(who);
    let mut probe = before;
    let mut n = 0;
    while probe != sim.rng {
        probe.roll();
        n += 1;
        assert!(n < 40, "too many draws");
    }
    n
}

/// `docs/AI.md` §75: an easy AI outside a wonder victory values no
/// wonder once one of its own has activated, because the arm's last
/// gate is `wonder_mark == 0` and `Wonders::init_wonder` raised it.
/// Great Lakes 17181: who=1's Pyramids activated on 17084, and the
/// original spends none of the three (city, wonder) pairs this crate
/// spent. A hard AI does not read the mark.
#[test]
fn an_easy_ai_wants_no_second_wonder_once_its_first_stands() {
    let (mut sim, t) = sim();
    sim.tables.build_types[t.silo].wonder = true;
    let _a = city(&mut sim, &t, 0, 40, 40);
    let _b = city(&mut sim, &t, 0, 80, 40);
    sim.ai.lobby.difficulty = 1;
    let first = sim
        .place_building(0, t.silo, tile_pos(48, 40))
        .expect("the first wonder places");
    finish(&mut sim, first);
    assert!(pass_draws(&mut sim, 0) > 0, "no mark yet: the arm draws");
    sim.note_wonders();
    assert_eq!(sim.ai.ai[0].census.wonder_mark, 1);
    assert_eq!(pass_draws(&mut sim, 0), 0, "the mark closes the arm");
    sim.ai.lobby.difficulty = 2;
    assert!(pass_draws(&mut sim, 0) > 0, "a hard AI does not read it");
}

/// `Wonders::init_wonder` takes the first clear entry under the mark,
/// and `close_wonder` walks the mark down only past clear entries at
/// its top: the mark is one past the highest entry in use, not a count.
#[test]
fn the_wonder_mark_is_one_past_the_highest_entry_in_use() {
    let (mut sim, t) = sim();
    // One wonder a city (`blocked_site`'s wonder clause), so two.
    let _a = city(&mut sim, &t, 0, 40, 40);
    let _b = city(&mut sim, &t, 0, 80, 40);
    let wonder = |sim: &mut Sim, x: i32, y: i32| {
        let b = sim
            .place_building(0, t.wonder, tile_pos(x, y))
            .expect("a wonder places");
        finish(sim, b);
        b
    };
    let one = wonder(&mut sim, 48, 40);
    let two = wonder(&mut sim, 88, 40);
    sim.note_wonders();
    assert_eq!(sim.ai.ai[0].census.wonder_mark, 2);
    sim.buildings[one].alive = false;
    sim.note_wonders();
    assert_eq!(sim.ai.ai[0].census.wonder_mark, 2, "entry 0 clears under 1");
    let three = wonder(&mut sim, 48, 48);
    sim.note_wonders();
    assert_eq!(sim.ai.ai[0].census.wonder_slots, [Some(three), Some(two)]);
    sim.buildings[two].alive = false;
    sim.note_wonders();
    assert_eq!(
        sim.ai.ai[0].census.wonder_mark, 1,
        "the top clears and walks"
    );
    assert_eq!(sim.ai.ai[1].census.wonder_mark, 0, "a leader's own list");
}

/// The wonder `t`'s listing after one pass from a copy of `sim`.
fn wonder_listed(sim: &Sim, t: usize) -> MakeObject {
    let mut s = sim.clone();
    s.create_buildings(0);
    listed(&s, 0, t).expect("the wonder is listed")
}

/// `docs/AI.md` §124: the arm's product takes the type's own
/// `WONDER_VAL` (vslot `+0x118`, `get_wonder_value`). French East
/// Indies 14784: Tikal's 2 is half of the original's factor of four.
#[test]
fn a_wonder_is_worth_its_own_points() {
    let (mut sim, t) = sim();
    let _c = city(&mut sim, &t, 0, 40, 40);
    sim.ai.lobby.difficulty = 3;
    let one = wonder_listed(&sim, t.wonder);
    sim.tables.build_types[t.wonder].wonder_val = 2;
    let two = wonder_listed(&sim, t.wonder);
    assert!(one.val > 0);
    assert!(
        (two.val - 2 * one.val).abs() <= 2,
        "{} {}",
        one.val,
        two.val
    );
    assert_eq!(sim.tables.build_types[t.barracks].wonder_points(), 0);
}

/// §124, the other half of Tikal's four: `(team + 1)`, the points of
/// the wonders in the team's lists. who=1's Pyramids (1) stood.
#[test]
fn a_held_wonder_doubles_the_next() {
    let (mut sim, t) = sim();
    let _a = city(&mut sim, &t, 0, 40, 40);
    let _b = city(&mut sim, &t, 0, 80, 40);
    sim.ai.lobby.difficulty = 3;
    // A second wonder type's site in both: it shuts city b and prices
    // the wonder alike, so the team's points are the only difference.
    sim.tables.build_types[t.silo].wonder = true;
    sim.tables.build_types[t.silo].wonder_val = 1;
    let b = sim
        .place_building(0, t.silo, tile_pos(88, 40))
        .expect("the other wonder's site places");
    // Held is the wonder list's entry (`Wonders::init_wonder`): written
    // here directly, so the city's facts are the same on both sides.
    let mut held = sim.clone();
    held.ai.ai[0].census.wonder_slots = vec![Some(b)];
    held.ai.ai[0].census.wonder_mark = 1;
    assert_eq!(sim.team_wonder_points(0), 0, "a site is not held");
    assert_eq!(held.team_wonder_points(0), 1);
    assert_eq!(held.enemy_wonder_points(1), 1);
    assert_eq!(held.team_wonder_points(1), 0);
    let (none, one) = (
        wonder_listed(&sim, t.wonder),
        wonder_listed(&held, t.wonder),
    );
    assert!(
        (one.val - 2 * none.val).abs() <= 2,
        "{} {}",
        none.val,
        one.val
    );
    assert_eq!(one.escrow, 0);
}

/// §124: an enemy's wonders, standing or unbuilt, at half the wonder
/// victory's points or past four, value the wonder at 2,000,000 with
/// escrow and spend no draw.
#[test]
fn an_enemy_s_wonder_site_past_half_the_target_shuts_the_draws() {
    let (mut sim, t) = sim();
    let _a = city(&mut sim, &t, 0, 40, 40);
    let _b = city(&mut sim, &t, 1, 80, 40);
    sim.ai.lobby.difficulty = 3;
    sim.tables.build_types[t.wonder].wonder_val = 3;
    sim.place_building(1, t.wonder, tile_pos(88, 40))
        .expect("the enemy's site places");
    assert_eq!(sim.enemy_unbuilt_wonder_points(0), 3);
    assert!(
        pass_draws(&mut sim.clone(), 0) > 0,
        "3 < 8 / 2: the arm draws"
    );
    sim.ai.lobby.wonder_win_points = 6;
    assert_eq!(pass_draws(&mut sim.clone(), 0), 0, "6 / 2 ≤ 3");
    let l = wonder_listed(&sim, t.wonder);
    assert_eq!(l.escrow, 1);
}

/// §124, `Game::wonder_winning`: an ally whose team reaches the points
/// is winning, and a leader that is not the winner values every wonder
/// at 2,000,000 — the enemies' sum is 0 here, so only the winner shuts
/// the draws.
#[test]
fn an_ally_winning_by_wonders_shuts_the_draws() {
    let (mut sim, t) = sim();
    let _a = city(&mut sim, &t, 0, 40, 40);
    let _b = city(&mut sim, &t, 1, 80, 40);
    sim.ai.lobby.difficulty = 3;
    sim.set_diplo(0, 1, 2);
    sim.tables.build_types[t.wonder].wonder_val = 8;
    let b = sim
        .place_building(1, t.wonder, tile_pos(88, 40))
        .expect("the ally's wonder places");
    finish(&mut sim, b);
    assert_eq!(sim.wonder_winning(), -1, "not yet in the list");
    sim.note_wonders();
    assert_eq!(sim.enemy_wonder_points(0), 0);
    assert_eq!(sim.wonder_net(0), 8);
    assert_eq!(sim.wonder_winning(), 1, "the tie goes to the points");
    assert_eq!(pass_draws(&mut sim.clone(), 0), 0);
    sim.ai.lobby.wonder_win_points = 9;
    assert_eq!(sim.wonder_winning(), -1);
    assert!(pass_draws(&mut sim, 0) > 0);
}

#[test]
fn a_wonder_a_rival_is_a_quarter_through_is_abandoned() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    let _c2 = city(&mut sim, &t, 1, 80, 40);
    sim.ai.lobby.difficulty = 3;
    let b = sim
        .place_building(1, t.wonder, tile_pos(74, 40))
        .expect("the rival's wonder places");
    // A fifth of the way: still worth racing.
    sim.buildings[b].job_counter = sim.buildings[b].constr_time / 5;
    assert!(value(&mut sim, 0, c, t.wonder).is_some());
    // A third of the way: give up.
    sim.buildings[b].job_counter = sim.buildings[b].constr_time / 3;
    assert!(value(&mut sim, 0, c, t.wonder).is_none());
}

#[test]
fn my_own_site_blocks_the_wonder_however_far_along_it_is() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    sim.ai.lobby.difficulty = 3;
    let b = sim
        .place_building(0, t.wonder, tile_pos(48, 48))
        .expect("my wonder places");
    sim.buildings[b].job_counter = 0;
    assert!(value(&mut sim, 0, c, t.wonder).is_none());
}

#[test]
fn the_wonder_mod_wants_a_wonder_and_devalues_everything_else() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    sim.ai.lobby.difficulty = 3;
    sim.ai.ai[0].wonder_mod = 0x100;
    let before = sim.rng;
    let wd = value(&mut sim, 0, c, t.wonder).expect("a wonder is valued");
    assert_eq!(sim.rng, before, "the shortcut arm takes no draw");
    assert_eq!(wd.escrow, 1);
    // 100,000,000 × the city's level. The type carries `e`, so the
    // common tail's open-ground clause does not touch it, and the
    // divisor for a first wonder is one.
    assert_eq!(wd.val, 100_000_000);
    // And a library is worth a hundredth of what it was.
    let dulled = value(&mut sim, 0, c, t.library).expect("still valued").val;
    sim.ai.ai[0].wonder_mod = 0;
    let full = value(&mut sim, 0, c, t.library).expect("still valued").val;
    assert!(dulled * 90 < full, "{dulled} vs {full}");
}

// ------------------------------------------------------------------
// The producers
// ------------------------------------------------------------------

fn tech_at(sim: &mut Sim, at: usize) -> TypeId {
    let lib_t = sim.tables.build_types[at].tree.expect("in the tree");
    let mut tree = sim.tables.tech_tree.clone();
    let mut d = TypeDef::plain("Writing", 1);
    d.where_ = Some(lib_t);
    d.tribe_mask = u32::MAX;
    let writing = tree.add(d);
    sim.set_tech_tree(tree);
    writing
}

fn writing_tech(sim: &mut Sim, t: &Types) -> TypeId {
    tech_at(sim, t.library)
}

#[test]
fn produce_upgrade_queues_a_technology_at_the_best_member() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    let writing = writing_tech(&mut sim, &t);
    let b = sim
        .place_building(0, t.library, tile_pos(48, 40))
        .expect("a library places");
    finish(&mut sim, b);
    assert_eq!(sim.buildings[b].city, Some(c));
    assert!(sim.produce_upgrade(0, writing, Some(c), 0));
    assert_eq!(sim.buildings[b].queue.count_tech(writing), 1);
    // The same call again finds that member already holding it.
    assert!(!sim.produce_upgrade(0, writing, Some(c), 0));
}

#[test]
fn produce_upgrade_refuses_a_city_with_no_such_building() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    let writing = writing_tech(&mut sim, &t);
    assert!(!sim.produce_upgrade(0, writing, Some(c), 0));
    assert!(!sim.produce_upgrade(0, writing, None, 0));
}

#[test]
fn produce_upgrade_skips_a_site_and_takes_the_finished_building() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    // A drill queued at the Barracks, of which a city may hold several.
    let drill = tech_at(&mut sim, t.barracks);
    let writing = drill;
    let site = sim
        .place_building(0, t.barracks, tile_pos(48, 40))
        .expect("a barracks places");
    let done = sim
        .place_building(0, t.barracks, tile_pos(48, 48))
        .expect("a second barracks places");
    finish(&mut sim, done);
    assert!(sim.produce_upgrade(0, writing, Some(c), 0));
    assert_eq!(sim.buildings[site].queue.count_tech(writing), 0);
    assert_eq!(sim.buildings[done].queue.count_tech(writing), 1);
}

#[test]
fn produce_spell_never_casts() {
    let (mut sim, t) = sim();
    let c = city(&mut sim, &t, 0, 40, 40);
    let id = sim.tables.build_types[t.temple].tree.expect("in the tree");
    assert!(!sim.produce_spell(0, id, Some(c), 1));
}

/// **A building type gained converts the line below it** (item 1264,
/// `docs/TECH.md` step 8): `Leader::gain_tech`'s loop at
/// `6dde64`–`6ddf2b` sets every in-use building whose type's `upgrade`
/// is the gained type to it. East Indies' Tower `1/2014` is a Keep in
/// the original's `num_buildings` from the Keep's gain, and
/// `create_buildings` read the missing Keep as a first one to offer.
#[test]
fn gaining_the_keep_turns_the_leader_s_standing_tower_into_one() {
    let (mut sim, t) = sim();
    let _c = city(&mut sim, &t, 0, 40, 40);
    let tower_id = sim.tables.build_types[t.tower].tree.expect("in the tree");
    let mut keep = bt(Ident::Tower, "ecan", 2, 2);
    keep.attack = 12;
    keep.from = Some(t.tower);
    let keep = sim.add_build_type(keep);
    sim.tables.build_types[t.tower].to = Some(keep);
    let mut tree = sim.tables.tech_tree.clone();
    let mut d = TypeDef::building("Keep");
    d.tribe_mask = u32::MAX;
    d.from = Some(tower_id);
    let keep_id = tree.add(d);
    tree.types[tower_id].upgrade = Some(keep_id);
    sim.set_tech_tree(tree);
    sim.tables.build_types[keep].tree = Some(keep_id);

    let mine = sim
        .place_building(0, t.tower, tile_pos(46, 40))
        .expect("a tower places");
    finish(&mut sim, mine);
    let site = sim
        .place_building(0, t.tower, tile_pos(46, 46))
        .expect("a second tower places");
    let theirs = sim.init_build(1, t.tower, tile_pos(10, 10), false);
    finish(&mut sim, theirs);
    let library = sim
        .place_building(0, t.library, tile_pos(52, 40))
        .expect("a library places");
    assert_eq!(sim.num_buildings_of(0, t.tower), 1);
    assert_eq!(sim.num_buildings_of(0, keep), 0);

    sim.gain_tech(0, keep_id);

    assert_eq!(sim.buildings[mine].ty, Some(keep), "the finished tower");
    assert_eq!(
        sim.buildings[site].ty,
        Some(keep),
        "the site: in use is enough"
    );
    assert_eq!(
        sim.buildings[mine].orig_ty,
        Some(t.tower),
        "placed as a tower"
    );
    assert_eq!(sim.buildings[theirs].ty, Some(t.tower), "another leader's");
    assert_eq!(sim.buildings[library].ty, Some(t.library));
    assert_eq!(sim.num_buildings_of(0, t.tower), 0);
    assert_eq!(sim.num_buildings_of(0, keep), 1);
    assert_eq!(sim.buildings_of_line(0, t.tower), 1);
}
