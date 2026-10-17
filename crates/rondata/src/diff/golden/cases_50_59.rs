// Golden case bodies. Wrappers in golden.rs preserve libtest names.
use super::*;

pub(super) fn chapter_fifty_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch50", "chapter50", 50, 6, 1099) else {
        return;
    };
    eprintln!(
        "chapter fifty: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FIFTY,
        "chapter fifty's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FIFTY}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FIFTY,
        "chapter fifty's golden word moved; re-pin it here and say so in docs/GOLDEN.md §59"
    );
}

pub(super) fn chapter_fifty_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch50",
        "chapter50",
        WIDENING_CHAPTER_FIFTY,
        1099,
        0,
        (698, 701),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch50 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch50: {} rows", got.len());
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch50 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH50.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch50: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH50_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    pin_eq!(got_pool, want_pool, "ch50: what parts in the pool moved");
}

pub(super) fn chapter_fifty_one_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch51", "chapter51", 51, 6, 1099) else {
        return;
    };
    eprintln!(
        "chapter fifty-one: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FIFTY_ONE,
        "chapter fifty-one's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FIFTY_ONE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FIFTY_ONE,
        "chapter fifty-one's golden word moved; re-pin it here and say so in docs/GOLDEN.md §61"
    );
}

pub(super) fn chapter_fifty_one_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch51",
        "chapter51",
        WIDENING_CHAPTER_FIFTY_ONE,
        1099,
        0,
        (777, 780),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch51 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch51: {} rows", got.len());
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch51 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH51.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch51: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH51_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    pin_eq!(got_pool, want_pool, "ch51: what parts in the pool moved");
}

pub(super) fn chapter_fifty_three_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch53", "chapter53", 53, 10, 1119) else {
        return;
    };
    eprintln!(
        "chapter fifty-three: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FIFTY_THREE,
        "chapter fifty-three's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FIFTY_THREE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FIFTY_THREE,
        "chapter fifty-three's golden word moved; re-pin it here and say so in docs/GOLDEN.md §63"
    );
}

pub(super) fn chapter_fifty_three_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch53",
        "chapter53",
        WIDENING_CHAPTER_FIFTY_THREE,
        1119,
        0,
        (1099, 1101),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch53 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch53: {} rows", got.len());
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch53 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH53.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch53: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH53_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    pin_eq!(got_pool, want_pool, "ch53: what parts in the pool moved");
}

pub(super) fn chapter_fifty_five_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch55", "chapter55", 55, 6, 1010) else {
        return;
    };
    eprintln!(
        "chapter fifty-five: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FIFTY_FIVE,
        "chapter fifty-five's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FIFTY_FIVE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FIFTY_FIVE,
        "chapter fifty-five's golden word moved; re-pin it here and say so in docs/GOLDEN.md §67"
    );
}

pub(super) fn chapter_fifty_five_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch55",
        "chapter55",
        WIDENING_CHAPTER_FIFTY_FIVE,
        1010,
        0,
        (990, 992),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch55 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch55: {} rows", got.len());
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch55 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH55.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch55: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH55_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    pin_eq!(got_pool, want_pool, "ch55: what parts in the pool moved");
}

pub(super) fn chapter_fifty_six_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch56", "chapter56", 56, 7, 1010) else {
        return;
    };
    eprintln!(
        "chapter fifty-six: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FIFTY_SIX,
        "chapter fifty-six's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FIFTY_SIX}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FIFTY_SIX,
        "chapter fifty-six's golden word moved; re-pin it here and say so in docs/GOLDEN.md §68"
    );
}

pub(super) fn chapter_fifty_six_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch56",
        "chapter56",
        WIDENING_CHAPTER_FIFTY_SIX,
        1010,
        0,
        (990, 992),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch56 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch56: {} rows", got.len());
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch56 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH56.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch56: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH56_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    pin_eq!(got_pool, want_pool, "ch56: what parts in the pool moved");
}

pub(super) fn chapter_fifty_seven_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch57", "chapter57", 57, 7, 1010) else {
        return;
    };
    eprintln!(
        "chapter fifty-seven: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FIFTY_SEVEN,
        "chapter fifty-seven's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FIFTY_SEVEN}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FIFTY_SEVEN,
        "chapter fifty-seven's golden word moved; re-pin it here and say so in docs/GOLDEN.md §69"
    );
}

pub(super) fn chapter_fifty_seven_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch57",
        "chapter57",
        WIDENING_CHAPTER_FIFTY_SEVEN,
        1010,
        0,
        (990, 992),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch57 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch57: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH57.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch57: what parts under the word moved");
}

pub(super) fn chapter_fifty_nine_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch59", "chapter59", 59, 6, 760) else {
        return;
    };
    eprintln!(
        "chapter fifty-nine: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FIFTY_NINE,
        "chapter fifty-nine's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FIFTY_NINE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FIFTY_NINE,
        "chapter fifty-nine's golden word moved; re-pin it here and say so in docs/GOLDEN.md §71"
    );
}

pub(super) fn chapter_fifty_nine_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch59",
        "chapter59",
        WIDENING_CHAPTER_FIFTY_NINE,
        760,
        0,
        (758, 760),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch59 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch59: {} rows", got.len());
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch59 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH59.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch59: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH59_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    pin_eq!(got_pool, want_pool, "ch59: the pool's parting moved");
}

pub(super) fn chapter_fifty_two_s_fog_plane_agrees_through_the_first_round() {
    let _pins = Pins::hold();
    let Some((firsts, _pool)) = widen_civilians_and_pool(
        "ch52fog",
        "chapter51",
        (690, 714),
        714,
        0,
        (697, 700),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch52fog f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    // The eleven rows are the units' `form`, standing on the window's first
    // block (every chapter's); no fog cell parts on any of 25 frames.
    // Item 2384: 11 -> 13, the two `aggression` rows (parked 2385).
    pin_eq!(got.len(), 13, "ch52fog: keys that part");
    pin_eq!(
        got.iter().filter(|r| r.contains("seen2")).count(),
        0,
        "ch52fog: fog cells that part"
    );
}
