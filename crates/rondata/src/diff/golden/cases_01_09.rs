// Golden case bodies. Wrappers in golden.rs preserve libtest names.
use super::*;

pub(super) fn chapter_one_holds_to_the_golden_word() {
    let Some(w) = walk_chapter("g4", 1, 6, 900) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_ONE,
        "chapter one's golden word fell to {} from {GOLDEN_WORD_CHAPTER_ONE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_ONE,
        "chapter one's golden word moved; re-pin it here and on the \
         handoff's `Golden:` line together"
    );
}

pub(super) fn chapter_two_holds_to_the_golden_word() {
    let Some(w) = walk_chapter("ch2", 2, 6, 900) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_TWO,
        "chapter two's golden word fell to {} from {GOLDEN_WORD_CHAPTER_TWO}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_TWO,
        "chapter two's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §6"
    );
    // The value diff beside the draw stream, printed rather than pinned
    // until it is the thing an item is taken on.
    eprintln!("chapter two: sequence {}, values {:?}", w.sequence, w.value);
}

pub(super) fn chapter_five_holds_to_the_golden_word() {
    let Some(w) = walk_chapter("ch5", 5, 4, 900) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FIVE,
        "chapter five's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FIVE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FIVE,
        "chapter five's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §9"
    );
    eprintln!(
        "chapter five: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_six_holds_to_the_golden_word() {
    let Some(w) = walk_chapter("ch6", 6, 6, 900) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_SIX,
        "chapter six's golden word fell to {} from {GOLDEN_WORD_CHAPTER_SIX}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SIX,
        "chapter six's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §10"
    );
    eprintln!("chapter six: sequence {}, values {:?}", w.sequence, w.value);
}

pub(super) fn chapter_six_b_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch6b", "chapter6b", 6, 8, 1250) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_SIX_B,
        "chapter six-b's golden word fell to {} from {GOLDEN_WORD_CHAPTER_SIX_B}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SIX_B,
        "chapter six-b's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §10"
    );
    eprintln!(
        "chapter six-b: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_nine_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch9", "chapter9", 9, 5, 1100) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_NINE,
        "chapter nine's golden word fell to {} from {GOLDEN_WORD_CHAPTER_NINE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_NINE,
        "chapter nine's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §17"
    );
    eprintln!(
        "chapter nine: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_eight_holds_to_the_golden_word() {
    let Some(w) = walk_chapter("ch8", 8, 10, 900) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_EIGHT,
        "chapter eight's golden word fell to {} from {GOLDEN_WORD_CHAPTER_EIGHT}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_EIGHT,
        "chapter eight's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §12"
    );
    eprintln!(
        "chapter eight: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_two_s_squads_stand_where_the_dump_stands_them() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = chapter(2);
    // **617, not 622.** Both `add` lines that matter have run and nothing
    // has moved yet on either side; 622 is five frames of this crate's own
    // marching later, which is what item 415 mistook for a seating error.
    for _ in 0..617 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    // The dump's own coordinates, frames 616–618, unchanged across them:
    // bowmen 0/6–0/8, hoplites 1/6–1/8.
    let dump: [(u8, i16, (i32, i32)); 6] = [
        (0, 6, (888, 7800)),
        (0, 7, (1032, 7800)),
        (0, 8, (936, 7944)),
        (1, 6, (2424, 7800)),
        (1, 7, (2568, 7800)),
        (1, 8, (2472, 7944)),
    ];
    let mut got = Vec::new();
    for (who, o, _) in dump {
        let u = (0..built.sim.units.len())
            .find(|&u| built.sim.units[u].owner == who && built.sim.units[u].index == o)
            .unwrap_or_else(|| panic!("no unit {who}/{o} after chapter two's `add` lines"));
        let p = built.sim.units[u].pos;
        got.push((who, o, (p.x, p.y)));
    }
    // **All six are the dump's own cells at 617.** `add`'s ring is right
    // for both squads; item 415's contrary finding was five frames of
    // marching read at 622.
    assert_eq!(
        got,
        dump.to_vec(),
        "chapter two's squads are not seated where the golden dump seats \
         them at 617"
    );
    // ~~**And the drift is arithmetic.** Five more frames and who=1's
    // squad stands 140 units west on all three figures, at 28 a frame,
    // while the dump holds the original's still: item 415's number, with
    // the mechanism it actually has.~~ **Closed by item 447**
    // (`docs/COMBAT.md` §31): the chase that walked them was an attack
    // order the original never issued, and `is_seen` in `valid_target`
    // retires it. So the assertion inverts — who=1's squad **does not
    // move**, which is the dump's own behaviour through 634, and the 140
    // is a number this crate can no longer produce.
    for _ in 0..5 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    for (who, o, seat) in dump.iter().skip(3) {
        let u = (0..built.sim.units.len())
            .find(|&u| built.sim.units[u].owner == *who && built.sim.units[u].index == *o)
            .expect("the squad is still alive");
        let p = built.sim.units[u].pos;
        assert_eq!(
            (p.x, p.y),
            *seat,
            "unit {who}/{o} has walked off its seat by 622; the original's \
             hoplites hold no order at all through 634 and neither should \
             these (docs/COMBAT.md §31)"
        );
    }
    // Anti-vacuity: the gap the chapter is built on. 2424 − 1032 is 1392
    // units, 7.25 tiles, inside the Bowmen's ten and outside the Hoplites'
    // zero — which is the whole premise of the chapter.
    assert_eq!(
        dump[3].2.0 - dump[1].2.0,
        1392,
        "the two squads are not 7.25 tiles apart; chapter two's premise is \
         a ranged squad outside a melee squad's reach"
    );
}

