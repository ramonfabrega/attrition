// Golden case bodies. Wrappers in golden.rs preserve libtest names.
use super::*;

pub(super) fn chapter_eleven_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch11", "chapter11", 11, 8, 1250) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_ELEVEN,
        "chapter eleven's golden word fell to {} from {GOLDEN_WORD_CHAPTER_ELEVEN}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_ELEVEN,
        "chapter eleven's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §19"
    );
    eprintln!(
        "chapter eleven: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_twelve_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch12", "chapter12", 12, 11, 1150) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_TWELVE,
        "chapter twelve's golden word fell to {} from {GOLDEN_WORD_CHAPTER_TWELVE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_TWELVE,
        "chapter twelve's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §20"
    );
    eprintln!(
        "chapter twelve: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_thirteen_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch13", "chapter13", 13, 7, 1000) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_THIRTEEN,
        "chapter thirteen's golden word fell to {} from {GOLDEN_WORD_CHAPTER_THIRTEEN}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_THIRTEEN,
        "chapter thirteen's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §21"
    );
    eprintln!(
        "chapter thirteen: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_fourteen_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch14", "chapter14", 14, 7, 1150) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FOURTEEN,
        "chapter fourteen's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FOURTEEN}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FOURTEEN,
        "chapter fourteen's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §22"
    );
    eprintln!(
        "chapter fourteen: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_fifteen_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch15", "chapter15", 15, 7, 1250) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FIFTEEN,
        "chapter fifteen's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FIFTEEN}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FIFTEEN,
        "chapter fifteen's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §23"
    );
    eprintln!(
        "chapter fifteen: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_sixteen_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch16", "chapter16", 16, 7, 1250) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_SIXTEEN,
        "chapter sixteen's golden word fell to {} from {GOLDEN_WORD_CHAPTER_SIXTEEN}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SIXTEEN,
        "chapter sixteen's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §24"
    );
    eprintln!(
        "chapter sixteen: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_seventeen_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch17", "chapter17", 17, 11, 1399) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_SEVENTEEN,
        "chapter seventeen's golden word fell to {} from {GOLDEN_WORD_CHAPTER_SEVENTEEN}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SEVENTEEN,
        "chapter seventeen's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §25"
    );
    eprintln!(
        "chapter seventeen: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_eighteen_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch18", "chapter18", 18, 8, 1449) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_EIGHTEEN,
        "chapter eighteen's golden word fell to {} from {GOLDEN_WORD_CHAPTER_EIGHTEEN}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_EIGHTEEN,
        "chapter eighteen's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §26"
    );
    eprintln!(
        "chapter eighteen: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_nineteen_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch19", "chapter19", 19, 6, 1099) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_NINETEEN,
        "chapter nineteen's golden word fell to {} from {GOLDEN_WORD_CHAPTER_NINETEEN}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_NINETEEN,
        "chapter nineteen's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §27"
    );
    eprintln!(
        "chapter nineteen: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_ten_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch10", "chapter10", 10, 5, 1250) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_TEN,
        "chapter ten's golden word fell to {} from {GOLDEN_WORD_CHAPTER_TEN}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_TEN,
        "chapter ten's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §18"
    );
    eprintln!("chapter ten: sequence {}, values {:?}", w.sequence, w.value);
}

