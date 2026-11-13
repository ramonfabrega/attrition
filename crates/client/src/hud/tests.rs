//! The HUD's scores (`docs/HUD.md` §5, item 4197): the clicked file
//! minted from button presses, a button's press kept from the world, and
//! the panel's numbers against the original's dump of the right-pressed
//! script's playback (run1568).

use super::*;
use crate::input::tests::{
    CLICKED_RCX, CLICKED_RCX_BYTES, MINTED_RCX, RIGHT, Scripted, camera, events, lobby, mint_with,
    right_events, view,
};
use rondata::commands::Command;
use rondata::script;
use sim::observe::ObjId;

/// The scripts' window: the size every scripted pixel is pinned at
/// (`crate::pinned_screen`).
const SIZE: (i32, i32) = (1920, 1080);

/// **Where the buttons stand**, pinned as pixels at [`SIZE`]: each key's
/// button's centre. Both are the first of their row, so one pixel.
const BUTTONS_AT: [(char, (i32, i32)); 2] = [('c', (128, 1054)), ('f', (128, 1054))];

fn units_selected() -> Input {
    let mut i = Input::new(0);
    i.selection = vec![ObjId { who: 0, o: 1 }];
    i
}

fn building_selected() -> Input {
    let mut i = Input::new(0);
    i.selection = vec![ObjId { who: 0, o: 2000 }];
    i.building = true;
    i
}

/// **The menu follows the selection**, as the keys' arms do, and a press
/// inside a button is its key: nothing selected, no button; units, the
/// build keys'; a building, the train keys'. The pinned pixels are each
/// key's button at the scripts' size, and no pixel a script presses or
/// right-presses falls inside any button the HUD could show.
#[test]
fn the_menu_follows_the_selection_and_a_button_is_its_key() {
    assert!(menu(&Input::new(0), SIZE).is_empty());
    let keys =
        |i: &Input| -> Vec<(char, i32)> { menu(i, SIZE).iter().map(|b| (b.key, b.ty)).collect() };
    assert_eq!(keys(&units_selected()), BUILD_KEYS.to_vec());
    assert_eq!(keys(&building_selected()), TRAIN_KEYS.to_vec());
    for (key, px) in BUTTONS_AT {
        let input = if TRAIN_KEYS.iter().any(|(k, _)| *k == key) {
            building_selected()
        } else {
            units_selected()
        };
        let b = menu(&input, SIZE)
            .into_iter()
            .find(|b| b.key == key)
            .unwrap();
        assert_eq!(b.rect.centre(), px, "{key}'s button");
        let press = Event::Press { x: px.0, y: px.1 };
        assert_eq!(route(&input, SIZE, press), Event::Key(key));
        assert_eq!(route(&Input::new(0), SIZE, press), press, "no selection");
        let right = Event::RightPress { x: px.0, y: px.1 };
        assert_eq!(
            route(&input, SIZE, right),
            right,
            "a right press is the layer's"
        );
    }
    let buttons: Vec<Rect> = [units_selected(), building_selected()]
        .iter()
        .flat_map(|i| menu(i, SIZE))
        .map(|b| b.rect)
        .collect();
    for (frame, _, ev) in right_events() {
        if let Event::Press { x, y } | Event::RightPress { x, y } = ev {
            assert!(
                !buttons.iter().any(|r| r.contains(x, y)),
                "frame {frame}: the script's ({x}, {y}) is under a button"
            );
        }
    }
}

/// The clicked script's events (`docs/CLIENT_INPUT.md` §5) with each key
/// replaced by a press on its button's pinned pixel.
fn button_events() -> Vec<Scripted> {
    events(false)
        .into_iter()
        .map(|(frame, at, ev)| match ev {
            Event::Key(k) => {
                let (_, (x, y)) = BUTTONS_AT.iter().find(|(c, _)| *c == k).unwrap();
                (frame, at, Event::Press { x: *x, y: *y })
            }
            _ => (frame, at, ev),
        })
        .collect()
}

/// An order a key stands behind: the `queue_up` a train key mints, and
/// the `build` the press after a build key mints.
fn keyed(o: &Order) -> bool {
    matches!(o.2[1], Command::QueueUp { .. } | Command::Build { .. })
}

/// The clicked script's keyed orders minted whole, of all of them.
fn keyed_whole(ours: &[Order]) -> (usize, usize) {
    let clicked = script::clicked();
    let whole = clicked
        .iter()
        .filter(|o| keyed(o))
        .filter(|o| ours.contains(o))
        .count();
    (whole, clicked.iter().filter(|o| keyed(o)).count())
}

