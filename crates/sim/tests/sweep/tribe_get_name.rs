//! `Tribe::get_name@006f0530` under the emulator against the port (item 2336; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Tribe::get_name@006f0530`, run by `tools/emu/sweep_obj.py` on its row `tribe_get_name` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 2 argument(s), argument 0 names 16 random bytes; callee 0x00a1d590 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("tribe_get_name") else {
        return;
    };
    let obj = super::Obj {
        name: "get_name",
        full: "Tribe::get_name@006f0530",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The name (+0x18) copied into the caller's storage (argument 0) by the `String`
            // copy constructor (stub 0, `ecx` the storage), and the storage answered.
            log.push("p0+0x0,m+0x18".into());
            i64::from(a[0])
        }
    });
}
