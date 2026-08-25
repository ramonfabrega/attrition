//! The world grid, its ownership, and the three lengths the original measures
//! in.
//!
//! # Three units, and why it matters
//!
//! Rise of Nations stores object positions in a fine integer unit, computes
//! territory in a coarser one, and stores ownership in a coarser one again.
//! Confusing them would put every border at four times or a quarter of its
//! real size, so they are named here and converted only through [`Pos`].
//!
//! | Unit | Size | Used for |
//! | --- | --- | --- |
//! | position unit | 1/768 cell | stored object coordinates |
//! | tile | 1/4 cell = 192 position units | territory distances |
//! | cell | 768 position units | ownership, one `WData` record each |
//!
//! The scale is not inferred from the constants' `"24 tiles"` annotations. It
//! comes from the original's own conversion: object coordinates are converted
//! by `div_3_table[pos >> 8]` when a cell is wanted and `div_3_table[pos >> 6]`
//! when a tile is, and `init_coord_lookup_array` fills that table with
//! `i / 3` — the table exists to make a divide-by-three cheap in 2002, nothing
//! more. So a cell is `256 * 3` position units and a tile is `64 * 3`. That
//! the designers' own annotations then read "24 tiles" and "44 tiles" is the
//! independent confirmation, arriving from the data rather than the code.
//!
//! The table is filled for negative indices too, with `(i - 2) / 3`, which is
//! floor division rather than C's truncation. [`Pos::cell`] therefore floors.

/// A player index.
///
/// The original hardcodes eight players in fixed-size arrays throughout. We do
/// not; nothing here assumes a count.
pub type Player = u8;

/// Position units per tile.
pub const UNITS_PER_TILE: i32 = 192;
/// Tiles per world cell.
pub const TILES_PER_CELL: i32 = 4;
/// Position units per world cell.
pub const UNITS_PER_CELL: i32 = UNITS_PER_TILE * TILES_PER_CELL;

/// A position in the original's stored coordinate units.
///
/// In a save file or in memory these are XOR-masked with `0x63637`, a
/// tamper-resistance measure. Decoding is the reader's job; by the time a
/// coordinate is a `Pos` it is a plain number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

/// A world cell, the unit ownership is recorded in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub const fn new(x: i32, y: i32) -> Pos {
        Pos { x, y }
    }

    /// The cell containing this position.
    pub const fn cell(self) -> Cell {
        Cell {
            x: floor_div(self.x, UNITS_PER_CELL),
            y: floor_div(self.y, UNITS_PER_CELL),
        }
    }

    /// This position in tiles, the unit territory distances are measured in.
    pub const fn tile(self) -> Pos {
        Pos {
            x: floor_div(self.x, UNITS_PER_TILE),
            y: floor_div(self.y, UNITS_PER_TILE),
        }
    }
}

impl Cell {
    pub const fn new(x: i32, y: i32) -> Cell {
        Cell { x, y }
    }

    /// The centre of this cell, in tiles.
    ///
    /// A cell spans four tiles, so its centre falls on the half-tile `4c + 2`.
    /// The original works in whole tiles and takes the `+ 2` as exact, which
    /// biases every distance from a cell centre by half a tile in each axis —
    /// deliberately, since the alternative would have been a fraction.
    pub const fn centre_tile(self) -> Pos {
        Pos {
            x: self.x * TILES_PER_CELL + TILES_PER_CELL / 2,
            y: self.y * TILES_PER_CELL + TILES_PER_CELL / 2,
        }
    }
}

