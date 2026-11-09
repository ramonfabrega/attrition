//! **The sixth melee: the third's lobby with two massed armies, past 100
//! live corpses** (item 3971, `docs/GOLDEN.md` §177; the shape is §155).
//! run1467's lobby held whole — Great Sahara, `MAP_STYLE 7`, seed 12345,
//! `--profile PLAYERS=2`, the idle human Nubians (4) in seat 0, the French
//! (10) as who 1 and the Russians (13) as who 2, `STARTING_TECHNOLOGY 7`,
//! `ENDING_TECHNOLOGY 7`, `STARTING_RESOURCES 7` — with
//! `tools/gamelog/golden/melee-corpses.cmd`: each computer's buckets
//! filled on 600 and 602, then three massed lines a side from 610. The
//! subject is `Objects::add_death@00653b60`'s pool of 100 `DeathObj`
//! slots, which no capture on disk had filled.

use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) use super::information_melee::MELEE_START;

/// The melee's script, read from the tree.
pub(crate) fn melee_script() -> crate::golden::Script {
    let path = format!(
        "{}/../../tools/gamelog/golden/melee-corpses.cmd",
        env!("CARGO_MANIFEST_DIR")
    );
    crate::golden::Script::read(std::path::Path::new(&path))
        .unwrap_or_else(|e| panic!("tools/gamelog/golden/melee-corpses.cmd: {e}"))
}

/// The script's own line count: two `resource` lines and three `add`
/// twins.
pub(crate) const MELEE_STAGED: usize = 8;

/// **The staging, walked before any capture holds it** (`docs/GOLDEN.md`
/// §3, §177): run1467's start stood up with the script staged by the
/// harness's own tick, to frame `RON_MELEE_STAGE` (default 2600), the
/// death list and the pool printed every 50 frames.
#[test]
fn corpse_melee_staging_walk() {
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
        .unwrap_or(2600);
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let mut init = log.initial().expect("a start dump");
    let own = log.initial().expect("a start dump");
    borrow_from_siblings(&mut init, &[&own]);
    let mut built =
        crate::diff::setup::with_staged_script(script, || build_sim(&loaded, &init, Tuning::RON));
    let mut most = (0usize, 0i64);
    while built.sim.frame <= to {
        built.tick();
        let s = &built.sim;
        let live: usize = s
            .combat
            .deaths
            .iter()
            .map(|d| d.figures.live(s.frame - d.first_frame) as usize)
            .sum();
        if live > most.0 {
            most = (live, s.frame);
        }
        if s.frame % 50 == 0 {
            let alive = |w| s.units.iter().filter(|u| u.alive() && u.owner == w).count();
            eprintln!(
                "  {}: deaths {} records {} alive who1 {} who2 {}",
                s.frame,
                s.combat.deaths.len(),
                live,
                alive(1),
                alive(2)
            );
        }
    }
    let st = built.staged.as_ref().expect("the script is staged");
    eprintln!(
        "melee staged to {to}: {} line(s) ran, {} unit(s); {} carried and not acted on; most records {} on {}",
        st.applied.ran,
        st.applied.units,
        st.applied.skipped_total(),
        most.0,
        most.1
    );
    for ((word, why), k) in &st.applied.skipped {
        eprintln!("  {k:5} {word}: {why}");
    }
    assert_eq!(st.applied.skipped_total(), 0, "add refuses nothing");
    assert_eq!(st.applied.ran, MELEE_STAGED, "every line ran");
}

/// run1542: the staged game, the draw stream to its end at `cover=0`, the
/// melee's records on a stride of 50.
pub(crate) const MELEE_LONG: (&str, &str) = (
    "gamelog-run1542-greatsahara-corpse-melee-trace.txt",
    "rontrace-run1542.log",
);

pub(crate) use super::gunpowder_melee::Melee;

pub(crate) const CORPSE_MELEE: Melee = Melee {
    length: 2600,
    // Item 3971, first try: **744 of 2600**, the count and the sequence
    // together, 124 frames past the last `add` (620) and 257 before the
    // pool first fills (1001, both sides). On 744 ours spends 82 against
    // 83: at index 74 theirs spends `Unit::close+0xcb6` — a death's draw —
    // where ours spends `Farms::inc_time+0x1ae`: theirs acted
    // (`docs/GOLDEN.md` §177). Item 3975: **744 → 2600, the game whole**,
    // with the MLRS's rack (piece 223's rows in `sim::pivot::RELEASES`):
    // the rockets left from the launcher's own square, 240 low, where
    // theirs leave the rack some 41 out and 226..240 up, so every blast
    // from 649 landed elsewhere — on 701 `2/143` read `hits:damage` ours
    // 135 against theirs' 136, and died on 745 against 744.
    count: 2600,
    sequence: 2600,
};

