//! **`Terrain::check_and_set_halfland_wcoord@0084dec0`'s body** — the
//! half-land writes a corner height decides (`docs/MAPGEN.md` §16, §10,
//! §15 steps 6 and 7): `Terrain.cpp` 706 → 720 on a map whose gate opens.
//!
//! On each gated cell (`CoordInfo.flags & 0x84`), in this order:
//! - `Terrain::clear_halfland_wcoord@0084e500`: the cell's wet tiles dry
//!   (`World::clear_waterhalf@006b1bd0`), HALFLAND (0x100) and every
//!   tile's 0x400 go, and a cell that was **not** HALFLAND loses its
//!   `region2` (the listing tests `~flags`, 0x84e534).
//! - The region scan over `move[1..25]`: the first neighbour not HALFLAND
//!   whose `region` is not the cell's stops it; each HALFLAND neighbour
//!   whose `region2` is not the cell's region is kept, the last one
//!   winning. The first is the other region, else the second.
//! - Each sub-tile with all four corners strictly below −30.0f is counted
//!   and, when an other region exists, handed to
//!   `Terrain::mark_halfwater@0084ece0`.
//! - Any water: `spot_update_land(x, y, 1, 0)`. None: the dry arm — `land`
//!   2 becomes 0 at `COAST_BASE`, flags `& 0xffc3`, `CoordInfo` `& ~4`,
//!   0x4 to the first orthogonal neighbour that is wet and not yet 0x4
//!   (queued for a second visit), `verify_land`, and the region rescan.
//!
//! `Terrain::verify_land@0084d8a0` walks the eight neighbours against
//! `TileSet::is_combo_acceptable@0087abe0` and recurses. Its depth counter
//! is a static that a refusal past 500 never decrements, so it is carried
//! across the pass ([`Pass::depth`]).
//!
//! The `Regions` lists `mark_halfwater` and the rescan keep are not
//! walked by any checksum part and are not held. `move_ocean_goody` moves
//! a goody box or oil off a cell turning wet; a good other than oil is
//! [`Unported::Goody`].
//!
//! The corner test reads the heights as [`Single`]s and compares them
//! as integers; no float.

use crate::mapgen::MapGrid;
use crate::single::Single;
use crate::terrain_groups::{OIL_GOOD, tile_at};
use crate::terrain_init::{HalfLand, MOVE, ORTHOG, coord};

/// `check_and_set_halfland_wcoord`'s threshold (`__real_c1f00000`), −30.
pub const WATERLINE: Single = Single(0xc1f0_0000);

/// The tileset's numbers `is_combo_acceptable` reads: `COAST_BASE` and
/// `num_tiles` per land (`None` where the loader is not read for it).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Combo {
    pub coast_base: i32,
    pub num_tiles: [Option<i32>; 4],
}

/// An arm of the half-land pass this port does not build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unported {
    /// `move_ocean_goody` would move something off these cells.
    Goody(Vec<(i32, i32)>),
    /// `is_combo_acceptable` read a `num_tiles` entry not loaded.
    NumTiles(usize),
    /// `check_and_set_halfland`'s relabel of a `BASELAND` cell standing in
    /// a water region (above 0x40) whose `region2` is 0: its neighbour
    /// search reads the `Regions` lists' `add_unique`.
    Relabel(Vec<(i32, i32)>),
}

/// **`TileSet::is_combo_acceptable@0087abe0`**: whether land `(l1, s1)`
/// may stand beside `(l2, s2)`; 2 is "yes, and the second is the lower".
pub fn is_combo_acceptable(c: &Combo, l1: i32, s1: i32, l2: i32, s2: i32) -> Result<i32, usize> {
    let tiles = |l: i32| {
        c.num_tiles
            .get(l as usize)
            .copied()
            .flatten()
            .ok_or(l as usize)
    };
    if l1 == l2 {
        if (s2 - s1).abs() < 2 {
            return Ok(1 + (s2 <= s1) as i32);
        }
        return Ok(0);
    }
    if l1 == 0 {
        if l2 == 1 {
            return Ok((s1 == c.coast_base) as i32);
        }
    } else if l1 == 1 && l2 == 0 {
        return Ok(if s2 == c.coast_base { 2 } else { 0 });
    }
    if l2 == 2 {
        return Ok(1);
    }
    if l2 < l1 {
        if s2 == tiles(l2)? - 1 {
            return Ok(if s1 != 0 { 0 } else { 2 });
        }
    } else if s2 == 0 {
        return Ok((s1 == tiles(l1)? - 1) as i32);
    }
    Ok(0)
}

/// The half-land pass's state: the grid, the `CoordInfo` flags, the
/// 706 heights, and what the pass builds.
pub struct Pass<'a> {
    pub g: &'a mut MapGrid,
    pub coord: &'a mut [u16],
    pub z: &'a [Single],
    pub combo: Combo,
    /// Parts 10–12's lists.
    pub lists: HalfLand,
    /// `verify_land`'s static `depth`.
    pub depth: i32,
    /// The cells `_wcoord`'s dry arm queued (`y·xs + x`).
    queue: Vec<i32>,
    goody: Vec<(i32, i32)>,
    unread: Option<usize>,
}

