//! **The second melee: a held Classical age on Great Sahara, every seat a
//! computer** (item 3774, `docs/GOLDEN.md` §150; the shape is §155, the
//! first melee's chapter §149). Great Sahara, `MAP_STYLE 7`, seed 12345,
//! `--profile PLAYERS=2`: the idle human Nubians (4) in seat 0, the Greeks
//! (5) as who 1 and the Egyptians (7) as who 2 (`--seat-tribe 1=5
//! --seat-tribe 2=7`), all three at war from frame 0 with no `war` line;
//! `STARTING_TECHNOLOGY 1`, `ENDING_TECHNOLOGY 1`, `STARTING_RESOURCES 7`.
//! `tools/gamelog/golden/melee-classical.cmd` grants each computer 100,000
//! of every resource on 600 and 602 and spawns twelve of the age's types a
//! side from 610, two frames apart, between the two territories, each
//! computer's own line among them (the Companion, the Chariot); the age's
//! other never-seen lines go to who 0.
//!
//! Captures: run1449, the `DUMP_ALL` start; run1450, the staged game to
//! its end at `cover=0`, `end:UNITS=3,GUYS=2,DEATHS=1,AMMO=5` on a stride
//! of 100, walked from run1449's initial state with the script staged by
//! [`Built::tick`] ([`crate::diff::setup::with_staged_script`]); run1450b,
//! the same script at `cover=1`, pinned in `blind::TRACES`.

use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

/// run1449: the lobby's `DUMP_ALL` start, 37 frames.
pub(crate) const MELEE_START: &str = "gamelog-run1449-greatsahara-classical-melee-start.txt";

/// The melee's script, read from the tree.
pub(crate) fn melee_script() -> crate::golden::Script {
    let path = format!(
        "{}/../../tools/gamelog/golden/melee-classical.cmd",
        env!("CARGO_MANIFEST_DIR")
    );
    crate::golden::Script::read(std::path::Path::new(&path))
        .unwrap_or_else(|e| panic!("tools/gamelog/golden/melee-classical.cmd: {e}"))
}

/// The script's own line count: two `resource` lines, twelve `add` twins
/// and the six lines on who 0.
pub(crate) const MELEE_STAGED: usize = 32;

