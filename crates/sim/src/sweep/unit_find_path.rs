//! `Unit::find_path@005fb910` under the emulator against the port (item 3386; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `Unit::find_path@005fb910`, run by `tools/emu/sweep_unit_find_path.py` under unicorn
/// (tools/emu): 720 random bytes of object, 2 argument(s), `p` windows in order: p0..p2 the place(s), in flag order, p3..p4 the stubs' answer window(s); the global cell 0x00c06188 points at 512 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; callee 0x00607c30 (8 args) stubbed; callee 0x0061de70 (16 args) stubbed; callee 0x005f8d20 (4 args) stubbed; callee 0x006179d0 (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; vtable slot +0x17c (3 args) stubbed; callee 0x005fc350 (4 args) stubbed; callee 0x005fb910 (2 args) stubbed; the import cell 0x00ac54f0 holds a stub (1 args); the import cell 0x00ac5500 holds a stub (1 args); callee 0x0055e0ac (3 args) stubbed; the word at `m+0x18` points at 704 random bytes less `0` (placed); the word at `m+0xb8` points at 1024 random bytes less `0` (placed); the word at `c0+0x138` points at 2048 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 0, 0, 0, 0, 0, 0, 1] (90%); stub 7 answered from [0, 0, 0, 1, 2] (90%); stub 3 answers the address of 64 random bytes, a `p` window after the places; stub 4 answers the address of 64 random bytes, a `p` window after the places; stub 0 answers 30 values in call order, the last from [0], printed after the answers; stub 5 answers 0..200 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 1 writes a value in 3000..4200 through its argument 2, printed after the answers (`o` in a place); stub 1 writes a value in 3000..4200 through its argument 3, printed after the answers (`o` in a place); stub 8 is `__cdecl`: the caller pops its arguments; stub 9 is `__cdecl`: the caller pops its arguments; stub 10 is `__cdecl`: the caller pops its arguments; `0x00a46980` run once before the rows (a table builder the startup would have run); stub 8 answers a fresh bump-heap address of its argument 0 bytes (a `malloc`); stub 10 copies its argument 2 bytes from argument 1 to argument 0 (a `memcpy`); stub 6 writes inc (4 bytes) at `m+0xc0` each call; stub 6 writes inc (1 bytes) at `m+0xaf` each call; the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_find_path.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        // `invalid_loc`'s eighth argument is a register left over from the caller; the callee never
        // reads it (`param_8` is unused), so it is not compared.
        let stripped: Vec<String> = calls
            .split_whitespace()
            .map(|t| match t.strip_prefix("s0:") {
                Some(r) => format!("s0:{}", r.rsplit_once(',').expect("eight arguments").0),
                None => t.to_string(),
            })
            .collect();
        let want_log: Vec<&str> = stripped.iter().map(String::as_str).collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
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
            use crate::find_path_view::{FindPathView, PathEntry, find_path_steps};
            use crate::movement::{Angle, cos_component, find_angle, sin_component};
            use crate::Pos;
            const K: i32 = 0x63637;
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                asked: usize,
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
                    wr(self.m, 0xc0, n.wrapping_add(1));
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
                log: &mut log,
                asked: 0,
            };
            let r = i64::from(find_path_steps(&mut v, Pos::new(a[0], a[1])));
            if log.is_empty() {
                log.push("-".into());
            }
            r
        };
        assert_eq!(got, want_eax, "find_path — eax, row `{line}`");
        assert_eq!(log, want_log, "find_path — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        let parts: Vec<String> = (0..m.len())
            .filter(|&i| m[i] != want[0][i])
            .map(|i| format!("m+{i:#x}: got {:02x} want {:02x}", m[i], want[0][i]))
            .collect();
        assert!(
            parts.is_empty(),
            "find_path — this's bytes {parts:?}, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_path — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "find_path — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::find_path@005fb910: {n} rows agree");
}
