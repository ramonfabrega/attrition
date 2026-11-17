//! **Step VII-a of the leaders chain** (item 4206, `docs/DECISIONS.md`
//! entry 131 §2): the diplomacy leaves' capture, and our walk of it.
//!
//! The first act walks run1078 — the coverage pair's lobby at `TEAM_STYLE
//! 0` — from the coverage pair's own frame zero ([`walk_style_zero`]:
//! run652's initial with run651's head and run652's pasture) with the lobby
//! re-read at style 0: the field and the peace `Leader::init` writes there
//! (`diplos` 1 both ways), which run1078's block 896 prints.

use super::*;
use crate::diff::testkit::*;
use crate::testenv::dump;

/// run1078 (item 3460): the coverage pair's lobby at `TEAM_STYLE 0`, a
/// human's paid declaration of war through the diplomacy window on 910,
/// the level-9 leader record 896..960 (`docs/RUNS.md` run1078).
pub(crate) const RUN1078: (&str, &str) = (
    "gamelog-run1078-eastindies-persian-diplomacy-declare.txt",
    "rontrace-run1078.log",
);

/// A capture's file: an absolute path as given (an unfiled recon under
/// `~/ron-golden`), or one of the kept dumps by name.
fn take_file(name: &str) -> Option<String> {
    if name.starts_with('/') {
        return std::path::Path::new(name)
            .is_file()
            .then(|| name.to_string());
    }
    dump(name)
}

/// [`take_file`]'s trace.
fn take_trace(name: &str) -> Option<crate::trace::Trace> {
    if !name.starts_with('/') {
        return trace(name);
    }
    let path = take_file(name)?;
    Some(
        crate::trace::Trace::read(std::path::Path::new(&path))
            .expect("invalid finalized trace")
            .expect("missing RONT header"),
    )
}

/// A script from `tools/gamelog/coverage/`, read from the tree.
pub(crate) fn coverage_script(name: &str) -> crate::golden::Script {
    let path = format!(
        "{}/../../tools/gamelog/coverage/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    crate::golden::Script::read(std::path::Path::new(&path))
        .unwrap_or_else(|e| panic!("tools/gamelog/coverage/{name}: {e}"))
}

/// The diplomacy leaves of `docs/LEADERS.md` §7 to §9 that the level-9
/// record prints: the `[scan]` arrays by slot, and `blacken`. The ring's
/// words, `incoming` and the two tribute totals are not printed; a packet
/// holds them.
pub(crate) const PRINTED_ARRAYS: [&str; 14] = [
    "good_deeds",
    "tribute_stamp",
    "gift_stamp",
    "hire_stamp",
    "hire_who",
    "dow",
    "broke_alliance",
    "made_peace",
    "last_spoke",
    "counteroffer",
    "tribute_demanded",
    "last_taunt",
    "taunt_frame",
    "tributes",
];

/// One leader block's diplomacy leaves, keyed `dow[1]`, `blacken`, … , with
/// `diplos[k]`, `treaties[k]` and `aggression[k]` beside them.
pub(crate) fn diplomacy_leaves(
    block: &crate::gamelog::Block<'_>,
) -> std::collections::BTreeMap<String, i64> {
    let mut out = std::collections::BTreeMap::new();
    let all = |k: &str| -> Vec<i64> {
        block
            .all(k)
            .iter()
            .map(|v| v.trim().parse().unwrap_or(i64::MIN))
            .collect()
    };
    for key in PRINTED_ARRAYS
        .iter()
        .chain(&["diplos", "treaties", "aggression"])
    {
        for (i, x) in all(&format!("{key}[scan]")).iter().enumerate().take(8) {
            out.insert(format!("{key}[{i}]"), *x);
        }
    }
    if let Some(x) = all("blacken").first() {
        out.insert("blacken".to_string(), *x);
    }
    out
}

/// Where a walk parts from a take's trace, what its staging did, and the
/// take's own record of the diplomacy leaves.
pub(crate) struct TakeWord {
    pub count: i64,
    pub sequence: i64,
    pub last: i64,
    pub row: String,
    pub skipped: Vec<String>,
    /// `(block, who, key, theirs, ours)` for every leader row both sides
    /// carry that parts, over the dump's blocks the walk reached.
    pub leader_parted: Vec<(i64, usize, String, i64, i64)>,
    /// `(block, who, key, before, after)`: every diplomacy leaf the take's
    /// own record moves, block on block, over the blocks the walk reached.
    pub movers: Vec<(i64, usize, String, i64, i64)>,
    /// `(block, who, key) -> ours`: our value of every diplomacy leaf the
    /// level-9 record prints ([`PRINTED_ARRAYS`], `[k]` by slot) that our
    /// leader rows carry, on every block the walk reached.
    pub ours: std::collections::BTreeMap<(i64, usize, String), i64>,
    /// `(block, who, key, theirs, ours)`: every diplomacy leaf ours carries
    /// ([`our_diplomacy_leaves`]) set against the take's own record on
    /// every block the walk reached — the leaves `diff::leader`'s record
    /// rows do not parse (item 4298).
    pub leaves: Vec<(i64, usize, String, i64, i64)>,
    /// `(frame, theirs, ours)`: the first frame whose head word — ours
    /// before the frame's staging, theirs the trace's `FRAME` record at
    /// `do_frame`'s entry — differs (parked 4300b2). The labels can agree
    /// while the values part: on recon4300-2 frame 1's head parted and the
    /// labels first did on 5 (item 4300, third act).
    pub seed: Option<(i64, u32, u32)>,
}

/// The diplomacy leaves of [`PRINTED_ARRAYS`] and `blacken` ours carries,
/// keyed as [`diplomacy_leaves`] keys theirs (item 4298). `last_spoke` is
/// a constant 0 and the taunt ring is printed in the ring's own order, so
/// neither is here.
pub(crate) fn our_diplomacy_leaves(
    built: &Built,
    who: usize,
) -> std::collections::BTreeMap<String, i64> {
    let mut out = std::collections::BTreeMap::new();
    let (Some(d), Some(t)) = (
        built.sim.leaders.declare.get(who),
        built.sim.leaders.tribute.get(who),
    ) else {
        return out;
    };
    let rows: [(&str, Vec<i64>); 10] = [
        (
            "good_deeds",
            d.good_deeds.iter().map(|&v| v.into()).collect(),
        ),
        ("hire_stamp", d.hire_stamp.to_vec()),
        ("hire_who", d.hire_who.iter().map(|&v| v.into()).collect()),
        ("dow", d.dow.iter().map(|&v| v.into()).collect()),
        ("broke_alliance", d.broke_alliance.to_vec()),
        ("made_peace", d.made_peace.to_vec()),
        ("tribute_stamp", t.tribute_stamp.to_vec()),
        ("gift_stamp", t.gift_stamp.to_vec()),
        (
            "counteroffer",
            t.counteroffer.iter().map(|&v| v.into()).collect(),
        ),
        ("tributes", t.tributes.iter().map(|&v| v.into()).collect()),
    ];
    for (k, v) in rows {
        for (i, x) in v.iter().enumerate() {
            out.insert(format!("{k}[{i}]"), *x);
        }
    }
    for (i, x) in t.tribute_demanded.iter().enumerate() {
        out.insert(format!("tribute_demanded[{i}]"), i64::from(*x));
    }
    out.insert("blacken".to_string(), i64::from(d.blacken));
    out
}

/// Walk `built` against `capture`'s trace to its end or its first count
/// parting, comparing the leader record on every block the capture's dump
/// prints, and listing the take's own diplomacy movers.
pub(crate) fn walk_take_from(
    loaded: &crate::load::Loaded,
    mut built: Built,
    capture: (&str, &str),
    seats: usize,
    probes: &[(i64, sim::Player)],
    diplomacy: &[TakeDiplo],
    taunts: &[TakeTaunt],
) -> Option<TakeWord> {
    let (Some(own), Some(tr)) = (take_file(capture.0), take_trace(capture.1)) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", capture.0);
        return None;
    };
    built.sim.trace.trace_phases = true;
    let mut ix = crate::capture::indexed::IndexedCapture::open(&own).unwrap();
    let blocks: Vec<i64> = ix.frames().iter().map(|f| f.number).collect();
    let last = tr.frames.last().map_or(0, |(n, _)| *n);
    let (mut count, mut sequence) = (None, None);
    let mut leader_parted = Vec::new();
    let mut movers = Vec::new();
    let mut leaves_seen = Vec::new();
    let mut ours = std::collections::BTreeMap::new();
    let mut prev: Vec<Option<std::collections::BTreeMap<String, i64>>> = vec![None; seats];
    let mut seed = None;
    for _ in 0..last {
        let at = built.sim.frame;
        if seed.is_none()
            && let Some(&(_, theirs)) = tr.frames.iter().find(|(f, _)| *f == at)
            && theirs != built.sim.rng.seed
        {
            seed = Some((at, theirs, built.sim.rng.seed));
        }
        stage_frame(&mut built, seats, probes, diplomacy, taunts);
        built.tick();
        // `RON_DEBUG_SITES=<lo>-<hi>`: each draw's owner, as the pairs' walks
        // print it.
        if crate::diff::harness::site_window().is_some_and(|(lo, hi)| (lo..=hi).contains(&at)) {
            for (label, who) in crate::diff::harness::attributed_sites(&built) {
                eprintln!("  f{at} {who}: {label}");
            }
        }
        crate::diff::harness::debug_watch(&built, at);
        // PROBE (item 4314): the map round the army.
        if std::env::var("RON_PROBE_MAP").ok().and_then(|v| v.parse::<i64>().ok()) == Some(at) {
            let mut grid = std::collections::BTreeMap::new();
            for x in built.sim.units.iter() {
                if x.alive() && x.on_map && x.owner == 1 {
                    let t = x.pos.tile();
                    grid.insert((t.x, t.y), x.index);
                }
            }
            for ty in 150..190 {
                let mut row = String::new();
                for tx in 60..100 {
                    let p = sim::Pos { x: tx, y: ty };
                    let c = if let Some(o) = grid.get(&(tx, ty)) {
                        format!("{:>3}", o)
                    } else if built.sim.world.is_tocean(p) {
                        "  ~".to_string()
                    } else {
                        "  .".to_string()
                    };
                    row.push_str(&c);
                }
                eprintln!("PROBE map {ty:>3} {row}");
            }
        }
        // PROBE (item 4314): near misses of the shore.
        if std::env::var_os("RON_PROBE_NEAR").is_some() && (4185..=4202).contains(&at) {
            for (i, x) in built.sim.units.iter().enumerate() {
                if !(x.alive() && x.on_map)
                    || built.sim.profile(sim::combat::Obj::Unit(i)).domain
                        != sim::territory::attrition::Domain::Land
                {
                    continue;
                }
                let t = x.pos.tile();
                let mut best = i32::MAX;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let q = sim::Pos { x: t.x + dx, y: t.y + dy };
                        if !built.sim.world.is_tocean(q) {
                            continue;
                        }
                        let (x0, y0) = (q.x * 192, q.y * 192);
                        let ddx = (x0 - x.pos.x).max(x.pos.x - (x0 + 191)).max(0);
                        let ddy = (y0 - x.pos.y).max(x.pos.y - (y0 + 191)).max(0);
                        best = best.min(ddx.max(ddy));
                    }
                }
                if best <= 40 {
                    eprintln!(
                        "PROBE f{at} near {}/{} at ({}, {}) gap {best} can {}",
                        x.owner,
                        x.index,
                        x.pos.x,
                        x.pos.y,
                        built.sim.unit_can_transport(i)
                    );
                }
            }
        }
        // PROBE (item 4314): land units on ocean.
        if std::env::var_os("RON_PROBE_WET").is_some() && (4195..=4215).contains(&at) {
            for (i, x) in built.sim.units.iter().enumerate() {
                if x.alive()
                    && x.on_map
                    && built.sim.world.is_tocean(x.pos.tile())
                    && built.sim.profile(sim::combat::Obj::Unit(i)).domain == sim::territory::attrition::Domain::Land
                {
                    eprintln!(
                        "PROBE f{at} landwet {}/{} #{i} at ({}, {}) can {} orders {:?}",
                        x.owner,
                        x.index,
                        x.pos.x,
                        x.pos.y,
                        built.sim.unit_can_transport(i),
                        x.orders.front().map(|o| format!("{:?}", o.body).chars().take(120).collect::<String>())
                    );
                }
            }
        }
        // PROBE (item 4314): tiles and transport flags around 1/5.
        if std::env::var("RON_PROBE_TILES").ok().and_then(|v| v.parse::<i64>().ok()) == Some(at) {
            for ty in 180..185 {
                let row: Vec<String> = (88..97)
                    .map(|tx| {
                        let p = sim::Pos { x: tx * 192 + 96, y: ty * 192 + 96 };
                        format!("{tx}:{:04x}", built.sim.world.tile_mask(p.tile()))
                    })
                    .collect();
                eprintln!("PROBE tiles y{ty} {}", row.join(" "));
            }
            for (i, x) in built.sim.units.iter().enumerate() {
                if x.alive() && x.owner == 1 && x.index <= 125 && !x.orders.is_empty() {
                    let t = x.pos.tile();
                    let mut wet = 0;
                    for dy in -2..=2 {
                        for dx in -2..=2 {
                            let q = sim::Pos { x: t.x + dx, y: t.y + dy };
                            if built.sim.world.is_tocean(q) {
                                wet += 1;
                            }
                        }
                    }
                    let wp = match x.orders.front().map(|o| &o.body) {
                        Some(sim::orders::Body::Move(m)) => Some(m.waypoint),
                        _ => None,
                    };
                    let top = x.path.last().map(|n| (n.to, n.flags, built.sim.world.is_tocean(n.to.tile())));
                    if let Some(sim::orders::Body::Move(m)) = x.orders.front().map(|o| &o.body) {
                        let nodes: Vec<String> = x
                            .path
                            .iter()
                            .map(|n| format!("({},{}){}{}", n.to.x, n.to.y, n.flags, if built.sim.world.is_tocean(n.to.tile()) { "W" } else { "" }))
                            .collect();
                        eprintln!(
                            "PROBE path 1/{} dest ({},{}) destwet {} orders {} nodes {}",
                            x.index,
                            m.dest.x,
                            m.dest.y,
                            built.sim.world.is_tocean(m.dest.tile()),
                            x.orders.len(),
                            nodes.join(" ")
                        );
                    }
                    if wet > 0 && wp.is_some() {
                        let w = wp.unwrap();
                        let d = (w.x - x.pos.x).abs().max((w.y - x.pos.y).abs());
                        eprintln!(
                            "PROBE wp 1/{} wp ({}, {}) wet {} dist {d} top {:?}",
                            x.index,
                            w.x,
                            w.y,
                            built.sim.world.is_tocean(w.tile()),
                            top
                        );
                    }
                    if wet > 0 {
                        eprintln!(
                            "PROBE wet 1/{} #{i} at ({}, {}) tile ({}, {}) wet {wet} can {} head {:?}",
                            x.index,
                            x.pos.x,
                            x.pos.y,
                            t.x,
                            t.y,
                            built.sim.unit_can_transport(i),
                            x.orders.front().map(|o| format!("{:?}", o.body).chars().take(160).collect::<String>())
                        );
                    }
                }
                if x.alive() && x.owner == 1 && x.index == 5 {
                    eprintln!(
                        "PROBE 1/5 #{i} auto {} never {} can {}",
                        x.auto_transport,
                        x.never_transport,
                        built.sim.unit_can_transport(i)
                    );
                }
            }
        }
        let n = built.sim.frame;
        if let Some((f, ours)) = built.frame_sites.last() {
            let theirs = tr.labels(*f);
            if count.is_none() && ours.len() != theirs.len() {
                count = Some(*f);
            }
            if sequence.is_none() && *ours != theirs {
                sequence = Some(*f);
            }
        }
        // PROBE (item 4314): `RON_WALK_PAST=<n>` walks n frames past the
        // count parting, for `RON_DEBUG_SITES`/`RON_DEBUG_UNIT` on ours.
        if let Some(c) = count {
            let past: i64 = std::env::var("RON_WALK_PAST")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            if n > c + past {
                break;
            }
            continue;
        }
        let Some(at) = blocks.iter().position(|&b| b == n) else {
            continue;
        };
        let raw = ix.read_frame(at).unwrap();
        let blog = Log::parse(&raw);
        for (who, prev) in prev.iter_mut().enumerate() {
            let Some(block) = blog.leader_block(n, who as i64) else {
                continue;
            };
            let leaves = diplomacy_leaves(&block);
            if let Some(p) = prev.as_ref() {
                for (k, &v) in &leaves {
                    if let Some(&b) = p.get(k)
                        && b != v
                    {
                        movers.push((n, who, k.clone(), b, v));
                    }
                }
            }
            let ours_leaves = our_diplomacy_leaves(&built, who);
            for (k, &t) in &leaves {
                if let Some(&m) = ours_leaves.get(k) {
                    leaves_seen.push((n, who, k.clone(), t, m));
                }
            }
            *prev = Some(leaves);
            let theirs = crate::diff::leader::theirs(&block);
            let mine: std::collections::BTreeMap<String, i64> =
                crate::diff::leader::rows(loaded, &built, who)
                    .into_iter()
                    .collect();
            crate::diff::held_probe::map("", n, who, &mine, &theirs);
            for (k, &m) in &mine {
                let leaf = k.split('[').next().unwrap_or(k);
                if PRINTED_ARRAYS.contains(&leaf) {
                    ours.insert((n, who, k.clone()), m);
                }
            }
            for (k, &t) in &theirs {
                if let Some(&m) = mine.get(k)
                    && m != t
                {
                    leader_parted.push((n, who, k.clone(), t, m));
                }
            }
        }
    }
    let (count, sequence) = (count.unwrap_or(last), sequence.unwrap_or(last));
    let mut row = String::new();
    if let Some((f, ours)) = built.frame_sites.iter().find(|(f, _)| *f == sequence) {
        let theirs = tr.labels(*f);
        let at = (0..ours.len().max(theirs.len()))
            .find(|&i| ours.get(i) != theirs.get(i))
            .unwrap_or(0);
        row = format!(
            "frame {f}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
            ours.len(),
            theirs.len(),
            ours.get(at),
            theirs.get(at)
        );
        if std::env::var("RON_DEBUG_SITES").is_ok() {
            for (i, l) in ours.iter().enumerate() {
                eprintln!("    ours  {i}: {l}");
            }
            for (i, l) in theirs.iter().enumerate() {
                eprintln!("    thrs  {i}: {l}");
            }
        }
    }
    let skipped = built
        .staged
        .as_ref()
        .map(|st| {
            st.applied
                .skipped
                .iter()
                .map(|((w, why), k)| format!("{k} {w}: {why}"))
                .collect()
        })
        .unwrap_or_default();
    Some(TakeWord {
        count,
        sequence,
        last,
        row,
        skipped,
        leader_parted,
        movers,
        leaves: leaves_seen,
        ours,
        seed,
    })
}

