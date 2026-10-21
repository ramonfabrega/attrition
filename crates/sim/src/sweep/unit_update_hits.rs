//! `Unit::update_hits@0060e930` under the emulator against the port (item 3399; tools/sweep_gen.py, obj with windows).

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

/// `Unit::update_hits@0060e930`, run by `tools/emu/sweep_unit_update_hits.py` under unicorn
/// (tools/emu): 344 random bytes of object, 1 argument(s), `p` windows in order: p0..p5 sparse table 0 (2 rows × 3 slice(s), row-major), p6..p14 the place(s), in flag order; the global cell 0x00c061f0 points at 3280 random bytes; callee 0x00647010 (1 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x006e1090 (0 args) stubbed; callee 0x006e0830 (0 args) stubbed; callee 0x006e0880 (0 args) stubbed; callee 0x00595260 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0xc4 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0xc8 (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x130 (0 args) stubbed; the word at `0xc0aec0+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p6+0` points at 160 random bytes less `0` (placed); the word at `p6+4` points at 160 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p9+0` points at 160 random bytes less `0` (placed); the word at `p9+4` points at 160 random bytes less `0` (placed); the word at `m+0x18` points at 768 random bytes less `0` (placed); the word at `p2+0` points at 224 random bytes less `0` (placed); the word at `p5+0` points at 224 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xc4 laid as 0x0046ce60 or its stub, a flag after the answers; slot +0xcc laid as 0x0046ce80 or its stub, a flag after the answers; slot +0xd0 laid as 0x0046ce90 or its stub, a flag after the answers; slot +0xc8 laid as 0x0046ce70 or its stub, a flag after the answers; slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; stub 0 answered from [0, 1, 50, 60, 90, 100, 125, 135, 250, 1000, 3000, -5, 31] (90%); stub 1 answered from [0, 1] (90%); stub 5 answered from [8, 9, 9, 9, 10] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 8 answered from [0, 1] (90%); stub 9 answered from [0, 1] (90%); stub 10 answered from [0, 1] (90%); stub 12 answered from [0, 1] (90%); stub 13 answered from [0, 1] (90%); stub 1 answers 3 values in call order, printed after the answers; stub 6 answers 2 values in call order, printed after the answers; stub 8 answers 3 values in call order, printed after the answers; stub 9 answers 2 values in call order, printed after the answers; stub 12 answers 2 values in call order, printed after the answers; stub 13 answers 2 values in call order, printed after the answers; stub 2 answers -3..7 every row; stub 3 answers -2..3 every row; stub 4 answers 0..3 every row; stub 11 answers 1..1 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x6da4:0x8, +0x6dcc:0x8, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); figures p7..p8 link at +0x90 (2 bytes) below their own index or to -1; figures p10..p11 link at +0x90 (2 bytes) below their own index or to -1; the stubbed vtable laid at `p12+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_update_hits.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Row: `a[0]` the argument (unread), `a[1 + k]` stub `k`'s answer: 0 `Object::update_hits`, 2 `get_spy_upgrade`,
                // 3 `get_general_upgrade`, 4 `get_supply_upgrade`, 5 `get_patch_version`, 7 slot `+0xc4`, 10 slot `+0xc8`, 11
                // slot `+0xe8` (1 on every row); stubs 1 (`has_tribe_bonus`), 6 (slot `+0xb8`), 8 (`+0xcc`), 9 (`+0xd0`), 12
                // (type slot `+0x60`) and 13 (type slot `+0x130`) answer in call order, their values `a[15..]`: 1 x3, 6 x2, 8
                // x3, 9 x2, 12 x2, 13 x2. Then the six flags, `a[29..35]`, that slot `+0xb8`, `+0xc4`, `+0xcc`, `+0xd0`, `+0xc8`,
                // `+0xe8` held the native function (inlined: `ObjectData::is` asks the type's slot `+0x60`, `is_hero`,
                // `is_supply`, `is_caravan` asks the type's slot `+0x130`, `is_gov_hero`, `is_captain` reads bit 15 of the
                // short at `+0x8e`). Windows: `m` the unit, `c0` `constants` (marine `+0x870`, copper `+0x920`, bananas
                // `+0x980`, iroquois `+0x830`, dutch `+0x8bc`, nubian `+0x5cc`, spy `+0x204`, supply `+0xcb4`), `p[3r..3r+3]`
                // leader `r`'s `+0x6da4`, `+0x6dcc`, `+0x6eb8`; `p6` `units[0]` and `p7`/`p8` its two units, `p9` `units[1]`
                // and `p10`/`p11`; `p12` the unit's type record; `p13`/`p14` leader 0's and 1's `+0x6eb8` record (`+0xdc` ages).
                // The leaders table is the image's, so `has_tribe_bonus`'s `ecx` is `0xe3a390 + who * 0x6eec`.
                use crate::unit_update_hits_view::{UnitHitsK, UnitHitsView, unit_update_hits};
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    who: usize,
                    k: [usize; 14],
                }
                fn rd(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                impl V<'_> {
                    /// The next value of stub `s`'s call-order sequence (`base` in `a`, `n` long).
                    fn seq(&mut self, s: usize, base: usize, n: usize) -> i32 {
                        let v = self.a[base + self.k[s].min(n - 1)];
                        self.k[s] += 1;
                        v
                    }
                    fn flag(&self, i: usize) -> bool {
                        self.a[29 + i] != 0
                    }
                    fn t(&self, off: usize) -> i32 {
                        rd(&self.p[12], off)
                    }
                    fn leader(&self, s: usize, off: usize) -> u8 {
                        self.p[3 * self.who + s][off]
                    }
                    fn unit(&self, i: i32) -> usize {
                        7 + 3 * self.who + i as usize
                    }
                }
                impl UnitHitsView for V<'_> {
                    fn is_captain(&mut self) -> bool {
                        if self.flag(5) {
                            u16::from_le_bytes([self.m[0x8e], self.m[0x8f]]) >> 15 != 0
                        } else {
                            self.a[12] != 0
                        }
                    }
                    fn redirect(&mut self) -> i32 {
                        unreachable!("the captain bit is held set")
                    }
                    fn base_hits(&mut self) -> i32 {
                        self.a[1]
                    }
                    fn is_marine(&mut self) -> bool {
                        rd(self.m, 0x6c) & 0x200 != 0
                    }
                    fn copper_bit(&mut self) -> bool {
                        self.leader(0, 2) & 4 != 0 || self.leader(1, 2) & 4 != 0
                    }
                    fn type_218(&mut self) -> i32 {
                        self.t(0x218)
                    }
                    fn trainer(&mut self) -> i32 {
                        self.t(0x40)
                    }
                    fn bananas_bit(&mut self) -> bool {
                        self.leader(0, 4) & 0x80 != 0 || self.leader(1, 4) & 0x80 != 0
                    }
                    fn has_tribe(&mut self, n: i32) -> bool {
                        self.log
                            .push(format!("s1:{},{n}", 0xe3a390 + self.who * 0x6eec));
                        self.seq(1, 15, 3) != 0
                    }
                    fn attack_word(&mut self) -> i32 {
                        self.t(0x1e8)
                    }
                    fn is_type(&mut self, n: i32) -> bool {
                        if self.flag(0) {
                            self.log.push(format!("s12:p12+0x0,{n},0"));
                            self.seq(12, 25, 2) != 0
                        } else {
                            self.log.push(format!("s6:m+0x0,{n},0"));
                            self.seq(6, 18, 2) != 0
                        }
                    }
                    fn type_index(&mut self) -> i32 {
                        self.t(4)
                    }
                    fn is_caravan(&mut self) -> bool {
                        if self.flag(3) {
                            self.seq(13, 27, 2) != 0
                        } else {
                            self.seq(9, 23, 2) != 0
                        }
                    }
                    fn is_supply(&mut self) -> bool {
                        if self.flag(2) {
                            self.t(0x2b8) & 0x40 != 0
                        } else {
                            self.seq(8, 20, 3) != 0
                        }
                    }
                    fn patch_version(&mut self) -> i32 {
                        self.a[6]
                    }
                    fn is_gov_hero(&mut self) -> bool {
                        if self.flag(4) {
                            self.t(0x2b4) & 0x4000000 != 0
                        } else {
                            self.a[11] != 0
                        }
                    }
                    fn ages_stored(&mut self) -> i32 {
                        rd(&self.p[13 + self.who], 0xdc)
                    }
                    fn spy_upgrade(&mut self) -> i32 {
                        self.a[3]
                    }
                    fn is_hero(&mut self) -> bool {
                        if self.flag(1) {
                            self.t(0x2b8) & 0x20 != 0
                        } else {
                            self.a[8] != 0
                        }
                    }
                    fn general_upgrade(&mut self) -> i32 {
                        self.a[4]
                    }
                    fn supply_upgrade(&mut self) -> i32 {
                        self.a[5]
                    }
                    fn link_of_self(&mut self) -> i32 {
                        i32::from(i16::from_le_bytes([self.m[0x90], self.m[0x91]]))
                    }
                    fn link_of(&mut self, i: i32) -> i32 {
                        let w = &self.p[self.unit(i)];
                        i32::from(i16::from_le_bytes([w[0x90], w[0x91]]))
                    }
                    fn store(&mut self, at: Option<i32>, h: i32) {
                        match at {
                            None => self.m[0x20..0x24].copy_from_slice(&h.to_le_bytes()),
                            Some(i) => {
                                let u = self.unit(i);
                                self.p[u][0x20..0x24].copy_from_slice(&h.to_le_bytes());
                            }
                        }
                    }
                    fn stored(&mut self) -> i32 {
                        rd(self.m, 0x20)
                    }
                }
                let w = |o: usize| rd(&c[0], o);
                let k = UnitHitsK {
                    marine: w(0x870),
                    copper: w(0x920),
                    bananas: w(0x980),
                    iroquois: w(0x830),
                    dutch: w(0x8bc),
                    nubian: w(0x5cc),
                    spy: [w(0x204), w(0x208), w(0x20c), w(0x210)],
                    supply: [w(0xcb4), w(0xcb8), w(0xcbc), w(0xcc0)],
                };
                let who = usize::from(m[9]);
                let mut v = V {
                    a,
                    m,
                    p,
                    log: &mut log,
                    who,
                    k: [0; 14],
                };
                i64::from(unit_update_hits(&k, &mut v))
            }
        };
        assert_eq!(got, want_eax, "update_hits — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "update_hits — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_hits — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "update_hits — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "update_hits — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Unit::update_hits@0060e930: {n} rows agree");
}