pub(super) fn chapter_one_s_two_squads_are_seated_where_the_dump_says() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("g4") else {
        eprintln!("skipping: no golden capture (see docs/RUNS.md run101–run105)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = chapter(1);
    // Far enough for both `add` lines (610 and 615) and no further: the
    // units walk from 616 on, and this is about where they are *born*.
    for _ in 0..616 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    // The dump's own `GUY` blocks for frames 616 and 617, `who`/`o`/`x`/`y`
    // (`docs/RUNS.md` run101–run105). `o` 6, 7, 8 are the three members of
    // each `add`, threaded `down`/`up` in that order.
    let want: [(u8, i16, (i32, i32)); 6] = [
        (0, 6, (888, 7800)),
        (0, 7, (1032, 7800)),
        (0, 8, (936, 7944)),
        (1, 6, (1368, 7992)),
        (1, 7, (1512, 7992)),
        (1, 8, (1416, 8136)),
    ];
    let mut got = Vec::new();
    for (who, o, _) in want {
        let u = (0..built.sim.units.len())
            .find(|&u| built.sim.units[u].owner == who && built.sim.units[u].index == o)
            .unwrap_or_else(|| panic!("no unit {who}/{o} after chapter one's two `add` lines"));
        let p = built.sim.units[u].pos;
        got.push((who, o, (p.x, p.y)));
    }
    assert_eq!(
        got,
        want.to_vec(),
        "the staged squads are not seated where the golden dump puts them"
    );
}

pub(super) fn chapter_one_s_captains_reach_each_other_and_nobody_else() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("g4") else {
        eprintln!("skipping: no golden capture (see docs/RUNS.md run101–run105)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = chapter(1);
    // Frame 615 stepped: both `add` lines are in, the squads are seated,
    // and nothing has walked off its seat yet.
    for _ in 0..616 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    let find = |b: &Built, who: u8, o: i16| {
        (0..b.sim.units.len())
            .find(|&u| b.sim.units[u].owner == who && b.sim.units[u].index == o)
            .unwrap_or_else(|| panic!("no unit {who}/{o} after chapter one's two `add` lines"))
    };
    // (attacker, target, attack_dist, is_in_range). Every pair that
    // crosses the two squads, in `who`/`o` order.
    /// (who, o) — the pair the dump names a unit by.
    type Who = (u8, i16);
    /// attacker, target, `attack_dist`, `is_in_range`.
    type Reach = (Who, Who, i32, bool);
    let want: [Reach; 6] = [
        ((1, 6), (0, 6), 339, false),
        ((1, 6), (0, 7), 198, true),
        ((1, 6), (0, 8), 288, false),
        ((1, 7), (0, 7), 339, false),
        ((1, 8), (0, 7), 316, false),
        ((0, 7), (1, 6), 198, true),
    ];
    let mut got = Vec::new();
    for (a, t, _, _) in want {
        let (ua, ut) = (find(&built, a.0, a.1), find(&built, t.0, t.1));
        let (oa, ot) = (sim::combat::Obj::Unit(ua), sim::combat::Obj::Unit(ut));
        got.push((
            a,
            t,
            built.sim.attack_dist(oa, ot),
            built.sim.is_in_range(oa, ot),
        ));
    }
    assert_eq!(
        got,
        want.to_vec(),
        "the engagement frame's reach is not the original's: the HOPLITES \
         line takes 0xf6 where everything else takes 0x66 \
         (docs/COMBAT.md §13.2)"
    );
}

pub(super) fn chapter_one_s_captain_picks_the_one_it_can_reach() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("g4") else {
        eprintln!("skipping: no golden capture (see docs/RUNS.md run101–run105)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = chapter(1);
    for _ in 0..616 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    let find = |b: &Built, who: u8, o: i16| {
        sim::combat::Obj::Unit(
            (0..b.sim.units.len())
                .find(|&u| b.sim.units[u].owner == who && b.sim.units[u].index == o)
                .unwrap_or_else(|| panic!("no unit {who}/{o} after chapter one's two `add` lines")),
        )
    };
    let captain = find(&built, 1, 6);
    // **The frame's own answer**, not a re-run: the search ran inside tick
    // 615 and left its target on the unit, and a second call here would
    // see the `targeted` bump the first one made. The dump's `1/6` reads
    // `ox 7 whom 0 uid 14` under frame 616, and so do `1/7` and `1/8` —
    // `Group::target_opportunity` hands the captain's find to the squad
    // (`docs/GROUPS.md` §13), which this crate does not model, so only the
    // captain's is asserted.
    let sim::combat::Obj::Unit(ci) = captain else {
        unreachable!()
    };
    assert_eq!(
        built.sim.units[ci].combat.target,
        Some(find(&built, 0, 7)),
        "the golden record's `1/6` attacks `0/7`; the candidates arrive \
         `0/8`, `0/7`, `0/6` in the cell's `down` order and a tie would \
         keep the first (docs/COMBAT.md §18)"
    );
    // The three values the pick rests on, in the trace's own candidate
    // order. `in_range` is the search's permission-to-test, `true` for
    // every candidate of a non-guarding AGGRESSIVE unit.
    //
    // **The fourth argument is this crate's own answer, checked against
    // the trace's** — run108 printed `ai=1` on this very captain, and
    // since item 470 [`sim::Sim::search_ai`] computes it. That equality is
    // the one place on disk where the predicate at `00648e6e` is measured
    // rather than read (`docs/COMBAT.md` §33.2).
    let ai = built.sim.search_ai(1);
    assert!(
        ai,
        "run108's proxy printed `ai=1` on who=1's captain; `search_ai`          disagrees, so the gate at `00648e6e` is modelled wrong"
    );
    let v: Vec<i32> = [(0, 8), (0, 7), (0, 6)]
        .into_iter()
        .map(|(w, o)| {
            built
                .sim
                .compare_target(captain, find(&built, w, o), true, ai)
        })
        .collect();
    assert_eq!(
        v[0], v[2],
        "the two candidates out of reach must value alike: {v:?}"
    );
    assert!(
        v[1] > v[0],
        "the reachable candidate must outvalue the two out of reach — \
         `compare_target`'s `/5` at `0064f1ed`: {v:?}"
    );
    // **And the falsifier.** `in_range = false` is the caller saying "do
    // not test", which is the arm the old reading took for every
    // candidate: the three then come back equal, the score's `/ 0xc0`
    // bucket is 3 for all three, and the tie hands the pick to whichever
    // the cell's `down` chain reached first — `0/8`. One bit of one
    // predicate is the whole distance between this frame and the
    // original's.
    let flat: Vec<i32> = [(0, 8), (0, 7), (0, 6)]
        .into_iter()
        .map(|(w, o)| {
            built
                .sim
                .compare_target(captain, find(&built, w, o), false, ai)
        })
        .collect();
    assert!(
        flat[0] == flat[1] && flat[1] == flat[2],
        "undiscounted, three identical hoplites must tie — that tie is why \
         reading `in_range` as the verdict cost the frame: {flat:?}"
    );
}

pub(super) fn chapter_one_s_hit_is_answered_by_the_victim_s_captain() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("g4") else {
        eprintln!("skipping: no golden capture (see docs/RUNS.md run101-run105)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = chapter(1);
    // Through the end of frame 616 — the dump block labelled 617.
    for _ in 0..617 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    let find = |b: &Built, who: u8, o: i16| {
        (0..b.sim.units.len())
            .find(|&u| b.sim.units[u].owner == who && b.sim.units[u].index == o)
            .unwrap_or_else(|| panic!("no unit {who}/{o} after chapter one's two `add` lines"))
    };
    let striker = sim::combat::Obj::Unit(find(&built, 1, 6));
    let (cap, hit, other) = (find(&built, 0, 6), find(&built, 0, 7), find(&built, 0, 8));
    // The `o_up` chain the dump prints for who=0: `0/6` is `o_up -1`, and
    // `0/7`/`0/8` hang off it (`docs/COMBAT.md` §18.2).
    assert!(
        built.sim.units[cap].captain
            && !built.sim.units[hit].captain
            && !built.sim.units[other].captain,
        "who=0's squad is one captain and two members"
    );
    assert_eq!(
        (
            built.sim.units[cap].combat.target,
            built.sim.units[hit].combat.target,
            built.sim.units[other].combat.target,
        ),
        (Some(striker), None, None),
        "the dump's frame 617 has the ATTACKORDER on `0/6` alone"
    );
}

pub(super) fn chapter_one_s_squad_is_handed_its_captain_s_target() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("g4") else {
        eprintln!("skipping: no golden capture (see docs/RUNS.md run101-run105)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = chapter(1);
    let find = |b: &Built, who: u8, o: i16| {
        (0..b.sim.units.len())
            .find(|&u| b.sim.units[u].owner == who && b.sim.units[u].index == o)
            .unwrap_or_else(|| panic!("no unit {who}/{o} after chapter one's two `add` lines"))
    };
    // Through the end of frame 615 — the dump block labelled 616.
    for _ in 0..616 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    let hit = sim::combat::Obj::Unit(find(&built, 0, 7));
    let (c, m1, m2) = (find(&built, 1, 6), find(&built, 1, 7), find(&built, 1, 8));
    assert!(
        built.sim.units[c].captain && !built.sim.units[m1].captain && !built.sim.units[m2].captain,
        "who=1's squad is one captain and two members"
    );
    assert_eq!(
        (
            built.sim.units[c].combat.target,
            built.sim.units[m1].combat.target,
            built.sim.units[m2].combat.target,
        ),
        (Some(hit), Some(hit), Some(hit)),
        "the dump's block 616 has `ox 7 whom 0` on all three of who=1"
    );
    // On to the end of frame 617 — block 618, and the ordered points.
    for _ in 616..618 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    let at = |u: usize| {
        let p = built.sim.units[u].orders_pos;
        (p.x, p.y)
    };
    assert_eq!(
        (at(m1), at(m2), at(find(&built, 0, 6))),
        ((1320, 7800), (1176, 8088), (1080, 8280)),
        "block 618's `orders_x, orders_y` for `1/7`, `1/8` and `0/6`"
    );
}

pub(super) fn chapter_one_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    /// run110's window (`docs/RUNS.md` run110): its clocks cover these
    /// blocks and nothing else.
    const RUN110: (i64, i64) = (610, 630);
    const FIRST: i64 = WIDENING_CHAPTER_ONE.0;
    const LAST: i64 = WIDENING_CHAPTER_ONE.1;
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("g4") else {
        eprintln!("skipping: no golden capture g4 (see docs/RUNS.md run105)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let mut clocks =
        golden("g6").and_then(|(d, _)| crate::capture::indexed::IndexedCapture::open(&d).ok());
    if clocks.is_none() {
        eprintln!("no g6 capture: the GUY clock rows are unchecked (docs/RUNS.md run110)");
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let players = built.sim.players.len();
    let mut script = chapter(1);
    use std::collections::BTreeMap;
    // Every key that parted, against the **first** block it parted on and
    // the value diff there.
    let mut first: BTreeMap<String, (i64, String)> = BTreeMap::new();
    let mut note = |k: String, n: i64, row: String| {
        first.entry(k).or_insert((n, row));
    };
    let mut blocks = 0usize;
    let mut rows = 0usize;
    let mut clock_blocks = 0usize;
    let mut clock_rows = 0usize;
    let mut near_read = 0usize;
    let mut group_blocks = 0usize;
    for f in 0..LAST - 1 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
        // The dump's label is the sim frame plus one, as in chapter two:
        // the tick that spends the word's draws, frame 626, writes block
        // 627.
        let n = f + 1;
        if n < FIRST {
            continue;
        }
        let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = ix.frame_state(at).unwrap();
        let raw = ix.read_frame(at).unwrap();
        blocks += 1;
        // `RON_DEBUG_UNIT=<who>/<o>@<lo>-<hi>`: what this crate's own
        // record held on the block, which the rows below never say.
        crate::diff::harness::debug_watch(&built, n);
        let r = compare(&built, &frame, players);
        for d in &r.diverged {
            note(
                format!("pos {}/{}", d.who, d.o),
                n,
                format!("ours {:?} theirs {:?}", d.ours, d.theirs),
            );
        }
        for d in &r.order_diverged {
            note(
                format!("order:{} {}/{}", d.what.label(), d.who, d.o),
                n,
                format!("{:?}", d.what),
            );
        }
        for (tag, list) in [("angle", &r.angle_diverged), ("angle~", &r.angle_parted)] {
            for d in list {
                note(
                    format!("{tag}:{:?} {}/{}", d.which, d.who, d.o),
                    n,
                    format!("ours {} theirs {}", d.ours, d.theirs),
                );
            }
        }
        for (tag, list) in [("", &r.collide_diverged), ("~", &r.collide_parted)] {
            for d in list {
                note(
                    format!("{}{tag} {}/{}", d.field, d.who, d.o),
                    n,
                    format!("ours {} theirs {}", d.ours, d.theirs),
                );
            }
        }
        for (tag, list) in [("", &r.search_diverged), ("~", &r.search_parted)] {
            for d in list {
                note(
                    format!("start_dist{tag} {}/{}", d.who, d.o),
                    n,
                    format!("ours {} theirs {}", d.ours, d.theirs),
                );
            }
        }
        for d in &r.los_diverged {
            note(
                format!("mylos {}/{}", d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.packed_diverged {
            note(
                format!("packed {}/{}", d.who, d.o),
                n,
                format!("ours {}", d.ours),
            );
        }
        for d in &r.visible_diverged {
            note(
                format!("visible {}/{}", d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.hits_diverged {
            note(
                format!("{} {}/{}", d.field, d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.firing_diverged {
            note(
                format!("{} {}/{}", d.field, d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.death_diverged {
            note(
                format!("death:{} {}/{}", d.field, d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.gather_diverged {
            note(
                format!("gather:{}[{}] {}/{}", d.field, d.at, d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.build_diverged {
            note(
                format!("build:{} {}/{}", d.field, d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.queue_diverged {
            note(
                format!("queue:{} {}/{}", d.field, d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.city_diverged {
            note(
                format!("city:{} {}/{}", d.field, d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for &(who, o) in &r.unlinked_units {
            note(
                format!("unlinked {who}/{o}"),
                n,
                "the dump holds it alone".into(),
            );
        }
        for &(who, o) in &r.extra_units {
            note(
                format!("extra {who}/{o}"),
                n,
                "this crate holds it alone".into(),
            );
        }
        // The other direction on the buildings, which `compare` does not
        // take.
        for b in built.sim.buildings.iter().filter(|b| b.alive) {
            let (w, o) = (i64::from(b.owner), i64::from(b.index));
            if (0..players as i64).contains(&w)
                && !frame.builds.iter().any(|x| x.who == w && x.o == o)
            {
                note(
                    format!("build:extra {w}/{o}"),
                    n,
                    "this crate holds it alone".into(),
                );
            }
        }
        // `near_o`/`near_who`, which no parser carries: read off the
        // block's own text, as chapter two's near test does.
        let near = raw_near(&raw);
        let clock = clocks.as_mut().and_then(|c| {
            let at = c.frames().iter().position(|x| x.number == n)?;
            c.frame_state(at).ok()
        });
        if clock.is_some() {
            clock_blocks += 1;
        }
        // **The army's group record**, which run110 alone prints
        // (`GROUPS=9`, item 399) and which no row read until item 530. The
        // pool slot is not an identity (`CLAUDE.md`): the group is matched
        // on what the slot holds, a live group of this player's army.
        let pool_text = clocks.as_mut().and_then(|c| {
            let at = c.frames().iter().position(|x| x.number == n)?;
            c.read_frame(at).ok()
        });
        if let Some(text) = pool_text.as_deref() {
            let plog = Log::parse(text);
            for (_, b) in plog.frames() {
                for g in crate::gamelog::groups(b) {
                    let (Ok(who), Ok(slot)) = (usize::try_from(g.who), usize::try_from(g.army))
                    else {
                        continue;
                    };
                    if g.num == 0 || g.buildings != 0 || who >= players {
                        continue;
                    }
                    let Some(a) = built.sim.armies.get(who).and_then(|a| a.list.get(slot)) else {
                        continue;
                    };
                    group_blocks += 1;
                    let st = &a.group;
                    for (name, ours, theirs) in [
                        ("facing", i64::from(st.facing), g.facing),
                        ("order_num", i64::from(st.order_num), g.order_num),
                        ("form", i64::from(st.form), g.form),
                        ("speed", i64::from(st.speed), g.speed),
                        ("new_speed", i64::from(st.new_speed), g.new_speed),
                    ] {
                        rows += 1;
                        if ours != theirs {
                            note(
                                format!("group:{name} {who}/army{slot}"),
                                n,
                                format!("ours {ours} theirs {theirs}"),
                            );
                        }
                    }
                }
            }
        }
        // **The record's own rows, ungated by the position**: everything
        // the `UNITDATA` and `GUY` blocks print that `compare` either does
        // not read or reads only where the positions agree.
        for them in &frame.units {
            if !(0..players as i64).contains(&them.who) {
                continue;
            }
            let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                continue;
            };
            let Some(u) = built.sim.unit_by_o(who, o) else {
                continue;
            };
            let un = &built.sim.units[u];
            let near_ours = match un.near {
                Some(sim::combat::Obj::Unit(x)) => (
                    i64::from(built.sim.units[x].owner),
                    i64::from(built.sim.units[x].index),
                ),
                _ => (-1, -1),
            };
            let near_theirs = near.get(&(them.who, them.o)).copied();
            if near_theirs.is_some() {
                near_read += 1;
            }
            let mut own: Vec<(String, i64, Option<i64>)> = vec![
                ("near_o".into(), near_ours.1, near_theirs.map(|p| p.0)),
                ("near_who".into(), near_ours.0, near_theirs.map(|p| p.1)),
                (
                    "heading".into(),
                    i64::from(un.movement.heading.0),
                    them.angle,
                ),
                (
                    "dest_angle".into(),
                    i64::from(un.movement.des_angle.0),
                    them.dest_angle,
                ),
                ("orders_x".into(), i64::from(un.orders_pos.x), them.orders_x),
                ("orders_y".into(), i64::from(un.orders_pos.y), them.orders_y),
                ("tolerance".into(), i64::from(un.tolerance), them.tolerance),
                (
                    "path_recursion".into(),
                    i64::from(un.path_recursion),
                    them.path_recursion,
                ),
                ("idle".into(), i64::from(un.idle), them.idle),
                ("stance".into(), i64::from(un.stance), them.stance),
                // `unit_masks & 4`, the in-danger latch §6.6 step 6 of
                // `docs/GROUPS.md` exempts a group move by (item 530).
                (
                    "in_danger".into(),
                    i64::from(un.in_danger) * 4,
                    them.unit_masks.map(|m| m & 4),
                ),
                ("myspeed".into(), i64::from(un.movement.speed), them.myspeed),
                ("group".into(), built.sim.pool_group_of(u), them.group),
                ("form".into(), i64::from(un.form), them.form),
                ("form_mod".into(), i64::from(un.form_width), them.form_mod),
                (
                    "orders.len".into(),
                    un.orders.len() as i64,
                    Some(them.orders.len() as i64),
                ),
                (
                    "guys.len".into(),
                    un.guys.len() as i64,
                    Some(them.guys.len() as i64),
                ),
            ];
            if un.on_map {
                for (k, ours, theirs) in [
                    ("collide", i64::from(un.collide), them.collide),
                    ("collide_o", i64::from(un.collide_o), them.collide_o),
                    ("collide_who", i64::from(un.collide_who), them.collide_who),
                    ("collide_guy", i64::from(un.collide_guy), them.collide_guy),
                    ("safe", i64::from(un.safe), them.safe),
                    ("start_dist", i64::from(un.start_dist), them.start_dist),
                    (
                        "half_step",
                        i64::from(un.half_step),
                        them.unit_masks.map(|m| i64::from(m & 0x10_0000 != 0)),
                    ),
                ] {
                    own.push((format!("{k}!"), ours, theirs));
                }
            }
            // The same unit in the clock capture, by `(who, o)`: the two
            // files are one game, so the slot is the same unit.
            let g6 = clock
                .as_ref()
                .and_then(|c| c.units.iter().find(|x| x.who == them.who && x.o == them.o));
            for (k, g) in them.guys.iter().enumerate() {
                let Some(og) = un.guys.get(k).copied() else {
                    continue;
                };
                let g = match g6.and_then(|x| x.guys.get(k)) {
                    Some(c) => {
                        // Every key run105 prints on a `GUY` block at
                        // `GUYS=2`; `des` is not one of them.
                        pin_eq!(
                            (g.pos, g.angle, g.des.or(c.des)),
                            (c.pos, c.angle, c.des),
                            "block {n}, {who}/{o} guy {k}: run110 is not run105's game"
                        );
                        clock_rows += 1;
                        c
                    }
                    None => g,
                };
                let (body, facing, des, des_angle) = match og.follow {
                    Some(b) => (b.body, b.facing, b.des, b.des_angle),
                    None if k == 0 => (
                        un.movement.body,
                        un.movement.facing,
                        un.pos,
                        un.movement.heading,
                    ),
                    None => (
                        un.movement.body,
                        un.movement.facing,
                        un.pos,
                        un.movement.facing,
                    ),
                };
                let track = og.follow.map_or((0, 0), |b| b.track);
                // What the figure last swung at, `(o, who)` (item 530).
                let aim = match og.aim {
                    Some(sim::combat::Obj::Unit(x)) => (
                        i64::from(built.sim.units[x].index),
                        i64::from(built.sim.units[x].owner),
                    ),
                    Some(sim::combat::Obj::Building(b)) => (
                        i64::from(built.sim.buildings[b].index),
                        i64::from(built.sim.buildings[b].owner),
                    ),
                    None => (-1, -1),
                };
                for (name, ours, theirs) in [
                    ("g.x", i64::from(body.pos.x), g.pos.map(|p| p.x)),
                    ("g.y", i64::from(body.pos.y), g.pos.map(|p| p.y)),
                    ("g.angle", i64::from(facing.0), g.angle),
                    ("g.des_x", i64::from(des.x), g.des.map(|p| p.x)),
                    ("g.des_y", i64::from(des.y), g.des.map(|p| p.y)),
                    ("g.des_angle", i64::from(des_angle.0), g.des_angle),
                    ("g.cur_anim", i64::from(og.anim), g.cur_anim),
                    ("g.cur_time", i64::from(og.cur_time), g.cur_time),
                    ("g.end_time", i64::from(og.end_time), g.end_time),
                    ("g.last_time", i64::from(og.last_time), g.last_time),
                    ("g.gpiece", i64::from(og.gpiece), g.gpiece),
                    ("g.ox", aim.0, g.ox),
                    ("g.whom", aim.1, g.whom),
                    // `guy_flags & 0x20`, which `Unit::set_in_danger`
                    // raises beside the unit's bit; run110 alone prints it
                    // (item 530).
                    (
                        "g.flags&0x20",
                        i64::from(un.guy_flag_0x20) * 0x20,
                        g.guy_flags.map(|f| f & 0x20),
                    ),
                    ("g.stopped", i64::from(og.stopped), g.stopped),
                    ("g.hold_attack", i64::from(og.pending_attack), g.hold_attack),
                    (
                        "g.queued_attack",
                        i64::from(og.queued_attack),
                        g.queued_attack,
                    ),
                    ("g.track_dx", i64::from(track.0), g.track.map(|t| t.0)),
                    ("g.track_dy", i64::from(track.1), g.track.map(|t| t.1)),
                    ("g.last_speed", i64::from(body.last_speed), g.last_speed),
                    ("g.avg_speed", i64::from(body.avg_speed), g.avg_speed),
                ] {
                    own.push((format!("{name}[{k}]"), ours, theirs));
                }
            }
            for (name, ours, theirs) in own {
                let Some(theirs) = theirs else { continue };
                rows += 1;
                if ours != theirs {
                    note(
                        format!("{name} {who}/{o}"),
                        n,
                        format!("ours {ours} theirs {theirs}"),
                    );
                }
            }
        }
    }
    let on: Vec<String> = first
        .iter()
        .map(|(k, (n, row))| format!("{n} {k}: {row}"))
        .collect();
    eprintln!(
        "ch1 widening: {blocks} blocks [{FIRST}, {LAST}), {rows} record rows, \
         {clock_rows} clock rows over {clock_blocks} blocks, {near_read} near pairs, \
         {} keys parted",
        first.len()
    );
    for r in &on {
        eprintln!("  {r}");
    }
    // **Anti-vacuity.** Every block of the window is in run105, run110
    // covers its own window whole, and the near pairs and clock rows were
    // actually read: each is a row whose agreement would otherwise be a
    // silence.
    //
    // run105 has **no block 900**, as run112 has none: its window is
    // 605..899 and the `!quit` block is 901. Since item 530 the window
    // runs to the capture's end, so that one absence is the dump's own
    // shape and not a truncation.
    const RUN105_NO_BLOCK: i64 = 900;
    let absent = usize::from((FIRST..LAST).contains(&RUN105_NO_BLOCK));
    pin_eq!(
        blocks,
        (LAST - FIRST) as usize - absent,
        "run105 no longer carries every block of [{FIRST}, {LAST})"
    );
    if clocks.is_some() {
        pin_eq!(
            clock_blocks,
            (RUN110.1 - RUN110.0) as usize,
            "run110 no longer carries every block of [{}, {})",
            RUN110.0,
            RUN110.1
        );
        // 379 when it was written: sixteen to nineteen living figures on
        // each of run110's twenty blocks.
        pin!(
            clock_rows >= 350,
            "only {clock_rows} guy records came from run110; the clock rows compare nothing"
        );
    }
    if clocks.is_some() {
        // Item 530: who=1's army group on each of run110's blocks from its
        // muster at 615: fourteen.
        pin!(
            group_blocks >= 14,
            "only {group_blocks} army groups came from run110's pool; the group rows compare nothing"
        );
    }
    // 2993 when it was written: every unit of both players on every block.
    pin!(
        near_read >= 2_500,
        "only {near_read} unit-frames carried a near pair; the raw read found nothing"
    );
    let staged = built
        .sim
        .units
        .iter()
        .filter(|u| u.owner < 2 && u.index >= 6)
        .count();
    pin_eq!(
        staged,
        6,
        "the window does not hold chapter one's six hoplites"
    );
    // **The map, and its shape is the finding** (item 445). Before the
    // enemy ladder landed, the window `[605, 630)` held 87 keys and the
    // earliest row on the word's own frame was `0/8` on **625**, a block
    // under the word: position, order list, path, `collide`, the clock,
    // `recharging`, `visible`, and `damage 1/6` on 626 downstream of it.
    // All of those close with §6 step 3's arm B (`docs/COMBAT.md` §48),
    // and the word went 626 → 774. The window moved with it.
    //
    // **Item 530 moved the word 774 → 900, the capture's end**, and the
    // window with it (`docs/ORDERS.md` §22). Four keys went into the
    // record first and two of them were the answer: `in_danger`
    // (`unit_masks & 4`) and `g.flags&0x20` parted on **617** on all six
    // hoplites, 148 frames under the order kind the item was booked on;
    // the army group's `facing` parted on 617 from run110's pool; and the
    // guy's `ox`/`whom`. `stance` closed as an instrument row: the
    // original writes one byte for every stance panel and this crate
    // wrote only the combat half. The map fell from 88 keys to 46, and
    // every `dest_angle` and `orders_x/y` row closed with
    // `Unit::work`'s unconditional `update_action`.
    //
    // What stands, by family, and none spends a draw in the capture:
    //
    // - **605, the floor**: the city record of `0/2000`, which this crate
    //   holds empty (`busy`, `filled`, `land`, `space`, `ter`, and
    //   `1/2000`'s by one), and `form` on the ten pre-existing units.
    //   All older than the chapter's first staged line.
    // - ~~**610, `g.gpiece`** on the ten pre-existing units: one age
    //   bracket under the dump's~~ and ~~**617–623, `g.end_time`** on
    //   three citizens, an idle length beside it~~: closed by item 1281,
    //   the age's `update_gpiece` (`docs/TECH.md`, "The piece moves with
    //   the age").
    // - ~~**611 and 616, the births**: `form` −1 against 0 on all six
    //   hoplites.~~ Closed by item 1330, `Unit::init`'s `form`.
    // - ~~**616, the army group's `speed`/`new_speed`**, 0 against 25 on
    //   every block run110 prints~~: closed by item 2002, the member's
    //   search normalizing its group (`find_nearby_target`'s add,
    //   `docs/GOLDEN.md` §73).
    let got: Vec<(&str, i64)> = first.iter().map(|(k, (n, _))| (k.as_str(), *n)).collect();
    // Item 1605 took the human's twelve `0/2000 city:*` rows: the human
    // leader takes the sweep (`docs/AI.md` §169).
    let measured = [
        ("form 0/0", 605),
        ("form 0/1", 605),
        ("form 0/2", 605),
        ("form 0/3", 605),
        ("form 0/4", 605),
        ("form 0/5", 605),
        ("form 1/1", 605),
        ("form 1/2", 605),
        ("form 1/3", 605),
        ("form 1/4", 605),
        ("form 1/5", 605),
    ];
    pin_eq!(
        got,
        measured.to_vec(),
        "chapter one's word frame no longer widens the way item 445 \
         measured it; re-pin this map and say so in docs/COMBAT.md §48"
    );
}

pub(super) fn chapter_two_s_first_attack_orders_are_the_dump_s() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = chapter(2);
    // `(who, o)` → the first frame this crate has an attack order on it.
    let mut first: std::collections::BTreeMap<(u8, i16), i64> = std::collections::BTreeMap::new();
    for _ in 0..640 {
        let f = built.sim.frame;
        script.stage(f, &mut built, &loaded);
        for u in 0..built.sim.units.len() {
            let un = &built.sim.units[u];
            if !un.alive() || un.owner > 1 || !(6..=11).contains(&un.index) {
                continue;
            }
            if un
                .orders
                .iter()
                .any(|o| matches!(o.body, sim::orders::Body::Attack(_)))
            {
                first.entry((un.owner, un.index)).or_insert(f);
            }
        }
        built.tick();
    }
    // The dump's own first-`ATTACKORDER` frame per unit, read from run112.
    let theirs: [((u8, i16), i64); 9] = [
        ((0, 6), 635),
        ((0, 7), 635),
        ((0, 8), 635),
        ((0, 9), 621),
        ((0, 10), 621),
        ((0, 11), 621),
        ((1, 6), 635),
        ((1, 7), 635),
        ((1, 8), 635),
    ];
    // Ours. **Pinned in no direction**, and since item 457 it is the
    // dump's own nine, frame for frame.
    //
    // The history is the point. Item 415 had the hoplites ordering at
    // **616**, nineteen frames early and on the wrong mechanism. Item 447
    // put `is_seen` into `valid_target` (`docs/COMBAT.md` §31) and the
    // spurious 616 went away without the dump's 635 arriving, because the
    // other half of `UnitData::is_seen` — `ObjectData::visible` — was not
    // modelled: three rows moved from *wrong and early* to *absent*.
    // Item 457 modelled it (`docs/VISION.md` §7), and the original's
    // `1/6` now accepts `0/10` at 635 here for the reason it does there —
    // the slinger set its own `visible` bit for player 1 at **631** by
    // attacking a hoplite (`Unit::set_attacking@005ff5b0`), and player 1's
    // line of sight never reaches that cell.
    let ours: [((u8, i16), i64); 9] = [
        ((0, 6), 635),
        ((0, 7), 635),
        ((0, 8), 635),
        ((0, 9), 621),
        ((0, 10), 621),
        ((0, 11), 621),
        ((1, 6), 635),
        ((1, 7), 635),
        ((1, 8), 635),
    ];
    let got: Vec<((u8, i16), i64)> = first.into_iter().collect();
    assert_eq!(
        got,
        ours.to_vec(),
        "chapter two's engagement timeline moved. The dump's own is \
         {theirs:?}; re-pin `ours` and say so in docs/GOLDEN.md §6"
    );
    // **All nine agree with the original exactly.** The count is asserted
    // separately from the list because it is the row that reads at a
    // glance, and because a regression that swapped one squad's rightness
    // for another's would still pass a length check.
    let agree = theirs
        .iter()
        .filter(|(k, f)| got.iter().any(|(g, n)| g == k && n == f))
        .count();
    assert_eq!(
        agree,
        theirs.len(),
        "chapter two's engagement timeline no longer matches the dump on \
         every one of its nine figures"
    );
    // The original issues nothing at all on the frame the word parts.
    assert!(
        !theirs.iter().any(|(_, f)| *f == 616),
        "the dump's own timeline now has an attack order at 616"
    );
}

pub(super) fn chapter_two_s_hoplite_captain_refused_a_cell_three_searches_reached() {
    let Some((dump, _)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
        return;
    };
    let text = crate::capture::read(&dump);
    let at = |f: i64| ch2_dump_units(&text, f);
    let (f616, f621, f635) = (at(616), at(621), at(635));
    assert!(
        !f616.is_empty() && !f621.is_empty() && !f635.is_empty(),
        "run112's dump no longer carries frames 616, 621 and 635"
    );
    // **The searcher never moves.** `1/6` stands on its seat through both
    // of its own searches, so its cell is one number in both rows.
    for f in [&f616, &f621, &f635] {
        let h = f[&(1, 6)];
        assert_eq!(
            (h[0], h[1]),
            (2424, 7800),
            "the hoplite captain has left its seat; §30's cell argument \
             assumes it stands still from 616 to 635"
        );
    }
    let hoplite = (cell(2424), cell(7800));
    assert_eq!(hoplite, (3, 10), "the hoplite captain's object-grid cell");
    // 616: every bowman sits in one cell, and the captain's `near_o` is -1
    // — the search accepted nothing from it.
    for o in 6..=8 {
        let b = f616[&(0, o)];
        assert_eq!(
            (cell(b[0]), cell(b[1])),
            (1, 10),
            "bowman 0/{o} is no longer in the cell §30 argues on"
        );
    }
    assert_eq!(
        f616[&(1, 6)][2],
        -1,
        "the hoplite captain's near_o at 616 is no longer -1; §30's \
         'the search accepted nothing' rests on it"
    );
    // 635: the slinger it *does* accept sits in the same cell the bowmen
    // sat in, and the bowmen are still there.
    let s = f635[&(0, 10)];
    assert_eq!(
        (cell(s[0]), cell(s[1])),
        (1, 10),
        "slinger 0/10 is not in the bowmen's cell at 635; §30's argument \
         is that one cell was reached and refused, then reached and taken"
    );
    assert_eq!(
        f635[&(1, 6)][2],
        10,
        "the hoplite captain's near_o at 635 is no longer 0/10"
    );
    let b7 = f635[&(0, 7)];
    assert_eq!(
        (cell(b7[0]), cell(b7[1])),
        (1, 10),
        "the bowmen have left the cell between 616 and 635, so the two \
         frames are no longer comparable"
    );
    // The two searches that cross the same pair the other way, which is
    // what makes this a measurement and not one unit's oddity.
    assert_eq!(
        (
            f621[&(0, 9)][2],
            (cell(f621[&(0, 9)][0]), cell(f621[&(0, 9)][1]))
        ),
        (6, (1, 10)),
        "the slinger captain's birth-frame search no longer finds 1/6 \
         from the bowmen's cell"
    );
    assert_eq!(
        (
            f635[&(0, 6)][2],
            (cell(f635[&(0, 6)][0]), cell(f635[&(0, 6)][1]))
        ),
        (6, (1, 10)),
        "the bowman captain no longer finds 1/6 from its own cell at 635"
    );
    // **Anti-vacuity.** Both searchers are on the arm that searches: the
    // dump's `idle` is 1 on a birth-frame search and >1 on the grid one,
    // so neither row is a unit that simply never thought.
    assert_eq!(
        (f616[&(1, 6)][3], f621[&(0, 9)][3], f635[&(1, 6)][3]),
        (1, 1, 4),
        "the idle counters that put these three rows on `Unit::think`'s \
         search arm have moved"
    );
}

pub(super) fn chapter_two_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    /// run112's one missing block inside its own window: 605..899, then
    /// the `!quit` block at 901.
    const RUN112_NO_BLOCK: i64 = 900;
    /// run118's last block (`docs/RUNS.md` run118): its clocks stop here.
    const RUN118_LAST_BLOCK: i64 = 846;
    /// The first frame compared: **run112's own first block**, which is
    /// the only floor that cannot hide a row. It stood at 633 until item
    /// 470, three frames under the word — and 633 was above a live
    /// divergence: `order 0/10` and `pos 0/10` part at **630**, on a
    /// chase this crate drops one frame before the original does, and no
    /// run had ever said so. A floor chosen to sit "just under the word"
    /// is the same shape as §33.4's unearned green: an instrument that
    /// agrees because it is not looking. Declared in
    /// [`WIDENING_CHAPTER_TWO`] beside the `WIDENINGS` row, so the floors
    /// guard reads the word against the same window this walks.
    const FIRST: i64 = WIDENING_CHAPTER_TWO.0;
    /// One past the last. 680 is the word; four frames past it is enough
    /// to say the parting opens *there* and not later.
    const LAST: i64 = WIDENING_CHAPTER_TWO.1;
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    // **The clock capture** (item 510). run112 was taken at `GUYS=2`, and
    // `GuyData::log_data@005de6c0` stops a `GUY` block after `ox` at that
    // level: `cur_anim`, `cur_time`, `end_time`, `last_time`,
    // `des_angle`, `hold_attack` and `queued_attack` are **not in the
    // file**. run118 is the same lobby, the same seed and the same
    // `chapter2.cmd` at `GUYS=4` (`docs/RUNS.md` run118), truncated at
    // block 846 — which contains the whole of this window.
    //
    // It is opened as a *second* source rather than as the comparator,
    // because the draw stream this chapter is scored on is run112's
    // trace. [`clock_agrees`] below is what makes borrowing one capture's
    // field into another's walk honest: every key both files print is
    // asserted equal on every compared block, so a clock from a different
    // game cannot arrive quietly.
    let mut clocks =
        golden("ch2g4").and_then(|(d, _)| crate::capture::indexed::IndexedCapture::open(&d).ok());
    if clocks.is_none() {
        eprintln!("no ch2g4 capture: the GUY clock rows are unchecked (docs/RUNS.md run118)");
    }
    let mut clock_rows = 0usize;
    let mut clock_blocks = 0usize;
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let players = built.sim.players.len();
    let mut script = chapter(2);
    // Every key that parted, against the **first** frame it parted on —
    // a residue standing before the window opened is not this word's.
    let mut first: std::collections::BTreeMap<String, i64> = std::collections::BTreeMap::new();
    let mut blocks = 0usize;
    let mut deaths = 0usize;
    for f in 0..LAST - 1 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
        // The dump's frame label is the sim frame plus one throughout this
        // chapter (`docs/COMBAT.md` §30.1), so the block this tick produced
        // is `f + 1`.
        let n = f + 1;
        if n < FIRST {
            continue;
        }
        let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = ix.frame_state(at).unwrap();
        let r = compare(&built, &frame, players);
        blocks += 1;
        let mut note = |k: String| {
            first.entry(k).or_insert(n);
        };
        for d in &r.diverged {
            eprintln!(
                "  {n} pos {}/{} ours {:?} theirs {:?}",
                d.who, d.o, d.ours, d.theirs
            );
            note(format!("pos {}/{}", d.who, d.o));
        }
        for d in &r.order_diverged {
            eprintln!("  {n} order {}/{} {:?}", d.who, d.o, d);
        }
        for d in &r.angle_diverged {
            eprintln!("  {n} angle {}/{} {:?}", d.who, d.o, d);
        }
        for (who, o) in &r.unlinked_units {
            note(format!("unlinked {who}/{o}"));
        }
        for (who, o) in &r.extra_units {
            note(format!("extra {who}/{o}"));
        }
        for d in &r.order_diverged {
            note(format!("order {}/{}", d.who, d.o));
        }
        for d in &r.los_diverged {
            note(format!("los {}/{}", d.who, d.o));
        }
        for d in &r.angle_diverged {
            note(format!("angle {}/{}", d.who, d.o));
        }
        for d in &r.collide_diverged {
            note(format!("collide {}/{}", d.who, d.o));
        }
        for d in &r.search_diverged {
            note(format!("search {}/{}", d.who, d.o));
        }
        for d in &r.packed_diverged {
            note(format!("packed {}/{}", d.who, d.o));
        }
        for d in &r.visible_diverged {
            note(format!("visible {}/{}", d.who, d.o));
        }
        // **The hit-point record** (item 484), keyed on the **field**
        // rather than the unit: a key that collapses a record's fields
        // hides all but the first for the rest of the run, which is
        // parked 452's lesson and `CLAUDE.md`'s "a row is a field, never
        // a unit". The value diff goes beside it, because a walk that
        // can see a death and not the wounds that made it is what this
        // row exists to fix (`docs/COMBAT.md` §38.4).
        for d in &r.hits_diverged {
            eprintln!(
                "  {n} {} {}/{} ours {} theirs {}",
                d.field, d.who, d.o, d.ours, d.theirs
            );
            note(format!("{} {}/{}", d.field, d.who, d.o));
        }
        // **The firing record** (item 485), on the same terms: the reload
        // clock and the overkill window are written at `UNITDATA`'s own
        // indent on every unit of every block and were compared nowhere,
        // so 484's ladder could say that a wound arrived a frame late and
        // not whether the *shot* was fired on the original's frame.
        for d in &r.firing_diverged {
            eprintln!(
                "  {n} {} {}/{} ours {} theirs {}",
                d.field, d.who, d.o, d.ours, d.theirs
            );
            note(format!("{} {}/{}", d.field, d.who, d.o));
        }
        // **The death-object list** (item 491, §42.1) — `DEATH_OBJS`,
        // both directions, keyed on the field. run112's window carries
        // exactly one: `1/8`, `first_frame 683`, `cur_anim 17`, on every
        // block from 684 to the end of the capture.
        deaths += r.death_compared;
        for d in &r.death_diverged {
            eprintln!(
                "  {n} death:{} {}/{} ours {} theirs {}",
                d.field, d.who, d.o, d.ours, d.theirs
            );
            note(format!("death:{} {}/{}", d.field, d.who, d.o));
        }
        // **The four `FrameResult` carries that this walk ignored until
        // item 466.** They are empty over this window — chapter two
        // stages nine soldiers and no building — but "widened whole"
        // has to mean every vector the comparison produces, not the ten
        // the mechanic happened to care about, or the claim decays into
        // the shape `CLAUDE.md` warns about: an instrument that agrees
        // because it is not looking. Adding them changed no row, which
        // is the only way to find that out.
        for d in &r.gather_diverged {
            note(format!("gather {}/{}", d.who, d.o));
        }
        for d in &r.build_diverged {
            note(format!("build {}/{}", d.who, d.o));
        }
        for d in &r.queue_diverged {
            note(format!("queue {}/{}", d.who, d.o));
        }
        for d in &r.city_diverged {
            note(format!("city {}/{}", d.who, d.o));
        }
        // **The `GUY` record whole** (item 510), which this walk had
        // never opened. `compare` carries no guy row at all — the one
        // window that compares a figure's animation clock is Great
        // Lakes' `run100_s_word_block_is_every_record_the_dump_carries`,
        // and chapter two's word has been an *animation* draw since 695.
        // Eleven items ran on a window that could not see the clock the
        // draw is spent on.
        //
        // `hold_attack` and `queued_attack` are new to the parser in this
        // landing and are the two that decide the item: an attack the
        // swing frame defers lands in the first and costs the wrap one
        // draw; one asked for while an attack is already playing lands in
        // the second and costs it two (`docs/COMBAT.md` §45).
        let clock = clocks.as_mut().and_then(|c| {
            let at = c.frames().iter().position(|x| x.number == n)?;
            c.frame_state(at).ok()
        });
        if clock.is_some() {
            clock_blocks += 1;
        }
        for them in &frame.units {
            if !(0..players as i64).contains(&them.who) {
                continue;
            }
            let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                continue;
            };
            let Some(u) = built.sim.unit_by_o(who, o) else {
                continue;
            };
            // The same unit in the clock capture, by `(who, o)` — the
            // two files are the same game, so the slot is the same unit.
            let g4 = clock
                .as_ref()
                .and_then(|c| c.units.iter().find(|x| x.who == them.who && x.o == them.o));
            let un = &built.sim.units[u];
            for (k, g) in them.guys.iter().enumerate() {
                let Some(og) = un.guys.get(k).copied() else {
                    continue;
                };
                // **The clock's own record, and the check that it is this
                // game's.** Every key run112 prints on a `GUY` block is
                // asserted equal in run118 before a key run112 does *not*
                // print is read from it.
                let g = match g4.and_then(|x| x.guys.get(k)) {
                    Some(c) => {
                        pin_eq!(
                            (g.pos, g.angle),
                            (c.pos, c.angle),
                            "block {n}, {who}/{o} guy {k}: run118 is not run112's game"
                        );
                        clock_rows += 1;
                        c
                    }
                    None => g,
                };
                let (body, facing, des, des_angle) = match og.follow {
                    Some(b) => (b.body, b.facing, b.des, b.des_angle),
                    None if k == 0 => (
                        un.movement.body,
                        un.movement.facing,
                        un.pos,
                        un.movement.heading,
                    ),
                    None => (
                        un.movement.body,
                        un.movement.facing,
                        un.pos,
                        un.movement.facing,
                    ),
                };
                let track = og.follow.map_or((0, 0), |b| b.track);
                for (name, ours, theirs) in [
                    ("g.x", i64::from(body.pos.x), g.pos.map(|p| p.x)),
                    ("g.y", i64::from(body.pos.y), g.pos.map(|p| p.y)),
                    ("g.angle", i64::from(facing.0), g.angle),
                    ("g.des_x", i64::from(des.x), g.des.map(|p| p.x)),
                    ("g.des_y", i64::from(des.y), g.des.map(|p| p.y)),
                    ("g.des_angle", i64::from(des_angle.0), g.des_angle),
                    ("g.cur_anim", i64::from(og.anim), g.cur_anim),
                    ("g.cur_time", i64::from(og.cur_time), g.cur_time),
                    ("g.end_time", i64::from(og.end_time), g.end_time),
                    ("g.last_time", i64::from(og.last_time), g.last_time),
                    ("g.gpiece", i64::from(og.gpiece), g.gpiece),
                    ("g.stopped", i64::from(og.stopped), g.stopped),
                    ("g.hold_attack", i64::from(og.pending_attack), g.hold_attack),
                    (
                        "g.queued_attack",
                        i64::from(og.queued_attack),
                        g.queued_attack,
                    ),
                    ("g.track_dx", i64::from(track.0), g.track.map(|t| t.0)),
                    ("g.track_dy", i64::from(track.1), g.track.map(|t| t.1)),
                    ("g.last_speed", i64::from(body.last_speed), g.last_speed),
                    ("g.avg_speed", i64::from(body.avg_speed), g.avg_speed),
                ] {
                    let Some(theirs) = theirs else { continue };
                    if ours != theirs {
                        let key = format!("{name}[{k}] {who}/{o}");
                        let before = first.len();
                        first.entry(key.clone()).or_insert(n);
                        if first.len() != before {
                            eprintln!("  {n} {key}: ours {ours} theirs {theirs}");
                        }
                    }
                }
            }
        }
    }
    // run112 has **no block 900**: its window is 605..899 and the
    // `!quit` block is 901 (`docs/RUNS.md` run112). Since item 523 the
    // window runs to the capture's end, so that one absence is the
    // dump's own shape and not a truncation.
    let absent = usize::from((FIRST..LAST).contains(&RUN112_NO_BLOCK));
    pin_eq!(
        blocks,
        (LAST - FIRST) as usize - absent,
        "run112's dump no longer carries every frame of [{FIRST}, {LAST})"
    );
    // **Anti-vacuity for the borrowed clock** (item 510), and it is the
    // one row here whose agreement would otherwise be a silence twice
    // over: run112 prints no `cur_anim`, `cur_time`, `end_time`,
    // `last_time`, `des_angle`, `gpiece`, `hold_attack` or
    // `queued_attack` on a per-frame `GUY` block at all, so before this
    // landing every one of those rows compared `Some(ours)` against
    // `None` and was skipped. A reader that stopped finding run118's
    // blocks would do exactly the same thing and print nothing, so the
    // count of guy records actually taken from it is pinned. Nine staged
    // figures, ten pre-existing ones and one guy each, over 123 blocks.
    if clocks.is_some() {
        // run118 is truncated at 846, and since item 523 the window runs
        // past it to run112's end: the clock rows cover `[FIRST, 847)` and
        // the frames above are compared on run112's own `GUYS=2` fields.
        let clocked = LAST.min(RUN118_LAST_BLOCK + 1);
        pin_eq!(
            clock_blocks,
            (clocked - FIRST) as usize,
            "run118 no longer carries every block of [{FIRST}, {clocked}); it is \
             truncated at {RUN118_LAST_BLOCK}"
        );
        pin!(
            clock_rows >= 2000,
            "only {clock_rows} guy records came from run118; the clock rows \
             are comparing nothing"
        );
        eprintln!("ch2 clock: {clock_rows} guy records read from run118");
    }
    // **Anti-vacuity for the death list**, which is the one row here
    // whose agreement is a *silence*: three fields on every block from
    // 684 to the ceiling, for the one death run112's window carries. A
    // reader that stopped parsing `DEATH_OBJS` would print no
    // divergence and neither would one that matched nothing, so the
    // count is pinned beside the map (parked 449's shape). It was
    // fifteen blocks while the ceiling stood at 699; item 502 moved the
    // word to 725 and the ceiling to 729, so it is forty-five — and
    // that the count is still exactly `3 × (LAST − 684)` is itself the
    // check that the window gained no **second** death.
    //
    // **Item 495 moved the ceiling to 766 and the window holds three
    // deaths now**, each on the original's own frame: `1/8` from 684,
    // `0/11` (dies 729) from 730 and `1/6` (dies 743) from 744 — the
    // hoplite the original's bowmen had been shooting since 696, which
    // this crate now targets too. Every one of the 420 comparisons
    // agrees.
    //
    // **Item 523 ran the window to run112's end and it holds four**:
    // `1/7`, the last hoplite, dies on 816 and is carried from 817. Each
    // record is on every block from its first to the end, less the one
    // block run112 does not have.
    let carried =
        |from: i64| (LAST - from) as usize - usize::from((from..LAST).contains(&RUN112_NO_BLOCK));
    pin_eq!(
        deaths,
        3 * (carried(684) + carried(730) + carried(744) + carried(817)),
        "the death-object comparison's own width moved; 1/8 dies on 683          and the dump carries its record on every block from 684 to the          end of run112"
    );
    // **Anti-vacuity**: the window has to hold the cast the chapter is
    // about, or an empty comparison reads as agreement.
    let ninth = built
        .sim
        .units
        .iter()
        .filter(|u| u.owner < 2 && u.index >= 6)
        .count();
    pin_eq!(
        ninth,
        9,
        "the window does not hold chapter two's nine staged figures"
    );
    // **The map the widening exists to pin**, and its shape is the
    // finding. Item 462 measured the *values* parting at **635** on six
    // `Target` rows — every one of them a tie among near-equidistant
    // identical figures that this crate's ranking broke the wrong way.
    //
    // Item 466 found the six to be **two** faults, one per squad, and
    // landed one of them (`docs/COMBAT.md` §33). What landed is §33.1:
    // `ObjectData::targeted` is a decaying crowding penalty quartered
    // every sixteenth frame, which this crate bumped per order — three
    // times for a squad handed one target through the mirror — and never
    // decayed, so a stale `+3` pushed `1/8` out of the bowmen's exact
    // tie. The bowmen's three rows are gone; `order 0/6`, `0/7` and `0/8`
    // no longer appear at all, and nor do the `angle` rows that followed
    // them on 636.
    //
    // **The hoplites' three closed on item 470**, which landed §33.2:
    // `compare_target`'s fourth argument **divides** by the damage it
    // would deal for a computer leader where a human's multiplies, so
    // who=1's hoplites rank two slingers above three fatter bowmen and
    // this crate now does too. `order 1/6`, `1/7` and `1/8` move 635 →
    // **636** and the values no longer part at 635 at all. The cost is
    // `chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame`,
    // which is **red on purpose** — `docs/COMBAT.md` §33.4 and §35.
    //
    // **`order 0/10` / `pos 0/10` at 630 are what the old floor hid.**
    // They are not item 470's: they stand identically with §33.2 disabled
    // (measured both ways on this window), and they are the first row on
    // the map now that it starts at run112's own first block. This crate
    // drops `0/10`'s chase on the frame it first reads in range; the
    // original drops it one frame later (`docs/COMBAT.md` §35.2).
    //
    // `order 0/5` / `pos 0/5` at 639-640 are a **citizen** far from the
    // engagement; they are named here so a regression in them cannot hide
    // behind the engagement, and they are nobody's item yet
    // (`docs/COMBAT.md` §32.5).
    //
    // **Item 479 closed all five of 472's rows and the map moved 645 →
    // 671**, sixty-five frames of run112 — every record the dump carries,
    // both directions, from its own first block to 670 — going to nought.
    // `do_move`'s captain retarget and `Unit::change_target` under it
    // (`docs/COMBAT.md` §37): `0/9` retargets to `1/6` on 645 as the dump
    // does, so `pos 0/9` no longer walks on, `order 0/10` and `order
    // 0/11` take the new target down the `o_down` chain on the same
    // block, and `visible 0/9` arrives on 646 rather than 648.
    //
    // **Item 481 closed all six of 671's rows and they were not a tie.**
    // `do_move@005f7b30:207`'s `ptype->max_range != 0` opens a *block*
    // whose brace closes past the captain retarget, and item 479 wrote
    // the gate as a conjunct of the in-range kill alone — so a **melee**
    // captain reached the retarget. run112's hoplite captain `1/6`
    // carries `near_o 10` from block 621 to the end of the window, and
    // on 671 the plain reach admits `0/10`, so this crate switched all
    // three hoplites off `0/11` down the `o_down` chain. The dump has
    // `1/6`, `1/7` and `1/8` on `ox 11 whom 0 uid 18` on **every block
    // of the capture** and never retargets them at all; 479's reading
    // that this was §33's tie mirrored onto who=1 is falsified by the
    // dump's own field. `docs/COMBAT.md` §38.
    //
    // **What stands now is 684, and it is a death.** The original's
    // `1/8` dies on 683 — `DEATH_OBJS` with `first_frame 683`, at its
    // 40-hit share of `myhits 120` (`docs/ATTRITION.md`'s figure table)
    // after `damage` 0 → 8 → 17 → 25 → 34 on 656/657/660/682 — and this
    // crate's is still alive, so `extra 1/8` is the row. `order 1/4` at
    // 685 and `pos 1/4` at 686 are a **citizen** forty thousand units
    // from the engagement, the same family as `order 0/5` (§32.5), and
    // all three stood identically with 481's change reverted and this
    // window: they were hidden by the old ceiling at 684, not opened by
    // the fix.
    //
    // **The window's ceiling is what hid the death, and `compare` is
    // what hides its cause**: [`crate::diff::compare`] carries no hit
    // -point row at all — no `myhits`, no `damage`, no `hits_left` —
    // where `run100_s_word_block_is_every_record_the_dump_carries` has
    // had both since §34.4. So this walk can see that `1/8` is *gone*
    // and not that it was wounded differently; the successor item owes
    // that row before it names a mechanism.
    //
    // **Item 484 put the hit-point record in `compare` and the map grew
    // from three rows to eight — the earliest of them 656, twenty-seven
    // blocks *under* the word.** `myhits`, `damage` and `damage_frac`
    // are printed inside the `OBJECT` block on every block of every
    // capture and were compared on none, so this walk was reporting
    // agreement over `[606, 684)` that it had never measured: the floor
    // hid `0/10` until item 470 moved it, the ceiling hid `1/8`'s death
    // until item 481 raised it, and a **field** hid the wounds that made
    // that death until item 484 added it. Three ways for the same
    // instrument to agree because it is not looking.
    //
    // None of the five new rows is this landing's doing — it writes no
    // simulation code — and all five are the same fact from two ends:
    // `Object::take_damage` divides the squad's `myhits` by `uber_size`
    // on the way in (`docs/COMBAT.md` §7.3) and this crate does not, so
    // its hoplite figures absorb a squad's worth apiece and `1/8` never
    // reaches the 40 that kills the original's. `damage 1/8` runs one
    // hit behind from 656 and `extra 1/8` at 684 is its consequence.
    // `docs/COMBAT.md` §40 is the record; naming the mechanism is item
    // 485's.
    //
    // **Item 491 moved the word 683 → 695 and the ceiling with it**, and
    // the map is now thirteen rows that are really two. `order 1/4` — the
    // far-off citizen — is gone, and nothing replaced it under 686.
    //
    // The first is `damage 1/6` at 680, which is §41.3's unmeasured
    // release node, untouched by this landing and still the only row this
    // capture cannot settle.
    //
    // The other twelve are **one arrow**. run112's `0/8` fires at 677 at
    // `1/8`; `0/7`'s arrow kills `1/8` on 683; `0/8`'s finds its target
    // dead, and because it carries `Ammo` flag 4 it does not puncture —
    // it rolls on (§42.2) and comes down two frames later where
    // `check_hit` finds **`1/7`**, for `19+5` on 685. This crate rolled
    // it on too and could not land it until item 495, which also found
    // that the retarget at 696 was `compare_target`'s bearing and not the
    // wound (`docs/COMBAT.md` §46).
    //
    // **Item 496 moved the three `order` rows 696 → 684**, and it moved
    // no code in the simulation: the comparison could not see them.
    // `crate::diff::order::compare_orders` reported a target only when
    // **both** sides named one, so this crate's attack order carrying
    // *nothing* against the dump's `ox 8 whom 1` read as agreement.
    // `Sim::forget` drops a dead object from every attacker's target
    // slot on the frame it dies; the original does not, and run112's
    // three bowmen hold the dead `1/8` from 683 until their reload opens
    // on 695. Thirty-six unit-frames, under the word, quiet.
    // `docs/COMBAT.md` §43.
    //
    // **Item 502 landed §43.2's arm whole and the word moved 695 → 725**,
    // which moved this window's ceiling to 729 with it. The three
    // `order` rows go **back** to 696 — the twelve frames item 496 opened
    // are agreement now, because this crate holds the dead `1/8` where
    // the original does — and all three `recharging` rows close, because
    // `0/7` and `0/8` strike on 695 through `Unit::fight`'s follower
    // inherit and `0/6` does not strike at all.
    //
    // `unit_masks2` joined `compare` in the same landing and reports
    // **nothing on this window**: the bit the dump carries on `0/6` at
    // block 696 is this crate's too. The chapter-wide check is
    // `chapter_two_s_frozen_frames_are_the_dump_s_on_every_unit_frame`,
    // which is where that row's anti-vacuity lives — this window holds
    // one of the seven unit-frames run112 ever marks.
    //
    // **`damage_frac 1/6` at 712 is the raised ceiling's**, not this
    // landing's — the same shape as `extra 1/8` at 684 on item 481 and
    // `1/8`'s wounds at 656 on item 484, and the third time this window
    // has grown a row by being allowed to look further. It is downstream
    // of the parked arrow: this crate's `1/7` is unwounded, so §33's
    // damage weight sends all three bowmen to `1/7` where the dump sends
    // them to `1/6`, and from 712 the two sides are wounding different
    // hoplites. `docs/COMBAT.md` §42.5 is what it waits on.
    //
    // **Item 510 opened the `GUY` record and the map went from eleven
    // rows to thirty-nine.** `compare` carries no guy row at all and this
    // walk built none, so for eleven items the window could not see the
    // animation clock — which is the record chapter two's word has been
    // spent in since 695. Twenty-eight of the new rows were invisible
    // twice over: run112 was taken at `GUYS=2` and prints **no**
    // `cur_anim`, `cur_time`, `end_time`, `last_time`, `des_angle`,
    // `gpiece`, `hold_attack` or `queued_attack` on a per-frame `GUY`
    // block, so even a walk that had built the rows would have compared
    // `Some(ours)` against `None` and skipped every one. The clock is
    // borrowed from run118, asserted to be the same game key by key
    // above. `docs/COMBAT.md` §45.
    //
    // The three families, and none of them is this landing's doing:
    //
    // - **`g.cur_anim` / `g.end_time` / `g.stopped` / `g.hold_attack` on
    //   `0/7` and `0/8` at 726 and `0/6` at 727** are the word's own
    //   delta, seen as values for the first time. The original's figure
    //   comes out of its attack-end wrap on an **attack** slot
    //   (`cur_anim 11`, `12`) and this crate's on the idle (`0`), with
    //   `hold_attack 1` where the dump has 0: the swing that asked while
    //   the attack was still playing lands in `queued_attack` for the
    //   original and in `hold_attack` here, and only the first is paid
    //   inside `Guy::inc_time`'s own loop with a roll. That is the two
    //   `Guy::set_anim+0xf2f < Guy::inc_time+0x271` the word is.
    // - **`g.des_angle` beside each `g.angle`**: the same parting as the
    //   unit-level `angle` rows, one level down, and the reason for the
    //   four above — `Unit::fight` writes the guy's `des_angle` to the
    //   attack angle and `Guy::set_anim` defers the attack while it
    //   differs from the figure's own `angle` (§45.1).
    // - **`g.gpiece` on ten pre-existing units at 606**, the window's
    //   floor, on both players and five types: this crate's piece is
    //   exactly [`sim::anim::PIECES_PER_AGE`] below the dump's on every
    //   one of them — one age bracket. The animation clock agrees on all
    //   ten for the whole window, so the two pieces share their lengths
    //   here and it spends no draw; it is named so it cannot hide, and it
    //   is `docs/ANIM.md`'s.
    // **Item 495 landed the rolled arrow and the map fell from
    // thirty-nine rows to twenty** (`docs/COMBAT.md` §46). Every row from
    // 686 to 727 is gone: `damage`, `damage_frac` and `damage_frame 1/7`
    // close because `0/8`'s shot now comes down on 685 where the
    // original's does; the bowmen's `order`, `angle` and `des_angle` rows
    // close because `compare_target` now ranks at the real bearing and
    // picks `1/6`; and the animation clock 510 opened at 726-727 closes
    // with them. Nothing parts on the word's own frames, 762 and 763.
    // What stands is `damage 1/6` at 680 (§41.3's release node), the ten
    // `g.gpiece` rows at 606 (parked, `docs/ANIM.md`), and the two rows
    // past the word: `0/10`'s clock at 764 and `1/7` — the original's
    // last hoplite — taking a long walk at 765 that this crate's does not.
    //
    // **Item 523 ran the window to run112's end and the map fell from
    // twenty rows to eleven** (`docs/COMBAT.md` §47). The nine rows past
    // 762 were one mechanism: `1/7` was never promoted when its captain
    // `1/6` died on 743, so on 762 it skipped the one-in-five roll the
    // original's captain spends. With the promotion, the lead's
    // `avg_speed` and the hit's launch-to-landing bearing, **nothing
    // parts on any frame from 606 to the end of the capture** except the
    // two standing residues: `damage 1/6` at 680 and the ten `g.gpiece`
    // rows at 606. Both are older than this window's word and neither
    // spends a draw. **Item 1281 closed the `g.gpiece` rows**: the stand-up
    // lands the chapter's third age before 606, and the age now re-pieces
    // every figure (`docs/TECH.md`, "The piece moves with the age").
    let measured = [("damage 1/6", 680)];
    let got: Vec<(&str, i64)> = first.iter().map(|(k, &n)| (k.as_str(), n)).collect();
    pin_eq!(
        got,
        measured.to_vec(),
        "chapter two's word frame no longer widens the way item 523 \
         measured it; re-pin this map and say so in docs/COMBAT.md §47"
    );
}

pub(super) fn chapter_two_s_near_o_is_the_dump_s_on_every_unit_frame() {
    const FIRST: i64 = WIDENING_CHAPTER_TWO.0;
    const LAST: i64 = WIDENING_CHAPTER_TWO.1;
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let legacy = std::env::var_os("RON_DISABLE_GOLDEN_NEAR_INDEX").is_some();
    let verify = std::env::var_os("RON_VERIFY_GOLDEN_NEAR").is_some();
    let text = (legacy || verify).then(|| crate::capture::read(&dump));
    let (mut built, mut ix) = if legacy {
        let log = Log::parse(text.as_ref().unwrap());
        let texts = sibling_texts();
        let logs = siblings(&texts);
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        if refs.is_empty() {
            eprintln!("skipping: no sibling dumps");
            return;
        }
        (stand_up(&loaded, &log, &refs, &trace), None)
    } else {
        let Some((built, ix)) = build_golden("ch2", &dump, &loaded, &trace, false) else {
            return;
        };
        (built, Some(ix))
    };
    let mut script = chapter(2);
    let mut read = 0usize;
    let mut agree = 0usize;
    let mut live = 0usize;
    let mut parted: std::collections::BTreeMap<(i64, i64), i64> = std::collections::BTreeMap::new();
    for f in 0..LAST - 1 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
        let n = f + 1;
        if n < FIRST {
            continue;
        }
        let pairs = if let Some(ix) = &mut ix {
            let pairs = indexed_ch2_near(ix, n);
            if verify {
                assert_eq!(
                    pairs,
                    ch2_dump_near(text.as_ref().unwrap(), n),
                    "whole near map at frame {n}"
                );
            }
            pairs
        } else {
            ch2_dump_near(text.as_ref().unwrap(), n)
        };
        for ((who, o), pair) in pairs {
            let Some(mine) = i16::try_from(o)
                .ok()
                .and_then(|x| built.sim.unit_by_o(who as sim::Player, x))
            else {
                continue;
            };
            read += 1;
            let ours = match built.sim.units[mine].near {
                Some(sim::combat::Obj::Unit(u)) => (
                    i64::from(built.sim.units[u].owner),
                    i64::from(built.sim.units[u].index),
                ),
                // A building incumbent and "none" are both `(-1, -1)` to
                // this comparison: the original prints one pair for both
                // classes and this crate carries a unit's only, so a
                // building would read as a parting here and be one.
                _ => (-1, -1),
            };
            let theirs = (pair.1, pair.0);
            if theirs != (-1, -1) {
                live += 1;
            }
            if ours == theirs {
                agree += 1;
            } else {
                parted.entry((who, o)).or_insert(n);
            }
        }
    }
    // **Anti-vacuity**: the dump has to have been read, and the window has
    // to hold the nine.
    assert!(
        read > 2_000,
        "only {read} unit-frames carried a near pair; run112's dump no \
         longer prints the OBJECT block over [{FIRST}, {LAST})"
    );
    eprintln!("near: {agree} of {read} agree, {live} live; parted {parted:?}");
    let got: Vec<((i64, i64), i64)> = parted.iter().map(|(k, &n)| (*k, n)).collect();
    assert_eq!(
        got,
        NEAR_PARTED.to_vec(),
        "chapter two's `near_o` footprint no longer parts where item 479 \
         measured it; re-pin this and say so in docs/COMBAT.md §37.1"
    );
    assert_eq!(
        (read, live, agree),
        NEAR_TALLY,
        "the near comparison's own width moved; a tally that shrinks is \
         an instrument that stopped looking"
    );
}

pub(super) fn chapter_two_s_hit_points_are_the_dump_s_on_every_unit_frame() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let players = built.sim.players.len();
    let mut script = chapter(2);
    /// One past the last frame read — the chapter's last logged block,
    /// as [`chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame`]
    /// reads it.
    const LAST: i64 = 899;
    let mut compared = 0usize;
    // Unit-frames on which the **dump** carries a non-zero `damage` or
    // `damage_frac`: the live values, without which the row proves
    // nothing.
    let mut live = 0usize;
    // `(who, o)` → the first block the dump ever wounds it on, and ours.
    let mut wounded_theirs: std::collections::BTreeMap<(i64, i64), i64> =
        std::collections::BTreeMap::new();
    let mut wounded_ours: std::collections::BTreeMap<(i64, i64), i64> =
        std::collections::BTreeMap::new();
    // Every field that ever parted, against the first block it parted on.
    let mut first: std::collections::BTreeMap<(&'static str, i64, i64), (i64, String)> =
        std::collections::BTreeMap::new();
    for f in 0..LAST - 1 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
        // The dump's frame label is the sim frame plus one throughout this
        // chapter (`docs/COMBAT.md` §30.1).
        let n = f + 1;
        let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = ix.frame_state(at).unwrap();
        let r = compare(&built, &frame, players);
        compared += r.hits_compared;
        for d in &r.hits_diverged {
            first
                .entry((d.field, d.who, d.o))
                .or_insert((n, format!("ours {} theirs {}", d.ours, d.theirs)));
        }
        for u in &frame.units {
            if !(0..players as i64).contains(&u.who) {
                continue;
            }
            if u.damage.is_some_and(|d| d != 0) || u.damage_frac.is_some_and(|d| d != 0) {
                live += 1;
                wounded_theirs.entry((u.who, u.o)).or_insert(n);
            }
            let Some(mine) = i16::try_from(u.o)
                .ok()
                .and_then(|o| built.sim.unit_by_o(u.who as sim::Player, o))
            else {
                continue;
            };
            let un = &built.sim.units[mine];
            if un.health < un.max_health || un.damage_frac != 0 {
                wounded_ours.entry((u.who, u.o)).or_insert(n);
            }
        }
    }
    // **Anti-vacuity, both halves.** The dump has to have been read at
    // all, and it has to have carried wounds: a window in which every
    // value is zero on both sides passes on a crate that never writes
    // the field, which is exactly what `compare` was before this row.
    assert!(
        compared > 1_000,
        "only {compared} hit-point comparisons were made; run112's dump \
         no longer prints `myhits`/`damage`/`damage_frac` in the OBJECT \
         block"
    );
    eprintln!("ch2: {compared} hit-point comparisons, {live} of them live");
    assert!(
        live > 100,
        "only {live} of run112's unit-frames carry a wound; this row \
         would agree on a crate that never took damage at all"
    );
    // **`myhits` is the dump's on every unit-frame of the chapter**, and
    // so is `damage_frac`'s own agreement where the two sides agree on
    // the damage — the maximum is a baked constant and a wrong one is
    // wrong whatever the engagement does.
    let myhits: Vec<String> = first
        .iter()
        .filter(|((field, ..), _)| *field == "myhits")
        .map(|((f, who, o), (n, v))| format!("{f} {who}/{o} first at {n} \u{2014} {v}"))
        .collect();
    assert_eq!(
        myhits,
        Vec::<String>::new(),
        "a unit's `myhits` parts from run112's: this crate's squad-sized \
         maximum is no longer the original's"
    );
    // **The shape: the same units are wounded on both sides.** The
    // *frames* are not pinned, for the reason
    // `chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame`
    // does not pin its own — a wound arrives when the arrow does — and
    // the *amounts* are not pinned because §7.3's divide is item 485's.
    let theirs: Vec<(i64, i64)> = wounded_theirs.keys().copied().collect();
    let ours: Vec<(i64, i64)> = wounded_ours.keys().copied().collect();
    assert_eq!(
        theirs,
        vec![(0, 9), (0, 10), (0, 11), (1, 6), (1, 7), (1, 8)],
        "run112's own wounded set has changed"
    );
    // Item 485 added `(0, 9)`: with the figure's share as the death
    // threshold (§7.2 step 8) the hoplite squad loses `1/8` on the
    // original's own block and its survivors then reach a second
    // slinger, which the dump also wounds. `(0, 10)` was the one the
    // dump wounded and this crate did not — and **item 502 closed it**,
    // so the two sets are now identical and the assertion is written
    // that way rather than against a copy of the list. §43.2's arm is
    // what reaches it: the original's hoplites fight on where this
    // crate's went idle for a frame each time a target died, and the
    // frames they gain back are the ones that reach `0/10`.
    //
    // ~~**Item 495 took `(0, 9)` back out, and past the word.**~~ The dump
    // first wounds `0/9` on block 792, and **item 523 put it back**:
    // with `1/7` promoted to captain on 743 it spends the one-in-five
    // roll, walks the original's walk from 765, and reaches `0/9` as the
    // dump's does (`docs/COMBAT.md` §47). The sets are identical again,
    // and the assertion is written that way.
    assert_eq!(ours, theirs, "the set this crate wounds has moved");
    assert!(
        theirs.iter().all(|w| ours.contains(w)),
        "a unit the dump wounds and this crate does not"
    );
    // **What parts, printed and pinned by its field.** `damage` and
    // `damage_frac` part on the engagement and the reason is §7.3's
    // divide — item 485's, not this row's.
    let rows: Vec<(&str, i64, i64, i64, &str)> = first
        .iter()
        .map(|((field, who, o), (n, v))| (*field, *who, *o, *n, v.as_str()))
        .collect();
    for r in rows.iter().take(24) {
        eprintln!("  {} {}/{} first at {} — {}", r.0, r.1, r.2, r.3, r.4);
    }
    let fields: std::collections::BTreeSet<&str> = rows.iter().map(|r| r.0).collect();
    assert!(
        !fields.contains("myhits"),
        "`myhits` is in the parting set and the assertion above missed it"
    );
}

pub(super) fn chapter_two_s_frozen_frames_are_the_dump_s_on_every_unit_frame() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let players = built.sim.players.len();
    let mut script = chapter(2);
    /// One past the last frame read — the chapter's last logged block.
    const LAST: i64 = 899;
    let mut compared = 0usize;
    let mut theirs: Vec<(i64, i64, i64)> = Vec::new();
    let mut ours: Vec<(i64, i64, i64)> = Vec::new();
    let mut parted: Vec<String> = Vec::new();
    for f in 0..LAST - 1 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
        // The dump's frame label is the sim frame plus one throughout this
        // chapter (`docs/COMBAT.md` §30.1).
        let n = f + 1;
        let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = ix.frame_state(at).unwrap();
        let r = compare(&built, &frame, players);
        for d in &r.firing_diverged {
            if d.field == "unit_masks2" {
                parted.push(format!(
                    "{n} {}/{} ours {} theirs {}",
                    d.who, d.o, d.ours, d.theirs
                ));
            }
        }
        for u in &frame.units {
            if !(0..players as i64).contains(&u.who) {
                continue;
            }
            let Some(bit) = u.unit_masks2 else { continue };
            compared += 1;
            if bit & i64::from(sim::combat::umask2::NOT_FIRING) != 0 {
                theirs.push((n, u.who, u.o));
            }
            let Some(mine) = i16::try_from(u.o)
                .ok()
                .and_then(|o| built.sim.unit_by_o(u.who as sim::Player, o))
            else {
                continue;
            };
            if built.sim.units[mine].unit_masks2 & sim::combat::umask2::NOT_FIRING != 0 {
                ours.push((n, u.who, u.o));
            }
        }
    }
    // **Anti-vacuity**: the dump has to have been read at all, and it has
    // to have carried marks. An empty list on both sides is what a reader
    // that stopped parsing looks like, and what a crate that never writes
    // the bit looks like.
    assert!(
        compared > 5_000,
        "only {compared} unit-frames carried `unit_masks2`; run112's dump          no longer prints it in the UNITDATA block"
    );
    eprintln!(
        "ch2 frozen frames: {compared} unit-frames, {} marked",
        theirs.len()
    );
    assert_eq!(
        theirs,
        vec![
            (696, 0, 6),
            (736, 1, 6),
            (745, 0, 9),
            (756, 0, 7),
            (756, 0, 8),
            (757, 0, 6),
            (762, 1, 7),
        ],
        "run112's own frozen frames have moved"
    );
    // **Item 495 closed the two this crate missed**, `0/9` at 745 and
    // `1/7` at 762: with the rolled arrow landed and the bowmen on `1/6`,
    // the engagement is the original's through the word, and so is every
    // frame a unit finds its target gone.
    assert_eq!(
        ours, theirs,
        "this crate's frozen frames have moved; they were the original's \
         own seven on item 495"
    );
    // The two the dump has and this crate does not, by name and with the
    // reason: both are past the word, where the draw stream has already
    // parted. Pinned so that a landing which closes them has to say so.
    let missing: Vec<(i64, i64, i64)> = theirs
        .iter()
        .filter(|r| !ours.contains(r))
        .copied()
        .collect();
    assert_eq!(
        missing,
        Vec::<(i64, i64, i64)>::new(),
        "the frozen frames this crate misses have moved"
    );
    assert!(
        missing.iter().all(|&(n, ..)| n > GOLDEN_WORD_CHAPTER_TWO),
        "a frozen frame is missed at or under the word ({GOLDEN_WORD_CHAPTER_TWO}), \
         which is a divergence the word itself should be reporting"
    );
    // Every block the two sides *both* mark agrees field for field, which
    // is the row `compare` gained in the same landing.
    let past: Vec<String> = parted
        .iter()
        .filter(|r| {
            r.split_whitespace()
                .next()
                .and_then(|n| n.parse::<i64>().ok())
                .is_some_and(|n| n <= GOLDEN_WORD_CHAPTER_TWO)
        })
        .cloned()
        .collect();
    assert_eq!(
        past,
        Vec::<String>::new(),
        "`unit_masks2` parts at or under chapter two's word"
    );
}

pub(super) fn chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = chapter(2);
    /// One past the last frame read — the chapter's last logged block.
    const LAST: i64 = 899;
    // `(who, o)` → the first frame the unit's `visible` is non-zero, and
    // the byte it holds there. One entry per side.
    let mut theirs: std::collections::BTreeMap<(i64, i64), (i64, u8)> =
        std::collections::BTreeMap::new();
    let mut ours: std::collections::BTreeMap<(i64, i64), (i64, u8)> =
        std::collections::BTreeMap::new();
    let mut read = 0usize;
    for f in 0..LAST - 1 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
        // The dump's frame label is the sim frame plus one throughout this
        // chapter (`docs/COMBAT.md` §30.1).
        let n = f + 1;
        let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = ix.frame_state(at).unwrap();
        for u in &frame.units {
            if !(0..2).contains(&u.who) {
                continue;
            }
            let Some(v) = u.visible else { continue };
            read += 1;
            if v != 0 {
                theirs.entry((u.who, u.o)).or_insert((n, v as u8));
            }
            // Ours is read on the same unit-frames the dump carries, so
            // that a unit the dump has stopped printing — a dead one —
            // cannot contribute to one side and not the other.
            let Some(mine) = i16::try_from(u.o)
                .ok()
                .and_then(|o| built.sim.unit_by_o(u.who as sim::Player, o))
            else {
                continue;
            };
            let mv = built.sim.units[mine].visible;
            if mv != 0 {
                ours.entry((u.who, u.o)).or_insert((n, mv));
            }
        }
    }
    // **Anti-vacuity.** The dump has to have been read at all, and it has
    // to have carried the arrivals — a window in which every value is zero
    // would pass on a crate that never wrote the field.
    assert!(
        read > 1_000,
        "only {read} unit-frames carried `visible`; run112's dump no \
         longer prints the OBJECT block"
    );
    // **The dump's own nine**, as a guard on the oracle rather than on
    // this crate: if run112 stops saying this, every row below is void.
    let dumped: Vec<((i64, i64), (i64, u8))> = theirs.iter().map(|(k, v)| (*k, *v)).collect();
    assert_eq!(
        dumped,
        vec![
            ((0, 6), (636, 2)),
            ((0, 7), (636, 2)),
            ((0, 8), (636, 2)),
            ((0, 9), (646, 2)),
            ((0, 10), (631, 2)),
            ((0, 11), (640, 2)),
            ((1, 6), (672, 1)),
            ((1, 7), (698, 1)),
            ((1, 8), (665, 1)),
        ],
        "run112's own `visible` arrivals have changed"
    );
    // **The shape, asserted exactly**: the same nine units gain a bit, and
    // each gains the same one — player 1's on the bowmen and slingers,
    // player 0's on the hoplites. This is the mechanic's own claim and
    // nothing about the engagement's timing can excuse a failure here.
    let shape: Vec<((i64, i64), u8)> = ours.iter().map(|(k, v)| (*k, v.1)).collect();
    let want: Vec<((i64, i64), u8)> = theirs.iter().map(|(k, v)| (*k, v.1)).collect();
    assert_eq!(
        shape, want,
        "`visible` no longer arrives on the same units, or no longer \
         carries the same players' bits"
    );
    // **The frames, pinned in no direction** and printed beside the
    // dump's. Every one of the nine is within fourteen frames of the
    // original's, and every gap is the strike's and not the field's.
    let mine: Vec<((i64, i64), (i64, u8))> = ours.iter().map(|(k, v)| (*k, *v)).collect();
    assert_eq!(
        mine,
        vec![
            ((0, 6), (636, 2)),
            ((0, 7), (636, 2)),
            ((0, 8), (636, 2)),
            ((0, 9), (646, 2)),
            ((0, 10), (631, 2)),
            ((0, 11), (640, 2)),
            ((1, 6), (672, 1)),
            ((1, 7), (698, 1)),
            ((1, 8), (665, 1)),
        ],
        "chapter two's `visible` arrivals moved; re-pin them and say so \
         in docs/VISION.md §7"
    );
    // **The three hoplites moved on item 470**, 677/694/679 →
    // 669/678/675 against the dump's 672/698/665, and they moved because
    // all three now chase a slot of their **own**:
    // `find_ordered_collision`'s group pass reads the asker's group and
    // not the last one pushed (`docs/COMBAT.md` §35.1). Before that,
    // `1/8` never struck at all and the shape row above was red; two of
    // the three are closer to the dump's frame and `1/7` is twenty
    // frames under it. Pinned in no direction, as the doc comment says.
    //
    // **All nine land on the dump's own frame.** It was five until item
    // 472, which brought the three hoplites over — 669/678/675 against
    // 672/698/665, the engagement's own residue for four items (§31.6,
    // §35.4) — and eight until item 479, whose ninth is `0/9`'s own: 648
    // → **646**, the dump's. `0/9` is the slinger captain that retargets
    // to `1/6` on block 645 under `do_move`'s captain arm, and it strikes
    // a frame later because `change_target` leaves the attack to the next
    // frame rather than dispatching it (`docs/COMBAT.md` §37.2). Before
    // that it kept walking to `1/8` and struck two frames late.
    //
    // **`visible`'s arrival frame is the frame of a unit's first strike**
    // — `Unit::set_attacking` fires from the tail of `Unit::fight` — so
    // nine of nine says chapter two's whole engagement, both squads and
    // both directions, now opens fire on the original's own frames. That
    // is the strongest thing this capture can say and it is why the
    // count is pinned exactly: a change that bought one squad's frames by
    // losing another's fails here, in both directions.
    let exact = mine
        .iter()
        .filter(|(k, v)| theirs.get(k).is_some_and(|t| t.0 == v.0))
        .count();
    assert_eq!(
        exact, 9,
        "the nine arrivals this crate puts on the dump's own frame are \
         no longer nine"
    );
}

pub(super) fn chapter_two_s_v1z_is_arc_v1z_to_the_last_digit() {
    let Some((dump, _)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
        return;
    };
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let (mut read, mut spent, mut wrong) = (0usize, 0usize, Vec::new());
    for at in 0..ix.frames().len() {
        let n = ix.frames()[at].number;
        let body = ix.read_frame(at).unwrap();
        for (a, _) in super::ammo::blocks(&body) {
            // A spent shot (`flags 1`) keeps its launch `v1z` but carries
            // its landing's `total_time` and `ez`, which
            // [`every_rolled_shot_comes_down_where_the_original_s_does`]
            // reads instead.
            if a.flags & 2 == 0 {
                spent += 1;
                continue;
            }
            read += 1;
            let v = sim::combat::arc_v1z(a.sz as i32, a.ez as i32, a.total_time as i32);
            if super::ammo::tests::printed(v) != a.v1z {
                wrong.push((n, a.o, a.sz, a.ez, a.total_time, a.v1z));
            }
        }
    }
    assert!(wrong.is_empty(), "v1z is not arc_v1z's single: {wrong:?}");
    assert_eq!(
        (read, spent),
        (134, 239),
        "run112 carries 373 AMMO blocks, 134 of them live"
    );
}

pub(super) fn chapter_five_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    use std::collections::{BTreeMap, BTreeSet};
    const FIRST: i64 = WIDENING_CHAPTER_FIVE.0;
    const LAST: i64 = WIDENING_CHAPTER_FIVE.1;
    /// run127's window is 605..899 and its `!quit` block is 901.
    const RUN127_NO_BLOCK: i64 = 900;
    /// **The frame item 549 moved the word from.** With the word at the
    /// capture's end there is no word block to print, so the record the
    /// last word was spent in is printed instead: `1/6` on 739 and 740,
    /// where this crate's swing was `CHAR_ATTACK3` three frames long, and
    /// the round on 742 it then failed to release.
    const MOVED_FROM: i64 = 739;
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("ch5") else {
        eprintln!("skipping: no golden capture ch5 (see docs/RUNS.md run127)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let players = built.sim.players.len();
    let mut script = chapter(5);
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let mut missing: BTreeSet<String> = BTreeSet::new();
    let (mut blocks, mut rows, mut leader_rows) = (0usize, 0usize, 0usize);
    let (mut ammo_theirs, mut ammo_ours) = (0usize, 0usize);
    let ship = |who: i64, o: i64| who < 2 && o >= 6;
    for f in 0..LAST - 1 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
        let n = f + 1;
        if n < FIRST {
            continue;
        }
        let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = ix.frame_state(at).unwrap();
        blocks += 1;
        let (_, k) = crate::diff::harness::widen_block(&built, &frame, players, n, &mut firsts);
        rows += k;
        rows += widen_turrets(&built, &frame, players, n, &mut firsts);
        let raw = ix.read_frame(at).unwrap();
        let flog = Log::parse(&raw);
        for who in 0..2usize {
            let Some(block) = flog.leader_block(n, who as i64) else {
                continue;
            };
            let t = crate::diff::leader::theirs(&block);
            for (k, v) in crate::diff::leader::rows(&loaded, &built, who) {
                let Some(&y) = t.get(&k) else {
                    missing.insert(k);
                    continue;
                };
                leader_rows += 1;
                if v != y {
                    firsts
                        .entry((who as i64, -1, format!("leader:{k}")))
                        .or_insert((n, format!("ours {v} theirs {y}")));
                }
            }
        }
        // **The `AMMO` record, both directions.** A live round is keyed on
        // its shooter and its pool slot, the order `Objects::inc_time`
        // steps it in (`docs/COMBAT.md` §46.4).
        let theirs: BTreeMap<(i64, i64, i64), super::ammo::Ammo> = super::ammo::blocks(&raw)
            .into_iter()
            .filter(|(a, _)| a.flags & 2 != 0)
            .map(|(a, _)| ((a.who, a.o, a.index), a))
            .collect();
        let ours: BTreeMap<(i64, i64, i64), sim::combat::Projectile> = built
            .sim
            .projectiles
            .iter()
            .filter_map(|p| match p.shooter {
                sim::combat::Obj::Unit(u) => {
                    let un = &built.sim.units[u];
                    Some((
                        (i64::from(un.owner), i64::from(un.index), i64::from(p.slot)),
                        *p,
                    ))
                }
                sim::combat::Obj::Building(_) => None,
            })
            .collect();
        ammo_theirs += theirs.len();
        ammo_ours += ours.len();
        // **The value diff on the frame the word moved** (item 549): the
        // round the dump held alone on 742, now in both airs.
        if n == MOVED_FROM + 3 {
            for (key, a) in theirs.iter().filter(|(k, _)| (k.0, k.1) == (1, 6)) {
                eprintln!("  block {n} 1/6 ammo{key:?} theirs {a:?}");
                eprintln!("  block {n} 1/6 ammo{key:?} ours {:?}", ours.get(key));
            }
        }
        let keys: BTreeSet<_> = theirs.keys().chain(ours.keys()).copied().collect();
        for key @ (who, o, slot) in keys {
            let (Some(a), Some(p)) = (theirs.get(&key), ours.get(&key)) else {
                let side = if theirs.contains_key(&key) {
                    "the dump"
                } else {
                    "this crate"
                };
                firsts
                    .entry((who, o, format!("ammo[{slot}]")))
                    .or_insert((n, format!("{side} holds it alone")));
                continue;
            };
            let target = p.target.map_or((-1, -1), |t| match t {
                sim::combat::Obj::Unit(u) => {
                    let tu = &built.sim.units[u];
                    (i64::from(tu.owner), i64::from(tu.index))
                }
                sim::combat::Obj::Building(_) => (-2, -2),
            });
            for (name, mine, dumped) in [
                ("cur_time", i64::from(p.cur_time), a.cur_time),
                ("total_time", i64::from(p.total_time), a.total_time),
                ("sx", i64::from(p.launch.x), a.sx),
                ("sy", i64::from(p.launch.y), a.sy),
                ("ex", i64::from(p.landing.x), a.ex),
                ("ey", i64::from(p.landing.y), a.ey),
                ("whom", target.0, a.whom),
                ("ox", target.1, a.ox),
                ("accuracy", i64::from(p.accuracy), a.accuracy),
                ("splash_area", i64::from(p.splash_area), a.splash_area),
                ("num_guys", i64::from(p.num_guys), a.num_guys),
                // Item 542: the launch height, the aim height, the
                // bearing, the arc and the roll flag, the rest of the
                // record this crate carries. `traj`, `dx`, the roll and
                // bank angles, `gpiece` and `graph_index` it does not.
                ("sz", i64::from(p.sz), a.sz),
                ("ez", i64::from(p.ez), a.ez),
                ("angle", i64::from(p.angle.0), a.angle),
                ("v1z", super::ammo::tests::printed(p.v1z), a.v1z),
                // Item 602: flag 4 is `flags & 4`, and `rolling` is
                // `AmmoData +0x5`, a byte `Ammo::init` zeroes on every
                // round (see `ammo_flag_rows`).
            ]
            .into_iter()
            .chain(ammo_flag_rows(p, a))
            {
                if mine != dumped {
                    firsts
                        .entry((who, o, format!("ammo[{slot}].{name}")))
                        .or_insert((n, format!("ours {mine} theirs {dumped}")));
                }
            }
        }
        // **Both sides printed once on the word's two blocks**, for the
        // three hulls: the record the draw is spent in, before any quiet
        // row is trusted.
        if (MOVED_FROM..=MOVED_FROM + 1).contains(&n) {
            for them in frame.units.iter().filter(|u| ship(u.who, u.o)) {
                let mine = u8::try_from(them.who)
                    .ok()
                    .zip(i16::try_from(them.o).ok())
                    .and_then(|(w, o)| built.sim.unit_by_o(w, o))
                    .map(|u| &built.sim.units[u]);
                eprintln!("  block {n} {}/{} theirs {them:?}", them.who, them.o);
                match mine {
                    Some(u) => eprintln!(
                        "  block {n} {}/{} ours pos {:?} heading {:?} facing {:?} \
                         orders {:?} guys {:?}",
                        them.who,
                        them.o,
                        u.pos,
                        u.movement.heading,
                        u.movement.facing,
                        u.orders,
                        u.guys
                    ),
                    None => eprintln!("  block {n} {}/{} ours: absent", them.who, them.o),
                }
            }
        }
    }
    let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    for ((w, o, what), (f, row)) in &firsts {
        by_block
            .entry(*f)
            .or_default()
            .push(format!("{w}/{o} {what}: {row}"));
    }
    for (f, rows) in &by_block {
        let near = (MOVED_FROM - 4..=GOLDEN_WORD_CHAPTER_FIVE + 2).contains(f);
        if near || rows.len() <= 4 {
            for r in rows {
                eprintln!("  f{f} {r}");
            }
        } else {
            eprintln!("  f{f}: {} keys", rows.len());
        }
    }
    eprintln!(
        "ch5 widening: {blocks} blocks [{FIRST}, {LAST}), {rows} record rows, \
         {leader_rows} leader rows, ammo {ammo_theirs} theirs / {ammo_ours} ours, \
         {} keys parted",
        firsts.len()
    );
    // `LEADERS=2` prints the goods block and nothing after it, so the
    // leader rows this capture can compare are the goods' own. The keys it
    // does not print are absent rather than divergent, and what is pinned
    // is that the goods rows were actually taken on every block.
    eprintln!(
        "ch5: {} leader keys not printed at LEADERS=2",
        missing.len()
    );
    pin_eq!(
        leader_rows,
        2 * LEADERS_TWO_KEYS * blocks,
        "the leader rows LEADERS=2 prints ({LEADERS_TWO_KEYS} a player) are not compared on \
         every block"
    );
    // **Anti-vacuity for the `AMMO` record**: run127's 506 live rounds
    // are all read, and this crate's side is not empty.
    pin_eq!(ammo_theirs, 506, "run127's live rounds are not all read");
    pin!(ammo_ours > 0, "this crate fired no round in the window");
    // **The map's shape.** Three standing families from the capture's
    // first block, none of them the water's: `build:extra` (the end
    // detail prints no `BUILDDATA`), the unmodelled `form`, and two
    // `filled_gather_slots`. `form` is also every hull's birth row.
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let at_floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == FIRST)
        .map(|((_, _, what), _)| what)
        .collect();
    // Item 1605 took the human's `filled_gather_slots` (and city) rows: the
    // human leader takes the sweep (`docs/AI.md` §169).
    pin!(
        at_floor.iter().all(|w| standing(w)) && at_floor.len() == 24,
        "the standing rows on run127's first block moved: {at_floor:?}"
    );
    // **Nothing parts under the word's own block.** The trace's frame
    // `f` writes block `f + 1`. Item 535's broadside (`docs/COMBAT.md`
    // §49) closed the four `1/6` angle rows on block 617, item 542's
    // release frame and keel nodes (§50) the first round on 622, and item
    // 543's human rare-collector arm (`docs/ORDERS.md` §23) the fisher
    // `0/7`'s: its birth `[MOVE_TO, CAST 0x292]` on 621, the walk from
    // 622, and the deploy on 665 with `packed` and `mylos` 4 → 6. Made to
    // fail once with the arm removed: the word fell back to 664, all
    // fifteen of the fisher's rows came back, and so did its leader's
    // food and wealth rows from 673, the deployed boat's pay.
    //
    // **Item 549 took the word to the capture's end** (`docs/ANIM.md`
    // §4.13): a rolled `CHAR_ATTACK3` the Trireme's packet does not name
    // plays `CHAR_ATTACK2`, forty frames, not three. With the fallback
    // removed the word falls back to 739 and the map holds 224 keys, the
    // first `1/6 ammo[0]` on 742; with it, 30.
    // ~~One row stood under the word and it was not the water's: the
    // explore order of who=1's scout `1/0` carried the formation mirror
    // `facing` 1 against the dump's 0 from 847 (parked 275).~~ Nothing
    // stands since `push_group` keeps an equal group's record (item 723): the scout's
    // repeated pushes lay out from the last layout's slot bytes.
    let under: Vec<String> = firsts
        .iter()
        .filter(|((_, _, what), (f, _))| *f <= GOLDEN_WORD_CHAPTER_FIVE && !standing(what))
        .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
        .collect();
    pin_eq!(
        under,
        Vec::<String>::new(),
        "what parts at or under the word moved"
    );
    // **Every round of the capture agrees whole** (items 542 and 549):
    // all 506 live rounds of both ships, on every `AMMO` field this crate
    // carries, `v1z` to the last printed digit. Before 549 the first shot
    // row was `1/6 ammo[0]` on 742, the round the dump held alone.
    let first_ammo = firsts
        .iter()
        .filter(|((_, _, what), _)| what.starts_with("ammo["))
        .map(|(_, (f, _))| *f)
        .min();
    pin_eq!(
        first_ammo,
        None,
        "a round of the capture parts: first ammo row on {first_ammo:?}"
    );
    pin_eq!(ammo_ours, 506, "this crate's rounds are not run127's 506");
    let absent = usize::from((FIRST..LAST).contains(&RUN127_NO_BLOCK));
    pin_eq!(
        blocks,
        (LAST - FIRST) as usize - absent,
        "run127's dump no longer carries every frame of [{FIRST}, {LAST})"
    );
}

pub(super) fn chapter_four_s_two_captures_are_one_game() {
    let (Some((_, a)), Some((_, b))) = (golden("ch4b"), golden("ch4u")) else {
        eprintln!("skipping: chapter four needs both run132 and run133 (docs/RUNS.md)");
        return;
    };
    let read = |p: &str| {
        crate::trace::Trace::read(std::path::Path::new(p))
            .expect("a finalized golden trace")
            .expect("missing RONT header")
    };
    let (ta, tb) = (read(&a), read(&b));
    let last = |t: &crate::trace::Trace| t.frames.last().map_or(0, |(f, _)| *f);
    assert!(
        last(&ta) >= 1500 && last(&tb) >= 1500,
        "chapter four's traces end at {} and {}; both runs go to 1500",
        last(&ta),
        last(&tb)
    );
    let mut draws = 0usize;
    let mut parted = Vec::new();
    for f in 0..=1500 {
        let (la, lb) = (ta.labels(f), tb.labels(f));
        draws += la.len();
        if la != lb {
            parted.push((f, la.len(), lb.len()));
        }
    }
    eprintln!("chapter four: {draws} draws over 1501 frames in each trace");
    assert!(draws > 0, "chapter four's traces carry no draw");
    assert_eq!(
        parted,
        vec![],
        "run132's and run133's draw streams part: (frame, run132, run133)"
    );
}

pub(super) fn chapter_four_s_border_is_widened_cell_for_cell() {
    let _pins = Pins::hold();
    use std::collections::BTreeMap;
    const FIRST: i64 = WIDENING_CHAPTER_FOUR_BORDER.0;
    const LAST: i64 = WIDENING_CHAPTER_FOUR_BORDER.1;
    let Some(mut s) = stage_chapter_four("ch4b") else {
        return;
    };
    let players = s.built.sim.players.len();
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let (mut blocks, mut cells_compared) = (0usize, 0usize);
    let mut counts: Vec<(i64, [i64; 2], [i64; 2], usize)> = Vec::new();
    let mut applied = crate::golden::Applied::default();
    for f in 0..LAST - 1 {
        let did = s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        applied.merge(&did);
        s.built.tick();
        let n = f + 1;
        if n < FIRST {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = s.ix.frame_state(at).unwrap();
        blocks += 1;
        let mut unit_rows: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
        crate::diff::harness::widen_block(&s.built, &frame, players, n, &mut unit_rows);
        for (k, v) in unit_rows {
            // No `UNITDATA` in this window: a unit is "extra" on every
            // block and says nothing. Buildings are `o` 2000 and up.
            if k.2 == "extra" && k.1 < 2000 {
                continue;
            }
            firsts.entry(k).or_insert(v);
        }
        let raw = s.ix.read_frame(at).unwrap();
        let parsed = Log::parse(&raw);
        let Some((_, block)) = parsed.frames().into_iter().find(|(k, _)| *k == n) else {
            continue;
        };
        let Some(world) = block.kid("WORLD") else {
            continue;
        };
        let fields = world.fields().to_vec();
        let theirs = crate::gamelog::world_cells(&fields);
        pin_eq!(
            theirs.len(),
            3_600,
            "run132's block {n} is not a whole WORLD scan"
        );
        let w = &s.built.sim.world;
        let code = |o: sim::world::Owner| match o {
            sim::world::Owner::Player(p) => i64::from(p),
            sim::world::Owner::Ambiguous => -2,
            sim::world::Owner::None => -1,
        };
        let (mut ours_n, mut theirs_n) = ([0i64; 2], [0i64; 2]);
        let mut owners_part = 0usize;
        for y in 0..w.height() {
            for x in 0..w.width() {
                let c = sim::world::Cell::new(x, y);
                let i = (y * w.width() + x) as usize;
                let t = &theirs[i];
                let d = w.cell_data(c);
                let (who, who2) = (code(w.owner(c)), code(w.second(c)));
                for p in 0..2 {
                    ours_n[p] += i64::from(who == p as i64);
                    theirs_n[p] += i64::from(t.who == p as i64);
                }
                cells_compared += 1;
                owners_part += usize::from(who != t.who);
                for (name, mine, dumped) in [
                    ("who", who, t.who),
                    ("who2", who2, t.who2),
                    ("flags", i64::from(d.flags), t.flags),
                    ("blocked", i64::from(d.blocked), t.blocked),
                    ("solid", i64::from(d.solid), t.solid),
                    ("bad", i64::from(d.bad), t.bad),
                ] {
                    if mine != dumped {
                        firsts
                            .entry((-3, i as i64, format!("cell({x},{y}).{name}")))
                            .or_insert((n, format!("ours {mine} theirs {dumped}")));
                    }
                }
            }
        }
        if counts
            .last()
            .is_none_or(|l| (l.1, l.2, l.3) != (ours_n, theirs_n, owners_part))
        {
            counts.push((n, ours_n, theirs_n, owners_part));
        }
    }
    eprintln!(
        "chapter four staged: {} line(s) ran, {} unit(s), {} building(s)",
        applied.ran, applied.units, applied.buildings
    );
    for ((word, why), k) in &applied.skipped {
        eprintln!("  {k:5} {word}: {why}");
    }
    for (n, o, t, k) in &counts {
        eprintln!("  block {n}: owner 0/1 cells ours {o:?} theirs {t:?}, {k} owners part");
    }
    let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    for ((w, o, what), (f, row)) in &firsts {
        by_block
            .entry(*f)
            .or_default()
            .push(format!("{w}/{o} {what}: {row}"));
    }
    for (f, rows) in &by_block {
        if rows.len() <= 12 {
            for r in rows {
                eprintln!("  f{f} {r}");
            }
        } else {
            eprintln!("  f{f}: {} keys, first {}", rows.len(), rows[0]);
        }
    }
    eprintln!(
        "ch4 border widening: {blocks} blocks [{FIRST}, {LAST}), {cells_compared} cells, \
         {} keys parted",
        firsts.len()
    );
    pin_eq!(
        blocks,
        (LAST - FIRST) as usize,
        "run132's dump no longer carries every frame of [{FIRST}, {LAST})"
    );
    // **The three levers, each settled cell for cell** (item 552). The
    // original's owner-0 count is 266, then 296 from block 310 (the
    // Temple), 327 from 411 (Religion, temple level 2) and 445 from 511
    // (Civic 3). The count is §8's falsifier and it did not fire on any
    // lever; this crate reaches each figure on every cell once the sweep
    // is done. Each of the item's three fixes was found by this test's
    // counts before it had assertions: the interpreter's old
    // `place_building` arm left an unstarted site (266 throughout), the
    // temple level was a constant 1 (327 never reached), and
    // `set_leader_epoch` skipped `gain_tech`'s tail (445 never reached).
    // Made to fail once with the last reverted: the settled list stops at
    // 411.
    let settled: Vec<(i64, i64)> = counts
        .iter()
        .filter(|c| c.3 == 0)
        .map(|c| (c.0, c.2[0]))
        .collect();
    // **Since item 1106 the sweep's own blocks settle too** (`docs/AI.md`
    // §84): the border pass writes the fix's owners at 256 cells a frame,
    // as the original's does, so the blocks where the original's count is
    // mid-sweep (276, 282, 285 after the Temple; 308 … 319 after Religion;
    // 346 … 406 after Civic 3) agree cell for cell as well.
    pin_eq!(
        settled,
        vec![
            (295, 266),
            (306, 276),
            (307, 282),
            (308, 285),
            (310, 296),
            (406, 308),
            (407, 314),
            (408, 316),
            (410, 319),
            (411, 327),
            (505, 346),
            (506, 366),
            (507, 384),
            (508, 397),
            (509, 403),
            (510, 406),
            (511, 445)
        ],
        "the blocks on which every owner agrees, with the original's count"
    );
    pin!(
        counts.iter().all(|c| c.2[1] == 261 && c.1[1] == 261),
        "player 1's 261 cells moved on one side"
    );
    // **What parts is the sweep, and only the sweep.** The original spreads
    // each lever over the blocks `GameDaemon::check_borders` takes at 256
    // cells a frame; this crate recomputes wholesale on the line's own
    // frame (`docs/ATTRITION.md`, "Territory"). So a `who` row may part
    // inside the three windows and nowhere else, and every one has closed
    // by the next settled block above.
    let sweeping =
        |f: i64| (301..310).contains(&f) || (401..411).contains(&f) || (501..511).contains(&f);
    let cell_rows: Vec<String> = firsts
        .iter()
        .filter(|((w, _, what), (f, _))| *w == -3 && !(what.ends_with(".who") && sweeping(*f)))
        .map(|((_, _, what), (f, row))| format!("{f} {what}: {row}"))
        .collect();
    pin_eq!(
        cell_rows,
        Vec::<String>::new(),
        "a cell parts outside the sweep: the Temple's footprint, `who2`, or \
         a steady-state owner"
    );
    // The records beside the cells: the 14 standing rows of run132's first
    // block — `0/2000`'s AI half of Napata's record, which `ai off` leaves
    // this crate holding at zero, and London's `filled`/`land` one apart —
    // and nothing else. The Temple's `BUILDDATA` and Napata's temple bit
    // (`city_flags[0x80]`, 0 → 1 on block 301) agree on every block.
    let other: Vec<String> = firsts
        .iter()
        .filter(|((w, _, _), (f, _))| *w != -3 && *f != FIRST)
        .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
        .collect();
    pin_eq!(other, Vec::<String>::new(), "a building or city row parts");
    // ~~**run132's first parting is 301**, the Temple's frame, and it is the
    // sweep's: 30 cells this crate owns a sweep early.~~ **Since item 1106
    // nothing parts past the first block**: the border pass writes the
    // sweep at the original's 256 cells a frame (`docs/AI.md` §84). The
    // chapter's word, 1277, is in run133's window; this is the border
    // window's own.
    let first_parting = firsts
        .iter()
        .filter(|(_, (f, _))| *f != FIRST)
        .map(|(_, (f, _))| *f)
        .min();
    pin_eq!(first_parting, None, "run132's first parting moved");
    pin_eq!(
        firsts.iter().filter(|(_, (f, _))| *f == FIRST).count(),
        0,
        "the standing rows on run132's first block moved"
    );
}

pub(super) fn chapter_four_holds_to_the_golden_word() {
    let Some(w) = walk_chapter("ch4u", 4, 8, 1500) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FOUR,
        "chapter four's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FOUR}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FOUR,
        "chapter four's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §8"
    );
    eprintln!(
        "chapter four: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

pub(super) fn chapter_four_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    use std::collections::BTreeMap;
    const FIRST: i64 = WIDENING_CHAPTER_FOUR.0;
    const LAST: i64 = WIDENING_CHAPTER_FOUR.1;
    let Some(mut s) = stage_chapter_four("ch4u") else {
        return;
    };
    let players = s.built.sim.players.len();
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let mut missing = std::collections::BTreeSet::new();
    let (mut blocks, mut rows, mut leader_rows) = (0usize, 0usize, 0usize);
    let mut last_seen: BTreeMap<(i64, i64), String> = BTreeMap::new();
    for f in 0..LAST - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < FIRST {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = s.ix.frame_state(at).unwrap();
        blocks += 1;
        let (_, k) = crate::diff::harness::widen_block(&s.built, &frame, players, n, &mut firsts);
        rows += k;
        rows += widen_turrets(&s.built, &frame, players, n, &mut firsts);
        let raw = s.ix.read_frame(at).unwrap();
        let flog = Log::parse(&raw);
        for who in 0..2usize {
            let Some(block) = flog.leader_block(n, who as i64) else {
                continue;
            };
            let t = crate::diff::leader::theirs(&block);
            for (k, v) in crate::diff::leader::rows(&s.loaded, &s.built, who) {
                let Some(&y) = t.get(&k) else {
                    missing.insert(k);
                    continue;
                };
                leader_rows += 1;
                if v != y {
                    firsts
                        .entry((who as i64, -1, format!("leader:{k}")))
                        .or_insert((n, format!("ours {v} theirs {y}")));
                }
            }
        }
        // **Both sides printed once on the word's two blocks** (DECISIONS
        // 43), for player 1's staged units: the record the draw is spent
        // in, before any quiet row is trusted.
        if (GOLDEN_WORD_CHAPTER_FOUR..=GOLDEN_WORD_CHAPTER_FOUR + 1).contains(&n)
            || (1416..=1417).contains(&n)
        {
            for them in frame.units.iter().filter(|u| u.who == 1 && u.o >= 6) {
                let mine = u8::try_from(them.who)
                    .ok()
                    .zip(i16::try_from(them.o).ok())
                    .and_then(|(w, o)| s.built.sim.unit_by_o(w, o))
                    .map(|u| &s.built.sim.units[u]);
                eprintln!("  block {n} 1/{} theirs {them:?}", them.o);
                match mine {
                    Some(u) => eprintln!(
                        "  block {n} 1/{} ours pos {:?} heading {:?} orders {:?} guys {:?}",
                        them.o, u.pos, u.movement.heading, u.orders, u.guys
                    ),
                    None => eprintln!("  block {n} 1/{} ours: absent", them.o),
                }
            }
        }
        // **The wagon's first route, on the block it is planned** (item
        // 569): the first parting of `1/10` in run133, and the frame this
        // chapter's word 1416 traces back to. The Supply Wagon is born on
        // 1101 and sent after its army's first member (`docs/ARMY.md`
        // §4.3), and its `find_wpath` from cell (6, 44) to (13, 37) is the
        // first of a supply unit anywhere in the corpus. The original's
        // route, in world cells, top first: one step **south-east** to
        // (7, 45), then the diagonal, then north along x = 12 — every cell
        // of it clear of the `0x200` flag, which an army pays `base << 5`
        // for (`docs/PATHFINDER.md` §5). Printed both sides by cell.
        //
        // **And this crate's is the same route since item 569**: the
        // wagon's `find_wpath` takes the army mode through `is_supply`'s
        // arm (`docs/PATHFINDER.md` §25). Without that arm it went (7, 43)
        // north-east through the `0x200` cells, one leg shorter, and
        // walked apart from the original's from 1102 on.
        if n == 1101 {
            let cells = |v: Vec<(i64, i64)>| -> Vec<(i64, i64)> {
                v.into_iter()
                    .rev()
                    .map(|(x, y)| (x / 768, y / 768))
                    .collect()
            };
            let theirs = cells(
                frame
                    .units
                    .iter()
                    .find(|u| u.who == 1 && u.o == 10)
                    .expect("the original's wagon is born on 1101")
                    .path
                    .iter()
                    .map(|p| p.to)
                    .collect(),
            );
            let u = s
                .built
                .sim
                .unit_by_o(1, 10)
                .expect("this crate's wagon on 1101");
            let ours = cells(
                s.built.sim.units[u]
                    .path
                    .iter()
                    .map(|p| (i64::from(p.to.x), i64::from(p.to.y)))
                    .collect(),
            );
            eprintln!("  route 1101 1/10 theirs {theirs:?}");
            eprintln!("  route 1101 1/10 ours   {ours:?}");
            pin_eq!(
                theirs,
                vec![
                    (7, 45),
                    (8, 44),
                    (9, 43),
                    (10, 42),
                    (11, 41),
                    (12, 40),
                    (12, 39),
                    (12, 38),
                    (13, 37)
                ],
                "run133's wagon route on 1101"
            );
            pin_eq!(ours, theirs, "this crate's wagon route on 1101");
        }
        // **The escort's own row on the block it is issued** (item 567,
        // `docs/ORDERS.md` §24), read off both sides by name rather than
        // through `compare`'s labels: each hoplite's `GUARDORDER` is on the
        // wagon `1/10`, and its offset and both counters are the
        // original's — (0, 264), (144, 264), (−144, 264), `idle` and
        // `retry` 0. The post, which is the wagon's position plus the
        // offset, differed until item 569 walked the wagon on the
        // original's line (`docs/PATHFINDER.md` §25); it agrees now.
        if n == 1277 {
            for (o, dx) in [(6, 0), (7, 144), (8, -144)] {
                let them = frame
                    .units
                    .iter()
                    .find(|u| u.who == 1 && u.o == o)
                    .and_then(|u| u.orders.iter().find(|x| x.index == 12))
                    .expect("the original's escort holds a GUARDORDER on 1277");
                let theirs = (
                    them.ox,
                    them.guard_dx,
                    them.guard_dy,
                    them.guard_idle,
                    them.guard_retry,
                );
                pin_eq!(theirs, (Some(10), Some(dx), Some(264), Some(0), Some(0)));
                // **And the post itself since item 569**: the wagon walks
                // the original's line, so the wagon's position plus the
                // offset is the original's point on all three figures.
                let post = them.guard_x.zip(them.guard_y);
                let u = s.built.sim.unit_by_o(1, i16::try_from(o).unwrap()).unwrap();
                let g = s.built.sim.units[u]
                    .orders
                    .iter()
                    .find_map(|x| match x.body {
                        sim::orders::Body::Guard(g) => Some(g),
                        _ => None,
                    })
                    .expect("this crate's escort holds a GUARD on 1277");
                let ours = (
                    s.built.unit_ids(g.target).map(|(_, o)| o),
                    Some(i64::from(g.dx)),
                    Some(i64::from(g.dy)),
                    Some(i64::from(g.idle)),
                    Some(i64::from(g.retry)),
                );
                pin_eq!(ours, theirs, "1/{o}'s guard row");
                pin_eq!(
                    Some((i64::from(g.guard.x), i64::from(g.guard.y))),
                    post,
                    "1/{o}'s post on 1277"
                );
            }
        }
        // The bleed's own timeline: player 1's staged units, `o` 6 and up.
        let w = &s.built.sim.world;
        for them in frame.units.iter().filter(|u| u.who == 1 && u.o >= 6) {
            let mine = u8::try_from(them.who)
                .ok()
                .zip(i16::try_from(them.o).ok())
                .and_then(|(w, o)| s.built.sim.unit_by_o(w, o))
                .map(|u| &s.built.sim.units[u]);
            let cell = |x: i64, y: i64| {
                let c = sim::Pos::new(x as i32, y as i32).cell();
                w.owner(c).player().map_or(-1, i64::from)
            };
            let theirs_row = format!(
                "attr {:?} dmg {:?}+{:?}/16 shelter {:?} on {}",
                them.attrition,
                them.damage,
                them.damage_frac,
                them.unit_masks2.map(|m| i64::from(m & 0x4_0000 != 0)),
                cell(them.pos.x, them.pos.y)
            );
            let ours_row = mine.map_or("absent".to_string(), |u| {
                format!(
                    "attr {} hp {}/{} +{}/16 shelter {} on {}",
                    u.attrition,
                    u.health,
                    u.max_health,
                    u.damage_frac,
                    i64::from(u.sheltered),
                    w.owner_at(u.pos).player().map_or(-1, i64::from)
                )
            });
            let row = format!("theirs {theirs_row} | ours {ours_row}");
            if last_seen.get(&(them.who, them.o)) != Some(&row) {
                eprintln!("  bleed {n} 1/{} {row}", them.o);
                last_seen.insert((them.who, them.o), row);
            }
        }
    }
    let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    for ((w, o, what), (f, row)) in &firsts {
        by_block
            .entry(*f)
            .or_default()
            .push(format!("{w}/{o} {what}: {row}"));
    }
    for (f, rows) in &by_block {
        if rows.len() <= 12 || (GOLDEN_WORD_CHAPTER_FOUR..=GOLDEN_WORD_CHAPTER_FOUR + 1).contains(f)
        {
            for r in rows {
                eprintln!("  f{f} {r}");
            }
        } else {
            eprintln!("  f{f}: {} keys, first {}", rows.len(), rows[0]);
        }
    }
    eprintln!(
        "ch4 widening: {blocks} blocks [{FIRST}, {LAST}), {rows} record rows, \
         {leader_rows} leader rows, {} keys parted; {} leader keys not printed \
         at LEADERS=2",
        firsts.len(),
        missing.len()
    );
    pin_eq!(
        blocks,
        (LAST - FIRST) as usize - 1,
        "run133's dump no longer carries every frame of [{FIRST}, {LAST}) \
         but 1500, the block its `!quit` replaces"
    );
    pin_eq!(
        leader_rows,
        2 * LEADERS_TWO_KEYS * blocks,
        "the leader rows LEADERS=2 prints ({LEADERS_TWO_KEYS} a player) are not compared on \
         every block"
    );
    // **The standing rows of run133's first block**, chapter five's three
    // families and nothing of this chapter's: `build:extra` (the end
    // detail prints no `BUILDDATA`), the unmodelled `form`, and two
    // `filled_gather_slots`.
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let at_floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == FIRST)
        .map(|((_, _, what), _)| what)
        .collect();
    // Item 1605 took the human's `filled_gather_slots` (and city) rows: the
    // human leader takes the sweep (`docs/AI.md` §169).
    pin!(
        at_floor.iter().all(|w| standing(w)) && at_floor.len() == 25,
        "the standing rows on run133's first block moved: {at_floor:?}"
    );
    // **The bleed agrees, tick for tick, on every block of run133** (item
    // 552, item 567). Every hoplite figure's period, every 6/16 and every
    // whole point of `damage` from block 601 to 1500, the scout's and the
    // wagon's zero, and the supply mark. Item 552 left the first row of
    // the namesake's record on 1338, `1/7 attrition: ours 0 theirs 48`,
    // after this crate had walked `1/7` onto unowned ground; with the
    // escort built (item 567) the squad stays beside its wagon as the
    // original's does, and the row is gone. It took two fixes: `Leader::calc_attrition` wired to the tech
    // tree (the period was 0 on every figure, `docs/ATTRITION.md`
    // "Strength") and `curr_uber_size` counted rather than stored (each
    // tick took 16/16, not 6/16). Made to fail once with the second
    // reverted: `1/8 damage_frac` parts on 617.
    let bleed = |what: &str| {
        matches!(
            what,
            "attrition" | "sheltered" | "damage_frac" | "hits_left" | "myhits"
        ) || what.starts_with("hits:")
    };
    let first_bleed = firsts
        .iter()
        .filter(|((_, _, what), _)| bleed(what))
        .map(|((w, o, what), (f, row))| (*f, format!("{w}/{o} {what}: {row}")))
        .min();
    pin_eq!(
        first_bleed,
        None,
        "the namesake's record parts somewhere new"
    );
    // **What parts under the word**, none of it a draw, and the word is
    // the capture's end: ~~the scout's explore-order `facing` (parked 275),
    // gone since `push_group` keeps an equal group's record (item 723)~~; the squad's group id (`1020001` against `1026401`, a
    // numbering, on 1021); and the scout's second figure three units off
    // on 1172. Pinned by key and block so none of them can stand in for
    // anything else.
    //
    // **Item 569 took eighty-nine rows off this list.** Twenty were the
    // wagon's first route on 1101, one leg short, and its walk from 1102.
    // The other sixty-nine were the escort's posts from 1277: the post is
    // the wagon's position plus the slot offset (`docs/ORDERS.md` §24.6),
    // so it could not agree until the wagon's walk did. Both were
    // `find_wpath`'s missing `is_supply` arm (`docs/PATHFINDER.md` §25).
    let under: Vec<String> = firsts
        .iter()
        .filter(|((_, _, what), (f, _))| {
            *f > FIRST && *f < GOLDEN_WORD_CHAPTER_FOUR && !standing(what)
        })
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    // Item 1461: `1/9`'s second figure's 1172/1173 rows left; it steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed"). Item 1863's pool id took the last row.
    let mut want: Vec<String> = Vec::new();
    let mut under_sorted = under.clone();
    under_sorted.sort();
    want.sort();
    pin_eq!(under_sorted, want, "what parts under the word moved");
    // **The move's block, 1416** (item 569; its delta is in
    // `GOLDEN_WORD_CHAPTER_FOUR`'s comment). Item 567 left two rows here:
    // the wagon's `pause`, 15 in the original and 0 here, and `1/8`'s half
    // step. The pause is `do_attack_to_pause`'s (`docs/ORDERS.md` §24.9):
    // on (1415 + 10) % 15 == 0 the wagon has the escort's captain within
    // `0x600`, and it waits. Now every record agrees on 1416 and on every
    // block above the scout's 1173, to the end of the capture.
    let above: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > 1173)
        .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
        .collect();
    pin_eq!(
        above,
        Vec::<String>::new(),
        "a row parts above 1173, where run133 agrees to its end"
    );
}

pub(super) fn chapter_seven_s_pair_is_one_game_until_the_gate() {
    let (Some((_, a)), Some((_, b))) = (golden("ch7"), golden("ch7c")) else {
        eprintln!("skipping: chapter seven needs both run141 and run142 (docs/RUNS.md)");
        return;
    };
    let read = |p: &str| {
        crate::trace::Trace::read(std::path::Path::new(p))
            .expect("a finalized golden trace")
            .expect("missing RONT header")
    };
    let (ta, tb) = (read(&a), read(&b));
    let last = |t: &crate::trace::Trace| t.frames.last().map_or(0, |(f, _)| *f);
    assert!(
        last(&ta) >= 1200 && last(&tb) >= 1200,
        "chapter seven's traces end at {} and {}; both runs go to 1200",
        last(&ta),
        last(&tb)
    );
    let parted = (0..=1200).find(|&f| ta.labels(f) != tb.labels(f));
    let agreed: usize = (0..parted.unwrap_or(1201))
        .map(|f| ta.labels(f).len())
        .sum();
    let (da, db): (usize, usize) = (0..=1200)
        .map(|f| (ta.labels(f).len(), tb.labels(f).len()))
        .fold((0, 0), |(x, y), (p, q)| (x + p, y + q));
    eprintln!(
        "chapter seven: run141 {da} draws, run142 {db}; {agreed} identical \
         before the parting at {parted:?}"
    );
    if let Some(f) = parted {
        eprintln!(
            "  frame {f}: run141 {} draws, run142 {}",
            ta.labels(f).len(),
            tb.labels(f).len()
        );
    }
    assert_eq!(
        parted,
        Some(1),
        "the pair parts where `ai off` is first read"
    );
    assert_eq!(agreed, 120, "frame 0's draws, identical in the pair");
}

pub(super) fn chapter_seven_s_civilians_act_alike_with_the_ai_off_and_on() {
    use std::collections::BTreeMap;
    let (Some((a, _)), Some((b, _))) = (golden("ch7"), golden("ch7c")) else {
        eprintln!("skipping: chapter seven needs both run141 and run142 (docs/RUNS.md)");
        return;
    };
    // (block, o) → the order list's kinds, newest first as the log writes
    // them, and `idle`, for player 0's `o` 6..=10.
    type Kinds = BTreeMap<(i64, i64), (Vec<String>, Option<i64>)>;
    let kinds = |p: &str| -> Kinds {
        let mut ix = crate::capture::indexed::IndexedCapture::open(p).unwrap();
        let mut out = Kinds::new();
        for at in 0..ix.frames().len() {
            let frame = ix.frame_state(at).unwrap();
            for u in frame
                .units
                .iter()
                .filter(|u| u.who == 0 && (6..=10).contains(&u.o))
            {
                out.insert(
                    (frame.n, u.o),
                    (u.orders.iter().map(|o| o.kind.clone()).collect(), u.idle),
                );
            }
        }
        out
    };
    let (ka, kb) = (kinds(&a), kinds(&b));
    let born = |k: &Kinds, o: i64| k.keys().find(|(_, x)| *x == o).map(|(n, _)| *n);
    for (o, n) in [(6, 611), (7, 616), (8, 621), (9, 626), (10, 631)] {
        assert_eq!(born(&ka, o), Some(n), "run141's 0/{o} is born on {n}");
        assert_eq!(born(&kb, o), Some(n), "run142's 0/{o} is born on {n}");
    }
    let only = |k: &Kinds| -> Vec<(i64, i64, String)> {
        k.iter()
            .flat_map(|(&(n, o), (kinds, _))| kinds.iter().map(move |k| (n, o, k.clone())))
            .collect()
    };
    let (oa, ob) = (only(&ka), only(&kb));
    let first = |v: &[(i64, i64, String)]| v.first().cloned();
    eprintln!(
        "chapter seven: {} ordered unit-blocks in run141, {} in run142; first {:?}",
        oa.len(),
        ob.len(),
        first(&oa)
    );
    // Only the citizen ever holds an order: its gather, and the gather's
    // own transit legs (a `MOVEORDER` pushed over it, 764 onward).
    let gathering = |k: &str| k == "GATHERORDER" || k == "MOVEORDER";
    for (run, v) in [("run141", &oa), ("run142", &ob)] {
        assert!(
            v.iter().all(|(_, o, k)| *o == 6 && gathering(k)),
            "{run}: an order on a civilian other than the citizen's gather: {:?}",
            v.iter().find(|(_, o, k)| *o != 6 || !gathering(k))
        );
        assert_eq!(
            first(v).map(|(n, o, _)| (n, o)),
            Some((763, 6)),
            "{run}: the citizen's first order"
        );
    }
    assert_eq!(
        ka.get(&(763, 6)).and_then(|x| x.1),
        Some(12),
        "the human's idle wait"
    );
    assert_eq!(
        ka, kb,
        "the five's orders differ between the AI-off chapter and its control"
    );
}

pub(super) fn chapter_seven_holds_to_the_golden_word() {
    let Some(w) = walk_chapter("ch7", 7, 7, 1200) else {
        return;
    };
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SEVEN,
        "chapter seven's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §11"
    );
    assert_eq!(
        (w.sequence, w.value),
        (GOLDEN_WORD_CHAPTER_SEVEN, None),
        "chapter seven's label order or value word parts under its end"
    );
}

pub(super) fn chapter_seven_s_control_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch7c", "chapter7_control", 7, 6, 1200) else {
        return;
    };
    assert_eq!(
        (w.word, w.sequence, w.value),
        (GOLDEN_WORD_CHAPTER_SEVEN, GOLDEN_WORD_CHAPTER_SEVEN, None),
        "chapter seven's control parts under its end"
    );
}

pub(super) fn chapter_seven_b_s_pair_is_one_game_until_the_gate() {
    let (Some((_, a)), Some((_, b))) = (golden("ch7b"), golden("ch7bc")) else {
        eprintln!("skipping: chapter seven-b needs both run156 and run157 (docs/RUNS.md)");
        return;
    };
    let read = |p: &str| {
        crate::trace::Trace::read(std::path::Path::new(p))
            .expect("a finalized golden trace")
            .expect("missing RONT header")
    };
    let (ta, tb) = (read(&a), read(&b));
    let parted = (0..=1200).find(|&f| ta.labels(f) != tb.labels(f));
    let agreed: usize = (0..parted.unwrap_or(1201))
        .map(|f| ta.labels(f).len())
        .sum();
    assert_eq!(
        parted,
        Some(1),
        "the pair parts where `ai off` is first read"
    );
    assert_eq!(agreed, 120, "frame 0's draws, identical in the pair");
    eprintln!(
        "chapter seven-b: frame 1 is {} draws in run156 and {} in run157",
        ta.labels(1).len(),
        tb.labels(1).len()
    );
}

pub(super) fn chapter_seven_b_s_civilians_act_alike_with_the_ai_off_and_on() {
    use std::collections::BTreeMap;
    let (Some((a, _)), Some((b, _))) = (golden("ch7b"), golden("ch7bc")) else {
        eprintln!("skipping: chapter seven-b needs both run156 and run157 (docs/RUNS.md)");
        return;
    };
    // birth block → (guy type, masks at birth, first block of each order kind)
    type Five = BTreeMap<i64, (Option<i64>, i64, BTreeMap<String, i64>)>;
    let five = |p: &str| -> Five {
        let mut ix = crate::capture::indexed::IndexedCapture::open(p).unwrap();
        let mut born: BTreeMap<i64, i64> = BTreeMap::new();
        let mut out = Five::new();
        for at in 0..ix.frames().len() {
            let frame = ix.frame_state(at).unwrap();
            if frame.n < 605 {
                continue;
            }
            for u in frame.units.iter().filter(|u| u.who == 1) {
                let birth = *born.entry(u.o).or_insert(frame.n);
                if ![611, 616, 621, 626, 631].contains(&birth) {
                    continue;
                }
                let e = out.entry(birth).or_insert_with(|| {
                    (
                        u.guys.first().and_then(|g| g.kind),
                        u.unit_masks.unwrap_or(0),
                        BTreeMap::new(),
                    )
                });
                for o in &u.orders {
                    e.2.entry(o.kind.clone()).or_insert(frame.n);
                }
            }
        }
        out
    };
    let (fa, fb) = (five(&a), five(&b));
    for (run, f) in [("run156", &fa), ("run157", &fb)] {
        let types: Vec<(i64, Option<i64>)> = f.iter().map(|(n, e)| (*n, e.0)).collect();
        assert_eq!(
            types,
            [
                (611, Some(50)),
                (616, Some(59)),
                (621, Some(61)),
                (626, Some(52)),
                (631, Some(400))
            ],
            "{run}: the five's births and types"
        );
        assert!(
            f.values().all(|e| e.1 & 0x40000 != 0),
            "{run}: a civilian of who=1 without `unit_masks & 0x40000`"
        );
        let firsts = |n: i64| -> Vec<(String, i64)> {
            f[&n].2.iter().map(|(k, v)| (k.clone(), *v)).collect()
        };
        assert_eq!(
            firsts(611).first(),
            Some(&("GATHERORDER".to_string(), 611)),
            "{run}: the citizen gathers on its birth block"
        );
        assert!(firsts(616).is_empty(), "{run}: the caravan takes an order");
        assert!(firsts(626).is_empty(), "{run}: the scholar takes an order");
        assert_eq!(
            firsts(631),
            [
                ("CASTORDER".to_string(), 1151),
                ("MOVEORDER".to_string(), 631)
            ],
            "{run}: the fur trapper's orders"
        );
    }
    // The merchant is the one of the five whose orders differ in time:
    // it casts on 900 under `!ai off` and on 887 without it.
    let cast = |f: &Five| f[&621].2.get("CASTORDER").copied();
    assert_eq!((cast(&fa), cast(&fb)), (Some(900), Some(887)));
}

pub(super) fn chapter_seven_b_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch7b", "chapter7b", 7, 7, 1200) else {
        return;
    };
    // Closed at the trace's end since item 629: no value parts either
    // (`GOLDEN_WORD_CHAPTER_SEVEN_B`).
    assert_eq!(
        (w.word, w.sequence, w.value),
        (
            GOLDEN_WORD_CHAPTER_SEVEN_B,
            GOLDEN_WORD_CHAPTER_SEVEN_B,
            None
        ),
        "chapter seven-b's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §11"
    );
}

pub(super) fn chapter_seven_b_s_control_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch7bc", "chapter7b_control", 7, 6, 1200) else {
        return;
    };
    // Closed at the trace's end since item 647: no value parts either
    // (`GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL`).
    assert_eq!(
        (w.word, w.sequence, w.value),
        (
            GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL,
            GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL,
            None
        ),
        "chapter seven-b's control's word moved"
    );
}

pub(super) fn chapter_seven_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    const FIRST: i64 = WIDENING_CHAPTER_SEVEN.0;
    const LAST: i64 = WIDENING_CHAPTER_SEVEN.1;
    /// The window is 605..1199 and the `!quit` block is 1201.
    const NO_BLOCK: i64 = 1200;
    let mut summaries = Vec::new();
    for (run, stem) in [("ch7", "chapter7"), ("ch7c", "chapter7_control")] {
        let Some(firsts) = widen_civilians(
            run,
            stem,
            (FIRST, LAST),
            NO_BLOCK,
            0,
            (763, 764),
            false,
            LEADERS_TWO_KEYS,
        ) else {
            return;
        };
        summaries.push((run, firsts));
    }
    // **The standing rows of each capture's first block**, none of them
    // this chapter's: the unmodelled `form` on every unit, two
    // `filled_gather_slots` (chapter four's and five's families), and the
    // two capitals' `CITIES` rows, which chapter four's bleed window did
    // not print. The control adds its computer's scout `1/0`, whose group
    // and explore-order `facing` part from the first block there (parked
    // 275's mirror), and three more `form`s. Until item 644 two leader
    // `bucket`s stood here too, who=0's knowledge and metal at 0 against
    // 100: `library who=0 2` skipped `gain_tech`'s tail, and the Classical
    // age's starting grant is in it (29 → 27, 34 → 32).
    let standing = |what: &str| {
        what == "form"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("city:")
    };
    let control_standing = |what: &str| what == "group" || what == "order:move.facing";
    for ((run, firsts), (floor, under_want)) in summaries
        .iter()
        // run142's 32 lost the scout's `order:move.facing` (parked 275)
        // when `push_group` kept an equal group's record (item 723). Its
        // 31 lost the control scout `1/0`'s `group` on block 605, 65 here
        // against 64 there (a05f90a6, item 870): who=1's building groups
        // take a pool slot before the scout's first push, as the
        // original's do (`docs/GROUPS.md` §28.4, §29).
        // Item 1330 took ch7c's three staged births' `form` (30 → 27).
        // Item 1477 took the two `1/2000` rows `city:filled` and `city:land` (27 → 25,
        // the census walk starts at entry 1, `docs/AI.md` §122).
        // Item 1605 took the human's census rows (25 → 11, `docs/AI.md` §169).
        .zip([(11usize, &[][..] as &[i64]), (11usize, &[][..])])
    {
        let at_floor: Vec<&String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == FIRST)
            .map(|((_, _, what), _)| what)
            .collect();
        pin!(
            at_floor
                .iter()
                .all(|w| standing(w) || (*run == "ch7c" && control_standing(w)))
                && at_floor.len() == floor,
            "{run}: the standing rows on the first block moved ({}): {at_floor:?}",
            at_floor.len()
        );
        // **What parts under the word**, none of it a draw, pinned by key
        // and block. ~~The **Nubian hit points**: the caravan `0/7`, the
        // merchant `0/8` and the fur trapper `0/10` are born with 135, 135
        // and 180 in the original against this crate's 90, 90 and 120 —
        // `NUBIAN_HIT_POINTS`, "50% more" on merchants, caravans and
        // markets (`rules.xml`), which `Unit::update_hits@0060e930` reads
        // and this crate does not apply.~~ Agree since item 1268
        // (`Sim::unit_hits`; `docs/GOLDEN.md` §54): the caravan's by
        // `is_caravan`, the merchant's and the trapper's by their ids.
        // Player 0 is Nubia (its capital is
        // Napata). The citizen's and the scholar's agree. ~~And run141
        // alone carries the scout `1/0`'s `facing` from 847~~, gone since
        // `push_group` keeps an equal group's record (item 723); in run142 it stands from the
        // first block.
        let mut got: Vec<String> = firsts
            .iter()
            .filter(|((_, _, what), (f, _))| *f > FIRST && !standing(what))
            .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
            .collect();
        let mut want: Vec<String> = under_want
            .iter()
            .map(|f| format!("{f} 1/0 order:move.facing"))
            .collect();
        got.sort();
        want.sort();
        pin_eq!(got, want, "{run}: what parts under the word moved");
        // **The `GOOD` list agrees whole** on every block, `ever_seen`
        // included, bar the one good the dump cannot print.
        pin!(
            !firsts
                .keys()
                .any(|(w, _, what)| *w == 255 || what.starts_with("good")),
            "{run}: a good parts"
        );
    }
}

pub(super) fn chapter_six_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch6",
        "chapter6",
        WIDENING_CHAPTER_SIX,
        900,
        1,
        (GOLDEN_WORD_CHAPTER_SIX, GOLDEN_WORD_CHAPTER_SIX + 1),
        true,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch6 f{f} {w}/{o} {what}: {row}");
    }
    // **The standing rows of the first block**: the unmodelled `form`
    // on all ten start units, two `filled_gather_slots`, and the fourteen
    // start buildings as `build:extra`, which run168 cannot print.
    let standing = |what: &str| {
        what == "form" || what.starts_with("leader:filled_gather_slots") || what == "build:extra"
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_SIX.0)
        .map(|((_, _, what), _)| what)
        .collect();
    // Item 1605 took the human's `filled_gather_slots` (and city) rows: the
    // human leader takes the sweep (`docs/AI.md` §169).
    pin!(
        floor.iter().all(|w| standing(w)) && floor.len() == 24,
        "ch6: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    // **What parts under the word**, pinned by block and key (`docs/GOLDEN.md`
    // §10). Each aircraft's `form` on its birth block, the standing family,
    // and nothing else on any block of run168. **Item 652 staged `bird`**
    // at the channel's measured cursor, (0, 6), and the 701 rows went: the
    // AI scout `1/0`'s move order and path, which parted because
    // `think_scout`'s roll came one draw early without the bird's birth
    // draw. The window is run168 whole now, 605 to its last block, 899,
    // and no round is in either air.
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_SIX.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    // The births' `form` closed with item 1330 (`docs/GROUPS.md` §24.3).
    let want: Vec<String> = Vec::new();
    pin_eq!(got, want, "ch6: what parts under the word moved");
}

pub(super) fn chapter_six_b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch6b",
        "chapter6b",
        WIDENING_CHAPTER_SIX_B,
        1250,
        0,
        (632, 634),
        true,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch6b f{f} {w}/{o} {what}: {row}");
    }
    // **The standing rows of the first block**: the unmodelled `form` on
    // the eleven start units and two `filled_gather_slots` — chapter six's
    // family less its thirteen `build:extra`, because run175 prints
    // `BUILDS` and every building, the two Airbases included, agrees on
    // every block.
    let standing = |what: &str| what == "form" || what.starts_with("leader:filled_gather_slots");
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_SIX_B.0)
        .map(|((_, _, what), _)| what)
        .collect();
    // Item 1605 took the human's `filled_gather_slots` (and city) rows: the
    // human leader takes the sweep (`docs/AI.md` §169).
    pin!(
        floor.iter().all(|w| standing(w)) && floor.len() == 11,
        "ch6b: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    // **What parts under the word**, pinned by block and key
    // (`docs/GOLDEN.md` §10). Each aircraft's `form` on its birth block,
    // the standing family. Each aircraft's `order:target` on its birth
    // block, **the harness's and not the simulation's**: this crate's
    // attack order names the enemy Airbase exactly as run175's does, but
    // `Built::build_ids` names only a building the start dump linked, so
    // a staged Airbase reads `None` (item 651 probed it: with a fallback
    // to the building's own `(owner, index)` both rows go and nothing else
    // moves; parked 681). Item 1502 built that fallback, and they go.
    //
    // **Item 680 took the word to the capture's end**: a captain's attack
    // on a building re-searches every frame (`docs/COMBAT.md` §62), and
    // the Fighter's search at its attack point finds nothing. The rows
    // item 651 pinned on 633 and 634 are gone: the Fighter's `orders.len`,
    // the stack's `length` and `dest_angle`, and `idle`. The window is
    // run175 whole now, 605 to its last block, 1249; 1250 is the trace's
    // end and prints no block, as run168's 900. The Bomber `1/6` agrees on
    // every block, its alternating `ATTACK` and `MOVE` from 700 included.
    //
    // ~~One row stood that was neither aircraft's: the scout `1/0`'s
    // formation mirror `facing` from 991 (parked 275).~~ Gone since
    // `push_group` keeps an equal group's record (item 723).
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_SIX_B.0)
        .map(|((w, o, what), (f, row))| {
            if what == "order:move.facing" {
                format!("{f} {w}/{o} {what}: {row}")
            } else {
                format!("{f} {w}/{o} {what}")
            }
        })
        .collect();
    got.sort();
    // Item 1502: ["611 0/6 order:target", "616 1/6 order:target"] → [];
    // `target_ids` names the staged Airbase by `(owner, index)` (parked
    // 681's fallback, `docs/AI.md` §131.4), and both rows agree.
    let want: Vec<String> = Vec::new();
    pin_eq!(got, want, "ch6b: what parts under the word moved");
}

pub(super) fn chapter_nine_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch9",
        "chapter9",
        WIDENING_CHAPTER_NINE,
        1100,
        0,
        (692, 694),
        false,
        LEADERS_TWO_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch9 f{f} {w}/{o} {what}: {row}");
    }
    // **The standing rows of the first block**: the unmodelled `form` on
    // the eleven start units, two `filled_gather_slots`, and thirteen
    // `build:extra` — chapter six's family, since run180 does not print
    // `BUILDS` either.
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_NINE.0)
        .map(|((_, _, what), _)| what)
        .collect();
    // Item 1605 took the human's `filled_gather_slots` (and city) rows: the
    // human leader takes the sweep (`docs/AI.md` §169).
    pin!(
        floor.iter().all(|w| standing(w)) && floor.len() == 24,
        "ch9: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    // **What parts under the word**, by block and key (`docs/GOLDEN.md`
    // §17). Each staged unit's `form` on its birth block, the standing
    // family.
    //
    // **Item 676 took the word to the capture's end.** The 622 rows —
    // the chariot's plan through the unseen sand, its heading and its
    // position — went with `invalid_loc`'s human fog arm, and 648–649
    // with them; the one-frame re-plan on 693 went with `find_wpath`'s
    // human pop. The window is run180 whole, 605 to its last block,
    // 1099; 1100 is the `!quit` frame and prints no block.
    //
    // **642, the squad's `GroupMoveOrder` id**, 641000 against 647600,
    // stands: `(group.id + frame × 10) × 100 + order_num`, and the
    // original's `group.id` is the pool slot, 0 here (the chariot's push
    // took slot 1 on 621), where this crate's pushed group reads `64 +`
    // its index in `Sim::pushed`. It is a value no step reads, and the
    // fix reaches the AI's pushed groups on both long captures.
    //
    // ~~One row stood that was neither order's: the scout `1/0`'s
    // formation mirror `facing` from 847 (parked 275).~~ Gone since
    // `push_group` keeps an equal group's record (item 723).
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_NINE.0)
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
    pin_eq!(got, want, "ch9: what parts under the word moved");
}

pub(super) fn chapter_eight_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(firsts) = widen_civilians(
        "ch8",
        "chapter8",
        WIDENING_CHAPTER_EIGHT,
        0,
        1,
        // The first blow's two blocks, where what stands parts; the word
        // is the capture's end since item 668.
        (659, 661),
        true,
        CHAPTER_EIGHT_LEADER_KEYS,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch8 f{f} {w}/{o} {what}: {row}");
    }
    // **The standing rows of the first block**, chapter six's family:
    // the unmodelled `form` on the eleven start units, two
    // `filled_gather_slots`, and the thirteen start buildings as
    // `build:extra`, which run171 prints no `BUILDS` for.
    let standing = |what: &str| {
        what == "form"
            || what.starts_with("leader:filled_gather_slots")
            || what == "build:extra"
            // Item 2384: both leaders' `aggression` on the first block,
            // parked 2385's initializer.
            || what.starts_with("leader:aggression")
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_EIGHT.0)
        .map(|((_, _, what), _)| what)
        .collect();
    // Item 1605 took the human's `filled_gather_slots` (and city) rows: the
    // human leader takes the sweep (`docs/AI.md` §169).
    pin!(
        floor.iter().all(|w| standing(w)) && floor.len() == 26,
        "ch8: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    // **What parts under the word**, pinned by block and key
    // (`docs/GOLDEN.md` §12). Each staged unit's `form` on its birth
    // block, and the first blow's block. **Item 664** took the Spy
    // `1/9`'s nine rows on 618: with `is(SPY, 0)` modelled, its birth
    // think takes the region scan and it holds the original's explore
    // order, group and path. **Item 668** took `1/6`'s eleven rows on
    // 659 and its eight on 660, and `0/7`'s `damage_frame`: bumped by
    // `0/7` while it chased the General, `1/6` takes the enemy ladder's
    // arm C, re-searches from where it stands and strikes `0/7` on 659
    // (`docs/COLLISION.md` §14). The window is run171 whole since, and
    // **what stands** is the first blow's size and the meeting on 660:
    // `0/7` takes 2 hits and `damage_frac` 5 in run171 and 3 here, the General's
    // rally armor (parked 666), and both leaders' `treaties[·]` read 3
    // there and 0 here. Nothing parts after 660.
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_EIGHT.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    let mut want: Vec<String> = [
        // Item 3385: `0/7`'s `damage_frac`, `hits:damage`,
        // `hits:damage_frac` and `hits_left` agree (the General's rally armour, `Sim::rally_armor`).
        "660 0/-1 leader:treaties[1]",
        "660 1/-1 leader:treaties[0]",
    ]
    .iter()
    .map(|r| r.to_string())
    .collect();
    want.sort();
    pin_eq!(got, want, "ch8: what parts under the word moved");
}

pub(super) fn chapter_seven_b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    for (run, stem, window, print) in [
        ("ch7b", "chapter7b", WIDENING_CHAPTER_SEVEN_B, (1199, 1200)),
        (
            "ch7bc",
            "chapter7b_control",
            WIDENING_CHAPTER_SEVEN_B_CONTROL,
            (1199, 1200),
        ),
    ] {
        let Some(firsts) =
            widen_civilians(run, stem, window, 1200, 1, print, false, LEADERS_TWO_KEYS)
        else {
            return;
        };
        for ((w, o, what), (f, row)) in &firsts {
            eprintln!("  {run} f{f} {w}/{o} {what}: {row}");
        }
        // **The standing rows of the first block**, chapter seven's
        // families: the unmodelled `form`, two `filled_gather_slots` and
        // the two capitals' `CITIES` rows. The control adds its scout
        // `1/0`'s `group` and explore-order `facing`, as chapter seven's
        // control does. **Item 644 took seven** (34 → 27, 39 → 32): who=1's
        // knowledge and metal `bucket`s, 0 against 100, and its five
        // `resource_cap`s, 1392 against 2992. `library who=1 2` now raises
        // the levels through `gain_tech`'s tail, which pays the Classical
        // age's starting grant, and the caps read the Commerce level live,
        // as `calc_resource_caps` does every frame (parked 633).
        let standing = |what: &str| {
            what == "form"
                || what.starts_with("leader:filled_gather_slots")
                || what.starts_with("city:")
        };
        let control = |what: &str| what == "group" || what == "order:move.facing";
        let at_floor: Vec<&String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == window.0)
            .map(|((_, _, what), _)| what)
            .collect();
        // The control's 32 was 31 and the scout's `order:move.facing`
        // (parked 275), gone since `push_group` keeps an equal group's
        // record (item 723). Its 31 was 30 and the control scout `1/0`'s
        // `group` on block 605, 65 here against 64 there, gone since
        // who=1's building groups take a pool slot first (a05f90a6, item
        // 870; `docs/GROUPS.md` §29).
        // Item 1330 took the control's three staged births' `form` (30 →
        // 27, `docs/GROUPS.md` §24.3). Item 1477 took the two `1/2000` rows
        // `city:filled` and `city:land` (27 → 25, `docs/AI.md` §122).
        // Item 1605 took the human's census rows (25 → 11, `docs/AI.md`
        // §169).
        let floor = 11;
        pin!(
            at_floor
                .iter()
                .all(|w| standing(w) || (run == "ch7bc" && control(w)))
                && at_floor.len() == floor,
            "{run}: the standing rows on the first block moved ({}): {at_floor:?}",
            at_floor.len()
        );
        // **What parts under each word**, pinned by block and key, none of
        // it a mechanism (`docs/GOLDEN.md` §11). run156: the scout `1/0`'s
        // `facing` from 847, as in chapter seven (parked 635). Until item
        // 629 the merchant `1/8` stood 24 units off the original's when its
        // cast ended on 1070, and the fur trapper `1/10`'s move was handed
        // another `dest_y` on 1091, 14408 against 14804, whose turn was the
        // word 1148: 22 rows, gone with `cast_unpack`'s merchant arm (the
        // move's value diff; its delta is the constant's). run157, **the
        // control's block, item 632** (the delta is in
        // `GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL`'s comment): `1/1` keeps its
        // `BUILDORDER` behind the goody box's walk from 990, and only the
        // group id stands there, 64 against 65, the two-slot permutation
        // the scout's standing `group` row began (`docs/GROUPS.md` §24).
        // Past it the scout `1/0`'s explore path parts from 1077, value
        // only. **Item 644's block** (the delta is in the constant's
        // comment): who=1's food bucket from 1018 and `1/7`'s thirteen
        // rows on 1177 are gone. The food was a goody pile paid to metal
        // here, and `1/7` is now sent toward the Woodcutter's Camp the
        // script places and destroys on 1176 and keeps its walk and
        // `BUILDORDER` to the word, as the original's does: on 1177 both
        // sides hold explore-to (40248, 23160) and the `BUILDORDER` on
        // `1/2009`, at (40271, 18360). Nothing new parts on 1177..1188.
        // **Item 647's block** (the delta is in the constant's comment):
        // the scout `1/0`'s eighteen rows are gone — its path from 1077
        // (47 nodes against 48, `path[41..46].to`), its position, figures
        // and `dest_y` from 1116, and its `dest`/`dest_x` from 1137. The
        // path was built across the Small City `1/2007`'s footprint, which
        // this crate's who=1 had not seen: `Wall::start` now writes the
        // owner's bit over it (`docs/SCOUT.md` §14). The window is the
        // capture whole, and only the group id on 990 stands.
        let mut got: Vec<String> = firsts
            .iter()
            .filter(|((_, _, what), (f, _))| *f > window.0 && what != "form")
            .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
            .collect();
        got.sort();
        // run156's scout row on 847 (parked 275) is gone since
        // `push_group` keeps an equal group's record (item 723). The
        // control's `1/1` `group` on 990, 64 here against 65 there — §24's
        // two-slot permutation — agrees since a05f90a6 (item 870): who=1's
        // building groups take their pool slots first, as the original's
        // do (`docs/GROUPS.md` §29).
        let want: &[&str] = &[];
        pin_eq!(got, want, "{run}: what parts under the word moved");
        pin!(
            !firsts
                .keys()
                .any(|(w, _, what)| *w == 255 || what.starts_with("good")),
            "{run}: a good parts"
        );
    }
}

pub(super) fn chapter_three_holds_to_the_golden_word() {
    let Some(w) = walk_chapter("ch3", 3, 6, 900) else {
        return;
    };
    assert_eq!(
        (w.word, w.sequence, w.value),
        (GOLDEN_WORD_CHAPTER_THREE, GOLDEN_WORD_CHAPTER_THREE, None),
        "chapter three's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §7"
    );
}

pub(super) fn chapter_three_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    const FIRST: i64 = WIDENING_CHAPTER_THREE.0;
    const WORD: i64 = GOLDEN_WORD_CHAPTER_THREE;
    let Some(w) = widen_chapter_three(
        "ch3",
        "chapter3",
        WIDENING_CHAPTER_THREE,
        WORD,
        8,
        &[651, 652, 753, 754, WORD, WORD + 1],
    ) else {
        return;
    };
    let (firsts, ammo_theirs, ammo_ours) = (w.firsts, w.ammo_theirs, w.ammo_ours);
    // **Anti-vacuity for the `AMMO` record**: run145's live rounds are all
    // read — the chariots' from 651 — and this crate's side is not empty.
    pin_eq!(ammo_theirs, 89, "run145's live rounds are not all read");
    pin!(ammo_ours > 0, "this crate fired no round in the window");
    // **The standing families on the capture's first block**, none of
    // them the chapter's: the unmodelled `form` (also every staged unit's
    // birth row), `build:extra` (the end detail prints no `BUILDDATA`)
    // and player 0's two `filled_gather_slots`. Until item 644 four more
    // stood here, `leader:bucket` 3 and 4 for both players, 0 against the
    // dump's 100: `age who=N 2` raised the ages without `gain_tech`'s
    // tail, and the Classical age's starting grant is in the tail
    // (`Sim::set_leader_levels`). 30 → 26.
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let at_floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == FIRST)
        .map(|((_, _, what), _)| what)
        .collect();
    // Item 1605 took the human's `filled_gather_slots` (and city) rows: the
    // human leader takes the sweep (`docs/AI.md` §169).
    pin!(
        at_floor.iter().all(|w| standing(w)) && at_floor.len() == 24,
        "the standing rows on run145's first block moved: {at_floor:?}"
    );
    // **What parts first, and what parts on the word's own two blocks**
    // (item 601; the delta is in `GOLDEN_WORD_CHAPTER_THREE`'s comment).
    // 635 is quiet now: `0/6` takes `1/8`, as the dump's does, once the
    // flank reduction reads the target's mask. The first value parting is
    // the chariots' first rounds on 651, which leave from each unit's own
    // square and height where the dump's leave from the archer's release
    // node (`docs/COMBAT.md` §52.4, item 602's 651 on run146). `rolling`
    // parts with them. No key parts first on the word's blocks (item
    // 603): `0/8` holds its heading on 685 as the dump's does, its pivot
    // bearing from its node, and `1/4`'s move offsets, which followed
    // 684's draws, agree until 804.
    let rows_on = |lo: i64, hi: i64| -> Vec<String> {
        firsts
            .iter()
            .filter(|((_, _, what), (f, _))| (lo..=hi).contains(f) && !standing(what))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect()
    };
    let first = firsts
        .iter()
        .filter(|((_, _, what), _)| !standing(what))
        .map(|(_, (f, _))| *f)
        .min();
    let whole_on = |lo: i64, hi: i64| -> Vec<&String> {
        w.at_word
            .iter()
            .filter(|r| {
                r.split(' ')
                    .next()
                    .and_then(|b| b.parse::<i64>().ok())
                    .is_some_and(|b| (lo..=hi).contains(&b))
                    && !r
                        .split(": ")
                        .next()
                        .is_some_and(|k| standing(k.rsplit(' ').next().unwrap_or("")))
            })
            .collect()
    };
    // **Item 602: the word is the capture's end.** With the release
    // through the pivot turret (`docs/COMBAT.md` §55) the chariots' rounds
    // leave from the dump's point, and 651–652 agree on every record and
    // figure. What parts past the standing families, every row first or
    // not on the blocks kept whole, is three families of the `AMMO`
    // record, and no draw follows from any of them to 900:
    //
    // - a round's target, cleared here on its target's death (678, 705,
    //   757) where the original's `Ammo::check_hit` rewrites it on the
    //   round's due frame;
    // - the pool slot a round takes (703, 729, 730, 754), one apart;
    // - ~~`0/8`'s launch point on 753, one unit in `sx` and `sy`, after
    //   its turn to 61.7° on 711~~: **item 1257** — the release rotates by
    //   `fast_angle_to_degrees` of the facing, not `angle_to_degrees`
    //   (`8e4c7d`–`8e4c98`), 240 against 242 at 61.7°, and the round
    //   leaves from the dump's `941, 8155` with no turret step at all.
    pin_eq!(first, Some(678), "run145's first value parting moved");
    pin_eq!(
        rows_on(FIRST + 1, WORD + 1),
        vec![
            "705 0/6 ammo[1].ox: ours -1 theirs 7",
            "705 0/6 ammo[1].whom: ours -1 theirs 1",
            "730 0/6 ammo[2]: this crate holds it alone",
            "678 0/6 ammo[2].ox: ours -1 theirs 8",
            "678 0/6 ammo[2].whom: ours -1 theirs 1",
            "730 0/6 ammo[5]: the dump holds it alone",
            "754 0/7 ammo[1]: this crate holds it alone",
            "703 0/7 ammo[2]: this crate holds it alone",
            "703 0/7 ammo[3]: the dump holds it alone",
            "754 0/7 ammo[4]: the dump holds it alone",
            "757 0/8 ammo[0].ox: ours -1 theirs 6",
            "757 0/8 ammo[0].whom: ours -1 theirs 1",
            "729 0/8 ammo[1]: this crate holds it alone",
            "678 0/8 ammo[1].ox: ours -1 theirs 8",
            "678 0/8 ammo[1].whom: ours -1 theirs 1",
            "729 0/8 ammo[4]: the dump holds it alone",
        ],
        "a row past run145's first rounds moved"
    );
    pin_eq!(
        whole_on(651, 652),
        Vec::<&String>::new(),
        "the first rounds' launch blocks part again"
    );
    pin_eq!(
        whole_on(753, 754),
        vec![
            "754 0/7 ammo[1]: this crate holds it alone",
            "754 0/7 ammo[4]: the dump holds it alone",
        ],
        "the turned chariot's round moved (item 1257: its launch agrees)"
    );
    pin_eq!(
        whole_on(WORD, WORD + 1),
        Vec::<&String>::new(),
        "the capture's last block parts"
    );
    let catapult: Vec<String> = firsts
        .iter()
        .filter(|((w, o, what), _)| (*w, *o) == (0, 9) && !standing(what))
        .map(|((_, _, what), (f, row))| format!("{f} {what}: {row}"))
        .collect();
    pin!(
        catapult.is_empty(),
        "the catapult parts from the dump again: {catapult:?}"
    );
}

