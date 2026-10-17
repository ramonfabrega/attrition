// Golden case bodies. Wrappers in golden.rs preserve libtest names.
use super::*;

pub(super) fn chapter_thirty_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch30", "chapter30", 30, 14, 1449) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_THIRTY,
        "chapter thirty's golden word fell to {} from {GOLDEN_WORD_CHAPTER_THIRTY}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_THIRTY,
        "chapter thirty's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §39"
    );
    eprintln!(
        "chapter thirty: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_thirty_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch30",
        "chapter30",
        WIDENING_CHAPTER_THIRTY,
        1449,
        0,
        (616, 620),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch30 f{f} {w}/{o} {what}: {row}");
    }
    // The first block's standing rows are chapter twenty-eight's.
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_THIRTY.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch30: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_THIRTY.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch30 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH30.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch30: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH30_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: 6 rows → 0 rows; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch30: what parts in the pool moved");
}

pub(super) fn chapter_thirty_one_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch31", "chapter31", 31, 16, 1400) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_THIRTY_ONE,
        "chapter thirty-one's golden word fell to {} from {GOLDEN_WORD_CHAPTER_THIRTY_ONE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_THIRTY_ONE,
        "chapter thirty-one's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §40"
    );
    eprintln!(
        "chapter thirty-one: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_thirty_one_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch31",
        "chapter31",
        WIDENING_CHAPTER_THIRTY_ONE,
        1400,
        0,
        (616, 620),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch31 f{f} {w}/{o} {what}: {row}");
    }
    // The first block's standing rows are chapter twenty-eight's, as
    // chapter thirty's.
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_THIRTY_ONE.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch31: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_THIRTY_ONE.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch31 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH31.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch31: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH31_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: ["618 slot 1 ox", "618 slot 1 oy", "622 slot 0 … → []; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch31: what parts in the pool moved");
}

pub(super) fn chapter_thirty_two_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch32", "chapter32", 32, 16, 2359) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_THIRTY_TWO,
        "chapter thirty-two's golden word fell to {} from {GOLDEN_WORD_CHAPTER_THIRTY_TWO}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_THIRTY_TWO,
        "chapter thirty-two's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §41"
    );
    eprintln!(
        "chapter thirty-two: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_thirty_two_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch32",
        "chapter32",
        WIDENING_CHAPTER_THIRTY_TWO,
        2359,
        0,
        (1601, 1603),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch32 f{f} {w}/{o} {what}: {row}");
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
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_THIRTY_TWO.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch32: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_THIRTY_TWO.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch32 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH32.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch32: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH32_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: ["622 slot 1 ox", "622 slot 1 oy", "642 slot 0 … → []; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch32: what parts in the pool moved");
}

pub(super) fn chapter_thirty_three_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch33", "chapter33", 33, 22, 2739) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_THIRTY_THREE,
        "chapter thirty-three's golden word fell to {} from {GOLDEN_WORD_CHAPTER_THIRTY_THREE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_THIRTY_THREE,
        "chapter thirty-three's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §42"
    );
    eprintln!(
        "chapter thirty-three: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_thirty_three_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch33",
        "chapter33",
        WIDENING_CHAPTER_THIRTY_THREE,
        2739,
        0,
        (2261, 2263),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch33 f{f} {w}/{o} {what}: {row}");
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
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_THIRTY_THREE.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch33: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_THIRTY_THREE.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch33 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH33.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch33: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH33_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: ["2314 slot 1 facing", "622 slot 1 ox", "622 sl… → ["2314 slot 1 facing"]; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch33: what parts in the pool moved");
}