/// The original's integer hypotenuse, and the only distance the simulation
/// measures.
///
/// With `hi` the larger of `|dx|` and `|dy|` and `lo` the smaller, this is
/// `hi + lo² / (2·hi)` — a first-order approximation that is exact on the
/// axes, worst on the diagonal, and never a square root. It is what makes Rise
/// of Nations' borders read as faintly octagonal rather than circular;
/// substituting a true hypotenuse would visibly change every border.
///
/// The original has a second branch, `hi + lo / 2`, guarded by `lo < 60000`.
/// That is an overflow guard on `lo * lo`, not a shape decision: no map is
/// sixty thousand tiles across, so the branch is unreachable in play. It is
/// kept because it costs nothing and because leaving it out would quietly
/// change behaviour at a size the original defined.
///
/// It lives here, in world, rather than in the subsystem that needed it first,
/// because it turned out to be shared: the territory pass inlines this
/// arithmetic and `Supplies::find_supply` calls the engine's own `vector_dist`,
/// which is the same thing. Supply radii are octagonal in exactly the way
/// borders are, and one copy is what keeps them from drifting apart.
///
/// The name is the engine's.
pub const fn vector_dist(dx: i32, dy: i32) -> i32 {
    let (dx, dy) = (dx.abs(), dy.abs());
    let (hi, lo) = if dx >= dy { (dx, dy) } else { (dy, dx) };
    if hi == 0 {
        return 0;
    }
    if lo < 60_000 {
        hi + (lo * lo) / (hi * 2)
    } else {
        hi + lo / 2
    }
}

/// Floor division. `i32::div_euclid` agrees for the positive divisors used
/// here and is what the original's lookup table encodes.
const fn floor_div(n: i32, d: i32) -> i32 {
    let q = n / d;
    if n % d < 0 { q - 1 } else { q }
}

/// Who owns a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Owner {
    /// No claim reached this cell, or the nearest claim cost too much.
    #[default]
    None,
    /// A claim won the cell, but the winning city's own record of its owner
    /// disagreed with the player whose list it was found under. The original
    /// writes `-2` here, and every consumer treats it as unowned; it survives
    /// only because it is observable in a save file.
    Ambiguous,
    Player(Player),
}

impl Owner {
    /// The owning player, if the cell is owned by anybody in particular.
    pub const fn player(self) -> Option<Player> {
        match self {
            Owner::Player(p) => Some(p),
            _ => None,
        }
    }

    /// Whether any claim was recorded, ambiguous or not. This is the
    /// original's `!= -1` test, which is not the same as [`Owner::player`].
    pub const fn is_claimed(self) -> bool {
        !matches!(self, Owner::None)
    }
}

/// Whether a region is land or water. Territory is computed per land region;
/// water is never owned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terrain {
    Land,
    Sea,
}

/// The rest of a cell's record — the original's `WData` (`World+0x134`,
/// `0x1c` bytes a cell; `struct /rise.pdb/WData`) beyond the owner and the
/// region the [`World`] already keeps. The map maker writes these once; the
/// production AI reads them (`docs/AI.md` §2.3, §2.7, §2.13) and so does the
/// pathfinder's cost function. A headless world that never sets them
/// answers every field with zero — the flat map the harness has always
/// assumed — and the `WORLD=6` start-of-game dump is what fills them
/// (`docs/ORACLE.md`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CellData {
    /// `WData.flags`: `0x8000` … `0x2` as `WData::log_data` names them;
    /// `0x100` marks a coastal cell whose `region2` is the sea region,
    /// `0x4000` a cell holding a building's centre, `0x78` the
    /// `buildings_allowed` field.
    pub flags: u16,
    /// `WData.land`: the terrain kind (`land_key[land]` in the dump).
    pub land: i8,
    pub land_sub: u8,
    /// `WData.region2`: the alternate region — a coastal land cell's sea
    /// region — or `None` for `-1`.
    pub region2: Option<u16>,
    /// `WData.val`: the city-site value byte the site score starts from.
    pub val: u8,
    /// `WData.goods`: a bit per good gatherable near this cell.
    pub goods: u8,
    /// `WData.blocked`, `WData.solid`.
    pub blocked: u8,
    pub solid: i8,
    /// `WData.down`, `WData.down_who`: the object chained at this cell.
    pub down: i16,
    pub down_who: i8,
}

