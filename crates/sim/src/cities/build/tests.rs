use super::*;

fn t() -> Tuning {
    Tuning::RON
}

/// The template, and the fallback for a type nothing loaded one for.
#[test]
fn the_blocking_template_is_read_row_major_by_y() {
    // `7x7 extra space`, cut down: a 3×3 footprint blocking 2×2.
    let mut b = BuildType {
        x_size: 3,
        y_size: 3,
        block_mask: vec![1, 1, 0, 1, 1, 0, 0, 0, 0],
        ..BuildType::default()
    };
    assert!(b.blocks(0, 0) && b.blocks(1, 1));
    assert!(!b.blocks(2, 0), "the last column");
    assert!(!b.blocks(0, 2), "the last row");
    assert!(!b.blocks(3, 0), "off the template");
    // No template: the old flat/non-flat line, which every hand-built
    // type in this crate still takes.
    b.block_mask.clear();
    assert!(b.blocks(2, 2));
    b.flags |= flags::FLAT;
    assert!(!b.blocks(0, 0));
}

fn types() -> Vec<BuildType> {
    let mk = |ident, from, job, hits, wonder| BuildType {
        ident,
        from,
        job_time: job,
        hits,
        wonder,
        x_size: 4,
        y_size: 4,
        ..BuildType::default()
    };
    vec![
        mk(Ident::Village, None, 600, 1200, false),
        mk(Ident::Town, Some(0), 600, 2500, false),
        mk(Ident::Metropolis, Some(1), 600, 5000, false),
        mk(Ident::Fort, None, 2000, 2000, false),
        mk(Ident::Other, Some(3), 2000, 2800, false), // Castle
        mk(Ident::RedFort, Some(3), 3000, 6000, true),
        mk(Ident::Tower, None, 1000, 750, false),
        mk(Ident::Other, Some(6), 1000, 1000, false), // Keep
        mk(Ident::Wonder, None, 4000, 2000, true),    // Pyramids
        mk(Ident::Senate, None, 500, 1200, false),
        mk(Ident::Market, None, 420, 1200, false),
        mk(Ident::ForbiddenCity, Some(0), 5000, 3000, true),
    ]
}

#[test]
fn the_derived_flags_are_marked_where_the_loader_marks_them() {
    let mut ty = types();
    // The Fort is a military trainer and a research building; a tech is
    // researched at the Tower line's root; the Village trains and holds a
    // craft.
    let d = Derived {
        trains: vec![0, 3],
        military_trains: vec![3],
        research_at: vec![6, 9],
        craft_at: vec![0],
    };
    init_derived_flags(&mut ty, &d);
    assert!(ty[0].has(flags::TRAINS) && ty[0].has(flags::CRAFT_HERE));
    assert!(!ty[0].has(flags::MILITARY_TRAINER));
    assert!(ty[3].has(flags::MILITARY_TRAINER) && ty[3].has(flags::TRAINS));
    // `RESEARCH_HERE` walks the lineage: the Keep gets it from the Tower.
    assert!(ty[6].has(flags::RESEARCH_HERE) && ty[7].has(flags::RESEARCH_HERE));
    assert!(!ty[8].has(flags::RESEARCH_HERE));
    // The upgrade mark lands on both ends of every `FROM`.
    assert!(ty[0].has(flags::UPGRADE_LINE) && ty[1].has(flags::UPGRADE_LINE));
    assert!(!ty[9].has(flags::UPGRADE_LINE));
    // And the three predicates read the root, not the type.
    assert!(is_military_trainer(&ty, 4), "the Castle's root is the Fort");
    assert!(
        is_training_building(&ty, 2),
        "the Major City's is the Village"
    );
    assert_eq!(queue_capacity(&ty, 2), 20);
    assert_eq!(queue_capacity(&ty, 7), 10, "a research building holds ten");
    assert_eq!(queue_capacity(&ty, 8), 2, "everything else holds two");
}

#[test]
fn flags_parse_letters_and_digits_the_loaders_way() {
    assert_eq!(
        flags::parse("ean"),
        flags::NO_CITY | 0x1 | flags::NOT_CIVILIAN
    );
    assert_eq!(flags::parse("jam"), flags::ONE_PER_CITY | 0x1 | 0x1000);
    assert_eq!(flags::parse("gda"), flags::GATHERER | 0x8 | 0x1);
    assert_eq!(
        flags::parse("ebn") & (flags::WATER | flags::NO_CITY),
        flags::WATER | flags::NO_CITY
    );
    assert_eq!(flags::parse("3"), flags::FLAT);
    assert_eq!(flags::parse("2"), flags::RESEARCH_HERE);
}