/// What a take stages at the head of the frame `built` is about to tick,
/// before the tick: a probe, the window's diplomacy and the script's
/// taunts, each on the frame its command ran.
pub(crate) fn stage_frame(
    built: &mut Built,
    seats: usize,
    probes: &[(i64, sim::Player)],
    diplomacy: &[TakeDiplo],
    taunts: &[TakeTaunt],
) {
    let at = built.sim.frame;
    // A probe: `Armies::diplo_change@006f30f0(w)` alone at the head of a
    // frame, the call item 4206 isolated the parting with.
    for &(f, w) in probes {
        if f == at {
            built.sim.armies_diplo_change(w);
        }
    }
    // The window's diplomacy, the human's commands read off the take's own
    // record ([`take_diplomacy`]): a Declare as `Leader::action_declare`, a
    // proposal as `Leader::action_treaty`, an accept as
    // `Leader::action_respond`. A computer's answer is its own offers arm's
    // (item 4159, step VII-c).
    for d in diplomacy {
        if d.frame == at {
            // The Mutual Attack check sends the toggle of the issuer's own
            // record (`IFaceDiploNeg::on_check_clicked@00803bd0`: `onoff =
            // console.dip[whom].attacks[whose] == 0`).
            let attack_on = built
                .sim
                .leaders
                .tribute
                .get(d.this as usize)
                .and_then(|t| t.dip.get(d.whom as usize))
                .and_then(|dip| dip.attacks.get(d.level as usize))
                .is_some_and(|&a| a == 0);
            let mut c = built.sim.command();
            match d.kind {
                TakeKind::Declare => c.action_declare(d.this, d.whom, d.level, false, false),
                TakeKind::Treaty => c.action_treaty(d.this, d.whom, d.level),
                TakeKind::Accept => c.action_respond(d.this, d.whom, 1),
                TakeKind::Attack => c.action_attack(d.this, d.whom, d.level, i32::from(attack_on)),
                TakeKind::Tribute(amount) => {
                    let good = sim::economy::Resource::ALL[d.level as usize];
                    c.action_offer(d.this, d.whom, good, amount)
                }
            }
        }
    }
    // The script's taunts ([`script_taunts`]): `process_chat`'s
    // `Leader::receive_taunt` on every slot, which refuses all but the
    // sender's live computer allies.
    for t in taunts {
        if t.frame == at {
            for k in 0..seats {
                built
                    .sim
                    .command()
                    .receive_taunt(k as sim::Player, t.request, t.sender);
            }
        }
    }
}

/// `OPTION_CHAT_ALLIES`'s key in `diplomacy4206.cmd`'s profile: F14, VK
/// 125 (the script's header; `--bindkey`).
const CHAT_ALLIES_VK: &str = "125";

/// The taunt requests the mint reads, by the order string the install's
/// `sound.xml` gives a taunt (`id`), and `rise.pdb`'s `TauntRequest`:
/// `TAUNT_FOOD` 1 to `TAUNT_OIL` 5 and `TAUNT_HELP` 16. That `NEED_FOOD` is
/// 1 is the record's (run1575's `last_taunt[0]` 1 on block 967); the rest
/// follow the enum's names.
const TAUNT_ORDERS: [(&str, i32); 6] = [
    ("NEED_FOOD", 1),
    ("NEED_TIMBER", 2),
    ("NEED_WEALTH", 3),
    ("NEED_METAL", 4),
    ("NEED_OIL", 5),
    ("HELP", 16),
];

/// One taunt a take's script sends to allies: applied on `frame`, from
/// `sender`, as `request`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TakeTaunt {
    pub frame: i64,
    pub sender: sim::Player,
    pub request: i32,
}

