//! `Unit::find_build_spot@00603e20` under the emulator against the port (item 3372; tools/sweep_gen.py, obj with windows).

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

/// `Unit::find_build_spot@00603e20`, run by `tools/emu/sweep_unit_find_build_spot.py` under unicorn
/// (tools/emu): 384 random bytes of object, 0 argument(s), `p` windows in order: p0..p27 the place(s), in flag order, p28..p33 the stubs' answer window(s); the global cell 0x00c0618c points at 768 random bytes; the global cell 0x00c061f0 points at 256 random bytes; the global cell 0x00c061ec points at 256 random bytes; vtable slot +0xf8 (0 args) stubbed; vtable slot +0x104 (1 args) stubbed; vtable slot +0x134 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0xa8 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0xd8 (0 args) stubbed; callee 0x0065a120 (10 args) stubbed; callee 0x0065a620 (11 args) stubbed; callee 0x00608450 (0 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x0070fbe0 (5 args) stubbed; the import cell 0x00ac54f0 holds a stub (1 args); the import cell 0x00ac5500 holds a stub (1 args); callee 0x0055e0ac (3 args) stubbed; the word at `c0+0x214` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `p0+16` points at 32 random bytes less `0` (placed); the word at `p0+20` points at 32 random bytes less `0` (placed); the word at `m+0x100` points at 24 random bytes less `0` (placed); the word at `p7+0` points at 128 random bytes less `0` (placed); the word at `p7+4` points at 128 random bytes less `0` (placed); the word at `p7+8` points at 128 random bytes less `0` (placed); the word at `p7+12` points at 128 random bytes less `0` (placed); the word at `p7+16` points at 128 random bytes less `0` (placed); the word at `p7+20` points at 128 random bytes less `0` (placed); the word at `m+0x120` points at 24 random bytes less `0` (placed); the word at `p14+0` points at 32 random bytes less `0` (placed); the word at `p14+4` points at 32 random bytes less `0` (placed); the word at `p14+8` points at 32 random bytes less `0` (placed); the word at `p14+12` points at 32 random bytes less `0` (placed); the word at `p14+16` points at 32 random bytes less `0` (placed); the word at `p14+20` points at 32 random bytes less `0` (placed); the word at `m+0x18` points at 32 random bytes less `0` (placed); the word at `p15+0x10` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p16+0x10` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p17+0x10` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p18+0x10` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p19+0x10` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p20+0x10` points at 64 random bytes less `0` (placed, null a third of the rows); slot +0xf8 laid as 0x006109f0 or its stub, a flag after the answers; slot +0x104 laid as 0x0046ced0 or its stub, a flag after the answers; stub 5 answered from [6, 6, 6, 6, 6, 0, 1, 6] (90%); stub 0 answered from [0, 1, 2, 2] (90%); stub 1 answered from [0, 1, 1, 2] (90%); stub 2 answered from [0, 1, 1] (90%); stub 6 answers a window of its own to each call, in call order (32 bytes, ...); stub 3 answers `p8+j`, the data of the object its `ecx` is in `p1+j` (j < 6), linked at `+0x7c`; stub 4 answers `p15+j`, the data of the object its `ecx` is in `p1+j` (j < 6), linked at `+0x1c`; stub 5 answers 6 values in call order, printed after the answers; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 is `__cdecl`: the caller pops its arguments; stub 13 is `__cdecl`: the caller pops its arguments; stub 14 is `__cdecl`: the caller pops its arguments; stub 12 answers a fresh bump-heap address of its argument 0 bytes (a `malloc`); stub 14 copies its argument 2 bytes from argument 1 to argument 0 (a `memcpy`); stub 9 answers from its own `ecx`: the word at `+0x10`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_find_build_spot.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Window layout (sweep_gen's flag order): `m` the unit (`+9` owner, `+0xa` its index, `+0x10`/`+0x14` the position
            // XOR 0x63637, `+0x18` the type with the stubbed vtable, `+0xb1` the stance byte); `c[0]` the objects record
            // (`+0x208` the result count, `+0x214` the pointer array), `c[1]` the constants (`+0x24` the respond range). `p[1+i]`
            // the result array's objects (the stubbed vtable), `p[8+i]` object i's `BuildData` (`+0x60` the masks, `+9` the owner,
            // `+10` the site), `p[15+i]` its `UnitData` (`+0x10` the action pointer, null a third of the rows), `p[22+i]` the
            // action, `p[28+k]` the k-th `+0xd8` answer (`+8` site, `+0xc` owner). The answers `a`: `a[0..15]` the stubs'
            // single answers (0 `+0xf8`, 1 `+0x104`, 2 `+0x134`), `a[15..21]` the `+0x10` answers in call order, `a[21]` the
            // `+0xf8` slot flag (1 = `get_worker_stance`), `a[22]` the `+0x104` slot flag (1 = `has_stance_type`).
            use sim::build_spot_view::{BuildSpotView, build_spot_steps};
            const ENC: i32 = 0x63637;
            struct V<'a> {
                m: &'a [u8],
                c: &'a [Vec<u8>],
                p: &'a [Vec<u8>],
                a: &'a [i32],
                log: &'a mut Vec<String>,
                kinds: usize,
                targets: usize,
            }
            fn r32(w: &[u8], o: usize) -> i32 {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            }
            impl V<'_> {
                fn pos(&self) -> (i32, i32) {
                    (r32(self.m, 0x10) ^ ENC, r32(self.m, 0x14) ^ ENC)
                }
                fn put(&mut self, k: usize, s: String) {
                    self.log[k] = s;
                }
            }
            impl BuildSpotView for V<'_> {
                fn respond_range(&self) -> i32 {
                    r32(&self.c[1], 0x24)
                }
                fn worker_stance(&mut self) -> i32 {
                    if self.a[21] != 0 {
                        let has = if self.a[22] != 0 {
                            self.a[2]
                        } else {
                            self.a[1]
                        };
                        if has == 0 {
                            0
                        } else {
                            i32::from(self.m[0xb1] as i8)
                        }
                    } else {
                        self.a[0]
                    }
                }
                fn find_builds(&mut self, range: i32) {
                    let (x, y) = self.pos();
                    let o = self.m[9];
                    self.put(0, format!("{x},{x},{y},1,{o},{range},512,6,0,{x},0"));
                }
                fn find_units(&mut self, range: i32) {
                    let (x, y) = self.pos();
                    let o = self.m[9];
                    self.put(1, format!("{x},{x},{y},1,{o},{range},512,10,1,0,{x},1"));
                }
                fn len(&self) -> usize {
                    r32(&self.c[0], 0x208) as usize
                }
                fn site(&mut self, i: usize) -> Option<i32> {
                    let b = &self.p[8 + i];
                    if r32(b, 0x60) & 0x20 != 0 || b[9] != self.m[9] {
                        return None;
                    }
                    Some(i32::from(i16::from_le_bytes([b[10], b[11]])))
                }
                fn building_at(&mut self, i: usize) -> Option<i32> {
                    if r32(&self.p[15 + i], 0x10) == 0 {
                        return None;
                    }
                    let kind = self.a[15 + self.kinds.min(5)];
                    self.kinds += 1;
                    if kind != 6 {
                        return None;
                    }
                    let rec = &self.p[28 + self.targets.min(5)];
                    self.targets += 1;
                    if r32(rec, 0xc) as u32 != u32::from(self.m[9]) {
                        return None;
                    }
                    Some(r32(rec, 8))
                }
                fn swarm(&mut self, site: i32) {
                    let o = self.m[9];
                    let idx = i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]));
                    self.put(2, format!("sp-0xa30,{idx},{o},0,0"));
                    self.put(3, format!("sp-0xa30,{site},{o},1,6,0"));
                }
            }
            log.extend(["-", "-", "-", "-"].map(String::from));
            let mut v = V {
                m: &*m,
                c: &*c,
                p: &*p,
                a,
                log: &mut log,
                kinds: 0,
                targets: 0,
            };
            i64::from(build_spot_steps(&mut v))
        };
        assert_eq!(got, want_eax, "find_build_spot — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_build_spot — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_build_spot — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_build_spot — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "find_build_spot — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::find_build_spot@00603e20: {n} rows agree");
}
