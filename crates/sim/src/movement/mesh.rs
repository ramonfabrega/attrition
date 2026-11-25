//! The road **mesh** — `Roads::add_roads@0088f4b0` and the road tiles
//! `Roads::set_diags@0088e9d0` lays of its own.
//!
//! `docs/ROADS.md` §9. `crate::movement::roads` is the *plan* — the ring a building
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
use crate::ai::ai_place::{MOVE_X, MOVE_Y};
use crate::movement::roads_view::{ModQueue, RoadsView, TileRec, Which, add_roads_steps};
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
/// `support_codes`' two **corner** notes (item 3287, `docs/ROADS.md` §12):
/// `set_diags`' tail puts a C4 element on two cardinal neighbours of a tile
/// whose claims meet at a corner, and writes `0x8000 | 0x4000` with a side
/// for each — `0x1000`/`0x800` for the first (vertical, and which way), and
/// `0x400`/`0x200` for the second. `Roads::clear_support@0088e3f0` reads
/// them back to take one reference off each.
const SUPPORT_CORNER_A: u16 = 0x8000;
const SUPPORT_CORNER_B: u16 = 0x4000;

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
    /// `+0x0` `element_num == Roads +0xdc`, the C4 corner piece `set_diags`'
    /// tail writes on a neighbour beside a corner (§12). The sweep spares
    /// it as it spares a made tile. Only the piece's identity is carried:
    /// the texture passes that re-piece a road tile are not modelled, so a
    /// tile that is road reads as re-pieced ([`RedoView::finish_pieces`]).
    pub c4: bool,
    /// `+0x4`'s low byte, the **intersections** (item 4677,
    /// `docs/RENDERER.md` §6.5): `RoadsOut::mark_intersections@00892390`
    /// sets them, `mark_splits@00891d40` clears a joined neighbour's, and
    /// the redo pass's `flags = 0` clears the tile's own. Texture only:
    /// nothing in this crate reads it, and the door carries it to the
    /// client, which keeps it apart from [`Elem::flags`] so no reader of
    /// the connections can see it.
    pub inter: u8,
    /// [`Elem::inter`] as `RoadsOut::find_piece@00890c90` last read it
    /// (item 4682, `docs/RENDERER.md` §6.7). The record's piece is picked
    /// there, in the redo's fourth pass and in `add_roads`' own, and only
    /// there; a neighbour's `mark_splits` that zeroes [`Elem::inter`] later
    /// leaves the piece as it was. Texture only, like [`Elem::inter`].
    pub picked: u8,
    /// `+0x4`'s second and third bytes, the **white and yellow lines**
    /// (`0x0000_ff00` and `0x00ff_0000`, item 4687, `docs/RENDERER.md`
    /// §6.8), in place: `mark_white_lines@00892c10`,
    /// `mark_yellow_lines@00892550`, `fixup_lines@00891210` and
    /// `delete_straglers@008910b0` write them, and the redo pass's
    /// `flags = 0` clears them. Texture only, like [`Elem::inter`].
    pub lines: u32,
    /// The cardinals `mark_splits@00891d40` joined this tile in, beyond
    /// [`Elem::flags`] (item 4687). The original writes them into the
    /// element, where the tile's own later texture passes read them; here
    /// they are kept apart so no reader of the connections sees them, and
    /// the redo pass's `flags = 0` clears them. Texture only.
    pub joins: u32,
}