pub(super) fn chapter_eleven_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch11",
        "chapter11",
        WIDENING_CHAPTER_ELEVEN,
        1250,
        0,
        (730, 735),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch11 f{f} {w}/{o} {what}: {row}");
    }
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_ELEVEN.0)
        .map(|((_, _, what), _)| what)
        .collect();
    // Item 1605 took the human's `filled_gather_slots` (and city) rows: the
    // human leader takes the sweep (`docs/AI.md` §169).
    pin!(
        floor.iter().all(|w| standing(w)) && floor.len() == 24,
        "ch11: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_ELEVEN.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    // **What parts under the word**, by block and key (`docs/GOLDEN.md`
    // §19). Each staged unit's `form` on its birth block, the standing
    // family. Everything of the guard's own agrees from 622 to 721: the
    // `GUARDORDER` (`ox 7 whom 0`, offset (0, 372), post (3528, 8760)),
    // its transit leg and `timer`, the walk to the post, and `idle`.
    //
    // **The first pin, word 724**: the value parted on 722, where the
    // original's supply wagon `0/7`, on its first step under the player's
    // move, pushed the guard `0/6` standing on its post (run191's brackets:
    // `set_new_location(0/6)` inside the wagon's `detect_unit_collision`).
    // This crate took `detect_boat_collision` only for a ship; item 696
    // built its land half (`docs/COLLISION.md` §13.3, §13.5) → 726, where
    // the wagon's scan named the walking guard hard. The original's scan
    // (run191's third take) reaches `is_here` on the guard and never
    // `is_corner`: §4.3's escort row, "its action is `GUARD` on me", is
    // soft. Built → **734**.
    //
    // **The second pin, word 734**: on tick 733 the original's guard was
    // refused its own step (its `coll` held the refused point) and the
    // wagon then pushed it (`collide_o 7`); this crate's guard stepped
    // through. The disc follows guy 0, not the unit (item 703,
    // `docs/COLLISION.md` §16): the guard's figures still stood on the
    // cell tick 732's push left, and so did its bits, one of which is
    // the wagon's by `is_here`. Built, with a trackless crew arriving on
    // guy 0's point → **1036**. Every row from 736 to 1036 agrees but
    // who=1's chariot's `form` on its birth block, 1001, the standing
    // family, and the scout `1/0`'s formation mirror on 847, chapter
    // ten's row (parked 275).
    //
    // **On 1037** the original's guard holds its `GUARD` alone, the
    // attack on `1/6` gone with `recharging 0`; this crate's keeps the
    // `ATTACK` above it and fires (`recharging 25`). `1/6` stands on
    // (3355, 14070), about 1,810 units from the guard. No mechanism is
    // named.
    //
    // **The `near` row** (item 707, the widening's first new row since
    // 696): the guard's `near_o/near_who` goes (6, 1) → (−1, −1) on
    // 1037 in the original, so a search ran on tick 1036 and saw no
    // candidate; this crate's ran none. Every other block agrees on it,
    // `1/6`'s own −1 from 1030 among them.
    //
    // **The third pin, word 1036**: the guard's attack is leashed to its
    // post (item 707, `docs/COMBAT.md` §63). `fight` asks `check_target`
    // with its guarding argument once `valid_target` passes, and `1/6`
    // stood ≈1,780 from the post against 1,536. The guard's search is
    // leashed too (`near` −1), and a human's `GUARD` holds its
    // retaliation (`target_opportunity`'s action gate), which closed
    // 1091, where the leash alone left a one-frame `ATTACK`. Built →
    // **1133**. Every row from 736 to 1133 agrees.
    //
    // **On 1134** the original's `1/6` had dropped its `ATTACK` on the
    // guard (`recharging 0`, its army's `ATTACKTOORDER` alone); this
    // crate's fired (`recharging 25`).
    //
    // **The fourth pin, word 1133**: the hundredth-frame resync (item 709,
    // `docs/VISION.md` §10). `update_all_seen` on 1133 clears `seen` and
    // relights it; the guard's `visible` byte, 0 since 1051, no longer
    // lights its cell for who=1, and `1/6`'s disc stops a half-cell short.
    // `valid_target`'s fog test fails. 1033's resync had relit the cell
    // through the byte, which is why `1/6` fired from 1058 to 1108. Built →
    // **1139**. The seven 1134 rows close, and every row from 736 to 1140
    // agrees.
    //
    // **The fifth pin, word 1139**: the guard's idle stand on 1139,
    // `Guy::set_anim+0x97a < Unit::do_guard+0x7f4`, with no record parting
    // (`GUYS=2` prints no clock). `guy_flags & 0x20` is guy 0's (item 713,
    // `docs/ANIM.md` §13): the crew figure's p91 on tick 1100 is `IDLE2`,
    // 71 frames, and its re-roll falls on 1139. Built → **1250, closed**.
    //
    // **The block, run190 whole** (this test's, beside the constant's
    // delta). Without the fix the whole window parts from 1156: `1/4`'s
    // move (`orders_x/y`, `dest_angle`, the move's `x/y/off/angle`), then
    // `0/4` on 1172, `1/3` on 1188 and `1/1`'s gather `wait` on 1189. With
    // it no row parts past 1001: the list below is the standing family
    // (parked 275's row, as on 1139, gone since `push_group` keeps an equal group's record (item 723)).
    let mut want: Vec<String> = Vec::new();
    want.sort();
    pin_eq!(got, want, "ch11: what parts under the word moved");
}

