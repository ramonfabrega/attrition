//! **`BuildType::place_roads@0063c580`** (item 3402): the whole function over a
//! [`PlaceRoadsView`] — three arms on the type, the footprint's ring, and the
//! two regeneration modes that follow a ring.
//!
//! [`crate::Sim::place_roads`] is the `REGEN_FORCE` call `Build::process`
//! makes and [`crate::Sim::unplace_roads`] the `param_5 == 0` teardown; it
//! carries neither the `REGEN_TOTAL` arm nor the virtual calls the original
//! makes on the object (slots `0x20` and `0xac`) and nothing in `Sim` calls
//! this walk. It is the original's *order of asking* and of writing, which is
//! what the sweep holds it to (`sweep/build_type_place_roads.rs`).

/// `RegenRoads`: the values `param_6` is compared with.
pub const REGEN_SIMPLE: i32 = 1;
/// Flag the object's city record for the next regeneration.
pub const REGEN_TOTAL: i32 = 2;
/// Search for the road to the city centre now.
pub const REGEN_FORCE: i32 = 3;

/// The tile bits the ring reads.
const BLOCKED: u16 = 0x4000;
const OCCUPIED_LOW: u16 = 3;
const OCCUPIED_BIT: u16 = 0x80;
const SURFACE: u16 = 0x30;
const SURFACE_OCEAN: u16 = 0x20;

/// The state the function reads and the calls it makes.
pub trait PlaceRoadsView {
    /// `road.length = 0`: the function-local static list is emptied first.
    fn road_clear(&mut self);
    /// `BuildTypeData::is_city` (slot `0x64`, or `is(0x19e, 0)` when the slot
    /// is the base).
    fn is_city(&mut self) -> bool;
    /// `BuildTypeData::connects_to_roads@006398c0`.
    fn connects_to_roads(&mut self) -> bool;
    /// The type's gatherer flag (`build_flags & 0x40`, slot `0x24` on a
    /// subclass).
    fn is_gatherer(&mut self) -> bool;
    /// The footprint's width and height in tiles (`+0x234`, `+0x238`).
    fn size(&self) -> (i32, i32);
    /// `div_3_table[i]`.
    fn div3(&self, i: i32) -> i32;
    /// The world's width and height in tiles.
    fn world_dims(&self) -> (i32, i32);
    /// The tile mask at `(x, y)`, unchecked: `world[width * y + x]`.
    fn tile_mask(&self, x: i32, y: i32) -> u16;
    /// `World::set_road_at(x, y, set, 0, 0)`.
    fn set_road_at(&mut self, x: i32, y: i32, set: i32);
    /// The object `(player, idx)`'s slot `0x20`.
    fn obj_can_road(&mut self, player: i32, idx: i32) -> bool;
    /// The object's slot `0xac` and the short at `+0x72` of what it answers: the
    /// city index, negative for none.
    fn obj_city(&mut self, player: i32, idx: i32) -> i16;
    /// The object's slot `0xac` again, and `+0x60 |= 0x100` on its answer.
    fn obj_mark_regen(&mut self, player: i32, idx: i32);
    /// The short at `+8` of `cities[player][city]`: the city's centre building.
    fn city_building(&mut self, player: i32, city: i16) -> i16;
    /// `PathFinder::find_road(city, &road, idx, player, building, player, -1, -1)`.
    fn find_road(&mut self, city: i16, idx: i32, player: i32, building: i16);
    /// `road.length`.
    fn road_length(&self) -> i32;
    fn set_road_length(&mut self, n: i32);
    /// `road.list[i]`'s `(to_x, to_y)`.
    fn road_entry(&self, i: i32) -> (i32, i32);
}

/// `BuildTypeData::tile_corner@00636440`, which the city arm inlines.
fn tile_corner(v: &impl PlaceRoadsView, x: i32, y: i32) -> (i32, i32) {
    let (sx, sy) = v.size();
    let mut cx = v.div3(x >> 6) * 0xc0 + 0x60;
    if sx & 1 == 0 {
        cx -= 0x60;
    }
    let mut cy = v.div3(y >> 6) * 0xc0 + 0x60;
    if sy & 1 == 0 {
        cy -= 0x60;
    }
    (v.div3(cx >> 6) - (sx >> 1), v.div3(cy >> 6) - (sy >> 1))
}

/// One ring: the border of the box from `corner - 1`, `size + pad` tiles on a
/// side, each tile that is on the map, not blocked, not ocean, and not an
/// occupied tile on the box's first row or column, handed to `set_road_at`.
fn ring(v: &mut impl PlaceRoadsView, corner: (i32, i32), pad: i32, set: i32) {
    let (sx, sy) = v.size();
    let (x0, y0) = (corner.0 - 1, corner.1 - 1);
    let (x1, y1) = (x0 + sx + pad, y0 + sy + pad);
    let mut x = x0;
    while x <= x1 {
        let mut y = y0;
        while y <= y1 {
            let edge = x == x0 || x == x1 || y == y0 || y == y1;
            let (w, h) = v.world_dims();
            if edge && x >= 0 && y >= 0 && x < w && y < h {
                let m = v.tile_mask(x, y);
                let free = m & OCCUPIED_LOW != OCCUPIED_LOW && m & OCCUPIED_BIT == 0;
                if m & BLOCKED == 0
                    && (free || (x != x0 && y != y0))
                    && m & SURFACE != SURFACE_OCEAN
                {
                    v.set_road_at(x, y, set);
                }
            }
            y += 1;
        }
        x += 1;
    }
}

/// The two regeneration modes after a ring (`param_5 != 0`).
fn regenerate(v: &mut impl PlaceRoadsView, idx: i32, player: i32, regen: i32) {
    if regen == REGEN_TOTAL {
        if v.obj_can_road(player, idx) && v.obj_city(player, idx) >= 0 {
            v.obj_mark_regen(player, idx);
        }
    } else if regen == REGEN_FORCE && v.obj_can_road(player, idx) && v.obj_city(player, idx) >= 0 {
        let city = v.obj_city(player, idx);
        let building = v.city_building(player, city);
        v.find_road(city, idx, player, building);
        while v.road_length() != 0 {
            let n = (v.road_length().max(1)) - 1;
            v.set_road_length(n);
            let (rx, ry) = v.road_entry(n);
            let (tx, ty) = (v.div3(rx >> 6), v.div3(ry >> 6));
            if v.tile_mask(tx, ty) & BLOCKED == 0 {
                v.set_road_at(tx, ty, 1);
            }
        }
    }
}

/// The original's body over a view. `x` and `y` are the building's `Coord`s,
/// `idx` and `player` name its object, `set` is `param_5`.
pub fn place_roads_steps(
    v: &mut impl PlaceRoadsView,
    x: i32,
    y: i32,
    idx: i32,
    player: i32,
    set: i32,
    regen: i32,
) {
    v.road_clear();
    if v.is_city() {
        let corner = tile_corner(v, x, y);
        ring(v, corner, 0, set);
        return;
    }
    if v.connects_to_roads() {
        let corner = tile_corner(v, x, y);
        ring(v, corner, 0, set);
    } else {
        if v.is_gatherer() {
            return;
        }
        if v.obj_city(player, idx) < 0 {
            return;
        }
        let corner = tile_corner(v, x, y);
        ring(v, corner, 1, set);
    }
    if set != 0 {
        regenerate(v, idx, player, regen);
    }
}
