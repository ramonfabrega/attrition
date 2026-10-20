//! `Unit::suffer_attrition@005e1a10` under the emulator against the port (item 3372; tools/sweep_gen.py, obj with windows).

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

/// `Unit::suffer_attrition@005e1a10`, run by `tools/emu/sweep_unit_suffer_attrition.py` under unicorn
/// (tools/emu): 64 random bytes of object, 1 argument(s), `p` windows in order: p0..p7 sparse table 0 (4 rows × 2 slice(s), row-major), p8..p9 the place(s), in flag order; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 512 random bytes; the global cell 0x00c06210 points at 768 random bytes; the global cell 0x00c061ec points at 1536 random bytes; 4 random bytes at 0x00c06208; 4 random bytes at 0x00c8cd00; callee 0x006edb50 (1 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x006e33a0 (2 args) stubbed; callee 0x0060a760 (1 args) stubbed; vtable slot +0x16c (9 args) stubbed; callee 0x00608e20 (0 args) stubbed; the word at `m+0x18` points at 800 random bytes less `0` (placed); the word at `c0+0x134` points at 4096 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 5 answers 1..18 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 4 rows 0x6eec apart at `@0xe3a390`, slices +0xc:0x4, +0x1f8:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_suffer_attrition.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Windows: `m` the unit; `c[0]` the world (`0xc06188`: width +0), `c[1]` the `div_3_table`, `c[2]` the console
                // (`who` +0x298), `c[3]` the game (`frame` +0x550), `c[4]` the message window's word (`0xc06208`), `c[5]` the
                // string table's base (`0xc8cd00`); `p[2r]` leader `r`'s kind (+0xc), `p[2r+1]` its warning stamp (+0x1f8),
                // `p[8]` the unit's type (`uber_size` +0x308), `p[9]` the world's cells. The answers `a`: `a[0]` the
                // argument, `a[1+stub]` the stubs' single answers (0 `is_ally`, 4 `type_avail`, 5 `curr_uber_size`, 6 the hurt).
                use crate::suffer_view::{Hurt, SufferView, suffer_attrition_steps};
                const ENC: i32 = 0x63637;
                struct V<'a> {
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    a: &'a [i32],
                    /// The calls of stub `s`, one string a call.
                    calls: [Vec<String>; 8],
                }
                impl V<'_> {
                    fn xy(&self) -> (i32, i32) {
                        (rd(self.m, 0x10) ^ ENC, rd(self.m, 0x14) ^ ENC)
                    }
                    fn ecx_msg(&self) -> i32 {
                        rd(&self.c[4], 0)
                    }
                }
                impl SufferView for V<'_> {
                    fn player(&self) -> usize {
                        self.m[9] as usize
                    }
                    fn warned_at(&self, who: usize) -> i32 {
                        rd(&self.p[2 * who + 1], 0)
                    }
                    fn stamp_warned(&mut self, who: usize) {
                        let f = rd(&self.c[3], 0x550);
                        wr(&mut self.p[2 * who + 1], 0, f);
                    }
                    fn console_who(&self) -> i32 {
                        rd(&self.c[2], 0x298)
                    }
                    fn tile_owner(&self) -> i32 {
                        let (x, y) = self.xy();
                        let d3 = |k: i32| rd(&self.c[1], 4 * (k >> 8) as usize);
                        let w = rd(&self.c[0], 0);
                        let cell = (d3(y) * w + d3(x)) as usize * 28;
                        i32::from(self.p[9][cell + 0xf] as i8)
                    }
                    fn is_ally(&mut self, who: usize, other: usize) -> bool {
                        self.calls[0].push(format!("{who}:r+0x0,{other}"));
                        self.a[1] != 0
                    }
                    fn leader_kind(&self, who: i32) -> i32 {
                        rd(&self.p[2 * who as usize], 0)
                    }
                    fn type_avail(&mut self, who: usize, id: i32) -> bool {
                        self.calls[4].push(format!("{who}:r+0x0,{id},1"));
                        self.a[5] != 0
                    }
                    fn message(&mut self, text: i32, shown: i32, red: bool) {
                        let base = rd(&self.c[5], 0);
                        self.calls[1].push(format!("15558448,{}", base.wrapping_add(text)));
                        let (x, y) = self.xy();
                        let colour = if red { 13161032 } else { 13161068 };
                        self.calls[2].push(format!(
                            "{},15558448,{x},{y},{shown},{colour},-1,{y},0,0,{y}",
                            self.ecx_msg()
                        ));
                    }
                    fn play(&mut self, sound: i32) {
                        // The stub leaves `ecx` as `add_message` had it.
                        self.calls[3].push(format!("{},{sound}", self.ecx_msg()));
                    }
                    fn type_uber_one(&self) -> bool {
                        rd(&self.p[8], 0x308) == 1
                    }
                    fn curr_uber_size(&mut self) -> i32 {
                        self.calls[5].push("m+0x0,0".into());
                        self.a[6]
                    }
                    fn hurt(&mut self, h: Hurt) -> bool {
                        self.calls[6].push(format!(
                            "m+0x0,{},{},1,-1,1,1431655765,-1,-1,0",
                            h.whole, h.sixteenths
                        ));
                        self.a[7] != 0
                    }
                    fn graphic(&mut self) {
                        self.calls[7].push("m+0x0".into());
                    }
                }
                let mut v = V {
                    m,
                    c,
                    p,
                    a,
                    calls: Default::default(),
                };
                let ret = suffer_attrition_steps(&mut v, a[0] != 0);
                // The split stubs' tokens: a token a leader row, then the calls outside the table.
                for s in 0..8 {
                    let tokens = std::mem::take(&mut v.calls[s]);
                    let join = |t: &[String]| {
                        if t.is_empty() {
                            "-".to_string()
                        } else {
                            t.join("/")
                        }
                    };
                    if s == 0 || s == 4 {
                        for r in 0..4 {
                            let row: Vec<String> = tokens
                                .iter()
                                .filter_map(|t| t.strip_prefix(&format!("{r}:")))
                                .map(str::to_string)
                                .collect();
                            log.push(join(&row));
                        }
                        log.push("-".into());
                    } else {
                        log.push(join(&tokens));
                    }
                }
                i64::from(ret)
            }
        };
        assert_eq!(got, want_eax, "suffer_attrition — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "suffer_attrition — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "suffer_attrition — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "suffer_attrition — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[7 + i],
                "suffer_attrition — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::suffer_attrition@005e1a10: {n} rows agree");
}
