//! `Group::action_form@00707220` under the emulator against the port (item 3379; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Group::action_form@00707220`, run by `tools/emu/sweep_obj.py` on its row `group_action_form` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2272 random bytes of object, 5 argument(s), `p` windows in order: p0..p19 the place(s), in flag order; the global cell 0x00c0618c points at 768 random bytes; vtable slot +0x14 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xc0 (0 args) stubbed; callee 0x0070c450 (0 args) stubbed; callee 0x0070ccb0 (1 args) stubbed; callee 0x0070beb0 (1 args) stubbed; callee 0x0046d3f0 (0 args) stubbed; callee 0x0070e520 (1 args) stubbed; callee 0x0070d0c0 (1 args) stubbed; callee 0x0070e620 (1 args) stubbed; callee 0x0046f600 (0 args) stubbed; callee 0x0070cc90 (0 args) stubbed; callee 0x0070c5d0 (3 args) stubbed; callee 0x0070fba0 (10 args) stubbed; the word at `c0+0x14+0x1c*0` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p10+0` points at 192 random bytes less `0` (placed); the word at `p10+4` points at 192 random bytes less `0` (placed); the word at `p10+8` points at 192 random bytes less `0` (placed); the word at `p10+12` points at 192 random bytes less `0` (placed); the word at `p11+0` points at 192 random bytes less `0` (placed); the word at `p11+4` points at 192 random bytes less `0` (placed); the word at `p11+8` points at 192 random bytes less `0` (placed); the word at `p11+12` points at 192 random bytes less `0` (placed); stub 3 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 0] (90%); stub 2 answered from [0, 0, 0, 1] (90%); stub 1 answers 6 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 3 answers 2 values in call order, printed after the answers; stub 4 answers 4 values in call order, printed after the answers; stub 5 answers 3 values in call order, printed after the answers; stub 11 answers 2 values in call order, printed after the answers; stub 4 answers -1..3 every row; stub 5 answers -7..6 every row; stub 11 answers -1..3 every row; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 12 writes a value in [-1, 0, 100, 400, 4096] through its argument 0, printed after the answers (`o` in a place); stub 12 writes a value in [-1, 0, 100, 400, 4096] through its argument 1, printed after the answers (`o` in a place); stub 6 writes 0 (4 bytes) at `ecx+0x10` each call; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_action_form") else {
        return;
    };
    let obj = super::Obj {
        name: "action_form",
        full: "Group::action_form@00707220",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 3000,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Window layout. `m` the group (`+0xc` count, `+0x10` form, `+0x18`/`+0x1c` `ox`/`oy`, `+0x24` `o_angle`,
            // `+0x49` building byte, `+0x4a` player, `+0x8cc` the member shorts); `c[0]` the objects record; `p[0..2]` the
            // players' arrays of objects, `p[2 + 4 * who + i]` the objects, `p[10..12]` the `units` arrays,
            // `p[12 + 4 * who + i]` the units. `a`: 0 form, 1 rotate, 2 queue, 3 the dead fourth (the leftover `ecx`),
            // 4 inserting; 5.. the stubs' own answers (14 of them, void ones random), then the `get_loc_to` writes
            // (`x` at 19, `y` at 20) and each answering stub's run: slot `+0x18` 21..27, slot `+0xc0` 27..35,
            // `is_on_map` 35..37, `find_leader` 37..41, `get_form_option` 41..44, `get_unit` 44..46.
            //
            // **The port is the crate's own steps**, `crate::groups::form_view::action_form_steps`, which
            // `Sim::group_action_form` runs over its `SimActionForm`; here the view is the emulated original's windows.
            use crate::groups::form_view::{ActionFormView, action_form_steps};
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                used: [usize; 46],
            }
            impl V<'_> {
                fn seq(&mut self, base: usize, n: usize) -> i32 {
                    let k = self.used[base].min(n - 1);
                    self.used[base] += 1;
                    self.a[base + k]
                }
                fn who(&self) -> usize {
                    self.m[0x4a] as usize
                }
                fn unit(&mut self, i: i32) -> &mut Vec<u8> {
                    let w = self.who();
                    &mut self.p[12 + 4 * w + i as usize]
                }
            }
            impl ActionFormView for V<'_> {
                fn is_on_map(&mut self) -> bool {
                    self.seq(35, 2) != 0
                }
                fn refresh(&mut self) {}
                fn buildings(&mut self) -> bool {
                    self.m[0x49] != 0
                }
                fn count(&mut self) -> i32 {
                    rd(self.m, 0xc)
                }
                fn find_leader(&mut self) -> i32 {
                    self.seq(37, 4)
                }
                fn unit_plane(&mut self, _i: i32) -> bool {
                    self.seq(27, 8) != 0
                }
                fn get_form_option(&mut self) -> i32 {
                    self.seq(41, 3)
                }
                fn set_group_form(&mut self, v: i32) {
                    wr(self.m, 0x10, v);
                }
                fn get_unit(&mut self) -> i32 {
                    self.seq(44, 2)
                }
                fn unit_form(&mut self, i: i32) -> i32 {
                    i32::from(self.unit(i)[0xaa] as i8)
                }
                fn member(&mut self, k: i32) -> i32 {
                    let o = 0x8cc + 2 * k as usize;
                    i32::from(i16::from_le_bytes([self.m[o], self.m[o + 1]]))
                }
                fn obj_unit(&mut self, _i: i32) -> bool {
                    self.seq(21, 6) != 0
                }
                fn set_unit_form(&mut self, i: i32, v: i32) {
                    self.unit(i)[0xaa] = v as u8;
                }
                fn new_list(&mut self) {
                    self.log.push("s6:sp-0x44".into());
                }
                fn set_up_insert(&mut self) {
                    self.log.push("s7:m+0x0,sp-0x44".into());
                }
                fn action_halt(&mut self) {
                    self.log.push("s8:m+0x0,0".into());
                }
                fn list_pending(&mut self) -> bool {
                    false // the ctor stub writes `0` at `list + 0x10`
                }
                fn finish_insert(&mut self) {
                    self.log.push("s9:m+0x0,sp-0x44".into());
                }
                fn clear_list(&mut self) {
                    self.log.push("s10:sp-0x40".into());
                }
                fn loc_to(&mut self) -> Option<(i32, i32)> {
                    Some((self.a[19], self.a[20]))
                }
                fn o_xy(&mut self) -> (i32, i32) {
                    (rd(self.m, 0x18), rd(self.m, 0x1c))
                }
                fn o_angle(&mut self) -> i32 {
                    rd(self.m, 0x24)
                }
                fn unit_heading(&mut self, i: i32) -> i32 {
                    rd(self.unit(i), 0x50)
                }
                fn action_move_to(&mut self, b: [i32; 10]) {
                    let t: Vec<String> = b.iter().map(|x| x.to_string()).collect();
                    self.log.push(format!("s13:m+0x0,{}", t.join(",")));
                }
            }
            let mut v = V {
                a,
                m,
                p,
                log: &mut *log,
                used: [0; 46],
            };
            action_form_steps(&mut v, a[0], a[1], a[2], a[4]);
            if log.is_empty() {
                log.push("-".into());
            }
            0
        }
    });
}
