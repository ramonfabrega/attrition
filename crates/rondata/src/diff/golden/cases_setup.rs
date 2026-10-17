// Golden case bodies. Wrappers in golden.rs preserve libtest names.
use super::*;

pub(super) fn the_golden_record_s_setup_is_run11_s() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("g4") else {
        eprintln!("skipping: no golden capture (see docs/RUNS.md run101–run105)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let own = log.initial().unwrap().checksums.len();
    assert_eq!(
        own, 0,
        "the golden capture now carries its own checksum trace; the borrow \
         below is no longer what seeds it"
    );
    let built = stand_up(&loaded, &log, &refs, &trace);
    let theirs = trace
        .frame_draws(0)
        .first()
        .map(|d| d.seed)
        .expect("the golden trace has a frame-0 draw");
    assert_eq!(
        built.sim.rng.seed, theirs,
        "the borrowed setup trace does not end on the golden run's own \
         frame-0 word: the two setups are not one game and nothing may be \
         borrowed from the siblings"
    );
    assert_eq!(
        built.frame_seeds.len(),
        0,
        "the golden run took a sibling's per-frame words"
    );
}

pub(super) fn run255_s_repair_is_the_original_s_unit_for_unit() {
    use std::collections::BTreeMap;
    let Some(mut s) = stage_script("ch21", "chapter21") else {
        return;
    };
    let (first, last) = WIDENING_CHAPTER_TWENTY_ONE;
    let players = s.built.sim.players.len() as i64;
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let (mut rows, mut repairs) = (0usize, 0usize);
    for f in 0..last - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < first {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = s.ix.frame_state(at).unwrap();
        let sim = &s.built.sim;
        let mut alone: Vec<(i64, i64, &str)> = Vec::new();
        let mut note = |who: i64, o: i64, what: &str, ours: i64, theirs: Option<i64>| {
            let Some(theirs) = theirs else {
                return;
            };
            rows += 1;
            if ours != theirs {
                firsts
                    .entry((who, o, what.to_string()))
                    .or_insert((n, format!("ours {ours} theirs {theirs}")));
            }
        };
        for them in &frame.units {
            if !(0..players).contains(&them.who) {
                continue;
            }
            let (Ok(w), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                continue;
            };
            let Some(u) = sim.unit_by_o(w, o) else {
                continue;
            };
            let un = &sim.units[u];
            note(
                them.who,
                them.o,
                "builder",
                i64::from(un.was_builder),
                them.unit_masks.map(|m| i64::from(m & 0x400 != 0)),
            );
            let ours = un.orders.iter().find_map(|x| match x.body {
                sim::orders::Body::Repair(b) => Some((b, x.flags)),
                _ => None,
            });
            let theirs = them.orders.iter().find(|x| x.kind == "REPAIRORDER");
            match (ours, theirs) {
                (Some((b, flags)), Some(t)) => {
                    repairs += 1;
                    note(
                        them.who,
                        them.o,
                        "repair.flags",
                        i64::from(flags),
                        Some(t.flags),
                    );
                    // `build_ids` names only the buildings of the start
                    // dump, and a staged one reads back `None` — which is
                    // why `target_ids` leaves this order's target
                    // uncompared in the widening. The identity is the
                    // simulation's own `(owner, index)`, as `unit_ids`'
                    // fallback has it for a unit (item 462).
                    let bd = &sim.buildings[b];
                    note(them.who, them.o, "repair.ox", i64::from(bd.index), t.ox);
                    note(them.who, them.o, "repair.whom", i64::from(bd.owner), t.whom);
                }
                (None, None) => {}
                (mine, _) => alone.push((
                    them.who,
                    them.o,
                    if mine.is_some() {
                        "this crate holds it alone"
                    } else {
                        "the dump holds it alone"
                    },
                )),
            }
        }
        for (who, o, side) in alone {
            firsts
                .entry((who, o, "repair".into()))
                .or_insert((n, side.into()));
        }
    }
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  run255 f{f} {w}/{o} {what}: {row}");
    }
    eprintln!(
        "run255's repair: {rows} rows compared, {repairs} repair-order frames, {} parted",
        firsts.len()
    );
    assert!(rows > 0, "run255 carries the fields it prints");
    assert!(repairs > 0, "run255 holds its REPAIRORDERs");
    let got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    // **The builder bit outlives the repair here.** The original clears
    // `unit_masks & 0x400` in `Unit::add_gather_order@0061a5c0`, always,
    // and in `Unit::think_peasant@005f5760` when its build-and-repair arm
    // finds nothing; this crate sets `was_builder` in `add_build_order`
    // and `add_repair_order` and never clears it. Each citizen's first
    // gather is the block the original's goes to 0: `orders.rs`, not this
    // item's module.
    let want: Vec<String> = [
        "1083 0/6 builder",
        "1130 0/7 builder",
        "1129 0/8 builder",
        "1176 0/9 builder",
    ]
    .iter()
    .map(|r| r.to_string())
    .collect();
    assert_eq!(
        got, want,
        "run255: what parts in the repair's own state moved"
    );
}

