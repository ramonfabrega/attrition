//! `Army::find_besieged_city@006f58b0` under the emulator against the port (item 3281; tools/sweep_gen.py, obj with windows).

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

/// `Army::find_besieged_city@006f58b0`, run by `tools/emu/sweep_army_find_besieged_city.py` under unicorn
/// (tools/emu): 176 random bytes of object, 0 argument(s), `p` windows in order: p0..p23 sparse table 0 (8 rows × 3 slice(s), row-major), p24..p31 sparse table 1 (8 rows × 1 slice(s), row-major), p32..p215 the place(s), in flag order; the global cell 0x00c06188 points at 16 random bytes; the global cell 0x00c061b8 points at 32 random bytes; the global cell 0x00c0618c points at 32 random bytes; vtable slot +0x3c (0 args) stubbed; callee 0x00680f90 (1 args) stubbed; callee 0x0065ca80 (13 args) stubbed; callee 0x0092d130 (0 args) stubbed; callee 0x006f5cc0 (3 args) stubbed; callee 0x0046cff0 (0 args) run, not hooked; the word at `0xc09970+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p32+0` points at 112 random bytes less `0` (placed); the word at `p32+4` points at 112 random bytes less `0` (placed); the word at `p32+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p36+0` points at 112 random bytes less `0` (placed); the word at `p36+4` points at 112 random bytes less `0` (placed); the word at `p36+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*2` points at 12 random bytes less `0` (placed); the word at `p40+0` points at 112 random bytes less `0` (placed); the word at `p40+4` points at 112 random bytes less `0` (placed); the word at `p40+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*3` points at 12 random bytes less `0` (placed); the word at `p44+0` points at 112 random bytes less `0` (placed); the word at `p44+4` points at 112 random bytes less `0` (placed); the word at `p44+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*4` points at 12 random bytes less `0` (placed); the word at `p48+0` points at 112 random bytes less `0` (placed); the word at `p48+4` points at 112 random bytes less `0` (placed); the word at `p48+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*5` points at 12 random bytes less `0` (placed); the word at `p52+0` points at 112 random bytes less `0` (placed); the word at `p52+4` points at 112 random bytes less `0` (placed); the word at `p52+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*6` points at 12 random bytes less `0` (placed); the word at `p56+0` points at 112 random bytes less `0` (placed); the word at `p56+4` points at 112 random bytes less `0` (placed); the word at `p56+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*7` points at 12 random bytes less `0` (placed); the word at `p60+0` points at 112 random bytes less `0` (placed); the word at `p60+4` points at 112 random bytes less `0` (placed); the word at `p60+8` points at 112 random bytes less `0` (placed); the word at `0xc09710+0x1c*0` points at 64 random bytes less `0` (placed); the word at `p64+0` points at 36 random bytes less `0` (placed); the word at `p64+4` points at 36 random bytes less `0` (placed); the word at `p64+8` points at 36 random bytes less `0` (placed); the word at `p64+12` points at 36 random bytes less `0` (placed); the word at `p64+16` points at 36 random bytes less `0` (placed); the word at `p64+20` points at 36 random bytes less `0` (placed); the word at `p64+24` points at 36 random bytes less `0` (placed); the word at `p64+28` points at 36 random bytes less `0` (placed); the word at `p64+32` points at 36 random bytes less `0` (placed); the word at `p64+36` points at 36 random bytes less `0` (placed); the word at `p64+40` points at 36 random bytes less `0` (placed); the word at `p64+44` points at 36 random bytes less `0` (placed); the word at `p64+48` points at 36 random bytes less `0` (placed); the word at `p64+52` points at 36 random bytes less `0` (placed); the word at `p64+56` points at 36 random bytes less `0` (placed); the word at `p64+60` points at 36 random bytes less `0` (placed); the word at `0xc09710+0x1c*1` points at 64 random bytes less `0` (placed); the word at `p81+0` points at 36 random bytes less `0` (placed); the word at `p81+4` points at 36 random bytes less `0` (placed); the word at `p81+8` points at 36 random bytes less `0` (placed); the word at `p81+12` points at 36 random bytes less `0` (placed); the word at `p81+16` points at 36 random bytes less `0` (placed); the word at `p81+20` points at 36 random bytes less `0` (placed); the word at `p81+24` points at 36 random bytes less `0` (placed); the word at `p81+28` points at 36 random bytes less `0` (placed); the word at `p81+32` points at 36 random bytes less `0` (placed); the word at `p81+36` points at 36 random bytes less `0` (placed); the word at `p81+40` points at 36 random bytes less `0` (placed); the word at `p81+44` points at 36 random bytes less `0` (placed); the word at `p81+48` points at 36 random bytes less `0` (placed); the word at `p81+52` points at 36 random bytes less `0` (placed); the word at `p81+56` points at 36 random bytes less `0` (placed); the word at `p81+60` points at 36 random bytes less `0` (placed); the word at `0xc09710+0x1c*2` points at 64 random bytes less `0` (placed); the word at `p98+0` points at 36 random bytes less `0` (placed); the word at `p98+4` points at 36 random bytes less `0` (placed); the word at `p98+8` points at 36 random bytes less `0` (placed); the word at `p98+12` points at 36 random bytes less `0` (placed); the word at `p98+16` points at 36 random bytes less `0` (placed); the word at `p98+20` points at 36 random bytes less `0` (placed); the word at `p98+24` points at 36 random bytes less `0` (placed); the word at `p98+28` points at 36 random bytes less `0` (placed); the word at `p98+32` points at 36 random bytes less `0` (placed); the word at `p98+36` points at 36 random bytes less `0` (placed); the word at `p98+40` points at 36 random bytes less `0` (placed); the word at `p98+44` points at 36 random bytes less `0` (placed); the word at `p98+48` points at 36 random bytes less `0` (placed); the word at `p98+52` points at 36 random bytes less `0` (placed); the word at `p98+56` points at 36 random bytes less `0` (placed); the word at `p98+60` points at 36 random bytes less `0` (placed); the word at `0xc09710+0x1c*3` points at 64 random bytes less `0` (placed); the word at `p115+0` points at 36 random bytes less `0` (placed); the word at `p115+4` points at 36 random bytes less `0` (placed); the word at `p115+8` points at 36 random bytes less `0` (placed); the word at `p115+12` points at 36 random bytes less `0` (placed); the word at `p115+16` points at 36 random bytes less `0` (placed); the word at `p115+20` points at 36 random bytes less `0` (placed); the word at `p115+24` points at 36 random bytes less `0` (placed); the word at `p115+28` points at 36 random bytes less `0` (placed); the word at `p115+32` points at 36 random bytes less `0` (placed); the word at `p115+36` points at 36 random bytes less `0` (placed); the word at `p115+40` points at 36 random bytes less `0` (placed); the word at `p115+44` points at 36 random bytes less `0` (placed); the word at `p115+48` points at 36 random bytes less `0` (placed); the word at `p115+52` points at 36 random bytes less `0` (placed); the word at `p115+56` points at 36 random bytes less `0` (placed); the word at `p115+60` points at 36 random bytes less `0` (placed); the word at `0xc09710+0x1c*4` points at 64 random bytes less `0` (placed); the word at `p132+0` points at 36 random bytes less `0` (placed); the word at `p132+4` points at 36 random bytes less `0` (placed); the word at `p132+8` points at 36 random bytes less `0` (placed); the word at `p132+12` points at 36 random bytes less `0` (placed); the word at `p132+16` points at 36 random bytes less `0` (placed); the word at `p132+20` points at 36 random bytes less `0` (placed); the word at `p132+24` points at 36 random bytes less `0` (placed); the word at `p132+28` points at 36 random bytes less `0` (placed); the word at `p132+32` points at 36 random bytes less `0` (placed); the word at `p132+36` points at 36 random bytes less `0` (placed); the word at `p132+40` points at 36 random bytes less `0` (placed); the word at `p132+44` points at 36 random bytes less `0` (placed); the word at `p132+48` points at 36 random bytes less `0` (placed); the word at `p132+52` points at 36 random bytes less `0` (placed); the word at `p132+56` points at 36 random bytes less `0` (placed); the word at `p132+60` points at 36 random bytes less `0` (placed); the word at `0xc09710+0x1c*5` points at 64 random bytes less `0` (placed); the word at `p149+0` points at 36 random bytes less `0` (placed); the word at `p149+4` points at 36 random bytes less `0` (placed); the word at `p149+8` points at 36 random bytes less `0` (placed); the word at `p149+12` points at 36 random bytes less `0` (placed); the word at `p149+16` points at 36 random bytes less `0` (placed); the word at `p149+20` points at 36 random bytes less `0` (placed); the word at `p149+24` points at 36 random bytes less `0` (placed); the word at `p149+28` points at 36 random bytes less `0` (placed); the word at `p149+32` points at 36 random bytes less `0` (placed); the word at `p149+36` points at 36 random bytes less `0` (placed); the word at `p149+40` points at 36 random bytes less `0` (placed); the word at `p149+44` points at 36 random bytes less `0` (placed); the word at `p149+48` points at 36 random bytes less `0` (placed); the word at `p149+52` points at 36 random bytes less `0` (placed); the word at `p149+56` points at 36 random bytes less `0` (placed); the word at `p149+60` points at 36 random bytes less `0` (placed); the word at `0xc09710+0x1c*6` points at 64 random bytes less `0` (placed); the word at `p166+0` points at 36 random bytes less `0` (placed); the word at `p166+4` points at 36 random bytes less `0` (placed); the word at `p166+8` points at 36 random bytes less `0` (placed); the word at `p166+12` points at 36 random bytes less `0` (placed); the word at `p166+16` points at 36 random bytes less `0` (placed); the word at `p166+20` points at 36 random bytes less `0` (placed); the word at `p166+24` points at 36 random bytes less `0` (placed); the word at `p166+28` points at 36 random bytes less `0` (placed); the word at `p166+32` points at 36 random bytes less `0` (placed); the word at `p166+36` points at 36 random bytes less `0` (placed); the word at `p166+40` points at 36 random bytes less `0` (placed); the word at `p166+44` points at 36 random bytes less `0` (placed); the word at `p166+48` points at 36 random bytes less `0` (placed); the word at `p166+52` points at 36 random bytes less `0` (placed); the word at `p166+56` points at 36 random bytes less `0` (placed); the word at `p166+60` points at 36 random bytes less `0` (placed); the word at `0xc09710+0x1c*7` points at 64 random bytes less `0` (placed); the word at `p183+0` points at 36 random bytes less `0` (placed); the word at `p183+4` points at 36 random bytes less `0` (placed); the word at `p183+8` points at 36 random bytes less `0` (placed); the word at `p183+12` points at 36 random bytes less `0` (placed); the word at `p183+16` points at 36 random bytes less `0` (placed); the word at `p183+20` points at 36 random bytes less `0` (placed); the word at `p183+24` points at 36 random bytes less `0` (placed); the word at `p183+28` points at 36 random bytes less `0` (placed); the word at `p183+32` points at 36 random bytes less `0` (placed); the word at `p183+36` points at 36 random bytes less `0` (placed); the word at `p183+40` points at 36 random bytes less `0` (placed); the word at `p183+44` points at 36 random bytes less `0` (placed); the word at `p183+48` points at 36 random bytes less `0` (placed); the word at `p183+52` points at 36 random bytes less `0` (placed); the word at `p183+56` points at 36 random bytes less `0` (placed); the word at `p183+60` points at 36 random bytes less `0` (placed); the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p200+0` points at 16 random bytes less `0` (placed); the word at `p200+4` points at 16 random bytes less `0` (placed); the word at `p200+8` points at 16 random bytes less `0` (placed); the word at `p200+12` points at 16 random bytes less `0` (placed); the word at `m+0x98` points at 16 random bytes less `0` (placed); the word at `p205+0` points at 32 random bytes less `0` (placed); the word at `p205+4` points at 32 random bytes less `0` (placed); the word at `p205+8` points at 32 random bytes less `0` (placed); the word at `p205+12` points at 32 random bytes less `0` (placed); the word at `m+0xa0` points at 16 random bytes less `0` (placed); the word at `p210+0` points at 16 random bytes less `0` (placed); the word at `p210+4` points at 16 random bytes less `0` (placed); the word at `p210+8` points at 16 random bytes less `0` (placed); the word at `p210+12` points at 16 random bytes less `0` (placed); the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); stub 1 answered from [0, 1] (90%); stub 2 answered from [-1, 0, 5, -1] (90%); the word at `p206+0x18` laid as the address of `p211` every row; the word at `p207+0x18` laid as the address of `p212` every row; the word at `p208+0x18` laid as the address of `p213` every row; the word at `p209+0x18` laid as the address of `p214` every row; stub 0 answers `p206+j`, the data of the object its `ecx` is in `p201+j` (j < 4), linked at `+0x1c`; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0xc, +0x74:0x20, +0x408:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x20 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p201+0` as well as at `this`; the stubbed vtable laid at `p202+0` as well as at `this`; the stubbed vtable laid at `p203+0` as well as at `this`; the stubbed vtable laid at `p204+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_find_besieged_city.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `a`: the stubs' answers in the order the line names them (0 slot +0x3c, answered by the objects'
                // own windows; 1 `is_coast`; 2 `find_unit`; 3 `find_angle`; 4 `find_muster_spot`; 5 `vector_dist`,
                // run, not hooked). Windows: `m` the army (`+2` its slot, `+8` region, `+0x24` navy, `+0x30`/`+0x34`
                // the target, `+0x38`/`+0x3c` the point, `+0x48`/`+0x4c` the muster, `+0x50` its angle, `+0x94` the
                // leader), `c[0]` the world, `c[1]` the regions' cell, `c[2]` the objects' singleton, `p[3r..3r+3]`
                // leader `r`'s slices (`+0..12`, `+0x74..`, `+0x408`), `p[24+r]` the diplomacy rows of the second table,
                // `p[32+4j]` the pointer array of leader `j`'s cities and `p[33+4j+k]` their records, `p[64+17j]` its
                // armies' array and `p[65+17j+s]` their records, `p[200..205]` the objects table (`+0` the stubbed
                // vtable), `p[205..210]` slot +0x3c's answers, `p[210..215]` the records those point at (`+4` the type),
                // `p[215]` the `div_3_table`. Logs, one per recorded stub, each call `ecx,edx,args`; the `edx` of a
                // callee that is not handed one is whatever was left, `*`, masked below.
                //
                // **The port is the crate's own steps**, `crate::army::find_besieged_city_steps`, which
                // `Sim::find_besieged_city` calls over its `BesiegedSim`; here the view is the emulated original's windows.
                use crate::army::{BesiegedView, find_besieged_city_steps};
                use crate::movement::Angle;
                use crate::world::Pos;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn rd16(w: &[u8], o: usize) -> i32 {
                    i16::from_le_bytes(w[o..o + 2].try_into().expect("two bytes")) as i32
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
                    fn city(&self, j: i32, k: i32) -> &[u8] {
                        &self.p[33 + 4 * j as usize + k as usize]
                    }
                }
                impl BesiegedView for V<'_> {
                    fn leaders(&self) -> i32 {
                        8
                    }
                    fn who(&self) -> i32 {
                        rd16(self.m, 0x94)
                    }
                    fn slot(&self) -> i32 {
                        rd16(self.m, 2)
                    }
                    fn navy(&self) -> bool {
                        rd32(self.m, 0x24) != 0
                    }
                    fn reg(&self) -> i32 {
                        rd32(self.m, 8)
                    }
                    fn pos(&self) -> Pos {
                        Pos::new(rd32(self.m, 0x38), rd32(self.m, 0x3c))
                    }
                    fn leader_active(&mut self, j: i32) -> bool {
                        self.p[3 * j as usize][0] & 1 != 0
                    }
                    fn friendly(&mut self, j: i32) -> bool {
                        let l = self.who() as usize;
                        let me = rd32(&self.p[3 * l], 8);
                        j == me
                            || (rd32(&self.p[3 * l + 1], 4 * j as usize) == 2
                                && rd32(&self.p[24 + j as usize], 4 * me as usize) == 2)
                    }
                    fn city_bound(&self) -> i32 {
                        rd32(&self.p[3 * self.who() as usize + 2], 0)
                    }
                    fn city_flags(&mut self, j: i32, k: i32) -> i32 {
                        i32::from(u16::from_le_bytes([self.city(j, k)[4], self.city(j, k)[5]]))
                    }
                    fn city_reg(&self, j: i32, k: i32) -> i32 {
                        rd16(self.city(j, k), 0xa)
                    }
                    fn is_coast(&mut self, reg: i32, city_reg: i32) -> bool {
                        let ecx = rd32(&self.c[1], 0x10).wrapping_add(reg.wrapping_mul(0x88));
                        self.push(0, format!("{ecx},*,{city_reg}"));
                        self.a[1] != 0
                    }
                    fn city_type(&mut self, j: i32, k: i32) -> i32 {
                        // the objects table at `0xc0ab84 + 0x1c * (byte at +0x5e)`: the byte is 0 on every row.
                        assert_eq!(
                            self.city(j, k)[0x5e],
                            0,
                            "the objects table's second row is not laid"
                        );
                        let idx = rd16(self.city(j, k), 8) as usize;
                        rd32(&self.p[211 + idx], 4)
                    }
                    fn enemy_ship_near(&mut self, j: i32, k: i32) -> bool {
                        let c = self.city(j, k);
                        let (x, y) = (rd32(c, 0xc), rd32(c, 0x10));
                        let who = self.who();
                        self.push(
                            1,
                            format!(
                                "c2+0x0,*,{x},{y},3,{who},4608,0,13,1,0,0,0,0,p{}+{:#x}",
                                32 + 4 * j,
                                4 * k
                            ),
                        );
                        self.a[2] >= 0
                    }
                    fn army_valid(&self, j: i32, s: i32) -> bool {
                        rd16(&self.p[65 + 17 * j as usize + s as usize], 0) != 0
                    }
                    fn army_in_city(&self, j: i32, s: i32, k: i32) -> bool {
                        rd32(&self.p[65 + 17 * j as usize + s as usize], 0x20) == k
                    }
                    fn army_standard(&self, j: i32, s: i32) -> i32 {
                        rd32(&self.p[65 + 17 * j as usize + s as usize], 0x14)
                    }
                    fn city_pos(&self, j: i32, k: i32) -> Pos {
                        let c = self.city(j, k);
                        Pos::new(rd32(c, 0xc), rd32(c, 0x10))
                    }
                    fn world_cells(&self) -> (i32, i32) {
                        (rd32(&self.c[0], 0), rd32(&self.c[0], 4))
                    }
                    fn find_muster_spot(&mut self, j: i32, k: i32) {
                        let b = rd16(self.city(j, k), 8);
                        self.push(3, format!("m+0x0,*,{b},{j},0"));
                    }
                    fn muster_cell(&self) -> (i32, i32) {
                        (rd32(self.m, 0x48), rd32(self.m, 0x4c))
                    }
                    fn set_muster_from_pos(&mut self) {
                        let c = self.pos().cell();
                        wr32(self.m, 0x48, c.x);
                        wr32(self.m, 0x4c, c.y);
                    }
                    fn set_muster_angle(&mut self, a: Angle) {
                        wr32(self.m, 0x50, a.0);
                    }
                    fn find_angle(&mut self, dx: i32, dy: i32) -> Angle {
                        self.push(2, format!("{dx},{dy}"));
                        Angle(self.a[3])
                    }
                    fn set_pos(&mut self, p: Pos) {
                        wr32(self.m, 0x38, p.x);
                        wr32(self.m, 0x3c, p.y);
                    }
                    fn set_target(&mut self, j: i32, k: i32) {
                        let b = rd16(self.city(j, k), 8);
                        wr32(self.m, 0x30, b);
                        wr32(self.m, 0x34, j);
                    }
                }
                log.extend(std::iter::repeat_n("-".to_string(), 4));
                let mut v = V {
                    a,
                    m,
                    c,
                    p: &*p,
                    log: &mut log,
                };
                i64::from(find_besieged_city_steps(&mut v))
            }
        };
        assert_eq!(got, want_eax, "find_besieged_city — eax, row `{line}`");
        // The `edx` of a callee that is not handed one is the stack's and not the function's (every stub but
        // `find_angle`), so both sides are masked.
        let mask = |logs: &[&str]| -> Vec<String> {
            logs.iter()
                .enumerate()
                .map(|(k, col)| {
                    col.split('/')
                        .map(|call| {
                            let mut f: Vec<&str> = call.split(',').collect();
                            if k != 2 && f.len() > 1 {
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
            "find_besieged_city — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "find_besieged_city — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_besieged_city — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "find_besieged_city — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("Army::find_besieged_city@006f58b0: {n} rows agree");
}
