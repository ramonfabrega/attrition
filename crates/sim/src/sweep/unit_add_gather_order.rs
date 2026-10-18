//! `Unit::add_gather_order@0061a5c0` under the emulator against the port (item 3268; tools/sweep_gen.py, obj with windows).

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

/// `Unit::add_gather_order@0061a5c0`, run by `tools/emu/sweep_unit_add_gather_order.py` under unicorn
/// (tools/emu): 256 random bytes of object, 3 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p26 the place(s), in flag order, p27 stub 3's answer window; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00cae5fc points at 64 random bytes; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x6c (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; callee 0x0062f640 (2 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x94 (0 args) stubbed; vtable slot +0x60 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x0046f790 (0 args) stubbed; callee 0x0046f290 (0 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; the word at `0xc0aec0+u8(m,9)*0x1c` points at 12 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed); the word at `c0+0x14+u8(m,9)*0x1c` points at 12 random bytes less `0` (placed); the word at `p6+0` points at 64 random bytes less `0` (placed); the word at `p6+4` points at 64 random bytes less `0` (placed); the word at `p6+8` points at 64 random bytes less `0` (placed); the word at `p7+0x18` points at 16 random bytes less `0` (placed); the word at `p8+0x18` points at 16 random bytes less `0` (placed); the word at `p9+0x18` points at 16 random bytes less `0` (placed); the word at `p7+0x20` points at 64 random bytes less `0` (placed); the word at `p8+0x20` points at 64 random bytes less `0` (placed); the word at `p9+0x20` points at 64 random bytes less `0` (placed); the word at `p7+0x28` points at 64 random bytes less `0` (placed); the word at `p8+0x28` points at 64 random bytes less `0` (placed); the word at `p9+0x28` points at 64 random bytes less `0` (placed); the word at `p13+0x18` points at 32 random bytes less `0` (placed); the word at `p14+0x18` points at 32 random bytes less `0` (placed); the word at `p15+0x18` points at 32 random bytes less `0` (placed); the word at `p16+0x18` points at 32 random bytes less `0` (placed); the word at `p17+0x18` points at 32 random bytes less `0` (placed); the word at `p18+0x18` points at 32 random bytes less `0` (placed); stub 4's answer points at 64 random bytes less `0` (placed); the word at `p25+4` points at 64 random bytes less `0` (placed); stub 6 answered from [0, 1, 1, 1] (90%); stub 9 answered from [0, 0, 1] (90%); stub 12 answered from [0, 1, 2, 3] (90%); stub 13 answered from [0, 1] (90%); stub 3 answers the address of 64 random bytes, a `p` window after the places; the stubbed vtable laid at `p27+0`; the word at `p16+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p17+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p18+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 10 answers 2 values in call order, printed after the answers; stub 11 answers 2 values in call order, printed after the answers; stub 14 answers 2 values in call order, printed after the answers; stub 10 answers 0..1 every row; stub 11 answers 0..1 every row; stub 14 answers 0..2 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 14 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 14 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 5 answers from its own `ecx`: the word at `+0x20`; stub 8 answers from its own `ecx`: the word at `+0x28`; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_add_gather_order.py") else {
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
            let calls_log = &mut log;
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a[0..3]` the building `b`, the queue position (2 `QUEUE_NEW`, 0 `QUEUE_FIRST`, 1 `QUEUE_LAST`) and the
            // action flag; `a[3 + i]` stub `i`'s answer, `a[19..21]` stub 10's two (slot `+0x60`, the `Build::vftable`
            // arm), `a[21..23]` stub 11's two (slot `+0xb8`), `a[23..25]` stub 14's two (`get_tregion`: the
            // building's, then the unit's). Stubs: 0 `close_orders`, 1 `clear_partial_path`, 2 `update_action`,
            // 3 `get_obj`, 4 slot `+0x6c` (the order, `p[25]`; its node `p[26]`, whose `+4` is the offset), 5 `+0xb0`
            // (`p[13 + b]`, whose `+0x18` is `p[19 + b]`), 6 `+0x90` (is_gather_type), 7 `Build::add_gatherer`, 8 `+0xac`
            // (`p[16 + b]`, its `+0x18` `p[22 + b]`), 9 `+0x94` (is_flat), 10/11 university and mine, 12
            // `transport_type`, 13 `can_ever_transport`, 14 `get_tregion`, 15 `LinkListBase::add`. Windows: `m` the unit,
            // `c[0]` the objects singleton, `c[1]` `div_3_table`; `p[0..2]` the leaders' flags, `p[2]` the types table
            // (`p[3..6]` its records), `p[6]` the objects table (`p[7..10]` the objects: `+0x10`/`+0x14` the point
            // `^ 0x63637`, `+0x30` the short, `+0x18` `p[10 + b]` whose `+4` is the order's `+0x1c`).
            //
            // **The port is the crate's own steps**, `crate::orders::add_gather_order_steps`, which `Sim::add_gather_order`
            // calls over its `GatherOrderView`; here the view is the emulated original's windows. The transport arm,
            // the order's bytes and the `+0x68` bits are the view's: this crate has none of them (SEAMs on the trait).
            use crate::orders::{GatherOrder, GatherOrderView, QueuePos, add_gather_order_steps};
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                b: usize,
                tails: Vec<String>,
                off: usize,
            }
            impl V<'_> {
                fn tile(&self, w: i32) -> i32 {
                    rd(&self.c[1], (((w ^ 0x63637) >> 6) as usize) * 4)
                }
                fn build_arm(&self) -> bool {
                    rd(&self.p[16 + self.b], 0) as u32 == 0xb42174
                }
                fn seq(&self, k: usize) -> bool {
                    // The Build arm asks slot `+0x60` (stub 10), the other `+0xb8` (stub 11): the first call is
                    // the University test, the second the Mine test.
                    let base = if self.build_arm() { 19 } else { 21 };
                    self.a[base + k] != 0
                }
            }
            impl GatherOrderView for V<'_> {
                fn restart(&mut self) {
                    wr(self.m, 0x68, rd(self.m, 0x68) & 0xfbffffffu32 as i32);
                    wr(self.m, 0xc0, 0);
                    self.log.push("s0:m+0x0,0".into());
                    self.log.push("s1:m+0x0".into());
                    self.log.push("s2:m+0x0".into());
                }
                fn take_order(&mut self) {
                    wr(self.m, 0x68, rd(self.m, 0x68) & 0xfffffbffu32 as i32);
                    self.log.push("s3:m+0x0,7".into());
                    let b = self.b;
                    let short = [self.p[7 + b][0x30], self.p[7 + b][0x31]];
                    let node4 = rd(&self.p[10 + b], 4);
                    let player = i32::from(self.m[9]);
                    let o = &mut self.p[25];
                    wr(o, 8, b as i32);
                    wr(o, 0xc, player);
                    o[0x10..0x12].copy_from_slice(&short);
                    wr(o, 0x14, -1);
                    wr(o, 0x18, -1);
                    wr(o, 0x1c, node4);
                    o[0x25] = 0;
                    o[0x26] = 0;
                }
                fn is_gather_type(&mut self) -> bool {
                    self.a[3 + 6] != 0
                }
                fn add_gatherer(&mut self) {
                    let id = i16::from_le_bytes([self.m[0xa], self.m[0xb]]);
                    self.log
                        .push(format!("s7:p{}+0x0,{},{}", 16 + self.b, id, self.m[9]));
                }
                fn is_flat(&mut self) -> bool {
                    self.a[3 + 9] != 0
                }
                fn is_university(&mut self) -> bool {
                    self.seq(0)
                }
                fn is_mine(&mut self) -> bool {
                    self.seq(1)
                }
                fn lay(&mut self, order: GatherOrder, action: bool) {
                    let off = rd(&self.p[26], 4) as usize;
                    let o = &mut self.p[25];
                    if order.dist_mod != 0 {
                        o[0x25] = 1;
                        o[0x26] = order.dist_mod as u8;
                    }
                    if action {
                        o[off + 8] |= 4;
                    } else {
                        o[off + 8] &= 0xfb;
                    }
                    self.off = off;
                }
                fn link(&mut self) {
                    self.log.push(format!("s15:m+0xcc,p25+{:#x}", 4 + self.off));
                }
                fn transport_arm(&mut self) {
                    let o = &self.p[7 + self.b];
                    let (ox, oy) = (self.tile(rd(o, 0x10)), self.tile(rd(o, 0x14)));
                    let (ux, uy) = (self.tile(rd(self.m, 0x10)), self.tile(rd(self.m, 0x14)));
                    // `[0xc06188]`, the world, as the image holds it.
                    self.log.push("s14:12621800,sp-0xc,sp-0x8".into());
                    self.log.push("s14:12621800,sp-0x18,sp-0x4".into());
                    let le = |v: i32| {
                        v.to_le_bytes()
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>()
                    };
                    self.tails.push(format!("{}/{}", le(ox), le(ux)));
                    self.tails.push(format!("{}/{}", le(oy), le(uy)));
                    if self.a[23] != self.a[24] {
                        let lead = rd(&self.p[usize::from(self.m[9])], 0) as u32;
                        let level = if lead & 0x100 != 0 {
                            3
                        } else if lead & 0x200 != 0 {
                            2
                        } else {
                            ((lead >> 10) & 1) as i32
                        };
                        let ty = i16::from_le_bytes([self.m[0xa], self.m[0xb]]);
                        let ecx = format!("p{}+0x0", 3 + i32::from(ty));
                        self.log.push(format!("s12:{ecx}"));
                        if level >= self.a[3 + 12] {
                            self.log.push(format!("s13:{ecx}"));
                            if self.a[3 + 13] != 0 {
                                wr(self.m, 0x68, rd(self.m, 0x68) | 0x800000);
                            }
                        }
                    }
                }
                fn update_action(&mut self) {
                    self.log.push("s2:m+0x0".into());
                }
            }
            let pos = match a[1] {
                2 => QueuePos::New,
                0 => QueuePos::First,
                _ => QueuePos::Last,
            };
            let mut v = V {
                a,
                m,
                c,
                p,
                log: &mut *calls_log,
                b: a[0] as usize,
                tails: Vec::new(),
                off: 0,
            };
            add_gather_order_steps(&mut v, a[0] as usize, pos, a[2] != 0);
            let tails = std::mem::take(&mut v.tails);
            calls_log.extend(tails);
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "add_gather_order — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_gather_order — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_gather_order — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "add_gather_order — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::add_gather_order@0061a5c0: {n} rows agree");
}
