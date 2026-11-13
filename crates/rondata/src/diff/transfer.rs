//! **The transfer harness** (item 4159's step T; `docs/DECISIONS.md` entry
//! 128 §3): a leaf of the leaders line that parts only beside a parent is
//! released when the crate's own step, run on the original's printed values
//! of its inputs on one record, gives the original's printed leaf on the
//! next — on every consecutive pair of records the kept dumps hold, at least
//! one of them a pair on which the leaf moves.
//!
//! No simulation runs here. Each kept dump that prints the level-9
//! `LEADERDATA` record is read once for the words the steps take (and, for
//! `supply_mark`, the units' own `supply` records), every pair of one
//! leader's consecutive records is formed, and three steps of `sim` are run
//! on the original's side alone:
//!
//! - **the six `*_queued` bins** (`docs/LEADERS.md` §14):
//!   [`sim::Sim::armed_queued_bins`] and [`sim::leaders::marks::step_bin`],
//!   driven by the record's own `num_queued` moves between the two records
//!   — `track_queued` steps `num_queued[t]` and the bins by the same `d` in
//!   one call, so the array's step is the bins' input, printed;
//! - **`supply_mark`** (§13): the crate's [`sim::territory::supply::SupplyList`]
//!   rebuilt from the supply records the first record's units hold, then
//!   given back each record a unit no longer holds and registered for each
//!   unit that newly holds one, its length against the printed mark;
//! - **the four averages** (§3): [`sim::leaders::rates::Rates::fold`] on the
//!   first record's averages, `frame_battle` and age, with the window's
//!   count the fold reads taken from the total's step across the pair (the
//!   fold adds that same count to `*_fifteen_seconds`, step 1); a pair with
//!   no fold between its records must hold the averages still.
//!
//! - **`territory_high`** (§20, item 4187): [`sim::territory::fold_territory_high`]
//!   on the first record's mark and the second's `territory` — the fold runs
//!   where `territory` is summed again, so a pair with no fold holds both;
//! - **`high_buildings`** (§20, item 4187): every building record whose
//!   `num_buildings` rose across the pair is an activation, its line count
//!   (`Sim::building_line`, the record and its `to` chain) read off the
//!   second record, and [`sim::cities::nations::raise_building_high`] raises
//!   its root's mark (`Sim::base_build_type`) — unless the pair's
//!   `cities_captured` rose (a capture's `activate(1, 1, 0)`); a pair over a
//!   gap that moves `num_buildings` has no step (the order is not printed).
//!
//! The pairs and the moving pairs are pinned per leaf ([`PINNED`]), so a
//! change that drops pairs is red too, and a bent step is seen red on a named
//! pair ([`the_bent_steps_part_on_a_named_pair`]).
//!
//! The extraction is cached outside the repository, beside the gates' own
//! caches (`~/Library/Caches/attrition/transfer-v1/`), keyed by each dump's
//! name, size and modification time: the original's own values, never ours,
//! and never in git. The held-out battery's dumps are not opened
//! (`docs/DECISIONS.md` entry 61 §6).

use super::testkit::initial;
use crate::diff::setup::build_sim;
use crate::testenv::install;
use sim::leaders::marks::{ARMED_QUEUED, step_bin};
use sim::leaders::rates::{FOLD_PERIOD, Rates};
use sim::territory::supply::SupplyList;
use sim::{Sim, Tuning};
use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

/// The cache's schema: bumped whenever [`extract`] reads a different set of
/// words, so a stale extraction is never read as the new one.
const SCHEMA: &str = "transfer-v3";

/// `BASE_UNITTYPES`: `num_queued[scan]`'s index 0 is `TypeIndex` 0, and the
/// unit records are `0x32..0x19e` (`TypeData::is_unit_type`).
const BASE_UNITTYPES: usize = 0x32;
const END_UNITTYPES: usize = 0x19e;

/// The record's words the steps read, beside the thirteen rates.
const WORDS: &[&str] = &[
    "territory",
    "territory_high",
    "cities_captured",
    "ages_get()",
    "leader_flags",
    "supply_mark",
    "barracks_queued",
    "stable_queued",
    "factory_queued",
    "combat_queued",
    "dock_queued",
    "air_queued",
];

/// The averages' leaves, in [`Rates::average`]'s kind order (death, kill,
/// hit, damage) with the record's own name and the total that feeds each.
const AVERAGES: [(&str, &str); 4] = [
    ("average_death_rate", "deaths_fifteen_seconds"),
    ("average_kill_rate", "kills_fifteen_seconds"),
    ("average_hit_rate", "hits_fifteen_seconds"),
    ("average_damage_rate", "damage_fifteen_seconds"),
];

/// The record's window counts, in [`Rates::current`]'s kind order.
const CURRENT: [&str; 4] = [
    "deaths_current_frame",
    "kills_current_frame",
    "hits_current_frame",
    "damage_current_frame",
];

