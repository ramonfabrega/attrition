//! **A farm's ground: the wheat plots** (items 4636, 4650; `docs/RENDERER.md`
//! §6.1).
//!
//! Two draws lay a farm's plots on run1814b's packet:
//! - `WallOut::draw_weathering@0063dde0` lays the tileset's `wheatfarm`
//!   underlay (`<UNDERLAYS>`, `farm_wheat.tga`) over the farm's 4 × 4 tiles
//!   through `TerrainOut::add_weather_underlay@00874d00`, for a farm whose
//!   record's `farm_type & 1` is clear; the pasture takes `pigfarm`
//!   ([`underlay_quads`]);
//! - `FarmsOut::draw@008d75f0` builds one box per plot, the 4 × 4 grid of
//!   the farm's `FarmStruct`, from the box template `build_box@008d6140`
//!   builds, its height `percent` of the full and its foot on
//!   `terrain_height`, coloured by `percent`, and skips the pasture
//!   ([`wheat`]).
//!
//! [`corner`] is where both start: the farm's tile less half its type's
//! size, as `FarmsOut::draw` snaps it. The client draws in `f32`: these are
//! the original's single-precision sums, and the client is not the
//! simulation.

use std::sync::Arc;

use render::silhouette::Mips;
use sim::observe::{BuildingScene, FieldScene};

/// World units on a tile's side, `0xc0`.
const TILE: i32 = 192;

/// **The farm's corner tile** (`FarmsOut::draw@008d75f0`): the tile of its
/// point (`div_3_table[x >> 6]`, `x / 192` for a point on the map) less
/// half its type's size on each axis, `size >> 1`. The plots' boxes stand
/// at its tiles' centres.
pub fn corner(b: &BuildingScene, f: &FieldScene) -> (i32, i32) {
    (
        b.pos.x.div_euclid(TILE) - (f.x_size >> 1),
        b.pos.y.div_euclid(TILE) - (f.y_size >> 1),
    )
}

/// The farm's footprint tiles, `x_size × y_size` from its [`corner`].
#[cfg_attr(not(test), allow(dead_code))] // the plots' build is its caller
pub fn tiles(b: &BuildingScene, f: &FieldScene) -> impl Iterator<Item = (i32, i32)> + use<> {
    let (cx, cy) = corner(b, f);
    let (sx, sy) = (f.x_size, f.y_size);
    (0..sy).flat_map(move |j| (0..sx).map(move |i| (cx + i, cy + j)))
}

/// **`build_box@008d6140`'s template**, `wheat_box_verts`: a ±96 top quad
/// at z 1 and three sides down to z 0. Run1814b's packet holds the same ten
/// (`tests::the_template_is_the_packet_s`).
pub const BOX_VERTS: [[f32; 3]; 10] = [
    [-96.0, -96.0, 1.0],
    [96.0, -96.0, 1.0],
    [-96.0, 96.0, 1.0],
    [96.0, 96.0, 1.0],
    [-96.0, 96.0, 1.0],
    [-96.0, 96.0, 0.0],
    [96.0, 96.0, 1.0],
    [96.0, 96.0, 0.0],
    [96.0, -96.0, 1.0],
    [96.0, -96.0, 0.0],
];

/// `wheat_box_faces`: the top, then the +y and +x sides.
pub const BOX_FACES: [usize; 18] = [0, 1, 2, 2, 1, 3, 4, 6, 5, 5, 6, 7, 6, 8, 7, 7, 8, 9];

/// `wheat_box_cols`, `wheat_box_rows`: the `terrain_height` corner each
/// template vertex stands on, from the plot's own.
pub const BOX_COLS: [usize; 10] = [0, 1, 0, 1, 0, 0, 1, 1, 1, 1];
pub const BOX_ROWS: [usize; 10] = [0, 0, 1, 1, 1, 1, 1, 1, 0, 0];

/// `wheat_use_wind`: the vertices the wind moves, every one at z 1.
pub const BOX_WIND: [bool; 10] = [
    true, true, true, true, true, false, true, false, true, false,
];

