//! Candidate C's first row (`docs/RENDERER.md` §5.3, item 4038): the
//! scene's silhouettes on run1559b's frame 3294 against that take's own
//! photograph of it, at the photograph's camera.
//!
//! The photograph is kept outside the repository, converted once to a
//! `.tga` so `render::tga` reads it with no new dependency
//! (`sips -s format tga`): `$RON_PHOTO`, else
//! `~/ron-data/lab-experiments/2026-11-09-item-4038/run1559b-3294.tga`.

use crate::world::{Drawing, World};
use render::silhouette::{Footprint, Object, Photo, READING, Rect, footprints, rasterize};
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

/// Every scored object's footprints on the frame at `camera`, beside its
/// drawing.
pub fn score_frame(
    world: &mut World,
    camera: &render::Camera,
    photo: &render::tga::Image,
) -> Vec<(Footprint, (u8, i16, i32, bool))> {
    let size = (photo.width, photo.height);
    let (figures, buildings, list) = world.draw_list(
        camera,
        &crate::pinned_screen(),
        (size.0 as i32, size.1 as i32),
    );
    let drawings = world.objects(&figures, &buildings, &list);
    let keys = rasterize(&objects(&drawings), camera, size);
    let photo = Photo {
        width: photo.width,
        height: photo.height,
        rgba: &photo.rgba,
    };
    footprints(&keys, &photo, &INTERFACE, &READING)
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
        let rows = score_frame(&mut world, &camera, &photo);
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
        println!("silhouettes {} of {}", got.0, got.1);
        assert_eq!(
            got,
            render::silhouette::RUN1559B_3294,
            "candidate C's first row"
        );

        let mut elsewhere = camera;
        (elsewhere.loc_x, elsewhere.loc_y) = (3168.0, 29744.68);
        let rows = score_frame(&mut world, &elsewhere, &photo);
        let footprints: Vec<Footprint> = rows.iter().map(|(f, _)| *f).collect();
        let off = score(&footprints, &READING);
        println!("at run1441's camera: {} of {}", off.0, off.1);
        assert_eq!(off, render::silhouette::RUN1441_CAMERA_3294);
        assert!(
            off.0 < got.0,
            "the photograph's camera scores above another"
        );
    }
}