/// The world: a grid of cells, each in at most one region, each with a primary
/// and a runner-up claimant.
#[derive(Clone, Debug)]
pub struct World {
    width: i32,
    height: i32,
    who: Vec<Owner>,
    who2: Vec<Owner>,
    /// Region index per cell. The original caps this at 64 land and 64 sea
    /// regions because it stores the set in a `BitMask<64>`; we do not.
    region: Vec<Option<u16>>,
    regions: Vec<Terrain>,
    /// One mask per **tile** — four by four per cell — the original's `TData`
    /// (`World +0x138`, one `ushort` each). Placement reads it, buildings mark
    /// it; see `docs/CITIES.md` §2.3 for the bit legend, and [`tile`] for the
    /// names.
    tiles: Vec<u16>,
    /// The rest of each cell's `WData` record — [`CellData`]; all zero until
    /// a map is loaded.
    cells: Vec<CellData>,
    /// `WorldData::danger[who][region]` — the per-player danger figure the
    /// AI's trainers and placement read. Empty until something writes it;
    /// [`World::danger`] answers 0 then.
    danger: Vec<Vec<i32>>,
    /// One height per **tile** — what `TerrainOut::find_tcoord_z@008544a0`
    /// answers for it: the truncated mean of two corners of the terrain's
    /// float height grid, `(int)((h[ty+1][tx] + h[ty][tx+1]) × 0.5)`, and 0
    /// on an ocean tile. A pinned table the loader builds from the dump's
    /// `master_land_heights` before the first frame (`docs/DECISIONS.md`
    /// entry 16's clause); empty on a flat world, where every tile is 0.
    tile_z: Vec<i32>,
    /// `world+0x34` — see [`World::sea_map`].
    sea_map: i32,
    /// `WorldData::seen2` — the fog grid, two entries per cell each way
    /// (`fog_xs = 2 × xs`), one bit per player: what `was_seen` answers.
    /// Empty until a dump supplies it, and then a snapshot the simulation
    /// does not yet advance (nothing here models sight); see
    /// [`World::seen2`].
    fog: Vec<u8>,
}

/// The bits of a tile mask, as the placement code names them — `TData.mask`
/// in the original. Two-bit fields are tested as `(mask & field) == value`.
pub mod tile {
    /// The two-bit terrain-object field: `3` a building footprint, `2` a
    /// mountain.
    pub const OBJECT: u16 = 0x3;
    pub const OBJECT_BUILDING: u16 = 0x3;
    pub const OBJECT_MOUNTAIN: u16 = 0x2;
    /// The two-bit surface field: `0` plain land, `0x10` road, `0x20` ocean,
    /// `0x30` forest.
    pub const SURFACE: u16 = 0x30;
    pub const SURFACE_ROAD: u16 = 0x10;
    pub const SURFACE_OCEAN: u16 = 0x20;
    pub const SURFACE_FOREST: u16 = 0x30;
    /// A second building placed on this tile (`start_me`).
    pub const PLACED_TWICE: u16 = 0x40;
    /// A building placed, not yet started, here (`start_me`).
    pub const PLACED: u16 = 0x80;
    /// Inside some city's radius (`Wall::mask_city`).
    pub const CITY_RADIUS: u16 = 0x100;
    /// Treated as a building (unnamed in the original).
    pub const AS_BUILDING: u16 = 0x200;
    /// River.
    pub const RIVER: u16 = 0x800;
    /// Next to something blocked (`set_bad_path`).
    pub const BAD_PATH: u16 = 0x2000;
    /// Blocked (`set_blocked_at`).
    pub const BLOCKED: u16 = 0x4000;
}

