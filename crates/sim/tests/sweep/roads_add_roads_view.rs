//! The emulated original's windows as `Roads::add_roads`' view (item 3376): the one port the
//! `roads_add_roads`, `roads_add_roads_redo` and `roads_add_roads_amb` sweeps share (hand-written;
//! `sweep_gen.py --regen` redraws each test's body and keeps this file). The three differ only in
//! their draw: the queue sweep keeps `param` at 0, the redo sweep makes every `redo_changed_roads`
//! call raise *both* lists' counts so the ping-pong runs to its eleventh round, the ambience sweep
//! keeps `param` at 1 over a two-by-two world of drawn cells.
//!
//! Windows. `m` the `Roads` (`+0xe4` the queue's count, `+0xe8` its capacity, `+0xec` the step
//! `increase_size` is handed, `+0xf0` the buffer, `+0xf4` the iterator's flag byte, `+0xf8` its
//! cursor, `+0x120`/`+0x124` the tile cursor, `+0x14c` the element, `+0x154`/`+0x170` the two redo
//! lists' counts); `c0` the world (`+0`/`+4` cells, `+0x18`/`+0x1c` tiles, `+0x138` the tile flag
//! words), `c1` the terrain (`+0x6ab0` the cell table), `c2` the const world (`+0` cells wide),
//! `c3` `_TSS0`, `c4` the function-static `remove_list`'s head (`+4` count, `+8` capacity, `+0xc`
//! step), `c5` its iterator (`+0` flag byte, `+4` cursor); `p0` the cell table, `p1..p4` the cells
//! (`+0x1c` flags, `+0x5c` tiles, `+0x66` has tiles), `p5` the tile flag words, `p6` the queue's
//! buffer, `p7` the remove list's, `p8`/`p9` the thread-local chain the `_TSS0` guard reads (the
//! init arm is never run), `p10..p13` the cells' tile records (sixteen bytes each: `+8` kind, `+10`,
//! `+11`), `p14 + k` the element the k-th `get_road_data` answers (its `+4` the flags).
//!
//! `a`: the argument, the stubs' answers (`a[1..16]`: s0 `clear_roads`, s1 `fill_cache`, s2
//! `road_changed`, s3 `get_road_data`, s4 `mark_and_trim_directions`, s5 `set_diags`, s6
//! `mark_splits`, s7 `mark_white_lines`, s8 `mark_yellow_lines`, s9 `mark_intersections`, s10
//! `fixup_lines`, s11 `find_piece`, s12 `redo_changed_roads`, s13 `increase_size`, s14
//! `add_ambience`), then one null flag a `get_road_data` call (`a[16 + k]`).
//!
//! **The port is the crate's own steps**, `crate::roads_view::add_roads_steps`, which `Sim::add_roads`
//! runs over its `SimRoads`; here the view is the emulated original's windows: the queue and the
//! remove list are loaded from them and written back, and every callee is a record.

use crate::roads_view::{ModQueue, RoadsView, TileRec, Which, add_roads_steps};

const ELEM0: usize = 14;
const ELEMS: usize = 24;

fn rd32(w: &[u8], o: usize) -> i32 {
    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
}

fn rd16(w: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([w[o], w[o + 1]])
}