/// **The taunt's carrier, minted from the script** (item 4159, step
/// VII-b2): the dump prints no chat line and no recording holds the
/// command, so each `@key 125` (chat to allies) whose typed text closes
/// with Return (`@char 13`) is one `Chat` command from the console's seat.
/// The text is the taunt's index, the install's `sound.xml` names its order
/// string, and [`TAUNT_ORDERS`] its request. It is applied one frame after
/// the Return's line, as every window command of the take is (the Declare
/// clicked on 985 is `process_declare 1 0 986` on the record), and
/// run1575's packet holds the receipt there: `incoming_taunt_frame[0]` 966,
/// 60 frames on from 906. A text that names no order string panics: the
/// mint reads none it cannot map.
pub(crate) fn script_taunts(name: &str) -> Option<Vec<TakeTaunt>> {
    let inst = crate::testenv::install()?;
    let path = format!(
        "{}/../../tools/gamelog/coverage/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let sounds = std::fs::read_to_string(inst.root().join("data/sound.xml")).ok()?;
    let order = |index: &str| -> Option<String> {
        let at = sounds.find(&format!("<TAUNT index=\"{index}\" "))?;
        let tag = &sounds[at..at + sounds[at..].find('>')?];
        let id = tag.split("id=\"").nth(1)?.split('"').next()?;
        (!id.is_empty()).then(|| id.to_string())
    };
    let mut out = Vec::new();
    let mut open: Option<String> = None;
    for line in text.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        let (Some(f), Some(verb), Some(arg)) = (w.first(), w.get(1), w.get(2)) else {
            continue;
        };
        let Ok(frame) = f.parse::<i64>() else {
            continue;
        };
        match (*verb, open.as_mut()) {
            ("@key", _) if *arg == CHAT_ALLIES_VK => open = Some(String::new()),
            ("@char", Some(typed)) if *arg == "13" => {
                let id = order(typed).unwrap_or_else(|| panic!("{name}: taunt {typed:?}"));
                let request = TAUNT_ORDERS
                    .iter()
                    .find(|(n, _)| *n == id)
                    .unwrap_or_else(|| panic!("{name}: an unread order {id}"))
                    .1;
                out.push(TakeTaunt {
                    frame: frame + 1,
                    sender: ISSUER,
                    request,
                });
                open = None;
            }
            ("@char", Some(typed)) => {
                let c = arg.parse::<u32>().ok().and_then(char::from_u32);
                typed.push(c.unwrap_or_else(|| panic!("{name}: @char {arg}")));
            }
            _ => {}
        }
    }
    Some(out)
}

/// The coverage pair's frame zero re-read at `TEAM_STYLE 0`, `script`
/// staged, the window's `diplomacy` staged ([`take_diplomacy`]), walked
/// against `capture` ([`walk_take_from`]).
pub(crate) fn walk_style_zero(
    capture: (&str, &str),
    script: crate::golden::Script,
    diplomacy: &[TakeDiplo],
) -> Option<TakeWord> {
    let inst = crate::testenv::install()?;
    let pair = crate::diff::coverage_pair::COVERAGE;
    let (Some(path), Some(sib), Some(pasture)) =
        (dump(pair.long.0), dump(pair.start), trace(pair.long.1))
    else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", pair.start);
        return None;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    borrow_pasture(&mut init, &pasture);
    // The style-0 lobby: the field, and `Leader::init`'s peace between the
    // two seats (`diplos` 1 both ways).
    for kv in init.game_info.iter_mut() {
        if kv.0 == "TEAM_STYLE" {
            kv.1 = "0";
        }
    }
    for l in init.leaders.iter_mut() {
        if (0..2).contains(&l.who) {
            let other = 1 - l.who as usize;
            if let Some(d) = l.diplos.get_mut(other) {
                *d = 1;
            }
        }
    }
    let built =
        crate::diff::setup::with_staged_script(script, || build_sim(&loaded, &init, Tuning::RON));
    assert_eq!(built.sim.ai.lobby.team_style, 0, "the lobby reads style 0");
    walk_take_from(&loaded, built, capture, 2, &[], diplomacy, &[])
}

/// Print a [`TakeWord`]: the word, the staging's refusals, each leader
/// row's first parting and every mover.
pub(crate) fn report(name: &str, w: &TakeWord) {
    eprintln!(
        "{name}: count {} sequence {} of {}; {}",
        w.count, w.sequence, w.last, w.row
    );
    for s in &w.skipped {
        eprintln!("  skipped {s}");
    }
    match w.seed {
        Some((f, t, m)) => {
            eprintln!("{name}: head word first parted on {f}: theirs {t:#010x} ours {m:#010x}")
        }
        None => eprintln!("{name}: head word agreed on every frame walked"),
    }
    let mut firsts: std::collections::BTreeMap<(usize, String), (i64, i64, i64)> =
        Default::default();
    for (n, who, k, t, m) in &w.leader_parted {
        firsts.entry((*who, k.clone())).or_insert((*n, *t, *m));
    }
    eprintln!(
        "{name} leader rows parted: {} records",
        w.leader_parted.len()
    );
    for ((who, k), (n, t, m)) in &firsts {
        eprintln!("  first {n} who={who} {k}: theirs {t} ours {m}");
    }
    for (n, who, k, b, a) in &w.movers {
        eprintln!("  mover {n} who={who} {k}: {b} -> {a}");
    }
    let mut leaf_firsts: std::collections::BTreeMap<(usize, &str), (i64, i64, i64)> =
        Default::default();
    for (n, who, k, t, m) in w.leaves.iter().filter(|l| l.3 != l.4) {
        leaf_firsts
            .entry((*who, k.as_str()))
            .or_insert((*n, *t, *m));
    }
    eprintln!(
        "{name} diplomacy leaves: {} compared, {} parted",
        w.leaves.len(),
        w.leaves.iter().filter(|l| l.3 != l.4).count()
    );
    for ((who, k), (n, t, m)) in &leaf_firsts {
        eprintln!("  leaf first {n} who={who} {k}: theirs {t} ours {m}");
    }
}

/// **The first act** (item 4206): run1078's own trace and dump against the
/// coverage pair's walk at style 0 with run1078's script
/// (`declare3460-diplomacy.cmd`) — how far our walk holds a style-0 game
/// before any take.
#[test]
fn run1078_walked_at_style_zero() {
    // Item 4159, step VII-b1: the human's Declare (`process_declare 1 0
    // 911`) staged off the record as `Sim::action_declare`.
    let diplomacy = take_diplomacy(RUN1078.0).unwrap_or_default();
    let Some(w) = walk_style_zero(
        RUN1078,
        coverage_script("declare3460-diplomacy.cmd"),
        &diplomacy,
    ) else {
        return;
    };
    report("run1078 at style 0", &w);
    let _pins = Pins::hold();
    // Item 4206: whole, 960 of 960. The original draws the same 961 frames
    // at style 0 as at style 1 (`rngcmp` run652 against run1078), the paid
    // Declare included; the record parted where the staging mapped no line:
    // `meet` before that item (`treaties[1]` from 901), the window's
    // Declare (`diplos`, `dow[1]`, and `pay_dow`'s −640 wealth, on 912).
    // Item 4159, step VII-b1: still whole with the Declare staged as
    // `Sim::action_declare` (a human's command, an input; entry 132 §1).
    pin_eq!(w.count, 960, "run1078's draw count at style 0");
    pin_eq!(w.sequence, 960, "run1078's draw sequence at style 0");
    // The Declare's leaves, `diplos` and the wealth agree on every block:
    // the record moves `dow[1]` 0 → 1 on 912 (block A, a first count, so no
    // `blacken`; at peace, so no `broke_alliance`).
    let movers: Vec<String> = w
        .movers
        .iter()
        .filter(|(_, _, k, _, _)| declaration_row(k))
        .map(|(n, who, k, b, a)| format!("{n} {who} {k} {b}->{a}"))
        .collect();
    pin_eq!(
        movers,
        vec!["912 0 dow[1] 0->1".to_string()],
        "run1078's declaration movers"
    );
    let parted: Vec<String> = w
        .leader_parted
        .iter()
        .filter(|(_, _, k, _, _)| {
            declaration_row(k) || k == "bucket[2:wealth]" || k.starts_with("diplos[")
        })
        .map(|(n, who, k, t, m)| format!("{n} {who} {k}: theirs {t} ours {m}"))
        .collect();
    pin_eq!(
        parted,
        Vec::<String>::new(),
        "run1078's declaration, diplos and wealth, ours against the record"
    );
}

/// The sea melee's lobby (run1482: East Indies, three seats, the
/// Information age held, Deathmatch), the one three-seat lobby on East
/// Indies our walk holds whole (`diff::sea_melee`).
pub(crate) const SEA_START: &str = crate::diff::sea_melee::MELEE_START;

/// run1482's frame zero re-read at `TEAM_STYLE 0` — the field, and
/// `Leader::init`'s peace between every two seats (`diplos` 1) — with
/// `capture`'s own pasture, `script` staged, walked against `capture`
/// ([`walk_take_from`]).
pub(crate) fn walk_sea_style_zero(
    capture: (&str, &str),
    script: crate::golden::Script,
    probes: &[(i64, sim::Player)],
    diplomacy: &[TakeDiplo],
    taunts: &[TakeTaunt],
) -> Option<TakeWord> {
    let (loaded, built) = stand_up_sea_style_zero(capture, script)?;
    walk_take_from(&loaded, built, capture, 3, probes, diplomacy, taunts)
}

/// [`walk_sea_style_zero`]'s game at frame zero: run1482's start re-read
/// at style 0 with `capture`'s pasture and `script` staged.
pub(crate) fn stand_up_sea_style_zero(
    capture: (&str, &str),
    script: crate::golden::Script,
) -> Option<(crate::load::Loaded, Built)> {
    let inst = crate::testenv::install()?;
    let (Some(start), Some(pasture)) = (dump(SEA_START), take_trace(capture.1)) else {
        eprintln!(
            "skipping: no {SEA_START} or {} (set RON_GAMELOG_DIR)",
            capture.1
        );
        return None;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&start);
    let log = Log::parse(&text);
    let own = log.initial().expect("a start dump");
    let mut init = log.initial().expect("a start dump");
    borrow_from_siblings(&mut init, &[&own]);
    borrow_pasture(&mut init, &pasture);
    for kv in init.game_info.iter_mut() {
        if kv.0 == "TEAM_STYLE" {
            kv.1 = "0";
        }
    }
    for l in init.leaders.iter_mut() {
        if (0..3).contains(&l.who) {
            for (j, d) in l.diplos.iter_mut().enumerate().take(3) {
                if j as i64 != l.who {
                    *d = 1;
                }
            }
        }
    }
    let built =
        crate::diff::setup::with_staged_script(script, || build_sim(&loaded, &init, Tuning::RON));
    assert_eq!(built.sim.ai.lobby.team_style, 0, "the lobby reads style 0");
    Some((loaded, built))
}

/// Which of the window's commands a [`TakeDiplo`] is: the three
/// `CommandPackage::process_*` lines the dump prints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TakeKind {
    /// `process_declare <whom> <treaty> <frame>`:
    /// `Leader::action_declare(who, whom, treaty, 0, 0)`.
    Declare,
    /// `process_treaty <whom> <treaty> <frame>`:
    /// `Leader::action_treaty(who, whom, treaty)`.
    Treaty,
    /// `process_accept <whom> <frame>`: `Leader::action_respond(who, whom,
    /// 1)`.
    Accept,
    /// `process_propose_attack <whom> <whose> <frame>`:
    /// `Leader::action_attack(who, whom, whose, onoff)`, `onoff` the
    /// window's toggle (item 4298).
    Attack,
    /// `process_tribute <whom> <good> <frame>`: `Leader::action_offer(who,
    /// whom, good, amount)` (`CommandPackage::process_tribute@00946d70`).
    /// The line prints no amount (its `logToMemory` twin carries it), so
    /// the amount is read off the issuer's own record ([`tribute_amount`]);
    /// the good is the [`TakeDiplo`]'s `level`.
    Tribute(i32),
}

