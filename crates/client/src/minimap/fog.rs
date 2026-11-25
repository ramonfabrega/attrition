//! **The fog** (item 4624; `docs/RENDERER.md` §3.32.1): one grid of four
//! levels, `WorldMapFog::fill_colors_fog@008d1420`'s, drawn twice.
//!
//! - **The plane** is `sim::Sim::observe_fog`: per fog cell (half a cell
//!   each way), 3 `is_seen`, 2 `was_really_seen`, 1 `was_seen` by the
//!   region rule, 0 never ([`sim::observe::fog`]).
//! - **The main view**: `TerrainOut::render_fow@00854630` draws the
//!   terrain's own vertices once more after the ground and before the
//!   objects, in HDR. Each vertex's `u` is `fog_tcoord_lookup` of the
//!   levels of the fog cells it touches ([`corner_u`],
//!   `TerrainVis::fill_fog_texcoords@008b19f0`), and the pass keeps
//!   `1 − α(u)` of what is under it, `α` the tileset's `fow_fade.tga`
//!   sampled linearly ([`mesh`]; the GPU's `fs_fow` and its compose).
//! - **The minimap**: `WorldMap::render_fog@008cc1f0` multiplies a grey per
//!   fog cell over the background, filtered by point ([`layers`]).
//!
//! The transition (`TerrainOut::inc_fog_time@0086d8e0`) is taken as
//! settled: each `u` is its end value at once.

use render::terrain::TILES_PER_CELL;

/// **`fog_tcoord_lookup`** (`.rdata` `0x00af2df4`, 13 singles, read off the
/// executable): the `u` of a vertex whose touched levels sum, scaled to
/// four cells, to the index. Every step is 1/12 but the first, a half.
pub const TCOORD_LOOKUP: [u32; 13] = [
    0x0000_0000,
    0x3d2a_9931,
    0x3e2a_9931,
    0x3e7f_e5ca,
    0x3eaa_9931,
    0x3ed5_3f7d,
    0x3eff_e5ca,
    0x3f15_460b,
    0x3f2a_9931,
    0x3f3f_ec57,
    0x3f55_3f7d,
    0x3f6a_92a3,
    0x3f80_0000,
];

/// **The minimap's grey by level** (`fill_colors_fog`'s writes to
/// `texture_fog`): 0, `0x50`, `0x80`, 255.
pub const GREY: [u8; 4] = [0, 0x50, 0x80, 0xff];

/// The texture key the fog's ramp is uploaded under, apart from every
/// piece's.
pub const FOW_PIECE: i32 = -4624;

/// The fog cells a corner of the tile grid touches along one axis: one
/// when it lies inside a fog cell (an odd tile index), two when it lies
/// on their boundary, each held to the grid as `set_fog_lookup@008d1300`
/// copies the border out.
fn touched(q: i32, n: i32) -> ([i32; 2], usize) {
    let hold = |v: i32| v.clamp(0, n - 1);
    if q & 1 == 1 {
        ([hold((q - 1) / 2), 0], 1)
    } else {
        ([hold(q / 2 - 1), hold(q / 2)], 2)
    }
}

/// **A vertex's `u`** (`fill_fog_texcoords`): the levels of the fog cells
/// the tile corner `(cx, cy)` touches, each counted `4 / touched`, looked
/// up in [`TCOORD_LOOKUP`]. `plane` is `fog.0 × fog.1`, row-major.
pub fn corner_u(plane: &[u8], fog: (i32, i32), cx: i32, cy: i32) -> f32 {
    let (xs, nx) = touched(cx, fog.0);
    let (ys, ny) = touched(cy, fog.1);
    let mut sum = 0usize;
    for &y in &ys[..ny] {
        for &x in &xs[..nx] {
            sum += usize::from(plane[(y * fog.0 + x) as usize].min(3));
        }
    }
    let index = sum * 4 / (nx * ny);
    f32::from_bits(TCOORD_LOOKUP[index])
}

