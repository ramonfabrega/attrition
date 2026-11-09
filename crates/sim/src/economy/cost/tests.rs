use super::*;

const T: Tuning = Tuning::RON;

/// The Citizen, from `unitrules.xml`: `2f`, `1f support`, `PROGRESSION 0`.
fn citizen() -> Price {
    Price {
        class: RampClass::Worker,
        pop: 1,
        ..Price::free()
            .with_base(Resource::Food, 2)
            .with_support(Resource::Food, 1)
    }
}

/// The Hoplite: `5f/3m`, `1f/1m support`, `PROGRESSION 3`.
fn hoplite() -> Price {
    Price {
        progression: Progression::PROGRESSIVE_BY_GROUP,
        class: RampClass::Military,
        pop: 1,
        ..Price::free()
            .with_base(Resource::Food, 5)
            .with_base(Resource::Metal, 3)
            .with_support(Resource::Food, 1)
            .with_support(Resource::Metal, 1)
    }
}

/// The Scholar: `3g`, `2g support`, `PROGRESSION 0`.
fn scholar() -> Price {
    Price {
        class: RampClass::Scholar,
        pop: 1,
        ..Price::free()
            .with_base(Resource::Wealth, 3)
            .with_support(Resource::Wealth, 2)
    }
}

/// The Small City, from `buildingrules.xml`: `COST 1t/1f`,
/// `SUPPORT food 50 / timber 50`, and no progression column at all.
fn small_city() -> Price {
    Price {
        kind: Kind::Building,
        class: RampClass::Building,
        ..Price::free()
            .with_base(Resource::Food, 1)
            .with_base(Resource::Timber, 1)
            .with_support(Resource::Food, 50)
            .with_support(Resource::Timber, 50)
    }
}

fn owned(n: i32) -> Counts {
    Counts {
        of_type: n,
        of_group: n,
    }
}

fn plain(price: &Price, r: Resource, counts: Counts) -> i32 {
    cost_of(&T, price, r, counts, &Modifiers::default())
}

/// **A building's ramp has no ceiling**, and that is the difference
/// between an AI that founds its second city on frame 576 and one that
/// founds it on 776.
///
/// `TypeData::get_cost` has two ramps. The unit arm reads
/// `UNIT_COST_FACTOR` (`+0x354`) and one of the four `*_RAMP_MAX`
/// (`+0x394`..`+0x3a0`); the building arm (`00665787`..`00665b5a`) reads
/// `BUILD_COST_FACTOR` (`+0x358`) and `BUILD_SUPPORT_FACTOR` (`+0x37c`)
/// and loads no `RAMP_MAX` at all. With the military 125% applied — which
/// is what `RampClass::default()` gave every building — the Small City's
/// `SUPPORT 50` clamps to 12 and the second city costs 22 instead of 60.
///
/// Measured, not inferred: run40 and run41 (`rondata::diff`) put the AI
/// at 69 food / 59 timber on the frame it cannot buy and 83 / 73 on the
/// frame it can, and its buckets fall by sixty of each when it does.
#[test]
fn a_fort_and_a_barracks_escalate_their_count() {
    // The Fort line: 0, 1, then 2n−1 — the Redoubt's third is priced as
    // three (run767's 175 / 525 on block 3182).
    let fort: Vec<i32> = (0..9).map(|n| building_family_count(0x1be, n)).collect();
    assert_eq!(fort, [0, 1, 3, 5, 8, 11, 15, 19, 24]);
    // Production: free to two, then 2n−2 …
    let rax: Vec<i32> = (0..14).map(|n| building_family_count(0x1ab, n)).collect();
    assert_eq!(rax, [0, 1, 2, 4, 6, 8, 12, 16, 20, 27, 34, 41, 48, 59]);
    // Anything else is linear.
    assert_eq!(building_family_count(0x1b0, 9), 9);
}

/// **Salt takes a tenth off a Barracks unit's scaled base, ahead of
/// the ramp** (item 3318, `docs/AI.md` §264): run958d's Spanish
/// Hoplites at `1/2019`, three and four of the group ahead of them and
/// one Military level ahead (`MILITARY_UNIT_DISCOUNT` 5), are 48/31 and
/// 52/35 on block 7783; the list price is 53/34 and 57/38. The tenth
/// taken after the ramp would make the first 47/30, and Salt's written
/// 15% before it 45/29.
#[test]
fn salt_cuts_a_tenth_before_the_ramp() {
    let h = hoplite();
    let m = |salt| Modifiers {
        salt,
        late_discount: 5,
        ..Modifiers::default()
    };
    let both =
        |n, salt| [Resource::Food, Resource::Metal].map(|r| cost_of(&T, &h, r, owned(n), &m(salt)));
    assert_eq!(both(3, false), [53, 34]);
    assert_eq!(both(4, false), [57, 38]);
    assert_eq!(both(3, true), [48, 31]);
    assert_eq!(both(4, true), [52, 35]);
}

/// **The rares' tail** (item 3343): `get_cost`'s shared tail takes
/// Sugar's tenth off the food alone, after every other term (run978f's
/// Enlightenment Age: 425 food after Science, 382 with Sugar, its 850
/// knowledge untouched) — and Gypsum's off every good after it.
#[test]
fn sugar_takes_a_tenth_off_the_food_alone_and_gypsum_off_everything() {
    let age = Price {
        kind: Kind::Tech,
        ..Price::free()
    }
    .with_base(Resource::Food, 50)
    .with_base(Resource::Knowledge, 100);
    let mut sugar = Modifiers::default();
    sugar.rare_goods[Resource::Food.index()] = 10;
    let at = |m: &Modifiers| {
        [Resource::Food, Resource::Knowledge].map(|r| cost_of(&T, &age, r, owned(0), m))
    };
    assert_eq!(at(&Modifiers::default()), [500, 1000]);
    assert_eq!(at(&sugar), [450, 1000]);
    let gypsum = Modifiers {
        gypsum: 10,
        ..sugar
    };
    assert_eq!(at(&gypsum), [405, 900]);
}