pub(super) fn chapter_thirty_four_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch34", "chapter34", 34, 27, 2849) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_THIRTY_FOUR,
        "chapter thirty-four's golden word fell to {} from {GOLDEN_WORD_CHAPTER_THIRTY_FOUR}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_THIRTY_FOUR,
        "chapter thirty-four's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §43"
    );
    eprintln!(
        "chapter thirty-four: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_thirty_five_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch35", "chapter35", 35, 30, 3259) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_THIRTY_FIVE,
        "chapter thirty-five's golden word fell to {} from {GOLDEN_WORD_CHAPTER_THIRTY_FIVE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_THIRTY_FIVE,
        "chapter thirty-five's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §44"
    );
    eprintln!(
        "chapter thirty-five: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_thirty_five_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch35",
        "chapter35",
        WIDENING_CHAPTER_THIRTY_FIVE,
        3259,
        0,
        (2444, 2447),
        true,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch35 f{f} {w}/{o} {what}: {row}");
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
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_THIRTY_FIVE.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch35: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_THIRTY_FIVE.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch35 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    eprintln!("ch35: {} rows, {} pool rows", got.len(), got_pool.len());
    let mut want: Vec<String> = WANT_CH35.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch35: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH35_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: 12 rows → 0 rows; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch35: what parts in the pool moved");
}

pub(super) fn chapter_thirty_six_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch36", "chapter36", 36, 42, 3419) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_THIRTY_SIX,
        "chapter thirty-six's golden word fell to {} from {GOLDEN_WORD_CHAPTER_THIRTY_SIX}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_THIRTY_SIX,
        "chapter thirty-six's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §45"
    );
    eprintln!(
        "chapter thirty-six: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_thirty_six_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch36",
        "chapter36",
        WIDENING_CHAPTER_THIRTY_SIX,
        3419,
        0,
        (3110, 3113),
        true,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch36 f{f} {w}/{o} {what}: {row}");
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
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_THIRTY_SIX.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch36: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_THIRTY_SIX.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch36 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    eprintln!("ch36: {} rows, {} pool rows", got.len(), got_pool.len());
    let mut want: Vec<String> = WANT_CH36.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch36: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH36_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: 12 rows → 0 rows; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch36: what parts in the pool moved");
}

pub(super) fn chapter_thirty_seven_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch37", "chapter37", 37, 14, 3489) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_THIRTY_SEVEN,
        "chapter thirty-seven's golden word fell to {} from {GOLDEN_WORD_CHAPTER_THIRTY_SEVEN}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_THIRTY_SEVEN,
        "chapter thirty-seven's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §46"
    );
    eprintln!(
        "chapter thirty-seven: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_thirty_seven_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch37",
        "chapter37",
        WIDENING_CHAPTER_THIRTY_SEVEN,
        3489,
        0,
        (3199, 3202),
        true,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch37 f{f} {w}/{o} {what}: {row}");
    }
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_THIRTY_SEVEN.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch37: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_THIRTY_SEVEN.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch37 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    eprintln!("ch37: {} rows, {} pool rows", got.len(), got_pool.len());
    let mut want: Vec<String> = WANT_CH37.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch37: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH37_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: ["617 slot 1 ox", "617 slot 1 oy"] → []; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch37: what parts in the pool moved");
}

pub(super) fn chapter_thirty_eight_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch38", "chapter38", 38, 11, 1761) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_THIRTY_EIGHT,
        "chapter thirty-eight's golden word fell to {} from {GOLDEN_WORD_CHAPTER_THIRTY_EIGHT}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_THIRTY_EIGHT,
        "chapter thirty-eight's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §47"
    );
    eprintln!(
        "chapter thirty-eight: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_thirty_nine_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch39", "chapter39", 39, 14, 1099) else {
        return;
    };
    eprintln!(
        "chapter thirty-nine: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_THIRTY_NINE,
        "chapter thirty-nine's golden word fell to {} from {GOLDEN_WORD_CHAPTER_THIRTY_NINE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_THIRTY_NINE,
        "chapter thirty-nine's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §48"
    );
}

pub(super) fn chapter_thirty_nine_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch39",
        "chapter39",
        WIDENING_CHAPTER_THIRTY_NINE,
        1099,
        0,
        (620, 627),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch39 f{f} {w}/{o} {what}: {row}");
    }
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_THIRTY_NINE.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch39: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_THIRTY_NINE.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch39 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    eprintln!("ch39: {} rows, {} pool rows", got.len(), got_pool.len());
    let mut want: Vec<String> = WANT_CH39.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch39: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH39_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: ["622 slot 1 ox", "622 slot 1 oy", "642 slot 0 … → []; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch39: what parts in the pool moved");
}

