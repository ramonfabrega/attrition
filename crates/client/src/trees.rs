//! **The trees' own pixels** (item 4173; `docs/RENDERER.md` §3.7, "The
//! trees' own colour"): what colour a tree's pixel takes, and the row that
//! reads it against the photograph, `RUN1559B_3279_TREES`.
//!
//! A tree is drawn by `TreeFlushMethod::execute` under `render_states[9]`.
//! run1566b's card trace holds that draw on the pass after frame 3279, one
//! draw a texture: stage 0 `ADDSIGNED(TEXTURE, CURRENT)`, stage 1
//! `MODULATE(CURRENT, TFACTOR)` at a white factor, the options
//! `ALPHA_TEST VERTEX_COLOR_DIFFUSE TL_FAKE_LIGHT` and no `LIGHTING`; the
//! vertices' diffuse is `render::silhouette::TREE_DIFFUSE`.

use render::silhouette::Texel;

/// **How a tree's pixel is coloured** (item 4173; the candidates the row
/// counts, the rule committed before the count).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeColour {
    /// The flat green `World::tree_soup` drew before item 4173, shaded by
    /// its face and stored by the sRGB target: the client's still.
    #[cfg_attr(not(test), allow(dead_code))]
    Flat,
    /// The stage colour alone: the leaf's texel under `ADDSIGNED` with the
    /// vertex diffuse (`CURRENT` on stage 0 the diffuse, as Direct3D
    /// defines it), `t + 0x80 − 127.5`.
    #[cfg_attr(not(test), allow(dead_code))]
    Texel,
    /// [`TreeColour::Texel`] through `TL_FAKE_LIGHT` and the tone map
    /// (`render::silhouette::fake_light`).
    #[cfg_attr(not(test), allow(dead_code))]
    FakeLight,
    /// The shader read literally: `CURRENT` on stage 0 the initial
    /// `outColor` of 1, so `t + 127.5` clamped, through `TL_FAKE_LIGHT`
    /// and the tone map.
    #[cfg_attr(not(test), allow(dead_code))]
    Literal,
}

/// The flat green `World::tree_soup` lays, before its face's shade.
pub const FLAT_GREEN: [f32; 3] = [0.25, 0.45, 0.2];

/// A linear 0–1 channel as an sRGB target stores it, 0–255.
pub fn srgb_byte(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    let e = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    e * 255.0
}

impl TreeColour {
    /// The pixel's colour, 0–255, from the leaf's texel and the face it is
    /// drawn on (for [`TreeColour::Flat`]'s shade).
    pub fn colour(self, texel: [f32; 3], face: [[f32; 3]; 3]) -> [f32; 3] {
        use render::silhouette::{TREE_DIFFUSE, fake_light, tree_stage};
        match self {
            TreeColour::Flat => {
                let lit = crate::world::shade(face);
                FLAT_GREEN.map(|v| srgb_byte(v * lit))
            }
            TreeColour::Texel => tree_stage(texel, f32::from(TREE_DIFFUSE)),
            TreeColour::FakeLight => fake_light(tree_stage(texel, f32::from(TREE_DIFFUSE))),
            TreeColour::Literal => fake_light(tree_stage(texel, 255.0)),
        }
    }
}

