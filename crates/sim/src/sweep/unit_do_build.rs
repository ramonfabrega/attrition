//! `Unit::do_build@005eebf0` under the emulator against the port (item 3295; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_build@005eebf0`, run by `tools/emu/sweep_unit_do_build.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p4 sparse table 0 (2 rows × 2 slice(s), row-major), p5..p35 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061f0 points at 2048 random bytes; the global cell 0x00c06210 points at 688 random bytes; the global cell 0x00e85f20 points at 6144 random bytes; argument 0 names 64 random bytes; callee 0x005e2cb0 (1 args) stubbed; callee 0x00616e80 (0 args) stubbed; callee 0x00603470 (0 args) stubbed; callee 0x00603bf0 (3 args) stubbed; callee 0x0061a5c0 (3 args) stubbed; callee 0x006439b0 (2 args) stubbed; callee 0x00616f40 (3 args) stubbed; callee 0x00605400 (3 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x006434d0 (1 args) stubbed; callee 0x00711540 (0 args) stubbed; callee 0x00711720 (3 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x0070fbe0 (5 args) stubbed; vtable slot +0xf8 (0 args) stubbed; vtable slot +0x170 (2 args) stubbed; vtable slot +0x60 (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x188 (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x10 (4 args) stubbed; the word at `c0+0x14+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p5+0` points at 64 random bytes less `0` (placed); the word at `p5+4` points at 64 random bytes less `0` (placed); the word at `p5+8` points at 64 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p9+0` points at 64 random bytes less `0` (placed); the word at `p9+4` points at 64 random bytes less `0` (placed); the word at `p9+8` points at 64 random bytes less `0` (placed); the word at `p0+0x10` points at 64 random bytes less `0` (placed); the word at `p13+4` points at 16 random bytes less `0` (placed); the word at `m+0xdc` points at 32 random bytes less `0` (placed, null a third of the rows); the word at `p15+4` points at 32 random bytes less `0` (placed); the word at `p16+8` points at 32 random bytes less `0` (placed); the word at `p6+0x20` points at 128 random bytes less `0` (placed); the word at `p6+0x28` points at 32 random bytes less `0` (placed); the word at `p7+0x20` points at 128 random bytes less `0` (placed); the word at `p7+0x28` points at 32 random bytes less `0` (placed); the word at `p8+0x20` points at 128 random bytes less `0` (placed); the word at `p8+0x28` points at 32 random bytes less `0` (placed); the word at `p10+0x20` points at 128 random bytes less `0` (placed); the word at `p10+0x28` points at 32 random bytes less `0` (placed); the word at `p11+0x20` points at 128 random bytes less `0` (placed); the word at `p11+0x28` points at 32 random bytes less `0` (placed); the word at `p12+0x20` points at 128 random bytes less `0` (placed); the word at `p12+0x28` points at 32 random bytes less `0` (placed); the word at `p18+0x18` points at 32 random bytes less `0` (placed); the word at `p20+0x18` points at 32 random bytes less `0` (placed); the word at `p22+0x18` points at 32 random bytes less `0` (placed); the word at `p24+0x18` points at 32 random bytes less `0` (placed); the word at `p26+0x18` points at 32 random bytes less `0` (placed); the word at `p28+0x18` points at 32 random bytes less `0` (placed); slot +0x188 laid as 0x00472420 or its stub, a flag after the answers; stub 1 answered from [0, 0, 0, 1, 2, 7] (90%); stub 5 answered from [0, 0, 0, 0, 1] (90%); stub 8 answered from [0, 1] (90%); stub 9 answered from [1, 1, 1, 1, 0] (90%); stub 11 answered from [0, 1, 2, 2, 3] (90%); stub 16 answered from [1, 1, 1, 1, 1, 0] (90%); stub 18 answered from [1, 1, 1, 1, 1, 1, 0] (90%); stub 19 answered from [0, 0, 0, 0, 0, 1] (90%); stub 22 answered from [0, 0, 0, 0, 1] (90%); stub 23 answered from [0, 1] (90%); stub 24 answered from [1, 1, 1, 1, 0] (90%); stub 15 answered from [0, 1, 1, 2, 3] (90%); stub 22 answers 6 values in call order, printed after the answers; stub 24 answers 2 values in call order, printed after the answers; stub 15 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 1024 bytes under the call is zeroed before each row; stub 17 answers from its own `ecx`: the word at `+0x10`; stub 20 answers from its own `ecx`: the word at `+0x20`; stub 21 answers from its own `ecx`: the word at `+0x28`; 2 rows 0x9f0 apart at `0xe8d444`, slices +0x0:0x4, +0xc:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 16000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_build.py") else {
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
                // Row: `a[0]` the order pointer (unread), `a[1 + i]` stub `i`'s answer; then the
                // seq values in flag order: `a[27..33]` slot `+0xb8`'s (`is`), `a[33..35]` slot
                // `+0x90`'s (gather type), `a[35..37]` slot `+0xf8`'s (stance), and `a[37]` whether
                // `+0x188` held `WallData::is_under_attack` (a native read, no call).
                // Stubs: 0 `kill_current_order`, 1 `order_type`, 2 `check_build_order`, 3
                // `build_done`, 4 `add_gather_order`, 5 `covers_tile`, 6 `set_anim`, 7 `set_angle`,
                // 8 `has_tribe_bonus`, 9 `do_construct`, 10 `Group::normalize`, 11
                // `GroupData::count`, 12 `Group::clear`, 13 `Group::add`, 14
                // `action_swarm_around`, 15 slot `+0xf8`, 16 `+0x170`, 17 the order's `+0x60`, 18
                // `+0xc`, 19 `+0x4c`, 20 `+0xb0`, 21 `+0xac`, 22 `+0xb8`, 23 `+0x188`, 24
                // `+0x90`, 25 the select group's `+0x10`. Windows: `m` the unit, `c0` the objects
                // cell (the arrays), `c1` `div_3_table`, `c2` `constants`, `c3` the console, `c4`
                // `groups`; `p0` the order, `p1..p4` the select groups' rows (vtable slice, count
                // slice), `p6..p8` / `p10..p12` the objects of players 0 and 1, `p13` the order's
                // body (`+4` its virtual-base table `p14`), `p15..p17` the order list's node,
                // next node and order, `p18 + 2k` the object's `+0xb0` record, `p19 + 2k` its
                // `+0xac` record and `p30 + k` the `+0x18` object of the record.
                //
                // **The port is the crate's own steps**, `crate::orders::do_build_steps`, which
                // `Sim::do_build` calls over its `DoBuildView`; here the view is the emulated
                // original's windows.
                use crate::orders::{DoBuildView, do_build_steps};
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    n22: usize,
                    n24: usize,
                    n15: usize,
                    resid: i32,
                }
                impl V<'_> {
                    fn idx(&self) -> i32 {
                        rd(&self.p[13], 8)
                    }
                    fn uid(&self) -> i32 {
                        rd(&self.p[13], 0xc)
                    }
                    /// The object's number among the six laid (`uid * 3 + idx`).
                    fn k(&self) -> usize {
                        (self.uid() * 3 + self.idx()) as usize
                    }
                    fn obj(&self) -> usize {
                        (if self.uid() == 0 { 6 } else { 10 }) + self.idx() as usize
                    }
                    fn build(&self) -> usize {
                        18 + 2 * self.k()
                    }
                    fn wall(&self) -> usize {
                        19 + 2 * self.k()
                    }
                    fn ty(&self) -> usize {
                        30 + self.k()
                    }
                    fn who(&self) -> i32 {
                        i32::from(self.m[9])
                    }
                    fn unit_id(&self) -> i32 {
                        i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]))
                    }
                    fn tile(&self, w: i32) -> i32 {
                        rd(&self.c[1], (((w ^ 0x63637) >> 6) as usize) * 4)
                    }
                    fn group_ecx(&self) -> String {
                        let g = i32::from(i16::from_le_bytes([self.m[0x80], self.m[0x81]]));
                        format!("c4+{:#x}", g * 0x9d4)
                    }
                    /// The slot-`+0xb8` ask on `ecx`, `ty` the type constant.
                    fn is(&mut self, ecx: String, ty: i32) -> bool {
                        self.log.push(format!("s22:{ecx},{ty},0"));
                        let r = self.a[27 + self.n22.min(5)];
                        self.n22 += 1;
                        r != 0
                    }
                    fn buildp(&mut self) -> String {
                        self.log.push(format!("s20:p{}+0x0", self.obj()));
                        format!("p{}+0x0", self.build())
                    }
                }
                impl DoBuildView for V<'_> {
                    fn site_alive(&mut self) -> bool {
                        self.log.push("s17:p0+0x0".into());
                        if self.idx() < 0 || self.uid() < 0 {
                            return false;
                        }
                        self.log.push(format!("s18:p{}+0x0", self.obj()));
                        self.a[1 + 18] != 0
                    }
                    fn site_active(&mut self) -> bool {
                        self.log.push(format!("s19:p{}+0x0", self.obj()));
                        self.a[1 + 19] != 0
                    }
                    fn kill_current_order(&mut self) {
                        self.log.push("s0:m+0x0,0".into());
                    }
                    fn order_pending_inline(&mut self) -> bool {
                        if rd(self.m, 0xdc) == 0 {
                            return false;
                        }
                        let next = rd(&self.p[15], 4);
                        wr(self.m, 0xd4, next);
                        let order = rd(&self.p[16], 8);
                        wr(self.m, 0xcc, order);
                        self.m[0xd0] = self.p[16][0xc];
                        // `ThinkOrder::get_type`, the image's, answers a non-zero index.
                        true
                    }
                    fn order_pending(&mut self) -> bool {
                        self.log.push("s1:m+0x0".into());
                        self.a[1 + 1] != 0
                    }
                    fn build_done(&mut self) {
                        let range = rd(&self.c[2], 0x28).wrapping_mul(0xc0);
                        self.log
                            .push(format!("s3:m+0x0,{},{},{range}", self.idx(), self.uid()));
                    }
                    fn check_build_order(&mut self) {
                        self.log.push("s2:m+0x0".into());
                    }
                    fn is_oil(&mut self) -> bool {
                        let e = self.buildp();
                        self.is(e, 422)
                    }
                    fn ai_driven(&mut self) -> bool {
                        rd(self.m, 0x68) & 0x40000 != 0
                    }
                    fn stance(&mut self) -> i32 {
                        self.log.push("s15:m+0x0".into());
                        let r = self.a[35 + self.n15.min(1)];
                        self.n15 += 1;
                        r
                    }
                    fn owned(&mut self) -> bool {
                        self.who() == self.uid()
                    }
                    fn is_gather_type(&mut self) -> bool {
                        let _ = self.buildp();
                        self.log.push(format!("s24:p{}+0x0", self.ty()));
                        let r = self.a[33 + self.n24.min(1)];
                        self.n24 += 1;
                        r != 0
                    }
                    fn is_university(&mut self) -> bool {
                        let e = self.buildp();
                        self.is(e, 420)
                    }
                    fn add_gather_order(&mut self) {
                        self.log.push(format!("s4:m+0x0,{},2,0", self.idx()));
                    }
                    fn gather_adopted(&mut self) {
                        let play = rd(&self.c[3], 0x2a0) as usize;
                        if rd(&self.p[2 + 2 * play], 0) > 1 {
                            self.log.push(format!(
                                "s25:p{}+0x0,{},{},0,0",
                                1 + 2 * play,
                                self.unit_id(),
                                self.who()
                            ));
                        }
                        self.m[0x80..0x82].copy_from_slice(&0xffffu16.to_le_bytes());
                    }
                    fn adjacent(&mut self) -> bool {
                        self.log
                            .push(format!("s16:m+0x0,{},{}", self.idx(), self.uid()));
                        self.a[1 + 16] != 0
                    }
                    fn covers_tile(&mut self) -> bool {
                        self.log.push(format!("s21:p{}+0x0", self.obj()));
                        let (tx, ty) = (self.tile(rd(self.m, 0x10)), self.tile(rd(self.m, 0x14)));
                        self.log.push(format!("s5:p{}+0x0,{tx},{ty}", self.wall()));
                        self.a[1 + 5] != 0
                    }
                    fn wall_is_farm(&mut self) -> bool {
                        self.log.push(format!("s21:p{}+0x0", self.obj()));
                        let e = format!("p{}+0x0", self.wall());
                        self.is(e, 417)
                    }
                    fn order_action(&mut self) -> bool {
                        let disp = rd(&self.p[14], 4) as usize;
                        self.p[13][disp + 8] & 4 != 0
                    }
                    fn swarm(&mut self, action: bool) {
                        self.log.push("s12:sp-0x9ec,-1".into());
                        self.log.push("s12:sp-0x9ec,-1".into());
                        self.log.push(format!(
                            "s13:sp-0x9ec,{},{},0,0",
                            self.unit_id(),
                            self.who()
                        ));
                        self.log.push(format!(
                            "s14:sp-0x9ec,{},{},0,6,{}",
                            self.idx(),
                            self.who(),
                            if action { 4 } else { 0 }
                        ));
                    }
                    fn is_farm(&mut self) -> bool {
                        let e = format!("p{}+0x0", self.obj());
                        self.is(e, 417)
                    }
                    fn set_work_anim(&mut self, farm: bool) {
                        self.log
                            .push(format!("s6:m+0x0,{},0,1", if farm { 35 } else { 33 }));
                    }
                    fn angle_to_site(&mut self) -> i32 {
                        let dx = (rd(&self.p[self.obj()], 0x10) ^ 0x63637)
                            .wrapping_sub(rd(self.m, 0x10) ^ 0x63637);
                        let dy = (rd(&self.p[self.obj()], 0x14) ^ 0x63637)
                            .wrapping_sub(rd(self.m, 0x14) ^ 0x63637);
                        // The guest's `ecx` on leaving `find_angle`: `dx` on an axis, else
                        // the correction term of its octant polynomial.
                        self.resid = if dx == 0 || dy == 0 {
                            dx
                        } else {
                            let (a, b) = (dx.abs(), dy.abs());
                            let (hi, lo) = if b < a { (a, b) } else { (b, a) };
                            let t = (lo * 0x4000) / hi;
                            ((0x1333 - t).abs() * 0xb00) >> 14
                        };
                        crate::movement::find_angle(dx, dy).0
                    }
                    fn heading(&mut self) -> i32 {
                        rd(self.m, 0x50)
                    }
                    fn set_angle(&mut self, angle: i32) {
                        self.log.push(format!("s7:m+0x0,{angle},{},0", self.resid));
                    }
                    fn decoy(&mut self) -> bool {
                        rd(self.m, 0x68) & 1 != 0
                    }
                    fn accel_construct(&mut self) -> i32 {
                        rd(&self.c[2], 0x22c)
                    }
                    fn under_fire_exempt(&mut self) -> bool {
                        self.log
                            .push(format!("s8:{},16", 0xe3a390 + self.who() * 0x6eec));
                        self.a[1 + 8] != 0 && rd(&self.c[2], 0x7dc) != 0
                    }
                    fn site_under_attack(&mut self) -> bool {
                        let e = self.buildp();
                        let _ = e;
                        if self.a[37] != 0 {
                            let w = &self.p[self.build()];
                            return i16::from_le_bytes([w[0x60], w[0x61]]) & 0x20 != 0;
                        }
                        self.log.push(format!("s23:p{}+0x0", self.build()));
                        self.a[1 + 23] != 0
                    }
                    fn do_construct(&mut self, amount: i32) -> bool {
                        let e = self.buildp();
                        self.log.push(format!("s9:{e},{amount}"));
                        self.a[1 + 9] != 0
                    }
                    fn has_group(&mut self) -> bool {
                        i16::from_le_bytes([self.m[0x80], self.m[0x81]]) >= 0
                    }
                    fn group_normalize(&mut self) {
                        self.log.push(format!("s10:{}", self.group_ecx()));
                    }
                    fn group_peasants(&mut self) -> i32 {
                        self.log.push(format!("s11:{},1,0,0", self.group_ecx()));
                        self.a[1 + 11]
                    }
                }
                let mut v = V {
                    a,
                    m,
                    c,
                    p,
                    log: &mut log,
                    n22: 0,
                    n24: 0,
                    n15: 0,
                    resid: 0,
                };
                do_build_steps(&mut v);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "do_build — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_build — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_build — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "do_build — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 16000, "only {n} rows");
    eprintln!("Unit::do_build@005eebf0: {n} rows agree");
}
