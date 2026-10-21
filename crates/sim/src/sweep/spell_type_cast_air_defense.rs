//! `SpellType::cast@00676ce0` under the emulator against the port (item 3391; tools/sweep_gen.py, obj with windows).

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

/// `SpellType::cast@00676ce0`, run by `tools/emu/sweep_spell_type_cast_air_defense.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 4 argument(s), `p` windows in order: p0..p6 the place(s), in flag order; the global cell 0x00c0618c points at 1024 random bytes; callee 0x00675bc0 (3 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x006e3140 (1 args) stubbed; vtable slot +0x80 (2 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p1+4` points at 32 random bytes less `0` (placed); stub 1's answer points at 32 random bytes less `0` (placed); stub 0 answered from [3] (90%); the stubbed vtable laid at `p2+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast_air_defense.py") else {
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
            // The airdefense arm (`0x299`) of `crate::spell_cast_view::spell_cast_steps`. Windows: `c[0]` the
            // objects table the cell `0xc0618c` points at (`+0x14 + 0x1c * p2` the row's array), `p[0..1]` the
            // two arrays (four-byte entries by `p1`), `p[2..5]` the objects they name (word 0 the stubbed
            // vtable), `p[6]` the object slot `+0xac` answers. `p1` and `p2` are held to 0 or 1.
            // Stubs: 0 `is_castable`, 1 slot `+0xac`, 2 `current_upgrade` (`ecx` the leader's row), 3 slot `+0x80`.
            // `eax` is the last stub's answer; `is_castable`'s when it is not 3.
            use crate::spell_cast_view::{Arg, Callee, Ecx, SpellCastView, spell_cast_steps};
            struct V<'a> {
                m: &'a [u8],
                a: &'a [i32],
                log: &'a mut Vec<String>,
                eax: i32,
            }
            impl SpellCastView for V<'_> {
                fn spell(&self) -> i32 {
                    rd(self.m, 4)
                }
                fn is_castable(&mut self, p1: i32, p2: i32) -> i32 {
                    self.log.push(format!("s0:m+0x0,{p1},{p2},1"));
                    self.eax = self.a[4];
                    self.a[4]
                }
                fn is_valid_target(&mut self, _: i32, _: i32, _: i32) -> bool {
                    unreachable!("the first sweep's")
                }
                fn call(&mut self, _: Callee, _: Ecx, _: &[Arg]) {
                    unreachable!("the first sweep's")
                }
                fn air_target(&mut self, p1: i32, p2: i32) -> i32 {
                    self.log.push(format!("s1:p{}+0x0", 2 + 2 * p2 + p1));
                    self.a[5]
                }
                fn current_upgrade(&mut self, p2: i32, ti: i32) -> i32 {
                    let ecx = p2.wrapping_mul(0x6eec).wrapping_add(0xe3a390);
                    self.log.push(format!("s2:{ecx},{ti}"));
                    self.a[6]
                }
                fn air_set(&mut self, _target: i32, upgrade: i32, zero: i32) {
                    self.log.push(format!("s3:p6+0x0,{upgrade},{zero}"));
                    self.eax = self.a[7];
                }
            }
            let mut v = V {
                m,
                a,
                log: &mut log,
                eax: 0,
            };
            spell_cast_steps(&mut v, [a[0], a[1], a[2], a[3]]);
            i64::from(v.eax)
            // end hand
        };
        assert_eq!(got, want_eax, "cast — eax, row `{line}`");
        assert_eq!(log, want_log, "cast — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "cast — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "cast — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "cast — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("SpellType::cast@00676ce0: {n} rows agree");
}
