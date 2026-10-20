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

/// `Unit::work@0060d180`, run by `tools/emu/sweep_unit_work.py` under unicorn
/// (tools/emu): 272 random bytes of object, 0 argument(s), `p` windows in order: p0..p3 the place(s), in flag order; the global cell 0x00c061ec points at 1536 random bytes; vtable slot +0x10 (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; the word at `m+0x18` points at 704 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed, null a third of the rows); stub 0 answered from [0, 1, 3, 10, 10, 5, 9] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the call ends where stub 1 is first entered (the function is swept to that callee); the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_work.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Row: `a[0]` slot `+0x10`'s answer (the order's type), `a[1]` slot `+0xd0`'s.
            // Windows: `m` the unit, `c0` the game (`+0x550` the frame), `p0` the unit's
            // type record, `p1` the order list (`+4` its current node), `p2` the node
            // (`+8` the order, `+0xc` its iterator byte), `p3` the order.
            //
            // **The call ends where `this->vtable[0xd0]` is first entered**
            // (`--stop-stub`): the sweep holds `Unit::work`'s prologue (`:40-87`) and
            // nothing after it. **The port is the crate's own steps**,
            // `crate::work_gate::work_gate_steps`, which `Sim::work` runs over its
            // `SimWorkGate`; here the view is the emulated original's windows.
            use crate::work_gate::{Gate, WorkGateView, work_gate_steps};
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a [Vec<u8>],
                p: &'a [Vec<u8>],
                asked: bool,
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
                    // `current = list->head->data`: the iterator refresh.
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
                    if rd(self.m, 0xdc) == 0 || rd(self.m, 0xcc) == 0 {
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
            let mut v = V {
                a,
                m,
                c,
                p,
                asked: false,
            };
            let gate = work_gate_steps(&mut v);
            // The recorded columns: slot `+0x10` (the order's type) then slot
            // `+0xd0` (the stop), a `-` for a slot not entered.
            log.push(if v.asked { "p3+0x0" } else { "-" }.to_string());
            log.push(if gate == Gate::Go { "m+0x0" } else { "-" }.to_string());
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
            assert_eq!(w, &want[2 + i], "work — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::work@0060d180: {n} rows agree");
}