pub(super) fn chapter_three_s_catapult_unpacks_on_the_dump_s_frame() {
    const CAST: i64 = sim::orders::index::CAST_SPELL as i64;
    for (run, stem, o, want) in [
        (
            "ch3",
            "chapter3",
            9_i16,
            vec![
                (621, 1, 1, -1, -1),
                (622, 2, 1, -1, -1),
                (695, 6, 1, -1, -1),
                (696, 7, 1, CAST, 652),
                (697, 0, 1, CAST, 652),
                (775, 0, 1, CAST, 652),
                (776, 0, 0, -1, -1),
                (777, 1, 0, -1, -1),
            ],
        ),
        (
            "ch3b",
            "chapter3b",
            6,
            vec![
                (606, 1, 1, -1, -1),
                (683, 7, 1, -1, -1),
                (698, 7, 1, -1, -1),
                (699, 8, 1, CAST, 652),
                (700, 0, 1, CAST, 652),
                (778, 0, 1, CAST, 652),
                (779, 0, 0, -1, -1),
            ],
        ),
    ] {
        let at: Vec<i64> = want.iter().map(|r| r.0).collect();
        let Some(rows) = catapult_rows(run, stem, o, &at) else {
            return;
        };
        let theirs: Vec<_> = rows.iter().map(|r| r[0]).collect();
        let ours: Vec<_> = rows.iter().map(|r| r[1]).collect();
        assert_eq!(
            theirs, want,
            "{run}: the dump's catapult is not the one pinned"
        );
        assert_eq!(
            ours, theirs,
            "{run}: this crate's catapult parts from the dump's"
        );
    }
}