/// [`TreeColour::colour`] over a painted window (`render::silhouette::
/// paint_cut`'s texels, `tris` the tree's triangles).
pub fn recolour(
    texels: &[Option<Texel>],
    tris: &[[[f32; 3]; 3]],
    how: TreeColour,
) -> Vec<Option<Texel>> {
    texels
        .iter()
        .map(|t| {
            t.map(|mut t| {
                if let Some(face) = tris.get(t.tri as usize) {
                    t.rgb = how.colour(t.rgb, *face);
                }
                t
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::silhouette::{INTERFACE, PHOTO_FRAME, frame_scene, photo_path};
    use crate::world::{TreePixels, World};
    use render::silhouette::{Colours, HUE, LUMA_READER, Object, Photo, Rect, paint_cut};
    use std::path::PathBuf;

    /// One scored tree: its index in `World::tree_list`, its key and its
    /// colours.
    type Row = (usize, u32, Colours);

    /// **The trees' row** (item 4173; the rule, §3.7 "The trees' own
    /// colour", committed before the count): every tree of the frame's
    /// key buffer — `TreePixels::Scene`, the leaves' alpha test and cull,
    /// the silhouettes' pass taking the pixels a figure owns — whose own
    /// pixels off the photograph's [`INTERFACE`] number at least
    /// [`HUE`]'s 40; each such pixel its texel coloured `how` against the
    /// photograph's pixel, under the colour reader and the luminance
    /// reader. No footprint is read off the photograph: the trees stand
    /// in register (4080, 4085), so a tree's pixel is the photograph's.
    fn tree_rows(
        world: &mut World,
        camera: &render::Camera,
        photo: &render::tga::Image,
        how: TreeColour,
    ) -> Vec<Row> {
        let size = (photo.width, photo.height);
        let before = world.tree_pixels;
        world.tree_pixels = TreePixels::Scene;
        let scene = frame_scene(world, camera, size);
        world.tree_pixels = before;
        let p = Photo {
            width: photo.width,
            height: photo.height,
            rgba: &photo.rgba,
        };
        let first_tree = (scene.drawings.len() + scene.sprites.len()) as u32 + 1;
        let trees = world.tree_drawings();
        let cuts = world.tree_cuts();
        let off = |x: i32, y: i32| {
            INTERFACE
                .iter()
                .any(|r| x >= r.x0 && x < r.x1 && y >= r.y0 && y < r.y1)
        };
        let mut out = Vec::new();
        for (j, (d, cut)) in trees.iter().zip(&cuts).enumerate() {
            let key = first_tree + j as u32;
            let Some(cut) = cut.as_ref() else { continue };
            let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
            let mut own = 0u32;
            for y in 0..size.1 {
                for x in 0..size.0 {
                    if scene.keys.at(x, y) == key && !off(x as i32, y as i32) {
                        own += 1;
                        (x0, y0) = (x0.min(x as i32), y0.min(y as i32));
                        (x1, y1) = (x1.max(x as i32 + 1), y1.max(y as i32 + 1));
                    }
                }
            }
            if own < HUE.min_pixels {
                continue;
            }
            let window = Rect::new(x0, y0, x1, y1);
            let object = Object {
                key,
                tris: d.tris.clone(),
            };
            let texels = paint_cut(&object, cut, camera, size, window);
            let ours = recolour(&texels, &d.tris, how);
            let ww = (x1 - x0) as usize;
            let theirs: Vec<Option<bool>> = (0..ours.len())
                .map(|j| {
                    let (x, y) = (x0 + (j % ww) as i32, y0 + (j / ww) as i32);
                    (!off(x, y)).then_some(true)
                })
                .collect();
            let c = render::silhouette::read_colours(
                &scene.keys,
                &p,
                key,
                window,
                &theirs,
                &ours,
                &HUE,
                |_, _| false,
            );
            out.push((j, key, c));
        }
        out
    }

    fn count(rows: &[Row]) -> ((usize, usize), (usize, usize)) {
        (
            (
                rows.iter().filter(|(_, _, c)| c.passes(&HUE)).count(),
                rows.len(),
            ),
            (
                rows.iter()
                    .filter(|(_, _, c)| c.luma_passes(&LUMA_READER))
                    .count(),
                rows.len(),
            ),
        )
    }

    /// **`RUN1559B_3279_TREES`** (item 4173; §3.7, "The trees' own
    /// colour"): the trees on frame 3279 at the photograph's camera, each
    /// candidate of [`TreeColour`] counted by the colour reader and the
    /// luminance reader, and made to fail at run1441's camera.
    #[test]
    fn the_trees_own_colour_on_the_photograph() {
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
        let mut elsewhere = camera;
        (elsewhere.loc_x, elsewhere.loc_y) = (3168.0, 29744.68);
        world.run_to(PHOTO_FRAME);
        world.blend = crate::world::Blend::Last;
        world.run_to(PHOTO_FRAME + 1);
        let list = world.tree_list();
        let mut counts = Vec::new();
        for how in [
            TreeColour::Flat,
            TreeColour::Texel,
            TreeColour::FakeLight,
            TreeColour::Literal,
        ] {
            let rows = tree_rows(&mut world, &camera, &photo, how);
            for (j, key, c) in &rows {
                let [ya, yb] = c.luma_means();
                let t = &list[*j];
                println!(
                    "{how:?} tree {j:>2} key {key:>3} model {:>2} at {:?}: held {:>5} lit {:>5} agree {:>5} ({:.2}) mean {:.3} offset ({:+.3}, {:+.3}) masked {}/{} | Y ours {ya:.1} theirs {yb:.1} | colour {} luma {}",
                    t.model,
                    t.at,
                    c.held,
                    c.lit,
                    c.agree,
                    c.agree as f32 / c.lit.max(1) as f32,
                    c.mean,
                    c.offset[0] / c.rest.max(1) as f32,
                    c.offset[1] / c.rest.max(1) as f32,
                    c.masked_part,
                    c.masked,
                    if c.passes(&HUE) { "in" } else { "out" },
                    if c.luma_passes(&LUMA_READER) {
                        "in"
                    } else {
                        "out"
                    },
                );
            }
            let (colour, luma) = count(&rows);
            println!("{how:?}: colour {colour:?} luma {luma:?}");
            counts.push((how, colour, luma));
        }
        let (off_colour, off_luma) = count(&tree_rows(
            &mut world,
            &elsewhere,
            &photo,
            TreeColour::FakeLight,
        ));
        println!("run1441's camera: colour {off_colour:?} luma {off_luma:?}");
        let drawn = counts
            .iter()
            .find(|(how, _, _)| *how == TreeColour::Flat)
            .unwrap();
        assert_eq!(drawn.1, render::silhouette::RUN1559B_3279_TREES);
        assert_eq!(drawn.2, render::silhouette::RUN1559B_3279_TREES_LUMA);
    }

    fn card_trace_path() -> PathBuf {
        std::env::var_os("RON_CARD_TRACE")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_default();
                PathBuf::from(home).join("ron-golden/run1566b/map-14/rontrace.log")
            })
    }

    #[test]
    fn probe_the_trees_on_the_card() {
        use rondata::trace::card;
        let Ok(bytes) = std::fs::read(card_trace_path()) else {
            println!("skipping: no card trace");
            return;
        };
        let all = card::draws(&bytes);
        let pass: Vec<&card::CardDraw> = all.iter().filter(|d| d.frame == 3279).collect();
        println!("{} draws in the pass", pass.len());
        for d in &pass {
            let n0 = d.file(0).to_ascii_lowercase();
            if !n0.contains("tree") {
                continue;
            }
            println!(
                "draw {:>4} | {} | c0 {:?} c1 {:?} a0 {:?} a1 {:?} | en {:#x} af {} ar {} tf {:?} | opts {}",
                d.index,
                d.file(0),
                d.colour_op(0),
                d.colour_op(1),
                d.alpha_op(0),
                d.alpha_op(1),
                d.enables(),
                d.alpha_func(),
                d.alpha_ref(),
                d.tfactor(),
                d.option_names().join(" "),
            );
        }
    }
}
