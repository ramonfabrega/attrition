//! Candidate C's first row (`docs/RENDERER.md` §5.3, item 4038): the
//! scene's silhouettes on run1559b's frame 3294 against that take's own
//! photograph of it, at the photograph's camera.
//!
//! The photograph is kept outside the repository, converted once to a
//! `.tga` so `render::tga` reads it with no new dependency
//! (`sips -s format tga`): `$RON_PHOTO`, else
//! `~/ron-data/lab-experiments/2026-11-09-item-4038/run1559b-3294.tga`.

use crate::world::{Drawing, SpriteDrawing, World};
use render::silhouette::{
    Footprint, Object, Photo, READING, Reader, Rect, Stamp, footprints_read, rasterize_with,
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
    let keys = rasterize_with(&objects(&drawings), &stamps(&sprites, first), camera, size);
    let who = drawings
        .iter()
        .map(|d| (d.who, d.o, d.piece, d.building))
        .chain(sprites.iter().map(|s| (s.who, s.o, s.piece, true)))
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
}