impl World {
    /// An unowned world of `width` by `height` cells, with no regions yet.
    pub fn new(width: i32, height: i32) -> World {
        assert!(width > 0 && height > 0, "world must have positive extent");
        let n = (width as usize) * (height as usize);
        World {
            width,
            height,
            who: vec![Owner::None; n],
            who2: vec![Owner::None; n],
            region: vec![None; n],
            regions: Vec::new(),
            tiles: vec![0; n * (TILES_PER_CELL as usize) * (TILES_PER_CELL as usize)],
            cells: vec![CellData::default(); n],
            danger: Vec::new(),
            tile_z: Vec::new(),
            sea_map: 0,
            fog: Vec::new(),
        }
    }

    /// Install the fog grid's `seen2` bytes, `(2 × width) × (2 × height)`
    /// row-major; any other length is refused and the world stays fogless.
    pub fn set_fog(&mut self, seen2: Vec<u8>) -> bool {
        let n = (self.width as usize) * (self.height as usize) * 4;
        if seen2.len() != n {
            return false;
        }
        self.fog = seen2;
        true
    }

    /// `WorldData::seen2[fy × fog_xs + fx]` — the fog cell's seen bits, one
    /// per player; `None` when no fog grid has been installed (the caller
    /// keeps its "always seen" reading then) or off the grid.
    pub fn seen2(&self, fx: i32, fy: i32) -> Option<u8> {
        if self.fog.is_empty() {
            return None;
        }
        let (fw, fh) = (self.width * 2, self.height * 2);
        if fx < 0 || fy < 0 || fx >= fw || fy >= fh {
            return None;
        }
        self.fog.get((fy * fw + fx) as usize).copied()
    }

    /// A tile's height as `find_tcoord_z` answers it; 0 off the map and on
    /// a world without a height table.
    pub fn tile_z(&self, t: Pos) -> i32 {
        self.tile_index(t)
            .and_then(|i| self.tile_z.get(i))
            .copied()
            .unwrap_or(0)
    }

    /// Writes a tile's height (the map loader's).
    pub fn set_tile_z(&mut self, t: Pos, z: i32) {
        if let Some(i) = self.tile_index(t) {
            if self.tile_z.is_empty() {
                self.tile_z = vec![0; self.tiles.len()];
            }
            self.tile_z[i] = z;
        }
    }

    /// The rest of a cell's record — zero off the map and on a flat world.
    pub fn cell_data(&self, c: Cell) -> CellData {
        self.index(c)
            .map_or_else(CellData::default, |i| self.cells[i])
    }

    /// Writes a cell's record (the map loader's, and a test's).
    pub fn set_cell_data(&mut self, c: Cell, d: CellData) {
        if let Some(i) = self.index(c) {
            self.cells[i] = d;
        }
    }

    /// `WData.val` of the cell under a tile — the site value.
    pub fn site_value(&self, t: Pos) -> i32 {
        i32::from(self.cell_data(Self::cell_of_tile(t)).val)
    }

    /// `WData.goods` of the cell under a tile — the nearby-goods bits.
    pub fn goods_near(&self, t: Pos) -> u8 {
        self.cell_data(Self::cell_of_tile(t)).goods
    }

    /// `WorldData::get_tregion` with the coastal refinement: a cell flagged
    /// `0x100` answers its `region2` (the sea region) when the tile's own
    /// surface is ocean. `docs/AI.md` §2.3 step 10.
    pub fn tregion_alt(&self, t: Pos) -> Option<u16> {
        let c = Self::cell_of_tile(t);
        let d = self.cell_data(c);
        let coastal_water =
            d.flags & 0x100 != 0 && self.tile_mask(t) & tile::SURFACE == tile::SURFACE_OCEAN;
        match d.region2 {
            Some(r) if coastal_water => Some(r),
            _ => self.region_of(c),
        }
    }

    /// `WorldData::danger[who][region]`; 0 when never written.
    pub fn danger(&self, who: Player, region: u16) -> i32 {
        self.danger
            .get(who as usize)
            .and_then(|d| d.get(region as usize))
            .copied()
            .unwrap_or(0)
    }

