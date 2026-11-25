//! The roads against run1559b's packet and on the two photographs (item
//! 4199, `docs/RENDERER.md` §3.19).

use super::*;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// One `element` row of `tools/render/road_records.py`.
#[derive(Clone, Debug)]
struct Element {
    page: u8,
    size: (u8, u8),
    uv: [f32; 4],
    flags: [u32; 8],
    cx: [f32; 8],
    cy: [f32; 8],
}

/// One `road` row.
#[derive(Clone, Copy, Debug)]
struct Record {
    elem: i32,
    flags: u32,
    rot: u8,
    age: u8,
    refs: u8,
}

/// What the packet's dump holds: the pieces, every record, and the drawn
/// pieces' boxes and corner texcoords (NW, NE, SW, SE).
struct Packet {
    elements: Vec<Element>,
    records: BTreeMap<(i32, i32), Record>,
    pieces: Vec<([f32; 4], [[f32; 2]; 4])>,
    /// Item 4653: each piece's tile-page texcoords (`tex1`) at the same
    /// corners, where its pass holds them.
    pages: Vec<Option<[[f32; 2]; 4]>>,
}

/// `tools/render/road_records.py` on a packet, the file at `$RON_ROADS_<RUN>`
/// or the item's lab directory. Nothing it holds enters the repository.
fn packet(run: &str) -> Option<Packet> {
    let var = format!("RON_ROADS_{}", run.to_uppercase());
    // Item 4653: run1814b's dump, with the tile page's texcoords.
    let item = if run == "1814b" {
        "2026-11-25-item-4653"
    } else {
        "2026-11-13-item-4199"
    };
    let path = std::env::var_os(&var).map(PathBuf::from).or_else(|| {
        std::env::var_os("HOME").map(|h| {
            PathBuf::from(h).join(format!("ron-data/lab-experiments/{item}/roads-{run}.txt"))
        })
    })?;
    let Ok(text) = std::fs::read_to_string(&path) else {
        println!("skipping: no road records at {}", path.display());
        return None;
    };
    let mut p = Packet {
        elements: Vec::new(),
        records: BTreeMap::new(),
        pieces: Vec::new(),
        pages: Vec::new(),
    };
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        let at = |k: &str| f.iter().position(|x| *x == k).expect("a key") + 1;
        let num = |i: usize| f[i].parse::<f32>().expect("a number");
        let pair = |q: &str| {
            let (u, v) = q.split_once(',').expect("a pair");
            [u.parse().expect("u"), v.parse().expect("v")]
        };
        match f.first().copied() {
            Some("element") => {
                let (fl, cx, cy, uv) = (at("flags"), at("cx"), at("cy"), at("uv"));
                p.elements.push(Element {
                    page: f[at("page")].parse().expect("page"),
                    size: (
                        f[at("w")].parse().expect("w"),
                        f[at("h")].parse().expect("h"),
                    ),
                    uv: [num(uv), num(uv + 1), num(uv + 2), num(uv + 3)],
                    flags: std::array::from_fn(|k| {
                        u32::from_str_radix(f[fl + k], 16).expect("hex")
                    }),
                    cx: std::array::from_fn(|k| num(cx + k)),
                    cy: std::array::from_fn(|k| num(cy + k)),
                });
            }
            Some("road") => {
                let n = |k: &str| f[at(k)].parse::<i32>().expect("a number");
                p.records.insert(
                    (f[1].parse().expect("x"), f[2].parse().expect("y")),
                    Record {
                        elem: n("elem"),
                        flags: u32::from_str_radix(f[at("flags")], 16).expect("hex"),
                        rot: n("rot") as u8,
                        age: n("age") as u8,
                        refs: f[at("refs")]
                            .split_once('/')
                            .and_then(|(a, _)| a.parse().ok())
                            .expect("refs"),
                    },
                );
            }
            Some("piece") => {
                let u = at("uv");
                p.pieces.push((
                    [num(2), num(3), num(4), num(5)],
                    std::array::from_fn(|k| pair(f[u + k])),
                ));
                let g = f.iter().position(|x| *x == "page").map(|i| i + 1);
                p.pages
                    .push(g.map(|g| std::array::from_fn(|k| pair(f[g + k]))));
            }
            _ => {}
        }
    }
    Some(p)
}

fn art() -> Option<RoadArt> {
    let root = std::env::var_os("RON_INSTALL")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("../../game"));
    let install = rondata::Install::new(&root);
    match rondata::artdata::road_art(&install) {
        Ok(a) => Some(a),
        Err(e) => {
            println!(
                "skipping: roads.xml does not read under {}: {e:?}",
                root.display()
            );
            None
        }
    }
}

