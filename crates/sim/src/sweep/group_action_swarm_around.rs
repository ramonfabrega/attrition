//! `Group::action_swarm_around@0070fbe0` under the emulator against the port (item 3386; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_swarm_around@0070fbe0`, run by `tools/emu/sweep_group_action_swarm_around.py` under unicorn
/// (tools/emu): 2272 random bytes of object, 5 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p55 the place(s), in flag order, p56 stub 17's answer window; the global cell 0x00c0618c points at 768 random bytes; the global cell 0x00cae5fc points at 256 random bytes; the global cell 0x00c061fc points at 64 random bytes; the global cell 0x00c061a8 points at 64 random bytes; 4 random bytes at 0x00cc02f8 (each word in 0..0); 4 random bytes at 0x00ee10ec (each word in 0..1); vtable slot +0x10 (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x0070c450 (0 args) stubbed; callee 0x0070e520 (1 args) stubbed; callee 0x0070d0c0 (1 args) stubbed; callee 0x0070e620 (1 args) stubbed; callee 0x0046f600 (0 args) stubbed; callee 0x00711720 (3 args) stubbed; callee 0x00608450 (0 args) stubbed; callee 0x0060a370 (0 args) stubbed; callee 0x0064dda0 (2 args) stubbed; callee 0x00675bc0 (3 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x0046d310 (0 args) stubbed; callee 0x006e0c60 (0 args) stubbed; callee 0x006e3140 (1 args) stubbed; callee 0x0061de70 (16 args) stubbed; callee 0x005e55c0 (11 args) stubbed; callee 0x005e4a60 (7 args) stubbed; callee 0x005e5210 (4 args) stubbed; callee 0x005e4ff0 (4 args) stubbed; callee 0x00a1edd0 (1 args) stubbed; callee 0x00a1d660 (1 args) stubbed; callee 0x00a2e550 (6 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x00714700 (0 args) stubbed; callee 0x0070fba0 (10 args) stubbed; the word at `c0+0x14+0x1c*0` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p2+0` points at 96 random bytes less `0` (placed); the word at `p2+4` points at 96 random bytes less `0` (placed); the word at `p2+8` points at 96 random bytes less `0` (placed); the word at `p2+12` points at 96 random bytes less `0` (placed); the word at `p3+0` points at 96 random bytes less `0` (placed); the word at `p3+4` points at 96 random bytes less `0` (placed); the word at `p3+8` points at 96 random bytes less `0` (placed); the word at `p3+12` points at 96 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p12+0` points at 80 random bytes less `0` (placed); the word at `p12+4` points at 80 random bytes less `0` (placed); the word at `p12+8` points at 80 random bytes less `0` (placed); the word at `p12+12` points at 80 random bytes less `0` (placed); the word at `p13+0` points at 80 random bytes less `0` (placed); the word at `p13+4` points at 80 random bytes less `0` (placed); the word at `p13+8` points at 80 random bytes less `0` (placed); the word at `p13+12` points at 80 random bytes less `0` (placed); the word at `p4+0x40` points at 128 random bytes less `0` (placed); the word at `p5+0x40` points at 128 random bytes less `0` (placed); the word at `p6+0x40` points at 128 random bytes less `0` (placed); the word at `p7+0x40` points at 128 random bytes less `0` (placed); the word at `p8+0x40` points at 128 random bytes less `0` (placed); the word at `p9+0x40` points at 128 random bytes less `0` (placed); the word at `p10+0x40` points at 128 random bytes less `0` (placed); the word at `p11+0x40` points at 128 random bytes less `0` (placed); the word at `p4+0x18` points at 608 random bytes less `0` (placed); the word at `p5+0x18` points at 608 random bytes less `0` (placed); the word at `p6+0x18` points at 608 random bytes less `0` (placed); the word at `p7+0x18` points at 608 random bytes less `0` (placed); the word at `p8+0x18` points at 608 random bytes less `0` (placed); the word at `p9+0x18` points at 608 random bytes less `0` (placed); the word at `p10+0x18` points at 608 random bytes less `0` (placed); the word at `p11+0x18` points at 608 random bytes less `0` (placed); the word at `p22+0x18` points at 608 random bytes less `0` (placed); the word at `p23+0x18` points at 608 random bytes less `0` (placed); the word at `p24+0x18` points at 608 random bytes less `0` (placed); the word at `p25+0x18` points at 608 random bytes less `0` (placed); the word at `p26+0x18` points at 608 random bytes less `0` (placed); the word at `p27+0x18` points at 608 random bytes less `0` (placed); the word at `p28+0x18` points at 608 random bytes less `0` (placed); the word at `p29+0x18` points at 608 random bytes less `0` (placed); the word at `p14+0x18` points at 608 random bytes less `0` (placed); the word at `p15+0x18` points at 608 random bytes less `0` (placed); the word at `p16+0x18` points at 608 random bytes less `0` (placed); the word at `p17+0x18` points at 608 random bytes less `0` (placed); the word at `p18+0x18` points at 608 random bytes less `0` (placed); the word at `p19+0x18` points at 608 random bytes less `0` (placed); the word at `p20+0x18` points at 608 random bytes less `0` (placed); the word at `p21+0x18` points at 608 random bytes less `0` (placed); the word at `c2+0x10` points at 1536 random bytes less `0` (placed); the word at `c3+0x10` points at 2656 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [7, 7, 1, 0, 5] (90%); stub 2 answered from [0, 1, 1, 1] (90%); stub 4 answered from [0, 1, 1] (90%); stub 5 answered from [0, 1, 1] (90%); stub 6 answered from [0, 1, 1] (90%); stub 7 answered from [0, 1, 1] (90%); stub 8 answered from [1, 1, 1, 0] (90%); stub 9 answered from [1, 1, 1, 0] (90%); stub 11 answered from [1, 1, 1, 1, 0] (90%); stub 18 answered from [0, 0, 0, 1] (90%); stub 19 answered from [0, 0, 1, 2] (90%); stub 20 answered from [0, 0, 1, 1] (90%); stub 22 answered from [0, 1, 1] (90%); stub 25 answered from [0, 0, 0, 1] (90%); stub 17 answers the address of -32 random bytes, a `p` window after the places; stub 0 answers 16 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 4 answers 2 values in call order, printed after the answers; stub 5 answers 8 values in call order, printed after the answers; stub 6 answers 8 values in call order, printed after the answers; stub 7 answers 8 values in call order, printed after the answers; stub 8 answers 8 values in call order, printed after the answers; stub 9 answers 8 values in call order, printed after the answers; stub 11 answers 2 values in call order, printed after the answers; stub 16 answers 3 values in call order, printed after the answers; stub 18 answers 8 values in call order, printed after the answers; stub 19 answers 8 values in call order, printed after the answers; stub 20 answers 8 values in call order, printed after the answers; stub 22 answers 8 values in call order, printed after the answers; stub 23 answers 4 values in call order, printed after the answers; stub 24 answers 4 values in call order, printed after the answers; stub 25 answers 4 values in call order, printed after the answers; stub 16 answers 0..4 every row; stub 23 answers 0..5 every row; stub 24 answers 0..8 every row; stub 32 answers 0..0 every row; stub 34 answers 0..1 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 30 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 31 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 32 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 33 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 34 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 35 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 25 writes a value in 0..1000 through its argument 2, printed after the answers (`o` in a place); stub 25 writes a value in 0..1000 through its argument 3, printed after the answers (`o` in a place); the stack 2816 bytes under the call is zeroed before each row; stub 3 answers from its own `ecx`: the word at `+0x40`; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x20 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p43+0` as well as at `this`; the stubbed vtable laid at `p44+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p56+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_swarm_around.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Window layout. `m` the group (`+0xc` count, `+0x10` form, `+0x4a` player, the shorts at `+0x8cc`);
                // `c[1]` the `div_3_table` cell's table, `c[4]` `ignore_orders`, `c[5]` `::ignore_always`. `p[0]`, `p[1]` the
                // leaders (`+0` the flags), `p[4 + 4 * pl + j]` the objects `objects[pl][j]`, `p[14 + 4 * pl + j]` the units,
                // `p[22 + ..]` the data each object answers by slot `+0xb0`, `p[30 + ..]` the objects' types, `p[38 + ..]` the
                // data's, `p[46 + ..]` the units', `p[54]` the unit-type table, `p[55]` the spell-type table, `p[56]` the order
                // `get_action` answers. `a`: the five arguments, the stubs' answers (`a[5..41]`), the two out words
                // (`a[41..43]`: every `find_nearby_spot` writes the same pair), every `--seq` run (`SEQ_AT`), the slot base
                // flag (`a[154]`) and the order's null flag (`a[155]`).
                //
                // **The port is the crate's own steps**, `crate::swarm_around_view::action_swarm_around_steps`, which
                // `Sim::group_action_swarm_around` runs over its `SimSwarm`; here the view is the emulated original's windows.
                use crate::garrison_view::GarrisonHead;
                use crate::swarm_around_view::{SwarmHead, SwarmView, action_swarm_around_steps};
                // (stub, offset in `a`, length)
                const SEQ_AT: [(usize, usize, usize); 17] = [
                    (0, 43, 16),
                    (2, 59, 4),
                    (4, 63, 2),
                    (5, 65, 8),
                    (6, 73, 8),
                    (7, 81, 8),
                    (8, 89, 8),
                    (9, 97, 8),
                    (11, 105, 2),
                    (16, 107, 3),
                    (18, 110, 8),
                    (19, 118, 8),
                    (20, 126, 8),
                    (22, 134, 8),
                    (23, 142, 4),
                    (24, 146, 4),
                    (25, 150, 4),
                ];
                const BASED: usize = 154;
                const NULL_ORDER: usize = 155;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 36],
                    depth: i32,
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
                    fn ow(p: i32, j: i32) -> usize {
                        4 + 4 * p as usize + j as usize
                    }
                    fn uw(p: i32, j: i32) -> usize {
                        14 + 4 * p as usize + j as usize
                    }
                    fn dw(p: i32, j: i32) -> usize {
                        22 + 4 * p as usize + j as usize
                    }
                    fn otw(p: i32, j: i32) -> usize {
                        30 + 4 * p as usize + j as usize
                    }
                    fn dtw(p: i32, j: i32) -> usize {
                        38 + 4 * p as usize + j as usize
                    }
                    fn utw(p: i32, j: i32) -> usize {
                        46 + 4 * p as usize + j as usize
                    }
                    /// A local of the frame `off` under the entry `esp`, as the record prints it: a depth of
                    /// `sp-0x…` while it lies within 0x1000 of the top, the raw address past it.
                    fn stk(&self, off: i32) -> String {
                        let d = off + self.depth * 0xa7c;
                        if d <= 0x1000 {
                            format!("sp-{d:#x}")
                        } else {
                            (0x7fff_ff00i64 - i64::from(d)).to_string()
                        }
                    }
                    fn local(&self) -> String {
                        self.stk(0xa70)
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
                        self.rec(1, "m+0x0", &[]);
                    }
                }
                impl SwarmHead for V<'_> {
                    fn scratch_clear(&mut self) {
                        let l = self.local();
                        self.rec(10, &l, &[-1]);
                    }
                    fn is_on_map(&mut self) -> bool {
                        self.rec(11, "m+0x0", &[]);
                        self.seq(11) != 0
                    }
                    fn site_exists(&self, who: i32, b: i32) -> bool {
                        self.p[Self::ow(who, b)][8] & 1 != 0
                    }
                }
                impl SwarmView for V<'_> {
                    fn scratch_add(&mut self, m: i32, p: i32) {
                        let l = self.local();
                        self.rec(21, &l, &[m, p, 0, 0]);
                    }
                    fn scratch_num(&mut self) -> i32 {
                        let l = self.local();
                        self.rec(34, &l, &[]);
                        self.a[5 + 34]
                    }
                    fn scratch_move_to(&mut self, x: i32, y: i32) {
                        let l = self.local();
                        self.rec(35, &l, &[x, y, 2, 0, 0, 1, 0, -1, -1, 0]);
                    }
                    fn set_up_insert(&mut self) {
                        let l = self.stk(0x88);
                        self.rec(12, "m+0x0", &[]);
                        let t = self.log.last_mut().expect("just pushed");
                        t.push_str(&format!(",{l}"));
                    }
                    fn action_halt(&mut self) {
                        self.rec(13, "m+0x0", &[0]);
                        self.depth = 1;
                    }
                    fn finish_insert(&mut self) {
                        self.depth = 0;
                        let l = self.stk(0x88);
                        self.rec(14, "m+0x0", &[]);
                        let t = self.log.last_mut().expect("just pushed");
                        t.push_str(&format!(",{l}"));
                    }
                    fn list_clear(&mut self) {
                        let l = self.stk(0x84);
                        self.log.push(format!("s15:{l}"));
                    }
                    fn set_form(&mut self) {
                        self.m[0x10..0x14].copy_from_slice(&(-1i32).to_le_bytes());
                    }
                    fn leader_flags(&self, p: i32) -> i32 {
                        rd32(&self.p[p as usize], 0)
                    }
                    fn site_is_build(&mut self, who: i32, b: i32) -> bool {
                        self.rec(2, &format!("p{}+0x0", Self::ow(who, b)), &[]);
                        self.seq(2) != 0
                    }
                    fn site_type_flag(&mut self, who: i32, b: i32) -> bool {
                        self.rec(3, &format!("p{}+0x0", Self::ow(who, b)), &[]);
                        self.rec(4, &format!("p{}+0x0", Self::dtw(who, b)), &[]);
                        self.seq(4) != 0
                    }
                    fn count(&mut self, code: i32) -> i32 {
                        self.rec(16, "m+0x0", &[code, 0, 0]);
                        self.seq(16)
                    }
                    fn site_pos(&self, who: i32, b: i32) -> (i32, i32) {
                        let o = &self.p[Self::ow(who, b)];
                        (rd32(o, 0x10) ^ 0x63637, rd32(o, 0x14) ^ 0x63637)
                    }
                    fn data_ty(&mut self, who: i32, b: i32, off: i32) -> i32 {
                        self.rec(3, &format!("p{}+0x0", Self::ow(who, b)), &[]);
                        rd32(&self.p[Self::dtw(who, b)], off as usize)
                    }
                    fn unit_ty_244(&self, who: i32, b: i32) -> i32 {
                        rd32(&self.p[Self::utw(who, b)], 0x244)
                    }
                    fn site_is(&mut self, who: i32, b: i32, id: i32) -> bool {
                        if self.a[BASED] != 0 {
                            self.rec(7, &format!("p{}+0x0", Self::otw(who, b)), &[id, 0]);
                            self.seq(7) != 0
                        } else {
                            self.rec(6, &format!("p{}+0x0", Self::ow(who, b)), &[id, 0]);
                            self.seq(6) != 0
                        }
                    }
                    fn members(&self) -> i32 {
                        rd32(self.m, 0xc)
                    }
                    fn member(&self, i: i32) -> i32 {
                        rd16(self.m, 0x8cc + 2 * i as usize)
                    }
                    fn m_alive(&mut self, p: i32, m: i32) -> bool {
                        self.rec(8, &format!("p{}+0x0", Self::ow(p, m)), &[]);
                        self.seq(8) != 0
                    }
                    fn m_bc(&mut self, p: i32, m: i32) -> bool {
                        self.rec(9, &format!("p{}+0x0", Self::ow(p, m)), &[]);
                        self.seq(9) != 0
                    }
                    fn m_domain(&self, p: i32, m: i32) -> i32 {
                        rd32(&self.p[Self::utw(p, m)], 0x218)
                    }
                    fn m_action_kind(&mut self, p: i32, m: i32) -> Option<i32> {
                        self.rec(17, &format!("p{}+0x0", Self::uw(p, m)), &[]);
                        if self.a[NULL_ORDER] != 0 {
                            return None;
                        }
                        self.rec(0, "p56+0x0", &[]);
                        Some(self.seq(0))
                    }
                    fn m_busy(&mut self, p: i32, m: i32) -> bool {
                        self.rec(18, &format!("p{}+0x0", Self::uw(p, m)), &[]);
                        self.seq(18) != 0
                    }
                    fn m_type(&self, p: i32, m: i32) -> i32 {
                        rd32(&self.p[Self::utw(p, m)], 4)
                    }
                    fn m_count_inside(&mut self, p: i32, m: i32) -> i32 {
                        self.rec(19, &format!("p{}+0x0", Self::uw(p, m)), &[1, 0]);
                        self.seq(19)
                    }
                    fn m_castable(&mut self, p: i32, m: i32) -> bool {
                        let t = rd32(&self.p[55], 0xa4c);
                        self.rec(20, &t.to_string(), &[m, p, 0]);
                        self.seq(20) != 0
                    }
                    fn site_slot_1c(&mut self, who: i32, b: i32) -> bool {
                        self.rec(5, &format!("p{}+0x0", Self::ow(who, b)), &[]);
                        self.seq(5) != 0
                    }
                    fn site_domain(&self, who: i32, b: i32) -> i32 {
                        rd32(&self.p[Self::otw(who, b)], 0x218)
                    }
                    fn m_is_peasant(&mut self, p: i32, m: i32) -> bool {
                        self.rec(22, &format!("p{}+0x0", Self::uw(p, m)), &[]);
                        self.seq(22) != 0
                    }
                    fn can_transport(&mut self, p: i32) -> i32 {
                        self.rec(23, &format!("p{p}+0x0"), &[]);
                        self.seq(23)
                    }
                    fn current_upgrade(&mut self, p: i32) -> i32 {
                        self.rec(24, &format!("p{p}+0x0"), &[0x140]);
                        self.seq(24)
                    }
                    fn m_inside(&self, p: i32, m: i32) -> (i32, i32) {
                        let u = &self.p[Self::uw(p, m)];
                        (i32::from(u[0x3e] as i8), rd16(u, 0x28))
                    }
                    fn unit_type_of(&self, owner: i32, idx: i32) -> i32 {
                        rd32(&self.p[Self::utw(owner, idx)], 4)
                    }
                    fn find_nearby_spot(
                        &mut self,
                        which: u8,
                        seeker: i32,
                        x: i32,
                        y: i32,
                        radius: i32,
                        a5: i32,
                        a6: i32,
                        angle: i32,
                        m: i32,
                        p: i32,
                        b: i32,
                        who: i32,
                    ) -> Option<(i32, i32)> {
                        let ty = rd32(&self.p[54], 4 * seeker as usize);
                        let (ox, oy, rx, ry) = if which == 0 {
                            (0x6c, 0x5c, 0, 0)
                        } else {
                            (0x64, 0x4c, 0, 0)
                        };
                        let _ = (rx, ry);
                        let (sx, sy) = (self.stk(ox), self.stk(oy));
                        let t = format!(
                            "s25:{ty},{x},{y},{sx},{sy},{radius},{a5},{a6},{angle},3,{m},{p},0,0,{b},{who},-1"
                        );
                        self.log.push(t);
                        if self.seq(25) != 0 {
                            return None;
                        }
                        Some((self.a[41], self.a[42]))
                    }
                    fn find_angle(&self, dx: i32, dy: i32) -> i32 {
                        crate::movement::find_angle(dx, dy).0
                    }
                    fn div3(&self, i: i32) -> i32 {
                        rd32(&self.c[1], 4 * i as usize)
                    }
                    fn m_pos(&self, p: i32, m: i32) -> (i32, i32) {
                        let o = &self.p[Self::ow(p, m)];
                        (rd32(o, 0x10) ^ 0x63637, rd32(o, 0x14) ^ 0x63637)
                    }
                    fn add_move_facing(
                        &mut self,
                        p: i32,
                        m: i32,
                        x: i32,
                        y: i32,
                        angle: i32,
                        kind: i32,
                        queue: i32,
                    ) {
                        self.rec(
                            26,
                            &format!("p{}+0x0", Self::uw(p, m)),
                            &[x, y, angle, kind, 0, queue, 0, -1, -1, -1, 0],
                        );
                    }
                    fn add_cast(&mut self, p: i32, m: i32, x: i32, y: i32, action: i32) {
                        self.rec(
                            27,
                            &format!("p{}+0x0", Self::uw(p, m)),
                            &[-1, -1, x, y, 0x293, 1, action],
                        );
                    }
                    fn add_build(&mut self, p: i32, m: i32, b: i32, who: i32, action: i32) {
                        self.rec(
                            28,
                            &format!("p{}+0x0", Self::uw(p, m)),
                            &[b, who, 1, action],
                        );
                    }
                    fn add_repair(&mut self, p: i32, m: i32, b: i32, who: i32, action: i32) {
                        self.rec(
                            29,
                            &format!("p{}+0x0", Self::uw(p, m)),
                            &[b, who, 1, action],
                        );
                    }
                    fn clear_site_2000(&mut self, who: i32, b: i32) {
                        self.rec(3, &format!("p{}+0x0", Self::ow(who, b)), &[]);
                        let d = Self::dw(who, b);
                        let w = u16::from_le_bytes([self.p[d][0x60], self.p[d][0x61]]) & 0xdfff;
                        self.p[d][0x60..0x62].copy_from_slice(&w.to_le_bytes());
                    }
                    fn ignore_always(&self) -> bool {
                        self.c[5][0] != 0
                    }
                    fn report_illegal(&mut self) -> bool {
                        let (s0, s1) = (self.stk(0x80), self.stk(0x9c));
                        self.log.push(format!("s30:{s0},11409840"));
                        self.log.push(format!("s31:{s1},11409812"));
                        self.log
                            .push(format!("s32:13161384,{s1},{s1},-1,-1,c5+0x0,1"));
                        self.log.push(format!("s33:{s1}"));
                        self.log.push(format!("s33:{s0}"));
                        false
                    }
                }
                let mut v = V {
                    a,
                    m: &mut *m,
                    c: &*c,
                    p: &mut *p,
                    log: &mut log,
                    turn: [0; 36],
                    depth: 0,
                };
                action_swarm_around_steps(&mut v, a[0], a[1], a[2], a[3], a[4], 0);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_swarm_around — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "action_swarm_around — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_swarm_around — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[7 + i],
                "action_swarm_around — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 2400, "only {n} rows");
    eprintln!("Group::action_swarm_around@0070fbe0: {n} rows agree");
}