/// Every word a record keeps: [`WORDS`], then the thirteen rates.
fn keys() -> impl Iterator<Item = &'static str> {
    WORDS
        .iter()
        .copied()
        .chain(sim::leaders::rates::RATE_LEAVES)
}

/// A word's slot in [`Rec::words`].
fn slot(k: &str) -> Option<usize> {
    keys().position(|x| x == k)
}

/// How many words a record keeps.
const KEPT: usize = 12 + 13;

/// A word the record did not print.
const MISSING: i64 = i64::MIN;

/// One leader's record as the harness reads it, kept small: the corpus
/// holds millions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Rec {
    /// The record's frame: the `FRAME` it is printed in, or the last frame
    /// plus one for the game's closing record (the state after the last
    /// frame's tick, `diff::gunpowder_melee`'s "to 2601").
    pub frame: i64,
    pub who: i64,
    /// By [`slot`], [`MISSING`] where not printed.
    pub words: [i64; KEPT],
    /// Whether the record printed `num_queued[scan]` at all.
    pub queued_printed: bool,
    /// `num_queued[scan]` by unit record (`TypeIndex − 0x32`), non-zero,
    /// in record order.
    pub queued: Vec<(u16, i32)>,
    /// The supply records this leader's units hold on the record's frame,
    /// slot → `(o, uid)`; `None` when the frame printed no unit.
    pub supply: Option<BTreeMap<i64, (i64, i64)>>,
    /// `num_buildings[scan]` and `high_buildings[scan]` by building
    /// record, non-zero, in record order; `None` when not printed.
    pub builds: Option<(Counts, Counts)>,
}

impl Default for Rec {
    fn default() -> Rec {
        Rec {
            frame: 0,
            who: -1,
            words: [MISSING; KEPT],
            queued_printed: false,
            queued: Vec::new(),
            supply: None,
            builds: None,
        }
    }
}

impl Rec {
    fn queued_of(&self, r: usize) -> i64 {
        self.queued
            .binary_search_by_key(&r, |q| usize::from(q.0))
            .map_or(0, |i| i64::from(self.queued[i].1))
    }
}

/// A per-building-record array read sparse: `(record, value)`, non-zero.
pub(crate) type Counts = Vec<(u8, u16)>;

/// A unit record being read: its indent, and its `who`, `o`, `uid` and
/// `supply` once seen.
type OpenUnit = (usize, Option<i64>, Option<i64>, Option<i64>, Option<i64>);

/// The kept dumps' directory, as `crate::testenv::dump` finds them.
fn logs_dir() -> PathBuf {
    let dir = std::env::var("RON_GAMELOG_DIR").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{home}/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs")
    });
    PathBuf::from(dir)
}

fn cache_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(format!("{home}/Library/Caches/attrition/{SCHEMA}"))
}

/// The scored kept dumps: every `gamelog-*.txt` of this tree's corpus
/// (`crate::corpus`: a dump another worktree filed and this tree's
/// `docs/RUNS.md` does not head is skipped and named) but the held-out
/// battery's.
fn corpus() -> Vec<String> {
    corpus_in(&logs_dir(), crate::corpus::Ledger::of_tree())
}

/// [`corpus`] of `dir` under `ledger`.
fn corpus_in(dir: &Path, ledger: &crate::corpus::Ledger) -> Vec<String> {
    let held: std::collections::BTreeSet<&str> = crate::diff::battery::BATTERY
        .iter()
        .flat_map(|l| [l.start, l.long.0])
        .collect();
    let Ok(listed) = crate::corpus::listed(dir, ledger, crate::corpus::is_gamelog) else {
        return Vec::new();
    };
    listed
        .kept
        .into_iter()
        .filter(|n| !held.contains(n.as_str()))
        .collect()
}

fn int(b: &[u8]) -> Option<i64> {
    std::str::from_utf8(b).ok()?.trim().parse().ok()
}

