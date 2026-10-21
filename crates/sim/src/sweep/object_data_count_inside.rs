//! `ObjectData::count_inside@0064dda0` under the emulator against the port (item 3391; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::count_inside@0064dda0`, run by `tools/emu/sweep_object_data_count_inside.py` under unicorn
/// (tools/emu): 80 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p59 the place(s), in flag order; the global cell 0x00e85ddc points at 272 random bytes; the global cell 0x00cae5fc points at 256 random bytes; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; vtable slot +0x148 (1 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x120 (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xf0 (1 args) stubbed; vtable slot +0x10c (0 args) stubbed; vtable slot +0x68 (1 args) stubbed; callee 0x0046fa10 (0 args) stubbed; callee 0x0046d310 (0 args) stubbed; callee 0x0046d330 (0 args) stubbed; callee 0x0060a850 (0 args) stubbed; callee 0x0046ffa0 (0 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x006d9380 (1 args) stubbed; callee 0x006e0c80 (1 args) stubbed; callee 0x006e3140 (1 args) stubbed; the word at `0xc0ab84` points at 24 random bytes less `0` (placed); the word at `p2+0` points at 64 random bytes less `0` (placed); the word at `p2+4` points at 64 random bytes less `0` (placed); the word at `p2+8` points at 64 random bytes less `0` (placed); the word at `p2+12` points at 64 random bytes less `0` (placed); the word at `p2+16` points at 64 random bytes less `0` (placed); the word at `p2+20` points at 64 random bytes less `0` (placed); the word at `0xc0aba0` points at 24 random bytes less `0` (placed); the word at `p9+0` points at 64 random bytes less `0` (placed); the word at `p9+4` points at 64 random bytes less `0` (placed); the word at `p9+8` points at 64 random bytes less `0` (placed); the word at `p9+12` points at 64 random bytes less `0` (placed); the word at `p9+16` points at 64 random bytes less `0` (placed); the word at `p9+20` points at 64 random bytes less `0` (placed); the word at `0xc0aec0` points at 24 random bytes less `0` (placed); the word at `p16+0` points at 108 random bytes less `0` (placed); the word at `p16+4` points at 108 random bytes less `0` (placed); the word at `p16+8` points at 108 random bytes less `0` (placed); the word at `p16+12` points at 108 random bytes less `0` (placed); the word at `p16+16` points at 108 random bytes less `0` (placed); the word at `p16+20` points at 108 random bytes less `0` (placed); the word at `0xc0aedc` points at 24 random bytes less `0` (placed); the word at `p23+0` points at 108 random bytes less `0` (placed); the word at `p23+4` points at 108 random bytes less `0` (placed); the word at `p23+8` points at 108 random bytes less `0` (placed); the word at `p23+12` points at 108 random bytes less `0` (placed); the word at `p23+16` points at 108 random bytes less `0` (placed); the word at `p23+20` points at 108 random bytes less `0` (placed); the word at `0xc0a274` points at 16 random bytes less `0` (placed); the word at `p30+0` points at 496 random bytes less `0` (placed); the word at `p30+4` points at 496 random bytes less `0` (placed); the word at `p30+8` points at 496 random bytes less `0` (placed); the word at `p30+12` points at 496 random bytes less `0` (placed); the word at `p17+0x18` points at 544 random bytes less `0` (placed); the word at `p18+0x18` points at 544 random bytes less `0` (placed); the word at `p19+0x18` points at 544 random bytes less `0` (placed); the word at `p20+0x18` points at 544 random bytes less `0` (placed); the word at `p21+0x18` points at 544 random bytes less `0` (placed); the word at `p22+0x18` points at 544 random bytes less `0` (placed); the word at `p24+0x18` points at 544 random bytes less `0` (placed); the word at `p25+0x18` points at 544 random bytes less `0` (placed); the word at `p26+0x18` points at 544 random bytes less `0` (placed); the word at `p27+0x18` points at 544 random bytes less `0` (placed); the word at `p28+0x18` points at 544 random bytes less `0` (placed); the word at `p29+0x18` points at 544 random bytes less `0` (placed); the word at `p3+0x18` points at 16 random bytes less `0` (placed); the word at `p4+0x18` points at 16 random bytes less `0` (placed); the word at `p5+0x18` points at 16 random bytes less `0` (placed); the word at `p6+0x18` points at 16 random bytes less `0` (placed); the word at `p7+0x18` points at 16 random bytes less `0` (placed); the word at `p8+0x18` points at 16 random bytes less `0` (placed); the word at `p10+0x18` points at 16 random bytes less `0` (placed); the word at `p11+0x18` points at 16 random bytes less `0` (placed); the word at `p12+0x18` points at 16 random bytes less `0` (placed); the word at `p13+0x18` points at 16 random bytes less `0` (placed); the word at `p14+0x18` points at 16 random bytes less `0` (placed); the word at `p15+0x18` points at 16 random bytes less `0` (placed); the word at `c0+0x108` points at 16 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0x148 laid as 0x0046cf10 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1, 1, 1, 1] (90%); stub 1 answered from [0, 1, 1, 1] (90%); stub 2 answered from [0, 1, 1, 1, 1] (90%); stub 3 answered from [0, 1, 1, 1, 1] (90%); stub 4 answered from [0, 1, 1, 1, 1] (90%); stub 5 answered from [0, 0, 1] (90%); stub 6 answered from [0, 0, 1] (90%); stub 7 answered from [0, 5, 10, 15, 25, 99, -7, -100, -15, -1, 1, 2, 3, 11, 12, 19, 20, 21, 123456, 2147483647, -2147483648] (90%); stub 8 answered from [0, 0, 1] (90%); stub 9 answered from [0, 1, 1, 1, 1] (90%); stub 10 answered from [0, 0, 0, 0, 1] (90%); stub 11 answered from [0, 1, 66, 67, 270, 319, 5] (90%); stub 12 answered from [0, 1] (90%); stub 13 answered from [0, 1, 1] (90%); stub 14 answered from [0, 1, 1] (90%); stub 15 answered from [6, 7, 0, 1, 6, 7, 5] (90%); stub 16 answered from [0, 1, 1, 1] (90%); stub 17 answered from [0, 1, 2, 3] (90%); stub 18 answered from [0, 1] (90%); stub 19 answered from [0, 1] (90%); stub 0 answers 10 values in call order, printed after the answers; stub 1 answers 10 values in call order, printed after the answers; stub 2 answers 10 values in call order, printed after the answers; stub 3 answers 10 values in call order, printed after the answers; stub 4 answers 10 values in call order, printed after the answers; stub 5 answers 10 values in call order, printed after the answers; stub 6 answers 10 values in call order, printed after the answers; stub 7 answers 10 values in call order, printed after the answers; stub 8 answers 10 values in call order, printed after the answers; stub 9 answers 10 values in call order, printed after the answers; stub 10 answers 10 values in call order, printed after the answers; stub 11 answers 10 values in call order, printed after the answers; stub 12 answers 10 values in call order, printed after the answers; stub 13 answers 10 values in call order, printed after the answers; stub 14 answers 10 values in call order, printed after the answers; stub 15 answers 10 values in call order, printed after the answers; stub 16 answers 10 values in call order, printed after the answers; stub 17 answers 10 values in call order, printed after the answers; stub 18 answers 10 values in call order, printed after the answers; stub 19 answers 10 values in call order, printed after the answers; stub 20 answers 10 values in call order, printed after the answers; stub 20 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 17 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 17 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); 2 rows 0x6eec apart at `0xc061e0`, slices +0x0:0xc (`p` windows after the pointed buffers, before the places); figures p3..p8 link at +0x28 (2 bytes) below their own index or to -1; figures p10..p15 link at +0x28 (2 bytes) below their own index or to -1; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p43+0` as well as at `this`; the stubbed vtable laid at `p44+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; the stubbed vtable laid at `p48+0` as well as at `this`; the stubbed vtable laid at `p49+0` as well as at `this`; the stubbed vtable laid at `p50+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; the stubbed vtable laid at `p52+0` as well as at `this`; the stubbed vtable laid at `p53+0` as well as at `this`; the stubbed vtable laid at `p54+0` as well as at `this`; the stubbed vtable laid at `p55+0` as well as at `this`; the stubbed vtable laid at `p56+0` as well as at `this`; the stubbed vtable laid at `p57+0` as well as at `this`; the stubbed vtable laid at `p58+0` as well as at `this`; the stubbed vtable laid at `p59+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_count_inside.py") else {
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
            // hand: port
            // The swept steps are `crate::count_inside_view::count_inside` over a view of the
            // emulated windows. The row's layout (`gen.py`): `p0`/`p1` the two leaders' rows,
            // `p3+7w+i` `objects[w][i]`, `p17+7w+i` `units[w][i]`, `p35+6w+i` that unit's type
            // record, `p47+6w+i` that object's type record, `p31..p34` the unit types, `p59` the
            // type list, `c[1]` `div_3_table`. Stub `s` answers `a[23 + 10 s + min(call, 9)]` in
            // its own call order; `a[233]` lays slot `+0xb8` as `ObjectData::is` itself and
            // `a[234]` slot `+0x148` as `has_objmask`, each a flag after the answers.
            use crate::count_inside_view::{CountInsideView, count_inside};
            struct W<'a> {
                a: &'a [i32],
                m: &'a [u8],
                c: &'a [Vec<u8>],
                p: &'a [Vec<u8>],
                calls: [usize; 21],
                log: &'a mut Vec<String>,
                deref: [Vec<[u8; 4]>; 2],
            }
            impl W<'_> {
                fn i16at(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes([w[o], w[o + 1]]))
                }
                fn obj(w: i32, i: i32) -> usize {
                    (3 + 7 * w + i) as usize
                }
                fn unit(w: i32, i: i32) -> usize {
                    (17 + 7 * w + i) as usize
                }
                fn utype(w: i32, i: i32) -> usize {
                    (35 + 6 * w + i) as usize
                }
                fn otype(w: i32, i: i32) -> usize {
                    (47 + 6 * w + i) as usize
                }
                /// Stub `s` handed `ecx` and `args`: the next answer in its order.
                fn ans(&mut self, s: usize, ecx: &str, args: &[i32]) -> i32 {
                    let k = self.calls[s];
                    self.calls[s] += 1;
                    let mut t = format!("s{s}:{ecx}");
                    for g in args {
                        t.push_str(&format!(",{g}"));
                    }
                    self.log.push(t);
                    self.a[23 + 10 * s + k.min(9)]
                }
                fn at(&mut self, s: usize, p: usize) -> bool {
                    let e = format!("p{p}+0x0");
                    self.ans(s, &e, &[]) != 0
                }
            }
            impl CountInsideView for W<'_> {
                fn this_on_map(&mut self) -> bool {
                    self.ans(1, "m+0x0", &[]) != 0
                }
                fn this_who(&self) -> i32 {
                    i32::from(self.m[9])
                }
                fn this_idx(&self) -> i32 {
                    Self::i16at(self.m, 0xa)
                }
                fn link(&self, w: i32, i: i32) -> i32 {
                    Self::i16at(&self.p[Self::obj(w, i)], 0x28)
                }
                fn owner(&self, w: i32, i: i32) -> i32 {
                    i32::from(self.p[Self::obj(w, i)][0x3e] as i8)
                }
                fn alive(&mut self, w: i32, i: i32) -> bool {
                    self.at(0, Self::obj(w, i))
                }
                fn obj_on_map(&mut self, w: i32, i: i32) -> bool {
                    self.at(1, Self::obj(w, i))
                }
                fn obj_is_unit(&mut self, w: i32, i: i32) -> bool {
                    self.at(2, Self::obj(w, i))
                }
                fn obj_captain(&mut self, w: i32, i: i32) -> bool {
                    self.at(3, Self::obj(w, i))
                }
                fn obj_slot_d0(&mut self, w: i32, i: i32) -> bool {
                    self.at(5, Self::obj(w, i))
                }
                fn obj_slot_148_4(&mut self, w: i32, i: i32) -> bool {
                    if self.a[234] != 0 {
                        // `has_objmask` itself: a tail jump into the type's slot `+0xf0`.
                        let e = format!("p{}+0x0", Self::otype(w, i));
                        self.ans(9, &e, &[4]) != 0
                    } else {
                        let e = format!("p{}+0x0", Self::obj(w, i));
                        self.ans(4, &e, &[4]) != 0
                    }
                }
                fn obj_is(&mut self, w: i32, i: i32, ty: i32, strict: i32) -> bool {
                    if self.a[233] != 0 {
                        let e = format!("p{}+0x0", Self::otype(w, i));
                        self.ans(8, &e, &[ty, strict]) != 0
                    } else {
                        let e = format!("p{}+0x0", Self::obj(w, i));
                        self.ans(6, &e, &[ty, strict]) != 0
                    }
                }
                fn obj_is_worker(&mut self, w: i32, i: i32) -> bool {
                    self.at(12, Self::obj(w, i))
                }
                fn obj_is_map_unit(&mut self, w: i32, i: i32) -> bool {
                    self.at(16, Self::obj(w, i))
                }
                fn obj_type_index(&self, w: i32, i: i32) -> i32 {
                    rd(&self.p[Self::otype(w, i)], 4)
                }
                fn obj_pos(&self, w: i32, i: i32, off: i32) -> i32 {
                    rd(&self.p[Self::obj(w, i)], off as usize)
                }
                fn unit_type_word(&self, w: i32, i: i32, off: i32) -> i32 {
                    rd(&self.p[Self::utype(w, i)], off as usize)
                }
                fn unit_decoy(&self, w: i32, i: i32) -> bool {
                    self.p[Self::unit(w, i)][0x68] & 1 != 0
                }
                fn unit_is_peasant(&mut self, w: i32, i: i32) -> bool {
                    self.at(13, Self::unit(w, i))
                }
                fn unit_is_worker(&mut self, w: i32, i: i32) -> bool {
                    self.at(12, Self::unit(w, i))
                }
                fn unit_is_scholar(&mut self, w: i32, i: i32) -> bool {
                    self.at(14, Self::unit(w, i))
                }
                fn unit_action_type(&mut self, w: i32, i: i32) -> i32 {
                    let e = format!("p{}+0x0", Self::unit(w, i));
                    self.ans(15, &e, &[])
                }
                fn unit_objmask_20(&mut self, w: i32, i: i32) -> bool {
                    if self.a[234] != 0 {
                        let e = format!("p{}+0x0", Self::utype(w, i));
                        self.ans(9, &e, &[0x20]) != 0
                    } else {
                        let e = format!("p{}+0x0", Self::unit(w, i));
                        self.ans(4, &e, &[0x20]) != 0
                    }
                }
                fn unit_type_slot_10c(&mut self, w: i32, i: i32) -> bool {
                    self.at(10, Self::utype(w, i))
                }
                fn unit_is(&mut self, w: i32, i: i32, ty: i32) -> bool {
                    if self.a[233] != 0 {
                        let e = format!("p{}+0x0", Self::utype(w, i));
                        self.ans(8, &e, &[ty, 0]) != 0
                    } else {
                        let e = format!("p{}+0x0", Self::unit(w, i));
                        self.ans(6, &e, &[ty, 0]) != 0
                    }
                }
                fn unit_slot_120(&mut self, w: i32, i: i32) -> i32 {
                    let e = format!("p{}+0x0", Self::unit(w, i));
                    self.ans(7, &e, &[])
                }
                fn div3(&self, idx: i32) -> i32 {
                    assert!(
                        (0..64).contains(&idx),
                        "div_3_table index {idx} outside the window"
                    );
                    rd(&self.c[1], 4 * idx as usize)
                }
                fn tregion(&mut self, x: i32, y: i32) -> i32 {
                    self.deref[0].push(x.to_le_bytes());
                    self.deref[1].push(y.to_le_bytes());
                    self.ans(17, "12621800,sp-0x20,sp-0x1c", &[])
                }
                fn tribe_can_type(&mut self, w: i32) -> bool {
                    let e = format!("p{w}+0x0,p59+0x0");
                    self.ans(18, &e, &[]) != 0
                }
                fn list_slot_68(&mut self, w: i32) -> i32 {
                    let word = rd(&self.p[w as usize], 8);
                    self.ans(11, "p59+0x0", &[word])
                }
                fn has_tech(&mut self, w: i32, ty: i32) -> bool {
                    let e = format!("p{w}+0x0");
                    self.ans(19, &e, &[ty]) != 0
                }
                fn current_upgrade(&mut self, w: i32, ty: i32) -> i32 {
                    let e = format!("p{w}+0x0");
                    self.ans(20, &e, &[ty])
                }
                fn unittype_1e8(&self, ix: i32) -> i32 {
                    rd(&self.p[31 + ix as usize], 0x1e8)
                }
            }
            let mut view = W {
                a,
                m,
                c,
                p,
                calls: [0; 21],
                log: &mut log,
                deref: [Vec::new(), Vec::new()],
            };
            let n = count_inside(&mut view, a[0], a[1]);
            let hex = |v: &[[u8; 4]]| -> String {
                if v.is_empty() {
                    return "-".to_string();
                }
                v.iter()
                    .map(|b| b.iter().map(|x| format!("{x:02x}")).collect::<String>())
                    .collect::<Vec<_>>()
                    .join("/")
            };
            let (d0, d1) = (hex(&view.deref[0]), hex(&view.deref[1]));
            log.push(d0);
            log.push(d1);
            i64::from(n)
            // end hand
        };
        assert_eq!(got, want_eax, "count_inside — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "count_inside — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "count_inside — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "count_inside — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "count_inside — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1600, "only {n} rows");
    eprintln!("ObjectData::count_inside@0064dda0: {n} rows agree");
}
