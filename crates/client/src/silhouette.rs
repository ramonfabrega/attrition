//! Candidate C's first row (`docs/RENDERER.md` §5.3, item 4038): the
//! scene's silhouettes on run1559b's frame 3294 against that take's own
//! photograph of it, at the photograph's camera.
//!
//! The photograph is kept outside the repository, converted once to a
//! `.tga` so `render::tga` reads it with no new dependency
//! (`sips -s format tga`): `$RON_PHOTO`, else
//! `~/ron-data/lab-experiments/2026-11-09-item-4038/run1559b-3294.tga`.

use crate::world::{Drawing, SpriteDrawing, TREE_WHO, TreePixels, World};
use render::silhouette::{
    Footprint, Object, Photo, READING, Reader, Rect, Stamp, footprints_read, rasterize_cut,
};
#[cfg(test)]
use render::silhouette::{Interface, Neighbour, Trace, footprints_traced};
use std::path::PathBuf;

/// **The photograph's interface**, by eye on run1559b's 1920×1080 still:
/// the panels, the message log, the city's label, the selected figure's
/// portrait and the beam over the cavalry — everything drawn over the
/// scene that is not the scene. An object whose window or ring meets one
/// is not scored.
pub const INTERFACE: [Rect; 11] = [
    Rect::new(0, 0, 150, 148),       // resources
    Rect::new(0, 160, 522, 330),     // the message log
    Rect::new(705, 0, 1215, 70),     // the age banner and the hint
    Rect::new(1695, 0, 1920, 30),    // population and the top-right buttons
    Rect::new(1720, 40, 1920, 122),  // the clock and "Armageddon"
    Rect::new(1840, 130, 1915, 270), // the portraits
    Rect::new(1790, 830, 1920, 888), // the military count
    Rect::new(0, 888, 1920, 1080),   // the bottom bar and the minimap
    Rect::new(1080, 578, 1176, 610), // the city's label
    Rect::new(1238, 688, 1310, 750), // the selected figure's portrait
    Rect::new(1290, 465, 1350, 580), // the beam over the cavalry
];

/// **Bare ground**, by eye on the same still and chosen with the second
/// reader's constants, before any count of it (item 4071): grass with no
/// object on it, below the tanks and on the open slope to the west. A
/// reader of the photograph must read each empty.
///
/// The second window was first chosen at half scale as (400, 500)..(560,
/// 600) and holds a tree in its lowest third (y ≥ 557); it was moved up the
/// slope after the first reader's count of it was read — a fixture
/// corrected because it was not bare, recorded in item 4071's journal.
#[cfg(test)]
pub const BARE: [Rect; 2] = [
    Rect::new(1250, 790, 1400, 870), // between the trees and the tanks
    Rect::new(460, 420, 620, 520),   // the open slope, below the cliff
];

/// **The photograph's objects traced by hand** (item 4071), in its own
/// pixels, on 4× crops with a 10-pixel grid, before any count of the mask
/// reader was read: the three tanks, and four figures the colour reader
/// passes, to hold the trace against it. Gun barrels, antennae and swords
/// are left out; each vertex is good to a pixel or two.
///
/// **The two buildings** (item 4077), traced the same way on 2× and 4×
/// crops of the photograph alone, before the reader that counts with them
/// was written: the capital 0/2000's five buildings, the dark tower and
/// the fountain on its sandy terrace, smoke and fire left in where they
/// lie over a wall and out where they lie over grass, the paved plaza
/// left out as flat ground ([`PLAZA`] is the other reading); and the
/// woodcutter 0/2001's tower, boom and two log piles, its shadow left out
/// and notched round the figure standing on the logs at (1652..1662,
/// 372..405). Each vertex is good to two or three pixels, the boom's to
/// four.
///
/// **The last tank, 1/27** (item 4079), traced the same way on 4× and 6×
/// crops of the photograph alone, by the same rule and before any count
/// with it: the olive hull from its nose at (1193, 669) to its right flank
/// at x 1296, the turret's top at y 649, the bottom edge where the
/// portrait's frame cuts it at y 692; the barrel left out, and notched
/// round the two figures the photograph shows in front of the hull, the
/// rifleman at (1202..1221, 670..700) and the kneeling figure at
/// (1269..1291, 658..690), as the woodcutter's is round the figure on its
/// logs. Each vertex is good to two or three pixels. [`TANK_1_27_WHOLE`]
/// is the other reading, the hull un-notched.
#[cfg(test)]
pub const TRACED: [(&str, &[(i32, i32)]); 10] = [
    (
        "1/40",
        &[(1129, 641), (1168, 641), (1173, 709), (1132, 709)],
    ),
    (
        "1/15",
        &[
            (1493, 756),
            (1520, 751),
            (1534, 751),
            (1542, 770),
            (1555, 787),
            (1562, 817),
            (1555, 821),
            (1532, 827),
            (1522, 829),
            (1510, 805),
            (1495, 770),
        ],
    ),
    (
        "1/16",
        &[(1596, 774), (1637, 773), (1646, 844), (1606, 844)],
    ),
    (
        "0/5",
        &[
            (1125, 282),
            (1131, 282),
            (1132, 290),
            (1135, 300),
            (1134, 316),
            (1131, 321),
            (1126, 321),
            (1124, 307),
            (1124, 290),
        ],
    ),
    (
        "1/21",
        &[
            (1252, 565),
            (1258, 565),
            (1259, 572),
            (1262, 577),
            (1262, 600),
            (1260, 605),
            (1256, 605),
            (1255, 601),
            (1253, 610),
            (1246, 610),
            (1250, 600),
            (1250, 587),
            (1241, 586),
            (1240, 582),
            (1250, 580),
            (1251, 572),
        ],
    ),
    (
        "0/0",
        &[
            (1487, 581),
            (1495, 582),
            (1498, 590),
            (1497, 607),
            (1494, 617),
            (1493, 625),
            (1483, 623),
            (1484, 617),
            (1482, 607),
            (1482, 590),
        ],
    ),
    (
        "1/23",
        &[
            (1518, 590),
            (1525, 590),
            (1527, 597),
            (1528, 605),
            (1527, 615),
            (1525, 620),
            (1525, 634),
            (1519, 634),
            (1519, 620),
            (1517, 613),
            (1516, 601),
        ],
    ),
    (
        "0/2000",
        &[
            (952, 460),
            (977, 442),
            (980, 412),
            (1015, 400),
            (1047, 410),
            (1050, 435),
            (1070, 435),
            (1071, 400),
            (1090, 394),
            (1102, 395),
            (1105, 352),
            (1140, 342),
            (1171, 360),
            (1167, 415),
            (1175, 425),
            (1220, 417),
            (1225, 435),
            (1240, 432),
            (1270, 442),
            (1272, 510),
            (1240, 520),
            (1205, 512),
            (1175, 512),
            (1170, 535),
            (1171, 555),
            (1155, 561),
            (1140, 557),
            (1135, 550),
            (1105, 550),
            (1070, 550),
            (1052, 542),
            (1052, 525),
            (1010, 525),
            (985, 522),
            (960, 510),
            (955, 480),
        ],
    ),
    (
        "0/2001",
        &[
            (1636, 322),
            (1648, 320),
            (1650, 343),
            (1656, 346),
            (1655, 360),
            (1657, 372),
            (1651, 372),
            (1651, 405),
            (1662, 405),
            (1662, 394),
            (1686, 393),
            (1688, 400),
            (1685, 418),
            (1670, 432),
            (1660, 436),
            (1644, 438),
            (1641, 451),
            (1607, 452),
            (1590, 447),
            (1586, 436),
            (1597, 426),
            (1615, 421),
            (1603, 365),
            (1603, 360),
            (1608, 361),
            (1622, 360),
            (1620, 344),
            (1632, 340),
        ],
    ),
    (
        "1/27",
        &[
            (1193, 669),
            (1194, 665),
            (1203, 662),
            (1218, 659),
            (1229, 657),
            (1233, 654),
            (1238, 651),
            (1248, 649),
            (1257, 649),
            (1272, 652),
            (1285, 657),
            (1292, 662),
            (1295, 668),
            (1296, 678),
            (1295, 687),
            (1291, 689),
            (1291, 672),
            (1286, 664),
            (1279, 658),
            (1272, 658),
            (1269, 664),
            (1269, 691),
            (1242, 692),
            (1227, 690),
            (1221, 690),
            (1221, 676),
            (1214, 670),
            (1206, 670),
            (1202, 676),
            (1202, 685),
            (1195, 678),
        ],
    ),
];

/// **Tank 1/27's hull, un-notched** (item 4079): [`TRACED`]'s 1/27 with
/// the two figures in front of it counted as the tank's, traced with it
/// and pinned beside it before either count; the count with it is
/// reported beside the instrument's.
#[cfg(test)]
pub const TANK_1_27_WHOLE: &[(i32, i32)] = &[
    (1193, 669),
    (1194, 665),
    (1203, 662),
    (1218, 659),
    (1229, 657),
    (1233, 654),
    (1238, 651),
    (1248, 649),
    (1257, 649),
    (1272, 652),
    (1285, 657),
    (1292, 662),
    (1295, 668),
    (1296, 678),
    (1295, 687),
    (1288, 690),
    (1278, 692),
    (1242, 692),
    (1227, 690),
    (1212, 688),
    (1202, 685),
    (1195, 678),
];

/// **The capital's paved plaza** (item 4077), the other reading of what
/// the photograph shows as the capital: the diamond its red border draws,
/// its top corner hidden behind the buildings and placed where the two
/// upper edges would meet. Traced with [`TRACED`] and pinned beside it;
/// the instrument leaves it out as flat ground, and the count with it is
/// reported beside the instrument's.
#[cfg(test)]
pub const PLAZA: &[(i32, i32)] = &[(945, 502), (1150, 395), (1322, 502), (1165, 583)];

/// **The frame the photograph shows** (item 4073): 3279, fifteen frames
/// before the packet's 3294. The photograph was taken paused on 3294
/// (`docs/RUNS.md`, run1559 and run1559b), and the packet agrees with the
/// dump there: the drawn point is `GuyData`'s own, blended from `last_*` by
/// `graphic_percent`, and 1/15's `last_*` is its point. What stands still
/// sits on the photograph at any of these frames; what moves sits only at
/// 3279 ([`PHOTO_ANCHORS`]). Why the photograph trails the paused frame is
/// not established (parked 4073a).
#[cfg(test)]
pub const PHOTO_FRAME: i64 = 3279;