impl<'a> Pass<'a> {
    pub fn new(g: &'a mut MapGrid, coord: &'a mut [u16], z: &'a [Single], combo: Combo) -> Self {
        Pass {
            g,
            coord,
            z,
            combo,
            lists: HalfLand::default(),
            depth: 0,
            queue: Vec::new(),
            goody: Vec::new(),
            unread: None,
        }
    }

    /// `verify_land`'s `depth` as an earlier pass left it.
    pub fn with_depth(mut self, depth: i32) -> Self {
        self.depth = depth;
        self
    }

    fn inb(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.g.xs && y < self.g.ys
    }

    fn at(&self, x: i32, y: i32) -> usize {
        (y * self.g.xs + x) as usize
    }

    fn below(&self, u: i32, v: i32) -> bool {
        let stride = 4 * self.g.xs + 1;
        WATERLINE.gt(self.z[(v * stride + u) as usize])
    }

    fn combo(&mut self, l1: i32, s1: i32, l2: i32, s2: i32) -> i32 {
        match is_combo_acceptable(&self.combo, l1, s1, l2, s2) {
            Ok(v) => v,
            Err(l) => {
                self.unread.get_or_insert(l);
                0
            }
        }
    }

    /// `land` and `land_sub` with `spot_update_land`'s and
    /// `verify_land`'s guard: a negative value is not written.
    fn set_land(&mut self, i: usize, land: i32, sub: i32) {
        let c = &mut self.g.cells[i];
        if sub >= 0 {
            c.land_sub = sub as u8;
        }
        if land >= 0 {
            c.land = land as i8;
        }
    }

    /// **`Terrain::verify_land@0084d8a0`**.
    pub fn verify_land(&mut self, x: i32, y: i32, land: &mut i32, sub: &mut i32) {
        self.depth += 1;
        if self.depth > 500 {
            // The original returns here without the decrement.
            return;
        }
        let mut marked = [false; 9];
        for (k, mark) in marked.iter_mut().enumerate().skip(1) {
            let (nx, ny) = (x + MOVE[k].0, y + MOVE[k].1);
            if !self.inb(nx, ny) {
                continue;
            }
            let n = self.at(nx, ny);
            let (nl, ns) = (self.g.cells[n].land as i32, self.g.cells[n].land_sub as i32);
            let l1 = *land;
            if self.combo(l1, *sub, nl, ns) != 0 {
                continue;
            }
            if nl == 1 || nl == 2 {
                let cb = self.combo.coast_base;
                let i = self.at(x, y);
                self.set_land(i, 0, cb);
                let (mut l, mut s) = (0, cb);
                self.verify_land(x, y, &mut l, &mut s);
                *land = l;
                *sub = s;
                continue;
            }
            let mut step = ns;
            if (*sub - ns).abs() > 1 {
                step = if *sub <= ns { ns - 1 } else { ns + 1 };
            }
            if l1 == 0 {
                if ns != step && ns != *sub {
                    self.set_land(n, nl, step);
                    *mark = true;
                }
            } else if l1 == 1 || l1 == 2 {
                let cb = self.combo.coast_base;
                self.set_land(n, 0, cb);
                *mark = true;
            }
        }
        for (k, &mark) in marked.iter().enumerate().skip(1) {
            let (nx, ny) = (x + MOVE[k].0, y + MOVE[k].1);
            if mark && self.inb(nx, ny) {
                let n = self.at(nx, ny);
                let (mut l, mut s) = (self.g.cells[n].land as i32, self.g.cells[n].land_sub as i32);
                self.verify_land(nx, ny, &mut l, &mut s);
            }
        }
        self.depth -= 1;
    }

    /// **`Terrain::spot_update_land@0084fa50`**'s world half.
    fn spot_update_land(&mut self, x: i32, y: i32, mut land: i32, mut sub: i32) {
        if !self.inb(x, y) {
            return;
        }
        let i = self.at(x, y);
        let c = self.g.cells[i];
        if c.land as i32 != land || c.land_sub as i32 != sub {
            self.set_land(i, land, sub);
            self.verify_land(x, y, &mut land, &mut sub);
        }
    }

    /// `ObjectsData::find_good_at@0065bec0` with its last argument 0: the
    /// index of a live good other than oil at the end of the cell's `down`
    /// chain, else −1. Before the first frame the chain is the cell's own.
    fn find_good_at(&self, x: i32, y: i32) -> i32 {
        let c = &self.g.cells[self.at(x, y)];
        if c.down != -2 {
            return -1;
        }
        match self.g.goods.get(c.down_who as usize) {
            Some(Some((ty, _, _))) if *ty != OIL_GOOD => c.down_who as i32,
            _ => -1,
        }
    }

    /// `Terrain::has_south_forest@00850060`: forest (0x20) at `(x+1, y+1)`,
    /// `(x, y+1)` or `(x+1, y)`.
    fn has_south_forest(&self, x: i32, y: i32) -> bool {
        [(1, 1), (0, 1), (1, 0)].iter().any(|&(dx, dy)| {
            self.inb(x + dx, y + dy) && self.g.cells[self.at(x + dx, y + dy)].flags & 0x20 != 0
        })
    }

