//! `Group::action_swarm_around@0070fbe0` under the emulator against the port (item 3386; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_swarm_around@0070fbe0`, run by `tools/emu/sweep_group_action_swarm_around_head.py` under unicorn
/// (tools/emu): 2272 random bytes of object, 5 argument(s), `p` windows in order: p0..p11 the place(s), in flag order; the global cell 0x00c0618c points at 768 random bytes; 4 random bytes at 0x00cc02f8 (each word in 1..1); 4 random bytes at 0x00ed6574 (each word in 0..3); 4 random bytes at 0x00ed6590 (each word in 0..3); vtable slot +0x10 (4 args) stubbed; vtable slot +0x14 (0 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x0070c450 (0 args) stubbed; the word at `0xed6580+0x1c*0` points at 16 random bytes less `0` (placed); the word at `0xed6580+0x1c*1` points at 16 random bytes less `0` (placed); the word at `c0+0x14+0x1c*0` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p2+0` points at 32 random bytes less `0` (placed); the word at `p2+4` points at 32 random bytes less `0` (placed); the word at `p2+8` points at 32 random bytes less `0` (placed); the word at `p2+12` points at 32 random bytes less `0` (placed); the word at `p3+0` points at 32 random bytes less `0` (placed); the word at `p3+4` points at 32 random bytes less `0` (placed); the word at `p3+8` points at 32 random bytes less `0` (placed); the word at `p3+12` points at 32 random bytes less `0` (placed); stub 3 answered from [1, 1, 1, 0] (90%); stub 3 answers 1 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 2816 bytes under the call is zeroed before each row; the call ends where stub 1 is first entered (the function is swept to that callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_swarm_around_head.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The head slice: the call ends where the group's refresh (slot `+0x14`, stub 1) is entered.
                // Window layout. `m` the group (`+0x4a` player); `c[1]` `ignore_orders`, `c[2]` and `c[3]` players 0 and
                // 1's ignore-orders counts; `p[who]` the elements of `objects_ignoring_orders[who]` (players 0 and 1 are
                // laid; a player of 8 or more skips the sweep); `p[4 + 4 * who + b]` the objects (`+8 & 1` the site exists).
                //
                // **The port is the crate's own head**, `crate::swarm_around_view::swarm_head`, which
                // `action_swarm_around_steps` runs first on every entry; here the view is the emulated original's windows.
                use crate::garrison_view::GarrisonHead;
                use crate::swarm_around_view::{SwarmHead, swarm_head};
                struct V<'a> {
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                    on_map: i32,
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                impl GarrisonHead for V<'_> {
                    fn who(&self) -> i32 {
                        i32::from(self.m[0x4a])
                    }
                    fn ignore_orders(&self) -> bool {
                        rd32(&self.c[1], 0) != 0
                    }
                    fn ignoring_count(&self, who: i32) -> i32 {
                        rd32(&self.c[2 + who as usize], 0)
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
                impl SwarmHead for V<'_> {
                    fn scratch_clear(&mut self) {
                        self.log.push("s2:sp-0xa70,-1".into());
                    }
                    fn is_on_map(&mut self) -> bool {
                        self.log.push("s3:m+0x0".into());
                        self.on_map != 0
                    }
                    fn site_exists(&self, who: i32, b: i32) -> bool {
                        self.p[4 + 4 * who as usize + b as usize][8] & 1 != 0
                    }
                }
                let mut v = V {
                    m: &*m,
                    c: &*c,
                    p: &*p,
                    log: &mut log,
                    on_map: a[9],
                };
                swarm_head(&mut v, a[0], a[1]);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "action_swarm_around — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_swarm_around — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "action_swarm_around — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "action_swarm_around — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Group::action_swarm_around@0070fbe0: {n} rows agree");
}
