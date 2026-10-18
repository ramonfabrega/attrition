//! `Army::do_marching@006f3df0` under the emulator against the port (item 3238; tools/sweep_gen.py, obj with windows).

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

/// `Army::do_marching@006f3df0`, run by `tools/emu/sweep_army_do_marching_found.py` under unicorn
/// (tools/emu): 152 random bytes of object, 0 argument(s), `p` windows in order: p0..p9 the place(s), in flag order, p10 stub 3's answer window; the global cell 0x00c0618c points at 128 random bytes; 56792 random bytes at 0x00e3a390 (each word in 0..1); callee 0x006f9120 (2 args) stubbed; callee 0x006f8ea0 (0 args) stubbed; callee 0x006ebaa0 (1 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x00738030 (0 args) stubbed; vtable slot +0x11c (1 args) stubbed; vtable slot +0x188 (0 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x006d5b90 (1 args) stubbed; callee 0x006f69b0 (0 args) stubbed; callee 0x006f43c0 (1 args) stubbed; callee 0x006f4d80 (0 args) stubbed; the word at `c0+0x14+0x1c*m[0x34]` points at 20 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `p0+16` points at 32 random bytes less `0` (placed); the word at `0xc09970+0x1c*m[0x34]` points at 12 random bytes less `0` (placed); the word at `p6+0` points at 16 random bytes less `0` (placed); the word at `p6+4` points at 16 random bytes less `0` (placed); the word at `p6+8` points at 16 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 2, 2, 3, 0] (90%); stub 2 answered from [0, 0, 0, 1] (90%); stub 4 answered from [0, 0, 0, 1] (90%); stub 5 answered from [0, 1, 4, 40, 100, 101] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0, 0, 0, 1] (90%); stub 8 answered from [0, 0, 0, 1] (90%); stub 3 answers the address of 128 random bytes, a `p` window after the places; the word at `p10+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 writes 1 (4 bytes) at `m+0x30` each call; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_do_marching_found.py") else {
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
            {
                // `a`: the twelve stubs' single answers, in the order the line names them: 0 `count` (2, 0), 1 `close`,
                // 2 `is_enemy`, 3 slot +0xac's address (the data record, p10), 4 `is_unassimilated`, 5 slot +0x11c's hits,
                // 6 slot +0x188's under-attack answer, 7 `is_ally`, 8 `is_tribute_period`, 9 `find_target`, 10 `do_forming`,
                // 11 `march_to_target`. Windows: `c[1]` the leaders table (stride 0x6eec; `strong` +0x230 and `weak` +0x250
                // a word a player), `p6` the cities' array and `p7..p9` their records (flags the ushort at +4), `p10` the
                // target's data record (flags +8, damage +0x24, build masks +0x60, city number the short at +0x72, the
                // vtable word at +0 either `Build::vftable` or the stubbed table).
                //
                // **The port is the crate's own steps**, `crate::army::do_marching_steps`, which `Sim::do_marching`
                // calls over its `MarchSim`; here the view is the emulated original's windows.
                use crate::army::{MarchView, do_marching_steps};
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                    found: Option<i32>,
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn rd16(w: &[u8], o: usize) -> i32 {
                    i16::from_le_bytes(w[o..o + 2].try_into().expect("two bytes")) as i32
                }
                impl V<'_> {
                    fn leader(&self, off: usize) -> i32 {
                        let idx = rd16(self.m, 0x94) as usize;
                        let tw = rd32(self.m, 0x34) as usize;
                        rd32(&self.c[1], idx * 0x6eec + off + 4 * tw)
                    }
                }
                impl MarchView for V<'_> {
                    fn attack_count(&mut self) -> i32 {
                        self.a[0]
                    }
                    fn close(&mut self) {
                        self.log[0] = "m+0x0".into();
                    }
                    fn has_target(&self) -> bool {
                        rd32(self.m, 0x30) >= 0
                    }
                    fn clear_target(&mut self) {
                        self.m[0x30..0x34].copy_from_slice(&(-1i32).to_le_bytes());
                        self.m[0x34..0x38].copy_from_slice(&(-1i32).to_le_bytes());
                    }
                    fn status(&self) -> i32 {
                        rd32(self.m, 4)
                    }
                    fn set_status(&mut self, s: i32) {
                        self.m[4..8].copy_from_slice(&s.to_le_bytes());
                    }
                    fn target_valid(&mut self) -> bool {
                        self.p[10][8] & 1 != 0
                    }
                    fn target_city_centre(&mut self) -> bool {
                        self.p[10][8] & 0x20 != 0
                    }
                    fn target_enemy(&mut self) -> bool {
                        self.a[2] != 0
                    }
                    fn target_under_attack(&mut self) -> bool {
                        if rd32(&self.p[10], 0) as u32 == 0x00b4_2174 {
                            rd16(&self.p[10], 0x60) & 0x20 != 0
                        } else {
                            self.a[6] != 0
                        }
                    }
                    fn target_city_flags(&mut self) -> Option<i32> {
                        let n = rd16(&self.p[10], 0x72);
                        (n >= 0).then(|| rd16(&self.p[7 + n as usize], 4) & 0xffff)
                    }
                    fn navy(&self) -> bool {
                        rd32(self.m, 0x24) != 0
                    }
                    fn city_unassimilated(&mut self) -> bool {
                        self.a[4] != 0
                    }
                    fn target_hits(&mut self) -> i32 {
                        self.a[5]
                    }
                    fn target_damage(&mut self) -> i32 {
                        rd32(&self.p[10], 0x24)
                    }
                    fn ally(&mut self) -> bool {
                        self.a[7] != 0
                    }
                    fn weak(&mut self) -> bool {
                        self.leader(0x250) != 0
                    }
                    fn strong(&mut self) -> bool {
                        self.leader(0x230) != 0
                    }
                    fn tribute(&mut self) -> bool {
                        self.a[8] != 0
                    }
                    fn find_target(&mut self) -> bool {
                        self.log[1] = "m+0x0".into();
                        if let Some(v) = self.found {
                            self.m[0x30..0x34].copy_from_slice(&v.to_le_bytes());
                        }
                        true
                    }
                    fn do_forming(&mut self) {
                        self.log[2] = "m+0x0,m+0x0".into();
                    }
                    fn march_to_target(&mut self) {
                        self.log[3] = "m+0x0".into();
                    }
                }
                log.extend(["-", "-", "-", "-"].map(String::from));
                let mut v = V {
                    a,
                    m,
                    c: &*c,
                    p: &*p,
                    log: &mut log,
                    found: Some(1),
                };
                do_marching_steps(&mut v);
                want_eax
            }
        };
        assert_eq!(got, want_eax, "do_marching — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "do_marching — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_marching — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_marching — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "do_marching — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2400, "only {n} rows");
    eprintln!("Army::do_marching@006f3df0: {n} rows agree");
}
