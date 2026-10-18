//! `BuildData::construct_time@0062d5c0` under the emulator against the port (item 3261; tools/sweep_gen.py, obj with windows).

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

/// `BuildData::construct_time@0062d5c0`, run by `tools/emu/sweep_build_data_construct_time.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 1 argument(s), `p` windows in order: p0..p31 sparse table 0 (8 rows × 4 slice(s), row-major), p32..p33 sparse table 1 (2 rows × 1 slice(s), row-major), p34..p35 sparse table 2 (2 rows × 1 slice(s), row-major), p36..p37 sparse table 3 (2 rows × 1 slice(s), row-major), p38..p39 sparse table 4 (2 rows × 1 slice(s), row-major), p40..p41 sparse table 5 (2 rows × 1 slice(s), row-major), p42..p43 sparse table 6 (2 rows × 1 slice(s), row-major), p44..p45 sparse table 7 (2 rows × 1 slice(s), row-major), p46..p47 sparse table 8 (2 rows × 1 slice(s), row-major), p48 sparse table 9 (1 rows × 1 slice(s), row-major), p49 sparse table 10 (1 rows × 1 slice(s), row-major), p50 sparse table 11 (1 rows × 1 slice(s), row-major), p51 sparse table 12 (1 rows × 1 slice(s), row-major), p52 sparse table 13 (1 rows × 1 slice(s), row-major), p53 sparse table 14 (1 rows × 1 slice(s), row-major), p54 sparse table 15 (1 rows × 1 slice(s), row-major), p55 sparse table 16 (1 rows × 1 slice(s), row-major), p56..p184 the place(s), in flag order, p185..p188 the stubs' answer window(s); the global cell 0x00c061e4 points at 3224 random bytes; 32 random bytes at 0x00c0acf4 (each word in 2000..2002); vtable slot +0x2c (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x006db680 (0 args) stubbed; callee 0x006ebc10 (1 args) stubbed; callee 0x00646b00 (2 args) stubbed; the word at `m+0x18` points at 16 random bytes less `0` (placed); the word at `p32+0` points at 32 random bytes less `0` (placed); the word at `p57+0x18` points at 16 random bytes less `0` (placed); the word at `p32+4` points at 32 random bytes less `0` (placed); the word at `p59+0x18` points at 16 random bytes less `0` (placed); the word at `p32+8` points at 32 random bytes less `0` (placed); the word at `p61+0x18` points at 16 random bytes less `0` (placed); the word at `p32+12` points at 32 random bytes less `0` (placed); the word at `p63+0x18` points at 16 random bytes less `0` (placed); the word at `p34+0` points at 32 random bytes less `0` (placed); the word at `p65+0x18` points at 16 random bytes less `0` (placed); the word at `p34+4` points at 32 random bytes less `0` (placed); the word at `p67+0x18` points at 16 random bytes less `0` (placed); the word at `p34+8` points at 32 random bytes less `0` (placed); the word at `p69+0x18` points at 16 random bytes less `0` (placed); the word at `p34+12` points at 32 random bytes less `0` (placed); the word at `p71+0x18` points at 16 random bytes less `0` (placed); the word at `p36+0` points at 32 random bytes less `0` (placed); the word at `p73+0x18` points at 16 random bytes less `0` (placed); the word at `p36+4` points at 32 random bytes less `0` (placed); the word at `p75+0x18` points at 16 random bytes less `0` (placed); the word at `p36+8` points at 32 random bytes less `0` (placed); the word at `p77+0x18` points at 16 random bytes less `0` (placed); the word at `p36+12` points at 32 random bytes less `0` (placed); the word at `p79+0x18` points at 16 random bytes less `0` (placed); the word at `p38+0` points at 32 random bytes less `0` (placed); the word at `p81+0x18` points at 16 random bytes less `0` (placed); the word at `p38+4` points at 32 random bytes less `0` (placed); the word at `p83+0x18` points at 16 random bytes less `0` (placed); the word at `p38+8` points at 32 random bytes less `0` (placed); the word at `p85+0x18` points at 16 random bytes less `0` (placed); the word at `p38+12` points at 32 random bytes less `0` (placed); the word at `p87+0x18` points at 16 random bytes less `0` (placed); the word at `p40+0` points at 32 random bytes less `0` (placed); the word at `p89+0x18` points at 16 random bytes less `0` (placed); the word at `p40+4` points at 32 random bytes less `0` (placed); the word at `p91+0x18` points at 16 random bytes less `0` (placed); the word at `p40+8` points at 32 random bytes less `0` (placed); the word at `p93+0x18` points at 16 random bytes less `0` (placed); the word at `p40+12` points at 32 random bytes less `0` (placed); the word at `p95+0x18` points at 16 random bytes less `0` (placed); the word at `p42+0` points at 32 random bytes less `0` (placed); the word at `p97+0x18` points at 16 random bytes less `0` (placed); the word at `p42+4` points at 32 random bytes less `0` (placed); the word at `p99+0x18` points at 16 random bytes less `0` (placed); the word at `p42+8` points at 32 random bytes less `0` (placed); the word at `p101+0x18` points at 16 random bytes less `0` (placed); the word at `p42+12` points at 32 random bytes less `0` (placed); the word at `p103+0x18` points at 16 random bytes less `0` (placed); the word at `p44+0` points at 32 random bytes less `0` (placed); the word at `p105+0x18` points at 16 random bytes less `0` (placed); the word at `p44+4` points at 32 random bytes less `0` (placed); the word at `p107+0x18` points at 16 random bytes less `0` (placed); the word at `p44+8` points at 32 random bytes less `0` (placed); the word at `p109+0x18` points at 16 random bytes less `0` (placed); the word at `p44+12` points at 32 random bytes less `0` (placed); the word at `p111+0x18` points at 16 random bytes less `0` (placed); the word at `p46+0` points at 32 random bytes less `0` (placed); the word at `p113+0x18` points at 16 random bytes less `0` (placed); the word at `p46+4` points at 32 random bytes less `0` (placed); the word at `p115+0x18` points at 16 random bytes less `0` (placed); the word at `p46+8` points at 32 random bytes less `0` (placed); the word at `p117+0x18` points at 16 random bytes less `0` (placed); the word at `p46+12` points at 32 random bytes less `0` (placed); the word at `p119+0x18` points at 16 random bytes less `0` (placed); the word at `p33+0` points at 32 random bytes less `0` (placed); the word at `p121+0x18` points at 16 random bytes less `0` (placed); the word at `p33+4` points at 32 random bytes less `0` (placed); the word at `p123+0x18` points at 16 random bytes less `0` (placed); the word at `p35+0` points at 32 random bytes less `0` (placed); the word at `p125+0x18` points at 16 random bytes less `0` (placed); the word at `p35+4` points at 32 random bytes less `0` (placed); the word at `p127+0x18` points at 16 random bytes less `0` (placed); the word at `p37+0` points at 32 random bytes less `0` (placed); the word at `p129+0x18` points at 16 random bytes less `0` (placed); the word at `p37+4` points at 32 random bytes less `0` (placed); the word at `p131+0x18` points at 16 random bytes less `0` (placed); the word at `p39+0` points at 32 random bytes less `0` (placed); the word at `p133+0x18` points at 16 random bytes less `0` (placed); the word at `p39+4` points at 32 random bytes less `0` (placed); the word at `p135+0x18` points at 16 random bytes less `0` (placed); the word at `p41+0` points at 32 random bytes less `0` (placed); the word at `p137+0x18` points at 16 random bytes less `0` (placed); the word at `p41+4` points at 32 random bytes less `0` (placed); the word at `p139+0x18` points at 16 random bytes less `0` (placed); the word at `p43+0` points at 32 random bytes less `0` (placed); the word at `p141+0x18` points at 16 random bytes less `0` (placed); the word at `p43+4` points at 32 random bytes less `0` (placed); the word at `p143+0x18` points at 16 random bytes less `0` (placed); the word at `p45+0` points at 32 random bytes less `0` (placed); the word at `p145+0x18` points at 16 random bytes less `0` (placed); the word at `p45+4` points at 32 random bytes less `0` (placed); the word at `p147+0x18` points at 16 random bytes less `0` (placed); the word at `p47+0` points at 32 random bytes less `0` (placed); the word at `p149+0x18` points at 16 random bytes less `0` (placed); the word at `p47+4` points at 32 random bytes less `0` (placed); the word at `p151+0x18` points at 16 random bytes less `0` (placed); the word at `p48+0` points at 16 random bytes less `0` (placed); the word at `p48+4` points at 16 random bytes less `0` (placed); the word at `p48+8` points at 16 random bytes less `0` (placed); the word at `p48+12` points at 16 random bytes less `0` (placed); the word at `p49+0` points at 16 random bytes less `0` (placed); the word at `p49+4` points at 16 random bytes less `0` (placed); the word at `p49+8` points at 16 random bytes less `0` (placed); the word at `p49+12` points at 16 random bytes less `0` (placed); the word at `p50+0` points at 16 random bytes less `0` (placed); the word at `p50+4` points at 16 random bytes less `0` (placed); the word at `p50+8` points at 16 random bytes less `0` (placed); the word at `p50+12` points at 16 random bytes less `0` (placed); the word at `p51+0` points at 16 random bytes less `0` (placed); the word at `p51+4` points at 16 random bytes less `0` (placed); the word at `p51+8` points at 16 random bytes less `0` (placed); the word at `p51+12` points at 16 random bytes less `0` (placed); the word at `p52+0` points at 16 random bytes less `0` (placed); the word at `p52+4` points at 16 random bytes less `0` (placed); the word at `p52+8` points at 16 random bytes less `0` (placed); the word at `p52+12` points at 16 random bytes less `0` (placed); the word at `p53+0` points at 16 random bytes less `0` (placed); the word at `p53+4` points at 16 random bytes less `0` (placed); the word at `p53+8` points at 16 random bytes less `0` (placed); the word at `p53+12` points at 16 random bytes less `0` (placed); the word at `p54+0` points at 16 random bytes less `0` (placed); the word at `p54+4` points at 16 random bytes less `0` (placed); the word at `p54+8` points at 16 random bytes less `0` (placed); the word at `p54+12` points at 16 random bytes less `0` (placed); the word at `p55+0` points at 16 random bytes less `0` (placed); the word at `p55+4` points at 16 random bytes less `0` (placed); the word at `p55+8` points at 16 random bytes less `0` (placed); the word at `p55+12` points at 16 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0x2c laid as 0x00472320 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 0, 1] (90%); stub 6 answered from [0, 1, 1, 1] (90%); stub 7 answered from [0, 0, 0, 1, 2] (90%); stub 8 answered from [0, 1, 1, 1, 2, 3] (90%); stub 9 answered from [-1, -1, 0, 7, 3] (90%); stub 5 answers a window of its own to each call, in call order (128 bytes, ...); the word at `p56+0` laid as the image vtable at 0xb42b94 (the stub slots copied there) half the rows; stub 4 answers 14 values in call order, printed after the answers; stub 6 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4, +0x424:0x4, +0x854:0x4, +0x5720:0x48 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0ab84`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0aba0`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0abbc`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0abd8`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0abf4`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0ac10`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0ac2c`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 2 rows 0x1f40 apart at `0xc0ac48`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a390`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a3ac`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a3c8`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a3e4`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a400`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a41c`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a438`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc0a454`, slices +0x0:0x10 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p57+0` as well as at `this`; the stubbed vtable laid at `p59+0` as well as at `this`; the stubbed vtable laid at `p61+0` as well as at `this`; the stubbed vtable laid at `p63+0` as well as at `this`; the stubbed vtable laid at `p65+0` as well as at `this`; the stubbed vtable laid at `p67+0` as well as at `this`; the stubbed vtable laid at `p69+0` as well as at `this`; the stubbed vtable laid at `p71+0` as well as at `this`; the stubbed vtable laid at `p73+0` as well as at `this`; the stubbed vtable laid at `p75+0` as well as at `this`; the stubbed vtable laid at `p77+0` as well as at `this`; the stubbed vtable laid at `p79+0` as well as at `this`; the stubbed vtable laid at `p81+0` as well as at `this`; the stubbed vtable laid at `p83+0` as well as at `this`; the stubbed vtable laid at `p85+0` as well as at `this`; the stubbed vtable laid at `p87+0` as well as at `this`; the stubbed vtable laid at `p89+0` as well as at `this`; the stubbed vtable laid at `p91+0` as well as at `this`; the stubbed vtable laid at `p93+0` as well as at `this`; the stubbed vtable laid at `p95+0` as well as at `this`; the stubbed vtable laid at `p97+0` as well as at `this`; the stubbed vtable laid at `p99+0` as well as at `this`; the stubbed vtable laid at `p101+0` as well as at `this`; the stubbed vtable laid at `p103+0` as well as at `this`; the stubbed vtable laid at `p105+0` as well as at `this`; the stubbed vtable laid at `p107+0` as well as at `this`; the stubbed vtable laid at `p109+0` as well as at `this`; the stubbed vtable laid at `p111+0` as well as at `this`; the stubbed vtable laid at `p113+0` as well as at `this`; the stubbed vtable laid at `p115+0` as well as at `this`; the stubbed vtable laid at `p117+0` as well as at `this`; the stubbed vtable laid at `p119+0` as well as at `this`; the stubbed vtable laid at `p121+0` as well as at `this`; the stubbed vtable laid at `p123+0` as well as at `this`; the stubbed vtable laid at `p125+0` as well as at `this`; the stubbed vtable laid at `p127+0` as well as at `this`; the stubbed vtable laid at `p129+0` as well as at `this`; the stubbed vtable laid at `p131+0` as well as at `this`; the stubbed vtable laid at `p133+0` as well as at `this`; the stubbed vtable laid at `p135+0` as well as at `this`; the stubbed vtable laid at `p137+0` as well as at `this`; the stubbed vtable laid at `p139+0` as well as at `this`; the stubbed vtable laid at `p141+0` as well as at `this`; the stubbed vtable laid at `p143+0` as well as at `this`; the stubbed vtable laid at `p145+0` as well as at `this`; the stubbed vtable laid at `p147+0` as well as at `this`; the stubbed vtable laid at `p149+0` as well as at `this`; the stubbed vtable laid at `p151+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_construct_time.py") else {
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
        let log: Vec<String>;
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Windows: m the building (`+9` owner, `+0xa` id, `+0x18` type, `+0x50` base, `+0x72` city);
                // c0 the constants (`+0x44c` hanging_gardens_build_time, `+0xc90` thepresident_building_speed,
                // `+0x834` iroquois_quick_senate), c1 the eight object counts; p[4k..4k+4] the leader k's
                // flags, wonder_mark (`+0x424`), wonders_built (`+0x854`) and the short table (`+0x5720`..);
                // p56 the type; p[57+2(4k+e)] / +1 the leader k's own object e and its type, p[121+2(2k+e)] /
                // +1 its object 2000+e and its type, p[153+4k+e] its wonder record e, p[185+j] the j-th
                // record vslot +0x3c answers. `a[0]` is param_1, `a[1+i]` stub i's answer, `a[11..25]` slot
                // +0x4c's fourteen, `a[25..27]` has_tribe_bonus's two, `a[27]` / `a[28]` the slot +0xb8 /
                // +0x2c holds the image's `ObjectData::is` / `is_wonder`. The crate's `build::construct_time`
                // is driven with the four clauses' inputs derived inline from the stubs' answers (the crate
                // derives only two of the four; `ClockMods::free_first_wonder` / `hanging_gardens` have no
                // writer in `Sim`).
                use crate::Tuning;
                use crate::build::{BuildType, ClockMods, Ident, construct_time};
                fn i16at(w: &[u8], o: usize) -> i16 {
                    i16::from_le_bytes([w[o], w[o + 1]])
                }
                struct Q<'a> {
                    a: &'a [i32],
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    cols: Vec<Vec<String>>,
                    s4: usize,
                    s6: usize,
                    s3c: usize,
                }
                impl Q<'_> {
                    fn who(&self) -> usize {
                        self.m[9] as usize
                    }
                    fn push(&mut self, col: usize, s: String) {
                        self.cols[col].push(s);
                    }
                    fn is_wonder(&mut self) -> bool {
                        if self.a[28] != 0 {
                            if rd(&self.p[56], 0) as u32 == 0xb42b94 {
                                (0x20e..=0x21e).contains(&rd(&self.p[56], 4))
                            } else {
                                self.push(2, "p56+0x0".into());
                                self.a[3] != 0
                            }
                        } else {
                            self.push(0, "m+0x0".into());
                            self.a[1] != 0
                        }
                    }
                    fn tribe(&mut self, arg: i32) -> bool {
                        let w = self.who();
                        self.push(6 + w, format!("r+0x0,{arg}"));
                        let v = self.a[25 + self.s6.min(1)];
                        self.s6 += 1;
                        v != 0
                    }
                    fn get_wonders(&mut self) -> i32 {
                        let w = self.who();
                        self.push(15 + w, "r+0x0".into());
                        self.a[8]
                    }
                    fn flag(&self, l: usize) -> bool {
                        self.p[4 * l][0] & 1 != 0
                    }
                    fn built(&self, l: usize) -> i32 {
                        rd(&self.p[4 * l + 2], 0)
                    }
                    fn quick_short(&self, l: usize, t: i32) -> i16 {
                        let off = 2 * (i64::from(t) - 0x1fe);
                        if (0..0x48).contains(&off) {
                            i16at(&self.p[4 * l + 3], off as usize)
                        } else {
                            0
                        }
                    }
                    fn rival(&mut self, tyidx: i32) -> bool {
                        let who = self.who();
                        let id = i32::from(i16at(self.m, 0xa));
                        for l in 0..8 {
                            let cnt = rd(&self.c[1], 4 * l);
                            if self.flag(l) && l != who && cnt > 2000 {
                                for iv in 2000..cnt {
                                    let r = 121 + 2 * (2 * l + (iv - 2000) as usize);
                                    if self.p[r][8] & 1 != 0 {
                                        self.push(4, format!("p{r}+0x0"));
                                        let ans = self.a[11 + self.s4.min(13)];
                                        self.s4 += 1;
                                        if ans == 0
                                            && (iv != id || l != who)
                                            && rd(&self.p[r + 1], 4) == tyidx
                                        {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                        false
                    }
                    fn gardens(&mut self, city: i16) -> bool {
                        let who = self.who();
                        for i in 0..rd(&self.p[4 * who + 1], 0) as usize {
                            let wr = &self.p[153 + 4 * who + i];
                            if wr[0xc] & 1 != 0 {
                                let oi = i16at(wr, 2) as usize;
                                let obj = 57 + 2 * (4 * who + oi);
                                if rd(&self.p[obj + 1], 4) == 0x210 {
                                    self.push(5, format!("p{obj}+0x0"));
                                    let r = 185 + self.s3c.min(3);
                                    self.s3c += 1;
                                    if i16at(&self.p[r], 0x72) == city {
                                        return true;
                                    }
                                }
                            }
                        }
                        false
                    }
                }
                let (m, c, p) = (&*m, &*c, &*p);
                let mut q = Q {
                    a,
                    m,
                    c,
                    p,
                    cols: vec![Vec::new(); 34],
                    s4: 0,
                    s6: 0,
                    s3c: 0,
                };
                let who = m[9] as usize;
                let city = i16at(m, 0x72);
                let base = rd(m, 0x50);
                let tyidx = rd(&p[56], 4);
                let mut t = Tuning::RON;
                t.hanging_gardens_build_time = rd(&c[0], 0x44c);
                t.thepresident_building_speed = rd(&c[0], 0xc90);
                t.iroquois_quick_senate = rd(&c[0], 0x834);
                let wonder = q.is_wonder();
                let mut mods = ClockMods::default();
                if wonder
                    && q.tribe(0x14)
                    && q.get_wonders() == 0
                    && q.built(who) == 0
                    && tyidx != 0x21d
                    && tyidx != 0x21e
                {
                    mods.free_first_wonder = !q.rival(tyidx);
                }
                let mut ty = BuildType {
                    wonder,
                    ..BuildType::default()
                };
                if a[0] == 0 {
                    q.push(24 + who, "r+0x0,528".into());
                    let hw = q.a[9];
                    let w2 = q.is_wonder();
                    if !w2 && hw != 0 && hw & 2 == 0 && city >= 0 {
                        mods.hanging_gardens = q.gardens(city);
                    }
                    q.push(33, "m+0x0,0,355".into());
                    mods.president = q.a[10] >= 0;
                    if q.tribe(0x12) {
                        let senate = if q.a[27] != 0 {
                            q.push(3, "p56+0x0,438,0".into());
                            q.a[4] != 0
                        } else {
                            q.push(1, "m+0x0,438,0".into());
                            q.a[2] != 0
                        };
                        if senate {
                            ty.ident = Ident::Senate;
                            mods.iroquois_first_senate = q.quick_short(who, tyidx) == 0;
                        }
                    }
                }
                log = q
                    .cols
                    .iter()
                    .map(|c| {
                        if c.is_empty() {
                            "-".to_string()
                        } else {
                            c.join("/")
                        }
                    })
                    .collect();
                i64::from(construct_time(&t, &ty, base, &mods))
            }
        };
        assert_eq!(got, want_eax, "construct_time — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "construct_time — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "construct_time — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "construct_time — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "construct_time — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("BuildData::construct_time@0062d5c0: {n} rows agree");
}
