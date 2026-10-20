//! `Group::redo_patrol_order@00706d90` under the emulator against the port (item 3379; tools/sweep_gen.py, obj with windows).

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

/// `Group::redo_patrol_order@00706d90`, run by `tools/emu/sweep_group_redo_patrol_order.py` under unicorn
/// (tools/emu): 2272 random bytes of object, 2 argument(s), `p` windows in order: p0 argument 0, p1..p33 the place(s), in flag order; the global cell 0x00c0618c points at 768 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; argument 0 names 96 random bytes; callee 0x00711540 (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xc0 (0 args) stubbed; callee 0x005e4560 (10 args) stubbed; callee 0x005e35e0 (1 args) stubbed; callee 0x00434630 (1 args) stubbed; the word at `c0+0x14+0x1c*0` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p1+0` points at 160 random bytes less `0` (placed); the word at `p1+4` points at 160 random bytes less `0` (placed); the word at `p1+8` points at 160 random bytes less `0` (placed); the word at `p1+12` points at 160 random bytes less `0` (placed); the word at `p2+0` points at 160 random bytes less `0` (placed); the word at `p2+4` points at 160 random bytes less `0` (placed); the word at `p2+8` points at 160 random bytes less `0` (placed); the word at `p2+12` points at 160 random bytes less `0` (placed); the word at `p3+0x18` points at 512 random bytes less `0` (placed); the word at `p4+0x18` points at 512 random bytes less `0` (placed); the word at `p5+0x18` points at 512 random bytes less `0` (placed); the word at `p6+0x18` points at 512 random bytes less `0` (placed); the word at `p7+0x18` points at 512 random bytes less `0` (placed); the word at `p8+0x18` points at 512 random bytes less `0` (placed); the word at `p9+0x18` points at 512 random bytes less `0` (placed); the word at `p10+0x18` points at 512 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p19+0` points at 64 random bytes less `0` (placed); the word at `p19+4` points at 64 random bytes less `0` (placed); the word at `p19+8` points at 64 random bytes less `0` (placed); the word at `p19+12` points at 64 random bytes less `0` (placed); the word at `p20+0` points at 64 random bytes less `0` (placed); the word at `p20+4` points at 64 random bytes less `0` (placed); the word at `p20+8` points at 64 random bytes less `0` (placed); the word at `p20+12` points at 64 random bytes less `0` (placed); the word at `p0+0x14` points at 64 random bytes less `0` (placed); the word at `p0+0x30` points at 64 random bytes less `0` (placed); stub 5's answer points at 80 random bytes less `0` (placed); the word at `p31+0x14` points at 64 random bytes less `0` (placed); the word at `p31+0x30` points at 64 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 1, 1, 0] (90%); stub 2 answered from [1, 1, 1, 1, 0] (90%); stub 3 answered from [0, 0, 0, 0, 0, 1] (90%); stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 4 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_redo_patrol_order.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Window layout. `m` the group (`+0xc` count, `+0x4a` player, the shorts at `+0x8cc`); `c[0]` the objects
            // cell, `c[1]` `div_3_table`. `p[0]` the saved order (`+8` points, `+0x3c` waypoint, `+0x44..+0x50`,
            // `+0x14`/`+0x30` the arrays `p[29]`/`p[30]`); `p[3 + 4 * pl + j]` the objects, `p[11 + ..]` their data
            // (`+0x1e4` the masks), `p[21 + ..]` the units; `p[31]` the order `update_patrol_order` answers
            // (arrays `p[32]`/`p[33]`). `a`: the two arguments, the stubs' answers (`a[2..9]`), then the `--seq`
            // runs of stubs 1, 2 and 3 (`a[9..13]`, `a[13..17]`, `a[17..21]`).
            //
            // **The port is the crate's own steps**, `crate::redo_patrol_view::redo_patrol_order_steps`, which
            // `Sim::group_redo_patrol_order` runs over its `SimRedoPatrol`; here the view is the original's windows.
            use crate::redo_patrol_view::{RedoPatrolView, redo_patrol_order_steps};
            struct V<'a> {
                a: &'a [i32],
                m: &'a [u8],
                c: &'a [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                turn: [usize; 4],
            }
            impl V<'_> {
                fn who(&self) -> usize {
                    self.m[0x4a] as usize
                }
                fn seq(&mut self, k: usize) -> i32 {
                    let i = self.turn[k].min(3);
                    self.turn[k] += 1;
                    self.a[9 + 4 * (k - 1) + i]
                }
                fn rec(&mut self, k: usize, ecx: String, args: &[i32]) {
                    let mut t = format!("s{k}:{ecx}");
                    for v in args {
                        t.push_str(&format!(",{v}"));
                    }
                    self.log.push(t);
                }
                fn uname(&self, m: i32) -> String {
                    format!("p{}+0x0", 21 + 4 * self.who() + m as usize)
                }
            }
            impl RedoPatrolView for V<'_> {
                fn normalize(&mut self) {
                    self.rec(0, "m+0x0".to_string(), &[]);
                }
                fn count(&self) -> i32 {
                    rd(self.m, 0xc)
                }
                fn member(&self, i: i32) -> i32 {
                    let o = 0x8cc + 2 * i as usize;
                    i32::from(i16::from_le_bytes([self.m[o], self.m[o + 1]]))
                }
                fn obj_alive(&mut self, _m: i32) -> bool {
                    self.seq(1) != 0
                }
                fn obj_on_map(&mut self, _m: i32) -> bool {
                    self.seq(2) != 0
                }
                fn obj_plane(&mut self, _m: i32) -> bool {
                    self.seq(3) != 0
                }
                fn obj_masks(&mut self, m: i32) -> u32 {
                    rd(&self.p[11 + 4 * self.who() + m as usize], 0x1e4) as u32
                }
                fn p_word(&self, off: i32) -> i32 {
                    rd(&self.p[0], off as usize)
                }
                fn p_point(&self, axis: i32, i: i32) -> i32 {
                    rd(&self.p[29 + axis as usize], 4 * i as usize)
                }
                fn div3(&self, i: i32) -> i32 {
                    rd(&self.c[1], 4 * i as usize)
                }
                fn add_patrol_order(&mut self, m: i32, a: [i32; 10]) {
                    let e = self.uname(m);
                    self.rec(4, e, &a);
                }
                fn update_patrol_order(&mut self, m: i32, id: i32) {
                    let e = self.uname(m);
                    self.rec(5, e, &[id]);
                }
                fn o_word(&self, off: i32) -> i32 {
                    rd(&self.p[31], off as usize)
                }
                fn o_short(&self, off: i32) -> i32 {
                    let o = off as usize;
                    i32::from(i16::from_le_bytes([self.p[31][o], self.p[31][o + 1]]))
                }
                fn o_set(&mut self, off: i32, v: i32) {
                    wr(&mut self.p[31], off as usize, v);
                }
                fn grow(&mut self, off: i32, n: i32) {
                    self.rec(6, format!("p31+0x{off:x}"), &[n]);
                }
                fn copy_point(&mut self, off: i32) {
                    let o = off as usize;
                    let (x, y) = (rd(&self.p[29], o), rd(&self.p[30], o));
                    wr(&mut self.p[32], o, x);
                    wr(&mut self.p[33], o, y);
                }
            }
            let mut v = V {
                a,
                m,
                c,
                p,
                log: &mut log,
                turn: [0; 4],
            };
            redo_patrol_order_steps(&mut v);
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "redo_patrol_order — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "redo_patrol_order — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "redo_patrol_order — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "redo_patrol_order — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Group::redo_patrol_order@00706d90: {n} rows agree");
}
