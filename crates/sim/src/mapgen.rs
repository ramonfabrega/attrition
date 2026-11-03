//! **Map generation** (`docs/MAPGEN.md`): the random map the original builds
//! before frame 0, one phase at a time, each checked at its own
//! `say_checksum` checkpoint by the Adler-32 of §3.
//!
//! The first port step (item 3776, §15 step 1) is the grid itself, the
//! checksum it is falsified by, and Great Sahara's `make_continents`
//! (§5): `World::wipe`, `Regions::clear_all`'s cell write,
//! `Map::invert_land`, and the start ring. No float: Great Sahara's mask
//! reads no `Fractal`.

use crate::combat::Rng;
use crate::movement::{Angle, cos_component, sin_component};

/// The bytes of one `WData` (`types.txt`, size 0x1c) that
/// `World::walk_data@006b5cf0`'s part 5 walks: `[cell, cell + 0x15)`,
/// `flags` through `was_seen`. The three bytes after `was_seen` are padding
/// and the `CollBlock *` at +0x18 is part 9's.
pub const WDATA_WALKED: usize = 0x15;

/// One cell, the `WData` fields part 5 walks, in their widths and
/// signedness (`types.txt`). `land` is the `land_key` index (`BASELAND` 0,
/// `SANDY` 1, `OCEAN` 2, `NONE` 3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WCell {
    pub flags: u16,
    pub land: i8,
    pub land_sub: u8,
    pub region: i16,
    pub region2: i16,
    pub down: i16,
    pub down_who: i16,
    pub val: u8,
    pub goods: u8,
    pub light: u8,
    pub who: i8,
    pub who2: i8,
    pub blocked: u8,
    pub bad: u8,
    pub solid: i8,
    pub was_seen: u8,
}

impl WCell {
    /// What `World::wipe@006b2c00` writes into every cell: water (`land`
    /// 2), sea region 0x40, `down` −1, no owner (`who`/`who2` −1),
    /// everything else 0.
    pub const WIPED: WCell = WCell {
        flags: 0,
        land: 2,
        land_sub: 0,
        region: 0x40,
        region2: 0,
        down: -1,
        down_who: 0,
        val: 0,
        goods: 0,
        light: 0,
        who: -1,
        who2: -1,
        blocked: 0,
        bad: 0,
        solid: 0,
        was_seen: 0,
    };

    /// The cell's walked bytes, little-endian at their `WData` offsets.
    pub fn walked(&self) -> [u8; WDATA_WALKED] {
        let mut b = [0u8; WDATA_WALKED];
        b[0..2].copy_from_slice(&self.flags.to_le_bytes());
        b[2] = self.land as u8;
        b[3] = self.land_sub;
        b[4..6].copy_from_slice(&self.region.to_le_bytes());
        b[6..8].copy_from_slice(&self.region2.to_le_bytes());
        b[8..10].copy_from_slice(&self.down.to_le_bytes());
        b[10..12].copy_from_slice(&self.down_who.to_le_bytes());
        b[0xc] = self.val;
        b[0xd] = self.goods;
        b[0xe] = self.light;
        b[0xf] = self.who as u8;
        b[0x10] = self.who2 as u8;
        b[0x11] = self.blocked;
        b[0x12] = self.bad;
        b[0x13] = self.solid as u8;
        b[0x14] = self.was_seen;
        b
    }
}

/// `CheckSum::walk_function@00936ff0`'s running sum: Adler-32, `accum`
/// starting at 1 (`docs/MAPGEN.md` §3). Every walked range of one part
/// feeds the same sum, in walk order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Adler32 {
    a: u32,
    b: u32,
}

impl Default for Adler32 {
    fn default() -> Self {
        Adler32 { a: 1, b: 0 }
    }
}

impl Adler32 {
    const MOD: u32 = 65521;

    pub fn feed(&mut self, bytes: &[u8]) {
        for &x in bytes {
            self.a = (self.a + u32::from(x)) % Self::MOD;
            self.b = (self.b + self.a) % Self::MOD;
        }
    }

    pub const fn sum(&self) -> u32 {
        (self.b << 16) | self.a
    }
}

/// The Adler-32 of one byte run.
pub fn adler32(bytes: &[u8]) -> u32 {
    let mut s = Adler32::default();
    s.feed(bytes);
    s.sum()
}