    /// `TerrainOut::is_clear_land@00874b60` for the masks `move_ocean_goody`
    /// passes, `0x80` and `0x88`: none of 0x10, 0x20, 0x8, 0x800, 0x1000,
    /// 0x40 or 0x8000, no good, not blocked, `CoordInfo` without 0x4, and a
    /// wet cell only under 0x8.
    fn is_clear_land(&self, x: i32, y: i32, mask: u32) -> bool {
        let i = self.at(x, y);
        let c = &self.g.cells[i];
        let u = c.flags;
        if u & 0x10 != 0 || u & 0x20 != 0 || u & 0x8 != 0 || u & 0x800 != 0 {
            return false;
        }
        if u & 0x100 == 0 && (c.land == 1 || c.land == 2) && mask & 8 == 0 {
            return false;
        }
        if u & 0x1000 != 0 || u & 0x40 != 0 || u & 0x8000 != 0 || self.find_good_at(x, y) >= 0 {
            return false;
        }
        c.blocked == 0 && self.coord[i] & coord::HALF == 0
    }

    /// `World::set_oil_at@006b2a10`: any live oil good on the cell is
    /// killed, then on the flag and a fresh oil good, off the flag cleared.
    /// The kill is `Good::close@0066d860`: the cell loses flag 1 and
    /// `World::clear_down@006b3ad0` writes `down` and `down_who` −1, so an
    /// oil cell's `down_who` 0 turns −1 (part 5 at 720: the two oil cells'
    /// `0x0000 → 0xffff` are the whole of the Adler difference, +1020 low,
    /// 7729 high). The footprint's tile 0x200 clear is not held; part 6
    /// does not move for it on Great Lakes.
    fn set_oil_at(&mut self, x: i32, y: i32, on: bool) {
        let i = self.at(x, y);
        for k in 0..self.g.goods.len() {
            if self.g.goods[k] == Some((OIL_GOOD, x, y)) {
                self.g.goods[k] = None;
                let c = &mut self.g.cells[i];
                c.flags &= 0xfffe;
                if c.down < 0 {
                    c.down = -1;
                    c.down_who = -1;
                }
            }
        }
        if on {
            self.g.cells[i].flags |= 0x800;
            self.g.init_good(OIL_GOOD, x, y);
        } else {
            self.g.cells[i].flags &= 0xf7ff;
        }
    }

    /// **`Terrain::move_ocean_goody@0084aec0`**: what stands on a cell
    /// turning wet moves to a neighbour. The neighbours' lands are read
    /// once, at entry (`move[1..49]`, −1 off the map); each search skips
    /// the map's edge row and column (the listing compares both
    /// coordinates with `xs − 1`).
    /// - A goody box (`down` −3, on a cell not already wet): `Item::close`
    ///   takes the cell's 0x8000 and its `down`, and the first of
    ///   `move[9..25]` that is dry `BASELAND`, not HALFLAND, free of 0x10,
    ///   0x28, 0x800, 0x1000, 0x40 and 0x8000, holds no good, is not
    ///   blocked, has `CoordInfo` without 0x4 and no forest south of it
    ///   takes it (`Item::init`'s 0x8000, `set_down(−3, item)`).
    /// - Oil (0x800): the cell loses its oil and 0x8; the first of
    ///   `move[9..49]` of the cell's land class (water counts `SANDY` and
    ///   `OCEAN` alike), not HALFLAND, [`Self::is_clear_land`] and with no
    ///   forest south of it gains oil and 0x8.
    /// - A good: not reached on Great Lakes ([`Unported::Goody`]).
    fn move_ocean_goody(&mut self, x: i32, y: i32) {
        let c = self.g.cells[self.at(x, y)];
        let wet = c.flags & 0x100 == 0 && (c.land == 1 || c.land == 2);
        let goody = if !wet && c.down == -3 {
            c.down_who as i32
        } else {
            -1
        };
        let oil = c.flags & 0x800 != 0;
        let good = self.find_good_at(x, y);
        if goody == -1 && !oil && good < 0 {
            return;
        }
        let lands: Vec<i32> = MOVE
            .iter()
            .map(|&(dx, dy)| {
                if self.inb(x + dx, y + dy) {
                    self.g.cells[self.at(x + dx, y + dy)].land as i32
                } else {
                    -1
                }
            })
            .collect();
        let edge = |nx: i32, ny: i32, xs: i32| nx == 0 || ny == 0 || nx == xs - 1 || ny == xs - 1;
        if goody != -1 {
            let i = self.at(x, y);
            self.g.cells[i].flags &= 0x7fff;
            self.g.cells[i].down = -1;
            self.g.cells[i].down_who = -1;
            for k in 9..25 {
                let (nx, ny) = (x + MOVE[k].0, y + MOVE[k].1);
                if lands[k] != 0 || edge(nx, ny, self.g.xs) {
                    continue;
                }
                let n = self.at(nx, ny);
                let u = self.g.cells[n].flags;
                if u & (0x100 | 0x10 | 0x28 | 0x800 | 0x1000 | 0x40 | 0x8000) != 0
                    || self.find_good_at(nx, ny) >= 0
                    || self.g.cells[n].blocked != 0
                    || self.coord[n] & coord::HALF != 0
                    || self.has_south_forest(nx, ny)
                {
                    continue;
                }
                let m = &mut self.g.cells[n];
                m.flags |= 0x8000;
                m.down = -3;
                m.down_who = goody as i16;
                break;
            }
        }
        if good >= 0 {
            self.goody.push((x, y));
        }
        if oil {
            self.set_oil_at(x, y, false);
            let i = self.at(x, y);
            self.g.cells[i].flags &= 0xfff7;
            let land = self.g.cells[i].land as i32;
            let (a, b, mask) = if land == 1 || land == 2 {
                (2, 1, 0x88)
            } else {
                (land, land, 0x80)
            };
            for k in 9..49 {
                let (nx, ny) = (x + MOVE[k].0, y + MOVE[k].1);
                if lands[k] == -1 || edge(nx, ny, self.g.xs) || (lands[k] != a && lands[k] != b) {
                    continue;
                }
                if self.g.cells[self.at(nx, ny)].flags & 0x100 != 0
                    || !self.is_clear_land(nx, ny, mask)
                    || self.has_south_forest(nx, ny)
                {
                    continue;
                }
                self.set_oil_at(nx, ny, true);
                let n = self.at(nx, ny);
                self.g.cells[n].flags |= 0x8;
                return;
            }
        }
    }

