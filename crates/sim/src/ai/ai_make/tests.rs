use super::*;

/// Loop 2955 (the thirty-ninth pass): the `can_pay` probe every AI item
/// re-added by hand is an env-guarded print now; the spec names one
/// frame, a window or every frame, and the line carries every input.
#[test]
fn the_make_debug_print_is_named_by_frame_and_carries_the_can_pay_inputs() {
    assert!(debug_make_wanted("all", 7));
    assert!(debug_make_wanted("576", 576));
    assert!(!debug_make_wanted("576", 577));
    assert!(debug_make_wanted("570-580", 576));
    assert!(!debug_make_wanted("570-575", 576));
    assert!(!debug_make_wanted("", 576));
    assert!(!debug_make_wanted("x-y", 576));
    let head = MakeObject {
        t: 340,
        val: 9,
        escrow: 1,
        city: 2,
        up: 0,
        o: -1,
        num: 1,
        cat: 0,
        wx: 0,
        wy: 0,
    };
    let line = make_debug_line(
        576,
        1,
        &head,
        &[0, 80, 0, 0, 0, 0],
        &[12, 78, 0, 0, 0, 0],
        0,
        false,
    );
    assert_eq!(
        line,
        "MAKE f576 who=1 head t=340 num=1 escrow=1 cost=[0, 80, 0, 0, 0, 0] bucket=[12, 78, 0, 0, 0, 0] affordable=0 paid=no"
    );
}
use crate::ai::CityAi;
use crate::build::BuildType;
use crate::economy::Resource;
use crate::tech::{TypeDef, UnitTraits};
use crate::world::{Cell, Terrain, UNITS_PER_TILE};
use crate::{Pos, Tuning, UnitType, World, cost};

fn tile_pos(tx: i32, ty: i32) -> Pos {
    Pos::new(
        tx * UNITS_PER_TILE + UNITS_PER_TILE / 2,
        ty * UNITS_PER_TILE + UNITS_PER_TILE / 2,
    )
}

const COMBAT: UnitTraits = UnitTraits {
    free: false,
    jumpable: false,
    unique: false,
    hero: false,
    patriot: false,
    combat: false,
};

/// Everything the make list can hold, wired tree entry to record.
struct Fx {
    sim: Sim,
    village: TypeId,
    tower: TypeId,
    wonder: TypeId,
    farm: TypeId,
    barracks: TypeId,
    soldier: TypeId,
    citizen: TypeId,
    tech: TypeId,
    town: TypeId,
    farm_rec: usize,
    village_rec: usize,
}

fn bt(ident: Ident, tree: TypeId, x: i32, y: i32, food: i32) -> BuildType {
    BuildType {
        ident,
        x_size: x,
        y_size: y,
        job_time: 100,
        hits: 100,
        tree: Some(tree),
        price: cost::Price::free().with_base(Resource::Food, food),
        ..BuildType::default()
    }
}

fn fx() -> Fx {
    let mut w = World::new(16, 16);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
    let mut sim = Sim::new(Tuning::RON, w, 2);
    sim.cities_state.nation[1].human = false;
    let t = &mut sim.tables.tech_tree;
    for n in ["Food", "Timber", "Wealth", "Knowledge", "Metal", "Oil"] {
        t.add(TypeDef::good(n));
    }
    let village = t.add(TypeDef::building("Small City"));
    let town = t.add(TypeDef::building("Large City").from(village));
    let tower = t.add(TypeDef::building("Tower"));
    let wonder = t.add(TypeDef::new(
        "Colossus",
        Kind::Building {
            auto: false,
            wonder: true,
        },
    ));
    let farm = t.add(TypeDef::building("Farm"));
    let barracks = t.add(TypeDef::building("Barracks"));
    let university = t.add(TypeDef::building("University"));
    let soldier = t.add(TypeDef::unit("Hoplite", COMBAT));
    let citizen = t.add(TypeDef::unit("Citizen", COMBAT));
    let tech = t.add(TypeDef::plain("Fortification", -1));
    t.roles.tower = Some(tower);
    t.roles.university = Some(university);
    t.roles.town = Some(town);
    sim.tech = (0..2)
        .map(|_| crate::tech::PlayerTech::new(&sim.tables.tech_tree))
        .collect();

    let village_rec = sim.add_build_type(bt(Ident::Village, village, 7, 7, 0));
    let mut town_bt = bt(Ident::Town, town, 7, 7, 0);
    town_bt.from = Some(village_rec);
    sim.add_build_type(town_bt);
    sim.add_build_type(bt(Ident::Tower, tower, 2, 2, 30));
    sim.add_build_type(bt(Ident::Wonder, wonder, 5, 5, 0));
    let mut f = bt(Ident::Farm, farm, 4, 4, 60);
    // A farm is a gather type *and* a flat one — `init_final_flags`
    // makes the FARM, OILWELL and OILPLATFORM lineages flat, and the
    // flat half is what keeps `blocked_location`'s gather tail off it
    // (`docs/CITIES.md` §2.6.7). Without it a farm on this bare test
    // world is refused for having nothing to gather.
    f.flags |= flags::GATHERER | flags::FLAT;
    let farm_rec = sim.add_build_type(f);
    // The barracks costs two goods, so that `need`'s mean has something
    // to average.
    let mut b = bt(Ident::Barracks, barracks, 4, 4, 100);
    b.price = b.price.with_base(Resource::Timber, 100);
    sim.add_build_type(b);
    sim.add_build_type(bt(Ident::University, university, 5, 5, 0));
    sim.add_unit_type(UnitType {
        tree: Some(soldier),
        price: cost::Price::free().with_base(Resource::Food, 50),
        ..UnitType::default()
    });
    sim.add_unit_type(UnitType {
        tree: Some(citizen),
        worker: Worker::Citizen,
        price: cost::Price::free().with_base(Resource::Food, 20),
        ..UnitType::default()
    });
    Fx {
        sim,
        village,
        tower,
        wonder,
        farm,
        barracks,
        soldier,
        citizen,
        tech,
        town,
        farm_rec,
        village_rec,
    }
}