/// **The Greeks' research and Library arms** (item 3326, `docs/AI.md`
/// §266): `get_cost:1254` takes a tenth off a tech's price in every good
/// but knowledge, and `get_cost:970` two thirds off a Library or a
/// University — run961c's City State at food 108 for 120, and run961b's
/// University at timber 27 and wealth 17 for 80 and 50.
#[test]
fn the_greeks_take_a_tenth_off_research_and_two_thirds_off_a_library() {
    let tech = Price {
        kind: Kind::Tech,
        ..Price::free()
    }
    .with_base(Resource::Food, 120)
    .with_base(Resource::Knowledge, 120);
    let greek = Modifiers {
        greek_research: 10,
        ..Modifiers::default()
    };
    for r in [Resource::Food, Resource::Knowledge] {
        let plain = cost_of(&T, &tech, r, owned(0), &Modifiers::default());
        let off = cost_of(&T, &tech, r, owned(0), &greek);
        let want = if r == Resource::Knowledge {
            plain
        } else {
            90 * plain / 100
        };
        assert_eq!(off, want, "{r:?}");
    }
    let university = small_city()
        .with_base(Resource::Timber, 80)
        .with_base(Resource::Wealth, 50);
    let library = Modifiers {
        greek_library: 66,
        ..Modifiers::default()
    };
    for r in [Resource::Timber, Resource::Wealth] {
        let plain = cost_of(&T, &university, r, owned(0), &Modifiers::default());
        let off = cost_of(&T, &university, r, owned(0), &library);
        assert_eq!(off, 34 * plain / 100, "{r:?}");
        assert_ne!(off, plain);
    }
}

/// `get_cost:1003`–`1008`: a Japanese Farm at `(100 − 50) × cost / 100`
/// in every good, after the Greek Library arm (item 3331).
#[test]
fn the_japanese_pay_half_for_a_farm() {
    let farm = small_city().with_base(Resource::Timber, 55);
    let japanese = Modifiers {
        japanese_farms: T.japanese_farms_cheaper,
        ..Modifiers::default()
    };
    assert_eq!(T.japanese_farms_cheaper, 50);
    for r in [Resource::Food, Resource::Timber] {
        for n in 0..3 {
            let plain = cost_of(&T, &farm, r, owned(n), &Modifiers::default());
            let off = cost_of(&T, &farm, r, owned(n), &japanese);
            assert_eq!(off, plain / 2, "{r:?} {n}");
        }
    }
}

/// The Indian ramp arm (`get_cost:843`–`876`, item 3335): the support
/// term is zeroed for an ordinary building and halved rounding up for
/// a wonder or a defence; the base stands either way.
#[test]
fn an_indian_building_does_not_ramp_and_a_defence_ramps_half() {
    let farm = small_city();
    let with = |r| Modifiers {
        indian_ramp: r,
        ..Modifiers::default()
    };
    for n in 0..5 {
        let plain = cost_of(&T, &farm, Resource::Food, owned(n), &Modifiers::default());
        let zero = cost_of(&T, &farm, Resource::Food, owned(n), &with(IndianRamp::Zero));
        let half = cost_of(&T, &farm, Resource::Food, owned(n), &with(IndianRamp::Half));
        assert_eq!(zero, 10, "{n}: the base alone");
        assert_eq!(half, 10 + (plain - 10 + 1) / 2, "{n}");
    }
    assert_eq!(
        cost_of(&T, &farm, Resource::Food, owned(1), &with(IndianRamp::Half)),
        35
    );
}

#[test]
fn a_building_s_ramp_is_not_capped() {
    let city = small_city();
    // The first is free of the ramp, as everything's first is.
    assert_eq!(plain(&city, Resource::Food, owned(0)), 10);
    assert_eq!(plain(&city, Resource::Timber, owned(0)), 10);
    // The second: 10 + 50 x BUILD_SUPPORT_FACTOR x 1, and the military
    // ceiling would have made it 10 + 12.
    assert_eq!(plain(&city, Resource::Food, owned(1)), 60);
    assert_eq!(plain(&city, Resource::Timber, owned(1)), 60);
    assert_eq!(plain(&city, Resource::Metal, owned(1)), 0);
    // And it keeps climbing, linearly, for ever.
    assert_eq!(plain(&city, Resource::Food, owned(4)), 210);
    let capped = Price {
        class: RampClass::Military,
        ..small_city()
    };
    assert_eq!(
        plain(&capped, Resource::Food, owned(4)),
        22,
        "what this crate priced it at before item 81"
    );
}

/// **The Pyramids take a third off a city**, after the ramp
/// (`get_cost`'s city tail, `00665c46`..`00665c9b`). French East Indies'
/// AI holds the Pyramids and four cities on frame 9777 and pays 140 food
/// and 140 timber for the fifth (run623's buckets, 501 → 361 and
/// 158 → 18), where the bare ramp asks 210. Bantu's term comes first and
/// truncates on its own.
#[test]
fn the_pyramids_take_a_third_off_a_city() {
    let city = small_city();
    let with = |city_pct: [i32; 2]| Modifiers {
        city: city_pct,
        ..Modifiers::default()
    };
    let cost = |m: &Modifiers, r| cost_of(&T, &city, r, owned(4), m);
    assert_eq!(cost(&with([0, 0]), Resource::Food), 210);
    let pyramids = with([0, T.pyramids_city_discount]);
    assert_eq!(cost(&pyramids, Resource::Food), 140);
    assert_eq!(cost(&pyramids, Resource::Timber), 140);
    // Bantu (75 off) then the Pyramids: 210 → 52 → 34, each truncated.
    let both = with([T.bantu_city_cost, T.pyramids_city_discount]);
    assert_eq!(cost(&both, Resource::Food), 34);
}