    /// `WorldData::num_waterhalf@006b4db0`: the wet tiles of a HALFLAND
    /// cell, 0 for any other.
    fn num_waterhalf(&self, x: i32, y: i32) -> u32 {
        if self.g.cells[self.at(x, y)].flags & 0x100 == 0 {
            return 0;
        }
        (0..16)
            .filter(|k| {
                self.g.tiles[tile_at(self.g, 4 * x + (k & 3), 4 * y + (k >> 2))] & 0x30 == 0x20
            })
            .count() as u32
    }

    /// **`Terrain::mark_halfwater@0084ece0`**: sub-tile `(tx, ty)` of cell
    /// `(x, y)` turns wet; `region` is the cell's region before the pass,
    /// `other` the scan's.
    fn mark_halfwater(&mut self, x: i32, y: i32, tx: i32, ty: i32, region: i32, other: i32) {
        let i = self.at(x, y);
        let first = self.g.cells[i].flags & 0x100 == 0;
        if first {
            self.move_ocean_goody(x, y);
        }
        let t = tile_at(self.g, tx, ty);
        self.g.tiles[t] = self.g.tiles[t] & 0xffef | 0x20;
        self.g.cells[i].flags |= 0x100;
        let full = self.num_waterhalf(x, y) == 16;
        let (land, sub) = (self.g.cells[i].land, self.g.cells[i].land_sub);
        let (lr, wr) = if region > 0x40 {
            (other, region)
        } else {
            (region, other)
        };
        if !first {
            if !full {
                return;
            }
            let c = &mut self.g.cells[i];
            c.flags &= !0x100;
            c.region = wr as i16;
            c.region2 = 0;
            c.land = 2;
            c.land_sub = 0;
            let (mut l, mut s) = (2, 0);
            self.verify_land(x, y, &mut l, &mut s);
            return;
        }
        let c = &mut self.g.cells[i];
        c.region = lr as i16;
        c.region2 = wr as i16;
        self.lists.locs.push((x, y));
        self.lists.types.push(land as i32);
        self.lists.subtypes.push(sub as i32);
        let c = &mut self.g.cells[i];
        c.land = 0;
        c.land_sub = 0;
        let (mut l, mut s) = (0, 0);
        self.verify_land(x, y, &mut l, &mut s);
    }

    /// **`Terrain::clear_halfland_wcoord@0084e500`**.
    fn clear_halfland_wcoord(&mut self, x: i32, y: i32) {
        let i = self.at(x, y);
        let was = self.g.cells[i].flags & 0x100 != 0;
        for k in 0..16 {
            let t = tile_at(self.g, 4 * x + (k & 3), 4 * y + (k >> 2));
            if self.g.tiles[t] & 0x30 == 0x20 {
                self.g.tiles[t] &= 0xffcf;
            }
        }
        self.g.cells[i].flags &= !0x100;
        for k in 0..16 {
            let t = tile_at(self.g, 4 * x + (k & 3), 4 * y + (k >> 2));
            self.g.tiles[t] &= 0xfbff;
        }
        if !was {
            self.g.cells[i].region2 = 0;
        }
    }

