//! `Wall::do_construct@006434d0` under the emulator against the port (item 3289; tools/sweep_gen.py, obj with windows).

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

/// `Wall::do_construct@006434d0`, run by `tools/emu/sweep_wall_do_construct.py` under unicorn
/// (tools/emu): 112 random bytes of object, 1 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p9 the place(s), in flag order, p10 stub 1's answer window; the global cell 0x00c061c0 points at 4 random bytes; the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c0620c points at 576 random bytes; 4 random bytes at 0x00e40cb8; 4 random bytes at 0x00e47ba4; callee 0x00636a50 (5 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x007382b0 (1 args) stubbed; vtable slot +0x1a4 (1 args) stubbed; vtable slot +0x18c (1 args) stubbed; vtable slot +0x1a8 (3 args) stubbed; callee 0x6455c0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x007e9b70 (9 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x00a1d590 (1 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1ee30 (0 args) stubbed; callee 0x00a1d440 (1 args) stubbed; callee 0x00a1cd60 (2 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; vtable slot +0x2c (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; vtable slot +0x10c (1 args) stubbed; the word at `c1+0x14` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 96 random bytes less `0` (placed); the word at `p2+4` points at 96 random bytes less `0` (placed); the word at `p2+8` points at 96 random bytes less `0` (placed); the word at `p2+12` points at 96 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `m+0x18` points at 128 random bytes less `0` (placed); slot +0x50 laid as 0x00472360 or its stub, a flag after the answers; slot +0x10c laid as 0x00645310 or its stub, a flag after the answers; stub 0 answered from [0, 39, 40, 41, 42, 43, 42, 42, 5, 44, 1] (90%); stub 2 answered from [0, 1, 2] (90%); stub 3 answered from [0, 1, 2, 3, -1, 2147483647] (90%); stub 5 answered from [1000, 100, 5000, 0, -1, 7] (90%); stub 18 answered from [0, 1] (90%); stub 19 answered from [0, 1, 2, 6] (90%); stub 1 answers the address of 144 random bytes, a `p` window after the places; the word at `c1+0x30` laid as the address of `p2` every row; stub 5 answers 3 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 1024 bytes under the call is zeroed before each row; 2 rows 0x1c apart at `@0xc09960`, slices +0x10:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_do_construct.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().filter(|t| *t != "-").collect();
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
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The row: `a[0]` the amount, `a[1 + i]` stub i's answer (s0 `blocked_site`, s1 slot
            // `0xac`, s2 `has_tribe_bonus`, s3 `num_wonders`, s4 `0x1a4`, s5 `0x18c`, s6 `0x1a8`,
            // s7 `disband`, s8 `add_message`, s9 `add_event`, s10 `play`, s11..s16 the String
            // calls, s17 `get_neon`, s18 slot `0x2c`, s19 slot `0x50`, s20 slot `0x10c`), then
            // the three `construct_time` answers `a[22..25]` and the two vtable keeps `a[25]`
            // (`0x50` is `is_started`) and `a[26]` (`0x10c` is `say_name`). Windows: `m` the
            // site, `c0` the `ai_speed` cell, `c2` the console, `c3` the scene, `c4`/`c5` the
            // leaders' team-colour bytes; `p3 + idx` the objects table's entries, `p7 + who`
            // the leader's city array, `p9` the build type, `p10` the `BuildData` slot `0xac`
            // answers.
            struct Host<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a mut [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                ct_calls: usize,
            }
            impl Host<'_> {
                fn who(&self) -> usize {
                    usize::from(self.m[9])
                }
                fn idx(&self) -> usize {
                    short(self.m, 0xa) as usize
                }
                fn x(&self) -> i32 {
                    rd(self.m, 0x10) ^ 0x63637
                }
                fn y(&self) -> i32 {
                    rd(self.m, 0x14) ^ 0x63637
                }
            }
            fn short(w: &[u8], off: usize) -> i32 {
                i32::from(i16::from_le_bytes([w[off], w[off + 1]]))
            }
            impl crate::construct::ConstructHost for Host<'_> {
                fn ai_speed(&mut self) -> i32 {
                    rd(&self.c[0], 0)
                }
                fn started(&mut self) -> bool {
                    if self.a[25] != 0 {
                        self.m[8] & 2 != 0
                    } else {
                        self.log.push("s19:m+0x0".into());
                        self.a[20] != 0
                    }
                }
                fn blocked_site(&mut self) -> i32 {
                    self.log.push(format!(
                        "s0:p9+0x0,{},{},{},{},0",
                        self.x(),
                        self.y(),
                        self.m[9],
                        short(self.m, 0xa)
                    ));
                    self.a[1]
                }
                fn wonder_city(&mut self) -> i32 {
                    self.log.push(format!("s1:p{}+0x0", 3 + self.idx()));
                    short(&self.p[10], 0x72)
                }
                fn tribe_bonus_7(&mut self) -> bool {
                    self.log
                        .push(format!("s2:{},7", 0xe3a390 + self.who() * 0x6eec));
                    self.a[3] != 0
                }
                fn num_wonders(&mut self, city: i32) -> i32 {
                    let w = rd(&self.p[7 + self.who()], 4 * city as usize);
                    self.log.push(format!("s3:{w},1"));
                    self.a[4]
                }
                fn start(&mut self) {
                    self.log.push("s4:m+0x0,1".into());
                }
                fn disband(&mut self) {
                    self.log.push("s7:m+0x0,1".into());
                }
                fn console_owns(&mut self) -> bool {
                    self.who() as i32 == rd(&self.c[2], 0x298)
                }
                fn blocked_message(&mut self) {
                    let tc = i32::from(self.c[4 + self.who()][0]);
                    let (x, y) = (self.x(), self.y());
                    self.log.push("s12:15558448,15418236".into());
                    self.log.push("s14:15558448,p9+0x60".into());
                    self.log.push("s14:15558448,74640".into());
                    self.log.push("s14:15558448,74660".into());
                    self.log
                        .push(format!("s17:{},sp-0x28", 0xe39ce0 + 100 * tc));
                    self.log.push(format!(
                        "s8:15267872,15558448,{x},{y},1,{},-1,15558448,0,0,15558448",
                        self.a[18]
                    ));
                    self.log.push("s10:15267872,140".into());
                }
                fn active(&mut self) -> bool {
                    self.m[8] & 4 != 0
                }
                fn helpers(&mut self) -> i32 {
                    i32::from(self.m[0x64])
                }
                fn counted(&mut self) -> bool {
                    self.m[0x61] & 8 != 0
                }
                fn mark_counted(&mut self) {
                    self.log.push(format!("s1:p{}+0x0", 3 + self.idx()));
                    let r = short(&self.p[10], 0x7a) as u16;
                    self.p[10][0x7a..0x7c].copy_from_slice(&r.wrapping_add(1).to_le_bytes());
                    self.m[0x61] |= 8;
                }
                fn bump_helpers(&mut self) {
                    self.m[0x64] = self.m[0x64].wrapping_add(1);
                }
                fn job_counter(&mut self) -> i32 {
                    rd(self.m, 0x48)
                }
                fn add_progress(&mut self, share: i32) {
                    let (c2, c1) = (rd(self.m, 0x4c), rd(self.m, 0x48));
                    wr(self.m, 0x4c, c2.wrapping_add(share));
                    wr(self.m, 0x48, c1.wrapping_add(share));
                }
                fn construct_time(&mut self) -> i32 {
                    self.log.push("s5:m+0x0,0".into());
                    let v = self.a[22 + self.ct_calls.min(2)];
                    self.ct_calls += 1;
                    v
                }
                fn activate(&mut self) {
                    self.log.push("s6:m+0x0,0,1,1".into());
                }
                fn is_city(&mut self) -> bool {
                    self.m[8] & 0x20 != 0
                }
                fn slot_2c(&mut self) -> bool {
                    self.log.push("s18:m+0x0".into());
                    self.a[19] != 0
                }
                fn finish_event(&mut self) {
                    self.log.push(format!(
                        "s9:15267872,{},{},1,13161068,1,3072,0,0,0",
                        self.x(),
                        self.y()
                    ));
                }
                fn finish_begin(&mut self) {
                    self.log.push("s11:sp-0x30,15418236".into());
                }
                fn finish_text(&mut self) {
                    self.log.push("s12:sp-0x30,74680".into());
                    self.log.push("s13:sp-0x44".into());
                    if self.a[26] != 0 {
                        self.log.push("s14:sp-0x44,p9+0x74".into());
                    } else {
                        self.log.push("s20:m+0x0,sp-0x44".into());
                    }
                    self.log.push("s15:sp-0x30,sp-0x58,sp-0x44".into());
                    self.log.push(format!("s12:sp-0x30,{}", self.a[16]));
                    self.log.push("s16:sp-0x58".into());
                    self.log.push("s16:sp-0x44".into());
                }
                fn finish_message(&mut self, a: bool, b: bool) {
                    self.log.push(format!(
                        "s8:15267872,sp-0x30,{},{},{},13161068,{},-8,0,0,-8",
                        self.x(),
                        self.y(),
                        if a { -8 } else { -1 },
                        if b { -3 } else { 1 }
                    ));
                    self.log.push("s16:sp-0x30".into());
                }
                fn set_recalc_builds(&mut self) {
                    self.c[3][0x22c] = 1;
                }
            }
            let mut host = Host {
                a,
                m,
                c,
                p,
                log: &mut log,
                ct_calls: 0,
            };
            i64::from(crate::construct::do_construct_steps(&mut host, a[0]))
        };
        assert_eq!(got, want_eax, "do_construct — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "do_construct — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_construct — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "do_construct — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[7 + i],
                "do_construct — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Wall::do_construct@006434d0: {n} rows agree");
}
