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
    assert_eq!(
        esf.ty,
        Some(77 - 50),
        "0/22 is the generic Elite Special Forces"
    );
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
    // Item 3797, first try: **623 of 2600**, the count and the sequence
    // together, thirteen frames past the first `add` (610). Ours 10 draws
    // against 9: index 0 agrees, and at index 1 ours spends a second
    // `Unit::fight+0x9b0` — the French Stealth Bomber `1/17`'s, behind
    // the Jet Fighter `1/14`'s (TI 295, air) — where theirs spends
    // `Farms::inc_time+0x1ae`. Theirs' `1/14` stands on its spawn through
    // block 701, `idle 1`, its attack `in_range 0` `new_ord 1`, so the
    // strike ours adds is the Jet Fighter's (`docs/GOLDEN.md` §152).
    //
    // Item 3800: **623 → 628**. The extra roll was the Stealth Bomber's:
    // `1/17`, `add`ed on 622, took the Attack Helicopter `2/15` on its
    // first idle and rolled `fight`'s re-search on 623, where the
    // original's `valid_target_const` refuses a Bomber-line searcher a
    // helicopter target (`006474ee`, §152.1). On 628 ours spends 14
    // against 15: index 2 is both sides' `Unit::fight+0x9b0` (ours the
    // Attack Helicopter `1/15`, TI 311), and at index 3 theirs spends a
    // second `fight` roll where ours spends `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d` (`2/18`'s birth idle) — theirs acted.
    //
    // Item 3802: **628 → 640**. Theirs' extra roll was the Jet Fighter
    // `1/14`'s, which held an attack on the Russian Jet Fighter `2/14`
    // because `2/14`'s guided round struck it a frame sooner than ours:
    // `Ammo::inc_time`'s spline arm re-plans a missile every fourth frame
    // and closes it on the first point inside the target's `target_size`
    // (§152.2), where ours flew one arc. run1469's window (620..640)
    // agrees on that round's every field. On 640 ours spends 45 against
    // 47: at index 37 theirs spends `Ammo::do_damage+0xc59` where ours
    // spends `Farms::inc_time+0x1ae` — a round lands in theirs.
    //
    // Item 3804: **640 → 648**. Theirs' round was the Attack Helicopter
    // `2/15`'s second rocket on `1/15` (both `TI` 311): it re-planned to
    // five points on 636 where ours re-planned to six, so it landed on 640
    // and ours flew on. Three launch inputs parted (§152.3): the
    // helicopters flew 30 low from birth (`Unit::init`'s
    // `set_new_location(·, 1, 1)` climbs), an air target was aimed at the
    // ground and not its figure's `z`, and piece 261's rockets left the
    // figure's point at ground + 100 where theirs leave its pods at figure
    // + 41. run1469's window agrees on every `2/15` and `2/14` round's
    // launch, landing and clock, 620..639. On 648 ours spends 44 against
    // 43: at index 35 ours spends `Unit::close+0xcb6` where theirs spends
    // `Farms::inc_time+0x1ae` — ours acted.
    count: 648,
    sequence: 648,
};

/// The melee's word: the lower of the two partings.
pub(crate) const INFORMATION_MELEE_WORD: i64 =
    if INFORMATION_MELEE.count < INFORMATION_MELEE.sequence {
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
/// 250 frames past it (item 3802: the word 640, so 601..890 — the same
/// three blocks as 3797's 601..873 and 3800's 601..878).
pub(crate) const WIDENING_INFORMATION_MELEE: (i64, i64) = (601, 890);

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
    // Item 3797, first try (window 601..873, blocks 601, 701 and 801):
    // **536** keys, rounds read (19, 20), 49 parting. Block 601 holds the
    // standing families alone (`build` 41, `form` 40, the groups' and a
    // pool's). The state first parts on block 701: the French Stealth
    // Bomber `1/17` (TI 306, air) at (16344, 15720) in ours against
    // (15960, 15384), `damage_frame` 644 against 682, `hits_left` 328
    // against 287; the Russian Attack Helicopter `2/15` (TI 311)
    // `damage_frame` 664 against 698; the Advanced Fighter `2/16` (TI
    // 301) last struck by who 1 in ours and by who 0 in theirs.
    //
    // Item 3800 (window 601..878, the same three blocks): **536 → 385**
    // keys, rounds read (19, 20) → (19, 17), 49 → 43 parting. Block 601
    // is standing-only still. On 701 the Stealth Bomber `1/17` holds no
    // order in ours (`orders.len` 0, at (16344, 14808)) where theirs'
    // attacks the Partisan `0/17` in range at (15960, 15384),
    // `recharging` 13 against 0 — the helicopter it took before is the
    // route ours no longer has, and its ground target is the next word's.
    //
    // Item 3802 (window 601..890, the same three blocks): **385 → 449**
    // keys, rounds read (19, 17) → (19, 15), 43 → 51 parting — a game
    // that holds to 640 now and parts 61 frames before block 701. Block
    // 601 is standing-only still (84 keys). On 701 (128 keys) `1/17`
    // holds no order as before, the Jet Fighter `1/14` holds none against
    // theirs' one (`idle` 0 against 1, `damage_frame` 667 against 668), and
    // the helicopters' figures fly 30 low (`1/15` `g.z` 690 against 720,
    // `2/15` 360 against 390). The word's own value diff is run1469's
    // per-frame window, not these stride blocks (GOLDEN §152.2).
    pin_eq!(w.blocks, 3, "every stride block in the window");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    pin_eq!(w.firsts.len(), 449, "run1468's word window");
    pin_eq!(w.rounds_read, (19, 15), "the rounds read");
    pin_eq!(w.rounds.len(), 51, "the rounds that part");
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
    // Item 3797, first try: the standing family (`SITE[i].reg`,
    // `aggression[j]`) on both frames and every seat, and nothing else —
    // 12, 9 and 7 keys (the Russians' 7 against the Egyptians' 9 in
    // §150's lobby).
    pin_eq!(w.standing(), vec![12, 9, 7], "standing keys, by seat");
    pin_eq!(
        w.rest(),
        Vec::<String>::new(),
        "the keys that part, by seat"
    );
}
