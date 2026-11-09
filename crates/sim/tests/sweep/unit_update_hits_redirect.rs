//! `Unit::update_hits@0060e930` under the emulator against the port (item 3399; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::update_hits@0060e930`, run by `tools/emu/sweep_obj.py` on its row `unit_update_hits_redirect` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 344 random bytes of object, 1 argument(s), `p` windows in order: p0..p5 the place(s), in flag order, p6 stub 1's answer window; the global cell 0x00c0618c points at 96 random bytes; vtable slot +0xe8 (0 args) stubbed; vtable slot +0xa8 (0 args) stubbed; vtable slot +0x15c (1 args) stubbed; the word at `c0+0x14` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `c0+0x30` points at 32 random bytes less `0` (placed); the word at `p3+0` points at 64 random bytes less `0` (placed); the word at `p3+4` points at 64 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; stub 2 answered from [0, 1, 5, -7, 1000] (90%); stub 1 answers the address of 64 random bytes, a `p` window after the places; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_update_hits_redirect") else {
        return;
    };
    let obj = super::Obj {
        name: "update_hits",
        full: "Unit::update_hits@0060e930",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 600,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Row: `a[0]` the argument (the tail call is handed it), `a[1 + k]` stub `k`'s answer: 0 slot `+0xe8` (unread: the
            // base is laid every row), 1 slot `+0xa8` (the captain's object: its address is `p6`), 2 slot `+0x15c`, whose answer
            // is the function's. Windows: `c0` `GameAccess::objects` (`+0x14 + who * 0x1c` the unit array of player `who`);
            // `p0`/`p3` those arrays, `p1`/`p2`/`p4`/`p5` the objects they hold, `p6` what slot `+0xa8` answers.
            // The not-captain unit (the short at `+0x8e` clear in bit 15, its low bits the index) hands its whole answer to
            // that object's `update_hits`: slot `+0x15c` of `p6`, the argument unchanged.
            use crate::units::unit_update_hits_view::{UnitHitsK, UnitHitsView, unit_update_hits};
            struct V<'a> {
                a: &'a [i32],
                m: &'a [u8],
                log: &'a mut Vec<String>,
            }
            impl UnitHitsView for V<'_> {
                fn is_captain(&mut self) -> bool {
                    u16::from_le_bytes([self.m[0x8e], self.m[0x8f]]) >> 15 != 0
                }
                fn redirect(&mut self) -> i32 {
                    self.log.push(format!("s2:p6+0x0,{}", self.a[0]));
                    self.a[3]
                }
                fn base_hits(&mut self) -> i32 {
                    unreachable!()
                }
                fn is_marine(&mut self) -> bool {
                    unreachable!()
                }
                fn copper_bit(&mut self) -> bool {
                    unreachable!()
                }
                fn type_218(&mut self) -> i32 {
                    unreachable!()
                }
                fn trainer(&mut self) -> i32 {
                    unreachable!()
                }
                fn bananas_bit(&mut self) -> bool {
                    unreachable!()
                }
                fn has_tribe(&mut self, _n: i32) -> bool {
                    unreachable!()
                }
                fn attack_word(&mut self) -> i32 {
                    unreachable!()
                }
                fn is_type(&mut self, _n: i32) -> bool {
                    unreachable!()
                }
                fn type_index(&mut self) -> i32 {
                    unreachable!()
                }
                fn is_caravan(&mut self) -> bool {
                    unreachable!()
                }
                fn is_supply(&mut self) -> bool {
                    unreachable!()
                }
                fn patch_version(&mut self) -> i32 {
                    unreachable!()
                }
                fn is_gov_hero(&mut self) -> bool {
                    unreachable!()
                }
                fn ages_stored(&mut self) -> i32 {
                    unreachable!()
                }
                fn spy_upgrade(&mut self) -> i32 {
                    unreachable!()
                }
                fn is_hero(&mut self) -> bool {
                    unreachable!()
                }
                fn general_upgrade(&mut self) -> i32 {
                    unreachable!()
                }
                fn supply_upgrade(&mut self) -> i32 {
                    unreachable!()
                }
                fn link_of_self(&mut self) -> i32 {
                    unreachable!()
                }
                fn link_of(&mut self, _i: i32) -> i32 {
                    unreachable!()
                }
                fn store(&mut self, _at: Option<i32>, _h: i32) {
                    unreachable!()
                }
                fn stored(&mut self) -> i32 {
                    unreachable!()
                }
            }
            let _ = (&c, &p);
            let mut v = V {
                a,
                m,
                log: &mut *log,
            };
            i64::from(unit_update_hits(&UnitHitsK::default(), &mut v))
        }
    });
}