fn slot(t: TypeId, val: i32) -> MakeObject {
    MakeObject {
        t: t as i32,
        val,
        ..MakeObject::EMPTY
    }
}

/// How many `Random::get(0, 0xffff)` calls separate two seeds.
fn draws(before: u32, after: u32) -> usize {
    let mut r = crate::combat::Rng::new(before);
    for n in 0..64 {
        if r.seed == after {
            return n;
        }
        r.get(0, 0xffff);
    }
    panic!("not reached within 64 draws");
}

/// Step 4 draws once for every slot holding the head's type, on every
/// call, whether or not the head could be paid for.
#[test]
fn the_head_s_expiry_draws_once_per_matching_slot() {
    // Three slots hold the head's type. The two behind it carry `val =
    // 0`, so the slot loop passes over them — but the expiry walk keys
    // on the type alone and draws for all three.
    let mut f = fx();
    let s = &mut f.sim;
    s.ai_state.ai[1].make_list.list[0] = slot(f.tech, 100);
    s.ai_state.ai[1].make_list.list[3] = slot(f.tech, 0);
    s.ai_state.ai[1].make_list.list[7] = slot(f.tech, 0);
    s.economy.ledgers[1].bucket = [0; RESOURCES];
    let before = s.rng.seed;
    s.make_stuff(1);
    assert_eq!(draws(before, s.rng.seed), 3);

    // A tower head clears every one of them outright, and still takes
    // the three draws: the `Random::get` is evaluated before the flag.
    let mut f = fx();
    let s = &mut f.sim;
    s.ai_state.ai[1].make_list.list[0] = slot(f.tower, 100);
    s.ai_state.ai[1].make_list.list[3] = slot(f.tower, 0);
    s.ai_state.ai[1].make_list.list[7] = slot(f.tower, 0);
    let before = s.rng.seed;
    s.make_stuff(1);
    assert_eq!(draws(before, s.rng.seed), 3);
    for k in [0, 3, 7] {
        assert_eq!(s.ai_state.ai[1].make_list.list[k].t, -1, "slot {k} cleared");
    }
}

/// A plain tech head expires one slot in three; a tower, a city type, a
/// wonder, a university and a military unit expire every duplicate.
#[test]
fn the_head_s_unconditional_expiry_is_the_decompile_s_five_cases() {
    let f = fx();
    let s = &f.sim;
    assert!(!s.expire_all(f.tech, true), "a plain tech is probabilistic");
    assert!(
        !s.expire_all(f.citizen, true),
        "a peasant is probabilistic — it is not a military unit"
    );
    assert!(s.expire_all(f.tower, true));
    assert!(s.expire_all(f.village, true), "a city type, through +0x64");
    assert!(s.expire_all(f.wonder, true));
    assert!(
        s.expire_all(f.soldier, true),
        "a unit type that is not a peasant — the clause docs/AI.md inverts"
    );
    // The non-head slots lose the last two clauses.
    assert!(!s.expire_all(f.soldier, false));
    assert!(s.expire_all(f.tower, false));
    assert!(s.expire_all(f.wonder, false));
}

/// **Run18b's five expiry draws, against the original's own seeds**
/// (`docs/RUNS.md`, "The producers' run"; `docs/AI.md` §15). The trace
/// records the seed before every `Random::get`, and the dump records
/// which slots survived — so the two together settle the arm that
/// `docs/AI.md` §2.6's prose gets backwards. An ordinary building (the
/// original's Temple; a Barracks here) takes the **probabilistic** arm,
/// and all five rolls decide the observed slot by `% 3 == 0`. Were the
/// arm unconditional, four of these five slots would be empty.
///
/// The seeds chain: each draw's successor is the next record's seed, and
/// the last is the frame's following draw at another site.
#[test]
fn run18_s_expiry_rolls_decide_exactly_the_slots_the_dump_kept() {
    let mut f = fx();
    let s = &mut f.sim;
    let temple = f.barracks; // an ordinary building — the Temple's class
    assert!(
        !s.expire_all(temple, true),
        "an ordinary building is probabilistic, head or not"
    );

    // sim-frame 6383: slots 0 (the head) and 8 hold it. Rolls 61545 and
    // 25792 — the head clears, slot 8 keeps.
    s.rng = crate::combat::Rng::new(0xc593_8177);
    s.ai_state.ai[1].make_list.list[0] = slot(temple, 2_499_999);
    s.ai_state.ai[1].make_list.list[8] = slot(temple, 2_499_999);
    s.expire(1, temple as i32, 0, false);
    assert_eq!(
        s.ai_state.ai[1].make_list.list[0].t, -1,
        "61545 % 3 == 0: cleared"
    );
    assert_eq!(
        s.ai_state.ai[1].make_list.list[8].t, temple as i32,
        "25792 % 3 == 1: kept"
    );
    assert_eq!(s.rng.seed, 0xbb3f_64c1, "two draws, the trace's next seed");

    // sim-frame 6582: the same two slots, the other way round — 17105
    // keeps the head, 1032 clears slot 8. Then the slot loop's own walk
    // (`make_stuff+0x63d`) from slot 5, over the citizen it just bought:
    // 48595 keeps it, and the demoted `val` stays in the list.
    s.rng = crate::combat::Rng::new(0x833a_ab7f);
    s.ai_state.ai[1].make_list.list[0] = slot(temple, 2_499_999);
    s.ai_state.ai[1].make_list.list[8] = slot(temple, 2_499_999);
    s.ai_state.ai[1].make_list.list[5] = slot(f.citizen, 714);
    s.expire(1, temple as i32, 0, false);
    assert_eq!(
        s.ai_state.ai[1].make_list.list[0].t, temple as i32,
        "17105 % 3 == 2: the head survived its own expiry"
    );
    assert_eq!(
        s.ai_state.ai[1].make_list.list[8].t, -1,
        "1032 % 3 == 0: cleared"
    );
    assert_eq!(s.rng.seed, 0xeb75_0409, "two draws");

    assert!(
        !s.expire_all(f.citizen, false),
        "a peasant slot is probabilistic"
    );
    s.expire(1, f.citizen as i32, 5, false);
    assert_eq!(
        s.ai_state.ai[1].make_list.list[5].t, f.citizen as i32,
        "48595 % 3 == 1: the bought slot is demoted, not cleared"
    );
    assert_eq!(
        s.rng.seed, 0x35dc_bdd4,
        "one draw, and the frame's next record's seed"
    );
}

