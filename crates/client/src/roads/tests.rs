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
}

/// `tools/render/road_records.py` on a packet, the file at `$RON_ROADS_<RUN>`
/// or the item's lab directory. Nothing it holds enters the repository.
fn packet(run: &str) -> Option<Packet> {
    let var = format!("RON_ROADS_{}", run.to_uppercase());
    let path = std::env::var_os(&var).map(PathBuf::from).or_else(|| {
        std::env::var_os("HOME").map(|h| {
            PathBuf::from(h).join(format!(
                "ron-data/lab-experiments/2026-11-13-item-4199/roads-{run}.txt"
            ))
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
    let laid: BTreeMap<(i32, i32), Laid> = p
        .world
        .road_pieces()
        .into_iter()
        .map(|l| (l.tile, l))
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
/// tiles east, back to the before counts.
const RUN1559B_3279_RING: [[(usize, usize); 2]; 3] = [
    [(1, 24), (20, 24)],
    [(21, 24), (19, 24)],
    [(1, 24), (20, 24)],
];
/// **`RUN1570_RING`** (item 4199), the same on run1570's photograph: hue
/// **0 → 22** of 24, luma 14 → 14. The two that part on hue: (18, 163),
/// (86, 41, 33), the name plate; (19, 163), the T.
const RUN1570_RING: [[(usize, usize); 2]; 3] = [
    [(0, 24), (14, 24)],
    [(22, 24), (14, 24)],
    [(0, 24), (14, 24)],
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