pub(super) fn chapter_twelve_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch12",
        "chapter12",
        WIDENING_CHAPTER_TWELVE,
        1150,
        0,
        (715, 718),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch12 f{f} {w}/{o} {what}: {row}");
    }
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_TWELVE.0)
        .map(|((_, _, what), _)| what)
        .collect();
    // Item 1605 took the human's `filled_gather_slots` (and city) rows: the
    // human leader takes the sweep (`docs/AI.md` §169).
    pin!(
        floor.iter().all(|w| standing(w)) && floor.len() == 24,
        "ch12: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_TWELVE.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    // **What parts under the word**, by block and key (`docs/GOLDEN.md`
    // §20). Each staged unit's `form` on its birth block, the standing
    // family. ~~The scout `1/0`'s formation mirror on 847 (parked 275)~~,
    // gone since `push_group` keeps an equal group's record (item 723).
    //
    // **The first pin, word 717**: this crate had no follow, and the
    // harness skipped both `@follow` lines. On 622 the original's chariot
    // `0/6` held one `FOLLOWORDER` on the wagon and was its pushed group's
    // (`group 1`), where this crate's held nothing and counted `idle`; the
    // same for the squad on 642 (`group 0`); the wagon's move group on
    // 702 was `group 2` against this crate's 1; and on 711 the original's
    // chariot took its first one-tile leg and stepped.
    //
    // **Built → 1150, closed** (`docs/ORDERS.md` §28): every one of those
    // rows goes. The chariot's 24 hops and its rest, the hoplites' legs
    // under the doubled `k`, both turns and every follower's final stand
    // agree to run204's end, block for block.
    let mut want: Vec<String> = Vec::new();
    want.sort();
    pin_eq!(got, want, "ch12: what parts under the word moved");
}

pub(super) fn chapter_thirteen_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch13",
        "chapter13",
        WIDENING_CHAPTER_THIRTEEN,
        1000,
        0,
        (638, 641),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch13 f{f} {w}/{o} {what}: {row}");
    }
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_THIRTEEN.0)
        .map(|((_, _, what), _)| what)
        .collect();
    // Item 1605 took the human's `filled_gather_slots` (and city) rows: the
    // human leader takes the sweep (`docs/AI.md` §169).
    pin!(
        floor.iter().all(|w| standing(w)) && floor.len() == 11,
        "ch13: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_THIRTEEN.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    // **What parts under the word**, by block and key (`docs/GOLDEN.md`
    // §21). Each staged unit's `form` on its birth block, the standing
    // family. ~~The scout `1/0`'s formation mirror on 847 (parked 275)~~,
    // gone since `push_group` keeps an equal group's record (item 723).
    //
    // **The first pin, word 640**: this crate could not take the garrison
    // command, and the harness skipped both `@garrison` lines. On 622 the
    // original's chariot held its `GARRISONORDER` under a plain leg and
    // was its pushed group's (`group 1`), where this crate's held nothing
    // and counted `idle`; on 623 it had pathed and stepped.
    //
    // **Built → 1000, closed** (`docs/ORDERS.md` §29): the two entries,
    // `check_target_path`'s GARRISON arm, the door as `adjacent_to`, the
    // chain's kill, and `come_out`'s angles. ~~What stays is the squad's
    // pool slot on 903: `come_out` pushes a squad that leaves a building
    // into a fresh group (`Groups::push_group(…, 1)`, type `+0x308 > 1`),
    // slot 3 behind the eject's own building group in 2. Neither is
    // carried here (parked 689's family).~~ **Both built** (item 882,
    // `docs/GOLDEN.md` §33): the eject's `process_group` seats `[2007]` in slot 2
    // and `come_out` pushes the squad, and `0/7`–`0/9` read group 3 on 903
    // on both sides. What stays is the births' `form`, `Unit::init`'s
    // (parked 646).
    let mut want: Vec<String> = Vec::new();
    want.sort();
    pin_eq!(got, want, "ch13: what parts under the word moved");
}

