//! **SPIKE, not for landing** (pass 1113b): every scored game stood up and
//! walked ONCE, in one process, on a pool of std threads, with the same
//! stand-up and walk the floor tests use (`third::stand_up_from` +
//! `third::walk_word`, the melees under their staged script, the second
//! pair through `second::walk_second`). Prints per-word frames and
//! seconds and the pool's totals. `CORPUS_FLOOR_THREADS` sets N (default
//! cores - 2); `CORPUS_FLOOR_ONLY=<family,...>` narrows the families;
//! `CORPUS_FLOOR_SPLIT=1` additionally times the original's side (trace
//! parse, dump read+parse) and the install load per word, outside the
//! measured walk.

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

const PAIRS: &[&Pair] = &[
    &crate::diff::american_enlightenment_pair::AMERICAN_ENLIGHTENMENT,
    &crate::diff::american_industrial_pair::AMERICAN_INDUSTRIAL,
    &crate::diff::american_information_pair::AMERICAN_INFORMATION,
    &crate::diff::american_modern_pair::AMERICAN_MODERN,
    &crate::diff::aztec_alltech_pair::AZTEC_ALLTECH,
    &crate::diff::aztec_ancient_pair::AZTEC_ANCIENT,
    &crate::diff::aztec_medieval_pair::AZTEC_MEDIEVAL,
    &crate::diff::aztec_modern_pair::AZTEC_MODERN,
    &crate::diff::bantu_ancient_pair::BANTU_ANCIENT,
    &crate::diff::bantu_medieval_pair::BANTU_MEDIEVAL,
    &crate::diff::bantu_modern_pair::BANTU_MODERN,
    &crate::diff::british_classical_pair::BRITISH_CLASSICAL,
    &crate::diff::british_enlightenment_pair::BRITISH_ENLIGHTENMENT,
    &crate::diff::british_industrial_pair::BRITISH_INDUSTRIAL,
    &crate::diff::british_medieval_pair::BRITISH_MEDIEVAL,
    &crate::diff::british_modern_pair::BRITISH_MODERN,
    &crate::diff::chinese_enlightenment_pair::CHINESE_ENLIGHTENMENT,
    &crate::diff::chinese_gunpowder_pair::CHINESE_GUNPOWDER,
    &crate::diff::chinese_industrial_pair::CHINESE_INDUSTRIAL,
    &crate::diff::chinese_medieval_pair::CHINESE_MEDIEVAL,
    &crate::diff::chinese_modern_pair::CHINESE_MODERN,
    &crate::diff::coverage_pair::COVERAGE,
    &crate::diff::dutch_medieval_pair::DUTCH_MEDIEVAL,
    &crate::diff::egyptian_classical_pair::EGYPTIAN_CLASSICAL,
    &crate::diff::egyptian_gunpowder_pair::EGYPTIAN_GUNPOWDER,
    &crate::diff::egyptian_industrial_pair::EGYPTIAN_INDUSTRIAL,
    &crate::diff::egyptian_information_pair::EGYPTIAN_INFORMATION,
    &crate::diff::egyptian_modern_pair::EGYPTIAN_MODERN,
    &crate::diff::french_classical_pair::FRENCH_CLASSICAL,
    &crate::diff::french_enlightenment_pair::FRENCH_ENLIGHTENMENT,
    &crate::diff::french_information_pair::FRENCH_INFORMATION,
    &crate::diff::french_medieval_pair::FRENCH_MEDIEVAL,
    &crate::diff::german_alltech_pair::GERMAN_ALLTECH,
    &crate::diff::german_ancient_pair::GERMAN_ANCIENT,
    &crate::diff::german_industrial_moderate_pair::GERMAN_INDUSTRIAL_MODERATE,
    &crate::diff::german_industrial_pair::GERMAN_INDUSTRIAL,
    &crate::diff::german_modern_pair::GERMAN_MODERN,
    &crate::diff::greek_classical_pair::GREEK_CLASSICAL,
    &crate::diff::greek_enlightenment_pair::GREEK_ENLIGHTENMENT,
    &crate::diff::greek_gunpowder_pair::GREEK_GUNPOWDER,
    &crate::diff::greek_medieval_pair::GREEK_MEDIEVAL,
    &crate::diff::inca_classical_pair::INCA_CLASSICAL,
    &crate::diff::inca_enlightenment_pair::INCA_ENLIGHTENMENT,
    &crate::diff::inca_industrial_pair::INCA_INDUSTRIAL,
    &crate::diff::indian_enlightenment_pair::INDIAN_ENLIGHTENMENT,
    &crate::diff::iroquois_ancient_pair::IROQUOIS_ANCIENT,
    &crate::diff::iroquois_enlightenment_pair::IROQUOIS_ENLIGHTENMENT,
    &crate::diff::japanese_ancient_pair::JAPANESE_ANCIENT,
    &crate::diff::japanese_enlightenment_pair::JAPANESE_ENLIGHTENMENT,
    &crate::diff::japanese_gunpowder_pair::JAPANESE_GUNPOWDER,
    &crate::diff::japanese_industrial_pair::JAPANESE_INDUSTRIAL,
    &crate::diff::japanese_medieval_pair::JAPANESE_MEDIEVAL,
    &crate::diff::japanese_modern_pair::JAPANESE_MODERN,
    &crate::diff::korean_ancient_pair::KOREAN_ANCIENT,
    &crate::diff::korean_classical_pair::KOREAN_CLASSICAL,
    &crate::diff::korean_medieval_pair::KOREAN_MEDIEVAL,
    &crate::diff::lakota_enlightenment_pair::LAKOTA_ENLIGHTENMENT,
    &crate::diff::lakota_gunpowder_pair::LAKOTA_GUNPOWDER,
    &crate::diff::lakota_medieval_pair::LAKOTA_MEDIEVAL,
    &crate::diff::lakota_pair::LAKOTA,
    &crate::diff::maya_ancient_pair::MAYA_ANCIENT,
    &crate::diff::maya_information_pair::MAYA_INFORMATION,
    &crate::diff::mongol_classical_pair::MONGOL_CLASSICAL,
    &crate::diff::mongol_enlightenment_pair::MONGOL_ENLIGHTENMENT,
    &crate::diff::mongol_gunpowder_pair::MONGOL_GUNPOWDER,
    &crate::diff::mongol_medieval_pair::MONGOL_MEDIEVAL,
    &crate::diff::mongol_pair::MONGOL,
    &crate::diff::persian_classical_pair::PERSIAN_CLASSICAL,
    &crate::diff::persian_gunpowder_pair::PERSIAN_GUNPOWDER,
    &crate::diff::persian_medieval_pair::PERSIAN_MEDIEVAL,
    &crate::diff::roman_classical_pair::ROMAN_CLASSICAL,
    &crate::diff::roman_gunpowder_pair::ROMAN_GUNPOWDER,
    &crate::diff::roman_medieval_pair::ROMAN_MEDIEVAL,
    &crate::diff::roman_modern_pair::ROMAN_MODERN,
    &crate::diff::russian_gunpowder_pair::RUSSIAN_GUNPOWDER,
    &crate::diff::russian_information_pair::RUSSIAN_INFORMATION,
    &crate::diff::russian_medieval_pair::RUSSIAN_MEDIEVAL,
    &crate::diff::russian_modern_pair::RUSSIAN_MODERN,
    &crate::diff::sahara_coverage::SAHARA_COVERAGE,
    &crate::diff::sahara_mongol::SAHARA_MONGOL,
    &crate::diff::spanish_classical_pair::SPANISH_CLASSICAL,
    &crate::diff::spanish_gunpowder_pair::SPANISH_GUNPOWDER,
    &crate::diff::spanish_industrial_pair::SPANISH_INDUSTRIAL,
    &crate::diff::spanish_information_pair::SPANISH_INFORMATION,
    &crate::diff::spanish_modern_pair::SPANISH_MODERN,
    &crate::diff::turkish_enlightenment_pair::TURKISH_ENLIGHTENMENT,
    &crate::diff::turkish_information_pair::TURKISH_INFORMATION,
];

