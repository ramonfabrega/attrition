//! **`TerrainGroups::place_all@006a70d0`** — map generation's third phase
//! (`docs/MAPGEN.md` §7, §15 step 3): forest, mountains and rock dropped
//! onto the grid the zero-draw passes left, then the doober rolls and the
//! mountain fringe. It is checked two ways: draw for draw against the
//! original's own draw-site trace of the setup path (every draw below is
//! tagged with the **return address** the trace records for it, so a
//! parting names its site), and at `map.cpp` 7859 by parts 5 and 6.
//!
//! The arithmetic is integers, and one single-precision site: the pattern-2
//! and -3 region weights (`addss`/`divss`/`mulss`/`cvttss2si`, listing
//! 0x6a7e24..0x6a7f29), each operation correctly rounded, which
//! [`crate::single`] reproduces exactly (`docs/DECISIONS.md` 19). On a map
//! with one region the weight is `(c / c)·n = n` exactly.

use crate::ai_place::{MOVE_X, MOVE_Y};
use crate::combat::Rng;
use crate::mapgen::MapGrid;
use crate::scout::circle;
use crate::single::Single;
use crate::world::vector_dist;

/// A `game_random` stream that records, for every draw, the site the
/// original's trace names for it and the word before the step — the
/// sequence `rondata::mapgen::setup_draws` reads from a trace.
#[derive(Clone, Debug)]
pub struct Draws {
    pub rng: Rng,
    pub log: Vec<(u32, u32)>,
}

impl Draws {
    pub fn new(seed: u32) -> Draws {
        Draws {
            rng: Rng::new(seed),
            log: Vec::new(),
        }
    }

    /// `Random::get(game_random, 0, 0xffff)` from `site`.
    pub fn get(&mut self, site: u32) -> i32 {
        self.log.push((site, self.rng.seed));
        self.rng.roll()
    }
}

/// The draw sites, as return addresses (`report.py`'s `Function+0xNN`).
pub mod site {
    pub const LADDER: u32 = 0x006a_70d0 + 0x41e;
    pub const CLUMPS: u32 = 0x006a_70d0 + 0x498;
    pub const SIZE: u32 = 0x006a_70d0 + 0x6de;
    pub const OIL: u32 = 0x006a_70d0 + 0x771;
    pub const REGION_START: u32 = 0x006a_70d0 + 0x10ce;
    pub const MTN_SMALL: u32 = 0x0089_ca70 + 0x2d;
    pub const MTN_MED: u32 = 0x0089_ca70 + 0x71;
    pub const MTN_LARGE: u32 = 0x0089_ca70 + 0xc4;
    pub const ORTHOG: u32 = 0x006a_2320 + 0x69;
    pub const CORNER: u32 = 0x006a_2320 + 0xeb;
    pub const PPG_RING: u32 = 0x006a_4190 + 0x14e;
    pub const PPG_MIRROR: u32 = 0x006a_4190 + 0x99f;
    pub const PPG_ROUND: u32 = 0x006a_4190 + 0xd16;
    pub const PPG_DIAG: u32 = 0x006a_4190 + 0xdbd;
    pub const PRG_START: u32 = 0x006a_2f60 + 0x86;
    pub const PRG_STEP: u32 = 0x006a_2f60 + 0xd93;
    pub const BUSH_S: u32 = 0x006a_1540 + 0x19a;
    pub const BUSH_S_SIZE: u32 = 0x006a_1540 + 0x1bd;
    pub const BUSH_E: u32 = 0x006a_1540 + 0x319;
    pub const BUSH_E_SIZE: u32 = 0x006a_1540 + 0x33c;
    pub const ROCK: u32 = 0x006a_1540 + 0x5d7;
    pub const ROCK_SIZE: u32 = 0x006a_1540 + 0x602;
    pub const FRINGE_A: u32 = 0x006a_1cc0 + 0xf4;
    pub const FRINGE_B: u32 = 0x006a_1cc0 + 0x213;
}

/// `WData::flags` bits this phase reads and writes.
pub mod flag {
    pub const COAST: u16 = 0x4;
    pub const ROCK: u16 = 0x8;
    pub const MOUNTAIN: u16 = 0x10;
    pub const FOREST: u16 = 0x20;
    pub const ROAD: u16 = 0x80;
    pub const OIL: u16 = 0x800;
}

/// The orthogonal and diagonal direction lists as `TerrainGroup::TerrainGroup`
/// builds them, by prepending, so iteration from the head is the reverse of
/// `orthog_x/y[1..=4]` and `corner_x/y[1..=4]`.
const ORTHOG: [(i32, i32); 4] = [(-1, 0), (0, 1), (1, 0), (0, -1)];
const CORNER: [(i32, i32); 4] = [(-1, 1), (1, 1), (1, -1), (-1, -1)];
/// `corner_x/corner_y[1..=4]`, the copy's reflection.
const MIRROR: [(i32, i32); 4] = [(-1, -1), (1, -1), (1, 1), (-1, 1)];
/// `corner_box_x/y`: the four map corners, as multiples of `(xs, ys)`.
const CORNER_BOX: [(i32, i32); 4] = [(0, 0), (0, 1), (1, 0), (1, 1)];
/// `move_x/move_y[0..81]`: the 9×9 square, for the pure any-tests.
const SQUARE: i32 = 4;

/// One `GROUPENTRY`, as `TerrainGroups::init_terrain_data@006a6540` parses
/// it (`docs/MAPGEN.md` §7).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Group {
    /// 4 forest, 5 mountain, 6 rock, 7 oil, 8 cliff (a `Lands` record).
    pub kind: i32,
    pub chance: i32,
    pub grouping: i32,
    pub min_clumps: i32,
    pub max_clumps: i32,
    /// 0 player, 1 continent, 2 world, 3 nonplayer, 4 corner.
    pub pattern: i32,
    pub min_size: i32,
    pub max_size: i32,
    pub start_min: i32,
    pub start_max: i32,
    pub forest_space: i32,
    pub mount_space: i32,
    pub rock_space: i32,
    pub coast_space: i32,
    pub cent_min: i32,
    pub cent_max: i32,
    pub edge_min: i32,
    pub edge_max: i32,
    pub corner_min: i32,
    pub corner_max: i32,
    pub min_oil: i32,
    pub max_oil: i32,
}

/// One mountain template, `MountainRange::init@008998b0`'s cell-level
/// product: the tiles that carry mountain art, the cells holding them in
/// first-appearance order, and the solid cells — all relative to the
/// template's origin cell.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Template {
    pub tiles: Vec<(i32, i32)>,
    pub cells: Vec<(i32, i32)>,
    pub solid: Vec<(i32, i32)>,
}

/// The tileset's numbers this phase reads (`Data/tilesets.xml`'s
/// `TERRAINGROUP`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TilesetGroups {
    pub bush_clump_prob: i32,
    pub mnt_rock_clump_prob: i32,
    pub mnt_fringe_tree_prob: i32,
    /// `num_tiles[0]`: what `clear_group`'s first branch writes back as
    /// `land_sub − 1`'s base. Unreached on Great Sahara.
    pub base_tiles: i32,
}

/// What `place_all` reads that is not the grid: the groups, the sixteen
/// templates in range order, and the tileset's numbers.
#[derive(Clone, Debug, Default)]
pub struct Inputs {
    pub groups: Vec<Group>,
    pub templates: Vec<Template>,
    pub tileset: TilesetGroups,
    /// `game.info.map_style`; style 9 skips `treeify_mountains`.
    pub map_style: i32,
}

