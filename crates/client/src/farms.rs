//! **A farm's ground: the wheat plots** (item 4636; `docs/RENDERER.md`
//! §6.1).
//!
//! Two draws lay a farm's plots on run1814b's packet, and the client draws
//! neither yet:
//! - `WallOut::draw_weathering@0063dde0` lays the tileset's `wheatfarm`
//!   underlay (`<UNDERLAYS>`, `farm_wheat.tga`) over the farm's 4 × 4 tiles
//!   through `TerrainOut::add_weather_underlay@00874d00`, for a farm whose
//!   record's `farm_type & 1` is clear; the pasture takes `pigfarm`, faded;
//! - `FarmsOut::draw@008d75f0` builds one box per plot, the 4 × 4 grid of
//!   the farm's `FarmStruct`, from the box template `wheat_box_verts`, its
//!   height `percent` of the full and its foot on `terrain_height`, and
//!   skips the pasture.
//!
//! [`corner`] is where both start: the farm's tile less half its type's
//! size, as `FarmsOut::draw` snaps it.

use sim::observe::{BuildingScene, FieldScene};

/// World units on a tile's side, `0xc0`.
const TILE: i32 = 192;

/// **The farm's corner tile** (`FarmsOut::draw@008d75f0`): the tile of its
/// point (`div_3_table[x >> 6]`, `x / 192` for a point on the map) less
/// half its type's size on each axis, `size >> 1`. The plots' boxes stand
/// at its tiles' centres.
#[cfg_attr(not(test), allow(dead_code))] // the plots' build is its caller
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
mod tests {
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

    /// **`RUN1814B_3294_FARM`** (item 4636; §6.1): `(agree, scored)` on
    /// `HUE`, the still's mean against the photograph's over each tile of
    /// slot 0's farms' footprints with at least `HUE.min_pixels` ground
    /// pixels (`ground_why`'s ground or an object's window, as the plaza's
    /// row takes them); then on `LUMA_READER`. Minted red: the client lays
    /// grass on the plots where the photograph has wheat, hue **2** and luma
    /// **29 of 48**. Tile (10, 165), 1,032 pixels: theirs (186, 156, 51)
    /// Y 153.4, ours (185, 188, 74) Y 174.5. The two that agree on hue,
    /// (11, 168) and (15, 166), are the photograph's greenest, theirs
    /// (158, 149, 50) and (159, 148, 50).
    const RUN1814B_3294_FARM: [(usize, usize); 2] = [(2, 48), (29, 48)];

    fn photo_path() -> PathBuf {
        std::env::var_os("RON_PHOTO_RUN1814B")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                    .join("ron-data/lab-experiments/2026-11-23-item-4624/run1814b-3294.tga")
            })
    }

    /// run1814b's frame drawn as the fog's and the ring's rows draw it: the
    /// harness's lobby at 3294, blended last at 3295, slot 0's fog, the
    /// pinned camera at the packet's `loc`.
    fn probe() -> Option<Probe> {
        let path = photo_path();
        let Ok(bytes) = std::fs::read(&path) else {
            println!("skipping: no photograph at {}", path.display());
            return None;
        };
        let photo = render::tga::decode(&bytes).expect("the photograph decodes");
        let root = std::env::var_os("RON_INSTALL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../../game"));
        let mut world = match crate::world::World::open_game(
            &root,
            &crate::world::gamelog_dir(),
            crate::world::Game::Lobby,
        ) {
            Ok(w) => w,
            Err(e) => {
                println!("skipping: {e}");
                return None;
            }
        };
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
            root,
            traced: &[],
        })
    }

    /// **The farm's row** (item 4636): our slot 0's fields stand on the
    /// packet's corners, and the photograph's plots against the still.
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
        let s = still(&mut p, "run1814b-3294-farm");
        let row = count("RUN1814B_3294_FARM", &plaza_tiles(&p, &pixels, &s));
        println!("RUN1814B_3294_FARM: {row:?}");
        assert_eq!(row, RUN1814B_3294_FARM);
    }
}
