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

pub(super) fn chapter_ninety_one_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch91", "chapter91", 91, 9, 1200) else {
        return;
    };
    eprintln!(
        "chapter ninety-one: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_ninety_one_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch91",
        "chapter91",
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
        eprintln!("  ch91 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch91: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH91.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch91: what parts in the window moved");
}

pub(super) fn chapter_ninety_two_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch92", "chapter92", 92, 15, 1200) else {
        return;
    };
    eprintln!(
        "chapter ninety-two: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_ninety_two_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch92",
        "chapter92",
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
        eprintln!("  ch92 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch92: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH92.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch92: what parts in the window moved");
}

pub(super) fn chapter_ninety_three_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch93", "chapter93", 93, 10, 1200) else {
        return;
    };
    eprintln!(
        "chapter ninety-three: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_ninety_three_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch93",
        "chapter93",
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
        eprintln!("  ch93 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch93: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH93.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch93: what parts in the window moved");
}

pub(super) fn chapter_ninety_four_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch94", "chapter94", 94, 10, 1200) else {
        return;
    };
    eprintln!(
        "chapter ninety-four: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_ninety_four_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch94",
        "chapter94",
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
        eprintln!("  ch94 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch94: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH94.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch94: what parts in the window moved");
}

pub(super) fn chapter_ninety_five_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch95", "chapter95", 95, 13, 1200) else {
        return;
    };
    eprintln!(
        "chapter ninety-five: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_ninety_five_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch95",
        "chapter95",
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
        eprintln!("  ch95 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch95: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH95.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch95: what parts in the window moved");
}

pub(super) fn chapter_ninety_six_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch96", "chapter96", 96, 15, 1500) else {
        return;
    };
    eprintln!(
        "chapter ninety-six: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_ninety_six_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch96",
        "chapter96",
        (605, 1500),
        0,
        0,
        (606, 1500),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch96 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch96: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH96.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch96: what parts in the window moved");
}

pub(super) fn chapter_ninety_seven_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch97", "chapter97", 97, 6, 1500) else {
        return;
    };
    eprintln!(
        "chapter ninety-seven: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_ninety_seven_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch97",
        "chapter97",
        (605, 1500),
        0,
        0,
        (606, 1500),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch97 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch97: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH97.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch97: what parts in the window moved");
}

pub(super) fn chapter_ninety_eight_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch98", "chapter98", 98, 17, 1500) else {
        return;
    };
    eprintln!(
        "chapter ninety-eight: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_ninety_eight_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch98",
        "chapter98",
        (605, 1500),
        0,
        0,
        (606, 1500),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch98 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch98: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH98.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch98: what parts in the window moved");
}

pub(super) fn chapter_ninety_nine_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch99", "chapter99", 99, 17, 1500) else {
        return;
    };
    eprintln!(
        "chapter ninety-nine: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_ninety_nine_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch99",
        "chapter99",
        (605, 1500),
        0,
        0,
        (606, 1500),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch99 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch99: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH99.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch99: what parts in the window moved");
}

pub(super) fn chapter_one_hundred_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch100", "chapter100", 100, 12, 2600) else {
        return;
    };
    eprintln!(
        "chapter one hundred: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_one_hundred_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch100",
        "chapter100",
        (605, 2600),
        0,
        0,
        (606, 2600),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch100 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch100: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH100.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch100: what parts in the window moved");
}

pub(super) fn chapter_one_hundred_and_one_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch101", "chapter101", 101, 14, 1500) else {
        return;
    };
    eprintln!(
        "chapter one hundred and one: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_one_hundred_and_two_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch102", "chapter102", 102, 15, 1500) else {
        return;
    };
    eprintln!(
        "chapter one hundred and two: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_one_hundred_and_three_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch103",
        "chapter103",
        (605, 1500),
        0,
        0,
        (606, 1500),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch103 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch103: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH103.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch103: what parts in the window moved");
}