/// **The main view's fog mesh**: every tile of `terrain` (the full
/// heightfield mesh, [`crate::world::World::terrain`], two triangles a
/// tile in its order `[a, b, d]`, `[b, c, d]`) once more, each corner's
/// `u` [`corner_u`]'s, over `ramp`. `cells` is the map's size; the fog
/// grid is twice it. `None` when the mesh is not whole.
pub fn mesh(
    terrain: &render::terrain::TerrainMesh,
    cells: (i32, i32),
    plane: &[u8],
    ramp: &std::sync::Arc<render::silhouette::Mips>,
) -> Option<crate::world::TexturedMesh> {
    let (tw, th) = (cells.0 * TILES_PER_CELL, cells.1 * TILES_PER_CELL);
    let fog = (cells.0 * 2, cells.1 * 2);
    if terrain.positions.len() != (tw * th * 6) as usize || plane.len() != (fog.0 * fog.1) as usize
    {
        return None;
    }
    let mut uvs = Vec::with_capacity(terrain.positions.len());
    for ty in 0..th {
        for tx in 0..tw {
            let u = |dx: i32, dy: i32| [corner_u(plane, fog, tx + dx, ty + dy), 0.5];
            let (a, b, c, d) = (u(0, 0), u(1, 0), u(1, 1), u(0, 1));
            uvs.extend([a, b, d, b, c, d]);
        }
    }
    Some(crate::world::TexturedMesh {
        piece: FOW_PIECE,
        who: 0,
        positions: terrain.positions.clone(),
        uvs,
        mips: ramp.clone(),
        light: Vec::new(),
    })
}

