//! **Orders minted through the order door** (item 4059, `docs/RECGAME.md`
//! §9): a script of a person's orders in the wire's own vocabulary —
//! `group` with `move_to`, with `queue_up` and with `build`, slot 0's,
//! inside the harness lobby's first 300 frames — minted as packages into
//! the file 4058 writes, and the same packages applied to the built start's
//! walk by [`crate::input::Stream::pump`]. One list of packages is the
//! single source of both sides: the file the original plays, and the
//! stream our walk takes.

use super::start_walk::recording::{LOBBY_DUMP, lobby_recording_with};
use super::start_walk::{Walked, debug_leaves, keyed, walk_with};
use super::testkit::*;

use super::*;
use crate::recgame::{Order, RecGame};
use crate::testenv::{dump, install};
use std::collections::BTreeSet;

pub(crate) use crate::script::{
    BUILD_FRAME, BUILDER, CAPITAL, CITIZEN, FARM_AT, FARM_TYPE as FARM, MOVE_FRAME, QUEUE_FRAME,
    SCOUT_OBJECT as SCOUT, SCOUT_TO, boxed, clicked, right_pressed, script,
};

/// The minted recording: run53's lobby with [`script`]'s packages.
pub(crate) fn minted(orders: &[Order]) -> Option<RecGame> {
    lobby_recording_with(orders).map(|(ours, ..)| ours)
}

/// The built start of run53's lobby (`docs/START.md` §7.4), as the
/// playback walks take it: `(loaded, built, players)`.
pub(crate) fn built_start() -> Option<(Loaded, Built, usize)> {
    let inst = install()?;
    let lobby = dump(LOBBY_DUMP)?;
    let loaded = crate::load::load(&inst).unwrap();
    let mut source = crate::capture::indexed::IndexedCapture::open(&lobby).unwrap();
    let (built, players) = source
        .with_replay_initial(|init| {
            let players = player_count(&init);
            let ours = from_lobby(&loaded, &init.game_info, &init.players, Tuning::RON);
            (built_from_lobby(&loaded, ours).0, players)
        })
        .unwrap();
    Some((loaded, built, players))
}

/// Tick `built` to `frame`, the stream's packages applied before each tick.
pub(crate) fn tick_to(built: &mut Built, stream: Option<&mut crate::input::Stream>, frame: i64) {
    let mut stream = stream;
    while built.sim.frame < frame {
        if let Some(s) = stream.as_deref_mut() {
            s.apply(built.sim.frame as i32, built);
        }
        built.tick();
    }
}

/// The objects whose leaves part between two `Sim`s, by `who/o`, and the
/// top-level fields that part beside them — the "places" two walks differ.
fn parted_places(a: &Sim, b: &Sim) -> BTreeSet<String> {
    let la = keyed(a, debug_leaves(&format!("{a:#?}")));
    let lb = keyed(b, debug_leaves(&format!("{b:#?}")));
    let ma: std::collections::HashMap<&str, &str> =
        la.iter().map(|(p, v)| (p.as_str(), v.as_str())).collect();
    let mb: std::collections::HashMap<&str, &str> =
        lb.iter().map(|(p, v)| (p.as_str(), v.as_str())).collect();
    let place = |p: &str| -> String {
        if let Some(close) = p.find('}')
            && (p.starts_with(".units{") || p.starts_with(".buildings{"))
        {
            return p[..=close].to_string();
        }
        // The first two components: `.economy.ledgers`, `.rng.seed`.
        let mut cut = p.len();
        let mut dots = 0;
        for (i, c) in p.char_indices().skip(1) {
            if c == '.' || c == '[' || c == '{' {
                dots += 1;
                if dots == 2 {
                    cut = i;
                    break;
                }
            }
        }
        p[..cut].to_string()
    };
    let mut out = BTreeSet::new();
    for (p, v) in &la {
        if mb.get(p.as_str()) != Some(&v.as_str()) {
            out.insert(place(p));
        }
    }
    for (p, _) in &lb {
        if !ma.contains_key(p.as_str()) {
            out.insert(place(p));
        }
    }
    out
}

/// **Score 1: the minted packages round-trip and name the built start's
/// objects.** The script's packages, minted into the lobby's file, written
/// and parsed back: every package equal, stamps contiguous from 1, `valid`
/// on the first only, each order's payload decoding to the script's
/// commands. And the objects, types and site the script names are the
/// built start's: the scout, the capital, a citizen; the Farm and the
/// Citizen available to slot 0, affordable, and the site clear on our own
/// placement check. With `RON_RCX_OUT=<file>` it writes the `.rcx` the
/// playback takes open.
#[test]
fn the_minted_packages_round_trip_and_name_the_built_start_s_objects() {
    let Some((_, built, _)) = built_start() else {
        eprintln!("skipping: no install or no {LOBBY_DUMP} (set RON_GAMELOG_DIR)");
        return;
    };
    let Some(rec) = minted(&script()) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "minted").expect("the minted file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "the file re-writes byte for byte"
    );
    assert_eq!(back.packages, rec.packages);
    let heads: Vec<(i32, i32, i32, u32)> = back
        .packages
        .iter()
        .map(|p| (p.frame, p.play, p.valid, p.stamp))
        .collect();
    assert_eq!(
        heads,
        vec![
            (0, 0, 1, 1),
            (MOVE_FRAME, 0, 0, 2),
            (QUEUE_FRAME, 0, 0, 3),
            (BUILD_FRAME, 0, 0, 4),
            (24_000, 0, 0, 5),
        ],
        "frame, play, valid, stamp"
    );
    for ((f, play, cmds), p) in script().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "the package on frame {f}"
        );
    }
    assert_eq!(
        back.packages
            .iter()
            .map(|p| p.data.len())
            .collect::<Vec<_>>(),
        vec![0x21, 5 + 0x16, 5 + 9, 5 + 0x19, 0],
        "each package's size: the group's 3 + 2 and the command's own"
    );
    // The objects, read off the built start.
    let s = &built.sim;
    let name = |who: sim::Player, o: i16| {
        let u = s.unit_by_o(who, o).unwrap();
        built.unit_type_names[s.units[u].ty.unwrap()].clone()
    };
    assert_eq!(name(0, SCOUT), "Scout");
    assert_eq!(name(0, BUILDER), "Citizen");
    let capital = s.building_by_o(0, CAPITAL).unwrap();
    let farm = (FARM - crate::load::BASE_BUILDTYPES) as usize;
    assert_eq!(
        s.tables.build_types[s.buildings[capital].ty.unwrap()].ident,
        sim::cities::build::Ident::Village
    );
    assert_eq!(
        s.tables.build_types[farm].ident,
        sim::cities::build::Ident::Farm
    );
    assert_eq!(s.tables.build_types[farm].tree, Some(FARM as usize));
    assert_eq!(s.type_avail(0, FARM as usize), sim::tech::AVAILABLE);
    assert_eq!(built.type_index[built.unit_tree[0]], CITIZEN);
    assert_eq!(built.unit_type_names[0], "Citizen");
    assert_eq!(
        s.blocked_site(Some(0), farm, FARM_AT, None),
        sim::cities::place::Blocked::Clear,
        "the Farm's site, on our placement check"
    );
    let price = s.building_price(0, farm);
    let stock = s.economy.ledgers[0].bucket;
    assert!(
        price.iter().zip(stock).all(|(p, b)| *p <= b),
        "the Farm's price {price:?} against slot 0's stock {stock:?}"
    );
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, crate::recgame::write_gz(&rec)).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// **Score 1, the other half: the pump moves our walk, in exactly the
/// script's places.** The built start walked twice — with no stream, and
/// under [`crate::input::Stream::pump`] on the minted file — and compared
/// whole on the frame after each order: the two part on the objects that
/// order names and the stock its price moves, and nowhere else. Then
/// what each order asked for is read off the pumped walk.
#[test]
fn the_pump_moves_our_walk_in_exactly_the_script_s_places() {
    let Some((_, built, _)) = built_start() else {
        eprintln!("skipping: no install or no {LOBBY_DUMP} (set RON_GAMELOG_DIR)");
        return;
    };
    let Some(rec) = minted(&script()) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    // The walk under the first k orders, for k = 0..=3: each order is
    // compared on its own frame against the walk that has every order
    // before it and not it, so a draw an earlier order moved cannot be
    // charged to a later one.
    let prefix = |k: usize| {
        let mut r = rec.clone();
        r.packages = r.packages[..1 + k]
            .iter()
            .chain(r.packages.last())
            .cloned()
            .collect();
        r
    };
    let mut seen = Vec::new();
    for (k, f) in [MOVE_FRAME, QUEUE_FRAME, BUILD_FRAME]
        .into_iter()
        .enumerate()
    {
        let n = i64::from(f) + 1;
        let mut before = built.clone();
        let mut after = built.clone();
        tick_to(
            &mut before,
            Some(&mut crate::input::Stream::pump(&prefix(k))),
            n,
        );
        tick_to(
            &mut after,
            Some(&mut crate::input::Stream::pump(&prefix(k + 1))),
            n,
        );
        let places = parted_places(&after.sim, &before.sim);
        eprintln!("PLACES at {n}: {places:?}");
        seen.push((n, places.into_iter().collect::<Vec<_>>()));
    }
    let pinned: Vec<(i64, Vec<String>)> = PUMPED_PLACES
        .iter()
        .map(|(n, p)| (*n, p.iter().map(|s| s.to_string()).collect()))
        .collect();
    assert_eq!(seen, pinned, "PUMPED_PLACES: each order's footprint");
    // What each order asked for, on the pumped walk against the free one.
    let mut free = built.clone();
    let mut pumped = built;
    let mut stream = crate::input::Stream::pump(&rec);
    let unit = |b: &Built, o: i16| b.sim.unit_by_o(0, o).map(|u| b.sim.units[u].clone());
    let building = |b: &Built, o: i16| {
        b.sim
            .building_by_o(0, o)
            .map(|i| b.sim.buildings[i].clone())
    };
    tick_to(&mut free, None, i64::from(MOVE_FRAME) + 1);
    tick_to(&mut pumped, Some(&mut stream), i64::from(MOVE_FRAME) + 1);
    let dest = |u: &sim::units::unit::Unit| {
        u.orders.front().and_then(|o| match &o.body {
            sim::orders::Body::Move(m) => Some((m.kind, m.dest)),
            _ => None,
        })
    };
    assert_eq!(
        dest(&unit(&pumped, SCOUT).unwrap()),
        Some((sim::orders::MoveKind::MoveTo, SCOUT_TO)),
        "the scout's order on {}",
        MOVE_FRAME + 1
    );
    assert_eq!(dest(&unit(&free, SCOUT).unwrap()), None);
    tick_to(&mut free, None, i64::from(QUEUE_FRAME) + 1);
    tick_to(&mut pumped, Some(&mut stream), i64::from(QUEUE_FRAME) + 1);
    let queue = |b: &Built| {
        building(b, CAPITAL)
            .unwrap()
            .queue
            .items
            .iter()
            .map(|i| built_type(b, i.ty))
            .collect::<Vec<_>>()
    };
    assert_eq!(queue(&pumped), vec![CITIZEN], "the capital's queue");
    assert_eq!(queue(&free), Vec::<i32>::new());
    tick_to(&mut free, None, i64::from(BUILD_FRAME) + 1);
    tick_to(&mut pumped, Some(&mut stream), i64::from(BUILD_FRAME) + 1);
    let site = building(&pumped, FARM_SITE).expect("the Farm's site");
    assert_eq!(
        (site.orig_ty, site.pos, site.started),
        (
            Some((FARM - crate::load::BASE_BUILDTYPES) as usize),
            FARM_SITE_AT,
            false
        ),
        "the site on {}",
        BUILD_FRAME + 1
    );
    assert!(building(&free, FARM_SITE).is_none());
    let builder = unit(&pumped, BUILDER).unwrap();
    assert!(
        matches!(
            builder.orders.iter().map(|o| &o.body).collect::<Vec<_>>()[..],
            [sim::orders::Body::Move(_), sim::orders::Body::Build(_)]
        ),
        "the builder's orders: {:?}",
        builder.orders
    );
    // And by frame 300: the scout arrived, the citizen trained, the Farm up.
    tick_to(&mut free, None, 300);
    tick_to(&mut pumped, Some(&mut stream), 300);
    assert_eq!(unit(&pumped, SCOUT).unwrap().pos, SCOUT_TO);
    assert!(unit(&pumped, TRAINED).is_some() && unit(&free, TRAINED).is_none());
    assert!(building(&pumped, FARM_SITE).unwrap().active);
}

/// A queue entry's `TypeIndex`, as the dump writes it.
fn built_type(b: &Built, ty: usize) -> i32 {
    b.type_index[b.unit_tree[ty]]
}

/// The Farm's object number and the point [`sim::Sim::snap_center`] puts
/// it on; the trained citizen's object number.
pub(crate) use crate::script::{FARM_SITE, TRAINED};
pub(crate) const FARM_SITE_AT: Pos = Pos { x: 3456, y: 29760 };

/// Each order's footprint on our walk (item 4059): on the frame after its
/// package, the places the walk with it parts from the walk with every
/// order before it and not it — the objects it names (the scout; the
/// capital; the builder, the Farm's site and the Woodcutter the builder
/// left) and the bookkeeping its entry writes (the group pool, the path
/// scratch, the stock the price moves, the footprint's tiles). Since item
/// 4098 the Farm's site also heads its cell's chain of objects
/// (`.terrain.object_heads`, `docs/COLLISION.md` §3.2). Since item 4113
/// the builder's `GATHER`, killed by the `build`, gives back its hold
/// doober (`.roster.hold_doobers`, `docs/ORDERS.md` §6.12). Since item
/// 4124 the Farm's `Build::init` counts `buildings_built`
/// (`.cities.city_tally`, `docs/CHECKSUM.md` §20). Since item 4505 the
/// gatherer chain is one link a unit: the builder's leaving the
/// Woodcutter's chain writes its predecessor's link (`.units{0/2}`), not a
/// list on the Woodcutter (`.buildings{0/2001}` leaves).
const PUMPED_PLACES: &[(i64, &[&str])] = &[
    (
        31,
        &[
            ".movement.coll",
            ".movement.pf_start",
            ".orders.last_group",
            ".orders.pushed",
            ".trace.phase_marks",
            ".units{0/0}",
        ],
    ),
    (
        61,
        &[
            ".buildings{0/2000}",
            ".economy.ledgers",
            ".orders.last_group",
            ".orders.pushed",
            ".roster.muster",
            ".tech[0]",
        ],
    ),
    (
        91,
        &[
            ".buildings{0/2007}",
            ".cities.cities",
            ".cities.city_tally",
            ".cities.farm_order",
            ".economy.ledgers",
            // Item 4187: the score (`leaders.score`), recounted on who 0's
            // frame 90 with the farm the order placed.
            ".leaders.score",
            ".movement.pf_start",
            ".orders.last_group",
            ".orders.pushed",
            ".roster.hold_doobers",
            ".roster.marks",
            ".terrain.object_heads",
            ".units{0/1}",
            ".units{0/2}",
            ".world.tiles",
        ],
    ),
];

/// The frames the orders' effects are read on, both sides: the frame after
/// each package, and the take's last.
const READ_AT: [i64; 4] = [31, 61, 91, 299];

/// What the three orders did, read off one frame: the scout's point and
/// its first order's target; the capital's live queue, by `TypeIndex`; the
/// Farm's site — `(type, x, y)` — if it stands; and the builder's order
/// list, by class.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Effect {
    pub frame: i64,
    pub scout: (i64, i64),
    pub scout_to: Option<(i64, i64)>,
    pub queue: Vec<i64>,
    pub site: Option<(i64, i64, i64)>,
    pub builder: Vec<&'static str>,
}

/// The dump's order block names, the classes [`effect_ours`] maps to.
fn class(name: &str) -> &'static str {
    match name {
        "MOVEORDER" => "move",
        "BUILDORDER" => "build",
        "GATHERORDER" => "gather",
        _ => "other",
    }
}

fn effect_theirs(f: &crate::gamelog::Frame) -> Effect {
    let unit = |o: i16| f.units.iter().find(|u| u.who == 0 && u.o == i64::from(o));
    let build = |o: i16| f.builds.iter().find(|b| b.who == 0 && b.o == i64::from(o));
    let scout = unit(SCOUT);
    Effect {
        frame: f.n,
        scout: scout.map_or((-1, -1), |u| (u.pos.x, u.pos.y)),
        scout_to: scout
            .and_then(|u| u.current_order())
            .and_then(|o| Some((o.x?, o.y?))),
        queue: build(CAPITAL).map_or_else(Vec::new, |b| {
            let live = b.queued.unwrap_or(0).max(0) as usize;
            b.queue.iter().take(live).map(|q| q.ty).collect()
        }),
        site: build(FARM_SITE).map(|b| (b.orig_type.unwrap_or(-1), b.pos.x, b.pos.y)),
        builder: unit(BUILDER).map_or_else(Vec::new, |u| {
            u.orders_front_first().map(|o| class(&o.kind)).collect()
        }),
    }
}

fn effect_ours(b: &Built) -> Effect {
    let s = &b.sim;
    let unit = |o: i16| s.unit_by_o(0, o).map(|u| &s.units[u]);
    let build = |o: i16| s.building_by_o(0, o).map(|i| &s.buildings[i]);
    let scout = unit(SCOUT);
    Effect {
        frame: s.frame,
        scout: scout.map_or((-1, -1), |u| (i64::from(u.pos.x), i64::from(u.pos.y))),
        scout_to: scout
            .and_then(|u| u.orders.front())
            .and_then(|o| match &o.body {
                sim::orders::Body::Move(m) => Some((i64::from(m.dest.x), i64::from(m.dest.y))),
                _ => None,
            }),
        queue: build(CAPITAL).map_or_else(Vec::new, |c| {
            c.queue
                .items
                .iter()
                .map(|i| i64::from(built_type(b, i.ty)))
                .collect()
        }),
        site: build(FARM_SITE).map(|c| {
            (
                i64::from(crate::load::BASE_BUILDTYPES) + c.orig_ty.map_or(-1, |t| t as i64),
                i64::from(c.pos.x),
                i64::from(c.pos.y),
            )
        }),
        builder: unit(BUILDER).map_or_else(Vec::new, |u| {
            u.orders
                .iter()
                .map(|o| match &o.body {
                    sim::orders::Body::Move(_) => "move",
                    sim::orders::Body::Build(_) => "build",
                    sim::orders::Body::Gather(_) => "gather",
                    _ => "other",
                })
                .collect()
        }),
    }
}

/// The original's playback of the minted file, per take: `(the take's
/// dump, its trace, frames whose draws agree, frames walked, records
/// compared, records parted)`, as [`super::start_walk::PLAYBACK_WALKED`]
/// pins 4058's. run1561a prints every unit and building for 300 frames
/// (`end:MISC,UNITS=3,BUILDS=7`); run1561b only the trace, for 2,400.
pub(crate) const MINTED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[
    (
        "gamelog-run1561a-greatlakes-playback-of-minted-300.txt",
        "rontrace-run1561a.log",
        300,
        300,
        299,
        0,
    ),
    (
        "gamelog-run1561b-greatlakes-playback-of-minted-2400.txt",
        "rontrace-run1561b.log",
        2400,
        2400,
        1,
        0,
    ),
    (
        "gamelog-run1561-greatlakes-playback-of-minted-24000.txt",
        "rontrace-run1561.log",
        24000,
        24000,
        1,
        0,
    ),
];

/// The built start walked under the pump on `orders`' file against one
/// take: the walk, and the walked `Built`.
fn walk_minted(orders: &[Order], take: &str, take_trace: &str) -> Option<(Walked, Built)> {
    let (_, built, players) = built_start()?;
    let rec = minted(orders)?;
    let (Some(playback), Some(tr)) = (dump(take), trace(take_trace)) else {
        eprintln!("skipping {take}: not filed (set RON_GAMELOG_DIR)");
        return None;
    };
    let mut stream = crate::input::Stream::pump(&rec);
    Some(walk_with(
        built,
        players,
        &[(take, playback)],
        &tr,
        i64::MAX,
        Some(&mut stream),
    ))
}

/// **Scores 2 and 3: the original plays the minted file, and our walk
/// under the pump keeps lockstep with it.** Per take, the built start
/// walked under [`crate::input::Stream::pump`] on the same packages the
/// file holds: every frame's draws against the playback's trace, every
/// record of its dump compared whole ([`MINTED_WALKED`]). And on the take
/// with records, what each order did is read on both sides on the frame
/// after it and on the last ([`MINTED_EFFECTS`]): two sides that both
/// ignored an order would agree, so the effects are pinned, not only their
/// agreement.
#[test]
fn the_original_s_playback_of_the_minted_file_walks_in_lockstep() {
    playback_walks_in_lockstep(&script(), MINTED_WALKED, &minted_effects());
}