pub(super) fn run235_s_bombs_are_the_original_s_record_for_record() {
    use std::collections::{BTreeMap, BTreeSet};
    let Some(mut s) = stage_script("ch17-ammo", "chapter17") else {
        return;
    };
    let (first, last) = RUN235_BOMBS;
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let (mut theirs_n, mut ours_n) = (0usize, 0usize);
    for f in 0..last - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < first {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let raw = s.ix.read_frame(at).unwrap();
        let theirs: BTreeMap<(i64, i64, i64), super::ammo::Ammo> = super::ammo::blocks(&raw)
            .into_iter()
            .filter(|(a, _)| a.flags & 2 != 0)
            .map(|(a, _)| ((a.who, a.o, a.index), a))
            .collect();
        let sim = &s.built.sim;
        let ident = |o: sim::combat::Obj| match o {
            sim::combat::Obj::Unit(u) => {
                (i64::from(sim.units[u].owner), i64::from(sim.units[u].index))
            }
            sim::combat::Obj::Building(b) => (
                i64::from(sim.buildings[b].owner),
                i64::from(sim.buildings[b].index),
            ),
        };
        let ours: BTreeMap<(i64, i64, i64), sim::combat::Projectile> = sim
            .projectiles
            .iter()
            .map(|p| {
                let (w, o) = ident(p.shooter);
                ((w, o, i64::from(p.slot)), *p)
            })
            .collect();
        theirs_n += theirs.len();
        ours_n += ours.len();
        if (805..=808).contains(&n) {
            for (k, a) in &theirs {
                eprintln!("  run235 block {n} theirs {k:?} {a:?}");
            }
            for (k, p) in &ours {
                eprintln!("  run235 block {n} ours   {k:?} {p:?}");
            }
        }
        let keys: BTreeSet<_> = theirs.keys().chain(ours.keys()).copied().collect();
        for key @ (who, o, slot) in keys {
            let (Some(a), Some(p)) = (theirs.get(&key), ours.get(&key)) else {
                let side = if theirs.contains_key(&key) {
                    "the dump"
                } else {
                    "this crate"
                };
                firsts
                    .entry((who, o, format!("ammo[{slot}]")))
                    .or_insert((n, format!("{side} holds it alone")));
                continue;
            };
            let target = p.target.map_or((-1, -1), ident);
            for (name, mine, dumped) in [
                ("cur_time", i64::from(p.cur_time), a.cur_time),
                ("total_time", i64::from(p.total_time), a.total_time),
                ("sx", i64::from(p.launch.x), a.sx),
                ("sy", i64::from(p.launch.y), a.sy),
                ("ex", i64::from(p.landing.x), a.ex),
                ("ey", i64::from(p.landing.y), a.ey),
                ("whom", target.0, a.whom),
                ("ox", target.1, a.ox),
                ("accuracy", i64::from(p.accuracy), a.accuracy),
                ("splash_area", i64::from(p.splash_area), a.splash_area),
                ("num_guys", i64::from(p.num_guys), a.num_guys),
                ("sz", i64::from(p.sz), a.sz),
                ("ez", i64::from(p.ez), a.ez),
                ("angle", i64::from(p.angle.0), a.angle),
                ("v1z", super::ammo::tests::printed(p.v1z), a.v1z),
            ]
            .into_iter()
            .chain(ammo_flag_rows(p, a))
            {
                if mine != dumped {
                    firsts
                        .entry((who, o, format!("ammo[{slot}].{name}")))
                        .or_insert((n, format!("ours {mine} theirs {dumped}")));
                }
            }
        }
    }
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  run235 f{f} {w}/{o} {what}: {row}");
    }
    eprintln!("run235: {theirs_n} dumped rounds, {ours_n} of ours");
    assert!(theirs_n > 0, "run235 dumps the bombs");
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    // **What stands**: the six rounds in flight when the Barracks dies on
    // 1080 keep it as their target there and lose it here — chapter
    // three's standing family (`Sim::forget` clears a dead object from
    // every round; the original's round keeps it to its landing), which
    // no landing reads: a dead target is no target on both sides.
    let mut want: Vec<String> = [
        "1080 0/8 ammo[0].ox",
        "1080 0/8 ammo[0].whom",
        "1080 0/8 ammo[1].ox",
        "1080 0/8 ammo[1].whom",
        "1080 0/8 ammo[3].ox",
        "1080 0/8 ammo[3].whom",
        "1080 0/8 ammo[4].ox",
        "1080 0/8 ammo[4].whom",
        "1080 0/8 ammo[5].ox",
        "1080 0/8 ammo[5].whom",
        "1080 0/8 ammo[6].ox",
        "1080 0/8 ammo[6].whom",
    ]
    .iter()
    .map(|r| r.to_string())
    .collect();
    want.sort();
    assert_eq!(got, want, "run235: what parts on the bombs moved");
}

pub(super) fn run265_s_rounds_are_the_original_s_record_for_record() {
    let Some((firsts, theirs_n, ours_n)) =
        rounds_record_for_record("ch22", "chapter22", RUN265_ROUNDS, &[924, 926])
    else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  run265 f{f} {w}/{o} {what}: {row}");
    }
    eprintln!("run265: {theirs_n} dumped rounds, {ours_n} of ours");
    assert!(theirs_n > 0, "run265 dumps the Fighter's rounds");
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    let mut want: Vec<String> = WANT_RUN265_ROUNDS.iter().map(|r| r.to_string()).collect();
    want.sort();
    assert_eq!(got, want, "run265: what parts on the rounds moved");
}

pub(super) fn near_projection_preserves_the_raw_reader_s_lexical_rules() {
    let cases = [
        "",
        "who 1\no 2\nnear_o 3\nnear_who 4\n",
        "BEGIN UNITDATA\nwho bad\nwho 1\nwho 9\no 2\nnear_o -3\nnear_who 4\n",
        "BEGIN UNITDATA\nwho 1\no 2\n BEGIN GUY\n  near_o 3\n  near_who 4\n",
        "BEGIN UNITDATA\nwho 1\no 2\nnear_o 3\nnear_who 4\nBEGIN UNITDATA\nwho 1\no 2\nnear_o 5\nnear_who 6\n",
        "BEGIN UNITDATA\nwho 1\no 2\nnear_o 3\nBEGIN UNITDATA\nwho 2\no 4\nnear_who 1\n",
        "\tBEGIN UNITDATA\r\n who  1\r\n who 1\r\n o 2\r\n near_o 9223372036854775808\n near_o -1\n near_who\t2\n near_who 2 \n",
    ];
    for text in cases {
        assert_eq!(raw_near(text), raw_near_legacy(text), "{text:?}");
    }
}

pub(super) fn refusing_the_borrowed_frame_stream_does_not_move_great_lakes_word() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(tr)) = (
        crate::testenv::dump("gamelog-run53-greatlakes-24k-trace.txt"),
        trace("rontrace-run53.log"),
    ) else {
        eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    let last = tr.frames.last().map_or(0, |(n, _)| *n);
    let mut answers = Vec::new();
    for refuse in [false, true] {
        let mut init = log.initial().unwrap();
        let own_seeds = init.frame_seeds.clone();
        let own_guys = init.frame_guys.clone();
        borrow_from_siblings(&mut init, &refs);
        if refuse {
            init.frame_seeds = own_seeds.clone();
            init.frame_guys = own_guys.clone();
        }
        let installs = init.frame_seeds.len();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        for _ in 0..last {
            built.tick();
        }
        let count = built
            .frame_sites
            .iter()
            .find(|(f, ours)| ours.len() != tr.labels(*f).len())
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let sequence = built
            .frame_sites
            .iter()
            .find(|(f, ours)| *ours != tr.labels(*f))
            .map(|(f, _)| *f)
            .unwrap_or(last);
        eprintln!(
            "run53 refuse={refuse}: {installs} installed frame word(s); word parts at \
             {count}, sequence at {sequence}"
        );
        answers.push((installs, count, sequence));
    }
    // The anti-vacuity half: the borrow must actually have fired, or the
    // two columns are the same run twice.
    assert_eq!(
        (answers[0].0, answers[1].0),
        (14, 0),
        "run53 no longer takes the siblings' fourteen per-frame words, so \
         this test compares nothing"
    );
    assert_eq!(
        (answers[0].1, answers[0].2),
        (answers[1].1, answers[1].2),
        "Great Lakes' word moves when the borrowed frame stream is refused: \
         the pinned number rests on another capture's per-frame data"
    );
}

