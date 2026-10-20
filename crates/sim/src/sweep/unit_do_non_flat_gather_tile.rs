//! `Unit::do_non_flat_gather@005f0170` under the emulator against the port (item 3381; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_non_flat_gather@005f0170`, run by `tools/emu/sweep_unit_do_non_flat_gather_tile.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p2 sparse table 0 (2 rows × 1 slice(s), row-major), p3..p15 the place(s), in flag order; the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 128 random bytes; the global cell 0x00c061f0 points at 2560 random bytes; 8 random bytes at 0x00ee1088 (each byte in 0..1); argument 0 names 52 random bytes; vtable slot +0xac (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x00616f40 (3 args) stubbed; callee 0x006b4e50 (5 args) stubbed; callee 0x00846cd0 (1 args) stubbed; callee 0x0046d4f0 (2 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x00a1edd0 (1 args) stubbed; callee 0x00a1d660 (1 args) stubbed; callee 0x00a2e550 (6 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; the word at `c0+0x14` points at 12 random bytes less `0` (placed); the word at `p3+0` points at 64 random bytes less `0` (placed); the word at `p3+4` points at 64 random bytes less `0` (placed); the word at `p3+8` points at 64 random bytes less `0` (placed); the word at `c0+0x30` points at 12 random bytes less `0` (placed); the word at `p7+0` points at 64 random bytes less `0` (placed); the word at `p7+4` points at 64 random bytes less `0` (placed); the word at `p7+8` points at 64 random bytes less `0` (placed); stub 0's answer points at 256 random bytes less `0` (placed); the word at `c1+0x138` points at 256 random bytes less `-40` (placed); the word at `m+0x18` points at 64 random bytes less `0` (placed); the word at `p11+0x18` points at 64 random bytes less `0` (placed); the word at `p11+0xa8` points at 256 random bytes less `0` (placed); stub 4 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); the word at `p11+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 4 answers 30 values in call order, printed after the answers; stub 7 answers 0..65535 every row; stub 10 answers 0..0 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 4 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 4 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_non_flat_gather_tile.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Row: `a[0]` the order pointer (unread), `a[1 + i]` stub `i`'s answer (stubs 0 to 11),
            // then `a[13..43]` stub 4's thirty `has_gather_access` answers in call order.
            // Stubs: 0 slot `+0xac` (the building's record), 1 slot `+0x60`, 2 slot `+0xb8` (the
            // `is(0x1a2, 0)` of a `Build` / any other site), 3 `set_anim`, 4 `has_gather_access`,
            // 5 `remove_hold_doobers`, 6 `Array<TCoordData>::remove`, 7 `Random::get`, 8..9 the
            // two `String`s, 10 `Error::report`, 11 `String::close`. Windows: `m` the unit, `c0`
            // the objects cell, `c1` the world, `c2` `div_3_table`, `c3` `game`'s constants,
            // `c4` the `ignore_always` bytes; `p0` the order, `p1..p2` the leaders' first words,
            // `p3 + 4h ..` the objects of player `h` (`p4..6`, `p8..10`), `p11` the site's
            // record, `p12` the tile masks, `p13` the unit's type, `p14` the site's inner record,
            // `p15` its tile list's data.
            use crate::gather_tile::{
                GatherTileView, TileOrder, gather_head_steps, gather_tile_steps,
            };
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                n4: usize,
                deref: Vec<[String; 2]>,
            }
            fn hex(v: i32) -> String {
                v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
            }
            impl V<'_> {
                fn who(&self) -> i32 {
                    i32::from(self.m[9])
                }
                fn obj(&self) -> usize {
                    let owner = rd(&self.p[0], 0xc) as usize;
                    let at = rd(&self.p[0], 8) as usize;
                    [[4, 5, 6], [8, 9, 10]][owner][at]
                }
                fn short(w: &[u8], off: usize) -> i32 {
                    i32::from(i16::from_le_bytes([w[off], w[off + 1]]))
                }
                fn push(&mut self, s: String) {
                    self.log.push(s);
                }
                fn site_call(&mut self) {
                    let s = format!("s0:p{}+0x0", self.obj());
                    self.push(s);
                }
                fn gather_call(&mut self, args: &str, x: i32, y: i32) -> bool {
                    let s = format!("s4:c1+0x0,{args},{},1,0", self.who());
                    self.push(s);
                    self.deref.push([hex(x), hex(y)]);
                    let r = self.a[13 + self.n4.min(29)];
                    self.n4 += 1;
                    r != 0
                }
            }
            impl GatherTileView for V<'_> {
                fn site_recharge_flag(&mut self) -> bool {
                    self.site_call();
                    Self::short(&self.p[11], 0x60) & 0x800 != 0
                }
                fn site_recharge_bump(&mut self) {
                    self.site_call();
                    let n = Self::short(&self.p[11], 0x7a).wrapping_add(1) as u16;
                    self.p[11][0x7a..0x7c].copy_from_slice(&n.to_le_bytes());
                }
                fn site_recharge_flag_set(&mut self) {
                    self.site_call();
                    let f = (Self::short(&self.p[11], 0x60) | 0x800) as u16;
                    self.p[11][0x60..0x62].copy_from_slice(&f.to_le_bytes());
                }
                fn clear_group(&mut self) {
                    self.m[0x80..0x82].copy_from_slice(&[0xff, 0xff]);
                }
                fn mark_leader(&mut self) {
                    let w = 1 + self.who() as usize;
                    let v = rd(&self.p[w], 0) | 0x200_0000;
                    wr(&mut self.p[w], 0, v);
                }
                fn stand(&mut self) {
                    self.push("s3:m+0x0,0,0,1".into());
                }
                fn order_tile_gatherable(&mut self, tx: i32, ty: i32) -> bool {
                    self.gather_call("p0+0x14,p0+0x18", tx, ty)
                }
                fn list_tile_gatherable(&mut self, x: i32, y: i32) -> bool {
                    self.gather_call("sp-0x48,sp-0x44", x, y)
                }
                fn site_enter(&mut self) {
                    self.site_call();
                }
                fn unit_type_index(&mut self) -> i32 {
                    rd(&self.p[13], 4)
                }
                fn doober(&mut self) -> i32 {
                    Self::short(self.m, 0x86)
                }
                fn remove_doober(&mut self, id: i32) {
                    self.push(format!("s5:15400352,{id}"));
                    self.m[0x86..0x88].copy_from_slice(&[0xff, 0xff]);
                }
                fn tiles_len(&mut self) -> i32 {
                    rd(&self.p[11], 0x9c)
                }
                fn site_mtn(&mut self) -> i32 {
                    i32::from(self.p[11][0xb4] as i8)
                }
                fn mtn_tiny(&mut self) -> i32 {
                    rd(&self.c[3], 0x9cc)
                }
                fn tile_at(&mut self, i: i32) -> (i32, i32) {
                    let at = i as usize * 8;
                    (rd(&self.p[15], at), rd(&self.p[15], at + 4))
                }
                fn tile_mask(&mut self, x: i32, y: i32) -> u16 {
                    let idx = rd(&self.c[1], 0x18).wrapping_mul(y).wrapping_add(x);
                    // The mask pointer is the window's byte 40.
                    let at = (40 + 2 * idx as isize) as usize;
                    u16::from_le_bytes([self.p[12][at], self.p[12][at + 1]])
                }
                fn building_tile(&mut self) -> (i32, i32) {
                    let o = &self.p[self.obj()];
                    let t = |w: i32| rd(&self.c[2], (((w ^ 0x63637) >> 6) as usize) * 4);
                    (t(rd(o, 0x10)), t(rd(o, 0x14)))
                }
                fn remove_tile(&mut self, x: i32, y: i32) {
                    self.push(format!("s6:p11+0x98,{x},{y}"));
                }
                fn push_tile(&mut self, x: i32, y: i32) {
                    let n = rd(&self.p[11], 0x9c);
                    let at = n as usize * 8;
                    wr(&mut self.p[15], at, x);
                    wr(&mut self.p[15], at + 4, y);
                    wr(&mut self.p[11], 0x9c, n + 1);
                }
                fn rnd(&mut self) -> i32 {
                    self.a[1 + 7]
                }
                fn site_is_woodcutter(&mut self) -> bool {
                    self.site_call();
                    if rd(&self.p[11], 0) == 0xb4_2174 {
                        self.push("s1:p14+0x0,418,0".into());
                        self.a[2] != 0
                    } else {
                        self.push("s2:p11+0x0,418,0".into());
                        self.a[3] != 0
                    }
                }
                fn or_unit_flags(&mut self, bits: u32) {
                    let v = rd(self.m, 0x68) | bits as i32;
                    wr(self.m, 0x68, v);
                }
            }
            let before = TileOrder {
                tx: rd(&p[0], 0x14),
                ty: rd(&p[0], 0x18),
                wait: rd(&p[0], 0x20),
                goto_build: p[0][0x24] != 0,
                dist_mod: i32::from(p[0][0x26]),
                been_there: p[0][0x27] != 0,
            };
            let mut o = before;
            let mut v = V {
                a,
                m,
                c,
                p,
                log: &mut log,
                n4: 0,
                deref: Vec::new(),
            };
            gather_head_steps(&mut v, o.been_there);
            gather_tile_steps(&mut v, &mut o);
            let deref = std::mem::take(&mut v.deref);
            let (c0, c1): (Vec<&str>, Vec<&str>) =
                deref.iter().map(|d| (d[0].as_str(), d[1].as_str())).unzip();
            let col = |c: Vec<&str>| {
                if c.is_empty() {
                    "-".to_string()
                } else {
                    c.join("/")
                }
            };
            let (s0, s1) = (col(c0), col(c1));
            log.push(s0);
            log.push(s1);
            wr(&mut p[0], 0x14, o.tx);
            wr(&mut p[0], 0x18, o.ty);
            wr(&mut p[0], 0x20, o.wait);
            if o.goto_build != before.goto_build {
                p[0][0x24] = u8::from(o.goto_build);
            }
            if o.been_there != before.been_there {
                p[0][0x27] = u8::from(o.been_there);
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "do_non_flat_gather — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_non_flat_gather — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_non_flat_gather — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "do_non_flat_gather — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2500, "only {n} rows");
    eprintln!("Unit::do_non_flat_gather@005f0170: {n} rows agree");
}
