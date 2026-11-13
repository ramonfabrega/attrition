//! **The corpus floor's instrument** (item 4216, stage 0; DECISIONS 132
//! §11): every scored game stood up and walked ONCE, in one process, on a
//! pool of std threads, with the same stand-up and walk the floor tests use
//! (`third::stand_up_from` + `third::walk_word`, the melees under their
//! staged script, the second pair through `second::walk_second`). It
//! prints per-word frames and seconds and the pool's totals, and it is the
//! number the suite's form is read against: what the scoreboard's walks
//! cost when each game is walked once, beside what the tests on those
//! games cost one process a test. Ignored: it is a measurement, and every
//! word it walks has its own floor test.
//!
//! `CORPUS_FLOOR_THREADS` sets N (default cores - 2);
//! `CORPUS_FLOOR_ONLY=<family,...>` narrows the families;
//! `CORPUS_FLOOR_SPLIT=1` additionally times the original's side (trace
//! parse, dump read+parse) and the install load per word, outside the
//! measured walk.
//!
//! Written as a spike by the second pass of 2026-11-13
//! (`docs/audit/2026-11-13-fable-pass-supply-2.md`, "One walk a game").
//! Not walked: the first pair's three 1,850-frame words and the two
//! 24,000-frame long captures, which are the harness's own replay.

use super::battery::Lobby;
use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone, Copy)]
enum How {
    From,
    Staged(fn() -> crate::golden::Script),
    Second(bool),
}

#[derive(Clone)]
struct Entry {
    family: &'static str,
    name: String,
    start: &'static str,
    long: (&'static str, &'static str),
    length: i64,
    count: i64,
    sequence: i64,
    how: How,
    /// The floor test that walks this word: the worker thread takes its
    /// name, as the ground guard keys its pin on it.
    test: &'static str,
}

impl Entry {
    /// Frames the floor test ticks: it stops where the count parts.
    fn frames(&self) -> i64 {
        self.count.min(self.length)
    }
}

struct Done {
    name: String,
    family: &'static str,
    frames: i64,
    last: i64,
    stand: f64,
    walk: f64,
    split: Option<(f64, f64, f64)>,
    ok: Result<(), String>,
}

/// A word's constant beside the floor test that walks it, which every
/// pair and word module names `<module>_first_parting`.
macro_rules! floor {
    ($module:ident, $word:ident) => {
        (
            &crate::diff::$module::$word,
            concat!(stringify!($module), "_first_parting"),
        )
    };
}

