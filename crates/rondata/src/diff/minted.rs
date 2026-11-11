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
    SCOUT_OBJECT as SCOUT, SCOUT_TO, clicked, script,
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
pub(crate) const FARM_SITE: i16 = 2007;
pub(crate) const FARM_SITE_AT: Pos = Pos { x: 3456, y: 29760 };
pub(crate) const TRAINED: i16 = 6;

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
/// (`.cities.city_tally`, `docs/CHECKSUM.md` §20).
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
            ".buildings{0/2001}",
            ".buildings{0/2007}",
            ".cities.cities",
            ".cities.city_tally",
            ".cities.farm_order",
            ".economy.ledgers",
            ".movement.pf_start",
            ".orders.last_group",
            ".orders.pushed",
            ".roster.hold_doobers",
            ".roster.marks",
            ".terrain.object_heads",
            ".units{0/1}",
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
