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

/// The leader slots the *player-facing* world uses, and the first gaia one.
///
/// `Leaders::list` is `Leader[10]` (`rise.pdb`, `sizeof(Leader) = 0x6eec`):
/// eight players and then 8 and 9, gaia's animals and gaia's birds. Every
/// object search stops at this bound, and the original says so twice —
///
/// - `ObjectsData::find_unit@0065ca80` walks the per-leader object lists with
///   a stride of `0x6eec` while the cursor is `< 0x37760`, which is exactly
///   eight of them, and its by-cell branch guards `(int)leader < 8` outright
///   before it will even call `Search::valid_search`;
/// - `ObjectData::valid_target_const@006472c0` returns 0 on `7 < who` in its
///   **first line**, before it would reach `LeaderData::is_enemy`.
///
/// So gaia's units cannot be found by a search, cannot be a valid target, and
/// cannot be hit by ammunition (`Ammo::check_hit@00678d90` is a `find_unit`).
/// The second bound is also why the original never asks a diplomacy question
/// about them: `LeaderData::diplos` is `int[8]`, so `is_enemy(8)` would read
/// `treaties[0]` — the original has no answer there either.
pub const PLAYER_SLOTS: Player = 8;

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
/// The original has a second branch, `hi + lo / 2`, guarded by `lo < 60000`,
/// and both are computed **unsigned**. The guard is an overflow guard on
/// `lo * lo` — but for `unsigned`, and in world units, not tiles: 60,000
/// units is 78 cells, and `lo²` passes `i32::MAX` from `lo = 46341`, sixty
/// cells short of the guard. Squaring in `i32` here panicked in debug and
/// wrapped in release for every diagonal past sixty cells, until the
/// emulated original (`tools/emu/callfn.py`, `path::tests`) answered 89998
/// for `(59999, 59999)` on 2026-09-01.
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
    let (hi, lo) = (hi as u32, lo as u32);
    if lo < 60_000 {
        (hi + (lo * lo) / (hi * 2)) as i32
    } else {
        (hi + lo / 2) as i32
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
    /// `WData.flags` — the bits [`cell`] names; `0x78` is also read whole as
    /// the `buildings_allowed` field.
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
    /// `WData.blocked`, `WData.solid` — **counts, not flags**: how many of
    /// the cell's sixteen tiles carry [`tile::BLOCKED`], and how many of
    /// those a forest-walker is stopped by too.
    /// [`World::set_blocked_at`] is what keeps them.
    pub blocked: u8,
    pub solid: i8,
    /// `WData.bad`: how many of the cell's tiles carry [`tile::BAD_PATH`],
    /// kept by the same function. Nothing reads it here yet; it is loaded
    /// and maintained so the dump's own column can be compared.
    pub bad: u8,
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
    /// `Region +0x8 flags`, by region — the map generator's own word
    /// (`Map::region_flags`). Bit `8` is the **resource-region** flag, and
    /// it is the whole of `Region::go_here`'s first arm
    /// (`docs/TRANSPORT.md` §9.4). Nothing in the cells implies it, so it
    /// arrives from a dump's `REGIONS` block or not at all; a world without
    /// one has every region at 0, and `go_here` never answers bit 1.
    region_flags: Vec<i32>,
    /// `Region +0x34 BitMask<8> scouted`, one bit a leader, by region —
    /// "this leader's scout has looked here and found nothing".
    /// `Unit::think_scout`'s tail is the only writer and
    /// `think_civilian_transport` the only reader (`docs/TRANSPORT.md` §7).
    scouted: Vec<u8>,
    /// One mask per **tile** — four by four per cell — the original's `TData`
    /// (`World +0x138`, one `ushort` each). Placement reads it, buildings mark
    /// it; see `docs/CITIES.md` §2.3 for the bit legend, and [`tile`] for the
    /// names.
    tiles: Vec<u16>,
    /// The rest of each cell's `WData` record — [`CellData`]; all zero until
    /// a map is loaded.
    cells: Vec<CellData>,
    /// `WorldData::danger[who]@+0x13c` — the per-player danger grid,
    /// `int[reg_size]` per leader over a **half-resolution cell** grid
    /// (`reg_xs × reg_ys`, each entry two cells square).
    ///
    /// [`crate::danger`] is the writer (`GameDaemon::calc_danger`, every
    /// two hundredth frame) and every consumer indexes it the one way the
    /// executable does — `reg_xs × div3(y >> 9) + div3(x >> 9)`, which is
    /// [`World::danger_half`] at the position's cell. Eight rows, one per
    /// leader; Gaia has none.
    danger: Vec<Vec<i32>>,
    /// One height per **tile** — what `TerrainOut::find_tcoord_z@008544a0`
    /// answers for it: the truncated mean of two corners of the terrain's
    /// float height grid, `(int)((h[ty+1][tx] + h[ty][tx+1]) × 0.5)`, and 0
    /// on an ocean tile. A pinned table the loader builds from the dump's
    /// `master_land_heights` before the first frame (`docs/DECISIONS.md`
    /// entry 16's clause); empty on a flat world, where every tile is 0.
    tile_z: Vec<i32>,
    /// `TerrainData::master_land_heights` — the **corner** grid
    /// `find_tcoord_z` reads, `4·xs + 1` columns by `4·ys + 1` rows, in
    /// exact millionths of a world unit (the scale the dump prints, and
    /// the one `crate::terrain` does its arithmetic at).
    ///
    /// [`World::tile_z`] is the pinned per-tile table derived from it; this
    /// is here because a building **re-terraforms** the grid when it is
    /// placed (`TerrainOut::terraform_for_building@00875210`,
    /// `docs/ROADS.md` §7.4), so the table is no longer built once before
    /// the first frame. Empty on a flat world.
    corner_z: Vec<i64>,
    /// Per corner, whether [`World::set_corner_z`] has changed it since the
    /// grid was installed. The terraform works in millionths where the
    /// original works in singles (`docs/DECISIONS.md` entry 31), so a
    /// rewritten corner is no longer known to be the original's single —
    /// [`World::data_z`] reports a read of one. Empty until the first write.
    corner_rewritten: Vec<bool>,
    /// The map's goods, in the original's own `goods` list order — the
    /// index `WData.down_who` carries when `down` is `−2`.
    goods: Vec<Good>,
    /// Per cell, the good its object chain **terminates** in, or `-1`.
    ///
    /// `Objects::init_good@00653f30` writes `wdata[cell].down = −2` and
    /// `down_who = <goods index>` for every good whose type is not `OIL`,
    /// and `Object::add_to_world` only ever pushes onto the *head*, so the
    /// terminator a chain walk reaches is this and stays this. That is the
    /// whole of `ObjectsData::find_good_at`'s fast path, without the chain.
    cell_good: Vec<i32>,
    /// `world+0x34` — see [`World::sea_map`].
    sea_map: i32,
    /// `WorldData::seen2` (`World +0x160`) — the fog grid, two entries per
    /// cell each way (`fog_xs = 2 × xs`), one bit per player: what
    /// `was_seen` and `was_really_seen` answer from. Empty until a dump or
    /// [`World::set_fog`] supplies it; **monotone**, since nothing in a
    /// running game clears it (`docs/VISION.md` §1). [`World::set_seen`] is
    /// what grows it as units move.
    fog: Vec<u8>,
    /// `WorldData::seen` (`World +0x15c`) — current line of sight, the same
    /// shape as [`World::fog`]. Written by the same call and cleared whole
    /// only by `update_all_seen`, every hundredth frame, so between
    /// resyncs it only ever grows (`docs/VISION.md` §6). Nothing in this
    /// simulation reads it yet; it is kept because the write is free and
    /// the `WORLD` dump prints it beside `seen2`.
    fog_now: Vec<u8>,
    /// `Region::coast` per region: for a land region, the sea regions any
    /// of its cells touches in the eight-neighbourhood — `Regions::
    /// set_coastals`' first mask (`docs/TRANSPORT.md` §9.1). Sorted,
    /// deduplicated, empty for a sea region; rebuilt by
    /// [`World::rebuild_coasts`].
    coast: Vec<Vec<u16>>,
}