/// **`get_cost`'s wonder arm, from the listing** (`00665848`..`0066594c`
/// and the halving at `00665af9`; `docs/COSTS.md`, "A wonder is ramped by every wonder"). A wonder is
/// ramped by every wonder the leader holds or has a site of, not by
/// its own type, and the term is halved after the count. Up to three
/// held that is one step a wonder, which run289's frame 20781 measures
/// (item 890); past three the count gains `held − 3` and past six
/// `held − 6`, and an odd product truncates — no capture reaches either,
/// so they are pinned here from the listing alone.
#[test]
fn a_wonder_is_ramped_by_every_wonder_held_and_halved() {
    // The count, held 0..=8, and the space-race arm's two tails.
    let c: Vec<i32> = (0..=8).map(|n| wonder_count(n, SpaceRace::None)).collect();
    assert_eq!(c, [0, 2, 4, 6, 9, 12, 15, 19, 23]);
    assert_eq!(wonder_count(1, SpaceRace::Own), 4, "my own site: held + 1");
    assert_eq!(wonder_count(4, SpaceRace::Own), 11, "the tail reads held");
    assert_eq!(wonder_count(1, SpaceRace::Team), 3, "a teammate's: a step");
    // An odd support amount, so the halving truncates past three.
    let wonder = Price {
        kind: Kind::Building,
        class: RampClass::Building,
        ..Price::free()
            .with_base(Resource::Wealth, 20)
            .with_support(Resource::Wealth, 45)
    };
    let m = Modifiers {
        wonder: true,
        ..Modifiers::default()
    };
    let at = |held: i32| {
        let counts = Counts {
            of_type: wonder_count(held, SpaceRace::None),
            of_group: 0,
        };
        cost_of(&T, &wonder, Resource::Wealth, counts, &m)
    };
    assert_eq!(at(0), 200);
    // One held (run289's Pyramids site): 45 x 2 / 2, one whole step.
    assert_eq!(at(1), 245);
    assert_eq!(at(3), 200 + 135);
    // Four: 45 x 9 / 2 = 202, where a count of four steps would be 180.
    assert_eq!(at(4), 200 + 202);
    assert_eq!(at(7), 200 + 45 * 19 / 2);
    // Without the flag the same count is an ordinary building's steps.
    let counts = Counts {
        of_type: wonder_count(4, SpaceRace::None),
        of_group: 0,
    };
    assert_eq!(plain(&wonder, Resource::Wealth, counts), 200 + 45 * 9);
}

/// `get_cost`'s Egyptian wonder tail (`00665ced`..): a quarter off the
/// whole wonder price, one `(100 - x) * cost / 100` after the ramp.
#[test]
fn an_egyptian_wonder_costs_a_quarter_less() {
    let wonder = Price {
        kind: Kind::Building,
        class: RampClass::Building,
        ..Price::free()
            .with_base(Resource::Wealth, 20)
            .with_support(Resource::Wealth, 45)
    };
    let price = |egyptian_wonders: i32| {
        let m = Modifiers {
            wonder: true,
            egyptian_wonders,
            ..Modifiers::default()
        };
        let counts = Counts {
            of_type: wonder_count(4, SpaceRace::None),
            of_group: 0,
        };
        cost_of(&T, &wonder, Resource::Wealth, counts, &m)
    };
    assert_eq!(price(0), 402);
    assert_eq!(price(T.egyptian_wonders), (100 - 25) * 402 / 100);
}

/// `get_cost`'s fort arm (`00665e0f`..`00665e8a`, item 3314): the
/// Colosseum's 20 % and then the Romans' 25 % off a fort or a Tower,
/// each its own truncating step after the ramp.
#[test]
fn a_roman_tower_costs_a_quarter_less_after_the_colosseum_s_fifth() {
    let tower = Price {
        kind: Kind::Building,
        class: RampClass::Building,
        ..Price::free().with_base(Resource::Timber, 13)
    };
    let price = |fort: [i32; 2]| {
        let m = Modifiers {
            fort,
            ..Modifiers::default()
        };
        let counts = Counts {
            of_type: 0,
            of_group: 0,
        };
        cost_of(&T, &tower, Resource::Timber, counts, &m)
    };
    let base = price([0, 0]);
    assert_eq!(price([0, T.roman_fort_cost]), (100 - 25) * base / 100);
    assert_eq!(
        price([T.colosseum_fort_cost, T.roman_fort_cost]),
        (100 - 25) * ((100 - 20) * base / 100) / 100
    );
}

/// `get_cost:916`–`922` (item 3365): a Korean leader's Tower-line
/// building loses `KOREAN_TOWER_COST` percent in its own truncating step,
/// ahead of the fort arm's.
#[test]
fn a_korean_tower_costs_a_third_less_before_the_fort_arm() {
    let tower = Price {
        kind: Kind::Building,
        class: RampClass::Building,
        ..Price::free().with_base(Resource::Metal, 50)
    };
    let price = |korean_tower: i32, fort: [i32; 2]| {
        let m = Modifiers {
            korean_tower,
            fort,
            ..Modifiers::default()
        };
        let counts = Counts {
            of_type: 0,
            of_group: 0,
        };
        cost_of(&T, &tower, Resource::Metal, counts, &m)
    };
    let base = price(0, [0, 0]);
    assert_eq!(price(T.korean_tower_cost, [0, 0]), (100 - 33) * base / 100);
    assert_eq!(
        price(T.korean_tower_cost, [T.colosseum_fort_cost, 0]),
        (100 - 20) * ((100 - 33) * base / 100) / 100
    );
}

/// `get_cost:268`–`282`: the American aircraft arm's three tests are
/// each their own truncating step, so two that enter stack (item 3755).
#[test]
fn an_american_aircraft_arm_stacks_its_steps() {
    let plane = Price {
        kind: Kind::Unit,
        ..Price::free().with_base(Resource::Wealth, 9)
    };
    let price = |american_aircraft: [i32; 3]| {
        let m = Modifiers {
            american_aircraft,
            ..Modifiers::default()
        };
        cost_of(&T, &plane, Resource::Wealth, owned(0), &m)
    };
    let plain = price([0; 3]);
    let k = T.americans_aircraft;
    assert_eq!(k, 20);
    let once = (100 - k) * plain / 100;
    assert_eq!(price([k, 0, 0]), once);
    assert_eq!(price([0, 0, k]), once);
    assert_eq!(price([k, 0, k]), (100 - k) * once / 100);
}

