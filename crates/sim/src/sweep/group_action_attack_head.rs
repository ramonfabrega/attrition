//! `Group::action_attack@00712490` under the emulator against the port (item 3362; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_attack@00712490`, run by `tools/emu/sweep_group_action_attack_head.py` under unicorn
/// (tools/emu): 256 random bytes of object, 5 argument(s), `p` windows in order: p0 the place; 4 random bytes at 0x00cc02f8 (each word in 0..1); 4 random bytes at 0x00ed6574 (each word in 0..3); 4 random bytes at 0x00ed6590 (each word in 0..3); 4 random bytes at 0x00ed65ac (each word in 0..3); vtable slot +0x10 (4 args) stubbed; vtable slot +0x14 (0 args) stubbed; callee 0x0070c450 (0 args) stubbed; the word at `0xed6580+0x1c*m[0x4a]` points at 32 random bytes less `0` (placed); stub 2 answered from [1, 1, 1, 0] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the call ends where stub 1 is first entered (the function is swept to that callee); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_attack_head.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Window layout. `m` the group (`+0xc` count, `+0x4a` player); `c[0]` the scenario's ignore-orders
                // flag, `c[1..=3]` the counts of `objects_ignoring_orders[0..=2]`; `p[0]` the list the player's entry
                // points at. `a`: the five arguments, one answer a stub (`a[5 + k]`).
                //
                // **The port is the crate's own head**, `crate::attack_view::attack_head`; the call ends at the
                // group's refresh slot (`--stop-stub 1`), as the steps' next call.
                use crate::attack_view::{AttackHead, attack_head};
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                impl V<'_> {
                    fn rec(&mut self, k: usize, args: &[i32]) {
                        let mut t = format!("s{k}:m+0x0");
                        for v in args {
                            t.push_str(&format!(",{v}"));
                        }
                        self.log.push(t);
                    }
                }
                impl AttackHead for V<'_> {
                    fn is_on_map(&mut self) -> bool {
                        self.rec(2, &[]);
                        self.a[5 + 2] != 0
                    }
                    fn ignore_orders_on(&self) -> bool {
                        rd32(&self.c[0], 0) != 0
                    }
                    fn ignored_count(&self, who: u8) -> i32 {
                        rd32(&self.c[1 + usize::from(who)], 0)
                    }
                    fn ignored_nth(&self, _who: u8, i: i32) -> i32 {
                        rd32(&self.p[0], 4 * i as usize)
                    }
                    fn ignore_call(&mut self, o: i32, who: u8) {
                        self.rec(0, &[o, i32::from(who), 0, 0]);
                    }
                    fn who(&self) -> u8 {
                        self.m[0x4a]
                    }
                    fn count(&self) -> i32 {
                        rd32(self.m, 0xc)
                    }
                    fn refresh(&mut self) {
                        self.rec(1, &[]);
                    }
                }
                let mut v = V {
                    a,
                    m: &*m,
                    c: &*c,
                    p: &*p,
                    log: &mut log,
                };
                let _ = attack_head(&mut v, a[0]);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_attack — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_attack — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_attack — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "action_attack — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Group::action_attack@00712490: {n} rows agree");
}