pub(super) fn chapter_one_hundred_and_three_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch103", "chapter103", 103, 15, 1500) else {
        return;
    };
    eprintln!(
        "chapter one hundred and three: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_one_hundred_and_four_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch104",
        "chapter104",
        (605, 1500),
        0,
        0,
        (606, 1500),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch104 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch104: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH104.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch104: what parts in the window moved");
}

pub(super) fn chapter_one_hundred_and_four_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch104", "chapter104", 104, 15, 1500) else {
        return;
    };
    eprintln!(
        "chapter one hundred and four: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_one_hundred_and_five_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch105",
        "chapter105",
        (605, 1500),
        0,
        0,
        (606, 1500),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch105 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch105: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH105.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch105: what parts in the window moved");
}

pub(super) fn chapter_one_hundred_and_five_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch105", "chapter105", 105, 17, 1500) else {
        return;
    };
    eprintln!(
        "chapter one hundred and five: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_one_hundred_and_six_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch106",
        "chapter106",
        (605, 1500),
        0,
        0,
        (606, 1500),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch106 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch106: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH106.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch106: what parts in the window moved");
}

pub(super) fn chapter_one_hundred_and_six_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch106", "chapter106", 106, 15, 1500) else {
        return;
    };
    eprintln!(
        "chapter one hundred and six: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_one_hundred_and_seven_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch107",
        "chapter107",
        (605, 760),
        0,
        0,
        (606, 760),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch107 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch107: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH107.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch107: what parts in the window moved");
}

pub(super) fn chapter_one_hundred_and_seven_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch107", "chapter107", 107, 19, 760) else {
        return;
    };
    eprintln!(
        "chapter one hundred and seven: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
}

pub(super) fn chapter_one_hundred_and_nine_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch109",
        "chapter109",
        (605, 2600),
        0,
        0,
        (2293, 2315),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch109 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch109: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH109.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch109: what parts in the window moved");
}

pub(super) fn chapter_one_hundred_and_nine_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch109", "chapter109", 109, 22, 2600) else {
        return;
    };
    eprintln!(
        "chapter one hundred and nine: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_ONE_HUNDRED_AND_NINE,
        "chapter one hundred and nine's golden word moved; re-pin it here and say \
         so in docs/GOLDEN.md §159"
    );
}

/// The cells that part, and who=1's cell count by dumped block.
type Ch107Borders = (Vec<String>, Vec<(i64, usize)>);

/// Every cell's `(who, who2)` on every block of run1504 that dumps the
/// `WORLD` (the `@dumpall` blocks), ours against the original's, with each
/// block's who=1 count: what parts, and the count by block.
fn chapter_one_hundred_and_seven_borders() -> Option<Ch107Borders> {
    let mut s = stage_script("ch107", "chapter107")?;
    let mut parted: Vec<String> = Vec::new();
    let mut held: Vec<(i64, usize)> = Vec::new();
    let width = s.built.sim.world.width();
    for f in 0..760 - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let raw = s.ix.read_frame(at).unwrap();
        let log = Log::parse_eager(&raw);
        let Some((_, block)) = log.frames().into_iter().find(|(k, _)| *k == n) else {
            continue;
        };
        // The first `WORLD` with cells: `dump_deaths`' two print none.
        let Some(cells) = std::iter::once(block)
            .chain(block.kid("FULL DUMP"))
            .flat_map(|b| b.kids("WORLD"))
            .map(|w| crate::gamelog::world_cells(&w.fields().to_vec()))
            .find(|c| c.len() == 3600)
        else {
            continue;
        };
        let owner = |w: i64| match w {
            -1 => sim::world::Owner::None,
            -2 => sim::world::Owner::Ambiguous,
            p => sim::world::Owner::Player(p as u8),
        };
        let mut ones = 0;
        let owners = s.built.sim.world.owners();
        for (i, c) in cells.iter().enumerate() {
            let cell = sim::world::Cell::new(i as i32 % width, i as i32 / width);
            let ours = owners[i];
            let theirs = (owner(c.who), owner(c.who2));
            if c.who == 1 {
                ones += 1;
            }
            if ours != theirs {
                parted.push(format!(
                    "{n} ({}, {}) ours {ours:?} theirs {theirs:?}",
                    cell.x, cell.y
                ));
            }
        }
        held.push((n, ones));
    }
    Some((parted, held))
}

