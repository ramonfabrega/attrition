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
        }
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