/// **The pieces are the packet's** (item 4199, §3.19): `roads.xml`'s 57
/// `PAGEELEMENT`s, both pages in file order, are `RoadsPieces::
/// modern_elements` whole — page, size, the texcoords in sixteenths, the
/// centre offsets, and all eight orientations' flags [`rotations`] turns.
#[test]
fn the_pieces_and_their_turns_are_the_packet_s() {
    let (Some(art), Some(pk)) = (art(), packet("1559b")) else {
        return;
    };
    assert_eq!(art.pieces.len(), 57);
    assert_eq!(pk.elements.len(), art.pieces.len());
    let mut whole = 0;
    for (k, (ours, theirs)) in art.pieces.iter().zip(&pk.elements).enumerate() {
        let uv = [
            f32::from(ours.offset.0) / 16.0,
            f32::from(ours.offset.1) / 16.0,
            f32::from(ours.width) / 16.0,
            f32::from(ours.height) / 16.0,
        ];
        let same = ours.page == theirs.page
            && (ours.width, ours.height) == theirs.size
            && uv == theirs.uv
            && rotations(ours) == theirs.flags
            && ours.center.map(|c| c.0) == theirs.cx
            && ours.center.map(|c| c.1) == theirs.cy;
        if !same {
            println!(
                "element {k} {}: ours {ours:?} {:08x?}, theirs {theirs:?}",
                ours.name,
                rotations(ours)
            );
        }
        whole += usize::from(same);
    }
    assert_eq!(whole, 57);
    assert_eq!(art.named("C31"), Some(44));
    assert_eq!(art.named("C4"), Some(55));
}

/// **The quads are the packet's** (item 4199, §3.19): every piece the
/// packet drew, laid by [`lay`] from the record's own element and
/// orientation on its tile — the box and the four corners' texcoords —
/// 28 of 28 on run1559b's frame 3294, the capital's ring.
#[test]
fn the_quads_are_the_packet_s() {
    let (Some(art), Some(pk)) = (art(), packet("1559b")) else {
        return;
    };
    let drawn: BTreeSet<String> = pk.pieces.iter().map(|q| format!("{q:?}")).collect();
    let mut hit = 0;
    let mut tried = 0;
    for (&tile, r) in &pk.records {
        let Some(piece) = usize::try_from(r.elem).ok().and_then(|e| art.pieces.get(e)) else {
            continue;
        };
        if r.refs == 0 || r.rot > 7 {
            continue;
        }
        let l = lay(piece, r.elem as usize, tile, r.rot);
        let ours = format!("{:?}", (l.rect, l.uv));
        // Only the drawn records have a piece in the buffer: those inside
        // the view.
        let near = pk.pieces.iter().any(|(b, _)| {
            (b[0]..b[2]).contains(&(tile.0 as f32 + 0.5))
                && (b[1]..b[3]).contains(&(tile.1 as f32 + 0.5))
        });
        if !near {
            continue;
        }
        tried += 1;
        if drawn.contains(&ours) {
            hit += 1;
        } else {
            println!("tile {tile:?} elem {} rot {}: ours {ours}", r.elem, r.rot);
        }
    }
    println!(
        "quads {hit} of {tried}, the packet's pieces {}",
        pk.pieces.len()
    );
    assert_eq!((hit, tried, pk.pieces.len()), (28, 28, 28));
}

/// **The records against the packet's** (item 4199, §3.19): our walk's
/// mesh on frame 3279 ([`crate::ground::tests::run1559b`]'s) and the
/// packet's on 3294, record for record over every record the packet holds
/// with a reference — the connections, the white lines, the intersections,
/// and on the drawn ring the piece and its orientation.
#[test]
fn the_records_against_the_packet_s() {
    let Some(pk) = packet("1559b") else {
        return;
    };
    let Some(mut p) = crate::ground::tests::probe_at(crate::ground::tests::run1559b()) else {
        return;
    };
    let scene = p.world.runner.roads();
    let ours = candidates(&scene);
    // Item 4653: the plates are laid beside bucket 2; this row counts the
    // modern pieces it was written on.
    let all = p.world.road_pieces();
    let plates = all
        .iter()
        .filter(|(b, _)| *b == crate::world::PLATE_BUCKET)
        .count();
    println!("plates laid on run1559b: {plates}");
    let laid: BTreeMap<(i32, i32), Laid> = all
        .into_iter()
        .filter(|(b, _)| *b == crate::world::ROAD_BUCKET)
        .map(|(_, l)| (l.tile, l))
        .collect();
    let mut n = [0usize; 6];
    for (&t, r) in pk.records.iter().filter(|(_, r)| r.refs != 0) {
        n[0] += 1;
        let Some(&c) = ours.get(&t) else {
            println!("tile {t:?}: theirs {:08x}, ours none", r.flags);
            continue;
        };
        n[1] += 1;
        let byte = |w: u32, k: u32| (w >> (8 * k)) & 0xff;
        let same = [3, 1, 0].map(|k| byte(c, k) == byte(r.flags, k));
        for (i, s) in same.iter().enumerate() {
            n[2 + i] += usize::from(*s);
        }
        if same != [true; 3] {
            println!("tile {t:?}: theirs {:08x}, ours {c:08x}", r.flags);
        }
        if let Some(l) = laid.get(&t) {
            let hit = l.elem as i32 == r.elem && l.rot == r.rot;
            n[5] += usize::from(hit);
            println!(
                "drawn {t:?}: theirs elem {} rot {} age {}, ours elem {} rot {}{}",
                r.elem,
                r.rot,
                r.age,
                l.elem,
                l.rot,
                if hit { "" } else { " PARTS" }
            );
        }
    }
    println!(
        "records {} with a reference, {} ours; connections {}, white {}, intersections {}; drawn {} laid, {} of them the packet's piece",
        n[0],
        n[1],
        n[2],
        n[3],
        n[4],
        laid.len(),
        n[5]
    );
    assert_eq!(
        (n[0], n[1], n[2], n[3], n[4], laid.len(), n[5]),
        RUN1559B_RECORDS
    );
}