pub(super) fn every_object_s_z_is_the_ground_it_stands_on() {
    let Some((dump, text, built)) = chapter_two_stood_up() else {
        return;
    };
    let log = Log::parse(&text);
    let world = &built.sim.world;
    let tile = |p: crate::gamelog::Pos| sim::Pos::new(p.x as i32, p.y as i32).tile();
    // ---- buildings, from the start dump.
    let init = log.initial().expect("the golden dump has a BEGIN GAME");
    let mut wrong_builds = Vec::new();
    for b in &init.builds {
        if i64::from(world.object_z(tile(b.pos))) != b.pos.z {
            wrong_builds.push((b.who, b.o, b.pos, world.object_z(tile(b.pos))));
        }
    }
    // ---- units and their figures, every block.
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let (mut figures, mut units, mut inexact, mut sea) = (0usize, 0usize, 0usize, 0usize);
    let mut wrong_units = Vec::new();
    let mut wrong_figures = Vec::new();
    let mut points = std::collections::BTreeSet::new();
    for at in 0..ix.frames().len() {
        let n = ix.frames()[at].number;
        let frame = ix.frame_state(at).unwrap();
        for u in &frame.units {
            units += 1;
            if i64::from(world.object_z(tile(u.pos))) != u.pos.z {
                wrong_units.push((n, u.who, u.o, u.pos, world.object_z(tile(u.pos))));
            }
            // `Guy::update_z` writes 0 for a unit of sea domain and never
            // reads the ground: Gaia's fish and whales.
            let is_sea = u8::try_from(u.who)
                .ok()
                .zip(i16::try_from(u.o).ok())
                .and_then(|(w, o)| built.sim.unit_by_o(w, o))
                .is_none_or(|i| {
                    matches!(
                        built.sim.profile(sim::combat::Obj::Unit(i)).domain,
                        sim::attrition::Domain::Sea
                    )
                });
            if is_sea {
                sea += u.guys.len();
                continue;
            }
            for (k, g) in u.guys.iter().enumerate() {
                let Some(p) = g.pos else { continue };
                let (z, exact) = world.data_z(p.x as i32, p.y as i32);
                figures += 1;
                points.insert((p.x, p.y));
                if !exact {
                    inexact += 1;
                }
                if i64::from(z) != p.z {
                    wrong_figures.push((n, u.who, u.o, k, p.x, p.y, p.z, z));
                }
            }
        }
    }
    eprintln!(
        "  {} buildings, {units} unit-blocks, {figures} figure-blocks at {} points \
         ({sea} sea figures skipped), {inexact} inexact reads",
        init.builds.len(),
        points.len()
    );
    assert!(
        wrong_builds.is_empty(),
        "building z is not tile_z: {wrong_builds:?}"
    );
    assert!(
        wrong_units.is_empty(),
        "unit z is not tile_z: {:?}",
        &wrong_units[..wrong_units.len().min(10)]
    );
    assert!(
        wrong_figures.is_empty(),
        "figure z is not find_data_z: {:?}",
        &wrong_figures[..wrong_figures.len().min(10)]
    );
    assert_eq!(inexact, 10, "reads on a corner the print cannot pin");
    // The instrument, pinned: a sweep that stopped reading would pass.
    assert!(init.builds.len() > 10, "{} buildings", init.builds.len());
    assert_eq!(
        (figures, points.len()),
        (5310, 947),
        "the figure sweep's reach"
    );
}

pub(super) fn every_rolled_shot_comes_down_where_the_original_s_does() {
    let Some((dump, _, built)) = chapter_two_stood_up() else {
        return;
    };
    let world = &built.sim.world;
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    // Per shot, keyed on its launch: the live record, whether it rolled,
    // its last live cur_time, and its spent record if it stuck.
    type Shot = (super::ammo::Ammo, bool, i64, Option<super::ammo::Ammo>);
    let mut shots: std::collections::BTreeMap<(i64, i64, i64), Shot> =
        std::collections::BTreeMap::new();
    for at in 0..ix.frames().len() {
        let body = ix.read_frame(at).unwrap();
        for (a, _) in super::ammo::blocks(&body) {
            let e = shots
                .entry((a.o, a.sx, a.sy))
                .or_insert((a, false, a.cur_time, None));
            if a.flags & 2 != 0 {
                e.1 |= a.flags & 8 != 0;
                e.2 = e.2.max(a.cur_time);
            } else if e.3.is_none() {
                e.3 = Some(a);
            }
        }
    }
    let mut rolled = Vec::new();
    for ((o, ..), (a, rolls, last, spent)) in &shots {
        if !rolls {
            continue;
        }
        let (launch, landing) = (
            sim::Pos::new(a.sx as i32, a.sy as i32),
            sim::Pos::new(a.ex as i32, a.ey as i32),
        );
        let (sz, total) = (a.sz as i32, a.total_time as i32);
        let v1z = sim::combat::arc_v1z(sz, a.ez as i32, total);
        let mut down = None;
        for t in total..=3 * total {
            let at = sim::combat::arc_point(launch, landing, t, total);
            let z = sim::combat::arc_z(v1z, sz, t);
            let (ground, exact) = world.data_z(at.x, at.y);
            assert!(exact, "shot {o}: the ground at {at:?} is not pinned");
            if !z.gt(sim::single::Single::from_i32(ground)) {
                down = Some((t, z.to_i32(), at, ground));
                break;
            }
        }
        let (t, z, at, ground) = down.unwrap_or_else(|| panic!("shot {o} never comes down"));
        match spent {
            Some(s) => {
                assert_eq!(
                    (i64::from(t), i64::from(at.x), i64::from(at.y), i64::from(z)),
                    (s.total_time, s.ex, s.ey, s.ez),
                    "shot {o}: lands on step {t} at {at:?}, z {z}; the original's spent \
                     record says step {} at ({}, {}), z {}",
                    s.total_time,
                    s.ex,
                    s.ey,
                    s.ez
                );
            }
            None => assert_eq!(
                i64::from(t),
                last + 1,
                "shot {o}: lands on step {t}; the original closed it after step {last}"
            ),
        }
        rolled.push((*o, t, z, at, ground));
    }
    eprintln!("  rolled shots (o, step, z, point, ground): {rolled:?}");
    assert_eq!(
        rolled.iter().map(|r| (r.0, r.1)).collect::<Vec<_>>(),
        vec![(6, 11), (7, 10), (8, 9)],
        "run112's three rolled shots and their landing steps"
    );
}

