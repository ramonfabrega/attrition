//! `SpellType::cast_ambush@00671610` under the emulator against the port (item 3320; tools/sweep_gen.py, obj with windows).

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

/// `SpellType::cast_ambush@00671610`, run by `tools/emu/sweep_spell_type_cast_ambush.py` under unicorn
/// (tools/emu): 480 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p16 the place(s), in flag order; callee 0x006e0830 (0 args) stubbed; callee 0x0048b150 (1 args) stubbed; callee 0x0097f770 (1 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `0xc061ec` points at 4 random bytes less `0x550` (placed); the word at `0xc0aec0+0x1c*a[1]` points at 8 random bytes less `0` (placed); the word at `p3+4*a[0]` points at 144 random bytes less `0` (placed); the word at `0xc0a130+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 24 random bytes less `0` (placed); the word at `p5+4` points at 24 random bytes less `0` (placed); the word at `p5+8` points at 24 random bytes less `0` (placed); the word at `p5+12` points at 24 random bytes less `0` (placed); the word at `p6+0x14` points at 60 random bytes less `0` (placed); the word at `p7+0x14` points at 60 random bytes less `0` (placed); the word at `p8+0x14` points at 60 random bytes less `0` (placed); the word at `p9+0x14` points at 60 random bytes less `0` (placed); the word at `0xc06210` points at 4 random bytes less `0x298` (placed); the word at `0xc061f0` points at 2912 random bytes less `0` (placed); the word at `p4+0x18` points at 8 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 0 answers 0..3 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast_ambush.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a[0]` o, `a[1]` who; `a[2]` is `get_general_upgrade`'s answer,
            // `a[5]` slot `+0xb8`'s and `a[6]` slot `+0x60`'s (the unit's
            // `is`, direct or through `ObjectData::is`'s tail, `a[7]` the
            // arm). The frame `p[2]`, the unit `p[4]`, the hero `p[6 + h]`
            // and its spells `p[10 + h]`, the console `p[14]`, the constants
            // `p[15]`, the leader's flags `p[who]`. `eax` is not an answer.
            struct Bytes<'x> {
                a: &'x [i32],
                m: &'x [u8],
                p: &'x mut [Vec<u8>],
                log: &'x mut Vec<String>,
            }
            impl Bytes<'_> {
                fn h(&self) -> usize {
                    i16::from_le_bytes([self.p[4][0x86], self.p[4][0x87]]) as usize
                }
            }
            impl crate::cast::AmbushView for Bytes<'_> {
                fn frame(&self) -> i32 {
                    rd(&self.p[2], 0)
                }
                fn spell_extent(&self) -> (i32, i32) {
                    (rd(self.m, 0x1d4), rd(self.m, 0x1d8))
                }
                fn general_upgrade(&mut self) -> i32 {
                    self.a[2]
                }
                fn is_parmenio(&mut self) -> bool {
                    if self.a[7] != 0 {
                        self.log.push("s4:p16+0x0,360,0".into());
                        self.a[6] != 0
                    } else {
                        self.log.push("s3:p4+0x0,360,0".into());
                        self.a[5] != 0
                    }
                }
                fn parmenio_ambush_time(&self) -> i32 {
                    rd(&self.p[15], 0xb58)
                }
                fn push_active_spell(&mut self, start: i32, end: i32) {
                    let h = self.h();
                    let count = rd(&self.p[6 + h], 8);
                    if count >= rd(&self.p[6 + h], 0xc) {
                        let step = i16::from_le_bytes([self.p[6 + h][0x10], self.p[6 + h][0x11]]);
                        self.log.push(format!("s1:p{}+0x4,{step}", 6 + h));
                    }
                    let at = 12 * count as usize;
                    wr(&mut self.p[10 + h], at, crate::orders::spell::AMBUSH);
                    wr(&mut self.p[10 + h], at + 4, start);
                    wr(&mut self.p[10 + h], at + 8, end);
                    wr(&mut self.p[6 + h], 8, count + 1);
                }
                fn set_flags(&mut self) {
                    let v = rd(&self.p[4], 0x68) | 0x2800;
                    wr(&mut self.p[4], 0x68, v);
                    let who = self.a[1] as usize;
                    let v = rd(&self.p[who], 0) | 0x4000;
                    wr(&mut self.p[who], 0, v);
                }
                fn console_sound(&mut self) {
                    if rd(&self.p[14], 0) == self.a[1] {
                        self.log.push(format!("s2:{},44", self.a[1]));
                    }
                }
            }
            crate::cast::cast_ambush_steps(&mut Bytes {
                a,
                m,
                p,
                log: &mut log,
            });
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "cast_ambush — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "cast_ambush — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "cast_ambush — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "cast_ambush — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("SpellType::cast_ambush@00671610: {n} rows agree");
}