/// One `SimpleArray<WCoord>` as `SimpleArray<WCoord>::walk_data@0047c660`
/// walks it: an empty array is its 4-byte `num` alone; a full one is
/// `num`, the capacity, the 2-byte growth increment at +0xc, the flag byte
/// at +0x14 with bit 0x40 cleared, and then the `num` entries.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoordArray {
    pub list: Vec<i32>,
    /// The allocated slots: `size` at +0x8.
    pub size: i32,
    /// The growth increment, the short at +0xc that
    /// `World::add_starting_location@006b2de0` passes to the grow slot.
    pub increment: i16,
    /// The flag byte at +0x14.
    pub flags: u8,
}

impl CoordArray {
    /// An empty array whose grow step is `increment`.
    pub const fn new(increment: i16) -> CoordArray {
        CoordArray {
            list: Vec::new(),
            size: 0,
            increment,
            flags: 0,
        }
    }

    /// `add`: grows when full, then appends. The grow is
    /// `SimpleArray<WCoord>::increase_size@00434630(increment)`: a negative
    /// increment doubles the allocation, or makes it 4 when there is none.
    pub fn push(&mut self, v: i32) {
        if self.size <= self.list.len() as i32 {
            self.size += match self.increment {
                n if n < 0 => {
                    if self.size == 0 {
                        4
                    } else {
                        self.size
                    }
                }
                n => i32::from(n),
            };
        }
        self.list.push(v);
    }

    /// Feeds the walk into `s`.
    pub fn walk(&self, s: &mut Adler32) {
        let num = self.list.len() as i32;
        s.feed(&num.to_le_bytes());
        if num == 0 {
            return;
        }
        s.feed(&self.size.to_le_bytes());
        s.feed(&self.increment.to_le_bytes());
        s.feed(&[self.flags & !0x40]);
        for v in &self.list {
            s.feed(&v.to_le_bytes());
        }
    }
}

/// The start arrays' growth increment, −1: each doubles from 4. **Measured,
/// not read**: of the increments −1..=64 and every flag byte, −1 with flag
/// byte 0 is the only one whose part 2 at `map.cpp` 7807 is the trace's on
/// both Great Sahara lobbies (run1000, 2 nations; run1412, 3) — the
/// constructor that sets it was not read (`docs/MAPGEN.md` §5).
pub const START_INCREMENT: i16 = -1;

/// The world grid map generation writes: `World.wdata` (`xs × ys`,
/// row-major) and the four start arrays part 2 walks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapGrid {
    pub xs: i32,
    pub ys: i32,
    pub cells: Vec<WCell>,
    /// `start_x` (+0x80), `start_y` (+0x9c), `start_city_x` (+0xb8),
    /// `start_city_y` (+0xd4).
    pub starts: [CoordArray; 4],
}

impl MapGrid {
    /// A grid of `xs × ys` default cells and empty start arrays growing by
    /// [`START_INCREMENT`].
    pub fn new(xs: i32, ys: i32) -> MapGrid {
        MapGrid {
            xs,
            ys,
            cells: vec![WCell::default(); (xs * ys) as usize],
            starts: core::array::from_fn(|_| CoordArray::new(START_INCREMENT)),
        }
    }

    pub fn cell(&self, x: i32, y: i32) -> &WCell {
        &self.cells[(y * self.xs + x) as usize]
    }

    /// Part 5: every cell's walked bytes, in `wdata` order.
    pub fn cell_part(&self) -> u32 {
        let mut s = Adler32::default();
        for c in &self.cells {
            s.feed(&c.walked());
        }
        s.sum()
    }

    /// Part 2: the four start arrays, in walk order.
    pub fn start_part(&self) -> u32 {
        let mut s = Adler32::default();
        for a in &self.starts {
            a.walk(&mut s);
        }
        s.sum()
    }

    /// `World::wipe@006b2c00`'s writes this grid holds: the start arrays
    /// emptied (their `num`s zeroed, the allocation kept) and every cell
    /// [`WCell::WIPED`].
    pub fn wipe(&mut self) {
        for a in &mut self.starts {
            a.list.clear();
        }
        self.cells.fill(WCell::WIPED);
    }

    /// `Regions::clear_all@00680060`'s one write into the cells: every
    /// `region` to 0. The region table it also resets is not held here.
    pub fn clear_regions(&mut self) {
        for c in &mut self.cells {
            c.region = 0;
        }
    }

    /// `Map::invert_land@0069d880`: a cell with flag 0x100 clear whose
    /// `land` is 1 or 2 becomes land 0, any other becomes water 2; both
    /// writes are the 16-bit store at +2, so `land_sub` is zeroed with it.
    /// Every `region` is zeroed, then `Regions::clear_all` runs again.
    pub fn invert_land(&mut self) {
        for c in &mut self.cells {
            let wet = c.flags & 0x100 == 0 && (c.land == 1 || c.land == 2);
            c.land = if wet { 0 } else { 2 };
            c.land_sub = 0;
            c.region = 0;
        }
        self.clear_regions();
    }

