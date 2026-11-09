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

/// **The CEO's city takes half again** (item 3516,
/// `calc_city_resources@006d5530:166`–`195`): the trade and the sites,
/// every good, scaled by `(THECEO_PRODUCTION_BONUS + 100) / 100` before
/// the city's own flat `CITY_GATHER`, which is not scaled. run1162e's
/// who=1 wealth `income` reads 3448 against 3183 without it.
#[test]
fn the_ceo_s_city_takes_half_again_before_the_flats() {
    let t = Tuning::RON;
    let mut city = City {
        trade_val: 530,
        ..farm_city(3)
    };
    let plain = city_rates(&t, &city);
    city.ceo = true;
    let ceo = city_rates(&t, &city);
    for r in [Resource::Food, Resource::Wealth] {
        let flat = t.city_gather[r.index()] * RATE_SCALE;
        assert_eq!(
            ceo[r.index()] - flat,
            (plain[r.index()] - flat) * 3 / 2,
            "{r:?}: the gather scaled, the flat not"
        );
    }
    assert_eq!(
        ceo[Resource::Wealth.index()] - plain[Resource::Wealth.index()],
        265
    );
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

/// **An Iroquois woodcutter pays two food** (item 3170, `docs/AI.md`
/// §250): run923's who=1, one woodcutter and three farmers, reads food
/// 672 on block 8 — the city's 160, three farms' 480, and the camp's
/// `IROQUOIS_FOOD × 16`.
#[test]
fn an_iroquois_timber_site_pays_food_too() {
    let t = Tuning::RON;
    let city = City {
        sites: vec![
            Site::new(Resource::Timber, 1),
            Site::new(Resource::Food, 1),
            Site::new(Resource::Food, 1),
            Site::new(Resource::Food, 1),
        ],
        ..City::default()
    };
    let mut h = Holdings::new();
    h.cities = vec![city];
    let food = Resource::Food.index();
    assert_eq!(assemble(&t, &h)[food], 640, "anyone else");
    h.iroquois = true;
    assert_eq!(assemble(&t, &h)[food], 672, "the Iroquois");
    assert_eq!(
        assemble(&t, &h)[Resource::Timber.index()],
        320,
        "and the timber is the same"
    );
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
fn a_german_city_adds_five_food_and_timber_and_metal_once_available() {
    // `calc_city_resources@006d5530`'s per-city arm: `GERMAN_CITY_GATHER`
    // (5) × 16 to food and timber, and to metal under `type_avail`; the
    // Romans' `ROMAN_CITY_GATHER` (10) × 16 to wealth. The battery's
    // East Indies Germans read food 880 and timber 720 on one city where
    // ours read 800 and 640 without it (item 2931).
    let t = Tuning::RON;
    let mut h = Holdings::new();
    h.available[Resource::Metal.index()] = false;
    h.cities.push(City::default());
    h.cities.push(City::default());
    let plain = assemble(&t, &h);
    h.germans = true;
    let german = assemble(&t, &h);
    let d = |r: Resource| german[r.index()] - plain[r.index()];
    assert_eq!(d(Resource::Food), 2 * 5 * RATE_SCALE);
    assert_eq!(d(Resource::Timber), 2 * 5 * RATE_SCALE);
    assert_eq!(d(Resource::Metal), 0, "metal not yet available");
    assert_eq!(d(Resource::Wealth), 0);
    h.available[Resource::Metal.index()] = true;
    let metal = assemble(&t, &h);
    assert_eq!(
        metal[Resource::Metal.index()] - plain[Resource::Metal.index()],
        2 * 5 * RATE_SCALE
    );
    h.germans = false;
    h.romans = true;
    let roman = assemble(&t, &h);
    assert_eq!(
        roman[Resource::Wealth.index()] - plain[Resource::Wealth.index()],
        2 * 10 * RATE_SCALE
    );
    assert_eq!(roman[Resource::Food.index()], plain[Resource::Food.index()]);
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
fn the_porcelain_tower_quadruples_a_markets_taxes_and_only_a_markets() {
    // `get_taxes@00737b50`: `(PORCELAIN_MARKET + 100) * market_taxes / 100`
    // under a market, nothing without one (item 2598).
    let t = Tuning::RON;
    let held = City {
        market: true,
        porcelain: true,
        ..City::default()
    };
    assert_eq!(taxes(&t, &held), 40);
    let bare = City {
        porcelain: true,
        ..City::default()
    };
    assert_eq!(taxes(&t, &bare), 0);
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
            nation_pct: 0,
            farm_wealth: 0,
            miner_wealth: 0,
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
fn a_dutch_stockpile_past_its_grant_earns_interest_past_the_cap() {
    // `do_gather@006ce450:116`–`147`: five percent of the stockpile past
    // the starting grant, in whole resources, after the cap and capped
    // fifty past it; knowledge earns none.
    let t = Tuning::RON;
    let mut h = Holdings::new();
    h.dutch = true;
    h.starting = [200; RESOURCES];
    let mut l = Ledger::default();
    let (food, know) = (Resource::Food.index(), Resource::Knowledge.index());
    l.rate[food] = 10 * RATE_SCALE;
    l.cap[food] = 70 * RATE_SCALE;
    l.rate[know] = 10 * RATE_SCALE;
    l.cap[know] = CAP_CEILING * RATE_SCALE;
    l.bucket[food] = 439;
    l.bucket[know] = 439;
    pay(&t, &mut l, &h, 1);
    // 239 over: 5 × 239 / 100 = 11 whole resources.
    assert_eq!(l.income[food], 21 * RATE_SCALE);
    assert_eq!(l.income[know], 10 * RATE_SCALE);
    // A pile at its grant earns nothing.
    l.bucket[food] = 200;
    pay(&t, &mut l, &h, 2);
    assert_eq!(l.income[food], 10 * RATE_SCALE);
    // A capped rate passes the cap by fifty and no more.
    l.rate[food] = 100 * RATE_SCALE;
    l.bucket[food] = 20_000;
    pay(&t, &mut l, &h, 3);
    assert_eq!(l.income[food], (70 + 50) * RATE_SCALE);
    // And a leader without the power earns none.
    h.dutch = false;
    l.bucket[food] = 439;
    l.rate[food] = 10 * RATE_SCALE;
    pay(&t, &mut l, &h, 4);
    assert_eq!(l.income[food], 10 * RATE_SCALE);
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
            nation_pct: 0,
            farm_wealth: 0,
            miner_wealth: 0,
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
    resource_bonuses(&t, 0, false, &mut out);
    assert_eq!(out, [1600, 2560, 1024, 1840, 1120, 0]);
    resource_bonuses(&t, 1 << crate::tech::wonder::PYRAMIDS, false, &mut out);
    assert_eq!(out, [1920, 2560, 1024, 1840, 1120, 0]);
}

/// `calc_resource_bonuses@006db030`'s first term: a Russian leader's oil
/// rate is `(RUSSIAN_OIL + 100) * oil / 100`, before every wonder term.
#[test]
fn a_russian_leader_pays_a_fifth_more_oil() {
    let t = Tuning::RON;
    let mut out = [0, 0, 0, 0, 0, 1000];
    resource_bonuses(&t, 0, false, &mut out);
    assert_eq!(out[5], 1000);
    resource_bonuses(&t, 0, true, &mut out);
    assert_eq!(out[5], 1200);
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

/// Step 8: Capitalism's `CAPITALISM_OIL_PROD × 16` lands **after** the
/// refinery multiply, so two refineries leave it whole. run879's
/// who=1 reads `resources[oil]` 9966 = 8366 + 1600 on 5976: nine
/// worked wells outside every city (`9 × 560`, `× 166 / 100`) and the
/// term (`docs/AI.md` §234).
#[test]
fn capitalism_adds_its_oil_after_the_refineries() {
    let t = Tuning::RON;
    let mut h = Holdings::new();
    h.refineries = 2;
    for _ in 0..9 {
        let mut s = Site::new(Resource::Oil, 1);
        s.gatherers = 1;
        h.outside.push(s);
    }
    let oil = Resource::Oil.index();
    assert_eq!(assemble(&t, &h)[oil], 8366);
    h.capitalism = true;
    assert_eq!(assemble(&t, &h)[oil], 9966);
    assert_eq!(t.capitalism_oil_prod * RATE_SCALE, 1600);
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

/// **The Mongols' food** (`docs/AI.md` §191): two nations, 287 of 1669
/// tiles, `2 × 287 × 800 / 1669` is the 275 sixteenths that run776's
/// Mongol AI reads above the 160 its farmers pay (435).
#[test]
fn the_mongols_take_food_from_their_land_and_nobody_else_does() {
    let t = Tuning::RON;
    let mut h = Holdings::new();
    h.land_size = 1669;
    h.territory = 287;
    h.num_nations = 2;
    assert_eq!(mongol_nomadic_food(&t, &h), 0);
    h.mongols = true;
    assert_eq!(mongol_nomadic_food(&t, &h), 275);
    h.land_size = 0;
    assert_eq!(mongol_nomadic_food(&t, &h), 0);
}

/// **The Americans' Barracks gather** (`docs/AI.md` §292): each
/// Barracks unit outside a building pays `2 × 16` sixteenths of food,
/// timber, metal and wealth, each only while it is available — and
/// nothing for a leader that is not American.
#[test]
fn an_american_barracks_unit_gathers_two_of_each_available_resource() {
    let t = Tuning::RON;
    let mut h = Holdings::new();
    h.available = [true, true, true, false, false, false];
    h.american_units = 3;
    let base = assemble(&t, &h);
    h.americans = true;
    assert_eq!(american_barracks_gather(&t, &h), 3 * 2 * RATE_SCALE);
    let with = assemble(&t, &h);
    let gained: Vec<i32> = (0..RESOURCES).map(|r| with[r] - base[r]).collect();
    assert_eq!(gained, [96, 96, 96, 0, 0, 0]);
    h.available[Resource::Metal.index()] = true;
    let metal = assemble(&t, &h);
    assert_eq!(
        metal[Resource::Metal.index()] - base[Resource::Metal.index()],
        96
    );
}

/// **The Lakota's food** (`docs/AI.md` §219): run846's who=1 on block
/// 1 holds ten citizens and a scout, none garrisoned — `11 × 4 × 16 =
/// 704` sixteenths, the whole of its 864 against the human's 160.
#[test]
fn the_lakota_eat_off_their_citizens_and_scouts_and_nobody_else_does() {
    let t = Tuning::RON;
    let mut h = Holdings::new();
    h.lakota_units = 11;
    assert_eq!(lakota_food(&t, &h), 0);
    h.lakota = true;
    assert_eq!(lakota_food(&t, &h), 704);
    assert_eq!(assemble(&t, &h)[Resource::Food.index()], 704);
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
