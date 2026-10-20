//! **`Objects::find_builds@0065a120`'s walk** (item 3379): the building
//! search as a pure walk over a [`FindBuildsView`], so the sweep's port is a
//! view over the emulated original's windows and the crate's is
//! [`SimFindBuilds`] (`docs/ORDERS.md` §5.5).
//!
//! The listing's shape, which the decompile hides in three places:
//!
//! 1. `objects.len` (`+0x208`) is cleared only when the continue flag
//!    (argument 9) is zero; `+0x200` takes `who` and `+0x1fc` takes
//!    99 999 999 (the "nearest so far" pair `find_unit_with_radius` shares);
//! 2. with the `0x200` bit set the searcher's own cell region (`+4` of its
//!    world cell, **no bounds test**) gates every candidate, else −1;
//! 3. `ring = (range + 0x2ff) / 0x300` and the **circle walk** runs when
//!    `range >= 0` and `circle_radius[ring] <= game.num_def_builds`: each
//!    cell of the ring in bounds (and, gated, of the searcher's region), its
//!    chain `(id, player)` from the cell's shorts at `+8` and `+10`, then
//!    each object's own `+0x2c`/`+0x2e` — **read before the tests**;
//!    `player < 8`, `player < 0 || valid_search`, `exists`, then
//!    `valid_filter` when `filter - 1 <= 0x16`;
//! 4. otherwise the **list walk**: each leader whose flag bit 0 is set and
//!    that `valid_search` keeps, objects `2000 .. count` (re-read each step)
//!    by `exists`, the filter, the region gate on the *object's* position
//!    (`pos ^ 0x63637`), and — when `range >= 0` — `vector_dist(dx - 192 *
//!    r1, dx - 192 * r2)` against the range, where **both** operands are
//!    the x offset: the listing reads `+0x10` once, never `+0x14`, and
//!    subtracts the type's `+0x234` from one copy and `+0x238` from the
//!    other (each through its own vtable `+0xac` call).

use crate::world::vector_dist;

/// The arguments, in stack order (`param_1 .. param_10`; `param_9` is never read).
#[derive(Clone, Copy, Debug)]
pub struct FindBuildsArgs {
    /// `param_1`, the point's x.
    pub x: i32,
    /// `param_2`, the point's y.
    pub y: i32,
    /// `param_3`, the `SearchIndexBH` handed to `valid_search`.
    pub search: i32,
    /// `param_4`, `who`.
    pub who: i32,
    /// `param_5`, the range (negative: the list walk, unbounded).
    pub range: i32,
    /// `param_6`, the flags (`& 0x200` the region gate).
    pub flags: i32,
    /// `param_7`, the filter.
    pub filter: i32,
    /// `param_8`, handed to `valid_filter`.
    pub p8: i32,
    /// `param_10`, non-zero: append to what the array holds.
    pub cont: i32,
}

/// What `find_builds` reads and calls, in the listing's order. An object is
/// `(player, id)`: the player's array and the index into it.
pub trait FindBuildsView {
    /// `objects.len = 0`.
    fn clear_len(&mut self);
    /// `objects+0x200 = who`, `objects+0x1fc = 99999999`.
    fn note_search(&mut self, who: i32);
    /// `div_3_table[i]`.
    fn div3(&mut self, i: i32) -> i32;
    /// `world.width`.
    fn width(&mut self) -> i32;
    /// `world.height`.
    fn height(&mut self) -> i32;
    /// The sign-extended short at `+4` of the world cell at flat index `y * width + x`; no bounds test.
    fn cell_region(&mut self, index: i32) -> i32;
    /// The sign-extended shorts at `+8` and `+10` of the cell at flat index `index`: `(id, player)`.
    fn cell_head(&mut self, index: i32) -> (i32, i32);
    /// `game.num_def_builds`.
    fn num_def_builds(&mut self) -> i32;
    /// `circle_radius[ring]`.
    fn circle_radius(&mut self, ring: i32) -> i32;
    /// `(circle_x[i], circle_y[i])`.
    fn circle_point(&mut self, i: i32) -> (i32, i32);
    /// The object's shorts at `+0x2c` and `+0x2e`: `(next id, next player)`.
    fn link(&mut self, player: i32, id: i32) -> (i32, i32);
    /// `Search::valid_search(player, search, who)`.
    fn valid_search(&mut self, player: i32, search: i32, who: i32) -> bool;
    /// The object's vtable `+0x10`.
    fn exists(&mut self, player: i32, id: i32) -> bool;
    /// `Search::valid_filter(player, id, p8, 0, filter - 1)`.
    fn valid_filter(&mut self, player: i32, id: i32, p8: i32, filter: i32) -> bool;
    /// Append the object to the result array (growing it first when full).
    fn push(&mut self, player: i32, id: i32);
    /// `objects.len`.
    fn result_len(&mut self) -> i32;
    /// `leaders[k]`'s flag bit 0.
    fn leader_in_game(&mut self, k: i32) -> bool;
    /// `objects+0x184 + 4k`, the count of player `k`'s array.
    fn count(&mut self, k: i32) -> i32;
    /// The object's `(+0x10, +0x14)` words, un-xored.
    fn position(&mut self, player: i32, id: i32) -> (i32, i32);
    /// `[[vtable +0xac]()->+0x18]->+0x234`: the first reach.
    fn reach_x(&mut self, player: i32, id: i32) -> i32;
    /// `[[vtable +0xac]()->+0x18]->+0x238`: the second reach.
    fn reach_y(&mut self, player: i32, id: i32) -> i32;
}

