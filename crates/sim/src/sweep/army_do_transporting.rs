//! `Army::do_transporting@006f4690` under the emulator against the port (item 3259; tools/sweep_gen.py, obj with windows).

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

/// `Army::do_transporting@006f4690`, run by `tools/emu/sweep_army_do_transporting.py` under unicorn
/// (tools/emu): 152 random bytes of object, 0 argument(s), `p` windows in order: p0..p14 the place(s), in flag order; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 64 random bytes; the global cell 0x00c061b8 points at 32 random bytes; 56792 random bytes at 0x00e3a390; callee 0x006810f0 (1 args) stubbed; callee 0x006f5cc0 (3 args) stubbed; callee 0x006f69b0 (0 args) stubbed; callee 0x00739340 (0 args) stubbed; callee 0x0046cff0 (0 args) run, not hooked; the word at `c0+0x134` points at 1024 random bytes less `0` (placed); the word at `c2+0x10` points at 8704 random bytes less `0` (placed); the word at `p1+124` points at 8 random bytes less `0` (placed); the word at `p1+260` points at 8 random bytes less `0` (placed); the word at `p1+396` points at 8 random bytes less `0` (placed); the word at `p1+532` points at 8 random bytes less `0` (placed); the word at `p1+668` points at 8 random bytes less `0` (placed); the word at `p1+804` points at 8 random bytes less `0` (placed); the word at `p1+940` points at 8 random bytes less `0` (placed); the word at `p1+1076` points at 8 random bytes less `0` (placed); the word at `0xc09970+0x1c*m[0x94]` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 32 random bytes less `0` (placed); the word at `p10+4` points at 32 random bytes less `0` (placed); the word at `p10+8` points at 32 random bytes less `0` (placed); the word at `p10+12` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 2, 4, 6, 1, 3, 8, 0, 0] (90%); stub 1 answered from [0, 1] (90%); stub 3 answered from [0, 1, 2, 3, 5, -1] (90%); stub 0 answers 8 values in call order, printed after the answers; stub 3 answers 16 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_do_transporting.py") else {
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
            // `a`: 0..=4 the single answers of the five callees (only 1, `find_muster_spot`, is read), 5..13 what
            // `go_here` answers its calls in order, 13..29 what `get_level` answers. Windows: `c0` the world
            // (width +0, height +4, the cells' pointer +0x134), `c1` the div_3 table, `c2` the regions' object, `c3`
            // the leaders table (stride 0x6eec; city count +0x408, strategy words +0xa68), `p0` the world's cells
            // (stride 0x1c, region the short at +4), `p1` the regions (stride 0x88; size +0x14), `p2..p9` the
            // regions' first cells, `p10` the leader's city pointers, `p11..p14` the city records (flags the ushort at
            // +4, building the short at +8, region the short at +10, centre at +0xc / +0x10).
            //
            // **The port is the crate's own steps**, `crate::army::do_transporting_steps`, which
            // `Sim::do_transporting` calls over its `TransSim`; here the view is the emulated original's windows.
            use crate::army::{TransView, do_transporting_steps};
            fn rd32(w: &[u8], o: usize) -> i32 {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            }
            fn rd16(w: &[u8], o: usize) -> i32 {
                i32::from(i16::from_le_bytes(
                    w[o..o + 2].try_into().expect("two bytes"),
                ))
            }
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a [Vec<u8>],
                p: &'a [Vec<u8>],
                l0: Vec<String>,
                l1: Vec<String>,
                l2: Vec<String>,
                l3: Vec<String>,
            }
            impl V<'_> {
                fn who(&self) -> i32 {
                    rd16(self.m, 0x94)
                }
                fn div3(&self, v: i32) -> i32 {
                    rd32(&self.c[1], 4 * (v >> 8) as usize)
                }
                fn cell_region(&self) -> i32 {
                    let idx = self.div3(rd32(self.m, 0x3c)) * rd32(&self.c[0], 0)
                        + self.div3(rd32(self.m, 0x38));
                    rd16(&self.p[0], 28 * idx as usize + 4)
                }
                fn first(&self, r: usize) -> (i32, i32) {
                    let w = &self.p[2 + r];
                    (rd32(w, 0), rd32(w, 4))
                }
                fn city(&self, k: usize) -> &Vec<u8> {
                    &self.p[11 + k]
                }
            }
            impl TransView for V<'_> {
                fn at_home(&mut self) -> bool {
                    self.cell_region() == rd32(self.m, 8)
                }
                fn my_region(&self) -> Option<u16> {
                    Some(rd32(self.m, 8) as u16)
                }
                fn regions(&mut self, my: u16) -> Vec<u16> {
                    (1..64u16).filter(|&r| r != my).collect()
                }
                fn region_size(&mut self, r: u16) -> i32 {
                    rd32(&self.p[1], 0x88 * r as usize + 0x14)
                }
                fn strategy(&mut self, r: u16) -> i32 {
                    let o = 0x6eec * self.who() as usize + 0xa68 + 2 * r as usize;
                    i32::from(u16::from_le_bytes([self.c[3][o], self.c[3][o + 1]]))
                }
                fn go_here(&mut self, r: u16) -> i32 {
                    let k = self.l0.len();
                    self.l0
                        .push(format!("p1+{:#x},{}", 0x88 * r as usize, self.who()));
                    self.a[5 + k.min(7)]
                }
                fn distance(&mut self, my: u16, r: u16) -> i32 {
                    let (mx, my_) = self.first(my as usize);
                    let (tx, ty) = self.first(r as usize);
                    crate::world::vector_dist(
                        mx.wrapping_sub(tx).wrapping_abs(),
                        my_.wrapping_sub(ty).wrapping_abs(),
                    )
                }
                fn cities(&mut self) -> Vec<usize> {
                    (0..rd32(&self.c[3], 0x6eec * self.who() as usize + 0x408).max(0) as usize)
                        .collect()
                }
                fn city_in(&mut self, k: usize, r: u16) -> bool {
                    let rec = self.city(k);
                    let flags = rd16(rec, 4) & 0xffff;
                    flags & 1 != 0 && rd16(rec, 10) == i32::from(r) && flags & 2 != 0
                }
                fn city_level(&mut self, k: usize) -> i32 {
                    let n = self.l3.len();
                    self.l3.push(format!("p{}+0x0", 11 + k));
                    self.a[13 + n.min(15)]
                }
                fn set_region(&mut self, r: Option<u16>) {
                    wr(self.m, 8, i32::from(r.expect("a region")));
                }
                fn find_target(&mut self) -> bool {
                    self.l2.push("m+0x0".to_string());
                    true
                }
                fn has_target(&self) -> bool {
                    rd32(self.m, 0x30) >= 0
                }
                fn region_home(&mut self) {
                    let r = self.cell_region();
                    wr(self.m, 8, r);
                }
                fn target_city(&mut self, k: usize) {
                    let (who, bld) = (self.who(), rd16(self.city(k), 8));
                    wr(self.m, 0x34, who);
                    wr(self.m, 0x20, k as i32);
                    wr(self.m, 0x30, bld);
                }
                fn find_muster_spot(&mut self, k: usize) -> bool {
                    let bld = rd16(self.city(k), 8);
                    self.l1.push(format!("m+0x0,{bld},{},1", self.who()));
                    self.a[1] != 0
                }
                fn muster_fallback(&mut self, k: usize) {
                    let (cx, cy) = (rd32(self.city(k), 0xc), rd32(self.city(k), 0x10));
                    let (x, mut y) = (self.div3(cx), self.div3(cy));
                    y += if y >= rd32(&self.c[0], 4) - 1 { -1 } else { 1 };
                    wr(self.m, 0x48, x);
                    wr(self.m, 0x4c, y);
                }
                fn set_marching(&mut self) {
                    wr(self.m, 4, 2);
                }
            }
            let mut v = V {
                a,
                m,
                c,
                p,
                l0: Vec::new(),
                l1: Vec::new(),
                l2: Vec::new(),
                l3: Vec::new(),
            };
            let acted = do_transporting_steps(&mut v);
            for l in [v.l0, v.l1, v.l2, v.l3] {
                log.push(if l.is_empty() {
                    "-".to_string()
                } else {
                    l.join("/")
                });
            }
            i64::from(acted)
        };
        assert_eq!(got, want_eax, "do_transporting — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "do_transporting — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_transporting — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "do_transporting — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "do_transporting — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Army::do_transporting@006f4690: {n} rows agree");
}
