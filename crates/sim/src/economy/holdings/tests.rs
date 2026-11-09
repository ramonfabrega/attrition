use super::*;
use crate::economy::{RATE_SCALE, RESOURCES};
use crate::orders::{QueuePos, Worker};
use crate::tuning::Tuning;
use crate::world::{Cell, Player as P, Pos, Terrain, World};
use crate::{Unit, UnitType, build::BuildType, build::flags, cost, movement};

const TILE: i32 = crate::world::UNITS_PER_TILE;

fn tile_pos(tx: i32, ty: i32) -> Pos {
    Pos::new(tx * TILE + TILE / 2, ty * TILE + TILE / 2)
}

fn world_sim() -> Sim {
    let mut w = World::new(16, 16);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
    let mut sim = Sim::new(Tuning::RON, w, 2);
    for l in &mut sim.economy.ledgers {
        l.bucket = [10_000; RESOURCES];
    }
    // Both players are people: no difficulty handicap in these tests.
    for n in &mut sim.cities_state.nation {
        n.human = true;
    }
    sim
}

fn bt(ident: Ident, flag: &str, xs: i32, ys: i32) -> BuildType {
    BuildType {
        ident,
        x_size: xs,
        y_size: ys,
        flags: flags::parse(flag),
        job_time: 150,
        hits: 400,
        price: cost::Price {
            kind: cost::Kind::Building,
            ..cost::Price::free()
        },
        ..BuildType::default()
    }
}

struct Types {
    village: usize,
    farm: usize,
    camp: usize,
    university: usize,
    library: usize,
    market: usize,
    granary: usize,
    oil_well: usize,
    refinery: usize,
    citizen: usize,
    scholar: usize,
}

fn install(sim: &mut Sim) -> Types {
    let village = sim.add_build_type(bt(Ident::Village, "ean", 7, 7));
    let mut farm = bt(Ident::Farm, "gda", 4, 4);
    farm.flags |= flags::FLAT;
    let farm = sim.add_build_type(farm);
    // `e` no city, `f` territory-exempt: a camp can stand in the wild.
    let camp = sim.add_build_type(bt(Ident::Woodcutter, "gaef", 4, 4));
    let university = sim.add_build_type(bt(Ident::University, "gja", 4, 4));
    let library = sim.add_build_type(bt(Ident::Library, "ja", 5, 5));
    let market = sim.add_build_type(bt(Ident::Market, "ja", 4, 4));
    let granary = sim.add_build_type(bt(Ident::Granary, "ja", 4, 4));
    let mut oil = bt(Ident::OilWell, "ga", 4, 4);
    oil.flags |= flags::FLAT;
    let oil_well = sim.add_build_type(oil);
    let refinery = sim.add_build_type(bt(Ident::Refinery, "ja", 4, 4));

    let citizen = sim.add_unit_type(worker_type(village, Worker::Citizen));
    let scholar = sim.add_unit_type(worker_type(village, Worker::Scholar));
    Types {
        village,
        farm,
        camp,
        university,
        library,
        market,
        granary,
        oil_well,
        refinery,
        citizen,
        scholar,
    }
}

fn worker_type(village: usize, worker: Worker) -> UnitType {
    UnitType {
        price: cost::Price {
            pop: 1,
            ..cost::Price::free()
        },
        hits: 40,
        worker,
        garrison: crate::garrison::UnitTraits {
            trained_at: Some(village),
            ..crate::garrison::UnitTraits::default()
        },
        ..UnitType::default()
    }
}

fn spawn(sim: &mut Sim, owner: P, ty: usize, pos: Pos) -> usize {
    let index = i16::try_from(sim.units.len()).unwrap();
    let hits = sim.tables.unit_types[ty].hits;
    let mut u = Unit::new(owner, index, pos, hits);
    u.ty = Some(ty);
    u.kind = sim.tables.unit_types[ty].kind;
    u.movement.speed = 25;
    u.movement.turning = movement::Turning {
        type_turn_speed: movement::degrees_to_angle(45).0,
        packed: false,
        instant_from_stop: true,
        wide_limit: false,
    };
    sim.add_unit(u)
}

fn finish(sim: &mut Sim, b: usize) {
    let mut guard = 0;
    while !sim.buildings[b].active && sim.buildings[b].alive {
        sim.do_construct(b, 1_000_000);
        guard += 1;
        assert!(guard < 10);
    }
    sim.buildings[b].helpers = 0;
}

fn build(sim: &mut Sim, who: P, ty: usize, tx: i32, ty_: i32) -> usize {
    let b = sim
        .place_building(who, ty, tile_pos(tx, ty_))
        .unwrap_or_else(|e| panic!("should place: {e:?}"));
    finish(sim, b);
    b
}