pub(super) fn chapter_three_s_unpacked_catapult_sees_its_hoplites() {
    const ATTACK: i64 = sim::orders::index::ATTACK as i64;
    let Some(mut s) = stage_script("ch3b", "chapter3b") else {
        return;
    };
    let want = [(779, 0, -1, -1, -1), (780, 1, ATTACK, 2472, 8136)];
    let mut rows = Vec::new();
    for f in 0..780 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < 779 {
            continue;
        }
        let ix =
            s.ix.frames()
                .iter()
                .position(|x| x.number == n)
                .expect("the block");
        let frame = s.ix.frame_state(ix).unwrap();
        let them = frame
            .units
            .iter()
            .find(|u| u.who == 0 && u.o == 6)
            .expect("the catapult in the dump");
        let head = them.orders_front_first().next();
        let at = head
            .and_then(|h| Some((h.whom?, h.ox?)))
            .and_then(|(w, o)| {
                frame
                    .units
                    .iter()
                    .find(|u| u.who == w && u.o == o)
                    .map(|t| (t.pos.x, t.pos.y))
            });
        let theirs = (
            n,
            them.idle.expect("idle"),
            head.map_or(-1, |h| h.index),
            at.map_or(-1, |p| p.0),
            at.map_or(-1, |p| p.1),
        );
        let sim = &s.built.sim;
        let u = &sim.units[sim.unit_by_o(0, 6).expect("our catapult")];
        let at = match u.combat.target {
            Some(sim::combat::Obj::Unit(t)) => Some(sim.units[t].pos),
            _ => None,
        };
        let ours = (
            n,
            i64::from(u.idle),
            u.orders.front().map_or(-1, |h| i64::from(h.index())),
            at.map_or(-1, |p| i64::from(p.x)),
            at.map_or(-1, |p| i64::from(p.y)),
        );
        eprintln!("  ch3b 0/6 block {n}: theirs {theirs:?} ours {ours:?}");
        rows.push((theirs, ours));
    }
    let theirs: Vec<_> = rows.iter().map(|r| r.0).collect();
    let ours: Vec<_> = rows.iter().map(|r| r.1).collect();
    assert_eq!(theirs, want, "the dump's catapult is not the one pinned");
    assert_eq!(ours, theirs, "this crate's catapult parts from the dump's");
}

