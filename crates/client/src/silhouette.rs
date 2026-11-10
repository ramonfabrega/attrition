//! Candidate C's first row (`docs/RENDERER.md` §5.3, item 4038): the
//! scene's silhouettes on run1559b's frame 3294 against that take's own
//! photograph of it, at the photograph's camera.
//!
//! The photograph is kept outside the repository, converted once to a
//! `.tga` so `render::tga` reads it with no new dependency
//! (`sips -s format tga`): `$RON_PHOTO`, else
//! `~/ron-data/lab-experiments/2026-11-09-item-4038/run1559b-3294.tga`.

use crate::world::{Drawing, World};
use render::silhouette::{
    Footprint, Object, Photo, READING, Reader, Rect, footprints_read, rasterize,
};
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
#[cfg(test)]
pub const TRACED: [(&str, &[(i32, i32)]); 7] = [
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

/// The frame's drawings at `camera`, and their key buffer at `size`.
pub fn frame_keys(
    world: &mut World,
    camera: &render::Camera,
    size: (usize, usize),
) -> (render::silhouette::Keys, Vec<Drawing>) {
    let (figures, buildings, list) = world.draw_list(
        camera,
        &crate::pinned_screen(),
        (size.0 as i32, size.1 as i32),
    );
    let drawings = world.objects(&figures, &buildings, &list);
    let keys = rasterize(&objects(&drawings), camera, size);
    (keys, drawings)
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
            let d = &drawings[f.key as usize - 1];
            (f, (d.who, d.o, d.piece, d.building))
        })
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
    let (figures, buildings, list) = world.draw_list(
        camera,
        &crate::pinned_screen(),
        (size.0 as i32, size.1 as i32),
    );
    let drawings = world.objects(&figures, &buildings, &list);
    let keys = rasterize(&objects(&drawings), camera, size);
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
        world.run_to(3294);
        let camera = crate::pinned_camera();
        // The default drawing: every figure posed, at the original's facing.
        assert!(world.pose, "the client draws posed by default");
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

        let mut elsewhere = camera;
        (elsewhere.loc_x, elsewhere.loc_y) = (3168.0, 29744.68);
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
            let traced = TRACED.iter().any(|(n, _)| *n == name);
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
}
