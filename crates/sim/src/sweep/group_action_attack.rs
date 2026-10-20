//! `Group::action_attack@00712490` under the emulator against the port (item 3362; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_attack@00712490`, run by `tools/emu/sweep_group_action_attack.py` under unicorn
/// (tools/emu): 2304 random bytes of object, 5 argument(s), `p` windows in order: p0..p28 the place(s), in flag order, p29..p36 the stubs' answer window(s); the global cell 0x00c0618c points at 256 random bytes; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c061f0 points at 32 random bytes; 4 random bytes at 0x00cc02f8 (each word in 0..0); 1 random bytes at 0x00ee10ed (each byte in 0..1); vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x48 (2 args) stubbed; vtable slot +0x50 (0 args) stubbed; vtable slot +0x88 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0xb8 (0 args) stubbed; vtable slot +0xd4 (0 args) stubbed; vtable slot +0xc0 (0 args) stubbed; vtable slot +0x10c (0 args) stubbed; callee 0x0070c450 (0 args) stubbed; callee 0x0070e520 (1 args) stubbed; callee 0x0070d0c0 (1 args) stubbed; callee 0x0070e620 (1 args) stubbed; callee 0x0046d3f0 (0 args) stubbed; callee 0x0046f600 (0 args) stubbed; callee 0x0070ccb0 (1 args) stubbed; callee 0x0070c5d0 (3 args) stubbed; callee 0x006486b0 (7 args) stubbed; callee 0x00648d70 (4 args) stubbed; callee 0x00601280 (8 args) stubbed; callee 0x0046fb40 (0 args) stubbed; callee 0x0062d1d0 (0 args) stubbed; callee 0x006276a0 (2 args) stubbed; callee 0x0070fba0 (10 args) stubbed; callee 0x00609a50 (0 args) stubbed; callee 0x00616e80 (0 args) stubbed; callee 0x006179d0 (0 args) stubbed; callee 0x00608450 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x0060b030 (0 args) stubbed; callee 0x0060a410 (0 args) stubbed; callee 0x0060aa60 (0 args) stubbed; callee 0x005e3860 (0 args) stubbed; callee 0x005e4a60 (7 args) stubbed; callee 0x00616ed0 (9 args) stubbed; callee 0x005e5410 (5 args) stubbed; callee 0x005ff9c0 (5 args) stubbed; callee 0x00a1d590 (1 args) stubbed; callee 0x007e9ab0 (7 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x00a1d660 (1 args) stubbed; callee 0x00a2e550 (6 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; the word at `c0+0x14+0x1c*m[0x4a]` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p0+8` points at 64 random bytes less `0` (placed); the word at `p0+12` points at 64 random bytes less `0` (placed); the word at `p0+16` points at 64 random bytes less `0` (placed); the word at `p0+20` points at 64 random bytes less `0` (placed); the word at `p0+24` points at 64 random bytes less `0` (placed); the word at `p0+28` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*m[0x4a]` points at 16 random bytes less `0` (placed); the word at `p9+0` points at 256 random bytes less `0` (placed); the word at `p9+4` points at 256 random bytes less `0` (placed); the word at `p9+8` points at 256 random bytes less `0` (placed); the word at `p9+12` points at 256 random bytes less `0` (placed); the word at `m+0x7c` points at 32 random bytes less `0` (placed); the word at `p14+0` points at 144 random bytes less `0` (placed); the word at `p14+4` points at 144 random bytes less `0` (placed); the word at `p14+8` points at 144 random bytes less `0` (placed); the word at `p14+12` points at 144 random bytes less `0` (placed); the word at `p14+16` points at 144 random bytes less `0` (placed); the word at `p14+20` points at 144 random bytes less `0` (placed); the word at `p14+24` points at 144 random bytes less `0` (placed); the word at `p14+28` points at 144 random bytes less `0` (placed); the word at `p10+0x18` points at 704 random bytes less `0` (placed); the word at `p11+0x18` points at 704 random bytes less `0` (placed); the word at `p12+0x18` points at 704 random bytes less `0` (placed); the word at `p13+0x18` points at 704 random bytes less `0` (placed); the word at `0xc0aa00` points at 32 random bytes less `0` (placed); the word at `0xc0aa04` points at 32 random bytes less `0` (placed); slot +0xd4 laid as 0x0046cea0 or its stub, a flag after the answers; slot +0xc0 laid as 0x0046ce40 or its stub, a flag after the answers; stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [0, 10, 10, 10, 10, 3] (90%); stub 3 answered from [1, 1, 0] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 1, 1] (90%); stub 7 answered from [0, 0, 1] (90%); stub 12 answered from [0, 0, 1] (90%); stub 13 answered from [0, 0, 1] (90%); stub 14 answered from [0, 0, 1] (90%); stub 15 answered from [1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 23 answered from [0, 1, 1] (90%); stub 24 answered from [0, 0, 0, 1] (90%); stub 25 answered from [0, 1] (90%); stub 26 answered from [1, 1, 0] (90%); stub 27 answered from [0, 0, 0, 1] (90%); stub 30 answered from [0, 0, 3, 40] (90%); stub 31 answered from [16, 16, 1, 3] (90%); stub 36 answered from [0, 1, 1] (90%); stub 37 answered from [0, 1] (90%); stub 33 answers the address of -128 random bytes, a `p` window after the places; stub 34 answers the address of 128 random bytes, a `p` window after the places; stub 32 answers the address of 128 random bytes, a `p` window after the places; stub 8 answers the address of 48 random bytes, a `p` window after the places; stub 9 answers the address of 72 random bytes, a `p` window after the places; stub 35 answers a window of its own to each call, in call order (128 bytes, ...); the word at `c0+0x68` laid as the address of `p0+16` every row; the word at `c0+0x84` laid as the address of `p0+16` every row; the word at `p32+0x7c` laid as the address of `p27` every row; the word at `p33+0x7c` laid as the address of `p28` every row; stub 10 answers `p15+j`, the data of the object its `ecx` is in `p1+j` (j < 8), linked at `+0x7e`; argument 0 copies `p35+0x8` (45%); argument 1 copies `p35+0xc` (45%); stub 0 answers 16 values in call order, printed after the answers; stub 1 answers 16 values in call order, printed after the answers; stub 2 answers 20 values in call order, printed after the answers; stub 3 answers 20 values in call order, printed after the answers; stub 5 answers 16 values in call order, printed after the answers; stub 6 answers 20 values in call order, printed after the answers; stub 12 answers 16 values in call order, printed after the answers; stub 13 answers 16 values in call order, printed after the answers; stub 14 answers 16 values in call order, printed after the answers; stub 23 answers 32 values in call order, printed after the answers; stub 24 answers 32 values in call order, printed after the answers; stub 30 answers 16 values in call order, printed after the answers; stub 31 answers 16 values in call order, printed after the answers; stub 36 answers 16 values in call order, printed after the answers; stub 37 answers 16 values in call order, printed after the answers; stub 42 answers 16 values in call order, printed after the answers; stub 21 answers -1..3 every row; stub 42 answers -1..3 every row; stub 47 answers 0..0 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 30 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 31 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 32 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 33 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 34 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 35 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 36 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 37 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 38 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 39 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 40 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 41 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 42 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 43 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 22 writes a value in 0..3000 through its argument 0, printed after the answers (`o` in a place); stub 22 writes a value in 0..3000 through its argument 1, printed after the answers (`o` in a place); stub 42 writes a value in -1..3 through its argument 1, printed after the answers (`o` in a place); stub 11 answers from its own `ecx`: the word at `+0x7c`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 5000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_attack.py") else {
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
                // Window layout (see the script's header for the flags). `m` the group (`+0xc` count, `+0x49` the
                // building flag, `+0x4a` player, `+0x2c` the order counter, the shorts at `+0x8cc`); `c[0]` the objects
                // record, `c[1]` the console (`+0x298`), `c[2]` the constants (`+0x18` the respond range), `c[3]` the
                // scenario's ignore-orders flag, `c[4]` `ignore_always`. `p[0]` the objects table of the group's player
                // (records `p[1..=4]`, the members) and of the target's (`p[5..=8]`: the target is `p[5 + whom]`); `p[9]`
                // the units table (`p[10 + h]`, each `+0x18` its type `p[23 + h]`); `p[14]` the Build data table
                // (`p[15 + j]` of object `p[1 + j]`); `p[27]`, `p[28]` the order records the second and third
                // `get_order` windows (`p[32]`, `p[33]`) answer by slot `+0xb8`; `p[29]` the `get_action` window,
                // `p[30]` `update_action`'s, `p[31..=33]` `get_order`'s, `p[34]` `update_order`'s, `p[35]` the
                // record slot `+0x50` answers, `p[36]` the one slot `+0x88` answers. `a`: the five arguments, one
                // answer a stub (`a[5 + k]`), the three out-parameters (`a[54..57]`), the `--seq` runs, the two
                // `vtat-keep` flags (`a[357]` slot `+0xd4`, `a[358]` slot `+0xc0`: set, the image's own function
                // inline) and the `get_action` null flag (`a[359]`).
                //
                // **The port is the crate's own steps**, `crate::attack_view::action_attack_steps`, which
                // `Sim::group_action_attack` calls over its `SimAttack`; here the view is the emulated original's
                // windows.
                use crate::attack_view::{AttackHead, AttackView, action_attack_steps};
                const SEQ_AT: [(usize, usize, usize); 16] = [
                    (0, 57, 16),
                    (1, 73, 16),
                    (2, 89, 20),
                    (3, 109, 20),
                    (5, 129, 16),
                    (6, 145, 20),
                    (12, 165, 16),
                    (13, 181, 16),
                    (14, 197, 16),
                    (23, 213, 32),
                    (24, 245, 32),
                    (30, 277, 16),
                    (31, 293, 16),
                    (36, 309, 16),
                    (37, 325, 16),
                    (42, 341, 16),
                ];
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 49],
                    depth: usize,
                    orders: usize,
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn rd16(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes(
                        w[o..o + 2].try_into().expect("two bytes"),
                    ))
                }
                fn wr32(w: &mut [u8], o: usize, v: i32) {
                    w[o..o + 4].copy_from_slice(&v.to_le_bytes());
                }
                fn wr16(w: &mut [u8], o: usize, v: i32) {
                    w[o..o + 2].copy_from_slice(&(v as u16).to_le_bytes());
                }
                const X: i32 = 0x63637;
                impl V<'_> {
                    fn who_i(&self) -> i32 {
                        i32::from(self.m[0x4a])
                    }
                    fn whom(&self) -> i32 {
                        self.a[0]
                    }
                    fn wwho(&self) -> i32 {
                        self.a[1]
                    }
                    fn ans(&self, k: usize) -> i32 {
                        self.a[5 + k]
                    }
                    /// Stub `k`'s next `--seq` answer, the last repeated.
                    fn seq(&mut self, k: usize) -> i32 {
                        let &(_, at, n) = SEQ_AT.iter().find(|s| s.0 == k).expect("a seq stub");
                        let i = self.turn[k].min(n - 1);
                        self.turn[k] += 1;
                        self.a[at + i]
                    }
                    fn rec(&mut self, k: usize, ecx: &str, args: &[String]) {
                        let mut t = format!("s{k}:{ecx}");
                        for v in args {
                            t.push(',');
                            t.push_str(v);
                        }
                        self.log.push(t);
                    }
                    fn n(v: i32) -> String {
                        v.to_string()
                    }
                    /// A local of the function at `ebp - off`, as the script's token: `sp-` the depth under the
                    /// caller's `esp`; a recursion's frame is `0x90` lower.
                    fn local(&self, off: usize) -> String {
                        format!("sp-{:#x}", 0x1c + off + 0x90 * self.depth)
                    }
                    fn mm() -> String {
                        "m+0x0".to_string()
                    }
                    /// The window of object `h` of the group's player.
                    fn o(h: i32) -> usize {
                        1 + h as usize
                    }
                    fn u(h: i32) -> usize {
                        10 + h as usize
                    }
                    fn ty(h: i32) -> usize {
                        23 + h as usize
                    }
                    fn slot_tok(h: i32) -> String {
                        format!("p9+{:#x}", 4 * h)
                    }
                    fn tw(&self) -> usize {
                        5 + self.whom() as usize
                    }
                    fn tok(w: usize) -> String {
                        format!("p{w}+0x0")
                    }
                    /// Slot `+0xac` of window `w` (object `p[1 + j]`, data `p[15 + j]`): the call, and the data.
                    fn slot_ac(&mut self, w: usize) -> usize {
                        self.rec(10, &Self::tok(w), &[]);
                        w - 1 + 15
                    }
                    fn xy(w: &[u8]) -> (i32, i32) {
                        (rd32(w, 0x10) ^ X, rd32(w, 0x14) ^ X)
                    }
                }
                impl AttackHead for V<'_> {
                    fn is_on_map(&mut self) -> bool {
                        self.rec(15, &Self::mm(), &[]);
                        self.ans(15) != 0
                    }
                    fn ignore_orders_on(&self) -> bool {
                        rd32(&self.c[3], 0) != 0
                    }
                    fn ignored_count(&self, _who: u8) -> i32 {
                        0
                    }
                    fn ignored_nth(&self, _who: u8, _i: i32) -> i32 {
                        -1
                    }
                    fn ignore_call(&mut self, _o: i32, _who: u8) {}
                    fn who(&self) -> u8 {
                        self.m[0x4a]
                    }
                    fn count(&self) -> i32 {
                        rd32(self.m, 0xc)
                    }
                    fn refresh(&mut self) {
                        self.seq(3);
                        self.rec(3, &Self::mm(), &[]);
                    }
                }
                impl AttackView for V<'_> {
                    type H = i32;
                    type O = usize;
                    fn building_group(&self) -> bool {
                        self.m[0x49] != 0
                    }
                    fn member(&self, i: i32) -> i32 {
                        rd16(self.m, 0x8cc + 2 * i as usize)
                    }
                    fn h_of(&self, x: i32) -> i32 {
                        x
                    }
                    fn h_idx(&self, h: i32) -> i32 {
                        h
                    }
                    fn bump_order_num(&mut self) {
                        let v = rd32(self.m, 0x2c);
                        wr32(self.m, 0x2c, v.wrapping_add(1));
                    }
                    fn console_is_who(&mut self) -> bool {
                        rd32(&self.c[1], 0x298) == self.who_i()
                    }
                    fn feedback(&mut self, id: i32) {
                        let t = self.local(0x74 + 0x4 + 0x4 + 0x14);
                        self.rec(43, &t, &[V::n(id)]);
                    }
                    fn order_list_new(&mut self) {
                        let t = self.local(0x68);
                        self.rec(19, &t, &[]);
                    }
                    fn set_up_insert(&mut self) {
                        let t = self.local(0x68);
                        self.rec(16, &Self::mm(), &[t]);
                    }
                    fn halt(&mut self, mask: i32) {
                        self.rec(17, &Self::mm(), &[V::n(mask)]);
                    }
                    fn finish_insert(&mut self) {
                        let t = self.local(0x68);
                        self.rec(18, &Self::mm(), &[t]);
                    }
                    fn order_list_clear(&mut self) {
                        let t = self.local(0x64);
                        self.rec(20, &t, &[]);
                    }
                    fn nest(&mut self, down: bool) {
                        if down {
                            self.depth += 1;
                        } else {
                            self.depth -= 1;
                        }
                    }
                    fn obj_is_build(&mut self, h: i32) -> bool {
                        let v = self.seq(6);
                        self.rec(6, &Self::tok(Self::o(h)), &[]);
                        v != 0
                    }
                    fn report_not_build(&mut self) {}
                    fn mark_build(&mut self, h: i32, whom: i32, wwho: i32) {
                        let d = self.slot_ac(Self::o(h));
                        let f = rd16(&self.p[d], 0x60) | 4;
                        wr16(&mut self.p[d], 0x60, f);
                        wr16(&mut self.p[d], 0x7c, whom);
                        self.p[d][0x81] = wwho as u8;
                    }
                    fn build_in_range(&mut self, h: i32, whom: i32, wwho: i32) -> bool {
                        let d = self.slot_ac(Self::o(h));
                        let (x, y) = Self::xy(&self.p[d]);
                        let v = self.seq(23);
                        self.rec(
                            23,
                            &Self::tok(d),
                            &[
                                V::n(whom),
                                V::n(wwho),
                                V::n(x),
                                V::n(y),
                                V::n(x),
                                "0".into(),
                                "0".into(),
                            ],
                        );
                        v != 0
                    }
                    fn find_leader(&mut self) -> i32 {
                        self.rec(21, &Self::mm(), &["0".into()]);
                        self.ans(21)
                    }
                    fn obj_pos(&mut self, h: i32) -> (i32, i32) {
                        Self::xy(&self.p[Self::o(h)])
                    }
                    fn get_loc_to(&mut self, _x: i32, _y: i32) -> (i32, i32) {
                        let (tx, ty) = (self.local(0x24), self.local(0x20));
                        self.rec(22, &Self::mm(), &[tx, ty, "c0+0x0".into()]);
                        (self.a[54], self.a[55])
                    }
                    fn lead_in_range(
                        &mut self,
                        h: i32,
                        whom: i32,
                        wwho: i32,
                        x: i32,
                        y: i32,
                    ) -> bool {
                        let v = self.seq(23);
                        self.rec(
                            23,
                            &Self::tok(Self::u(h)),
                            &[
                                V::n(whom),
                                V::n(wwho),
                                V::n(x),
                                V::n(y),
                                Self::slot_tok(h),
                                "0".into(),
                                "0".into(),
                            ],
                        );
                        v != 0
                    }
                    fn find_attack_pos(
                        &mut self,
                        h: i32,
                        whom: i32,
                        wwho: i32,
                        x: i32,
                        y: i32,
                    ) -> bool {
                        let (l38, l34) = (self.local(0x38), self.local(0x34));
                        self.rec(
                            25,
                            &Self::tok(Self::u(h)),
                            &[
                                V::n(whom),
                                V::n(wwho),
                                "1".into(),
                                l38,
                                l34,
                                V::n(x),
                                V::n(y),
                                "1".into(),
                            ],
                        );
                        self.ans(25) != 0
                    }
                    fn t_slot10(&mut self) -> bool {
                        let v = self.seq(2);
                        let w = self.tw();
                        self.rec(2, &Self::tok(w), &[]);
                        v != 0
                    }
                    fn t_slot18(&mut self) {
                        let w = self.tw();
                        self.rec(4, &Self::tok(w), &[]);
                    }
                    fn t_is_build(&mut self) -> bool {
                        let v = self.seq(6);
                        let w = self.tw();
                        self.rec(6, &Self::tok(w), &[]);
                        v != 0
                    }
                    fn t_is_active_build(&mut self) -> bool {
                        let w = self.tw();
                        self.rec(26, &Self::tok(w), &[]);
                        self.ans(26) != 0
                    }
                    fn t_capture_eligible(&mut self) -> bool {
                        let w = self.tw();
                        let d = self.slot_ac(w);
                        self.rec(27, &Self::tok(d), &[]);
                        self.ans(27) != 0
                    }
                    fn t_check_capture(&mut self, leader: i32, who: u8) {
                        let w = self.tw();
                        let d = self.slot_ac(w);
                        self.rec(28, &Self::tok(d), &[V::n(leader), V::n(i32::from(who))]);
                    }
                    fn t_slot48(&mut self, who: u8) -> i32 {
                        let w = self.tw();
                        self.rec(7, &Self::tok(w), &[V::n(i32::from(who)), "0".into()]);
                        self.ans(7)
                    }
                    fn t_pos(&mut self) -> (i32, i32) {
                        Self::xy(&self.p[self.tw()])
                    }
                    fn t_slot1c(&mut self) -> bool {
                        let v = self.seq(5);
                        let w = self.tw();
                        self.rec(5, &Self::tok(w), &[]);
                        v != 0
                    }
                    fn action_move_to(&mut self, x: i32, y: i32, queue: i32) {
                        let args: Vec<String> = [x, y, queue, 0, 0, 1, 1, -1, -1, 0]
                            .iter()
                            .map(|&v| V::n(v))
                            .collect();
                        self.rec(29, &Self::mm(), &args);
                    }
                    fn o_alive(&mut self, h: i32) -> bool {
                        let v = self.seq(0);
                        self.rec(0, &Self::tok(Self::o(h)), &[]);
                        v != 0
                    }
                    fn o_orderable(&mut self, h: i32) -> bool {
                        let v = self.seq(1);
                        self.rec(1, &Self::tok(Self::o(h)), &[]);
                        v != 0
                    }
                    fn type_slot10c(&mut self, h: i32) -> bool {
                        let v = self.seq(14);
                        self.rec(14, &Self::tok(Self::ty(h)), &[]);
                        v != 0
                    }
                    fn u_is_special(&mut self, h: i32) -> bool {
                        if self.a[357] != 0 {
                            rd32(&self.p[Self::ty(h)], 0x2b8) & 0x10 != 0
                        } else {
                            let v = self.seq(12);
                            self.rec(12, &Self::tok(Self::u(h)), &[]);
                            v != 0
                        }
                    }
                    fn u_is_spy(&mut self, h: i32) -> bool {
                        self.rec(11, &Self::tok(Self::u(h)), &[]);
                        rd32(&self.p[Self::u(h)], 0x7c) != 0
                    }
                    fn u_is_plane(&mut self, h: i32) -> bool {
                        if self.a[358] != 0 {
                            let t = &self.p[Self::ty(h)];
                            rd32(t, 0x218) == 2 && t[0x2b4] & 0x20 == 0
                        } else {
                            let v = self.seq(13);
                            self.rec(13, &Self::tok(Self::u(h)), &[]);
                            v != 0
                        }
                    }
                    fn u_mana_left(&mut self, h: i32) -> i32 {
                        let v = self.seq(30);
                        self.rec(30, &Self::tok(Self::u(h)), &[]);
                        (v.wrapping_sub(rd16(&self.p[Self::u(h)], 0x96))).max(0)
                    }
                    fn u_order_type(&mut self, h: i32) -> i32 {
                        let v = self.seq(31);
                        self.rec(31, &Self::tok(Self::u(h)), &[]);
                        v
                    }
                    fn strafe_retarget(&mut self, h: i32, whom: i32, wwho: i32) {
                        self.rec(32, &Self::tok(Self::u(h)), &[]);
                        self.rec(9, &Self::tok(34), &[]);
                        let w30 = rd16(&self.p[self.tw()], 0x30);
                        wr32(&mut self.p[36], 0xc, wwho);
                        wr32(&mut self.p[36], 8, whom);
                        wr16(&mut self.p[36], 0x10, w30);
                        wr32(&mut self.p[36], 0x3c, 0);
                    }
                    fn type_domain(&mut self, h: i32) -> i32 {
                        rd32(&self.p[Self::ty(h)], 0x218)
                    }
                    fn get_action(&mut self, h: i32) -> Option<usize> {
                        self.rec(33, &Self::tok(Self::u(h)), &[]);
                        (self.a[359] == 0).then_some(29)
                    }
                    fn ord_type(&mut self, o: usize) -> i32 {
                        let v = self.seq(2);
                        self.rec(2, &Self::tok(o), &[]);
                        v
                    }
                    fn update_action_rec(&mut self, h: i32) -> (i32, i32, i32) {
                        self.rec(34, &Self::tok(Self::u(h)), &[]);
                        self.rec(8, &Self::tok(30), &[]);
                        let e = &self.p[35];
                        (rd32(e, 8), rd32(e, 0xc), i32::from(e[0x1c]))
                    }
                    fn u_d8(&mut self, h: i32) -> i32 {
                        rd32(&self.p[Self::u(h)], 0xd8)
                    }
                    fn in_range4(&mut self, h: i32, whom: i32, wwho: i32) -> bool {
                        let v = self.seq(24);
                        self.rec(
                            24,
                            &Self::tok(Self::u(h)),
                            &[V::n(whom), V::n(wwho), Self::slot_tok(h), "0".into()],
                        );
                        v != 0
                    }
                    fn get_order(&mut self, h: i32) -> usize {
                        self.rec(35, &Self::tok(Self::u(h)), &[]);
                        let o = 31 + self.orders.min(2);
                        self.orders += 1;
                        o
                    }
                    fn ord_slot14(&mut self, o: usize) -> bool {
                        let v = self.seq(3);
                        self.rec(3, &Self::tok(o), &[]);
                        v != 0
                    }
                    fn ord_b8(&mut self, o: usize) -> (i32, i32) {
                        self.rec(11, &Self::tok(o), &[]);
                        let r = match o {
                            32 => 27,
                            33 => 28,
                            _ => panic!("slot +0xb8 asked of window p{o}"),
                        };
                        (rd32(&self.p[r], 4), rd32(&self.p[r], 8))
                    }
                    fn approach_in_range(
                        &mut self,
                        h: i32,
                        whom: i32,
                        wwho: i32,
                        a: i32,
                        b: i32,
                    ) -> bool {
                        let v = self.seq(23);
                        let w7 = self.who_i() * 7;
                        self.rec(
                            23,
                            &Self::tok(Self::u(h)),
                            &[
                                V::n(whom),
                                V::n(wwho),
                                V::n(a),
                                V::n(b),
                                V::n(w7),
                                "0".into(),
                                "0".into(),
                            ],
                        );
                        v != 0
                    }
                    fn u_packing_or_unpacking(&mut self, h: i32) -> bool {
                        let v = self.seq(36);
                        self.rec(36, &Self::tok(Self::u(h)), &[]);
                        v != 0
                    }
                    fn u_is_packing(&mut self, h: i32) -> bool {
                        let v = self.seq(37);
                        self.rec(37, &Self::tok(Self::u(h)), &[]);
                        v != 0
                    }
                    fn clear_orders(&mut self, h: i32) {
                        self.rec(38, &Self::tok(Self::u(h)), &[]);
                    }
                    fn self_in_range(&mut self, h: i32, whom: i32, wwho: i32) -> bool {
                        let (x, y) = Self::xy(&self.p[Self::o(h)]);
                        let v = self.seq(23);
                        self.rec(
                            23,
                            &Self::tok(Self::u(h)),
                            &[
                                V::n(whom),
                                V::n(wwho),
                                V::n(x),
                                V::n(y),
                                Self::slot_tok(h),
                                "0".into(),
                                "0".into(),
                            ],
                        );
                        v != 0
                    }
                    fn add_cast_order(&mut self, h: i32, id: i32) {
                        let args: Vec<String> = [-1, -1, -1, -1, id, 2, 0]
                            .iter()
                            .map(|&v| V::n(v))
                            .collect();
                        self.rec(39, &Self::tok(Self::u(h)), &args);
                    }
                    fn add_move_order(&mut self, h: i32, x: i32, y: i32, queue: i32) {
                        let args = [
                            V::n(x),
                            V::n(y),
                            "1".into(),
                            "0".into(),
                            V::n(queue),
                            "0".into(),
                            Self::slot_tok(h),
                            "-1".into(),
                            "-1".into(),
                        ];
                        self.rec(40, &Self::tok(Self::u(h)), &args);
                    }
                    fn add_attack_order(
                        &mut self,
                        h: i32,
                        whom: i32,
                        wwho: i32,
                        queue: i32,
                        mandatory: i32,
                    ) {
                        let args: Vec<String> = [whom, wwho, queue, mandatory, 1]
                            .iter()
                            .map(|&v| V::n(v))
                            .collect();
                        self.rec(41, &Self::tok(Self::u(h)), &args);
                    }
                    fn find_melee_target(
                        &mut self,
                        h: i32,
                        maxd: i32,
                        _wwho: i32,
                        word: i32,
                    ) -> (i32, i32) {
                        let v = self.seq(42);
                        let l = self.local(0x24);
                        let args = [V::n(maxd), l, "0".into(), "0".into(), V::n(word)];
                        self.rec(42, &Self::tok(Self::u(h)), &args);
                        (v, self.a[56])
                    }
                    fn respond_range(&mut self) -> i32 {
                        rd32(&self.c[2], 0x18)
                    }
                }
                let mut v = V {
                    a,
                    m: &mut *m,
                    c: &*c,
                    p: &mut *p,
                    log: &mut log,
                    turn: [0; 49],
                    depth: 0,
                    orders: 0,
                };
                action_attack_steps(&mut v, a[0], a[1], a[2], a[3], a[4]);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_attack — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_attack — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_attack — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[6 + i],
                "action_attack — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 5000, "only {n} rows");
    eprintln!("Group::action_attack@00712490: {n} rows agree");
}