pub(super) fn chapter_three_s_catapult_fires_on_the_ground() {
    use sim::orders::Body;
    let Some(mut s) = stage_script("ch3b", "chapter3b") else {
        return;
    };
    let mut parted = Vec::new();
    let mut round = None;
    for f in 0..864 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < 779 {
            continue;
        }
        let ix =
            s.ix.frames()
                .iter()
                .position(|x| x.number == n)
                .expect("the block");
        let frame = s.ix.frame_state(ix).unwrap();
        let them = frame
            .units
            .iter()
            .find(|u| u.who == 0 && u.o == 6)
            .expect("the catapult in the dump");
        let theirs = (ground_rows(them), them.recharging.expect("recharging"));
        let sim = &s.built.sim;
        let i = sim.unit_by_o(0, 6).expect("our catapult");
        let u = &sim.units[i];
        let rows: Vec<[i64; 6]> = u
            .orders
            .iter()
            .map(|o| {
                let flags = i64::from(o.flags as i8);
                match o.body {
                    Body::AttackGround(g) => [
                        i64::from(o.index()),
                        i64::from(g.at.x),
                        i64::from(g.at.y),
                        i64::from(g.sea),
                        i64::from(g.attack_unit),
                        flags,
                    ],
                    Body::Attack(a) => [
                        i64::from(o.index()),
                        i64::from(a.in_range),
                        i64::from(a.new_ord),
                        -1,
                        -1,
                        flags,
                    ],
                    _ => [i64::from(o.index()), -1, -1, -1, -1, flags],
                }
            })
            .collect();
        let ours = (rows, i64::from(u.combat.recharging));
        if ours != theirs {
            parted.push((n, ours, theirs));
        }
        if round.is_none() {
            let text = s.ix.read_frame(ix).unwrap();
            if let Some((a, _)) = crate::diff::ammo::blocks(&text)
                .into_iter()
                .find(|(a, _)| a.who == 0 && a.o == 6)
            {
                let mine = sim
                    .projectiles
                    .iter()
                    .find(|p| p.shooter == sim::combat::Obj::Unit(i))
                    .map(|p| {
                        (
                            p.target.is_some(),
                            p.launch.x,
                            p.launch.y,
                            p.sz,
                            p.landing.x,
                            p.landing.y,
                            p.ez,
                            p.accuracy,
                            p.total_time,
                        )
                    });
                round = Some((
                    n,
                    mine,
                    (
                        a.ox != -1,
                        a.sx as i32,
                        a.sy as i32,
                        a.sz as i32,
                        a.ex as i32,
                        a.ey as i32,
                        a.ez as i32,
                        a.accuracy as i32,
                        a.total_time as i32,
                    ),
                ));
            }
        }
    }
    assert!(parted.is_empty(), "0/6's ground order parts: {parted:?}");
    let (n, mine, theirs) = round.expect("the dump's catapult fires");
    eprintln!("  ch3b 0/6 round on {n}: ours {mine:?} theirs {theirs:?}");
    assert_eq!(n, 798, "the dump's round prints on 798");
    assert_eq!(
        theirs,
        (false, 855, 7990, 496, 2413, 8276, 188, 5, 33),
        "the dump's round is not the one pinned"
    );
    // What the order decides agrees: the release block, no object, the
    // ground's `ez` at the point, the accuracy and the flight. **And the
    // landing** (item 625): the point plus the scatter's two draws, taken
    // on 798, now that the stream holds to 792 — (2413, 8276) on both
    // sides, where ours read (2655, 8142) while the crew's walk parted
    // the stream on 782. **And the launch** (item 2304, `docs/GOLDEN.md`
    // §83): the unit's own square here, (888, 7992, 251), until piece
    // 567's release row was read off run842's packet.
    let mine = mine.expect("this crate's catapult fires");
    assert_eq!(
        (mine.0, mine.4, mine.5, mine.6, mine.7, mine.8),
        (theirs.0, theirs.4, theirs.5, theirs.6, theirs.7, theirs.8),
        "this crate's round parts from the dump's on what the order decides"
    );
    assert_eq!(
        (mine.1, mine.2, mine.3),
        (theirs.1, theirs.2, theirs.3),
        "the launch parts from the dump's"
    );
}