/// **Reads one dump's leader records**, every frame's first per leader (as
/// `Log::leader_block` takes the first), and its units' supply records.
pub(crate) fn extract(path: &Path) -> std::io::Result<Vec<Rec>> {
    let f = std::fs::File::open(path)?;
    let mut r = std::io::BufReader::with_capacity(1 << 20, f);
    let mut buf = Vec::with_capacity(256);
    let mut out: Vec<Rec> = Vec::new();
    // The frame being read: its label and the indent of its `BEGIN`.
    let mut frame: Option<(i64, usize)> = None;
    let mut last: Option<i64> = None;
    let mut closing = false;
    // This frame's records' start in `out`, the leaders seen, its units.
    let mut start = 0usize;
    let mut units_seen = false;
    let mut supply: BTreeMap<i64, BTreeMap<i64, (i64, i64)>> = BTreeMap::new();
    let mut seen_units: std::collections::BTreeSet<(i64, i64)> = Default::default();
    // The open leader record: its indent, the record, the scan index.
    let mut lead: Option<(usize, Rec, usize)> = None;
    // The open record's `num_buildings[scan]` and `high_buildings[scan]`
    // indices.
    let mut bscan = [0usize; 2];
    // The open units: indent, who, o, uid, supply.
    let mut units: Vec<OpenUnit> = Vec::new();

    let finish_frame =
        |out: &mut Vec<Rec>,
         start: usize,
         units_seen: bool,
         supply: &mut BTreeMap<i64, BTreeMap<i64, (i64, i64)>>| {
            for rec in &mut out[start..] {
                rec.supply = units_seen.then(|| supply.get(&rec.who).cloned().unwrap_or_default());
            }
            supply.clear();
        };

    loop {
        buf.clear();
        if r.read_until(b'\n', &mut buf)? == 0 {
            break;
        }
        let mut line: &[u8] = &buf;
        while let Some((&b'\n' | &b'\r', rest)) = line.split_last() {
            line = rest;
        }
        let ind = line.iter().take_while(|&&c| c == b' ').count();
        let body = &line[ind..];
        // Close what this line's indent ends.
        if let Some((li, _, _)) = &lead
            && ind <= *li
        {
            let (_, rec, _) = lead.take().unwrap();
            if !out[start..].iter().any(|x| x.who == rec.who) {
                out.push(rec);
            }
        }
        while let Some(u) = units.last()
            && ind <= u.0
        {
            let (_, who, o, uid, s) = units.pop().unwrap();
            if let (Some(who), Some(o)) = (who, o) {
                units_seen = true;
                if seen_units.insert((who, o))
                    && let Some(s) = s.filter(|s| *s >= 0)
                {
                    supply
                        .entry(who)
                        .or_default()
                        .insert(s, (o, uid.unwrap_or(-1)));
                }
            }
        }
        if let Some((_, fi)) = frame
            && ind <= fi
            && !body.starts_with(b"BEGIN FRAME ")
            && !closing
        {
            // Past the last frame: the closing records.
            finish_frame(&mut out, start, units_seen, &mut supply);
            start = out.len();
            units_seen = false;
            seen_units.clear();
            closing = true;
        }
        if let Some(rest) = body.strip_prefix(b"BEGIN ") {
            if let Some(n) = rest.strip_prefix(b"FRAME ").and_then(int) {
                finish_frame(&mut out, start, units_seen, &mut supply);
                start = out.len();
                units_seen = false;
                seen_units.clear();
                closing = false;
                frame = Some((n, ind));
                last = Some(n);
                continue;
            }
            let at = if closing {
                last.map(|l| l + 1)
            } else {
                frame.map(|f| f.0)
            };
            let Some(at) = at else { continue };
            match rest {
                b"LEADERDATA" => {
                    bscan = [0, 0];
                    lead = Some((
                        ind,
                        Rec {
                            frame: at,
                            who: -1,
                            ..Rec::default()
                        },
                        0,
                    ));
                }
                b"UNITDATA" => units.push((ind, None, None, None, None)),
                _ => {}
            }
            continue;
        }
        if let Some((li, rec, scan)) = &mut lead {
            if ind != *li + 1 {
                continue;
            }
            let (k, v) = match body.iter().position(|&c| c == b' ') {
                Some(p) => (&body[..p], &body[p + 1..]),
                None => (body, &b""[..]),
            };
            if let Some(which) = [&b"num_buildings[scan]"[..], &b"high_buildings[scan]"[..]]
                .iter()
                .position(|n| *n == k)
            {
                let i = bscan[which];
                bscan[which] += 1;
                let b = rec.builds.get_or_insert_with(Default::default);
                if i < 129
                    && let Some(x) = int(v).filter(|x| *x != 0)
                {
                    let list = if which == 0 { &mut b.0 } else { &mut b.1 };
                    list.push((i as u8, x as u16));
                }
                continue;
            }
            if k == b"num_queued[scan]" {
                rec.queued_printed = true;
                let i = *scan;
                *scan += 1;
                if (BASE_UNITTYPES..END_UNITTYPES).contains(&i)
                    && let Some(x) = int(v).filter(|x| *x != 0)
                {
                    rec.queued
                        .push(((i - BASE_UNITTYPES) as u16, i32::try_from(x).unwrap_or(0)));
                }
                continue;
            }
            let Ok(k) = std::str::from_utf8(k) else {
                continue;
            };
            if k == "who" && rec.who < 0 {
                rec.who = int(v).unwrap_or(-1);
            } else if let Some(i) = slot(k)
                && rec.words[i] == MISSING
                && let Some(x) = int(v)
            {
                rec.words[i] = x;
            }
            continue;
        }
        if let Some(u) = units.last_mut() {
            let (k, v) = match body.iter().position(|&c| c == b' ') {
                Some(p) => (&body[..p], &body[p + 1..]),
                None => (body, &b""[..]),
            };
            if ind == u.0 + 1 && k == b"supply" && u.4.is_none() {
                u.4 = int(v);
            } else if ind == u.0 + 2 && k == b"uid" && u.3.is_none() {
                u.3 = int(v);
            } else if ind == u.0 + 3 {
                match k {
                    b"who" if u.1.is_none() => u.1 = int(v),
                    b"o" if u.2.is_none() => u.2 = int(v),
                    _ => {}
                }
            }
        }
    }
    if let Some((_, rec, _)) = lead.take()
        && !out[start..].iter().any(|x| x.who == rec.who)
    {
        out.push(rec);
    }
    while let Some((_, who, o, uid, s)) = units.pop() {
        if let (Some(who), Some(o)) = (who, o) {
            units_seen = true;
            if seen_units.insert((who, o))
                && let Some(s) = s.filter(|s| *s >= 0)
            {
                supply
                    .entry(who)
                    .or_default()
                    .insert(s, (o, uid.unwrap_or(-1)));
            }
        }
    }
    finish_frame(&mut out, start, units_seen, &mut supply);
    // Only the level-9 record carries what the steps read.
    out.retain(|r| word(r, "supply_mark").is_some() || r.queued_printed || r.builds.is_some());
    Ok(out)
}

