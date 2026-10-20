//! `Build::activate@00623e20` under the emulator against the port (item 3364; tools/sweep_gen.py, obj with windows).

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

/// `Build::activate@00623e20`, run by `tools/emu/sweep_build_activate_head.py` under unicorn
/// (tools/emu): 128 random bytes of object, 3 argument(s), `p` windows in order: p0 the place; the global cell 0x00c061f0 points at 2048 random bytes; 4 random bytes at 0x00e3a390; 4 random bytes at 0x00e3a788; 4 random bytes at 0x00e4127c; 4 random bytes at 0x00e41674; 32 random bytes at 0x00e3fd88; 32 random bytes at 0x00e46c74; 8 random bytes at 0x00cc1950; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x84 (2 args) stubbed; callee 0x00627fe0 (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x00642fc0 (2 args) stubbed; callee 0x0063e4b0 (3 args) stubbed; the word at `m+0x18` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 0 answers 2 values in call order, printed after the answers; stub 1 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the call ends where stub 6 is first entered (the function is swept to that callee); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_activate_head.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(8);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Row: `a[0..3]` captured, announce, counted; `a[3..10]` the seven stubs'
            // answers (slot +0x60, +0xb8, +0x84, `new_library`, `has_tribe_bonus`,
            // `mask_me`, `Wall::activate`), `a[10..12]` slot +0x60's two `is` answers,
            // `a[12..14]` slot +0xb8's.
            // Windows: `m` the building (+8 flags, +9 owner, +0xa index), `c0` the
            // constants (+0x788 `chinese_large_cities`), `c1`/`c3` the leaders' flags
            // (0xe3a390, +0x6eec), `c2`/`c4` their `+0x3f8` cities, `c5`/`c6` their
            // `+0x59f8` last-built tables, `c7` `last_build_built_o`, `p0` the type
            // (+4 its index).
            //
            // **The call ends where `Wall::activate` is first entered** (`--stop-stub`):
            // the sweep holds the head (`:121-159`) and nothing after it. **The port is
            // the crate's own steps**, `crate::activate::activate_head_steps`, which
            // `Sim::activate` runs over its `SimHead`; here the view is the emulated
            // original's windows.
            use crate::activate::{ActivateHeadView, activate_head_steps};
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a mut [Vec<u8>],
                p: &'a [Vec<u8>],
                log: &'a mut Vec<String>,
                is_calls: usize,
            }
            impl V<'_> {
                fn owner(&self) -> usize {
                    usize::from(self.m[9])
                }
                fn leader(&self) -> String {
                    // `leaders + owner * 0x6eec` is window c1 (owner 0) or c3 (owner 1).
                    format!("c{}+0x0", 1 + 2 * self.owner())
                }
            }
            impl ActivateHeadView for V<'_> {
                fn is_type(&mut self, id: i32) -> bool {
                    let k = self.is_calls.min(1);
                    self.is_calls += 1;
                    if rd(self.m, 0) == 0x00b4_2174 {
                        self.log.push(format!("s0:p0+0x0,{id},0"));
                        self.a[10 + k] != 0
                    } else {
                        self.log.push(format!("s1:m+0x0,{id},0"));
                        self.a[12 + k] != 0
                    }
                }
                fn new_library(&mut self) {
                    self.log.push("s3:m+0x0".to_string());
                }
                fn clear_recharging(&mut self) {
                    self.m[0x7a] = 0;
                    self.m[0x7b] = 0;
                }
                fn city_flag(&mut self) -> bool {
                    self.m[8] & 0x20 != 0
                }
                fn cities_add(&mut self, d: i32) {
                    let o = self.owner();
                    let at = &mut self.c[2 + 2 * o];
                    let w = rd(at, 0).wrapping_add(d);
                    wr(at, 0, w);
                }
                fn tribe_bonus(&mut self, bonus: i32) -> bool {
                    let who = self.leader();
                    self.log.push(format!("s4:{who},{bonus}"));
                    self.a[7] != 0
                }
                fn chinese_large_cities(&mut self) -> bool {
                    rd(&self.c[0], 0x788) != 0
                }
                fn set_type(&mut self, id: i32) {
                    self.log.push(format!("s2:m+0x0,{id},0"));
                }
                fn mask_me(&mut self) {
                    self.log.push("s5:m+0x0,1,0".to_string());
                }
                fn flag_wall_stats(&mut self) {
                    let o = self.owner();
                    let at = &mut self.c[1 + 2 * o];
                    let w = rd(at, 0) | 0x0800_0000;
                    wr(at, 0, w);
                }
                fn record_last_built(&mut self) {
                    let o = self.owner();
                    let idx = rd(&self.p[0], 4) as usize;
                    let short = i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]));
                    wr(&mut self.c[5 + o], idx * 4, short);
                    wr(&mut self.c[7], o * 4, short);
                }
                fn wall_activate(&mut self, captured: i32, announce: i32) {
                    self.log.push(format!("s6:m+0x0,{captured},{announce},0"));
                }
            }
            let mut v = V {
                a,
                m,
                c,
                p,
                log: &mut log,
                is_calls: 0,
            };
            activate_head_steps(&mut v, a[0], a[1], a[2]);
            i64::from(a[9])
        };
        assert_eq!(got, want_eax, "activate — eax, row `{line}`");
        assert_eq!(log, want_log, "activate — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "activate — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "activate — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[9 + i], "activate — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Build::activate@00623e20: {n} rows agree");
}
