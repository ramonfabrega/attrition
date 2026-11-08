//! `MiscAccess::coord_vis@00583a60` under the emulator against the port (item 2253; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `MiscAccess::coord_vis@00583a60`, run by `tools/emu/sweep_obj.py` on its row `misc_access_coord_vis_3` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 3 argument(s), callee 0x00583840 (0 args) stubbed; stub 0 answered from [0, 1, -1, 256, 305419776, 2147483392] (90%); `__cdecl`: the caller pops the arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("misc_access_coord_vis_3") else {
        return;
    };
    let obj = super::Obj {
        name: "coord_vis",
        full: "MiscAccess::coord_vis@00583a60",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let s = a[3];
            i64::from((s & !0xff) | i32::from(s == 0))
        }
    });
}