const POS_KEY: i32 = 0x63637;

/// `Objects::find_builds`' walk; the answer is the original's `eax`, the array's length.
pub fn find_builds_steps(v: &mut impl FindBuildsView, a: &FindBuildsArgs) -> i32 {
    if a.cont == 0 {
        v.clear_len();
    }
    let gated = a.flags & 0x200 != 0;
    v.note_search(a.who);
    let mut region = -1;
    if gated {
        let ty = v.div3(a.y >> 8);
        let w = v.width();
        let tx = v.div3(a.x >> 8);
        region = v.cell_region(ty.wrapping_mul(w).wrapping_add(tx));
    }
    let ring = a.range.wrapping_add(0x2ff) / 0x300;
    if a.range >= 0 {
        let n = v.circle_radius(ring);
        if n <= v.num_def_builds() {
            let cx = v.div3(a.x >> 8);
            let cy = v.div3(a.y >> 8);
            for i in 0..n {
                let (dx, dy) = v.circle_point(i);
                let (x, y) = (dx + cx, dy + cy);
                if x < 0 || y < 0 || x >= v.width() || y >= v.height() {
                    continue;
                }
                let w = v.width();
                let index = y.wrapping_mul(w).wrapping_add(x);
                let (mut id, mut player) = v.cell_head(index);
                if id < 0 || (gated && v.cell_region(index) != region) {
                    continue;
                }
                while id >= 0 {
                    let (nid, nplayer) = v.link(player, id);
                    if player < 8
                        && (player < 0 || v.valid_search(player, a.search, a.who))
                        && v.exists(player, id)
                        && (a.filter == 0
                            || (a.filter - 1) as u32 > 0x16
                            || v.valid_filter(player, id, a.p8, a.filter - 1))
                    {
                        v.push(player, id);
                    }
                    id = nid;
                    player = nplayer;
                }
            }
            return v.result_len();
        }
    }
    for k in 0..8 {
        if !v.leader_in_game(k) || !v.valid_search(k, a.search, a.who) {
            continue;
        }
        if v.count(k) <= 2000 {
            continue;
        }
        let mut id = 2000;
        loop {
            if v.exists(k, id)
                && (a.filter == 0
                    || (a.filter - 1) as u32 > 0x16
                    || v.valid_filter(k, id, a.p8, a.filter - 1))
                && (!gated || {
                    let (px, py) = v.position(k, id);
                    let ty = v.div3((py ^ POS_KEY) >> 8);
                    let w = v.width();
                    let tx = v.div3((px ^ POS_KEY) >> 8);
                    v.cell_region(ty.wrapping_mul(w).wrapping_add(tx)) == region
                })
            {
                let (px, _) = v.position(k, id);
                let dx = (px ^ POS_KEY).wrapping_sub(a.x);
                let dx = dx.wrapping_abs();
                let r1 = v.reach_x(k, id);
                let c1 = dx.wrapping_sub(r1.wrapping_mul(192));
                let e1 = if c1 < 0 || dx < 0 { 0 } else { c1.min(dx) };
                let r2 = v.reach_y(k, id);
                let c2 = dx.wrapping_sub(r2.wrapping_mul(192));
                let e2 = if c2 < 0 || dx < 0 { 0 } else { c2.min(dx) };
                if a.range < 0 || vector_dist(e1, e2) <= a.range {
                    v.push(k, id);
                }
            }
            id += 1;
            if id >= v.count(k) {
                break;
            }
        }
    }
    v.result_len()
}

