//! `Unit::do_gather@005ef2a0` under the emulator against the port (item 3356; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_gather@005ef2a0`, run by `tools/emu/sweep_unit_do_gather_005ef2a0.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p2 sparse table 0 (2 rows × 1 slice(s), row-major), p3..p50 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c06188 points at 64 random bytes; the global cell 0x00c061ec points at 1536 random bytes; the global cell 0x00c0a914 points at 1024 random bytes; 4 random bytes at 0x00ee1088 (each word in 0..1); argument 0 names 64 random bytes; callee 0x005e2cb0 (1 args) stubbed; callee 0x005e3df0 (1 args) stubbed; callee 0x0062f520 (1 args) stubbed; callee 0x00630450 (2 args) stubbed; callee 0x0062f640 (2 args) stubbed; callee 0x006439b0 (2 args) stubbed; callee 0x00616ed0 (9 args) stubbed; callee 0x0046f960 (0 args) stubbed; callee 0x0061de70 (16 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x0046d330 (0 args) stubbed; callee 0x0061a2e0 (3 args) stubbed; callee 0x0062f710 (0 args) stubbed; callee 0x00646c40 (1 args) stubbed; callee 0x005f9400 (2 args) stubbed; callee 0x00647080 (3 args) stubbed; callee 0x006ec000 (0 args) stubbed; callee 0x005e4ff0 (4 args) stubbed; callee 0x00616f40 (3 args) stubbed; callee 0x00605400 (3 args) stubbed; callee 0x005f8d20 (4 args) stubbed; callee 0x008d9160 (1 args) stubbed; callee 0x00643440 (2 args) stubbed; callee 0x008d91c0 (3 args) stubbed; callee 0x008d9240 (3 args) stubbed; callee 0x0043cca0 (0 args) stubbed; callee 0x005f0170 (1 args) stubbed; callee 0x00a1edd0 (1 args) stubbed; callee 0x00a1d660 (1 args) stubbed; callee 0x00a2e550 (6 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; vtable slot +0x6c (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x94 (0 args) stubbed; vtable slot +0x60 (0 args) stubbed; vtable slot +0x188 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x170 (2 args) stubbed; vtable slot +0x158 (3 args) stubbed; the word at `c0+0x14+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p3+0` points at 96 random bytes less `0` (placed); the word at `p3+4` points at 96 random bytes less `0` (placed); the word at `p3+8` points at 96 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p7+0` points at 96 random bytes less `0` (placed); the word at `p7+4` points at 96 random bytes less `0` (placed); the word at `p7+8` points at 96 random bytes less `0` (placed); the word at `0xc09970+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p11+0` points at 128 random bytes less `0` (placed); the word at `p11+4` points at 128 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p14+0` points at 128 random bytes less `0` (placed); the word at `p14+4` points at 128 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p17+0` points at 64 random bytes less `0` (placed); the word at `p17+4` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p20+0` points at 64 random bytes less `0` (placed); the word at `p20+4` points at 64 random bytes less `0` (placed); the word at `p0+0x10` points at 64 random bytes less `0` (placed); the word at `m+0x18` points at 576 random bytes less `0` (placed); the word at `m+0xf4` points at 8 random bytes less `0` (placed); the word at `p25+0` points at 160 random bytes less `0` (placed); the word at `p4+0x20` points at 64 random bytes less `0` (placed); the word at `p5+0x20` points at 64 random bytes less `0` (placed); the word at `p6+0x20` points at 64 random bytes less `0` (placed); the word at `p8+0x20` points at 64 random bytes less `0` (placed); the word at `p9+0x20` points at 64 random bytes less `0` (placed); the word at `p10+0x20` points at 64 random bytes less `0` (placed); the word at `p4+0x28` points at 160 random bytes less `0` (placed); the word at `p5+0x28` points at 160 random bytes less `0` (placed); the word at `p6+0x28` points at 160 random bytes less `0` (placed); the word at `p8+0x28` points at 160 random bytes less `0` (placed); the word at `p9+0x28` points at 160 random bytes less `0` (placed); the word at `p10+0x28` points at 160 random bytes less `0` (placed); the word at `p27+0x18` points at 576 random bytes less `0` (placed); the word at `p28+0x18` points at 576 random bytes less `0` (placed); the word at `p29+0x18` points at 576 random bytes less `0` (placed); the word at `p30+0x18` points at 576 random bytes less `0` (placed); the word at `p31+0x18` points at 576 random bytes less `0` (placed); the word at `p32+0x18` points at 576 random bytes less `0` (placed); the word at `p33+0x18` points at 576 random bytes less `0` (placed); the word at `p34+0x18` points at 576 random bytes less `0` (placed); the word at `p35+0x18` points at 576 random bytes less `0` (placed); the word at `p36+0x18` points at 576 random bytes less `0` (placed); the word at `p37+0x18` points at 576 random bytes less `0` (placed); the word at `p38+0x18` points at 576 random bytes less `0` (placed); slot +0x158 laid as 0x0060eda0 or its stub, a flag after the answers; stub 2 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 8 answered from [0, 0, 0, 1] (90%); stub 9 answered from [0, 1, 2] (90%); stub 10 answered from [0, 0, 0, 1] (90%); stub 13 answered from [0, 1, 1] (90%); stub 16 answered from [2, 3, 2, 3, 2, 3, 1, 0] (90%); stub 21 answered from [0, 0, 1, 2] (90%); stub 34 answered from [1, 1, 1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 35 answered from [1, 1, 1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 36 answered from [0, 1, 1] (90%); stub 37 answered from [0, 1, 1] (90%); stub 38 answered from [0, 0, 0, 1] (90%); stub 39 answered from [0, 1, 1] (90%); stub 40 answered from [0, 1, 1] (90%); the word at `p33+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p34+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p35+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p36+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p37+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p38+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 2 answers 3 values in call order, printed after the answers; stub 3 answers 3 values in call order, printed after the answers; stub 5 answers 2 values in call order, printed after the answers; stub 9 answers 2 values in call order, printed after the answers; stub 25 answers 2 values in call order, printed after the answers; stub 3 answers -2..6 every row; stub 25 answers 0..7 every row; stub 29 answers 0..0 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 39 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 40 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 41 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 9 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 9 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 8 writes a value in [0, 100, 5000, 40000, -300, 1000] through its argument 2, printed after the answers (`o` in a place); stub 8 writes a value in [0, 100, 5000, 40000, -300, 1000] through its argument 3, printed after the answers (`o` in a place); stub 22 writes a value in [0, 1, 2, 3, 4, 5] through its argument 0, printed after the answers (`o` in a place); stub 22 writes a value in [0, 1, 2, 3, 4, 5] through its argument 1, printed after the answers (`o` in a place); the stack 1024 bytes under the call is zeroed before each row; stub 31 answers from its own `ecx`: the word at `+0x10`; stub 32 answers from its own `ecx`: the word at `+0x20`; stub 33 answers from its own `ecx`: the word at `+0x28`; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; the stubbed vtable laid at `p48+0` as well as at `this`; the stubbed vtable laid at `p49+0` as well as at `this`; the stubbed vtable laid at `p50+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 8000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_gather_005ef2a0.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Row: `a[0]` the order pointer (unread), `a[1 + i]` stub `i`'s answer (stubs 0 to
                // 41); then `a[43..47]` the values stubs write through their arguments (the spot
                // search's `x` and `y`, then `tile_corner`'s corner `x` and `y`), then the seq
                // values in flag order: `a[47..50]` `is_gathered_by`'s, `a[50..53]`
                // `num_gatherers`', `a[53..55]` `covers_tile`'s, `a[55..57]` `get_tregion`'s,
                // `a[57..59]` `rnd`'s; then `a[59]` whether slot `+0x158` held the image's
                // `Object::die` (a native compare).
                // Stubs: 0 `kill_current_order`, 1 `add_think_order`, 2 `is_gathered_by`, 3
                // `num_gatherers`, 4 `add_gatherer`, 5 `covers_tile`, 6 `add_move_order`, 7
                // `can_transport`, 8 `find_nearby_spot`, 9 `get_tregion`, 10 `is_scholar`, 11
                // `go_inside`, 12 `check_gatherers`, 13 `can_carry`, 14 `same_damage`, 15
                // `Object::die`, 16 `get_diff`, 17 `add_repair_order`, 18 `set_anim`, 19
                // `set_angle`, 20 `set_new_location`, 21 `get_farm_type`, 22 `tile_corner`, 23
                // `Farms::grow`, 24 `Farms::snip`, 25 `GameAccess::rnd`, 26
                // `do_non_flat_gather`, 27..28 the two `String`s, 29 `Error::report`, 30
                // `String::close`; then the slots `+0x6c` (the order's body), `+0xb0`, `+0xac`
                // (the site's records), `+0xc`, `+0x4c`, `+0x94`, `+0x60`, `+0x188`, `+0xb8`,
                // `+0x170`, `+0x158` (31 to 41). Windows: `m` the unit, `c0` the objects cell,
                // `c1` `div_3_table`, `c2` the world, `c3` `game`, `c4` the farm table, `c5`
                // `ignore_always`; `p0` the order, `p1`/`p2` the leaders' first words, `p3 + 4h
                // ..` the objects of player `h` (`p4..6`, `p8..10`), `p12..13`/`p15..16` their
                // cities, `p18..19`/`p21..22` the unit tables, `p23` the order's body, `p24` the
                // unit's type, `p25`/`p26` the unit's guy, `p27 + k` the `+0xb0` record, `p33 +
                // k` the `+0xac` record, `p39 + k` and `p45 + k` their types, `k = 3 * uid +
                // idx`.
                use crate::orders::{DoGatherView, do_gather_steps};
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    n2: usize,
                    n3: usize,
                    n5: usize,
                    n9: usize,
                    n25: usize,
                    /// The leftover `ecx` the last recorded question left.
                    leftover: String,
                    /// Every window's start, by the name the recorded tokens give it.
                    deref: Vec<Vec<String>>,
                }
                fn hex(v: i32) -> String {
                    v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
                }
                impl V<'_> {
                    fn idx(&self) -> i32 {
                        rd(&self.p[23], 8)
                    }
                    fn uid(&self) -> i32 {
                        rd(&self.p[23], 0xc)
                    }
                    fn who(&self) -> i32 {
                        i32::from(self.m[9])
                    }
                    fn id(&self) -> i32 {
                        i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]))
                    }
                    fn k(&self) -> usize {
                        (self.uid() * 3 + self.idx()) as usize
                    }
                    fn obj(&self) -> usize {
                        [4, 5, 6, 8, 9, 10][self.k()]
                    }
                    fn b0(&self) -> usize {
                        27 + self.k()
                    }
                    fn ac(&self) -> usize {
                        33 + self.k()
                    }
                    fn b0t(&self) -> usize {
                        39 + self.k()
                    }
                    fn act(&self) -> usize {
                        45 + self.k()
                    }
                    fn tile(&self, w: i32) -> i32 {
                        rd(&self.c[1], (((w ^ 0x63637) >> 6) as usize) * 4)
                    }
                    fn short(w: &[u8], off: usize) -> i32 {
                        i32::from(i16::from_le_bytes([w[off], w[off + 1]]))
                    }
                    fn push(&mut self, s: String) {
                        self.log.push(s);
                    }
                    /// Whether the site record carries the image's `Build` vtable.
                    fn is_build(&self) -> bool {
                        rd(&self.p[self.ac()], 0) == 0xb4_2174
                    }
                }
                impl DoGatherView for V<'_> {
                    fn order_owner(&mut self) -> i32 {
                        self.uid()
                    }
                    fn unit_player(&mut self) -> i32 {
                        self.who()
                    }
                    fn site_alive(&mut self) -> bool {
                        self.a[1 + 34] != 0
                    }
                    fn site_active(&mut self) -> bool {
                        self.a[1 + 35] != 0
                    }
                    fn been_there(&mut self) -> bool {
                        self.p[23][0x27] != 0
                    }
                    fn set_been_there(&mut self, on: bool) {
                        self.p[23][0x27] = u8::from(on);
                    }
                    fn build_type(&mut self) -> i32 {
                        rd(&self.p[23], 0x1c)
                    }
                    fn is_gathered_by(&mut self) -> bool {
                        let s = format!("s2:p{}+0x0,{}", self.ac(), self.id());
                        self.push(s);
                        self.leftover = format!("p{}+0x0", self.ac());
                        let r = self.a[47 + self.n2.min(2)];
                        self.n2 += 1;
                        r != 0
                    }
                    fn max_gatherers(&mut self) -> i32 {
                        i32::from(self.p[self.ac()][0x80] as i8)
                    }
                    fn num_gatherers(&mut self) -> i32 {
                        let s = format!("s3:p{}+0x0,0,0", self.ac());
                        self.push(s);
                        let r = self.a[50 + self.n3.min(2)];
                        self.n3 += 1;
                        r
                    }
                    fn unit_domain(&mut self) -> i32 {
                        rd(&self.p[24], 0x218)
                    }
                    fn site_domain(&mut self) -> i32 {
                        rd(&self.p[self.act()], 0x218)
                    }
                    fn add_gatherer(&mut self) {
                        let s = format!("s4:p{}+0x0,{},{}", self.ac(), self.id(), self.who());
                        self.push(s);
                    }
                    fn kill_current_order(&mut self) {
                        self.push("s0:m+0x0,0".into());
                    }
                    fn add_think_order(&mut self) {
                        self.push("s1:m+0x0,m+0x0".into());
                    }
                    fn site_flat(&mut self) -> bool {
                        self.a[1 + 36] != 0
                    }
                    fn covers_unit_tile(&mut self) -> bool {
                        let (x, y) = (self.tile(rd(self.m, 0x10)), self.tile(rd(self.m, 0x14)));
                        let s = format!("s5:p{}+0x0,{x},{y}", self.ac());
                        self.push(s);
                        let r = self.a[53 + self.n5.min(1)];
                        self.n5 += 1;
                        r != 0
                    }
                    fn unit_pos(&mut self) -> (i32, i32) {
                        (rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637)
                    }
                    fn site_pos(&mut self) -> (i32, i32) {
                        let o = &self.p[self.obj()];
                        (rd(o, 0x10) ^ 0x63637, rd(o, 0x14) ^ 0x63637)
                    }
                    fn add_move_order(&mut self, x: i32, y: i32) {
                        self.push(format!("s6:m+0x0,{x},{y},1,0,0,0,{y},-1,-1"));
                    }
                    fn add_move_order_spot(&mut self, x: i32, y: i32) {
                        let l = self.leftover.clone();
                        self.push(format!("s6:m+0x0,{x},{y},1,0,0,0,{l},-1,-1"));
                    }
                    fn site_city(&mut self) -> i32 {
                        Self::short(&self.p[self.ac()], 0x72)
                    }
                    fn farm_index(&mut self) -> i32 {
                        Self::short(&self.p[self.ac()], 0x78)
                    }
                    fn frame(&mut self) -> i32 {
                        rd(&self.c[3], 0x550)
                    }
                    fn unit_id(&mut self) -> i32 {
                        self.id()
                    }
                    fn non_flat_gather(&mut self) {
                        self.push("s26:m+0x0,p23+0x0".into());
                    }
                    fn adjacent(&mut self) -> bool {
                        let s = format!("s40:m+0x0,{},{}", self.idx(), self.uid());
                        self.push(s);
                        self.a[1 + 40] != 0
                    }
                    fn site_min_size(&mut self) -> i32 {
                        let t = &self.p[self.b0t()];
                        rd(t, 0x234).min(rd(t, 0x238))
                    }
                    fn site_size(&mut self) -> (i32, i32) {
                        let t = &self.p[self.act()];
                        (rd(t, 0x234), rd(t, 0x238))
                    }
                    fn can_transport(&mut self) -> bool {
                        self.push("s7:m+0x0".into());
                        self.a[1 + 7] != 0
                    }
                    fn find_spot(&mut self, dist: i32, angle: i32) -> Option<(i32, i32)> {
                        let (sx, sy) = self.site_pos();
                        let s = format!(
                            "s8:p24+0x0,{sx},{sy},sp-0x34,sp-0x30,{dist},-1,0,{angle},3,{},{},0,0,-1,0,-1",
                            self.id(),
                            self.who()
                        );
                        self.push(s);
                        if self.a[1 + 8] != 0 {
                            None
                        } else {
                            Some((self.a[43], self.a[44]))
                        }
                    }
                    fn set_cant_reach(&mut self) {
                        self.m[8] |= 0x10;
                    }
                    fn region_differs(&mut self) -> bool {
                        let (sx, sy) = self.site_pos();
                        let _ = (sx, sy);
                        let o = &self.p[self.obj()];
                        let (ox, oy) = (self.tile(rd(o, 0x10)), self.tile(rd(o, 0x14)));
                        let (ux, uy) = (self.tile(rd(self.m, 0x10)), self.tile(rd(self.m, 0x14)));
                        self.push("s9:c2+0x0,sp-0x30,sp-0x34".into());
                        self.deref.push(vec![hex(ox), hex(oy)]);
                        self.push("s9:c2+0x0,sp-0x28,sp-0x2c".into());
                        self.deref.push(vec![hex(ux), hex(uy)]);
                        let first = self.a[55];
                        let second = self.a[56];
                        self.n9 = 2;
                        first != second
                    }
                    fn mark_leader(&mut self) {
                        let w = self.who() as usize;
                        let v = rd(&self.p[1 + w], 0) | 0x200_0000;
                        wr(&mut self.p[1 + w], 0, v);
                    }
                    fn is_scholar(&mut self) -> bool {
                        self.push("s10:m+0x0".into());
                        self.a[1 + 10] != 0
                    }
                    fn go_inside(&mut self) {
                        let s = format!("s11:m+0x0,{},{},0", self.idx(), self.uid());
                        self.push(s);
                    }
                    fn check_gatherers(&mut self) {
                        let s = format!("s12:p{}+0x0", self.ac());
                        self.push(s);
                    }
                    fn is_platform(&mut self) -> bool {
                        if self.is_build() {
                            self.leftover = format!("p{}+0x0", self.act());
                            self.a[1 + 37] != 0
                        } else {
                            let s = format!("s39:p{}+0x0,422,0", self.ac());
                            self.push(s);
                            self.leftover = format!("p{}+0x0", self.ac());
                            self.a[1 + 39] != 0
                        }
                    }
                    fn can_carry_ground(&mut self) -> bool {
                        self.push("s13:m+0x0,0".into());
                        self.a[1 + 13] != 0
                    }
                    fn carrier_same_damage(&mut self) {
                        if i32::from(i16::from_le_bytes([self.m[0x28], self.m[0x29]])) >= 0 {
                            let o = usize::from(self.m[0x3e]);
                            let i = usize::from(i16::from_le_bytes([self.m[0x28], self.m[0x29]]) as u16);
                            let w = [[18, 19], [21, 22]][o][i];
                            let s = format!("s14:p{w}+0x0,{},{}", self.id(), self.who());
                            self.push(s);
                        }
                    }
                    fn die(&mut self) {
                        if self.a[59] != 0 {
                            self.push("s15:m+0x0,0,-1,0".into());
                        } else {
                            self.push("s41:m+0x0,0,-1,0".into());
                        }
                    }
                    fn ai_driven(&mut self) -> bool {
                        rd(self.m, 0x68) & 0x40000 != 0
                    }
                    fn leader_diff(&mut self) -> i32 {
                        let s = format!("s16:p{}+0x0", 1 + self.who() as usize);
                        self.push(s);
                        self.a[1 + 16]
                    }
                    fn site_damage(&mut self) -> i32 {
                        rd(&self.p[self.ac()], 0x24)
                    }
                    fn site_under_attack(&mut self) -> bool {
                        if self.is_build() {
                            Self::short(&self.p[self.ac()], 0x60) & 0x20 != 0
                        } else {
                            self.a[1 + 38] != 0
                        }
                    }
                    fn city_under_attack(&mut self) -> bool {
                        let c = Self::short(&self.p[self.ac()], 0x72) as usize;
                        self.p[[12, 15][self.uid() as usize..][0] + c][4] & 2 != 0
                    }
                    fn add_repair_order(&mut self) {
                        let s = format!("s17:m+0x0,{},{},2,0", self.idx(), self.uid());
                        self.push(s);
                    }
                    fn site_counted(&mut self) -> bool {
                        Self::short(&self.p[self.ac()], 0x60) & 0x800 != 0
                    }
                    fn bump_site_counter(&mut self) {
                        let ac = self.ac();
                        let n = Self::short(&self.p[ac], 0x7a).wrapping_add(1) as u16;
                        self.p[ac][0x7a..0x7c].copy_from_slice(&n.to_le_bytes());
                        let f = (Self::short(&self.p[ac], 0x60) | 0x800) as u16;
                        self.p[ac][0x60..0x62].copy_from_slice(&f.to_le_bytes());
                    }
                    fn cur_anim(&mut self) -> i32 {
                        i32::from(self.p[26][0x9c])
                    }
                    fn set_anim(&mut self, anim: i32) {
                        self.push(format!("s18:m+0x0,{anim},0,1"));
                    }
                    fn face_east(&mut self) {
                        self.push("s19:m+0x0,1073741824,m+0x0,0".into());
                    }
                    fn set_new_location(&mut self, x: i32, y: i32) {
                        self.push(format!("s20:m+0x0,{x},{y},1,1"));
                    }
                    fn farm_type(&mut self, farm: i32) -> i32 {
                        self.push(format!("s21:12626160,{farm}"));
                        self.a[1 + 21]
                    }
                    fn tile_corner(&mut self) -> (i32, i32) {
                        let s = format!("s22:p{}+0x0,sp-0x20,sp-0x24", self.ac());
                        self.push(s);
                        (self.a[45], self.a[46])
                    }
                    fn unit_tile(&mut self) -> (i32, i32) {
                        (self.tile(rd(self.m, 0x10)), self.tile(rd(self.m, 0x14)))
                    }
                    fn farm_cell(&mut self, farm: i32, fx: i32, fy: i32) -> i32 {
                        let at = ((farm * 0x30 + fx) * 4 + fy + 0xac) as usize;
                        i32::from(self.c[4][at])
                    }
                    fn farm_grow(&mut self, farm: i32, fy: i32, fx: i32) {
                        self.push(format!("s23:m+0x0,{farm},{fy},{fx}"));
                    }
                    fn farm_snip(&mut self, farm: i32, fy: i32, fx: i32) {
                        self.push(format!("s24:m+0x0,{farm},{fy},{fx}"));
                    }
                    fn rnd(&mut self, n: i32) -> i32 {
                        self.push(format!("s25:{n}"));
                        let r = self.a[57 + self.n25.min(1)];
                        self.n25 += 1;
                        r
                    }
                    fn invalid_farm_cell(&mut self) {
                        if self.c[5][0] == 0 {
                            self.push("s29:13161384,sp-0x48,sp-0x48,0,6225271,c5+0x0,1".into());
                        }
                    }
                }
                let mut v = V {
                    a,
                    m,
                    c,
                    p,
                    log: &mut log,
                    n2: 0,
                    n3: 0,
                    n5: 0,
                    n9: 0,
                    n25: 0,
                    leftover: String::new(),
                    deref: Vec::new(),
                };
                do_gather_steps(&mut v);
                let deref = std::mem::take(&mut v.deref);
                // The two `get_tregion` columns: the tile bytes each call's arguments pointed at.
                let (c0, c1): (Vec<&str>, Vec<&str>) = deref
                    .iter()
                    .map(|d| (d[0].as_str(), d[1].as_str()))
                    .unzip();
                log.push(if c0.is_empty() { "-".into() } else { c0.join("/") });
                log.push(if c1.is_empty() { "-".into() } else { c1.join("/") });
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "do_gather — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_gather — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_gather — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[7 + i], "do_gather — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 8000, "only {n} rows");
    eprintln!("Unit::do_gather@005ef2a0: {n} rows agree");
}