/// The bits of `WData.flags`, as `WData::log_data@006af7e0` spells them —
/// each word below is printed exactly when its bit is set, solved from
/// run20's 3,600 cells (`docs/ORACLE.md`, "The map is a dump too";
/// 2026-08-25). `0x1`, `0x400`, `0x800`, `0x1000` and `0x2000` have no word
/// in that dump: the writer tests `0x1000` and `0x2000` and run20 never
/// sets them; `0x400` has no name here. `0x800` is **`OIL`**, settled by
/// `WorldData::is_oil_at@00472af0`, which is that bit and nothing else.
pub mod cell {
    /// `COAST` — a land cell of the shore; the muster search's class 3.
    pub const COAST: u16 = 0x4;
    /// `ROCK` — `WorldData::is_rocks@006b4380`; the land class 6, or 7
    /// with [`OIL`].
    pub const ROCK: u16 = 0x8;
    /// `MOUNTAIN` — with the unnamed `0x40`, the muster search's class 5.
    pub const MOUNTAIN: u16 = 0x10;
    /// `FOREST` — the muster search's class 4.
    pub const FOREST: u16 = 0x20;
    /// `ROAD`.
    pub const ROAD: u16 = 0x80;
    /// `HALFLAND` — a coastal cell whose `region2` is the sea region; what
    /// `WorldData::is_ocean` tests first.
    pub const HALFLAND: u16 = 0x100;
    /// `NEARBLOCK`.
    pub const NEARBLOCK: u16 = 0x200;
    /// `WorldData::is_oil_at@00472af0` is `flags & 0x800`, and nothing
    /// else — so the bit the map dump prints no word for is oil. It is
    /// what turns a rock cell into land class 7 and a water cell into an
    /// offshore oil patch ([`World::land_class`]).
    pub const OIL: u16 = 0x800;
    /// `BUILDING` — a cell holding a building's centre.
    pub const BUILDING: u16 = 0x4000;
    /// `GOODY`.
    pub const GOODY: u16 = 0x8000;
}

/// `move_x[1..=8]`, `move_y[1..=8]` — the original's compass ring
/// (`.rdata 0x00adcaf0` / `0x00adc400`), the eight neighbours in its order.
pub const MOVE_8: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
];

/// `corner_x[0..5]@00adc3e0` / `corner_y[0..5]@00adc3c0` — the centre and
/// its four corners, in `compass.obj` beside [`MOVE_8`] and read out of the
/// PE against `rise_z.map`. Two mechanics index them: the nomad's wood test
/// (`Leader::compute_sites`, `docs/AI.md` §11) and a pasture animal's own
/// place in its five, which is what `Animal+0x154` is for
/// (`docs/SYNC.md` §3.11).
pub const CORNER_X: [i32; 5] = [0, -1, 1, 1, -1];
pub const CORNER_Y: [i32; 5] = [0, -1, -1, 1, 1];

/// `move_x[0..0x31]`, `move_y[0..0x31]` — the original's walk of the 7 × 7
/// neighbourhood: the cell itself, the eight neighbours as [`MOVE_8`], the
/// sixteen cells of the 5 × 5 ring (the twelve edge cells clockwise from
/// north-north-west, then the four corners), then the twenty-four of the
/// 7 × 7 ring clockwise from its north-west corner. Read from
/// `riseofnations.exe` on 2026-08-25: `rise.pdb` places `move_x` at
/// `0002:97008` (VA `0xADCAF0`) and `move_y` at `0002:95232` (VA
/// `0xADC400`), each an `int[441]` — the whole 21 × 21 — of which the muster
/// search (`docs/ARMY.md` §13) walks the first 49.
pub const MOVE_49: [(i32, i32); 49] = [
    (0, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -2),
    (0, -2),
    (1, -2),
    (2, -1),
    (2, 0),
    (2, 1),
    (1, 2),
    (0, 2),
    (-1, 2),
    (-2, 1),
    (-2, 0),
    (-2, -1),
    (-2, -2),
    (2, -2),
    (2, 2),
    (-2, 2),
    (-3, -3),
    (-2, -3),
    (-1, -3),
    (0, -3),
    (1, -3),
    (2, -3),
    (3, -3),
    (3, -2),
    (3, -1),
    (3, 0),
    (3, 1),
    (3, 2),
    (3, 3),
    (2, 3),
    (1, 3),
    (0, 3),
    (-1, 3),
    (-2, 3),
    (-3, 3),
    (-3, 2),
    (-3, 1),
    (-3, 0),
    (-3, -1),
    (-3, -2),
];

/// `move_x[0 .. 0x121]` / `move_y[0 .. 0x121]` — the whole 17 × 17 the
/// original's widest neighbourhood walk covers, of which [`MOVE_49`] is the
/// first 49. `Unit::think_fish` is the caller that reads all of it
/// (`docs/ORDERS.md` §6.8).
///
/// Rings 4–8 are the plain clockwise walk from the ring's north-west
/// corner, so they are generated rather than typed; ring 2's corner quirk
/// lives in [`MOVE_49`], which is copied in.
///
/// **`move_y[288]` is `−16`, not `−7`.** The very last entry of ring 8 —
/// the one cell that closes the square, `(−8, −7)` — is stored with the
/// wrong `y`: `.rdata 0x00adc880` holds `f0 ff ff ff`, and the neighbours
/// either side are the `−6` and the `−9` a clean table wants (`rise.pdb`
/// types both arrays `int[441]`, so this is inside the array, not past it).
/// It is a typo in the shipped data, and it is **load-bearing**: run58's
/// Fisherman `1/14` takes exactly this offset out of `think_fish` on frame
/// 4462 and walks sixteen cells north, which no 17 × 17 could reach.
pub const MOVE_289: [(i32, i32); 289] = move_289();

const fn move_289() -> [(i32, i32); 289] {
    let mut out = [(0, 0); 289];
    let mut i = 0;
    while i < MOVE_49.len() {
        out[i] = MOVE_49[i];
        i += 1;
    }
    let mut r = 3;
    while r < 8 {
        r += 1;
        let mut x = -r;
        while x <= r {
            out[i] = (x, -r);
            i += 1;
            x += 1;
        }
        let mut y = -r + 1;
        while y <= r {
            out[i] = (r, y);
            i += 1;
            y += 1;
        }
        let mut x = r - 1;
        while x >= -r {
            out[i] = (x, r);
            i += 1;
            x -= 1;
        }
        let mut y = r - 1;
        while y > -r {
            out[i] = (-r, y);
            i += 1;
            y -= 1;
        }
    }
    // The typo, last.
    out[288] = (-8, -16);
    out
}

/// One of the map's **goods** — a Fish, a Whale, an Oil patch, a rare —
/// as `Objects::init_good@00653f30` lays them out before the first frame.
///
/// Nothing here consumes one yet: the list exists because
/// `ObjectsData::find_good_at` is what tells an idle fisherman which water
/// is worth standing in (`docs/ORDERS.md` §6.8), and that answer is the
/// difference between the original's Fisherman and a boat that wanders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Good {
    pub pos: Pos,
    /// The good type's `TypeIndex` — `OIL` is 5, `FISH` 6.
    pub ty: usize,
    /// `SubObjectData::flags & 1`. Nothing clears it here.
    pub alive: bool,
}

/// `TypeIndex::OIL`. `Objects::init_good` skips **every** world-side effect
/// for it — the cell's terminator included — so an oil patch is in the
/// goods list and in no cell's chain.
pub const OIL: usize = 5;

/// The bits of a tile mask, as the placement code names them — `TData.mask`
/// in the original. Two-bit fields are tested as `(mask & field) == value`.
pub mod tile {
    /// The two-bit terrain-object field: `3` a building footprint, `2` a
    /// mountain, `1` a cliff — `WorldData::is_cliff_at@0046f8c0` is exactly
    /// `(mask & 3) == 1` (`docs/SCOUT.md` §12, 2026-08-26).
    pub const OBJECT: u16 = 0x3;
    pub const OBJECT_BUILDING: u16 = 0x3;
    pub const OBJECT_MOUNTAIN: u16 = 0x2;
    pub const OBJECT_CLIFF: u16 = 0x1;
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
    /// River — `WorldData::is_river@0046d390` is this bit and nothing else.
    /// It is also the slow ground of `UnitData::get_speed`'s land arm: a
    /// land unit standing on it at or below the waterline walks at half
    /// speed ([`crate::Sim::on_river`], `docs/MOVEMENT.md`).
    pub const RIVER: u16 = 0x800;
    /// Some building already gathers from this tile —
    /// `WorldData::is_gathered_from@00472ac0` reads it and
    /// `World::set_gathered_at@006b46b0` sets it. Two writers clear it,
    /// both inline (`and $0xefff`): `Build::close@00628980`'s tail over the
    /// closing building's own list, and `Build::verify_gather_tiles@00623570`
    /// over a tile it drops (`docs/ECONOMY.md` §17). Re-exported as
    /// [`crate::gather::GATHERED_FROM`], which is where the gather-site pass
    /// that sets it lives; [`World::gather_at`] is the other reader.
    pub const GATHERED_FROM: u16 = 0x1000;
    /// Next to something blocked (`set_bad_path`).
    pub const BAD_PATH: u16 = 0x2000;
    /// Blocked (`set_blocked_at`).
    pub const BLOCKED: u16 = 0x4000;
}

