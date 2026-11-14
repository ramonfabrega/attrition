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

#[cfg(test)]
use render::silhouette::Texel;

/// **How a tree's pixel is coloured** (item 4173; the candidates the row
/// counts, the rule committed before the count).
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeColour {
    /// The flat green `World::tree_soup` drew before item 4173, shaded by
    /// its face and stored by the sRGB target: the client's still.
    Flat,
    /// The stage colour alone: the leaf's texel under `ADDSIGNED` with the
    /// vertex diffuse (`CURRENT` on stage 0 the diffuse, as Direct3D
    /// defines it), `t + 0x80 − 127.5`.
    Texel,
    /// [`TreeColour::Texel`] through `TL_FAKE_LIGHT` and the tone map
    /// (`render::silhouette::fake_light`).
    FakeLight,
    /// The shader read literally: `CURRENT` on stage 0 the initial
    /// `outColor` of 1, so `t + 127.5` clamped, through `TL_FAKE_LIGHT`
    /// and the tone map.
    Literal,
}

/// **The colour a tree's pixel is drawn in** (item 4173): the candidate
/// the rule picks, most trees agreeing by colour — `RUN1559B_3279_TREES`.
#[cfg(test)]
pub const DRAWN: TreeColour = TreeColour::FakeLight;

/// The flat green `World::tree_soup` lays, before its face's shade.
pub const FLAT_GREEN: [f32; 3] = [0.25, 0.45, 0.2];

/// A linear 0–1 channel as an sRGB target stores it, 0–255.
#[cfg(test)]
pub fn srgb_byte(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    let e = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    e * 255.0
}