pub(super) fn run245_s_cast_is_the_original_s_field_for_field() {
    use std::collections::BTreeMap;
    let Some(mut s) = stage_script("ch19", "chapter19") else {
        return;
    };
    let (first, last) = WIDENING_CHAPTER_NINETEEN;
    let players = s.built.sim.players.len() as i64;
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let mut rows = 0usize;
    for f in 0..last - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < first {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = s.ix.frame_state(at).unwrap();
        let raw = s.ix.read_frame(at).unwrap();
        let recs = raw_records(&raw);
        let sim = &s.built.sim;
        let ident = |o: Option<sim::combat::Obj>| match o {
            Some(sim::combat::Obj::Unit(x)) => {
                (i64::from(sim.units[x].index), i64::from(sim.units[x].owner))
            }
            Some(sim::combat::Obj::Building(b)) => (
                i64::from(sim.buildings[b].index),
                i64::from(sim.buildings[b].owner),
            ),
            None => (-1, 0),
        };
        let mut alone: Vec<(i64, i64, &str)> = Vec::new();
        let mut note = |who: i64, o: i64, what: &str, ours: i64, theirs: Option<i64>| {
            let Some(theirs) = theirs else {
                return;
            };
            rows += 1;
            if ours != theirs {
                firsts
                    .entry((who, o, what.to_string()))
                    .or_insert((n, format!("ours {ours} theirs {theirs}")));
            }
        };
        for them in &frame.units {
            if !(0..players).contains(&them.who) {
                continue;
            }
            let (Ok(w), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                continue;
            };
            let Some(u) = sim.unit_by_o(w, o) else {
                continue;
            };
            let un = &sim.units[u];
            let r = recs.get(&("UNITDATA", them.who, them.o));
            let get = |k: &str| r.and_then(|r| r.get(k).copied());
            let (to, tw) = ident(un.cast_target);
            note(
                them.who,
                them.o,
                "mana_burn",
                i64::from(un.mana_burn),
                get("mana_burn"),
            );
            note(
                them.who,
                them.o,
                "spell_time",
                i64::from(un.spell_time),
                get("spell_time"),
            );
            note(
                them.who,
                them.o,
                "cavarch_o",
                if un.cavarch_idle { -2 } else { to },
                get("cavarch_o"),
            );
            let _ = tw;
            note(
                them.who,
                them.o,
                "cavarch_who",
                i64::from(un.cavarch_who),
                get("cavarch_who"),
            );
            note(
                them.who,
                them.o,
                "casting",
                i64::from(un.casting),
                them.unit_masks.map(|m| i64::from(m & 0x20000 != 0)),
            );
            note(
                them.who,
                them.o,
                "visible",
                i64::from(un.visible),
                them.visible,
            );
            let ours_cast = un.orders.iter().find_map(|x| match x.body {
                sim::orders::Body::Cast(c) => Some(c),
                _ => None,
            });
            let theirs_cast = them.orders.iter().find(|x| x.kind == "CASTORDER");
            match (ours_cast, theirs_cast) {
                (Some(c), Some(t)) => {
                    let (co, cw) = ident(c.target);
                    note(them.who, them.o, "cast.ox", co, t.ox);
                    note(them.who, them.o, "cast.whom", cw, t.whom);
                    note(them.who, them.o, "cast.x", i64::from(c.at.x), t.x);
                    note(them.who, them.o, "cast.y", i64::from(c.at.y), t.y);
                    note(
                        them.who,
                        them.o,
                        "cast.spell",
                        i64::from(c.spell),
                        t.cast_spell,
                    );
                    note(
                        them.who,
                        them.o,
                        "cast.paid",
                        i64::from(c.paid),
                        t.cast_paid,
                    );
                }
                (None, None) => {}
                (mine, _) => alone.push((
                    them.who,
                    them.o,
                    if mine.is_some() {
                        "this crate"
                    } else {
                        "the dump"
                    },
                )),
            }
        }
        for ((kind, who, o), r) in &recs {
            if *kind != "BUILDDATA" || !(0..players).contains(who) {
                continue;
            }
            let (Ok(w), Ok(oo)) = (u8::try_from(*who), i16::try_from(*o)) else {
                continue;
            };
            let Some(b) = sim.building_by_o(w, oo) else {
                continue;
            };
            note(
                *who,
                *o,
                "infiltrated",
                i64::from(sim.buildings[b].infiltrated),
                r.get("infiltrated").copied(),
            );
        }
        for (who, o, side) in alone {
            firsts
                .entry((who, o, "cast".into()))
                .or_insert((n, format!("{side} holds it alone")));
        }
    }
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  run245 f{f} {w}/{o} {what}: {row}");
    }
    eprintln!(
        "run245's cast: {rows} rows compared, {} parted",
        firsts.len()
    );
    assert!(rows > 0, "run245 carries the fields it prints");
    let got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    let want: Vec<String> = Vec::new();
    assert_eq!(
        got, want,
        "run245: what parts in the cast's own fields moved"
    );
}

pub(super) fn run265_s_launch_is_the_original_s_field_for_field() {
    use std::collections::BTreeMap;
    let Some(mut s) = stage_script("ch22", "chapter22") else {
        return;
    };
    let (first, last) = WIDENING_CHAPTER_TWENTY_TWO;
    let players = s.built.sim.players.len() as i64;
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let mut rows = 0usize;
    for f in 0..last - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < first {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = s.ix.frame_state(at).unwrap();
        let raw = s.ix.read_frame(at).unwrap();
        let recs = raw_records(&raw);
        let sim = &s.built.sim;
        let mut note = |who: i64, o: i64, what: &str, ours: i64, theirs: Option<i64>| {
            let Some(theirs) = theirs else {
                return;
            };
            rows += 1;
            if ours != theirs {
                firsts
                    .entry((who, o, what.to_string()))
                    .or_insert((n, format!("ours {ours} theirs {theirs}")));
            }
        };
        for them in &frame.units {
            if !(0..players).contains(&them.who) {
                continue;
            }
            let (Ok(w), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                continue;
            };
            let Some(u) = sim.unit_by_o(w, o) else {
                continue;
            };
            let un = &sim.units[u];
            let r = recs.get(&("UNITDATA", them.who, them.o));
            let get = |k: &str| r.and_then(|r| r.get(k).copied());
            note(
                them.who,
                them.o,
                "mana_burn",
                i64::from(un.mana_burn),
                get("mana_burn"),
            );
            // `inside_up` is the chain's link, not the container: the
            // head's is the base, and a plane landing behind another
            // carries the one above it (run265's `0/7` on 1489 reads 6,
            // behind `0/6`: the chain appends at its tail, as the
            // garrison list does here). So the row is in or out.
            note(
                them.who,
                them.o,
                "inside",
                i64::from(un.inside.is_some()),
                get("inside_up").map(|x| i64::from(x >= 0)),
            );
        }
        for ((kind, who, o), r) in &recs {
            if *kind != "BUILDDATA" || !(0..players).contains(who) {
                continue;
            }
            let (Ok(w), Ok(oo)) = (u8::try_from(*who), i16::try_from(*o)) else {
                continue;
            };
            let Some(b) = sim.building_by_o(w, oo) else {
                continue;
            };
            note(
                *who,
                *o,
                "launch_frames",
                i64::from(sim.buildings[b].launch_frames),
                r.get("launch_frames").copied(),
            );
        }
    }
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  run265 f{f} {w}/{o} {what}: {row}");
    }
    eprintln!(
        "run265's launch: {rows} rows compared, {} parted",
        firsts.len()
    );
    assert!(rows > 0, "run265 carries the fields it prints");
    let got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    let mut got = got;
    got.sort();
    // Item 842: the climb agrees, so the Fighter lands on 1385 with the
    // original's and nothing parts (1381's three rows were its early
    // landing).
    let mut want: Vec<String> = Vec::new();
    want.sort();
    assert_eq!(
        got, want,
        "run265: what parts in the launch's own fields moved"
    );
}

