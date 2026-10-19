//! `Unit::think_carry@005f2ee0` under the emulator against the port (item 3281; tools/sweep_gen.py, obj with windows).

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

/// `Unit::think_carry@005f2ee0`, run by `tools/emu/sweep_unit_think_carry.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), `p` windows in order: none; callee 0x0064dda0 (2 args) stubbed; callee 0x005f37c0 (1 args) stubbed; callee 0x005f3260 (0 args) stubbed; callee 0x005f2f30 (0 args) stubbed; stub 0 answered from [0, 0, 0, 0, 1] (90%); stub 0 answers 3 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_think_carry.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a`: 0 the single answer of `count_inside` (unused: the sequence is read), 1 `think_carry_civilian`'s,
            // 2 `think_carry_military`'s, 3 `think_carry_idle`'s, 4..7 what `count_inside` answers its calls in
            // order. Stub logs: 0 the counts (`ecx,kind,arg`, `/`-joined), 1 civilian (`ecx,workers`), 2 and 3 `-`
            // when not entered. **The port is the crate's own steps**, `think_carry_steps`, which `Sim::think_carry`
            // calls over its `SimCarry`; here the view is the emulated original's stubs. `this` is never read.
            use crate::tech::TypeId;
            use crate::transport::{CarryView, think_carry_steps};
            struct V<'a> {
                a: &'a [i32],
                counts: Vec<String>,
                civ: Option<String>,
                mil: bool,
                idle: bool,
            }
            impl V<'_> {
                fn count(&mut self, kind: i32, arg: i32) -> i32 {
                    let k = self.counts.len();
                    self.counts.push(format!("m+0x0,{kind},{arg}"));
                    self.a[4 + k.min(2)]
                }
            }
            impl CarryView for V<'_> {
                fn count_workers(&mut self) -> i32 {
                    self.count(0, 0)
                }
                fn count_type(&mut self, line: TypeId) -> i32 {
                    self.count(0x11, line as i32)
                }
                fn count_attack(&mut self) -> i32 {
                    self.count(2, 0)
                }
                fn civilian(&mut self, workers: bool) -> i32 {
                    self.civ = Some(format!("m+0x0,{}", i32::from(workers)));
                    self.a[1]
                }
                fn military(&mut self) -> i32 {
                    self.mil = true;
                    self.a[2]
                }
                fn idle(&mut self) -> i32 {
                    self.idle = true;
                    self.a[3]
                }
            }
            let mut v = V {
                a,
                counts: Vec::new(),
                civ: None,
                mil: false,
                idle: false,
            };
            let r = think_carry_steps(&mut v);
            log.push(if v.counts.is_empty() {
                "-".to_string()
            } else {
                v.counts.join("/")
            });
            log.push(v.civ.unwrap_or_else(|| "-".to_string()));
            log.push(if v.mil {
                "m+0x0".to_string()
            } else {
                "-".to_string()
            });
            log.push(if v.idle {
                "m+0x0".to_string()
            } else {
                "-".to_string()
            });
            i64::from(r)
        };
        assert_eq!(got, want_eax, "think_carry — eax, row `{line}`");
        assert_eq!(log, want_log, "think_carry — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "think_carry — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "think_carry — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "think_carry — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("Unit::think_carry@005f2ee0: {n} rows agree");
}