/// `get_cost:283`: a British AA Gun loses `BRITISH_AA_COST` percent in
/// its own truncating step, ahead of the Russian spy's and the
/// Spy/General arm's (item 3678).
#[test]
fn a_british_aa_gun_costs_a_quarter_less() {
    let aa = Price {
        kind: Kind::Unit,
        ..Price::free().with_base(Resource::Metal, 4)
    };
    let price = |british_aa: i32, discount: i32| {
        let m = Modifiers {
            british_aa,
            discount,
            ..Modifiers::default()
        };
        cost_of(&T, &aa, Resource::Metal, owned(0), &m)
    };
    let plain = price(0, 0);
    assert_eq!(T.british_aa_cost, 25);
    assert_eq!(price(T.british_aa_cost, 0), (100 - 25) * plain / 100);
    assert_eq!(
        price(T.british_aa_cost, 50),
        ((100 - 25) * plain / 100) * (100 - 50) / 100
    );
}

/// `get_cost:299`: a Russian Spy loses `RUSSIAN_SPY_COST` percent in its
/// own truncating step, ahead of the Spy/General arm's.
#[test]
fn a_russian_spy_costs_half() {
    let spy = Price {
        kind: Kind::Unit,
        ..Price::free().with_base(Resource::Wealth, 5)
    };
    let price = |russian_spy: i32, discount: i32| {
        let m = Modifiers {
            russian_spy,
            discount,
            ..Modifiers::default()
        };
        cost_of(&T, &spy, Resource::Wealth, owned(0), &m)
    };
    let plain = price(0, 0);
    assert_eq!(price(T.russian_spy_cost, 0), (100 - 50) * plain / 100);
    assert_eq!(
        price(T.russian_spy_cost, 50),
        (100 - 50) * ((100 - 50) * plain / 100) / 100
    );
}

/// `get_cost`'s Citizen arms: Wool's 15% then the Turks' 33%, each its
/// own truncating step.
#[test]
fn a_turkish_citizen_costs_a_third_less() {
    let citizen = Price {
        kind: Kind::Unit,
        ..Price::free().with_base(Resource::Food, 5)
    };
    let price = |citizens: [i32; 2]| {
        let m = Modifiers {
            citizens,
            ..Modifiers::default()
        };
        cost_of(&T, &citizen, Resource::Food, owned(0), &m)
    };
    let plain = price([0, 0]);
    assert!(plain > 0);
    assert_eq!(price([0, 33]), (100 - 33) * plain / 100);
    assert_eq!(
        price([15, 33]),
        (100 - 33) * ((100 - 15) * plain / 100) / 100
    );
}

/// `get_cost:306`: a Roman Legion loses `ROMAN_LEGION_CHEAP` percent in
/// its own truncating step.
#[test]
fn a_roman_legion_costs_a_tenth_less() {
    let legion = Price {
        kind: Kind::Unit,
        ..Price::free().with_base(Resource::Food, 5)
    };
    let price = |roman_legion: i32| {
        let m = Modifiers {
            roman_legion,
            ..Modifiers::default()
        };
        cost_of(&T, &legion, Resource::Food, owned(0), &m)
    };
    let plain = price(0);
    assert!(plain > 0);
    assert_eq!(price(T.roman_legion_cheap), (100 - 10) * plain / 100);
    assert!(price(T.roman_legion_cheap) < plain);
}

/// `get_cost:530`: a Dutch ship upgrade loses `DUTCH_SHIP_DISCOUNT`
/// percent in a truncating step after the Military discount.
#[test]
fn a_dutch_ship_upgrade_costs_a_tenth_less() {
    let upgrade = Price {
        kind: Kind::Unit,
        ..Price::free().with_base(Resource::Food, 5)
    };
    let price = |dutch_ship: i32| {
        let m = Modifiers {
            research: Some(Research {
                premium_cost: 256,
                discount: 20,
                dutch_ship,
                ..Research::default()
            }),
            ..Modifiers::default()
        };
        cost_of(&T, &upgrade, Resource::Food, owned(0), &m)
    };
    let plain = price(0);
    assert!(plain > 0);
    assert_eq!(price(T.dutch_ship_discount), (100 - 10) * plain / 100);
}

/// `get_cost:528`: an American land-unit upgrade loses
/// `AMERICANS_UPGRADE_DISCOUNT` percent in a truncating step after the
/// Military discount, before the Dutch arm's (item 3374).
#[test]
fn an_american_land_upgrade_costs_a_quarter_less() {
    let upgrade = Price {
        kind: Kind::Unit,
        ..Price::free().with_base(Resource::Food, 8)
    };
    let price = |american_land: i32| {
        let m = Modifiers {
            research: Some(Research {
                premium_cost: 256,
                american_land,
                ..Research::default()
            }),
            ..Modifiers::default()
        };
        cost_of(&T, &upgrade, Resource::Food, owned(0), &m)
    };
    let plain = price(0);
    assert!(plain > 0);
    assert_eq!(
        price(T.americans_upgrade_discount),
        (100 - 25) * plain / 100
    );
}

#[test]
fn base_cost_is_ten_times_the_file() {
    assert_eq!(plain(&citizen(), Resource::Food, owned(0)), 20);
    assert_eq!(plain(&hoplite(), Resource::Food, owned(0)), 50);
    assert_eq!(plain(&hoplite(), Resource::Metal, owned(0)), 30);
    assert_eq!(plain(&scholar(), Resource::Wealth, owned(0)), 30);
}

/// The first one is never ramped: the count is what you already have, and
/// the guard in the original is `count > 0`.
#[test]
fn the_first_of_anything_is_unramped() {
    for n in [0, 1, 2] {
        assert_eq!(plain(&citizen(), Resource::Food, owned(n)), 20 + n);
    }
}

#[test]
fn a_citizen_ramps_by_one_food_each_to_five_times_its_base() {
    let c = citizen();
    assert_eq!(plain(&c, Resource::Food, owned(19)), 39);
    assert_eq!(plain(&c, Resource::Food, owned(49)), 69);
    // 500% of the scaled base is a hundred, and nothing past it moves.
    assert_eq!(plain(&c, Resource::Food, owned(100)), 120);
    assert_eq!(plain(&c, Resource::Food, owned(500)), 120);
}