fn encode(recs: &[Rec]) -> String {
    let mut s = String::new();
    for r in recs {
        let w: Vec<String> = keys()
            .zip(r.words)
            .filter(|(_, v)| *v != MISSING)
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        let q: Vec<String> = r.queued.iter().map(|(k, v)| format!("{k}={v}")).collect();
        let u = match &r.supply {
            None => "-".to_string(),
            Some(m) => {
                let v: Vec<String> = m
                    .iter()
                    .map(|(s, (o, uid))| format!("{s}:{o}:{uid}"))
                    .collect();
                format!("+{}", v.join(","))
            }
        };
        let list = |l: &[(u8, u16)]| -> String {
            let v: Vec<String> = l.iter().map(|(k, v)| format!("{k}:{v}")).collect();
            format!("+{}", v.join(","))
        };
        let (bn, bh) = match &r.builds {
            None => ("-".to_string(), "-".to_string()),
            Some((n, h)) => (list(n), list(h)),
        };
        s.push_str(&format!(
            "{} {} {} {} {} {} {} {}\n",
            r.frame,
            r.who,
            u8::from(r.queued_printed),
            w.join(","),
            if q.is_empty() {
                "-".into()
            } else {
                q.join(",")
            },
            u,
            bn,
            bh
        ));
    }
    s
}

fn decode(text: &str) -> Option<Vec<Rec>> {
    let mut out = Vec::new();
    for line in text.lines() {
        let f: Vec<&str> = line.split(' ').collect();
        if f.len() != 8 {
            return None;
        }
        let mut rec = Rec {
            frame: f[0].parse().ok()?,
            who: f[1].parse().ok()?,
            queued_printed: f[2] == "1",
            ..Rec::default()
        };
        for kv in f[3].split(',').filter(|x| !x.is_empty()) {
            let (k, v) = kv.split_once('=')?;
            rec.words[slot(k)?] = v.parse().ok()?;
        }
        if f[4] != "-" {
            for kv in f[4].split(',') {
                let (k, v) = kv.split_once('=')?;
                rec.queued.push((k.parse().ok()?, v.parse().ok()?));
            }
        }
        if let Some(rest) = f[5].strip_prefix('+') {
            let mut m = BTreeMap::new();
            for t in rest.split(',').filter(|x| !x.is_empty()) {
                let mut p = t.split(':');
                let s = p.next()?.parse().ok()?;
                let o = p.next()?.parse().ok()?;
                let uid = p.next()?.parse().ok()?;
                m.insert(s, (o, uid));
            }
            rec.supply = Some(m);
        }
        let list = |t: &str| -> Option<Vec<(u8, u16)>> {
            t.strip_prefix('+')?
                .split(',')
                .filter(|x| !x.is_empty())
                .map(|kv| {
                    let (k, v) = kv.split_once(':')?;
                    Some((k.parse().ok()?, v.parse().ok()?))
                })
                .collect()
        };
        if f[6] != "-" {
            rec.builds = Some((list(f[6])?, list(f[7])?));
        }
        out.push(rec);
    }
    Some(out)
}

/// One dump's records, from the cache or read fresh (and cached).
fn records(name: &str) -> Option<Vec<Rec>> {
    let path = crate::testenv::dump(name)?;
    let meta = std::fs::metadata(&path).ok()?;
    let mtime = meta
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let key = cache_dir().join(format!("{name}.{}.{mtime}", meta.len()));
    if let Ok(text) = std::fs::read_to_string(&key)
        && let Some(r) = decode(&text)
    {
        return Some(r);
    }
    let recs = extract(Path::new(&path)).expect("the dump reads");
    let _ = std::fs::create_dir_all(cache_dir());
    let tmp = key.with_extension(format!("tmp{}", std::process::id()));
    if let Ok(mut f) = std::fs::File::create(&tmp)
        && f.write_all(encode(&recs).as_bytes()).is_ok()
    {
        let _ = std::fs::rename(&tmp, &key);
    }
    Some(recs)
}

