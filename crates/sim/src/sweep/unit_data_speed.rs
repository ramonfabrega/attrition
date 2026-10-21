//! `UnitData::speed@0060aae0` under the emulator against the port (item 3399; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::speed@0060aae0`, run by `tools/emu/sweep_unit_data_speed.py` under unicorn
/// (tools/emu): 160 random bytes of object, 0 argument(s), `p` windows in order: p0..p3 sparse table 0 (2 rows × 2 slice(s), row-major), p4..p7 sparse table 1 (1 rows × 4 slice(s), row-major), p8..p25 the place(s), in flag order; the global cell 0x00c061d0 points at 312 random bytes; the global cell 0x00cae5fc points at 64 random bytes; vtable slot +0x18 (0 args) stubbed; vtable slot +0xc4 (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x0073a1b0 (6 args) stubbed; the word at `m+0x18` points at 576 random bytes less `0` (placed); the word at `c0+0x134` points at 256 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `0xc0aedc` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 112 random bytes less `0` (placed); the word at `p10+4` points at 112 random bytes less `0` (placed); the word at `p11+0` points at 112 random bytes less `0` (placed); the word at `p11+4` points at 112 random bytes less `0` (placed); the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `0xc0aba0` points at 16 random bytes less `0` (placed); the word at `p16+0` points at 32 random bytes less `0` (placed); the word at `p16+4` points at 32 random bytes less `0` (placed); the word at `p17+0` points at 32 random bytes less `0` (placed); the word at `p17+4` points at 32 random bytes less `0` (placed); the word at `p18+0x18` points at 8 random bytes less `0` (placed); the word at `p19+0x18` points at 8 random bytes less `0` (placed); the word at `p20+0x18` points at 8 random bytes less `0` (placed); the word at `p21+0x18` points at 8 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 0, 0, 0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1, 1] (90%); stub 3 answers 10 values in call order, printed after the answers; stub 4 answers 10 values in call order, printed after the answers; stub 6 answers -1..1 every row; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4, +0x59cc:0x20 (`p` windows after the pointed buffers, before the places); 1 rows 0x1000 apart at `0xc061e4`, slices +0x4:0x4, +0x838:0x10, +0xb4c:0x4, +0xc50:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_speed.py") else {
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
            {
                // `a`: 0..=6 the stubs' single answers (slot +0x18 `is_active`, slot +0xc4 `is_hero`, slot +0x20 the
                // footprint, slot +0xb8 and slot +0x60 (their sequences below), `is_ally`, `find_hero`), 7.. ten of slot
                // +0xb8's answers in call order, 17.. ten of slot +0x60's, 27 the flag that `this`'s slot +0xb8 (and every
                // hero object's) is `ObjectData::is` itself, whose answer is then slot +0x60's. Windows: `m` the unit
                // (owner byte +9, own slot short +0xa, y +0x10 and x +0x14 both xor 0x63637, `myspeed` short +0x9a),
                // `c0` the leaders (0x6eec a record; flags +0), `c1` the constants, `c2` the world (width at +0,
                // cells at +0x134), `c3` the `div_3_table`; `p0` the type (+0x218), `p1` the cells (0x1c a record,
                // owner byte at +0xf), `p4 + 2 * owner + slot` the units' records (masks at +0x68), `p10 + 2 * owner
                // + slot` the hero objects. The leaders' power words (`+0x59cc..+0x59e8`) are held zero: the auras
                // are not walked (SEAM, `unit_speed_view`). `is_ally`'s and `find_hero`'s arguments, and the
                // footprint handed to it, are not held (no `--record`); owners 0 and 1 only, never Gaia.
                use crate::unit_speed_view::{UnitSpeedView, unit_speed};
                const KEY: i32 = 0x63637;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    k: [usize; 2],
                    log: &'a mut Vec<String>,
                }
                fn rd(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                impl V<'_> {
                    fn owner(&self) -> usize {
                        usize::from(self.m[9])
                    }
                    fn slot(&self) -> i32 {
                        i32::from(i16::from_le_bytes([self.m[10], self.m[11]]))
                    }
                    /// One `is` answer: slot +0x60's when the slot is `ObjectData::is`, else slot +0xb8's.
                    fn is(&mut self, who: &str, x: i32, strict: bool) -> bool {
                        let (s, base) = if self.a[27] != 0 { (1, 17) } else { (0, 7) };
                        // The recorded call is the stub's, headed by its index, `ecx` the object (slot +0xb8) or its type.
                        self.log
                            .push(format!("s{}:{who}+0x0,{x},{}", 3 + s, i32::from(strict)));
                        let v = self.a[base + self.k[s].min(9)];
                        self.k[s] += 1;
                        v != 0
                    }
                }
                impl UnitSpeedView for V<'_> {
                    fn myspeed(&self) -> i32 {
                        i32::from(i16::from_le_bytes([self.m[0x9a], self.m[0x9b]]))
                    }
                    fn not_land(&self) -> bool {
                        rd(&self.p[8], 0x218) != 0
                    }
                    fn is_this(&mut self, x: i32, strict: bool) -> bool {
                        let who = if self.a[27] != 0 { "p8" } else { "m" };
                        self.is(who, x, strict)
                    }
                    fn ground_owner(&mut self) -> Option<i32> {
                        let d3 = |v: i32| rd(&self.c[1], 4 * ((v ^ KEY) >> 8) as usize);
                        let cell = d3(rd(self.m, 0x14)) * rd(&self.c[0], 0) + d3(rd(self.m, 0x10));
                        let b = i32::from(self.p[9][0x1c * cell as usize + 0xf] as i8);
                        (b >= 0).then_some(b)
                    }
                    fn is_ally(&mut self, _other: i32) -> bool {
                        self.a[5] != 0
                    }
                    fn spear_speed(&self, k: usize) -> i32 {
                        rd(&self.p[5], 4 * k)
                    }
                    fn leader_marching(&self) -> bool {
                        rd(&self.p[2 * self.owner()], 0) & 0x8000 != 0
                    }
                    fn is_active(&mut self) -> bool {
                        self.a[0] != 0
                    }
                    fn is_hero(&mut self) -> bool {
                        self.a[1] != 0
                    }
                    fn own_slot_marching(&mut self) -> bool {
                        rd(&self.p[12 + 2 * self.owner() + self.slot() as usize], 0x68) & 0x8000
                            != 0
                    }
                    fn own_slot(&self) -> i32 {
                        self.slot()
                    }
                    fn find_hero(&mut self) -> i32 {
                        self.a[6]
                    }
                    fn hero_is(&mut self, slot: i32, x: i32) -> bool {
                        let obj = 2 * self.owner() + slot as usize;
                        let who = if self.a[27] != 0 {
                            format!("p{}", 22 + obj)
                        } else {
                            format!("p{}", 18 + obj)
                        };
                        self.is(&who, x, false)
                    }
                    fn forced_march_speed(&self) -> i32 {
                        rd(&self.p[7], 0)
                    }
                    fn alexander_forced_march_speed(&self) -> i32 {
                        rd(&self.p[6], 0)
                    }
                    fn unit_move_speed(&self) -> i32 {
                        rd(&self.p[4], 0)
                    }
                }
                let mut v = V {
                    a,
                    m,
                    c,
                    p,
                    k: [0; 2],
                    log: &mut log,
                };
                let r = unit_speed(&mut v);
                if log.is_empty() {
                    log.push("-".to_string());
                }
                i64::from(r)
            }
        };
        assert_eq!(got, want_eax, "speed — eax, row `{line}`");
        assert_eq!(log, want_log, "speed — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "speed — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "speed — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "speed — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("UnitData::speed@0060aae0: {n} rows agree");
}
