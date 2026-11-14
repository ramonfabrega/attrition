use super::*;

fn tuning() -> Tuning {
    Tuning::RON
}

fn charges(pairs: &[(Resource, i32)]) -> [i32; RESOURCES] {
    let mut out = [0; RESOURCES];
    for &(r, n) in pairs {
        out[r.index()] = n;
    }
    out
}

#[test]
fn the_accelerators_all_ship_as_one_frame_per_call() {
    let t = tuning();
    for job in [Job::Construct, Job::Train, Job::Research] {
        assert_eq!(accel(&t, job, 1), TIME_SCALE);
    }
}

#[test]
fn ai_speed_multiplies_the_accelerator_only_above_one() {
    let t = tuning();
    assert_eq!(accel(&t, Job::Train, 0), TIME_SCALE);
    assert_eq!(accel(&t, Job::Train, 1), TIME_SCALE);
    assert_eq!(accel(&t, Job::Train, 4), TIME_SCALE * 4);
}

#[test]
fn a_citizen_takes_four_seconds_before_any_are_owned() {
    let t = tuning();
    // JOB_TIME 50 frames, UNIT_RATE_BASE 6/5 loaded as 120.
    let times = Times {
        job_time: 50,
        research_premium_time: 512,
        job_extra_time: 10,
    };
    let time = train_time(&t, &times, true, 0, &[]);
    assert_eq!(time, 6000);
    assert_eq!(time / TIME_SCALE, 60);
    assert_eq!(time / TIME_SCALE / crate::FRAMES_PER_SECOND, 4);
}

#[test]
fn a_base_shortened_before_the_ramp_scales_through_it() {
    // The Uranium arm (`train_time@006508c0:79`–`85`): an ICBM's 600 frames
    // of base become 54545 hundredths, then the ramp's 120 percent — the
    // original's 65454 on Great Sahara's `1/2017` (item 1697, AI §184).
    let t = tuning();
    let times = Times {
        job_time: 600,
        research_premium_time: 512,
        job_extra_time: 0,
    };
    assert_eq!(train_time_from(&t, &times, true, 0, 0, &[]), 72000);
    assert_eq!(train_time_from(&t, &times, true, 0, 10, &[]), 65454);
}

#[test]
fn each_citizen_already_owned_adds_half_a_second() {
    let t = tuning();
    let times = Times {
        job_time: 50,
        research_premium_time: 512,
        job_extra_time: 10,
    };
    // JOB_EXTRA_TIME 10 times UNIT_RATE_PROGRESSION 75 is 750 hundredths.
    let none = train_time(&t, &times, true, 0, &[]);
    let one = train_time(&t, &times, true, 1, &[]);
    assert_eq!(one - none, 750);
    // Seven and a half frames. The comparison is in hundredths because
    // that is where the half is; dividing to frames first would lose it.
    assert_eq!((one - none) * 2, crate::FRAMES_PER_SECOND * TIME_SCALE);
}

#[test]
fn the_time_ramp_triples_and_stops() {
    let t = tuning();
    let times = Times {
        job_time: 50,
        research_premium_time: 512,
        job_extra_time: 10,
    };
    let base = train_time(&t, &times, true, 0, &[]);
    assert_eq!(train_time(&t, &times, true, 16, &[]), base * RAMP_CEILING);
    assert_eq!(train_time(&t, &times, true, 200, &[]), base * RAMP_CEILING);
}

#[test]
fn the_first_of_a_type_is_a_research_job_and_takes_twice_as_long() {
    let t = tuning();
    let times = Times {
        job_time: 50,
        research_premium_time: 512, // the `2` that 356 records carry
        job_extra_time: 10,
    };
    // Research skips the ramp, so compare against the unscaled base.
    assert_eq!(train_time(&t, &times, false, 0, &[]), 50 * TIME_SCALE * 2);
    // And it advances at the research accelerator, by the same bit.
    assert_eq!(Job::for_unit(false), Job::Research);
    assert_eq!(Job::for_unit(true), Job::Train);
}

#[test]
fn an_entry_records_only_three_of_six_resources() {
    let c = charges(&[
        (Resource::Food, 10),
        (Resource::Timber, 20),
        (Resource::Wealth, 30),
        (Resource::Knowledge, 40),
    ]);
    let item = Item::queued(7, &c);
    let paid: Vec<_> = item.paid().collect();
    assert_eq!(
        paid,
        vec![
            (Resource::Food, 10),
            (Resource::Timber, 20),
            (Resource::Wealth, 30)
        ]
    );
    assert_eq!(item.good[2], Resource::Wealth.index() as i16);
}

#[test]
fn zero_amounts_do_not_consume_a_pair() {
    let c = charges(&[
        (Resource::Food, 0),
        (Resource::Metal, 5),
        (Resource::Oil, 7),
    ]);
    let item = Item::queued(7, &c);
    let paid: Vec<_> = item.paid().collect();
    assert_eq!(paid, vec![(Resource::Metal, 5), (Resource::Oil, 7)]);
    assert_eq!(item.good[2], NO_GOOD);
}

