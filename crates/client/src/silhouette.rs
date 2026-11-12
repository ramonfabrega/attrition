//! Candidate C's first row (`docs/RENDERER.md` §5.3, item 4038): the
//! scene's silhouettes on run1559b's frame 3294 against that take's own
//! photograph of it, at the photograph's camera.
//!
//! The photograph is kept outside the repository, converted once to a
//! `.tga` so `render::tga` reads it with no new dependency
//! (`sips -s format tga`): `$RON_PHOTO`, else
//! `~/ron-data/lab-experiments/2026-11-09-item-4038/run1559b-3294.tga`.

use crate::world::{Drawing, SpriteDrawing, TREE_WHO, TreePixels, World};
use render::silhouette::{
    Footprint, Object, Photo, READING, Reader, Rect, Stamp, footprints_read, rasterize_cut,
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
    let scene = frame_scene(world, camera, size);
    (scene.keys, scene.who)
}

/// [`frame_keys`]'s key buffer and who each key is, with the meshes and
/// sprites it was drawn from (item 4099): key `k` is `drawings[k − 1]`
/// while `k ≤ drawings.len()`, then `sprites[k − 1 − drawings.len()]`.
pub struct Scene {
    pub keys: render::silhouette::Keys,
    pub who: Vec<Who>,
    /// The pixels a figure took over a nearer tree or sprite by the
    /// silhouette's pass (item 4085), row-major.
    #[cfg_attr(not(test), allow(dead_code))]
    pub behind: Vec<bool>,
    /// At a `behind` pixel, the key of the tree or sprite the figure is
    /// behind (item 4103); 0 elsewhere.
    #[cfg_attr(not(test), allow(dead_code))]
    pub nearer: Vec<u32>,
    #[cfg_attr(not(test), allow(dead_code))]
    pub drawings: Vec<Drawing>,
    #[cfg_attr(not(test), allow(dead_code))]
    pub sprites: Vec<SpriteDrawing>,
}

/// The frame as [`frame_keys`] draws it, its parts kept.
pub fn frame_scene(world: &mut World, camera: &render::Camera, size: (usize, usize)) -> Scene {
    let (figures, buildings, list) = world.draw_list(
        camera,
        &crate::pinned_screen(),
        (size.0 as i32, size.1 as i32),
    );
    let drawings: Vec<Drawing> = world.objects(&figures, &buildings, &list);
    let sprites = world.sprite_drawings(&buildings, &list, camera.zoom_distance);
    let first = drawings.len() as u32 + 1;
    // The map's trees (item 4080) after the sprites, keyed from `trees`.
    let trees = world.tree_drawings();
    let first_tree = first + sprites.len() as u32;
    let mut meshes = objects(&drawings);
    meshes.extend(trees.iter().enumerate().map(|(j, d)| Object {
        key: first_tree + j as u32,
        tris: d.tris.clone(),
    }));
    let cuts = world.tree_cuts();
    let cut: Vec<Option<&render::silhouette::Cut<'_>>> = match world.leaves {
        crate::world::Leaves::Solid => Vec::new(),
        crate::world::Leaves::Alpha => std::iter::repeat_n(None, drawings.len())
            .chain(cuts.iter().map(Option::as_ref))
            .collect(),
    };
    let mut keys = rasterize_cut(&meshes, &cut, &stamps(&sprites, first), camera, size).keys;
    let mut behind = vec![false; keys.keys.len()];
    let mut nearer = vec![0u32; keys.keys.len()];
    if world.silhouettes == crate::world::Silhouettes::Drawn {
        // The guys pass after `flush_trees` (item 4085): a figure's
        // `render_states[1]` draw passes where it is behind, writing no
        // depth, so over a tree or a sprite its pixel is its own. Among the
        // meshes the scene's depth order already stands.
        let figures: Vec<Object> = objects(&drawings)
            .into_iter()
            .filter(|o| !drawings[o.key as usize - 1].building)
            .collect();
        let alone = rasterize_cut(&figures, &[], &[], camera, size).keys;
        for (((k, f), b), n) in keys
            .keys
            .iter_mut()
            .zip(&alone.keys)
            .zip(&mut behind)
            .zip(&mut nearer)
        {
            if *f != 0 && *k >= first {
                *n = *k;
                *k = *f;
                *b = true;
            }
        }
    }
    if world.tree_pixels == TreePixels::Ground {
        for k in keys.keys.iter_mut().filter(|k| **k >= first_tree) {
            *k = 0;
        }
    }
    let who = drawings
        .iter()
        .map(|d| (d.who, d.o, d.piece, d.building))
        .chain(sprites.iter().map(|s| (s.who, s.o, s.piece, true)))
        .chain(trees.iter().map(|d| (d.who, d.o, d.piece, false)))
        .collect();
    Scene {
        keys,
        who,
        behind,
        nearer,
        drawings,
        sprites,
    }
}

/// How the colour reader's figures are painted (item 4099).
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Colouring {
    /// Each owner's debug tint ([`crate::world::tint`]), the draw before.
    Tint,
    /// The `<UNIT>`'s `texture=` through the `.BH3`'s texcoords.
    Texture,
    /// As `Texture`, the owner's unit colour through the texel's alpha
    /// (item 4102, `render::silhouette::team_blend`).
    Team,
    /// As `Team`, and a pixel the silhouette's pass owns (`Scene::behind`)
    /// the owner's unit colour at 128/255 over the nearer tree's or
    /// sprite's own pixel (item 4103, `render::silhouette::
    /// silhouette_blend`): the client's draw.
    Silhouette,
}

/// The nearer object's own colour over `window` (item 4103): a tree's
/// texel through its cut ([`render::silhouette::paint_cut`]), else
/// [`painted`]'s, team-coloured.
#[cfg(test)]
pub fn paint_nearer(
    world: &mut World,
    scene: &Scene,
    camera: &render::Camera,
    size: (usize, usize),
    key: u32,
    window: Rect,
) -> Vec<Option<render::silhouette::Texel>> {
    let first_tree = (scene.drawings.len() + scene.sprites.len()) as u32 + 1;
    if key < first_tree {
        return painted(world, scene, camera, size, key, window, Colouring::Team);
    }
    let i = (key - first_tree) as usize;
    let trees = world.tree_drawings();
    let cuts = world.tree_cuts();
    match (trees.get(i), cuts.get(i).and_then(Option::as_ref)) {
        (Some(d), Some(cut)) => render::silhouette::paint_cut(
            &Object {
                key,
                tris: d.tris.clone(),
            },
            cut,
            camera,
            size,
            window,
        ),
        _ => vec![None; ((window.x1 - window.x0) * (window.y1 - window.y0)).max(0) as usize],
    }
}

/// [`painted`], with the silhouette's pass laid over `key`'s pixels where
/// it owns them (item 4103), when `colouring` is `Silhouette`.
#[cfg(test)]
pub fn painted_scene(
    world: &mut World,
    scene: &Scene,
    camera: &render::Camera,
    size: (usize, usize),
    key: u32,
    window: Rect,
    colouring: Colouring,
) -> Vec<Option<render::silhouette::Texel>> {
    let mut ours = painted(world, scene, camera, size, key, window, colouring);
    if colouring != Colouring::Silhouette {
        return ours;
    }
    let owner = render::silhouette::team_unit(scene.who[key as usize - 1].0);
    let ww = (window.x1 - window.x0) as usize;
    let mut near: std::collections::BTreeMap<u32, Vec<Option<render::silhouette::Texel>>> =
        Default::default();
    for y in window.y0..window.y1 {
        for x in window.x0..window.x1 {
            let at = y as usize * size.0 + x as usize;
            if !scene.behind[at] || scene.keys.at(x as usize, y as usize) != key {
                continue;
            }
            let j = (y - window.y0) as usize * ww + (x - window.x0) as usize;
            let k = scene.nearer[at];
            let n = near
                .entry(k)
                .or_insert_with(|| paint_nearer(world, scene, camera, size, k, window));
            if let (Some(t), Some(u)) = (ours[j].as_mut(), n[j]) {
                t.rgb = render::silhouette::silhouette_blend(owner, u.rgb);
                t.alpha = 255.0;
            }
        }
    }
    ours
}

/// One object's own colour over its window, as `colouring` paints it: a
/// mesh by its tint or its texture (a mesh with neither texcoords nor a
/// texture keeps its tint), a sprite by its image.
#[cfg(test)]
pub fn painted(
    world: &mut World,
    scene: &Scene,
    camera: &render::Camera,
    size: (usize, usize),
    key: u32,
    window: Rect,
    colouring: Colouring,
) -> Vec<Option<render::silhouette::Texel>> {
    use render::silhouette::{Paint, paint, paint_stamp};
    let k = key as usize - 1;
    if let Some(d) = scene.drawings.get(k) {
        let object = Object {
            key,
            tris: d.tris.clone(),
        };
        let tint = crate::world::tint(d.who, d.building).map(|v| (v * 255.0).round() as u8);
        let team = matches!(colouring, Colouring::Team | Colouring::Silhouette);
        let mips = (colouring != Colouring::Tint && !d.uvs.is_empty())
            .then(|| world.figure_texture(d.piece))
            .flatten();
        let how = match &mips {
            Some(m) => Paint::Texture {
                uvs: &d.uvs,
                mips: m,
                owner: team.then(|| render::silhouette::team_unit(d.who)),
            },
            None => Paint::Flat(tint),
        };
        return paint(&object, &how, camera, size, window);
    }
    let first = scene.drawings.len() as u32 + 1;
    let j = k - scene.drawings.len();
    let Some(sp) = scene.sprites.get(j) else {
        return vec![None; ((window.x1 - window.x0) * (window.y1 - window.y0)) as usize];
    };
    let st = &stamps(&scene.sprites, first)[j];
    paint_stamp(st, &sp.image.image.rgba, size, window)
}

/// **The colour row** (item 4099; `docs/RENDERER.md` §5.3, "The colour
/// reader"): every object the traced reader scores on the frame, its
/// footprints and its colours under `colouring`.
#[cfg(test)]
pub fn colour_rows(
    world: &mut World,
    camera: &render::Camera,
    photo: &render::tga::Image,
    traced: &[(&str, &[(i32, i32)])],
    colouring: Colouring,
) -> Vec<(Footprint, render::silhouette::Colours, Who)> {
    use render::silhouette::{HUE, read_colours, traced_theirs};
    let size = (photo.width, photo.height);
    let scene = frame_scene(world, camera, size);
    let p = Photo {
        width: photo.width,
        height: photo.height,
        rgba: &photo.rgba,
    };
    let traces: Vec<Trace<'_>> = traced
        .iter()
        .map(|&(name, poly)| Trace {
            key: scene
                .who
                .iter()
                .position(|&(who, o, _, _)| format!("{who}/{o}") == name)
                .map_or(u32::MAX, |i| i as u32 + 1),
            poly,
        })
        .collect();
    let rows = footprints_traced(
        &scene.keys,
        &p,
        &INTERFACE,
        &READING,
        &traces,
        Neighbour::Neither,
        Interface::Neither,
    );
    let mut out = Vec::new();
    for f in rows {
        let who = scene.who[f.key as usize - 1];
        if who.0 == TREE_WHO {
            continue;
        }
        let theirs = traced_theirs(
            &scene.keys,
            &p,
            &INTERFACE,
            &READING,
            &traces,
            Neighbour::Neither,
            Interface::Neither,
            f.key,
            f.window,
        );
        let ours = painted_scene(world, &scene, camera, size, f.key, f.window, colouring);
        let w = photo.width;
        let c = read_colours(
            &scene.keys,
            &p,
            f.key,
            f.window,
            &theirs,
            &ours,
            &HUE,
            |x, y| scene.behind[y as usize * w + x as usize],
        );
        out.push((f, c, who));
    }
    out
}

/// **One object's row on the client's still** (item 4171; `docs/RENDERER.md`
/// §3.15): the painter's footprint and colours beside the
/// still's, and how far the still's pixels lie from the painter's.
#[cfg(test)]
pub struct StillRow {
    pub painter: Footprint,
    pub still: Footprint,
    pub painter_colours: render::silhouette::Colours,
    pub colours: render::silhouette::Colours,
    pub who: Who,
    /// The object's own pixels the painter paints, and of those the ones
    /// whose every channel on the still lies within 8 of the painter's.
    pub pixels: u32,
    pub within: u32,
    /// The first own pixels: the value diff.
    pub sample: Vec<PixelPair>,
}

/// One pixel's colour, `((x, y), painter, still)` (item 4171).
#[cfg(test)]
pub type PixelPair = ((i32, i32), [f32; 3], [f32; 3]);

