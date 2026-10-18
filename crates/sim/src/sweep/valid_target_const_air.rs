//! `ObjectData::valid_target_const@006472c0` under the emulator against the port (item 3272; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::valid_target_const@006472c0`, run by `tools/emu/sweep_valid_target_const_air.py` under unicorn
/// (tools/emu): 32 random bytes of object, 2 argument(s), `p` windows in order: p0..p13 the place(s), in flag order; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00cae5fc points at 64 random bytes; vtable slot +0x48 (2 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0x10c (0 args) stubbed; vtable slot +0x110 (0 args) stubbed; vtable slot +0x60 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x148 (1 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x006ebaa0 (1 args) stubbed; callee 0x00646c40 (1 args) stubbed; callee 0x0060a310 (0 args) stubbed; callee 0x0060a140 (0 args) stubbed; the word at `c0+0x138` points at 64 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*u8(m,9)` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*u8(m,9)` points at 16 random bytes less `0` (placed); the word at `p1+4*a[0]` points at 32 random bytes less `0` (placed); the word at `p2+4*a[0]` points at 144 random bytes less `0` (placed); the word at `p3+4*i16(m,0xa)` points at 32 random bytes less `0` (placed); the word at `p4+4*i16(m,0xa)` points at 32 random bytes less `0` (placed); the word at `p5+0x18` points at 704 random bytes less `0` (placed); the word at `p6+0x18` points at 704 random bytes less `0` (placed); the word at `m+0x18` points at 704 random bytes less `0` (placed); the word at `p7+0x18` points at 704 random bytes less `0` (placed); the word at `p8+0x18` points at 704 random bytes less `0` (placed); slot +0xbc laid as 0x0046ce30 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [1, 1, 1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 0] (90%); stub 2 answered from [0, 0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 0, 0, 1] (90%); stub 5 answered from [0, 0, 0, 1] (90%); stub 6 answered from [0, 0, 0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 8 answered from [0, 1] (90%); stub 9 answered from [0, 1] (90%); stub 10 answered from [1, 1, 1, 0] (90%); stub 11 answered from [1, 1, 1, 1, 1, 0] (90%); stub 12 answered from [0, 1] (90%); stub 13 answered from [0, 0, 1] (90%); stub 14 answered from [0, 0, 1] (90%); stub 1 answers 3 values in call order, printed after the answers; stub 13 answers 2 values in call order, printed after the answers; stub 14 answers 2 values in call order, printed after the answers; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_valid_target_const_air.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The emulator's windows as the port's view (`crate::fight::TargetConstView`).
                //
                // Arguments: `a[0]` `param_1`, `a[1]` `param_2`; stub `k`'s answer is
                // `a[2 + k]` (stubs in flag order: 0 `+0x48`, 1 `+0x18`, 2 `+0x1c`, 3 `+0x20`,
                // 4 `+0x130`, 5 `+0x10c`, 6 `+0x110`, 7 `+0x60`, 8 `+0xb8`, 9 `+0x148`, 10
                // `+0xbc`, 11 `is_enemy`, 12 `can_carry`, 13 `is_flying_high`, 14
                // `is_flying_low`); the three seqs follow in call order (`+0x18` `a[17..20]`,
                // fly-high `a[20..22]`, fly-low `a[22..24]`), then the two keep flags
                // (`a[24]` slot `+0xbc` is the real `is_on_map`, `a[25]` slot `+0xb8` is the
                // base `is`). Windows: `m` the asker, `c0` the world record, `c1` the
                // `div_3_table`, `p0` the cell bytes, `p5` the target, `p6` its unit, `p9`
                // the target's type, `p10` its unit's type, `p11` the asker's type, `p12`
                // the asker slot's object type, `p13` the asker's unit's type.
                struct Win<'a> {
                    a: &'a [i32],
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    asked18: usize,
                    high: usize,
                    low: usize,
                }
                impl Win<'_> {
                    fn stub(&self, k: usize) -> bool {
                        self.a[2 + k] != 0
                    }
                    fn ask18(&mut self) -> bool {
                        let k = 17 + self.asked18.min(2);
                        self.asked18 += 1;
                        self.a[k] != 0
                    }
                }
                impl crate::fight::TargetConstView for Win<'_> {
                    fn who(&mut self) -> i32 {
                        i32::from(self.m[9])
                    }
                    fn is_enemy(&mut self, _who: i32, _other: i32) -> bool {
                        self.stub(11)
                    }
                    fn target_active(&mut self) -> bool {
                        self.p[5][8] & 1 != 0
                    }
                    fn target_is_seen(&mut self, _who: i32) -> bool {
                        self.stub(0)
                    }
                    fn target_is_unit(&mut self) -> bool {
                        self.ask18()
                    }
                    fn target_on_map(&mut self) -> bool {
                        if self.a[24] != 0 {
                            u16::from_le_bytes([self.p[6][0x82], self.p[6][0x83]]) >> 15 != 0
                        } else {
                            self.stub(10)
                        }
                    }
                    fn target_domain(&mut self) -> i32 {
                        rd(&self.p[9], 0x218)
                    }
                    fn self_is_unit(&mut self) -> bool {
                        self.ask18()
                    }
                    fn self_unit_flags(&mut self) -> u32 {
                        rd(&self.p[13], 0x2b4) as u32
                    }
                    fn target_vslot_1c(&mut self) -> bool {
                        self.stub(2)
                    }
                    fn self_vslot_130(&mut self) -> bool {
                        self.stub(4)
                    }
                    fn target_cell_flags(&mut self) -> u8 {
                        let (x, y) = (rd(&self.p[5], 0x10), rd(&self.p[5], 0x14));
                        let tx = rd(&self.c[1], 4 * ((x ^ 0x63637) >> 6) as usize);
                        let ty = rd(&self.c[1], 4 * ((y ^ 0x63637) >> 6) as usize);
                        let cell = ty.wrapping_mul(rd(&self.c[0], 0x18)).wrapping_add(tx);
                        self.p[0][2 * cell as usize]
                    }
                    fn target_unit_flags(&mut self) -> u32 {
                        rd(&self.p[10], 0x2b4) as u32
                    }
                    fn self_domain(&mut self) -> i32 {
                        rd(&self.p[11], 0x218)
                    }
                    fn self_vslot_20(&mut self) -> bool {
                        self.stub(3)
                    }
                    fn self_type_flags(&mut self) -> i32 {
                        rd(&self.p[11], 0x1e4)
                    }
                    fn self_range(&mut self) -> i32 {
                        rd(&self.p[11], 0x1fc)
                    }
                    fn self_type_vslot_10c(&mut self) -> bool {
                        self.stub(5)
                    }
                    fn self_type_vslot_110(&mut self) -> bool {
                        self.stub(6)
                    }
                    fn self_is_base(&mut self) -> bool {
                        self.a[25] != 0
                    }
                    fn self_type_vslot_60(&mut self) -> bool {
                        self.stub(7)
                    }
                    fn self_vslot_b8(&mut self, _what: i32) -> bool {
                        self.stub(8)
                    }
                    fn target_unit_is_base(&mut self) -> bool {
                        self.a[25] != 0
                    }
                    fn target_unit_type_vslot_60(&mut self) -> bool {
                        self.stub(7)
                    }
                    fn target_unit_vslot_b8(&mut self, _what: i32) -> bool {
                        self.stub(8)
                    }
                    fn can_carry_air(&mut self) -> bool {
                        self.stub(12)
                    }
                    fn self_vslot_148(&mut self, _what: u32) -> bool {
                        self.stub(9)
                    }
                    fn slot_fly_high(&mut self) -> i32 {
                        rd(&self.p[12], 0x250)
                    }
                    fn slot_fly_low(&mut self) -> i32 {
                        rd(&self.p[12], 0x254)
                    }
                    fn target_fly_high(&mut self) -> i32 {
                        rd(&self.p[10], 0x250)
                    }
                    fn target_fly_low(&mut self) -> i32 {
                        rd(&self.p[10], 0x254)
                    }
                    fn target_is_flying_high(&mut self) -> bool {
                        let k = 20 + self.high.min(1);
                        self.high += 1;
                        self.a[k] != 0
                    }
                    fn target_is_flying_low(&mut self) -> bool {
                        let k = 22 + self.low.min(1);
                        self.low += 1;
                        self.a[k] != 0
                    }
                    fn target_type_flags(&mut self) -> u32 {
                        rd(&self.p[9], 0x1e4) as u32
                    }
                }
                let mut w = Win {
                    a,
                    m,
                    c,
                    p,
                    asked18: 0,
                    high: 0,
                    low: 0,
                };
                i64::from(crate::fight::valid_target_const_steps(&mut w, a[0], a[1]))
            }
        };
        assert_eq!(got, want_eax, "valid_target_const — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "valid_target_const — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "valid_target_const — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "valid_target_const — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("ObjectData::valid_target_const@006472c0: {n} rows agree");
}