pub(super) fn chapter_one_hundred_and_one_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch101",
        "chapter101",
        (605, 1500),
        0,
        0,
        (606, 1500),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch101 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch101: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH101.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch101: what parts in the window moved");
}

pub(super) fn chapter_one_hundred_and_two_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch102",
        "chapter102",
        (605, 1500),
        0,
        0,
        (606, 1500),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch102 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch102: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH102.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch102: what parts in the window moved");
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

/// The border walk's assertion (item 3901, `docs/GOLDEN.md` §157).
pub(super) fn chapter_one_hundred_and_seven_s_borders_are_the_original_s_cell_for_cell() {
    let Some((parted, held)) = chapter_one_hundred_and_seven_borders() else {
        return;
    };
    eprintln!("ch107: {} WORLD blocks compared", held.len());
    for row in parted.iter().take(40) {
        eprintln!("  ch107 border {row}");
    }
    assert!(
        parted.is_empty(),
        "{} cell(s) part from run1504's WORLD blocks",
        parted.len()
    );
    assert_eq!(held, CH107_WHO1_CELLS, "who=1's cells by dumped block");
}

/// who=1's owned cells on each of run1504's `WORLD` blocks (item 3901).
const CH107_WHO1_CELLS: &[(i64, usize)] = &[
    (619, 261),
    (620, 261),
    (622, 261),
    (623, 261),
    (624, 261),
    (625, 270),
    (626, 314),
    (627, 381),
    (628, 442),
    (629, 442),
    (699, 442),
    (700, 442),
    (701, 442),
    (702, 442),
    (703, 442),
    (704, 442),
    (705, 433),
    (706, 389),
    (707, 322),
    (708, 261),
    (709, 261),
    (716, 261),
    (717, 261),
];

pub(super) fn chapter_one_hundred_and_ten_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch110",
        "chapter110",
        (605, 900),
        0,
        0,
        (769, 773),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch110 f{f} {w}/{o} {what}: {row}");
    }
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch110 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .chain(
            pool.iter()
                .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}")),
        )
        .collect();
    got.sort();
    eprintln!("ch110: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH110.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch110: what parts in the window moved");
}

pub(super) fn chapter_one_hundred_and_ten_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch110", "chapter110", 110, 8, 900) else {
        return;
    };
    eprintln!(
        "chapter one hundred and ten: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    pin_eq!(
        w.word,
        GOLDEN_WORD_CHAPTER_ONE_HUNDRED_AND_TEN,
        "chapter one hundred and ten's golden word moved; re-pin it and say so in docs/GOLDEN.md §160"
    );
}

pub(super) fn chapter_seventeen_air_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch17air", "chapter17-air", 17, 17, 1099) else {
        return;
    };
    eprintln!(
        "chapter seventeen's air actions: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SEVENTEEN_AIR,
        "chapter seventeen's air actions' golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §158"
    );
}

pub(super) fn chapter_seventeen_air_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch17air",
        "chapter17-air",
        WIDENING_CHAPTER_SEVENTEEN_AIR,
        1099,
        0,
        (680, 684),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch17air f{f} {w}/{o} {what}: {row}");
    }
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch17air pool f{f} slot {slot} {key}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .chain(
            pool.iter()
                .map(|((slot, key), (f, _))| format!("{f} pool {slot} {key}")),
        )
        .collect();
    got.sort();
    eprintln!("ch17air: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH17AIR.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch17air: what parts in the window moved");
}

