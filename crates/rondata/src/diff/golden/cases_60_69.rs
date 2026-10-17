// Golden case bodies. Wrappers in golden.rs preserve libtest names.
use super::*;

pub(super) fn chapter_sixty_one_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch61", "chapter61", 61, 13, 1000) else {
        return;
    };
    eprintln!(
        "chapter sixty-one: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_SIXTY_ONE,
        "chapter sixty-one's golden word fell to {} from {GOLDEN_WORD_CHAPTER_SIXTY_ONE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SIXTY_ONE,
        "chapter sixty-one's golden word moved; re-pin it here and say so in docs/GOLDEN.md §74"
    );
}

pub(super) fn chapter_sixty_one_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch61",
        "chapter61",
        WIDENING_CHAPTER_SIXTY_ONE,
        1000,
        0,
        (778, 780),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch61 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch61: {} rows", got.len());
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch61 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH61.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch61: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH61_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    pin_eq!(got_pool, want_pool, "ch61: the pool's parting moved");
}

pub(super) fn chapter_sixty_two_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch62", "chapter62", 62, 13, 1000) else {
        return;
    };
    eprintln!(
        "chapter sixty-two: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_SIXTY_TWO,
        "chapter sixty-two's golden word fell to {} from {GOLDEN_WORD_CHAPTER_SIXTY_TWO}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SIXTY_TWO,
        "chapter sixty-two's golden word moved; re-pin it here and say so in docs/GOLDEN.md §75"
    );
}

pub(super) fn chapter_sixty_two_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let (_, hi) = WIDENING_CHAPTER_SIXTY_TWO;
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch62",
        "chapter62",
        WIDENING_CHAPTER_SIXTY_TWO,
        1000,
        0,
        (hi - 3, hi - 1),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch62 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch62: {} rows", got.len());
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch62 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH62.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch62: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH62_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    pin_eq!(got_pool, want_pool, "ch62: the pool's parting moved");
}

pub(super) fn chapter_sixty_three_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch63", "chapter63", 63, 9, 760) else {
        return;
    };
    eprintln!(
        "chapter sixty-three: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_SIXTY_THREE,
        "chapter sixty-three's golden word fell to {} from {GOLDEN_WORD_CHAPTER_SIXTY_THREE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SIXTY_THREE,
        "chapter sixty-three's golden word moved; re-pin it here and say so in docs/GOLDEN.md §78"
    );
}

pub(super) fn chapter_sixty_three_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let (_, hi) = WIDENING_CHAPTER_SIXTY_THREE;
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch63",
        "chapter63",
        WIDENING_CHAPTER_SIXTY_THREE,
        760,
        0,
        (hi - 3, hi - 1),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch63 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch63: {} rows", got.len());
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch63 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH63.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch63: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH63_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    pin_eq!(got_pool, want_pool, "ch63: the pool's parting moved");
}

pub(super) fn chapter_sixty_four_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch64", "chapter64", 64, 8, 820) else {
        return;
    };
    eprintln!(
        "chapter sixty-four: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_SIXTY_FOUR,
        "chapter sixty-four's golden word fell to {} from {GOLDEN_WORD_CHAPTER_SIXTY_FOUR}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SIXTY_FOUR,
        "chapter sixty-four's golden word moved; re-pin it here and say so in docs/GOLDEN.md §79"
    );
}

pub(super) fn chapter_sixty_four_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let (_, hi) = WIDENING_CHAPTER_SIXTY_FOUR;
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch64",
        "chapter64",
        WIDENING_CHAPTER_SIXTY_FOUR,
        820,
        0,
        (hi - 3, hi - 1),
        true,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch64 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch64: {} rows", got.len());
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch64 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH64.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch64: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH64_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    pin_eq!(got_pool, want_pool, "ch64: the pool's parting moved");
}

pub(super) fn chapter_sixty_five_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch65", "chapter65", 65, 7, 1100) else {
        return;
    };
    eprintln!(
        "chapter sixty-five: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_SIXTY_FIVE,
        "chapter sixty-five's golden word fell to {} from {GOLDEN_WORD_CHAPTER_SIXTY_FIVE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SIXTY_FIVE,
        "chapter sixty-five's golden word moved; re-pin it here and say so in docs/GOLDEN.md §81"
    );
}

pub(super) fn chapter_sixty_five_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let (_, hi) = WIDENING_CHAPTER_SIXTY_FIVE;
    let Some(firsts) = widen_civilians(
        "ch65",
        "chapter65",
        WIDENING_CHAPTER_SIXTY_FIVE,
        1100,
        0,
        (hi - 3, hi - 1),
        true,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch65 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch65: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH65.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch65: what parts under the word moved");
}

pub(super) fn chapter_sixty_seven_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch67", "chapter67", 67, 10, 1330) else {
        return;
    };
    eprintln!(
        "chapter sixty-seven: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_SIXTY_SEVEN,
        "chapter sixty-seven's golden word fell to {} from {GOLDEN_WORD_CHAPTER_SIXTY_SEVEN}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SIXTY_SEVEN,
        "chapter sixty-seven's golden word moved; re-pin it here and say so in docs/GOLDEN.md §84"
    );
}

pub(super) fn chapter_sixty_seven_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch67",
        "chapter67",
        WIDENING_CHAPTER_SIXTY_SEVEN,
        1330,
        0,
        (799, 806),
        false,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch67 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch67: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH67.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch67: what parts under the word moved");
}

pub(super) fn chapter_sixty_eight_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch68", "chapter68", 68, 14, 1360) else {
        return;
    };
    eprintln!(
        "chapter sixty-eight: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_SIXTY_EIGHT,
        "chapter sixty-eight's golden word fell to {} from {GOLDEN_WORD_CHAPTER_SIXTY_EIGHT}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SIXTY_EIGHT,
        "chapter sixty-eight's golden word moved; re-pin it here and say so in docs/GOLDEN.md §85"
    );
}

pub(super) fn chapter_sixty_eight_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch68",
        "chapter68",
        WIDENING_CHAPTER_SIXTY_EIGHT,
        1360,
        0,
        (614, 618),
        false,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch68 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch68: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH68.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch68: what parts under the word moved");
}

pub(super) fn chapter_sixty_s_fog_plane() {
    let _pins = Pins::hold();
    let Some((firsts, _pool)) = widen_civilians_and_pool(
        "ch59fog",
        "chapter59",
        (605, 645),
        645,
        0,
        (640, 643),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch60fog f{f} {w}/{o} {what}: {row}");
    }
    // Before the build 7 `seen2` cells parted on 605 (first (117,75)); the
    // eleven `form` rows are every chapter's first block.
    pin_eq!(
        firsts
            .iter()
            .filter(|((_, _, w), _)| w.contains("seen2"))
            .count(),
        0,
        "ch60fog: fog cells that part"
    );
}