/// Per take of `table`, the built start walked under the pump on
/// `orders`' file against the take's dump and trace, pinned to the table;
/// and on the first take, which prints records, each order's effect read
/// on both sides on [`READ_AT`] and pinned to `pinned`.
fn playback_walks_in_lockstep(
    orders: &[Order],
    table: &[(&str, &str, i64, i64, usize, usize)],
    pinned: &[Effect],
) {
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in table {
        let Some((w, _)) = walk_minted(orders, take, take_trace) else {
            return;
        };
        eprintln!(
            "MINTED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        if let Some((o, t)) = &w.sites
            && let Some((_, d)) = first_parting(o, t)
        {
            eprintln!("the first parting frame, {:?}: {d}", w.first_sequence);
        }
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    // The effects, both sides, on the take that prints records.
    let (take, ..) = table[0];
    let mut source = crate::capture::indexed::IndexedCapture::open(dump(take).unwrap()).unwrap();
    let mut theirs = Vec::new();
    for i in 0..source.frames().len() {
        if READ_AT.contains(&source.frames()[i].number) {
            theirs.push(effect_theirs(&source.frame_state(i).unwrap()));
        }
    }
    let ours = effects_ours(orders);
    for (o, t) in ours.iter().zip(&theirs) {
        eprintln!("EFFECT ours {o:?}\n       theirs {t:?}");
    }
    assert_eq!(
        walked, table,
        "the walked table: frames agreeing, frames walked, records, parted"
    );
    assert_eq!(theirs, pinned, "the original's effects");
    assert_eq!(ours, pinned, "our effects");
}

/// Our walk's effects under the pump on `orders`' file, on [`READ_AT`].
fn effects_ours(orders: &[Order]) -> Vec<Effect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for n in READ_AT {
        tick_to(&mut built, Some(&mut stream), n);
        ours.push(effect_ours(&built));
    }
    ours
}

/// **What the three orders did, on both sides** (run1561a): on 31 the
/// scout walks to its point and on 299 stands on it; on 61 the capital's
/// queue holds one Citizen, and by 299 it has trained it; on 91 the Farm's
/// site stands at the snapped point and the builder holds a move and a
/// build, and by 299 it farms what it built.
fn minted_effects() -> Vec<Effect> {
    let to = Some((i64::from(SCOUT_TO.x), i64::from(SCOUT_TO.y)));
    let site = Some((
        i64::from(FARM),
        i64::from(FARM_SITE_AT.x),
        i64::from(FARM_SITE_AT.y),
    ));
    let citizen = i64::from(CITIZEN);
    vec![
        Effect {
            frame: 31,
            scout: (3514, 31896),
            scout_to: to,
            queue: vec![],
            site: None,
            builder: vec!["gather"],
        },
        Effect {
            frame: 61,
            scout: (4534, 31896),
            scout_to: to,
            queue: vec![citizen],
            site: None,
            builder: vec!["gather"],
        },
        Effect {
            frame: 91,
            scout: (5554, 31896),
            scout_to: to,
            queue: vec![citizen],
            site,
            builder: vec!["move", "build"],
        },
        Effect {
            frame: 299,
            scout: (5880, 31896),
            scout_to: None,
            queue: vec![],
            site,
            builder: vec!["gather"],
        },
    ]
}

/// **Score 4: made to fail.** Our walk with one order withheld, against
/// the original's playback of the full script (run1561a): it must part,
/// and first on the frame after the withheld order's package. Per order,
/// `(the order's frame, the first parted record's frame, its first
/// parting's kind, the first frame whose draws part)` ([`WITHHELD`]).
#[test]
fn our_walk_with_one_order_withheld_parts_at_that_order_s_frame() {
    let (take, take_trace, ..) = MINTED_WALKED[0];
    let mut seen = Vec::new();
    for k in 0..script().len() {
        let mut orders = script();
        let (f, ..) = orders.remove(k);
        let Some((w, _)) = walk_minted(&orders, take, take_trace) else {
            return;
        };
        let (frame, _, partings) = w.first_record.clone().expect("the walk parts");
        eprintln!(
            "WITHHELD {f}: first record {frame}: {partings:?}; draws part (count) {:?}, (sequence) {:?}; records {} parted {}",
            w.first_count, w.first_sequence, w.records, w.records_parted
        );
        if let Some((o, t)) = &w.sites
            && let Some((_, d)) = first_parting(o, t)
        {
            eprintln!("  the first parting draw, {:?}: {d}", w.first_sequence);
        }
        seen.push((
            f,
            frame,
            partings[0].0,
            partings[0].2.clone(),
            w.first_sequence,
        ));
    }
    let pinned: Vec<_> = WITHHELD
        .iter()
        .map(|&(f, n, kind, row, draws)| (f, n, kind, row.to_string(), draws))
        .collect();
    assert_eq!(seen, pinned, "WITHHELD");
}

/// Each order withheld from our walk against run1561a (item 4059): the
/// order's frame, the first parted record's frame, its first parting's
/// kind and row (both values), and the first frame whose draws part (in
/// `Built::frame_sites`' numbering, the frame before the gamelog's — the
/// original's own word parts from run53's no-order game at the same 40).
/// The move withheld leaves the scout on its start point and its order
/// list empty; the `queue_up` withheld leaves the capital's `queued` 0;
/// the `build` withheld leaves the builder at its tree and the Market's
/// `city_down` short of the Farm.
const WITHHELD: &[(i32, i64, &str, &str, Option<i64>)] = &[
    (
        30,
        31,
        "diverged",
        "Divergence { frame: 31, who: 0, o: 0, ours: Pos { x: 3480, y: 31896 }, theirs: Pos { x: 3514, y: 31896 } }",
        Some(40),
    ),
    (
        60,
        61,
        "queue_diverged",
        "QueueDivergence { frame: 61, who: 0, o: 2000, field: \"queued\", ours: 0, theirs: 1 }",
        Some(158),
    ),
    (
        90,
        91,
        "diverged",
        "Divergence { frame: 91, who: 0, o: 1, ours: Pos { x: 4008, y: 28296 }, theirs: Pos { x: 4023, y: 28316 } }",
        Some(296),
    ),
];

/// **The clicked file's size** (item 4149, `docs/CLIENT_INPUT.md` §5): the
/// lobby's file with [`crate::script::clicked`]'s packages, gzipped as the
/// original reads it.
const CLICKED_RCX_BYTES: usize = 59_280;

/// **The clicked script, minted** (item 4149): the three orders a person
/// clicks at the opening camera, as the client's input layer mints them
/// (`minted_clicked_at_the_opening_camera` holds them equal to
/// [`crate::script::clicked`]), round-trip through the file like the first
/// script's; they differ from it in the move's and the build's
/// coordinates alone; the clicked Farm's site is clear on our placement
/// check and snaps where the first script's does. With
/// `RON_RCX_OUT=<file>` it writes the `.rcx` the playback takes open.
#[test]
fn the_clicked_minted_packages_round_trip_and_stand_on_our_placement_check() {
    let Some((_, built, _)) = built_start() else {
        eprintln!("skipping: no install or no {LOBBY_DUMP} (set RON_GAMELOG_DIR)");
        return;
    };
    let (Some(rec), Some(first)) = (minted(&clicked()), minted(&script())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "clicked").expect("the clicked file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in clicked().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let parted: Vec<i32> = back
        .packages
        .iter()
        .zip(&first.packages)
        .filter(|(a, b)| a != b)
        .map(|(a, _)| a.frame)
        .collect();
    assert_eq!(
        parted,
        vec![MOVE_FRAME, BUILD_FRAME],
        "the packages that part"
    );
    let s = &built.sim;
    let farm = (FARM - crate::load::BASE_BUILDTYPES) as usize;
    assert_eq!(
        s.blocked_site(Some(0), farm, crate::script::CLICKED_FARM_AT, None),
        sim::cities::place::Blocked::Clear,
        "the clicked Farm's site, on our placement check"
    );
    let gz = crate::recgame::write_gz(&rec);
    assert_eq!(gz.len(), CLICKED_RCX_BYTES, "CLICKED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// **What the clicked orders do on our walk** (item 4149), on [`READ_AT`]:
/// the scout sent to its clicked point and standing on it by 299, the
/// Citizen queued and trained, the Farm's site where the first script's
/// stands and its builder farming it. Pinned before the original's
/// playback of the file is read ([`CLICKED_WALKED`]).
#[test]
fn the_clicked_minted_script_s_effects_on_our_walk() {
    if built_start().is_none() || minted(&clicked()).is_none() {
        eprintln!("skipping: no install, lobby or recording");
        return;
    }
    let ours = effects_ours(&clicked());
    for o in &ours {
        eprintln!("CLICKED EFFECT ours {o:?}");
    }
    assert_eq!(ours, clicked_effects(), "clicked_effects");
}

/// The original's playback of the clicked file, per take (run 1563), as
/// [`MINTED_WALKED`]: run1563a prints every unit and building for 300
/// frames, run1563b only the trace for 2,400, run1563 the game's length.
pub(crate) const CLICKED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[
    (
        "gamelog-run1563a-greatlakes-playback-of-clicked-300.txt",
        "rontrace-run1563a.log",
        300,
        300,
        299,
        0,
    ),
    (
        "gamelog-run1563-greatlakes-playback-of-clicked-24000.txt",
        "rontrace-run1563.log",
        24000,
        24000,
        1,
        0,
    ),
];

/// **The original plays the clicked file, and our walk keeps lockstep**
/// (item 4149): [`CLICKED_WALKED`] and [`clicked_effects`], both sides.
#[test]
fn the_original_s_playback_of_the_clicked_minted_file_walks_in_lockstep() {
    if CLICKED_WALKED.is_empty() {
        eprintln!("skipping: the clicked file's playback is not captured");
        return;
    }
    playback_walks_in_lockstep(&clicked(), CLICKED_WALKED, &clicked_effects());
}

/// What the clicked orders did, on [`READ_AT`]: as [`minted_effects`] but
/// for the scout's point on 299, its clicked (5883, 31896). Its order's
/// target reads (5880, 31896) on 31, 61 and 91 on our walk, as the first
/// script's does: the clicked destination is not what the move order
/// holds, and the scout still ends on the clicked point.
fn clicked_effects() -> Vec<Effect> {
    let to = crate::script::CLICKED_SCOUT_TO;
    let mut e = minted_effects();
    e[3].scout = (i64::from(to.x), i64::from(to.y));
    e
}

/// **Made to fail: the first script against the clicked playback**
/// (item 4149). Our walk under [`script`]'s file, whose move goes to
/// (5880, 31896) and whose Farm drops at (3552, 29856), against run1563a,
/// the original's playback of the clicked file: three units of the move's
/// destination must part, and the first record that parts is pinned
/// ([`FIRST_AGAINST_CLICKED`]): `(frame, kind, row, the first frame whose
/// draws part)`.
#[test]
fn the_first_minted_script_against_the_clicked_playback_parts() {
    let (take, take_trace, ..) = CLICKED_WALKED[0];
    let Some((w, _)) = walk_minted(&script(), take, take_trace) else {
        return;
    };
    let (frame, _, partings) = w.first_record.clone().expect("the walk parts");
    eprintln!(
        "FIRST AGAINST CLICKED: record {frame}: {partings:?}; draws part {:?}; records {} parted {}",
        w.first_sequence, w.records, w.records_parted
    );
    let seen = (
        frame,
        partings[0].0,
        partings[0].2.clone(),
        w.first_sequence,
    );
    let (f, kind, row, draws) = FIRST_AGAINST_CLICKED;
    assert_eq!(
        seen,
        (f, kind, row.to_string(), draws),
        "FIRST_AGAINST_CLICKED"
    );
    // The long take: the same walk against run1563. It does not part: the
    // draws of 24,000 frames and the closing record agree with the first
    // script's orders too (the two playbacks' words are identical on all
    // 24,001 frames, `rngcmp.py`), so the click's three units are seen by
    // run1563a's records alone. Seen red first: pinned `(None, Some(-1))`.
    let (take, take_trace, ..) = CLICKED_WALKED[1];
    let Some((w, _)) = walk_minted(&script(), take, take_trace) else {
        return;
    };
    eprintln!(
        "FIRST AGAINST CLICKED (long): first {:?}; draws part {:?}; records {} parted {}",
        w.first_record, w.first_sequence, w.records, w.records_parted
    );
    let long = w.first_record.map(|(n, _, p)| (n, p[0].2.clone()));
    assert_eq!(
        (long, w.first_sequence),
        (None, None),
        "FIRST_AGAINST_CLICKED, the long take"
    );
}

/// The first script's walk against run1563a: the first parted record is
/// the scout's path target on 31, ours (5880, 31896) against the original's
/// clicked (5883, 31896); 269 of 299 records part; the draws do not part
/// inside the take's 300 frames. Seen red first: pinned `(0, "", "", None)`.
const FIRST_AGAINST_CLICKED: (i64, &str, &str, Option<i64>) = (
    31,
    "order_diverged",
    "OrderDivergence { frame: 31, who: 0, o: 0, slot: 0, what: PathTo { slot: 0, ours: (5880, 31896), theirs: (5883, 31896) } }",
    None,
);

/// **The main view's clicked file's size** (item 4194,
/// `docs/CLIENT_INPUT.md` §9): the lobby's file with
/// [`crate::script::clicked_main_view`]'s packages, gzipped.
const CLICKED_MAIN_RCX_BYTES: usize = 59_278;

/// **The clicked script as the main view mints it** (item 4194): the
/// clicked file with the Scout's move carrying `form` 0 and `width` 50 in
/// place of the minimap's −1/−1; it round-trips, and parts from the played
/// clicked file in the move's package alone. With `RON_RCX_OUT=<file>` it
/// writes the `.rcx` run1572 takes open.
#[test]
fn the_clicked_main_view_minted_packages_round_trip() {
    let main = crate::script::clicked_main_view();
    let (Some(rec), Some(played)) = (minted(&main), minted(&clicked())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "clicked-main").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in main.iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let parted: Vec<i32> = back
        .packages
        .iter()
        .zip(&played.packages)
        .filter(|(a, b)| a != b)
        .map(|(a, _)| a.frame)
        .collect();
    assert_eq!(parted, vec![MOVE_FRAME], "the packages that part");
    let gz = crate::recgame::write_gz(&rec);
    assert_eq!(gz.len(), CLICKED_MAIN_RCX_BYTES, "CLICKED_MAIN_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The original's playback of the main view's clicked file (run1572), as
/// [`CLICKED_WALKED`]: every unit and building printed for 300 frames.
pub(crate) const CLICKED_MAIN_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1572-greatlakes-playback-of-clicked-main-view-300.txt",
    "rontrace-run1572.log",
    300,
    300,
    299,
    0,
)];

/// **The original plays the main view's clicked file, and our walk keeps
/// lockstep** (item 4194): [`CLICKED_MAIN_WALKED`], and each order's
/// effect equal on both sides — [`clicked_effects`], the played clicked
/// file's own, since the move's (0, 50) leaves the Scout where −1/−1 does.
/// Seen red first: the row pinned `(0, 0, 0, 0)`.
#[test]
fn the_original_s_playback_of_the_main_view_clicked_file_walks_in_lockstep() {
    playback_walks_in_lockstep(
        &crate::script::clicked_main_view(),
        CLICKED_MAIN_WALKED,
        &clicked_effects(),
    );
}

/// The frames the Scout's formation bytes are read on, both sides: the
/// frame before the move, the frame after it, the frame its order's
/// target turns to the clicked point, and the take's last.
const SCOUT_FORM_AT: [i64; 4] = [29, 31, 101, 299];

/// **The Scout's formation bytes, both sides** (item 4194):
/// `(frame, form, form_mod)` of `0/0` on [`SCOUT_FORM_AT`] under the main
/// view's file — `UnitData +0xaa`, `+0xab` in run1572's `UNITDATA`, and
/// `Unit::form`, `form_width` on our walk. The move's `width` 50 is the
/// `form_mod` both hold from 31; run1563a, the minimap's −1/−1, prints
/// the same. Seen red first: pinned `(f, 0, -1)` on every frame.
const SCOUT_FORM: [(i64, i64, i64); 4] = [(29, 0, -1), (31, 0, 50), (101, 0, 50), (299, 0, 50)];

#[test]
fn the_main_view_s_scout_holds_the_move_s_formation_on_both_sides() {
    let (take, ..) = CLICKED_MAIN_WALKED[0];
    let Some(text) = dump(take) else {
        eprintln!("skipping {take}: not filed (set RON_GAMELOG_DIR)");
        return;
    };
    let Some((_, mut built, _)) = built_start() else {
        return;
    };
    let mut source = crate::capture::indexed::IndexedCapture::open(text).unwrap();
    let mut theirs = Vec::new();
    for i in 0..source.frames().len() {
        let n = source.frames()[i].number;
        if SCOUT_FORM_AT.contains(&n) {
            let f = source.frame_state(i).unwrap();
            let u = f.units.iter().find(|u| u.who == 0 && u.o == 0).unwrap();
            theirs.push((n, u.form.unwrap_or(-2), u.form_mod.unwrap_or(-2)));
        }
    }
    let mut stream =
        crate::input::Stream::pump(&minted(&crate::script::clicked_main_view()).unwrap());
    let mut ours = Vec::new();
    for n in SCOUT_FORM_AT {
        tick_to(&mut built, Some(&mut stream), n);
        let s = &built.sim;
        let u = &s.units[s.unit_by_o(0, SCOUT).unwrap()];
        ours.push((n, i64::from(u.form), i64::from(u.form_width)));
    }
    eprintln!("SCOUT FORM theirs {theirs:?}\n           ours {ours:?}");
    assert_eq!(theirs, SCOUT_FORM, "the original's");
    assert_eq!(ours, SCOUT_FORM, "ours");
}

/// **Made to fail against the main view's playback** (item 4194). The
/// played clicked file, the minimap's −1/−1, walked against run1572: it
/// parts nowhere, since the two arms leave the same state (the two dumps
/// differ in three wall-clock lines alone). The first script's file,
/// three units off on the move, parts there as it parts on run1563a
/// ([`FIRST_AGAINST_CLICKED`]): the take can fail. Seen red first: the
/// clicked walk pinned to part on 31.
#[test]
fn the_minimap_s_clicked_file_against_the_main_view_playback() {
    let (take, take_trace, ..) = CLICKED_MAIN_WALKED[0];
    let Some((w, _)) = walk_minted(&clicked(), take, take_trace) else {
        return;
    };
    eprintln!(
        "CLICKED AGAINST MAIN: first {:?}; draws part {:?}; records {} parted {}",
        w.first_record, w.first_sequence, w.records, w.records_parted
    );
    assert_eq!(
        (
            w.first_record.map(|r| r.0),
            w.first_sequence,
            w.records,
            w.records_parted
        ),
        (None, None, 299, 0),
        "the minimap's file against run1572"
    );
    let Some((w, _)) = walk_minted(&script(), take, take_trace) else {
        return;
    };
    let (frame, _, partings) = w.first_record.clone().expect("the walk parts");
    let (f, kind, row, draws) = FIRST_AGAINST_CLICKED;
    assert_eq!(
        (
            frame,
            partings[0].0,
            partings[0].2.clone(),
            w.first_sequence
        ),
        (f, kind, row.to_string(), draws),
        "the first script against run1572"
    );
}

/// **The right-pressed file's size** (item 4179, `docs/CLIENT_INPUT.md` §6):
/// the lobby's file with [`crate::script::right_pressed`]'s packages,
/// gzipped as the original reads it.
const RIGHT_PRESSED_RCX_BYTES: usize = 59_312;

/// **The right-pressed script, minted** (item 4179): the clicked script's
/// three orders and two right presses on objects, as the client's input
/// layer mints them (`minted_right_pressed_at_the_opening_camera` holds
/// them equal to [`crate::script::right_pressed`]), round-trip through the
/// file; its packages are the clicked file's and the two right presses'
/// behind them, nothing else. With `RON_RCX_OUT=<file>` it writes the
/// `.rcx` the playback takes open.
#[test]
fn the_right_pressed_minted_packages_round_trip() {
    let (Some(rec), Some(first)) = (minted(&right_pressed()), minted(&clicked())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "right-pressed").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in right_pressed().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let added: Vec<i32> = back
        .packages
        .iter()
        .filter(|p| !first.packages.contains(p))
        .map(|p| p.frame)
        .collect();
    // The closing package's `stamp` counts the packages before it, so it
    // parts too: 5 in the clicked file, 7 here (`recgame::minted_packages`).
    let last = back.packages.last().unwrap();
    assert_eq!((last.stamp, first.packages.last().unwrap().stamp), (7, 5));
    assert_eq!(
        added,
        vec![
            crate::script::RIGHT_BUILD_FRAME,
            crate::script::RIGHT_GATHER_FRAME,
            last.frame
        ],
        "the packages the clicked file does not hold"
    );
    assert_eq!(back.packages.len(), first.packages.len() + 2);
    let gz = crate::recgame::write_gz(&rec);
    assert_eq!(gz.len(), RIGHT_PRESSED_RCX_BYTES, "RIGHT_PRESSED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The frames the right presses' effects are read on: the frame after each
/// package, and the take's last.
const RIGHT_READ_AT: [i64; 3] = [121, 201, 299];

/// What the two right presses did, read off one frame: the helper's and
/// the trained Citizen's order lists, by class, front first; and whether
/// the Farm's site is finished (`WallData::is_active`, `flags & 4`).
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RightEffect {
    pub frame: i64,
    pub helper: Vec<&'static str>,
    pub trained: Vec<&'static str>,
    pub site_finished: Option<bool>,
}

fn right_effect_theirs(f: &crate::gamelog::Frame) -> RightEffect {
    let unit = |o: i16| f.units.iter().find(|u| u.who == 0 && u.o == i64::from(o));
    let orders = |o: i16| {
        unit(o).map_or_else(Vec::new, |u| {
            u.orders_front_first().map(|o| class(&o.kind)).collect()
        })
    };
    RightEffect {
        frame: f.n,
        helper: orders(crate::script::HELPER),
        trained: orders(TRAINED),
        site_finished: f
            .builds
            .iter()
            .find(|b| b.who == 0 && b.o == i64::from(FARM_SITE))
            .map(|b| b.flags & 4 != 0),
    }
}

fn right_effect_ours(b: &Built) -> RightEffect {
    let s = &b.sim;
    let orders = |o: i16| {
        s.unit_by_o(0, o).map_or_else(Vec::new, |u| {
            s.units[u]
                .orders
                .iter()
                .map(|o| match &o.body {
                    sim::orders::Body::Move(_) => "move",
                    sim::orders::Body::Build(_) => "build",
                    sim::orders::Body::Gather(_) => "gather",
                    _ => "other",
                })
                .collect()
        })
    };
    RightEffect {
        frame: s.frame,
        helper: orders(crate::script::HELPER),
        trained: orders(TRAINED),
        site_finished: s.building_by_o(0, FARM_SITE).map(|i| s.buildings[i].active),
    }
}

/// Our walk under the pump on `orders`' file: each right press's effect on
/// [`RIGHT_READ_AT`], and the first frame the Farm's site is finished.
fn right_effects_ours(orders: &[Order]) -> (Vec<RightEffect>, Option<i64>) {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let (mut ours, mut done) = (Vec::new(), None);
    for n in 91..=*RIGHT_READ_AT.last().unwrap() {
        tick_to(&mut built, Some(&mut stream), n);
        if done.is_none() && s_site_finished(&built) {
            done = Some(n);
        }
        if RIGHT_READ_AT.contains(&n) {
            ours.push(right_effect_ours(&built));
        }
    }
    (ours, done)
}

fn s_site_finished(b: &Built) -> bool {
    b.sim
        .building_by_o(0, FARM_SITE)
        .is_some_and(|i| b.sim.buildings[i].active)
}

/// **What the right presses do** (item 4179), pinned before the original's
/// playback is read: `(the effects on [`RIGHT_READ_AT`], the frame the
/// Farm's site is first finished)`.
fn right_pressed_effects() -> (Vec<RightEffect>, Option<i64>) {
    let e = |frame, helper: &[&'static str], trained: &[&'static str], site| RightEffect {
        frame,
        helper: helper.to_vec(),
        trained: trained.to_vec(),
        site_finished: Some(site),
    };
    (
        vec![
            e(121, &["move", "build"], &[], false),
            e(201, &["build"], &["move", "gather"], false),
            e(299, &["gather"], &["move", "gather"], true),
        ],
        Some(254),
    )
}

/// The same read on the clicked script's walk, the control: no right
/// press, the site finished on its one builder's frame.
fn clicked_right_effects() -> (Vec<RightEffect>, Option<i64>) {
    let e = |frame, site| RightEffect {
        frame,
        helper: vec!["gather"],
        trained: vec![],
        site_finished: Some(site),
    };
    (vec![e(121, false), e(201, false), e(299, true)], Some(297))
}

/// **What the right presses do on our walk** (item 4179): the helper sent to
/// the site and building it, the site finished sooner than under one
/// builder, the trained Citizen sent from idle to the Woodcutter's camp;
/// against the clicked script's walk, where neither press is made.
#[test]
fn the_right_pressed_minted_script_s_effects_on_our_walk() {
    if built_start().is_none() || minted(&right_pressed()).is_none() {
        eprintln!("skipping: no install, lobby or recording");
        return;
    }
    let ours = right_effects_ours(&right_pressed());
    let control = right_effects_ours(&clicked());
    eprintln!("RIGHT EFFECT ours {ours:?}\n      control {control:?}");
    assert_eq!(control, clicked_right_effects(), "clicked_right_effects");
    assert_eq!(ours, right_pressed_effects(), "right_pressed_effects");
}

/// The original's playback of the right-pressed file (run 1568), as
/// [`CLICKED_WALKED`]: every unit and building printed for 300 frames,
/// which holds both right presses and the site's finish. Draws agree on
/// 300 of 300 frames, count and sequence, and 299 records compare whole.
/// Seen red first: pinned `(0, 0, 0, 0)`, it printed `(300, 300, 299, 0)`.
pub(crate) const RIGHT_PRESSED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1568-greatlakes-playback-of-right-pressed-300.txt",
    "rontrace-run1568.log",
    300,
    300,
    299,
    0,
)];

/// **The original plays the right-pressed file, and our walk keeps
/// lockstep** (item 4179): [`RIGHT_PRESSED_WALKED`], the clicked orders'
/// effects ([`clicked_effects`]) and the right presses' own
/// ([`right_pressed_effects`]), both sides.
#[test]
fn the_original_s_playback_of_the_right_pressed_file_walks_in_lockstep() {
    let (take, ..) = RIGHT_PRESSED_WALKED[0];
    let Some(d) = dump(take) else {
        eprintln!("skipping {take}: not filed (set RON_GAMELOG_DIR)");
        return;
    };
    playback_walks_in_lockstep(
        &right_pressed(),
        RIGHT_PRESSED_WALKED,
        &right_pressed_base_effects(),
    );
    let mut source = crate::capture::indexed::IndexedCapture::open(d).unwrap();
    let (mut theirs, mut done) = (Vec::new(), None);
    for i in 0..source.frames().len() {
        let n = source.frames()[i].number;
        if n < 91 {
            continue;
        }
        let f = source.frame_state(i).unwrap();
        let e = right_effect_theirs(&f);
        if done.is_none() && e.site_finished == Some(true) {
            done = Some(n);
        }
        if RIGHT_READ_AT.contains(&n) {
            theirs.push(e);
        }
    }
    eprintln!("RIGHT EFFECT theirs {theirs:?} finished {done:?}");
    assert_eq!((theirs, done), right_pressed_effects(), "the original's");
}

/// The clicked orders' effects on [`READ_AT`] under the right presses: as
/// [`clicked_effects`], but on 299 the first builder `0/1` holds a move and
/// a gather, the site having finished on 254 with two builders where one
/// finishes it on 297.
fn right_pressed_base_effects() -> Vec<Effect> {
    let mut e = clicked_effects();
    e[3].builder = vec!["move", "gather"];
    e
}

/// **Made to fail: the clicked script against the right-pressed playback**
/// (item 4179). Our walk under [`clicked`]'s file, which makes neither
/// right press, against run1568: it must part, and the first record that
/// parts is pinned ([`CLICKED_AGAINST_RIGHT_PRESSED`]).
#[test]
fn the_clicked_script_against_the_right_pressed_playback_parts() {
    let (take, take_trace, ..) = RIGHT_PRESSED_WALKED[0];
    let Some((w, _)) = walk_minted(&clicked(), take, take_trace) else {
        return;
    };
    let (frame, _, partings) = w.first_record.clone().expect("the walk parts");
    eprintln!(
        "CLICKED AGAINST RIGHT PRESSED: record {frame}: {partings:?}; draws part {:?}; records {} parted {}",
        w.first_sequence, w.records, w.records_parted
    );
    let seen = (
        frame,
        partings[0].0,
        partings[0].2.clone(),
        w.first_sequence,
        w.records_parted,
    );
    let (f, kind, row, draws, parted) = CLICKED_AGAINST_RIGHT_PRESSED;
    assert_eq!(
        seen,
        (f, kind, row.to_string(), draws, parted),
        "CLICKED_AGAINST_RIGHT_PRESSED"
    );
}

/// The clicked script's walk against run1568: `(the first parted record's
/// frame, its kind, its row, the first frame whose draws part, records
/// parted)`. The first parting is the helper `0/2` on 121, standing at its
/// camp in ours and walking to the site in theirs; beside it its order
/// list, 1 long against 2, and the camp's `gather_down`, which still names
/// it in ours. 179 of 299 records part, the draws from 244. Seen red first:
/// pinned `(0, "", "", None, 0)`.
const CLICKED_AGAINST_RIGHT_PRESSED: (i64, &str, &str, Option<i64>, usize) = (
    121,
    "diverged",
    "Divergence { frame: 121, who: 0, o: 2, ours: Pos { x: 4536, y: 28680 }, theirs: Pos { x: 4519, y: 28700 } }",
    Some(244),
    179,
);

/// **The boxed file's size** (item 4180, `docs/CLIENT_INPUT.md` §7.3): the
/// lobby's file with [`crate::script::boxed`]'s packages, gzipped.
const BOXED_RCX_BYTES: usize = 59_338;

/// **The boxed script, minted** (item 4180): the right-pressed file and
/// one package more, the box's `group` of three and its `move_to` with the
/// formation fields; it round-trips through the file. With
/// `RON_RCX_OUT=<file>` it writes the `.rcx` the playback takes open.
#[test]
fn the_boxed_minted_packages_round_trip() {
    let (Some(rec), Some(first)) = (minted(&boxed()), minted(&right_pressed())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "boxed").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in boxed().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let added: Vec<i32> = back
        .packages
        .iter()
        .filter(|p| !first.packages.contains(p))
        .map(|p| p.frame)
        .collect();
    let last = back.packages.last().unwrap();
    assert_eq!((last.stamp, first.packages.last().unwrap().stamp), (8, 7));
    assert_eq!(
        added,
        vec![crate::script::BOX_FRAME, last.frame],
        "the packages the right-pressed file does not hold"
    );
    // The box's package: a `group` of three (3 + 2·3 bytes) and the
    // `move_to` (22), its formation bytes `form` 9 and `width` 50.
    let package = &back.packages[back.packages.len() - 2];
    assert_eq!(package.data.len(), 9 + 22, "the box's package");
    let gz = crate::recgame::write_gz(&rec);
    assert_eq!(gz.len(), BOXED_RCX_BYTES, "BOXED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The frames the box's move is read on: the frame after its package, a
/// frame of the walk, and the take's last.
const BOX_READ_AT: [i64; 3] = [241, 270, 299];

/// One boxed unit, read off one frame: its order list by class, front
/// first; the front move's target, snapped to its 48-unit cell; its path's
/// goal, the bottom of the stack, its own slot of the formation;
/// `orders_x`/`orders_y`, the end of its leading moves; and its formation
/// bytes, `form` and `form_mod`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BoxUnit {
    pub o: i16,
    pub orders: Vec<&'static str>,
    pub to: Option<(i64, i64)>,
    pub goal: Option<(i64, i64)>,
    pub orders_xy: (i64, i64),
    pub form: (i64, i64),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BoxEffect {
    pub frame: i64,
    pub units: Vec<BoxUnit>,
}

fn box_effect_theirs(f: &crate::gamelog::Frame) -> BoxEffect {
    let units = crate::script::BOXED
        .iter()
        .filter_map(|&o| {
            let u = f.units.iter().find(|u| u.who == 0 && u.o == i64::from(o))?;
            Some(BoxUnit {
                o,
                orders: u.orders_front_first().map(|o| class(&o.kind)).collect(),
                to: u
                    .orders_front_first()
                    .find(|o| o.kind == "MOVEORDER")
                    .and_then(|o| Some((o.x?, o.y?))),
                goal: u.path.first().map(|p| p.to),
                orders_xy: (u.orders_x.unwrap_or(-1), u.orders_y.unwrap_or(-1)),
                form: (u.form.unwrap_or(-2), u.form_mod.unwrap_or(-2)),
            })
        })
        .collect();
    BoxEffect { frame: f.n, units }
}

fn box_effect_ours(b: &Built) -> BoxEffect {
    let s = &b.sim;
    let units = crate::script::BOXED
        .iter()
        .filter_map(|&o| {
            let u = &s.units[s.unit_by_o(0, o)?];
            Some(BoxUnit {
                o,
                orders: u
                    .orders
                    .iter()
                    .map(|o| match &o.body {
                        sim::orders::Body::Move(_) => "move",
                        sim::orders::Body::Build(_) => "build",
                        sim::orders::Body::Gather(_) => "gather",
                        _ => "other",
                    })
                    .collect(),
                to: u.orders.iter().find_map(|o| match &o.body {
                    sim::orders::Body::Move(m) => Some((i64::from(m.dest.x), i64::from(m.dest.y))),
                    _ => None,
                }),
                goal: u
                    .path
                    .first()
                    .map(|p| (i64::from(p.to.x), i64::from(p.to.y))),
                orders_xy: (i64::from(u.orders_pos.x), i64::from(u.orders_pos.y)),
                form: (i64::from(u.form), i64::from(u.form_width)),
            })
        })
        .collect();
    BoxEffect {
        frame: s.frame,
        units,
    }
}

/// Our walk under the pump on `orders`' file, read on [`BOX_READ_AT`].
fn box_effects_ours(orders: &[Order]) -> Vec<BoxEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in &BOX_READ_AT {
        tick_to(&mut built, Some(&mut stream), n);
        ours.push(box_effect_ours(&built));
    }
    ours
}

/// A boxed unit's read: `(o, its orders, the front move's target, its
/// path's goal, orders_x/y, form, form_mod)`.
type BoxRow = (
    i16,
    &'static [&'static str],
    Option<(i64, i64)>,
    Option<(i64, i64)>,
    (i64, i64),
    (i64, i64),
);

fn box_effect(frame: i64, rows: [BoxRow; 3]) -> BoxEffect {
    BoxEffect {
        frame,
        units: rows
            .into_iter()
            .map(|(o, orders, to, goal, orders_xy, form)| BoxUnit {
                o,
                orders: orders.to_vec(),
                to,
                goal,
                orders_xy,
                form,
            })
            .collect(),
    }
}

/// **What the box's move does** (item 4180), pinned before the original's
/// playback is read: each of the three on [`BOX_READ_AT`]. On 241 each
/// holds one move to its own slot of the mob formation round [`BOX_TO`]
/// (`form` 9): the path's goal is the slot, `0/3`'s the click itself, and
/// the move's target the slot's 48-unit cell centre. Each carries `form` 9
/// and `form_mod` 50, the move's own bytes; by 299 each stands on its goal
/// with no order.
///
/// [`BOX_TO`]: crate::script::BOX_TO
fn boxed_effects() -> Vec<BoxEffect> {
    let walking = |f| {
        box_effect(
            f,
            [
                (
                    3,
                    &["move"],
                    Some((2328, 32712)),
                    Some((2344, 32710)),
                    (2328, 32712),
                    (9, 50),
                ),
                (
                    4,
                    &["move"],
                    Some((2232, 32760)),
                    Some((2214, 32775)),
                    (2232, 32760),
                    (9, 50),
                ),
                (
                    5,
                    &["move"],
                    Some((2424, 32568)),
                    Some((2408, 32580)),
                    (2424, 32568),
                    (9, 50),
                ),
            ],
        )
    };
    vec![
        walking(241),
        walking(270),
        box_effect(
            299,
            [
                (3, &[], None, None, (2344, 32710), (9, 50)),
                (4, &[], None, None, (2214, 32775), (9, 50)),
                (5, &[], None, None, (2408, 32580), (9, 50)),
            ],
        ),
    ]
}

/// The same read on the right-pressed script's walk, the control: no box.
/// The three farm on, `0/3` walking to its next field on 241, and their
/// formation bytes stay the start's, `form` 9 and `form_mod` −1.
fn right_box_effects() -> Vec<BoxEffect> {
    let farming = |f, three: BoxRow| {
        box_effect(
            f,
            [
                three,
                (4, &["gather"], None, None, (2040, 31992), (9, -1)),
                (5, &["gather"], None, None, (1848, 31608), (9, -1)),
            ],
        )
    };
    vec![
        farming(
            241,
            (
                3,
                &["move", "gather"],
                Some((2424, 32376)),
                None,
                (2424, 32376),
                (9, -1),
            ),
        ),
        farming(270, (3, &["gather"], None, None, (2424, 32376), (9, -1))),
        farming(299, (3, &["gather"], None, None, (2424, 32376), (9, -1))),
    ]
}

/// **What the box does on our walk** (item 4180): each Citizen taken from
/// its farm and sent to its slot of the move, against the right-pressed
/// walk, where they farm on.
#[test]
fn the_boxed_minted_script_s_effects_on_our_walk() {
    if built_start().is_none() || minted(&boxed()).is_none() {
        eprintln!("skipping: no install, lobby or recording");
        return;
    }
    let ours = box_effects_ours(&boxed());
    let control = box_effects_ours(&right_pressed());
    eprintln!("BOX EFFECT ours {ours:#?}\n      control {control:#?}");
    assert_eq!(control, right_box_effects(), "right_box_effects");
    assert_eq!(ours, boxed_effects(), "boxed_effects");
}

/// The original's playback of the boxed file (run 1569), as
/// [`RIGHT_PRESSED_WALKED`]: every unit and building printed for 300
/// frames, which holds the box's move and 59 frames of its walk. Draws
/// agree on 300 of 300 frames, count and sequence, and 299 records compare
/// whole. Seen red first: pinned `(0, 0, 0, 0)`, it printed
/// `(300, 300, 299, 0)`.
pub(crate) const BOXED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1569-greatlakes-playback-of-boxed-300.txt",
    "rontrace-run1569.log",
    300,
    300,
    299,
    0,
)];

/// **The original plays the boxed file, and our walk keeps lockstep**
/// (item 4180): [`BOXED_WALKED`], the clicked orders' effects, the right
/// presses' and the box's, both sides.
#[test]
fn the_original_s_playback_of_the_boxed_file_walks_in_lockstep() {
    let (take, ..) = BOXED_WALKED[0];
    let Some(d) = dump(take) else {
        eprintln!("skipping {take}: not filed (set RON_GAMELOG_DIR)");
        return;
    };
    playback_walks_in_lockstep(&boxed(), BOXED_WALKED, &right_pressed_base_effects());
    let mut source = crate::capture::indexed::IndexedCapture::open(d).unwrap();
    let (mut right, mut done, mut theirs) = (Vec::new(), None, Vec::new());
    for i in 0..source.frames().len() {
        let n = source.frames()[i].number;
        if n < 91 {
            continue;
        }
        let f = source.frame_state(i).unwrap();
        let e = right_effect_theirs(&f);
        if done.is_none() && e.site_finished == Some(true) {
            done = Some(n);
        }
        if RIGHT_READ_AT.contains(&n) {
            right.push(e);
        }
        if BOX_READ_AT.contains(&n) {
            theirs.push(box_effect_theirs(&f));
        }
    }
    eprintln!("BOX EFFECT theirs {theirs:#?}");
    assert_eq!((right, done), right_pressed_effects(), "the right presses");
    assert_eq!(theirs, boxed_effects(), "the original's box");
}

/// **Made to fail: the right-pressed script against the boxed playback**
/// (item 4180). Our walk under [`right_pressed`]'s file, which makes no
/// box, against run1569: it must part, and the first record that parts is
/// pinned ([`RIGHT_PRESSED_AGAINST_BOXED`]).
#[test]
fn the_right_pressed_script_against_the_boxed_playback_parts() {
    let (take, take_trace, ..) = BOXED_WALKED[0];
    let Some((w, _)) = walk_minted(&right_pressed(), take, take_trace) else {
        return;
    };
    let (frame, _, partings) = w.first_record.clone().expect("the walk parts");
    eprintln!(
        "RIGHT PRESSED AGAINST BOXED: record {frame}: {partings:?}; draws part {:?}; records {} parted {}",
        w.first_sequence, w.records, w.records_parted
    );
    let seen = (
        frame,
        partings[0].0,
        partings[0].2.clone(),
        w.first_sequence,
        w.records_parted,
    );
    let (f, kind, row, draws, parted) = RIGHT_PRESSED_AGAINST_BOXED;
    assert_eq!(
        seen,
        (f, kind, row.to_string(), draws, parted),
        "RIGHT_PRESSED_AGAINST_BOXED"
    );
}

/// The right-pressed script's walk against run1569: `(the first parted
/// record's frame, its kind, its row, the first frame whose draws part,
/// records parted)`. The first parting is `0/3` on 241, still at its field
/// in ours and walking south in theirs; beside it its order list, 2 long
/// against 1, `0/5`'s heading, and the farm `0/2002`'s `gather_down`, which
/// still names `0/3` in ours. 59 of 299 records part, every one from 241,
/// and the draws from 240. Seen red first: pinned `(0, "", "", None, 0)`.
const RIGHT_PRESSED_AGAINST_BOXED: (i64, &str, &str, Option<i64>, usize) = (
    241,
    "diverged",
    "Divergence { frame: 241, who: 0, o: 3, ours: Pos { x: 2808, y: 31992 }, theirs: Pos { x: 2794, y: 32014 } }",
    Some(240),
    59,
);

/// The attacked file's gzip length (item 4195).
const ATTACKED_RCX_BYTES: usize = 59_404;

/// **The attacked script, minted** (item 4195, `docs/CLIENT_INPUT.md` §8):
/// the boxed file and four packages more — the research, the Barracks,
/// the Slingers and the attack; it round-trips through the file. With
/// `RON_RCX_OUT=<file>` it writes the `.rcx` the playback takes open.
#[test]
fn the_attacked_minted_packages_round_trip() {
    use crate::script::{
        ATTACK_FRAME, ATTACK_RESEARCH_FRAME, BARRACKS_FRAME, TRAIN_FRAME, attacked,
    };
    let (Some(rec), Some(first)) = (minted(&attacked()), minted(&boxed())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "attacked").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in attacked().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let added: Vec<i32> = back
        .packages
        .iter()
        .filter(|p| !first.packages.contains(p))
        .map(|p| p.frame)
        .collect();
    let last = back.packages.last().unwrap();
    assert_eq!((last.stamp, first.packages.last().unwrap().stamp), (12, 8));
    assert_eq!(
        added,
        vec![
            ATTACK_RESEARCH_FRAME,
            BARRACKS_FRAME,
            TRAIN_FRAME,
            ATTACK_FRAME,
            last.frame
        ],
        "the packages the boxed file does not hold"
    );
    let gz = crate::recgame::write_gz(&rec);
    assert_eq!(gz.len(), ATTACKED_RCX_BYTES, "ATTACKED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The frames the attack is read on (item 4195): the package's own frame,
/// the frame after it, the frame before the first blow on our walk, the
/// blow's, and the window's last record (run1571 prints 6690..6899).
const ATTACK_READ_AT: [i64; 5] = [6703, 6704, 6804, 6805, 6899];

/// The attack, read off one frame: the target `1/0`'s position and hits
/// left (`myhits − damage` in the dump), and the Slinger `0/7`'s position
/// and order list by class, front first.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AttackEffect {
    pub frame: i64,
    pub target: ((i64, i64), i64),
    pub slinger: ((i64, i64), Vec<&'static str>),
}

/// The dump's order block names, as [`AttackEffect`] classes them.
fn attack_class(name: &str) -> &'static str {
    match name {
        "ATTACKORDER" => "attack",
        "MOVEORDER" => "move",
        _ => "other",
    }
}

fn attack_effect_theirs(f: &crate::gamelog::Frame) -> AttackEffect {
    let (tw, to) = crate::script::ATTACKED;
    let unit = |who: u8, o: i16| {
        f.units
            .iter()
            .find(|u| u.who == i64::from(who) && u.o == i64::from(o))
            .unwrap()
    };
    let t = unit(tw, to);
    let sl = unit(0, crate::script::SLINGER);
    AttackEffect {
        frame: f.n,
        target: (
            (t.pos.x, t.pos.y),
            t.myhits.unwrap_or(-1) - t.damage.unwrap_or(0),
        ),
        slinger: (
            (sl.pos.x, sl.pos.y),
            sl.orders_front_first()
                .map(|o| attack_class(&o.kind))
                .collect(),
        ),
    }
}

fn attack_effect_ours(b: &Built) -> AttackEffect {
    let s = &b.sim;
    let (tw, to) = crate::script::ATTACKED;
    let t = &s.units[s.unit_by_o(tw, to).unwrap()];
    let sl = &s.units[s.unit_by_o(0, crate::script::SLINGER).unwrap()];
    AttackEffect {
        frame: s.frame,
        target: (
            (i64::from(t.pos.x), i64::from(t.pos.y)),
            i64::from(t.health),
        ),
        slinger: (
            (i64::from(sl.pos.x), i64::from(sl.pos.y)),
            sl.orders
                .iter()
                .map(|o| match &o.body {
                    sim::orders::Body::Attack(_) => "attack",
                    sim::orders::Body::Move(_) => "move",
                    _ => "other",
                })
                .collect(),
        ),
    }
}

/// Our walk under the pump on `orders`' file, read on [`ATTACK_READ_AT`].
fn attack_effects_ours(orders: &[Order]) -> Vec<AttackEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in &ATTACK_READ_AT {
        tick_to(&mut built, Some(&mut stream), n);
        ours.push(attack_effect_ours(&built));
    }
    ours
}

/// The original's, off the take's records on [`ATTACK_READ_AT`].
fn attack_effects_theirs(take: &str) -> Option<Vec<AttackEffect>> {
    let mut source = crate::capture::indexed::IndexedCapture::open(dump(take)?).unwrap();
    let mut theirs = Vec::new();
    for i in 0..source.frames().len() {
        if ATTACK_READ_AT.contains(&source.frames()[i].number) {
            theirs.push(attack_effect_theirs(&source.frame_state(i).unwrap()));
        }
    }
    Some(theirs)
}

/// **The attack's effect, both sides** (item 4195), our walk's values
/// pinned before the original's playback was read and run1571's equal to
/// them on every frame: the Slinger idle by its Barracks on 6703, `move`
/// then `attack` from 6704; the Scout's 50 hits until 6804 and 47 on
/// 6805, the first blow; 31 on 6899.
fn attacked_effects() -> Vec<AttackEffect> {
    let row = |frame, t: ((i64, i64), i64), at: (i64, i64), orders: &[&'static str]| AttackEffect {
        frame,
        target: t,
        slinger: (at, orders.to_vec()),
    };
    let chase: &[&str] = &["move", "attack"];
    vec![
        row(6703, ((6762, 31117), 50), (1176, 32760), &[]),
        row(6704, ((6731, 31133), 50), (1202, 32744), chase),
        row(6804, ((4320, 32084), 50), (3104, 31512), chase),
        row(6805, ((4320, 32118), 47), (3135, 31512), chase),
        row(6899, ((2130, 33412), 31), (3463, 31825), chase),
    ]
}

/// **What the attacked orders do on our walk** (item 4195), on
/// [`ATTACK_READ_AT`]: the Slinger takes the attack on the frame after
/// the package and lands its first blow on 6805, the Scout's 50 hits
/// going to 47.
#[test]
fn the_attacked_minted_script_s_effects_on_our_walk() {
    if built_start().is_none() || minted(&crate::script::attacked()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = attack_effects_ours(&crate::script::attacked());
    for e in &ours {
        eprintln!("ATTACK EFFECT ours {e:?}");
    }
    assert_eq!(ours, attacked_effects());
}

/// The attacked file's playback (`docs/RUNS.md` run1571): `(take, trace,
/// frames agreeing, frames walked, records, parted)`.
pub(crate) const ATTACKED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1571-greatlakes-playback-of-attacked-6900.txt",
    "rontrace-run1571.log",
    6900,
    6900,
    210,
    0,
)];

/// **The original plays the attacked file in lockstep to the blow**
/// (item 4195): the built start walked under the pump against run1571,
/// every frame's draws against its trace and every record of its window
/// whole ([`ATTACKED_WALKED`]); and the attack's effect read on both sides
/// on [`ATTACK_READ_AT`], the target's hits among them.
#[test]
fn the_original_s_playback_of_the_attacked_file_walks_in_lockstep() {
    let attacked = crate::script::attacked();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in ATTACKED_WALKED {
        let Some((w, _)) = walk_minted(&attacked, take, take_trace) else {
            return;
        };
        eprintln!(
            "ATTACKED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        if let Some((o, t)) = &w.sites
            && let Some((_, d)) = first_parting(o, t)
        {
            eprintln!("the first parting frame, {:?}: {d}", w.first_sequence);
        }
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let theirs = attack_effects_theirs(ATTACKED_WALKED[0].0).unwrap();
    let ours = attack_effects_ours(&attacked);
    for (o, t) in ours.iter().zip(&theirs) {
        eprintln!("ATTACK EFFECT ours {o:?}\n              theirs {t:?}");
    }
    assert_eq!(walked, ATTACKED_WALKED, "the walked table");
    assert_eq!(theirs, attacked_effects(), "the original's effects");
    assert_eq!(ours, attacked_effects(), "our effects");
}

/// **Made to fail** (item 4195): our walk under the boxed file — no
/// research, no Barracks, no Slinger, no attack — against run1571; and
/// the boxed walk read on the blow's frame, the control's values.
#[test]
fn the_boxed_script_against_the_attacked_playback_parts() {
    let (take, take_trace, ..) = ATTACKED_WALKED[0];
    let Some((w, built)) = walk_minted(&boxed(), take, take_trace) else {
        return;
    };
    let (frame, _, partings) = w.first_record.clone().expect("the walk parts");
    eprintln!(
        "BOXED AGAINST ATTACKED: record {frame}: {partings:?}; draws part {:?}; records {} parted {}",
        w.first_sequence, w.records, w.records_parted
    );
    let seen = (
        frame,
        partings[0].0,
        partings[0].2.clone(),
        w.first_sequence,
        w.records_parted,
    );
    let (f, kind, row, draws, parted) = BOXED_AGAINST_ATTACKED;
    assert_eq!(
        seen,
        (f, kind, row.to_string(), draws, parted),
        "BOXED_AGAINST_ATTACKED"
    );
    drop(built);
    let (_, mut b, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(&boxed()).unwrap());
    tick_to(&mut b, Some(&mut stream), 6805);
    let (tw, to) = crate::script::ATTACKED;
    let t = &b.sim.units[b.sim.unit_by_o(tw, to).unwrap()];
    let control = (
        b.sim.unit_by_o(0, crate::script::SLINGER).is_some(),
        (i64::from(t.pos.x), i64::from(t.pos.y)),
        t.health,
    );
    eprintln!("BOXED CONTROL on 6805: {control:?}");
    assert_eq!(control, BOXED_ON_THE_BLOW, "BOXED_ON_THE_BLOW");
}

/// The boxed script's walk against run1571: `(the first parted record's
/// frame, its kind, its row, the first frame whose draws part, records
/// parted)`. Every record of the window parts, first on the Slinger `0/7`
/// the boxed walk never trains; the draws part from 535, the Barracks'
/// site under construction. Seen red first: pinned `(0, "", "", None, 0)`.
const BOXED_AGAINST_ATTACKED: (i64, &str, &str, Option<i64>, usize) =
    (6690, "unlinked_units", "(0, 7)", Some(535), 210);
/// The boxed walk on 6805: `(a 0/7 on the map, 1/0's position, its hits)`
/// — no Slinger, and the Scout 4,152 units east of where the attacked walk
/// has it, untouched at 50. Seen red first: pinned `(true, (0, 0), 0)`.
const BOXED_ON_THE_BLOW: (bool, (i64, i64), i32) = (false, (8472, 33213), 50);

/// The followed file's gzip length (item 4202).
const FOLLOWED_RCX_BYTES: usize = 59_435;

/// **The followed script, minted** (item 4202, `docs/CLIENT_INPUT.md`
/// §10): the attacked file and two packages more, the Scout's fog press on
/// [`crate::script::FOG_FRAME`] and its follow beside the Slinger's attack;
/// it round-trips through the file. With `RON_RCX_OUT=<file>` it writes the
/// `.rcx` the playback takes open.
#[test]
fn the_followed_minted_packages_round_trip() {
    use crate::script::{ATTACK_FRAME, FOG_FRAME, FOLLOW_FRAME, attacked, followed};
    let (Some(rec), Some(first)) = (minted(&followed()), minted(&attacked())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "followed").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in followed().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let added: Vec<i32> = back
        .packages
        .iter()
        .filter(|p| !first.packages.contains(p))
        .map(|p| p.frame)
        .collect();
    let last = back.packages.last().unwrap();
    assert_eq!((last.stamp, first.packages.last().unwrap().stamp), (14, 12));
    assert_eq!(
        added,
        vec![FOG_FRAME, ATTACK_FRAME, FOLLOW_FRAME, last.frame],
        "the packages the attacked file does not hold: the attack's stamp moves by one"
    );
    let gz = crate::recgame::write_gz(&rec);
    eprintln!("FOLLOWED RCX {} bytes", gz.len());
    assert_eq!(gz.len(), FOLLOWED_RCX_BYTES, "FOLLOWED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The frames the followed script is read on (item 4202): the fog press's
/// frame, the attack's, the follow's and the one after it, the
/// Slinger's first blow under this script, and the window's last record.
const FOLLOW_READ_AT: [i64; 6] = [6702, 6703, 6704, 6705, 6805, 6899];

/// The followed script, read off one frame: [`AttackEffect`]'s target and
/// Slinger, and the Scout `0/0`'s position and order list by class, front
/// first.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FollowEffect {
    pub attack: AttackEffect,
    pub scout: ((i64, i64), Vec<&'static str>),
}

/// The dump's order block names, as [`FollowEffect`]'s Scout classes them.
fn follow_class(name: &str) -> &'static str {
    match name {
        "FOLLOWORDER" => "follow",
        "MOVEORDER" => "move",
        _ => "other",
    }
}

fn follow_effect_theirs(f: &crate::gamelog::Frame) -> FollowEffect {
    let sc = f
        .units
        .iter()
        .find(|u| u.who == 0 && u.o == i64::from(crate::script::SCOUT_OBJECT))
        .unwrap();
    FollowEffect {
        attack: attack_effect_theirs(f),
        scout: (
            (sc.pos.x, sc.pos.y),
            sc.orders_front_first()
                .map(|o| follow_class(&o.kind))
                .collect(),
        ),
    }
}

fn follow_effect_ours(b: &Built) -> FollowEffect {
    let s = &b.sim;
    let sc = &s.units[s.unit_by_o(0, crate::script::SCOUT_OBJECT).unwrap()];
    FollowEffect {
        attack: attack_effect_ours(b),
        scout: (
            (i64::from(sc.pos.x), i64::from(sc.pos.y)),
            sc.orders
                .iter()
                .map(|o| match &o.body {
                    sim::orders::Body::Follow(_) => "follow",
                    sim::orders::Body::Move(_) => "move",
                    _ => "other",
                })
                .collect(),
        ),
    }
}

/// Our walk under the pump on `orders`' file, read on [`FOLLOW_READ_AT`].
fn follow_effects_ours(orders: &[Order]) -> Vec<FollowEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in &FOLLOW_READ_AT {
        tick_to(&mut built, Some(&mut stream), n);
        ours.push(follow_effect_ours(&built));
    }
    ours
}

/// The original's, off the take's records on [`FOLLOW_READ_AT`].
fn follow_effects_theirs(take: &str) -> Option<Vec<FollowEffect>> {
    let mut source = crate::capture::indexed::IndexedCapture::open(dump(take)?).unwrap();
    let mut theirs = Vec::new();
    for i in 0..source.frames().len() {
        if FOLLOW_READ_AT.contains(&source.frames()[i].number) {
            theirs.push(follow_effect_theirs(&source.frame_state(i).unwrap()));
        }
    }
    Some(theirs)
}

/// **The followed script's effects** (item 4202), our walk's values
/// pinned before the original's playback was read, and run1573's equal to
/// them on every frame: the Scout idle on
/// 6702, moving to the fog press's ground from 6703, and `move`, `follow`
/// from 6705, the frame after its package, closing on `1/0` from the
/// east; the Slinger's attack as the attacked script's, its first blow
/// on 6805 (50 hits to 47).
fn followed_effects() -> Vec<FollowEffect> {
    type Side = ((i64, i64), &'static [&'static str]);
    let row = |frame, t: ((i64, i64), i64), sl: Side, sc: Side| FollowEffect {
        attack: AttackEffect {
            frame,
            target: t,
            slinger: (sl.0, sl.1.to_vec()),
        },
        scout: (sc.0, sc.1.to_vec()),
    };
    const CHASE: &[&str] = &["move", "attack"];
    const FOLLOW: &[&str] = &["move", "follow"];
    vec![
        row(
            6702,
            ((6792, 31099), 50),
            ((1176, 32760), &[]),
            ((5883, 31896), &[]),
        ),
        row(
            6703,
            ((6762, 31117), 50),
            ((1176, 32760), &[]),
            ((5908, 31874), &["move"]),
        ),
        row(
            6704,
            ((6731, 31133), 50),
            ((1202, 32744), CHASE),
            ((5933, 31852), &["move"]),
        ),
        row(
            6705,
            ((6700, 31148), 50),
            ((1228, 32729), CHASE),
            ((5959, 31831), FOLLOW),
        ),
        row(
            6805,
            ((4320, 32118), 47),
            ((3135, 31512), CHASE),
            ((4506, 31571), FOLLOW),
        ),
        row(
            6899,
            ((2122, 32090), 31),
            ((3240, 31944), &["attack"]),
            ((2641, 32500), FOLLOW),
        ),
    ]
}

/// **What the followed orders do on our walk** (item 4202), on
/// [`FOLLOW_READ_AT`].
#[test]
fn the_followed_minted_script_s_effects_on_our_walk() {
    if built_start().is_none() || minted(&crate::script::followed()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = follow_effects_ours(&crate::script::followed());
    for e in &ours {
        eprintln!("FOLLOW EFFECT ours {e:?}");
    }
    assert_eq!(ours, followed_effects());
}

/// The followed file's playback (`docs/RUNS.md` run1573): `(take, trace,
/// frames agreeing, frames walked, records, parted)`.
pub(crate) const FOLLOWED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1573-greatlakes-playback-of-followed-6900.txt",
    "rontrace-run1573.log",
    6900,
    6900,
    210,
    0,
)];

/// **The original plays the followed file in lockstep** (item 4202): the
/// built start walked under the pump against run1573, every frame's draws
/// against its trace and every record of its window whole
/// ([`FOLLOWED_WALKED`]); and the fog press's move, the follow and the
/// attack read on both sides on [`FOLLOW_READ_AT`].
#[test]
fn the_original_s_playback_of_the_followed_file_walks_in_lockstep() {
    let followed = crate::script::followed();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in FOLLOWED_WALKED {
        let Some((w, _)) = walk_minted(&followed, take, take_trace) else {
            return;
        };
        eprintln!(
            "FOLLOWED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let theirs = follow_effects_theirs(FOLLOWED_WALKED[0].0).unwrap();
    let ours = follow_effects_ours(&followed);
    for (o, t) in ours.iter().zip(&theirs) {
        eprintln!("FOLLOW EFFECT ours {o:?}\n              theirs {t:?}");
    }
    assert_eq!(walked, FOLLOWED_WALKED, "the walked table");
    assert_eq!(theirs, followed_effects(), "the original's effects");
    assert_eq!(ours, followed_effects(), "our effects");
}

/// **Made to fail** (item 4202): our walk under the attacked file — no fog
/// press, no follow — against run1573; and the attacked walk's Scout on
/// the follow's next frame, the control's values.
#[test]
fn the_attacked_script_against_the_followed_playback_parts() {
    let (take, take_trace, ..) = FOLLOWED_WALKED[0];
    let Some((w, _)) = walk_minted(&crate::script::attacked(), take, take_trace) else {
        return;
    };
    let (frame, _, partings) = w.first_record.clone().expect("the walk parts");
    eprintln!(
        "ATTACKED AGAINST FOLLOWED: record {frame}: {partings:?}; draws part {:?}; records {} parted {}",
        w.first_sequence, w.records, w.records_parted
    );
    let seen = (
        frame,
        partings[0].0,
        partings[0].2.clone(),
        w.first_sequence,
        w.records_parted,
    );
    let (f, kind, row, draws, parted) = ATTACKED_AGAINST_FOLLOWED;
    assert_eq!(
        seen,
        (f, kind, row.to_string(), draws, parted),
        "ATTACKED_AGAINST_FOLLOWED"
    );
}

/// The attacked script's walk against run1573: `(the first parted record's
/// frame, its kind, its row, the first frame whose draws part, records
/// parted)`. It parts first on 6703, the frame after the fog press: the
/// Scout `0/0` idle at (5883, 31896) on ours, moving at (5908, 31874) with
/// one order on run1573's; 197 of 210 records part, and the draws from
/// 6725. Seen red first: pinned `(0, "", "", None, 0)`.
const ATTACKED_AGAINST_FOLLOWED: (i64, &str, &str, Option<i64>, usize) = (
    6703,
    "diverged",
    "Divergence { frame: 6703, who: 0, o: 0, ours: Pos { x: 5883, y: 31896 }, theirs: Pos { x: 5908, y: 31874 } }",
    Some(6725),
    197,
);

/// The gathered file's gzip length (item 4249).
/// Seen red first: pinned 0, it printed 59,459.
const GATHERED_RCX_BYTES: usize = 59_459;

/// **The gathered script, minted** (item 4249, `docs/CLIENT_INPUT.md`
/// §11): the followed file and two packages more, the capital's rally
/// point on [`crate::script::GATHER_FRAME`] and its Citizen on
/// [`crate::script::GATHER_TRAIN_FRAME`]; it round-trips through the file.
/// With `RON_RCX_OUT=<file>` it writes the `.rcx` the playback takes open.
#[test]
fn the_gathered_minted_packages_round_trip() {
    use crate::script::{GATHER_FRAME, GATHER_TRAIN_FRAME, followed, full_groups, gathered};
    let (Some(rec), Some(first)) = (
        minted(&full_groups(&gathered())),
        minted(&full_groups(&followed())),
    ) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "gathered").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in full_groups(&gathered()).iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let added: Vec<i32> = back
        .packages
        .iter()
        .filter(|p| !first.packages.contains(p))
        .map(|p| p.frame)
        .collect();
    let last = back.packages.last().unwrap();
    assert_eq!((last.stamp, first.packages.last().unwrap().stamp), (16, 14));
    assert_eq!(
        added,
        vec![GATHER_FRAME, GATHER_TRAIN_FRAME, last.frame],
        "the packages the followed file does not hold"
    );
    let gz = crate::recgame::write_gz(&rec);
    eprintln!("GATHERED RCX {} bytes", gz.len());
    assert_eq!(gz.len(), GATHERED_RCX_BYTES, "GATHERED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The frames the gathered script is read on (item 4249): the rally
/// point's frame and the one after it, the Citizen's queue, the first
/// frame it stands on the map, the first it stands at the point, and the
/// window's last record.
const GATHER_READ_AT: [i64; 6] = [6706, 6707, 6709, 6814, 6832, 6999];

/// The trained Citizen, `0/10`.
const GATHERED_CITIZEN: i16 = 10;

/// The gathered script, read off one frame: the capital's gather list
/// (`BuildData::gather`, `(x, y, action)`), and the trained Citizen's
/// position and order list by class, front first, if it is on the map.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct GatherEffect {
    pub frame: i64,
    pub gather: Vec<(i64, i64, i64)>,
    pub citizen: Option<((i64, i64), Vec<&'static str>)>,
}

fn gather_class(name: &str) -> &'static str {
    match name {
        "MOVEORDER" => "move",
        "GATHERORDER" => "gather",
        _ => "other",
    }
}

fn gather_effect_theirs(f: &crate::gamelog::Frame) -> GatherEffect {
    let cap = f
        .builds
        .iter()
        .find(|b| b.who == 0 && b.o == i64::from(crate::script::CAPITAL))
        .unwrap();
    let citizen = f
        .units
        .iter()
        .find(|u| u.who == 0 && u.o == i64::from(GATHERED_CITIZEN))
        .map(|u| {
            (
                (u.pos.x, u.pos.y),
                u.orders_front_first()
                    .map(|o| gather_class(&o.kind))
                    .collect(),
            )
        });
    GatherEffect {
        frame: f.n,
        gather: cap.gather.clone(),
        citizen,
    }
}

fn gather_effect_ours(b: &Built) -> GatherEffect {
    let s = &b.sim;
    let cap = &s.buildings[s.building_by_o(0, crate::script::CAPITAL).unwrap()];
    let citizen = s.unit_by_o(0, GATHERED_CITIZEN).map(|i| {
        let u = &s.units[i];
        (
            (i64::from(u.pos.x), i64::from(u.pos.y)),
            u.orders
                .iter()
                .map(|o| match &o.body {
                    sim::orders::Body::Move(_) => "move",
                    sim::orders::Body::Gather(_) => "gather",
                    _ => "other",
                })
                .collect(),
        )
    });
    GatherEffect {
        frame: s.frame,
        gather: cap
            .gather
            .iter()
            .map(|g| (i64::from(g.pos.x), i64::from(g.pos.y), i64::from(g.action)))
            .collect(),
        citizen,
    }
}

/// Our walk under the pump on `orders`' file, read on [`GATHER_READ_AT`].
fn gather_effects_ours(orders: &[Order]) -> Vec<GatherEffect> {
    gather_effects_ours_at(orders, &GATHER_READ_AT)
}

/// [`gather_effects_ours`], read on the frames `at`, ascending.
fn gather_effects_ours_at(orders: &[Order], at: &[i64]) -> Vec<GatherEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in at {
        tick_to(&mut built, Some(&mut stream), n);
        ours.push(gather_effect_ours(&built));
    }
    ours
}

/// The original's, off the take's records on [`GATHER_READ_AT`].
fn gather_effects_theirs(take: &str) -> Option<Vec<GatherEffect>> {
    gather_effects_theirs_at(take, &GATHER_READ_AT)
}

/// [`gather_effects_theirs`], read on the frames `at`.
fn gather_effects_theirs_at(take: &str, at: &[i64]) -> Option<Vec<GatherEffect>> {
    let mut source = crate::capture::indexed::IndexedCapture::open(dump(take)?).unwrap();
    let mut theirs = Vec::new();
    for i in 0..source.frames().len() {
        if at.contains(&source.frames()[i].number) {
            theirs.push(gather_effect_theirs(&source.frame_state(i).unwrap()));
        }
    }
    Some(theirs)
}

/// **The gathered script's effects** (item 4249), our walk's values pinned
/// before the original's playback was read: the capital's gather list
/// empty on 6706 and `(4306, 31001, 0)` from 6707, the frame after the
/// package; the Citizen out of the capital on 6814, moving, and idle at
/// the point's arrival on 6832; gathering by 6999. Seen red first: pinned
/// empty.
fn gathered_effects() -> Vec<GatherEffect> {
    let row =
        |frame, gather: &[(i64, i64, i64)], citizen: Option<((i64, i64), &[&'static str])>| {
            GatherEffect {
                frame,
                gather: gather.to_vec(),
                citizen: citizen.map(|(p, o)| (p, o.to_vec())),
            }
        };
    const AT: &[(i64, i64, i64)] = &[(4306, 31001, 0)];
    vec![
        row(6706, &[], None),
        row(6707, AT, None),
        row(6709, AT, None),
        row(6814, AT, Some(((4008, 31320), &["move"]))),
        row(6832, AT, Some(((4296, 30984), &[]))),
        row(6999, AT, Some(((4271, 30624), &["move", "gather"]))),
    ]
}

/// **What the gathered orders do on our walk** (item 4249), on
/// [`GATHER_READ_AT`].
#[test]
fn the_gathered_minted_script_s_effects_on_our_walk() {
    if built_start().is_none() || minted(&crate::script::gathered()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = gather_effects_ours(&crate::script::gathered());
    for e in &ours {
        eprintln!("GATHER EFFECT ours {e:?}");
    }
    assert_eq!(ours, gathered_effects());
}

/// The gathered file's playback (`docs/RUNS.md` run1598): `(take, trace,
/// frames agreeing, frames walked, records, parted)`.
pub(crate) const GATHERED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1598-greatlakes-playback-of-gathered-7000.txt",
    "rontrace-run1598.log",
    7000,
    7000,
    300,
    0,
)];

/// **The original plays the gathered file in lockstep** (item 4249): the
/// built start walked under the pump against run1598, every frame's draws
/// against its trace and every record of its window whole
/// ([`GATHERED_WALKED`]); and the rally point, the Citizen's walk to it and
/// its gather read on both sides on [`GATHER_READ_AT`].
#[test]
fn the_original_s_playback_of_the_gathered_file_walks_in_lockstep() {
    let gathered = crate::script::gathered();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in GATHERED_WALKED {
        let Some((w, _)) = walk_minted(&gathered, take, take_trace) else {
            return;
        };
        eprintln!(
            "GATHERED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let theirs = gather_effects_theirs(GATHERED_WALKED[0].0).unwrap();
    let ours = gather_effects_ours(&gathered);
    for (o, t) in ours.iter().zip(&theirs) {
        eprintln!("GATHER EFFECT ours {o:?}\n              theirs {t:?}");
    }
    assert_eq!(walked, GATHERED_WALKED, "the walked table");
    assert_eq!(theirs, gathered_effects(), "the original's effects");
    assert_eq!(ours, gathered_effects(), "our effects");
}

/// **Made to fail** (item 4249): our walk under the followed file — no
/// rally point, no Citizen — against run1598.
#[test]
fn the_followed_script_against_the_gathered_playback_parts() {
    let (take, take_trace, ..) = GATHERED_WALKED[0];
    let Some((w, _)) = walk_minted(&crate::script::followed(), take, take_trace) else {
        return;
    };
    let (frame, _, partings) = w.first_record.clone().expect("the walk parts");
    eprintln!(
        "FOLLOWED AGAINST GATHERED: record {frame}: {partings:?}; draws part {:?}; records {} parted {}",
        w.first_sequence, w.records, w.records_parted
    );
    let seen = (
        frame,
        partings[0].0,
        partings[0].2.clone(),
        w.first_sequence,
        w.records_parted,
    );
    let (f, kind, row, draws, parted) = FOLLOWED_AGAINST_GATHERED;
    assert_eq!(
        seen,
        (f, kind, row.to_string(), draws, parted),
        "FOLLOWED_AGAINST_GATHERED"
    );
}

/// The followed script's walk against run1598: `(the first parted
/// record's frame, its kind, its row, the first frame whose draws part,
/// records parted)`. It parts first on 6707, the frame after the rally
/// point, at the capital's gather list: none on ours, one on run1598's;
/// 293 of 300 records part, and the draws from 6813, the Citizen's
/// coming out. Seen red first: pinned `(0, "", "", None, 0)`.
const FOLLOWED_AGAINST_GATHERED: (i64, &str, &str, Option<i64>, usize) = (
    6707,
    "build_diverged",
    "BuildDivergence { frame: 6707, who: 0, o: 2000, field: \"gather_len\", ours: 0, theirs: 1 }",
    Some(6813),
    293,
);

/// The chatted file's gzip length (item 4250).
/// Seen red first: pinned 0, it printed 59,491.
const CHATTED_RCX_BYTES: usize = 59_491;

/// **The chatted script, minted** (item 4250, `docs/CLIENT_INPUT.md`
/// §12): the gathered file and one package more, the typed line on
/// [`crate::script::CHAT_FRAME`]; it round-trips through the file. With
/// `RON_RCX_OUT=<file>` it writes the `.rcx` the playback takes open.
#[test]
fn the_chatted_minted_packages_round_trip() {
    use crate::script::{CHAT_FRAME, chatted, full_groups, gathered};
    let (Some(rec), Some(first)) = (
        minted(&full_groups(&chatted())),
        minted(&full_groups(&gathered())),
    ) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "chatted").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in full_groups(&chatted()).iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let added: Vec<i32> = back
        .packages
        .iter()
        .filter(|p| !first.packages.contains(p))
        .map(|p| p.frame)
        .collect();
    let last = back.packages.last().unwrap();
    assert_eq!((last.stamp, first.packages.last().unwrap().stamp), (17, 16));
    assert_eq!(
        added,
        vec![CHAT_FRAME, last.frame],
        "the packages the gathered file does not hold"
    );
    let gz = crate::recgame::write_gz(&rec);
    eprintln!("CHATTED RCX {} bytes", gz.len());
    assert_eq!(gz.len(), CHATTED_RCX_BYTES, "CHATTED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The chatted file's playback (`docs/RUNS.md` run1599): `(take, trace,
/// frames agreeing, frames walked, records, parted)`. Pinned from our walk
/// against run1598 before run1599 was read: the chat moves no record.
pub(crate) const CHATTED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1599-greatlakes-playback-of-chatted-7000.txt",
    "rontrace-run1599.log",
    7000,
    7000,
    300,
    0,
)];

/// The functions a chat runs on the original's side
/// (`CommandPackage::process_chat` and what it calls for each recipient),
/// by entry address.
const CHAT_FUNCTIONS: [(u32, &str); 5] = [
    (0x0094_54f0, "CommandPackage::process_chat"),
    (0x006b_8ab0, "Leader::receive_chat"),
    (0x006d_3320, "LeaderOut::display_chat"),
    (0x006b_8bf0, "Leader::receive_taunt"),
    (0x0098_0cf0, "Taunts::play"),
];

/// Each take's first entry into [`CHAT_FUNCTIONS`], in sim-frames, pinned
/// before run1599 was read: run1598 enters none; run1599 runs the chat on
/// 6709, the tick that consumes [`crate::script::CHAT_FRAME`]'s package
/// (run1598's rally point, on 6706, is first entered on 6705), and never
/// the taunt's two, `taunt` −1 and `taunt_num` 0. Seen red first: run1599
/// pinned as run1598's.
const CHAT_ENTRIES: &[(&str, [Option<i64>; 5])] = &[
    ("rontrace-run1598.log", [None; 5]),
    (
        "rontrace-run1599.log",
        [Some(6709), Some(6709), Some(6709), None, None],
    ),
];

/// **The original plays the chatted file in lockstep** (item 4250): the
/// built start walked under the pump against run1599, every frame's draws
/// against its trace and every record of its window whole
/// ([`CHATTED_WALKED`]); and the chat's own effect on the original's
/// side, the functions it enters ([`CHAT_ENTRIES`]), against run1598's.
#[test]
fn the_original_s_playback_of_the_chatted_file_walks_in_lockstep() {
    let chatted = crate::script::chatted();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in CHATTED_WALKED {
        let Some((w, _)) = walk_minted(&chatted, take, take_trace) else {
            return;
        };
        eprintln!(
            "CHATTED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let mut entries = Vec::new();
    for &(take_trace, _) in CHAT_ENTRIES {
        let tr = trace(take_trace).expect("the take's trace");
        let first = CHAT_FUNCTIONS.map(|(va, _)| tr.first_entry(va));
        for ((_, name), f) in CHAT_FUNCTIONS.iter().zip(&first) {
            eprintln!("CHAT ENTRY {take_trace} {name}: {f:?}");
        }
        entries.push((take_trace, first));
    }
    assert_eq!(walked, CHATTED_WALKED, "the walked table");
    assert_eq!(entries, CHAT_ENTRIES, "the chat's functions");
}

/// **The chat moves no record** (item 4250): our walk under the chatted
/// file against run1598, the gathered file's playback, is as whole as the
/// gathered walk is ([`GATHERED_WALKED`]): a plain chat writes nothing the
/// dump prints, on our side by construction (`Stream`'s chat arm applies
/// nothing) and on the original's by the playback's own records.
#[test]
fn the_chatted_script_against_the_gathered_playback_is_whole() {
    let (take, take_trace, agree, last, records, parted) = GATHERED_WALKED[0];
    let Some((w, _)) = walk_minted(&crate::script::chatted(), take, take_trace) else {
        return;
    };
    eprintln!(
        "CHATTED AGAINST GATHERED: to {}; draws part {:?}; records {} parted {}, first {:?}",
        w.last, w.first_sequence, w.records, w.records_parted, w.first_record
    );
    assert_eq!(
        (
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted
        ),
        (agree, last, records, parted)
    );
}

/// The paused file's gzip length (item 4252).
/// Seen red first: pinned 0, it printed 59,515.
const PAUSED_RCX_BYTES: usize = 59_515;

/// **The paused script, minted** (item 4252, `docs/CLIENT_INPUT.md`
/// §13): the chatted file and three packages more, all on
/// [`crate::script::PAUSE_FRAME`], stamped one a turn; it round-trips
/// through the file. With `RON_RCX_OUT=<file>` it writes the `.rcx` the
/// playback takes open.
#[test]
fn the_paused_minted_packages_round_trip() {
    use crate::script::{PAUSE_FRAME, chatted, full_groups, paused};
    let (Some(rec), Some(first)) = (
        minted(&full_groups(&paused())),
        minted(&full_groups(&chatted())),
    ) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "paused").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in full_groups(&paused()).iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let added: Vec<(i32, u32)> = back
        .packages
        .iter()
        .filter(|p| !first.packages.contains(p))
        .map(|p| (p.frame, p.stamp))
        .collect();
    let last = back.packages.last().unwrap();
    assert_eq!((last.stamp, first.packages.last().unwrap().stamp), (20, 17));
    assert_eq!(
        added,
        vec![
            (PAUSE_FRAME, 17),
            (PAUSE_FRAME, 18),
            (PAUSE_FRAME, 19),
            (last.frame, 20)
        ],
        "the packages the chatted file does not hold"
    );
    let gz = crate::recgame::write_gz(&rec);
    eprintln!("PAUSED RCX {} bytes", gz.len());
    assert_eq!(gz.len(), PAUSED_RCX_BYTES, "PAUSED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The rallied file's gzip length (item 4254).
/// Seen red first: pinned 0, it printed 59,504.
const RALLIED_RCX_BYTES: usize = 59_504;

/// **The rallied script, minted** (item 4254, `docs/CLIENT_INPUT.md`
/// §14.1): the chatted file and one package more, the capital's rally
/// point moved on [`crate::script::PAUSE_FRAME`] with no pause; it
/// round-trips through the file. With `RON_RCX_OUT=<file>` it writes the
/// `.rcx` run1602 takes open.
#[test]
fn the_rallied_minted_packages_round_trip() {
    use crate::script::{PAUSE_FRAME, chatted, full_groups, rallied};
    let (Some(rec), Some(first)) = (
        minted(&full_groups(&rallied())),
        minted(&full_groups(&chatted())),
    ) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "rallied").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in full_groups(&rallied()).iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let added: Vec<(i32, u32)> = back
        .packages
        .iter()
        .filter(|p| !first.packages.contains(p))
        .map(|p| (p.frame, p.stamp))
        .collect();
    let last = back.packages.last().unwrap();
    assert_eq!((last.stamp, first.packages.last().unwrap().stamp), (18, 17));
    assert_eq!(
        added,
        vec![(PAUSE_FRAME, 17), (last.frame, 18)],
        "the packages the chatted file does not hold"
    );
    let gz = crate::recgame::write_gz(&rec);
    eprintln!("RALLIED RCX {} bytes", gz.len());
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
    assert_eq!(gz.len(), RALLIED_RCX_BYTES, "RALLIED_RCX_BYTES");
}

/// The rallied file's playback (`docs/RUNS.md` run1602): `(take, trace,
/// frames agreeing, frames walked, records, parted)`. Pinned before run1602
/// was read, from our walk: the rallied script walks as the paused one
/// does (`Stream`'s pause arm applies nothing), so this pins the blow on
/// `1/0` where run1599 and our walk land it, on 6872, and the take whole
/// if the pause's three turns are what moved it on run1601 (4252a). Seen
/// red there: **40 of 300 part, the first on 6866 at `1/0`'s `damage`,
/// ours 13 and theirs 16**, as on run1601. The pause does not move the
/// blow; the moved rally point does, and our walk misses it
/// (`docs/CLIENT_INPUT.md` §14.1). **Whole since item 4256**, 40 → 0: the
/// blow is `0/8`'s round, released by the Slinger's `CHAR_ATTACK3` from its
/// node, a row `sim::data::launch` lacked ([`RALLIED_ROUND`]).
pub(crate) const RALLIED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1602-greatlakes-playback-of-rallied-7000.txt",
    "rontrace-run1602.log",
    7000,
    7000,
    300,
    0,
)];

/// **The round in flight on 6860 of the rallied game** (item 4256, run1603):
/// the original's whole ammo pool at logger frame 6860, read off the packet
/// by `tools/recomp/ammo.py` — one live slot, `0/8`'s round on `1/0`, born
/// on 6859. `(shooter, target, slot, (sx, sy, sz), (ex, ey, ez), cur_time,
/// total_time, accuracy)`. Seen red before the Slinger's `CHAR_ATTACK3` row:
/// ours left the unit's square at ground height, (3720, 31656, 533), for
/// (2884, 32738, 632) in 8 frames (`docs/CLIENT_INPUT.md` §15.1).
type Round = (
    (i64, i64),
    Option<(i64, i64)>,
    u32,
    (i32, i32, i32),
    (i32, i32, i32),
    i32,
    i32,
    i32,
);
const RALLIED_ROUND: (i64, &[Round]) = (
    6860,
    &[(
        (0, 8),
        Some((1, 0)),
        0,
        (3762, 31769, 710),
        (2915, 32738, 632),
        1,
        7,
        300,
    )],
);

/// **Our walk's ammo pool is the packet's** on [`RALLIED_ROUND`]'s frame:
/// the rallied script walked from the built start under the pump, every
/// live round read field for field.
#[test]
fn the_rallied_game_s_round_in_flight_is_the_packet_s() {
    let Some(ours) = rounds_at(&crate::script::rallied(), RALLIED_ROUND.0) else {
        return;
    };
    eprintln!("RALLIED ROUND ours {ours:?}");
    assert_eq!(ours, RALLIED_ROUND.1, "the packet's ammo pool");
}

/// **The rounds in flight on 6869 of the buildmasked game** (item 4277,
/// run1615b): the original's whole ammo pool at logger frame 6869, read
/// off the packet by `tools/recomp/ammo.py`, laid out as
/// [`RALLIED_ROUND`]. `0/9`'s stone, born on 6868, is the blow 4273a's
/// parting lands a frame late: its unled landing is ours, but it leaves
/// two units nearer the Scout, 749.5 from it rather than 750.3, and at
/// `proj_speed` 150 flies 4 frames where ours flew 5, so its lead is
/// 4 × 31 short of ours (`docs/CLIENT_INPUT.md` §25). Seen red on
/// `launch::MEASURED`'s planar rows: `0/8` from (3679, 31775), `0/9`
/// from (3391, 32213) to (2702, 32740) in 5.
const BUILDMASKED_ROUND: (i64, &[Round]) = (
    6869,
    &[
        (
            (0, 8),
            Some((1, 0)),
            0,
            (3679, 31773, 684),
            (2676, 32736, 623),
            4,
            8,
            300,
        ),
        (
            (0, 9),
            Some((1, 0)),
            1,
            (3389, 32212, 686),
            (2733, 32740, 633),
            2,
            4,
            300,
        ),
    ],
);

/// **Our walk's ammo pool is the packet's** on [`BUILDMASKED_ROUND`]'s
/// frame, the buildmasked script walked from the built start under the
/// pump: both of the Slingers' stones, field for field.
#[test]
fn the_buildmasked_game_s_round_in_flight_is_the_packet_s() {
    let Some(ours) = rounds_at(&crate::script::buildmasked(), BUILDMASKED_ROUND.0) else {
        return;
    };
    eprintln!("BUILDMASKED ROUND ours {ours:?}");
    assert_eq!(ours, BUILDMASKED_ROUND.1, "the packet's ammo pool");
}

/// Our walk's live rounds on `frame`, `script` walked from the built start
/// under the pump; `None` without the install.
fn rounds_at(script: &[Order], frame: i64) -> Option<Vec<Round>> {
    let (_, mut built, _) = built_start()?;
    let mut stream = crate::input::Stream::pump(&minted(script).unwrap());
    tick_to(&mut built, Some(&mut stream), frame);
    let sim = &built.sim;
    let ours: Vec<Round> = sim
        .combat
        .projectiles
        .iter()
        .filter(|p| !p.spent)
        .map(|p| {
            (
                crate::diff::golden::obj_ident(sim, p.shooter),
                p.target.map(|t| crate::diff::golden::obj_ident(sim, t)),
                p.slot,
                (p.launch.x, p.launch.y, p.sz),
                (p.landing.x, p.landing.y, p.ez),
                p.cur_time,
                p.total_time,
                p.accuracy,
            )
        })
        .collect();
    Some(ours)
}

/// A take's dumped window, from its first `BEGIN FRAME` line to its end.
fn window_bytes(take: &str) -> Option<Vec<u8>> {
    let bytes = std::fs::read(dump(take)?).unwrap();
    let at = bytes
        .windows(13)
        .position(|w| w == b" BEGIN FRAME ")
        .expect("a frame");
    Some(bytes[at..].to_vec())
}

/// **The rallied file's playback separates the pause from the rally
/// point** (item 4254, parked 4252a): the built start walked under the
/// pump against run1602, every frame's draws against its trace and every
/// record of its window ([`RALLIED_WALKED`]); the rally point's effects,
/// read on both sides on [`PAUSE_READ_AT`], as the paused file's; and the
/// pause's functions entered nowhere ([`PAUSE_FUNCTIONS`]).
#[test]
fn the_original_s_playback_of_the_rallied_file_separates_the_pause() {
    let rallied = crate::script::rallied();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in RALLIED_WALKED {
        let Some((w, _)) = walk_minted(&rallied, take, take_trace) else {
            return;
        };
        eprintln!(
            "RALLIED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let tr = trace(RALLIED_WALKED[0].1).expect("the take's trace");
    let entries = PAUSE_FUNCTIONS.map(|(va, _)| tr.first_entry(va));
    let ours = gather_effects_ours_at(&rallied, &PAUSE_READ_AT);
    let theirs = gather_effects_theirs_at(RALLIED_WALKED[0].0, &PAUSE_READ_AT).unwrap();
    for t in &theirs {
        eprintln!("RALLIED EFFECT theirs {t:?}");
    }
    // The pause moves no dumped byte: run1602's window is run1601's,
    // 7,001 frames of it, and is not run1599's.
    let (Some(rallied_w), Some(paused_w), Some(chatted_w)) = (
        window_bytes(RALLIED_WALKED[0].0),
        window_bytes(PAUSED_WALKED[0].0),
        window_bytes(CHATTED_WALKED[0].0),
    ) else {
        return;
    };
    eprintln!(
        "RALLIED WINDOW {} bytes; paused {}; chatted {}",
        rallied_w.len(),
        paused_w.len(),
        chatted_w.len()
    );
    assert!(rallied_w == paused_w, "run1602's window is run1601's");
    assert!(rallied_w != chatted_w, "and is not run1599's");
    assert_eq!(walked, RALLIED_WALKED, "the walked table");
    assert_eq!(entries, [None; 4], "the pause's functions");
    assert_eq!(ours, paused_effects(), "our effects");
    assert_eq!(theirs, paused_effects(), "the original's effects");
}

/// The frames the paused script is read on (item 4252): the pause's frame
/// and the one after it, the Citizen's first frame on the map, the first
/// it stands at the moved point, and the window's last record.
const PAUSE_READ_AT: [i64; 5] = [6712, 6713, 6814, 6844, 6999];

/// **The paused script's effects** (item 4252), our walk's values pinned
/// before the original's playback was read: the capital's gather list
/// `(4306, 31001, 0)` on the pause's frame and `(4655, 31075, 0)` from
/// 6713, the frame after it, though the three packages share one frame;
/// the Citizen out of the capital on 6814, moving, idle at the moved point
/// on 6844 and gathering by 6999. Seen red first: pinned empty.
fn paused_effects() -> Vec<GatherEffect> {
    let row =
        |frame, gather: &[(i64, i64, i64)], citizen: Option<((i64, i64), &[&'static str])>| {
            GatherEffect {
                frame,
                gather: gather.to_vec(),
                citizen: citizen.map(|(p, o)| (p, o.to_vec())),
            }
        };
    const BEFORE: &[(i64, i64, i64)] = &[(4306, 31001, 0)];
    const AT: &[(i64, i64, i64)] = &[(4655, 31075, 0)];
    vec![
        row(6712, BEFORE, None),
        row(6713, AT, None),
        row(6814, AT, Some(((3960, 31320), &["move"]))),
        row(6844, AT, Some(((4632, 31080), &[]))),
        row(6999, AT, Some(((4632, 31080), &["gather"]))),
    ]
}

/// **What the paused orders do on our walk** (item 4252), on
/// [`PAUSE_READ_AT`]: the rally point given while the game stands paused
/// is the capital's from the frame after the pause's, and the Citizen
/// walks to it.
#[test]
fn the_paused_minted_script_s_effects_on_our_walk() {
    if built_start().is_none() || minted(&crate::script::paused()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = gather_effects_ours_at(&crate::script::paused(), &PAUSE_READ_AT);
    for e in &ours {
        eprintln!("PAUSE EFFECT ours {e:?}");
    }
    assert_eq!(ours, paused_effects());
}

/// **The value diff against the previous playback** (item 4252): our walk
/// under the paused file against run1599, the chatted file's playback,
/// pinned before run1601 was read.
#[test]
fn the_paused_script_against_the_chatted_playback_parts() {
    let (take, take_trace, ..) = CHATTED_WALKED[0];
    let Some((w, _)) = walk_minted(&crate::script::paused(), take, take_trace) else {
        return;
    };
    let (frame, _, partings) = w.first_record.clone().expect("the walk parts");
    eprintln!(
        "PAUSED AGAINST CHATTED: record {frame}: {partings:?}; draws part {:?}; records {} parted {}",
        w.first_sequence, w.records, w.records_parted
    );
    let seen = (
        frame,
        partings[0].0,
        partings[0].2.clone(),
        w.first_sequence,
        w.records_parted,
    );
    let (f, kind, row, draws, parted) = PAUSED_AGAINST_CHATTED;
    assert_eq!(
        seen,
        (f, kind, row.to_string(), draws, parted),
        "PAUSED_AGAINST_CHATTED"
    );
}

/// The paused script's walk against run1599: `(the first parted record's
/// frame, its kind, its row, the first frame whose draws part, records
/// parted)`. It parts first on 6713, the frame after the pause's, at the
/// capital's rally point: ours the point given while paused, run1599's
/// the gathered one; 287 of 300 records part, and the draws from 6832,
/// where the Citizen stands at the old point on run1599 and walks on
/// toward the new one on ours. Seen red first: pinned
/// `(0, "", "", None, 0)`.
const PAUSED_AGAINST_CHATTED: (i64, &str, &str, Option<i64>, usize) = (
    6713,
    "build_diverged",
    "BuildDivergence { frame: 6713, who: 0, o: 2000, field: \"gather[0].x\", ours: 4655, theirs: 4306 }",
    Some(6832),
    287,
);

/// The paused file's playback (`docs/RUNS.md` run1601): `(take, trace,
/// frames agreeing, frames walked, records, parted)`. Pinned before run1601
/// was read with none parted, and seen red there: draws agree on all 7,000
/// frames, but 40 of the 300 records part, the first on 6866 at player 1's
/// `1/0`, `damage` ours 13 and theirs 16. That blow lands six frames
/// earlier on run1601 than on run1599, where our walk's lands
/// (`docs/CLIENT_INPUT.md` §13.4, parked 4252a). **Whole since item 4256**,
/// 40 → 0: the slinger's third swing releases from its node
/// (`docs/CLIENT_INPUT.md` §15, [`RALLIED_ROUND`]).
pub(crate) const PAUSED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1601-greatlakes-playback-of-paused-7000.txt",
    "rontrace-run1601.log",
    7000,
    7000,
    300,
    0,
)];

/// The functions a pause runs on the original's side: the package's arm,
/// the turn control's, the toggle and the notice, by entry address.
const PAUSE_FUNCTIONS: [(u32, &str); 4] = [
    (0x0094_4160, "CommandPackage::process_pause"),
    (0x0095_6990, "TurnControl::process_pause"),
    (0x0095_7ab0, "TurnControl::toggle_pause"),
    (0x0080_d9e0, "IFaceMainBase::do_pause_notice"),
];

/// Each take's first entry into [`PAUSE_FUNCTIONS`], in sim-frames, pinned
/// before run1601 was read: run1599 enters none; run1601 all four on 6711,
/// the tick that consumes [`crate::script::PAUSE_FRAME`]'s packages.
const PAUSE_ENTRIES: &[(&str, [Option<i64>; 4])] = &[
    ("rontrace-run1599.log", [None; 4]),
    ("rontrace-run1601.log", [Some(6711); 4]),
];

/// **The original plays the paused file in lockstep** (item 4252): the
/// built start walked under the pump against run1601, every frame's draws
/// against its trace and every record of its window whole
/// ([`PAUSED_WALKED`]); the pause's own functions on the original's side
/// ([`PAUSE_ENTRIES`]); and the rally point given while paused, and the
/// Citizen's walk to it, read on both sides on [`PAUSE_READ_AT`].
#[test]
fn the_original_s_playback_of_the_paused_file_walks_in_lockstep() {
    let paused = crate::script::paused();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in PAUSED_WALKED {
        let Some((w, _)) = walk_minted(&paused, take, take_trace) else {
            return;
        };
        eprintln!(
            "PAUSED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let mut entries = Vec::new();
    for &(take_trace, _) in PAUSE_ENTRIES {
        let tr = trace(take_trace).expect("the take's trace");
        let first = PAUSE_FUNCTIONS.map(|(va, _)| tr.first_entry(va));
        for ((_, name), f) in PAUSE_FUNCTIONS.iter().zip(&first) {
            eprintln!("PAUSE ENTRY {take_trace} {name}: {f:?}");
        }
        entries.push((take_trace, first));
    }
    let theirs = gather_effects_theirs_at(PAUSED_WALKED[0].0, &PAUSE_READ_AT).unwrap();
    for t in &theirs {
        eprintln!("PAUSE EFFECT theirs {t:?}");
    }
    assert_eq!(walked, PAUSED_WALKED, "the walked table");
    assert_eq!(entries, PAUSE_ENTRIES, "the pause's functions");
    assert_eq!(theirs, paused_effects(), "the original's effects");
}

/// The marketed file's gzip length (item 4259).
/// Seen red first: pinned 0, it printed 59,535.
const MARKETED_RCX_BYTES: usize = 59_535;

/// **The marketed script, minted** (item 4259, `docs/CLIENT_INPUT.md`
/// §16): the paused file and two packages more, the sale on
/// [`crate::script::SELL_FRAME`] and the purchase on
/// [`crate::script::BUY_FRAME`], each stamped one a turn; it round-trips
/// through the file. With `RON_RCX_OUT=<file>` it writes the `.rcx` the
/// playback takes open.
#[test]
fn the_marketed_minted_packages_round_trip() {
    use crate::script::{BUY_FRAME, SELL_FRAME, full_groups, marketed, paused};
    let (Some(rec), Some(first)) = (
        minted(&full_groups(&marketed())),
        minted(&full_groups(&paused())),
    ) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "marketed").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in full_groups(&marketed()).iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let added: Vec<(i32, u32)> = back
        .packages
        .iter()
        .filter(|p| !first.packages.contains(p))
        .map(|p| (p.frame, p.stamp))
        .collect();
    let last = back.packages.last().unwrap();
    assert_eq!((last.stamp, first.packages.last().unwrap().stamp), (22, 20));
    assert_eq!(
        added,
        vec![(SELL_FRAME, 20), (BUY_FRAME, 21), (last.frame, 22)],
        "the packages the paused file does not hold"
    );
    let gz = crate::recgame::write_gz(&rec);
    eprintln!("MARKETED RCX {} bytes", gz.len());
    assert_eq!(gz.len(), MARKETED_RCX_BYTES, "MARKETED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// One frame of the market's effect: slot 0's six buckets, in the
/// record's order (food, timber, wealth, knowledge, metal, oil).
#[derive(Clone, Debug, PartialEq, Eq)]
struct MarketEffect {
    frame: i64,
    bucket: [i64; 6],
}

/// The frames the market's effects are read on: the sale's frame, the
/// frame after it, the purchase's, the frame after it, and the last.
const MARKET_READ_AT: [i64; 5] = [6714, 6715, 6716, 6717, 6999];

/// Ours: the built start walked under `orders`' file, slot 0's ledger on
/// each frame of `at`.
fn market_effects_ours(orders: &[Order], at: &[i64]) -> Vec<MarketEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in at {
        tick_to(&mut built, Some(&mut stream), n);
        ours.push(MarketEffect {
            frame: n,
            bucket: built.sim.economy.ledgers[0].bucket.map(i64::from),
        });
    }
    ours
}

/// The original's: the take's `LEADERS=2` records read as text, leader
/// 0's six `bucket` lines (`LeaderDataEncrypt::log_data`, after
/// `LeaderData::log_data@006e5110`'s `set_detail(2)`) on each frame of
/// `at`. `None` when the take is not filed or prints no bucket.
fn market_effects_theirs(take: &str, at: &[i64]) -> Option<Vec<MarketEffect>> {
    use std::io::BufRead;
    let file = std::fs::File::open(dump(take)?).ok()?;
    let mut out = Vec::new();
    let (mut frame, mut who, mut got) = (-1i64, -1i64, Vec::new());
    for line in std::io::BufReader::new(file).lines() {
        let line = line.ok()?;
        let t = line.trim();
        if let Some(n) = t.strip_prefix("BEGIN FRAME ") {
            frame = n.parse().ok()?;
            who = -1;
        } else if t == "BEGIN LEADERDATA" {
            who = -2;
            got.clear();
        } else if who == -2 {
            if let Some(w) = t.strip_prefix("who ") {
                who = w.parse().ok()?;
            }
        } else if who == 0
            && at.contains(&frame)
            && let Some(b) = t.strip_prefix("bucket ")
        {
            got.push(b.parse().ok()?);
            if got.len() == 6 {
                out.push(MarketEffect {
                    frame,
                    bucket: got.clone().try_into().unwrap(),
                });
                who = -1;
            }
        }
    }
    (!out.is_empty()).then_some(out)
}

/// **The marketed script's effects** (item 4259), our walk's values
/// pinned before the original's playback was read: a hundred food out and
/// the sell price, 81, in on 6715; a hundred timber in and the buy price,
/// 107, out on 6717 (both beside the frame's gathering). Seen red first:
/// pinned empty.
fn marketed_effects() -> Vec<MarketEffect> {
    let row = |frame, bucket| MarketEffect { frame, bucket };
    vec![
        row(6714, [721, 425, 249, 0, 0, 0]),
        row(6715, [622, 425, 330, 0, 0, 0]),
        row(6716, [622, 425, 330, 0, 0, 0]),
        row(6717, [622, 526, 223, 0, 0, 0]),
        row(6999, [653, 544, 229, 0, 0, 0]),
    ]
}

/// **What the marketed orders do on our walk** (item 4259), on
/// [`MARKET_READ_AT`]; and without them, the paused file's walk, which is
/// the value diff on our side.
#[test]
fn the_marketed_minted_script_s_effects_on_our_walk() {
    if built_start().is_none() || minted(&crate::script::marketed()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = market_effects_ours(&crate::script::marketed(), &MARKET_READ_AT);
    let without = market_effects_ours(&crate::script::paused(), &MARKET_READ_AT);
    for (e, w) in ours.iter().zip(&without) {
        eprintln!("MARKET EFFECT ours {e:?} without {w:?}");
    }
    assert_eq!(ours, marketed_effects());
    assert_ne!(ours[1..], without[1..], "the orders move the buckets");
}

/// **The value diff against the previous playback** (item 4259): our walk
/// under the marketed file against run1601, the paused file's playback,
/// pinned before run1604 was read.
#[test]
fn the_marketed_script_against_the_paused_playback() {
    let (take, take_trace, ..) = PAUSED_WALKED[0];
    let Some((w, _)) = walk_minted(&crate::script::marketed(), take, take_trace) else {
        return;
    };
    eprintln!(
        "MARKETED AGAINST PAUSED: to {}; draws part {:?}; records {} parted {}, first {:?}",
        w.last, w.first_sequence, w.records, w.records_parted, w.first_record
    );
    assert_eq!(
        (w.first_sequence, w.records, w.records_parted),
        MARKETED_AGAINST_PAUSED,
        "MARKETED_AGAINST_PAUSED"
    );
}

/// The marketed script's walk against run1601: `(the first frame whose
/// draws part, records, parted)`. Whole: no draw parts in 7,000 frames
/// and none of the 300 records, because a sale and a purchase write the
/// leader's buckets and the market's prices, and the take prints neither;
/// player 1's Market, `1/2015`, reads the moved prices only from its first
/// market draw, on 8582 of this game (`docs/AI.md` §40). Seen red first:
/// pinned `(Some(0), 0, 1)`.
const MARKETED_AGAINST_PAUSED: (Option<i64>, usize, usize) = (None, 300, 0);

/// The marketed file's playback (`docs/RUNS.md` run1604): `(take, trace,
/// frames agreeing, frames walked, records, parted)`. Pinned before
/// run1604 was read, from our walk against run1601
/// ([`MARKETED_AGAINST_PAUSED`]): whole.
pub(crate) const MARKETED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1604-greatlakes-playback-of-marketed-7000.txt",
    "rontrace-run1604.log",
    7000,
    7000,
    300,
    0,
)];

/// The functions a sale and a purchase run on the original's side: the
/// package's two arms, the leader's sale, the purchase's loop and the
/// leader's purchase, by entry address.
const MARKET_FUNCTIONS: [(u32, &str); 5] = [
    (0x0094_68a0, "CommandPackage::process_sell"),
    (0x006c_fc60, "Leader::do_sell"),
    (0x0094_6a20, "CommandPackage::process_buy"),
    (0x006c_fa20, "Leader::action_buy"),
    (0x006c_fbd0, "Leader::do_buy"),
];

/// Each take's first entry into [`MARKET_FUNCTIONS`], in sim-frames,
/// pinned before run1604 was read: run1601 enters none (the AI's first
/// market draw on this game is on 8582); run1604 the sale's two on 6713,
/// the tick that consumes [`crate::script::SELL_FRAME`]'s package, and the
/// purchase's three on 6715.
const MARKET_ENTRIES: &[(&str, [Option<i64>; 5])] = &[
    ("rontrace-run1601.log", [None; 5]),
    (
        "rontrace-run1604.log",
        [Some(6713), Some(6713), Some(6715), Some(6715), Some(6715)],
    ),
];

/// **The original plays the marketed file in lockstep** (item 4259): the
/// built start walked under the pump against run1604, every frame's draws
/// against its trace and every record of its window whole
/// ([`MARKETED_WALKED`]); the market's own functions on the original's
/// side ([`MARKET_ENTRIES`]); and slot 0's buckets on
/// [`MARKET_READ_AT`], read off the take's `LEADERS=2` records, equal to
/// our walk's ([`marketed_effects`]).
#[test]
fn the_original_s_playback_of_the_marketed_file_walks_in_lockstep() {
    let marketed = crate::script::marketed();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in MARKETED_WALKED {
        let Some((w, _)) = walk_minted(&marketed, take, take_trace) else {
            return;
        };
        eprintln!(
            "MARKETED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let mut entries = Vec::new();
    for &(take_trace, _) in MARKET_ENTRIES {
        let tr = trace(take_trace).expect("the take's trace");
        let first = MARKET_FUNCTIONS.map(|(va, _)| tr.first_entry(va));
        for ((_, name), f) in MARKET_FUNCTIONS.iter().zip(&first) {
            eprintln!("MARKET ENTRY {take_trace} {name}: {f:?}");
        }
        entries.push((take_trace, first));
    }
    let theirs = market_effects_theirs(MARKETED_WALKED[0].0, &MARKET_READ_AT)
        .expect("the take prints leader 0's buckets");
    for t in &theirs {
        eprintln!("MARKET EFFECT theirs {t:?}");
    }
    assert_eq!(walked, MARKETED_WALKED, "the walked table");
    assert_eq!(entries, MARKET_ENTRIES, "the market's functions");
    assert_eq!(theirs, marketed_effects(), "the original's effects");
    // Made to fail: the paused file's walk, without the two orders, holds
    // other buckets from 6715 on, so an original that ignored them would
    // part here.
    let without = market_effects_ours(&crate::script::paused(), &MARKET_READ_AT);
    assert_ne!(theirs[1..], without[1..], "the orders' effect is visible");
}

/// **The bucket reader can see and can fail** (item 4259): on run1577's
/// `@dumpall` of frame 91, the minted file's playback, it reads leader 0's
/// six buckets as the record prints them, and our walk under the minted
/// script holds the same on that frame; on run1601, which prints no
/// leader record, it reads nothing rather than agreeing.
#[test]
fn the_bucket_reader_reads_a_known_dump() {
    let Some(theirs) = market_effects_theirs(
        "gamelog-run1577-greatlakes-playback-of-minted-checksums-120.txt",
        &[91],
    ) else {
        eprintln!("skipping: run1577 not filed");
        return;
    };
    let want = vec![MarketEffect {
        frame: 91,
        bucket: [182, 153, 102, 0, 0, 0],
    }];
    assert_eq!(theirs, want, "run1577's record");
    if built_start().is_some() && minted(&script()).is_some() {
        assert_eq!(
            market_effects_ours(&script(), &[91]),
            want,
            "our walk on 91"
        );
    }
    assert_eq!(
        market_effects_theirs(PAUSED_WALKED[0].0, &MARKET_READ_AT),
        None,
        "run1601 prints no leader record"
    );
}

/// The frames the city's gather is read on (item 4261): its frame, the
/// frame after it, and the window's last record.
const CITY_GATHER_READ_AT: [i64; 3] = [6718, 6719, 6999];

/// **The city-gathered script's effects** (item 4261), our walk's values
/// pinned before the original's playback was read: the capital's list,
/// the paused script's rally point `(4655, 31075, 0)` through 6718,
/// cleared and given one point per Farm of its city on 6719, each at the
/// Farm's own position with action 1; the Citizen `0/10` moving to a Farm
/// by 6999, where without the order it gathers at the rally point. Seen
/// red first: pinned empty.
fn city_gathered_effects() -> Vec<GatherEffect> {
    const RALLY: &[(i64, i64, i64)] = &[(4655, 31075, 0)];
    const FARMS: &[(i64, i64, i64)] = &[
        (2688, 32064, 1),
        (1920, 32064, 1),
        (2112, 31296, 1),
        (3456, 29760, 1),
    ];
    let row =
        |frame, gather: &[(i64, i64, i64)], citizen: Option<((i64, i64), &[&'static str])>| {
            GatherEffect {
                frame,
                gather: gather.to_vec(),
                citizen: citizen.map(|(p, o)| (p, o.to_vec())),
            }
        };
    vec![
        row(6718, RALLY, None),
        row(6719, FARMS, None),
        row(6999, FARMS, Some(((5572, 29304), &["move", "gather"]))),
    ]
}

/// **What the city's gather does on our walk** (item 4261), on
/// [`CITY_GATHER_READ_AT`]; and without it, the marketed file's walk,
/// which is the value diff on our side.
#[test]
fn the_city_gathered_minted_script_s_effects_on_our_walk() {
    if built_start().is_none() || minted(&crate::script::city_gathered()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = gather_effects_ours_at(&crate::script::city_gathered(), &CITY_GATHER_READ_AT);
    let without = gather_effects_ours_at(&crate::script::marketed(), &CITY_GATHER_READ_AT);
    for (e, w) in ours.iter().zip(&without) {
        eprintln!("CITY GATHER EFFECT ours {e:?} without {w:?}");
    }
    assert_eq!(ours, city_gathered_effects());
    assert_ne!(ours[1..], without[1..], "the order moves the gather list");
}

/// The city-gathered file's gzip length (item 4261). Seen red first:
/// pinned 0, it printed 59,546.
const CITY_GATHERED_RCX_BYTES: usize = 59_546;

/// **The city-gathered script, minted** (item 4261, `docs/CLIENT_INPUT.md`
/// §17): the marketed file and one package more, the capital's FARM gather
/// on [`crate::script::CITY_GATHER_FRAME`], stamped 22; it round-trips
/// through the file. With `RON_RCX_OUT=<file>` it writes the `.rcx` the
/// playback takes open.
#[test]
fn the_city_gathered_minted_packages_round_trip() {
    use crate::script::{CITY_GATHER_FRAME, city_gathered, full_groups, marketed};
    let (Some(rec), Some(first)) = (
        minted(&full_groups(&city_gathered())),
        minted(&full_groups(&marketed())),
    ) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "city-gathered").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in full_groups(&city_gathered())
        .iter()
        .zip(&back.packages[1..])
    {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let added: Vec<(i32, u32)> = back
        .packages
        .iter()
        .filter(|p| !first.packages.contains(p))
        .map(|p| (p.frame, p.stamp))
        .collect();
    let last = back.packages.last().unwrap();
    assert_eq!((last.stamp, first.packages.last().unwrap().stamp), (23, 22));
    assert_eq!(
        added,
        vec![(CITY_GATHER_FRAME, 22), (last.frame, 23)],
        "the packages the marketed file does not hold"
    );
    let gz = crate::recgame::write_gz(&rec);
    eprintln!("CITY GATHERED RCX {} bytes", gz.len());
    assert_eq!(gz.len(), CITY_GATHERED_RCX_BYTES, "CITY_GATHERED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// **The value diff against the previous playback** (item 4261): our walk
/// under the city-gathered file against run1604, the marketed file's
/// playback, pinned before the city-gathered file's playback was read.
#[test]
fn the_city_gathered_script_against_the_marketed_playback_parts() {
    let (take, take_trace, ..) = MARKETED_WALKED[0];
    let Some((w, _)) = walk_minted(&crate::script::city_gathered(), take, take_trace) else {
        return;
    };
    let (frame, _, partings) = w.first_record.clone().expect("the walk parts");
    eprintln!(
        "CITY GATHERED AGAINST MARKETED: record {frame}: {partings:?}; draws part {:?}; records {} parted {}",
        w.first_sequence, w.records, w.records_parted
    );
    let seen = (
        frame,
        partings[0].0,
        partings[0].2.clone(),
        w.first_sequence,
        w.records_parted,
    );
    let (f, kind, row, draws, parted) = CITY_GATHERED_AGAINST_MARKETED;
    assert_eq!(
        seen,
        (f, kind, row.to_string(), draws, parted),
        "CITY_GATHERED_AGAINST_MARKETED"
    );
}

/// The city-gathered script's walk against run1604: `(the first parted
/// record's frame, its kind, its row, the first frame whose draws part,
/// records parted)`. It parts first on 6719, the frame after the gather,
/// at the capital's gather list: four points on ours, one on run1604's;
/// 281 of 300 records part, and the draws from 6822. Seen red first:
/// pinned `(0, "", "", None, 0)`.
const CITY_GATHERED_AGAINST_MARKETED: (i64, &str, &str, Option<i64>, usize) = (
    6719,
    "build_diverged",
    "BuildDivergence { frame: 6719, who: 0, o: 2000, field: \"gather_len\", ours: 4, theirs: 1 }",
    Some(6822),
    281,
);

/// The city-gathered file's playback (`docs/RUNS.md` run1605): `(take,
/// trace, frames agreeing, frames walked, records, parted)`. Pinned before
/// run1605 was read: whole.
pub(crate) const CITY_GATHERED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1605-greatlakes-playback-of-city-gathered-7000.txt",
    "rontrace-run1605.log",
    7000,
    7000,
    300,
    0,
)];

/// The functions the city's gather runs on the original's side: the
/// package's arm and the group's action, by entry address.
const CITY_GATHER_FUNCTIONS: [(u32, &str); 2] = [
    (0x0094_8a10, "CommandPackage::process_city_gather"),
    (0x0070_1780, "Group::action_city_gather"),
];

/// Each take's first entry into [`CITY_GATHER_FUNCTIONS`], in sim-frames,
/// pinned before run1605 was read: run1604 enters neither; run1605 both
/// on 6717, the tick that consumes [`crate::script::CITY_GATHER_FRAME`]'s
/// package.
const CITY_GATHER_ENTRIES: &[(&str, [Option<i64>; 2])] = &[
    ("rontrace-run1604.log", [None; 2]),
    ("rontrace-run1605.log", [Some(6717), Some(6717)]),
];

/// **The original plays the city-gathered file in lockstep** (item 4261):
/// the built start walked under the pump against run1605, every frame's
/// draws against its trace and every record of its window whole
/// ([`CITY_GATHERED_WALKED`]); the gather's own functions on the
/// original's side ([`CITY_GATHER_ENTRIES`]); and the capital's gather
/// list and the Citizen `0/10` on [`CITY_GATHER_READ_AT`], read off the
/// take's records, equal to our walk's ([`city_gathered_effects`]).
#[test]
fn the_original_s_playback_of_the_city_gathered_file_walks_in_lockstep() {
    let gathered = crate::script::city_gathered();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in CITY_GATHERED_WALKED {
        let Some((w, _)) = walk_minted(&gathered, take, take_trace) else {
            return;
        };
        eprintln!(
            "CITY GATHERED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let mut entries = Vec::new();
    for &(take_trace, _) in CITY_GATHER_ENTRIES {
        let tr = trace(take_trace).expect("the take's trace");
        let first = CITY_GATHER_FUNCTIONS.map(|(va, _)| tr.first_entry(va));
        for ((_, name), f) in CITY_GATHER_FUNCTIONS.iter().zip(&first) {
            eprintln!("CITY GATHER ENTRY {take_trace} {name}: {f:?}");
        }
        entries.push((take_trace, first));
    }
    let theirs = gather_effects_theirs_at(CITY_GATHERED_WALKED[0].0, &CITY_GATHER_READ_AT)
        .expect("the take's records");
    for t in &theirs {
        eprintln!("CITY GATHER EFFECT theirs {t:?}");
    }
    assert_eq!(walked, CITY_GATHERED_WALKED, "the walked table");
    assert_eq!(entries, CITY_GATHER_ENTRIES, "the gather's functions");
    assert_eq!(theirs, city_gathered_effects(), "the original's effects");
    // Made to fail: the marketed file's walk, without the order, holds the
    // rally point from 6719 on, so an original that ignored it would part
    // here.
    let without = gather_effects_ours_at(&crate::script::marketed(), &CITY_GATHER_READ_AT);
    assert_ne!(theirs[1..], without[1..], "the order's effect is visible");
}

/// The frames the cancel is read on (item 4265): its frame, the frame
/// after it, and the window's last record.
const UNQUEUE_READ_AT: [i64; 3] = [6720, 6721, 6999];

/// What the cancel moves (item 4265, `docs/CLIENT_INPUT.md` §19): the
/// capital's live queue entries' `job_counter`s (`BuildData::queued` of
/// them) and the Citizen `0/10` the gathered script trains, its position
/// where it is on the map.
#[derive(Clone, Debug, PartialEq, Eq)]
struct QueueEffect {
    frame: i64,
    queue: Vec<i64>,
    citizen: Option<(i64, i64)>,
}

fn queue_effect_theirs(f: &crate::gamelog::Frame) -> QueueEffect {
    let cap = f
        .builds
        .iter()
        .find(|b| b.who == 0 && b.o == i64::from(crate::script::CAPITAL))
        .unwrap();
    let queued = cap.queued.unwrap_or(0) as usize;
    QueueEffect {
        frame: f.n,
        queue: cap
            .queue
            .iter()
            .take(queued)
            .map(|q| q.job_counter)
            .collect(),
        citizen: f
            .units
            .iter()
            .find(|u| u.who == 0 && u.o == i64::from(GATHERED_CITIZEN))
            .map(|u| (u.pos.x, u.pos.y)),
    }
}

fn queue_effect_ours(b: &Built) -> QueueEffect {
    let s = &b.sim;
    let cap = &s.buildings[s.building_by_o(0, crate::script::CAPITAL).unwrap()];
    QueueEffect {
        frame: s.frame,
        queue: cap
            .queue
            .items
            .iter()
            .map(|q| i64::from(q.job_counter))
            .collect(),
        citizen: s.unit_by_o(0, GATHERED_CITIZEN).map(|i| {
            let u = &s.units[i];
            (i64::from(u.pos.x), i64::from(u.pos.y))
        }),
    }
}

/// Our walk under the pump on `orders`' file, read on the frames `at`.
fn queue_effects_ours(orders: &[Order], at: &[i64]) -> Vec<QueueEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in at {
        tick_to(&mut built, Some(&mut stream), n);
        ours.push(queue_effect_ours(&built));
    }
    ours
}

/// The original's, off the take's records on the frames `at`.
fn queue_effects_theirs(take: &str, at: &[i64]) -> Option<Vec<QueueEffect>> {
    let mut source = crate::capture::indexed::IndexedCapture::open(dump(take)?).unwrap();
    let mut theirs = Vec::new();
    for i in 0..source.frames().len() {
        if at.contains(&source.frames()[i].number) {
            theirs.push(queue_effect_theirs(&source.frame_state(i).unwrap()));
        }
    }
    Some(theirs)
}

/// **The unqueued script's effects** (item 4265), our walk's values pinned
/// before the original's playback was read: the capital trains the
/// gathered script's Citizen (job 1200) on 6720; on 6721, the frame after
/// the package, its queue is empty, where without the order it reads 1300;
/// on 6999 the Citizen `0/10` is not on the map, where without the order
/// it walks at (5572, 29304). Seen red first: pinned empty.
fn unqueued_effects() -> Vec<QueueEffect> {
    let row = |frame, queue: &[i64]| QueueEffect {
        frame,
        queue: queue.to_vec(),
        citizen: None,
    };
    vec![row(6720, &[1200]), row(6721, &[]), row(6999, &[])]
}

/// **What the cancel does on our walk** (item 4265), on
/// [`UNQUEUE_READ_AT`]; and without it, the city-gathered file's walk,
/// which is the value diff on our side.
#[test]
fn the_unqueued_minted_script_s_effects_on_our_walk() {
    if built_start().is_none() || minted(&crate::script::unqueued()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = queue_effects_ours(&crate::script::unqueued(), &UNQUEUE_READ_AT);
    let without = queue_effects_ours(&crate::script::city_gathered(), &UNQUEUE_READ_AT);
    for (e, w) in ours.iter().zip(&without) {
        eprintln!("UNQUEUE EFFECT ours {e:?} without {w:?}");
    }
    assert_eq!(ours, unqueued_effects());
    assert_ne!(ours[1..], without[1..], "the order moves the queue");
}

/// The unqueued file's gzip length (item 4265). Seen red first: pinned 0,
/// it printed 59,559.
const UNQUEUED_RCX_BYTES: usize = 59_559;

/// **The unqueued script, minted** (item 4265, `docs/CLIENT_INPUT.md`
/// §19): the city-gathered file and one package more, the capital's
/// cancel on [`crate::script::UNQUEUE_FRAME`], stamped 23; it round-trips
/// through the file. With `RON_RCX_OUT=<file>` it writes the `.rcx` the
/// playback takes open.
#[test]
fn the_unqueued_minted_packages_round_trip() {
    use crate::script::{UNQUEUE_FRAME, city_gathered, full_groups, unqueued};
    let (Some(rec), Some(first)) = (
        minted(&full_groups(&unqueued())),
        minted(&full_groups(&city_gathered())),
    ) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "unqueued").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in full_groups(&unqueued()).iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let added: Vec<(i32, u32)> = back
        .packages
        .iter()
        .filter(|p| !first.packages.contains(p))
        .map(|p| (p.frame, p.stamp))
        .collect();
    let last = back.packages.last().unwrap();
    assert_eq!((last.stamp, first.packages.last().unwrap().stamp), (24, 23));
    assert_eq!(
        added,
        vec![(UNQUEUE_FRAME, 23), (last.frame, 24)],
        "the packages the city-gathered file does not hold"
    );
    let gz = crate::recgame::write_gz(&rec);
    eprintln!("UNQUEUED RCX {} bytes", gz.len());
    assert_eq!(gz.len(), UNQUEUED_RCX_BYTES, "UNQUEUED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The repeated file's gzip length (item 4267).
const REPEATED_RCX_BYTES: usize = 59_564;

/// The frames whose `group` repeats the last one sent (item 4267): the
/// gathered Citizen's queue, the paused rally point and the city's gather,
/// each the capital again.
pub(crate) const REPEATED_FRAMES: [i32; 3] = [
    crate::script::GATHER_TRAIN_FRAME,
    crate::script::PAUSE_FRAME,
    crate::script::CITY_GATHER_FRAME,
];

/// **The unqueued script as `add_group` writes it, minted** (item 4267,
/// `docs/CLIENT_INPUT.md` §20): the packages of the unqueued file, frame,
/// slot and stamp, each `group` that repeats the last one sent written as
/// a count of zero, on [`REPEATED_FRAMES`] alone; with each such group in
/// full again it is the unqueued file run1606 played, byte for byte. With
/// `RON_RCX_OUT=<file>` it writes the `.rcx` the playback takes open.
#[test]
fn the_repeated_minted_packages_round_trip() {
    use crate::script::{full_groups, unqueued};
    let (Some(rec), Some(played)) = (minted(&unqueued()), minted(&full_groups(&unqueued()))) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "repeated").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    let mut empty = Vec::new();
    for ((f, play, cmds), p) in unqueued().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        let got = crate::commands::decode(&p.data).unwrap();
        assert_eq!(&got, cmds, "frame {f}");
        if matches!(got.first(), Some(crate::commands::Command::Group { objects, .. }) if objects.is_empty())
        {
            empty.push(*f);
        }
    }
    assert_eq!(empty, REPEATED_FRAMES, "the repeated groups");
    let stamps = |r: &crate::recgame::RecGame| -> Vec<(i32, i32, u32)> {
        r.packages
            .iter()
            .map(|p| (p.frame, p.play, p.stamp))
            .collect()
    };
    assert_eq!(stamps(&back), stamps(&played), "the same packages");
    let gz = crate::recgame::write_gz(&rec);
    let old = crate::recgame::write_gz(&played);
    eprintln!(
        "REPEATED RCX {} bytes (the played file {})",
        gz.len(),
        old.len()
    );
    assert_eq!(old.len(), UNQUEUED_RCX_BYTES, "the file run1606 played");
    assert_ne!(gz, old, "not the file run1606 played");
    assert_eq!(gz.len(), REPEATED_RCX_BYTES, "REPEATED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The renamed file's gzip length (item 4268). Seen red first: pinned 0,
/// it printed 59,591.
const RENAMED_RCX_BYTES: usize = 59_591;

/// **The renamed script, minted** (item 4268, `docs/CLIENT_INPUT.md`
/// §21): the repeated file and one package more, the capital's
/// `rename_city` on [`crate::script::RENAME_FRAME`], with no group; it
/// round-trips through the file, and every package before it is the
/// repeated file's, frame, slot and stamp. With `RON_RCX_OUT=<file>` it
/// writes the `.rcx` the playback takes open.
#[test]
fn the_renamed_minted_packages_round_trip() {
    use crate::script::{RENAME_FRAME, renamed, unqueued};
    let (Some(rec), Some(before)) = (minted(&renamed()), minted(&unqueued())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "renamed").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in renamed().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let stamps = |r: &crate::recgame::RecGame| -> Vec<(i32, i32, u32)> {
        r.packages
            .iter()
            .map(|p| (p.frame, p.play, p.stamp))
            .collect()
    };
    let (ours, old) = (stamps(&back), stamps(&before));
    eprintln!(
        "RENAMED tails {:?} against {:?}",
        &ours[ours.len() - 3..],
        &old[old.len() - 2..]
    );
    // The orders' packages, then the closing one, whose stamp steps by one.
    let (n, m) = (ours.len(), old.len());
    assert_eq!((n, ours[..m - 1].to_vec()), (m + 1, old[..m - 1].to_vec()));
    assert_eq!((ours[n - 2].0, ours[n - 2].1), (RENAME_FRAME, 0));
    assert_eq!(
        (ours[n - 1].0, ours[n - 1].2),
        (old[m - 1].0, old[m - 1].2 + 1)
    );
    let gz = crate::recgame::write_gz(&rec);
    let old = crate::recgame::write_gz(&before);
    eprintln!(
        "RENAMED RCX {} bytes (the repeated file {})",
        gz.len(),
        old.len()
    );
    assert_eq!(old.len(), REPEATED_RCX_BYTES, "the file run1607 played");
    assert_eq!(gz.len(), RENAMED_RCX_BYTES, "RENAMED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The frames the rename's effect is read on (item 4268): the frame the
/// package is consumed on, the one after, and the window's last.
const RENAME_READ_AT: [i64; 3] = [6730, 6731, 6999];

/// The capital's city name on one frame (item 4268): the bare line
/// `CityData::log_data@004895c0` writes after `who` in slot 0's `CITY`
/// record, or what a rename last wrote on our walk; `None` where no name
/// is held.
#[derive(Clone, Debug, PartialEq, Eq)]
struct NameEffect {
    frame: i64,
    name: Option<String>,
}

/// Ours, on our walk of `orders` at the frames `at`.
fn name_effects_ours(orders: &[Order], at: &[i64]) -> Vec<NameEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in at {
        tick_to(&mut built, Some(&mut stream), n);
        let s = &built.sim;
        let city = s.buildings[s.building_by_o(0, crate::script::CAPITAL).unwrap()].city;
        ours.push(NameEffect {
            frame: s.frame,
            name: city.and_then(|c| s.city_name(c)).map(str::to_string),
        });
    }
    ours
}

/// The original's, off the take's `CITY` records on the frames `at`: slot
/// 0's record whose `o` is the capital, its first field after
/// `who`. The block parser keeps a bare line as a key with an empty value
/// and the rest of the line, past its first space, as the value.
fn name_effects_theirs(take: &str, at: &[i64]) -> Option<Vec<NameEffect>> {
    let text = std::fs::read_to_string(dump(take)?).unwrap();
    let log = crate::gamelog::Log::parse(&text);
    let mut theirs = Vec::new();
    for (n, frame) in log.frames() {
        if !at.contains(&n) {
            continue;
        }
        let name = frame.find("CITIES").and_then(|cities| {
            cities
                .kids("CITY")
                .find(|c| {
                    c.int("who") == Some(0) && c.int("o") == Some(i64::from(crate::script::CAPITAL))
                })
                .and_then(|c| {
                    let fields = c.fields();
                    let i = (0..fields.len()).find(|&i| fields.get(i).unwrap().0 == "who")?;
                    let (k, v) = fields.get(i + 1)?;
                    (k != "race").then(|| format!("{k} {v}").trim_end().to_string())
                })
        });
        theirs.push(NameEffect { frame: n, name });
    }
    Some(theirs)
}

/// **The renamed script's effects on our walk** (item 4268), pinned before
/// the original's playback was read: [`crate::script::RENAME_NAME`] from
/// the frame after the package. Seen red first: pinned empty. On the
/// package's frame the capital holds its founding name, `Napata`, since
/// item 4418 ported `City::generate_name` (it was `None` before); the
/// source that backs it is run1609's take, slot 0's `CITY` record on frame
/// 6730 ([`renamed_effects_theirs`], read off
/// `gamelog-run1609-greatlakes-playback-of-renamed-7000.txt`), which is
/// now equal to this row on all three frames.
fn renamed_effects_ours() -> Vec<NameEffect> {
    let row = |frame, name: &str| NameEffect {
        frame,
        name: Some(name.to_string()),
    };
    let name = crate::script::RENAME_NAME;
    vec![row(6730, "Napata"), row(6731, name), row(6999, name)]
}

/// **What the rename does on our walk** (item 4268), on
/// [`RENAME_READ_AT`]; and without it, the repeated file's walk, which is
/// the value diff on our side.
#[test]
fn the_renamed_minted_script_s_effects_on_our_walk() {
    if built_start().is_none() || minted(&crate::script::renamed()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = name_effects_ours(&crate::script::renamed(), &RENAME_READ_AT);
    let without = name_effects_ours(&crate::script::unqueued(), &RENAME_READ_AT);
    for (e, w) in ours.iter().zip(&without) {
        eprintln!("RENAME EFFECT ours {e:?} without {w:?}");
    }
    assert_eq!(ours, renamed_effects_ours());
    assert_ne!(ours[1..], without[1..], "the order writes the name");
}

/// **The original's playback of the renamed file** (item 4268), pinned
/// before run1609 was read: `(take, trace, first sequence parting or the
/// last frame, last frame, records compared, records parted)`.
pub(crate) const RENAMED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1609-greatlakes-playback-of-renamed-7000.txt",
    "rontrace-run1609.log",
    7000,
    7000,
    300,
    0,
)];

/// **The original's names**, pinned before run1609 was read: the
/// founding name `Napata` on the package's frame (run1579's capital, the
/// same lobby's), then [`crate::script::RENAME_NAME`].
fn renamed_effects_theirs() -> Vec<NameEffect> {
    let row = |frame, name: &str| NameEffect {
        frame,
        name: Some(name.to_string()),
    };
    let name = crate::script::RENAME_NAME;
    vec![row(6730, "Napata"), row(6731, name), row(6999, name)]
}

/// **The original plays the renamed file in lockstep** (item 4268,
/// `docs/CLIENT_INPUT.md` §21): the built start walked under the pump on
/// the renamed file against run1609, every frame's draws against its trace
/// and every record of its window whole ([`RENAMED_WALKED`]); and the
/// capital's name in slot 0's `CITY` record on [`RENAME_READ_AT`]
/// ([`renamed_effects_theirs`]), ours from the frame after the package on.
#[test]
fn the_original_s_playback_of_the_renamed_file_walks_in_lockstep() {
    let renamed = crate::script::renamed();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in RENAMED_WALKED {
        let Some((w, _)) = walk_minted(&renamed, take, take_trace) else {
            return;
        };
        eprintln!(
            "RENAMED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let theirs =
        name_effects_theirs(RENAMED_WALKED[0].0, &RENAME_READ_AT).expect("the take's records");
    for t in &theirs {
        eprintln!("RENAME EFFECT theirs {t:?}");
    }
    assert_eq!(walked, RENAMED_WALKED, "the walked table");
    assert_eq!(theirs, renamed_effects_theirs(), "the original's names");
    assert_eq!(
        theirs,
        renamed_effects_ours(),
        "ours: the founding name on the package's frame (item 4418), the rename after it"
    );
}

/// The hot-keyed file's gzip length (item 4270). Seen red first: pinned 0,
/// it printed 59,605.
const HOTKEYED_RCX_BYTES: usize = 59_605;

/// **The hot-keyed script, minted** (item 4270, `docs/CLIENT_INPUT.md`
/// §22): the renamed file and one package more, the capital's repeated
/// `group` and `hotkey` on [`crate::script::HOTKEY_FRAME`]; it round-trips
/// through the file, and every package before it is the renamed file's,
/// frame, slot and stamp. With `RON_RCX_OUT=<file>` it writes the `.rcx`
/// the playback takes open.
#[test]
fn the_hotkeyed_minted_packages_round_trip() {
    use crate::script::{HOTKEY_FRAME, hotkeyed, renamed};
    let (Some(rec), Some(before)) = (minted(&hotkeyed()), minted(&renamed())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "hotkeyed").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in hotkeyed().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let stamps = |r: &crate::recgame::RecGame| -> Vec<(i32, i32, u32)> {
        r.packages
            .iter()
            .map(|p| (p.frame, p.play, p.stamp))
            .collect()
    };
    let (ours, old) = (stamps(&back), stamps(&before));
    eprintln!(
        "HOTKEYED tails {:?} against {:?}",
        &ours[ours.len() - 3..],
        &old[old.len() - 2..]
    );
    let (n, m) = (ours.len(), old.len());
    assert_eq!((n, ours[..m - 1].to_vec()), (m + 1, old[..m - 1].to_vec()));
    assert_eq!((ours[n - 2].0, ours[n - 2].1), (HOTKEY_FRAME, 0));
    assert_eq!(
        (ours[n - 1].0, ours[n - 1].2),
        (old[m - 1].0, old[m - 1].2 + 1)
    );
    let gz = crate::recgame::write_gz(&rec);
    let old = crate::recgame::write_gz(&before);
    eprintln!(
        "HOTKEYED RCX {} bytes (the renamed file {})",
        gz.len(),
        old.len()
    );
    assert_eq!(old.len(), RENAMED_RCX_BYTES, "the file run1609 played");
    assert_eq!(gz.len(), HOTKEYED_RCX_BYTES, "HOTKEYED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The frames the hot key's effect is read on, on our walk's count (item
/// 4270): the frame the package is consumed on, and the two after it.
const HOTKEY_READ_AT: [i64; 3] = [6740, 6741, 6742];

/// **Where a whole dump falls on our count**: a `@dumpall` on frame `N` is
/// written inside the take's `FRAME N` block **after** that frame's
/// packages ran — run1610's block 6740 holds the hot key's slot stamped
/// 6740 — so it is our walk's state at `N + 1`, where the end-frame records
/// of the same block are its state at `N` (the renamed take's `CITY` on
/// 6730 still prints the founding name).
const DUMPALL_LAGS: i64 = 1;

/// Slot 0's hot-key group [`crate::script::HOTKEY_KEY`] on one frame (item
/// 4270): the `HOTKEYGROUPDATA` record's fields in the dump's order — its
/// `GROUPDATA`'s, each member's six arrays as `name[k]`, then `loc_x`,
/// `loc_y` and `valid` — as `(key, value)` pairs.
#[derive(Clone, Debug, PartialEq, Eq)]
struct HotKeyEffect {
    frame: i64,
    fields: Vec<(String, String)>,
}

/// The record as our walk holds it: the slot a hot key wrote, and
/// `HotKeyGroups::clear@00715230`'s values for every field `copy_group`
/// does not write (`army` and `form` −1, `priority` 1, the rest 0).
fn hotkey_record(id: i64, s: Option<&sim::groups::hotkey::HotKeySlot>) -> Vec<(String, String)> {
    let d = sim::groups::hotkey::HotKeySlot::default();
    let s = s.unwrap_or(&d);
    let mut f: Vec<(String, String)> = Vec::new();
    let mut put = |k: &str, v: String| f.push((k.to_string(), v));
    put("id", id.to_string());
    put("who", s.who.to_string());
    put("num", s.num.to_string());
    put("army", "-1".into());
    put("ox", s.ox.to_string());
    put("oy", s.oy.to_string());
    put("o_dist", s.o_dist.to_string());
    put("o_angle", s.o_angle.to_string());
    put("buildings", u8::from(s.buildings).to_string());
    put("disband", "0".into());
    put("order_num", "0".into());
    put("priority", "1".into());
    put("stamp", s.stamp.to_string());
    put("role", "0".into());
    put("form", "-1".into());
    put("think_frame", "0".into());
    put("facing", "0".into());
    put("new_speed", "0".into());
    put("speed", s.speed.to_string());
    put("form_num", "0".into());
    for k in 0..s.list.len() {
        put(&format!("list[{k}]"), s.list[k].to_string());
        put(&format!("off_x[{k}]"), s.off[k].0.to_string());
        put(&format!("off_y[{k}]"), s.off[k].1.to_string());
        put(&format!("curr_x[{k}]"), s.curr[k].0.to_string());
        put(&format!("curr_y[{k}]"), s.curr[k].1.to_string());
        put(&format!("angles[{k}]"), s.angles[k].to_string());
    }
    let float = |b: u32| -> String {
        // The dump's `%f` of the bits: 0.0 and −1.0 are the two a slot
        // holds until a screen group writes the camera's point.
        match b {
            0 => "0.000000".into(),
            0xbf80_0000 => "-1.000000".into(),
            b => format!("bits {b:#x}"),
        }
    };
    put("loc_x", float(s.loc.0));
    put("loc_y", float(s.loc.1));
    put("valid", s.valid.to_string());
    f
}

/// Ours, on our walk of `orders` at the frames `at`.
fn hotkey_effects_ours(orders: &[Order], at: &[i64]) -> Vec<HotKeyEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let key = crate::script::HOTKEY_KEY;
    let mut ours = Vec::new();
    for &n in at {
        tick_to(&mut built, Some(&mut stream), n);
        ours.push(HotKeyEffect {
            frame: built.sim.frame,
            fields: hotkey_record(i64::from(key), built.sim.hotkey_slot(key as u16)),
        });
    }
    ours
}

/// The original's, off the take's whole dumps in the `FRAME` blocks
/// `blocks` (the first `HOTKEYGROUPS` of each), each on our count
/// ([`DUMPALL_LAGS`]): the `HOTKEYGROUPDATA` whose `GROUPDATA` has `id`
/// `key`, every field
/// in order, a member's `name[scan]` numbered by its place. The nested
/// `GROUPDATA` has no `END` of its own, so the parser hands it the
/// wrapper's `loc_x`, `loc_y` and `valid` and the wrapper's `END` line;
/// the reader keeps the three once and drops the `END`.
fn hotkey_effects_theirs(take: &str, blocks: &[i64], key: i64) -> Option<Vec<HotKeyEffect>> {
    let mut source = crate::capture::indexed::IndexedCapture::open(&dump(take)?).unwrap();
    let mut theirs = Vec::new();
    for index in 0..source.frames().len() {
        let n = source.frames()[index].number;
        if !blocks.contains(&n) {
            continue;
        }
        let text = source.read_frame(index).unwrap();
        let log = crate::gamelog::Log::parse(&text);
        let (_, frame) = log.frames().into_iter().next().expect("indexed frame");
        let Some(hk) = frame.find("HOTKEYGROUPS") else {
            continue;
        };
        let Some(w) = hk
            .kids("HOTKEYGROUPDATA")
            .find(|w| w.kid("GROUPDATA").and_then(|g| g.int("id")) == Some(key))
        else {
            continue;
        };
        let mut fields = Vec::new();
        let mut scan: std::collections::BTreeMap<String, usize> = Default::default();
        for (k, v) in w.kid("GROUPDATA").unwrap().fields() {
            let k = match k.strip_suffix("[scan]") {
                Some(name) => {
                    let i = scan.entry(name.to_string()).or_default();
                    *i += 1;
                    format!("{name}[{}]", *i - 1)
                }
                None => k.to_string(),
            };
            if k != "END" {
                fields.push((k, v.trim().to_string()));
            }
        }
        theirs.push(HotKeyEffect {
            frame: n + DUMPALL_LAGS,
            fields,
        });
    }
    Some(theirs)
}

/// **The hot key's effect on our walk** (item 4270), pinned before the
/// original's playback was read: slot 1 empty on the package's frame, and
/// the capital's group from the frame after, stamped with the package's
/// frame. Seen red first: pinned empty.
fn hotkeyed_effects_ours() -> Vec<HotKeyEffect> {
    let key = i64::from(crate::script::HOTKEY_KEY);
    let capital = sim::groups::hotkey::HotKeySlot {
        who: 0,
        num: 1,
        buildings: true,
        stamp: i64::from(crate::script::HOTKEY_FRAME),
        list: vec![crate::script::CAPITAL],
        off: vec![(0, 0)],
        curr: vec![(0, 0)],
        angles: vec![0],
        ..Default::default()
    };
    vec![
        HotKeyEffect {
            frame: 6740,
            fields: hotkey_record(key, None),
        },
        HotKeyEffect {
            frame: 6741,
            fields: hotkey_record(key, Some(&capital)),
        },
        HotKeyEffect {
            frame: 6742,
            fields: hotkey_record(key, Some(&capital)),
        },
    ]
}

/// **What the hot key does on our walk** (item 4270), on
/// [`HOTKEY_READ_AT`]; and without it, the renamed file's walk, which is
/// the value diff on our side.
#[test]
fn the_hotkeyed_minted_script_s_effects_on_our_walk() {
    if built_start().is_none() || minted(&crate::script::hotkeyed()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = hotkey_effects_ours(&crate::script::hotkeyed(), &HOTKEY_READ_AT);
    let without = hotkey_effects_ours(&crate::script::renamed(), &HOTKEY_READ_AT);
    for (e, w) in ours.iter().zip(&without) {
        eprintln!("HOTKEY EFFECT ours {e:?} without {w:?}");
    }
    assert_eq!(ours, hotkeyed_effects_ours());
    assert_ne!(ours[1..], without[1..], "the order writes the slot");
}

/// run1611, the hot-keyed file played to 6745 with one whole dump in
/// block 6739: the original's slot before the package.
const HOTKEYED_BEFORE: &str = "gamelog-run1611-greatlakes-playback-of-hotkeyed-6745.txt";

/// **The original's playback of the hot-keyed file** (item 4270), pinned
/// before run1610 was read: `(take, trace, first sequence parting or the
/// last frame, last frame, records compared, records parted)`.
pub(crate) const HOTKEYED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1610-greatlakes-playback-of-hotkeyed-7000.txt",
    "rontrace-run1610.log",
    7000,
    7000,
    300,
    0,
)];

/// **The original plays the hot-keyed file in lockstep** (item 4270,
/// `docs/CLIENT_INPUT.md` §22): the built start walked under the pump on
/// the hot-keyed file against run1610, every frame's draws against its
/// trace and every record of its window whole ([`HOTKEYED_WALKED`]); and
/// slot 0's hot-key group 1 in the take's whole dumps on
/// [`HOTKEY_READ_AT`], equal to ours field for field.
#[test]
fn the_original_s_playback_of_the_hotkeyed_file_walks_in_lockstep() {
    let hotkeyed = crate::script::hotkeyed();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in HOTKEYED_WALKED {
        let Some((w, _)) = walk_minted(&hotkeyed, take, take_trace) else {
            return;
        };
        eprintln!(
            "HOTKEYED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    // run1611's whole dump in block 6739, the frame before the package, then
    // run1610's in blocks 6740 and 6741.
    let key = i64::from(crate::script::HOTKEY_KEY);
    let mut theirs = hotkey_effects_theirs(HOTKEYED_BEFORE, &[6739], key).expect("run1611");
    theirs.extend(
        hotkey_effects_theirs(HOTKEYED_WALKED[0].0, &[6740, 6741], key).expect("the take's dumps"),
    );
    for t in &theirs {
        eprintln!("HOTKEY EFFECT theirs {t:?}");
    }
    assert_eq!(walked, HOTKEYED_WALKED, "the walked table");
    assert_eq!(
        theirs,
        hotkeyed_effects_ours(),
        "the original's slot is ours"
    );
}

/// Hot-key group 55 lives outside the checksum pool. Run1825 dumps the
/// separate hot-key table on both sides of the recorded assignment.
#[test]
fn the_sample_s_unit_hotkey_slot_matches_run1825() {
    const TAKE: &str = "gamelog-run1825-sample2017-unit-hotkey-9233.txt";
    let (Some(inst), Some(rcx), Some(theirs)) = (
        install(),
        super::sample::tests::sample(),
        // Consecutive @dumpall requests print two tables in block 9233,
        // one on each side of the package. The adjacent blocks each have one.
        hotkey_effects_theirs(TAKE, &[9232, 9234], 55),
    ) else {
        return;
    };
    let rec = crate::recgame::read(&rcx).unwrap();
    let loaded = crate::load::load(&inst).unwrap();
    let mut built = crate::checksum::playback::recording_start(&loaded, &rec);
    let mut stream = crate::input::Stream::pump(&rec);
    let mut ours = Vec::new();
    for frame in [9233, 9235] {
        tick_to(&mut built, Some(&mut stream), frame);
        let slot = built.sim.hotkey_slot(55);
        if frame == 9235 {
            let slot = slot.expect("player 3 assigned hot key 55");
            assert_eq!(slot.stamp, 9233);
            assert_eq!(slot.list, [66, 67, 68, 16, 69, 74, 27]);
        }
        ours.push(HotKeyEffect {
            frame,
            fields: hotkey_record(55, slot),
        });
    }
    assert_eq!(
        ours, theirs,
        "the separate hot-key slot before and after assignment"
    );
}

/// **The original's playback of the repeated file** (item 4267), pinned
/// before run1607 was read: `(take, trace, first sequence parting or the
/// last frame, last frame, records compared, records parted)`.
pub(crate) const REPEATED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1607-greatlakes-playback-of-repeated-7000.txt",
    "rontrace-run1607.log",
    7000,
    7000,
    300,
    0,
)];

/// **The original plays the repeated file in lockstep** (item 4267,
/// `docs/CLIENT_INPUT.md` §20): the built start walked under the pump on
/// the unqueued script as the layer writes it, its three repeated groups
/// of no objects ([`REPEATED_FRAMES`]), against run1607: every frame's
/// draws against its trace and every record of its window whole
/// ([`REPEATED_WALKED`]); and the capital's queue and the Citizen `0/10` on
/// [`UNQUEUE_READ_AT`], read off the take's records, equal to run1606's
/// and to our walk's ([`unqueued_effects`]). Our walk is whole against
/// both takes ([`UNQUEUED_WALKED`]), so the original's window under the
/// repeated file is its window under the full one, record for record.
#[test]
fn the_original_s_playback_of_the_repeated_file_walks_in_lockstep() {
    let repeated = crate::script::unqueued();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in REPEATED_WALKED {
        let Some((w, _)) = walk_minted(&repeated, take, take_trace) else {
            return;
        };
        eprintln!(
            "REPEATED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let theirs =
        queue_effects_theirs(REPEATED_WALKED[0].0, &UNQUEUE_READ_AT).expect("the take's records");
    for t in &theirs {
        eprintln!("REPEATED EFFECT theirs {t:?}");
    }
    assert_eq!(walked, REPEATED_WALKED, "the walked table");
    assert_eq!(theirs, unqueued_effects(), "the original's effects");
    if let Some(full) = queue_effects_theirs(UNQUEUED_WALKED[0].0, &UNQUEUE_READ_AT) {
        assert_eq!(theirs, full, "run1606's, under the full groups");
    }
}

/// **The value diff against the previous playback** (item 4265): our walk
/// under the unqueued file against run1605, the city-gathered file's
/// playback, pinned before the unqueued file's playback was read.
#[test]
fn the_unqueued_script_against_the_city_gathered_playback_parts() {
    let (take, take_trace, ..) = CITY_GATHERED_WALKED[0];
    let Some((w, _)) = walk_minted(&crate::script::unqueued(), take, take_trace) else {
        return;
    };
    let (frame, _, partings) = w.first_record.clone().expect("the walk parts");
    eprintln!(
        "UNQUEUED AGAINST CITY GATHERED: record {frame}: {partings:?}; draws part {:?}; records {} parted {}",
        w.first_sequence, w.records, w.records_parted
    );
    let seen = (
        frame,
        partings[0].0,
        partings[0].2.clone(),
        w.first_sequence,
        w.records_parted,
    );
    let (f, kind, row, draws, parted) = UNQUEUED_AGAINST_CITY_GATHERED;
    assert_eq!(
        seen,
        (f, kind, row.to_string(), draws, parted),
        "UNQUEUED_AGAINST_CITY_GATHERED"
    );
}

/// The unqueued script's walk against run1605: `(the first parted
/// record's frame, its kind, its row, the first frame whose draws part,
/// records parted)`. It parts first on 6721, the frame after the cancel,
/// at the capital's `queued`: 0 on ours, 1 on run1605's; 279 of 300
/// records part, and the draws from 6813, the frame run1605's Citizen
/// leaves the capital. Seen red first: pinned `(0, "", "", None, 0)`.
const UNQUEUED_AGAINST_CITY_GATHERED: (i64, &str, &str, Option<i64>, usize) = (
    6721,
    "queue_diverged",
    "QueueDivergence { frame: 6721, who: 0, o: 2000, field: \"queued\", ours: 0, theirs: 1 }",
    Some(6813),
    279,
);

/// The unqueued file's playback (`docs/RUNS.md` run1606): `(take, trace,
/// frames agreeing, frames walked, records, parted)`. Pinned before
/// run1606 was read: whole.
pub(crate) const UNQUEUED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1606-greatlakes-playback-of-unqueued-7000.txt",
    "rontrace-run1606.log",
    7000,
    7000,
    300,
    0,
)];

/// The functions the cancel runs on the original's side: the package's
/// arm and the building's cancel, by entry address.
const UNQUEUE_FUNCTIONS: [(u32, &str); 2] = [
    (0x0094_66f0, "CommandPackage::process_unqueue"),
    (0x0062_0280, "Build::action_unqueue"),
];

/// Each take's first entry into [`UNQUEUE_FUNCTIONS`], in sim-frames,
/// pinned before run1606 was read: run1605 enters neither; run1606 both on
/// 6719, the tick that consumes [`crate::script::UNQUEUE_FRAME`]'s package.
const UNQUEUE_ENTRIES: &[(&str, [Option<i64>; 2])] = &[
    ("rontrace-run1605.log", [None; 2]),
    ("rontrace-run1606.log", [Some(6719), Some(6719)]),
];

/// **The original plays the unqueued file in lockstep** (item 4265): the
/// built start walked under the pump against run1606, every frame's draws
/// against its trace and every record of its window whole
/// ([`UNQUEUED_WALKED`]); the cancel's own functions on the original's
/// side ([`UNQUEUE_ENTRIES`]); and the capital's queue and the Citizen
/// `0/10` on [`UNQUEUE_READ_AT`], read off the take's records, equal to
/// our walk's ([`unqueued_effects`]).
#[test]
fn the_original_s_playback_of_the_unqueued_file_walks_in_lockstep() {
    let unqueued = crate::script::unqueued();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in UNQUEUED_WALKED {
        let Some((w, _)) = walk_minted(&unqueued, take, take_trace) else {
            return;
        };
        eprintln!(
            "UNQUEUED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let mut entries = Vec::new();
    for &(take_trace, _) in UNQUEUE_ENTRIES {
        let tr = trace(take_trace).expect("the take's trace");
        let first = UNQUEUE_FUNCTIONS.map(|(va, _)| tr.first_entry(va));
        for ((_, name), f) in UNQUEUE_FUNCTIONS.iter().zip(&first) {
            eprintln!("UNQUEUE ENTRY {take_trace} {name}: {f:?}");
        }
        entries.push((take_trace, first));
    }
    let theirs =
        queue_effects_theirs(UNQUEUED_WALKED[0].0, &UNQUEUE_READ_AT).expect("the take's records");
    for t in &theirs {
        eprintln!("UNQUEUE EFFECT theirs {t:?}");
    }
    assert_eq!(walked, UNQUEUED_WALKED, "the walked table");
    assert_eq!(entries, UNQUEUE_ENTRIES, "the cancel's functions");
    assert_eq!(theirs, unqueued_effects(), "the original's effects");
    // Made to fail: the city-gathered file's walk, without the order,
    // still trains the Citizen on 6721 and walks it on 6999, so an original
    // that ignored the cancel would part here.
    let without = queue_effects_ours(&crate::script::city_gathered(), &UNQUEUE_READ_AT);
    assert_ne!(theirs[1..], without[1..], "the order's effect is visible");
}

/// The garrisoned file's gzip length (item 4273). Seen red first: pinned
/// 0, it printed 59,620.
const GARRISONED_RCX_BYTES: usize = 59_620;

/// **The garrisoned script, minted** (item 4273, `docs/CLIENT_INPUT.md`
/// §23): the hot-keyed file and one package more, the Citizen's `group`
/// and `garrison` on [`crate::script::GARRISON_FRAME`]; it round-trips
/// through the file, and every package before it is the hot-keyed file's,
/// frame, slot and stamp. With `RON_RCX_OUT=<file>` it writes the `.rcx`
/// the playback takes open.
#[test]
fn the_garrisoned_minted_packages_round_trip() {
    use crate::script::{GARRISON_FRAME, garrisoned, hotkeyed};
    let (Some(rec), Some(before)) = (minted(&garrisoned()), minted(&hotkeyed())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "garrisoned").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in garrisoned().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let stamps = |r: &crate::recgame::RecGame| -> Vec<(i32, i32, u32)> {
        r.packages
            .iter()
            .map(|p| (p.frame, p.play, p.stamp))
            .collect()
    };
    let (ours, old) = (stamps(&back), stamps(&before));
    eprintln!(
        "GARRISONED tails {:?} against {:?}",
        &ours[ours.len() - 3..],
        &old[old.len() - 2..]
    );
    let (n, m) = (ours.len(), old.len());
    assert_eq!((n, ours[..m - 1].to_vec()), (m + 1, old[..m - 1].to_vec()));
    assert_eq!((ours[n - 2].0, ours[n - 2].1), (GARRISON_FRAME, 0));
    assert_eq!(
        (ours[n - 1].0, ours[n - 1].2),
        (old[m - 1].0, old[m - 1].2 + 1)
    );
    let gz = crate::recgame::write_gz(&rec);
    let old = crate::recgame::write_gz(&before);
    eprintln!(
        "GARRISONED RCX {} bytes (the hot-keyed file {})",
        gz.len(),
        old.len()
    );
    assert_eq!(old.len(), HOTKEYED_RCX_BYTES, "the file run1610 played");
    assert_eq!(gz.len(), GARRISONED_RCX_BYTES, "GARRISONED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The frames the garrison's effect is read on, on our walk's count (item
/// 4273): the package's frame, the one after it, the frame the Citizen is
/// first inside and the one before it, and the window's last record.
const GARRISON_READ_AT: [i64; 5] = [6750, 6751, 6766, 6767, 6999];

/// The garrisoned script, read off one frame: the Citizen
/// [`crate::script::GARRISON_CITIZEN`]'s position, what it is inside
/// (`inside_up`, −1 for nothing) and its order list by class, front first.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct GarrisonEffect {
    pub frame: i64,
    pub at: (i64, i64),
    pub inside_up: i64,
    pub orders: Vec<&'static str>,
}

/// The dump's order block names, as [`GarrisonEffect`] classes them.
fn garrison_class(name: &str) -> &'static str {
    match name {
        "GARRISONORDER" => "garrison",
        "MOVEORDER" => "move",
        "GATHERORDER" => "gather",
        _ => "other",
    }
}

fn garrison_effect_ours(b: &Built) -> GarrisonEffect {
    let s = &b.sim;
    let u = &s.units[s.unit_by_o(0, crate::script::GARRISON_CITIZEN).unwrap()];
    GarrisonEffect {
        frame: s.frame,
        at: (i64::from(u.pos.x), i64::from(u.pos.y)),
        inside_up: u.inside.map_or(-1, |k| i64::from(s.buildings[k].index)),
        orders: u
            .orders
            .iter()
            .map(|o| match &o.body {
                sim::orders::Body::Garrison { .. } => "garrison",
                sim::orders::Body::Move(_) => "move",
                sim::orders::Body::Gather(_) => "gather",
                _ => "other",
            })
            .collect(),
    }
}

/// Our walk under the pump on `orders`' file, read on `at`.
fn garrison_effects_ours(orders: &[Order], at: &[i64]) -> Vec<GarrisonEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in at {
        tick_to(&mut built, Some(&mut stream), n);
        ours.push(garrison_effect_ours(&built));
    }
    ours
}

/// The original's, off the take's records on `at`.
fn garrison_effects_theirs(take: &str, at: &[i64]) -> Option<Vec<GarrisonEffect>> {
    let mut source = crate::capture::indexed::IndexedCapture::open(dump(take)?).unwrap();
    let mut theirs = Vec::new();
    for &n in at {
        let Some(i) = source.frames().iter().position(|f| f.number == n) else {
            continue;
        };
        let f = source.frame_state(i).unwrap();
        let u = f
            .units
            .iter()
            .find(|u| u.who == 0 && u.o == i64::from(crate::script::GARRISON_CITIZEN))?;
        theirs.push(GarrisonEffect {
            frame: n,
            at: (u.pos.x, u.pos.y),
            inside_up: u.inside_up.unwrap_or(-1),
            orders: u
                .orders_front_first()
                .map(|o| garrison_class(&o.kind))
                .collect(),
        });
    }
    Some(theirs)
}

/// **What the garrison does on our walk** (item 4273), on
/// [`GARRISON_READ_AT`]; and without it, the hot-keyed file's walk.
#[test]
fn the_garrisoned_minted_script_s_effects_on_our_walk() {
    if built_start().is_none() || minted(&crate::script::garrisoned()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = garrison_effects_ours(&crate::script::garrisoned(), &GARRISON_READ_AT);
    let without = garrison_effects_ours(&crate::script::hotkeyed(), &GARRISON_READ_AT);
    for (e, w) in ours.iter().zip(&without) {
        eprintln!("GARRISON EFFECT ours {e:?} without {w:?}");
    }
    assert_eq!(ours, garrisoned_effects());
    assert_ne!(ours[1..], without[1..], "the order moves the Citizen");
}

/// **The garrisoned script's effects** (item 4273), our walk's values
/// pinned before the original's playback was read: the Citizen gathering
/// at the farm on 6750; `move`, `garrison` in front of its gather from
/// 6751, the frame after its package; still outside on 6766; inside the
/// capital `0/2000` on 6767, its gather kept; and so on the window's last
/// record. Seen red pinned empty.
fn garrisoned_effects() -> Vec<GarrisonEffect> {
    let row = |frame, at, inside_up, orders: &[&'static str]| GarrisonEffect {
        frame,
        at,
        inside_up,
        orders: orders.to_vec(),
    };
    const SENT: &[&str] = &["move", "garrison", "gather"];
    vec![
        row(6750, (3576, 29688), -1, &["gather"]),
        row(6751, (3576, 29688), -1, SENT),
        row(6766, OUTSIDE_6766, -1, SENT),
        row(6767, (3456, 30048), 2000, &["gather"]),
        row(6999, (3456, 30048), 2000, &["gather"]),
    ]
}

/// The Citizen's last point outside, on 6766.
const OUTSIDE_6766: (i64, i64) = (3456, 30048);

/// **The original's playback of the garrisoned file** (item 4273):
/// `(take, trace, first sequence parting or the last frame, last frame,
/// records compared, records parted)`. Pinned before run1613 was read with
/// no record parted; the take parted 33 of 300, from 6871 on, the Scout
/// `1/0`'s `damage` (ours 13, theirs 16) and `damage_frame` (ours 6837,
/// theirs 6870): `0/9`'s stone of 6868, loosed from piece 384's planar
/// row two units off the original's bay, flew 5 frames where it flies 4
/// (4273a). Item 4277's bays make it whole: **0 of 300 parted**
/// ([`BUILDMASKED_ROUND`], `docs/CLIENT_INPUT.md` §25). The draws agree
/// on every frame.
pub(crate) const GARRISONED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1613-greatlakes-playback-of-garrisoned-7000.txt",
    "rontrace-run1613.log",
    7000,
    7000,
    300,
    0,
)];

/// **The original plays the garrisoned file in lockstep** (item 4273,
/// `docs/CLIENT_INPUT.md` §23): the built start walked under the pump on
/// the garrisoned file against run1613, every frame's draws against its
/// trace and every record of its window whole ([`GARRISONED_WALKED`]),
/// `inside_up` among the fields compared; and the Citizen on
/// [`GARRISON_READ_AT`], read off the take's records, equal to ours.
#[test]
fn the_original_s_playback_of_the_garrisoned_file_walks_in_lockstep() {
    let garrisoned = crate::script::garrisoned();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in GARRISONED_WALKED {
        let Some((w, _)) = walk_minted(&garrisoned, take, take_trace) else {
            return;
        };
        eprintln!(
            "GARRISONED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let theirs =
        garrison_effects_theirs(GARRISONED_WALKED[0].0, &GARRISON_READ_AT).expect("the take");
    for t in &theirs {
        eprintln!("GARRISON EFFECT theirs {t:?}");
    }
    assert_eq!(walked, GARRISONED_WALKED, "the walked table");
    assert_eq!(
        theirs,
        garrisoned_effects(),
        "the original's Citizen is ours"
    );
}

/// The buildmasked file's gzip length (item 4276). Seen red first: pinned
/// 0, it printed 59,637.
const BUILDMASKED_RCX_BYTES: usize = 59_637;

/// **The buildmasked script, minted** (item 4276, `docs/CLIENT_INPUT.md`
/// §24): the garrisoned file and two packages more, the capital's Citizen
/// on [`crate::script::BUILDMASK_TRAIN_FRAME`] and the infinite-queue
/// button on [`crate::script::BUILDMASK_FRAME`]; it round-trips through
/// the file, and every package before them is the garrisoned file's,
/// frame, slot and stamp. With `RON_RCX_OUT=<file>` it writes the `.rcx`
/// the playback takes open.
#[test]
fn the_buildmasked_minted_packages_round_trip() {
    use crate::script::{BUILDMASK_FRAME, BUILDMASK_TRAIN_FRAME, buildmasked, garrisoned};
    let (Some(rec), Some(before)) = (minted(&buildmasked()), minted(&garrisoned())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "buildmasked").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in buildmasked().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let stamps = |r: &crate::recgame::RecGame| -> Vec<(i32, i32, u32)> {
        r.packages
            .iter()
            .map(|p| (p.frame, p.play, p.stamp))
            .collect()
    };
    let (ours, old) = (stamps(&back), stamps(&before));
    eprintln!(
        "BUILDMASKED tails {:?} against {:?}",
        &ours[ours.len() - 4..],
        &old[old.len() - 2..]
    );
    let (n, m) = (ours.len(), old.len());
    assert_eq!((n, ours[..m - 1].to_vec()), (m + 2, old[..m - 1].to_vec()));
    assert_eq!((ours[n - 3].0, ours[n - 3].1), (BUILDMASK_TRAIN_FRAME, 0));
    assert_eq!((ours[n - 2].0, ours[n - 2].1), (BUILDMASK_FRAME, 0));
    assert_eq!(
        (ours[n - 1].0, ours[n - 1].2),
        (old[m - 1].0, old[m - 1].2 + 2)
    );
    let gz = crate::recgame::write_gz(&rec);
    let old = crate::recgame::write_gz(&before);
    eprintln!(
        "BUILDMASKED RCX {} bytes (the garrisoned file {})",
        gz.len(),
        old.len()
    );
    assert_eq!(old.len(), GARRISONED_RCX_BYTES, "the file run1613 played");
    assert_eq!(gz.len(), BUILDMASKED_RCX_BYTES, "BUILDMASKED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// **The came-out file** (item 4280, `docs/CLIENT_INPUT.md` §26): the
/// buildmasked script and one package more, the come-out on
/// [`crate::script::COME_OUT_FRAME`]; it round-trips through the file, and
/// every package before it is the buildmasked file's, frame, slot and
/// stamp. With `RON_RCX_OUT=<file>` it writes the `.rcx` the playback
/// takes open.
#[test]
fn the_came_out_minted_packages_round_trip() {
    use crate::script::{COME_OUT_FRAME, buildmasked, came_out};
    let (Some(rec), Some(before)) = (minted(&came_out()), minted(&buildmasked())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "came-out").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in came_out().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let stamps = |r: &crate::recgame::RecGame| -> Vec<(i32, i32, u32)> {
        r.packages
            .iter()
            .map(|p| (p.frame, p.play, p.stamp))
            .collect()
    };
    let (ours, old) = (stamps(&back), stamps(&before));
    eprintln!(
        "CAME_OUT tails {:?} against {:?}",
        &ours[ours.len() - 3..],
        &old[old.len() - 2..]
    );
    let (n, m) = (ours.len(), old.len());
    assert_eq!((n, ours[..m - 1].to_vec()), (m + 1, old[..m - 1].to_vec()));
    assert_eq!((ours[n - 2].0, ours[n - 2].1), (COME_OUT_FRAME, 0));
    assert_eq!(
        (ours[n - 1].0, ours[n - 1].2),
        (old[m - 1].0, old[m - 1].2 + 1)
    );
    let gz = crate::recgame::write_gz(&rec);
    let old = crate::recgame::write_gz(&before);
    eprintln!(
        "CAME_OUT RCX {} bytes (the buildmasked file {})",
        gz.len(),
        old.len()
    );
    assert_eq!(old.len(), BUILDMASKED_RCX_BYTES, "the file run1614 played");
    assert_eq!(gz.len(), CAME_OUT_RCX_BYTES, "CAME_OUT_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The came-out file's size, gzipped (item 4280).
const CAME_OUT_RCX_BYTES: usize = 59_649;

/// The frames the buildmask's effect is read on, on our walk's count (item
/// 4276): the Citizen's package, the button's, the frame after it, and
/// the window's last record.
const BUILDMASK_READ_AT: [i64; 4] = [6760, 6762, 6763, 6999];

/// The buildmasked script, read off one frame: the capital's
/// `build_masks & 0x40` (the infinite queue) and its queue's job counters,
/// front first.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BuildmaskEffect {
    pub frame: i64,
    pub infinite: bool,
    pub queue: Vec<i64>,
}

fn buildmask_effect_ours(b: &Built) -> BuildmaskEffect {
    let s = &b.sim;
    let cap = &s.buildings[s.building_by_o(0, crate::script::CAPITAL).unwrap()];
    BuildmaskEffect {
        frame: s.frame,
        infinite: cap.queue.infinite,
        queue: cap
            .queue
            .items
            .iter()
            .map(|q| i64::from(q.job_counter))
            .collect(),
    }
}

/// Our walk under the pump on `orders`' file, read on `at`.
fn buildmask_effects_ours(orders: &[Order], at: &[i64]) -> Vec<BuildmaskEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in at {
        tick_to(&mut built, Some(&mut stream), n);
        ours.push(buildmask_effect_ours(&built));
    }
    ours
}

/// The original's, off the take's records on `at`.
fn buildmask_effects_theirs(take: &str, at: &[i64]) -> Option<Vec<BuildmaskEffect>> {
    let mut source = crate::capture::indexed::IndexedCapture::open(dump(take)?).unwrap();
    let mut theirs = Vec::new();
    for i in 0..source.frames().len() {
        if !at.contains(&source.frames()[i].number) {
            continue;
        }
        let f = source.frame_state(i).unwrap();
        let cap = f
            .builds
            .iter()
            .find(|b| b.who == 0 && b.o == i64::from(crate::script::CAPITAL))?;
        let queued = cap.queued.unwrap_or(0) as usize;
        theirs.push(BuildmaskEffect {
            frame: f.n,
            infinite: cap.build_masks? & 0x40 != 0,
            queue: cap
                .queue
                .iter()
                .take(queued)
                .map(|q| q.job_counter)
                .collect(),
        });
    }
    Some(theirs)
}

/// **What the infinite-queue button does on our walk** (item 4276), on
/// [`BUILDMASK_READ_AT`]; and without it, the walk of the garrisoned file
/// and the Citizen's package alone.
#[test]
fn the_buildmasked_minted_script_s_effects_on_our_walk() {
    if built_start().is_none() || minted(&crate::script::buildmasked()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let all = crate::script::buildmasked();
    let ours = buildmask_effects_ours(&all, &BUILDMASK_READ_AT);
    let without = buildmask_effects_ours(&all[..all.len() - 1], &BUILDMASK_READ_AT);
    for (e, w) in ours.iter().zip(&without) {
        eprintln!("BUILDMASK EFFECT ours {e:?} without {w:?}");
    }
    assert_eq!(ours, buildmasked_effects());
    assert_ne!(ours[2..], without[2..], "the order sets the bit");
}

/// **The buildmasked script's effects** (item 4276), our walk's values
/// pinned before the original's playback was read: the capital's queue
/// empty on 6760; the Citizen queued from that package, job 200 on 6762,
/// the bit still clear; set on 6763, the frame after the button's
/// package; and on 6999 the queue holds the Citizen re-queued at the
/// first one's finish (job 1900), where without the button it is empty.
/// Seen red pinned empty.
fn buildmasked_effects() -> Vec<BuildmaskEffect> {
    let row = |frame, infinite, queue: &[i64]| BuildmaskEffect {
        frame,
        infinite,
        queue: queue.to_vec(),
    };
    vec![
        row(6760, false, &[]),
        row(6762, false, &[200]),
        row(6763, true, &[300]),
        row(6999, true, &[1900]),
    ]
}

/// **The original's playback of the buildmasked file** (item 4276):
/// `(take, trace, first sequence parting or the last frame, last frame,
/// records compared, records parted)`. Pinned before run1614 was read
/// with the garrisoned file's 33 parted; the take parted 127 of 300, from
/// 6871, every one on the Scout `1/0`'s `damage` and `damage_frame`
/// alone: `0/9`'s stone of 6868 landed on 6870 in the original and 6871 on
/// ours (4273a), and the blows after it drifted. Item 4277's bays for
/// piece 384 make it whole: **0 of 300 parted** ([`BUILDMASKED_ROUND`],
/// `docs/CLIENT_INPUT.md` §25). The draws agree on every frame.
pub(crate) const BUILDMASKED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1614-greatlakes-playback-of-buildmasked-7000.txt",
    "rontrace-run1614.log",
    7000,
    7000,
    300,
    0,
)];

/// **The original plays the buildmasked file in lockstep** (item 4276,
/// `docs/CLIENT_INPUT.md` §24): the built start walked under the pump on
/// the buildmasked file against the take, every frame's draws against its
/// trace and every record of its window ([`BUILDMASKED_WALKED`]),
/// `build_masks & 0x40` among the fields compared; and the capital on
/// [`BUILDMASK_READ_AT`], read off the take's records, equal to ours.
#[test]
fn the_original_s_playback_of_the_buildmasked_file_walks_in_lockstep() {
    let buildmasked = crate::script::buildmasked();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in BUILDMASKED_WALKED {
        let Some((w, _)) = walk_minted(&buildmasked, take, take_trace) else {
            return;
        };
        eprintln!(
            "BUILDMASKED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let theirs =
        buildmask_effects_theirs(BUILDMASKED_WALKED[0].0, &BUILDMASK_READ_AT).expect("the take");
    for t in &theirs {
        eprintln!("BUILDMASK EFFECT theirs {t:?}");
    }
    assert_eq!(walked, BUILDMASKED_WALKED, "the walked table");
    assert_eq!(
        theirs,
        buildmasked_effects(),
        "the original's capital is ours"
    );
}

/// The frames the come-out's effect is read on, on our walk's count (item
/// 4280): the package's frame, the two after it, and the window's last
/// record.
const CAME_OUT_READ_AT: [i64; 4] = [6780, 6781, 6782, 6999];

/// **What the come-out does on our walk** (item 4280,
/// `docs/CLIENT_INPUT.md` §26): the Citizen
/// [`crate::script::GARRISON_CITIZEN`] on [`CAME_OUT_READ_AT`], pinned
/// before run1616 was read; seen red pinned empty.
fn came_out_effects() -> Vec<GarrisonEffect> {
    let row = |frame, at, inside_up| GarrisonEffect {
        frame,
        at,
        inside_up,
        orders: vec!["gather"],
    };
    vec![
        row(6780, (3456, 30048), 2000),
        row(6781, (3432, 29880), -1),
        row(6782, (3432, 29880), -1),
        row(6999, (3576, 29688), -1),
    ]
}

/// **The come-out on our walk**, against the buildmasked file's walk,
/// where the Citizen stays inside.
#[test]
fn what_the_come_out_does_on_our_walk() {
    if built_start().is_none() {
        eprintln!("skipping: no lobby");
        return;
    }
    let ours = garrison_effects_ours(&crate::script::came_out(), &CAME_OUT_READ_AT);
    let without = garrison_effects_ours(&crate::script::buildmasked(), &CAME_OUT_READ_AT);
    for (a, b) in ours.iter().zip(&without) {
        eprintln!("COME_OUT EFFECT ours {a:?}\n                without {b:?}");
    }
    assert_eq!(ours, came_out_effects(), "ours, pinned");
    assert_ne!(ours, without, "the order moves the Citizen");
}

/// **The original's playback of the came-out file** (item 4280):
/// `(take, trace, first sequence parting or the last frame, last frame,
/// records compared, records parted)`. Pinned before run1616 was read,
/// with none parted: the buildmasked file it extends is whole.
pub(crate) const CAME_OUT_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1616-greatlakes-playback-of-came-out-7000.txt",
    "rontrace-run1616.log",
    7000,
    7000,
    300,
    0,
)];

/// **The original plays the came-out file in lockstep** (item 4280,
/// `docs/CLIENT_INPUT.md` §26): the built start walked under the pump on
/// the came-out file against the take, every frame's draws and every
/// record of its window ([`CAME_OUT_WALKED`]), `inside_up` among the
/// fields compared; and the Citizen on [`CAME_OUT_READ_AT`], read off the
/// take's records, equal to ours.
#[test]
fn the_original_s_playback_of_the_came_out_file_walks_in_lockstep() {
    let came_out = crate::script::came_out();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in CAME_OUT_WALKED {
        let Some((w, _)) = walk_minted(&came_out, take, take_trace) else {
            return;
        };
        eprintln!(
            "CAME_OUT WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let theirs =
        garrison_effects_theirs(CAME_OUT_WALKED[0].0, &CAME_OUT_READ_AT).expect("the take");
    for t in &theirs {
        eprintln!("COME_OUT EFFECT theirs {t:?}");
    }
    assert_eq!(walked, CAME_OUT_WALKED, "the walked table");
    assert_eq!(theirs, came_out_effects(), "the original's Citizen is ours");
}

/// **The stanced file** (item 4282, `docs/CLIENT_INPUT.md` §27): the
/// came-out script and one package more, the stance press on
/// [`crate::script::STANCE_FRAME`]; it round-trips through the file, and
/// every package before it is the came-out file's, frame, slot and stamp.
/// With `RON_RCX_OUT=<file>` it writes the `.rcx` the playback takes open.
#[test]
fn the_stanced_minted_packages_round_trip() {
    use crate::script::{STANCE_FRAME, came_out, stanced};
    let (Some(rec), Some(before)) = (minted(&stanced()), minted(&came_out())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "stanced").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in stanced().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let stamps = |r: &crate::recgame::RecGame| -> Vec<(i32, i32, u32)> {
        r.packages
            .iter()
            .map(|p| (p.frame, p.play, p.stamp))
            .collect()
    };
    let (ours, old) = (stamps(&back), stamps(&before));
    eprintln!(
        "STANCED tails {:?} against {:?}",
        &ours[ours.len() - 3..],
        &old[old.len() - 2..]
    );
    let (n, m) = (ours.len(), old.len());
    assert_eq!((n, ours[..m - 1].to_vec()), (m + 1, old[..m - 1].to_vec()));
    assert_eq!((ours[n - 2].0, ours[n - 2].1), (STANCE_FRAME, 0));
    assert_eq!(
        (ours[n - 1].0, ours[n - 1].2),
        (old[m - 1].0, old[m - 1].2 + 1)
    );
    let gz = crate::recgame::write_gz(&rec);
    let old = crate::recgame::write_gz(&before);
    eprintln!(
        "STANCED RCX {} bytes (the came-out file {})",
        gz.len(),
        old.len()
    );
    assert_eq!(old.len(), CAME_OUT_RCX_BYTES, "the file run1616 played");
    assert_eq!(gz.len(), STANCED_RCX_BYTES, "STANCED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The stanced file's size, gzipped (item 4282).
const STANCED_RCX_BYTES: usize = 59_661;

/// The frames the stance's effect is read on (item 4282): the package's
/// frame, the frame after it, the first Citizen the capital trains after
/// it, and the window's last record.
const STANCE_READ_AT: [i64; 4] = [6790, 6791, 6866, 6999];

/// The first unit number slot 0 trains after the opening's: the Citizens
/// the capital's infinite queue lays (`0/10` on 6866, `0/11` on 6980).
const STANCE_TRAINED_FROM: i64 = 10;

/// The stanced script, read off one frame: the capital's `Build +0x7e`,
/// and each living unit of slot 0 from [`STANCE_TRAINED_FROM`] with its
/// `+0xb1`, which `Build::train` copies from its trainer's byte.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct StanceEffect {
    pub frame: i64,
    pub capital: i64,
    pub trained: Vec<(i64, i64)>,
}

/// Our walk under the pump on `orders`' file, read on `at`.
fn stance_effects_ours(orders: &[Order], at: &[i64]) -> Vec<StanceEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in at {
        tick_to(&mut built, Some(&mut stream), n);
        let s = &built.sim;
        let cap = s.building_by_o(0, crate::script::CAPITAL).unwrap();
        let mut trained: Vec<(i64, i64)> = s
            .units
            .iter()
            .filter(|u| u.alive() && u.owner == 0 && i64::from(u.index) >= STANCE_TRAINED_FROM)
            .map(|u| (i64::from(u.index), i64::from(u.stance)))
            .collect();
        trained.sort_unstable();
        ours.push(StanceEffect {
            frame: s.frame,
            capital: i64::from(s.build_stance(cap)),
            trained,
        });
    }
    ours
}

/// The original's, off the take's records on `at`.
fn stance_effects_theirs(take: &str, at: &[i64]) -> Option<Vec<StanceEffect>> {
    let mut source = crate::capture::indexed::IndexedCapture::open(dump(take)?).unwrap();
    let mut theirs = Vec::new();
    for &n in at {
        let Some(i) = source.frames().iter().position(|f| f.number == n) else {
            continue;
        };
        let f = source.frame_state(i).unwrap();
        let cap = f
            .builds
            .iter()
            .find(|b| b.who == 0 && b.o == i64::from(crate::script::CAPITAL))?;
        let mut trained: Vec<(i64, i64)> = f
            .units
            .iter()
            .filter(|u| u.who == 0 && u.o >= STANCE_TRAINED_FROM)
            .map(|u| (u.o, u.stance.unwrap_or(-1)))
            .collect();
        trained.sort_unstable();
        theirs.push(StanceEffect {
            frame: n,
            capital: cap.stance?,
            trained,
        });
    }
    Some(theirs)
}

/// **What the stance press does on our walk** (item 4282,
/// `docs/CLIENT_INPUT.md` §27), pinned before the original's playback was
/// read; seen red pinned empty. The press is on 6790 and the capital's
/// worker byte goes 0 → 1 on 6791; the two Citizens its infinite queue
/// trains after it, `0/10` on 6866 and `0/11` on 6980, take 1 from it
/// where the came-out file's walk gives them 0.
fn stanced_effects() -> Vec<StanceEffect> {
    let row = |frame, capital, trained: &[(i64, i64)]| StanceEffect {
        frame,
        capital,
        trained: trained.to_vec(),
    };
    vec![
        row(6790, 0, &[]),
        row(6791, 1, &[]),
        row(6866, 1, &[(10, 1)]),
        row(6999, 1, &[(10, 1), (11, 1)]),
    ]
}

/// **The stance press on our walk**, against the came-out file's walk,
/// where the capital keeps `Build::init`'s byte.
#[test]
fn what_the_stance_does_on_our_walk() {
    if built_start().is_none() || minted(&crate::script::stanced()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = stance_effects_ours(&crate::script::stanced(), &STANCE_READ_AT);
    let without = stance_effects_ours(&crate::script::came_out(), &STANCE_READ_AT);
    for (a, b) in ours.iter().zip(&without) {
        eprintln!("STANCE EFFECT ours {a:?}\n               without {b:?}");
    }
    assert_eq!(ours, stanced_effects(), "ours, pinned");
    assert_ne!(ours[1..], without[1..], "the press writes the byte");
}

/// **The original's playback of the stanced file** (item 4282):
/// `(take, trace, first sequence parting or the last frame, last frame,
/// records compared, records parted)`. Pinned before the take was read,
/// with none parted: the came-out file it extends is whole.
pub(crate) const STANCED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1617-greatlakes-playback-of-stanced-7000.txt",
    "rontrace-run1617.log",
    7000,
    7000,
    300,
    0,
)];

/// **The original plays the stanced file in lockstep** (item 4282,
/// `docs/CLIENT_INPUT.md` §27): the built start walked under the pump on
/// the stanced file against the take, every frame's draws and every record
/// of its window ([`STANCED_WALKED`]), the buildings' `stance` and the
/// units' among the fields compared; and the capital and its trained
/// Citizens on [`STANCE_READ_AT`], read off the take's records, equal to
/// ours.
#[test]
fn the_original_s_playback_of_the_stanced_file_walks_in_lockstep() {
    let stanced = crate::script::stanced();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in STANCED_WALKED {
        let Some((w, _)) = walk_minted(&stanced, take, take_trace) else {
            return;
        };
        eprintln!(
            "STANCED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let theirs = stance_effects_theirs(STANCED_WALKED[0].0, &STANCE_READ_AT).expect("the take");
    for t in &theirs {
        eprintln!("STANCE EFFECT theirs {t:?}");
    }
    assert_eq!(walked, STANCED_WALKED, "the walked table");
    assert_eq!(theirs, stanced_effects(), "the original's capital is ours");
}

/// **The toggled file** (item 4283, `docs/CLIENT_INPUT.md` §28): the
/// stanced script and one package more, the console's `ai off` on
/// [`crate::script::TOGGLE_FRAME`]; it round-trips through the file, and
/// every package before it is the stanced file's, frame, slot and stamp.
/// With `RON_RCX_OUT=<file>` it writes the `.rcx` the playback takes open.
#[test]
fn the_toggled_minted_packages_round_trip() {
    use crate::script::{TOGGLE_FRAME, stanced, toggled};
    let (Some(rec), Some(before)) = (minted(&toggled()), minted(&stanced())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "toggled").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in toggled().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let stamps = |r: &crate::recgame::RecGame| -> Vec<(i32, i32, u32)> {
        r.packages
            .iter()
            .map(|p| (p.frame, p.play, p.stamp))
            .collect()
    };
    let (ours, old) = (stamps(&back), stamps(&before));
    eprintln!(
        "TOGGLED tails {:?} against {:?}",
        &ours[ours.len() - 3..],
        &old[old.len() - 2..]
    );
    let (n, m) = (ours.len(), old.len());
    assert_eq!((n, ours[..m - 1].to_vec()), (m + 1, old[..m - 1].to_vec()));
    assert_eq!((ours[n - 2].0, ours[n - 2].1), (TOGGLE_FRAME, 0));
    assert_eq!(
        (ours[n - 1].0, ours[n - 1].2),
        (old[m - 1].0, old[m - 1].2 + 1)
    );
    let gz = crate::recgame::write_gz(&rec);
    let old = crate::recgame::write_gz(&before);
    eprintln!(
        "TOGGLED RCX {} bytes (the stanced file {})",
        gz.len(),
        old.len()
    );
    assert_eq!(old.len(), STANCED_RCX_BYTES, "the file run1617 played");
    assert_eq!(gz.len(), TOGGLED_RCX_BYTES, "TOGGLED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The toggled file's size, gzipped (item 4283).
const TOGGLED_RCX_BYTES: usize = 59_670;

/// The frames the toggle's effect is read on (item 4283): the package's
/// frame, the frame after it, the first record the stanced walk parts on
/// under the toggled file, and the window's last record.
const TOGGLE_READ_AT: [i64; 4] = [6800, 6801, 6982, 6999];

/// The toggled script, read off one frame: each living unit of slot 1,
/// the computer seat, whose current order carries a point (a move, an
/// explore or an attack-move: the dump's `MOVEORDER`, `EXPLORETOORDER`,
/// `ATTACKTOORDER`, one move body on ours), with that point, by unit
/// number.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ToggleEffect {
    pub frame: i64,
    pub moves: Vec<(i64, i64, i64)>,
}

/// Our walk under the pump on `orders`' file, read on `at`.
fn toggle_effects_ours(orders: &[Order], at: &[i64]) -> Vec<ToggleEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in at {
        tick_to(&mut built, Some(&mut stream), n);
        let s = &built.sim;
        let mut moves: Vec<(i64, i64, i64)> = s
            .units
            .iter()
            .filter(|u| u.alive() && u.owner == 1)
            .filter_map(|u| match &u.orders.front()?.body {
                sim::orders::Body::Move(m) => {
                    Some((i64::from(u.index), i64::from(m.dest.x), i64::from(m.dest.y)))
                }
                _ => None,
            })
            .collect();
        moves.sort_unstable();
        ours.push(ToggleEffect {
            frame: s.frame,
            moves,
        });
    }
    ours
}

/// The original's, off the take's records on `at`.
fn toggle_effects_theirs(take: &str, at: &[i64]) -> Option<Vec<ToggleEffect>> {
    let mut source = crate::capture::indexed::IndexedCapture::open(dump(take)?).unwrap();
    let mut theirs = Vec::new();
    for &n in at {
        let Some(i) = source.frames().iter().position(|f| f.number == n) else {
            continue;
        };
        let f = source.frame_state(i).unwrap();
        let mut moves: Vec<(i64, i64, i64)> = f
            .units
            .iter()
            .filter(|u| u.who == 1)
            .filter_map(|u| {
                let o = u.current_order()?;
                Some((u.o, o.x?, o.y?))
            })
            .collect();
        moves.sort_unstable();
        theirs.push(ToggleEffect { frame: n, moves });
    }
    Some(theirs)
}

/// **What the console's `ai off` does on our walk** (item 4283,
/// `docs/CLIENT_INPUT.md` §28), pinned before the original's playback was
/// read; seen red pinned as the stanced walk's. Nothing moves on 6800 or
/// 6801: the flag stops `Leader::production_ai`'s step machine, and slot
/// 1's units carry the orders it gave them. From 6982 the Citizens it
/// would have sent elsewhere take other points: `1/17` on 6982, and `1/1`,
/// `1/3` and `1/13` by 6999.
fn toggled_effects() -> Vec<ToggleEffect> {
    let row = |frame, moves: &[(i64, i64, i64)]| ToggleEffect {
        frame,
        moves: moves.to_vec(),
    };
    let opening = [
        (0, 4344, 32760),
        (4, 41208, 17208),
        (23, 41592, 16056),
        (28, 31800, 21816),
        (29, 28824, 15000),
    ];
    vec![
        row(6800, &opening),
        row(6801, &opening),
        row(
            6982,
            &[
                (0, 6648, 35064),
                (12, 40488, 18072),
                (17, 35448, 22968),
                (23, 36600, 20856),
                (28, 31800, 21816),
                (29, 28824, 15000),
            ],
        ),
        row(
            6999,
            &[
                (0, 6648, 35064),
                (1, 36408, 23928),
                (3, 41784, 17016),
                (5, 41208, 16440),
                (12, 40488, 18072),
                (13, 36792, 22776),
                (17, 35448, 22968),
                (23, 36600, 20856),
                (28, 31800, 21816),
                (29, 28824, 15000),
                (30, 31224, 21096),
                (31, 31176, 21240),
                (32, 31272, 20952),
            ],
        ),
    ]
}

/// **The toggle on our walk**, against the stanced file's walk, whose
/// computer seat keeps its AI; and against run1617, the stanced file's
/// playback, which our reader of the take agrees with on every frame of
/// [`TOGGLE_READ_AT`] (the comparison can fail: it is the stanced walk's
/// own value on both sides).
#[test]
fn what_the_toggle_does_on_our_walk() {
    if built_start().is_none() || minted(&crate::script::toggled()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = toggle_effects_ours(&crate::script::toggled(), &TOGGLE_READ_AT);
    let without = toggle_effects_ours(&crate::script::stanced(), &TOGGLE_READ_AT);
    for (a, b) in ours.iter().zip(&without) {
        eprintln!("TOGGLE EFFECT ours {a:?}\n                without {b:?}");
    }
    assert_eq!(ours, toggled_effects(), "ours, pinned");
    assert_eq!(ours[..2], without[..2], "nothing moves on the frame after");
    assert_ne!(ours[2..], without[2..], "the computer's Citizens part");
    if let Some(run1617) = toggle_effects_theirs(STANCED_WALKED[0].0, &TOGGLE_READ_AT) {
        assert_eq!(run1617, without, "the reader agrees with the AI on");
    }
}

/// **The original's playback of the toggled file** (item 4283):
/// `(take, trace, first sequence parting or the last frame, last frame,
/// records compared, records parted)`. Pinned before the take was read,
/// with none parted.
pub(crate) const TOGGLED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1618-greatlakes-playback-of-toggled-7000.txt",
    "rontrace-run1618.log",
    7000,
    7000,
    300,
    0,
)];

/// The toggle's handler on the original's side, by entry address:
/// `Game::action_cheat_ai_toggle` has no caller, and the handler
/// flips the flag inline (`rondata::blind`).
const TOGGLE_FUNCTIONS: [(u32, &str); 1] =
    [(0x0094_4d20, "CommandPackage::process_cheat_ai_toggle")];

/// Each take's first entry into [`TOGGLE_FUNCTIONS`], in sim-frames, pinned
/// before run1618 was read: run1617 never; run1618 on 6799, the tick that
/// consumes [`crate::script::TOGGLE_FRAME`]'s package.
const TOGGLE_ENTRIES: &[(&str, [Option<i64>; 1])] = &[
    ("rontrace-run1617.log", [None]),
    ("rontrace-run1618.log", [Some(6799)]),
];

/// **The original plays the toggled file in lockstep** (item 4283,
/// `docs/CLIENT_INPUT.md` §28): the built start walked under the pump on
/// the toggled file against the take, every frame's draws and every record
/// of its window ([`TOGGLED_WALKED`]); slot 1's pointed orders on
/// [`TOGGLE_READ_AT`], read off the take's records, equal to ours; and the
/// handler's entry on the original's side ([`TOGGLE_ENTRIES`]).
#[test]
fn the_original_s_playback_of_the_toggled_file_walks_in_lockstep() {
    let toggled = crate::script::toggled();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in TOGGLED_WALKED {
        let Some((w, _)) = walk_minted(&toggled, take, take_trace) else {
            return;
        };
        eprintln!(
            "TOGGLED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let theirs = toggle_effects_theirs(TOGGLED_WALKED[0].0, &TOGGLE_READ_AT).expect("the take");
    for t in &theirs {
        eprintln!("TOGGLE EFFECT theirs {t:?}");
    }
    let mut entries = Vec::new();
    for &(take_trace, _) in TOGGLE_ENTRIES {
        let tr = trace(take_trace).expect("the take's trace");
        let first = TOGGLE_FUNCTIONS.map(|(va, _)| tr.first_entry(va));
        eprintln!("TOGGLE ENTRY {take_trace}: {first:?}");
        entries.push((take_trace, first));
    }
    assert_eq!(walked, TOGGLED_WALKED, "the walked table");
    assert_eq!(theirs, toggled_effects(), "the original's computer is ours");
    assert_eq!(entries, TOGGLE_ENTRIES, "the handler's entry");
}

/// **The disbanded file** (item 4287, `docs/CLIENT_INPUT.md` §29): the
/// toggled script and one package more, the came-out Citizen's `group` and
/// `disband(0)` on [`crate::script::DISBAND_FRAME`]; it round-trips
/// through the file, and every package before it is the toggled file's,
/// frame, slot and stamp. With `RON_RCX_OUT=<file>` it writes the `.rcx`
/// the playback takes open.
#[test]
fn the_disbanded_minted_packages_round_trip() {
    use crate::script::{DISBAND_FRAME, disbanded, toggled};
    let (Some(rec), Some(before)) = (minted(&disbanded()), minted(&toggled())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "disbanded").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in disbanded().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let stamps = |r: &crate::recgame::RecGame| -> Vec<(i32, i32, u32)> {
        r.packages
            .iter()
            .map(|p| (p.frame, p.play, p.stamp))
            .collect()
    };
    let (ours, old) = (stamps(&back), stamps(&before));
    eprintln!(
        "DISBANDED tails {:?} against {:?}",
        &ours[ours.len() - 3..],
        &old[old.len() - 2..]
    );
    let (n, m) = (ours.len(), old.len());
    assert_eq!((n, ours[..m - 1].to_vec()), (m + 1, old[..m - 1].to_vec()));
    assert_eq!((ours[n - 2].0, ours[n - 2].1), (DISBAND_FRAME, 0));
    assert_eq!(
        (ours[n - 1].0, ours[n - 1].2),
        (old[m - 1].0, old[m - 1].2 + 1)
    );
    let gz = crate::recgame::write_gz(&rec);
    let old = crate::recgame::write_gz(&before);
    eprintln!(
        "DISBANDED RCX {} bytes (the toggled file {})",
        gz.len(),
        old.len()
    );
    assert_eq!(old.len(), TOGGLED_RCX_BYTES, "the file run1618 played");
    assert_eq!(gz.len(), DISBANDED_RCX_BYTES, "DISBANDED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The disbanded file's size, gzipped (item 4287).
const DISBANDED_RCX_BYTES: usize = 59_681;

/// The frames the disband's effect is read on (item 4287): the package's
/// frame, the frame after it, the one after that, and the window's last
/// record.
const DISBAND_READ_AT: [i64; 4] = [6810, 6811, 6812, 6999];

/// The disbanded script, read off one frame: slot 0's units on the
/// record, by unit number.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DisbandEffect {
    pub frame: i64,
    pub units: Vec<i64>,
}

/// Our walk under the pump on `orders`' file, read on `at`: slot 0's
/// living units.
fn disband_effects_ours(orders: &[Order], at: &[i64]) -> Vec<DisbandEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in at {
        tick_to(&mut built, Some(&mut stream), n);
        let s = &built.sim;
        let mut units: Vec<i64> = s
            .units
            .iter()
            .filter(|u| u.alive() && u.owner == 0)
            .map(|u| i64::from(u.index))
            .collect();
        units.sort_unstable();
        ours.push(DisbandEffect {
            frame: s.frame,
            units,
        });
    }
    ours
}

/// The original's, off the take's records on `at`.
fn disband_effects_theirs(take: &str, at: &[i64]) -> Option<Vec<DisbandEffect>> {
    let mut source = crate::capture::indexed::IndexedCapture::open(dump(take)?).unwrap();
    let mut theirs = Vec::new();
    for &n in at {
        let Some(i) = source.frames().iter().position(|f| f.number == n) else {
            continue;
        };
        let f = source.frame_state(i).unwrap();
        let mut units: Vec<i64> = f.units.iter().filter(|u| u.who == 0).map(|u| u.o).collect();
        units.sort_unstable();
        theirs.push(DisbandEffect { frame: n, units });
    }
    Some(theirs)
}

/// **What the disband does on our walk** (item 4287,
/// `docs/CLIENT_INPUT.md` §29), pinned before the original's playback was
/// read; seen red pinned as the toggled walk's. The Citizen `0/2` stands
/// on 6810 and is off slot 0's living units from 6811, the tick that takes
/// the package; nothing else of slot 0's moves, and the capital's infinite
/// queue still trains `0/10` and `0/11` by 6999.
fn disbanded_effects() -> Vec<DisbandEffect> {
    let row = |frame, units: &[i64]| DisbandEffect {
        frame,
        units: units.to_vec(),
    };
    vec![
        row(6810, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
        row(6811, &[0, 1, 3, 4, 5, 6, 7, 8, 9]),
        row(6812, &[0, 1, 3, 4, 5, 6, 7, 8, 9]),
        row(6999, &[0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
    ]
}

/// **The disband on our walk**, against the toggled file's walk, which
/// keeps the Citizen; and against run1618, the toggled file's playback,
/// which our reader of the take agrees with on every frame of
/// [`DISBAND_READ_AT`] (the comparison can fail: it is the toggled walk's
/// own value on both sides).
#[test]
fn what_the_disband_does_on_our_walk() {
    if built_start().is_none() || minted(&crate::script::disbanded()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = disband_effects_ours(&crate::script::disbanded(), &DISBAND_READ_AT);
    let without = disband_effects_ours(&crate::script::toggled(), &DISBAND_READ_AT);
    for (a, b) in ours.iter().zip(&without) {
        eprintln!("DISBAND EFFECT ours {a:?}\n                without {b:?}");
    }
    assert_eq!(ours, disbanded_effects(), "ours, pinned");
    assert_eq!(ours[0], without[0], "nothing moves on the package's frame");
    assert_ne!(ours[1..], without[1..], "the Citizen is gone after");
    if let Some(run1618) = disband_effects_theirs(TOGGLED_WALKED[0].0, &DISBAND_READ_AT) {
        assert_eq!(run1618, without, "the reader agrees without the order");
    }
}

/// **The original's playback of the disbanded file** (item 4287):
/// `(take, trace, first sequence parting or the last frame, last frame,
/// records compared, records parted)`. Pinned before the take was read,
/// with none parted. run1623 is run1622's line with `DEATHS=1`: run1622
/// printed no `DEATH_OBJS`, so the comparer read our corpse of `0/2` as
/// one the original does not hold, on 189 of its 300 records.
pub(crate) const DISBANDED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1623-greatlakes-playback-of-disbanded-7000.txt",
    "rontrace-run1623.log",
    7000,
    7000,
    300,
    0,
)];

/// The disband's handler and the object's on the original's side, by
/// entry address.
const DISBAND_FUNCTIONS: [(u32, &str); 2] = [
    (0x0094_8660, "CommandPackage::process_disband"),
    (0x0064_55c0, "Object::disband"),
];

/// Each take's first entry into [`DISBAND_FUNCTIONS`], in sim-frames,
/// pinned before run1622 was read: run1618 never; run1622 and run1623 on 6809, the
/// tick that consumes [`crate::script::DISBAND_FRAME`]'s package.
const DISBAND_ENTRIES: &[(&str, [Option<i64>; 2])] = &[
    ("rontrace-run1618.log", [None, None]),
    ("rontrace-run1622.log", [Some(6809), Some(6809)]),
    ("rontrace-run1623.log", [Some(6809), Some(6809)]),
];

/// **The original plays the disbanded file in lockstep** (item 4287,
/// `docs/CLIENT_INPUT.md` §29): the built start walked under the pump on
/// the disbanded file against the take, every frame's draws and every
/// record of its window ([`DISBANDED_WALKED`]); slot 0's units on
/// [`DISBAND_READ_AT`], read off the take's records, equal to ours; and
/// the handler's entry on the original's side ([`DISBAND_ENTRIES`]).
#[test]
fn the_original_s_playback_of_the_disbanded_file_walks_in_lockstep() {
    let disbanded = crate::script::disbanded();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in DISBANDED_WALKED {
        let Some((w, _)) = walk_minted(&disbanded, take, take_trace) else {
            return;
        };
        eprintln!(
            "DISBANDED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let theirs = disband_effects_theirs(DISBANDED_WALKED[0].0, &DISBAND_READ_AT).expect("the take");
    for t in &theirs {
        eprintln!("DISBAND EFFECT theirs {t:?}");
    }
    let mut entries = Vec::new();
    for &(take_trace, _) in DISBAND_ENTRIES {
        let tr = trace(take_trace).expect("the take's trace");
        let first = DISBAND_FUNCTIONS.map(|(va, _)| tr.first_entry(va));
        eprintln!("DISBAND ENTRY {take_trace}: {first:?}");
        entries.push((take_trace, first));
    }
    assert_eq!(walked, DISBANDED_WALKED, "the walked table");
    assert_eq!(theirs, disbanded_effects(), "the original's units are ours");
    assert_eq!(entries, DISBAND_ENTRIES, "the handler's entry");
}

/// **The auto-explored file** (item 4288, `docs/CLIENT_INPUT.md` §31): the
/// disbanded script and one package more, the Scout's `group` and
/// `unitmask(0x100, −1)` on [`crate::script::AUTO_EXPLORE_FRAME`]; it
/// round-trips through the file, and every package before it is the
/// disbanded file's, frame, slot and stamp. With `RON_RCX_OUT=<file>` it
/// writes the `.rcx` a playback takes open.
#[test]
fn the_auto_explored_minted_packages_round_trip() {
    use crate::script::{AUTO_EXPLORE_FRAME, auto_explored, disbanded};
    let (Some(rec), Some(before)) = (minted(&auto_explored()), minted(&disbanded())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "auto-explored").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in auto_explored().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let stamps = |r: &crate::recgame::RecGame| -> Vec<(i32, i32, u32)> {
        r.packages
            .iter()
            .map(|p| (p.frame, p.play, p.stamp))
            .collect()
    };
    let (ours, old) = (stamps(&back), stamps(&before));
    eprintln!(
        "AUTO_EXPLORED tails {:?} against {:?}",
        &ours[ours.len() - 3..],
        &old[old.len() - 2..]
    );
    let (n, m) = (ours.len(), old.len());
    assert_eq!((n, ours[..m - 1].to_vec()), (m + 1, old[..m - 1].to_vec()));
    assert_eq!((ours[n - 2].0, ours[n - 2].1), (AUTO_EXPLORE_FRAME, 0));
    assert_eq!(
        (ours[n - 1].0, ours[n - 1].2),
        (old[m - 1].0, old[m - 1].2 + 1)
    );
    let gz = crate::recgame::write_gz(&rec);
    let old = crate::recgame::write_gz(&before);
    eprintln!(
        "AUTO_EXPLORED RCX {} bytes (the disbanded file {})",
        gz.len(),
        old.len()
    );
    assert_eq!(old.len(), DISBANDED_RCX_BYTES, "the file run1623 played");
    assert_eq!(gz.len(), AUTO_EXPLORED_RCX_BYTES, "AUTO_EXPLORED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The auto-explored file's gzipped length (`docs/CLIENT_INPUT.md` §31.3).
const AUTO_EXPLORED_RCX_BYTES: usize = 59_692;

/// **The picked file** (item 4352, `docs/CLIENT_INPUT.md` §32.11): the
/// auto-explored script and three packages more, one per pick button, on
/// [`crate::script::PICK_MOVE_FRAME`], `PICK_REPAIR_FRAME` and
/// `PICK_ATTACK_FRAME`; it round-trips through the file, and every package
/// before them is the auto-explored file's, frame, slot and stamp. With
/// `RON_RCX_OUT=<file>` it writes the `.rcx` a playback takes open.
#[test]
fn the_picked_minted_packages_round_trip() {
    use crate::script::{
        PICK_ATTACK_FRAME, PICK_MOVE_FRAME, PICK_REPAIR_FRAME, auto_explored, picked,
    };
    let (Some(rec), Some(before)) = (minted(&picked()), minted(&auto_explored())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "picked").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in picked().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let stamps = |r: &crate::recgame::RecGame| -> Vec<(i32, i32, u32)> {
        r.packages
            .iter()
            .map(|p| (p.frame, p.play, p.stamp))
            .collect()
    };
    let (ours, old) = (stamps(&back), stamps(&before));
    eprintln!(
        "PICKED tails {:?} against {:?}",
        &ours[ours.len() - 5..],
        &old[old.len() - 2..]
    );
    let (n, m) = (ours.len(), old.len());
    assert_eq!((n, ours[..m - 1].to_vec()), (m + 3, old[..m - 1].to_vec()));
    let frames: Vec<(i32, i32)> = ours[n - 4..n - 1].iter().map(|p| (p.0, p.1)).collect();
    assert_eq!(
        frames,
        [
            (PICK_MOVE_FRAME, 0),
            (PICK_REPAIR_FRAME, 0),
            (PICK_ATTACK_FRAME, 0)
        ]
    );
    assert_eq!(
        (ours[n - 1].0, ours[n - 1].2),
        (old[m - 1].0, old[m - 1].2 + 3)
    );
    let gz = crate::recgame::write_gz(&rec);
    let old = crate::recgame::write_gz(&before);
    eprintln!(
        "PICKED RCX {} bytes (the auto-explored file {})",
        gz.len(),
        old.len()
    );
    assert_eq!(
        old.len(),
        AUTO_EXPLORED_RCX_BYTES,
        "the file run1627 played"
    );
    assert_eq!(gz.len(), PICKED_RCX_BYTES, "PICKED_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The picked file's gzipped length (`docs/CLIENT_INPUT.md` §32.11).
const PICKED_RCX_BYTES: usize = 59_729;

/// The units the picks select, each pick's own: the Move pick's Citizen,
/// the Repair pick's and the Attack pick's.
const PICKED_UNITS: [i16; 3] = [
    crate::script::PICK_MOVE_UNIT,
    crate::script::PICK_REPAIR_UNIT,
    crate::script::PICK_ATTACK_UNIT,
];

/// The frames each pick's effect is read on, both sides: the record after
/// each package's tick, and three across the window.
const PICK_READ_AT: [i64; 6] = [6831, 6841, 6851, 6900, 7200, 7499];

/// One picked unit on one frame: where it stands (`None` once it is gone)
/// and its head order's `OrderIndex`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PickEffect {
    pub frame: i64,
    pub o: i16,
    pub pos: Option<(i64, i64)>,
    pub head: Option<i64>,
}

/// Our walk under the pump on `orders`' file, read on `at`.
fn pick_effects_ours(orders: &[Order], at: &[i64]) -> Vec<PickEffect> {
    unit_effects_ours(orders, at, &PICKED_UNITS)
}

/// [`pick_effects_ours`] for any of slot 0's units.
fn unit_effects_ours(orders: &[Order], at: &[i64], units: &[i16]) -> Vec<PickEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in at {
        tick_to(&mut built, Some(&mut stream), n);
        let s = &built.sim;
        for &o in units {
            let u = s.unit_by_o(0, o).map(|i| &s.units[i]).filter(|u| u.alive());
            ours.push(PickEffect {
                frame: s.frame,
                o,
                pos: u.map(|u| (i64::from(u.pos.x), i64::from(u.pos.y))),
                head: u.and_then(|u| u.orders.front().map(|o| i64::from(o.index()))),
            });
        }
    }
    ours
}

/// The original's, off the take's records on `at`.
fn pick_effects_theirs(take: &str, at: &[i64]) -> Option<Vec<PickEffect>> {
    unit_effects_theirs(take, at, &PICKED_UNITS)
}

/// [`pick_effects_theirs`] for any of slot 0's units.
fn unit_effects_theirs(take: &str, at: &[i64], units: &[i16]) -> Option<Vec<PickEffect>> {
    let mut source = crate::capture::indexed::IndexedCapture::open(dump(take)?).unwrap();
    let mut theirs = Vec::new();
    for &n in at {
        let Some(i) = source.frames().iter().position(|f| f.number == n) else {
            continue;
        };
        let f = source.frame_state(i).unwrap();
        for &o in units {
            let u = f.units.iter().find(|u| u.who == 0 && u.o == i64::from(o));
            theirs.push(PickEffect {
                frame: n,
                o,
                pos: u.map(|u| (u.pos.x, u.pos.y)),
                head: u.and_then(|u| u.current_order().map(|o| o.index)),
            });
        }
    }
    Some(theirs)
}

/// **What the picks do on our walk** (item 4352, `docs/CLIENT_INPUT.md`
/// §32.11), pinned off our walk's print before run1656's records were
/// read on these frames.
fn picked_effects() -> Vec<PickEffect> {
    let row = |frame, o, pos, head| PickEffect {
        frame,
        o,
        pos,
        head,
    };
    vec![
        row(6831, 4, Some((2640, 31797)), Some(1)),
        row(6831, 5, Some((1848, 31608)), Some(7)),
        row(6831, 3, Some((2232, 32184)), Some(7)),
        row(6841, 4, Some((2880, 31766)), Some(1)),
        row(6841, 5, Some((1848, 31608)), Some(1)),
        row(6841, 3, Some((2040, 32248)), Some(1)),
        row(6851, 4, Some((3120, 31726)), Some(1)),
        row(6851, 5, Some((2062, 31735)), Some(1)),
        row(6851, 3, Some((1848, 32328)), Some(1)),
        row(6900, 4, Some((3323, 31830)), Some(1)),
        row(6900, 5, Some((3199, 32052)), Some(1)),
        row(6900, 3, Some((2568, 32520)), Some(10)),
        row(7200, 4, Some((1176, 30072)), Some(11)),
        row(7200, 5, Some((3576, 29688)), Some(7)),
        row(7200, 3, Some((1224, 29112)), None),
        row(7499, 4, Some((1176, 30072)), Some(11)),
        row(7499, 5, Some((3384, 29880)), Some(7)),
        row(7499, 3, Some((4200, 28296)), Some(7)),
    ]
}

/// **The picks on our walk and on the original's** (item 4352): the three
/// picked units on [`PICK_READ_AT`], ours pinned, against the
/// auto-explored file's walk, where none of them takes the pick's order;
/// and run1656's records, equal to ours.
#[test]
fn what_the_picks_do_on_both_sides() {
    if built_start().is_none() || minted(&crate::script::picked()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = pick_effects_ours(&crate::script::picked(), &PICK_READ_AT);
    let without = pick_effects_ours(&crate::script::auto_explored(), &PICK_READ_AT);
    for (a, b) in ours.iter().zip(&without) {
        eprintln!("PICK EFFECT ours {a:?}\n            without {b:?}");
    }
    assert_eq!(ours, picked_effects(), "ours, pinned");
    for o in PICKED_UNITS {
        let of = |v: &[PickEffect]| -> Vec<Option<(i64, i64)>> {
            v.iter().filter(|e| e.o == o).map(|e| e.pos).collect()
        };
        assert_ne!(of(&ours), of(&without), "0/{o}'s path parts after its pick");
    }
    let Some(theirs) = pick_effects_theirs(PICKED_WALKED[0].0, &PICK_READ_AT) else {
        return;
    };
    for t in &theirs {
        eprintln!("PICK EFFECT theirs {t:?}");
    }
    assert_eq!(theirs, ours, "the original's picked units are ours");
}

/// **The original's playback of the picked file** (item 4352,
/// `docs/CLIENT_INPUT.md` §32.11): `(take, trace, first sequence parting
/// or the last frame, last frame, records compared, records parted)`.
/// Pinned before run1656 was read, with none parted.
pub(crate) const PICKED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1656-greatlakes-playback-of-picked-7500.txt",
    "rontrace-run1656.log",
    7500,
    7500,
    700,
    0,
)];

/// **The original plays the picked file in lockstep** (item 4352): the
/// built start walked under the pump on the picked file against the take,
/// every frame's draws and every record of its window ([`PICKED_WALKED`]).
#[test]
fn the_original_s_playback_of_the_picked_file_walks_in_lockstep() {
    let picked = crate::script::picked();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in PICKED_WALKED {
        let Some((w, _)) = walk_minted(&picked, take, take_trace) else {
            return;
        };
        eprintln!(
            "PICKED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        if let Some((o, t)) = &w.sites
            && let Some((_, d)) = first_parting(o, t)
        {
            eprintln!("the first parting frame, {:?}: {d}", w.first_sequence);
        }
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    assert_eq!(walked, PICKED_WALKED, "the walked table");
}

/// **The arms' file** (item 4361, `docs/CLIENT_INPUT.md` §32.13): the
/// picked script and five packages more, the picks' arms the picked file
/// does not reach, on [`crate::script::ARM_MOVE_FRAME`] to
/// `ARM_SITE_FRAME`; it round-trips through the file, and every package
/// before them is the picked file's, frame, slot and stamp. With
/// `RON_RCX_OUT=<file>` it writes the `.rcx` a playback takes open.
#[test]
fn the_arms_minted_packages_round_trip() {
    use crate::script::{
        ARM_ATTACK_FRAME, ARM_BUILD_FRAME, ARM_GUARD_FRAME, ARM_MOVE_FRAME, ARM_SITE_FRAME, picked,
        picked_arms,
    };
    let (Some(rec), Some(before)) = (minted(&picked_arms()), minted(&picked())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "arms").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in picked_arms().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let stamps = |r: &crate::recgame::RecGame| -> Vec<(i32, i32, u32)> {
        r.packages
            .iter()
            .map(|p| (p.frame, p.play, p.stamp))
            .collect()
    };
    let (ours, old) = (stamps(&back), stamps(&before));
    eprintln!(
        "ARMS tails {:?} against {:?}",
        &ours[ours.len() - 7..],
        &old[old.len() - 2..]
    );
    let (n, m) = (ours.len(), old.len());
    assert_eq!((n, ours[..m - 1].to_vec()), (m + 5, old[..m - 1].to_vec()));
    let frames: Vec<(i32, i32)> = ours[n - 6..n - 1].iter().map(|p| (p.0, p.1)).collect();
    assert_eq!(
        frames,
        [
            (ARM_MOVE_FRAME, 0),
            (ARM_GUARD_FRAME, 0),
            (ARM_ATTACK_FRAME, 0),
            (ARM_BUILD_FRAME, 0),
            (ARM_SITE_FRAME, 0)
        ]
    );
    assert_eq!(
        (ours[n - 1].0, ours[n - 1].2),
        (old[m - 1].0, old[m - 1].2 + 5)
    );
    let gz = crate::recgame::write_gz(&rec);
    let old = crate::recgame::write_gz(&before);
    eprintln!(
        "ARMS RCX {} bytes (the picked file {})",
        gz.len(),
        old.len()
    );
    assert_eq!(old.len(), PICKED_RCX_BYTES, "the file run1656 played");
    assert_eq!(gz.len(), ARMS_RCX_BYTES, "ARMS_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The arms' file's gzipped length (`docs/CLIENT_INPUT.md` §32.13).
const ARMS_RCX_BYTES: usize = 59_813;

/// The units the arms select, each arm's own: the Move pick's ground,
/// the guard, the Attack pick's ground, the Farm's builder and the site's
/// Repair pick.
const ARMED_UNITS: [i16; 5] = [
    crate::script::ARM_MOVE_UNIT,
    crate::script::ARM_GUARD_UNIT,
    crate::script::ARM_ATTACK_UNIT,
    crate::script::ARM_BUILD_UNIT,
    crate::script::ARM_SITE_UNIT,
];

/// The frames each arm's effect is read on, both sides: the record after
/// each package's tick, and three across the window.
const ARM_READ_AT: [i64; 8] = [6861, 6871, 6881, 6891, 6901, 7000, 7200, 7499];

/// **What the arms do on our walk** (item 4361, `docs/CLIENT_INPUT.md`
/// §32.13), pinned off our walk's print before run1663's records were
/// read on these frames.
fn arm_effects() -> Vec<PickEffect> {
    let row = |frame, o, pos, head| PickEffect {
        frame,
        o,
        pos,
        head,
    };
    vec![
        row(6861, 1, Some((4923, 29041)), Some(1)),
        row(6861, 6, Some((4728, 29016)), Some(7)),
        row(6861, 10, Some((2527, 31536)), Some(1)),
        row(6861, 4, Some((3240, 31704)), Some(11)),
        row(6861, 5, Some((2282, 31855)), Some(1)),
        row(6871, 1, Some((4955, 29291)), Some(1)),
        row(6871, 6, Some((4710, 29038)), Some(2)),
        row(6871, 10, Some((2607, 31776)), Some(1)),
        row(6871, 4, Some((3240, 31704)), Some(11)),
        row(6871, 5, Some((2503, 31972)), Some(1)),
        row(6881, 1, Some((4995, 29541)), Some(1)),
        row(6881, 6, Some((4530, 29266)), Some(2)),
        row(6881, 10, Some((2643, 31870)), Some(2)),
        row(6881, 4, Some((3240, 31704)), Some(11)),
        row(6881, 5, Some((2743, 32005)), Some(1)),
        row(6891, 1, Some((5035, 29791)), Some(1)),
        row(6891, 6, Some((4350, 29494)), Some(2)),
        row(6891, 10, Some((2763, 32090)), Some(2)),
        row(6891, 4, Some((3261, 31718)), Some(1)),
        row(6891, 5, Some((2983, 32035)), Some(1)),
        row(6901, 1, Some((5075, 30041)), Some(1)),
        row(6901, 6, Some((4170, 29722)), Some(2)),
        row(6901, 10, Some((2883, 32310)), Some(2)),
        row(6901, 4, Some((3471, 31858)), Some(1)),
        row(6901, 5, Some((3223, 32052)), Some(1)),
        row(7000, 1, Some((5148, 30423)), None),
        row(7000, 6, Some((2979, 32008)), Some(2)),
        row(7000, 10, Some((3306, 33036)), None),
        row(7000, 4, Some((3768, 32040)), Some(6)),
        row(7000, 5, Some((3576, 32136)), Some(6)),
        row(7200, 1, Some((4095, 28725)), Some(1)),
        row(7200, 6, Some((640, 28731)), Some(2)),
        row(7200, 10, Some((4137, 30725)), Some(1)),
        row(7200, 4, Some((3768, 31992)), Some(7)),
        row(7200, 5, Some((1908, 32124)), Some(1)),
        row(7499, 1, Some((4536, 27624)), Some(7)),
        row(7499, 6, Some((5178, 26534)), Some(2)),
        row(7499, 10, Some((4248, 28296)), Some(7)),
        row(7499, 4, Some((3960, 32184)), Some(7)),
        row(7499, 5, Some((1848, 31800)), Some(7)),
    ]
}

/// **The arms on our walk and on the original's** (item 4361): the five
/// armed units on [`ARM_READ_AT`], ours pinned, against the picked file's
/// walk, where none of them takes the arm's order; and run1663's records,
/// equal to ours.
#[test]
fn what_the_arms_do_on_both_sides() {
    if built_start().is_none() || minted(&crate::script::picked_arms()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = unit_effects_ours(&crate::script::picked_arms(), &ARM_READ_AT, &ARMED_UNITS);
    let without = unit_effects_ours(&crate::script::picked(), &ARM_READ_AT, &ARMED_UNITS);
    for (a, b) in ours.iter().zip(&without) {
        eprintln!("ARM EFFECT ours {a:?}\n           without {b:?}");
    }
    assert_eq!(ours, arm_effects(), "ours, pinned");
    for o in ARMED_UNITS {
        let of = |v: &[PickEffect]| -> Vec<Option<(i64, i64)>> {
            v.iter().filter(|e| e.o == o).map(|e| e.pos).collect()
        };
        assert_ne!(of(&ours), of(&without), "0/{o}'s path parts after its arm");
    }
    let Some(theirs) = unit_effects_theirs(ARMS_WALKED[0].0, &ARM_READ_AT, &ARMED_UNITS) else {
        return;
    };
    for t in &theirs {
        eprintln!("ARM EFFECT theirs {t:?}");
    }
    assert_eq!(theirs, ours, "the original's armed units are ours");
}

/// **The original's playback of the arms' file** (item 4361,
/// `docs/CLIENT_INPUT.md` §32.13): `(take, trace, first sequence parting
/// or the last frame, last frame, records compared, records parted)`.
/// Pinned before run1663 was read, with none parted.
pub(crate) const ARMS_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1663-greatlakes-playback-of-arms-7500.txt",
    "rontrace-run1663.log",
    7500,
    7500,
    700,
    0,
)];

/// **The original plays the arms' file in lockstep** (item 4361): the
/// built start walked under the pump on the arms' file against the take,
/// every frame's draws and every record of its window ([`ARMS_WALKED`]).
#[test]
fn the_original_s_playback_of_the_arms_file_walks_in_lockstep() {
    let arms = crate::script::picked_arms();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in ARMS_WALKED {
        let Some((w, _)) = walk_minted(&arms, take, take_trace) else {
            return;
        };
        eprintln!(
            "ARMS WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        if let Some((o, t)) = &w.sites
            && let Some((_, d)) = first_parting(o, t)
        {
            eprintln!("the first parting frame, {:?}: {d}", w.first_sequence);
        }
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    assert_eq!(walked, ARMS_WALKED, "the walked table");
}

/// **The transport press's packages round-trip** (item 4375,
/// `docs/CLIENT_INPUT.md` §32.14): the arms' script and one package more,
/// `group [10]`, `set_transport(1)` on [`crate::script::TRANSPORT_FRAME`];
/// every package before it is the arms' file's, frame, slot and stamp.
/// With `RON_RCX_OUT=<file>` it writes the `.rcx` a playback takes open.
#[test]
fn the_transport_minted_packages_round_trip() {
    use crate::script::{TRANSPORT_FRAME, picked_arms, picked_transport};
    let (Some(rec), Some(before)) = (minted(&picked_transport()), minted(&picked_arms())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "transport").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in picked_transport().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let stamps = |r: &crate::recgame::RecGame| -> Vec<(i32, i32, u32)> {
        r.packages
            .iter()
            .map(|p| (p.frame, p.play, p.stamp))
            .collect()
    };
    let (ours, old) = (stamps(&back), stamps(&before));
    eprintln!(
        "TRANSPORT tails {:?} against {:?}",
        &ours[ours.len() - 3..],
        &old[old.len() - 2..]
    );
    let (n, m) = (ours.len(), old.len());
    assert_eq!((n, ours[..m - 1].to_vec()), (m + 1, old[..m - 1].to_vec()));
    assert_eq!((ours[n - 2].0, ours[n - 2].1), (TRANSPORT_FRAME, 0));
    assert_eq!(
        (ours[n - 1].0, ours[n - 1].2),
        (old[m - 1].0, old[m - 1].2 + 1)
    );
    let gz = crate::recgame::write_gz(&rec);
    let old = crate::recgame::write_gz(&before);
    eprintln!(
        "TRANSPORT RCX {} bytes (the arms' file {})",
        gz.len(),
        old.len()
    );
    assert_eq!(old.len(), ARMS_RCX_BYTES, "the file run1663 played");
    assert_eq!(gz.len(), TRANSPORT_RCX_BYTES, "TRANSPORT_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The transport file's gzipped length (`docs/CLIENT_INPUT.md` §32.14).
const TRANSPORT_RCX_BYTES: usize = 59_825;

/// The frames the press's effect is read on, both sides: the package's
/// frame, the two records after its tick, and two across the window.
const TRANSPORT_READ_AT: [i64; 5] = [6910, 6911, 6912, 7000, 7499];

/// The pressed Citizen on one frame: `unit_masks & 0x800000` (ours
/// `Unit::auto_transport`), its position and its head order.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TransportEffect {
    pub frame: i64,
    pub auto_transport: bool,
    pub pos: (i64, i64),
    pub head: Option<i64>,
}

/// Our walk under the pump on `orders`' file, read on `at`, and slot 0's
/// transport level on each.
fn transport_effects_ours(orders: &[Order], at: &[i64]) -> Vec<TransportEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in at {
        tick_to(&mut built, Some(&mut stream), n);
        let s = &built.sim;
        let u = &s.units[s
            .unit_by_o(0, crate::script::TRANSPORT_UNIT)
            .expect("the Citizen")];
        ours.push(TransportEffect {
            frame: s.frame,
            auto_transport: u.auto_transport,
            pos: (i64::from(u.pos.x), i64::from(u.pos.y)),
            head: u.orders.front().map(|o| i64::from(o.index())),
        });
    }
    ours
}

/// The original's, off the take's records on `at`.
fn transport_effects_theirs(take: &str, at: &[i64]) -> Option<Vec<TransportEffect>> {
    let mut source = crate::capture::indexed::IndexedCapture::open(dump(take)?).unwrap();
    let mut theirs = Vec::new();
    for &n in at {
        let Some(i) = source.frames().iter().position(|f| f.number == n) else {
            continue;
        };
        let f = source.frame_state(i).unwrap();
        let o = i64::from(crate::script::TRANSPORT_UNIT);
        let u = f
            .units
            .iter()
            .find(|u| u.who == 0 && u.o == o)
            .expect("the Citizen");
        theirs.push(TransportEffect {
            frame: n,
            auto_transport: u.unit_masks.expect("unit_masks") & 0x80_0000 != 0,
            pos: (u.pos.x, u.pos.y),
            head: u.current_order().map(|o| o.index),
        });
    }
    Some(theirs)
}

/// **What the press does on our walk** (item 4375, `docs/CLIENT_INPUT.md`
/// §32.14), pinned off our walk's print before run1668's records were
/// read on these frames.
fn transport_effects() -> Vec<TransportEffect> {
    let row = |frame, pos, head| TransportEffect {
        frame,
        auto_transport: false,
        pos,
        head,
    };
    vec![
        row(6910, (2991, 32508), Some(2)),
        row(6911, (3003, 32530), Some(2)),
        row(6912, (3015, 32552), Some(2)),
        row(7000, (3306, 33036), None),
        row(7499, (4248, 28296), Some(7)),
    ]
}

/// **The press on our walk and on the original's** (item 4375): the
/// pressed Citizen on [`TRANSPORT_READ_AT`], ours pinned and equal to the
/// arms' file's walk (slot 0's level is 0, so `action_set_transport`
/// forces the flag to 0 and the bit stays clear); and run1668's records,
/// equal to ours.
#[test]
fn what_the_transport_press_does_on_both_sides() {
    if built_start().is_none() || minted(&crate::script::picked_transport()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = transport_effects_ours(&crate::script::picked_transport(), &TRANSPORT_READ_AT);
    let without = transport_effects_ours(&crate::script::picked_arms(), &TRANSPORT_READ_AT);
    for (a, b) in ours.iter().zip(&without) {
        eprintln!("TRANSPORT EFFECT ours {a:?}\n                 without {b:?}");
    }
    assert_eq!(ours, transport_effects(), "ours, pinned");
    assert_eq!(ours, without, "level 0: the press changes nothing");
    let Some(theirs) = transport_effects_theirs(TRANSPORT_WALKED[0].0, &TRANSPORT_READ_AT) else {
        return;
    };
    for t in &theirs {
        eprintln!("TRANSPORT EFFECT theirs {t:?}");
    }
    assert_eq!(theirs, ours, "the original's Citizen is ours");
}

/// **The original's playback of the transport file** (item 4375,
/// `docs/CLIENT_INPUT.md` §32.14): `(take, trace, first sequence parting
/// or the last frame, last frame, records compared, records parted)`.
/// Pinned before run1668 was read, with none parted.
pub(crate) const TRANSPORT_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1668-greatlakes-playback-of-transport-7500.txt",
    "rontrace-run1668.log",
    7500,
    7500,
    700,
    0,
)];

/// The functions the press's package enters on the original's side:
/// `process_set_transport@00948f60`, `Group::action_set_transport@007024b0`
/// and the member walk's `UnitData::can_ever_transport@0046f290`.
const TRANSPORT_FUNCTIONS: [(u32, &str); 3] = [
    (0x0094_8f60, "CommandPackage::process_set_transport"),
    (0x0070_24b0, "Group::action_set_transport"),
    (0x0046_f290, "UnitData::can_ever_transport"),
];

/// Each take's first entry into [`TRANSPORT_FUNCTIONS`], in sim-frames:
/// none on run1663, the arms' file's playback, and the tick that takes the
/// 6910 package on run1668 (`process_guard` reads 6869 for 6870's on both).
const TRANSPORT_ENTRIES: &[(&str, [Option<i64>; 3])] = &[
    ("rontrace-run1663.log", [None; 3]),
    ("rontrace-run1668.log", [Some(6909); 3]),
];

/// **The original plays the transport file in lockstep** (item 4375): the
/// built start walked under the pump on the transport file against the
/// take, every frame's draws and every record of its window
/// ([`TRANSPORT_WALKED`]).
#[test]
fn the_original_s_playback_of_the_transport_file_walks_in_lockstep() {
    let orders = crate::script::picked_transport();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in TRANSPORT_WALKED {
        let Some((w, _)) = walk_minted(&orders, take, take_trace) else {
            return;
        };
        eprintln!(
            "TRANSPORT WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        if let Some((o, t)) = &w.sites
            && let Some((_, d)) = first_parting(o, t)
        {
            eprintln!("the first parting frame, {:?}: {d}", w.first_sequence);
        }
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let mut entries = Vec::new();
    for &(take_trace, _) in TRANSPORT_ENTRIES {
        let Some(tr) = trace(take_trace) else {
            return;
        };
        let first = TRANSPORT_FUNCTIONS.map(|(va, _)| tr.first_entry(va));
        for ((_, name), f) in TRANSPORT_FUNCTIONS.iter().zip(&first) {
            eprintln!("TRANSPORT ENTRY {take_trace} {name}: {f:?}");
        }
        entries.push((take_trace, first));
    }
    assert_eq!(walked, TRANSPORT_WALKED, "the walked table");
    assert_eq!(entries, TRANSPORT_ENTRIES, "the press's functions");
}

/// **The repair file's packages round-trip** (item 4383,
/// `docs/CLIENT_INPUT.md` §32.15): the transport script and two packages
/// more, the computer's `attack` on [`crate::script::WOUND_FRAME`] on
/// `play` 1 and the Repair pick's `REPAIR` on
/// [`crate::script::REPAIR_FRAME`]; every package before them is the
/// transport file's, frame, slot and stamp. With `RON_RCX_OUT=<file>` it
/// writes the `.rcx` a playback takes open.
#[test]
fn the_repair_minted_packages_round_trip() {
    use crate::script::{REPAIR_FRAME, WOUND_FRAME, picked_repair, picked_transport};
    let (Some(rec), Some(before)) = (minted(&picked_repair()), minted(&picked_transport())) else {
        eprintln!("skipping: no recording of the lobby on disk");
        return;
    };
    let bytes = crate::recgame::write(&rec);
    let back = crate::recgame::parse(&bytes, "repair").expect("the file parses");
    assert_eq!(
        crate::recgame::write(&back),
        bytes,
        "re-writes byte for byte"
    );
    for ((f, play, cmds), p) in picked_repair().iter().zip(&back.packages[1..]) {
        assert_eq!((p.frame, p.play), (*f, *play));
        assert_eq!(
            &crate::commands::decode(&p.data).unwrap(),
            cmds,
            "frame {f}"
        );
    }
    let stamps = |r: &crate::recgame::RecGame| -> Vec<(i32, i32, u32)> {
        r.packages
            .iter()
            .map(|p| (p.frame, p.play, p.stamp))
            .collect()
    };
    let (ours, old) = (stamps(&back), stamps(&before));
    eprintln!(
        "REPAIR tails {:?} against {:?}",
        &ours[ours.len() - 4..],
        &old[old.len() - 2..]
    );
    let (n, m) = (ours.len(), old.len());
    assert_eq!((n, ours[..m - 1].to_vec()), (m + 2, old[..m - 1].to_vec()));
    assert_eq!((ours[n - 3].0, ours[n - 3].1), (WOUND_FRAME, 1));
    assert_eq!((ours[n - 2].0, ours[n - 2].1), (REPAIR_FRAME, 0));
    let gz = crate::recgame::write_gz(&rec);
    let old = crate::recgame::write_gz(&before);
    eprintln!(
        "REPAIR RCX {} bytes (the transport file {})",
        gz.len(),
        old.len()
    );
    assert_eq!(old.len(), TRANSPORT_RCX_BYTES, "the file run1668 played");
    assert_eq!(gz.len(), REPAIR_RCX_BYTES, "REPAIR_RCX_BYTES");
    if let Some(out) = std::env::var_os("RON_RCX_OUT") {
        std::fs::write(&out, &gz).unwrap();
        eprintln!("wrote {}", out.to_string_lossy());
    }
}

/// The repair file's gzipped length (`docs/CLIENT_INPUT.md` §32.15).
const REPAIR_RCX_BYTES: usize = 59_862;

/// The frames the two packages' effects are read on, both sides.
const REPAIR_READ_AT: [i64; 16] = [
    6920, 6921, 6922, 7500, 8180, 8190, 8200, 8201, 8202, 8250, 8300, 8325, 8350, 8375, 8400, 8500,
];

/// One frame of the two packages' effects: the Capital's `damage`
/// (`hits − health` on ours), the repairing Citizen's position and head
/// order (`None` before it is trained), and each Bowman's, `None` once
/// it is gone.
/// A unit on one frame: its position and its head order.
type Seen = Option<((i64, i64), Option<i64>)>;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RepairEffect {
    pub frame: i64,
    pub damage: i64,
    pub repairer: Seen,
    pub wounders: [Seen; 3],
}

/// Our walk under the pump on `orders`' file, read on `at`.
fn repair_effects_ours(orders: &[Order], at: &[i64]) -> Vec<RepairEffect> {
    use crate::script::{REPAIR_UNIT, WOUNDED, WOUNDERS};
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in at {
        tick_to(&mut built, Some(&mut stream), n);
        let s = &built.sim;
        let b = &s.buildings[s.building_by_o(0, WOUNDED).expect("the Capital")];
        let unit = |who, o| {
            s.unit_by_o(who, o).map(|k| {
                let u = &s.units[k];
                (
                    (i64::from(u.pos.x), i64::from(u.pos.y)),
                    u.orders.front().map(|o| i64::from(o.index())),
                )
            })
        };
        ours.push(RepairEffect {
            frame: s.frame,
            damage: i64::from(b.hits - b.health),
            repairer: unit(0, REPAIR_UNIT),
            wounders: WOUNDERS.map(|o| unit(1, o)),
        });
    }
    ours
}

/// The original's, off the take's records on `at`.
fn repair_effects_theirs(take: &str, at: &[i64]) -> Option<Vec<RepairEffect>> {
    use crate::script::{REPAIR_UNIT, WOUNDED, WOUNDERS};
    let mut source = crate::capture::indexed::IndexedCapture::open(dump(take)?).unwrap();
    let mut theirs = Vec::new();
    for &n in at {
        let Some(i) = source.frames().iter().position(|f| f.number == n) else {
            continue;
        };
        let f = source.frame_state(i).unwrap();
        let b = f
            .builds
            .iter()
            .find(|b| b.who == 0 && b.o == i64::from(WOUNDED))
            .expect("the Capital");
        let unit = |who: i64, o: i16| {
            f.units
                .iter()
                .find(|u| u.who == who && u.o == i64::from(o))
                .map(|u| ((u.pos.x, u.pos.y), u.current_order().map(|o| o.index)))
        };
        theirs.push(RepairEffect {
            frame: n,
            damage: b.damage.expect("damage"),
            repairer: unit(0, REPAIR_UNIT),
            wounders: WOUNDERS.map(|o| unit(1, o)),
        });
    }
    Some(theirs)
}

/// **What the two packages do on our walk** (item 4383,
/// `docs/CLIENT_INPUT.md` §32.15), pinned off our walk's print before
/// run1679's records were read on these frames.
fn repair_effects() -> Vec<RepairEffect> {
    let u = |x, y, head| Some(((x, y), head));
    let row = |frame, damage, repairer: Seen, wounders: [Seen; 3]| RepairEffect {
        frame,
        damage,
        repairer,
        wounders,
    };
    let (bowmen, citizen) = ([None; 3], u(2232, 31224, Some(7)));
    // Item 4391: the squad walks plain from 7163, and the attack reaches
    // the Capital on 8200 with `1/24` alone, as theirs does. Item 4395: the
    // forming on 7162 claims the record, so the squad stays the army's;
    // the repairer waits at (2040, 31032), not (1848, 31608), and `1/26`
    // lives to 8190, as theirs.
    let citizen_late = u(2040, 31032, Some(7));
    let at = u(2472, 30936, Some(13));
    let done = u(2472, 30936, None);
    let a = u(6216, 29976, Some(10));
    let c = u(6168, 29592, Some(10));
    vec![
        row(
            6920,
            0,
            None,
            [
                u(30919, 21067, None),
                u(30888, 21192, None),
                u(30936, 20952, None),
            ],
        ),
        row(
            6921,
            0,
            None,
            [
                u(30936, 21048, Some(1)),
                u(30888, 21192, Some(1)),
                u(30936, 20952, Some(1)),
            ],
        ),
        row(
            6922,
            0,
            None,
            [
                u(30964, 21052, Some(1)),
                u(30911, 21176, Some(1)),
                u(30965, 20952, Some(1)),
            ],
        ),
        row(
            7500,
            0,
            citizen,
            [
                u(22740, 27052, Some(1)),
                u(22529, 26686, Some(1)),
                u(22882, 26635, Some(1)),
            ],
        ),
        row(8180, 0, citizen_late, [a, None, c]),
        row(8190, 0, citizen_late, [a, None, c]),
        row(8200, 1, citizen_late, [a, None, None]),
        row(8201, 1, u(2064, 31027, Some(1)), [a, None, None]),
        row(8202, 1, u(2088, 31022, Some(1)), [a, None, None]),
        row(8250, 2, at, bowmen),
        row(8300, 0, done, bowmen),
        row(8325, 0, done, bowmen),
        row(8350, 0, done, bowmen),
        row(8375, 0, done, bowmen),
        row(8400, 0, done, bowmen),
        row(8500, 0, done, bowmen),
    ]
}

/// **The computer's attack and the Repair pick's `REPAIR` on our walk and
/// on the original's** (item 4383): on [`REPAIR_READ_AT`], ours pinned,
/// against the transport file's walk (no wound, no repair); and run1679's
/// records, equal to ours on every frame it prints (item 4395).
#[test]
fn what_the_repair_does_on_both_sides() {
    if built_start().is_none() || minted(&crate::script::picked_repair()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = repair_effects_ours(&crate::script::picked_repair(), &REPAIR_READ_AT);
    let without = repair_effects_ours(&crate::script::picked_transport(), &REPAIR_READ_AT);
    for (a, b) in ours.iter().zip(&without) {
        eprintln!("REPAIR EFFECT ours {a:?}\n              without {b:?}");
    }
    assert_eq!(ours, repair_effects(), "ours, pinned");
    assert!(
        without.iter().all(|e| e.damage == 0),
        "the transport file wounds nothing"
    );
    let Some(theirs) = repair_effects_theirs(REPAIR_WALKED[0].0, &REPAIR_READ_AT) else {
        return;
    };
    for t in &theirs {
        eprintln!("REPAIR EFFECT theirs {t:?}");
    }
    // The walk is whole to 8500 (item 4395): ours equals theirs on every
    // frame the take prints; 8500 is its closing block.
    assert_eq!(
        theirs.len(),
        REPAIR_READ_AT.len() - 1,
        "8500 is the closing block"
    );
    assert_eq!(theirs[..], ours[..theirs.len()], "every read, both sides");
    let first = |v: &[RepairEffect]| v.iter().find(|e| e.damage > 0).map(|e| e.frame);
    let repairing = |v: &[RepairEffect]| {
        v.iter()
            .find(|e| e.repairer.and_then(|r| r.1) == Some(i64::from(sim::ids::order::REPAIR)))
            .map(|e| e.frame)
    };
    let whole = |v: &[RepairEffect]| {
        v.iter()
            .rev()
            .take_while(|e| e.damage == 0)
            .last()
            .map(|e| e.frame)
    };
    let read = |v: &[RepairEffect]| (first(v), repairing(v), whole(v));
    assert_eq!(read(&ours), (Some(8200), Some(8250), Some(8300)), "ours");
    assert_eq!(
        read(&theirs),
        (Some(8200), Some(8250), Some(8300)),
        "the original wounds the Capital, repairs it, and it is whole"
    );
}

/// **The original's playback of the repair file** (item 4383,
/// `docs/CLIENT_INPUT.md` §32.15): `(take, trace, first sequence parting
/// or the last frame, last frame, records compared, records parted)`.
/// Pinned before run1679 was read at 8500 and none parted; it read parted,
/// first on 7163 (slot 1's new Longbowman `1/30` heads `GROUP_ATTACK_TO`
/// on ours, `ATTACK_TO` on theirs; parked 4383a), the draws on 7274, and
/// is pinned as measured. Item 4391 moved it (7274, 1337) → (7420, 1081):
/// the squad answers no army and walks plain on 7163 as theirs does
/// ([`a_pushed_group_s_member_answers_no_army_on_both_sides`]); the first
/// record parting was then 7419 (`1/32`'s position, `1/31`'s order count).
/// Item 4395 moved it (7420, 1081) → (8500, 0), whole: the army's forming
/// on 7162 claims the adopted record, so the squad's mark is down by 7418
/// and the tick's move there groups it as theirs
/// ([`the_army_s_forming_claims_the_adopted_record_on_both_sides`]).
pub(crate) const REPAIR_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1679-greatlakes-playback-of-repair-8500.txt",
    "rontrace-run1679.log",
    8500,
    8500,
    1600,
    0,
)];

/// **The original plays the repair file in lockstep** (item 4383): the
/// built start walked under the pump on the repair file against the take,
/// every frame's draws and every record of its window ([`REPAIR_WALKED`]).
#[test]
fn the_original_s_playback_of_the_repair_file_walks_in_lockstep() {
    let orders = crate::script::picked_repair();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in REPAIR_WALKED {
        let Some((w, _)) = walk_minted(&orders, take, take_trace) else {
            return;
        };
        eprintln!(
            "REPAIR WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        if let Some((o, t)) = &w.sites
            && let Some((_, d)) = first_parting(o, t)
        {
            eprintln!("the first parting frame, {:?}: {d}", w.first_sequence);
        }
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    assert_eq!(walked, REPAIR_WALKED, "the walked table");
}

/// **A pushed group's member answers no army** (item 4391, `docs/ORDERS.md`
/// §8.2, §14; `docs/GOLDEN.md` §77): slot 1's Longbowmen `1/30` (captain),
/// `1/31` and `1/32`, born 6993 in group 65, army 1's index, emptied of its
/// Bowmen by the `play` 1 package of 6920. `Object::get_army@00649d70`
/// answers −1 for them (the record's `army` is −1), so `add_to_army`
/// re-runs on every 32-frame edge and `go_to_unit` re-marks the squad
/// (`unit_masks & 4`): on block 7162, the army's tick, the mark stands on
/// all three, on both sides, and ours answers no army. The gate then sends
/// each a plain `ATTACK_TO` on 7163, where ours sent `GROUP_ATTACK_TO`.
#[test]
fn a_pushed_group_s_member_answers_no_army_on_both_sides() {
    const SQUAD: [i16; 3] = [30, 31, 32];
    const AT: i64 = 7162;
    if built_start().is_none() || minted(&crate::script::picked_repair()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(&crate::script::picked_repair()).unwrap());
    tick_to(&mut built, Some(&mut stream), AT);
    let s = &built.sim;
    let ours = SQUAD.map(|o| {
        let k = s.unit_by_o(1, o).expect("the squad");
        (s.units[k].in_danger, s.army_of(k))
    });
    eprintln!("SQUAD ours on {AT}: {ours:?}");
    assert_eq!(ours, [(true, None); 3], "ours: marked, in no army");
    tick_to(&mut built, Some(&mut stream), AT + 1);
    let s = &built.sim;
    let heads = SQUAD.map(|o| {
        let k = s.unit_by_o(1, o).expect("the squad");
        s.units[k].orders.front().map(|o| i64::from(o.index()))
    });
    let attack_to = Some(i64::from(sim::ids::order::ATTACK_TO));
    assert_eq!(heads, [attack_to; 3], "ours on 7163: a plain ATTACK_TO");
    let Some(path) = dump(REPAIR_WALKED[0].0) else {
        return;
    };
    let mut source = crate::capture::indexed::IndexedCapture::open(path).unwrap();
    let i = source
        .frames()
        .iter()
        .position(|f| f.number == AT)
        .expect("block 7162");
    let f = source.frame_state(i).unwrap();
    let theirs = SQUAD.map(|o| {
        let u = f
            .units
            .iter()
            .find(|u| u.who == 1 && u.o == i64::from(o))
            .expect("the squad");
        u.unit_masks.expect("unit_masks") & 4 != 0
    });
    assert_eq!(theirs, [true; 3], "theirs: marked");
    let i = source
        .frames()
        .iter()
        .position(|f| f.number == AT + 1)
        .expect("block 7163");
    let f = source.frame_state(i).unwrap();
    let heads = SQUAD.map(|o| {
        f.units
            .iter()
            .find(|u| u.who == 1 && u.o == i64::from(o))
            .and_then(|u| u.current_order().map(|o| o.index))
    });
    assert_eq!(heads, [attack_to; 3], "theirs on 7163: a plain ATTACK_TO");
}

/// **The army's forming claims the adopted record** (item 4395,
/// `docs/ORDERS.md` §8.2, §14; `docs/GOLDEN.md` §77): the army's tick on
/// 7162 runs `Army::do_forming@006f43c0`, which writes the army's id into
/// each populated group record it walks (`:39`, record `+8`), so group
/// 65's `army` is no longer −1 and `Object::get_army` answers army 1 for
/// the squad. `add_to_army` stops re-running, nothing re-marks, and each
/// member's own 32-frame edge in `Unit::work` clears `unit_masks & 4`
/// (theirs 7170 and 7171). On block 7200 the mark is down on all three, on
/// both sides, and ours answers army 1.
#[test]
fn the_army_s_forming_claims_the_adopted_record_on_both_sides() {
    const SQUAD: [i16; 3] = [30, 31, 32];
    const AT: i64 = 7200;
    if built_start().is_none() || minted(&crate::script::picked_repair()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(&crate::script::picked_repair()).unwrap());
    tick_to(&mut built, Some(&mut stream), AT);
    let s = &built.sim;
    let ours = SQUAD.map(|o| {
        let k = s.unit_by_o(1, o).expect("the squad");
        (s.units[k].in_danger, s.army_of(k))
    });
    eprintln!("SQUAD ours on {AT}: {ours:?}");
    assert_eq!(ours, [(false, Some(1)); 3], "ours: unmarked, in army 1");
    let Some(path) = dump(REPAIR_WALKED[0].0) else {
        return;
    };
    let mut source = crate::capture::indexed::IndexedCapture::open(path).unwrap();
    let i = source
        .frames()
        .iter()
        .position(|f| f.number == AT)
        .expect("block 7200");
    let f = source.frame_state(i).unwrap();
    let theirs = SQUAD.map(|o| {
        let u = f
            .units
            .iter()
            .find(|u| u.who == 1 && u.o == i64::from(o))
            .expect("the squad");
        u.unit_masks.expect("unit_masks") & 4 != 0
    });
    assert_eq!(theirs, [false; 3], "theirs: unmarked");
}

/// **The original's playback of the auto-explored file** (item 4291,
/// `docs/CLIENT_INPUT.md` §31.5): `(take, trace, first sequence parting or
/// the last frame, last frame, records compared, records parted)`. Pinned
/// before run1627 was read, with none parted.
pub(crate) const AUTO_EXPLORED_WALKED: &[(&str, &str, i64, i64, usize, usize)] = &[(
    "gamelog-run1627-greatlakes-playback-of-auto-explored-7500.txt",
    "rontrace-run1627.log",
    7500,
    7500,
    700,
    0,
)];

/// The frames the auto-explore's effect is read on, both sides: the tick
/// that takes the package, the two after it, and three across the window.
const AUTO_EXPLORE_READ_AT: [i64; 6] = [6819, 6820, 6821, 6900, 7200, 7499];

/// The Scout `0/0` on one frame: where it stands, its head order's point,
/// and `unit_masks & 0x100`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AutoExploreEffect {
    pub frame: i64,
    pub pos: (i64, i64),
    pub to: Option<(i64, i64)>,
    pub exploring: bool,
}

/// Our walk under the pump on `orders`' file, read on `at`.
fn auto_explore_effects_ours(orders: &[Order], at: &[i64]) -> Vec<AutoExploreEffect> {
    let (_, mut built, _) = built_start().unwrap();
    let mut stream = crate::input::Stream::pump(&minted(orders).unwrap());
    let mut ours = Vec::new();
    for &n in at {
        tick_to(&mut built, Some(&mut stream), n);
        let s = &built.sim;
        let u = &s.units[s
            .unit_by_o(0, crate::script::SCOUT_OBJECT)
            .expect("the Scout")];
        ours.push(AutoExploreEffect {
            frame: s.frame,
            pos: (i64::from(u.pos.x), i64::from(u.pos.y)),
            to: u.orders.front().and_then(|o| match &o.body {
                sim::orders::Body::Move(m) => Some((i64::from(m.dest.x), i64::from(m.dest.y))),
                _ => None,
            }),
            exploring: u.auto_explore,
        });
    }
    ours
}

/// The original's, off the take's records on `at`.
fn auto_explore_effects_theirs(take: &str, at: &[i64]) -> Option<Vec<AutoExploreEffect>> {
    let mut source = crate::capture::indexed::IndexedCapture::open(dump(take)?).unwrap();
    let mut theirs = Vec::new();
    for &n in at {
        let Some(i) = source.frames().iter().position(|f| f.number == n) else {
            continue;
        };
        let f = source.frame_state(i).unwrap();
        let o = i64::from(crate::script::SCOUT_OBJECT);
        let u = f
            .units
            .iter()
            .find(|u| u.who == 0 && u.o == o)
            .expect("the Scout");
        theirs.push(AutoExploreEffect {
            frame: n,
            pos: (u.pos.x, u.pos.y),
            to: u.current_order().and_then(|o| Some((o.x?, o.y?))),
            exploring: u.unit_masks.expect("unit_masks") & 0x100 != 0,
        });
    }
    Some(theirs)
}

/// **What the auto-explore does on our walk** (item 4291,
/// `docs/CLIENT_INPUT.md` §31.5), pinned off our walk's print before
/// run1627's records were read on these frames.
fn auto_explored_effects() -> Vec<AutoExploreEffect> {
    let row = |frame, pos, to, exploring| AutoExploreEffect {
        frame,
        pos,
        to,
        exploring,
    };
    vec![
        row(6819, (4405, 31970), Some((4392, 32040)), false),
        row(6820, (4399, 32004), Some((4392, 32040)), false),
        row(6821, (4399, 32004), Some((5880, 27384)), true),
        row(6900, (6256, 30085), Some((5880, 27384)), true),
        row(7200, (1450, 26481), Some((504, 27384)), true),
        row(7499, (3552, 34526), Some((3576, 35064)), true),
    ]
}

/// **The auto-explore on our walk and on the original's** (item 4291):
/// the Scout's path and its bit on [`AUTO_EXPLORE_READ_AT`], ours pinned,
/// against the disbanded file's walk, which keeps the Scout's own path
/// and never sets the bit; and run1627's records, equal to ours.
#[test]
fn what_the_auto_explore_does_on_both_sides() {
    if built_start().is_none() || minted(&crate::script::auto_explored()).is_none() {
        eprintln!("skipping: no install, no lobby or no recording");
        return;
    }
    let ours = auto_explore_effects_ours(&crate::script::auto_explored(), &AUTO_EXPLORE_READ_AT);
    let without = auto_explore_effects_ours(&crate::script::disbanded(), &AUTO_EXPLORE_READ_AT);
    for (a, b) in ours.iter().zip(&without) {
        eprintln!("AUTO_EXPLORE EFFECT ours {a:?}\n                    without {b:?}");
    }
    assert_eq!(ours, auto_explored_effects(), "ours, pinned");
    assert!(
        without.iter().all(|e| !e.exploring),
        "no bit without the order"
    );
    assert_ne!(
        ours[1..],
        without[1..],
        "the Scout's path parts after the package"
    );
    let Some(theirs) =
        auto_explore_effects_theirs(AUTO_EXPLORED_WALKED[0].0, &AUTO_EXPLORE_READ_AT)
    else {
        return;
    };
    for t in &theirs {
        eprintln!("AUTO_EXPLORE EFFECT theirs {t:?}");
    }
    assert_eq!(theirs, ours, "the original's Scout is ours");
}

/// The unitmask's handler and the group's action on the original's side,
/// by entry address (`docs/CLIENT_INPUT.md` §31.1). `Unit::think_scout`
/// is no witness: slot 1's scout enters it on frame 0 of both takes.
const UNITMASK_FUNCTIONS: [(u32, &str); 2] = [
    (0x0094_7790, "CommandPackage::process_unitmask"),
    (0x006f_cb90, "Group::action_unitmask"),
];

/// Each take's first entry into [`UNITMASK_FUNCTIONS`], in sim-frames,
/// pinned before run1627 was read: run1623 never; run1627 on 6819, the
/// tick that consumes [`crate::script::AUTO_EXPLORE_FRAME`]'s package.
const UNITMASK_ENTRIES: &[(&str, [Option<i64>; 2])] = &[
    ("rontrace-run1623.log", [None, None]),
    ("rontrace-run1627.log", [Some(6819), Some(6819)]),
];

/// **The original plays the auto-explored file in lockstep** (item 4291,
/// `docs/CLIENT_INPUT.md` §31.5): the built start walked under the pump
/// on the auto-explored file against the take, every frame's draws and
/// every record of its window ([`AUTO_EXPLORED_WALKED`]); and the
/// handler's entry on the original's side ([`UNITMASK_ENTRIES`]).
#[test]
fn the_original_s_playback_of_the_auto_explored_file_walks_in_lockstep() {
    let auto_explored = crate::script::auto_explored();
    let mut walked = Vec::new();
    for &(take, take_trace, ..) in AUTO_EXPLORED_WALKED {
        let Some((w, _)) = walk_minted(&auto_explored, take, take_trace) else {
            return;
        };
        eprintln!(
            "AUTO_EXPLORED WALK {take}: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        if let Some((o, t)) = &w.sites
            && let Some((_, d)) = first_parting(o, t)
        {
            eprintln!("the first parting frame, {:?}: {d}", w.first_sequence);
        }
        walked.push((
            take,
            take_trace,
            w.first_sequence.unwrap_or(w.last),
            w.last,
            w.records,
            w.records_parted,
        ));
    }
    let mut entries = Vec::new();
    for &(take_trace, _) in UNITMASK_ENTRIES {
        let tr = trace(take_trace).expect("the take's trace");
        let first = UNITMASK_FUNCTIONS.map(|(va, _)| tr.first_entry(va));
        eprintln!("UNITMASK ENTRY {take_trace}: {first:?}");
        entries.push((take_trace, first));
    }
    assert_eq!(entries, UNITMASK_ENTRIES, "the handler's entry");
    assert_eq!(walked, AUTO_EXPLORED_WALKED, "the walked table");
}