/// The triangular count reaches the 125% ceiling at eleven, and a hoplite
/// never costs more than the twelfth one did.
#[test]
fn a_military_ramp_is_triangular_and_plateaus() {
    let h = hoplite();
    assert_eq!(plain(&h, Resource::Food, owned(1)), 51);
    assert_eq!(plain(&h, Resource::Food, owned(10)), 50 + 55);
    assert_eq!(plain(&h, Resource::Food, owned(11)), 50 + 62);
    assert_eq!(plain(&h, Resource::Food, owned(100)), 112);
    assert_eq!(plain(&h, Resource::Metal, owned(100)), 67);
}

/// Elephants are the four records that ramp triangularly against their own
/// type rather than the stable's whole output.
#[test]
fn progression_picks_which_count_is_read() {
    let by_group = Progression::PROGRESSIVE_BY_GROUP;
    let by_type = Progression::PROGRESSIVE_BY_TYPE;
    let counts = Counts {
        of_type: 2,
        of_group: 10,
    };
    assert_eq!(counts.for_progression(by_group), 10);
    assert_eq!(counts.for_progression(by_type), 2);
    assert_eq!(ramp_steps(4, by_type), 10);
    assert_eq!(ramp_steps(4, Progression::BY_TYPE), 4);
}

#[test]
fn a_scholar_carries_a_second_term_every_seven() {
    assert_eq!(scholar_surcharge(7), 0);
    assert_eq!(scholar_surcharge(8), 1);
    assert_eq!(scholar_surcharge(21), 21);
    let s = scholar();
    // The eighth scholar: seven owned, two wealth each, no second term yet.
    assert_eq!(plain(&s, Resource::Wealth, owned(7)), 44);
    // The ninth: the term opens.
    assert_eq!(plain(&s, Resource::Wealth, owned(8)), 47);
    assert_eq!(plain(&s, Resource::Wealth, owned(21)), 93);
}

/// The ceiling is a percentage of the base *before* the discount, so a
/// discount does not shrink the ramp's headroom.
#[test]
fn the_ceiling_ignores_the_discount() {
    let c = citizen();
    let m = Modifiers {
        discount: 50,
        ..Modifiers::default()
    };
    assert_eq!(cost_of(&T, &c, Resource::Food, owned(500), &m), 10 + 100);
}

/// **Horses, then Rubber, each truncating** (`docs/AI.md` §62). The
/// Light Horse, 60 food and 40 timber, costs 51 and 34 to a Horses
/// holder — East Indies 10980's price, which makes two affordable on
/// 53 food and 70 timber. With Rubber as well the second 15% is taken
/// off 51, not folded into 30% off 60: 43, not 42.
#[test]
fn horses_and_rubber_each_take_their_own_percentage() {
    let lh = Price {
        class: RampClass::Military,
        pop: 1,
        ..Price::free()
            .with_base(Resource::Food, 6)
            .with_base(Resource::Timber, 4)
    };
    let at = |r, pcts| {
        let m = Modifiers {
            stable_rares: pcts,
            ..Modifiers::default()
        };
        cost_of(&T, &lh, r, Counts::default(), &m)
    };
    assert_eq!(at(Resource::Food, [0, 0, 0]), 60);
    assert_eq!(at(Resource::Food, [15, 0, 0]), 51);
    assert_eq!(at(Resource::Timber, [15, 0, 0]), 34);
    assert_eq!(at(Resource::Food, [15, 15, 0]), 43);
    // Sulphur is its own third step: 15% then 20% off 60.
    assert_eq!(at(Resource::Food, [15, 0, 20]), 40);
}

/// **Monarchy takes its quarter after `MILITARY_UNIT_DISCOUNT`, in a
/// truncation of its own** (`get_cost@00664090:657`–`665`; item 2331,
/// `docs/GOLDEN.md` §84). run845's second Light Horse is 61 food ramped,
/// 45 under Monarchy alone; under a 10% military discount as well it is
/// 54 then 40, where one fold of 35% would give 39.
#[test]
fn monarchy_takes_its_quarter_after_the_military_discount() {
    let horse = Price {
        class: RampClass::Military,
        pop: 1,
        ..Price::free()
            .with_base(Resource::Food, 6)
            .with_support(Resource::Food, 1)
    };
    let at = |late_discount, monarchy| {
        let m = Modifiers {
            late_discount,
            monarchy,
            ..Modifiers::default()
        };
        cost_of(&T, &horse, Resource::Food, owned(1), &m)
    };
    assert_eq!(at(0, 0), 61);
    assert_eq!(at(0, 25), 45);
    assert_eq!(at(10, 25), 40);
    assert_eq!(at(35, 0), 39);
}

/// **Socialism takes its quarter last, after the military discount and
/// Monarchy, each in its own truncation** (`get_cost@00664090:632`–`672`;
/// item 2357, `docs/GOLDEN.md` §85). run852's Catapult B is 90 timber
/// ramped under a 10% military discount: 81, then 60 under Socialism,
/// where one fold of 35% would give 58. Light Horse A's food is 70: 63
/// under the discount and 47 under Monarchy after it, where Monarchy
/// first would give 52 and then 46.
#[test]
fn socialism_takes_its_quarter_after_the_military_discount() {
    let ramped = |base: i32, support: i32| Price {
        class: RampClass::Military,
        pop: 1,
        ..Price::free()
            .with_base(Resource::Food, base)
            .with_support(Resource::Food, support)
    };
    let at = |p: &Price, n, late_discount, monarchy, socialism| {
        let m = Modifiers {
            late_discount,
            monarchy,
            socialism,
            ..Modifiers::default()
        };
        cost_of(&T, p, Resource::Food, owned(n), &m)
    };
    let catapult = ramped(7, 10);
    assert_eq!(at(&catapult, 2, 0, 0, 0), 90);
    assert_eq!(at(&catapult, 2, 10, 0, 0), 81);
    assert_eq!(at(&catapult, 2, 10, 0, 25), 60);
    assert_eq!(at(&catapult, 2, 35, 0, 0), 58);
    let horse = ramped(6, 2);
    assert_eq!(at(&horse, 5, 0, 0, 0), 70);
    assert_eq!(at(&horse, 5, 10, 25, 0), 47);
    assert_eq!(at(&horse, 5, 25, 0, 0), 52);
    assert_eq!(
        at(&horse, 5, 10, 25, 25),
        35,
        "Monarchy and Socialism both, in order"
    );
}

