//! The `CITY` record, and the census and prices it is read beside.

/// One field of a `CITY` record the two sides disagree on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CityDivergence {
    pub frame: i64,
    pub who: i64,
    /// The city building's object number — the identity the link is made
    /// on, and the one a `BUILDDATA` row shares.
    pub o: i64,
    /// The field, named as `CityData::log_data` writes it; an array element
    /// as `space[k]` / `ter[k]`.
    pub field: String,
    pub ours: i64,
    pub theirs: i64,
}

#[cfg(test)]
mod tests {

    use crate::diff::testkit::*;
    use crate::diff::*;

    use crate::testenv::{dump, install};

    /// `LEADERDATA who 1` (`LEADERS=9`) prints the encrypted goods block —
    /// `resources` is the assembled rate, in sixteenths — and it reads
    /// `[160, 160, 0, 0, 0, 0]`: `CITY_GATHER × 16` for food and timber and
    /// nothing from the five citizens, all on their chains and none yet
    /// arrived (`Unit::do_gather` sets the dirty flag on the same statement
    /// as `been_there`). By frame 60 it is `[640, 320, …]` — three farmers
    /// and one camp citizen arrived. The rate, not the cap: the AI is
    /// British and its cap carries `BRITISH_COMMERCE`.
    #[test]
    fn run8_s_frame_2_income_is_the_city_and_nothing_the_citizens_have_reached() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run8-personality.txt") else {
            eprintln!("skipping: no gamelog-run8-personality.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let mut built = build_sim(&loaded, &log.initial().unwrap(), Tuning::RON);
        for _ in 0..3 {
            built.sim.tick();
        }
        let theirs = log.leader_block(2, 1).expect("FRAME 2 LEADERDATA who 1");
        let rate: Vec<i64> = theirs
            .all("resources")
            .iter()
            .map(|v| v.trim().parse().unwrap_or(0))
            .collect();
        assert_eq!(&rate[..6], &[160, 160, 0, 0, 0, 0], "the dump's own rate");
        let ours: Vec<i64> = built.sim.ledgers[1]
            .rate
            .iter()
            .map(|&v| i64::from(v))
            .collect();
        assert_eq!(
            ours,
            rate[..6].to_vec(),
            "the AI's five citizens are on their chains and none has arrived: \
             CITY_GATHER x 16 and nothing else"
        );
    }

