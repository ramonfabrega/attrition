//! `Object::take_damage@00652020` under the emulator against the port (item 3376; tools/sweep_gen.py, obj with windows).

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

/// `Object::take_damage@00652020`, run by `tools/emu/sweep_object_take_damage_arms.py` under unicorn
/// (tools/emu): 256 random bytes of object, 9 argument(s), `p` windows in order: p0..p3 sparse table 0 (2 rows × 2 slice(s), row-major), p4..p59 the place(s), in flag order, p60 stub 14's answer window; the global cell 0x00c06378 points at 64 random bytes; the global cell 0x00c061ec points at 2304 random bytes; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c06188 points at 512 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; callee 0x00473590 (7 args) stubbed; vtable slot +0x1c (0 args) stubbed; callee 0x00a39d70 (2 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x10c (0 args) stubbed; callee 0x0065c0e0 (4 args) stubbed; callee 0x00594670 (0 args) stubbed; callee 0x006ebaa0 (1 args) stubbed; callee 0x006dab50 (4 args) stubbed; vtable slot +0x11c (1 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x006ec000 (0 args) stubbed; callee 0x0046d310 (0 args) stubbed; callee 0x0065b870 (5 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x0070f9e0 (3 args) stubbed; callee 0x0070ec30 (0 args) stubbed; callee 0x00646c40 (1 args) stubbed; callee 0x0064cd20 (4 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; the word at `c2+0x14+0x1c*0` points at 32 random bytes less `0` (placed); the word at `c2+0x14+0x1c*1` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 32 random bytes less `0` (placed); the word at `c3+0x134` points at 4032 random bytes less `0` (placed); the word at `p4+0` points at 96 random bytes less `0` (placed); the word at `p4+4` points at 96 random bytes less `0` (placed); the word at `p4+8` points at 96 random bytes less `0` (placed); the word at `p4+12` points at 96 random bytes less `0` (placed); the word at `p5+0` points at 96 random bytes less `0` (placed); the word at `p5+4` points at 96 random bytes less `0` (placed); the word at `p5+8` points at 96 random bytes less `0` (placed); the word at `p5+12` points at 96 random bytes less `0` (placed); the word at `p6+0` points at 96 random bytes less `0` (placed); the word at `p6+4` points at 96 random bytes less `0` (placed); the word at `p6+8` points at 96 random bytes less `0` (placed); the word at `p6+12` points at 96 random bytes less `0` (placed); the word at `p7+0` points at 96 random bytes less `0` (placed); the word at `p7+4` points at 96 random bytes less `0` (placed); the word at `p7+8` points at 96 random bytes less `0` (placed); the word at `p7+12` points at 96 random bytes less `0` (placed); the word at `p9+0x18` points at 800 random bytes less `0` (placed); the word at `p10+0x18` points at 800 random bytes less `0` (placed); the word at `p11+0x18` points at 800 random bytes less `0` (placed); the word at `p12+0x18` points at 800 random bytes less `0` (placed); the word at `p13+0x18` points at 800 random bytes less `0` (placed); the word at `p14+0x18` points at 800 random bytes less `0` (placed); the word at `p15+0x18` points at 800 random bytes less `0` (placed); the word at `p16+0x18` points at 800 random bytes less `0` (placed); the word at `p17+0x18` points at 800 random bytes less `0` (placed); the word at `p18+0x18` points at 800 random bytes less `0` (placed); the word at `p19+0x18` points at 800 random bytes less `0` (placed); the word at `p20+0x18` points at 800 random bytes less `0` (placed); the word at `p21+0x18` points at 800 random bytes less `0` (placed); the word at `p22+0x18` points at 800 random bytes less `0` (placed); the word at `p23+0x18` points at 800 random bytes less `0` (placed); the word at `p24+0x18` points at 800 random bytes less `0` (placed); the word at `m+0x18` points at 64 random bytes less `0` (placed); the word at `p25+0x18` points at 64 random bytes less `0` (placed); the word at `p26+0x18` points at 64 random bytes less `0` (placed); the word at `p27+0x18` points at 64 random bytes less `0` (placed); the word at `p28+0x18` points at 64 random bytes less `0` (placed); the word at `p29+0x18` points at 64 random bytes less `0` (placed); the word at `p30+0x18` points at 64 random bytes less `0` (placed); the word at `p31+0x18` points at 64 random bytes less `0` (placed); the word at `p32+0x18` points at 64 random bytes less `0` (placed); the word at `0xc09970+0x1c*0` points at 32 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p50+0` points at 128 random bytes less `0` (placed); the word at `p50+4` points at 128 random bytes less `0` (placed); the word at `p50+8` points at 128 random bytes less `0` (placed); the word at `p50+12` points at 128 random bytes less `0` (placed); the word at `p51+0` points at 128 random bytes less `0` (placed); the word at `p51+4` points at 128 random bytes less `0` (placed); the word at `p51+8` points at 128 random bytes less `0` (placed); the word at `p51+12` points at 128 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1, 2, 3, 4, 5, 6, 50, 99, 100, 103, 65535] (90%); stub 3 answered from [0, 0, 1] (90%); stub 5 answered from [0, 1, 0] (90%); stub 6 answered from [0, 1, 1] (90%); stub 4 answered from [0, 0, 1] (90%); stub 8 answered from [0, 1, 1] (90%); stub 9 answered from [0, 0, 1] (90%); stub 11 answered from [0, 1, 5, 100, 200] (90%); stub 12 answered from [0, 1] (90%); stub 13 answered from [0, 1] (90%); stub 16 answered from [0, 1, 2, 3] (90%); stub 17 answered from [1, 1, 0] (90%); stub 21 answered from [-1, 0, 1, 2, 3] (90%); stub 26 answered from [0, 1] (90%); stub 23 answered from [0, 1] (90%); stub 14 answers the address of 128 random bytes, a `p` window after the places; the word at `p25+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p26+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p27+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p28+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p29+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p30+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p31+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p32+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 15 answers `p25+j`, the data of the object its `ecx` is in `p9+j` (j < 8), linked at `+0x1f0`; stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 2 values in call order, printed after the answers; stub 5 answers 8 values in call order, printed after the answers; stub 6 answers 2 values in call order, printed after the answers; stub 4 answers 4 values in call order, printed after the answers; stub 11 answers 8 values in call order, printed after the answers; stub 12 answers 4 values in call order, printed after the answers; stub 13 answers 1 values in call order, printed after the answers; stub 17 answers 2 values in call order, printed after the answers; stub 26 answers 2 values in call order, printed after the answers; stub 18 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 2816 bytes under the call is zeroed before each row; the call ends where stub 25 is first entered (the function is swept to that callee); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x8, +0xa40:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p43+0` as well as at `this`; the stubbed vtable laid at `p44+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; the stubbed vtable laid at `p48+0` as well as at `this`; the stubbed vtable laid at `p49+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_take_damage_arms.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The arms slice: the call ends where the kill arm's first call (slot `+0x18`, stub 25) is
                // entered. Window layout as the head's: `m` the struck object, `c[0]` the log cell, `c[1]` the
                // game, `c[2]` the objects cell, `c[3]` the world, `c[4]` the `div_3_table`; `p[2 * who]` the
                // leader slices (`+0`, `+4`), `p[2 * who + 1]` (`+0xa40`), `p[8]` the world's cells,
                // `p[9 + 4 * pl + j]` `objects[pl][j]` with its data `p[25 + 4 * pl + j]` and the data's type
                // `p[42 + 4 * pl + j]`, `p[17 + 4 * pl + j]` the units and `p[33 + 4 * pl + j]` their data,
                // `p[41]` `this`'s data, `p[52 + 4 * pl + c]` `cities[pl][c]`, `p[60]` what slot `+0xb0`
                // answers. `a`: the nine arguments, the stubs' answers (`a[9..36]`), every `--seq` run
                // (`SEQ_AT`) and the `+0xb8` flag.
                //
                // **The port is the crate's own steps**, `crate::take_damage_view::take_damage_head` then
                // `take_damage_arms`; here the view is the emulated original's windows.
                use crate::take_damage_view::{
                    ArmsOut, HeadOut, TakeDamageArms, TakeDamageHead, TdArgs, take_damage_arms,
                    take_damage_head,
                };
                const SEQ_AT: [(usize, usize, usize); 11] = [
                    (1, 36, 4),
                    (2, 40, 4),
                    (3, 44, 2),
                    (5, 46, 8),
                    (6, 54, 2),
                    (4, 56, 4),
                    (11, 60, 8),
                    (12, 68, 4),
                    (13, 72, 1),
                    (17, 73, 2),
                    (26, 75, 2),
                ];
                const BASED: usize = 77;
                const KEY: i32 = 0x63637;
                const RANDOM_CELL: i32 = 14_908_044;
                const BUILD_VT: i32 = 0x00b4_2174;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 27],
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn rd16(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes([w[o], w[o + 1]]))
                }
                impl V<'_> {
                    fn seq(&mut self, k: usize) -> i32 {
                        let &(_, at, n) = SEQ_AT.iter().find(|s| s.0 == k).expect("a seq stub");
                        let i = self.turn[k].min(n - 1);
                        self.turn[k] += 1;
                        self.a[at + i]
                    }
                    fn ans(&self, k: usize) -> i32 {
                        self.a[9 + k]
                    }
                    fn rec(&mut self, k: usize, ecx: &str, args: &[i32]) {
                        let mut t = format!("s{k}:{ecx}");
                        for v in args {
                            t.push_str(&format!(",{v}"));
                        }
                        self.log.push(t);
                    }
                    fn pos(&self, off: usize) -> i32 {
                        rd32(self.m, off) ^ KEY
                    }
                    fn div3(&self, i: i32) -> i32 {
                        rd32(&self.c[4], 4 * i as usize)
                    }
                    fn based(&self) -> bool {
                        self.a[BASED] != 0
                    }
                    fn leader_ecx(who: i32) -> String {
                        format!("p{}+0x0", 2 * who)
                    }
                    /// `objects[who][idx]` of `this`: its object window's index.
                    fn mine(&self) -> i32 {
                        4 * i32::from(self.m[9]) + rd16(self.m, 0xa)
                    }
                    /// Slot `+0xac` of `this`'s own record: the data window's index.
                    fn data_of_mine(&mut self) -> usize {
                        let j = self.mine();
                        self.rec(15, &format!("p{}+0x0", 9 + j), &[]);
                        (25 + j) as usize
                    }
                }
                impl TakeDamageHead for V<'_> {
                    fn flags8(&self) -> i32 {
                        i32::from(self.m[8])
                    }
                    fn say(&mut self, which: i32, a: i32, b: i32, c: i32, d: i32) {
                        let s = rd32(&self.c[0], 0x10) + if which == 1 { 0x18a9c } else { 0x1070c };
                        self.rec(0, "m+0x0", &[0, 0, s, a, b, c, d]);
                    }
                    fn sem4(&self) -> bool {
                        self.c[1][0x820] & 4 != 0
                    }
                    fn difficulty(&self) -> i32 {
                        i32::from(self.c[1][0x2b])
                    }
                    fn game_frame(&self) -> i32 {
                        rd32(&self.c[1], 0x550)
                    }
                    fn who(&self) -> i32 {
                        i32::from(self.m[9])
                    }
                    fn set_frame_attacked(&mut self, who: i32, frame: i32) {
                        self.p[2 * who as usize + 1][..4].copy_from_slice(&frame.to_le_bytes());
                    }
                    fn damage(&self) -> i32 {
                        rd32(self.m, 0x24)
                    }
                    fn is_build(&mut self) -> bool {
                        self.rec(1, "m+0x0", &[]);
                        self.seq(1) != 0
                    }
                    fn roll(&mut self) -> i32 {
                        self.rec(2, &RANDOM_CELL.to_string(), &[0, 0xffff]);
                        self.seq(2)
                    }
                    fn data_fc(&mut self) -> bool {
                        self.rec(3, "p41+0x0", &[]);
                        self.seq(3) != 0
                    }
                    fn this_is(&mut self, ty: i32) -> bool {
                        if self.based() {
                            self.rec(5, "p41+0x0", &[ty, 0]);
                            self.seq(5) != 0
                        } else {
                            self.rec(4, "m+0x0", &[ty, 0]);
                            self.seq(4) != 0
                        }
                    }
                    fn attacker_siege(&mut self, p8: i32, p7: i32) -> bool {
                        self.rec(6, &format!("p{}+0x0", 25 + 4 * p8 + p7), &[]);
                        self.seq(6) != 0
                    }
                    fn flock(&mut self, n: i32) {
                        let (ix, iy) = (self.pos(0x10) >> 6, self.pos(0x14) >> 6);
                        let (x, y) = (self.div3(ix), self.div3(iy));
                        self.rec(7, &iy.to_string(), &[x, y, -1, n]);
                    }
                    fn attacker_provokes(&self, p8: i32, p7: i32) -> bool {
                        rd32(&self.p[(25 + 4 * p8 + p7) as usize], 0x1e4) & 0x0800_0000 != 0
                    }
                    fn attacker_is(&mut self, p8: i32, p7: i32, ty: i32) -> bool {
                        if self.based() {
                            self.rec(5, &format!("p{}+0x0", 33 + 4 * p8 + p7), &[ty, 0]);
                            self.seq(5) != 0
                        } else {
                            self.rec(4, &format!("p{}+0x0", 17 + 4 * p8 + p7), &[ty, 0]);
                            self.seq(4) != 0
                        }
                    }
                    fn war_allowed(&mut self) -> bool {
                        self.rec(8, "c1+0x0", &[]);
                        self.ans(8) != 0
                    }
                    fn leader_flags(&self, who: i32) -> i32 {
                        rd32(&self.p[2 * who as usize], 0)
                    }
                    fn cell_owner(&self) -> i32 {
                        let (x, y) = (self.pos(0x10) >> 8, self.pos(0x14) >> 8);
                        let i = self.div3(y) * rd32(&self.c[3], 0) + self.div3(x);
                        i32::from(self.p[8][28 * i as usize + 0xf] as i8)
                    }
                    fn is_enemy(&mut self, who: i32, p8: i32) -> bool {
                        self.rec(9, &Self::leader_ecx(who), &[p8]);
                        self.ans(9) != 0
                    }
                    fn declare(&mut self, who: i32, p8: i32, flag: i32) {
                        self.rec(10, &Self::leader_ecx(who), &[p8, 0, flag, 0]);
                    }
                    fn carry(&self) -> i32 {
                        i32::from(self.m[0x3b] as i8)
                    }
                    fn set_carry(&mut self, v: i32) {
                        self.m[0x3b] = v as u8;
                    }
                }
                impl TakeDamageArms for V<'_> {
                    fn share(&mut self) -> i32 {
                        self.rec(11, "m+0x0", &[0]);
                        self.seq(11)
                    }
                    fn set_damage(&mut self, v: i32) {
                        self.m[0x24..0x28].copy_from_slice(&v.to_le_bytes());
                    }
                    fn set_flags8(&mut self, v: i32) {
                        self.m[8] = v as u8;
                    }
                    fn this_finished(&mut self) -> bool {
                        self.rec(12, "m+0x0", &[]);
                        self.seq(12) != 0
                    }
                    fn attacker_air(&self, p8: i32, p7: i32) -> bool {
                        self.p[(9 + 4 * p8 + p7) as usize][8] & 1 != 0
                            && rd32(&self.p[(25 + 4 * p8 + p7) as usize], 0x218) == 2
                    }
                    fn progress(&mut self) -> u32 {
                        let j = self.mine();
                        self.rec(14, &format!("p{}+0x0", 9 + j), &[]);
                        rd32(&self.p[60], 0x48) as u32
                    }
                    fn set_progress(&mut self, v: u32) {
                        let j = self.mine();
                        self.rec(14, &format!("p{}+0x0", 9 + j), &[]);
                        self.p[60][0x48..0x4c].copy_from_slice(&v.to_le_bytes());
                    }
                    fn this_city_building(&mut self) -> bool {
                        self.rec(13, "m+0x0", &[]);
                        self.seq(13) != 0
                    }
                    fn attacker_peasant(&mut self, p8: i32, p7: i32) -> bool {
                        self.rec(17, &format!("p{}+0x0", 9 + 4 * p8 + p7), &[]);
                        self.seq(17) != 0
                    }
                    fn attacker_obj_is(&mut self, p8: i32, p7: i32, ty: i32) -> bool {
                        if self.based() {
                            self.rec(5, &format!("p{}+0x0", 25 + 4 * p8 + p7), &[ty, 0]);
                            self.seq(5) != 0
                        } else {
                            self.rec(4, &format!("p{}+0x0", 9 + 4 * p8 + p7), &[ty, 0]);
                            self.seq(4) != 0
                        }
                    }
                    fn this_peasant(&mut self) -> bool {
                        self.rec(17, "m+0x0", &[]);
                        self.seq(17) != 0
                    }
                    fn mark_struck(&mut self) {
                        let d = self.data_of_mine();
                        let w = u16::from_le_bytes([self.p[d][0x60], self.p[d][0x61]]) | 0x30;
                        self.p[d][0x60..0x62].copy_from_slice(&w.to_le_bytes());
                    }
                    fn data_city(&mut self) -> i32 {
                        let d = self.data_of_mine();
                        rd16(&self.p[d], 0x72)
                    }
                    fn data_type_slot90(&mut self) -> bool {
                        let d = self.data_of_mine();
                        self.rec(26, &format!("p{}+0x0", d + 17), &[]);
                        self.seq(26) != 0
                    }
                    fn data_flags8(&mut self) -> i32 {
                        let d = self.data_of_mine();
                        i32::from(self.p[d][8])
                    }
                    fn city_flags(&self, c: i32) -> i32 {
                        let w = &self.p[(52 + 4 * i32::from(self.m[9]) + c) as usize];
                        i32::from(u16::from_le_bytes([w[4], w[5]]))
                    }
                    fn set_city_flags(&mut self, c: i32, v: i32) {
                        let w = &mut self.p[(52 + 4 * i32::from(self.m[9]) + c) as usize];
                        w[4..6].copy_from_slice(&(v as u16).to_le_bytes());
                    }
                    fn city_building(&self, c: i32) -> i32 {
                        rd16(&self.p[(52 + 4 * i32::from(self.m[9]) + c) as usize], 8)
                    }
                    fn city_player(&self, c: i32) -> i32 {
                        i32::from(self.p[(52 + 4 * i32::from(self.m[9]) + c) as usize][0x5e] as i8)
                    }
                    fn leader_word4(&self, who: i32) -> i32 {
                        rd32(&self.p[2 * who as usize], 4)
                    }
                    fn leader_diff(&mut self, who: i32) -> i32 {
                        self.rec(16, &Self::leader_ecx(who), &[]);
                        self.ans(16)
                    }
                    fn find_city_at(&mut self, who: i32) -> i32 {
                        let (x, y) = (self.pos(0x10), self.pos(0x14));
                        self.rec(18, "c2+0x0", &[x, y, who, -1, 0]);
                        self.ans(18)
                    }
                    fn city_building_finished(&mut self, player: i32, idx: i32) -> bool {
                        let j = 4 * player + idx;
                        self.rec(15, &format!("p{}+0x0", 9 + j), &[]);
                        let d = (25 + j) as usize;
                        if rd32(&self.p[d], 0) == BUILD_VT {
                            self.p[d][8] & 4 != 0
                        } else {
                            self.rec(12, &format!("p{d}+0x0"), &[]);
                            self.seq(12) != 0
                        }
                    }
                    fn group_clear(&mut self) {
                        self.log.push("s19:sp-0xa18,-1".into());
                    }
                    fn group_add(&mut self, idx: i32, player: i32) {
                        self.log.push(format!("s20:sp-0xa18,{idx},{player},0,0"));
                    }
                    fn push_group(&mut self, who: i32) -> i32 {
                        self.log.push(format!("s21:sp-0xa18,{who},sp-0xa18,1"));
                        self.ans(21)
                    }
                    fn group_alarm(&mut self, g: i32) {
                        self.log.push(format!("s22:{}", g.wrapping_mul(0x9d4)));
                    }
                    fn can_carry_air(&mut self) -> bool {
                        self.rec(23, "m+0x0", &[2]);
                        self.ans(23) != 0
                    }
                    fn eject(&mut self) {
                        self.rec(24, "m+0x0", &[1, -1, 0, 1]);
                    }
                }
                let mut v = V {
                    a,
                    m: &mut *m,
                    c: &*c,
                    p: &mut *p,
                    log: &mut log,
                    turn: [0; 27],
                };
                let args = TdArgs {
                    p: a[..9].try_into().expect("nine arguments"),
                };
                let r = match take_damage_head(&mut v, &args) {
                    HeadOut::Return(r) => i64::from(r),
                    HeadOut::Accumulate { whole, .. } => {
                        match take_damage_arms(&mut v, &args, whole) {
                            ArmsOut::Return(r) => i64::from(r),
                            ArmsOut::Join { .. } => {
                                v.rec(25, "m+0x0", &[]);
                                i64::from(v.ans(25))
                            }
                        }
                    }
                };
                // A row that called nothing records `-`.
                if log.is_empty() {
                    log.push("-".to_string());
                }
                r
            }
        };
        assert_eq!(got, want_eax, "take_damage — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "take_damage — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "take_damage — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "take_damage — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "take_damage — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Object::take_damage@00652020: {n} rows agree");
}