#[test]
fn the_domain_is_derived_from_two_flags() {
    let mut b = BuildType::default();
    assert_eq!(b.domain(), BuildDomain::Land);
    // A dock is `ebn`: water. No shipped building carries `a` with `b`.
    b.flags = flags::parse("ebn");
    assert_eq!(b.domain(), BuildDomain::Water);
    b.flags = flags::parse("igbe");
    assert_eq!(b.domain(), BuildDomain::Water);
    b.flags = flags::parse("ab");
    assert_eq!(b.domain(), BuildDomain::Both);
}

#[test]
fn lineage_walks_from_and_levels_follow_the_ident() {
    let ty = types();
    assert!(is_city(&ty, 0) && is_city(&ty, 1) && is_city(&ty, 2) && is_city(&ty, 11));
    assert!(!is_city(&ty, 3));
    assert!(is_fort(&ty, 4) && is_fort(&ty, 5) && !is_fort(&ty, 6));
    assert!(is_tower(&ty, 7) && !is_tower(&ty, 3));
    assert_eq!(city_level(&ty, 0), 1);
    assert_eq!(city_level(&ty, 1), 2);
    assert_eq!(city_level(&ty, 2), 3);
    assert_eq!(city_level(&ty, 11), 3);
    assert_eq!(city_level(&ty, 3), 0);
    assert_eq!(upgrades_to(&ty, 0), Some(1));
    assert_eq!(upgrades_to(&ty, 1), Some(2));
    assert_eq!(upgrades_to(&ty, 2), None);
    assert_eq!(upgrades_to(&ty, 11), None);
}

#[test]
fn an_unmodified_type_takes_job_time_frames_at_one_accel() {
    let ty = types();
    let base = construct_base(&t(), &ty, 0, &BuildMods::default());
    assert_eq!(base, 60_000);
    let ct = construct_time(&t(), &ty[0], base, &ClockMods::default());
    assert_eq!(ct, 60_000);
    // One builder a frame: exactly job_time frames.
    let mut jc = 0;
    let mut frames = 0;
    while jc < ct {
        jc += contribution(builder_amount(&t(), false, false), 0);
        frames += 1;
    }
    assert_eq!(frames, 600);
}

#[test]
fn the_first_city_of_a_nomad_takes_three_times_as_long() {
    let ty = types();
    let m = BuildMods {
        no_city: true,
        ..BuildMods::default()
    };
    assert_eq!(construct_base(&t(), &ty, 0, &m), 180_000);
}

#[test]
fn speed_techs_and_nations_stack_in_the_loaders_order() {
    let ty = types();
    let m = BuildMods {
        maya: true,
        created_faster: true,
        speed_upgrade: 3,
        ..BuildMods::default()
    };
    // 60000 × 100/120 = 50000; ×3>>2 = 37500; ×7/10 = 26250.
    assert_eq!(construct_base(&t(), &ty, 0, &m), 26_250);
    // A Roman fort: 200000 × 100/150 = 133333.
    let m = BuildMods {
        romans: true,
        ..BuildMods::default()
    };
    assert_eq!(construct_base(&t(), &ty, 3, &m), 133_333);
    // And the castle, through the lineage.
    assert_eq!(construct_base(&t(), &ty, 4, &m), 133_333);
    // But not the Red Fort, a wonder.
    assert_eq!(construct_base(&t(), &ty, 5, &m), 300_000);
}

#[test]
fn the_per_call_clauses() {
    let ty = types();
    let base = 60_000;
    assert_eq!(
        construct_time(
            &t(),
            &ty[8],
            base,
            &ClockMods {
                free_first_wonder: true,
                ..Default::default()
            }
        ),
        1
    );
    // The Gardens ship at 0 %: no change.
    assert_eq!(
        construct_time(
            &t(),
            &ty[0],
            base,
            &ClockMods {
                hanging_gardens: true,
                ..Default::default()
            }
        ),
        60_000
    );
    assert_eq!(
        construct_time(
            &t(),
            &ty[0],
            base,
            &ClockMods {
                president: true,
                ..Default::default()
            }
        ),
        45_112
    );
    // The Iroquois first senate: 0, floored to 1 — done on the first contribution.
    assert_eq!(
        construct_time(
            &t(),
            &ty[9],
            base,
            &ClockMods {
                iroquois_first_senate: true,
                ..Default::default()
            }
        ),
        1
    );
    // The same flag on a market does nothing.
    assert_eq!(
        construct_time(
            &t(),
            &ty[10],
            base,
            &ClockMods {
                iroquois_first_senate: true,
                ..Default::default()
            }
        ),
        60_000
    );
}

#[test]
fn builders_are_harmonic_and_the_under_attack_quarter_is_sign_fixed() {
    assert_eq!(contribution(100, 0), 100);
    assert_eq!(contribution(100, 1), 50);
    assert_eq!(contribution(100, 2), 33);
    assert_eq!(contribution(100, 3), 25);
    assert_eq!(contribution(3, 5), 1);
    assert_eq!(builder_amount(&t(), true, false), 25);
    assert_eq!(builder_amount(&t(), true, true), 100);
    assert_eq!(builder_amount(&t(), false, false), 100);
}