/// **`Hud: the clicked file from buttons`** (`docs/HUD.md` §5): the
/// clicked script's events with each key replaced by a press on its
/// button, through the window's own routing ([`event`]) at the scripts'
/// size, mint the clicked script field for field, and the file written
/// from them is the clicked file byte for byte — of the orders a key
/// stands behind, [`script::HUD_BUTTONS`]. Made to fail twice: the same
/// presses through the layer alone (the HUD before this item) mint none
/// of them, and on a window 80 pixels shorter the buttons stand off their
/// pinned pixel and mint none.
#[test]
fn the_buttons_mint_the_clicked_file() {
    let through = |size: (i32, i32)| {
        move |i: &mut Input, f: i32, ev: Event, v: &View| event(i, size, f, ev, v)
    };
    let Some((ours, _)) = mint_with(button_events(), through(SIZE)) else {
        return;
    };
    assert_eq!(keyed_whole(&ours), script::HUD_BUTTONS, "{ours:?}");
    assert_eq!(ours, script::clicked());

    let (layer, _) = mint_with(button_events(), |i, f, ev, v| i.event(f, ev, v)).unwrap();
    assert_eq!(keyed_whole(&layer), (0, 2), "the layer alone: {layer:?}");
    let (moved, _) = mint_with(button_events(), through((1920, 1000))).unwrap();
    assert_eq!(keyed_whole(&moved), (0, 2), "the buttons moved: {moved:?}");

    let home = std::env::var_os("HOME").unwrap_or_default();
    let (Ok(minted), Ok(file)) = (
        std::fs::read(std::path::Path::new(&home).join(MINTED_RCX)),
        std::fs::read(std::path::Path::new(&home).join(CLICKED_RCX)),
    ) else {
        println!("skipping the bytes: no {MINTED_RCX} or {CLICKED_RCX}");
        return;
    };
    assert_eq!(file.len(), CLICKED_RCX_BYTES);
    let path = std::path::Path::new(&home).join(MINTED_RCX);
    let rec = rondata::recgame::read(&path.to_string_lossy()).unwrap();
    let lo = rondata::commands::decode(&rec.packages[0].data).unwrap();
    let last = rec.packages.last().unwrap().frame;
    let write = |orders: &[Order]| {
        let mut w = rec.clone();
        w.packages = rondata::recgame::minted_packages(lo[0].clone(), orders, last).unwrap();
        rondata::recgame::write_gz(&w)
    };
    assert!(write(&ours) == file, "the file from the buttons' orders");
    assert!(write(&ours) != minted, "not the first script's file");
    assert!(write(&layer) != file, "the layer alone's is not the file");
}

/// **A press that is the HUD's is not the world's** (`docs/HUD.md` §5),
/// on the clicked script's frame 90 with the builder selected: a press on
/// the Farm's button chooses the Farm and leaves the selection whole, and
/// a second press there places nothing. Seen red with the routing
/// reversed: the same pixel through the layer alone places a Farm at the
/// ground under the button, and with nothing chosen clears the selection.
#[test]
fn a_press_on_a_button_is_not_a_press_on_the_world() {
    let Ok(mut r) = lobby() else {
        println!("skipping: no lobby");
        return;
    };
    let clicked = script::clicked();
    let mut next = 0;
    while r.frame() < 90 {
        while next < clicked.len() && i64::from(clicked[next].0) == r.frame() {
            r.issue(&clicked[next]);
            next += 1;
        }
        r.step();
    }
    let (scene, terrain) = (r.scene(None), r.terrain());
    let c = camera(None);
    let v = view(&c, &scene, &terrain);
    let (_, (bx, by)) = BUTTONS_AT[1];
    let builder = ObjId {
        who: 0,
        o: script::BUILDER,
    };
    let selected = || {
        let mut i = Input::new(0);
        i.selection = vec![builder];
        i
    };
    let press = Event::Press { x: bx, y: by };

    let mut hud = selected();
    assert_eq!(event(&mut hud, SIZE, 90, press, &v), None);
    assert_eq!(
        (hud.selection.clone(), hud.placing),
        (vec![builder], Some(417))
    );
    assert_eq!(event(&mut hud, SIZE, 90, press, &v), None, "places nothing");
    assert_eq!(
        (hud.selection.clone(), hud.placing),
        (vec![builder], Some(417))
    );

    // Reversed: the layer takes the press.
    let mut layer = selected();
    layer.placing = Some(417);
    let placed = layer.event(90, press, &v).expect("the layer places");
    assert!(matches!(placed.2[1], Command::Build { build: 417, .. }));
    let mut layer = selected();
    assert_eq!(layer.event(90, press, &v), None);
    assert!(layer.selection.is_empty(), "the layer's press clears");
}