/// **Run19's four, which separate the head clause from the rest**
/// (`docs/AI.md` §15.6). Run18b only ever had an ordinary building at
/// the head, so it could confirm the probabilistic arm and no more. Run19
/// puts a **scholar** — a unit type that is not a peasant, `0x34` — at
/// slot 0 on one frame and at slot 1 on another, and the same residue
/// decides opposite outcomes:
///
/// - sim-frame 8182, slot 1, *not* the head: roll 22883, `% 3 == 2`,
///   **kept** — the non-head walk is probabilistic for every type.
/// - sim-frame 8185, slot 0, the head: roll 45911, `% 3 == 2`, and the
///   dump shows it **gone** — the roll is taken and then ignored, which
///   is the unconditional arm and the exact clause `docs/AI.md` §2.6
///   inverts.
///
/// A same-type, same-residue, opposite-outcome pair is the strongest
/// evidence either run produced, and it fails under either inversion.
#[test]
fn run19_s_head_clause_clears_a_scholar_a_non_head_slot_would_keep() {
    let mut f = fx();
    let s = &mut f.sim;
    // `f.soldier` stands for the scholar: a unit type, not a peasant.
    let scholar = f.soldier;
    assert!(s.expire_all(scholar, true), "at the head: unconditional");
    assert!(!s.expire_all(scholar, false), "elsewhere: probabilistic");

    // 8182: the head is a tech (Coinage — an ordinary, probabilistic
    // type), bought and demoted; its duplicate at slot 4 clears; then the
    // slot loop's own walk keeps the scholar it just bought at slot 1.
    let coinage = f.tech;
    s.rng = crate::combat::Rng::new(0xa6d1_84cf);
    s.ai_state.ai[1].make_list.list[0] = slot(coinage, 9_999_999);
    s.ai_state.ai[1].make_list.list[1] = slot(scholar, 9_999_999);
    s.ai_state.ai[1].make_list.list[4] = slot(coinage, 9_999_999);
    s.expire(1, coinage as i32, 0, s.expire_all(coinage, true));
    assert_eq!(
        s.ai_state.ai[1].make_list.list[0].t, coinage as i32,
        "11233 % 3 == 1: the bought head is demoted, not cleared"
    );
    assert_eq!(
        s.ai_state.ai[1].make_list.list[4].t, -1,
        "14808 % 3 == 0: cleared"
    );
    s.expire(1, scholar as i32, 1, s.expire_all(scholar, false));
    assert_eq!(
        s.ai_state.ai[1].make_list.list[1].t, scholar as i32,
        "22883 % 3 == 2: a scholar at a non-head slot survives"
    );
    assert_eq!(s.rng.seed, 0x78f6_5964, "three draws");

    // 8185: the same scholar type, now the head, on a residue that would
    // have kept it anywhere else.
    s.rng = crate::combat::Rng::new(0x3f5a_529d);
    s.ai_state.ai[1].make_list.list = [MakeObject::EMPTY; MAKE_SLOTS];
    s.ai_state.ai[1].make_list.list[0] = slot(scholar, 9_999_999);
    s.expire(1, scholar as i32, 0, s.expire_all(scholar, true));
    assert_eq!(
        s.ai_state.ai[1].make_list.list[0].t, -1,
        "45911 % 3 == 2 and it goes anyway: the head clause is unconditional"
    );
    assert_eq!(s.rng.seed, 0x8244_b358, "the roll is still taken");
}

/// `need` is the mean shortfall over the goods the head costs, floored
/// at zero, and only when the leader is saving up.
///
/// The barracks head costs 1000 food and 1000 timber; the farm in slot 8
/// costs 600 food. The food purse sits a hundred over `head + slot`, so
/// the slot's fate turns on `need` alone — and `need` is the mean over
/// **both** the head's goods, so how short the *timber* purse is decides
/// whether a food purchase goes through.
#[test]
fn the_mean_shortfall_gates_the_rest_of_the_list() {
    fn case(timber_purse: i32) -> usize {
        let mut f = fx();
        let s = &mut f.sim;
        let pb = s.type_price(1, f.barracks).expect("a price");
        let pf = s.type_price(1, f.farm).expect("a price");
        let (fo, ti) = (Resource::Food as usize, Resource::Timber as usize);
        assert_eq!((pb[fo], pb[ti], pf[fo], pf[ti]), (1000, 1000, 600, 0));
        let mut head = slot(f.barracks, 100);
        head.num = 1; // so `can_pay` can answer no
        s.ai_state.ai[1].make_list.list[0] = head;
        s.ai_state.ai[1].make_list.list[8] = slot(f.farm, 50);
        s.economy.ledgers[1].bucket[fo] = pb[fo] + pf[fo] + 100;
        s.economy.ledgers[1].bucket[ti] = timber_purse;
        let before = s.rng.seed;
        s.make_stuff(1);
        draws(before, s.rng.seed)
    }
    // No timber: need = ((1000 − 1700) + (1000 − 0)) / 2 = 150, and
    // 1700 < 1000 + 600 + 150. Only the head expires.
    assert_eq!(case(0), 1);
    // 900 timber: need = (−700 + 100) / 2 → floored to 0, and 1700 ≥
    // 1600. The slot runs and takes an expiry of its own.
    assert_eq!(case(900), 2);
}

