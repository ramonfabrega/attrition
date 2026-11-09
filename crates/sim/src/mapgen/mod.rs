//! **Map generation** (`docs/MAPGEN.md`): the random map the original builds
//! before frame 0, one phase at a time, each checked at its own
//! `say_checksum` checkpoint by the Adler-32 of §3.
//!
//! The first port step (item 3776, §15 step 1) is the grid itself, the
//! checksum it is falsified by, and Great Sahara's `make_continents`
//! (§5): `World::wipe`, `Regions::clear_all`'s cell write,
//! `Map::invert_land`, and the start ring. No float: Great Sahara's mask
//! reads no `Fractal`.

use crate::ai::ai_place::{MOVE_X, MOVE_Y};
use crate::combat::Rng;
use crate::movement::{Angle, cos_component, sin_component};
use crate::num::double::Double;
use crate::world::{CORNER_X, CORNER_Y};

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

/// The Adler-32 of a `len`-byte run whose sum is `sum`, after the bytes at
/// `at` change from `old` to `new`: a byte `i` from the start weighs 1 in
/// the low half and `len − i` in the high half. This checks one field's
/// write against a part whose other fields the port does not carry
/// (`docs/MAPGEN.md` §10: part 4 at `setup.cpp` 734).
pub fn adler_rewrite(sum: u32, len: usize, at: usize, old: &[u8], new: &[u8]) -> u32 {
    const MOD: i64 = 65521;
    let (mut a, mut b) = (i64::from(sum & 0xffff), i64::from(sum >> 16));
    for (k, (&o, &n)) in old.iter().zip(new).enumerate() {
        let d = i64::from(n) - i64::from(o);
        a += d;
        b += d * (len - at - k) as i64;
    }
    ((b.rem_euclid(MOD) as u32) << 16) | a.rem_euclid(MOD) as u32
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
    /// `World.tdata` (+0x138): one `TData::mask` per tile, `4·xs × 4·ys`
    /// row-major — a cell `(x, y)` covers tiles `4x..4x+3 × 4y..4y+3`. Part 6
    /// walks it.
    pub tiles: Vec<u16>,
    /// `World.start_city_locs` (+0xf0): the start-city bit of each cell,
    /// `y·xs + x` — the 2×2 footprint [`MapGrid::add_starting_location`]
    /// marks. No part walks it; `place_all`'s predicates read it.
    pub city: Vec<bool>,
    /// `goods` (`PtrArray<Good>`), up to `Objects+0x154`: each slot's live
    /// good as `(type, cell x, cell y)`, `None` once killed.
    /// `Objects::init_good@00653f30` takes the first dead slot, else appends,
    /// so a good's index — the `down_who` its cell holds — counts the oil
    /// `World::set_oil_at` made and unmade in `place_all`. No part walks it.
    pub goods: Vec<Option<(i32, i32, i32)>>,
    /// `MountainsData`'s placed ranges (+0x7c, +0x98, +0xd0): each
    /// `Mountains::add_mountain@0089c2e0` that wrote appends its origin
    /// cell and template index, and nothing in map generation removes one.
    /// No part walks it; `World::set_edge_flags`' 0x8000 pass reads it
    /// (`docs/MAPGEN.md` §9).
    pub mountains: Vec<(i32, i32, usize)>,
    /// `RiversOut::all_rivers` (+0x40): one entry per
    /// `RiversOut::start_river@00888f70`, each its spline's control points
    /// as the cells `RiverOut::add_spline_point@00885a40` was handed —
    /// one per path cell [`crate::mapgen::map_rivers::make_riverlist`] marks, and
    /// on a border end the end cell again and one off the map. The
    /// troughs read them ([`crate::mapgen::map_troughs`]); no part walks them.
    pub rivers: Vec<Vec<(i32, i32)>>,
    /// `Cliffs`' queued cliffs (`cliffs` +0x40): each
    /// `Cliffs::add_cliff@008a6690` appends its origin cell, size index and
    /// template index, for `Cliffs::init@008a8020` to place in
    /// `Terrain::init` (`docs/MAPGEN.md` §18.3). No part walks it.
    pub cliffs: Vec<(i32, i32, usize, usize)>,
}

impl MapGrid {
    /// **`Objects::init_good@00653f30`**'s slot: the first dead one, else a
    /// new one. Returns the good's index.
    pub fn init_good(&mut self, ty: i32, x: i32, y: i32) -> usize {
        let i = self
            .goods
            .iter()
            .position(Option::is_none)
            .unwrap_or(self.goods.len());
        if i == self.goods.len() {
            self.goods.push(None);
        }
        self.goods[i] = Some((ty, x, y));
        i
    }

