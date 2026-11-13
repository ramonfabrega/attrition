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
    let mut prev: Vec<Option<std::collections::BTreeMap<String, i64>>> = vec![None; seats];
    for _ in 0..last {
        let at = built.sim.frame;
        stage_frame(&mut built, seats, probes, diplomacy, taunts);
        built.tick();
        // `RON_DEBUG_SITES=<lo>-<hi>`: each draw's owner, as the pairs' walks
        // print it.
        if crate::diff::harness::site_window().is_some_and(|(lo, hi)| (lo..=hi).contains(&at)) {
            for (label, who) in crate::diff::harness::attributed_sites(&built) {
                eprintln!("  f{at} {who}: {label}");
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
        if count.is_some() {
            break;
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
            *prev = Some(leaves);
            let theirs = crate::diff::leader::theirs(&block);
            let mine: std::collections::BTreeMap<String, i64> =
                crate::diff::leader::rows(loaded, &built, who)
                    .into_iter()
                    .collect();
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
    // The window's diplomacy, staged off the take's own record
    // ([`take_diplomacy`]): the human's Declare as `Leader::action_declare`,
    // a computer's answer as `Leader::set_diplo` with the original's own
    // `this`.
    for d in diplomacy {
        if d.frame == at {
            if d.declare {
                built
                    .sim
                    .command()
                    .action_declare(d.this, d.whom, d.level, false, false);
            } else {
                built.sim.setup().set_diplo(d.this, d.whom, d.level);
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

/// One diplomacy call the take's own record shows, on the frame its
/// command ran: the human's `Leader::action_declare(this, whom, level, 0,
/// 0)` when `declare`, else a computer's `Leader::set_diplo(this, whom,
/// level)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TakeDiplo {
    pub frame: i64,
    pub this: sim::Player,
    pub whom: sim::Player,
    pub level: i32,
    /// The human's Declare, an input: `Sim::action_declare` computes it
    /// (item 4159, step VII-b1). A computer's answer read off the record
    /// is not (`docs/DECISIONS.md` entry 132 §1).
    pub declare: bool,
}

/// The seat that issues the window's commands: the console's leader, who 0
/// in every lobby these takes use. The dump's `process_*` line prints
/// `whom` and not `who` (`CommandPackage::process_declare@009472b0`'s
/// `Log::say`; its `logToMemory` twin carries `who`).
const ISSUER: sim::Player = 0;

/// **The window's diplomacy, read off a take's own dump** (item 4207): the
/// `Leader::set_diplo` calls our staging cannot reach, with the original's
/// own `this`, so that `Sim::set_diplo` runs the armies of the leader the
/// original's does.
///
/// - `process_declare <whom> <treaty> <frame>` is `CommandPackage::
///   process_declare@009472b0`'s `Leader::action_declare(issuer, whom,
///   treaty, 0, 0)` — staged as that, a human's command and so an input
///   (item 4159, step VII-b1; before it, the `set_diplo` it ends in).
/// - `process_accept <whom> <frame>` is the issuer's `action_respond(whom,
///   1)`: it commits the proposal and forces `whom`'s next
///   `Leader::diplomacy` pass, whose offers arm decides — a decision this
///   crate does not carry (the offers arm is a SEAM in
///   `sim::leaders::diplomacy`). So the answer is read off the record: when
///   `whom`'s `diplos[issuer]` moves between the block of the accept's frame
///   and the next, the answer was `action_respond`'s treaty arm, and it is
///   staged as `set_diplo(whom, issuer, <the new level>)` — `whom` as
///   `this`. An answer the record shows unmoved stages nothing.
///
/// This is staging from the oracle, not the AI's decision; VII-b ports the
/// offers arm and retires it.
pub(crate) fn take_diplomacy(capture: &str) -> Option<Vec<TakeDiplo>> {
    use std::io::BufRead;
    let path = take_file(capture)?;
    let file = std::fs::File::open(&path).ok()?;
    let mut lines: Vec<(String, Vec<i64>)> = Vec::new();
    for line in std::io::BufReader::new(file).split(b'\n') {
        let line = line.ok()?;
        let line = String::from_utf8_lossy(&line);
        let t = line.trim();
        let Some(verb) = ["process_declare", "process_accept"]
            .into_iter()
            .find(|v| t.starts_with(&format!("{v} ")))
        else {
            continue;
        };
        let nums: Vec<i64> = t[verb.len()..]
            .split_whitespace()
            .filter_map(|x| x.parse().ok())
            .collect();
        lines.push((verb.to_string(), nums));
    }
    let mut ix = crate::capture::indexed::IndexedCapture::open(&path).ok()?;
    let numbers: Vec<i64> = ix.frames().iter().map(|f| f.number).collect();
    let mut diplos = |n: i64, who: sim::Player, of: sim::Player| -> Option<i64> {
        let at = numbers.iter().position(|&b| b == n)?;
        let raw = ix.read_frame(at).ok()?;
        let blog = Log::parse(&raw);
        let block = blog.leader_block(n, i64::from(who))?;
        diplomacy_leaves(&block)
            .get(&format!("diplos[{of}]"))
            .copied()
    };
    let mut out = Vec::new();
    for (verb, nums) in lines {
        match (verb.as_str(), nums.as_slice()) {
            ("process_declare", &[whom, treaty, frame]) => out.push(TakeDiplo {
                frame,
                this: ISSUER,
                whom: whom as sim::Player,
                level: treaty as i32,
                declare: true,
            }),
            ("process_accept", &[whom, frame]) => {
                let whom = whom as sim::Player;
                let (Some(before), Some(after)) =
                    (diplos(frame, whom, ISSUER), diplos(frame + 1, whom, ISSUER))
                else {
                    continue;
                };
                if before != after {
                    out.push(TakeDiplo {
                        frame,
                        this: whom,
                        whom: ISSUER,
                        level: after as i32,
                        declare: false,
                    });
                }
            }
            _ => panic!("{capture}: an unread {verb} line: {nums:?}"),
        }
    }
    Some(out)
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
        take_diplomacy(g).unwrap_or_default()
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
/// `Leader::diplomacy@006bc950`'s offers arm accepts, `Leader::
/// action_respond@006d03c0`'s treaty arm calls `Leader::set_diplo
/// @006ec6a0(French, human, 1)`, and that ends in `Armies::diplo_change
/// @006f30f0(1)`.
pub(crate) const RUN1575_ACCEPT: i64 = 1027;

/// **The window's diplomacy on run1575's record** (item 4207,
/// [`take_diplomacy`]): the two Declares as the human's `set_diplo`, and
/// the accepted peace as the French one; the alliance asked for on 1041
/// and answered short on 1051 (`counteroffer`) stages nothing.
#[test]
fn run1575_s_window_diplomacy_is_read_off_its_record() {
    let _pins = Pins::hold();
    let Some(d) = take_diplomacy(RUN1575.0) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", RUN1575.0);
        return;
    };
    let got: Vec<(i64, sim::Player, sim::Player, i32, bool)> = d
        .iter()
        .map(|d| (d.frame, d.this, d.whom, d.level, d.declare))
        .collect();
    // Item 4159, step VII-b1: the two Declares are `action_declare`, the
    // accept stays the French `set_diplo`.
    pin_eq!(
        got,
        vec![
            (986, 0, 1, 1, true),
            (1001, 0, 1, 0, true),
            (RUN1575_ACCEPT, 1, 0, 1, false)
        ],
        "the take's staged diplomacy calls"
    );
}

/// **The take's word** (items 4206, 4207): our walk of run1575 from
/// run1482's start re-read at style 0, the script staged, and the window's
/// diplomacy staged off the take's own record ([`take_diplomacy`]) — the
/// taunt and the proposals' bookkeeping are not carried (`docs/LEADERS.md`
/// §7.4).
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
    // moves no draw: the rule does.
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
/// entry 132 §1): the French accept of 1027 is read off the record, so a
/// leaf leaves on a mover at or before this block. Quoted both ways, the
/// word is 1150 of 1150 staged and 1099 computed.
pub(crate) const RUN1575_HELD: i64 = 1026;

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