/// The right-pressed script's playback, every unit and building on all
/// 300 frames (`rondata::diff::minted::RIGHT_PRESSED_WALKED`, run 1568).
const RIGHT_PRESSED_DUMP: &str = "gamelog-run1568-greatlakes-playback-of-right-pressed-300.txt";

/// The frame the panel's probe selects the Farm's site `0/2007`, after
/// the right presses and before the site finishes (254 in run1568), and
/// the pixel: its anchor, [`RIGHT`]'s.
const SITE_PRESS: i32 = 210;

/// **The panel's walk**: the right-pressed script's events and one press
/// selecting the Farm's site; a press mints nothing, so the walk is the
/// right-pressed file's. On every frame 1..=299, the panel's printed
/// lines for the selection as `(frame, object, its printed lines)`.
fn panel_walk(names: &Names) -> Option<Vec<(i64, ObjId, Line)>> {
    let mut r = lobby().ok()?;
    let mut evs = right_events();
    let site = RIGHT[0].4;
    evs.push((
        SITE_PRESS,
        None,
        Event::Press {
            x: site.0,
            y: site.1,
        },
    ));
    let mut input = Input::new(0);
    let mut out = Vec::new();
    let mut next = 0;
    for n in 1..=299 {
        while r.frame() < n {
            r.step();
        }
        while next < evs.len() && i64::from(evs[next].0) == n {
            let (frame, at, ev) = evs[next];
            let (scene, terrain) = (r.scene(None), r.terrain());
            let c = camera(at);
            if let Some(o) = event(&mut input, SIZE, frame, ev, &view(&c, &scene, &terrain)) {
                r.issue(&o);
            }
            next += 1;
        }
        if n == i64::from(SITE_PRESS) {
            let id = ObjId {
                who: 0,
                o: script::FARM_SITE,
            };
            assert_eq!(input.selection, vec![id], "the site selected");
        }
        let scene = r.scene(None);
        let lines = panel(&input, &scene, names);
        assert_eq!(lines.len(), input.selection.len(), "frame {n}");
        for (id, line) in input.selection.iter().zip(lines) {
            out.push((n, *id, line));
        }
    }
    Some(out)
}

/// The `a/b` pairs a printed line holds, in order.
fn printed_numbers(text: &str) -> Vec<i64> {
    text.split_whitespace()
        .filter_map(|t| t.split_once('/'))
        .flat_map(|(a, b)| [a.parse().unwrap(), b.parse().unwrap()])
        .collect()
}

/// The dump's numbers for an object, in the panel's order: its hits left
/// of `myhits`, and on an unfinished building (`flags & 4` clear)
/// `(int)construct_hits` of `myhits`. Hits left is
/// `ObjectData::hits_left@006535c0`, `hits(0) − damage` floored at 0: a
/// unit's `hits(0)` is its `myhits` (`rondata::diff::harness`'s hit-point
/// rows), a building's is `construct_hits` (`WallData::hits@00642bb0`,
/// `BuildData::hits@0062e740`: `hits(1)` is `myhits`), so a site's health
/// climbs with its construction (`docs/HUD.md` §3).
fn theirs(f: &rondata::gamelog::Frame, id: ObjId) -> Option<Vec<i64>> {
    let (who, o) = (i64::from(id.who), i64::from(id.o));
    if let Some(u) = f.units.iter().find(|u| u.who == who && u.o == o) {
        let hits = u.myhits?;
        return Some(vec![(hits - u.damage?).max(0), hits]);
    }
    let b = f.builds.iter().find(|b| b.who == who && b.o == o)?;
    let (hits, construct) = (b.myhits?, b.construct_hits?);
    let mut v = vec![(construct - b.damage?).max(0), hits];
    if b.flags & 4 == 0 {
        v.extend([construct, hits]);
    }
    Some(v)
}