/// Walks a worker onto a gather building and returns when the arrival has
/// registered — `been_there`, which is what `num_gatherers(1, 1)` reads.
fn gather_until_arrived(sim: &mut Sim, u: usize, b: usize) {
    sim.add_gather_order(u, b, QueuePos::New, false);
    let mut guard = 0;
    while sim.num_gatherers(b, true, true) == 0 {
        sim.tick();
        guard += 1;
        assert!(guard < 200, "the worker never arrived");
    }
}

/// **The CEO's city is the one its owner's counted, on-map `0x165`
/// hero stands in** (`calc_city_resources@006d5530:166`–`195`, item
/// 3516): no city while the leader's count is 0, none for a hero off
/// the map or not a general, none for another player's; otherwise the
/// city at its position, which the city's income then reads.
#[test]
fn the_ceo_s_city_is_where_a_counted_on_map_hero_stands() {
    use crate::ai_load::uflags2;
    let mut sim = world_sim();
    let t = install(&mut sim);
    let b = build(&mut sim, 0, t.village, 32, 32);
    let c = sim.buildings[b].city.expect("a finished city");
    let mut ceo = UnitType {
        type_index: 0x165,
        hits: 50,
        ..UnitType::default()
    };
    ceo.cols.unit_flags2 = uflags2::FLAG2_GENERAL;
    let ceo = sim.add_unit_type(ceo);
    let h = spawn(&mut sim, 0, ceo, tile_pos(33, 33));
    sim.units[h].type_index = 0x165;
    sim.units[h].on_map = true;
    assert_eq!(sim.ceo_city(0), None, "a hero the leader does not count");
    sim.roster.muster[0].by_type[ceo] = 1;
    assert_eq!(sim.ceo_city(0), Some(c));
    assert!(sim.city_holdings(c, Levels::BASE).ceo);
    sim.units[h].on_map = false;
    assert_eq!(sim.ceo_city(0), None, "off the map");
    sim.units[h].on_map = true;
    sim.tables.unit_types[ceo].cols.unit_flags2 = 0;
    assert_eq!(sim.ceo_city(0), None, "not a hero");
    sim.tables.unit_types[ceo].cols.unit_flags2 = uflags2::FLAG2_GENERAL;
    sim.roster.muster[1].by_type[ceo] = 1;
    assert_eq!(sim.ceo_city(1), None, "another player's hero");
}

// ------------------------------------------------------------------
// The one that matters: a farm with a citizen on it pays.
// ------------------------------------------------------------------

#[test]
fn a_farm_with_one_gathering_citizen_pays_the_peasant_rate() {
    let mut sim = world_sim();
    let t = install(&mut sim);
    let city = build(&mut sim, 0, t.village, 32, 32);
    let c = sim.buildings[city]
        .city
        .expect("a finished city has a record");
    let farm = build(&mut sim, 0, t.farm, 40, 32);
    assert_eq!(sim.buildings[farm].city, Some(c), "the farm joins the city");

    // Before anybody works it, the city is worth CITY_GATHER and no more.
    sim.assemble_holdings(0);
    assert_eq!(sim.economy.holdings[0].cities.len(), 1);
    assert_eq!(
        sim.economy.holdings[0].cities[0].sites,
        vec![Site {
            resource: Resource::Food,
            gatherers: 0,
            level: 1,
            nation_pct: 0,
            farm_wealth: 0,
            miner_wealth: 0,
        }],
        "the farm is a site the moment it is finished, with nobody on it"
    );
    let bare = economy::assemble(&sim.tables.tuning, &sim.economy.holdings[0]);
    assert_eq!(bare[0], sim.tables.tuning.city_gather[0] * RATE_SCALE);

    // Now put a citizen on it and walk it there.
    let u = spawn(&mut sim, 0, t.citizen, tile_pos(44, 32));
    gather_until_arrived(&mut sim, u, farm);

    sim.assemble_holdings(0);
    assert_eq!(sim.economy.holdings[0].cities[0].sites[0].gatherers, 1);

    // Through `economy::process`: one gatherer is PEASANT_RATE, the city
    // is CITY_GATHER, and the pair arrive exactly once per GATHER_RATE
    // frames.
    let mut ledger = economy::Ledger {
        dirty: true,
        ..economy::Ledger::default()
    };
    for frame in 0..i64::from(sim.tables.tuning.gather_rate) {
        economy::process(
            &sim.tables.tuning,
            &mut ledger,
            &sim.economy.holdings[0],
            0,
            frame,
        );
    }
    let expected = (sim.tables.tuning.peasant_rate >> 8) + sim.tables.tuning.city_gather[0];
    assert_eq!(expected, 20);
    assert_eq!(ledger.rate[0], expected * RATE_SCALE);
    assert_eq!(ledger.bucket[Resource::Food.index()], expected);
    // And nothing else moved.
    assert_eq!(ledger.bucket[Resource::Metal.index()], 0);
}