/// `MountainsData`'s three range rings, small/medium/large: range indices,
/// circular from the head, and the cursor `get_range` reads and advances.
#[derive(Clone, Debug)]
struct Rings {
    lists: [Vec<i32>; 3],
    cursor: [usize; 3],
}

impl Rings {
    /// The install's sixteen ranges: large 0..=6, medium 7..=14, small 15,
    /// each list prepended in file order.
    fn stock() -> Rings {
        Rings {
            lists: [vec![15], (7..=14).rev().collect(), (0..=6).rev().collect()],
            cursor: [0; 3],
        }
    }

    /// **`Mountains::randomize_mountains@0089ca70`**: small, medium, large,
    /// each re-seated at `r % len` when it holds two or more.
    fn randomize(&mut self, d: &mut Draws) {
        let sites = [site::MTN_SMALL, site::MTN_MED, site::MTN_LARGE];
        for ((list, cursor), site) in self.lists.iter().zip(&mut self.cursor).zip(sites) {
            let n = list.len();
            *cursor = if n >= 2 {
                (d.get(site) % n as i32) as usize
            } else {
                0
            };
        }
    }

    /// **`Mountains::get_range@0089cb50`**: the current range of size
    /// 1..=3, then the ring advances. Any other size answers 0.
    fn get(&mut self, size: i32) -> i32 {
        if !(1..=3).contains(&size) {
            return 0;
        }
        let k = (size - 1) as usize;
        let v = self.lists[k][self.cursor[k]];
        self.cursor[k] = (self.cursor[k] + 1) % self.lists[k].len();
        v
    }
}

/// `place_all`'s globals: `help_lowest_index`, `lowest_player`,
/// `player_scores` and `is_helping`.
#[derive(Clone, Debug, Default)]
struct Help {
    lowest_index: [i32; 5],
    lowest_player: [i32; 5],
    scores: Vec<[i32; 5]>,
    helping: bool,
}

/// One running pass: the grid, the stream and everything the callees
/// share.
struct Pass<'a> {
    g: &'a mut MapGrid,
    d: &'a mut Draws,
    templates: &'a [Template],
    rings: Rings,
    help: Help,
    /// `Mountains::verify_bits`, the sliding verify's per-cell marks.
    marks: Vec<bool>,
    /// The current group's `tiles` (`WCoordList` at +0x80).
    tiles: Vec<(i32, i32)>,
    base_tiles: i32,
}

/// `add_mountain`'s two verify modes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Verify {
    /// 4: `excluding_verify`, at the given origin.
    Excluding,
    /// 5: `sliding_excluding_verify`, which may move the origin.
    Sliding,
}

