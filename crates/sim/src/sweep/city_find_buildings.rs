//! `City::find_buildings@007384c0` under the emulator against the port (item 3304; tools/sweep_gen.py, obj with windows).

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

/// `City::find_buildings@007384c0`, run by `tools/emu/sweep_city_find_buildings.py` under unicorn
/// (tools/emu): 128 random bytes of object, 0 argument(s), `p` windows in order: p0..p7 sparse table 0 (8 rows × 1 slice(s), row-major), p8..p73 the place(s), in flag order, p74 stub 9's answer window; the global cell 0x00c0618c points at 504 random bytes; the global cell 0x00c06198 points at 16 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c06210 points at 672 random bytes; vtable slot +0xac (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x006db790 (1 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x00627fe0 (0 args) stubbed; callee 0x00622380 (1 args) stubbed; vtable slot +0x94 (1 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0x10c (1 args) stubbed; callee 0x00a1cd60 (2 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; vtable slot +0x1a8 (3 args) stubbed; vtable slot +0x150 (3 args) stubbed; callee 0x00642fc0 (2 args) stubbed; callee 0x00738b20 (0 args) stubbed; callee 0x00647080 (3 args) stubbed; vtable slot +0x158 (3 args) stubbed; the word at `c0+0x14+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 36 random bytes less `0` (placed); the word at `p8+4` points at 36 random bytes less `0` (placed); the word at `p8+8` points at 36 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p12+0` points at 36 random bytes less `0` (placed); the word at `p12+4` points at 36 random bytes less `0` (placed); the word at `p12+8` points at 36 random bytes less `0` (placed); the word at `c0+0x14+0x1c*2` points at 12 random bytes less `0` (placed); the word at `p16+0` points at 36 random bytes less `0` (placed); the word at `p16+4` points at 36 random bytes less `0` (placed); the word at `p16+8` points at 36 random bytes less `0` (placed); the word at `c0+0x14+0x1c*3` points at 12 random bytes less `0` (placed); the word at `p20+0` points at 36 random bytes less `0` (placed); the word at `p20+4` points at 36 random bytes less `0` (placed); the word at `p20+8` points at 36 random bytes less `0` (placed); the word at `c0+0x14+0x1c*4` points at 12 random bytes less `0` (placed); the word at `p24+0` points at 36 random bytes less `0` (placed); the word at `p24+4` points at 36 random bytes less `0` (placed); the word at `p24+8` points at 36 random bytes less `0` (placed); the word at `c0+0x14+0x1c*5` points at 12 random bytes less `0` (placed); the word at `p28+0` points at 36 random bytes less `0` (placed); the word at `p28+4` points at 36 random bytes less `0` (placed); the word at `p28+8` points at 36 random bytes less `0` (placed); the word at `c0+0x14+0x1c*6` points at 12 random bytes less `0` (placed); the word at `p32+0` points at 36 random bytes less `0` (placed); the word at `p32+4` points at 36 random bytes less `0` (placed); the word at `p32+8` points at 36 random bytes less `0` (placed); the word at `c0+0x14+0x1c*7` points at 12 random bytes less `0` (placed); the word at `p36+0` points at 36 random bytes less `0` (placed); the word at `p36+4` points at 36 random bytes less `0` (placed); the word at `p36+8` points at 36 random bytes less `0` (placed); the word at `0xc0ab84+u8(m,0x5e)*0x1c` points at 12 random bytes less `0` (placed); the word at `p40+0` points at 32 random bytes less `0` (placed); the word at `p40+4` points at 32 random bytes less `0` (placed); the word at `p40+8` points at 32 random bytes less `0` (placed); the word at `p9+0x20` points at 128 random bytes less `0` (placed); the word at `p10+0x20` points at 128 random bytes less `0` (placed); the word at `p11+0x20` points at 128 random bytes less `0` (placed); the word at `p13+0x20` points at 128 random bytes less `0` (placed); the word at `p14+0x20` points at 128 random bytes less `0` (placed); the word at `p15+0x20` points at 128 random bytes less `0` (placed); the word at `p17+0x20` points at 128 random bytes less `0` (placed); the word at `p18+0x20` points at 128 random bytes less `0` (placed); the word at `p19+0x20` points at 128 random bytes less `0` (placed); the word at `p21+0x20` points at 128 random bytes less `0` (placed); the word at `p22+0x20` points at 128 random bytes less `0` (placed); the word at `p23+0x20` points at 128 random bytes less `0` (placed); the word at `p25+0x20` points at 128 random bytes less `0` (placed); the word at `p26+0x20` points at 128 random bytes less `0` (placed); the word at `p27+0x20` points at 128 random bytes less `0` (placed); the word at `p29+0x20` points at 128 random bytes less `0` (placed); the word at `p30+0x20` points at 128 random bytes less `0` (placed); the word at `p31+0x20` points at 128 random bytes less `0` (placed); the word at `p33+0x20` points at 128 random bytes less `0` (placed); the word at `p34+0x20` points at 128 random bytes less `0` (placed); the word at `p35+0x20` points at 128 random bytes less `0` (placed); the word at `p37+0x20` points at 128 random bytes less `0` (placed); the word at `p38+0x20` points at 128 random bytes less `0` (placed); the word at `p39+0x20` points at 128 random bytes less `0` (placed); the word at `m+0x70` points at 712 random bytes less `0` (placed); the word at `m+0x74` points at 712 random bytes less `0` (placed); the word at `m+0x78` points at 712 random bytes less `0` (placed); the word at `m+0x7c` points at 712 random bytes less `0` (placed); the word at `c0+0x1ec` points at 32 random bytes less `0` (placed); the word at `c0+0x1f0` points at 32 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [0, 1, 2, 3, 5, 8, 10, 15, 20, 30, 40, 63, 64, 65, 100, 64, 65, 66, 100, 100] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 9 answers the address of 128 random bytes, a `p` window after the places; the word at `p44+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p45+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p46+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p47+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p48+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p49+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p50+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p51+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p52+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p53+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p54+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p55+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p56+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p57+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p58+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p59+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p60+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p61+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p62+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p63+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p64+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p65+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p66+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p67+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p44+0x18` laid as the address of `p68` every row; the word at `p45+0x18` laid as the address of `p69` every row; the word at `p46+0x18` laid as the address of `p70` every row; the word at `p47+0x18` laid as the address of `p71` every row; the word at `p48+0x18` laid as the address of `p68` every row; the word at `p49+0x18` laid as the address of `p69` every row; the word at `p50+0x18` laid as the address of `p70` every row; the word at `p51+0x18` laid as the address of `p71` every row; the word at `p52+0x18` laid as the address of `p68` every row; the word at `p53+0x18` laid as the address of `p69` every row; the word at `p54+0x18` laid as the address of `p70` every row; the word at `p55+0x18` laid as the address of `p71` every row; the word at `p56+0x18` laid as the address of `p68` every row; the word at `p57+0x18` laid as the address of `p69` every row; the word at `p58+0x18` laid as the address of `p70` every row; the word at `p59+0x18` laid as the address of `p71` every row; the word at `p60+0x18` laid as the address of `p68` every row; the word at `p61+0x18` laid as the address of `p69` every row; the word at `p62+0x18` laid as the address of `p70` every row; the word at `p63+0x18` laid as the address of `p71` every row; the word at `p64+0x18` laid as the address of `p68` every row; the word at `p65+0x18` laid as the address of `p69` every row; the word at `p66+0x18` laid as the address of `p70` every row; the word at `p67+0x18` laid as the address of `p71` every row; the word at `p41+0x18` laid as the address of `p68` every row; the word at `p42+0x18` laid as the address of `p68` every row; the word at `p43+0x18` laid as the address of `p68` every row; stub 1 answers 12 values in call order, printed after the answers; stub 7 answers 8 values in call order, printed after the answers; stub 7 answers -2..2 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 answers from its own `ecx`: the word at `+0x20`; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p44+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; the stubbed vtable laid at `p48+0` as well as at `this`; the stubbed vtable laid at `p49+0` as well as at `this`; the stubbed vtable laid at `p50+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; the stubbed vtable laid at `p52+0` as well as at `this`; the stubbed vtable laid at `p53+0` as well as at `this`; the stubbed vtable laid at `p54+0` as well as at `this`; the stubbed vtable laid at `p55+0` as well as at `this`; the stubbed vtable laid at `p56+0` as well as at `this`; the stubbed vtable laid at `p57+0` as well as at `this`; the stubbed vtable laid at `p58+0` as well as at `this`; the stubbed vtable laid at `p59+0` as well as at `this`; the stubbed vtable laid at `p60+0` as well as at `this`; the stubbed vtable laid at `p61+0` as well as at `this`; the stubbed vtable laid at `p62+0` as well as at `this`; the stubbed vtable laid at `p63+0` as well as at `this`; the stubbed vtable laid at `p64+0` as well as at `this`; the stubbed vtable laid at `p65+0` as well as at `this`; the stubbed vtable laid at `p66+0` as well as at `this`; the stubbed vtable laid at `p67+0` as well as at `this`; the stubbed vtable laid at `p68+0` as well as at `this`; the stubbed vtable laid at `p69+0` as well as at `this`; the stubbed vtable laid at `p70+0` as well as at `this`; the stubbed vtable laid at `p71+0` as well as at `this`; the stubbed vtable laid at `p74+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_find_buildings.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Windows: `m` the city (`+6` its number, `+8` its object slot, `+0x5e` who); `c0` the objects (`+0x14 + 0x1c *
                // leader` the leader's list, `+0x1ec`/`+0x1f0` the two count arrays), `c1` `obj_base` (`+4`/`+8` the two
                // ranges' first index), `c2` `div_3_table` (unread here: the port takes `Pos::tile`), `c3` the console (`+0x298`
                // who). `p0..p7` the leaders' first words (bit 0 live); leader `w`'s list is `p[8 + 4w]`, its object `j`
                // `p[9 + 4w + j]` (`+8` flags, `+0x10`/`+0x14` position xor `0x63637`, `+0x20` -> its `Build` `p[44 + 3w + j]`
                // (`+0x70` the city short at `+0x72`, `+0x18` -> its type `p[68 + (3w + j) % 4]`, `+0x2c0` flags), `p[41..44]`
                // the city table's records, `p[72]`/`p[73]` the counts, `p[74]` the answer window of slot `+0xb0`.
                // `a[0..20]` one answer a stub (2 `get_radius`, 3 the type's slot `+0x60`, 4 the build's slot `+0xb8`),
                // `a[20..32]` stub 1's (`+0x10`) twelve answers in call order, `a[32..40]` stub 7's `swap_team`.
                //
                // **The port is the crate's own steps**, `crate::city::find_buildings_steps`, which `Sim::find_buildings` runs
                // over its building slots; here the view walks the emulated original's `leaders × two ranges × index`.
                use crate::city::{FindBuildingsView, find_buildings_steps};
                use crate::world::{Pos, vector_dist};
                const KEY: i32 = 0x63637;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                    walk: Vec<(usize, usize)>,
                    at: usize,
                    is_building_n: usize,
                    swaps: usize,
                    swapped: usize,
                }
                impl V<'_> {
                    fn who(&self) -> usize {
                        self.m[0x5e] as i8 as usize
                    }
                    fn w(&self) -> usize {
                        self.walk[self.at].0
                    }
                    fn j(&self) -> usize {
                        self.walk[self.at].1
                    }
                    fn obj(&self) -> usize {
                        9 + 4 * self.w() + self.j()
                    }
                    fn data(&self) -> usize {
                        44 + 3 * self.w() + self.j()
                    }
                    fn ty(&self) -> usize {
                        68 + (3 * self.w() + self.j()) % 4
                    }
                    fn pos(&self, win: usize) -> Pos {
                        Pos::new(rd(&self.p[win], 0x10) ^ KEY, rd(&self.p[win], 0x14) ^ KEY)
                    }
                    fn build(&mut self) {
                        let o = self.obj();
                        self.log.push(format!("s0:p{o}+0x0"));
                    }
                    fn at_obj(&mut self, stub: usize, win: usize, rest: &str) {
                        self.log.push(format!("s{stub}:p{win}+0x0{rest}"));
                    }
                }
                impl FindBuildingsView for V<'_> {
                    fn radius(&mut self) -> i32 {
                        let who = self.who();
                        let ty = rd(&self.p[68], 4);
                        self.log.push(format!("s2:p{who}+0x0,{ty}"));
                        self.a[2]
                    }
                    fn next_object(&mut self) -> bool {
                        self.at = self.at.wrapping_add(1);
                        self.at < self.walk.len()
                    }
                    fn is_building(&mut self) -> bool {
                        let o = self.obj();
                        self.at_obj(1, o, "");
                        let v = self.a[20 + self.is_building_n.min(11)];
                        self.is_building_n += 1;
                        v != 0
                    }
                    fn flagged(&mut self) -> bool {
                        rd(&self.p[self.obj()], 8) & 0x20 != 0
                    }
                    fn cityless(&mut self) -> bool {
                        self.build();
                        ((rd(&self.p[self.data()], 0x70) >> 16) as i16) < 0
                    }
                    fn same_owner(&mut self) -> bool {
                        self.w() == self.who()
                    }
                    fn no_city(&mut self) -> bool {
                        self.build();
                        rd(&self.p[self.ty()], 0x2c0) & 0x10 != 0
                    }
                    fn dist(&mut self) -> i32 {
                        let city = self.pos(
                            9 + 4 * self.who()
                                + i16::from_le_bytes([self.m[8], self.m[9]]) as usize,
                        );
                        let (ct, bt) = (city.tile(), self.pos(self.obj()).tile());
                        vector_dist(ct.x - bt.x, ct.y - bt.y)
                    }
                    fn is_library(&mut self) -> bool {
                        self.build();
                        let d = self.data();
                        if rd(&self.p[d], 0) == 0xb42174 {
                            let t = self.ty();
                            self.at_obj(3, t, ",435,0");
                            self.a[3] != 0
                        } else {
                            self.at_obj(4, d, ",435,0");
                            self.a[4] != 0
                        }
                    }
                    fn new_library(&mut self) {
                        self.build();
                        let d = self.data();
                        self.at_obj(5, d, "");
                    }
                    fn add_to_city(&mut self) {
                        self.build();
                        let d = self.data();
                        let city = i16::from_le_bytes([self.m[6], self.m[7]]);
                        self.at_obj(6, d, &format!(",{city}"));
                    }
                    fn swap_team(&mut self) -> bool {
                        self.build();
                        let d = self.data();
                        let who = self.who();
                        self.at_obj(7, d, &format!(",{who}"));
                        let r = self.a[32 + self.swaps.min(7)];
                        self.swaps += 1;
                        self.swapped = r as usize;
                        r >= 0
                    }
                    fn die_old(&mut self) {
                        self.build();
                        let d = self.data();
                        let stub = if rd(&self.p[d], 0) == 0xb42174 {
                            18
                        } else {
                            19
                        };
                        self.at_obj(stub, d, ",0,-1,0");
                    }
                    fn console_involved(&mut self) -> bool {
                        let cw = rd(&self.c[3], 0x298);
                        self.who() as i32 == cw || self.w() as i32 == cw
                    }
                    fn announce(&mut self) {
                        // The text stubs: `String::operator=` is not recorded (its argument is an image word the
                        // emulator leaves varying); the constants are the executable's own (`message_win` at `0xc06208`,
                        // the `msg` string, `RED`).
                        let o = self.obj();
                        self.at_obj(9, o, "");
                        self.log.push("s10:p74+0x0,sp-0x50".into());
                        self.log.push("s11:15558448,sp-0x64,sp-0x50".into());
                        self.log.push("s12:sp-0x64".into());
                        let q = self.pos(o);
                        let (x, y) = (q.x, q.y);
                        let arr = 8 + 4 * self.w();
                        self.log.push(format!(
                            "s13:15267872,15558448,{x},{y},0,13161032,-1,p{arr}+0x0,0,0,p{arr}+0x0"
                        ));
                        self.log.push("s12:sp-0x50".into());
                    }
                    fn activate_new(&mut self) {
                        let n = 9 + 4 * self.who() + self.swapped;
                        self.at_obj(9, n, "");
                        self.log.push("s14:p74+0x0,0,1,0".into());
                    }
                    fn close_old(&mut self) {
                        self.build();
                        let d = self.data();
                        self.at_obj(15, d, ",0,-1,0");
                    }
                    fn mask_new(&mut self) {
                        let n = 9 + 4 * self.who() + self.swapped;
                        self.at_obj(9, n, "");
                        self.log.push("s16:p74+0x0,1,2".into());
                    }
                    fn check_upgrade(&mut self) {
                        self.log.push("s17:m+0x0".into());
                    }
                }
                // The original's walk: every live leader (bit 0 of its first byte), the two index ranges
                // `[obj_base + 4, count1)` and `[obj_base + 8, count2)`, each over the leader's list.
                let mut walk = Vec::new();
                for (w, lw) in p.iter().enumerate().take(8) {
                    if lw[0] & 1 == 0 {
                        continue;
                    }
                    for (first, counts) in [(rd(&c[1], 4), &p[72]), (rd(&c[1], 8), &p[73])] {
                        let count = rd(counts, 4 * w);
                        let mut j = first;
                        while j < count {
                            walk.push((w, j as usize));
                            j += 1;
                        }
                    }
                }
                let mut v = V {
                    a,
                    m,
                    c,
                    p,
                    log: &mut log,
                    walk,
                    at: usize::MAX,
                    is_building_n: 0,
                    swaps: 0,
                    swapped: 0,
                };
                find_buildings_steps(&mut v);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "find_buildings — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_buildings — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_buildings — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "find_buildings — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("City::find_buildings@007384c0: {n} rows agree");
}