/// `BuildTypeData::calc_gather@00639e40:978`–`985`: a Japanese farm's
/// food per gatherer is `(JAPANESE_FISHING_BOATS + 100) × rate / 100`,
/// and it is the nation, not the player (item 3331, `docs/AI.md` §268).
#[test]
fn a_japanese_farmer_gathers_a_quarter_more_food() {
    let mut sim = world_sim();
    let t = install(&mut sim);
    build(&mut sim, 0, t.village, 32, 32);
    let farm = build(&mut sim, 0, t.farm, 40, 32);
    let u = spawn(&mut sim, 0, t.citizen, tile_pos(44, 32));
    gather_until_arrived(&mut sim, u, farm);
    let rate = |sim: &mut Sim| {
        sim.assemble_holdings(0);
        economy::city_rates(&sim.tables.tuning, &sim.economy.holdings[0].cities[0])[0]
    };
    let city = sim.tables.tuning.city_gather[0] * RATE_SCALE;
    let plain = rate(&mut sim);
    assert_eq!(plain - city, 160, "PEASANT_RATE, in sixteenths");
    sim.set_tribe(0, 15);
    assert_eq!(sim.economy.holdings[0].cities.len(), 1);
    assert_eq!(rate(&mut sim) - city, 200, "160 × 125 / 100");
    assert_eq!(sim.economy.holdings[0].cities[0].sites[0].nation_pct, 25);
    sim.set_tribe(0, 16);
    assert_eq!(rate(&mut sim), plain, "the Koreans farm at cost");
}

/// `BuildTypeData::calc_gather@00639e40:997`–`1005`: an Egyptian Farm
/// with any gatherer pays `EGYPTIAN_FARM_WEALTH × 16` wealth, flat — not
/// per gatherer — and an unworked one, or another nation's, nothing
/// (item 3332).
#[test]
fn an_inca_mine_pays_ten_wealth_a_miner() {
    let mut sim = world_sim();
    let t = install(&mut sim);
    let mine = sim.add_build_type(bt(Ident::Mine, "gaef", 4, 4));
    // A mountain range the cell east of the mine.
    for cy in 7..10 {
        for ty in 0..crate::world::TILES_PER_CELL {
            for tx in 0..crate::world::TILES_PER_CELL {
                let p = Pos::new(44 + tx, cy * crate::world::TILES_PER_CELL + ty);
                sim.world.set_tile_bits(
                    p,
                    crate::world::tile::OBJECT_MOUNTAIN | crate::gather::GATHERABLE,
                );
            }
        }
    }
    build(&mut sim, 0, t.village, 32, 32);
    let m = build(&mut sim, 0, mine, 42, 32);
    let rates = |sim: &mut Sim| {
        sim.assemble_holdings(0);
        economy::city_rates(&sim.tables.tuning, &sim.economy.holdings[0].cities[0])
    };
    let w = Resource::Wealth.index();
    let city = rates(&mut sim)[w];
    sim.set_tribe(0, 2);
    assert_eq!(rates(&mut sim)[w], city, "an unworked Mine pays nothing");
    for (i, x) in [40, 42].into_iter().enumerate() {
        let u = spawn(&mut sim, 0, t.citizen, tile_pos(x, 38));
        let n = i32::try_from(i + 1).unwrap();
        sim.add_gather_order(u, m, QueuePos::New, false);
        for _ in 0..200 {
            if sim.num_gatherers(m, true, true) == n {
                break;
            }
            sim.tick();
        }
        assert_eq!(sim.num_gatherers(m, true, true), n, "miner {n} arrived");
        assert_eq!(sim.tables.tuning.inca_wealth_per_miner, 10);
        assert_eq!(
            rates(&mut sim)[w] - city,
            10 * RATE_SCALE * n,
            "ten wealth a miner, per head"
        );
    }
    let metal = rates(&mut sim)[Resource::Metal.index()];
    sim.set_tribe(0, 16);
    assert_eq!(rates(&mut sim)[w], city, "the Koreans' Mine pays no wealth");
    assert_eq!(
        rates(&mut sim)[Resource::Metal.index()],
        metal,
        "the metal is the same"
    );
}

#[test]
fn an_egyptian_worked_farm_pays_two_wealth_flat() {
    let mut sim = world_sim();
    let t = install(&mut sim);
    build(&mut sim, 0, t.village, 32, 32);
    let farm = build(&mut sim, 0, t.farm, 40, 32);
    let wealth = |sim: &mut Sim| {
        sim.assemble_holdings(0);
        economy::city_rates(&sim.tables.tuning, &sim.economy.holdings[0].cities[0])
            [Resource::Wealth.index()]
    };
    let city = sim.tables.tuning.city_gather[Resource::Wealth.index()] * RATE_SCALE;
    sim.set_tribe(0, 7);
    assert_eq!(wealth(&mut sim), city, "an unworked Farm pays nothing");
    let u = spawn(&mut sim, 0, t.citizen, tile_pos(44, 32));
    gather_until_arrived(&mut sim, u, farm);
    assert_eq!(sim.tables.tuning.egyptian_farm_wealth, 2);
    assert_eq!(wealth(&mut sim) - city, 2 * RATE_SCALE, "two wealth a Farm");
    let v = spawn(&mut sim, 0, t.citizen, tile_pos(44, 34));
    gather_until_arrived(&mut sim, v, farm);
    assert_eq!(
        wealth(&mut sim) - city,
        2 * RATE_SCALE,
        "flat, not per head"
    );
    sim.set_tribe(0, 16);
    assert_eq!(wealth(&mut sim), city, "the Koreans' Farm pays no wealth");
}

