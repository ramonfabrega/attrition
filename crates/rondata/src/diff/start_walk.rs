//! **The game run from the built start** (item 4057, `docs/START.md` §7):
//! the harness lobby's `Sim` built by [`from_lobby`] and the installed one
//! [`build_sim`] stands up, compared whole, and the lobby's long capture
//! walked from the built one with no start dump installed.

use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

/// A `{:#?}` print flattened to `(path, value)` leaves: every field of
/// every struct, every element of every list, in print order. What the
/// whole comparison of two `Sim`s reads.
pub(crate) fn debug_leaves(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    // (component, next unnamed index)
    let mut stack: Vec<(String, usize)> = Vec::new();
    let path = |stack: &[(String, usize)], last: &str| -> String {
        let mut p = String::new();
        for (c, _) in stack.iter().skip(1) {
            p.push_str(c);
        }
        p.push_str(last);
        p
    };
    for raw in text.lines() {
        let line = raw.trim();
        let line = line.strip_suffix(',').unwrap_or(line);
        if line.is_empty() {
            continue;
        }
        if matches!(line, "}" | "]" | ")") {
            stack.pop();
            continue;
        }
        // A map entry whose key printed over several lines closes the key
        // and carries the value: `): 3`, `}: Foo {`.
        let closes_key = line.len() > 2
            && matches!(&line[..1], "}" | "]" | ")")
            && line[1..].starts_with(": ");
        let named = if closes_key {
            stack
                .pop()
                .map(|(c, _)| (format!("{}=>", c.trim_start_matches('.')), &line[3..]))
        } else {
            None
        };
        // `name: rest` where the name is a field or a map key.
        let named = named.or_else(|| line.find(": ").and_then(|i| {
            let k = &line[..i];
            let ok = k.chars().all(|c| c.is_alphanumeric() || c == '_')
                || (k.starts_with('(') && k.ends_with(')'))
                || k.chars().all(|c| c.is_ascii_digit() || c == '-');
            ok.then(|| (k.to_string(), &line[i + 2..]))
        }));
        let (comp, rest) = match named {
            Some((k, r)) => (format!(".{k}"), r),
            None => {
                let idx = stack.last_mut().map_or(0, |t| {
                    t.1 += 1;
                    t.1 - 1
                });
                (format!("[{idx}]"), line)
            }
        };
        let opens = rest.ends_with('{') || rest.ends_with('[') || rest.ends_with('(');
        if opens {
            let label = rest[..rest.len() - 1].trim();
            if !label.is_empty() {
                out.push((path(&stack, &comp), format!("<{label}>")));
            }
            stack.push((comp, 0));
        } else {
            out.push((path(&stack, &comp), rest.to_string()));
        }
    }
    out
}

/// A path with its list indices starred, `units[3].pos.x` →
/// `units[*].pos.x`: the take a leaf belongs to.
pub(crate) fn take_of(path: &str) -> String {
    let mut out = String::new();
    let mut in_idx = false;
    for c in path.chars() {
        match c {
            '[' => {
                in_idx = true;
                out.push_str("[*]");
            }
            '{' => {
                in_idx = true;
                out.push_str("{*}");
            }
            ']' | '}' => in_idx = false,
            _ if in_idx => {}
            _ => out.push(c),
        }
    }
    out
}

/// A building's or a unit's leaves re-keyed by its `(who, o)` rather than
/// its place in the store: `.units[3].pos` → `.units{1/0}.pos`. The two
/// starts make their objects in different orders (the installer in the
/// dump's, the walk in the original's creation order), so a positional
/// comparison would part on every object.
fn keyed(sim: &Sim, leaves: Vec<(String, String)>) -> Vec<(String, String)> {
    let key = |p: &str, list: &str, k: &dyn Fn(usize) -> Option<String>| -> Option<String> {
        let rest = p.strip_prefix(list)?;
        let close = rest.find(']')?;
        let i: usize = rest[..close].parse().ok()?;
        Some(format!("{}{{{}}}{}", &list[..list.len() - 1], k(i)?, &rest[close + 1..]))
    };
    let bk = |i: usize| sim.buildings.get(i).map(|b| format!("{}/{}", b.owner, b.index));
    let uk = |i: usize| sim.units.get(i).map(|u| format!("{}/{}", u.owner, u.index));
    leaves
        .into_iter()
        .map(|(p, v)| {
            let p = key(&p, ".buildings[", &bk)
                .or_else(|| key(&p, ".units[", &uk))
                .unwrap_or(p);
            (p, v)
        })
        .collect()
}

