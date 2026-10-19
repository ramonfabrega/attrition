//! `Objects::add_flock@0065c0e0` under the emulator against the port (item 3320; tools/sweep_gen.py, obj with windows).

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

/// `Objects::add_flock@0065c0e0`, run by `tools/emu/sweep_objects_add_flock.py` under unicorn
/// (tools/emu): 8 random bytes of object, 4 argument(s), `p` windows in order: p0..p5 sparse table 0 (3 rows × 2 slice(s), row-major), p6..p28 the place(s), in flag order, p29..p44 the stubs' answer window(s); the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c06184 points at 16 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c0618c points at 32 random bytes; 4 random bytes at 0x00c8da98 (each word in -2..4); 4 random bytes at 0x00cbe33c (each word in -1..7); 4 random bytes at 0x00cbe350 (each word in 0..12); 16 random bytes at 0x00cb7e90 (each byte in -4..4); 16 random bytes at 0x00cbb0e0 (each byte in -4..4); 320 random bytes at 0x00e90000; callee 0x00a39d70 (2 args) stubbed; callee 0x0065e0c0 (7 args) stubbed; callee 0x00605400 (3 args) stubbed; callee 0x00606590 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x58 (0 args) stubbed; the import cell 0x00ac54f0 holds a stub (1 args); callee 0x0055d14a (5 args) stubbed; callee 0x0060a870 (0 args) stubbed; the import cell 0x00ac5500 holds a stub (1 args); the word at `0xc8da90` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 16 random bytes less `0` (placed); the word at `p6+4` points at 16 random bytes less `0` (placed); the word at `p6+8` points at 16 random bytes less `0` (placed); the word at `p6+12` points at 16 random bytes less `0` (placed); the word at `0xc0afbc` points at 12 random bytes less `0` (placed); the word at `p11+0` points at 256 random bytes less `0` (placed); the word at `p11+4` points at 256 random bytes less `0` (placed); the word at `p11+8` points at 256 random bytes less `0` (placed); the word at `c2+0x134` points at 4032 random bytes less `0` (placed); the word at `p12+0xf4` points at 4 random bytes less `0` (placed); the word at `p13+0xf4` points at 4 random bytes less `0` (placed); the word at `p14+0xf4` points at 4 random bytes less `0` (placed); the word at `p16+0` points at 128 random bytes less `0` (placed); the word at `p17+0` points at 128 random bytes less `0` (placed); the word at `p18+0` points at 128 random bytes less `0` (placed); the word at `p12+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p13+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p14+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p22+4` points at 16 random bytes less `0` (placed); the word at `p23+4` points at 16 random bytes less `0` (placed); the word at `p24+4` points at 16 random bytes less `0` (placed); the word at `p12+0xf8` points at 8 random bytes less `0` (placed); stub 4 answers a window of its own to each call, in call order (4 bytes, ...); stub 5 answers a window of its own to each call, in call order (128 bytes, ...); the word at `p22+4` laid as the address of `p22` half the rows; the word at `p23+4` laid as the address of `p23` half the rows; the word at `p24+4` laid as the address of `p24` half the rows; the word at `p37+0` laid as the address of `p28` every row; the word at `p38+0` laid as the address of `p28` every row; the word at `p39+0` laid as the address of `p28` every row; the word at `p40+0` laid as the address of `p28` every row; the word at `p41+0` laid as the address of `p28` every row; the word at `p42+0` laid as the address of `p28` every row; the word at `p43+0` laid as the address of `p28` every row; the word at `p44+0` laid as the address of `p28` every row; stub 0 answers 40 values in call order, printed after the answers; stub 1 answers 3 values in call order, printed after the answers; stub 6 answers 9 values in call order, printed after the answers; stub 0 answers 0..65535 every row; stub 1 answers -1..2 every row; stub 6 answers 15269888..15270144 step 0x20 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 6 is `__cdecl`: the caller pops its arguments; stub 9 is `__cdecl`: the caller pops its arguments; 3 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x7b8:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 700 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_add_flock.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(10);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The row: `a[0..4]` the tile `x`, `y`, `who`, `n`; `a[4 + i]` stub `i`'s answer, the stubs being
            // 0 `Random::get`, 1 `init_unit`, 2 `Unit::set_angle`, 3 `update_z`, 4 `get_obj`, 5 the pool
            // object's slot `+0x58`, 6 `malloc` (list, then nodes), 7 `_eh_vector_constructor_iterator_`,
            // 8 `update_action`, 9 `free`; then the `--seq` answers: `a[14..54]` the forty rolls, `a[54..57]`
            // `init_unit`'s answers, `a[57..66]` `malloc`'s (all `0xe90000 + 0x20 k`, the first the `WCoordList`'s
            // storage).
            //
            // Windows: `c[0]` the game (`+0x550` the frame), `c[1]` the random, `c[2]` the world (`+0` width,
            // `+4` height, `+0x134` the cells), `c[3]` the objects, `c[4]` the recycler's length, `c[5]` and
            // `c[6]` `circle_radius[3]`, `[8]`, `c[7]`, `c[8]` `circle_x`, `circle_y`, `c[9]` the heap malloc
            // answers from. `p[0..6]` the leaders' flag and stamp words, `p[6]` the recycler's list (`p[7..11]`
            // its nodes), `p[11]` the units array (`p[12..15]` the units), `p[15]` the cells, `p[16..19]` the
            // units' `+0xf4` records (`p[19..22]` the figures they name), `p[22..25]` the units' ring heads
            // (`p[25..28]` the nodes before them), `p[28]` the order base table, `p[29..37]` the pool objects
            // `get_obj` answers, `p[37..45]` the orders its slot answers.
            //
            // **The port is the crate's own steps**, `crate::flock::add_flock_steps`; the view is the emulated
            // original's windows. The ring is the original's circular `LinkListBase` laid in the windows (and in
            // `c[9]`, byte-addressed, since a node, a list and a node may overlap there), the order's fields what
            // `Objects::add_flock` stores in line.
            use crate::flock::{FlockView, add_flock_steps};
            use crate::world::Pos;
            #[derive(Clone, Copy)]
            enum Win {
                C(usize),
                P(usize),
            }
            struct V<'a> {
                a: &'a [i32],
                c: &'a mut [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                /// Where an image address lies: `(base, size, window)`.
                ranges: Vec<(u32, u32, Win)>,
                rolls: usize,
                inits: usize,
                mallocs: usize,
                orders: usize,
                list: u32,
                pushed: usize,
            }
            impl V<'_> {
                fn at(&self, addr: u32) -> (Win, usize) {
                    for &(base, size, w) in &self.ranges {
                        if addr >= base && addr - base < size {
                            return (w, (addr - base) as usize);
                        }
                    }
                    panic!("address {addr:#x} lies in no window");
                }
                fn win(&mut self, w: Win) -> &mut Vec<u8> {
                    match w {
                        Win::C(i) => &mut self.c[i],
                        Win::P(i) => &mut self.p[i],
                    }
                }
                fn r8(&mut self, addr: u32) -> u8 {
                    let (w, o) = self.at(addr);
                    self.win(w)[o]
                }
                fn w8(&mut self, addr: u32, v: u8) {
                    let (w, o) = self.at(addr);
                    self.win(w)[o] = v;
                }
                fn r32(&mut self, addr: u32) -> u32 {
                    u32::from_le_bytes([
                        self.r8(addr),
                        self.r8(addr + 1),
                        self.r8(addr + 2),
                        self.r8(addr + 3),
                    ])
                }
                fn w32(&mut self, addr: u32, v: u32) {
                    for (i, b) in v.to_le_bytes().into_iter().enumerate() {
                        self.w8(addr + i as u32, b);
                    }
                }
                fn seq(&self, base: usize, n: usize, k: usize) -> i32 {
                    self.a[base + k.min(n - 1)]
                }
            }
            impl FlockView for V<'_> {
                fn leader_live(&self, who: i32) -> bool {
                    self.p[2 * who as usize][0] & 1 != 0
                }
                fn stamp(&self, who: i32) -> i64 {
                    i64::from(rd(&self.p[2 * who as usize + 1], 0))
                }
                fn set_stamp(&mut self, who: i32, frame: i64) {
                    wr(&mut self.p[2 * who as usize + 1], 0, frame as i32);
                }
                fn frame(&self) -> i64 {
                    i64::from(rd(&self.c[0], 0x550))
                }
                fn roll(&mut self, _site: &'static str) -> i32 {
                    let v = self.seq(14, 40, self.rolls);
                    self.rolls += 1;
                    self.log.push("s0:c1+0x0,0,65535".into());
                    v
                }
                fn world_size(&self) -> (i32, i32) {
                    (rd(&self.c[2], 0), rd(&self.c[2], 4))
                }
                fn forest(&self, x: i32, y: i32) -> bool {
                    let w = rd(&self.c[2], 0);
                    self.p[15][((w * y + x) * 0x1c) as usize] & 0x20 != 0
                }
                fn push_cell(&mut self, x: i32, y: i32) {
                    if self.pushed == 0 {
                        // `increase_size`'s `malloc(4 × 8)`, `ecx` the size.
                        self.log.push("s6:32,32".into());
                        self.list = self.seq(57, 9, self.mallocs) as u32;
                        self.mallocs += 1;
                    }
                    let at = self.list + 8 * self.pushed as u32;
                    self.w32(at, x as u32);
                    self.w32(at + 4, y as u32);
                    self.pushed += 1;
                }
                fn cell(&self, k: usize) -> (i32, i32) {
                    let at = self.list + 8 * k as u32;
                    let (w, o) = self.at(at);
                    let win = match w {
                        Win::C(i) => &self.c[i],
                        Win::P(i) => &self.p[i],
                    };
                    (rd(win, o), rd(win, o + 4))
                }
                fn ring_radius(&self, k: usize) -> i32 {
                    rd(&self.c[if k == 3 { 5 } else { 6 }], 0)
                }
                fn ring_at(&self, i: usize) -> (i32, i32) {
                    (i32::from(self.c[7][i] as i8), i32::from(self.c[8][i] as i8))
                }
                fn make_bird(&mut self, at: Pos) -> Option<usize> {
                    self.log
                        .push(format!("s1:c3+0x0,9,403,{},{},-1,-1,-1", at.x, at.y));
                    let idx = self.seq(54, 3, self.inits);
                    self.inits += 1;
                    if idx < 0 {
                        return None;
                    }
                    Some(idx as usize)
                }
                fn turn(&mut self, b: usize, angle: i32) {
                    self.log
                        .push(format!("s2:p{}+0x0,{angle},c3+0x0,1", 12 + b));
                }
                fn update_z(&mut self, b: usize) {
                    self.log.push(format!("s3:p{}+0x0", 12 + b));
                }
                fn lift(&mut self, b: usize) {
                    let z = rd(&self.p[19 + b], 0x14);
                    wr(&mut self.p[19 + b], 0x70, z);
                    wr(&mut self.p[19 + b], 0x14, z.wrapping_add(200));
                }
                fn fly_to(&mut self, b: usize, at: Pos) {
                    let j = self.orders.min(7);
                    self.orders += 1;
                    // `get_obj`'s `ecx` is the figure's index × 4 (a stale register).
                    self.log.push(format!("s4:{},24", 4 * b));
                    self.log.push(format!("s5:p{}+0x0", 29 + j));
                    let order_at = (self.a[9] as u32).wrapping_sub(0xa0 * (7 - j) as u32);
                    let off = rd(&self.p[28], 4);
                    let o = &mut self.p[37 + j];
                    wr(o, 4, at.x);
                    wr(o, 8, at.y);
                    wr(o, 0x10, 0);
                    wr(o, 0xc, 0);
                    wr(o, 0x2c, 0);
                    wr(o, 0x18, -1);
                    wr(o, 0x1c, -1);
                    wr(o, 0x20, 0x640);
                    o[(off + 4) as usize] &= 0xfb;
                    let data = order_at.wrapping_add(off as u32);
                    // A node from the recycler, or a fresh one.
                    let len = rd(&self.c[4], 0);
                    let node = if len == 0 {
                        self.log.push(format!("s6:p{}+0x0,20", 37 + j));
                        let m = self.seq(57, 9, self.mallocs) as u32;
                        self.mallocs += 1;
                        self.w32(m, 1);
                        self.log.push(format!(
                            "s7:p{}+0x0,c9+{:#x},16,1,4648320,4449680",
                            37 + j,
                            m + 4 - 0xe90000
                        ));
                        m + 4
                    } else {
                        let l = len.max(1) - 1;
                        wr(&mut self.c[4], 0, l);
                        rd(&self.p[6], 4 * l as usize) as u32
                    };
                    self.w8(node + 0xc, 0);
                    self.w32(node + 8, data);
                    self.w32(node + 4, 0);
                    self.w32(node, 0);
                    let head = rd(&self.p[12 + b], 0xdc) as u32;
                    if head != 0 {
                        self.w32(node, head);
                        let tail = self.r32(head + 4);
                        self.w32(node + 4, tail);
                        let tail = self.r32(head + 4);
                        self.w32(tail, node);
                        let h = rd(&self.p[12 + b], 0xdc) as u32;
                        self.w32(h + 4, node);
                        wr(&mut self.p[12 + b], 0xdc, node as i32);
                    } else {
                        wr(&mut self.p[12 + b], 0xdc, node as i32);
                        self.w32(node, node);
                        let h = rd(&self.p[12 + b], 0xdc) as u32;
                        self.w32(h + 4, h);
                    }
                    let metric = self.r8(node + 0xc);
                    let u = &mut self.p[12 + b];
                    let n = rd(u, 0xd8).wrapping_add(1);
                    wr(u, 0xd8, n);
                    wr(u, 0xd4, node as i32);
                    wr(u, 0xcc, data as i32);
                    u[0xd0] = metric;
                    self.log.push(format!("s8:p{}+0x0", 12 + b));
                }
            }
            let mut ranges = vec![(0xe90000u32, 0x140u32, Win::C(9))];
            for k in 0..4 {
                ranges.push((rd(&p[6], 4 * k) as u32, 0x10, Win::P(7 + k)));
            }
            for k in 0..3 {
                let h = rd(&p[12 + k], 0xdc) as u32;
                if h != 0 {
                    ranges.push((h, 0x10, Win::P(22 + k)));
                    let t = rd(&p[22 + k], 4) as u32;
                    if t != h && t != 0 {
                        ranges.push((t, 0x10, Win::P(25 + k)));
                    }
                }
            }
            let mut v = V {
                a,
                c,
                p,
                log: &mut log,
                ranges,
                rolls: 0,
                inits: 0,
                mallocs: 0,
                orders: 0,
                list: 0,
                pushed: 0,
            };
            add_flock_steps(&mut v, a[0], a[1], a[2], a[3]);
            // A row that called nothing records `-`.
            if log.is_empty() {
                log.push("-".into());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "add_flock — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_flock — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "add_flock — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[11 + i], "add_flock — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 700, "only {n} rows");
    eprintln!("Objects::add_flock@0065c0e0: {n} rows agree");
}