/// Slot 5 buys through a shortfall when the leader has no workers at
/// all; slot 4 buys a gather building for a good `econ` calls short.
#[test]
fn slots_four_and_five_have_their_exceptions() {
    // Every case: a barracks head (1000 food, 1000 timber) over a purse
    // of 600 food — the farm's own price and nothing else — so the head
    // cannot be paid, the slot under test is unaffordable-while-saving,
    // and only its exception can buy it; `can_pay` (`num 1` against the
    // affordable count) then lets the one farm through. One draw is the
    // head alone; two means the slot ran.
    fn case(slot_no: usize, t: TypeId, set: impl FnOnce(&mut Sim)) -> usize {
        let mut f = fx();
        let s = &mut f.sim;
        let head = slot(f.barracks, 100);
        s.ai_state.ai[1].make_list.list[0] = head;
        s.ai_state.ai[1].make_list.list[slot_no] = slot(t, 50);
        s.economy.ledgers[1].bucket[Resource::Food as usize] = 600;
        set(s);
        let before = s.rng.seed;
        s.make_stuff(1);
        draws(before, s.rng.seed)
    }
    let f = fx();
    let (farm, tower) = (f.farm, f.tower);
    // Slot 5 with a free peasant: skipped. With none: bought anyway.
    assert_eq!(
        case(5, farm, |s| s.ai_state.ai[1].census.free_peasants = 1),
        1
    );
    assert_eq!(case(5, farm, |s| s.ai_state.ai[1].census.gatherers = 1), 1);
    assert_eq!(case(5, farm, |_| {}), 2);
    // Slot 4, a farm, with food's `econ` bit 4 clear: skipped; set: bought.
    assert_eq!(case(4, farm, |_| {}), 1);
    assert_eq!(
        case(4, farm, |s| s.ai_state.ai[1].econ
            [Resource::Food as usize] = 4),
        2
    );
    // The exception is the *type's own* good and only a gather building
    // gets it: a tower in slot 4 is skipped however short food is.
    assert_eq!(
        case(4, tower, |s| s.ai_state.ai[1].econ
            [Resource::Food as usize] = 4),
        1
    );
    // And neither exception reaches any other slot.
    assert_eq!(case(6, farm, |_| {}), 1);
}

/// From slot 4 on, a `(t, city)` already in **any** of the ranked four is
/// dropped — slot 0 included — and the city has to match too.
#[test]
fn a_slot_that_repeats_the_ranked_four_is_dropped() {
    /// A seed whose next `n` draws all miss the one-in-three, so an
    /// expiry walk over `n` slots clears nothing and the dup rule is
    /// what decides whether slot 9 runs.
    fn quiet_seed(n: usize) -> u32 {
        for seed in 1u32..10_000 {
            let mut r = crate::combat::Rng::new(seed);
            if (0..n).all(|_| r.get(0, 0xffff) % 3 != 0) {
                return seed;
            }
        }
        panic!("no quiet seed");
    }
    // `k` is the ranked slot slot 9 duplicates; `city` is slot 9's.
    fn case(k: usize, city: i32) -> usize {
        let mut f = fx();
        let s = &mut f.sim;
        s.rng.seed = quiet_seed(4);
        // The head is a farm at city 0, so `k == 0` is a real case; the
        // farm's expiry is probabilistic and the quiet seed keeps every
        // slot alive through it.
        let mut head = slot(f.farm, 100);
        head.city = 0;
        s.ai_state.ai[1].make_list.list[0] = head;
        if k != 0 {
            let mut m = slot(f.farm, 0); // `val == 0`: the slot loop skips it
            m.city = 0;
            s.ai_state.ai[1].make_list.list[k] = m;
        }
        let mut dup = slot(f.farm, 10);
        dup.city = city;
        s.ai_state.ai[1].make_list.list[9] = dup;
        s.economy.ledgers[1].bucket = [10_000; RESOURCES];
        let before = s.rng.seed;
        s.make_stuff(1);
        draws(before, s.rng.seed)
    }
    // The head's expiry walk always draws for slot 0, the ranked slot and
    // slot 9 — every one holds the farm. A slot 9 that survives the dup
    // test adds one more, its own expiry walk.
    for k in 0..4 {
        let n = if k == 0 { 2 } else { 3 };
        assert_eq!(case(k, 0), n, "slot {k}: the duplicate was dropped");
        assert_eq!(case(k, 1), n + 1, "slot {k}: a different city runs");
    }
}

