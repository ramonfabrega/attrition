//! **`Roads::add_roads@0088f4b0`'s steps** (item 3376): the queue's walks, in
//! the order the original makes them, over a view of everything the function
//! touches — so the sweep's port is the same steps over the emulated
//! original's windows, and the crate's is `mesh.rs`'s `SimRoads`.
//!
//! The function has a shape worth stating once. `RoadsData::added_roads` is
//! an array with an iterator of its own (a cursor and a flag byte whose
//! `0x40` says "past the end"), and four of the function's passes walk it:
//!
//! 1. each queued tile is either a *removal* (`clear_roads`) or an *add*
//!    (`road_changed`, then — when that count is two or more — the add is
//!    re-queued as a removal, and the original entry is remembered in the
//!    function-static `remove_list`);
//! 2. the remembered entries are taken out of the queue;
//! 3. every valid tile has its directions marked and `set_diags` run, which
//!    may append to the queue under the very walk;
//! 4. every valid tile has its pieces picked.
//!
//! A tail ping-pongs between the two `changed_roads` lists, at most eleven
//! rounds. The arrays are modelled as the original lays them — a buffer, a
//! count, a capacity, the iterator — because a callee appends under the walk
//! and the walk's end is read from the count again each step ([`ModQueue`]).

use crate::ai_place::{MOVE_X, MOVE_Y};

/// One `Array<RoadModification>`: the buffer of eight-byte entries — the
/// packed tile index and a dword whose low byte is `added` and next byte
/// `valid` — the count the walks read, the capacity `increase_size` is asked
/// when a push meets it, and the iterator's cursor and flag byte.
///
/// The buffer outlives the count: an entry removed from the middle shifts the
/// ones above it down and leaves the last where it was, and a later append
/// that only raises the count finds that stale entry there.
#[derive(Clone, Debug)]
pub struct ModQueue {
    /// The buffer. Slots at or above [`ModQueue::count`] are stale.
    pub data: Vec<(i32, i32)>,
    /// `+0x4`.
    pub count: i32,
    /// `+0x8`: a push at or past it asks `increase_size` first.
    pub cap: i32,
    /// `+0xc`: the step `increase_size` is handed.
    pub grow: i16,
    /// `+0x18`, the iterator's cursor.
    pub cursor: i32,
    /// `+0x14`; `0x40` is the iterator's end.
    pub flags: u8,
}

impl Default for ModQueue {
    fn default() -> Self {
        ModQueue {
            data: Vec::new(),
            count: 0,
            // The crate's queue is a `Vec`: it is never full.
            cap: i32::MAX,
            grow: 0,
            cursor: 0,
            flags: 0,
        }
    }
}

/// The iterator's end flag.
const END: u8 = 0x40;

impl ModQueue {
    /// The entry in slot `i`, a zero entry past the buffer.
    pub fn slot(&self, i: i32) -> (i32, i32) {
        usize::try_from(i)
            .ok()
            .and_then(|i| self.data.get(i))
            .copied()
            .unwrap_or_default()
    }

    /// Write slot `i`, growing the buffer to hold it.
    fn put(&mut self, i: i32, e: (i32, i32)) {
        let Ok(i) = usize::try_from(i) else {
            return;
        };
        if self.data.len() <= i {
            self.data.resize(i + 1, (0, 0));
        }
        self.data[i] = e;
    }

    /// Write at the count and raise it. The caller has asked
    /// `increase_size` if the capacity said to.
    pub fn push(&mut self, index: i32, flags: i32) {
        self.put(self.count, (index, flags));
        self.count = self.count.wrapping_add(1);
    }

    /// Whether a push must ask `increase_size` first.
    pub fn full(&self) -> bool {
        self.cap <= self.count
    }

    /// The iterator's start: the cursor to zero, the end flag by the count.
    pub fn iter_start(&mut self) {
        self.cursor = 0;
        if self.count == 0 {
            self.flags |= END;
        } else {
            self.flags &= !END;
        }
    }

    /// Past the end.
    pub fn at_end(&self) -> bool {
        self.flags & END != 0
    }

    /// One step: the cursor up, the end flag set once it meets the count.
    pub fn iter_next(&mut self) {
        self.cursor = self.cursor.wrapping_add(1);
        if self.count <= self.cursor {
            self.flags |= END;
        }
    }

