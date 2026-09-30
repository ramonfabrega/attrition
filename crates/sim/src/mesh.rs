//! The road **mesh** — `Roads::add_roads@0088f4b0` and the road tiles
//! `Roads::set_diags@0088e9d0` lays of its own.
//!
//! `docs/ROADS.md` §9. `crate::roads` is the *plan* — the ring a building
//! draws and the search back to its city; this is what the world does with
//! each tile as it is laid. Every `World::set_road_at@006b43b0` on a tile
//! that was not already a road queues a `RoadModification` and runs
//! `add_roads` over the queue, and `set_diags` — the pass that decides how a
//! road element joins its neighbours — **lays road** where a road stands
//! diagonally from a new one and neither tile between them is anything at
//! all. Those tiles are ordinary road as far as everything downstream is
//! concerned: `PathFinder::calc_road_cost` prices them at a road's `>>3`
//! rather than a plain tile's `×4`.
//!
//! The state is one `RoadElementCandidate` per road tile — the PDB's own
//! name, `RoadsPieces::candidates`, sixteen to a terrain patch. Only three of
//! its fields reach the simulation: `flags`, whose top byte is the eight
//! directions the tile is joined in; `support_codes`, whose `0x2000` is the
//! note that says "I made the road beside me"; and the pair of reference
//! counts that decide when an element — and with it a mesh-laid tile — goes
//! away.
//!
//! **What is modelled and what is not** is §9.4 of the document. The short
//! version: the passes that write `flags`' connection bits and the passes
//! that write the world are here; the ten that exist to choose a texture and
//! a rotation are not.

use crate::Sim;
use crate::ai_place::{MOVE_X, MOVE_Y};
use crate::world::{Cell, Pos, World, cell, tile};
use std::collections::BTreeMap;

/// `corner_x` / `corner_y`, the `.rdata` pair `set_diags` indexes 1..=4 —
/// the same table `docs/AI.md` §2.13 names, `(0,0), (−1,−1), (1,−1), (1,1),
/// (−1,1)`. Slot 0 is dead here.
const CORNER_X: [i32; 5] = [0, -1, 1, 1, -1];
const CORNER_Y: [i32; 5] = [0, -1, -1, 1, 1];

/// The eight connection bits of `RoadElementCandidate::flags`. The four
/// cardinals are what `RoadsOut::get_orthog_connects@00893560` counts; the
/// four diagonals are `set_diags`' own.
pub mod dir {
    pub const N: u32 = 0x4000_0000;
    pub const E: u32 = 0x1000_0000;
    pub const S: u32 = 0x0400_0000;
    pub const W: u32 = 0x0100_0000;
    pub const NW: u32 = 0x8000_0000;
    pub const NE: u32 = 0x2000_0000;
    pub const SE: u32 = 0x0800_0000;
    pub const SW: u32 = 0x0200_0000;
    /// What `get_orthog_connects` counts.
    pub const ORTHOG: u32 = N | E | S | W;
}

/// `RoadElementCandidate::support_codes`' one bit that reaches the world: the
/// note `set_diags` leaves on the tile whose corner it filled, so that
/// `Roads::clear_support@0088e3f0` can take the filled tile away with it.
/// [`SUPPORT_EAST`] beside it says the fill went east rather than west.
const SUPPORT_MADE: u16 = 0x2000;
const SUPPORT_EAST: u16 = 0x0080;

/// `set_diags`' seven parallel five-entry tables, laid end to end on its
/// stack and indexed `[i]`, `[i + 5]`, `[i + 10]` … from one base. Slot 0 of
/// each is dead; the loop runs `i` from 1 to 4, one corner each.
mod tables {
    use super::dir::*;
    /// `[i]` — corner `i`'s compass index, into `move_x`/`move_y`.
    pub const DIAG: [usize; 5] = [0, 1, 3, 5, 7];
    /// `[i + 5]` — the bit `set_neighbor` puts on the **corner**.
    pub const OPPOSITE: [u32; 5] = [0, SE, SW, NW, NE];
    /// `[i + 10]` — the bit that goes on the tile itself.
    pub const OWN: [u32; 5] = [0, NW, NE, SE, SW];
    /// `[i + 15]`, `[i + 20]` — the two diagonals whose presence forbids this
    /// one. They are the corner's two neighbours round the compass.
    pub const BLOCK_A: [u32; 5] = [0, NE, NW, SW, SE];
    pub const BLOCK_B: [u32; 5] = [0, SW, SE, NE, NW];
    /// `[i + 25]`, `[i + 30]` — the two cardinals flanking the corner, as
    /// compass indices. Both must be free of road **and** of footprint for
    /// the fill to run.
    pub const FLANK_A: [usize; 5] = [0, 2, 4, 6, 8];
    pub const FLANK_B: [usize; 5] = [0, 8, 2, 4, 6];
}

/// The four cardinals: compass index, this tile's bit, the neighbour's
/// answering bit, and the step. `mark_and_trim_directions` walks them in this
/// order — north, south, east, west.
const CARDINALS: [(usize, u32, u32, i32, i32); 4] = [
    (2, dir::N, dir::S, 0, -1),
    (6, dir::S, dir::N, 0, 1),
    (4, dir::E, dir::W, 1, 0),
    (8, dir::W, dir::E, -1, 0),
];

/// One `RoadElementCandidate`, as far as the simulation reads it.
///
/// The two counts are the original's `ref_count` (`+0xa`) and
/// `pending_camel_steps` (`+0xb`) — the PDB's names, whatever the second once
/// meant. `TerrainOut::road_changed@00866a70` raises one or the other and
/// lowers `pending_camel_steps` first; the element goes when both are zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Elem {
    /// `+0x4`. Only the top byte — [`dir`] — is read here.
    pub flags: u32,
    /// `+0xc`, of which [`SUPPORT_MADE`] and [`SUPPORT_EAST`] are read.
    pub support: u16,
    /// `+0xa` `ref_count`.
    pub refs: u8,
    /// `+0xb` `pending_camel_steps`.
    pub refs2: u8,
    /// `+0xf` `is_terrain_creation` — this road tile is the mesh's own, and
    /// may be taken away again.
    pub made: bool,
}

/// One entry of `RoadsData::added_roads` — a `RoadModification`: a packed
/// tile index and two bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Modification {
    index: i32,
    added: u8,
    valid: u8,
}