/// `make_this` routes by class, and a producer that bought demotes the
/// slot's value a hundredfold rather than clearing it.
#[test]
fn make_this_dispatches_by_class_and_demotes_on_a_buy() {
    let mut f = fx();
    let s = &mut f.sim;
    // Every producer is a stub returning "could not", so nothing is
    // demoted and nothing panics on any of the four classes.
    for (i, t) in [f.tech, f.soldier, f.farm, f.village]
        .into_iter()
        .enumerate()
    {
        s.ai_state.ai[1].make_list.list[i] = slot(t, 500);
        assert!(!s.make_this(1, i));
        assert_eq!(s.ai_state.ai[1].make_list.list[i].val, 500);
    }
    // A good in the list is the original's bare `return 1`.
    s.ai_state.ai[1].make_list.list[0] = slot(0, 500);
    assert!(!s.make_this(1, 0));
    // The `is(TOWN, 0)` arm the listing settled: a Large City is a city
    // type *and* an upgrade, and the upgrade test comes first. Both
    // producers are stubs, so the routing is not yet observable through
    // a return value; what is pinned here is the predicate that picks it.
    assert!(s.type_is_role(f.town, s.tables.tech_tree.roles.town, false));
    assert!(s.slot64(f.town), "and it is also is_city, hence the order");
    assert!(!s.type_is_role(f.village, s.tables.tech_tree.roles.town, false));
    s.ai_state.ai[1].make_list.list[0] = slot(f.town, 500);
    assert!(!s.make_this(1, 0));

    // The demotion itself, driven through the one producer that is not a
    // stub: a farm placed next to a city.
    let mut f = fx();
    let s = &mut f.sim;
    let at = tile_pos(32, 32);
    let b = s.init_build(1, f.village_rec, at, false);
    s.activate(b, false, true);
    let c = s.buildings[b].city.expect("a finished city has a record");
    s.ai_state.ai[1].city_ai = vec![CityAi::default(); s.cities_state.cities.len()];
    s.ai_state.ai[1].city_ai[c].land = 40;
    s.ai_state.ai[1].city_ai[c].space = [40, 40, 40];
    s.ai_state.ai[1].census.resize(256, 2);
    let reg = s.world.region_of(at.cell()).expect("a region");
    s.ai_state.ai[1].census.reg_free_peasants[usize::from(reg)] = 1;
    s.economy.ledgers[1].bucket = [10_000; RESOURCES];
    let mut m = slot(f.farm, 500);
    m.city = c as i32;
    s.ai_state.ai[1].make_list.list[0] = m;
    assert!(s.make_this(1, 0), "produce_building placed the farm");
    assert_eq!(s.ai_state.ai[1].make_list.list[0].val, 5, "val /= 100");
    assert_eq!(s.tables.build_types[f.farm_rec].ident, Ident::Farm);
}

/// An active market of my own, which is half the gate.
fn give_a_market(s: &mut Sim) {
    let market = s.add_build_type(BuildType {
        ident: Ident::Market,
        x_size: 4,
        y_size: 4,
        job_time: 100,
        hits: 100,
        ..BuildType::default()
    });
    let b = s.init_build(1, market, tile_pos(32, 32), false);
    s.activate(b, false, true);
}

/// **run117's own frame, as a unit test** — `market_speculation`'s buy
/// pass takes the **first** good in index order whose price its purse
/// can reach, and four wealth decide which one that is.
///
/// Great Lakes sim-frame 10576, measured both ways (`docs/ECONOMY.md`
/// §13). The original enters the rotation's `Setup` holding
/// `93 86 128 269 100 0` and buys a hundred **food** for **128**,
/// leaving nothing; this crate entered it holding `94 86 124 …` —
/// four wealth short on an income row that has been wrong since
/// before the window — and food's price is 128 on both sides, so its
/// purse cannot reach it and the pass falls through to **timber** at
/// 94. Five frames later the AI's `make_stuff` is spending a purse
/// the original does not have, and that was Great Lakes' word.
///
/// The prices are the frame's own: food `(buy 128, sell 83)` and
/// timber `(94, 58)`, which is `price 45 flux 38` and `price 36
/// flux 22` through [`Sim::calc_market_prices`].
///
/// **Made to fail on purpose** with the purse test written `<`
/// instead of `<=`: 128 against a purse of exactly 128 is the
/// boundary the original sits on, and a strict comparison buys the
/// timber on both halves below.
#[test]
fn market_speculation_buys_the_first_good_its_purse_can_reach() {
    let buy_at = |purse: i32| -> [i32; RESOURCES] {
        let mut f = fx();
        let s = &mut f.sim;
        give_a_market(s);
        s.tech[1].epoch[Line::Commerce.index()] = BUY_SELL_COMMERCE_LEVEL;
        // Oil is not available before the Industrial age, which is what
        // run117's own record says and what keeps a stock of nought out
        // of the tier.
        s.economy.holdings[1].available[Resource::Oil as usize] = false;
        for (g, (price, flux)) in [(45, 38), (36, 22), (50, 15), (50, 15), (50, 15), (50, 15)]
            .into_iter()
            .enumerate()
        {
            s.economy.market.price[g] = price;
            s.economy.market.flux[g] = flux;
        }
        s.economy.ledgers[1].bucket = [94, 86, purse, 269, 100, 0];
        // The frame's own prices, asserted rather than assumed: a
        // change in `calc_market_prices` must not quietly turn this
        // test into a statement about some other pair of numbers.
        assert_eq!(s.calc_market_prices(1, 0), (128, 83), "food's price");
        assert_eq!(s.calc_market_prices(1, 1), (94, 58), "timber's price");
        s.market_speculation(1);
        s.economy.ledgers[1].bucket
    };
    // The original's purse: food is reachable at exactly its price, so
    // food is what it buys and the purse is emptied.
    assert_eq!(
        buy_at(128),
        [194, 86, 0, 269, 100, 0],
        "a purse of 128 buys the food at 128 and nothing after it"
    );
    // This crate's purse on the same frame. One wealth less and the
    // whole rotation turns: timber instead of food, and 30 left over.
    assert_eq!(
        buy_at(127),
        [94, 186, 33, 269, 100, 0],
        "a purse of 127 cannot reach the food and buys the timber"
    );
}