/// **The Statue of Liberty takes its third off the base, not the
/// ramp** (`docs/AI.md` §204). Great Sahara's Persians queue a
/// Strategic Bomber (110 wealth and oil, five a unit up the ramp, one
/// percent of military discount) with thirteen and then fourteen held
/// and queued on 3982: 136 and 141 at `1/2047` and `1/2053`, where the
/// whole price less a third would be 115 and 119, and no Statue 173
/// and 178.
#[test]
fn the_statue_of_liberty_takes_a_third_off_a_bomber_before_the_ramp() {
    let bomber = Price {
        class: RampClass::Military,
        pop: 1,
        ..Price::free()
            .with_base(Resource::Wealth, 11)
            .with_support(Resource::Wealth, 5)
    };
    let at = |n, liberty| {
        let m = Modifiers {
            liberty,
            late_discount: 1,
            ..Modifiers::default()
        };
        cost_of(&T, &bomber, Resource::Wealth, owned(n), &m)
    };
    assert_eq!((at(13, 0), at(14, 0)), (173, 178));
    assert_eq!((at(13, 33), at(14, 33)), (136, 141));
}

#[test]
fn maize_halves_the_ramp_and_nothing_else() {
    let c = citizen();
    let m = Modifiers {
        maize: true,
        ..Modifiers::default()
    };
    assert_eq!(cost_of(&T, &c, Resource::Food, owned(40), &m), 20 + 20);
}

#[test]
fn salmon_takes_its_fifth_after_socialism() {
    let c = citizen();
    let at = |salmon, socialism| {
        let m = Modifiers {
            salmon,
            socialism,
            ..Modifiers::default()
        };
        cost_of(&T, &c, Resource::Food, owned(0), &m)
    };
    let base = at(0, 0);
    assert_eq!(at(20, 0), (100 - 20) * base / 100);
    assert_eq!(at(20, 25), (100 - 20) * ((100 - 25) * base / 100) / 100);
}

#[test]
fn a_captured_building_doubles_the_finished_price() {
    let c = citizen();
    let m = Modifiers {
        unassimilated: true,
        ..Modifiers::default()
    };
    assert_eq!(cost_of(&T, &c, Resource::Food, owned(10), &m), 60);
}

/// The Indian elephant arm (`get_cost:1556`–`1576`, item 3335) is the
/// listing's last step: fifteen percent off the doubled price, not
/// doubled after it. run970d's War Elephant is 96/56 → 81/47.
#[test]
fn an_indian_elephant_is_discounted_after_the_captured_doubling() {
    let c = citizen();
    let m = Modifiers {
        unassimilated: true,
        indian_elephant: T.indians_elephant_cost,
        ..Modifiers::default()
    };
    assert_eq!(
        cost_of(&T, &c, Resource::Food, owned(10), &m),
        85 * 60 / 100
    );
    let elephant = |p: i32| (100 - T.indians_elephant_cost) * p / 100;
    assert_eq!((elephant(96), elephant(56)), (81, 47));
}

/// The headline of the whole document: the same hoplite costs timber
/// before Metal is available and metal after — and the timber is five
/// quarters of the metal, because `UNDISC_COST_RATE` is `5/4`.
#[test]
fn metal_is_charged_as_timber_until_it_is_available() {
    let h = hoplite();
    let m = Modifiers::default();
    let all = [true; RESOURCES];
    let none = [false; RESOURCES];

    let after = charges(&T, &h, owned(0), &m, &all, &none, &Redirects::RON);
    assert_eq!(after[Resource::Food.index()], 50);
    assert_eq!(after[Resource::Metal.index()], 30);
    assert_eq!(after[Resource::Timber.index()], 0);

    let mut early = all;
    early[Resource::Metal.index()] = false;
    let before = charges(&T, &h, owned(0), &m, &early, &none, &Redirects::RON);
    assert_eq!(before[Resource::Food.index()], 50);
    assert_eq!(before[Resource::Metal.index()], 0);
    // (30 * 320) >> 8.
    assert_eq!(before[Resource::Timber.index()], 37);
}

/// A tech priced in knowledge before the Classical Age is charged three
/// halves of it in food. Mathematics is `8k/12g`: a hundred and twenty
/// food, not eighty.
#[test]
fn an_ancient_tech_pays_three_halves_of_its_knowledge_in_food() {
    let mathematics = Price {
        kind: Kind::Tech,
        ..Price::free()
            .with_base(Resource::Knowledge, 8)
            .with_base(Resource::Wealth, 12)
    };
    let mut available = [true; RESOURCES];
    available[Resource::Knowledge.index()] = false;
    let out = charges(
        &T,
        &mathematics,
        owned(0),
        &Modifiers::default(),
        &available,
        &[false; RESOURCES],
        &Redirects::RON,
    );
    assert_eq!(out[Resource::Food.index()], 120);
    assert_eq!(out[Resource::Wealth.index()], 120);
    assert_eq!(out[Resource::Knowledge.index()], 0);
}

/// **The Hanging Gardens' industry discount** (`get_cost:1270`–`1276`,
/// item 3332): `(100 − 66) × p / 100` on every good of an industry
/// tech's price — Metal Alloys' 200 timber was 68 on Great Sahara's
/// 15778 (run967e's `get_cost`). Here `25t/25f` at this table's factor
/// is 250 of each, and 85 with the Gardens.
#[test]
fn the_hanging_gardens_take_two_thirds_off_an_industry_tech() {
    let alloys = Price {
        kind: Kind::Tech,
        ..Price::free()
            .with_base(Resource::Food, 25)
            .with_base(Resource::Timber, 25)
    };
    let price = |m: &Modifiers| {
        charges(
            &T,
            &alloys,
            owned(0),
            m,
            &[true; RESOURCES],
            &[false; RESOURCES],
            &Redirects::RON,
        )
    };
    let gardens = Modifiers {
        industry: T.hanging_gardens_industry_discount,
        ..Modifiers::default()
    };
    assert_eq!(T.hanging_gardens_industry_discount, 66);
    assert_eq!(price(&Modifiers::default())[Resource::Timber.index()], 250);
    assert_eq!(price(&gardens)[Resource::Timber.index()], 85);
    assert_eq!(price(&gardens)[Resource::Food.index()], 85);
    // The German term is its own step after the Gardens' (item 3378):
    // 250 → 125 alone, and 85 → 42 behind them — never a combined 83 %.
    let german = Modifiers {
        german_industry: T.german_industry_discount,
        ..Modifiers::default()
    };
    let both = Modifiers {
        german_industry: T.german_industry_discount,
        ..gardens
    };
    assert_eq!(T.german_industry_discount, 50);
    assert_eq!(price(&german)[Resource::Timber.index()], 125);
    assert_eq!(price(&both)[Resource::Timber.index()], 42);
}

