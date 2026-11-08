//! The emulated original's windows as `astar_path`'s view (item 3381): one
//! `V` that the slices' ports share. Not a sweep itself — the slices are
//! `path_finder_astar_suspend` (a fresh search whose first node suspends it)
//! and `path_finder_astar_probe_{world,tile,unit}` (a fresh search whose first
//! node is expanded, to the first probe of the wheel).
//!
//! **The windows** every slice lays alike (`tools/sweep_gen.py`, the same
//! `--cell-win`/`--place`/`--answer-win` order): `c0` the world (`+0` width,
//! `+0x18` width in tiles, `+0x134` the region table, `+0x138` the tile
//! table), `c1` `div_3_table`, `c2..c5` the `units` rows the unit is laid in;
//! `p0` the stack (`+0` list, `+8` length), `p1` its list, `p2` the unit data
//! (the `units` rows point back at it), `p3` its type, `p4` the order list's
//! end node, `p5` its first node (`+8` the order, `+0xc` its flag), `p6` the
//! order, `p7` the open tree (`+8` length, `+0x10` current node), `p8` the
//! region table, `p9` the tile table, `p10`/`p11` the two `get_action`
//! answers, `p12` `update_order`'s, `p13` the move order its slot answers
//! (`+0x4c`/`+0x4e`), `p14` the node `Recycler<PathNode>::pop` answers,
//! `p15`/`p16` the two `get_order` answers (`+4` the flags), `p17` the node
//! `first_open_node` answers (`PathNode`: `+0` x, `+4` y, `+8` length, `+0xc`
//! estimate, `+0x10` value, `+0x14` timeout, `+0x18` metric, `+0x1c` z,
//! `+0x1e` transport, `+0x1f` building, `+0x20` parent).
//!
//! **The row's numbers**, `a`: the three stack arguments, then the stubs'
//! single answers (`a[3 + k]`, stub k: 0 `get_action`, 1 its vslot `+0x10`,
//! 2 `order_type`, 3 the order's vslot `+0x14`, 4 `update_order`, 5 its
//! vslot `+0x40`, 6 `pop`, 7/8 the two `ordered_insert`s, 9 `get_tregion`, 10
//! `is_ocean`, 11 `get_order`, 12 `first_open_node`, then the slice's own),
//! then `get_tregion`'s two answers and the four null flags (`get_action`
//! twice, `get_order` twice): the last six numbers of the row.

use sim::Pos;
use sim::astar_view::{AstarView, NoDangerView, Stash, SuspendView};
use sim::orders::PathData;

pub(crate) fn rd32(w: &[u8], o: usize) -> i32 {
    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
}

fn rd16(w: &[u8], o: usize) -> i32 {
    i32::from(i16::from_le_bytes(
        w[o..o + 2].try_into().expect("two bytes"),
    ))
}

pub(crate) fn wr32(w: &mut [u8], o: usize, v: i32) {
    w[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

pub(crate) struct V<'a> {
    pub a: &'a [i32],
    pub m: &'a mut [u8],
    pub c: &'a mut [Vec<u8>],
    pub p: &'a mut [Vec<u8>],
    /// `get_action` asked so far (each ask has its own null flag).
    actions: usize,
    /// `get_tregion` asked so far (each ask has its own answer).
    tregions: usize,
    pub ins_tree: Vec<String>,
    pub ins_br: Vec<String>,
}

impl<'a> V<'a> {
    pub(crate) fn new(
        a: &'a [i32],
        m: &'a mut [u8],
        c: &'a mut [Vec<u8>],
        p: &'a mut [Vec<u8>],
    ) -> Self {
        V {
            a,
            m,
            c,
            p,
            actions: 0,
            tregions: 0,
            ins_tree: Vec::new(),
            ins_br: Vec::new(),
        }
    }
    /// The null flag of the `k`th nullable answer (`get_action` 0, 1;
    /// `get_order` 2, 3).
    fn null_flag(&self, k: usize) -> i32 {
        self.a[self.a.len() - 4 + k]
    }
    /// `get_tregion`'s `k`th answer.
    fn tregion_answer(&self, k: usize) -> i32 {
        self.a[self.a.len() - 6 + k]
    }
    /// The single answer of stub `k`.
    pub(crate) fn stub(&self, k: usize) -> i32 {
        self.a[3 + k]
    }
}

impl NoDangerView for V<'_> {
    fn action_type(&mut self) -> Option<i32> {
        let flag = self.null_flag(self.actions);
        self.actions += 1;
        (flag == 0).then(|| self.stub(1))
    }
    fn owner(&mut self) -> i32 {
        i32::from(self.p[2][9])
    }
    fn order_type(&mut self) -> i32 {
        self.stub(2)
    }
}

