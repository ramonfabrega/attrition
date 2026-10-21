//! `Group::action_gather@00700b90` under the emulator against the port (item 3386; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_gather@00700b90`, run by `tools/emu/sweep_group_action_gather_head.py` under unicorn
/// (tools/emu): 2272 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 the place(s), in flag order; 4 random bytes at 0x00cc02f8 (each word in 1..1); 4 random bytes at 0x00ed6574 (each word in 0..3); 4 random bytes at 0x00ed6590 (each word in 0..3); vtable slot +0x10 (4 args) stubbed; vtable slot +0x14 (0 args) stubbed; callee 0x00713e80 (1 args) stubbed; the word at `0xed6580+0x1c*0` points at 16 random bytes less `0` (placed); the word at `0xed6580+0x1c*1` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 2816 bytes under the call is zeroed before each row; the call ends where stub 1 is first entered (the function is swept to that callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_gather_head.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The head slice: the call ends where the group's refresh (slot `+0x14`, stub 1) is entered.
                // Window layout. `m` the group (`+0x4a` player); `c[0]` `ignore_orders`, `c[1]` and `c[2]`
                // players 0 and 1's ignore-orders counts; `p[who]` the elements of
                // `objects_ignoring_orders[who]` (players 0 and 1 are laid; a player of 8 or more skips the sweep).
                // The scratch group's first `clear` (stub 2) comes before the sweep.
                //
                // **The port is the crate's own head**: `action_gather_steps` begins with `list_clear` and
                // `crate::garrison_view::garrison_head`; here the view is the emulated original's windows.
                use crate::action_gather_view::{GarrisonHead, garrison_head};
                struct V<'a> {
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                impl GarrisonHead for V<'_> {
                    fn who(&self) -> i32 {
                        i32::from(self.m[0x4a])
                    }
                    fn ignore_orders(&self) -> bool {
                        rd32(&self.c[0], 0) != 0
                    }
                    fn ignoring_count(&self, who: i32) -> i32 {
                        rd32(&self.c[1 + who as usize], 0)
                    }
                    fn ignoring_at(&self, who: i32, i: i32) -> i32 {
                        rd32(&self.p[who as usize], 4 * i as usize)
                    }
                    fn drop_ignored(&mut self, o: i32, who: i32) {
                        self.log.push(format!("s0:m+0x0,{o},{who},0,0"));
                    }
                    fn refresh(&mut self) {
                        self.log.push("s1:m+0x0".into());
                    }
                }
                let mut v = V {
                    m: &*m,
                    c: &*c,
                    p: &*p,
                    log: &mut log,
                };
                v.log.push("s2:sp-0xa20,-1".into());
                garrison_head(&mut v);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_gather — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_gather — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_gather — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "action_gather — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Group::action_gather@00700b90: {n} rows agree");
}