pub(super) fn chapter_fourteen_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch14",
        "chapter14",
        WIDENING_CHAPTER_FOURTEEN,
        1150,
        0,
        (629, 632),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch14 f{f} {w}/{o} {what}: {row}");
    }
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_FOURTEEN.0)
        .map(|((_, _, what), _)| what)
        .collect();
    // Item 1605 took the human's `filled_gather_slots` (and city) rows: the
    // human leader takes the sweep (`docs/AI.md` §169).
    pin!(
        floor.iter().all(|w| standing(w)) && floor.len() == 11,
        "ch14: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_FOURTEEN.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    // **What parts under the word**, by block and key (`docs/GOLDEN.md`
    // §22). Each staged unit's `form` on its birth block, the standing
    // family.
    //
    // **The first pin, word 631**: this crate could not take the
    // formation command, and the harness skipped both `@form` lines. On
    // 622 each of the original's nine held `form 2`, `form_mod 50` and
    // `group 1`, and a plain move to its Envelop slot round `0/6`, where
    // this crate's held nothing and counted `idle`; from 623 they walked.
    //
    // **Built → 1150, closed** (`docs/ORDERS.md` §30): the command's
    // entry, an equal group's slot and record kept by `push_group`, and
    // the replay to `orig`. What stays is the group move's `id` on 702:
    // this crate numbers a pushed group `64 +` its seat, where the
    // original's is `who·64 +` the pool slot, 701101 against 707501
    // (parked 689).
    let mut want: Vec<String> = Vec::new();
    want.sort();
    pin_eq!(got, want, "ch14: what parts under the word moved");
}

pub(super) fn chapter_fifteen_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch15",
        "chapter15",
        WIDENING_CHAPTER_FIFTEEN,
        1250,
        0,
        (751, 754),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch15 f{f} {w}/{o} {what}: {row}");
    }
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_FIFTEEN.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch15: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_FIFTEEN.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch15 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    // **What parts under the word**, by block and key (`docs/GOLDEN.md`
    // §23). Each staged unit's `form` on its birth block, the standing
    // family, and the right-click's group move `id` on 622, this crate's
    // numbering of a pushed group (parked 689).
    //
    // **The first pin, word 753**: the harness skipped `@attack`, and on
    // 736 105 keys of the six parted, the original's `ATTACKORDER`s
    // against this crate's walk; the charge's first roll parted the draw
    // stream on 753.
    //
    // **Built → 1250, closed**: `input::group_attack` entered alone took
    // the word to the trace's end and emptied the 736 block. What stays
    // beside the standing family and the id is **one instrument row**:
    // `1/6`'s death object on 809 is `extra`, because run215 dumps no
    // `DEATHS` (off under `[End Frame]` for the pool) and `DEATH_OBJS`
    // prints a record per corpse and nothing else, so the dump cannot say
    // the list was not asked for. The target last prints on 808 on both
    // sides.
    let mut want: Vec<String> = ["809 1/6 death:extra"]
        .iter()
        .map(|r| r.to_string())
        .collect();
    want.sort();
    pin_eq!(got, want, "ch15: what parts under the word moved");
    // **What parts in the pool: nothing**, on all 645 blocks. Slot 1 is
    // the pushed selection from 621. The first pin had its `stamp`
    // (`GroupData +0x14`) 621 in the dump and 0 here from 622, until
    // `push_group` stamped a fresh slot as `Groups::copy_group@006fa690`
    // does; and on 736 and 738 the skipped attack's `order_num`,
    // `facing` and `curr`.
    let want_pool: Vec<String> = Vec::new();
    pin_eq!(got_pool, want_pool, "ch15: what parts in the pool moved");
}