    /// The entry under the cursor.
    pub fn cur(&self) -> (i32, i32) {
        self.slot(self.cursor)
    }
}

/// Which array a `grow` is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Which {
    /// `RoadsData::added_roads`, the queue.
    Queue,
    /// `add_roads`' function-static `remove_list`.
    Remove,
}

/// One road tile record of a terrain cell, as `add_roads`' ambience scan
/// reads it: `+0x8`, `+0xa`, `+0xb`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileRec {
    /// `+0x8`; `'\n'` marks a tile with nothing laid.
    pub kind: u8,
    /// `+0xa`.
    pub a: u8,
    /// `+0xb`.
    pub b: u8,
}

/// What `add_roads` asks of the roads, the world and the terrain.
///
/// The cursor is `Roads +0x120` / `+0x124` (`set_tile`), and every callee
/// that takes a tile reads it from there; the element is `+0x14c`
/// (`get_road_data`'s answer) and the flag methods read and write its
/// `+0x4` dword.
pub trait RoadsView {
    /// `RoadsData::added_roads`.
    fn queue(&mut self) -> &mut ModQueue;
    /// The function-static `remove_list`.
    fn remove_list(&mut self) -> &mut ModQueue;
    /// `Array::increase_size(step)` on the array a push found full.
    fn grow(&mut self, which: Which, step: i16);
    /// `world->width_in_cells`, `+0` — the stride `<< 2` the packed indices
    /// use — and `+0x4`, the height.
    fn world_cells(&self) -> (i32, i32);
    /// `world +0x18`, `+0x1c`: the extent in tiles.
    fn world_tiles(&self) -> (i32, i32);
    /// The cursor goes to the tile `(x, y)`.
    fn set_tile(&mut self, x: i32, y: i32);
    /// `Roads::clear_roads(&modification)`.
    fn clear_roads(&mut self, index: i32, flags: i32);
    /// `RoadsOut::fill_cache(x, y)` at the cursor.
    fn fill_cache(&mut self);
    /// `TerrainOut::road_changed(x, y, 1, 0, flag)` at the cursor.
    fn road_changed(&mut self, flag: bool) -> i32;
    /// `TerrainOut::get_road_data(x, y, 0, 0, 0)` at the cursor: whether the
    /// tile has an element, which becomes the cursor's.
    fn get_road_data(&mut self) -> bool;
    /// `RoadsOut::mark_and_trim_directions(list)`.
    fn mark_and_trim(&mut self, list: i32);
    /// `Roads::set_diags(list, param)`; zero sends the pass on to the marks.
    fn set_diags(&mut self, list: i32, param: i32) -> i32;
    /// `mark_splits(1)`.
    fn mark_splits(&mut self);
    /// `mark_white_lines(1)`.
    fn mark_white_lines(&mut self);
    /// `mark_yellow_lines(1)`.
    fn mark_yellow_lines(&mut self);
    /// `mark_intersections`, handed what the last call left in `ecx`.
    fn mark_intersections(&mut self);
    /// `RoadsOut::fixup_lines(arg)`.
    fn fixup_lines(&mut self, arg: i32);
    /// The cursor's element's `+0x4` dword.
    fn elem_flags(&self) -> u32;
    /// Write it.
    fn set_elem_flags(&mut self, flags: u32);
    /// `RoadsOut::find_piece()` at the cursor.
    fn find_piece(&mut self);
    /// How many tiles `changed_roads` (`which` 0) or `changed_roads2` (1)
    /// holds.
    fn redo_len(&self, which: usize) -> i32;
    /// `Roads::redo_changed_roads(list, arg, param)` over that list.
    fn redo_changed_roads(&mut self, which: usize, arg: i32, param: i32);
    /// The list's count to zero.
    fn redo_clear(&mut self, which: usize);
    // ---- the ambience scan the add arm runs under a non-zero `param`
    /// The word of `world +0x138`'s table at tile `(x, y)`.
    fn tile_flags(&self, x: i32, y: i32) -> u16;
    /// The terrain cell `(cx, cy)`'s byte `+0x66`: it holds tile records.
    fn cell_has_tiles(&self, cx: i32, cy: i32) -> bool;
    /// The cell's `+0x1c` word.
    fn cell_flags(&self, cx: i32, cy: i32) -> u16;
    /// The cell's `+0x1c` word gets `bits` set.
    fn set_cell_flags(&mut self, cx: i32, cy: i32, bits: u16);
    /// The tile record at `(x, y)` of a cell that has them; `None` where its
    /// address is null.
    fn tile_rec(&self, x: i32, y: i32) -> Option<TileRec>;
    /// `GraphicEvents::add_ambience(0, x, y, 0.0, -1, -1)` with `ecx = y`
    /// (both scaled to `0xc0` a tile, `+0x60`).
    fn add_ambience(&mut self, x: i32, y: i32);
    /// The queue is emptied — `Roads +0xe4`, `+0x154` and `+0x170` to zero.
    fn finish(&mut self);
}