/// `wheat_box_texs[2]`: the texcoords of a plot whose `(col + row)` is
/// even, then odd.
pub const BOX_TEXS: [[[f32; 2]; 10]; 2] = [
    [
        [0.5, 0.0],
        [0.5, 0.5],
        [0.0, 0.0],
        [0.0, 0.5],
        [0.0, 0.5],
        [0.0, 1.0],
        [0.5, 0.5],
        [0.5, 1.0],
        [1.0, 0.5],
        [1.0, 1.0],
    ],
    [
        [1.0, 0.0],
        [1.0, 0.5],
        [0.5, 0.0],
        [0.5, 0.5],
        [1.0, 0.5],
        [1.0, 1.0],
        [0.5, 0.5],
        [0.5, 1.0],
        [0.0, 0.5],
        [0.0, 1.0],
    ],
];

/// `Farms::init@008d9310` keeps `min_height` and `max_height` × 3.0
/// (`__real_40400000`): run1814b's `farms + 0x50`, `+ 0x4c` read 30 and
/// 108 for the install's 10 and 36.
pub const HEIGHT_SCALE: f32 = 3.0;

/// **`Scene::wind_vec`** (`+0x2e8`) on run1814b's packet, as single bits.
/// The scene's wind is the view's, not the simulation's, and its motion
/// over a game is not read (§6.1): the sway it gives is about one world
/// unit.
pub const WIND_VEC: [u32; 2] = [0x3863_a101, 0x3887_a38d];

/// The sway's scale (`___real_46c35000`).
pub const WIND_SWAY: f32 = 25_000.0;

/// **A cell's `percent`** as the original holds it: a growing or ripe cell's
/// `adds` sums of `0.005f` from zero (a ripe cell is clamped to exactly
/// `1.0f`); a cut cell's `1.0f` less `0.01f` once a frame, `(FULL − adds) /
/// 2` times (`sim::cities::farms`).
pub fn percent(adds: i32, status: u8) -> f32 {
    use sim::cities::farms::{CUT, FULL, RIPE};
    if status == RIPE {
        return 1.0;
    }
    if status == CUT {
        let mut v = 1.0f32;
        for _ in 0..(FULL - adds).max(0) / 2 {
            v -= 0.01;
        }
        return v;
    }
    let mut v = 0.0f32;
    for _ in 0..adds.max(0) {
        v += 0.005;
    }
    v
}

/// **A plot's colour** (`FarmsOut::draw`, the wind arm), RGBA bytes:
/// `start_color` (the `<FARM>`'s stalk) below a quarter, its alpha raised
/// toward 255 by `percent · 4` from its own times `share` (the build's,
/// 1 on a finished farm); from a quarter to three quarters its colour
/// toward white by `2 · (percent − ¼)` and opaque; past that white. At
/// exactly a quarter, the stalk as it stands. `get_global_color` is read
/// as the identity: run1814b's 480 colours are these.
pub fn plot_colour(stalk: [u8; 4], p: f32, share: f32) -> [u8; 4] {
    let mut c = stalk;
    if 0.25 < p {
        if p <= 0.75 {
            let f = (p - 0.25) + (p - 0.25);
            for k in 0..3 {
                c[k] = ((255 - i32::from(stalk[k])) as f32 * f + f32::from(stalk[k])) as u32 as u8;
            }
            c[3] = 255;
        } else {
            c = [255; 4];
        }
    }
    if p < 0.25 {
        let a = f32::from(stalk[3]);
        c[3] = ((255.0 - a) * p * 4.0 + a * share) as i32 as u8;
    }
    c
}

/// The wheat's art: `TileSetData::wheat_texture` (the tileset's `FARMTEX`,
/// `PF_32`, untinted: `TileSet::init@0087b370` hands it no `set_tint`), the
/// stalk colour and the heights.
pub struct WheatArt {
    pub mips: Arc<Mips>,
    pub stalk: [u8; 4],
    /// `wheat_min_height`, `wheat_max_height`.
    pub heights: [f32; 2],
}

