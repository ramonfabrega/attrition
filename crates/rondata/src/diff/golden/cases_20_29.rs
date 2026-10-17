// Golden case bodies. Wrappers in golden.rs preserve libtest names.
use super::*;

pub(super) fn chapter_twenty_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch20", "chapter20", 20, 11, 1299) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_TWENTY,
        "chapter twenty's golden word fell to {} from {GOLDEN_WORD_CHAPTER_TWENTY}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_TWENTY,
        "chapter twenty's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §28"
    );
    eprintln!(
        "chapter twenty: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_twenty_one_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch21", "chapter21", 21, 12, 1299) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_TWENTY_ONE,
        "chapter twenty-one's golden word fell to {} from {GOLDEN_WORD_CHAPTER_TWENTY_ONE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_TWENTY_ONE,
        "chapter twenty-one's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §29"
    );
    eprintln!(
        "chapter twenty-one: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_twenty_two_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch22", "chapter22", 22, 12, 1499) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_TWENTY_TWO,
        "chapter twenty-two's golden word fell to {} from {GOLDEN_WORD_CHAPTER_TWENTY_TWO}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_TWENTY_TWO,
        "chapter twenty-two's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §31"
    );
    eprintln!(
        "chapter twenty-two: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_twenty_three_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch23", "chapter23", 23, 13, 1839) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_TWENTY_THREE,
        "chapter twenty-three's golden word fell to {} from {GOLDEN_WORD_CHAPTER_TWENTY_THREE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_TWENTY_THREE,
        "chapter twenty-three's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §32"
    );
    eprintln!(
        "chapter twenty-three: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_twenty_four_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch24", "chapter24", 24, 8, 1559) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_TWENTY_FOUR,
        "chapter twenty-four's golden word fell to {} from {GOLDEN_WORD_CHAPTER_TWENTY_FOUR}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_TWENTY_FOUR,
        "chapter twenty-four's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §33"
    );
    eprintln!(
        "chapter twenty-four: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_twenty_five_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch25", "chapter25", 25, 13, 1465) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_TWENTY_FIVE,
        "chapter twenty-five's golden word fell to {} from {GOLDEN_WORD_CHAPTER_TWENTY_FIVE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_TWENTY_FIVE,
        "chapter twenty-five's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §34"
    );
    eprintln!(
        "chapter twenty-five: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_twenty_six_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch26", "chapter26", 26, 12, 1491) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_TWENTY_SIX,
        "chapter twenty-six's golden word fell to {} from {GOLDEN_WORD_CHAPTER_TWENTY_SIX}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_TWENTY_SIX,
        "chapter twenty-six's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §35"
    );
    eprintln!(
        "chapter twenty-six: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_twenty_seven_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch27", "chapter27", 27, 12, 1559) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_TWENTY_SEVEN,
        "chapter twenty-seven's golden word fell to {} from {GOLDEN_WORD_CHAPTER_TWENTY_SEVEN}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_TWENTY_SEVEN,
        "chapter twenty-seven's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §36"
    );
    eprintln!(
        "chapter twenty-seven: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_twenty_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch20",
        "chapter20",
        WIDENING_CHAPTER_TWENTY,
        1299,
        0,
        (702, 705),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch20 f{f} {w}/{o} {what}: {row}");
    }
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_TWENTY.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch20: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_TWENTY.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch20 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH20.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch20: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH20_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: 4 rows → 0 rows; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch20: what parts in the pool moved");
}

pub(super) fn chapter_twenty_one_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch21",
        "chapter21",
        WIDENING_CHAPTER_TWENTY_ONE,
        1299,
        0,
        (1139, 1142),
        true,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch21 f{f} {w}/{o} {what}: {row}");
    }
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_TWENTY_ONE.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch21: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_TWENTY_ONE.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch21 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH21.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch21: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH21_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: ["782 slot 1 ox", "782 slot 1 oy", "802 slot 0 … → []; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch21: what parts in the pool moved");
}

pub(super) fn chapter_twenty_two_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch22",
        "chapter22",
        WIDENING_CHAPTER_TWENTY_TWO,
        1499,
        0,
        (776, 780),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch22 f{f} {w}/{o} {what}: {row}");
    }
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_TWENTY_TWO.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch22: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_TWENTY_TWO.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch22 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH22.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch22: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH22_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: ["622 slot 1 ox", "622 slot 1 oy", "642 slot 0 … → []; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch22: what parts in the pool moved");
}

