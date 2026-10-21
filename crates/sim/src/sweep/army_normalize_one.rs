//! `Army::normalize@006f9b50` under the emulator against the port (item 3387; tools/sweep_gen.py, obj with windows).

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

/// `Army::normalize@006f9b50`, run by `tools/emu/sweep_army_normalize_one.py` under unicorn
/// (tools/emu): 160 random bytes of object, 0 argument(s), `p` windows in order: p0..p35 sparse table 0 (6 rows × 6 slice(s), row-major), p36..p44 the place(s), in flag order; callee 0x00711540 (0 args) stubbed; callee 0x006f8b50 (1 args) stubbed; vtable slot +0x8 (0 args) stubbed; callee 0x00711720 (3 args) stubbed; callee 0x0070ccb0 (1 args) stubbed; the word at `0xc0aec0+m[0x94]*0x1c` points at 16 random bytes less `0` (placed); the word at `p36+0` points at 32 random bytes less `0` (placed); the word at `p36+4` points at 32 random bytes less `0` (placed); the word at `p36+8` points at 32 random bytes less `0` (placed); the word at `p36+12` points at 32 random bytes less `0` (placed); the word at `p37+0x18` points at 24 random bytes less `0` (placed); the word at `p38+0x18` points at 24 random bytes less `0` (placed); the word at `p39+0x18` points at 24 random bytes less `0` (placed); the word at `p40+0x18` points at 24 random bytes less `0` (placed); the stubbed vtable laid at `p0+0`; stub 2 answers 12 values in call order, printed after the answers; stub 3 answers 30 values in call order, printed after the answers; stub 2 answers -50..50 every row; stub 3 answers -40..40 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 4 answers from its own `ecx`: the word at `+0x10`; 6 rows 0x9d4 apart at `0xe85f20`, slices +0x0:0x4, +0xc:0x4, +0x10:0x4, +0x34:0x4, +0x48:0x4, +0x8cc:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_normalize_one.py") else {
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
            {
                // `Army::normalize` through `army_normalize_view` (not the crate's `Sim::army_normalize`, a namesake):
                // the view over the windows. Group id `g` is row `g` of the sparse table, its slices `p[6g..6g+6]`
                // (vtable, num, leader word at +0x10, role, +0x48 word, list word); the units chain is `p[41 + i]`
                // (`+0x14` of unit i's type record). Stub answers: stub 2 `a[5 + k]`, stub 3 `a[17 + k]` for the k-th call.
                use crate::army_normalize_view::{ArmyNormalizeView, army_normalize_steps};
                struct V<'a> {
                    m: &'a mut [u8],
                    p: &'a mut [Vec<u8>],
                    a: &'a [i32],
                    log: &'a mut Vec<String>,
                    k2: usize,
                    k3: usize,
                }
                impl ArmyNormalizeView for V<'_> {
                    fn who(&self) -> i32 {
                        i32::from(i16::from_le_bytes([self.m[0x94], self.m[0x95]]))
                    }
                    fn count(&self) -> i32 {
                        i32::from(i16::from_le_bytes([self.m[0x96], self.m[0x97]]))
                    }
                    fn slot(&self, s: i32) -> i32 {
                        rd(self.m, 0x54 + 4 * s as usize)
                    }
                    fn set_slot(&mut self, s: i32, id: i32) {
                        wr(self.m, 0x54 + 4 * s as usize, id);
                    }
                    fn total(&self, t: usize) -> i32 {
                        rd(self.m, [0xc, 0x10, 0x14, 0x18, 0x1c][t])
                    }
                    fn set_total(&mut self, t: usize, v: i32) {
                        wr(self.m, [0xc, 0x10, 0x14, 0x18, 0x1c][t], v);
                    }
                    fn group_normalize(&mut self, g: i32) {
                        self.log.push(format!("s0:p{}+0x0", 6 * g));
                    }
                    fn num(&self, g: i32) -> i32 {
                        rd(&self.p[6 * g as usize + 1], 0)
                    }
                    fn buildings(&self, g: i32) -> u8 {
                        self.p[6 * g as usize + 4][1]
                    }
                    fn role(&self, g: i32) -> i32 {
                        rd(&self.p[6 * g as usize + 3], 0)
                    }
                    fn list0(&self, g: i32) -> i32 {
                        let w = &self.p[6 * g as usize + 5];
                        i32::from(i16::from_le_bytes([w[0], w[1]]))
                    }
                    fn vcall8(&mut self, g: i32) -> i32 {
                        self.log.push(format!("s2:p{}+0x0", 6 * g));
                        let v = self.a[5 + self.k2.min(11)];
                        self.k2 += 1;
                        v
                    }
                    fn group_count(&mut self, g: i32, kind: i32, x: i32, y: i32) -> i32 {
                        self.log.push(format!("s3:p{}+0x0,{kind},{x},{y}", 6 * g));
                        let v = self.a[17 + self.k3.min(29)];
                        self.k3 += 1;
                        v
                    }
                    fn find_leader(&mut self, g: i32) -> i32 {
                        self.log.push(format!("s4:p{}+0x0,0", 6 * g));
                        rd(&self.p[6 * g as usize + 2], 0)
                    }
                    fn remove_group(&mut self, id: i32) {
                        self.log.push(format!("s1:m+0x0,{id}"));
                    }
                    fn unit_type_field(&self, who: i32, unit: i32) -> i32 {
                        // The cell read is `units + who * 0x1c`: the ptrtab sits at the army's own player.
                        assert_eq!(who, self.who());
                        rd(&self.p[41 + unit as usize], 0x14)
                    }
                }
                army_normalize_steps(&mut V {
                    m,
                    p,
                    a,
                    log: &mut log,
                    k2: 0,
                    k3: 0,
                });
                if log.is_empty() {
                    log.push("-".into());
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "normalize — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "normalize — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "normalize — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "normalize — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2500, "only {n} rows");
    eprintln!("Army::normalize@006f9b50: {n} rows agree");
}
