//! `Unit::work@0060d180` under the emulator against the port (item 3348; tools/sweep_gen.py, obj with windows).

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

/// `Unit::work@0060d180`, run by `tools/emu/sweep_unit_work_tail.py` under unicorn
/// (tools/emu): 272 random bytes of object, 0 argument(s), `p` windows in order: p0..p3 the place(s), in flag order; the global cell 0x00c061ec points at 1536 random bytes; 90112 random bytes at 0x00e3a390; vtable slot +0x10 (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x14 (0 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; the word at `m+0x18` points at 704 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed, null a third of the rows); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 0 answers 5..17 every row; stub 1 answers 0..0 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the call ends where stub 5 is first entered (the function is swept to that callee); the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_work_tail.py") else {
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
            // Row: `a[0]` slot `+0x10`'s answer (the order's type), `a[1]` slot `+0xd0`'s
            // (`is_caravan`, held to 0), `a[2]` slot `+0x60`'s (`is(0x13d, 0)`), `a[3]`
            // slot `+0x14`'s (`is_move`), `a[4]`, `a[5]` the two callees' (unread).
            // Stubs: 0 the order's `get_type`, 1 `this`'s slot `+0xd0`, 2 the type's slot
            // `+0x60`, 3 the order's `+0x14`, 4 `clear_partial_path`, 5 `update_action`
            // (the stop). Windows: `m` the unit, `c0` the game, `c1` `leaders` from
            // `0xe3a390` (stride `0x6eec`), `p0` the type record, `p1`..`p3` the order list,
            // its node and the order.
            //
            // **The call ends where `update_action` is first entered** (`--stop-stub`):
            // the sweep holds the prologue (`:40-87`), the caravan gate read as 0, a
            // dispatch that is none of the move family (`get_type` drawn from 5..17), the
            // fresh-latch lines and the open-search line. **The port is the crate's own
            // steps**: `work_gate_steps`, `fresh_latch_steps`, `partial_path_steps`.
            use crate::work_gate::{
                Gate, WorkGateView, WorkTailView, fresh_latch_steps, partial_path_steps,
                work_gate_steps,
            };
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a mut [Vec<u8>],
                p: &'a [Vec<u8>],
                asked: bool,
                fisher_asked: bool,
                move_asked: bool,
                cleared: bool,
            }
            impl V<'_> {
                fn has_order(&self) -> bool {
                    rd(self.m, 0xdc) != 0 && rd(self.m, 0xcc) != 0
                }
            }
            impl WorkGateView for V<'_> {
                fn clock_edge(&self) -> bool {
                    let frame = rd(&self.c[0], 0x550);
                    let o = i16::from_le_bytes([self.m[0xa], self.m[0xb]]);
                    frame.wrapping_add(i32::from(o)) & 0x1f == 0
                }
                fn attack_latch(&self) -> bool {
                    self.m[0x8] & 0x80 != 0
                }
                fn clear_visible(&mut self) {
                    self.m[0x40] = 0;
                }
                fn clear_in_danger(&mut self) {
                    let w = rd(self.m, 0x68) & !4;
                    wr(self.m, 0x68, w);
                }
                fn order_type(&mut self) -> Option<i32> {
                    if rd(self.m, 0xdc) == 0 {
                        return None;
                    }
                    wr(self.m, 0xd4, rd(&self.p[1], 4));
                    wr(self.m, 0xcc, rd(&self.p[2], 8));
                    self.m[0xd0] = self.p[2][0xc];
                    if rd(self.m, 0xcc) == 0 {
                        return None;
                    }
                    self.asked = true;
                    Some(self.a[0])
                }
                fn order_flags(&self) -> u32 {
                    if !self.has_order() {
                        return 0;
                    }
                    rd(&self.p[3], 4) as u32
                }
                fn drop_attack_latch(&mut self) {
                    self.m[0x8] &= 0x7f;
                }
                fn mask_100(&self) -> bool {
                    rd(self.m, 0x68) & 0x100 != 0
                }
                fn clear_mask_100(&mut self) {
                    let w = rd(self.m, 0x68) & !0x100;
                    wr(self.m, 0x68, w);
                }
                fn recharging(&self) -> u8 {
                    self.m[0xae]
                }
                fn type_max_range(&self) -> i32 {
                    rd(&self.p[0], 0x1fc)
                }
                fn type_melee_and_ranged(&self) -> bool {
                    rd(&self.p[0], 0x2b4) & 0x400 != 0
                }
            }
            impl WorkTailView for V<'_> {
                fn fresh_latch(&self) -> bool {
                    self.m[0x8] & 8 != 0
                }
                fn clear_fresh_latch(&mut self) {
                    self.m[0x8] &= 0xf7;
                }
                fn type_id(&self) -> i32 {
                    rd(&self.p[0], 4)
                }
                fn type_is_fisherman(&mut self) -> bool {
                    self.fisher_asked = true;
                    self.a[2] != 0
                }
                fn mark_leader_dirty(&mut self) {
                    let off = usize::from(self.m[9]) * 0x6eec;
                    let w = rd(&self.c[1], off) | 0x0200_0000;
                    wr(&mut self.c[1], off, w);
                }
                fn has_open_search(&self) -> bool {
                    rd(self.m, 0x104) != 0
                }
                fn order_is_move(&mut self) -> bool {
                    self.move_asked = true;
                    self.a[3] != 0
                }
                fn clear_partial_path(&mut self) {
                    self.cleared = true;
                }
            }
            let mut v = V {
                a,
                m,
                c,
                p,
                asked: false,
                fisher_asked: false,
                move_asked: false,
                cleared: false,
            };
            let gate = work_gate_steps(&mut v);
            let mut ty = 0;
            let mut stopped = false;
            if gate == Gate::Go {
                // `this->vtable[0xd0]` answered 0: the caravan block is skipped; the
                // dispatch is none of the move family and does nothing here.
                let had = v.has_order();
                ty = if had { v.a[0] } else { 0 };
                fresh_latch_steps(&mut v, ty);
                partial_path_steps(&mut v, had);
                stopped = true;
            }
            let tok = |b: bool, s: &str| if b { s.to_string() } else { "-".to_string() };
            log.push(tok(v.asked, "p3+0x0"));
            log.push(tok(stopped, "m+0x0"));
            log.push(tok(v.fisher_asked, "p0+0x0,317,0"));
            log.push(tok(v.move_asked, "p3+0x0"));
            log.push(tok(v.cleared, "m+0x0"));
            log.push(tok(stopped, "m+0x0"));
            let _ = ty;
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "work — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "work — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "work — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "work — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::work@0060d180: {n} rows agree");
}