/// The two `Sim`s' leaves compared: per take, the leaves that differ (or
/// stand on one side only), with the first such leaf's two values.
pub(crate) fn whole_diff(
    ours: &Sim,
    theirs: &Sim,
) -> std::collections::BTreeMap<String, (usize, String, Option<String>, Option<String>)> {
    let a = keyed(ours, debug_leaves(&format!("{ours:#?}")));
    let b = keyed(theirs, debug_leaves(&format!("{theirs:#?}")));
    let mine: std::collections::HashMap<&str, &str> =
        a.iter().map(|(p, v)| (p.as_str(), v.as_str())).collect();
    let yours: std::collections::HashMap<&str, &str> =
        b.iter().map(|(p, v)| (p.as_str(), v.as_str())).collect();
    let mut out: std::collections::BTreeMap<String, (usize, String, Option<String>, Option<String>)> =
        Default::default();
    let mut note = |p: &str, o: Option<&str>, t: Option<&str>| {
        let e = out.entry(take_of(p)).or_insert_with(|| {
            (0, p.to_string(), o.map(str::to_string), t.map(str::to_string))
        });
        e.0 += 1;
    };
    for (p, v) in &b {
        let o = mine.get(p.as_str()).copied();
        if o != Some(v.as_str()) {
            note(p, o, Some(v));
        }
    }
    for (p, v) in &a {
        if !yours.contains_key(p.as_str()) {
            note(p, Some(v), None);
        }
    }
    out
}

/// The harness lobby's two starts, `(ours, theirs, players)`: [`from_lobby`]
/// on run9's `GAMEINFO` and [`build_sim`] on run9 with its siblings, as
/// `the_harness_lobby_s_start_built_from_its_choices` builds them.
fn harness_starts<R>(f: impl FnOnce(&Loaded, FromLobby, Built, usize) -> R) -> Option<R> {
    let inst = install()?;
    let Some(path) = dump("gamelog-run9-world6.txt") else {
        eprintln!("skipping: no gamelog-run9-world6.txt (set RON_GAMELOG_DIR)");
        return None;
    };
    let loaded = crate::load::load(&inst).unwrap();
    Some(with_sibling_initials(|sibs| {
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let mut init = replay_initial(&log).unwrap();
        borrow_from_siblings(&mut init, sibs);
        let players = player_count(&init);
        let theirs = build_sim(&loaded, &init, Tuning::RON);
        let ours = from_lobby(&loaded, &init.game_info, &init.players, Tuning::RON);
        f(&loaded, ours, theirs, players)
    }))
}

/// **Frame zero wider than the block, on the harness lobby** (item 4057,
/// `docs/START.md` §7.3): [`start_wide`] of the built start against the
/// installed one. Every take it projects agrees but the three
/// [`START_WIDE_PARTED`] names, each pinned by its count. `RON_START_DIFF=1`
/// prints the parted rows; `RON_START_WHOLE=1` also prints the two `Sim`s'
/// whole comparison ([`whole_diff`]), take by take.
#[test]
fn the_harness_lobby_s_start_is_the_installed_one_wide() {
    harness_starts(|loaded, ours, theirs, players| {
        let (ours, _) = built_from_lobby(loaded, ours);
        let a = start_wide(&ours.sim, players);
        let b = start_wide(&theirs.sim, players);
        let parted = compare_start(&a, &b);
        let mut by: std::collections::BTreeMap<(String, &str), usize> = Default::default();
        for p in &parted {
            let kind = p.record.split(' ').next().unwrap_or("").to_string();
            *by.entry((kind, p.field)).or_default() += 1;
        }
        if std::env::var_os("RON_START_DIFF").is_some() {
            for p in parted.iter().take(40) {
                eprintln!("{p}");
            }
        }
        if std::env::var_os("RON_START_WHOLE").is_some() {
            for (take, (n, first, o, t)) in whole_diff(&ours.sim, &theirs.sim) {
                eprintln!("TAKE {take}: {n} parted; first {first}: ours {o:?} theirs {t:?}");
            }
        }
        let got: Vec<(String, &str, usize)> = by.into_iter().map(|((k, f), n)| (k, f, n)).collect();
        let want: Vec<(String, &str, usize)> = START_WIDE_PARTED
            .iter()
            .map(|&(k, f, n)| (k.to_string(), f, n))
            .collect();
        eprintln!(
            "wide: {} fields, {} parted, by take {got:?}",
            b.len(),
            parted.len()
        );
        assert_eq!(b.len(), START_WIDE_FIELDS, "the wide projection's field count");
        assert_eq!(got, want, "the wide takes that part (RON_START_DIFF=1 prints them)");
    });
}

/// The wide projection's fields on the harness lobby: 14,400 fog cells,
/// 36 goods and 22 goody boxes at three fields each, 13 mountains at
/// three, 13 buildings' `ever_seen`, the limits, 8 regions, 57,600 tiles
/// and 3,600 cells.
const START_WIDE_FIELDS: usize = 75_835;