#[cfg(test)]
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
#[cfg(test)]
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
        still: Option<&[u8]>,
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
            let mut ours = recolour(&texels, &d.tris, how);
            let ww = (x1 - x0) as usize;
            // On the still (item 4171's harness), the same pixels, our side
            // the client's own bytes.
            if let Some(st) = still {
                for (j, t) in ours.iter_mut().enumerate() {
                    let (x, y) = (x0 as usize + j % ww, y0 as usize + j / ww);
                    let i = (y * size.0 + x) * 4;
                    if let Some(t) = t {
                        t.rgb = [0, 1, 2].map(|k| f32::from(st[i + k]));
                    }
                }
            }
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
            // The value diff (item 4221): the tree's own pixels' mean,
            // ours and the photograph's, on the still.
            if still.is_some() {
                let (mut a, mut b, mut n) = ([0f64; 3], [0f64; 3], 0f64);
                for (k, t) in ours.iter().enumerate() {
                    let (x, y) = (x0 as usize + k % ww, y0 as usize + k / ww);
                    if t.is_none() || off(x as i32, y as i32) || scene.keys.at(x, y) != key {
                        continue;
                    }
                    let i = (y * size.0 + x) * 4;
                    for c in 0..3 {
                        a[c] += f64::from(t.unwrap().rgb[c]);
                        b[c] += f64::from(photo.rgba[i + c]);
                    }
                    n += 1.0;
                }
                let m = |v: [f64; 3]| v.map(|c| (c / n.max(1.0) * 10.0).round() / 10.0);
                println!(
                    "still rgb: tree {j} n {n} ours {:?} theirs {:?}",
                    m(a),
                    m(b)
                );
            }
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
            let rows = tree_rows(&mut world, &camera, &photo, how, None);
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
            None,
        ));
        // Item 4171's harness: the row on the client's still.
        let size = (photo.width as u32, photo.height as u32);
        let still = crate::still_of(
            &mut world,
            &camera,
            size,
            crate::gpu::SilhouetteOrder::Before,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let on_still = tree_rows(&mut world, &camera, &photo, DRAWN, Some(&still));
        let painted = tree_rows(&mut world, &camera, &photo, DRAWN, None);
        let (mut px, mut close) = (0u32, 0u32);
        for ((j, _, a), (_, _, b)) in painted.iter().zip(&on_still) {
            px += a.lit;
            close += u32::from(a.agree.abs_diff(b.agree) * 20 <= a.lit.max(1));
            if a.passes(&HUE) != b.passes(&HUE) {
                println!(
                    "still: tree {j} agree {} of {} painter, {} of {} still",
                    a.agree, a.lit, b.agree, b.lit
                );
            }
        }
        for (j, _, c) in &on_still {
            let [ya, yb] = c.luma_means();
            println!(
                "still: tree {j:>2} Y ours {ya:.1} theirs {yb:.1} ratio {:.3} lit {} agree {}",
                yb / ya.max(1.0),
                c.lit,
                c.agree
            );
        }
        let (still_colour, still_luma) = count(&on_still);
        println!(
            "on the still: colour {still_colour:?} luma {still_luma:?}; {close} of {} trees agree within 5 % of the painter's ({px} lit pixels)",
            painted.len()
        );
        println!("run1441's camera: colour {off_colour:?} luma {off_luma:?}");
        use render::silhouette as s;
        let of = |how: TreeColour| {
            let c = counts.iter().find(|c| c.0 == how).unwrap();
            [c.1, c.2]
        };
        assert_eq!(of(TreeColour::Flat), s::RUN1559B_3279_TREES_FLAT);
        assert_eq!(of(TreeColour::Texel), s::RUN1559B_3279_TREES_TEXEL);
        assert_eq!(of(TreeColour::Literal), s::RUN1559B_3279_TREES_LITERAL);
        assert_eq!(
            of(DRAWN),
            [s::RUN1559B_3279_TREES, s::RUN1559B_3279_TREES_LUMA]
        );
        assert_eq!([off_colour, off_luma], s::RUN1441_CAMERA_3279_TREES);
        assert_eq!([still_colour, still_luma], s::RUN1559B_3279_TREES_STILL);
        // The rule (committed before the count): the candidate most trees
        // agree with by colour is drawn, a tie broken by the luminance
        // count, a second tie keeping the stage colour.
        let pick = counts
            .iter()
            .filter(|c| c.0 != TreeColour::Flat)
            .max_by_key(|c| (c.1.0, c.2.0, c.0 == TreeColour::Texel))
            .unwrap();
        assert_eq!(pick.0, DRAWN, "the rule picks the drawn candidate");
    }

    fn card_trace_path() -> PathBuf {
        std::env::var_os("RON_CARD_TRACE")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_default();
                PathBuf::from(home).join("ron-golden/run1566b/map-14/rontrace.log")
            })
    }

    /// **The trees' draw on the card** (item 4173; §3.7, "The trees' own
    /// colour"): on run1566b's card trace, the pass after frame 3279 draws
    /// the trees one texture a draw — the bark and the six leaf layers —
    /// each under the state [`TreeColour::FakeLight`] reads: stage 0
    /// `ADDSIGNED(TEXTURE, CURRENT)`, stage 1 `MODULATE(CURRENT,
    /// TFACTOR)` at white, alpha `MODULATE(TEXTURE, CURRENT)` then
    /// `SELECTARG2`, the alpha test Greater at 0, `TL_FAKE_LIGHT` and the
    /// vertex diffuse with no `LIGHTING` and no `NO_HDR`.
    #[test]
    fn the_trees_draw_on_the_card() {
        use rondata::trace::card::{
            self, TA_CURRENT, TA_TEXTURE, TA_TFACTOR, TOP_MODULATE, TOP_SELECTARG2,
        };
        const TOP_ADDSIGNED: i32 = 7;
        let Ok(bytes) = std::fs::read(card_trace_path()) else {
            println!("skipping: no card trace at {}", card_trace_path().display());
            return;
        };
        let all = card::draws(&bytes);
        let trees: Vec<&card::CardDraw> = all
            .iter()
            .filter(|d| i64::from(d.frame) == PHOTO_FRAME)
            .filter(|d| {
                let n = d.file(0).to_ascii_lowercase();
                n.starts_with("tree_") && !n.contains("shadow")
            })
            .collect();
        let names: Vec<String> = trees
            .iter()
            .map(|d| d.file(0).to_ascii_lowercase())
            .collect();
        println!("the trees' draws: {names:?}");
        assert_eq!(
            names,
            [
                "tree_t_bark.tga",
                "tree_tea_bottom_a.tga",
                "tree_tea_bottom_b.tga",
                "tree_tea_middle_a.tga",
                "tree_tea_middle_b.tga",
                "tree_tea_middle_c.tga",
                "tree_tea_top_a.tga",
            ]
        );
        for d in &trees {
            assert_eq!(d.colour_op(0), (TOP_ADDSIGNED, TA_TEXTURE, TA_CURRENT));
            assert_eq!(d.colour_op(1), (TOP_MODULATE, TA_CURRENT, TA_TFACTOR));
            assert_eq!(d.alpha_op(0), (TOP_MODULATE, TA_TEXTURE, TA_CURRENT));
            assert_eq!(d.alpha_op(1), (TOP_SELECTARG2, TA_TEXTURE, TA_CURRENT));
            assert_eq!(d.tfactor(), [255; 4]);
            assert_eq!(
                (d.enables(), d.alpha_func(), d.alpha_ref()),
                (0x20087, 4, 0)
            );
            // Item 4221: blended, `set_blend_state`'s `enables & 4`
            // (d3dgl `1002ba07`), `SRCALPHA` over `INVSRCALPHA` — the
            // client's tree pass.
            assert!(d.enables() & 4 != 0);
            assert_eq!((d.src_blend(), d.dst_blend()), (5, 6));
            assert!(d.has("TL_FAKE_LIGHT") && d.has("ENABLE_VERTEX_COLOR_DIFFUSE"));
            assert!(d.has("ENABLE_ALPHA_TEST"));
            assert!(!d.has("ENABLE_LIGHTING") && !d.has("NO_HDR"));
        }
    }

    /// **The trees' textures load tinted** (item 4221; §3.23): `GraphicPieces::tint_color`, which
    /// `GraphicPieces::init_tree_data@008eeb30` hands each tree rdata as
    /// `TintColor` and `TreeTransformMethod::load@00921aa0` sets before its
    /// seven textures load, is on run1559b's and run1564's packets
    /// (`tools/render/tree_tint_packet.py`) the tileset's own tint, the one
    /// `crate::tint::of` gives the land; and the world's tree textures are
    /// the files' texels with it added. Made to fail: untinted.
    #[test]
    fn the_trees_take_the_tileset_s_tint() {
        let install = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let dir = std::env::var_os("RON_TREE_TINT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_default();
                PathBuf::from(home).join("ron-data/lab-experiments/2026-11-13-item-4221")
            });
        let tint = crate::tint::of(&install, "early_autumn");
        if tint == [0; 3] {
            println!("skipping: no tileset tint under {}", install.display());
            return;
        }
        for run in ["run1559b", "run1564"] {
            let path = dir.join(format!("{run}-tree-tint.txt"));
            let Ok(text) = std::fs::read_to_string(&path) else {
                println!("skipping: no tree tint row at {}", path.display());
                return;
            };
            let f: Vec<&str> = text.split_whitespace().collect();
            let at = |k: &str| f[f.iter().position(|x| *x == k).expect("a key") + 1];
            assert_eq!(at("frame"), "3294", "{run}");
            let got: Vec<u8> = at("tree_tint")
                .split(',')
                .map(|v| v.parse().expect("a byte"))
                .collect();
            assert_eq!(
                got[..3],
                tint,
                "{run}: the packet's tree tint is the tileset's"
            );
            assert_ne!(got[..3], [0, 0, 0], "{run}: made to fail, untinted");
        }
        let world = match World::open(&install, &crate::world::gamelog_dir()) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        let (raw, drawn) = world
            .tree_texture_pair("early_autumn", 1)
            .expect("a leaf layer");
        let mut want = raw.clone();
        want.tint(tint);
        assert_eq!(drawn.rgba, want.rgba, "the leaf's texels tinted at load");
        assert_ne!(drawn.rgba, raw.rgba, "made to fail: the file's own texels");
    }
}