/// `Roads`' own state: the element store, the modification queue, and the two
/// lists the redo pass ping-pongs between.
///
/// The caches are `RoadsData::neighbor_cache` and `road_cache`, nine entries
/// each — the compass, with `[0]` the tile itself — refilled by
/// `RoadsOut::fill_cache@00893b30` before every pass that reads them.
#[derive(Clone, Debug, Default)]
pub struct RoadMesh {
    /// Keyed by the original's own packed index, `x + width_in_tiles · y`.
    /// Ordered, because a simulation may not carry a hash map
    /// (`crate::no_float` is not the only rule with teeth): nothing here
    /// iterates it, but the crate's own lint would rather it could not.
    elems: BTreeMap<i32, Elem>,
    /// `RoadsData::added_roads`.
    mods: Vec<Modification>,
    /// `Roads::add_roads`' function-static `remove_list`.
    remove: Vec<Modification>,
    /// `changed_roads` (`+0x150`) and `changed_roads2` (`+0x16c`).
    redo: [Vec<i32>; 2],
    /// `RoadsData::tx` / `ty`.
    at: Pos,
    /// `neighbor_cache` — a building footprint that is not a road.
    blocked: [bool; 9],
    /// `road_cache` — a road.
    road: [bool; 9],
    /// `road_cache2` (`RoadsData +0x5c8`) — the stray-road sweep's own
    /// compass of roads. **It persists**: `scan_and_kill_stray_roads`
    /// writes nothing to an off-map neighbour's entry (its off-map arm
    /// zeroes `road_cache` instead), so an edge tile reads whatever the
    /// previous tile left there.
    sweep_road: [bool; 9],
}

/// `road_compass_flags@00af2570` — the bit a tile's element claims toward
/// each compass neighbour, `[0]` the tile itself. Read out of the PE; it is
/// [`dir`] in [`MOVE_X`]/[`MOVE_Y`] order.
const COMPASS_FLAGS: [u32; 9] = [
    0,
    dir::NW,
    dir::N,
    dir::NE,
    dir::E,
    dir::SE,
    dir::S,
    dir::SW,
    dir::W,
];

/// `x + width_in_tiles · y` — how the original packs a `RoadModification` and
/// every `changed_roads` entry, `world->cell_width << 2` being the tile
/// width.
fn index_of(world: &World, t: Pos) -> i32 {
    t.x + world.width() * 4 * t.y
}

fn tile_of(world: &World, index: i32) -> Pos {
    let w = world.width() * 4;
    Pos::new(index.rem_euclid(w), index.div_euclid(w))
}

impl RoadMesh {
    /// The element of a tile, if it has one — `TerrainOut::get_road_data`
    /// with no leave to create. Nothing in the simulation reads it; the
    /// diffs do.
    pub fn elem(&self, world: &World, t: Pos) -> Option<Elem> {
        self.elems.get(&index_of(world, t)).copied()
    }

    /// How many elements stand. A count for the diffs, and the one number
    /// that says the store is not leaking.
    pub fn len(&self) -> usize {
        self.elems.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }

    /// How many road tiles the mesh laid itself — the elements carrying
    /// `is_terrain_creation`. Nothing in the simulation reads it; it is the
    /// number that says this module did anything at all, and the world diff
    /// beside it is what says the tiles are the original's.
    pub fn made(&self) -> usize {
        self.elems.values().filter(|e| e.made).count()
    }

    /// Every road tile of a world handed to us already built — a map's own
    /// roads, or a mid-game grid the harness borrowed — gets the element it
    /// would have had, derived the way
    /// `Roads::redo_changed_roads@0088e670`'s second pass derives one: flags
    /// cleared, then `mark_and_trim_directions` over the cache.
    ///
    /// It is a **reconstruction, not a replay** (`docs/ROADS.md` §9.4).
    pub fn seed(&mut self, world: &World) {
        let (tw, th) = (world.width() * 4, world.height() * 4);
        let mut tiles = Vec::new();
        for y in 0..th {
            for x in 0..tw {
                let t = Pos::new(x, y);
                if world.tile_mask(t) & tile::SURFACE == tile::SURFACE_ROAD {
                    tiles.push(t);
                    self.elems.insert(
                        index_of(world, t),
                        Elem {
                            refs: 1,
                            ..Elem::default()
                        },
                    );
                }
            }
        }
        for t in tiles {
            self.at = t;
            self.fill_cache(world, t);
            self.trim_directions(world, 0);
        }
    }

    /// `RoadsOut::fill_cache@00893b30` — the nine-entry compass, `[0]` the
    /// tile itself. Off the map both caches read zero.
    fn fill_cache(&mut self, world: &World, t: Pos) {
        let (tw, th) = (world.width() * 4, world.height() * 4);
        for d in 0..9 {
            let (x, y) = (t.x + MOVE_X[d], t.y + MOVE_Y[d]);
            if x < 0 || y < 0 || x >= tw || y >= th {
                self.blocked[d] = false;
                self.road[d] = false;
                continue;
            }
            let m = world.tile_mask(Pos::new(x, y));
            let road = m & tile::SURFACE == tile::SURFACE_ROAD;
            self.road[d] = road;
            self.blocked[d] = m & tile::OBJECT == tile::OBJECT_BUILDING && !road;
        }
    }

    /// `RoadsOut::get_orthog_connects@00893560` — how many of the four
    /// cardinals a tile's element claims. A tile with no element is zero.
    fn orthog_connects(&self, world: &World, t: Pos) -> i32 {
        self.elems
            .get(&index_of(world, t))
            .map_or(0, |e| (e.flags & dir::ORTHOG).count_ones() as i32)
    }

    /// `TerrainOut::road_changed@00866a70`. Answers the two reference counts
    /// summed — what `add_roads` compares against 2 and `clear_roads` against
    /// zero.
    fn road_changed(&mut self, world: &World, t: Pos, set: bool, p4: i32, p5: i32) -> i32 {
        let i = index_of(world, t);
        if set {
            let fresh = !self.elems.contains_key(&i);
            let e = self.elems.entry(i).or_default();
            if fresh {
                e.flags = 0;
            }
            if p5 == 0 {
                if fresh && p4 != 0 {
                    e.refs2 = e.refs2.wrapping_add(1);
                } else {
                    e.refs = e.refs.wrapping_add(1);
                }
            }
            return e.refs as i32 + e.refs2 as i32;
        }
        let Some(e) = self.elems.get_mut(&i) else {
            return 0;
        };
        if e.refs2 == 0 {
            e.refs = e.refs.wrapping_sub(1);
        } else {
            e.refs2 = e.refs2.wrapping_sub(1);
        }
        let (a, b) = (e.refs, e.refs2);
        if a == 0 && b == 0 {
            self.elems.remove(&i);
        }
        a as i32 + b as i32
    }

    /// `TerrainOut::get_road_data@008744d0` — whether the tile has an
    /// element, creating one under `create` and raising a count under `bump`.
    fn get_road_data(&mut self, world: &World, t: Pos, create: bool, p4: i32, bump: bool) -> bool {
        let i = index_of(world, t);
        if let Some(e) = self.elems.get_mut(&i) {
            if bump {
                if p4 == 0 {
                    e.refs = e.refs.wrapping_add(1);
                } else {
                    e.refs2 = e.refs2.wrapping_add(1);
                }
            }
            return true;
        }
        if create {
            self.road_changed(world, t, true, p4, 0);
            return true;
        }
        false
    }

    /// `ArrayBase<int>::add_unique` into `changed_roads` (`list == 1`) or
    /// `changed_roads2` (any other non-zero). Zero is the original's "no
    /// list", which every caller spells as a zero `param`.
    fn add_unique(&mut self, index: i32, list: i32) {
        if list == 0 {
            return;
        }
        let l = &mut self.redo[usize::from(list != 1)];
        if !l.contains(&index) {
            l.push(index);
        }
    }