/// A push to the queue or the remove list, asking `increase_size` first when
/// the array is full.
fn push<V: RoadsView>(v: &mut V, which: Which, index: i32, flags: i32) {
    let q = match which {
        Which::Queue => v.queue(),
        Which::Remove => v.remove_list(),
    };
    if q.full() {
        let step = q.grow;
        v.grow(which, step);
    }
    match which {
        Which::Queue => v.queue().push(index, flags),
        Which::Remove => v.remove_list().push(index, flags),
    }
}

/// `add_roads` run over the queue, as the original's twelve hundred
/// instructions go.
pub fn add_roads_steps<V: RoadsView>(v: &mut V, param: i32) {
    v.remove_list().count = 0;
    // ---- pass one: the adds and the removals
    v.queue().iter_start();
    let mut cur = (0, 0x100);
    if !v.queue().at_end() {
        cur = v.queue().cur();
    }
    while !v.queue().at_end() {
        let (index, flags) = cur;
        let stride = v.world_cells().0.wrapping_shl(2);
        let (x, y) = (index % stride, index / stride);
        v.set_tile(x, y);
        if (flags >> 8) & 0xff != 0 {
            if flags & 0xff == 0 {
                v.clear_roads(index, flags);
                // The first still-valid entry of the same tile is struck.
                let q = v.queue();
                for i in 0..q.count {
                    let (qi, qf) = q.slot(i);
                    if qi == index && (qf >> 8) & 0xff == 1 {
                        q.put(i, (qi, qf & !0xff00));
                        break;
                    }
                }
            } else {
                v.fill_cache();
                let n = v.road_changed(param != 0);
                if n < 2 {
                    if param != 0 {
                        ambience_scan(v, x, y);
                    }
                    if v.get_road_data() {
                        v.mark_and_trim(1);
                    }
                } else {
                    // The tile already stands: queue a removal and remember
                    // the add.
                    let w = v.world_cells().0;
                    let packed = x.wrapping_add(w.wrapping_mul(y).wrapping_mul(4));
                    push(v, Which::Queue, packed, 0x100);
                    push(v, Which::Remove, index, flags);
                }
            }
        }
        v.queue().iter_next();
        if !v.queue().at_end() {
            cur = v.queue().cur();
        }
    }
    // ---- the adds pass one asked to take back come out of the queue
    v.remove_list().iter_start();
    if !v.remove_list().at_end() {
        cur = v.remove_list().cur();
    }
    if !v.remove_list().at_end() {
        loop {
            let q = v.queue();
            let mut found = -1;
            for i in 0..q.count {
                let (qi, qf) = q.slot(i);
                if qi == cur.0 && qf & 0xff == cur.1 & 0xff {
                    found = i;
                    break;
                }
            }
            if found != q.count {
                // A shift down one, and a count one less: from slot 0 when
                // nothing matched, which writes below the buffer.
                let mut j = found.wrapping_add(1);
                while j < q.count {
                    if j >= 1 {
                        let e = q.slot(j);
                        q.put(j - 1, e);
                    }
                    j += 1;
                }
                q.count = q.count.wrapping_sub(1);
            }
            let r = v.remove_list();
            r.iter_next();
            if r.at_end() {
                break;
            }
            cur = r.cur();
        }
    }
    // ---- pass two: directions, `set_diags`, the marks
    v.queue().iter_start();
    if !v.queue().at_end() {
        cur = v.queue().cur();
    }
    while !v.queue().at_end() {
        let (index, flags) = cur;
        if (flags >> 8) & 0xff != 0 {
            let stride = v.world_cells().0.wrapping_shl(2);
            v.set_tile(index % stride, index / stride);
            if v.get_road_data() {
                if flags & 0xff == 0 {
                    v.mark_and_trim(1);
                }
                v.fill_cache();
                if v.set_diags(1, param) == 0 {
                    v.fill_cache();
                    v.mark_splits();
                    v.mark_white_lines();
                    v.mark_yellow_lines();
                    v.mark_intersections();
                    v.fixup_lines(1);
                }
            }
        }
        v.queue().iter_next();
        if !v.queue().at_end() {
            cur = v.queue().cur();
        }
    }
    // ---- pass three: the pieces
    v.queue().iter_start();
    if !v.queue().at_end() {
        cur = v.queue().cur();
    }
    while !v.queue().at_end() {
        let (index, flags) = cur;
        if (flags >> 8) & 0xff != 0 {
            let stride = v.world_cells().0.wrapping_shl(2);
            v.set_tile(index % stride, index / stride);
            if v.get_road_data() {
                v.fixup_lines(0);
                // A texture bit pair is dropped when its cardinal is not
                // claimed: four tests, each re-reading the dword.
                for (any, claimed, keep) in [
                    (0x00c0_c000, 0x4000_0000, 0xff3f_3fff),
                    (0x0030_3000, 0x1000_0000, 0xffcf_cfff),
                    (0x000c_0c00, 0x0400_0000, 0xfff3_f3ff),
                    (0x0003_0300, 0x0100_0000, 0xfffc_fcff),
                ] {
                    let f = v.elem_flags();
                    if f & any != 0 && f & claimed == 0 {
                        v.set_elem_flags(f & keep);
                    }
                }
                v.find_piece();
            }
        }
        v.queue().iter_next();
        if !v.queue().at_end() {
            cur = v.queue().cur();
        }
    }
    // ---- the redo ping-pong
    let mut flip = false;
    let mut rounds = 0;
    let (mut which, mut arg) = (0usize, 2);
    let mut n = v.redo_len(0);
    while n != 0 {
        rounds += 1;
        v.redo_changed_roads(which, arg, param);
        flip = !flip;
        v.redo_clear(which);
        (which, arg) = if flip { (1, 1) } else { (0, 2) };
        if rounds > 10 {
            break;
        }
        n = v.redo_len(which);
    }
    v.finish();
}