impl Pass<'_> {
    fn players(&self) -> usize {
        self.g.starts[0].list.len()
    }

    fn start(&self, p: usize) -> (i32, i32) {
        (self.g.starts[0].list[p], self.g.starts[1].list[p])
    }

    fn inb(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.g.xs && y < self.g.ys
    }

    fn at(&self, x: i32, y: i32) -> usize {
        (y * self.g.xs + x) as usize
    }

    fn flags(&self, x: i32, y: i32) -> u16 {
        self.g.cells[self.at(x, y)].flags
    }

    fn ocean(&self, x: i32, y: i32) -> bool {
        self.g.cells[self.at(x, y)].is_ocean()
    }

    fn city(&self, x: i32, y: i32) -> bool {
        self.g.city[self.at(x, y)]
    }

    fn own(&self, x: i32, y: i32) -> bool {
        self.tiles.contains(&(x, y))
    }

    /// `circle_x/y[circle_radius[0] .. circle_radius[r]]`: rings 1..=r.
    fn ring_points(r: i32) -> impl Iterator<Item = (i32, i32)> {
        let t = circle::table();
        let r = r.clamp(0, circle::RINGS as i32 - 1) as usize;
        (t.radius[0]..t.radius[r])
            .map(move |i| (i32::from(t.x[i as usize]), i32::from(t.y[i as usize])))
    }

    fn tile_at(&self, tx: i32, ty: i32) -> usize {
        (ty * 4 * self.g.xs + tx) as usize
    }

    fn tile_inb(&self, tx: i32, ty: i32) -> bool {
        let (w, h) = self.g.tile_dims();
        tx >= 0 && ty >= 0 && tx < w && ty < h
    }

    /// The cell that owns a tile.
    fn owner(&mut self, tx: i32, ty: i32) -> &mut crate::mapgen::WCell {
        let i = self.at(tx >> 2, ty >> 2);
        &mut self.g.cells[i]
    }

    /// `WorldData::has_mountain_tcoords@006b3050`: any of the cell's 16
    /// tiles is mountain (`mask & 3 == 2`).
    fn mountain_tiles(&self, x: i32, y: i32) -> bool {
        (0..16).any(|j| self.g.tiles[self.tile_at(4 * x + (j & 3), 4 * y + (j >> 2))] & 3 == 2)
    }

    /// `WorldData::has_blocked_neighbors@006b2990`.
    fn blocked_neighbours(&self, tx: i32, ty: i32) -> bool {
        (1..=8).any(|k| {
            let (u, v) = (tx + MOVE_X[k], ty + MOVE_Y[k]);
            self.tile_inb(u, v) && self.g.tiles[self.tile_at(u, v)] & 0x4000 != 0
        })
    }

    /// **`World::set_blocked_at@006b4900`**: the tile's 0x4000, its owner's
    /// `blocked`/`solid`, and the 0x2000 "beside a blocked tile" mark with
    /// its owner's `bad` count, on the tile and its eight neighbours — in
    /// call order, which is why the callers block in the original's order.
    /// `set_road_at(…, 0, 0, 0)` is the ROAD flag's clear: no road exists
    /// before the first frame.
    fn set_blocked(&mut self, tx: i32, ty: i32, on: bool) {
        let i = self.tile_at(tx, ty);
        let m = self.g.tiles[i];
        if on {
            if m & 0x4000 == 0 {
                let c = self.owner(tx, ty);
                c.blocked = c.blocked.wrapping_add(1);
                c.solid = c.solid.wrapping_add(1);
            }
            let mut m = m | 0x4000;
            if m & 0x2000 != 0 {
                let c = self.owner(tx, ty);
                c.bad = c.bad.wrapping_sub(1);
            }
            m &= !0x2000;
            self.g.tiles[i] = m;
            self.owner(tx, ty).flags &= !flag::ROAD;
        } else {
            if m & 0x4000 != 0 {
                let c = self.owner(tx, ty);
                c.blocked = c.blocked.wrapping_sub(1);
                c.solid = c.solid.wrapping_sub(1);
            }
            self.g.tiles[i] = m & 0x3fff;
        }
        for k in 1..=8 {
            let (u, v) = (tx + MOVE_X[k], ty + MOVE_Y[k]);
            if !self.tile_inb(u, v) {
                continue;
            }
            let j = self.tile_at(u, v);
            if on {
                if self.g.tiles[j] & 0x2000 == 0 {
                    let c = self.owner(u, v);
                    c.bad = c.bad.wrapping_add(1);
                    self.g.tiles[j] |= 0x2000;
                }
            } else if !self.blocked_neighbours(u, v) {
                if self.g.tiles[j] & 0x2000 != 0 {
                    let c = self.owner(u, v);
                    c.bad = c.bad.wrapping_sub(1);
                }
                self.g.tiles[j] &= !0x2000;
            }
        }
        if !on && self.blocked_neighbours(tx, ty) {
            if self.g.tiles[i] & 0x2000 == 0 {
                let c = self.owner(tx, ty);
                c.bad = c.bad.wrapping_add(1);
            }
            self.g.tiles[i] |= 0x2000;
        }
    }

    /// `World::set_oil_at@006b2a10`'s cell write. The OIL goods object it
    /// also makes is not held: no part walks it.
    fn set_oil(&mut self, x: i32, y: i32, on: bool) {
        let i = self.at(x, y);
        if on {
            self.g.cells[i].flags |= flag::OIL;
        } else {
            self.g.cells[i].flags &= 0xf7ff;
        }
    }

    /// **`TerrainGroup::available_rough_tile@006a24d0`** for `kind` 0x20
    /// (forest) or 8 (rock): pure.
    fn available(&self, gr: &Group, x: i32, y: i32, kind: u16) -> bool {
        if !self.inb(x, y) || self.ocean(x, y) {
            return false;
        }
        let f = self.flags(x, y);
        if f & 0x28 != 0
            || x == 0
            || y == 0
            || x == self.g.xs - 1
            || y == self.g.ys - 1
            || f & flag::MOUNTAIN != 0
            || self.city(x, y)
            || f & 0x44 != 0
        {
            return false;
        }
        for (space, bit) in [
            (gr.mount_space, flag::MOUNTAIN),
            (gr.forest_space, flag::FOREST),
            (gr.rock_space, flag::ROCK),
        ] {
            for (dx, dy) in Self::ring_points(space) {
                let (u, v) = (x + dx, y + dy);
                if self.inb(u, v) && self.flags(u, v) & bit != 0 && !self.own(u, v) {
                    return false;
                }
            }
        }
        for (dx, dy) in Self::ring_points(gr.coast_space) {
            let (u, v) = (x + dx, y + dy);
            if self.inb(u, v) && self.ocean(u, v) {
                return false;
            }
        }
        if kind == flag::ROCK {
            for k in 1..=8 {
                let (u, v) = (x + MOVE_X[k], y + MOVE_Y[k]);
                if self.inb(u, v)
                    && (self.mountain_tiles(u, v)
                        || (self.flags(u, v) & flag::ROCK != 0 && !self.own(u, v)))
                {
                    return false;
                }
            }
        }
        true
    }

    /// `drop_tile`'s land rewrite: a COAST cell becomes water 2, any other
    /// keeps its `land`.
    fn coast_land(&mut self, x: i32, y: i32) {
        let i = self.at(x, y);
        if self.g.cells[i].flags & flag::COAST != 0 {
            self.g.cells[i].land = 2;
        }
    }

    /// **`TerrainGroup::drop_tile@006a2ab0`**.
    fn drop_tile(&mut self, gr: &Group, x: i32, y: i32, maxd: i32, range: i32) -> bool {
        match gr.kind {
            4 => {
                if !self.available(gr, x, y, flag::FOREST) {
                    return false;
                }
                if let Some(&(sx, sy)) = self.tiles.first()
                    && vector_dist(x - sx, y - sy) > maxd
                {
                    return false;
                }
                self.coast_land(x, y);
                let i = self.at(x, y);
                self.g.cells[i].flags = (self.g.cells[i].flags & 0xffc3) | flag::FOREST;
                for j in 0..16 {
                    self.set_blocked(4 * x + (j & 3), 4 * y + (j >> 2), true);
                }
                self.tiles.push((x, y));
                true
            }
            6 => {
                if !self.available(gr, x, y, flag::ROCK) {
                    return false;
                }
                self.coast_land(x, y);
                let i = self.at(x, y);
                self.g.cells[i].flags = (self.g.cells[i].flags & 0xffc3) | flag::ROCK;
                self.tiles.push((x, y));
                true
            }
            5 => {
                if self
                    .add_mountain(gr, range, x, y, Verify::Excluding)
                    .is_none()
                {
                    return false;
                }
                self.tiles.push((x, y));
                true
            }
            7 => {
                self.set_oil(x, y, true);
                self.tiles.push((x, y));
                true
            }
            _ => false,
        }
    }

    /// **`TerrainGroup::clear_group@006a28e0`**: the undo of a failed group,
    /// which is not a true undo (§14).
    fn clear_group(&mut self) {
        for (x, y) in std::mem::take(&mut self.tiles) {
            let i = self.at(x, y);
            let f = self.g.cells[i].flags;
            let wet = self.ocean(x, y);
            if f & 0x30 == 0 && !wet && f & 0x48 == 0 {
                self.g.cells[i].land = 0;
                self.g.cells[i].land_sub = (self.base_tiles - 1) as u8;
            } else {
                if wet {
                    continue;
                }
                self.g.cells[i].flags &= 0xffd7;
                self.set_oil(x, y, false);
                for j in 0..16 {
                    let (tx, ty) = (4 * x + (j & 3), 4 * y + (j >> 2));
                    if self.g.tiles[self.tile_at(tx, ty)] & 3 != 2 {
                        self.set_blocked(tx, ty, false);
                    }
                }
            }
        }
    }

    /// **`TerrainGroup::place_oil_deposits@006a2d90`**: the first `n` rock
    /// tiles, in tile order, with no oil and a free 8-neighbour.
    fn place_oil(&mut self, gr: &Group, n: i32) {
        if gr.kind != 6 || n == 0 {
            return;
        }
        let n = n.min(self.tiles.len() as i32);
        let mut placed = 0;
        for (x, y) in self.tiles.clone() {
            if self.flags(x, y) & flag::OIL != 0 {
                continue;
            }
            let free = (1..=8).any(|k| {
                let (u, v) = (x + MOVE_X[k], y + MOVE_Y[k]);
                self.inb(u, v) && self.flags(u, v) & 0x28 == 0 && !self.mountain_tiles(u, v)
            });
            if free {
                self.set_oil(x, y, true);
                placed += 1;
                if placed == n {
                    return;
                }
            }
        }
    }

    /// **`TerrainGroup::randomize_orthogs@006a2320`**: the orthogonal and
    /// diagonal starts, `% 4` each, orthogonal first.
    fn randomize_orthogs(&mut self) -> (usize, usize) {
        let o = (self.d.get(site::ORTHOG) % 4) as usize;
        let c = (self.d.get(site::CORNER) % 4) as usize;
        (o, c)
    }

    // ---- mountains --------------------------------------------------------

    /// **`Mountains::quick_verify_template@00898ab0`** at origin `(x, y)`.
    fn quick_verify(&self, t: &Template, x: i32, y: i32) -> bool {
        t.cells.iter().all(|&(wx, wy)| {
            let (cx, cy) = (x + wx, y + wy);
            if !self.inb(cx, cy) {
                return false;
            }
            let f = self.flags(cx, cy);
            if f & 0x30 != 0 || self.ocean(cx, cy) || f & 0x48 != 0 || self.city(cx, cy) {
                return false;
            }
            Self::ring_points(1).all(|(dx, dy)| {
                let (u, v) = (cx + dx, cy + dy);
                self.inb(u, v) && !self.city(u, v)
            })
        })
    }

    /// Whether any start is nearer `(x, y)` than `start_min`.
    fn start_too_near(&self, x: i32, y: i32, start_min: i32) -> bool {
        (0..self.players()).any(|p| {
            let (sx, sy) = self.start(p);
            vector_dist(x - sx, y - sy) < start_min
        })
    }

    fn clear_marks(&mut self) {
        self.marks.fill(false);
    }

    /// **`Mountains::excluding_verify@00898570`** (mode 4).
    fn excluding_verify(&mut self, t: &Template, x: i32, y: i32, start_min: i32) -> bool {
        for &(wx, wy) in &t.cells {
            let (cx, cy) = (x + wx, y + wy);
            if !self.inb(cx, cy) || self.marks[self.at(cx, cy)] {
                continue;
            }
            let i = self.at(cx, cy);
            self.marks[i] = true;
            if self.start_too_near(cx, cy, start_min) {
                self.clear_marks();
                return false;
            }
        }
        self.clear_marks();
        self.quick_verify(t, x, y)
    }

    /// **`Mountains::sliding_excluding_verify@00898170`** (mode 5): each
    /// template cell in turn as the anchor at `(x, y)`. A cell's mark is
    /// **not** cleared between anchors, so a cell tested once — passed or
    /// failed — is skipped by every later anchor of the call (listing
    /// 0x89847b).
    fn sliding_verify(
        &mut self,
        t: &Template,
        x: i32,
        y: i32,
        start_min: i32,
    ) -> Option<(i32, i32)> {
        'anchor: for &(ax, ay) in &t.cells {
            let (ox, oy) = (x - ax, y - ay);
            for &(wx, wy) in &t.cells {
                let (cx, cy) = (ox + wx, oy + wy);
                if !self.inb(cx, cy) || self.marks[self.at(cx, cy)] {
                    continue;
                }
                let i = self.at(cx, cy);
                self.marks[i] = true;
                if self.start_too_near(cx, cy, start_min) {
                    continue 'anchor;
                }
            }
            if self.quick_verify(t, ox, oy) {
                self.clear_marks();
                return Some((ox, oy));
            }
        }
        self.clear_marks();
        None
    }

    /// **`Mountains::add_mountain@0089c2e0`**: verify, the spacing rings,
    /// then the writes — the solid cells' MOUNTAIN flag, every tile blocked
    /// and marked mountain in template order, and `set_behind_flags`.
    /// Answers the origin placed at, or `None` with nothing written.
    fn add_mountain(
        &mut self,
        gr: &Group,
        range: i32,
        x: i32,
        y: i32,
        mode: Verify,
    ) -> Option<(i32, i32)> {
        let templates = self.templates;
        let t = templates.get(usize::try_from(range).ok()?)?;
        let (x, y) = match mode {
            Verify::Excluding => self
                .excluding_verify(t, x, y, gr.start_min)
                .then_some((x, y))?,
            Verify::Sliding => self.sliding_verify(t, x, y, gr.start_min)?,
        };
        for &(wx, wy) in &t.cells {
            let (cx, cy) = (x + wx, y + wy);
            for (space, bits) in [
                (gr.mount_space, flag::MOUNTAIN | 0x40),
                (gr.coast_space, flag::COAST),
                (gr.forest_space, flag::FOREST),
                (gr.rock_space, flag::ROCK),
            ] {
                for (dx, dy) in Self::ring_points(space) {
                    let (u, v) = (cx + dx, cy + dy);
                    if self.inb(u, v) && self.flags(u, v) & bits != 0 {
                        return None;
                    }
                }
            }
        }
        for &(wx, wy) in &t.solid {
            let i = ((y + wy) * self.g.xs + x + wx) as usize;
            if let Some(c) = self.g.cells.get_mut(i) {
                c.flags = (c.flags & !0x28) | flag::MOUNTAIN;
            }
        }
        for &(tx, ty) in &t.tiles {
            let (tx, ty) = (tx + 4 * x, ty + 4 * y);
            self.set_blocked(tx, ty, true);
            let i = self.tile_at(tx, ty);
            self.g.tiles[i] = (self.g.tiles[i] & 0xfffe) | 2;
        }
        for &(tx, ty) in &t.tiles {
            let (tx, ty) = (tx + 4 * x, ty + 4 * y);
            for k in 1..=3 {
                let (u, v) = (tx + MOVE_X[k], ty + MOVE_Y[k]);
                if self.tile_inb(u, v) {
                    let j = self.tile_at(u, v);
                    if self.g.tiles[j] & 3 != 2 {
                        self.g.tiles[j] |= 8;
                    }
                }
            }
        }
        Some((x, y))
    }

    // ---- the shared predicates --------------------------------------------

    /// The centre, edge and corner bounds every start test carries: `adj`
    /// moves the centre one cell toward a cell past it (`cmovle`), which the
    /// player candidates and the region starts take and the region growth
    /// does not.
    fn bounds_ok(&self, gr: &Group, x: i32, y: i32, adj: bool, edge_corner: bool) -> bool {
        let (hw, hh) = (self.g.xs >> 1, self.g.ys >> 1);
        let (cx, cy) = if adj {
            (hw + i32::from(x > hw), hh + i32::from(y > hh))
        } else {
            (hw, hh)
        };
        let dc = vector_dist(x - cx, y - cy);
        if (gr.cent_min > 0 && dc < gr.cent_min) || (gr.cent_max > 0 && dc > gr.cent_max) {
            return false;
        }
        if !edge_corner {
            return true;
        }
        let e = x.min(y).min(self.g.xs - x).min(self.g.ys - y);
        if (gr.edge_min > 0 && e < gr.edge_min) || (gr.edge_max > 0 && e > gr.edge_max) {
            return false;
        }
        let cd = CORNER_BOX
            .iter()
            .map(|&(bx, by)| vector_dist(x - bx * self.g.xs, y - by * self.g.ys))
            .fold(99_999, i32::min);
        !((gr.corner_min > 0 && cd < gr.corner_min) || (gr.corner_max > 0 && cd > gr.corner_max))
    }

    /// The border test, which compares `y` with `xs − 1` as the listing
    /// does (0x6a4692, 0x6a357a).
    fn border(&self, x: i32, y: i32) -> bool {
        x == 0 || y == 0 || x == self.g.xs - 1 || y == self.g.xs - 1
    }

    /// Any in-bounds cell of the `(2r+1)²` square about `(x, y)` carrying
    /// `bit`.
    fn square_has(&self, x: i32, y: i32, r: i32, bit: u16) -> bool {
        (-r..=r).any(|dx| {
            (-r..=r).any(|dy| {
                let (u, v) = (x + dx, y + dy);
                self.inb(u, v) && self.flags(u, v) & bit != 0
            })
        })
    }

    // ---- place_player_group -----------------------------------------------

    /// **`TerrainGroup::place_player_group@006a4190`** (`docs/MAPGEN.md`
    /// §7): a clump of `gr` near player `p`'s start, on the circle ring
    /// `[start_min, start_max]` from a drawn index. `shape` is the clump's
    /// forest shape, shared by every player of one clump.
    #[allow(clippy::too_many_arguments)]
    fn place_player_group(
        &mut self,
        gr: &mut Group,
        size: i32,
        p: usize,
        size2: i32,
        oil: i32,
        shape: &mut Vec<(i32, i32)>,
        forest_flag: bool,
    ) -> bool {
        let (s0x, s0y) = self.start(p);
        let home = self.g.cells[self.at(s0x, s0y)].region;
        self.tiles.clear();
        if gr.start_max == 0 || gr.start_max > 64 {
            gr.start_max = 64;
        }
        if gr.start_min > gr.start_max {
            return false;
        }
        assert!(
            gr.start_min != gr.start_max,
            "place_player_group: an empty ring never ends"
        );
        let msz = if gr.kind == 8 { size2 - 1 } else { size2 };
        let t = circle::table();
        let lo = t.radius[gr.start_min as usize];
        let hi = t.radius[gr.start_max as usize];
        let s = if lo < hi - 1 {
            self.d.get(site::PPG_RING) % (hi - lo) + lo
        } else {
            lo
        };
        let first = s + 1;
        let mut idx = first;
        loop {
            if idx == hi {
                idx = lo;
            }
            let (x, y) = (
                s0x + i32::from(t.x[idx as usize]),
                s0y + i32::from(t.y[idx as usize]),
            );
            if self.player_candidate(gr, p, x, y, home, forest_flag)
                && self.player_place(gr, size, (s0x, s0y), x, y, msz, oil, shape)
            {
                return true;
            }
            idx += 1;
            if idx == first {
                return false;
            }
        }
    }

    /// `place_player_group`'s candidate predicates, in order (pure).
    fn player_candidate(
        &self,
        gr: &Group,
        p: usize,
        x: i32,
        y: i32,
        home: i16,
        forest_flag: bool,
    ) -> bool {
        if !self.inb(x, y) {
            return false;
        }
        let c = &self.g.cells[self.at(x, y)];
        if !(c.region >= 0x40 || c.region == home) {
            return false;
        }
        let (s0x, s0y) = self.start(p);
        let d0 = vector_dist(x - s0x, y - s0y);
        for q in 0..self.players() {
            let (qx, qy) = self.start(q);
            if q != p && vector_dist(x - qx, y - qy) < d0 {
                return false;
            }
        }
        if c.flags & (0x10 | 0x68 | 0x1800) != 0 || c.blocked != 0 {
            return false;
        }
        if !self.bounds_ok(gr, x, y, true, false) {
            return false;
        }
        if gr.kind == 4 && forest_flag && self.square_has(x, y, SQUARE, flag::FOREST) {
            return false;
        }
        if !self.bounds_ok(gr, x, y, true, true) || self.border(x, y) {
            return false;
        }
        !(gr.kind == 4 && self.square_has(x, y, 1, 0x40))
    }

    /// An accepted candidate: the mountain, the copy of the clump's shape,
    /// or the seed and its growth.
    #[allow(clippy::too_many_arguments)]
    fn player_place(
        &mut self,
        gr: &Group,
        size: i32,
        home: (i32, i32),
        x: i32,
        y: i32,
        msz: i32,
        oil: i32,
        shape: &mut Vec<(i32, i32)>,
    ) -> bool {
        if gr.kind == 5 {
            return self.add_mountain(gr, msz, x, y, Verify::Sliding).is_some();
        }
        if gr.kind == 8 {
            return false;
        }
        if gr.kind == 4 && !shape.is_empty() {
            let (mx, my) = MIRROR[(self.d.get(site::PPG_MIRROR) & 3) as usize];
            let cells: Vec<_> = shape
                .iter()
                .map(|&(dx, dy)| (mx * dx + x, my * dy + y))
                .collect();
            if !cells
                .iter()
                .all(|&(u, v)| self.available(gr, u, v, flag::FOREST))
            {
                return false;
            }
            for (u, v) in cells {
                self.drop_tile(gr, u, v, size, msz);
            }
            return true;
        }
        if !self.drop_tile(gr, x, y, size / 4 + 1, msz) {
            return false;
        }
        if gr.kind == 4 {
            shape.push((0, 0));
        }
        if self.player_grow(gr, size, home, shape) {
            if gr.kind == 6 {
                self.place_oil(gr, oil);
            }
            return true;
        }
        if (self.tiles.len() as i32) < size {
            self.clear_group();
        }
        false
    }

    /// `place_player_group`'s growth rounds: from the seed alone for forest,
    /// from each tile in turn for the rest, a diagonal coin per direction
    /// once the clump holds three.
    fn player_grow(
        &mut self,
        gr: &Group,
        size: i32,
        (s0x, s0y): (i32, i32),
        shape: &mut Vec<(i32, i32)>,
    ) -> bool {
        'round: loop {
            let n = self.tiles.len();
            let k = if n >= 2 {
                (self.d.get(site::PPG_ROUND) % n as i32) as usize
            } else {
                0
            };
            for iter in 0..n {
                let c = (k + 1 + iter) % n;
                let idx = if gr.kind == 4 { 0 } else { c };
                let (o, c0) = self.randomize_orthogs();
                for dir in 0..4 {
                    let diag = n >= 3 && self.d.get(site::PPG_DIAG) & 1 != 0;
                    let (dx, dy) = if diag {
                        CORNER[(c0 + dir) % 4]
                    } else {
                        ORTHOG[(o + dir) % 4]
                    };
                    let (bx, by) = self.tiles[idx];
                    let (nx, ny) = (bx + dx, by + dy);
                    if self.own(nx, ny) || !self.inb(nx, ny) || self.city(nx, ny) {
                        continue;
                    }
                    let d = vector_dist(nx - s0x, ny - s0y);
                    if d < gr.start_min
                        || d > gr.start_max
                        || !self.bounds_ok(gr, nx, ny, true, true)
                    {
                        continue;
                    }
                    if self.drop_tile(gr, nx, ny, size / 4 + 1, -1) {
                        if gr.kind == 4 {
                            let (ox, oy) = self.tiles[0];
                            shape.push((nx - ox, ny - oy));
                        }
                        if self.tiles.len() as i32 >= size {
                            return true;
                        }
                        continue 'round;
                    }
                }
            }
            return false;
        }
    }

    // ---- place_region_group -----------------------------------------------

    /// A region's cells as `Regions::rebuild_coords@0067f800` lists them:
    /// y outer, x inner.
    fn region_coords(&self, reg: i32) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        for y in 0..self.g.ys {
            for x in 0..self.g.xs {
                if i32::from(self.g.cells[self.at(x, y)].region) == reg {
                    out.push((x, y));
                }
            }
        }
        out
    }

    /// The start-distance bounds against **every** start.
    fn starts_ok(&self, gr: &Group, x: i32, y: i32) -> bool {
        (0..self.players()).all(|i| {
            let (sx, sy) = self.start(i);
            let d = vector_dist(x - sx, y - sy);
            !((gr.start_min > 0 && d < gr.start_min) || (gr.start_max > 0 && d > gr.start_max))
        })
    }

    /// **`TerrainGroup::place_region_group@006a2f60`**: a clump of `gr` in
    /// region `reg`, its start the first cell a 16-bit Galois LFSR (`(s >> 1)
    /// ^ 0xB400` on an odd state, period 65,535) reaches from a drawn state
    /// that passes every test, then
    /// grown orthogonally. `range` is the start drop's mountain range.
    fn place_region_group(
        &mut self,
        gr: &Group,
        size: i32,
        reg: i32,
        range: i32,
        oil: i32,
    ) -> bool {
        let tidx = (gr.kind - 4) as usize;
        self.tiles.clear();
        let coords = self.region_coords(reg);
        let n = coords.len() as i32;
        let s0 = if n >= 2 {
            self.d.get(site::PRG_START) % n + 1
        } else {
            1
        };
        let mut placed = false;
        let mut s = s0;
        loop {
            let (x, y) = coords[(s - 1) as usize];
            if self.region_candidate(gr, x, y) {
                placed = self.drop_tile(gr, x, y, size / 4 + 1, range);
                if placed {
                    for p in 0..self.players() {
                        let (sx, sy) = self.start(p);
                        self.help.scores[p][tidx] += vector_dist(sx - x, sy - y);
                    }
                    let (mut best, mut lp) = (0, 0);
                    for p in 0..self.players() {
                        if self.help.scores[p][tidx] > best {
                            best = self.help.scores[p][tidx];
                            lp = p as i32;
                        }
                    }
                    self.help.lowest_player[tidx] = lp;
                    if matches!(gr.kind, 5 | 7 | 8) {
                        return placed;
                    }
                    if self.region_grow(gr, size, (x, y)) {
                        if gr.kind == 6 {
                            self.place_oil(gr, oil);
                        }
                        return true;
                    }
                }
            }
            loop {
                s = if s & 1 != 0 { s ^ 0x16800 } else { s } >> 1;
                if s <= n {
                    break;
                }
            }
            if s == s0 {
                return placed;
            }
        }
    }

    /// `place_region_group`'s start tests, in order, the drop excluded.
    fn region_candidate(&self, gr: &Group, x: i32, y: i32) -> bool {
        if !self.inb(x, y) {
            return false;
        }
        let c = &self.g.cells[self.at(x, y)];
        let wet = c.is_ocean();
        if (gr.kind == 7) == !(c.flags & flag::COAST != 0 || c.land == 2) && gr.kind == 7 {
            return false;
        }
        if gr.kind != 7 && wet {
            return false;
        }
        if self.city(x, y) || !self.starts_ok(gr, x, y) {
            return false;
        }
        if c.flags & (0x10 | 0x68 | 0x1800) != 0 || c.blocked != 0 {
            return false;
        }
        assert!(
            gr.kind != 7,
            "place_region_group: sea oil's circle test is not ported"
        );
        if !self.bounds_ok(gr, x, y, true, true) || self.border(x, y) {
            return false;
        }
        if gr.kind == 4 && self.square_has(x, y, 1, 0x40) {
            return false;
        }
        if self.help.helping {
            let mut best = i32::MAX;
            let mut near = i32::MAX;
            for p in 0..self.players() {
                let (sx, sy) = self.start(p);
                let d = vector_dist(sx - x, sy - y);
                if d < best {
                    best = d;
                    near = p as i32;
                }
            }
            if near != self.help.lowest_player[(gr.kind - 4) as usize] {
                return false;
            }
        }
        true
    }

    /// `place_region_group`'s growth: one orthogonal step at a time, each
    /// tile tried from a drawn one on, four directions a tile; a step that
    /// adds nothing clears the group and the walk resumes. The start cell's
    /// start-city test is re-read for every neighbour (listing 0x6a3da1),
    /// so it never fires; the centre is not moved here.
    fn region_grow(&mut self, gr: &Group, size: i32, _start: (i32, i32)) -> bool {
        loop {
            let cnt = self.tiles.len();
            let k0 = if cnt >= 2 {
                (self.d.get(site::PRG_STEP) % cnt as i32 + 1) as usize
            } else {
                1
            };
            let mut k = k0;
            let added = 'step: loop {
                let t = if k != cnt { k } else { 0 };
                let (o, _) = self.randomize_orthogs();
                for j in 0..4 {
                    let (dx, dy) = ORTHOG[(o + j) % 4];
                    let (bx, by) = self.tiles[t];
                    let (nx, ny) = (bx + dx, by + dy);
                    let reject = self.own(nx, ny)
                        || !self.inb(nx, ny)
                        || !self.starts_ok(gr, nx, ny)
                        || !self.bounds_ok(gr, nx, ny, false, false);
                    if gr.kind == 4 && self.square_has(nx, ny, 1, 0x40) {
                        continue;
                    }
                    if !reject && self.drop_tile(gr, nx, ny, size / 4 + 1, -1) {
                        break 'step true;
                    }
                }
                k = t + 1;
                if k == k0 {
                    break false;
                }
            };
            if !added {
                self.clear_group();
                return false;
            }
            if size as usize <= self.tiles.len() {
                return true;
            }
        }
    }
}