/// **The movers' anchors on the photograph**, in its pixels (item 4073):
/// the two moving tanks' points, each [`TRACED`] polygon's pixel centroid
/// less the 14 px our tank's centroid stands above its anchor, and 1/35's
/// feet, by eye on a 2× crop. They are the frame's clock: the frame
/// between 3270 and 3294 whose movers project nearest them is
/// [`PHOTO_FRAME`].
#[cfg(test)]
pub const PHOTO_ANCHORS: [((u8, i16), (i32, i32)); 3] = [
    ((1, 15), (1527, 802)),
    ((1, 16), (1622, 823)),
    ((1, 35), (1440, 827)),
];

/// [`TRACED`]'s polygons, as [`Reader::Mask`] takes them.
#[cfg(test)]
pub const TRACED_POLYGONS: [&[(i32, i32)]; 7] = [
    TRACED[0].1,
    TRACED[1].1,
    TRACED[2].1,
    TRACED[3].1,
    TRACED[4].1,
    TRACED[5].1,
    TRACED[6].1,
];
// The buildings' traces, [`TRACED`]'s last two, are read by item 4077's
// reader by their names and enter no [`Reader::Mask`] row.

/// The photograph's path.
pub fn photo_path() -> PathBuf {
    std::env::var_os("RON_PHOTO")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME").unwrap_or_default();
            PathBuf::from(home)
                .join("ron-data/lab-experiments/2026-11-09-item-4038/run1559b-3294.tga")
        })
}

/// The drawn objects as the rasterizer takes them: key `i + 1` for the
/// `i`th drawing.
pub fn objects(drawings: &[Drawing]) -> Vec<Object> {
    drawings
        .iter()
        .enumerate()
        .map(|(i, d)| Object {
            key: i as u32 + 1,
            tris: d.tris.clone(),
        })
        .collect()
}

/// The sprites as the rasterizer takes them: key `first + j` for the `j`th.
pub fn stamps(sprites: &[SpriteDrawing], first: u32) -> Vec<Stamp<'_>> {
    sprites
        .iter()
        .enumerate()
        .map(|(j, s)| Stamp {
            key: first + j as u32,
            at: s.at,
            width: s.image.image.width,
            height: s.image.image.height,
            covered: &s.image.covered,
            depth: s.depth,
        })
        .collect()
}

/// Who each key is: `(owner, object, piece, building)`, key `i + 1` at `i`.
pub type Who = (u8, i16, i32, bool);

/// The frame's drawings at `camera` — the meshes, then the buildings drawn
/// as sprites (item 4076) — and their key buffer at `size`.
pub fn frame_keys(
    world: &mut World,
    camera: &render::Camera,
    size: (usize, usize),
) -> (render::silhouette::Keys, Vec<Who>) {
    let (figures, buildings, list) = world.draw_list(
        camera,
        &crate::pinned_screen(),
        (size.0 as i32, size.1 as i32),
    );
    let drawings: Vec<Drawing> = world.objects(&figures, &buildings, &list);
    let sprites = world.sprite_drawings(&buildings, &list, camera.zoom_distance);
    let first = drawings.len() as u32 + 1;
    // The map's trees (item 4080) after the sprites, keyed from `trees`.
    let trees = world.tree_drawings();
    let first_tree = first + sprites.len() as u32;
    let mut meshes = objects(&drawings);
    meshes.extend(trees.iter().enumerate().map(|(j, d)| Object {
        key: first_tree + j as u32,
        tris: d.tris.clone(),
    }));
    let cuts = world.tree_cuts();
    let cut: Vec<Option<&render::silhouette::Cut<'_>>> = match world.leaves {
        crate::world::Leaves::Solid => Vec::new(),
        crate::world::Leaves::Alpha => std::iter::repeat_n(None, drawings.len())
            .chain(cuts.iter().map(Option::as_ref))
            .collect(),
    };
    let mut keys = rasterize_cut(&meshes, &cut, &stamps(&sprites, first), camera, size).keys;
    if world.silhouettes == crate::world::Silhouettes::Drawn {
        // The guys pass after `flush_trees` (item 4085): a figure's
        // `render_states[1]` draw passes where it is behind, writing no
        // depth, so over a tree or a sprite its pixel is its own. Among the
        // meshes the scene's depth order already stands.
        let figures: Vec<Object> = objects(&drawings)
            .into_iter()
            .filter(|o| !drawings[o.key as usize - 1].building)
            .collect();
        let alone = rasterize_cut(&figures, &[], &[], camera, size).keys;
        for (k, f) in keys.keys.iter_mut().zip(&alone.keys) {
            if *f != 0 && *k >= first {
                *k = *f;
            }
        }
    }
    if world.tree_pixels == TreePixels::Ground {
        for k in keys.keys.iter_mut().filter(|k| **k >= first_tree) {
            *k = 0;
        }
    }
    let who = drawings
        .iter()
        .map(|d| (d.who, d.o, d.piece, d.building))
        .chain(sprites.iter().map(|s| (s.who, s.o, s.piece, true)))
        .chain(trees.iter().map(|d| (d.who, d.o, d.piece, false)))
        .collect();
    (keys, who)
}

/// Every scored object's footprints on the frame at `camera`, beside its
/// drawing.
pub fn score_frame(
    world: &mut World,
    camera: &render::Camera,
    photo: &render::tga::Image,
    reader: Reader,
) -> Vec<(Footprint, (u8, i16, i32, bool))> {
    let size = (photo.width, photo.height);
    let (keys, drawings) = frame_keys(world, camera, size);
    let photo = Photo {
        width: photo.width,
        height: photo.height,
        rgba: &photo.rgba,
    };
    footprints_read(&keys, &photo, &INTERFACE, &READING, reader)
        .into_iter()
        .map(|f| {
            let d = drawings[f.key as usize - 1];
            (f, d)
        })
        .filter(|(_, d)| d.0 != TREE_WHO)
        .collect()
}

/// Every scored object's footprints on the frame at `camera`, read by the
/// traced reader (item 4077; `render::silhouette::footprints_traced`):
/// each of `traced`'s regions is the object its name, `owner/object`,
/// names, and a region whose object is not drawn is every object's
/// neighbour.
#[cfg(test)]
pub fn score_frame_traced(
    world: &mut World,
    camera: &render::Camera,
    photo: &render::tga::Image,
    traced: &[(&str, &[(i32, i32)])],
    neighbour: Neighbour,
    iface: Interface,
) -> Vec<(Footprint, Who)> {
    let size = (photo.width, photo.height);
    let (keys, drawings) = frame_keys(world, camera, size);
    let photo = Photo {
        width: photo.width,
        height: photo.height,
        rgba: &photo.rgba,
    };
    let traces: Vec<Trace<'_>> = traced
        .iter()
        .map(|&(name, poly)| Trace {
            key: drawings
                .iter()
                .position(|&(who, o, _, _)| format!("{who}/{o}") == name)
                .map_or(u32::MAX, |i| i as u32 + 1),
            poly,
        })
        .collect();
    footprints_traced(
        &keys, &photo, &INTERFACE, &READING, &traces, neighbour, iface,
    )
    .into_iter()
    .map(|f| (f, drawings[f.key as usize - 1]))
    .filter(|(_, d)| d.0 != TREE_WHO)
    .collect()
}