    /// **run59 — the census window East Indies' word has been asking for**
    /// (2026-09-02, item 154's second half).
    ///
    /// The word is 5376 and the frame is the AI's Market: `economic.bhs`
    /// case 15 calls `place_building_with_cost(who, "Market", my_capital)`
    /// on it, the original places it, and this crate cannot pay — a Market
    /// is eighty timber and the AI holds thirty-four. That is a resource
    /// level, and no capture on disk carried one past frame 800:
    /// `LEADERS=1` at `[End Frame]` is five scalars and no goods, and the
    /// only `LEADERS=9` windows were run40's `[560, 600)` and run41's
    /// `[770, 800)`, both on Great Lakes.
    ///
    /// run59 is that window moved: run58's recipe (`MAP_STYLE 18`, seed
    /// 12345, the profile's lobby, no input) with `LEADERS=9` at
    /// `[End Frame]` and the frame window `[5150, 5400)` — the dump is
    /// written only there, so the run costs minutes rather than the hour a
    /// per-frame one does.
    #[test]
    fn run59_s_census_is_where_the_ai_s_timber_goes() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run59-islands-census-5150.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run59.log"),
        ) else {
            eprintln!("skipping: no run59 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        assert_eq!(init.pasture.len(), 1, "run59's trace reached the setup");
        let mut built = build_sim(&loaded, &init, Tuning::RON);

        let mut compared = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        let mut steps: Vec<(i64, i64, i64, i64)> = Vec::new();
        // (who, field, good) keyed to (frames wrong, ours and theirs first).
        let mut shapes: std::collections::BTreeMap<(i64, String, usize), (usize, i64, i64)> =
            std::collections::BTreeMap::new();
        // The AI's timber shortfall, frame by frame.
        let mut timber_gap: Vec<(i64, i64)> = Vec::new();
        for n in 1..=5399 {
            built.tick();
            if n < 5150 {
                continue;
            }
            for who in 0..2i64 {
                let Some(block) = log.leader_block(n, who) else {
                    continue;
                };
                if who == 1 {
                    steps.push((
                        n,
                        block.int("production_step").unwrap_or(-1),
                        block.int("script_step").unwrap_or(-1),
                        block.int("prod_script_run").unwrap_or(-1),
                    ));
                }
                let field = |k: &str| -> Vec<i64> {
                    block
                        .all(k)
                        .iter()
                        .map(|v| v.trim().parse().unwrap_or(i64::MIN))
                        .collect()
                };
                let l = &built.sim.ledgers[who as usize];
                let rows: [(&str, [i32; sim::economy::RESOURCES]); 6] = [
                    ("bucket", l.bucket),
                    ("leftover", l.leftover),
                    ("resources", l.rate),
                    ("income", l.income),
                    ("resource_cap", l.cap),
                    ("gather_slots[scan]", l.gather_slots),
                ];
                for (key, ours) in rows {
                    let theirs = field(key);
                    if theirs.len() < sim::economy::RESOURCES {
                        continue;
                    }
                    for g in 0..sim::economy::RESOURCES {
                        compared += 1;
                        if who == 1 && key == "bucket" && g == 1 {
                            timber_gap.push((n, theirs[g] - i64::from(ours[g])));
                        }
                        if i64::from(ours[g]) != theirs[g] {
                            wrong.push(format!(
                                "frame {n} who {who} {key} good {g}: ours {} theirs {}",
                                ours[g], theirs[g]
                            ));
                            let e = shapes.entry((who, key.to_string(), g)).or_insert((
                                0,
                                i64::from(ours[g]),
                                theirs[g],
                            ));
                            e.0 += 1;
                        }
                    }
                }
            }
        }
        eprintln!("run59: {} of {compared} good-frames disagree", wrong.len());
        for (k, (n, o, t)) in &shapes {
            eprintln!("  {k:?}: {n} frames, ours {o} theirs {t} on the first");
        }
        assert_eq!(
            compared, 18_000,
            "250 frames, two players, six goods, six fields"
        );
        assert_eq!(
            wrong.len(),
            0,
            "the census's own count, and it is now **nothing**. run59 \
             measured 4,798; item 162 took 500 with the pre-placed camp's \
             six timber slots and 798 more with the wealth slot a dock \
             claims — which is the whole timber lineage, bucket and income \
             both — item 165 took 2,000 with `Leader::calc_gather` step 6, \
             the AI's Fisherman on its fish, and item 156 took the last \
             1,500: the starting grant of knowledge, metal and oil arrives \
             with the age and not at `Leader::init`, so neither player holds \
             a hundred of any of the three. Eighteen thousand good-frames, \
             two players, six goods, six fields, and every one of them is \
             the original's"
        );

        // **The record, and every shape in it is a standing state.** Each row
        // is (who, field, good) to (frames wrong, ours and theirs on 5150).
        // 250 is "every frame of the window", which is what says a shape is
        // a level rather than an event.
        let row = |who: i64, key: &str, g: usize| shapes.get(&(who, key.to_string(), g)).copied();
        let n = |who: i64, key: &str, g: usize| row(who, key, g).map_or(0, |(n, _, _)| n);

        // **The item, closed.** run59 measured the AI fifty timber short on
        // every one of these frames, with `leftover` agreeing on 234 of
        // them — a lump, banked before the window. run60 found the frame it
        // was banked on (sim-frame 4988, a goody box) and run42 the reason
        // the good was wrong (the dock's thirty wealth, 1,409 frames
        // earlier). Both landed 2026-09-02, and the AI's timber is now the
        // original's on every frame of this window — including 5377, where
        // it used to flip to thirty ahead because the original could pay
        // eighty for the Market and this crate could not.
        assert_eq!(n(1, "bucket", 1), 0, "the AI's timber, across the Market");
        assert!(
            timber_gap.iter().all(|&(_, d)| d == 0),
            "and the gap is zero on every frame: {:?}",
            timber_gap.iter().find(|&&(_, d)| d != 0)
        );

        // **And the slot count is no longer where it comes from.**
        // `gather_slots` is the running inventory `Build::activate` adds a
        // finished gather building's `gather_max` to, and the dump prints it
        // per good beside its own high-water mark. This census is what
        // caught the pre-placed camp claiming nothing — the human, which
        // builds nothing at all in this game, held 0 against 6 — and since
        // 2026-09-02 the harness hands a dump-stood camp its list *before*
        // `activate` surveys it, so **both players' timber slots are now
        // exact** and so are their food slots.
        assert_eq!(
            n(0, "gather_slots[scan]", 1),
            0,
            "the human's starting camp"
        );
        assert_eq!(n(1, "gather_slots[scan]", 1), 0, "the AI's six plus four");
        assert_eq!(n(0, "gather_slots[scan]", 0), 0, "the human's farms");
        assert_eq!(n(1, "gather_slots[scan]", 0), 0, "and the AI's");
        // The one wealth slot was item 82's, and it is closed too:
        // `BuildTypeData::get_good`'s table indeed cannot produce a
        // wealth-gathering building, because that is not where the slot
        // comes from — `Build::activate` line 590 gives one to a dock, a
        // market or a temple, off the same array (`Sim::claim_commerce_slot`).
        assert_eq!(n(0, "gather_slots[scan]", 2), 0, "the human's market");
        assert_eq!(n(1, "gather_slots[scan]", 2), 0, "the AI's dock");
        // **And fixing the timber slots moved the timber not at all**,
        // which is the measurement that refuted the fifty's first
        // explanation. The bonus is `TIMBER_BONUS_PER_WOOD_SLOT` per slot
        // *past the high-water mark*, and `Build::activate` pays nothing at
        // frame 0: the six arrive during setup, raise the mark to six
        // unpaid, and the one camp the AI builds in the run (frame 2424,
        // four slots) then pays 6 → 10 where it used to pay 0 → 4. Twenty
        // timber either way. The human was the control that said so from
        // the other side — six slots short and its timber bucket exact on
        // all 250 frames.

        // **The two rate seams are closed, and they were one thing.** The
        // AI's food used to be ten short in sixteenths and its wealth
        // missing entirely — `160` and `160` exactly, which is `10 × 16`
        // twice, and `10` is both of Fish's `BONUS_NUM`s in
        // `resourcerules.xml`. `Leader::calc_gather` **step 6** is what
        // pays them: the AI's `1/14` has been standing on its fish since
        // frame 4992 and nothing here walked the idle fishermen
        // (`crates/sim/src/rares.rs`, item 165). Every one of both players'
        // six rates and six incomes is now the original's on every frame of
        // the window.
        for g in 0..sim::economy::RESOURCES {
            assert_eq!(n(1, "income", g), 0, "the AI's income, good {g}");
            assert_eq!(n(1, "resources", g), 0, "the AI's rate, good {g}");
            assert_eq!(n(0, "income", g), 0, "the human's income, good {g}");
            assert_eq!(n(0, "resources", g), 0, "the human's rate, good {g}");
            assert_eq!(n(0, "leftover", g), 0, "the human's leftover, good {g}");
        }
        // **And the hundred in goods 3, 4 and 5 was item 156, and it is
        // gone.** Each of the three read `(250, 100, 0)` — every frame of
        // the window, both players, a hundred here against the original's
        // nothing. `STARTING_GOODS` arrives **with the good**: `Leader::init`
        // pays only what `type_avail` already holds and `Leader::gain_tech`
        // pays the rest as their age lands, so in the Ancient age a leader
        // holds no knowledge, no metal and no oil
        // ([`sim::Sim::lay_starting_goods`]; `docs/COSTS.md`, "The starting
        // grant arrives with the good"). This is 1,500 of the 1,500 rows
        // above, which is why the count is zero.
        for g in 3..sim::economy::RESOURCES {
            assert_eq!(row(0, "bucket", g), None, "the human's good {g}");
            assert_eq!(row(1, "bucket", g), None, "the AI's good {g}");
        }

        // **The step machine, at the other end of the same record**
        // (`docs/AI.md` §25). run58 stops at 5,201 and could not show this:
        // the AI's script is still live 5,400 frames in, the machine is
        // armed on its phase frames and cleared again, and `script_step`
        // moves 23, 15, 18 as `economic.bhs` walks its cases.
        assert!(
            steps.iter().all(|&(_, step, _, run)| step <= 1 && run == 1),
            "the ladder never leaves step 1 while the script lives: {:?}",
            steps
                .iter()
                .find(|&&(_, step, _, run)| step > 1 || run != 1)
        );
        let script_steps: Vec<i64> = {
            let mut v: Vec<i64> = steps.iter().map(|&(_, _, s, _)| s).collect();
            v.dedup();
            v
        };
        assert_eq!(script_steps, vec![23, 15, 18], "economic.bhs's cases");

        // **And the positions, which this capture carried for a day before
        // anything compared them** (item 87's ledger, 2026-09-02). run59 is
        // a `UNITS=3` window like run58's frames and run63's, so every
        // dumped unit's point is in it; the census read six goods a leader
        // and left the other nine tenths of the file alone.
        //
        // What the widening found on its first run was the item the queue
        // had booked a *capture* for: `1/18`, the AI's Transport Barge, is
        // born on **5342** — inside this window, 88 frames before run63's
        // opens — and this crate bore it two tiles closer to its
        // destination, which is the whole of the "five frames ahead" run63
        // measured at 5430. `docs/TRANSPORT.md` §6.1: the spot search's
        // `(-1, -1)` form takes `find_unit_with_radius`, not the pairwise
        // pair, and the pairwise pair was refusing the bearing the original
        // takes because the caster stands three cells and four cells away.
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        // **The window, and not the shutdown block after it.** The file's
        // last block is the quit dump — `GameInfo closing` is the next line
        // — and its header is a frame ahead of the state it holds: the
        // blocks run 5150…5399, then this one at 5401, and 5400 is never
        // written. So every unit still walking reads one step behind in it,
        // which is a property of the capture and not of the simulation.
        let tail = report.frames.last().map_or(0, |f| f.frame);
        assert_eq!(tail, 5401, "run59's last block is the shutdown dump");
        assert!(
            !report.frames.iter().any(|f| f.frame == 5400),
            "…and 5400 is the frame it stands in for"
        );
        let parted: Vec<(i64, i64, i64)> = report
            .first_divergence_by_unit()
            .into_iter()
            .filter(|&(_, _, f)| f < tail)
            .collect();
        let unit_frames: usize = report.frames.iter().map(|f| f.compared).sum();
        eprintln!(
            "run59: {unit_frames} unit-frames over {} frames, {} ever off position",
            report.frames.len(),
            parted.len()
        );
        assert!(
            unit_frames > 5_000,
            "run59's window is a UNITS=3 one — {unit_frames} points is a wrong file"
        );
        assert_eq!(
            parted,
            vec![],
            "the census window stands where the original's does too"
        );
    }

    /// One census's disagreements, folded: `(who, field, good)` to the
    /// number of frames it was wrong on, the first such frame, and what the
    /// two sides read there. A shape wrong on *every* frame of a window is a
    /// standing level; one that starts partway through names its own event.
    type Shapes = std::collections::BTreeMap<(i64, String, usize), (usize, i64, i64, i64)>;

    /// **run42's nine hundred frames, and where the fifty is not**
    /// (item 162, 2026-09-02).
    ///
    /// run59's census measured the AI fifty timber short at frame 5,150 and
    /// could not say when the fifty was banked: nothing on disk carried a
    /// resource level between frame 800 and 5,150. Nothing *had to* be
    /// captured to narrow it, though — **run42 was already on disk**. It is
    /// run39's game (`samegame.py --exclude LEADERDATA`: 900 frames in
    /// common, not one differing) at run39's detail plus `LEADERS=2`, which
    /// is the detail `LeaderData::log_data` announces the encrypted block
    /// at: `bucket`, `leftover`, `resources`, `income`, `rate` and
    /// `resource_cap`, per good, on every one of its 900 frames. run39's
    /// game is run54's is run58's is run59's, so this is the same AI walking
    /// the same script — the census before the word, for the cost of a
    /// parser.
    ///
    /// Five fields, six goods, two players, 900 frames.
    #[test]
    fn run42_s_nine_hundred_frames_are_the_census_before_the_word() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run42-islands-goodybucket.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run42.log"),
        ) else {
            eprintln!("skipping: no run42 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);

        let mut compared = 0usize;
        let mut shapes = Shapes::new();
        // The AI's timber, frame by frame, so the *step* can be found rather
        // than the level.
        let mut timber: Vec<(i64, i64, i64)> = Vec::new();
        for n in 1..=900 {
            built.tick();
            for who in 0..2i64 {
                let Some(block) = log.leader_block(n, who) else {
                    continue;
                };
                let field = |k: &str| -> Vec<i64> {
                    block
                        .all(k)
                        .iter()
                        .map(|v| v.trim().parse().unwrap_or(i64::MIN))
                        .collect()
                };
                let l = &built.sim.ledgers[who as usize];
                let rows: [(&str, [i32; sim::economy::RESOURCES]); 5] = [
                    ("bucket", l.bucket),
                    ("leftover", l.leftover),
                    ("resources", l.rate),
                    ("income", l.income),
                    ("resource_cap", l.cap),
                ];
                for (key, ours) in rows {
                    let theirs = field(key);
                    if theirs.len() < sim::economy::RESOURCES {
                        continue;
                    }
                    for g in 0..sim::economy::RESOURCES {
                        compared += 1;
                        if who == 1 && key == "bucket" && g == 1 {
                            timber.push((n, i64::from(ours[g]), theirs[g]));
                        }
                        if i64::from(ours[g]) != theirs[g] {
                            let e = shapes.entry((who, key.to_string(), g)).or_insert((
                                0,
                                n,
                                i64::from(ours[g]),
                                theirs[g],
                            ));
                            e.0 += 1;
                        }
                    }
                }
            }
        }
        for (k, (n, f, o, t)) in &shapes {
            eprintln!("  {k:?}: {n} frames, from {f}: ours {o} theirs {t}");
        }
        // The AI's timber gap, and every frame it changes on.
        let mut steps: Vec<(i64, i64)> = Vec::new();
        let mut last = 0i64;
        for &(n, o, t) in &timber {
            let d = t - o;
            if d != last {
                steps.push((n, d));
                last = d;
            }
        }
        eprintln!("run42: the AI's timber gap steps at {steps:?}");
        assert_eq!(
            compared, 54_000,
            "900 frames, two players, six goods, five fields"
        );
    }

    /// **run60 — the whole timber curve, and the frame the fifty is banked
    /// on** (item 162, 2026-09-02).
    ///
    /// run42 pins frames 1–900 of this game exact and run59 measures a
    /// standing fifty at 5,150; between them nothing on disk carried a
    /// resource level at all. run60 closes that: run58's game with
    /// `[End Frame]` cut to `MISC,LEADERS=2` — the detail
    /// `LeaderData::log_data` announces the encrypted block at, and nothing
    /// else — so every one of its 5,400 frames prints `bucket`, `leftover`,
    /// `resources`, `income`, `rate` and `resource_cap` per good and the run
    /// costs minutes rather than the hour a full-detail one does.
    ///
    /// Five fields, six goods, two players, 5,400 frames.
    #[test]
    fn run60_s_whole_curve_is_where_the_ai_s_timber_parts() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run60-islands-census-thin.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run60.log"),
        ) else {
            eprintln!("skipping: no run60 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);

        let mut compared = 0usize;
        let mut shapes = Shapes::new();
        // Every good's gap, frame by frame, so the *step* can be found —
        // and the AI's own bucket beside it, so a step can be read as *this
        // crate spent* or *the original was paid*.
        let mut gaps: Vec<(
            i64,
            [i64; sim::economy::RESOURCES],
            [i32; sim::economy::RESOURCES],
        )> = Vec::new();
        for n in 1..=5400 {
            built.tick();
            let mut row = [0i64; sim::economy::RESOURCES];
            for who in 0..2i64 {
                let Some(block) = log.leader_block(n, who) else {
                    continue;
                };
                let field = |k: &str| -> Vec<i64> {
                    block
                        .all(k)
                        .iter()
                        .map(|v| v.trim().parse().unwrap_or(i64::MIN))
                        .collect()
                };
                let l = &built.sim.ledgers[who as usize];
                let rows: [(&str, [i32; sim::economy::RESOURCES]); 5] = [
                    ("bucket", l.bucket),
                    ("leftover", l.leftover),
                    ("resources", l.rate),
                    ("income", l.income),
                    ("resource_cap", l.cap),
                ];
                for (key, ours) in rows {
                    let theirs = field(key);
                    if theirs.len() < sim::economy::RESOURCES {
                        continue;
                    }
                    for g in 0..sim::economy::RESOURCES {
                        compared += 1;
                        if who == 1 && key == "bucket" {
                            row[g] = theirs[g] - i64::from(ours[g]);
                        }
                        if i64::from(ours[g]) != theirs[g] {
                            let e = shapes.entry((who, key.to_string(), g)).or_insert((
                                0,
                                n,
                                i64::from(ours[g]),
                                theirs[g],
                            ));
                            e.0 += 1;
                        }
                    }
                }
            }
            gaps.push((n, row, built.sim.ledgers[1].bucket));
        }
        eprintln!("run60: {compared} good-frames compared");
        for (k, (n, f, o, t)) in &shapes {
            eprintln!("  {k:?}: {n} frames, from {f}: ours {o} theirs {t}");
        }
        let mut last = [0i64; sim::economy::RESOURCES];
        for &(n, row, mine) in &gaps {
            if row != last {
                eprintln!("  f{n}: the AI's bucket gap {row:?}, ours {mine:?}");
                last = row;
            }
        }
    }

    /// **run64's frame-6167 world-path prices**, the caravan's own search,
    /// against the original's proxied `calc_cost`.
    ///
    /// The sibling of [`run55_s_frame_1477_prices_are_the_originals`] on
    /// the other map and the other search, and the reason it exists is the
    /// danger map: eleven of the forty-seven steps the original priced here
    /// were 8, 10 or 16 too dear in this crate, every one of them a cell
    /// whose half-cell carries a negative danger, and the expansion order
    /// parted on the seventh pop because of it.
    #[test]
    fn run64_s_frame_6167_prices_are_the_originals() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(t64)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            trace("rontrace-run64.log"),
        ) else {
            eprintln!("skipping: no run54/run64 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let theirs = t64.calls_in(6167, crate::trace::call_site::CALC_COST);
        assert!(
            theirs.len() >= 40,
            "run64's callwin covers 6167; it priced {} steps there",
            theirs.len()
        );
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        for _ in 0..6167 {
            built.tick();
        }
        built.sim.trace_costs = true;
        built.tick();
        let ours = std::mem::take(&mut built.sim.cost_marks);

        let mut theirs_by_key: std::collections::BTreeMap<sim::path::CostKey, i32> =
            std::collections::BTreeMap::new();
        for c in &theirs {
            theirs_by_key.insert(c.cost_key().expect("a calc_cost call"), c.ret);
        }
        let mut shared = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        for m in &ours {
            let Some(&t) = theirs_by_key.get(&m.key()) else {
                continue;
            };
            shared += 1;
            if t != m.cost {
                wrong.push(format!(
                    "({},{}) -> ({},{}) dir {} depth {}: ours {} theirs {t}",
                    m.from.0 / m.step,
                    m.from.1 / m.step,
                    m.to.0 / m.step,
                    m.to.1 / m.step,
                    m.dir,
                    m.depth,
                    m.cost,
                ));
            }
        }
        assert!(
            shared >= 40,
            "frame 6167: {} of the original's steps and {} of ours, {shared} shared \
             — the searches did not start from the same place",
            theirs.len(),
            ours.len()
        );
        assert!(
            wrong.is_empty(),
            "frame 6167: {} of {shared} shared steps priced differently:\n  {}",
            wrong.len(),
            wrong.join("\n  ")
        );
    }

    /// **run40 and run41 — the leader census over a window, and what the
    /// AI's second city actually costs.**
    ///
    /// run33 carries `LEADERS=1` at `[End Frame]`: five scalars, and no
    /// resources. A `LEADERS=9` record is the census oracle
    /// (`docs/ORACLE.md`) but it is ~10k lines a leader, so it is a *window*
    /// setting — `tools/gamelog/censuswindow.sh`, run10's game and lobby
    /// with the frame window moved. run40 is `[560, 600)` and run41 is
    /// `[770, 800)`, the two frames on which
    /// `ScenarioFuncSet::place_city_with_cost` is reached in the whole
    /// 1,850. Both traces' words are run33's draw for draw where they
    /// overlap, so all three are the same game.
    ///
    /// What they measure is a **price**. The AI's `defensive.bhs` step 11
    /// calls `city_placement` once per turn of its `num_loops = 5` loop, and
    /// `place_city_with_cost` returns −1 without a draw once
    /// `city_limit <= total_cities`. So the *number of draws* on those two
    /// frames says whether the city was bought:
    ///
    /// - **576**: sixty draws, five whole blocks — not bought. The census
    ///   says why: 69 food and **59** timber.
    /// - **776**: five draws, one block — bought. 83 food and 73 timber
    ///   before, 23 and 14 after.
    ///
    /// Sixty food and sixty timber, twice over. A Small City is `COST 1t/1f`
    /// with `SUPPORT food 50 / timber 50`, so that is
    /// `1 × BUILD_COST_FACTOR(10) + 50 × BUILD_SUPPORT_FACTOR(1) × 1` with
    /// **no ceiling** — and the ceiling is what this crate had wrong. All
    /// four `*_RAMP_MAX` belong to `TypeData::get_cost`'s unit arm; the
    /// building arm (`00665787`..`00665b5a`) reads `BUILD_COST_FACTOR` at
    /// `+0x358` and `BUILD_SUPPORT_FACTOR` at `+0x37c` and loads no
    /// `RAMP_MAX` at all. With `RampClass::default()` — military, 125% — the
    /// ramp was clamped to 12 and the city priced at 22, which the AI could
    /// always pay.
    ///
    /// **And the rest of the record, because a price is only half of an
    /// affordability test.** `LEADERS=9` prints ten numbers per good per
    /// leader and this compares six of them — `bucket`, `leftover`,
    /// `resources` (the assembled rate), `income` (the rate after the cap),
    /// `resource_cap`, and `gather_slots` — over forty frames and two
    /// players. 480 good-frames apiece.
    ///
    /// Two of the six carried the whole finding. `leftover` agreed on every
    /// frame from the first widening, which is what said the AI's missing
    /// food was a **lump and not a rate**: the fractional accumulator can
    /// only agree if the two sides are paid the same amount every frame.
    /// And the AI was thirty-two short in `bucket` on all forty. The two
    /// lumps are `Build::do_bonus` — twenty food when the AI's fourth farm
    /// finishes on frame 166 — and `Build::refund_cost` — twelve when
    /// Written Word lands on 201 and re-prices the City State waiting
    /// behind it in the library. See `docs/ECONOMY.md` and `docs/COSTS.md`.
    ///
    /// Two of the six were wrong. One of them is now right, and for a
    /// reason outside the mechanic. **`resource_cap`**: the AI's is
    /// **1392** on every frame against the human's 1120, which is the
    /// British `+25%` on `COMMERCE_CAP[0]` — `70 × 125 / 100 = 87` with
    /// the half truncated before the `× 16`.
    /// [`sim::economy::commerce_cap`] computed that all along and
    /// `Nation::british` was never true, because nothing in this harness
    /// read the dump's own `tribe`. `Sim::set_tribe` does now
    /// (`crates/sim/src/nations.rs`) — run40's `tribe 11` and `tribe 4`
    /// are the roster's British and Nubians — and the 200 became **0**.
    ///
    /// The other is still wrong and inert, so it is asserted **as it
    /// stands** rather than left out — the day it is fixed the assertion
    /// moves rather than passing quietly.
    ///
    /// `gather_slots`: the farms agree and the camps do not — the human's
    /// seven and the AI's five read zero here. `Build::init` surveys a
    /// camp's slots against its own **still empty** `gather_from` and
    /// `Build::find_gather_tiles` recomputes once the list is filled, so a
    /// camp the harness stands up from a dump is activated before it has
    /// any. The human also files **one slot under good 2**, which
    /// `BuildTypeData::get_good@0063bd50` cannot produce — its jump table
    /// at `0063bd84` is Farm 0, Camp 1, Mine 4, University 3, Oil 5 and
    /// nothing else — so a second writer puts it there:
    /// `Leader::plan_strategy@006b9620` line 1137 assigns the **whole**
    /// array from `City::count_gather_slots` and raises the high-water to
    /// match. Reading that is the next widening, and it is booked.
    #[test]
    fn run40_s_census_prices_the_ai_s_second_city_at_sixty() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run40-census.txt") else {
            eprintln!("skipping: no run40 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);

        // The window is [560, 600); a `FRAME n` block is the end of
        // sim-frame n − 1, so the record for `n` is read after `n` ticks.
        let mut compared = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        let mut city_price = [0i32; sim::economy::RESOURCES];
        for n in 1..=599 {
            built.tick();
            if n < 560 {
                continue;
            }
            if n == 576 {
                let village = built
                    .sim
                    .build_types
                    .iter()
                    .position(|b| b.ident == sim::build::Ident::Village)
                    .expect("the Small City");
                city_price = built.sim.building_price(1, village);
            }
            for who in 0..2i64 {
                let Some(block) = log.leader_block(n, who) else {
                    continue;
                };
                let field = |k: &str| -> Vec<i64> {
                    block
                        .all(k)
                        .iter()
                        .map(|v| v.trim().parse().unwrap_or(i64::MIN))
                        .collect()
                };
                let l = &built.sim.ledgers[who as usize];
                // The dump's names, and what each is here. `resources` is
                // the assembled rate and `income` the rate after the cap —
                // the dump's own `rate` is a different field and is zero on
                // every frame of this capture, so it is not compared.
                let rows: [(&str, [i32; sim::economy::RESOURCES]); 6] = [
                    ("bucket", l.bucket),
                    ("leftover", l.leftover),
                    ("resources", l.rate),
                    ("income", l.income),
                    ("resource_cap", l.cap),
                    ("gather_slots[scan]", l.gather_slots),
                ];
                for (key, ours) in rows {
                    let theirs = field(key);
                    // `resource_cap` is written a seventh time for the
                    // leader as a whole; only the six goods are compared.
                    if theirs.len() < sim::economy::RESOURCES {
                        continue;
                    }
                    for g in 0..sim::economy::RESOURCES {
                        compared += 1;
                        if i64::from(ours[g]) != theirs[g] {
                            wrong.push(format!(
                                "frame {n} who {who} {key} good {g}: ours {} theirs {}",
                                ours[g], theirs[g]
                            ));
                        }
                    }
                }
            }
        }

        // **The price, which is the item.** Twenty-two before, sixty after,
        // and sixty is what run41's before-and-after measures.
        assert_eq!(
            &city_price[..2],
            &[60, 60],
            "the AI's second Small City: 1 x BUILD_COST_FACTOR + 50 x 1 x 1, uncapped"
        );
        assert!(
            city_price[2..].iter().all(|&c| c == 0),
            "and nothing else: {city_price:?}"
        );

        // **The record**: 2,880 good-frames — forty frames, two players, six
        // goods, six fields — and **120** disagree, in one shape, a
        // *standing* state rather than anything the window does.
        //
        // - **120 are `gather_slots`** — the two woodcutters' camps and the
        //   human's odd wealth slot, three per frame, for the two reasons
        //   the doc comment above sets out.
        //
        // Two shapes are gone. `resource_cap` was 200 and is **0**, the
        // British commerce bonus arriving with the dump's own `tribe`. And
        // **`bucket` on goods 3, 4 and 5 was 240 and is 0**: the starting
        // grant of a good arrives **with the age**, not at `Leader::init`,
        // so the Ancient age holds no knowledge, no metal and no oil where
        // this crate used to hand out a hundred of each
        // ([`sim::Sim::lay_starting_goods`] and its half in
        // `Sim::gain_tech`; `docs/COSTS.md`, "The starting grant arrives
        // with the good").
        //
        // Everything else is exact on every frame: both players' `bucket` on
        // food, timber and wealth — which is the item — every `leftover`,
        // every `resources`, every `income`, every farm's gather slot, and
        // both players' whole cap.
        eprintln!("run40: {} of {compared} good-frames disagree", wrong.len());
        assert_eq!(
            compared, 2_880,
            "forty frames, two players, six goods, six fields"
        );
        let of = |k: &str| wrong.iter().filter(|w| w.contains(k)).count();
        assert_eq!(
            (
                of("bucket good 0"),
                of("bucket good 1"),
                of("bucket good 2")
            ),
            (0, 0, 0),
            "food, timber and wealth are the original's on every frame: {wrong:?}"
        );
        assert_eq!(
            (
                of("bucket good 3"),
                of("bucket good 4"),
                of("bucket good 5")
            ),
            (0, 0, 0),
            "and so are knowledge, metal and oil, which the Ancient age has \
             not granted yet: {wrong:?}"
        );
        assert_eq!(
            (of("leftover"), of("resources"), of("income")),
            (0, 0, 0),
            "the rate and its accumulator are exact: {wrong:?}"
        );
        assert_eq!(
            of("gather_slots[scan] good 0"),
            0,
            "every farm's gather slot, both players: {wrong:?}"
        );
        assert_eq!(
            of("resource_cap"),
            0,
            "both players' whole commerce cap, the British +25% included: {wrong:?}"
        );
        assert!(
            of("gather_slots") <= 120 && wrong.len() <= 120,
            "the census fell: {} of {compared}, slots {} — the floors are \
             120 and 120",
            wrong.len(),
            of("gather_slots")
        );
    }

    /// **The second map's score.**
    ///
    /// Phase 3's finish line is a traced human-versus-AI capture holding
    /// lockstep on **two** maps, and until 2026-08-29 there was only one:
    /// every number in this file is Great Lakes, seed 12345, the lobby
    /// run10–14 played. A residue chased on one map can be chased into that
    /// map's shape, and nothing here would say so.
    ///
    /// run38 and run39 are the other map — East Indies (`MAP_STYLE 18`),
    /// the same seed and the same rules (`MAP_SIZE 2`, `GAME_RULES 1`,
    /// `REVEAL_MAP 1`). Getting there took finding that **`-config
    /// check.ini` pins the map style and no file can move it**
    /// (`docs/ORACLE.md`, "The lobby is a file"): the capture scripts drop
    /// it for any style but 14 and take the profile's lobby instead.
    ///
    /// run39 is the 1,850-frame dump; run38 is its `DUMP_ALL` start, and it
    /// is the whole sibling list — its own `Initial` carries the heights,
    /// the checksum trace, the herds and the frame seeds, so `build_sim`
    /// stands the simulation up on a map it has never seen with nothing
    /// borrowed. Frame 0 is **175 draws against 175** on the first try.
    ///
    /// History:
    ///   2026-08-29  ticks **167**, orders **167**; player 0 @ 219,
    ///               player 1 @ 168 (item 38's second capture, the first
    ///               number this map has ever had). Great Lakes stands at
    ///               252 the same day, so the two are within a hundred
    ///               frames of each other — the residue chased on one map
    ///               was not chased into its shape.
    ///
    /// What parts it first is an order-list **length**: `1/4` holds two
    /// orders on the original's frame 168 where this simulation holds one,
    /// and its position parts on the same frame. `1/3` at 202 and `1/5` at
    /// 186 are the same disagreement.
    ///
    /// **2026-08-30: that is not the first divergence, and this is not a
    /// fidelity number.** run39's own trace says the word parts at **19**
    /// — see
    /// [`run39_s_long_trace_says_where_the_second_map_s_word_parts`] — so
    /// every figure below sits 148 frames deep into a stream that is
    /// nobody's, and moving them by chasing frame 168 moves nothing. The
    /// floor stays where it is so that nothing falls through it by
    /// accident; the number to steer this map by is the word.
    /// **The original's own per-step prices, against ours.**
    ///
    /// run55 is run39's game — same lobby, same seed, same detail — taken
    /// with `rontrace.cfg`'s `callwin` over frames 1460–1490, which
    /// **proxies** `PathFinder::calc_cost` and `PathFinder::astar_path` and
    /// logs each call's arguments *and its answer* (`tools/trace/tracer.c`,
    /// instrument 4). `docs/PATHFINDER.md` §10 recorded that there is no
    /// numeric per-search oracle to switch on; there is one now, and this
    /// is the check it exists for.
    ///
    /// The comparison is **by argument, not by position**. Two searches
    /// that price differently expand differently, so their call sequences
    /// cannot be lined up entry for entry — but every step *both* sides
    /// priced is a row of §5 with the original's answer beside ours, and a
    /// disagreement there is a bug in the formula rather than in the
    /// search. What the sequences do beyond that is the queue's item 125.
    #[test]
    fn run55_s_frame_1477_prices_are_the_originals() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(costs)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run39.log"),
            trace("rontrace-run55.log"),
        ) else {
            eprintln!("skipping: no East Indies cost trace (set RON_GAMELOG_DIR)");
            return;
        };
        let theirs = costs.calls_in(FRAME, crate::trace::call_site::CALC_COST);
        assert!(
            !theirs.is_empty(),
            "run55's callwin covers frame {FRAME}; it priced nothing there"
        );

        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().expect("run39 is a dump");
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        for _ in 0..FRAME {
            built.tick();
        }
        built.sim.trace_costs = true;
        built.tick();
        let ours = std::mem::take(&mut built.sim.cost_marks);

        // The original's answers, keyed by the whole argument list.
        let mut theirs_by_key: std::collections::BTreeMap<sim::path::CostKey, i32> =
            std::collections::BTreeMap::new();
        for c in &theirs {
            theirs_by_key.insert(c.cost_key().expect("a calc_cost call"), c.ret);
        }
        let mut shared = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        for m in &ours {
            let Some(&t) = theirs_by_key.get(&m.key()) else {
                continue;
            };
            shared += 1;
            if t != m.cost {
                let cell = |v: i32| v.div_euclid(m.step);
                wrong.push(format!(
                    "({},{}) -> ({},{}) dir {} depth {}: ours {} theirs {}",
                    cell(m.from.0),
                    cell(m.from.1),
                    cell(m.to.0),
                    cell(m.to.1),
                    m.dir,
                    m.depth,
                    m.cost,
                    t
                ));
            }
        }
        assert!(
            shared > 0,
            "frame {FRAME}: {} of the original's steps and {} of ours, and not one \
             argument list in common — the searches did not start from the same place",
            theirs.len(),
            ours.len()
        );
        assert!(
            wrong.is_empty(),
            "frame {FRAME}: {} of {shared} shared steps priced differently:\n  {}",
            wrong.len(),
            wrong.join("\n  ")
        );
    }

    /// The **sim-frame** the scout plans on — `docs/PATHFINDER.md` §12's
    /// search, which every document before this one called frame 1477
    /// because that is the label of the dump block it lands in, and a
    /// block `FRAME n` is the end of sim-frame `n − 1` (`docs/ORACLE.md`,
    /// "The frame label, settled"). run55's trace counts `Game::frame` and
    /// puts the whole search on **1476**, which is also the only search in
    /// its thirty-one-frame window.
    const FRAME: i64 = 1476;
}