pub(super) fn chapter_thirty_eight_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch38",
        "chapter38",
        WIDENING_CHAPTER_THIRTY_EIGHT,
        1761,
        0,
        (752, 755),
        true,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch38 f{f} {w}/{o} {what}: {row}");
    }
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_THIRTY_EIGHT.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch38: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_THIRTY_EIGHT.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch38: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH38.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch38: what parts under the word moved");
}

pub(super) fn chapter_thirty_eight_s_battery_is_short_of_its_aim_on_the_word_s_block() {
    const BLOCK: i64 = 780;
    let Some(mut s) = stage_script("ch38", "chapter38") else {
        return;
    };
    for _ in 0..BLOCK {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
    }
    let at =
        s.ix.frames()
            .iter()
            .position(|x| x.number == BLOCK)
            .expect("run404 carries block 780");
    let frame = s.ix.frame_state(at).unwrap();
    let them = frame
        .units
        .iter()
        .find(|u| (u.who, u.o) == (1, 9))
        .expect("the Battery on 780");
    let g = &them.guys[0];
    assert_eq!(
        (g.cur_anim, g.cur_time, g.node_flags, g.des_node_flags),
        (Some(12), Some(4), Some(14), Some(1)),
        "run404's block 780 is not the one this test was written from"
    );
    assert_eq!(g.turret_angles, Some([1_476_220_240, 0, 0, 0]));
    assert_eq!(g.des_turret_angles, Some([-1_978_269_696, 0, 0, 0]));
    let u = s.built.sim.unit_by_o(1, 9).expect("ours holds the Battery");
    let t = s.built.sim.units[u].guys[0].turret;
    // Both sides: the bits agree, and so does the step the turret is on
    // (its aim is 2,359,296 off theirs from 743, the node's vector —
    // `pivot::NODES` has no row for the Battery's piece).
    assert_eq!(
        (i64::from(t.node_flags), i64::from(t.des_flags)),
        (14, 1),
        "ours: node_flags / des_node_flags on 780"
    );
    let og = s.built.sim.units[u].guys[0];
    assert_eq!(
        (i64::from(og.anim), i64::from(og.cur_time)),
        (12, 4),
        "ours: the Battery's clock on 780"
    );
    // **And the round is held on both sides** (§85): run404 prints no
    // `AMMO` of `1/9` from 762 to 782, and ours holds none on 780 — the
    // word's own row, `780 1/9 ammo[0]` ("this crate holds it alone"),
    // until the gate stopped keying on the piece's measured vectors.
    let mine = s
        .built
        .sim
        .projectiles
        .iter()
        .filter(|p| p.shooter == sim::combat::Obj::Unit(u))
        .count();
    assert_eq!(mine, 0, "ours: the Battery's held round on 780");
}