/// `Search::valid_search`'s case 1, `SEARCH_FRIENDLY`.
pub const SEARCH_FRIENDLY: i32 = 1;
/// `FILTER_CONSTRUCT`, arm 5 of `Search::valid_filter@0067dbb0` (`filter - FILTER_TYPE`
/// with `FILTER_TYPE` 1): an object that is not active.
pub const FILTER_CONSTRUCT: i32 = 6;

/// [`FindBuildsView`] over the sim, for `Unit::find_build_spot`'s search
/// (`Objects::find_builds(SEARCH_FRIENDLY, owner, range, 0x200,
/// FILTER_CONSTRUCT)`; `docs/ORDERS.md` §5.5).
///
/// The sim threads no building into a cell's chain (`docs/QUEUE.md` 48), so
/// the chain of a cell is its live buildings in descending slot order —
/// `Object::add_to_world` pushes onto the head — each `(owner, index)` as the
/// owner's array holds them (`Building::index`, 2000 and up), and the list
/// walk is the owner's array from 2000. The leaves fold the caller's own
/// tests in: a player is searchable when it is the searcher's (`site` keeps
/// only the builder's own sites, so an ally's could never survive the second
/// pass), `exists` is `alive`, `FILTER_CONSTRUCT` is `!active`, and
/// `build_masks & 0x20` (under attack) drops a site with it.
pub struct SimFindBuilds<'a> {
    s: &'a crate::Sim,
    who: i32,
    by_id: std::collections::BTreeMap<(i32, i32), usize>,
    chain: Vec<usize>,
    /// The sites found, as building slots, in the original's array order.
    pub out: Vec<usize>,
}

impl<'a> SimFindBuilds<'a> {
    /// A view of `who`'s search over `s`.
    pub fn new(s: &'a crate::Sim, who: usize) -> Self {
        let by_id = (0..s.buildings.len())
            .filter(|&b| s.buildings[b].alive)
            .map(|b| {
                (
                    (
                        i32::from(s.buildings[b].owner),
                        i32::from(s.buildings[b].index),
                    ),
                    b,
                )
            })
            .collect();
        Self {
            s,
            who: who as i32,
            by_id,
            chain: Vec::new(),
            out: Vec::new(),
        }
    }

    fn site(&self, player: i32, id: i32) -> Option<usize> {
        self.by_id.get(&(player, id)).copied()
    }

    fn ident(&self, b: usize) -> (i32, i32) {
        (
            i32::from(self.s.buildings[b].index),
            i32::from(self.s.buildings[b].owner),
        )
    }

    fn type_size(&self, player: i32, id: i32, y: bool) -> i32 {
        self.site(player, id)
            .and_then(|b| self.s.buildings[b].ty)
            .map_or(0, |t| {
                let bt = &self.s.build_types[t];
                if y { bt.y_size } else { bt.x_size }
            })
    }

    fn cell_at(&self, index: i32) -> crate::world::Cell {
        let w = self.s.world.width();
        crate::world::Cell::new(index % w, index / w)
    }
}