#[test]
fn done_is_read_before_the_increment() {
    let mut item = Item::default();
    // A target of two frames, one frame per call.
    let target = 200;
    assert!(!advance(&mut item, target, 100));
    assert_eq!(item.job_counter, 100);
    assert!(!advance(&mut item, target, 100));
    assert_eq!(item.job_counter, 200);
    // The counter is at the target, and only now does a call see it.
    assert!(advance(&mut item, target, 100));
}

#[test]
fn the_counter_never_passes_the_target() {
    let mut item = Item::default();
    advance(&mut item, 150, 100);
    advance(&mut item, 150, 100);
    assert_eq!(item.job_counter, 150);
}

#[test]
fn a_target_of_one_completes_on_the_first_call() {
    let mut item = Item::default();
    assert!(advance(&mut item, MIN_TIME, 100));
}

#[test]
fn the_population_cap_stops_the_handover_not_the_clock() {
    // Progress runs to completion at the train accelerator regardless.
    let t = tuning();
    let times = Times {
        job_time: 1,
        research_premium_time: 512,
        job_extra_time: 10,
    };
    let target = train_time(&t, &times, true, 0, &[]);
    let mut item = Item::default();
    let accel = accel(&t, Job::Train, 1);
    while !advance(&mut item, target, accel) {}
    assert_eq!(item.job_counter, target);
    // And then stalls, at full progress, forever.
    assert_eq!(hand_over(10, 10, 1, false), Handover::Population);
    assert_eq!(hand_over(10, 9, 1, false), Handover::Trained);
}

#[test]
fn a_limit_refuses_differently_from_the_cap() {
    assert_eq!(hand_over(10, 0, 1, true), Handover::Limit);
    // The cap is checked first, so it wins when both apply.
    assert_eq!(hand_over(0, 0, 1, true), Handover::Population);
}

#[test]
fn cancelling_a_run_removes_the_last_of_it() {
    let mut q = Queue::new(8);
    let c = charges(&[(Resource::Food, 10)]);
    for _ in 0..3 {
        q.push(4, &c);
    }
    q.push(9, &c);
    // The one in progress is slot 0 and keeps its progress.
    q.items[0].job_counter = 4200;
    let mut ledger = Ledger::default();
    q.unqueue(0, true, &mut ledger);
    assert_eq!(q.items.len(), 3);
    assert_eq!(q.items[0].job_counter, 4200);
    assert_eq!(q.items.iter().map(|i| i.ty).collect::<Vec<_>>(), [4, 4, 9]);
}

#[test]
fn cancelling_refunds_the_recorded_price_and_completing_does_not() {
    let mut q = Queue::new(8);
    let c = charges(&[(Resource::Food, 30), (Resource::Timber, 12)]);
    q.push(4, &c);
    q.push(4, &c);

    let mut ledger = Ledger::default();
    q.unqueue(0, true, &mut ledger);
    assert_eq!(ledger.bucket[Resource::Food.index()], 30);
    assert_eq!(ledger.bucket[Resource::Timber.index()], 12);

    q.unqueue(0, false, &mut ledger);
    assert_eq!(ledger.bucket[Resource::Food.index()], 30);
    assert_eq!(ledger.bucket[Resource::Timber.index()], 12);
    assert!(q.items.is_empty());
}

#[test]
fn science_reprices_a_queued_item_in_place() {
    let t = tuning();
    let mut ledger = Ledger::default();
    let c = charges(&[(Resource::Knowledge, 100)]);
    let mut item = Item::queued(4, &c);

    // First level: nothing was discounted yet, so the base is the price.
    let back = reprice(&mut item, t.tech_science_discount, 0, &mut ledger);
    assert_eq!(back, 10);
    assert_eq!(ledger.bucket[Resource::Knowledge.index()], 10);
    assert_eq!(item.cost[0], 90);

    // Second level re-prices from the new stored amount, not the old one.
    let back = reprice(&mut item, t.tech_science_discount, 1, &mut ledger);
    assert_eq!(item.cost[0], 80);
    assert_eq!(back, 10);
}

#[test]
fn the_science_speedup_is_run39_s_second_library_entry() {
    // run39's own clock. The AI queues Written Word and City State at
    // frame 2; both are `JOB_TIME 200`, so both start at 20,000
    // hundredths. Written Word is researched at Science 0 — its own level
    // is 0, and the gate is *strict* — so it takes the full 20,000 and
    // lands on 201. City State is then researched with `epoch[3]` at one
    // against its own level of zero, takes 18,000, and lands on 382. The
    // twenty frames between 382 and 402 were the whole of the twenty-one
    // fields `run39_s_build_queues_are_the_original_s_clock` disagreed on.
    let t = tuning();
    assert_eq!(
        science_speedup(&t, 0, 0),
        None,
        "level with it is not ahead"
    );
    let up = science_speedup(&t, 1, 0).expect("a level ahead");
    assert_eq!(up, Adjust::Off(10));
    assert_eq!(up.apply(20_000), 18_000);
    // Two levels ahead is twice off the *whole*, not a compounding.
    assert_eq!(science_speedup(&t, 2, 0).unwrap().apply(20_000), 16_000);
    // And being behind costs nothing in time, where on the price side the
    // same distance is a surcharge.
    assert_eq!(science_speedup(&t, 0, 3), None);
}