    /// `flags |= mask` on a neighbour that has an element, creating one if it
    /// has none, and the redo-list entry the change earns. The original
    /// spells this three ways in three functions and they are the same
    /// three lines.
    fn join(&mut self, world: &World, t: Pos, mask: u32, list: i32) {
        if !self.get_road_data(world, t, true, 0, false) {
            return;
        }
        let i = index_of(world, t);
        let e = self.elems.get_mut(&i).unwrap();
        if e.flags & mask != 0 {
            return;
        }
        e.flags |= mask;
        self.add_unique(i, list);
    }

    fn flags_at(&self, world: &World, t: Pos) -> u32 {
        self.elems.get(&index_of(world, t)).map_or(0, |e| e.flags)
    }

    fn set_flags(&mut self, world: &World, t: Pos, flags: u32) {
        if let Some(e) = self.elems.get_mut(&index_of(world, t)) {
            e.flags = flags;
        }
    }

    /// `RoadsOut::leech_codes@00890da0` — the redo pass's step between the
    /// trim and `set_diags`: each cardinal neighbour that is a road lends
    /// this tile its answering claim. A road east whose element claims west
    /// makes this tile claim east, whatever the trim just withheld — which
    /// is what keeps a road beside a footprint joined to the rest of its
    /// run (`docs/ROADS.md` §11).
    ///
    /// The walk is north, south, east, west, and a road neighbour with **no
    /// element** clears this tile's bit toward it and ends the walk there:
    /// the listing returns from inside each arm. The other bits it copies —
    /// the white and yellow line codes, `0x100`…`0x80_0000` — belong to the
    /// texture passes this crate does not run, and only the direction is
    /// taken.
    fn leech_codes(&mut self, world: &World) {
        let at = self.at;
        for (d, own, back, dx, dy) in CARDINALS {
            if !self.road[d] {
                continue;
            }
            match self.elem(world, Pos::new(at.x + dx, at.y + dy)) {
                None => {
                    let f = self.flags_at(world, at) & !own;
                    self.set_flags(world, at, f);
                    return;
                }
                Some(n) if n.flags & back != 0 => {
                    let f = self.flags_at(world, at) | own;
                    self.set_flags(world, at, f);
                }
                Some(_) => {}
            }
        }
    }

    /// `RoadsOut::mark_and_trim_directions@008935c0` — the pass that decides,
    /// from the cache alone, which cardinals this tile is joined in, and
    /// joins each neighbour back.
    ///
    /// Two things in it are worth stating because neither is guessable from
    /// the shape:
    ///
    /// - `neighbor_cache[d]` is *a building footprint that is not a road*, so
    ///   whenever the neighbour **is** a road its gate is open and the bit is
    ///   simply `road_cache[d]`. The gate only ever withholds a
    ///   recomputation beside a building.
    /// - The two halves are gated on **each other's** axis. A footprint east
    ///   or west is what lets the north–south pair be recomputed, and one
    ///   north or south is what lets east–west be. With no footprint at all,
    ///   both run.
    fn trim_directions(&mut self, world: &World, list: i32) {
        let at = self.at;
        let mut a = self.blocked[2] || self.blocked[6];
        let b = self.blocked[4] || self.blocked[8];
        let mut do_ns = true;
        if !a && !b {
            a = true;
        } else if !b {
            do_ns = false;
        }
        let axis = |m: &mut RoadMesh, world: &World, k: usize| {
            let (d, own, back, dx, dy) = CARDINALS[k];
            if m.blocked[d] {
                return;
            }
            let mut f = m.flags_at(world, at) | own;
            if !m.road[d] {
                f &= !own;
                m.set_flags(world, at, f);
            } else {
                m.set_flags(world, at, f);
                m.join(world, Pos::new(at.x + dx, at.y + dy), back, list);
            }
        };
        if do_ns {
            axis(self, world, 0);
            axis(self, world, 1);
        }
        if a {
            axis(self, world, 2);
            axis(self, world, 3);
        }
        // The tail: a dead end takes every road neighbour it has, whatever
        // the gates above said.
        if (self.flags_at(world, at) & dir::ORTHOG).count_ones() != 1 {
            return;
        }
        for &(d, own, back, dx, dy) in &CARDINALS {
            if !self.road[d] {
                continue;
            }
            let f = self.flags_at(world, at) | own;
            self.set_flags(world, at, f);
            self.join(world, Pos::new(at.x + dx, at.y + dy), back, list);
        }
    }
}

impl Sim {
    /// **`Roads::scan_and_kill_stray_roads@008956a0`** (`docs/ROADS.md`
    /// §10): `Game::do_frame`'s last road step, right after `frame++`. A
    /// cursor walks the map a cell at a time, `size / 500` cells a frame
    /// — seven on a 60 × 60 map, the whole of it every 515 frames — and
    /// every tile of each cell is asked whether its road should stand.
    ///
    /// The cursor (`curscan_x`/`curscan_y`) starts at cell 0 and is
    /// stepped **before** each cell, so it is a function of the frame and
    /// carried nowhere: run189's packet holds it at cell 903 after 14,529
    /// frames, `14,529 × 7 mod 3,600`.
    pub(crate) fn scan_and_kill_stray_roads(&mut self) {
        let (xs, ys) = (self.world.width(), self.world.height());
        let size = xs * ys;
        let per = size / 500;
        if per <= 0 {
            return;
        }
        let done = i64::from(per) * (self.frame - 1);
        for j in 1..=i64::from(per) {
            let idx = ((done + j) % i64::from(size)) as i32;
            let (cx, cy) = (idx % xs, idx / xs);
            for i in 0..16 {
                let t = Pos::new(cx * 4 + (i & 3), cy * 4 + (i >> 2));
                self.scan_stray_tile(t);
            }
        }
    }