/// `Region.flags`' `0x10`: every active non-flat gather building in the
/// region runs `Build::find_gather_tiles` again from `Build::process`
/// (`docs/ECONOMY.md` §17). Only [`World::cycle_gather_flags`] sets it.
pub const REGION_REFIND_GATHER: i32 = 0x10;

/// `Region.flags`' `0x20`: a gather building in the region has closed and
/// given its tiles back; the buildings processed after it this frame run
/// `Build::verify_gather_tiles`, and the next region pass turns it into
/// [`REGION_REFIND_GATHER`] (`docs/ECONOMY.md` §17).
pub const REGION_VERIFY_GATHER: i32 = 0x20;

/// How many `<LAND>` records `rules.xml` carries — `NUM_GATHER_LAND`, which
/// `Lands::init@0067e730` refuses to start without.
pub const NUM_GATHER_LAND: usize = 9;
/// How many `<MAKE>` children each carries — `NUM_MAKE`, likewise asserted.
pub const MAKES_PER_LAND: usize = 4;

/// One `<LAND>` of `rules.xml` — what a citizen takes off a tile of that
/// kind, per good.
///
/// `Lands::init@0067e730` reads the nine records into a `0x138`-stride
/// array: the four `<MAKE type>` names through `Types::good_key@006691c0`
/// at `+0x04..+0x10`, the four `<MAKE num>` amounts at `+0x14..+0x20`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Land {
    /// The `<MAKE type>` good indices. `-1` is `TYPE_NONE`, what
    /// `Types::good_key` answers for the file's `"none"` — and the
    /// original's gate on it is **unsigned** `< 6`, so it is rejected
    /// rather than indexing backwards.
    pub good: [i32; MAKES_PER_LAND],
    /// The `<MAKE num>` amounts. Zero is the other half of the gate.
    pub amount: [i32; MAKES_PER_LAND],
}

/// The shipped `LANDS` block, in file order — the index
/// [`World::land_class`] answers.
///
/// Five of the nine make anything at all: plain land gives a knowledge and
/// a food, forest a timber, mountains and cliffs a metal, an oil cell an
/// oil. Sand, ocean, coast and bare rock make nothing, which is why a city
/// ringed by them scores no gathering building at all.
///
/// `cargo run -p rondata -- <install>` re-derives this from the install's
/// own `rules.xml` and fails if it has drifted.
pub const LANDS: [Land; NUM_GATHER_LAND] = [
    // 0 Land
    Land {
        good: [3, 0, -1, -1],
        amount: [1, 1, 0, 0],
    },
    // 1 Sandy
    Land {
        good: [-1, -1, -1, -1],
        amount: [0, 0, 0, 0],
    },
    // 2 Ocean
    Land {
        good: [-1, -1, -1, -1],
        amount: [0, 0, 0, 0],
    },
    // 3 Coast
    Land {
        good: [-1, -1, -1, -1],
        amount: [0, 0, 0, 0],
    },
    // 4 Forest
    Land {
        good: [1, -1, -1, -1],
        amount: [1, 0, 0, 0],
    },
    // 5 Mountains
    Land {
        good: [4, -1, -1, -1],
        amount: [1, 0, 0, 0],
    },
    // 6 Rocks
    Land {
        good: [-1, -1, -1, -1],
        amount: [0, 0, 0, 0],
    },
    // 7 Oil
    Land {
        good: [5, -1, -1, -1],
        amount: [1, 0, 0, 0],
    },
    // 8 Cliffs
    Land {
        good: [4, -1, -1, -1],
        amount: [1, 0, 0, 0],
    },
];