/// The wide takes still parted on the harness lobby, `(record, field,
/// count)` (`docs/START.md` §7.3, each named with what it would need):
/// the land region's `flags` (ours 0x4, theirs 0xa4: the writer of 0x20
/// and 0x80 is not found), tile bit 0x4 on 131 tiles north of seat 1's
/// city (the writer not identified), and `land_sub` on eight cells beside
/// the two cities.
const START_WIDE_PARTED: &[(&str, &str, usize)] = &[
    ("CELL", "land_sub", 8),
    ("REGION", "flags", 1),
    ("TILE", "mask", 131),
];

/// What one frame's whole-record comparison parted on: each kind of
/// divergence the frame holds, its count and its first row.
pub(crate) fn frame_partings(r: &FrameResult) -> Vec<(&'static str, usize, String)> {
    let mut out = Vec::new();
    macro_rules! kinds {
        ($($f:ident),*) => {$(
            if let Some(first) = r.$f.first() {
                out.push((stringify!($f), r.$f.len(), format!("{first:?}")));
            }
        )*};
    }
    kinds!(
        unlinked_units,
        extra_units,
        diverged,
        order_diverged,
        los_diverged,
        packed_diverged,
        visible_diverged,
        inside_diverged,
        hits_diverged,
        firing_diverged,
        death_diverged,
        angle_diverged,
        collide_diverged,
        search_diverged,
        gather_diverged,
        build_diverged,
        queue_diverged,
        city_diverged
    );
    if r.build_unlinked > 0 {
        out.push(("build_unlinked", r.build_unlinked, String::new()));
    }
    if r.city_unlinked > 0 {
        out.push(("city_unlinked", r.city_unlinked, String::new()));
    }
    out
}