#[test]
fn a_walking_citizen_earns_nothing_until_it_arrives() {
    // `BuildData::calc_gather` passes `num_gatherers(1, 1)` — arrived —
    // so the whole walk is unpaid. This is the difference between the
    // count the chain carries and the count the economy reads.
    let mut sim = world_sim();
    let t = install(&mut sim);
    build(&mut sim, 0, t.village, 32, 32);
    let farm = build(&mut sim, 0, t.farm, 40, 32);
    let u = spawn(&mut sim, 0, t.citizen, tile_pos(44, 32));
    sim.add_gather_order(u, farm, QueuePos::New, false);

    assert_eq!(sim.num_gatherers(farm, false, false), 1, "on the chain");
    sim.assemble_holdings(0);
    assert_eq!(
        sim.economy.holdings[0].cities[0].sites[0].gatherers, 0,
        "not arrived: worth nothing"
    );
    sim.tick();
    sim.assemble_holdings(0);
    assert_eq!(sim.economy.holdings[0].cities[0].sites[0].gatherers, 0);

    gather_until_arrived(&mut sim, u, farm);
    sim.assemble_holdings(0);
    assert_eq!(sim.economy.holdings[0].cities[0].sites[0].gatherers, 1);
}

#[test]
fn the_arrival_marks_the_economy_dirty_so_income_shows_within_eight_frames() {
    // The `0x2000000` half of this module's notes, end to end: the frame
    // the citizen arrives is the frame the flag goes up, and the rate is
    // reassembled on the next multiple of eight rather than at 512.
    let mut sim = world_sim();
    let t = install(&mut sim);
    build(&mut sim, 0, t.village, 32, 32);
    let farm = build(&mut sim, 0, t.farm, 40, 32);
    let u = spawn(&mut sim, 0, t.citizen, tile_pos(44, 32));
    sim.economy.ledgers[0].dirty = false;
    sim.economy.ledgers[0].gather_stamp = sim.frame;
    gather_until_arrived(&mut sim, u, farm);
    assert!(sim.economy.ledgers[0].dirty, "the arrival raised the flag");

    let arrived = sim.frame;
    let next = (arrived..arrived + 64)
        .find(|&f| sim.holdings_due(0, f))
        .expect("a dirty economy reassembles");
    assert!(
        next - arrived <= 8,
        "waited {} frames, not the lazy 512",
        next - arrived
    );
}

// ------------------------------------------------------------------
// The city's own five terms
// ------------------------------------------------------------------

#[test]
fn a_market_a_library_and_a_granary_are_read_off_the_member_chain() {
    let mut sim = world_sim();
    let t = install(&mut sim);
    let city = build(&mut sim, 0, t.village, 32, 32);
    let c = sim.buildings[city].city.unwrap();

    sim.assemble_holdings(0);
    let bare = sim.economy.holdings[0].cities[0].clone();
    assert!(!bare.market && !bare.library && !bare.university);
    assert_eq!(bare.granary, 0);
    assert_eq!(bare.buildings, 1, "the city centre is in its own chain");

    let market = build(&mut sim, 0, t.market, 38, 32);
    let library = build(&mut sim, 0, t.library, 32, 38);
    let granary = build(&mut sim, 0, t.granary, 26, 32);
    for b in [market, library, granary] {
        assert_eq!(sim.buildings[b].city, Some(c));
    }

    sim.assemble_holdings(0);
    let full = &sim.economy.holdings[0].cities[0];
    assert!(full.market && full.library);
    assert!(!full.university, "a library is not a university");
    assert_eq!(full.buildings, 4);
    // GRANARY_BONUS level 1, the level a player with no bonus type holds.
    assert_eq!(full.granary, sim.tables.tuning.granary_bonus[0]);
    assert_eq!(full.lumber_mill, 0, "no lumber mill, no percentage");

    // A market is ten wealth per thirty seconds. A library on its own is
    // worth nothing — `LIBRARY_LITERACY` ships as zero, and its value is
    // that scholars can live in it — and so is a granary with nobody
    // farming under it.
    let rate = economy::assemble(&sim.tables.tuning, &sim.economy.holdings[0]);
    assert_eq!(rate[Resource::Wealth.index()], 10 * RATE_SCALE);
    assert_eq!(rate[Resource::Knowledge.index()], 0);
}