/// **`RUN1559B_RECORDS`** (item 4199): the packet's records with a
/// reference, ours among them, then those whose connections, white lines
/// and intersections agree; the pieces we lay (age bucket 2, page 0), and
/// those that are the packet's piece and orientation. The connections are
/// the simulation's, whole. The 23 white partings are junctions (the T at
/// (19, 163), whose south neighbour holds no record on the packet; two
/// crossroads at (199–201, 110) and (223–224, 79–87)); the intersections
/// are sticky — `mark_intersections` only ever sets them, as each tile of
/// a ring is laid beside a half-built one — and evaluated once here they
/// part on 279 (parked 4199a). The pieces part where they do: an
/// intersection bit widens `find_map`'s candidates, and the fractal picks
/// another straight.
const RUN1559B_RECORDS: (usize, usize, usize, usize, usize, usize, usize) =
    (599, 599, 599, 576, 320, 485, 286);

/// **The ring's tiles** (item 4199, §3.19; the rule committed before its
/// count): for every building whose type is a city (`is(VILLAGE)`), the
/// ring `BuildType::place_roads@0063c580`'s city arm walks — the border of
/// `[c − 1, c − 1 + size]` on each axis, `c` the tile less `size >> 1` —
/// whether or not a road stands on it; each pixel of the photograph on such
/// a tile that `ground_why` calls ground or an object's window, as the
/// plaza's row takes them.
fn ring_pixels(p: &mut crate::ground::tests::Probe) -> Vec<Option<(i32, i32)>> {
    let scene = p.world.runner.scene(None);
    let types = p.world.build_types_for_test();
    let mut ring = BTreeSet::new();
    for b in &scene.buildings {
        let Some(t) = b.ty.filter(|&t| sim::cities::build::is_city(types, t)) else {
            continue;
        };
        let (sx, sy) = (types[t].x_size, types[t].y_size);
        let tile = (
            b.pos.x / render::terrain::UNITS_PER_TILE,
            b.pos.y / render::terrain::UNITS_PER_TILE,
        );
        let (x0, y0) = (tile.0 - (sx >> 1) - 1, tile.1 - (sy >> 1) - 1);
        let (x1, y1) = (x0 + sx, y0 + sy);
        for x in x0..=x1 {
            for y in y0..=y1 {
                if x == x0 || x == x1 || y == y0 || y == y1 {
                    ring.insert((x, y));
                }
            }
        }
    }
    let (_, any, why) = crate::ground::tests::ground_why(p);
    any.into_iter()
        .zip(why)
        .map(|(t, w)| t.filter(|t| ring.contains(t) && (w == 0 || w == 2)))
        .collect()
}

/// The ring's row on one photograph: before (no roads), drawn, and the
/// made-to-fail, every piece moved [`RING_MOVE`] tiles east.
fn ring_rows(p: &mut crate::ground::tests::Probe, name: &str) -> [[(usize, usize); 2]; 3] {
    use crate::weathering::tests::{count, plaza_tiles, still};
    let pixels = ring_pixels(p);
    if let Some(path) = std::env::var_os("RON_RING_DUMP") {
        let w = p.photo.width;
        let rows: String = pixels
            .iter()
            .enumerate()
            .filter_map(|(i, t)| t.map(|t| format!("{} {} {} {}\n", i % w, i / w, t.0, t.1)))
            .collect();
        std::fs::write(
            PathBuf::from(path).join(format!("ring-pixels-{name}.txt")),
            rows,
        )
        .expect("the dump writes");
    }
    let mut out = [[(0, 0); 2]; 3];
    for (k, (on, shift, what)) in [
        (false, (0, 0), "before"),
        (true, (0, 0), "drawn"),
        (true, RING_MOVE, "moved"),
    ]
    .into_iter()
    .enumerate()
    {
        p.world.roads = on;
        p.world.roads_shift = shift;
        let s = still(p, &format!("ring-{name}-{what}"));
        out[k] = count(&format!("{name} ring {what}"), &plaza_tiles(p, &pixels, &s));
    }
    p.world.roads = true;
    p.world.roads_shift = (0, 0);
    out
}

/// The made-to-fail's move: past the capital's ring, eight tiles across,
/// so no column lands on the other.
const RING_MOVE: (i32, i32) = (12, 0);