pub(super) fn chapter_thirty_eight_s_squad_stands_on_its_points_and_packs_on_its_phase() {
    let Some(mut s) = stage_script("ch38", "chapter38") else {
        return;
    };
    let blocks: Vec<i64> = s.ix.frames().iter().map(|x| x.number).collect();
    let (mut packs, mut compared) = (Vec::new(), 0usize);
    for (at, &block) in blocks.iter().enumerate() {
        if block > WIDENING_CHAPTER_THIRTY_EIGHT.1 - 1 {
            break;
        }
        while s.built.sim.frame < block {
            s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
            s.built.tick();
        }
        let Ok(frame) = s.ix.frame_state(at) else {
            continue;
        };
        for o in 6..=8i16 {
            let Some(them) = frame
                .units
                .iter()
                .find(|u| (u.who, u.o) == (1, i64::from(o)))
            else {
                continue;
            };
            let Some(u) = s.built.sim.unit_by_o(1, o) else {
                continue;
            };
            let ours = s.built.sim.units[u]
                .orders
                .front()
                .and_then(|x| match &x.body {
                    sim::orders::Body::Move(m) => Some(*m),
                    _ => None,
                });
            let theirs = them.orders.last();
            if block == 765 {
                let m = ours.expect("ours: the squad's move on 765");
                let t = theirs.expect("theirs: the squad's move on 765");
                assert_eq!(
                    (i64::from(m.dest.x), i64::from(m.dest.y)),
                    (t.x.unwrap(), t.y.unwrap()),
                    "765 1/{o}: the order's point"
                );
            }
            // both sides: a unit whose current order is a move on one side
            // only is the widening's `order:kind` row, not this test's; the
            // count of packs found (twenty) is what keeps this from passing
            // quiet.
            if let (Some(m), Some(t)) = (ours, theirs)
                && let (Some(r), Some(a)) = (t.retry, t.attempts)
            {
                compared += 1;
                assert_eq!(
                    (i64::from(m.retry), i64::from(m.attempts)),
                    (r, a),
                    "{block} 1/{o}: retry, attempts"
                );
                if (r, a) == (22, -3) {
                    packs.push((block - 1, o));
                }
            }
        }
    }
    // The dump's own points on 765, so the test cannot pass on an empty
    // record: the scatter's two.
    let at = blocks.iter().position(|&b| b == 765).expect("block 765");
    let frame = s.ix.frame_state(at).unwrap();
    let point = |o: i64| {
        let t = frame.units.iter().find(|u| (u.who, u.o) == (1, o)).unwrap();
        let m = t.orders.last().unwrap();
        (m.x.unwrap(), m.y.unwrap())
    };
    assert_eq!((point(7), point(8)), ((38568, 13368), (38712, 13176)));
    eprintln!("ch38 squad: {compared} moves compared, packs {packs:?}");
    assert_eq!(packs.len(), 20, "run404's twenty packs");
    assert_eq!(&packs[..3], &[(777, 7), (794, 6), (888, 8)]);
    assert!(
        packs
            .iter()
            .all(|&(f, o)| (i64::from(o) * 0x11 + f) & 0x7f == 0),
        "every pack on its unit's phase"
    );
}