    /// One tile of the sweep: the four compasses, then
    /// `scan_and_kill_bad_tcoord@0088e100` and — on a road —
    /// `scan_and_kill_straggled_tcoord@0088e050`.
    fn scan_stray_tile(&mut self, t: Pos) {
        let (tw, th) = (self.world.width() * 4, self.world.height() * 4);
        let elem = self.mesh.elem(&self.world, t);
        let mut build = [false; 9];
        let mut water = [false; 9];
        let mut sup = [false; 9];
        for k in 0..9 {
            let (x, y) = (t.x + MOVE_X[k], t.y + MOVE_Y[k]);
            if x < 0 || y < 0 || x >= tw || y >= th {
                continue;
            }
            let m = self.world.tile_mask(Pos::new(x, y));
            let road = m & tile::SURFACE == tile::SURFACE_ROAD;
            self.mesh.sweep_road[k] = road;
            build[k] = m & tile::OBJECT == tile::OBJECT_BUILDING;
            water[k] = m & tile::RIVER != 0 || m & tile::SURFACE == tile::SURFACE_OCEAN;
            // The **centre's** element, read at the centre's own slot: does
            // the tile claim the direction whose neighbour is a road.
            sup[k] = road && elem.is_some_and(|e| e.flags & COMPASS_FLAGS[k] != 0);
        }
        let road = self.mesh.sweep_road;
        // `scan_and_kill_bad_tcoord`.
        match elem {
            // No element on the tile — `rotation == 10`, an empty slot: a
            // road nothing in the mesh holds goes, and nothing is counted
            // down, because nothing holds it.
            None => {
                if road[0] {
                    self.world_set_road_at(t, false, 0, 0);
                }
            }
            // The mesh's own tile (or the `element_C4` piece, which this
            // crate does not pick): it stands while any of the nine is a
            // road.
            Some(e) if e.made => {
                if !road.iter().any(|&r| r) {
                    self.kill_stray(t);
                }
            }
            Some(_) => {
                // Every claimed direction has a road on the map there by
                // construction of `sup`, so the arm kills a tile that
                // claims none.
                if !road[0] || !sup[1..].iter().any(|&s| s) {
                    self.kill_stray(t);
                }
            }
        }
        // `scan_and_kill_straggled_tcoord`, on the cached centre: a road
        // that claims exactly one neighbour, with no building and no water
        // beside it, is a stub.
        if road[0] {
            let one = sup[1..].iter().filter(|&&s| s).count() == 1;
            let near = build[1..].iter().any(|&b| b) || water[1..].iter().any(|&w| w);
            if one && !near {
                self.kill_stray(t);
            }
        }
    }

    /// The sweep's kill: `TerrainOut::road_changed(x, y, 0, 0, 1)`, one
    /// reference down, then `World::set_road_at(x, y, 0, 0, 0)` through the
    /// door, which queues the removal and trims the neighbours.
    fn kill_stray(&mut self, t: Pos) {
        self.mesh.road_changed(&self.world, t, false, 0, 1);
        self.world_set_road_at(t, false, 0, 0);
    }

    /// **`World::set_blocked_at@006b4900`, whole** (`docs/ROADS.md` §9.5):
    /// the counts and the halo bits are [`crate::world::World::set_blocked_at`]'s,
    /// and the blocking arm's last statement before the halo loop is
    /// `set_road_at(x, y, 0, 0, 0)` — **a tile that becomes blocked loses
    /// its road**, through the mesh's door, on every call with `on` set,
    /// whether or not the tile was blocked before. The unblocking arm lays
    /// nothing. Every writer of the blocked bit in the simulation proper
    /// goes through here; the world's own half stays for the map loader and
    /// the tests, which have no mesh.
    pub(crate) fn set_blocked_at(&mut self, t: Pos, on: bool) {
        self.world.set_blocked_at(t, on);
        if on && self.world.tile_in_bounds(t) {
            self.world_set_road_at(t, false, 0, 0);
        }
    }

    /// `World::set_road_at@006b43b0` — the one door into the mesh.
    ///
    /// `p4` is the original's fourth argument, which only decides whether the
    /// terrain is told; `p5` is the recursion stop, and the mesh's own calls
    /// pass it 1 so that a tile it lays does not re-enter `add_roads`.
    pub(crate) fn world_set_road_at(&mut self, t: Pos, set: bool, p4: i32, p5: i32) {
        let m = self.world.tile_mask(t);
        let c = Cell::new(t.x.div_euclid(4), t.y.div_euclid(4));
        if !set {
            if m & tile::SURFACE == tile::SURFACE_ROAD {
                self.world.set_tile_mask(t, m & !tile::SURFACE);
                // The other half of the door, and it is the mesh's:
                // `Roads::road_cleared@008955d0` queues the tile as a
                // *removal* and runs `add_roads` over it, which is what
                // takes the element down. Without it a tile could stop
                // being a road and keep its `RoadElementCandidate`, and
                // `get_orthog_connects` would go on answering for it.
                if p5 == 0 {
                    self.road_cleared(t);
                }
            }
            // The cell keeps its `ROAD` flag while any of its sixteen tiles
            // is still a road — and the scan runs whether or not this tile
            // was one.
            let any = (0..16).any(|k| {
                let q = Pos::new(c.x * 4 + k % 4, c.y * 4 + k / 4);
                self.world.tile_mask(q) & tile::SURFACE == tile::SURFACE_ROAD
            });
            if !any {
                let mut d = self.world.cell_data(c);
                d.flags &= !cell::ROAD;
                self.world.set_cell_data(c, d);
            }
            return;
        }
        self.world
            .set_tile_mask(t, (m & !tile::SURFACE_OCEAN) | tile::SURFACE_ROAD);
        let mut d = self.world.cell_data(c);
        d.flags |= cell::ROAD;
        self.world.set_cell_data(c, d);
        if m & tile::SURFACE != tile::SURFACE_ROAD && p5 == 0 {
            self.road_added(t, p4, 1);
        }
    }

    /// `Roads::road_cleared@008955d0` — queue the tile as a removal and run
    /// the mesh. The original's `RoadModification` carries `added = 0` and
    /// `valid = 1`, and `add_roads` is entered with a zero.
    fn road_cleared(&mut self, t: Pos) {
        let index = index_of(&self.world, t);
        self.mesh.mods.push(Modification {
            index,
            added: 0,
            valid: 1,
        });
        self.add_roads(0);
    }

    /// `Roads::road_added@008954d0` — queue the tile, and run the mesh when
    /// asked to.
    fn road_added(&mut self, t: Pos, p3: i32, p4: i32) {
        if p3 != 0 {
            self.mesh.road_changed(&self.world, t, true, 1, 0);
        }
        let index = index_of(&self.world, t);
        self.mesh.mods.push(Modification {
            index,
            added: 1,
            valid: 1,
        });
        if p4 != 0 {
            self.add_roads(i32::from(p3 != 0));
        }
    }