/// **Where the sixth melee's draw stream first parts** (item 3971): the
/// harness walks run1467's start with the script staged and compares every
/// frame's draws against run1542's trace, the count and the sequence.
#[test]
fn corpse_melee_first_parting() {
    let _pins = Pins::hold();
    let m = &CORPSE_MELEE;
    let Some(w) = crate::diff::setup::with_staged_script(melee_script(), || {
        third::walk_from(MELEE_START, MELEE_LONG)
    }) else {
        return;
    };
    eprintln!(
        "CORPSE_MELEE: count {} sequence {} of {}; {}",
        w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.last, m.length, "the game's last frame");
    pin_eq!(w.count, m.count, "the draw count's first parting");
    pin_eq!(w.sequence, m.sequence, "the draw sequence's first parting");
}

/// The melee's word: the lower of the two partings.
pub(crate) const CORPSE_MELEE_WORD: i64 = if CORPSE_MELEE.count < CORPSE_MELEE.sequence {
    CORPSE_MELEE.count
} else {
    CORPSE_MELEE.sequence
};

/// **The melee's closing state, as the `Melee:` line says it** (Loop
/// 3842): the sum of [`endpoint::EndpointResult::counts`]' seven at
/// run1542's closing block.
///
/// Item 3971, first try: 420 off at a close 1857 frames past the word —
/// 217 units off, 15 unlinked, 11 buildings, 7 cities, 150 leader rows
/// and 20 groups; 241 units compared. Item 3975: 420 → **0**, the game
/// whole, with the MLRS's rack: 256 units compared, every count 0.
pub(crate) const CORPSE_MELEE_CLOSING_OFF: usize = 0;

/// The word's window on run1542's stride: the block before the word to
/// 250 frames past it (item 3971: the word 744, so 701..995). Item 3975
/// closes the game: the block before the word, 2501, through the closing
/// block 2601, as the other closed melees'.
pub(crate) const WIDENING_CORPSE_MELEE: (i64, i64) = (2501, 2601);

/// The word's window, widened whole on run1542's stride blocks, the rounds
/// compared — the coverage driver's slice too.
pub(crate) fn corpse_melee_closing_window() -> Option<harness::tests::Widened> {
    crate::diff::setup::with_staged_script(melee_script(), || {
        harness::tests::widen_on_siblings_with_rounds(
            &[MELEE_START],
            true,
            MELEE_LONG,
            "run1542",
            &[(MELEE_LONG.0, WIDENING_CORPSE_MELEE.0)],
            WIDENING_CORPSE_MELEE,
            1,
            &[2601],
            true,
        )
    })
}

/// **The sixth melee's word window, widened whole** (item 3971): every
/// dumped record on run1542's stride blocks over [`WIDENING_CORPSE_MELEE`],
/// both directions, the rounds and the death records too. `RON_FIRSTS=1`
/// prints every row.
#[test]
fn corpse_melee_run1542_s_word_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = corpse_melee_closing_window() else {
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
        let mut rounds: Vec<_> = w.rounds.iter().collect();
        rounds.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
        for (b, rows) in &w.standing {
            for ((who, o, what), r) in rows {
                eprintln!("  standing {b} {who}/{o} {what}: {r}");
            }
        }
    }
    // Item 3971, on 701..995: 6 blocks, 5,752 rows, the rounds read
    // (81, 84) and 235 parting. Item 3975, the game closed, on 2501..2601:
    // 3 blocks, 344 rows, (30, 30) and 31 — the rounds' `angle` alone
    // (parked 3975a), and on 2501 the first block's standing families
    // (`group`, `pool`, `form`, `build:extra`) beside 29 units' `hits`:
    // theirs heals units ours does not, from 1151 (`1/17`, an Elite Special
    // Forces, `healing` 13 against 0; parked 3975b). The draws agree
    // throughout and the close is 0 off. Item 3978: 344 → **276**, with
    // the supply heal (`sim::heal`'s `supply_heal`: the French bonus's
    // period 20 within the Despot `1/164`'s supply radius) — the 68 `hits`
    // rows of 2501 agree; on 1151 `1/17` reads `healing` 13, `damage_frac`
    // 0 and `hits:damage` 8 both. What stands is the first block's standing
    // families (248) and who=0's seven orders on the closing block (28).
    // Item 3979: the rounds 31 → **6**, with a ground round's `angle` the
    // scattered landing's (`Ammo::init@0067bbf0`, `67c747`–`67c75a`): on
    // 2551 `2/159`'s `ammo[0]` reads theirs' -1128988672 both, every round
    // fired before 2600 agrees whole. What parts is the two fired on 2600
    // (`cur_time 1` on 2601): `2/159`'s `ammo[12]` lands (2697, 24408)
    // against (2656, 24424) and `2/160`'s `ammo[13]` (2654, 24414) against
    // (2661, 24406), each `angle` following its own landing (parked 3979a).
    pin_eq!(w.blocks, 3, "every stride block in the window");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    pin_eq!(w.firsts.len(), 276, "run1542's word window");
    pin_eq!(w.rounds_read, (30, 30), "the rounds read");
    pin_eq!(w.rounds.len(), 6, "the rounds that part");
}

