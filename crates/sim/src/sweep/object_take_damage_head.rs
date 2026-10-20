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

/// `Object::take_damage@00652020`, run by `tools/emu/sweep_object_take_damage_head.py` under unicorn
/// (tools/emu): 256 random bytes of object, 9 argument(s), `p` windows in order: p0..p3 sparse table 0 (2 rows × 2 slice(s), row-major), p4..p41 the place(s), in flag order; the global cell 0x00c06378 points at 64 random bytes; the global cell 0x00c061ec points at 2304 random bytes; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c06188 points at 512 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; callee 0x00473590 (7 args) stubbed; vtable slot +0x1c (0 args) stubbed; callee 0x00a39d70 (2 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x10c (0 args) stubbed; callee 0x0065c0e0 (4 args) stubbed; callee 0x00594670 (0 args) stubbed; callee 0x006ebaa0 (1 args) stubbed; callee 0x006dab50 (4 args) stubbed; vtable slot +0x11c (1 args) stubbed; the word at `c2+0x14+0x1c*0` points at 32 random bytes less `0` (placed); the word at `c2+0x14+0x1c*1` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 32 random bytes less `0` (placed); the word at `c3+0x134` points at 4032 random bytes less `0` (placed); the word at `p4+0` points at 96 random bytes less `0` (placed); the word at `p4+4` points at 96 random bytes less `0` (placed); the word at `p4+8` points at 96 random bytes less `0` (placed); the word at `p4+12` points at 96 random bytes less `0` (placed); the word at `p5+0` points at 96 random bytes less `0` (placed); the word at `p5+4` points at 96 random bytes less `0` (placed); the word at `p5+8` points at 96 random bytes less `0` (placed); the word at `p5+12` points at 96 random bytes less `0` (placed); the word at `p6+0` points at 96 random bytes less `0` (placed); the word at `p6+4` points at 96 random bytes less `0` (placed); the word at `p6+8` points at 96 random bytes less `0` (placed); the word at `p6+12` points at 96 random bytes less `0` (placed); the word at `p7+0` points at 96 random bytes less `0` (placed); the word at `p7+4` points at 96 random bytes less `0` (placed); the word at `p7+8` points at 96 random bytes less `0` (placed); the word at `p7+12` points at 96 random bytes less `0` (placed); the word at `p9+0x18` points at 800 random bytes less `0` (placed); the word at `p10+0x18` points at 800 random bytes less `0` (placed); the word at `p11+0x18` points at 800 random bytes less `0` (placed); the word at `p12+0x18` points at 800 random bytes less `0` (placed); the word at `p13+0x18` points at 800 random bytes less `0` (placed); the word at `p14+0x18` points at 800 random bytes less `0` (placed); the word at `p15+0x18` points at 800 random bytes less `0` (placed); the word at `p16+0x18` points at 800 random bytes less `0` (placed); the word at `p17+0x18` points at 800 random bytes less `0` (placed); the word at `p18+0x18` points at 800 random bytes less `0` (placed); the word at `p19+0x18` points at 800 random bytes less `0` (placed); the word at `p20+0x18` points at 800 random bytes less `0` (placed); the word at `p21+0x18` points at 800 random bytes less `0` (placed); the word at `p22+0x18` points at 800 random bytes less `0` (placed); the word at `p23+0x18` points at 800 random bytes less `0` (placed); the word at `p24+0x18` points at 800 random bytes less `0` (placed); the word at `m+0x18` points at 64 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1, 2, 3, 4, 5, 6, 50, 99, 100, 103, 65535] (90%); stub 3 answered from [0, 0, 1] (90%); stub 5 answered from [0, 1, 0] (90%); stub 6 answered from [0, 1, 1] (90%); stub 8 answered from [0, 1, 1] (90%); stub 9 answered from [0, 0, 1] (90%); stub 4 answered from [0, 0, 1] (90%); stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 2 values in call order, printed after the answers; stub 5 answers 8 values in call order, printed after the answers; stub 6 answers 2 values in call order, printed after the answers; stub 4 answers 4 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 2816 bytes under the call is zeroed before each row; the call ends where stub 11 is first entered (the function is swept to that callee); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0xa40:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_take_damage_head.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The head slice: the call ends where the share (slot `+0x11c`, stub 11) is first entered.
                // Window layout. `m` the struck object (`+8` flags, `+9` owner, `+0x10`/`+0x14` its position
                // xor 0x63637, `+0x24` damage, `+0x3b` the carried sixteenths); `c[0]` the log cell, `c[1]`
                // the game (`+0x820` semaphore, `+0x2b` difficulty, `+0x550` frame), `c[3]` the world (`+0`
                // width), `c[4]` the `div_3_table`. `p[2 * who]` and `p[2 * who + 1]` the slices of leader
                // `who` (`+0`, `+0xa40`), `p[8]` the world's cells (28 bytes each, owner at `+0xf`),
                // `p[9 + 4 * pl + j]` `objects[pl][j]` and `p[25 + 4 * pl + j]` its data, `p[17 + 4 * pl + j]`
                // `units[pl][j]` and `p[33 + 4 * pl + j]` its data, `p[41]` `this`'s data. `a`: the nine
                // arguments, the stubs' answers (`a[9..21]`), every `--seq` run (`SEQ_AT`) and the `+0xb8`
                // flag.
                //
                // **The port is the crate's own head**, `crate::take_damage_view::take_damage_head`; here
                // the view is the emulated original's windows.
                use crate::take_damage_view::{HeadOut, TakeDamageHead, TdArgs, take_damage_head};
                const SEQ_AT: [(usize, usize, usize); 6] = [
                    (1, 21, 4),
                    (2, 25, 4),
                    (3, 29, 2),
                    (5, 31, 8),
                    (6, 39, 2),
                    (4, 41, 4),
                ];
                const BASED: usize = 45;
                const KEY: i32 = 0x63637;
                const RANDOM_CELL: i32 = 14_908_044;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 12],
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
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
                let mut v = V {
                    a,
                    m: &mut *m,
                    c: &*c,
                    p: &mut *p,
                    log: &mut log,
                    turn: [0; 12],
                };
                let args = TdArgs {
                    p: a[..9].try_into().expect("nine arguments"),
                };
                let r = match take_damage_head(&mut v, &args) {
                    HeadOut::Return(r) => i64::from(r),
                    HeadOut::Accumulate { .. } => {
                        v.rec(11, "m+0x0", &[0]);
                        i64::from(v.ans(11))
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
        assert_eq!(log, want_log, "take_damage — the recorded calls, row `{line}`");
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