pub(super) fn run249_s_transport_is_the_original_s_unit_for_unit() {
    use std::collections::BTreeMap;
    let Some(mut s) = stage_script("ch20", "chapter20") else {
        return;
    };
    let (first, last) = WIDENING_CHAPTER_TWENTY;
    let players = s.built.sim.players.len() as i64;
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let (mut rows, mut casts, mut boards) = (0usize, 0usize, 0usize);
    for f in 0..last - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < first {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = s.ix.frame_state(at).unwrap();
        let sim = &s.built.sim;
        let mut alone: Vec<(i64, i64, &str, &str)> = Vec::new();
        let mut note = |who: i64, o: i64, what: &str, ours: i64, theirs: Option<i64>| {
            let Some(theirs) = theirs else {
                return;
            };
            rows += 1;
            if ours != theirs {
                firsts
                    .entry((who, o, what.to_string()))
                    .or_insert((n, format!("ours {ours} theirs {theirs}")));
            }
        };
        for l in &frame.leaders {
            let Some(t) = usize::try_from(l.who)
                .ok()
                .and_then(|w| sim.transport.get(w))
            else {
                continue;
            };
            let ours =
                i64::from(t.civilian) << 8 | i64::from(t.military) << 9 | i64::from(t.scout) << 10;
            note(
                l.who,
                -1,
                "leader_flags & 0x700",
                ours,
                Some(l.leader_flags & 0x700),
            );
        }
        for them in &frame.units {
            boards += them
                .orders
                .iter()
                .filter(|x| x.kind == "BOARDORDER" || x.kind == "AWAITBOARDORDER")
                .count();
            if !(0..players).contains(&them.who) {
                continue;
            }
            let (Ok(w), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                continue;
            };
            let Some(u) = sim.unit_by_o(w, o) else {
                alone.push((them.who, them.o, "unit", "the dump holds it alone"));
                continue;
            };
            let un = &sim.units[u];
            note(
                them.who,
                them.o,
                "auto_transport",
                i64::from(un.auto_transport),
                them.unit_masks.map(|m| i64::from(m & 0x80_0000 != 0)),
            );
            let ours_cast = un.orders.iter().find_map(|x| match x.body {
                sim::orders::Body::Cast(c) => Some((c, x.flags)),
                _ => None,
            });
            let theirs_cast = them.orders.iter().find(|x| x.kind == "CASTORDER");
            match (ours_cast, theirs_cast) {
                (Some((c, flags)), Some(t)) => {
                    casts += 1;
                    note(
                        them.who,
                        them.o,
                        "cast.spell",
                        i64::from(c.spell),
                        t.cast_spell,
                    );
                    note(
                        them.who,
                        them.o,
                        "cast.flags",
                        i64::from(flags),
                        Some(t.flags),
                    );
                }
                (None, None) => {}
                (mine, _) => alone.push((
                    them.who,
                    them.o,
                    "cast",
                    if mine.is_some() {
                        "this crate holds it alone"
                    } else {
                        "the dump holds it alone"
                    },
                )),
            }
        }
        for (who, o, what, side) in alone {
            firsts
                .entry((who, o, what.into()))
                .or_insert((n, side.into()));
        }
    }
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  run249 f{f} {w}/{o} {what}: {row}");
    }
    eprintln!(
        "run249's transport: {rows} rows compared, {casts} casts, {} parted",
        firsts.len()
    );
    assert!(rows > 0, "run249 carries the fields it prints");
    assert_eq!(casts, 2, "run249 holds each Transport cast on one block");
    assert_eq!(
        boards, 0,
        "run249 prints a BOARDORDER or an AWAITBOARDORDER"
    );
    let got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    let want: Vec<String> = Vec::new();
    assert_eq!(
        got, want,
        "run249: what parts in the transport's own state moved"
    );
}

pub(super) fn run223_s_air_orders_are_read_whole() {
    let Some((dump, _)) = golden("ch17") else {
        eprintln!("skipping: no golden capture ch17 (see docs/RUNS.md)");
        return;
    };
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let mut at = |n: i64| {
        let i = ix
            .frames()
            .iter()
            .position(|x| x.number == n)
            .expect("run223 holds the block");
        ix.frame_state(i).unwrap()
    };
    let mut head = |n: i64, o: i64| {
        let f = at(n);
        let u = f
            .units
            .iter()
            .find(|u| u.who == 0 && u.o == o)
            .expect("the aircraft is on the block")
            .clone();
        u.orders_front_first().next().expect("an order").clone()
    };
    let strafe = head(642, 6);
    assert_eq!((strafe.index, strafe.kind.as_str()), (16, "STRAFEORDER"));
    assert_eq!(
        (strafe.ox, strafe.whom, strafe.flags),
        (Some(-1), Some(-1), 4)
    );
    assert_eq!(strafe.mandatory, Some(1));
    assert_eq!((strafe.air_oxx, strafe.air_whose), (Some(2007), Some(0)));
    assert_eq!(
        (strafe.cruising_alt, strafe.sharp_turn, strafe.air_old),
        (Some(1600), Some(0), Some(0))
    );
    assert_eq!(strafe.returning, Some(1));
    assert_eq!((strafe.strafe_xx, strafe.strafe_yy), (Some(-1), Some(-1)));
    let patrol = head(666, 7);
    assert_eq!((patrol.index, patrol.kind.as_str()), (17, "AIRPATROLORDER"));
    assert_eq!(
        (patrol.patrol_x.clone(), patrol.patrol_y.clone()),
        (vec![21120], vec![16512])
    );
    assert_eq!((patrol.air_oxx, patrol.air_whose), (Some(2007), Some(0)));
    assert_eq!(patrol.returning, Some(0));
    assert_eq!((patrol.strafe_xx, patrol.strafe_yy), (None, None));
}