const WORDS: &[&Lobby] = &[
    &crate::diff::american_lakes_word::AMERICAN_LAKES,
    &crate::diff::american_word::AMERICAN,
    &crate::diff::aztec_word::AZTEC,
    &crate::diff::bantu_word::BANTU,
    &crate::diff::dutch_sahara_word::DUTCH_SAHARA,
    &crate::diff::dutch_word::DUTCH,
    &crate::diff::egyptian_word::EGYPTIAN,
    &crate::diff::french_word::FRENCH,
    &crate::diff::german_sahara_word::GERMAN_SAHARA,
    &crate::diff::german_word::GERMAN,
    &crate::diff::greek_word::GREEK,
    &crate::diff::inca_sahara_word::INCA_SAHARA,
    &crate::diff::inca_word::INCA,
    &crate::diff::indian_sahara_word::INDIAN_SAHARA,
    &crate::diff::indian_word::INDIAN,
    &crate::diff::iroquois_sahara_word::IROQUOIS_SAHARA,
    &crate::diff::iroquois_word::IROQUOIS,
    &crate::diff::japanese_word::JAPANESE,
    &crate::diff::korean_word::KOREAN,
    &crate::diff::lakota_word::LAKOTA,
    &crate::diff::maya_word::MAYA,
    &crate::diff::persian_word::PERSIAN,
    &crate::diff::roman_word::ROMAN,
    &crate::diff::russian_word::RUSSIAN,
    &crate::diff::spanish_east_word::SPANISH_EAST,
    &crate::diff::spanish_lakes_word::SPANISH_LAKES,
    &crate::diff::spanish_word::SPANISH,
    &crate::diff::turkish_word::TURKISH,
];

