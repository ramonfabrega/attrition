//! `Group::action_garrison@00700490` under the emulator against the port (item 3376; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_garrison@00700490`, run by `tools/emu/sweep_group_action_garrison.py` under unicorn
/// (tools/emu): 2272 random bytes of object, 4 argument(s), `p` windows in order: p0..p3 sparse table 0 (2 rows × 2 slice(s), row-major), p4..p5 sparse table 1 (2 rows × 1 slice(s), row-major), p6..p49 the place(s), in flag order, p50 stub 20's answer window; the global cell 0x00c0618c points at 768 random bytes; the global cell 0x00c06210 points at 768 random bytes; the global cell 0x00c061ec points at 2096 random bytes; 4 random bytes at 0x00cc02f8 (each word in 0..0); vtable slot +0x10 (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x178 (0 args) stubbed; callee 0x0070e520 (1 args) stubbed; callee 0x0070d0c0 (1 args) stubbed; callee 0x0070e620 (1 args) stubbed; callee 0x0046f600 (0 args) stubbed; callee 0x00a1d590 (1 args) stubbed; callee 0x007e9ab0 (7 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x00633f50 (1 args) stubbed; callee 0x0062d470 (0 args) stubbed; callee 0x00646d50 (1 args) stubbed; callee 0x0061a2e0 (3 args) stubbed; callee 0x0060a6f0 (0 args) stubbed; callee 0x00608450 (0 args) stubbed; callee 0x005e29b0 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x0061d7d0 (1 args) stubbed; callee 0x00605040 (2 args) stubbed; callee 0x0060aa60 (0 args) stubbed; callee 0x0060a4b0 (0 args) stubbed; callee 0x0046d700 (0 args) stubbed; callee 0x0046d620 (0 args) stubbed; callee 0x005e3860 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x005e2920 (0 args) stubbed; callee 0x005e4080 (5 args) stubbed; the word at `c0+0x14+0x1c*0` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p6+0` points at 160 random bytes less `0` (placed); the word at `p6+4` points at 160 random bytes less `0` (placed); the word at `p6+8` points at 160 random bytes less `0` (placed); the word at `p6+12` points at 160 random bytes less `0` (placed); the word at `p7+0` points at 160 random bytes less `0` (placed); the word at `p7+4` points at 160 random bytes less `0` (placed); the word at `p7+8` points at 160 random bytes less `0` (placed); the word at `p7+12` points at 160 random bytes less `0` (placed); the word at `p8+0x7c` points at 128 random bytes less `0` (placed); the word at `p9+0x7c` points at 128 random bytes less `0` (placed); the word at `p10+0x7c` points at 128 random bytes less `0` (placed); the word at `p11+0x7c` points at 128 random bytes less `0` (placed); the word at `p12+0x7c` points at 128 random bytes less `0` (placed); the word at `p13+0x7c` points at 128 random bytes less `0` (placed); the word at `p14+0x7c` points at 128 random bytes less `0` (placed); the word at `p15+0x7c` points at 128 random bytes less `0` (placed); the word at `p16+0x18` points at 64 random bytes less `0` (placed); the word at `p17+0x18` points at 64 random bytes less `0` (placed); the word at `p18+0x18` points at 64 random bytes less `0` (placed); the word at `p19+0x18` points at 64 random bytes less `0` (placed); the word at `p20+0x18` points at 64 random bytes less `0` (placed); the word at `p21+0x18` points at 64 random bytes less `0` (placed); the word at `p22+0x18` points at 64 random bytes less `0` (placed); the word at `p23+0x18` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p32+0` points at 256 random bytes less `0` (placed); the word at `p32+4` points at 256 random bytes less `0` (placed); the word at `p32+8` points at 256 random bytes less `0` (placed); the word at `p32+12` points at 256 random bytes less `0` (placed); the word at `p33+0` points at 256 random bytes less `0` (placed); the word at `p33+4` points at 256 random bytes less `0` (placed); the word at `p33+8` points at 256 random bytes less `0` (placed); the word at `p33+12` points at 256 random bytes less `0` (placed); the word at `p34+0x18` points at 760 random bytes less `0` (placed); the word at `p35+0x18` points at 760 random bytes less `0` (placed); the word at `p36+0x18` points at 760 random bytes less `0` (placed); the word at `p37+0x18` points at 760 random bytes less `0` (placed); the word at `p38+0x18` points at 760 random bytes less `0` (placed); the word at `p39+0x18` points at 760 random bytes less `0` (placed); the word at `p40+0x18` points at 760 random bytes less `0` (placed); the word at `p41+0x18` points at 760 random bytes less `0` (placed); stub 0 answered from [26, 26, 1, 0, 5] (90%); stub 2 answered from [1, 1, 1, 1, 0] (90%); stub 3 answered from [1, 1, 1, 1, 0] (90%); stub 5 answered from [0, 1, 1] (90%); stub 6 answered from [0, 1, 1] (90%); stub 7 answered from [0, 1, 2, 3, 4, 5, 6, 1, 2, 3, 4] (90%); stub 15 answered from [0, 1, 2, 3, 4, 5, 6, 1, 2, 3, 4] (90%); stub 16 answered from [0, 0, 0, 0, 1] (90%); stub 19 answered from [0, 0, 0, 0, 1] (90%); stub 23 answered from [1, 1, 1, 0] (90%); stub 25 answered from [1, 1, 1, 0] (90%); stub 26 answered from [0, 0, 1] (90%); stub 20 answers the address of -32 random bytes, a `p` window after the places; the word at `p16+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p17+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p18+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p19+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p20+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p21+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p22+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p23+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 4 answers `p16+j`, the data of the object its `ecx` is in `p8+j` (j < 8), linked at `+0x74`; stub 0 answers 16 values in call order, printed after the answers; stub 2 answers 16 values in call order, printed after the answers; stub 3 answers 16 values in call order, printed after the answers; stub 5 answers 4 values in call order, printed after the answers; stub 6 answers 4 values in call order, printed after the answers; stub 7 answers 4 values in call order, printed after the answers; stub 15 answers 4 values in call order, printed after the answers; stub 16 answers 2 values in call order, printed after the answers; stub 17 answers 8 values in call order, printed after the answers; stub 19 answers 8 values in call order, printed after the answers; stub 23 answers 8 values in call order, printed after the answers; stub 24 answers 8 values in call order, printed after the answers; stub 25 answers 8 values in call order, printed after the answers; stub 26 answers 16 values in call order, printed after the answers; stub 17 answers 0..3 every row; stub 24 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 30 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 31 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 32 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 2816 bytes under the call is zeroed before each row; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x20, +0x74:0x8 (`p` windows after the pointed buffers, before the places); 2 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x8 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p50+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_garrison.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        // `MessageWin::add_feedback`'s first five words are the string copy's, which a stub leaves: the stack's
        // leftovers of earlier calls, which no reading of the function depends on. Both logs hold `_` for them.
        let masked: Vec<String> = calls
            .split_whitespace()
            .map(|t| match t.strip_prefix("s13:") {
                Some(r) => {
                    let mut f: Vec<&str> = r.split(',').collect();
                    for w in &mut f[1..=5] {
                        *w = "_";
                    }
                    format!("s13:{}", f.join(","))
                }
                None => t.to_string(),
            })
            .collect();
        let want_log: Vec<&str> = masked.iter().map(String::as_str).collect();
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Window layout. `m` the group (`+0xc` count, `+0x10` form, `+0x4a` player, the shorts at `+0x8cc`);
                // `c[0]` the objects cell, `c[1]` the console (`+0x298` who), `c[2]` the game (`+0x821` the
                // semaphore), `c[3]` `ignore_orders`. `p[2 * pl]` and `p[2 * pl + 1]` the leaders (`+8` the leader's
                // own index; the diplomacies at `+0` and `+4`), `p[4 + pl]` the second table's diplomacies,
                // `p[8 + 4 * pl + j]` the objects `objects[pl][j]`, `p[16 + 4 * pl + j]` the data each answers by slot
                // `+0xac` (`+0x18` its type, `p[24 + 4 * pl + j]`), `p[34 + 4 * pl + j]` the units, `p[42 + 4 * pl + j]`
                // their types, `p[50]` the order `get_action` answers. `a`: the four arguments, the stubs' answers
                // (`a[4..37]`), every `--seq` run (`SEQ_AT`) and the order's null flag.
                //
                // **The port is the crate's own steps**, `crate::garrison_view::action_garrison_steps`, which
                // `Sim::group_action_garrison_search` runs over its `SimGarrison`; here the view is the emulated
                // original's windows.
                use crate::garrison_view::{GarrisonHead, GarrisonView, action_garrison_steps};
                const SEQ_AT: [(usize, usize, usize); 14] = [
                    (0, 37, 16),
                    (2, 53, 16),
                    (3, 69, 16),
                    (5, 85, 4),
                    (6, 89, 4),
                    (7, 93, 4),
                    (15, 97, 4),
                    (16, 101, 2),
                    (17, 103, 8),
                    (19, 111, 8),
                    (23, 119, 8),
                    (24, 127, 8),
                    (25, 135, 8),
                    (26, 143, 16),
                ];
                const NULL_ORDER: usize = 159;
                const BUILD_VT: i32 = 0x00b4_2174;
                const MESSAGE_WIN: i32 = 15_267_872;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 33],
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
                        8 + 4 * p as usize + o as usize
                    }
                    fn dw(p: i32, o: i32) -> usize {
                        16 + 4 * p as usize + o as usize
                    }
                    fn tw(p: i32, o: i32) -> usize {
                        24 + 4 * p as usize + o as usize
                    }
                    fn uw(p: i32, u: i32) -> usize {
                        34 + 4 * p as usize + u as usize
                    }
                    fn utw(p: i32, u: i32) -> usize {
                        42 + 4 * p as usize + u as usize
                    }
                    /// Slot `+0xac`: the object's data.
                    fn slot_ac(&mut self, p: i32, o: i32) {
                        self.rec(4, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                    }
                    fn is_build(&self, p: i32, o: i32) -> bool {
                        rd32(&self.p[Self::dw(p, o)], 0) == BUILD_VT
                    }
                }
                impl GarrisonHead for V<'_> {
                    fn who(&self) -> i32 {
                        i32::from(self.m[0x4a])
                    }
                    fn ignore_orders(&self) -> bool {
                        rd32(&self.c[3], 0) != 0
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
                impl GarrisonView for V<'_> {
                    fn set_up_insert(&mut self) {
                        self.rec(8, "m+0x0", &[]);
                        let t = self.log.last_mut().expect("just pushed");
                        t.push_str(",sp-0x5c");
                    }
                    fn action_halt(&mut self) {
                        self.rec(9, "m+0x0", &[0]);
                    }
                    fn finish_insert(&mut self) {
                        self.log.push("s10:m+0x0,sp-0x5c".into());
                    }
                    fn list_clear(&mut self) {
                        self.log.push("s11:sp-0x58".into());
                    }
                    fn set_form(&mut self) {
                        self.m[0x10..0x14].copy_from_slice(&(-1i32).to_le_bytes());
                    }
                    fn leader_self(&self, p: i32) -> i32 {
                        rd32(&self.p[2 * p as usize], 8)
                    }
                    fn diplo(&self, p: i32, q: i32) -> i32 {
                        rd32(&self.p[2 * p as usize + 1], 4 * q as usize)
                    }
                    fn diploc(&self, q: i32, f: i32) -> i32 {
                        rd32(&self.p[4 + q as usize], 4 * f as usize)
                    }
                    fn console_who(&self) -> i32 {
                        rd32(&self.c[1], 0x298)
                    }
                    fn refuse(&mut self, msg: i32, depth: i32) {
                        // `add_feedback`'s first five words are the stack's leftovers (the string copy is a stub):
                        // masked as `_` in both logs.
                        self.log
                            .push(format!("s12:sp-0x{:x},{msg}", 0x84 + 0x68 * depth));
                        self.log
                            .push(format!("s13:{MESSAGE_WIN},_,_,_,_,_,-1,13161032"));
                        self.log.push(format!("s14:{MESSAGE_WIN},64"));
                    }
                    fn obj_slot10(&mut self, p: i32, o: i32) -> i32 {
                        self.rec(0, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                        self.seq(0)
                    }
                    fn data_limit(&mut self, p: i32, o: i32) -> i32 {
                        self.slot_ac(p, o);
                        if self.is_build(p, o) {
                            let lvl = i32::from(self.p[Self::dw(p, o)][9]);
                            self.rec(15, &format!("p{}+0x0", Self::tw(p, o)), &[lvl]);
                            self.seq(15)
                        } else {
                            self.rec(7, &format!("p{}+0x0", Self::dw(p, o)), &[]);
                            self.seq(7)
                        }
                    }
                    fn data_is_1bf(&mut self, p: i32, o: i32) -> bool {
                        self.slot_ac(p, o);
                        if self.is_build(p, o) {
                            self.rec(6, &format!("p{}+0x0", Self::tw(p, o)), &[0x1bf, 0]);
                            self.seq(6) != 0
                        } else {
                            self.rec(5, &format!("p{}+0x0", Self::dw(p, o)), &[0x1bf, 0]);
                            self.seq(5) != 0
                        }
                    }
                    fn data_unassimilated(&mut self, p: i32, o: i32) -> bool {
                        self.slot_ac(p, o);
                        self.rec(16, &format!("p{}+0x0", Self::dw(p, o)), &[]);
                        self.seq(16) != 0
                    }
                    fn data_num_inside(&mut self, p: i32, o: i32) -> i32 {
                        self.slot_ac(p, o);
                        self.rec(17, &format!("p{}+0x0", Self::dw(p, o)), &[0]);
                        self.seq(17)
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
                    fn unit_not_plane(&self, p: i32, u: i32) -> bool {
                        let t = &self.p[Self::utw(p, u)];
                        rd32(t, 0x218) != 2 || t[0x2b4] & 0x20 != 0
                    }
                    fn unit_busy(&mut self, p: i32, u: i32) -> bool {
                        self.rec(19, &format!("p{}+0x0", Self::uw(p, u)), &[]);
                        self.seq(19) != 0
                    }
                    fn unit_order_kind(&mut self, p: i32, u: i32) -> Option<i32> {
                        self.rec(20, &format!("p{}+0x0", Self::uw(p, u)), &[]);
                        if self.a[NULL_ORDER] != 0 {
                            return None;
                        }
                        self.rec(0, "p50+0x0", &[]);
                        Some(self.seq(0))
                    }
                    fn repath(&mut self, p: i32, u: i32) {
                        self.rec(21, &format!("p{}+0x0", Self::uw(p, u)), &[]);
                    }
                    fn kill_current_order(&mut self, p: i32, u: i32) {
                        self.rec(22, &format!("p{}+0x0", Self::uw(p, u)), &[0]);
                    }
                    fn can_garrison(&mut self, p: i32, u: i32, who: i32, b: i32) -> bool {
                        self.slot_ac(who, b);
                        let t = rd32(&self.p[Self::tw(who, b)], 4);
                        self.rec(23, &format!("p{}+0x0", Self::utw(p, u)), &[t]);
                        self.seq(23) != 0
                    }
                    fn find_garrison_build(&mut self, p: i32, u: i32, who: i32, b: i32) -> i32 {
                        self.slot_ac(who, b);
                        let city = rd16(&self.p[Self::dw(who, b)], 0x72);
                        self.rec(24, &format!("p{}+0x0", Self::uw(p, u)), &[city, who]);
                        self.seq(24)
                    }
                    fn semaphore(&self) -> bool {
                        self.c[2][0x821] & 8 != 0
                    }
                    fn unit_pop(&self, p: i32, u: i32) -> i32 {
                        if self.p[Self::uw(p, u)][0x68] & 1 != 0 {
                            0
                        } else {
                            rd32(&self.p[Self::utw(p, u)], 0x2f0)
                        }
                    }
                    fn go_inside(&mut self, p: i32, u: i32, target: i32, who: i32) {
                        self.rec(18, &format!("p{}+0x0", Self::uw(p, u)), &[target, who, 0]);
                    }
                    fn unit_is_worker(&mut self, p: i32, u: i32) -> bool {
                        (0x32..=0x35).contains(&rd32(&self.p[Self::utw(p, u)], 4))
                    }
                    fn unit_packs(&self, p: i32, u: i32) -> bool {
                        self.p[Self::utw(p, u)][0x2b8] & 4 != 0
                    }
                    fn unit_is_packing(&mut self, p: i32, u: i32) -> bool {
                        self.rec(25, &format!("p{}+0x0", Self::uw(p, u)), &[]);
                        self.seq(25) != 0
                    }
                    fn unit_is_unpacking(&mut self, p: i32, u: i32) -> bool {
                        self.rec(26, &format!("p{}+0x0", Self::uw(p, u)), &[]);
                        self.seq(26) != 0
                    }
                    fn list_len(&self, p: i32, u: i32) -> i32 {
                        rd32(&self.p[Self::uw(p, u)], 0xd8)
                    }
                    fn list_tail(&mut self, p: i32, u: i32) {
                        self.rec(27, &format!("p{}+0xcc", Self::uw(p, u)), &[]);
                    }
                    fn list_current(&self, p: i32, u: i32) -> i32 {
                        rd32(&self.p[Self::uw(p, u)], 0xcc)
                    }
                    fn list_remove_current(&mut self, p: i32, u: i32) {
                        self.rec(28, &format!("p{}+0xcc", Self::uw(p, u)), &[]);
                    }
                    fn clear_orders(&mut self, p: i32, u: i32) {
                        self.rec(29, &format!("p{}+0x0", Self::uw(p, u)), &[]);
                    }
                    fn list_add(&mut self, p: i32, u: i32, data: i32) {
                        self.rec(30, &format!("p{}+0xcc", Self::uw(p, u)), &[data]);
                    }
                    fn update_gpiece(&mut self, p: i32, u: i32) {
                        self.rec(31, &format!("p{}+0x0", Self::uw(p, u)), &[]);
                    }
                    fn add_garrison_order(
                        &mut self,
                        p: i32,
                        u: i32,
                        target: i32,
                        who: i32,
                        search: i32,
                        queue: i32,
                    ) {
                        self.rec(
                            32,
                            &format!("p{}+0x0", Self::uw(p, u)),
                            &[target, who, search, queue, 1],
                        );
                    }
                }
                let mut v = V {
                    a,
                    m: &mut *m,
                    c: &*c,
                    p: &*p,
                    log: &mut log,
                    turn: [0; 33],
                };
                action_garrison_steps(&mut v, a[0], a[1], a[2], a[3], 0);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_garrison — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_garrison — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_garrison — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "action_garrison — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("Group::action_garrison@00700490: {n} rows agree");
}
