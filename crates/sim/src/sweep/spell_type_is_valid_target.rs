//! `SpellTypeData::is_valid_target@006763c0` under the emulator against the port (item 3395; tools/sweep_gen.py, obj with windows).

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

/// `SpellTypeData::is_valid_target@006763c0`, run by `tools/emu/sweep_spell_type_is_valid_target.py` under unicorn
/// (tools/emu): 464 random bytes of object, 4 argument(s), `p` windows in order: p0 argument 3, p1..p2 sparse table 0 (2 rows × 1 slice(s), row-major), p3..p32 the place(s), in flag order; the global cell 0x00c061d0 points at 320 random bytes; 4 random bytes at 0x00c061e0 (each word in 14918544..14918544); argument 3 names 4 random bytes (null a third of the rows); callee 0x006ebaa0 (1 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x006e1200 (1 args) stubbed; callee 0x0046fb40 (0 args) stubbed; callee 0x00646b00 (2 args) stubbed; callee 0x0046fa10 (0 args) stubbed; callee 0x0046d370 (0 args) stubbed; callee 0x00594880 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0x148 (1 args) stubbed; vtable slot +0xdc (1 args) stubbed; vtable slot +0xc8 (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x120 (0 args) stubbed; vtable slot +0xc4 (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x48 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xf0 (1 args) stubbed; the word at `0xc0ab84+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p3+0` points at 32 random bytes less `0` (placed); the word at `p3+4` points at 32 random bytes less `0` (placed); the word at `p3+8` points at 32 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p7+0` points at 32 random bytes less `0` (placed); the word at `p7+4` points at 32 random bytes less `0` (placed); the word at `p7+8` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p11+0` points at 32 random bytes less `0` (placed); the word at `p11+4` points at 32 random bytes less `0` (placed); the word at `p11+8` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p15+0` points at 32 random bytes less `0` (placed); the word at `p15+4` points at 32 random bytes less `0` (placed); the word at `p15+8` points at 32 random bytes less `0` (placed); the word at `0xcae5fc` points at 64 random bytes less `0` (placed); the word at `c0+0x134` points at 384 random bytes less `0` (placed); the word at `p4+0x18` points at 544 random bytes less `0` (placed); the word at `p5+0x18` points at 544 random bytes less `0` (placed); the word at `p6+0x18` points at 544 random bytes less `0` (placed); the word at `p8+0x18` points at 544 random bytes less `0` (placed); the word at `p9+0x18` points at 544 random bytes less `0` (placed); the word at `p10+0x18` points at 544 random bytes less `0` (placed); the word at `p12+0x18` points at 544 random bytes less `0` (placed); the word at `p13+0x18` points at 544 random bytes less `0` (placed); the word at `p14+0x18` points at 544 random bytes less `0` (placed); the word at `p16+0x18` points at 544 random bytes less `0` (placed); the word at `p17+0x18` points at 544 random bytes less `0` (placed); the word at `p18+0x18` points at 544 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0x148 laid as 0x0046cf10 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1, 1] (90%); stub 1 answered from [0, 0, 1] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1, 1] (90%); stub 4 answered from [-1, -1, -1, 0, 3] (90%); stub 5 answered from [0, 0, 0, 0, 1] (90%); stub 6 answered from [0, 0, 0, 0, 1] (90%); stub 7 answered from [0, 1, 1] (90%); stub 8 answered from [0, 1, 1] (90%); stub 9 answered from [0, 1, 1] (90%); stub 10 answered from [0, 0, 1, 1] (90%); stub 11 answered from [0, 0, 1, 1] (90%); stub 12 answered from [0, 0, 0, 0, 1] (90%); stub 13 answered from [0, 0, 1] (90%); stub 14 answered from [0, 1, 1] (90%); stub 15 answered from [0, 1, 1] (90%); stub 16 answered from [0, 0, 0, 0, 1] (90%); stub 17 answered from [0, 0, 0, 0, 1] (90%); stub 18 answered from [0, 0, 1] (90%); stub 19 answered from [0, 1, 1] (90%); stub 20 answered from [0, 0, 1] (90%); stub 21 answered from [0, 0, 1, 1] (90%); stub 0 answers 4 values in call order, printed after the answers; stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 4 values in call order, printed after the answers; stub 4 answers 4 values in call order, printed after the answers; stub 5 answers 4 values in call order, printed after the answers; stub 6 answers 4 values in call order, printed after the answers; stub 7 answers 4 values in call order, printed after the answers; stub 8 answers 4 values in call order, printed after the answers; stub 9 answers 4 values in call order, printed after the answers; stub 10 answers 4 values in call order, printed after the answers; stub 11 answers 4 values in call order, printed after the answers; stub 12 answers 4 values in call order, printed after the answers; stub 13 answers 4 values in call order, printed after the answers; stub 14 answers 4 values in call order, printed after the answers; stub 15 answers 4 values in call order, printed after the answers; stub 16 answers 4 values in call order, printed after the answers; stub 17 answers 4 values in call order, printed after the answers; stub 18 answers 4 values in call order, printed after the answers; stub 19 answers 4 values in call order, printed after the answers; stub 20 answers 4 values in call order, printed after the answers; stub 21 answers 4 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 5000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_is_valid_target.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Window layout. `m` the craft row (`+0x4` the index, `+0x1c8` the flags); `c[0]` the world cell's record
            // (`+0` the width, `+0x134` the tiles, `p[20]`); `p[0]` the out-pointer's word (null when `a[3]` is 0);
            // `p[1 + k]` the leaders' first words (`0xe3a390 + 0x6eec * k`); `p[4 + 4 * pl + ox]` the `objects[pl][ox]`,
            // `p[12 + 4 * pl + ox]` the `units[pl][ox]`; `p[19]` the division table (`0xcae5fc`), `p[20]` the tiles;
            // `p[21 + 3 * pl + ox]` the type record of an object, `p[27 + 3 * pl + ox]` of a unit. `a`: the four
            // arguments, the stubs' single answers (`a[4..26]`), every stub's four call-order answers (`a[26 + 4 * k]`),
            // then the two image-slot flags (`a[114]` slot `0xb8` as `ObjectData::is`, `a[115]` slot `0x148` as
            // `has_objmask`: the base laid, so the call goes to the type record's slot).
            //
            // **The port is the crate's own steps**, `crate::spell_valid_target_view::is_valid_target_steps`, over a view
            // of the emulated original's windows. The crate's `Sim::spell_valid_target` is a different shape of the same
            // arms (see that module's head).
            use crate::spell_valid_target_view::{Table, ValidTargetView, is_valid_target_steps};
            fn rd32(w: &[u8], o: usize) -> i32 {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            }
            struct V<'a> {
                a: &'a [i32],
                m: &'a [u8],
                c: &'a [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                turn: [usize; 22],
            }
            impl V<'_> {
                fn seq(&mut self, k: usize) -> i32 {
                    let i = self.turn[k].min(3);
                    self.turn[k] += 1;
                    self.a[26 + 4 * k + i]
                }
                fn rec(&mut self, k: usize, ecx: &str, args: &[i32]) {
                    let mut t = format!("s{k}:{ecx}");
                    for v in args {
                        t.push_str(&format!(",{v}"));
                    }
                    self.log.push(t);
                }
                fn ow(w: i32, o: i32) -> usize {
                    4 + 4 * w as usize + o as usize
                }
                fn uw(w: i32, o: i32) -> usize {
                    12 + 4 * w as usize + o as usize
                }
                fn tw(w: i32, o: i32) -> usize {
                    21 + 3 * w as usize + o as usize
                }
                fn utw(w: i32, o: i32) -> usize {
                    27 + 3 * w as usize + o as usize
                }
                fn at(w: usize) -> String {
                    format!("p{w}+0x0")
                }
                fn leader(who: i32) -> String {
                    format!("r{who}+0x0")
                }
                fn is_based(&self) -> bool {
                    self.a[114] != 0
                }
                fn mask_based(&self) -> bool {
                    self.a[115] != 0
                }
            }
            impl ValidTargetView for V<'_> {
                fn flags(&self) -> i32 {
                    rd32(self.m, 0x1c8)
                }
                fn spell(&self) -> i32 {
                    rd32(self.m, 4)
                }
                fn set_out(&mut self, code: i32) {
                    if self.a[3] != 0 {
                        self.p[0][..4].copy_from_slice(&code.to_le_bytes());
                    }
                }
                fn is_unit(&mut self, whom: i32, ox: i32) -> bool {
                    self.rec(8, &Self::at(Self::ow(whom, ox)), &[]);
                    self.seq(8) != 0
                }
                fn is_building(&mut self, whom: i32, ox: i32) -> bool {
                    self.rec(9, &Self::at(Self::ow(whom, ox)), &[]);
                    self.seq(9) != 0
                }
                fn has_objmask(&mut self, table: Table, whom: i32, ox: i32) -> bool {
                    let (obj, ty) = match table {
                        Table::Objects => (Self::ow(whom, ox), Self::tw(whom, ox)),
                        Table::Units => (Self::uw(whom, ox), Self::utw(whom, ox)),
                    };
                    if self.mask_based() {
                        self.rec(21, &Self::at(ty), &[0x20_0000]);
                        self.seq(21) != 0
                    } else {
                        self.rec(10, &Self::at(obj), &[0x20_0000]);
                        self.seq(10) != 0
                    }
                }
                fn is_enemy(&mut self, who: i32, whom: i32) -> bool {
                    self.rec(0, &Self::leader(who), &[whom]);
                    self.seq(0) != 0
                }
                fn is_ally(&mut self, who: i32, whom: i32) -> bool {
                    self.rec(1, &Self::leader(who), &[whom]);
                    self.seq(1) != 0
                }
                fn is_peace(&mut self, who: i32, whom: i32) -> bool {
                    self.rec(2, &Self::leader(who), &[whom]);
                    self.seq(2) != 0
                }
                fn is_active_build(&mut self, whom: i32, ox: i32) -> bool {
                    self.rec(3, &Self::at(Self::ow(whom, ox)), &[]);
                    self.seq(3) != 0
                }
                fn object_is_city(&self, whom: i32, ox: i32) -> bool {
                    self.p[Self::ow(whom, ox)][8] & 0x20 != 0
                }
                fn slot_dc(&mut self, whom: i32, ox: i32, arg: i32) -> bool {
                    self.rec(11, &Self::at(Self::ow(whom, ox)), &[arg]);
                    self.seq(11) != 0
                }
                fn has_general(&mut self, whom: i32, ox: i32) -> i32 {
                    self.rec(4, &Self::at(Self::ow(whom, ox)), &[0, 0x165]);
                    self.seq(4)
                }
                fn slot_c8(&mut self, whom: i32, ox: i32) -> bool {
                    self.rec(12, &Self::at(Self::ow(whom, ox)), &[]);
                    self.seq(12) != 0
                }
                fn slot_20(&mut self, whom: i32, ox: i32) -> bool {
                    self.rec(13, &Self::at(Self::ow(whom, ox)), &[]);
                    self.seq(13) != 0
                }
                fn is_type(&mut self, whom: i32, ox: i32, ty: i32) -> bool {
                    if self.is_based() {
                        self.rec(20, &Self::at(Self::tw(whom, ox)), &[ty, 0]);
                        self.seq(20) != 0
                    } else {
                        self.rec(18, &Self::at(Self::ow(whom, ox)), &[ty, 0]);
                        self.seq(18) != 0
                    }
                }
                fn slot_120(&mut self, whom: i32, ox: i32) -> bool {
                    self.rec(14, &Self::at(Self::ow(whom, ox)), &[]);
                    self.seq(14) != 0
                }
                fn slot_c4(&mut self, whom: i32, ox: i32) -> bool {
                    self.rec(15, &Self::at(Self::ow(whom, ox)), &[]);
                    self.seq(15) != 0
                }
                fn slot_d0(&mut self, whom: i32, ox: i32) -> bool {
                    self.rec(16, &Self::at(Self::ow(whom, ox)), &[]);
                    self.seq(16) != 0
                }
                fn slot_cc(&mut self, whom: i32, ox: i32) -> bool {
                    self.rec(17, &Self::at(Self::ow(whom, ox)), &[]);
                    self.seq(17) != 0
                }
                fn is_worker(&mut self, whom: i32, ox: i32) -> bool {
                    self.rec(5, &Self::at(Self::ow(whom, ox)), &[]);
                    self.seq(5) != 0
                }
                fn is_merchant(&mut self, whom: i32, ox: i32) -> bool {
                    self.rec(6, &Self::at(Self::ow(whom, ox)), &[]);
                    self.seq(6) != 0
                }
                fn type_domain(&self, whom: i32, ox: i32) -> i32 {
                    rd32(&self.p[Self::tw(whom, ox)], 0x218)
                }
                fn teams_locked(&mut self, who: i32) -> bool {
                    self.rec(7, &Self::leader(who), &[]);
                    self.seq(7) != 0
                }
                fn leader_bit2(&self, whom: i32) -> bool {
                    self.p[1 + whom as usize][0] & 2 != 0
                }
                fn tile_owner(&mut self, whom: i32, ox: i32) -> i32 {
                    let obj = &self.p[Self::ow(whom, ox)];
                    let (x, y) = (rd32(obj, 0x10), rd32(obj, 0x14));
                    let div3 = &self.p[19];
                    let yi = ((y ^ 0x63637) >> 8) as usize;
                    let xi = ((x ^ 0x63637) >> 8) as usize;
                    let idx = rd32(div3, 4 * yi)
                        .wrapping_mul(rd32(&self.c[0], 0))
                        .wrapping_add(rd32(div3, 4 * xi));
                    i32::from(self.p[20][28 * idx as usize + 0xf] as i8)
                }
                fn slot_48(&mut self, who: i32, whom: i32, ox: i32) -> bool {
                    self.rec(19, &Self::at(Self::ow(whom, ox)), &[who, 0]);
                    self.seq(19) != 0
                }
            }
            let mut v = V {
                a,
                m: &*m,
                c: &*c,
                p: &mut *p,
                log: &mut log,
                turn: [0; 22],
            };
            let r = is_valid_target_steps(&mut v, a[0], a[1], a[2]);
            if log.is_empty() {
                log.push("-".to_string());
            }
            i64::from(r)
        };
        assert_eq!(got, want_eax, "is_valid_target — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "is_valid_target — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_valid_target — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_valid_target — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "is_valid_target — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 5000, "only {n} rows");
    eprintln!("SpellTypeData::is_valid_target@006763c0: {n} rows agree");
}