pub(super) fn chapter_three_s_crew_mirror_the_turn() {
    let Some(mut s) = stage_script("ch3b", "chapter3b") else {
        return;
    };
    let mut rows = Vec::new();
    for f in 0..785 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < 781 {
            continue;
        }
        let ix =
            s.ix.frames()
                .iter()
                .position(|x| x.number == n)
                .expect("the block");
        let frame = s.ix.frame_state(ix).unwrap();
        let them = frame
            .units
            .iter()
            .find(|u| u.who == 0 && u.o == 6)
            .expect("the catapult in the dump");
        let sim = &s.built.sim;
        let u = &sim.units[sim.unit_by_o(0, 6).expect("our catapult")];
        let lead = u.guys[0];
        for (g, theirs) in u.guys.iter().zip(&them.guys).skip(1) {
            let at = g.follow.expect("a tracked crew figure").body.pos;
            let tp = theirs.pos.expect("the dump's figure");
            rows.push((
                n,
                i64::from(g.anim),
                i64::from(g.anim) == i64::from(lead.anim) && g.cur_time == lead.cur_time,
                (i64::from(at.x), i64::from(at.y)) == (tp.x, tp.y),
            ));
        }
    }
    let want: Vec<_> = (781..=785)
        .flat_map(|n| [(n, 21, true, true), (n, 21, true, true)])
        .collect();
    assert_eq!(
        rows, want,
        "0/6's crew on the turn: (block, slot, mirrored, on the dump's point)"
    );
}