#[test]
fn an_unfinished_building_is_not_in_the_economy_at_all() {
    // Both `count_buildings` and `num_buildings` insist on `is_active`,
    // and `BuildTypeData::calc_gather` answers zero for an inactive site.
    let mut sim = world_sim();
    let t = install(&mut sim);
    build(&mut sim, 0, t.village, 32, 32);
    let market = sim.place_building(0, t.market, tile_pos(38, 32)).unwrap();
    assert!(!sim.buildings[market].active);
    let farm = sim.place_building(0, t.farm, tile_pos(32, 39)).unwrap();

    sim.assemble_holdings(0);
    let c = &sim.economy.holdings[0].cities[0];
    assert!(!c.market, "a site under construction pays no taxes");
    assert_eq!(c.buildings, 1);
    assert!(c.sites.is_empty(), "and an unfinished farm is not a site");

    finish(&mut sim, farm);
    sim.assemble_holdings(0);
    assert_eq!(sim.economy.holdings[0].cities[0].sites.len(), 1);
}

/// **`num_gatherers(_, 1)` drops a decoy from the garrison term too**
/// (twenty-fourth pass, A7 row 28): the garrison is
/// `count_inside(COUNT_TYPE + 2 * skip, …)`, and `COUNT_NON_DECOY_TYPE`
/// skips a unit whose `unit_masks & 1` is set. The admission count
/// (`skip = 0`) keeps it.
#[test]
fn a_decoy_scholar_is_not_the_economy_s_gatherer() {
    let mut sim = world_sim();
    let t = install(&mut sim);
    build(&mut sim, 0, t.village, 32, 32);
    let uni = build(&mut sim, 0, t.university, 40, 32);
    let real = spawn(&mut sim, 0, t.scholar, tile_pos(44, 32));
    let decoy = spawn(&mut sim, 0, t.scholar, tile_pos(44, 33));
    sim.units[decoy].decoy = true;
    for s in [real, decoy] {
        sim.buildings[uni].garrison.push(s);
        sim.units[s].on_map = false;
        sim.units[s].inside = Some(uni);
    }
    assert_eq!(
        sim.num_gatherers(uni, true, false),
        2,
        "admission counts both"
    );
    assert_eq!(
        sim.num_gatherers(uni, true, true),
        1,
        "the economy skips the decoy"
    );
}

#[test]
fn a_scholar_in_a_university_gathers_knowledge_at_the_leader_s_level() {
    // `num_gatherers` counts a *garrisoned* scholar, and the rate is
    // `SCHOLAR_RATE[university_level - 1]` off the leader, not the
    // building.
    let mut sim = world_sim();
    let t = install(&mut sim);
    build(&mut sim, 0, t.village, 32, 32);
    let uni = build(&mut sim, 0, t.university, 40, 32);
    let s = spawn(&mut sim, 0, t.scholar, tile_pos(44, 32));
    sim.buildings[uni].garrison.push(s);
    sim.units[s].on_map = false;
    sim.units[s].inside = Some(uni);
    assert_eq!(sim.num_gatherers(uni, true, true), 1);

    sim.assemble_holdings_with(0, Levels::BASE);
    let site = sim.economy.holdings[0].cities[0]
        .sites
        .iter()
        .find(|s| s.resource == Resource::Knowledge)
        .expect("the university is a knowledge site");
    assert_eq!(site.gatherers, 1);
    assert_eq!(site.level, 1);

    let base = economy::assemble(&sim.tables.tuning, &sim.economy.holdings[0]);
    // A university is ten knowledge of literacy plus one scholar at
    // SCHOLAR_RATE level 1.
    assert_eq!(
        base[Resource::Knowledge.index()],
        10 * RATE_SCALE + economy::per_gatherer(&sim.tables.tuning, Resource::Knowledge, 1)
    );

    // The last university level is worth five times the first.
    sim.assemble_holdings_with(
        0,
        Levels {
            university: 6,
            ..Levels::BASE
        },
    );
    let top = economy::assemble(&sim.tables.tuning, &sim.economy.holdings[0]);
    assert_eq!(
        top[Resource::Knowledge.index()] - 10 * RATE_SCALE,
        25 * RATE_SCALE
    );
}

// ------------------------------------------------------------------
// The player-level terms
// ------------------------------------------------------------------

#[test]
fn refineries_are_counted_across_the_whole_nation() {
    let mut sim = world_sim();
    let t = install(&mut sim);
    build(&mut sim, 0, t.village, 32, 32);
    sim.assemble_holdings(0);
    assert_eq!(sim.economy.holdings[0].refineries, 0);

    build(&mut sim, 0, t.refinery, 38, 32);
    sim.assemble_holdings(0);
    assert_eq!(sim.economy.holdings[0].refineries, 1);

    // Another player's refinery is not ours.
    build(&mut sim, 1, t.village, 8, 8);
    build(&mut sim, 1, t.refinery, 14, 8);
    sim.assemble_holdings(0);
    sim.assemble_holdings(1);
    assert_eq!(sim.economy.holdings[0].refineries, 1);
    assert_eq!(sim.economy.holdings[1].refineries, 1);
}

