#[test]
fn the_barracks_garrison_term_starts_at_zero() {
    // `remove_from_inside` counts it (item 3372); no reader yet.
    assert_eq!(Census::default().barracks_garr, 0);
}

#[test]
fn a_ushort_region_slot_reads_unsigned() {
    // `reg_free_peasants` is a `ushort[64]` and `Leader::produce_building`
    // subtracts one at a site nothing counted: the dump prints 65534
    // where this crate holds −2, and `create_units`' `reg_free >=
    // reg_cities` is the unsigned compare (`docs/AI.md` §158).
    let v = [0, -2, 3];
    assert_eq!(Census::reg(&v, 1), -2);
    assert_eq!(Census::reg_u16(&v, 1), 65534);
    assert_eq!(Census::reg_u16(&v, 2), 3);
    assert_eq!(Census::reg_u16(&v, 9), 0);
}

use super::*;

/// **Two offers of one type at one value are not one offer**, and the
/// tie splits the ranked copy from the category copy — item 432.
///
/// `make_me` ranks on `list[k].val <= val` and fills the category slot
/// on `list[cat].val < val` (`006c9be0`, both senses read off the
/// decompile). The asymmetry is invisible while every offer has its own
/// number and decides a purchase the moment two share one: the later
/// offer **displaces** the earlier in the ranked list and **does not**
/// in the category slot, so slot 1 and slot `cat` end up naming
/// different cities for the same type.
///
/// `make_stuff`'s step 6 then walks slot `cat` with the duplicate test
/// `t == list[k].t && city == list[k].city` over slots 0..4
/// (`006c8af0:147–167`), finds no match, and **buys the same type a
/// second time** — one extra `make_this`, one extra `expire` call, one
/// extra `Leader::make_stuff+0x63d` draw.
///
/// That is what happens on Great Lakes 9382 the moment this crate's
/// Scholar is given a value it can win with: `create_units` offers the
/// Scholar once per city and this crate computes the **same** number for
/// both, where the original computes two (`docs/AI.md` §50). The second
/// half of the test is the original's shape — distinct values — and
/// there the category slot follows the better offer and the duplicate
/// test fires.
///
/// Made to fail on purpose by weakening the ranked insert to `<`:
/// the tie then does not displace, both copies name city 1, and the
/// first `assert!` below stops it. The test is about the two
/// comparison senses, not about the numbers.
#[test]
fn a_tie_splits_the_ranked_copy_from_the_category_copy() {
    /// `make_stuff` step 6's duplicate test, verbatim: does slot `at`
    /// repeat a `(t, city)` already among the ranked four?
    fn deduped(l: &MakeList, at: usize) -> bool {
        let m = l.list[at];
        (0..4).any(|k| l.list[k].t == m.t && l.list[k].city == m.city)
    }
    // A head far above both offers, so neither can take slot 0 — the
    // Great Lakes shape, where `t573` stands at 9,999,999.
    let head = |l: &mut MakeList| l.make_me(573, 9_999_999, 1, 10, -1, 0, 1, 0, 0);

    // One value for both cities: this crate's own 45,568 twice.
    let mut tied = MakeList::new();
    head(&mut tied);
    tied.make_me(52, 45_568, 1, 4, 1, 0, 1, 0, 0);
    tied.make_me(52, 45_568, 1, 4, 2, 0, 1, 0, 0);
    assert_eq!((tied.list[1].t, tied.list[1].city), (52, 2));
    assert_eq!((tied.list[4].t, tied.list[4].city), (52, 1));
    // The earlier ranked copy is cleared to `t −1` and keeps its
    // `val` and `city`, which is how a dump tells the two clears apart.
    assert_eq!(
        (tied.list[2].t, tied.list[2].val, tied.list[2].city),
        (-1, 45_568, 1)
    );
    assert!(
        !deduped(&tied, 4),
        "the tie must defeat step 6's duplicate test — that is the              second purchase, and the extra expiry draw with it"
    );

    // The original's shape on the same frame: one value per city,
    // 4,891,136 for its first and 5,755,741 for its second, in that
    // order (run111 block 9381 is what says both the values and the
    // order — `rondata::diff`'s reconstruction of it).
    let mut apart = MakeList::new();
    head(&mut apart);
    apart.make_me(52, 4_891_136, 1, 4, 1, 0, 1, 0, 0);
    apart.make_me(52, 5_755_741, 1, 4, 2, 0, 1, 0, 0);
    assert_eq!((apart.list[1].t, apart.list[1].city), (52, 2));
    assert_eq!((apart.list[4].t, apart.list[4].city), (52, 2));
    assert_eq!(
        (apart.list[2].t, apart.list[2].val, apart.list[2].city),
        (-1, 4_891_136, 1)
    );
    assert!(
        deduped(&apart, 4),
        "with the offers apart the category slot follows the better one              and step 6 skips it: one purchase, two expiry draws"
    );
}

