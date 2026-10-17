// Golden case bodies. Wrappers in golden.rs preserve libtest names.
use super::*;

pub(super) fn chapter_eighty_staging_reaches_the_stealth_arm() {
    let Some(mut s) = stage_script("ch76", "chapter80") else {
        return;
    };
    let at = |s: &mut Staged4, to: i64| {
        while s.built.sim.frame < to {
            s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
            s.built.tick();
        }
    };
    let find = |s: &Staged4, who: u8, o: i16| {
        s.built
            .sim
            .units
            .iter()
            .position(|u| u.alive() && u.owner == who && u.index == o)
            .expect("a staged unit")
    };
    let probe = |s: &Staged4, who: u8, o: i16| {
        let sim = &s.built.sim;
        let i = find(s, who, o);
        let u = &sim.units[i];
        (
            sim.world
                .danger_half(1, sim::world::Cell::new(u.pos.cell().x, u.pos.cell().y)),
            sim.unit_stealth_hides(i, 1),
            sim.world_sees(u.pos, 1),
            u.orders.len(),
        )
    };
    at(&mut s, 1001);
    // (danger for leader 1 at the unit's half-cell, hidden by the arm, lit,
    // orders)
    assert_eq!(
        probe(&s, 0, 6),
        (60, true, true, 0),
        "an idle Partisan, lit, undetected"
    );
    assert_eq!(
        probe(&s, 0, 7),
        (120, false, true, 0),
        "the second Partisan before the Scout: it fought the Airbase on 612, so it is blown (item 2785)"
    );
    let (_, hides, lit, _) = probe(&s, 0, 8);
    assert!(
        !hides && lit,
        "the Hoplites are the control: lit, never hidden"
    );
    at(&mut s, 1198);
    assert!(
        !probe(&s, 0, 7).1,
        "the Scout's disc reaches the second Partisan"
    );
    assert!(probe(&s, 0, 6).1, "and not the first");
    at(&mut s, 1201);
    assert_eq!(probe(&s, 0, 7).0, 120, "weighed whole at the 1200 pass");
    assert_eq!(probe(&s, 0, 6).0, 60, "still half");
}

pub(super) fn chapter_eighty_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch80", "chapter80", 80, 6, 1204) else {
        return;
    };
    eprintln!(
        "chapter eighty: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    // The draw stream agrees to the capture's end; the state word is
    // `GOLDEN_WORD_CHAPTER_EIGHTY`, the widening's (`docs/GOLDEN.md` §102).
    let _ = GOLDEN_WORD_CHAPTER_EIGHTY;
    assert!(w.word >= 1204, "chapter eighty's draws part before 1204");
}

pub(super) fn chapter_eighty_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch80",
        "chapter80",
        (1198, 1204),
        0,
        0,
        (1199, 1204),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch80 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch80: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH80.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch80: what parts in the window moved");
}

/// **run878 whole, both directions** (item 2785, `docs/GOLDEN.md` §103): the
/// 1000 pass's own blocks, 998–1001 — block 1002 is not in the dump.
pub(super) fn chapter_eighty_w_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch80w",
        "chapter80w",
        (998, 1002),
        0,
        0,
        (999, 1002),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch80w f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch80w: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH80W.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch80w: what parts in the window moved");
}

pub(super) fn chapter_eighty_five_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch85", "chapter85", 85, 10, 1217) else {
        return;
    };
    eprintln!(
        "chapter eighty-five: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_eighty_five_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch85",
        "chapter85",
        (1100, 1219),
        0,
        0,
        (1101, 1219),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch85 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch85: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH85.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch85: what parts in the window moved");
}

pub(super) fn chapter_eighty_six_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch86", "chapter86", 86, 6, 1299) else {
        return;
    };
    eprintln!(
        "chapter eighty-six: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_eighty_six_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch86",
        "chapter86",
        (1100, 1300),
        0,
        0,
        (1101, 1300),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch86 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch86: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH86.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch86: what parts in the window moved");
}

pub(super) fn chapter_eighty_eight_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch88", "chapter88", 88, 8, 1500) else {
        return;
    };
    eprintln!(
        "chapter eighty-eight: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_eighty_eight_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch88",
        "chapter88",
        (1100, 1500),
        0,
        0,
        (1101, 1500),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch88 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch88: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH88.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch88: what parts in the window moved");
}

pub(super) fn chapter_eighty_nine_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch89", "chapter89", 89, 6, 1500) else {
        return;
    };
    eprintln!(
        "chapter eighty-nine: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_eighty_nine_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch89",
        "chapter89",
        (1100, 1500),
        0,
        0,
        (1101, 1500),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch89 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch89: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH89.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch89: what parts in the window moved");
}

pub(super) fn chapter_ninety_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch90", "chapter90", 90, 14, 1200) else {
        return;
    };
    eprintln!(
        "chapter ninety: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_ninety_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch90",
        "chapter90",
        (605, 1200),
        0,
        0,
        (606, 1200),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch90 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch90: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH90.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch90: what parts in the window moved");
}

pub(super) fn chapter_eighty_seven_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch87", "chapter87", 87, 6, 1500) else {
        return;
    };
    eprintln!(
        "chapter eighty-seven: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_eighty_seven_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch87",
        "chapter87",
        (1100, 1500),
        0,
        0,
        (1101, 1500),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch87 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch87: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH87.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch87: what parts in the window moved");
}
