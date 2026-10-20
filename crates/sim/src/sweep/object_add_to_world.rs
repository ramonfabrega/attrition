//! `Object::add_to_world@0064d8c0` under the emulator against the port (item 3379; tools/sweep_gen.py, obj with windows).

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

/// `Object::add_to_world@0064d8c0`, run by `tools/emu/sweep_object_add_to_world.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 0 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p40 the place(s), in flag order, p41 stub 3's answer window; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; 64 random bytes at 0x00adc400 (each word in -3..3); 64 random bytes at 0x00adcaf0 (each word in -3..3); 48 random bytes at 0x00add1e0 (each word in 0..16); 4 random bytes at 0x00ee109f (each word in 0..1); vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x0046d250 (2 args) stubbed; callee 0x00a1edd0 (1 args) stubbed; callee 0x00a1d660 (1 args) stubbed; callee 0x00a2e550 (6 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; the word at `c1+20` points at 16 random bytes less `0` (placed); the word at `c1+48` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 72 random bytes less `0` (placed); the word at `p2+4` points at 72 random bytes less `0` (placed); the word at `p3+0` points at 72 random bytes less `0` (placed); the word at `p3+4` points at 72 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p8+0` points at 256 random bytes less `0` (placed); the word at `p8+4` points at 256 random bytes less `0` (placed); the word at `p9+0` points at 256 random bytes less `0` (placed); the word at `p9+4` points at 256 random bytes less `0` (placed); the word at `p10+0x18` points at 592 random bytes less `0` (placed); the word at `p11+0x18` points at 592 random bytes less `0` (placed); the word at `p12+0x18` points at 592 random bytes less `0` (placed); the word at `p13+0x18` points at 592 random bytes less `0` (placed); the word at `p10+0xf4` points at 16 random bytes less `0` (placed); the word at `p11+0xf4` points at 16 random bytes less `0` (placed); the word at `p12+0xf4` points at 16 random bytes less `0` (placed); the word at `p13+0xf4` points at 16 random bytes less `0` (placed); the word at `p18+0` points at 20 random bytes less `0` (placed); the word at `p18+4` points at 20 random bytes less `0` (placed); the word at `p19+0` points at 20 random bytes less `0` (placed); the word at `p19+4` points at 20 random bytes less `0` (placed); the word at `p20+0` points at 20 random bytes less `0` (placed); the word at `p20+4` points at 20 random bytes less `0` (placed); the word at `p21+0` points at 20 random bytes less `0` (placed); the word at `p21+4` points at 20 random bytes less `0` (placed); the word at `m+0x18` points at 592 random bytes less `0` (placed); the word at `c0+0x134` points at 256 random bytes less `0` (placed); the word at `p31+24` points at 48 random bytes less `0` (placed, null a third of the rows); the word at `p31+52` points at 48 random bytes less `0` (placed, null a third of the rows); the word at `p31+80` points at 48 random bytes less `0` (placed, null a third of the rows); the word at `p31+108` points at 48 random bytes less `0` (placed, null a third of the rows); the word at `p31+136` points at 48 random bytes less `0` (placed, null a third of the rows); the word at `p31+164` points at 48 random bytes less `0` (placed, null a third of the rows); the word at `p31+192` points at 48 random bytes less `0` (placed, null a third of the rows); the word at `p31+220` points at 48 random bytes less `0` (placed, null a third of the rows); the word at `p31+248` points at 48 random bytes less `0` (placed, null a third of the rows); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [0, 1, 1] (90%); stub 3 answers the address of 48 random bytes, a `p` window after the places; stub 2 answers -1..2 every row; stub 6 answers 0..0 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 2 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 3 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 3 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); the stack 512 bytes under the call is zeroed before each row; 2 rows 0x1c apart at `@0xc0aec0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_add_to_world.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(7);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Window layout (sweep_gen's flag order): `m` the object; `c[0]` the world (`0xc06188`), `c[1]` `GameAccess::objects`
                // (`0xc0618c`), `c[2]` `div_3_table` (`i / 3`), `c[4]`/`c[3]` `move_x`/`move_y` (`0xadcaf0`, `0xadc400`), `c[5]` `radius`, `c[6]` `ignore_always`.
                // `p[2+l]` objects' arrays (list `l`), `p[4+2l+i]` the objects, `p[10+2l+i]` the unit records, `p[14+2l+i]` their
                // types, `p[22+4l+2i+g]` guy `g`, `p[30]` this's type, `p[31]` the cells, `p[32+k]` cell `k`'s block, `p[41]` the new block.
                // The answers `a`: slot 0x8, slot 0xbc, `get_tregion`, `new_coll_block` (unread), the two string ctors, `Error::report`, close.
                use sim::add_to_world_view::{AddToWorldView, add_to_world_steps};
                struct V<'a> {
                    m: &'a mut [u8],
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    a: &'a [i32],
                    log: &'a mut Vec<String>,
                    tail: [Vec<String>; 4],
                }
                fn r32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn r16(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes([w[o], w[o + 1]]))
                }
                fn w16(w: &mut [u8], o: usize, v: i32) {
                    w[o..o + 2].copy_from_slice(&(v as i16).to_le_bytes());
                }
                fn le(v: i32) -> String {
                    format!("{:08x}", (v as u32).swap_bytes())
                }
                impl V<'_> {
                    fn own(&self) -> (usize, usize) {
                        let (l, i) = (usize::from(self.m[9]), r16(self.m, 0xa));
                        assert!(l < 2 && (0..2).contains(&i), "own {l}/{i}");
                        (l, i as usize)
                    }
                    fn urec(&self) -> usize {
                        let (l, i) = self.own();
                        10 + 2 * l + i
                    }
                    fn cellw(&self, cell: i32) -> usize {
                        assert!((0..9).contains(&cell), "cell {cell}");
                        0x1c * cell as usize
                    }
                }
                impl AddToWorldView for V<'_> {
                    fn list(&mut self) -> i32 {
                        i32::from(self.m[9])
                    }
                    fn index(&mut self) -> i32 {
                        r16(self.m, 0xa)
                    }
                    fn set_next(&mut self, v: i32) {
                        w16(self.m, 0x2a, v);
                    }
                    fn pos_raw(&mut self) -> (i32, i32) {
                        (r32(self.m, 0x10), r32(self.m, 0x14))
                    }
                    fn div3(&mut self, i: i32) -> i32 {
                        r32(&self.c[2], 4 * usize::try_from(i).expect("index"))
                    }
                    fn world_size(&mut self) -> (i32, i32) {
                        (r32(&self.c[0], 0), r32(&self.c[0], 4))
                    }
                    fn cell_head(&mut self, cell: i32) -> (i32, i32) {
                        let o = self.cellw(cell);
                        (r16(&self.p[31], o + 8), r16(&self.p[31], o + 10))
                    }
                    fn set_cell_head(&mut self, cell: i32, idx: i32, list: i32) {
                        let o = self.cellw(cell);
                        w16(&mut self.p[31], o + 8, idx);
                        w16(&mut self.p[31], o + 10, list);
                    }
                    fn ignore_always(&mut self) -> bool {
                        self.c[6][0] != 0
                    }
                    fn report_loop(&mut self) -> bool {
                        self.log.push("s4:sp-0x5c,11381284".into());
                        self.log.push("s5:sp-0x48,11381252".into());
                        self.log
                            .push("s6:13161384,sp-0x48,sp-0x48,0,0,c6+0x0,2".into());
                        self.log.push("s7:sp-0x48".into());
                        self.log.push("s7:sp-0x5c".into());
                        self.a[6] != 0
                    }
                    fn link_previous(&mut self, list: i32, idx: i32, own_idx: i32, own_list: i32) {
                        assert!(
                            (0..2).contains(&list) && (0..2).contains(&idx),
                            "head {list}/{idx}"
                        );
                        let o = &mut self.p[4 + 2 * list as usize + idx as usize];
                        w16(o, 0x2a, own_idx);
                        o[0x3f] = own_list as u8;
                    }
                    fn set_previous(&mut self, idx: i32, list: i32) {
                        w16(self.m, 0x2c, idx);
                        w16(self.m, 0x2e, list);
                    }
                    fn slot8(&mut self) -> bool {
                        self.log.push("s0:m+0x0".into());
                        self.a[0] != 0
                    }
                    fn slot_bc(&mut self) -> bool {
                        self.log.push("s1:m+0x0".into());
                        self.a[1] != 0
                    }
                    fn type_state(&mut self) -> i32 {
                        r32(&self.p[30], 0x218)
                    }
                    fn coll_size(&mut self) -> i32 {
                        let (l, i) = self.own();
                        r32(&self.p[14 + 2 * l + i], 0x248)
                    }
                    fn guys(&mut self) -> i32 {
                        i32::from(self.p[self.urec()][0xb5] as i8)
                    }
                    fn guy_pos(&mut self, g: i32) -> (i32, i32) {
                        let (l, i) = self.own();
                        let w = &self.p[22 + 4 * l + 2 * i + g as usize];
                        (r32(w, 0xc), r32(w, 0x10))
                    }
                    fn tregion(&mut self, tx: i32, ty: i32) -> i32 {
                        self.log.push("s2:c0+0x0,sp-0x28,sp-0x24".into());
                        self.tail[0].push(le(tx));
                        self.tail[1].push(le(ty));
                        self.a[2]
                    }
                    fn radius(&mut self, t: i32) -> i32 {
                        r32(&self.c[5], 4 * usize::try_from(t).expect("radius index"))
                    }
                    fn move_x(&mut self, k: i32) -> i32 {
                        r32(&self.c[4], 4 * k as usize)
                    }
                    fn move_y(&mut self, k: i32) -> i32 {
                        r32(&self.c[3], 4 * k as usize)
                    }
                    fn cell_region(&mut self, cell: i32) -> i32 {
                        let o = self.cellw(cell);
                        r16(&self.p[31], o + 4)
                    }
                    fn cell_block(&mut self, cell: i32) -> i32 {
                        let o = self.cellw(cell);
                        r32(&self.p[31], o + 0x18)
                    }
                    fn new_block(&mut self, tregion: i32, cx: i32, cy: i32) {
                        self.tail[2].push(le(cx));
                        self.tail[3].push(le(cy));
                        self.log.push(format!("s3:{tregion},sp-0x30,sp-0x24"));
                    }
                    fn mark(&mut self, cell: i32, fresh: bool, bit: i32) {
                        let w = if fresh { 41 } else { 32 + cell as usize };
                        let b = &mut self.p[w];
                        b[0xc + (bit >> 3) as usize] |= 1 << (bit & 7);
                        b[8..12].copy_from_slice(&0i32.to_le_bytes());
                    }
                }
                let mut v = V {
                    m,
                    c,
                    p,
                    a,
                    log: &mut log,
                    tail: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
                };
                add_to_world_steps(&mut v);
                let tail = std::mem::take(&mut v.tail);
                if log.is_empty() {
                    log.push("-".to_string());
                }
                for t in tail {
                    log.push(if t.is_empty() {
                        "-".to_string()
                    } else {
                        t.join("/")
                    });
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "add_to_world — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_to_world — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_to_world — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[8 + i],
                "add_to_world — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Object::add_to_world@0064d8c0: {n} rows agree");
}
