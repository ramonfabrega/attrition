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

/// Probe: the two starts compared whole, every take that parts printed.
#[test]
fn the_two_starts_compared_whole() {
    harness_starts(|loaded, ours, theirs, _| {
        let (ours, _) = built_from_lobby(loaded, ours);
        let d = whole_diff(&ours.sim, &theirs.sim);
        for (take, (n, first, o, t)) in &d {
            eprintln!("TAKE {take}: {n} parted; first {first}: ours {o:?} theirs {t:?}");
        }
        eprintln!("{} takes part", d.len());
        // Probe detail: the fog and the goods.
        let (fw, fh) = (ours.sim.world.fog_xs(), 2 * ours.sim.world.height());
        let mut by = std::collections::BTreeMap::<(u8, u8), usize>::new();
        let mut first = Vec::new();
        for fy in 0..fh {
            for fx in 0..fw {
                let (a, b) = (ours.sim.world.seen2(fx, fy), theirs.sim.world.seen2(fx, fy));
                let k = (a.unwrap_or(99), b.unwrap_or(99));
                *by.entry(k).or_default() += 1;
                if a != b && first.len() < 12 {
                    first.push((fx, fy, a, b));
                }
            }
        }
        eprintln!("FOG (ours, theirs) -> n: {by:?}; first {first:?}");
        for bit in 0..2u8 {
            let m = 1u8 << bit;
            let mut only: Vec<(i32, i32)> = Vec::new();
            for fy in 0..fh {
                for fx in 0..fw {
                    let a = ours.sim.world.seen2(fx, fy).unwrap_or(0) & m != 0;
                    let b = theirs.sim.world.seen2(fx, fy).unwrap_or(0) & m != 0;
                    if b && !a {
                        only.push((fx, fy));
                    }
                    if a && !b {
                        only.push((-fx - 1, -fy - 1));
                    }
                }
            }
            let (xs, ys): (Vec<i32>, Vec<i32>) = only.iter().copied().unzip();
            eprintln!(
                "FOGBIT {bit}: {} cells; x {:?}..{:?} y {:?}..{:?}; ours-only {}",
                only.len(),
                xs.iter().filter(|x| **x >= 0).min(),
                xs.iter().max(),
                ys.iter().filter(|y| **y >= 0).min(),
                ys.iter().max(),
                xs.iter().filter(|x| **x < 0).count()
            );
        }
        for b in theirs.sim.buildings.iter().filter(|b| b.alive) {
            eprintln!("BUILD {}/{} pos {:?} fog ({}, {}) los {}", b.owner, b.index, b.pos, b.pos.x / 384, b.pos.y / 384, b.mylos);
        }
        for u in theirs.sim.units.iter().filter(|u| u.owner < 2) {
            eprintln!("UNIT {}/{} pos {:?} fog ({}, {}) los {}", u.owner, u.index, u.pos, u.pos.x / 384, u.pos.y / 384, u.mylos);
        }
        for (i, (a, b)) in ours.sim.world.goods().iter().zip(theirs.sim.world.goods()).enumerate() {
            if a != b {
                eprintln!("GOOD {i}: ours {a:?} theirs {b:?}");
            }
        }
    });
}

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
/// the long capture's own, the `DUMP_ALL` heads (run3, run12), the
/// windows the widenings walk (run13; the mid-game chain run125 to run174;
/// run226, run243, run294) and the last blocks any dump holds (run80).
pub(crate) const LOBBY_GAME: &[&str] = &[
    "gamelog-run53-greatlakes-24k-trace.txt",
    "gamelog-run3-fulldump-types.txt",
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

/// Probe: the harness lobby's long capture (run53, 24,000 frames) walked
/// from the built start, and — as the control — from the installed start
/// with its corrections off (`RON_START_WALK=installed`).
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
    let limit: i64 = std::env::var("RON_START_WALK_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(i64::MAX);
    let side = std::env::var("RON_START_WALK").unwrap_or_default();
    let loaded = crate::load::load(&inst).unwrap();
    let captures: Vec<(&str, String)> = LOBBY_GAME
        .iter()
        .filter_map(|n| dump(n).map(|p| (*n, p)))
        .collect();
    let mut source = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
    let (built, players) = with_sibling_initials(|sibs| {
        source
            .with_replay_initial(|init| {
                let mut init = init;
                borrow_from_siblings(&mut init, sibs);
                borrow_pasture(&mut init, &tr);
                let players = player_count(&init);
                if side == "installed" {
                    let mut b = build_sim(&loaded, &init, Tuning::RON);
                    b.frame_seeds.clear();
                    b.frame_guys.clear();
                    b.sim.trace.trace_phases = true;
                    (b, players)
                } else {
                    let ours = from_lobby(&loaded, &init.game_info, &init.players, Tuning::RON);
                    (built_from_lobby(&loaded, ours).0, players)
                }
            })
            .unwrap()
    });
    drop(source);
    let w = walk(built, players, &captures, &tr, limit);
    eprintln!(
        "WALK {side}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}; per capture {:?}",
        w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record, w.per_capture
    );
    if let Some((o, t)) = &w.sites
        && let Some((_, d)) = first_parting(o, t)
    {
        eprintln!("{d}");
    }
}