    /// A grid of `xs × ys` default cells and empty start arrays growing by
    /// [`START_INCREMENT`].
    pub fn new(xs: i32, ys: i32) -> MapGrid {
        MapGrid {
            xs,
            ys,
            cells: vec![WCell::default(); (xs * ys) as usize],
            starts: core::array::from_fn(|_| CoordArray::new(START_INCREMENT)),
            tiles: vec![0; (16 * xs * ys) as usize],
            city: vec![false; (xs * ys) as usize],
            goods: Vec::new(),
            mountains: Vec::new(),
            rivers: Vec::new(),
            cliffs: Vec::new(),
        }
    }

    /// `tile_xs` (+0x18) and `tile_ys` (+0x1c): four tiles a cell each way.
    pub fn tile_dims(&self) -> (i32, i32) {
        (4 * self.xs, 4 * self.ys)
    }

    /// Part 6: the tile masks (two bytes a tile), then `seen`, `seen2` and
    /// `seen3`, `fog_size` bytes each. Map generation writes none of the
    /// three, so they walk as zeros; `fog_size` is `4·xs·ys` (`fog_xs` is
    /// `2·xs`), which is what makes 7854's part 6 on Great Sahara the
    /// 158,400 zero bytes it prints (`docs/MAPGEN.md` §3).
    pub fn tile_part(&self) -> u32 {
        let mut s = Adler32::default();
        for m in &self.tiles {
            s.feed(&m.to_le_bytes());
        }
        let zeros = [0u8; 4096];
        let mut left = 3 * 4 * (self.xs * self.ys) as usize;
        while left > 0 {
            let n = left.min(zeros.len());
            s.feed(&zeros[..n]);
            left -= n;
        }
        s.sum()
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
        self.tiles.fill(0);
        self.city.fill(false);
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
    /// write, the start-city bits of the same four cells ([`MapGrid::city`]),
    /// no part walks.
    pub fn add_starting_location(&mut self, x: i32, y: i32) {
        for (cx, cy) in [(x, y), (x - 1, y), (x, y - 1), (x - 1, y - 1)] {
            let i = cy * self.xs + cx;
            if let Some(b) = usize::try_from(i).ok().and_then(|i| self.city.get_mut(i)) {
                *b = true;
            }
        }
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
/// `x + sin·d`, `y − cos·d` — the same fold as [`crate::ai::army::step_along`].
pub(crate) fn project(x: i32, y: i32, angle: Angle, d: i32) -> (i32, i32) {
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

/// **`MapSouthwestMesa::make_continents@0069bb20`** (`docs/MAPGEN.md` §5,
/// §18.3; item 3869): instruction for instruction Great Sahara's
/// ([`great_sahara_make_continents`]) — the listing from 0x69bb20 to its
/// `ret $4` and 0x69bf70's to its own agree but for the relative call
/// targets — so the same whole land, the same ring and the same four draws.
/// What makes Mesa Mesa is its XML, read from `place_all` on (the cliffs).
pub fn southwest_mesa_make_continents(grid: &mut MapGrid, players: i32, rng: &mut Rng) -> i32 {
    great_sahara_make_continents(grid, players, rng)
}

/// `WData::flags` bit 4, COAST, which `Map::make_coastlines@006947a0` sets
/// with 0x400 and its thinning clears.
pub use crate::ids::cell::COAST;
/// `WData::flags` bit 0x100, HALFLAND (`crate::world`'s `HALFLAND`): a cell
/// that carries it is never ocean.
pub use crate::ids::cell::HALFLAND;

impl WCell {
    /// `WorldData::is_ocean@006b4830`: HALFLAND clear and `land` `SANDY`
    /// (1) or `OCEAN` (2).
    pub const fn is_ocean(&self) -> bool {
        self.flags & HALFLAND == 0 && (self.land == 1 || self.land == 2)
    }

    /// The 16-bit store at +2 the passes make: `land` and `land_sub` together.
    fn set_land(&mut self, land: i8) {
        self.land = land;
        self.land_sub = 0;
    }
}

/// The land and sea labels' first bins and their overflow bins
/// (`Regions::find_all@0067eff0`): land takes the first free record from 1
/// and clamps to 0x3e, sea counts up from 0x40 and clamps to 0x7d; a region
/// that lands on a clamp evicts the smallest into the bin after it (0x3f,
/// 0x7e).
pub const LAND_BIN: i16 = 0x3e;
pub const SEA_BIN: i16 = 0x7d;

/// What `Regions` (`GameAccess::regions`, 128 records of 0x88) holds that
/// the labelling reads back: each record's cell count (+0x14) and the sea
/// counter. The records' coordinate lists, coastal sets and ranks are not
/// held — no part walks them, and nothing here reads them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Regions {
    pub count: [i32; 128],
    pub sea: i16,
}

impl Default for Regions {
    fn default() -> Self {
        Regions {
            count: [0; 128],
            sea: 0x40,
        }
    }
}

impl Regions {
    /// **`Regions::clear_all@00680060`**: every record's count to 0, every
    /// cell's `region` to 0 — `region2` is left — and the sea counter to
    /// 0x40.
    pub fn clear_all(&mut self, grid: &mut MapGrid) {
        *self = Regions::default();
        grid.clear_regions();
    }

    /// **`Regions::find_all@0067eff0`** (`docs/MAPGEN.md` §6), its cell
    /// writes: y outer, x inner, each unlabelled cell either takes the label
    /// of its first neighbour of the same wetness that has one (in
    /// `move_x/move_y[1..=8]` order, never 0x7f), or opens a region and
    /// floods it ([`Regions::find`]). Then the second pass, x outer and y
    /// inner: a HALFLAND cell whose `region2` is still 0 takes its first
    /// ocean neighbour's `region`. A `region2` already set is kept, so a
    /// stale one survives a re-run. `set_coastals`, `sort_regions` and
    /// `rebuild_coords` follow and only read cells.
    pub fn find_all(&mut self, grid: &mut MapGrid) {
        for y in 0..grid.ys {
            for x in 0..grid.xs {
                let here = *grid.cell(x, y);
                if here.region != 0 {
                    continue;
                }
                let wet = here.is_ocean();
                let adopted = (1..=8).find_map(|d| {
                    let (nx, ny) = (x + MOVE_X[d], y + MOVE_Y[d]);
                    let n = grid.get(nx, ny)?;
                    (n.is_ocean() == wet && n.region != 0 && n.region != 0x7f).then_some(n.region)
                });
                if let Some(r) = adopted {
                    grid.cell_mut(x, y).region = r;
                    self.count[r as usize] += 1;
                    continue;
                }
                let (lo, bin, id) = if wet {
                    self.sea += 1;
                    (0x40, SEA_BIN, self.sea.clamp(0x40, SEA_BIN))
                } else {
                    let free = (1..0x40).find(|&k| self.count[k] == 0).unwrap_or(0x40);
                    (0, LAND_BIN, (free as i16).clamp(0, LAND_BIN))
                };
                self.find(grid, x, y, id);
                if id == bin {
                    self.evict(grid, lo, bin);
                }
            }
        }
        for x in 0..grid.xs {
            for y in 0..grid.ys {
                let here = *grid.cell(x, y);
                if here.flags & HALFLAND == 0 || here.region2 != 0 {
                    continue;
                }
                let sea = (1..=8).find_map(|d| {
                    let n = grid.get(x + MOVE_X[d], y + MOVE_Y[d])?;
                    n.is_ocean().then_some(n.region)
                });
                if let Some(r) = sea {
                    grid.cell_mut(x, y).region2 = r;
                }
            }
        }
    }

    /// A region that landed on its clamp `bin`: the smallest record in
    /// `[lo, bin)` by count, if any is smaller than it, moves into `bin + 1`
    /// and gives its number to the new region's cells; with none smaller,
    /// the new region's cells go to `bin + 1`. On land `lo` is 0, whose
    /// record never counts, so it is the one taken — and its relabel moves
    /// every still-unlabelled cell into 0x3f, as the original's does.
    fn evict(&mut self, grid: &mut MapGrid, lo: i16, bin: i16) {
        let over = bin + 1;
        let mut best = None;
        let mut least = self.count[bin as usize];
        for k in lo..bin {
            if self.count[k as usize] < least {
                least = self.count[k as usize];
                best = Some(k);
            }
        }
        let relabel = |grid: &mut MapGrid, from: i16, to: i16| {
            for c in &mut grid.cells {
                if c.region == from {
                    c.region = to;
                }
            }
        };
        match best {
            Some(k) => {
                self.count[over as usize] += self.count[k as usize];
                self.count[k as usize] = 0;
                relabel(grid, k, over);
                self.count[k as usize] += self.count[bin as usize];
                self.count[bin as usize] = 0;
                relabel(grid, bin, k);
            }
            None => {
                self.count[over as usize] += self.count[bin as usize];
                self.count[bin as usize] = 0;
                relabel(grid, bin, over);
            }
        }
    }

    /// **`Regions::find@00680180`**: a breadth-first flood from `(x, y)`
    /// over the eight neighbours, taking every unlabelled cell of the seed's
    /// wetness; the record's count is the cells dequeued.
    pub fn find(&mut self, grid: &mut MapGrid, x: i32, y: i32, id: i16) {
        let wet = grid.cell(x, y).is_ocean();
        grid.cell_mut(x, y).region = id;
        self.count[id as usize] = 0;
        let mut queue = std::collections::VecDeque::from([(x, y)]);
        while let Some((cx, cy)) = queue.pop_front() {
            self.count[id as usize] += 1;
            for d in 1..=8 {
                let (nx, ny) = (cx + MOVE_X[d], cy + MOVE_Y[d]);
                let Some(n) = grid.get(nx, ny) else { continue };
                if n.is_ocean() == wet && n.region == 0 {
                    grid.cell_mut(nx, ny).region = id;
                    queue.push_back((nx, ny));
                }
            }
        }
    }
}

impl MapGrid {
    /// The cell at `(x, y)`, or `None` off the grid.
    pub fn get(&self, x: i32, y: i32) -> Option<&WCell> {
        (0..self.xs).contains(&x).then_some(())?;
        (0..self.ys).contains(&y).then_some(())?;
        Some(self.cell(x, y))
    }

    pub fn cell_mut(&mut self, x: i32, y: i32) -> &mut WCell {
        &mut self.cells[(y * self.xs + x) as usize]
    }

    fn land_at(&self, x: i32, y: i32) -> Option<i8> {
        self.get(x, y).map(|c| c.land)
    }

    /// **`Map::fix_diag_land@0069c250`** (`docs/MAPGEN.md` §6): x outer, y
    /// inner, a `BASELAND` cell that meets another across a corner
    /// (`CORNER_X/CORNER_Y[1..=4]`) while both cells flanking that corner
    /// are on the grid and not `BASELAND` becomes water, `land` 2.
    pub fn fix_diag_land(&mut self) {
        for x in 0..self.xs {
            for y in 0..self.ys {
                if self.cell(x, y).land != 0 {
                    continue;
                }
                for k in 1..=4 {
                    let (cx, cy) = (x + CORNER_X[k], y + CORNER_Y[k]);
                    if self.land_at(cx, cy) != Some(0) {
                        continue;
                    }
                    // The two cells either side of the corner: the corner's
                    // x with this y, and this x with the corner's y.
                    let (a, b) = (self.land_at(cx, y), self.land_at(x, cy));
                    if matches!((a, b), (Some(a), Some(b)) if a != 0 && b != 0) {
                        self.cell_mut(x, y).set_land(2);
                        break;
                    }
                }
            }
        }
    }

    /// **`Map::fix_lakes@0069c470`**: over the interior, y outer and x inner,
    /// an ocean cell with more than five of its eight neighbours not ocean
    /// becomes `BASELAND`; repeated until a sweep changes nothing.
    pub fn fix_lakes(&mut self) {
        if self.ys - 1 < 2 {
            return;
        }
        loop {
            let mut changed = false;
            for y in 1..self.ys - 1 {
                for x in 1..self.xs - 1 {
                    if !self.cell(x, y).is_ocean() {
                        continue;
                    }
                    let dry = (1..=8)
                        .filter(|&d| !self.cell(x + MOVE_X[d], y + MOVE_Y[d]).is_ocean())
                        .count();
                    if dry > 5 {
                        self.cell_mut(x, y).set_land(0);
                        changed = true;
                    }
                }
            }
            if !changed {
                return;
            }
        }
    }

    /// **`Map::make_coastlines@006947a0`** (`docs/MAPGEN.md` §6):
    /// [`MapGrid::fix_lakes`]; then, y outer and x inner, a cell that is not
    /// ocean and not yet COAST, with an ocean neighbour that is not COAST,
    /// takes COAST|0x400 and loses 0x38; then the thinning, in the same
    /// order: a COAST cell none of whose neighbours is land without COAST
    /// becomes water, `land` 2, losing 0x3c|0x100|0x400.
    pub fn make_coastlines(&mut self) {
        self.fix_lakes();
        for y in 0..self.ys {
            for x in 0..self.xs {
                let here = *self.cell(x, y);
                if here.is_ocean() || here.flags & COAST != 0 {
                    continue;
                }
                let shore = (1..=8).any(|d| {
                    self.get(x + MOVE_X[d], y + MOVE_Y[d])
                        .is_some_and(|n| n.is_ocean() && n.flags & COAST == 0)
                });
                if shore {
                    let c = self.cell_mut(x, y);
                    c.flags = (c.flags & 0xffc3) | COAST | 0x400;
                }
            }
        }
        for y in 0..self.ys {
            for x in 0..self.xs {
                if self.cell(x, y).flags & COAST == 0 {
                    continue;
                }
                let inland = (1..=8).any(|d| {
                    self.get(x + MOVE_X[d], y + MOVE_Y[d])
                        .is_some_and(|n| !n.is_ocean() && n.flags & COAST == 0)
                });
                if !inland {
                    let c = self.cell_mut(x, y);
                    c.flags &= 0xfac3;
                    c.set_land(2);
                }
            }
        }
    }

    /// **`TerrainGroups::fill_fertile@006a6f90`** (`docs/MAPGEN.md` §6): x
    /// outer, y inner, every `BASELAND` cell's `land_sub` is the number of
    /// leading `partitions` at or below its byte of `fractal`.
    pub fn fill_fertile(&mut self, fractal: &Fractal, partitions: &[u8]) {
        for x in 0..self.xs {
            for y in 0..self.ys {
                if self.cell(x, y).land != 0 {
                    continue;
                }
                let b = fractal.at(x, y);
                let k = partitions.iter().take_while(|&&p| b >= p).count();
                self.cell_mut(x, y).land_sub = k as u8;
            }
        }
    }
}

/// **`Fractal`'s byte grid** as `Fractal::init@006aa2d0` builds it: `xs + 1`
/// columns of `ys + 1` bytes, `frac[x][y]`, and the `flags` it was built
/// with. Its doubles (`x_inc`, `y_inc`) are `Fractal::get_height`'s and are
/// derived there from `xs`, `ys` and `flags & 1`; its `parts` are all −1,
/// which `init` writes and nothing in map generation changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fractal {
    pub xs: i32,
    pub ys: i32,
    pub flags: i32,
    pub frac: Vec<Vec<u8>>,
}

impl Fractal {
    /// **`Fractal::init`**: a midpoint displacement on its own `Random`
    /// seeded with `seed`, from level `smooth` (clamped to 0..=5) down to 0.
    /// The top level draws each `2^smooth` lattice point as `get(0, 0xff)`.
    /// Each finer level `i`, x outer and y inner, fills the points with an
    /// odd coordinate at that spacing: the rounded mean of the two
    /// (an edge midpoint) or four (a centre, the diagonals) parents, plus
    /// `get(0, 2^(8−smooth+i) − 1) − 2^(7−smooth+i)`, clamped to a byte.
    /// Unless `flags & 1`, the column `x = xs` copies column 0 before each
    /// level and after the last.
    pub fn init(xs: i32, ys: i32, smooth: i32, flags: i32, seed: u32) -> Fractal {
        let mut frac = vec![vec![0u8; (ys + 1) as usize]; (xs + 1) as usize];
        let mut rng = Rng::new(seed);
        let open = flags & 1;
        let p = smooth.clamp(0, 5);
        let wrap = |frac: &mut Vec<Vec<u8>>| {
            if open == 0 {
                frac[xs as usize] = frac[0].clone();
            }
        };
        for i in (0..=p).rev() {
            let mask = (1 << (i + 1)) - 1;
            wrap(&mut frac);
            for a in 0..(xs >> i) + open {
                let gx = a << i;
                for b in 0..ys >> i {
                    let gy = b << i;
                    let at = |frac: &Vec<Vec<u8>>, x: i32, y: i32| {
                        i32::from(frac[(x << i) as usize][(y << i) as usize])
                    };
                    let v = if i == p {
                        rng.get(0, 0xff)
                    } else {
                        let mean = match (gx & mask == 0, gy & mask == 0) {
                            (true, true) => continue,
                            (true, false) => (at(&frac, a, b + 1) + at(&frac, a, b - 1) + 1) >> 1,
                            (false, true) => (at(&frac, a + 1, b) + 1 + at(&frac, a - 1, b)) >> 1,
                            (false, false) => {
                                (at(&frac, a - 1, b - 1)
                                    + at(&frac, a + 1, b + 1)
                                    + at(&frac, a - 1, b + 1)
                                    + at(&frac, a + 1, b - 1)
                                    + 2)
                                    >> 2
                            }
                        };
                        let r = rng.get(0, (1 << (8 - p + i)) - 1);
                        (mean + r - (1 << (7 - p + i))).clamp(0, 0xff)
                    };
                    frac[gx as usize][gy as usize] = v as u8;
                }
            }
        }
        wrap(&mut frac);
        Fractal {
            xs,
            ys,
            flags,
            frac,
        }
    }

    pub fn at(&self, x: i32, y: i32) -> u8 {
        self.frac[x as usize][y as usize]
    }

    /// **`Fractal::get_height@006aa870`** (`docs/MAPGEN.md` §10, §11): a
    /// bilinear sample of the bytes in SSE2 doubles, in the listing's order
    /// (0x6aa877..0x6aa960). `x_inc` is `xs / (xs − (flags & 1) + 1)` and
    /// `y_inc` is `ys / ys`, both as `init` writes them (0x6aa4f2); the
    /// point is `(x_inc·(x + 0.5), y_inc·(y + 0.5))`, its cell the
    /// truncation, `fx`/`fy` the remainders and `ox`/`oy` their
    /// complements; the sum is `ox·oy·f00 + 0.0 + oy·fx·f10 + ox·fy·f01 +
    /// f11·(fy·fx)`, truncated and clamped to a byte. With `parts[0] == −1`
    /// a fractal built with `flags & 2` answers `h·100 >> 8`.
    ///
    /// Every operation is correctly rounded, so [`Double`] reproduces it.
    pub fn get_height(&self, x: i32, y: i32) -> u8 {
        let x_inc = Double::ratio(self.xs, self.xs - (self.flags & 1) + 1);
        let y_inc = Double::ratio(self.ys, self.ys);
        let dx = x_inc.mulsd(Double::from_scaled(2 * i64::from(x) + 1, -1));
        let dy = y_inc.mulsd(Double::from_scaled(2 * i64::from(y) + 1, -1));
        let (i, j) = (dx.trunc(), dy.trunc());
        let fx = dx.subsd(Double::from_i32(i));
        let fy = dy.subsd(Double::from_i32(j));
        let (ox, oy) = (Double::ONE.subsd(fx), Double::ONE.subsd(fy));
        let b = |u: i32, v: i32| Double::from_i32(i32::from(self.at(u, v)));
        let mut t = ox.mulsd(oy).mulsd(b(i, j));
        t = t.addsd(Double::ZERO);
        t = t.addsd(oy.mulsd(fx).mulsd(b(i + 1, j)));
        t = t.addsd(ox.mulsd(fy).mulsd(b(i, j + 1)));
        t = t.addsd(b(i + 1, j + 1).mulsd(fy.mulsd(fx)));
        let h = t.trunc().clamp(0, 0xff);
        if self.flags & 2 != 0 {
            ((h * 100) >> 8) as u8
        } else {
            h as u8
        }
    }
}

/// **The fertility partitions** `TerrainGroups::init_tileset_data@006a61f0`
/// builds from a tileset's `BASELAND` frequencies (percent, one per base
/// tile): the running sum of `(int)(f / 100.0f · 255.0f) & 0xff` over all
/// but the last, each stored as a byte. The single-precision product is
/// `f·255/100` truncated for every frequency of magnitude under 6,420
/// (`the_partition_float_is_the_integer_quotient`), and the files write
/// percentages, so it is computed so.
pub fn fertile_partitions(freqs: &[i32]) -> Vec<u8> {
    let mut sum = 0i64;
    freqs[..freqs.len().saturating_sub(1)]
        .iter()
        .map(|&f| {
            sum += (i64::from(f) * 255 / 100) & 0xff;
            sum as u8
        })
        .collect()
}

#[cfg(test)]
mod tests;

// The files of `mapgen/` (item 4011, `docs/STRUCTURE.md` §3).
pub mod cliffs;
pub mod halfland;
pub mod map_colonial;
pub mod map_east_indies;
pub mod map_east_west;
pub mod map_heights;
pub mod map_lakes;
pub mod map_mediterranean;
pub mod map_new_world;
pub mod map_nile_delta;
pub mod map_outback;
pub mod map_resources;
pub mod map_rivers;
pub mod map_sea_power;
pub mod map_troughs;
pub mod map_values;
pub mod map_watering_hole;
pub mod terrain_groups;
pub mod terrain_init;