#[test]
fn taking_a_percentage_off_is_not_scaling_by_its_complement() {
    // `Adjust::Off(pct)` truncates the term and subtracts it;
    // `Adjust::Scale(100 - pct)` truncates the product. The original
    // spells the science speedup the first way — a magic multiply by
    // −0x51eb851f added to the time — so the difference is not cosmetic.
    // Seven hundredths of a frame at ten percent: 7 − 0 = 7, against
    // 630 / 100 = 6.
    assert_eq!(Adjust::Off(10).apply(7), 7);
    assert_eq!(Adjust::Scale(90).apply(7), 6);
    // They agree wherever the product is a round hundred.
    for time in [0, 100, 1_000, 20_000] {
        assert_eq!(Adjust::Off(10).apply(time), Adjust::Scale(90).apply(time));
    }
}

#[test]
fn the_magic_multiply_is_a_division_by_a_hundred() {
    for x in [0, 1, 99, 100, 250, 1000, 3200, 12345, 1_000_000] {
        assert_eq!(neg_hundredth(x), -(x / 100));
    }
}

#[test]
fn a_research_entry_completes_without_asking_the_cap() {
    // The bit is clear, the player is over the cap — and the answer is
    // still positive, because the cap check is inside the bit-set branch.
    assert_eq!(finished(false, 0, 10, 1, true), Handover::Researched);
    assert_eq!(finished(true, 0, 10, 1, true), Handover::Population);
    assert_eq!(finished(true, 10, 0, 1, false), Handover::Trained);
}

#[test]
fn a_stuck_head_lets_the_first_research_entry_through() {
    let mut q = Queue::new(8);
    let c = charges(&[(Resource::Food, 1)]);
    for ty in [1, 2, 3, 2, 4] {
        q.push(ty, &c);
    }
    let researched = [false, true, true, false, false];
    // Slot 0 is never a candidate, even if it is itself a research job.
    assert_eq!(q.next_research(|ty| !researched[ty]), Some(2));
    // Only train jobs behind the head: nothing gets through.
    assert_eq!(q.next_research(|_| false), None);
}

#[test]
fn libraries_decide_how_many_slots_advance() {
    assert_eq!(parallel_slots(1, 5), 1);
    assert_eq!(parallel_slots(4, 5), 4);
    assert_eq!(parallel_slots(4, 2), 2);
    assert_eq!(parallel_slots(0, 3), 0);
}

#[test]
fn a_queue_refuses_when_it_is_full() {
    let mut q = Queue::new(2);
    let c = charges(&[(Resource::Food, 1)]);
    assert!(q.has_room());
    q.push(1, &c);
    assert!(q.has_room());
    q.push(1, &c);
    assert!(!q.has_room());
}

/// **A vacated slot keeps the last entry's bytes** (item 4222):
/// `BuildQueue::un_queue@00630ad0` copies the entries above the removed one
/// down a slot and leaves the last slot as it was, and `set_queue` overwrites
/// it when an entry is laid there. The minted game's AI Town Center `1/2000`
/// on 100: three Citizens at 25, 26 and 27, the first finished, and slot 2
/// still reads the 27 (run1581).
#[test]
fn an_unqueued_slot_keeps_the_last_entry_s_bytes() {
    let food = |n| charges(&[(Resource::Food, n)]);
    let mut q = Queue::new(20);
    for n in [25, 26, 27] {
        q.push(0, &food(n));
    }
    let mut ledger = Ledger::default();
    q.unqueue(0, false, &mut ledger);
    let costs = |v: &[Item]| v.iter().map(|i| i.cost[0]).collect::<Vec<_>>();
    assert_eq!(costs(&q.items), [26, 27]);
    assert_eq!(costs(&q.vacated), [27], "slot 2 keeps the third entry's copy");
    // Removing the last entry leaves its own bytes, its counter zeroed
    // first (`Build::unqueue`): slot 1 now, slot 2 still.
    q.items[1].job_counter = 500;
    q.unqueue(1, false, &mut ledger);
    assert_eq!(costs(&q.items), [26]);
    assert_eq!(costs(&q.vacated), [27, 27]);
    assert_eq!(q.vacated[0].job_counter, 0, "the removed last entry's counter");
    // A lay overwrites the slot at `queued` and no other.
    q.push(0, &food(30));
    assert_eq!(costs(&q.items), [26, 30]);
    assert_eq!(costs(&q.vacated), [27]);
}