/// **`Hud: the panel's values`** (`docs/HUD.md` §5): on every frame
/// run1568 prints, each number the panel prints for the selection is the
/// dump's own for that object on that frame — `(equal, compared)`,
/// [`script::HUD_PANEL`]. The selection includes an unfinished object
/// (the Farm's site, `construct_hits` climbing), so the quiet fields are
/// not all that is compared. Made to fail: the site's `construct_hits`
/// printed one higher parts on every frame it is compared; the dump read
/// one frame late parts too.
#[test]
fn the_panel_s_values_are_the_dump_s() {
    let root = std::env::var_os("RON_INSTALL")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "../../game".into());
    let install = rondata::Install::new(&root);
    let path = crate::world::gamelog_dir().join(RIGHT_PRESSED_DUMP);
    if !install.looks_valid() || !path.is_file() {
        println!("skipping: no install or no {}", path.display());
        return;
    }
    let loaded = rondata::load::load(&install).unwrap();
    let names = Names::new(&loaded);
    // Each button's label: its key and the type it orders, joined through
    // `TypeIndex`. Seen red as drawn first: `c` read `unit_names[50]`, the
    // Riflemen, for the Citizen it trains.
    let labels: Vec<String> = [building_selected(), units_selected()]
        .iter()
        .flat_map(|i| menu(i, SIZE))
        .map(|b| label(&b, &names))
        .collect();
    // Item 4195 widened both rows: The Art of War and the Slingers on a
    // building, the Barracks on units (seen red first as `a TYPE 572`,
    // before the names took the technologies).
    assert_eq!(
        labels,
        [
            "c Citizen",
            "a The Art of War",
            "s Slingers",
            "f Farm",
            "b Barracks"
        ]
    );
    let walk = panel_walk(&names).expect("the lobby");

    let mut dump = rondata::capture::indexed::IndexedCapture::open(&path).unwrap();
    let mut frames = std::collections::BTreeMap::new();
    for i in 0..dump.frames().len() {
        let f = dump.frame_state(i).unwrap();
        frames.insert(f.n, f);
    }
    let score = |late: i64, bump: bool| {
        let (mut equal, mut compared, mut site) = (0, 0, Vec::new());
        for (n, id, line) in &walk {
            let Some(f) = frames.get(&(n + late)) else {
                continue;
            };
            let Some(want) = theirs(f, *id) else { continue };
            let mut line = line.clone();
            if bump && let Some((c, _)) = line.building.as_mut() {
                *c += 1;
            }
            let text = line.text();
            for c in text.chars() {
                assert!(crate::overlay::has_glyph(c), "{c:?} in {text:?}");
            }
            let got = printed_numbers(&text);
            if got.len() != want.len() {
                // A perturbed side may disagree on the shape (one frame
                // late, the site finished): every number of it parts.
                assert!(
                    late != 0 || bump,
                    "frame {n} {id:?}: {text:?} against {want:?}"
                );
                compared += got.len().max(want.len());
                continue;
            }
            for (k, (g, w)) in got.iter().zip(&want).enumerate() {
                compared += 1;
                if g == w {
                    equal += 1;
                } else if !bump && late == 0 {
                    eprintln!("PANEL PARTS frame {n} {id:?} number {k}: ours {g} theirs {w}");
                }
            }
            if line.building.is_some() && late == 0 {
                site.push((*n, want[2]));
            }
        }
        (equal, compared, site)
    };
    let (equal, compared, site) = score(0, false);
    eprintln!(
        "PANEL {equal} of {compared}; the site unfinished on {} frames, construct_hits {:?} .. {:?}",
        site.len(),
        site.first(),
        site.last()
    );
    for (n, id, line) in walk
        .iter()
        .filter(|(n, ..)| [30, 60, 90, 120, 200, 210, 254, 299].contains(n))
    {
        eprintln!("PANEL frame {n} {id:?}: {}", line.text());
    }
    assert_eq!((equal, compared), script::HUD_PANEL, "HUD_PANEL");
    let distinct: std::collections::BTreeSet<i64> = site.iter().map(|s| s.1).collect();
    assert!(
        distinct.len() > 1,
        "the site's construct_hits climbs: {site:?}"
    );

    let (bumped, bumped_of, _) = score(0, true);
    assert_eq!(
        (bumped, bumped_of),
        (equal - site.len(), compared),
        "bumped"
    );
    let (late, late_of, _) = score(1, false);
    assert!(late < late_of, "one frame late parts: {late} of {late_of}");
    eprintln!(
        "PANEL made to fail: bumped {bumped} of {bumped_of}, one frame late {late} of {late_of}"
    );
}

/// **The panel's text**: a closing count past [`PANEL_LINES`], and every
/// printed character in the font.
#[test]
fn the_panel_closes_with_a_count_of_the_rest() {
    let line = Line {
        name: "Citizen".into(),
        health: (35, 50),
        building: None,
        figures: None,
    };
    let lines = vec![line.clone(); PANEL_LINES + 3];
    let text = panel_text(&lines);
    assert_eq!(text.len(), PANEL_LINES + 1);
    assert_eq!(text.last().unwrap(), "AND 3 MORE");
    assert_eq!(text[0], "Citizen 35/50");
    let site = Line {
        name: "Farm".into(),
        health: (400, 400),
        building: Some((143, 400)),
        figures: None,
    };
    assert_eq!(site.text(), "Farm 400/400 BUILDING 143/400");
    assert_eq!(printed_numbers(&site.text()), vec![400, 400, 143, 400]);
    let mut p = Painter::new(SIZE);
    p.text(0, 0, &site.text(), SCALE, overlay::INK);
    assert!(!p.pos.is_empty());
}
