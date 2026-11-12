//! **What stands under things** (item 4182; `docs/RENDERER.md` §3.7, "The
//! shadows"): the row over the tiles the frame's shadow decals darken on
//! the photograph, read on the client's still. The decals themselves are
//! `render::decal` and `World::shadows`; the draw is `client::gpu`'s
//! decal pass.

#[cfg(test)]
mod tests {
    use crate::ground::tests::{Probe, ground_why, probe};
    use crate::world::Shadow;
    use render::silhouette::{HUE, LUMA_READER, chromaticity, luminance};
    use std::collections::BTreeMap;

    /// The darkening past which an undarkened pixel parts
    /// [`LUMA_READER`]: `1 − 1/1.2`.
    const UNDER: f32 = 1.0 - 1.0 / 1.2;

    /// **`RUN1559B_3279_SHADOW`** (item 4182; §3.7, "The shadows", its rule
    /// committed before this count in `53f6cfdfd`): `(agree, scored)` on
    /// [`HUE`], the still's mean against the photograph's over each scored
    /// tile's pixels under a shadow; then `RUN1559B_3279_SHADOW_LUMA`'s on
    /// [`LUMA_READER`]. From [`RUN1559B_3279_SHADOW_BEFORE`]: hue **92 →
    /// 92**, luma **41 → 89** of 105, the trees' decals taken into the
    /// frame before the tone map (`client::gpu`'s shade pass): the tree
    /// tiles 13 → 61 of 68, median ours/theirs 1.29 → 0.93. Tile (8, 183),
    /// 242 pixels under trees: theirs (140, 117, 43) Y 115.9; before (163,
    /// 145, 67) Y 141.5, parts; drawn (99, 88, 41) Y 86.1 when blended after
    /// the tone map, the first draw (24 of 105 with the guys', ours 0.79×
    /// theirs), and agreeing now. The guys' decals are not drawn (parked
    /// 4182c, [`RUN1559B_3279_SHADOW_UNITS`]).
    const RUN1559B_3279_SHADOW: [(usize, usize); 2] = [(92, 105), (89, 105)];
    /// The same rows on the still drawn before item 4182, no decal drawn
    /// (`World::shadows` off): the count as drawn, pinned first. Ours
    /// 1.25× theirs on brightness (median).
    const RUN1559B_3279_SHADOW_BEFORE: [(usize, usize); 2] = [(92, 105), (41, 105)];
    /// The made-to-fail: every decal mirrored in x (`World::shadow_mirror`),
    /// selected and drawn so: luma 59 of 108, the tree tiles 31 of 71
    /// against the drawn 61 of 68.
    const RUN1559B_3279_SHADOW_MIRRORED: [(usize, usize); 2] = [(96, 108), (59, 108)];
    /// The before still composed on the CPU under the rule's own darkening,
    /// the guys' in (nearest texels): through the tone map, the GPU's 66
    /// with the guys' drawn within one (65); in display space, the first
    /// draw's 24 within three (21).
    const RUN1559B_3279_SHADOW_CPU: [[(usize, usize); 2]; 2] =
        [[(91, 105), (65, 105)], [(92, 105), (21, 105)]];
    /// The row with the guys' decals drawn too (`World::unit_shadows`), as
    /// `draw_unit_shadow` reads: luma 89 → 66, the figure tiles 28 → 5 of
    /// 37, median ours/theirs 0.99 → 0.82. Tile (21, 166), 318 pixels
    /// under the guys' decals: theirs (148, 148, 64) Y 138.7, ours (119, 113, 46) Y
    /// 107.1 (parked 4182c).
    const RUN1559B_3279_SHADOW_UNITS: [(usize, usize); 2] = [(92, 105), (66, 105)];
    /// Before, drawn and with the guys': the tree tiles, then the figure
    /// tiles, each `[hue, luma]`.
    const RUN1559B_3279_SHADOW_BY_KIND: [[[(usize, usize); 2]; 2]; 3] = [
        [[(63, 68), (13, 68)], [(29, 37), (28, 37)]],
        [[(63, 68), (61, 68)], [(29, 37), (28, 37)]],
        [[(63, 68), (61, 68)], [(29, 37), (5, 37)]],
    ];

    /// One scored tile: the tile, the photograph's mean, the still's, and
    /// how many of its pixels are under a shadow, by its trees' and its
    /// figures' share of the darkening.
    type ShadowTile = ((i32, i32), [f32; 3], [f32; 3], usize, bool);