/// **The minimap's fog** (`render_fog`): `texture_fog` on the background's
/// quad under `ZERO`/`SRCCOLOR`, filtered by point — each fog cell's
/// diamond multiplied by its [`GREY`]. Drawn as the screen pass blends
/// (`c·a + d·(1 − a)`): black at alpha `255 − grey`, one layer a level
/// below 3 (whose grey keeps the whole).
pub fn layers(size: (i32, i32), cells: (i32, i32), plane: &[u8]) -> Vec<crate::gpu::ScreenLayer> {
    let fog = (cells.0 * 2, cells.1 * 2);
    if plane.len() != (fog.0 * fog.1) as usize {
        return Vec::new();
    }
    let extent = (cells.0 * super::CELL, cells.1 * super::CELL);
    let half = super::CELL / 2;
    let (sw, sh) = (size.0 as f32, size.1 as f32);
    let ndc = |x: i32, y: i32| {
        let (px, py) = super::to_screen(size, extent, x, y);
        [px / sw * 2.0 - 1.0, 1.0 - py / sh * 2.0, 0.0]
    };
    let mut out: Vec<crate::gpu::ScreenLayer> = GREY[..3]
        .iter()
        .map(|g| crate::gpu::ScreenLayer {
            alpha: 255 - g,
            pos: Vec::new(),
            col: Vec::new(),
        })
        .collect();
    for fy in 0..fog.1 {
        for fx in 0..fog.0 {
            let level = usize::from(plane[(fy * fog.0 + fx) as usize]);
            let Some(l) = out.get_mut(level) else {
                continue;
            };
            let (x, y) = (fx * half, fy * half);
            let (a, b, c, d) = (
                ndc(x, y),
                ndc(x + half, y),
                ndc(x + half, y + half),
                ndc(x, y + half),
            );
            l.pos.extend([a, b, c, a, c, d]);
            l.col.extend([[0.0; 3]; 6]);
        }
    }
    out.retain(|l| !l.pos.is_empty());
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// `fill_fog_texcoords`' sums on a 2 × 2 fog grid of levels 0, 1, 2
    /// and 3 (one terrain cell): the corner inside a fog cell reads its
    /// level four times, the edge between two reads each twice, and the
    /// centre reads all four once.
    #[test]
    fn a_corner_sums_the_cells_it_touches() {
        let plane = [0, 1, 2, 3];
        let u = |cx, cy| corner_u(&plane, (2, 2), cx, cy);
        let at = |i: usize| f32::from_bits(TCOORD_LOOKUP[i]);
        assert_eq!(u(1, 1), at(0));
        assert_eq!(u(3, 1), at(4));
        assert_eq!(u(1, 3), at(8));
        assert_eq!(u(3, 3), at(12));
        assert_eq!(u(2, 1), at(2), "between 0 and 1, each twice");
        assert_eq!(u(2, 2), at(6), "the centre, each once");
        // The border copied out: the map's corner reads cell (0, 0) four
        // times, its far corner cell (1, 1).
        assert_eq!(u(0, 0), at(0));
        assert_eq!(u(4, 4), at(12));
        // The far edge's two columns are one cell, copied out: 1 and 3,
        // each counted twice (`fill_fog_texcoords`' `L2 + L3 + L6 + L7`).
        assert_eq!(u(4, 2), at(8), "1 and 3, each twice");
    }

    /// The lab directory of item 4624, where the packet's grid, its pixel
    /// rows and the photograph's TGA lie (outside git).
    fn lab(name: &str, var: &str) -> std::path::PathBuf {
        std::env::var_os(var)
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                    .join("ron-data/lab-experiments/2026-11-23-item-4624")
                    .join(name)
            })
    }

    /// The packet's `fog_lookup` on run1814b's frame 3294, read off
    /// `WorldMapFog fog` (`0x00e8a790` `+0x104`, the inner 120 × 120) by
    /// the lab's `foggrid.py`: one row of digits a fog y.
    fn packet_levels() -> Option<Vec<u8>> {
        let path = lab("fog-lookup-run1814b-3294.txt", "RON_FOG_LOOKUP_RUN1814B");
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no fog grid at {}", path.display());
            return None;
        };
        Some(
            text.lines()
                .filter(|l| !l.starts_with('#'))
                .flat_map(|l| l.bytes().map(|b| b - b'0'))
                .collect(),
        )
    }

    /// **The four levels on run1814b** (item 4624; `docs/RENDERER.md`
    /// §3.32.1): the packet's `fog_lookup` counts 13,336 / 727 / 127 / 210
    /// on levels 0 to 3, and slot 0's plane on our walk of the harness's
    /// lobby to frame 3294 ([`sim::Sim::observe_fog`]) is it cell for cell.
    /// Seen red: the plane read off the `seen` and `seen2` bits alone, as
    /// it stood before, called the 727 level-1 cells unseen.
    pub const FOG_LEVELS_RUN1814B: [usize; 4] = [13_336, 727, 127, 210];

    #[test]
    fn the_four_levels_are_the_packet_s() {
        let Some(theirs) = packet_levels() else {
            return;
        };
        let mut r = match crate::input::tests::lobby() {
            Ok(r) => r,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        r.run_to(3294, None, |_| {});
        let ours = r.sim().observe_fog(0).expect("a fog grid");
        assert_eq!((ours.len(), theirs.len()), (14_400, 14_400));
        let count = |p: &[u8]| {
            let mut n = [0usize; 4];
            for &v in p {
                n[usize::from(v)] += 1;
            }
            n
        };
        let parted = ours.iter().zip(&theirs).filter(|(a, b)| a != b).count();
        println!(
            "FOG_LEVELS_RUN1814B: ours {:?} theirs {:?}, {parted} cells part",
            count(&ours),
            count(&theirs)
        );
        assert_eq!(count(&theirs), FOG_LEVELS_RUN1814B);
        assert_eq!(parted, 0, "cell for cell");
    }

    /// One row of the lab's `fowpix-rows.txt` (item 4621's `fowpix.py`):
    /// a fog cell in view above the bar, `(fx, fy, sx, sy, level)`, its
    /// centre projected by the packet's own `MiscAccess::world_to_screen`
    /// at `TerrainOut::find_data_z`'s height under unicorn.
    /// `(fx, fy, sx, sy, level)`.
    type PixelRow = (i32, i32, i32, i32, u8);

    fn pixel_rows() -> Option<Vec<PixelRow>> {
        let path = lab("fowpix-rows.txt", "RON_FOG_PIXELS_RUN1814B");
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no pixel rows at {}", path.display());
            return None;
        };
        Some(
            text.lines()
                .map(|l| {
                    let w: Vec<i32> = l
                        .split_whitespace()
                        .take(6)
                        .map(|v| v.parse().unwrap())
                        .collect();
                    (w[0], w[1], w[3], w[4], w[5] as u8)
                })
                .collect(),
        )
    }

    /// run1814b's camera: the client's pinned one at the packet's `loc`
    /// (item 4612, `docs/RUNS.md` run1814b).
    pub(crate) const RUN1814B_LOC: (f32, f32) = (2639.94, 30287.94);

    /// **`RUN1814B_3294_FOG`** (item 4624; `docs/RENDERER.md` §3.32.1): on
    /// run1814b's frame 3294, each fog cell in view above the bar
    /// ([`pixel_rows`]), by level 1, 2 and 3: `[cells, the photograph's
    /// mean luma, ours]`, the lumas ×10 and rounded (0.299 R + 0.587 G +
    /// 0.114 B). Seen red: with no fog drawn ([`RUN1814B_3294_UNFOGGED`]).
    /// Minted at item 4624's build: ours 142.9 / 143.5 / 141.7 →
    /// 78.6 / 130.5 / 158.3 against theirs 75.0 / 125.2 / 156.4. Level 3
    /// rises with no fog on it: the frame's mean falls, and the tone map's
    /// white with it (§3.21), as theirs does.
    /// Item 4650's plots: 786 → 789, 1305 → 1308, 1583 → 1592, all away
    /// from theirs. Item 4653's plates: → 787 and 1306 (toward) and 1600
    /// (away): the HDR round trip re-tones the frame (§6.1).
    /// Item 4661's plates on their own alpha: → 788 and 1307 (away by a
    /// tenth) and 1593 (toward 1564).
    pub const RUN1814B_3294_FOG: [[i64; 3]; 3] =
        [[80, 750, 788], [93, 1252, 1307], [209, 1564, 1593]];
    /// The same cells on the client's still with no fog drawn, the row as
    /// it stood before item 4624. Each row: the main view's cells at that
    /// fog level, the photograph's median luma, ours (×10). Item 4650's
    /// plots: ours 1429 → 1432 and 1435 → 1438 (away from theirs, 75.0 and
    /// 125.2) and 1417 → 1423 (toward 156.4). Item 4653's plates: 1432 →
    /// 1430, 1438 → 1436 and 1423 → 1431, all toward theirs (their run takes
    /// the land pass's HDR round trip, which re-tones the frame;
    /// `docs/RENDERER.md` §6.2). Item 4661's plates on their own alpha:
    /// 1430 → 1431 and 1436 → 1437 (away, a tenth each) and 1431 → 1424
    /// (away from 156.4): a named fall, the plates' sand where the borders'
    /// ramp laid it before.
    pub const RUN1814B_3294_UNFOGGED: [[i64; 3]; 3] =
        [[80, 750, 1431], [93, 1252, 1437], [209, 1564, 1424]];
    /// **`RUN1814B_3294_FOG_MINIMAP`**: the minimap's fog cells, each at its
    /// centre's pixel through [`super::super::to_screen`], by level 0 to 3:
    /// `[cells, the photograph's median luma, ours]`, ×10 and rounded.
    /// Minted at item 4624's build: ours 148.9 / 152.8 / 154.9 / 146.3 →
    /// 0 / 48.2 / 78.1 / 146.3 against theirs 0 / 47.1 / 74.2 / 144.1.
    /// Item 4627 culls the blips by the console's sight: level 1 48.2 →
    /// 48.0, the oil and the far blips off its cells.
    /// Item 4650's plots: level 3 1463 → 1438 (toward theirs, 1441).
    /// Item 4653's plates: 1438 → 1435, away by 3 (§6.1).
    /// Item 4661's plates on their own alpha: 1435 → 1438, toward theirs.
    pub const RUN1814B_3294_FOG_MINIMAP: [[i64; 3]; 4] = [
        [13_336, 0, 0],
        [727, 471, 480],
        [127, 742, 781],
        [210, 1441, 1438],
    ];
    /// The same with no fog drawn. Item 4627: levels 0 and 1 148.9 →
    /// 148.7 and 152.8 → 152.6, the oils' blips gone (`type_avail`).
    /// Item 4650's plots: level 3 146.3 → 143.8, toward theirs (144.1).
    /// Item 4653's plates: level 0 148.7 → 148.6 (theirs 0), level 3 143.8
    /// → 143.5, away by 0.3: the HDR round trip re-tones the frame (§6.1).
    /// Item 4661's plates on their own alpha: level 0 148.6 → 148.7 (theirs
    /// 0), level 3 143.5 → 143.8, toward theirs (144.1).
    pub const RUN1814B_3294_FOG_MINIMAP_UNFOGGED: [[i64; 3]; 4] = [
        [13_336, 0, 1487],
        [727, 471, 1526],
        [127, 742, 1549],
        [210, 1441, 1438],
    ];

    fn luma(rgba: &[u8], w: usize, x: i32, y: i32) -> f64 {
        let i = (y as usize * w + x as usize) * 4;
        0.299 * f64::from(rgba[i]) + 0.587 * f64::from(rgba[i + 1]) + 0.114 * f64::from(rgba[i + 2])
    }

    #[test]
    fn the_fog_on_run1814b() {
        let (Some(rows), Some(levels)) = (pixel_rows(), packet_levels()) else {
            return;
        };
        let path = lab("run1814b-3294.tga", "RON_PHOTO_RUN1814B");
        let Ok(bytes) = std::fs::read(&path) else {
            println!("skipping: no photograph at {}", path.display());
            return;
        };
        let photo = render::tga::decode(&bytes).expect("the photograph decodes");
        let root = std::env::var_os("RON_INSTALL")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("../../game"));
        let mut world = match crate::world::World::open_game(
            &root,
            &crate::world::gamelog_dir(),
            crate::world::Game::Lobby,
        ) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        world.run_to(3294);
        world.blend = crate::world::Blend::Last;
        world.run_to(3295);
        let mut camera = crate::pinned_camera();
        (camera.loc_x, camera.loc_y) = RUN1814B_LOC;
        let size = (photo.width as u32, photo.height as u32);
        assert_eq!(size, (1920, 1080), "the photograph's frame");
        // Every row's level is the packet's, and ours.
        let plane = {
            world.fog = Some(0);
            world.fog_plane().expect("slot 0's plane")
        };
        let fog_xs = world.runner.terrain().width * 2;
        for &(fx, fy, _, _, level) in &rows {
            let i = (fy * fog_xs + fx) as usize;
            assert_eq!((levels[i], plane[i]), (level, level), "({fx}, {fy})");
        }
        let draw = |world: &mut crate::world::World, fog: bool| {
            world.fog = fog.then_some(0);
            let still = crate::still_minimap(
                world,
                &camera,
                size,
                crate::gpu::SilhouetteOrder::Before,
                wgpu::TextureFormat::Rgba8Unorm,
                &[],
                &|_| None,
                true,
            );
            if let Some(dir) = std::env::var_os("RON_STILL_DIR") {
                let name = format!(
                    "still-run1814b-3294-fog-{}.bmp",
                    if fog { "on" } else { "off" }
                );
                let out = std::path::PathBuf::from(dir).join(name);
                std::fs::write(&out, crate::gpu::bmp(&still, size)).expect("the still writes");
            }
            still
        };
        let w = photo.width;
        let main_row = |ours: &[u8]| -> [[i64; 3]; 3] {
            std::array::from_fn(|k| {
                let level = k as u8 + 1;
                let at: Vec<_> = rows.iter().filter(|r| r.4 == level).collect();
                let mean = |img: &[u8]| {
                    at.iter().map(|r| luma(img, w, r.2, r.3)).sum::<f64>() / at.len() as f64
                };
                [
                    at.len() as i64,
                    (mean(&photo.rgba) * 10.0).round() as i64,
                    (mean(ours) * 10.0).round() as i64,
                ]
            })
        };
        let t = world.runner.terrain();
        let extent = (t.width * super::super::CELL, t.height * super::super::CELL);
        let fog_w = t.width * 2;
        let mini_row = |ours: &[u8]| -> [[i64; 3]; 4] {
            std::array::from_fn(|k| {
                let mut a = Vec::new();
                let mut b = Vec::new();
                for (i, _) in levels
                    .iter()
                    .enumerate()
                    .filter(|(_, v)| usize::from(**v) == k)
                {
                    let (fx, fy) = (i as i32 % fog_w, i as i32 / fog_w);
                    let (x, y) = super::super::to_screen(
                        super::super::QUAD_FRAME,
                        extent,
                        fx * 384 + 192,
                        fy * 384 + 192,
                    );
                    a.push(luma(&photo.rgba, w, x as i32, y as i32));
                    b.push(luma(ours, w, x as i32, y as i32));
                }
                let median = |v: &mut Vec<f64>| {
                    v.sort_by(|p, q| p.total_cmp(q));
                    v[v.len() / 2]
                };
                [
                    a.len() as i64,
                    (median(&mut a) * 10.0).round() as i64,
                    (median(&mut b) * 10.0).round() as i64,
                ]
            })
        };
        let off = draw(&mut world, false);
        let (off_main, off_mini) = (main_row(&off), mini_row(&off));
        let on = draw(&mut world, true);
        let (on_main, on_mini) = (main_row(&on), mini_row(&on));
        println!("RUN1814B_3294_UNFOGGED: {off_main:?}");
        println!("RUN1814B_3294_FOG: {on_main:?}");
        println!("RUN1814B_3294_FOG_MINIMAP_UNFOGGED: {off_mini:?}");
        println!("RUN1814B_3294_FOG_MINIMAP: {on_mini:?}");
        assert_eq!(off_main, RUN1814B_3294_UNFOGGED);
        assert_eq!(off_mini, RUN1814B_3294_FOG_MINIMAP_UNFOGGED);
        assert_eq!(on_main, RUN1814B_3294_FOG);
        assert_eq!(on_mini, RUN1814B_3294_FOG_MINIMAP);
    }

    /// The table as the executable holds it: 0, half a step, then twelfths
    /// to 1.
    #[test]
    fn the_lookup_runs_by_twelfths() {
        let t = TCOORD_LOOKUP.map(f32::from_bits);
        assert_eq!(t[0], 0.0);
        assert_eq!(t[12], 1.0);
        assert!((t[1] - 0.5 / 12.0).abs() < 1e-3);
        for (i, v) in t.iter().enumerate().skip(2).take(10) {
            assert!((v - i as f32 / 12.0).abs() < 1e-3, "{i}: {v}");
        }
    }
}