/// **`RUN1559B_3279_RING`** (item 4199; the rule, [`ring_pixels`],
/// committed before the count): `[before, drawn, moved]`, each `[(agree,
/// scored)]` on hue then luma, 24 tiles scored. Hue **1 → 21**; luma 20
/// → 19. Tile (19, 160), 915 pixels: theirs (150, 126, 102) Y 130.4,
/// before (grass) parts on hue, drawn (157, 145, 117) Y 145.7 agrees on
/// both. Across the asphalt's middle the photograph is 0.89 of ours on
/// every channel ((133, 123, 100) against (148, 138, 112)), which draw
/// 91's state does not explain (parked 4199b): luma does not move. The
/// three that part on hue: (18, 156) and (19, 157), under the fire's
/// glow; (19, 163), the T, whose white lines part (4199a). Moved twelve
/// tiles east, back to the before counts. Before and moved 1 → 2 at the
/// merge beside item 4201's fire: tile (19, 156), 166 pixels, under the
/// fire's glow, theirs (177, 168, 75), the still's (162, 142, 64), now
/// agrees on hue with no road drawn.
/// Item 4212: drawn unchanged; before and moved hue 2 → 1, luma 20 → 8 and 7: the grass under the ring lit at the frame's white, §3.21. Tile (19, 160), theirs (150, 126, 102) Y 130.4, drawn (157, 145, 117) Y 145.7 → (145, 137, 107) Y 135.8: 4199b's 0.89 is the frame's white.
/// Item 4221: luma before 8 → 9, drawn 19 → 20, moved 7 → 9: the trees tinted and blended move the frame's white (§3.23).
/// Item 4233: before and moved hue 1 → 2, luma 9 → 10; drawn luma 20 → 21: the particles drawn and 4201's stand-ins gone (§3.25) move the frame's mean, so its white (§3.21).
/// Item 4237: before and moved hue 2 → 1: the smoke's dense core blended in HDR (§3.25) moves the frame's mean, so its white (§3.21).
/// Item 4241: luma before 10 → 9, drawn 21 → 20, moved 10 → 7: the bloom drawn and a plume's pixel clamped after the white (§3.25).
/// Item 4269: the luma 9 → 10 and 7 → 8 of 24: the range lit by its own normals (item 4269, `docs/RENDERER.md` §3.22) moves the frame's mean, so its white.
/// Item 4272: the luma before 10 → 6 and moved 8 → 6 of 24, drawn unchanged (a named fall: the grass under the ring with no road, which the drawn row does not score): the template alphas untinted (item 4272, `docs/RENDERER.md` §3.22) move the frame's mean, so its white.
/// Item 4292: the luma before and moved 6 → 11, drawn 20 → 21 of 24 (a rise from 6 and 20): the plain ground lit by the land's own N·L (item 4292, `docs/RENDERER.md` §3.22) moves the frame's mean, so its white.
/// Item 4311: the luma before 11 → 15 and moved 11 → 13 of 24, drawn unchanged: the map's edge border drawn (item 4311, `docs/RENDERER.md` §3.15) moves the frame's mean, so its white.
const RUN1559B_3279_RING: [[(usize, usize); 2]; 3] = [
    [(1, 24), (15, 24)],
    [(21, 24), (21, 24)],
    [(1, 24), (13, 24)],
];
/// **`RUN1570_RING`** (item 4199), the same on run1570's photograph: hue
/// **0 → 22** of 24, luma 14 → 14. The two that part on hue: (18, 163),
/// (86, 41, 33), the name plate; (19, 163), the T.
/// Item 4212: drawn luma 14 → 15; before and moved luma 14 → 9: the grass under the ring lit at the frame's white, §3.21.
/// Item 4221: before and moved luma 9 → 10, the trees tinted and blended (§3.23).
/// Item 4233: drawn luma 15 → 16, the particles drawn and 4201's stand-ins gone (§3.25) move the frame's mean, so its white (§3.21).
/// Item 4237: before luma 10 → 11, the smoke blended in HDR moving the frame's white (§3.25).
/// Item 4240: before luma 11 → 10, moved 10 → 9: the emission's starts (the node's direction and the original's draws, §3.25) move the frame's mean, so its white (§3.21).
/// Item 4241: before luma 10 → 9, the bloom drawn and a plume's pixel clamped after the white (§3.25).
/// Item 4269: the first luma 9 → 10 of 24: the range lit by its own normals (item 4269, `docs/RENDERER.md` §3.22) moves the frame's mean, so its white.
/// Item 4272: the luma before 10 → 9 and moved 9 → 8 of 24: the template alphas untinted (item 4272, `docs/RENDERER.md` §3.22) move the frame's mean, so its white.
/// Item 4275: the luma before 9 → 8 of 24, a named fall on the grass under the ring with no road drawn: the snowcap underlay drawn (item 4275, `docs/RENDERER.md` §3.10) moves the frame's mean, so its white.
/// Item 4292: the luma before and moved 8 → 11 of 24 (a rise from 8), drawn unchanged: the plain ground lit by the land's own N·L (item 4292, `docs/RENDERER.md` §3.22) moves the frame's mean, so its white.
/// Item 4311: the luma before and moved 11 → 12 of 24, drawn unchanged: the map's edge border drawn (item 4311, `docs/RENDERER.md` §3.15) moves the frame's mean, so its white.
const RUN1570_RING: [[(usize, usize); 2]; 3] = [
    [(0, 24), (12, 24)],
    [(22, 24), (16, 24)],
    [(0, 24), (12, 24)],
];

/// **The ring's row** (item 4199, §3.19): on run1559b's and run1570's
/// photographs, before, drawn, and moved.
#[test]
fn the_asphalt_ring_around_a_city() {
    let Some(mut p) = crate::ground::tests::probe_at(crate::ground::tests::run1559b()) else {
        return;
    };
    let a = ring_rows(&mut p, "run1559b");
    let Some(mut p) = crate::ground::tests::probe_at(crate::weathering::tests::run1570()) else {
        return;
    };
    let b = ring_rows(&mut p, "run1570");
    assert_eq!(a, RUN1559B_3279_RING);
    assert_eq!(b, RUN1570_RING);
}

