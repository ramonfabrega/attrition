//! `Herd::create_units@007417f0` under the emulator against the port (item 3391; tools/sweep_gen.py, obj with windows).

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

/// `Herd::create_units@007417f0`, run by `tools/emu/sweep_herd_create_units.py` under unicorn
/// (tools/emu): 32 random bytes of object, 0 argument(s), `p` windows in order: p0..p6 the place(s), in flag order; callee 0x0065e0c0 (7 args) stubbed; callee 0x0061de70 (16 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x00a28eb0 (0 args) stubbed; callee 0x00605400 (3 args) stubbed; callee 0x005f8d20 (4 args) stubbed; the word at `0xc0afa0` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 144 random bytes less `0` (placed); the word at `p0+4` points at 144 random bytes less `0` (placed); the word at `p0+8` points at 144 random bytes less `0` (placed); the word at `p1+0x18` points at 64 random bytes less `0` (placed); the word at `p2+0x18` points at 64 random bytes less `0` (placed); the word at `p3+0x18` points at 64 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; the stubbed vtable laid at `p1+0`; stub 0 answers 4 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 3 answers 8 values in call order, printed after the answers; stub 4 answers 4 values in call order, printed after the answers; stub 5 answers 4 values in call order, printed after the answers; stub 0 answers -2..2 every row; stub 2 answers 0..1 every row; stub 3 answers 0..1 every row; stub 4 answers 0..65535 every row; stub 5 answers -5000..5000 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 1 writes a value in -600..600 through its argument 2, printed after the answers (`o` in a place); stub 1 writes a value in -600..600 through its argument 3, printed after the answers (`o` in a place); the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_herd_create_units.py") else {
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
            // The row: `a[0..8]` the stubs' single answers (unused: every stub answers by its `--seq`), `a[8..10]`
            // `find_nearby_spot`'s two out words, then the `--seq` answers: `a[10..14]` `init_unit`'s, `a[14..22]`
            // the unit's `is` slot's, `a[22..30]` the type's `is` slot's, `a[30..34]` `Random::get`'s, `a[34..38]`
            // `degrees_to_angle`'s, and `a[38]` whether the unit's `+0xb8` slot is `ObjectData::is` (inlined).
            //
            // Windows: `m` the herd (`+0` `+4` the cell, `+0x10` the type, `+0x18` the tag), `p[0]` the units array,
            // `p[1..4]` the units, `p[4..7]` their type records (`unit + 0x18` points at `p[3 + i]`).
            //
            // **The port is `crate::herd_create_view`**; the view is the emulated original's windows and stubs.
            use crate::herd_create_view::{HerdRec, HerdView, Left, create_units};
            struct V<'a> {
                a: &'a [i32],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                turns: [usize; 6],
            }
            impl V<'_> {
                /// The next answer of a `--seq` stub: `a[base..base + n]`, the last repeated.
                fn seq(&mut self, which: usize, base: usize, n: usize) -> i32 {
                    let t = self.turns[which].min(n - 1);
                    self.turns[which] += 1;
                    self.a[base + t]
                }
                fn utok(u: i32) -> String {
                    format!("p{}+0x0", 1 + u)
                }
                fn ttok(u: i32) -> String {
                    format!("p{}+0x0", 4 + u)
                }
            }
            impl HerdView for V<'_> {
                fn init_unit(&mut self, kind: i32, x: i32, y: i32) -> i32 {
                    self.log
                        .push(format!("s0:12626800,8,{kind},{x},{y},-1,-1,-1"));
                    self.seq(0, 10, 4)
                }
                fn unit_dims(&mut self, u: i32) -> (u8, i16) {
                    let w = &self.p[1 + u as usize];
                    (w[9], i16::from_le_bytes([w[0xa], w[0xb]]))
                }
                fn find_nearby_spot(&mut self, u: i32, x: i32, y: i32, d: (u8, i16)) -> (i32, i32) {
                    self.log.push(format!(
                        "s1:{},{x},{y},sp-0x10,sp-0x14,192,-1,0,1431655765,3,{},{},0,0,-1,0,-1",
                        Self::ttok(u),
                        d.1,
                        d.0
                    ));
                    (self.a[8], self.a[9])
                }
                fn is_inlined(&mut self, _u: i32) -> bool {
                    self.a[38] != 0
                }
                fn unit_is(&mut self, u: i32, t: i32) -> bool {
                    if self.a[38] != 0 {
                        self.log.push(format!("s3:{},{t},0", Self::ttok(u)));
                        self.seq(3, 22, 8) != 0
                    } else {
                        self.log.push(format!("s2:{},{t},0", Self::utok(u)));
                        self.seq(2, 14, 8) != 0
                    }
                }
                fn random(&mut self) -> i32 {
                    self.log.push("s4:14908044,0,65535".into());
                    self.seq(4, 30, 4)
                }
                fn degrees_to_angle(&mut self, rem: i32) -> i32 {
                    self.log.push(format!("s5:{rem}"));
                    self.seq(5, 34, 4)
                }
                fn set_angle(&mut self, u: i32, angle: i32, left: Left) {
                    let l = match left {
                        Left::TypeOf(u) => Self::ttok(u),
                        Left::Unit(u) => Self::utok(u),
                        Left::Degrees => "360".into(),
                    };
                    self.log.push(format!("s6:{},{angle},{l},1", Self::utok(u)));
                }
                fn set_new_location(&mut self, u: i32, x: i32, y: i32) {
                    self.log.push(format!("s7:{},{x},{y},1,1", Self::utok(u)));
                }
                fn set_tag(&mut self, u: i32, tag: i16) {
                    let w = &mut self.p[1 + u as usize];
                    w[0x86..0x88].copy_from_slice(&tag.to_le_bytes());
                }
            }
            let h = HerdRec {
                cx: rd(m, 0),
                cy: rd(m, 4),
                kind: rd(m, 0x10),
                tag: i16::from_le_bytes([m[0x18], m[0x19]]),
            };
            let mut v = V {
                a,
                p,
                log: &mut log,
                turns: [0; 6],
            };
            i64::from(create_units(&h, &mut v))
        };
        assert_eq!(got, want_eax, "create_units — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "create_units — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "create_units — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "create_units — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "create_units — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Herd::create_units@007417f0: {n} rows agree");
}