const PAIRS: &[(&Pair, &str)] = &[
    floor!(american_enlightenment_pair, AMERICAN_ENLIGHTENMENT),
    floor!(american_industrial_pair, AMERICAN_INDUSTRIAL),
    floor!(american_information_pair, AMERICAN_INFORMATION),
    floor!(american_modern_pair, AMERICAN_MODERN),
    floor!(aztec_alltech_pair, AZTEC_ALLTECH),
    floor!(aztec_ancient_pair, AZTEC_ANCIENT),
    floor!(aztec_medieval_pair, AZTEC_MEDIEVAL),
    floor!(aztec_modern_pair, AZTEC_MODERN),
    floor!(bantu_ancient_pair, BANTU_ANCIENT),
    floor!(bantu_medieval_pair, BANTU_MEDIEVAL),
    floor!(bantu_modern_pair, BANTU_MODERN),
    floor!(british_classical_pair, BRITISH_CLASSICAL),
    floor!(british_enlightenment_pair, BRITISH_ENLIGHTENMENT),
    floor!(british_industrial_pair, BRITISH_INDUSTRIAL),
    floor!(british_medieval_pair, BRITISH_MEDIEVAL),
    floor!(british_modern_pair, BRITISH_MODERN),
    floor!(chinese_enlightenment_pair, CHINESE_ENLIGHTENMENT),
    floor!(chinese_gunpowder_pair, CHINESE_GUNPOWDER),
    floor!(chinese_industrial_pair, CHINESE_INDUSTRIAL),
    floor!(chinese_medieval_pair, CHINESE_MEDIEVAL),
    floor!(chinese_modern_pair, CHINESE_MODERN),
    floor!(coverage_pair, COVERAGE),
    floor!(dutch_medieval_pair, DUTCH_MEDIEVAL),
    floor!(egyptian_classical_pair, EGYPTIAN_CLASSICAL),
    floor!(egyptian_gunpowder_pair, EGYPTIAN_GUNPOWDER),
    floor!(egyptian_industrial_pair, EGYPTIAN_INDUSTRIAL),
    floor!(egyptian_information_pair, EGYPTIAN_INFORMATION),
    floor!(egyptian_modern_pair, EGYPTIAN_MODERN),
    floor!(french_classical_pair, FRENCH_CLASSICAL),
    floor!(french_enlightenment_pair, FRENCH_ENLIGHTENMENT),
    floor!(french_information_pair, FRENCH_INFORMATION),
    floor!(french_medieval_pair, FRENCH_MEDIEVAL),
    floor!(german_alltech_pair, GERMAN_ALLTECH),
    floor!(german_ancient_pair, GERMAN_ANCIENT),
    floor!(german_industrial_moderate_pair, GERMAN_INDUSTRIAL_MODERATE),
    floor!(german_industrial_pair, GERMAN_INDUSTRIAL),
    floor!(german_modern_pair, GERMAN_MODERN),
    floor!(greek_classical_pair, GREEK_CLASSICAL),
    floor!(greek_enlightenment_pair, GREEK_ENLIGHTENMENT),
    floor!(greek_gunpowder_pair, GREEK_GUNPOWDER),
    floor!(greek_medieval_pair, GREEK_MEDIEVAL),
    floor!(inca_classical_pair, INCA_CLASSICAL),
    floor!(inca_enlightenment_pair, INCA_ENLIGHTENMENT),
    floor!(inca_industrial_pair, INCA_INDUSTRIAL),
    floor!(indian_enlightenment_pair, INDIAN_ENLIGHTENMENT),
    floor!(iroquois_ancient_pair, IROQUOIS_ANCIENT),
    floor!(iroquois_enlightenment_pair, IROQUOIS_ENLIGHTENMENT),
    floor!(japanese_ancient_pair, JAPANESE_ANCIENT),
    floor!(japanese_enlightenment_pair, JAPANESE_ENLIGHTENMENT),
    floor!(japanese_gunpowder_pair, JAPANESE_GUNPOWDER),
    floor!(japanese_industrial_pair, JAPANESE_INDUSTRIAL),
    floor!(japanese_medieval_pair, JAPANESE_MEDIEVAL),
    floor!(japanese_modern_pair, JAPANESE_MODERN),
    floor!(korean_ancient_pair, KOREAN_ANCIENT),
    floor!(korean_classical_pair, KOREAN_CLASSICAL),
    floor!(korean_medieval_pair, KOREAN_MEDIEVAL),
    floor!(lakota_enlightenment_pair, LAKOTA_ENLIGHTENMENT),
    floor!(lakota_gunpowder_pair, LAKOTA_GUNPOWDER),
    floor!(lakota_medieval_pair, LAKOTA_MEDIEVAL),
    floor!(lakota_pair, LAKOTA),
    floor!(maya_ancient_pair, MAYA_ANCIENT),
    floor!(maya_information_pair, MAYA_INFORMATION),
    floor!(mongol_classical_pair, MONGOL_CLASSICAL),
    floor!(mongol_enlightenment_pair, MONGOL_ENLIGHTENMENT),
    floor!(mongol_gunpowder_pair, MONGOL_GUNPOWDER),
    floor!(mongol_medieval_pair, MONGOL_MEDIEVAL),
    floor!(mongol_pair, MONGOL),
    floor!(persian_classical_pair, PERSIAN_CLASSICAL),
    floor!(persian_gunpowder_pair, PERSIAN_GUNPOWDER),
    floor!(persian_medieval_pair, PERSIAN_MEDIEVAL),
    floor!(roman_classical_pair, ROMAN_CLASSICAL),
    floor!(roman_gunpowder_pair, ROMAN_GUNPOWDER),
    floor!(roman_medieval_pair, ROMAN_MEDIEVAL),
    floor!(roman_modern_pair, ROMAN_MODERN),
    floor!(russian_gunpowder_pair, RUSSIAN_GUNPOWDER),
    floor!(russian_information_pair, RUSSIAN_INFORMATION),
    floor!(russian_medieval_pair, RUSSIAN_MEDIEVAL),
    floor!(russian_modern_pair, RUSSIAN_MODERN),
    floor!(sahara_coverage, SAHARA_COVERAGE),
    floor!(sahara_mongol, SAHARA_MONGOL),
    floor!(spanish_classical_pair, SPANISH_CLASSICAL),
    floor!(spanish_gunpowder_pair, SPANISH_GUNPOWDER),
    floor!(spanish_industrial_pair, SPANISH_INDUSTRIAL),
    floor!(spanish_information_pair, SPANISH_INFORMATION),
    floor!(spanish_modern_pair, SPANISH_MODERN),
    floor!(turkish_enlightenment_pair, TURKISH_ENLIGHTENMENT),
    floor!(turkish_information_pair, TURKISH_INFORMATION),
];