#[test]
fn an_oil_well_pays_outside_the_city_loop_even_when_it_stands_in_a_city() {
    // The city loop skips `is(OILWELL)` and `is(OILPLATFORM)` explicitly,
    // because the `oil_wells` list already covers them. Counting them in
    // both places would double a nation's oil.
    let mut sim = world_sim();
    let t = install(&mut sim);
    let city = build(&mut sim, 0, t.village, 32, 32);
    let c = sim.buildings[city].city.unwrap();
    // An Oil Well stands only on the cell's oil (`blocked_tcoord`'s
    // `NoOil`, `docs/AI.md` §78).
    for cy in 7..=8 {
        for cx in 9..=10 {
            let at = crate::Cell::new(cx, cy);
            let mut d = sim.world.cell_data(at);
            d.flags |= crate::world::cell::OIL;
            sim.world.set_cell_data(at, d);
        }
    }
    let well = build(&mut sim, 0, t.oil_well, 40, 32);
    assert_eq!(sim.buildings[well].city, Some(c), "it is a city member");

    sim.assemble_holdings(0);
    assert!(
        sim.economy.holdings[0].cities[0]
            .sites
            .iter()
            .all(|s| s.resource != Resource::Oil),
        "not a city site"
    );
    let outside = sim.gather_sites_outside_cities(0);
    assert_eq!(outside.len(), 1);
    assert_eq!(outside[0].resource, Resource::Oil);
}

/// **The oil well's stand puts the worker on `CHAR_FARM`** — and a guy
/// already on it returns (`do_gather@005ef2a0:377`), so `Guy::move`'s
/// arrival test (`cur_anim == CHAR_WALK`) never sees a walk and the
/// arrival stand `Guy::move+0x19f` is never rolled (`docs/AI.md` §150,
/// item 1549: Great Sahara in the coverage lobby, frame 720, where this
/// crate spent one draw the original did not).
#[test]
fn an_oil_well_s_gatherer_stands_on_the_farm_animation_and_rolls_no_arrival() {
    let mut sim = world_sim();
    let t = install(&mut sim);
    build(&mut sim, 0, t.village, 32, 32);
    for cy in 7..=8 {
        for cx in 9..=10 {
            let at = crate::Cell::new(cx, cy);
            let mut d = sim.world.cell_data(at);
            d.flags |= crate::world::cell::OIL;
            sim.world.set_cell_data(at, d);
        }
    }
    let well = build(&mut sim, 0, t.oil_well, 40, 32);
    let u = spawn(&mut sim, 0, t.citizen, tile_pos(44, 32));
    sim.units[u].guys.push(crate::anim::Guy::fresh(0));
    sim.trace.trace_phases = true;
    gather_until_arrived(&mut sim, u, well);
    sim.trace.phase_marks.clear();
    let mut arrivals = 0;
    for _ in 0..8 {
        sim.tick();
        arrivals += sim
            .trace
            .phase_marks
            .iter()
            .filter(|(l, _)| l == crate::anim::SITE_ARRIVE)
            .count();
        sim.trace.phase_marks.clear();
    }
    assert_eq!(
        sim.units[u].guys[0].anim,
        crate::anim::ANIM_FARM,
        "the worker works the well on the farm animation"
    );
    assert_eq!(arrivals, 0, "and it rolls no arrival stand");
    // **The guard**: a guy already on `CHAR_FARM` returns before the
    // turn and the relocation, so a worker nudged off its stand is not
    // put back (`do_gather:377`).
    let stand = sim.units[u].pos;
    let nudged = Pos::new(stand.x + 8, stand.y);
    let from = sim.units[u].pos;
    sim.set_new_location(u, nudged, true);
    sim.moved_to(u, from, false);
    sim.tick();
    assert_eq!(
        sim.units[u].pos, nudged,
        "the guard leaves the worker where it stands"
    );
}

#[test]
fn a_camp_with_no_city_is_outside_the_city_loop() {
    let mut sim = world_sim();
    let t = install(&mut sim);
    // No city anywhere near: `find_city` leaves the link empty.
    build(&mut sim, 0, t.village, 4, 4);
    sim.plant_camp_forest(tile_pos(56, 56));
    let camp = build(&mut sim, 0, t.camp, 56, 56);
    assert_eq!(sim.buildings[camp].city, None);

    sim.assemble_holdings(0);
    assert!(sim.economy.holdings[0].cities[0].sites.is_empty());
    let outside = sim.gather_sites_outside_cities(0);
    assert_eq!(outside.len(), 1);
    assert_eq!(outside[0].resource, Resource::Timber);
}