pub(super) fn the_ground_order_s_life_is_the_dump_s() {
    const ATTACK: i64 = sim::orders::index::ATTACK as i64;
    const GROUND: i64 = 23;
    let Some((dump, _)) = golden("ch3b") else {
        eprintln!("skipping: no golden capture ch3b (docs/RUNS.md run146)");
        return;
    };
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let mut holders = std::collections::BTreeSet::new();
    let mut life = Vec::new();
    for at in 0..ix.frames().len() {
        let n = ix.frames()[at].number;
        let f = ix.frame_state(at).unwrap();
        for u in &f.units {
            if u.orders.iter().any(|o| o.index == GROUND) {
                holders.insert((u.who, u.o));
            }
        }
        if (779..=866).contains(&n) {
            let u = f
                .units
                .iter()
                .find(|u| u.who == 0 && u.o == 6)
                .expect("the catapult");
            life.push((n, ground_rows(u), u.recharging.expect("recharging")));
        }
    }
    assert_eq!(
        holders.into_iter().collect::<Vec<_>>(),
        vec![(0, 6)],
        "run146: only the catapult ever holds a ground order"
    );
    for (n, rows, rech) in &life {
        let want: (Vec<[i64; 6]>, i64) = match n {
            779 => (vec![], 0),
            780 => (vec![[ATTACK, 0, 1, -1, -1, 0]], 0),
            781..=863 => (
                vec![[GROUND, 2472, 8136, 0, 1, -128], [ATTACK, 1, 1, -1, -1, 0]],
                864 - n,
            ),
            _ => (vec![], 0),
        };
        assert_eq!((rows.clone(), *rech), want, "run146 block {n}");
    }

    let Some(r44) = crate::testenv::dump("gamelog-run44-islands-turners.txt") else {
        return;
    };
    let mut ix = crate::capture::indexed::IndexedCapture::open(&r44).unwrap();
    let mut firsts = std::collections::BTreeMap::new();
    for at in 0..ix.frames().len() {
        let n = ix.frames()[at].number;
        if !(240..=410).contains(&n) {
            continue;
        }
        let f = ix.frame_state(at).unwrap();
        for u in &f.units {
            let mut front = u.orders_front_first();
            let (Some(head), Some(under)) = (front.next(), front.next()) else {
                continue;
            };
            if head.index != GROUND || firsts.contains_key(&(u.who, u.o)) {
                continue;
            }
            let (w, o) = (under.whom.unwrap(), under.ox.unwrap());
            let t = f
                .units
                .iter()
                .find(|t| t.who == w && t.o == o)
                .expect("the attack's target");
            firsts.insert(
                (u.who, u.o),
                (
                    n,
                    under.index,
                    head.ag_att_x.unwrap(),
                    head.ag_att_y.unwrap(),
                    t.pos.x,
                    t.pos.y,
                ),
            );
        }
    }
    let firsts: Vec<_> = firsts.into_iter().collect();
    assert_eq!(
        firsts,
        vec![
            ((0, 15), (324, ATTACK, 39816, 39720, 39816, 39720)),
            ((1, 6), (247, ATTACK, 39421, 40762, 39421, 40762)),
            ((1, 7), (248, ATTACK, 40152, 41016, 40152, 41016)),
        ],
        "run44: a ground order's point is its attack's target where it stands"
    );
}

pub(super) fn a_turning_catapult_s_crew_mirror_and_never_walk() {
    let Some(r44) = crate::testenv::dump("gamelog-run44-islands-turners.txt") else {
        eprintln!("skipping: no run44 (docs/RUNS.md run44)");
        return;
    };
    let mut ix = crate::capture::indexed::IndexedCapture::open(&r44).unwrap();
    let mut turn = Vec::new();
    let mut stepped: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
    for at in 0..ix.frames().len() {
        let n = ix.frames()[at].number;
        let f = ix.frame_state(at).unwrap();
        for u in &f.units {
            let packer = matches!(u.guys.first().and_then(|g| g.kind), Some(265 | 266));
            if !packer || u.unit_masks.is_some_and(|m| m & 0x80000 != 0) {
                continue;
            }
            for g in u.guys.iter().skip(1) {
                if g.last_speed.is_some_and(|s| s != 0) {
                    *stepped.entry(g.cur_anim.unwrap()).or_default() += 1;
                }
            }
            if u.who == 0 && u.o == 15 && (323..=330).contains(&n) {
                let lead = u.guys[0];
                let crew: Vec<_> = u.guys[1..]
                    .iter()
                    .map(|g| {
                        (
                            g.cur_anim.unwrap(),
                            g.cur_time.unwrap(),
                            g.end_time.unwrap(),
                            g.last_time.unwrap(),
                            g.pos.unwrap().x == g.des.unwrap().x
                                && g.pos.unwrap().y == g.des.unwrap().y,
                            g.last_speed.unwrap() != 0,
                        )
                    })
                    .collect();
                turn.push((n, lead.cur_anim.unwrap(), lead.cur_time.unwrap(), crew));
            }
        }
    }
    for (n, anim, time, crew) in &turn {
        let want = if *n == 323 {
            // The idle before the push: the crew on their slots, still.
            (0, 1, vec![(0, 1, 79, -1, true, false); 2])
        } else {
            let t = n - 323;
            (21, t, vec![(21, t, 79, -1, true, true); 2])
        };
        assert_eq!((*anim, *time, crew.clone()), want, "run44 0/15 block {n}");
    }
    assert_eq!(turn.len(), 8, "run44 carries 323–330");
    assert!(
        stepped
            .keys()
            .all(|a| sim::anim::category(*a as i8) != sim::anim::category(sim::anim::WALK)),
        "run44: an unpacked packer's stepping crew played the walk: {stepped:?}"
    );
    assert_eq!(
        stepped.into_iter().collect::<Vec<_>>(),
        vec![(21, 92), (22, 22)],
        "run44: the stepping crew of an unpacked packer"
    );
}

pub(super) fn run875_start_seen3_plane_is_compared_cell_for_cell() {
    let Some(s) = stage_script("ch79", "chapter79") else {
        return;
    };
    let Some((dump, _)) = golden("ch79") else {
        return;
    };
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let init = log.initial().expect("run875 has a start block");
    let theirs = crate::gamelog::world_fog3(&init.world);
    assert_eq!(
        theirs.len(),
        14_400,
        "run875's start plane is the whole map"
    );
    let w = &s.built.sim.world;
    let fw = w.fog_xs();
    let ours: Vec<u8> = (0..theirs.len() as i32)
        .map(|i| w.seen3(i % fw, i / fw))
        .collect();
    let bad: Vec<(i32, i32, u8, u8)> = (0..theirs.len())
        .filter(|&i| ours[i] != theirs[i])
        .map(|i| (i as i32 % fw, i as i32 / fw, ours[i], theirs[i]))
        .collect();
    eprintln!(
        "run875 start seen3: theirs {} lit, ours {} lit, {} cells part, first {:?}",
        theirs.iter().filter(|&&b| b != 0).count(),
        ours.iter().filter(|&&b| b != 0).count(),
        bad.len(),
        bad.iter().take(5).collect::<Vec<_>>()
    );
    assert_eq!(
        (bad.len(), theirs.iter().filter(|&&b| b != 0).count()),
        (0, 42),
        "run875's start `seen3` plane parts from this crate's: the writer \
         (`World::set_seen3` through `update_seen`) is what lays the 42 lit \
         half-cells, 63 of bit 1 and bit 2 together on later blocks"
    );
}