pub(super) fn chapter_sixteen_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch16",
        "chapter16",
        WIDENING_CHAPTER_SIXTEEN,
        1250,
        0,
        (836, 839),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch16 f{f} {w}/{o} {what}: {row}");
    }
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_SIXTEEN.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch16: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_SIXTEEN.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch16 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    // **What parts under the word**, by block and key (`docs/GOLDEN.md`
    // §24): each staged unit's `form` on its birth block, the standing
    // family, and nothing else, on run219 whole.
    //
    // **The first pin, word 838, open** (605, 840): the re-issue behind a
    // goody-box leg. `finish_insert` replays the copied explore to its
    // `orig`, the click, and this crate, which carried no `orig` on a
    // plain move, replayed it to the snapped `dest`. `0/6`'s plan goal on
    // 685 was (2424, 17304) against (2400, 17280), and from it the
    // waypoint on 779, the heading and the one-unit drift of 793 and 797,
    // and the arrival: the original's chariot stands on the click on 838
    // and goes idle, the word's two `Unit::do_idle` rolls. The squad's
    // re-issue on 804 was the same seam, re-laid round the leader's slot
    // (`0/8` to (12600, 15096) against (12552, 15048)), carried into the
    // box leg's re-aim on 819 and `0/9`'s step on 823. 71 keys in all.
    //
    // **Built → 1250, closed** (item 738): `MoveOrder::orig` on a plain
    // move, set by `action_move_near`'s plain arm, and `finish_insert`
    // replaying to it. All 67 keys past the births went.
    let mut want: Vec<String> = Vec::new();
    want.sort();
    pin_eq!(got, want, "ch16: what parts under the word moved");
    // **What parts in the pool.** Slot 0 is the squad's explore (641),
    // slot 1 the chariot's (621); the chariot's box leg on 684 frees slot
    // 1 and takes slot 2, and the squad's on 803 takes slot 1 back.
    //
    // ~~`ox`/`oy` on the box leg's slot~~: the re-issue's group point, the
    // click there and the snap here until `orig` was carried.
    //
    // **`speed`/`new_speed` on 685 and 804 stand.** `Groups::
    // get_open_slot@006fa460` asks each live slot it walks past for
    // `get_num` (vslot `+4`), which normalizes a seated group of fewer
    // than four and so re-seats its speed (`docs/GROUPS.md` §3.1, §4.1,
    // §18.1): the squad's slot 0 takes 25 on the chariot's push, and the
    // fresh slot takes its leader's. This crate does not model the slot
    // allocation (`docs/GROUPS.md`'s table of what the pool leaves out),
    // and no step of an explore or a flee reads a group's speed.
    let mut want_pool: Vec<String> = [
        "685 slot 2 new_speed",
        "685 slot 2 speed",
        "804 slot 1 new_speed",
        "804 slot 1 speed",
    ]
    .iter()
    .map(|r| r.to_string())
    .collect();
    want_pool.sort();
    // Item 1457: 6 rows → 4 rows; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch16: what parts in the pool moved");
}

