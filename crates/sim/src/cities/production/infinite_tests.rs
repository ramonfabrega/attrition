use crate::build::{BuildType, Ident, flags};
use crate::economy::RESOURCES;
use crate::{Sim, tech};

/// A Barracks (a training building) and one unit type it trains.
fn barracks(researched: bool) -> (Sim, usize, usize) {
    let mut w = crate::world::World::new(16, 16);
    w.fill_region(
        crate::world::Terrain::Land,
        crate::world::Cell::new(0, 0),
        crate::world::Cell::new(15, 15),
    );
    let mut sim = Sim::new(crate::Tuning::RON, w, 1);
    let mut tree = tech::TechTree::new();
    for n in ["Food", "Timber", "Wealth", "Knowledge", "Metal", "Oil"] {
        tree.add(tech::TypeDef::good(n));
    }
    let unit = tree.add(tech::TypeDef::unit("Bowmen", tech::UnitTraits::default()));
    let build = tree.add(tech::TypeDef::building("Barracks"));
    tree.types[unit].where_ = Some(build);
    tree.finalize();
    sim.set_tech_tree(tree);
    sim.tech[0].tech[unit] = true;
    sim.tech[0].tech[build] = true;
    let mut ty = crate::UnitType {
        tree: Some(unit),
        ..crate::UnitType::default()
    };
    ty.price.pop = 1;
    ty.price.base[crate::economy::Resource::Wealth.index()] = 50;
    let rec = sim.add_unit_type(ty);
    sim.muster[0].researched[rec] = researched;
    let brec = sim.tables.build_types.len();
    sim.tables.build_types.push(BuildType {
        ident: Ident::Barracks,
        tree: Some(build),
        flags: flags::TRAINS,
        x_size: 2,
        y_size: 2,
        hits: 100,
        ..BuildType::default()
    });
    let b = sim.add_building(0, crate::Pos::new(4 * 256, 4 * 256), 20);
    sim.buildings[b].ty = Some(brec);
    sim.muster[0].cap = 100;
    sim.ledgers[0].bucket = [10_000; RESOURCES];
    sim.holdings[0].available = [true; RESOURCES];
    (sim, b, rec)
}

/// **A Disband queued at a Barracks** (item 3918, `docs/GOLDEN.md`
/// §165): run1512's `0/2007` takes the entry, holds it 21000 (`JOB_TIME`
/// 420 × 100 / 2) at 100 a frame and is gone on the frame after it
/// reaches the end; a second Barracks' press is refused whole while the
/// first holds one (`Group::action_queue_up`'s research arm, run1512's
/// 641 and 661).
#[test]
fn a_disband_queued_at_a_barracks_closes_it_when_the_job_completes() {
    let (mut s, b, _) = barracks(true);
    let bt = s.buildings[b].ty.unwrap();
    s.tables.build_types[bt].job_time = 420;
    let b2 = s.add_building(0, crate::Pos::new(10 * 256, 10 * 256), 20);
    s.buildings[b2].ty = Some(bt);
    assert_eq!(s.action_queue_disband(&[b]), 1);
    assert_eq!(s.buildings[b].queue.items.len(), 1);
    assert!(s.buildings[b].queue.items[0].disband);
    assert_eq!(s.disband_time(b), 21000);
    assert_eq!(s.queue_target(b, 0), 21000);
    assert_eq!(s.action_queue_disband(&[b2]), 0, "the research arm refuses");
    assert!(s.buildings[b2].queue.items.is_empty());
    assert_eq!(s.action_queue_disband(&[b]), 0, "one is held");
    assert_eq!(s.buildings[b].queue.items.len(), 1);
    let bucket = s.ledgers[0].bucket;
    let mut out = Vec::new();
    for _ in 0..210 {
        s.process_queue(b, &mut out);
    }
    assert!(s.buildings[b].alive, "at 21000 and not yet finished");
    assert_eq!(s.buildings[b].queue.items[0].job_counter, 21000);
    s.process_queue(b, &mut out);
    assert!(!s.buildings[b].alive, "Build::finished cast it");
    assert!(s.buildings[b].queue.items.is_empty());
    assert!(out.is_empty(), "no unit");
    assert_eq!(s.ledgers[0].bucket, bucket, "a Barracks plunders nothing");
    assert_eq!(s.action_queue_disband(&[b2]), 1, "the second may go now");
}