pub(super) fn run436_s_other_seat_s_commands_are_dropped_at_the_pump() {
    const RUN436: &str = "0 !ai off\n606 add airbase who=1 212,104\n608 add biplane who=1 212,118\n\
        610 add barracks who=1 190,82\n612 add citizen who=1 198,82\n614 add trireme who=0 181,82\n\
        616 add hoplite who=0 204,88\n618 add catapult who=0 194,92\n620 be 1\n\
        620 @flight 1 2006 1 6\n620 be 0\n622 @attack 0 2007 1 10\n630 @attack 0 2007 1 6\n\
        700 @attack 0 2002 1 7\n720 be 1\n720 @repair 1 2007 1 7\n720 be 0\n";
    let Some(mut s) = stage_script("ch41-run436", "chapter41") else {
        return;
    };
    s.script = Script::parse(RUN436);
    let mut done = crate::golden::Applied::default();
    while s.built.sim.frame <= 725 {
        done.merge(&s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded));
        s.built.tick();
    }
    let dropped = done
        .skipped
        .iter()
        .filter(|((_, why), _)| why.contains("not the seat at the pump"))
        .map(|(_, n)| *n)
        .sum::<usize>();
    assert_eq!(
        dropped, 2,
        "the flight and the repair are dropped at the pump"
    );
    let biplane = s
        .built
        .sim
        .units
        .iter()
        .position(|u| u.owner == 1 && u.index == 6)
        .expect("who=1's Biplane");
    assert!(
        s.built.sim.units[biplane].orders.is_empty(),
        "the Biplane took the dropped flight: {:?}",
        s.built.sim.units[biplane].orders
    );
}

pub(super) fn an_issuer_line_for_another_seat_is_refused_as_the_dll_refuses_it() {
    let Some(mut s) = stage_script("ch41-run436", "chapter41") else {
        return;
    };
    s.script = Script::parse(
        "0 !ai off\n606 add airbase who=1 212,104\n608 add biplane who=1 212,118\n620 @flight 1 2006 1 6\n",
    );
    let mut done = crate::golden::Applied::default();
    while s.built.sim.frame <= 622 {
        done.merge(&s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded));
        s.built.tick();
    }
    let why: Vec<&str> = done.skipped.keys().map(|(_, w)| *w).collect();
    assert_eq!(
        why,
        vec!["not the console's seat (refusal 1)"],
        "{:?}",
        done.skipped
    );
}

pub(super) fn run422_s_casts_are_the_original_s_field_for_field() {
    let Some((rows, decoys, firsts)) =
        casts_field_for_field("ch39", "chapter39", WIDENING_CHAPTER_THIRTY_NINE)
    else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  run422 f{f} {w}/{o} {what}: {row}");
    }
    eprintln!(
        "run422's casts: {rows} rows compared, {decoys} decoy rows, {} parted",
        firsts.len()
    );
    assert!(rows > 0, "run422 carries the fields it prints");
    assert!(decoys > 0, "run422's decoys are compared");
    let got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    let want: Vec<String> = Vec::new();
    assert_eq!(
        got, want,
        "run422: what parts in the casts' own fields moved"
    );
}

pub(super) fn run430_s_casts_are_the_original_s_field_for_field() {
    let Some((rows, _, firsts)) =
        casts_field_for_field("ch40", "chapter40", WIDENING_CHAPTER_FORTY)
    else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  run430 f{f} {w}/{o} {what}: {row}");
    }
    eprintln!(
        "run430's casts: {rows} rows compared, {} parted",
        firsts.len()
    );
    assert!(rows > 0, "run430 carries the fields it prints");
    let got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    let want: Vec<String> = Vec::new();
    assert_eq!(
        got, want,
        "run430: what parts in the casts' own fields moved"
    );
}

pub(super) fn the_missile_offset_is_the_install_s() {
    let Some(root) = crate::testenv::install_root() else {
        eprintln!("skipping: no install (set RON_INSTALL)");
        return;
    };
    let path = std::path::Path::new(&root).join("Data/effects_graphics.xml");
    let text = std::fs::read(&path).expect("effects_graphics.xml");
    let text = String::from_utf8_lossy(&text);
    let tag = text
        .split("<MISSILEOFFSET")
        .nth(1)
        .and_then(|t| t.split("/>").next())
        .expect("one MISSILEOFFSET");
    let attr = |k: &str| -> i32 {
        tag.split(&format!("{k}=\""))
            .nth(1)
            .and_then(|v| v.split('"').next())
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or_else(|| panic!("MISSILEOFFSET {k} in {tag:?}"))
    };
    assert_eq!(
        (attr("x"), attr("y"), attr("z")),
        sim::air::MISSILE_OFFSET,
        "the install's MISSILEOFFSET"
    );
}