pub(super) fn chapter_thirty_seven_s_nuke_is_launched_and_fired_field_for_field() {
    use std::collections::{BTreeMap, BTreeSet};
    let Some(mut s) = stage_script("ch37", "chapter37") else {
        return;
    };
    let (first, last) = NUKE_LAUNCH;
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let (mut rows, mut rounds) = (0usize, 0usize);
    for f in 0..last - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < first {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let raw = s.ix.read_frame(at).unwrap();
        let recs = raw_records(&raw);
        let sim = &s.built.sim;
        let mut row = |who: i64, o: i64, what: &str, mine: i64, dumped: i64| {
            rows += 1;
            if mine != dumped {
                firsts
                    .entry((who, o, what.to_string()))
                    .or_insert((n, format!("ours {mine} theirs {dumped}")));
            }
        };
        // The silo.
        let silo = sim
            .buildings
            .iter()
            .position(|b| b.owner == 0 && b.index == 2007 && b.alive)
            .expect("the silo stands");
        let bd = &sim.buildings[silo];
        let theirs = recs
            .get(&("BUILDDATA", 0, 2007))
            .expect("run397 prints the silo");
        let get = |k: &str| theirs.get(k).copied().unwrap_or(-1);
        let launching = bd
            .launching
            .first()
            .map_or(-1, |&u| i64::from(sim.units[u].index));
        row(
            0,
            2007,
            "recharging",
            i64::from(bd.recharging),
            get("recharging"),
        );
        row(
            0,
            2007,
            "launch_frames",
            i64::from(bd.launch_frames),
            get("launch_frames"),
        );
        row(
            0,
            2007,
            "launching.length",
            bd.launching.len() as i64,
            get("length"),
        );
        row(0, 2007, "launching[0]", launching, get("list[scan]"));
        // The nuke's launch shows the silo to everyone (item 1091).
        row(
            0,
            2007,
            "visible",
            sim.nukes.visible_of(silo),
            get("visible"),
        );
        // The nuke.
        let ours_v2 = sim.unit_by_o(0, 10).is_some();
        let theirs_v2 = recs.contains_key(&("UNITDATA", 0, 10));
        row(0, 10, "present", i64::from(ours_v2), i64::from(theirs_v2));
        // Its rounds.
        let theirs: BTreeMap<i64, super::ammo::Ammo> = super::ammo::blocks(&raw)
            .into_iter()
            .filter(|(a, _)| a.flags & 2 != 0 && a.who == 0 && a.o == 10)
            .map(|(a, _)| (a.index, a))
            .collect();
        let ours: BTreeMap<i64, sim::combat::Projectile> = sim
            .projectiles
            .iter()
            .filter(|p| match p.shooter {
                sim::combat::Obj::Unit(u) => sim.units[u].owner == 0 && sim.units[u].index == 10,
                sim::combat::Obj::Building(_) => false,
            })
            .map(|p| (i64::from(p.slot), *p))
            .collect();
        rounds += theirs.len();
        let slots: BTreeSet<i64> = theirs.keys().chain(ours.keys()).copied().collect();
        for slot in slots {
            let (Some(a), Some(p)) = (theirs.get(&slot), ours.get(&slot)) else {
                // Held alone: 1 on the side that holds it.
                row(
                    0,
                    10,
                    &format!("ammo[{slot}]"),
                    i64::from(ours.contains_key(&slot)),
                    i64::from(theirs.contains_key(&slot)),
                );
                continue;
            };
            for (name, mine, dumped) in [
                ("cur_time", i64::from(p.cur_time), a.cur_time),
                ("total_time", i64::from(p.total_time), a.total_time),
                ("whom", -1, a.whom),
                ("ox", -1, a.ox),
                ("sx", i64::from(p.launch.x), a.sx),
                ("sy", i64::from(p.launch.y), a.sy),
                ("sz", i64::from(p.sz), a.sz),
                ("ex", i64::from(p.landing.x), a.ex),
                ("ey", i64::from(p.landing.y), a.ey),
                ("ez", i64::from(p.ez), a.ez),
                ("angle", i64::from(p.angle.0), a.angle),
                ("traj", 2, a.traj),
                ("v1z", super::ammo::tests::printed(p.v1z), a.v1z),
                ("accuracy", i64::from(p.accuracy), a.accuracy),
                ("splash_area", i64::from(p.splash_area), a.splash_area),
                ("num_guys", i64::from(p.num_guys), a.num_guys),
            ]
            .into_iter()
            .chain(ammo_flag_rows(p, a))
            {
                row(0, 10, &format!("ammo[{slot}].{name}"), mine, dumped);
            }
        }
    }
    for ((w, o, what), (f, r)) in &firsts {
        eprintln!("  ch37 nuke f{f} {w}/{o} {what}: {r}");
    }
    eprintln!("ch37 nuke: {rows} rows, {rounds} dumped rounds");
    assert!(
        rounds >= 100,
        "run397 dumps the nuke's round from 3082 to its landing"
    );
    let got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    assert_eq!(
        got, WANT_CH37_NUKE,
        "ch37: what parts in the nuke's launch moved"
    );
}

