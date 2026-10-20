//! `SpellTypeData::is_castable@00675bc0` under the emulator against the port (item 3364; tools/sweep_gen.py, obj with windows).

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

/// `SpellTypeData::is_castable@00675bc0`, run by `tools/emu/sweep_spell_type_data_is_castable.py` under unicorn
/// (tools/emu): 512 random bytes of object, 3 argument(s), `p` windows in order: p0..p15 sparse table 0 (8 rows × 2 slice(s), row-major), p16..p27 the place(s), in flag order; the global cell 0x00c0ab10 points at 2672 random bytes; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00c061d4 points at 224 random bytes; the global cell 0x00cae5fc points at 256 random bytes; the global cell 0x00e85ddc points at 288 random bytes; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x110 (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; vtable slot +0x54 (0 args) stubbed; vtable slot +0x68 (1 args) stubbed; callee 0x006d9380 (1 args) stubbed; callee 0x006e30f0 (1 args) stubbed; callee 0x006db510 (4 args) stubbed; callee 0x0062d470 (0 args) stubbed; callee 0x0046ffa0 (0 args) stubbed; callee 0x0065b870 (5 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x00738060 (0 args) stubbed; callee 0x007380a0 (0 args) stubbed; callee 0x006e0c80 (1 args) stubbed; the word at `0xc0ab84+a[1]*0x1c` points at 4 random bytes less `a[0]*4` (placed); the word at `p16+0` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+a[1]*0x1c` points at 4 random bytes less `a[0]*4` (placed); the word at `p18+0` points at 112 random bytes less `0` (placed); the word at `c0+0xa68` points at 512 random bytes less `0` (placed); the word at `c1+0x134` points at 512 random bytes less `0` (placed); the word at `c1+0x138` points at 512 random bytes less `0` (placed); stub 3's answer points at 128 random bytes less `0` (placed); the word at `p23+0x18` points at 720 random bytes less `0` (placed); the word at `c2+0x10+a[1]*0x1c` points at 4 random bytes less `(s[14],i16(p[23],0x72))[i32(m,4)!=0x27e]*4` (placed); the word at `p25+0` points at 112 random bytes less `0` (placed); the word at `c4+0` points at 32 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 9 answered from [0, 1] (90%); stub 11 answered from [0, 1] (90%); stub 12 answered from [0, 1] (90%); stub 13 answered from [0, 1] (90%); stub 15 answered from [0, 1] (90%); stub 16 answered from [0, 1] (90%); stub 17 answered from [0, 1] (90%); stub 18 answered from [0, 1] (90%); the word at `c4+4` laid as the address of `p27` every row; the word at `c4+8` laid as the address of `p27` every row; the word at `c4+12` laid as the address of `p27` every row; the word at `c4+0x108` laid as the address of `p27` every row; the word at `c4+0x10c` laid as the address of `p27` every row; the word at `c4+0x110` laid as the address of `p27` every row; the word at `p17+0x18` laid as the address of `p17` every row; stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 2 values in call order, printed after the answers; stub 4 answers 2 values in call order, printed after the answers; stub 5 answers 2 values in call order, printed after the answers; stub 6 answers 2 values in call order, printed after the answers; stub 7 answers 2 values in call order, printed after the answers; stub 8 answers 5 values in call order, printed after the answers; stub 9 answers 5 values in call order, printed after the answers; stub 10 answers 2 values in call order, printed after the answers; stub 11 answers 2 values in call order, printed after the answers; stub 12 answers 2 values in call order, printed after the answers; stub 13 answers 2 values in call order, printed after the answers; stub 15 answers 2 values in call order, printed after the answers; stub 17 answers 2 values in call order, printed after the answers; stub 18 answers 3 values in call order, printed after the answers; stub 14 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x1, +0x8:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_data_is_castable.py") else {
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
                // The view over the emulated windows. `a`: the three arguments, the 19
                // stubs' answers (from 3), the seq values, then the slot-0xb8 flag (73).
                struct Sw<'a> {
                    a: &'a [i32],
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    pass: usize,
                    turn: [usize; 19],
                    calls: Vec<Vec<String>>,
                }
                impl Sw<'_> {
                    fn ans(&mut self, i: usize) -> i32 {
                        let (base, n) = match i {
                            0 => (22, 8),
                            1 => (30, 8),
                            2 => (38, 2),
                            4 => (40, 2),
                            5 => (42, 2),
                            6 => (44, 2),
                            7 => (46, 2),
                            8 => (48, 5),
                            9 => (53, 5),
                            10 => (58, 2),
                            11 => (60, 2),
                            12 => (62, 2),
                            13 => (64, 2),
                            15 => (66, 2),
                            17 => (68, 2),
                            18 => (70, 3),
                            _ => return self.a[3 + i],
                        };
                        let t = self.turn[i].min(n - 1);
                        self.turn[i] += 1;
                        self.a[base + t]
                    }
                    fn rec_log(&mut self, i: usize, s: String) {
                        self.calls[i].push(s);
                    }
                    fn sp(&self) -> &[u8] {
                        if self.pass == 0 { self.m } else { &self.p[20] }
                    }
                    fn this_tok(&self) -> String {
                        if self.pass == 0 {
                            "m+0x0".into()
                        } else {
                            "p20+0x0".into()
                        }
                    }
                    fn lead(&self) -> String {
                        format!("p{}+0x0", 2 * self.a[1])
                    }
                    fn xy(&self) -> (i32, i32) {
                        (
                            rd(&self.p[17], 0x10) ^ 0x63637,
                            rd(&self.p[17], 0x14) ^ 0x63637,
                        )
                    }
                    fn div3(&self, i: i32) -> i32 {
                        rd(&self.c[3], i as usize * 4)
                    }
                }
                impl crate::castable::CastableView for Sw<'_> {
                    fn id(&mut self) -> i32 {
                        rd(self.sp(), 4)
                    }
                    fn from(&mut self) -> i32 {
                        rd(self.sp(), 0x3c)
                    }
                    fn from2(&mut self) -> i32 {
                        rd(self.sp(), 0x1dc)
                    }
                    fn next_spell(&mut self) {
                        self.pass = 1;
                    }
                    fn spell_slot50(&mut self) -> i32 {
                        let t = self.this_tok();
                        self.rec_log(6, t);
                        self.ans(6)
                    }
                    fn spell_slot54(&mut self) -> i32 {
                        let t = self.this_tok();
                        self.rec_log(7, t);
                        self.ans(7)
                    }
                    fn obj_is(&mut self, ty: i32) -> i32 {
                        let s = format!("p17+0x0,{ty},0");
                        if self.a[73] == 0 {
                            self.rec_log(0, s);
                            self.ans(0)
                        } else {
                            self.rec_log(1, s);
                            self.ans(1)
                        }
                    }
                    fn obj_slot18(&mut self) -> i32 {
                        self.rec_log(2, "p17+0x0".into());
                        self.ans(2)
                    }
                    fn obj_slot20(&mut self) -> i32 {
                        self.rec_log(4, "p17+0x0".into());
                        self.ans(4)
                    }
                    fn obj_slot110(&mut self) -> i32 {
                        self.rec_log(5, "p17+0x0".into());
                        self.ans(5)
                    }
                    fn is_map_unit(&mut self) -> i32 {
                        self.rec_log(13, "p17+0x0".into());
                        self.ans(13)
                    }
                    fn unit_flags(&mut self) -> u32 {
                        rd(&self.p[19], 0x68) as u32
                    }
                    fn tribe_can_type(&mut self, _ty: i32) -> i32 {
                        let s = format!("{},p27+0x0", self.lead());
                        self.rec_log(9, s);
                        self.ans(9)
                    }
                    fn type_slot68(&mut self, _ty: i32) -> i32 {
                        let s = format!("p27+0x0,{}", rd(&self.p[2 * self.a[1] as usize + 1], 0));
                        self.rec_log(8, s);
                        self.ans(8)
                    }
                    fn get_graft(&mut self, ty: i32) -> i32 {
                        let s = format!("{},{ty}", self.lead());
                        self.rec_log(10, s);
                        self.ans(10)
                    }
                    fn has_tech(&mut self, ty: i32) -> i32 {
                        let s = format!("{},{ty}", self.lead());
                        self.rec_log(18, s);
                        self.ans(18)
                    }
                    fn researching(&mut self, id: i32) -> i32 {
                        let s = format!("{},{id},{},1,0", self.lead(), self.a[0]);
                        self.rec_log(11, s);
                        self.ans(11)
                    }
                    fn is_ally(&mut self, whom: i32) -> i32 {
                        let s = format!("{},{whom}", self.lead());
                        self.rec_log(15, s);
                        self.ans(15)
                    }
                    fn leader_flag2(&mut self, i: i32) -> bool {
                        self.p[2 * i as usize][0] & 2 != 0
                    }
                    fn tile_flag100(&mut self) -> bool {
                        let (x, y) = self.xy();
                        let idx = self
                            .div3(y >> 6)
                            .wrapping_mul(rd(&self.c[1], 0x18))
                            .wrapping_add(self.div3(x >> 6));
                        let w = &self.p[22];
                        u16::from_le_bytes([w[idx as usize * 2], w[idx as usize * 2 + 1]]) & 0x100
                            != 0
                    }
                    fn tile_owner(&mut self) -> i32 {
                        let (x, y) = self.xy();
                        let idx = self
                            .div3(y >> 8)
                            .wrapping_mul(rd(&self.c[1], 0))
                            .wrapping_add(self.div3(x >> 8));
                        i32::from(self.p[21][idx as usize * 0x1c + 0xf] as i8)
                    }
                    fn find_city_at(&mut self) -> i32 {
                        let (x, y) = self.xy();
                        let s = format!("12626800,{x},{y},{},-1,0", self.a[1]);
                        self.rec_log(14, s);
                        self.ans(14)
                    }
                    fn rec(&mut self) {
                        self.rec_log(3, "p17+0x0".into());
                    }
                    fn rec_unassimilated(&mut self) -> i32 {
                        self.rec_log(12, "p23+0x0".into());
                        self.ans(12)
                    }
                    fn rec_city(&mut self) -> i32 {
                        i32::from(i16::from_le_bytes([self.p[23][0x72], self.p[23][0x73]]))
                    }
                    fn rec_flag8(&mut self) -> u32 {
                        u32::from(self.p[23][8])
                    }
                    fn rec_cfg(&mut self) -> u32 {
                        u32::from(self.p[24][0x2c0])
                    }
                    fn city_owner(&mut self, _c: i32) -> i32 {
                        i32::from(self.p[26][0x5f] as i8)
                    }
                    fn city_b60(&mut self, _c: i32) -> i32 {
                        i32::from(self.p[26][0x60] as i8)
                    }
                    fn city_flags68(&mut self, _c: i32) -> u32 {
                        u32::from(self.p[26][0x68])
                    }
                    fn city_flags4(&mut self, _c: i32) -> u32 {
                        u32::from(self.p[26][4])
                    }
                    fn depop_can_queue(&mut self, _c: i32) -> i32 {
                        self.rec_log(16, "p26+0x0".into());
                        self.ans(16)
                    }
                    fn is_disbanding(&mut self, _c: i32) -> i32 {
                        self.rec_log(17, "p26+0x0".into());
                        self.ans(17)
                    }
                }
                let mut sw = Sw {
                    a,
                    m,
                    c,
                    p,
                    pass: 0,
                    turn: [0; 19],
                    calls: vec![Vec::new(); 19],
                };
                let r = crate::castable::is_castable_steps(&mut sw, a[1], a[2]);
                for col in &sw.calls {
                    log.push(if col.is_empty() {
                        "-".to_string()
                    } else {
                        col.join("/")
                    });
                }
                i64::from(r)
            }
        };
        assert_eq!(got, want_eax, "is_castable — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "is_castable — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_castable — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_castable — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "is_castable — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2500, "only {n} rows");
    eprintln!("SpellTypeData::is_castable@00675bc0: {n} rows agree");
}
