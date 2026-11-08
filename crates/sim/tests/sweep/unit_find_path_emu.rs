//! The emulated original's windows as `Unit::find_path`'s view (item 3386): one `V` that the two
//! sweeps' ports share (`unit_find_path`: varied positions, one entry pushed by
//! `go_around_building`; `unit_find_path_pushes`: a fixed position and a short list of coordinates,
//! three entries pushed, so the detour's equality tests meet).

use crate::Pos;
use crate::find_path_view::{FindPathView, PathEntry, find_path_steps};
use crate::movement::{Angle, cos_component, find_angle, sin_component};

fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

pub(crate) fn run(
    a: &[i32],
    m: &mut [u8],
    c: &[Vec<u8>],
    p: &mut [Vec<u8>],
    log: &mut Vec<String>,
    pushes: i32,
) -> i64 {
    // **The port is the crate's own steps** (`crate::find_path_view::find_path_steps`, which
    // `Sim::find_path` runs over its `SimFindPath`); the view is the emulated original's windows.
    // Windows: `m` the unit (`+0x10`/`+0x14` the position xor `0x63637`, `+0x68` its masks,
    // `+0xaf` the recursion byte, `+0xb8` the path stack: list, size, length), `c0` the world
    // (`+0x18` its width), `c1` `div_3_table`, `p0` the unit's type (`+0x218` domain, `+0x2b4`
    // flags), `p1` the path's list (16 bytes an entry), `p2` the tile masks, `p3` what
    // `update_order` answers, `p4` the move order its slot answers (`+0x2c` waypoint, `+0x34`
    // last plan). Row: `a[0..2]` the goal, `a[2 + i]` stub `i`'s answer (stubs 0 to 10: the
    // `invalid_loc`, `find_nearby_spot`, `set_new_location`, `update_order`, order slot `+0x40`,
    // unit slot `+0x17c` (the step), `go_around_building`, the recursive `find_path`, then
    // `malloc`, `free`, `memcpy`), `a[13]`/`a[14]` the spot `find_nearby_spot` writes, and
    // `a[15..45]` the thirty `invalid_loc` answers in call order. The last argument of
    // `invalid_loc` is a leftover register the callee never reads: it is not compared.
    const K: i32 = 0x63637;
    struct V<'a> {
        a: &'a [i32],
        m: &'a mut [u8],
        c: &'a [Vec<u8>],
        p: &'a mut [Vec<u8>],
        log: &'a mut Vec<String>,
        asked: usize,
        pushes: i32,
    }
    impl V<'_> {
        fn div3(&self, i: i32) -> i32 {
            rd(&self.c[1], 4 * i as usize)
        }
        fn tile(&self, at: Pos) -> Pos {
            Pos::new(self.div3(at.x >> 6), self.div3(at.y >> 6))
        }
        fn raw_here(&self) -> Pos {
            Pos::new(rd(self.m, 0x10) ^ K, rd(self.m, 0x14) ^ K)
        }
        fn mask(&self, at: Pos) -> bool {
            let t = self.tile(at);
            let idx = rd(&self.c[0], 0x18).wrapping_mul(t.y).wrapping_add(t.x);
            let o = 2 * idx as usize;
            u16::from_le_bytes([self.p[2][o], self.p[2][o + 1]]) & 0x30 == 0x20
        }
        fn entry(&self, i: usize) -> PathEntry {
            let w = &self.p[1][16 * i..16 * i + 16];
            PathEntry {
                to: Pos::new(rd(w, 0), rd(w, 4)),
                tolerance: rd(w, 8),
                flags: rd(w, 12) as u32,
            }
        }
    }
    impl FindPathView for V<'_> {
        fn helicopter(&mut self) -> bool {
            self.p[0][0x2b4] & 0x20 != 0
        }
        fn here(&mut self) -> Pos {
            self.raw_here()
        }
        fn invalid_loc(&mut self, at: Pos, ib: bool, fr: bool) -> bool {
            let t = self.tile(at);
            self.log.push(format!(
                "s0:m+0x0,{},{},{},{},0,0,0",
                t.x,
                t.y,
                i32::from(ib),
                i32::from(fr)
            ));
            let r = self.a[15 + self.asked.min(29)];
            self.asked += 1;
            r != 0
        }
        fn relocate(&mut self) {
            let h = self.raw_here();
            let id = i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]));
            self.log.push(format!(
                "s1:p0+0x0,{},{},sp-0x4c,sp-0x50,0,-1,0,1431655765,3,{},{},0,0,-1,0,-1",
                h.x, h.y, id, self.m[9]
            ));
            self.log
                .push(format!("s2:m+0x0,{},{},1,0", self.a[13], self.a[14]));
        }
        fn order_begin(&mut self) {
            self.log.push("s3:m+0x0".into());
            self.log.push("s4:p3+0x0".into());
        }
        fn cells_apart(&mut self, to: Pos) -> i32 {
            let h = self.raw_here();
            (self.div3(h.x >> 8) - self.div3(to.x >> 8)).abs()
                + (self.div3(h.y >> 8) - self.div3(to.y >> 8)).abs()
        }
        fn speed(&mut self) -> i32 {
            let h = self.raw_here();
            self.log.push(format!("s5:m+0x0,{},{},0", h.x, h.y));
            self.a[7]
        }
        fn angle(&mut self, dx: i32, dy: i32) -> i32 {
            find_angle(dx, dy).0
        }
        fn sin(&mut self, angle: i32, distance: i32) -> i32 {
            sin_component(Angle(angle), distance)
        }
        fn cos(&mut self, angle: i32, distance: i32) -> i32 {
            cos_component(Angle(angle), distance)
        }
        fn bump_recursion(&mut self) {
            self.m[0xaf] = self.m[0xaf].saturating_add(1);
        }
        fn recursion(&mut self) -> u8 {
            self.m[0xaf]
        }
        fn waypoint(&mut self) -> Pos {
            Pos::new(rd(&self.p[4], 0x2c), rd(&self.p[4], 0x30))
        }
        fn set_waypoint(&mut self, to: Pos) {
            wr(&mut self.p[4], 0x2c, to.x);
            wr(&mut self.p[4], 0x30, to.y);
        }
        fn last_plan(&mut self) -> Pos {
            Pos::new(rd(&self.p[4], 0x34), rd(&self.p[4], 0x38))
        }
        fn set_last_plan(&mut self, to: Pos) {
            wr(&mut self.p[4], 0x34, to.x);
            wr(&mut self.p[4], 0x38, to.y);
        }
        fn path_len(&mut self) -> i32 {
            rd(self.m, 0xc0)
        }
        fn path_pop(&mut self) -> Option<PathEntry> {
            // `Stack::pop`: the length is floored at 1, then lowered; the slot is
            // read whatever it holds (a stale one on an empty stack).
            let n = rd(self.m, 0xc0).max(1) - 1;
            wr(self.m, 0xc0, n);
            Some(self.entry(n as usize))
        }
        fn path_push(&mut self, e: PathEntry) {
            let n = rd(self.m, 0xc0);
            assert!(rd(self.m, 0xbc) > n, "the push would grow the stack");
            let w = &mut self.p[1][16 * n as usize..16 * n as usize + 16];
            wr(w, 0, e.to.x);
            wr(w, 4, e.to.y);
            wr(w, 8, e.tolerance);
            wr(w, 12, e.flags as i32);
            wr(self.m, 0xc0, n + 1);
        }
        fn path_peek(&mut self) -> PathEntry {
            let n = rd(self.m, 0xc0);
            self.entry(if n > 0 { n as usize - 1 } else { 0 })
        }
        fn set_avoid(&mut self, at: Pos) {
            wr(self.m, 0x120, at.x);
            wr(self.m, 0x124, at.y);
        }
        fn domain(&mut self) -> i32 {
            rd(&self.p[0], 0x218)
        }
        fn go_around(&mut self, at: Pos, angle: i32, speed: i32) {
            self.log
                .push(format!("s6:m+0x0,{},{},{angle},{speed}", at.x, at.y));
            let n = rd(self.m, 0xc0);
            wr(self.m, 0xc0, n.wrapping_add(self.pushes));
            self.m[0xaf] = self.m[0xaf].wrapping_add(1);
        }
        fn recurse(&mut self, to: Pos) -> i32 {
            self.log.push(format!("s7:m+0x0,{},{}", to.x, to.y));
            self.a[9]
        }
        fn line_ok(&mut self) -> bool {
            rd(self.m, 0x68) & 8 != 0
        }
        fn set_line_ok(&mut self) {
            let v = rd(self.m, 0x68) | 8;
            wr(self.m, 0x68, v);
        }
        fn shore_between(&mut self, a: Pos, b: Pos) -> bool {
            self.mask(a) != self.mask(b)
        }
    }
    let mut v = V {
        a,
        m,
        c,
        p,
        log: &mut *log,
        asked: 0,
        pushes,
    };
    let r = i64::from(find_path_steps(&mut v, Pos::new(a[0], a[1])));
    if log.is_empty() {
        log.push("-".into());
    }
    r
}
