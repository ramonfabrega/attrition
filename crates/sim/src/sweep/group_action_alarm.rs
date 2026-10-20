//! `Group::action_alarm@0070ec30` under the emulator against the port (item 3364; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_alarm@0070ec30`, run by `tools/emu/sweep_group_action_alarm.py` under unicorn
/// (tools/emu): 2272 random bytes of object, 0 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p46 the place(s), in flag order, p47 stub 28's answer window; the global cell 0x00c0618c points at 768 random bytes; the global cell 0x00c06188 points at 512 random bytes; the global cell 0x00c06210 points at 768 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061fc points at 64 random bytes; 4 random bytes at 0x00cc02f8 (each word in 0..0); 224 random bytes at 0x00ed6574 (each word in 0..3); 1 random bytes at 0x00ee10eb (each byte in 0..1); callee 0x006fd980 (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x178 (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x68 (1 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00a1d660 (1 args) stubbed; callee 0x00a2e550 (6 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x00738410 (0 args) stubbed; callee 0x00646d50 (1 args) stubbed; callee 0x00633f50 (1 args) stubbed; callee 0x006d9380 (1 args) stubbed; callee 0x0061d7d0 (1 args) stubbed; callee 0x0065b870 (5 args) stubbed; callee 0x004613d0 (4 args) stubbed; callee 0x0070f8f0 (3 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x00700490 (4 args) stubbed; callee 0x00608450 (0 args) stubbed; callee 0x005e29b0 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x006704a0 (2 args) stubbed; callee 0x0064cd20 (4 args) stubbed; the word at `c0+0x14+0x1c*0` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p2+0` points at 160 random bytes less `0` (placed); the word at `p2+4` points at 160 random bytes less `0` (placed); the word at `p2+8` points at 160 random bytes less `0` (placed); the word at `p2+12` points at 160 random bytes less `0` (placed); the word at `p3+0` points at 160 random bytes less `0` (placed); the word at `p3+4` points at 160 random bytes less `0` (placed); the word at `p3+8` points at 160 random bytes less `0` (placed); the word at `p3+12` points at 160 random bytes less `0` (placed); the word at `p4+0x7c` points at 128 random bytes less `0` (placed); the word at `p5+0x7c` points at 128 random bytes less `0` (placed); the word at `p6+0x7c` points at 128 random bytes less `0` (placed); the word at `p7+0x7c` points at 128 random bytes less `0` (placed); the word at `p12+0x18` points at 64 random bytes less `0` (placed); the word at `p13+0x18` points at 64 random bytes less `0` (placed); the word at `p14+0x18` points at 64 random bytes less `0` (placed); the word at `p15+0x18` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p20+0` points at 160 random bytes less `0` (placed); the word at `p20+4` points at 160 random bytes less `0` (placed); the word at `p20+8` points at 160 random bytes less `0` (placed); the word at `p20+12` points at 160 random bytes less `0` (placed); the word at `p21+0` points at 160 random bytes less `0` (placed); the word at `p21+4` points at 160 random bytes less `0` (placed); the word at `p21+8` points at 160 random bytes less `0` (placed); the word at `p21+12` points at 160 random bytes less `0` (placed); the word at `p22+0x18` points at 760 random bytes less `0` (placed); the word at `p23+0x18` points at 760 random bytes less `0` (placed); the word at `p24+0x18` points at 760 random bytes less `0` (placed); the word at `p25+0x18` points at 760 random bytes less `0` (placed); the word at `p26+0x18` points at 760 random bytes less `0` (placed); the word at `p27+0x18` points at 760 random bytes less `0` (placed); the word at `p28+0x18` points at 760 random bytes less `0` (placed); the word at `p29+0x18` points at 760 random bytes less `0` (placed); the word at `0xc09970+0x1c*m[0x4a]` points at 32 random bytes less `0` (placed); the word at `p38+0` points at 32 random bytes less `0` (placed); the word at `p38+4` points at 32 random bytes less `0` (placed); the word at `p38+8` points at 32 random bytes less `0` (placed); the word at `p38+12` points at 32 random bytes less `0` (placed); the word at `c1+0x134` points at 4032 random bytes less `0` (placed); the word at `c4+0x10` points at 1024 random bytes less `0` (placed); the word at `0xe85ddc` points at 256 random bytes less `0` (placed); the word at `p45+0xc8` points at 64 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 2 answered from [1, 1, 1, 0] (90%); stub 3 answered from [1, 1, 1, 0] (90%); stub 4 answered from [26, 26, 1, 0, 5] (90%); stub 5 answered from [1, 1, 0] (90%); stub 7 answered from [0, 1] (90%); stub 8 answered from [0, 1, 1] (90%); stub 9 answered from [0, 1, 1] (90%); stub 11 answered from [0, 1, 1] (90%); stub 21 answered from [0, 1] (90%); stub 22 answered from [0, 1, 1] (90%); stub 23 answered from [0, 1, 2, 3] (90%); stub 25 answered from [0, 0, 0, 1] (90%); stub 28 answers the address of -32 random bytes, a `p` window after the places; the word at `p12+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p13+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p14+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p15+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p4+0x18` laid as the address of `p16` every row; the word at `p5+0x18` laid as the address of `p17` every row; the word at `p6+0x18` laid as the address of `p18` every row; the word at `p7+0x18` laid as the address of `p19` every row; stub 6 answers `p12+j`, the data of the object its `ecx` is in `p4+j` (j < 4), linked at `+0x74`; stub 2 answers 48 values in call order, printed after the answers; stub 3 answers 48 values in call order, printed after the answers; stub 4 answers 48 values in call order, printed after the answers; stub 5 answers 24 values in call order, printed after the answers; stub 7 answers 16 values in call order, printed after the answers; stub 8 answers 16 values in call order, printed after the answers; stub 9 answers 16 values in call order, printed after the answers; stub 10 answers 16 values in call order, printed after the answers; stub 11 answers 16 values in call order, printed after the answers; stub 12 answers 2 values in call order, printed after the answers; stub 18 answers 8 values in call order, printed after the answers; stub 19 answers 24 values in call order, printed after the answers; stub 20 answers 24 values in call order, printed after the answers; stub 21 answers 2 values in call order, printed after the answers; stub 22 answers 24 values in call order, printed after the answers; stub 23 answers 64 values in call order, printed after the answers; stub 25 answers 16 values in call order, printed after the answers; stub 10 answers 0..6 every row; stub 12 answers 0..40 every row; stub 15 answers 0..0 every row; stub 18 answers 6..16 every row; stub 19 answers 0..2 every row; stub 20 answers 3..8 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 30 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 31 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 32 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 24 writes a value in [0, 1, 2, 3] through its argument 2, printed after the answers (`o` in a place); stub 24 writes a value in [0, 0, 0, 1] through its argument 3, printed after the answers (`o` in a place); the stack 2816 bytes under the call is zeroed before each row; `0x006817f0` run once before the rows (a table builder the startup would have run); stub 13 writes 0 (4 bytes) at `ecx+0xc` each call; stub 26 writes inc (4 bytes) at `ecx+0xc` each call; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x20 (`p` windows after the pointed buffers, before the places); figures p4..p7 link at +0x2c (2 bytes) above their own index or to -1; figures p8..p11 link at +0x2c (2 bytes) above their own index or to -1; figures p22..p25 link at +0x92 (2 bytes) above their own index or to -1; figures p22..p25 link at +0x28 (2 bytes) above their own index or to -1; figures p26..p29 link at +0x92 (2 bytes) above their own index or to -1; figures p26..p29 link at +0x28 (2 bytes) above their own index or to -1; figures p4..p7 hold a short at +0x2e drawn from [0, 1], after the links; figures p8..p11 hold a short at +0x2e drawn from [0, 1], after the links; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_alarm.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        // `Error::report`'s third and fourth arguments are two uninitialised stack slots (`sub esp, 8` in the
        // listing, `in_stack_fffff584/588` in the decompile): the stack's leftovers of earlier calls, which no
        // reading of the function depends on. Both logs hold `_` for them.
        let masked: Vec<String> = calls
            .split_whitespace()
            .map(|t| match t.strip_prefix("s15:") {
                Some(r) => {
                    let mut f: Vec<&str> = r.split(',').collect();
                    f[3] = "_";
                    f[4] = "_";
                    format!("s15:{}", f.join(","))
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
        let mut p: Vec<Vec<u8>> = wins.split_off(8);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Window layout. `m` the group (`+0xc` count, `+0x49` armed, `+0x4a` player, the shorts at
                // `+0x8cc`); `c[0]` the objects cell, `c[1]` the world (`+0`/`+4` extent), `c[2]` the console (`+0x298`
                // who), `c[3]` the `div_3_table`, `c[4]` the unit types, `c[5]` `ignore_orders`, `c[6]` the
                // ignore-orders counts, `c[7]` `ignore_always`. `p[0..2]` the leaders, `p[4 + 4 * pl + j]` the objects
                // `objects[pl][j]`, `p[12 + j]` the data each of player 0's answers by slot `+0xac` (`+0x18` its type
                // `p[16 + j]`), `p[22 + 4 * pl + j]` the units, `p[30 + 4 * pl + j]` their types, `p[39 + j]` the
                // cities, `p[43]` the world's cells (28 bytes each), `p[44]` the unit-type array, `p[46]` the
                // tribe's type, `p[47]` the order `get_action` answers. `a`: the stubs' answers (`a[0..33]`), the
                // two `get_down` out-values, then every `--seq` run (`SEQ_AT`), the `+0xb8` flag and the order's
                // null flag.
                //
                // **The port is the crate's own steps**, `crate::alarm_view::action_alarm_steps`, which
                // `Sim::action_alarm` calls over its `SimAlarm`; here the view is the emulated original's windows.
                use crate::alarm_view::{AlarmHead, AlarmView, action_alarm_steps};
                const SEQ_AT: [(usize, usize, usize); 17] = [
                    (2, 35, 48),
                    (3, 83, 48),
                    (4, 131, 48),
                    (5, 179, 24),
                    (7, 203, 16),
                    (8, 219, 16),
                    (9, 235, 16),
                    (10, 251, 16),
                    (11, 267, 16),
                    (12, 283, 2),
                    (18, 285, 8),
                    (19, 293, 24),
                    (20, 317, 24),
                    (21, 341, 2),
                    (22, 343, 24),
                    (23, 367, 64),
                    (25, 431, 16),
                ];
                const BASED: usize = 447;
                const NULL_ORDER: usize = 448;
                const BUILD_VT: i32 = 0x00b4_2174;
                const DIV3: usize = 3;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 33],
                    count: i32,
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
                    fn rec_raw(&mut self, t: String) {
                        self.log.push(t);
                    }
                    fn ow(p: i32, o: i32) -> usize {
                        4 + 4 * p as usize + o as usize
                    }
                    fn uw(p: i32, u: i32) -> usize {
                        22 + 4 * p as usize + u as usize
                    }
                    fn dw(o: i32) -> usize {
                        12 + o as usize
                    }
                    fn tw(o: i32) -> usize {
                        16 + o as usize
                    }
                    fn utw(p: i32, u: i32) -> usize {
                        30 + 4 * p as usize + u as usize
                    }
                    fn cw(c: i32) -> usize {
                        39 + c as usize
                    }
                    /// Slot `+0xac`: the object's data.
                    fn slot_ac(&mut self, p: i32, o: i32) {
                        self.rec(6, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                    }
                    fn is_build(&self, o: i32) -> bool {
                        rd32(&self.p[Self::dw(o)], 0) == BUILD_VT
                    }
                    fn div3(&self, i: i32) -> i32 {
                        rd32(&self.c[DIV3], 4 * i as usize)
                    }
                    fn cell_of(&self, rec: &[u8], sh: u32) -> (i32, i32) {
                        (
                            self.div3(rd32(rec, 0xc) >> sh),
                            self.div3(rd32(rec, 0x10) >> sh),
                        )
                    }
                }
                impl AlarmHead for V<'_> {
                    fn who(&self) -> i32 {
                        i32::from(self.m[0x4a])
                    }
                    fn group_clear(&mut self) {
                        self.rec_raw("s13:sp-0xa60,-1".into());
                        self.count = 0;
                    }
                    fn ignore_orders(&self) -> bool {
                        rd32(&self.c[5], 0) != 0
                    }
                    fn ignoring_count(&self, who: i32) -> i32 {
                        rd32(&self.c[6], 0x1c * who as usize)
                    }
                    fn ignoring_at(&self, _who: i32, _i: i32) -> i32 {
                        unreachable!("the whole-function sweep keeps ignore_orders at 0")
                    }
                    fn drop_ignored(&mut self, _o: i32, _who: i32) {
                        unreachable!("the whole-function sweep keeps ignore_orders at 0")
                    }
                    fn armed(&self) -> bool {
                        self.m[0x49] != 0
                    }
                    fn peasant_alarm(&mut self) {
                        self.rec(0, "m+0x0", &[]);
                    }
                    fn refresh(&mut self) {
                        self.rec(1, "m+0x0", &[]);
                    }
                }
                impl AlarmView for V<'_> {
                    fn mark_economy(&mut self, who: i32) {
                        let w = &mut self.p[who as usize];
                        let v = rd32(w, 0) | 0x0200_0000;
                        w[0..4].copy_from_slice(&v.to_le_bytes());
                    }
                    fn count(&self) -> i32 {
                        rd32(self.m, 0xc)
                    }
                    fn member(&self, i: i32) -> i32 {
                        rd16(self.m, 0x8cc + 2 * i as usize)
                    }
                    fn obj_slot10(&mut self, p: i32, o: i32) -> i32 {
                        self.rec(4, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                        self.seq(4)
                    }
                    fn ignore_always(&self) -> bool {
                        self.c[7][0] != 0
                    }
                    fn report_pancake(&mut self) -> bool {
                        // The two argument slots `sub esp, 8` leaves are uninitialised: masked as `_` in both logs.
                        self.rec(14, "sp-0x84", &[11_409_864]);
                        self.rec_raw("s15:13161384,sp-0x84,sp-0x84,_,_,c7+0x0,1".into());
                        self.rec_raw("s16:sp-0x84".into());
                        self.rec_raw("s16:sp-0x60".into());
                        false
                    }
                    fn obj_flags8(&self, p: i32, o: i32) -> i32 {
                        i32::from(self.p[Self::ow(p, o)][8])
                    }
                    fn data_city(&mut self, p: i32, o: i32) -> i32 {
                        self.slot_ac(p, o);
                        rd16(&self.p[Self::dw(o)], 0x72)
                    }
                    fn data_finished(&mut self, p: i32, o: i32) -> bool {
                        self.slot_ac(p, o);
                        if self.is_build(o) {
                            self.p[Self::dw(o)][8] & 4 != 0
                        } else {
                            self.rec(9, &format!("p{}+0x0", Self::dw(o)), &[]);
                            self.seq(9) != 0
                        }
                    }
                    fn data_first_inside(&mut self, p: i32, o: i32) -> i32 {
                        self.slot_ac(p, o);
                        rd16(&self.p[Self::dw(o)], 0x28)
                    }
                    fn data_inside_player(&mut self, p: i32, o: i32) -> i32 {
                        self.slot_ac(p, o);
                        i32::from(self.p[Self::dw(o)][0x3e] as i8)
                    }
                    fn data_next_building(&mut self, p: i32, o: i32) -> i32 {
                        self.slot_ac(p, o);
                        rd16(&self.p[Self::dw(o)], 0x74)
                    }
                    fn data_first_unit(&mut self, p: i32, o: i32) -> i32 {
                        self.slot_ac(p, o);
                        rd16(&self.p[Self::dw(o)], 0x70)
                    }
                    fn data_num_inside(&mut self, p: i32, o: i32) -> i32 {
                        self.slot_ac(p, o);
                        self.rec(19, &format!("p{}+0x0", Self::dw(o)), &[0]);
                        self.seq(19)
                    }
                    fn data_limit(&mut self, p: i32, o: i32) -> i32 {
                        self.slot_ac(p, o);
                        if self.is_build(o) {
                            let lvl = i32::from(self.p[Self::dw(o)][9]);
                            self.rec(20, &format!("p{}+0x0", Self::tw(o)), &[lvl]);
                            self.seq(20)
                        } else {
                            self.rec(10, &format!("p{}+0x0", Self::dw(o)), &[]);
                            self.seq(10)
                        }
                    }
                    fn data_type_slot90(&mut self, p: i32, o: i32) -> bool {
                        self.slot_ac(p, o);
                        self.rec(11, &format!("p{}+0x0", Self::tw(o)), &[]);
                        self.seq(11) != 0
                    }
                    fn eject_peasants(&mut self, p: i32, o: i32) {
                        self.slot_ac(p, o);
                        self.rec(32, &format!("p{}+0x0", Self::dw(o)), &[0, 50, 0, 0]);
                    }
                    fn obj_alive(&mut self, p: i32, o: i32) -> bool {
                        self.rec(2, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                        self.seq(2) != 0
                    }
                    fn obj_bc(&mut self, p: i32, o: i32) -> bool {
                        self.rec(3, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                        self.seq(3) != 0
                    }
                    fn obj_slot1c(&mut self, p: i32, o: i32) -> bool {
                        self.rec(5, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                        self.seq(5) != 0
                    }
                    fn obj_type(&self, _p: i32, o: i32) -> i32 {
                        rd32(&self.p[Self::tw(o)], 4)
                    }
                    fn obj_tile(&self, p: i32, o: i32) -> (i32, i32) {
                        let w = &self.p[Self::ow(p, o)];
                        (
                            self.div3((rd32(w, 0x10) ^ 0x63637) >> 6),
                            self.div3((rd32(w, 0x14) ^ 0x63637) >> 6),
                        )
                    }
                    fn obj_chain(&self, p: i32, o: i32) -> (i32, i32) {
                        let w = &self.p[Self::ow(p, o)];
                        (rd16(w, 0x2c), rd16(w, 0x2e))
                    }
                    fn unit_type(&self, p: i32, u: i32) -> i32 {
                        rd32(&self.p[Self::utw(p, u)], 4)
                    }
                    fn unit_pop(&self, p: i32, u: i32) -> i32 {
                        if self.p[Self::uw(p, u)][0x68] & 1 != 0 {
                            0
                        } else {
                            rd32(&self.p[Self::utw(p, u)], 0x2f0)
                        }
                    }
                    fn unit_next_inside(&self, p: i32, u: i32) -> i32 {
                        rd16(&self.p[Self::uw(p, u)], 0x92)
                    }
                    fn unit_next_civ(&self, p: i32, u: i32) -> (i32, i32) {
                        let w = &self.p[Self::uw(p, u)];
                        (rd16(w, 0x28), i32::from(w[0x3e] as i8))
                    }
                    fn unit_is_militia(&mut self, p: i32, u: i32) -> bool {
                        if self.a[BASED] != 0 {
                            self.rec(8, &format!("p{}+0x0", Self::utw(p, u)), &[0x42, 0]);
                            self.seq(8) != 0
                        } else {
                            self.rec(7, &format!("p{}+0x0", Self::uw(p, u)), &[0x42, 0]);
                            self.seq(7) != 0
                        }
                    }
                    fn cast_civilian(&mut self, p: i32, u: i32) {
                        // `ecx` is whatever the `+0xb8` call left: the unit's type on the base arm, the unit else.
                        let ecx = if self.a[BASED] != 0 {
                            Self::utw(p, u)
                        } else {
                            Self::uw(p, u)
                        };
                        self.rec(31, &format!("p{ecx}+0x0"), &[u, p]);
                    }
                    fn unit_order_kind(&mut self, p: i32, u: i32) -> Option<i32> {
                        self.rec(28, &format!("p{}+0x0", Self::uw(p, u)), &[]);
                        if self.a[NULL_ORDER] != 0 {
                            return None;
                        }
                        self.rec(4, "p47+0x0", &[]);
                        Some(self.seq(4))
                    }
                    fn repath(&mut self, p: i32, u: i32) {
                        self.rec(29, &format!("p{}+0x0", Self::uw(p, u)), &[]);
                    }
                    fn kill_current_order(&mut self, p: i32, u: i32) {
                        self.rec(30, &format!("p{}+0x0", Self::uw(p, u)), &[0]);
                    }
                    fn city_flags(&self, _who: i32, c: i32) -> i32 {
                        i32::from(u16::from_le_bytes([
                            self.p[Self::cw(c)][4],
                            self.p[Self::cw(c)][5],
                        ]))
                    }
                    fn set_city_flags(&mut self, _who: i32, c: i32, v: i32) {
                        self.p[Self::cw(c)][4..6].copy_from_slice(&(v as u16).to_le_bytes());
                    }
                    fn city_building(&self, _who: i32, c: i32) -> i32 {
                        rd16(&self.p[Self::cw(c)], 8)
                    }
                    fn city_radius(&mut self, _who: i32, c: i32) -> i32 {
                        self.rec(18, &format!("p{}+0x0", Self::cw(c)), &[]);
                        self.seq(18)
                    }
                    fn city_cell(&self, _who: i32, c: i32) -> (i32, i32) {
                        self.cell_of(&self.p[Self::cw(c)], 8)
                    }
                    fn city_tile(&self, _who: i32, c: i32) -> (i32, i32) {
                        self.cell_of(&self.p[Self::cw(c)], 6)
                    }
                    fn console_who(&self) -> i32 {
                        rd32(&self.c[2], 0x298)
                    }
                    fn play(&mut self, who: i32, id: i32) {
                        self.rec_raw(format!("s17:{who},{id}"));
                    }
                    fn world_wh(&self) -> (i32, i32) {
                        (rd32(&self.c[1], 0), rd32(&self.c[1], 4))
                    }
                    fn cell_first(&self, x: i32, y: i32) -> (i32, i32) {
                        let i = (rd32(&self.c[1], 0) * y + x) as usize * 28;
                        (rd16(&self.p[43], i + 8), rd16(&self.p[43], i + 0xa))
                    }
                    fn tribe_can_type(&mut self, who: i32) -> bool {
                        self.rec_raw(format!("s21:p{who}+0x0,p46+0x0"));
                        self.seq(21) != 0
                    }
                    fn type_slot68(&mut self, who: i32) -> i32 {
                        let lw = rd32(&self.p[who as usize], 8);
                        self.rec(12, "p46+0x0", &[lw]);
                        self.seq(12)
                    }
                    fn can_garrison(&mut self, k: i32, t: i32) -> bool {
                        let ecx = rd32(&self.p[44], 4 * k as usize);
                        self.rec_raw(format!("s22:{ecx},{t}"));
                        self.seq(22) != 0
                    }
                    fn find_city_at(&mut self, x: i32, y: i32, who: i32) -> i32 {
                        self.rec(23, "c0+0x0", &[x, y, who, -1, 0]);
                        self.seq(23)
                    }
                    fn get_down(&mut self, _x: i32, _y: i32) -> (i32, i32) {
                        self.rec_raw("s24:c1+0x0,sp-0x48,sp-0x28,sp-0x18,sp-0x2c".into());
                        (self.a[33], self.a[34])
                    }
                    fn group_member(&mut self, u: i32, who: i32) -> bool {
                        self.rec(25, "sp-0xa60", &[u, who, 1]);
                        self.seq(25) != 0
                    }
                    fn group_add(&mut self, u: i32, who: i32) {
                        self.rec(26, "sp-0xa60", &[u, who, 0, 0]);
                        self.count += 1;
                    }
                    fn group_count(&self) -> i32 {
                        self.count
                    }
                    fn action_garrison(&mut self, b: i32, who: i32) {
                        self.rec(27, "sp-0xa60", &[b, who, 2, 1]);
                    }
                }
                let mut v = V {
                    a,
                    m: &*m,
                    c: &*c,
                    p,
                    log: &mut log,
                    turn: [0; 33],
                    count: 0,
                };
                action_alarm_steps(&mut v);
                0
            }
        };
        let _ = (got, want_eax);
        if let Some(k) = (0..log.len().max(want_log.len()))
            .find(|&k| log.get(k).map(String::as_str) != want_log.get(k).copied())
        {
            panic!(
                "action_alarm — the recorded calls part at call {k}: port {:?}, original {:?}; before it {:?}; row {n}",
                log.get(k),
                want_log.get(k),
                &want_log[k.saturating_sub(6)..k.min(want_log.len())]
            );
        }
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_alarm — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_alarm — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[9 + i],
                "action_alarm — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("Group::action_alarm@0070ec30: {n} rows agree");
}