    /// `World::add_starting_location@006b2de0`: the start cell once on
    /// each of `start_x`/`start_y`, and the four cells of the city's 2×2
    /// footprint — `(x, x−1, x, x−1)` and `(y, y, y−1, y−1)`. Its other
    /// write, the start-city bit grid at +0xf8, no part walks.
    pub fn add_starting_location(&mut self, x: i32, y: i32) {
        self.starts[0].push(x);
        self.starts[1].push(y);
        for dx in [0, -1, 0, -1] {
            self.starts[2].push(x + dx);
        }
        for dy in [0, 0, -1, -1] {
            self.starts[3].push(y + dy);
        }
    }
}

/// `project@0092d000` on cells: the point `d` along `angle` from `(x, y)`,
/// `x + sin·d`, `y − cos·d` — the same fold as [`crate::army::step_along`].
fn project(x: i32, y: i32, angle: Angle, d: i32) -> (i32, i32) {
    (x + sin_component(angle, d), y - cos_component(angle, d))
}

/// **`MapGreatSahara::make_continents@0069bf70`** (`docs/MAPGEN.md` §5):
/// the whole map land, and the players on a ring about its centre. Four
/// draws: the angle's high and low halves, then the centre's x and y
/// jitters. Returns `Map::start_dist` (+0x28), `max(4, xs / 16)`, which the
/// function writes beside them.
pub fn great_sahara_make_continents(grid: &mut MapGrid, players: i32, rng: &mut Rng) -> i32 {
    grid.wipe();
    grid.clear_regions();
    grid.invert_land();
    let half = grid.xs.min(grid.ys) / 2;
    let radius = (half - 4).min(half * 80 / 100);
    let step = (u32::MAX / players.max(1) as u32) as i32;
    let hi = rng.roll() % 0xffff;
    let lo = rng.roll() % 0xffff;
    let mut angle = (hi << 16).wrapping_add(lo);
    let cx = grid.xs / 2 + (rng.roll() & 1);
    let cy = grid.ys / 2 + (rng.roll() & 1);
    let start_dist = (grid.xs / 16).max(4);
    for _ in 0..players {
        angle = angle.wrapping_add(step);
        let (x, y) = project(cx, cy, Angle(angle), radius);
        grid.add_starting_location(x, y);
    }
    start_dist
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tell §3 names: an empty walk prints 1, and N zero bytes print
    /// `N·65536 + 1`.
    #[test]
    fn the_sum_is_adler_32() {
        assert_eq!(adler32(&[]), 1);
        assert_eq!(adler32(&[0; 16]), 16 * 65536 + 1);
        assert_eq!(adler32(b"Wikipedia"), 0x11e6_0398);
    }

    /// Four empty start arrays are part 2 at `map.cpp` 7805 on every
    /// trace: 1,048,577, sixteen zero bytes.
    #[test]
    fn four_empty_start_arrays_are_sixteen_zero_bytes() {
        assert_eq!(MapGrid::new(60, 60).start_part(), 1_048_577);
    }

    /// Great Sahara's grid at 7807 is uniform whatever the draws: every
    /// cell land 0, region 0, `down`/`who`/`who2` −1. run1000's part 5
    /// there is 1,267,075,849.
    #[test]
    fn great_sahara_s_mask_is_one_cell_repeated() {
        let mut g = MapGrid::new(60, 60);
        let mut rng = Rng::new(0x1234);
        great_sahara_make_continents(&mut g, 2, &mut rng);
        assert!(g.cells.iter().all(|c| *c
            == WCell {
                land: 0,
                region: 0,
                ..WCell::WIPED
            }));
        assert_eq!(g.cell_part(), 1_267_075_849);
    }

    /// Four draws, whatever the player count.
    #[test]
    fn great_sahara_draws_four() {
        for players in [2, 3, 8] {
            let mut g = MapGrid::new(60, 60);
            let mut ours = Rng::new(0xa289_127d);
            great_sahara_make_continents(&mut g, players, &mut ours);
            let mut four = Rng::new(0xa289_127d);
            for _ in 0..4 {
                four.roll();
            }
            assert_eq!(ours.seed, four.seed);
            assert_eq!(g.starts[0].list.len(), players as usize);
            assert_eq!(g.starts[2].list.len(), 4 * players as usize);
        }
    }
}