/// **`is_castable`'s city arm** (`CityData::is_disbanding`): in a city,
/// a building may not queue a Disband while another of the city's
/// holds one, and at the finish its own entry is allowed (1 > 1 is
/// false). No capture disbands in a city (parked 3918a).
#[test]
fn a_city_holds_one_disband_at_the_queue_and_allows_its_own_at_the_finish() {
    let (mut s, b, _) = barracks(true);
    let bt = s.buildings[b].ty.unwrap();
    // A type that needs a city (`NO_CITY` clear): the arm is asked.
    s.tables.build_types[bt].flags &= !crate::build::flags::NO_CITY;
    let b2 = s.add_building(0, crate::Pos::new(10 * 256, 10 * 256), 20);
    s.buildings[b2].ty = Some(bt);
    s.buildings[b].city = Some(0);
    s.buildings[b2].city = Some(0);
    s.buildings[b2].queue.push_disband();
    assert!(!s.disband_castable(b, false), "the city holds one");
    assert!(s.disband_castable(b2, true), "its own entry at the finish");
    s.buildings[b].queue.push_disband();
    assert!(!s.disband_castable(b2, true), "two in the city");
    s.buildings[b].city = None;
    s.buildings[b].queue.items.clear();
    assert!(s.disband_castable(b, false), "outside any city");
    // `researching(.., o, 1, 0)`, the building's own queue: the group's
    // research arm refuses a second press first, so only a direct
    // `Build::queue_up` reaches it.
    assert!(s.queue_disband(b));
    assert!(!s.disband_castable(b, false), "its own entry stands");
    assert!(s.disband_castable(b, true), "and is allowed at the finish");
    assert!(!s.queue_disband(b));
    assert_eq!(s.buildings[b].queue.count_disband(), 1);
}

/// **A `NO_CITY` building skips the city arm** (item 3930, `docs/GOLDEN.md`
/// §168): `is_castable`'s 0x29a case asks `is_disbanding` only of a type
/// with `+0x2c0 & 0x10` clear, or a city, and `is_disbanding` counts no
/// `NO_CITY` building. A Barracks (`e`) in a city whose other building
/// holds a `DISBAND` still queues; run1517b's twin enters no
/// `CityData::is_disbanding` on the press at the Barracks `0/2008`.
#[test]
fn a_no_city_building_skips_the_city_s_disband_count() {
    let (mut s, b, _) = barracks(true);
    let bt = s.buildings[b].ty.unwrap();
    s.tables.build_types[bt].flags |= crate::build::flags::NO_CITY;
    let temple = s.tables.build_types.len();
    s.tables.build_types.push(BuildType::default());
    let b2 = s.add_building(0, crate::Pos::new(10 * 256, 10 * 256), 20);
    s.buildings[b2].ty = Some(temple);
    s.buildings[b].city = Some(0);
    s.buildings[b2].city = Some(0);
    s.buildings[b2].queue.push_disband();
    assert!(s.disband_castable(b, false), "the Barracks skips the arm");
    // And the Barracks' own entry is not the city's.
    s.buildings[b2].queue.items.clear();
    s.buildings[b].queue.push_disband();
    assert!(
        s.disband_castable(b2, false),
        "the Barracks' entry is not counted"
    );
    assert!(s.disband_castable(b2, true));
}

#[test]
fn university_admission_counts_seated_and_queued_scholars_before_charging() {
    let (mut s, b, rec) = barracks(true);
    let bt = s.buildings[b].ty.unwrap();
    s.tables.build_types[bt].ident = Ident::University;
    s.tables.unit_types[rec].worker = crate::orders::Worker::Scholar;
    for o in 0..6 {
        let mut u = crate::Unit::new(0, o, s.buildings[b].pos, 100);
        u.ty = Some(rec);
        u.inside = Some(b);
        u.on_map = false;
        let u = s.add_unit(u);
        s.buildings[b].garrison.push(u);
    }
    assert_eq!(s.num_gatherers(b, false, false), 6);
    assert_eq!(s.queue_up(b, rec), Ok(0), "six permits the seventh");
    let bucket = s.ledgers[0].bucket;
    let tracked = s.muster[0].queued_by_type[rec];
    assert_eq!(s.queue_up(b, rec), Err(super::QueueFail::Full));
    assert_eq!(
        s.ledgers[0].bucket, bucket,
        "a refused eighth costs nothing"
    );
    assert_eq!(s.muster[0].queued_by_type[rec], tracked);
    assert_eq!(s.buildings[b].queue.items.len(), 1);
    // The special limit belongs to this producer and this worker kind.
    let citizen = s.add_unit_type(crate::UnitType {
        worker: crate::orders::Worker::Citizen,
        ..crate::UnitType::default()
    });
    assert!(s.unit_queue_has_room(b, citizen));
    s.tables.build_types[bt].ident = Ident::Barracks;
    assert!(s.unit_queue_has_room(b, rec));
    s.buildings[b].queue.capacity = s.buildings[b].queue.items.len();
    assert!(
        !s.unit_queue_has_room(b, rec),
        "ordinary queue capacity still applies"
    );
}