pub(super) fn chapter_nineteen_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch19",
        "chapter19",
        WIDENING_CHAPTER_NINETEEN,
        1099,
        0,
        (754, 758),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch19 f{f} {w}/{o} {what}: {row}");
    }
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_NINETEEN.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch19: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_NINETEEN.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch19 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    let mut want: Vec<String> = WANT_CH19.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch19: what parts under the word moved");
    let mut want_pool: Vec<String> = WANT_CH19_POOL.iter().map(|r| r.to_string()).collect();
    want_pool.sort();
    // Item 1457: 2 rows → 0 rows; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch19: what parts in the pool moved");
}

pub(super) fn chapter_eighteen_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch18",
        "chapter18",
        WIDENING_CHAPTER_EIGHTEEN,
        1449,
        0,
        (620, 623),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch18 f{f} {w}/{o} {what}: {row}");
    }
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_EIGHTEEN.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch18: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_EIGHTEEN.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch18 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    // **What parts under the word**, by block and key (`docs/GOLDEN.md`
    // §26). **The first pin, word 642, open** (605, 645): the harness
    // skipped both `@build` lines, the command having no entry into this
    // simulation. So, past the births' `form`: on 622 the Barracks
    // `0/2007`, which the dump held alone, the 120 timber it cost, and
    // `0/6` standing idle with no order where the original's holds its
    // `MOVEORDER` and `BUILDORDER` and has taken its first step; on 642
    // the same for the Siege Factory `0/2008`, its 60 metal and the
    // three. 104 keys in all. The word was the draw an idle citizen spends
    // on 642 that the walking one does not.
    //
    // **The command entered → 1450, closed** (item 779, `docs/ORDERS.md`
    // §36; the window widened to run241 whole). Every key past the births
    // went: the sites, the prices, each builder's two orders, every step
    // of the walks, every block of construction, the completions, and
    // `0/8`'s help on 1097 once the one-unit swarm asked whose builder it
    // is (an `EXPLORETOORDER` there before).
    let mut want: Vec<String> = Vec::new();
    want.sort();
    pin_eq!(got, want, "ch18: what parts under the word moved");
    // **What parts in the pool.** The first pin had each command's pushed
    // selection missing here — slot 1 `0/6` on 622 and slot 0 the three
    // on 642 — and the entry seats both. What stands is `ox`/`oy` on each:
    // (0, 0) in the original, `Group::clear`'s (−1, −1) here, as on
    // chapter seventeen's flight groups (§25). `action_build` writes no
    // group point and no step of a build reads one.
    let mut want_pool: Vec<String> = Vec::new();
    want_pool.sort();
    // Item 1457: 4 rows → 0 rows; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch18: what parts in the pool moved");
}

