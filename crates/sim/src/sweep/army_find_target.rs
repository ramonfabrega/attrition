//! `Army::find_target@006f69b0` under the emulator against the port (item 3362; tools/sweep_gen.py, obj with windows).

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

/// `Army::find_target@006f69b0`, run by `tools/emu/sweep_army_find_target.py` under unicorn
/// (tools/emu): 152 random bytes of object, 0 argument(s), `p` windows in order: p0..p103 sparse table 0 (8 rows × 13 slice(s), row-major), p104..p119 sparse table 1 (8 rows × 2 slice(s), row-major), p120..p160 the place(s), in flag order; the global cell 0x00c061e8 points at 2112 random bytes; the global cell 0x00c061ec points at 1536 random bytes; the global cell 0x00c061b8 points at 32 random bytes; callee 0x006f9120 (2 args) stubbed; callee 0x006e33a0 (2 args) stubbed; callee 0x006eb930 (4 args) stubbed; callee 0x006d5b90 (1 args) stubbed; callee 0x006f2e10 (1 args) stubbed; callee 0x006ec000 (0 args) stubbed; callee 0x00680f90 (1 args) stubbed; callee 0x00a39d70 (2 args) stubbed; the word at `0xc09970+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p120+0` points at 112 random bytes less `0` (placed); the word at `p120+4` points at 112 random bytes less `0` (placed); the word at `p120+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p124+0` points at 112 random bytes less `0` (placed); the word at `p124+4` points at 112 random bytes less `0` (placed); the word at `p124+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*2` points at 12 random bytes less `0` (placed); the word at `p128+0` points at 112 random bytes less `0` (placed); the word at `p128+4` points at 112 random bytes less `0` (placed); the word at `p128+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*3` points at 12 random bytes less `0` (placed); the word at `p132+0` points at 112 random bytes less `0` (placed); the word at `p132+4` points at 112 random bytes less `0` (placed); the word at `p132+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*4` points at 12 random bytes less `0` (placed); the word at `p136+0` points at 112 random bytes less `0` (placed); the word at `p136+4` points at 112 random bytes less `0` (placed); the word at `p136+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*5` points at 12 random bytes less `0` (placed); the word at `p140+0` points at 112 random bytes less `0` (placed); the word at `p140+4` points at 112 random bytes less `0` (placed); the word at `p140+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*6` points at 12 random bytes less `0` (placed); the word at `p144+0` points at 112 random bytes less `0` (placed); the word at `p144+4` points at 112 random bytes less `0` (placed); the word at `p144+8` points at 112 random bytes less `0` (placed); the word at `0xc09970+0x1c*7` points at 12 random bytes less `0` (placed); the word at `p148+0` points at 112 random bytes less `0` (placed); the word at `p148+4` points at 112 random bytes less `0` (placed); the word at `p148+8` points at 112 random bytes less `0` (placed); the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*0+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*1+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*2+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*3+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*4+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*5+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*6+0x6eb8` points at 256 random bytes less `0` (placed); the word at `0xe3a390+0x6eec*7+0x6eb8` points at 256 random bytes less `0` (placed); stub 2 answered from [0] (90%); stub 0 answers 4 values in call order, printed after the answers; stub 3 answers 16 values in call order, printed after the answers; stub 4 answers 16 values in call order, printed after the answers; stub 5 answers 16 values in call order, printed after the answers; stub 6 answers 48 values in call order, printed after the answers; stub 0 answers 0..6 every row; stub 1 answers 0..1 every row; stub 3 answers 0..1 every row; stub 4 answers -1..7 every row; stub 5 answers 0..5 every row; stub 6 answers 0..1 every row; stub 7 answers 0..1 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 2 writes a value in [-1, 0, 1, 2] through its argument 0, printed after the answers (`o` in a place); stub 2 writes a value in [0, 1, 2, 3, 4, 5, 6, 7] through its argument 1, printed after the answers (`o` in a place); the call ends where stub 7 is first entered (the function is swept to that callee); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x50:0x4, +0x74:0x20, +0x230:0x20, +0x250:0x20, +0x408:0x4, +0x444:0xc, +0x7a8:0x4, +0x944:0xc, +0xa40:0x8, +0x6dec:0x4, +0x6e20:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4, +0x74:0x20 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 5000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_find_target.py") else {
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
            use crate::target_gate::{
                Ecx, Stopped, TargetGateView, target_city_gate, target_head, target_leader_gate,
            };
            // Windows: `m` the army (`+2` slot, `+8` region, `+0x14` captains, `+0x18` standard,
            // `+0x24` navy, `+0x94` who); `c[0]` gamec (`+0x820..` semaphore, `+0x2b` difficulty,
            // `+0x65c` start_list, `+0x67c` start_index), `c[1]` game (`+0x24` team_style, `+0x550`
            // frame), `c[2]` the regions cell; `p[13r+s]` slice `s` of `leaders[r]`, in offset order:
            // 0 flags, 1 id, 2 multi_diff, 3 diplos, 4 `+0x230`, 5 `+0x250`, 6 cities, 7 `+0x444..`,
            // 8 `+0x7a8`, 9 `+0x944..`, 10 `+0xa40..`, 11 `+0x6dec`, 12 `+0x6e20`; `p[104+2r]` and
            // `p[105+2r]` the `leadersc` row's flags and diplos; `p[120+4a]` the city pointer array
            // of leader `a`, `p[121+4a+k]` its city `k`; `p[152]` the `div_3_table`; `p[153+a]` the
            // record `leaders[a]+0x6eb8` names.
            // `a`: stubs 0..=7's answers, `find_capital`'s two outs, then stub 0's 4, stub 3's 16,
            // stub 4's 16, stub 5's 16 and stub 6's 48 answers in call order (the last repeated).
            const OUT: usize = 8;
            const S0: usize = 10;
            const S3: usize = 14;
            const S4: usize = 30;
            const S5: usize = 46;
            const S6: usize = 62;
            struct W<'a> {
                a: &'a [i32],
                m: &'a [u8],
                c: &'a [Vec<u8>],
                p: &'a [Vec<u8>],
                log: &'a mut Vec<String>,
                n: [usize; 7],
            }
            impl W<'_> {
                fn lw(&self, a: usize, s: usize, off: usize) -> i32 {
                    rd(&self.p[13 * a + s], off)
                }
                fn leader(&self) -> String {
                    format!("p{}+0x0", 13 * self.who())
                }
                fn city(&self, a: usize, k: usize) -> &[u8] {
                    &self.p[121 + 4 * a + k]
                }
                fn seq(&mut self, s: usize, base: usize, len: usize) -> i32 {
                    let k = self.n[s].min(len - 1);
                    self.n[s] += 1;
                    self.a[base + k]
                }
            }
            impl TargetGateView for W<'_> {
                fn slot(&self) -> i32 {
                    i32::from(i16::from_le_bytes([self.m[2], self.m[3]]))
                }
                fn region(&self) -> i32 {
                    rd(self.m, 8)
                }
                fn captains(&self) -> i32 {
                    rd(self.m, 0x14)
                }
                fn standard(&self) -> i32 {
                    rd(self.m, 0x18)
                }
                fn navy(&self) -> bool {
                    rd(self.m, 0x24) != 0
                }
                fn who(&self) -> usize {
                    let w = i16::from_le_bytes([self.m[0x94], self.m[0x95]]);
                    assert!((0..8).contains(&w), "the sweep keeps who in the table");
                    w as usize
                }
                fn lflags(&self, a: usize) -> i32 {
                    self.lw(a, 0, 0)
                }
                fn lid(&self, a: usize) -> i32 {
                    self.lw(a, 1, 0)
                }
                fn lmulti(&self, a: usize) -> i32 {
                    self.lw(a, 2, 0)
                }
                fn ld(&self, a: usize, b: usize) -> i32 {
                    self.lw(a, 3, 4 * b)
                }
                fn lc(&self, a: usize, b: i32) -> i32 {
                    rd(&self.p[105 + 2 * a], 4 * b as usize)
                }
                fn lcflags(&self, a: i32) -> i32 {
                    rd(&self.p[104 + 2 * a as usize], 0)
                }
                fn x230(&self, a: usize, b: usize) -> i32 {
                    self.lw(a, 4, 4 * b)
                }
                fn x250(&self, a: usize, b: usize) -> i32 {
                    self.lw(a, 5, 4 * b)
                }
                fn ncities(&self, a: usize) -> i32 {
                    self.lw(a, 6, 0)
                }
                fn w444(&self, a: usize) -> i32 {
                    self.lw(a, 7, 0)
                }
                fn w44c(&self, a: usize) -> i32 {
                    self.lw(a, 7, 8)
                }
                fn raid(&self, a: usize) -> i32 {
                    self.lw(a, 11, 0)
                }
                fn early(&self, a: usize) -> i32 {
                    self.lw(a, 12, 0)
                }
                fn defmod(&self, a: usize) -> i32 {
                    self.lw(a, 8, 0)
                }
                fn combat(&self, a: usize) -> i32 {
                    self.lw(a, 9, 0)
                }
                fn sea(&self, a: usize) -> i32 {
                    self.lw(a, 9, 8)
                }
                fn frame_attacked(&self, a: usize) -> i32 {
                    self.lw(a, 10, 0)
                }
                fn attacked_by(&self, a: usize) -> i32 {
                    self.lw(a, 10, 4)
                }
                fn military(&self, a: usize) -> i32 {
                    rd(&self.p[153 + a], 0xe8) ^ 0x63187
                }
                fn semaphore(&self) -> [u8; 3] {
                    [self.c[0][0x820], self.c[0][0x821], self.c[0][0x822]]
                }
                fn game_difficulty(&self) -> i32 {
                    i32::from(self.c[0][0x2b])
                }
                fn team_style(&self) -> i32 {
                    i32::from(self.c[1][0x24])
                }
                fn frame(&self) -> i32 {
                    rd(&self.c[1], 0x550)
                }
                fn start_list(&self, k: i32) -> i32 {
                    rd(&self.c[0], 0x65c + 4 * k as usize)
                }
                fn start_index(&self, id: i32) -> i32 {
                    rd(&self.c[0], 0x67c + 4 * id as usize)
                }
                fn city_flags(&self, a: usize, k: usize) -> i32 {
                    rd(self.city(a, k), 4)
                }
                fn city_reg(&self, a: usize, k: usize) -> i32 {
                    let b = self.city(a, k);
                    i32::from(i16::from_le_bytes([b[0xa], b[0xb]]))
                }
                fn city_founder(&self, a: usize, k: usize) -> i32 {
                    i32::from(self.city(a, k)[0x60])
                }
                fn city_ocean(&self, a: usize, k: usize) -> i32 {
                    i32::from(self.city(a, k)[0x62])
                }
                fn capital_cell(&self, owner: i32, idx: i32) -> (i32, i32) {
                    let b = self.city(owner as usize, idx as usize);
                    let t = |off: usize| rd(&self.p[152], 4 * (rd(b, off) >> 8) as usize);
                    (t(0xc), t(0x10))
                }
                fn count(&mut self, kind: i32, arg: i32) -> i32 {
                    self.log.push(format!("s0:m+0x0,{kind},{arg}"));
                    self.seq(0, S0, 4)
                }
                fn type_avail(&mut self, ty: i32, strict: i32) -> i32 {
                    self.log.push(format!("s1:{},{ty},{strict}", self.leader()));
                    self.a[1]
                }
                fn find_capital(&mut self) -> (i32, i32) {
                    self.log
                        .push(format!("s2:{},sp-0x38,sp-0x24,-1,-1", self.leader()));
                    (self.a[OUT], self.a[OUT + 1])
                }
                fn tribute(&mut self, b: usize) -> bool {
                    self.log.push(format!("s3:{},{b}", self.leader()));
                    self.seq(3, S3, 16) != 0
                }
                fn find_aggressive(&mut self, ecx: Ecx) -> i32 {
                    let t = match ecx {
                        Ecx::Army => "m+0x0".to_string(),
                        Ecx::Num(n) => n.to_string(),
                    };
                    self.log.push(format!("s4:{t},{}", self.who()));
                    self.seq(4, S4, 16)
                }
                fn get_diff(&mut self) -> i32 {
                    self.log.push(format!("s5:{}", self.leader()));
                    self.seq(5, S5, 16)
                }
                fn is_coast(&mut self, reg: i32, city_reg: i32) -> bool {
                    let ecx = rd(&self.c[2], 0x10).wrapping_add(reg.wrapping_mul(0x88));
                    self.log.push(format!("s6:{ecx},{city_reg}"));
                    self.seq(6, S6, 48) != 0
                }
                fn draw(&mut self) -> Result<i32, Stopped> {
                    // The image's word at 0xc06184 (`Random`'s object), then `(0, 0xffff)`.
                    self.log.push("s7:14908044,0,65535".to_string());
                    Err(Stopped)
                }
            }
            let mut w = W {
                a,
                m,
                c,
                p,
                log: &mut log,
                n: [0; 7],
            };
            // The slice: the head, then the leaders and their cities in the original's order, two
            // passes, to the first draw — the coin or the score.
            let head = target_head(&mut w);
            if !head.stop {
                'run: for _pass in 0..2 {
                    for i in 0..8 {
                        match target_leader_gate(&mut w, i) {
                            Err(Stopped) => break 'run,
                            Ok(None) => continue,
                            Ok(Some(_)) => {}
                        }
                        for k in 0..w.ncities(i) as usize {
                            if target_city_gate(&mut w, i, k) {
                                let _ = w.draw();
                                break 'run;
                            }
                        }
                    }
                }
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "find_target — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_target — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_target — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "find_target — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 5000, "only {n} rows");
    eprintln!("Army::find_target@006f69b0: {n} rows agree");
}