impl WheatArt {
    /// `None` where `rondata::artdata::farm_art` or the texture does not
    /// read.
    pub fn load(install: &rondata::Install, tileset: &str) -> Option<WheatArt> {
        let art = rondata::artdata::farm_art(install, tileset).ok()?;
        let bytes =
            std::fs::read(rondata::artdata::resolve_art(install.root(), &art.wheat)?).ok()?;
        let img = render::tga::decode(&bytes).ok()?;
        Some(WheatArt {
            mips: Arc::new(Mips::new(img)),
            stalk: art.stalk,
            heights: art.heights.map(|h| h as f32 * HEIGHT_SCALE),
        })
    }
}

/// **The frame's wheat boxes**, one triangle list: per corner its position,
/// texcoord and RGBA colour (each byte over 255).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WheatMesh {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub colours: Vec<[f32; 4]>,
}

/// **One plot's box** (`FarmsOut::draw`, the wind arm): its ten vertices,
/// the colour and texcoord set, at `(col, row)` of the farm at `corner`,
/// on the game's `frame`. The plot's centre is the corner tile's plus
/// `(col, row)` tiles, each axis summed in single precision; z is the
/// template's times `(max − min) · percent + min` plus `terrain_height` at
/// the vertex's own corner; the wind moves the top by
/// `(tri / 16 − 1) · 25000 · percent` along `wind`, `tri` the triangle wave
/// of `(frame + 5 · row + 17 · col + corner_y + corner_x) & 63`.
pub fn plot(
    f: &FieldScene,
    corner: (i32, i32),
    (col, row): (usize, usize),
    heights: [f32; 2],
    wind: [f32; 2],
    frame: i64,
) -> [[f32; 3]; 10] {
    let cell = col * 4 + row;
    let p = percent(f.adds[cell], f.status[cell]);
    let full = (heights[1] - heights[0]) * p + heights[0];
    let x0 = (corner.0 * 192 + 0x60) as f32;
    let y0 = (corner.1 * 192 + 0x60) as f32;
    let cx = (col as i32 * 192) as f32 + x0;
    let cy = (row as i32 * 192) as f32 + y0;
    let phase =
        (frame + 5 * row as i64 + 17 * col as i64 + i64::from(corner.1) + i64::from(corner.0))
            & 0x3f;
    let tri = if phase < 0x20 { phase } else { 0x40 - phase };
    let sway = (tri as f32 * 0.0625 - 1.0) * WIND_SWAY * p;
    let (dx, dy) = (wind[0] * sway, wind[1] * sway);
    let mut out = [[0.0f32; 3]; 10];
    for (k, v) in BOX_VERTS.iter().enumerate() {
        let ground = f.heights[(col + BOX_COLS[k]) * 5 + row + BOX_ROWS[k]] as f32;
        let mut x = cx + v[0];
        let mut y = v[1] + cy;
        if BOX_WIND[k] {
            x += dx;
            y += dy;
        }
        out[k] = [x, y, v[2] * full + ground];
    }
    out
}

/// **The wheat the frame draws** (`FarmsOut::draw`'s walk, the wind arm):
/// every field of [`fields`], its sixteen plots row by row, each a box of
/// [`plot`], [`plot_colour`] at a finished farm's share and `BOX_TEXS` by
/// the plot's parity, expanded to `BOX_FACES`' triangles.
pub fn wheat(buildings: &[BuildingScene], art: &WheatArt, frame: i64) -> WheatMesh {
    let wind = WIND_VEC.map(f32::from_bits);
    let mut m = WheatMesh::default();
    for b in buildings.iter().filter(|b| b.seen) {
        let Some(f) = b
            .field
            .filter(|f| f.farm_type != sim::cities::farms::ANIMAL_FARM)
        else {
            continue;
        };
        let c = corner(b, &f);
        for row in 0..4 {
            for col in 0..4 {
                let cell = col * 4 + row;
                let p = percent(f.adds[cell], f.status[cell]);
                let rgba = plot_colour(art.stalk, p, 1.0).map(|v| f32::from(v) / 255.0);
                let verts = plot(&f, c, (col, row), art.heights, wind, frame);
                let texs = &BOX_TEXS[(col + row) & 1];
                for &k in &BOX_FACES {
                    m.positions.push(verts[k]);
                    m.uvs.push(texs[k]);
                    m.colours.push(rgba);
                }
            }
        }
    }
    m
}

