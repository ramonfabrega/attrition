//! **A citizen builds in one live session** (item 4695,
//! `docs/CLIENT_INPUT.md` §39): Ramon's recording of 2026-11-25 walked,
//! and the arm he could not reach, pressed as the window presses it.
use crate::input::tests::{camera, install_roster, lobby};
use crate::input::{Event, Input, View};
use rondata::free::FreeRunner;
use sim::Pos;
use sim::observe::ObjId;

/// Ramon's recording, outside the repo (`docs/QUEUE.md` item 4695).
const SPIN_1125: &str = "ron-data/lab-experiments/2026-11-25-pass-second/spin-1125.rcx";

/// **`SPIN_1125_WALKED`**: each build order of the recording, its frame
/// and type, and what our walk does with it on its frame, the selection
/// being object 0 (the Scout) throughout: the first Farm placed (the
/// site's building record, 3) and never started; the Barracks
/// unavailable in the first age (`type_avail` 0) and its site under a
/// building; the second Farm outside the city's radius; the second
/// Barracks unavailable and on forest. Seen red pinned with the first
/// Farm's verdict perturbed.
const SPIN_1125_WALKED: [(i64, usize, &str); 4] = [
    (1637, 417, "placed 3"),
    (2180, 427, "avail 0, Building"),
    (2714, 417, "avail 4, OutsideRadius"),
    (6790, 427, "avail 0, Forest"),
];

fn verdict(sim: &sim::Sim, at: (i32, i32), ty: usize) -> String {
    let rec = ty - rondata::load::BASE_BUILDTYPES as usize;
    let avail = sim.tables.build_types[rec]
        .tree
        .map_or(-1, |t| sim.type_avail(0, t));
    let at = sim.snap_center_placed(rec, Pos::new(at.0, at.1), Some(0));
    let blocked = sim.blocked_site(Some(0), rec, at, None);
    if avail == sim::tech::AVAILABLE && blocked == sim::cities::place::Blocked::Clear {
        return format!("placed {rec}");
    }
    format!("avail {avail}, {blocked:?}")
}

#[test]
fn the_recording_s_builds_walked() {
    let path = std::env::var_os("HOME")
        .map(|h| std::path::PathBuf::from(h).join(SPIN_1125))
        .filter(|p| p.is_file());
    let (Some(path), Ok(r)) = (path, lobby()) else {
        println!("skipping: no recording or no lobby");
        return;
    };
    let rec = rondata::recgame::read(&path.to_string_lossy()).unwrap();
    let mut r = r.with_stream(rondata::input::Stream::pump(&rec));
    // Object 0, every group's (40 of 42 repeat it), is the Scout.
    let scout = r
        .scene(Some(0))
        .units
        .into_iter()
        .find(|u| u.id == ObjId { who: 0, o: 0 });
    assert_eq!(
        scout.map(|u| u.type_index),
        Some(69),
        "object 0 is the Scout"
    );
    let mut got = Vec::new();
    for p in &rec.packages {
        for c in rondata::commands::decode(&p.data).unwrap() {
            if let rondata::commands::Command::Build { x, y, build, .. } = c {
                while r.frame() < i64::from(p.frame) {
                    r.step();
                }
                let v = verdict(r.sim(), (x, y), build as usize);
                got.push((i64::from(p.frame), build as usize, v));
            }
        }
    }
    let want: Vec<_> = SPIN_1125_WALKED
        .iter()
        .map(|&(f, t, v)| (f, t, v.to_string()))
        .collect();
    assert_eq!(got, want);
    while r.frame() < 7460 {
        r.step();
    }
    // The one site stands at the game's end, a Farm no one has started.
    let sites: Vec<_> = r
        .scene(Some(0))
        .buildings
        .into_iter()
        .filter(|b| b.owner == 0 && !b.active)
        .map(|b| (b.id.o, b.ty, b.pos, b.started))
        .collect();
    assert_eq!(sites, [(2007, Some(3), Pos::new(5376, 31488), false)]);
}

/// The view the window draws with, over the walk, the camera looking at
/// `at`.
fn with_view<T>(r: &FreeRunner, at: Pos, f: impl FnOnce(&View) -> T) -> T {
    let (scene, terrain) = (r.scene(Some(0)), r.terrain());
    let z = terrain_z(&terrain, at);
    let c = camera(Some((at, z)));
    let v = View {
        matrix: c.to_screen_matrix(),
        screen: crate::pinned_screen(),
        size: Some((1920, 1080)),
        scene: &scene,
        terrain: &terrain,
        sim: Some(r.sim()),
    };
    f(&v)
}

