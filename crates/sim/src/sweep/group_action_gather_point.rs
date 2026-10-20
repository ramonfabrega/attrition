//! `Group::action_gather_point@006ff1b0` under the emulator against the port (item 3358; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_gather_point@006ff1b0`, run by `tools/emu/sweep_group_action_gather_point.py` under unicorn
/// (tools/emu): 2304 random bytes of object, 4 argument(s), `p` windows in order: p0..p19 the place(s), in flag order; the global cell 0x00c0618c points at 7232 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c06210 points at 672 random bytes; vtable slot +0x14 (0 args) stubbed; vtable slot +0x4 (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0xe4 (0 args) stubbed; vtable slot +0x178 (0 args) stubbed; callee 0x00633f50 (1 args) stubbed; callee 0x0046f180 (0 args) stubbed; callee 0x00623180 (0 args) stubbed; callee 0x00622e70 (4 args) stubbed; callee 0x00711720 (3 args) stubbed; callee 0x0065d260 (9 args) stubbed; callee 0x007cb870 (5 args) stubbed; callee 0x006439b0 (2 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x006fb260 (6 args) stubbed; callee 0x00a1d590 (1 args) stubbed; callee 0x007e9ab0 (7 args) stubbed; the word at `c0+0x14+0x1c*m[0x4a]` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `m+0x7c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 128 random bytes less `0` (placed); the word at `p5+4` points at 128 random bytes less `0` (placed); the word at `p5+8` points at 128 random bytes less `0` (placed); the word at `p5+12` points at 128 random bytes less `0` (placed); the word at `0xc0aaa0` points at 12 random bytes less `0` (placed); the word at `p10+0` points at 720 random bytes less `0` (placed); the word at `p10+4` points at 720 random bytes less `0` (placed); the word at `p10+8` points at 720 random bytes less `0` (placed); the word at `p6+0x18` points at 96 random bytes less `0` (placed); the word at `p7+0x18` points at 96 random bytes less `0` (placed); the word at `p8+0x18` points at 96 random bytes less `0` (placed); the word at `p9+0x18` points at 96 random bytes less `0` (placed); the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); the word at `c1+0x138` points at 4096 random bytes less `0` (placed); slot +0xe4 laid as 0x00639970 or its stub, a flag after the answers; stub 1 answered from [1, 2, 3, 3, 0] (90%); stub 2 answered from [1, 1, 1, 0] (90%); stub 3 answered from [1, 1, 1, 0] (90%); stub 5 answered from [0, 0, 0, 1] (90%); stub 6 answered from [0, 0, 0, 1] (90%); stub 8 answered from [0, 0, 0, 1, 5] (90%); stub 9 answered from [0, 0, 0, 1, 5] (90%); stub 10 answered from [0, 0, 0, 1, 2] (90%); stub 13 answered from [1, 2, 3, 3, 0] (90%); stub 16 answered from [0, 0, 1, 1] (90%); the word at `p6+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p7+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p8+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p9+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 4 answers `p6+j`, the data of the object its `ecx` is in `p1+j` (j < 4), linked at `+0x7c`; stub 2 answers 24 values in call order, printed after the answers; stub 3 answers 24 values in call order, printed after the answers; stub 5 answers 40 values in call order, printed after the answers; stub 6 answers 40 values in call order, printed after the answers; stub 7 answers 8 values in call order, printed after the answers; stub 8 answers 8 values in call order, printed after the answers; stub 9 answers 8 values in call order, printed after the answers; stub 10 answers 8 values in call order, printed after the answers; stub 14 answers 2 values in call order, printed after the answers; stub 16 answers 8 values in call order, printed after the answers; stub 7 answers 0..2 every row; stub 14 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 17 writes inc (4 bytes) at `ecx+0xc` each call; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_gather_point.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Window layout. `m` the group (`+0xc` count, `+0x49` armed, `+0x4a` player, the shorts at `+0x8cc`);
                // `c[0]` the objects record (the per-player arrays at `+0x14 + 0x1c * who`), `c[1]` the world
                // (`+0`/`+4` extent, `+0x18` stride, `+0x138` the tile words, p19), `c[2]` the console (`+0x298`
                // who). `p[1 + j]` the member objects (`+8 & 1`, `+0x10`/`+0x14` the xor'd position), `p[6 + j]`
                // the `Build` data each answers by slot `+0xac` (`+0` vtable: `Build::vftable` or the stubbed
                // one; `+9` level; `+0x18` its type, `p[14 + j]`), `p[11..14]` the build types (`+0x2c0` the
                // training flag word), `p[18]` the `div_3_table`, `p[19]` the tile words. `a`: the four
                // arguments, one answer a stub (`a[4 + k]`, k the stub), then the `--seq` runs in flag order
                // (stubs 2, 3, 5, 6, 7, 8, 9, 10, 14, 16) and the `basic_type` flag (`a[195]`).
                //
                // **The port is the crate's own steps**, `crate::gather_view::action_gather_point_steps`, which
                // `Sim::action_gather_point` calls over its `SimGather`; here the view is the emulated original's
                // windows.
                use crate::gather_view::{GatherView, action_gather_point_steps};
                const SEQ_AT: [(usize, usize, usize); 10] = [
                    (2, 25, 24),
                    (3, 49, 24),
                    (5, 73, 40),
                    (6, 113, 40),
                    (7, 153, 8),
                    (8, 161, 8),
                    (9, 169, 8),
                    (10, 177, 8),
                    (14, 185, 2),
                    (16, 187, 8),
                ];
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 21],
                    flight: i32,
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                impl V<'_> {
                    fn who(&self) -> i32 {
                        i32::from(self.m[0x4a])
                    }
                    /// Stub `k`'s next `--seq` answer, the last repeated.
                    fn seq(&mut self, k: usize) -> i32 {
                        let &(_, at, n) = SEQ_AT.iter().find(|s| s.0 == k).expect("a seq stub");
                        let i = self.turn[k].min(n - 1);
                        self.turn[k] += 1;
                        self.a[at + i]
                    }
                    fn rec(&mut self, k: usize, ecx: &str, args: &[i32]) {
                        let mut t = format!("s{k}:{ecx}");
                        for v in args {
                            t.push_str(&format!(",{v}"));
                        }
                        self.log.push(t);
                    }
                    /// Slot `+0xac`: the object `p[1 + j]` answers its data.
                    fn slot_ac(&mut self, j: usize) {
                        self.log.push(format!("s4:p{}+0x0", 1 + j));
                    }
                    fn div3(&self, v: i32) -> i32 {
                        rd32(&self.p[18], 4 * (v >> 6) as usize)
                    }
                    fn is_build(&self, j: usize) -> bool {
                        rd32(&self.p[6 + j], 0) as u32 == 0x00b4_2174
                    }
                }
                impl GatherView for V<'_> {
                    type H = usize;
                    fn refresh(&mut self) {
                        self.log.push("s0:m+0x0".into());
                    }
                    fn count(&self) -> i32 {
                        rd32(self.m, 0xc)
                    }
                    fn member(&self, i: i32) -> usize {
                        let o = 0x8cc + 2 * i as usize;
                        i16::from_le_bytes([self.m[o], self.m[o + 1]]) as usize
                    }
                    fn armed(&self) -> bool {
                        self.m[0x49] != 0
                    }
                    fn alive_flag(&mut self, h: usize) -> bool {
                        self.p[1 + h][8] & 1 != 0
                    }
                    fn alive_slot(&mut self, h: usize) -> bool {
                        self.rec(2, &format!("p{}+0x0", 1 + h), &[]);
                        self.seq(2) != 0
                    }
                    fn is_building(&mut self, h: usize) -> bool {
                        self.rec(3, &format!("p{}+0x0", 1 + h), &[]);
                        self.seq(3) != 0
                    }
                    fn is_ident(&mut self, h: usize, id: i32) -> bool {
                        self.slot_ac(h);
                        if self.is_build(h) {
                            self.rec(5, &format!("p{}+0x0", 14 + h), &[id, 0]);
                            self.seq(5) != 0
                        } else {
                            self.rec(6, &format!("p{}+0x0", 6 + h), &[id, 0]);
                            self.seq(6) != 0
                        }
                    }
                    fn gather_inside(&mut self, h: usize) -> bool {
                        self.slot_ac(h);
                        self.rec(10, &format!("p{}+0x0", 6 + h), &[]);
                        self.seq(10) != 0
                    }
                    fn clear_gather(&mut self, h: usize) {
                        self.slot_ac(h);
                        self.rec(11, &format!("p{}+0x0", 6 + h), &[]);
                    }
                    fn add_gather_point(&mut self, h: usize, x: i32, y: i32, action: i32, pos: i32) {
                        self.slot_ac(h);
                        self.rec(12, &format!("p{}+0x0", 6 + h), &[x, y, action, pos]);
                    }
                    fn world_extent(&self) -> (i32, i32) {
                        (rd32(&self.c[1], 0) * 0x300, rd32(&self.c[1], 4) * 0x300)
                    }
                    fn tile_mask(&mut self, x: i32, y: i32) -> i32 {
                        let i = rd32(&self.c[1], 0x18) * self.div3(y) + self.div3(x);
                        let o = 2 * i as usize;
                        i32::from(u16::from_le_bytes([self.p[19][o], self.p[19][o + 1]]))
                    }
                    fn covers_tile(&mut self, h: usize, x: i32, y: i32) -> bool {
                        self.slot_ac(h);
                        let (tx, ty) = (self.div3(x), self.div3(y));
                        self.rec(16, &format!("p{}+0x0", 6 + h), &[tx, ty]);
                        self.seq(16) != 0
                    }
                    fn count_type(&mut self, id: i32) -> i32 {
                        self.rec(13, "m+0x0", &[0x11, id, 0]);
                        self.a[4 + 13]
                    }
                    fn group_num(&mut self) -> i32 {
                        self.rec(1, "m+0x0", &[]);
                        self.a[4 + 1]
                    }
                    fn find_building(&mut self, x: i32, y: i32, id: i32) -> i32 {
                        let who = self.who();
                        self.rec(14, "c0+0x0", &[x, y, 1, who, 0x600, 0, 1, id, 0]);
                        self.seq(14)
                    }
                    fn object_pos(&mut self, idx: i32) -> (i32, i32) {
                        let o = &self.p[1 + idx as usize];
                        (rd32(o, 0x10) ^ 0x63637, rd32(o, 0x14) ^ 0x63637)
                    }
                    fn console_is_owner(&mut self) -> bool {
                        rd32(&self.c[2], 0x298) == self.who()
                    }
                    fn ping_target(&mut self, idx: i32) {
                        let who = self.who();
                        self.rec(15, "c2+0x0", &[idx, who, -16_711_936, 132_121_568, 2]);
                    }
                    fn trains(&mut self, h: usize) -> bool {
                        self.slot_ac(h);
                        let ty = &self.p[14 + h];
                        let idx = if self.a[195] != 0 {
                            // `basic_type` inline (the slot holds it): the type's own `+4`, or the build type
                            // it names (`+0x3c`), whose `basic_type` is its `+4` (its `+0x3c` is -1 here).
                            let n = rd32(ty, 0x3c);
                            if n < 0 {
                                rd32(ty, 4)
                            } else {
                                rd32(&self.p[11 + n as usize], 4)
                            }
                        } else {
                            self.rec(7, &format!("p{}+0x0", 14 + h), &[]);
                            self.seq(7)
                        };
                        rd32(&self.p[11 + idx as usize], 0x2c0) < 0
                    }
                    fn garrison_limit(&mut self, h: usize) -> i32 {
                        self.slot_ac(h);
                        if self.is_build(h) {
                            let lvl = i32::from(self.p[6 + h][9]);
                            self.rec(9, &format!("p{}+0x0", 14 + h), &[lvl]);
                            self.seq(9)
                        } else {
                            self.rec(8, &format!("p{}+0x0", 6 + h), &[]);
                            self.seq(8)
                        }
                    }
                    fn flight_add(&mut self, h: usize) {
                        self.flight += 1;
                        let who = self.who();
                        self.rec(17, "sp-0xa08", &[h as i32, who, 0, 0]);
                    }
                    fn flight_count(&mut self) -> i32 {
                        self.flight
                    }
                    fn flight_go(&mut self, x: i32, y: i32) {
                        self.rec(18, "sp-0xa08", &[x, y, 10, 0, 0, 0]);
                    }
                    fn feedback(&mut self, id: i32) {
                        self.rec(19, "sp-0xa34", &[34120 + (id - 0x8548)]);
                    }
                }
                let mut v = V {
                    a,
                    m: &*m,
                    c: &*c,
                    p: &*p,
                    log: &mut log,
                    turn: [0; 21],
                    flight: 0,
                };
                action_gather_point_steps(&mut v, a[0], a[1], a[2], a[3]);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "action_gather_point — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_gather_point — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "action_gather_point — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "action_gather_point — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Group::action_gather_point@006ff1b0: {n} rows agree");
}
