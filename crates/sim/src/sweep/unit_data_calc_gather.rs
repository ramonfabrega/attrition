//! `UnitData::calc_gather@00609180` under the emulator against the port (item 3330; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::calc_gather@00609180`, run by `tools/emu/sweep_unit_data_calc_gather.py` under unicorn
/// (tools/emu): 176 random bytes of object, 10 argument(s), `p` windows in order: p0..p5 pointed argument(s), p6..p13 sparse table 0 (4 rows × 2 slice(s), row-major), p14..p38 the place(s), in flag order; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; argument 0 names 8 random bytes (null a third of the rows); argument 1 names 8 random bytes (null a third of the rows); argument 2 names 8 random bytes (null a third of the rows); argument 3 names 32 random bytes; argument 4 names 64 random bytes (null a third of the rows); argument 5 names 256 random bytes (null a third of the rows); vtable slot +0x60 (2 args) stubbed; vtable slot +0x124 (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x0065bec0 (5 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x006e08d0 (3 args) stubbed; callee 0x0060a4b0 (0 args) stubbed; the word at `0xc0ab84+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p14+0` points at 64 random bytes less `0` (placed); the word at `p14+4` points at 64 random bytes less `0` (placed); the word at `p14+8` points at 64 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p18+0` points at 64 random bytes less `0` (placed); the word at `p18+4` points at 64 random bytes less `0` (placed); the word at `p18+8` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p22+0` points at 112 random bytes less `0` (placed); the word at `p22+4` points at 112 random bytes less `0` (placed); the word at `p22+8` points at 112 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p26+0` points at 112 random bytes less `0` (placed); the word at `p26+4` points at 112 random bytes less `0` (placed); the word at `p26+8` points at 112 random bytes less `0` (placed); the word at `p23+0x18` points at 64 random bytes less `0` (placed); the word at `p24+0x18` points at 64 random bytes less `0` (placed); the word at `p25+0x18` points at 64 random bytes less `0` (placed); the word at `p27+0x18` points at 64 random bytes less `0` (placed); the word at `p28+0x18` points at 64 random bytes less `0` (placed); the word at `p29+0x18` points at 64 random bytes less `0` (placed); the word at `m+0x18` points at 64 random bytes less `0` (placed); the word at `c0+0x134` points at 1024 random bytes less `0` (placed); the word at `c0+0x138` points at 1280 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1, 0] (90%); stub 2 answered from [1, 1, 1, 0] (90%); stub 3 answered from [1, 1, 1, 0] (90%); stub 4 answered from [1, 1, 0] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0] (90%); stub 8 answered from [0, 1] (90%); stub 0 answers 40 values in call order, printed after the answers; stub 1 answers 40 values in call order, printed after the answers; stub 2 answers 40 values in call order, printed after the answers; stub 3 answers 40 values in call order, printed after the answers; stub 4 answers 40 values in call order, printed after the answers; stub 5 answers 80 values in call order, printed after the answers; stub 6 answers 10 values in call order, printed after the answers; stub 8 answers 10 values in call order, printed after the answers; stub 1 answers 0..4 every row; stub 5 answers -20..40 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `0x006817f0` run once before the rows (a table builder the startup would have run); stub 7 writes 1000 (4 bytes) at `p3+0x0` each call; stub 7 writes -77 (4 bytes) at `p3+0x4` each call; stub 7 writes 12345 (4 bytes) at `p3+0x8` each call; stub 7 writes 3 (4 bytes) at `p3+0xc` each call; stub 7 writes 999999 (4 bytes) at `p3+0x10` each call; stub 7 writes 0 (4 bytes) at `p3+0x14` each call; 4 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x4, +0x74:0x10 (`p` windows after the pointed buffers, before the places); figures p15..p17 link at +0x2c (2 bytes) below their own index or to -1; figures p19..p21 link at +0x2c (2 bytes) below their own index or to -1; figures p15..p17 hold a short at +0x2e drawn from [0, 1], after the links; figures p19..p21 hold a short at +0x2e drawn from [0, 1], after the links; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_calc_gather.py") else {
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
                // **The port is the crate's own steps**, `crate::calc_gather::unit_calc_gather_steps`, which
                // `Sim::calc_gather_at` and `Sim::do_gather_rates` run over their `SimGather`; here the view is the
                // emulated original's windows.
                //
                // `a`: the ten stack arguments (`param_1` `rare`, `param_2` the flag, `param_3` `good_obj`, `param_4`
                // the six rates, `param_5` the mask, `param_6` the tally — a pointer argument is 0 when the row laid
                // it null — then `param_7`, `param_8` and the position `param_9`, `param_10`), the nine stubs' single
                // answers (`a[10 + k]`; only stub 7's is read, and it is ignored), then the call-order answers: stub 0
                // (a type's `is`) from `a[19]`, 1 (its `get_upgrade_level`) `a[59]`, 2 (an object's `+0x8`) `a[99]`,
                // 3 (`+0xbc`) `a[139]`, 4 (a unit's `+0xb8`) `a[179]`, 5 (`find_good_at`) `a[219]`, 6 (`is_ally`)
                // `a[299]`, 8 (`is_unpacking`) `a[309]`, and the vtable flag `a[319]` (slot `+0xb8` laid as
                // `ObjectData::is`: the unit's `is` is its type's slot `+0x60`, stub 0, not stub 4).
                // Windows: `c[0]` the world, `c[1]` `div_3_table`; `p0..p5` the pointed arguments, `p6 + 2r` and
                // `p7 + 2r` leader `r`'s `+0x8` and `+0x74..`, `p15 + 4L + i` the objects of list `L`, `p23 + 4L + i`
                // the units, `p30 + 3L + i` their types, `p36` this's type, `p37` the cells, `p38` the tile masks.
                use crate::calc_gather::{GatherArgs, GatherView, Who, unit_calc_gather_steps};
                type Obj = (i32, i32);
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 9],
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn wr32(w: &mut [u8], o: usize, v: i32) {
                    w[o..o + 4].copy_from_slice(&v.to_le_bytes());
                }
                fn rd16(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes(
                        w[o..o + 2].try_into().expect("two bytes"),
                    ))
                }
                impl V<'_> {
                    fn seq(&mut self, k: usize, base: usize, n: usize) -> i32 {
                        let i = self.turn[k].min(n - 1);
                        self.turn[k] += 1;
                        self.a[base + i]
                    }
                    fn obj(&self, h: Obj) -> usize {
                        15 + 4 * h.0 as usize + h.1 as usize
                    }
                    fn unit(&self, h: Obj) -> usize {
                        23 + 4 * h.0 as usize + h.1 as usize
                    }
                    fn ty(&self, w: Who<Obj>) -> usize {
                        match w {
                            Who::Me => 36,
                            Who::Other(h) => 30 + 3 * h.0 as usize + h.1 as usize,
                        }
                    }
                    fn rec(&self, w: Who<Obj>) -> &[u8] {
                        match w {
                            Who::Me => &*self.m,
                            Who::Other(h) => &self.p[self.unit(h)],
                        }
                    }
                    fn cell_slot(&self, cx: i32, cy: i32) -> usize {
                        (rd32(&self.c[0], 0) * cy + cx) as usize * 0x1c
                    }
                    fn link(&self, o: usize) -> Option<Obj> {
                        let (idx, list) = (rd16(&self.p[o], 0x2c), rd16(&self.p[o], 0x2e));
                        (idx >= 0).then_some((list, idx))
                    }
                }
                impl GatherView for V<'_> {
                    type H = Obj;
                    fn type_index(&self, w: Who<Obj>) -> i32 {
                        rd32(&self.p[self.ty(w)], 4)
                    }
                    fn type_is_fisher(&mut self, w: Who<Obj>) -> bool {
                        self.log.push(format!("s0:p{}+0x0,317,0", self.ty(w)));
                        self.seq(0, 19, 40) != 0
                    }
                    fn type_radius_base(&mut self, w: Who<Obj>) -> i32 {
                        self.log.push(format!("s1:p{}+0x0", self.ty(w)));
                        self.seq(1, 59, 40)
                    }
                    fn packed(&self, w: Who<Obj>) -> bool {
                        rd32(self.rec(w), 0x68) & 0x80000 != 0
                    }
                    fn pos(&self, w: Who<Obj>) -> (i32, i32) {
                        // The place is read off the **objects** list's element, the mask and the type off the units'.
                        let r = match w {
                            Who::Me => &*self.m,
                            Who::Other(h) => &self.p[self.obj(h)],
                        };
                        (rd32(r, 0x10) ^ 0x63637, rd32(r, 0x14) ^ 0x63637)
                    }
                    fn me_owner(&self) -> i32 {
                        i32::from(self.m[9])
                    }
                    fn good_obj(&self) -> i32 {
                        rd16(self.m, 0x94)
                    }
                    fn div3(&self, i: i32) -> i32 {
                        rd32(&self.c[1], 4 * i as usize)
                    }
                    fn tile_dims(&self) -> (i32, i32) {
                        (rd32(&self.c[0], 0x18), rd32(&self.c[0], 0x1c))
                    }
                    fn cell_dims(&self) -> (i32, i32) {
                        (rd32(&self.c[0], 0), rd32(&self.c[0], 4))
                    }
                    fn tile_mask(&self, x: i32, y: i32) -> u16 {
                        let o = 2 * (rd32(&self.c[0], 0x18) * y + x) as usize;
                        u16::from_le_bytes(self.p[38][o..o + 2].try_into().expect("two bytes"))
                    }
                    fn cell_owner(&self, cx: i32, cy: i32) -> i32 {
                        i32::from(self.p[37][self.cell_slot(cx, cy) + 0xf] as i8)
                    }
                    fn cell_head(&self, cx: i32, cy: i32) -> Option<Obj> {
                        let o = self.cell_slot(cx, cy);
                        let (idx, list) = (rd16(&self.p[37], o + 8), rd16(&self.p[37], o + 0xa));
                        (idx >= 0).then_some((list, idx))
                    }
                    fn next(&self, h: Obj) -> Option<Obj> {
                        self.link(self.obj(h))
                    }
                    fn list_of(&self, h: Obj) -> i32 {
                        h.0
                    }
                    fn active(&mut self, h: Obj) -> bool {
                        self.log.push(format!("s2:p{}+0x0", self.obj(h)));
                        self.seq(2, 99, 40) != 0
                    }
                    fn on_map(&mut self, h: Obj) -> bool {
                        self.log.push(format!("s3:p{}+0x0", self.obj(h)));
                        self.seq(3, 139, 40) != 0
                    }
                    fn is_line(&mut self, h: Obj, ty: i32) -> bool {
                        if self.a[319] != 0 {
                            self.log
                                .push(format!("s0:p{}+0x0,{ty},0", self.ty(Who::Other(h))));
                            self.seq(0, 19, 40) != 0
                        } else {
                            self.log.push(format!("s4:p{}+0x0,{ty},0", self.unit(h)));
                            self.seq(4, 179, 40) != 0
                        }
                    }
                    fn unpacking(&mut self, h: Obj) -> bool {
                        self.log.push(format!("s8:p{}+0x0", self.unit(h)));
                        self.seq(8, 309, 10) != 0
                    }
                    fn is_self(&self, h: Obj) -> bool {
                        h.1 == rd16(self.m, 0xa)
                    }
                    fn vector_dist(&mut self, dx: i32, dy: i32) -> i32 {
                        crate::world::vector_dist(dx, dy)
                    }
                    fn find_good(&mut self, cx: i32, cy: i32, who: i32, ecx: i32) -> i32 {
                        self.log.push(format!("s5:{ecx},{cx},{cy},{who},0,0"));
                        self.seq(5, 219, 80)
                    }
                    fn is_ally(&mut self, who: i32, owner: i32) -> bool {
                        self.log.push(format!("s6:r{who}+0x0,{owner}"));
                        self.seq(6, 299, 10) != 0
                    }
                    fn leader_id(&self, who: i32) -> i32 {
                        rd32(&self.p[6 + 2 * who as usize], 0)
                    }
                    fn relation(&self, a: i32, b: i32) -> i32 {
                        rd32(&self.p[7 + 2 * a as usize], 4 * b as usize)
                    }
                    fn calc_rare(&mut self, good: i32, ally: bool) {
                        let who = self.me_owner();
                        self.log
                            .push(format!("s7:r{who}+0x0,{good},p3+0x0,{}", i32::from(ally)));
                        for (k, v) in [1000, -77, 12345, 3, 999999, 0].into_iter().enumerate() {
                            wr32(&mut self.p[3], 4 * k, v);
                        }
                    }
                    fn rates_zero(&mut self) {
                        for k in 0..6 {
                            wr32(&mut self.p[3], 4 * k, 0);
                        }
                    }
                    fn rates_div(&mut self, n: i32) {
                        for k in 0..6 {
                            let v = rd32(&self.p[3], 4 * k);
                            wr32(&mut self.p[3], 4 * k, v.wrapping_div(n));
                        }
                    }
                    fn mask_set(&mut self, good: i32) {
                        let i = good.wrapping_sub(6);
                        let at = (0xc + (i >> 3)) as usize;
                        self.p[4][at] |= 1 << (i & 7);
                        wr32(&mut self.p[4], 8, 0);
                    }
                    fn tally_inc(&mut self, good: i32) {
                        // `tally[good - 6]`: below the buffer for `good < 6`, which lands in the allocator's slack.
                        let off = 4 * i64::from(good) - 0x18;
                        if off >= 0 {
                            let v = rd32(&self.p[5], off as usize);
                            wr32(&mut self.p[5], off as usize, v.wrapping_add(1));
                        }
                    }
                    fn out_rare(&mut self, v: i32) {
                        wr32(&mut self.p[0], 0, v);
                    }
                    fn out_flag(&mut self, v: i32) {
                        wr32(&mut self.p[1], 0, v);
                    }
                    fn out_good(&mut self, v: i32) {
                        self.p[2][0..2].copy_from_slice(&(v as i16).to_le_bytes());
                    }
                }
                let args = GatherArgs {
                    rare: a[0] != 0,
                    flag: a[1] != 0,
                    good: a[2] != 0,
                    mask: a[4] != 0,
                    tally: a[5] != 0,
                    p7: a[6],
                    p8: a[7],
                    x: a[8],
                    y: a[9],
                };
                let mut v = V {
                    a,
                    m,
                    c,
                    p,
                    log: &mut log,
                    turn: [0; 9],
                };
                let r = unit_calc_gather_steps(&mut v, &args);
                if v.log.is_empty() {
                    v.log.push("-".to_string());
                }
                i64::from(r)
            }
        };
        assert_eq!(got, want_eax, "calc_gather — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "calc_gather — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "calc_gather — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "calc_gather — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "calc_gather — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2500, "only {n} rows");
    eprintln!("UnitData::calc_gather@00609180: {n} rows agree");
}
