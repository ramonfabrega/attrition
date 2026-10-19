//! `Unit::find_gather_spot@005f5170` under the emulator against the port (item 3320; tools/sweep_gen.py, obj with windows).

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

/// `Unit::find_gather_spot@005f5170`, run by `tools/emu/sweep_unit_find_gather_spot.py` under unicorn
/// (tools/emu): 64 random bytes of object, 3 argument(s), `p` windows in order: p0..p39 the place(s), in flag order; the global cell 0x00c0618c points at 496 random bytes; the global cell 0x00c06198 points at 16 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061ac points at 32 random bytes; callee 0x006b52e0 (2 args) stubbed; callee 0x0065b870 (5 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x180 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x0046d330 (0 args) run, not hooked; callee 0x00630450 (2 args) stubbed; callee 0x00608880 (3 args) stubbed; callee 0x0046cff0 (0 args) run, not hooked; callee 0x006e33a0 (2 args) stubbed; callee 0x00670230 (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x0061a5c0 (3 args) stubbed; the word at `c0+0x14+0x1c*m[9]` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 40 random bytes less `0` (placed); the word at `p0+4` points at 40 random bytes less `0` (placed); the word at `p0+8` points at 40 random bytes less `0` (placed); the word at `p0+12` points at 40 random bytes less `0` (placed); the word at `p0+16` points at 40 random bytes less `0` (placed); the word at `p0+20` points at 40 random bytes less `0` (placed); the word at `0xc09970+0x1c*m[9]` points at 16 random bytes less `0` (placed); the word at `p7+0` points at 96 random bytes less `0` (placed); the word at `p7+4` points at 96 random bytes less `0` (placed); the word at `p7+8` points at 96 random bytes less `0` (placed); the word at `p7+12` points at 96 random bytes less `0` (placed); the word at `c0+0x1ec` points at 16 random bytes less `0` (placed); the word at `0xe41248+0x6eec*m[9]` points at 176 random bytes less `0` (placed); the word at `m+0x18` points at 16 random bytes less `0` (placed); the word at `c3+0x10` points at 24 random bytes less `0` (placed); the word at `p1+0x20` points at 32 random bytes less `0` (placed); the word at `p1+0x24` points at 136 random bytes less `0` (placed); the word at `p16+0x18` points at 16 random bytes less `0` (placed); the word at `p17+0x18` points at 16 random bytes less `0` (placed); the word at `p2+0x20` points at 32 random bytes less `0` (placed); the word at `p2+0x24` points at 136 random bytes less `0` (placed); the word at `p20+0x18` points at 16 random bytes less `0` (placed); the word at `p21+0x18` points at 16 random bytes less `0` (placed); the word at `p3+0x20` points at 32 random bytes less `0` (placed); the word at `p3+0x24` points at 136 random bytes less `0` (placed); the word at `p24+0x18` points at 16 random bytes less `0` (placed); the word at `p25+0x18` points at 16 random bytes less `0` (placed); the word at `p4+0x20` points at 32 random bytes less `0` (placed); the word at `p4+0x24` points at 136 random bytes less `0` (placed); the word at `p28+0x18` points at 16 random bytes less `0` (placed); the word at `p29+0x18` points at 16 random bytes less `0` (placed); the word at `p5+0x20` points at 32 random bytes less `0` (placed); the word at `p5+0x24` points at 136 random bytes less `0` (placed); the word at `p32+0x18` points at 16 random bytes less `0` (placed); the word at `p33+0x18` points at 16 random bytes less `0` (placed); the word at `p6+0x20` points at 32 random bytes less `0` (placed); the word at `p6+0x24` points at 136 random bytes less `0` (placed); the word at `p36+0x18` points at 16 random bytes less `0` (placed); the word at `p37+0x18` points at 16 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0x180 laid as 0x0063ed30 or its stub, a flag after the answers; stub 0 answered from [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2] (90%); stub 2 answered from [1, 1, 1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 3 answered from [1, 1, 1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 6 answered from [1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 7 answered from [0, 0, 0, 0, 0, 0, 0, 1] (90%); stub 8 answered from [1, 1, 1, 0, 0] (90%); stub 9 answered from [0, 0, 0, 0, 1] (90%); stub 11 answered from [0, 0, 0, 1, 1, 2, 3] (90%); stub 12 answered from [0, 0, 1, 1] (90%); stub 14 answered from [1, 1, 1, 1, 1, 1, 0] (90%); stub 1 answered from [-1, -1, -1, 0, 1, 2, 3] (90%); stub 16 answered from [0, 1] (90%); the word at `p17+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p21+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p25+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p29+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p33+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p37+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 0 answers 7 values in call order, printed after the answers; stub 2 answers 6 values in call order, printed after the answers; stub 3 answers 6 values in call order, printed after the answers; stub 6 answers 6 values in call order, printed after the answers; stub 7 answers 6 values in call order, printed after the answers; stub 8 answers 14 values in call order, printed after the answers; stub 9 answers 14 values in call order, printed after the answers; stub 11 answers 6 values in call order, printed after the answers; stub 12 answers 7 values in call order, printed after the answers; stub 14 answers 36 values in call order, printed after the answers; stub 1 answers -9..3 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 4 answers from its own `ecx`: the word at `+0x20`; stub 5 answers from its own `ecx`: the word at `+0x24`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 6000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_find_gather_spot.py") else {
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
                // `a`: the three stack arguments (the first is the range), the eighteen stubs' single answers in the order
                // the line names them (0 `get_tregion`, 1 `find_city_at`, 2 slot +0xc, 3 slot +0x4c, 4 slot +0xb0 and 5 slot
                // +0xac (both answered from the object's `ecx`, not read), 6 slot +0x90, 7 slot +0x180, 8 slot +0xb8, 9 slot
                // +0x60, 10 `is_scholar` and 13 `vector_dist` (run), 11 `num_gatherers`, 12 `is_gathering_at`, 14 `type_avail`,
                // 15 `best_gather_type`, 16 `has_tribe_bonus`, 17 `add_gather_order`), then the `--seq` runs, then the two
                // `--vtat-keep` flags (slot +0xb8 is `ObjectData::is`, slot +0x180 `WallData::is_neutralized`).
                // Windows: `c[0]` the objects singleton (the owner's list at +0x14 + 0x1c * owner), `c[1]` `obj_base`
                // (start at +4), `p[1 + k]` object k (data at +0x20, build data at +0x24, position xor 0x63637 at +0x10,
                // +0x14), `p[8 + c]` city c (`free` +0x5a, `gatherers` +0x5c), `p[12]` the owners' object counts, `p[13]` the
                // ledger (cap +0x30, state +0x4c, income +0x94, a word a good), `p[14]` the unit's type record (+4),
                // `p[16 + 4k]` object k's data record, `p[17 + 4k]` its build data (vtable word either `Build::vftable` or
                // the stubbed table; city the short at +0x72, limit the byte at +0x80).
                //
                // **The port is the crate's own steps**, `crate::orders::find_gather_spot_steps`, which `Sim::find_gather_spot`
                // calls over its `GatherSpotSim`; here the view is the emulated original's windows.
                use crate::orders::{GatherSpotView, find_gather_spot_steps};
                const K: i32 = 0x63637;
                // (stub, first answer in `a`, run length): the `--seq` stubs.
                const SEQS: [(usize, usize, usize); 10] = [
                    (0, 21, 7),
                    (2, 28, 6),
                    (3, 34, 6),
                    (6, 40, 6),
                    (7, 46, 6),
                    (8, 52, 14),
                    (9, 66, 14),
                    (11, 80, 6),
                    (12, 86, 7),
                    (14, 93, 36),
                ];
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    turn: [usize; 20],
                    log: &'a mut Vec<String>,
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn rd16(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes(
                        w[o..o + 2].try_into().expect("two bytes"),
                    ))
                }
                impl V<'_> {
                    fn owner(&self) -> usize {
                        usize::from(self.m[9])
                    }
                    /// The stub's next answer, the last repeated.
                    fn next(&mut self, stub: usize) -> i32 {
                        let &(_, base, n) = SEQS.iter().find(|s| s.0 == stub).expect("a seq stub");
                        let k = self.turn[stub].min(n - 1);
                        self.turn[stub] += 1;
                        self.a[base + k]
                    }
                    /// Slot +0xb8 or +0x60 on a record: `ObjectData::is` answers through +0x60.
                    fn is_slot(&mut self) -> bool {
                        if self.a[129] != 0 {
                            self.next(9) != 0
                        } else {
                            self.next(8) != 0
                        }
                    }
                }
                impl GatherSpotView for V<'_> {
                    fn first(&mut self) -> i32 {
                        rd32(&self.c[1], 4)
                    }
                    fn count(&mut self) -> i32 {
                        rd32(&self.p[12], 4 * self.owner())
                    }
                    fn scholar(&mut self) -> bool {
                        matches!(rd32(&self.p[14], 4), 0x34 | 0x35)
                    }
                    fn region_of_unit(&mut self) -> i32 {
                        self.next(0)
                    }
                    fn city_of_unit(&mut self) -> i32 {
                        let (x, y) = self.unit_pos();
                        let o = self.owner();
                        self.log.push(format!("s1:c0+0x0,{x},{y},{o},-1,0"));
                        self.a[3 + 1]
                    }
                    fn unit_pos(&mut self) -> (i32, i32) {
                        (rd32(self.m, 0x10) ^ K, rd32(self.m, 0x14) ^ K)
                    }
                    fn alive(&mut self, _i: i32) -> bool {
                        self.next(2) != 0
                    }
                    fn active(&mut self, _i: i32) -> bool {
                        self.next(3) != 0
                    }
                    fn gather_type(&mut self, _i: i32) -> bool {
                        self.next(6) != 0
                    }
                    fn neutralized(&mut self, _i: i32) -> bool {
                        // The kept slot is `WallData::is_neutralized`, which answers no.
                        self.a[130] == 0 && self.next(7) != 0
                    }
                    fn is_university(&mut self, i: i32) -> bool {
                        if rd32(&self.p[17 + 4 * i as usize], 0) as u32 == 0x00b4_2174 {
                            self.next(9) != 0
                        } else {
                            self.is_slot()
                        }
                    }
                    fn num_gatherers(&mut self, _i: i32) -> i32 {
                        self.next(11)
                    }
                    fn max_gatherers(&mut self, i: i32) -> i32 {
                        i32::from(self.p[17 + 4 * i as usize][0x80] as i8)
                    }
                    fn gathering_at(&mut self, i: i32) -> bool {
                        let o = self.owner();
                        self.log.push(format!("s12:m+0x0,{i},{o},0"));
                        self.next(12) != 0
                    }
                    fn region_of(&mut self, _i: i32) -> i32 {
                        self.next(0)
                    }
                    fn city_of(&mut self, i: i32) -> i32 {
                        rd16(&self.p[17 + 4 * i as usize], 0x72)
                    }
                    fn city_pop(&mut self, c: i32) -> i32 {
                        let w = &self.p[8 + c as usize];
                        i32::from(w[0x5c]) + i32::from(w[0x5a])
                    }
                    fn pos_of(&mut self, i: i32) -> (i32, i32) {
                        let w = &self.p[1 + i as usize];
                        (rd32(w, 0x10) ^ K, rd32(w, 0x14) ^ K)
                    }
                    fn good_avail(&mut self, _g: usize) -> bool {
                        self.next(14) != 0
                    }
                    fn gathers(&mut self, _i: i32, _g: usize) -> bool {
                        self.is_slot()
                    }
                    fn under_cap(&mut self, g: usize) -> bool {
                        rd32(&self.p[13], 0x4c + 4 * g) == 0x8932
                    }
                    fn headroom(&mut self, g: usize) -> i32 {
                        (rd32(&self.p[13], 0x30 + 4 * g) ^ 0x1281)
                            .wrapping_sub(rd32(&self.p[13], 0x94 + 4 * g) ^ 0x90236)
                    }
                    fn dutch(&mut self) -> bool {
                        self.a[3 + 16] != 0
                    }
                    fn add_gather_order(&mut self, i: i32) {
                        let o = self.owner();
                        let _ = o;
                        self.log.push(format!("s17:m+0x0,{i},1,0"));
                    }
                }
                let mut v = V {
                    a,
                    m: &*m,
                    c: &*c,
                    p: &*p,
                    turn: [0; 20],
                    log: &mut log,
                };
                i64::from(find_gather_spot_steps(&mut v, a[0]))
            }
        };
        assert_eq!(got, want_eax, "find_gather_spot — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_gather_spot — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_gather_spot — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_gather_spot — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "find_gather_spot — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 6000, "only {n} rows");
    eprintln!("Unit::find_gather_spot@005f5170: {n} rows agree");
}