/// What one step says of one pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// An input of the step is not printed on the pair.
    Unprinted,
    /// The step gives the printed leaf; `moved` if the leaf moved.
    Agree { moved: bool },
    /// The step gives `ours` where the original prints `theirs`.
    Part { ours: i64, theirs: i64 },
}

/// The leaves the harness transfers, each with its step.
pub(crate) const LEAVES: [&str; 13] = [
    "territory_high",
    "high_buildings",
    "barracks_queued",
    "stable_queued",
    "factory_queued",
    "combat_queued",
    "dock_queued",
    "air_queued",
    "supply_mark",
    "average_death_rate",
    "average_kill_rate",
    "average_hit_rate",
    "average_damage_rate",
];

/// How a step is bent for the red-first check: `None` is the crate's step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Bend {
    None,
    /// The bins' increment doubled.
    BinIncrement,
    /// The empty fold's decay `·6 >> 3` in place of `·7 >> 3`.
    Decay,
    /// A newborn supply record appended rather than reusing a free one.
    SupplyAppend,
    /// `territory_high` taking the second record's `territory` outright.
    HighFollows,
    /// `high_buildings`' line count one too many.
    LineOver,
}

fn word(r: &Rec, k: &str) -> Option<i64> {
    let v = r.words[slot(k)?];
    (v != MISSING).then_some(v)
}

/// **The bins' step** across a pair: every unit record's `num_queued` move,
/// one `±1` at a time, through the bins `armed_queued_bins` chooses.
fn step_bins(sim: &Sim, a: &Rec, b: &Rec, bend: Bend) -> Option<[i32; 6]> {
    if !a.queued_printed || !b.queued_printed {
        return None;
    }
    let mut bins = [0i32; 6];
    for (i, name) in ARMED_QUEUED.iter().enumerate() {
        bins[i] = i32::try_from(word(a, name)?).ok()?;
    }
    let moved: std::collections::BTreeSet<usize> = a
        .queued
        .iter()
        .chain(&b.queued)
        .map(|q| usize::from(q.0))
        .collect();
    for r in moved {
        let mut d = b.queued_of(r) - a.queued_of(r);
        // `num_queued` is a `ushort`: a move past half its range is a wrap.
        if d > 0x8000 {
            d -= 0x1_0000;
        } else if d < -0x8000 {
            d += 0x1_0000;
        }
        if d == 0 {
            continue;
        }
        let Some(on) = sim.armed_queued_bins(r) else {
            continue;
        };
        let unit: i32 = if bend == Bend::BinIncrement { 2 } else { 1 };
        let step = if d > 0 { unit } else { -unit };
        for _ in 0..d.abs() {
            for (c, &on) in bins.iter_mut().zip(&on) {
                if on {
                    step_bin(c, step);
                }
            }
        }
    }
    Some(bins)
}

/// **The supply mark's step**: the crate's list rebuilt from the first
/// record's holders, each record a unit gives up given back, each new
/// holder registered (the lowest printed slot first), the list's length.
fn step_supply(a: &Rec, b: &Rec, bend: Bend) -> Option<i64> {
    let (sa, sb) = (a.supply.as_ref()?, b.supply.as_ref()?);
    let mark = word(a, "supply_mark")?;
    let mut list = SupplyList::new();
    for s in 0..mark.max(0) {
        list.register(s as usize);
    }
    for s in 0..mark.max(0) {
        if !sa.contains_key(&s) {
            list.close(s as usize);
        }
    }
    // A record the first frame's units do not account for: the list
    // cannot be rebuilt, so the step's input is not printed.
    if list.len() as i64 != mark || sa.keys().any(|s| *s >= mark) {
        return None;
    }
    for (s, u) in sa {
        if sb.get(s) != Some(u) {
            list.close(*s as usize);
        }
    }
    let mut born = 0;
    for (s, u) in sb {
        if sa.get(s) != Some(u) {
            list.register(*s as usize);
            born += 1;
        }
    }
    let bent = i64::from(bend == Bend::SupplyAppend && born > 0);
    Some(list.len() as i64 + bent)
}

/// **`territory_high`'s step** across a pair: the fold on the first
/// record's mark and the second's `territory`.
fn step_territory_high(a: &Rec, b: &Rec, bend: Bend) -> Option<i64> {
    let high = i32::try_from(word(a, "territory_high")?).ok()?;
    let t = i32::try_from(word(b, "territory")?).ok()?;
    if bend == Bend::HighFollows {
        return Some(i64::from(t));
    }
    Some(i64::from(sim::territory::fold_territory_high(high, t)))
}