    /// Each pixel's combined darkening `1 − Π(1 − α)` under `shadows`, and
    /// whether a tree's decal (rather than only figures') falls on it.
    fn darkening(p: &Probe, shadows: &[Shadow]) -> (Vec<f32>, Vec<bool>) {
        let (w, h) = (p.photo.width, p.photo.height);
        let m = p.camera.to_screen_matrix().0;
        let screen = |q: [f32; 3]| {
            let (x, y, z) = (q[0], q[1], q[2]);
            let mut sx = m[4] * y + m[0] * x + m[8] * z + m[12];
            let mut sy = m[5] * y + m[1] * x + m[9] * z + m[13];
            let ww = m[7] * y + m[3] * x + m[11] * z + m[15];
            if ww != 0.0 && ww != 1.0 {
                sx /= ww;
                sy /= ww;
            }
            [(sx + 1.0) * w as f32 * 0.5, (1.0 - sy) * h as f32 * 0.5]
        };
        let mut keep = vec![1.0f32; w * h];
        let mut tree = vec![false; w * h];
        for s in shadows {
            let Some(tex) = p.world.shadow_texture(s.texture) else {
                continue;
            };
            let img = &tex.levels[0];
            let (pos, uv) = s.decal.triangles();
            let va = f32::from(s.decal.alpha) / 255.0;
            for t in 0..2 {
                let k = [3 * t, 3 * t + 1, 3 * t + 2];
                let v = k.map(|i| screen(pos[i]));
                let u = k.map(|i| uv[i]);
                let x0 = v
                    .iter()
                    .map(|q| q[0])
                    .fold(f32::MAX, f32::min)
                    .floor()
                    .max(0.0) as usize;
                let x1 = (v.iter().map(|q| q[0]).fold(f32::MIN, f32::max).ceil() as usize).min(w);
                let y0 = v
                    .iter()
                    .map(|q| q[1])
                    .fold(f32::MAX, f32::min)
                    .floor()
                    .max(0.0) as usize;
                let y1 = (v.iter().map(|q| q[1]).fold(f32::MIN, f32::max).ceil() as usize).min(h);
                let area = (v[1][0] - v[0][0]) * (v[2][1] - v[0][1])
                    - (v[2][0] - v[0][0]) * (v[1][1] - v[0][1]);
                if area.abs() < 1e-6 {
                    continue;
                }
                for py in y0..y1 {
                    for px in x0..x1 {
                        let c = [px as f32 + 0.5, py as f32 + 0.5];
                        let e = |a: [f32; 2], b: [f32; 2]| {
                            (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])
                        };
                        let l0 = e(v[1], v[2]) / area;
                        let l1 = e(v[2], v[0]) / area;
                        let l2 = e(v[0], v[1]) / area;
                        if l0 < 0.0 || l1 < 0.0 || l2 < 0.0 {
                            continue;
                        }
                        let tu = l0 * u[0][0] + l1 * u[1][0] + l2 * u[2][0];
                        let tv = l0 * u[0][1] + l1 * u[1][1] + l2 * u[2][1];
                        let tx = ((tu * img.width as f32) as usize).min(img.width - 1);
                        let ty = ((tv * img.height as f32) as usize).min(img.height - 1);
                        let a = f32::from(img.rgba[(ty * img.width + tx) * 4 + 3]) / 255.0 * va;
                        let i = py * w + px;
                        keep[i] *= 1.0 - a;
                        if s.texture == crate::world::DECAL_PIECE && a > 0.0 {
                            tree[i] = true;
                        }
                    }
                }
            }
        }
        (keep.into_iter().map(|k| 1.0 - k).collect(), tree)
    }

    /// The row's tiles on `still`: every tile with at least
    /// `HUE.min_pixels` pixels under the frame's shadows, by §3.7's rule.
    fn shadow_tiles(p: &mut Probe, still: &[u8]) -> Vec<ShadowTile> {
        let size = (p.photo.width, p.photo.height);
        let (_, any, why) = ground_why(p);
        let (figures, _, list) = p.world.draw_list(
            &p.camera,
            &crate::pinned_screen(),
            (size.0 as i32, size.1 as i32),
        );
        let shadows = p.world.shadows_of(&figures, &list, true);
        let (dark, tree) = darkening(p, &shadows);
        let px = |rgba: &[u8], i: usize| [0, 1, 2].map(|c| f32::from(rgba[i * 4 + c]));
        type Acc = (Vec<[f32; 3]>, Vec<[f32; 3]>, bool);
        let mut tiles: BTreeMap<(i32, i32), Acc> = BTreeMap::new();
        for i in 0..size.0 * size.1 {
            let Some(tt) = any[i] else { continue };
            if dark[i] < UNDER || !(why[i] == 0 || why[i] == 2) {
                continue;
            }
            let e = tiles.entry(tt).or_default();
            e.0.push(px(&p.photo.rgba, i));
            e.1.push(px(still, i));
            e.2 |= tree[i];
        }
        let mean = |v: &[[f32; 3]]| {
            let n = v.len().max(1) as f32;
            [0, 1, 2].map(|c| v.iter().map(|q| q[c]).sum::<f32>() / n)
        };
        tiles
            .into_iter()
            .filter(|(_, (th, _, _))| th.len() as u32 >= HUE.min_pixels)
            .map(|(tt, (th, us, t))| (tt, mean(&th), mean(&us), th.len(), t))
            .collect()
    }

    /// `T(T⁻¹(g)·k)`: the shade pass's arithmetic on the CPU, `T` the
    /// tone mapper's curve at its live constants (§3.14).
    fn toned(g: f32, k: f32) -> f32 {
        use render::silhouette::{EXPOSURE, MIN_LUMINANCE, WHITE_POINT, filmic};
        let white = 1.0 / filmic(MIN_LUMINANCE * WHITE_POINT);
        let t = |l: f32| (filmic(EXPOSURE * l) * white).clamp(0.0, 1.0);
        let (mut lo, mut hi) = (0.0f32, 64.0f32);
        for _ in 0..60 {
            let m = 0.5 * (lo + hi);
            if t(m) < g { lo = m } else { hi = m }
        }
        t(lo * k)
    }

    fn agrees_hue(a: [f32; 3], b: [f32; 3]) -> bool {
        let (p, q) = (chromaticity(a), chromaticity(b));
        ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt() <= HUE.distance
    }

    /// [`count`] by kind: the tiles a tree's decal falls on, then the rest
    /// (the figures' alone), each `[hue, luma]`.
    fn by_kind(name: &str, tiles: &[ShadowTile]) -> [[(usize, usize); 2]; 2] {
        let (t, f): (Vec<ShadowTile>, Vec<ShadowTile>) = tiles.iter().partition(|q| q.4);
        [
            count(&format!("{name} trees"), &t),
            count(&format!("{name} figures"), &f),
        ]
    }

    /// Both counts, `[hue, luma]`, and the tiles printed with both sides.
    fn count(name: &str, tiles: &[ShadowTile]) -> [(usize, usize); 2] {
        let mut hue = 0;
        let mut luma = 0;
        for (tt, th, us, n, tree) in tiles {
            let h = agrees_hue(*us, *th);
            let l = LUMA_READER.agrees(luminance(*us), luminance(*th));
            hue += usize::from(h);
            luma += usize::from(l);
            println!(
                "{name} tile {tt:?} {} px {n}: theirs ({:.0}, {:.0}, {:.0}) Y {:.1}, the still's ({:.0}, {:.0}, {:.0}) Y {:.1}: hue {h} luma {l}",
                if *tree { "tree" } else { "figure" },
                th[0],
                th[1],
                th[2],
                luminance(*th),
                us[0],
                us[1],
                us[2],
                luminance(*us),
            );
        }
        let n = tiles.len();
        println!("{name}: hue {hue} of {n}, luma {luma} of {n}");
        [(hue, n), (luma, n)]
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

    /// **The shadows' row** (item 4182; §3.7, "The shadows"): the count
    /// as drawn before (no decal), as drawn now (the trees' decals), with
    /// the guys' drawn too, and the made-to-fail; each in total and by kind.
    #[test]
    fn what_stands_under_things() {
        let Some(mut p) = probe() else { return };
        let row = |p: &mut Probe, name: &str| {
            let s = still(p);
            if let Some(dir) = std::env::var_os("RON_STILL_DIR") {
                let size = (p.photo.width as u32, p.photo.height as u32);
                let out = std::path::PathBuf::from(dir).join(format!("still-run1559b-{name}.bmp"));
                std::fs::write(&out, crate::gpu::bmp(&s, size)).expect("the still writes");
            }
            let tiles = shadow_tiles(p, &s);
            (count(name, &tiles), by_kind(name, &tiles), s)
        };
        p.world.shadows = false;
        let (before, before_by, b) = row(&mut p, "before");
        // The selection is the frame's decals, the guys' with them, whether
        // or not they are drawn.
        p.world.shadows = true;
        let (drawn, drawn_by, _) = row(&mut p, "drawn");
        p.world.unit_shadows = true;
        let (units, units_by, _) = row(&mut p, "units");
        p.world.unit_shadows = false;
        p.world.shadow_mirror = true;
        let (mirrored, _, _) = row(&mut p, "mirrored");
        p.world.shadow_mirror = false;
        // The two composes on the CPU over the before still, every decal
        // the rule selects, beside the GPU's with the guys': the decal
        // taken before the tone map, and after it.
        let (figures, _, list) = p.world.draw_list(
            &p.camera,
            &crate::pinned_screen(),
            (p.photo.width as i32, p.photo.height as i32),
        );
        let sh = p.world.shadows_of(&figures, &list, true);
        let (dark, _) = darkening(&p, &sh);
        let composed = |f: &dyn Fn(f32, f32) -> f32| {
            let mut c = b.clone();
            for (i, d) in dark.iter().enumerate() {
                for ch in 0..3 {
                    let g = f32::from(b[i * 4 + ch]) / 255.0;
                    c[i * 4 + ch] = (f(g, 1.0 - d) * 255.0).round() as u8;
                }
            }
            c
        };
        let cpu = [
            count("toned", &shadow_tiles(&mut p, &composed(&toned))),
            count("display", &shadow_tiles(&mut p, &composed(&|g, k| g * k))),
        ];
        assert_eq!(
            [before, drawn, units, mirrored],
            [
                RUN1559B_3279_SHADOW_BEFORE,
                RUN1559B_3279_SHADOW,
                RUN1559B_3279_SHADOW_UNITS,
                RUN1559B_3279_SHADOW_MIRRORED
            ]
        );
        assert_eq!(
            [before_by, drawn_by, units_by],
            RUN1559B_3279_SHADOW_BY_KIND
        );
        assert_eq!(cpu, RUN1559B_3279_SHADOW_CPU);
    }
}