fn wr32(w: &mut [u8], o: usize, v: i32) {
    w[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

fn load(buf: &[u8], count: i32, cap: i32, grow: i16, cursor: i32, flags: u8) -> ModQueue {
    ModQueue {
        data: (0..buf.len() / 8)
            .map(|i| (rd32(buf, 8 * i), rd32(buf, 8 * i + 4)))
            .collect(),
        count,
        cap,
        grow,
        cursor,
        flags,
    }
}

fn store(q: &ModQueue, buf: &mut [u8]) {
    for (i, &(a, b)) in q.data.iter().enumerate() {
        if 8 * i + 8 <= buf.len() {
            wr32(buf, 8 * i, a);
            wr32(buf, 8 * i + 4, b);
        }
    }
}

struct V<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a mut [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: Vec<String>,
    queue: ModQueue,
    remove: ModQueue,
    tile: (i32, i32),
    /// The window the cursor's element lies in.
    elem: Option<usize>,
    /// `get_road_data` calls so far.
    reads: usize,
    /// `+0x150`'s and `+0x16c`'s counts.
    redo: [i32; 2],
    /// The redo stub raises both counts a call, not only the second's.
    redo_both: bool,
}

impl V<'_> {
    fn rec(&mut self, k: usize, ecx: &str, args: &[i32]) {
        let mut t = format!("s{k}:{ecx}");
        for v in args {
            t.push_str(&format!(",{v}"));
        }
        self.log.push(t);
    }

    /// The cell `(cx, cy)`'s index in the table `terrain +0x6ab0` names: `p1 + that`.
    fn cell(&self, cx: i32, cy: i32) -> usize {
        let wc = rd32(&self.c[2], 0);
        let i = wc.wrapping_mul(cy).wrapping_add(cx);
        1 + usize::try_from(i).expect("a cell of the table")
    }
}

impl RoadsView for V<'_> {
    fn queue(&mut self) -> &mut ModQueue {
        &mut self.queue
    }
    fn remove_list(&mut self) -> &mut ModQueue {
        &mut self.remove
    }
    fn grow(&mut self, which: Which, step: i16) {
        let ecx = match which {
            Which::Queue => "m+0xe0",
            // `0xee19f4` lies one past `_TSS0`'s four bytes, which the record names as that window's `+0x4`.
            Which::Remove => "c3+0x4",
        };
        self.rec(13, ecx, &[i32::from(step)]);
    }
    fn world_cells(&self) -> (i32, i32) {
        (rd32(&self.c[0], 0), rd32(&self.c[0], 4))
    }
    fn world_tiles(&self) -> (i32, i32) {
        (rd32(&self.c[0], 0x18), rd32(&self.c[0], 0x1c))
    }
    fn set_tile(&mut self, x: i32, y: i32) {
        self.tile = (x, y);
        wr32(self.m, 0x120, x);
        wr32(self.m, 0x124, y);
    }
    fn clear_roads(&mut self, _index: i32, _flags: i32) {
        self.rec(0, "m+0x0", &[]);
        let t = self.log.last_mut().expect("just pushed");
        t.push_str(",sp-0x4c");
        // `clear_roads` queues the road neighbours it leaves on the first redo list; the stub raises
        // that list's count by one.
        self.redo[0] += 1;
    }
    fn fill_cache(&mut self) {
        let (x, y) = self.tile;
        self.rec(1, "m+0x0", &[x, y]);
    }
    fn road_changed(&mut self, flag: bool) -> i32 {
        let (x, y) = self.tile;
        self.rec(2, "c1+0x0", &[x, y, 1, 0, i32::from(flag)]);
        self.a[3]
    }
    fn get_road_data(&mut self) -> bool {
        let (x, y) = self.tile;
        self.rec(3, "c1+0x0", &[x, y, 0, 0, 0]);
        let k = self.reads.min(ELEMS - 1);
        self.reads += 1;
        let null = self.a[16 + k] != 0;
        self.elem = if null { None } else { Some(ELEM0 + k) };
        // The window's own address: the port cannot see it, so the row's check holds this word
        // to null or not and nothing else.
        wr32(self.m, 0x14c, i32::from(!null));
        !null
    }
    fn mark_and_trim(&mut self, list: i32) {
        self.rec(4, "m+0x0", &[list]);
    }
    fn set_diags(&mut self, list: i32, param: i32) -> i32 {
        self.rec(5, "m+0x0", &[list, param]);
        // `road_added` under it appends to the queue: the entry is the buffer's own next slot.
        self.queue.count += 1;
        self.a[6]
    }
    fn mark_splits(&mut self) {
        self.rec(6, "m+0x0", &[1]);
    }
    fn mark_white_lines(&mut self) {
        self.rec(7, "m+0x0", &[1]);
    }
    fn mark_yellow_lines(&mut self) {
        self.rec(8, "m+0x0", &[1]);
    }
    fn mark_intersections(&mut self) {
        // Its argument is the `ecx` the call before left, which the stub leaves as `this`.
        self.log.push("s9:m+0x0,m+0x0".into());
    }
    fn fixup_lines(&mut self, arg: i32) {
        self.rec(10, "m+0x0", &[arg]);
    }
    fn elem_flags(&self) -> u32 {
        rd32(&self.p[self.elem.expect("an element")], 4) as u32
    }
    fn set_elem_flags(&mut self, flags: u32) {
        let k = self.elem.expect("an element");
        wr32(&mut self.p[k], 4, flags as i32);
    }
    fn find_piece(&mut self) {
        self.rec(11, "m+0x0", &[]);
    }
    fn redo_len(&self, which: usize) -> i32 {
        self.redo[which]
    }
    fn redo_changed_roads(&mut self, which: usize, arg: i32, param: i32) {
        let list = if which == 0 { "m+0x150" } else { "m+0x16c" };
        self.rec(12, "m+0x0", &[]);
        let t = self.log.last_mut().expect("just pushed");
        t.push_str(&format!(",{list},{arg},{param}"));
        // The stub raises the second list's count by one each call, the first's too when asked.
        self.redo[1] += 1;
        if self.redo_both {
            self.redo[0] += 1;
        }
    }
    fn redo_clear(&mut self, which: usize) {
        self.redo[which] = 0;
    }
    fn tile_flags(&self, x: i32, y: i32) -> u16 {
        let (tw, _) = self.world_tiles();
        let i = usize::try_from(tw.wrapping_mul(y).wrapping_add(x)).expect("a tile of the table");
        rd16(&self.p[5], 2 * i)
    }
    fn cell_has_tiles(&self, cx: i32, cy: i32) -> bool {
        self.p[self.cell(cx, cy)][0x66] != 0
    }
    fn cell_flags(&self, cx: i32, cy: i32) -> u16 {
        rd16(&self.p[self.cell(cx, cy)], 0x1c)
    }
    fn set_cell_flags(&mut self, cx: i32, cy: i32, bits: u16) {
        let k = self.cell(cx, cy);
        let w = rd16(&self.p[k], 0x1c) | bits;
        self.p[k][0x1c..0x1e].copy_from_slice(&w.to_le_bytes());
    }
    fn tile_rec(&self, x: i32, y: i32) -> Option<TileRec> {
        let k = self.cell(x >> 2, y >> 2);
        // The cell's tile array is the place that follows the cells.
        let rec = &self.p[10 + (k - 1)];
        let at = 16 * usize::try_from((y & 3) * 4 + (x & 3)).expect("a tile of the cell");
        Some(TileRec {
            kind: rec[at + 8],
            a: rec[at + 10],
            b: rec[at + 11],
        })
    }
    fn add_ambience(&mut self, x: i32, y: i32) {
        let (px, py) = (x * 0xc0 + 0x60, y * 0xc0 + 0x60);
        self.rec(14, &py.to_string(), &[0, px, py, 0, -1, -1]);
    }
    fn finish(&mut self) {
        self.redo = [0, 0];
        self.queue.count = 0;
    }
}

