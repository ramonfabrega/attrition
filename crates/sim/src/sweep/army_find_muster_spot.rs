//! `Army::find_muster_spot@006f5cc0` under the emulator against the port (item 3348; tools/sweep_gen.py, obj with windows).

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

/// `Army::find_muster_spot@006f5cc0`, run by `tools/emu/sweep_army_find_muster_spot.py` under unicorn
/// (tools/emu): 152 random bytes of object, 3 argument(s), `p` windows in order: p0..p11 sparse table 0 (4 rows × 3 slice(s), row-major), p12..p15 sparse table 1 (4 rows × 1 slice(s), row-major), p16..p102 the place(s), in flag order, p103..p126 the stubs' answer window(s); the global cell 0x00c06188 points at 312 random bytes; the global cell 0x00c0618c points at 528 random bytes; 4 random bytes at 0x00c09704; 4 random bytes at 0x00c09720; 4 random bytes at 0x00c0973c; 4 random bytes at 0x00c09758; vtable slot +0xac (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; callee 0x0062d470 (0 args) stubbed; callee 0x00738410 (0 args) stubbed; callee 0x006f9120 (2 args) stubbed; callee 0x0065ca80 (13 args) stubbed; callee 0x0065d260 (9 args) stubbed; callee 0x0092d130 (0 args) stubbed; the word at `c1+0x14+0x1c*0` points at 20 random bytes less `0` (placed); the word at `p16+0` points at 48 random bytes less `0` (placed); the word at `p16+4` points at 48 random bytes less `0` (placed); the word at `p16+8` points at 48 random bytes less `0` (placed); the word at `p16+12` points at 48 random bytes less `0` (placed); the word at `p16+16` points at 48 random bytes less `0` (placed); the word at `c1+0x14+0x1c*1` points at 20 random bytes less `0` (placed); the word at `p22+0` points at 48 random bytes less `0` (placed); the word at `p22+4` points at 48 random bytes less `0` (placed); the word at `p22+8` points at 48 random bytes less `0` (placed); the word at `p22+12` points at 48 random bytes less `0` (placed); the word at `p22+16` points at 48 random bytes less `0` (placed); the word at `c1+0x14+0x1c*2` points at 20 random bytes less `0` (placed); the word at `p28+0` points at 48 random bytes less `0` (placed); the word at `p28+4` points at 48 random bytes less `0` (placed); the word at `p28+8` points at 48 random bytes less `0` (placed); the word at `p28+12` points at 48 random bytes less `0` (placed); the word at `p28+16` points at 48 random bytes less `0` (placed); the word at `c1+0x14+0x1c*3` points at 20 random bytes less `0` (placed); the word at `p34+0` points at 48 random bytes less `0` (placed); the word at `p34+4` points at 48 random bytes less `0` (placed); the word at `p34+8` points at 48 random bytes less `0` (placed); the word at `p34+12` points at 48 random bytes less `0` (placed); the word at `p34+16` points at 48 random bytes less `0` (placed); the word at `c1+0x14+0x1c*4` points at 20 random bytes less `0` (placed); the word at `p40+0` points at 48 random bytes less `0` (placed); the word at `p40+4` points at 48 random bytes less `0` (placed); the word at `p40+8` points at 48 random bytes less `0` (placed); the word at `p40+12` points at 48 random bytes less `0` (placed); the word at `p40+16` points at 48 random bytes less `0` (placed); the word at `0xc09970+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p46+0` points at 112 random bytes less `0` (placed); the word at `p46+4` points at 112 random bytes less `0` (placed); the word at `p46+8` points at 112 random bytes less `0` (placed); the word at `p46+12` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p51+0` points at 112 random bytes less `0` (placed); the word at `p51+4` points at 112 random bytes less `0` (placed); the word at `p51+8` points at 112 random bytes less `0` (placed); the word at `p51+12` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*2` points at 16 random bytes less `0` (placed); the word at `p56+0` points at 112 random bytes less `0` (placed); the word at `p56+4` points at 112 random bytes less `0` (placed); the word at `p56+8` points at 112 random bytes less `0` (placed); the word at `p56+12` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*3` points at 16 random bytes less `0` (placed); the word at `p61+0` points at 112 random bytes less `0` (placed); the word at `p61+4` points at 112 random bytes less `0` (placed); the word at `p61+8` points at 112 random bytes less `0` (placed); the word at `p61+12` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*4` points at 16 random bytes less `0` (placed); the word at `p66+0` points at 112 random bytes less `0` (placed); the word at `p66+4` points at 112 random bytes less `0` (placed); the word at `p66+8` points at 112 random bytes less `0` (placed); the word at `p66+12` points at 112 random bytes less `0` (placed); the word at `0xc09710+0x1c*0` points at 20 random bytes less `0` (placed); the word at `p71+0` points at 80 random bytes less `0` (placed); the word at `p71+4` points at 80 random bytes less `0` (placed); the word at `p71+8` points at 80 random bytes less `0` (placed); the word at `p71+12` points at 80 random bytes less `0` (placed); the word at `p71+16` points at 80 random bytes less `0` (placed); the word at `0xc09710+0x1c*1` points at 20 random bytes less `0` (placed); the word at `p77+0` points at 80 random bytes less `0` (placed); the word at `p77+4` points at 80 random bytes less `0` (placed); the word at `p77+8` points at 80 random bytes less `0` (placed); the word at `p77+12` points at 80 random bytes less `0` (placed); the word at `p77+16` points at 80 random bytes less `0` (placed); the word at `0xc09710+0x1c*2` points at 20 random bytes less `0` (placed); the word at `p83+0` points at 80 random bytes less `0` (placed); the word at `p83+4` points at 80 random bytes less `0` (placed); the word at `p83+8` points at 80 random bytes less `0` (placed); the word at `p83+12` points at 80 random bytes less `0` (placed); the word at `p83+16` points at 80 random bytes less `0` (placed); the word at `0xc09710+0x1c*3` points at 20 random bytes less `0` (placed); the word at `p89+0` points at 80 random bytes less `0` (placed); the word at `p89+4` points at 80 random bytes less `0` (placed); the word at `p89+8` points at 80 random bytes less `0` (placed); the word at `p89+12` points at 80 random bytes less `0` (placed); the word at `p89+16` points at 80 random bytes less `0` (placed); the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p95+0` points at 16 random bytes less `0` (placed); the word at `p95+4` points at 16 random bytes less `0` (placed); the word at `p95+8` points at 16 random bytes less `0` (placed); the word at `p95+12` points at 16 random bytes less `0` (placed); the word at `c0+0x134` points at 12288 random bytes less `0` (placed); the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); the word at `0xe3a380` points at 2624 random bytes less `0` (placed); stub 2 answered from [0, 1] (90%); stub 0 answers a window of its own to each call, in call order (128 bytes, ...); the stubbed vtable laid at `m+0x90`; stub 4 answers 10 values in call order, printed after the answers; stub 5 answers 10 values in call order, printed after the answers; stub 6 answers 10 values in call order, printed after the answers; stub 7 answers 3 values in call order, printed after the answers; stub 1 answers 0..4096 every row; stub 3 answers 0..24 every row; stub 4 answers 0..2 every row; stub 5 answers -4..4 every row; stub 6 answers -4..4 every row; stub 7 answers -1000000..1000000 every row; stub 0 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; `0x006817f0` run once before the rows (a table builder the startup would have run); 4 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x74:0x10 (`p` windows after the pointed buffers, before the places); 4 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x10 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p43+0` as well as at `this`; the stubbed vtable laid at `p44+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p96+0` as well as at `this`; the stubbed vtable laid at `p97+0` as well as at `this`; the stubbed vtable laid at `p98+0` as well as at `this`; the stubbed vtable laid at `p99+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_find_muster_spot.py") else {
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
                // Windows: `m` the army (`+2` slot, `+8` region, `+0x24` navy, `+0x2c` hurry, `+0x38`/`+0x3c` the point,
                // `+0x48`/`+0x4c` the muster, `+0x50` its angle, `+0x94` the leader); `c[0]` the world (`+0` width, `+4`
                // height), `c[1]` the objects' singleton (`+0x200` the row a search lies in), `c[2+who]` the leader's army
                // count; `p[3r..3r+3]` leader `r`'s `+0`, `+8`, `+0x74` slices and `p[12+r]` the second table's diplomacy
                // row; `p[16+6r]` objects row `r`'s array and `p[17+6r+k]` its records; `p[46+5r]` cities row `r`'s array and
                // `p[47+5r+k]` its records; `p[71+6r]` armies row `r`'s array and `p[72+6r+k]` its records; `p[95]` the image's
                // object row 0 and `p[96+k]` its records; `p[100]` the world's cells (0x1c apart), `p[101]` the
                // `div_3_table`, `p[102]` the lands; `p[103+n]` the type record the `n`th `vtable[0xac]` call answered.
                // Answers: `a[3..11]` one a stub (1 the `vtable[0x3c]` answer, 2 `is_unassimilated`, 3 `get_radius`),
                // `a[11..21]` `count`'s, `a[21..31]` `find_unit`'s, `a[31..41]` `find_building`'s, `a[41..44]` `find_angle`'s.
                // The recorded calls' `edx` is whatever the caller left, `*`, but for `find_angle`'s, masked below.
                //
                // **The port is the crate's own steps**, `crate::muster::find_muster_spot_steps`, which
                // `Sim::find_muster_spot` calls over its `SimMuster`; here the view is the emulated original's windows.
                use crate::muster::{MusterView, find_muster_spot_steps};
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    types: usize,
                    counts: usize,
                    units: usize,
                    builds: usize,
                    angles: usize,
                    city: i32,
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
                impl V<'_> {
                    fn push(&mut self, k: usize, s: String) {
                        if self.log[k] == "-" {
                            self.log[k] = s;
                        } else {
                            self.log[k].push('/');
                            self.log[k].push_str(&s);
                        }
                    }
                    fn tw_(&self) -> usize {
                        self.a[1] as usize
                    }
                    fn who_(&self) -> usize {
                        rd16(self.m, 0x94) as usize
                    }
                    fn obj(&self, row: usize, i: i32) -> usize {
                        17 + 6 * row + i as usize
                    }
                    /// `div_3_table[(raw ^ 0x63637) >> 8]` of the object's `+off`.
                    fn enc(&self, w: usize, off: usize) -> i32 {
                        rd32(
                            &self.p[101],
                            4 * ((rd32(&self.p[w], off) ^ 0x63637) >> 8) as usize,
                        )
                    }
                    fn raw(&self, w: usize, off: usize) -> i32 {
                        rd32(&self.p[w], off) ^ 0x63637
                    }
                    fn div3(&self, v: i32) -> i32 {
                        rd32(&self.p[101], 4 * (v >> 8) as usize)
                    }
                    /// One `vtable[0xac]` call on object window `w`: its token and the type record's window.
                    fn ty(&mut self, w: usize) -> usize {
                        self.push(0, format!("p{w}+0x0,*"));
                        let t = 103 + self.types.min(23);
                        self.types += 1;
                        t
                    }
                    fn cities(&self, c: i32) -> usize {
                        47 + 5 * self.tw_() + c as usize
                    }
                    fn cell_at(&self, x: i32, y: i32) -> usize {
                        ((rd32(&self.c[0], 0) * y + x) * 0x1c) as usize
                    }
                }
                impl MusterView for V<'_> {
                    fn who(&self) -> i32 {
                        rd16(self.m, 0x94)
                    }
                    fn tw(&self) -> i32 {
                        self.a[1]
                    }
                    fn first(&self) -> i32 {
                        self.a[0]
                    }
                    fn wide(&self) -> bool {
                        self.a[2] != 0
                    }
                    fn navy(&self) -> bool {
                        rd32(self.m, 0x24) != 0
                    }
                    fn slot(&self) -> i32 {
                        rd16(self.m, 2)
                    }
                    fn reg(&self) -> i32 {
                        rd32(self.m, 8)
                    }
                    fn set_hurry(&mut self, h: i32) {
                        wr32(self.m, 0x2c, h);
                    }
                    fn muster(&self) -> (i32, i32) {
                        (rd32(self.m, 0x48), rd32(self.m, 0x4c))
                    }
                    fn set_muster(&mut self, x: i32, y: i32) {
                        wr32(self.m, 0x48, x);
                        wr32(self.m, 0x4c, y);
                    }
                    fn army_cell(&self) -> (i32, i32) {
                        (self.div3(rd32(self.m, 0x38)), self.div3(rd32(self.m, 0x3c)))
                    }
                    fn find_angle(&mut self, dx: i32, dy: i32) -> i32 {
                        self.push(7, format!("{dx},{dy}"));
                        let r = self.a[41 + self.angles.min(2)];
                        self.angles += 1;
                        r
                    }
                    fn set_angle(&mut self, a: i32) {
                        wr32(self.m, 0x50, a);
                    }
                    fn friend(&mut self, a: i32, b: i32) -> bool {
                        let (a, b) = (a as usize, b as usize);
                        let me = rd32(&self.p[3 * a + 1], 0) as usize;
                        b == me
                            || (rd32(&self.p[3 * a + 2], 4 * b) == 2
                                && rd32(&self.p[12 + b], 4 * me) == 2)
                    }
                    fn transports(&self) -> bool {
                        rd32(&self.p[3 * self.who_()], 0) & 0x700 != 0
                    }
                    fn world_cells(&self) -> (i32, i32) {
                        (rd32(&self.c[0], 0), rd32(&self.c[0], 4))
                    }
                    fn elem_damaged(&mut self, i: i32) -> bool {
                        let w = self.obj(self.tw_(), i);
                        let t = self.ty(w);
                        rd32(&self.p[t], 0x24) != 0
                    }
                    fn elem_alive(&mut self, i: i32) -> bool {
                        let w = self.obj(self.tw_(), i);
                        let t = self.ty(w);
                        self.p[t][0x60] & 0x20 != 0
                    }
                    fn elem_next(&mut self, i: i32) -> i32 {
                        let w = self.obj(self.tw_(), i);
                        let t = self.ty(w);
                        rd16(&self.p[t], 0x74)
                    }
                    fn elem_cell(&self, i: i32) -> (i32, i32) {
                        let w = self.obj(self.tw_(), i);
                        (self.enc(w, 0x10), self.enc(w, 0x14))
                    }
                    fn find_combat(&mut self, i: i32) -> i32 {
                        let w = self.obj(self.tw_(), i);
                        let (x, y, who) = (self.raw(w, 0x10), self.raw(w, 0x14), self.who());
                        self.push(
                            5,
                            format!("c1+0x0,*,{x},{y},3,{who},2304,512,9,0,0,13,0,0,c1+0x0"),
                        );
                        let r = self.a[21 + self.units.min(9)];
                        self.units += 1;
                        r
                    }
                    fn find_target(&mut self, i: i32) -> i32 {
                        let w = self.obj(self.tw_(), i);
                        let (x, y, who, tw) =
                            (self.raw(w, 0x10), self.raw(w, 0x14), self.who(), self.tw());
                        self.push(
                            5,
                            format!("c1+0x0,*,{x},{y},3,{who},4608,512,11,{i},{tw},13,0,0,c1+0x0"),
                        );
                        let r = self.a[21 + self.units.min(9)];
                        self.units += 1;
                        r
                    }
                    fn find_building(&mut self, i: i32) -> i32 {
                        let w = self.obj(self.tw_(), i);
                        let (x, y, who) = (self.raw(w, 0x10), self.raw(w, 0x14), self.who());
                        self.push(6, format!("c1+0x0,*,{x},{y},3,{who},4608,512,9,0,0"));
                        let r = self.a[31 + self.builds.min(9)];
                        self.builds += 1;
                        r
                    }
                    fn siege(&mut self) -> i32 {
                        self.push(4, "m+0x0,*,4,0".to_string());
                        let r = self.a[11 + self.counts.min(9)];
                        self.counts += 1;
                        r
                    }
                    fn found_row_ok(&self) -> bool {
                        rd32(&self.c[1], 0x200) >= 0
                    }
                    fn found_cell(&self, f: i32) -> (i32, i32) {
                        let w = self.obj(rd32(&self.c[1], 0x200) as usize, f);
                        (self.enc(w, 0x10), self.enc(w, 0x14))
                    }
                    fn target_type_city(&mut self) -> bool {
                        let w = self.obj(self.tw_(), self.a[0]);
                        let t = self.ty(w);
                        self.p[t][8] & 0x20 != 0
                    }
                    fn city_unassimilated(&mut self) -> bool {
                        let w = self.obj(self.tw_(), self.a[0]);
                        let t = self.ty(w);
                        self.city = rd16(&self.p[t], 0x72);
                        let rec = self.cities(self.city);
                        assert_eq!(
                            self.p[rec][0x5e], 0,
                            "the objects table's second row is not laid"
                        );
                        let b = rd16(&self.p[rec], 8);
                        self.push(1, format!("p{}+0x0,*", 96 + b));
                        self.push(2, format!("{},*", self.a[4]));
                        self.a[5] != 0
                    }
                    fn city_radius(&mut self) -> i32 {
                        let rec = self.cities(self.city);
                        self.push(3, format!("p{rec}+0x0,*"));
                        self.a[6]
                    }
                    fn target_is_city(&self) -> bool {
                        let w = self.obj(self.tw_(), self.a[0]);
                        self.p[w][8] & 0x20 != 0
                    }
                    fn target_city(&mut self) -> i32 {
                        let w = self.obj(self.tw_(), self.a[0]);
                        let t = self.ty(w);
                        rd16(&self.p[t], 0x72)
                    }
                    fn city_flags(&self, c: i32) -> i32 {
                        let rec = self.cities(c);
                        i32::from(u16::from_le_bytes([self.p[rec][4], self.p[rec][5]]))
                    }
                    fn set_city_flags(&mut self, c: i32, f: i32) {
                        let rec = self.cities(c);
                        self.p[rec][4..6].copy_from_slice(&(f as u16).to_le_bytes());
                    }
                    fn city_cell(&self, c: i32) -> (i32, i32) {
                        let rec = self.cities(c);
                        (
                            self.div3(rd32(&self.p[rec], 0xc)),
                            self.div3(rd32(&self.p[rec], 0x10)),
                        )
                    }
                    fn armies(&self) -> i32 {
                        rd32(&self.c[2 + self.who_()], 0)
                    }
                    fn army_valid(&self, j: i32) -> bool {
                        rd16(&self.p[72 + 6 * self.who_() + j as usize], 0) != 0
                    }
                    fn army_muster(&self, j: i32) -> (i32, i32) {
                        let w = 72 + 6 * self.who_() + j as usize;
                        (rd32(&self.p[w], 0x48), rd32(&self.p[w], 0x4c))
                    }
                    fn cell_flags(&self, x: i32, y: i32) -> i32 {
                        let o = self.cell_at(x, y);
                        i32::from(u16::from_le_bytes([self.p[100][o], self.p[100][o + 1]]))
                    }
                    fn cell_land(&self, x: i32, y: i32) -> i32 {
                        i32::from(self.p[100][self.cell_at(x, y) + 2] as i8)
                    }
                    fn cell_region(&self, x: i32, y: i32) -> i32 {
                        rd16(&self.p[100], self.cell_at(x, y) + 4)
                    }
                    fn cell_owner(&self, x: i32, y: i32) -> i32 {
                        i32::from(self.p[100][self.cell_at(x, y) + 0xf] as i8)
                    }
                    fn move_rate(&self, class: i32) -> i32 {
                        rd32(&self.p[102], class as usize * 0x138 + 0x100)
                    }
                }
                log.extend(std::iter::repeat_n("-".to_string(), 8));
                let mut v = V {
                    a,
                    m,
                    c,
                    p,
                    log: &mut log,
                    types: 0,
                    counts: 0,
                    units: 0,
                    builds: 0,
                    angles: 0,
                    city: 0,
                };
                i64::from(find_muster_spot_steps(&mut v))
            }
        };
        assert_eq!(got, want_eax, "find_muster_spot — eax, row `{line}`");
        // The `edx` of a callee that is not handed one is the stack's and not the function's (every stub but
        // `find_angle`), so both sides are masked.
        let mask = |logs: &[&str]| -> Vec<String> {
            logs.iter()
                .enumerate()
                .map(|(k, col)| {
                    col.split('/')
                        .map(|call| {
                            let mut f: Vec<&str> = call.split(',').collect();
                            if k != 7 && f.len() > 1 {
                                f[1] = "*";
                            }
                            f.join(",")
                        })
                        .collect::<Vec<_>>()
                        .join("/")
                })
                .collect()
        };
        let mine: Vec<&str> = log.iter().map(String::as_str).collect();
        assert_eq!(
            mask(&mine),
            mask(&want_log),
            "find_muster_spot — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_muster_spot — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_muster_spot — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[7 + i],
                "find_muster_spot — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Army::find_muster_spot@006f5cc0: {n} rows agree");
}