pub(super) fn chapter_seventeen_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some((firsts, pool)) = widen_civilians_and_pool(
        "ch17",
        "chapter17",
        WIDENING_CHAPTER_SEVENTEEN,
        1399,
        0,
        (640, 644),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch17 f{f} {w}/{o} {what}: {row}");
    }
    // `g.gpiece` is chapter one's age bracket (`library who=0 6` on 600,
    // before the window): the pre-existing citizens' art, which the
    // stand-up does not carry.
    let standing = |what: &str| {
        what == "form"
            || what == "build:extra"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("g.gpiece")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_SEVENTEEN.0)
        .map(|((_, _, what), _)| what)
        .collect();
    pin!(
        floor.iter().all(|w| standing(w)),
        "ch17: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_SEVENTEEN.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    for ((slot, key), (f, row)) in &pool {
        eprintln!("  ch17 pool f{f} slot {slot} {key}: {row}");
    }
    let mut got_pool: Vec<String> = pool
        .iter()
        .map(|((slot, key), (f, _))| format!("{f} slot {slot} {key}"))
        .collect();
    got_pool.sort();
    // **What parts under the word**, by block and key (`docs/GOLDEN.md`
    // §25). **The first pin, word 642, open** (605, 645): the births'
    // `form`, chapter one's `g.end_time` beside the age bracket, the
    // pair's `group` on 622 (`process_group` pushed the refused strike's
    // selection), and the Fighter on 642 — no order, no path, idle, where
    // the original's holds its `STRAFEORDER` home and has taken its first
    // step — 26 keys past the births.
    //
    // **The command entered** (item 746, `docs/ORDERS.md` §32; the window
    // widened to 667 to hold the pair's strafe and the strike): every
    // `STRAFEORDER` row agrees on 642 and 662 — the target, `mandatory`,
    // the `AIRORDER`'s home `0/2007`, `cruising_alt` 1600, `sharp_turn`,
    // `old`, `returning 1`, `xx/yy −1` — and the pushed selections' `group`
    // with them. What stands is **the flight**, which this crate does not
    // fly: each aircraft's first step (its figure, heading, `pos`, `path`)
    // on its processed block and `orders_x/y` and `dest_angle` the next;
    // the Fighter's `cruising_alt` redraw on 643, 1600 → 1300, the word's
    // draw; and on 666 the strike turned `AIRPATROLORDER` (17), a class
    // this crate has no body for. Past the word the stream is one draw
    // short, and that is what the citizens' and `1/5`'s rows from 650
    // are: `1/5`'s scout leg re-rolled on 659, `1/1`'s gather wait.
    //
    // **The flight home, flown** (item 759, `docs/ORDERS.md` §33; the
    // window widened to 807 to hold the new word 805 and the bomb on
    // 806). Every aircraft agrees whole on 642 — the Fighter's point,
    // heading, figure, path, bank, pitch and altitude, all compared now —
    // and so does the Fighter's whole flight to its landing inside
    // `0/2007` on 722, its figure's one standing frame there and the
    // inside arm's `last_z` after it, and the pair's flight home on
    // 662–665. The one-draw shift past the word is gone with it. What
    // stands:
    // - **666 onward, the pair**: the strike turned `AIRPATROLORDER`
    //   (17), a class this crate has no body for, so its aircraft hold
    //   the re-pointed strafe where they stand — every figure row, the
    //   point, the heading, the path, the attitude and the altitude, and
    //   the order rows on 667; on 777 and 778 the patrol's own search
    //   pushes a `STRAFEORDER` on `1/2006` (`mandatory 0`, no action
    //   bit), and on 806 `0/8`'s bomb: its `recharging` and the
    //   `CHAR_ATTACK2` clock. **666 is the next item's frame.**
    // - the citizens' standing rows, 617 to 655, as before the flight.
    //
    // **The pair's patrol, flown** (item 763, `docs/ORDERS.md` §34; the
    // window widened to 823 to hold the new word 821 and the damage on
    // 822). Every row of 666 onward goes: the patrol's class and its
    // step on 666, `orders_x/y` and `dest_angle` on 667, the searches'
    // strikes on 777 and 778 — whose `orders_x/y` needed the
    // `QUEUE_FIRST` push to skip `update_action` — and `0/8`'s bomb on
    // 806. What stands is the Barracks' `damage`/`damage_frac` on 822,
    // compared by the shared instrument since this item (parked 728):
    // the bomb's round, which this crate does not release.
    //
    // **The strafe's round, released** (item 770, `docs/ORDERS.md` §35;
    // the window widened to run223 whole, (605, 1401), the word walking
    // to the capture's end). A strafe with a target releases its bombs,
    // from the plane's own height and its two bays, and each falls a tile
    // ahead in 17 frames: the Barracks' `damage` agrees on every block,
    // 822 to its death on 1080, all five attacks and every fringe. What
    // stands past the births is **the tank** (parked 765), and nothing
    // before it: `0/7`'s `returning` 1 there and 0 here on 1212, `0/8`'s
    // on 1214 — `mana_burn` is not carried — and each plane's flight
    // home from then, which this crate does not turn for. No draw
    // follows from them to 1400.
    //
    // **The tank, built** (item 836, `docs/ORDERS.md` §38.1): `mana_burn`
    // +1 a frame on the map, so `check_fuel`'s empty arm turns `0/7` for
    // home on 1212 and `0/8` on 1214, `returning 1` on both sides, and
    // each flight home agrees to 1400. ~~What stands is the births' nine.~~
    // Nothing stands since item 1330: the births' `form` is `Unit::init`'s.
    let want: Vec<String> = Vec::new();
    pin_eq!(got, want, "ch17: what parts under the word moved");
    // **What parts in the pool.** The first pin had each command's
    // pushed selection missing here — the refused strike's pair in slot 1
    // on 622, the Fighter's flight in slot 0 on 642 — and the entry seats
    // both. What stands:
    // - **`ox`/`oy` on each pushed flight group**: (0, 0) in the original,
    //   `Group::clear`'s (−1, −1) here. `action_flight` writes no group
    //   point and no step of a flight reads one; which writer leaves 0 is
    //   not read.
    // - **`speed`/`new_speed`** on 662 and 666: `Groups::get_open_slot`
    //   re-seating a live slot's speed as it walks past (item 738's
    //   `get_num`), which this crate does not model, and no flight reads.
    let mut want_pool: Vec<String> = Vec::new();
    want_pool.sort();
    // Item 1457: 10 rows → 0 rows; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(got_pool, want_pool, "ch17: what parts in the pool moved");
}