impl AstarView for V<'_> {
    fn masks2(&mut self) -> i32 {
        rd32(&self.p[2], 0x6c)
    }
    fn collision(&mut self) -> i32 {
        rd32(&self.p[3], 0x248)
    }
    fn transport_type(&mut self) -> bool {
        self.p[3][0x2b4] & 0x10 != 0
    }
    fn ai_bit(&mut self) -> bool {
        rd32(&self.p[2], 0x68) & 0x40000 != 0
    }
    fn list_present(&mut self) -> bool {
        rd32(&self.p[2], 0xdc) != 0
    }
    fn cursor_to_front(&mut self) {
        // `+0xd4 = end->next`, `+0xcc = its order`, `+0xd0 = its flag`.
        let n = rd32(&self.p[4], 4);
        let o = rd32(&self.p[5], 8);
        let f = self.p[5][0xc];
        wr32(&mut self.p[2], 0xd4, n);
        wr32(&mut self.p[2], 0xcc, o);
        self.p[2][0xd0] = f;
    }
    fn has_order(&mut self) -> bool {
        rd32(&self.p[2], 0xcc) != 0
    }
    fn order_is_move(&mut self) -> bool {
        self.stub(3) != 0
    }
    fn move_offset(&mut self) -> (i32, i32) {
        (rd16(&self.p[13], 0x4c), rd16(&self.p[13], 0x4e))
    }
    fn order_present(&mut self) -> bool {
        self.null_flag(2) == 0
    }
    fn order_flags(&mut self) -> i32 {
        // `get_order` asked again: a null answer reads the zero page.
        if self.null_flag(3) == 0 {
            i32::from(self.p[16][4])
        } else {
            0
        }
    }
    fn world_width(&mut self) -> i32 {
        rd32(&self.c[0], 0)
    }
    fn world_width_tiles(&mut self) -> i32 {
        rd32(&self.c[0], 0x18)
    }
    fn div3(&mut self, i: i32) -> i32 {
        rd32(&self.c[1], 4 * i as usize)
    }
    fn region_of_cell(&mut self, cx: i32, cy: i32) -> i32 {
        let w = rd32(&self.c[0], 0);
        rd16(&self.p[8], 4 + (cy * w + cx) as usize * 0x1c) & 0xffff
    }
    fn tregion(&mut self, _tx: i32, _ty: i32) -> i32 {
        let v = self.tregion_answer(self.tregions.min(1));
        self.tregions += 1;
        v
    }
    fn is_ocean_cell(&mut self, _cx: i32, _cy: i32) -> bool {
        self.stub(10) != 0
    }
    fn tile_mask(&mut self, tx: i32, ty: i32) -> i32 {
        let w = rd32(&self.c[0], 0x18);
        i32::from(u16::from_le_bytes(
            self.p[9][(ty * w + tx) as usize * 2..][..2]
                .try_into()
                .expect("two bytes"),
        ))
    }
    fn pop_path(&mut self) -> PathData {
        let mut len = rd32(&self.p[0], 8);
        if len < 1 {
            len = 1;
        }
        len -= 1;
        wr32(&mut self.p[0], 8, len);
        let k = len as usize * 16;
        PathData {
            to: Pos::new(rd32(&self.p[1], k), rd32(&self.p[1], k + 4)),
            tolerance: rd32(&self.p[1], k + 8),
            flags: rd32(&self.p[1], k + 12) as u8,
        }
    }
    fn top_tolerance(&mut self) -> i32 {
        let len = rd32(&self.p[0], 8);
        if len == 0 {
            0
        } else {
            rd32(&self.p[1], (len as usize - 1) * 16 + 8)
        }
    }
    fn set_iroquois(&mut self, word: i32) {
        wr32(self.m, 0x74, word);
    }
    fn set_no_danger(&mut self, on: bool) {
        wr32(self.m, 0x7c, i32::from(on));
    }
    fn reset_valid_hit(&mut self) {
        wr32(self.m, 0x90, 0);
    }
    fn seed_open(&mut self, start: Pos, metric: i32, estimate: i32) {
        // The node `pop` answered (`p14`), field by field: the listing
        // clears `length`, `parent`, `timeout`, `transport` and `building`.
        let n = &mut self.p[14];
        wr32(n, 0, start.x);
        wr32(n, 4, start.y);
        wr32(n, 8, 0);
        wr32(n, 0x14, 0);
        wr32(n, 0x20, 0);
        n[0x1e] = 0;
        n[0x1f] = 0;
        wr32(n, 0x18, metric);
        wr32(n, 0xc, estimate);
        wr32(n, 0x10, estimate);
        self.ins_tree.push(format!("p7+0x0,p14+0x0,{estimate}"));
        let cur = rd32(&self.p[7], 0x10);
        let ecx = rd32(self.m, 0x44);
        self.ins_br.push(format!("{ecx},{cur},{metric}"));
    }
    fn set_avoid(&mut self, land: i32, sea: i32) {
        wr32(self.m, 0x88, land);
        wr32(self.m, 0x8c, sea);
    }
}