/// The redirect is a recursive call, so it chains. Oil names Metal and
/// Metal names Timber, and in the Ancient age neither is available: an oil
/// price arrives in timber, through metal, at both rates.
#[test]
fn a_redirect_chains_through_a_resource_that_is_itself_unavailable() {
    let tanker = Price {
        kind: Kind::Unit,
        ..Price::free().with_base(Resource::Oil, 3)
    };
    let mut available = [true; RESOURCES];
    available[Resource::Oil.index()] = false;
    available[Resource::Metal.index()] = false;
    let out = charges(
        &T,
        &tanker,
        owned(0),
        &Modifiers::default(),
        &available,
        &[false; RESOURCES],
        &Redirects::RON,
    );
    // (30 * 384) >> 8 = 45 in metal, and (45 * 320) >> 8 = 56 in timber.
    assert_eq!(out[Resource::Timber.index()], 56);
    assert_eq!(out[Resource::Metal.index()], 0);
    assert_eq!(out[Resource::Oil.index()], 0);

    // With Metal available the chain stops one link short.
    available[Resource::Metal.index()] = true;
    let out = charges(
        &T,
        &tanker,
        owned(0),
        &Modifiers::default(),
        &available,
        &[false; RESOURCES],
        &Redirects::RON,
    );
    assert_eq!(out[Resource::Metal.index()], 45);
    assert_eq!(out[Resource::Timber.index()], 0);
}

/// An obsolete resource redirects somewhere else than an undiscovered one,
/// and the choice is the prerequisite. Every `<OBS>` ships as `disable`, so
/// the obsolete table is inert in a stock game — including its one
/// interesting entry, Timber at half into Oil.
#[test]
fn obsolete_and_undiscovered_are_different_tables() {
    let r = Redirects::RON;
    assert_eq!(r.of(Resource::Metal, false).good, Resource::Timber);
    assert_eq!(r.of(Resource::Metal, false).rate, 320);
    assert_eq!(r.of(Resource::Metal, true).good, Resource::Wealth);
    assert_eq!(r.of(Resource::Metal, true).rate, 256);
    assert_eq!(r.of(Resource::Knowledge, false).good, Resource::Food);
    assert_eq!(r.of(Resource::Knowledge, false).rate, 384);
    assert_eq!(r.of(Resource::Timber, true).good, Resource::Oil);
    assert_eq!(r.of(Resource::Timber, true).rate, 128);
}

fn ledger_with(bucket: [i32; RESOURCES]) -> Ledger {
    Ledger {
        bucket,
        ..Ledger::default()
    }
}

#[test]
fn paying_takes_only_available_resources_and_floors_at_zero() {
    let mut l = ledger_with([100, 0, 0, 0, 100, 0]);
    let charges = [50, 0, 0, 0, 200, 0];
    let all = [true; RESOURCES];
    pay(&charges, &mut l, &all, false);
    assert_eq!(l.bucket[Resource::Food.index()], 50);
    assert_eq!(l.bucket[Resource::Metal.index()], 0);
}

#[test]
fn unpaying_puts_it_back() {
    let mut l = ledger_with([100, 0, 0, 0, 0, 0]);
    let charges = [30, 0, 0, 0, 0, 0];
    let all = [true; RESOURCES];
    pay(&charges, &mut l, &all, false);
    unpay(&charges, &mut l, &all);
    assert_eq!(l.bucket[Resource::Food.index()], 100);
}

#[test]
fn escrow_is_invisible_until_it_is_needed_and_then_abandoned() {
    let mut l = ledger_with([100, 0, 0, 0, 0, 0]);
    l.escrow[Resource::Food.index()] = 60;
    let all = [true; RESOURCES];

    // Forty is spendable; fifty is not.
    assert!(can_pay(&[40, 0, 0, 0, 0, 0], &l, &all, 1));
    assert!(!can_pay(&[50, 0, 0, 0, 0, 0], &l, &all, 1));

    // Paying fifty anyway abandons the whole reservation.
    pay(&[50, 0, 0, 0, 0, 0], &mut l, &all, false);
    assert_eq!(l.bucket[Resource::Food.index()], 50);
    assert_eq!(l.escrow[Resource::Food.index()], 0);
}

/// Recorded because it is the original's arithmetic and it is wrong: the
/// number affordable is the minimum over resources, and this is the
/// maximum. The early return is why nobody notices.
#[test]
fn the_affordability_count_takes_the_maximum() {
    let l = ledger_with([100, 0, 0, 0, 100, 0]);
    let all = [true; RESOURCES];
    // Ten food's worth, two metal's worth.
    let charges = [10, 0, 0, 0, 50, 0];
    assert_eq!(affordable(&charges, &l, &all, false), 10);
    // But one you cannot afford at all still answers zero.
    let charges = [10, 0, 0, 0, 200, 0];
    assert_eq!(affordable(&charges, &l, &all, false), 0);
    // And a price of nothing answers the sentinel.
    assert_eq!(affordable(&[0; RESOURCES], &l, &all, false), PLENTY);
}

/// The early return is `== 0`. A purse smaller than its own escrow divides
/// to a negative, which is neither zero nor larger than the running
/// maximum, so the loop ends having raised nothing and answers the
/// sentinel. The original's quirk, reproduced rather than corrected.
#[test]
fn a_purse_below_its_own_escrow_falls_through_to_the_sentinel() {
    let mut l = ledger_with([10, 0, 0, 0, 0, 0]);
    l.escrow[Resource::Food.index()] = 100;
    let all = [true; RESOURCES];
    assert_eq!(affordable(&[30, 0, 0, 0, 0, 0], &l, &all, false), PLENTY);
    // Against the escrow the purse is the whole bucket, and thirty of ten
    // is a plain zero.
    assert_eq!(affordable(&[30, 0, 0, 0, 0, 0], &l, &all, true), 0);
}