/// The buy pass's **ladder**, which is the half that has no tier in it:
/// a stock of 500 or more wants a price under 26, 200 wants under 51,
/// 100 wants under 101, and anything under 100 wants only under 201.
/// A stock of 2000 buys nothing at any price.
///
/// Walked on one good with the purse held wide open, so the only thing
/// deciding each row is the pair `(stock, buy)`.
#[test]
fn the_buy_pass_ladder_gets_stricter_as_the_pile_grows() {
    let bought = |stock: i32, price: i32, flux: i32| -> bool {
        let mut f = fx();
        let s = &mut f.sim;
        give_a_market(s);
        s.tech[1].epoch[Line::Commerce.index()] = BUY_SELL_COMMERCE_LEVEL;
        s.economy.holdings[1].available[Resource::Oil as usize] = false;
        // Every other good is priced out of reach, so the row is timber's.
        for g in 0..RESOURCES {
            s.economy.market.price[g] = 1_000;
            s.economy.market.flux[g] = 0;
        }
        s.economy.market.price[1] = price;
        s.economy.market.flux[1] = flux;
        s.economy.ledgers[1].bucket = [2_000, stock, 100_000, 2_000, 2_000, 0];
        let before = s.economy.ledgers[1].bucket[1];
        s.market_speculation(1);
        s.economy.ledgers[1].bucket[1] == before + 100
    };
    // `buy = 2 * price + flux`, so `(12, 1)` is 25 and `(12, 2)` is 26.
    assert!(bought(500, 12, 1), "500 at 25 is under the rung");
    assert!(!bought(500, 12, 2), "500 at 26 is not");
    assert!(bought(200, 25, 0), "200 at 50 is under the rung");
    assert!(!bought(200, 25, 1), "200 at 51 is not");
    assert!(bought(100, 50, 0), "100 at 100 is under the rung");
    assert!(!bought(100, 50, 1), "100 at 101 is not");
    assert!(bought(99, 100, 0), "99 at 200 is under the last rung");
    assert!(!bought(99, 100, 1), "99 at 201 is not");
    // And the ceiling on the pile itself, whatever the price.
    assert!(!bought(2_000, 1, 0), "a stock of 2000 buys nothing");
    assert!(bought(1_999, 12, 1), "1999 does, at a price 500 allows");
}

/// The **sell** pass, and both of its tier-scaled thresholds: a pile of
/// `2000 >> tier` over the escrow, at a sell price of `75 / (tier + 1)`
/// or better. A hungrier leader parts with a smaller pile and takes a
/// worse price — which is the whole of what the tier does here.
#[test]
fn the_sell_pass_scales_both_of_its_thresholds_by_the_tier() {
    // `hungry` sets the tier by the smallest stock: 2 under 100, 1 under
    // 200, 0 otherwise. Timber is the good under test in every row.
    let sold = |tier: i32, stock: i32, price: i32, flux: i32| -> bool {
        let mut f = fx();
        let s = &mut f.sim;
        give_a_market(s);
        s.tech[1].epoch[Line::Commerce.index()] = BUY_SELL_COMMERCE_LEVEL;
        s.economy.holdings[1].available[Resource::Oil as usize] = false;
        // Nothing else is sellable (a pile under 250) and nothing is
        // buyable (a price out of reach), so the row is timber's alone.
        for g in 0..RESOURCES {
            s.economy.market.price[g] = 100_000;
            s.economy.market.flux[g] = 0;
        }
        s.economy.market.price[1] = price;
        s.economy.market.flux[1] = flux;
        let floor = match tier {
            2 => 99,
            1 => 199,
            _ => 249,
        };
        // **Wealth counts towards the tier too** — the first pass
        // reads every available good, wealth and knowledge included,
        // and a purse of nought would make every row tier 2. So the
        // purse is held at the tier's floor and **escrowed away**:
        // the buy pass spends `bucket − escrow` where the tier reads
        // `bucket`, so this sets the tier without letting the third
        // pass buy back what the second sold. (It would: a pile of
        // 500 sold down to 400 leaves the 26-rung behind and the
        // 51-rung in front of a price of 50.) The clamp only fires
        // over 4000, so these floors stand.
        s.economy.ledgers[1].bucket = [floor, stock, floor, floor, floor, 0];
        s.economy.ledgers[1].escrow[Resource::Wealth as usize] = floor;
        let before = s.economy.ledgers[1].bucket[1];
        s.market_speculation(1);
        s.economy.ledgers[1].bucket[1] == before - 100
    };
    // `sell = price + flux`. Tier 0: 2000 of the pile at a price of 75.
    assert!(sold(0, 2_000, 75, 0), "tier 0: 2000 at 75");
    assert!(!sold(0, 1_999, 75, 0), "tier 0: 1999 is short of the pile");
    assert!(!sold(0, 2_000, 74, 0), "tier 0: 74 is short of the price");
    // Tier 1: half the pile, and `75 / 2 = 37`.
    assert!(sold(1, 1_000, 37, 0), "tier 1: 1000 at 37");
    assert!(!sold(1, 999, 37, 0), "tier 1: 999 is short of the pile");
    assert!(!sold(1, 1_000, 36, 0), "tier 1: 36 is short of the price");
    // Tier 2: a quarter, and `75 / 3 = 25`.
    assert!(sold(2, 500, 25, 0), "tier 2: 500 at 25");
    assert!(!sold(2, 499, 25, 0), "tier 2: 499 is short of the pile");
    assert!(!sold(2, 500, 24, 0), "tier 2: 24 is short of the price");
}

/// `market_speculation` clamps a runaway escrow — and only behind the
/// market gate.
#[test]
fn market_speculation_clamps_escrow_behind_the_gate() {
    let mut f = fx();
    let s = &mut f.sim;
    s.economy.ledgers[1].escrow[Resource::Food as usize] = 9000;
    s.market_speculation(1);
    assert_eq!(
        s.economy.ledgers[1].escrow[Resource::Food as usize],
        9000,
        "no market: the whole function is skipped"
    );
    // Give the leader an active market — and, since item 348, the
    // **Coinage** that `BUY_SELL` actually asks for. A market alone is
    // Barter, one library tech short of the ability.
    give_a_market(s);
    s.market_speculation(1);
    assert_eq!(
        s.economy.ledgers[1].escrow[Resource::Food as usize],
        9000,
        "a market without Coinage is not the ability"
    );
    s.tech[1].epoch[Line::Commerce.index()] = BUY_SELL_COMMERCE_LEVEL;
    s.market_speculation(1);
    assert_eq!(s.economy.ledgers[1].escrow[Resource::Food as usize], 2000);
    // Under 4000 it stands.
    s.economy.ledgers[1].escrow[Resource::Timber as usize] = 3999;
    s.market_speculation(1);
    assert_eq!(s.economy.ledgers[1].escrow[Resource::Timber as usize], 3999);
}

