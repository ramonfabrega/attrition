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

use crate::map_resources::{Footprint, close_good, place_good};
use crate::mapgen::MapGrid;
use crate::single::Single;
use crate::terrain_groups::{OIL_GOOD, tile_at};
use crate::terrain_init::{HalfLand, MOVE, ORTHOG, coord};

/// `check_and_set_halfland_wcoord`'s threshold (`__real_c1f00000`), −30.
pub use crate::map_heights::WATERLINE;

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
    /// The good types' footprints, per type, for `move_goody`'s good arm;
    /// without them the arm refuses ([`Unported::Goody`]).
    goods: Option<&'a [Footprint]>,
    /// The goods the good arm moved, from and to.
    pub moved_goods: Vec<((i32, i32), (i32, i32))>,
    /// The good arm's candidates that passed every test but the forest
    /// south of them, before the one it took: what the south-forest test
    /// decided (item 3890's measure of that arm).
    pub south_forest_refusals: usize,
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
            goods: None,
            moved_goods: Vec::new(),
            south_forest_refusals: 0,
        }
    }

    /// The good types' footprints `move_goody`'s good arm closes and
    /// reopens a good by.
    pub fn with_goods(mut self, goods: &'a [Footprint]) -> Self {
        self.goods = Some(goods);
        self
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
        c.blocked == 0 && self.coord[i] & coord::GATE_HALF == 0
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
    /// - A good: refused here ([`Unported::Goody`]); `move_goody` under the
    ///   types' footprints ([`Self::with_goods`]) moves it.
    fn move_ocean_goody(&mut self, x: i32, y: i32) {
        self.move_any(x, y, false);
    }

    /// **`Terrain::move_goody@0084a3d0`**, which the river troughs call:
    /// [`Self::move_ocean_goody`]'s body, except that the oil's new cell
    /// takes the rock bit only when the old one was dry, and — `TerrainOil`
    /// being initialised by then — its rocky patch
    /// (`TerrainOil::add_rocky_patch@008a2b30`), whose terraform is the
    /// caller's: the cell the oil moved to is returned.
    pub fn move_goody(&mut self, x: i32, y: i32) -> Result<Option<(i32, i32)>, Unported> {
        let to = self.move_any(x, y, true);
        self.refusals().map(|()| to)
    }

    /// **`move_goody`'s good arm** (0x84a6b3..0x84a844): the good in `slot`
    /// on `(x, y)` is closed ([`close_good`], vtable +0x90), and the first
    /// of `move[9..25]` that the box's arm would take — entry land 0, off
    /// the map's edge, dry `BASELAND` free of 0x100, 0x10, 0x28, 0x800,
    /// 0x1000, 0x40 and 0x8000, holding no good, not blocked, `CoordInfo`
    /// without 0x4, no forest south of it — takes a new good of its type at
    /// `snap_center` ([`place_good`]). No such cell leaves it closed.
    fn move_good(&mut self, x: i32, y: i32, slot: usize, lands: &[i32], fps: &[Footprint]) {
        let Some((ty, _, _)) = self.g.goods[slot] else {
            return;
        };
        let fp = fps[ty as usize];
        close_good(self.g, fp, slot);
        let edge = |nx: i32, ny: i32, xs: i32| nx == 0 || ny == 0 || nx == xs - 1 || ny == xs - 1;
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
                || self.coord[n] & coord::GATE_HALF != 0
            {
                continue;
            }
            if self.has_south_forest(nx, ny) {
                self.south_forest_refusals += 1;
                continue;
            }
            place_good(self.g, fp, ty, nx, ny);
            self.moved_goods.push(((x, y), (nx, ny)));
            break;
        }
    }

    fn move_any(&mut self, x: i32, y: i32, rocky: bool) -> Option<(i32, i32)> {
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
            return None;
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
                    || self.coord[n] & coord::GATE_HALF != 0
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
            match self.goods {
                Some(fps) if rocky => self.move_good(x, y, good as usize, &lands, fps),
                _ => self.goody.push((x, y)),
            }
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
                if !rocky || mask != 0x88 {
                    self.g.cells[n].flags |= 0x8;
                }
                return Some((nx, ny));
            }
        }
        None
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
        if self.coord[i] & coord::GATE_MASK == 0 {
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
        self.coord[i] &= !coord::GATE_HALF;
        for &(dx, dy) in &ORTHOG[1..5] {
            let (nx, ny) = (x + dx, y + dy);
            if !self.inb(nx, ny) {
                continue;
            }
            let n = self.at(nx, ny);
            if self.g.cells[n].land != 0 && self.g.cells[n].flags & 0x4 == 0 {
                self.g.cells[n].flags |= 0x4;
                self.coord[n] |= coord::GATE_HALF;
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
mod tests;