/// The tileset's `pigfarm` and `wheatfarm` underlays, tinted as every
/// underlay is (`TileSet::init@0087b370`).
pub struct UnderlayArt {
    pub pig: Arc<Mips>,
    pub wheat: Arc<Mips>,
}

impl UnderlayArt {
    /// `None` where the tileset names either underlay not, or a texture
    /// does not read.
    pub fn load(install: &rondata::Install, tileset: &str, tint: [u8; 3]) -> Option<UnderlayArt> {
        let (_, list) = rondata::artdata::underlay_textures(install, tileset).ok()?;
        let read = |id: &str| -> Option<Arc<Mips>> {
            let file = &list.iter().find(|(k, _)| k.eq_ignore_ascii_case(id))?.1;
            let bytes = std::fs::read(rondata::artdata::resolve_art(install.root(), file)?).ok()?;
            let mut img = render::tga::decode(&bytes).ok()?;
            img.tint(tint);
            Some(Arc::new(Mips::new(img)))
        };
        Some(UnderlayArt {
            pig: read("pigfarm")?,
            wheat: read("wheatfarm")?,
        })
    }
}

/// **The farms' underlays** (`WallOut::draw_weathering`'s farm arm), the
/// `pigfarm` patches then the `wheatfarm` ones, as `render_weathering(0, 3)`
/// draws class 0 before class 1: `add_weather_underlay(corner, 4, 4, …, 0,
/// 0, 0)` on each seen farm with a record — no variant, a step of one, no
/// odd-corner step, so UV0 and UV1 are `i / 4` ([`crate::oil::uv`]) and
/// every vertex's alpha byte 255. A pasture (`farm_type & 1`) takes
/// `pigfarm` at the alpha `draw_weathering` leaves it, 255.
pub fn underlay_quads(
    buildings: &[BuildingScene],
) -> (Vec<crate::oil::Quad>, Vec<crate::oil::Quad>) {
    let (mut pig, mut wheat) = (Vec::new(), Vec::new());
    for b in buildings.iter().filter(|b| b.seen) {
        let Some(f) = b.field else {
            continue;
        };
        let q = crate::oil::Quad {
            corner: corner(b, &f),
            side: 4,
            variant: None,
        };
        if f.farm_type & 1 != 0 {
            pig.push(q);
        } else {
            wheat.push(q);
        }
    }
    (pig, wheat)
}