/// **The button needs a train job** (`BuildData::can_infinite@
/// 0062d4d0`, under the emulator): nothing on an empty queue or a
/// research entry, a toggle on a train job, and the queue's emptying
/// clears it. Made to fail with the gate answering for any queue.
#[test]
fn the_infinite_button_needs_a_train_job() {
    let (mut s, b, rec) = barracks(false);
    assert_eq!(s.action_buildmask(&[b], 0x40), 0, "an empty queue");
    s.queue_up(b, rec).expect("a research entry");
    assert_eq!(s.action_buildmask(&[b], 0x40), 0, "a research entry");
    let (mut s, b, rec) = barracks(true);
    s.queue_up(b, rec).expect("a train job");
    assert_eq!(s.action_buildmask(&[b], 0x40), 1);
    assert!(s.buildings[b].queue.infinite, "4096 -> 4160");
    assert_eq!(s.action_buildmask(&[b], 0x40), 1);
    assert!(!s.buildings[b].queue.infinite, "4160 -> 4096");
    s.action_buildmask(&[b], 0x40);
    let mut ledger = crate::economy::Ledger::default();
    s.buildings[b].queue.unqueue(0, false, &mut ledger);
    assert!(!s.buildings[b].queue.infinite, "the empty queue clears it");
}

/// **A train job finished under the bit re-queues itself, paid again,
/// until the stockpile refuses** (`do_queue`'s `61ec24`, run285's 1060
/// and 1272). Made to fail with the bit read after the unqueue, and
/// with the re-queue skipped.
#[test]
fn a_finished_train_job_requeues_under_the_bit() {
    let (mut s, b, rec) = barracks(true);
    s.queue_up(b, rec).expect("queued");
    s.action_buildmask(&[b], 0x40);
    let finish = |s: &mut Sim| {
        let n = s.buildings[b].queue.items.len();
        for _ in 0..100_000 {
            let trained = !s.process_queues().is_empty();
            if trained || s.buildings[b].queue.items.len() != n {
                return trained;
            }
        }
        panic!("the entry never finished");
    };
    assert!(finish(&mut s), "the first trains");
    let q = &s.buildings[b].queue;
    assert_eq!(q.items.len(), 1, "re-queued");
    assert_eq!(q.items[0].job_counter, 0);
    assert!(q.infinite, "the bit survives the empty queue's clear");
    assert_ne!(q.items[0].cost, [0; super::PAIRS], "paid again");
    // A stockpile short of the next price in every resource it costs.
    let price = s.price_of(0, rec);
    for (r, p) in price.iter().enumerate() {
        if *p > 0 {
            s.ledgers[0].bucket[r] = p - 1;
        }
    }
    assert!(finish(&mut s), "the second trains");
    let q = &s.buildings[b].queue;
    assert!(q.items.is_empty(), "the re-queue is refused");
    assert!(!q.infinite, "and the bit goes with it");
}

/// **The player's cancel reads its selector** (`Build::action_unqueue@
/// 00620280`, under the emulator on a synthesized Barracks, item 884):
/// a slot in a run removes the run's last and keeps the head's
/// progress, refunding what that entry recorded; a single cancel on an
/// infinite queue only clears the bit; −1 removes the last slot; −2
/// clears the bit *and* removes; −5 removes five from the end; −10
/// empties; an empty queue keeps its bit. Made to fail with the bit's
/// early return dropped, and with a negative `p` read as slot 0.
#[test]
fn the_player_s_cancel_reads_its_selector() {
    let (mut s, b, rec) = barracks(true);
    for _ in 0..3 {
        s.queue_up(b, rec).expect("queued");
    }
    s.buildings[b].queue.items[0].job_counter = 5000;
    let wealth = crate::economy::Resource::Wealth.index();
    let paid = |s: &Sim, i: usize| i32::from(s.buildings[b].queue.items[i].cost[0]);
    let (before, second) = (s.ledgers[0].bucket[wealth], paid(&s, 1));
    assert_eq!(s.action_unqueue(b, 0), 1);
    let q = &s.buildings[b].queue;
    assert_eq!(q.items.len(), 2, "one of the run");
    assert_eq!(q.items[0].job_counter, 5000, "the head keeps its progress");
    assert_eq!(
        s.ledgers[0].bucket[wealth],
        before + second,
        "the recorded price"
    );
    s.action_buildmask(&[b], 0x40);
    assert!(s.buildings[b].queue.infinite);
    let before = s.ledgers[0].bucket[wealth];
    assert_eq!(s.action_unqueue(b, -1), 0, "a single cancel on 4160");
    assert!(!s.buildings[b].queue.infinite, "4160 -> 4096");
    assert_eq!(s.buildings[b].queue.items.len(), 2, "and nothing removed");
    assert_eq!(s.ledgers[0].bucket[wealth], before, "nor refunded");
    assert_eq!(s.action_unqueue(b, -1), 1, "the bit off: the last goes");
    assert_eq!(
        s.buildings[b].queue.items[0].job_counter, 5000,
        "the head stays"
    );
    s.queue_up(b, rec).expect("queued");
    s.action_buildmask(&[b], 0x40);
    assert_eq!(s.action_unqueue(b, -2), 1, "-2 clears the bit and removes");
    assert!(!s.buildings[b].queue.infinite);
    for _ in 0..6 {
        s.queue_up(b, rec).expect("queued");
    }
    assert_eq!(s.action_unqueue(b, -5), 5, "five from the end");
    assert_eq!(s.buildings[b].queue.items.len(), 2);
    assert_eq!(s.buildings[b].queue.items[0].job_counter, 5000);
    assert_eq!(s.action_unqueue(b, 7), 0, "a slot past the end");
    assert_eq!(s.action_unqueue(b, -10), 2, "all");
    assert!(s.buildings[b].queue.items.is_empty());
    s.buildings[b].queue.infinite = true;
    assert_eq!(s.action_unqueue(b, -1), 0);
    assert!(
        s.buildings[b].queue.infinite,
        "an empty queue keeps its bit"
    );
    // Two types: −1 is the last slot, not slot 0's run.
    s.buildings[b].queue.infinite = false;
    let mut other = crate::UnitType::default();
    other.price.pop = 1;
    other.price.base[crate::economy::Resource::Food.index()] = 30;
    let rec2 = s.add_unit_type(other);
    s.muster[0].researched[rec2] = true;
    s.queue_up(b, rec).expect("queued");
    s.queue_up(b, rec2).expect("queued");
    s.buildings[b].queue.items[0].job_counter = 700;
    assert_eq!(s.action_unqueue(b, -1), 1);
    let q = &s.buildings[b].queue;
    assert_eq!(
        (q.items.len(), q.items[0].ty, q.items[0].job_counter),
        (1, rec, 700)
    );
}