pub(super) fn chapter_ten_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch10",
        "chapter10",
        WIDENING_CHAPTER_TEN,
        1250,
        0,
        (1249, 1249),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch10 f{f} {w}/{o} {what}: {row}");
    }
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_TEN.0)
        .map(|((_, _, what), _)| what)
        .collect();
    // Item 1605 took the human's `filled_gather_slots` (and city) rows: the
    // human leader takes the sweep (`docs/AI.md` §169).
    pin!(
        floor.iter().all(|w| standing(w)) && floor.len() == 24,
        "ch10: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_TEN.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    // **What parts under the word**, by block and key (`docs/GOLDEN.md`
    // §18). Each staged unit's `form` on its birth block, the standing
    // family.
    //
    // **Item 693 took the word to the capture's end** by building the
    // patrol (`docs/ORDERS.md` §27): `input::group_patrol` →
    // `Sim::group_action_patrol` → `add_patrol_order`, the leader's
    // `do_patrol` legs, and `redo_patrol_order` in a group `QUEUE_FIRST`.
    // The first pin, (605, 642) on the open word 640, held the chariot's
    // missing patrol on 622 and the squad's on 642; all of it went. The
    // window is run184 whole, 605 to its last block, 1249; 1250 is the
    // `!quit` frame and prints no block.
    //
    // **The ids stand** — the patrol's on 622 and 642 and the squad's
    // first leg's `GroupAttackToOrder` on 642: `(group.id + frame × 10) ×
    // 100 + order_num`, where the original's `group.id` is the pool slot
    // (1 for the chariot, 0 for the squad) and this crate's pushed group
    // reads `64 +` its index (parked 689, chapter nine's 642 row). Values
    // no step reads; declared non-scoring.
    //
    // ~~One row stood that was neither order's: the scout `1/0`'s
    // formation mirror `facing` from 847 (parked 275).~~ Gone since
    // `push_group` keeps an equal group's record (item 723).
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_TEN.0)
        .map(|((w, o, what), (f, row))| {
            if what == "order:move.facing" {
                format!("{f} {w}/{o} {what}: {row}")
            } else {
                format!("{f} {w}/{o} {what}")
            }
        })
        .collect();
    got.sort();
    let mut want: Vec<String> = Vec::new();
    want.sort();
    pin_eq!(got, want, "ch10: what parts under the word moved");
}
