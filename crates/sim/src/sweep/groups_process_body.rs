//! `Groups::process@006fa210` under the emulator against the port (item 3396; tools/sweep_gen.py, obj with windows).

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

/// `Groups::process@006fa210`, run by `tools/emu/sweep_groups_process_body.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), `p` windows in order: p0..p9 the place(s), in flag order; the global cell 0x00c0618c points at 768 random bytes; the global cell 0x00e85f20 points at 2516 random bytes; 4 random bytes at 0x00e3a390 (each word in 1..1); 4 random bytes at 0x00e85f50 (each word in 0..0); vtable slot +0x18 (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; callee 0x007081f0 (0 args) stubbed; callee 0x0070ccb0 (1 args) stubbed; callee 0x0060aae0 (0 args) stubbed; the word at `c0+0x14+0x1c*c[1][0x4a]` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*c[1][0x4a]` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 144 random bytes less `0` (placed); the word at `p5+4` points at 144 random bytes less `0` (placed); the word at `p5+8` points at 144 random bytes less `0` (placed); the word at `p5+12` points at 144 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 8 values in call order, printed after the answers; stub 3 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 900 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_groups_process_body.py") else {
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
                // **The per-group body, with the driver around it.** `Groups::process` through `groups_process_view`
                // (not the crate's `Sim::groups_process`, a namesake), the group's body being `normalize_steps`
                // (`Group::normalize@00711540` inlined, the sweep of item 3387): `c[0]` the objects cell's table,
                // `c[1]` the group table's one group (index 0: leader 0 in use, the cursor 0 — the other seven
                // leaders' flags are the image's zeroes, and the draw's only call is gid 0), `c[2]` the leader byte,
                // `c[3]` the cursor; `p[1+id]` the object, `p[6+id]` the unit. `a`: the five stubs' answers
                // (`a[..5]`), then `--seq 0`'s eight slot-`+0x18` answers (`a[5..13]`) and `--seq 1`'s eight
                // slot-`+0x20` answers (`a[13..21]`).
                use crate::groups_process_view::{GroupsProcessView, groups_process_steps};
                use crate::normalize_view::{NormalizeView, normalize_steps};
                struct G<'a> {
                    a: &'a [i32],
                    g: &'a mut [u8],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 2],
                }
                impl G<'_> {
                    fn seq(&mut self, k: usize) -> i32 {
                        let i = self.turn[k].min(7);
                        self.turn[k] += 1;
                        self.a[5 + 8 * k + i]
                    }
                }
                impl NormalizeView for G<'_> {
                    fn count(&self) -> i32 {
                        rd(self.g, 0xc)
                    }
                    fn set_count(&mut self, n: i32) {
                        wr(self.g, 0xc, n);
                    }
                    fn member(&self, i: i32) -> i32 {
                        let o = 0x8cc + 2 * i as usize;
                        i32::from(i16::from_le_bytes([self.g[o], self.g[o + 1]]))
                    }
                    fn obj_valid(&mut self, m: i32) -> bool {
                        self.p[1 + m as usize][8] & 1 != 0
                    }
                    fn f30(&self) -> i32 {
                        rd(self.g, 0x30)
                    }
                    fn id(&self) -> i32 {
                        rd(self.g, 4)
                    }
                    fn obj_slot18(&mut self, m: i32) -> i32 {
                        self.log.push(format!("s0:p{}+0x0", 1 + m));
                        self.seq(0)
                    }
                    fn obj_slot20(&mut self, m: i32) -> i32 {
                        self.log.push(format!("s1:p{}+0x0", 1 + m));
                        self.seq(1)
                    }
                    fn unit_group(&mut self, m: i32) -> i32 {
                        let w = &self.p[6 + m as usize];
                        i32::from(i16::from_le_bytes([w[0x80], w[0x81]]))
                    }
                    fn shift_down(&mut self, j: i32) {
                        let j = j as usize;
                        for k in 0..4 {
                            let o = 0x4c + 0x200 * k + 4 * j;
                            let w = rd(self.g, o + 4);
                            wr(self.g, o, w);
                        }
                        self.g[0x84c + j] = self.g[0x84c + j + 1];
                        self.g[0x8cc + 2 * j] = self.g[0x8cc + 2 * j + 2];
                        self.g[0x8cc + 2 * j + 1] = self.g[0x8cc + 2 * j + 3];
                    }
                    fn find_role(&mut self) {
                        self.log.push("s2:c1+0x0".to_string());
                    }
                    fn f49(&self) -> i32 {
                        i32::from(self.g[0x49])
                    }
                    fn find_leader(&mut self) -> i32 {
                        self.log.push("s3:c1+0x0,0".to_string());
                        self.a[3]
                    }
                    fn unit_speed(&mut self, l: i32) -> i32 {
                        self.log.push(format!("s4:p{}+0x0", 6 + l));
                        self.a[4]
                    }
                    fn set_speed(&mut self, v: i32) {
                        wr(self.g, 0x40, v);
                        wr(self.g, 0x3c, v);
                    }
                }
                struct V<'a> {
                    a: &'a [i32],
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                }
                impl GroupsProcessView for V<'_> {
                    fn leader_in_use(&self, k: usize) -> bool {
                        k == 0 && self.c[2][0] & 1 != 0
                    }
                    fn cursor(&self) -> i32 {
                        rd(&self.c[3], 0)
                    }
                    fn set_cursor(&mut self, c: i32) {
                        wr(&mut self.c[3], 0, c);
                    }
                    fn normalize_group(&mut self, gid: i32) {
                        assert_eq!(gid, 0, "the draw's one group");
                        let mut g = G {
                            a: self.a,
                            g: &mut self.c[1],
                            p: &mut *self.p,
                            log: &mut *self.log,
                            turn: [0; 2],
                        };
                        normalize_steps(&mut g);
                    }
                }
                let mut v = V {
                    a,
                    c,
                    p,
                    log: &mut log,
                };
                groups_process_steps(&mut v);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "process — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 900, "only {n} rows");
    eprintln!("Groups::process@006fa210: {n} rows agree");
}