    /// Writes a danger figure, growing the table as needed.
    pub fn set_danger(&mut self, who: Player, region: u16, value: i32) {
        let w = who as usize;
        if self.danger.len() <= w {
            self.danger.resize_with(w + 1, Vec::new);
        }
        let r = region as usize;
        if self.danger[w].len() <= r {
            self.danger[w].resize(r + 1, 0);
        }
        self.danger[w][r] = value;
    }

    /// How many regions the world has, land and sea — the length every
    /// per-region census array takes.
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    /// `world+0x34`, `sea_map`: the map's **sea class**, a property of the
    /// map style and not a count of anything on the map. Every `mapstyles/
    /// *.xml` carries `<SEA_MAP value="n"/>` (0 the land maps, 1 Great
    /// Lakes / Mediterranean / Outback, 2 Warring States, 3 the two-shore
    /// and island maps, 4 Colonial Powers), `Map::init_map_data` reads it
    /// with −1 for absent, `Map::make` copies it into the world when it is
    /// not −1, and the conquest maker's `check_sea_map` computes 0/2/3
    /// (no sea / every start on one landmass / starts apart) for the
    /// styles that leave it out. The AI reads it as "how much sea": `> 2`
    /// is the coastal-ring and dock-value predicate. It was read as "the
    /// number of land regions" until run20 (2026-08-25) reported 4 on a
    /// map with twelve of them; 0 on a world with no map loaded.
    pub fn sea_map(&self) -> i32 {
        self.sea_map
    }

    pub fn set_sea_map(&mut self, v: i32) {
        self.sea_map = v;
    }

    /// `world+0x8`, `size`: the cell count.
    pub const fn cell_count(&self) -> i32 {
        self.width * self.height
    }

    pub const fn width(&self) -> i32 {
        self.width
    }

    pub const fn height(&self) -> i32 {
        self.height
    }

    pub const fn contains(&self, c: Cell) -> bool {
        c.x >= 0 && c.y >= 0 && c.x < self.width && c.y < self.height
    }

    fn index(&self, c: Cell) -> Option<usize> {
        self.contains(c)
            .then(|| (c.y as usize) * (self.width as usize) + (c.x as usize))
    }

    /// Declares a region and returns its index.
    pub fn add_region(&mut self, terrain: Terrain) -> u16 {
        self.regions.push(terrain);
        u16::try_from(self.regions.len() - 1).expect("too many regions")
    }

    /// Puts a cell in a region, replacing any previous membership.
    pub fn set_region(&mut self, c: Cell, region: u16) {
        assert!(
            (region as usize) < self.regions.len(),
            "no such region: {region}"
        );
        if let Some(i) = self.index(c) {
            self.region[i] = Some(region);
        }
    }

    /// Puts every cell of a rectangle in a region. Convenient for tests and
    /// for the flat worlds the headless harness builds.
    pub fn fill_region(&mut self, terrain: Terrain, from: Cell, to: Cell) -> u16 {
        let r = self.add_region(terrain);
        for y in from.y..=to.y {
            for x in from.x..=to.x {
                self.set_region(Cell::new(x, y), r);
            }
        }
        r
    }

    pub fn terrain(&self, region: u16) -> Terrain {
        self.regions[region as usize]
    }