pub(super) fn chapter_thirty_five_s_v2_is_counted_out_and_fired_field_for_field() {
    use std::collections::{BTreeMap, BTreeSet};
    let Some(mut s) = stage_script("ch35", "chapter35") else {
        return;
    };
    let (first, last) = V2_LAUNCH;
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let (mut rows, mut rounds) = (0usize, 0usize);
    for f in 0..last - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < first {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let raw = s.ix.read_frame(at).unwrap();
        let recs = raw_records(&raw);
        let sim = &s.built.sim;
        let mut row = |who: i64, o: i64, what: &str, mine: i64, dumped: i64| {
            rows += 1;
            if mine != dumped {
                firsts
                    .entry((who, o, what.to_string()))
                    .or_insert((n, format!("ours {mine} theirs {dumped}")));
            }
        };
        // The silo.
        let silo = sim
            .buildings
            .iter()
            .position(|b| b.owner == 0 && b.index == 2009 && b.alive)
            .expect("the silo stands");
        let bd = &sim.buildings[silo];
        let theirs = recs
            .get(&("BUILDDATA", 0, 2009))
            .expect("run371 prints the silo");
        let get = |k: &str| theirs.get(k).copied().unwrap_or(-1);
        let launching = bd
            .launching
            .first()
            .map_or(-1, |&u| i64::from(sim.units[u].index));
        row(
            0,
            2009,
            "recharging",
            i64::from(bd.recharging),
            get("recharging"),
        );
        row(
            0,
            2009,
            "launch_frames",
            i64::from(bd.launch_frames),
            get("launch_frames"),
        );
        row(
            0,
            2009,
            "launching.length",
            bd.launching.len() as i64,
            get("length"),
        );
        row(0, 2009, "launching[0]", launching, get("list[scan]"));
        // The V2.
        let ours_v2 = sim.unit_by_o(0, 10).is_some();
        let theirs_v2 = recs.contains_key(&("UNITDATA", 0, 10));
        row(0, 10, "present", i64::from(ours_v2), i64::from(theirs_v2));
        // Its rounds.
        let theirs: BTreeMap<i64, super::ammo::Ammo> = super::ammo::blocks(&raw)
            .into_iter()
            .filter(|(a, _)| a.flags & 2 != 0 && a.who == 0 && a.o == 10)
            .map(|(a, _)| (a.index, a))
            .collect();
        let ours: BTreeMap<i64, sim::combat::Projectile> = sim
            .projectiles
            .iter()
            .filter(|p| match p.shooter {
                sim::combat::Obj::Unit(u) => sim.units[u].owner == 0 && sim.units[u].index == 10,
                sim::combat::Obj::Building(_) => false,
            })
            .map(|p| (i64::from(p.slot), *p))
            .collect();
        rounds += theirs.len();
        let slots: BTreeSet<i64> = theirs.keys().chain(ours.keys()).copied().collect();
        for slot in slots {
            let (Some(a), Some(p)) = (theirs.get(&slot), ours.get(&slot)) else {
                // Held alone: 1 on the side that holds it.
                row(
                    0,
                    10,
                    &format!("ammo[{slot}]"),
                    i64::from(ours.contains_key(&slot)),
                    i64::from(theirs.contains_key(&slot)),
                );
                continue;
            };
            for (name, mine, dumped) in [
                ("cur_time", i64::from(p.cur_time), a.cur_time),
                ("total_time", i64::from(p.total_time), a.total_time),
                ("whom", -1, a.whom),
                ("ox", -1, a.ox),
                ("sx", i64::from(p.launch.x), a.sx),
                ("sy", i64::from(p.launch.y), a.sy),
                ("sz", i64::from(p.sz), a.sz),
                ("ex", i64::from(p.landing.x), a.ex),
                ("ey", i64::from(p.landing.y), a.ey),
                ("ez", i64::from(p.ez), a.ez),
                ("angle", i64::from(p.angle.0), a.angle),
                ("traj", 2, a.traj),
                ("v1z", super::ammo::tests::printed(p.v1z), a.v1z),
                ("accuracy", i64::from(p.accuracy), a.accuracy),
                ("splash_area", i64::from(p.splash_area), a.splash_area),
                ("num_guys", i64::from(p.num_guys), a.num_guys),
            ]
            .into_iter()
            .chain(ammo_flag_rows(p, a))
            {
                row(0, 10, &format!("ammo[{slot}].{name}"), mine, dumped);
            }
        }
    }
    for ((w, o, what), (f, r)) in &firsts {
        eprintln!("  ch35 v2 f{f} {w}/{o} {what}: {r}");
    }
    eprintln!("ch35 v2: {rows} rows, {rounds} dumped rounds");
    assert!(rounds >= 2, "run371 dumps the V2's round on 2702 and 2703");
    let got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    assert_eq!(
        got, WANT_CH35_V2,
        "ch35: what parts in the V2's launch moved"
    );
}

