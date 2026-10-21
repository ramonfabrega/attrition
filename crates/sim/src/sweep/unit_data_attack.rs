//! `UnitData::attack@006103c0` under the emulator against the port (item 3399; tools/sweep_gen.py, obj with windows).

use sim::unit_attack_view::{self, UnitAttackView};

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

/// `UnitData::attack@006103c0`, run by `tools/emu/sweep_unit_data_attack.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 0 argument(s), `p` windows in order: p0..p5 sparse table 0 (2 rows × 3 slice(s), row-major), p6 the place; the global cell 0x00c061e4 points at 3328 random bytes; callee 0x006469f0 (0 args) stubbed; callee 0x006ebc10 (1 args) stubbed; callee 0x00646b00 (2 args) stubbed; vtable slot +0x148 (1 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xf0 (1 args) stubbed; vtable slot +0x10c (0 args) stubbed; the word at `m+0x18` points at 560 random bytes less `0` (placed); slot +0x148 laid as 0x0046cf10 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [-5, 0, 10, 100] (90%); stub 1 answered from [0, 1, 2] (90%); stub 2 answered from [-1, -1, 0, 3] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 1 answers 2 values in call order, printed after the answers; stub 2 answers 10 values in call order, printed after the answers; stub 3 answers 2 values in call order, printed after the answers; stub 4 answers 5 values in call order, printed after the answers; stub 5 answers 5 values in call order, printed after the answers; stub 6 answers 2 values in call order, printed after the answers; stub 7 answers 1 values in call order, printed after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x6eec apart at `0xc061e0`, slices +0x59c0:0x30, +0x6da4:0x4, +0x6dcc:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_attack.py") else {
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
            let mut view = Rows {
                a,
                m,
                c: &*c,
                p: &*p,
                log: &mut log,
                turn: [0; 8],
            };
            let got = i64::from(unit_attack_view::unit_attack_steps(&mut view));
            if log.is_empty() {
                log.push("-".to_string());
            }
            got
        };
        assert_eq!(got, want_eax, "attack — eax, row `{line}`");
        assert_eq!(log, want_log, "attack — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "attack — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "attack — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "attack — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("UnitData::attack@006103c0: {n} rows agree");
}

/// The row as the view. `a[0]` is `ObjectData::attack`'s answer; the stubs' sequences follow the
/// seven single answers: `a[8..10]` has_wonder, `a[10..20]` has_general, `a[20..22]` slot 0x148,
/// `a[22..27]` slot 0xb8, `a[27..32]` the type's slot 0x60, `a[32..34]` its slot 0xf0, `a[34]` its
/// slot 0x10c; then the flags `a[35]` (slot 0x148 is the base) and `a[36]` (slot 0xb8 is the base).
/// `m[9]` is the leader row; the row's windows are `p[3 * row..]` (the shorts from `+0x59c0`, the
/// bytes at `+0x6da4` and `+0x6dcc`), the type record is `p[6]`, the constants `c[0]`.
struct Rows<'a> {
    a: &'a [i32],
    m: &'a [u8],
    c: &'a [Vec<u8>],
    p: &'a [Vec<u8>],
    log: &'a mut Vec<String>,
    turn: [usize; 8],
}

impl Rows<'_> {
    fn row(&self) -> usize {
        usize::from(self.m[9])
    }
    /// The next answer of stub `s` (its sequence at `a[start..start + n]`, the last repeated).
    fn seq(&mut self, s: usize, start: usize, n: usize) -> i32 {
        let k = self.turn[s].min(n - 1);
        self.turn[s] += 1;
        self.a[start + k]
    }
}

impl UnitAttackView for Rows<'_> {
    fn base_attack(&mut self) -> i32 {
        self.a[0]
    }
    fn has_wonder(&mut self, ty: i32) -> bool {
        self.log.push(format!("s1:r{}+0x0,{ty}", self.row()));
        self.seq(1, 8, 2) != 0
    }
    fn has_general(&mut self, ty: i32) -> i32 {
        self.log.push(format!("s2:m+0x0,0,{ty}"));
        self.seq(2, 10, 10)
    }
    fn constant(&mut self, off: u32) -> i32 {
        rd(&self.c[0], off as usize)
    }
    fn leader_short(&mut self, off: u32) -> i16 {
        let o = off as usize - 0x59c0;
        let w = &self.p[3 * self.row()];
        i16::from_le_bytes([w[o], w[o + 1]])
    }
    fn obsidian(&mut self) -> bool {
        let r = self.row();
        self.p[3 * r + 1][3] & 4 != 0 || self.p[3 * r + 2][3] & 4 != 0
    }
    fn slot_148_is_base(&mut self) -> bool {
        self.a[35] != 0
    }
    fn type_slot_f0(&mut self, mask: i32) -> i32 {
        self.log.push(format!("s6:p6+0x0,{mask}"));
        self.seq(6, 32, 2)
    }
    fn slot_148(&mut self, mask: i32) -> i32 {
        self.log.push(format!("s3:m+0x0,{mask}"));
        self.seq(3, 20, 2)
    }
    fn slot_b8_is_base(&mut self) -> bool {
        self.a[36] != 0
    }
    fn type_slot_60(&mut self, ty: i32) -> i32 {
        self.log.push(format!("s5:p6+0x0,{ty},0"));
        self.seq(5, 27, 5)
    }
    fn slot_b8(&mut self, ty: i32) -> i32 {
        self.log.push(format!("s4:m+0x0,{ty},0"));
        self.seq(4, 22, 5)
    }
    fn type_slot_10c(&mut self) -> i32 {
        self.log.push("s7:p6+0x0".to_string());
        self.seq(7, 34, 1)
    }
    fn type_word_218(&mut self) -> i32 {
        rd(&self.p[6], 0x218)
    }
    fn type_word_40(&mut self) -> i32 {
        rd(&self.p[6], 0x40)
    }
}
