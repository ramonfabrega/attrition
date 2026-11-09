//! `Group::action_gather@00700b90` under the emulator against the port (item 3386; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Group::action_gather@00700b90`, run by `tools/emu/sweep_obj.py` on its row `group_action_gather_head` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2272 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 the place(s), in flag order; 4 random bytes at 0x00cc02f8 (each word in 1..1); 4 random bytes at 0x00ed6574 (each word in 0..3); 4 random bytes at 0x00ed6590 (each word in 0..3); vtable slot +0x10 (4 args) stubbed; vtable slot +0x14 (0 args) stubbed; callee 0x00713e80 (1 args) stubbed; the word at `0xed6580+0x1c*0` points at 16 random bytes less `0` (placed); the word at `0xed6580+0x1c*1` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 2816 bytes under the call is zeroed before each row; the call ends where stub 1 is first entered (the function is swept to that callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_action_gather_head") else {
        return;
    };
    let obj = super::Obj {
        name: "action_gather",
        full: "Group::action_gather@00700b90",
        cells: 3,
        calls: true,
        eax: false,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The head slice: the call ends where the group's refresh (slot `+0x14`, stub 1) is entered.
            // Window layout. `m` the group (`+0x4a` player); `c[0]` `ignore_orders`, `c[1]` and `c[2]`
            // players 0 and 1's ignore-orders counts; `p[who]` the elements of
            // `objects_ignoring_orders[who]` (players 0 and 1 are laid; a player of 8 or more skips the sweep).
            // The scratch group's first `clear` (stub 2) comes before the sweep.
            //
            // **The port is the crate's own head**: `action_gather_steps` begins with `list_clear` and
            // `crate::groups::garrison_view::garrison_head`; here the view is the emulated original's windows.
            use crate::groups::action_gather_view::{GarrisonHead, garrison_head};
            struct V<'a> {
                m: &'a [u8],
                c: &'a [Vec<u8>],
                p: &'a [Vec<u8>],
                log: &'a mut Vec<String>,
            }
            fn rd32(w: &[u8], o: usize) -> i32 {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            }
            impl GarrisonHead for V<'_> {
                fn who(&self) -> i32 {
                    i32::from(self.m[0x4a])
                }
                fn ignore_orders(&self) -> bool {
                    rd32(&self.c[0], 0) != 0
                }
                fn ignoring_count(&self, who: i32) -> i32 {
                    rd32(&self.c[1 + who as usize], 0)
                }
                fn ignoring_at(&self, who: i32, i: i32) -> i32 {
                    rd32(&self.p[who as usize], 4 * i as usize)
                }
                fn drop_ignored(&mut self, o: i32, who: i32) {
                    self.log.push(format!("s0:m+0x0,{o},{who},0,0"));
                }
                fn refresh(&mut self) {
                    self.log.push("s1:m+0x0".into());
                }
            }
            let mut v = V {
                m: &*m,
                c: &*c,
                p: &*p,
                log: &mut *log,
            };
            v.log.push("s2:sp-0xa20,-1".into());
            garrison_head(&mut v);
            0
        }
    });
}
