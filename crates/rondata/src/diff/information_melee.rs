//! **The third melee: a held Information age on Great Sahara, every seat a
//! computer** (item 3797, `docs/GOLDEN.md` §152; the shape is §155, the
//! two before §149 and §150). Great Sahara, `MAP_STYLE 7`, seed 12345,
//! `--profile PLAYERS=2`: the idle human Nubians (4) in seat 0, the French
//! (10) as who 1 and the Russians (13) as who 2 (`--seat-tribe 1=10
//! --seat-tribe 2=13`), all three at war from frame 0 with no `war` line;
//! `STARTING_TECHNOLOGY 7`, `ENDING_TECHNOLOGY 7`, `STARTING_RESOURCES 7`.
//! `tools/gamelog/golden/melee-information.cmd` grants each computer
//! 100,000 of every resource on 600 and 602 and spawns thirteen of the
//! age's types a side from 610, two frames apart, between the two
//! territories — the air, anti-air and missile lines among them; the
//! age's other never-seen lines go to who 0, and three probes of
//! `parse_type`'s name column to all three.
//!
//! Captures: run1467, the `DUMP_ALL` start; run1468, the staged game to
//! its end at `cover=0`, `end:UNITS=3,GUYS=2,DEATHS=1,AMMO=5` on a stride
//! of 100, walked from run1467's initial state with the script staged by
//! [`Built::tick`] ([`crate::diff::setup::with_staged_script`]); run1468b,
//! the same script at `cover=1`, pinned in `blind::TRACES`.

use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

/// run1467: the lobby's `DUMP_ALL` start, 37 frames.
pub(crate) const MELEE_START: &str = "gamelog-run1467-greatsahara-information-melee-start.txt";

/// The melee's script, read from the tree.
pub(crate) fn melee_script() -> crate::golden::Script {
    let path = format!(
        "{}/../../tools/gamelog/golden/melee-information.cmd",
        env!("CARGO_MANIFEST_DIR")
    );
    crate::golden::Script::read(std::path::Path::new(&path))
        .unwrap_or_else(|e| panic!("tools/gamelog/golden/melee-information.cmd: {e}"))
}

/// The script's own line count: two `resource` lines, thirteen `add`
/// twins, the two seats' probes and the seven lines on who 0.
pub(crate) const MELEE_STAGED: usize = 37;

/// **The staging, walked before any capture holds it** (`docs/GOLDEN.md`
/// §3): run1467's start stood up with the script staged by the harness's
/// own tick, to frame `RON_MELEE_STAGE` (default 700), printing where
/// every `add` landed and every line the interpreter carried and did not
/// act on.
#[test]
fn information_melee_staging_walk() {
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
    // `mirage` and `hind` name nothing: `add`'s walk reads the `NAME`
    // column alone, and an earlier generic type holds each line's
    // (`golden::add_named`; run1468's block 701 has no `1/29` or `2/29`).
    // `elite_special_forces_spanish` is the generic 77, on twenty
    // characters (run1468's `0/22`).
    assert_eq!(st.applied.skipped_total(), 2, "the two probes add refuses");
    let probes: Vec<_> = st
        .applied
        .landed
        .iter()
        .filter(|l| l.starts_with("add mirage") || l.starts_with("add hind"))
        .collect();
    assert!(probes.is_empty(), "the probes stood up: {probes:?}");
    // `TypeIndex` 77 is `unitrules.xml`'s record 27 (the table starts at
    // `0x32`); 78, the Spanish line, would be 28.
    let esf = built
        .sim
        .units
        .iter()
        .find(|u| u.alive() && u.owner == 0 && u.index == 22)
        .expect("0/22 on 700");
    assert_eq!(esf.ty, Some(77 - 50), "0/22 is the generic Elite Special Forces");
    assert_eq!(st.applied.ran, MELEE_STAGED - 2, "every other line ran");
}