/// **The fields the frame lays** (`FarmsOut::draw`'s walk): every
/// building with a farm record that is not the pasture, its owner and
/// corner.
#[cfg_attr(not(test), allow(dead_code))] // the plots' build is its caller
pub fn fields(buildings: &[BuildingScene]) -> Vec<(i32, (i32, i32), FieldScene)> {
    buildings
        .iter()
        .filter_map(|b| {
            let f = b.field?;
            (f.farm_type != sim::cities::farms::ANIMAL_FARM)
                .then(|| (i32::from(b.owner), corner(b, &f), f))
        })
        .collect()
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use crate::ground::tests::{Probe, ground_why};
    use crate::weathering::tests::{count, plaza_tiles, still};

    /// **Slot 0's three farms on run1814b's packet** (item 4636; the lab's
    /// `farm_probe.py`): each block of 160 vertices in `wheat_verts`
    /// (`0x00C8C464`, 480 of them, 16 boxes of `wheat_box_verts`' 10), its
    /// corner the least x and y of the template's foot vertices, over 192.
    /// The `wheatfarm` underlay's 75 vertices (three 5 × 5 grids) span the
    /// same tiles, (8, 161) to (16, 169).
    const RUN1814B_3294_FARM_CORNERS: [(i32, i32); 3] = [(8, 165), (9, 161), (12, 165)];

    /// **`RUN1814B_3294_FARM`** (items 4636, 4650; §6.1): `(agree, scored)`
    /// on `HUE`, the still's mean against the photograph's over each tile of
    /// slot 0's farms' footprints with at least `HUE.min_pixels` ground
    /// pixels (`ground_why`'s ground or an object's window, as the plaza's
    /// row takes them); then on `LUMA_READER`. Item 4650: hue **2 → 48**
    /// and luma **29 → 48 of 48**, the wheat boxes over the `wheatfarm`
    /// underlay. Tile (8, 165), 908 pixels: theirs (197, 147, 45) Y 150.6,
    /// ours (194, 143, 45) Y 147.7, where the grass read (198, 195, 94)
    /// Y 184.8.
    const RUN1814B_3294_FARM: [(usize, usize); 2] = [(48, 48), (48, 48)];

    /// **The row as drawn before** (item 4636's mint): both switches off,
    /// the grass on the plots. It is the made-to-fail: the same row, red.
    /// Item 4653: luma 29 → 30, the capital's plates drawn beside the plots.
    /// Item 4661: 30 → 29, a named fall: the plates on their own alpha, no
    /// longer the borders' ramp.
    /// Item 4665: 29 → 30, the plates on `early_road_pieces.tga` at last
    /// (921 was the surf's mask 1), laid twice.
    /// Item 4670: 30 → 28, a named fall: the trees past sight darkened, the
    /// frame's mean falls and the tone map's white with it (§3.21).
    const RUN1814B_3294_FARM_BEFORE: [(usize, usize); 2] = [(2, 48), (28, 48)];

    /// **The row by half** (item 4650): the `wheatfarm` underlay alone, and
    /// the wheat boxes alone over the grass.
    /// Item 4653: the underlay alone, luma 40 → 39, a named fall, the plates
    /// drawn (their HDR round trip re-tones the frame, §6.2).
    /// Item 4665: 39 → 40, the plates on their own alpha at last.
    /// Item 4670: 40 → 31, a named fall: the trees past sight darkened
    /// re-tone the frame (the drawn row holds 48 of 48).
    /// Item 4673: 31 → 32, element 1's four plates laid beside the plots.
    const RUN1814B_3294_FARM_UNDERLAY: [(usize, usize); 2] = [(9, 48), (32, 48)];
    const RUN1814B_3294_FARM_BOXES: [(usize, usize); 2] = [(48, 48), (48, 48)];

    /// **The frame `FarmsOut::draw` last built run1814b's mesh on**: the
    /// packet's stamp 218,666 is `frame · 200 / 3 + graphic_dt` at 3280,
    /// fourteen frames before the dump's farm records, and the four plots
    /// still growing or cut are fourteen frames behind them.
    const RUN1814B_WHEAT_FRAME: i64 = 3280;

    fn lab(file: &str) -> PathBuf {
        PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
            .join("ron-data/lab-experiments")
            .join(file)
    }

    fn photo_path() -> PathBuf {
        std::env::var_os("RON_PHOTO_RUN1814B")
            .map(PathBuf::from)
            .unwrap_or_else(|| lab("2026-11-23-item-4624/run1814b-3294.tga"))
    }

    fn install() -> PathBuf {
        std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"))
    }

    /// The harness's lobby, the game run1814b captured, at its start.
    fn lobby() -> Option<crate::world::World> {
        match crate::world::World::open_game(
            &install(),
            &crate::world::gamelog_dir(),
            crate::world::Game::Lobby,
        ) {
            Ok(w) => Some(w),
            Err(e) => {
                println!("skipping: {e}");
                None
            }
        }
    }

    /// run1814b's frame drawn as the fog's and the ring's rows draw it: the
    /// harness's lobby at 3294, blended last at 3295, slot 0's fog, the
    /// pinned camera at the packet's `loc`.
    pub(crate) fn probe() -> Option<Probe> {
        let path = photo_path();
        let Ok(bytes) = std::fs::read(&path) else {
            println!("skipping: no photograph at {}", path.display());
            return None;
        };
        let photo = render::tga::decode(&bytes).expect("the photograph decodes");
        let mut world = lobby()?;
        world.run_to(3294);
        world.blend = crate::world::Blend::Last;
        world.run_to(3295);
        world.fog = Some(0);
        let mut camera = crate::pinned_camera();
        (camera.loc_x, camera.loc_y) = crate::minimap::fog::tests::RUN1814B_LOC;
        Some(Probe {
            world,
            photo,
            camera,
            root: install(),
            traced: &[],
        })
    }

    /// **The farm's row** (items 4636, 4650): our slot 0's fields stand on
    /// the packet's corners, and the photograph's plots against the still,
    /// drawn, as drawn before, and by half.
    #[test]
    fn the_farm_on_run1814b() {
        let Some(mut p) = probe() else {
            return;
        };
        assert_eq!(
            (p.photo.width, p.photo.height),
            (1920, 1080),
            "the photograph's frame"
        );
        let scene = p.world.runner.scene(None);
        let mut ours: Vec<(i32, i32)> = super::fields(&scene.buildings)
            .into_iter()
            .filter(|(who, _, _)| *who == 0)
            .map(|(_, c, _)| c)
            .collect();
        ours.sort();
        assert_eq!(ours, RUN1814B_3294_FARM_CORNERS, "the packet's corners");
        let laid: BTreeSet<(i32, i32)> = scene
            .buildings
            .iter()
            .filter(|b| b.owner == 0)
            .filter_map(|b| b.field.map(|f| super::tiles(b, &f).collect::<Vec<_>>()))
            .flatten()
            .collect();
        assert_eq!(laid.len(), 48, "three 4 × 4 farms");
        let (_, any, why) = ground_why(&mut p);
        let pixels: Vec<Option<(i32, i32)>> = any
            .into_iter()
            .zip(why)
            .map(|(t, w)| t.filter(|t| laid.contains(t) && (w == 0 || w == 2)))
            .collect();
        let mut rows = Vec::new();
        for (switches, name) in [
            ((false, false), "before"),
            ((true, false), "underlay"),
            ((false, true), "boxes"),
            ((true, true), "farm"),
        ] {
            p.world.farms = switches;
            let s = still(&mut p, &format!("run1814b-3294-{name}"));
            let row = count("RUN1814B_3294_FARM", &plaza_tiles(&p, &pixels, &s));
            println!("RUN1814B_3294_FARM {name}: {row:?}");
            rows.push(row);
        }
        assert_eq!(
            rows,
            [
                RUN1814B_3294_FARM_BEFORE,
                RUN1814B_3294_FARM_UNDERLAY,
                RUN1814B_3294_FARM_BOXES,
                RUN1814B_3294_FARM
            ]
        );
    }

    /// One packet vertex: x, y, z, the colour's bytes (R, G, B, A), u, v.
    type Theirs = ([f32; 3], [u8; 4], [f32; 2]);

    /// **run1814b's wheat mesh** (`tools/render/wheat_packet.py`): its
    /// heights, stalk, wind and 480 vertices. `None`, saying so, where the
    /// dump is absent.
    #[allow(clippy::type_complexity)]
    fn wheat_packet() -> Option<([f32; 2], [u8; 4], [u32; 2], Vec<Theirs>)> {
        let path = std::env::var_os("RON_WHEAT_RUN1814B")
            .map(PathBuf::from)
            .unwrap_or_else(|| lab("2026-11-25-item-4650/run1814b-wheat.txt"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no wheat dump at {}", path.display());
            return None;
        };
        let (mut heights, mut stalk, mut wind, mut verts) = ([0.0; 2], [0; 4], [0; 2], Vec::new());
        let hex = |w: &str| u32::from_str_radix(w, 16).unwrap();
        for l in text.lines() {
            let w: Vec<&str> = l.split_whitespace().collect();
            match w.first().copied() {
                Some("heights") => heights = [1, 2].map(|i| w[i].parse().unwrap()),
                Some("stalk") => stalk = [1, 2, 3, 4].map(|i| w[i].parse().unwrap()),
                Some("wind") => wind = [1, 2].map(|i| hex(w[i])),
                Some("w") => {
                    let f = |i: usize| f32::from_bits(hex(w[i]));
                    verts.push(([f(1), f(2), f(3)], hex(w[4]).to_le_bytes(), [f(5), f(6)]));
                }
                _ => {}
            }
        }
        Some((heights, stalk, wind, verts))
    }

    /// **The wheat boxes are the packet's** (item 4650): our boxes against
    /// its 480 vertices — every position's bits, colour and texcoord —
    /// block by block, each farm found by its corner, the wind's phase at
    /// [`RUN1814B_WHEAT_FRAME`]. Our art (heights, stalk) and [`WIND_VEC`]
    /// are the packet's. Our state one frame either side of it is the
    /// made-to-fail: the plots still growing or cut part.
    #[test]
    fn the_wheat_is_the_packet_s() {
        let Some((heights, stalk, wind, theirs)) = wheat_packet() else {
            return;
        };
        assert_eq!(theirs.len(), 480, "three farms of sixteen boxes of ten");
        assert_eq!(wind, super::WIND_VEC, "the scene's wind");
        let Some(mut w) = lobby() else {
            return;
        };
        let art = w.wheat_art().expect("the lobby's wheat art");
        assert_eq!((art.heights, art.stalk), (heights, stalk), "the art");
        let (heights, stalk) = (art.heights, art.stalk);
        let windf = wind.map(f32::from_bits);
        let mut found = Vec::new();
        for at in [
            RUN1814B_WHEAT_FRAME - 1,
            RUN1814B_WHEAT_FRAME,
            RUN1814B_WHEAT_FRAME + 1,
        ] {
            w.run_to(at);
            let scene = w.runner.scene(None);
            let (mut same, mut worst) = (0usize, 0.0f32);
            for block in theirs.chunks(160) {
                let foot = |a: usize| {
                    block
                        .iter()
                        .enumerate()
                        .filter(|(k, _)| matches!(k % 10, 5 | 7 | 9))
                        .map(|(_, v)| v.0[a])
                        .fold(f32::MAX, f32::min)
                };
                let corner = ((foot(0) / 192.0) as i32, (foot(1) / 192.0) as i32);
                let Some((b, f)) = scene
                    .buildings
                    .iter()
                    .filter_map(|b| b.field.map(|f| (b, f)))
                    .find(|(b, f)| super::corner(b, f) == corner)
                else {
                    panic!("no field of ours at {corner:?}");
                };
                let _ = b;
                for (bx, plot) in block.chunks(10).enumerate() {
                    let (row, col) = (bx / 4, bx % 4);
                    let cell = col * 4 + row;
                    let pc = super::percent(f.adds[cell], f.status[cell]);
                    let colour = super::plot_colour(stalk, pc, 1.0);
                    let verts =
                        super::plot(&f, corner, (col, row), heights, windf, RUN1814B_WHEAT_FRAME);
                    let texs = super::BOX_TEXS[(col + row) & 1];
                    for (k, (pos, rgba, uv)) in plot.iter().enumerate() {
                        let ok = verts[k].map(f32::to_bits) == pos.map(f32::to_bits)
                            && colour == *rgba
                            && texs[k] == *uv;
                        same += usize::from(ok);
                        for a in 0..3 {
                            worst = worst.max((verts[k][a] - pos[a]).abs());
                        }
                        if !ok && at == RUN1814B_WHEAT_FRAME {
                            println!(
                                "wheat {corner:?} plot ({col}, {row}) v{k}: ours {:?} {colour:?}, theirs {pos:?} {rgba:?}",
                                verts[k]
                            );
                        }
                    }
                }
            }
            println!("wheat at {at}: {same} of 480 whole, worst {worst}");
            found.push(same);
        }
        assert_eq!(found[1], 480, "the mesh on its frame");
        assert!(
            found[0] < 480 && found[2] < 480,
            "a frame either side parts"
        );
    }

    /// **The `wheatfarm` underlay is the packet's slot 1** (item 4650;
    /// `tools/render/snowcap_packet.py`): our patch's corners — tile, UV0
    /// and UV1, a multiset, since neighbouring farms share a column of
    /// tiles — against the packet's 75, and their `light_vert` colours;
    /// no `pigfarm` on either side.
    #[test]
    fn the_underlay_is_the_packet_s_slot_1() {
        use render::silhouette::{LAND_AMBIENT, LAND_DIFFUSE};
        let path = std::env::var_os("RON_UNDERLAYS_RUN1814B")
            .map(PathBuf::from)
            .unwrap_or_else(|| lab("2026-11-25-item-4650/run1814b-underlays.txt"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            println!("skipping: no underlay dump at {}", path.display());
            return;
        };
        let (mut slot, mut theirs, mut slots) = (-1, Vec::new(), BTreeSet::new());
        for l in text.lines() {
            let w: Vec<&str> = l.split_whitespace().collect();
            match w.first().copied() {
                Some("slot") => {
                    slot = w[1].parse().unwrap();
                    slots.insert(slot);
                }
                Some("v") if slot == 1 => {
                    let f = |i: usize| w[i].parse::<f32>().unwrap();
                    let tile: (i32, i32) = (w[1].parse().unwrap(), w[2].parse().unwrap());
                    let c = u32::from_str_radix(w[3], 16).unwrap();
                    theirs.push((tile, [f(4), f(5)], [f(6), f(7)], c));
                }
                _ => {}
            }
        }
        assert!(!slots.contains(&0), "the packet lays no pigfarm");
        let Some(mut p) = probe() else {
            return;
        };
        let (_, _, patches) = p.world.drawn_terrain();
        let per = render::terrain::UNITS_PER_TILE as f32;
        let tile = |q: &[f32; 3]| ((q[0] / per).round() as i32, (q[1] / per).round() as i32);
        // `ObjectsOut::draw_weathering` walks the objects in view, so the
        // packet's slot holds the farms on screen; ours lays every seen
        // farm's, and the corners compared are those over the packet's box.
        let axis = |a: fn(&(i32, i32)) -> i32| {
            let v = theirs.iter().map(|t| a(&t.0));
            v.clone().min().unwrap()..=v.max().unwrap()
        };
        let (xs, ys) = (axis(|t| t.0), axis(|t| t.1));
        let inside = |t: (i32, i32)| xs.contains(&t.0) && ys.contains(&t.1);
        assert!(
            !patches
                .iter()
                .filter(|m| m.piece == crate::world::FARM_PIG_PIECE)
                .any(|m| m.positions.iter().any(|q| inside(tile(q)))),
            "we lay no pigfarm there"
        );
        let m = patches
            .iter()
            .find(|m| m.piece == crate::world::FARM_WHEAT_PIECE)
            .expect("a wheatfarm laid");
        let key =
            |t: (i32, i32), a: [f32; 2], b: [f32; 2]| (t, a.map(f32::to_bits), b.map(f32::to_bits));
        let mut ours: Vec<_> = (0..m.positions.len())
            .filter(|&i| inside(tile(&m.positions[i])))
            .map(|i| {
                (
                    key(tile(&m.positions[i]), m.uvs[i], m.alpha_uvs[i]),
                    m.light[i],
                )
            })
            .collect();
        ours.sort_by_key(|a| a.0);
        ours.dedup_by(|a, b| a.0 == b.0);
        let mut want: Vec<_> = theirs
            .iter()
            .map(|(t, a, b, c)| (key(*t, *a, *b), *c))
            .collect();
        want.sort_by_key(|a| a.0);
        assert_eq!(
            ours.iter().map(|o| o.0).collect::<Vec<_>>(),
            want.iter().map(|o| o.0).collect::<Vec<_>>(),
            "the 75 corners and their texcoords"
        );
        let (mut exact, mut near) = (0, 0);
        for ((at, nl), (_, c)) in ours.iter().zip(&want) {
            let theirs = [c & 0xff, (c >> 8) & 0xff, (c >> 16) & 0xff].map(|b| b as i32);
            let got = render::terrain::light_vert(*nl, LAND_AMBIENT, LAND_DIFFUSE).map(i32::from);
            let d = (0..3).map(|k| (got[k] - theirs[k]).abs()).max().unwrap();
            exact += usize::from(d == 0);
            near += usize::from(d <= 1);
            if d > 1 {
                println!(
                    "wheatfarm corner {:?}: ours {got:?} (N·L {nl}), theirs {theirs:?}",
                    at.0
                );
            }
        }
        println!(
            "wheatfarm: {} corners, colour {exact} exact, {near} within 1",
            want.len()
        );
        assert_eq!((want.len(), exact, near), UNDERLAY_CORNERS);
    }

    /// The `wheatfarm` underlay's corners on run1814b's packet, those whose
    /// `light_vert` colour is the packet's exactly, and within one step.
    /// The 23 apart part by 2 to 7 steps near white, ours the brighter on
    /// 17 (parked 4650a): the land's light under the capital's farms, not
    /// the underlay's rule.
    const UNDERLAY_CORNERS: (usize, usize, usize) = (75, 48, 52);
}
