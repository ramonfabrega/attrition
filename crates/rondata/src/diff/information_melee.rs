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
    // + 41; the release height is the one that, put back alone, returns
    // the word to 640. run1469's window agrees on every `2/15` and `2/14` round's
    // launch, landing and clock, 620..639. On 648 ours spends 44 against
    // 43: at index 35 ours spends `Unit::close+0xcb6` where theirs spends
    // `Farms::inc_time+0x1ae` — ours acted.
    //
    // Item 3806: **648 → 676**. Ours' death was the Anti-Aircraft
    // Missile `1/20`'s (TI 284), struck by the Cruise Missile `2/19`'s
    // (TI 314) round, fired on 645 on both sides: ours flew it four
    // frames on the straight line from the figure's (16632, 16632, 672),
    // where theirs leaves the unit's point plus `MISSILE_OFFSET`,
    // (16523, 16633, 958), on `calc_nuke_spline`'s Cruise Missile arm —
    // a cubic over the ground at 250 up, 29 points, 28 frames (§152.4).
    // run1470's window (640..665) agrees on that round's launch, landing,
    // clock and four re-plans (28, 11, 9, 6, 4), blocks 646..661. On 676
    // ours spends 22 against 23: at index 13 theirs spends
    // `Unit::fight+0x9b0` where ours spends `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271` — theirs acted.
    //
    // Item 3808: **676 → 713**. Theirs' extra roll was the Jet Fighter
    // `1/14`'s (TI 295) on `2/14`, which it held because `2/14`'s second
    // guided round struck it on tick 675; ours landed it on 674. That
    // round's first re-plan counted 10 points against theirs' 11, from a
    // launch spline whose middle control point ours rotated by the
    // figure's 120° on the sine table, where theirs turns `(0, −750, 0)`
    // as floats through two quaternions at `fast_angle_to_degrees`' degree
    // negated and folded — 121° (`Ammo::init`'s `67cb1f`–`67cd00`,
    // §152.5). run1471's window (660..701) agrees on that round's spline,
    // its re-plans (13, 11, 8, 5) and `1/14`'s `damage` 86 on block 676.
    // On 713 ours spends 27 against 28: at index 4 theirs spends
    // `Unit::fight+0x9b0` where ours spends `Guy::set_anim+0xf2f <
    // Unit::set_anim+0x56` — theirs acted.
    //
    // Item 3810: **713 → 724**. Theirs' extra roll was `1/14`'s research
    // again: its 712 search re-found the Partisan `0/17` (uid 24), hit on
    // 711 by the Main Battle Tank `1/21`'s round of 707, and kept an
    // attack into 713; ours' round flew 5 frames from the hull's square and
    // struck on 712, after the search, so `1/14` re-found `2/14`, dropped
    // it at the refused chase, and stood. Piece 201's muzzle (§152.6,
    // run1472's packet) launches it from (17878, 14908, 610) with
    // `total_time` 4, run1472's blocks 709–711 to the unit. On 724 ours
    // spends 16 against 17: at index 1 theirs spends `Guy::set_anim+0x97a
    // < Unit::move_step+0x823` where ours spends `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271`.
    //
    // Item 3813: **724 → 755**. Theirs' extra stand was the Elite Special
    // Forces `0/22`'s, refused a step by `0/20`'s cell (run1473's
    // `RON_COLLIDE_PROBE`); ours' `0/22` walked a tick behind, struck on
    // 718 against 717 and fleeing a frame late, because the Advanced
    // Machine Gun `2/25`'s pair of 712 and 713 left its square and flew 7
    // frames. Piece 77's muzzle (§152.7, run1472's packet; frames 3 and 4
    // of `CHAR_ATTACK3`) launches them from (20432, 18244, 554) and
    // (20434, 18245, 555) in 6, run1473's `AMMO` to the unit, and `0/22`
    // stands on (18438, 15672) on 724 as theirs' block does. On 755 ours
    // spends 12 against 13: at index 4 theirs spends `Unit::close+0xcb6`
    // where ours spends `Farms::inc_time+0x1ae`.
    //
    // Item 3814: **755 → 718, a regression this landing names**. Theirs'
    // death on 755 was the Main Battle Tank `2/21`'s, 47 hit points down
    // on ours since 690: the Nubian V2 `0/16`'s round of 671 re-plans at
    // its fourth point and lands on 690 in theirs, and flew its 120 frames
    // in ours. `calc_nuke_spline`'s other arm (§152.8, run1471's block 672
    // to the bit) lays it, and `2/21` takes theirs' 49 on 690. But the
    // ammo pool's slots part on 642 (air rounds' release points and
    // flight times, unbuilt), so ours' V2 holds slot 2 where theirs holds
    // 6, and on 690 it lands before the Partisan `0/17`'s round on `2/24`
    // (slot 6 against theirs' 5): its splash of 1 comes first and
    // overkill cuts `0/17`'s 43 to 14, so `2/24` reads 15 against 44 and
    // does not die on 718. Landed last (`RON_PROBE_3814`, reverted whole),
    // `2/24` reads 44 and the word is 761. On 718 ours spends 14 against
    // 15: at index 6 theirs spends `Unit::close+0xcb6` where ours spends
    // `Farms::inc_time+0x1ae`.
    //
    // Item 3816: **718 → 761**. The pool's slots parted on 646, not in
    // the release points: the Cruise Missile `2/19`'s round took slot 6 in
    // theirs because the Anti-Aircraft Missile `1/20`'s round on `2/18`
    // still held slot 2 — run1469's block 639 lays it eight points, run1470
    // re-plans it to eight on 642 and to three on 646 — where ours laid the
    // land shooter's middle point half way on the straight line, seven
    // points, re-planned to five and freed slot 2 by 645. A land shooter's
    // (and a building's) middle point is `attack_dist / 2` out at 45° up
    // on the ammo's angle (`Ammo::init`'s `67c437`–`67c469`, §152.9;
    // block 639's point to the bit). So the V2 takes slot 6 and `0/17`'s
    // round slot 5, and `2/24` takes 44 on 690 and dies on 718 as theirs'
    // does. On 761 ours spends 14 against 13: at index 1 ours spends
    // `Guy::set_anim+0x97a < Unit::move_step+0x823` where theirs spends
    // `Guy::set_anim+0x97a < Guy::do_turn+0x4a < Unit::move_step+0x389`.
    count: 761,
    sequence: 761,
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
/// three blocks as 3797's 601..873 and 3800's 601..878; item 3806: the
/// word 676, so 601..926, which takes block 901 in; item 3808: the word
/// 713, so 701..963, which lets block 601 go; item 3810: the word 724, so
/// 701..974, the same three blocks; item 3813: the word 755, so 701..1005,
/// which takes block 1001 in; item 3814: the word 718, so 701..968,
/// which lets block 1001 go; item 3816: the word 761, so 701..1011, which
/// takes block 1001 back in).
pub(crate) const WIDENING_INFORMATION_MELEE: (i64, i64) = (701, 1011);

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
    //
    // Item 3804 (the same window and blocks; the word 648): **449 → 388**
    // keys, rounds read (19, 15), 51 → 47 parting. Block 601 is
    // standing-only still (84 keys), 701 holds 113 and 801 191. The
    // helicopters' `g.z` rows have left 701 (`1/15` and `2/15` no longer
    // 30 low): their birth climb (§152.3). On 701 `1/15` carries 12 of
    // damage against 6, `1/14` and `1/17` hold no order against theirs'
    // one as before. The word's value diff is run1469's window (§152.3).
    //
    // Item 3806 (the word 676, so 601..926: block 901 joins). On the old
    // three blocks **388 → 325** keys, rounds read (19, 15) → (19, 14),
    // 47 → 41 parting — rows leaving: the Cruise Missile `2/19`'s round
    // on `1/20` flies theirs' 28 frames, so `1/20` stands to 661 as
    // theirs' does. With 901: 480 keys, rounds read (33, 16), 46
    // parting. The word's value diff is run1470's window (§152.4).
    //
    // Item 3808 (the word 713, so 701..963: block 601 leaves, blocks
    // 701, 801 and 901). **480 → 437** keys, rounds read (33, 16) →
    // (33, 13), 46 → 47 parting. Block 701 holds 118 keys, 801 145 and
    // 901 174. On 701 the standing families aside, the rows are blows a
    // frame either side and their tallies: `1/17`, `1/18` and `2/21`
    // struck on 683, 683 and 686 against 682, 682 and 685; `2/18` on
    // 675 against 676; `2/22` and `2/23` not yet struck in ours against
    // 690. The French Jet Fighter `1/14` no longer holds a missing order
    // on 701 — the lead's rotation (§152.5). The word's value diff is
    // run1471's window (§152.5).
    //
    // Item 3810 (the word 724, so 701..974, the same three blocks). **437
    // → 406** keys, rounds read (33, 13) → (33, 16), 47 → 49 parting.
    // Block 701 holds its 118 keys unchanged, 801 145 → 103 and 901 174 →
    // 185: the Main Battle Tanks' rounds leave their muzzles (piece 201,
    // §152.6) from 707 on. The word's value diff is run1472's window
    // (700..714): `1/21`'s round of 707 at (17878, 14908, 610), four
    // frames, and `1/14`'s attack on `0/17` on block 713.
    //
    // Item 3813 (the word 755, so 701..1005: block 1001 joins). **406 →
    // 674** keys over four blocks, rounds read (33, 16) → (45, 17), 49 →
    // 47 parting. Block 701 holds 117 keys (118), 801 135 (103), 901 190
    // (185) and 1001 232: the Advanced Machine Guns' rounds leave their
    // muzzles (piece 77, §152.7) from the first volley on. The word's
    // value diff is run1473's window (714..730): `2/25`'s pair of 712 and
    // 713 at (20432, 18244, 554) and (20434, 18245, 555), six frames, and
    // `0/22` at (18438, 15672) on block 724.
    //
    // Item 3814 (the word 718, so 701..968: block 1001 leaves). **674 →
    // 428** keys over three blocks, rounds read (45, 17) → (33, 14), 47
    // → 53 parting. Block 701 holds 108 keys (117), 801 174 (135) and 901
    // 146 (190). On 701 the Main Battle Tank `2/21`'s standing
    // `hits:damage` (ours 85 against 132) is gone: the V2 `0/16`'s round
    // re-plans and lands on 690 (§152.8). `2/24` stands in its place at
    // 15 against 44, `damage_o` 16 against 17 — the V2's splash first on
    // a slot the pool parted on 642. On 801 `2/21` dies on 754 against
    // 755 and `2/24` on 721 against 718. The word's value diff is
    // run1471's window: `0/16`'s spline on block 672, its re-plan to 12
    // points on 675, and `2/21`'s `damage` 105 on block 691.
    //
    // Item 3816 (the word 761, so 701..1011: block 1001 joins again).
    // **428 → 469** keys over four blocks, rounds read (33, 14) → (45,
    // 13), 53 → 55 parting. Block 701 holds 108 keys, 801 52 (174), 901
    // 164 (146) and 1001 145. `2/24`'s 15 against 44 has left block 701,
    // and its death and `2/21`'s have left 801: the Anti-Aircraft
    // Missile `1/20`'s round holds slot 2 to 648 (§152.9), so the V2
    // lands after `0/17`'s round on 690. `2/18`'s standing `damage_frame`
    // 646 against 676 on 701 is the next pool parting's (its own round on
    // `1/18` flies 6 frames against 5 from its square). The word's value
    // diff is run1469's and run1470's windows: `1/20`'s round of 638,
    // eight points, re-planned to eight on 642 and three on 646, and
    // `2/24` at 221 of 265 on 691.
    pin_eq!(w.blocks, 4, "every stride block in the window");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    pin_eq!(w.firsts.len(), 469, "run1468's word window");
    pin_eq!(w.rounds_read, (45, 13), "the rounds read");
    pin_eq!(w.rounds.len(), 55, "the rounds that part");
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