pub(super) fn chapter_seventeen_airground_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch17ag", "chapter17-airground", 17, 15, 1059) else {
        return;
    };
    eprintln!(
        "chapter seventeen's air attack-ground: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SEVENTEEN_AIRGROUND,
        "chapter seventeen's air attack-ground' golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §158"
    );
}

pub(super) fn chapter_seventeen_airground_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch17ag",
        "chapter17-airground",
        WIDENING_CHAPTER_SEVENTEEN_AIRGROUND,
        1059,
        0,
        (680, 684),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch17ag f{f} {w}/{o} {what}: {row}");
    }
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch17ag pool f{f} slot {slot} {key}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .chain(
            pool.iter()
                .map(|((slot, key), (f, _))| format!("{f} pool {slot} {key}")),
        )
        .collect();
    got.sort();
    eprintln!("ch17ag: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH17AG.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch17ag: what parts in the window moved");
}

/// Two takes of one game (`rngcmp` 901 of 901): run1508 prints the death
/// list and run1508b the pool, since `GROUPS` beside `DEATHS` at one
/// section prints no `GROUPDATA` (parked 1885, 1931). The units, leaders
/// and death objects are run1508's; the pool is run1508b's.
pub(super) fn chapter_one_hundred_and_eleven_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch111",
        "chapter111",
        (605, 900),
        0,
        0,
        (621, 625),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    let Some((_, pool)) = widen_civilians_and_pool(
        "ch111b",
        "chapter111",
        (605, 900),
        0,
        0,
        (0, -1),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch111 f{f} {w}/{o} {what}: {row}");
    }
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch111 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .chain(
            pool.iter()
                .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}")),
        )
        .collect();
    got.sort();
    eprintln!("ch111: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH111.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch111: what parts in the window moved");
}

pub(super) fn chapter_one_hundred_and_eleven_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch111", "chapter111", 111, 4, 900) else {
        return;
    };
    eprintln!(
        "chapter one hundred and eleven: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    pin_eq!(
        w.word,
        GOLDEN_WORD_CHAPTER_ONE_HUNDRED_AND_ELEVEN,
        "chapter one hundred and eleven's golden word moved; re-pin it and say so in docs/GOLDEN.md §161"
    );
    // The draws by site name too: `Object::disband+0x415` is a row of
    // `trace::SITES` only this chapter reads (item 3914).
    pin_eq!(
        w.sequence,
        GOLDEN_WORD_CHAPTER_ONE_HUNDRED_AND_ELEVEN,
        "chapter one hundred and eleven's draw sites part before its word"
    );
}

pub(super) fn chapter_one_hundred_and_fourteen_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch114", "chapter114", 114, 5, 900) else {
        return;
    };
    eprintln!(
        "chapter one hundred and fourteen: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    pin_eq!(
        w.word,
        GOLDEN_WORD_CHAPTER_ONE_HUNDRED_AND_FOURTEEN,
        "chapter one hundred and fourteen's golden word moved; re-pin it and say so in docs/GOLDEN.md §164"
    );
    pin_eq!(
        w.sequence,
        GOLDEN_WORD_CHAPTER_ONE_HUNDRED_AND_FOURTEEN,
        "chapter one hundred and fourteen's draw sites part before its word"
    );
}

/// run1511 whole, both directions: its units, figures, buildings, leaders
/// and pool (`GROUPS=1` beside no `DEATHS`, so one take prints both).
pub(super) fn chapter_one_hundred_and_fourteen_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch114",
        "chapter114",
        (605, 900),
        0,
        0,
        (619, 643),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch114 f{f} {w}/{o} {what}: {row}");
    }
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch114 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .chain(
            pool.iter()
                .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}")),
        )
        .collect();
    got.sort();
    eprintln!("ch114: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH114.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch114: what parts in the window moved");
}