const WORDS: &[(&Lobby, &str)] = &[
    floor!(american_lakes_word, AMERICAN_LAKES),
    floor!(american_word, AMERICAN),
    floor!(aztec_word, AZTEC),
    floor!(bantu_word, BANTU),
    floor!(dutch_sahara_word, DUTCH_SAHARA),
    floor!(dutch_word, DUTCH),
    floor!(egyptian_word, EGYPTIAN),
    floor!(french_word, FRENCH),
    floor!(german_sahara_word, GERMAN_SAHARA),
    floor!(german_word, GERMAN),
    floor!(greek_word, GREEK),
    floor!(inca_sahara_word, INCA_SAHARA),
    floor!(inca_word, INCA),
    floor!(indian_sahara_word, INDIAN_SAHARA),
    floor!(indian_word, INDIAN),
    floor!(iroquois_sahara_word, IROQUOIS_SAHARA),
    floor!(iroquois_word, IROQUOIS),
    floor!(japanese_word, JAPANESE),
    floor!(korean_word, KOREAN),
    floor!(lakota_word, LAKOTA),
    floor!(maya_word, MAYA),
    floor!(persian_word, PERSIAN),
    floor!(roman_word, ROMAN),
    floor!(russian_word, RUSSIAN),
    floor!(spanish_east_word, SPANISH_EAST),
    floor!(spanish_lakes_word, SPANISH_LAKES),
    floor!(spanish_word, SPANISH),
    floor!(turkish_word, TURKISH),
];

/// One melee's row: its floor test, its game, and what its walk reads.
type MeleeRow = (
    &'static str,
    &'static crate::diff::gunpowder_melee::Melee,
    &'static str,
    (&'static str, &'static str),
    fn() -> crate::golden::Script,
    &'static str,
);

const MELEES: &[MeleeRow] = &[
    (
        "GreatSaharaGunpowderMelee",
        &crate::diff::gunpowder_melee::GUNPOWDER_MELEE,
        crate::diff::gunpowder_melee::MELEE_START,
        crate::diff::gunpowder_melee::MELEE_LONG,
        crate::diff::gunpowder_melee::melee_script,
        "gunpowder_melee_first_parting",
    ),
    (
        "GreatSaharaClassicalMelee",
        &crate::diff::classical_melee::CLASSICAL_MELEE,
        crate::diff::classical_melee::MELEE_START,
        crate::diff::classical_melee::MELEE_LONG,
        crate::diff::classical_melee::melee_script,
        "classical_melee_first_parting",
    ),
    (
        "GreatSaharaInformationMelee",
        &crate::diff::information_melee::INFORMATION_MELEE,
        crate::diff::information_melee::MELEE_START,
        crate::diff::information_melee::MELEE_LONG,
        crate::diff::information_melee::melee_script,
        "information_melee_first_parting",
    ),
    (
        "EastIndiesInformationMelee",
        &crate::diff::sea_melee::SEA_MELEE,
        crate::diff::sea_melee::MELEE_START,
        crate::diff::sea_melee::MELEE_LONG,
        crate::diff::sea_melee::melee_script,
        "sea_melee_first_parting",
    ),
    (
        "EastIndiesAirbaseMelee",
        &crate::diff::airbase_melee::AIRBASE_MELEE,
        crate::diff::airbase_melee::MELEE_START,
        crate::diff::airbase_melee::MELEE_LONG,
        crate::diff::airbase_melee::melee_script,
        "airbase_melee_first_parting",
    ),
    (
        "GreatSaharaCorpseMelee",
        &crate::diff::corpse_melee::CORPSE_MELEE,
        crate::diff::corpse_melee::MELEE_START,
        crate::diff::corpse_melee::MELEE_LONG,
        crate::diff::corpse_melee::melee_script,
        "corpse_melee_first_parting",
    ),
];

fn ai(map: &str) -> (i64, i64) {
    let w = AI_WORDS.iter().find(|w| w.map == map).expect(map);
    (w.length, w.word)
}

fn entry(
    family: &'static str,
    name: &str,
    start: &'static str,
    long: (&'static str, &'static str),
    (length, count, sequence): (i64, i64, i64),
    how: How,
    test: &'static str,
) -> Entry {
    Entry {
        family,
        name: name.to_string(),
        start,
        long,
        length,
        count,
        sequence,
        how,
        test,
    }
}

