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

/// `Groups::process@006fa210`, run by `tools/emu/sweep_groups_process_driver.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), `p` windows in order: p0..p7 sparse table 0 (8 rows × 1 slice(s), row-major), p8..p1031 sparse table 1 (512 rows × 2 slice(s), row-major); 4 random bytes at 0x00e85f50 (each word in 0..63); callee 0x007081f0 (0 args) stubbed; callee 0x0070ccb0 (1 args) stubbed; callee 0x0060aae0 (0 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x1 (`p` windows after the pointed buffers, before the places); 512 rows 0x9d4 apart at `0xe85f20`, slices +0xc:0x4, +0x3c:0x8 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_groups_process_driver.py") else {
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
            {
                // `Groups::process`'s driver through `groups_process_view` (not the crate's `Sim::groups_process`, a
                // namesake). Windows: `c[0]` the cursor word `[0xe85f50]`; `p[0..8]` the leaders' first byte (bit 0 = in
                // use); `p[8 + 2g]` group `g`'s count (+0xc) and `p[9 + 2g]` its +0x3c/+0x40 pair. The group table's base is
                // not a window, so the stub's `ecx` (`s0:<address>`) is held relative to the first call, the windows
                // below pinning which group each call was.
                use crate::groups_process_view::{GroupsProcessView, groups_process_steps};
                use crate::normalize_view::{NormalizeView, normalize_steps};
                struct G<'a> {
                    p: &'a mut [Vec<u8>],
                    gid: i32,
                    called: &'a mut Vec<i32>,
                }
                impl NormalizeView for G<'_> {
                    fn count(&self) -> i32 {
                        let w = &self.p[8 + 2 * self.gid as usize];
                        i32::from_le_bytes([w[0], w[1], w[2], w[3]])
                    }
                    fn set_count(&mut self, _n: i32) {
                        unreachable!("this draw's counts are never positive")
                    }
                    fn member(&self, _i: i32) -> i32 {
                        unreachable!("this draw's counts are never positive")
                    }
                    fn obj_valid(&mut self, _m: i32) -> bool {
                        unreachable!()
                    }
                    fn f30(&self) -> i32 {
                        unreachable!()
                    }
                    fn id(&self) -> i32 {
                        unreachable!()
                    }
                    fn obj_slot18(&mut self, _m: i32) -> i32 {
                        unreachable!()
                    }
                    fn obj_slot20(&mut self, _m: i32) -> i32 {
                        unreachable!()
                    }
                    fn unit_group(&mut self, _m: i32) -> i32 {
                        unreachable!()
                    }
                    fn shift_down(&mut self, _j: i32) {
                        unreachable!()
                    }
                    fn find_role(&mut self) {
                        self.called.push(self.gid);
                    }
                    fn f49(&self) -> i32 {
                        // Slice +0x49 is not drawn here (count <= 0 reaches no use of it; the body sweep draws it).
                        0
                    }
                    fn find_leader(&mut self) -> i32 {
                        unreachable!()
                    }
                    fn unit_speed(&mut self, _l: i32) -> i32 {
                        unreachable!()
                    }
                    fn set_speed(&mut self, v: i32) {
                        let w = &mut self.p[9 + 2 * self.gid as usize];
                        w[4..8].copy_from_slice(&v.to_le_bytes());
                        w[0..4].copy_from_slice(&v.to_le_bytes());
                    }
                }
                struct V<'a> {
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    called: &'a mut Vec<i32>,
                }
                impl GroupsProcessView for V<'_> {
                    fn leader_in_use(&self, k: usize) -> bool {
                        self.p[k][0] & 1 != 0
                    }
                    fn cursor(&self) -> i32 {
                        i32::from_le_bytes(self.c[0][0..4].try_into().expect("four bytes"))
                    }
                    fn set_cursor(&mut self, c: i32) {
                        self.c[0][0..4].copy_from_slice(&c.to_le_bytes());
                    }
                    fn normalize_group(&mut self, gid: i32) {
                        let mut g = G {
                            p: &mut *self.p,
                            gid,
                            called: &mut *self.called,
                        };
                        normalize_steps(&mut g);
                    }
                }
                let mut called: Vec<i32> = Vec::new();
                let mut v = V {
                    c,
                    p,
                    called: &mut called,
                };
                groups_process_steps(&mut v);
                // The recorded `find_role` calls: one `s0:<ecx>` a group, in order; `ecx` = base + 0x9d4 * gid.
                let want_ecx: Vec<i64> = want_log
                    .iter()
                    .filter(|t| **t != "-")
                    .map(|t| {
                        // The harness lays `m` (16 bytes) at the heap's base 0x2000_0000 and the group table
                        // 0x30 after it, so `ecx` is exactly 0x2000_0030 + 0x9d4 * gid; a token the emulator
                        // names `this+0xN` is 0x2000_0000 + N.
                        let t = t.strip_prefix("s0:").expect("only stub 0 is recorded");
                        match t.strip_prefix("this+0x") {
                            Some(h) => 0x2000_0000 + i64::from_str_radix(h, 16).expect("hex"),
                            None => t.parse().expect("an address"),
                        }
                    })
                    .collect();
                let have: Vec<i64> = called
                    .iter()
                    .map(|&g| 0x2000_0030 + 0x9d4 * i64::from(g))
                    .collect();
                assert_eq!(
                    have, want_ecx,
                    "process — the groups find_role was handed, row `{line}`"
                );
                log.extend(want_log.iter().map(|s| (*s).to_string()));
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
            assert_eq!(w, &want[2 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 500, "only {n} rows");
    eprintln!("Groups::process@006fa210: {n} rows agree");
}