/// The photograph with each scored window drawn over it: our footprint's
/// pixels red, the original's blue, both magenta, the window's edge white,
/// and the interface darkened — the picture a reader checks the numbers by.
pub fn explain(
    world: &mut World,
    camera: &render::Camera,
    photo: &render::tga::Image,
    scored: &[Footprint],
) -> Vec<u8> {
    use render::silhouette::ground;
    let size = (photo.width, photo.height);
    let (keys, _) = frame_keys(world, camera, size);
    let p = Photo {
        width: photo.width,
        height: photo.height,
        rgba: &photo.rgba,
    };
    let mut out = photo.rgba.clone();
    let at = |x: i32, y: i32| (y as usize * size.0 + x as usize) * 4;
    for r in &INTERFACE {
        for y in r.y0.max(0)..r.y1.min(size.1 as i32) {
            for x in r.x0.max(0)..r.x1.min(size.0 as i32) {
                let i = at(x, y);
                for c in 0..3 {
                    out[i + c] /= 3;
                }
            }
        }
    }
    for f in scored {
        let w = f.window;
        let outer = Rect::new(
            w.x0 - READING.ring,
            w.y0 - READING.ring,
            w.x1 + READING.ring,
            w.y1 + READING.ring,
        );
        let g = ground(&keys, &p, w, outer, &READING);
        for y in w.y0..w.y1 {
            for x in w.x0..w.x1 {
                let ours = keys.at(x as usize, y as usize) != 0;
                let theirs = !g.holds(&p, x, y);
                let i = at(x, y);
                let edge = x == w.x0 || y == w.y0 || x == w.x1 - 1 || y == w.y1 - 1;
                let c = match (edge, ours, theirs) {
                    (true, _, _) => [255, 255, 255],
                    (_, true, true) => [255, 0, 255],
                    (_, true, false) => [255, 0, 0],
                    (_, false, true) => [0, 80, 255],
                    _ => continue,
                };
                out[i..i + 3].copy_from_slice(&c);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use render::silhouette::score;

    /// **Candidate C's first row**: on frame 3294 at run1559b's camera, the
    /// objects whose footprint overlaps the photograph's by half or more,
    /// of the objects scored. Made to fail on run1441's camera of the same
    /// frame, which the photograph was not taken at.
    /// One scored row and its drawing, as [`score_frame`] gives it.
    type Scored = (Footprint, (u8, i16, i32, bool));

    #[test]
    fn silhouette_frame_3294_against_run1559b_s_photograph() {
        let install = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let path = photo_path();
        let Ok(bytes) = std::fs::read(&path) else {
            println!("skipping: no photograph at {}", path.display());
            return;
        };
        let photo = render::tga::decode(&bytes).expect("the photograph decodes");
        assert_eq!((photo.width, photo.height), (1920, 1080));
        let mut world = match World::open(&install, &crate::world::gamelog_dir()) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        let camera = crate::pinned_camera();
        let mut elsewhere = camera;
        (elsewhere.loc_x, elsewhere.loc_y) = (3168.0, 29744.68);
        // The default drawing: every figure posed, at the original's facing.
        assert!(world.pose, "the client draws posed by default");
        // These rows are the treeless draw's, kept by name (item 4080).
        world.trees = false;

        // The photograph's frame (item 4073): the frame whose movers project
        // nearest the photograph's, walked over 3270..=3294, and candidate C
        // scored on it.
        let (m, screen) = (camera.to_screen_matrix(), crate::pinned_screen());
        let mut misses = Vec::new();
        let mut at_photo = None;
        for n in 3270..=3294 {
            world.run_to(n);
            let (figures, _, _) = world.draw_list(&camera, &screen, (1920, 1080));
            let mut miss = 0.0f64;
            let mut line = format!("frame {n}:");
            for ((who, o), (px, py)) in PHOTO_ANCHORS {
                let f = figures
                    .iter()
                    .find(|f| (f.who, f.o) == (who, o))
                    .expect("each mover is on the map");
                let q =
                    render::camera::world_to_screen(&m, &screen, f.x, f.y, f.z, Some((1920, 1080)));
                let d = f64::from((q.x - px).pow(2) + (q.y - py).pow(2)).sqrt();
                miss += d;
                line += &format!(
                    " {who}/{o} ({}, {}, {}) -> ({}, {}) {d:.1};",
                    f.x, f.y, f.z, q.x, q.y
                );
            }
            println!("{line} total {miss:.1}");
            misses.push((n, miss));
            if n != PHOTO_FRAME {
                continue;
            }
            let mixed = |colour: &[Scored], masked: &[Scored]| {
                let rows: Vec<Footprint> = colour
                    .iter()
                    .zip(masked)
                    .map(|((c, (_, _, piece, _)), (k, _))| if *piece == 196 { *k } else { *c })
                    .collect();
                score(&rows, &READING)
            };
            let print_rows = |draw: &str, colour: &[Scored], masked: &[Scored]| {
                for ((c, (who, o, piece, building)), (k, _)) in colour.iter().zip(masked) {
                    println!(
                        "{PHOTO_FRAME} {draw}: {who}/{o} piece {piece}{} window ({}, {})..({}, {}) ours {} theirs colour {} mask {} overlap colour {:.3} mask {:.3}",
                        if *building { " building" } else { "" },
                        c.window.x0,
                        c.window.y0,
                        c.window.x1,
                        c.window.y1,
                        c.ours,
                        c.theirs,
                        k.theirs,
                        c.overlap(),
                        k.overlap()
                    );
                }
            };
            // The buildings as the original draws them, their sprites (item
            // 4076): the reader as it stands cannot score them
            // ([`render::silhouette::RUN1559B_3279_SPRITES`]).
            assert!(world.sprites, "the client draws the sprites by default");
            let colour = score_frame(&mut world, &camera, &photo, Reader::Colour);
            let masked = score_frame(&mut world, &camera, &photo, Reader::Mask(&TRACED_POLYGONS));
            print_rows("sprites", &colour, &masked);
            let posed: Vec<Footprint> = colour.iter().map(|(f, _)| *f).collect();
            let sprites = (score(&posed, &READING), mixed(&colour, &masked));
            let off = score_frame(&mut world, &elsewhere, &photo, Reader::Colour);
            let off_masked = score_frame(
                &mut world,
                &elsewhere,
                &photo,
                Reader::Mask(&TRACED_POLYGONS),
            );
            let off_colour: Vec<Footprint> = off.iter().map(|(f, _)| *f).collect();
            let sprites_off = (score(&off_colour, &READING), mixed(&off, &off_masked));
            println!(
                "frame {PHOTO_FRAME}: the sprites, posed {:?}, the tanks by the mask {:?}; at run1441's camera {:?} and {:?}",
                sprites.0, sprites.1, sprites_off.0, sprites_off.1
            );
            // The pinned rows: each building drawn as its `zbuffer` model.
            world.sprites = false;
            let colour = score_frame(&mut world, &camera, &photo, Reader::Colour);
            let masked = score_frame(&mut world, &camera, &photo, Reader::Mask(&TRACED_POLYGONS));
            print_rows("zbuffer", &colour, &masked);
            let posed: Vec<Footprint> = colour.iter().map(|(f, _)| *f).collect();
            let posed = score(&posed, &READING);
            let by_mask = mixed(&colour, &masked);
            world.pose = false;
            let rest: Vec<Footprint> = score_frame(&mut world, &camera, &photo, Reader::Colour)
                .iter()
                .map(|(f, _)| *f)
                .collect();
            let rest = score(&rest, &READING);
            world.pose = true;
            let off = score_frame(&mut world, &elsewhere, &photo, Reader::Colour);
            let off_masked = score_frame(
                &mut world,
                &elsewhere,
                &photo,
                Reader::Mask(&TRACED_POLYGONS),
            );
            let off_colour: Vec<Footprint> = off.iter().map(|(f, _)| *f).collect();
            let off = (score(&off_colour, &READING), mixed(&off, &off_masked));
            world.sprites = true;
            println!(
                "frame {PHOTO_FRAME}: posed {posed:?}, the tanks by the mask {by_mask:?}, at rest {rest:?}; at run1441's camera {:?} and {:?}",
                off.0, off.1
            );
            at_photo = Some((posed, by_mask, rest, off, sprites, sprites_off));
        }
        let nearest = misses
            .iter()
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .expect("frames walked");
        assert_eq!(
            nearest.0, PHOTO_FRAME,
            "the movers sit on the photograph at its frame"
        );
        let (posed, by_mask, rest, off, sprites, sprites_off) =
            at_photo.expect("the photograph's frame was walked");
        assert_eq!(
            sprites,
            (
                render::silhouette::RUN1559B_3279_SPRITES_POSED,
                render::silhouette::RUN1559B_3279_SPRITES
            )
        );
        assert_eq!(
            sprites_off,
            (
                render::silhouette::RUN1441_CAMERA_3279,
                render::silhouette::RUN1441_CAMERA_3279_MASKED
            )
        );
        assert_eq!(posed, render::silhouette::RUN1559B_3279_POSED);
        assert_eq!(by_mask, render::silhouette::RUN1559B_3279_MASKED);
        assert_eq!(rest, render::silhouette::RUN1559B_3279);
        assert_eq!(
            off,
            (
                render::silhouette::RUN1441_CAMERA_3279,
                render::silhouette::RUN1441_CAMERA_3279_MASKED
            )
        );
        assert!(
            off.1.0 < by_mask.0,
            "the photograph's camera scores above another"
        );
        assert!(
            by_mask.0 > render::silhouette::RUN1559B_3294_MASKED.0,
            "the photograph's frame scores above the packet's"
        );

        // The packet's frame, as items 4038 to 4071 scored it: the buildings
        // as their `zbuffer` models.
        world.sprites = false;
        world.run_to(3294);
        let rows = score_frame(&mut world, &camera, &photo, Reader::Colour);
        for (f, (who, o, piece, building)) in &rows {
            println!(
                "{who}/{o} piece {piece}{} window ({}, {})..({}, {}) ours {} theirs {} both {} overlap {:.3}",
                if *building { " building" } else { "" },
                f.window.x0,
                f.window.y0,
                f.window.x1,
                f.window.y1,
                f.ours,
                f.theirs,
                f.both,
                f.overlap()
            );
        }
        let footprints: Vec<Footprint> = rows.iter().map(|(f, _)| *f).collect();
        let got = score(&footprints, &READING);
        println!("posed: {} of {}", got.0, got.1);
        assert_eq!(
            got,
            render::silhouette::RUN1559B_3294_POSED,
            "candidate C's row"
        );

        world.pose = false;
        let rows = score_frame(&mut world, &camera, &photo, Reader::Colour);
        let footprints: Vec<Footprint> = rows.iter().map(|(f, _)| *f).collect();
        let rest = score(&footprints, &READING);
        println!("at rest: {} of {}", rest.0, rest.1);
        assert_eq!(rest, render::silhouette::RUN1559B_3294);
        world.pose = true;

        let rows = score_frame(&mut world, &elsewhere, &photo, Reader::Colour);
        let footprints: Vec<Footprint> = rows.iter().map(|(f, _)| *f).collect();
        let off = score(&footprints, &READING);
        println!("at run1441's camera: {} of {}", off.0, off.1);
        assert_eq!(off, render::silhouette::RUN1441_CAMERA_3294);
        assert!(
            off.0 < got.0,
            "the photograph's camera scores above another"
        );

        // The second reader (item 4071): the same windows, the photograph's
        // side read by its robust ground, every row beside the colour
        // reader's.
        let robust = Reader::Robust(render::silhouette::ROBUST);
        let colour_rows = score_frame(&mut world, &camera, &photo, Reader::Colour);
        let robust_rows = score_frame(&mut world, &camera, &photo, robust);
        assert_eq!(colour_rows.len(), robust_rows.len());
        for ((c, (who, o, piece, _)), (r, _)) in colour_rows.iter().zip(&robust_rows) {
            assert_eq!((c.key, c.window, c.ours), (r.key, r.window, r.ours));
            println!(
                "{who}/{o} piece {piece} ours {} theirs colour {} robust {} (Δ {:+}) overlap colour {:.3} robust {:.3}{}",
                c.ours,
                c.theirs,
                r.theirs,
                i64::from(r.theirs) - i64::from(c.theirs),
                c.overlap(),
                r.overlap(),
                match (
                    c.overlap() >= READING.threshold,
                    r.overlap() >= READING.threshold
                ) {
                    (true, false) => " — colour passes, robust does not",
                    (false, true) => " — robust passes, colour does not",
                    _ => "",
                }
            );
        }
        let footprints: Vec<Footprint> = robust_rows.iter().map(|(f, _)| *f).collect();
        let by_robust = score(&footprints, &READING);
        println!(
            "posed, the robust reader: {} of {}",
            by_robust.0, by_robust.1
        );
        let rows = score_frame(&mut world, &elsewhere, &photo, robust);
        let footprints: Vec<Footprint> = rows.iter().map(|(f, _)| *f).collect();
        let off_robust = score(&footprints, &READING);
        println!(
            "at run1441's camera, the robust reader: {} of {}",
            off_robust.0, off_robust.1
        );
        assert_eq!(by_robust, render::silhouette::ROBUST_3294);
        let p = Photo {
            width: photo.width,
            height: photo.height,
            rgba: &photo.rgba,
        };
        let empty = render::silhouette::Keys {
            width: photo.width,
            height: photo.height,
            keys: vec![0; photo.width * photo.height],
        };
        let mut bare = Vec::new();
        for w in BARE {
            let c = render::silhouette::read_window(&empty, &p, w, &READING, Reader::Colour);
            let r = render::silhouette::read_window(&empty, &p, w, &READING, robust);
            let (c, r) = (
                c.iter().filter(|t| **t).count(),
                r.iter().filter(|t| **t).count(),
            );
            println!(
                "bare ground {w:?}: colour {c}, robust {r} of {}",
                (w.x1 - w.x0) * (w.y1 - w.y0)
            );
            assert!(
                c < READING.min_pixels as usize,
                "the colour reader reads bare ground empty"
            );
            bare.push(r as u32);
        }
        // The first try is refused: it reads bare grass as objects.
        assert_eq!(bare, render::silhouette::ROBUST_BARE);
        assert!(bare.iter().all(|&r| r >= READING.min_pixels));

        // The second try (item 4071): the hand-traced mask. On the traced
        // figures, the colour reader's pixels against the trace's, pixel by
        // pixel; on the tanks, the trace's footprint beside ours.
        let mask = Reader::Mask(&TRACED_POLYGONS);
        let (keys, _) = frame_keys(&mut world, &camera, (photo.width, photo.height));
        let mask_rows = score_frame(&mut world, &camera, &photo, mask);
        let mut mixed = Vec::new();
        for ((c, (who, o, piece, _)), (m, _)) in colour_rows.iter().zip(&mask_rows) {
            let name = format!("{who}/{o}");
            // Item 4071's seven: the buildings' traces are item 4077's.
            let traced = TRACED[..TRACED_POLYGONS.len()]
                .iter()
                .any(|(n, _)| *n == name);
            let tank = *piece == 196;
            mixed.push(if tank { *m } else { *c });
            if !traced {
                continue;
            }
            let cw = render::silhouette::read_window(&keys, &p, c.window, &READING, Reader::Colour);
            let mw = render::silhouette::read_window(&keys, &p, c.window, &READING, mask);
            let both = cw.iter().zip(&mw).filter(|(a, b)| **a && **b).count();
            let either = cw.iter().zip(&mw).filter(|(a, b)| **a || **b).count();
            println!(
                "traced {name} piece {piece}: ours {} theirs colour {} mask {} (readers agree {both} of {either}, {:.3}) overlap colour {:.3} mask {:.3}",
                c.ours,
                c.theirs,
                m.theirs,
                both as f32 / either.max(1) as f32,
                c.overlap(),
                m.overlap()
            );
            if !tank {
                // Where colour works, the trace agrees with it: the same
                // verdict, and half or more of their pixels shared.
                assert!(c.overlap() >= READING.threshold && m.overlap() >= READING.threshold);
                assert!(
                    2 * both >= either,
                    "{name}: the readers agree on {both} of {either}"
                );
            }
        }
        let by_mask = score(&mixed, &READING);
        println!(
            "posed, the tanks read by the mask: {} of {}",
            by_mask.0, by_mask.1
        );
        assert_eq!(by_mask, render::silhouette::RUN1559B_3294_MASKED);
        let rows = score_frame(&mut world, &elsewhere, &photo, mask);
        let footprints: Vec<Footprint> = rows.iter().map(|(f, _)| *f).collect();
        let off_mask = score(&footprints, &READING);
        println!(
            "at run1441's camera, the mask: {} of {}",
            off_mask.0, off_mask.1
        );
        assert_eq!(off_mask, render::silhouette::RUN1441_CAMERA_3294_MASKED);
        assert!(
            off_mask.0 < by_mask.0,
            "the mask's camera scores above another"
        );
        for w in BARE {
            let m = render::silhouette::read_window(&empty, &p, w, &READING, mask);
            let n = m.iter().filter(|t| **t).count();
            println!("bare ground {w:?}: mask {n}");
            assert_eq!(n, 0, "the mask reads bare ground empty");
        }
    }

    /// **The traced reader** (item 4077; `docs/RENDERER.md` §5.3, "A traced
    /// region") on the photograph's frame, the client's default draw: the
    /// buildings as their sprites, every figure posed.
    #[test]
    fn the_traced_reader_scores_frame_3279_on_the_default_draw() {
        let install = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let path = photo_path();
        let Ok(bytes) = std::fs::read(&path) else {
            println!("skipping: no photograph at {}", path.display());
            return;
        };
        let photo = render::tga::decode(&bytes).expect("the photograph decodes");
        let mut world = match World::open(&install, &crate::world::gamelog_dir()) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        assert!(world.pose && world.sprites, "the client's default draw");
        let camera = crate::pinned_camera();
        let mut elsewhere = camera;
        (elsewhere.loc_x, elsewhere.loc_y) = (3168.0, 29744.68);
        world.run_to(PHOTO_FRAME);
        let colour = score_frame(&mut world, &camera, &photo, Reader::Colour);
        let row = |name: &str, rows: &[(Footprint, Who)]| {
            rows.iter()
                .find(|(_, (who, o, _, _))| format!("{who}/{o}") == name)
                .map(|(f, _)| *f)
        };
        let print = |label: &str, rows: &[(Footprint, Who)]| {
            for (f, (who, o, piece, building)) in rows {
                println!(
                    "{label}: {who}/{o} piece {piece}{} window ({}, {})..({}, {}) ours {} theirs {} both {} overlap {:.3}",
                    if *building { " building" } else { "" },
                    f.window.x0,
                    f.window.y0,
                    f.window.x1,
                    f.window.y1,
                    f.ours,
                    f.theirs,
                    f.both,
                    f.overlap()
                );
            }
            let fs: Vec<Footprint> = rows.iter().map(|(f, _)| *f).collect();
            let s = score(&fs, &READING);
            println!("{label}: {} of {}", s.0, s.1);
            s
        };
        let read =
            |world: &mut World, camera: &render::Camera, traced: &[(&str, &[(i32, i32)])], n, i| {
                score_frame_traced(world, camera, &photo, traced, n, i)
            };
        let rule = read(
            &mut world,
            &camera,
            &TRACED,
            Neighbour::Neither,
            Interface::Neither,
        );
        let got = print("traced", &rule);
        // The silhouettes' rejected reading, a figure hidden by what is
        // nearer (item 4085).
        world.silhouettes = crate::world::Silhouettes::None;
        let hidden_rows = read(
            &mut world,
            &camera,
            &TRACED,
            Neighbour::Neither,
            Interface::Neither,
        );
        let hidden = print("traced, no silhouettes", &hidden_rows);
        world.silhouettes = crate::world::Silhouettes::Drawn;
        // The trees' rejected reading, and the treeless draw (item 4080).
        world.tree_pixels = crate::world::TreePixels::Scene;
        let tree_scene_rows = read(
            &mut world,
            &camera,
            &TRACED,
            Neighbour::Neither,
            Interface::Neither,
        );
        let tree_scene = print("traced, a tree's pixel as scene", &tree_scene_rows);
        world.tree_pixels = crate::world::TreePixels::Ground;
        // The leaves' rejected reading, every triangle whole (item 4082).
        world.leaves = crate::world::Leaves::Solid;
        let solid_rows = read(
            &mut world,
            &camera,
            &TRACED,
            Neighbour::Neither,
            Interface::Neither,
        );
        let solid = print("traced, the leaves solid", &solid_rows);
        world.leaves = crate::world::Leaves::Alpha;
        world.trees = false;
        let treeless_rows = read(
            &mut world,
            &camera,
            &TRACED,
            Neighbour::Neither,
            Interface::Neither,
        );
        let treeless = print("traced, no trees", &treeless_rows);
        world.trees = true;
        for (g, (who, o, piece, _)) in &treeless_rows {
            if !rule
                .iter()
                .any(|(_, w)| (w.0, w.1, w.2) == (*who, *o, *piece))
            {
                println!(
                    "unscored with trees {who}/{o} piece {piece}: ours {} theirs {} both {} overlap {:.3}",
                    g.ours,
                    g.theirs,
                    g.both,
                    g.overlap()
                );
            }
        }
        for (f, (who, o, piece, _)) in &rule {
            let before = treeless_rows
                .iter()
                .find(|(_, w)| (w.0, w.1, w.2) == (*who, *o, *piece))
                .map(|(g, _)| (g.ours, g.theirs, g.both, g.overlap()));
            let now = (f.ours, f.theirs, f.both, f.overlap());
            if before.map(|b| (b.0, b.1, b.2)) != Some((now.0, now.1, now.2)) {
                println!("moved {who}/{o} piece {piece}: no trees {before:?} trees {now:?}");
            }
        }
        let scene = print(
            "traced, a neighbour's trace as scene",
            &read(
                &mut world,
                &camera,
                &TRACED,
                Neighbour::Scene,
                Interface::Neither,
            ),
        );
        let skip = print(
            "traced, the interface skipped",
            &read(
                &mut world,
                &camera,
                &TRACED,
                Neighbour::Neither,
                Interface::Skip,
            ),
        );
        let mut plazad: Vec<(&str, &[(i32, i32)])> = TRACED.to_vec();
        plazad.push(("0/2000", PLAZA));
        let plaza = print(
            "traced, the plaza the capital's",
            &read(
                &mut world,
                &camera,
                &plazad,
                Neighbour::Neither,
                Interface::Neither,
            ),
        );
        let mut wholed: Vec<(&str, &[(i32, i32)])> = TRACED.to_vec();
        for t in wholed.iter_mut() {
            if t.0 == "1/27" {
                t.1 = TANK_1_27_WHOLE;
            }
        }
        let whole = print(
            "traced, tank 1/27 un-notched",
            &read(
                &mut world,
                &camera,
                &wholed,
                Neighbour::Neither,
                Interface::Neither,
            ),
        );
        let off = print(
            "traced, run1441's camera",
            &read(
                &mut world,
                &elsewhere,
                &TRACED,
                Neighbour::Neither,
                Interface::Neither,
            ),
        );
        // The four figures by the woodcutter, before (the colour reader on
        // this draw) and after.
        for name in [
            "0/9", "0/1", "0/10", "0/8", "0/6", "0/2000", "0/2001", "1/27", "1/20", "1/32", "1/31",
            "1/36",
        ] {
            println!(
                "{name}: colour {:?} traced {:?}",
                row(name, &colour).map(|f| f.overlap()),
                row(name, &rule).map(|f| f.overlap())
            );
        }
        // Where colour and the trace both read a traced figure, their
        // verdicts agree.
        for (name, _) in TRACED {
            if let (Some(c), Some(r)) = (row(name, &colour), row(name, &rule)) {
                println!(
                    "{name}: colour {:.3} traced {:.3}",
                    c.overlap(),
                    r.overlap()
                );
                if c.overlap() >= READING.threshold {
                    assert!(
                        r.overlap() >= READING.threshold,
                        "{name}: the verdicts agree"
                    );
                }
            }
        }
        // Bare ground reads empty.
        let p = Photo {
            width: photo.width,
            height: photo.height,
            rgba: &photo.rgba,
        };
        let empty = render::silhouette::Keys {
            width: photo.width,
            height: photo.height,
            keys: vec![0; photo.width * photo.height],
        };
        let traces: Vec<Trace<'_>> = TRACED
            .iter()
            .map(|&(_, poly)| Trace {
                key: u32::MAX,
                poly,
            })
            .collect();
        for w in BARE {
            let f = render::silhouette::read_traced(
                &empty,
                &p,
                &INTERFACE,
                &READING,
                &traces,
                Neighbour::Neither,
                Interface::Neither,
                0,
                w,
            );
            println!("bare ground {w:?}: traced {}", f.theirs);
            assert!(f.theirs < READING.min_pixels, "bare ground reads empty");
        }
        // A building's trace shifted by its own width drops its row.
        let wood = TRACED
            .iter()
            .find(|(n, _)| *n == "0/2001")
            .expect("traced")
            .1;
        let width =
            wood.iter().map(|p| p.0).max().unwrap() - wood.iter().map(|p| p.0).min().unwrap();
        let shifted: Vec<(i32, i32)> = wood.iter().map(|&(x, y)| (x + width, y)).collect();
        let moved: Vec<(&str, &[(i32, i32)])> = TRACED
            .iter()
            .map(|&(n, poly)| {
                if n == "0/2001" {
                    (n, &shifted[..])
                } else {
                    (n, poly)
                }
            })
            .collect();
        let shifted_rows = read(
            &mut world,
            &camera,
            &moved,
            Neighbour::Neither,
            Interface::Neither,
        );
        println!(
            "0/2001 shifted {width} px: {:?}",
            row("0/2001", &shifted_rows).map(|f| f.overlap())
        );
        // The tank's trace shifted by its own width drops its row (item 4079).
        let tank = TRACED.iter().find(|(n, _)| *n == "1/27").expect("traced").1;
        let tank_width =
            tank.iter().map(|p| p.0).max().unwrap() - tank.iter().map(|p| p.0).min().unwrap();
        let tank_shifted: Vec<(i32, i32)> =
            tank.iter().map(|&(x, y)| (x + tank_width, y)).collect();
        let tank_moved: Vec<(&str, &[(i32, i32)])> = TRACED
            .iter()
            .map(|&(n, poly)| {
                if n == "1/27" {
                    (n, &tank_shifted[..])
                } else {
                    (n, poly)
                }
            })
            .collect();
        let tank_rows = read(
            &mut world,
            &camera,
            &tank_moved,
            Neighbour::Neither,
            Interface::Neither,
        );
        println!(
            "1/27 shifted {tank_width} px: {:?}",
            row("1/27", &tank_rows).map(|f| f.overlap())
        );
        use render::silhouette as s;
        assert_eq!(got, s::RUN1559B_3279_TRACED);
        assert_eq!(hidden, s::RUN1559B_3279_TRACED_NO_SILHOUETTES);
        assert_eq!(tree_scene, s::RUN1559B_3279_TRACED_TREES_SCENE);
        assert_eq!(solid, s::RUN1559B_3279_TRACED_LEAVES_SOLID);
        assert_eq!(treeless, s::RUN1559B_3279_TRACED_TREELESS);
        assert_eq!(scene, s::RUN1559B_3279_TRACED_SCENE);
        assert_eq!(skip, s::RUN1559B_3279_TRACED_SKIP);
        assert_eq!(plaza, s::RUN1559B_3279_TRACED_PLAZA);
        assert_eq!(whole, s::RUN1559B_3279_TRACED_WHOLE);
        assert_eq!(off, s::RUN1441_CAMERA_3279_TRACED);
        assert!(got.0 > s::RUN1559B_3279_SPRITES.0, "the row is raised");
        for name in ["0/2000", "0/2001"] {
            let f = row(name, &rule).expect("both buildings are scored");
            assert!(f.overlap() >= READING.threshold, "{name} passes");
        }
        let f = row("0/2001", &shifted_rows).expect("still scored");
        assert!(
            f.overlap() < READING.threshold,
            "a trace shifted by its own width drops its row"
        );
        for name in ["1/27", "1/36"] {
            let f = row(name, &rule).expect("scored");
            assert!(f.overlap() >= READING.threshold, "{name} passes");
        }
        let f = row("1/27", &tank_rows).expect("still scored");
        assert!(
            f.overlap() < READING.threshold,
            "the tank's trace shifted by its own width drops its row"
        );
    }

    /// **The woodcutter's right edge** (item 4079): on frame 3279 at
    /// run1559b's camera our sprite reaches x 1700 where the photograph's
    /// traced building ends nearer 1686. The sprite is drawn as the
    /// original places it: `woodcutter_6.tga`, 173 × 201, its alpha two-level
    /// (0 or 255) in every column past screen x 1688 and its last column
    /// opaque; at the anchor (1640, 438) less
    /// `(int)(83 · s)`, `(int)(178 · s)`, `s` = 13905 / 20858, its left is
    /// 1585 and its right 1585 + 173 · s = 1700.3. The 307 of its pixels in
    /// 0/10's window that lie outside the trace are the crate's lower right,
    /// which the photograph shows behind a pine the client does not draw
    /// (parked 4079a).
    /// **A value before a cause** (item 4082, step 1): for each figure the
    /// trees hide at the woodcutter, on our own buffers at run1559b's
    /// camera on frame 3279 — its pixels drawn without trees; how many of
    /// them a tree's triangle covers; of those, how many the leaf's own
    /// texel passes (`render_states[9]`'s test, [`crate::world::TREE_ALPHA_REF`])
    /// and how many it cuts; and where a tree covers it, which is nearer by
    /// our depth and by how much (the tree's depth less the figure's,
    /// camera units: positive, the figure nearer). Prints the table; it
    /// asserts only that the five are drawn.
    #[test]
    fn the_figures_the_trees_cover_by_triangle_and_by_texel() {
        let install = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let path = photo_path();
        let Ok(bytes) = std::fs::read(&path) else {
            println!("skipping: no photograph at {}", path.display());
            return;
        };
        let photo = render::tga::decode(&bytes).expect("the photograph decodes");
        let mut world = match World::open(&install, &crate::world::gamelog_dir()) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        let camera = crate::pinned_camera();
        let size = (photo.width, photo.height);
        world.run_to(PHOTO_FRAME);
        let (figures, buildings, list) = world.draw_list(
            &camera,
            &crate::pinned_screen(),
            (size.0 as i32, size.1 as i32),
        );
        let drawings = world.objects(&figures, &buildings, &list);
        let sprites = world.sprite_drawings(&buildings, &list, camera.zoom_distance);
        let scene = rasterize_cut(
            &objects(&drawings),
            &[],
            &stamps(&sprites, drawings.len() as u32 + 1),
            &camera,
            size,
        );
        let trees = world.tree_drawings();
        for cull in [
            render::silhouette::Cull::CounterClockwise,
            render::silhouette::Cull::None,
            render::silhouette::Cull::Clockwise,
        ] {
            world.tree_cull = cull;
            println!("cull {cull:?}");
            let cuts = world.tree_cuts();
            assert!(!trees.is_empty() && cuts.iter().all(Option::is_some));
            let tree_objects = objects(&trees);
            let solid = rasterize_cut(&tree_objects, &[], &[], &camera, size);
            let cut: Vec<_> = cuts.iter().map(Option::as_ref).collect();
            let leaves = rasterize_cut(&tree_objects, &cut, &[], &camera, size);
            let count =
                |r: &render::silhouette::Raster| r.depth.iter().filter(|d| d.is_finite()).count();
            let lone: (usize, usize) = (0..trees.len())
                .map(|j| {
                    let one = [Object {
                        key: 1,
                        tris: trees[j].tris.clone(),
                    }];
                    let c = [cut[j]];
                    (
                        count(&rasterize_cut(&one, &[], &[], &camera, size)),
                        count(&rasterize_cut(&one, &c, &[], &camera, size)),
                    )
                })
                .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
            println!(
                "trees: solid {} pixels, by texel {}; each tree alone, summed: solid {} by texel {}",
                count(&solid),
                count(&leaves),
                lone.0,
                lone.1
            );
            let mut drawn = 0;
            for o in [1i16, 2, 6, 8, 10] {
                let keys: Vec<u32> = drawings
                    .iter()
                    .enumerate()
                    .filter(|(_, d)| (d.who, d.o, d.building) == (0, o, false))
                    .map(|(i, _)| i as u32 + 1)
                    .collect();
                let pixels: Vec<usize> = (0..scene.keys.keys.len())
                    .filter(|&i| keys.contains(&scene.keys.keys[i]))
                    .collect();
                drawn += !pixels.is_empty() as usize;
                let covered: Vec<usize> = pixels
                    .iter()
                    .copied()
                    .filter(|&i| solid.depth[i].is_finite())
                    .collect();
                let passed = covered
                    .iter()
                    .filter(|&&i| leaves.depth[i].is_finite())
                    .count();
                let mut gaps: Vec<f32> = covered
                    .iter()
                    .map(|&i| solid.depth[i] - scene.depth[i])
                    .collect();
                gaps.sort_by(f32::total_cmp);
                let nearer = gaps.iter().filter(|g| **g > 0.0).count();
                let mut leaf_gaps: Vec<f32> = covered
                    .iter()
                    .filter(|&&i| leaves.depth[i].is_finite())
                    .map(|&i| leaves.depth[i] - scene.depth[i])
                    .collect();
                leaf_gaps.sort_by(f32::total_cmp);
                let leaf_nearer = leaf_gaps.iter().filter(|g| **g > 0.0).count();
                let median = |g: &[f32]| g.get(g.len() / 2).copied().unwrap_or(f32::NAN);
                let kept_solid = pixels.len() - (covered.len() - nearer);
                let kept_alpha = pixels.len() - (leaf_gaps.len() - leaf_nearer);
                println!(
                    "figure 0/{o}: pixels {} covered {} texel passes {} cuts {}; covered, figure nearer {} tree nearer {} (median tree-figure {:.1}, range {:.1}..{:.1}); passed, figure nearer {} tree nearer {} (median {:.1}); kept solid {} kept by texel {}",
                    pixels.len(),
                    covered.len(),
                    passed,
                    covered.len() - passed,
                    nearer,
                    covered.len() - nearer,
                    median(&gaps),
                    gaps.first().copied().unwrap_or(f32::NAN),
                    gaps.last().copied().unwrap_or(f32::NAN),
                    leaf_nearer,
                    leaf_gaps.len() - leaf_nearer,
                    median(&leaf_gaps),
                    kept_solid,
                    kept_alpha,
                );
            }
            assert_eq!(drawn, 5, "the five figures are drawn");
        }
    }

    #[test]
    fn the_photograph_on_the_pixels_the_pine_covers() {
        let install = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let path = photo_path();
        let Ok(bytes) = std::fs::read(&path) else {
            println!("skipping: no photograph at {}", path.display());
            return;
        };
        let photo = render::tga::decode(&bytes).expect("the photograph decodes");
        let mut world = match World::open(&install, &crate::world::gamelog_dir()) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        let camera = crate::pinned_camera();
        let size = (photo.width, photo.height);
        world.run_to(PHOTO_FRAME);
        let (figures, buildings, list) = world.draw_list(
            &camera,
            &crate::pinned_screen(),
            (size.0 as i32, size.1 as i32),
        );
        let drawings = world.objects(&figures, &buildings, &list);
        let sprites = world.sprite_drawings(&buildings, &list, camera.zoom_distance);
        let scene = rasterize_cut(
            &objects(&drawings),
            &[],
            &stamps(&sprites, drawings.len() as u32 + 1),
            &camera,
            size,
        );
        let trees = world.tree_drawings();
        let cuts = world.tree_cuts();
        let cut: Vec<_> = cuts.iter().map(Option::as_ref).collect();
        let leaves = rasterize_cut(&objects(&trees), &cut, &[], &camera, size);
        let key_of = |o: i16| -> Vec<u32> {
            drawings
                .iter()
                .enumerate()
                .filter(|(_, d)| (d.who, d.o, d.building) == (0, o, false))
                .map(|(i, _)| i as u32 + 1)
                .collect()
        };
        let w = size.0;
        let mut boxes = Vec::new();
        for o in [1i16, 2, 8, 10, 6, 9] {
            let keys = key_of(o);
            let px: Vec<usize> = (0..scene.keys.keys.len())
                .filter(|&i| keys.contains(&scene.keys.keys[i]))
                .collect();
            let (x0, x1) = (
                px.iter().map(|i| i % w).min().unwrap(),
                px.iter().map(|i| i % w).max().unwrap(),
            );
            let (y0, y1) = (
                px.iter().map(|i| i / w).min().unwrap(),
                px.iter().map(|i| i / w).max().unwrap(),
            );
            println!(
                "figure 0/{o}: {} px, x {x0}..={x1}, y {y0}..={y1}",
                px.len()
            );
            boxes.push((o, x0, x1, y0, y1));
        }
        // Every figure's pixels, ours, dilated by two: the leaf's reference
        // excludes them.
        let all_figs: Vec<u32> = boxes.iter().flat_map(|b| key_of(b.0)).collect();
        let fig_at = |x: i64, y: i64| -> bool {
            (-2..=2).any(|dy| {
                (-2..=2).any(|dx| {
                    let (xx, yy) = (x + dx, y + dy);
                    xx >= 0
                        && yy >= 0
                        && all_figs.contains(&scene.keys.keys[yy as usize * w + xx as usize])
                })
            })
        };
        let rgb = |i: usize| -> [f32; 3] {
            [
                photo.rgba[i * 4] as f32,
                photo.rgba[i * 4 + 1] as f32,
                photo.rgba[i * 4 + 2] as f32,
            ]
        };
        // The photograph's flat red: red over both other channels by 60.
        let red = |c: [f32; 3]| c[0] - c[1] > 60.0 && c[0] - c[2] > 60.0;
        // The leaf under a pixel: the mean of the photograph within seven
        // pixels, under our pine, clear of every figure, and not red.
        let leaf = |x: usize, y: usize| -> Option<[f32; 3]> {
            let (mut sum, mut n) = ([0f32; 3], 0f32);
            for yy in y.saturating_sub(7)..=y + 7 {
                for xx in x.saturating_sub(7)..=x + 7 {
                    let i = yy * w + xx;
                    let c = rgb(i);
                    if leaves.depth[i].is_finite() && !fig_at(xx as i64, yy as i64) && !red(c) {
                        for k in 0..3 {
                            sum[k] += c[k];
                        }
                        n += 1.0;
                    }
                }
            }
            (n >= 8.0).then(|| sum.map(|v| v / n))
        };
        // The figure's share of a pixel, under `c = a * (255, 0, 0) + (1 - a) * leaf`,
        // read on the red and on the green channel apart.
        let classify = |name: &str, px: &[usize]| {
            let (mut bins, mut red_n, mut no_leaf) = ([0usize; 11], 0, 0);
            let mut ar = Vec::new();
            let mut ag = Vec::new();
            for &i in px {
                let (x, y) = (i % w, i / w);
                let c = rgb(i);
                red_n += red(c) as usize;
                let Some(l) = leaf(x, y) else {
                    no_leaf += 1;
                    continue;
                };
                let a_g = (1.0 - c[1] / l[1].max(1.0)).clamp(-0.5, 1.5);
                let a_r = ((c[0] - l[0]) / (255.0 - l[0]).max(1.0)).clamp(-0.5, 1.5);
                // The share of `render_states[1]`'s tfactor colour (255, 82, 34)
                // over the leaf, least squares on all three channels.
                let t = [255.0f32, 82.0, 34.0];
                let (mut num, mut den) = (0f32, 0f32);
                for k in 0..3 {
                    num += (c[k] - l[k]) * (t[k] - l[k]);
                    den += (t[k] - l[k]) * (t[k] - l[k]);
                }
                let a_t = (num / den.max(1.0)).clamp(0.0, 1.0);
                bins[(a_t * 10.0).round() as usize] += 1;
                ar.push(a_r);
                ag.push(a_g);
            }
            ar.sort_by(f32::total_cmp);
            ag.sort_by(f32::total_cmp);
            let q = |v: &[f32], f: f32| {
                v.get(((v.len() as f32 - 1.0) * f) as usize)
                    .copied()
                    .unwrap_or(f32::NAN)
            };
            println!(
                "{name}: {} px, red {red_n}, no leaf {no_leaf}; a by tfactor, tenths 0..=10 {bins:?}; a_g q10/50/90 {:.2}/{:.2}/{:.2}; a_r q10/50/90 {:.2}/{:.2}/{:.2}",
                px.len(),
                q(&ag, 0.1),
                q(&ag, 0.5),
                q(&ag, 0.9),
                q(&ar, 0.1),
                q(&ar, 0.5),
                q(&ar, 0.9),
            );
            let mode = (0..11).max_by_key(|&b| bins[b]).unwrap();
            (red_n, mode)
        };
        for &(o, ..) in &boxes {
            let keys = key_of(o);
            let px: Vec<usize> = (0..scene.keys.keys.len())
                .filter(|&i| keys.contains(&scene.keys.keys[i]) && leaves.depth[i].is_finite())
                .collect();
            let (red_n, mode) = classify(&format!("0/{o} under the pine"), &px);
            let shifted: Vec<usize> = px.iter().map(|i| i + 10).collect();
            let (red_shifted, mode_shifted) = classify(&format!("0/{o} shifted 10 px"), &shifted);
            if [1, 2, 8, 10].contains(&o) {
                // The silhouette, `render_states[1]`'s tfactor at 128/255
                // over the leaf (item 4085): red on nine in ten of the
                // pixels the pine covers, its share's mode a half or more;
                // ten pixels over, neither.
                assert!(red_n * 10 >= px.len() * 9, "0/{o} is red");
                assert!((5..=7).contains(&mode), "0/{o}'s share is the state's");
                assert!(red_shifted * 4 <= px.len(), "0/{o} shifted is not red");
                assert_eq!(mode_shifted, 0, "0/{o} shifted is the leaf");
            }
            // Where the photograph's red stands against our figure.
            let (x0, x1, y0, y1) = boxes
                .iter()
                .find(|b| b.0 == o)
                .map(|b| (b.1 - 6, b.2 + 6, b.3 - 6, b.4 + 6))
                .unwrap();
            let (mut ours_red, mut ours_n, mut red_out) = (0, 0, 0);
            let (mut cr, mut co) = ((0f32, 0f32, 0f32), (0f32, 0f32, 0f32));
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let i = y * w + x;
                    let mine = keys.contains(&scene.keys.keys[i]);
                    let r = red(rgb(i));
                    ours_n += mine as usize;
                    ours_red += (mine && r) as usize;
                    red_out += (!mine && r && !fig_at(x as i64, y as i64)) as usize;
                    if mine {
                        co = (co.0 + x as f32, co.1 + y as f32, co.2 + 1.0);
                    }
                    if r {
                        cr = (cr.0 + x as f32, cr.1 + y as f32, cr.2 + 1.0);
                    }
                }
            }
            println!(
                "0/{o} place: ours {ours_n}, of them red {ours_red}; red clear of every figure {red_out}; centroid ours ({:.1}, {:.1}) red ({:.1}, {:.1})",
                co.0 / co.2,
                co.1 / co.2,
                cr.0 / cr.2,
                cr.1 / cr.2
            );
        }
        // The woodcutter, which no tree covers: its own pixels.
        let wood: Vec<u32> = (0..drawings.len() + sprites.len())
            .filter(|&j| {
                if j < drawings.len() {
                    (drawings[j].who, drawings[j].o) == (0, 2001)
                } else {
                    (
                        sprites[j - drawings.len()].who,
                        sprites[j - drawings.len()].o,
                    ) == (0, 2001)
                }
            })
            .map(|j| j as u32 + 1)
            .collect();
        let wpx: Vec<usize> = (0..scene.keys.keys.len())
            .filter(|&i| wood.contains(&scene.keys.keys[i]))
            .collect();
        assert_eq!(
            classify("0/2001, the woodcutter", &wpx).1,
            0,
            "no silhouette on the woodcutter"
        );
        // The pine by row: ours, the one tree over the figures alone, and
        // the photograph's foliage walked left from inside ours.
        let pine = (0..trees.len())
            .max_by_key(|&j| {
                let one = [Object {
                    key: 1,
                    tris: trees[j].tris.clone(),
                }];
                let r = rasterize_cut(&one, &[cut[j]], &[], &camera, size);
                (0..r.keys.keys.len())
                    .filter(|&i| r.depth[i].is_finite() && all_figs.contains(&scene.keys.keys[i]))
                    .count()
            })
            .unwrap();
        let lone = rasterize_cut(
            &[Object {
                key: 1,
                tris: trees[pine].tris.clone(),
            }],
            &[cut[pine]],
            &[],
            &camera,
            size,
        );
        let foliage = |c: [f32; 3]| {
            let grass = c[1] > 135.0 && c[0] > 130.0;
            !grass && !red(c) && c[1] >= c[2] + 15.0
        };
        println!("the pine over the figures is tree {pine}");
        for y in (384..=480).step_by(4) {
            let row: Vec<usize> = (1600..1800)
                .filter(|&x| lone.depth[y * w + x].is_finite())
                .collect();
            let all = (1560..1800).find(|&x| leaves.depth[y * w + x].is_finite());
            let (Some(&l), Some(&r)) = (row.first(), row.last()) else {
                continue;
            };
            let mut t = l + 5;
            while t > 1560 && foliage(rgb(y * w + t - 1)) {
                t -= 1;
            }
            println!(
                "row {y}: pine ours {l}..={r}; every tree ours from {all:?}; the photograph's foliage from {t}"
            );
        }
        if let Some(dir) = std::env::var_os("PROBE_DIR") {
            let dir = PathBuf::from(dir);
            let (x0, x1, y0, y1) = (1540usize, 1760usize, 300usize, 500usize);
            let mut a = format!("P6 {} {} 255\n", x1 - x0, y1 - y0).into_bytes();
            let mut b = a.clone();
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = y * w + x;
                    let c = &photo.rgba[i * 4..i * 4 + 3];
                    a.extend_from_slice(c);
                    let k = scene.keys.keys[i];
                    let fig = boxes.iter().position(|b| key_of(b.0).contains(&k));
                    let pine = leaves.depth[i].is_finite();
                    let o = match (fig, pine) {
                        (Some(f), true) => [255, (40 * f) as u8, 255],
                        (Some(f), false) => [255, (40 * f) as u8, 0],
                        (None, true) => [0, 160, 0],
                        (None, false) => [c[0] / 3, c[1] / 3, c[2] / 3],
                    };
                    b.extend_from_slice(&o);
                }
            }
            std::fs::write(dir.join("photo.ppm"), a).unwrap();
            std::fs::write(dir.join("ours.ppm"), b).unwrap();
        }
    }

    #[test]
    fn the_woodcutter_s_right_edge_is_the_sprite_s_own() {
        let install = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let mut world = match World::open(&install, &crate::world::gamelog_dir()) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        let camera = crate::pinned_camera();
        // The sprite's own edge, read on the treeless draw (item 4080), the
        // figures hidden where it is nearer: with the silhouettes (item
        // 4085) they take 61 of these pixels, (307, 99) → (246, 80).
        world.trees = false;
        world.silhouettes = crate::world::Silhouettes::None;
        world.run_to(PHOTO_FRAME);
        let (_, buildings, list) = world.draw_list(&camera, &crate::pinned_screen(), (1920, 1080));
        let d = list
            .iter()
            .find(|d| (d.who, d.o) == (0, 2001))
            .expect("listed");
        assert_eq!((d.anchor.x, d.anchor.y), (1640, 438));
        let sprites = world.sprite_drawings(&buildings, &list, camera.zoom_distance);
        let sp = sprites
            .iter()
            .find(|s| (s.who, s.o) == (0, 2001))
            .expect("drawn as its sprite");
        let im = &sp.image.image;
        assert_eq!((im.width, im.height), (173, 201));
        assert_eq!(sp.image.offset, (83, 178));
        assert_eq!((sp.at.left, sp.at.top), (1585, 320));
        // Partial alpha, over the image and in its columns past screen x
        // 1688 (u >= 155).
        let partial = |u0: usize| {
            (0..im.height)
                .flat_map(|v| (u0..im.width).map(move |u| (v, u)))
                .filter(|&(v, u)| {
                    let a = im.rgba[(v * im.width + u) * 4 + 3];
                    a != 0 && a != 255
                })
                .count()
        };
        let opaque_past = (0..im.height)
            .flat_map(|v| (155..im.width).map(move |u| (v, u)))
            .filter(|&(v, u)| im.rgba[(v * im.width + u) * 4 + 3] == 255)
            .count();
        println!(
            "partial alpha: {} in the image, {} past column 155; opaque past it {opaque_past}",
            partial(0),
            partial(155)
        );
        assert_eq!(partial(155), 0, "the right columns are two-level");
        let last = (0..im.height)
            .filter(|&v| im.rgba[(v * im.width + im.width - 1) * 4 + 3] == 255)
            .count();
        println!("opaque rows in the last column: {last}");
        assert!(last > 0, "the image's last column is opaque");
        let right = sp.at.left as f32 + im.width as f32 * sp.at.scale;
        println!("the sprite's right edge: {right:.1}");
        assert_eq!(right as i32, 1700);
        let (keys, who) = frame_keys(&mut world, &camera, (1920, 1080));
        let wood = who
            .iter()
            .position(|&(w, o, _, _)| (w, o) == (0, 2001))
            .expect("keyed") as u32
            + 1;
        let trace = TRACED
            .iter()
            .find(|(n, _)| *n == "0/2001")
            .expect("traced")
            .1;
        let (mut outside, mut past) = (0, 0);
        for y in 404..452i32 {
            for x in 1680..1710i32 {
                if keys.keys[y as usize * 1920 + x as usize] == wood
                    && !render::silhouette::inside(trace, x, y)
                {
                    outside += 1;
                    past += usize::from(x >= 1688);
                }
            }
        }
        println!("0/10's window: the woodcutter outside its trace {outside}, at x >= 1688 {past}");
        assert_eq!((outside, past), (307, 99));
    }

    /// **Item 4086, step 1: the riflemen by the tank, by row.** On frame
    /// 3279 at run1559b's camera, on the default draw, each of 1/31, 1/20
    /// and 1/32, and a player-1 figure in the row that stands clear, read
    /// as `read_traced` reads its window: `own` our pixels keyed to the
    /// figure, `other` ours keyed to anything else, `theirs` the
    /// photograph's pixels the window's ground does not hold. Per figure:
    /// both sides' box, size and centroid, the rows, and the window as a
    /// map. Shifted ten pixels, our figure no longer stands on theirs.
    #[test]
    fn the_riflemen_by_the_tank_by_row() {
        let install = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let path = photo_path();
        let Ok(bytes) = std::fs::read(&path) else {
            println!("skipping: no photograph at {}", path.display());
            return;
        };
        let photo = render::tga::decode(&bytes).expect("the photograph decodes");
        let mut world = match World::open(&install, &crate::world::gamelog_dir()) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        let camera = crate::pinned_camera();
        world.run_to(PHOTO_FRAME);
        let rows = score_frame_traced(
            &mut world,
            &camera,
            &photo,
            &TRACED,
            Neighbour::Neither,
            Interface::Neither,
        );
        let size = (photo.width, photo.height);
        let (keys, who) = frame_keys(&mut world, &camera, size);
        let p = Photo {
            width: photo.width,
            height: photo.height,
            rgba: &photo.rgba,
        };
        let w = size.0;
        // Each figure drawn alone: its whole extent on our draw.
        let (figures, buildings, list) = world.draw_list(
            &camera,
            &crate::pinned_screen(),
            (size.0 as i32, size.1 as i32),
        );
        let drawings = world.objects(&figures, &buildings, &list);
        let alone_of = |o: i16| {
            let objs: Vec<Object> = objects(&drawings)
                .into_iter()
                .filter(|b| {
                    let d = &drawings[b.key as usize - 1];
                    (d.who, d.o, d.building) == (1, o, false)
                })
                .collect();
            rasterize_cut(&objs, &[], &[], &camera, size).keys
        };
        let iface = |x: i32, y: i32| {
            INTERFACE
                .iter()
                .any(|r| (r.x0..r.x1).contains(&x) && (r.y0..r.y1).contains(&y))
        };
        // A clear figure: player 1's, in the row, its window holding no
        // pixel of ours but its own and no trace.
        let traces: Vec<(u32, &[(i32, i32)])> = TRACED
            .iter()
            .map(|&(name, poly)| {
                let k = who
                    .iter()
                    .position(|&(a, o, _, _)| format!("{a}/{o}") == name)
                    .map_or(u32::MAX, |i| i as u32 + 1);
                (k, poly)
            })
            .collect();
        let own_of = |f: &Footprint| -> (u8, i16) {
            let d = who[f.key as usize - 1];
            (d.0, d.1)
        };
        let clear = rows
            .iter()
            .filter(|(f, d)| d.0 == 1 && !d.3 && f.overlap() >= READING.threshold)
            .find(|(f, _)| {
                let me = own_of(f);
                let r = READING.ring;
                let win = Rect::new(
                    f.window.x0 - r,
                    f.window.y0 - r,
                    f.window.x1 + r,
                    f.window.y1 + r,
                );
                (win.y0..win.y1).all(|y| {
                    (win.x0..win.x1).all(|x| {
                        let k = keys.at(x as usize, y as usize);
                        (k == 0 || {
                            let d = who[k as usize - 1];
                            (d.0, d.1) == me && !d.3
                        }) && !traces.iter().any(|t| render::silhouette::inside(t.1, x, y))
                    })
                })
            })
            .map(|(_, d)| d.1)
            .expect("a clear figure of player 1's in the row");
        println!("the clear figure: 1/{clear}");
        let mut owed = Vec::new();
        for o in [31i16, 20, 32, clear] {
            let (f, d) = rows
                .iter()
                .find(|(_, d)| (d.0, d.1, d.3) == (1, o, false))
                .copied()
                .expect("the figure is scored");
            let win = f.window;
            let r = READING.ring;
            let outer = Rect::new(win.x0 - r, win.y0 - r, win.x1 + r, win.y1 + r);
            let key = f.key;
            let alone = alone_of(o);
            let whole = |x: i32, y: i32| alone.at(x as usize, y as usize) != 0;
            let kind = |x: i32, y: i32| -> u8 {
                if iface(x, y) {
                    return 3;
                }
                let mut k = 0;
                for t in &traces {
                    if render::silhouette::inside(t.1, x, y) {
                        if t.0 == key {
                            return 1;
                        }
                        k = 2;
                    }
                }
                k
            };
            let g = render::silhouette::ground_where(&keys, &p, win, outer, &READING, |x, y| {
                kind(x, y) == 0
            });
            let own = |x: i32, y: i32| {
                let k = keys.at(x as usize, y as usize);
                k != 0 && {
                    let e = who[k as usize - 1];
                    (e.0, e.1, e.3) == (1, o, false)
                }
            };
            let theirs = |x: i32, y: i32| kind(x, y) == 0 && !g.holds(&p, x, y);
            // Box, size, centroid and count of a mask in the window.
            let measure = |m: &dyn Fn(i32, i32) -> bool| {
                let (mut n, mut sx, mut sy) = (0i64, 0i64, 0i64);
                let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
                for y in win.y0..win.y1 {
                    for x in win.x0..win.x1 {
                        if kind(x, y) >= 2 || !m(x, y) {
                            continue;
                        }
                        n += 1;
                        sx += i64::from(x);
                        sy += i64::from(y);
                        (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
                    }
                }
                let n1 = n.max(1) as f64;
                (
                    n,
                    (x0, y0, x1, y1),
                    if n == 0 {
                        (0, 0)
                    } else {
                        (x1 - x0 + 1, y1 - y0 + 1)
                    },
                    (sx as f64 / n1, sy as f64 / n1),
                )
            };
            let mo = measure(&own);
            let mt = measure(&theirs);
            let mot = measure(&|x, y| own(x, y) && theirs(x, y));
            let mw = measure(&whole);
            let mwt = measure(&|x, y| whole(x, y) && theirs(x, y));
            println!(
                "1/{o} piece {} window ({}, {})..({}, {}): ours {} theirs {} both {} overlap {:.3}",
                d.2,
                win.x0,
                win.y0,
                win.x1,
                win.y1,
                f.ours,
                f.theirs,
                f.both,
                f.overlap()
            );
            println!(
                "  own: {} px, box {:?}, {}x{}, centroid ({:.1}, {:.1})",
                mo.0, mo.1, mo.2.0, mo.2.1, mo.3.0, mo.3.1
            );
            println!(
                "  theirs: {} px, box {:?}, {}x{}, centroid ({:.1}, {:.1}); on own {}",
                mt.0, mt.1, mt.2.0, mt.2.1, mt.3.0, mt.3.1, mot.0
            );
            println!(
                "  alone: {} px, box {:?}, {}x{}, centroid ({:.1}, {:.1}); on theirs {}; hidden in our draw {}",
                mw.0,
                mw.1,
                mw.2.0,
                mw.2.1,
                mw.3.0,
                mw.3.1,
                mwt.0,
                mw.0 - mo.0
            );
            // Who else of ours is in the window, by key.
            let mut others: std::collections::BTreeMap<String, u32> = Default::default();
            for y in win.y0..win.y1 {
                for x in win.x0..win.x1 {
                    let k = keys.at(x as usize, y as usize);
                    if k != 0 && !own(x, y) && kind(x, y) < 2 {
                        let e = who[k as usize - 1];
                        *others
                            .entry(format!("{}/{} piece {}", e.0, e.1, e.2))
                            .or_default() += 1;
                    }
                }
            }
            println!("  others of ours in the window: {others:?}");
            // What the photograph holds on our own pixels it does not read
            // as the figure's: their mean colour and how many are red.
            let (mut n, mut sum, mut reds) = (0u32, [0u32; 3], 0u32);
            for y in win.y0..win.y1 {
                for x in win.x0..win.x1 {
                    if own(x, y) && kind(x, y) == 0 && !theirs(x, y) {
                        let i = (y as usize * w + x as usize) * 4;
                        let c = [photo.rgba[i], photo.rgba[i + 1], photo.rgba[i + 2]];
                        n += 1;
                        for k in 0..3 {
                            sum[k] += u32::from(c[k]);
                        }
                        let (r, gg, b) = (i32::from(c[0]), i32::from(c[1]), i32::from(c[2]));
                        reds += u32::from(r - gg > 60 && r - b > 60);
                    }
                }
            }
            println!(
                "  own, not theirs: {n} px, mean {:?}, red {reds}",
                sum.map(|v| v / n.max(1))
            );
            // The window by row, then as a map: '#' own and theirs, 'o' own
            // only, '%' another of ours and theirs, '-' another only, '+'
            // theirs only, 'T' a neighbour's trace, '=' its own trace.
            // The photograph's red among theirs, on our own, on another of
            // ours, and on nothing of ours.
            let mut red = [0u32; 3];
            for y in win.y0..win.y1 {
                for x in win.x0..win.x1 {
                    if !theirs(x, y) {
                        continue;
                    }
                    let i = (y as usize * w + x as usize) * 4;
                    let (r, gg, b) = (
                        i32::from(photo.rgba[i]),
                        i32::from(photo.rgba[i + 1]),
                        i32::from(photo.rgba[i + 2]),
                    );
                    if r - gg > 60 && r - b > 60 {
                        let k = keys.at(x as usize, y as usize);
                        red[if own(x, y) {
                            0
                        } else if k != 0 {
                            1
                        } else {
                            2
                        }] += 1;
                    }
                }
            }
            println!(
                "  theirs red: on own {}, on another {}, on none {}",
                red[0], red[1], red[2]
            );
            println!(
                "  row: own other theirs own&theirs | map ('*' hidden by another, 'I' interface)"
            );
            for y in win.y0..win.y1 {
                let (mut a, mut b, mut c, mut e) = (0, 0, 0, 0);
                let mut line = String::new();
                for x in win.x0..win.x1 {
                    let k = kind(x, y);
                    let ow = own(x, y);
                    let ot = keys.at(x as usize, y as usize) != 0 && !ow;
                    let th = theirs(x, y);
                    if k < 2 {
                        a += i32::from(ow);
                        b += i32::from(ot);
                        c += i32::from(th);
                        e += i32::from(ow && th);
                    }
                    let hid = whole(x, y) && !ow;
                    line.push(match (k, ow, ot, th) {
                        (3, _, _, _) => 'I',
                        (2, _, _, _) => 'T',
                        (_, _, true, _) if hid => '*',
                        (1, _, _, _) => '=',
                        (_, true, _, true) => '#',
                        (_, true, _, false) => 'o',
                        (_, _, true, true) => '%',
                        (_, _, true, false) => '-',
                        (_, _, _, true) => '+',
                        _ => '.',
                    });
                }
                println!("  {y:4}: {a:3} {b:3} {c:3} {e:3} | {line}");
            }
            // The photograph's luminance by tenths of 256, ring-free, beside
            // the figure drawn alone ('A') and where ours is another ('-').
            println!("  row: photograph's luminance | ours");
            for y in win.y0..win.y1 {
                let (mut l, mut m) = (String::new(), String::new());
                for x in win.x0..win.x1 {
                    let i = (y as usize * w + x as usize) * 4;
                    let lum = (299 * u32::from(photo.rgba[i])
                        + 587 * u32::from(photo.rgba[i + 1])
                        + 114 * u32::from(photo.rgba[i + 2]))
                        / 1000;
                    l.push(char::from(b'0' + (lum * 10 / 256) as u8));
                    let k = keys.at(x as usize, y as usize);
                    m.push(if whole(x, y) {
                        'A'
                    } else if k != 0 {
                        '-'
                    } else {
                        '.'
                    });
                }
                println!("  {y:4}: {l} | {m}");
            }
            // Seen to fail: our own mask shifted ten pixels right.
            let shifted = measure(&|x, y| own(x - 10, y) && theirs(x, y));
            println!("  own shifted 10 px, on theirs: {} of {}", shifted.0, mo.0);
            owed.push((o, mo, mt, mot.0, shifted.0, mw, mwt.0));
        }
        // Step 2: each figure's state as our draw takes it, at the
        // photograph's frame and at the packet's (3294), beside
        // `tree_points.py --guys`'s rows of the packet.
        // `(o, x, y, z, cur_anim, cur_time, angle)` of each `GuyData` on
        // run1559b's packet (`tree_points.py --guys`, item 4086).
        const PACKET_3294: [(i16, i32, i32, i32, i8, u32, u32); 5] = [
            (31, 4776, 31464, 534, 11, 23, 0xcfa9_0000),
            (20, 4536, 31224, 593, 0, 15, 0x9d08_0000),
            (32, 4584, 31320, 578, 0, 29, 0x5d77_0000),
            (35, 5208, 31272, 487, 9, 4, 0xd112_0000),
            (38, 4920, 31560, 506, 0, 25, 0x3781_0000),
        ];
        let state = |world: &mut World| -> Vec<(i16, i32, i32, i32, i8, u32, u32)> {
            let mut out = Vec::new();
            let (figures, _, list) = world.draw_list(
                &camera,
                &crate::pinned_screen(),
                (size.0 as i32, size.1 as i32),
            );
            for o in [31i16, 20, 32, 35, 38] {
                let p = figures
                    .iter()
                    .find(|p| (p.who, p.o) == (1, o))
                    .expect("placed");
                let facing = list.iter().find_map(|d| match d.pose {
                    render::draw::Pose::Figure { facing, .. } if (d.who, d.o) == (1, o) => {
                        Some(facing)
                    }
                    _ => None,
                });
                println!(
                    "ours 1/{o} at {}: x {} {} {} piece {} slot {} frame {} ms {} facing {:#x}",
                    world.runner.frame(),
                    p.x,
                    p.y,
                    p.z,
                    p.piece,
                    p.slot,
                    p.frame,
                    p.frame * 200 / 3,
                    facing.unwrap_or(0)
                );
                out.push((o, p.x, p.y, p.z, p.slot, p.frame, facing.unwrap_or(0)));
            }
            out
        };
        state(&mut world);
        world.run_to(3294);
        assert_eq!(
            state(&mut world),
            PACKET_3294,
            "ours is the packet's at 3294"
        );
        // The clear figure: ours and theirs alike, and not so shifted.
        let (_, mo, mt, both, shifted, _, _) = owed[3];
        assert!(
            (mo.3.0 - mt.3.0).abs() < 3.0 && (mo.3.1 - mt.3.1).abs() < 3.0,
            "the clear figure's centroids agree"
        );
        assert!(
            both * 10 >= mo.0 * 6,
            "the clear figure's own stands on theirs"
        );
        assert!(shifted * 2 < both, "shifted, it does not");
    }
}