fn entries() -> Vec<Entry> {
    let mut v = Vec::new();
    for (l, t) in WORDS {
        let pins = (l.length, l.count, l.sequence);
        v.push(entry("word", l.name, l.start, l.long, pins, How::From, t));
    }
    for l in super::battery::BATTERY {
        let pins = (l.length, l.count, l.sequence);
        let t = "battery_floors_hold";
        v.push(entry(
            "battery",
            l.name,
            l.start,
            l.long,
            pins,
            How::From,
            t,
        ));
    }
    for (p, t) in PAIRS {
        let pins = (p.length, p.count, p.sequence);
        v.push(entry("pair", p.name, p.start, p.long, pins, How::From, t));
    }
    for (name, m, start, long, script, t) in MELEES {
        let pins = (m.length, m.count, m.sequence);
        v.push(entry(
            "melee",
            name,
            start,
            *long,
            pins,
            How::Staged(*script),
            t,
        ));
    }
    for (map, start, long, t) in [
        (
            "EastIndiesFrench",
            crate::diff::third_pair::EAST_START,
            crate::diff::third_pair::EAST_LONG,
            "run600_french_east_indies_word",
        ),
        (
            "GreatLakesFrench",
            crate::diff::third_pair::LAKES_START,
            crate::diff::third_pair::LAKES_LONG,
            "run598_french_great_lakes_word",
        ),
        (
            "GreatSahara",
            third::SAHARA_START,
            third::SAHARA_LONG,
            "run383_s_long_trace_says_where_the_third_map_s_word_parts",
        ),
        (
            "GreatSaharaToughest",
            crate::diff::sahara_toughest::TOUGHEST_START,
            crate::diff::sahara_toughest::TOUGHEST_LONG,
            "run470_is_great_sahara_at_toughest_and_its_word_holds",
        ),
    ] {
        let (length, word) = ai(map);
        let pins = (length, word, word);
        v.push(entry("third", map, start, long, pins, How::From, t));
    }
    for (map, long, east, t) in [
        (
            "EastIndies",
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            true,
            "run346_is_east_indies_at_toughest_and_its_word_holds",
        ),
        (
            "GreatLakes",
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            false,
            "run347_is_great_lakes_at_toughest_and_its_word_holds",
        ),
    ] {
        let (length, word) = ai(map);
        let name = format!("{map}Toughest");
        let pins = (length, word, word);
        v.push(entry("second", &name, "", long, pins, How::Second(east), t));
    }
    // One walk per game: a long capture named twice is walked once.
    let mut seen = std::collections::HashSet::new();
    v.retain(|e| seen.insert(e.long.0));
    if let Ok(only) = std::env::var("CORPUS_FLOOR_ONLY") {
        let fams: Vec<&str> = only.split(',').collect();
        v.retain(|e| fams.contains(&e.family));
    }
    v.sort_by_key(|e| std::cmp::Reverse(e.frames()));
    v
}

fn split_times(e: &Entry) -> (f64, f64, f64) {
    // The original's side, timed apart: the trace, and the long dump (and
    // the start sibling) read and parsed; and the install's load.
    let t = Instant::now();
    let _tr = trace(e.long.1);
    let tr_s = t.elapsed().as_secs_f64();
    let t = Instant::now();
    for name in [e.long.0, e.start] {
        if name.is_empty() {
            continue;
        }
        if let Some(p) = crate::testenv::dump(name) {
            let text = crate::capture::read(&p);
            let log = Log::parse(&text);
            let _ = log.initial();
        }
    }
    let dump_s = t.elapsed().as_secs_f64();
    let t = Instant::now();
    let _ = crate::load::load(&crate::testenv::install().unwrap());
    let load_s = t.elapsed().as_secs_f64();
    (tr_s, dump_s, load_s)
}

fn run_one(e: &Entry, split: bool) -> Done {
    let sp = split.then(|| split_times(e));
    let t0 = Instant::now();
    let (stand, walk, last, count, sequence) = match e.how {
        How::Second(east) => {
            let w = second::walk_second(e.long.0, e.long.1, east).expect("captures on disk");
            let s = t0.elapsed().as_secs_f64();
            (0.0, s, w.last, w.count, w.sequence)
        }
        How::From | How::Staged(_) => {
            let body = || {
                let (built, tr) = third::stand_up_from(e.start, e.long).expect("captures on disk");
                let stand = t0.elapsed().as_secs_f64();
                let t1 = Instant::now();
                let w = third::walk_word(built, &tr, e.long.0);
                let walk = t1.elapsed().as_secs_f64();
                (stand, walk, w.last, w.count, w.sequence)
            };
            match e.how {
                How::Staged(script) => crate::diff::setup::with_staged_script(script(), body),
                _ => body(),
            }
        }
    };
    let ok = if last != e.length {
        Err(format!("last {last} != pinned length {}", e.length))
    } else if count < e.count || sequence < e.sequence {
        Err(format!(
            "fell: count {count} (pin {}), sequence {sequence} (pin {})",
            e.count, e.sequence
        ))
    } else {
        Ok(())
    };
    Done {
        name: e.name.clone(),
        family: e.family,
        frames: count.min(last),
        last,
        stand,
        walk,
        split: sp,
        ok,
    }
}