/// A tree carrying the five `TECHBONUSES` ladders, nothing else: the
/// rows in level order, so `roles.*_preq[i]` is the `i+1`-th rung.
fn with_the_ladders(sim: &mut Sim) {
    use crate::tech::{Line, TechTree, TypeDef, TypeId};
    let mut tree = TechTree::new();
    let rung = |tree: &mut TechTree, n: &str| -> Option<TypeId> {
        Some(tree.add(TypeDef::epoch(n, Line::Commerce, 0)))
    };
    let g = [
        rung(&mut tree, "GRANARY2"),
        rung(&mut tree, "GRANARY3"),
        rung(&mut tree, "GRANARY4"),
        rung(&mut tree, "GRANARY5"),
    ];
    let l = [
        rung(&mut tree, "LUMBERMILL2"),
        rung(&mut tree, "LUMBERMILL3"),
        rung(&mut tree, "LUMBERMILL4"),
    ];
    let sm = [
        rung(&mut tree, "SMELTER2"),
        rung(&mut tree, "SMELTER3"),
        rung(&mut tree, "SMELTER4"),
    ];
    let u = [
        rung(&mut tree, "UNIVERSITY2"),
        rung(&mut tree, "UNIVERSITY3"),
        rung(&mut tree, "UNIVERSITY4"),
        rung(&mut tree, "UNIVERSITY5"),
        rung(&mut tree, "UNIVERSITY6"),
    ];
    let tx = [
        rung(&mut tree, "TAX_1"),
        rung(&mut tree, "TAX_2"),
        rung(&mut tree, "TAX_3"),
        rung(&mut tree, "TAX_4"),
    ];
    tree.roles.granary_preq = g;
    tree.roles.lumbermill_preq = l;
    tree.roles.smelter_preq = sm;
    tree.roles.university_preq = u;
    tree.roles.taxation_preq = tx;
    sim.set_tech_tree(tree);
}

/// **The five ladders answer their floor, and only their floor, with
/// nothing held** — one everywhere, and **zero** for taxation. That
/// asymmetry is `get_granary@006db340`'s `(held) + 1` against
/// `get_taxation@006d6e20`'s `(uint)(held != 0)`, and it is the whole
/// reason the territory tax is the sharp one (item 232, ECONOMY R8).
#[test]
fn the_five_ladders_floor_at_one_except_taxation_which_floors_at_zero() {
    let mut sim = world_sim();
    with_the_ladders(&mut sim);
    assert_eq!(Levels::for_player(&sim, 0), Levels::BASE);
    assert_eq!(Levels::BASE.taxation, 0);

    // And a tree that names none of the rows — every fixture that builds
    // its own — answers the same, so item 232 is a widening.
    let bare = world_sim();
    assert_eq!(Levels::for_player(&bare, 0), Levels::BASE);
}

/// **Each ladder is the highest rung the player holds**, tested most
/// advanced first: holding rung 2 and rung 4 answers 4.
#[test]
fn each_ladder_answers_the_highest_rung_held() {
    let mut sim = world_sim();
    with_the_ladders(&mut sim);
    let roles = sim.tables.tech_tree.roles.clone();
    for (rows, floor, want) in [
        (roles.granary_preq.to_vec(), 1, 5),
        (roles.lumbermill_preq.to_vec(), 1, 4),
        (roles.smelter_preq.to_vec(), 1, 4),
        (roles.university_preq.to_vec(), 1, 6),
        (roles.taxation_preq.to_vec(), 0, 4),
    ] {
        let mut s = world_sim();
        with_the_ladders(&mut s);
        // The second rung alone: floor + 2.
        s.tech[0].tech[rows[1].unwrap()] = true;
        assert_eq!(s.bonus_level(0, &rows) + floor, floor + 2);
        // The top rung as well — the ladder does not add up, it picks.
        s.tech[0].tech[rows[rows.len() - 1].unwrap()] = true;
        assert_eq!(s.bonus_level(0, &rows) + floor, want);
        // And it is per player.
        assert_eq!(s.bonus_level(1, &rows), 0);
    }
}

/// **The territory tax is a whole income line that could never fire**
/// (audit R8): `Holdings::taxation` was written by nothing, so it kept
/// its `Default` zero and `TERRITORY_TAXES[0]` — 0 per cent — for the
/// life of every game. `assemble_holdings` writes it now.
///
/// **No capture can falsify this.** The original's own territory tax is
/// zero through every run on disk too, because neither leader reaches a
/// taxation level inside any captured window (ECONOMY, "What run60
/// leaves"), so the two agree before the fix and after it. This is
/// correctness with no oracle behind it, and the check that would give
/// it one is a capture past a `TAX_1` research with `LEADERDATA`'s
/// `resources[2]`/`income[2]` on the frame after.
#[test]
fn taxation_reaches_the_territory_tax() {
    let mut sim = world_sim();
    let t = install(&mut sim);
    with_the_ladders(&mut sim);
    build(&mut sim, 0, t.village, 32, 32);
    sim.update_territory_holdings();
    assert!(sim.economy.holdings[0].territory > 0 && sim.economy.holdings[0].land_size > 0);

    sim.assemble_holdings(0);
    assert_eq!(sim.economy.holdings[0].taxation, 0, "no TAX_n held");
    assert_eq!(
        economy::territory_tax(&sim.tables.tuning, &sim.economy.holdings[0]),
        0
    );

    let tax2 = sim.tables.tech_tree.roles.taxation_preq[1].unwrap();
    sim.tech[0].tech[tax2] = true;
    sim.assemble_holdings(0);
    assert_eq!(sim.economy.holdings[0].taxation, 2);
    let h = &sim.economy.holdings[0];
    assert_eq!(
        economy::territory_tax(&sim.tables.tuning, h),
        h.territory * sim.tables.tuning.territory_taxes[2] * RATE_SCALE / h.land_size,
    );
    assert!(
        economy::territory_tax(&sim.tables.tuning, h) > 0,
        "the line the audit called unfirable",
    );
}