impl SuspendView for V<'_> {
    fn reinsert_current(&mut self) {
        let (value, metric) = (rd32(&self.p[17], 0x10), rd32(&self.p[17], 0x18));
        self.ins_tree.push(format!("p7+0x0,p17+0x0,{value}"));
        let cur = rd32(&self.p[7], 0x10);
        let ecx = rd32(self.m, 0x44);
        self.ins_br.push(format!("{ecx},{cur},{metric}"));
    }
    fn hand_over(&mut self, st: &Stash) {
        wr32(self.m, 0x84, 1);
        // Each container goes to the unit and a fresh one comes off its
        // recycler (stubs 13..17 of the suspend slice).
        let a = self.a;
        let m = &mut *self.m;
        let u = &mut self.p[2];
        for (k, (mine, theirs)) in [(0x40, 0x104), (0x44, 0x108), (0x48, 0x10c), (0x50, 0x110)]
            .into_iter()
            .enumerate()
        {
            wr32(u, theirs, rd32(m, mine));
            wr32(m, mine, a[3 + 13 + k]);
        }
        wr32(u, 0x134, rd32(m, 0x90));
        wr32(u, 0x114, rd32(m, 0x4c));
        wr32(m, 0x4c, a[3 + 17]);
        wr32(u, 0x128, st.tol);
        wr32(u, 0x12c, st.pref);
        wr32(u, 0x140, st.goal.x);
        wr32(u, 0x144, st.goal.y);
        wr32(u, 0x130, st.start_dist);
        wr32(u, 0x148, st.traversed);
        wr32(u, 0x138, rd32(m, 0x88));
        wr32(u, 0x13c, rd32(m, 0x8c));
    }
}

use sim::astar_view::{
    Stash as St, Top, arrive, child_metric, child_pos, entry_steps, fresh_steps, suspend_steps,
    top_of_loop, wheel_dir, world_probe,
};