#[test]
fn a_site_grows_linearly_and_a_wonder_from_half() {
    assert_eq!(site_hits(1200, 0, 60_000, false), 1);
    // 937 × 1200 / 1875: the `>> 5` truncation shows.
    assert_eq!(site_hits(1200, 30_000, 60_000, false), 599);
    assert_eq!(site_hits(1200, 60_000, 60_000, false), 1200);
    assert_eq!(site_hits(2000, 0, 400_000, true), 1001);
    assert_eq!(site_hits(2000, 200_000, 400_000, true), 1500);
    assert_eq!(site_hits(2000, 400_000, 400_000, true), 2000);
    // A tiny clock still works through the >> 5 guard.
    assert_eq!(site_hits(100, 0, 1, false), 100);
}

#[test]
fn the_senate_bonus_and_the_temple_bonus() {
    let ty = types();
    let m = HitsMods::default();
    // A market in a Large City: 1200 + 35 % = 1620; in a Major City 1200 + 70 % = 2040.
    assert_eq!(full_hits(&t(), &ty, 10, true, 2, false, &m), 1620);
    assert_eq!(full_hits(&t(), &ty, 10, true, 3, false, &m), 2040);
    // Not before it is active, and not outside a city.
    assert_eq!(full_hits(&t(), &ty, 10, false, 2, false, &m), 1200);
    assert_eq!(full_hits(&t(), &ty, 10, true, 0, false, &m), 1200);
    // A tower in a Large City: no senate bonus.
    assert_eq!(full_hits(&t(), &ty, 6, true, 2, false, &m), 750);
    // Nor a Keep: `is(0x1b7, 0)` reads the line, and run488's Keep
    // `1/2015` in a Large City holds 1000 (item 1275).
    assert_eq!(full_hits(&t(), &ty, 7, true, 2, false, &m), 1000);
    // A city with a level-one temple: +25 %; with Tikal, (150 × 25 + 99) / 100 = 38 → +38 %.
    let m = HitsMods {
        temple_level: 1,
        ..Default::default()
    };
    assert_eq!(full_hits(&t(), &ty, 0, true, 1, true, &m), 1500);
    let m = HitsMods {
        temple_level: 1,
        tikal: true,
        ..Default::default()
    };
    assert_eq!(full_hits(&t(), &ty, 0, true, 1, true, &m), 1656);
    // Three HP techs: +30 %.
    let m = HitsMods {
        hp_upgrade: 3,
        ..Default::default()
    };
    assert_eq!(full_hits(&t(), &ty, 0, false, 0, false, &m), 1560);
}

#[test]
fn the_refund_is_what_was_not_built() {
    assert_eq!(refund(120, 60_000, 0), 120);
    assert_eq!(refund(120, 60_000, 30_000), 60);
    assert_eq!(refund(120, 60_000, 60_000), 0);
    assert_eq!(refund(120, 60_000, 70_000), 0);
    assert_eq!(refund(100, 3, 1), 66);
}

#[test]
fn a_full_repair_takes_twice_the_build_time() {
    let r = RepairTarget {
        helpers: 0,
        construct_time: 60_000,
        hits: 1200,
        accel: 100,
        ..Default::default()
    };
    // (60000 << 9) / (1200 x 100) = 256: one hit point a frame, 1200 frames for 1200 hits
    // against 600 frames to build.
    let p = repair_period(&r);
    assert_eq!(p, 256);
    assert_eq!(repair_amount(7, i64::from(p), 1200), 1);
    // A city doubles it, a captured city quadruples it.
    let c = RepairTarget { city: true, ..r };
    assert_eq!(repair_period(&c), 512);
    let c = RepairTarget {
        city: true,
        captured: true,
        ..r
    };
    assert_eq!(repair_period(&c), 1024);
    // Under attack, freshly hit: x 16.
    let u = RepairTarget {
        under_attack: true,
        fresh_hit: true,
        ..r
    };
    assert_eq!(repair_period(&u), 4096);
    // Koreans under fire shrug it off and repair in half the time.
    let k = RepairTarget {
        under_attack: false,
        fresh_hit: false,
        korean_repair: 50,
        ..u
    };
    assert_eq!(repair_period(&k), 128);
    // A second repairer's own period is longer: harmonic, like builders.
    let two = RepairTarget { helpers: 1, ..r };
    assert_eq!(repair_period(&two), 512);
    // A period below one repairs everything now.
    assert_eq!(repair_amount(3, 0, 77), 77);
}

#[test]
fn repair_amount_is_one_point_every_period_frames() {
    let p = 4 * 256;
    let total: i32 = (1..=40).map(|f| repair_amount(f, p, 1000)).sum();
    assert_eq!(total, 10);
}
