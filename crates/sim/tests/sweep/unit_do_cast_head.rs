//! `Unit::do_cast@005ebfe0` under the emulator against the port (item 3376; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::do_cast@005ebfe0`, run by `tools/emu/sweep_obj.py` on its row `unit_do_cast_head` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1024 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p6 the place(s), in flag order; argument 0 names 256 random bytes; vtable slot +0x7c (0 args) stubbed; callee 0x00676c40 (2 args) stubbed; vtable slot +0x48 (0 args) stubbed; stub 0's answer points at 64 random bytes less `0` (placed); the word at `0xe85ddc` points at 4 random bytes less `i32(p[1],0x20)*4` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `0xc061a8` points at 32 random bytes less `0` (placed); the word at `p4+0x10` points at 4 random bytes less `i32(p[1],0x20)*4` (placed); the word at `p5+0` points at 16 random bytes less `0` (placed); slot +0x48 laid as 0x00470590 or its stub, a flag after the answers; stub 2 answered from [0, 1] (90%); the stubbed vtable laid at `p0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the call ends where stub 1 is first entered (the function is swept to that callee); the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_do_cast_head") else {
        return;
    };
    let obj = super::Obj {
        name: "do_cast",
        full: "Unit::do_cast@005ebfe0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The head slice: the call ends where `SpellType::pay_cast_costs` (stub 1) is first entered,
            // so a row's last call is the payment. Window layout. `m` the caster (`+9` player, `+0xa` index);
            // `p[1]` the order record slot `+0x7c` answers (`+0x1c` paid, `+0x20` the spell index); `p[3]` the
            // type record `types.list` holds at that index; `p[6]` the spell-type row the payment asks.
            // The arguments are the stubs' answers then the `+0x48` flag (1: the image's own
            // `is_spell_type`, the inline range test; 0: the stub's answer `a[3]`).
            //
            // **The port is the crate's own head**, `crate::spells::do_cast_view::do_cast_head_steps`; here the
            // view is the emulated original's windows. The original is stopped at the payment, so the
            // paid mark after it is not written here (the next slice carries it).
            use crate::spells::do_cast_view::{DoCastHead, do_cast_head_steps};
            struct V<'a> {
                m: &'a [u8],
                p: &'a [Vec<u8>],
                a: &'a [i32],
                log: &'a mut Vec<String>,
            }
            impl DoCastHead for V<'_> {
                fn order_spell(&mut self) -> i32 {
                    self.log.push("s0:p0+0x0".into());
                    rd(&self.p[1], 0x20)
                }
                fn order_paid(&self) -> bool {
                    rd(&self.p[1], 0x1c) != 0
                }
                fn is_spell_type(&mut self, idx: i32) -> bool {
                    if self.a[4] == 1 {
                        (0x275..=0x2ab).contains(&idx)
                    } else {
                        self.log.push("s2:p3+0x0".into());
                        self.a[3] != 0
                    }
                }
                fn caster_index(&self) -> i32 {
                    i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]))
                }
                fn caster_owner(&self) -> i32 {
                    i32::from(self.m[9])
                }
                fn pay_cast_costs(&mut self, _idx: i32, index: i32, owner: i32) -> bool {
                    self.log.push(format!("s1:p6+0x0,{index},{owner}"));
                    self.a[2] != 0
                }
                fn mark_paid(&mut self) {}
            }
            let mut v = V {
                m: &*m,
                p: &*p,
                a,
                log: &mut *log,
            };
            let _ = do_cast_head_steps(&mut v);
            i64::from(a[2])
        }
    });
}