/// One walk of the lobby's game: the frame the draws first part on
/// (count, then sequence) against the trace, and the first record the
/// whole comparison parts on with what parted.
#[derive(Debug, Default)]
pub(crate) struct Walked {
    pub last: i64,
    pub first_count: Option<i64>,
    pub first_sequence: Option<i64>,
    /// `(frame, capture, what parted)`.
    pub first_record: Option<(i64, String, Vec<(&'static str, usize, String)>)>,
    pub records: usize,
    pub records_parted: usize,
    /// The records compared and parted, per capture.
    pub per_capture: Vec<(String, usize, usize)>,
    /// The first parting frame's two draw sequences.
    pub sites: Option<(Vec<String>, Vec<String>)>,
}

/// Walk `built` to the trace's last frame (or `limit`), comparing every
/// record each capture of `captures` holds — every capture a recording of
/// the same game — whole as it is reached, and every frame's draws
/// against `trace`.
pub(crate) fn walk(
    mut built: Built,
    players: usize,
    captures: &[(&str, String)],
    trace: &crate::trace::Trace,
    limit: i64,
) -> Walked {
    let mut w = Walked::default();
    let mut sources: Vec<crate::capture::indexed::IndexedCapture> = captures
        .iter()
        .map(|(_, p)| crate::capture::indexed::IndexedCapture::open(p).unwrap())
        .collect();
    let mut at: Vec<(i64, usize, usize)> = Vec::new();
    for (k, s) in sources.iter().enumerate() {
        for (i, f) in s.frames().iter().enumerate() {
            at.push((f.number, k, i));
        }
    }
    at.sort();
    w.per_capture = captures
        .iter()
        .map(|(n, _)| (n.to_string(), 0, 0))
        .collect();
    for (n, k, i) in at {
        if n > limit {
            break;
        }
        let frame = sources[k].frame_state(i).unwrap();
        while built.sim.frame < frame.n {
            built.tick();
        }
        let r = compare(&built, &frame, players);
        w.records += 1;
        w.per_capture[k].1 += 1;
        let p = frame_partings(&r);
        if !p.is_empty() {
            w.records_parted += 1;
            w.per_capture[k].2 += 1;
            if w.per_capture[k].2 == 1 && std::env::var_os("RON_START_WALK_ROWS").is_some() {
                eprintln!("FIRST {} at {}: {p:?}", captures[k].0, frame.n);
            }
            if w.first_record.is_none() {
                w.first_record = Some((frame.n, captures[k].0.to_string(), p));
            }
        }
    }
    // On to the trace's own end: the draws are the oracle there.
    let end = trace.frames.last().map_or(0, |(n, _)| *n).min(limit);
    while built.sim.frame < end {
        built.tick();
    }
    w.last = built.sim.frame;
    for (f, ours) in &built.frame_sites {
        let theirs = trace.labels(*f);
        if w.first_count.is_none() && ours.len() != theirs.len() {
            w.first_count = Some(*f);
        }
        if w.first_sequence.is_none() && *ours != theirs {
            w.first_sequence = Some(*f);
            w.sites = Some((ours.clone(), theirs));
        }
    }
    w
}

/// The captures of the harness lobby's game (run53's: seed 12345, Great
/// Lakes, Small, two players, no input) whose records the walk compares:
/// the long capture's own, the `DUMP_ALL` head run12 (run3, the heights'
/// sibling, is another game of the lobby: its frame 1 parts on the
/// installed start as well), the
/// windows the widenings walk (run13; the mid-game chain run125 to run174;
/// run226, run243, run294) and the last blocks any dump holds (run80).
pub(crate) const LOBBY_GAME: &[&str] = &[
    "gamelog-run53-greatlakes-24k-trace.txt",
    "gamelog-run12-dumpall-seeds.txt",
    "gamelog-run13-window-95-105.txt",
    "gamelog-run125-greatlakes-armyidle.txt",
    "gamelog-run130-greatlakes-armytwo.txt",
    "gamelog-run135-greatlakes-crossing.txt",
    "gamelog-run136-greatlakes-detour.txt",
    "gamelog-run163-greatlakes-upgradeword.txt",
    "gamelog-run174-greatlakes-civicword.txt",
    "gamelog-run226-greatlakes-giveupword.txt",
    "gamelog-run243-greatlakes-standword.txt",
    "gamelog-run294-greatlakes-departword.txt",
    "gamelog-run80-greatlakes-latecensus.txt",
];

/// **The lobby's own walk from the built start** (item 4057,
/// `docs/START.md` §7.4): run53's game (the harness lobby, 24,000 frames,
/// no input) walked from [`from_lobby`]'s `Sim` with no start dump
/// installed and no correction, every frame's draws against run53's trace
/// and every record of [`LOBBY_GAME`]'s captures compared whole. The
/// control is the same walk from [`build_sim`]'s install with its
/// corrections off (no frame's word, no clock, no re-seat): the two must
/// agree on the word, on the first parting draw and on every capture's
/// parted records, and the built side's word is the pinned
/// [`START_WALKED`]. `RON_START_WALK_ROWS=1` prints each capture's first
/// parted record, both sides.
#[test]
fn the_lobby_walked_from_the_built_start() {
    let Some(inst) = install() else { return };
    let (Some(path), Some(tr)) = (
        dump("gamelog-run53-greatlakes-24k-trace.txt"),
        trace("rontrace-run53.log"),
    ) else {
        eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let captures: Vec<(&str, String)> = LOBBY_GAME
        .iter()
        .filter_map(|n| dump(n).map(|p| (*n, p)))
        .collect();
    let mut source = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
    let (built, installed, players) = with_sibling_initials(|sibs| {
        source
            .with_replay_initial(|init| {
                let mut init = init;
                borrow_from_siblings(&mut init, sibs);
                borrow_pasture(&mut init, &tr);
                let players = player_count(&init);
                let mut b = build_sim(&loaded, &init, Tuning::RON);
                b.frame_seeds.clear();
                b.frame_guys.clear();
                b.sim.trace.trace_phases = true;
                let ours = from_lobby(&loaded, &init.game_info, &init.players, Tuning::RON);
                (built_from_lobby(&loaded, ours).0, b, players)
            })
            .unwrap()
    });
    drop(source);
    let ours = walk(built, players, &captures, &tr, i64::MAX);
    let theirs = walk(installed, players, &captures, &tr, i64::MAX);
    for (side, w) in [("built", &ours), ("installed", &theirs)] {
        eprintln!(
            "WALK {side}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
    }
    if let Some((o, t)) = &ours.sites
        && let Some((_, d)) = first_parting(o, t)
    {
        eprintln!("the built walk's first parting frame, {:?}: {d}", ours.first_sequence);
    }
    let word = ours.first_count.unwrap_or(ours.last);
    let (map, floor, length) = START_WALKED;
    assert_eq!(map, "GreatLakes");
    assert_eq!(ours.last, length + 1, "the walk reached the trace's end");
    assert_eq!(word, floor, "the built start's word (START_WALKED)");
    assert_eq!(
        (ours.first_count, ours.first_sequence),
        (theirs.first_count, theirs.first_sequence),
        "the built start's draws part where the installed start's do"
    );
    assert_eq!(
        ours.per_capture, theirs.per_capture,
        "every capture's records part alike from both starts"
    );
    assert_eq!(
        ours.first_record, theirs.first_record,
        "the first parted record is the same from both starts"
    );
    assert_eq!(
        (ours.records, ours.records_parted),
        START_WALK_RECORDS,
        "the records the built walk compares and parts on"
    );
}

/// The records [`LOBBY_GAME`]'s captures hold and the built walk compares,
/// and those it parts on: 2,770 and 10 (item 4057) — each window's
/// opening block (its three cities unlinked, and on run136, run163 and
/// run226 one citizen a few units off), the same ten from the installed
/// start.
const START_WALK_RECORDS: (usize, usize) = (2770, 10);