/// One row: the port over the row's windows. `after` is the row's after-state, of which only
/// the element pointer's address is read (the port cannot see addresses).
pub(super) fn run_row(
    a: &[i32],
    m: &mut [u8],
    c: &mut [Vec<u8>],
    p: &mut [Vec<u8>],
    log: &mut Vec<String>,
    after: &str,
    redo_both: bool,
) {
    let queue = load(
        &p[6],
        rd32(m, 0xe4),
        rd32(m, 0xe8),
        i16::from_le_bytes([m[0xec], m[0xed]]),
        rd32(m, 0xf8),
        m[0xf4],
    );
    let remove = load(
        &p[7],
        rd32(&c[4], 4),
        rd32(&c[4], 8),
        i16::from_le_bytes([c[4][0xc], c[4][0xd]]),
        rd32(&c[5], 4),
        c[5][0],
    );
    let redo = [rd32(m, 0x154), rd32(m, 0x170)];
    let mut v = V {
        a,
        m,
        c,
        p,
        log: Vec::new(),
        queue,
        remove,
        tile: (0, 0),
        elem: None,
        reads: 0,
        redo,
        redo_both,
    };
    add_roads_steps(&mut v, a[0]);
    log.extend(std::mem::take(&mut v.log));
    if log.is_empty() {
        log.push("-".into());
    }
    let (queue, remove, redo) = (v.queue.clone(), v.remove.clone(), v.redo);
    let (m, c, p) = (v.m, v.c, v.p);
    store(&queue, &mut p[6]);
    store(&remove, &mut p[7]);
    wr32(m, 0xe4, queue.count);
    m[0xf4] = queue.flags;
    wr32(m, 0xf8, queue.cursor);
    wr32(m, 0x154, redo[0]);
    wr32(m, 0x170, redo[1]);
    wr32(&mut c[4], 4, remove.count);
    c[5][0] = remove.flags;
    wr32(&mut c[5], 4, remove.cursor);
    // The element pointer is an address in the original; the port holds it to null or not.
    let want_m = (0..m.len())
        .map(|i| {
            let s = after.trim().split(';').next().expect("this");
            u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex")
        })
        .skip(0x14c)
        .take(4)
        .collect::<Vec<u8>>();
    let w = rd32(&want_m, 0);
    assert_eq!(
        w != 0,
        rd32(m, 0x14c) != 0,
        "add_roads — the element pointer's nullness"
    );
    wr32(m, 0x14c, w);
}