/// **A queue order on a mixed group** (item 3966, `docs/GOLDEN.md`
/// §172, T71-14): `Group::action_queue_up@006fdbb0`'s train arm offers
/// `Build::queue_up(ty, 1)` to every member and leaves the type to
/// `could_queue` → `BuildData::can_make@0062db10`, whose
/// `BuildTypeData::queue_here` gate (`:31`–`:36`) binds a unit too. A
/// Barracks and a Stable pressed for a Barracks unit lay one entry, at
/// the Barracks; pressed for a Stable unit twice, two at the Stable.
#[test]
fn a_press_on_a_barracks_and_a_stable_queues_only_where_the_type_is_made() {
    let (mut s, barracks, bowmen) = barracks(true);
    // A second tree: the first is finalized inside the helper.
    let mut tree = tech::TechTree::new();
    for n in ["Food", "Timber", "Wealth", "Knowledge", "Metal", "Oil"] {
        tree.add(tech::TypeDef::good(n));
    }
    let bow = tree.add(tech::TypeDef::unit("Bowmen", tech::UnitTraits::default()));
    let bar = tree.add(tech::TypeDef::building("Barracks"));
    let horse = tree.add(tech::TypeDef::unit(
        "Light Horse",
        tech::UnitTraits::default(),
    ));
    let stab = tree.add(tech::TypeDef::building("Stable"));
    tree.types[bow].where_ = Some(bar);
    tree.types[horse].where_ = Some(stab);
    tree.finalize();
    s.set_tech_tree(tree);
    for t in [bow, bar, horse, stab] {
        s.tech[0].tech[t] = true;
    }
    let bt = s.buildings[barracks].ty.unwrap();
    s.tables.build_types[bt].tree = Some(bar);
    s.tables.unit_types[bowmen].tree = Some(bow);
    // The tree's install clears the availability bits.
    s.muster[0].researched[bowmen] = true;
    let srec = s.tables.build_types.len();
    s.tables.build_types.push(BuildType {
        ident: Ident::Stable,
        tree: Some(stab),
        ..s.tables.build_types[bt].clone()
    });
    let stable = s.add_building(0, crate::Pos::new(10 * 256, 10 * 256), 20);
    s.buildings[stable].ty = Some(srec);
    let mut lh = s.tables.unit_types[bowmen].clone();
    lh.tree = Some(horse);
    let lhrec = s.add_unit_type(lh);
    s.muster[0].researched[lhrec] = true;

    assert_eq!(s.action_queue_up(&[barracks, stable], bowmen, 1), 1);
    let len = |s: &Sim, b: usize| s.buildings[b].queue.items.len();
    assert_eq!((len(&s, barracks), len(&s, stable)), (1, 0));
    // Least queued first: the Stable is offered first, and the Barracks
    // refuses both passes.
    assert_eq!(s.action_queue_up(&[barracks, stable], lhrec, 2), 2);
    assert_eq!((len(&s, barracks), len(&s, stable)), (1, 2));
}
