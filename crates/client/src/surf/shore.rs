//! The shore the surf is laid along (item 4239; `docs/RENDERER.md` §3.27):
//! `TerrainOut::sort_waterline_wcoords@00872f30` and what it calls, read
//! from the client's own world.
//!
//! The original's names: a **wcoord** is a cell (768 units, one `WData`
//! record), a **tcoord** a tile (192 units, one `TData` mask). A chain is a
//! list of tcoord indices, `x + y · 4W`.

use sim::observe::TerrainScene;

/// `move_x`/`move_y` entries 0 to 8 (`0x00adcaf0`, `0x00adc400`): the
/// spot, then NW, N, NE, E, SE, S, SW, W.
const MOVE: [(i32, i32); 9] = [
    (0, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
];

/// `orthog_x`/`orthog_y` entries 1 to 4 (`0x00add250`, `0x00add210`).
const ORTHOG: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// `land_left_of_vector@00867730`'s table, by direction (`move` entry 1
/// to 8): the `move` entry of the tcoord on the step's left (`0x00b698d0`,
/// `0x00b69940`, read off the PE). NW looks at the spot itself.
const LEFT: [usize; 8] = [0, 8, 8, 2, 2, 4, 4, 6];

/// `WData.flags` `0x100`, the half-land cell.
const HALFLAND: u16 = 0x100;
/// `WData.flags` `0x4`.
const COAST: u16 = 0x4;

/// What the shore reads of the world: the cells' `WData` (flags, land,
/// region, region2) and the tiles' `TData`.
pub struct ShoreWorld<'a> {
    /// In cells.
    pub width: i32,
    pub height: i32,
    pub flags: &'a [u16],
    pub land: &'a [i8],
    pub region: &'a [i16],
    pub region2: &'a [i16],
    pub tiles: &'a [u16],
}

impl<'a> ShoreWorld<'a> {
    pub fn of(t: &'a TerrainScene) -> Self {
        ShoreWorld {
            width: t.width,
            height: t.height,
            flags: &t.flags,
            land: &t.land,
            region: &t.region,
            region2: &t.region2,
            tiles: &t.tiles,
        }
    }

    fn tw(&self) -> i32 {
        self.width * 4
    }

    fn th(&self) -> i32 {
        self.height * 4
    }

    fn cell(&self, cx: i32, cy: i32) -> usize {
        (cy * self.width + cx) as usize
    }

    fn tile_in(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.tw() && y < self.th()
    }

    /// The tile's surface is ocean (`TData & 0x30 == 0x20`).
    fn tile_ocean(&self, x: i32, y: i32) -> bool {
        self.tiles[(y * self.tw() + x) as usize] & 0x30 == 0x20
    }

    /// A whole cell of land: not half-land, not `COAST`, `land` 0.
    fn cell_land(&self, c: usize) -> bool {
        self.flags[c] & (HALFLAND | COAST) == 0 && self.land[c] == 0
    }

    /// `WorldData::is_tocean_slow@006b2400`.
    fn is_tocean_slow(&self, x: i32, y: i32) -> bool {
        let c = self.cell(x >> 2, y >> 2);
        (self.flags[c] & HALFLAND == 0 && matches!(self.land[c], 1 | 2)) || self.tile_ocean(x, y)
    }

    /// `TerrainOut::tcoord_is_surfline@00867470`: on an ocean tcoord with
    /// a land tcoord beside it, the sea region that land borders (above
    /// `0x40`); 0 otherwise. A half-land neighbour answers its cell's
    /// `region2` whatever it holds, as the original does.
    pub fn tcoord_is_surfline(&self, x: i32, y: i32) -> i32 {
        let c = self.cell(x >> 2, y >> 2);
        let ocean = if self.flags[c] & HALFLAND == 0 {
            self.land[c] != 0 || self.flags[c] & COAST != 0
        } else {
            self.is_tocean_slow(x, y)
        };
        if !ocean {
            return 0;
        }
        for (ox, oy) in ORTHOG {
            let (nx, ny) = (x + ox, y + oy);
            if !self.tile_in(nx, ny) {
                continue;
            }
            let (cx, cy) = (nx >> 2, ny >> 2);
            let n = self.cell(cx, cy);
            if self.flags[n] & HALFLAND == 0 {
                if !self.cell_land(n) {
                    continue;
                }
                for (mx, my) in MOVE {
                    let (ax, ay) = (cx + mx, cy + my);
                    if ax < 0 || ay < 0 || ax >= self.width || ay >= self.height {
                        continue;
                    }
                    let a = self.cell(ax, ay);
                    if self.region[a] > 0x40 {
                        return i32::from(self.region[a]);
                    }
                    if self.region2[a] > 0x40 {
                        return i32::from(self.region2[a]);
                    }
                }
            } else if !self.is_tocean_slow(nx, ny) {
                return i32::from(self.region2[n]);
            }
        }
        0
    }

    /// `TerrainOut::wcoord_has_surfline@00867690`.
    fn wcoord_has_surfline(&self, cx: i32, cy: i32) -> bool {
        let c = self.cell(cx, cy);
        if self.flags[c] & HALFLAND == 0 && self.land[c] == 0 {
            return false;
        }
        (0..4).any(|oy| (0..4).any(|ox| self.tcoord_is_surfline(cx * 4 + ox, cy * 4 + oy) != 0))
    }