pub(super) fn chapter_three_s_restage_holds_to_its_word() {
    let Some(w) = walk_script("ch3b", "chapter3b", 3, 7, 1000) else {
        return;
    };
    // **Item 627: closed.** Word, sequence and values hold to the
    // capture's end, 1000, as run145's do to 900.
    assert_eq!(
        (w.word, w.sequence, w.value),
        (
            GOLDEN_WORD_CHAPTER_THREE_RESTAGE,
            GOLDEN_WORD_CHAPTER_THREE_RESTAGE,
            None
        ),
        "run146's word moved; re-pin it here and say so in docs/GOLDEN.md §7"
    );
}

pub(super) fn chapter_three_s_restage_is_widened_whole() {
    let _pins = Pins::hold();
    const WORD: i64 = GOLDEN_WORD_CHAPTER_THREE_RESTAGE;
    let Some(w) = widen_chapter_three(
        "ch3b",
        "chapter3b",
        WIDENING_CHAPTER_THREE_RESTAGE,
        WORD,
        6,
        &[651, 652, 780, 781, 865, 866, 868, 869, WORD, WORD + 1],
    ) else {
        return;
    };
    let firsts = w.firsts;
    pin_eq!(w.ammo_theirs, 89, "run146's live rounds are not all read");
    pin!(w.ammo_ours > 0, "this crate fired no round in the window");
    // The standing families on the first block are run145's, the same 26.
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let at_floor = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_THREE_RESTAGE.0)
        .map(|((_, _, what), _)| what)
        .collect::<Vec<_>>();
    // Item 1605 took the human's `filled_gather_slots` (and city) rows: the
    // human leader takes the sweep (`docs/AI.md` §169).
    pin!(
        at_floor.iter().all(|w| standing(w)) && at_floor.len() == 24,
        "the standing rows on run146's first block moved: {at_floor:?}"
    );
    // **What parts first, and on the word's own two blocks** (item 595;
    // the delta is in `GOLDEN_WORD_CHAPTER_THREE_RESTAGE`'s comment). The
    // first row past the standing families is `0/8`'s first arrow on 651,
    // which now leaves on the dump's frame from the wrong point: the
    // unit's own square and height, where the dump's leaves from the
    // archer's release node. **Nothing parts on the word's blocks**, 664
    // and 665: the draw is the crew figure's swing, and `GUYS=2` prints no
    // figure's clock. That is a blind row, named so that an empty list is
    // not read as agreement.
    let rows_on = |lo: i64, hi: i64| -> Vec<String> {
        firsts
            .iter()
            .filter(|((_, _, what), (f, _))| (lo..=hi).contains(f) && !standing(what))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect()
    };
    let first = firsts
        .iter()
        .filter(|((_, _, what), _)| !standing(what))
        .map(|(_, (f, _))| *f)
        .min();
    // **Item 602: 664 → 780.** The crew swings with its leader
    // (`Guy::inc_time`'s foot, `docs/ANIM.md` §5.2) and the rounds leave
    // through the turret (`docs/COMBAT.md` §55). 651–652 agree whole. The
    // first parting is a round's target cleared on its target's death
    // (736), as on run145; on 771 the dump culls the dead hoplites'
    // `DEATH_OBJS` as arena A's are born (`docs/COMBAT.md` §42.5's cull,
    // which this crate does not do), so their `o`s link differently.
    //
    // **Item 616: 780 → 782** (the delta is in
    // `GOLDEN_WORD_CHAPTER_THREE_RESTAGE`'s comment; this is the word's
    // block). `cast_unpack` lights the whole disc (`docs/COMBAT.md` §56),
    // so the catapult `0/6` takes an attack on 780 and turns on 781 as the
    // dump's does: no angle or figure-position row parts on 781 now. Its
    // 780 `order:target` row is 617's numbering, not a choice: ours `1/8`
    // and the dump's `1/11` are the same hoplite at (2472, 8136)
    // (`chapter_three_s_unpacked_catapult_sees_its_hoplites`). What parts
    // on 781 is the siege arm this crate does not carry: the original
    // pushes an `ATTACKGROUNDORDER` (index 23) over the attack at the
    // target's point, its figure keeps `ox −1`, and its reload reads 83
    // where ours reads 82 (`docs/COMBAT.md` §56.3).
    //
    // **Item 617: 792 → 865** (the delta is in
    // `GOLDEN_WORD_CHAPTER_THREE_RESTAGE`'s comment; this is the word's
    // block). `find_free` skips a held dead number (`docs/COMBAT.md` §59),
    // so arena A's hoplites are `1/9`–`1/11` on both sides and every
    // `extra`/`unlinked` row and the catapult's `order:target` are gone.
    // What the numbering had hidden, in frame order past 736: the
    // catapult's round takes pool slot 0 here and 1 in the dump on 798
    // (run145's pool-slot family, §55.5); the scout `1/0`'s fresh
    // `EXPLORETOORDER` move reads `facing 1` here and 0 in the dump on
    // 847, no draw; and on 865–866 the catapult's fresh attack after the
    // reload (621's park), which was the word.
    //
    // **Item 627: 865 → 1000, closed** (the delta is in
    // `GOLDEN_WORD_CHAPTER_THREE_RESTAGE`'s comment; this is the word's
    // block). An unpacked packer's search must reach what it takes, and
    // a packer re-searches before it chases (`docs/COMBAT.md` §60), so
    // every `0/6` row on 865–866 is gone and nothing new parts to the
    // capture's end. What stands, past the first rounds, is three
    // families and no draw: a round's target (736), a round's pool slot
    // (798) and the scout's move facing (847).
    pin_eq!(first, Some(736), "run146's first value parting moved");
    pin_eq!(
        rows_on(WIDENING_CHAPTER_THREE_RESTAGE.0 + 1, WORD + 1),
        vec![
            "798 0/6 ammo[0]: this crate holds it alone",
            "798 0/6 ammo[1]: the dump holds it alone",
            "736 0/7 ammo[0].ox: ours -1 theirs 8",
            "736 0/7 ammo[0].whom: ours -1 theirs 1",
            "847 1/0 order:move.facing: Move { field: \"facing\", ours: 1, theirs: 0 }",
        ],
        "a row past run146's first rounds moved"
    );
    let whole: Vec<&String> = w
        .at_word
        .iter()
        .filter(|r| {
            !r.split(": ")
                .next()
                .is_some_and(|k| standing(k.rsplit(' ').next().unwrap_or("")))
        })
        .collect();
    // The blocks kept whole are the move's, 780–781, and the word's.
    // **Item 621**: the siege arm is carried, so every `0/6` row the
    // ground order made on 781–783 — the order list's length, the
    // unspellable kind, the figure's `ox`/`whom`, the reload's 83 — is
    // gone. What stands is 617's numbering on the attack beneath the
    // ground order, the same hoplite by position.
    //
    // **Item 625: 782 → 792**: the crew mirror the turn
    // (`docs/COMBAT.md` §58), and the word's blocks were 792–793, where
    // nothing parted but 617's numbering.
    //
    // **Item 617: 792 → 865** (the delta is in
    // `GOLDEN_WORD_CHAPTER_THREE_RESTAGE`'s comment; this is the word's
    // block). The numbering is the dump's (`docs/COMBAT.md` §59), so
    // 780–781 are quiet whole: the catapult's attack names `1/11` on both
    // sides. The word's blocks, 865–866, are the catapult's fresh attack
    // after its reload, with its chase on 866 (621's park, §57.6): one
    // order here and none in the dump, then its turn and its crew. The
    // scout `1/0`'s `facing` row has stood since 847 and spends no draw.
    //
    // **Item 627: 865 → 1000** (the delta is in
    // `GOLDEN_WORD_CHAPTER_THREE_RESTAGE`'s comment; this is the word's
    // block). The blocks kept whole are 865–866, the old word's, and
    // 868–869, the catapult's first retaliation and the block it drops
    // it, as well as the capture's `!quit` block, 1001. Every `0/6` row
    // is gone from them; the scout's `facing` row stands on each until it
    // ends, and 1001 is quiet.
    pin_eq!(
        whole,
        vec![
            "865 1/0 order:move.facing: Move { field: \"facing\", ours: 1, theirs: 0 }",
            "866 1/0 order:move.facing: Move { field: \"facing\", ours: 1, theirs: 0 }",
            "868 1/0 order:move.facing: Move { field: \"facing\", ours: 1, theirs: 0 }",
            "869 1/0 order:move.facing: Move { field: \"facing\", ours: 1, theirs: 0 }",
        ],
        "a row on run146's whole blocks moved"
    );
}

