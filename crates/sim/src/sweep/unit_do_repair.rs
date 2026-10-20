//! `Unit::do_repair@005ee420` under the emulator against the port (item 3346; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_repair@005ee420`, run by `tools/emu/sweep_unit_do_repair.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p10 sparse table 0 (2 rows × 5 slice(s), row-major), p11..p12 sparse table 1 (2 rows × 1 slice(s), row-major), p13..p55 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c061f0 points at 2048 random bytes; the global cell 0x00c061ec points at 1536 random bytes; the global cell 0x00c06210 points at 688 random bytes; the global cell 0x00c06208 points at 64 random bytes; argument 0 names 64 random bytes; callee 0x00616f40 (3 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x0070fbe0 (5 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x006e33a0 (2 args) stubbed; callee 0x00a1d590 (1 args) stubbed; callee 0x007e9ab0 (7 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x00604320 (0 args) stubbed; callee 0x00616e80 (0 args) stubbed; callee 0x0061a5c0 (3 args) stubbed; vtable slot +0x70 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x188 (0 args) stubbed; vtable slot +0x170 (2 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x11c (1 args) stubbed; vtable slot +0x18c (1 args) stubbed; vtable slot +0xf8 (0 args) stubbed; vtable slot +0x114 (0 args) stubbed; vtable slot +0x78 (7 args) stubbed; vtable slot +0x168 (3 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; the word at `c0+0x14+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p13+0` points at 96 random bytes less `0` (placed); the word at `p13+4` points at 96 random bytes less `0` (placed); the word at `p13+8` points at 96 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p17+0` points at 96 random bytes less `0` (placed); the word at `p17+4` points at 96 random bytes less `0` (placed); the word at `p17+8` points at 96 random bytes less `0` (placed); the word at `0xc09970+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p21+0` points at 128 random bytes less `0` (placed); the word at `p21+4` points at 128 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p24+0` points at 128 random bytes less `0` (placed); the word at `p24+4` points at 128 random bytes less `0` (placed); the word at `p0+0x10` points at 64 random bytes less `0` (placed); the word at `p27+4` points at 16 random bytes less `0` (placed); the word at `p14+0x20` points at 128 random bytes less `0` (placed); the word at `p14+0x28` points at 128 random bytes less `0` (placed); the word at `p15+0x20` points at 128 random bytes less `0` (placed); the word at `p15+0x28` points at 128 random bytes less `0` (placed); the word at `p16+0x20` points at 128 random bytes less `0` (placed); the word at `p16+0x28` points at 128 random bytes less `0` (placed); the word at `p18+0x20` points at 128 random bytes less `0` (placed); the word at `p18+0x28` points at 128 random bytes less `0` (placed); the word at `p19+0x20` points at 128 random bytes less `0` (placed); the word at `p19+0x28` points at 128 random bytes less `0` (placed); the word at `p20+0x20` points at 128 random bytes less `0` (placed); the word at `p20+0x28` points at 128 random bytes less `0` (placed); the word at `p14+0x18` points at 32 random bytes less `0` (placed); the word at `p15+0x18` points at 32 random bytes less `0` (placed); the word at `p16+0x18` points at 32 random bytes less `0` (placed); the word at `p18+0x18` points at 32 random bytes less `0` (placed); the word at `p19+0x18` points at 32 random bytes less `0` (placed); the word at `p20+0x18` points at 32 random bytes less `0` (placed); the word at `p30+0x18` points at 32 random bytes less `0` (placed); the word at `p32+0x18` points at 32 random bytes less `0` (placed); the word at `p34+0x18` points at 32 random bytes less `0` (placed); the word at `p36+0x18` points at 32 random bytes less `0` (placed); the word at `p38+0x18` points at 32 random bytes less `0` (placed); the word at `p40+0x18` points at 32 random bytes less `0` (placed); the word at `p5+0` points at 24 random bytes less `0` (placed); the word at `p10+0` points at 24 random bytes less `0` (placed); the word at `c2+0x134` points at 1024 random bytes less `0` (placed); slot +0x4c laid as 0x00472350 or its stub, a flag after the answers; slot +0x188 laid as 0x00472420 or its stub, a flag after the answers; stub 2 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [1, 1, 1, 1, 0] (90%); stub 12 answered from [0, 0, 0, 1, 2] (90%); stub 15 answered from [1, 1, 1, 1, 1, 0] (90%); stub 17 answered from [1, 1, 1, 1, 0] (90%); stub 18 answered from [0, 0, 0, 1] (90%); stub 19 answered from [1, 1, 1, 1, 0] (90%); stub 23 answered from [0, 1, 2, 3] (90%); stub 27 answered from [0, 1, 0, 0] (90%); stub 28 answered from [0, 1, 0, 0] (90%); stub 29 answered from [1, 1, 0] (90%); the word at `p30+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p32+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p34+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p36+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p38+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p40+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 7 answers 12 values in call order, printed after the answers; stub 15 answers 3 values in call order, printed after the answers; stub 18 answers 2 values in call order, printed after the answers; stub 21 answers 7 values in call order, printed after the answers; stub 25 answers 6 values in call order, printed after the answers; stub 21 answers 1..80 every row; stub 22 answers 0..200 every row; stub 24 answers 0..80 every row; stub 25 answers 0..9 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 1024 bytes under the call is zeroed before each row; stub 14 answers from its own `ecx`: the word at `+0x10`; stub 16 answers from its own `ecx`: the word at `+0x20`; stub 20 answers from its own `ecx`: the word at `+0x28`; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x8:0x4, +0x74:0x8, +0x208:0x4, +0x468:0x18, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); 2 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x8 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p43+0` as well as at `this`; the stubbed vtable laid at `p44+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; the stubbed vtable laid at `p48+0` as well as at `this`; the stubbed vtable laid at `p49+0` as well as at `this`; the stubbed vtable laid at `p50+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; the stubbed vtable laid at `p52+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 12000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_repair.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(7);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Row: `a[0]` the order pointer (unread), `a[1 + i]` stub `i`'s answer (stubs 0 to
                // 29); then the seq values in flag order: `a[31..43]` `type_avail`'s (12),
                // `a[43..46]` slot `+0x1c`'s, `a[46..48]` slot `+0x188`'s, `a[48..55]` slot
                // `+0x11c`'s, `a[55..61]` slot `+0x78`'s; then `a[61]` whether slot `+0x4c` held
                // `WallData::is_active` (a native read) and `a[62]` the same for `+0x188`.
                // Stubs: 0 `set_anim`, 1 `kill_current_order`, 2 `is_ally`, 3 `Group::clear`, 4
                // `Group::add`, 5 `action_swarm_around`, 6 `has_tribe_bonus`, 7 `type_avail`, 8
                // the complaint's `String`, 9 `add_feedback`, 10 `SoundGlobal::play`, 11
                // `find_repair_spot`, 12 `order_type`, 13 `add_gather_order`, then the slots
                // `+0x70` (the order's body), `+0x1c`, `+0xb0`, `+0x4c`, `+0x188`, `+0x170`, `+0xac`,
                // `+0x11c`, `+0x18c`, `+0xf8`, `+0x114`, `+0x78`, `+0x168`, `+0xb8`, `+0x60`,
                // `+0x90` (14 to 29). Windows: `m` the unit, `c0` the objects cell, `c1`
                // `div_3_table`, `c2` the world, `c3` `constants`, `c4` `game`, `c5` the console;
                // `p0` the order, `p1 + 5r ..` leader row `r`'s `+0x8`, `+0x74`, `+0x208`, `+0x468`
                // and `+0x6eb8`, `p11 + r` the constants leaders' `+0x74`, `p14..16` and `p18..20`
                // the objects of players 0 and 1, `p22/p23`, `p25/p26` their cities, `p27` the
                // order's body (`p28` its virtual-base table), `p29 + 2k` the `+0xb0` record and
                // `p30 + 2k` the `+0xac` record of object `k` (`3 * uid + idx`), `p41 + k` the
                // object's own `+0x18` type, `p47 + k` the `+0xac` record's `+0x18` type, `p53`/`p54`
                // the buckets of leaders 0 and 1, `p55` the tile array.
                //
                // **The port is the crate's own steps**, `crate::orders::do_repair_steps`, which
                // `Sim::do_repair` calls over its `DoRepairView`; here the view is the emulated
                // original's windows. The goods' cost is `LeaderDataEncrypt::scratch`'s only
                // reader nothing of the row holds (a global the original writes beside the
                // bucket), and it is not compared.
                use crate::orders::{DoRepairView, do_repair_steps};
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    n7: usize,
                    n15: usize,
                    n18: usize,
                    n21: usize,
                    n25: usize,
                }
                impl V<'_> {
                    fn idx(&self) -> i32 {
                        rd(&self.p[27], 8)
                    }
                    fn uid(&self) -> i32 {
                        rd(&self.p[27], 0xc)
                    }
                    fn who(&self) -> i32 {
                        i32::from(self.m[9])
                    }
                    fn unit_id(&self) -> i32 {
                        i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]))
                    }
                    /// The object's number among the six laid (`uid * 3 + idx`).
                    fn k(&self) -> usize {
                        (self.uid() * 3 + self.idx()) as usize
                    }
                    fn obj(&self) -> usize {
                        (if self.uid() == 0 { 14 } else { 18 }) + self.idx() as usize
                    }
                    fn b0(&self) -> usize {
                        29 + 2 * self.k()
                    }
                    fn ac(&self) -> usize {
                        30 + 2 * self.k()
                    }
                    fn ty(&self) -> usize {
                        41 + self.k()
                    }
                    fn ac_ty(&self) -> usize {
                        47 + self.k()
                    }
                    /// The leader row's address in the image.
                    fn leader(&self, who: i32) -> i32 {
                        0xe3a390 + who * 0x6eec
                    }
                    /// Slice `s` of the unit's leader row.
                    fn slice(&self, s: usize) -> usize {
                        1 + self.who() as usize * 5 + s
                    }
                    fn tile(&self, w: i32) -> i32 {
                        rd(&self.c[1], (((w ^ 0x63637) >> 8) as usize) * 4)
                    }
                    fn city(&self) -> usize {
                        let cidx =
                            i16::from_le_bytes([self.p[self.ac()][0x72], self.p[self.ac()][0x73]]);
                        22 + 3 * self.uid() as usize + cidx as usize
                    }
                }
                impl DoRepairView for V<'_> {
                    fn set_repair_anim(&mut self) {
                        self.log.push("s0:m+0x0,34,0,1".into());
                    }
                    fn site_damage(&mut self) -> i32 {
                        rd(&self.p[self.obj()], 0x24)
                    }
                    fn order_owner(&mut self) -> i32 {
                        self.uid()
                    }
                    fn unit_player(&mut self) -> i32 {
                        self.who()
                    }
                    fn leader_id(&mut self) -> i32 {
                        rd(&self.p[self.slice(0)], 0)
                    }
                    fn relation(&mut self) -> i32 {
                        rd(&self.p[self.slice(1)], 4 * self.uid() as usize)
                    }
                    fn relation_const(&mut self) -> i32 {
                        let leader = rd(&self.p[self.slice(0)], 0);
                        rd(&self.p[11 + self.uid() as usize], 4 * leader as usize)
                    }
                    fn order_none(&mut self) -> bool {
                        self.log.push("s12:m+0x0".into());
                        self.a[1 + 12] == 0
                    }
                    fn ai_driven(&mut self) -> bool {
                        rd(self.m, 0x68) & 0x40000 != 0
                    }
                    fn stance(&mut self) -> i32 {
                        self.log.push("s23:m+0x0".into());
                        self.a[1 + 23]
                    }
                    fn site_in_use(&mut self) -> bool {
                        self.log.push(format!("s15:p{}+0x0", self.obj()));
                        let r = self.a[43 + self.n15.min(2)];
                        self.n15 += 1;
                        r != 0
                    }
                    fn is_gather_type(&mut self) -> bool {
                        self.log.push(format!("s29:p{}+0x0", self.ac_ty()));
                        self.a[1 + 29] != 0
                    }
                    fn is_university(&mut self) -> bool {
                        if rd(&self.p[self.ac()], 0) == 0xb4_2174 {
                            self.log.push(format!("s28:p{}+0x0", self.ac_ty()));
                            self.a[1 + 28] != 0
                        } else {
                            self.log.push(format!("s27:p{}+0x0,420,0", self.ac()));
                            self.a[1 + 27] != 0
                        }
                    }
                    fn add_gather_order(&mut self) {
                        self.log.push(format!("s13:m+0x0,{},2,0", self.idx()));
                    }
                    fn find_repair_spot(&mut self) {
                        self.log.push("s11:m+0x0".into());
                    }
                    fn kill_current_order(&mut self) {
                        self.log.push("s1:m+0x0,0".into());
                    }
                    fn site_active(&mut self) -> bool {
                        if self.a[61] != 0 {
                            return self.p[self.b0()][8] & 4 != 0;
                        }
                        self.log.push(format!("s17:p{}+0x0", self.b0()));
                        self.a[1 + 17] != 0
                    }
                    fn site_under_attack(&mut self) -> bool {
                        if self.a[62] != 0 {
                            let w = &self.p[self.b0()];
                            return i16::from_le_bytes([w[0x60], w[0x61]]) & 0x20 != 0;
                        }
                        self.log.push(format!("s18:p{}+0x0", self.b0()));
                        let r = self.a[46 + self.n18.min(1)];
                        self.n18 += 1;
                        r != 0
                    }
                    fn order_action(&mut self) -> bool {
                        let disp = rd(&self.p[28], 4) as usize;
                        self.p[27][disp + 8] & 4 != 0
                    }
                    fn tile_owner(&mut self) -> i32 {
                        let o = &self.p[self.obj()];
                        let (x, y) = (rd(o, 0x10), rd(o, 0x14));
                        let i = rd(&self.c[2], 0) * self.tile(y) + self.tile(x);
                        i32::from(self.p[55][i as usize * 0x1c + 0xf] as i8)
                    }
                    fn leader_ally(&mut self, owner: i32) -> bool {
                        self.log
                            .push(format!("s2:{},{owner}", self.leader(self.uid())));
                        self.a[1 + 2] != 0
                    }
                    fn adjacent(&mut self) -> bool {
                        self.log
                            .push(format!("s19:m+0x0,{},{}", self.idx(), self.uid()));
                        self.a[1 + 19] != 0
                    }
                    fn swarm(&mut self, action: bool) {
                        self.log.push("s3:sp-0xa1c,-1".into());
                        self.log.push("s3:sp-0xa1c,-1".into());
                        self.log
                            .push(format!("s4:sp-0xa1c,{},{},0,0", self.unit_id(), self.who()));
                        self.log.push(format!(
                            "s5:sp-0xa1c,{},{},0,13,{}",
                            self.idx(),
                            self.who(),
                            if action { 4 } else { 0 }
                        ));
                    }
                    fn decoy(&mut self) -> bool {
                        rd(self.m, 0x68) & 1 != 0
                    }
                    fn hits(&mut self) -> i32 {
                        self.log.push(format!("s21:p{}+0x0,0", self.b0()));
                        let r = self.a[48 + self.n21.min(6)];
                        self.n21 += 1;
                        r
                    }
                    fn good_hits(&mut self) -> i32 {
                        self.log.push(format!("s21:p{}+0x0,0", self.obj()));
                        let r = self.a[48 + self.n21.min(6)];
                        self.n21 += 1;
                        r
                    }
                    fn accel_construct(&mut self) -> i32 {
                        rd(&self.c[3], 0x22c)
                    }
                    fn construct_time(&mut self) -> i32 {
                        self.log.push(format!("s22:p{}+0x0,1", self.b0()));
                        self.a[1 + 22]
                    }
                    fn helpers(&mut self) -> i32 {
                        i32::from(self.p[self.b0()][0x64])
                    }
                    fn is_city(&mut self) -> bool {
                        self.p[self.b0()][8] & 0x20 != 0
                    }
                    fn captured(&mut self) -> bool {
                        let w = &self.p[self.city()];
                        w[0x5f] != w[0x5e]
                    }
                    fn korean(&mut self) -> bool {
                        self.log.push(format!("s6:{},16", self.leader(self.who())));
                        self.a[1 + 6] != 0
                    }
                    fn korean_repair(&mut self) -> i32 {
                        rd(&self.c[3], 0x7e0)
                    }
                    fn korean_under_fire(&mut self) -> i32 {
                        rd(&self.c[3], 0x7dc)
                    }
                    fn fresh_hit(&mut self) -> bool {
                        self.p[self.b0()][0x60] & 0x10 != 0
                    }
                    fn bump_helpers(&mut self) {
                        let b = self.b0();
                        self.p[b][0x64] = self.p[b][0x64].wrapping_add(1);
                    }
                    fn frame(&mut self) -> i32 {
                        rd(&self.c[4], 0x550)
                    }
                    fn hits_left(&mut self) -> i32 {
                        self.log.push(format!("s24:p{}+0x0", self.obj()));
                        self.a[1 + 24]
                    }
                    fn good_avail(&mut self, good: usize) -> bool {
                        self.log
                            .push(format!("s7:{},{good},1", self.leader(self.who())));
                        let r = self.a[31 + self.n7.min(11)];
                        self.n7 += 1;
                        r != 0
                    }
                    fn price(&mut self, good: usize) -> i32 {
                        self.log.push(format!(
                            "s25:p{}+0x0,{good},{},-1,-1,0,1,-1",
                            self.ty(),
                            self.uid()
                        ));
                        let r = self.a[55 + self.n25.min(5)];
                        self.n25 += 1;
                        r
                    }
                    fn bucket(&mut self, good: usize) -> i32 {
                        rd(&self.p[53 + self.who() as usize], 4 * good) ^ 0x8221
                    }
                    fn feedback_stamp(&mut self) -> i32 {
                        rd(&self.p[self.slice(2)], 0)
                    }
                    fn set_feedback_stamp(&mut self, frame: i32) {
                        let s = self.slice(2);
                        wr(&mut self.p[s], 0, frame);
                    }
                    fn is_console_player(&mut self) -> bool {
                        rd(&self.c[5], 0x298) == self.who()
                    }
                    fn feedback(&mut self) {
                        self.log.push("s8:sp-0xa44,73580".into());
                        self.log.push("s10:c6+0x0,64".into());
                    }
                    fn charge(&mut self, good: usize, d: i32) {
                        let w = 53 + self.who() as usize;
                        let new = (rd(&self.p[w], 4 * good) ^ 0x8221).wrapping_sub(d);
                        wr(&mut self.p[w], 4 * good, new ^ 0x8221);
                        let e = self.slice(3);
                        let left = rd(&self.p[e], 4 * good).wrapping_sub(d);
                        wr(&mut self.p[e], 4 * good, left.max(0));
                    }
                    fn repair_damage(&mut self, amount: i32) {
                        self.log
                            .push(format!("s26:p{}+0x0,{amount},0,1", self.obj()));
                    }
                }
                let mut v = V {
                    a,
                    m,
                    c,
                    p,
                    log: &mut log,
                    n7: 0,
                    n15: 0,
                    n18: 0,
                    n21: 0,
                    n25: 0,
                };
                do_repair_steps(&mut v);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "do_repair — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_repair — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_repair — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[8 + i], "do_repair — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 12000, "only {n} rows");
    eprintln!("Unit::do_repair@005ee420: {n} rows agree");
}
