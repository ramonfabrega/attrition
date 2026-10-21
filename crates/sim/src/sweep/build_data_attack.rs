//! `BuildData::attack@0062e610` under the emulator against the port (item 3387; tools/sweep_gen.py, obj with windows).

use sim::build_attack_view::{self, BuildAttackView};

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

/// `BuildData::attack@0062e610`, run by `tools/emu/sweep_build_data_attack.py` under unicorn
/// (tools/emu): 128 random bytes of object, 0 argument(s), `p` windows in order: p0..p7 sparse table 0 (2 rows × 4 slice(s), row-major), p8..p9 the place(s), in flag order; the global cell 0x00c061e4 points at 3328 random bytes; the global cell 0x00e85ddc points at 256 random bytes; callee 0x006469f0 (0 args) stubbed; callee 0x006d9380 (1 args) stubbed; callee 0x0064dda0 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0x68 (1 args) stubbed; the word at `c1+0xd8` points at 32 random bytes less `0` (placed); the word at `m+0x18` points at 64 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [-5, 0, 10, 100] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1, 2] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 5, 54, 100] (90%); stub 2 answers 2 values in call order, printed after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x4, +0x59d0:0x4, +0x6da4:0x4, +0x6dcc:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_attack.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
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
                count_turn: 0,
            };
            i64::from(build_attack_view::build_attack_steps(&mut view))
        };
        assert_eq!(got, want_eax, "attack — eax, row `{line}`");
        assert_eq!(log, want_log, "attack — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "attack — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "attack — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "attack — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("BuildData::attack@0062e610: {n} rows agree");
}

/// The row's windows as the view: `a` is `s0..s6`, the seq's two `count_inside` answers, the slot flag.
struct Rows<'a> {
    a: &'a [i32],
    m: &'a [u8],
    c: &'a [Vec<u8>],
    p: &'a [Vec<u8>],
    log: &'a mut Vec<String>,
    count_turn: usize,
}

impl Rows<'_> {
    fn row(&self) -> usize {
        usize::from(self.m[9])
    }
    fn leader(&self, slice: usize) -> &[u8] {
        &self.p[4 * self.row() + slice]
    }
    fn c0(&self, off: usize) -> i32 {
        rd(&self.c[0], off)
    }
}

impl BuildAttackView for Rows<'_> {
    fn base_attack(&mut self) -> i32 {
        self.log.push("s0:m+0x0".to_string());
        self.a[0]
    }
    fn is_base_is(&mut self) -> bool {
        self.a[9] != 0
    }
    fn type_slot_60(&mut self) -> i32 {
        self.a[4]
    }
    fn is_override(&mut self) -> i32 {
        self.a[3]
    }
    fn type_slot_fc(&mut self) -> i32 {
        self.a[5]
    }
    fn flag_20(&mut self) -> bool {
        self.m[8] & 0x20 != 0
    }
    fn city(&mut self) -> i16 {
        i16::from_le_bytes([self.m[0x72], self.m[0x73]])
    }
    fn tribe_can_type(&mut self) -> bool {
        self.log.push(format!("s1:r{}+0x0,p8+0x0", self.row()));
        self.a[1] != 0
    }
    fn type_slot_68_who(&mut self) -> i32 {
        let who = rd(self.leader(0), 0);
        self.log.push(format!("s6:p8+0x0,{who}"));
        self.a[6]
    }
    fn count_inside(&mut self, kind: i32, ty: i32) -> i32 {
        self.log.push(format!("s2:m+0x0,{kind},{ty}"));
        let v = self.a[7 + self.count_turn.min(1)];
        self.count_turn += 1;
        v
    }
    fn obsidian(&mut self) -> bool {
        self.leader(2)[3] & 4 != 0 || self.leader(3)[3] & 4 != 0
    }
    fn antipater(&mut self) -> bool {
        self.leader(1)[..2] != [0, 0]
    }
    fn general_building_attack(&mut self) -> i32 {
        self.c0(0xc60)
    }
    fn obsidian_archers_attack(&mut self) -> i32 {
        self.c0(0x944)
    }
    fn antipater_garrison_attack_bonus(&mut self) -> i32 {
        self.c0(0xb94)
    }
}