/// The add arm's scan under a non-zero `param`: of the eight tiles around
/// the one just added, the first that has terrain data and is clear of
/// flags, with no neighbouring cell already marked, and that has a bare tile
/// beside it, gets an ambience event and its cell a mark — and the scan ends.
fn ambience_scan<V: RoadsView>(v: &mut V, x: i32, y: i32) {
    for d in 1..=8 {
        let (nx, ny) = (x + MOVE_X[d], y + MOVE_Y[d]);
        let (tw, th) = v.world_tiles();
        if nx < 0 || ny < 0 || nx >= tw || ny >= th {
            continue;
        }
        let (cx, cy) = (nx >> 2, ny >> 2);
        if !v.cell_has_tiles(cx, cy) {
            continue;
        }
        match v.tile_rec(nx, ny) {
            Some(r) if r.kind != b'\n' && r.a != 0 && r.b == 0 => {}
            _ => continue,
        }
        if v.tile_flags(nx, ny) & 0x100 != 0 {
            continue;
        }
        let marked = (1..=8).any(|k| {
            let (ax, ay) = (cx + MOVE_X[k], cy + MOVE_Y[k]);
            let (wc, hc) = v.world_cells();
            ax >= 0 && ay >= 0 && ax < wc && ay < hc && v.cell_flags(ax, ay) & 0x800 != 0
        });
        if marked {
            continue;
        }
        for k in 1..=8 {
            let (kx, ky) = (nx + MOVE_X[k], ny + MOVE_Y[k]);
            let (tw, th) = v.world_tiles();
            if kx < 0 || ky < 0 || kx >= tw || ky >= th {
                continue;
            }
            if v.tile_flags(kx, ky) & 0x4000 != 0 {
                continue;
            }
            let bare = !v.cell_has_tiles(kx >> 2, ky >> 2)
                || v.tile_rec(kx, ky).is_none_or(|r| r.kind == b'\n');
            if bare {
                v.add_ambience(kx, ky);
                v.set_cell_flags(kx >> 2, ky >> 2, 0x800);
                return;
            }
        }
    }
}