/// **`high_buildings`' step** across a pair: each record whose
/// `num_buildings` rose is activated, its line counted on the second record,
/// its root's mark raised. `Ok(())` when the step gives the second record's
/// array, else the first index that parts, `(ours, theirs)`.
fn step_high_buildings(
    sim: &Sim,
    a: &Rec,
    b: &Rec,
    bend: Bend,
) -> Option<Result<bool, (i64, i64)>> {
    let ((na, ha), (nb, hb)) = (a.builds.as_ref()?, b.builds.as_ref()?);
    let dense = |l: &[(u8, u16)]| {
        let mut v = [0i32; 129];
        for &(k, x) in l {
            v[usize::from(k)] = i32::from(x);
        }
        v
    };
    let (na, ha, nb, hb) = (dense(na), dense(ha), dense(nb), dense(hb));
    // Across more than one tick the order of the activations and losses is
    // not printed: a pair that moves `num_buildings` over a gap has no
    // step.
    if b.frame != a.frame + 1 && na != nb {
        return None;
    }
    // A city taken in the pair (`cities_captured`, printed, rose) brings its
    // buildings through `Cities::capture_city`'s `activate(1, 1, 0)`: the
    // rises are captures and raise no mark.
    let captured = word(b, "cities_captured")? > word(a, "cities_captured")?;
    let mut high = ha;
    for t in 0..129usize.min(sim.tables.build_types.len()) {
        if nb[t] <= na[t] {
            continue;
        }
        let line = sim.building_line(t);
        let n: i32 = line
            .iter()
            .map(|&u| nb.get(u).copied().unwrap_or(0))
            .sum::<i32>()
            + i32::from(bend == Bend::LineOver);
        let root = sim.base_build_type(t);
        if let Some(r) = sim::cities::nations::raise_building_high(high[root], n, captured) {
            high[root] = r;
        }
    }
    Some(match (0..129).find(|&i| high[i] != hb[i]) {
        None => Ok(ha != hb),
        Some(i) => Err((i64::from(high[i]), i64::from(hb[i]))),
    })
}

/// The age `Rates::fold`'s battle test reads: `ages_get()`, 0-based.
fn age(r: &Rec) -> Option<i32> {
    i32::try_from(word(r, "ages_get()")?).ok()
}

/// **The averages' step** across a pair: no fold between the records holds
/// them still; one fold runs `Rates::fold` on the first record's words,
/// the window's count read off each total's step; more than one is a count
/// per fold the pair does not print.
fn step_rates(a: &Rec, b: &Rec, bend: Bend) -> Option<Rates> {
    let mut r = Rates {
        frame_battle: word(a, "frame_battle")?,
        ..Rates::default()
    };
    for (k, (avg, total)) in AVERAGES.iter().enumerate() {
        r.average[k] = u16::try_from(word(a, avg)?).ok()?;
        r.fifteen[k] = u16::try_from(word(a, total)?).ok()?;
        let after = u16::try_from(word(b, total)?).ok()?;
        r.current[k] = after.wrapping_sub(r.fifteen[k]);
        let _ = word(a, CURRENT[k])?;
    }
    // The ticks between the records are `a.frame .. b.frame`; the fold runs
    // on a tick whose frame is a multiple of 50, for a seat (`who < 8`).
    let folds: Vec<i64> = (a.frame..b.frame)
        .filter(|f| f % FOLD_PERIOD == 0)
        .collect();
    if a.who >= 8 || folds.is_empty() {
        let mut held = r.clone();
        held.current = [0; 4];
        return Some(held);
    }
    let [f] = folds[..] else { return None };
    let before = r.average;
    r.fold(f, age(a)?);
    if bend == Bend::Decay {
        for ((avg, c), was) in r.average.iter_mut().zip(r.current).zip(before) {
            if c == 0 && *avg == ((u32::from(was) * 7) >> 3) as u16 {
                *avg = ((u32::from(was) * 6) >> 3) as u16;
            }
        }
    }
    Some(r)
}

/// One leaf's verdict on one pair.
pub(crate) fn verdict(sim: &Sim, leaf: &str, a: &Rec, b: &Rec, bend: Bend) -> Verdict {
    if leaf == "high_buildings" {
        return match step_high_buildings(sim, a, b, bend) {
            None => Verdict::Unprinted,
            Some(Ok(moved)) => Verdict::Agree { moved },
            Some(Err((ours, theirs))) => Verdict::Part { ours, theirs },
        };
    }
    let Some(theirs) = word(b, leaf) else {
        return Verdict::Unprinted;
    };
    let ours = if let Some(i) = ARMED_QUEUED.iter().position(|n| *n == leaf) {
        step_bins(sim, a, b, bend).map(|v| i64::from(v[i]))
    } else if leaf == "supply_mark" {
        step_supply(a, b, bend)
    } else if leaf == "territory_high" {
        step_territory_high(a, b, bend)
    } else if let Some(k) = AVERAGES.iter().position(|(n, _)| *n == leaf) {
        step_rates(a, b, bend).map(|r| i64::from(r.average[k]))
    } else {
        None
    };
    match ours {
        None => Verdict::Unprinted,
        Some(o) if o == theirs => Verdict::Agree {
            moved: word(a, leaf) != Some(theirs),
        },
        Some(o) => Verdict::Part { ours: o, theirs },
    }
}