/// **Every seat's leader** (the shape item 3791 gave the first two,
/// `docs/GOLDEN.md` §151): run1467's frames 1 and 37 print all three
/// seats' `LEADERDATA` whole, and run1542's closing dump at 2601.
#[test]
fn corpse_melee_s_leaders_are_compared_on_every_seat() {
    let _pins = Pins::hold();
    let Some(w) = crate::diff::gunpowder_melee::seat_walk(
        melee_script,
        MELEE_START,
        MELEE_LONG,
        &[MELEE_START],
        true,
    ) else {
        return;
    };
    w.report("run1467+closing");
    pin_eq!(w.players, 3, "the seats the lobby seats");
    pin_eq!(
        w.records.clone(),
        vec![3, 3, 3],
        "leader records compared, by seat"
    );
    pin_eq!(w.standing(), vec![12, 9, 7], "standing keys, by seat");
    // Item 3971: 176. Item 3975, the game closed: 1.
    pin_eq!(w.rest().len(), 1, "the keys that part, by seat");
}

/// **The melee's closing state** (item 3971): run1542's closing block, the
/// whole map's units, walked from run1467's start with the script staged.
#[test]
fn corpse_melee_run1542_s_closing_state() {
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
            "Corpse melee closing: frame {} units {} counts {:?}",
            e.frame,
            e.compared,
            e.counts(),
        );
        Some(e)
    }) else {
        return;
    };
    pin_eq!(e.frame, 2601, "the closing block");
    pin_eq!(e.compared, 256, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.counts().iter().sum::<usize>(),
        CORPSE_MELEE_CLOSING_OFF,
        "the closing records off, the `Melee:` line's suffix"
    );
}

/// The capacity both sides must hold: `Objects::init@0065ea80:384`.
const POOL: usize = sim::combat::DEATH_POOL;

/// **The death pool stops at 100 on both sides** (item 3971,
/// `docs/GOLDEN.md` §177; parked T71-10): run1542's every stride block,
/// the dump's `DEATH_OBJS` records counted against ours' live figure
/// records on the same frame of the staged walk. Past the word (744) the
/// two games are not one game, so this is a census, not a lockstep
/// comparison: it pins that each side fills the pool and that neither
/// holds more than [`POOL`] records, which the grow-only list this crate
/// kept before broke on the first full block. The original's evictions
/// are read off the dump too — a record that leaves a full block long
/// before its life is out, and every one older than the oldest that stays.
#[test]
fn corpse_melee_s_death_pool_stops_at_a_hundred_on_both_sides() {
    let _pins = Pins::hold();
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(start)) = (dump(MELEE_LONG.0), dump(MELEE_START)) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let log = Log::parse(&text);
    let start_text = crate::capture::read(start);
    let start_log = Log::parse(&start_text);
    let mut init = start_log.initial().expect("a start dump");
    let own = start_log.initial().expect("a start dump");
    borrow_from_siblings(&mut init, &[&own]);
    let mut built = crate::diff::setup::with_staged_script(melee_script(), || {
        build_sim(&loaded, &init, Tuning::RON)
    });
    let mut rows = Vec::new();
    for f in log.frame_states() {
        while built.sim.frame < f.n {
            built.tick();
        }
        let s = &built.sim;
        let ours: usize = s
            .combat
            .deaths
            .iter()
            .map(|d| d.figures.live(s.frame - d.first_frame) as usize)
            .sum();
        rows.push((f.n, ours, f.deaths.len()));
    }
    for (n, o, t) in &rows {
        eprintln!("  {n}: ours {o} theirs {t}");
    }
    let full = |side: fn(&(i64, usize, usize)) -> usize| -> Vec<i64> {
        rows.iter()
            .filter(|r| side(r) == POOL)
            .map(|r| r.0)
            .collect()
    };
    let most = |side: fn(&(i64, usize, usize)) -> usize| rows.iter().map(side).max();
    pin_eq!(most(|r| r.2), Some(POOL), "theirs never past the pool");
    pin_eq!(most(|r| r.1), Some(POOL), "ours never past the pool");
    // Item 3971: theirs holds 100 on 1001..1251 and 1351 (99 on 1301);
    // ours on 1001..1101, then 99 — the two games parted on 744, and the
    // kills past it are not the same kills. Item 3975, the draws whole to
    // the end: ours on 1001..1251 too; 1351 only theirs', not read.
    pin_eq!(
        full(|r| r.2),
        vec![1001, 1051, 1101, 1151, 1201, 1251, 1351],
        "theirs' full blocks"
    );
    pin_eq!(
        full(|r| r.1),
        vec![1001, 1051, 1101, 1151, 1201, 1251],
        "ours' full blocks"
    );
}