/// **The staging, walked before any capture holds it** (`docs/GOLDEN.md`
/// §3): run1449's start stood up with the script staged by the harness's
/// own tick, to frame `RON_MELEE_STAGE` (default 700), printing where
/// every `add` landed and every line the interpreter carried and did not
/// act on.
#[test]
fn classical_melee_staging_walk() {
    let Some(inst) = install() else {
        return;
    };
    let Some(path) = dump(MELEE_START) else {
        eprintln!("skipping: no {MELEE_START} (set RON_GAMELOG_DIR)");
        return;
    };
    let script = melee_script();
    assert_eq!(
        script.lines().len(),
        MELEE_STAGED,
        "the melee's staged lines"
    );
    let to: i64 = std::env::var("RON_MELEE_STAGE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(700);
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let mut init = log.initial().expect("a start dump");
    let own = log.initial().expect("a start dump");
    borrow_from_siblings(&mut init, &[&own]);
    let mut built =
        crate::diff::setup::with_staged_script(script, || build_sim(&loaded, &init, Tuning::RON));
    while built.sim.frame <= to {
        built.tick();
    }
    let st = built.staged.as_ref().expect("the script is staged");
    eprintln!(
        "melee staged to {to}: {} line(s) ran, {} unit(s), {} building(s); {} carried and not acted on",
        st.applied.ran,
        st.applied.units,
        st.applied.buildings,
        st.applied.skipped_total()
    );
    for ((word, why), k) in &st.applied.skipped {
        eprintln!("  {k:5} {word}: {why}");
    }
    for line in &st.applied.landed {
        eprintln!("  {line}");
    }
    for (w, n) in built.sim.nation.iter().enumerate() {
        eprintln!(
            "  who={w} human={} age {} tribe {:?}",
            n.human, built.sim.players[w].age, built.sim.tech[w].power
        );
    }
    assert_eq!(
        st.applied.skipped_total(),
        0,
        "every staged line reached the simulation"
    );
    assert_eq!(st.applied.ran, MELEE_STAGED, "every line ran");
}

/// run1450: the staged game, the draw stream to its end at `cover=0`, the
/// melee's records on a stride of 100.
pub(crate) const MELEE_LONG: (&str, &str) = (
    "gamelog-run1450-greatsahara-classical-melee-trace.txt",
    "rontrace-run1450.log",
);

pub(crate) use super::gunpowder_melee::Melee;

pub(crate) const CLASSICAL_MELEE: Melee = Melee {
    length: 2600,
    // Item 3774, first try: **718 of 2600**, the count and the sequence
    // together, fifty frames past the script's last line (668). Ours 13
    // draws against 14: index 0 agrees, and at index 1 theirs spends
    // `Unit::fight+0x9b0` where ours spends `Guy::set_anim+0x97a <
    // Unit::do_move+0x11cf` (`docs/GOLDEN.md` §150).
    //
    // **718 → 784 (sequence) and 786 (count), item 3785** (§150.1): the
    // human's Royal Balamob Slingers (piece 39) throw from `launch::BAYS`'
    // rows off run1457's packet, so `0/19`'s stone of 713 leaves (16772,
    // 15529) at 699 and the Greek Catapult `1/26`'s strike of 718 — theirs'
    // second `Unit::fight+0x9b0` — comes in ours too. Pieces 124 (the
    // Kushite Archers' bays) and 141 (the Camel Archer's pivot rows), from
    // the same packet, close block 701's `1/19` blow and `0/17` round and
    // leave the draws as they were. On 784 ours spends 47 draws against 47,
    // parting at index 30: ours the Barbarians `0/21`'s `Guy::set_anim
    // +0x97a < Unit::move_step+0x823` where theirs spends `0/25`'s
    // `Guy::set_anim+0xf2f < Guy::move+0x166`.
    //
    // **784 → 865, item 3787** (§150.2), two builds: a death clears every
    // figure's disc, the crew's too (`Objects::kill_guy`'s last arm), so
    // the dead Catapult `1/26`'s crew punches its holes and `0/21` steps on
    // 785 — the word went to 828; and `set_type` keeps no same-type guard,
    // so who=1's second government re-types The Despot `1/30` on 828 with
    // its three `init_real` draws. On 865 ours spends 12 draws against 13,
    // parting at index 3: theirs' `Unit::close+0xcb6`, a death, where ours
    // spends `Farms::inc_time+0x1ae`.
    //
    // **865 → 1284, item 3789** (§150.3): a player's ranged unit chasing
    // a unit on a move leads it — `find_attack_pos`'s flanking branch —
    // so the human's Royal Balamob Slingers `0/20` walk at (20424, 12216)
    // on 860, ahead of the running Light Horse `1/16`, where ours walked
    // at (17112, 14760); faced along that walk, `0/20` dies to `2/24`'s
    // arrow on 865, its share 31 of 95 passed by the flank's blow. On 1284
    // ours spends 15 draws against 1002, parting at index 5: theirs'
    // `PathFinder::calc_road_cost+0x46` where ours spends `Guy::set_anim
    // +0x97a < Guy::inc_time+0x271`.
    //
    // **1284 → 1382, item 3790** (§150.4): an epoch's research takes
    // `train_time`'s catch-up term — who=1 holds eight epochs from 1004
    // against who=2's four, so who=2's Military epoch finishes on 1018 and
    // its Civic epoch by 1228, not 1087 and 1362 (`Sim::epoch_catch_up`).
    // The Civic sweep pulls the Stable `2/2012`, placed on 757 one tile
    // past Thebes' mask, into the city, and its activation on 1279 flags
    // the city's roads: theirs' 986-draw `calc_road_cost` replan on its
    // slot 1284. On 1382 ours spends 20 draws against 21, parting at
    // index 2: ours `Leader::produce_building+0x1805` where theirs spends
    // `Leader::make_stuff+0x63d`.
    //
    // **1382 → 2600, whole, item 3793** (§150.5): `unit_prod_value` reads
    // `UnitTypeData::relative_value`, the matchup table
    // `UnitType::fill_relative_values` fills at load, not the combat
    // table. who=1's Light Horse (TI 209) against the human's Barbarians,
    // Bushi and Royal Balamob Slingers reads −170 in theirs, so its
    // escrowed offer of 1380 is 4,584,896 and ranks above the Temple
    // (391,136): on 1382 theirs buys two Light Horse and a Citizen with
    // the wood, and the Temple — ours' `produce_building` — waits. Ours
    // read −24 and offered 130,624. run1465b's `LEADERS=9` window
    // prints both lists.
    count: 2600,
    sequence: 2600,
};

/// The melee's word: the lower of the two partings.
pub(crate) const CLASSICAL_MELEE_WORD: i64 = if CLASSICAL_MELEE.count < CLASSICAL_MELEE.sequence {
    CLASSICAL_MELEE.count
} else {
    CLASSICAL_MELEE.sequence
};

/// **Where the second melee's draw stream first parts** (item 3774): the
/// harness walks run1449's start with the script staged and compares every
/// frame's draws against run1450's trace, the count and the sequence.
#[test]
fn classical_melee_first_parting() {
    let _pins = Pins::hold();
    let m = &CLASSICAL_MELEE;
    let Some(w) = crate::diff::setup::with_staged_script(melee_script(), || {
        third::walk_from(MELEE_START, MELEE_LONG)
    }) else {
        return;
    };
    eprintln!(
        "CLASSICAL_MELEE: count {} sequence {} of {}; {}",
        w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.last, m.length, "the game's last frame");
    pin_eq!(w.count, m.count, "the draw count's first parting");
    pin_eq!(w.sequence, m.sequence, "the draw sequence's first parting");
}

/// **The Kushite Archers loose from the bow** (item 3785, `docs/GOLDEN.md`
/// §150.1). The human's Kushite Archers `0/14` (TI 174, piece 124) fire on
/// the Greek Phalanx `1/19` on 673 at `CHAR_ATTACK1`'s frame 15: the round
/// leaves piece 124's bay, (15167, 15587) at 690 — run1457's window prints
/// it there on 675 — and flies 14 frames, so `1/19` takes its first blow on
/// 687, `damage_frame` 687 and 16 + 10/16, as run1457's frame 688 prints.
/// From its own square (15096, 15672) at 574 the round flew 15 and the
/// blow landed on 688.
#[test]
fn classical_melee_s_kushite_archers_loose_from_the_bow() {
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        crate::diff::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    crate::diff::setup::borrow_pasture(&mut init, &tr);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    let unit = |sim: &sim::Sim, who: u8, o: usize| {
        sim.units
            .iter()
            .position(|u| u.alive() && u.owner == who && u.index as usize == o)
    };
    while built.sim.frame < 673 {
        built.tick();
    }
    let bow = sim::combat::Obj::Unit(unit(&built.sim, 0, 14).expect("0/14 on 673"));
    let p = unit(&built.sim, 1, 19).expect("1/19 on 673");
    let find = |sim: &sim::Sim| {
        sim.projectiles
            .iter()
            .find(|r| r.shooter == bow && r.target == Some(sim::combat::Obj::Unit(p)))
            .copied()
    };
    while find(&built.sim).is_none() && built.sim.frame < 675 {
        built.tick();
    }
    let round = find(&built.sim).expect("0/14's round at 1/19 on 673");
    assert_eq!(
        ((round.launch.x, round.launch.y), round.sz, round.total_time),
        ((15167, 15587), 690, 14),
        "piece 124's bay, and fourteen frames"
    );
    while built.sim.frame <= 687 {
        built.tick();
    }
    let phalanx = &built.sim.units[p];
    assert_eq!(
        (
            phalanx.combat.damage_frame,
            phalanx.health,
            phalanx.damage_frac
        ),
        (687, 129, 10),
        "1/19 struck on 687, as run1457's frame 688"
    );
}

/// **The Royal Balamob Slingers loose from the sling** (item 3785,
/// `docs/GOLDEN.md` §150.1): the human's `0/19` (TI 89, piece 39) throws
/// at the Greek Catapult `1/26` (TI 265) on 713 at `CHAR_ATTACK1`'s frame
/// 18, and the stone leaves piece 39's bay, (16772, 15529) at 699, as
/// run1457's window prints it on 715, in 4 frames. Without the row it left
/// the figure's square, and the Catapult's strike of 718 — theirs' second
/// `Unit::fight+0x9b0`, the word 3774 booked — did not come.
#[test]
fn classical_melee_s_royal_balamob_slingers_loose_from_the_sling() {
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        crate::diff::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    crate::diff::setup::borrow_pasture(&mut init, &tr);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    let unit = |sim: &sim::Sim, who: u8, o: usize| {
        sim.units
            .iter()
            .position(|u| u.alive() && u.owner == who && u.index as usize == o)
    };
    while built.sim.frame < 713 {
        built.tick();
    }
    let sling = sim::combat::Obj::Unit(unit(&built.sim, 0, 19).expect("0/19 on 713"));
    let cat = sim::combat::Obj::Unit(unit(&built.sim, 1, 26).expect("1/26 on 713"));
    let find = |sim: &sim::Sim| {
        sim.projectiles
            .iter()
            .find(|r| r.shooter == sling && r.target == Some(cat))
            .copied()
    };
    while find(&built.sim).is_none() && built.sim.frame < 715 {
        built.tick();
    }
    let stone = find(&built.sim).expect("0/19's stone at 1/26 on 713");
    assert_eq!(
        ((stone.launch.x, stone.launch.y), stone.sz, stone.total_time),
        ((16772, 15529), 699, 4),
        "piece 39's bay, and four frames"
    );
}

/// **A dead Catapult's crew clears its own disc** (item 3787,
/// `docs/GOLDEN.md` §150.2). The Greek Catapult `1/26` dies on 747 with its
/// crew figure at (16509, 15030), cell (343, 313); `Objects::kill_guy`
/// clears a radius-2 disc around that figure's own cell, punching holes in
/// the Barbarians `0/22`'s disc (rows 314–315, x 342–344) and `0/24`'s
/// (row 311, x 343–345), which run1459's packet at 780 and run1460's at
/// 760 print. With the holes the Barbarians `0/21` takes its step on 785,
/// as theirs does; without them it stood blocked on 784.
#[test]
fn classical_melee_s_dead_catapult_crew_clears_its_disc() {
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        crate::diff::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    crate::diff::setup::borrow_pasture(&mut init, &tr);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    while built.sim.frame < 780 {
        built.tick();
    }
    let holes: Vec<(i32, i32)> = (342..=344)
        .flat_map(|x| [(x, 314), (x, 315)])
        .chain((343..=345).map(|x| (x, 311)))
        .collect();
    let set: Vec<_> = holes
        .iter()
        .filter(|&&(x, y)| built.sim.coll.get(x, y))
        .collect();
    assert!(
        set.is_empty(),
        "the packets' holes at 780, set in ours: {set:?}"
    );
}

/// **The human's slingers lead a running horseman** (item 3789,
/// `docs/GOLDEN.md` §150.3). The Royal Balamob Slingers `0/20` (TI 89,
/// land, who 0) chase the Greek Light Horse `1/16` (TI 209) on 860, which
/// is running from them under its own move. `Unit::find_attack_pos`'s
/// flanking branch sweeps round a point ahead of the horseman and takes
/// the spot outright: run1462's block 861 has `0/20`'s `MOVEORDER` at
/// (20424, 12216) and its path's last leg at (17352, 15288), and `0/20`
/// at (16652, 15416). Facing along that walk it dies to `2/24`'s arrow on
/// 865, as run1450's block 901 dates it; aimed at the horseman's own seat
/// it walked to (17112, 14760) and died on 867.
#[test]
fn classical_melee_s_slingers_lead_the_running_light_horse() {
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        crate::diff::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    crate::diff::setup::borrow_pasture(&mut init, &tr);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    let unit = |sim: &sim::Sim, who: u8, o: usize| {
        sim.units
            .iter()
            .position(|u| u.alive() && u.owner == who && u.index as usize == o)
    };
    while built.sim.frame <= 860 {
        built.tick();
    }
    let s = unit(&built.sim, 0, 20).expect("0/20 on 861");
    let u = &built.sim.units[s];
    let dest = u.orders.front().and_then(|o| match o.body {
        sim::orders::Body::Move(m) => Some((m.dest.x, m.dest.y)),
        _ => None,
    });
    assert_eq!(
        (dest, (u.pos.x, u.pos.y)),
        (Some((20424, 12216)), (16652, 15416)),
        "0/20 leads 1/16, as run1462's block 861"
    );
    while built.sim.frame <= 864 {
        built.tick();
    }
    assert!(unit(&built.sim, 0, 20).is_some(), "0/20 stands through 864");
    built.tick();
    assert!(unit(&built.sim, 0, 20).is_none(), "0/20 dies on 865");
}

/// **The Egyptians catch up to City State** (item 3790, `docs/GOLDEN.md`
/// §150.4): an epoch's research takes `train_time`'s catch-up term
/// (`Sim::epoch_catch_up`). who=1 holds eight epochs from 1004 against
/// who=2's four, so who=2's Civic epoch lands by 1228 rather than 1362;
/// its sweep pulls the Stable `2/2012` — placed on 757 one tile past
/// Thebes' mask — into the city, and the Stable's activation on 1279
/// flags the city's roads, so it replans on its slot 1284 as run1450's
/// trace does (986 `calc_road_cost` draws). Each leader's next queue
/// frame is theirs' closing `LEADERDATA`'s: who=2's Military 1157, who=1's
/// Civic 1182 (ours queued them on 957 and 982 without the term).
#[test]
fn classical_melee_s_egyptians_catch_up_to_city_state() {
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(MELEE_LONG.0),
        dump(MELEE_START),
        crate::diff::testkit::trace(MELEE_LONG.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    crate::diff::setup::borrow_pasture(&mut init, &tr);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    let stable = |sim: &sim::Sim| {
        sim.buildings
            .iter()
            .position(|b| b.alive && b.owner == 2 && b.index == 2012)
    };
    let mut joined = None;
    while built.sim.frame <= 1279 {
        built.tick();
        if joined.is_none()
            && stable(&built.sim).is_some_and(|b| built.sim.buildings[b].city.is_some())
        {
            joined = Some(built.sim.frame);
        }
    }
    assert_eq!(joined, Some(1228), "2/2012 joins Thebes with City State");
    let b = stable(&built.sim).expect("2/2012 on 1279");
    assert!(
        built.sim.buildings[b].active && built.sim.buildings[b].regen_roads,
        "2/2012's activation flags the city's roads"
    );
    assert_eq!(
        (
            built.sim.ai[2].tech_cat_frame[0],
            built.sim.ai[1].tech_cat_frame[1]
        ),
        (1157, 1182),
        "theirs' closing LEADERDATA's queue frames"
    );
}

/// The word's window on run1450's stride: the block before the word to
/// 250 frames past it (item 3789 moved it from 601..968 with the word,
/// item 3790 from 1201..1534). Item 3793 closes the game: the window is
/// 1301 through the closing block 2601.
pub(crate) const WIDENING_CLASSICAL_MELEE: (i64, i64) = (1301, 2601);

/// The word's window, widened whole on run1450's stride blocks, the rounds
/// compared — the coverage driver's slice too.
pub(crate) fn classical_melee_closing_window() -> Option<harness::tests::Widened> {
    crate::diff::setup::with_staged_script(melee_script(), || {
        harness::tests::widen_on_siblings_with_rounds(
            &[MELEE_START],
            true,
            MELEE_LONG,
            "run1450",
            &[(MELEE_LONG.0, WIDENING_CLASSICAL_MELEE.0)],
            WIDENING_CLASSICAL_MELEE,
            1,
            &[2601],
            true,
        )
    })
}

/// **The second melee's word window, widened whole** (item 3774): every
/// dumped record on run1450's stride blocks over
/// [`WIDENING_CLASSICAL_MELEE`], both directions, the rounds too.
/// `RON_FIRSTS=1` prints every row.
#[test]
fn classical_melee_run1450_s_word_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = classical_melee_closing_window() else {
        return;
    };
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
        eprintln!("  rounds read {:?}", w.rounds_read);
        for ((who, o, what), (f, r)) in &w.rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
        for (b, rows) in &w.standing {
            for ((who, o, what), r) in rows {
                eprintln!("  standing {b} {who}/{o} {what}: {r}");
            }
        }
    }
    // Item 3793, the game whole (window 1301..2601, thirteen stride blocks
    // and the closing block): **150** keys, rounds read (6, 6), none
    // parting (686 keys and 5 rounds parting on 1301..1632 before). Block
    // 1301 carries the standing families alone (`build:extra` 42, `form`
    // 40, the groups' `held` and pools); later blocks add only those
    // families as each unit is born (a birth's `g.angle`, a new group's
    // `held`, a new building's `build:extra`); the closing block 2601 adds
    // the idle human's citizens `0/1`..`0/7`, which hold an order in ours
    // and stand idle in theirs (parked 3793a). The closing state is
    // `classical_melee_run1450_s_closing_state`.
    pin_eq!(w.blocks, 14, "every stride block in the window");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    pin_eq!(w.firsts.len(), 150, "run1450's word window");
    pin_eq!(w.rounds_read, (6, 6), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **The second melee's closing state** (item 3793, `docs/GOLDEN.md`
/// §150.5): run1450's closing block, the whole map's units, walked from
/// run1449's start with the script staged.
#[test]
fn classical_melee_run1450_s_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) =
        (dump(MELEE_LONG.0), dump(MELEE_START), trace(MELEE_LONG.1))
    else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let Some(e) = crate::diff::setup::with_staged_script(melee_script(), || {
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(path);
        let start_text = crate::capture::read(start);
        let log = Log::parse(&text);
        let start_log = Log::parse(&start_text);
        let mut init = log.initial().unwrap();
        let sibling = start_log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sibling]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let fin = log
            .final_state()
            .or_else(|| log.frame_states().pop())
            .expect("closing whole-map state");
        let e = endpoint::walk_to_close(&mut built, &fin, 8);
        eprintln!(
            "Classical melee closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
            e.frame,
            e.compared,
            e.counts(),
            e.torn,
            e.off,
            e.unlinked,
            e.extra
        );
        Some(e)
    }) else {
        return;
    };
    pin_eq!(e.frame, 2601, "the closing block");
    // Item 3793, the game whole: 95 units compared, every count zero, no
    // extra. One unit is torn: the idle human's Citizen `0/6` stands at
    // (35472, 33327) in ours against (35450, 33316), holding two orders
    // where theirs' stands idle (parked 3793a).
    pin_eq!(e.compared, 95, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(e.torn.clone(), vec![(0, 6)], "the closing tears");
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}