fn draws(seed: u32, f: impl FnOnce(&mut Rng)) -> usize {
    let mut a = Rng::new(seed);
    let mut n = 0;
    let mut probe = a;
    f(&mut a);
    while probe != a {
        probe.roll();
        n += 1;
        assert!(n < 100, "diverged");
    }
    n
}

#[test]
fn a_nubian_alone_takes_twenty_or_twenty_one_draws() {
    // Twenty fixed draws — rush, cities, the arms coin and its roll,
    // army, army_size, raid, invade, target, strategy, spells, forts,
    // nukes, air, naval, market, scouts, civilians, friendly, alliance —
    // and a rush roll of 1 costs a Nubian one more.
    let setup = RollSetup {
        sea_map: 1,
        rush_rules_off: false,
    };
    let mut seen = [false; 2];
    for seed in 0..40u32 {
        let n = draws(seed, |r| {
            Personality::roll(r, tribe::NUBIANS, &[], &setup);
        });
        assert!(n == 20 || n == 21, "seed {seed}: {n} draws");
        seen[n - 20] = true;
    }
    assert_eq!(seen, [true, true]);
}

#[test]
fn the_romans_always_want_forts_and_the_germans_air() {
    let setup = RollSetup {
        sea_map: 1,
        rush_rules_off: false,
    };
    for seed in 0..20u32 {
        let p = Personality::roll(&mut Rng::new(seed), tribe::ROMANS, &[], &setup);
        assert_eq!(p.forts, 1);
        let p = Personality::roll(&mut Rng::new(seed), tribe::GERMANS, &[], &setup);
        assert_eq!(p.air, 1);
        assert_eq!(p.friendly_ai, p.friendly_human);
        assert_eq!(p.alliance_ai, p.alliance_human);
        assert_eq!(p.raze, 0);
    }
}

#[test]
fn rush_rules_off_pins_raid() {
    let setup = RollSetup {
        sea_map: 1,
        rush_rules_off: true,
    };
    let p = Personality::roll(&mut Rng::new(3), tribe::AZTECS, &[], &setup);
    assert_eq!(p.raid, -1);
}

#[test]
fn make_me_head_overwrites_and_does_not_shift() {
    let mut m = MakeList::new();
    m.make_me(10, 100, 1, 9, -1, 0, 1, 0, 0);
    m.make_me(11, 200, 1, 8, -1, 0, 1, 0, 0);
    assert_eq!(m.list[0].t, 11);
    assert_eq!(m.list[1].t, -1, "the old head is gone, not demoted");
    assert_eq!(m.list[9].t, 10, "the category slot keeps it");
    assert_eq!(m.list[8].t, 11);
}

#[test]
fn make_me_ranks_runners_up_and_deduplicates() {
    let mut m = MakeList::new();
    m.make_me(1, 500, 1, 4, -1, 0, 1, 0, 0);
    m.make_me(2, 300, 1, 5, -1, 0, 1, 0, 0);
    m.make_me(3, 400, 1, 6, -1, 0, 1, 0, 0);
    m.make_me(4, 100, 1, 7, -1, 0, 1, 0, 0);
    assert_eq!(
        [m.list[0].t, m.list[1].t, m.list[2].t, m.list[3].t],
        [1, 3, 2, 4]
    );
    // A better offer of type 2 displaces its own lower entry.
    m.make_me(2, 450, 1, 5, -1, 0, 1, 0, 0);
    assert_eq!(
        [m.list[0].t, m.list[1].t, m.list[2].t, m.list[3].t],
        [1, 2, 3, -1]
    );
    assert_eq!(m.list[5].val, 450);
    // A worse offer of a type already ranked is dropped.
    m.make_me(3, 350, 1, 6, -1, 0, 1, 0, 0);
    assert_eq!(
        [m.list[0].t, m.list[1].t, m.list[2].t, m.list[3].t],
        [1, 2, 3, -1]
    );
}