    /// **`Terrain::check_and_set_halfland_wcoord@0084dec0`**.
    pub fn wcoord(&mut self, x: i32, y: i32) {
        let i = self.at(x, y);
        if self.coord[i] & coord::GATE == 0 {
            return;
        }
        self.clear_halfland_wcoord(x, y);
        let region = self.g.cells[i].region as i32;
        let (mut near, mut half) = (-1, -1);
        for &(dx, dy) in &MOVE[1..25] {
            let (nx, ny) = (x + dx, y + dy);
            if !self.inb(nx, ny) {
                continue;
            }
            let n = &self.g.cells[self.at(nx, ny)];
            if n.flags & 0x100 == 0 {
                if n.region as i32 != region {
                    near = n.region as i32;
                    break;
                }
            } else if n.region2 as i32 != region {
                half = n.region2 as i32;
            }
        }
        let mut water = 0;
        for sy in 0..4 {
            for sx in 0..4 {
                let (tx, ty) = (4 * x + sx, 4 * y + sy);
                if !(self.below(tx, ty)
                    && self.below(tx + 1, ty)
                    && self.below(tx, ty + 1)
                    && self.below(tx + 1, ty + 1))
                {
                    continue;
                }
                water += 1;
                let other = if near != -1 { near } else { half };
                if other != -1 {
                    self.mark_halfwater(x, y, tx, ty, region, other);
                    near = other;
                }
            }
        }
        if water != 0 {
            self.spot_update_land(x, y, 1, 0);
            return;
        }
        let c = self.g.cells[i];
        let (mut land, mut sub) = (c.land as i32, c.land_sub as i32);
        if land == 2 {
            land = 0;
            sub = self.combo.coast_base;
        }
        self.set_land(i, land, sub);
        self.g.cells[i].flags &= 0xffc3;
        self.coord[i] &= !coord::HALF;
        for &(dx, dy) in &ORTHOG[1..5] {
            let (nx, ny) = (x + dx, y + dy);
            if !self.inb(nx, ny) {
                continue;
            }
            let n = self.at(nx, ny);
            if self.g.cells[n].land != 0 && self.g.cells[n].flags & 0x4 == 0 {
                self.g.cells[n].flags |= 0x4;
                self.coord[n] |= coord::HALF;
                self.queue.push(n as i32);
                break;
            }
        }
        self.verify_land(x, y, &mut land, &mut sub);
        let region = self.g.cells[i].region;
        for &(dx, dy) in &MOVE[1..25] {
            let (nx, ny) = (x + dx, y + dy);
            if !self.inb(nx, ny) {
                continue;
            }
            let n = self.g.cells[self.at(nx, ny)];
            if n.flags & 0x100 == 0 && n.region != region {
                self.g.cells[i].region = n.region;
                self.move_ocean_goody(x, y);
                break;
            }
        }
    }

    /// `check_and_set_halfland`'s wet marking after each visit: a cell
    /// not HALFLAND whose `land` is `SANDY` or `OCEAN` marks its sixteen
    /// tiles `& 0xffef | 0x20`.
    fn mark_wet(&mut self, x: i32, y: i32) {
        let c = self.g.cells[self.at(x, y)];
        if c.flags & 0x100 == 0 && (c.land == 1 || c.land == 2) {
            for k in 0..16 {
                let t = tile_at(self.g, 4 * x + (k & 3), 4 * y + (k >> 2));
                self.g.tiles[t] = self.g.tiles[t] & 0xffef | 0x20;
            }
        }
    }

    /// **`Terrain::spot_update_land_batch@0084d540`**: each listed cell
    /// whose `land` or `land_sub` differs takes `land_sub`, then `land`
    /// (a negative value not written); then `verify_land(land, sub)` on
    /// every listed cell, repeats included, each from the batch's own pair.
    /// The render lists it rebuilds are not walked by any part.
    pub fn spot_update_land_batch(
        &mut self,
        cells: &[(i32, i32)],
        land: i32,
        sub: i32,
    ) -> Result<(), Unported> {
        for &(x, y) in cells {
            let i = self.at(x, y);
            let c = self.g.cells[i];
            if c.land as i32 != land || c.land_sub as i32 != sub {
                self.set_land(i, land, sub);
            }
        }
        for &(x, y) in cells {
            let (mut l, mut s) = (land, sub);
            self.verify_land(x, y, &mut l, &mut s);
        }
        self.refusals()
    }

    /// **`Terrain::convert_beach_to_sandy@0084b8e0`** (815 → 5012), y outer,
    /// x inner: a `BASELAND` cell with an `OCEAN` cell among `move[1..9]`
    /// goes through `spot_update_land(x, y, 1, 0)` and clears `CoordInfo`
    /// 0x2000. Returns how many cells it entered.
    pub fn convert_beach_to_sandy(&mut self) -> Result<usize, Unported> {
        let mut entered = 0;
        for y in 0..self.g.ys {
            for x in 0..self.g.xs {
                if self.g.cells[self.at(x, y)].land != 0 {
                    continue;
                }
                let beach = MOVE[1..9].iter().any(|&(dx, dy)| {
                    self.inb(x + dx, y + dy) && self.g.cells[self.at(x + dx, y + dy)].land == 2
                });
                if beach {
                    entered += 1;
                    self.spot_update_land(x, y, 1, 0);
                    let i = self.at(x, y);
                    self.coord[i] &= !coord::BEACH;
                }
            }
        }
        self.refusals().map(|()| entered)
    }

    fn refusals(&mut self) -> Result<(), Unported> {
        if let Some(l) = self.unread {
            return Err(Unported::NumTiles(l));
        }
        if !self.goody.is_empty() {
            return Err(Unported::Goody(std::mem::take(&mut self.goody)));
        }
        Ok(())
    }

    /// `check_and_set_halfland(0)`'s two sweeps: every cell (y outer, x
    /// inner), then the dry arm's queue until it stays empty.
    pub fn sweeps(&mut self) -> Result<(), Unported> {
        for y in 0..self.g.ys {
            for x in 0..self.g.xs {
                self.wcoord(x, y);
                self.mark_wet(x, y);
            }
        }
        while !self.queue.is_empty() {
            let batch = std::mem::take(&mut self.queue);
            for idx in batch {
                let (x, y) = (idx % self.g.xs, idx / self.g.xs);
                self.wcoord(x, y);
                self.mark_wet(x, y);
            }
        }
        self.refusals()
    }
}