/// **`use_market` spends one draw on a wealth shortfall, and none on
/// anything else** — item 348, `docs/AI.md` §40.
///
/// The gate is Coinage *and* a market; the draw is the sell rotation's
/// offset, which wealth reaches without a price because
/// `TVar3 == WEALTH` short-circuits `calc_market_prices` away.
#[test]
fn the_market_draws_once_for_wealth_behind_coinage() {
    let spent = |s: &mut Sim| {
        let before = s.rng.seed;
        s.use_market(1);
        draws(before, s.rng.seed)
    };
    let f_tower = fx().tower;
    let mut f = fx();
    let s = &mut f.sim;
    // A make-list head that costs 64 wealth and nothing else, which is
    // the shape of Great Lakes 8582 — and that map's own bucket.
    let rec = s
        .tables
        .build_types
        .iter()
        .position(|b| b.ident == Ident::Tower)
        .expect("the fixture's tower");
    // The fixture's prices come out of `type_price` ten times their
    // base, so 6 is the 60 a make-list slot sees.
    s.tables.build_types[rec].price = cost::Price::free().with_base(Resource::Wealth, 6);
    s.ai_state.ai[1].make_list.list[0] = slot(f_tower, 500);
    s.economy.holdings[1].available = [true, true, true, true, true, false];
    s.economy.ledgers[1].bucket = [58, 55, 43, 68, 54, 0];
    assert_eq!(spent(s), 0, "no market, no ability: nothing is spent");
    give_a_market(s);
    assert_eq!(spent(s), 0, "a market is Barter, not Coinage");
    s.tech[1].epoch[Line::Commerce.index()] = BUY_SELL_COMMERCE_LEVEL;
    assert_eq!(spent(s), 1, "wealth is 43 against a need of 60");
    assert_eq!(
        s.economy.ledgers[1].bucket,
        [58, 55, 43, 68, 54, 0],
        "nothing here clears `need <= bucket - 100`, so nothing is sold"
    );
    // **Raise food over the bar and the sale goes through** (item
    // 358). It is still *one* draw — not because nothing was
    // sellable, but because the sale covers the need and the outer
    // `while` exits before the once-flag is consulted. A hundred food
    // leaves for the default market's `price + flux`, 65.
    s.economy.ledgers[1].bucket[Resource::Food as usize] = 300;
    assert_eq!(spent(s), 1, "a covering sale is also one draw");
    assert_eq!(
        s.economy.ledgers[1].bucket,
        [200, 55, 108, 68, 54, 0],
        "a hundred food for 65 wealth"
    );
    assert_eq!(
        s.economy.market.price[Resource::Food as usize],
        47,
        "supply walks it down"
    );
    // A bucket that covers the need takes nothing at all.
    s.economy.ledgers[1].bucket[Resource::Wealth as usize] = 60;
    assert_eq!(spent(s), 0, "wealth is no longer short");
    // And a shortfall in anything but wealth is **bought**, not sold
    // for, when wealth can stand the buy price — and a buy spends no
    // draw at all.
    s.tables.build_types[rec].price = cost::Price::free().with_base(Resource::Food, 6);
    s.economy.ledgers[1].bucket = [0, 0, 10_000, 0, 0, 0];
    s.economy.market.price = [50; crate::market::GOODS];
    assert_eq!(
        spent(s),
        0,
        "food's branch is a buy, and a buy is draw-free"
    );
    assert_eq!(
        s.economy.ledgers[1].bucket,
        [100, 0, 9_885, 0, 0, 0],
        "a hundred food for the buy price, 2 x 50 + 15"
    );
}

/// **The market's three prices and its two trades** — item 358,
/// `docs/ECONOMY.md` §12. Capture-free, and every clamp on its own
/// line.
#[test]
fn the_market_prices_a_trade_and_moves_its_own_price() {
    let mut f = fx();
    let s = &mut f.sim;
    s.economy.holdings[1].available = [true; RESOURCES];
    let g = Resource::Timber as usize;
    // The default market: price 50, flux 15. Sell is `price + flux`,
    // buy is `2 x price + flux`.
    assert_eq!(s.calc_market_prices(1, g), (115, 65));
    // The sell floor of 1, and the buy floor of `MARKET_BASEMENT x 2`
    // — which a price of zero reaches before the `sell + 10` floor.
    s.economy.market.price[g] = 0;
    s.economy.market.flux[g] = 0;
    assert_eq!(s.calc_market_prices(1, g), (20, 1));
    // And the spread floor: a buy is never under ten over a sell.
    s.economy.market.flux[g] = 40;
    assert_eq!(s.calc_market_prices(1, g), (50, 40));
    // Amber widens the spread by `AMBER_MARKET` in the holder's
    // favour — added to the sell, taken off the buy, never both.
    s.economy.market.price[g] = 50;
    s.economy.market.flux[g] = 15;
    s.economy.ledgers[1].rare = 1 << (crate::economy::AMBER - crate::economy::BASE_RARE);
    assert_eq!(s.calc_market_prices(1, g), (105, 75));
    s.economy.ledgers[1].rare = 0;
    // `do_sell`: a hundred out, the sell price in, the price down by
    // `MARKET_SUPPLY_DEMAND`, and the good's escrow released.
    s.economy.ledgers[1].bucket = [0, 250, 7, 0, 0, 0];
    s.economy.ledgers[1].escrow[g] = 40;
    assert!(s.do_sell(1, g));
    assert_eq!(s.economy.ledgers[1].bucket, [0, 150, 72, 0, 0, 0]);
    assert_eq!(
        (s.economy.market.price[g], s.economy.ledgers[1].escrow[g]),
        (47, 0)
    );
    // Under a hundred in the bucket is the one refusal, and it moves
    // nothing.
    s.economy.ledgers[1].bucket[g] = 99;
    assert!(!s.do_sell(1, g));
    assert_eq!(
        (s.economy.ledgers[1].bucket[g], s.economy.market.price[g]),
        (99, 47)
    );
    // `do_buy`: the buy price out of **wealth**, a hundred of the
    // good in, the price up, and it is *wealth's* escrow that is
    // released.
    s.economy.market.price[g] = 50;
    s.economy.ledgers[1].bucket = [0, 0, 200, 0, 0, 0];
    s.economy.ledgers[1].escrow = [0, 0, 130, 0, 0, 0];
    assert!(s.do_buy(1, g));
    assert_eq!(s.economy.ledgers[1].bucket, [0, 100, 85, 0, 0, 0]);
    assert_eq!(
        (
            s.economy.market.price[g],
            s.economy.ledgers[1].escrow[Resource::Wealth as usize]
        ),
        (53, 30)
    );
    // Wealth under the price refuses, and moves nothing.
    s.economy.ledgers[1].bucket[Resource::Wealth as usize] = 10;
    assert!(!s.do_buy(1, g));
    assert_eq!(
        (s.economy.ledgers[1].bucket[g], s.economy.market.price[g]),
        (100, 53)
    );
}