/// One of the window's commands the take's own record shows, on the frame
/// it ran: issued by `this` (the console's seat) to `whom`, `level` the
/// treaty asked or declared (1 for an accept), the slot to attack, or the
/// good offered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TakeDiplo {
    pub frame: i64,
    pub this: sim::Player,
    pub whom: sim::Player,
    pub level: i32,
    pub kind: TakeKind,
}

/// The seat that issues the window's commands: the console's leader, who 0
/// in every lobby these takes use. The dump's `process_*` line prints
/// `whom` and not `who` (`CommandPackage::process_declare@009472b0`'s
/// `Log::say`; its `logToMemory` twin carries `who`).
const ISSUER: sim::Player = 0;

/// **The window's commands, read off a take's own dump**: each
/// `CommandPackage::process_*` line the record prints is the console
/// human's command on that frame, an input as a script's line is
/// (`docs/DECISIONS.md` entry 132 §1).
///
/// - `process_declare <whom> <treaty> <frame>`: `Leader::action_declare
///   (issuer, whom, treaty, 0, 0)` (item 4159, step VII-b1).
/// - `process_treaty <whom> <treaty> <frame>`: `Leader::action_treaty
///   (issuer, whom, treaty)`, the proposal (step VII-c).
/// - `process_accept <whom> <frame>`: `Leader::action_respond(issuer, whom,
///   1)`, the human committing to the proposal, which forces `whom`'s next
///   `Leader::diplomacy` pass (step VII-c).
///
/// No computer's answer is read here. Until step VII-c an accept that
/// moved `whom`'s `diplos` on the record was staged as the answerer's
/// `set_diplo` (item 4207): the oracle's own output fed back, retired now
/// that the offers arm computes it.
pub(crate) fn take_diplomacy(capture: &str) -> Option<Vec<TakeDiplo>> {
    use std::io::BufRead;
    let path = take_file(capture)?;
    let file = std::fs::File::open(&path).ok()?;
    let mut out = Vec::new();
    for line in std::io::BufReader::new(file).split(b'\n') {
        let line = line.ok()?;
        let line = String::from_utf8_lossy(&line);
        let t = line.trim();
        let Some(verb) = [
            "process_declare",
            "process_treaty",
            "process_accept",
            "process_propose_attack",
            "process_tribute",
        ]
        .into_iter()
        .find(|v| t.starts_with(&format!("{v} "))) else {
            continue;
        };
        let nums: Vec<i64> = t[verb.len()..]
            .split_whitespace()
            .filter_map(|x| x.parse().ok())
            .collect();
        let (kind, whom, level, frame) = match (verb, nums.as_slice()) {
            ("process_declare", &[whom, treaty, frame]) => (TakeKind::Declare, whom, treaty, frame),
            ("process_treaty", &[whom, treaty, frame]) => (TakeKind::Treaty, whom, treaty, frame),
            ("process_accept", &[whom, frame]) => (TakeKind::Accept, whom, 1, frame),
            ("process_propose_attack", &[whom, whose, frame]) => {
                (TakeKind::Attack, whom, whose, frame)
            }
            ("process_tribute", &[whom, good, frame]) => {
                let amount = tribute_amount(&path, ISSUER, whom, good, frame);
                (TakeKind::Tribute(amount), whom, good, frame)
            }
            _ => panic!("{capture}: an unread {verb} line: {nums:?}"),
        };
        out.push(TakeDiplo {
            frame,
            this: ISSUER,
            whom: whom as sim::Player,
            level: level as i32,
            kind,
        });
    }
    Some(out)
}

/// `GetKeyState`'s shift (`VK_SHIFT`, 16) and alt (`VK_MENU`, 18), the two
/// modifiers `IFaceDiploNeg::on_button_clicked@00804320` reads.
const VK_SHIFT: i64 = 16;
const VK_MENU: i64 = 18;

/// **A tribute's amount, minted from the take's script, held against the
/// record's** (item 4298): the record prints a `process_tribute` line
/// without its amount, which is the window's input — `IFaceDiploNeg::on_button_clicked@00804320` offers
/// 100 a click, 500 with shift held, and the giver's whole stock with alt.
/// The click is the script's line one frame before the command's (the
/// window's commands run on the frame after their click, as `process_*`
/// lines show); a key is held over `<from> @hold <vk> <to>`, both ends in.
/// An alt-held click is refused here: no take uses one. Each amount
/// [`tribute_amount`] read off the issuer's record must be the script's,
/// so the two readings check each other.
pub(crate) fn check_tribute_amounts(diplomacy: &[TakeDiplo], script: &str) {
    let path = format!(
        "{}/../../tools/gamelog/coverage/{script}",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let holds: Vec<(i64, i64, i64)> = text
        .lines()
        .filter_map(|l| {
            let w: Vec<&str> = l.split_whitespace().collect();
            match w.as_slice() {
                [from, "@hold", vk, to] => {
                    Some((vk.parse().ok()?, from.parse().ok()?, to.parse().ok()?))
                }
                _ => None,
            }
        })
        .collect();
    let held = |vk: i64, f: i64| {
        holds
            .iter()
            .any(|&(k, a, b)| k == vk && (a..=b).contains(&f))
    };
    for d in diplomacy {
        let TakeKind::Tribute(amount) = d.kind else {
            continue;
        };
        let click = d.frame - 1;
        assert!(
            !held(VK_MENU, click),
            "{script}: an alt-held tribute on {click}"
        );
        let minted = if held(VK_SHIFT, click) { 500 } else { 100 };
        assert_eq!(
            amount, minted,
            "{script}: the tribute of {} off the record",
            d.frame
        );
    }
}

/// **A tribute's amount, off the issuer's own record** (item 4159, step
/// 4159r19): `process_tribute <whom> <good> <frame>` prints no amount, and
/// `Leader::action_offer@006d1780` adds it to the issuer's
/// `dip[whom].offers[good]`. The command runs after block `frame` is
/// printed and before block `frame + 1` (the line sits between them), so
/// the amount is that slot on block `frame + 1` less block `frame`. A
/// human's command is an input (`docs/DECISIONS.md` entry 132 §1); the
/// French side of the record is not read. Panics on a record that does not
/// print both blocks, or on a slot that did not move.
pub(crate) fn tribute_amount(
    path: &str,
    issuer: sim::Player,
    whom: i64,
    good: i64,
    frame: i64,
) -> i32 {
    let mut ix = crate::capture::indexed::IndexedCapture::open(path).unwrap();
    let blocks: Vec<i64> = ix.frames().iter().map(|f| f.number).collect();
    let mut slot = |n: i64| -> i64 {
        let at = blocks
            .iter()
            .position(|&b| b == n)
            .unwrap_or_else(|| panic!("{path}: no block {n} for the tribute of {frame}"));
        let raw = ix.read_frame(at).unwrap();
        let log = Log::parse(&raw);
        let block = log
            .leader_block(n, i64::from(issuer))
            .unwrap_or_else(|| panic!("{path}: no who {issuer} on block {n}"));
        let dip = block
            .kids("DIPLOMACY")
            .nth(whom as usize)
            .unwrap_or_else(|| panic!("{path}: no DIPLOMACY {whom} on block {n}"));
        dip.all("offers[scan]")
            .get(good as usize)
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or_else(|| panic!("{path}: no offers[{good}] on block {n}"))
    };
    let amount = slot(frame + 1) - slot(frame);
    assert_ne!(amount, 0, "{path}: the tribute of {frame} moved no offer");
    amount as i32
}

/// A take named by `RON_TAKE=<gamelog>,<trace>` walked at style 0 on the
/// sea melee's lobby with `diplomacy4206.cmd` (item 4206's recons).
#[test]
fn diplomacy_take_walked_by_name() {
    let Ok(v) = std::env::var("RON_TAKE") else {
        return;
    };
    let (g, t) = v.split_once(',').expect("RON_TAKE=<gamelog>,<trace>");
    // `RON_TAKE_SCRIPT=<path>`: another staging, for a probe of which
    // line's absence parts the walk.
    let script = match std::env::var("RON_TAKE_SCRIPT") {
        Ok(p) => crate::golden::Script::read(std::path::Path::new(&p)).unwrap(),
        Err(_) => coverage_script("diplomacy4206.cmd"),
    };
    // `RON_TAKE_DIPLO_CHANGE=<frame>:<who>[,…]`: the probe's frames.
    let probes: Vec<(i64, sim::Player)> = std::env::var("RON_TAKE_DIPLO_CHANGE")
        .ok()
        .map(|v| {
            v.split(',')
                .filter_map(|p| {
                    let (f, w) = p.split_once(':')?;
                    Some((f.parse().ok()?, w.parse().ok()?))
                })
                .collect()
        })
        .unwrap_or_default();
    // `RON_TAKE_NO_DIPLOMACY`: leave the window's diplomacy unstaged.
    let diplomacy = if std::env::var_os("RON_TAKE_NO_DIPLOMACY").is_some() {
        Vec::new()
    } else {
        let d = take_diplomacy(g).unwrap_or_default();
        // `RON_TAKE_SCRIPT_NAME=<coverage cmd>`: the script each tribute's
        // amount off the record is held against.
        if let Ok(n) = std::env::var("RON_TAKE_SCRIPT_NAME") {
            check_tribute_amounts(&d, &n);
        }
        d
    };
    eprintln!("{g}: staged {diplomacy:?}");
    // `RON_TAKE_NO_TAUNT`: leave the script's taunts unstaged.
    let taunts = if std::env::var_os("RON_TAKE_NO_TAUNT").is_some() {
        Vec::new()
    } else {
        script_taunts("diplomacy4206.cmd").unwrap_or_default()
    };
    let Some(w) = walk_sea_style_zero((g, t), script, &probes, &diplomacy, &taunts) else {
        return;
    };
    report(g, &w);
}

/// **run1575, the take** (item 4206, `docs/RUNS.md` run1575): run1482's
/// lobby at `TEAM_STYLE 0` with `diplomacy4206.cmd`, `cover=1`, the level-9
/// leader record 896..1150 and a packet at logger frame 1090.
pub(crate) const RUN1575: (&str, &str) = (
    "gamelog-run1575-eastindies-melee-style0-diplomacy-take-packet-1090.txt",
    "rontrace-run1575.log",
);

/// The frame the French answer the human's peace (`process_accept 1 1027`):
/// the human's accept (`Leader::action_respond(0, 1, 1)`) forces the French
/// pass, `Leader::diplomacy@006bc950`'s offers arm accepts, `Leader::
/// action_respond@006d03c0`'s treaty arm calls `Leader::set_diplo
/// @006ec6a0(French, human, 1)`, and that ends in `Armies::diplo_change
/// @006f30f0(1)`.
pub(crate) const RUN1575_ACCEPT: i64 = 1027;

/// **The window's commands on run1575's record** ([`take_diplomacy`]): the
/// human's two Declares, the peace asked on 1021 and accepted on 1027, the
/// alliance asked on 1041 and accepted on 1051, each the human's own
/// command. Until item 4159's step VII-c the accept of 1027 was staged as
/// the French `set_diplo` read off the record's `diplos` move, and the
/// proposals were not staged at all.
#[test]
fn run1575_s_window_diplomacy_is_read_off_its_record() {
    let _pins = Pins::hold();
    let Some(d) = take_diplomacy(RUN1575.0) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", RUN1575.0);
        return;
    };
    let got: Vec<(i64, sim::Player, sim::Player, i32, TakeKind)> = d
        .iter()
        .map(|d| (d.frame, d.this, d.whom, d.level, d.kind))
        .collect();
    pin_eq!(
        got,
        vec![
            (986, 0, 1, 1, TakeKind::Declare),
            (1001, 0, 1, 0, TakeKind::Declare),
            (1021, 0, 1, 1, TakeKind::Treaty),
            (RUN1575_ACCEPT, 0, 1, 1, TakeKind::Accept),
            (1041, 0, 1, 2, TakeKind::Treaty),
            (1051, 0, 1, 1, TakeKind::Accept),
        ],
        "the take's window commands"
    );
}

