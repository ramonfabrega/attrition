//! `Wall::update_los@0063eeb0` under the emulator against the port (item 3391; tools/sweep_gen.py, obj with windows).

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

/// `Wall::update_los@0063eeb0`, run by `tools/emu/sweep_wall_update_los.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), `p` windows in order: p0..p23 sparse table 0 (8 rows × 3 slice(s), row-major), p24..p32 the place(s), in flag order; the global cell 0x00c061f0 points at 2560 random bytes; vtable slot +0x50 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x2c (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x006daf30 (0 args) stubbed; callee 0x006daf80 (0 args) stubbed; callee 0x006ebc10 (1 args) stubbed; the word at `p2+0` points at 256 random bytes less `0` (placed); the word at `p5+0` points at 256 random bytes less `0` (placed); the word at `p8+0` points at 256 random bytes less `0` (placed); the word at `p11+0` points at 256 random bytes less `0` (placed); the word at `p14+0` points at 256 random bytes less `0` (placed); the word at `p17+0` points at 256 random bytes less `0` (placed); the word at `p20+0` points at 256 random bytes less `0` (placed); the word at `p23+0` points at 256 random bytes less `0` (placed); the word at `m+0x18` points at 576 random bytes less `0` (placed); slot +0x50 laid as 0x00472360 or its stub, a flag after the answers; slot +0x4c laid as 0x00472350 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 8 answered from [0, 1, 2, 3] (90%); stub 6 answers 1..4 every row; stub 7 answers 1..4 every row; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x6da4:0x30, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p32+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_update_los.py") else {
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
            // hand: port
            // `crate::wall_los_view`: the building's sight over the windows. `m` the building, `c[0]`
            // the constants block, `p[3k..3k+3]` leader k's slices (flags byte; `+0x6da4..`; the
            // tech pointer), `p[24 + k]` its tech record, `p[32]` the type record. The stubs' answers
            // are `a[0..9]`, the three base flags after them (`1` = the base function is laid).
            struct V<'w> {
                m: &'w [u8],
                c: &'w [u8],
                p: &'w [Vec<u8>],
                a: &'w [i32],
                who: usize,
                toks: Vec<String>,
            }
            impl V<'_> {
                fn note(&mut self, stub: usize, args: &str) {
                    let t = &mut self.toks[stub * 9 + self.who];
                    let s = format!("r+0x0{args}");
                    if t == "-" {
                        *t = s;
                    } else {
                        t.push(' ');
                        t.push_str(&s);
                    }
                }
                fn rng(&self, base: usize, i: i32) -> i8 {
                    self.c[base + 4 * i as usize] as i8
                }
            }
            impl crate::wall_los_view::WallLosView for V<'_> {
                fn is_started(&mut self) -> bool {
                    if self.a[9] == 1 {
                        self.m[8] & 2 != 0
                    } else {
                        self.a[0] != 0
                    }
                }
                fn is_active(&mut self) -> bool {
                    if self.a[10] == 1 {
                        self.m[8] & 4 != 0
                    } else {
                        self.a[1] != 0
                    }
                }
                fn is_wonder(&mut self) -> bool {
                    self.a[2] != 0
                }
                fn leader_live(&self) -> bool {
                    self.p[3 * self.who][0] & 1 != 0
                }
                fn type_los(&self) -> i8 {
                    self.p[32][0x21c] as i8
                }
                fn type_science_los(&self) -> i8 {
                    self.p[32][0x220] as i8
                }
                fn type_x_size(&self) -> i32 {
                    rd(&self.p[32], 0x234)
                }
                fn science_epoch_raw(&self) -> u8 {
                    self.p[24 + self.who][0xf4]
                }
                fn is_fort(&mut self) -> bool {
                    self.a[3] != 0
                }
                fn is_tower(&mut self) -> bool {
                    if self.a[11] == 1 {
                        self.a[4] != 0
                    } else {
                        self.a[5] != 0
                    }
                }
                fn fort_los(&mut self) -> i32 {
                    self.note(0, "");
                    self.a[6]
                }
                fn tower_fort_los(&mut self) -> i32 {
                    self.note(1, "");
                    self.a[7]
                }
                fn has_colosseum(&mut self) -> bool {
                    self.note(2, ",530");
                    self.a[8] != 0
                }
                fn fort_upgrade_range(&self, i: i32) -> i8 {
                    self.rng(0x180, i)
                }
                fn tower_fort_range(&self, i: i32) -> i8 {
                    self.rng(0x1a8, i)
                }
                fn colosseum_fort_range(&self) -> i8 {
                    self.c[0x474] as i8
                }
                fn furs_los(&self) -> i8 {
                    self.c[0x914] as i8
                }
                fn has_furs(&self) -> bool {
                    (self.p[3 * self.who + 1][1] | self.p[3 * self.who + 1][0x29]) & 0x80 != 0
                }
            }
            let who = m[9] as usize;
            let mut v = V {
                m,
                c: &c[0],
                p,
                a,
                who,
                toks: vec!["-".to_string(); 27],
            };
            let los = crate::wall_los_view::wall_update_los(&mut v);
            let toks = v.toks;
            log.extend(toks);
            m[0x3c] = los as u8;
            i64::from(los)
            // end hand
        };
        assert_eq!(got, want_eax, "update_los — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "update_los — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_los — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "update_los — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "update_los — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Wall::update_los@0063eeb0: {n} rows agree");
}