#[test]
fn the_sweep_is_frame_zero_then_every_two_hundred_on_the_phase() {
    let due: Vec<i64> = (0..800).filter(|&f| cadence::sweep_due(1, f, 1)).collect();
    assert_eq!(due, vec![0, 175, 375, 575, 775]);
    let due: Vec<i64> = (0..300).filter(|&f| cadence::sweep_due(0, f, 1)).collect();
    assert_eq!(due, vec![0, 200]);
    assert!(cadence::research_tick_due(1, 205, 1));
    assert!(!cadence::research_tick_due(1, 206, 1));
    assert!(cadence::explore_due(1, 163, 1));
}

#[test]
fn the_step_machine_follows_the_script_s_answer() {
    let mut l = Leader::new();
    l.script_live = true;
    l.step = Step::Script;
    l.enter(false);
    assert_eq!(l.step, Step::Script);
    assert!(l.after_script(ScriptResult::Returned(1)));
    assert_eq!(l.step, Step::Idle);
    assert!(l.script_live);

    l.step = Step::Script;
    assert!(!l.after_script(ScriptResult::Returned(3)));
    assert_eq!(l.step, Step::Setup);
    assert!(!l.script_live, "SCRIPT_DONE drops the script");

    l.step = Step::Script;
    l.enter(false);
    assert_eq!(l.step, Step::Setup, "a dead script is skipped");

    l.step = Step::Script;
    l.script_live = true;
    assert!(!l.after_script(ScriptResult::Failed));
    assert!(!l.script_live);
    assert_eq!(l.step, Step::Setup);

    l.step = Step::Make;
    l.after_make(false, false);
    assert_eq!(l.step, Step::Idle);
    l.step = Step::Make;
    l.after_make(true, false);
    assert_eq!(l.step, Step::Units2);
    l.after_producer();
    assert_eq!(l.step, Step::Buildings2);
    l.after_second_pass();
    assert_eq!(l.step, Step::Make2);
    l.after_second_pass();
    assert_eq!(l.step, Step::Idle);
}

#[test]
fn the_goods_picture_is_about_the_rate() {
    let mut l = Leader::new();
    let mut ledger = Ledger {
        income: [40 * 16, 10 * 16, 0, 0, 0, 0],
        bucket: [5000, 100, 0, 0, 0, 0],
        ..Ledger::default()
    };
    let s = GoodsSetup {
        city_num: 1,
        difficulty: 3,
        resources_unlimited: false,
        lakota: false,
        next_age_cost: None,
        available: [true, true, false, false, false, false],
        cap: [1000 * 16; RESOURCES],
    };
    goods_picture(&mut l, &mut ledger, &s);
    assert_eq!(l.rate[0], 40);
    assert_eq!(l.rate[1], 10);
    assert_eq!(l.shortages, 1, "wood under 30");
    assert_eq!(l.worst_good, 1);
    assert_eq!(l.best_good, 0);
    // lo = min(15, 500, 250) = 15, hi = min(30, 800, 350) = 30: food is
    // at 40 → comfortable; wood at 10 → under both.
    assert_eq!(l.econ[0], 8);
    assert_eq!(l.econ[1], 1 | 2 | 4);
    assert_eq!(l.econ[2], 0, "unavailable");
    assert_eq!(ledger.bucket[0], 5000, "no clamp at difficulty 3");
}

#[test]
fn an_easy_ai_throws_its_stockpile_away() {
    let mut l = Leader::new();
    let mut ledger = Ledger {
        bucket: [5000, 5000, 0, 0, 0, 0],
        escrow: [1000, 0, 0, 0, 0, 0],
        ..Ledger::default()
    };
    let s = GoodsSetup {
        city_num: 1,
        difficulty: 0,
        resources_unlimited: false,
        lakota: false,
        next_age_cost: Some([200, 100, 0, 0, 0, 0]),
        available: [true; RESOURCES],
        cap: [1000 * 16; RESOURCES],
    };
    goods_picture(&mut l, &mut ledger, &s);
    // m = max(200, 300) × 3/2 = 450.
    assert_eq!(ledger.bucket[0], 450);
    assert_eq!(ledger.escrow[0], 1000 * 450 / 5000);
    assert_eq!(ledger.bucket[1], 450);
}