/// **The take's word** (items 4206, 4207, 4159): our walk of run1575 from
/// run1482's start re-read at style 0, the script staged, the chat minted,
/// and the human's window commands read off the take's own record
/// ([`take_diplomacy`]). Since item 4159's step VII-c no computer's answer
/// is staged: the French accept of 1027 is their offers arm's.
#[test]
fn run1575_s_word_is_pinned_as_found() {
    let _pins = Pins::hold();
    let Some(diplomacy) = take_diplomacy(RUN1575.0) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", RUN1575.0);
        return;
    };
    let Some(w) = walk_sea_style_zero(
        RUN1575,
        coverage_script("diplomacy4206.cmd"),
        &[],
        &diplomacy,
        &script_taunts("diplomacy4206.cmd").unwrap_or_default(),
    ) else {
        return;
    };
    report("run1575", &w);
    pin_eq!(w.last, 1150, "the take's last frame");
    // Item 4206, first walk: 1099, count and sequence together, 72 frames
    // after the accepted peace — theirs spent a guard's idle stand
    // (`Guy::set_anim+0x97a < Unit::do_guard+0x7f4`) at index 1 where ours
    // went on to `Farms::inc_time+0x1ae`. Item 4207: **whole**, 1150 of
    // 1150. `Sim::set_diplo` runs `Armies::diplo_change(this)` on every
    // level, and the staged accept's `this` is the French. The same walk
    // under the declaration-only rule holds this pin's old 1099 exactly
    // (count, sequence and row; item 4207's journal), so the staging alone
    // moves no draw: the rule does. Item 4159, step VII-c: **1150 of 1150
    // computed**, the staging retired. The human's proposal (1021) and
    // accept (1027) are staged as his commands, the accept forces the
    // French pass on 1027, and their offers arm answers it (a need of 0:
    // `S` is `good_deeds` 0, the shortfall's adjustments skipped while the
    // human holds the French gift of 966 as tribute). Before it, quoted
    // both ways (`docs/DECISIONS.md` entry 132 §1): 1150 staged, 1099
    // computed; with the arm's answer forced to refuse, this walk parts on
    // 1099 again (the journal's red first).
    pin_eq!(w.count, 1150, "the draw count's first parting");
    pin_eq!(w.sequence, 1150, "the draw sequence's first parting");
    pin_eq!(w.row.as_str(), "", "no parting");
}

/// **The parting is the French armies' `Armies::diplo_change` on the
/// accepted peace** (item 4206): the same walk with
/// `Sim::armies_diplo_change(1)` run at the head of the acceptance frame —
/// one call, and nothing else of the diplomacy carried — holds the take
/// whole; without it the walk parts 72 frames on. Kept as the isolation's
/// proof: since item 4207 the word itself is whole with no probe, the
/// call made by `Sim::set_diplo` ([`run1575_s_word_is_pinned_as_found`]).
#[test]
fn run1575_holds_whole_with_the_french_armies_processed_on_the_accept() {
    let _pins = Pins::hold();
    let Some(w) = walk_sea_style_zero(
        RUN1575,
        coverage_script("diplomacy4206.cmd"),
        &[(RUN1575_ACCEPT, 1)],
        &[],
        &[],
    ) else {
        return;
    };
    eprintln!(
        "run1575 with diplo_change(1) on {RUN1575_ACCEPT}: count {} sequence {} of {}",
        w.count, w.sequence, w.last
    );
    pin_eq!(w.count, 1150, "the draw count's first parting");
    pin_eq!(w.sequence, 1150, "the draw sequence's first parting");
}

/// **The take's movers, on its own record** (item 4206): every diplomacy
/// leaf of `docs/LEADERS.md` §7 to §9 the level-9 record prints that moves,
/// block on block, over 896..1150 — `(block, who, key, before, after)`.
#[test]
fn run1575_s_diplomacy_movers_are_the_script_s() {
    let _pins = Pins::hold();
    let Some(w) = walk_sea_style_zero(
        RUN1575,
        coverage_script("diplomacy4206.cmd"),
        &[(RUN1575_ACCEPT, 1)],
        &[],
        &[],
    ) else {
        return;
    };
    let movers: Vec<String> = w
        .movers
        .iter()
        .filter(|(_, _, k, _, _)| !k.starts_with("aggression"))
        .map(|(n, who, k, b, a)| format!("{n} {who} {k} {b}->{a}"))
        .collect();
    for m in &movers {
        eprintln!("  {m}");
    }
    // The console's meet (901) and ally (903); the taunt's gift (967, due
    // 60 frames after the chat of 905 and stamped the frame before its
    // block); the Declare on the ally (987: peace, `dow` 1, the alliance's
    // break); the Declare at peace (1002: war, `dow` 2, `blacken` 1); the
    // accepted peace (1028, `made_peace` both ways); the short alliance
    // (1052, the French `counteroffer`). Leaf for leaf, the movers of
    // run1078 (`dow`), run1086 (`last_taunt`, `taunt_frame`, `gift_stamp`)
    // and the leaves no capture had moved (`blacken`, `broke_alliance`,
    // `made_peace`, `counteroffer`, `tribute_stamp`).
    let want = [
        "901 0 treaties[1] 0->1",
        "901 1 treaties[0] 0->1",
        "903 0 diplos[1] 1->2",
        "903 1 diplos[0] 1->2",
        "967 0 tribute_stamp[1] 0->966",
        "967 1 gift_stamp[0] 0->966",
        "967 1 last_taunt[0] 0->1",
        "967 1 taunt_frame[0] 0->966",
        "987 0 broke_alliance[1] 0->986",
        "987 0 diplos[1] 2->1",
        "987 0 dow[1] 0->1",
        "987 1 diplos[0] 2->1",
        "1002 0 blacken 0->1",
        "1002 0 diplos[1] 1->0",
        "1002 0 dow[1] 1->2",
        "1002 1 diplos[0] 1->0",
        "1028 0 diplos[1] 0->1",
        "1028 0 made_peace[1] 0->1027",
        "1028 1 diplos[0] 0->1",
        "1028 1 made_peace[0] 0->1027",
        "1052 1 counteroffer[0] 0->1",
    ];
    pin_eq!(
        movers,
        want.iter().map(|m| m.to_string()).collect::<Vec<_>>(),
        "the take's movers"
    );
}

/// The last block of run1575 no staged decision precedes (`docs/DECISIONS.md`
/// entry 132 §1). Until item 4159's step VII-c it was 1026, the French
/// accept of 1027 read off the record; since the offers arm computes it,
/// no decision is staged and the whole take is held, 1150 of 1150 computed.
pub(crate) const RUN1575_HELD: i64 = 1150;

/// Whether a leader row is one of the declaration's (`docs/LEADERS.md`
/// §7.1): `blacken`, `dow[t]`, `broke_alliance[t]`.
fn declaration_row(k: &str) -> bool {
    k == "blacken" || k.starts_with("dow[") || k.starts_with("broke_alliance[")
}

/// **The human's two Declares, ours against the record** (item 4159, step
/// VII-b1): run1575 walked with the window's diplomacy off its record
/// ([`take_diplomacy`]), the Declares through `Sim::action_declare`. The
/// record moves the three leaves on 987 (`dow[1]` 0 → 1, `broke_alliance[1]`
/// 0 → 986: the Declare on the ally, turned into peace) and 1002 (`dow[1]`
/// 1 → 2, `blacken` 0 → 1: the Declare at peace), each inside the held
/// window; `pay_dow` takes the "Cancel Alliance" and "Declare War" crafts'
/// price off the human's wealth on the same blocks. Pinned: the record's
/// movers of the three, and every parting of the three over the whole walk
/// and of the wealth bucket through [`RUN1575_HELD`].
#[test]
fn run1575_s_declarations_are_ours() {
    let _pins = Pins::hold();
    let Some(diplomacy) = take_diplomacy(RUN1575.0) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", RUN1575.0);
        return;
    };
    let Some(w) = walk_sea_style_zero(
        RUN1575,
        coverage_script("diplomacy4206.cmd"),
        &[],
        &diplomacy,
        &script_taunts("diplomacy4206.cmd").unwrap_or_default(),
    ) else {
        return;
    };
    let movers: Vec<String> = w
        .movers
        .iter()
        .filter(|(_, _, k, _, _)| declaration_row(k))
        .map(|(n, who, k, b, a)| format!("{n} {who} {k} {b}->{a}"))
        .collect();
    pin_eq!(
        movers,
        vec![
            "987 0 broke_alliance[1] 0->986".to_string(),
            "987 0 dow[1] 0->1".to_string(),
            "1002 0 blacken 0->1".to_string(),
            "1002 0 dow[1] 1->2".to_string(),
        ],
        "the record's declaration movers"
    );
    let parted: Vec<String> = w
        .leader_parted
        .iter()
        .filter(|(n, _, k, _, _)| {
            declaration_row(k) || (k == "bucket[2:wealth]" && *n <= RUN1575_HELD)
        })
        .map(|(n, who, k, t, m)| format!("{n} {who} {k}: theirs {t} ours {m}"))
        .collect();
    for p in &parted {
        eprintln!("  parted {p}");
    }
    pin_eq!(
        parted,
        Vec::<String>::new(),
        "the declaration's leaves and wealth, ours against the record"
    );
}

/// Whether a leader row is one the accepted peace moves (`docs/LEADERS.md`
/// §7.1, §8.2): `made_peace[t]`, `diplos[t]`, and the deal records'
/// `dip.treaty[t]`.
fn peace_row(k: &str) -> bool {
    k.starts_with("made_peace[") || k.starts_with("diplos[") || k.starts_with("dip.treaty[")
}

/// **The French answer to the human's peace, ours against the record**
/// (item 4159, step VII-c): run1575 walked with the human's window commands
/// off its record and nothing of the French staged. The human asks a peace
/// on 1021 (`action_treaty`: `treaty` 1 on both records, block 1022) and
/// commits to it on 1027 (`action_respond`, which forces the French pass);
/// the French offers arm accepts on 1027, the treaty arm sets the peace
/// both ways and stamps `made_peace` 1027 on both, and both records are
/// cleared (block 1028). The alliance asked on 1041 (`treaty` 2, block
/// 1042) is answered short on 1051 (step VII-d,
/// [`run1575_s_alliance_is_found_short`]); the counteroffer withdraws both
/// agreements and the records keep `treaty` 2, as theirs do. Pinned:
/// the record's movers of the three rows, and no parting of them over the
/// whole walk.
#[test]
fn run1575_s_peace_is_the_french_answer() {
    let _pins = Pins::hold();
    let (Some(diplomacy), Some(taunts)) = (
        take_diplomacy(RUN1575.0),
        script_taunts("diplomacy4206.cmd"),
    ) else {
        eprintln!(
            "skipping: no {} or install (RON_GAMELOG_DIR, RON_INSTALL)",
            RUN1575.0
        );
        return;
    };
    let Some(w) = walk_sea_style_zero(
        RUN1575,
        coverage_script("diplomacy4206.cmd"),
        &[],
        &diplomacy,
        &taunts,
    ) else {
        return;
    };
    let movers: Vec<String> = w
        .movers
        .iter()
        .filter(|(n, _, k, _, _)| peace_row(k) && *n > RUN1575_ACCEPT - 10)
        .map(|(n, who, k, b, a)| format!("{n} {who} {k} {b}->{a}"))
        .collect();
    pin_eq!(
        movers,
        vec![
            "1028 0 diplos[1] 0->1".to_string(),
            "1028 0 made_peace[1] 0->1027".to_string(),
            "1028 1 diplos[0] 0->1".to_string(),
            "1028 1 made_peace[0] 0->1027".to_string(),
        ],
        "the record's peace movers from 1018 (`diplomacy_leaves` reads no deal record)"
    );
    let parted: Vec<String> = w
        .leader_parted
        .iter()
        .filter(|(_, _, k, _, _)| peace_row(k))
        .map(|(n, who, k, t, m)| format!("{n} {who} {k}: theirs {t} ours {m}"))
        .collect();
    for p in &parted {
        eprintln!("  parted {p}");
    }
    pin_eq!(
        parted,
        Vec::<String>::new(),
        "the peace's rows, ours against the record"
    );
    pin_eq!(w.count, 1150, "the draw count, computed");
    pin_eq!(w.sequence, 1150, "the draw sequence, computed");
}