fn panicked(e: &Entry, p: Box<dyn std::any::Any + Send>) -> Done {
    let msg = p
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default();
    Done {
        name: e.name.clone(),
        family: e.family,
        frames: 0,
        last: 0,
        stand: 0.0,
        walk: 0.0,
        split: None,
        ok: Err(format!("panicked: {msg}")),
    }
}

fn print(d: &Done) {
    let split = d.split.map_or(String::new(), |(a, b, c)| {
        format!(" | trace {a:.2}s dump {b:.2}s load {c:.2}s")
    });
    let ok = match &d.ok {
        Ok(()) => "ok".to_string(),
        Err(m) => format!("NOT OK: {m}"),
    };
    eprintln!(
        "WORD {:<8} {:<40} frames {:>6} last {:>6} stand {:>7.2}s walk {:>8.2}s{split} {ok}",
        d.family, d.name, d.frames, d.last, d.stand, d.walk
    );
}

#[test]
#[ignore = "the corpus floor: a measurement, not a check (item 4216)"]
fn every_word_walked_once() {
    if crate::testenv::install().is_none() {
        eprintln!("skipping: no install");
        return;
    }
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let n: usize = std::env::var("CORPUS_FLOOR_THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(cores.saturating_sub(2).max(1));
    let split = std::env::var_os("CORPUS_FLOOR_SPLIT").is_some();
    let list = Arc::new(entries());
    let mut fam = std::collections::BTreeMap::<&str, usize>::new();
    for e in list.iter() {
        *fam.entry(e.family).or_default() += 1;
    }
    eprintln!(
        "CORPUS words {} frames-to-walk {} threads {n} families {fam:?}",
        list.len(),
        list.iter().map(Entry::frames).sum::<i64>()
    );
    let next = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(Mutex::new(Vec::<Done>::new()));
    let t0 = Instant::now();
    let handles: Vec<_> = (0..n)
        .map(|_| {
            let (list, next, done) = (list.clone(), next.clone(), done.clone());
            std::thread::Builder::new()
                .stack_size(64 << 20)
                .spawn(move || {
                    loop {
                        let i = next.fetch_add(1, Ordering::SeqCst);
                        let Some(e) = list.get(i) else { break };
                        // The ground guard (`setup.rs`, `Drop for Built`) keys its
                        // pin on the thread's name, as libtest names it after the
                        // test: each word runs on a thread named after its floor test.
                        let tname = format!("corpus_floor::{}", e.test);
                        let e2 = e.clone();
                        let r = std::thread::Builder::new()
                            .name(tname)
                            .stack_size(64 << 20)
                            .spawn(move || run_one(&e2, split))
                            .unwrap()
                            .join();
                        let d = r.unwrap_or_else(|p| panicked(e, p));
                        print(&d);
                        done.lock().unwrap().push(d);
                    }
                })
                .unwrap()
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    let wall = t0.elapsed().as_secs_f64();
    let done = done.lock().unwrap();
    let frames: i64 = done.iter().map(|d| d.frames).sum();
    let stand: f64 = done.iter().map(|d| d.stand).sum();
    let walk: f64 = done.iter().map(|d| d.walk).sum();
    let bad: Vec<&Done> = done.iter().filter(|d| d.ok.is_err()).collect();
    eprintln!(
        "CORPUS TOTAL words {} frames {frames} summed {:.1}s (stand {stand:.1}s walk {walk:.1}s) \
         pool wall {wall:.1}s threads {n} not-ok {}",
        done.len(),
        stand + walk,
        bad.len()
    );
    if split {
        let (a, b, c) = done
            .iter()
            .filter_map(|d| d.split)
            .fold((0.0, 0.0, 0.0), |s, x| (s.0 + x.0, s.1 + x.1, s.2 + x.2));
        eprintln!("CORPUS SPLIT trace {a:.1}s dump {b:.1}s load {c:.1}s");
    }
    for d in &bad {
        eprintln!("NOT OK {} {}: {:?} last {}", d.family, d.name, d.ok, d.last);
    }
    // A measurement of walks that fell short of their pins is no floor.
    // Seen red with one melee's pinned length raised by a frame.
    assert!(
        bad.is_empty(),
        "{} of {} words did not reach their pins",
        bad.len(),
        done.len()
    );
}