/// run1468: the staged game, the draw stream to its end at `cover=0`, the
/// melee's records on a stride of 100.
pub(crate) const MELEE_LONG: (&str, &str) = (
    "gamelog-run1468-greatsahara-information-melee-trace.txt",
    "rontrace-run1468.log",
);

pub(crate) use super::gunpowder_melee::Melee;

pub(crate) const INFORMATION_MELEE: Melee = Melee {
    length: 2600,
    // Item 3797, first try: FIRST_TRY.
    count: 0,
    sequence: 0,
};

/// The melee's word: the lower of the two partings.
pub(crate) const INFORMATION_MELEE_WORD: i64 = if INFORMATION_MELEE.count < INFORMATION_MELEE.sequence {
    INFORMATION_MELEE.count
} else {
    INFORMATION_MELEE.sequence
};

/// **Where the third melee's draw stream first parts** (item 3797): the
/// harness walks run1467's start with the script staged and compares every
/// frame's draws against run1468's trace, the count and the sequence.
#[test]
fn information_melee_first_parting() {
    let _pins = Pins::hold();
    let m = &INFORMATION_MELEE;
    let Some(w) = crate::diff::setup::with_staged_script(melee_script(), || {
        third::walk_from(MELEE_START, MELEE_LONG)
    }) else {
        return;
    };
    eprintln!(
        "INFORMATION_MELEE: count {} sequence {} of {}; {}",
        w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.last, m.length, "the game's last frame");
    pin_eq!(w.count, m.count, "the draw count's first parting");
    pin_eq!(w.sequence, m.sequence, "the draw sequence's first parting");
}

/// The word's window on run1468's stride: the block before the word to
/// the block after it.
pub(crate) const WIDENING_INFORMATION_MELEE: (i64, i64) = (0, 0);

/// The word's window, widened whole on run1468's stride blocks, the rounds
/// compared — the coverage driver's slice too.
pub(crate) fn information_melee_word_window() -> Option<harness::tests::Widened> {
    crate::diff::setup::with_staged_script(melee_script(), || {
        harness::tests::widen_on_siblings_with_rounds(
            &[MELEE_START],
            true,
            MELEE_LONG,
            "run1468",
            &[(MELEE_LONG.0, WIDENING_INFORMATION_MELEE.0)],
            WIDENING_INFORMATION_MELEE,
            1,
            &[],
            true,
        )
    })
}

/// **The third melee's word window, widened whole** (item 3797): every
/// dumped record on run1468's stride blocks over
/// [`WIDENING_INFORMATION_MELEE`], both directions, the rounds too.
/// `RON_FIRSTS=1` prints every row.
#[test]
fn information_melee_run1468_s_word_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = information_melee_word_window() else {
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
    // Item 3797, first try: FIRST_WIDENING.
    pin_eq!(w.blocks, 0, "every stride block in the window");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    pin_eq!(w.firsts.len(), 0, "run1468's word window");
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **Every seat's leader on the start dump's frames** (item 3797, the
/// shape item 3791 gave the first two, `docs/GOLDEN.md` §151): run1467's frames 1 and 37 print all three
/// seats' `LEADERDATA` whole. run1468's only other leader records are its
/// closing dump at 2601, past the word, so they are not compared while the
/// game parts.
#[test]
fn information_melee_s_leaders_are_compared_on_every_seat() {
    let _pins = Pins::hold();
    let Some(w) = crate::diff::gunpowder_melee::seat_walk(
        melee_script,
        MELEE_START,
        MELEE_LONG,
        &[MELEE_START],
        false,
    ) else {
        return;
    };
    w.report("run1467");
    pin_eq!(w.players, 3, "the seats the lobby seats");
    pin_eq!(
        w.records.clone(),
        vec![2, 2, 2],
        "leader records compared, by seat"
    );
    // Item 3797, first try: FIRST_SEATS.
    pin_eq!(w.standing(), vec![12, 9, 9], "standing keys, by seat");
    pin_eq!(
        w.rest(),
        Vec::<String>::new(),
        "the keys that part, by seat"
    );
}