// ---- place_all ------------------------------------------------------------

/// **`TerrainGroups::place_all@006a70d0`** on `grid` (`docs/MAPGEN.md` §7),
/// as `Map::make` calls it: `place_all(splash, 1)`. Every draw goes through
/// `d`, tagged by its site.
pub fn place_all(grid: &mut MapGrid, d: &mut Draws, inputs: &Inputs) {
    let players = grid.starts[0].list.len();
    let marks = vec![false; grid.cells.len()];
    let mut pass = Pass {
        g: grid,
        d,
        templates: &inputs.templates,
        rings: Rings::stock(),
        help: Help {
            scores: vec![[0; 5]; players],
            ..Help::default()
        },
        marks,
        tiles: Vec::new(),
        base_tiles: inputs.tileset.base_tiles,
    };
    let mut groups = inputs.groups.clone();
    pass.rings.randomize(pass.d);

    // The selection ladder.
    let n_groups = groups.len();
    let mut clumps = vec![0; n_groups];
    let mut chosen = vec![false; n_groups];
    let (mut roll, mut last_grouping, mut prev_placed) = (0, -10, false);
    for (i, gr) in groups.iter().enumerate() {
        let carried;
        if gr.grouping == 0 || gr.grouping != last_grouping {
            roll = pass.d.get(site::LADDER) % 100;
            last_grouping = gr.grouping;
            carried = false;
        } else {
            carried = prev_placed;
        }
        let placed = if roll < 0 {
            gr.chance == 0 && carried
        } else {
            roll -= gr.chance;
            roll < 0
        };
        prev_placed = placed;
        if !placed {
            continue;
        }
        chosen[i] = true;
        let n = if gr.min_clumps < gr.max_clumps {
            pass.d.get(site::CLUMPS) % (gr.max_clumps - gr.min_clumps + 1) + gr.min_clumps
        } else {
            gr.min_clumps
        };
        clumps[i] = n;
        pass.help.lowest_index[(gr.kind - 4) as usize] += n;
    }
    for k in 0..5 {
        let h = pass.help.lowest_index[k];
        pass.help.lowest_index[k] = if h <= 1 {
            i32::MAX
        } else {
            let h = h >> 1;
            if h & 1 != 0 { h - 1 } else { h }
        };
    }
    pass.help.helping = false;

    for i in 0..n_groups {
        if !chosen[i] {
            continue;
        }
        let n = clumps[i];
        let gr = &groups[i];
        let k = (gr.kind - 4) as usize;
        let (size_lo, mut size_hi) = (gr.min_size.min(gr.max_size), gr.min_size.max(gr.max_size));
        let (mut oil_lo, mut oil_hi) = (gr.min_oil.min(gr.max_oil), gr.min_oil.max(gr.max_oil));
        match gr.kind {
            5 => size_hi = size_hi.min(3),
            8 => size_hi = size_hi.min(5),
            6 => {
                oil_hi = oil_hi.min(size_hi);
                oil_lo = oil_lo.max(0);
            }
            _ => {}
        }
        let mut sizes = Vec::with_capacity(n as usize);
        let mut oils = Vec::with_capacity(n as usize);
        for _ in 0..n {
            let mut s = if size_lo < size_hi {
                size_lo + pass.d.get(site::SIZE) % (size_hi - size_lo + 1)
            } else {
                size_lo
            };
            if gr.pattern == 0 && gr.kind == 4 {
                if s & 1 == 0 {
                    s += 1;
                }
                s = s.min(9);
            }
            sizes.push(s);
            oils.push(if gr.kind == 6 {
                if oil_lo < oil_hi {
                    oil_lo + pass.d.get(site::OIL) % (oil_hi - oil_lo + 1)
                } else {
                    oil_lo
                }
            } else {
                0
            });
        }
        let gr = &mut groups[i];
        match gr.pattern {
            0 => pass.pattern_player(gr, &sizes, &oils),
            1 => pass.pattern_continent(gr, k, &sizes, &oils),
            2 | 3 => pass.pattern_world(gr, k, &sizes, &oils),
            _ => {}
        }
    }

    add_doobers(&mut pass, &inputs.tileset);
    if inputs.map_style != 9 {
        treeify_mountains(&mut pass, inputs.tileset.mnt_fringe_tree_prob);
    }
}