impl FindBuildsView for SimFindBuilds<'_> {
    fn clear_len(&mut self) {
        self.out.clear();
    }
    fn note_search(&mut self, _who: i32) {}
    fn div3(&mut self, i: i32) -> i32 {
        i / 3
    }
    fn width(&mut self) -> i32 {
        self.s.world.width()
    }
    fn height(&mut self) -> i32 {
        self.s.world.height()
    }
    fn cell_region(&mut self, index: i32) -> i32 {
        self.s
            .world
            .region_of(self.cell_at(index))
            .map_or(-1, i32::from)
    }
    fn cell_head(&mut self, index: i32) -> (i32, i32) {
        let c = self.cell_at(index);
        self.chain = self
            .by_id
            .values()
            .copied()
            .filter(|&b| self.s.buildings[b].pos.cell() == c)
            .collect();
        self.chain.sort_unstable_by(|a, b| b.cmp(a));
        self.chain.first().map_or((-1, -1), |&b| self.ident(b))
    }
    fn num_def_builds(&mut self) -> i32 {
        crate::orders::NUM_DEF_BUILDS as i32
    }
    fn circle_radius(&mut self, ring: i32) -> i32 {
        crate::ai_place::circle().radius[ring.clamp(0, 0x40) as usize] as i32
    }
    fn circle_point(&mut self, i: i32) -> (i32, i32) {
        let c = crate::ai_place::circle();
        (c.x[i as usize], c.y[i as usize])
    }
    fn link(&mut self, player: i32, id: i32) -> (i32, i32) {
        let here = self.site(player, id);
        self.chain
            .iter()
            .position(|&b| Some(b) == here)
            .and_then(|k| self.chain.get(k + 1))
            .map_or((-1, -1), |&b| self.ident(b))
    }
    fn valid_search(&mut self, player: i32, _search: i32, _who: i32) -> bool {
        player == self.who
    }
    fn exists(&mut self, player: i32, id: i32) -> bool {
        self.site(player, id).is_some()
    }
    fn valid_filter(&mut self, player: i32, id: i32, _p8: i32, _filter: i32) -> bool {
        self.site(player, id).is_some_and(|b| {
            let bd = &self.s.buildings[b];
            !bd.active && !bd.is_under_attack()
        })
    }
    fn push(&mut self, player: i32, id: i32) {
        if let Some(b) = self.site(player, id) {
            self.out.push(b);
        }
    }
    fn result_len(&mut self) -> i32 {
        self.out.len() as i32
    }
    fn leader_in_game(&mut self, k: i32) -> bool {
        k == self.who
    }
    fn count(&mut self, k: i32) -> i32 {
        self.by_id
            .keys()
            .filter(|&&(p, _)| p == k)
            .map(|&(_, id)| id + 1)
            .max()
            .unwrap_or(0)
            .max(2000)
    }
    fn position(&mut self, player: i32, id: i32) -> (i32, i32) {
        self.site(player, id).map_or((0, 0), |b| {
            let p = self.s.buildings[b].pos;
            (p.x ^ POS_KEY, p.y ^ POS_KEY)
        })
    }
    fn reach_x(&mut self, player: i32, id: i32) -> i32 {
        self.type_size(player, id, false)
    }
    fn reach_y(&mut self, player: i32, id: i32) -> i32 {
        self.type_size(player, id, true)
    }
}

#[cfg(test)]
mod tests {
    use crate::world::{Cell, Pos};
    use crate::{Sim, Tuning, Unit, World};

    /// A land of one region, a builder of player 0 in cell (2, 2), and `n` sites.
    fn land(sites: &[(u8, i32, i32)]) -> (Sim, usize, Vec<usize>) {
        let mut world = World::new(8, 8);
        world.fill_region(
            crate::world::Terrain::Land,
            Cell::new(0, 0),
            Cell::new(7, 7),
        );
        world.add_region(crate::world::Terrain::Land);
        let mut s = Sim::new(Tuning::RON, world, 4);
        let u = s.add_unit(Unit::new(0, 1, Pos::new(2 * 768 + 100, 2 * 768 + 100), 10));
        let b = sites
            .iter()
            .map(|&(owner, x, y)| {
                let b = s.add_building(owner, Pos::new(x, y), 1);
                s.buildings[b].active = false;
                b
            })
            .collect();
        (s, u, b)
    }

    #[test]
    fn a_cell_lists_its_newest_site_first_and_keeps_only_mine_unfinished_and_unattacked() {
        let c = (2 * 768 + 300, 2 * 768 + 300);
        let (mut s, u, b) = land(&[
            (0, c.0, c.1),
            (0, c.0 + 5, c.1),
            (1, c.0, c.1 + 5),
            (0, c.0 + 9, c.1),
            (0, c.0 + 20, c.1),
        ]);
        s.buildings[b[3]].active = true;
        s.buildings[b[4]].under_attack = 2;
        // Range 0x300 is ring 1: the circle walk.
        assert_eq!(s.find_construct_sites(u, 0x300), vec![b[1], b[0]]);
    }

    #[test]
    fn a_site_in_another_region_is_not_found() {
        let (mut s, u, b) = land(&[(0, 2 * 768 + 300, 2 * 768 + 300)]);
        s.world.set_region(Cell::new(3, 2), 1);
        s.buildings[b[0]].pos = Pos::new(3 * 768 + 10, 2 * 768 + 10);
        assert!(s.find_construct_sites(u, 0x300).is_empty());
        s.world.set_region(Cell::new(3, 2), 0);
        assert_eq!(s.find_construct_sites(u, 0x300), vec![b[0]]);
    }

    #[test]
    fn the_list_walk_measures_the_x_offset_twice_less_the_footprint() {
        // Range 0x2400 is ring 12, 452 points: past `num_def_builds`, so the list walk.
        let here = 2 * 768 + 100;
        // Straight north, 1500 units away: the original never reads the y offset.
        let (s, u, b) = land(&[(0, here, here + 3 * 768), (0, here + 0x2500, here)]);
        assert_eq!(s.find_construct_sites(u, 0x2400), vec![b[0]]);
    }
}