/// The index is the Military library level, so this is the table a player
/// climbs by researching The Art of War and the six techs above it —
/// twenty-five with none of them, two hundred with all seven.
#[test]
fn the_population_cap_is_the_military_table_clamped_by_the_lobby() {
    let b = PopBonuses::default();
    assert_eq!(pop_cap(&T, 0, 200, &b), 25);
    assert_eq!(pop_cap(&T, 1, 200, &b), 50);
    assert_eq!(pop_cap(&T, 7, 200, &b), 200);
    assert_eq!(pop_cap(&T, 7, 100, &b), 100);
    assert_eq!(pop_cap(&T, 3, 50, &b), 50);
}

/// The scenario editor's ignore-cap flag hands over the lobby limit and
/// skips the table, the cities and the clamp — but not the wonders.
#[test]
fn the_ignore_cap_flag_is_the_lobby_limit_outright() {
    let b = PopBonuses {
        ignore_cap: true,
        ..PopBonuses::default()
    };
    assert_eq!(pop_cap(&T, 0, 200, &b), 200);
    let with_colossus = PopBonuses {
        colossus: true,
        ..b
    };
    assert_eq!(pop_cap(&T, 0, 200, &with_colossus), 250);
}

#[test]
fn cities_raise_nothing_because_village_pop_ships_as_zero() {
    let b = PopBonuses {
        cities: vec![CityLevel::Metropolis; 4],
        ..PopBonuses::default()
    };
    assert_eq!(pop_cap(&T, 2, 200, &b), 75);
}

#[test]
fn the_colossus_and_the_bantu_move_the_cap() {
    let colossus = PopBonuses {
        colossus: true,
        ..PopBonuses::default()
    };
    assert_eq!(pop_cap(&T, 7, 200, &colossus), 250);

    // The Bantu double the cap and raise the ceiling a quarter, so the
    // clamp lands on the raised ceiling rather than the doubled cap.
    let bantu = PopBonuses {
        bantu: true,
        ..PopBonuses::default()
    };
    assert_eq!(pop_cap(&T, 7, 200, &bantu), 250);
    assert_eq!(pop_cap(&T, 0, 200, &bantu), 50);
}

#[test]
fn a_scenario_cap_overrides_the_age_table_but_not_the_wonders() {
    let b = PopBonuses {
        scenario: Some(17),
        colossus: true,
        ..PopBonuses::default()
    };
    assert_eq!(pop_cap(&T, 0, 200, &b), 67);
}

#[test]
fn landing_exactly_on_the_cap_is_allowed() {
    assert!(!exceeds_population(25, 24, 1));
    assert!(exceeds_population(25, 25, 1));
    assert!(!exceeds_population(25, 25, 0));
}
#[test]
fn science_prices_an_epoch_flat_and_surcharges_a_plain_tech() {
    // run40's own number: City State is `12f`, so `120` after
    // `TECH_COST_FACTOR`, and the AI pays exactly that. It is an *epoch*,
    // so its level is its `AGE` of zero with no plus-one, and a player at
    // Science 0 is neither ahead nor behind.
    let t = Tuning::RON;
    assert_eq!(science_discount(&t, 0, 120), 120);
    // A level of Science ahead takes ten percent off, two levels twenty —
    // off the base each time, not compounding.
    assert_eq!(science_discount(&t, 1, 120), 108);
    assert_eq!(science_discount(&t, 2, 120), 96);
    // Behind, the same expression charges more. A *plain* tech is where
    // this bites in a shipped game: its level is `AGE + 1`, so an Ancient
    // one is already a level ahead of a player who has researched no
    // Science at all, and costs 110% until they do.
    assert_eq!(science_discount(&t, -1, 120), 132);
    assert_eq!(science_discount(&t, -3, 120), 156);
    // The truncation is the original's: the term is truncated toward zero
    // and then taken away, on both signs.
    assert_eq!(science_discount(&t, 1, 5), 5, "10% of 5 truncates to 0");
    assert_eq!(science_discount(&t, -1, 5), 5);
}

#[test]
fn the_refund_inverts_the_discount_exactly_for_an_epoch() {
    // [`reprice`] reads the `AGE` column raw where [`science_discount`]
    // adds one for a plain tech, so the pair is an exact inverse for an
    // age or an epoch and one level out for anything else. That asymmetry
    // is the original's, and it is the reason a plain tech queued at one
    // Science level and refunded at the next comes back slightly dearer
    // than it went in.
    let t = Tuning::RON;
    for base in [120, 250, 1_000] {
        for level in 0..4 {
            // An epoch: `science_discount`'s `ahead` and `reprice`'s
            // `level - age` are the same number, so the refund of a price
            // struck at `level` is the price struck at `level + 1`.
            let paid = science_discount(&t, level, base);
            assert_eq!(
                reprice(&t, paid, level, 0),
                science_discount(&t, level + 1, base),
                "base {base} at level {level}"
            );
        }
    }
}

#[test]
fn a_queued_tech_is_repriced_by_reconstruction_not_by_memory() {
    // run40's own numbers: the AI pays 120 food for City State at
    // Science 0, and Written Word lands while it is still in the queue.
    let t = Tuning::RON;
    assert_eq!(reprice(&t, 120, 0, 0), 108, "120 - 10% of 120");
    // And again at the next level, from the *new* stored amount: 108 is
    // divided by the 90% that was in force to recover the 120 base, and
    // re-struck at 80%. That is what makes the chain lossless where a
    // remembered number would compound its truncation.
    assert_eq!(reprice(&t, 108, 1, 0), 96);
    assert_eq!(reprice(&t, 96, 2, 0), 84);
    // A tech whose own age is above the player's Science level costs
    // *more*, from the same expression — the surcharge `docs/COSTS.md`
    // names — and the reconstruction inverts that too.
    assert_eq!(
        reprice(&t, 120, 0, 2),
        110,
        "120 was 100 at a 20% surcharge; three levels behind it is 110"
    );
}