    /// `Roads::add_roads@0088f4b0`, in the three passes that reach the world.
    ///
    /// Pass one takes each queued tile, counts its element's references and
    /// either derives its directions or — when the element was already
    /// standing — re-queues it as a *removal* and remembers to drop the
    /// original entry. Pass two runs `set_diags`, which is where road gets
    /// laid; a tile it lays is appended to the very queue being walked, so
    /// the pass sees it. The tail then re-derives every tile either pass
    /// touched, ping-ponging between two lists until neither grows, at most
    /// eleven times round.
    fn add_roads(&mut self, param: i32) {
        self.mesh.remove.clear();
        // ---- pass one
        let mut cursor = 0;
        while cursor < self.mesh.mods.len() {
            let cur = self.mesh.mods[cursor];
            let at = tile_of(&self.world, cur.index);
            self.mesh.at = at;
            if cur.valid != 0 {
                if cur.added == 0 {
                    self.clear_roads(cur);
                    if let Some(e) = self
                        .mesh
                        .mods
                        .iter_mut()
                        .find(|e| e.index == cur.index && e.valid == 1)
                    {
                        e.valid = 0;
                    }
                } else {
                    self.mesh.fill_cache(&self.world, at);
                    let n = self
                        .mesh
                        .road_changed(&self.world, at, true, 0, i32::from(param != 0));
                    if n < 2 {
                        if self.mesh.get_road_data(&self.world, at, false, 0, false) {
                            self.mesh.trim_directions(&self.world, 1);
                        }
                    } else {
                        self.mesh.mods.push(Modification {
                            index: cur.index,
                            added: 0,
                            valid: 1,
                        });
                        self.mesh.remove.push(cur);
                    }
                }
            }
            cursor += 1;
        }
        // ---- the removals pass one asked for
        let remove = std::mem::take(&mut self.mesh.remove);
        for r in remove {
            if let Some(k) = self
                .mesh
                .mods
                .iter()
                .position(|e| e.index == r.index && e.added == r.added)
            {
                self.mesh.mods.remove(k);
            }
        }
        // ---- pass two
        let mut cursor = 0;
        while cursor < self.mesh.mods.len() {
            let cur = self.mesh.mods[cursor];
            cursor += 1;
            if cur.valid == 0 {
                continue;
            }
            let at = tile_of(&self.world, cur.index);
            self.mesh.at = at;
            if !self.mesh.get_road_data(&self.world, at, false, 0, false) {
                continue;
            }
            if cur.added == 0 {
                self.mesh.trim_directions(&self.world, 1);
            }
            self.mesh.fill_cache(&self.world, at);
            self.set_diags(1, param);
        }
        // ---- the redo ping-pong
        let mut which = 0;
        for _ in 0..12 {
            if self.mesh.redo[which].is_empty() {
                break;
            }
            self.redo_changed_roads(which, if which == 0 { 2 } else { 1 }, param);
            self.mesh.redo[which].clear();
            which = 1 - which;
        }
        self.mesh.redo[0].clear();
        self.mesh.redo[1].clear();
        self.mesh.mods.clear();
    }

    /// `Roads::set_diags@0088e9d0`. Answers the original's `local_8` — set
    /// when some corner was *joined* rather than filled, and the only reason
    /// the answer matters is that a zero sends `add_roads` on to the passes
    /// that pick a texture.
    ///
    /// The gate at the top is the mechanic in one line: **a tile with two or
    /// more orthogonal connections does none of this**. What the pass is for
    /// is a road that ends, or turns, beside another one.
    ///
    /// `_param` is the original's second argument, which only reaches the
    /// rendering tail this module does not have (`docs/ROADS.md` §9.4).
    fn set_diags(&mut self, list: i32, _param: i32) -> i32 {
        let at = self.mesh.at;
        if self.mesh.orthog_connects(&self.world, at) > 1
            || !self.mesh.get_road_data(&self.world, at, false, 0, false)
        {
            return 0;
        }
        let (tw, th) = (self.world.width() * 4, self.world.height() * 4);
        let mut joined = 0;
        for i in 1..=4usize {
            let corner = Pos::new(at.x + CORNER_X[i], at.y + CORNER_Y[i]);
            if corner.x < 0 || corner.y < 0 || corner.x >= tw || corner.y >= th {
                continue;
            }
            // The four exclusions: a cardinal connection forbids the two
            // corners that touch it.
            let f = self.mesh.flags_at(&self.world, at);
            let barred = (f & dir::N != 0 && (i == 1 || i == 2))
                || (f & dir::E != 0 && (i == 2 || i == 3))
                || (f & dir::S != 0 && (i == 3 || i == 4))
                || (f & dir::W != 0 && (i == 4 || i == 1));
            if barred || !self.mesh.road[tables::DIAG[i]] {
                continue;
            }
            if self.mesh.orthog_connects(&self.world, corner) < 2 {
                joined = 1;
                if f & tables::BLOCK_A[i] == 0 && f & tables::BLOCK_B[i] == 0 {
                    self.mesh.set_flags(&self.world, at, f | tables::OWN[i]);
                    self.mesh
                        .join(&self.world, corner, tables::OPPOSITE[i], list);
                }
                continue;
            }
            // The corner is a road junction in its own right. Fill the gap
            // between the two — the *horizontal* neighbour, whichever way
            // the corner lies — if neither tile between them is a road or a
            // footprint.
            let (fa, fb) = (tables::FLANK_A[i], tables::FLANK_B[i]);
            if self.mesh.road[fa]
                || self.mesh.blocked[fa]
                || self.mesh.road[fb]
                || self.mesh.blocked[fb]
            {
                continue;
            }
            let made = Pos::new(at.x + CORNER_X[i], at.y);
            self.world_set_road_at(made, true, 0, 1);
            self.road_added(made, 0, 0);
            self.mesh.road_changed(&self.world, made, true, 0, 0);
            self.mesh.add_unique(index_of(&self.world, made), list);
            if !self.mesh.get_road_data(&self.world, made, false, 0, false) {
                return 0;
            }
            let i_made = index_of(&self.world, made);
            self.mesh.elems.get_mut(&i_made).unwrap().made = true;
            let i_at = index_of(&self.world, at);
            let e = self.mesh.elems.get_mut(&i_at).unwrap();
            e.support |= SUPPORT_MADE;
            if CORNER_X[i] == 1 {
                e.support |= SUPPORT_EAST;
            }
            self.mesh.add_unique(i_at, list);
            self.mesh.add_unique(index_of(&self.world, corner), list);
        }
        joined
    }

    /// `Roads::clear_roads@0088fef0` — a queued *removal*. The road tile only
    /// goes if the mesh laid it and nothing else still holds its element.
    fn clear_roads(&mut self, m: Modification) {
        let at = tile_of(&self.world, m.index);
        self.mesh.at = at;
        let mine = self.clear_support(at, 0);
        self.mesh.fill_cache(&self.world, at);
        let left = self.mesh.road_changed(&self.world, at, false, 0, 1);
        if mine && left == 0 {
            self.world_set_road_at(at, false, 0, 1);
        }
        for (d, _, _, dx, dy) in CARDINALS {
            if self.mesh.road[d] {
                let n = Pos::new(at.x + dx, at.y + dy);
                self.mesh.add_unique(index_of(&self.world, n), 1);
            }
        }
    }

    /// `Roads::clear_support@0088e3f0`, the one arm of it that moves a road
    /// tile: `0x2000` says this element made the road beside it, and `0x80`
    /// says which side. `keep` is the original's third argument — the redo
    /// pass sets it, which is why a mesh-laid tile survives being re-derived.
    ///
    /// Answers `is_terrain_creation`, which is what `clear_roads` needs.
    fn clear_support(&mut self, at: Pos, keep: i32) -> bool {
        let Some(e) = self.mesh.elem(&self.world, at) else {
            return false;
        };
        let mine = e.made;
        if e.support & 0xe000 == 0 {
            self.mesh
                .elems
                .get_mut(&index_of(&self.world, at))
                .unwrap()
                .support = 0;
            return mine;
        }
        if keep == 0 && e.support & SUPPORT_MADE != 0 {
            let dx = if e.support & SUPPORT_EAST != 0 { 1 } else { -1 };
            let made = Pos::new(at.x + dx, at.y);
            self.mesh
                .elems
                .get_mut(&index_of(&self.world, at))
                .unwrap()
                .support = 0;
            if let Some(n) = self.mesh.elem(&self.world, made) {
                let was = n.made;
                let left = self.mesh.road_changed(&self.world, made, false, 0, 1);
                if left == 0 && was {
                    self.world_set_road_at(made, false, 0, 1);
                }
            }
        }
        mine
    }

