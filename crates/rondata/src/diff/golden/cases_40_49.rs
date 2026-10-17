// Golden case bodies. Wrappers in golden.rs preserve libtest names.
use super::*;

pub(super) fn chapter_forty_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch40", "chapter40", 40, 21, 1149) else {
        return;
    };
    eprintln!(
        "chapter forty: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FORTY,
        "chapter forty's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FORTY}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FORTY,
        "chapter forty's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §49"
    );
}

pub(super) fn chapter_forty_one_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch41", "chapter41", 41, 15, 1769) else {
        return;
    };
    eprintln!(
        "chapter forty-one: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FORTY_ONE,
        "chapter forty-one's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FORTY_ONE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FORTY_ONE,
        "chapter forty-one's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §50"
    );
}

pub(super) fn chapter_forty_two_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch42", "chapter42", 42, 18, 1159) else {
        return;
    };
    eprintln!(
        "chapter forty-two: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FORTY_TWO,
        "chapter forty-two's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FORTY_TWO}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FORTY_TWO,
        "chapter forty-two's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §51"
    );
}

pub(super) fn chapter_forty_three_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch43", "chapter43", 43, 13, 2199) else {
        return;
    };
    eprintln!(
        "chapter forty-three: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FORTY_THREE,
        "chapter forty-three's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FORTY_THREE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FORTY_THREE,
        "chapter forty-three's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §52"
    );
}

pub(super) fn chapter_forty_four_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch44", "chapter44", 44, 7, 1449) else {
        return;
    };
    eprintln!(
        "chapter forty-four: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FORTY_FOUR,
        "chapter forty-four's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FORTY_FOUR}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FORTY_FOUR,
        "chapter forty-four's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §53"
    );
}

pub(super) fn chapter_forty_five_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch45", "chapter45", 45, 7, 1649) else {
        return;
    };
    eprintln!(
        "chapter forty-five: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FORTY_FIVE,
        "chapter forty-five's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FORTY_FIVE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FORTY_FIVE,
        "chapter forty-five's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §54"
    );
}

pub(super) fn chapter_forty_six_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch46", "chapter46", 46, 18, 2199) else {
        return;
    };
    eprintln!(
        "chapter forty-six: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FORTY_SIX,
        "chapter forty-six's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FORTY_SIX}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FORTY_SIX,
        "chapter forty-six's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §55"
    );
}

pub(super) fn chapter_forty_seven_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch47", "chapter47", 47, 13, 1399) else {
        return;
    };
    eprintln!(
        "chapter forty-seven: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FORTY_SEVEN,
        "chapter forty-seven's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FORTY_SEVEN}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FORTY_SEVEN,
        "chapter forty-seven's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §56"
    );
}

pub(super) fn chapter_forty_eight_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch48", "chapter48", 48, 17, 1749) else {
        return;
    };
    eprintln!(
        "chapter forty-eight: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FORTY_EIGHT,
        "chapter forty-eight's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FORTY_EIGHT}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FORTY_EIGHT,
        "chapter forty-eight's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §57"
    );
}

pub(super) fn chapter_forty_nine_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch49", "chapter49", 49, 15, 1499) else {
        return;
    };
    eprintln!(
        "chapter forty-nine: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FORTY_NINE,
        "chapter forty-nine's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FORTY_NINE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FORTY_NINE,
        "chapter forty-nine's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §58"
    );
}

pub(super) fn chapter_forty_nine_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch49",
        "chapter49",
        WIDENING_CHAPTER_FORTY_NINE,
        1499,
        0,
        (933, 936),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch49 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch49: {} rows", got.len());
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch49 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH49.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch49: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH49_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: ["622 slot 1 ox", "622 slot 1 oy", "702 slot 0 … → ["702 slot 0 new_speed", "702 slot 0 speed"]; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch49: what parts in the pool moved");
}

pub(super) fn chapter_forty_one_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch41",
        "chapter41",
        WIDENING_CHAPTER_FORTY_ONE,
        1769,
        0,
        (620, 626),
        true,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch41 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch41: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH41.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch41: what parts under the word moved");
}

pub(super) fn chapter_forty_two_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch42",
        "chapter42",
        WIDENING_CHAPTER_FORTY_TWO,
        1159,
        0,
        (620, 626),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch42 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch42: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH42.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch42: what parts under the word moved");
}

pub(super) fn chapter_forty_four_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch44",
        "chapter44",
        WIDENING_CHAPTER_FORTY_FOUR,
        1449,
        0,
        (668, 672),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch44 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch44: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH44.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch44: what parts under the word moved");
}

pub(super) fn chapter_forty_five_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch45",
        "chapter45",
        WIDENING_CHAPTER_FORTY_FIVE,
        1649,
        0,
        (799, 806),
        false,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch45 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch45: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH45.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch45: what parts under the word moved");
}

pub(super) fn chapter_forty_six_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch46",
        "chapter46",
        WIDENING_CHAPTER_FORTY_SIX,
        2199,
        0,
        (1899, 1903),
        false,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch46 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch46: {} rows", got.len());
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch46 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH46.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch46: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH46_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    pin_eq!(got_pool, want_pool, "ch46: what parts in the pool moved");
}

pub(super) fn chapter_forty_seven_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch47",
        "chapter47",
        WIDENING_CHAPTER_FORTY_SEVEN,
        1399,
        0,
        (1099, 1103),
        false,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch47 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch47: {} rows", got.len());
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch47 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH47.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch47: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH47_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: ["622 slot 0 ox", "622 slot 0 oy"] → []; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch47: what parts in the pool moved");
}

pub(super) fn chapter_forty_eight_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch48",
        "chapter48",
        WIDENING_CHAPTER_FORTY_EIGHT,
        1749,
        0,
        (1449, 1452),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch48 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch48: {} rows", got.len());
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch48 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH48.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch48: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH48_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: ["642 slot 1 ox", "642 slot 1 oy", "904 slot 0 … → []; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch48: what parts in the pool moved");
}

pub(super) fn chapter_forty_three_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch43",
        "chapter43",
        WIDENING_CHAPTER_FORTY_THREE,
        2199,
        0,
        (1899, 1903),
        false,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch43 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch43: {} rows", got.len());
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch43 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH43.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch43: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH43_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    pin_eq!(got_pool, want_pool, "ch43: what parts in the pool moved");
}

pub(super) fn chapter_forty_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch40",
        "chapter40",
        WIDENING_CHAPTER_FORTY,
        1149,
        0,
        (630, 640),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch40 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch40 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    eprintln!("ch40: {} rows, {} pool rows", got.len(), got_pool.len());
    let mut want: Vec<String> = WANT_CH40.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch40: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH40_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: 23 rows → 7 rows; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch40: what parts in the pool moved");
}