/// **Draw 91's state** (item 4199, §3.19): on run1566b's card trace, the
/// pass after 3279, `modern_road_pieces.tga` under enables `0x20004`,
/// `SRCALPHA`/`INVSRCALPHA`, stage 0 `MODULATE(TEXTURE, DIFFUSE)` with A
/// `SELECTARG1(TEXTURE)`, stage 1 disabled, `TL_FAKE_LIGHT` and no
/// lighting or vertex colour; 450 vertices, the packet's own count.
#[test]
fn draw_91_s_state() {
    use rondata::trace::card;
    let path = std::env::var_os("RON_CARD_TRACE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME").unwrap_or_default();
            PathBuf::from(home).join("ron-golden/run1566b/map-14/rontrace.log")
        });
    let Ok(bytes) = std::fs::read(&path) else {
        println!("skipping: no card trace at {}", path.display());
        return;
    };
    let all = card::draws(&bytes);
    let r = all
        .iter()
        .find(|d| d.frame == 3279 && d.index == 91)
        .expect("the pass after 3279 holds draw 91");
    assert_eq!(r.file(0), "modern_road_pieces.tga");
    assert_eq!((r.enables(), r.src_blend(), r.dst_blend()), (0x20004, 5, 6));
    assert_eq!(
        r.colour_op(0),
        (card::TOP_MODULATE, card::TA_TEXTURE, card::TA_DIFFUSE)
    );
    assert_eq!(r.alpha_op(0), (1, card::TA_TEXTURE, card::TA_CURRENT));
    assert_eq!(r.colour_op(1).0, 0, "stage 1 disabled");
    // `ps_global.hlsl`'s unlit branch: no vertex colour, so `vDiff` is 1
    // and the stage is the texel, raised by `TL_FAKE_LIGHT`.
    assert!(r.has("TL_FAKE_LIGHT"));
    assert!(!r.has("ENABLE_LIGHTING") && !r.has("ENABLE_VERTEX_COLOR_DIFFUSE"));
    // The packet's buffer: 450 vertices.
    assert_eq!(r.args[1], 450);
}

/// **The plates' box on run1814b** (item 4653, `docs/RENDERER.md` §6.2):
/// the tiles, inclusive, that `tools/render/road_records.py` finds ringing
/// slot 0's city on the packet, x 7 to 19 and y 151 to 163. It holds 61
/// records, 54 with a reference, and the packet's helper 0 (page 0, age
/// bucket 0) draws 54 pieces. The seven without one are `refs 0/0` at
/// rotation 10, which `build_render_data` skips; our door holds the 54.
const RUN1814B_PLATE_BOX: [i32; 4] = [7, 151, 19, 163];

/// **`RUN1814B_3294_PLATES`** (item 4653; §6.2): `[before, drawn,
/// theirs]`, each `[(agree, scored)]` on hue then luma, over each road
/// record of our door inside [`RUN1814B_PLATE_BOX`] with at least
/// `HUE.min_pixels` ground or object-window pixels, as the ring's row takes
/// them. Before is the roads as item 4199 left them, bucket 2 alone, which
/// lays nothing in the Ancient Age: the mint, hue **12** and luma 43 of 46,
/// and the made-to-fail. Drawn, hue **37**; theirs, the packet's own element
/// and orientation on every tile (the by-half), **43** and luma 42. Tile
/// (19, 159), 915 pixels: theirs (235, 182, 65) Y 184.7, before (177, 178,
/// 74), drawn (217, 185, 69) Y 181.7. On the way: the file's alpha as it
/// stands, on the frame, 29 (theirs 27); blended on the HDR target, 32
/// (30); its complement, 37 (43).
/// **Item 4661, a named fall: drawn 37 → 24 and luma 43 → 42, theirs 43 →
/// 22 and luma 42 → 43.** Every number above was read on the national
/// borders' alpha ramp: `PLATE_ALPHA` shared `BORDER_ALPHA_INDEX`'s cache
/// key, 913, so the plates never sampled `early_road_pieces.tga`. On its
/// own alpha, uncomplemented, and untested in depth (run1833's draw 81),
/// they lay sand in tile-sized squares where the photograph has bands
/// (4661a). Tile (19, 159): theirs (235, 182, 65), drawn (208, 184, 68).
/// **Item 4665: drawn hue 24 → 41, theirs 22 → 44; luma 42 and 43 → 42
/// and 42.** 921 was the surf's mask 1 (`SURF_ALPHA + 1`), so the plates
/// still never sampled `early_road_pieces.tga`. On it (key 902), each plate
/// laid twice as draw 81 lays it ([`crate::world::PLATE_LAYS`]). Laid once,
/// 41 and 44 too, luma 43 and 42. Tile (19, 159): drawn (226, 187, 67)
/// Y 185.3.
const RUN1814B_3294_PLATES: [[(usize, usize); 2]; 3] = [
    [(12, 46), (43, 46)],
    [(41, 46), (42, 46)],
    [(44, 46), (42, 46)],
];

