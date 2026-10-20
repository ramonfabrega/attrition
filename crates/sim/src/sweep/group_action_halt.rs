//! `Group::action_halt@0070d0c0` under the emulator against the port (item 3379; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_halt@0070d0c0`, run by `tools/emu/sweep_group_action_halt.py` under unicorn
/// (tools/emu): 2272 random bytes of object, 1 argument(s), `p` windows in order: p0..p27 the place(s), in flag order; the global cell 0x00c0618c points at 768 random bytes; 4 random bytes at 0x00cc02f8 (each word in 0..0); vtable slot +0x10 (4 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xc0 (0 args) stubbed; vtable slot +0xd4 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x10c (0 args) stubbed; callee 0x0060a6f0 (0 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; the word at `c0+0x14+0x1c*0` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 160 random bytes less `0` (placed); the word at `p0+4` points at 160 random bytes less `0` (placed); the word at `p0+8` points at 160 random bytes less `0` (placed); the word at `p0+12` points at 160 random bytes less `0` (placed); the word at `p1+0` points at 160 random bytes less `0` (placed); the word at `p1+4` points at 160 random bytes less `0` (placed); the word at `p1+8` points at 160 random bytes less `0` (placed); the word at `p1+12` points at 160 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p10+0` points at 256 random bytes less `0` (placed); the word at `p10+4` points at 256 random bytes less `0` (placed); the word at `p10+8` points at 256 random bytes less `0` (placed); the word at `p10+12` points at 256 random bytes less `0` (placed); the word at `p11+0` points at 256 random bytes less `0` (placed); the word at `p11+4` points at 256 random bytes less `0` (placed); the word at `p11+8` points at 256 random bytes less `0` (placed); the word at `p11+12` points at 256 random bytes less `0` (placed); the word at `p12+0x18` points at 760 random bytes less `0` (placed); the word at `p13+0x18` points at 760 random bytes less `0` (placed); the word at `p14+0x18` points at 760 random bytes less `0` (placed); the word at `p15+0x18` points at 760 random bytes less `0` (placed); the word at `p16+0x18` points at 760 random bytes less `0` (placed); the word at `p17+0x18` points at 760 random bytes less `0` (placed); the word at `p18+0x18` points at 760 random bytes less `0` (placed); the word at `p19+0x18` points at 760 random bytes less `0` (placed); slot +0xc0 laid as 0x0046ce40 or its stub, a flag after the answers; slot +0xd4 laid as 0x0046cea0 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 2 answered from [1, 1, 1, 0] (90%); stub 3 answered from [1, 1, 1, 0] (90%); stub 4 answered from [0, 0, 0, 1] (90%); stub 5 answered from [0, 0, 16, 1] (90%); stub 6 answered from [0, 0, 1] (90%); stub 7 answered from [0, 0, 1] (90%); stub 8 answered from [0, 0, 1] (90%); stub 9 answered from [0, 0, 0, 1] (90%); stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 4 values in call order, printed after the answers; stub 4 answers 4 values in call order, printed after the answers; stub 5 answers 4 values in call order, printed after the answers; stub 6 answers 4 values in call order, printed after the answers; stub 7 answers 4 values in call order, printed after the answers; stub 8 answers 4 values in call order, printed after the answers; stub 9 answers 4 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 2816 bytes under the call is zeroed before each row; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_halt.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Window layout. `m` the group (`+0x49` unit-group byte, `+0x4a` player, `+0xc` count, `+0x10` form,
                // `+0x8cc` the member shorts); `c[0]` `GameAccess::objects`, `c[1]` `ignore_orders` (held 0: the head has
                // its own sweep). `p[0..2]` the players' arrays of objects (`+0x14 + 0x1c * who`), `p[2 + 4 * who + o]` the
                // objects `objects[who][o]`, `p[10..12]` the `units` arrays, `p[12 + 4 * who + u]` the units and
                // `p[20 + 4 * who + u]` their types (`unit +0x18`). `a`: 0 the mask, 1.. the stubs' answers, 14.. each
                // answering stub's `--seq` run (`SEQ_AT`), 46/47/48 the flags that lay the image's `is_plane`,
                // `is_special` and `ObjectData::is` at the unit's vtable (the original then answers inline) rather than the stub.
                //
                // **The port is the crate's own steps**, `crate::halt_view::action_halt_steps`, which
                // `Sim::group_action_halt` runs over its `SimHalt`; here the view is the emulated original's windows.
                use crate::garrison_view::GarrisonHead;
                use crate::halt_view::{HaltView, action_halt_steps};
                const SEQ_AT: [(usize, usize, usize); 8] = [
                    (2, 14, 4),
                    (3, 18, 4),
                    (4, 22, 4),
                    (5, 26, 4),
                    (6, 30, 4),
                    (7, 34, 4),
                    (8, 38, 4),
                    (9, 42, 4),
                ];
                const KEEP_PLANE: usize = 46;
                const KEEP_SPECIAL: usize = 47;
                const KEEP_IS: usize = 48;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 10],
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
                    fn tw(p: i32, u: i32) -> usize {
                        20 + 4 * p as usize + u as usize
                    }
                }
                impl GarrisonHead for V<'_> {
                    fn who(&self) -> i32 {
                        i32::from(self.m[0x4a])
                    }
                    fn ignore_orders(&self) -> bool {
                        rd32(&self.c[1], 0) != 0
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
                        self.rec(1, "m+0x0", &[]);
                    }
                }
                impl HaltView for V<'_> {
                    fn unit_group(&self) -> bool {
                        self.m[0x49] == 0
                    }
                    fn set_form(&mut self) {
                        self.m[0x10..0x14].copy_from_slice(&(-1i32).to_le_bytes());
                    }
                    fn count(&self) -> i32 {
                        rd32(self.m, 0xc)
                    }
                    fn member(&self, i: i32) -> i32 {
                        rd16(self.m, 0x8cc + 2 * i as usize)
                    }
                    fn obj_alive(&mut self, p: i32, o: i32) -> bool {
                        self.rec(2, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                        self.seq(2) != 0
                    }
                    fn obj_bc(&mut self, p: i32, o: i32) -> bool {
                        self.rec(3, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                        self.seq(3) != 0
                    }
                    fn unit_not_plane(&mut self, p: i32, u: i32) -> bool {
                        if self.a[KEEP_PLANE] != 0 {
                            let t = &self.p[Self::tw(p, u)];
                            return rd32(t, 0x218) != 2 || t[0x2b4] & 0x20 != 0;
                        }
                        self.rec(4, &format!("p{}+0x0", Self::uw(p, u)), &[]);
                        self.seq(4) == 0
                    }
                    fn unit_busy(&mut self, p: i32, u: i32) -> bool {
                        self.rec(9, &format!("p{}+0x0", Self::uw(p, u)), &[]);
                        self.seq(9) != 0
                    }
                    fn unit_siege(&mut self, p: i32, u: i32) -> bool {
                        self.rec(8, &format!("p{}+0x0", Self::tw(p, u)), &[]);
                        self.seq(8) != 0
                    }
                    fn unit_special(&mut self, p: i32, u: i32) -> bool {
                        if self.a[KEEP_SPECIAL] != 0 {
                            return rd32(&self.p[Self::tw(p, u)], 0x2b8) & 0x10 != 0;
                        }
                        self.rec(5, &format!("p{}+0x0", Self::uw(p, u)), &[]);
                        self.seq(5) != 0
                    }
                    fn unit_is_3a(&mut self, p: i32, u: i32) -> bool {
                        if self.a[KEEP_IS] != 0 {
                            self.rec(7, &format!("p{}+0x0", Self::tw(p, u)), &[0x3a, 0]);
                            return self.seq(7) != 0;
                        }
                        self.rec(6, &format!("p{}+0x0", Self::uw(p, u)), &[0x3a, 0]);
                        self.seq(6) != 0
                    }
                    fn clear_orders(&mut self, p: i32, u: i32) {
                        let w = Self::uw(p, u);
                        let f = rd32(&self.p[w], 0x68) & 0xfbff_ffffu32 as i32;
                        self.p[w][0x68..0x6c].copy_from_slice(&f.to_le_bytes());
                        self.p[w][0xc0..0xc4].copy_from_slice(&0i32.to_le_bytes());
                        let at = format!("p{w}+0x0");
                        self.rec(10, &at, &[0]);
                        self.rec(11, &at, &[]);
                        self.rec(12, &at, &[]);
                    }
                    fn clear_mask_bit(&mut self, p: i32, u: i32) {
                        let w = Self::uw(p, u);
                        let f = rd32(&self.p[w], 0x68) & 0xffff_feffu32 as i32;
                        self.p[w][0x68..0x6c].copy_from_slice(&f.to_le_bytes());
                    }
                }
                let mut v = V {
                    a,
                    m: &mut *m,
                    c: &*c,
                    p: &mut *p,
                    log: &mut log,
                    turn: [0; 10],
                };
                action_halt_steps(&mut v, a[0]);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_halt — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_halt — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "action_halt — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "action_halt — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Group::action_halt@0070d0c0: {n} rows agree");
}