fn terrain_z(t: &sim::observe::TerrainScene, at: Pos) -> i32 {
    let w = t.width * 4;
    let (tx, ty) = (at.x / 192, at.y / 192);
    t.tile_z
        .get((ty * w + tx) as usize)
        .copied()
        .unwrap_or(0)
        .max(0)
}

/// **`PLACED_LIVE`**: the frames after the press when the site stands
/// placed and when the citizen has started it. Seen red with the gate
/// removed (the Scout armed the Farm).
const PLACED_LIVE: (i64, i64) = (1, 51);

/// **The score's row** (item 4695): in one session on the harness lobby,
/// the slot's citizen selected by its press, `F` arms the Farm, the ghost
/// stands clear under the cursor where the build will place it, the press
/// mints the `group` and the `build`, and the walk places the site there
/// and the citizen starts it. Object 0, the Scout, and the first age's
/// greyed Barracks arm nothing (Ramon's play, `SPIN_1125_WALKED`).
#[test]
fn a_citizen_builds_in_one_live_session() {
    let (Ok(mut r), Some(roster)) = (lobby(), install_roster()) else {
        println!("skipping: no lobby");
        return;
    };
    let mut input = Input {
        roster: std::sync::Arc::new(roster),
        ..Input::new(0)
    };
    let citizen = ObjId { who: 0, o: 1 };
    let start = r
        .scene(Some(0))
        .units
        .iter()
        .find(|u| u.id == citizen)
        .unwrap()
        .pos;
    // A clear Farm site the city's radius holds: the first on a walk east.
    let site = (1..40)
        .map(|k| Pos::new(start.x + 96 * k, start.y + 1536))
        .find(|p| verdict(r.sim(), (p.x, p.y), 417) == "placed 3")
        .expect("a clear site");
    let frame = r.frame() as i32;
    let order = with_view(&r, site, |v| {
        // The Scout: nothing arms, by key or by button.
        input.selection = vec![ObjId { who: 0, o: 0 }];
        assert_eq!(input.event(frame, Event::Key('f'), v), None);
        assert_eq!(input.event(frame, Event::Place(417), v), None);
        assert_eq!(input.placing, None, "a Scout arms no build");
        // The citizen, by its press; the Barracks is greyed in the first age.
        let (cx, cy) = v.pixel(start.x, start.y, terrain_z(v.terrain, start));
        input.event(frame, Event::Press { x: cx, y: cy }, v);
        assert_eq!(input.selection, [citizen], "the press selects the citizen");
        assert_eq!(input.event(frame, Event::Key('b'), v), None);
        assert_eq!(input.placing, None, "the greyed Barracks arms nothing");
        assert_eq!(input.event(frame, Event::Key('f'), v), None);
        assert_eq!(input.placing, Some(417), "F arms the Farm");
        // The ghost under the cursor, then the press.
        let (px, py) = v.pixel(site.x, site.y, terrain_z(v.terrain, site));
        input.hover = Some((px, py));
        let ghost = input.ghost(v).expect("the ghost is drawn");
        assert!(ghost.clear, "{ghost:?}");
        assert_eq!(ghost.ty, 417);
        let order = input
            .event(frame, Event::Press { x: px, y: py }, v)
            .expect("a build");
        (order, ghost)
    });
    let ((f, play, cmds), ghost) = order;
    assert!(matches!(cmds[0], rondata::commands::Command::Group { .. }));
    assert!(matches!(
        cmds[1],
        rondata::commands::Command::Build { build: 417, .. }
    ));
    r.issue(&(f, play, cmds));
    let mut placed = None;
    let mut started = None;
    for k in 1..=300 {
        r.step();
        let s = r.scene(Some(0));
        let b = s
            .buildings
            .iter()
            .find(|b| b.owner == 0 && b.orig_ty == Some(3) && !b.active && b.pos == ghost.at);
        if let Some(b) = b {
            placed.get_or_insert(k);
            if b.started {
                started.get_or_insert(k);
            }
        }
    }
    println!("PLACED_LIVE {placed:?} {started:?} at {:?}", ghost.at);
    assert_eq!(
        (placed, started),
        (Some(PLACED_LIVE.0), Some(PLACED_LIVE.1))
    );
}