#[test]
fn the_commerce_level_is_the_commerce_epoch() {
    // `calc_resource_caps` reads `LeaderDataEncrypt + 0xf0`, which is
    // `epoch[2]`, and `Line::Commerce` is 2.
    let mut sim = world_sim();
    let t = install(&mut sim);
    build(&mut sim, 0, t.village, 32, 32);
    sim.tech[0].epoch[Line::Commerce.index()] = 3;
    sim.tech[0].epoch[Line::Science.index()] = 7;
    sim.assemble_holdings(0);
    assert_eq!(sim.economy.holdings[0].commerce, 3);
    assert_eq!(
        economy::commerce_cap(&sim.tables.tuning, &sim.economy.holdings[0], Resource::Food),
        sim.tables.tuning.commerce_cap[3] * RATE_SCALE
    );
}

#[test]
fn only_an_ai_carries_the_difficulty_handicap() {
    let mut sim = world_sim();
    let t = install(&mut sim);
    build(&mut sim, 0, t.village, 32, 32);
    build(&mut sim, 1, t.village, 8, 8);
    sim.ai_state.lobby.difficulty = 5;
    sim.cities_state.nation[1].human = false;

    sim.assemble_holdings(0);
    sim.assemble_holdings(1);
    assert_eq!(
        sim.economy.holdings[0].handicap, 0,
        "a human earns their rate"
    );
    assert_eq!(
        sim.economy.holdings[1].handicap, 50,
        "the AI earns half again"
    );
}

#[test]
fn the_fields_the_rest_of_the_simulation_owns_are_left_alone() {
    let mut sim = world_sim();
    let t = install(&mut sim);
    build(&mut sim, 0, t.village, 32, 32);
    sim.update_territory_holdings();
    let territory = sim.economy.holdings[0].territory;
    let land_size = sim.economy.holdings[0].land_size;
    assert!(land_size > 0);
    sim.economy.holdings[0].available[Resource::Oil.index()] = false;
    sim.economy.holdings[0].discovered[Resource::Oil.index()] = true;
    sim.economy.holdings[0].bonus_cap[Resource::Wealth.index()] = 42;
    sim.economy.holdings[0].taxation = 4;

    sim.assemble_holdings(0);

    let h = &sim.economy.holdings[0];
    assert_eq!(h.territory, territory);
    assert_eq!(h.land_size, land_size);
    assert!(!h.available[Resource::Oil.index()]);
    assert!(h.discovered[Resource::Oil.index()]);
    assert_eq!(h.bonus_cap[Resource::Wealth.index()], 42);
    // `taxation` was on this list until item 232, and the hand-set 4
    // survived because nothing wrote it. It is assembled now, from the
    // player's `TAX_n` ladder, so a hand-set value is *overwritten* —
    // which is the point of the fix.
    assert_eq!(h.taxation, 0, "assembled from the ladder, not preserved");
}

#[test]
fn a_lost_city_stops_paying() {
    let mut sim = world_sim();
    let t = install(&mut sim);
    let city = build(&mut sim, 0, t.village, 32, 32);
    build(&mut sim, 0, t.farm, 40, 32);
    sim.assemble_holdings(0);
    assert_eq!(sim.economy.holdings[0].cities.len(), 1);
    let with = economy::assemble(&sim.tables.tuning, &sim.economy.holdings[0]);
    assert!(with[0] > 0);

    sim.buildings[city].alive = false;
    let c = sim.buildings[city].city.unwrap();
    sim.cities_state.cities[c].alive = false;
    sim.assemble_holdings(0);
    assert!(sim.economy.holdings[0].cities.is_empty());
    let without = economy::assemble(&sim.tables.tuning, &sim.economy.holdings[0]);
    assert_eq!(without, [0; RESOURCES], "BASIC_GATHER ships as zero");
}

#[test]
fn rate_now_is_the_assembly_without_the_cadence() {
    let mut sim = world_sim();
    let t = install(&mut sim);
    build(&mut sim, 0, t.village, 32, 32);
    let rate = rate_now(&mut sim, 0);
    assert_eq!(rate[0], sim.tables.tuning.city_gather[0] * RATE_SCALE);
    assert_eq!(rate[1], sim.tables.tuning.city_gather[1] * RATE_SCALE);
}