pub(super) fn stage_walk() {
    let Ok(spec) = std::env::var("RON_STAGE") else {
        return;
    };
    let mut parts = spec.splitn(3, ':');
    let (Some(run), Some(stem), Some(to)) = (parts.next(), parts.next(), parts.next()) else {
        panic!("RON_STAGE=<run>:<script>:<to frame>");
    };
    let to: i64 = to.parse().expect("RON_STAGE's frame");
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden(run) else {
        eprintln!("skipping: no golden capture {run}");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = if stem.contains('/') {
        Script::read(std::path::Path::new(stem)).expect("the staged script")
    } else {
        script_named(stem)
    };
    let mut applied = crate::golden::Applied::default();
    let mut seen: std::collections::BTreeMap<String, String> = Default::default();
    // `RON_STAGE_ALL=1` (item 1091, `docs/GOLDEN.md` §46): every unit, not
    // the aircraft and missiles alone, with its health — a blast's cast is
    // on the ground.
    let all = std::env::var_os("RON_STAGE_ALL").is_some();
    // `RON_STAGE_DRAWS=1` (item 1310, `docs/GOLDEN.md` §56): each frame's
    // draw labels, so a mutation can be walked on the staging before any
    // capture exists — an arm whose mutation leaves every frame's draws
    // alone cannot be held by the chapter's walk, whatever the original does.
    let draws = std::env::var_os("RON_STAGE_DRAWS").is_some();
    if draws {
        built.sim.trace_phases = true;
    }
    let unit_name = |t: Option<usize>| {
        t.and_then(|t| loaded.unit_type_names.get(t))
            .cloned()
            .unwrap_or_default()
    };
    let build_name = |t: Option<usize>| {
        t.and_then(|t| loaded.build_type_names.get(t))
            .cloned()
            .unwrap_or_default()
    };
    while built.sim.frame <= to {
        let did = script.stage(built.sim.frame, &mut built, &loaded);
        applied.merge(&did);
        built.tick();
        let f = built.sim.frame - 1;
        if draws {
            let ours = built.frame_sites.last().map_or(&[][..], |(_, v)| v);
            eprintln!("draws f{f} {} {}", ours.len(), ours.join(" "));
        }
        let s = &built.sim;
        let mut now: Vec<(String, String)> = Vec::new();
        for (i, u) in s.units.iter().enumerate() {
            if !u.alive() && !seen.contains_key(&format!("u{i}")) {
                continue;
            }
            let air = matches!(
                s.profile(sim::combat::Obj::Unit(i)).domain,
                sim::attrition::Domain::Air
            );
            if !air && !all {
                continue;
            }
            let orders: Vec<String> = u
                .orders
                .iter()
                .map(|o| {
                    let b = format!("{:?}", o.body);
                    let b: String = b.chars().take(160).collect();
                    format!("f{} {b}", o.flags)
                })
                .collect();
            now.push((
                format!("u{i}"),
                format!(
                    "{}/{} {} alive={} hp={} in={:?} ride={:?} avg={} pos=({}, {}) z={} mana_burn={} head={} des={} [{}]",
                    u.owner,
                    u.index,
                    unit_name(u.ty),
                    u.alive(),
                    u.health,
                    u.inside.map(|b| s.buildings[b].index),
                    // Item 1223: the boat a passenger rides, and the
                    // figure's running average — the repaint's standing
                    // test on a boat that steps ashore.
                    u.inside_unit.map(|b| s.units[b].index),
                    u.movement.body.avg_speed,
                    u.pos.x,
                    u.pos.y,
                    u.airframe.z,
                    u.mana_burn,
                    // Item 1310: the heading and the order's `dest_angle`,
                    // which an age's snap must leave apart to be seen.
                    u.movement.heading.0,
                    u.movement.des_angle.0,
                    orders.join(" | ")
                ),
            ));
        }
        for (i, b) in s.buildings.iter().enumerate() {
            let hangar = b.ty.is_some_and(|t| s.is_hangar(t));
            if !(all
                || hangar
                || !b.queue.items.is_empty()
                || !b.gather.is_empty()
                || !b.gatherers.is_empty())
            {
                continue;
            }
            let inside: Vec<i16> = b.garrison.iter().map(|&u| s.units[u].index).collect();
            // Item 1278: the gatherer chain, head first (`gather_down`'s
            // walk), which `all_gathering`'s prune reads.
            let chain: Vec<i16> = b.gatherers.iter().map(|&u| s.units[u].index).collect();
            // The queue by type and price: a job's counter moves every frame.
            let queue: Vec<(usize, [i16; 3])> =
                b.queue.items.iter().map(|q| (q.ty, q.cost)).collect();
            now.push((
                format!("b{i}"),
                format!(
                    "B {}/{} {} alive={} hp={} active={} damage={}/{} job={} under_attack={} pos=({}, {}) queue={:?} gather={:?} chain={chain:?} inside={inside:?} launch={}",
                    b.owner,
                    b.index,
                    build_name(b.ty),
                    b.alive,
                    b.health,
                    // Item 1358: a site's progress and its wound, which a
                    // blow, a bomb and the enemy land each charge.
                    b.active,
                    b.damage,
                    b.damage_frac,
                    b.job_counter,
                    b.under_attack,
                    b.pos.x,
                    b.pos.y,
                    queue,
                    b.gather,
                    b.launch_frames
                ),
            ));
        }
        now.push((
            "leader0".into(),
            // Item 1268: the rare mask beside the buckets, which the
            // library line's tail reads (`has_rare`).
            format!(
                "L 0 buckets {:?} rare {:#x}",
                s.ledgers[0].bucket, s.ledgers[0].rare
            ),
        ));
        // Item 1182: each city's members and its attack bits, which the
        // computer's sortie reads (`city_flags & 3`).
        if all {
            for (c, city) in s.cities.iter().enumerate() {
                let members: Vec<i16> =
                    city.members.iter().map(|&m| s.buildings[m].index).collect();
                now.push((
                    format!("c{c}"),
                    // Item 2235: the capital bit and the alarm's two
                    // stamps (`Sim::city_alarm`).
                    format!(
                        "C {}/{} alive={} under_attack={} attacking={} capital={} raid_stamp={} attack_stamp={} members={members:?}",
                        city.owner,
                        s.buildings[city.building].index,
                        city.alive,
                        city.no_heal,
                        city.attacking,
                        city.capital,
                        city.raid_stamp,
                        city.attack_stamp
                    ),
                ));
            }
        }
        for (k, v) in now {
            if seen.get(&k) != Some(&v) {
                eprintln!("f{f} {v}");
                seen.insert(k, v);
            }
        }
    }
    // `RON_STAGE_MAP=1` (item 1078, `docs/GOLDEN.md` §45): the territory
    // on the last frame, one character a cell (`.` unowned, `?` a claim
    // the original writes −2, else the player), and every live building
    // with its cell, so a target can be placed by who owns its landing.
    if let Ok(win) = std::env::var("RON_STAGE_TILES") {
        let v: Vec<i32> = win.split(',').map(|x| x.parse().unwrap()).collect();
        let s = &built.sim;
        for y in v[1]..v[3] {
            let row: String = (v[0]..v[2])
                .map(|x| {
                    let m = s.world.tile_mask(sim::world::Pos::new(x, y));
                    if m & sim::world::tile::BLOCKED != 0 {
                        '#'
                    } else if m & 0x30 == 0x20 {
                        '~'
                    } else if m & 0x30 == 0x30 {
                        'T'
                    } else if m & 3 == 2 {
                        'M'
                    } else if m & 3 == 1 {
                        'C'
                    } else if m & 3 == 3 {
                        'b'
                    } else {
                        '.'
                    }
                })
                .collect();
            eprintln!("tile {y:3} {row}");
        }
    }
    if std::env::var_os("RON_STAGE_MAP").is_some() {
        let s = &built.sim;
        for y in 0..s.world.height() {
            let row: String = (0..s.world.width())
                .map(|x| match s.world.owner(sim::world::Cell { x, y }) {
                    // Item 1182: the sea, so a hull and a shore can be
                    // placed off the same print.
                    sim::world::Owner::None
                        if s.world
                            .region_of(sim::world::Cell { x, y })
                            .is_some_and(|r| {
                                matches!(s.world.terrain(r), sim::world::Terrain::Sea)
                            }) =>
                    {
                        '~'
                    }
                    sim::world::Owner::None => '.',
                    sim::world::Owner::Ambiguous => '?',
                    sim::world::Owner::Player(p) => char::from(b'0' + p),
                })
                .collect();
            eprintln!("map {y:2} {row}");
        }
        for b in s.buildings.iter().filter(|b| b.alive) {
            let c = b.pos.cell();
            eprintln!(
                "bld {}/{} {} pos=({}, {}) cell=({}, {}) owner_there={:?}",
                b.owner,
                b.index,
                build_name(b.ty),
                b.pos.x,
                b.pos.y,
                c.x,
                c.y,
                s.world.owner(c)
            );
            for q in &b.queue.items {
                eprintln!("    queue ty={} job_counter={}", q.ty, q.job_counter);
            }
        }
    }
    eprintln!(
        "staged: {} ran, {} unit(s), {} building(s); skipped {:?}",
        applied.ran, applied.units, applied.buildings, applied.skipped
    );
}