/// **The plates' row** (item 4653, §6.2): our door holds the packet's 54
/// records with a reference in the box, and no other; the photograph's
/// pixels on them against the still, before and drawn.
#[test]
fn the_plates_on_run1814b() {
    use crate::weathering::tests::{count, plaza_tiles, still};
    let Some(mut p) = crate::farms::tests::probe() else {
        return;
    };
    let [x0, y0, x1, y1] = RUN1814B_PLATE_BOX;
    let inside = |x: i32, y: i32| (x0..=x1).contains(&x) && (y0..=y1).contains(&y);
    let scene = p.world.runner.roads();
    let tiles: BTreeSet<(i32, i32)> = scene
        .roads
        .iter()
        .filter(|r| inside(r.x, r.y))
        .map(|r| (r.x, r.y))
        .collect();
    let referenced = scene
        .roads
        .iter()
        .filter(|r| inside(r.x, r.y) && r.refs != 0)
        .count();
    assert_eq!(
        (tiles.len(), referenced),
        (54, 54),
        "the packet's records with a reference in the box"
    );
    let (_, any, why) = crate::ground::tests::ground_why(&mut p);
    let pixels: Vec<Option<(i32, i32)>> = any
        .into_iter()
        .zip(why)
        .map(|(t, w)| t.filter(|t| tiles.contains(t) && (w == 0 || w == 2)))
        .collect();
    // The by-half: the packet's own element and orientation on each tile.
    let picks = packet("1814b").map(|pk| {
        pk.records
            .iter()
            .filter(|(_, r)| r.refs != 0 && r.rot <= 7 && r.elem >= 0)
            .map(|(&t, r)| (t, (r.elem as usize, r.rot)))
            .collect::<BTreeMap<_, _>>()
    });
    let mut rows = [[(0, 0); 2]; 3];
    let once = std::env::var_os("RON_PLATES_ONCE").is_some();
    for (k, (early, pick, what)) in [
        (false, false, "before"),
        (true, false, "drawn"),
        (true, true, "theirs"),
    ]
    .into_iter()
    .enumerate()
    {
        if pick && picks.is_none() {
            continue;
        }
        p.world.roads = true;
        p.world.early_roads = early;
        p.world.plate_lays = if once { 1 } else { crate::world::PLATE_LAYS };
        p.world.road_pick = if pick { picks.clone() } else { None };
        p.world.relay_roads();
        let s = still(&mut p, &format!("run1814b-3294-plates-{what}"));
        rows[k] = count(
            &format!("RUN1814B_3294_PLATES {what}"),
            &plaza_tiles(&p, &pixels, &s),
        );
    }
    p.world.early_roads = true;
    p.world.road_pick = None;
    assert_eq!(rows, RUN1814B_3294_PLATES);
}

/// **`RUN1814B_PLATE_QUADS`** (item 4653, §6.2): on run1814b's packet,
/// `[the packet's pieces, their tile-page corners at the box over
/// PLATE_PAGE_TILES, the records laid by lay from their own element and
/// orientation that are a drawn piece, those records]`.
/// 54 pieces, 54 pages, 53 of 54 records: the one apart is (12, 156),
/// element 1 at orientation 5, a piece the buffer holds no box of.
const RUN1814B_PLATE_QUADS: [usize; 4] = [54, 54, 53, 54];

/// **The plates' quads are the packet's** (item 4653, §6.2): helper 0's
/// pieces, page 0's age bucket 0, are [`lay`]'s boxes and texcoords from
/// each record's element and orientation, and the tile page's texcoord at
/// every corner is the corner's tile over [`crate::world::PLATE_PAGE_TILES`]
/// — the tile's, never the piece's. Made to fail: the page's texcoord at
/// a third of the tile, or at the piece's own.
#[test]
fn the_plates_quads_are_the_packet_s() {
    let (Some(art), Some(pk)) = (art(), packet("1814b")) else {
        return;
    };
    let per = crate::world::PLATE_PAGE_TILES;
    let page_of = |b: &[f32; 4], k: f32| {
        [(b[0], b[1]), (b[2], b[1]), (b[0], b[3]), (b[2], b[3])].map(|(x, y)| [x / k, y / k])
    };
    let pages = pk
        .pieces
        .iter()
        .zip(&pk.pages)
        .filter(|((b, _), g)| **g == Some(page_of(b, per)))
        .count();
    let thirds = pk
        .pieces
        .iter()
        .zip(&pk.pages)
        .filter(|((b, _), g)| **g == Some(page_of(b, 3.0)))
        .count();
    let own = pk
        .pieces
        .iter()
        .zip(&pk.pages)
        .filter(|((_, uv), g)| **g == Some(*uv))
        .count();
    assert_eq!((thirds, own), (0, 0), "the made-to-fail: no other page");
    let drawn: BTreeSet<String> = pk.pieces.iter().map(|q| format!("{q:?}")).collect();
    let (mut hit, mut tried) = (0, 0);
    for (&tile, r) in &pk.records {
        let Some(piece) = usize::try_from(r.elem).ok().and_then(|e| art.pieces.get(e)) else {
            continue;
        };
        if r.refs == 0 || r.rot > 7 {
            continue;
        }
        // Only the records in view have a piece in the buffer.
        let near = pk.pieces.iter().any(|(b, _)| {
            (b[0]..b[2]).contains(&(tile.0 as f32 + 0.5))
                && (b[1]..b[3]).contains(&(tile.1 as f32 + 0.5))
        });
        if !near {
            continue;
        }
        tried += 1;
        let l = lay(piece, r.elem as usize, tile, r.rot);
        if drawn.contains(&format!("{:?}", (l.rect, l.uv))) {
            hit += 1;
        } else {
            println!(
                "plate quad {tile:?} elem {} rot {}: ours {:?}",
                r.elem,
                r.rot,
                (l.rect, l.uv)
            );
        }
    }
    let row = [pk.pieces.len(), pages, hit, tried];
    println!("run1814b plate quads: {row:?}");
    assert_eq!(row, RUN1814B_PLATE_QUADS);
}

/// **`RUN1814B_PLATE_PIECES`** (item 4653, §6.2): our walk on run1814b
/// against the packet's records with a reference in
/// [`RUN1814B_PLATE_BOX`]: `[the packet's, ours laid as plates, ours on
/// the packet's element and orientation]`.
/// 54, 50, 29: we lay no plate at element 1's (12, 155), (12, 156),
/// (15, 155) and (15, 156), where those four go not traced (4653b); the
/// 21 others part on the orientation or the straight the fractal picks,
/// as run1559b's do (4199a).
const RUN1814B_PLATE_PIECES: [usize; 3] = [54, 50, 29];