/// **The builder walk is in object-number order** (item 2130,
/// `docs/AI.md` §210; `Leader::produce_building@006e1400`,
/// `006e2650`–`006e27b6`, walks `objects->units[who][0..count]` and keeps
/// a tie on `<`). Two citizens on one tile tie for the farm; the one with
/// the lower `o` is sent though it was born second — run820's `1/186`
/// (uid 346, a recycled number) against `1/208` (uid 334) on 4182.
///
/// Made to fail once with the walk in `units` order: the elder `1/9` is
/// sent.
#[test]
fn a_tied_builder_is_the_lower_object_number_not_the_elder() {
    let mut f = fx();
    let s = &mut f.sim;
    s.frame = 100;
    let at = tile_pos(32, 32);
    let b = s.init_build(1, f.village_rec, at, false);
    s.activate(b, false, true);
    let c = s.buildings[b].city.expect("a finished city has a record");
    s.ai_state.ai[1].city_ai = vec![CityAi::default(); s.cities_state.cities.len()];
    s.ai_state.ai[1].city_ai[c].land = 40;
    s.ai_state.ai[1].city_ai[c].space = [40, 40, 40];
    s.ai_state.ai[1].census.resize(256, 2);
    let reg = s.world.region_of(at.cell()).expect("a region");
    s.ai_state.ai[1].census.reg_free_peasants[usize::from(reg)] = 2;
    s.economy.ledgers[1].bucket = [10_000; RESOURCES];
    // The tree's first unit type is the one `produce_building` counts
    // as a citizen (`0x32`/`0x33` in the original): record 0 here.
    let citizen = s
        .tables
        .unit_types
        .iter()
        .position(|t| {
            t.tree.is_some_and(|id| {
                Some(id)
                    == s.tables
                        .tech_tree
                        .types
                        .iter()
                        .position(|d| d.kind.is_unit())
            })
        })
        .expect("the counted type");
    for o in [9, 3] {
        let mut u = crate::Unit::new(1, o, tile_pos(38, 32), 40);
        u.ty = Some(citizen);
        s.add_unit(u);
    }
    let mut m = slot(f.farm, 500);
    m.city = c as i32;
    s.ai_state.ai[1].make_list.list[0] = m;
    assert!(s.make_this(1, 0), "produce_building placed the farm");
    let sent: Vec<i16> = s
        .units
        .iter()
        .filter(|u| u.orders.iter().any(|o| matches!(o.body, Body::Build(_))))
        .map(|u| u.index)
        .collect();
    assert_eq!(sent, vec![3], "the lower number keeps the tie");
}

/// A never-started site with no builder is disbanded where the danger
/// map is positive, and gets a citizen where it is not.
#[test]
fn the_orphan_check_disbands_or_recruits() {
    let at = tile_pos(32, 32);
    let mut f = fx();
    let s = &mut f.sim;
    let b = s.init_build(1, f.farm_rec, at, false);
    assert!(!s.buildings[b].active && s.buildings[b].job_counter == 0);
    s.world.set_danger_at(1, at.cell(), 5);
    s.check_orphaned_buildings(1);
    assert!(!s.buildings[b].alive, "disbanded: no builder, danger > 0");

    // With no danger the site keeps, and an idle citizen in the region is
    // sent to build it.
    let mut f = fx();
    let s = &mut f.sim;
    let b = s.init_build(1, f.farm_rec, at, false);
    let mut u = crate::Unit::new(1, 0, tile_pos(36, 32), 40);
    u.ty = Some(1); // the citizen record
    s.units.push(u);
    s.check_orphaned_buildings(1);
    assert!(s.buildings[b].alive);
    assert!(
        s.units[0]
            .orders
            .iter()
            .any(|o| matches!(o.body, Body::Build(x) if x == b)),
        "the citizen was swarmed onto the site"
    );

    // A builder already on it in the same region is left alone.
    let before = s.units[0].orders.len();
    s.check_orphaned_buildings(1);
    assert_eq!(s.units[0].orders.len(), before);
}

/// A human leader is never swept.
#[test]
fn a_human_leader_is_skipped() {
    let at = tile_pos(32, 32);
    let mut f = fx();
    let s = &mut f.sim;
    let b = s.init_build(0, f.farm_rec, at, false);
    s.world.set_danger_at(0, at.cell(), 5);
    s.cities_state.nation[0].human = true;
    s.check_orphaned_buildings(0);
    assert!(s.buildings[b].alive);
}