/// **The comparison can part** (item 4159, step VII-c): the same walk with
/// the human's proposal and accept left unstaged — the Declares and the
/// taunt only — never asks the French, who never answer. The deal records
/// part on 1022 and 1042 (`treaty`), the peace and its stamps on 1028, and
/// the word on 1099, as the staging's isolation found (item 4206). The
/// stub's first partings, per row, and its word.
#[test]
fn run1575_s_peace_parts_without_the_human_s_proposal() {
    let _pins = Pins::hold();
    let (Some(diplomacy), Some(taunts)) = (
        take_diplomacy(RUN1575.0),
        script_taunts("diplomacy4206.cmd"),
    ) else {
        eprintln!(
            "skipping: no {} or install (RON_GAMELOG_DIR, RON_INSTALL)",
            RUN1575.0
        );
        return;
    };
    let declares: Vec<TakeDiplo> = diplomacy
        .into_iter()
        .filter(|d| d.kind == TakeKind::Declare)
        .collect();
    let Some(w) = walk_sea_style_zero(
        RUN1575,
        coverage_script("diplomacy4206.cmd"),
        &[],
        &declares,
        &taunts,
    ) else {
        return;
    };
    let mut firsts: Vec<String> = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (n, who, k, t, m) in &w.leader_parted {
        if peace_row(k) && seen.insert((*who, k.clone())) {
            firsts.push(format!("{n} {who} {k}: theirs {t} ours {m}"));
        }
    }
    pin_eq!(
        firsts,
        vec![
            "1022 0 dip.treaty[1]: theirs 1 ours -1".to_string(),
            "1022 1 dip.treaty[0]: theirs 1 ours -1".to_string(),
            "1028 0 diplos[1]: theirs 1 ours 0".to_string(),
            "1028 0 made_peace[1]: theirs 1027 ours 0".to_string(),
            "1028 1 diplos[0]: theirs 1 ours 0".to_string(),
            "1028 1 made_peace[0]: theirs 1027 ours 0".to_string(),
        ],
        "the stub's first partings"
    );
    pin_eq!(w.count, 1099, "the stub's draw count");
}

/// Whether a leader row is one the alliance found short moves
/// (`docs/LEADERS.md` §7.2, §8.2): `counteroffer[t]`, and the deal records'
/// `dip.agree[t]`.
fn alliance_row(k: &str) -> bool {
    k.starts_with("counteroffer[") || k.starts_with("dip.agree[")
}

/// **The French answer to the human's alliance, ours against the record**
/// (item 4159, step VII-d): the human asks an alliance on 1041
/// (`action_treaty`, `treaty` 2 on both records, block 1042) and commits to
/// it on 1051, which forces the French pass. Their offers arm runs the
/// alliance's gates (`docs/LEADERS.md` §7.2): no ally of theirs, no victory
/// clock, a human asking, so the price `(epoch + 1) · 200` at peace comes
/// off `S` (`good_deeds` 0). The shortfall's `aggression == 0` arm adds
/// nothing (not weak; no peace terms for a `treaty` of 2), the need is the
/// price, and with no give of theirs it is the counteroffer:
/// `counteroffer[0]` 1 and both agreements withdrawn (block 1052). Pinned:
/// the record's movers of `counteroffer`, and no parting of it or of
/// `agree` over the whole walk.
#[test]
fn run1575_s_alliance_is_found_short() {
    let _pins = Pins::hold();
    let (Some(diplomacy), Some(taunts)) = (
        take_diplomacy(RUN1575.0),
        script_taunts("diplomacy4206.cmd"),
    ) else {
        eprintln!(
            "skipping: no {} or install (RON_GAMELOG_DIR, RON_INSTALL)",
            RUN1575.0
        );
        return;
    };
    let Some(w) = walk_sea_style_zero(
        RUN1575,
        coverage_script("diplomacy4206.cmd"),
        &[],
        &diplomacy,
        &taunts,
    ) else {
        return;
    };
    let movers: Vec<String> = w
        .movers
        .iter()
        .filter(|(_, _, k, _, _)| alliance_row(k))
        .map(|(n, who, k, b, a)| format!("{n} {who} {k} {b}->{a}"))
        .collect();
    pin_eq!(
        movers,
        vec!["1052 1 counteroffer[0] 0->1".to_string()],
        "the record's counteroffer movers (`diplomacy_leaves` reads no deal record)"
    );
    let parted: Vec<String> = w
        .leader_parted
        .iter()
        .filter(|(_, _, k, _, _)| alliance_row(k))
        .map(|(n, who, k, t, m)| format!("{n} {who} {k}: theirs {t} ours {m}"))
        .collect();
    for p in &parted {
        eprintln!("  parted {p}");
    }
    pin_eq!(
        parted,
        Vec::<String>::new(),
        "the alliance's rows, ours against the record"
    );
    pin_eq!(w.count, 1150, "the draw count, computed");
    pin_eq!(w.sequence, 1150, "the draw sequence, computed");
}

/// **The comparison can part** (item 4159, step VII-d): the same walk with
/// the human's alliance asked on 1041 and committed on 1051 left unstaged.
/// The French are never asked, so `counteroffer[0]` stays 0 where theirs
/// is 1 from 1052, and the deal records part on 1042 (`treaty`). The
/// stub's first partings of the alliance's rows and of `dip.treaty`.
#[test]
fn run1575_s_alliance_parts_without_the_human_s_proposal() {
    let _pins = Pins::hold();
    let (Some(diplomacy), Some(taunts)) = (
        take_diplomacy(RUN1575.0),
        script_taunts("diplomacy4206.cmd"),
    ) else {
        eprintln!(
            "skipping: no {} or install (RON_GAMELOG_DIR, RON_INSTALL)",
            RUN1575.0
        );
        return;
    };
    let early: Vec<TakeDiplo> = diplomacy.into_iter().filter(|d| d.frame < 1041).collect();
    let Some(w) = walk_sea_style_zero(
        RUN1575,
        coverage_script("diplomacy4206.cmd"),
        &[],
        &early,
        &taunts,
    ) else {
        return;
    };
    let mut firsts: Vec<String> = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (n, who, k, t, m) in &w.leader_parted {
        if (alliance_row(k) || k.starts_with("dip.treaty[")) && seen.insert((*who, k.clone())) {
            firsts.push(format!("{n} {who} {k}: theirs {t} ours {m}"));
        }
    }
    for f in &firsts {
        eprintln!("  first {f}");
    }
    pin_eq!(
        firsts,
        vec![
            "1042 0 dip.treaty[1]: theirs 2 ours -1".to_string(),
            "1042 1 dip.treaty[0]: theirs 2 ours -1".to_string(),
            "1052 1 counteroffer[0]: theirs 1 ours 0".to_string(),
        ],
        "the stub's first partings"
    );
}

/// Whether a leader row is one of the taunt's answer's printed rows
/// (`docs/LEADERS.md` §8.1, §9.1): `tribute_stamp[t]`, `gift_stamp[t]`,
/// `last_taunt[t]`, `taunt_frame[t]`.
fn taunt_row(k: &str) -> bool {
    [
        "tribute_stamp[",
        "gift_stamp[",
        "last_taunt[",
        "taunt_frame[",
    ]
    .iter()
    .any(|p| k.starts_with(p))
}

/// The rows the taunt's gift moves beside its leaves, compared through
/// [`RUN1575_HELD`]: the commit's `agendas` bit, and the food each side's
/// bucket holds.
fn gift_row(k: &str) -> bool {
    k.starts_with("agendas[") || k == "bucket[0:food]"
}

/// The script's taunt ([`script_taunts`]): one `NEED_FOOD` from the
/// console's seat, applied on 906.
#[test]
fn diplomacy4206_s_taunt_is_minted_from_its_chat() {
    let _pins = Pins::hold();
    let Some(t) = script_taunts("diplomacy4206.cmd") else {
        eprintln!("skipping: no install (RON_INSTALL)");
        return;
    };
    pin_eq!(
        t,
        vec![TakeTaunt {
            frame: 906,
            sender: 0,
            request: 1
        }],
        "the script's taunts"
    );
}

/// **The taunt's answer, ours against the record** (item 4159, step
/// VII-b2): run1575 walked with the window's diplomacy off its record and
/// the script's chat minted as its `Chat` command. The French drain the
/// request on 966, sixty frames after its receipt, and give a third of
/// their food: on block 967 the human's `tribute_stamp[1]` and the French
/// `gift_stamp[0]` read 966, the French `last_taunt[0]` 1 and
/// `taunt_frame[0]` 966, and the commit's `agendas` bit is the human's
/// `agendas[1]` 4. Pinned: the record's movers of the four, and no parting
/// of them over the whole walk, nor of `agendas` and the food buckets
/// through [`RUN1575_HELD`].
#[test]
fn run1575_s_taunt_is_answered_as_the_original_s() {
    let _pins = Pins::hold();
    let (Some(diplomacy), Some(taunts)) = (
        take_diplomacy(RUN1575.0),
        script_taunts("diplomacy4206.cmd"),
    ) else {
        eprintln!(
            "skipping: no {} or install (RON_GAMELOG_DIR, RON_INSTALL)",
            RUN1575.0
        );
        return;
    };
    let Some(w) = walk_sea_style_zero(
        RUN1575,
        coverage_script("diplomacy4206.cmd"),
        &[],
        &diplomacy,
        &taunts,
    ) else {
        return;
    };
    let movers: Vec<String> = w
        .movers
        .iter()
        .filter(|(_, _, k, _, _)| taunt_row(k))
        .map(|(n, who, k, b, a)| format!("{n} {who} {k} {b}->{a}"))
        .collect();
    pin_eq!(
        movers,
        vec![
            "967 0 tribute_stamp[1] 0->966".to_string(),
            "967 1 gift_stamp[0] 0->966".to_string(),
            "967 1 last_taunt[0] 0->1".to_string(),
            "967 1 taunt_frame[0] 0->966".to_string(),
        ],
        "the record's taunt movers"
    );
    let parted: Vec<String> = w
        .leader_parted
        .iter()
        .filter(|(n, _, k, _, _)| taunt_row(k) || (gift_row(k) && *n <= RUN1575_HELD))
        .map(|(n, who, k, t, m)| format!("{n} {who} {k}: theirs {t} ours {m}"))
        .collect();
    for p in &parted {
        eprintln!("  parted {p}");
    }
    pin_eq!(
        parted,
        Vec::<String>::new(),
        "the taunt's leaves, agendas and food, ours against the record"
    );
}