/// **The plates we lay against the packet's records** (item 4653, §6.2):
/// on run1814b's lobby at the photograph's frame, every piece our walk
/// lays in the box is a plate (slot 0 is in the Ancient Age), and each
/// against the record's own element and orientation.
#[test]
fn the_plates_against_the_packet_s() {
    let Some(pk) = packet("1814b") else {
        return;
    };
    let Some(mut p) = crate::farms::tests::probe() else {
        return;
    };
    let [x0, y0, x1, y1] = RUN1814B_PLATE_BOX;
    let inside = |t: (i32, i32)| (x0..=x1).contains(&t.0) && (y0..=y1).contains(&t.1);
    let ours: BTreeMap<(i32, i32), (i32, Laid)> = p
        .world
        .road_pieces()
        .into_iter()
        .filter(|(_, l)| inside(l.tile))
        .map(|(b, l)| (l.tile, (b, l)))
        .collect();
    let plates = ours
        .values()
        .filter(|(b, _)| *b == crate::world::PLATE_BUCKET)
        .count();
    let mut theirs = 0;
    let mut same = 0;
    for (&t, r) in pk
        .records
        .iter()
        .filter(|(t, r)| inside(**t) && r.refs != 0)
    {
        theirs += 1;
        let Some((_, l)) = ours.get(&t) else {
            println!(
                "plate {t:?}: theirs elem {} rot {}, ours none",
                r.elem, r.rot
            );
            continue;
        };
        let hit = l.elem as i32 == r.elem && l.rot == r.rot;
        same += usize::from(hit);
        if !hit {
            println!(
                "plate {t:?}: theirs elem {} rot {} flags {:08x}, ours elem {} rot {} PARTS",
                r.elem, r.rot, r.flags, l.elem, l.rot
            );
        }
    }
    let row = [theirs, plates, same];
    println!("run1814b plates: {row:?}");
    assert_eq!(row, RUN1814B_PLATE_PIECES);
}

/// **Every alpha key is its own** (item 4661): `Gpu::alpha_texture` caches
/// an alpha by its index, so two layers on one key share the first one's
/// texture. The plates sat on the borders' 913 from item 4653 to 4661 and
/// sampled the borders' ramp. Made to fail: `PLATE_ALPHA` back at 913;
/// item 4665, at 921, inside the surf's masks.
#[test]
fn every_alpha_key_is_its_own() {
    use crate::world::*;
    let keys = [
        ("WATER_ALPHA", WATER_ALPHA),
        ("LAND_FADE_ALPHA", LAND_FADE_ALPHA),
        ("WEATHER_ALPHA", WEATHER_ALPHA),
        ("ROAD_ALPHA", ROAD_ALPHA),
        ("SNOWCAP_ALPHA", SNOWCAP_ALPHA),
        ("BORDER_ALPHA_INDEX", BORDER_ALPHA_INDEX),
        ("EDGE_ALPHA", EDGE_ALPHA),
        ("FARM_PIG_ALPHA", FARM_PIG_ALPHA),
        ("OIL_ALPHA", OIL_ALPHA),
        ("LAND_ROUGH_ALPHA", LAND_ROUGH_ALPHA),
        ("LAND_OIL_ALPHA", LAND_OIL_ALPHA),
        ("FARM_WHEAT_ALPHA", FARM_WHEAT_ALPHA),
        ("SURF_ALPHA", SURF_ALPHA),
        ("PLATE_ALPHA", PLATE_ALPHA),
        ("MOUNTAIN_ALPHA", MOUNTAIN_ALPHA),
    ];
    for (i, (a, ka)) in keys.iter().enumerate() {
        for (b, kb) in &keys[i + 1..] {
            assert_ne!(ka, kb, "{a} and {b} share alpha key {ka}");
        }
    }
    // Item 4665: a family that owns a range owns every key in it. The
    // plates moved to 921, the surf's mask 1, and sampled it.
    let surf = SURF_ALPHA..SURF_ALPHA + SURF_MASKS;
    for (a, k) in keys.iter().filter(|(a, _)| *a != "SURF_ALPHA") {
        assert!(
            !surf.contains(k),
            "{a} at {k} inside the surf's masks {surf:?}"
        );
    }
}

/// **`RUN1833_DRAW_81`** (item 4665, §6.2): run1833's draw 81, the plates,
/// against our plate mesh on run1814b's lobby at the photograph's frame:
/// `[its triangles, distinct by position, each drawn twice, ours distinct,
/// ours on its cells, those laid as many times as its, their corners, the tile page's texcoord equal, the
/// piece's texcoord equal, the corner's height within a unit]`, our walk
/// then the packet's own element and orientation (the by-half).
/// 640 triangles on 320 cells, every one twice. We lay 752 cells (the box
/// and past it), 304 of them the card's: the 16 apart are element 1's four
/// plates our walk does not lay (4653b). The tile page's texcoord is
/// whole; the piece's parts on 354 of 912 corners on our walk, the tiles
/// whose element or orientation the fractal picks apart (4199a). With the
/// packet's own: 318 of the 320 cells (the two apart are (12, 156), the
/// piece the buffer holds no box of, `RUN1814B_PLATE_QUADS`), the piece's
/// texcoord 948 of 954.
const RUN1833_DRAW_81: [[usize; 10]; 2] = [
    [640, 320, 320, 752, 304, 304, 912, 912, 558, 912],
    [640, 320, 320, 768, 318, 318, 954, 954, 948, 954],
];

