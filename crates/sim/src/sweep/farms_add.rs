//! `Farms::add@008d8a40` under the emulator against the port (item 3281; tools/sweep_gen.py, obj with windows).

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

/// `Farms::add@008d8a40`, run by `tools/emu/sweep_farms_add.py` under unicorn
/// (tools/emu): 64 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p22 the place(s), in flag order, p23..p48 the stubs' answer window(s); the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00c0a914 points at 1024 random bytes; the global cell 0x00c06218 points at 16 random bytes; the global cell 0x00c06184 points at 16 random bytes; 12 random bytes at 0x00c0a908; vtable slot +0xac (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x00739390 (3 args) stubbed; callee 0x007368c0 (1 args) stubbed; callee 0x008d73a0 (2 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x00643440 (2 args) stubbed; callee 0x008e3e70 (6 args) stubbed; callee 0x00866560 (4 args) stubbed; callee 0x00434e10 (1 args) stubbed; callee 0x0055e0b2 (3 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 96 random bytes less `0` (placed); the word at `p2+4` points at 96 random bytes less `0` (placed); the word at `p2+8` points at 96 random bytes less `0` (placed); the word at `p2+12` points at 96 random bytes less `0` (placed); the word at `p0+0` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 12 random bytes less `0` (placed); the word at `p7+0` points at 16 random bytes less `0` (placed); the word at `p7+4` points at 16 random bytes less `0` (placed); the word at `p7+8` points at 16 random bytes less `0` (placed); the word at `p8+0` points at 16 random bytes less `0` (placed); the word at `p8+4` points at 16 random bytes less `0` (placed); the word at `p8+8` points at 16 random bytes less `0` (placed); the word at `p3+0x40` points at 128 random bytes less `0` (placed); the word at `p4+0x40` points at 128 random bytes less `0` (placed); the word at `p5+0x40` points at 128 random bytes less `0` (placed); the word at `p6+0x40` points at 128 random bytes less `0` (placed); the word at `p15+0x18` points at 32 random bytes less `0` (placed); the word at `p16+0x18` points at 32 random bytes less `0` (placed); the word at `p17+0x18` points at 32 random bytes less `0` (placed); the word at `p18+0x18` points at 32 random bytes less `0` (placed); stub 2 answered from [0, 1, 2] (90%); stub 3 answered from [0, 1, 2] (90%); stub 4 answered from [1, 2, 3, 4, 5] (90%); stub 5 answered from [0, 1, 2, 3, 4] (90%); stub 6 answered from [-1, -1, -1, 0, 1, 256, 2, 769] (90%); stub 7 answered from [0, 1, 2, 3, 4, 5, 6, 7, 65535, 32767, -1, -2, -3, 2147483647, -2147483648] (90%); stub 1 answers the address of 16 random bytes, a `p` window after the places; stub 10 answers a window of its own to each call, in call order (4 bytes, ...); the word at `p15+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p16+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p17+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p18+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `c0+0x30` laid as the address of `p2` every row; stub 0 answers `p15+j`, the data of the object its `ecx` is in `p3+j` (j < 4), linked at `+0x74`; stub 7 answers 3 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 8 writes a value in [0, 1, 5, 100, -1, 2147483647, 1073741823] through its argument 0, printed after the answers (`o` in a place); stub 8 writes a value in [0, 1, 5, 100, -1, 2147483647, 1073741823] through its argument 1, printed after the answers (`o` in a place); stub 12 is `__cdecl`: the caller pops its arguments; the stack 1024 bytes under the call is zeroed before each row; 2 rows 0x1c apart at `@0xc09960`, slices +0x10:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_farms_add.py") else {
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
            // The row: `a[0]` who, `a[1]` o, then the stubs' answers `a[2 + i]` (s0..s12), the two
            // `tile_corner` outs `a[15]`, `a[16]` (x, y) and the three draws `a[17..20]`. Windows: `c1`
            // the farm records (0xc0 each), `c4` the counters (`+0` count, `+4` capacity, `+8` the
            // short `increase_size` is handed); `p3 + j` the objects, `p15 + j` the `BuildData` each
            // answers (`+0x72` city, `+0x74` next, `+0x78` farm slot, `+0` the vtable, `+0x18` the
            // build object `p19 + j`), `p9 + 3 * who + city` the `CityData` (`+8` its first member),
            // `p24 + k` the 25 height words.
            struct Host<'a> {
                a: &'a [i32],
                who: usize,
                o: usize,
                log: &'a mut Vec<String>,
                c: &'a mut [Vec<u8>],
                p: &'a mut [Vec<u8>],
                city: usize,
                draws: usize,
                heights: [i32; 25],
                sample: usize,
            }
            fn short(w: &[u8], off: usize) -> i32 {
                i32::from(i16::from_le_bytes([w[off], w[off + 1]]))
            }
            impl crate::farms::FarmsAddHost for Host<'_> {
                fn city(&mut self) -> i32 {
                    self.log.push(format!("s0:p{}+0x0", 3 + self.o));
                    self.log
                        .push(format!("s12:p{}+0x0,sp-0xf0,0,180", 3 + self.o));
                    let c = short(&self.p[15 + self.o], 0x72);
                    if c >= 0 {
                        self.city = c as usize;
                    }
                    c
                }
                fn count_buildings(&mut self, city: i32) -> i32 {
                    self.log.push(format!(
                        "s4:p{}+0x0,417,1,0",
                        9 + 3 * self.who + city as usize
                    ));
                    self.a[6]
                }
                fn count_farms(&mut self, city: i32) -> i32 {
                    self.log.push(format!(
                        "s5:p{}+0x0,p{}+{:#x}",
                        9 + 3 * self.who + city as usize,
                        7 + self.who,
                        4 * city
                    ));
                    self.a[7]
                }
                fn nearest(&mut self) -> i32 {
                    self.log
                        .push(format!("s6:12626160,{},{}", self.o, self.who));
                    self.a[8]
                }
                fn draw(&mut self, _site: &'static str) -> i32 {
                    self.log.push("s7:c3+0x0,0,65535".to_string());
                    let v = self.a[17 + self.draws.min(2)];
                    self.draws += 1;
                    v
                }
                fn tile_corner(&mut self) -> (i32, i32) {
                    self.log.push(format!("s1:p{}+0x0", 3 + self.o));
                    self.log.push("s8:p23+0x0,sp-0x28,sp-0x20".to_string());
                    (self.a[15], self.a[16])
                }
                fn chain_first(&mut self, city: i32) -> i32 {
                    // Slot `+0xac` of the building again, for `+0x72`.
                    self.log.push(format!("s0:p{}+0x0", 3 + self.o));
                    short(&self.p[9 + 3 * self.who + city as usize], 8)
                }
                fn member_is_farm(&mut self, m: i32) -> bool {
                    let m = m as usize;
                    self.log.push(format!("s0:p{}+0x0", 3 + m));
                    if rd(&self.p[15 + m], 0) as u32 == 0x00b4_2174 {
                        self.log.push(format!("s2:p{}+0x0,417,0", 19 + m));
                        self.a[4] != 0
                    } else {
                        self.log.push(format!("s3:p{}+0x0,417,0", 15 + m));
                        self.a[5] != 0
                    }
                }
                fn member_farm_slot(&mut self, m: i32) -> i32 {
                    self.log.push(format!("s0:p{}+0x0", 3 + m as usize));
                    short(&self.p[15 + m as usize], 0x78)
                }
                fn slot_ambience(&mut self, slot: i32) -> bool {
                    self.c[1][slot as usize * 0xc0 + 0xbd] & 4 != 0
                }
                fn member_next(&mut self, m: i32) -> i32 {
                    self.log.push(format!("s0:p{}+0x0", 3 + m as usize));
                    short(&self.p[15 + m as usize], 0x74)
                }
                fn add_ambience(&mut self, x: i32, y: i32) {
                    self.log.push(format!(
                        "s9:{y},1,{x},{y},1124532224,{},{}",
                        self.who, self.o
                    ));
                }
                fn terrain_height(&mut self, row: usize, col: usize, x: i32, y: i32) {
                    self.log.push(format!("s10:c2+0x0,sp-0x34,{x},{y},1"));
                    let w = rd(&self.p[24 + self.sample], 0);
                    self.sample += 1;
                    // `cvtdq2ps`, stored at `terrain_height[col][row]`.
                    self.heights[col * 5 + row] = (w as f32).to_bits() as i32;
                }
                fn slot_count(&mut self) -> i32 {
                    rd(&self.c[4], 0)
                }
                fn slot_valid(&mut self, i: i32) -> bool {
                    self.c[1][i as usize * 0xc0 + 0xbc] != 0
                }
                fn slot_capacity(&mut self) -> i32 {
                    rd(&self.c[4], 4)
                }
                fn grow(&mut self) {
                    let arg = short(&self.c[4], 8);
                    self.log.push(format!("s11:12626180,{arg}"));
                }
                fn store(&mut self, slot: i32, farm_type: u8) {
                    let at = slot as usize * 0xc0;
                    let rec = &mut self.c[1][at..at + 0xc0];
                    rec[8..0xbc].fill(0);
                    wr(rec, 0, self.who as i32);
                    wr(rec, 4, self.o as i32);
                    for (k, h) in self.heights.iter().enumerate() {
                        wr(rec, 0x48 + 4 * k, *h);
                    }
                    rec[0xbc] = 1;
                    rec[0xbd] = farm_type;
                    rec[0xbe] = 0;
                    rec[0xbf] = 0;
                    if slot == rd(&self.c[4], 0) {
                        let n = slot + 1;
                        wr(&mut self.c[4], 0, n);
                    }
                }
            }
            let mut host = Host {
                a,
                who: a[0] as usize,
                o: a[1] as usize,
                log: &mut log,
                c,
                p,
                city: 0,
                draws: 0,
                heights: [0; 25],
                sample: 0,
            };
            i64::from(crate::farms::farms_add_steps(&mut host, a[1]))
        };
        assert_eq!(got, want_eax, "add — eax, row `{line}`");
        assert_eq!(log, want_log, "add — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "add — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "add — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Farms::add@008d8a40: {n} rows agree");
}
