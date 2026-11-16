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
    /// Item 4227 (the floor's clamped second layer at its offset, §3.12):
    /// run1564 off row 1,412 → 1,565, luma 1,710 → 1,705; on row 1,389 →
    /// 1,543, luma 1,672 → 1,664. Off still beats on: off.
    /// Item 4232 (the lake lit and blended in HDR, §3.24): run1564 off hue
    /// 674 → 678, row 1,565 → 1,706, luma 1,705 → 1,805; on hue 666 → 676,
    /// row 1,543 → 1,686, luma 1,664 → 1,795. Off still beats on: off.
    /// Item 4233: luma run1559b off 1,036 → 1,038, on 1,079 → 1,081;
    /// run1570 off 1,121 → 1,122, on 1,153 → 1,154: the particles drawn and 4201's stand-ins gone (§3.25) move the frame's mean, so its white (§3.21).
    /// Item 4239 (the surf drawn, §3.27): run1564 off row 1,706 → 1,786,
    /// luma 1,805 → 1,851; on row 1,686 → 1,778, luma 1,795 → 1,843. Off
    /// still beats on: off.
    /// Item 4241: off luma run1559b 1,038 → 1,031, run1564 (on 4239's tree)
    /// 1,851 → 1,852, run1570 1,122 → 1,120; on unchanged: the bloom drawn and a plume's pixel clamped after the white (§3.25). Off still beats on: off.
    /// Item 4269: run1559b's luma 1,031 → 1,040 off and 1,081 → 1,086 on, run1570's 1,120 → 1,124 and 1,154 → 1,156: the range lit by its own normals (`docs/RENDERER.md` §3.22), under a mountain 61 → 70 of 106 on run1559b, and the frame's white with it.
    /// Item 4272: run1559b off row 1,104 → 1,105, luma 1,040 → 1,059; on hue 959 → 960, row 1,104 → 1,105, luma 1,086 → 1,101; run1570 off luma 1,124 → 1,138, on 1,156 → 1,164: the template alphas untinted, the tint laid once through `ADDSIGNED` (`docs/RENDERER.md` §3.22). On leads run1559b's grass hue by one (960 against 959, as at item 4221) and trails run1564's and run1570's: off.
    /// Item 4275: run1559b off luma 1,059 → 1,061 and on 1,101 → 1,103, on hue 960 → 959; run1570 off luma 1,138 → 1,140 and on 1,164 → 1,166: the snowcap underlay drawn (`docs/RENDERER.md` §3.10), whose land colour is `light_vert`'s whichever side the switch stands. The switch stays off: on now ties run1559b's grass hue (959) and trails run1564's and run1570's.
    /// Item 4278: run1559b on hue 959 → 960 and luma 1,103 → 1,104; run1570 off luma 1,140 → 1,139 and on 1,166 → 1,165: the mountain's `_mtrl` maps drawn (`docs/RENDERER.md` §3.22), the frame's white with them. The switch stays off: on leads run1559b's grass hue by one and trails run1564's and run1570's.
    /// Item 4279: run1559b off luma 1,061 → 1,066 and on 1,104 → 1,109; run1570 off luma 1,139 → 1,140 and on 1,165 → 1,166: the snowcap's stage 1 drawing its own alpha, not the surf's mask on a shared key (item 4279, `docs/RENDERER.md` §3.10), the frame's white with it. The switch stays off: on leads run1559b's grass hue by one and trails run1564's and run1570's.
    /// Item 4284: off luma run1559b 1,066 → 1,072 and run1570 1,140 → 1,142, on unchanged: the ground beneath a range takes the land's own N·L whichever side the switch stands (`docs/RENDERER.md` §3.22, "The ground beneath a range"), so off now lights those tiles as on does. The switch stays off: on still trails run1564's grass hue and ground row (parked 4284a, the whole ledger).
    pub(crate) const LAND_LIGHT_ROWS: [([usize; 3], [usize; 3]); 3] = [
        ([959, 1105, 1072], [960, 1105, 1109]),
        ([678, 1786, 1852], [676, 1778, 1843]),
        ([1042, 1171, 1142], [1041, 1171, 1166]),
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
    /// Item 4232: the lake lit and blended in HDR (§3.24) moves the frame's
    /// mean, so its white: run1564's untinted 621 → 600, drawn 674 → 678.
    /// Item 4239: the surf drawn (§3.27) moves it again, run1564's
    /// untinted 600 → 602; the drawn side holds.
    /// Item 4241: untinted 961 → 955, 602 → 566 (on 4239's tree), 1,045 → 1,034; the drawn
    /// side holds: the bloom drawn and a plume's pixel clamped after the white (§3.25).
    /// Item 4269: run1564's untinted side 566 → 561: the range lit by its own normals (item 4269, `docs/RENDERER.md` §3.22) moves the frame's mean, so its white.
    /// Item 4275: run1570's untinted side 1,034 → 1,036: the snowcap underlay drawn (item 4275, `docs/RENDERER.md` §3.10) moves the frame's mean, so its white.
    /// Item 4278: run1570's untinted side 1,036 → 1,034: the mountain's `_mtrl` maps drawn on its main pass (item 4278, `docs/RENDERER.md` §3.22) move the frame's mean, so its white.
    /// Item 4284: the untinted side run1559b 955 → 953, run1564 561 → 563; the drawn side holds: the ground beneath a range lit by the land's own N·L (item 4284, `docs/RENDERER.md` §3.22) moves the frame's mean, so its white.
    pub(crate) const GRASS_HUE: [((usize, usize), (usize, usize)); 3] = [
        ((953, 986), (959, 986)),
        ((563, 679), (678, 679)),
        ((1034, 1073), (1042, 1073)),
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

    /// **The mountain's draws on the card** (item 4246; `docs/RENDERER.md`
    /// §3.22): on both card traces' 3279 pass, every draw whose stage 0 is
    /// the tileset's mountain art, with its options and the stream the
    /// probe dumped before it. The probe dumps a stream only when the
    /// executable locks one through `get_internal_stream`; the mountain's
    /// mesh is a static buffer, so neither trace holds its normals or its
    /// vertex colour. The lighting it carries is the land's.
    #[test]
    fn the_mountain_s_draws_on_the_card() {
        use rondata::trace::card::{
            self, TA_CURRENT, TA_DIFFUSE, TA_TEXTURE, TEXADD_CLAMP, TOP_MODULATE, TOP_SELECTARG2,
        };
        // `TextureOp`'s second enumerator (`client::ground`'s tests name it too).
        const TOP_SELECTARG1: i32 = 1;
        let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
        let mut seen = 0;
        for run in ["run1566b", "run1567"] {
            let path = home.join(format!("ron-golden/{run}/map-14/rontrace.log"));
            let Ok(bytes) = std::fs::read(&path) else {
                println!("skipping: no card trace at {}", path.display());
                continue;
            };
            let pass: Vec<card::CardDraw> = card::draws(&bytes)
                .into_iter()
                .filter(|d| d.frame == 3279)
                .collect();
            let streams = pass.iter().filter(|d| d.stream.is_some()).count();
            let mountain: Vec<&card::CardDraw> = pass
                .iter()
                .filter(|d| d.file(0).contains("mountain"))
                .collect();
            println!(
                "{run}: {} draws on the 3279 pass, {streams} with a stream, {} the mountain's",
                pass.len(),
                mountain.len()
            );
            for d in &mountain {
                let l = d.light();
                println!(
                    "{run} draw {}: {} + {}, options {:?}, colour ops {:?} {:?}, light diffuse {:?}, ambient {:?}, material {:?}, stream {:?}",
                    d.index,
                    d.file(0),
                    d.file(1),
                    d.option_names(),
                    d.colour_op(0),
                    d.colour_op(1),
                    &l[2][..3],
                    &d.scene_ambient()[..3],
                    &d.material()[1][..3],
                    d.stream.as_ref().map(|s| (s.vertices, s.stride, s.before)),
                );
                println!(
                    "{run} draw {}: enables {:#x}, blend {} {}, alpha test {} {}, alpha ops {:?} {:?}, address {:?} {:?}, tfactor {:?}",
                    d.index,
                    d.enables(),
                    d.src_blend(),
                    d.dst_blend(),
                    d.alpha_func(),
                    d.alpha_ref(),
                    d.alpha_op(0),
                    d.alpha_op(1),
                    d.address(0),
                    d.address(1),
                    d.tfactor(),
                );
                assert!(
                    d.has("ENABLE_VERTEX_COLOR_DIFFUSE"),
                    "{run} draw {}",
                    d.index
                );
                let peak = d.file(0).starts_with("t_mountainpeak");
                assert_eq!(d.has("ENABLE_LIGHTING"), !peak, "{run} draw {}", d.index);
                assert_eq!(d.has("TL_FAKE_LIGHT"), peak, "{run} draw {}", d.index);
                assert!(d.stream.is_none(), "{run} draw {}: a stream", d.index);
                if peak {
                    // Item 4274: the snowcap underlay, `render_weathering(4, 4)`
                    // (§3.22, "The peak draw"): the colour the texel times the
                    // vertex colour, the alpha stage 1's texel alone, laid
                    // SRCALPHA over INVSRCALPHA with no alpha test, both
                    // stages clamped.
                    assert_eq!(d.enables(), 0x2_0004, "{run} draw {}", d.index);
                    assert_eq!(d.colour_op(0), (TOP_MODULATE, TA_TEXTURE, TA_DIFFUSE));
                    assert_eq!(d.colour_op(1), (TOP_SELECTARG2, TA_DIFFUSE, TA_CURRENT));
                    assert_eq!(d.alpha_op(0), (TOP_SELECTARG2, TA_TEXTURE, TA_DIFFUSE));
                    assert_eq!(d.alpha_op(1), (TOP_SELECTARG1, TA_TEXTURE, TA_CURRENT));
                    assert_eq!((d.src_blend(), d.dst_blend()), (5, 6));
                    assert_eq!(d.address(0), (TEXADD_CLAMP, TEXADD_CLAMP));
                    assert_eq!(d.address(1), (TEXADD_CLAMP, TEXADD_CLAMP));
                    assert_eq!(d.file(1), d.file(0));
                }
            }
            seen += mountain.len();
        }
        println!("{seen} mountain draws");
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