/// `GoodTypeData::is_flat@004780c0` — vtable slot `+0x94` of `GoodType`,
/// which the base `Type` leaves as the engine's return-zero stub.
///
/// It is `!(is(TIMBER) || is(METAL) || is(OIL))`: food, wealth and
/// knowledge are flat, and the three the ground actually holds are not.
/// The whole shape of [`World::gather_at`] is this predicate — a flat good
/// is taken from the cell you stand on at its face amount, a non-flat one
/// is summed over the neighbourhood and doubled.
pub const fn good_is_flat(g: usize) -> bool {
    use crate::economy::Resource::{Metal, Oil, Timber};
    g != Timber as usize && g != Metal as usize && g != Oil as usize
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
            region_flags: Vec::new(),
            scouted: Vec::new(),
            tiles: vec![0; n * (TILES_PER_CELL as usize) * (TILES_PER_CELL as usize)],
            cells: vec![CellData::default(); n],
            goods: Vec::new(),
            cell_good: vec![-1; n],
            danger: Vec::new(),
            tile_z: Vec::new(),
            corner_z: Vec::new(),
            corner_rewritten: Vec::new(),
            sea_map: 0,
            fog: Vec::new(),
            fog_now: Vec::new(),
            coast: Vec::new(),
        }
    }

    /// `Regions::set_coastals@0067fd70`'s `coast` masks: for every cell of
    /// a land region, each of the eight neighbouring cells that lies in a
    /// sea region marks that sea on the land region. Run once the regions
    /// are laid out; [`Sim::new`](crate::Sim::new) runs it.
    pub fn rebuild_coasts(&mut self) {
        let n = self.regions.len();
        let mut coast: Vec<Vec<u16>> = vec![Vec::new(); n];
        for y in 0..self.height {
            for x in 0..self.width {
                let c = Cell::new(x, y);
                let Some(r) = self.region_of(c) else {
                    continue;
                };
                if self.regions[r as usize] != Terrain::Land {
                    continue;
                }
                for (dx, dy) in MOVE_8 {
                    let nc = Cell::new(x + dx, y + dy);
                    if let Some(s) = self.region_of(nc)
                        && self.regions[s as usize] == Terrain::Sea
                        && !coast[r as usize].contains(&s)
                    {
                        coast[r as usize].push(s);
                    }
                }
            }
        }
        for v in &mut coast {
            v.sort_unstable();
        }
        self.coast = coast;
    }

    /// The sea regions a land region coasts — `Region::coast`, as a list.
    pub fn coasts(&self, region: u16) -> &[u16] {
        self.coast.get(region as usize).map_or(&[], Vec::as_slice)
    }

    /// `Region::is_coast@00680f90`: the same region, or a land region and
    /// a sea region that touch.
    pub fn is_coast(&self, a: u16, b: u16) -> bool {
        if a == b {
            return true;
        }
        match (self.terrain(a), self.terrain(b)) {
            (Terrain::Land, Terrain::Sea) => self.coasts(a).contains(&b),
            (Terrain::Sea, Terrain::Land) => self.coasts(b).contains(&a),
            _ => false,
        }
    }

    /// `Region::num_coasts@00680760`: how many sea regions a land region
    /// coasts. The original counts sea indices `0x41..=0x7d` — no region
    /// is ever numbered `0x40` (`Regions::find_all` pre-increments the sea
    /// counter from it), so the count is exact — and **a sea region
    /// answers 1**, itself: the loop's `region == i` arm is the only one
    /// it can take. The census asks it of the water cell's own region
    /// (`docs/AI.md` §2.3 step 13), so its "more than one coast" clause
    /// never holds there and the size test decides (audit B.41).
    pub fn num_coasts(&self, region: u16) -> i32 {
        match self.terrain(region) {
            Terrain::Sea => 1,
            Terrain::Land => self.coasts(region).len() as i32,
        }
    }

    /// Install the fog grid's `seen2` bytes, `(2 × width) × (2 × height)`
    /// row-major; any other length is refused and the world stays fogless.
    /// The `seen` plane starts as a copy, which is what the original's own
    /// start-of-game state is — every cell an object has ever seen it is
    /// currently seeing, because nothing has moved yet.
    pub fn set_fog(&mut self, seen2: Vec<u8>) -> bool {
        let n = (self.width as usize) * (self.height as usize) * 4;
        if seen2.len() != n {
            return false;
        }
        self.fog_now.clone_from(&seen2);
        self.fog = seen2;
        true
    }

    /// Install the `seen` plane on its own, when a dump carries it.
    pub fn set_fog_now(&mut self, seen: Vec<u8>) -> bool {
        let n = (self.width as usize) * (self.height as usize) * 4;
        if seen.len() != n {
            return false;
        }
        self.fog_now = seen;
        true
    }

    /// `World::clear_seen@006b2250` on the plane this world keeps: `seen`
    /// (`World +0x15c`) zeroed whole. The original also zeroes its cell
    /// twin `+0x168`, which this world does not keep, and queues each lit
    /// cell for the fog's redraw. `seen2` is untouched. `docs/VISION.md`
    /// §10.
    pub fn clear_seen(&mut self) {
        self.fog_now.fill(0);
    }

    /// `WorldData::seen[fy × fog_xs + fx]` — current line of sight.
    pub fn seen(&self, fx: i32, fy: i32) -> Option<u8> {
        self.fog_index(fx, fy).map(|i| self.fog_now[i])
    }

    /// `World::set_seen@006b3c60` on the two planes this world keeps: ors
    /// `mask` into `seen` and `seen2` at one fog cell, and answers whether
    /// **`seen2` changed** — the original's return value, which is what
    /// gates `reveal_fog`. Off the grid it writes nothing and answers
    /// false. `docs/VISION.md` §5.
    ///
    /// The three writes the original also makes and this does not — `seen3`
    /// (gated on a flag nothing here sets), `World +0x168` and `WData
    /// +0x14` — have no reader in this simulation; §7.
    pub fn set_seen(&mut self, fx: i32, fy: i32, mask: u8) -> bool {
        let Some(i) = self.fog_index(fx, fy) else {
            return false;
        };
        self.fog_now[i] |= mask;
        let before = self.fog[i];
        self.fog[i] = before | mask;
        self.fog[i] != before
    }

    /// `World::set_seen2@006b4bb0` with its `param_4` **set** — the call
    /// `Wall::update_local_seen` makes for an ordinary building. It ors
    /// `mask` into `seen2` (and the cell's `WData +0x14`, which this world
    /// does not keep) and leaves `seen` alone, which is what keeps the
    /// second reveal from feeding back into
    /// [`Sim::check_ever_seen`](crate::Sim::check_ever_seen): that predicate
    /// reads the *current* line of sight, so a building lighting its own
    /// footprint must not make its neighbour think it has been spotted.
    /// Answers whether `seen2` changed. `docs/VISION.md` §6.1.
    pub fn set_seen2_only(&mut self, fx: i32, fy: i32, mask: u8) -> bool {
        let Some(i) = self.fog_index(fx, fy) else {
            return false;
        };
        let before = self.fog[i];
        self.fog[i] = before | mask;
        self.fog[i] != before
    }

    /// The index of a fog cell, when there is a fog grid and the pair is on
    /// it.
    fn fog_index(&self, fx: i32, fy: i32) -> Option<usize> {
        if self.fog.is_empty() {
            return None;
        }
        let (fw, fh) = (self.width * 2, self.height * 2);
        if fx < 0 || fy < 0 || fx >= fw || fy >= fh {
            return None;
        }
        Some((fy * fw + fx) as usize)
    }

    /// Whether a fog grid has been installed at all — the difference
    /// between "nothing is seen" and "everything is seen", which is what
    /// [`World::seen2`] answering `None` means to its callers.
    pub fn has_fog(&self) -> bool {
        !self.fog.is_empty()
    }

    /// The fog grid's width in fog cells, `fog_xs = 2 × xs`.
    pub const fn fog_xs(&self) -> i32 {
        self.width * 2
    }

    /// The fog grid's height in fog cells, `fog_ys = 2 × ys`.
    pub const fn fog_ys(&self) -> i32 {
        self.height * 2
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

    /// The corner grid's stride, `4·xs + 1`.
    pub const fn corner_stride(&self) -> i32 {
        self.width * TILES_PER_CELL + 1
    }

    /// One corner of `master_land_heights`, in millionths; `None` off the
    /// grid or on a world with no height table.
    pub fn corner_z(&self, x: i32, y: i32) -> Option<i64> {
        let (w, h) = (self.corner_stride(), self.height * TILES_PER_CELL + 1);
        if self.corner_z.is_empty() || x < 0 || y < 0 || x >= w || y >= h {
            return None;
        }
        self.corner_z.get((y * w + x) as usize).copied()
    }

    /// Writes one corner, and nothing else — [`World::retile_z`] is what
    /// carries the change into [`World::tile_z`].
    pub fn set_corner_z(&mut self, x: i32, y: i32, z: i64) {
        let (w, h) = (self.corner_stride(), self.height * TILES_PER_CELL + 1);
        if self.corner_z.is_empty() || x < 0 || y < 0 || x >= w || y >= h {
            return;
        }
        let i = (y * w + x) as usize;
        if self.corner_z[i] == z {
            // A terraform that leaves a corner where it was leaves it the
            // single it was loaded as.
            return;
        }
        self.corner_z[i] = z;
        if self.corner_rewritten.is_empty() {
            self.corner_rewritten = vec![false; self.corner_z.len()];
        }
        self.corner_rewritten[i] = true;
    }

    /// `TerrainOut::find_data_z@00866560(x, y, 0)` — the ground's height
    /// under a world point, as the original's SSE computes it, in
    /// [`crate::single::Single`] arithmetic (`docs/COMBAT.md` §46.2).
    ///
    /// The cell is `div_3_table[x >> 6]`, a floor of `x / 192`, clamped to
    /// the grid; the point's offset in it picks one of the cell's two
    /// triangles, split on the anti-diagonal; and each of the two terms is
    /// `(Δh × 1/192) × offset`, rounded after each step, the sum taken in
    /// the listing's own order `((base + term) + 0.0) + term` and truncated.
    /// The fourth argument is 0 at every caller this crate has — a figure's
    /// own z (`Guy::update_z`, `Unit::update_z`) and a rolled shot's
    /// ground (`Ammo::inc_time`) — so the water arm and the clamp at zero
    /// never run.
    ///
    /// Returns the height and whether every corner it read is the
    /// original's own single: a corner whose print is shared by a
    /// neighbouring single ([`crate::single::Single::from_millionths`]), or one a
    /// terraform has rewritten, is a choice and not a recovery. A world
    /// with no grid answers `(0, true)`, as [`World::tile_z`] does.
    pub fn data_z(&self, x: i32, y: i32) -> (i32, bool) {
        /// `0x3baaaaab`, the single nearest 1/192, at `0xb6943c`.
        use crate::single::Single;
        const INV_192: Single = Single::from_bits(0x3baa_aaab);
        if self.corner_z.is_empty() {
            return (0, true);
        }
        let (tw, th) = (self.width * TILES_PER_CELL, self.height * TILES_PER_CELL);
        let div3 = |v: i32| (v >> 6).max(0) / 3;
        let ix = div3(x).min(tw - 1);
        let iy = div3(y).min(th - 1);
        let stride = self.corner_stride();
        let i0 = (stride * iy + ix) as usize;
        let mut exact = true;
        let mut h = |i: usize| {
            let (v, unique) = Single::from_millionths(self.corner_z[i]);
            exact &= unique && !self.corner_rewritten.get(i).copied().unwrap_or(false);
            v
        };
        let row = stride as usize;
        let fx = x - ix * 0xc0;
        let fy = y - iy * 0xc0;
        let h01 = h(i0 + row);
        let h10 = h(i0 + 1);
        let term = |d: Single, f: i32| d.mulss(INV_192).mulss(Single::from_i32(f));
        let (base, a, b) = if fx > 0xc0 - fy {
            let h11 = h(i0 + row + 1);
            (
                h10,
                term(h11.subss(h10), fy),
                term(h01.subss(h11), 0xc0 - fx),
            )
        } else {
            let h00 = h(i0);
            (h00, term(h10.subss(h00), fx), term(h01.subss(h00), fy))
        };
        let z = base.addss(a).addss(Single::ZERO).addss(b).to_i32();
        (z, exact)
    }

    /// Installs the corner grid and derives every tile's height from it.
    /// The grid is `(4·xs + 1) × (4·ys + 1)` millionths, row-major.
    pub fn set_corner_grid(&mut self, h: Vec<i64>) -> bool {
        let (w, ht) = (self.corner_stride(), self.height * TILES_PER_CELL + 1);
        if h.len() != (w * ht) as usize {
            return false;
        }
        self.corner_z = h;
        // A grid installed whole is the original's own, whatever was
        // written over the one before it.
        self.corner_rewritten.clear();
        let (tw, th) = (self.width * TILES_PER_CELL, self.height * TILES_PER_CELL);
        for ty in 0..th {
            for tx in 0..tw {
                self.retile_z(Pos::new(tx, ty));
            }
        }
        true
    }

    /// `TerrainOut::find_tcoord_z@008544a0` for one tile, from the corner
    /// grid: the truncated mean of the two **anti-diagonal** corners, and 0
    /// where the tile's surface bits say ocean.
    pub fn retile_z(&mut self, t: Pos) {
        let (Some(a), Some(b)) = (self.corner_z(t.x, t.y + 1), self.corner_z(t.x + 1, t.y)) else {
            return;
        };
        let ocean = self.tile_mask(t) & tile::SURFACE == tile::SURFACE_OCEAN;
        let z = if ocean {
            0
        } else {
            ((a + b) / 2_000_000) as i32
        };
        self.set_tile_z(t, z);
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

    /// Adds a good, and — unless it is oil — makes it its cell's chain
    /// terminator, exactly as `Objects::init_good@00653f30` does.
    pub fn add_good(&mut self, g: Good) -> usize {
        let i = self.goods.len();
        self.goods.push(g);
        if g.ty != OIL
            && let Some(c) = self.index(g.pos.cell())
        {
            self.cell_good[c] = i as i32;
        }
        i
    }

    /// The goods list, in the original's own order.
    pub fn goods(&self) -> &[Good] {
        &self.goods
    }

    /// The good this cell's object chain ends at, if any.
    pub fn good_at(&self, c: Cell) -> Option<(usize, Good)> {
        let i = self.index(c)?;
        let g = self.cell_good[i];
        (g >= 0).then(|| (g as usize, self.goods[g as usize]))
    }

    /// Writes a cell's record (the map loader's, and a test's).
    pub fn set_cell_data(&mut self, c: Cell, d: CellData) {
        if let Some(i) = self.index(c) {
            self.cells[i] = d;
        }
    }

    /// `WorldData::buildings_allowed@006b2340`: the cell's `flags & 0x78`
    /// — `ROCK`, `MOUNTAIN`, `FOREST` and the unnamed `0x40` — must all be
    /// clear. It is a **predicate, not a field**: the function returns 1
    /// when the four bits are zero and 0 otherwise, so rough ground takes
    /// no building. Off the map a zero record answers true, which is what
    /// the bounds test upstream is for.
    pub fn buildings_allowed(&self, c: Cell) -> bool {
        self.cell_data(c).flags & 0x78 == 0
    }

    /// `WorldData::is_ocean@006b4830`: not a `HALFLAND` cell, and its
    /// `land` is 1 or 2 — the kinds the census calls water. Off the map a
    /// zero record answers false.
    pub fn is_ocean(&self, c: Cell) -> bool {
        let d = self.cell_data(c);
        d.flags & cell::HALFLAND == 0 && (d.land == 1 || d.land == 2)
    }

    /// `WorldData::get_land@006b4730` with its third argument 1 — the
    /// land class of a cell, the index [`LANDS`] is read at.
    ///
    /// The stored `WData.land` (`land_key[]`: `BASELAND`, `SANDY`,
    /// `OCEAN`, `NONE`) is only the fall-through; five flag tests, in this
    /// order, override it with the classes the file calls Coast, Forest,
    /// Mountains, Rocks and Oil. `Leader::plan_strategy`'s muster search
    /// (`docs/ARMY.md` §13) computes exactly this inline, which is what
    /// says the two are one function.
    pub fn land_class(&self, c: Cell) -> i32 {
        let d = self.cell_data(c);
        let f = d.flags;
        if f & cell::COAST != 0 {
            3
        } else if f & cell::FOREST != 0 {
            4
        } else if f & (cell::MOUNTAIN | 0x40) != 0 {
            5
        } else if f & cell::ROCK != 0 {
            // `is_rocks` then `is_oil_at`: 6, or 7 where the oil is.
            6 + i32::from(f & cell::OIL != 0)
        } else if self.is_ocean(c) && f & cell::OIL != 0 {
            7
        } else {
            i32::from(d.land)
        }
    }

    /// `WorldData::get_land@006b4c70` with its third argument 1 — the
    /// **tile** form of [`World::land_class`], which
    /// `BuildTypeData::blocked_tcoord`'s flat-gather arm reads (`docs/CITIES.md`
    /// §2.5). It is not the cell form: a coast cell answers Ocean (2) under an
    /// ocean tile and plain Land (0) under any other, a mountain is the
    /// tile's own object rather than the cell's flag, and oil is tested
    /// before rock, so a cell carrying both reads Oil (7).
    pub fn land_class_tile(&self, t: Pos) -> i32 {
        let d = self.cell_data(Self::cell_of_tile(t));
        let f = d.flags;
        let mask = self.tile_mask(t);
        if f & cell::COAST != 0 {
            if mask & tile::SURFACE == tile::SURFACE_OCEAN {
                2
            } else {
                0
            }
        } else if f & cell::FOREST != 0 {
            4
        } else if mask & tile::OBJECT == tile::OBJECT_MOUNTAIN {
            5
        } else if f & cell::OIL != 0 {
            7
        } else if f & cell::ROCK != 0 {
            6
        } else {
            i32::from(d.land)
        }
    }

    /// `World::gather_at@006b07f0` — what a citizen would take off this
    /// cell, per good, and the only writer of `CityData::ter`.
    ///
    /// `centre_only` is the original's fifth argument, and the two callers
    /// disagree on it. `Leader::plan_strategy`'s step 13 passes **1**: read
    /// the cell's own land class, give each flat good its face amount, and
    /// give a non-flat good twice its amount unless the tile is already
    /// gathered from. `Leader::produce_building`'s gather score passes
    /// **0**: the flat goods still come from the centre alone, but a
    /// non-flat good is summed over the cell and its eight neighbours —
    /// each skipped if off the map or already gathered from — and the
    /// total doubled at the end.
    ///
    /// The neighbour pass does not re-test `amount != 0`: a `"none"` slot
    /// carries good `-1`, which matches no good, so the two gates come to
    /// the same thing.
    pub fn gather_at(&self, c: Cell, centre_only: bool) -> [i32; crate::economy::RESOURCES] {
        let mut out = [0; crate::economy::RESOURCES];
        let taken = |w: &World, n: Cell| w.tile_mask(n.centre_tile()) & tile::GATHERED_FROM != 0;
        if let Some(land) = usize::try_from(self.land_class(c))
            .ok()
            .and_then(|i| LANDS.get(i))
        {
            for k in 0..MAKES_PER_LAND {
                let amount = land.amount[k];
                let Ok(g) = usize::try_from(land.good[k]) else {
                    continue;
                };
                if amount == 0 || g >= crate::economy::RESOURCES {
                    continue;
                }
                if good_is_flat(g) {
                    out[g] += amount;
                } else if centre_only && !taken(self, c) {
                    out[g] += amount * 2;
                }
            }
        }
        if centre_only {
            return out;
        }
        for (dx, dy) in std::iter::once((0, 0)).chain(MOVE_8) {
            let n = Cell::new(c.x + dx, c.y + dy);
            if !self.contains(n) || taken(self, n) {
                continue;
            }
            let Some(land) = usize::try_from(self.land_class(n))
                .ok()
                .and_then(|i| LANDS.get(i))
            else {
                continue;
            };
            for (g, o) in out.iter_mut().enumerate() {
                if good_is_flat(g) {
                    continue;
                }
                for k in 0..MAKES_PER_LAND {
                    if land.good[k] == g as i32 {
                        *o += land.amount[k];
                    }
                }
            }
        }
        for (g, o) in out.iter_mut().enumerate() {
            if !good_is_flat(g) {
                *o *= 2;
            }
        }
        out
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

    /// `reg_xs` — `World::init@006b76f0:49`: `(xs · 4) >> 3`, the tile
    /// width halved. Not `xs / 2`: an odd cell width rounds *down* here
    /// and up there.
    pub const fn reg_xs(&self) -> i32 {
        (self.width * 4) >> 3
    }

    /// `reg_ys`, the same from the height.
    pub const fn reg_ys(&self) -> i32 {
        (self.height * 4) >> 3
    }

    /// `reg_size` — the length of one leader's danger row.
    pub const fn reg_size(&self) -> i32 {
        self.reg_xs() * self.reg_ys()
    }

    /// `danger[who][reg_xs × (cy / 2) + (cx / 2)]` — the read every
    /// consumer in the original makes (see the field's own note). 0 when
    /// never written.
    pub fn danger_half(&self, who: Player, c: Cell) -> i32 {
        if c.x < 0 || c.y < 0 || c.x / 2 >= self.reg_xs() || c.y / 2 >= self.reg_ys() {
            return 0;
        }
        self.danger_at_index((c.y / 2) * self.reg_xs() + (c.x / 2), who)
    }

    /// The same read at a **position**, which is how every caller in the
    /// original spells it: `div3(y >> 9)` and `div3(x >> 9)`.
    pub fn danger_at(&self, who: Player, p: Pos) -> i32 {
        self.danger_half(who, p.cell())
    }

    fn danger_at_index(&self, i: i32, who: Player) -> i32 {
        self.danger
            .get(who as usize)
            .and_then(|d| usize::try_from(i).ok().and_then(|i| d.get(i)))
            .copied()
            .unwrap_or(0)
    }

    /// `GameDaemon::calc_danger`'s opening `memset`: one leader's row, back
    /// to zero and sized to the map.
    pub fn clear_danger(&mut self, who: Player) {
        let n = usize::try_from(self.reg_size()).unwrap_or(0);
        let w = who as usize;
        if self.danger.len() <= w {
            self.danger.resize_with(w + 1, Vec::new);
        }
        self.danger[w].clear();
        self.danger[w].resize(n, 0);
    }

    /// `GameDaemon::do_danger`'s one write: `danger[who][index] += delta`,
    /// with the index already bounds-checked by the caller.
    pub fn add_danger(&mut self, who: Player, index: i32, delta: i32) {
        let w = who as usize;
        let Ok(i) = usize::try_from(index) else {
            return;
        };
        if self.danger.len() <= w {
            self.danger.resize_with(w + 1, Vec::new);
        }
        let n = usize::try_from(self.reg_size()).unwrap_or(0);
        if self.danger[w].len() < n {
            self.danger[w].resize(n, 0);
        }
        if let Some(v) = self.danger[w].get_mut(i) {
            *v += delta;
        }
    }

    /// One leader's whole row, for the diff.
    pub fn danger_row(&self, who: Player) -> &[i32] {
        self.danger.get(who as usize).map_or(&[], Vec::as_slice)
    }

    /// `Region.flags` — 0 for a region nothing has installed one for.
    pub fn region_flags(&self, region: u16) -> i32 {
        self.region_flags
            .get(region as usize)
            .copied()
            .unwrap_or_default()
    }

    /// Installs `Region.flags` for one region, from a dump.
    pub fn set_region_flags(&mut self, region: u16, flags: i32) {
        let r = region as usize;
        if self.region_flags.len() <= r {
            self.region_flags.resize(r + 1, 0);
        }
        self.region_flags[r] = flags;
    }

    /// `Region.flags |= bits` — `Build::close@00628980`'s `or $0x20` on
    /// the closing building's own region (`docs/ECONOMY.md` §17).
    pub fn or_region_flags(&mut self, region: u16, bits: i32) {
        let f = self.region_flags(region);
        self.set_region_flags(region, f | bits);
    }

    /// `GameDaemon::process_all@00732700`'s region pass, every frame: each
    /// region's [`REGION_REFIND_GATHER`] is cleared, and a region carrying
    /// [`REGION_VERIFY_GATHER`] trades it for `REGION_REFIND_GATHER` — so a
    /// camp closed in the AI's turn has its region re-walked by every
    /// other camp there in this frame's `Objects::process_all`, and one
    /// closed inside that pass is verified by the buildings after it and
    /// re-walked on the next frame (`docs/ECONOMY.md` §17.2).
    pub fn cycle_gather_flags(&mut self) {
        for f in &mut self.region_flags {
            *f &= !REGION_REFIND_GATHER;
            if *f & REGION_VERIFY_GATHER != 0 {
                *f &= !REGION_VERIFY_GATHER;
                *f |= REGION_REFIND_GATHER;
            }
        }
    }

    /// `Region.scouted`'s bit for one leader (`docs/TRANSPORT.md` §7).
    pub fn region_scouted(&self, region: u16, who: Player) -> bool {
        self.scouted
            .get(region as usize)
            .is_some_and(|b| b & (1 << (who & 7)) != 0)
    }

    /// `Unit::think_scout@005f6010:566` — the tail's mark, set when a
    /// scout's whole search has come back empty.
    pub fn mark_region_scouted(&mut self, region: u16, who: Player) {
        let r = region as usize;
        if self.scouted.len() <= r {
            self.scouted.resize(r + 1, 0);
        }
        self.scouted[r] |= 1 << (who & 7);
    }

    /// `WorldData::num_waterhalf(cx, cy)@006b4db0`: 0 for a cell that is
    /// not `HALFLAND`, else how many of its sixteen tiles are ocean.
    ///
    /// `think_civilian_transport` wants **zero** of them, which is the
    /// land side of a shore cell (`docs/TRANSPORT.md` §7).
    pub fn num_waterhalf(&self, c: Cell) -> i32 {
        if self.cell_data(c).flags & cell::HALFLAND == 0 {
            return 0;
        }
        let mut n = 0;
        for i in 0..16 {
            let t = Pos::new(
                c.x * TILES_PER_CELL + (i & 3),
                c.y * TILES_PER_CELL + (i >> 2),
            );
            if self.tile_mask(t) & tile::SURFACE == tile::SURFACE_OCEAN {
                n += 1;
            }
        }
        n
    }

    /// `Region::coast_here(r, s, cx, cy)@00681020` (`docs/TRANSPORT.md`
    /// §9.3): the cell sits in `r` or in `s`, the two coast each other, and
    /// one of its eight neighbouring cells — read at its **centre tile**,
    /// `× 4 + 2`, through `get_tregion` — lies in the other. The answer is
    /// that neighbour's index in [`MOVE_8`], `1..=8`, or 0.
    ///
    /// **The neighbour read is [`World::tregion_alt`], not
    /// [`World::tregion`]** (2026-09-01): the original's call at `0068106a`
    /// is `WorldData::get_tregion`, which answers a coastal cell's
    /// `region2` — the *sea* region — when the tile it is handed lies on
    /// ocean, and a neighbour's centre tile is exactly the tile that can.
    /// With the plain cell region the whole function is nearly dead: a
    /// land cell whose neighbours are all land cells never coasts anything,
    /// so only a cell touching a wholly-ocean cell ever answered, and
    /// `think_civilian_transport` (§7) could not see the shore it was
    /// looking for. This is `docs/SYNC.md` §3.24's lesson a second time,
    /// in a second caller.
    pub fn coast_here(&self, r: u16, s: u16, c: Cell) -> i32 {
        if !self.is_coast(r, s) {
            return 0;
        }
        let here = self.region_of(c);
        let want = if here == Some(r) {
            s
        } else if here == Some(s) {
            r
        } else {
            return 0;
        };
        for (i, (dx, dy)) in MOVE_8.iter().enumerate() {
            let (nx, ny) = (c.x + dx, c.y + dy);
            if nx < 0 || ny < 0 || nx >= self.width || ny >= self.height {
                continue;
            }
            let centre = Pos::new(nx * TILES_PER_CELL + 2, ny * TILES_PER_CELL + 2);
            if self.tregion_alt(centre) == Some(want) {
                return i as i32 + 1;
            }
        }
        0
    }

    /// Writes a danger figure at a cell, for a test that wants one without
    /// building the objects that would earn it.
    pub fn set_danger_at(&mut self, who: Player, c: Cell, value: i32) {
        if c.x < 0 || c.y < 0 {
            return;
        }
        let i = (c.y / 2) * self.reg_xs() + (c.x / 2);
        let cur = self.danger_at_index(i, who);
        self.add_danger(who, i, value - cur);
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
    /// not −1, and `World::analyze_map@006b58b0` finishes it: a value
    /// outside `0..4` is computed from the regions — starts on more than
    /// one landmass → 3, or 4 when the free regions of at least
    /// `size × 60 / dim²` cells hold twice the players' land; else 2 with
    /// any such free region; else 1 when `land_size < 4·size/5`, else 0 —
    /// and a file's 3 is promoted to 4 by the same test (East Indies on
    /// run20: 1097 ≥ 2 × 529). The AI reads it as "how much sea": `> 2`
    /// is the coastal-ring and dock-value predicate, `< 4` the expansion
    /// gate. It was read as "the number of land regions" until run20
    /// (2026-08-25) reported 4 on a map with twelve of them; 0 on a world
    /// with no map loaded (`docs/AI.md` §15.8).
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

    /// Every cell's `(who, who2)`, row-major — what a border fix's pass
    /// copies onto the map ([`crate::border_pass`]).
    pub fn owners(&self) -> Vec<(Owner, Owner)> {
        self.who
            .iter()
            .copied()
            .zip(self.who2.iter().copied())
            .collect()
    }

    /// Writes every cell's `(who, who2)` from [`World::owners`]' shape.
    pub fn set_owners(&mut self, all: &[(Owner, Owner)]) {
        for (i, &(a, b)) in all.iter().enumerate().take(self.who.len()) {
            self.who[i] = a;
            self.who2[i] = b;
        }
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

    /// `WorldData::has_blocked_neighbors@006b2990` — whether any of the
    /// eight neighbouring tiles carries [`tile::BLOCKED`]. Off-map
    /// neighbours count as clear.
    pub fn has_blocked_neighbors(&self, t: Pos) -> bool {
        MOVE_8
            .iter()
            .any(|(dx, dy)| self.tile_mask(Pos::new(t.x + dx, t.y + dy)) & tile::BLOCKED != 0)
    }

    /// `World::set_bad_path@006b4610`: sets or clears one tile's
    /// [`tile::BAD_PATH`], keeping the containing cell's [`CellData::bad`]
    /// count.
    pub fn set_bad_path(&mut self, t: Pos, on: bool) {
        self.set_bad_path_bit(t, on);
    }

    /// Sets or clears one tile's [`tile::BAD_PATH`], keeping the containing
    /// cell's [`CellData::bad`] count.
    fn set_bad_path_bit(&mut self, t: Pos, on: bool) {
        let Some(i) = self.tile_index(t) else { return };
        let had = self.tiles[i] & tile::BAD_PATH != 0;
        if had == on {
            return;
        }
        let c = Self::cell_of_tile(t);
        let mut d = self.cell_data(c);
        d.bad = if on {
            d.bad.saturating_add(1)
        } else {
            d.bad.saturating_sub(1)
        };
        self.set_cell_data(c, d);
        if on {
            self.tiles[i] |= tile::BAD_PATH;
        } else {
            self.tiles[i] &= !tile::BAD_PATH;
        }
    }

    /// **`World::set_blocked_at@006b4900` — the writer of the pathfinder's
    /// terrain cost.**
    ///
    /// `WData.blocked` is not a property of the ground: it is a running
    /// count of the cell's blocked *tiles*, and this is the only function
    /// that moves it. Every caller in the original goes through here —
    /// `BuildType::mask_me`, the mountains, the cliffs, a `Good`'s own
    /// footprint, a packed siege engine — so a tile that becomes blocked
    /// without it leaves `docs/PATHFINDER.md` §5's `+ 20 × tcost` and
    /// §5.1's corner-cutting gate reading a stale zero. That is exactly
    /// what item 125 was: a city went up on East Indies and the four cells
    /// under it stayed free to walk (`docs/PATHFINDER.md` §12).
    ///
    /// It keeps three things at once, each guarded so that setting an
    /// already-set bit costs nothing — which is what makes it safe to run
    /// over a footprint the map dump has already blocked:
    ///
    /// - the cell's `blocked` **and** `solid` counts, which move together
    ///   here (a building stops a forest-walker as surely as anyone; the
    ///   forest's own `blocked > 0, solid == 0` is written elsewhere);
    /// - the tile's own [`tile::BAD_PATH`], cleared when it becomes
    ///   blocked and restored on unblocking if it still has a blocked
    ///   neighbour;
    /// - that bit on all eight neighbours — set when this tile blocks,
    ///   cleared when it unblocks and the neighbour has no other blocked
    ///   neighbour left — with the cell's `bad` count beside it.
    ///
    /// **SEAM**: the original also clears the tile's road
    /// (`set_road_at(t, 0, 0, 0)`) when a tile becomes blocked. Only the
    /// laying half of `set_road_at` is modelled (`crate::roads`), so the
    /// clearing half is left out here rather than guessed at.
    pub fn set_blocked_at(&mut self, t: Pos, on: bool) {
        let Some(i) = self.tile_index(t) else { return };
        let had = self.tiles[i] & tile::BLOCKED != 0;
        if had != on {
            let c = Self::cell_of_tile(t);
            let mut d = self.cell_data(c);
            if on {
                d.blocked = d.blocked.saturating_add(1);
                d.solid = d.solid.saturating_add(1);
            } else {
                d.blocked = d.blocked.saturating_sub(1);
                d.solid = d.solid.saturating_sub(1);
            }
            self.set_cell_data(c, d);
        }
        if on {
            self.tiles[i] |= tile::BLOCKED;
            self.set_bad_path_bit(t, false);
        } else {
            self.tiles[i] &= !tile::BLOCKED;
        }
        for (dx, dy) in MOVE_8 {
            let n = Pos::new(t.x + dx, t.y + dy);
            if !self.tile_in_bounds(n) {
                continue;
            }
            if on {
                self.set_bad_path_bit(n, true);
            } else if !self.has_blocked_neighbors(n) {
                self.set_bad_path_bit(n, false);
            }
        }
        // Unblocking restores the tile's own halo bit when something else
        // beside it is still blocked.
        if !on && self.has_blocked_neighbors(t) {
            self.set_bad_path_bit(t, true);
        }
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

    /// `Region.coords[start], [start + stride], …` — the region's cell list
    /// in the order `Regions::rebuild_coords@0067f800` writes it.
    ///
    /// That function is the **only** writer that survives map load:
    /// `Regions::find_all` appends coordinates in its own flood order,
    /// merges regions, sorts them, then frees the list and calls this one
    /// last. And this one is a plain **row-major sweep of the cell grid** —
    /// `for y in 0..ys { for x in 0..xs { coords[wdata[xs*y + x].region]
    /// .push((x, y)) } }` — so the order needs no dump to recover: it is
    /// the grid's own (`docs/SCOUT.md` §11).
    ///
    /// Returns only the entries the caller will visit, since the region
    /// scan strides.
    pub fn region_coords_strided(&self, region: u16, start: i32, stride: i32) -> Vec<Cell> {
        debug_assert!(stride >= 1, "the region scan's stride is at least 1");
        let mut out = Vec::new();
        if start < 0 {
            return out;
        }
        let mut seen = 0i32;
        let mut want = start;
        for (i, r) in self.region.iter().enumerate() {
            if *r != Some(region) {
                continue;
            }
            if seen == want {
                let i = i as i32;
                out.push(Cell::new(i % self.width, i / self.width));
                want += stride;
            }
            seen += 1;
        }
        out
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

    /// **`coast_here`'s neighbour read is `get_tregion`, not the cell's
    /// plain region** (`0068106a`, 2026-09-01, `docs/TRANSPORT.md` §9.3).
    /// A coastal cell — `flags & 0x100`, `region2` the sea — is a *land*
    /// cell in `WData.region`, and it is exactly the cell a boat's half of
    /// the shore lies in. Asked with the plain region the function cannot
    /// see it, and `think_civilian_transport` then never picks the shore
    /// cell the original picks (`docs/SYNC.md` §3.25).
    #[test]
    fn coast_here_sees_a_shore_whose_water_half_is_a_land_cell() {
        let mut w = World::new(8, 8);
        let land = w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(6, 7));
        let sea = w.fill_region(Terrain::Sea, Cell::new(7, 0), Cell::new(7, 7));
        w.rebuild_coasts();
        assert!(w.is_coast(land, sea), "the two touch at x = 6/7");

        // (3, 3) is inland: every neighbour is a land cell by `WData.region`.
        assert_eq!(w.coast_here(land, sea, Cell::new(3, 3)), 0);

        // Make its eastern neighbour the half-water shore — still a land
        // cell in `region`, but `HALFLAND` with `region2` the sea and an
        // ocean tile at its centre, which is the tile `get_tregion` reads.
        let shore = Cell::new(4, 3);
        let mut d = w.cell_data(shore);
        d.flags |= 0x100;
        d.region2 = Some(sea);
        w.set_cell_data(shore, d);
        w.set_tile_field(shore.centre_tile(), tile::SURFACE, tile::SURFACE_OCEAN);
        assert_eq!(w.region_of(shore), Some(land), "still a land cell");
        assert_eq!(w.tregion(shore.centre_tile()), Some(land), "the plain read");
        assert_eq!(w.tregion_alt(shore.centre_tile()), Some(sea), "get_tregion");

        // MOVE_8[3] is (1, 0), so the answer is that neighbour's 1-based
        // index — and with the plain read it would still be 0.
        assert_eq!(w.coast_here(land, sea, Cell::new(3, 3)), 4);
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

    /// `Region.coords` is `Regions::rebuild_coords@0067f800`'s order and
    /// that is the grid's own: y outer, x inner. A column-major list would
    /// hold the same eight cells and hand a strided walk a different four,
    /// which is what `docs/SCOUT.md` §11 rides on.
    #[test]
    fn a_region_s_coords_are_row_major() {
        let mut w = World::new(4, 4);
        let land = w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(1, 3));
        w.fill_region(Terrain::Sea, Cell::new(2, 0), Cell::new(3, 3));
        assert_eq!(
            w.region_coords_strided(land, 0, 1),
            vec![
                Cell::new(0, 0),
                Cell::new(1, 0),
                Cell::new(0, 1),
                Cell::new(1, 1),
                Cell::new(0, 2),
                Cell::new(1, 2),
                Cell::new(0, 3),
                Cell::new(1, 3),
            ]
        );
        // And the stride is over that list, not over the grid.
        assert_eq!(
            w.region_coords_strided(land, 1, 3),
            vec![Cell::new(1, 0), Cell::new(0, 2), Cell::new(1, 3)]
        );
    }

    /// A world of plain land with one forest cell, one mountain and one
    /// offshore oil patch — enough to exercise every arm of
    /// [`World::land_class`].
    fn lands_world() -> World {
        let mut w = World::new(6, 6);
        let land = w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(5, 5));
        assert_eq!(w.terrain(land), Terrain::Land);
        let set = |w: &mut World, c: Cell, flags: u16, l: i8| {
            let mut d = w.cell_data(c);
            d.flags = flags;
            d.land = l;
            w.set_cell_data(c, d);
        };
        // The whole grid is BASELAND, land class 0.
        for y in 0..6 {
            for x in 0..6 {
                set(&mut w, Cell::new(x, y), 0, 0);
            }
        }
        set(&mut w, Cell::new(2, 2), cell::FOREST, 0);
        set(&mut w, Cell::new(3, 2), cell::MOUNTAIN, 0);
        set(&mut w, Cell::new(4, 2), cell::ROCK | cell::OIL, 0);
        // An ocean cell carrying oil: not rock, so the `is_ocean` arm.
        set(&mut w, Cell::new(0, 5), cell::OIL, 2);
        w
    }

    /// `WorldData::get_land@006b4730`'s five flag tests, in order.
    #[test]
    fn a_cell_s_land_class_is_its_flags_before_its_stored_kind() {
        let w = lands_world();
        assert_eq!(w.land_class(Cell::new(1, 1)), 0, "BASELAND");
        assert_eq!(w.land_class(Cell::new(2, 2)), 4, "Forest");
        assert_eq!(w.land_class(Cell::new(3, 2)), 5, "Mountains");
        assert_eq!(w.land_class(Cell::new(4, 2)), 7, "Rocks with oil");
        assert_eq!(w.land_class(Cell::new(0, 5)), 7, "ocean with oil");
    }

    /// Step 13's caller — `centre_only`. A flat good comes off the cell at
    /// its face amount; a non-flat one is doubled, and only if nothing
    /// already gathers the cell's centre tile.
    #[test]
    fn the_centre_only_gather_doubles_the_goods_the_ground_holds() {
        use crate::economy::Resource::{Food, Knowledge, Metal, Oil, Timber};
        let mut w = lands_world();
        let mut want = [0; crate::economy::RESOURCES];
        want[Knowledge as usize] = 1;
        want[Food as usize] = 1;
        assert_eq!(w.gather_at(Cell::new(1, 1), true), want, "plain land");

        let mut want = [0; crate::economy::RESOURCES];
        want[Timber as usize] = 2;
        assert_eq!(w.gather_at(Cell::new(2, 2), true), want, "forest");
        let mut want = [0; crate::economy::RESOURCES];
        want[Metal as usize] = 2;
        assert_eq!(w.gather_at(Cell::new(3, 2), true), want, "mountains");
        let mut want = [0; crate::economy::RESOURCES];
        want[Oil as usize] = 2;
        assert_eq!(w.gather_at(Cell::new(4, 2), true), want, "an oil cell");

        // A tile some building already gathers is worth nothing — and only
        // to the non-flat half, which is the whole of a forest cell's
        // value and none of plain land's.
        w.set_tile_bits(Cell::new(2, 2).centre_tile(), tile::GATHERED_FROM);
        w.set_tile_bits(Cell::new(1, 1).centre_tile(), tile::GATHERED_FROM);
        assert_eq!(w.gather_at(Cell::new(2, 2), true), [0; 6]);
        let mut want = [0; crate::economy::RESOURCES];
        want[Knowledge as usize] = 1;
        want[Food as usize] = 1;
        assert_eq!(w.gather_at(Cell::new(1, 1), true), want);
    }

    /// `produce_building`'s caller — the neighbourhood sum. The flat goods
    /// still come from the centre alone; a non-flat good is summed over
    /// the nine cells and doubled once at the end.
    #[test]
    fn the_full_gather_sums_the_nine_cells_and_doubles_once() {
        use crate::economy::Resource::{Food, Knowledge, Metal, Timber};
        let mut w = lands_world();
        // (3, 2) is mountains; its ring holds the forest at (2, 2) and the
        // oiled rock at (4, 2), neither of which makes metal.
        let got = w.gather_at(Cell::new(3, 2), false);
        assert_eq!(got[Timber as usize], 2, "one forest neighbour, doubled");
        assert_eq!(got[Metal as usize], 2, "the centre's own mountain");
        assert_eq!(got[Knowledge as usize], 0, "mountains make no knowledge");

        // Standing on plain land beside the forest: the flat pair comes
        // from the centre, the timber from the neighbour.
        let got = w.gather_at(Cell::new(1, 1), false);
        assert_eq!(got[Knowledge as usize], 1);
        assert_eq!(got[Food as usize], 1);
        assert_eq!(got[Timber as usize], 2);

        // And a gathered-from neighbour drops out of the sum.
        w.set_tile_bits(Cell::new(2, 2).centre_tile(), tile::GATHERED_FROM);
        assert_eq!(w.gather_at(Cell::new(1, 1), false)[Timber as usize], 0);
    }

    /// The nine are the centre and [`MOVE_8`], so a cell two away never
    /// counts and the map's edge is not walked off.
    #[test]
    fn the_full_gather_reaches_one_cell_and_stays_on_the_map() {
        use crate::economy::Resource::Timber;
        let w = lands_world();
        assert_eq!(w.gather_at(Cell::new(0, 0), false)[Timber as usize], 0);
        assert_eq!(w.gather_at(Cell::new(1, 1), false)[Timber as usize], 2);
        // (5, 5) is the far corner: three of its nine are off the map.
        assert_eq!(w.gather_at(Cell::new(5, 5), false)[Timber as usize], 0);
    }
}