/// **The comparison can part** (item 4159, step VII-b2): the same walk with
/// the chat left unminted holds every taunt leaf at its reset value, and
/// each of the four parts on block 967 with the commit's `agendas` bit and
/// the gift's food — the stub's partings, first per row.
#[test]
fn run1575_s_taunt_parts_without_its_chat() {
    let _pins = Pins::hold();
    let Some(diplomacy) = take_diplomacy(RUN1575.0) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", RUN1575.0);
        return;
    };
    let Some(w) = walk_sea_style_zero(
        RUN1575,
        coverage_script("diplomacy4206.cmd"),
        &[],
        &diplomacy,
        &[],
    ) else {
        return;
    };
    let mut firsts: Vec<String> = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (n, who, k, t, m) in &w.leader_parted {
        if (taunt_row(k) || gift_row(k)) && seen.insert((*who, k.clone())) {
            firsts.push(format!("{n} {who} {k}: theirs {t} ours {m}"));
        }
    }
    pin_eq!(
        firsts,
        vec![
            "967 0 agendas[1]: theirs 4 ours 0".to_string(),
            "967 0 bucket[0:food]: theirs 121994 ours 120085".to_string(),
            "967 0 tribute_stamp[1]: theirs 966 ours 0".to_string(),
            "967 1 bucket[0:food]: theirs 6580 ours 9870".to_string(),
            "967 1 gift_stamp[0]: theirs 966 ours 0".to_string(),
            "967 1 last_taunt[0]: theirs 1 ours 0".to_string(),
            "967 1 taunt_frame[0]: theirs 966 ours 0".to_string(),
        ],
        "the stub's first partings"
    );
}

/// **run1629, the attack deal's take** (item 4298, `docs/RUNS.md` run1629):
/// run1575's line to its 1081 Escape, then `attack4298.cmd`'s deal — the
/// Russians met on 1085, the alliance unticked, the human asking the French
/// to attack the Russians with 5,000 each of metal and oil — `cover=1`, the
/// level-9 leader record 1074..1301 and a packet at logger frame 1190.
pub(crate) const RUN1629: (&str, &str) = (
    "gamelog-run1629-eastindies-melee-style0-attack-deal-take-packet-1190.txt",
    "rontrace-run1629.log",
);

/// run1629's window commands, each tribute's amount off the record held
/// against the script's ([`check_tribute_amounts`]).
pub(crate) fn run1629_diplomacy() -> Option<Vec<TakeDiplo>> {
    let d = take_diplomacy(RUN1629.0)?;
    check_tribute_amounts(&d, "attack4298.cmd");
    Some(d)
}

/// **run1629's word** (item 4298): our walk of the take from run1482's
/// start re-read at style 0, `attack4298.cmd` staged, the chat minted, the
/// human's window commands read off the take's own record — the attack
/// asked (`process_propose_attack 1 2 1131`), twenty tributes and the
/// accept of 1186 — and every diplomacy leaf ours carries set against the
/// record on every block.
#[test]
fn run1629_s_word_is_pinned_as_found() {
    let _pins = Pins::hold();
    let Some(diplomacy) = run1629_diplomacy() else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", RUN1629.0);
        return;
    };
    let Some(w) = walk_sea_style_zero(
        RUN1629,
        coverage_script("attack4298.cmd"),
        &[],
        &diplomacy,
        &script_taunts("attack4298.cmd").unwrap_or_default(),
    ) else {
        return;
    };
    report("run1629", &w);
    pin_eq!(w.last, 1300, "the take's last frame");
    // Item 4298: whole, 1300 of 1300, with the attack asked, the tributes
    // offered and the French answer computed by their offers arm (the
    // attack terms, `Sim::attack_terms`).
    pin_eq!(w.count, 1300, "the draw count's first parting");
    pin_eq!(w.sequence, 1300, "the draw sequence's first parting");
    pin_eq!(w.row.as_str(), "", "no parting");
    let movers: Vec<String> = w
        .movers
        .iter()
        .map(|(n, who, k, b, a)| format!("{n} {who} {k} {b}->{a}"))
        .collect();
    pin_eq!(
        movers,
        vec![
            "1086 0 treaties[2] 0->1".to_string(),
            "1086 2 treaties[0] 0->1".to_string(),
            "1187 0 diplos[2] 1->0".to_string(),
            "1187 0 dow[2] 0->1".to_string(),
            "1187 0 gift_stamp[1] 0->1186".to_string(),
            "1187 0 hire_stamp[2] 0->1186".to_string(),
            "1187 0 hire_who[2] -1->1".to_string(),
            "1187 0 tributes[2] 0->640".to_string(),
            "1187 1 counteroffer[0] 1->0".to_string(),
            "1187 1 diplos[2] 1->0".to_string(),
            "1187 1 good_deeds[0] 0->6139".to_string(),
            "1187 1 hire_stamp[2] 0->1186".to_string(),
            "1187 1 hire_who[2] -1->0".to_string(),
            "1187 2 diplos[0] 1->0".to_string(),
            "1187 2 diplos[1] 1->0".to_string(),
        ],
        "the take's diplomacy movers, on its own record"
    );
    // **Every diplomacy leaf ours carries, against the record on every
    // block** (1074..1300, three leaders): none parts. The three leaves item
    // 4298 releases (`hire_stamp`, `hire_who`, `tributes`) agree off their
    // reset on every block from 1187 — the hire of 1186 and the attack
    // price's residue — and `good_deeds[0]` (the French, `−need`) with them.
    let parted: Vec<String> = w
        .leaves
        .iter()
        .filter(|l| l.3 != l.4)
        .map(|(n, who, k, t, m)| format!("{n} {who} {k}: theirs {t} ours {m}"))
        .collect();
    pin_eq!(
        parted,
        Vec::<String>::new(),
        "the diplomacy leaves, ours against the record"
    );
    pin_eq!(w.leaves.len(), 59247, "the diplomacy leaf rows compared");
    let off_reset = |leaf: &str| {
        w.leaves
            .iter()
            .filter(|(_, _, k, t, m)| {
                k.starts_with(&format!("{leaf}[")) && t == m && *t != -i64::from(leaf == "hire_who")
            })
            .count()
    };
    pin_eq!(
        [
            off_reset("hire_stamp"),
            off_reset("hire_who"),
            off_reset("tributes"),
            off_reset("good_deeds")
        ],
        [228, 228, 114, 114],
        "each leaf's rows off its reset, agreeing (114 blocks, 1187..1300)"
    );
}

/// **run1628, the peace bought with metal** (item 4159, step 4159r19;
/// `docs/RUNS.md` run1628, `docs/DECISIONS.md` entry 137 §1): recon4206-4
/// filed as is, run1575's game with four tribute clicks before the accept.
/// The capture is read where it was taken, under `RON_GOLDEN_DIR` (default
/// `~/ron-golden`): its dump and trace, `None` where either is missing.
pub(crate) fn run1628() -> Option<(String, String)> {
    let home = std::env::var("HOME").ok()?;
    let dir = std::env::var("RON_GOLDEN_DIR").unwrap_or(format!("{home}/ron-golden"));
    let map = format!("{dir}/recon4206-4/map-18");
    let (g, t) = (format!("{map}/gamelog.txt"), format!("{map}/rontrace.log"));
    (std::path::Path::new(&g).is_file() && std::path::Path::new(&t).is_file()).then_some((g, t))
}

/// run1628's block the French answer is first printed on: the human's
/// `process_accept 1 1031` commits, the French pass runs on 1031 and
/// accepts, and the record moves the French `good_deeds[0]` 0 → 1328 on
/// 1032.
pub(crate) const RUN1628_DEED: i64 = 1032;

/// The French `good_deeds[0]` the record prints on every block from
/// [`RUN1628_DEED`] to 1150.
pub(crate) const RUN1628_GOOD_DEEDS: i64 = 1328;

/// run1628 walked as run1575 is ([`walk_sea_style_zero`],
/// `diplomacy4206.cmd`'s console lines and taunt, which recon4206-4's own
/// command file repeats on the same frames), with `diplomacy` staged.
fn walk_run1628(diplomacy: &[TakeDiplo]) -> Option<TakeWord> {
    let Some((g, t)) = run1628() else {
        eprintln!("skipping: no recon4206-4 under RON_GOLDEN_DIR (run1628)");
        return None;
    };
    walk_sea_style_zero(
        (&g, &t),
        coverage_script("diplomacy4206.cmd"),
        &[],
        diplomacy,
        &script_taunts("diplomacy4206.cmd").unwrap_or_default(),
    )
}

/// **The window's commands on run1628's record** ([`take_diplomacy`]):
/// run1575's through the peace asked on 1021, then four tributes of metal
/// (good 4) on 1024 to 1027, the accept on 1031 and the alliance asked on
/// 1061. Each tribute's 500 is read off the human's own `dip[1].offers[4]`
/// (0, 500, 1,000, 1,500, 2,000 on blocks 1024 to 1028).
#[test]
fn run1628_s_window_diplomacy_is_read_off_its_record() {
    let _pins = Pins::hold();
    let Some((g, _)) = run1628() else {
        eprintln!("skipping: no recon4206-4 under RON_GOLDEN_DIR (run1628)");
        return;
    };
    let d = take_diplomacy(&g).expect("run1628's record");
    let got: Vec<(i64, sim::Player, sim::Player, i32, TakeKind)> = d
        .iter()
        .map(|d| (d.frame, d.this, d.whom, d.level, d.kind))
        .collect();
    pin_eq!(
        got,
        vec![
            (986, 0, 1, 1, TakeKind::Declare),
            (1001, 0, 1, 0, TakeKind::Declare),
            (1021, 0, 1, 1, TakeKind::Treaty),
            (1024, 0, 1, 4, TakeKind::Tribute(500)),
            (1025, 0, 1, 4, TakeKind::Tribute(500)),
            (1026, 0, 1, 4, TakeKind::Tribute(500)),
            (1027, 0, 1, 4, TakeKind::Tribute(500)),
            (1031, 0, 1, 1, TakeKind::Accept),
            (1061, 0, 1, 2, TakeKind::Treaty),
        ],
        "run1628's window commands"
    );
}