pub(super) fn chapter_three_s_restage_numbers_its_objects() {
    use std::collections::{BTreeMap, BTreeSet};
    let (first, last) = WIDENING_CHAPTER_THREE_RESTAGE;
    let Some(mut s) = stage_script("ch3b", "chapter3b") else {
        return;
    };
    type Units = BTreeMap<(i64, i64), (i64, i64)>;
    let (mut ours_prev, mut theirs_prev): (Units, Units) = Default::default();
    let (mut ours_do, mut theirs_do) = (BTreeSet::new(), BTreeSet::new());
    let (mut ours_ammo, mut theirs_ammo) = (BTreeSet::new(), BTreeSet::new());
    let mut rows: Vec<String> = Vec::new();
    let mut events = 0usize;
    for f in 0..last - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = s.ix.frame_state(at).unwrap();
        let sim = &s.built.sim;
        let ours: Units = sim
            .units
            .iter()
            .filter(|u| u.alive() && u.owner < 8)
            .map(|u| {
                (
                    (i64::from(u.owner), i64::from(u.index)),
                    (i64::from(u.pos.x), i64::from(u.pos.y)),
                )
            })
            .collect();
        let theirs: Units = frame
            .units
            .iter()
            .filter(|u| (0..8).contains(&u.who))
            .map(|u| ((u.who, u.o), (u.pos.x, u.pos.y)))
            .collect();
        let raw = s.ix.read_frame(at).unwrap();
        let o_do: BTreeSet<(i64, i64, i64)> = sim
            .deaths
            .iter()
            .map(|d| (i64::from(d.who), i64::from(d.o), d.first_frame))
            .collect();
        let t_do: BTreeSet<(i64, i64, i64)> = frame
            .deaths
            .iter()
            .filter(|d| d.valid == Some(1))
            .filter_map(|d| Some((d.who?, d.o?, d.first_frame?)))
            .collect();
        let o_ammo: BTreeSet<(i64, i64, i64)> = sim
            .projectiles
            .iter()
            .filter_map(|p| match p.shooter {
                sim::combat::Obj::Unit(u) => {
                    let un = &sim.units[u];
                    Some((i64::from(un.owner), i64::from(un.index), i64::from(p.slot)))
                }
                sim::combat::Obj::Building(_) => None,
            })
            .collect();
        let t_ammo: BTreeSet<(i64, i64, i64)> = super::ammo::blocks(&raw)
            .into_iter()
            .filter(|(a, _)| a.flags & 2 != 0 && a.who < 8)
            .map(|(a, _)| (a.who, a.o, a.index))
            .collect();
        // The window's first block seeds both sides: block 1 before it
        // prints no unit, so everything on 605 would read as born.
        if n > first {
            // Births, paired on the frame, the owner and the point.
            let born = |now: &Units, prev: &Units| -> BTreeMap<(i64, i64, i64), i64> {
                now.iter()
                    .filter(|(k, _)| !prev.contains_key(k))
                    .map(|(&(who, o), &(x, y))| ((who, x, y), o))
                    .collect()
            };
            let (ob, tb) = (born(&ours, &ours_prev), born(&theirs, &theirs_prev));
            for k in ob.keys().chain(tb.keys()).collect::<BTreeSet<_>>() {
                events += 1;
                let (who, x, y) = *k;
                match (ob.get(k), tb.get(k)) {
                    (Some(a), Some(b)) if a == b => {}
                    (a, b) => rows.push(format!(
                        "{n} birth {who} at ({x}, {y}): ours {a:?} theirs {b:?}"
                    )),
                }
            }
            // Deaths, on the number.
            let died = |now: &Units, prev: &Units| -> BTreeSet<(i64, i64)> {
                prev.keys()
                    .filter(|k| !now.contains_key(k))
                    .copied()
                    .collect()
            };
            let (od, td) = (died(&ours, &ours_prev), died(&theirs, &theirs_prev));
            for k in od.symmetric_difference(&td) {
                rows.push(format!(
                    "{n} death {}/{}: {} alone",
                    k.0,
                    k.1,
                    if od.contains(k) { "ours" } else { "the dump's" }
                ));
            }
            events += od.union(&td).count();
            // The death-object list and the rounds, arrivals and leavings.
            for (what, o_prev, t_prev, o_now, t_now) in [
                ("death_obj", &ours_do, &theirs_do, &o_do, &t_do),
                ("ammo", &ours_ammo, &theirs_ammo, &o_ammo, &t_ammo),
            ] {
                for (dir, o_set, t_set) in [
                    ("arrives", o_now - o_prev, t_now - t_prev),
                    ("leaves", o_prev - o_now, t_prev - t_now),
                ] {
                    events += o_set.union(&t_set).count();
                    for k in o_set.symmetric_difference(&t_set) {
                        rows.push(format!(
                            "{n} {what} {dir} {k:?}: {} alone",
                            if o_set.contains(k) {
                                "ours"
                            } else {
                                "the dump's"
                            }
                        ));
                    }
                }
            }
        }
        (ours_prev, theirs_prev) = (ours, theirs);
        (ours_do, theirs_do) = (o_do, t_do);
        (ours_ammo, theirs_ammo) = (o_ammo, t_ammo);
    }
    for r in &rows {
        eprintln!("  numbering: {r}");
    }
    assert!(events > 0, "run146's numbering walk read no event");
    // **The floor, before any reading (item 617)**, and what it was when
    // it was written: every death agrees
    // (`1/6` on 680, `1/7` on 728, `1/8` on 736), and so does the whole
    // death-object list: the three records arrive on those blocks on both
    // sides and **none leaves** before the capture ends. So nothing is
    // culled on 771 or anywhere else in the window. The first parting is
    // the births on 771: arena A's three hoplites, born on the same
    // points, take the dump's 9–11 and ours' 6–8, the numbers of the dead
    // whose death objects both sides still hold. **Since item 617's
    // build** (`docs/COMBAT.md` §59) those three rows are gone: a dead
    // number is not handed out while its death object holds it.
    //
    // The rounds' pool slot is a second family, run145's (`docs/COMBAT.md`
    // §55.5): the catapult's one round takes slot 1 in the dump and 0
    // here on 798. **Since item 627** (`docs/COMBAT.md` §60) ours no
    // longer fires again on 960: the catapult's fresh attack after its
    // reload, and the chase that carried it out of its minimum, are gone.
    assert_eq!(
        rows,
        vec![
            "798 ammo arrives (0, 6, 0): ours alone",
            "798 ammo arrives (0, 6, 1): the dump's alone",
            "830 ammo leaves (0, 6, 0): ours alone",
            "830 ammo leaves (0, 6, 1): the dump's alone",
        ],
        "run146's numbering moved"
    );
}

pub(super) fn chapter_three_s_arena_a_hoplites_take_the_dump_s_numbers() {
    let Some(mut s) = stage_script("ch3b", "chapter3b") else {
        return;
    };
    let (mut ours, mut theirs) = (Vec::new(), Vec::new());
    for f in 0..793 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < 790 {
            continue;
        }
        let at = s.ix.frames().iter().position(|x| x.number == n).unwrap();
        let frame = s.ix.frame_state(at).unwrap();
        for o in 9..=11 {
            if let Some(u) = frame.units.iter().find(|u| u.who == 1 && u.o == o) {
                theirs.push((n, o, u.pos.x, u.pos.y, u.idle.unwrap_or(-1), {
                    let mut k: Vec<i64> = u.orders.iter().map(|d| d.index).collect();
                    k.sort_unstable();
                    k
                }));
            }
            let sim = &s.built.sim;
            if let Some(i) = sim.unit_by_o(1, o as i16) {
                let u = &sim.units[i];
                ours.push((
                    n,
                    o,
                    i64::from(u.pos.x),
                    i64::from(u.pos.y),
                    i64::from(u.idle),
                    {
                        let mut k: Vec<i64> =
                            u.orders.iter().map(|d| i64::from(d.index())).collect();
                        k.sort_unstable();
                        k
                    },
                ));
            }
        }
    }
    for r in &theirs {
        eprintln!("  theirs {r:?}");
    }
    assert_eq!(theirs.len(), 12, "the dump's arena-A hoplites");
    assert_eq!(ours, theirs, "arena A's hoplites on 790-793");
}

pub(super) fn chapter_three_s_falsifiers_are_the_dump_s() {
    use std::collections::BTreeMap;
    let alive = |u: &crate::gamelog::UnitDump| u.myhits.is_some_and(|h| h > 1);
    for run in ["ch3", "ch3b"] {
        let Some((dump, _)) = golden(run) else {
            eprintln!("skipping: no golden capture {run} (docs/RUNS.md run145, run146)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
        let at = ix.frames().iter().position(|x| x.number == 611).unwrap();
        let f = ix.frame_state(at).unwrap();
        let chariots: Vec<_> = f
            .units
            .iter()
            .filter(|u| u.who == 0 && u.guys.first().and_then(|g| g.kind) == Some(195))
            .map(|u| (u.o, u.o_up, u.o_down))
            .collect();
        assert_eq!(
            chariots.len(),
            3,
            "{run}: `add 3 chariot` is not three units"
        );
        assert!(
            chariots.iter().all(|c| c.1 == Some(-1) && c.2 == Some(-1)),
            "{run}: the chariots are threaded as a squad: {chariots:?}"
        );
    }
    let Some((dump, _)) = golden("ch3b") else {
        return;
    };
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let d2 =
        |a: &crate::gamelog::Pos, b: &crate::gamelog::Pos| (a.x - b.x).pow(2) + (a.y - b.y).pow(2);
    const DEAD_ZONE: i64 = 3 * 192 - 6;
    let (mut launches, mut close_launches, mut inside) = (0usize, 0usize, 0usize);
    let mut last: BTreeMap<(i64, i64), crate::gamelog::Pos> = BTreeMap::new();
    let mut steps: BTreeMap<(i64, i64), (usize, i64)> = BTreeMap::new();
    for at in 0..ix.frames().len() {
        let n = ix.frames()[at].number;
        let f = ix.frame_state(at).unwrap();
        let raw = ix.read_frame(at).unwrap();
        let live: BTreeMap<(i64, i64), &crate::gamelog::UnitDump> = f
            .units
            .iter()
            .filter(|u| alive(u))
            .map(|u| ((u.who, u.o), u))
            .collect();
        let nearest = live.get(&(0, 6)).and_then(|c| {
            (9..=11)
                .filter_map(|o| live.get(&(1, o)))
                .map(|h| d2(&c.pos, &h.pos))
                .min()
        });
        if nearest.is_some_and(|d| d < DEAD_ZONE * DEAD_ZONE) {
            inside += 1;
        }
        for (a, _) in super::ammo::blocks(&raw) {
            if (a.who, a.o, a.cur_time) == (0, 6, 1) {
                launches += 1;
                if nearest.is_none_or(|d| d < DEAD_ZONE * DEAD_ZONE) {
                    close_launches += 1;
                }
            }
        }
        for (k, u) in &live {
            if let Some(p) = last.get(k) {
                let s = d2(p, &u.pos);
                if s > 0 {
                    let e = steps.entry(*k).or_insert((0, 0));
                    e.0 += 1;
                    e.1 = e.1.max(s);
                }
            }
        }
        last = live.iter().map(|(k, u)| (*k, u.pos)).collect();
        let _ = n;
    }
    eprintln!(
        "run146: {launches} catapult launch(es), {close_launches} inside the dead zone, \
         {inside} blocks a hoplite stands inside it; steps {steps:?}"
    );
    assert_eq!(
        (launches, close_launches, inside),
        (1, 0, 173),
        "§7's minimum-range falsifier: the catapult's launches, those inside \
         three tiles, and the blocks that make the refusal a measurement"
    );
    let chariot = (7..=9)
        .filter_map(|o| steps.get(&(0, o)))
        .filter(|s| s.0 >= 3)
        .map(|s| s.1)
        .max()
        .expect("no chariot walks three blocks in run146");
    let hoplite = (6..=8)
        .filter_map(|o| steps.get(&(1, o)))
        .map(|s| s.1)
        .max()
        .expect("arena B's hoplites never walk");
    assert!(
        chariot > hoplite,
        "§7's speed falsifier fires: a chariot's longest step² {chariot} is not \
         longer than a hoplite's {hoplite}"
    );
}

pub(super) fn chapter_three_s_catapult_after_its_reload() {
    use sim::orders::Body;
    type Row = (Vec<[i64; 6]>, i64, i64, (i64, i64), (i64, i64));
    let Some(mut s) = stage_script("ch3b", "chapter3b") else {
        return;
    };
    let mut parted = Vec::new();
    let mut theirs_shape = Vec::new();
    for f in 0..880 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < 858 {
            continue;
        }
        let ix =
            s.ix.frames()
                .iter()
                .position(|x| x.number == n)
                .expect("the block");
        let frame = s.ix.frame_state(ix).unwrap();
        let them = frame
            .units
            .iter()
            .find(|u| u.who == 0 && u.o == 6)
            .expect("the catapult in the dump");
        let theirs: Row = (
            them.orders_front_first()
                .map(|o| match o.ag_att_x {
                    Some(x) => [o.index, x, o.ag_att_y.unwrap_or(-1), -1, -1, o.flags],
                    None => [
                        o.index,
                        o.whom.unwrap_or(-1),
                        o.ox.unwrap_or(-1),
                        o.in_range.unwrap_or(-1),
                        o.new_ord.unwrap_or(-1),
                        o.flags,
                    ],
                })
                .collect(),
            them.idle.expect("idle"),
            them.recharging.expect("recharging"),
            (them.pos.x, them.pos.y),
            (
                them.orders_x.expect("orders_x"),
                them.orders_y.expect("orders_y"),
            ),
        );
        let sim = &s.built.sim;
        let u = &sim.units[sim.unit_by_o(0, 6).expect("our catapult")];
        let (tw, to) = match u.combat.target {
            Some(sim::combat::Obj::Unit(t)) => {
                (i64::from(sim.units[t].owner), i64::from(sim.units[t].index))
            }
            _ => (-1, -1),
        };
        let ours: Row = (
            u.orders
                .iter()
                .map(|o| {
                    let flags = i64::from(o.flags as i8);
                    match o.body {
                        Body::AttackGround(g) => [
                            i64::from(o.index()),
                            i64::from(g.at.x),
                            i64::from(g.at.y),
                            -1,
                            -1,
                            flags,
                        ],
                        Body::Attack(a) => [
                            i64::from(o.index()),
                            tw,
                            to,
                            i64::from(a.in_range),
                            i64::from(a.new_ord),
                            flags,
                        ],
                        _ => [i64::from(o.index()), -1, -1, -1, -1, flags],
                    }
                })
                .collect(),
            i64::from(u.idle),
            i64::from(u.combat.recharging),
            (i64::from(u.pos.x), i64::from(u.pos.y)),
            (i64::from(u.orders_pos.x), i64::from(u.orders_pos.y)),
        );
        for o in 9..=11i16 {
            let t = frame
                .units
                .iter()
                .find(|x| x.who == 1 && i64::from(o) == x.o)
                .expect("a hoplite in the dump");
            let k = sim.unit_by_o(1, o).expect("our hoplite");
            let p = sim.units[k].pos;
            if (i64::from(p.x), i64::from(p.y)) != (t.pos.x, t.pos.y) {
                parted.push(format!(
                    "{n} 1/{o} at ({}, {}) against ({}, {})",
                    p.x, p.y, t.pos.x, t.pos.y
                ));
            }
        }
        if ours != theirs {
            parted.push(format!("{n} 0/6 ours {ours:?} theirs {theirs:?}"));
        }
        theirs_shape.push((
            n,
            theirs
                .0
                .iter()
                .map(|r| (r[0], r[1], r[2]))
                .collect::<Vec<_>>(),
        ));
    }
    // The dump's own shape, pinned so the value diff above cannot agree
    // with a different one: the ground order over the attack to 863,
    // nothing 864–867, one attack on the hitter the block after each hit.
    const ATTACK: i64 = sim::orders::index::ATTACK as i64;
    let attacks: Vec<_> = theirs_shape
        .iter()
        .filter(|(n, v)| *n >= 864 && !v.is_empty())
        .cloned()
        .collect();
    assert_eq!(
        attacks,
        vec![
            (868, vec![(ATTACK, 1, 9)]),
            (870, vec![(ATTACK, 1, 11)]),
            (871, vec![(ATTACK, 1, 10)]),
        ],
        "the dump's catapult after its reload"
    );
    assert!(
        parted.is_empty(),
        "run146's catapult after its reload: {parted:#?}"
    );
}

pub(super) fn chapter_seven_b_s_control_scout_search_is_priced_as_the_original() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("ch7bc") else {
        eprintln!("skipping: no golden capture ch7bc (see docs/RUNS.md)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = script_named("chapter7b_control");
    const SEARCH: i64 = 1076;
    while built.sim.frame < SEARCH {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    script.stage(built.sim.frame, &mut built, &loaded);
    built.sim.trace_costs = true;
    built.tick();
    let ours = std::mem::take(&mut built.sim.cost_marks);
    let theirs = trace.calls_in(SEARCH, crate::trace::call_site::CALC_COST);
    let theirs_by_key: std::collections::BTreeMap<sim::path::CostKey, i32> = theirs
        .iter()
        .map(|c| (c.cost_key().expect("a calc_cost call"), c.ret))
        .collect();
    let mut shared = 0usize;
    let mut wrong: Vec<String> = Vec::new();
    for m in &ours {
        let Some(&t) = theirs_by_key.get(&m.key()) else {
            continue;
        };
        shared += 1;
        if t != m.cost {
            wrong.push(format!(
                "({},{})->({},{}) depth {}: ours {} theirs {t}",
                m.from.0 / m.step,
                m.from.1 / m.step,
                m.to.0 / m.step,
                m.to.1 / m.step,
                m.depth,
                m.cost,
            ));
        }
    }
    eprintln!(
        "run157 f{SEARCH}: ours {} steps, theirs {}, {shared} shared, {} priced apart",
        ours.len(),
        theirs.len(),
        wrong.len()
    );
    for w in &wrong {
        eprintln!("  {w}");
    }
    // **The floor, before the fix** (item 647, `33952d3`): 969 steps here
    // against 945, 587 shared, and nine priced apart, every one into cell
    // (48,31) or (47,31), the south half of the Small City `1/2007`'s
    // footprint (started on 1069, `mylos 0`). The original priced them
    // seen — 128 base, 20 × 9 of blocked tiles, − 4 own ground, + 8 on a
    // diagonal, so 304 and 312 — and this crate as unseen scouting ground,
    // 1 and 9 (`docs/PATHFINDER.md` §5). `Wall::start`'s write of the
    // owner's bit over the footprint is what lights them (`docs/SCOUT.md`
    // §14): with it the two searches are one, step for step.
    assert_eq!(
        (ours.len(), theirs.len(), shared),
        (945, 945, 945),
        "run157 f{SEARCH}: the search's steps moved"
    );
    assert_eq!(
        wrong,
        Vec::<String>::new(),
        "run157 f{SEARCH}: steps priced apart"
    );
}