    /// `TerrainOut::land_left_of_vector@00867730`: whether the tcoord on
    /// the left of the step `a → b` is land. A step that is not one of the
    /// eight moves answers 0.
    fn land_left(&self, a: (i32, i32), b: (i32, i32)) -> i32 {
        let d = (b.0 - a.0, b.1 - a.1);
        let Some(k) = MOVE[1..].iter().position(|&m| m == d) else {
            return 0;
        };
        let (mx, my) = MOVE[LEFT[k]];
        let (x, y) = (a.0 + mx, a.1 + my);
        if !self.tile_in(x, y) {
            return 0;
        }
        let c = self.cell(x >> 2, y >> 2);
        let land = if self.flags[c] & HALFLAND == 0 {
            self.cell_land(c)
        } else {
            !self.tile_ocean(x, y)
        };
        i32::from(land)
    }

    fn xy(&self, t: i32) -> (i32, i32) {
        (t % self.tw(), t / self.tw())
    }

    /// `TerrainOut::sort_waterline_wcoords@00872f30`, then
    /// `connect_waterline_regions@00867890`: the chains
    /// `sorted_waterline_tcoords` holds, empties included, and each one's
    /// sea region (`+0x4798`).
    pub fn sorted_waterline_tcoords(&self) -> (Vec<Vec<i32>>, Vec<i32>) {
        let (w, h, tw) = (self.width, self.height, self.tw());
        let mut visited = vec![false; (tw * self.th()) as usize];
        let mut done = vec![false; (w * h) as usize];
        let (mut chains, mut seas): (Vec<Vec<i32>>, Vec<i32>) = (vec![], vec![]);
        for cy in 0..h {
            for cx in 0..w {
                let c = self.cell(cx, cy);
                if done[c] {
                    continue;
                }
                if !self.wcoord_has_surfline(cx, cy) {
                    done[c] = true;
                    continue;
                }
                for oy in 0..4 {
                    for ox in 0..4 {
                        let (mut x, mut y) = (cx * 4 + ox, cy * 4 + oy);
                        let sea = self.tcoord_is_surfline(x, y);
                        if sea == 0 || visited[(x + tw * y) as usize] {
                            continue;
                        }
                        let mut chain = vec![x + tw * y];
                        visited[(x + tw * y) as usize] = true;
                        seas.push(sea);
                        // The walk: the first orthogonal step found, else
                        // the last diagonal one.
                        'walk: loop {
                            let mut diagonal = None;
                            for &(mx, my) in &MOVE[1..] {
                                let (nx, ny) = (x + mx, y + my);
                                if !self.tile_in(nx, ny)
                                    || visited[(nx + tw * ny) as usize]
                                    || self.tcoord_is_surfline(nx, ny) == 0
                                {
                                    continue;
                                }
                                if mx.abs() + my.abs() != 2 {
                                    (x, y) = (nx, ny);
                                    visited[(x + tw * y) as usize] = true;
                                    chain.push(x + tw * y);
                                    done[self.cell(x >> 2, y >> 2)] = true;
                                    continue 'walk;
                                }
                                diagonal = Some((nx, ny));
                            }
                            let Some((nx, ny)) = diagonal else { break };
                            (x, y) = (nx, ny);
                            visited[(x + tw * y) as usize] = true;
                            chain.push(x + tw * y);
                            done[self.cell(x >> 2, y >> 2)] = true;
                        }
                        chains.push(chain);
                    }
                }
            }
        }
        self.connect_waterline_regions(&mut chains, &mut seas);
        (chains, seas)
    }

    /// `TerrainOut::connect_waterline_regions@00867890`: joins two chains
    /// of one sea whose ends touch (within a tcoord either way), from the
    /// top after every join; then turns a chain whose start has the land
    /// on its left, and empties one whose end does too.
    fn connect_waterline_regions(&self, chains: &mut Vec<Vec<i32>>, seas: &mut Vec<i32>) {
        let near = |a: i32, b: i32| {
            let (p, q) = (self.xy(a), self.xy(b));
            (p.0 - q.0).abs() < 2 && (p.1 - q.1).abs() < 2
        };
        'restart: loop {
            for i in 0..chains.len() {
                for j in i + 1..chains.len() {
                    if seas[j] != seas[i] {
                        continue;
                    }
                    let (a0, al) = (chains[i][0], *chains[i].last().unwrap());
                    let (b0, bl) = (chains[j][0], *chains[j].last().unwrap());
                    if near(a0, bl) {
                        let a = chains.remove(i);
                        seas.remove(i);
                        chains[j - 1].extend(a);
                    } else if near(al, b0) {
                        let b = chains.remove(j);
                        seas.remove(j);
                        chains[i].extend(b);
                    } else if near(a0, b0) {
                        let b = chains.remove(j);
                        seas.remove(j);
                        let mut joined: Vec<i32> = b.into_iter().rev().collect();
                        joined.extend(&chains[i]);
                        chains[i] = joined;
                    } else if near(al, bl) {
                        let mut joined = chains.remove(j);
                        seas.remove(j);
                        joined.extend(chains[i].iter().rev());
                        chains[i] = joined;
                    } else {
                        continue;
                    }
                    continue 'restart;
                }
            }
            break;
        }
        for c in chains.iter_mut() {
            let p = |k: usize| self.xy(c[k]);
            let mut left = self.land_left(p(0), p(1));
            if c.len() > 16 {
                left += self.land_left(p(1), p(2)) + self.land_left(p(2), p(3));
            }
            if left > 1 {
                c.reverse();
            }
            let n = c.len();
            if n > 16 {
                let p = |k: usize| self.xy(c[k]);
                let end = self.land_left(p(n - 2), p(n - 1))
                    + self.land_left(p(n - 3), p(n - 2))
                    + self.land_left(p(n - 4), p(n - 3));
                if end > 1 {
                    c.clear();
                }
            }
        }
    }
}
