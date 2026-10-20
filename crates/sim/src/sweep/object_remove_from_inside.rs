//! `Object::remove_from_inside@006480f0` under the emulator against the port (item 3372; tools/sweep_gen.py, obj with windows).

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

/// `Object::remove_from_inside@006480f0`, run by `tools/emu/sweep_object_remove_from_inside.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 0 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p3 sparse table 1 (2 rows × 1 slice(s), row-major), p4..p7 sparse table 2 (2 rows × 2 slice(s), row-major), p8..p32 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c061f0 points at 2304 random bytes; vtable slot +0x8 (0 args) stubbed; vtable slot +0xd4 (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; callee 0x00651a80 (1 args) stubbed; callee 0x0064d8c0 (0 args) stubbed; callee 0x0046d310 (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; the word at `c0+0x14` points at 12 random bytes less `0` (placed); the word at `c0+0x30` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 12 random bytes less `0` (placed); the word at `p2+0` points at 12 random bytes less `0` (placed); the word at `p3+0` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 72 random bytes less `0` (placed); the word at `p8+4` points at 72 random bytes less `0` (placed); the word at `p8+8` points at 72 random bytes less `0` (placed); the word at `p9+0` points at 72 random bytes less `0` (placed); the word at `p9+4` points at 72 random bytes less `0` (placed); the word at `p9+8` points at 72 random bytes less `0` (placed); the word at `p10+0` points at 16 random bytes less `0` (placed); the word at `p10+4` points at 16 random bytes less `0` (placed); the word at `p10+8` points at 16 random bytes less `0` (placed); the word at `p11+0` points at 16 random bytes less `0` (placed); the word at `p11+4` points at 16 random bytes less `0` (placed); the word at `p11+8` points at 16 random bytes less `0` (placed); the word at `p12+0` points at 192 random bytes less `0` (placed); the word at `p12+4` points at 192 random bytes less `0` (placed); the word at `p12+8` points at 192 random bytes less `0` (placed); the word at `p13+0` points at 192 random bytes less `0` (placed); the word at `p13+4` points at 192 random bytes less `0` (placed); the word at `p13+8` points at 192 random bytes less `0` (placed); the word at `m+0x18` points at 72 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1, 1] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0, 1, 1] (90%); stub 4 answers -1..2 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 4 writes a value in 0..1 through its argument 0, printed after the answers (`o` in a place); 2 rows 0x1c apart at `@0xc0ab84`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 2 rows 0x1c apart at `@0xc0aec0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0xa28:0x10 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_remove_from_inside.py") else {
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
                // Window layout (sweep_gen's flag order): `m` the unit; `c[0]` `GameAccess::objects` (`0xc0618c`),
                // `c[1]` the constants. `p[0..4]` the rows of the objects table (`0xc0ab84`) and the units table
                // (`0xc0aec0`), `p[4+2r]` / `p[5+2r]` leader `r`'s flags word and counters (`+0xa28..`),
                // `p[8..14]` the arrays (objects of `c[0]`, hosts, units), `p[14+3l+i]` the objects' records,
                // `p[20+3l+i]` the hosts, `p[26+3l+i]` the units, `p[32]` the type record. The answers `a`: `a[0..8]`
                // the stubs' (slot 0x8, 0xd4, 0x20, 0xe8, get_inside, add_to_world, is_peasant, has_tribe_bonus),
                // `a[8]` get_inside's list, `a[9]` the 0xe8 image flag.
                use sim::remove_inside_view::{Garr, RemoveInsideView, remove_from_inside_steps};
                struct V<'a> {
                    m: &'a mut [u8],
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    a: &'a [i32],
                    log: &'a mut Vec<String>,
                }
                fn r32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn w32(w: &mut [u8], o: usize, v: i32) {
                    w[o..o + 4].copy_from_slice(&v.to_le_bytes());
                }
                fn r16(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes([w[o], w[o + 1]]))
                }
                fn w16(w: &mut [u8], o: usize, v: i32) {
                    w[o..o + 2].copy_from_slice(&(v as i16).to_le_bytes());
                }
                fn slot(base: usize, list: i32, i: i32) -> usize {
                    assert!(
                        (0..2).contains(&list) && (0..3).contains(&i),
                        "slot {list}/{i}"
                    );
                    base + 3 * list as usize + i as usize
                }
                impl V<'_> {
                    fn rec(&mut self, s: String) {
                        self.log.push(s);
                    }
                    fn own(&self) -> usize {
                        slot(26, i32::from(self.m[9]), r16(self.m, 0xa))
                    }
                    fn leader(&self) -> usize {
                        let r = usize::from(self.m[9]);
                        assert!(r < 2);
                        r
                    }
                }
                impl RemoveInsideView for V<'_> {
                    fn host_flag(&mut self) -> bool {
                        self.rec("s4:m+0x0,sp-0x10".into());
                        let (idx, list) = (self.a[4], self.a[8]);
                        if idx < 0 {
                            return false;
                        }
                        let h = slot(20, list, idx);
                        self.rec(format!("s2:p{h}+0x0"));
                        self.a[2] != 0
                    }
                    fn is_unit(&mut self) -> bool {
                        self.rec("s0:m+0x0".into());
                        self.a[0] != 0
                    }
                    fn own_up(&mut self) -> (i32, i32) {
                        let o = &self.p[self.own()];
                        (r16(o, 0x82), i32::from(o[0xb4] as i8))
                    }
                    fn own_down(&mut self) -> (i32, i32) {
                        (r16(self.m, 0x28), i32::from(self.m[0x3e] as i8))
                    }
                    fn set_object_down(&mut self, list: i32, idx: i32, down: (i32, i32)) {
                        let o = &mut self.p[slot(14, list, idx)];
                        w16(o, 0x28, down.0);
                        o[0x3e] = down.1 as u8;
                    }
                    fn set_unit_up(&mut self, list: i32, idx: i32, up: (i32, i32)) {
                        let o = &mut self.p[slot(26, list, idx)];
                        w16(o, 0x82, up.0);
                        o[0xb4] = up.1 as u8;
                    }
                    fn clear_own_links(&mut self) {
                        let i = self.own();
                        w16(&mut self.p[i], 0x82, -1);
                        w16(&mut self.p[i], 0x28, -1);
                    }
                    fn add_to_world(&mut self) {
                        self.rec("s5:m+0x0".into());
                    }
                    fn masked(&mut self) -> bool {
                        self.p[self.own()][0x68] & 1 != 0
                    }
                    fn is_captain(&mut self) -> bool {
                        let i = self.own();
                        if self.a[9] != 0 {
                            return (u16::from_le_bytes([self.p[i][0x8e], self.p[i][0x8f]]) >> 15)
                                != 0;
                        }
                        self.rec(format!("s3:p{i}+0x0"));
                        self.a[3] != 0
                    }
                    fn is_peasant(&mut self) -> bool {
                        self.rec("s6:m+0x0".into());
                        self.a[6] != 0
                    }
                    fn type_word(&mut self) -> i32 {
                        r32(&self.p[32], 0x40)
                    }
                    fn is_special(&mut self) -> bool {
                        self.rec("s1:m+0x0".into());
                        self.a[1] != 0
                    }
                    fn has_tribe_bonus(&mut self, id: i32) -> bool {
                        let l = 4 + 2 * self.leader();
                        self.rec(format!("s7:p{l}+0x0,{id}"));
                        self.a[7] != 0
                    }
                    fn bump(&mut self, which: Garr, delta: i32) {
                        let l = 5 + 2 * self.leader();
                        let off = match which {
                            Garr::Peasants => 0,
                            Garr::Stable => 4,
                            Garr::Scout => 8,
                            Garr::Barracks => 12,
                        };
                        let v = r32(&self.p[l], off).wrapping_add(delta);
                        w32(&mut self.p[l], off, v);
                    }
                    fn lakota_food(&mut self) -> i32 {
                        r32(&self.c[1], 0x848)
                    }
                    fn barracks_gather(&mut self) -> i32 {
                        r32(&self.c[1], 0x888)
                    }
                    fn set_leader_flag(&mut self) {
                        let l = 4 + 2 * self.leader();
                        let v = r32(&self.p[l], 0) | 0x200_0000;
                        w32(&mut self.p[l], 0, v);
                    }
                }
                let mut v = V {
                    m,
                    c,
                    p,
                    a,
                    log: &mut log,
                };
                remove_from_inside_steps(&mut v);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "remove_from_inside — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "remove_from_inside — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "remove_from_inside — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "remove_from_inside — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Object::remove_from_inside@006480f0: {n} rows agree");
}