const MELEES: &[(
    &str,
    &crate::diff::gunpowder_melee::Melee,
    &str,
    (&str, &str),
    fn() -> crate::golden::Script,
)] = &[
    (
        "GreatSaharaGunpowderMelee",
        &crate::diff::gunpowder_melee::GUNPOWDER_MELEE,
        crate::diff::gunpowder_melee::MELEE_START,
        crate::diff::gunpowder_melee::MELEE_LONG,
        crate::diff::gunpowder_melee::melee_script,
    ),
    (
        "GreatSaharaClassicalMelee",
        &crate::diff::classical_melee::CLASSICAL_MELEE,
        crate::diff::classical_melee::MELEE_START,
        crate::diff::classical_melee::MELEE_LONG,
        crate::diff::classical_melee::melee_script,
    ),
    (
        "GreatSaharaInformationMelee",
        &crate::diff::information_melee::INFORMATION_MELEE,
        crate::diff::information_melee::MELEE_START,
        crate::diff::information_melee::MELEE_LONG,
        crate::diff::information_melee::melee_script,
    ),
    (
        "EastIndiesInformationMelee",
        &crate::diff::sea_melee::SEA_MELEE,
        crate::diff::sea_melee::MELEE_START,
        crate::diff::sea_melee::MELEE_LONG,
        crate::diff::sea_melee::melee_script,
    ),
    (
        "EastIndiesAirbaseMelee",
        &crate::diff::airbase_melee::AIRBASE_MELEE,
        crate::diff::airbase_melee::MELEE_START,
        crate::diff::airbase_melee::MELEE_LONG,
        crate::diff::airbase_melee::melee_script,
    ),
    (
        "GreatSaharaCorpseMelee",
        &crate::diff::corpse_melee::CORPSE_MELEE,
        crate::diff::corpse_melee::MELEE_START,
        crate::diff::corpse_melee::MELEE_LONG,
        crate::diff::corpse_melee::melee_script,
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
    }
}

fn entries() -> Vec<Entry> {
    let mut v = Vec::new();
    for l in WORDS {
        let pins = (l.length, l.count, l.sequence);
        v.push(entry("word", l.name, l.start, l.long, pins, How::From));
    }
    for l in super::battery::BATTERY {
        let pins = (l.length, l.count, l.sequence);
        v.push(entry("battery", l.name, l.start, l.long, pins, How::From));
    }
    for p in PAIRS {
        let pins = (p.length, p.count, p.sequence);
        v.push(entry("pair", p.name, p.start, p.long, pins, How::From));
    }
    for (name, m, start, long, script) in MELEES {
        let pins = (m.length, m.count, m.sequence);
        v.push(entry("melee", name, start, *long, pins, How::Staged(*script)));
    }
    for (map, start, long) in [
        (
            "EastIndiesFrench",
            crate::diff::third_pair::EAST_START,
            crate::diff::third_pair::EAST_LONG,
        ),
        (
            "GreatLakesFrench",
            crate::diff::third_pair::LAKES_START,
            crate::diff::third_pair::LAKES_LONG,
        ),
        ("GreatSahara", third::SAHARA_START, third::SAHARA_LONG),
        (
            "GreatSaharaToughest",
            crate::diff::sahara_toughest::TOUGHEST_START,
            crate::diff::sahara_toughest::TOUGHEST_LONG,
        ),
    ] {
        let (length, word) = ai(map);
        v.push(entry("third", map, start, long, (length, word, word), How::From));
    }
    for (map, long, east) in [
        (
            "EastIndies",
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            true,
        ),
        (
            "GreatLakes",
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            false,
        ),
    ] {
        let (length, word) = ai(map);
        let name = format!("{map}Toughest");
        v.push(entry("second", &name, "", long, (length, word, word), How::Second(east)));
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
#[ignore = "spike: a measurement, pass 1113b"]
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
                        let tname = match e.name.as_str() {
                            "EastIndiesAirbaseMelee" => "airbase_melee_first_parting".to_string(),
                            n => format!("corpus_floor::{n}"),
                        };
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
}