    /// `Roads::redo_changed_roads@0088e670` — the three passes over one of
    /// the two lists: drop the support links, re-derive the directions from
    /// scratch, then take each road neighbour's answering claim
    /// (`leech_codes`) and run `set_diags` again. Anything it touches goes on
    /// the *other* list, which is what the ping-pong walks next.
    fn redo_changed_roads(&mut self, which: usize, list: i32, param: i32) {
        let work = self.mesh.redo[which].clone();
        for &index in &work {
            let at = tile_of(&self.world, index);
            self.mesh.at = at;
            self.clear_support(at, 1);
        }
        for &index in &work {
            let at = tile_of(&self.world, index);
            self.mesh.at = at;
            if !self.mesh.get_road_data(&self.world, at, false, 0, false) {
                continue;
            }
            self.mesh.set_flags(&self.world, at, 0);
            self.mesh.fill_cache(&self.world, at);
            self.mesh.trim_directions(&self.world, list);
        }
        for &index in &work {
            let at = tile_of(&self.world, index);
            self.mesh.at = at;
            if !self.mesh.get_road_data(&self.world, at, false, 0, false) {
                continue;
            }
            self.mesh.fill_cache(&self.world, at);
            self.mesh.leech_codes(&self.world);
            self.set_diags(list, param);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tuning;
    use crate::world::{Terrain, World};

    /// Sixty cells of plain land and nothing on them. The mesh is the whole
    /// subject here, so the fixture has no buildings: `set_road_at` is
    /// called by hand, tile by tile, the way `place_roads` calls it.
    fn bare() -> Sim {
        let mut w = World::new(60, 60);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(59, 59));
        Sim::new(Tuning::RON, w, 2)
    }

    fn is_road(sim: &Sim, x: i32, y: i32) -> bool {
        sim.world.tile_mask(Pos::new(x, y)) & tile::SURFACE == tile::SURFACE_ROAD
    }

    fn lay(sim: &mut Sim, tiles: &[(i32, i32)]) {
        for &(x, y) in tiles {
            sim.set_road_at(Pos::new(x, y));
        }
    }

    /// An elbow at `(223, 80)` — west and south — is a road with **two**
    /// orthogonal connections, which is what `set_diags`' corner branch
    /// wants.
    fn elbow(sim: &mut Sim) {
        lay(sim, &[(222, 80), (223, 80), (223, 81)]);
        assert_eq!(
            sim.mesh.orthog_connects(&sim.world, Pos::new(223, 80)),
            2,
            "the elbow joins west and south"
        );
    }

    /// **run72's frame 4803, in miniature.** The road already stands at
    /// `(223, 80)`; the new tile is `(224, 79)`, diagonally off it; neither
    /// `(223, 79)` nor `(224, 80)` is anything. The mesh lays `(223, 79)` —
    /// the seventeenth tile of a sixteen-tile ring.
    #[test]
    fn a_new_road_diagonally_off_a_junction_fills_the_tile_between_them() {
        let mut sim = bare();
        elbow(&mut sim);
        assert!(!is_road(&sim, 223, 79), "nothing there yet");
        sim.set_road_at(Pos::new(224, 79));
        assert!(is_road(&sim, 223, 79), "the mesh fills the corner");
        assert_eq!(sim.mesh.made(), 1, "and it is the mesh's own tile");
        let made = sim.mesh.elem(&sim.world, Pos::new(223, 79)).unwrap();
        assert!(made.made, "`is_terrain_creation` says who laid it");
        let at = sim.mesh.elem(&sim.world, Pos::new(224, 79)).unwrap();
        assert_eq!(
            at.support & (SUPPORT_MADE | SUPPORT_EAST),
            SUPPORT_MADE,
            "and the tile that laid it remembers, westward"
        );
    }

    /// The gate: **two orthogonal connections and the pass does nothing at
    /// all**. `(224, 79)` here is the middle of a road running east–west, so
    /// its own corners are never looked at.
    #[test]
    fn a_tile_with_two_orthogonal_connections_lays_nothing() {
        let mut sim = bare();
        elbow(&mut sim);
        lay(&mut sim, &[(225, 79), (224, 78)]);
        sim.set_road_at(Pos::new(224, 79));
        assert_eq!(
            sim.mesh.orthog_connects(&sim.world, Pos::new(224, 79)),
            2,
            "north and east"
        );
        assert!(!is_road(&sim, 223, 79), "so the corner is left open");
        assert_eq!(sim.mesh.made(), 0);
    }

    /// Either flank being a road is enough to stop it: there is already a
    /// way round.
    #[test]
    fn a_flank_that_is_already_a_road_stops_the_fill() {
        let mut sim = bare();
        elbow(&mut sim);
        // `(224, 80)` is the southern flank of the south-west corner.
        lay(&mut sim, &[(224, 80)]);
        let made = sim.mesh.made();
        sim.set_road_at(Pos::new(224, 79));
        assert_eq!(sim.mesh.made(), made, "the way round is the flank itself");
    }

    /// And a **building footprint** on a flank stops it too — the other half
    /// of `neighbor_cache`, and the reason the cache is two arrays rather
    /// than one.
    #[test]
    fn a_footprint_on_a_flank_stops_the_fill_as_a_road_would() {
        let mut sim = bare();
        elbow(&mut sim);
        let t = Pos::new(223, 79);
        let m = sim.world.tile_mask(t);
        sim.world.set_tile_mask(t, m | tile::OBJECT_BUILDING);
        sim.set_road_at(Pos::new(224, 79));
        assert!(!is_road(&sim, 223, 79), "a footprint is not paved over");
        assert_eq!(sim.mesh.made(), 0);
    }

    /// A corner road that is a **dead end** — one orthogonal connection —
    /// takes the other branch: the two elements are joined diagonally and no
    /// tile is laid.
    #[test]
    fn a_dead_end_corner_is_joined_rather_than_filled() {
        let mut sim = bare();
        lay(&mut sim, &[(222, 80), (223, 80)]);
        assert_eq!(
            sim.mesh.orthog_connects(&sim.world, Pos::new(223, 80)),
            1,
            "the road ends there"
        );
        sim.set_road_at(Pos::new(224, 79));
        assert!(!is_road(&sim, 223, 79), "nothing is laid");
        let at = sim.mesh.elem(&sim.world, Pos::new(224, 79)).unwrap();
        assert_eq!(at.flags & dir::SW, dir::SW, "the tile claims its corner");
        let corner = sim.mesh.elem(&sim.world, Pos::new(223, 80)).unwrap();
        assert_eq!(corner.flags & dir::NE, dir::NE, "and the corner answers");
    }

    /// `mark_and_trim_directions`, straight: a road tile claims exactly the
    /// cardinals that are roads, and each neighbour claims it back.
    #[test]
    fn a_straight_run_claims_the_cardinals_that_are_roads() {
        let mut sim = bare();
        lay(&mut sim, &[(10, 10), (10, 11), (10, 12), (11, 11)]);
        let e = |s: &Sim, x, y| s.mesh.elem(&s.world, Pos::new(x, y)).unwrap().flags;
        assert_eq!(
            e(&sim, 10, 11) & dir::ORTHOG,
            dir::N | dir::S | dir::E,
            "the middle of a T"
        );
        assert_eq!(e(&sim, 10, 10) & dir::ORTHOG, dir::S, "the top end");
        assert_eq!(e(&sim, 11, 11) & dir::ORTHOG, dir::W, "the arm");
    }

    /// **The removal half, driven by hand.** `clear_roads` is reached from
    /// one place — a queue entry with `added == 0`, which `add_roads` only
    /// writes when a tile it is laying **already had an element**. Nothing
    /// in this crate creates an element on a tile that is not a road, so
    /// the branch is unreachable here and the passes below would otherwise
    /// never run (`docs/ROADS.md` §9.4). They are called directly instead.
    ///
    /// What they do: a tile the mesh laid itself goes when the last
    /// reference does; a tile somebody else laid stays whatever the count
    /// says.
    #[test]
    fn a_cleared_tile_goes_only_if_the_mesh_laid_it() {
        let mut sim = bare();
        elbow(&mut sim);
        sim.set_road_at(Pos::new(224, 79));
        assert!(is_road(&sim, 223, 79), "the fill is down");

        // Somebody else's tile: the count falls, the road stays.
        let plain = Pos::new(222, 80);
        let m = Modification {
            index: index_of(&sim.world, plain),
            added: 0,
            valid: 1,
        };
        sim.clear_roads(m);
        assert!(!is_road(&sim, 222, 80) || sim.mesh.elem(&sim.world, plain).is_none());

        // The mesh's own: `clear_support` on the tile that made it takes
        // both, because `is_terrain_creation` is set and the count runs out.
        let at = Pos::new(224, 79);
        let m = Modification {
            index: index_of(&sim.world, at),
            added: 0,
            valid: 1,
        };
        sim.clear_roads(m);
        assert!(!is_road(&sim, 223, 79), "the fill goes with what made it");
        assert_eq!(sim.mesh.made(), 0);
    }

    /// Every road tile has an element and nothing else does — the invariant
    /// the diffs pin on run72's mid-game grid, here on a fixture.
    #[test]
    fn an_element_stands_for_each_road_tile_and_no_other() {
        let mut sim = bare();
        elbow(&mut sim);
        sim.set_road_at(Pos::new(224, 79));
        let mut roads = 0;
        for y in 70..90 {
            for x in 215..235 {
                let t = Pos::new(x, y);
                let road = is_road(&sim, x, y);
                roads += i32::from(road);
                assert_eq!(
                    road,
                    sim.mesh.elem(&sim.world, t).is_some(),
                    "tile ({x}, {y})"
                );
            }
        }
        assert_eq!(roads, 5, "three elbow tiles, the new one, and the fill");
        assert_eq!(sim.mesh.len(), 5, "and no element outside them");
    }

    // ------------------------------------------------------------------
    // The stray-road sweep (`docs/ROADS.md` §10)
    // ------------------------------------------------------------------

    /// A north–south road of six tiles on open ground, `(100, 40)` to
    /// `(100, 45)`.
    fn lane(sim: &mut Sim) {
        let tiles: Vec<(i32, i32)> = (40..46).map(|y| (100, y)).collect();
        lay(sim, &tiles);
    }

    /// **A road with nothing at its ends erodes from both, a tile a visit**
    /// — Great Lakes' trade road from (216, 115), a stub on 5905, to
    /// (220, 98) on 14100. Each end claims one neighbour, with no building
    /// and no water beside it: `scan_and_kill_straggled_tcoord` takes it,
    /// and the removal trims the next tile down to one claim of its own.
    ///
    /// Made to fail once with the straggler arm removed: the lane stood.
    #[test]
    fn a_lane_with_nothing_at_its_ends_erodes_a_tile_a_visit() {
        let mut sim = bare();
        lane(&mut sim);
        sim.scan_stray_tile(Pos::new(100, 44));
        assert!(is_road(&sim, 100, 44), "the middle claims two: it stands");
        sim.scan_stray_tile(Pos::new(100, 45));
        assert!(!is_road(&sim, 100, 45), "the south end is a stub");
        assert!(sim.mesh.elem(&sim.world, Pos::new(100, 45)).is_none());
        assert_eq!(
            sim.mesh
                .elem(&sim.world, Pos::new(100, 44))
                .map(|e| e.flags),
            Some(dir::N),
            "the next tile is trimmed to its one claim"
        );
        for y in (40..45).rev() {
            sim.scan_stray_tile(Pos::new(100, y));
        }
        assert!(
            (40..46).all(|y| !is_road(&sim, 100, y)),
            "and the lane is gone"
        );
    }

    /// **A building beside the end keeps it** — the straggler test counts
    /// `(mask & 3) == 3` neighbours, so a road that reaches a footprint is
    /// not a stub. Great Lakes' (216, 116) stands beside the city.
    #[test]
    fn a_road_that_ends_at_a_building_is_not_a_stub() {
        let mut sim = bare();
        lane(&mut sim);
        let foot = Pos::new(101, 46);
        let m = sim.world.tile_mask(foot);
        sim.world.set_tile_mask(foot, m | tile::OBJECT_BUILDING);
        sim.scan_stray_tile(Pos::new(100, 45));
        assert!(is_road(&sim, 100, 45), "a footprint beside it: it stands");
    }

    /// **A road tile with no element goes, and nothing is counted down** —
    /// `scan_and_kill_bad_tcoord`'s first arm, `rotation == 10`.
    #[test]
    fn a_road_tile_the_mesh_does_not_hold_is_taken_away() {
        let mut sim = bare();
        let t = Pos::new(100, 40);
        let m = sim.world.tile_mask(t);
        sim.world.set_tile_mask(t, m | tile::SURFACE_ROAD);
        assert!(sim.mesh.elem(&sim.world, t).is_none());
        sim.scan_stray_tile(t);
        assert!(!is_road(&sim, 100, 40));
    }

    /// **The cursor is the frame's**: seven cells a frame on a 60 × 60
    /// map, stepped before each, so the frame that has just become 14,529
    /// scans cells 897 to 903 — where run189's packet holds `curscan` —
    /// and not 904.
    #[test]
    fn the_sweep_s_cursor_is_seven_cells_a_frame_from_cell_zero() {
        let mut sim = bare();
        let at = |c: i32| Pos::new((c % 60) * 4, (c / 60) * 4);
        for c in [896, 897, 903, 904] {
            let m = sim.world.tile_mask(at(c));
            sim.world.set_tile_mask(at(c), m | tile::SURFACE_ROAD);
        }
        sim.frame = 14_529;
        sim.scan_and_kill_stray_roads();
        let road = |sim: &Sim, c: i32| is_road(sim, at(c).x, at(c).y);
        assert!(road(&sim, 896), "cell 896 was the frame before's");
        assert!(
            !road(&sim, 897) && !road(&sim, 903),
            "897..903 are this frame's"
        );
        assert!(road(&sim, 904), "904 is the next frame's");
    }

    /// **A tile that becomes blocked loses its road** (`World::set_blocked_at@006b4900`'s
    /// `set_road_at(x, y, 0, 0, 0)`, `docs/ROADS.md` §9.5, item 1185). East
    /// Indies' Temple `1/2025` started on 7479 over the caravan road's tile
    /// (200, 202); the original's caravan found it plain ground on 7512 and
    /// laid its road again, and this crate kept the road under the blocked
    /// tile. Unblocking lays nothing back, and a neighbour keeps its road.
    #[test]
    fn a_tile_that_becomes_blocked_loses_its_road() {
        let mut sim = bare();
        lay(&mut sim, &[(20, 20), (21, 20), (22, 20)]);
        assert!(is_road(&sim, 21, 20));
        sim.set_blocked_at(Pos::new(21, 20), true);
        assert!(!is_road(&sim, 21, 20), "the blocked tile's road goes");
        assert!(
            sim.world.tile_mask(Pos::new(21, 20)) & tile::BLOCKED != 0,
            "and it is blocked"
        );
        assert!(
            is_road(&sim, 20, 20) && is_road(&sim, 22, 20),
            "its neighbours keep theirs"
        );
        sim.set_blocked_at(Pos::new(21, 20), false);
        assert!(!is_road(&sim, 21, 20), "unblocking lays nothing");
        // **Every call clears**, not only a tile's first blocking: the
        // original's `set_road_at` sits outside the `& 0x4000` guard that
        // keeps the counts.
        sim.set_blocked_at(Pos::new(21, 20), true);
        lay(&mut sim, &[(21, 20)]);
        assert!(is_road(&sim, 21, 20), "a road laid over a blocked tile");
        sim.set_blocked_at(Pos::new(21, 20), true);
        assert!(!is_road(&sim, 21, 20), "goes when it is blocked again");
    }

    /// **A road beside a footprint stays joined to its run** —
    /// `RoadsOut::leech_codes@00890da0` in the redo pass (`docs/ROADS.md`
    /// §11, item 1260). Great Sahara at Toughest's Farm took the west end
    /// of a trade road on 6611; the tile beside it was re-derived with a
    /// footprint to its west, so the trim recomputed north–south alone and
    /// the tile's claim east was lost with the zeroing. The east neighbour's
    /// claim west lends it back. Without it the tile claims nothing, and the
    /// sweep takes it and then the rest of the run as stubs, where the
    /// original kept them.
    #[test]
    fn a_road_beside_a_footprint_keeps_the_claim_its_neighbour_lends_it() {
        let mut sim = bare();
        lay(
            &mut sim,
            &[(20, 20), (21, 20), (22, 20), (23, 20), (24, 20)],
        );
        let end = Pos::new(20, 20);
        let m = sim.world.tile_mask(end);
        sim.world.set_tile_mask(end, m | tile::OBJECT_BUILDING);
        sim.world_set_road_at(end, false, 0, 0);
        assert!(
            !is_road(&sim, 20, 20),
            "the footprint's tile loses its road"
        );
        assert_eq!(
            sim.mesh
                .elem(&sim.world, Pos::new(21, 20))
                .map(|e| e.flags & 0xff00_0000),
            Some(dir::E),
            "the tile beside it claims east, lent by (22, 20)'s west"
        );
        // The far end, (24, 20), is a stub in open country and erodes on
        // its own visit; the tile beside the footprint is what is asked.
        sim.scan_stray_tile(Pos::new(21, 20));
        assert!(
            (21..25).all(|x| is_road(&sim, x, 20)),
            "and the sweep leaves it standing"
        );
    }

    /// **A road neighbour with no element ends the walk** — each arm of
    /// `leech_codes` returns from inside when `get_road_data` answers
    /// nothing, clearing this tile's claim toward it and leaving the
    /// cardinals after it unread.
    #[test]
    fn leech_codes_stops_at_a_road_with_no_element() {
        let mut sim = bare();
        lay(&mut sim, &[(30, 30), (31, 30)]);
        // A road north of (30, 30) that the mesh does not hold.
        let north = Pos::new(30, 29);
        let m = sim.world.tile_mask(north);
        sim.world.set_tile_mask(north, m | tile::SURFACE_ROAD);
        let at = Pos::new(30, 30);
        sim.mesh.set_flags(&sim.world, at, dir::N);
        sim.mesh.at = at;
        sim.mesh.fill_cache(&sim.world, at);
        sim.mesh.leech_codes(&sim.world);
        assert_eq!(
            sim.mesh.elem(&sim.world, at).map(|e| e.flags & 0xff00_0000),
            Some(0),
            "north is cleared, and east is never lent"
        );
        // With the north road gone, the walk reaches east.
        sim.world.set_tile_mask(north, m);
        sim.mesh.fill_cache(&sim.world, at);
        sim.mesh.leech_codes(&sim.world);
        assert_eq!(
            sim.mesh.elem(&sim.world, at).map(|e| e.flags & 0xff00_0000),
            Some(dir::E),
            "(31, 30) claims west, so (30, 30) claims east"
        );
    }

    /// **And a building that starts over a road takes it** — the same arm,
    /// reached the way the Temple reached it: `Wall::start` →
    /// `BuildType::mask_me` → `set_blocked_at` on each template tile of a
    /// type that connects to roads, where `mask_me`'s own removal arm does
    /// not fire.
    #[test]
    fn a_building_that_starts_over_a_road_takes_it_under_its_blocked_tiles() {
        let mut sim = bare();
        let bt = sim.add_build_type(crate::build::BuildType {
            x_size: 3,
            y_size: 3,
            ..crate::build::BuildType::default()
        });
        assert!(
            crate::roads::connects_to_roads(&sim.build_types, bt),
            "the type connects to roads, so mask_me's own arm lays rather than clears"
        );
        let at = Pos::new(30 * 0x300 + 0x180, 30 * 0x300 + 0x180);
        let b = sim.add_building(0, at, 0);
        sim.buildings[b].ty = Some(bt);
        sim.buildings[b].started = false;
        let corner = sim.tile_corner(bt, at);
        let under = Pos::new(corner.x + 1, corner.y + 1);
        lay(
            &mut sim,
            &[
                (under.x - 3, under.y),
                (under.x - 2, under.y),
                (under.x - 1, under.y),
                (under.x, under.y),
            ],
        );
        assert!(is_road(&sim, under.x, under.y));
        sim.start_building(b);
        assert!(
            sim.world.tile_mask(under) & tile::BLOCKED != 0,
            "the footprint's middle is blocked"
        );
        assert!(!is_road(&sim, under.x, under.y), "and its road is gone");
    }
}