impl Pass<'_> {
    /// The region records' cell counts (`Region.size`, +0x14).
    fn region_size(&self, reg: i32) -> i32 {
        self.g
            .cells
            .iter()
            .filter(|c| i32::from(c.region) == reg)
            .count() as i32
    }

    /// Pattern 0, `player`: clump outer, player inner, one shape a clump.
    fn pattern_player(&mut self, gr: &mut Group, sizes: &[i32], oils: &[i32]) {
        if gr.kind == 7 {
            return;
        }
        for (c, &s) in sizes.iter().enumerate() {
            let mut shape = Vec::new();
            for p in 0..self.players() {
                if gr.kind == 5 {
                    self.player_mountain(gr, s, p, oils[c], &mut shape);
                    continue;
                }
                if !self.place_player_group(gr, s, p, s, oils[c], &mut shape, gr.kind == 4)
                    && gr.kind == 4
                {
                    self.place_player_group(gr, s, p, s, oils[c], &mut shape, false);
                }
            }
        }
    }

    /// A player mountain and its retries over the size's ring, then a size
    /// smaller.
    fn player_mountain(
        &mut self,
        gr: &mut Group,
        s: i32,
        p: usize,
        oil: i32,
        shape: &mut Vec<(i32, i32)>,
    ) {
        let r0 = self.rings.get(s);
        if self.place_player_group(gr, s, p, r0, oil, shape, false) {
            return;
        }
        let first = r0;
        let mut rr = r0;
        let mut res;
        loop {
            res = self.place_player_group(gr, s, p, rr, oil, shape, false);
            rr = self.rings.get(s);
            if rr == first || res {
                break;
            }
        }
        if res {
            return;
        }
        self.rings.randomize(self.d);
        if s > 1 {
            rr = self.rings.get(s - 1);
            let first = rr;
            loop {
                res = self.place_player_group(gr, s, p, rr, oil, shape, false);
                rr = self.rings.get(s - 1);
                if rr == first || res {
                    break;
                }
            }
            self.rings.randomize(self.d);
            if res && s == 2 {
                rr = self.rings.get(1);
                let first = rr;
                loop {
                    let r2 = self.place_player_group(gr, s, p, rr, oil, shape, false);
                    rr = self.rings.get(1);
                    if rr == first || r2 {
                        break;
                    }
                }
            }
        }
    }

    /// The land regions, or the sea ones for oil, with more than five cells.
    fn region_span(gr: &Group) -> std::ops::Range<i32> {
        if gr.kind == 7 { 0x40..0x7f } else { 1..0x40 }
    }

    /// Pattern 1, `continent`: region outer, clump inner, then every failed
    /// clump again in every region with `is_helping` set.
    fn pattern_continent(&mut self, gr: &mut Group, k: usize, sizes: &[i32], oils: &[i32]) {
        let mut failed = Vec::new();
        for reg in Self::region_span(gr) {
            if self.region_size(reg) <= 5 {
                continue;
            }
            for (c, &s) in sizes.iter().enumerate() {
                self.help.helping = c as i32 >= self.help.lowest_index[k];
                let a3 = if gr.kind == 5 { self.rings.get(s) } else { s };
                if !self.place_region_group(gr, s, reg, a3, oils[c]) {
                    failed.push(c);
                }
            }
        }
        for c in failed {
            for reg in Self::region_span(gr) {
                if self.region_size(reg) <= 5 {
                    continue;
                }
                self.help.helping = true;
                let a3 = if gr.kind == 5 {
                    self.rings.get(sizes[c])
                } else {
                    sizes[c]
                };
                self.place_region_group(gr, sizes[c], reg, a3, oils[c]);
            }
        }
    }

    /// Patterns 2 and 3, `world` and `nonplayer`: a list of the regions,
    /// weighted in single precision by size when there are more than six
    /// clumps over fewer than six regions, walked one node a clump from a
    /// drawn node.
    fn pattern_world(&mut self, gr: &mut Group, k: usize, sizes: &[i32], oils: &[i32]) {
        let n = sizes.len() as i32;
        // The list, head first; every insertion prepends.
        let mut list: Vec<i32> = Vec::new();
        for reg in Self::region_span(gr) {
            if self.region_size(reg) <= 5 {
                continue;
            }
            if gr.pattern == 3
                && (0..self.players()).any(|p| {
                    let (sx, sy) = self.start(p);
                    i32::from(self.g.cells[self.at(sx, sy)].region) == reg
                })
            {
                continue;
            }
            list.insert(0, reg);
        }
        if list.is_empty() {
            return;
        }
        if n > 6 && list.len() < 6 {
            let counts: Vec<i32> = list.iter().map(|&r| self.region_size(r)).collect();
            let (mut best_reg, mut best_cnt) = (0, 0);
            let mut total = Single::from_i32(0);
            for (&r, &c) in list.iter().zip(&counts) {
                if c > best_cnt {
                    best_reg = r;
                    best_cnt = c;
                }
                total = total.addss(Single::from_i32(c));
            }
            let weights: Vec<i32> = counts
                .iter()
                .map(|&c| {
                    Single::from_i32(c)
                        .divss(total)
                        .mulss(Single::from_i32(n))
                        .to_i32()
                })
                .collect();
            let mut cur = 0usize;
            for &w in &weights {
                let data = list[cur];
                let extra = if w == 0 { 0 } else { w - 1 };
                for _ in 0..extra {
                    list.insert(0, data);
                    cur = 0;
                }
                cur = (cur + 1) % list.len();
            }
            list.insert(0, best_reg);
        }
        let m = list.len();
        let mut cur = if m >= 2 {
            (self.d.get(site::REGION_START) % m as i32) as usize
        } else {
            0
        };
        let mut anchor = list[cur];
        let mut mtn_done = [false; 128];
        let mut c = 0usize;
        while c < sizes.len() {
            let saved = c;
            let reg = list[cur];
            self.help.helping = c as i32 >= self.help.lowest_index[k];
            let s = sizes[c];
            let res = match gr.kind {
                5 => self.world_mountain(gr, s, reg, oils[c], &mut mtn_done),
                8 => {
                    let r = self.place_region_group(gr, s, reg, (s - 1).max(1), oils[c]);
                    if r {
                        mtn_done[reg as usize] = true;
                    }
                    r
                }
                _ => self.place_region_group(gr, s, reg, -1, oils[c]),
            };
            c += 1;
            cur = (cur + 1) % m;
            if !res {
                if list[cur] != anchor {
                    c = saved;
                }
            } else {
                anchor = reg;
            }
        }
    }

    /// A world-pattern mountain and its retries (§7): not reached on Great
    /// Sahara, whose world groups are trees.
    fn world_mountain(
        &mut self,
        gr: &mut Group,
        s: i32,
        reg: i32,
        oil: i32,
        done: &mut [bool; 128],
    ) -> bool {
        let mut a3 = self.rings.get(s);
        let mut res = self.place_region_group(gr, s, reg, a3, oil);
        if !res {
            if done[reg as usize] {
                return false;
            }
            let helping = self.help.helping;
            self.help.helping = false;
            self.place_region_group(gr, s, reg, a3, oil);
            a3 = self.rings.get(s);
            let first = a3;
            loop {
                res = self.place_region_group(gr, s, reg, a3, oil);
                a3 = self.rings.get(s);
                if a3 == first || res {
                    break;
                }
            }
            if !res {
                a3 = self.rings.get(s);
                let first = a3;
                let cs = gr.coast_space;
                if cs != 0 {
                    gr.coast_space = cs - 1;
                    loop {
                        res = self.place_region_group(gr, s, reg, a3, oil);
                        a3 = self.rings.get(s);
                        if a3 == first || res {
                            break;
                        }
                    }
                    gr.coast_space = cs;
                }
                if !res {
                    self.rings.randomize(self.d);
                    if s > 1 {
                        a3 = self.rings.get(s - 1);
                        let first = a3;
                        loop {
                            res = self.place_region_group(gr, s - 1, reg, a3, oil);
                            a3 = self.rings.get(s - 1);
                            if a3 == first || res {
                                break;
                            }
                        }
                        self.rings.randomize(self.d);
                        if res && s == 2 {
                            a3 = self.rings.get(1);
                            let first = a3;
                            loop {
                                let r2 = self.place_region_group(gr, s - 1, reg, a3, oil);
                                a3 = self.rings.get(s - 1);
                                if a3 == first || r2 {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            self.help.helping = helping;
        }
        if res {
            done[reg as usize] = true;
        }
        res
    }
}

/// **`TerrainGroups::add_doobers@006a1540`**: writes nothing a part walks;
/// its draws are the bush rolls on forest edges, then the rock rolls at
/// mountain feet, each with a size draw when the roll passes.
fn add_doobers(p: &mut Pass<'_>, ts: &TilesetGroups) {
    let (xs, ys) = (p.g.xs, p.g.ys);
    for x in 1..xs - 1 {
        for y in 1..ys - 1 {
            let f = p.flags(x, y);
            if f & flag::FOREST == 0 || f & 0x1000 != 0 {
                continue;
            }
            let south = p.flags(x, y + 1) & flag::FOREST == 0;
            let east = p.flags(x + 1, y) & flag::FOREST == 0;
            if south && p.d.get(site::BUSH_S) % 100 < ts.bush_clump_prob {
                p.d.get(site::BUSH_S_SIZE);
            }
            if east && p.d.get(site::BUSH_E) % 100 < ts.bush_clump_prob {
                p.d.get(site::BUSH_E_SIZE);
            }
        }
    }
    for x in 1..xs - 1 {
        for y in 1..ys - 1 {
            let f = p.flags(x, y);
            if p.ocean(x, y) || f & 0x38 != 0 || f & 0x1000 != 0 {
                continue;
            }
            for k in [8, 2] {
                if p.flags(x + MOVE_X[k], y + MOVE_Y[k]) & flag::MOUNTAIN == 0 {
                    continue;
                }
                if p.d.get(site::ROCK) % 100 < ts.mnt_rock_clump_prob {
                    p.d.get(site::ROCK_SIZE);
                }
                break;
            }
        }
    }
}

/// **`TerrainGroups::treeify_mountains@006a1cc0`**: forest on the cells
/// mountain art spills into, and forest over mountain on a mountain cell
/// whose east (or, off the grid, south) neighbour carries no mountain art.
fn treeify_mountains(p: &mut Pass<'_>, prob: i32) {
    for x in 0..p.g.xs {
        for y in 0..p.g.ys {
            let i = p.at(x, y);
            let f = p.g.cells[i].flags;
            if p.mountain_tiles(x, y) && f & flag::MOUNTAIN == 0 {
                let mtn = |u: i32, v: i32| p.inb(u, v) && p.flags(u, v) & flag::MOUNTAIN != 0;
                if mtn(x, y + 1) || mtn(x + 1, y) || f & 0x404 != 0 {
                    continue;
                }
                if p.d.get(site::FRINGE_A) % 100 < prob {
                    p.g.cells[i].flags = (f & 0xffc3) | flag::FOREST;
                    p.g.cells[i].land = 0;
                }
                continue;
            }
            if f & flag::MOUNTAIN == 0 {
                continue;
            }
            for (u, v) in [(x + 1, y), (x, y + 1)] {
                if !p.inb(u, v) || p.mountain_tiles(u, v) {
                    continue;
                }
                if f & 0x404 == 0 && p.d.get(site::FRINGE_B) % 100 < prob {
                    p.g.cells[i].flags = (f & 0xffc3) | flag::FOREST | flag::MOUNTAIN;
                    p.g.cells[i].land = 0;
                }
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `randomize_mountains` on run1000's two words (`docs/MAPGEN.md` §7):
    /// the small ring holds one range and draws nothing; 1131 % 8 seats the
    /// medium ring on range 11 and 13530 % 7 the large on range 0, and
    /// `get_range` then walks each ring from its head, wrapping.
    #[test]
    fn the_rings_seat_on_the_traced_words() {
        let mut r = Rings::stock();
        let mut d = Draws::new(0x8c60_9701);
        r.randomize(&mut d);
        assert_eq!(
            d.log,
            [(site::MTN_MED, 0x8c60_9701), (site::MTN_LARGE, 0x989a_046c)]
        );
        assert_eq!((r.get(1), r.get(1)), (15, 15));
        assert_eq!((r.get(2), r.get(2), r.get(2), r.get(2)), (11, 10, 9, 8));
        assert_eq!(r.get(2), 7);
        assert_eq!(r.get(2), 14, "the medium ring wraps to its head");
        assert_eq!((r.get(3), r.get(3)), (0, 6), "the large ring wraps at once");
        assert_eq!(r.get(4), 0);
    }

    /// `place_region_group`'s walk is a maximal 16-bit Galois LFSR — the
    /// listing's 0x16800 is applied before the shift, so the taps are
    /// 0xB400: from any nonzero state it visits every state below 2^16 once
    /// before returning.
    #[test]
    fn the_region_walk_visits_every_state() {
        let (s0, mut s, mut n) = (1u32, 1u32, 0u32);
        loop {
            s = if s & 1 != 0 { s ^ 0x16800 } else { s } >> 1;
            n += 1;
            if s == s0 {
                break;
            }
            assert!(s != 0 && s < 1 << 16 && n < 1 << 16);
        }
        assert_eq!(n, (1 << 16) - 1);
    }
}