pub(super) fn chapter_thirty_five_s_v2_blast_is_compared_field_for_field() {
    use std::collections::{BTreeMap, BTreeSet};
    let Some(mut s) = stage_script("ch35", "chapter35") else {
        return;
    };
    let (first, last) = V2_BLAST;
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let mut rows = 0usize;
    let mut barracks = 0usize;
    for f in 0..last - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < first {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let raw = s.ix.read_frame(at).unwrap();
        let recs = raw_records(&raw);
        let sim = &s.built.sim;
        let mut row = |who: i64, o: i64, what: &str, mine: i64, dumped: i64| {
            rows += 1;
            if mine != dumped {
                firsts
                    .entry((who, o, what.to_string()))
                    .or_insert((n, format!("ours {mine} theirs {dumped}")));
            }
        };
        // Every building, both directions.
        let ours: BTreeMap<(i64, i64), &sim::Building> = sim
            .buildings
            .iter()
            .filter(|b| b.alive)
            .map(|b| ((i64::from(b.owner), i64::from(b.index)), b))
            .collect();
        let keys: BTreeSet<(i64, i64)> = recs
            .keys()
            .filter(|(k, _, _)| *k == "BUILDDATA")
            .map(|&(_, w, o)| (w, o))
            .chain(ours.keys().copied())
            .collect();
        for (w, o) in keys {
            let theirs = recs.get(&("BUILDDATA", w, o));
            let mine = ours.get(&(w, o));
            row(
                w,
                o,
                "present",
                i64::from(mine.is_some()),
                i64::from(theirs.is_some()),
            );
            // both sides: a building on one side only is the `present` row
            // above, which parts; `damage` is read where both hold it.
            if let (Some(b), Some(t)) = (mine, theirs) {
                row(
                    w,
                    o,
                    "damage",
                    i64::from(b.damage),
                    t.get("damage").copied().unwrap_or(-1),
                );
                if (w, o) == (1, 2006) {
                    barracks += 1;
                }
            }
        }
        // Every unit, both directions.
        let ours: BTreeSet<(i64, i64)> = sim
            .units
            .iter()
            .filter(|u| u.alive() && !u.is_gaia())
            .map(|u| (i64::from(u.owner), i64::from(u.index)))
            .collect();
        let theirs: BTreeSet<(i64, i64)> = recs
            .keys()
            .filter(|(k, w, _)| *k == "UNITDATA" && *w < 8)
            .map(|&(_, w, o)| (w, o))
            .collect();
        for &(w, o) in ours.union(&theirs) {
            row(
                w,
                o,
                "present",
                i64::from(ours.contains(&(w, o))),
                i64::from(theirs.contains(&(w, o))),
            );
        }
        // The round.
        let theirs = super::ammo::blocks(&raw)
            .into_iter()
            .filter(|(a, _)| a.flags & 2 != 0 && a.who == 0 && a.o == 10)
            .count();
        let ours = sim
            .projectiles
            .iter()
            .filter(|p| match p.shooter {
                sim::combat::Obj::Unit(u) => sim.units[u].owner == 0 && sim.units[u].index == 10,
                sim::combat::Obj::Building(_) => false,
            })
            .count();
        row(0, 10, "rounds", ours as i64, theirs as i64);
    }
    for ((w, o, what), (f, r)) in &firsts {
        eprintln!("  ch35 blast f{f} {w}/{o} {what}: {r}");
    }
    eprintln!("ch35 blast: {rows} rows, the Barracks held both sides on {barracks} blocks");
    assert_eq!(barracks, 3, "run371 dumps the Barracks on 2818..2820");
    let got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    assert_eq!(
        got, WANT_CH35_BLAST,
        "ch35: what parts in the V2's blast moved"
    );
}

pub(super) fn chapter_thirty_four_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch34",
        "chapter34",
        WIDENING_CHAPTER_THIRTY_FOUR,
        2849,
        0,
        (2306, 2308),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch34 f{f} {w}/{o} {what}: {row}");
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
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_THIRTY_FOUR.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch34: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_THIRTY_FOUR.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch34 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    eprintln!("ch34: {} rows, {} pool rows", got.len(), got_pool.len());
    let mut want: Vec<String> = WANT_CH34.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch34: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH34_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: ["2602 slot 3 ox", "2602 slot 3 oy", "622 slot … → []; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch34: what parts in the pool moved");
}