/// **`TerrainOil::create_doobers@008a28e0`'s cell write** (720 → 738): a
/// wet cell (not HALFLAND, `land` `SANDY` or `OCEAN`) carrying rock (0x8)
/// loses it — where `move_ocean_goody` moved oil onto water at 720. Its
/// rocky patches' heights are [`crate::map_heights::create_doobers`]'.
pub fn create_doobers_flags(g: &mut MapGrid) {
    for c in &mut g.cells {
        if c.flags & 0x8 != 0 && c.flags & 0x100 == 0 && (c.land == 1 || c.land == 2) {
            c.flags &= 0xfff7;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain_init::coord_flags;

    const COMBO: Combo = Combo {
        coast_base: 3,
        num_tiles: [Some(4), Some(1), Some(1), None],
    };

    /// A `xs × ys` grid of heights all at `h` (as `Single` bits).
    fn flat(xs: i32, ys: i32, h: Single) -> Vec<Single> {
        vec![h; ((4 * xs + 1) * (4 * ys + 1)) as usize]
    }

    const DRY: Single = Single(0x4120_0000); // 10
    const WET: Single = Single(0xc220_0000); // −40

    #[test]
    fn the_combo_table_reads_its_lands() {
        let c = COMBO;
        assert_eq!(is_combo_acceptable(&c, 0, 1, 0, 2), Ok(1));
        assert_eq!(is_combo_acceptable(&c, 0, 2, 0, 1), Ok(2));
        assert_eq!(is_combo_acceptable(&c, 0, 0, 0, 2), Ok(0));
        assert_eq!(is_combo_acceptable(&c, 0, 3, 1, 0), Ok(1));
        assert_eq!(is_combo_acceptable(&c, 0, 2, 1, 0), Ok(0));
        assert_eq!(is_combo_acceptable(&c, 1, 0, 0, 3), Ok(2));
        assert_eq!(is_combo_acceptable(&c, 1, 0, 2, 0), Ok(1));
        // A water centre: a land neighbour must sit at num_tiles[0] − 1.
        assert_eq!(is_combo_acceptable(&c, 2, 0, 0, 3), Ok(2));
        assert_eq!(is_combo_acceptable(&c, 2, 0, 0, 2), Ok(0));
        assert_eq!(is_combo_acceptable(&c, 2, 1, 0, 3), Ok(0));
        // Past the special cases, the lower land's last sub-tile decides.
        assert_eq!(is_combo_acceptable(&c, 0, 3, 3, 0), Ok(1));
        assert_eq!(is_combo_acceptable(&c, 3, 0, 1, 0), Ok(2));
        assert_eq!(is_combo_acceptable(&c, 3, 1, 1, 0), Ok(0));
        // A `num_tiles` entry not loaded is named, not guessed.
        assert_eq!(is_combo_acceptable(&c, 3, 0, 0, 1), Ok(0));
        assert_eq!(is_combo_acceptable(&c, 0, 1, 3, 0), Ok(0));
        assert_eq!(is_combo_acceptable(&c, 1, 0, 3, 0), Ok(1));
        let unread = Combo {
            num_tiles: [Some(4), None, None, None],
            ..c
        };
        assert_eq!(is_combo_acceptable(&unread, 1, 0, 3, 0), Err(1));
    }

    /// A fully wet gated cell turns `OCEAN` (its sixteenth tile), then
    /// `spot_update_land(1, 0)` makes it `SANDY` and `verify_land` sets each
    /// `BASELAND` neighbour off `COAST_BASE` to it.
    #[test]
    fn a_wet_cell_turns_sandy_and_its_shore_to_coast_base() {
        let mut g = MapGrid::new(3, 3);
        for c in g.cells.iter_mut() {
            c.land_sub = 1;
            c.region = 1;
        }
        g.cells[4].land = 2;
        g.cells[4].region = 0x41;
        g.cells[3].flags = 0x4;
        let mut coord = coord_flags(&g);
        let mut z = flat(3, 3, DRY);
        for v in 4..=8 {
            for u in 4..=8 {
                z[(v * 13 + u) as usize] = WET;
            }
        }
        let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
        p.sweeps().expect("nothing unported");
        // The dry shore cell (0, 1) spreads 0x4 east and queues (1, 1),
        // which its sixteenth tile had left not HALFLAND: the second visit
        // is a first touch again and lists it again, as `SANDY` at 0.
        assert_eq!(p.lists.locs, vec![(1, 1), (1, 1)]);
        assert_eq!(
            (p.lists.types.clone(), p.lists.subtypes.clone()),
            (vec![2, 1], vec![1, 0])
        );
        let c = g.cells[4];
        assert_eq!((c.land, c.land_sub, c.flags & 0x100), (1, 0, 0));
        assert_eq!((c.region, c.region2), (0x41, 0));
        for (i, n) in g.cells.iter().enumerate() {
            if i != 4 {
                assert_eq!((n.land, n.land_sub), (0, 3), "cell {i}");
            }
        }
    }

    /// A gated cell with no water sub-tile: `OCEAN` becomes `BASELAND` at
    /// `COAST_BASE`, flags lose 0x3c, and 0x4 goes to the first orthogonal
    /// wet neighbour, which is queued and visited.
    #[test]
    fn a_dry_cell_spreads_0x4_to_its_first_wet_neighbour() {
        let mut g = MapGrid::new(3, 1);
        g.cells[0].flags = 0x4 | 0x8 | 0x20;
        g.cells[1].land = 2;
        g.cells[1].region = 0x41;
        g.cells[2].land = 2;
        g.cells[2].region = 0x41;
        let mut coord = coord_flags(&g);
        let z = flat(3, 1, DRY);
        let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
        p.sweeps().expect("nothing unported");
        assert_eq!(g.cells[0].flags, 0);
        assert_eq!(coord[0] & coord::HALF, 0);
        // Both lake cells are shallow, so gated: each dry visit makes it
        // `BASELAND` at 3 and hands 0x4 on. `verify_land` then grades the
        // subs one step at a time from (1, 0)'s 3 against (0, 0)'s 0.
        assert!(g.cells.iter().all(|c| c.land == 0 && c.flags & 0x4 == 0));
        let subs: Vec<u8> = g.cells.iter().map(|c| c.land_sub).collect();
        assert_eq!(subs, vec![1, 2, 3]);
    }

    /// `clear_halfland_wcoord`: a cell that was not HALFLAND loses its
    /// `region2`; one that was keeps it until the scan rewrites it.
    #[test]
    fn a_cell_not_halfland_loses_its_region2() {
        let mut g = MapGrid::new(1, 1);
        g.cells[0].flags = 0x4;
        g.cells[0].region2 = 7;
        let mut coord = coord_flags(&g);
        let z = flat(1, 1, DRY);
        let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
        p.clear_halfland_wcoord(0, 0);
        assert_eq!(g.cells[0].region2, 0);
        g.cells[0].flags |= 0x100;
        g.cells[0].region2 = 7;
        let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
        p.clear_halfland_wcoord(0, 0);
        assert_eq!((g.cells[0].region2, g.cells[0].flags & 0x100), (7, 0));
    }

    /// `verify_land`'s depth is a static the refusal past 500 never
    /// decrements: once it passes, every later call returns at once.
    #[test]
    fn verify_land_past_500_stays_refused() {
        let mut g = MapGrid::new(2, 1);
        g.cells[1].land_sub = 3;
        let mut coord = vec![0; 2];
        let z = flat(2, 1, DRY);
        let mut p = Pass::new(&mut g, &mut coord, &z, COMBO).with_depth(500);
        let (mut l, mut s) = (0, 0);
        p.verify_land(0, 0, &mut l, &mut s);
        assert_eq!(p.depth, 501);
        assert_eq!(g.cells[1].land_sub, 3, "a refused call writes nothing");
        let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
        let (mut l, mut s) = (0, 0);
        p.verify_land(0, 0, &mut l, &mut s);
        assert_eq!(p.depth, 0);
        assert_eq!(g.cells[1].land_sub, 2, "one step toward the centre's 0");
    }

    /// A goody box on a cell turning wet moves to the first clear
    /// `BASELAND` cell of `move[9..25]` off the map's edge; the old cell
    /// loses 0x8000 and its `down`.
    #[test]
    fn a_goody_moves_off_a_cell_turning_wet() {
        let mut g = MapGrid::new(7, 7);
        let at = |x: i32, y: i32| (y * 7 + x) as usize;
        let c = &mut g.cells[at(3, 3)];
        c.flags = 0x8000;
        c.down = -3;
        c.down_who = 5;
        let mut coord = vec![0; 49];
        let z = flat(7, 7, DRY);
        let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
        p.move_ocean_goody(3, 3);
        let old = g.cells[at(3, 3)];
        assert_eq!((old.flags, old.down, old.down_who), (0, -1, -1));
        // move[9] is (−1, −2) from the cell.
        let new = g.cells[at(2, 1)];
        assert_eq!((new.flags, new.down, new.down_who), (0x8000, -3, 5));
    }

    /// Oil on a cell turning wet: the cell loses 0x800, 0x8 and its good
    /// (`down_who` −1); the first water cell of `move[9..49]` clear of
    /// 0x8 takes it.
    #[test]
    fn oil_moves_to_water_and_its_old_cell_takes_down_who_minus_one() {
        let mut g = MapGrid::new(7, 7);
        for c in g.cells.iter_mut() {
            c.land = 2;
            c.down = -1;
        }
        let at = |x: i32, y: i32| (y * 7 + x) as usize;
        g.cells[at(3, 3)].flags = 0x800 | 0x8;
        g.init_good(OIL_GOOD, 3, 3);
        let mut coord = vec![0; 49];
        let z = flat(7, 7, DRY);
        let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
        p.move_ocean_goody(3, 3);
        let old = g.cells[at(3, 3)];
        assert_eq!((old.flags, old.down_who), (0, -1));
        assert_eq!(g.cells[at(2, 1)].flags, 0x808);
        assert_eq!(g.goods, vec![Some((OIL_GOOD, 2, 1))]);
        create_doobers_flags(&mut g);
        assert_eq!(g.cells[at(2, 1)].flags, 0x800, "738 takes the rock back");
    }

    /// A 5×5 grid in region 1 whose centre is gated and has one wet
    /// sub-tile, its first.
    fn centre_wet() -> (MapGrid, Vec<u16>, Vec<Single>) {
        let mut g = MapGrid::new(5, 5);
        for c in g.cells.iter_mut() {
            c.region = 1;
        }
        let mut coord = vec![0; 25];
        coord[12] = coord::HALF;
        let mut z = flat(5, 5, DRY);
        for v in 8..=9 {
            for u in 8..=9 {
                z[v * 21 + u] = WET;
            }
        }
        (g, coord, z)
    }

    /// The scan stops at the first neighbour not HALFLAND of another
    /// region: `move[1]`'s 5 wins over `move[2]`'s 6.
    #[test]
    fn the_scan_stops_at_the_first_other_region() {
        let (mut g, mut coord, z) = centre_wet();
        g.cells[6].region = 5; // (1, 1), move[1]
        g.cells[7].region = 6; // (2, 1), move[2]
        Pass::new(&mut g, &mut coord, &z, COMBO).wcoord(2, 2);
        assert_eq!((g.cells[12].region, g.cells[12].region2), (1, 5));
    }

    /// With no such neighbour, the last HALFLAND neighbour whose `region2`
    /// is not the cell's region wins: `move[2]`'s 8 over `move[1]`'s 7.
    #[test]
    fn the_last_halfland_region2_wins() {
        let (mut g, mut coord, z) = centre_wet();
        for (i, r2) in [(6, 7), (7, 8)] {
            g.cells[i].flags = 0x100;
            g.cells[i].region2 = r2;
        }
        Pass::new(&mut g, &mut coord, &z, COMBO).wcoord(2, 2);
        assert_eq!((g.cells[12].region, g.cells[12].region2), (1, 8));
    }

    /// The dry arm hands 0x4 to the first orthogonal wet neighbour only:
    /// `orthog[2]`, east, before `orthog[4]`, west.
    #[test]
    fn the_dry_arm_spreads_0x4_once() {
        let mut g = MapGrid::new(3, 1);
        g.cells[0].land = 2;
        g.cells[2].land = 2;
        let mut coord = vec![0, coord::HALF, 0];
        let z = flat(3, 1, DRY);
        let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
        p.wcoord(1, 0);
        assert_eq!(p.queue, vec![2]);
        assert_eq!((g.cells[0].flags & 0x4, g.cells[2].flags & 0x4), (0, 0x4));
    }

    /// Oil searches `move[9..49]`: with water only in the third ring, the
    /// third ring's first cell, `move[25]` (−3, −3), takes it.
    #[test]
    fn oil_reaches_the_third_ring() {
        let mut g = MapGrid::new(9, 9);
        for c in g.cells.iter_mut() {
            c.down = -1;
        }
        let at = |x: i32, y: i32| (y * 9 + x) as usize;
        for &(dx, dy) in &MOVE[25..49] {
            g.cells[at(4 + dx, 4 + dy)].land = 2;
        }
        g.cells[at(4, 4)].land = 2;
        g.cells[at(4, 4)].flags = 0x800 | 0x8;
        g.init_good(OIL_GOOD, 4, 4);
        let mut coord = vec![0; 81];
        let z = flat(9, 9, DRY);
        Pass::new(&mut g, &mut coord, &z, COMBO).move_ocean_goody(4, 4);
        assert_eq!(g.cells[at(1, 1)].flags, 0x808);
        assert_eq!(g.goods, vec![Some((OIL_GOOD, 1, 1))]);
    }

    /// A good other than oil is refused by name.
    #[test]
    fn a_good_on_a_cell_turning_wet_is_refused() {
        let mut g = MapGrid::new(3, 3);
        let n = g.init_good(2, 1, 1) as i16;
        g.cells[4].down = -2;
        g.cells[4].down_who = n;
        let mut coord = vec![0; 9];
        let z = flat(3, 3, DRY);
        let mut p = Pass::new(&mut g, &mut coord, &z, COMBO);
        p.move_ocean_goody(1, 1);
        assert_eq!(p.refusals(), Err(Unported::Goody(vec![(1, 1)])));
    }

    /// The beach: a `BASELAND` cell beside `OCEAN` turns `SANDY` at 0 and
    /// clears `CoordInfo` 0x2000; its `BASELAND` neighbours off
    /// `COAST_BASE` follow to it.
    #[test]
    fn a_beach_cell_turns_sandy() {
        let mut g = MapGrid::new(3, 1);
        g.cells[2].land = 2;
        g.cells[0].land_sub = 1;
        g.cells[1].land_sub = 2;
        let mut coord = vec![coord::BEACH; 3];
        let z = flat(3, 1, DRY);
        let entered = Pass::new(&mut g, &mut coord, &z, COMBO)
            .convert_beach_to_sandy()
            .expect("nothing unported");
        assert_eq!(entered, 1);
        assert_eq!((g.cells[1].land, g.cells[1].land_sub), (1, 0));
        assert_eq!(coord[1], 0);
        assert_eq!((g.cells[0].land, g.cells[0].land_sub), (0, 3));
    }
}
