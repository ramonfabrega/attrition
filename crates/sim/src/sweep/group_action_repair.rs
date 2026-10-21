//! `Group::action_repair@007020c0` under the emulator against the port (item 3411; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_repair@007020c0`, run by `tools/emu/sweep_group_action_repair.py` under unicorn
/// (tools/emu): 2272 random bytes of object, 3 argument(s), `p` windows in order: p0..p28 the place(s), in flag order; the global cell 0x00c0618c points at 768 random bytes; the global cell 0x00c06188 points at 64 random bytes; the global cell 0x00c061b8 points at 64 random bytes; the global cell 0x00c061a8 points at 64 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; 4 random bytes at 0x00cc02f8 (each word in 0..0); vtable slot +0x10 (4 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x0070e520 (1 args) stubbed; callee 0x0070d0c0 (1 args) stubbed; callee 0x0070e620 (1 args) stubbed; callee 0x0046f600 (0 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x00680f90 (1 args) stubbed; callee 0x00675bc0 (3 args) stubbed; callee 0x005e4a60 (7 args) stubbed; callee 0x005e4ff0 (4 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; the word at `c3+0x10` points at 2656 random bytes less `0` (placed); the word at `c0+0x14+0x1c*0` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); the word at `p1+8` points at 64 random bytes less `0` (placed); the word at `p1+12` points at 64 random bytes less `0` (placed); the word at `p2+0` points at 64 random bytes less `0` (placed); the word at `p2+4` points at 64 random bytes less `0` (placed); the word at `p2+8` points at 64 random bytes less `0` (placed); the word at `p2+12` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p11+0` points at 256 random bytes less `0` (placed); the word at `p11+4` points at 256 random bytes less `0` (placed); the word at `p11+8` points at 256 random bytes less `0` (placed); the word at `p11+12` points at 256 random bytes less `0` (placed); the word at `p12+0` points at 256 random bytes less `0` (placed); the word at `p12+4` points at 256 random bytes less `0` (placed); the word at `p12+8` points at 256 random bytes less `0` (placed); the word at `p12+12` points at 256 random bytes less `0` (placed); the word at `p13+0x18` points at 16 random bytes less `0` (placed); the word at `p14+0x18` points at 16 random bytes less `0` (placed); the word at `p15+0x18` points at 16 random bytes less `0` (placed); the word at `p16+0x18` points at 16 random bytes less `0` (placed); the word at `p17+0x18` points at 16 random bytes less `0` (placed); the word at `p18+0x18` points at 16 random bytes less `0` (placed); the word at `p19+0x18` points at 16 random bytes less `0` (placed); the word at `p20+0x18` points at 16 random bytes less `0` (placed); stub 2 answered from [0, 1, 1, 1, 1] (90%); stub 3 answered from [0, 1, 1, 1] (90%); stub 9 answered from [0, 1, 1] (90%); stub 10 answered from [0, 1] (90%); stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 4 values in call order, printed after the answers; stub 8 answers 5 values in call order, printed after the answers; stub 9 answers 4 values in call order, printed after the answers; stub 10 answers 4 values in call order, printed after the answers; stub 8 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 8 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 8 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); the stack 2816 bytes under the call is zeroed before each row; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_repair.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Window layout. `m` the group (`+0xc` count, `+0x10` form, `+0x4a` player, the shorts at `+0x8cc`);
            // `c[0]` the objects cell, `c[1]` the world, `c[2]` the regions (`+0x10` the table), `c[3]` the spell
            // types, `c[4]` `div_3_table`, `c[5]` `ignore_orders`. `p[0]` the spell types' record (`+0xa4c` the
            // spell), `p[3 + 4 * pl + j]` the objects `objects[pl][j]`, `p[13 + 4 * pl + j]` the units, `p[21 + ..]`
            // their data (`+4` the type). `a`: the three arguments, the sixteen stubs' answers (`a[3..19]`) and
            // the `--seq` runs (`SEQ_AT`).
            //
            // **The port is the crate's own steps**, `crate::repair_view::action_repair_steps`, **unwired**
            // (nothing issues the `repair` command); here the view is the emulated original's windows.
            use crate::garrison_view::GarrisonHead;
            use crate::repair_view::{RepairView, action_repair_steps};
            const SEQ_AT: [(usize, usize, usize); 5] =
                [(2, 19, 4), (3, 23, 4), (8, 27, 5), (9, 32, 4), (10, 36, 4)];
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                turn: [usize; 11],
                d0: Vec<String>,
                d1: Vec<String>,
            }
            fn rd32(w: &[u8], o: usize) -> i32 {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            }
            impl V<'_> {
                fn seq(&mut self, k: usize) -> i32 {
                    let &(_, at, n) = SEQ_AT.iter().find(|s| s.0 == k).expect("a seq stub");
                    let i = self.turn[k].min(n - 1);
                    self.turn[k] += 1;
                    self.a[at + i]
                }
                fn hex(v: i32) -> String {
                    v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
                }
                fn obj(p: i32, o: i32) -> usize {
                    3 + 4 * p as usize + o as usize
                }
                fn unit(p: i32, o: i32) -> usize {
                    13 + 4 * p as usize + o as usize
                }
                fn div3(&self, v: i32) -> i32 {
                    rd32(&self.c[4], 4 * ((v ^ 0x63637) >> 6) as usize)
                }
            }
            impl GarrisonHead for V<'_> {
                fn who(&self) -> i32 {
                    i32::from(self.m[0x4a])
                }
                fn ignore_orders(&self) -> bool {
                    rd32(&self.c[5], 0) != 0
                }
                fn ignoring_count(&self, _who: i32) -> i32 {
                    0
                }
                fn ignoring_at(&self, _who: i32, _i: i32) -> i32 {
                    -1
                }
                fn drop_ignored(&mut self, o: i32, who: i32) {
                    self.log.push(format!("s0:m+0x0,{o},{who},0,0"));
                }
                fn refresh(&mut self) {
                    self.log.push("s1:m+0x0".into());
                }
            }
            impl RepairView for V<'_> {
                fn set_up_insert(&mut self) {
                    self.log.push("s4:m+0x0,sp-0x5c".into());
                }
                fn action_halt(&mut self) {
                    self.log.push("s5:m+0x0,0".into());
                }
                fn finish_insert(&mut self) {
                    self.log.push("s6:m+0x0,sp-0x5c".into());
                }
                fn list_clear(&mut self) {
                    self.log.push("s7:sp-0x58".into());
                }
                fn set_form(&mut self) {
                    self.m[0x10..0x14].copy_from_slice(&(-1i32).to_le_bytes());
                }
                fn count(&self) -> i32 {
                    rd32(self.m, 0xc)
                }
                fn member(&self, i: i32) -> i32 {
                    let o = 0x8cc + 2 * i as usize;
                    i32::from(i16::from_le_bytes([self.m[o], self.m[o + 1]]))
                }
                fn tregion(&mut self, p: i32, o: i32, first: bool, depth: i32) -> i32 {
                    let w = &self.p[Self::obj(p, o)];
                    let (x, y) = (rd32(w, 0x10), rd32(w, 0x14));
                    let (dx, dy) = (self.div3(x), self.div3(y));
                    let (a0, a1) = if first { (0x24, 0x28) } else { (0x34, 0x30) };
                    let sh = 0x64 * depth;
                    self.log
                        .push(format!("s8:c1+0x0,sp-0x{:x},sp-0x{:x}", a0 + sh, a1 + sh));
                    self.d0.push(Self::hex(dx));
                    self.d1.push(Self::hex(dy));
                    self.seq(8)
                }
                fn object_slot8(&mut self, who: i32, m: i32) -> i32 {
                    self.log.push(format!("s2:p{}+0x0", Self::obj(who, m)));
                    self.seq(2)
                }
                fn object_slot_bc(&mut self, who: i32, m: i32) -> i32 {
                    self.log.push(format!("s3:p{}+0x0", Self::obj(who, m)));
                    self.seq(3)
                }
                fn unit_type(&self, who: i32, m: i32) -> i32 {
                    rd32(&self.p[Self::unit(who, m) + 8], 4)
                }
                fn is_coast(&mut self, member_tregion: i32, building_tregion: i32) -> i32 {
                    let ecx =
                        rd32(&self.c[2], 0x10).wrapping_add(member_tregion.wrapping_mul(0x88));
                    self.log.push(format!("s9:{ecx},{building_tregion}"));
                    self.seq(9)
                }
                fn is_castable(&mut self, m: i32, who: i32) -> i32 {
                    let ecx = rd32(&self.p[0], 0xa4c);
                    self.log.push(format!("s10:{ecx},{m},{who},0"));
                    self.seq(10)
                }
                fn add_cast_order(&mut self, who: i32, m: i32, queue: i32) {
                    let w = &self.p[Self::obj(who, m)];
                    let (x, y) = (rd32(w, 0x10) ^ 0x63637, rd32(w, 0x14) ^ 0x63637);
                    self.log.push(format!(
                        "s11:p{}+0x0,-1,-1,{x},{y},659,{queue},1",
                        Self::unit(who, m)
                    ));
                }
                fn add_repair_order(&mut self, who: i32, m: i32, b: i32, bwho: i32, queue: i32) {
                    self.log.push(format!(
                        "s12:p{}+0x0,{b},{bwho},{queue},1",
                        Self::unit(who, m)
                    ));
                }
                fn reset_unit(&mut self, bwho: i32, b: i32) {
                    let k = Self::unit(bwho, b);
                    let w = &mut self.p[k];
                    let f = rd32(w, 0x68) & 0xfbff_ffffu32 as i32;
                    w[0x68..0x6c].copy_from_slice(&f.to_le_bytes());
                    w[0xc0..0xc4].copy_from_slice(&0i32.to_le_bytes());
                    self.log.push(format!("s13:p{k}+0x0,0"));
                    self.log.push(format!("s14:p{k}+0x0"));
                    self.log.push(format!("s15:p{k}+0x0"));
                }
            }
            let mut v = V {
                a,
                m: &mut *m,
                c: &*c,
                p: &mut *p,
                log: &mut log,
                turn: [0; 11],
                d0: Vec::new(),
                d1: Vec::new(),
            };
            action_repair_steps(&mut v, a[0], a[1], a[2], 0);
            let (d0, d1) = (v.d0.join("/"), v.d1.join("/"));
            if !d0.is_empty() {
                log.push(d0);
                log.push(d1);
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_repair — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_repair — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_repair — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[7 + i],
                "action_repair — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("Group::action_repair@007020c0: {n} rows agree");
}