/// **`good_deeds`, computed** (item 4159, step 4159r19; `docs/LEADERS.md`
/// §7.2, §8.5): run1628 walked with the human's window commands off its
/// record, the four tributes as `Leader::action_offer`. The human commits
/// 2,000 metal on 1031 and the French offers arm values the peace: the
/// accept's `good_deeds[0] −= need`, a negative need, so the French
/// `good_deeds[0]` reads [`RUN1628_GOOD_DEEDS`] from [`RUN1628_DEED`] on.
/// Pinned: the record's mover, ours on block 1032, and no `good_deeds`
/// parting anywhere on the walk.
#[test]
fn run1628_s_good_deeds_are_the_french_answer() {
    let _pins = Pins::hold();
    let Some((g, _)) = run1628() else {
        eprintln!("skipping: no recon4206-4 under RON_GOLDEN_DIR (run1628)");
        return;
    };
    let diplomacy = take_diplomacy(&g).expect("run1628's record");
    let Some(w) = walk_run1628(&diplomacy) else {
        return;
    };
    report("run1628", &w);
    let deeds: Vec<String> = w
        .movers
        .iter()
        .filter(|(_, _, k, _, _)| k.starts_with("good_deeds["))
        .map(|(n, who, k, b, a)| format!("{n} {who} {k} {b}->{a}"))
        .collect();
    pin_eq!(
        deeds,
        vec![format!(
            "{RUN1628_DEED} 1 good_deeds[0] 0->{RUN1628_GOOD_DEEDS}"
        )],
        "run1628's good_deeds movers"
    );
    let key = "good_deeds[0]".to_string();
    let ours_on_the_deed = w.ours.get(&(RUN1628_DEED, 1, key.clone())).copied();
    pin_eq!(
        ours_on_the_deed,
        Some(RUN1628_GOOD_DEEDS),
        "our French good_deeds[0] on block 1032"
    );
    let held = w
        .ours
        .iter()
        .filter(|((n, who, k), _)| *n >= RUN1628_DEED && *who == 1 && *k == key)
        .count();
    // The exchange beside it (`docs/LEADERS.md` §8.5): both metal buckets
    // (the French +1,020, `scale_tribute` at the human's `pct` 51), the
    // human's `gift_stamp[1]` 1031, the French `tribute_stamp[0]` left at 0
    // (a computer receiver over 3,000 metal), `tributes` back to 0.
    let exchange = |k: &str| {
        ["good_deeds[", "gift_stamp[", "tribute_stamp[", "tributes["]
            .iter()
            .any(|p| k.starts_with(p))
            || k == "bucket[4:metal]"
    };
    let parted: Vec<String> = w
        .leader_parted
        .iter()
        .filter(|(_, _, k, _, _)| exchange(k))
        .map(|(n, who, k, t, m)| format!("{n} {who} {k}: theirs {t} ours {m}"))
        .collect();
    eprintln!("run1628: ours on {held} French blocks from {RUN1628_DEED}");
    pin_eq!(held, 119, "our French good_deeds[0] blocks, 1032 to 1150");
    pin_eq!(
        parted,
        Vec::<String>::new(),
        "run1628's good_deeds and exchange partings"
    );
    let on_the_deed: Vec<String> = w
        .movers
        .iter()
        .filter(|(n, _, k, _, _)| *n == RUN1628_DEED && exchange(k))
        .map(|(n, who, k, b, a)| format!("{n} {who} {k} {b}->{a}"))
        .collect();
    pin_eq!(
        on_the_deed,
        vec![
            "1032 0 gift_stamp[1] 0->1031".to_string(),
            format!("{RUN1628_DEED} 1 good_deeds[0] 0->{RUN1628_GOOD_DEEDS}"),
        ],
        "the record's exchange movers on 1032"
    );
}

/// **The comparison can fail**: run1628 walked without the four tributes
/// (every other window command staged). The French find the peace asked
/// for nothing worth exactly nothing, as on run1575's 1027, accept it, and
/// leave `good_deeds[0]` at 0: it parts on [`RUN1628_DEED`].
#[test]
fn run1628_s_good_deeds_part_without_the_tributes() {
    let _pins = Pins::hold();
    let Some((g, _)) = run1628() else {
        eprintln!("skipping: no recon4206-4 under RON_GOLDEN_DIR (run1628)");
        return;
    };
    let diplomacy: Vec<TakeDiplo> = take_diplomacy(&g)
        .expect("run1628's record")
        .into_iter()
        .filter(|d| !matches!(d.kind, TakeKind::Tribute(_)))
        .collect();
    let Some(w) = walk_run1628(&diplomacy) else {
        return;
    };
    let first = w
        .leader_parted
        .iter()
        .find(|(_, _, k, _, _)| k.starts_with("good_deeds["))
        .map(|(n, who, k, t, m)| format!("{n} {who} {k}: theirs {t} ours {m}"));
    pin_eq!(
        first,
        Some(format!(
            "{RUN1628_DEED} 1 good_deeds[0]: theirs {RUN1628_GOOD_DEEDS} ours 0"
        )),
        "the stub's first good_deeds parting"
    );
}

/// recon4300-2, item 4300's unfiled recon of run1575's lobby at
/// `DIFFICULTY 4` with no command script (`docs/journal/2026-11-16-item-4300-first-act.md`),
/// under `RON_GOLDEN_DIR`: its gamelog and trace.
pub(crate) fn recon4300_2() -> Option<(String, String)> {
    let home = std::env::var("HOME").ok()?;
    let dir = std::env::var("RON_GOLDEN_DIR").unwrap_or(format!("{home}/ron-golden"));
    let map = format!("{dir}/recon4300-2/map-18");
    let (g, t) = (format!("{map}/gamelog.txt"), format!("{map}/rontrace.log"));
    (std::path::Path::new(&g).is_file() && std::path::Path::new(&t).is_file()).then_some((g, t))
}

/// recon4300-2's word at `DIFFICULTY 4` (item 4300): 5 at the first act
/// (run1482's frame words installed over a Tough game), 1750 at the third
/// (those words dropped; the Russian `2/23` left idle by a recruit whose
/// ring found no spot), **4202** at the fourth (a `QUEUE_NEW` swarm clears
/// only once a spot is found, [`sim::Sim::swarm_around_new`]).
pub(crate) const RECON4300_2_WORD: i64 = 4202;

/// A recon of run1575's lobby with no command script, walked from
/// run1482's start re-read at style 0 and at `difficulty`, nothing staged
/// (item 4300's first act). A re-read difficulty drops run1482's own frame
/// words (the third act).
pub(crate) fn walk_recon4300(capture: (&str, &str), difficulty: &'static str) -> Option<TakeWord> {
    let (g, t) = capture;
    let inst = crate::testenv::install()?;
    let (Some(start), Some(pasture)) = (dump(SEA_START), take_trace(t)) else {
        eprintln!("skipping: no {SEA_START} or {t}");
        return None;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&start);
    let log = Log::parse(&text);
    let own = log.initial().expect("a start dump");
    let mut init = log.initial().expect("a start dump");
    borrow_from_siblings(&mut init, &[&own]);
    borrow_pasture(&mut init, &pasture);
    let mut reread = false;
    for kv in init.game_info.iter_mut() {
        if kv.0 == "TEAM_STYLE" {
            kv.1 = "0";
        }
        if kv.0 == "DIFFICULTY" {
            reread = kv.1 != difficulty;
            kv.1 = difficulty;
        }
    }
    // A re-read difficulty makes run1482's own frame words another game's:
    // the setup does not read the difficulty, the first frame does, so its
    // frame-end words and clocks are dropped as a sibling's at another
    // lobby are (`borrow_from_siblings`, item 979). At the dump's own
    // difficulty they stay, as before.
    if reread && std::env::var_os("RON_RECON_KEEP_WORDS").is_none() {
        init.frame_seeds.clear();
        init.frame_guys.clear();
    }
    for l in init.leaders.iter_mut() {
        if (0..3).contains(&l.who) {
            for (j, d) in l.diplos.iter_mut().enumerate().take(3) {
                if j as i64 != l.who {
                    *d = 1;
                }
            }
        }
    }
    let script = crate::golden::Script::parse("");
    let built =
        crate::diff::setup::with_staged_script(script, || build_sim(&loaded, &init, Tuning::RON));
    assert_eq!(built.sim.ai.lobby.team_style, 0, "the lobby reads style 0");
    eprintln!("difficulty {}", built.sim.ai.lobby.difficulty);
    walk_take_from(&loaded, built, (g, t), 3, &[], &[], &[])
}

/// **Item 4300's first act, a probe** (its journal
/// `docs/journal/2026-11-16-item-4300-first-act.md`): an unfiled recon
/// named by `RON_TAKE=<gamelog>,<trace>`, at `RON_RECON_DIFFICULTY`
/// (default 0).
#[test]
#[ignore = "a probe of an unfiled recon; item 4300's journal"]
fn recon4300_walked_at_its_difficulty() {
    let Ok(v) = std::env::var("RON_TAKE") else {
        return;
    };
    let (g, t) = v.split_once(',').expect("RON_TAKE=<gamelog>,<trace>");
    let difficulty: &'static str = Box::leak(
        std::env::var("RON_RECON_DIFFICULTY")
            .unwrap_or_else(|_| "0".to_string())
            .into_boxed_str(),
    );
    let Some(w) = walk_recon4300((g, t), difficulty) else {
        return;
    };
    report(g, &w);
}

/// **recon4300-2 at `DIFFICULTY 4` holds to [`RECON4300_2_WORD`]**, and
/// its head word agrees with the trace's on every frame walked (parked
/// 4300b2; `RON_RECON_KEEP_WORDS=1` puts run1482's frame-0 word back and
/// parts it on frame 1, the comparison's failing case).
#[test]
fn recon4300_2_walks_to_its_word_at_difficulty_4() {
    let _pins = Pins::hold();
    let Some((g, t)) = recon4300_2() else {
        eprintln!("skipping: no recon4300-2 under RON_GOLDEN_DIR");
        return;
    };
    let Some(w) = walk_recon4300((&g, &t), "4") else {
        return;
    };
    report(&g, &w);
    pin_eq!(
        (w.count, w.sequence),
        (RECON4300_2_WORD, RECON4300_2_WORD),
        "recon4300-2's word at DIFFICULTY 4"
    );
    pin_eq!(
        w.seed,
        None,
        "recon4300-2's head word parts nowhere before its word"
    );
}

/// **Item 4299's first act, a probe**: an unfiled recon of run1575's lobby
/// with the human seated at who 1 (`unattended_capture.py --human-who 1`,
/// `--dump-all-start`, no command script), walked twice against its own
/// trace: from its own full start, and from the lobby's built start
/// ([`from_lobby`], which reads the human off `PLAYER flags & 4`).
/// `RON_TAKE=<gamelog>,<trace>` names the recon; `RON_RECON_BUILT_ONLY`
/// skips the first walk, and `RON_RECON_UNSEATED` builds the second from
/// run1482's player rows (the human at who 0), the control that parts.
#[test]
#[ignore = "a probe of an unfiled recon; item 4299's journal"]
fn recon4299_walked_reseated() {
    let Ok(v) = std::env::var("RON_TAKE") else {
        return;
    };
    let (g, t) = v.split_once(',').expect("RON_TAKE=<gamelog>,<trace>");
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(tr)) = (take_file(g), take_trace(t)) else {
        eprintln!("skipping: no {g} or {t}");
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let own = log.initial().expect("a start dump");
    let mut init = log.initial().expect("a start dump");
    for p in &init.players {
        eprintln!("PLAYER {p:?}");
    }
    borrow_from_siblings(&mut init, &[&own]);
    borrow_pasture(&mut init, &tr);
    if std::env::var_os("RON_RECON_BUILT_ONLY").is_none() {
        let built = build_sim(&loaded, &init, Tuning::RON);
        let w = crate::diff::third::walk_word(built, &tr, "own start");
        eprintln!(
            "own start: count {} sequence {} of {}; {}",
            w.count, w.sequence, w.last, w.row
        );
    }
    // The control: run1482's lobby, the human at who 0, against this trace.
    let unseated = dump(SEA_START).map(|p| crate::capture::read(&p));
    let unseated = unseated.as_deref().map(Log::parse);
    let unseated = unseated.as_ref().and_then(|l| l.initial());
    let (rows, info) = match (&unseated, std::env::var_os("RON_RECON_UNSEATED")) {
        (Some(u), Some(_)) => (&u.players, &own.game_info),
        _ => (&own.players, &own.game_info),
    };
    let lobby = from_lobby(&loaded, info, rows, Tuning::RON);
    for (s, ok) in &lobby.steps {
        eprintln!("built step {s}: {ok}");
    }
    for n in &lobby.notes {
        eprintln!("built note: {n}");
    }
    let built = built_from_lobby(&loaded, lobby).0;
    let theirs: usize = tr.frames.iter().map(|(f, _)| tr.labels(*f).len()).sum();
    eprintln!("theirs: {} frames, {theirs} draws", tr.frames.len());
    let w = crate::diff::third::walk_word(built, &tr, "built start");
    eprintln!(
        "built start: count {} sequence {} of {}; {}",
        w.count, w.sequence, w.last, w.row
    );
}
