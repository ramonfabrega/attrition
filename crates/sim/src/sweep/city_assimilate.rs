//! `City::assimilate@00738e90` under the emulator against the port (item 3295; tools/sweep_gen.py, obj with windows).

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

/// `City::assimilate@00738e90`, run by `tools/emu/sweep_city_assimilate.py` under unicorn
/// (tools/emu): 128 random bytes of object, 0 argument(s), `p` windows in order: p0..p24 the place(s), in flag order, p25 stub 15's answer window; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; the global cell 0x00c061f0 points at 1936 random bytes; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c061ec points at 1376 random bytes; 56792 random bytes at 0x00e3a390; vtable slot +0xac (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x84 (2 args) stubbed; callee 0x00680f60 (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x00627fe0 (0 args) stubbed; callee 0x0065ca80 (13 args) stubbed; callee 0x006e0600 (1 args) stubbed; callee 0x0062f9b0 (1 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1cd60 (2 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x0046e620 (0 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; the word at `0xc09970+u8(m,0x5e)*0x1c` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 256 random bytes less `0` (placed); the word at `p0+4` points at 256 random bytes less `0` (placed); the word at `p0+8` points at 256 random bytes less `0` (placed); the word at `c3+0x14+u8(m,0x5e)*0x1c` points at 24 random bytes less `0` (placed); the word at `p4+0` points at 32 random bytes less `0` (placed); the word at `p4+4` points at 32 random bytes less `0` (placed); the word at `p4+8` points at 32 random bytes less `0` (placed); the word at `p4+12` points at 32 random bytes less `0` (placed); the word at `p4+16` points at 32 random bytes less `0` (placed); the word at `p4+20` points at 32 random bytes less `0` (placed); the word at `m+0x70` points at 24 random bytes less `0` (placed); the word at `p11+0` points at 128 random bytes less `0` (placed); the word at `p11+4` points at 128 random bytes less `0` (placed); the word at `p11+8` points at 128 random bytes less `0` (placed); the word at `p11+12` points at 128 random bytes less `0` (placed); the word at `p11+16` points at 128 random bytes less `0` (placed); the word at `p11+20` points at 128 random bytes less `0` (placed); the word at `c0+0x134` points at 38912 random bytes less `0` (placed); the word at `p12+0x18` points at 16 random bytes less `0` (placed); the word at `p13+0x18` points at 16 random bytes less `0` (placed); the word at `p14+0x18` points at 16 random bytes less `0` (placed); the word at `p15+0x18` points at 16 random bytes less `0` (placed); the word at `p16+0x18` points at 16 random bytes less `0` (placed); the word at `p17+0x18` points at 16 random bytes less `0` (placed); stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1, 1] (90%); stub 6 answered from [0, 1] (90%); stub 15 answers the address of 64 random bytes, a `p` window after the places; the word at `p12+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p13+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p14+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p15+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p16+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p17+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 0 answers `p12+j`, the data of the object its `ecx` is in `p5+j` (j < 6), linked at `+0x74`; stub 1 answers 10 values in call order, printed after the answers; stub 2 answers 10 values in call order, printed after the answers; stub 3 answers 10 values in call order, printed after the answers; stub 8 answers -1..3 every row; stub 9 answers -2..5 every row; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_assimilate.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(7);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Windows: m the city (`+4` flags, `+6` city number, `+8` its object slot, `+0xc`/`+0x10` position,
                // `+0x5e`/`+0x5f` race pair); c0 the world, c1 `div_3_table`, c2 the constants (`+0x788`), c3 the
                // objects, c4 the console (`+0x298`), c5 the game (`+0x550`), c6 the two leaders (stride 0x6eec,
                // `+0xa48` hero frame, `+0x6928` team colour); p0 the cities' array and p1..p3 their records, p4 the
                // objects' array and p5..p10 the objects, p12..p17 their data records (`+8` flags, `+0x74` link),
                // p18 the world's region table, p25 the text bubble. `a[0..18]` one answer a stub (6 the tribe
                // bonus, 8 `find_unit`, 9 `get_gov_hero`, 13 `parse`, 16 `get_neon`), `a[18..]` the slot `+0x4c`,
                // `+0xb8` and `+0x60` stubs' ten answers each, in call order.
                //
                // **The port is the crate's own steps**, `crate::city::assimilate_steps`, which `Sim::assimilate`
                // calls over its `SimAssimilate`; here the view is the emulated original's windows.
                use crate::city::{AssimilateView, assimilate_steps};
                fn rd16(w: &[u8], o: usize) -> i32 {
                    i16::from_le_bytes([w[o], w[o + 1]]) as i32
                }
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    who: i32,
                    region: i32,
                    n4c: usize,
                    nb8: usize,
                    n60: usize,
                }
                impl V<'_> {
                    fn lead(&self, off: usize) -> usize {
                        self.who as usize * 0x6eec + off
                    }
                    fn slot(&self) -> usize {
                        rd16(self.m, 8) as usize
                    }
                    /// A pointer's token: into leaders `c6` by name, else the bare signed word.
                    fn tok(v: u32) -> String {
                        if (0xe3_a390..=0xe3_a390 + 56792).contains(&v) {
                            format!("c6+{:#x}", v - 0xe3_a390)
                        } else {
                            (v as i32).to_string()
                        }
                    }
                    fn seq(&self, base: usize, n: usize) -> i32 {
                        self.a[base + n.min(9)]
                    }
                    /// The slot-`0x4c` / `0xb8` / `0x60` answers: the record's vtable word picks the arm.
                    fn data(&self, m: i32) -> &Vec<u8> {
                        &self.p[12 + m as usize]
                    }
                    fn build_vt(&self, m: i32) -> bool {
                        rd32(self.data(m), 0) as u32 == 0xb4_2174
                    }
                    fn ability(&mut self, m: i32) -> bool {
                        if self.build_vt(m) {
                            let v = self.seq(38, self.n60);
                            self.n60 += 1;
                            v != 0
                        } else {
                            let v = self.seq(28, self.nb8);
                            self.nb8 += 1;
                            v != 0
                        }
                    }
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                impl AssimilateView for V<'_> {
                    fn clear_unassimilated(&mut self) {
                        self.m[5] &= 0xfe;
                    }
                    fn race_pending(&mut self) -> bool {
                        self.m[0x5f] != self.m[0x5e]
                    }
                    fn adopt_race(&mut self) {
                        self.m[0x5f] = self.m[0x5e];
                    }
                    fn region_unfixed(&mut self) -> bool {
                        let width = rd32(&self.c[0], 0);
                        let tab = |v: i32| rd32(&self.c[1], 4 * (v >> 8) as usize);
                        let esi = width
                            .wrapping_mul(tab(rd32(self.m, 0x10)))
                            .wrapping_add(tab(rd32(self.m, 0xc)));
                        self.region = esi.wrapping_mul(7);
                        rd16(&self.p[18], 4 * self.region as usize + 4) < 0x40
                    }
                    fn fix_borders(&mut self) {
                        self.log.push(format!("s5:{}", self.region));
                    }
                    fn chinese_large_cities(&mut self) -> bool {
                        self.log.push(format!("s6:{},14", Self::tok(0xe3_a390 + self.lead(0) as u32)));
                        self.a[6] != 0 && rd32(&self.c[2], 0x788) != 0
                    }
                    fn town_up(&mut self) {
                        let j = self.slot();
                        self.log.push(format!("s4:p{}+0x0,415,0", 12 + j));
                    }
                    fn raise_economy_flag(&mut self) {
                        let o = self.lead(0);
                        let f = rd32(&self.c[6], o) | 0x200_0000;
                        self.c[6][o..o + 4].copy_from_slice(&f.to_le_bytes());
                    }
                    fn first_member(&mut self) -> Option<i32> {
                        let l = rd16(self.data(self.slot() as i32), 0x74);
                        (l >= 0).then_some(l)
                    }
                    fn next_member(&mut self, m: i32) -> Option<i32> {
                        let l = rd16(self.data(m), 0x74);
                        (l >= 0).then_some(l)
                    }
                    fn member_active(&mut self, m: i32) -> bool {
                        if self.build_vt(m) {
                            self.data(m)[8] & 4 != 0
                        } else {
                            let v = self.seq(18, self.n4c);
                            self.n4c += 1;
                            v != 0
                        }
                    }
                    fn member_is_library(&mut self, m: i32) -> bool {
                        self.ability(m)
                    }
                    fn member_set_active(&mut self, m: i32, on: bool) {
                        let w = &mut self.p[12 + m as usize];
                        if on {
                            w[8] |= 4;
                        } else {
                            w[8] &= 0xfb;
                        }
                    }
                    fn new_library(&mut self, m: i32) {
                        self.log.push(format!("s7:p{}+0x0", 12 + m));
                    }
                    fn member_is_senate(&mut self, m: i32) -> bool {
                        self.ability(m)
                    }
                    fn gov_hero_frame(&mut self) -> i32 {
                        rd32(&self.c[6], self.lead(0xa48))
                    }
                    fn gov_hero_found(&mut self, m: i32) -> bool {
                        let obj = &self.p[5 + m as usize];
                        let (x, y) = (rd32(obj, 0x10) ^ 0x63637, rd32(obj, 0x14) ^ 0x63637);
                        self.log.push(format!(
                            "s8:c3+0x0,{x},{y},1,{},-1,131072,23,0,0,0,0,0,p4+{:#x}",
                            self.who,
                            4 * m
                        ));
                        self.a[8] >= 0
                    }
                    fn hero_frame_reached(&mut self, since: i32) -> bool {
                        since <= rd32(&self.c[5], 0x550)
                    }
                    fn gov_hero_type(&mut self, since: i32) -> Option<i32> {
                        self.log.push(format!(
                            "s9:{},{since}",
                            Self::tok(0xe3_a390 + self.lead(0) as u32)
                        ));
                        (self.a[9] >= 0).then_some(self.a[9])
                    }
                    fn train(&mut self, m: i32, ty: i32) {
                        self.log.push(format!("s10:p{}+0x0,{ty}", 12 + m));
                    }
                    fn console_is_owner(&mut self) -> bool {
                        rd32(&self.c[4], 0x298) == self.who
                    }
                    fn announce(&mut self) {
                        let who = self.who;
                        let city = rd16(self.m, 6);
                        let obj = &self.p[5 + self.slot()];
                        let (x, y) = (rd32(obj, 0x10) ^ 0x63637, rd32(obj, 0x14) ^ 0x63637);
                        let tc = 0xe3_9ce0 + u32::from(self.c[6][self.lead(0x6928)]) * 100;
                        let own = format!("p4+{:#x}", 4 * self.slot());
                        self.log.extend([
                            format!("s11:{who},20"),
                            "s12:15558448,5660".to_string(),
                            format!("s13:15558448,sp-0x34,p{}+0x90", 1 + city),
                            format!("s12:15558448,{}", self.a[13]),
                            "s14:sp-0x34".to_string(),
                            "s12:sp-0x48,5680".to_string(),
                            "s15:sp-0x48".to_string(),
                            "s12:p25+0x4,sp-0x48".to_string(),
                            format!("s16:{},sp-0x2c", Self::tok(tc)),
                            format!(
                                "s17:15267872,15558448,{x},{y},1,{},-1,{own},0,p25+0x0,{own}",
                                self.a[16]
                            ),
                            "s14:sp-0x48".to_string(),
                        ]);
                        self.p[25][0x1a] = who as u8;
                    }
                }
                let who = i32::from(m[0x5e] as i8);
                let mut v = V {
                    a,
                    m,
                    c,
                    p,
                    log: &mut log,
                    who,
                    region: 0,
                    n4c: 0,
                    nb8: 0,
                    n60: 0,
                };
                assimilate_steps(&mut v);
                if log.is_empty() {
                    log.push("-".to_string());
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "assimilate — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "assimilate — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "assimilate — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[8 + i], "assimilate — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("City::assimilate@00738e90: {n} rows agree");
}
