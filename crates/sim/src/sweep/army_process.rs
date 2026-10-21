//! `Army::process@006f93d0` under the emulator against the port (item 3399; tools/sweep_gen.py, obj with windows).

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

/// `Army::process@006f93d0`, run by `tools/emu/sweep_army_process.py` under unicorn
/// (tools/emu): 152 random bytes of object, 1 argument(s), `p` windows in order: p0..p7 sparse table 0 (4 rows × 2 slice(s), row-major), p8..p35 the place(s), in flag order; the global cell 0x00c061ec points at 1364 random bytes; the global cell 0x00c0618c points at 128 random bytes; callee 0x006f9b50 (0 args) stubbed; callee 0x006f9120 (2 args) stubbed; callee 0x006f4c30 (0 args) stubbed; callee 0x006f4af0 (0 args) stubbed; callee 0x006f49a0 (0 args) stubbed; callee 0x006f8ea0 (0 args) stubbed; callee 0x006f98a0 (3 args) stubbed; callee 0x006f5470 (0 args) stubbed; callee 0x006f56d0 (0 args) stubbed; callee 0x006ebaa0 (1 args) stubbed; callee 0x006f5cc0 (3 args) stubbed; callee 0x006f8750 (1 args) stubbed; callee 0x006f4260 (0 args) stubbed; callee 0x006f4070 (0 args) stubbed; callee 0x006f3df0 (0 args) stubbed; callee 0x006f43c0 (1 args) stubbed; callee 0x006f5160 (0 args) stubbed; callee 0x006f4690 (0 args) stubbed; the word at `0xc09970+0x1c*m[0x94]` points at 16 random bytes less `0` (placed); the word at `p8+0` points at 20 random bytes less `0` (placed); the word at `p8+4` points at 20 random bytes less `0` (placed); the word at `p8+8` points at 20 random bytes less `0` (placed); the word at `p8+12` points at 20 random bytes less `0` (placed); the word at `0xc09710+0x1c*m[0x94]` points at 64 random bytes less `0` (placed); the word at `p13+0` points at 48 random bytes less `0` (placed); the word at `p13+4` points at 48 random bytes less `0` (placed); the word at `p13+8` points at 48 random bytes less `0` (placed); the word at `p13+12` points at 48 random bytes less `0` (placed); the word at `p13+16` points at 48 random bytes less `0` (placed); the word at `p13+20` points at 48 random bytes less `0` (placed); the word at `p13+24` points at 48 random bytes less `0` (placed); the word at `p13+28` points at 48 random bytes less `0` (placed); the word at `p13+32` points at 48 random bytes less `0` (placed); the word at `p13+36` points at 48 random bytes less `0` (placed); the word at `p13+40` points at 48 random bytes less `0` (placed); the word at `p13+44` points at 48 random bytes less `0` (placed); the word at `p13+48` points at 48 random bytes less `0` (placed); the word at `p13+52` points at 48 random bytes less `0` (placed); the word at `p13+56` points at 48 random bytes less `0` (placed); the word at `p13+60` points at 48 random bytes less `0` (placed); the word at `c1+0x14+0x1c*m[0x34]` points at 16 random bytes less `0` (placed); the word at `p30+0` points at 24 random bytes less `0` (placed); the word at `p30+4` points at 24 random bytes less `0` (placed); the word at `p30+8` points at 24 random bytes less `0` (placed); the word at `p30+12` points at 24 random bytes less `0` (placed); the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); stub 1 answered from [0, 1, 1] (90%); stub 7 answered from [0, 0, 1] (90%); stub 8 answered from [0, 0, 1] (90%); stub 9 answered from [0, 1, 1] (90%); stub 1 answers 3 values in call order, printed after the answers; stub 8 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 4 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x3f8:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_process.py") else {
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
            // hand:
            {
                // `a`: 0 the argument (the bypass), 1..=18 the stubs' single answers (stub k at 1 + k: 7 `is_moving`,
                // 9 `is_enemy`), 19..=21 `count`'s three answers in call order, 22..=23 `is_engaged`'s two. Stubs: 0 `normalize`,
                // 1 `count`, 2 `use_generals`, 3 `use_spies`, 4 `use_scouts`, 5 `close`, 6 `send_here`, 7 `is_moving`,
                // 8 `is_engaged`, 9 `is_enemy` (its `ecx` the leader's row of the leaders table, `rK+0x0`), 10 `find_muster_spot`,
                // 11 `set_stance`, 12 `do_mustering`, 13 `do_defending`, 14 `do_marching`, 15 `do_forming`, 16 `engagement`,
                // 17 `do_transporting`. Windows: `c0` the game (the frame at +0x550), `c1` the objects (unused: the table is
                // placed), `p0..p7` the leaders' flag word and city count (`p(2k)`, `p(2k+1)`), `p8` the leader's city array and
                // `p9..p12` the cities (flags the byte at +4, point +0xc/+0x10), `p13` the leader's army array and `p14..p29` the
                // armies, `p30` the target player's object array and `p31..p34` the objects (y at +0x10, x at +0x14, both
                // xor 0x63637), `p35` the `div_3_table`. The group count (the short at +0x96) is held at 0, so the merge's
                // member copy (the groups and units tables) is not reached: `merge_into` adds nothing.
                use crate::army_process_view::{ArmyProcessView, Peer, army_process};
                use crate::world::Pos;
                const KEY: i32 = 0x63637;
                fn rd(w: &[u8], o: usize) -> i32 {
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
                    log: &'a mut Vec<String>,
                    counts: usize,
                    engaged: usize,
                }
                impl V<'_> {
                    fn leader(&self) -> usize {
                        rd16(self.m, 0x94) as usize
                    }
                    fn obj(&self) -> &[u8] {
                        &self.p[31 + rd(self.m, 0x30) as usize]
                    }
                    fn div3(&self, coord: i32) -> i32 {
                        rd(&self.p[35], 4 * ((coord ^ KEY) >> 8) as usize)
                    }
                }
                impl ArmyProcessView for V<'_> {
                    fn human_frame(&self) -> i32 {
                        rd(self.m, 0x28)
                    }
                    fn set_human_frame(&mut self, v: i32) {
                        self.m[0x28..0x2c].copy_from_slice(&v.to_le_bytes());
                    }
                    fn frame(&self) -> i32 {
                        rd(&self.c[0], 0x550)
                    }
                    fn phase(&self) -> i32 {
                        (rd16(self.m, 2) + rd16(self.m, 0x94) * 2) * 2
                    }
                    fn defeated(&self) -> bool {
                        self.p[2 * self.leader()][0] & 0x40 != 0
                    }
                    fn num_captains(&self) -> i32 {
                        rd(self.m, 0x14)
                    }
                    fn num_standard(&self) -> i32 {
                        rd(self.m, 0x18)
                    }
                    fn num_decoys(&self) -> i32 {
                        rd(self.m, 0x1c)
                    }
                    fn valid(&self) -> bool {
                        rd16(self.m, 0) != 0
                    }
                    fn status(&self) -> i32 {
                        rd(self.m, 4)
                    }
                    fn set_status(&mut self, s: i32) {
                        self.m[4..8].copy_from_slice(&s.to_le_bytes());
                    }
                    fn navy(&self) -> bool {
                        rd(self.m, 0x24) != 0
                    }
                    fn reg(&self) -> i32 {
                        rd(self.m, 8)
                    }
                    fn ping_pos(&self) -> Pos {
                        Pos::new(rd(self.m, 0x38), rd(self.m, 0x3c))
                    }
                    fn muster(&self) -> (i32, i32) {
                        (rd(self.m, 0x4c), rd(self.m, 0x48))
                    }
                    fn set_muster(&mut self, x: i32, y: i32) {
                        self.m[0x4c..0x50].copy_from_slice(&x.to_le_bytes());
                        self.m[0x48..0x4c].copy_from_slice(&y.to_le_bytes());
                    }
                    fn has_target(&self) -> bool {
                        rd(self.m, 0x30) >= 0 && rd(self.m, 0x34) >= 0
                    }
                    fn target_owner(&self) -> i32 {
                        rd(self.m, 0x34)
                    }
                    fn target_cell(&mut self) -> (i32, i32) {
                        let o = self.obj();
                        let (x, y) = (rd(o, 0x14), rd(o, 0x10));
                        (self.div3(x), self.div3(y))
                    }
                    fn city_count(&self) -> i32 {
                        rd(&self.p[2 * self.leader() + 1], 0)
                    }
                    fn set_scan(&mut self, i: i32) {
                        self.m[0x20..0x24].copy_from_slice(&i.to_le_bytes());
                    }
                    fn city_alive(&mut self, i: i32) -> bool {
                        self.p[9 + i as usize][4] & 1 != 0
                    }
                    fn city_pos(&mut self, i: i32) -> Pos {
                        let c = &self.p[9 + i as usize];
                        Pos::new(rd(c, 0xc), rd(c, 0x10))
                    }
                    fn peer(&mut self, k: usize) -> Peer {
                        let b = &self.p[14 + k];
                        Peer {
                            valid: rd16(b, 0) != 0,
                            navy: rd(b, 0x24) != 0,
                            reg: rd(b, 8),
                            num_captains: rd(b, 0x14),
                            num_standard: rd(b, 0x18),
                            num_decoys: rd(b, 0x1c),
                        }
                    }
                    fn normalize(&mut self) {
                        self.log.push("s0:m+0x0".into());
                    }
                    fn count_type(&mut self, ty: i32) -> i32 {
                        self.log.push(format!("s1:m+0x0,19,{ty}"));
                        let v = self.a[19 + self.counts.min(2)];
                        self.counts += 1;
                        v
                    }
                    fn use_generals(&mut self) {
                        self.log.push("s2:m+0x0".into());
                    }
                    fn use_spies(&mut self) {
                        self.log.push("s3:m+0x0".into());
                    }
                    fn use_scouts(&mut self) {
                        self.log.push("s4:m+0x0".into());
                    }
                    fn send_here(&mut self, p: Pos, kind: i32) {
                        self.log.push(format!("s6:m+0x0,{},{},{kind}", p.x, p.y));
                    }
                    fn close(&mut self) {
                        self.log.push("s5:m+0x0".into());
                    }
                    fn merge_into(&mut self, _k: usize) {
                        assert_eq!(rd16(self.m, 0x96), 0, "the groups walk is not swept");
                    }
                    fn is_moving(&mut self) -> bool {
                        self.log.push("s7:m+0x0".into());
                        self.a[1 + 7] != 0
                    }
                    fn is_engaged(&mut self) -> bool {
                        self.log.push("s8:m+0x0".into());
                        let v = self.a[22 + self.engaged.min(1)];
                        self.engaged += 1;
                        v != 0
                    }
                    fn is_enemy(&mut self, owner: i32) -> bool {
                        self.log.push(format!("s9:r{}+0x0,{owner}", self.leader()));
                        self.a[1 + 9] != 0
                    }
                    fn find_muster_spot(&mut self) {
                        self.log.push(format!(
                            "s10:m+0x0,{},{},0",
                            rd(self.m, 0x30),
                            rd(self.m, 0x34)
                        ));
                    }
                    fn set_stance(&mut self, code: i32) {
                        self.log.push(format!("s11:m+0x0,{code}"));
                    }
                    fn do_mustering(&mut self) {
                        self.log.push("s12:m+0x0".into());
                    }
                    fn do_defending(&mut self) {
                        self.log.push("s13:m+0x0".into());
                    }
                    fn do_marching(&mut self) {
                        self.log.push("s14:m+0x0".into());
                    }
                    fn do_forming(&mut self, ran: bool) {
                        // The stale `ecx` the original pushes: `this` after a
                        // call, else the status it loaded.
                        let arg = if ran {
                            "m+0x0".to_string()
                        } else {
                            rd(self.m, 4).to_string()
                        };
                        self.log.push(format!("s15:m+0x0,{arg}"));
                    }
                    fn engagement(&mut self) {
                        self.log.push("s16:m+0x0".into());
                    }
                    fn do_transporting(&mut self) {
                        self.log.push("s17:m+0x0".into());
                    }
                }
                let bypass = a[0] != 0;
                let mut v = V {
                    a,
                    m,
                    c,
                    p,
                    log: &mut log,
                    counts: 0,
                    engaged: 0,
                };
                army_process(&mut v, bypass);
                if log.is_empty() {
                    log.push("-".into());
                }
                0
            }
            // end hand
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "process — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Army::process@006f93d0: {n} rows agree");
}