/// **The plates' mesh is draw 81's** (item 4665, §6.2): run1833's card
/// keeps the plates' stream (`RON_CARD_STREAMS`): 842 vertices at stride
/// 52, position, normal, UV0 (the piece's), UV1 (the tile page's), and
/// 1,920 indices, each piece's cells drawn twice. Our mesh, cell by cell
/// at the same corners. Made to fail: [`crate::world::PLATE_LAYS`] at 1.
#[test]
fn the_plates_mesh_is_draw_81_s() {
    let path = std::env::var_os("RON_TRACE_RUN1833")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(|h| PathBuf::from(h).join("ron-golden/run1833/map-14/rontrace.log"))
        });
    let Some(bytes) = path.as_ref().and_then(|p| std::fs::read(p).ok()) else {
        println!("skipping: no run1833 trace at {path:?}");
        return;
    };
    let Some(mut p) = crate::farms::tests::probe() else {
        return;
    };
    let draws = rondata::trace::card::draws(&bytes);
    let d = draws
        .iter()
        .find(|d| d.frame == 3279 && d.index == 81)
        .expect("draw 81 after 3279");
    assert_eq!(
        (d.names[0].as_str(), d.drawn.stride, d.drawn.vertex_count()),
        ("./terrain art/roads/early_road_pieces.tga", 52, 842),
        "the plates' draw"
    );
    let v: Vec<[f32; 10]> = (0..842)
        .map(|k| d.drawn.floats::<10>(k).expect("a vertex"))
        .collect();
    let idx = d.drawn.index_list();
    let cell = |t: [[f32; 3]; 3]| {
        let mut k = t.map(|p| (p[0].round() as i32, p[1].round() as i32));
        k.sort();
        k
    };
    // Each cell's triangles, with every corner's (z, uv0, uv1).
    type Corner = (f32, [f32; 2], [f32; 2]);
    type Cell = [(i32, i32); 3];
    type Corners = Vec<((i32, i32), Corner)>;
    let mut card: BTreeMap<Cell, Vec<Corners>> = BTreeMap::new();
    for t in idx.as_chunks::<3>().0 {
        let c = |i: u32| v[i as usize];
        let pos = [0, 1, 2].map(|j| {
            let a = c(t[j]);
            [a[0], a[1], a[2]]
        });
        let corners = (0..3)
            .map(|j| {
                let a = c(t[j]);
                (
                    (a[0].round() as i32, a[1].round() as i32),
                    (a[2], [a[6], a[7]], [a[8], a[9]]),
                )
            })
            .collect();
        card.entry(cell(pos)).or_default().push(corners);
    }
    let twice = card.values().filter(|ts| ts.len() == 2).count();
    let mut rows = [[0; 10]; 2];
    let picks = packet("1814b").map(|pk| {
        pk.records
            .iter()
            .filter(|(_, r)| r.refs != 0 && r.rot <= 7 && r.elem >= 0)
            .map(|(&t, r)| (t, (r.elem as usize, r.rot)))
            .collect::<BTreeMap<_, _>>()
    });
    for (k, pick) in [None, picks].into_iter().enumerate() {
        p.world.roads = true;
        p.world.early_roads = true;
        p.world.road_pick = pick;
        let m = p.world.plate_mesh().expect("the plates are laid");
        let mut ours: BTreeMap<Cell, (usize, Corners)> = BTreeMap::new();
        for (j, t) in m.positions.as_chunks::<3>().0.iter().enumerate() {
            let corners = (0..3)
                .map(|q| {
                    let i = 3 * j + q;
                    let a = m.positions[i];
                    (
                        (a[0].round() as i32, a[1].round() as i32),
                        (a[2], m.alpha_uvs[i], m.uvs[i]),
                    )
                })
                .collect();
            let e = ours.entry(cell([t[0], t[1], t[2]])).or_insert((0, corners));
            e.0 += 1;
        }
        let (mut shared, mut corners, mut page, mut piece, mut high) = (0, 0, 0, 0, 0);
        let mut lays = 0;
        for (key, ts) in &card {
            let Some((n, mine)) = ours.get(key) else {
                continue;
            };
            shared += 1;
            lays += usize::from(*n == ts.len());
            for (at, (z, uv0, uv1)) in &ts[0] {
                let (_, (oz, ouv0, ouv1)) = mine.iter().find(|(a, _)| a == at).expect("a corner");
                corners += 1;
                let same = |a: &[f32; 2], b: &[f32; 2]| {
                    (a[0] - b[0]).abs() < 1e-5 && (a[1] - b[1]).abs() < 1e-5
                };
                page += usize::from(same(uv1, ouv1));
                piece += usize::from(same(uv0, ouv0));
                high += usize::from((z - oz).abs() <= 1.0);
            }
        }
        rows[k] = [
            idx.len() / 3,
            card.len(),
            twice,
            ours.len(),
            shared,
            lays,
            corners,
            page,
            piece,
            high,
        ];
        println!("run1833 draw 81 against our plates ({k}): {:?}", rows[k]);
    }
    p.world.road_pick = None;
    assert_eq!(rows, RUN1833_DRAW_81);
}
