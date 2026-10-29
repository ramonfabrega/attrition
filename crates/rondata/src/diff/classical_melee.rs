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
    count: 718,
    sequence: 718,
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

/// The word's window on run1450's stride: the block before the word to
/// the block after it.
pub(crate) const WIDENING_CLASSICAL_MELEE: (i64, i64) = (601, 968);

/// The word's window, widened whole on run1450's stride blocks, the rounds
/// compared — the coverage driver's slice too.
pub(crate) fn classical_melee_word_window() -> Option<harness::tests::Widened> {
    crate::diff::setup::with_staged_script(melee_script(), || {
        harness::tests::widen_on_siblings_with_rounds(
            &[MELEE_START],
            true,
            MELEE_LONG,
            "run1450",
            &[(MELEE_LONG.0, WIDENING_CLASSICAL_MELEE.0)],
            WIDENING_CLASSICAL_MELEE,
            1,
            &[],
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
    let Some(w) = classical_melee_word_window() else {
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
    pin_eq!(w.blocks, 4, "every stride block in the window");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    pin_eq!(w.firsts.len(), 988, "run1450's word window");
    pin_eq!(w.rounds_read, (14, 17), "the rounds read");
    pin_eq!(w.rounds.len(), 22, "the rounds that part");
}