/// The line bytes of `+0x4` ([`Elem::lines`]).
pub const LINES: u32 = 0x00ff_ff00;

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
    mods: ModQueue,
    /// `Roads::add_roads`' function-static `remove_list`.
    remove: ModQueue,
    /// `changed_roads` (`+0x150`) and `changed_roads2` (`+0x16c`).
    redo: [Vec<i32>; 2],
    /// The line passes' pushes (item 4687, `docs/RENDERER.md` §6.8): in the
    /// original they go onto `changed_roads`/`changed_roads2` with the rest.
    /// Here a tile on one of these and not on the matching [`RoadMesh::redo`]
    /// list is redone for its texture alone ([`Sim::texture_redo`]), so
    /// nothing the simulation reads is written (parked 4687a).
    tex: [Vec<i32>; 2],
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

    /// Every element, by the original's packed index `x + width_in_tiles ·
    /// y`, in index order: what the read door copies (item 4199,
    /// `crate::observe::RoadsScene`).
    pub fn elems(&self) -> impl Iterator<Item = (i32, &Elem)> {
        self.elems.iter().map(|(&i, e)| (i, e))
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

    /// `ArrayBase<int>::add_unique` from a texture pass: the same list the
    /// original names, kept in [`RoadMesh::tex`].
    fn tex_push(&mut self, index: i32, list: i32) {
        if list == 0 {
            return;
        }
        let l = &mut self.tex[usize::from(list != 1)];
        if !l.contains(&index) {
            l.push(index);
        }
    }

    /// The cursor's own line bits: `set` raised, then `clear` dropped.
    fn own_lines(&mut self, world: &World, set: u32, clear: u32) {
        if let Some(e) = self.elems.get_mut(&index_of(world, self.at)) {
            e.lines = (e.lines | set) & !clear;
        }
    }

    fn lines_of(&self, world: &World, t: Pos) -> u32 {
        self.elems.get(&index_of(world, t)).map_or(0, |e| e.lines)
    }

    /// `RoadsOut::set_neighbor@00890c30` on the line bits, with the push its
    /// callers make: the neighbour at the offset, if it has an element and
    /// holds none of `set`, takes `set`, drops `clear`, and goes on the
    /// list. Answers whether it did.
    fn line_neighbour(
        &mut self,
        world: &World,
        dx: i32,
        dy: i32,
        set: u32,
        clear: u32,
        list: i32,
    ) -> bool {
        let i = index_of(world, Pos::new(self.at.x + dx, self.at.y + dy));
        let Some(e) = self.elems.get_mut(&i) else {
            return false;
        };
        if e.lines & set != 0 {
            return false;
        }
        e.lines = (e.lines | set) & !clear;
        self.tex_push(i, list);
        true
    }

    /// One line: the cursor's `own` raised, and `nb` on the neighbour at the
    /// offset, pushed when it is fresh — the shape of every site in
    /// `mark_white_lines` and `fixup_lines`.
    fn line(&mut self, world: &World, own: u32, (dx, dy): (i32, i32), nb: u32, list: i32) {
        self.own_lines(world, own, 0);
        self.line_neighbour(world, dx, dy, nb, 0, list);
    }

    /// **`RoadsOut::mark_white_lines@00892c10`** (item 4687): the white edge
    /// lines along each claimed run, on the side with no road, and round a
    /// corner whose diagonal is a building footprint. `g` is the tile's
    /// connections with `mark_splits`' joins.
    fn white_lines(&mut self, world: &World, g: u32, list: i32) {
        use dir::{E, N, S, W};
        let (road, blocked) = (self.road, self.blocked);
        if g & (N | S) != 0 {
            if !road[4] {
                if road[2] {
                    self.line(world, 0x4000, (0, -1), 0x800, list);
                }
                if road[6] {
                    self.line(world, 0x800, (0, 1), 0x4000, list);
                }
            }
            if !road[8] {
                if road[2] {
                    self.line(world, 0x8000, (0, -1), 0x400, list);
                }
                if road[6] {
                    self.line(world, 0x400, (0, 1), 0x8000, list);
                }
            }
        }
        if g & (E | W) != 0 {
            if !road[2] {
                if road[4] {
                    self.line(world, 0x2000, (1, 0), 0x100, list);
                }
                if road[8] {
                    self.line(world, 0x100, (-1, 0), 0x2000, list);
                }
            }
            if !road[6] {
                if road[4] {
                    self.line(world, 0x1000, (1, 0), 0x200, list);
                }
                if road[8] {
                    self.line(world, 0x200, (-1, 0), 0x1000, list);
                }
            }
        }
        if g & N != 0 && g & W != 0 && blocked[1] {
            self.line(world, 0x8000, (0, -1), 0x400, list);
            self.line(world, 0x100, (-1, 0), 0x2000, list);
        }
        if g & N != 0 && g & E != 0 && blocked[3] {
            self.line(world, 0x4000, (0, -1), 0x800, list);
            self.line(world, 0x2000, (1, 0), 0x100, list);
        }
        if g & S != 0 && g & W != 0 && blocked[7] {
            self.line(world, 0x400, (0, 1), 0x8000, list);
            self.line(world, 0x200, (-1, 0), 0x1000, list);
        }
        if g & S != 0 && g & E != 0 && blocked[5] {
            self.line(world, 0x800, (0, 1), 0x4000, list);
            self.line(world, 0x1000, (1, 0), 0x200, list);
        }
    }

    /// **`RoadsOut::mark_yellow_lines@00892550`** (item 4687): a straight
    /// run beside a road that does not join it takes the yellow line in
    /// place of the white on that side, and so do its neighbours along the
    /// run; the side road goes on the list when either took one. A road
    /// beside it with no element drops the tile's claim toward it and ends
    /// the pass, which here only `g` sees (the answer).
    fn yellow_lines(&mut self, world: &World, mut g: u32, list: i32) -> u32 {
        use dir::{E, N, S, W};
        let road = self.road;
        let at = self.at;
        // (the side, its road cache, the claim the side road must lack, the
        // two along-run arms: (cache, own set, own clear, offset, nb set,
        // nb clear)).
        type Arm = (usize, u32, u32, (i32, i32), u32, u32);
        let side = |m: &mut RoadMesh,
                    g: &mut u32,
                    k: usize,
                    own: u32,
                    back: u32,
                    (dx, dy): (i32, i32),
                    arms: [Arm; 2]|
         -> bool {
            if !road[k] {
                return true;
            }
            let Some(n) = m.elem(world, Pos::new(at.x + dx, at.y + dy)) else {
                *g &= !own;
                return false;
            };
            if (n.flags | n.joins) & back != 0 {
                return true;
            }
            let mut fresh = false;
            for (c, os, oc, (ax, ay), ns, nc) in arms {
                if road[c] {
                    m.own_lines(world, os, oc);
                    fresh |= m.line_neighbour(world, ax, ay, ns, nc, list);
                }
            }
            if fresh {
                m.tex_push(index_of(world, Pos::new(at.x + dx, at.y + dy)), list);
            }
            true
        };
        if g & (N | S) != 0 && g & (E | W) == 0 {
            if !side(
                self,
                &mut g,
                4,
                E,
                W,
                (1, 0),
                [
                    (2, 0x40_0000, 0x4000, (0, -1), 0x8_0000, 0x800),
                    (6, 0x8_0000, 0x800, (0, 1), 0x40_0000, 0x4000),
                ],
            ) {
                return g;
            }
            side(
                self,
                &mut g,
                8,
                W,
                E,
                (-1, 0),
                [
                    (2, 0x80_0000, 0x8000, (0, -1), 0x4_0000, 0x400),
                    (6, 0x4_0000, 0x400, (0, 1), 0x80_0000, 0x8000),
                ],
            );
            return g;
        }
        if g & (E | W) == 0 || g & (N | S) != 0 {
            return g;
        }
        if !side(
            self,
            &mut g,
            2,
            N,
            S,
            (0, -1),
            [
                (4, 0x20_0000, 0x2000, (1, 0), 0x1_0000, 0x100),
                (8, 0x1_0000, 0x100, (-1, 0), 0x20_0000, 0x2000),
            ],
        ) {
            return g;
        }
        side(
            self,
            &mut g,
            6,
            S,
            N,
            (0, 1),
            [
                (4, 0x10_0000, 0x1000, (1, 0), 0x2_0000, 0x200),
                (8, 0x2_0000, 0x200, (-1, 0), 0x10_0000, 0x1000),
            ],
        );
        g
    }

    /// The cursor's connections as its texture passes read them: the
    /// element's own and `mark_splits`' joins ([`Elem::joins`]).
    fn tex_dirs(&self, world: &World) -> u32 {
        self.elems
            .get(&index_of(world, self.at))
            .map_or(0, |e| e.flags | e.joins)
    }

    /// **`RoadsOut::fixup_lines@00891210`** (item 4687): a white line under
    /// a yellow goes, and a line that reaches a claimed corner is carried
    /// round it onto the neighbour — eight white sites, then eight yellow.
    fn fixup_lines(&mut self, world: &World, list: i32) {
        use dir::{E, N, S, W};
        type Turn = (u32, u32, u32, (i32, i32), u32);
        let g = self.tex_dirs(world);
        // (the yellow over it, the white it drops).
        let drops = [
            (0x80_0000, 0x8000),
            (0x40_0000, 0x4000),
            (0x20_0000, 0x2000),
            (0x10_0000, 0x1000),
            (0x8_0000, 0x800),
            (0x4_0000, 0x400),
            (0x2_0000, 0x200),
            (0x1_0000, 0x100),
        ];
        // (the line, the claim, the line it turns into, the neighbour, its bit).
        let turns: [Turn; 8] = [
            (0x8000, W, 0x100, (-1, 0), 0x2000),
            (0x4000, E, 0x2000, (1, 0), 0x100),
            (0x2000, N, 0x4000, (0, -1), 0x800),
            (0x1000, S, 0x800, (0, 1), 0x4000),
            (0x800, E, 0x1000, (1, 0), 0x200),
            (0x400, W, 0x200, (-1, 0), 0x1000),
            (0x200, S, 0x400, (0, 1), 0x8000),
            (0x100, N, 0x8000, (0, -1), 0x400),
        ];
        let turn = |m: &mut RoadMesh, (have, claim, to, off, nb): Turn| {
            let l = m.lines_of(world, m.at);
            if l & have != 0 && g & claim != 0 && l & to == 0 {
                m.line(world, to, off, nb, list);
            }
        };
        for (k, (yellow, white)) in drops.into_iter().enumerate() {
            let l = self.lines_of(world, self.at);
            if l & (yellow | white) == yellow | white {
                self.own_lines(world, 0, white);
            }
            turn(self, turns[k]);
        }
        for t in [
            (0x80_0000, W, 0x1_0000, (-1, 0), 0x20_0000),
            (0x40_0000, E, 0x20_0000, (1, 0), 0x1_0000),
            (0x20_0000, N, 0x40_0000, (0, -1), 0x8_0000),
            (0x10_0000, S, 0x8_0000, (0, 1), 0x40_0000),
            (0x8_0000, E, 0x10_0000, (1, 0), 0x2_0000),
            (0x4_0000, W, 0x2_0000, (-1, 0), 0x10_0000),
            (0x2_0000, S, 0x4_0000, (0, 1), 0x80_0000),
            (0x1_0000, N, 0x80_0000, (0, -1), 0x4_0000),
        ] {
            turn(self, t);
        }
    }

    /// **`RoadsOut::delete_straglers@008910b0`** (item 4687): a line pair
    /// along a cardinal the tile does not claim goes, and the tile itself
    /// goes on the list.
    fn delete_straglers(&mut self, world: &World, list: i32) {
        let g = self.tex_dirs(world);
        for (any, claim) in [
            (0x00c0_c000, dir::N),
            (0x0030_3000, dir::E),
            (0x000c_0c00, dir::S),
            (0x0003_0300, dir::W),
        ] {
            if self.lines_of(world, self.at) & any != 0 && g & claim == 0 {
                self.own_lines(world, 0, any);
                self.tex_push(index_of(world, self.at), list);
            }
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
        self.leech_lines(world);
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

    /// **`leech_codes`' line half** (item 4687, `docs/RENDERER.md` §6.8):
    /// each cardinal road neighbour lends this tile the lines that run on
    /// into it — a north neighbour's west-side line going south (`0x400`)
    /// is this tile's going north (`0x8000`), and so on, white and yellow.
    /// The same walk as the claims, ending at the same neighbour with no
    /// element; texture only, so it reads and writes [`Elem::lines`] alone.
    /// This is what holds a joined corner's ring (`client::roads`'
    /// `corner_loops`) through a redo's `flags = 0`.
    fn leech_lines(&mut self, world: &World) {
        let at = self.at;
        let maps: [[(u32, u32); 4]; 4] = [
            [
                (0x400, 0x8000),
                (0x800, 0x4000),
                (0x4_0000, 0x80_0000),
                (0x8_0000, 0x40_0000),
            ],
            [
                (0x8000, 0x400),
                (0x4000, 0x800),
                (0x80_0000, 0x4_0000),
                (0x40_0000, 0x8_0000),
            ],
            [
                (0x100, 0x2000),
                (0x200, 0x1000),
                (0x1_0000, 0x20_0000),
                (0x2_0000, 0x10_0000),
            ],
            [
                (0x2000, 0x100),
                (0x1000, 0x200),
                (0x20_0000, 0x1_0000),
                (0x10_0000, 0x2_0000),
            ],
        ];
        for ((d, _, _, dx, dy), map) in CARDINALS.into_iter().zip(maps) {
            if !self.road[d] {
                continue;
            }
            let Some(n) = self.elem(world, Pos::new(at.x + dx, at.y + dy)) else {
                return;
            };
            let got = map
                .iter()
                .filter(|(from, _)| n.lines & from != 0)
                .fold(0, |a, (_, to)| a | to);
            self.own_lines(world, got, 0);
        }
    }

    /// **The pick's intersections** (item 4682, `docs/RENDERER.md` §6.7):
    /// `find_piece` at `t` reads the element's flags whole, so the piece is
    /// the one [`Elem::inter`] names now, until the tile is picked again.
    fn note_pick(&mut self, world: &World, t: Pos) {
        if let Some(e) = self.elems.get_mut(&index_of(world, t)) {
            e.picked = e.inter;
        }
    }

    /// **The intersections, as the texture marks leave them** (item 4677,
    /// `docs/RENDERER.md` §6.5): `RoadsOut::mark_splits@00891d40`'s clears of
    /// a neighbour's intersection byte, then
    /// `RoadsOut::mark_intersections@00892390`, on the tile at the cursor
    /// with the cache refilled. `mark_intersections` only ever sets a bit,
    /// and the redo pass clears a tile's flags only when the tile is on its
    /// list, so a bit set while a run was being laid outlives the run's end.
    ///
    /// `mark_splits` also joins the tile to a further road neighbour where
    /// it claims a corner pair (`docs/ROADS.md` §9.4). Those connections are
    /// not written: this pass changes nothing but [`Elem::inter`]. The tile's
    /// own joins are taken into a local copy, because the original's later
    /// arms and `mark_intersections` read them back. A neighbour with no
    /// element ends either function where the original returns.
    ///
    /// Item 4687 runs the two line passes between them, in the original's
    /// order (`mark_white_lines`, `mark_yellow_lines`), on the same joins,
    /// and keeps the joins in [`Elem::joins`] for the passes after.
    fn mark_intersections(&mut self, world: &World, list: i32) {
        let at = self.at;
        let mut g = self.flags_at(world, at);
        let road = self.road;
        let mut clear = |dx: i32, dy: i32| -> bool {
            let i = index_of(world, Pos::new(at.x + dx, at.y + dy));
            match self.elems.get_mut(&i) {
                Some(e) => {
                    e.inter = 0;
                    true
                }
                None => false,
            }
        };
        // mark_splits: each arm joins (here, locally) and clears the joined
        // neighbour's intersections; `false` is the original's return.
        let mut splits = || {
            use dir::{E, N, S, W};
            if g & (N | W) == N | W || g & (N | E) == N | E {
                let west = g & (N | E) == N | E && g & (N | W) != N | W;
                if road[6] {
                    g |= S;
                    if !clear(0, 1) {
                        return;
                    }
                }
                let (k, side, dx) = if west { (8, W, -1) } else { (4, E, 1) };
                if road[k] {
                    g |= side;
                    clear(dx, 0);
                }
                return;
            }
            if g & S != 0 && g & W != 0 {
                if road[2] {
                    g |= N;
                    if !clear(0, -1) {
                        return;
                    }
                }
                if road[4] {
                    g |= E;
                    if !clear(1, 0) {
                        return;
                    }
                }
            }
            if g & S == 0 || g & E == 0 {
                return;
            }
            if road[2] {
                g |= N;
                if !clear(0, -1) {
                    return;
                }
            }
            if road[8] {
                g |= W;
                clear(-1, 0);
            }
        };
        splits();
        self.white_lines(world, g, list);
        let g = self.yellow_lines(world, g, list);
        let both = |t: Pos, pair: u32| -> Option<bool> {
            self.elem(world, t).map(|n| n.flags & pair == pair)
        };
        let mut inter = 0u8;
        'marks: for (pair, (fwd, fb), (back, bb)) in [
            (dir::E | dir::W, ((1, 0), 0x10u8), ((-1, 0), 0x01u8)),
            (dir::N | dir::S, ((0, -1), 0x40), ((0, 1), 0x04)),
        ] {
            if g & pair != pair {
                continue;
            }
            for ((dx, dy), b) in [(fwd, fb), (back, bb)] {
                match both(Pos::new(at.x + dx, at.y + dy), pair) {
                    None => break 'marks,
                    Some(false) => inter |= b,
                    Some(true) => {}
                }
            }
        }
        if let Some(e) = self.elems.get_mut(&index_of(world, at)) {
            e.inter |= inter;
            e.joins |= g & !e.flags & dir::ORTHOG;
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
        let elem = self.movement.mesh.elem(&self.world, t);
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
            self.movement.mesh.sweep_road[k] = road;
            build[k] = m & tile::OBJECT == tile::OBJECT_BUILDING;
            water[k] = m & tile::RIVER != 0 || m & tile::SURFACE == tile::SURFACE_OCEAN;
            // The **centre's** element, read at the centre's own slot: does
            // the tile claim the direction whose neighbour is a road.
            sup[k] = road && elem.is_some_and(|e| e.flags & COMPASS_FLAGS[k] != 0);
        }
        let road = self.movement.mesh.sweep_road;
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
            Some(e) if e.made || e.c4 => {
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
        self.movement.mesh.road_changed(&self.world, t, false, 0, 1);
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
        self.movement.mesh.mods.push(index, 0x100);
        self.add_roads(0);
    }

    /// `Roads::road_added@008954d0` — queue the tile, and run the mesh when
    /// asked to.
    fn road_added(&mut self, t: Pos, p3: i32, p4: i32) {
        if p3 != 0 {
            self.movement.mesh.road_changed(&self.world, t, true, 1, 0);
        }
        let index = index_of(&self.world, t);
        self.movement.mesh.mods.push(index, 0x101);
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
        add_roads_steps(&mut SimRoads { sim: self }, param);
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
        let at = self.movement.mesh.at;
        if self.movement.mesh.orthog_connects(&self.world, at) > 1
            || !self
                .movement
                .mesh
                .get_road_data(&self.world, at, false, 0, false)
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
            let f = self.movement.mesh.flags_at(&self.world, at);
            let barred = (f & dir::N != 0 && (i == 1 || i == 2))
                || (f & dir::E != 0 && (i == 2 || i == 3))
                || (f & dir::S != 0 && (i == 3 || i == 4))
                || (f & dir::W != 0 && (i == 4 || i == 1));
            if barred || !self.movement.mesh.road[tables::DIAG[i]] {
                continue;
            }
            if self.movement.mesh.orthog_connects(&self.world, corner) < 2 {
                joined = 1;
                if f & tables::BLOCK_A[i] == 0 && f & tables::BLOCK_B[i] == 0 {
                    self.movement
                        .mesh
                        .set_flags(&self.world, at, f | tables::OWN[i]);
                    self.movement
                        .mesh
                        .join(&self.world, corner, tables::OPPOSITE[i], list);
                }
                continue;
            }
            // The corner is a road junction in its own right. Fill the gap
            // between the two — the *horizontal* neighbour, whichever way
            // the corner lies — if neither tile between them is a road or a
            // footprint.
            let (fa, fb) = (tables::FLANK_A[i], tables::FLANK_B[i]);
            if self.movement.mesh.road[fa]
                || self.movement.mesh.blocked[fa]
                || self.movement.mesh.road[fb]
                || self.movement.mesh.blocked[fb]
            {
                continue;
            }
            let made = Pos::new(at.x + CORNER_X[i], at.y);
            self.world_set_road_at(made, true, 0, 1);
            self.road_added(made, 0, 0);
            self.movement
                .mesh
                .road_changed(&self.world, made, true, 0, 0);
            self.movement
                .mesh
                .add_unique(index_of(&self.world, made), list);
            if !self
                .movement
                .mesh
                .get_road_data(&self.world, made, false, 0, false)
            {
                return 0;
            }
            let i_made = index_of(&self.world, made);
            self.movement.mesh.elems.get_mut(&i_made).unwrap().made = true;
            let i_at = index_of(&self.world, at);
            let e = self.movement.mesh.elems.get_mut(&i_at).unwrap();
            e.support |= SUPPORT_MADE;
            if CORNER_X[i] == 1 {
                e.support |= SUPPORT_EAST;
            }
            self.movement.mesh.add_unique(i_at, list);
            self.movement
                .mesh
                .add_unique(index_of(&self.world, corner), list);
        }
        if joined != 0 {
            self.corner_pieces(_param);
        }
        joined
    }

    /// **`set_diags`' tail** (`0088ee5a`–`0088f2c3`, item 3287,
    /// `docs/ROADS.md` §12): when a corner was joined, each pair of claims
    /// that meets at a corner — a cardinal and a diagonal on the far side of
    /// it, or two opposite diagonals — puts the C4 piece on two of the
    /// tile's cardinal neighbours, through `TerrainOut::get_road_data(x, y,
    /// 1, param, 1)`: an element is made if there is none, and one more
    /// reference is taken either way (`pending_camel_steps` under a
    /// caravan's `param`, `ref_count` otherwise). The tile notes each pair
    /// in `support_codes` for [`Sim::clear_support`] to give back.
    ///
    /// The references are what reach the world. A C4 element on a tile a
    /// building's ring lays later makes `add_roads`' count two, so the add
    /// becomes a removal and `clear_roads` re-queues every cardinal road
    /// neighbour: Great Lakes (Iroquois)' caravan diagonal at (235, 153)
    /// re-derived to N and SW when `1/2027`'s ring reached (235, 152) on
    /// 10485, where this crate left it claiming NW alone and the sweep took
    /// it on 10620 (run946's packet holds the C4 elements and the codes).
    fn corner_pieces(&mut self, param: i32) {
        use dir::{E, N, NE, NW, S, SE, SW, W};
        let at = self.movement.mesh.at;
        let (x, y) = (at.x, at.y);
        // `(first, second, bits)` per pair, in the listing's order; each
        // block re-reads the tile's flags, which none of them writes.
        type Pair = (u32, u32, (i32, i32), (i32, i32), u16);
        let pairs: [Pair; 10] = [
            (W, SE, (0, 1), (1, 0), 0x1000 | 0x200),
            (W, NE, (0, -1), (1, 0), 0x1800 | 0x200),
            (N, SW, (-1, 0), (0, 1), 0x400),
            (N, SE, (1, 0), (0, 1), 0x800 | 0x400),
            (E, NW, (0, -1), (-1, 0), 0x1800),
            (E, SW, (0, 1), (-1, 0), 0x1000),
            (S, NE, (1, 0), (0, -1), 0x800 | 0x600),
            (S, NW, (-1, 0), (0, -1), 0x600),
            (NW, SE, (-1, 0), (1, 0), 0x200),
            (NE, SW, (-1, 0), (1, 0), 0x200),
        ];
        let (tw, th) = (self.world.width() * 4, self.world.height() * 4);
        for (a, b, first, second, bits) in pairs {
            let f = self.movement.mesh.flags_at(&self.world, at);
            if f & a == 0 || f & b == 0 {
                continue;
            }
            for (dx, dy) in [first, second] {
                let n = Pos::new(x + dx, y + dy);
                if n.x < 0 || n.y < 0 || n.x >= tw || n.y >= th {
                    continue;
                }
                self.movement
                    .mesh
                    .get_road_data(&self.world, n, true, param, true);
                if let Some(e) = self.movement.mesh.elems.get_mut(&index_of(&self.world, n)) {
                    e.c4 = true;
                }
            }
            if let Some(e) = self.movement.mesh.elems.get_mut(&index_of(&self.world, at)) {
                e.support |= SUPPORT_CORNER_A | SUPPORT_CORNER_B | bits;
            }
        }
    }

    /// `Roads::clear_roads@0088fef0` — a queued *removal*. The road tile only
    /// goes if the mesh laid it and nothing else still holds its element.
    fn clear_roads(&mut self, index: i32) {
        let at = tile_of(&self.world, index);
        self.movement.mesh.at = at;
        let mine = self.clear_support(at, 0);
        self.movement.mesh.fill_cache(&self.world, at);
        let left = self
            .movement
            .mesh
            .road_changed(&self.world, at, false, 0, 1);
        if mine && left == 0 {
            self.world_set_road_at(at, false, 0, 1);
        }
        for (d, _, _, dx, dy) in CARDINALS {
            if self.movement.mesh.road[d] {
                let n = Pos::new(at.x + dx, at.y + dy);
                self.movement.mesh.add_unique(index_of(&self.world, n), 1);
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
        let Some(e) = self.movement.mesh.elem(&self.world, at) else {
            return false;
        };
        let mine = e.made;
        let sup = e.support;
        if sup & 0xe000 == 0 {
            self.set_support(at, 0);
            return mine;
        }
        // The two corner notes `set_diags`' tail leaves (§12): one reference
        // off each C4 neighbour. The first arm leaves the codes standing;
        // the second clears them.
        if sup & SUPPORT_CORNER_A != 0 {
            let d = if sup & 0x1000 == 0 {
                (if sup & 0x800 != 0 { 1 } else { -1 }, 0)
            } else {
                (0, if sup & 0x800 != 0 { -1 } else { 1 })
            };
            self.release_support(Pos::new(at.x + d.0, at.y + d.1));
        }
        if sup & SUPPORT_CORNER_B != 0 {
            let d = if sup & 0x400 == 0 {
                (if sup & 0x200 != 0 { 1 } else { -1 }, 0)
            } else {
                (0, if sup & 0x200 != 0 { -1 } else { 1 })
            };
            self.set_support(at, 0);
            self.release_support(Pos::new(at.x + d.0, at.y + d.1));
        }
        if keep == 0 && sup & SUPPORT_MADE != 0 {
            let d = if sup & 0x100 == 0 {
                (if sup & SUPPORT_EAST != 0 { 1 } else { -1 }, 0)
            } else {
                (0, if sup & SUPPORT_EAST != 0 { -1 } else { 1 })
            };
            self.set_support(at, 0);
            self.release_support(Pos::new(at.x + d.0, at.y + d.1));
        }
        mine
    }

    fn set_support(&mut self, at: Pos, v: u16) {
        if let Some(e) = self.movement.mesh.elems.get_mut(&index_of(&self.world, at)) {
            e.support = v;
        }
    }

    /// One of `clear_support`'s three releases: a reference off the
    /// neighbour, and its road with it when that was the last and the mesh
    /// had laid it.
    fn release_support(&mut self, n: Pos) {
        if let Some(e) = self.movement.mesh.elem(&self.world, n) {
            let was = e.made;
            let left = self.movement.mesh.road_changed(&self.world, n, false, 0, 1);
            if left == 0 && was {
                self.world_set_road_at(n, false, 0, 1);
            }
        }
    }

    /// `Roads::redo_changed_roads@0088e670` — the passes over one of the two
    /// lists; see [`redo_changed_roads_steps`], which this runs over the
    /// list `which` names.
    ///
    /// Then the tiles only the line passes put on that list (item 4687),
    /// for their texture alone ([`Sim::texture_redo`]).
    fn redo_changed_roads(&mut self, which: usize, list: i32, param: i32) {
        let work = self.movement.mesh.redo[which].clone();
        let tex: Vec<i32> = (self.movement.mesh.tex[which].iter())
            .filter(|i| !work.contains(i))
            .copied()
            .collect();
        redo_changed_roads_steps(&mut SimRedo { sim: self }, &work, list, param);
        self.texture_redo(&tex, list);
    }

    /// **The redo of a tile the line passes pushed** (item 4687,
    /// `docs/RENDERER.md` §6.8), texture only: the original runs
    /// `redo_changed_roads@0088e670`'s four passes on it as on any listed
    /// tile, and those rewrite its connections (parked 4687a). Here the
    /// tile's line bytes, intersections and joins are cleared and
    /// re-marked on its connections as they stand, then
    /// `fixup_lines`, `delete_straglers` and the pick. `set_diags` is not
    /// run, and the marks are taken as its zero answer would take them.
    /// The cursor and the caches are restored after, so nothing the
    /// simulation reads is written.
    fn texture_redo(&mut self, tiles: &[i32], list: i32) {
        if tiles.is_empty() {
            return;
        }
        let world = &self.world;
        let m = &mut self.movement.mesh;
        let saved = (m.at, m.blocked, m.road);
        let has = |m: &RoadMesh, i: i32| m.elems.contains_key(&i);
        for &i in tiles {
            if let Some(e) = m.elems.get_mut(&i) {
                e.inter = 0;
                e.lines = 0;
                e.joins = 0;
            }
        }
        for &i in tiles {
            if !has(m, i) {
                continue;
            }
            m.at = tile_of(world, i);
            m.fill_cache(world, m.at);
            m.leech_lines(world);
            m.mark_intersections(world, list);
            m.fixup_lines(world, list);
        }
        for &i in tiles {
            if !has(m, i) {
                continue;
            }
            m.at = tile_of(world, i);
            m.fixup_lines(world, list);
            m.delete_straglers(world, list);
            m.note_pick(world, m.at);
        }
        (m.at, m.blocked, m.road) = saved;
    }
}

/// What `Roads::redo_changed_roads@0088e670` asks of the road mesh (item
/// 3268): the steps read and write the roads only through this, so the
/// sweep's port is a view over the emulated original's windows and the
/// crate's is [`SimRedo`]. The cursor is the original's `Roads +0x120` /
/// `+0x124` (the tile `tile` names).
pub trait RedoView {
    /// The cursor goes to the tile whose packed index is `index`.
    fn tile(&mut self, index: i32);
    /// `Roads::clear_support(x, y, 1)`.
    fn clear_support(&mut self);
    /// `TerrainOut::get_road_data(x, y, 0, 0, 0)` — whether the tile has an
    /// element; the cursor's element is the answer's.
    fn road_data(&mut self) -> bool;
    /// The element's `flags = 0`.
    fn clear_flags(&mut self);
    /// `RoadsOut::fill_cache(x, y)`.
    fn fill_cache(&mut self);
    /// `RoadsOut::mark_and_trim_directions(list)`.
    fn trim_directions(&mut self, list: i32);
    /// `RoadsOut::leech_codes`.
    fn leech_codes(&mut self);
    /// `Roads::set_diags(list, param)`; zero sends the original on to the
    /// texture marks.
    fn set_diags(&mut self, list: i32, param: i32) -> i32;
    /// The cache refilled for the texture marks (rendering only).
    fn refill_for_marks(&mut self);
    /// `mark_splits`, `mark_white_lines`, `mark_yellow_lines`,
    /// `mark_intersections`, `fixup_lines` — texture picking, which this
    /// crate does not have (`docs/ROADS.md` §9.4).
    fn texture_marks(&mut self, list: i32);
    /// The fourth pass: `fixup_lines`, `delete_straglers`, `find_piece` —
    /// rendering only.
    fn finish_pieces(&mut self, list: i32);
}

/// **`Roads::redo_changed_roads@0088e670`'s steps** (item 3268): four passes
/// over `work`, the tiles of one change list in order — drop the support
/// links; clear each element's flags and re-derive its directions; take each
/// neighbour's answering claim and run `set_diags` (a zero answer goes on to
/// the texture marks); and the fourth pass, texture only. A tile with no
/// element is skipped by the last three.
pub fn redo_changed_roads_steps<V: RedoView>(v: &mut V, work: &[i32], list: i32, param: i32) {
    for &index in work {
        v.tile(index);
        v.clear_support();
    }
    for &index in work {
        v.tile(index);
        if !v.road_data() {
            continue;
        }
        v.clear_flags();
        v.fill_cache();
        v.trim_directions(list);
    }
    for &index in work {
        v.tile(index);
        if !v.road_data() {
            continue;
        }
        v.fill_cache();
        v.leech_codes();
        if v.set_diags(list, param) == 0 {
            v.refill_for_marks();
            v.texture_marks(list);
        }
    }
    for &index in work {
        v.tile(index);
        if !v.road_data() {
            continue;
        }
        v.finish_pieces(list);
    }
}

/// [`RedoView`] over the simulation: the mesh's own passes.
struct SimRedo<'a> {
    sim: &'a mut Sim,
}

impl RedoView for SimRedo<'_> {
    fn tile(&mut self, index: i32) {
        self.sim.movement.mesh.at = tile_of(&self.sim.world, index);
    }
    fn clear_support(&mut self) {
        let at = self.sim.movement.mesh.at;
        self.sim.clear_support(at, 1);
    }
    fn road_data(&mut self) -> bool {
        let at = self.sim.movement.mesh.at;
        self.sim
            .movement
            .mesh
            .get_road_data(&self.sim.world, at, false, 0, false)
    }
    fn clear_flags(&mut self) {
        let at = self.sim.movement.mesh.at;
        self.sim.movement.mesh.set_flags(&self.sim.world, at, 0);
        if let Some(e) = self
            .sim
            .movement
            .mesh
            .elems
            .get_mut(&index_of(&self.sim.world, at))
        {
            e.inter = 0;
            e.lines = 0;
            e.joins = 0;
        }
    }
    fn fill_cache(&mut self) {
        let at = self.sim.movement.mesh.at;
        self.sim.movement.mesh.fill_cache(&self.sim.world, at);
    }
    fn trim_directions(&mut self, list: i32) {
        self.sim
            .movement
            .mesh
            .trim_directions(&self.sim.world, list);
    }
    fn leech_codes(&mut self) {
        self.sim.movement.mesh.leech_codes(&self.sim.world);
    }
    fn set_diags(&mut self, list: i32, param: i32) -> i32 {
        self.sim.set_diags(list, param)
    }
    fn refill_for_marks(&mut self) {
        let at = self.sim.movement.mesh.at;
        self.sim.movement.mesh.fill_cache(&self.sim.world, at);
    }
    fn texture_marks(&mut self, list: i32) {
        // Items 4677 and 4687: the splits' clears, the lines, the
        // intersections and `fixup_lines`, texture only; the line passes'
        // pushes go to the texture lists (`RoadMesh::tex`).
        let m = &mut self.sim.movement.mesh;
        m.mark_intersections(&self.sim.world, list);
        m.fixup_lines(&self.sim.world, list);
    }
    fn finish_pieces(&mut self, list: i32) {
        // `fixup_lines` and `delete_straglers` on the lines (item 4687);
        // `find_piece` picks the road tile's own piece, so the C4 a corner
        // left on it goes (§12; the pick itself is the client's, on the
        // intersections it reads here: item 4682).
        let at = self.sim.movement.mesh.at;
        self.sim.movement.mesh.fixup_lines(&self.sim.world, list);
        self.sim
            .movement
            .mesh
            .delete_straglers(&self.sim.world, list);
        self.sim.movement.mesh.note_pick(&self.sim.world, at);
        if self.sim.world.tile_mask(at) & tile::SURFACE == tile::SURFACE_ROAD
            && let Some(e) = self
                .sim
                .movement
                .mesh
                .elems
                .get_mut(&index_of(&self.sim.world, at))
        {
            e.c4 = false;
        }
    }
}

/// [`RoadsView`] over the simulation: the mesh and the world as they stand.
/// The terrain's cell patches and the world's per-tile flag word are not
/// carried, so the ambience scan the add arm runs under a non-zero `param`
/// finds no cell with tile records and does nothing — which is all it could
/// do here, its two writes being an audio-and-graphics event and a mark on a
/// terrain cell that only the scan itself reads.
struct SimRoads<'a> {
    sim: &'a mut Sim,
}

