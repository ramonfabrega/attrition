//! `Unit::check_build_order@00603470` under the emulator against the port (item 3372; tools/sweep_gen.py, obj with windows).

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

/// `Unit::check_build_order@00603470`, run by `tools/emu/sweep_unit_check_build_order.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p25 the place(s), in flag order, p26..p36 the stubs' answer window(s); the global cell 0x00c0618c points at 768 random bytes; the global cell 0x00c061f0 points at 256 random bytes; vtable slot +0x10 (0 args) stubbed; vtable slot +0xb4 (0 args) stubbed; vtable slot +0xb8 (0 args) stubbed; vtable slot +0x0 (0 args) stubbed; vtable slot +0x4 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0x100 (0 args) stubbed; vtable slot +0xa8 (0 args) stubbed; vtable slot +0xd8 (0 args) stubbed; callee 0x005e29b0 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x005e5210 (4 args) stubbed; callee 0x0065a620 (11 args) stubbed; callee 0x00608450 (0 args) stubbed; the import cell 0x00ac54f0 holds a stub (1 args); the import cell 0x00ac5500 holds a stub (1 args); callee 0x0055e0ac (3 args) stubbed; the word at `c0+0x14` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p0+8` points at 64 random bytes less `0` (placed); the word at `c0+0x214` points at 12 random bytes less `0` (placed); the word at `p4+0` points at 32 random bytes less `0` (placed); the word at `p4+4` points at 32 random bytes less `0` (placed); the word at `p4+8` points at 32 random bytes less `0` (placed); the word at `m+0xf8` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 32 random bytes less `0` (placed); the word at `p8+4` points at 32 random bytes less `0` (placed); the word at `p8+8` points at 32 random bytes less `0` (placed); the word at `m+0xfc` points at 8 random bytes less `0` (placed); the word at `p12+0` points at 32 random bytes less `0` (placed); the word at `p12+4` points at 32 random bytes less `0` (placed); the word at `m+0xdc` points at 32 random bytes less `0` (placed); the word at `p15+4` points at 32 random bytes less `0` (placed); the word at `p16+4` points at 32 random bytes less `0` (placed); the word at `p17+4` points at 32 random bytes less `0` (placed); the word at `p15+8` points at 64 random bytes less `0` (placed); the word at `p16+8` points at 64 random bytes less `0` (placed); the word at `p17+8` points at 64 random bytes less `0` (placed); the word at `p18+8` points at 64 random bytes less `0` (placed); the word at `m+0xf0` points at 16 random bytes less `0` (placed); the word at `m+0xf4` points at 32 random bytes less `0` (placed); stub 3's answer points at 64 random bytes less `0` (placed); slot +0x0 laid as 0x0072ff60 or its stub, a flag after the answers; slot +0x4 laid as 0x0072ff10 or its stub, a flag after the answers; stub 0 answered from [6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 1, 1, 3, 0] (90%); stub 4 answered from [1, 1, 0] (90%); stub 5 answered from [0, 0, 0, 1] (90%); stub 6 answered from [0, 0, 0, 1] (90%); stub 7 answered from [0, 0, 0, 1] (90%); stub 2 answers the address of 32 random bytes, a `p` window after the places; stub 1 answers a window of its own to each call, in call order (64 bytes, ...); stub 9 answers a window of its own to each call, in call order (32 bytes, ...); the word at `p18+4` laid as the address of `p15` every row; the word at `p1+0x18` laid as the address of `p24` every row; the word at `p2+0x18` laid as the address of `p24` every row; the word at `p3+0x18` laid as the address of `p24` every row; the word at `p25+0x18` laid as the address of `p24` every row; the word at `p32+0` laid as the address of `p23` every row; stub 8 answers `p9+j`, the data of the object its `ecx` is in `p5+j` (j < 3), linked at `+0x1c`; stub 14 answers `p13+j`, the data of the object its `ecx` is in `p9+j` (j < 2), linked at `+0x1c`; stub 0 answers 60 values in call order, the last from [0, 2], printed after the answers; stub 4 answers 12 values in call order, printed after the answers; stub 5 answers 12 values in call order, printed after the answers; stub 6 answers 12 values in call order, printed after the answers; stub 7 answers 12 values in call order, printed after the answers; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 is `__cdecl`: the caller pops its arguments; stub 16 is `__cdecl`: the caller pops its arguments; stub 17 is `__cdecl`: the caller pops its arguments; stub 15 answers a fresh bump-heap address of its argument 0 bytes (a `malloc`); stub 17 copies its argument 2 bytes from argument 1 to argument 0 (a `memcpy`); the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_check_build_order.py") else {
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
                // Windows (sweep_gen's flag order): `m` the unit; `c0` the objects record (`0xc0618c`: `+0x14` the
                // list of uid 0's objects, `+0x208` the found count, `+0x214` the found list), `c1` the constants
                // (`+0x24` `unit_build_respond_range`). `p0` the uid-0 list, `p1..p3` its objects; `p4` the found
                // list, `p5..p7` its elements; `p8` the unit-data list, `p9..p11` the unit data (`answer-by` of the
                // elements); `p12` the action list, `p13..p14` the actions of unit data 0 and 1 (the third has none);
                // `p15..p18` the order list's end node and three more, in `next` (`+4`) order, ring-closed by an
                // alias; `p19..p22` their orders; `p23` the vbase record (`+4` the action flag's offset), `p24` the
                // site type, `p25` the object `get_target`'s slot answers; `p26..p31` the six target orders a call
                // answers in turn (`get_order_target`, the last repeated); `p32` the object slot `+0xb8` answers;
                // `p33..p36` the records slot `+0xd8` answers in turn. The answers: `a[0..18]` the stubs' single
                // answers, `a[18..78]` stub 0's (`get_type`) sixty by call order, `a[78..90]` stub 4's
                // (`target_exists`), `a[90..102]` stub 5's (`+0x4c`), `a[102..114]` stub 6's (`+0xfc`),
                // `a[114..126]` stub 7's (`+0x100`); `a[126]` 1 when the image's `TargetOrder::get_target` stands in
                // slot 0, `a[127]` 1 when its `target_exists` stands in slot 4.
                use crate::check_build_view::{CheckBuildView, check_build_order_steps};
                fn r32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn w32(w: &mut [u8], o: usize, v: i32) {
                    w[o..o + 4].copy_from_slice(&v.to_le_bytes());
                }
                fn r16(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes([w[o], w[o + 1]]))
                }
                struct V<'a> {
                    m: &'a mut [u8],
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    a: &'a [i32],
                    /// The recorded calls: repath, kill, add_build_order, find_units.
                    t: Vec<Vec<String>>,
                    /// The cursor's node (0 the end node, 1.. the front on).
                    node: usize,
                    /// Calls made so far of stubs 0, 4, 5, 6, 7 (by `--seq`), of slot `+0xb4` and of slot `+0xd8`.
                    seq_at: [usize; 8],
                    /// The object the last `target_site` named (a `p` window).
                    obj: usize,
                    /// The unit data `found_other` last asked (a `p` window).
                    ud: usize,
                }
                impl V<'_> {
                    /// The next call-order answer of stub `s` (`--seq`), the last repeated.
                    fn seq(&mut self, s: usize) -> i32 {
                        let (at, k) = match s {
                            0 => (18, 0),
                            4 => (78, 1),
                            5 => (90, 2),
                            6 => (102, 3),
                            7 => (114, 4),
                            _ => unreachable!(),
                        };
                        let len = if s == 0 { 60 } else { 12 };
                        let n = self.seq_at[k].min(len - 1);
                        self.seq_at[k] += 1;
                        self.a[at + n]
                    }
                    /// The next target order (slot `+0xb4`): `p26 + call`, the last repeated.
                    fn target_order(&mut self) -> usize {
                        let n = self.seq_at[5].min(5);
                        self.seq_at[5] += 1;
                        26 + n
                    }
                    /// `get_target` of the order window `t`: inline (`TargetOrder::get_target`'s table walk) or the
                    /// stub's one object.
                    fn get_target(&mut self, t: usize) -> usize {
                        if self.a[126] != 0 {
                            let (idx, uid) = (r32(&self.p[t], 8), r32(&self.p[t], 0xc));
                            assert!(
                                uid == 0 && (0..3).contains(&idx),
                                "only uid 0's three objects are laid"
                            );
                            1 + idx as usize
                        } else {
                            25
                        }
                    }
                    fn rec(&mut self, col: usize, s: String) {
                        self.t[col].push(s);
                    }
                    fn node_addr(&self, k: usize) -> i32 {
                        match k {
                            0 => r32(self.m, 0xdc),
                            _ => r32(&self.p[14 + k], 4),
                        }
                    }
                    fn store_cursor(&mut self) {
                        let k = self.node;
                        let addr = self.node_addr(k);
                        w32(self.m, 0xd4, addr);
                        let ord = r32(&self.p[15 + k], 8);
                        w32(self.m, 0xcc, ord);
                        self.m[0xd0] = self.p[15 + k][0xc];
                    }
                }
                impl CheckBuildView for V<'_> {
                    fn orders_present(&mut self) -> bool {
                        r32(self.m, 0xd8) != 0
                    }
                    fn list_present(&mut self) -> bool {
                        r32(self.m, 0xdc) != 0
                    }
                    fn cursor_to_front(&mut self) {
                        self.node = 1;
                        self.store_cursor();
                    }
                    fn cursor_advance(&mut self) {
                        let next = r32(&self.p[15 + self.node], 4);
                        self.node = (0..4)
                            .find(|&k| self.node_addr(k) == next)
                            .expect("a node of the ring");
                        self.store_cursor();
                    }
                    fn has_order(&mut self) -> bool {
                        r32(self.m, 0xcc) != 0
                    }
                    fn order_type(&mut self) -> i32 {
                        self.seq(0)
                    }
                    fn order_is_action(&mut self) -> bool {
                        let off = r32(&self.p[23], 4) as usize;
                        self.p[32][off + 4] & 4 != 0
                    }
                    fn cursor_at_end(&mut self) -> bool {
                        r32(self.m, 0xd4) == r32(self.m, 0xdc)
                    }
                    fn target_exists(&mut self) -> bool {
                        let t = self.target_order();
                        if self.a[127] != 0 {
                            let (idx, uid) = (r32(&self.p[t], 8), r32(&self.p[t], 0xc));
                            if idx < 0 || uid < 0 {
                                return false;
                            }
                            assert!(uid == 0 && idx < 3, "only uid 0's three objects are laid");
                            let o = &self.p[1 + idx as usize];
                            o[8] & 1 != 0 && r16(o, 0x30) == r16(&self.p[t], 0x10)
                        } else {
                            self.seq(4) != 0
                        }
                    }
                    fn target_finished(&mut self) -> bool {
                        let t = self.target_order();
                        let _ = self.get_target(t);
                        self.seq(5) != 0
                    }
                    fn target_site(&mut self) -> i32 {
                        let t = self.target_order();
                        self.obj = self.get_target(t);
                        r16(&self.p[self.obj], 0xa)
                    }
                    fn target_blocks(&mut self) -> bool {
                        self.p[self.obj][8] & 0x20 != 0 || self.seq(6) != 0 || self.seq(7) != 0
                    }
                    fn repath(&mut self) {
                        self.rec(0, "m+0x0".into());
                    }
                    fn kill_current_order(&mut self) {
                        self.rec(1, "m+0x0,0".into());
                    }
                    fn add_build_order(&mut self, site: i32) {
                        let owner = i32::from(self.m[9]);
                        self.rec(2, format!("m+0x0,{site},{owner},0,1"));
                    }
                    fn owner(&mut self) -> i32 {
                        i32::from(self.m[9])
                    }
                    fn find_units(&mut self) {
                        let (x, y) = (r32(self.m, 0x10) ^ 0x63637, r32(self.m, 0x14) ^ 0x63637);
                        let range = r32(&self.c[1], 0x24).wrapping_mul(3).wrapping_shl(6);
                        let owner = i32::from(self.m[9]);
                        self.rec(3, format!("{x},{x},{y},1,{owner},{range},512,10,1,0,{x},1"));
                    }
                    fn found_count(&mut self) -> i32 {
                        r32(&self.c[0], 0x208)
                    }
                    fn found_other(&mut self, j: i32) -> bool {
                        self.ud = 9 + j as usize;
                        r16(&self.p[self.ud], 0xa) != r16(self.m, 0xa)
                    }
                    fn found_builds(&mut self, j: i32) -> bool {
                        // `answer-by`: unit data 0 and 1 have an action, the third none.
                        j < 2 && self.seq(0) == 6
                    }
                    fn found_target(&mut self, j: i32) -> (i32, i32) {
                        assert!(j < 2);
                        let n = self.seq_at[6].min(3);
                        self.seq_at[6] += 1;
                        let r = &self.p[33 + n];
                        (r32(r, 0xc), r32(r, 8))
                    }
                }
                let mut v = V {
                    m,
                    c,
                    p,
                    a,
                    t: vec![Vec::new(); 4],
                    node: 0,
                    seq_at: [0; 8],
                    obj: 0,
                    ud: 0,
                };
                check_build_order_steps(&mut v);
                for col in v.t {
                    log.push(if col.is_empty() {
                        "-".to_string()
                    } else {
                        col.join("/")
                    });
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "check_build_order — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "check_build_order — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "check_build_order — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "check_build_order — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("Unit::check_build_order@00603470: {n} rows agree");
}