/// One leaf's tally over the corpus.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Tally {
    pub pairs: usize,
    pub moving: usize,
    pub unprinted: usize,
    /// Pairs whose frames repeat a label or do not advance.
    pub unplaced: usize,
    /// The first partings: `(dump, who, frame, next frame, ours, theirs)`.
    pub parts: Vec<(String, i64, i64, i64, i64, i64)>,
    pub parted: usize,
}

/// Every leaf's tally over the given dumps' records.
pub(crate) fn tally(
    sim: &Sim,
    dumps: &[(String, Vec<Rec>)],
    bend: Bend,
) -> BTreeMap<&'static str, Tally> {
    let mut out: BTreeMap<&'static str, Tally> =
        LEAVES.iter().map(|l| (*l, Tally::default())).collect();
    for (name, recs) in dumps {
        let mut by: BTreeMap<i64, Vec<&Rec>> = BTreeMap::new();
        for r in recs {
            by.entry(r.who).or_default().push(r);
        }
        for seq in by.values() {
            // A label printed twice (run27's and run29's 15101, the first
            // still the state before 15100's tick) cannot place its record
            // in time: a pair touching one is not a step's pair.
            let mut labels: BTreeMap<i64, usize> = BTreeMap::new();
            for r in seq {
                *labels.entry(r.frame).or_default() += 1;
            }
            for w in seq.windows(2) {
                let (a, b) = (w[0], w[1]);
                if b.frame <= a.frame || labels[&a.frame] > 1 || labels[&b.frame] > 1 {
                    for leaf in LEAVES {
                        out.get_mut(leaf).unwrap().unplaced += 1;
                    }
                    continue;
                }
                for leaf in LEAVES {
                    let t = out.get_mut(leaf).unwrap();
                    match verdict(sim, leaf, a, b, bend) {
                        Verdict::Unprinted => t.unprinted += 1,
                        Verdict::Agree { moved } => {
                            t.pairs += 1;
                            t.moving += usize::from(moved);
                        }
                        Verdict::Part { ours, theirs } => {
                            t.pairs += 1;
                            t.parted += 1;
                            if t.parts.len() < 8 {
                                t.parts
                                    .push((name.clone(), a.who, a.frame, b.frame, ours, theirs));
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// The corpus's records, once a test binary: both tests here read them.
fn corpus_records() -> &'static [(String, Vec<Rec>)] {
    static CORPUS: std::sync::OnceLock<Vec<(String, Vec<Rec>)>> = std::sync::OnceLock::new();
    CORPUS.get_or_init(read_corpus)
}

/// The corpus's records, read (or decoded from the cache) on a few threads.
fn read_corpus() -> Vec<(String, Vec<Rec>)> {
    let names = corpus();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let done = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(n) = names.get(i) else { break };
                    if let Some(r) = records(n)
                        && !r.is_empty()
                    {
                        done.lock().unwrap().push((n.clone(), r));
                    }
                }
            });
        }
    });
    let mut v = done.into_inner().unwrap();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

/// **The pinned transfer**, per leaf: `(leaf, pairs, moving pairs,
/// partings)` over every scored kept dump that prints the level-9 record
/// (910 at item 4159's step T). A pair touching a repeated frame label is
/// not counted (864 such, run27's and run29's among them); a pair whose
/// inputs the records do not print is not counted either (the bins and the
/// averages 9,738 and 9,972 such, mostly the level-8 neighbours;
/// `supply_mark` 6,788, a frame with no unit printed or a list the first
/// record's units do not account for).
///
/// `barracks_queued` and `combat_queued` part on one pair: run462's who 1,
/// 10367 → 10368, ours 2 theirs 1 — building 2020's queue retargets an
/// entry from `TypeIndex` 134 to 84 and `num_queued` does not move (it has
/// drifted to its floor on that type), so the step's input is not what the
/// array prints (parked 4159Ta). The three other averages transfer, but
/// part in the record diff on frames of their own (4177a, 4177b), so they
/// stay held.
///
/// Item 4187 added `territory_high` (286 moving pairs, none parting; it
/// agrees in the record diff too) and `high_buildings` (712 moving, none
/// parting, after the step learned two things the first try parted on:
/// a pair whose `cities_captured` rose is a capture — run594's 18140 →
/// 18141, run609's 5638 → 5639, run958f's 22700 → 22701, run1007c's 16564
/// → 16565 and run1018's 12658 → 12659, ours 6 / 3 / 6 / 6 / 4 against 5 /
/// 2 / 5 / 5 / 3 — and a pair over a gap that moves `num_buildings`
/// has no printed order — run25's and run26's 12132/12027 → 16008).
///
/// Item 4206: 910 → 912 dumps, run1575 and run1576 (the diplomacy take
/// and its twin, 255 level-9 blocks of three leaders each). Every leaf's
/// pairs rise by 2,540 (`supply_mark`'s none: the take prints no unit, so
/// its 2,540 are unprinted, 6,788 → 9,328). Moving pairs rise on five
/// leaves: `barracks_queued` 696 → 700, `combat_queued` 972 → 978,
/// `high_buildings` 712 → 726, `stable_queued` 338 → 344, `territory_high`
/// 286 → 288. No parting moves.
const PINNED: &[(&str, usize, usize, usize)] = &[
    ("air_queued", 479_666, 288, 0),
    ("average_damage_rate", 479_432, 1482, 0),
    ("average_death_rate", 479_432, 1419, 0),
    ("average_hit_rate", 479_432, 1286, 0),
    ("average_kill_rate", 479_432, 1527, 0),
    ("barracks_queued", 479_666, 700, 1),
    ("combat_queued", 479_666, 978, 1),
    ("dock_queued", 479_666, 136, 0),
    ("factory_queued", 479_666, 188, 0),
    ("high_buildings", 479_464, 726, 0),
    ("stable_queued", 479_666, 344, 0),
    ("supply_mark", 480_076, 82, 0),
    ("territory_high", 479_668, 288, 0),
];

/// The leaves released on transfer: each transfers on every pair (no
/// parting, a moving pair) and is emitted on the leaders line.
const RELEASED: [&str; 7] = [
    "stable_queued",
    "factory_queued",
    "air_queued",
    "supply_mark",
    "average_kill_rate",
    // Item 4187 (`docs/LEADERS.md` §20): `high_buildings` parts in the
    // record diff only in run1075's window, beside `num_buildings[21]`;
    // `territory_high` parts nowhere and transfers too.
    "high_buildings",
    "territory_high",
];

#[test]
fn the_held_leaves_transfer_on_the_original_s_printed_inputs() {
    let Some(inst) = install() else {
        eprintln!("skipping: no install (RON_INSTALL)");
        return;
    };
    let dumps = corpus_records();
    if dumps.is_empty() {
        eprintln!("skipping: no kept dump prints the level-9 record (RON_GAMELOG_DIR)");
        return;
    }
    let loaded = crate::load::load(&inst).unwrap();
    let built = build_sim(&loaded, &initial(), Tuning::RON);
    let t = tally(&built.sim, dumps, Bend::None);
    eprintln!("transfer: {} dumps print the level-9 record", dumps.len());
    let mut got = Vec::new();
    for (leaf, x) in &t {
        eprintln!(
            "TRANSFER\t{leaf}\tpairs {}\tmoving {}\tparted {}\tunprinted {}\tunplaced {}",
            x.pairs, x.moving, x.parted, x.unprinted, x.unplaced
        );
        for p in &x.parts {
            eprintln!("  PART\t{leaf}\t{p:?}");
        }
        got.push((*leaf, x.pairs, x.moving, x.parted));
    }
    assert_eq!(
        got, PINNED,
        "the transfer's pairs, moving pairs and partings"
    );
    for leaf in RELEASED {
        let x = &t[leaf];
        assert!(x.parted == 0 && x.moving > 0, "{leaf} transfers");
        assert!(
            !sim::leaders::rates::RATES_OFF_THE_LINE.contains(&leaf),
            "{leaf} is emitted"
        );
        assert!(
            !crate::diff::checksum_lines::LEADERS_HELD_OUT
                .iter()
                .any(|(n, _)| n.trim_end_matches(" (unheld)") == leaf),
            "{leaf} left the held-out list"
        );
    }
}

/// **A bent step parts on a named pair** (DECISIONS 128 §3's "seen red once
/// on a bent step", kept as a check): each family's step perturbed — the
/// bins' increment doubled, the empty fold's decay `·6 >> 3`, a newborn
/// supply record appended past the mark — parts its leaf on at least one
/// pair the dumps hold, and the first such pair is printed.
#[test]
fn the_bent_steps_part_on_a_named_pair() {
    let Some(inst) = install() else {
        eprintln!("skipping: no install (RON_INSTALL)");
        return;
    };
    let dumps = corpus_records();
    if dumps.is_empty() {
        eprintln!("skipping: no kept dump prints the level-9 record (RON_GAMELOG_DIR)");
        return;
    }
    let loaded = crate::load::load(&inst).unwrap();
    let built = build_sim(&loaded, &initial(), Tuning::RON);
    for (bend, leaf) in [
        (Bend::BinIncrement, "barracks_queued"),
        (Bend::Decay, "average_kill_rate"),
        (Bend::SupplyAppend, "supply_mark"),
        (Bend::HighFollows, "territory_high"),
        (Bend::LineOver, "high_buildings"),
    ] {
        let t = tally(&built.sim, dumps, bend);
        let x = &t[leaf];
        eprintln!(
            "BENT\t{bend:?}\t{leaf}\tparted {}\tfirst {:?}",
            x.parted,
            x.parts.first()
        );
        assert!(x.parted > 0, "a {bend:?} step parts {leaf} on a pair");
    }
}
