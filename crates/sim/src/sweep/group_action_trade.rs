//! `Group::action_trade@00701cc0` under the emulator against the port (item 3381; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_trade@00701cc0`, run by `tools/emu/sweep_group_action_trade.py` under unicorn
/// (tools/emu): 2272 random bytes of object, 5 argument(s), `p` windows in order: p0..p52 the place(s), in flag order; the global cell 0x00c0618c points at 768 random bytes; the global cell 0x00cae5fc points at 64 random bytes; the global cell 0x00c06188 points at 64 random bytes; the global cell 0x00c061b8 points at 64 random bytes; 4 random bytes at 0x00cc02f8 (each word in 0..0); vtable slot +0x14 (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0x24 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0x28 (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; callee 0x00711720 (3 args) stubbed; callee 0x0070e520 (1 args) stubbed; callee 0x0070d0c0 (1 args) stubbed; callee 0x0070e620 (1 args) stubbed; callee 0x0046f600 (0 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x00680f90 (1 args) stubbed; callee 0x005e4dc0 (6 args) stubbed; the word at `c0+0x14+0x1c*0` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 96 random bytes less `0` (placed); the word at `p0+4` points at 96 random bytes less `0` (placed); the word at `p0+8` points at 96 random bytes less `0` (placed); the word at `p0+12` points at 96 random bytes less `0` (placed); the word at `p1+0` points at 96 random bytes less `0` (placed); the word at `p1+4` points at 96 random bytes less `0` (placed); the word at `p1+8` points at 96 random bytes less `0` (placed); the word at `p1+12` points at 96 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p10+0` points at 64 random bytes less `0` (placed); the word at `p10+4` points at 64 random bytes less `0` (placed); the word at `p10+8` points at 64 random bytes less `0` (placed); the word at `p10+12` points at 64 random bytes less `0` (placed); the word at `p11+0` points at 64 random bytes less `0` (placed); the word at `p11+4` points at 64 random bytes less `0` (placed); the word at `p11+8` points at 64 random bytes less `0` (placed); the word at `p11+12` points at 64 random bytes less `0` (placed); the word at `p2+0x40` points at 64 random bytes less `0` (placed); the word at `p3+0x40` points at 64 random bytes less `0` (placed); the word at `p4+0x40` points at 64 random bytes less `0` (placed); the word at `p5+0x40` points at 64 random bytes less `0` (placed); the word at `p6+0x40` points at 64 random bytes less `0` (placed); the word at `p7+0x40` points at 64 random bytes less `0` (placed); the word at `p8+0x40` points at 64 random bytes less `0` (placed); the word at `p9+0x40` points at 64 random bytes less `0` (placed); the word at `p20+0x18` points at 32 random bytes less `0` (placed); the word at `p21+0x18` points at 32 random bytes less `0` (placed); the word at `p22+0x18` points at 32 random bytes less `0` (placed); the word at `p23+0x18` points at 32 random bytes less `0` (placed); the word at `p24+0x18` points at 32 random bytes less `0` (placed); the word at `p25+0x18` points at 32 random bytes less `0` (placed); the word at `p26+0x18` points at 32 random bytes less `0` (placed); the word at `p27+0x18` points at 32 random bytes less `0` (placed); the word at `p2+0x18` points at 32 random bytes less `0` (placed); the word at `p3+0x18` points at 32 random bytes less `0` (placed); the word at `p4+0x18` points at 32 random bytes less `0` (placed); the word at `p5+0x18` points at 32 random bytes less `0` (placed); the word at `p6+0x18` points at 32 random bytes less `0` (placed); the word at `p7+0x18` points at 32 random bytes less `0` (placed); the word at `p8+0x18` points at 32 random bytes less `0` (placed); the word at `p9+0x18` points at 32 random bytes less `0` (placed); the word at `p12+0x18` points at 32 random bytes less `0` (placed); the word at `p13+0x18` points at 32 random bytes less `0` (placed); the word at `p14+0x18` points at 32 random bytes less `0` (placed); the word at `p15+0x18` points at 32 random bytes less `0` (placed); the word at `p16+0x18` points at 32 random bytes less `0` (placed); the word at `p17+0x18` points at 32 random bytes less `0` (placed); the word at `p18+0x18` points at 32 random bytes less `0` (placed); the word at `p19+0x18` points at 32 random bytes less `0` (placed); the word at `c3+0x10` points at 560 random bytes less `0` (placed); slot +0x24 laid as 0x006424e0 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xd0 laid as 0x0046ce90 or its stub, a flag after the answers; stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [1, 1, 1, 0] (90%); stub 4 answered from [0, 1, 1] (90%); stub 5 answered from [0, 1, 1] (90%); stub 6 answered from [0, 1, 1] (90%); stub 7 answered from [1, 1, 1, 0] (90%); stub 8 answered from [1, 1, 1, 0] (90%); stub 9 answered from [0, 1, 1] (90%); stub 10 answered from [0, 1] (90%); stub 11 answered from [0, 1, 1] (90%); stub 12 answered from [0, 0, 1, 3] (90%); stub 18 answered from [0, 1, 1] (90%); stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 4 answers 8 values in call order, printed after the answers; stub 5 answers 8 values in call order, printed after the answers; stub 6 answers 8 values in call order, printed after the answers; stub 7 answers 8 values in call order, printed after the answers; stub 8 answers 8 values in call order, printed after the answers; stub 9 answers 8 values in call order, printed after the answers; stub 10 answers 8 values in call order, printed after the answers; stub 11 answers 8 values in call order, printed after the answers; stub 12 answers 2 values in call order, printed after the answers; stub 18 answers 6 values in call order, printed after the answers; stub 17 answers 6 values in call order, printed after the answers; stub 17 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 17 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 17 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); the stack 2816 bytes under the call is zeroed before each row; stub 3 answers from its own `ecx`: the word at `+0x40`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p43+0` as well as at `this`; the stubbed vtable laid at `p44+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; the stubbed vtable laid at `p48+0` as well as at `this`; the stubbed vtable laid at `p49+0` as well as at `this`; the stubbed vtable laid at `p50+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_trade.py") else {
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
                // Window layout. `m` the group (`+0xc` count, `+0x10` form, `+0x4a` player, the shorts at `+0x8cc`);
                // `c[0]` the objects cell, `c[1]` the `div_3_table`, `c[2]` the world, `c[3]` the regions cell,
                // `c[4]` `ignore_orders`. `p[2 + 4 * pl + j]` the objects `objects[pl][j]`, `p[12 + 4 * pl + j]` the
                // units, `p[20 + 4 * pl + j]` the data each object answers by slot `+0xb0` (at `+0x40`),
                // `p[28 + ..]` that data's type, `p[36 + ..]` the object's own type, `p[44 + ..]` the unit's type
                // (each at `+0x18`), `p[52]` the regions. `a`: the five arguments, the stubs' answers
                // (`a[5..25]`), every `--seq` run (`SEQ_AT`) and the three `--slotbase` flags.
                //
                // **The port is the crate's own steps**, `crate::trade_view::action_trade_steps`, over a view of
                // the emulated original's windows; the crate has no `Group::action_trade` to call (see that
                // module's head).
                use crate::trade_view::{GarrisonHead, TradeView, action_trade_steps};
                const SEQ_AT: [(usize, usize, usize); 13] = [
                    (1, 25, 8),
                    (2, 33, 8),
                    (4, 41, 8),
                    (5, 49, 8),
                    (6, 57, 8),
                    (7, 65, 8),
                    (8, 73, 8),
                    (9, 81, 8),
                    (10, 89, 8),
                    (11, 97, 8),
                    (12, 105, 2),
                    (18, 107, 6),
                    (17, 113, 6),
                ];
                const FLAG_24: usize = 119;
                const FLAG_B8: usize = 120;
                const FLAG_D0: usize = 121;
                // One call frame of the original: five arguments, the return address and its saved registers.
                const FRAME: i32 = 0x6c;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 19],
                    depth: i32,
                    deref: [Vec<String>; 2],
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
                    fn rec(&mut self, k: usize, ecx: &str, args: &[i32]) {
                        let mut t = format!("s{k}:{ecx}");
                        for v in args {
                            t.push_str(&format!(",{v}"));
                        }
                        self.log.push(t);
                    }
                    fn ow(p: i32, o: i32) -> usize {
                        2 + 4 * p as usize + o as usize
                    }
                    fn uw(p: i32, u: i32) -> usize {
                        12 + 4 * p as usize + u as usize
                    }
                    fn dw(p: i32, o: i32) -> usize {
                        20 + 4 * p as usize + o as usize
                    }
                    fn tdw(p: i32, o: i32) -> usize {
                        28 + 4 * p as usize + o as usize
                    }
                    fn tow(p: i32, o: i32) -> usize {
                        36 + 4 * p as usize + o as usize
                    }
                    fn tuw(p: i32, u: i32) -> usize {
                        44 + 4 * p as usize + u as usize
                    }
                    // `WallData::is_trade`, called on `this` (slot `+0x24`) or inlined by the caller when the
                    // slot is that function: `is(0x19e, 0)` through the slot `+0xb8`, itself inlined to the type's
                    // slot `+0x60` when it is `ObjectData::is`.
                    fn is_trade_on(&mut self, this_w: usize, type_w: usize) -> bool {
                        if self.a[FLAG_24] != 0 {
                            if self.a[FLAG_B8] != 0 {
                                self.rec(6, &format!("p{type_w}+0x0"), &[0x19e, 0]);
                                self.seq(6) != 0
                            } else {
                                self.rec(5, &format!("p{this_w}+0x0"), &[0x19e, 0]);
                                self.seq(5) != 0
                            }
                        } else {
                            self.rec(4, &format!("p{this_w}+0x0"), &[]);
                            self.seq(4) != 0
                        }
                    }
                    // `WorldData::get_tregion(&tbl[x], &tbl[y])`, the locals' frame offsets the call site's.
                    fn tregion(&mut self, p: i32, o: i32, l0: i32, l1: i32) -> i32 {
                        let w = &self.p[Self::ow(p, o)];
                        let at = |v: i32| 4 * ((v ^ 0x63637) >> 6) as usize;
                        let (x, y) = (at(rd32(w, 0x10)), at(rd32(w, 0x14)));
                        let hex = |o: usize| {
                            self.c[1][o..o + 4]
                                .iter()
                                .map(|b| format!("{b:02x}"))
                                .collect::<String>()
                        };
                        let (hx, hy) = (hex(x), hex(y));
                        let sp = self.depth * FRAME;
                        self.log
                            .push(format!("s17:c2+0x0,sp-0x{:x},sp-0x{:x}", l0 + sp, l1 + sp));
                        self.deref[0].push(hx);
                        self.deref[1].push(hy);
                        self.seq(17)
                    }
                }
                impl GarrisonHead for V<'_> {
                    fn who(&self) -> i32 {
                        i32::from(self.m[0x4a])
                    }
                    fn ignore_orders(&self) -> bool {
                        rd32(&self.c[4], 0) != 0
                    }
                    fn ignoring_count(&self, _who: i32) -> i32 {
                        unreachable!("the whole-function sweep keeps ignore_orders at 0")
                    }
                    fn ignoring_at(&self, _who: i32, _i: i32) -> i32 {
                        unreachable!("the whole-function sweep keeps ignore_orders at 0")
                    }
                    fn drop_ignored(&mut self, _o: i32, _who: i32) {
                        unreachable!("the whole-function sweep keeps ignore_orders at 0")
                    }
                    fn refresh(&mut self) {
                        self.rec(0, "m+0x0", &[]);
                    }
                }
                impl TradeView for V<'_> {
                    fn set_up_insert(&mut self) {
                        self.log.push("s13:m+0x0,sp-0x60".into());
                    }
                    fn action_halt(&mut self) {
                        self.rec(14, "m+0x0", &[0]);
                        self.depth += 1;
                    }
                    fn finish_insert(&mut self) {
                        self.depth -= 1;
                        self.log.push("s15:m+0x0,sp-0x60".into());
                    }
                    fn list_clear(&mut self) {
                        self.log.push("s16:sp-0x5c".into());
                    }
                    fn set_form(&mut self) {
                        self.m[0x10..0x14].copy_from_slice(&(-1i32).to_le_bytes());
                    }
                    fn obj_slot0c(&mut self, p: i32, o: i32) -> bool {
                        self.rec(1, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                        self.seq(1) != 0
                    }
                    fn obj_slot4c(&mut self, p: i32, o: i32) -> bool {
                        self.rec(2, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                        self.seq(2) != 0
                    }
                    fn data_is_trade(&mut self, p: i32, o: i32) -> bool {
                        self.rec(3, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                        self.is_trade_on(Self::dw(p, o), Self::tdw(p, o))
                    }
                    fn count_type(&mut self, kind: i32, ty: i32) -> i32 {
                        self.rec(12, "m+0x0", &[kind, ty, 0]);
                        self.seq(12)
                    }
                    fn count(&self) -> i32 {
                        rd32(self.m, 0xc)
                    }
                    fn member(&self, i: i32) -> i32 {
                        rd16(self.m, 0x8cc + 2 * i as usize)
                    }
                    fn target_tregion(&mut self, p: i32, o: i32) -> i32 {
                        self.tregion(p, o, 0x34, 0x38)
                    }
                    fn member_tregion(&mut self, p: i32, m: i32) -> i32 {
                        self.tregion(p, m, 0x40, 0x3c)
                    }
                    fn obj_alive(&mut self, p: i32, o: i32) -> bool {
                        self.rec(7, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                        self.seq(7) != 0
                    }
                    fn obj_bc(&mut self, p: i32, o: i32) -> bool {
                        self.rec(8, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                        self.seq(8) != 0
                    }
                    fn obj_is_trade(&mut self, p: i32, o: i32) -> bool {
                        self.is_trade_on(Self::ow(p, o), Self::tow(p, o))
                    }
                    fn obj_is_sea_trade(&mut self, p: i32, o: i32) -> bool {
                        self.rec(9, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                        self.seq(9) != 0
                    }
                    fn unit_is_caravan(&mut self, p: i32, u: i32) -> bool {
                        if self.a[FLAG_D0] != 0 {
                            self.rec(11, &format!("p{}+0x0", Self::tuw(p, u)), &[]);
                            self.seq(11) != 0
                        } else {
                            self.rec(10, &format!("p{}+0x0", Self::uw(p, u)), &[]);
                            self.seq(10) != 0
                        }
                    }
                    fn unit_is_ship(&mut self, p: i32, u: i32) -> bool {
                        if self.a[FLAG_B8] != 0 {
                            self.rec(6, &format!("p{}+0x0", Self::tuw(p, u)), &[0x13e, 0]);
                            self.seq(6) != 0
                        } else {
                            self.rec(5, &format!("p{}+0x0", Self::uw(p, u)), &[0x13e, 0]);
                            self.seq(5) != 0
                        }
                    }
                    fn is_coast(&mut self, unit_region: i32, target_region: i32) -> bool {
                        self.rec(
                            18,
                            &format!("p52+0x{:x}", 0x88 * unit_region),
                            &[target_region],
                        );
                        self.seq(18) != 0
                    }
                    fn add_trade_order(&mut self, p: i32, u: i32, args: [i32; 4], queue: i32) {
                        self.rec(
                            19,
                            &format!("p{}+0x0", Self::uw(p, u)),
                            &[args[0], args[1], args[2], args[3], queue, 1],
                        );
                    }
                }
                let mut v = V {
                    a,
                    m: &mut *m,
                    c: &*c,
                    p: &*p,
                    log: &mut log,
                    turn: [0; 19],
                    depth: 0,
                    deref: [Vec::new(), Vec::new()],
                };
                action_trade_steps(&mut v, a[0], a[1], a[2], a[3], a[4]);
                let [d0, d1] = std::mem::take(&mut v.deref);
                let tail = |d: Vec<String>| {
                    if d.is_empty() {
                        "-".to_string()
                    } else {
                        d.join("/")
                    }
                };
                v.log.push(tail(d0));
                v.log.push(tail(d1));
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_trade — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_trade — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_trade — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[6 + i],
                "action_trade — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1600, "only {n} rows");
    eprintln!("Group::action_trade@00701cc0: {n} rows agree");
}