/// Slice B: a fresh unit search whose first node suspends it (`+0x64` set,
/// `+0x80` negative, `anti` 0, the node far from the goal), so every row runs
/// from the entry to the `-1` return. Stubs 13..17 are the five recyclers'
/// pops.
pub(crate) fn run_suspend(
    a: &[i32],
    m: &mut [u8],
    c: &mut [Vec<u8>],
    p: &mut [Vec<u8>],
    log: &mut Vec<String>,
) -> i64 {
    let (step, anti) = (a[1], a[2]);
    assert_eq!(rd32(m, 0x84), 0, "slice B is a fresh search");
    let anti_unit = rd32(m, 0x64) != 0;
    let limit = rd32(m, 0x80);
    let mut v = V::new(a, m, c, p);
    let e = entry_steps(&mut v, step, anti);
    let f = fresh_steps(&mut v, step);
    // The loop's head: `first_open_node` answers `p17`.
    let (cx, cy) = (rd32(&v.p[17], 0), rd32(&v.p[17], 4));
    let manh = cx
        .wrapping_sub(f.goal.x)
        .wrapping_abs()
        .wrapping_add(cy.wrapping_sub(f.goal.y).wrapping_abs());
    let top = top_of_loop(
        manh,
        arrive(f.tol, e.stride),
        0,
        0,
        e.work_cap,
        anti_unit,
        limit,
        anti,
    );
    assert_eq!(top, Top::Suspend, "the draw reaches only the suspend");
    suspend_steps(
        &mut v,
        &St {
            tol: f.tol,
            pref: f.pref,
            goal: f.goal,
            start_dist: f.start_dist,
            traversed: 0,
            avoid: f.avoid,
        },
    );
    log.push(v.ins_tree.join("/"));
    log.push(v.ins_br.join("/"));
    -1
}

/// Slices C: a fresh search whose first node is expanded, to the wheel's
/// first probe — `valid_wcoord` (stub 14, `ecx` the pathfinder), `invalid_loc`
/// (`ecx` the unit data) or `valid_ucoord` (`ecx` the pathfinder) by step —
/// where the original's call ends (`--stop-stub 14`) and answers stub 14's
/// number. Stub 13 is the `malloc` the empty node pool falls back to.
pub(crate) fn run_probe(
    a: &[i32],
    m: &mut [u8],
    c: &mut [Vec<u8>],
    p: &mut [Vec<u8>],
    log: &mut Vec<String>,
) -> i64 {
    let (step, anti) = (a[1], a[2]);
    assert_eq!(rd32(m, 0x84), 0, "slice C is a fresh search");
    let anti_unit = rd32(m, 0x64) != 0;
    let limit = rd32(m, 0x80);
    let mut v = V::new(a, m, c, p);
    let e = entry_steps(&mut v, step, anti);
    let f = fresh_steps(&mut v, step);
    let (cx, cy) = (rd32(&v.p[17], 0), rd32(&v.p[17], 4));
    let manh = cx
        .wrapping_sub(f.goal.x)
        .wrapping_abs()
        .wrapping_add(cy.wrapping_sub(f.goal.y).wrapping_abs());
    let top = top_of_loop(
        manh,
        arrive(f.tol, e.stride),
        0,
        0,
        e.work_cap,
        anti_unit,
        limit,
        anti,
    );
    assert_eq!(top, Top::Expand, "the draw reaches only the expansion");
    // The wheel's first turn, `pref + 1`.
    let d = wheel_dir(f.pref + 1);
    let cur = Pos::new(cx, cy);
    let (cur_metric, cur_timeout) = (rd32(&v.p[17], 0x18), rd32(&v.p[17], 0x14));
    let child = child_pos(cur, d, e.stride);
    let probe = match step {
        0x300 => {
            let q = world_probe(cur_timeout, child, e.toff);
            format!(
                "m+0x0,{},{},{cur_timeout},{},{}",
                q.x, q.y, f.goal.x, f.goal.y
            )
        }
        0xc0 => {
            let tx = AstarView::div3(&mut v, child.x >> 6);
            let ty = AstarView::div3(&mut v, child.y >> 6);
            format!("p2+0x0,{tx},{ty},0,1,1,1,0,{}", child.x >> 6)
        }
        _ => {
            let key = child_metric(i64::from(cur_metric), d, i64::from(e.row));
            format!("m+0x0,{},{},{}", child.x, child.y, key as i32)
        }
    };
    log.push(v.ins_tree.join("/"));
    log.push(v.ins_br.join("/"));
    log.push(probe);
    i64::from(a[3 + 14])
}
