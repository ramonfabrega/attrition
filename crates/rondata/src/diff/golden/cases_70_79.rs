// Golden case bodies. Wrappers in golden.rs preserve libtest names.
use super::*;

pub(super) fn chapter_seventy_one_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch71", "chapter71", 71, 11, 1120) else {
        return;
    };
    eprintln!(
        "chapter seventy-one: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    pin_eq!(
        w.word,
        GOLDEN_WORD_CHAPTER_SEVENTY_ONE,
        "chapter seventy-one's golden word moved; re-pin it here and say so in docs/GOLDEN.md §88"
    );
}

pub(super) fn chapter_seventy_one_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch71",
        "chapter71",
        WIDENING_CHAPTER_SEVENTY_ONE,
        1120,
        0,
        (1010, 1016),
        false,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch71 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch71: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH71.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch71: what parts under the word moved");
}

pub(super) fn chapter_seventy_two_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch72", "chapter72", 72, 13, 1770) else {
        return;
    };
    eprintln!(
        "chapter seventy-two: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    pin_eq!(
        w.word,
        GOLDEN_WORD_CHAPTER_SEVENTY_TWO,
        "chapter seventy-two's golden word moved; re-pin it here and say so in docs/GOLDEN.md §89"
    );
}

pub(super) fn chapter_seventy_two_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch72",
        "chapter72",
        WIDENING_CHAPTER_SEVENTY_TWO,
        1770,
        0,
        (1720, 1726),
        false,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch72 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch72: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH72.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch72: what parts under the word moved");
}

pub(super) fn chapter_seventy_three_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch73", "chapter73", 73, 10, 760) else {
        return;
    };
    eprintln!(
        "chapter seventy-three: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    pin_eq!(
        w.word,
        GOLDEN_WORD_CHAPTER_SEVENTY_THREE,
        "chapter seventy-three's golden word moved; re-pin it here and say so in docs/GOLDEN.md §90"
    );
}

pub(super) fn chapter_seventy_three_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch73",
        "chapter73",
        WIDENING_CHAPTER_SEVENTY_THREE,
        760,
        0,
        (698, 705),
        false,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch73 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch73: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH73.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch73: what parts under the word moved");
}

pub(super) fn chapter_seventy_four_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch74", "chapter74", 74, 6, 760) else {
        return;
    };
    eprintln!(
        "chapter seventy-four: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    pin_eq!(
        w.word,
        GOLDEN_WORD_CHAPTER_SEVENTY_FOUR,
        "chapter seventy-four's golden word moved; re-pin it here and say so in docs/GOLDEN.md §91"
    );
}

pub(super) fn chapter_seventy_four_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch74",
        "chapter74",
        WIDENING_CHAPTER_SEVENTY_FOUR,
        760,
        0,
        (698, 705),
        false,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch74 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch74: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH74.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch74: what parts under the word moved");
}

pub(super) fn chapter_seventy_five_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch75", "chapter75", 75, 9, 960) else {
        return;
    };
    eprintln!(
        "chapter seventy-five: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    pin_eq!(
        w.word,
        GOLDEN_WORD_CHAPTER_SEVENTY_FIVE,
        "chapter seventy-five's golden word moved; re-pin it here and say so in docs/GOLDEN.md §92"
    );
}

pub(super) fn chapter_seventy_five_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch75",
        "chapter75",
        WIDENING_CHAPTER_SEVENTY_FIVE,
        960,
        0,
        (898, 905),
        false,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch75 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch75: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH75.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch75: what parts under the word moved");
}

pub(super) fn chapter_seventy_six_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch76", "chapter76", 76, 9, 2100) else {
        return;
    };
    eprintln!(
        "chapter seventy-six: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    pin_eq!(
        w.word,
        GOLDEN_WORD_CHAPTER_SEVENTY_SIX,
        "chapter seventy-six's golden word moved; re-pin it here and say so in docs/GOLDEN.md §95"
    );
}

pub(super) fn chapter_seventy_six_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch76",
        "chapter76",
        WIDENING_CHAPTER_SEVENTY_SIX,
        2100,
        0,
        (1748, 1755),
        false,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch76 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch76: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH76.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch76: what parts under the word moved");
}

pub(super) fn chapter_seventy_seven_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch77", "chapter77", 77, 12, 1400) else {
        return;
    };
    eprintln!(
        "chapter seventy-seven: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    pin_eq!(
        w.word,
        GOLDEN_WORD_CHAPTER_SEVENTY_SEVEN,
        "chapter seventy-seven's golden word moved; re-pin it here and say so in docs/GOLDEN.md §96"
    );
}

pub(super) fn chapter_seventy_seven_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch77",
        "chapter77",
        WIDENING_CHAPTER_SEVENTY_SEVEN,
        1400,
        0,
        (1099, 1104),
        false,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch77 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch77: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH77.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch77: what parts under the word moved");
}

pub(super) fn chapter_seventy_eight_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch78", "chapter78", 78, 6, 1100) else {
        return;
    };
    eprintln!(
        "chapter seventy-eight: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    pin_eq!(
        w.word,
        GOLDEN_WORD_CHAPTER_SEVENTY_EIGHT,
        "chapter seventy-eight's golden word moved; re-pin it here and say so in docs/GOLDEN.md §97"
    );
}

pub(super) fn chapter_seventy_eight_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch78",
        "chapter78",
        WIDENING_CHAPTER_SEVENTY_EIGHT,
        1100,
        0,
        (1099, 1104),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch78 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch78: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH78.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch78: what parts under the word moved");
}

pub(super) fn chapter_seventy_nine_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch79", "chapter79", 79, 7, 1500) else {
        return;
    };
    eprintln!(
        "chapter seventy-nine: word {}, sequence {}, values {:?}",
        w.word, w.sequence, w.value
    );
    pin_eq!(
        w.word,
        GOLDEN_WORD_CHAPTER_SEVENTY_NINE,
        "chapter seventy-nine's golden word moved; re-pin it here and say so in docs/GOLDEN.md §98"
    );
}

pub(super) fn chapter_seventy_nine_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch79",
        "chapter79",
        WIDENING_CHAPTER_SEVENTY_NINE,
        1500,
        0,
        (1499, 1504),
        false,
        92,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch79 f{f} {w}/{o} {what}: {row}");
    }
    let mut got: Vec<String> = firsts
        .iter()
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    eprintln!("ch79: {} rows", got.len());
    let mut want: Vec<String> = WANT_CH79.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch79: what parts under the word moved");
}
