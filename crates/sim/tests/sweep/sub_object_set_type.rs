//! `SubObject::set_type@00662450` under the emulator against the port (item 2946; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `SubObject::set_type@00662450`, run by `tools/emu/sweep_obj.py` on its row `sub_object_set_type` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 2 argument(s), vtable slot +0xb0 (0 args) stubbed; vtable slot +0x34 (0 args) stubbed; callee 0x00643070 (1 args) stubbed; callee 0x005e2920 (0 args) stubbed; callee 0x008e4780 (1 args) stubbed; the word at `0xc061f8` points at 32 random bytes less `0` (placed); the word at `p0+0x10` points at 2304 random bytes less `0` (placed); the word at `0xc0618c` points at 128 random bytes less `0` (placed); the word at `p2+0x14` points at 16 random bytes less `0` (placed); the word at `p2+0x30` points at 16 random bytes less `0` (placed); the word at `p3+0` points at 32 random bytes less `0` (placed); the word at `p3+4` points at 32 random bytes less `0` (placed); the word at `p3+8` points at 32 random bytes less `0` (placed); the word at `p4+0` points at 32 random bytes less `0` (placed); the word at `p4+4` points at 32 random bytes less `0` (placed); the word at `p4+8` points at 32 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `0xc0aedc` points at 16 random bytes less `0` (placed); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("sub_object_set_type") else {
        return;
    };
    let obj = super::Obj {
        name: "set_type",
        full: "SubObject::set_type@00662450",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 1200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // set_type: this+0x18 takes the objecttypes entry; a build-class type (0x19e..=0x21e) hands the object's Wall record
        // (slot 0xb0's answer, with the record) to Wall::update_gpiece, a unit-class type (0x32..=0x191) the unit's own
        // pointer to Unit::update_gpiece; the graphic load takes slot 0x34's answer either way.
        log.extend(["-"; 3].map(String::from));
        m[0x18..0x1c].copy_from_slice(&p[1][4 * a[0] as usize..4 * a[0] as usize + 4]);
        let owner = m[9] as usize;
        let idx = i16::from_le_bytes([m[10], m[11]]) as usize;
        if ((a[0] - 0x19e) as u32) <= 0x80 {
            log[0] = format!("{},p{}+0x0", a[2], 5 + owner * 3 + idx);
        } else if ((a[0] - 0x32) as u32) <= 0x15f {
            log[1] = rd(&p[11 + owner], 4 * idx).to_string();
        }
        log[2] = format!("m+0x0,{}", a[3]);
        0
    });
}
