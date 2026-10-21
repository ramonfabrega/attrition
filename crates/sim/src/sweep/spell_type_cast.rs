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

/// `SpellType::cast@00676ce0`, run by `tools/emu/sweep_spell_type_cast.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 4 argument(s), `p` windows in order: none; callee 0x00675bc0 (3 args) stubbed; callee 0x006763c0 (4 args) stubbed; callee 0x00674370 (4 args) stubbed; callee 0x00673e80 (4 args) stubbed; callee 0x00673a80 (4 args) stubbed; callee 0x00672d00 (4 args) stubbed; callee 0x00673740 (2 args) stubbed; callee 0x00671820 (4 args) stubbed; callee 0x00671610 (2 args) stubbed; callee 0x00671500 (2 args) stubbed; callee 0x00672820 (2 args) stubbed; callee 0x00671e50 (2 args) stubbed; callee 0x00671760 (3 args) stubbed; callee 0x00670990 (2 args) stubbed; callee 0x00671ed0 (2 args) stubbed; callee 0x006720e0 (4 args) stubbed; callee 0x00672cb0 (4 args) stubbed; callee 0x00671f80 (4 args) stubbed; callee 0x006714c0 (2 args) stubbed; callee 0x00671480 (2 args) stubbed; callee 0x00671440 (2 args) stubbed; callee 0x00670db0 (2 args) stubbed; callee 0x00670be0 (2 args) stubbed; callee 0x006709c0 (2 args) stubbed; callee 0x006704a0 (2 args) stubbed; callee 0x00670880 (2 args) stubbed; callee 0x00670740 (2 args) stubbed; callee 0x00670600 (2 args) stubbed; stub 0 answered from [3, 3, 3, 3, 0, 1, 2] (90%); stub 1 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast.py") else {
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
            // hand: port
            // The port is the crate's own switch, `crate::spell_cast_view::spell_cast_steps`; here the view is
            // the emulated original's stubs. Window layout: `m` the spell-type row (`+4` its index); the
            // arguments are `p1..p4` then each stub's answer in stub order (0 `is_castable`, 1
            // `is_valid_target`, then the casts). The function is `void`: `eax` is what the last call left
            // (the last stub's answer; `spell - 0x275` where the jump table or its range check sends the
            // index nowhere; `is_castable`'s answer when it is not 3).
            //
            // `ecx` the switch never loads (`Ecx::Stale`) is the stub's leftover, which the emulator
            // holds at `this`; the real value after the real `is_castable` is not modelled.
            use crate::spell_cast_view::{Arg, Callee, Ecx, SpellCastView, spell_cast_steps};
            struct V<'a> {
                m: &'a [u8],
                a: &'a [i32],
                log: &'a mut Vec<String>,
                eax: i32,
            }
            fn stub(c: Callee) -> usize {
                match c {
                    Callee::CreateDecoy => 2,
                    Callee::Pilfer => 3,
                    Callee::DoubleAgent => 4,
                    Callee::Bribe => 5,
                    Callee::CounterIntel => 6,
                    Callee::Assimilate => 7,
                    Callee::Ambush => 8,
                    Callee::March => 9,
                    Callee::Entrench => 10,
                    Callee::Depopulate => 11,
                    Callee::MoveCapital => 12,
                    Callee::ResearchProject => 13,
                    Callee::Disband => 14,
                    Callee::Sabotage => 15,
                    Callee::BlowTread => 16,
                    Callee::Paradrop => 17,
                    Callee::Supplies => 18,
                    Callee::LockGate => 19,
                    Callee::UnlockGate => 20,
                    Callee::Transport => 21,
                    Callee::Pack => 22,
                    Callee::Unpack => 23,
                    Callee::Civilian => 24,
                    Callee::ToArms => 25,
                    Callee::ImmortalArch => 26,
                    Callee::ImmortalHinf => 27,
                }
            }
            impl SpellCastView for V<'_> {
                fn spell(&self) -> i32 {
                    rd(self.m, 4)
                }
                fn is_castable(&mut self, p1: i32, p2: i32) -> i32 {
                    self.log.push(format!("s0:m+0x0,{p1},{p2},1"));
                    let r = self.a[4];
                    self.eax = if r == 3 {
                        self.spell().wrapping_sub(0x275)
                    } else {
                        r
                    };
                    r
                }
                fn is_valid_target(&mut self, p2: i32, p3: i32, p4: i32) -> bool {
                    self.log.push(format!("s1:m+0x0,{p2},{p3},{p4},0"));
                    self.eax = self.a[5];
                    self.a[5] != 0
                }
                fn air_target(&mut self, _: i32, _: i32) -> i32 {
                    unreachable!("0x299 is the second sweep's")
                }
                fn current_upgrade(&mut self, _: i32, _: i32) -> i32 {
                    unreachable!("0x299 is the second sweep's")
                }
                fn air_set(&mut self, _: i32, _: i32, _: i32) {
                    unreachable!("0x299 is the second sweep's")
                }
                fn call(&mut self, callee: Callee, ecx: Ecx, args: &[Arg]) {
                    let ecx = match ecx {
                        Ecx::This | Ecx::Stale => "m+0x0".to_string(),
                        Ecx::Val(v) => v.to_string(),
                    };
                    let args: Vec<String> = args
                        .iter()
                        .map(|g| match *g {
                            Arg::P(n) => self.a[n as usize - 1].to_string(),
                            Arg::Stale => "m+0x0".to_string(),
                        })
                        .collect();
                    let s = stub(callee);
                    self.log.push(format!("s{s}:{ecx},{}", args.join(",")));
                    self.eax = self.a[4 + s];
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
            assert_eq!(w, &want[1 + i], "cast — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("SpellType::cast@00676ce0: {n} rows agree");
}