impl RoadsView for SimRoads<'_> {
    fn queue(&mut self) -> &mut ModQueue {
        &mut self.sim.movement.mesh.mods
    }
    fn remove_list(&mut self) -> &mut ModQueue {
        &mut self.sim.movement.mesh.remove
    }
    fn grow(&mut self, _which: Which, _step: i16) {
        // Both arrays are `Vec`s, whose capacity `ModQueue` reports as
        // unbounded, so no push ever asks.
    }
    fn world_cells(&self) -> (i32, i32) {
        (self.sim.world.width(), self.sim.world.height())
    }
    fn world_tiles(&self) -> (i32, i32) {
        (self.sim.world.width() * 4, self.sim.world.height() * 4)
    }
    fn set_tile(&mut self, x: i32, y: i32) {
        self.sim.movement.mesh.at = Pos::new(x, y);
    }
    fn clear_roads(&mut self, index: i32, _flags: i32) {
        self.sim.clear_roads(index);
    }
    fn fill_cache(&mut self) {
        let at = self.sim.movement.mesh.at;
        self.sim.movement.mesh.fill_cache(&self.sim.world, at);
    }
    fn road_changed(&mut self, flag: bool) -> i32 {
        let at = self.sim.movement.mesh.at;
        self.sim
            .movement
            .mesh
            .road_changed(&self.sim.world, at, true, 0, i32::from(flag))
    }
    fn get_road_data(&mut self) -> bool {
        let at = self.sim.movement.mesh.at;
        self.sim
            .movement
            .mesh
            .get_road_data(&self.sim.world, at, false, 0, false)
    }
    fn mark_and_trim(&mut self, list: i32) {
        self.sim
            .movement
            .mesh
            .trim_directions(&self.sim.world, list);
    }
    fn set_diags(&mut self, list: i32, param: i32) -> i32 {
        self.sim.set_diags(list, param)
    }
    // The texture marks pick a texture and a rotation, which nothing here
    // reads (`docs/ROADS.md` §9.4).
    fn mark_splits(&mut self) {}
    fn mark_white_lines(&mut self) {}
    fn mark_yellow_lines(&mut self) {}
    /// Item 4678 (`docs/RENDERER.md` §6.6): `mark_splits`' clears and the
    /// marks on the tile being laid, as the redo pass runs them
    /// ([`RoadMesh::mark_intersections`]). A tile laid joined both ways
    /// between two runs takes its bits here, and nothing redoes it after.
    ///
    /// Item 4687: `mark_splits(1)`, `mark_white_lines(1)` and
    /// `mark_yellow_lines(1)` run in here, before the marks, as one pass on
    /// the joins they share; the three hooks above stay empty.
    fn mark_intersections(&mut self) {
        self.sim
            .movement
            .mesh
            .mark_intersections(&self.sim.world, 1);
    }
    fn fixup_lines(&mut self, arg: i32) {
        self.sim.movement.mesh.fixup_lines(&self.sim.world, arg);
    }
    /// The dword whole as the original's pass reads it: the connections
    /// and the line bytes ([`Elem::lines`]); the stragglers' masks the add
    /// arm runs inline write the line bytes back apart.
    fn elem_flags(&self) -> u32 {
        let m = &self.sim.movement.mesh;
        m.flags_at(&self.sim.world, m.at) | m.lines_of(&self.sim.world, m.at)
    }
    fn set_elem_flags(&mut self, flags: u32) {
        let at = self.sim.movement.mesh.at;
        let world = &self.sim.world;
        if let Some(e) = self.sim.movement.mesh.elems.get_mut(&index_of(world, at)) {
            e.lines = flags & LINES;
            e.flags = flags & !LINES;
        }
    }
    fn find_piece(&mut self) {
        // The piece the pick settles on is the road tile's own, so the C4 a
        // corner left on it goes (§12; the pick itself is the client's, on
        // the intersections it reads here: item 4682).
        let at = self.sim.movement.mesh.at;
        self.sim.movement.mesh.note_pick(&self.sim.world, at);
        if self.sim.world.tile_mask(at) & tile::SURFACE == tile::SURFACE_ROAD
            && let Some(e) = self
                .sim
                .movement
                .mesh
                .elems
                .get_mut(&index_of(&self.sim.world, at))
        {
            e.c4 = false;
        }
    }
    /// The list as the original holds it: the simulation's entries and the
    /// line passes' ([`RoadMesh::tex`]), each tile once.
    fn redo_len(&self, which: usize) -> i32 {
        let m = &self.sim.movement.mesh;
        let extra = m.tex[which]
            .iter()
            .filter(|i| !m.redo[which].contains(i))
            .count();
        (m.redo[which].len() + extra) as i32
    }
    fn redo_changed_roads(&mut self, which: usize, arg: i32, param: i32) {
        self.sim.redo_changed_roads(which, arg, param);
    }
    fn redo_clear(&mut self, which: usize) {
        self.sim.movement.mesh.redo[which].clear();
        self.sim.movement.mesh.tex[which].clear();
    }
    fn tile_flags(&self, _x: i32, _y: i32) -> u16 {
        0
    }
    fn cell_has_tiles(&self, _cx: i32, _cy: i32) -> bool {
        false
    }
    fn cell_flags(&self, _cx: i32, _cy: i32) -> u16 {
        0
    }
    fn set_cell_flags(&mut self, _cx: i32, _cy: i32, _bits: u16) {}
    fn tile_rec(&self, _x: i32, _y: i32) -> Option<TileRec> {
        None
    }
    fn add_ambience(&mut self, _x: i32, _y: i32) {}
    fn finish(&mut self) {
        self.sim.movement.mesh.redo[0].clear();
        self.sim.movement.mesh.redo[1].clear();
        self.sim.movement.mesh.tex[0].clear();
        self.sim.movement.mesh.tex[1].clear();
        self.sim.movement.mesh.mods.count = 0;
    }
}

#[cfg(test)]
mod tests;