pub(super) fn chapter_twenty_three_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch23",
        "chapter23",
        WIDENING_CHAPTER_TWENTY_THREE,
        1839,
        0,
        (1440, 1444),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch23 f{f} {w}/{o} {what}: {row}");
    }
    // The first block's standing rows are chapter twenty-two's.
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_TWENTY_THREE.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch23: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_TWENTY_THREE.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch23 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH23.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch23: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH23_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: ["622 slot 1 ox", "622 slot 1 oy", "642 slot 0 … → []; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch23: what parts in the pool moved");
}

pub(super) fn chapter_twenty_four_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch24",
        "chapter24",
        WIDENING_CHAPTER_TWENTY_FOUR,
        1559,
        0,
        (853, 857),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch24 f{f} {w}/{o} {what}: {row}");
    }
    // The first block's standing rows are chapter thirteen's.
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_TWENTY_FOUR.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch24: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_TWENTY_FOUR.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch24 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH24.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch24: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH24_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: 8 rows → 0 rows; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch24: what parts in the pool moved");
}

pub(super) fn chapter_twenty_five_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch25",
        "chapter25",
        WIDENING_CHAPTER_TWENTY_FIVE,
        1465,
        0,
        (853, 857),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch25 f{f} {w}/{o} {what}: {row}");
    }
    // The first block's standing rows are chapter thirteen's.
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_TWENTY_FIVE.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch25: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_TWENTY_FIVE.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch25 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH25.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch25: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH25_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: 8 rows → 2 rows; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch25: what parts in the pool moved");
}

pub(super) fn chapter_twenty_six_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch26",
        "chapter26",
        WIDENING_CHAPTER_TWENTY_SIX,
        1491,
        0,
        (1020, 1024),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch26 f{f} {w}/{o} {what}: {row}");
    }
    // The first block's standing rows are chapter thirteen's.
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_TWENTY_SIX.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch26: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_TWENTY_SIX.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch26 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH26.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch26: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH26_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: ["622 slot 1 ox", "622 slot 1 oy", "872 slot 0 … → []; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch26: what parts in the pool moved");
}

pub(super) fn chapter_twenty_seven_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch27",
        "chapter27",
        WIDENING_CHAPTER_TWENTY_SEVEN,
        1559,
        0,
        (920, 924),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch27 f{f} {w}/{o} {what}: {row}");
    }
    // The first block's standing rows are chapter thirteen's.
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_TWENTY_SEVEN.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch27: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_TWENTY_SEVEN.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch27 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH27.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch27: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH27_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: 4 rows → 0 rows; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch27: what parts in the pool moved");
}

pub(super) fn chapter_twenty_eight_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch28", "chapter28", 28, 11, 1579) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_TWENTY_EIGHT,
        "chapter twenty-eight's golden word fell to {} from {GOLDEN_WORD_CHAPTER_TWENTY_EIGHT}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_TWENTY_EIGHT,
        "chapter twenty-eight's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §37"
    );
    eprintln!(
        "chapter twenty-eight: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_twenty_eight_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch28",
        "chapter28",
        WIDENING_CHAPTER_TWENTY_EIGHT,
        1579,
        0,
        (640, 644),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch28 f{f} {w}/{o} {what}: {row}");
    }
    // The first block's standing rows are chapter thirteen's.
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_TWENTY_EIGHT.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch28: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_TWENTY_EIGHT.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch28 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH28.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch28: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH28_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: 10 rows → 0 rows; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch28: what parts in the pool moved");
}

pub(super) fn chapter_twenty_nine_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch29", "chapter29", 29, 13, 2069) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_TWENTY_NINE,
        "chapter twenty-nine's golden word fell to {} from {GOLDEN_WORD_CHAPTER_TWENTY_NINE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_TWENTY_NINE,
        "chapter twenty-nine's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §38"
    );
    eprintln!(
        "chapter twenty-nine: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_twenty_nine_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch29",
        "chapter29",
        WIDENING_CHAPTER_TWENTY_NINE,
        2069,
        0,
        (1744, 1748),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch29 f{f} {w}/{o} {what}: {row}");
    }
    // The first block's standing rows are chapter twenty-two's.
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_TWENTY_NINE.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch29: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_TWENTY_NINE.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch29 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH29.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch29: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH29_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: ["622 slot 1 ox", "622 slot 1 oy", "642 slot 0 … → []; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch29: what parts in the pool moved");
}