/// **The traced and colour rows on the still** (item 4171): [`colour_rows`]'
/// objects, windows and photograph side, unchanged, with our side read off
/// `still` (`photo`'s size, RGBA rows top first) by §5.3's rule — the
/// footprint by the photograph's colour reader run on the still, its ring
/// the still's pixels where no key lies; the colours the still's pixels.
#[cfg(test)]
pub fn still_rows(
    world: &mut World,
    camera: &render::Camera,
    photo: &render::tga::Image,
    traced: &[(&str, &[(i32, i32)])],
    still: &[u8],
) -> Vec<StillRow> {
    use render::silhouette::{HUE, Texel, read_colours, traced_theirs};
    let size = (photo.width, photo.height);
    assert_eq!(
        still.len(),
        size.0 * size.1 * 4,
        "the still is the photograph's size"
    );
    let scene = frame_scene(world, camera, size);
    let p = Photo {
        width: photo.width,
        height: photo.height,
        rgba: &photo.rgba,
    };
    let ours_photo = Photo {
        width: photo.width,
        height: photo.height,
        rgba: still,
    };
    let traces: Vec<Trace<'_>> = traced
        .iter()
        .map(|&(name, poly)| Trace {
            key: scene
                .who
                .iter()
                .position(|&(who, o, _, _)| format!("{who}/{o}") == name)
                .map_or(u32::MAX, |i| i as u32 + 1),
            poly,
        })
        .collect();
    let rows = footprints_traced(
        &scene.keys,
        &p,
        &INTERFACE,
        &READING,
        &traces,
        Neighbour::Neither,
        Interface::Neither,
    );
    let w = photo.width;
    let mut out = Vec::new();
    for f in rows {
        let who = scene.who[f.key as usize - 1];
        if who.0 == TREE_WHO {
            continue;
        }
        let theirs = traced_theirs(
            &scene.keys,
            &p,
            &INTERFACE,
            &READING,
            &traces,
            Neighbour::Neither,
            Interface::Neither,
            f.key,
            f.window,
        );
        // Our side by the photograph's colour reader, on the still: no
        // trace is ours, so a pixel is the interface's or read by colour.
        let mine = traced_theirs(
            &scene.keys,
            &ours_photo,
            &INTERFACE,
            &READING,
            &[],
            Neighbour::Neither,
            Interface::Neither,
            f.key,
            f.window,
        );
        let mut sf = Footprint {
            key: f.key,
            window: f.window,
            ours: 0,
            theirs: 0,
            both: 0,
        };
        for (t, m) in theirs.iter().zip(&mine) {
            let Some(t) = *t else { continue };
            let o = *m == Some(true);
            sf.ours += u32::from(o);
            sf.theirs += u32::from(t);
            sf.both += u32::from(o && t);
        }
        let painted = painted_scene(
            world,
            &scene,
            camera,
            size,
            f.key,
            f.window,
            Colouring::Silhouette,
        );
        let ww = (f.window.x1 - f.window.x0) as usize;
        let (mut pixels, mut within) = (0u32, 0u32);
        let mut sample = Vec::new();
        let mut ours: Vec<Option<Texel>> = Vec::with_capacity(painted.len());
        for y in f.window.y0..f.window.y1 {
            for x in f.window.x0..f.window.x1 {
                let j = (y - f.window.y0) as usize * ww + (x - f.window.x0) as usize;
                let i = (y as usize * w + x as usize) * 4;
                let rgb = [0, 1, 2].map(|k| f32::from(still[i + k]));
                let at = painted[j];
                if let Some(q) = at.filter(|_| scene.keys.at(x as usize, y as usize) == f.key) {
                    pixels += 1;
                    within += u32::from((0..3).all(|k| (q.rgb[k] - rgb[k]).abs() <= 8.0));
                    if pixels % 16 == 1 && sample.len() < 4 {
                        sample.push(((x, y), q.rgb, rgb));
                    }
                }
                ours.push(at.map(|q| Texel { rgb, ..q }));
            }
        }
        let behind = |x: i32, y: i32| scene.behind[y as usize * w + x as usize];
        let painter_colours = read_colours(
            &scene.keys,
            &p,
            f.key,
            f.window,
            &theirs,
            &painted,
            &HUE,
            behind,
        );
        let colours = read_colours(
            &scene.keys,
            &p,
            f.key,
            f.window,
            &theirs,
            &ours,
            &HUE,
            behind,
        );
        out.push(StillRow {
            painter: f,
            still: sf,
            painter_colours,
            colours,
            who,
            pixels,
            within,
            sample,
        });
    }
    out
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
        .filter(|(_, d)| d.0 != TREE_WHO)
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
    .filter(|(_, d)| d.0 != TREE_WHO)
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
        // These rows are the treeless draw's, kept by name (item 4080).
        world.trees = false;

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
            let rest_rows = score_frame(&mut world, &camera, &photo, Reader::Colour);
            print_rows("at rest", &rest_rows, &rest_rows);
            let rest: Vec<Footprint> = rest_rows.iter().map(|(f, _)| *f).collect();
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
        // The tilt's other readings (item 4089): every figure level, the
        // draw before it, and the ease at a draw a frame from 3200.
        world.tilt = crate::world::Tilt::Level;
        let level_rows = read(
            &mut world,
            &camera,
            &TRACED,
            Neighbour::Neither,
            Interface::Neither,
        );
        let level = print("traced, level", &level_rows);
        world.tilt = crate::world::Tilt::Eased;
        let eased_rows = read(
            &mut world,
            &camera,
            &TRACED,
            Neighbour::Neither,
            Interface::Neither,
        );
        let eased = print("traced, eased", &eased_rows);
        world.tilt = crate::world::Tilt::Target;
        println!("tilt: target {got:?} level {level:?} eased {eased:?}");
        // The silhouettes' rejected reading, a figure hidden by what is
        // nearer (item 4085).
        world.silhouettes = crate::world::Silhouettes::None;
        let hidden_rows = read(
            &mut world,
            &camera,
            &TRACED,
            Neighbour::Neither,
            Interface::Neither,
        );
        let hidden = print("traced, no silhouettes", &hidden_rows);
        world.silhouettes = crate::world::Silhouettes::Drawn;
        // The trees' rejected reading, and the treeless draw (item 4080).
        world.tree_pixels = crate::world::TreePixels::Scene;
        let tree_scene_rows = read(
            &mut world,
            &camera,
            &TRACED,
            Neighbour::Neither,
            Interface::Neither,
        );
        let tree_scene = print("traced, a tree's pixel as scene", &tree_scene_rows);
        world.tree_pixels = crate::world::TreePixels::Ground;
        // The leaves' rejected reading, every triangle whole (item 4082).
        world.leaves = crate::world::Leaves::Solid;
        let solid_rows = read(
            &mut world,
            &camera,
            &TRACED,
            Neighbour::Neither,
            Interface::Neither,
        );
        let solid = print("traced, the leaves solid", &solid_rows);
        world.leaves = crate::world::Leaves::Alpha;
        world.trees = false;
        let treeless_rows = read(
            &mut world,
            &camera,
            &TRACED,
            Neighbour::Neither,
            Interface::Neither,
        );
        let treeless = print("traced, no trees", &treeless_rows);
        world.trees = true;
        for (g, (who, o, piece, _)) in &treeless_rows {
            if !rule
                .iter()
                .any(|(_, w)| (w.0, w.1, w.2) == (*who, *o, *piece))
            {
                println!(
                    "unscored with trees {who}/{o} piece {piece}: ours {} theirs {} both {} overlap {:.3}",
                    g.ours,
                    g.theirs,
                    g.both,
                    g.overlap()
                );
            }
        }
        for (f, (who, o, piece, _)) in &rule {
            let before = treeless_rows
                .iter()
                .find(|(_, w)| (w.0, w.1, w.2) == (*who, *o, *piece))
                .map(|(g, _)| (g.ours, g.theirs, g.both, g.overlap()));
            let now = (f.ours, f.theirs, f.both, f.overlap());
            if before.map(|b| (b.0, b.1, b.2)) != Some((now.0, now.1, now.2)) {
                println!("moved {who}/{o} piece {piece}: no trees {before:?} trees {now:?}");
            }
        }
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
        assert_eq!(got, s::RUN1559B_3279_TRACED_CURRENT);
        assert_eq!(level, s::RUN1559B_3279_TRACED_LEVEL);
        assert_eq!(eased, s::RUN1559B_3279_TRACED_EASED);
        assert_eq!(hidden, s::RUN1559B_3279_TRACED_NO_SILHOUETTES);
        assert_eq!(tree_scene, s::RUN1559B_3279_TRACED_TREES_SCENE);
        assert_eq!(solid, s::RUN1559B_3279_TRACED_LEAVES_SOLID);
        assert_eq!(treeless, s::RUN1559B_3279_TRACED_TREELESS);
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
        // The point (item 4095, §5.3, committed before this count): frame
        // 3280's slot, clock and facing at the points held on 3279.
        world.blend = crate::world::Blend::Last;
        world.run_to(PHOTO_FRAME + 1);
        let point_rows = read(
            &mut world,
            &camera,
            &TRACED,
            Neighbour::Neither,
            Interface::Neither,
        );
        let point = print("traced, the point", &point_rows);
        for (f, (who, o, _, _)) in &point_rows {
            let name = format!("{who}/{o}");
            let was = row(&name, &rule).map(|g| g.overlap());
            let now = f.overlap();
            let t = READING.threshold;
            if was.is_none_or(|w| (w >= t) != (now >= t) || (w - now).abs() > 0.02) {
                println!("the point: {name} {was:?} -> {now:.3}");
            }
        }
        println!("the point: {point:?} against {got:?}");
        assert_eq!(point, s::RUN1559B_3279_TRACED);
        for name in ["1/20", "1/32"] {
            let f = row(name, &point_rows).expect("scored");
            assert!(f.overlap() >= READING.threshold, "{name} passes");
        }
        world.blend = crate::world::Blend::Current;
    }

    /// **The traced and colour rows on the client's still** (item 4171;
    /// `docs/RENDERER.md` §3.15, committed before this count):
    /// [`still_rows`] at both places of the silhouette pass, every object
    /// whose verdict the still changes named with both sides' values.
    /// Made to fail: the draw before (`SilhouetteOrder::After`) washes the
    /// figures, and its colour row falls.
    #[test]
    fn the_rows_on_the_client_s_still() {
        use crate::gpu::SilhouetteOrder;
        use render::silhouette::HUE;
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
        world.run_to(PHOTO_FRAME);
        world.blend = crate::world::Blend::Last;
        world.run_to(PHOTO_FRAME + 1);
        let size = (photo.width as u32, photo.height as u32);
        let mut got = Vec::new();
        use wgpu::TextureFormat::{Rgba8Unorm, Rgba8UnormSrgb};
        for (order, format) in [
            (SilhouetteOrder::Before, Rgba8Unorm),
            (SilhouetteOrder::After, Rgba8Unorm),
            (SilhouetteOrder::Before, Rgba8UnormSrgb),
            (SilhouetteOrder::After, Rgba8UnormSrgb),
        ] {
            let order_name = format!("{order:?} {format:?}");
            let still = crate::still_of(&mut world, &camera, size, order, format);
            if let Some(dir) = std::env::var_os("RON_STILL_DIR") {
                let out = PathBuf::from(dir).join(format!("still-1559b-{order:?}-{format:?}.bmp"));
                std::fs::write(&out, crate::gpu::bmp(&still, size)).expect("the still writes");
            }
            let rows = still_rows(&mut world, &camera, &photo, &TRACED, &still);
            let t = READING.threshold;
            let (mut pixels, mut within) = (0u32, 0u32);
            for r in &rows {
                let (a, b) = (r.painter.overlap(), r.still.overlap());
                let (c, d) = (r.painter_colours.passes(&HUE), r.colours.passes(&HUE));
                pixels += r.pixels;
                within += r.within;
                println!(
                    "{order_name} {}/{} piece {}{} window ({}, {})..({}, {}): overlap painter {a:.3} still {b:.3} (ours {} -> {}, both {} -> {}) | colour painter lit {} agree {} mean {:.3} {} still lit {} agree {} mean {:.3} {} behind {} part {} -> {} | within 8: {} of {}{}{}",
                    r.who.0,
                    r.who.1,
                    r.who.2,
                    if r.who.3 { " building" } else { "" },
                    r.painter.window.x0,
                    r.painter.window.y0,
                    r.painter.window.x1,
                    r.painter.window.y1,
                    r.painter.ours,
                    r.still.ours,
                    r.painter.both,
                    r.still.both,
                    r.painter_colours.lit,
                    r.painter_colours.agree,
                    r.painter_colours.mean,
                    if c { "in" } else { "out" },
                    r.colours.lit,
                    r.colours.agree,
                    r.colours.mean,
                    if d { "in" } else { "out" },
                    r.colours.behind,
                    r.painter_colours.behind_part,
                    r.colours.behind_part,
                    r.within,
                    r.pixels,
                    if (a >= t) != (b >= t) {
                        " TRACED CROSSES"
                    } else {
                        ""
                    },
                    if c != d { " COLOUR CROSSES" } else { "" },
                );
                for (at, a, b) in &r.sample {
                    println!(
                        "    pixel {at:?}: painter ({:.0}, {:.0}, {:.0}) still ({:.0}, {:.0}, {:.0})",
                        a[0], a[1], a[2], b[0], b[1], b[2]
                    );
                }
            }
            let traced = (
                rows.iter().filter(|r| r.still.overlap() >= t).count(),
                rows.len(),
            );
            let colour = (
                rows.iter().filter(|r| r.colours.passes(&HUE)).count(),
                rows.len(),
            );
            let painter = (
                rows.iter().filter(|r| r.painter.overlap() >= t).count(),
                rows.iter()
                    .filter(|r| r.painter_colours.passes(&HUE))
                    .count(),
            );
            println!(
                "{order_name}: traced {traced:?} colour {colour:?} on the still; the painter's {painter:?}; pixels within 8 {within} of {pixels}"
            );
            assert_eq!(
                painter,
                (s::RUN1559B_3279_TRACED.0, s::RUN1559B_3279_COLOUR.0)
            );
            got.push((traced, colour));
        }
        use render::silhouette as s;
        assert_eq!(got, s::RUN1559B_3279_STILL_READINGS);
        assert_eq!(
            got[0],
            (s::RUN1559B_3279_TRACED_STILL, s::RUN1559B_3279_COLOUR_STILL),
            "the client's draw"
        );
    }

    /// **The colour row** (item 4099; `docs/RENDERER.md` §5.3, "The colour
    /// reader", committed before this count): on the row's draw
    /// (`Blend::Last`), each object the traced reader scores, its colours
    /// on the pixels both footprints hold, painted by the debug tint and
    /// by the figure's texture through its texcoords. Made to fail: the
    /// tint reads most figures out, and run1441's camera reads 0.
    #[test]
    fn the_figures_colours_on_the_photograph() {
        use render::silhouette::{HUE, score};
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
        let tint = colour_rows(&mut world, &camera, &photo, &TRACED, Colouring::Tint);
        let tex = colour_rows(&mut world, &camera, &photo, &TRACED, Colouring::Texture);
        let team = colour_rows(&mut world, &camera, &photo, &TRACED, Colouring::Team);
        let sil = colour_rows(&mut world, &camera, &photo, &TRACED, Colouring::Silhouette);
        let off = colour_rows(
            &mut world,
            &elsewhere,
            &photo,
            &TRACED,
            Colouring::Silhouette,
        );
        let count = |rows: &[(Footprint, render::silhouette::Colours, Who)]| {
            (
                rows.iter().filter(|(_, c, _)| c.passes(&HUE)).count(),
                rows.len(),
            )
        };
        let mut sums = [(0u32, 0u32, 0u32, 0u32, 0u32, 0u32); 2];
        let mut offset = ([0f32; 2], 0u32);
        for ((f, t, (who, o, piece, building)), (_, x, _)) in tint.iter().zip(&tex) {
            println!(
                "{who}/{o} piece {piece}{} window ({}, {})..({}, {}) overlap {:.3}: held {} dark {} | tint lit {} agree {} mean {:.3} {} | texture lit {} agree {} mean {:.3} {} behind {} part {} masked {} part {} rest {}",
                if *building { " building" } else { "" },
                f.window.x0,
                f.window.y0,
                f.window.x1,
                f.window.y1,
                f.overlap(),
                x.held,
                x.dark,
                t.lit,
                t.agree,
                t.mean,
                if t.passes(&HUE) { "in" } else { "out" },
                x.lit,
                x.agree,
                x.mean,
                if x.passes(&HUE) { "in" } else { "out" },
                x.behind,
                x.behind_part,
                x.masked,
                x.masked_part,
                x.rest,
            );
            if !*building {
                for (k, c) in [t, x].iter().enumerate() {
                    sums[k].0 += c.lit;
                    sums[k].1 += c.part();
                    sums[k].2 += c.behind;
                    sums[k].3 += c.behind_part;
                    sums[k].4 += c.masked;
                    sums[k].5 += c.masked_part;
                }
                offset.0[0] += x.offset[0];
                offset.0[1] += x.offset[1];
                offset.1 += x.rest;
            }
        }
        for (k, label) in ["tint", "texture"].iter().enumerate() {
            let (lit, part, behind, behind_part, masked, masked_part) = sums[k];
            let rest_part = part - behind_part - masked_part;
            println!(
                "{label}, the figures: {lit} lit pixels, {part} part ({:.3}); behind {behind}, {behind_part} part ({:.3} of what parts); the mask's {masked}, {masked_part} part ({:.3}); the rest {}, {rest_part} part ({:.3})",
                part as f32 / lit.max(1) as f32,
                behind_part as f32 / part.max(1) as f32,
                masked_part as f32 / part.max(1) as f32,
                lit - behind - masked,
                rest_part as f32 / part.max(1) as f32,
            );
        }
        let n = offset.1.max(1) as f32;
        println!(
            "the light: the photograph's chromaticity less ours over the rest's {} pixels, ({:+.4}, {:+.4})",
            offset.1,
            offset.0[0] / n,
            offset.0[1] / n
        );
        // Item 4102: the team colour through the texel's alpha, each
        // object beside the texel alone, every crossing named.
        let mut team_sums = (0u32, 0u32, 0u32, 0u32);
        for ((_, x, (who, o, piece, building)), (_, m, _)) in tex.iter().zip(&team) {
            let (a, b) = (x.passes(&HUE), m.passes(&HUE));
            println!(
                "team {who}/{o} piece {piece}: texel lit {} agree {} {} | team lit {} agree {} mean {:.3} {} masked {} part {}{}",
                x.lit,
                x.agree,
                if a { "in" } else { "out" },
                m.lit,
                m.agree,
                m.mean,
                if b { "in" } else { "out" },
                m.masked,
                m.masked_part,
                match (a, b) {
                    (false, true) => " CROSSES UP",
                    (true, false) => " CROSSES DOWN",
                    _ => "",
                }
            );
            if !*building {
                team_sums.0 += m.lit;
                team_sums.1 += m.agree;
                team_sums.2 += m.masked;
                team_sums.3 += m.masked_part;
            }
        }
        println!(
            "team, the figures: {} lit, {} agree; the mask's {}, {} part ({:.3})",
            team_sums.0,
            team_sums.1,
            team_sums.2,
            team_sums.3,
            team_sums.3 as f32 / team_sums.2.max(1) as f32
        );
        let crossed_down = tex
            .iter()
            .zip(&team)
            .filter(|((_, x, _), (_, m, _))| x.passes(&HUE) && !m.passes(&HUE))
            .count();
        // Item 4103: the silhouette's colour over the nearer pixel, each
        // object beside the team colour alone, every crossing named.
        let mut sil_sums = (0u32, 0u32, 0u32, 0u32);
        for ((_, m, (who, o, piece, building)), (_, q, _)) in team.iter().zip(&sil) {
            let (a, b) = (m.passes(&HUE), q.passes(&HUE));
            println!(
                "silhouette {who}/{o} piece {piece}: team lit {} agree {} mean {:.3} {} | silhouette lit {} agree {} mean {:.3} {} behind {} part {}{}",
                m.lit,
                m.agree,
                m.mean,
                if a { "in" } else { "out" },
                q.lit,
                q.agree,
                q.mean,
                if b { "in" } else { "out" },
                q.behind,
                q.behind_part,
                match (a, b) {
                    (false, true) => " CROSSES UP",
                    (true, false) => " CROSSES DOWN",
                    _ => "",
                }
            );
            if !*building {
                sil_sums.0 += q.lit;
                sil_sums.1 += q.agree;
                sil_sums.2 += q.behind;
                sil_sums.3 += q.behind_part;
            }
        }
        println!(
            "silhouette, the figures: {} lit, {} agree; behind {}, {} part ({:.3})",
            sil_sums.0,
            sil_sums.1,
            sil_sums.2,
            sil_sums.3,
            sil_sums.3 as f32 / sil_sums.2.max(1) as f32
        );
        let sil_down = team
            .iter()
            .zip(&sil)
            .filter(|((_, m, _), (_, q, _))| m.passes(&HUE) && !q.passes(&HUE))
            .count();
        let fs: Vec<Footprint> = tex.iter().map(|(f, _, _)| *f).collect();
        let silhouettes = score(&fs, &READING);
        let (t, x, o) = (count(&tint), count(&tex), count(&off));
        let m = count(&team);
        let q = count(&sil);
        println!(
            "silhouettes {silhouettes:?}; colour: tint {t:?} texture {x:?} team {m:?} silhouette {q:?} run1441's camera {o:?}"
        );
        for (f, c, (who, o, _, _)) in &off {
            if c.lit > 0 {
                println!(
                    "run1441's camera {who}/{o}: overlap {:.3} lit {} agree {} mean {:.3}",
                    f.overlap(),
                    c.lit,
                    c.agree,
                    c.mean
                );
            }
        }
        use render::silhouette as s;
        assert_eq!(silhouettes, s::RUN1559B_3279_TRACED, "the row's draw");
        assert_eq!(q, s::RUN1559B_3279_COLOUR);
        assert_eq!(m, s::RUN1559B_3279_COLOUR_TEAM);
        assert_eq!(x, s::RUN1559B_3279_COLOUR_TEXEL);
        assert_eq!(t, s::RUN1559B_3279_COLOUR_TINT);
        assert_eq!(crossed_down, 0, "no object crosses down");
        assert_eq!(
            sil_down, 0,
            "no object crosses down on the silhouette's colour"
        );
        assert_eq!(o, s::RUN1441_CAMERA_3279_COLOUR);
        let figures_in = |rows: &[(Footprint, render::silhouette::Colours, Who)]| {
            rows.iter()
                .filter(|(_, c, w)| !w.3 && c.passes(&HUE))
                .count()
        };
        assert_eq!(figures_in(&tint), 0, "the tint reads every figure out");
        assert!(figures_in(&tex) > figures_in(&tint), "the texture is read");
        world.blend = crate::world::Blend::Current;
    }

    /// **Item 4099, step 2: a texel at a triangle's corner, by value,
    /// before the draw.** One rifleman (piece 60, 1/35, the figure whose
    /// point sits on the photograph's, 4095): the triangle that covers most
    /// of its held, lit pixels; its corners' texcoords and the texel each
    /// lands on in the `<UNIT>`'s `texture=`; and over the pixels it covers,
    /// our colour against the photograph's. Beside the binding, the other
    /// readings: `v` flipped, and `u`, `v` swapped. Every figure's player
    /// texture is named, with the share of its texels under alpha 255.
    #[test]
    fn the_rifleman_s_texel_at_a_triangle_s_corner() {
        use render::silhouette::{HUE, Paint, chromaticity, paint, read_colours, traced_theirs};
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
        world.run_to(PHOTO_FRAME);
        world.blend = crate::world::Blend::Last;
        world.run_to(PHOTO_FRAME + 1);
        let size = (photo.width, photo.height);
        let scene = frame_scene(&mut world, &camera, size);
        let p = Photo {
            width: photo.width,
            height: photo.height,
            rgba: &photo.rgba,
        };
        let traces: Vec<Trace<'_>> = TRACED
            .iter()
            .map(|&(name, poly)| Trace {
                key: scene
                    .who
                    .iter()
                    .position(|&(who, o, _, _)| format!("{who}/{o}") == name)
                    .map_or(u32::MAX, |i| i as u32 + 1),
                poly,
            })
            .collect();
        let rows = footprints_traced(
            &scene.keys,
            &p,
            &INTERFACE,
            &READING,
            &traces,
            Neighbour::Neither,
            Interface::Neither,
        );
        let mut pieces: Vec<i32> = scene.drawings.iter().map(|d| d.piece).collect();
        pieces.sort();
        pieces.dedup();
        for piece in pieces {
            let name = world.figure_texture_name(piece).map(str::to_string);
            let Some(m) = world.figure_texture(piece) else {
                println!("piece {piece}: texture {name:?} does not decode");
                continue;
            };
            let img = &m.levels[0];
            let under = img
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|c| c[3] < 255)
                .count();
            let d = scene
                .drawings
                .iter()
                .find(|d| d.piece == piece)
                .expect("drawn");
            let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
            for t in &d.uvs {
                for c in t {
                    for k in 0..2 {
                        lo[k] = lo[k].min(c[k]);
                        hi[k] = hi[k].max(c[k]);
                    }
                }
            }
            println!(
                "piece {piece}: texture {name:?} {}x{}, {under} of {} texels under alpha 255; texcoords u {:.3}..{:.3} v {:.3}..{:.3}",
                img.width,
                img.height,
                img.width * img.height,
                lo[0],
                hi[0],
                lo[1],
                hi[1]
            );
        }
        for name in ["1/35", "1/20", "1/27"] {
            let k = scene
                .who
                .iter()
                .position(|&(who, o, _, _)| format!("{who}/{o}") == name)
                .expect("drawn");
            let key = k as u32 + 1;
            let d = &scene.drawings[k];
            let f = rows.iter().find(|f| f.key == key).expect("scored");
            let theirs = traced_theirs(
                &scene.keys,
                &p,
                &INTERFACE,
                &READING,
                &traces,
                Neighbour::Neither,
                Interface::Neither,
                key,
                f.window,
            );
            let mips = world.figure_texture(d.piece).expect("the texture decodes");
            let object = Object {
                key,
                tris: d.tris.clone(),
            };
            let flipped: Vec<[[f32; 2]; 3]> = d
                .uvs
                .iter()
                .map(|t| t.map(|c| [c[0], 1.0 - c[1]]))
                .collect();
            let swapped: Vec<[[f32; 2]; 3]> =
                d.uvs.iter().map(|t| t.map(|c| [c[1], c[0]])).collect();
            let mut readings = Vec::new();
            for (label, uvs) in [
                ("bound", &d.uvs),
                ("v flipped", &flipped),
                ("u, v swapped", &swapped),
            ] {
                let how = Paint::Texture {
                    uvs,
                    mips: &mips,
                    owner: None,
                };
                let ours = paint(&object, &how, &camera, size, f.window);
                let c = read_colours(
                    &scene.keys,
                    &p,
                    key,
                    f.window,
                    &theirs,
                    &ours,
                    &HUE,
                    |_, _| false,
                );
                println!(
                    "{name} piece {} {label}: lit {} agree {} mean {:.3}",
                    d.piece, c.lit, c.agree, c.mean
                );
                readings.push((label, ours, c));
            }
            // The binding is the reading the figure agrees with most.
            let agree: Vec<u32> = readings.iter().map(|r| r.2.agree).collect();
            assert!(
                agree[0] > agree[1] && agree[0] > agree[2],
                "{name}: the texcoords bound as stored read nearest {agree:?}"
            );
            // The named triangle: the one covering most held, lit pixels.
            let ours = &readings[0].1;
            let ww = (f.window.x1 - f.window.x0) as usize;
            let mut by_tri: std::collections::BTreeMap<u32, Vec<(i32, i32)>> = Default::default();
            for y in f.window.y0..f.window.y1 {
                for x in f.window.x0..f.window.x1 {
                    let j = (y - f.window.y0) as usize * ww + (x - f.window.x0) as usize;
                    if scene.keys.at(x as usize, y as usize) != key || theirs[j] != Some(true) {
                        continue;
                    }
                    if let Some(t) = ours[j] {
                        by_tri.entry(t.tri).or_default().push((x, y));
                    }
                }
            }
            let (&tri, pixels) = by_tri
                .iter()
                .max_by_key(|(t, v)| (v.len(), std::cmp::Reverse(**t)))
                .expect("a triangle");
            let mesh = world.mesh(d.piece, false).expect("the mesh");
            let face = mesh.faces[tri as usize];
            let img = &mips.levels[0];
            println!(
                "{name} triangle {tri}, vertices {face:?}, covers {} held pixels",
                pixels.len()
            );
            for (c, v) in d.uvs[tri as usize].iter().zip(face) {
                let (tx, ty) = (
                    ((c[0] * img.width as f32).floor() as i64).rem_euclid(img.width as i64)
                        as usize,
                    ((c[1] * img.height as f32).floor() as i64).rem_euclid(img.height as i64)
                        as usize,
                );
                let t = &img.rgba[(ty * img.width + tx) * 4..][..4];
                let fy = img.height - 1 - ty;
                let tf = &img.rgba[(fy * img.width + tx) * 4..][..4];
                println!(
                    "  vertex {v}: uv ({:.4}, {:.4}) texel ({tx}, {ty}) rgba {t:?}; v flipped ({tx}, {fy}) rgba {tf:?}",
                    c[0], c[1]
                );
            }
            for (label, painted, _) in &readings {
                let mut sum_o = [0f32; 3];
                let mut sum_t = [0f32; 3];
                for &(x, y) in pixels {
                    let j = (y - f.window.y0) as usize * ww + (x - f.window.x0) as usize;
                    let o = painted[j].map_or([0.0; 3], |t| t.rgb);
                    let i = (y as usize * p.width + x as usize) * 4;
                    for c in 0..3 {
                        sum_o[c] += o[c];
                        sum_t[c] += f32::from(p.rgba[i + c]);
                    }
                }
                let n = pixels.len() as f32;
                let (mo, mt) = (sum_o.map(|v| v / n), sum_t.map(|v| v / n));
                let (a, b) = (chromaticity(mo), chromaticity(mt));
                println!(
                    "  {label}: ours {:?} photograph {:?}, chromaticity ({:.3}, {:.3}) against ({:.3}, {:.3}), distance {:.3}",
                    mo.map(|v| v.round() as i32),
                    mt.map(|v| v.round() as i32),
                    a[0],
                    a[1],
                    b[0],
                    b[1],
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
                );
                if *label == "bound" && name != "1/27" {
                    let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
                    assert!(
                        d <= HUE.distance,
                        "{name}: the named triangle agrees by value"
                    );
                }
            }
        }
        world.blend = crate::world::Blend::Current;
    }

    /// **Item 4092, step 1: the pose as a value.** What `GuyOut::draw
    /// @005dc490` hands the skeleton for a guy: its `anim` slot names the
    /// clip through the piece's `AnimationPacket::action_ids`, and the time
    /// is `cur_time · 200 / 3` (+ `graphic_dt`) clamped to `animmgr.times`;
    /// `draw_unit@008e7da0` writes both to the packet (`8e8be6`) and the
    /// transform method hands them to `HierObj::set_to_time@00536e30`
    /// (`9247a4`). For each rifleman by the tank and the two controls, at
    /// the photograph's frame and the packet's: our slot, clock, clip and
    /// time, and every node's two keys, beside the original's own
    /// `set_to_time` run on run1559b's packet (`tree_points.py --pose`).
    #[test]
    fn the_riflemen_s_pose_against_the_packet() {
        let install = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let mut world = match World::open(&install, &crate::world::gamelog_dir()) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        let path = photo_path();
        let Ok(bytes) = std::fs::read(&path) else {
            println!("skipping: no photograph at {}", path.display());
            return;
        };
        let photo = render::tga::decode(&bytes).expect("the photograph decodes");
        let size = (photo.width, photo.height);
        let camera = crate::pinned_camera();
        let mut rows: Vec<String> = Vec::new();
        for frame in [PHOTO_FRAME, 3294] {
            world.run_to(frame);
            let (figures, _, _) = world.draw_list(
                &camera,
                &crate::pinned_screen(),
                (size.0 as i32, size.1 as i32),
            );
            for o in [20i16, 31, 32, 35, 38] {
                let p = figures
                    .iter()
                    .find(|p| (p.who, p.o) == (1, o))
                    .expect("placed");
                let files = world.unit_slot_files(p.piece).expect("a unit piece");
                let mesh = world.mesh(p.piece, false).expect("the piece decodes");
                let file = files
                    .iter()
                    .find(|(s, _)| *s == p.slot)
                    .map_or("-", |(_, f)| f.as_str());
                let anim = mesh.anims.get(&p.slot).expect("the slot names a clip");
                let ms = crate::world::anim_ms(p.frame, anim);
                let len = anim
                    .played()
                    .first()
                    .and_then(|n| n.times().last().copied())
                    .unwrap_or(0);
                rows.push(format!(
                    "fig 1/{o} frame {frame} piece {} slot {} clock {} file {file} len {len} ms {ms}",
                    p.piece, p.slot, p.frame
                ));
                let joints = render::figure::joints(&mesh.model, anim, ms);
                for (i, (n, k)) in mesh
                    .model
                    .nodes
                    .iter()
                    .zip(render::figure::keyed(&mesh.model, anim, ms))
                    .enumerate()
                {
                    let w = joints[i].map(|v| format!("{v:.3}")).join(" ");
                    let k = k.map_or("rest".to_string(), |k| {
                        format!(
                            "keys {} {} q {} t {}",
                            k.keys.0,
                            k.keys.1,
                            k.rotation.map(|v| format!("{v:.6}")).join(" "),
                            k.translation.map(|v| format!("{v:.4}")).join(" ")
                        )
                    });
                    rows.push(format!(
                        "  node {i} start {} num {} parent {} {k} world {w}",
                        n.start,
                        n.num,
                        n.parent.map_or(-1, |p| p as i64)
                    ));
                }
            }
        }
        for r in &rows {
            println!("{r}");
        }
        // The original's own `HierObj::set_to_time@00536e30`, then
        // `update_all_transforms@00536db0`, on run1559b's packet (piece 60's
        // skeleton, `RData +0x48 → +8`, its 35 nodes): the world origin
        // (`world_transform.trans`, +0x74) of the pelvis, the two feet and
        // the two hands, theirs 4, 8, 30, 18, 23 — ours 4, 8, 33, 19, 25,
        // the 35 aligned by `(start, num)` with ours' five vertex-less nodes
        // 10, 21, 30, 38, 39 left out. `(o, slot, clock, ms, joints)`.
        type Joints = [[f32; 3]; 5];
        const PACKET_POSE: [(i16, i8, u32, u32, Joints); 10] = [
            (
                20,
                8,
                12,
                800,
                [
                    [1.15, 1.69, 20.88],
                    [2.67, 0.68, 1.12],
                    [-0.58, 10.93, 4.89],
                    [6.284, -5.371, 24.172],
                    [-3.19, -1.74, 24.74],
                ],
            ),
            (
                31,
                11,
                8,
                533,
                [
                    [-0.23, -1.98, 21.50],
                    [3.20, -3.83, 2.22],
                    [-1.94, 4.31, 2.27],
                    [-2.17, -14.00, 32.78],
                    [-4.62, -4.12, 34.74],
                ],
            ),
            (
                32,
                0,
                14,
                933,
                [
                    [0.05, -0.11, 22.07],
                    [2.66, -4.33, 2.22],
                    [-2.37, 3.88, 2.28],
                    [-2.00, -9.84, 25.09],
                    [-4.98, 1.24, 25.74],
                ],
            ),
            (
                35,
                9,
                1,
                66,
                [
                    [-0.76, 1.42, 21.66],
                    [0.42, 2.59, 2.43],
                    [-2.87, 10.06, 14.29],
                    [5.41, -5.34, 24.06],
                    [-2.54, -5.16, 27.03],
                ],
            ),
            (
                38,
                0,
                10,
                666,
                [
                    [0.05, -0.11, 22.07],
                    [2.74, -4.23, 2.21],
                    [-2.29, 3.98, 2.28],
                    [-2.08, -9.91, 24.93],
                    [-5.08, 1.21, 25.58],
                ],
            ),
            (
                20,
                0,
                15,
                1000,
                [
                    [0.05, -0.11, 22.07],
                    [2.63, -4.36, 2.22],
                    [-2.40, 3.85, 2.28],
                    [-1.99, -9.86, 25.12],
                    [-4.99, 1.26, 25.75],
                ],
            ),
            (
                31,
                11,
                23,
                1533,
                [
                    [-0.20, -2.06, 21.48],
                    [3.15, -3.85, 2.13],
                    [-1.88, 4.34, 2.22],
                    [-1.48, -13.21, 32.13],
                    [-4.20, -4.10, 33.64],
                ],
            ),
            (
                32,
                0,
                29,
                1933,
                [
                    [0.05, -0.11, 22.07],
                    [3.06, -3.94, 2.18],
                    [-1.97, 4.29, 2.29],
                    [-2.27, -10.00, 24.53],
                    [-5.27, 1.07, 25.22],
                ],
            ),
            (
                35,
                9,
                4,
                266,
                [
                    [-0.97, 1.17, 23.50],
                    [1.42, 13.49, 7.67],
                    [-3.46, -1.23, 4.66],
                    [3.64, -7.02, 26.59],
                    [-4.65, -2.96, 28.52],
                ],
            ),
            (
                38,
                0,
                25,
                1666,
                [
                    [0.05, -0.11, 22.07],
                    [2.98, -4.03, 2.19],
                    [-2.05, 4.19, 2.29],
                    [-2.16, -9.95, 24.70],
                    [-5.17, 1.10, 25.36],
                ],
            ),
        ];
        const OURS: [usize; 5] = [4, 8, 33, 19, 25];
        let mesh = world.mesh(60, false).expect("the rifleman decodes");
        let parted = |slot: i8, ms: u32, want: &Joints| -> f32 {
            let j = render::figure::joints(&mesh.model, &mesh.anims[&slot], ms);
            OURS.iter()
                .zip(want)
                .flat_map(|(&i, w)| {
                    let at = j[i];
                    (0..3).map(move |c| (at[c] - w[c]).abs())
                })
                .fold(0.0, f32::max)
        };
        for (k, (o, slot, clock, ms, want)) in PACKET_POSE.iter().enumerate() {
            let at = if k < 5 { PHOTO_FRAME } else { 3294 };
            let ours = rows
                .iter()
                .find(|r| r.starts_with(&format!("fig 1/{o} frame {at} ")))
                .expect("a row");
            assert!(
                ours.contains(&format!(" slot {slot} clock {clock} "))
                    && ours.ends_with(&format!(" ms {ms}")),
                "ours names the packet's clip and time: {ours}"
            );
            let d = parted(*slot, *ms, want);
            println!("1/{o} at {at}: joints within {d:.3} of the original's");
            assert!(d < 0.5, "1/{o}'s pose at {at} is the original's");
        }
        // Seen to fail: 1/20's clock three ticks on (800 → 1000 ms).
        let (_, slot, _, _, want) = PACKET_POSE[0];
        let d = parted(slot, 1000, &want);
        println!("1/20 at 1000 ms: {d:.3} off");
        assert!(d > 2.0, "a perturbed clock parts the pose");
    }

    /// **The woodcutter's right edge** (item 4079): on frame 3279 at
    /// run1559b's camera our sprite reaches x 1700 where the photograph's
    /// traced building ends nearer 1686. The sprite is drawn as the
    /// original places it: `woodcutter_6.tga`, 173 × 201, its alpha two-level
    /// (0 or 255) in every column past screen x 1688 and its last column
    /// opaque; at the anchor (1640, 438) less
    /// `(int)(83 · s)`, `(int)(178 · s)`, `s` = 13905 / 20858, its left is
    /// 1585 and its right 1585 + 173 · s = 1700.3. The 307 of its pixels in
    /// 0/10's window that lie outside the trace are the crate's lower right,
    /// which the photograph shows behind a pine the client does not draw
    /// (parked 4079a).
    /// **A value before a cause** (item 4082, step 1): for each figure the
    /// trees hide at the woodcutter, on our own buffers at run1559b's
    /// camera on frame 3279 — its pixels drawn without trees; how many of
    /// them a tree's triangle covers; of those, how many the leaf's own
    /// texel passes (`render_states[9]`'s test, [`crate::world::TREE_ALPHA_REF`])
    /// and how many it cuts; and where a tree covers it, which is nearer by
    /// our depth and by how much (the tree's depth less the figure's,
    /// camera units: positive, the figure nearer). Prints the table; it
    /// asserts only that the five are drawn.
    #[test]
    fn the_figures_the_trees_cover_by_triangle_and_by_texel() {
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
        let size = (photo.width, photo.height);
        world.run_to(PHOTO_FRAME);
        let (figures, buildings, list) = world.draw_list(
            &camera,
            &crate::pinned_screen(),
            (size.0 as i32, size.1 as i32),
        );
        let drawings = world.objects(&figures, &buildings, &list);
        let sprites = world.sprite_drawings(&buildings, &list, camera.zoom_distance);
        let scene = rasterize_cut(
            &objects(&drawings),
            &[],
            &stamps(&sprites, drawings.len() as u32 + 1),
            &camera,
            size,
        );
        let trees = world.tree_drawings();
        for cull in [
            render::silhouette::Cull::CounterClockwise,
            render::silhouette::Cull::None,
            render::silhouette::Cull::Clockwise,
        ] {
            world.tree_cull = cull;
            println!("cull {cull:?}");
            let cuts = world.tree_cuts();
            assert!(!trees.is_empty() && cuts.iter().all(Option::is_some));
            let tree_objects = objects(&trees);
            let solid = rasterize_cut(&tree_objects, &[], &[], &camera, size);
            let cut: Vec<_> = cuts.iter().map(Option::as_ref).collect();
            let leaves = rasterize_cut(&tree_objects, &cut, &[], &camera, size);
            let count =
                |r: &render::silhouette::Raster| r.depth.iter().filter(|d| d.is_finite()).count();
            let lone: (usize, usize) = (0..trees.len())
                .map(|j| {
                    let one = [Object {
                        key: 1,
                        tris: trees[j].tris.clone(),
                    }];
                    let c = [cut[j]];
                    (
                        count(&rasterize_cut(&one, &[], &[], &camera, size)),
                        count(&rasterize_cut(&one, &c, &[], &camera, size)),
                    )
                })
                .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
            println!(
                "trees: solid {} pixels, by texel {}; each tree alone, summed: solid {} by texel {}",
                count(&solid),
                count(&leaves),
                lone.0,
                lone.1
            );
            let mut drawn = 0;
            for o in [1i16, 2, 6, 8, 10] {
                let keys: Vec<u32> = drawings
                    .iter()
                    .enumerate()
                    .filter(|(_, d)| (d.who, d.o, d.building) == (0, o, false))
                    .map(|(i, _)| i as u32 + 1)
                    .collect();
                let pixels: Vec<usize> = (0..scene.keys.keys.len())
                    .filter(|&i| keys.contains(&scene.keys.keys[i]))
                    .collect();
                drawn += !pixels.is_empty() as usize;
                let covered: Vec<usize> = pixels
                    .iter()
                    .copied()
                    .filter(|&i| solid.depth[i].is_finite())
                    .collect();
                let passed = covered
                    .iter()
                    .filter(|&&i| leaves.depth[i].is_finite())
                    .count();
                let mut gaps: Vec<f32> = covered
                    .iter()
                    .map(|&i| solid.depth[i] - scene.depth[i])
                    .collect();
                gaps.sort_by(f32::total_cmp);
                let nearer = gaps.iter().filter(|g| **g > 0.0).count();
                let mut leaf_gaps: Vec<f32> = covered
                    .iter()
                    .filter(|&&i| leaves.depth[i].is_finite())
                    .map(|&i| leaves.depth[i] - scene.depth[i])
                    .collect();
                leaf_gaps.sort_by(f32::total_cmp);
                let leaf_nearer = leaf_gaps.iter().filter(|g| **g > 0.0).count();
                let median = |g: &[f32]| g.get(g.len() / 2).copied().unwrap_or(f32::NAN);
                let kept_solid = pixels.len() - (covered.len() - nearer);
                let kept_alpha = pixels.len() - (leaf_gaps.len() - leaf_nearer);
                println!(
                    "figure 0/{o}: pixels {} covered {} texel passes {} cuts {}; covered, figure nearer {} tree nearer {} (median tree-figure {:.1}, range {:.1}..{:.1}); passed, figure nearer {} tree nearer {} (median {:.1}); kept solid {} kept by texel {}",
                    pixels.len(),
                    covered.len(),
                    passed,
                    covered.len() - passed,
                    nearer,
                    covered.len() - nearer,
                    median(&gaps),
                    gaps.first().copied().unwrap_or(f32::NAN),
                    gaps.last().copied().unwrap_or(f32::NAN),
                    leaf_nearer,
                    leaf_gaps.len() - leaf_nearer,
                    median(&leaf_gaps),
                    kept_solid,
                    kept_alpha,
                );
            }
            assert_eq!(drawn, 5, "the five figures are drawn");
        }
    }

    #[test]
    fn the_photograph_on_the_pixels_the_pine_covers() {
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
        let size = (photo.width, photo.height);
        world.run_to(PHOTO_FRAME);
        let (figures, buildings, list) = world.draw_list(
            &camera,
            &crate::pinned_screen(),
            (size.0 as i32, size.1 as i32),
        );
        let drawings = world.objects(&figures, &buildings, &list);
        let sprites = world.sprite_drawings(&buildings, &list, camera.zoom_distance);
        let scene = rasterize_cut(
            &objects(&drawings),
            &[],
            &stamps(&sprites, drawings.len() as u32 + 1),
            &camera,
            size,
        );
        let trees = world.tree_drawings();
        let cuts = world.tree_cuts();
        let cut: Vec<_> = cuts.iter().map(Option::as_ref).collect();
        let leaves = rasterize_cut(&objects(&trees), &cut, &[], &camera, size);
        let key_of = |o: i16| -> Vec<u32> {
            drawings
                .iter()
                .enumerate()
                .filter(|(_, d)| (d.who, d.o, d.building) == (0, o, false))
                .map(|(i, _)| i as u32 + 1)
                .collect()
        };
        let w = size.0;
        let mut boxes = Vec::new();
        for o in [1i16, 2, 8, 10, 6, 9] {
            let keys = key_of(o);
            let px: Vec<usize> = (0..scene.keys.keys.len())
                .filter(|&i| keys.contains(&scene.keys.keys[i]))
                .collect();
            let (x0, x1) = (
                px.iter().map(|i| i % w).min().unwrap(),
                px.iter().map(|i| i % w).max().unwrap(),
            );
            let (y0, y1) = (
                px.iter().map(|i| i / w).min().unwrap(),
                px.iter().map(|i| i / w).max().unwrap(),
            );
            println!(
                "figure 0/{o}: {} px, x {x0}..={x1}, y {y0}..={y1}",
                px.len()
            );
            boxes.push((o, x0, x1, y0, y1));
        }
        // Every figure's pixels, ours, dilated by two: the leaf's reference
        // excludes them.
        let all_figs: Vec<u32> = boxes.iter().flat_map(|b| key_of(b.0)).collect();
        let fig_at = |x: i64, y: i64| -> bool {
            (-2..=2).any(|dy| {
                (-2..=2).any(|dx| {
                    let (xx, yy) = (x + dx, y + dy);
                    xx >= 0
                        && yy >= 0
                        && all_figs.contains(&scene.keys.keys[yy as usize * w + xx as usize])
                })
            })
        };
        let rgb = |i: usize| -> [f32; 3] {
            [
                photo.rgba[i * 4] as f32,
                photo.rgba[i * 4 + 1] as f32,
                photo.rgba[i * 4 + 2] as f32,
            ]
        };
        // The photograph's flat red: red over both other channels by 60.
        let red = |c: [f32; 3]| c[0] - c[1] > 60.0 && c[0] - c[2] > 60.0;
        // The leaf under a pixel: the mean of the photograph within seven
        // pixels, under our pine, clear of every figure, and not red.
        let leaf = |x: usize, y: usize| -> Option<[f32; 3]> {
            let (mut sum, mut n) = ([0f32; 3], 0f32);
            for yy in y.saturating_sub(7)..=y + 7 {
                for xx in x.saturating_sub(7)..=x + 7 {
                    let i = yy * w + xx;
                    let c = rgb(i);
                    if leaves.depth[i].is_finite() && !fig_at(xx as i64, yy as i64) && !red(c) {
                        for k in 0..3 {
                            sum[k] += c[k];
                        }
                        n += 1.0;
                    }
                }
            }
            (n >= 8.0).then(|| sum.map(|v| v / n))
        };
        // The figure's share of a pixel, under `c = a * (255, 0, 0) + (1 - a) * leaf`,
        // read on the red and on the green channel apart.
        let classify = |name: &str, px: &[usize]| {
            let (mut bins, mut red_n, mut no_leaf) = ([0usize; 11], 0, 0);
            let mut ar = Vec::new();
            let mut ag = Vec::new();
            for &i in px {
                let (x, y) = (i % w, i / w);
                let c = rgb(i);
                red_n += red(c) as usize;
                let Some(l) = leaf(x, y) else {
                    no_leaf += 1;
                    continue;
                };
                let a_g = (1.0 - c[1] / l[1].max(1.0)).clamp(-0.5, 1.5);
                let a_r = ((c[0] - l[0]) / (255.0 - l[0]).max(1.0)).clamp(-0.5, 1.5);
                // The share of `render_states[1]`'s tfactor colour (255, 82, 34)
                // over the leaf, least squares on all three channels.
                let t = [255.0f32, 82.0, 34.0];
                let (mut num, mut den) = (0f32, 0f32);
                for k in 0..3 {
                    num += (c[k] - l[k]) * (t[k] - l[k]);
                    den += (t[k] - l[k]) * (t[k] - l[k]);
                }
                let a_t = (num / den.max(1.0)).clamp(0.0, 1.0);
                bins[(a_t * 10.0).round() as usize] += 1;
                ar.push(a_r);
                ag.push(a_g);
            }
            ar.sort_by(f32::total_cmp);
            ag.sort_by(f32::total_cmp);
            let q = |v: &[f32], f: f32| {
                v.get(((v.len() as f32 - 1.0) * f) as usize)
                    .copied()
                    .unwrap_or(f32::NAN)
            };
            println!(
                "{name}: {} px, red {red_n}, no leaf {no_leaf}; a by tfactor, tenths 0..=10 {bins:?}; a_g q10/50/90 {:.2}/{:.2}/{:.2}; a_r q10/50/90 {:.2}/{:.2}/{:.2}",
                px.len(),
                q(&ag, 0.1),
                q(&ag, 0.5),
                q(&ag, 0.9),
                q(&ar, 0.1),
                q(&ar, 0.5),
                q(&ar, 0.9),
            );
            let mode = (0..11).max_by_key(|&b| bins[b]).unwrap();
            (red_n, mode)
        };
        for &(o, ..) in &boxes {
            let keys = key_of(o);
            let px: Vec<usize> = (0..scene.keys.keys.len())
                .filter(|&i| keys.contains(&scene.keys.keys[i]) && leaves.depth[i].is_finite())
                .collect();
            let (red_n, mode) = classify(&format!("0/{o} under the pine"), &px);
            let shifted: Vec<usize> = px.iter().map(|i| i + 10).collect();
            let (red_shifted, mode_shifted) = classify(&format!("0/{o} shifted 10 px"), &shifted);
            if [1, 2, 8, 10].contains(&o) {
                // The silhouette, `render_states[1]`'s tfactor at 128/255
                // over the leaf (item 4085): red on nine in ten of the
                // pixels the pine covers, its share's mode a half or more;
                // ten pixels over, neither.
                assert!(red_n * 10 >= px.len() * 9, "0/{o} is red");
                assert!((5..=7).contains(&mode), "0/{o}'s share is the state's");
                assert!(red_shifted * 4 <= px.len(), "0/{o} shifted is not red");
                assert_eq!(mode_shifted, 0, "0/{o} shifted is the leaf");
            }
            // Where the photograph's red stands against our figure.
            let (x0, x1, y0, y1) = boxes
                .iter()
                .find(|b| b.0 == o)
                .map(|b| (b.1 - 6, b.2 + 6, b.3 - 6, b.4 + 6))
                .unwrap();
            let (mut ours_red, mut ours_n, mut red_out) = (0, 0, 0);
            let (mut cr, mut co) = ((0f32, 0f32, 0f32), (0f32, 0f32, 0f32));
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let i = y * w + x;
                    let mine = keys.contains(&scene.keys.keys[i]);
                    let r = red(rgb(i));
                    ours_n += mine as usize;
                    ours_red += (mine && r) as usize;
                    red_out += (!mine && r && !fig_at(x as i64, y as i64)) as usize;
                    if mine {
                        co = (co.0 + x as f32, co.1 + y as f32, co.2 + 1.0);
                    }
                    if r {
                        cr = (cr.0 + x as f32, cr.1 + y as f32, cr.2 + 1.0);
                    }
                }
            }
            println!(
                "0/{o} place: ours {ours_n}, of them red {ours_red}; red clear of every figure {red_out}; centroid ours ({:.1}, {:.1}) red ({:.1}, {:.1})",
                co.0 / co.2,
                co.1 / co.2,
                cr.0 / cr.2,
                cr.1 / cr.2
            );
        }
        // The woodcutter, which no tree covers: its own pixels.
        let wood: Vec<u32> = (0..drawings.len() + sprites.len())
            .filter(|&j| {
                if j < drawings.len() {
                    (drawings[j].who, drawings[j].o) == (0, 2001)
                } else {
                    (
                        sprites[j - drawings.len()].who,
                        sprites[j - drawings.len()].o,
                    ) == (0, 2001)
                }
            })
            .map(|j| j as u32 + 1)
            .collect();
        let wpx: Vec<usize> = (0..scene.keys.keys.len())
            .filter(|&i| wood.contains(&scene.keys.keys[i]))
            .collect();
        assert_eq!(
            classify("0/2001, the woodcutter", &wpx).1,
            0,
            "no silhouette on the woodcutter"
        );
        // The pine by row: ours, the one tree over the figures alone, and
        // the photograph's foliage walked left from inside ours.
        let pine = (0..trees.len())
            .max_by_key(|&j| {
                let one = [Object {
                    key: 1,
                    tris: trees[j].tris.clone(),
                }];
                let r = rasterize_cut(&one, &[cut[j]], &[], &camera, size);
                (0..r.keys.keys.len())
                    .filter(|&i| r.depth[i].is_finite() && all_figs.contains(&scene.keys.keys[i]))
                    .count()
            })
            .unwrap();
        let lone = rasterize_cut(
            &[Object {
                key: 1,
                tris: trees[pine].tris.clone(),
            }],
            &[cut[pine]],
            &[],
            &camera,
            size,
        );
        let foliage = |c: [f32; 3]| {
            let grass = c[1] > 135.0 && c[0] > 130.0;
            !grass && !red(c) && c[1] >= c[2] + 15.0
        };
        println!("the pine over the figures is tree {pine}");
        for y in (384..=480).step_by(4) {
            let row: Vec<usize> = (1600..1800)
                .filter(|&x| lone.depth[y * w + x].is_finite())
                .collect();
            let all = (1560..1800).find(|&x| leaves.depth[y * w + x].is_finite());
            let (Some(&l), Some(&r)) = (row.first(), row.last()) else {
                continue;
            };
            let mut t = l + 5;
            while t > 1560 && foliage(rgb(y * w + t - 1)) {
                t -= 1;
            }
            println!(
                "row {y}: pine ours {l}..={r}; every tree ours from {all:?}; the photograph's foliage from {t}"
            );
        }
        if let Some(dir) = std::env::var_os("PROBE_DIR") {
            let dir = PathBuf::from(dir);
            let (x0, x1, y0, y1) = (1540usize, 1760usize, 300usize, 500usize);
            let mut a = format!("P6 {} {} 255\n", x1 - x0, y1 - y0).into_bytes();
            let mut b = a.clone();
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = y * w + x;
                    let c = &photo.rgba[i * 4..i * 4 + 3];
                    a.extend_from_slice(c);
                    let k = scene.keys.keys[i];
                    let fig = boxes.iter().position(|b| key_of(b.0).contains(&k));
                    let pine = leaves.depth[i].is_finite();
                    let o = match (fig, pine) {
                        (Some(f), true) => [255, (40 * f) as u8, 255],
                        (Some(f), false) => [255, (40 * f) as u8, 0],
                        (None, true) => [0, 160, 0],
                        (None, false) => [c[0] / 3, c[1] / 3, c[2] / 3],
                    };
                    b.extend_from_slice(&o);
                }
            }
            std::fs::write(dir.join("photo.ppm"), a).unwrap();
            std::fs::write(dir.join("ours.ppm"), b).unwrap();
        }
    }

    #[test]
    fn the_woodcutter_s_right_edge_is_the_sprite_s_own() {
        let install = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let mut world = match World::open(&install, &crate::world::gamelog_dir()) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        let camera = crate::pinned_camera();
        // The sprite's own edge, read on the treeless draw (item 4080), the
        // figures hidden where it is nearer: with the silhouettes (item
        // 4085) they take 61 of these pixels, (307, 99) → (246, 80).
        world.trees = false;
        world.silhouettes = crate::world::Silhouettes::None;
        world.run_to(PHOTO_FRAME);
        let (_, buildings, list) = world.draw_list(&camera, &crate::pinned_screen(), (1920, 1080));
        let d = list
            .iter()
            .find(|d| (d.who, d.o) == (0, 2001))
            .expect("listed");
        assert_eq!((d.anchor.x, d.anchor.y), (1640, 438));
        let sprites = world.sprite_drawings(&buildings, &list, camera.zoom_distance);
        let sp = sprites
            .iter()
            .find(|s| (s.who, s.o) == (0, 2001))
            .expect("drawn as its sprite");
        let im = &sp.image.image;
        assert_eq!((im.width, im.height), (173, 201));
        assert_eq!(sp.image.offset, (83, 178));
        assert_eq!((sp.at.left, sp.at.top), (1585, 320));
        // Partial alpha, over the image and in its columns past screen x
        // 1688 (u >= 155).
        let partial = |u0: usize| {
            (0..im.height)
                .flat_map(|v| (u0..im.width).map(move |u| (v, u)))
                .filter(|&(v, u)| {
                    let a = im.rgba[(v * im.width + u) * 4 + 3];
                    a != 0 && a != 255
                })
                .count()
        };
        let opaque_past = (0..im.height)
            .flat_map(|v| (155..im.width).map(move |u| (v, u)))
            .filter(|&(v, u)| im.rgba[(v * im.width + u) * 4 + 3] == 255)
            .count();
        println!(
            "partial alpha: {} in the image, {} past column 155; opaque past it {opaque_past}",
            partial(0),
            partial(155)
        );
        assert_eq!(partial(155), 0, "the right columns are two-level");
        let last = (0..im.height)
            .filter(|&v| im.rgba[(v * im.width + im.width - 1) * 4 + 3] == 255)
            .count();
        println!("opaque rows in the last column: {last}");
        assert!(last > 0, "the image's last column is opaque");
        let right = sp.at.left as f32 + im.width as f32 * sp.at.scale;
        println!("the sprite's right edge: {right:.1}");
        assert_eq!(right as i32, 1700);
        let (keys, who) = frame_keys(&mut world, &camera, (1920, 1080));
        let wood = who
            .iter()
            .position(|&(w, o, _, _)| (w, o) == (0, 2001))
            .expect("keyed") as u32
            + 1;
        let trace = TRACED
            .iter()
            .find(|(n, _)| *n == "0/2001")
            .expect("traced")
            .1;
        let (mut outside, mut past) = (0, 0);
        for y in 404..452i32 {
            for x in 1680..1710i32 {
                if keys.keys[y as usize * 1920 + x as usize] == wood
                    && !render::silhouette::inside(trace, x, y)
                {
                    outside += 1;
                    past += usize::from(x >= 1688);
                }
            }
        }
        println!("0/10's window: the woodcutter outside its trace {outside}, at x >= 1688 {past}");
        assert_eq!((outside, past), (307, 99));
    }

    /// **Item 4086, step 1: the riflemen by the tank, by row.** On frame
    /// 3279 at run1559b's camera, on the default draw, each of 1/31, 1/20
    /// and 1/32, and a player-1 figure in the row that stands clear, read
    /// as `read_traced` reads its window: `own` our pixels keyed to the
    /// figure, `other` ours keyed to anything else, `theirs` the
    /// photograph's pixels the window's ground does not hold. Per figure:
    /// both sides' box, size and centroid, the rows, and the window as a
    /// map. Shifted ten pixels, our figure no longer stands on theirs.
    #[test]
    fn the_riflemen_by_the_tank_by_row() {
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
        world.run_to(PHOTO_FRAME);
        let rows = score_frame_traced(
            &mut world,
            &camera,
            &photo,
            &TRACED,
            Neighbour::Neither,
            Interface::Neither,
        );
        let size = (photo.width, photo.height);
        let (keys, who) = frame_keys(&mut world, &camera, size);
        let p = Photo {
            width: photo.width,
            height: photo.height,
            rgba: &photo.rgba,
        };
        let w = size.0;
        // Each figure drawn alone: its whole extent on our draw.
        let (figures, buildings, list) = world.draw_list(
            &camera,
            &crate::pinned_screen(),
            (size.0 as i32, size.1 as i32),
        );
        let drawings = world.objects(&figures, &buildings, &list);
        let alone_of = |o: i16| {
            let objs: Vec<Object> = objects(&drawings)
                .into_iter()
                .filter(|b| {
                    let d = &drawings[b.key as usize - 1];
                    (d.who, d.o, d.building) == (1, o, false)
                })
                .collect();
            rasterize_cut(&objs, &[], &[], &camera, size).keys
        };
        let iface = |x: i32, y: i32| {
            INTERFACE
                .iter()
                .any(|r| (r.x0..r.x1).contains(&x) && (r.y0..r.y1).contains(&y))
        };
        // A clear figure: player 1's, in the row, its window holding no
        // pixel of ours but its own and no trace.
        let traces: Vec<(u32, &[(i32, i32)])> = TRACED
            .iter()
            .map(|&(name, poly)| {
                let k = who
                    .iter()
                    .position(|&(a, o, _, _)| format!("{a}/{o}") == name)
                    .map_or(u32::MAX, |i| i as u32 + 1);
                (k, poly)
            })
            .collect();
        let own_of = |f: &Footprint| -> (u8, i16) {
            let d = who[f.key as usize - 1];
            (d.0, d.1)
        };
        let clear = rows
            .iter()
            .filter(|(f, d)| d.0 == 1 && !d.3 && f.overlap() >= READING.threshold)
            .find(|(f, _)| {
                let me = own_of(f);
                let r = READING.ring;
                let win = Rect::new(
                    f.window.x0 - r,
                    f.window.y0 - r,
                    f.window.x1 + r,
                    f.window.y1 + r,
                );
                (win.y0..win.y1).all(|y| {
                    (win.x0..win.x1).all(|x| {
                        let k = keys.at(x as usize, y as usize);
                        (k == 0 || {
                            let d = who[k as usize - 1];
                            (d.0, d.1) == me && !d.3
                        }) && !traces.iter().any(|t| render::silhouette::inside(t.1, x, y))
                    })
                })
            })
            .map(|(_, d)| d.1)
            .expect("a clear figure of player 1's in the row");
        println!("the clear figure: 1/{clear}");
        let mut owed = Vec::new();
        for o in [31i16, 20, 32, clear] {
            let (f, d) = rows
                .iter()
                .find(|(_, d)| (d.0, d.1, d.3) == (1, o, false))
                .copied()
                .expect("the figure is scored");
            let win = f.window;
            let r = READING.ring;
            let outer = Rect::new(win.x0 - r, win.y0 - r, win.x1 + r, win.y1 + r);
            let key = f.key;
            let alone = alone_of(o);
            let whole = |x: i32, y: i32| alone.at(x as usize, y as usize) != 0;
            let kind = |x: i32, y: i32| -> u8 {
                if iface(x, y) {
                    return 3;
                }
                let mut k = 0;
                for t in &traces {
                    if render::silhouette::inside(t.1, x, y) {
                        if t.0 == key {
                            return 1;
                        }
                        k = 2;
                    }
                }
                k
            };
            let g = render::silhouette::ground_where(&keys, &p, win, outer, &READING, |x, y| {
                kind(x, y) == 0
            });
            let own = |x: i32, y: i32| {
                let k = keys.at(x as usize, y as usize);
                k != 0 && {
                    let e = who[k as usize - 1];
                    (e.0, e.1, e.3) == (1, o, false)
                }
            };
            let theirs = |x: i32, y: i32| kind(x, y) == 0 && !g.holds(&p, x, y);
            // Box, size, centroid and count of a mask in the window.
            let measure = |m: &dyn Fn(i32, i32) -> bool| {
                let (mut n, mut sx, mut sy) = (0i64, 0i64, 0i64);
                let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
                for y in win.y0..win.y1 {
                    for x in win.x0..win.x1 {
                        if kind(x, y) >= 2 || !m(x, y) {
                            continue;
                        }
                        n += 1;
                        sx += i64::from(x);
                        sy += i64::from(y);
                        (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
                    }
                }
                let n1 = n.max(1) as f64;
                (
                    n,
                    (x0, y0, x1, y1),
                    if n == 0 {
                        (0, 0)
                    } else {
                        (x1 - x0 + 1, y1 - y0 + 1)
                    },
                    (sx as f64 / n1, sy as f64 / n1),
                )
            };
            let mo = measure(&own);
            let mt = measure(&theirs);
            let mot = measure(&|x, y| own(x, y) && theirs(x, y));
            let mw = measure(&whole);
            let mwt = measure(&|x, y| whole(x, y) && theirs(x, y));
            println!(
                "1/{o} piece {} window ({}, {})..({}, {}): ours {} theirs {} both {} overlap {:.3}",
                d.2,
                win.x0,
                win.y0,
                win.x1,
                win.y1,
                f.ours,
                f.theirs,
                f.both,
                f.overlap()
            );
            println!(
                "  own: {} px, box {:?}, {}x{}, centroid ({:.1}, {:.1})",
                mo.0, mo.1, mo.2.0, mo.2.1, mo.3.0, mo.3.1
            );
            println!(
                "  theirs: {} px, box {:?}, {}x{}, centroid ({:.1}, {:.1}); on own {}",
                mt.0, mt.1, mt.2.0, mt.2.1, mt.3.0, mt.3.1, mot.0
            );
            println!(
                "  alone: {} px, box {:?}, {}x{}, centroid ({:.1}, {:.1}); on theirs {}; hidden in our draw {}",
                mw.0,
                mw.1,
                mw.2.0,
                mw.2.1,
                mw.3.0,
                mw.3.1,
                mwt.0,
                mw.0 - mo.0
            );
            // Who else of ours is in the window, by key.
            let mut others: std::collections::BTreeMap<String, u32> = Default::default();
            for y in win.y0..win.y1 {
                for x in win.x0..win.x1 {
                    let k = keys.at(x as usize, y as usize);
                    if k != 0 && !own(x, y) && kind(x, y) < 2 {
                        let e = who[k as usize - 1];
                        *others
                            .entry(format!("{}/{} piece {}", e.0, e.1, e.2))
                            .or_default() += 1;
                    }
                }
            }
            println!("  others of ours in the window: {others:?}");
            // What the photograph holds on our own pixels it does not read
            // as the figure's: their mean colour and how many are red.
            let (mut n, mut sum, mut reds) = (0u32, [0u32; 3], 0u32);
            for y in win.y0..win.y1 {
                for x in win.x0..win.x1 {
                    if own(x, y) && kind(x, y) == 0 && !theirs(x, y) {
                        let i = (y as usize * w + x as usize) * 4;
                        let c = [photo.rgba[i], photo.rgba[i + 1], photo.rgba[i + 2]];
                        n += 1;
                        for k in 0..3 {
                            sum[k] += u32::from(c[k]);
                        }
                        let (r, gg, b) = (i32::from(c[0]), i32::from(c[1]), i32::from(c[2]));
                        reds += u32::from(r - gg > 60 && r - b > 60);
                    }
                }
            }
            println!(
                "  own, not theirs: {n} px, mean {:?}, red {reds}",
                sum.map(|v| v / n.max(1))
            );
            // The window by row, then as a map: '#' own and theirs, 'o' own
            // only, '%' another of ours and theirs, '-' another only, '+'
            // theirs only, 'T' a neighbour's trace, '=' its own trace.
            // The photograph's red among theirs, on our own, on another of
            // ours, and on nothing of ours.
            let mut red = [0u32; 3];
            for y in win.y0..win.y1 {
                for x in win.x0..win.x1 {
                    if !theirs(x, y) {
                        continue;
                    }
                    let i = (y as usize * w + x as usize) * 4;
                    let (r, gg, b) = (
                        i32::from(photo.rgba[i]),
                        i32::from(photo.rgba[i + 1]),
                        i32::from(photo.rgba[i + 2]),
                    );
                    if r - gg > 60 && r - b > 60 {
                        let k = keys.at(x as usize, y as usize);
                        red[if own(x, y) {
                            0
                        } else if k != 0 {
                            1
                        } else {
                            2
                        }] += 1;
                    }
                }
            }
            println!(
                "  theirs red: on own {}, on another {}, on none {}",
                red[0], red[1], red[2]
            );
            println!(
                "  row: own other theirs own&theirs | map ('*' hidden by another, 'I' interface)"
            );
            for y in win.y0..win.y1 {
                let (mut a, mut b, mut c, mut e) = (0, 0, 0, 0);
                let mut line = String::new();
                for x in win.x0..win.x1 {
                    let k = kind(x, y);
                    let ow = own(x, y);
                    let ot = keys.at(x as usize, y as usize) != 0 && !ow;
                    let th = theirs(x, y);
                    if k < 2 {
                        a += i32::from(ow);
                        b += i32::from(ot);
                        c += i32::from(th);
                        e += i32::from(ow && th);
                    }
                    let hid = whole(x, y) && !ow;
                    line.push(match (k, ow, ot, th) {
                        (3, _, _, _) => 'I',
                        (2, _, _, _) => 'T',
                        (_, _, true, _) if hid => '*',
                        (1, _, _, _) => '=',
                        (_, true, _, true) => '#',
                        (_, true, _, false) => 'o',
                        (_, _, true, true) => '%',
                        (_, _, true, false) => '-',
                        (_, _, _, true) => '+',
                        _ => '.',
                    });
                }
                println!("  {y:4}: {a:3} {b:3} {c:3} {e:3} | {line}");
            }
            // The photograph's luminance by tenths of 256, ring-free, beside
            // the figure drawn alone ('A') and where ours is another ('-').
            println!("  row: photograph's luminance | ours");
            for y in win.y0..win.y1 {
                let (mut l, mut m) = (String::new(), String::new());
                for x in win.x0..win.x1 {
                    let i = (y as usize * w + x as usize) * 4;
                    let lum = (299 * u32::from(photo.rgba[i])
                        + 587 * u32::from(photo.rgba[i + 1])
                        + 114 * u32::from(photo.rgba[i + 2]))
                        / 1000;
                    l.push(char::from(b'0' + (lum * 10 / 256) as u8));
                    let k = keys.at(x as usize, y as usize);
                    m.push(if whole(x, y) {
                        'A'
                    } else if k != 0 {
                        '-'
                    } else {
                        '.'
                    });
                }
                println!("  {y:4}: {l} | {m}");
            }
            // Seen to fail: our own mask shifted ten pixels right.
            let shifted = measure(&|x, y| own(x - 10, y) && theirs(x, y));
            println!("  own shifted 10 px, on theirs: {} of {}", shifted.0, mo.0);
            owed.push((o, mo, mt, mot.0, shifted.0, mw, mwt.0));
        }
        // Step 2: each figure's state as our draw takes it, at the
        // photograph's frame and at the packet's (3294), beside
        // `tree_points.py --guys`'s rows of the packet.
        // `(o, x, y, z, cur_anim, cur_time, angle)` of each `GuyData` on
        // run1559b's packet (`tree_points.py --guys`, item 4086).
        const PACKET_3294: [(i16, i32, i32, i32, i8, u32, u32); 5] = [
            (31, 4776, 31464, 534, 11, 23, 0xcfa9_0000),
            (20, 4536, 31224, 593, 0, 15, 0x9d08_0000),
            (32, 4584, 31320, 578, 0, 29, 0x5d77_0000),
            (35, 5208, 31272, 487, 9, 4, 0xd112_0000),
            (38, 4920, 31560, 506, 0, 25, 0x3781_0000),
        ];
        let state = |world: &mut World| -> Vec<(i16, i32, i32, i32, i8, u32, u32)> {
            let mut out = Vec::new();
            let (figures, _, list) = world.draw_list(
                &camera,
                &crate::pinned_screen(),
                (size.0 as i32, size.1 as i32),
            );
            for o in [31i16, 20, 32, 35, 38] {
                let p = figures
                    .iter()
                    .find(|p| (p.who, p.o) == (1, o))
                    .expect("placed");
                let facing = list.iter().find_map(|d| match d.pose {
                    render::draw::Pose::Figure { facing, .. } if (d.who, d.o) == (1, o) => {
                        Some(facing)
                    }
                    _ => None,
                });
                println!(
                    "ours 1/{o} at {}: x {} {} {} piece {} slot {} frame {} ms {} facing {:#x}",
                    world.runner.frame(),
                    p.x,
                    p.y,
                    p.z,
                    p.piece,
                    p.slot,
                    p.frame,
                    p.frame * 200 / 3,
                    facing.unwrap_or(0)
                );
                out.push((o, p.x, p.y, p.z, p.slot, p.frame, facing.unwrap_or(0)));
            }
            out
        };
        state(&mut world);
        world.run_to(3294);
        assert_eq!(
            state(&mut world),
            PACKET_3294,
            "ours is the packet's at 3294"
        );
        // The clear figure: ours and theirs alike, and not so shifted.
        let (_, mo, mt, both, shifted, _, _) = owed[3];
        assert!(
            (mo.3.0 - mt.3.0).abs() < 3.0 && (mo.3.1 - mt.3.1).abs() < 3.0,
            "the clear figure's centroids agree"
        );
        assert!(
            both * 10 >= mo.0 * 6,
            "the clear figure's own stands on theirs"
        );
        assert!(shifted * 2 < both, "shifted, it does not");
    }

    /// Item 4095: **the frame each rifleman shows**, a value before a draw.
    /// For each frame 3264 to 3294 of our walk, 1/20 and 1/32 and the
    /// controls 1/31, 1/35 and 1/38, each drawn alone from that frame's
    /// state (place, facing, slot, clock), held against theirs in the
    /// figure's window as `RUN1559B_3279_TRACED` scores it on 3279: the
    /// window and theirs (the ground-model mask, a neighbour's trace and
    /// the interface left out) are frozen at 3279, ours moves. Per frame:
    /// ours, both, the overlap `both / (ours + theirs − both)`, ours' box
    /// and its centroid's offset from theirs. Each figure's best frame by
    /// the overlap and its margin over 3279.
    #[test]
    fn the_frame_each_rifleman_shows() {
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
        // A frame and each figure's pixels on it.
        type FramePixels = (i64, Vec<Vec<(i32, i32)>>);
        const FIGURES: [i16; 5] = [20, 32, 31, 35, 38];
        const FIRST: i64 = 3264;
        const LAST: i64 = 3294;
        let camera = crate::pinned_camera();
        let size = (photo.width, photo.height);
        // Each figure drawn alone on a frame, as its pixels.
        // `at`: another frame's points to draw this frame's figures at.
        let alone = |world: &mut World,
                     at: Option<&[render::draw::Placed]>|
         -> (Vec<Vec<(i32, i32)>>, Vec<render::draw::Placed>) {
            let (mut figures, buildings, list) = world.draw_list(
                &camera,
                &crate::pinned_screen(),
                (size.0 as i32, size.1 as i32),
            );
            let own = figures.clone();
            if let Some(at) = at {
                for f in figures.iter_mut() {
                    if let Some(p) = at
                        .iter()
                        .find(|p| (p.who, p.o, p.figure) == (f.who, f.o, f.figure))
                    {
                        (f.x, f.y, f.z) = (p.x, p.y, p.z);
                    }
                }
            }
            let drawings = world.objects(&figures, &buildings, &list);
            let pixels = FIGURES
                .iter()
                .map(|&o| {
                    let objs: Vec<Object> = objects(&drawings)
                        .into_iter()
                        .filter(|b| {
                            let d = &drawings[b.key as usize - 1];
                            (d.who, d.o, d.building) == (1, o, false)
                        })
                        .collect();
                    let keys = rasterize_cut(&objs, &[], &[], &camera, size).keys;
                    let mut px = Vec::new();
                    for y in 0..size.1 {
                        for x in 0..size.0 {
                            if keys.at(x, y) != 0 {
                                px.push((x as i32, y as i32));
                            }
                        }
                    }
                    px
                })
                .collect::<Vec<_>>();
            (pixels, own)
        };
        world.run_to(FIRST);
        let mut frames: Vec<FramePixels> = Vec::new();
        // Frame n's slot, clock and facing at frame n − 1's points.
        let mut hybrid: Vec<FramePixels> = Vec::new();
        let mut prev: Option<Vec<render::draw::Placed>> = None;
        // At 3279, per figure: the window, and theirs and the pixels left
        // out (a neighbour's trace, the interface) in it.
        type Frozen = (Rect, Vec<bool>, Vec<bool>);
        let mut frozen: Vec<Frozen> = Vec::new();
        loop {
            let n = world.runner.frame();
            let (px, own) = alone(&mut world, None);
            frames.push((n, px));
            if let Some(p) = &prev {
                hybrid.push((n, alone(&mut world, Some(p)).0));
            }
            prev = Some(own);
            if n == PHOTO_FRAME {
                let rows = score_frame_traced(
                    &mut world,
                    &camera,
                    &photo,
                    &TRACED,
                    Neighbour::Neither,
                    Interface::Neither,
                );
                let (keys, who) = frame_keys(&mut world, &camera, size);
                let p = Photo {
                    width: photo.width,
                    height: photo.height,
                    rgba: &photo.rgba,
                };
                let traces: Vec<(u32, &[(i32, i32)])> = TRACED
                    .iter()
                    .map(|&(name, poly)| {
                        let k = who
                            .iter()
                            .position(|&(a, o, _, _)| format!("{a}/{o}") == name)
                            .map_or(u32::MAX, |i| i as u32 + 1);
                        (k, poly)
                    })
                    .collect();
                for o in FIGURES {
                    let (f, _) = rows
                        .iter()
                        .find(|(_, d)| (d.0, d.1, d.3) == (1, o, false))
                        .copied()
                        .expect("the figure is scored on 3279");
                    let win = f.window;
                    let r = READING.ring;
                    let outer = Rect::new(win.x0 - r, win.y0 - r, win.x1 + r, win.y1 + r);
                    let kind = |x: i32, y: i32| -> u8 {
                        if INTERFACE
                            .iter()
                            .any(|r| (r.x0..r.x1).contains(&x) && (r.y0..r.y1).contains(&y))
                        {
                            return 3;
                        }
                        let mut k = 0;
                        for t in &traces {
                            if render::silhouette::inside(t.1, x, y) {
                                if t.0 == f.key {
                                    return 1;
                                }
                                k = 2;
                            }
                        }
                        k
                    };
                    let g = render::silhouette::ground_where(
                        &keys,
                        &p,
                        win,
                        outer,
                        &READING,
                        |x, y| kind(x, y) == 0,
                    );
                    let (w, h) = (win.x1 - win.x0, win.y1 - win.y0);
                    let mut theirs = vec![false; (w * h) as usize];
                    let mut out = vec![false; (w * h) as usize];
                    for y in win.y0..win.y1 {
                        for x in win.x0..win.x1 {
                            let i = ((y - win.y0) * w + (x - win.x0)) as usize;
                            out[i] = kind(x, y) >= 2;
                            theirs[i] = kind(x, y) == 0 && !g.holds(&p, x, y);
                        }
                    }
                    frozen.push((win, theirs, out));
                }
            }
            if n >= LAST {
                break;
            }
            world.run_to(n + 1);
        }
        // One figure on one frame against its frozen window.
        struct Read {
            ours: i64,
            theirs: i64,
            both: i64,
            outside: i64,
            boxed: (i32, i32),
            off: (f64, f64),
        }
        let read = |k: usize, px: &[(i32, i32)]| -> Read {
            let (win, theirs, out) = &frozen[k];
            let w = win.x1 - win.x0;
            let at = |x: i32, y: i32| ((y - win.y0) * w + (x - win.x0)) as usize;
            let (mut tn, mut tx, mut ty) = (0i64, 0i64, 0i64);
            for y in win.y0..win.y1 {
                for x in win.x0..win.x1 {
                    if theirs[at(x, y)] {
                        tn += 1;
                        tx += i64::from(x);
                        ty += i64::from(y);
                    }
                }
            }
            let (mut on, mut ox, mut oy, mut both, mut outside) = (0i64, 0i64, 0i64, 0i64, 0i64);
            let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
            for &(x, y) in px {
                if !((win.x0..win.x1).contains(&x) && (win.y0..win.y1).contains(&y)) {
                    outside += 1;
                    continue;
                }
                if out[at(x, y)] {
                    continue;
                }
                on += 1;
                ox += i64::from(x);
                oy += i64::from(y);
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
                both += i64::from(theirs[at(x, y)]);
            }
            let c = |s: i64, n: i64| s as f64 / n.max(1) as f64;
            Read {
                ours: on,
                theirs: tn,
                both,
                outside,
                boxed: if on == 0 {
                    (0, 0)
                } else {
                    (x1 - x0 + 1, y1 - y0 + 1)
                },
                off: (c(ox, on) - c(tx, tn), c(oy, on) - c(ty, tn)),
            }
        };
        let overlap = |r: &Read| r.both as f64 / (r.ours + r.theirs - r.both).max(1) as f64;
        let mut best = Vec::new();
        for (k, &o) in FIGURES.iter().enumerate() {
            let (win, _, _) = frozen[k];
            println!(
                "1/{o} window ({}, {})..({}, {}): frame ours theirs both outside overlap box centroid-off",
                win.x0, win.y0, win.x1, win.y1
            );
            let mut table: Vec<(i64, f64)> = Vec::new();
            for (n, px) in &frames {
                let r = read(k, &px[k]);
                let v = overlap(&r);
                table.push((*n, v));
                println!(
                    "  {n} {} {} {} {} {v:.3} {}x{} ({:+.1}, {:+.1})",
                    r.ours, r.theirs, r.both, r.outside, r.boxed.0, r.boxed.1, r.off.0, r.off.1
                );
            }
            let at_photo = table.iter().find(|t| t.0 == PHOTO_FRAME).expect("3279").1;
            let top = table
                .iter()
                .copied()
                .fold((0, f64::MIN), |a, t| if t.1 > a.1 { t } else { a });
            let runner_up = table
                .iter()
                .filter(|t| t.0 != top.0)
                .map(|t| t.1)
                .fold(f64::MIN, f64::max);
            println!(
                "1/{o}: best {} at {:.3}; 3279 at {at_photo:.3}; margin {:+.3} ({})",
                top.0,
                top.1,
                if top.0 == PHOTO_FRAME {
                    top.1 - runner_up
                } else {
                    top.1 - at_photo
                },
                if top.0 == PHOTO_FRAME {
                    "over the runner-up"
                } else {
                    "over 3279"
                }
            );
            best.push((o, top.0, top.1, at_photo));
        }
        println!("best frames: {best:?}");
        // The control mover reads 3279 by a margin; 1/20 parts from it by
        // more than the standing control 1/31 moves.
        let get = |o: i16| *best.iter().find(|b| b.0 == o).expect("a figure");
        let (_, at35, _, _) = get(35);
        assert_eq!(
            at35, PHOTO_FRAME,
            "the control mover reads the photograph's frame"
        );
        let (_, at20, top20, photo20) = get(20);
        let (_, _, top31, photo31) = get(31);
        assert!(at20 > PHOTO_FRAME && top20 - photo20 > 0.1, "1/20 parts");
        assert!(top31 - photo31 < 0.01, "the standing control does not");
        // Each figure's own pixels in its window left out of the reading
        // (under a neighbour's trace or the interface), at 3279.
        for (k, &o) in FIGURES.iter().enumerate() {
            let (win, _, out) = &frozen[k];
            let w = win.x1 - win.x0;
            let px = &frames.iter().find(|f| f.0 == PHOTO_FRAME).expect("3279").1[k];
            let masked = px
                .iter()
                .filter(|&&(x, y)| {
                    (win.x0..win.x1).contains(&x)
                        && (win.y0..win.y1).contains(&y)
                        && out[((y - win.y0) * w + (x - win.x0)) as usize]
                })
                .count();
            println!("1/{o} at 3279: {} px alone, {masked} left out", px.len());
        }
        // 1/20 and 1/32 drawn together against theirs over both windows:
        // if the photograph's blob by 1/20 holds 1/32 too, the pair reads
        // 3279 where 1/20 alone does not.
        let held = |x: i32, y: i32, k: usize| -> Option<(bool, bool)> {
            let (win, theirs, out) = &frozen[k];
            ((win.x0..win.x1).contains(&x) && (win.y0..win.y1).contains(&y)).then(|| {
                let i = ((y - win.y0) * (win.x1 - win.x0) + (x - win.x0)) as usize;
                (theirs[i], out[i])
            })
        };
        let cell = |x: i32, y: i32| -> Option<bool> {
            let (a, b) = (held(x, y, 0), held(x, y, 1));
            if a.is_none() && b.is_none() {
                return None;
            }
            if a.is_some_and(|v| v.1) || b.is_some_and(|v| v.1) {
                return None;
            }
            Some(a.is_some_and(|v| v.0) || b.is_some_and(|v| v.0))
        };
        let (w0, w1) = (frozen[0].0, frozen[1].0);
        let u = Rect::new(
            w0.x0.min(w1.x0),
            w0.y0.min(w1.y0),
            w0.x1.max(w1.x1),
            w0.y1.max(w1.y1),
        );
        let tn = (u.y0..u.y1)
            .flat_map(|y| (u.x0..u.x1).map(move |x| (x, y)))
            .filter(|&(x, y)| cell(x, y) == Some(true))
            .count() as i64;
        println!("1/20 with 1/32 over both windows: frame ours theirs both overlap");
        let mut pair = Vec::new();
        for (n, px) in &frames {
            let mut set: std::collections::BTreeSet<(i32, i32)> = px[0].iter().copied().collect();
            set.extend(px[1].iter().copied());
            let (mut on, mut both) = (0i64, 0i64);
            for &(x, y) in &set {
                if let Some(t) = cell(x, y) {
                    on += 1;
                    both += i64::from(t);
                }
            }
            let v = both as f64 / (on + tn - both).max(1) as f64;
            println!("  {n} {on} {tn} {both} {v:.3}");
            pair.push((*n, v));
        }
        let top = pair
            .iter()
            .copied()
            .fold((0, f64::MIN), |a, t| if t.1 > a.1 { t } else { a });
        let at_photo = pair.iter().find(|t| t.0 == PHOTO_FRAME).expect("3279").1;
        println!(
            "the pair: best {} at {:.3}; 3279 at {at_photo:.3}; margin {:+.3}",
            top.0,
            top.1,
            top.1 - at_photo
        );
        // Frame n's slot and clock at n − 1's place, each figure.
        for (k, &o) in FIGURES.iter().enumerate() {
            println!("1/{o}, n's pose at n − 1's place: n ours both overlap box centroid-off");
            if o == 20 || o == 35 {
                // §5.3's point: 3280 at 3279's points beats 3279 whole.
                let h = &hybrid
                    .iter()
                    .find(|h| h.0 == PHOTO_FRAME + 1)
                    .expect("3280")
                    .1;
                let p = &frames.iter().find(|f| f.0 == PHOTO_FRAME).expect("3279").1;
                assert!(
                    overlap(&read(k, &h[k])) > overlap(&read(k, &p[k])) + 0.1,
                    "1/{o}: the point's draw reads nearer"
                );
            }
            for (n, px) in hybrid.iter().filter(|h| (3277..=3283).contains(&h.0)) {
                let r = read(k, &px[k]);
                println!(
                    "  {n} {} {} {:.3} {}x{} ({:+.1}, {:+.1})",
                    r.ours,
                    r.both,
                    overlap(&r),
                    r.boxed.0,
                    r.boxed.1,
                    r.off.0,
                    r.off.1
                );
            }
        }
    }

    /// Item 4087, step 1: a value before a cause. On the pixels where our
    /// tank 1/27 hides the rifleman 1/31 on frame 3279, the camera-space
    /// depth (`render_matrix`'s third column, less is nearer) each is drawn
    /// with there, each drawn alone, as a distribution; each one's point
    /// and its depth; and the hull's extent toward the camera. Then the same
    /// table on the first pair of the row's passing figures that overlap
    /// another object of ours, so the table is seen to read "nearer, and on
    /// top" where the photograph agrees.
    #[test]
    fn the_rifleman_and_the_hull_by_depth() {
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
        world.run_to(PHOTO_FRAME);
        let size = (photo.width, photo.height);
        let rows = score_frame_traced(
            &mut world,
            &camera,
            &photo,
            &TRACED,
            Neighbour::Neither,
            Interface::Neither,
        );
        let (keys, who) = frame_keys(&mut world, &camera, size);
        let (figures, buildings, list) = world.draw_list(
            &camera,
            &crate::pinned_screen(),
            (size.0 as i32, size.1 as i32),
        );
        let drawings = world.objects(&figures, &buildings, &list);
        let r = camera.render_matrix().0;
        let depth_of = |p: [f32; 3]| r[2] * p[0] + r[6] * p[1] + r[10] * p[2] + r[14];
        // A unit's meshes drawn alone: keys and depth.
        let alone = |id: (u8, i16, bool)| {
            let objs: Vec<Object> = objects(&drawings)
                .into_iter()
                .filter(|b| {
                    let d = &drawings[b.key as usize - 1];
                    (d.who, d.o, d.building) == id
                })
                .collect();
            rasterize_cut(&objs, &[], &[], &camera, size)
        };
        // Its vertices' depth extent.
        let extent = |id: (u8, i16, bool)| {
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
            for d in drawings.iter().filter(|d| (d.who, d.o, d.building) == id) {
                for t in &d.tris {
                    for v in t {
                        let z = depth_of(*v);
                        (lo, hi) = (lo.min(z), hi.max(z));
                    }
                }
            }
            (lo, hi)
        };
        let point = |who_: u8, o: i16| {
            figures
                .iter()
                .find(|p| (p.who, p.o) == (who_, o))
                .map(|p| (p.x, p.y, p.z, p.piece))
        };
        let quantiles = |mut v: Vec<f32>| -> String {
            if v.is_empty() {
                return "none".into();
            }
            v.sort_by(|a, b| a.total_cmp(b));
            let q = |f: f32| v[((v.len() - 1) as f32 * f) as usize];
            format!(
                "n {} min {:.1} p10 {:.1} p50 {:.1} p90 {:.1} max {:.1}",
                v.len(),
                q(0.0),
                q(0.1),
                q(0.5),
                q(0.9),
                q(1.0)
            )
        };
        let key_of = |k: u32| -> Option<(u8, i16, bool)> {
            (k != 0 && (k as usize) <= who.len()).then(|| {
                let e = who[k as usize - 1];
                (e.0, e.1, e.3)
            })
        };
        // The table for a figure and the unit it overlaps. Returns (pixels
        // both draw, of them the figure nearer, of them the frame's key the
        // figure's).
        let table = |fig: (u8, i16, bool), unit: (u8, i16, bool)| -> (u32, u32, u32) {
            let a = alone(fig);
            let b = alone(unit);
            for (name, id) in [("figure", fig), ("unit", unit)] {
                let (lo, hi) = extent(id);
                let pt = point(id.0, id.1);
                let pd = pt.map(|p| depth_of([p.0 as f32, p.1 as f32, p.2 as f32]));
                println!(
                    "  {name} {}/{}: point {pt:?} depth {pd:?}; vertices' depth {lo:.1}..{hi:.1} ({:.1} toward the camera from its point)",
                    id.0,
                    id.1,
                    pd.map_or(0.0, |d| d - lo)
                );
            }
            let (mut fd, mut ud, mut diff) = (Vec::new(), Vec::new(), Vec::new());
            let (mut n, mut nearer, mut on_top) = (0u32, 0u32, 0u32);
            let w = size.0;
            for i in 0..a.depth.len() {
                if a.keys.keys[i] == 0 || b.keys.keys[i] == 0 {
                    continue;
                }
                let (x, y) = (i % w, i / w);
                if INTERFACE.iter().any(|q| {
                    (q.x0..q.x1).contains(&(x as i32)) && (q.y0..q.y1).contains(&(y as i32))
                }) {
                    continue;
                }
                n += 1;
                fd.push(a.depth[i]);
                ud.push(b.depth[i]);
                diff.push(a.depth[i] - b.depth[i]);
                nearer += u32::from(a.depth[i] < b.depth[i]);
                on_top += u32::from(key_of(keys.at(x, y)) == Some(fig));
            }
            println!("  on the {n} pixels both draw (interface left out):");
            println!("    figure's depth: {}", quantiles(fd));
            println!("    unit's depth:   {}", quantiles(ud));
            println!("    figure − unit:  {}", quantiles(diff));
            println!("    figure nearer on {nearer}; the frame's key the figure's on {on_top}");
            (n, nearer, on_top)
        };
        println!("1/31 against the tank 1/27 at {}:", world.runner.frame());
        let (n, nearer, on_top) = table((1, 31, false), (1, 27, false));
        // The control: the row's passing figures, each against the first
        // other unit of ours its alone-draw shares 20 pixels with.
        let mut control = None;
        for (f, d) in rows
            .iter()
            .filter(|(f, d)| !d.3 && f.overlap() >= READING.threshold)
        {
            let fig = (d.0, d.1, false);
            if point(d.0, d.1).is_none() || control.is_some() {
                continue;
            }
            let a = alone(fig);
            let mut shared: std::collections::BTreeMap<(u8, i16, bool), u32> = Default::default();
            for i in 0..a.keys.keys.len() {
                if a.keys.keys[i] == 0 {
                    continue;
                }
                if let Some(k) = key_of(keys.keys[i]).filter(|k| *k != fig) {
                    *shared.entry(k).or_default() += 1;
                }
            }
            let pick = shared
                .iter()
                .find(|(id, c)| **c >= 20 && !id.2)
                .map(|(id, _)| *id);
            if let Some(u) = pick {
                println!(
                    "control: {}/{} (overlap {:.3}) against {}/{}:",
                    fig.0,
                    fig.1,
                    f.overlap(),
                    u.0,
                    u.1
                );
                control = Some((fig, u, table(fig, u)));
            }
        }
        let (cf, cu, (cn, cnear, ctop)) = control.expect("a passing figure overlaps another unit");
        println!(
            "control, the other way: {}/{} against {}/{}:",
            cu.0, cu.1, cf.0, cf.1
        );
        let (rn, rnear, rtop) = table(cu, cf);
        // The tank's place on the photograph: its alone-draw's box and
        // centroid beside its hand trace's (item 4079), on the frame.
        let tank = alone((1, 27, false));
        let trace = TRACED
            .iter()
            .find(|(name, _)| *name == "1/27")
            .map(|t| t.1)
            .expect("1/27 is traced");
        let measure = |m: &dyn Fn(usize, usize) -> bool| {
            let (mut n, mut sx, mut sy) = (0u64, 0u64, 0u64);
            let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
            for y in 0..size.1 {
                for x in 0..size.0 {
                    if m(x, y) {
                        n += 1;
                        sx += x as u64;
                        sy += y as u64;
                        (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
                    }
                }
            }
            let c = |v: u64| v as f64 / n.max(1) as f64;
            (n, (x0, y0, x1, y1), (c(sx), c(sy)))
        };
        println!(
            "the tank's place on 3279: ours drawn alone {:?}; its trace {:?}",
            measure(&|x, y| tank.keys.at(x, y) != 0),
            measure(&|x, y| render::silhouette::inside(trace, x as i32, y as i32))
        );
        // The frames about the photograph's: the tank's point and depth,
        // and the table's three counts, as values (the photograph is 3279's).
        let mut near = World::open(&install, &crate::world::gamelog_dir()).expect("opened once");
        for f in PHOTO_FRAME - 2..=PHOTO_FRAME + 3 {
            near.run_to(f);
            let (figs, blds, lst) = near.draw_list(
                &camera,
                &crate::pinned_screen(),
                (size.0 as i32, size.1 as i32),
            );
            let ds = near.objects(&figs, &blds, &lst);
            let pick = |id: (u8, i16)| -> Vec<Object> {
                objects(&ds)
                    .into_iter()
                    .filter(|b| {
                        let d = &ds[b.key as usize - 1];
                        (d.who, d.o, d.building) == (id.0, id.1, false)
                    })
                    .collect()
            };
            let a = rasterize_cut(&pick((1, 31)), &[], &[], &camera, size);
            let b = rasterize_cut(&pick((1, 27)), &[], &[], &camera, size);
            let (mut both, mut fnear) = (0, 0);
            for i in 0..a.depth.len() {
                if a.keys.keys[i] != 0 && b.keys.keys[i] != 0 {
                    both += 1;
                    fnear += u32::from(a.depth[i] < b.depth[i]);
                }
            }
            let t = figs
                .iter()
                .find(|p| (p.who, p.o) == (1, 27))
                .expect("the tank");
            let g = figs.iter().find(|p| (p.who, p.o) == (1, 31)).expect("1/31");
            println!(
                "frame {f}: tank {} {} {} depth {:.1}, 1/31 depth {:.1}; both {both}, 1/31 nearer {fnear}, hidden {}",
                t.x,
                t.y,
                t.z,
                depth_of([t.x as f32, t.y as f32, t.z as f32]),
                depth_of([g.x as f32, g.y as f32, g.z as f32]),
                both - fnear
            );
        }
        // At 3294 the tank's point and facing beside the packet's
        // `GuyData` (`tree_points.py --guys 196:1/27`, item 4087).
        world.run_to(3294);
        let (figs, _, lst) = world.draw_list(
            &camera,
            &crate::pinned_screen(),
            (size.0 as i32, size.1 as i32),
        );
        let t = figs
            .iter()
            .find(|p| (p.who, p.o) == (1, 27))
            .expect("the tank");
        let facing = lst.iter().find_map(|d| match d.pose {
            render::draw::Pose::Figure { facing, .. } if (d.who, d.o) == (1, 27) => Some(facing),
            _ => None,
        });
        println!(
            "ours 1/27 at 3294: {} {} {} piece {} slot {} frame {} facing {facing:#x?}",
            t.x, t.y, t.z, t.piece, t.slot, t.frame
        );
        assert_eq!(
            (t.x, t.y, t.z, t.piece, facing),
            (4551, 31679, 563, 196, Some(0x9605_0000)),
            "our tank is the packet's at 3294"
        );
        // The draw is a depth test: on top never where farther, both sides.
        assert!(on_top <= nearer, "1/31 is on top only where nearer");
        assert_eq!((cnear, ctop), (0, 0), "the control is farther and below");
        assert_eq!(
            (rnear, rtop),
            (rn, rn),
            "the control's other is nearer and on top"
        );
        assert!(n >= 100 && cn >= 20);
    }

    /// **Item 4089, step 1: the tanks' slope normals, ours against the
    /// original's** on run1559b's packet (frame 3294). Per tank: our point
    /// beside the packet's; the three heights `find_slope_normal` reads on
    /// our terrain beside the packet's (`tree_points.py --slope`); our
    /// world normal's bits beside the original's on the packet; and the
    /// guy's eased `last_norm` replayed along our path from 3200 at `k`
    /// draws a frame, against the packet's `last_norm`, the angle between.
    #[test]
    fn the_tanks_slope_normals_against_the_packet() {
        let install = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let mut world = match World::open(&install, &crate::world::gamelog_dir()) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return;
            }
        };
        // `tree_points.py --guys 196:1/o` and `--slope x,y` on the packet:
        // o, point, the three heights, the world normal's bits, `last_norm`.
        type Row = (i16, [i32; 3], [i32; 3], [u32; 3], [f32; 3]);
        const PACKET: [Row; 4] = [
            (
                27,
                [4551, 31679, 563],
                [563, 559, 563],
                [0xbe48_d2ab, 0, 0xbf7b_0756],
                [0.183_931_93, 0.056_763_086, 0.980_415_2],
            ),
            (
                15,
                [5880, 30984, 423],
                [423, 420, 420],
                [0xbe16_41a6, 0xbe16_41a6, 0xbf7a_6d69],
                [0.170_600_97, 0.108_512_71, 0.976_893_8],
            ),
            (
                16,
                [6358, 30770, 370],
                [370, 366, 368],
                [0xbe47_dd45, 0xbdc7_dd45, 0xbf79_d496],
                [0.128_006_77, 0.062_975_99, 0.987_201_9],
            ),
            (
                28,
                [6170, 32078, 243],
                [243, 239, 242],
                [0xbe48_94fd, 0xbd48_94fd, 0xbf7a_ba3d],
                [0.102_563_34, 0.014_875_433, 0.991_204_6],
            ),
        ];
        let deg = |a: [f32; 3], b: [f32; 3]| {
            let d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
            let l = |v: [f32; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            (d / (l(a) * l(b))).clamp(-1.0, 1.0).acos().to_degrees()
        };
        let first = 3200;
        let mut paths: Vec<Vec<[i32; 3]>> = vec![Vec::new(); PACKET.len()];
        let mut targets: Vec<Vec<[f32; 3]>> = vec![Vec::new(); PACKET.len()];
        let camera = crate::pinned_camera();
        for f in first..=3294 {
            world.run_to(f);
            let (figs, _, _) = world.draw_list(&camera, &crate::pinned_screen(), (1024, 768));
            for (i, row) in PACKET.iter().enumerate() {
                let p = figs
                    .iter()
                    .find(|p| (p.who, p.o) == (1, row.0))
                    .map(|p| [p.x, p.y, p.z])
                    .unwrap_or([i32::MIN; 3]);
                paths[i].push(p);
                let (_, n) = world.slope_normal(p[0], p[1]);
                targets[i].push(render::slope::target(n));
            }
        }
        let mut agree = 0;
        for (i, (o, pt, z, bits, last)) in PACKET.iter().enumerate() {
            let ours = *paths[i].last().unwrap();
            let (oz, on) = world.slope_normal(ours[0], ours[1]);
            let packet_n = bits.map(f32::from_bits);
            let moved: Vec<i64> = paths[i]
                .windows(2)
                .enumerate()
                .filter(|(_, w)| w[0] != w[1])
                .map(|(k, _)| first + 1 + k as i64)
                .collect();
            let last_move = moved.last().copied();
            let target = render::slope::target(packet_n);
            println!(
                "1/{o}: point ours {ours:?} packet {pt:?}; heights ours {oz:?} packet {z:?}; \
                 normal ours {:?} packet {:?} ({:.4}°); target {target:?} tilt {:.2}°; \
                 last_norm {last:?} tilt {:.2}°, {:.2}° from the target; moved on {} frames of {}..=3294, last {last_move:?}",
                on.map(f32::to_bits),
                bits,
                deg(on, packet_n),
                deg(target, [0.0, 0.0, 1.0]),
                deg(*last, [0.0, 0.0, 1.0]),
                deg(*last, target),
                moved.len(),
                first + 1,
            );
            if on.map(f32::to_bits) == *bits && oz == *z && ours == *pt {
                agree += 1;
            }
            // The ease replayed at k draws a frame, each on the frame's own
            // point; 94 frames back the start's weight is 0.75^94k.
            for k in 1..=8 {
                let mut e = targets[i][0];
                for t in &targets[i] {
                    for _ in 0..k {
                        render::slope::ease(&mut e, *t);
                    }
                }
                println!(
                    "  k {k}: eased {e:?}, {:.3}° from the packet's last_norm",
                    deg(e, *last)
                );
            }
            // One draw every m frames, at each phase: the nearest phase.
            for m in 2..=16usize {
                let best = (0..m)
                    .map(|ph| {
                        let mut e = targets[i][0];
                        for (j, t) in targets[i].iter().enumerate() {
                            if j % m == ph {
                                render::slope::ease(&mut e, *t);
                            }
                        }
                        (deg(e, *last), ph)
                    })
                    .min_by(|a, b| a.0.total_cmp(&b.0))
                    .unwrap();
                println!("  every {m}: nearest {:.3}° at phase {}", best.0, best.1);
            }
        }
        // One draw clock for all four: every m frames at one phase, each
        // tank's angle to its packet `last_norm`, the worst of the four.
        for m in 1..=16usize {
            let best = (0..m)
                .map(|ph| {
                    let worst = PACKET
                        .iter()
                        .enumerate()
                        .map(|(i, row)| {
                            let mut e = targets[i][0];
                            for (j, t) in targets[i].iter().enumerate() {
                                if j % m == ph {
                                    render::slope::ease(&mut e, *t);
                                }
                            }
                            deg(e, row.4)
                        })
                        .fold(0.0f32, f32::max);
                    (worst, ph)
                })
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .unwrap();
            println!(
                "one clock, every {m}: worst of four {:.3}° at phase {} (frame {} mod {m})",
                best.0,
                best.1,
                first + best.1 as i64
            );
        }
        // A check that can fail: one height a unit off moves the normal.
        let n = render::slope::normal_of(4551, 31679, [563, 560, 563]);
        assert_ne!(n.map(f32::to_bits), PACKET[0].3, "a perturbed height parts");
        println!("tanks whose point, heights and world normal are the packet's: {agree} of 4");
        assert_eq!(
            agree, 4,
            "our slope normal is the original's at the four tanks"
        );
    }

    /// **Item 4102, step 1: the masked texels by value, before a rule.**
    /// On the colour row's draw, for each figure's lit pixels both
    /// footprints hold whose texel's alpha is under 255: the texel, its
    /// alpha, the owner's unit colour as run1559b's packet holds it
    /// (`team_colors.list[leader +0x6928].unit`, `TeamColor@0xe39ce0`,
    /// stride 0x64; 0 red, 1 blue), and the photograph's pixel; by alpha
    /// band, beside what each candidate blend predicts and its mean
    /// chromaticity distance to the photograph.
    #[test]
    fn the_masked_texels_against_the_owner_s_colour() {
        use render::silhouette::{HUE, chromaticity, traced_theirs};
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
        // run1559b's packet, frame 3294: `TeamColor::unit` of list 0, 1.
        let owner = |who: u8| -> [f32; 3] {
            match who {
                0 => [255.0, 0.0, 0.0],
                1 => [34.0, 82.0, 255.0],
                _ => [255.0, 0.0, 255.0],
            }
        };
        let camera = crate::pinned_camera();
        world.run_to(PHOTO_FRAME);
        world.blend = crate::world::Blend::Last;
        world.run_to(PHOTO_FRAME + 1);
        let size = (photo.width, photo.height);
        let scene = frame_scene(&mut world, &camera, size);
        let p = Photo {
            width: photo.width,
            height: photo.height,
            rgba: &photo.rgba,
        };
        let traces: Vec<Trace<'_>> = TRACED
            .iter()
            .map(|&(name, poly)| Trace {
                key: scene
                    .who
                    .iter()
                    .position(|&(who, o, _, _)| format!("{who}/{o}") == name)
                    .map_or(u32::MAX, |i| i as u32 + 1),
                poly,
            })
            .collect();
        let rows = footprints_traced(
            &scene.keys,
            &p,
            &INTERFACE,
            &READING,
            &traces,
            Neighbour::Neither,
            Interface::Neither,
        );
        type Blend = fn([f32; 3], f32, [f32; 3]) -> [f32; 3];
        let blends: [(&str, Blend); 7] = [
            ("texel", |t, _, _| t),
            ("owner", |_, _, c| c),
            ("t*c", |t, _, c| [0, 1, 2].map(|k| t[k] * c[k] / 255.0)),
            ("lerp(c,t,a)", |t, a, c| {
                [0, 1, 2].map(|k| t[k] * a + c[k] * (1.0 - a))
            }),
            ("lerp(t,c,a)", |t, a, c| {
                [0, 1, 2].map(|k| c[k] * a + t[k] * (1.0 - a))
            }),
            ("lerp(t*c,t,a)", |t, a, c| {
                [0, 1, 2].map(|k| t[k] * a + t[k] * c[k] / 255.0 * (1.0 - a))
            }),
            ("lerp(t,t*c,a)", |t, a, c| {
                [0, 1, 2].map(|k| t[k] * c[k] / 255.0 * a + t[k] * (1.0 - a))
            }),
        ];
        let dist = |a: [f32; 3], b: [f32; 3]| {
            let (x, y) = (chromaticity(a), chromaticity(b));
            ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2)).sqrt()
        };
        // [owner][band] -> (n, sum texel, sum alpha, sum photo, per-blend (sum d, agree))
        #[derive(Clone, Copy, Default)]
        struct Acc {
            n: u32,
            t: [f32; 3],
            a: f32,
            ph: [f32; 3],
            d: [f32; 7],
            agree: [u32; 7],
        }
        let mut acc = [[Acc::default(); 4]; 2];
        for f in rows {
            let who = scene.who[f.key as usize - 1];
            if who.0 == TREE_WHO || who.3 {
                continue;
            }
            let theirs = traced_theirs(
                &scene.keys,
                &p,
                &INTERFACE,
                &READING,
                &traces,
                Neighbour::Neither,
                Interface::Neither,
                f.key,
                f.window,
            );
            let ours = painted(
                &mut world,
                &scene,
                &camera,
                size,
                f.key,
                f.window,
                Colouring::Texture,
            );
            let ww = (f.window.x1 - f.window.x0) as usize;
            let mut obj = [Acc::default(); 1];
            for y in f.window.y0..f.window.y1 {
                for x in f.window.x0..f.window.x1 {
                    let j = (y - f.window.y0) as usize * ww + (x - f.window.x0) as usize;
                    if scene.keys.at(x as usize, y as usize) != f.key || theirs[j] != Some(true) {
                        continue;
                    }
                    if scene.behind[y as usize * photo.width + x as usize] {
                        continue;
                    }
                    let Some(t) = ours[j] else { continue };
                    if t.alpha >= HUE.mask_alpha {
                        continue;
                    }
                    let i = (y as usize * photo.width + x as usize) * 4;
                    let ph = [0, 1, 2].map(|k| f32::from(photo.rgba[i + k]));
                    let sum = |q: [f32; 3]| q[0] + q[1] + q[2];
                    if sum(t.rgb) < HUE.dark || sum(ph) < HUE.dark {
                        continue;
                    }
                    let a = t.alpha / 255.0;
                    let band = ((t.alpha as usize) / 64).min(3);
                    let c = owner(who.0);
                    for target in [&mut acc[usize::from(who.0 != 0)][band], &mut obj[0]] {
                        target.n += 1;
                        target.a += t.alpha;
                        for (k, v) in ph.iter().enumerate() {
                            target.t[k] += t.rgb[k];
                            target.ph[k] += v;
                        }
                        for (b, (_, blend)) in blends.iter().enumerate() {
                            let d = dist(blend(t.rgb, a, c), ph);
                            target.d[b] += d;
                            target.agree[b] += u32::from(d <= HUE.distance);
                        }
                    }
                }
            }
            let o = obj[0];
            if o.n > 0 {
                let n = o.n as f32;
                print!(
                    "{}/{} piece {} masked {} alpha {:.0} texel {:.0?} photo {:.0?} |",
                    who.0,
                    who.1,
                    who.2,
                    o.n,
                    o.a / n,
                    o.t.map(|v| v / n),
                    o.ph.map(|v| v / n)
                );
                for (b, (name, _)) in blends.iter().enumerate() {
                    print!(" {name} {:.3} {}", o.d[b] / n, o.agree[b]);
                }
                println!();
            }
        }
        for (owner_i, bands) in acc.iter().enumerate() {
            for (band, b) in bands.iter().enumerate() {
                if b.n == 0 {
                    continue;
                }
                let n = b.n as f32;
                print!(
                    "player {} alpha {}..{} n {} alpha {:.0} texel {:.0?} photo {:.0?} |",
                    owner_i,
                    band * 64,
                    band * 64 + 63,
                    b.n,
                    b.a / n,
                    b.t.map(|v| v / n),
                    b.ph.map(|v| v / n)
                );
                for (k, (name, _)) in blends.iter().enumerate() {
                    print!(" {name} {:.3} {}", b.d[k] / n, b.agree[k]);
                }
                println!();
            }
        }
    }

    /// **Item 4103, step 1: the silhouette's pixels by value, before a
    /// rule.** On the colour row's draw, for each figure's lit pixels both
    /// footprints hold that the silhouette's pass owns (`Scene::behind`):
    /// the photograph's colour (mean, spread), the nearer tree's or
    /// sprite's own pixel there, our team-coloured texel, and every colour
    /// of the owner's `TeamColor` record as run1559b's packet holds it
    /// (`team_colors.list[leader +0x6928]`, `0xe39ce0`, stride 0x64), each
    /// alone and each at `render_states[1]`'s alpha 128 over the nearer
    /// pixel; with the least-squares share of `unit` over the nearer pixel.
    #[test]
    fn the_silhouette_s_pixels_against_the_owner_s_record() {
        use render::silhouette::{HUE, chromaticity, traced_theirs};
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
        // run1559b's packet, frame 3294: `TeamColor` list 0 and 1, by field.
        const FIELDS: [&str; 8] = [
            "unit",
            "build",
            "text",
            "neon",
            "border",
            "iface",
            "unit_tint",
            "build_tint",
        ];
        const RECORD: [[[f32; 3]; 8]; 2] = [
            [
                [255.0, 0.0, 0.0],
                [102.0, 0.0, 0.0],
                [255.0, 100.0, 100.0],
                [255.0, 0.0, 0.0],
                [255.0, 0.0, 0.0],
                [255.0, 0.0, 0.0],
                [127.0, 47.0, 47.0],
                [127.0, 95.0, 95.0],
            ],
            [
                [34.0, 82.0, 255.0],
                [26.0, 57.0, 172.0],
                [70.0, 100.0, 255.0],
                [70.0, 100.0, 255.0],
                [0.0, 0.0, 255.0],
                [0.0, 0.0, 255.0],
                [47.0, 47.0, 159.0],
                [95.0, 95.0, 127.0],
            ],
        ];
        let share = 128.0 / 255.0;
        let camera = crate::pinned_camera();
        world.run_to(PHOTO_FRAME);
        world.blend = crate::world::Blend::Last;
        world.run_to(PHOTO_FRAME + 1);
        let size = (photo.width, photo.height);
        let scene = frame_scene(&mut world, &camera, size);
        let first_tree = (scene.drawings.len() + scene.sprites.len()) as u32 + 1;
        let p = Photo {
            width: photo.width,
            height: photo.height,
            rgba: &photo.rgba,
        };
        let traces: Vec<Trace<'_>> = TRACED
            .iter()
            .map(|&(name, poly)| Trace {
                key: scene
                    .who
                    .iter()
                    .position(|&(who, o, _, _)| format!("{who}/{o}") == name)
                    .map_or(u32::MAX, |i| i as u32 + 1),
                poly,
            })
            .collect();
        let rows = footprints_traced(
            &scene.keys,
            &p,
            &INTERFACE,
            &READING,
            &traces,
            Neighbour::Neither,
            Interface::Neither,
        );
        let dist = |a: [f32; 3], b: [f32; 3]| {
            let (x, y) = (chromaticity(a), chromaticity(b));
            ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2)).sqrt()
        };
        let sum = |q: [f32; 3]| q[0] + q[1] + q[2];
        // Candidates: the texel, the nearer pixel, each field alone, each
        // field at the share over the nearer pixel.
        let names: Vec<String> = ["texel".to_string(), "nearer".to_string()]
            .into_iter()
            .chain(FIELDS.iter().map(|f| f.to_string()))
            .chain(FIELDS.iter().map(|f| format!("{f}@128")))
            .collect();
        #[derive(Clone, Default)]
        struct Acc {
            n: u32,
            ph: [f32; 3],
            ph2: [f32; 3],
            u: [f32; 3],
            t: [f32; 3],
            s_num: f32,
            s_den: f32,
            d: Vec<f32>,
            agree: Vec<u32>,
        }
        let fresh = || Acc {
            d: vec![0.0; names.len()],
            agree: vec![0; names.len()],
            ..Acc::default()
        };
        let mut by_owner = [fresh(), fresh()];
        let report = |label: &str, a: &Acc| {
            let n = a.n as f32;
            let mean = a.ph.map(|v| v / n);
            let spread: Vec<f32> = (0..3)
                .map(|k| (a.ph2[k] / n - mean[k] * mean[k]).max(0.0).sqrt())
                .collect();
            print!(
                "{label} n {} photo {:.0?} spread {:.0?} nearer {:.0?} texel {:.0?} share {:.3} |",
                a.n,
                mean,
                spread,
                a.u.map(|v| v / n),
                a.t.map(|v| v / n),
                a.s_num / a.s_den.max(1e-6)
            );
            for (b, name) in names.iter().enumerate() {
                print!(" {name} {:.3} {}", a.d[b] / n, a.agree[b]);
            }
            println!();
        };
        for f in rows {
            let who = scene.who[f.key as usize - 1];
            if who.0 == TREE_WHO || who.3 || who.0 > 1 {
                continue;
            }
            let theirs = traced_theirs(
                &scene.keys,
                &p,
                &INTERFACE,
                &READING,
                &traces,
                Neighbour::Neither,
                Interface::Neither,
                f.key,
                f.window,
            );
            let ours = painted(
                &mut world,
                &scene,
                &camera,
                size,
                f.key,
                f.window,
                Colouring::Team,
            );
            let ww = (f.window.x1 - f.window.x0) as usize;
            let mut cache: std::collections::HashMap<u32, Vec<Option<render::silhouette::Texel>>> =
                std::collections::HashMap::new();
            let mut obj = fresh();
            let mut kinds = (0u32, 0u32);
            for y in f.window.y0..f.window.y1 {
                for x in f.window.x0..f.window.x1 {
                    let j = (y - f.window.y0) as usize * ww + (x - f.window.x0) as usize;
                    let at = y as usize * photo.width + x as usize;
                    if scene.keys.at(x as usize, y as usize) != f.key
                        || theirs[j] != Some(true)
                        || !scene.behind[at]
                    {
                        continue;
                    }
                    let Some(t) = ours[j] else { continue };
                    let k = scene.nearer[at];
                    let near = cache.entry(k).or_insert_with(|| {
                        paint_nearer(&mut world, &scene, &camera, size, k, f.window)
                    });
                    let Some(u) = near[j] else { continue };
                    let i = at * 4;
                    let ph = [0, 1, 2].map(|c| f32::from(photo.rgba[i + c]));
                    if sum(t.rgb) < HUE.dark || sum(ph) < HUE.dark {
                        continue;
                    }
                    if k >= first_tree {
                        kinds.0 += 1;
                    } else {
                        kinds.1 += 1;
                    }
                    let rec = RECORD[usize::from(who.0)];
                    let mut preds = vec![t.rgb, u.rgb];
                    preds.extend(rec);
                    preds.extend(
                        rec.iter()
                            .map(|c| [0, 1, 2].map(|q| c[q] * share + u.rgb[q] * (1.0 - share))),
                    );
                    let c = rec[0];
                    let cu = [0, 1, 2].map(|q| c[q] - u.rgb[q]);
                    let pu = [0, 1, 2].map(|q| ph[q] - u.rgb[q]);
                    for target in [&mut by_owner[usize::from(who.0)], &mut obj] {
                        target.n += 1;
                        for q in 0..3 {
                            target.ph[q] += ph[q];
                            target.ph2[q] += ph[q] * ph[q];
                            target.u[q] += u.rgb[q];
                            target.t[q] += t.rgb[q];
                            target.s_num += pu[q] * cu[q];
                            target.s_den += cu[q] * cu[q];
                        }
                        for (b, pred) in preds.iter().enumerate() {
                            let d = dist(*pred, ph);
                            target.d[b] += d;
                            target.agree[b] += u32::from(d <= HUE.distance);
                        }
                    }
                }
            }
            if obj.n > 0 {
                report(
                    &format!(
                        "{}/{} piece {} (trees {}, sprites {})",
                        who.0, who.1, who.2, kinds.0, kinds.1
                    ),
                    &obj,
                );
            }
        }
        for (o, a) in by_owner.iter().enumerate() {
            if a.n > 0 {
                report(&format!("player {o}"), a);
            }
        }
    }

    /// **Item 4106, step 2: the light on a figure's texel, as a value.** Per
    /// figure of the colour row, over its lit pixels that both footprints
    /// hold, not behind anything (`Scene::behind`), the texel's alpha at
    /// least 1 of 255 (not 255: item 4107 split it, see
    /// [`the_peasants_alpha`]) and neither side dark: the photograph's channel means over our
    /// texel's, each per-pixel ratio's spread, the same split by the face's
    /// `N·L` against run1559b's packet light (`lights[0].cur_dir`
    /// (−0.707, 0, −0.707), `L = −cur_dir`), and each candidate's
    /// chromaticity distance and agreeing pixels. The candidates' constants
    /// are the packet's (`unit_material`'s ambient (0.600, 0.716, 0.745)) and
    /// the lit branch's power 1.45 (the Extended Edition's `ps_global.hlsl`,
    /// read in the install); none is fitted. Then, for the three peasants
    /// under the pine, the pine's own pixels in their windows: the
    /// photograph over our leaf texel.
    #[test]
    #[allow(clippy::needless_range_loop)]
    fn the_light_on_the_figures_texel() {
        use render::silhouette::{HUE, chromaticity, traced_theirs};
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
        const AMBIENT: [f32; 3] = [0.600, 0.716, 0.745];
        const LIT_POWER: f32 = 1.45;
        let l = {
            let v = [0.707_f32, 0.0, 0.707];
            let n = (v[0] * v[0] + v[2] * v[2]).sqrt();
            [v[0] / n, 0.0, v[2] / n]
        };
        let camera = crate::pinned_camera();
        world.run_to(PHOTO_FRAME);
        world.blend = crate::world::Blend::Last;
        world.run_to(PHOTO_FRAME + 1);
        let size = (photo.width, photo.height);
        let scene = frame_scene(&mut world, &camera, size);
        let first_tree = (scene.drawings.len() + scene.sprites.len()) as u32 + 1;
        let p = Photo {
            width: photo.width,
            height: photo.height,
            rgba: &photo.rgba,
        };
        let traces: Vec<Trace<'_>> = TRACED
            .iter()
            .map(|&(name, poly)| Trace {
                key: scene
                    .who
                    .iter()
                    .position(|&(who, o, _, _)| format!("{who}/{o}") == name)
                    .map_or(u32::MAX, |i| i as u32 + 1),
                poly,
            })
            .collect();
        let rows = footprints_traced(
            &scene.keys,
            &p,
            &INTERFACE,
            &READING,
            &traces,
            Neighbour::Neither,
            Interface::Neither,
        );
        let dist = |a: [f32; 3], b: [f32; 3]| {
            let (x, y) = (chromaticity(a), chromaticity(b));
            ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2)).sqrt()
        };
        let sum = |q: [f32; 3]| q[0] + q[1] + q[2];
        let names = ["texel", "texel*A", "texel^1.45", "texel^1.45*A"];
        let preds = |t: [f32; 3]| {
            let pw = t.map(|v| 255.0 * (v / 255.0).powf(LIT_POWER));
            [
                t,
                [0, 1, 2].map(|q| t[q] * AMBIENT[q]),
                pw,
                [0, 1, 2].map(|q| pw[q] * AMBIENT[q]),
            ]
        };
        #[derive(Clone, Default)]
        struct Acc {
            n: u32,
            ph: [f64; 3],
            t: [f64; 3],
            r: [f64; 3],
            r2: [f64; 3],
            dc: [f64; 2],
            d: [f64; 4],
            agree: [u32; 4],
        }
        fn add(a: &mut Acc, ph: [f32; 3], t: [f32; 3], ds: [f32; 4], agree: f32, dc: [f32; 2]) {
            a.n += 1;
            for q in 0..3 {
                a.ph[q] += f64::from(ph[q]);
                a.t[q] += f64::from(t[q]);
                let r = f64::from(ph[q]) / f64::from(t[q].max(1.0));
                a.r[q] += r;
                a.r2[q] += r * r;
            }
            for b in 0..4 {
                a.d[b] += f64::from(ds[b]);
                a.agree[b] += u32::from(ds[b] <= agree);
            }
            a.dc[0] += f64::from(dc[0]);
            a.dc[1] += f64::from(dc[1]);
        }
        let line = |label: &str, a: &Acc| {
            if a.n == 0 {
                return;
            }
            let n = a.n as f64;
            let ph = a.ph.map(|v| v / n);
            let t = a.t.map(|v| v / n);
            let ratio = [0, 1, 2].map(|q| ph[q] / t[q].max(1.0));
            let spread: Vec<f64> = (0..3)
                .map(|q| (a.r2[q] / n - (a.r[q] / n).powi(2)).max(0.0).sqrt())
                .collect();
            print!(
                "{label} n {} photo {:.0?} texel {:.0?} ratio {:.3?} ratio/g {:.3?} spread {:.3?} dchroma ({:+.3}, {:+.3}) |",
                a.n,
                ph,
                t,
                ratio,
                [ratio[0] / ratio[1], 1.0, ratio[2] / ratio[1]],
                spread,
                a.dc[0] / n,
                a.dc[1] / n
            );
            for b in 0..4 {
                print!(" {} {:.3} {}", names[b], a.d[b] / n, a.agree[b]);
            }
            println!();
        };
        let bin = |nl: f32| -> usize {
            if nl < -0.25 {
                0
            } else if nl < 0.25 {
                1
            } else {
                2
            }
        };
        const BINS: [&str; 3] = ["N·L<-.25", "|N·L|<.25", "N·L>.25"];
        let mut owners = [Acc::default(), Acc::default()];
        let mut owners_bin: [[Acc; 3]; 2] = Default::default();
        let mut leaves = Acc::default();
        let pine_figures = ["0/2", "0/8", "0/10"];
        const SHIFTS: [(i64, i64); 9] = [
            (0, 0),
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ];
        let mut shift_agree = [[0u32; 9]; 2];
        let mut behind_n = 0u32;
        let mut behind_agree = (0u32, 0u32);
        for f in rows {
            let who = scene.who[f.key as usize - 1];
            if who.0 == TREE_WHO || who.3 || who.0 > 1 {
                continue;
            }
            let Some(d) = scene.drawings.get(f.key as usize - 1) else {
                continue;
            };
            let theirs = traced_theirs(
                &scene.keys,
                &p,
                &INTERFACE,
                &READING,
                &traces,
                Neighbour::Neither,
                Interface::Neither,
                f.key,
                f.window,
            );
            let ours = painted(
                &mut world,
                &scene,
                &camera,
                size,
                f.key,
                f.window,
                Colouring::Texture,
            );
            let ww = (f.window.x1 - f.window.x0) as usize;
            let mut obj = Acc::default();
            let mut by_bin: [Acc; 3] = Default::default();
            let name = format!("{}/{}", who.0, who.1);
            let mut tree_cache: std::collections::HashMap<
                u32,
                Vec<Option<render::silhouette::Texel>>,
            > = std::collections::HashMap::new();
            let pine = pine_figures
                .contains(&name.as_str())
                .then(|| {
                    (f.window.y0..f.window.y1)
                        .flat_map(|y| (f.window.x0..f.window.x1).map(move |x| (x, y)))
                        .map(|(x, y)| scene.nearer[y as usize * photo.width + x as usize])
                        .find(|&k| k >= first_tree)
                })
                .flatten();
            for y in f.window.y0..f.window.y1 {
                for x in f.window.x0..f.window.x1 {
                    let j = (y - f.window.y0) as usize * ww + (x - f.window.x0) as usize;
                    let at = y as usize * photo.width + x as usize;
                    let i = at * 4;
                    let ph = [0, 1, 2].map(|c| f32::from(photo.rgba[i + c]));
                    let key_here = scene.keys.at(x as usize, y as usize);
                    // The pine's own pixels by the three peasants: where it
                    // paints and no object holds the key.
                    if let Some(pine) = pine
                        && key_here == 0
                    {
                        let leaf = tree_cache.entry(pine).or_insert_with(|| {
                            paint_nearer(&mut world, &scene, &camera, size, pine, f.window)
                        });
                        if let Some(t) = leaf[j]
                            && sum(t.rgb) >= HUE.dark
                            && sum(ph) >= HUE.dark
                        {
                            let (a, b) = (chromaticity(ph), chromaticity(t.rgb));
                            let ds = preds(t.rgb).map(|q| dist(q, ph));
                            add(
                                &mut leaves,
                                ph,
                                t.rgb,
                                ds,
                                HUE.distance,
                                [a[0] - b[0], a[1] - b[1]],
                            );
                        }
                    }
                    // The peasants' behind pixels under 4103's blend, over
                    // the leaf texel and over its lit-branch power.
                    if let Some(pine) = pine
                        && key_here == f.key
                        && theirs[j] == Some(true)
                        && scene.behind[at]
                        && scene.nearer[at] == pine
                        && sum(ph) >= HUE.dark
                    {
                        let leaf = tree_cache.entry(pine).or_insert_with(|| {
                            paint_nearer(&mut world, &scene, &camera, size, pine, f.window)
                        });
                        if let Some(u) = leaf[j] {
                            let c = render::silhouette::team_unit(who.0).map(f32::from);
                            let a = f32::from(render::silhouette::SILHOUETTE_ALPHA) / 255.0;
                            let pw = u.rgb.map(|v| 255.0 * (v / 255.0).powf(LIT_POWER));
                            let over = |n: [f32; 3]| [0, 1, 2].map(|q| c[q] * a + n[q] * (1.0 - a));
                            behind_n += 1;
                            behind_agree.0 += u32::from(dist(over(u.rgb), ph) <= HUE.distance);
                            behind_agree.1 += u32::from(dist(over(pw), ph) <= HUE.distance);
                        }
                    }
                    if key_here != f.key || theirs[j] != Some(true) || scene.behind[at] {
                        continue;
                    }
                    let Some(t) = ours[j] else { continue };
                    if t.alpha < 1.0 || sum(t.rgb) < HUE.dark || sum(ph) < HUE.dark {
                        continue;
                    }
                    let nl = d.tris.get(t.tri as usize).map_or(0.0, |tri| {
                        let (a, b, c) = (tri[0], tri[1], tri[2]);
                        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                        let nrm = [
                            u[1] * v[2] - u[2] * v[1],
                            u[2] * v[0] - u[0] * v[2],
                            u[0] * v[1] - u[1] * v[0],
                        ];
                        let len = (nrm[0] * nrm[0] + nrm[1] * nrm[1] + nrm[2] * nrm[2]).sqrt();
                        if len == 0.0 {
                            0.0
                        } else {
                            (nrm[0] * l[0] + nrm[1] * l[1] + nrm[2] * l[2]) / len
                        }
                    });
                    let (a, b) = (chromaticity(ph), chromaticity(t.rgb));
                    let dc = [a[0] - b[0], a[1] - b[1]];
                    let ds = preds(t.rgb).map(|q| dist(q, ph));
                    for (si, (dx, dy)) in SHIFTS.iter().enumerate() {
                        let (sx, sy) = (x as i64 + dx, y as i64 + dy);
                        if sx < 0 || sy < 0 || sx >= photo.width as i64 || sy >= photo.height as i64
                        {
                            continue;
                        }
                        let si_at = (sy as usize * photo.width + sx as usize) * 4;
                        let q = [0, 1, 2].map(|c| f32::from(photo.rgba[si_at + c]));
                        shift_agree[usize::from(who.0)][si] +=
                            u32::from(dist(t.rgb, q) <= HUE.distance);
                    }
                    let k = bin(nl);
                    for target in [
                        &mut obj,
                        &mut by_bin[k],
                        &mut owners[usize::from(who.0)],
                        &mut owners_bin[usize::from(who.0)][k],
                    ] {
                        add(target, ph, t.rgb, ds, HUE.distance, dc);
                    }
                }
            }
            line(&format!("{name} piece {}", who.2), &obj);
            for (k, a) in by_bin.iter().enumerate() {
                line(&format!("  {name} {}", BINS[k]), a);
            }
        }
        for o in 0..2 {
            line(&format!("player {o}"), &owners[o]);
            for (k, a) in owners_bin[o].iter().enumerate() {
                line(&format!("  player {o} {}", BINS[k]), a);
            }
        }
        line("the pine's leaf by 0/2, 0/8, 0/10", &leaves);
        println!(
            "0/2, 0/8, 0/10 behind the pine: n {behind_n}, c·a + n·(1 − a) agreeing over the leaf texel {}, over the leaf^1.45 {}",
            behind_agree.0, behind_agree.1
        );
        for o in 0..2 {
            println!(
                "player {o} texel agreeing by photograph shift (dx, dy): {:?}",
                SHIFTS.iter().zip(shift_agree[o]).collect::<Vec<_>>()
            );
        }
        // The kill, as assertions (item 4106): player 1's net multiplier is
        // grey within 0.02 of chromaticity, the packet's coloured ambient
        // loses to the texel alone, the unshifted photograph agrees most for
        // both players, and the leaf's power under 4103's blend agrees less.
        let p1 = &owners[1];
        let n1 = f64::from(p1.n);
        assert!((p1.dc[0] / n1).abs() < 0.02 && (p1.dc[1] / n1).abs() < 0.02);
        assert!(
            p1.agree[0] > p1.agree[1],
            "the ambient's colour does not raise player 1"
        );
        for o in 0..2 {
            let best = shift_agree[o].iter().max().copied();
            assert_eq!(best, Some(shift_agree[o][0]), "player {o}: registration");
        }
        assert!(
            behind_agree.1 < behind_agree.0,
            "the leaf's power under the silhouette"
        );
    }

    /// **Item 4107, step 1: the peasants' alpha, as a value.** On 4106's
    /// pixels (both footprints hold them, not behind anything, the painter's
    /// texel at alpha 255, neither side dark), per (owner, piece): the
    /// texture's alpha at every level of its chain and at the painter's
    /// level and one either side, nearest-texel and bilinear, and the team
    /// blend `t·α + c·(1 − α)` at that level and filter against the
    /// photograph, agreeing as 4106's candidates were counted. Beside it,
    /// each texture's alpha histogram per level. No constant is fitted.
    #[test]
    #[allow(clippy::needless_range_loop)]
    fn the_peasants_alpha() {
        use render::silhouette::{HUE, chromaticity, team_blend, team_unit, traced_theirs};
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
        world.run_to(PHOTO_FRAME);
        world.blend = crate::world::Blend::Last;
        world.run_to(PHOTO_FRAME + 1);
        let size = (photo.width, photo.height);
        let scene = frame_scene(&mut world, &camera, size);
        let p = Photo {
            width: photo.width,
            height: photo.height,
            rgba: &photo.rgba,
        };
        let traces: Vec<Trace<'_>> = TRACED
            .iter()
            .map(|&(name, poly)| Trace {
                key: scene
                    .who
                    .iter()
                    .position(|&(who, o, _, _)| format!("{who}/{o}") == name)
                    .map_or(u32::MAX, |i| i as u32 + 1),
                poly,
            })
            .collect();
        let rows = footprints_traced(
            &scene.keys,
            &p,
            &INTERFACE,
            &READING,
            &traces,
            Neighbour::Neither,
            Interface::Neither,
        );
        let dist = |a: [f32; 3], b: [f32; 3]| {
            let (x, y) = (chromaticity(a), chromaticity(b));
            ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2)).sqrt()
        };
        let sum = |q: [f32; 3]| q[0] + q[1] + q[2];
        // Nearest texel at (u, v) on a level, wrapped.
        let nearest = |m: &render::silhouette::Mips, level: usize, u: f32, v: f32| {
            let im = &m.levels[level];
            let (w, h) = (im.width as i64, im.height as i64);
            let x = ((u * w as f32).floor() as i64).rem_euclid(w) as usize;
            let y = ((v * h as f32).floor() as i64).rem_euclid(h) as usize;
            let o = (y * im.width + x) * 4;
            [0, 1, 2, 3].map(|c| f32::from(im.rgba[o + c]))
        };
        // Columns: absolute levels 0..8, then the painter's −1, 0, +1.
        const COLS: usize = 12;
        let col_name = |c: usize| match c {
            0..=8 => format!("L{c}"),
            9 => "P-1".into(),
            10 => "P".into(),
            _ => "P+1".into(),
        };
        #[derive(Clone, Default)]
        struct Cell {
            n: u32,
            masked: u32,
            alpha: f64,
            agree_blend: u32,
            agree_texel: u32,
        }
        type Group = (u8, i32);
        let mut groups: std::collections::BTreeMap<Group, ([[Cell; COLS]; 2], u32, String)> =
            Default::default();
        let mut textures: std::collections::BTreeMap<i32, String> = Default::default();
        let mut painter_levels: std::collections::BTreeMap<Group, [u32; 10]> = Default::default();
        // 4106's own selection (alpha ≥ 1 on the 0–255 scale) split by the
        // painter's alpha: [255, 1–254] × (n, Δr, Δg, texel agreeing, blend
        // agreeing), per group.
        let mut split: std::collections::BTreeMap<Group, [[f64; 5]; 2]> = Default::default();
        for f in rows {
            let who = scene.who[f.key as usize - 1];
            if who.0 == TREE_WHO || who.3 || who.0 > 1 {
                continue;
            }
            let Some(d) = scene.drawings.get(f.key as usize - 1) else {
                continue;
            };
            let Some(mips) = world.figure_texture(d.piece) else {
                continue;
            };
            textures.insert(
                d.piece,
                world
                    .figure_texture_name(d.piece)
                    .unwrap_or("?")
                    .to_string(),
            );
            let theirs = traced_theirs(
                &scene.keys,
                &p,
                &INTERFACE,
                &READING,
                &traces,
                Neighbour::Neither,
                Interface::Neither,
                f.key,
                f.window,
            );
            let ours = painted(
                &mut world,
                &scene,
                &camera,
                size,
                f.key,
                f.window,
                Colouring::Texture,
            );
            let owner = team_unit(who.0);
            let g = groups
                .entry((who.0, d.piece))
                .or_insert_with(|| (Default::default(), 0, String::new()));
            g.2.push_str(&format!(" {}/{}", who.0, who.1));
            let pl = painter_levels.entry((who.0, d.piece)).or_default();
            let ww = (f.window.x1 - f.window.x0) as usize;
            let last = mips.levels.len() - 1;
            for y in f.window.y0..f.window.y1 {
                for x in f.window.x0..f.window.x1 {
                    let j = (y - f.window.y0) as usize * ww + (x - f.window.x0) as usize;
                    let at = y as usize * photo.width + x as usize;
                    let ph = [0, 1, 2].map(|c| f32::from(photo.rgba[at * 4 + c]));
                    if scene.keys.at(x as usize, y as usize) != f.key
                        || theirs[j] != Some(true)
                        || scene.behind[at]
                    {
                        continue;
                    }
                    let Some(t) = ours[j] else { continue };
                    if t.alpha < 1.0 || sum(t.rgb) < HUE.dark || sum(ph) < HUE.dark {
                        continue;
                    }
                    let (a, b) = (chromaticity(ph), chromaticity(t.rgb));
                    let row = &mut split.entry((who.0, d.piece)).or_default()
                        [usize::from(t.alpha < 255.0)];
                    row[0] += 1.0;
                    row[1] += f64::from(a[0] - b[0]);
                    row[2] += f64::from(a[1] - b[1]);
                    row[3] += f64::from(u8::from(dist(t.rgb, ph) <= HUE.distance));
                    row[4] += f64::from(u8::from(
                        dist(team_blend(t.rgb, t.alpha, owner), ph) <= HUE.distance,
                    ));
                    if t.alpha < 255.0 {
                        continue;
                    }
                    g.1 += 1;
                    pl[usize::from(t.level).min(9)] += 1;
                    let pv = usize::from(t.level);
                    for c in 0..COLS {
                        let level = match c {
                            0..=8 => c,
                            9 => pv.saturating_sub(1),
                            10 => pv,
                            _ => pv + 1,
                        };
                        if level > last {
                            continue;
                        }
                        for (fi, s) in [
                            nearest(&mips, level, t.uv[0], t.uv[1]),
                            mips.sample(level, t.uv[0], t.uv[1]),
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            let rgb = [s[0], s[1], s[2]];
                            let cell = &mut g.0[fi][c];
                            cell.n += 1;
                            cell.masked += u32::from(s[3] < 254.5);
                            cell.alpha += f64::from(s[3]);
                            cell.agree_blend +=
                                u32::from(dist(team_blend(rgb, s[3], owner), ph) <= HUE.distance);
                            cell.agree_texel += u32::from(dist(rgb, ph) <= HUE.distance);
                        }
                    }
                }
            }
        }
        for (piece, name) in &textures {
            let Some(m) = world.figure_texture(*piece) else {
                continue;
            };
            println!("piece {piece} texture {name}");
            for (l, im) in m.levels.iter().enumerate() {
                let mut h = [0u32; 5];
                for px in im.rgba.as_chunks::<4>().0 {
                    let a = px[3];
                    h[match a {
                        255 => 0,
                        192..=254 => 1,
                        128..=191 => 2,
                        1..=127 => 3,
                        _ => 4,
                    }] += 1;
                }
                println!(
                    "  L{l} {}x{} alpha 255 {} | 192-254 {} | 128-191 {} | 1-127 {} | 0 {}",
                    im.width, im.height, h[0], h[1], h[2], h[3], h[4]
                );
            }
        }
        for ((o, piece), (cells, n, names)) in &groups {
            println!(
                "owner {o} piece {piece}{names}: n {n}, painter's levels {:?}",
                painter_levels[&(*o, *piece)]
            );
            for (k, label) in ["α 255", "α 1–254"].iter().enumerate() {
                let r = split[&(*o, *piece)][k];
                if r[0] > 0.0 {
                    println!(
                        "  4106's selection, {label}: n {}, Δchroma ({:+.3}, {:+.3}), texel agreeing {}, blend agreeing {}",
                        r[0],
                        r[1] / r[0],
                        r[2] / r[0],
                        r[3],
                        r[4]
                    );
                }
            }
            for (fi, fname) in ["nearest", "bilinear"].iter().enumerate() {
                let mut l = format!("  {fname}:");
                for c in 0..COLS {
                    let cell = &cells[fi][c];
                    if cell.n == 0 {
                        continue;
                    }
                    l.push_str(&format!(
                        " {} α̅{:.1} <255 {} blend {} texel {} |",
                        col_name(c),
                        cell.alpha / f64::from(cell.n),
                        cell.masked,
                        cell.agree_blend,
                        cell.agree_texel
                    ));
                }
                println!("{l}");
            }
        }
        // The kill, as assertions (item 4107): on the peasants' (pieces
        // 4576, 10912) alpha-255 pixels the photograph is grey over our
        // texel, within 0.05 of chromaticity, and the team blend at no
        // level's or filter's alpha agrees on more of them than that
        // level's texel alone; their red is on the masked pixels, where the
        // blend 4102 draws beats the texel.
        for piece in [4576, 10912] {
            let r = split[&(0, piece)];
            let (n, m) = (r[0], r[1]);
            assert!(
                (n[1] / n[0]).abs() < 0.05 && (n[2] / n[0]).abs() < 0.05,
                "piece {piece}: grey at alpha 255"
            );
            assert!(m[4] > m[3], "piece {piece}: the blend on the masked pixels");
            let cells = &groups[&(0, piece)].0;
            for fi in 0..2 {
                for c in 0..COLS {
                    let cell = &cells[fi][c];
                    assert!(
                        cell.agree_blend <= cell.agree_texel,
                        "piece {piece}: {} raises the blend",
                        col_name(c)
                    );
                }
            }
        }
    }

    /// **Item 4110: the colour and the share the photograph implies, as
    /// values.** The figures' masked lit pixels: both footprints hold the
    /// pixel, it is not behind anything, the painter's texel has alpha
    /// under 255, neither side is dark. First the selection itself, per
    /// owner: the predicate, and the selecting channel's minimum, maximum
    /// and count. Then per (owner, piece) and per alpha band of 32: the mean
    /// photograph `p` and texel `t`, the colour the photograph implies
    /// under the drawn rule, `(p − t·α)/(1 − α)` per pixel (its mean and
    /// spread) and pooled by least squares, beside the owner's `unit`
    /// colour; and the least-squares share `s` in `p = t·s + c·(1 − s)`,
    /// beside α. `k`, the photograph over the texel on the same piece's
    /// alpha-255 pixels, is printed beside them and a `k`-corrected column
    /// with it; nothing is fitted to the masked pixels' count.
    #[test]
    #[allow(clippy::needless_range_loop)]
    fn the_implied_colour() {
        use render::silhouette::{HUE, chromaticity, team_blend, team_unit, traced_theirs};
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
        world.run_to(PHOTO_FRAME);
        world.blend = crate::world::Blend::Last;
        world.run_to(PHOTO_FRAME + 1);
        let size = (photo.width, photo.height);
        let scene = frame_scene(&mut world, &camera, size);
        let p = Photo {
            width: photo.width,
            height: photo.height,
            rgba: &photo.rgba,
        };
        let traces: Vec<Trace<'_>> = TRACED
            .iter()
            .map(|&(name, poly)| Trace {
                key: scene
                    .who
                    .iter()
                    .position(|&(who, o, _, _)| format!("{who}/{o}") == name)
                    .map_or(u32::MAX, |i| i as u32 + 1),
                poly,
            })
            .collect();
        let rows = footprints_traced(
            &scene.keys,
            &p,
            &INTERFACE,
            &READING,
            &traces,
            Neighbour::Neither,
            Interface::Neither,
        );
        let dist = |a: [f64; 3], b: [f64; 3]| {
            let f = |q: [f64; 3]| q.map(|v| v as f32);
            let (x, y) = (chromaticity(f(a)), chromaticity(f(b)));
            f64::from(((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2)).sqrt())
        };
        let sum = |q: [f32; 3]| q[0] + q[1] + q[2];
        // One masked pixel: (alpha 0–255, texel, photograph).
        type Px = (f64, [f64; 3], [f64; 3]);
        // Per (owner, piece): masked pixels; and the alpha-255 sums for k.
        let mut masked: std::collections::BTreeMap<(u8, i32), Vec<Px>> = Default::default();
        let mut opaque: std::collections::BTreeMap<(u8, i32), (f64, f64, u32)> = Default::default();
        // The selection, per owner: (count, min α, max α); and what 4102's
        // selection kept beside it (owners past 1, alpha under 1).
        let mut sel: std::collections::BTreeMap<u8, (u32, f32, f32)> = Default::default();
        let mut extra = (0u32, 0u32);
        let mut parting = (0u32, 0u32);
        for f in rows {
            let who = scene.who[f.key as usize - 1];
            if who.0 == TREE_WHO || who.3 {
                continue;
            }
            let Some(d) = scene.drawings.get(f.key as usize - 1) else {
                continue;
            };
            let piece = d.piece;
            let theirs = traced_theirs(
                &scene.keys,
                &p,
                &INTERFACE,
                &READING,
                &traces,
                Neighbour::Neither,
                Interface::Neither,
                f.key,
                f.window,
            );
            let ours = painted(
                &mut world,
                &scene,
                &camera,
                size,
                f.key,
                f.window,
                Colouring::Texture,
            );
            let ww = (f.window.x1 - f.window.x0) as usize;
            for y in f.window.y0..f.window.y1 {
                for x in f.window.x0..f.window.x1 {
                    let j = (y - f.window.y0) as usize * ww + (x - f.window.x0) as usize;
                    let at = y as usize * photo.width + x as usize;
                    if scene.keys.at(x as usize, y as usize) != f.key
                        || theirs[j] != Some(true)
                        || scene.behind[at]
                    {
                        continue;
                    }
                    let Some(t) = ours[j] else { continue };
                    let ph = [0, 1, 2].map(|c| f32::from(photo.rgba[at * 4 + c]));
                    if sum(t.rgb) < HUE.dark || sum(ph) < HUE.dark {
                        continue;
                    }
                    let tt = t.rgb.map(f64::from);
                    let pp = ph.map(f64::from);
                    if t.alpha >= 255.0 {
                        let o = opaque.entry((who.0, piece)).or_default();
                        o.0 += (0..3).map(|k| pp[k] * tt[k]).sum::<f64>();
                        o.1 += (0..3).map(|k| tt[k] * tt[k]).sum::<f64>();
                        o.2 += 1;
                        continue;
                    }
                    // 4102's selection: every owner, any alpha under 255.
                    let blend = team_blend(t.rgb, t.alpha, team_unit(who.0)).map(f64::from);
                    let parts = dist(blend, pp) > f64::from(HUE.distance);
                    if who.0 > 1 || t.alpha < 1.0 {
                        if who.0 > 1 {
                            extra.0 += 1;
                        } else {
                            extra.1 += 1;
                        }
                        parting.1 += u32::from(parts);
                        if who.0 > 1 {
                            continue;
                        }
                    }
                    parting.0 += u32::from(parts);
                    let e = sel.entry(who.0).or_insert((0, f32::MAX, f32::MIN));
                    e.0 += 1;
                    e.1 = e.1.min(t.alpha);
                    e.2 = e.2.max(t.alpha);
                    masked
                        .entry((who.0, piece))
                        .or_default()
                        .push((f64::from(t.alpha), tt, pp));
                }
            }
        }
        println!(
            "selection: key holds it, the photograph's footprint holds it, not behind, \
             neither side dark (sum < {}), the painter's texel alpha < 255.0 on 0–255; owners 0 and 1",
            HUE.dark
        );
        for (o, (n, lo, hi)) in &sel {
            println!("  owner {o}: n {n}, alpha min {lo}, max {hi}");
        }
        println!(
            "  beside it, 4102's selection also kept: owners past 1 {}, alpha under 1 (owners 0, 1) {}; \
             parting under the blend: ours {} , theirs' extra {}",
            extra.0, extra.1, parting.0, parting.1
        );
        let f3 = |q: [f64; 3]| format!("({:.0},{:.0},{:.0})", q[0], q[1], q[2]);
        // The rows: per (owner, piece, band) and per (owner, band) over all pieces.
        type Row = ([f64; 3], f64, f64);
        let report = |label: String, px: &[&Px], c: [f64; 3], k: f64| -> Option<Row> {
            let n = px.len() as f64;
            if px.is_empty() {
                return None;
            }
            let mut ab = 0.0;
            let mut pm = [0.0; 3];
            let mut tm = [0.0; 3];
            let mut im = [0.0; 3];
            let mut im2 = [0.0; 3];
            let mut ls = ([0.0; 3], 0.0);
            let mut lsk = ([0.0; 3], 0.0);
            let (mut sn, mut sd, mut skn, mut skd) = (0.0, 0.0, 0.0, 0.0);
            let (mut ag_t, mut ag_b, mut ag_c, mut ag_bk) = (0, 0, 0, 0);
            for &&(alpha, t, ph) in px {
                let a = alpha / 255.0;
                ab += alpha;
                for q in 0..3 {
                    pm[q] += ph[q];
                    tm[q] += t[q];
                    let v = (ph[q] - t[q] * a) / (1.0 - a);
                    im[q] += v;
                    im2[q] += v * v;
                    ls.0[q] += (1.0 - a) * (ph[q] - t[q] * a);
                    lsk.0[q] += (1.0 - a) * (ph[q] / k - t[q] * a);
                    sn += (ph[q] - c[q]) * (t[q] - c[q]);
                    sd += (t[q] - c[q]).powi(2);
                    skn += (ph[q] / k - c[q]) * (t[q] - c[q]);
                    skd += (t[q] - c[q]).powi(2);
                }
                ls.1 += (1.0 - a).powi(2);
                lsk.1 += (1.0 - a).powi(2);
                let b = [0, 1, 2].map(|q| t[q] * a + c[q] * (1.0 - a));
                ag_t += u32::from(dist(t, ph) <= f64::from(HUE.distance));
                ag_b += u32::from(dist(b, ph) <= f64::from(HUE.distance));
                ag_c += u32::from(dist(c, ph) <= f64::from(HUE.distance));
                let bk = b.map(|v| v * k);
                ag_bk += u32::from(dist(bk, ph) <= f64::from(HUE.distance));
            }
            let mean = |q: [f64; 3]| q.map(|v| v / n);
            let sdv = [0, 1, 2].map(|q| (im2[q] / n - (im[q] / n).powi(2)).max(0.0).sqrt());
            let lsc = ls.0.map(|v| v / ls.1);
            let lskc = lsk.0.map(|v| v / lsk.1);
            println!(
                "{label} n {:>4} α̅ {:>5.1} ({:.2}) p̅ {} t̅ {} | implied {} ±{} ls {} ls/k {} [c {}] | s {:.2} s/k {:.2} | agree texel {} blend {} owner {} k·blend {}",
                px.len(),
                ab / n,
                ab / n / 255.0,
                f3(mean(pm)),
                f3(mean(tm)),
                f3(mean(im)),
                f3(sdv),
                f3(lsc),
                f3(lskc),
                f3(c),
                sn / sd,
                skn / skd,
                ag_t,
                ag_b,
                ag_c,
                ag_bk
            );
            Some((lsc, sn / sd, skn / skd))
        };
        let band = |alpha: f64| ((alpha as usize) / 32).min(7);
        // (owner, piece or −1 for every piece, band) -> (ls colour, s, s/k).
        let mut out: std::collections::BTreeMap<(u8, i32, usize), Row> = Default::default();
        for ((o, piece), px) in &masked {
            let c = team_unit(*o).map(f64::from);
            let (pt, tt, no) = opaque.get(&(*o, *piece)).copied().unwrap_or((1.0, 1.0, 0));
            let k = pt / tt;
            println!(
                "owner {o} piece {piece} ({}): masked {}, k {:.3} on {no} alpha-255 pixels",
                world.figure_texture_name(*piece).unwrap_or("?"),
                px.len(),
                k
            );
            let all: Vec<&Px> = px.iter().collect();
            report("  all     ".into(), &all, c, k);
            for b in 0..8 {
                let in_b: Vec<&Px> = px.iter().filter(|q| band(q.0) == b).collect();
                if let Some(r) = report(format!("  α{:>3}–{:<3}", b * 32, b * 32 + 31), &in_b, c, k)
                {
                    out.insert((*o, *piece, b), r);
                }
            }
        }
        for o in [0u8, 1] {
            let c = team_unit(o).map(f64::from);
            let (pt, tt): (f64, f64) = opaque
                .iter()
                .filter(|((w, _), _)| *w == o)
                .fold((0.0, 0.0), |a, (_, v)| (a.0 + v.0, a.1 + v.1));
            let k = pt / tt;
            println!("owner {o}, every piece: k {k:.3}");
            for b in 0..8 {
                let in_b: Vec<&Px> = masked
                    .iter()
                    .filter(|((w, _), _)| *w == o)
                    .flat_map(|(_, v)| v.iter())
                    .filter(|q| band(q.0) == b)
                    .collect();
                if let Some(r) = report(format!("  α{:>3}–{:<3}", b * 32, b * 32 + 31), &in_b, c, k)
                {
                    out.insert((o, -1, b), r);
                }
            }
        }
        // The verdict, as assertions (item 4110). The selection is the
        // masked population 4107 counted for player 0 and, with its 25
        // pixels under alpha 1, for player 1.
        assert_eq!(sel[&0].0, 563, "player 0's masked pixels");
        assert_eq!(sel[&1].0, 2102, "player 1's masked pixels");
        assert!(sel.values().all(|v| v.1 >= 0.0 && v.2 < 255.0));
        // The colour is the owner's: where the implied colour is steady
        // (player 0's bands 32–127, the dragoon 26352's 64–191), its
        // chromaticity is nearer the owner's `unit` than its `unit_tint`
        // (run1559b's packet, `TeamColor` +0x4c: (127, 47, 47), (47, 47, 159)).
        let chroma_d = |a: [f64; 3], b: [f64; 3]| dist(a, b);
        for (o, piece, bands, tint) in [
            (0u8, -1, 1..=3, [127.0, 47.0, 47.0]),
            (1, 26352, 2..=5, [47.0, 47.0, 159.0]),
        ] {
            let c = team_unit(o).map(f64::from);
            for b in bands {
                let (ls, _, _) = out[&(o, piece, b)];
                assert!(
                    chroma_d(ls, c) < chroma_d(ls, tint),
                    "owner {o} piece {piece} band {b}: the implied colour is the tint's"
                );
            }
        }
        // The share is no one curve of α: at the same band (α 64–95) the
        // riflemen 60 read over 0.45 of texel and the dragoon 26352 under
        // 0.35, both under the photograph's own gain; the tanks 196 read
        // the texel nearly alone (over 0.8) in every band from 64 to 191.
        assert!(out[&(1, 60, 2)].2 > 0.45 && out[&(1, 26352, 2)].2 < 0.35);
        for b in 2..=5 {
            assert!(out[&(1, 196, b)].1 > 0.8, "the tanks' band {b}");
        }
    }

    /// **Item 4112: the level the card samples, as values.** Every lit
    /// pixel of each figure: its key holds it, the photograph's footprint
    /// holds it, it is not behind anything, neither the painter's blend
    /// (`Colouring::Team`, the colour row's own) nor the photograph sums
    /// under `HUE.dark` — whatever the painter's alpha. Per (owner, piece)
    /// and per column — the painter's level bilinear (the drawn rule) and
    /// nearest, then each fixed level 0 to 4 nearest and bilinear — the
    /// pixels whose blend `t·α + c·(1 − α)` agrees with the photograph and
    /// the pixels that column calls masked (alpha under 255). Then, at the
    /// painter's column and the best one, a texcoord shift of half a texel
    /// each way in u and v at that column's level. Then each colour-row
    /// object re-counted under each column, the pixels behind left as the
    /// silhouette paints them. Nothing is fitted; nothing is drawn.
    #[test]
    #[allow(clippy::needless_range_loop)]
    fn the_level_the_card_samples() {
        use render::silhouette::{HUE, chromaticity, team_blend, team_unit, traced_theirs};
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
        world.run_to(PHOTO_FRAME);
        world.blend = crate::world::Blend::Last;
        world.run_to(PHOTO_FRAME + 1);
        let size = (photo.width, photo.height);
        // The colour row as it stands: the baseline each object is
        // re-counted from.
        let base = colour_rows(&mut world, &camera, &photo, &TRACED, Colouring::Silhouette);
        let scene = frame_scene(&mut world, &camera, size);
        let p = Photo {
            width: photo.width,
            height: photo.height,
            rgba: &photo.rgba,
        };
        let traces: Vec<Trace<'_>> = TRACED
            .iter()
            .map(|&(name, poly)| Trace {
                key: scene
                    .who
                    .iter()
                    .position(|&(who, o, _, _)| format!("{who}/{o}") == name)
                    .map_or(u32::MAX, |i| i as u32 + 1),
                poly,
            })
            .collect();
        let rows = footprints_traced(
            &scene.keys,
            &p,
            &INTERFACE,
            &READING,
            &traces,
            Neighbour::Neither,
            Interface::Neither,
        );
        let agrees = |a: [f32; 3], b: [f32; 3]| {
            let (x, y) = (chromaticity(a), chromaticity(b));
            ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2)).sqrt() <= HUE.distance
        };
        let sum = |q: [f32; 3]| q[0] + q[1] + q[2];
        let nearest = |m: &render::silhouette::Mips, level: usize, u: f32, v: f32| {
            let im = &m.levels[level];
            let (w, h) = (im.width as i64, im.height as i64);
            let x = ((u * w as f32).floor() as i64).rem_euclid(w) as usize;
            let y = ((v * h as f32).floor() as i64).rem_euclid(h) as usize;
            let o = (y * im.width + x) * 4;
            [0, 1, 2, 3].map(|c| f32::from(im.rgba[o + c]))
        };
        // Columns: 0 the painter's level bilinear, 1 nearest; 2..=6 levels
        // 0..4 nearest; 7..=11 levels 0..4 bilinear.
        const COLS: usize = 12;
        let col_name = |c: usize| match c {
            0 => "P bil".to_string(),
            1 => "P near".to_string(),
            2..=6 => format!("L{} near", c - 2),
            _ => format!("L{} bil", c - 7),
        };
        let col = |c: usize, pv: usize| -> (usize, bool) {
            match c {
                0 => (pv, true),
                1 => (pv, false),
                2..=6 => (c - 2, false),
                _ => (c - 7, true),
            }
        };
        const SHIFTS: [(f32, f32); 9] = [
            (0.0, 0.0),
            (-0.5, 0.0),
            (0.5, 0.0),
            (0.0, -0.5),
            (0.0, 0.5),
            (-0.5, -0.5),
            (0.5, -0.5),
            (-0.5, 0.5),
            (0.5, 0.5),
        ];
        #[derive(Clone, Copy, Default)]
        struct Cell {
            agree: u32,
            masked: u32,
        }
        #[derive(Default)]
        struct Group {
            n: u32,
            painter_masked: u32,
            levels: [u32; 8],
            cells: [[Cell; 9]; COLS],
            names: String,
        }
        let mut groups: std::collections::BTreeMap<(u8, i32), Group> = Default::default();
        // Per object key: the non-behind lit pixels and, per column, agreeing.
        let mut per_key: std::collections::BTreeMap<u32, (u32, [u32; COLS])> = Default::default();
        let mut sel: std::collections::BTreeMap<u8, (u32, u32)> = Default::default();
        for f in rows {
            let who = scene.who[f.key as usize - 1];
            if who.0 == TREE_WHO || who.3 {
                continue;
            }
            let Some(d) = scene.drawings.get(f.key as usize - 1) else {
                continue;
            };
            let Some(mips) = world.figure_texture(d.piece) else {
                continue;
            };
            let theirs = traced_theirs(
                &scene.keys,
                &p,
                &INTERFACE,
                &READING,
                &traces,
                Neighbour::Neither,
                Interface::Neither,
                f.key,
                f.window,
            );
            let ours = painted(
                &mut world,
                &scene,
                &camera,
                size,
                f.key,
                f.window,
                Colouring::Texture,
            );
            let owner = team_unit(who.0);
            let g = groups.entry((who.0, d.piece)).or_default();
            g.names.push_str(&format!(" {}/{}", who.0, who.1));
            let pk = per_key.entry(f.key).or_default();
            let ww = (f.window.x1 - f.window.x0) as usize;
            let last = mips.levels.len() - 1;
            for y in f.window.y0..f.window.y1 {
                for x in f.window.x0..f.window.x1 {
                    let j = (y - f.window.y0) as usize * ww + (x - f.window.x0) as usize;
                    let at = y as usize * photo.width + x as usize;
                    if scene.keys.at(x as usize, y as usize) != f.key
                        || theirs[j] != Some(true)
                        || scene.behind[at]
                    {
                        continue;
                    }
                    let Some(t) = ours[j] else { continue };
                    let ph = [0, 1, 2].map(|c| f32::from(photo.rgba[at * 4 + c]));
                    if sum(team_blend(t.rgb, t.alpha, owner)) < HUE.dark || sum(ph) < HUE.dark {
                        continue;
                    }
                    g.n += 1;
                    pk.0 += 1;
                    let s = sel.entry(who.0).or_default();
                    s.0 += 1;
                    s.1 += u32::from(t.alpha < 255.0);
                    g.painter_masked += u32::from(t.alpha < 255.0);
                    let pv = usize::from(t.level);
                    g.levels[pv.min(7)] += 1;
                    for c in 0..COLS {
                        let (level, bil) = col(c, pv);
                        let level = level.min(last);
                        let im = &mips.levels[level];
                        for (si, &(du, dv)) in SHIFTS.iter().enumerate() {
                            let u = t.uv[0] + du / im.width as f32;
                            let v = t.uv[1] + dv / im.height as f32;
                            let q = if bil {
                                mips.sample(level, u, v)
                            } else {
                                nearest(&mips, level, u, v)
                            };
                            let ok = agrees(team_blend([q[0], q[1], q[2]], q[3], owner), ph);
                            let cell = &mut g.cells[c][si];
                            cell.agree += u32::from(ok);
                            cell.masked += u32::from(q[3] < 255.0);
                            if si == 0 {
                                pk.1[c] += u32::from(ok);
                            }
                        }
                    }
                }
            }
        }
        println!(
            "selection: key holds it, the photograph's footprint holds it, not behind, \
             neither the painter's blend nor the photograph sums under {}; every alpha",
            HUE.dark
        );
        for (o, (n, m)) in &sel {
            println!("  owner {o}: lit {n}, of them the painter's alpha < 255: {m}");
        }
        let mut totals = [0u32; COLS];
        for ((o, piece), g) in &groups {
            println!(
                "owner {o} piece {piece} ({}){}: lit {}, painter masked {}, painter's levels {:?}",
                world.figure_texture_name(*piece).unwrap_or("?"),
                g.names,
                g.n,
                g.painter_masked,
                g.levels
            );
            let mut l = String::from("   ");
            for c in 0..COLS {
                totals[c] += g.cells[c][0].agree;
                l.push_str(&format!(
                    " {} {}/{}m |",
                    col_name(c),
                    g.cells[c][0].agree,
                    g.cells[c][0].masked
                ));
            }
            println!("{l}");
        }
        let best = (1..COLS).max_by_key(|&c| totals[c]).expect("a column");
        println!("totals: {:?}; best column {}", totals, col_name(best));
        for c in [0, best] {
            println!("shift at {} (du, dv in texels):", col_name(c));
            for ((o, piece), g) in &groups {
                let mut l = format!("  {o}/{piece}:");
                for (si, sh) in SHIFTS.iter().enumerate() {
                    l.push_str(&format!(" {:?} {} |", sh, g.cells[c][si].agree));
                }
                println!("{l}");
            }
            let tot: Vec<u32> = (0..9)
                .map(|si| groups.values().map(|g| g.cells[c][si].agree).sum())
                .collect();
            println!("  total {tot:?}");
        }
        // Each colour-row object re-counted under each column: its
        // non-behind lit pixels' agreement replaced, the behind kept.
        let mut passing = [0usize; COLS];
        let mut down = [0usize; COLS];
        for (f, c, w) in &base {
            let Some((n, ag)) = per_key.get(&f.key) else {
                passing
                    .iter_mut()
                    .for_each(|v| *v += usize::from(c.passes(&HUE)));
                continue;
            };
            let rest = c.agree - ag[0];
            let mut l = format!("  {}/{} lit {} ({n} not behind):", w.0, w.1, c.lit);
            for k in 0..COLS {
                let q = render::silhouette::Colours {
                    agree: rest + ag[k],
                    ..*c
                };
                passing[k] += usize::from(q.passes(&HUE));
                down[k] += usize::from(c.passes(&HUE) && !q.passes(&HUE));
                l.push_str(&format!(" {}", q.agree));
            }
            println!("{l}");
        }
        println!("colour row per column: {passing:?}; down {down:?}");
        // The kill, as assertions (item 4112). The painter's column is the
        // row; no column raises it; no column beats the painter's on every
        // piece; no half-texel shift beats the unshifted sample in total;
        // and on the medium tank 196 a finer level (0, either filter) lowers
        // the agreement rather than turning its masked pixels to paint.
        assert_eq!(
            passing[0],
            render::silhouette::RUN1559B_3279_COLOUR.0,
            "the painter's column is the row"
        );
        for k in 1..COLS {
            assert!(passing[k] <= passing[0], "{} raises the row", col_name(k));
            assert!(
                groups
                    .values()
                    .any(|g| g.cells[k][0].agree < g.cells[0][0].agree),
                "{} beats the painter's on every piece",
                col_name(k)
            );
        }
        for c in [0, best] {
            let at = |si: usize| -> u32 { groups.values().map(|g| g.cells[c][si].agree).sum() };
            for si in 1..9 {
                assert!(at(si) < at(0), "{}: shift {:?}", col_name(c), SHIFTS[si]);
            }
        }
        let tank = &groups[&(1, 196)];
        for k in [2, 7] {
            assert!(
                tank.cells[k][0].agree < tank.cells[0][0].agree,
                "the tank at {}",
                col_name(k)
            );
        }
    }

    /// **Item 4114: the photograph over our texel, as the original's pixel
    /// pipeline.** Step 1, the values: on 4106's population at the
    /// painter's alpha exactly 255 (key, both footprints, not behind,
    /// neither the texel nor the photograph dark) and on the pine's leaf
    /// as 4106 selects it, per channel the photograph's median against the
    /// texel's value in 16 bins, by the face's band (N·L against
    /// `L = −cur_dir`, 4106's three) and by piece; whether the pieces sit
    /// on one curve (a grey texel's median image through each piece's
    /// three curves, in chromaticity, the reader's 0.05). Then the
    /// premise of the candidate (`render::silhouette::lit_pipeline`,
    /// committed before this count): the frame's mean luminance, inverted
    /// from the photograph on the pixels no object's key holds and no
    /// interface covers, under the clamp. Then the count: the candidate
    /// against the texel on the alpha-255 population and against the
    /// painter's blend on 4112's every-lit population, the team blend
    /// first (the card's stage runs before the lit branch), and the colour
    /// row re-counted under it, the pixels behind left as painted.
    #[test]
    #[allow(clippy::needless_range_loop, clippy::type_complexity)]
    fn the_lit_pixel() {
        use render::silhouette::{
            HUE, MIN_LUMINANCE, WHITE_POINT, chromaticity, filmic, lit_pipeline, team_blend,
            team_unit, traced_theirs,
        };
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
        world.run_to(PHOTO_FRAME);
        world.blend = crate::world::Blend::Last;
        world.run_to(PHOTO_FRAME + 1);
        let size = (photo.width, photo.height);
        let base = colour_rows(&mut world, &camera, &photo, &TRACED, Colouring::Silhouette);
        let scene = frame_scene(&mut world, &camera, size);
        let first_tree = (scene.drawings.len() + scene.sprites.len()) as u32 + 1;
        let p = Photo {
            width: photo.width,
            height: photo.height,
            rgba: &photo.rgba,
        };
        let traces: Vec<Trace<'_>> = TRACED
            .iter()
            .map(|&(name, poly)| Trace {
                key: scene
                    .who
                    .iter()
                    .position(|&(who, o, _, _)| format!("{who}/{o}") == name)
                    .map_or(u32::MAX, |i| i as u32 + 1),
                poly,
            })
            .collect();
        let rows = footprints_traced(
            &scene.keys,
            &p,
            &INTERFACE,
            &READING,
            &traces,
            Neighbour::Neither,
            Interface::Neither,
        );
        let dist = |a: [f32; 3], b: [f32; 3]| {
            let (x, y) = (chromaticity(a), chromaticity(b));
            ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2)).sqrt()
        };
        let agrees = |a: [f32; 3], b: [f32; 3]| dist(a, b) <= HUE.distance;
        let sum = |q: [f32; 3]| q[0] + q[1] + q[2];
        let l = [
            std::f32::consts::FRAC_1_SQRT_2,
            0.0,
            std::f32::consts::FRAC_1_SQRT_2,
        ];
        let band = |nl: f32| -> u8 {
            if nl < -0.25 {
                0
            } else if nl < 0.25 {
                1
            } else {
                2
            }
        };
        const BANDS: [&str; 4] = ["N·L<-.25", "|N·L|<.25", "N·L>.25", "leaf"];
        let median = |v: &mut Vec<f32>| -> f32 {
            v.sort_by(|a, b| a.total_cmp(b));
            v[v.len() / 2]
        };

        // --- The premise: the frame's mean luminance under the clamp. ---
        let white = 1.0 / filmic(MIN_LUMINANCE * WHITE_POINT);
        let ceiling = 1.0 - 0.02 / 0.30;
        let invert = |y: f32| -> f32 {
            // filmic(x) = y, solved for x ≥ 0 (a quadratic); capped at 64
            // where the curve saturates.
            let y = y.clamp(0.0, ceiling - 1e-4) + 0.02 / 0.30;
            let (a, b, c) = (0.15 * (1.0 - y), 0.05 - 0.5 * y, 0.004 - 0.06 * y);
            let disc = (b * b - 4.0 * a * c).max(0.0);
            ((-b + disc.sqrt()) / (2.0 * a)).clamp(0.0, 64.0)
        };
        let in_interface = |x: i32, y: i32| {
            INTERFACE
                .iter()
                .any(|r| x >= r.x0 && x < r.x1 && y >= r.y0 && y < r.y1)
        };
        let mut frame_px = Vec::new();
        for y in 0..photo.height {
            for x in 0..photo.width {
                if in_interface(x as i32, y as i32) || scene.keys.at(x, y) != 0 {
                    continue;
                }
                let i = (y * photo.width + x) * 4;
                frame_px.push([0, 1, 2].map(|c| f32::from(photo.rgba[i + c]) / 255.0));
            }
        }
        let mean_lum = |avg: f32| -> f64 {
            let w = 1.0 / filmic(avg.max(MIN_LUMINANCE) * WHITE_POINT);
            let s: f64 = frame_px
                .iter()
                .map(|d| {
                    let x = d.map(|v| invert(v / w));
                    f64::from(0.3 * x[0] + 0.59 * x[1] + 0.11 * x[2])
                })
                .sum();
            s / frame_px.len() as f64
        };
        println!(
            "premise: {} pixels no object's key holds and no interface covers; white scale at the clamp {white:.4}",
            frame_px.len()
        );
        let at_clamp = mean_lum(MIN_LUMINANCE);
        for avg in [1.4_f32, 2.0, 3.0, 4.0] {
            println!(
                "  assumed mean {avg}: the photograph inverts to a mean luminance of {:.3}",
                mean_lum(avg)
            );
        }

        // --- The populations. ---
        #[derive(Clone, Copy)]
        struct Px {
            owner: u8,
            piece: i32,
            key: u32,
            t: [f32; 3],
            alpha: f32,
            nl: f32,
            ph: [f32; 3],
        }
        let mut px: Vec<Px> = Vec::new();
        let mut leaf: Vec<([f32; 3], [f32; 3])> = Vec::new();
        let pine_figures = ["0/2", "0/8", "0/10"];
        let mut names: std::collections::BTreeMap<(u8, i32), String> = Default::default();
        for f in rows {
            let who = scene.who[f.key as usize - 1];
            if who.0 == TREE_WHO || who.3 {
                continue;
            }
            let Some(d) = scene.drawings.get(f.key as usize - 1) else {
                continue;
            };
            if world.figure_texture(d.piece).is_none() {
                continue;
            }
            let theirs = traced_theirs(
                &scene.keys,
                &p,
                &INTERFACE,
                &READING,
                &traces,
                Neighbour::Neither,
                Interface::Neither,
                f.key,
                f.window,
            );
            let ours = painted(
                &mut world,
                &scene,
                &camera,
                size,
                f.key,
                f.window,
                Colouring::Texture,
            );
            let name = format!("{}/{}", who.0, who.1);
            names
                .entry((who.0, d.piece))
                .or_default()
                .push_str(&format!(" {name}"));
            let pine = pine_figures
                .contains(&name.as_str())
                .then(|| {
                    (f.window.y0..f.window.y1)
                        .flat_map(|y| (f.window.x0..f.window.x1).map(move |x| (x, y)))
                        .map(|(x, y)| scene.nearer[y as usize * photo.width + x as usize])
                        .find(|&k| k >= first_tree)
                })
                .flatten();
            let leaf_tex =
                pine.map(|k| paint_nearer(&mut world, &scene, &camera, size, k, f.window));
            let ww = (f.window.x1 - f.window.x0) as usize;
            for y in f.window.y0..f.window.y1 {
                for x in f.window.x0..f.window.x1 {
                    let j = (y - f.window.y0) as usize * ww + (x - f.window.x0) as usize;
                    let at = y as usize * photo.width + x as usize;
                    let ph = [0, 1, 2].map(|c| f32::from(photo.rgba[at * 4 + c]));
                    let key_here = scene.keys.at(x as usize, y as usize);
                    if let Some(lt) = &leaf_tex
                        && key_here == 0
                        && let Some(t) = lt[j]
                        && sum(t.rgb) >= HUE.dark
                        && sum(ph) >= HUE.dark
                    {
                        leaf.push((t.rgb, ph));
                    }
                    if key_here != f.key || theirs[j] != Some(true) || scene.behind[at] {
                        continue;
                    }
                    let Some(t) = ours[j] else { continue };
                    let nl = d.tris.get(t.tri as usize).map_or(0.0, |tri| {
                        let (a, b, c) = (tri[0], tri[1], tri[2]);
                        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                        let n = [
                            u[1] * v[2] - u[2] * v[1],
                            u[2] * v[0] - u[0] * v[2],
                            u[0] * v[1] - u[1] * v[0],
                        ];
                        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                        if len == 0.0 {
                            0.0
                        } else {
                            (n[0] * l[0] + n[1] * l[1] + n[2] * l[2]) / len
                        }
                    });
                    px.push(Px {
                        owner: who.0,
                        piece: d.piece,
                        key: f.key,
                        t: t.rgb,
                        alpha: t.alpha,
                        nl,
                        ph,
                    });
                }
            }
        }
        let full = |q: &Px| q.alpha >= 255.0 && sum(q.t) >= HUE.dark && sum(q.ph) >= HUE.dark;
        let every = |q: &Px| {
            sum(team_blend(q.t, q.alpha, team_unit(q.owner))) >= HUE.dark && sum(q.ph) >= HUE.dark
        };

        // --- The selection, re-measured. ---
        println!(
            "colour row as it stands: {} of {}",
            base.iter().filter(|(_, c, _)| c.passes(&HUE)).count(),
            base.len()
        );
        for o in 0..=1u8 {
            let a: Vec<&Px> = px.iter().filter(|q| q.owner == o && full(q)).collect();
            let e: Vec<&Px> = px.iter().filter(|q| q.owner == o && every(q)).collect();
            println!(
                "owner {o}: alpha 255 {} (texel agreeing {}); every lit {} (masked {}, the painter's blend agreeing {})",
                a.len(),
                a.iter().filter(|q| agrees(q.t, q.ph)).count(),
                e.len(),
                e.iter().filter(|q| q.alpha < 255.0).count(),
                e.iter()
                    .filter(|q| agrees(team_blend(q.t, q.alpha, team_unit(q.owner)), q.ph))
                    .count(),
            );
        }
        let pw = |t: [f32; 3]| t.map(|v| 255.0 * (v / 255.0).powf(1.45));
        let leaf_texel = leaf.iter().filter(|(t, ph)| agrees(*t, *ph)).count();
        let leaf_power = leaf.iter().filter(|(t, ph)| agrees(pw(*t), *ph)).count();
        println!(
            "the pine's leaf by 0/2, 0/8, 0/10: {} pixels, texel agreeing {leaf_texel}, texel^1.45 {leaf_power}",
            leaf.len()
        );

        // --- Step 1: the transfer, as values. ---
        // (group, band, channel, bin) -> photograph values; group -1 the leaf.
        let mut cells: std::collections::BTreeMap<(i32, u8, usize, usize), Vec<f32>> =
            Default::default();
        for q in px.iter().filter(|q| full(q)) {
            for c in 0..3 {
                let bin = ((q.t[c] / 16.0) as usize).min(15);
                cells
                    .entry((q.piece, band(q.nl), c, bin))
                    .or_default()
                    .push(q.ph[c]);
            }
        }
        for (t, ph) in &leaf {
            for c in 0..3 {
                let bin = ((t[c] / 16.0) as usize).min(15);
                cells.entry((-1, 3, c, bin)).or_default().push(ph[c]);
            }
        }
        let mut med: std::collections::BTreeMap<(i32, u8, usize, usize), (f32, usize)> =
            Default::default();
        for (k, v) in cells.iter_mut() {
            let n = v.len();
            med.insert(*k, (median(v), n));
        }
        println!(
            "the transfer: per group, band and channel, `bin centre: median photograph (n)` where n ≥ 5"
        );
        let groups: Vec<i32> = {
            let mut g: Vec<i32> = med.keys().map(|k| k.0).collect();
            g.dedup();
            g
        };
        for &g in &groups {
            for b in 0..4u8 {
                for c in 0..3 {
                    let mut l = String::new();
                    for bin in 0..16 {
                        if let Some(&(m, n)) = med.get(&(g, b, c, bin))
                            && n >= 5
                        {
                            l.push_str(&format!(" {}:{m:.0}({n})", bin * 16 + 8));
                        }
                    }
                    if !l.is_empty() {
                        println!("  {g} {} {}:{l}", BANDS[usize::from(b)], ["r", "g", "b"][c]);
                    }
                }
            }
        }
        // One curve? A grey texel of each bin's centre, through each
        // piece's three channel medians, in chromaticity: two pieces at
        // one band part when that image differs by more than 0.05.
        let grey = |g: i32, b: u8, bin: usize| -> Option<[f32; 3]> {
            let r = med.get(&(g, b, 0, bin))?;
            let gg = med.get(&(g, b, 1, bin))?;
            let bb = med.get(&(g, b, 2, bin))?;
            (r.1 >= 20 && gg.1 >= 20 && bb.1 >= 20).then_some([r.0, gg.0, bb.0])
        };
        let pieces: Vec<i32> = groups.iter().copied().filter(|&g| g >= 0).collect();
        let mut apart_pairs = Vec::new();
        println!(
            "one curve: per band, piece pair, bins shared with 20 a channel, of them parting past 0.05"
        );
        for b in 0..3u8 {
            for (i, &pa) in pieces.iter().enumerate() {
                for &pb in &pieces[i + 1..] {
                    let mut shared = 0;
                    let mut apart = 0;
                    let mut worst = 0.0_f32;
                    for bin in 0..16 {
                        if let (Some(x), Some(y)) = (grey(pa, b, bin), grey(pb, b, bin)) {
                            shared += 1;
                            let d = dist(x, y);
                            worst = worst.max(d);
                            apart += usize::from(d > HUE.distance);
                        }
                    }
                    if shared > 0 {
                        println!(
                            "  {} {pa} / {pb}: shared {shared}, apart {apart}, worst {worst:.3}",
                            BANDS[usize::from(b)]
                        );
                        if apart * 3 > shared {
                            apart_pairs.push((b, pa, pb, shared, apart));
                        }
                    }
                }
            }
        }
        // The leaf beside the figures: at each bin both hold, the leaf's
        // channel median against the figures' pooled median at band 2.
        let mut pooled: std::collections::BTreeMap<(usize, usize), Vec<f32>> = Default::default();
        for q in px.iter().filter(|q| full(q) && band(q.nl) == 2) {
            for c in 0..3 {
                let bin = ((q.t[c] / 16.0) as usize).min(15);
                pooled.entry((c, bin)).or_default().push(q.ph[c]);
            }
        }
        let mut l = String::from(
            "the leaf against the figures (band N·L>.25), channel bin: leaf / figures —",
        );
        for c in 0..3 {
            for bin in 0..16 {
                if let (Some(&(m, n)), Some(v)) =
                    (med.get(&(-1, 3, c, bin)), pooled.get_mut(&(c, bin)))
                    && n >= 20
                    && v.len() >= 20
                {
                    l.push_str(&format!(
                        " {}{}: {m:.0}/{:.0}",
                        ["r", "g", "b"][c],
                        bin * 16 + 8,
                        median(v)
                    ));
                }
            }
        }
        println!("{l}");

        // --- The count: the candidate. ---
        #[derive(Default)]
        struct Tally {
            n: u32,
            base: u32,
            cand: u32,
            gain_pt: f64,
            gain_tt: f64,
            cand_pt: f64,
            dc: [f64; 2],
            dc_cand: [f64; 2],
        }
        let mut full_t: std::collections::BTreeMap<(u8, i32), Tally> = Default::default();
        let mut every_t: std::collections::BTreeMap<(u8, i32), Tally> = Default::default();
        let mut per_key: std::collections::BTreeMap<u32, (u32, u32, u32)> = Default::default();
        for q in &px {
            if full(q) {
                let c = lit_pipeline(q.t, q.nl);
                let s = full_t.entry((q.owner, q.piece)).or_default();
                s.n += 1;
                s.base += u32::from(agrees(q.t, q.ph));
                s.cand += u32::from(agrees(c, q.ph));
                for k in 0..3 {
                    s.gain_pt += f64::from(q.ph[k] * q.t[k]);
                    s.gain_tt += f64::from(q.t[k] * q.t[k]);
                    s.cand_pt += f64::from(c[k] * q.t[k]);
                }
                let (a, b, cc) = (chromaticity(q.ph), chromaticity(q.t), chromaticity(c));
                s.dc[0] += f64::from(a[0] - b[0]);
                s.dc[1] += f64::from(a[1] - b[1]);
                s.dc_cand[0] += f64::from(a[0] - cc[0]);
                s.dc_cand[1] += f64::from(a[1] - cc[1]);
            }
            if every(q) {
                let tb = team_blend(q.t, q.alpha, team_unit(q.owner));
                let c = lit_pipeline(tb, q.nl);
                let s = every_t.entry((q.owner, q.piece)).or_default();
                s.n += 1;
                let (ob, oc) = (agrees(tb, q.ph), agrees(c, q.ph));
                s.base += u32::from(ob);
                s.cand += u32::from(oc);
                let k = per_key.entry(q.key).or_default();
                k.0 += 1;
                k.1 += u32::from(ob);
                k.2 += u32::from(oc);
            }
        }
        let mut tot = [(0u32, 0u32, 0u32); 2];
        for (label, table, ti) in [
            ("alpha 255, the texel", &full_t, 0),
            ("every lit, the painter's blend", &every_t, 1),
        ] {
            println!("the count on {label} against the candidate:");
            for ((o, piece), s) in table.iter() {
                tot[ti].0 += s.n;
                tot[ti].1 += s.base;
                tot[ti].2 += s.cand;
                let n = f64::from(s.n.max(1));
                let gain = if s.gain_tt > 0.0 {
                    s.gain_pt / s.gain_tt
                } else {
                    0.0
                };
                let cgain = if s.gain_tt > 0.0 {
                    s.cand_pt / s.gain_tt
                } else {
                    0.0
                };
                println!(
                    "  {o}/{piece} ({}){}: n {}, base {}, candidate {} | gain photo {gain:.3} candidate {cgain:.3} | dchroma photo−base ({:+.3}, {:+.3}) photo−candidate ({:+.3}, {:+.3})",
                    world.figure_texture_name(*piece).unwrap_or("?"),
                    names.get(&(*o, *piece)).map_or("", String::as_str),
                    s.n,
                    s.base,
                    s.cand,
                    s.dc[0] / n,
                    s.dc[1] / n,
                    s.dc_cand[0] / n,
                    s.dc_cand[1] / n,
                );
            }
            println!(
                "  total n {}, base {}, candidate {}",
                tot[ti].0, tot[ti].1, tot[ti].2
            );
        }
        // --- Item 4168: the frame's mean, swept. ---
        // The packets hold every tone-map input but the mean (§3.14). The
        // kill (c), written before the count: no mean in 1.6..4 raises the
        // row past its 18 or brings tank 1/27 to passing, so clipping does
        // not part it.
        let key_1_27 = scene
            .who
            .iter()
            .position(|&(who, o, _, _)| (who, o) == (1, 27))
            .map(|i| i as u32 + 1)
            .expect("tank 1/27 is in the scene");
        for mean in [1.6_f32, 2.0, 3.0, 4.0] {
            let mut at: std::collections::BTreeMap<u32, u32> = Default::default();
            for q in &px {
                if every(q) {
                    let tb = team_blend(q.t, q.alpha, team_unit(q.owner));
                    let c = render::silhouette::lit_pipeline_at(tb, q.nl, mean);
                    *at.entry(q.key).or_default() += u32::from(agrees(c, q.ph));
                }
            }
            let mut row = 0usize;
            let mut tank = (0u32, false);
            for (f, c, _) in &base {
                let pass = match per_key.get(&f.key) {
                    Some(&(_, ab, _)) => {
                        let q = render::silhouette::Colours {
                            agree: c.agree - ab + at.get(&f.key).copied().unwrap_or(0),
                            ..*c
                        };
                        if f.key == key_1_27 {
                            tank = (q.agree, q.passes(&HUE));
                        }
                        q.passes(&HUE)
                    }
                    None => c.passes(&HUE),
                };
                row += usize::from(pass);
            }
            println!(
                "mean {mean}: colour row {row} under the candidate; tank 1/27 agreeing {}, passing {}",
                tank.0, tank.1
            );
            assert!(row <= 18, "the mean {mean} raises the row to {row}");
            assert!(!tank.1, "the mean {mean} brings 1/27 to passing");
        }

        let mut passing = (0usize, 0usize);
        let mut crossed = Vec::new();
        for (f, c, w) in &base {
            let Some(&(n, ab, ac)) = per_key.get(&f.key) else {
                passing.0 += usize::from(c.passes(&HUE));
                passing.1 += usize::from(c.passes(&HUE));
                continue;
            };
            let q = render::silhouette::Colours {
                agree: c.agree - ab + ac,
                ..*c
            };
            let (a, b) = (c.passes(&HUE), q.passes(&HUE));
            passing.0 += usize::from(a);
            passing.1 += usize::from(b);
            println!(
                "  {}/{}: lit {} ({n} not behind), agreeing {} → {} {}",
                w.0,
                w.1,
                c.lit,
                c.agree,
                q.agree,
                match (a, b) {
                    (true, false) => "DOWN",
                    (false, true) => "UP",
                    (true, true) => "in",
                    _ => "out",
                }
            );
            if a != b {
                crossed.push((w.0, w.1, a));
            }
        }
        println!(
            "colour row: {} as painted, {} under the candidate; crossings {crossed:?}",
            passing.0, passing.1
        );
        println!("kill (a) pairs: {apart_pairs:?}; premise mean at the clamp {at_clamp:.3}");
        // The kill, as assertions (item 4114). The selection re-measured:
        // the row, 4107's alpha-255 counts, 4112's every-lit counts and its
        // painter's 8,119, 4106's leaf. Then (a): the medium tank 196 and
        // the general 1008 part on every bin they share at N·L > .25.
        // Then (b): the candidate agrees on fewer pixels than the texel and
        // than the painter's blend, and takes the row down. And the
        // premise: under the clamp the photograph inverts to a mean over it,
        // so the one frame scalar is not the clamp (chromatically inert:
        // a white scale multiplies every channel alike).
        assert_eq!(passing.0, render::silhouette::RUN1559B_3279_COLOUR.0);
        let count =
            |o: u8, f: &dyn Fn(&Px) -> bool| px.iter().filter(|q| q.owner == o && f(q)).count();
        assert_eq!((count(0, &full), count(1, &full)), (564, 13_335));
        assert_eq!((count(0, &every), count(1, &every)), (1_150, 15_482));
        assert_eq!(tot[1].1, 8_119, "the painter's blend on every lit pixel");
        assert_eq!((leaf.len(), leaf_texel, leaf_power), (3_136, 514, 1_946));
        assert!(
            apart_pairs
                .iter()
                .any(|&(b, x, y, n, a)| (b, x, y) == (2, 196, 1008) && n == a && n >= 5),
            "the tank and the general on one curve"
        );
        assert!(
            tot[0].2 < tot[0].1 && tot[1].2 < tot[1].1,
            "the candidate beats the texel"
        );
        assert!(passing.1 < passing.0, "the candidate takes the row down");
        assert!(
            at_clamp > f64::from(MIN_LUMINANCE),
            "the clamp is self-consistent"
        );
    }

    /// **Item 4117: the medium tank's texel — which file, and where it
    /// lands.** On the alpha-255 population (4107's selection), per piece:
    /// n, the Pearson correlation of the photograph's luminance with our
    /// texel's (the luminance pass's weights 0.3, 0.59, 0.11), and the
    /// reader's agreeing count. Then the same under every texture the
    /// install holds on the piece's stem, sampled through the same
    /// texcoords, and under the draw moved by −4..4 pixels each way (and,
    /// for the tank, node by node). The kill (a), written before the count:
    /// no file and no move raises the tank's correlation by 0.10 and its
    /// count by a tenth. Nothing is drawn.
    #[test]
    fn the_tanks_texel() {
        use render::silhouette::{HUE, chromaticity, traced_theirs};
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
        world.run_to(PHOTO_FRAME);
        world.blend = crate::world::Blend::Last;
        world.run_to(PHOTO_FRAME + 1);
        let size = (photo.width, photo.height);
        let (w, h) = (photo.width as i32, photo.height as i32);
        let base = colour_rows(&mut world, &camera, &photo, &TRACED, Colouring::Silhouette);
        let scene = frame_scene(&mut world, &camera, size);
        let p = Photo {
            width: photo.width,
            height: photo.height,
            rgba: &photo.rgba,
        };
        let traces: Vec<Trace<'_>> = TRACED
            .iter()
            .map(|&(name, poly)| Trace {
                key: scene
                    .who
                    .iter()
                    .position(|&(who, o, _, _)| format!("{who}/{o}") == name)
                    .map_or(u32::MAX, |i| i as u32 + 1),
                poly,
            })
            .collect();
        let rows = footprints_traced(
            &scene.keys,
            &p,
            &INTERFACE,
            &READING,
            &traces,
            Neighbour::Neither,
            Interface::Neither,
        );
        let agrees = |a: [f32; 3], b: [f32; 3]| {
            let (x, y) = (chromaticity(a), chromaticity(b));
            ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2)).sqrt() <= HUE.distance
        };
        let sum = |q: [f32; 3]| q[0] + q[1] + q[2];
        let lum = |q: [f32; 3]| 0.3 * q[0] + 0.59 * q[1] + 0.11 * q[2];
        const TANK: i32 = 196;
        const CONTROLS: [i32; 2] = [60, 1008];
        const R: i32 = 4;

        // --- The selection, re-measured: the colour row object by object. ---
        let piece_of = |key: u32| scene.drawings.get(key as usize - 1).map_or(-1, |d| d.piece);
        let mut row = 0;
        for (f, c, who) in &base {
            row += usize::from(c.passes(&HUE));
            println!(
                "row {}/{}: piece {}, lit {}, agreeing {} ({:.3}), {}",
                who.0,
                who.1,
                piece_of(f.key),
                c.lit,
                c.agree,
                c.agree as f32 / c.lit.max(1) as f32,
                if c.passes(&HUE) { "passes" } else { "fails" }
            );
        }
        println!("colour row: {row} of {}", base.len());

        // --- Each figure row, painted once over its window grown by R. ---
        struct Fig {
            owner: u8,
            piece: i32,
            key: u32,
            win: Rect,
            grown: Rect,
            theirs: Vec<Option<bool>>,
            ours: Vec<Option<render::silhouette::Texel>>,
        }
        let mut figs: Vec<Fig> = Vec::new();
        for f in rows {
            let who = scene.who[f.key as usize - 1];
            if who.0 == TREE_WHO || who.3 {
                continue;
            }
            let Some(d) = scene.drawings.get(f.key as usize - 1) else {
                continue;
            };
            if world.figure_texture(d.piece).is_none() {
                continue;
            }
            let theirs = traced_theirs(
                &scene.keys,
                &p,
                &INTERFACE,
                &READING,
                &traces,
                Neighbour::Neither,
                Interface::Neither,
                f.key,
                f.window,
            );
            let grown = Rect {
                x0: (f.window.x0 - R).max(0),
                y0: (f.window.y0 - R).max(0),
                x1: (f.window.x1 + R).min(w),
                y1: (f.window.y1 + R).min(h),
            };
            let ours = painted(
                &mut world,
                &scene,
                &camera,
                size,
                f.key,
                grown,
                Colouring::Texture,
            );
            figs.push(Fig {
                owner: who.0,
                piece: d.piece,
                key: f.key,
                win: f.window,
                grown,
                theirs,
                ours,
            });
        }
        // One pixel: (owner, piece, texel, photograph, triangle).
        type Pix = (u8, i32, [f32; 3], [f32; 3], u32);
        // The alpha-255 population with the draw moved by (dx, dy): the
        // photograph at (x, y) against our pixel at (x − dx, y − dy).
        let sample = |fig: &Fig, ours: &[Option<render::silhouette::Texel>], dx: i32, dy: i32| {
            let mut out: Vec<Pix> = Vec::new();
            let ww = (fig.win.x1 - fig.win.x0) as usize;
            let gw = (fig.grown.x1 - fig.grown.x0) as usize;
            for y in fig.win.y0..fig.win.y1 {
                for x in fig.win.x0..fig.win.x1 {
                    let j = (y - fig.win.y0) as usize * ww + (x - fig.win.x0) as usize;
                    if fig.theirs[j] != Some(true) {
                        continue;
                    }
                    let (ox, oy) = (x - dx, y - dy);
                    if ox < fig.grown.x0
                        || ox >= fig.grown.x1
                        || oy < fig.grown.y0
                        || oy >= fig.grown.y1
                    {
                        continue;
                    }
                    let at = oy as usize * photo.width + ox as usize;
                    if scene.keys.at(ox as usize, oy as usize) != fig.key || scene.behind[at] {
                        continue;
                    }
                    let g = (oy - fig.grown.y0) as usize * gw + (ox - fig.grown.x0) as usize;
                    let Some(t) = ours[g] else { continue };
                    let pa = (y as usize * photo.width + x as usize) * 4;
                    let ph = [0, 1, 2].map(|c| f32::from(photo.rgba[pa + c]));
                    if t.alpha < 255.0 || sum(t.rgb) < HUE.dark || sum(ph) < HUE.dark {
                        continue;
                    }
                    out.push((fig.owner, fig.piece, t.rgb, ph, t.tri));
                }
            }
            out
        };
        // (n, Pearson, agreeing) over a set of pixels.
        let stats = |v: &[&Pix]| -> (usize, f64, usize) {
            let n = v.len();
            if n < 2 {
                return (n, 0.0, v.iter().filter(|q| agrees(q.2, q.3)).count());
            }
            let (mut sx, mut sy, mut sxx, mut syy, mut sxy) = (0f64, 0f64, 0f64, 0f64, 0f64);
            for q in v {
                let (a, b) = (f64::from(lum(q.3)), f64::from(lum(q.2)));
                sx += a;
                sy += b;
                sxx += a * a;
                syy += b * b;
                sxy += a * b;
            }
            let nf = n as f64;
            let cov = sxy - sx * sy / nf;
            let den = ((sxx - sx * sx / nf) * (syy - sy * sy / nf)).sqrt();
            let r = if den > 0.0 { cov / den } else { 0.0 };
            (n, r, v.iter().filter(|q| agrees(q.2, q.3)).count())
        };
        let all_at = |figs: &[Fig], dx: i32, dy: i32| -> Vec<Pix> {
            figs.iter()
                .flat_map(|f| sample(f, &f.ours, dx, dy))
                .collect()
        };

        // --- Step 1a: the values, per piece, as drawn. ---
        let zero = all_at(&figs, 0, 0);
        let owner_n = |o: u8| zero.iter().filter(|q| q.0 == o).count();
        let owner_a = |o: u8| zero.iter().filter(|q| q.0 == o && agrees(q.2, q.3)).count();
        println!(
            "alpha 255: owner 0 {} (texel agreeing {}), owner 1 {} ({})",
            owner_n(0),
            owner_a(0),
            owner_n(1),
            owner_a(1)
        );
        let pieces: std::collections::BTreeSet<i32> = zero.iter().map(|q| q.1).collect();
        let mut by_piece: std::collections::BTreeMap<i32, (usize, f64, usize)> = Default::default();
        for &pc in &pieces {
            let v: Vec<&Pix> = zero.iter().filter(|q| q.1 == pc).collect();
            let s = stats(&v);
            by_piece.insert(pc, s);
            println!(
                "piece {pc} ({}): n {}, r {:.3}, agreeing {}",
                world.figure_texture_name(pc).unwrap_or("?"),
                s.0,
                s.1,
                s.2
            );
        }
        let objects: Vec<String> = figs
            .iter()
            .filter(|f| f.piece == TANK)
            .map(|f| {
                let who = scene.who[f.key as usize - 1];
                format!("{}/{}", who.0, who.1)
            })
            .collect();
        println!("the tank's objects: {objects:?}");

        // --- Step 1b: the file. Every texture on the piece's stem. ---
        let art = install.join("art");
        let names: Vec<String> = std::fs::read_dir(&art)
            .map(|d| {
                d.filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        let mut file_table: Vec<(i32, String, usize, f64, usize)> = Vec::new();
        for pc in std::iter::once(TANK).chain(CONTROLS) {
            let own = world.figure_texture_name(pc).unwrap_or("").to_string();
            let file = own
                .rsplit(['\\', '/'])
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            let stem = file.trim_end_matches(".tga").to_string();
            let mut alts: Vec<String> = names
                .iter()
                .filter(|n| {
                    let l = n.to_ascii_lowercase();
                    l.ends_with(".tga") && l.starts_with(&stem)
                })
                .cloned()
                .collect();
            alts.sort();
            println!("piece {pc}: own {own}; on its stem {alts:?}");
            for alt in &alts {
                let xml = format!(".\\art\\{alt}");
                if !world.swap_figure_texture(pc, Some(&xml)) {
                    println!("  {alt}: does not decode");
                    continue;
                }
                let mut v: Vec<Pix> = Vec::new();
                for f in figs.iter().filter(|f| f.piece == pc) {
                    let o = painted(
                        &mut world,
                        &scene,
                        &camera,
                        size,
                        f.key,
                        f.grown,
                        Colouring::Texture,
                    );
                    v.extend(sample(f, &o, 0, 0));
                }
                let s = stats(&v.iter().collect::<Vec<_>>());
                println!("  {alt}: n {}, r {:.3}, agreeing {}", s.0, s.1, s.2);
                file_table.push((pc, alt.to_ascii_lowercase(), s.0, s.1, s.2));
            }
            world.swap_figure_texture(pc, None);
        }

        // --- Step 1c: the place. 81 cells, the zero, the best, its ring. ---
        let mut best_cells: std::collections::BTreeMap<String, (i32, i32, usize, f64, usize)> =
            Default::default();
        let grid = |label: &str,
                    pick: &dyn Fn(&Pix) -> bool,
                    best_cells: &mut std::collections::BTreeMap<
            String,
            (i32, i32, usize, f64, usize),
        >| {
            let mut cells = std::collections::BTreeMap::new();
            for dy in -R..=R {
                for dx in -R..=R {
                    let v = all_at(&figs, dx, dy);
                    let sel: Vec<&Pix> = v.iter().filter(|q| pick(q)).collect();
                    cells.insert((dx, dy), stats(&sel));
                }
            }
            let z = cells[&(0, 0)];
            let (&(bx, by), &b) = cells
                .iter()
                .max_by(|a, b| a.1.1.total_cmp(&b.1.1))
                .expect("cells");
            let (&(cx, cy), &c) = cells.iter().max_by_key(|a| a.1.2).expect("cells");
            let ring: Vec<f64> = (-1..=1)
                .flat_map(|j| (-1..=1).map(move |i| (bx + i, by + j)))
                .filter(|&k| k != (bx, by))
                .filter_map(|k| cells.get(&k).map(|s| s.1))
                .collect();
            let ring_mean = ring.iter().sum::<f64>() / ring.len().max(1) as f64;
            println!(
                "{label}: zero n {} r {:.3} agreeing {} | best r at ({bx:+},{by:+}) n {} r {:.3} agreeing {}, ring mean r {ring_mean:.3} | best count at ({cx:+},{cy:+}) n {} r {:.3} agreeing {}",
                z.0, z.1, z.2, b.0, b.1, b.2, c.0, c.1, c.2
            );
            best_cells.insert(label.to_string(), (bx, by, b.0, b.1, b.2));
            best_cells.insert(format!("{label} count"), (cx, cy, c.0, c.1, c.2));
        };
        for pc in std::iter::once(TANK).chain(CONTROLS) {
            grid(
                &format!("piece {pc}"),
                &|q: &Pix| q.1 == pc,
                &mut best_cells,
            );
        }
        // The tank node by node: a face's node is the one whose vertex run
        // holds its first corner.
        if let Some(mesh) = world.mesh(TANK, false) {
            let node_of = |tri: u32| -> Option<usize> {
                let f = mesh.faces.get(tri as usize)?;
                let v = u32::from(f[0]);
                mesh.model
                    .nodes
                    .iter()
                    .position(|n| v >= n.start && v < n.start + n.num)
            };
            let mut count: std::collections::BTreeMap<usize, usize> = Default::default();
            for q in zero.iter().filter(|q| q.1 == TANK) {
                if let Some(k) = node_of(q.4) {
                    *count.entry(k).or_default() += 1;
                }
            }
            for (k, n) in &mesh.model.nodes.iter().enumerate().collect::<Vec<_>>() {
                println!(
                    "  node {k} {:?} parent {:?} verts {}..{}: {} pixels",
                    n.name,
                    n.parent,
                    n.start,
                    n.start + n.num,
                    count.get(k).copied().unwrap_or(0)
                );
            }
            for (&k, &n) in &count {
                if n >= 200 {
                    let name = mesh.model.nodes[k].name.clone();
                    grid(
                        &format!("piece {TANK} node {name}"),
                        &|q: &Pix| q.1 == TANK && node_of(q.4) == Some(k),
                        &mut best_cells,
                    );
                }
            }
        }

        // --- The kill (a), as it falls. ---
        let (tn, tr, ta) = by_piece[&TANK];
        let bar = |r: f64, a: usize| r >= tr + 0.10 && a as f64 >= ta as f64 * 1.1;
        let file_clears: Vec<_> = file_table
            .iter()
            .filter(|e| e.0 == TANK && bar(e.3, e.4))
            .collect();
        let place_clears: Vec<_> = best_cells
            .iter()
            .filter(|(k, v)| k.starts_with("piece 196") && !k.contains("node") && bar(v.3, v.4))
            .collect();
        println!(
            "kill (a): the tank n {tn}, r {tr:.3}, agreeing {ta}; the bar r ≥ {:.3} and ≥ {:.0}; files clearing {file_clears:?}; moves clearing {place_clears:?}",
            tr + 0.10,
            ta as f64 * 1.1
        );
        assert_eq!(row, render::silhouette::RUN1559B_3279_COLOUR.0);
        assert_eq!((owner_n(0), owner_n(1)), (564, 13_335));
        assert_eq!((owner_a(0), owner_a(1)), (262, 6_788));
        assert_eq!(ta, 4_486, "the tank's texel count, 4114's");
        assert!(file_clears.is_empty(), "a file clears the bar");
        assert!(place_clears.is_empty(), "a move clears the bar");
    }

    /// **Item 4122: the model's own normals under the lit pixel.** Step 1,
    /// the normals themselves: per piece on the colour row, the vertices
    /// and stored `id 3` normals, their length and fourth word, and the
    /// angle between each face's geometric normal and the mean of its
    /// corners' stored normals, both in world space (the stored carried by
    /// [`crate::world::Drawing::normals`], the corner's node, pose and
    /// placement, nothing else) — the tank node by node. Step 2, against
    /// the photograph on 4107's alpha-255 population: q, the photograph's
    /// luminance over our stage colour's (0.3, 0.59, 0.11), correlated with
    /// N·L by the face and by the stored normals interpolated by the
    /// texel's perspective-correct weights and renormalised; then 4114's
    /// candidate (`render::silhouette::lit_pipeline`, untouched) re-counted
    /// with the stored N·L, and the colour row under it. The kill (a),
    /// written before the count: the stored candidate agrees on no more
    /// than the texel's 7,050 of 13,899. Nothing is drawn.
    #[test]
    #[allow(clippy::type_complexity)]
    fn the_stored_normals() {
        use render::silhouette::{
            HUE, chromaticity, lit_pipeline, team_blend, team_unit, traced_theirs,
        };
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
        world.run_to(PHOTO_FRAME);
        world.blend = crate::world::Blend::Last;
        world.run_to(PHOTO_FRAME + 1);
        let size = (photo.width, photo.height);
        let base = colour_rows(&mut world, &camera, &photo, &TRACED, Colouring::Silhouette);
        let scene = frame_scene(&mut world, &camera, size);
        let p = Photo {
            width: photo.width,
            height: photo.height,
            rgba: &photo.rgba,
        };
        let traces: Vec<Trace<'_>> = TRACED
            .iter()
            .map(|&(name, poly)| Trace {
                key: scene
                    .who
                    .iter()
                    .position(|&(who, o, _, _)| format!("{who}/{o}") == name)
                    .map_or(u32::MAX, |i| i as u32 + 1),
                poly,
            })
            .collect();
        let rows = footprints_traced(
            &scene.keys,
            &p,
            &INTERFACE,
            &READING,
            &traces,
            Neighbour::Neither,
            Interface::Neither,
        );
        let agrees = |a: [f32; 3], b: [f32; 3]| {
            let (x, y) = (chromaticity(a), chromaticity(b));
            ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2)).sqrt() <= HUE.distance
        };
        let sum = |q: [f32; 3]| q[0] + q[1] + q[2];
        let lum = |q: [f32; 3]| 0.3 * q[0] + 0.59 * q[1] + 0.11 * q[2];
        let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let unit = |a: [f32; 3]| {
            let l = dot(a, a).sqrt();
            if l > 0.0 { a.map(|x| x / l) } else { [0.0; 3] }
        };
        let face = |tri: &[[f32; 3]; 3]| {
            let (a, b, c) = (tri[0], tri[1], tri[2]);
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            unit([
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ])
        };
        // 4114's light, `L = −cur_dir`.
        let l = [
            std::f32::consts::FRAC_1_SQRT_2,
            0.0,
            std::f32::consts::FRAC_1_SQRT_2,
        ];
        const TANK: i32 = 196;
        let pearson = |v: &[(f32, f32)]| -> f32 {
            let n = v.len() as f64;
            if n < 2.0 {
                return f32::NAN;
            }
            let (mx, my) = v.iter().fold((0.0, 0.0), |(a, b), &(x, y)| {
                (a + f64::from(x), b + f64::from(y))
            });
            let (mx, my) = (mx / n, my / n);
            let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
            for &(x, y) in v {
                let (dx, dy) = (f64::from(x) - mx, f64::from(y) - my);
                sxy += dx * dy;
                sxx += dx * dx;
                syy += dy * dy;
            }
            (sxy / (sxx * syy).sqrt()) as f32
        };
        let tank_mesh = world.mesh(TANK, false).expect("the tank's mesh");
        let node_of = |tri: u32| -> String {
            tank_mesh
                .faces
                .get(tri as usize)
                .and_then(|f| {
                    let v = u32::from(f[0]);
                    tank_mesh
                        .model
                        .nodes
                        .iter()
                        .find(|n| v >= n.start && v < n.start + n.num)
                })
                .map_or_else(|| "?".to_string(), |n| n.name.clone())
        };

        // --- The population: 4107's alpha-255 pixels and 4112's every lit. ---
        #[derive(Clone)]
        struct Px {
            owner: u8,
            piece: i32,
            key: u32,
            tri: u32,
            t: [f32; 3],
            alpha: f32,
            face: f32,
            stored: f32,
            ph: [f32; 3],
        }
        let mut px: Vec<Px> = Vec::new();
        let mut on_row: std::collections::BTreeSet<i32> = Default::default();
        for f in rows {
            let who = scene.who[f.key as usize - 1];
            if who.0 == TREE_WHO || who.3 {
                continue;
            }
            let Some(d) = scene.drawings.get(f.key as usize - 1) else {
                continue;
            };
            if world.figure_texture(d.piece).is_none() {
                continue;
            }
            on_row.insert(d.piece);
            let theirs = traced_theirs(
                &scene.keys,
                &p,
                &INTERFACE,
                &READING,
                &traces,
                Neighbour::Neither,
                Interface::Neither,
                f.key,
                f.window,
            );
            let ours = painted(
                &mut world,
                &scene,
                &camera,
                size,
                f.key,
                f.window,
                Colouring::Texture,
            );
            let ww = (f.window.x1 - f.window.x0) as usize;
            for y in f.window.y0..f.window.y1 {
                for x in f.window.x0..f.window.x1 {
                    let j = (y - f.window.y0) as usize * ww + (x - f.window.x0) as usize;
                    let at = y as usize * photo.width + x as usize;
                    let key_here = scene.keys.at(x as usize, y as usize);
                    if key_here != f.key || theirs[j] != Some(true) || scene.behind[at] {
                        continue;
                    }
                    let Some(t) = ours[j] else { continue };
                    let ph = [0, 1, 2].map(|c| f32::from(photo.rgba[at * 4 + c]));
                    let fnl = d
                        .tris
                        .get(t.tri as usize)
                        .map_or(0.0, |tri| dot(face(tri), l));
                    let snl = d.normals.get(t.tri as usize).map_or(0.0, |n| {
                        let w = t.weights;
                        dot(
                            unit(
                                [0, 1, 2].map(|k| w[0] * n[0][k] + w[1] * n[1][k] + w[2] * n[2][k]),
                            ),
                            l,
                        )
                    });
                    px.push(Px {
                        owner: who.0,
                        piece: d.piece,
                        key: f.key,
                        tri: t.tri,
                        t: t.rgb,
                        alpha: t.alpha,
                        face: fnl,
                        stored: snl,
                        ph,
                    });
                }
            }
        }
        let full = |q: &Px| q.alpha >= 255.0 && sum(q.t) >= HUE.dark && sum(q.ph) >= HUE.dark;
        let stage = |q: &Px| team_blend(q.t, q.alpha, team_unit(q.owner));
        let every = |q: &Px| sum(stage(q)) >= HUE.dark && sum(q.ph) >= HUE.dark;

        // --- 0. The selection, re-measured. ---
        let row_now = base.iter().filter(|(_, c, _)| c.passes(&HUE)).count();
        println!("colour row as it stands: {row_now} of {}", base.len());
        let mut sel = [(0usize, 0usize); 2];
        for o in 0..=1u8 {
            let a: Vec<&Px> = px.iter().filter(|q| q.owner == o && full(q)).collect();
            sel[usize::from(o)] = (a.len(), a.iter().filter(|q| agrees(q.t, q.ph)).count());
            println!(
                "owner {o}: alpha 255 {} (texel agreeing {})",
                sel[usize::from(o)].0,
                sel[usize::from(o)].1
            );
        }
        let texel_total = sel[0].1 + sel[1].1;
        let full_total = sel[0].0 + sel[1].0;
        let face_cand = px
            .iter()
            .filter(|q| full(q) && agrees(lit_pipeline(q.t, q.face), q.ph))
            .count();
        let every_n = px.iter().filter(|q| every(q)).count();
        let every_base = px
            .iter()
            .filter(|q| every(q) && agrees(stage(q), q.ph))
            .count();
        let every_face = px
            .iter()
            .filter(|q| every(q) && agrees(lit_pipeline(stage(q), q.face), q.ph))
            .count();
        println!(
            "4114's face candidate: alpha 255 {face_cand} of {full_total} (texel {texel_total}); every lit {every_face} of {every_n} (painter {every_base})"
        );

        // --- 1. The normals themselves. ---
        println!("step 1: per piece, the stored normals and the angle to the faces' own");
        let deg = |c: f32| c.clamp(-1.0, 1.0).acos().to_degrees();
        let pct = |v: &mut Vec<f32>, p: usize| -> f32 {
            if v.is_empty() {
                return f32::NAN;
            }
            v.sort_by(|a, b| a.total_cmp(b));
            v[(v.len() * p / 100).min(v.len() - 1)]
        };
        let mut angle_rows: Vec<(String, f32, f32, f32, f32)> = Vec::new();
        let mut unit_counts: Vec<(i32, usize, usize)> = Vec::new();
        for &piece in &on_row {
            let Some(mesh) = world.mesh(piece, false) else {
                continue;
            };
            let m = &mesh.model;
            let lens: Vec<f32> = m
                .normals
                .iter()
                .map(|n| (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt())
                .collect();
            let finite = lens.iter().filter(|x| x.is_finite()).count();
            let near_unit = lens.iter().filter(|x| (**x - 1.0).abs() < 0.01).count();
            println!(
                "  piece {piece} ({}): vertices {}, normals {}, finite {finite}, unit within 0.01 {near_unit}",
                world.figure_texture_name(piece).unwrap_or("?"),
                m.verts.len(),
                m.normals.len(),
            );
            unit_counts.push((piece, m.normals.len(), near_unit));
            // The faces of the piece's first drawing on the row, in world space.
            let Some(d) = scene.drawings.iter().find(|d| d.piece == piece) else {
                continue;
            };
            let mut groups: std::collections::BTreeMap<String, Vec<f32>> = Default::default();
            for (ti, (tri, ns)) in d.tris.iter().zip(&d.normals).enumerate() {
                let fnorm = face(tri);
                let mean = unit([0, 1, 2].map(|k| ns[0][k] + ns[1][k] + ns[2][k]));
                if dot(fnorm, fnorm) == 0.0 || dot(mean, mean) == 0.0 {
                    continue;
                }
                let a = deg(dot(fnorm, mean));
                groups.entry(format!("{piece}")).or_default().push(a);
                if piece == TANK {
                    groups
                        .entry(format!("{piece} {}", node_of(ti as u32)))
                        .or_default()
                        .push(a);
                }
            }
            for (g, mut v) in groups {
                let n = v.len();
                let past30 = v.iter().filter(|a| **a > 30.0).count() as f32 / n as f32;
                let past90 = v.iter().filter(|a| **a > 90.0).count() as f32 / n as f32;
                let (med, p90) = (pct(&mut v, 50), pct(&mut v, 90));
                println!(
                    "    {g}: faces {n}, angle median {med:.1}°, 90th {p90:.1}°, past 30° {:.3}, past 90° {:.3}",
                    past30, past90
                );
                angle_rows.push((g, med, p90, past30, past90));
            }
        }

        // --- 2. Against the photograph. ---
        println!("step 2: q (photograph luminance over the stage colour's) against N·L");
        let mut groups: std::collections::BTreeMap<String, Vec<&Px>> = Default::default();
        for q in px.iter().filter(|q| full(q)) {
            groups.entry(format!("{}", q.piece)).or_default().push(q);
            if q.piece == TANK {
                groups
                    .entry(format!("{TANK} {}", node_of(q.tri)))
                    .or_default()
                    .push(q);
            }
        }
        let mut corr: std::collections::BTreeMap<String, (usize, f32, f32, f32)> =
            Default::default();
        for (g, v) in &groups {
            let qs = |nl: &dyn Fn(&Px) -> f32| -> Vec<(f32, f32)> {
                v.iter()
                    .filter(|q| lum(stage(q)) > 0.0)
                    .map(|q| (lum(q.ph) / lum(stage(q)), nl(q)))
                    .collect()
            };
            let rf = pearson(&qs(&|q| q.face));
            let rs = pearson(&qs(&|q| q.stored));
            let rl = pearson(&v.iter().map(|q| (lum(q.ph), lum(q.t))).collect::<Vec<_>>());
            println!(
                "  {g}: n {}, r(q, face N·L) {rf:.3}, r(q, stored N·L) {rs:.3} | r(photo lum, texel lum) {rl:.3}",
                v.len()
            );
            corr.insert(g.clone(), (v.len(), rf, rs, rl));
        }

        // The candidate with the stored normals.
        #[derive(Default)]
        struct Tally {
            n: u32,
            texel: u32,
            face: u32,
            stored: u32,
        }
        let mut by_piece: std::collections::BTreeMap<i32, Tally> = Default::default();
        let mut per_key: std::collections::BTreeMap<u32, (u32, u32, u32)> = Default::default();
        let mut every_stored = 0usize;
        for q in &px {
            if full(q) {
                let s = by_piece.entry(q.piece).or_default();
                s.n += 1;
                s.texel += u32::from(agrees(q.t, q.ph));
                s.face += u32::from(agrees(lit_pipeline(q.t, q.face), q.ph));
                s.stored += u32::from(agrees(lit_pipeline(q.t, q.stored), q.ph));
            }
            if every(q) {
                let tb = stage(q);
                let (ob, os) = (agrees(tb, q.ph), agrees(lit_pipeline(tb, q.stored), q.ph));
                every_stored += usize::from(os);
                let k = per_key.entry(q.key).or_default();
                k.0 += 1;
                k.1 += u32::from(ob);
                k.2 += u32::from(os);
            }
        }
        let mut stored_total = 0u32;
        println!("the count on alpha 255, by piece: texel, face candidate, stored candidate");
        for (piece, s) in &by_piece {
            stored_total += s.stored;
            println!(
                "  {piece}: n {}, texel {}, face {}, stored {}",
                s.n, s.texel, s.face, s.stored
            );
        }
        println!(
            "  total: n {full_total}, texel {texel_total}, face {face_cand}, stored {stored_total}"
        );
        println!(
            "every lit: n {every_n}, painter {every_base}, face {every_face}, stored {every_stored}"
        );
        let mut passing = (0usize, 0usize);
        let mut crossed = Vec::new();
        for (f, c, w) in &base {
            let Some(&(n, ab, ac)) = per_key.get(&f.key) else {
                passing.0 += usize::from(c.passes(&HUE));
                passing.1 += usize::from(c.passes(&HUE));
                continue;
            };
            let q = render::silhouette::Colours {
                agree: c.agree - ab + ac,
                ..*c
            };
            let (a, b) = (c.passes(&HUE), q.passes(&HUE));
            passing.0 += usize::from(a);
            passing.1 += usize::from(b);
            println!(
                "  {}/{}: lit {} ({n} not behind), agreeing {} → {} {}",
                w.0,
                w.1,
                c.lit,
                c.agree,
                q.agree,
                match (a, b) {
                    (true, false) => "DOWN",
                    (false, true) => "UP",
                    (true, true) => "in",
                    _ => "out",
                }
            );
            if a != b {
                crossed.push((w.0, w.1, a));
            }
        }
        println!(
            "colour row: {} as painted, {} under the stored candidate; crossings {crossed:?}",
            passing.0, passing.1
        );
        let root = corr.get(&format!("{TANK} root")).copied();
        println!("the tank's root: {root:?}");

        // The selection, as asserted; then the kill (a).
        assert_eq!(row_now, render::silhouette::RUN1559B_3279_COLOUR.0);
        assert_eq!((sel[0], sel[1]), ((564, 262), (13_335, 6_788)));
        assert_eq!((face_cand, every_base, every_face), (6_452, 8_119, 7_353));
        assert_eq!(by_piece.get(&TANK).map(|s| s.texel), Some(4_486));
        // Step 1: read packed, as `load_norms` copies them, every one of the
        // tank's normals is unit and they sit near its faces; the riflemen's
        // are smoothed past them.
        assert!(unit_counts.contains(&(TANK, 401, 401)));
        let median_of = |g: &str| angle_rows.iter().find(|r| r.0 == g).map(|r| r.1);
        assert!(median_of("196").is_some_and(|m| m < 10.0), "the tank's");
        assert!(median_of("60").is_some_and(|m| m > 15.0), "the riflemen's");
        // Step 2: the controls' shading follows the stored normals better
        // than the faces; the tank's follows neither.
        let r = |g: &str| corr.get(g).copied().expect(g);
        assert!(r("1008").2 > r("1008").1 && r("60").2 > r("60").1);
        assert!(
            r("196").2.abs() < 0.10,
            "the tank's q against its stored N·L"
        );
        // The tank's root follows the light under the stored normals
        // (r over 0.20): not a row of its own.
        assert!(r("196 root").2 > 0.20);
        // The count, and the kill (a): the stored candidate agrees on fewer
        // pixels than the texel, and the row goes down.
        assert_eq!((stored_total, every_stored), (6_565, 7_475));
        assert_eq!(passing, (18, 16));
        assert!(
            (stored_total as usize) <= texel_total,
            "the stored candidate beats the texel: kill (a) does not hold"
        );
    }

    /// The card trace's path: `RON_CARD_TRACE`, else run1566b's
    /// (`docs/RUNS.md`, item 4165).
    fn card_trace_path() -> PathBuf {
        std::env::var_os("RON_CARD_TRACE")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_default();
                PathBuf::from(home).join("ron-golden/run1566b/map-14/rontrace.log")
            })
    }

    /// **The device's state at each scored object** (item 4165;
    /// `docs/RENDERER.md` §3.14): run1566b's card trace
    /// (`tools/trace/card.h`, `rondata::trace::card`) — run1559b's take with
    /// the probe on `D3D11GLSys::render` — holds the pass drawn after frame
    /// 3279, the photograph's. Each of [`RUN1559B_3279_TRACED`]'s 33 objects
    /// is matched to that pass's draws: a figure to the lit draw whose
    /// `gWorld` translation is nearest its own point (`Placed` x, y, z, in
    /// the world integers both sides use) and within [`CARD_MATCH`], each
    /// draw taken once; a building to the sprite draw whose texture is its
    /// own art file's, `_NONQUAD`. A match counts when its state is whole:
    /// options, both stages' operations, the texture's name, the texture
    /// factor, the material, the light and the scene's ambient. The rule was
    /// written before the count. Then every value the colour row reads
    /// (4102, 4103, 4114) is asserted against the original's own state.
    #[test]
    fn the_device_state_at_each_figure() {
        use render::silhouette::{
            SCENE_AMBIENT, SILHOUETTE_ALPHA, UNIT_AMBIENT, UNIT_AMBIENT_I, UNIT_DIFFUSE, team_unit,
        };
        use rondata::trace::card::{
            self, TA_CURRENT, TA_DIFFUSE, TA_TEXTURE, TA_TFACTOR, TOP_BLENDTEXTUREALPHA,
            TOP_MODULATE, TOP_SELECTARG2,
        };
        let tpath = card_trace_path();
        let Ok(tbytes) = std::fs::read(&tpath) else {
            println!("skipping: no card trace at {}", tpath.display());
            return;
        };
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
        world.run_to(PHOTO_FRAME);
        world.blend = crate::world::Blend::Last;
        world.run_to(PHOTO_FRAME + 1);
        let size = (photo.width, photo.height);
        let rows = colour_rows(&mut world, &camera, &photo, &TRACED, Colouring::Silhouette);
        let (figures, _, _) = world.draw_list(
            &camera,
            &crate::pinned_screen(),
            (size.0 as i32, size.1 as i32),
        );

        let all = card::draws(&tbytes);
        // `RON_CARD_FRAME` takes another pass, to make the match fail once
        // (3263's: 4165's journal).
        let at: i64 = std::env::var("RON_CARD_FRAME")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(PHOTO_FRAME);
        let pass: Vec<&card::CardDraw> = all.iter().filter(|d| i64::from(d.frame) == at).collect();
        assert!(
            pass.iter().all(|d| d.args_checked),
            "every draw's arguments read where the call left them"
        );
        let lit = |d: &card::CardDraw| {
            d.colour_op(0) == (TOP_BLENDTEXTUREALPHA, TA_TEXTURE, TA_TFACTOR)
                && d.colour_op(1) == (TOP_MODULATE, TA_CURRENT, TA_DIFFUSE)
        };
        let silhouette =
            |d: &card::CardDraw| d.colour_op(0) == (TOP_SELECTARG2, TA_TEXTURE, TA_TFACTOR);
        let whole = |d: &card::CardDraw| {
            let m = d.material();
            let l = d.light();
            !d.options.is_empty()
                && !d.names[0].is_empty()
                && d.colour_op(0).0 != 0
                && d.tfactor()[3] != 0
                && m[1][..3].iter().any(|&v| v != 0.0)
                && l[2][..3].iter().any(|&v| v != 0.0)
                && d.scene_ambient()[..3].iter().any(|&v| v != 0.0)
        };
        let lits: Vec<&card::CardDraw> = pass.iter().copied().filter(|d| lit(d)).collect();
        let sils: Vec<&card::CardDraw> = pass.iter().copied().filter(|d| silhouette(d)).collect();
        println!(
            "pass after frame {at}: {} draws, {} lit figure draws, {} silhouette draws",
            pass.len(),
            lits.len(),
            sils.len()
        );

        // Each figure to its nearest lit draw, nearest pairs first.
        let mut pairs: Vec<(f32, usize, usize)> = Vec::new();
        for (fi, f) in figures.iter().enumerate() {
            for (di, d) in lits.iter().enumerate() {
                let o = d.origin();
                let dd = ((o[0] - f.x as f32).powi(2)
                    + (o[1] - f.y as f32).powi(2)
                    + (o[2] - f.z as f32).powi(2))
                .sqrt();
                pairs.push((dd, fi, di));
            }
        }
        pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut fig_draw: std::collections::BTreeMap<usize, (usize, f32)> =
            std::collections::BTreeMap::new();
        let mut taken = vec![false; lits.len()];
        for &(dd, fi, di) in &pairs {
            if dd > render::silhouette::CARD_MATCH || taken[di] || fig_draw.contains_key(&fi) {
                continue;
            }
            taken[di] = true;
            fig_draw.insert(fi, (di, dd));
        }

        let mut matched = 0;
        let mut scored = 0;
        let mut worst = 0.0f32;
        for (f, _, who) in &rows {
            scored += 1;
            let name = format!("{}/{}", who.0, who.1);
            if who.3 {
                let art = rondata::artdata::build_art(
                    &rondata::Install::new(&install),
                    &std::iter::once(who.2).collect(),
                );
                let file = art
                    .get(&who.2)
                    .and_then(|a| a.as_ref().ok())
                    .map(|a| {
                        a.texture
                            .rsplit(['\\', '/'])
                            .next()
                            .unwrap_or("")
                            .to_ascii_lowercase()
                    })
                    .unwrap_or_default();
                let hit = pass.iter().find(|d| {
                    d.file(0).to_ascii_lowercase() == format!("{file}_nonquad")
                        && d.has("ENABLE_ALPHA_TEST")
                });
                let ok = hit.is_some_and(|d| whole(d));
                matched += usize::from(ok);
                println!(
                    "{name:>7} building piece {} art {file}: draw {:?} whole {ok}",
                    who.2,
                    hit.map(|d| d.index)
                );
                continue;
            }
            let mine: Vec<usize> = figures
                .iter()
                .enumerate()
                .filter(|(_, p)| (p.who, p.o) == (who.0, who.1))
                .map(|(i, _)| i)
                .collect();
            let draws: Vec<(usize, f32)> = mine
                .iter()
                .filter_map(|i| fig_draw.get(i).copied())
                .collect();
            let ok = !mine.is_empty()
                && draws.len() == mine.len()
                && draws.iter().all(|&(di, _)| whole(lits[di]));
            matched += usize::from(ok);
            for &(di, dd) in &draws {
                worst = worst.max(dd);
                let d = lits[di];
                println!(
                    "{name:>7} key {:>3} piece {:>5} figures {} -> draw {:>3} at {:5.1} units  {}  tfactor {:?}  options {}",
                    f.key,
                    who.2,
                    mine.len(),
                    d.index,
                    dd,
                    d.file(0),
                    d.tfactor(),
                    d.option_names().join(" ")
                );
            }
            if !ok {
                println!(
                    "{name:>7} piece {}: {} figures, {} matched",
                    who.2,
                    mine.len(),
                    draws.len()
                );
            }
        }
        println!(
            "matched with a whole state: {matched} of {scored} (the farthest figure {worst:.1} units)"
        );
        assert_eq!(
            (matched, scored),
            render::silhouette::RUN1559B_3279_DEVICE_STATE
        );

        // The colour row's readings against the original's state, draw by draw.
        for d in &lits {
            let owner = figures
                .iter()
                .enumerate()
                .find(|(i, _)| {
                    fig_draw
                        .get(i)
                        .is_some_and(|&(di, _)| std::ptr::eq(lits[di], *d))
                })
                .map(|(_, p)| p.who);
            let tf = d.tfactor();
            if let Some(w) = owner {
                // 4102: `c` is the owner's `TEAM_UNIT`.
                assert_eq!(
                    [tf[0], tf[1], tf[2]],
                    team_unit(w),
                    "draw {} owner {w}",
                    d.index
                );
            }
            let names = d.option_names();
            assert!(names.contains(&"ENABLE_LIGHTING") && names.contains(&"ENABLE_ALPHA_TEST"));
            for off in [
                "NORMAL_MAP",
                "ENABLE_VERTEX_COLOR_DIFFUSE",
                "TL_FAKE_LIGHT",
                "NO_HDR",
                "ENABLE_TEXTURE_FACTOR",
            ] {
                assert!(!names.contains(&off), "draw {} has {off}", d.index);
            }
            let m = d.material();
            for k in 0..3 {
                // 4114: the material ambient `UNIT_AMBIENT · UNIT_AMBIENT_I`,
                // its diffuse `UNIT_DIFFUSE`, and the scene's ambient raw.
                assert!((m[0][k] - UNIT_AMBIENT[k] * UNIT_AMBIENT_I).abs() < 1e-3);
                assert!((m[1][k] - UNIT_DIFFUSE).abs() < 1e-3);
                assert_eq!(m[2][k], 0.0);
                assert!((d.scene_ambient()[k] - SCENE_AMBIENT[k]).abs() < 1e-3);
            }
            let l = d.light();
            let h = std::f32::consts::FRAC_1_SQRT_2;
            assert!(
                (l[0][0] + h).abs() < 1e-3 && l[0][1].abs() < 1e-3 && (l[0][2] + h).abs() < 1e-3
            );
            assert_eq!(l[1][..3], [1.0, 1.0, 1.0], "the light's ambient");
            assert_eq!(l[2][..3], [1.0, 1.0, 1.0], "the light's diffuse");
            assert_eq!(
                (d.alpha_func(), d.alpha_ref()),
                (4, 0),
                "alpha test Greater 0"
            );
        }
        // The tanks' own vertices (`get_internal_stream`'s lit ring, 40
        // bytes: position, normal, uv0, uv1, in the model's space), carried
        // by the draw's `gWorld`, against our drawing's corners: each of
        // theirs to our nearest corner, the distance and the angle between
        // the normals.
        let scene = frame_scene(&mut world, &camera, size);
        let mut tank_rows = Vec::new();
        for (fi, f) in figures.iter().enumerate() {
            let Some(&(di, _)) = fig_draw.get(&fi) else {
                continue;
            };
            let d = lits[di];
            let Some(st) = d.stream.as_ref().filter(|st| st.stride == 40) else {
                continue;
            };
            let Some(ours) = scene
                .drawings
                .iter()
                .find(|g| (g.who, g.o, g.piece) == (f.who, f.o, f.piece))
            else {
                continue;
            };
            let w = d.world();
            let at = |v: [f32; 3], t: f32| -> [f32; 3] {
                std::array::from_fn(|r| {
                    w[4 * r] * v[0] + w[4 * r + 1] * v[1] + w[4 * r + 2] * v[2] + w[4 * r + 3] * t
                })
            };
            let word =
                |i: usize| f32::from_le_bytes(st.bytes[4 * i..4 * i + 4].try_into().unwrap());
            let corners: Vec<([f32; 3], [f32; 3])> = ours
                .tris
                .iter()
                .zip(&ours.normals)
                .flat_map(|(t, n)| (0..3).map(move |k| (t[k], n[k])))
                .collect();
            let (mut dist, mut ang) = (Vec::new(), Vec::new());
            for v in 0..st.vertices as usize {
                let b = 10 * v;
                if 4 * (b + 6) > st.bytes.len() {
                    break;
                }
                let p = at([word(b), word(b + 1), word(b + 2)], 1.0);
                let n = at([word(b + 3), word(b + 4), word(b + 5)], 0.0);
                // A flat-shaded mesh holds several corners at one point, one
                // normal each: the angle is the best among our corners within
                // 6 units of the nearest, a lower bound.
                let gap = |c: &[f32; 3]| (0..3).map(|k| (c[k] - p[k]).powi(2)).sum::<f32>().sqrt();
                let Some(near) = corners.iter().map(|c| gap(&c.0)).min_by(f32::total_cmp) else {
                    continue;
                };
                dist.push(near);
                let best = corners
                    .iter()
                    .filter(|c| gap(&c.0) <= near + 6.0)
                    .map(|c| {
                        let dot: f32 = (0..3).map(|k| c.1[k] * n[k]).sum();
                        dot.clamp(-1.0, 1.0).acos().to_degrees()
                    })
                    .min_by(f32::total_cmp)
                    .unwrap_or(180.0);
                ang.push(best);
            }
            dist.sort_by(f32::total_cmp);
            ang.sort_by(f32::total_cmp);
            if dist.is_empty() {
                continue;
            }
            let med = |v: &[f32]| v[v.len() / 2];
            println!(
                "tank {}/{} draw {}: {} vertices, nearest corner median {:.2} max {:.2} units; normals median {:.1} deg, 90th {:.1}, past 30 deg {}",
                f.who,
                f.o,
                d.index,
                dist.len(),
                med(&dist),
                dist[dist.len() - 1],
                med(&ang),
                ang[ang.len() * 9 / 10],
                ang.iter().filter(|&&a| a > 30.0).count()
            );
            tank_rows.push((med(&dist), med(&ang)));
        }
        // Stage 1's alpha, `MODULATE4X(TEXTURE, TFACTOR)` with blending on
        // (`enables` 0x287, `SRCALPHA`/`INVSRCALPHA`): a lit figure's pixel
        // is `A·C + (1 − A)·behind`, `A = min(1, 4·α·a)`, α the texel's
        // alpha and `a` the factor's. The scored figures' own pixels whose
        // texel alpha leaves `A` under 1 — those the colour row draws opaque.
        for d in &lits {
            assert_eq!(
                d.alpha_op(1),
                (5, TA_TEXTURE, TA_TFACTOR),
                "draw {} MODULATE4X",
                d.index
            );
            assert_eq!((d.enables() & 4, d.src_blend(), d.dst_blend()), (4, 5, 6));
        }
        let w = photo.width;
        let (mut own, mut thin) = (0usize, 0usize);
        let mut thin_by: Vec<(String, usize, usize)> = Vec::new();
        for (f, _, who) in &rows {
            if who.3 {
                continue;
            }
            let a = figures
                .iter()
                .enumerate()
                .filter(|(_, p)| (p.who, p.o) == (who.0, who.1))
                .find_map(|(i, _)| fig_draw.get(&i))
                .map_or(255.0, |&(di, _)| f32::from(lits[di].tfactor()[3]));
            let ours = painted_scene(
                &mut world,
                &scene,
                &camera,
                size,
                f.key,
                f.window,
                Colouring::Team,
            );
            let ww = (f.window.x1 - f.window.x0) as usize;
            let (mut o, mut t) = (0, 0);
            for (i, tx) in ours.iter().enumerate() {
                let Some(tx) = tx else { continue };
                let (x, y) = (f.window.x0 as usize + i % ww, f.window.y0 as usize + i / ww);
                if scene.keys.keys[y * w + x] != f.key || tx.alpha <= 0.0 {
                    continue;
                }
                o += 1;
                t += usize::from(4.0 * tx.alpha * a / 255.0 < 255.0);
            }
            own += o;
            thin += t;
            if t > 0 {
                thin_by.push((format!("{}/{}", who.0, who.1), t, o));
            }
        }
        println!("the figures' own pixels with A under 1: {thin} of {own}; by object {thin_by:?}");
        for d in &sils {
            // 4103: `SELECTARG2(TFACTOR)` at 128, source alpha over the rest.
            assert_eq!(d.tfactor()[3], SILHOUETTE_ALPHA);
            assert_eq!((d.src_blend(), d.dst_blend()), (5, 6));
            assert_eq!(d.alpha_op(0), (TOP_SELECTARG2, TA_TEXTURE, TA_TFACTOR));
        }
    }
}
