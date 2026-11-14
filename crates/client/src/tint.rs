//! **The tileset's tint** (item 4212; `docs/RENDERER.md` §3.21): the
//! `<LIGHTING tint_*>` bytes `ImageIO::tint@00511aa0` adds, saturating, to
//! every tileset texture's texels once at load — the ground's bases, the
//! coral, the mountains and the weathering, each of which
//! `TileSet::init@0087b370`, `load_land_data@008767d0` and
//! `load_weathering_data@00875a60` hand `tint_color` through
//! `ImageIO::set_tint@00511a00`. The roads the client draws are
//! `roads.xml`'s pages, not the tileset's `<ROADTEX>`: untinted.

use std::path::Path;

#[cfg(test)]
thread_local! {
    /// The made-to-fail side: textures load untinted on this thread.
    pub(crate) static UNTINTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// The tint `tileset`'s textures take, `[0, 0, 0]` when the tileset names
/// none.
pub fn of(root: &Path, tileset: &str) -> [u8; 3] {
    #[cfg(test)]
    if UNTINTED.with(|u| u.get()) {
        return [0; 3];
    }
    rondata::artdata::tileset_tint(&rondata::Install::new(root), tileset).unwrap_or([0; 3])
}

#[cfg(test)]
pub(crate) mod tests {
    use super::UNTINTED;
    use crate::ground::tests::{
        KINDS, Photo, Probe, kind_of, probe_at, run1559b, run1564, still_ground_row,
    };
    use std::collections::BTreeSet;

    fn photos() -> [(&'static str, Photo); 3] {
        [
            ("run1559b", run1559b()),
            ("run1564", run1564()),
            ("run1570", crate::weathering::tests::run1570()),
        ]
    }

    fn still(p: &mut Probe) -> Vec<u8> {
        let size = (p.photo.width as u32, p.photo.height as u32);
        let camera = p.camera;
        crate::still_of(
            &mut p.world,
            &camera,
            size,
            crate::gpu::SilhouetteOrder::Before,
            wgpu::TextureFormat::Rgba8Unorm,
        )
    }

    /// HSV hue in degrees.
    pub(crate) fn hue(c: [f32; 3]) -> f32 {
        let (r, g, b) = (c[0], c[1], c[2]);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;
        if d <= 0.0 {
            return 0.0;
        }
        let h = if max == r {
            ((g - b) / d).rem_euclid(6.0)
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
        h * 60.0
    }

    /// **The grass's hue row's bar** (item 4212, committed before its
    /// count): a `plain` tile agrees when the HSV hue of the still's mean
    /// is the photograph's within this many degrees.
    pub(crate) const GRASS_HUE_BAR: f32 = 4.0;

    /// One photograph's grass: `(agree, scored)` on [`GRASS_HUE_BAR`],
    /// then the means over the tiles, theirs and ours.
    pub(crate) type GrassRow = ((usize, usize), [f32; 3], [f32; 3]);

    fn grass_row(name: &str, shot: Photo, untinted: bool) -> Option<GrassRow> {
        UNTINTED.with(|u| u.set(untinted));
        let probe = probe_at(shot);
        UNTINTED.with(|u| u.set(false));
        let mut p = probe?;
        let s = still(&mut p);
        let tiles = still_ground_row(&mut p, &s);
        let t = p.world.runner.terrain();
        let mountains: BTreeSet<(i32, i32)> =
            t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
        let mut by = vec![([0f64; 3], [0f64; 3], 0u32, Vec::new()); KINDS.len()];
        for (tt, th, us, _) in &tiles {
            let k = kind_of(&t, &mountains, *tt);
            let e = &mut by[k];
            for c in 0..3 {
                e.0[c] += f64::from(th[c]);
                e.1[c] += f64::from(us[c]);
            }
            e.2 += 1;
            e.3.push((*tt, *th, *us));
        }
        let side = if untinted { "untinted" } else { "tinted" };
        let mut row = None;
        for (k, (th, us, n, v)) in by.iter().enumerate() {
            if *n == 0 {
                continue;
            }
            let th = th.map(|x| (x / f64::from(*n)) as f32);
            let us = us.map(|x| (x / f64::from(*n)) as f32);
            let agree = v
                .iter()
                .filter(|(_, a, b)| (hue(*a) - hue(*b)).abs() <= GRASS_HUE_BAR)
                .count();
            println!(
                "{name} {side} {}: {agree} of {n} on hue, theirs ({:.1}, {:.1}, {:.1}) hue {:.1}, ours ({:.1}, {:.1}, {:.1}) hue {:.1}, ratio ({:.3}, {:.3}, {:.3})",
                KINDS[k],
                th[0],
                th[1],
                th[2],
                hue(th),
                us[0],
                us[1],
                us[2],
                hue(us),
                th[0] / us[0],
                th[1] / us[1],
                th[2] / us[2],
            );
            if k == 0 {
                if let Some((tt, a, b)) = v.first() {
                    println!(
                        "{name} {side} plain, first tile {tt:?}: theirs ({:.1}, {:.1}, {:.1}) hue {:.1}, ours ({:.1}, {:.1}, {:.1}) hue {:.1}",
                        a[0],
                        a[1],
                        a[2],
                        hue(*a),
                        b[0],
                        b[1],
                        b[2],
                        hue(*b)
                    );
                }
                row = Some(((agree, *n as usize), th, us));
            }
        }
        row
    }

    /// **`GRASS_HUE`** (item 4212; `docs/RENDERER.md` §3.21): the grass
    /// tiles' hue row on run1559b, run1564 and run1570, untinted (the
    /// made-to-fail side) and with the tileset's tint.
    #[test]
    fn the_grass_s_hue_on_three_photographs() {
        let mut got = Vec::new();
        for (name, shot) in photos() {
            let Some(before) = grass_row(name, shot, true) else {
                return;
            };
            let shot = photos().into_iter().find(|p| p.0 == name).expect("named").1;
            let Some(after) = grass_row(name, shot, false) else {
                return;
            };
            got.push((before.0, after.0));
        }
        println!("GRASS_HUE: {got:?}");
        assert_eq!(got, GRASS_HUE);
    }

    /// **The land lit by its normal** (item 4217; `docs/RENDERER.md`
    /// §3.22): on each photograph, the still with [`crate::world::World`]'s
    /// land light off (item 4212's flat N·L, the drawn side) and on (each
    /// corner's `calculate_norms` N·L and `light_vert` colour): the grass's
    /// hue row on [`GRASS_HUE_BAR`], the ground row's verdicts and its
    /// luminance row over every scored tile. The off side is the other
    /// tests' pins read again.
    #[test]
    fn the_land_lit_by_its_normal() {
        use render::silhouette::{LUMA_READER, luminance};
        let mut got = Vec::new();
        for (name, shot) in photos() {
            let Some(mut p) = probe_at(shot) else { return };
            let mut sides = Vec::new();
            for lit in [false, true] {
                p.world.set_land_light(lit);
                let s = still(&mut p);
                let tiles = still_ground_row(&mut p, &s);
                let t = p.world.runner.terrain();
                let mountains: BTreeSet<(i32, i32)> =
                    t.mountains.iter().flatten().map(|q| (q.x, q.y)).collect();
                let (mut hue_ok, mut row, mut luma) = (0, 0, 0);
                let mut mean = [[0f64; 3]; 2];
                let plain = tiles
                    .iter()
                    .filter(|q| kind_of(&t, &mountains, q.0) == 0)
                    .count();
                for (tt, th, us, ok) in &tiles {
                    row += usize::from(*ok);
                    luma += usize::from(LUMA_READER.agrees(luminance(*us), luminance(*th)));
                    if kind_of(&t, &mountains, *tt) == 0 {
                        hue_ok += usize::from((hue(*th) - hue(*us)).abs() <= GRASS_HUE_BAR);
                        for c in 0..3 {
                            mean[0][c] += f64::from(th[c]) / plain as f64;
                            mean[1][c] += f64::from(us[c]) / plain as f64;
                        }
                    }
                }
                println!(
                    "{name} land light {}: grass hue {hue_ok} of {plain}, ground row {row} and luma {luma} of {}; the grass theirs ({:.1}, {:.1}, {:.1}), ours ({:.1}, {:.1}, {:.1})",
                    if lit { "on" } else { "off" },
                    tiles.len(),
                    mean[0][0],
                    mean[0][1],
                    mean[0][2],
                    mean[1][0],
                    mean[1][1],
                    mean[1][2],
                );
                sides.push([hue_ok, row, luma]);
            }
            got.push((sides[0], sides[1]));
        }
        assert_eq!(got, LAND_LIGHT_ROWS);
    }

    /// [`the_land_lit_by_its_normal`]'s counts, `(off, on)` on run1559b,
    /// run1564 and run1570, each `[grass hue, ground row, luma]`. Off is
    /// `GRASS_HUE`'s drawn side, `RUN1559B_3279_GROUND_STILL`,
    /// `RUN1564_3279_GROUND_STILL` and `GROUND_LUMA_STILL`. On, the luma
    /// rises on all three (1,029 → 1,079, 1,396 → 1,410, 1,118 → 1,152) and
    /// run1564's grass hue and ground row fall (674 → 665, 1,021 → 1,001,
    /// its `SANDY` 311 → 290): the switch stays off (§3.22).
    /// Item 4221 (the trees tinted and blended, §3.23): run1559b off luma
    /// 1,029 → 1,036, on hue 960 → 959; run1570 off luma 1,118 → 1,121, on
    /// 1,152 → 1,153. Lit still beats unlit on no hue row: off.
    /// Item 4223 (run1564's photograph matched to sRGB, and the coral
    /// layer drawn, §3.18): run1564 off row 1,021 → 1,412, luma 1,396 →
    /// 1,710; on hue 665 → 666, row 1,001 → 1,389, luma 1,410 → 1,672. Off
    /// still beats on: off.
    pub(crate) const LAND_LIGHT_ROWS: [([usize; 3], [usize; 3]); 3] = [
        ([959, 1104, 1036], [959, 1104, 1079]),
        ([674, 1412, 1710], [666, 1389, 1672]),
        ([1042, 1171, 1121], [1041, 1171, 1153]),
    ];

    /// **`GRASS_HUE`**, `[(untinted, tinted)]` on run1559b, run1564 and
    /// run1570, both sides lit at the frame's white. At the base (`06e08d98`,
    /// the texel unlit) the row read 17 of 986, 1 of 679 and 17 of 1,073
    /// under its bar; tile (0, 140) on run1559b, theirs (165.9, 162.4, 68.6)
    /// hue 57.9, ours (160.5, 138.9, 64.8) hue 46.4. The tint alone, unlit,
    /// read 16, 1 and 17. Drawn: 959, 674 and 1,042. On the tree merged with
    /// 4208 (the gaia figures' shipped variant), run1564's untinted 605 → 607.
    /// Item 4221: the untinted side now untints the trees too (§3.23):
    /// run1564's 607 → 605, run1570's 1,044 → 1,045; the drawn side holds.
    /// Item 4223: run1564's photograph matched to sRGB (§3.18), its
    /// untinted side 605 → 621; the drawn side holds at 674.
    pub(crate) const GRASS_HUE: [((usize, usize), (usize, usize)); 3] = [
        ((961, 986), (959, 986)),
        ((621, 679), (674, 679)),
        ((1045, 1073), (1042, 1073)),
    ];
}

#[cfg(test)]
mod card {
    /// **The land is lit on the card** (item 4212; §3.21): on run1567's
    /// 3279 pass, every land draw (`grass`, `dirt`, `sand` and the patches
    /// over them) carries `ENABLE_LIGHTING` with the vertex diffuse, stage
    /// 0 `MODULATE(TEXTURE, DIFFUSE)`, the light's diffuse
    /// [`render::silhouette::LAND_DIFFUSE`] along (−0.707, 0, −0.707), the
    /// material white, and the scene's ambient
    /// [`render::silhouette::SCENE_AMBIENT`]: the constants `client::gpu`'s
    /// land shader is filled with.
    #[test]
    fn the_land_is_lit_on_the_card() {
        use render::silhouette::{LAND_DIFFUSE, SCENE_AMBIENT};
        use rondata::trace::card::{self, TA_DIFFUSE, TA_TEXTURE, TOP_MODULATE};
        let path = std::env::var_os("RON_CARD_TRACE_1567")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_default();
                std::path::PathBuf::from(home).join("ron-golden/run1567/map-14/rontrace.log")
            });
        let Ok(bytes) = std::fs::read(&path) else {
            println!("skipping: no card trace at {}", path.display());
            return;
        };
        let land: Vec<card::CardDraw> = card::draws(&bytes)
            .into_iter()
            .filter(|d| d.frame == 3279)
            .filter(|d| {
                let f = d.file(0);
                f.starts_with("grass_") || f.starts_with("dirt_") || f.starts_with("sand")
            })
            .collect();
        println!("{} land draws", land.len());
        assert!(land.len() >= 36, "the base layer's 36 draws at least");
        let near = |a: f32, b: f32| (a - b).abs() < 1e-4;
        for d in &land {
            assert!(d.has("ENABLE_LIGHTING"), "{} lit", d.file(0));
            assert!(d.has("ENABLE_VERTEX_COLOR_DIFFUSE"), "{}", d.file(0));
            assert!(!d.has("TL_FAKE_LIGHT") && !d.has("NO_HDR"), "{}", d.file(0));
            assert_eq!(d.colour_op(0), (TOP_MODULATE, TA_TEXTURE, TA_DIFFUSE));
            let l = d.light();
            assert!(near(l[0][0], -std::f32::consts::FRAC_1_SQRT_2) && near(l[0][2], l[0][0]));
            assert!(
                l[2][..3].iter().all(|&c| near(c, LAND_DIFFUSE)),
                "{:?}",
                l[2]
            );
            let m = d.material();
            assert!(m[0][..3].iter().chain(&m[1][..3]).all(|&c| near(c, 1.0)));
            let a = d.scene_ambient();
            assert!((0..3).all(|k| near(a[k], SCENE_AMBIENT[k])), "{a:?}");
        }
    }
}

#[cfg(test)]
mod image {
    /// `ImageIO::tint@00511aa0`'s add saturates per byte and leaves alpha.
    #[test]
    fn the_tint_saturates_and_leaves_alpha() {
        let mut i = render::tga::Image {
            width: 2,
            height: 1,
            kind: 2,
            bits: 32,
            footer: false,
            rgba: vec![10, 250, 0, 7, 240, 0, 255, 255],
        };
        i.tint([18, 18, 0]);
        assert_eq!(i.rgba, vec![28, 255, 0, 7, 255, 18, 255, 255]);
    }
}