    pub fn regions(&self) -> impl Iterator<Item = (u16, Terrain)> + '_ {
        self.regions.iter().enumerate().map(|(i, t)| (i as u16, *t))
    }

    pub fn region_of(&self, c: Cell) -> Option<u16> {
        self.index(c).and_then(|i| self.region[i])
    }

    /// Every cell of a region, in row-major order.
    ///
    /// The original keeps an explicit cell list per region and walks it. The
    /// result is the same and the order is the same; keeping the list would
    /// only duplicate what the grid already knows.
    pub fn cells_in(&self, region: u16) -> impl Iterator<Item = Cell> + '_ {
        (0..self.height).flat_map(move |y| {
            (0..self.width).filter_map(move |x| {
                let c = Cell::new(x, y);
                (self.region_of(c) == Some(region)).then_some(c)
            })
        })
    }

    /// Whether the world will accept a position — the bounds test
    /// `Unit::move_step` makes before committing the unit's step, and the
    /// original's `WorldData::is_valid`, which `Guy::move` asks for the body's.
    ///
    /// A refused step is not an error and does not stop the order: the unit
    /// simply does not move that frame and tries again on the next.
    pub fn accepts(&self, p: Pos) -> bool {
        self.index(p.cell()).is_some()
    }

    /// Who owns a cell. Cells outside the world are unowned.
    pub fn owner(&self, c: Cell) -> Owner {
        self.index(c).map_or(Owner::None, |i| self.who[i])
    }

    /// The runner-up claimant of a cell.
    pub fn second(&self, c: Cell) -> Owner {
        self.index(c).map_or(Owner::None, |i| self.who2[i])
    }

    pub fn set_owner(&mut self, c: Cell, who: Owner, who2: Owner) {
        if let Some(i) = self.index(c) {
            self.who[i] = who;
            self.who2[i] = who2;
        }
    }

    /// Who owns the cell a position falls in.
    pub fn owner_at(&self, p: Pos) -> Owner {
        self.owner(p.cell())
    }

    /// Whether a tile coordinate is on the map — `blocked_tcoord`'s first
    /// test.
    pub const fn tile_in_bounds(&self, t: Pos) -> bool {
        t.x >= 0
            && t.y >= 0
            && t.x < self.width * TILES_PER_CELL
            && t.y < self.height * TILES_PER_CELL
    }

    fn tile_index(&self, t: Pos) -> Option<usize> {
        self.tile_in_bounds(t).then(|| {
            (t.y as usize) * (self.width as usize) * (TILES_PER_CELL as usize) + (t.x as usize)
        })
    }

    /// The mask of a tile coordinate (see [`tile`]); zero off the map.
    pub fn tile_mask(&self, t: Pos) -> u16 {
        self.tile_index(t).map_or(0, |i| self.tiles[i])
    }

    /// Sets bits of a tile's mask.
    pub fn set_tile_bits(&mut self, t: Pos, bits: u16) {
        if let Some(i) = self.tile_index(t) {
            self.tiles[i] |= bits;
        }
    }

    /// Clears bits of a tile's mask.
    pub fn clear_tile_bits(&mut self, t: Pos, bits: u16) {
        if let Some(i) = self.tile_index(t) {
            self.tiles[i] &= !bits;
        }
    }

    /// Replaces a tile's whole mask (the map loader's).
    pub fn set_tile_mask(&mut self, t: Pos, mask: u16) {
        if let Some(i) = self.tile_index(t) {
            self.tiles[i] = mask;
        }
    }

    /// Replaces a two-bit field of a tile's mask.
    pub fn set_tile_field(&mut self, t: Pos, field: u16, value: u16) {
        if let Some(i) = self.tile_index(t) {
            self.tiles[i] = (self.tiles[i] & !field) | (value & field);
        }
    }

    /// The cell a tile coordinate lies in.
    pub const fn cell_of_tile(t: Pos) -> Cell {
        Cell {
            x: floor_div(t.x, TILES_PER_CELL),
            y: floor_div(t.y, TILES_PER_CELL),
        }
    }

    /// The region of the cell under a tile — `WorldData::get_tregion`,
    /// without the coastal `region2` refinement.
    pub fn tregion(&self, t: Pos) -> Option<u16> {
        self.region_of(Self::cell_of_tile(t))
    }

    /// How many cells a region has — `Region.size`.
    pub fn region_size(&self, region: u16) -> i32 {
        self.region.iter().filter(|r| **r == Some(region)).count() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_conversion_matches_the_originals_two_step_form() {
        // The original does `div_3_table[pos >> 8]` for a cell and
        // `div_3_table[pos >> 6]` for a tile, where the table is floor(i / 3).
        // Nesting two floor divisions is one floor division by the product,
        // which is what Pos::cell does directly. Check it over a range that
        // straddles zero, since that is the only place the two could differ.
        for p in -4000..4000i32 {
            let cell_two_step = floor_div(p >> 8, 3);
            let tile_two_step = floor_div(p >> 6, 3);
            let pos = Pos::new(p, p);
            assert_eq!(pos.cell().x, cell_two_step, "cell of {p}");
            assert_eq!(pos.tile().x, tile_two_step, "tile of {p}");
        }
    }

    #[test]
    fn the_three_units_relate_as_claimed() {
        assert_eq!(UNITS_PER_CELL, 768);
        assert_eq!(Pos::new(768, 1535).cell(), Cell::new(1, 1));
        assert_eq!(Pos::new(767, 0).cell(), Cell::new(0, 0));
        assert_eq!(Pos::new(-1, -768).cell(), Cell::new(-1, -1));
        assert_eq!(Pos::new(192 * 5, 0).tile().x, 5);
    }

    #[test]
    fn cell_centres_land_on_the_half_tile() {
        assert_eq!(Cell::new(0, 0).centre_tile(), Pos::new(2, 2));
        assert_eq!(Cell::new(3, 7).centre_tile(), Pos::new(14, 30));
    }

    #[test]
    fn the_hypotenuse_is_exact_on_the_axes() {
        for n in 0..200 {
            assert_eq!(vector_dist(n, 0), n);
            assert_eq!(vector_dist(0, -n), n);
        }
    }

    #[test]
    fn the_hypotenuse_overshoots_on_the_diagonal() {
        // 3-4-5 comes out exact by luck: 4 + 9/8 = 5.
        assert_eq!(vector_dist(3, 4), 5);
        // The pure diagonal is where the approximation is worst: it returns
        // 1.5x the leg where the true answer is 1.414x, so borders bulge
        // *inward* at the corners and the outline reads as an octagon.
        assert_eq!(vector_dist(40, 40), 60);
        assert_eq!(vector_dist(100, 100), 150);
    }

    #[test]
    fn the_hypotenuse_is_symmetric_and_off_by_at_most_a_truncation() {
        for dx in -60..60 {
            for dy in -60..60 {
                let d = vector_dist(dx, dy);
                assert_eq!(d, vector_dist(dy, dx), "{dx},{dy}");
                assert_eq!(d, vector_dist(-dx, dy), "{dx},{dy}");
                // Never shorter than the longer leg.
                assert!(d >= dx.abs().max(dy.abs()), "{dx},{dy} gave {d}");
                // The exact first-order form is an upper bound on the true
                // distance, but the division truncates, so the result can land
                // one below — 60,30 is the first case, giving 67 where the
                // true distance is 67.08. It is never worse than that.
                let (d, true_sq) = (
                    i64::from(d),
                    i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy),
                );
                assert!((d + 1) * (d + 1) >= true_sq, "{dx},{dy} gave {d}");
            }
        }
    }

    #[test]
    fn ambiguous_is_claimed_but_owned_by_nobody() {
        assert!(Owner::Ambiguous.is_claimed());
        assert_eq!(Owner::Ambiguous.player(), None);
        assert!(!Owner::None.is_claimed());
        assert_eq!(Owner::Player(3).player(), Some(3));
    }

    #[test]
    fn regions_partition_the_grid() {
        let mut w = World::new(4, 4);
        let land = w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(1, 3));
        let sea = w.fill_region(Terrain::Sea, Cell::new(2, 0), Cell::new(3, 3));
        assert_eq!(w.cells_in(land).count(), 8);
        assert_eq!(w.cells_in(sea).count(), 8);
        assert_eq!(w.terrain(land), Terrain::Land);
        assert_eq!(w.region_of(Cell::new(3, 3)), Some(sea));
    }
}
